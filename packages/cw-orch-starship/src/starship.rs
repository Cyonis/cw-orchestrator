//! Interactions with docker using bollard
use ibc_chain_registry::chain::ChainData;
use ibc_chain_registry::error::RegistryError;
use ibc_chain_registry::paths::IBCPath;
use serde_json::Value;
use tokio::process::Command;

use crate::config::Config;
use crate::faucet::Faucet;
use crate::IcResult;

use std::collections::HashMap;
use std::default::Default;
use std::io::{self, Write};
use tokio::runtime::Handle;

use super::error::InterchainError;

const CHAIN_REGISTRY: &str = "http://localhost:8081/chains";
const IBC_REGISTRY: &str = "http://localhost:8081/ibc";

pub type NetworkId = String;
pub type Mnemonic = String;
pub type GRpcUrl = String;

static STARSHIP_CONFIG: &str = "starship.toml";

/// Represents a set of locally running blockchain nodes and a Hermes relayer.
#[derive(Debug, Clone)]
pub struct Starship {
    /// Daemons indexable by network id, i.e. "juno-1", "osmosis-2", ...
    // chain_config: HashMap<NetworkId, ChainData>,
    pub config: Config,
    pub chains: Vec<ChainData>,
    pub ibc_paths: Vec<IBCPath>,
    pub(crate) runtime: Option<Handle>,
}

impl Starship {
    /// Create a Starship object from the localhost chain registry.
    pub fn new(rt: Handle) -> IcResult<Self> {
        let mut starship = rt.block_on(Self::new_async())?;
        starship.runtime = Some(rt);
        Ok(starship)
    }

    /// Builds a new `Starship` instance from the hosted chain registry.
    pub async fn new_async() -> IcResult<Self> {
        // Fetch all chain data from the chain registry
        let response = reqwest::get(CHAIN_REGISTRY)
            .await
            .map_err(|e| RegistryError::request_error(CHAIN_REGISTRY.to_string(), e))
            .unwrap();

        // All the chain data
        let chains: Result<Vec<ChainData>, RegistryError> = if response.status().is_success() {
            match response.text().await {
                Ok(body) => match serde_json::from_str::<Value>(&body) {
                    Ok(parsed) => serde_json::from_value(parsed["chains"].clone())
                        .map_err(|e| RegistryError::json_parse_error("chain_name".to_string(), e)),
                    Err(e) => Err(RegistryError::json_parse_error("chain_name".to_string(), e)),
                },
                Err(e) => Err(RegistryError::request_error(CHAIN_REGISTRY.to_string(), e)),
            }
        } else {
            Err(RegistryError::status_error(
                CHAIN_REGISTRY.to_string(),
                response.status().as_u16(),
            ))
        };

        // Fetch all the ibc paths from the chain registry
        let response = reqwest::get(IBC_REGISTRY)
            .await
            .map_err(|e| RegistryError::request_error(IBC_REGISTRY.to_string(), e))
            .unwrap();

        // All the ibc paths
        let ibc_paths: Result<Vec<IBCPath>, RegistryError> = if response.status().is_success() {
            match response.text().await {
                Ok(body) => match serde_json::from_str::<Value>(&body) {
                    Ok(parsed) => serde_json::from_value(parsed["data"].clone())
                        .map_err(|e| RegistryError::json_parse_error("chain_name".to_string(), e)),
                    Err(e) => Err(RegistryError::json_parse_error("chain_name".to_string(), e)),
                },
                Err(e) => Err(RegistryError::request_error(IBC_REGISTRY.to_string(), e)),
            }
        } else {
            Err(RegistryError::status_error(
                IBC_REGISTRY.to_string(),
                response.status().as_u16(),
            ))
        };

        let path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/examples/starship.yaml"
        );
        let config = crate::config::parse_config(&path).unwrap();

        // get all the ibc data:
        Ok(Self {
            config,
            chains: chains.unwrap(),
            ibc_paths: ibc_paths.unwrap(),
            runtime: None,
        })
    }

    pub async fn create_channel(
        &self,
        chain_id_a: &str,
        chain_id_b: &str,
        port_a: &str,
        port_b: &str,
        channel_version: &str,
    ) -> IcResult<()> {
        // find an hermes pod with these ids
        let relayer = self.config.relayer_for(chain_id_a, chain_id_b)?;
        // get the pod id
        let relayer_name = relayer.name();
        println!("relayer_name: {:?}", relayer_name);

        // get the connection where the chain names match the two provided chain_ids

        let connection_a = self.ibc_paths.iter().find(|path| {
            path.chain_1.chain_name == chain_id_a && path.chain_2.chain_name == chain_id_b 
            || path.chain_1.chain_name == chain_id_b && path.chain_2.chain_name == chain_id_a
        }).unwrap();

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
        // create channel by executing on this pod
        let command = [
            "hermes",
            "create",
            "channel",
            "--channel-version",
            channel_version,
            "--a-connection",
            connection_a.chain_1.connection_id.as_str(),
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
        let mut channel_out = Command::new("kubectl")
            .arg("exec")
            .arg(pod_id)
            .arg("--")
            // .arg(cmd)
            .output()
            .await
            .unwrap();
        Ok(())
    }
}
