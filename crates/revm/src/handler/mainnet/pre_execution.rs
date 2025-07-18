//! Handles related to the main function of the EVM.
//!
//! They handle initial setup of the EVM, call loop and the final return of the EVM

use crate::{
    precompile::PrecompileSpecId,
    primitives::{
        db::SyncDatabase as Database,
        eip7702, Account, Bytecode, ChainAddress, EVMError, Env, Spec,
        SpecId::{CANCUN, PRAGUE, SHANGHAI},
        TransactTo, TxKind, BLOCKHASH_STORAGE_ADDRESS, U256,
    },
    Context, ContextPrecompiles,
};

/// Main precompile load
#[inline]
pub fn load_precompiles<SPEC: Spec, DB: Database>() -> ContextPrecompiles<DB> {
    ContextPrecompiles::new(PrecompileSpecId::from_spec_id(SPEC::SPEC_ID))
}

/// Main load handle
#[inline]
pub fn load_accounts<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    chain_id: u64,
) -> Result<(), EVMError<DB::Error>> {
    // set journaling state flag.
    context.evm.journaled_state.set_spec_id(SPEC::SPEC_ID);

    // load coinbase
    // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
    if SPEC::enabled(SHANGHAI) {
        let coinbase = context.evm.inner.env.block.coinbase;
        context
            .evm
            .journaled_state
            .warm_preloaded_addresses
            .insert(coinbase);
    }

    // Load blockhash storage address
    // EIP-2935: Serve historical block hashes from state
    if SPEC::enabled(PRAGUE) {
        context
            .evm
            .journaled_state
            .warm_preloaded_addresses
            .insert(ChainAddress(chain_id, BLOCKHASH_STORAGE_ADDRESS));
    }

    // Load access list
    context.evm.load_access_list(chain_id)?;
    Ok(())
}

/// Helper function that deducts the caller balance.
#[inline]
pub fn deduct_caller_inner<SPEC: Spec>(caller_account: &mut Account, env: &Env) {
    // Subtract gas costs from the caller's account.
    // We need to saturate the gas cost to prevent underflow in case that `disable_balance_check` is enabled.
    let mut gas_cost = U256::from(env.tx.gas_limit).saturating_mul(env.effective_gas_price());

    // EIP-4844
    if SPEC::enabled(CANCUN) {
        let data_fee = env.calc_data_fee().expect("already checked");
        gas_cost = gas_cost.saturating_add(data_fee);
    }

    // set new caller account balance.
    caller_account.info.balance = caller_account.info.balance.saturating_sub(gas_cost);

    // bump the nonce for calls. Nonce for CREATE will be bumped in `handle_create`.
    if matches!(env.tx.transact_to, TransactTo::Call(_)) {
        // Nonce is already checked
        caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);
    }

    // touch account so we know it is changed.
    caller_account.mark_touch();
}

/// Deducts the caller balance to the transaction limit.
#[inline]
pub fn deduct_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    // load caller's account.
    let caller_account = context
        .evm
        .inner
        .journaled_state
        .load_account(context.evm.inner.env.tx.caller, &mut context.evm.inner.db)?;

    // deduct gas cost from caller's account.
    deduct_caller_inner::<SPEC>(caller_account.data, &context.evm.inner.env);

    Ok(())
}

/// Apply EIP-7702 auth list and return number gas refund on already created accounts.
#[inline]
pub fn apply_eip7702_auth_list<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<u64, EVMError<DB::Error>> {
    // EIP-7702. Load bytecode to authorized accounts.
    if !SPEC::enabled(PRAGUE) {
        return Ok(0);
    }

    // return if there is no auth list.
    let Some(authorization_list) = context.evm.inner.env.tx.authorization_list.as_ref() else {
        return Ok(0);
    };

    let mut refunded_accounts = 0;
    for authorization in authorization_list.recovered_iter() {
        // 1. recover authority and authorized addresses.
        // authority = ecrecover(keccak(MAGIC || rlp([chain_id, address, nonce])), y_parity, r, s]
        let Some(authority) = authorization.authority() else {
            continue;
        };

        // 2. Verify the chain id is either 0 or the chain's current ID.
        if !authorization.chain_id().is_zero()
            && !context.evm.inner.env.tx.chain_ids.clone().unwrap_or_default().contains(&authorization.chain_id().as_limbs()[0])
        {
            continue;
        }

        // warm authority account and check nonce.
        // 3. Add authority to accessed_addresses (as defined in EIP-2929.)
        let mut authority_acc = context
            .evm
            .inner
            .journaled_state
            .load_code(ChainAddress(authorization.chain_id().as_limbs()[0], authority), &mut context.evm.inner.db)?;

        // 4. Verify the code of authority is either empty or already delegated.
        if let Some(bytecode) = &authority_acc.info.code {
            // if it is not empty and it is not eip7702
            if !bytecode.is_empty() && !bytecode.is_eip7702() {
                continue;
            }
        }

        // 5. Verify the nonce of authority is equal to nonce.
        if authorization.nonce() != authority_acc.info.nonce {
            continue;
        }

        // 6. Refund the sender PER_EMPTY_ACCOUNT_COST - PER_AUTH_BASE_COST gas if authority exists in the trie.
        if !authority_acc.is_empty() {
            refunded_accounts += 1;
        }

        // 7. Set the code of authority to be 0xef0100 || address. This is a delegation designation.
        let bytecode = Bytecode::new_eip7702(authorization.address);
        authority_acc.info.code_hash = bytecode.hash_slow();
        authority_acc.info.code = Some(bytecode);

        // 8. Increase the nonce of authority by one.
        authority_acc.info.nonce = authority_acc.info.nonce.saturating_add(1);
        authority_acc.mark_touch();
    }

    let refunded_gas =
        refunded_accounts * (eip7702::PER_EMPTY_ACCOUNT_COST - eip7702::PER_AUTH_BASE_COST);

    Ok(refunded_gas)
}
