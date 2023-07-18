use cosmrs::tendermint::Time;
use cosmwasm_std::{Addr, Coin};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, rc::Rc, time::Duration};

use crate::{
    daemon::{error::DaemonError, CosmTxResponse, Daemon, DaemonState, Wallet},
    environment::TxHandler,
    prelude::{
        queriers::{CosmWasm, DaemonQuerier, Node},
        CallAs, ContractInstance, CwOrchExecute, IndexResponse, Uploadable, WasmPath,
    },
    state::ChainState,
};

impl ChainState for Daemon {
    type Out = Rc<DaemonState>;

    fn state(&self) -> Self::Out {
        self.daemon.state.clone()
    }
}

// Execute on the real chain, returns tx response
impl TxHandler for Daemon {
    type Response = CosmTxResponse;
    type Error = DaemonError;
    type ContractSource = WasmPath;

    fn sender(&self) -> Addr {
        self.daemon.sender.address().unwrap()
    }

    fn upload(&self, uploadable: &impl Uploadable) -> Result<Self::Response, DaemonError> {
        let sender = &self.daemon.sender;
        let wasm_path = uploadable.wasm();

        log::debug!("Uploading file at {:?}", wasm_path);

        let file_contents = std::fs::read(wasm_path.path())?;
        let store_msg = cosmrs::cosmwasm::MsgStoreCode {
            sender: sender.pub_addr()?,
            wasm_byte_code: file_contents,
            instantiate_permission: None,
        };
        let result = self
            .rt_handle
            .block_on(sender.commit_tx(vec![store_msg], None))?;

        log::info!("Uploaded: {:?}", result.txhash);

        let code_id = result.uploaded_code_id().unwrap();

        // wait for the node to return the contract information for this upload
        let wasm = CosmWasm::new(self.channel());
        while self.rt_handle.block_on(wasm.code(code_id)).is_err() {
            self.rt_handle
                .block_on(tokio::time::sleep(Duration::from_secs(6)));
        }

        Ok(result)
    }

    fn execute<E: Serialize>(
        &self,
        exec_msg: &E,
        coins: &[cosmwasm_std::Coin],
        contract_address: &Addr,
    ) -> Result<Self::Response, DaemonError> {
        self.rt_handle
            .block_on(self.daemon.execute(exec_msg, coins, contract_address))
    }

    fn instantiate<I: Serialize + Debug>(
        &self,
        code_id: u64,
        init_msg: &I,
        label: Option<&str>,
        admin: Option<&Addr>,
        coins: &[Coin],
    ) -> Result<Self::Response, DaemonError> {
        self.rt_handle.block_on(
            self.daemon
                .instantiate(code_id, init_msg, label, admin, coins),
        )
    }

    fn query<Q: Serialize + Debug, T: Serialize + DeserializeOwned>(
        &self,
        query_msg: &Q,
        contract_address: &Addr,
    ) -> Result<T, DaemonError> {
        self.rt_handle
            .block_on(self.daemon.query(query_msg, contract_address))
    }

    fn migrate<M: Serialize + Debug>(
        &self,
        migrate_msg: &M,
        new_code_id: u64,
        contract_address: &Addr,
    ) -> Result<Self::Response, DaemonError> {
        self.rt_handle.block_on(
            self.daemon
                .migrate(migrate_msg, new_code_id, contract_address),
        )
    }

    fn wait_blocks(&self, amount: u64) -> Result<(), DaemonError> {
        let mut last_height = self
            .rt_handle
            .block_on(self.query_client::<Node>().block_height())?;
        let end_height = last_height + amount;

        while last_height < end_height {
            // wait
            self.rt_handle
                .block_on(tokio::time::sleep(Duration::from_secs(4)));

            // ping latest block
            last_height = self
                .rt_handle
                .block_on(self.query_client::<Node>().block_height())?;
        }
        Ok(())
    }

    fn wait_seconds(&self, secs: u64) -> Result<(), DaemonError> {
        self.rt_handle
            .block_on(tokio::time::sleep(Duration::from_secs(secs)));

        Ok(())
    }

    fn next_block(&self) -> Result<(), DaemonError> {
        let mut last_height = self
            .rt_handle
            .block_on(self.query_client::<Node>().block_height())?;
        let end_height = last_height + 1;

        while last_height < end_height {
            // wait
            self.rt_handle
                .block_on(tokio::time::sleep(Duration::from_secs(4)));

            // ping latest block
            last_height = self
                .rt_handle
                .block_on(self.query_client::<Node>().block_height())?;
        }
        Ok(())
    }

    fn block_info(&self) -> Result<cosmwasm_std::BlockInfo, DaemonError> {
        let block = self
            .rt_handle
            .block_on(self.query_client::<Node>().latest_block())?;
        let since_epoch = block.header.time.duration_since(Time::unix_epoch())?;
        let time = cosmwasm_std::Timestamp::from_nanos(since_epoch.as_nanos() as u64);
        Ok(cosmwasm_std::BlockInfo {
            height: block.header.height.value(),
            time,
            chain_id: block.header.chain_id.to_string(),
        })
    }
}

impl<T: CwOrchExecute<Daemon> + ContractInstance<Daemon> + Clone> CallAs<Daemon> for T {
    type Sender = Wallet;

    fn set_sender(&mut self, sender: &Self::Sender) {
        self.as_instance_mut().chain.daemon.set_sender(sender);
    }

    fn call_as(&self, sender: &Self::Sender) -> Self {
        let mut contract = self.clone();
        contract.set_sender(sender);
        contract
    }
}