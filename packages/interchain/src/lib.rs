pub mod error;
pub mod interchain_channel;
pub mod interchain_channel_builder;
pub mod interchain_env;
pub mod packet_inspector;

// Tracking IBC state
mod ibc_tracker;

pub use error::InterchainError;
pub type IcResult<R> = Result<R, InterchainError>;
