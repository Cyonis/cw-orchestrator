mod contract;
mod daemon;
mod error;
mod helpers;
mod index_response;
///
pub mod interface;
mod keys;
mod mock;
pub mod networks;
pub mod prelude;
pub mod state;
mod tx_handler;

// pub mod traits;
pub use contract::Contract;
pub use daemon::core::Daemon;
pub use error::BootError;
pub use helpers::get_env_vars;
pub use index_response::IndexResponse;
pub use mock::core::Mock;
pub use tx_handler::{TxHandler, TxResponse};

/// Signals a supported execution environment
pub trait BootEnvironment: TxHandler + Clone {}

impl<T: TxHandler + Clone> BootEnvironment for T {}

pub(crate) mod cosmos_modules {
    pub use cosmrs::proto::cosmos::auth::v1beta1 as auth;
    pub use cosmrs::proto::cosmos::authz::v1beta1 as authz;
    pub use cosmrs::proto::cosmos::bank::v1beta1 as bank;
    pub use cosmrs::proto::cosmos::base::abci::v1beta1 as abci;
    pub use cosmrs::proto::cosmos::base::tendermint::v1beta1 as tendermint;
    pub use cosmrs::proto::cosmos::base::v1beta1 as base;
    pub use cosmrs::proto::cosmos::crisis::v1beta1 as crisis;
    pub use cosmrs::proto::cosmos::distribution::v1beta1 as distribution;
    pub use cosmrs::proto::cosmos::evidence::v1beta1 as evidence;
    pub use cosmrs::proto::cosmos::feegrant::v1beta1 as feegrant;
    pub use cosmrs::proto::cosmos::gov::v1beta1 as gov;
    pub use cosmrs::proto::cosmos::mint::v1beta1 as mint;
    pub use cosmrs::proto::cosmos::params::v1beta1 as params;
    pub use cosmrs::proto::cosmos::slashing::v1beta1 as slashing;
    pub use cosmrs::proto::cosmos::staking::v1beta1 as staking;
    pub use cosmrs::proto::cosmos::tx::v1beta1 as tx;
    pub use cosmrs::proto::cosmwasm::wasm::v1 as cosmwasm;
    pub use cosmrs::proto::tendermint::abci as tendermint_abci;
}

// mod macro_dev {
//     use terra_rust_script_derive::contract;

//     #[derive(Clone, Debug, contract)]
//     /// Updates the addressbook
//     pub enum ExecuteMsg {
//         UpdateContractAddresses {
//             to_add: Vec<(String, String)>,
//             to_remove: Vec<String>,
//         },
//         UpdateAssetAddresses {
//             to_add: Vec<(String, String)>,
//             to_remove: Vec<String>,
//         },
//         /// Sets a new Admin
//         SetAdmin {
//             admin: String,
//         },

//         Set {
//             init: InstantiateMsg,
//         },
//     }

//     #[derive(Clone, Debug, contract)]
//     pub struct InstantiateMsg {
//         /// Version control contract used to get code-ids and register OS
//         pub version_control_contract: String,
//         /// Memory contract
//         pub memory_contract: String,
//         // Creation fee in some denom (TBD)
//         pub creation_fee: u32,
//     }
// }