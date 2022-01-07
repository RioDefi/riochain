use std::collections::HashMap;
use std::convert::TryInto;
use std::str::FromStr;

use hex_literal::hex;
use serde::{Deserialize, Serialize};
use serde_json as json;

use sc_chain_spec::ChainSpecExtension;
use sc_network::config::MultiaddrWithPeerId;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
// use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::crypto::UncheckedInto;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    Perbill,
};

use rio_primitives::{AccountId, Balance, Block, CurrencyId, Signature};
use riochain_runtime::constants::currency::*;
use riochain_runtime::{
    self, rio_assets, AuraId, Bip32, DepositAddrInfo, Forcing, ImOnlineId, StakerStatus,
};
use riochain_runtime::{
    BalancesConfig, CouncilConfig, DemocracyConfig, GenesisConfig, ImOnlineConfig, IndicesConfig,
    OracleConfig, RioAssetsConfig, RioGatewayConfig, RioPaymentFeeConfig, RioRootConfig,
    SessionConfig, SessionKeys, SocietyConfig, StakingConfig, SudoConfig, SystemConfig,
    TechnicalCommitteeConfig, VestingConfig, WASM_BINARY,
};

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Block numbers with known hashes.
    pub fork_blocks: sc_client_api::ForkBlocks<Block>,
    /// Known bad block hashes.
    pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Alternative {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
    /// The Rio network.
    Testnet,
    Beta,
    Mainnet,
}

pub struct SpecInfo {
    name: &'static str,
    id: &'static str,
    chain_type: ChainType,
    protocol_id: Option<&'static str>,
    pub properties: Option<sc_service::Properties>,
}

lazy_static::lazy_static! {
    pub static ref CHAIN_TYPE: HashMap<Alternative, SpecInfo> = {
        let mut m = HashMap::new();
        // value is (name, id, properties)
        m.insert(
            Alternative::Development,
            SpecInfo {
                name: "Development",
                id: "dev",
                chain_type: ChainType::Development,
                protocol_id: Some("rio_dev"),
                properties: Some(json::from_str(DEFAULT_PROPERTIES_TESTNET).unwrap()),
            });
        m.insert(
            Alternative::LocalTestnet,
            SpecInfo {
                name: "Local Testnet",
                id: "local_testnet",
                chain_type: ChainType::Local,
                protocol_id: Some("rio_local"),
                properties: Some(json::from_str(DEFAULT_PROPERTIES_TESTNET).unwrap()),
            });
        m.insert(
            Alternative::Testnet,
            SpecInfo {
                name: "RioChain Testnet",
                id: "moniker",
                chain_type: ChainType::Custom("rio testnet".to_string()),
                protocol_id: Some("rio_moniker"),
                properties: Some(json::from_str(DEFAULT_PROPERTIES_TESTNET).unwrap()),
            });
        m.insert(
            Alternative::Beta,
            SpecInfo {
                name: "RioChain Beta",
                id: "beta",
                chain_type: ChainType::Live,
                protocol_id: Some("rio_beta"),
                properties: Some(json::from_str(DEFAULT_PROPERTIES_MAINNET).unwrap()),
            });
        m.insert(
            Alternative::Mainnet,
            SpecInfo {
                name: "RioChain CC-1",
                id: "mainnet",
                chain_type: ChainType::Live,
                protocol_id: Some("rio_mainnet"),
                properties: Some(json::from_str(DEFAULT_PROPERTIES_MAINNET).unwrap()),
            });
        m
    };
}

const DEFAULT_PROPERTIES_MAINNET: &str = r#"
{
"tokenSymbol": "RFUEL",
"tokenDecimals": 12,
"ss58Format": 42
}
"#;

const DEFAULT_PROPERTIES_TESTNET: &str = r#"
{
"tokenSymbol": "RFUEL",
"tokenDecimals": 12,
"ss58Format": 42
}
"#;

// pub fn get_alternative_from_id(id: &str) -> Result<Alternative, String> {
//     for (k, s) in CHAIN_TYPE.iter() {
//         if s.id == id {
//             return Ok(*k);
//         }
//     }
//     let ids = CHAIN_TYPE.iter().map(|(_, s)| s.id).collect::<Vec<_>>();
//     Err(format!(
//         "no support id in current `Alternative`:{:}|current support:{:?}",
//         id, ids
//     ))
// }

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

type AuthorityKeysTuple = (
    AccountId, // ValidatorId
    AccountId, // (SessionKey)
    AuraId,
    GrandpaId,
    ImOnlineId,
);

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(seed: &str) -> AuthorityKeysTuple {
    (
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_from_seed::<AuraId>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<ImOnlineId>(seed),
    )
}

fn session_keys(aura: AuraId, grandpa: GrandpaId, im_online: ImOnlineId) -> SessionKeys {
    SessionKeys {
        aura,
        grandpa,
        im_online,
    }
}

impl Alternative {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        let spec: &SpecInfo = CHAIN_TYPE
            .get(&self)
            .ok_or("not support for this Alternative")?;
        let wasm_binary = WASM_BINARY.ok_or("runtime wasm binary not available".to_string())?;

        Ok(match self {
            Alternative::Development => ChainSpec::from_genesis(
                spec.name,
                spec.id,
                spec.chain_type.clone(),
                move || {
                    testnet_genesis(
                        wasm_binary,
                        vec![authority_keys_from_seed("Alice")],
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        vec![
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            get_account_id_from_seed::<sr25519::Public>("Bob"),
                            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                        ],
                    )
                },
                vec![],
                None,
                spec.protocol_id,
                spec.properties.clone(),
                Default::default(),
            ),
            Alternative::LocalTestnet => ChainSpec::from_genesis(
                spec.name,
                spec.id,
                spec.chain_type.clone(),
                move || {
                    testnet_genesis(
                        wasm_binary,
                        vec![
                            authority_keys_from_seed("Alice"),
                            authority_keys_from_seed("Bob"),
                        ],
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        vec![
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            get_account_id_from_seed::<sr25519::Public>("Bob"),
                            get_account_id_from_seed::<sr25519::Public>("Charlie"),
                            get_account_id_from_seed::<sr25519::Public>("Dave"),
                            get_account_id_from_seed::<sr25519::Public>("Eve"),
                            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                        ],
                    )
                },
                vec![],
                None,
                spec.protocol_id,
                spec.properties.clone(),
                Default::default(),
            ),
            Alternative::Testnet => ChainSpec::from_genesis(
                spec.name,
                spec.id,
                spec.chain_type.clone(),
                move || {
                    testnet_genesis(
                        wasm_binary,
                        vec![
                            authority_keys_from_seed("Alice"),
                            authority_keys_from_seed("Bob"),
                        ],
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        vec![
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            get_account_id_from_seed::<sr25519::Public>("Bob"),
                            get_account_id_from_seed::<sr25519::Public>("Charlie"),
                            get_account_id_from_seed::<sr25519::Public>("Dave"),
                            get_account_id_from_seed::<sr25519::Public>("Eve"),
                            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                        ],
                    )
                },
                vec![
                    MultiaddrWithPeerId::from_str("/xxx/xx.xx.xxx.xxx/xxx/xxxxx/xxx/xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").unwrap()
                ],
                Some(
                    TelemetryEndpoints::new(
                        vec![
                            ("https://xxxxxxxxxxxxx".to_string(), 0)
                        ]
                    ).unwrap()
                ),
                spec.protocol_id,
                spec.properties.clone(),
                Default::default(),
            ),
            Alternative::Beta => ChainSpec::from_genesis(
                spec.name,
                spec.id,
                spec.chain_type.clone(),
                move || {
                    beta_genesis(
                        wasm_binary,
                        // initial_authorities
                        vec![(
                                 hex!["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"].unchecked_into(),
                                 hex!["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"].unchecked_into(),
                                 hex!["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"].unchecked_into(),
                             ), (
                                 hex!["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"].unchecked_into(),
                                 hex!["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"].unchecked_into(),
                                 hex!["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"].unchecked_into(),
                             )],
                        hex!["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"].into(),
                        // endowed_accounts
                        vec![(
                                hex!["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"].into(),
                                200 * RFUEL
                            ), (
                                hex!["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"].into(),
                                200 * RFUEL
                            ), (
                                hex!["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"].into(),
                                200 * RFUEL
                            )
                        ],
                    )
                },
                vec![
                    MultiaddrWithPeerId::from_str("/xxx/xx.xx.xxx.xxx/xxx/xxxxx/xxx/xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").unwrap()
                ],
                Some(
                    TelemetryEndpoints::new(
                        vec![
                            ("https://xxxxxxxxxxxxxxxxxx".to_string(), 0)
                        ]
                    ).unwrap()
                ),
                spec.protocol_id,
                spec.properties.clone(),
                Default::default(),
            ),
            Alternative::Mainnet => ChainSpec::from_json_bytes(
                &include_bytes!("../xxx/xxxxxxxxxxxxxxxxxxx")[..]
            )?,
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(Alternative::Development),
            "local" => Some(Alternative::LocalTestnet),
            "test" => Some(Alternative::Testnet),
            "beta" => Some(Alternative::Beta),
            "mainnet" => Some(Alternative::Mainnet),
            _ => None,
        }
    }
}

fn asset_init() -> Vec<(
    CurrencyId,
    rio_assets::AssetInfo,
    rio_assets::Restrictions,
    Vec<(AccountId, Balance)>,
)> {
    vec![
        // asset id defined in protocol
        (
            CurrencyId::from(riochain_runtime::LOCKED_RFUEL),
            rio_assets::AssetInfo {
                symbol: b"LOCKED_RFUEL".to_vec(),
                name: b"Locked Rio Fuel Token".to_vec(),
                decimals: 12,
                desc: b"Locked Rio Fuel Token".to_vec(),
                chain: rio_assets::Chain::Rio,
            },
            rio_assets::Restriction::Transferable.into(),
            vec![],
        ),
        (
            CurrencyId::from(riochain_runtime::OM),
            rio_assets::AssetInfo {
                symbol: b"OM".to_vec(),
                name: b"MANTRA DAO Token".to_vec(),
                decimals: 12,
                desc: b"MANTRA DAO Token".to_vec(),
                chain: rio_assets::Chain::Rio,
            },
            rio_assets::Restrictions::none(),
            vec![],
        ),
        (
            CurrencyId::from(riochain_runtime::RBTC),
            rio_assets::AssetInfo {
                symbol: b"RBTC".to_vec(),
                name: b"RBTC token".to_vec(),
                decimals: 8,
                desc: b"Bitcoin in RioChain".to_vec(),
                chain: rio_assets::Chain::Bitcoin,
            },
            rio_assets::Restrictions::none(),
            vec![],
        ),
        (
            CurrencyId::from(riochain_runtime::RLTC),
            rio_assets::AssetInfo {
                symbol: b"RLTC".to_vec(),
                name: b"RLTC token".to_vec(),
                decimals: 8,
                desc: b"Litecoin in RioChain".to_vec(),
                chain: rio_assets::Chain::Litecoin,
            },
            rio_assets::Restrictions::none(),
            vec![],
        ),
        (
            CurrencyId::from(riochain_runtime::RETH),
            rio_assets::AssetInfo {
                symbol: b"RETH".to_vec(),
                name: b"RETH token".to_vec(),
                decimals: 18,
                desc: b"Ether in RioChain".to_vec(),
                chain: rio_assets::Chain::Ethereum,
            },
            rio_assets::Restrictions::none(),
            vec![],
        ),
        (
            CurrencyId::from(riochain_runtime::RUSDT),
            rio_assets::AssetInfo {
                symbol: b"RUSDT".to_vec(),
                name: b"RUSDT token".to_vec(),
                decimals: 6,
                desc: b"USDT in RioChain".to_vec(),
                chain: rio_assets::Chain::Ethereum,
            },
            rio_assets::Restrictions::none(),
            vec![],
        ),
    ]
}

fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<AuthorityKeysTuple>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
    const ENDOWMENT: Balance = 10_000_000 * RFUEL;
    const STASH: Balance = 100 * RFUEL;
    let num_endowed_accounts = endowed_accounts.len();
    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_sudo: Some(SudoConfig {
            key: root_key.clone(),
        }),
        pallet_collective_Instance1: Some(CouncilConfig::default()),
        pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .collect(),
            phantom: Default::default(),
        }),
        pallet_membership_Instance1: Some(Default::default()),
        pallet_democracy: Some(DemocracyConfig::default()),
        pallet_treasury: Some(Default::default()),
        pallet_elections_phragmen: Some(Default::default()),
        pallet_im_online: Some(ImOnlineConfig { keys: vec![] }),
        pallet_society: Some(SocietyConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .collect(),
            pot: 0,
            max_members: 999,
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        session_keys(x.2.clone(), x.3.clone(), x.4.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        }),
        pallet_indices: Some(IndicesConfig { indices: vec![] }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, ENDOWMENT))
                .chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
                .collect(),
        }),
        pallet_staking: Some(StakingConfig {
            minimum_validator_count: 1,
            validator_count: 2,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            force_era: Forcing::NotForcing,
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }),
        rio_assets: Some(RioAssetsConfig {
            init: asset_init(),
        }),
        rio_gateway: Some(RioGatewayConfig {
            max_deposit_index: 10000,
            initial_supported_currencies: vec![
                (CurrencyId::from(riochain_runtime::assets_def::RFUEL), 10 * 1_000_000_000_000_000_000), // 10 RFUEL
                (CurrencyId::from(riochain_runtime::OM), 10 * 1_000_000_000_000_000_000),
                (CurrencyId::from(riochain_runtime::RBTC), 5 * 100_000),  // 0.005 BTC
                (CurrencyId::from(riochain_runtime::RETH), 5 * 1_000_000_000_000_000_000),  // 0.05 ETH
                (CurrencyId::from(riochain_runtime::RUSDT), 5 * 1_000_000), // 5 USDT
            ],
            deposit_addr_info: vec![(
                CurrencyId::from(riochain_runtime::RBTC),
                DepositAddrInfo::Bip32(
                    Bip32 {
                        x_pub: b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_vec(),
                        path: b"x/xx'/x'/x".to_vec()
                    }
                )
            )],
            admins: vec![(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                riochain_runtime::rio_gateway::Auths::all(),
            )],
        }),
        rio_payment_fee: Some(RioPaymentFeeConfig {
            account_id: get_account_id_from_seed::<sr25519::Public>("Eve"),
        }),
        rio_root: Some(RioRootConfig {
            managers: vec![root_key.clone()]
        }),
        orml_vesting: Some(VestingConfig { vesting: vec![] }),
        orml_oracle: Some(OracleConfig {
            members: Default::default(), // initialized by OperatorMembership
            phantom: Default::default(),
        }),
    }
}

fn beta_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId, ImOnlineId)>,
    root_key: AccountId,
    endowed_accounts: Vec<(AccountId, Balance)>,
) -> GenesisConfig {
    let num_endowed_accounts = endowed_accounts.len();
    // const ENDOWMENT: Balance = 10_000_000 * RFUEL;
    const STASH: Balance = 100 * RFUEL;
    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_sudo: Some(SudoConfig {
            key: root_key.clone(),
        }),
        pallet_collective_Instance1: Some(CouncilConfig::default()),
        pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
            members: endowed_accounts
                .iter()
                .map(|(a, _)| a)
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .collect(),
            phantom: Default::default(),
        }),
        pallet_membership_Instance1: Some(Default::default()),
        pallet_democracy: Some(DemocracyConfig::default()),
        pallet_treasury: Some(Default::default()),
        pallet_elections_phragmen: Some(Default::default()),
        pallet_im_online: Some(ImOnlineConfig { keys: vec![] }),
        pallet_society: Some(SocietyConfig {
            members: endowed_accounts
                .iter()
                .map(|(a, _)| a)
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .collect(),
            pot: 0,
            max_members: 999,
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.to_raw_vec().as_slice().try_into().unwrap(),
                        x.0.to_raw_vec().as_slice().try_into().unwrap(),
                        session_keys(x.0.clone(), x.1.clone(), x.2.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        }),
        pallet_indices: Some(IndicesConfig {
            indices: vec![],
        }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts,
        }),
        pallet_staking: Some(StakingConfig {
            minimum_validator_count: 1,
            validator_count: 2,
            stakers: initial_authorities.iter()
                .map(|x| (
                    x.0.to_raw_vec().as_slice().try_into().unwrap(),
                    x.1.to_raw_vec().as_slice().try_into().unwrap(),
                    STASH, StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.to_raw_vec().as_slice().try_into().unwrap(),).collect(),
            force_era: Forcing::NotForcing,
            slash_reward_fraction: Perbill::from_percent(10),
            .. Default::default()
        }),
        rio_assets: Some(RioAssetsConfig {
            init: asset_init(),
        }),
        rio_gateway: Some(RioGatewayConfig {
            max_deposit_index: 10000,
            initial_supported_currencies: vec![
                (CurrencyId::from(riochain_runtime::assets_def::RFUEL), 10 * 1_000_000_000_000_000_000), // 10 RFUEL
                (CurrencyId::from(riochain_runtime::OM), 10 * 1_000_000_000_000_000_000), // 10 OM
                (CurrencyId::from(riochain_runtime::RBTC), 5 * 100_000),  // 0.005 BTC
                (CurrencyId::from(riochain_runtime::RETH), 5 * 1_000_000_000_000_000_000),  // 0.05 ETH
                (CurrencyId::from(riochain_runtime::RUSDT), 5 * 1_000_000), // 5 USDT
            ],
            deposit_addr_info: vec![(
                CurrencyId::from(riochain_runtime::RBTC),
                DepositAddrInfo::Bip32(
                    Bip32 {
                        x_pub: b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_vec(),
                        path: b"x/xx'/x'/x".to_vec()
                    }
                )
            )],
            admins: vec![(
                // 
                hex!["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"].into(),
                riochain_runtime::rio_gateway::Auths::all(),
            )],
        }),
        rio_payment_fee: Some(RioPaymentFeeConfig {
            account_id: hex!["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"]
                .into(), //
        }),
        rio_root: Some(RioRootConfig {
            managers: vec![root_key.clone()]
        }),
        orml_vesting: Some(VestingConfig { vesting: vec![] }),
        orml_oracle: Some(OracleConfig {
            members: Default::default(), // initialized by OperatorMembership
            phantom: Default::default(),
        }),
    }
}
