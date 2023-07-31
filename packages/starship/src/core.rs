//! Interactions with docker using bollard
use ibc_chain_registry::chain::{ChainData, Grpc};

use tokio::process::Command;

use crate::config::{self, Config};

use crate::registry::Registry;
use crate::StarshipClientResult;

use std::default::Default;

use tokio::runtime::Handle;

// const CHAIN_REGISTRY: &str = "http://localhost:8081/chains";
// const IBC_REGISTRY: &str = "http://localhost:8081/ibc";
const LOCALHOST: &str = "http://localhost";

pub type NetworkId = String;
pub type Mnemonic = String;
pub type GRpcUrl = String;

static STARSHIP_CONFIG: &str = "starship.toml";

/// Represents a set of locally running blockchain nodes and a Hermes relayer.
#[derive(Debug, Clone, Default)]
pub struct StarshipClient {
    // Where starship is hosted, uses localhost by default.
    url: String,
    /// Daemons indexable by network id, i.e. "juno-1", "osmosis-2", ...
    // chain_config: HashMap<NetworkId, ChainData>,
    pub config: Config,
    pub chains: Vec<ChainData>,
}

impl StarshipClient {
    /// Create a Starship object from the localhost chain registry.
    pub fn new(rt: Handle, url: Option<&str>) -> StarshipClientResult<Self> {
        let starship = rt.block_on(Self::new_async(url))?;
        Ok(starship)
    }

    /// Builds a new `Starship` instance from the hosted chain registry.
    pub async fn new_async(url: Option<&str>) -> StarshipClientResult<Self> {
        let url = url
            .map(|u| u.to_string())
            .unwrap_or_else(|| LOCALHOST.to_string());

        let path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/examples/starship.yaml"
        );
        let config = config::parse_config(&path).unwrap();

        let registry = Registry::new(&url, &config).await;

        // Fetch all chain data from the chain registry
        let mut chains = registry.chain_data().await?;

        // Set the grpc address for the chains
        config.chains.iter().for_each(|chain| {
            let chain_id = chain.name.clone();
            let grpc = chain.ports.grpc.clone();
            chains.iter_mut().for_each(|chain| {
                if chain.chain_id.as_str() == &chain_id {
                    eprintln!("{}", chain.chain_id);
                    chain.apis.grpc = vec![Grpc {
                        address: format!("{}:{}", url, grpc),
                        provider: None,
                    }];
                }
            })
        });

        // get all the ibc data:
        Ok(Self {
            url,
            config,
            chains,
        })
    }

    // Get the `Registry` object for this `Starship` instance.
    pub async fn registry(&self) -> Registry {
        Registry::new(&self.url, &self.config).await
    }

    pub async fn create_channel(
        &self,
        chain_id_a: &str,
        chain_id_b: &str,
        port_a: &str,
        port_b: &str,
        channel_version: &str,
    ) -> StarshipClientResult<()> {
        // find an hermes pod with these ids
        let relayer = self.config.relayer_for(chain_id_a, chain_id_b)?;
        // get the pod id
        let relayer_name = relayer.name();
        println!("relayer_name: {:?}", relayer_name);

        // execute on the pad
        let pod_id_out = Command::new("kubectl")
            .args(["get", "pods", "--no-headers"])
            .arg(format!("-lapp.kubernetes.io/name={}", relayer_name))
            .output()
            .await
            .unwrap();

        let pod_id_output = String::from_utf8(pod_id_out.stdout).unwrap();

        let pod_id = pod_id_output.split_whitespace().next().unwrap();
        println!("pod_out: {:?}", pod_id);

        // get the ibc channel between the two chains
        let path = self
            .registry()
            .await
            .ibc_path(chain_id_a, chain_id_b)
            .await?;

        // create channel by executing on this pod
        let command = [
            "hermes",
            "create",
            "channel",
            "--channel-version",
            channel_version,
            "--a-connection",
            path.chain_1.connection_id.as_str(),
            "--a-chain",
            chain_id_a,
            // "--b-chain",
            // &contract_b.get_chain().state.id,
            "--a-port",
            port_a,
            "--b-port",
            port_b,
            "--yes",
        ]
        .to_vec();
        // now execute on the pod
        let _execute_channel_create = Command::new("kubectl")
            .arg("exec")
            .arg(pod_id)
            .arg("--")
            .args(command)
            .output()
            .await
            .unwrap();
        Ok(())
    }
}
