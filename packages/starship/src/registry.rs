use ibc_chain_registry::{chain::ChainData, paths::IBCPath};
use serde::Deserialize;
use serde_json::Value;

use crate::{config::Config, error::StarshipClientError, StarshipClientResult};

pub type URL = String;

#[derive(Debug, Clone)]
pub struct Registry(URL);

impl Registry {
    // Construct the registry url from the config
    pub async fn new(url: &str, config: &Config) -> Self {
        let url = format!("{}:{}", url, config.registry.ports.get("rest").unwrap());
        // Assert that the faucet is reachable
        let client = reqwest::Client::new();
        client
            .get(&format!("{}/chains", url))
            .send()
            .await
            .map_err(|e| StarshipClientError::FaucetError(e.to_string()))
            .unwrap();
        Self(url)
    }

    pub fn url(&self) -> String {
        self.0.clone()
    }

    fn chains_data_url(&self) -> String {
        format!("{}/chains", self.url())
    }

    fn ibc_data_url(&self) -> String {
        format!("{}/ibc", self.url())
    }

    /// Get an IBC path between two chains.
    pub async fn ibc_path(
        &self,
        chain_id_a: &str,
        chain_id_b: &str,
    ) -> StarshipClientResult<IBCPath> {
        let ibc_path_url = format!("{}/{}/{}", self.ibc_data_url(), chain_id_a, chain_id_b);
        eprintln!("ibc_paths_url: {:?}", ibc_path_url);

        let response = reqwest::get(&ibc_path_url).await?;
        let path: IBCPath = response.json().await?;
        Ok(path)
    }

    /// Get all the chain data for this registry.
    pub async fn chain_data(&self) -> StarshipClientResult<Vec<ChainData>> {
        let response = reqwest::get(&self.chains_data_url()).await?;
        let value: Value = response.json().await?;
        let chains: Vec<ChainData> = serde_json::from_value(value["chains"].clone()).unwrap();
        Ok(chains)
    }

    /// Get the first test account mnemonic from the chain registry.
    pub async fn test_mnemonic(&self, chain_id: &str) -> Result<String, StarshipClientError> {
        let url = format!("{}/chains/{}/keys", self.0, chain_id);
        let response = reqwest::get(&url).await?;
        let data: Mnemonics = response.json().await?;
        let first_test_account_mnemonic = data
            .genesis
            .get(0)
            .ok_or_else(|| StarshipClientError::MissingTestMnemonic(chain_id.to_string()))?
            .mnemonic
            .clone();
        Ok(first_test_account_mnemonic)
    }
}

#[derive(Deserialize, Debug)]
struct Record {
    name: String,
    #[serde(rename = "type")]
    record_type: String,
    mnemonic: String,
}

#[derive(Deserialize, Debug)]
struct Mnemonics {
    genesis: Vec<Record>,
    validators: Vec<Record>,
    keys: Vec<Record>,
    relayers: Vec<Record>,
}
