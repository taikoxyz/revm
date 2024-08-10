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

    if SPEC::enabled(SpecId::ONTAKE) {
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

/*
https://github.com/taikoxyz/taiko-geth/blob/60551be44eb3080be9d0ba0c6cf01c6e2a47caf5/core/state_transition.go#L475-L482
Ontake upgrade for basefee sharing:
    totalFee := new(big.Int).Mul(st.evm.Context.BaseFee, new(big.Int).SetUint64(st.gasUsed()))
    feeCoinbase := new(big.Int).Div(
        new(big.Int).Mul(totalFee, new(big.Int).SetUint64(uint64(st.msg.BasefeeSharingPctg))),
        new(big.Int).SetUint64(100),
    )
    feeTreasury := new(big.Int).Sub(totalFee, feeCoinbase)
    st.state.AddBalance(st.getTreasuryAddress(), uint256.MustFromBig(feeTreasury))
    st.state.AddBalance(st.evm.Context.Coinbase, uint256.MustFromBig(feeCoinbase))
*/
fn reward_beneficiary_ontake<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    let basefee_ratio = context.evm.env.tx.taiko.basefee_ratio;
    let treasury = context.evm.env.tx.taiko.treasury;
    let basefee = context.evm.env.block.basefee;

    let (treasury_account, _) = context
        .evm
        .inner
        .journaled_state
        .load_account(treasury, &mut context.evm.inner.db)?;
    treasury_account.mark_touch();
    let total_fee = basefee * U256::from(gas.spent() - gas.refunded() as u64);
    let fee_coinbase = total_fee * U256::from(basefee_ratio) / U256::from(100);
    let fee_treasury = total_fee - fee_coinbase;
    treasury_account.info.balance = treasury_account.info.balance.saturating_add(fee_treasury);

    let beneficiary = context.evm.env.block.coinbase;
    let (coinbase_account, _) = context
        .evm
        .inner
        .journaled_state
        .load_account(beneficiary, &mut context.evm.inner.db)?;
    coinbase_account.mark_touch();
    coinbase_account.info.balance = coinbase_account.info.balance.saturating_add(fee_coinbase);
    Ok(())
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
