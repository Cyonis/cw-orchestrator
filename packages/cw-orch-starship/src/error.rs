use thiserror::Error;

#[derive(Error, Debug)]
pub enum StarshipClientError {
    // #[error("Error interacting with docker {0}")]
    // Docker(#[from] ::bollard::errors::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    // #[error("Error validating IBC structures {0}")]
    // ValidationError(#[from] ValidationError),

    // #[error("Error validating IBC structures {0}")]
    // ICSChannel(#[from] ibc_relayer_types::core::ics04_channel::error::Error),
    #[error("Error connecting to faucet at {0}")]
    FaucetError(String),

    #[error("Could not find hermes for these chains on localhost. Ensure it is running.")]
    HermesNotFound,

    #[error("daemon for chain {0} not found")]
    DaemonNotFound(String),

    #[error("chain config for chain {0} not found")]
    ChainConfigNotFound(String),

    #[error("Configuration already registered for chain {0}")]
    AlreadyRegistered(String),

    #[error("Missing test mnemonic for chain {0}")]
    MissingTestMnemonic(String),
}
