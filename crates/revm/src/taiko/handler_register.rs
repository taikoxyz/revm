//! Handler related to Taiko chain

use crate::{
    handler::{
        mainnet::{self, deduct_caller_inner},
        register::EvmHandler,
    },
    interpreter::Gas,
    primitives::{
        db::Database, spec_to_generic, taiko_protocol_spec_to_generic, EVMError, Spec, SpecId,
        TaikoProtocolSpec, TaikoProtocolSpecId, U256
    },
    Context,
};
extern crate alloc;
use alloc::sync::Arc;

pub fn taiko_handle_register<DB: Database, EXT>(handler: &mut EvmHandler<'_, EXT, DB>) {
    spec_to_generic!(handler.cfg.spec_id, {
        handler.pre_execution.deduct_caller = Arc::new(deduct_caller::<SPEC, EXT, DB>);
        handler.post_execution.reimburse_caller = Arc::new(reimburse_caller::<SPEC, EXT, DB>);
        taiko_protocol_spec_to_generic!(handler.cfg.taiko_protocol_spec_id, {
            handler.post_execution.reward_beneficiary =
                Arc::new(reward_beneficiary::<SPEC, TAIKOSPEC, EXT, DB>);
        });
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
pub fn reward_beneficiary<SPEC: Spec, TAIKOSPEC: TaikoProtocolSpec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    if context.evm.env.tx.taiko.is_anchor {
        return Ok(());
    }

    mainnet::reward_beneficiary::<SPEC, EXT, DB>(context, gas)?;

    if TAIKOSPEC::enabled(TaikoProtocolSpecId::ONTAKE) {
        reward_beneficiary_ontake::<SPEC, EXT, DB>(context, gas)
    } else {
        reward_beneficiary_hekla::<SPEC, EXT, DB>(context, gas)
    }
}

fn reward_beneficiary_hekla<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    let treasury = context.evm.env.tx.taiko.treasury;
    let basefee = context.evm.env.block.basefee;

    let (treasury_account, _) = context
        .evm
        .inner
        .journaled_state
        .load_account(treasury, &mut context.evm.inner.db)?;
    treasury_account.mark_touch();
    treasury_account.info.balance = treasury_account
        .info
        .balance
        .saturating_add(basefee * U256::from(gas.spent() - gas.refunded() as u64));
    Ok(())
}

fn reward_beneficiary_ontake<SPEC: Spec, EXT, DB: Database>(
    _context: &mut Context<EXT, DB>,
    _gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    todo!();
}

/// Deduct max balance from caller
#[inline]
pub fn deduct_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    // load caller's account.
    let (caller_account, _) = context
        .evm
        .inner
        .journaled_state
        .load_account(context.evm.inner.env.tx.caller, &mut context.evm.inner.db)?;

    // deduct gas cost from caller's account.
    deduct_caller_inner::<SPEC>(
        caller_account,
        &context.evm.inner.env,
        !context.evm.inner.env.tx.taiko.is_anchor,
    );

    Ok(())
}
