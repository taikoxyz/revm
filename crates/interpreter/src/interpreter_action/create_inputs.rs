use revm_primitives::{ChainAddress, TransactTo};

pub use crate::primitives::CreateScheme;
use crate::primitives::{Address, Bytes, TxEnv, TxKind, U256};
use std::boxed::Box;

/// Inputs for a create call.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateInputs {
    /// Caller address of the EVM.
    pub caller: ChainAddress,
    /// The create scheme.
    pub scheme: CreateScheme,
    /// The value to transfer.
    pub value: U256,
    /// The init code of the contract.
    pub init_code: Bytes,
    /// The gas limit of the call.
    pub gas_limit: u64,
}

impl CreateInputs {
    /// Creates new create inputs.
    pub fn new(tx_env: &TxEnv, gas_limit: u64) -> Option<Self> {
        let TransactTo::Create = tx_env.transact_to else {
            return None;
        };

        Some(CreateInputs {
            caller: tx_env.caller,
            scheme: CreateScheme::Create,
            value: tx_env.value,
            init_code: tx_env.data.clone(),
            gas_limit,
        })
    }

    /// Returns boxed create inputs.
    pub fn new_boxed(tx_env: &TxEnv, gas_limit: u64) -> Option<Box<Self>> {
        Self::new(tx_env, gas_limit).map(Box::new)
    }

    /// Returns the address that this create call will create.
    pub fn created_address(&self, nonce: u64) -> Address {
        match self.scheme {
            CreateScheme::Create => self.caller.1.create(nonce), // TODO: Brecht
            CreateScheme::Create2 { salt } => self
                .caller
                .1 // TODO: Brecht
                .create2_from_code(salt.to_be_bytes(), &self.init_code),
        }
    }

    // TODO: Brecht
    // Returns the address that this create call will create, without calculating the init code hash.
    //
    // Note: `hash` must be `keccak256(&self.init_code)`.
    /*pub fn created_address_with_hash(&self, nonce: u64, hash: &B256) -> Address {
        match self.scheme {
            CreateScheme::Create => self.create(nonce),
            CreateScheme::Create2 { salt } => self.caller.1.create2(salt.to_be_bytes(), hash),
        }
    }

    // Modified CREATE address on booster chains
    fn create(&self, nonce: u64) -> Address {
        // If we're not on L1, change the CREATE address to contain the chain_id
        let chain_id = if self.caller.0 != 1 {
            Some(self.caller.0)
        } else {
            None
        };
        self.caller.1.create(nonce, chain_id)
    }*/
}
