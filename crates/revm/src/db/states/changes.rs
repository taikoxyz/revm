use super::RevertToSlot;
use revm_interpreter::primitives::{AccountInfo, Address, Bytecode, ChainAddress, B256, U256};
use std::vec::Vec;

/// accounts/storages/contracts for inclusion into database.
/// Structure is made so it is easier to apply directly to database
/// that mostly have separate tables to store account/storage/contract data.
///
/// Note: that data is **not** sorted. Some database benefit of faster inclusion
/// and smaller footprint if data is inserted in sorted order.
#[derive(Clone, Debug, Default)]
pub struct StateChangeset {
    /// Vector of **not** sorted accounts information.
    pub accounts: Vec<(ChainAddress, Option<AccountInfo>)>,
    /// Vector of **not** sorted storage.
    pub storage: Vec<PlainStorageChangeset>,
    /// Vector of contracts by bytecode hash. **not** sorted.
    pub contracts: Vec<((u64, B256), Bytecode)>,
}

impl StateChangeset {
    pub fn filter_for_chain(&mut self, chain_id: u64) {
        // multiple Txs has multiple reverts per ChainAddress, 
        // filter out account chainges and storage changes for given chain_id
        self.accounts = self
            .accounts
            .drain(..)
            .filter(|(chain, _)| chain.0 == chain_id)
            .collect();
        self.storage = self
            .storage
            .drain(..)
            .filter(|change| change.address.0 == chain_id)
            .collect();
        self.contracts = self
            .contracts
            .drain(..)
            .filter(|((chain, _), _)| *chain == chain_id)
            .collect();
    }
}

/// Plain storage changeset. Used to apply storage changes of plain state to
/// the database.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PlainStorageChangeset {
    /// Address of account
    pub address: ChainAddress,
    /// Wipe storage,
    pub wipe_storage: bool,
    /// Storage key value pairs.
    pub storage: Vec<(U256, U256)>,
}

/// Plain Storage Revert. Containing old values of changed storage.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PlainStorageRevert {
    /// Address of account
    pub address: ChainAddress,
    /// Is storage wiped in this revert. Wiped flag is set on
    /// first known selfdestruct and would require clearing the
    /// state of this storage from database (And moving it to revert).
    pub wiped: bool,
    /// Contains the storage key and old values of that storage.
    /// Reverts are **not** sorted.
    pub storage_revert: Vec<(U256, RevertToSlot)>,
}

/// Plain state reverts are used to easily store reverts into database.
///
/// Note that accounts are assumed **not** sorted.
#[derive(Clone, Debug, Default)]
pub struct PlainStateReverts {
    /// Vector of account with removed contracts bytecode
    ///
    /// Note: If AccountInfo is None means that account needs to be removed.
    pub accounts: Vec<Vec<(ChainAddress, Option<AccountInfo>)>>,
    /// Vector of storage with its address.
    pub storage: Vec<Vec<PlainStorageRevert>>,
}

impl PlainStateReverts {
    /// Constructs new [PlainStateReverts] with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            accounts: Vec::with_capacity(capacity),
            storage: Vec::with_capacity(capacity),
        }
    }

    pub fn filter_for_chain(&mut self, chain_id: u64) {
        // multiple Txs has multiple reverts per ChainAddress, 
        // filter out account chainges and storage changes for given chain_id
        self.accounts = self
            .accounts
            .drain(..)
            .map(|inner| {
                inner.into_iter()
                    .filter(|(chain, _)| chain.0 == chain_id)
                    .collect()
            })
            .collect();
        self.storage = self
            .storage
            .drain(..)
            .map(|inner| {
                inner.into_iter()
                    .filter(|revert| revert.address.0 == chain_id)
                    .collect()
            })
            .collect(); 
    }
}

/// Storage reverts
pub type StorageRevert = Vec<Vec<(ChainAddress, bool, Vec<(U256, RevertToSlot)>)>>;
