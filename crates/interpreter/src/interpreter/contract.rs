use revm_primitives::{ChainAddress, TransactTo, TxKind};

use super::analysis::to_analysed;
use crate::{
    primitives::{Address, Bytecode, Bytes, Env, B256, U256},
    CallInputs,
};

/// EVM contract information.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Contract {
    /// Contracts data
    pub input: Bytes,
    /// Bytecode contains contract code, size of original code, analysis with gas block and jump table.
    /// Note that current code is extended with push padding and STOP at end.
    pub bytecode: Bytecode,
    /// Bytecode hash for legacy. For EOF this would be None.
    pub hash: Option<B256>,
    /// Target address of the account. Storage of this address is going to be modified.
    pub target_address: ChainAddress,
    /// Address of the account the bytecode was loaded from. This can be different from target_address
    /// in the case of DELEGATECALL or CALLCODE
    pub bytecode_address: Option<ChainAddress>,
    /// Caller of the EVM.
    pub caller: ChainAddress,
    /// Value send to contract from transaction or from CALL opcodes.
    pub call_value: U256,
}

impl Contract {
    /// Instantiates a new contract by analyzing the given bytecode.
    #[inline]
    pub fn new(
        input: Bytes,
        bytecode: Bytecode,
        hash: Option<B256>,
        target_address: ChainAddress,
        bytecode_address: Option<ChainAddress>,
        caller: ChainAddress,
        call_value: U256,
    ) -> Self {
        let bytecode = to_analysed(bytecode);

        Self {
            input,
            bytecode,
            hash,
            target_address,
            bytecode_address,
            caller,
            call_value,
        }
    }

    /// Creates a new contract from the given [`Env`].
    #[inline]
    pub fn new_env(env: &Env, bytecode: Bytecode, hash: Option<B256>) -> Self {
        let contract_address = match env.tx.transact_to {
            TransactTo::Call(caller) => caller,
            TransactTo::Create => ChainAddress(1, Address::ZERO),
        };
        let bytecode_address = match env.tx.transact_to {
            TransactTo::Call(caller) => Some(caller),
            TransactTo::Create => None,
        };
        Self::new(
            env.tx.data.clone(),
            bytecode,
            hash,
            contract_address,
            bytecode_address,
            env.tx.caller,
            env.tx.value,
        )
    }

    /// Creates a new contract from the given inputs.
    #[inline]
    pub fn new_with_context(
        input: Bytes,
        bytecode: Bytecode,
        hash: Option<B256>,
        call_context: &CallInputs,
    ) -> Self {
        //println!("Contract::new_with_context");
        Self::new(
            input,
            bytecode,
            hash,
            call_context.target_address,
            Some(call_context.bytecode_address),
            call_context.caller,
            call_context.call_value(),
        )
    }

    /// Returns whether the given position is a valid jump destination.
    #[inline]
    pub fn is_valid_jump(&self, pos: usize) -> bool {
        self.bytecode
            .legacy_jump_table()
            .map(|i| i.is_valid(pos))
            .unwrap_or(false)
    }
}
