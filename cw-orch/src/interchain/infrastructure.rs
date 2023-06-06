//! Interactions with docker using bollard

use crate::daemon::Daemon;
use crate::daemon::DaemonError;
use crate::interchain::docker::DockerHelper;

use crate::interchain::IcResult;
use crate::prelude::InterchainEnv;
use ibc_chain_registry::chain::{ChainData, Grpc};
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;

use log4rs::Config;

use std::collections::HashMap;
use std::default::Default;
use std::path::PathBuf;
use tokio::runtime::Handle;

use super::error::InterchainError;

use crate::state::ChainState;

pub type ContainerId = String;
pub type Port = String;
pub type ChannelId = String;
pub type NetworkId = String;
pub type Mnemonic = String;

/// Represents a set of locally running blockchain nodes and a Hermes relayer.
pub struct InterchainInfrastructure {
    /// Daemons indexable by network id, i.e. "juno-1", "osmosis-2", ...
    daemons: HashMap<NetworkId, Daemon>,
}

impl InterchainInfrastructure {
    /// Builds a new `InterchainInfrastructure` instance.
    pub fn new<T>(runtime: &Handle, chains: Vec<(T, &str)>) -> IcResult<Self>
    where
        T: Into<ChainData>,
    {
        let (mut chains, mnemonics): (Vec<ChainData>, _) = chains
            .into_iter()
            .map(|(chain, mnemonic)| (chain.into(), mnemonic.to_string()))
            .unzip::<_, _, Vec<_>, Vec<_>>();
        // Start update gRPC ports with local daemons
        runtime.block_on(Self::configure_networks(&mut chains))?;

        let daemons = Self::build_daemons(
            runtime,
            // combine the chain with its mnemonic
            &chains.into_iter().zip(mnemonics).collect::<Vec<_>>(),
        )?;

        InterchainInfrastructure::setup_interchain_log(&daemons);

        Ok(Self { daemons })
    }

    /// Initiates an interchain log setup
    /// This will log the different chain interactions and updates on separate files for each chain.
    /// This is useful for tracking operations happenning on IBC chains
    pub fn setup_interchain_log(daemons: &HashMap<NetworkId, Daemon>) {
        let encoder = Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)(utc)} - {l}: {m}{n}",
        ));
        let main_log_path = generate_log_file_path("main");
        let main_appender = FileAppender::builder()
            .encoder(encoder.clone())
            .build(&main_log_path)
            .unwrap();
        // ensure dir exists
        std::fs::create_dir_all(main_log_path.parent().unwrap()).unwrap();
        // add main appender to config
        let mut config =
            Config::builder().appender(Appender::builder().build("main", Box::new(main_appender)));

        // add appender for each daemon
        for daemon in daemons.values() {
            let chain_id = daemon.state().chain_data.chain_id.to_string();
            let log_path = generate_log_file_path(&chain_id);
            let daemon_appender = FileAppender::builder()
                .encoder(encoder.clone())
                .build(&log_path)
                .unwrap();

            config = config
                .appender(Appender::builder().build(&chain_id, Box::new(daemon_appender)))
                .logger(
                    Logger::builder()
                        .appender(&chain_id)
                        .build(&chain_id, LevelFilter::Info),
                );
        }

        let config = config
            .build(Root::builder().appender("main").build(LevelFilter::Info))
            .unwrap();

        log4rs::init_config(config).unwrap();

        for daemon in daemons.values() {
            let log_target = &daemon.state().chain_data.chain_id.to_string();
            // log startup to each daemon log
            log::info!(target: log_target, "Starting daemon {log_target}");
        }
    }

    /// Get the daemon for a network-id in the interchain.
    pub fn daemon(&self, chain_id: impl ToString) -> Result<Daemon, InterchainError> {
        self.daemons
            .get(&chain_id.to_string())
            .ok_or(InterchainError::DaemonNotFound(chain_id.to_string()))
            .cloned()
    }

    /// Get the gRPC ports for the local daemons and set them in the `ChainData` objects.
    pub async fn configure_networks(networks: &mut [ChainData]) -> IcResult<()> {
        let docker_helper = DockerHelper::new().await?;

        // use chain data network name as to filter container ids
        let containers_grpc_port = docker_helper.grpc_ports().await?;

        // update network with correct grpc port
        networks.iter_mut().for_each(|network| {
            for container in &containers_grpc_port {
                if container.0.contains(&network.chain_name) {
                    network.apis.grpc = vec![Grpc {
                        address: format!("http://0.0.0.0:{}", container.1),
                        ..Default::default()
                    }];
                    log::info!(
                        "Connected to chain {} on port {}",
                        network.chain_name,
                        container.1
                    );
                }
            }
        });
        Ok(())
    }

    /// Build the daemons from the shared runtime and chain data
    fn build_daemons(
        runtime_handle: &Handle,
        chain_data: &[(ChainData, Mnemonic)],
    ) -> Result<HashMap<NetworkId, Daemon>, DaemonError> {
        let mut daemons = HashMap::new();
        for (chain, mnemonic) in chain_data {
            let daemon = Daemon::builder()
                .chain(chain.clone())
                .deployment_id("interchain")
                .handle(runtime_handle)
                .mnemonic(mnemonic)
                .build()
                .unwrap();

            daemons.insert(chain.chain_id.to_string(), daemon);
        }
        Ok(daemons)
    }

    /// Blocks until all the IBC packets sent during the transaction on chain `chain_id` with transaction hash `packet_send_tx_hash` have completed their cycle
    /// (Packet Sent, Packet Received, Packet Acknowledgment)
    /// This also follows additional packets sent out in the resulting transactions
    /// See the documentation for `InterchainEnv::await_ibc_execution` for more details about the awaiting procedure
    pub async fn await_ibc_execution(
        &self,
        chain_id: NetworkId,
        packet_send_tx_hash: String,
    ) -> Result<(), DaemonError> {
        // We crate an interchain env
        let mut interchain_env = InterchainEnv::default();

        for daemon in self.daemons.values() {
            interchain_env = interchain_env
                .add_custom_chain(
                    daemon.state().chain_data.chain_id.clone().to_string(),
                    daemon.clone(),
                )?
                .clone();
        }

        // We follow the trail
        interchain_env
            .await_ibc_execution(chain_id, packet_send_tx_hash)
            .await?;

        Ok(())
    }
}

/// Get the file path for the log target
fn generate_log_file_path(file: &str) -> PathBuf {
    let file_name = format!("{}.log", file);

    let mut log_path = std::env::current_dir().unwrap();
    log_path.push("logs");
    log_path.push(file_name);

    log_path
}