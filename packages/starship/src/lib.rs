pub mod config;
mod core;
mod error;
pub mod faucet;
pub mod registry;

pub use crate::core::StarshipClient;
pub use error::StarshipClientError;

pub type StarshipClientResult<T> = Result<T, error::StarshipClientError>;
