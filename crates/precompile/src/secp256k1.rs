use crate::{utilities::right_pad, Error, Precompile, PrecompileResult, PrecompileWithAddress};
use alloc::vec::Vec;
use revm_primitives::{alloy_primitives::B512, B256};
use core::cmp::min;

pub const ECRECOVER: PrecompileWithAddress = PrecompileWithAddress(
    crate::u64_to_address(1),
    Precompile::Standard(ec_recover_run),
);

#[cfg(all(target_os = "zkvm", target_vendor = "succinct"))]
#[allow(clippy::module_inception)]
mod secp256k1 {
    use crate::B256;
    use revm_primitives::keccak256;

    pub fn ecrecover(sig: &[u8; 65], msg: &B256) -> Result<B256, anyhow::Error> {
        let recovered_key = sp1_precompiles::secp256k1::ecrecover(sig, msg)?;

        let mut hash = keccak256(&recovered_key[1..]);

        // truncate to 20 bytes
        hash[..12].fill(0);
        Ok(hash)
    } 
}

#[cfg(all(
    not(all(target_os = "zkvm", target_vendor = "succinct")),
    not(feature = "secp256k1")
))]
#[allow(clippy::module_inception)]
mod secp256k1 {
    use crate::B256;
    use k256::ecdsa::{Error, RecoveryId, Signature, VerifyingKey};
    use revm_primitives::keccak256;

    // Silence the unused crate dependency warning.
    use anyhow as _;

    pub fn ecrecover(sig: &[u8; 65], msg: &B256) -> Result<B256, Error> {
        // parse signature
        let mut recid = sig[64];
        let mut sig = Signature::from_slice(&sig[..64])?;

        // normalize signature and flip recovery id if needed.
        if let Some(sig_normalized) = sig.normalize_s() {
            sig = sig_normalized;
            recid = recid ^ 1;
        };
        let recid = RecoveryId::from_byte(recid).expect("Recovery id is valid");

        // recover key
        let recovered_key = VerifyingKey::recover_from_prehash(&msg[..], &sig, recid)?;
        // hash it
        let mut hash = keccak256(
            &recovered_key
                .to_encoded_point(/* compress = */ false)
                .as_bytes()[1..],
        );

        // truncate to 20 bytes
        hash[..12].fill(0);
        Ok(hash)
    }
}

#[cfg(all(
    not(all(target_os = "zkvm", target_vendor = "succinct")),
    feature = "secp256k1"
))]
#[allow(clippy::module_inception)]
mod secp256k1 {
    use crate::B256;
    use revm_primitives::keccak256;
    use secp256k1::{
        ecdsa::{RecoverableSignature, RecoveryId},
        Message, Secp256k1,
    };

    // Silence the unused crate dependency warning.
    use anyhow as _;
    use k256 as _;
    use sp1_precompiles as _;

    pub fn ecrecover(sig: &[u8; 65], msg: &B256) -> Result<B256, secp256k1::Error> {
        let sig =
            RecoverableSignature::from_compact(&sig[0..64], RecoveryId::from_i32(sig[64] as i32)?)?;

        let secp = Secp256k1::new();
        let public = secp.recover_ecdsa(&Message::from_digest_slice(&msg[..])?, &sig)?;

        let mut hash = keccak256(&public.serialize_uncompressed()[1..]);
        hash[..12].fill(0);
        Ok(hash)
    }
}

fn ec_recover_run(i: &[u8], target_gas: u64) -> PrecompileResult {

    const ECRECOVER_BASE: u64 = 3_000;

    if ECRECOVER_BASE > target_gas {
        return Err(Error::OutOfGas);
    }
    let mut input = [0u8; 128];
    input[..min(i.len(), 128)].copy_from_slice(&i[..min(i.len(), 128)]);

    let msg = B256::from_slice(&input[0..32]);

    let mut sig = [0u8; 65];
    sig[0..64].copy_from_slice(&input[64..128]);

    if input[32..63] != [0u8; 31] || !matches!(input[63], 27 | 28) {
        return Ok((ECRECOVER_BASE, Vec::new()));
    }

    sig[64] = input[63] - 27;

    let out = secp256k1::ecrecover(&sig, &msg)
        .map(|o| o.to_vec())
        .unwrap_or_default();

    Ok((ECRECOVER_BASE, out))
}
