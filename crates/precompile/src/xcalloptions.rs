use revm_primitives::{Address, Bytes, CallOptions, ChainAddress, Env, PrecompileOutput};
use crate::{Error, Precompile, PrecompileResult, PrecompileWithAddress, CtxPrecompileFn};

pub const XCALLOPTIONS: PrecompileWithAddress = PrecompileWithAddress(
    crate::u64_to_address(1000),
    Precompile::Ctx(xcalloptions_run as CtxPrecompileFn),
);

/// Sets the xcall options
fn xcalloptions_run(input: &[u8], _gas_limit: u64, _env: &Env, call_options: &mut Option<CallOptions>) -> PrecompileResult {
    // Verify input length.
    if input.len() < 83 {
        return Err(Error::XCallOptionsInvalidInputLength.into());
    }

    // Read the input data
    let version = u16::from_le_bytes(input[0..2].try_into().unwrap());
    let chain_id = u64::from_le_bytes(input[2..10].try_into().unwrap());
    let sandbox = input[10] != 0;
    let tx_origin = Address(input[11..31].try_into().unwrap());
    let msg_sender = Address(input[31..51].try_into().unwrap());
    let block_hash = Some(input[51..83].try_into().unwrap());
    let proof = &input[83..];

    // Check the version
    if version != 1 {
        return Err(Error::XCallOptionsInvalidInputLength.into());
    }

    // Set the call options
    *call_options = Some(CallOptions {
        chain_id,
        sandbox,
        tx_origin: ChainAddress(chain_id, tx_origin),
        msg_sender: ChainAddress(chain_id, msg_sender),
        block_hash,
        proof: proof.to_vec(),
    });

    Ok(PrecompileOutput::new(0, Bytes::default()))
}
