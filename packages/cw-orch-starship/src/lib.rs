pub mod config;
pub mod error;
pub mod faucet;
pub mod starship;

pub type IcResult<T> = Result<T, error::InterchainError>;
