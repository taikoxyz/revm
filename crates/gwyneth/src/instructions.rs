use revm::{
    bytecode::opcode::{CALL, CALLCODE, DELEGATECALL, STATICCALL},
    handler::instructions::{EthInstructions, InstructionProvider},
    interpreter::{Host, InstructionTable, InterpreterTypes},
};

use crate::context::GwynethContextTr;

pub struct GwynethInstructions<WIRE: InterpreterTypes, HOST> {
    inner: EthInstructions<WIRE, HOST>,
}

impl<WIRE, HOST> GwynethInstructions<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: GwynethContextTr + Host,
{
    /// Returns `EthInstructions` with mainnet spec.
    pub fn new_mainnet() -> Self {
        let mut inner = EthInstructions::new_mainnet();
        inner.insert_instruction(CALL, hack::call);
        inner.insert_instruction(CALLCODE, hack::call_code);
        inner.insert_instruction(DELEGATECALL, hack::delegate_call);
        inner.insert_instruction(STATICCALL, hack::static_call);
        GwynethInstructions { inner }
    }
}

impl<IT, CTX> InstructionProvider for GwynethInstructions<IT, CTX>
where
    IT: InterpreterTypes,
    CTX: Host,
{
    type InterpreterTypes = IT;
    type Context = CTX;

    fn instruction_table(&self) -> &InstructionTable<Self::InterpreterTypes, Self::Context> {
        &self.inner.instruction_table
    }
}

mod hack {
    use crate::{context::GwynethContextTr, xcall::XCallOptions};
    use revm::{
        context::Cfg,
        interpreter::{
            gas,
            instructions::{
                contract::{calc_call_gas, get_memory_input_and_out_ranges},
                utility::IntoAddress,
            },
            interpreter_types::{InputsTr, LoopControl, RuntimeFlag, StackTr},
            popn, CallInput, CallInputs, CallScheme, CallValue, FrameInput, Host,
            InstructionContext, InstructionResult, InterpreterAction, InterpreterTypes,
        },
        primitives::Address,
    };

    pub fn call<WIRE: InterpreterTypes, H: GwynethContextTr + Host>(
        context: InstructionContext<'_, H, WIRE>,
    ) {
        popn!([local_gas_limit, to, value], context.interpreter);
        let to = to.into_address();
        // Get the target
        let call_targets = apply_xcall_options::<WIRE, H>(
            InstructionContext {
                host: context.host,
                interpreter: context.interpreter,
            },
            to,
            false,
            false,
        );
        // Max gas limit is not possible in real ethereum situation.
        let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

        let has_transfer = !value.is_zero();
        if context.interpreter.runtime_flag.is_static() && has_transfer {
            context
                .interpreter
                .control
                .set_instruction_result(InstructionResult::CallNotAllowedInsideStatic);
            return;
        }

        let Some((input, return_memory_offset)) =
            get_memory_input_and_out_ranges(context.interpreter)
        else {
            return;
        };

        let Some(account_load) = context
            .host
            .load_account_delegated(call_targets.bytecode_address)
        else {
            context
                .interpreter
                .control
                .set_instruction_result(InstructionResult::FatalExternalError);
            return;
        };

        let Some(mut gas_limit) = calc_call_gas(
            context.interpreter,
            account_load,
            has_transfer,
            local_gas_limit,
        ) else {
            return;
        };

        gas!(context.interpreter, gas_limit);

        // Add call stipend if there is value to be transferred.
        if has_transfer {
            gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
        }

        // Call host to interact with target contract
        context.interpreter.control.set_next_action(
            InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
                input: CallInput::SharedBuffer(input),
                gas_limit,
                target_address: call_targets.target_address,
                caller: call_targets.caller,
                bytecode_address: call_targets.bytecode_address,
                value: CallValue::Transfer(value),
                scheme: CallScheme::Call,
                is_static: context.interpreter.runtime_flag.is_static(),
                is_eof: false,
                return_memory_offset,
            }))),
            InstructionResult::CallOrCreate,
        );
    }

    pub fn call_code<WIRE: InterpreterTypes, H: Host + ?Sized>(
        context: InstructionContext<'_, H, WIRE>,
    ) {
        popn!([local_gas_limit, to, value], context.interpreter);
        let to = Address::from_word(B256::from(to));
        // Get the target
        let call_targets = apply_xcall_options::<WIRE, H>(
            InstructionContext {
                host: context.host,
                interpreter: context.interpreter,
            },
            to,
            false,
            true,
        );
        // Max gas limit is not possible in real ethereum situation.
        let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

        //pop!(context.interpreter, value);
        let Some((input, return_memory_offset)) =
            get_memory_input_and_out_ranges(context.interpreter)
        else {
            return;
        };

        let Some(mut load) = context
            .host
            .load_account_delegated(call_targets.bytecode_address)
        else {
            context
                .interpreter
                .control
                .set_instruction_result(InstructionResult::FatalExternalError);
            return;
        };

        // Set `is_empty` to false as we are not creating this account.
        load.is_empty = false;
        let Some(mut gas_limit) =
            calc_call_gas(context.interpreter, load, !value.is_zero(), local_gas_limit)
        else {
            return;
        };

        gas!(context.interpreter, gas_limit);

        // Add call stipend if there is value to be transferred.
        if !value.is_zero() {
            gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
        }

        // Call host to interact with target contract
        context.interpreter.control.set_next_action(
            InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
                input: CallInput::SharedBuffer(input),
                gas_limit,
                target_address: call_targets.target_address,
                caller: call_targets.caller,
                bytecode_address: call_targets.bytecode_address,
                value: CallValue::Transfer(value),
                scheme: CallScheme::CallCode,
                is_static: context.interpreter.runtime_flag.is_static(),
                is_eof: false,
                return_memory_offset,
            }))),
            InstructionResult::CallOrCreate,
        );
    }

    pub fn delegate_call<WIRE: InterpreterTypes, H: Host + ?Sized>(
        context: InstructionContext<'_, H, WIRE>,
    ) {
        check!(context.interpreter, HOMESTEAD);
        popn!([local_gas_limit, to], context.interpreter);
        let to = Address::from_word(B256::from(to));
        // Get the target
        let call_targets = apply_xcall_options::<WIRE, H>(
            InstructionContext {
                host: context.host,
                interpreter: context.interpreter,
            },
            to,
            true,
            false,
        );
        // Max gas limit is not possible in real ethereum situation.
        let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

        let Some((input, return_memory_offset)) =
            get_memory_input_and_out_ranges(context.interpreter)
        else {
            return;
        };

        let Some(mut load) = context
            .host
            .load_account_delegated(call_targets.bytecode_address)
        else {
            context
                .interpreter
                .control
                .set_instruction_result(InstructionResult::FatalExternalError);
            return;
        };

        // Set is_empty to false as we are not creating this account.
        load.is_empty = false;
        let Some(gas_limit) = calc_call_gas(context.interpreter, load, false, local_gas_limit)
        else {
            return;
        };

        gas!(context.interpreter, gas_limit);

        // Call host to interact with target contract
        context.interpreter.control.set_next_action(
            InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
                input: CallInput::SharedBuffer(input),
                gas_limit,
                target_address: call_targets.target_address,
                caller: call_targets.caller,
                bytecode_address: call_targets.bytecode_address,
                value: CallValue::Apparent(context.interpreter.input.call_value()),
                scheme: CallScheme::DelegateCall,
                is_static: context.interpreter.runtime_flag.is_static(),
                is_eof: false,
                return_memory_offset,
            }))),
            InstructionResult::CallOrCreate,
        );
    }

    pub fn static_call<WIRE: InterpreterTypes, H: Host + ?Sized>(
        context: InstructionContext<'_, H, WIRE>,
    ) {
        check!(context.interpreter, BYZANTIUM);
        popn!([local_gas_limit, to], context.interpreter);
        let to = Address::from_word(B256::from(to));
        // Get the target
        let call_targets = apply_xcall_options::<WIRE, H>(
            InstructionContext {
                host: context.host,
                interpreter: context.interpreter,
            },
            to,
            false,
            false,
        );
        // Max gas limit is not possible in real ethereum situation.
        let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

        let Some((input, return_memory_offset)) =
            get_memory_input_and_out_ranges(context.interpreter)
        else {
            return;
        };

        let Some(mut load) = context
            .host
            .load_account_delegated(call_targets.bytecode_address)
        else {
            context
                .interpreter
                .control
                .set_instruction_result(InstructionResult::FatalExternalError);
            return;
        };
        // Set `is_empty` to false as we are not creating this account.
        load.is_empty = false;
        let Some(gas_limit) = calc_call_gas(context.interpreter, load, false, local_gas_limit)
        else {
            return;
        };
        gas!(context.interpreter, gas_limit);

        // Call host to interact with target contract
        context.interpreter.control.set_next_action(
            InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
                input: CallInput::SharedBuffer(input),
                gas_limit,
                target_address: call_targets.target_address,
                caller: call_targets.caller,
                bytecode_address: call_targets.bytecode_address,
                value: CallValue::Transfer(U256::ZERO),
                scheme: CallScheme::StaticCall,
                is_static: true,
                is_eof: false,
                return_memory_offset,
            }))),
            InstructionResult::CallOrCreate,
        );
    }

    #[derive(Debug, Clone)]
    struct CallTargets {
        // The account address of bytecode that is going to be executed.
        //
        // Previously `context.code_address`.
        bytecode_address: Address,
        // Target address, this account storage is going to be modified.
        //
        // Previously `context.address`.
        target_address: Address,
        // This caller is invoking the call.
        //
        // Previously `context.caller`.
        caller: Address,
    }

    /// Collects call options for this CAL
    /// This consumes the data from the PREVIOUS call into the XCALLOPTIONS precompile.
    fn apply_xcall_options<WIRE: InterpreterTypes, H: GwynethContextTr + Host + ?Sized>(
        context: InstructionContext<'_, H, WIRE>,
        mut to: Address,
        delegate: bool,
        code: bool,
    ) -> CallTargets {
        //println!("apply_call_options {:?}", interpreter.call_options);
        let chain_id = context.host.cfg().chain_id();
        let (xcall_options, mut to) = match context.host.chain().xcall_options.take() {
            Some(xcall_options) => {
                println!("apply_call_options {:?}", xcall_options);
                // In delegate call, the caller & target address remains on the same chain
                // Otherwise set to the other chain.
                if delegate {
                    to.set_chain_id(chain_id);
                } else {
                    to.set_chain_id(xcall_options.chain_id);
                };
                (xcall_options, to)
            }
            None => {
                to.set_chain_id(chain_id);
                (
                    XCallOptions {
                        chain_id,
                        sandbox: false,
                        tx_origin: context.host.caller(),
                        msg_sender: context.interpreter.input.target_address(),
                        block_hash: None,
                        proof: Vec::new(),
                    },
                    to,
                )
            }
        };

        let call_targets = CallTargets {
            target_address: if delegate || code {
                xcall_options.msg_sender
            } else {
                to
            },
            caller: if delegate {
                context.interpreter.input.caller_address()
            } else {
                xcall_options.msg_sender
            },
            bytecode_address: if delegate {
                to.set_chain_id(xcall_options.chain_id);
                to
            } else {
                to
            },
        };

        //println!("call targets {:?}", call_targets);
        call_targets
    }
}
