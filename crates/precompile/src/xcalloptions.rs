use core::str::FromStr;

use revm_primitives::{address, Address, Bytes, CallOptions, ChainAddress, Env, PrecompileOutput, StatefulPrecompile};
use crate::{Error, Precompile, PrecompileResult, PrecompileWithAddress, CtxPrecompileFn};

pub const XCALLOPTIONS: PrecompileWithAddress = PrecompileWithAddress(
    crate::u64_to_address(1234),
    Precompile::Ctx(xcalloptions_run as CtxPrecompileFn),
);

/// Sets the xcall options
fn xcalloptions_run(input: &[u8], _gas_limit: u64, env: &Env, caller: ChainAddress, call_options: &mut Option<CallOptions>) -> PrecompileResult {
    println!("  xcalloptions_run: {}, {:?}", input.len(), input);

    // Verify input length.
    if input.len() < 83 {
        return Err(Error::XCallOptionsInvalidInputLength.into());
    }

    // Read the input data
    let version = u16::from_be_bytes(input[0..2].try_into().unwrap());
    let chain_id = u64::from_be_bytes(input[2..10].try_into().unwrap());
    let sandbox = input[10] != 0;
    let tx_origin = Address(input[11..31].try_into().unwrap());
    let msg_sender = Address(input[31..51].try_into().unwrap());
    let block_hash: Option<revm_primitives::FixedBytes<32>> = Some(input[51..83].try_into().unwrap());
    let proof = &input[83..];

    // Check the version
    if version != 1 {
        return Err(Error::XCallOptionsInvalidVersion.into());
    }

    if !sandbox && !env.cfg.allow_mocking {
        // env.tx.caller is the Signer of the transaction
        // caller is the address of the contract that is calling the precompile
        if tx_origin != env.tx.caller.1 || msg_sender != caller.1 {
            println!("  tx_origin: {:?}, env.tx.caller.1: {:?}, msg_sender: {:?}, caller.1: {:?}", tx_origin, env.tx.caller.1, msg_sender, caller.1);
            return Err(Error::XCallOptionsInvalidOrigin.into());
        }
    }

    // Set the call options
    *call_options = Some(CallOptions {
        chain_id,
        sandbox,
        tx_origin: ChainAddress(chain_id, tx_origin),
        msg_sender: ChainAddress(caller.0, msg_sender),
        block_hash,
        proof: proof.to_vec(),
    });
    println!("  CallOptions: {:?}", call_options);

    Ok(PrecompileOutput::new(0, Bytes::from_static(&[0x6c, 0x54, 0x13, 0x30])))
}
