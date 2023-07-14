use serde::{Deserialize, Serialize};

use crate::{error::InterchainError, IcResult};

// Faucet implementation based on: https://github.com/cosmos/cosmjs/tree/main/packages/faucet
#[derive(Debug, Clone)]
pub struct Faucet(String);

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub address: String,
    pub denom: String,
}

impl Faucet {
    // Get a faucet object from a url and port
    pub async fn new(url: impl ToString, port: impl ToString) -> Self {
        let path = format!("{}:{}", url.to_string(), port.to_string());
        // Assert that the faucet is reachable
        let client = reqwest::Client::new();
        client
            .get(&format!("http://{}/status", path))
            .send()
            .await
            .map_err(|e| InterchainError::FaucetError(e.to_string()))
            .unwrap();
        Self(path)
    }

    pub async fn request_funds(
        &self,
        address: impl ToString,
        denom: impl ToString,
    ) -> IcResult<()> {
        let faucet = &self.0;
        let url = format!("http://{}/{}", faucet, address.to_string());
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&Request {
                address: address.to_string(),
                denom: denom.to_string(),
            })
            .send()
            .await
            .map_err(|e| InterchainError::FaucetError(e.to_string()))?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(InterchainError::FaucetError(response.text().await?))
        }
    }
}
