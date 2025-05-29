use revm::{
    context::Cfg,
    handler::{EthPrecompiles, PrecompileProvider},
    interpreter::{Gas, InputsImpl, InstructionResult, InterpreterResult},
    precompile::{PrecompileError, PrecompileSpecId, Precompiles},
    primitives::{hardfork::SpecId, Address, Bytes},
};
use std::boxed::Box;
use std::string::String;

use crate::{context::GwynethContextTr, xcall};

// Gwyneth precompile provider
#[derive(Default, Debug, Clone)]
pub struct GwynethPrecompiles {
    /// Inner precompile provider is same as Ethereums.
    inner: EthPrecompiles,
}

impl GwynethPrecompiles {
    /// Create a new precompile provider with the given OpSpec.
    #[inline]
    pub fn new_with_spec(spec: SpecId) -> Self {
        Self {
            inner: EthPrecompiles {
                precompiles: Precompiles::new(PrecompileSpecId::from_spec_id(spec)),
                spec,
            },
        }
    }

    // Precompiles getter.
    #[inline]
    pub fn precompiles(&self) -> &'static Precompiles {
        self.inner.precompiles
    }

    /// Check if the current spec is BERLIN.
    /// We need attach the xcall precompile in BERLIN fork
    pub fn is_in_berlin(&self) -> bool {
        self.inner.spec.is_enabled_in(SpecId::BERLIN)
    }
}

impl<CTX: GwynethContextTr> PrecompileProvider<CTX> for GwynethPrecompiles {
    type Output = InterpreterResult;

    #[inline]
    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) -> bool {
        <EthPrecompiles as PrecompileProvider<CTX>>::set_spec(&mut self.inner, spec)
    }

    #[inline]
    fn run(
        &mut self,
        context: &mut CTX,
        address: &Address,
        inputs: &InputsImpl,
        is_static: bool,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, String> {
        if self.is_in_berlin() && *address == xcall::XCALL_ADDRESS {
            let mut result = InterpreterResult {
                result: InstructionResult::Return,
                gas: Gas::new(gas_limit),
                output: Bytes::new(),
            };
            match xcall::run_xcall(inputs, gas_limit, context, inputs.caller_address) {
                Ok(output) => {
                    let underflow = result.gas.record_cost(output.gas_used);
                    assert!(underflow, "Gas underflow is not possible");
                    result.result = InstructionResult::Return;
                    result.output = output.bytes;
                }
                Err(PrecompileError::Fatal(e)) => return Err(e),
                Err(e) => {
                    result.result = if e.is_oog() {
                        InstructionResult::PrecompileOOG
                    } else {
                        InstructionResult::PrecompileError
                    };
                }
            }
            return Ok(Some(result));
        }
        self.inner
            .run(context, address, inputs, is_static, gas_limit)
    }

    #[inline]
    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>> {
        self.inner.warm_addresses()
    }

    #[inline]
    fn contains(&self, address: &Address) -> bool {
        self.inner.contains(address)
    }
}
