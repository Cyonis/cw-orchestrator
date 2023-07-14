use ibc_chain_registry::paths::IBCPath;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::fs;

use crate::error::InterchainError;
use crate::IcResult;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    chains: Vec<Chain>,
    relayers: Vec<Relayer>,
    explorer: Service,
    registry: Service,
}

impl Config {
    // find a relayer that relays between the two chains
    pub fn relayer_for(&self, chain_id_a: &str, chain_id_b: &str) -> IcResult<Relayer> {
        self.relayers
            .iter()
            .find(|r| r.relays_over(chain_id_a, chain_id_b))
            .ok_or(InterchainError::HermesNotFound)
            .cloned()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Chain {
    name: String,
    #[serde(rename = "type")]
    chain_type: String,
    #[serde(rename = "numValidators")]
    num_validators: u32,
    ports: Ports,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Ports {
    rest: u32,
    rpc: u32,
    grpc: u32,
    faucet: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Relayer {
    name: String,
    #[serde(rename = "type")]
    relayer_type: String,
    replicas: u32,
    chains: Vec<String>,
}

impl Relayer {
    // get the pod name for the relayer
    pub fn name(&self) -> String {
        format!("{}-{}", self.relayer_type, self.name)
    }
    /// Returns true if the relayer is configured to relay between the two chains.
    pub fn relays_over(&self, chain_id_a: &str, chain_id_b: &str) -> bool {
        self.chains.contains(&chain_id_a.to_string())
            && self.chains.contains(&chain_id_b.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Service {
    enabled: bool,
    ports: HashMap<String, u32>,
}

// Parse the YAML file into the Config struct.
pub fn parse_config(filename: &str) -> Result<Config, serde_yaml::Error> {
    let file_contents = fs::read_to_string(filename).unwrap();
    serde_yaml::from_str(&file_contents)
}

// Get a Vec of (name, grpc, faucet) for each chain.
pub fn get_chain_info(config: &Config) -> Vec<(&String, u32, u32)> {
    config
        .chains
        .iter()
        .map(|chain| (&chain.name, chain.ports.grpc, chain.ports.faucet))
        .collect()
}

// Get a Vec of (name, chains) for each relayer.
pub fn get_relayers_info(config: &Config) -> Vec<(&String, &Vec<String>)> {
    config
        .relayers
        .iter()
        .map(|relayer| (&relayer.name, &relayer.chains))
        .collect()
}
