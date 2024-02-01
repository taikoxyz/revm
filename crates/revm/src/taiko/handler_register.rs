//! Handler related to Taiko chain

use crate::{
    handler::{
        mainnet::{self, deduct_caller, deduct_caller_inner, end, last_frame_return, output},
        register::EvmHandler,
    },
    interpreter::{return_ok, return_revert, Gas, InstructionResult},
    primitives::{
        db::Database, spec_to_generic, Account, EVMError, Env, ExecutionResult, HaltReason,
        HashMap, InvalidTransaction, Output, ResultAndState, Spec, SpecId, U256,
    },
    Context,
};
use alloc::sync::Arc;
use core::ops::Mul;
use SpecId::LONDON;

pub fn taiko_handle_register<DB: Database, EXT>(handler: &mut EvmHandler<'_, EXT, DB>) {
    spec_to_generic!(handler.spec_id, {
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
    let caller = context.evm.env.tx.caller;
    let effective_gas_price = context.evm.env.effective_gas_price();

    // return balance of not spend gas.
    let (caller_account, _) = context
        .evm
        .journaled_state
        .load_account(caller, &mut context.evm.db)
        .map_err(EVMError::Database)?;

    caller_account.info.balance = caller_account
        .info
        .balance
        .saturating_add(effective_gas_price * U256::from(gas.remaining() + gas.refunded() as u64));

    Ok(())
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
    let beneficiary = context.evm.env.block.coinbase;
    let effective_gas_price = context.evm.env.effective_gas_price();

    // transfer fee to coinbase/beneficiary.
    // EIP-1559 discard basefee for coinbase transfer. Basefee amount of gas is discarded.
    let coinbase_gas_price = if SPEC::enabled(LONDON) {
        effective_gas_price.saturating_sub(context.evm.env.block.basefee)
    } else {
        effective_gas_price
    };

    let (coinbase_account, _) = context
        .evm
        .journaled_state
        .load_account(beneficiary, &mut context.evm.db)
        .map_err(EVMError::Database)?;

    coinbase_account.mark_touch();
    coinbase_account.info.balance = coinbase_account
        .info
        .balance
        .saturating_add(coinbase_gas_price * U256::from(gas.spend() - gas.refunded() as u64));

    let treasury = context.evm.env.tx.taiko.treasury;
    let basefee = context.evm.env.block.basefee;

    let (treasury_account, _) = context
        .evm
        .journaled_state
        .load_account(treasury, &mut context.evm.db)
        .map_err(EVMError::Database)?;

    treasury_account.mark_touch();
    treasury_account.info.balance = treasury_account
        .info
        .balance
        .saturating_add(basefee * U256::from(gas.spend() - gas.refunded() as u64));
    Ok(())
}
