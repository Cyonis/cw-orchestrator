pub mod config;
mod error;
pub mod faucet;
pub mod registry;
mod starship;

pub use error::StarshipClientError;
pub use starship::StarshipClient;

pub type StarshipClientResult<T> = Result<T, error::StarshipClientError>;
