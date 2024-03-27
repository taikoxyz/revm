use revm_primitives::ChainAddress;

pub use crate::primitives::CreateScheme;
use crate::primitives::{Address, Bytes, TransactTo, TxEnv, U256};
use core::ops::Range;
use std::boxed::Box;

/// Inputs for a call.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallInputs {
    /// Booster: The target chain of the call
    pub chain_id: u64,
    /// The target of the call.
    pub contract: ChainAddress,
    /// The transfer, if any, in this call.
    pub transfer: Transfer,
    /// The call data of the call.
    pub input: Bytes,
    /// The gas limit of the call.
    pub gas_limit: u64,
    /// The context of the call.
    pub context: CallContext,
    /// Whether this is a static call.
    pub is_static: bool,
    /// The return memory offset where the output of the call is written.
    pub return_memory_offset: Range<usize>,
    /// Booster: Whether this is a sandboxed call.
    pub is_sandboxed: bool,
}

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

impl CallInputs {
    /// Creates new call inputs.
    pub fn new(tx_env: &TxEnv, gas_limit: u64) -> Option<Self> {
        let TransactTo::Call(address) = tx_env.transact_to else {
            return None;
        };

        Some(CallInputs {
            contract: address,
            transfer: Transfer {
                source: tx_env.caller,
                target: address,
                value: tx_env.value,
            },
            input: tx_env.data.clone(),
            gas_limit,
            context: CallContext {
                caller: tx_env.caller,
                address,
                code_address: address,
                apparent_value: tx_env.value,
                scheme: CallScheme::Call,
            },
            is_static: false,
            return_memory_offset: 0..0,
        })
    }

    /// Returns boxed call inputs.
    pub fn new_boxed(tx_env: &TxEnv, gas_limit: u64) -> Option<Box<Self>> {
        Self::new(tx_env, gas_limit).map(Box::new)
    }
}

impl CreateInputs {
    /// Creates new create inputs.
    pub fn new(tx_env: &TxEnv, gas_limit: u64) -> Option<Self> {
        let TransactTo::Create(scheme) = tx_env.transact_to else {
            return None;
        };

        Some(CreateInputs {
            caller: tx_env.caller,
            scheme,
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
            CreateScheme::Create => self.create(nonce),
            CreateScheme::Create2 { salt } => self
                .caller.1
                .create2_from_code(salt.to_be_bytes(), &self.init_code),
        }
    }

    /// Returns the address that this create call will create, without calculating the init code hash.
    ///
    /// Note: `hash` must be `keccak256(&self.init_code)`.
    pub fn created_address_with_hash(&self, nonce: u64, hash: &B256) -> Address {
        match self.scheme {
            CreateScheme::Create => self.create(nonce),
            CreateScheme::Create2 { salt } => self.caller.1.create2(salt.to_be_bytes(), hash),
        }
    }

    /// Modified CREATE address on booster chains
    fn create(&self, nonce: u64) -> Address {
        // If we're not on L1, change the CREATE address to contain the chain_id
        let chain_id = if self.caller.0 != 1 {
            Some(self.caller.0)
        } else {
            None
        };
        self.caller.1.create(nonce, chain_id)
    }
}

/// Call schemes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CallScheme {
    /// `CALL`.
    Call,
    /// `CALLCODE`
    CallCode,
    /// `DELEGATECALL`
    DelegateCall,
    /// `STATICCALL`
    StaticCall,
}

/// Context of a runtime call.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallContext {
    /// Execution address.
    pub address: ChainAddress,
    /// Caller address of the EVM.
    pub caller: ChainAddress,
    /// The address the contract code was loaded from, if any.
    pub code_address: ChainAddress,
    /// Apparent value of the EVM.
    pub apparent_value: U256,
    /// The scheme used for the call.
    pub scheme: CallScheme,
}

impl Default for CallContext {
    fn default() -> Self {
        CallContext {
            address: ChainAddress::default(),
            caller: ChainAddress::default(),
            code_address: ChainAddress::default(),
            apparent_value: U256::default(),
            scheme: CallScheme::Call,
        }
    }
}

/// Transfer from source to target, with given value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transfer {
    /// The source address.
    pub source: ChainAddress,
    /// The target address.
    pub target: ChainAddress,
    /// The transfer value.
    pub value: U256,
}

/// Result of a call that resulted in a self destruct.
#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelfDestructResult {
    pub had_value: bool,
    pub target_exists: bool,
    pub is_cold: bool,
    pub previously_destroyed: bool,
}
