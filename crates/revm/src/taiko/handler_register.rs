//! Handler related to Taiko chain

use crate::{
    handler::{
        mainnet::{self, deduct_caller_inner},
        register::EvmHandler,
    },
    interpreter::Gas,
    primitives::{db::Database, spec_to_generic, EVMError, Spec, SpecId, U256},
    Context,
};
extern crate alloc;
use alloc::sync::Arc;

pub fn taiko_handle_register<DB: Database, EXT>(handler: &mut EvmHandler<'_, EXT, DB>) {
    spec_to_generic!(handler.cfg.spec_id, {
        handler.pre_execution.deduct_caller = Arc::new(deduct_caller::<SPEC, EXT, DB>);
        handler.post_execution.reimburse_caller = Arc::new(reimburse_caller::<SPEC, EXT, DB>);
        handler.post_execution.reward_beneficiary = Arc::new(reward_beneficiary::<SPEC, EXT, DB>);
    });
}

#[inline]
pub fn reimburse_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    if context.evm.env.tx.taiko.is_anchor {
        return Ok(());
    }
    mainnet::reimburse_caller::<SPEC, EXT, DB>(context, gas)
}

/// Reward beneficiary with gas fee.
#[inline]
pub fn reward_beneficiary<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    if context.evm.env.tx.taiko.is_anchor {
        return Ok(());
    }

    mainnet::reward_beneficiary::<SPEC, EXT, DB>(context, gas)?;

    let treasury = context.evm.env.tx.taiko.treasury;
    let basefee = context.evm.env.block.basefee;

    let mut account_load = context
        .evm
        .inner
        .journaled_state
        .load_account(treasury, &mut context.evm.inner.db)?;
    account_load.data.mark_touch();
    account_load.info.balance = account_load
        .info
        .balance
        .saturating_add(basefee * U256::from(gas.spent() - gas.refunded() as u64));
    Ok(())
}

/// Deduct max balance from caller
#[inline]
pub fn deduct_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    // load caller's account.
    let mut account_load = context
        .evm
        .inner
        .journaled_state
        .load_account(context.evm.inner.env.tx.caller, &mut context.evm.inner.db)?;

    // deduct gas cost from caller's account.
    deduct_caller_inner::<SPEC>(
        *account_load,
        &context.evm.inner.env,
    );

    Ok(())
}
