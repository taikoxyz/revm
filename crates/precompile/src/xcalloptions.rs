use revm_primitives::{address, Address, Bytes, CallOptions, ChainAddress, Env, PrecompileOutput};
use crate::{Error, Precompile, PrecompileResult, PrecompileWithAddress, CtxPrecompileFn};

pub const XCALLOPTIONS: PrecompileWithAddress = PrecompileWithAddress(
    crate::u64_to_address(1234),
    Precompile::Ctx(xcalloptions_run as CtxPrecompileFn),
);

/// Sets the xcall options
fn xcalloptions_run(input: &[u8], _gas_limit: u64, _env: &Env, call_options: &mut Option<CallOptions>) -> PrecompileResult {
    println!("xcalloptions_run");
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
    println!("CallOptions: {:?}", call_options);

    let prefix = b"XCallOptions";
    let output = Bytes::copy_from_slice(&vec![prefix.to_vec(), input.to_vec()].concat());

    Ok(PrecompileOutput::new(0, output))
}


#[test]
fn test_xcalloptions() {
    let prefix = b"XCallOptions";
    String::from_utf8(prefix.to_vec()).unwrap();

    let input = [0u8; 95];
    vec![prefix.to_vec(), input.to_vec()].concat();
    let output = Bytes::copy_from_slice(&vec![prefix.to_vec(), input.to_vec()].concat());

    let prefix = String::from_utf8(output[0..12].to_vec()).unwrap();
    assert_eq!(prefix, "XCallOptions");
    let version = u16::from_le_bytes(input[12..14].try_into().unwrap());  
    let chain_id = u64::from_le_bytes(input[14..22].try_into().unwrap());  
    let sandbox = input[22] != 0;  
    let tx_origin = Address(input[23..43].try_into().unwrap());  
    let msg_sender = Address(input[43..63].try_into().unwrap());  
    let block_hash = Some(input[63..95].try_into().unwrap());  
    let proof = &output[95..];

    let co = CallOptions {
        chain_id,
        sandbox,
        tx_origin: ChainAddress(chain_id, tx_origin),
        msg_sender: ChainAddress(chain_id, msg_sender),
        block_hash,
        proof: proof.to_vec(),
    };
    println!("CallOptions: {:?}", co);
}