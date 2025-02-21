//! Database that is split on State and BlockHash traits.
pub mod block_hash;
pub mod state;

pub use block_hash::{BlockHash, BlockHashRef};
pub use state::{State, StateRef};

use crate::{
    db::{SyncDatabase as Database, SyncDatabaseRef as DatabaseRef},
    Account, AccountInfo, Address, ChainAddress, Bytecode, HashMap, B256, U256,
};

use super::DatabaseCommit;

#[derive(Debug)]
pub struct DatabaseComponents<S, BH> {
    pub state: S,
    pub block_hash: BH,
}

#[derive(Debug)]
pub enum DatabaseComponentError<SE, BHE> {
    State(SE),
    BlockHash(BHE),
}

impl<S: State, BH: BlockHash> Database for DatabaseComponents<S, BH> {
    type Error = DatabaseComponentError<S::Error, BH::Error>;

    fn basic(&mut self, address: ChainAddress) -> Result<Option<AccountInfo>, Self::Error> {
        self.state.basic(address).map_err(Self::Error::State)
    }

    fn code_by_hash(&mut self, chain_id: u64, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.state
            .code_by_hash(chain_id, code_hash)
            .map_err(Self::Error::State)
    }

    fn storage(&mut self, address: ChainAddress, index: U256) -> Result<U256, Self::Error> {
        self.state
            .storage(address, index)
            .map_err(Self::Error::State)
    }

    fn block_hash(&mut self, chain_id: u64, number: u64) -> Result<B256, Self::Error> {
        self.block_hash
            .block_hash(chain_id, number)
            .map_err(Self::Error::BlockHash)
    }
}

impl<S: StateRef, BH: BlockHashRef> DatabaseRef for DatabaseComponents<S, BH> {
    type Error = DatabaseComponentError<S::Error, BH::Error>;

    fn basic_ref(&self, address: ChainAddress) -> Result<Option<AccountInfo>, Self::Error> {
        self.state.basic(address).map_err(Self::Error::State)
    }

    fn code_by_hash_ref(&self, chain_id: u64, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.state
            .code_by_hash(chain_id, code_hash)
            .map_err(Self::Error::State)
    }

    fn storage_ref(&self, address: ChainAddress, index: U256) -> Result<U256, Self::Error> {
        self.state
            .storage(address, index)
            .map_err(Self::Error::State)
    }

    fn block_hash_ref(&self, chain_id: u64, number: u64) -> Result<B256, Self::Error> {
        self.block_hash
            .block_hash(chain_id, number)
            .map_err(Self::Error::BlockHash)
    }
}

impl<S: DatabaseCommit, BH: BlockHashRef> DatabaseCommit for DatabaseComponents<S, BH> {
    fn commit(&mut self, changes: HashMap<ChainAddress, Account>) {
        self.state.commit(changes);
    }
}
