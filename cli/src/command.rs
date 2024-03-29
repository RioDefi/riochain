use std::path::PathBuf;

use sc_cli::{ChainSpec, Role, RuntimeVersion, SubstrateCli};
use sc_service::PartialComponents;

use crate::chain_spec;
use crate::cli::{Cli, Subcommand};
use crate::service;
use crate::service::{new_full_base, new_partial, NewFullBase};

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Rio Defi Chain Node".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://riochain.io/".into()
    }

    fn copyright_start_year() -> i32 {
        2019
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn ChainSpec>, String> {
        Ok(match id {
            "dev" => Box::new(chain_spec::Alternative::Development.load()?),
            "local" => Box::new(chain_spec::Alternative::LocalTestnet.load()?),
            "testnet" => Box::new(chain_spec::Alternative::Testnet.load()?),
            "beta" => Box::new(chain_spec::Alternative::Beta.load()?),
            "mainnet" => Box::new(chain_spec::Alternative::Mainnet.load()?),
            path => Box::new(chain_spec::ChainSpec::from_json_file(PathBuf::from(path))?)
        })
    }

    fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        &riochain_runtime::VERSION
    }
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

	fn set_default_ss58_version(spec: &Box<dyn sc_service::ChainSpec>) -> sc_cli::Result<()> {
        use sp_core::crypto::Ss58AddressFormat;
        use std::convert::TryInto;

        let ss58_format: u8 = spec.properties()
            .get("ss58Format").ok_or("`ss58Format` must exist in properties")?
            .as_u64().ok_or("`ss58Format` must be a number")?
            .try_into().or(Err("`ss58Format` must be u8"))?;
        let ss58_format = Ss58AddressFormat::Custom(ss58_format);
        sp_core::crypto::set_default_ss58_version(ss58_format);
        Ok(())
	};

    match &cli.subcommand {
        None => {
            let runner = cli.create_runner(&cli.run)?;
            let chain_spec = &runner.config().chain_spec;
			set_default_ss58_version(chain_spec)?;
            
            runner.run_node_until_exit(|config| match config.role {
                Role::Light => service::new_light(config),
                _ => service::new_full(config),
            })
        }
        Some(Subcommand::Key(cmd)) => cmd.run(),
        Some(Subcommand::Sign(cmd)) => cmd.run(),
        Some(Subcommand::Verify(cmd)) => cmd.run(),
        Some(Subcommand::Vanity(cmd)) => cmd.run(),
        Some(Subcommand::Benchmark(cmd)) => {
            if cfg!(feature = "runtime-benchmarks") {
                let runner = cli.create_runner(cmd)?;
                runner.sync_run(|config| {
                    cmd.run::<riochain_runtime::Block, rio_executor::Executor>(config)
                })
            } else {
                println!(
                    "Benchmarking wasn't enabled when building the node. \
                    You can enable it with `--features runtime-benchmarks`."
                );
                Ok(())
            }
        }
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::BuildSyncSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.async_run(|config| {
                let chain_spec = config.chain_spec.cloned_box();
                let network_config = config.network.clone();
                let NewFullBase {
                    task_manager,
                    client,
                    network_status_sinks,
                    ..
                } = new_full_base(config)?;

                Ok((
                    cmd.run(chain_spec, network_config, client, network_status_sinks),
                    task_manager,
                ))
            })
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = new_partial(&config)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = new_partial(&config)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    backend,
                    ..
                } = new_partial(&config)?;
                Ok((cmd.run(client, backend), task_manager))
            })
        }
    }
}
