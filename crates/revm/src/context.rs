mod context_precompiles;
pub(crate) mod evm_context;
mod inner_evm_context;

pub use context_precompiles::{
    ContextPrecompile, ContextPrecompiles, ContextStatefulPrecompile, ContextStatefulPrecompileArc,
    ContextStatefulPrecompileBox, ContextStatefulPrecompileMut,
};
pub use evm_context::EvmContext;
pub use inner_evm_context::InnerEvmContext;
use revm_interpreter::{as_u64_saturated, Eip7702CodeLoad, StateLoad};

use crate::{
    db::{SyncDatabase as Database, EmptyDB},
    interpreter::{AccountLoad, Host, SStoreResult, SelfDestructResult},
    primitives::{Address, Bytes, ChainAddress, Env, HandlerCfg, Log, B256, BLOCK_HASH_HISTORY, U256, XCallData},
};
use std::boxed::Box;

/// Main Context structure that contains both EvmContext and External context.
pub struct Context<EXT, DB: Database> {
    /// Evm Context (internal context).
    pub evm: EvmContext<DB>,
    /// External contexts.
    pub external: EXT,
}

impl<EXT: Clone, DB: Database + Clone> Clone for Context<EXT, DB>
where
    DB::Error: Clone,
{
    fn clone(&self) -> Self {
        Self {
            evm: self.evm.clone(),
            external: self.external.clone(),
        }
    }
}

impl Default for Context<(), EmptyDB> {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl Context<(), EmptyDB> {
    /// Creates empty context. This is useful for testing.
    pub fn new_empty() -> Context<(), EmptyDB> {
        Context {
            evm: EvmContext::new(EmptyDB::new()),
            external: (),
        }
    }
}

impl<DB: Database> Context<(), DB> {
    /// Creates new context with database.
    pub fn new_with_db(db: DB) -> Context<(), DB> {
        Context {
            evm: EvmContext::new_with_env(db, Box::default()),
            external: (),
        }
    }
}

impl<EXT, DB: Database> Context<EXT, DB> {
    /// Creates new context with external and database.
    pub fn new(evm: EvmContext<DB>, external: EXT) -> Context<EXT, DB> {
        Context { evm, external }
    }
}

/// Context with handler configuration.
pub struct ContextWithHandlerCfg<EXT, DB: Database> {
    /// Context of execution.
    pub context: Context<EXT, DB>,
    /// Handler configuration.
    pub cfg: HandlerCfg,
}

impl<EXT, DB: Database> ContextWithHandlerCfg<EXT, DB> {
    /// Creates new context with handler configuration.
    pub fn new(context: Context<EXT, DB>, cfg: HandlerCfg) -> Self {
        Self { cfg, context }
    }
}

impl<EXT: Clone, DB: Database + Clone> Clone for ContextWithHandlerCfg<EXT, DB>
where
    DB::Error: Clone,
{
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            cfg: self.cfg,
        }
    }
}

impl<EXT, DB: Database> Host for Context<EXT, DB> {
    /// Returns reference to Environment.
    #[inline]
    fn env(&self) -> &Env {
        &self.evm.env
    }

    fn env_mut(&mut self) -> &mut Env {
        &mut self.evm.env
    }

    fn block_hash(&mut self, chain_id: u64, requested_number: u64) -> Option<B256> {
        let block_number = as_u64_saturated!(self.env().block.number);

        let Some(diff) = block_number.checked_sub(requested_number) else {
            return Some(B256::ZERO);
        };

        // blockhash should push zero if number is same as current block number.
        if diff == 0 {
            return Some(B256::ZERO);
        }

        if diff <= BLOCK_HASH_HISTORY {
            return self
                .evm
                .block_hash(chain_id, requested_number)
                .map_err(|e| self.evm.error = Err(e))
                .ok();
        }

        Some(B256::ZERO)
    }

    fn load_account_delegated(&mut self, address: ChainAddress) -> Option<AccountLoad> {
        self.evm
            .load_account_delegated(address)
            .map_err(|e| self.evm.error = Err(e))
            .ok()
    }

    fn balance(&mut self, address: ChainAddress) -> Option<StateLoad<U256>> {
        self.evm
            .balance(address)
            .map_err(|e| self.evm.error = Err(e))
            .ok()
    }

    fn code(&mut self, address: ChainAddress) -> Option<Eip7702CodeLoad<Bytes>> {
        self.evm
            .code(address)
            .map_err(|e| self.evm.error = Err(e))
            .ok()
    }

    fn code_hash(&mut self, address: ChainAddress) -> Option<Eip7702CodeLoad<B256>> {
        self.evm
            .code_hash(address)
            .map_err(|e| self.evm.error = Err(e))
            .ok()
    }

    fn sload(&mut self, address: ChainAddress, index: U256) -> Option<StateLoad<U256>> {
        self.evm
            .sload(address, index)
            .map_err(|e| self.evm.error = Err(e))
            .ok()
    }

    fn sstore(
        &mut self,
        address: ChainAddress,
        index: U256,
        value: U256,
    ) -> Option<StateLoad<SStoreResult>> {
        self.evm
            .sstore(address, index, value)
            .map_err(|e| self.evm.error = Err(e))
            .ok()
    }

    fn tload(&mut self, address: ChainAddress, index: U256) -> U256 {
        self.evm.tload(address, index)
    }

    fn tstore(&mut self, address: ChainAddress, index: U256, value: U256) {
        self.evm.tstore(address, index, value)
    }

    fn log(&mut self, log: Log) {
        self.evm.journaled_state.log(log);
    }

    fn selfdestruct(
        &mut self,
        address: ChainAddress,
        target: ChainAddress,
    ) -> Option<StateLoad<SelfDestructResult>> {
        self.evm
            .inner
            .journaled_state
            .selfdestruct(address, target, &mut self.evm.inner.db)
            .map_err(|e| self.evm.error = Err(e))
            .ok()
    }

    fn xcall(&mut self, xcall: XCallData) {
        self.evm.journaled_state.xcall(0, 0, xcall);
    }
}
