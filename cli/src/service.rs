//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use std::sync::Arc;
use std::time::Duration;

use sc_client_api::{ExecutorProvider, RemoteBackend};
use sc_finality_grandpa::FinalityProofProvider as GrandpaFinalityProofProvider;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
use sp_inherents::InherentDataProviders;
use sp_runtime::traits::Block as BlockT;

use rio_executor::Executor;
use rio_primitives::Block;
use riochain_runtime::{self, RuntimeApi};
use sc_network::NetworkService;

type FullClient = sc_service::TFullClient<Block, RuntimeApi, Executor>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

pub fn new_partial(
    config: &Configuration,
) -> Result<
    sc_service::PartialComponents<
        FullClient,
        FullBackend,
        FullSelectChain,
        sp_consensus::DefaultImportQueue<Block, FullClient>,
        sc_transaction_pool::FullPool<Block, FullClient>,
        (
            impl Fn(rio_rpc::DenyUnsafe, sc_rpc::SubscriptionTaskExecutor) -> rio_rpc::IoHandler,
            (
                sc_finality_grandpa::GrandpaBlockImport<
                    FullBackend,
                    Block,
                    FullClient,
                    FullSelectChain,
                >,
                sc_finality_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
            ),
            (
                sc_finality_grandpa::SharedVoterState,
                Arc<GrandpaFinalityProofProvider<FullBackend, Block>>,
            ),
        ),
    >,
    ServiceError,
> {
    let (client, backend, keystore, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config)?;
    let client = Arc::new(client);

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        // std::sync::Arc::new(pool_api),
        config.prometheus_registry(),
        task_manager.spawn_handle(),
        client.clone(),
    );

    let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
        client.clone(),
        &(client.clone() as Arc<_>),
        select_chain.clone(),
    )?;
    let justification_import = grandpa_block_import.clone();

    let aura_block_import = sc_consensus_aura::AuraBlockImport::<_, _, _, AuraPair>::new(
        grandpa_block_import.clone(),
        client.clone(),
    );

    let inherent_data_providers = sp_inherents::InherentDataProviders::new();
    let import_queue = sc_consensus_aura::import_queue::<_, _, _, AuraPair, _, _>(
        sc_consensus_aura::slot_duration(&*client)?,
        aura_block_import,
        Some(Box::new(justification_import)),
        None,
        client.clone(),
        inherent_data_providers.clone(),
        &task_manager.spawn_handle(),
        config.prometheus_registry(),
        sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
    )?;

    let import_setup = (grandpa_block_import, grandpa_link);

    let (rpc_extensions_builder, rpc_setup) = {
        let (_, grandpa_link) = &import_setup;

        let justification_stream = grandpa_link.justification_stream();
        let shared_authority_set = grandpa_link.shared_authority_set().clone();
        let shared_voter_state = sc_finality_grandpa::SharedVoterState::empty();
        let finality_proof_provider =
            GrandpaFinalityProofProvider::new_for_service(backend.clone(), client.clone());

        let rpc_setup = (shared_voter_state.clone(), finality_proof_provider.clone());

        let client = client.clone();
        let pool = transaction_pool.clone();

        let rpc_extensions_builder = Box::new(move |deny_unsafe, subscription_executor| {
            let deps = rio_rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                deny_unsafe,
                grandpa: rio_rpc::GrandpaDeps {
                    shared_voter_state: shared_voter_state.clone(),
                    shared_authority_set: shared_authority_set.clone(),
                    justification_stream: justification_stream.clone(),
                    subscription_executor,
                    finality_provider: finality_proof_provider.clone(),
                },
            };

            rio_rpc::create_full(deps)
        });

        (rpc_extensions_builder, rpc_setup)
    };
    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        keystore,
        select_chain,
        import_queue,
        transaction_pool,
        inherent_data_providers,
        other: (rpc_extensions_builder, import_setup, rpc_setup),
    })
}

pub struct NewFullBase {
    pub task_manager: TaskManager,
    pub inherent_data_providers: InherentDataProviders,
    pub client: Arc<FullClient>,
    pub network: Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
    pub network_status_sinks: sc_service::NetworkStatusSinks<Block>,
    pub transaction_pool: Arc<sc_transaction_pool::FullPool<Block, FullClient>>,
}

/// Creates a full service from the configuration.
pub fn new_full_base(config: Configuration) -> Result<NewFullBase, ServiceError> {
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore,
        select_chain,
        transaction_pool,
        inherent_data_providers,
        other: (rpc_extensions_builder, (block_import, grandpa_link), rpc_setup),
    } = new_partial(&config)?;

    let (shared_voter_state, finality_proof_provider) = rpc_setup;

    let (network, network_status_sinks, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            on_demand: None,
            block_announce_validator_builder: None,
            finality_proof_request_builder: None,
            finality_proof_provider: Some(finality_proof_provider.clone()),
        })?;

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            backend.clone(),
            task_manager.spawn_handle(),
            client.clone(),
            network.clone(),
        );
    }

    let role = config.role.clone();
    let force_authoring = config.force_authoring;
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let prometheus_registry = config.prometheus_registry().cloned();
    let telemetry_connection_sinks = sc_service::TelemetryConnectionSinks::default();

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        config,
        backend: backend.clone(),
        client: client.clone(),
        keystore: keystore.clone(),
        network: network.clone(),
        rpc_extensions_builder: Box::new(rpc_extensions_builder),
        transaction_pool: transaction_pool.clone(),
        task_manager: &mut task_manager,
        on_demand: None,
        remote_blockchain: None,
        telemetry_connection_sinks: telemetry_connection_sinks.clone(),
        network_status_sinks: network_status_sinks.clone(),
        system_rpc_tx,
    })?;

    if role.is_authority() {
        let proposer = sc_basic_authorship::ProposerFactory::new(
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
        );

        let can_author_with =
            sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

        let aura = sc_consensus_aura::start_aura::<_, _, _, _, _, AuraPair, _, _, _>(
            sc_consensus_aura::slot_duration(&*client)?,
            client.clone(),
            select_chain,
            block_import,
            proposer,
            network.clone(),
            inherent_data_providers.clone(),
            force_authoring,
            keystore.clone(),
            can_author_with,
        )?;

        // the AURA authoring task is considered essential, i.e. if it
        // fails we take down the service with it.
        task_manager
            .spawn_essential_handle()
            .spawn_blocking("aura", aura);
    }

    // if the node isn't actively participating in consensus then it doesn't
    // need a keystore, regardless of which protocol we use below.
    let keystore = if role.is_authority() {
        Some(keystore as sp_core::traits::BareCryptoStorePtr)
    } else {
        None
    };

    let grandpa_config = sc_finality_grandpa::Config {
        // FIXME #1578 make this available through chainspec
        gossip_duration: Duration::from_millis(333),
        justification_period: 512,
        name: Some(name),
        observer_enabled: false,
        keystore,
        is_authority: role.is_network_authority(),
    };

    if enable_grandpa {
        // start the full GRANDPA voter
        // NOTE: non-authorities could run the GRANDPA observer protocol, but at
        // this point the full voter should provide better guarantees of block
        // and vote data availability than the observer. The observer has not
        // been tested extensively yet and having most nodes in a network run it
        // could lead to finality stalls.
        let grandpa_config = sc_finality_grandpa::GrandpaParams {
            config: grandpa_config,
            link: grandpa_link,
            network: network.clone(),
            inherent_data_providers: inherent_data_providers.clone(),
            telemetry_on_connect: Some(telemetry_connection_sinks.on_connect_stream()),
            voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
            prometheus_registry,
            shared_voter_state,
        };

        // the GRANDPA voter task is considered infallible, i.e.
        // if it fails we take down the service with it.
        task_manager.spawn_essential_handle().spawn_blocking(
            "grandpa-voter",
            sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
        );
    } else {
        sc_finality_grandpa::setup_disabled_grandpa(
            client.clone(),
            &inherent_data_providers,
            network.clone(),
        )?;
    }

    network_starter.start_network();
    Ok(NewFullBase {
        task_manager,
        inherent_data_providers,
        client,
        network,
        network_status_sinks,
        transaction_pool,
    })
}

/// Builds a new service for a full client.
pub fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {
    new_full_base(config).map(|NewFullBase { task_manager, .. }| task_manager)
}

/// Builds a new service for a light client.
pub fn new_light(config: Configuration) -> Result<TaskManager, ServiceError> {
    let (client, backend, keystore, mut task_manager, on_demand) =
        sc_service::new_light_parts::<Block, RuntimeApi, Executor>(&config)?;

    let transaction_pool = Arc::new(sc_transaction_pool::BasicPool::new_light(
        config.transaction_pool.clone(),
        config.prometheus_registry(),
        task_manager.spawn_handle(),
        client.clone(),
        on_demand.clone(),
    ));

    let grandpa_block_import = sc_finality_grandpa::light_block_import(
        client.clone(),
        backend.clone(),
        &(client.clone() as Arc<_>),
        Arc::new(on_demand.checker().clone()),
    )?;

    let finality_proof_import = grandpa_block_import.clone();
    let finality_proof_request_builder =
        finality_proof_import.create_finality_proof_request_builder();

    let import_queue = sc_consensus_aura::import_queue::<_, _, _, AuraPair, _, _>(
        sc_consensus_aura::slot_duration(&*client)?,
        grandpa_block_import,
        None,
        Some(Box::new(finality_proof_import)),
        client.clone(),
        InherentDataProviders::new(),
        &task_manager.spawn_handle(),
        config.prometheus_registry(),
        sp_consensus::NeverCanAuthor,
    )?;

    let finality_proof_provider =
        GrandpaFinalityProofProvider::new_for_service(backend.clone(), client.clone());

    let (network, network_status_sinks, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            on_demand: Some(on_demand.clone()),
            block_announce_validator_builder: None,
            finality_proof_request_builder: Some(finality_proof_request_builder),
            finality_proof_provider: Some(finality_proof_provider),
        })?;
    network_starter.start_network();

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            backend.clone(),
            task_manager.spawn_handle(),
            client.clone(),
            network.clone(),
        );
    }

    let light_deps = rio_rpc::LightDeps {
        remote_blockchain: backend.remote_blockchain(),
        fetcher: on_demand.clone(),
        client: client.clone(),
        pool: transaction_pool.clone(),
    };

    let rpc_extensions = rio_rpc::create_light(light_deps);

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        on_demand: Some(on_demand),
        remote_blockchain: Some(backend.remote_blockchain()),
        rpc_extensions_builder: Box::new(sc_service::NoopRpcExtensionBuilder(rpc_extensions)),
        client: client.clone(),
        transaction_pool: transaction_pool.clone(),
        config,
        keystore,
        backend,
        network_status_sinks,
        system_rpc_tx,
        network: network.clone(),
        telemetry_connection_sinks: sc_service::TelemetryConnectionSinks::default(),
        task_manager: &mut task_manager,
    })?;

    Ok(task_manager)
}
