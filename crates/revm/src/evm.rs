use revm_interpreter::{CallOutcome, Gas, InstructionResult, InterpreterResult};

use crate::{
    builder::{EvmBuilder, HandlerStage, SetGenericStage},
    db::{SyncDatabase as Database, DatabaseCommit, EmptyDB},
    handler::Handler,
    interpreter::{
        CallInputs, CreateInputs, EOFCreateInputs, Host, InterpreterAction, SharedMemory,
    },
    primitives::{
        specification::SpecId, BlockEnv, Bytes, CfgEnv, ChainAddress, EVMError, EVMResult, EnvWithHandlerCfg,
        ExecutionResult, HandlerCfg, ResultAndState, TransactTo, TxEnv, TxKind, EOF_MAGIC_BYTES, XCallData, XCallInput, XCallOutput, JournalEntry, hex, Bytecode,
    },
    Context, ContextWithHandlerCfg, Frame, FrameOrResult, FrameResult,
};
use core::{fmt, future::pending};
use std::{boxed::Box, collections::HashMap, vec::Vec};
use crate::primitives::address;

/// EVM call stack limit.
pub const CALL_STACK_LIMIT: u64 = 1024;

/// EVM instance containing both internal EVM context and external context
/// and the handler that dictates the logic of EVM (or hardfork specification).
pub struct Evm<'a, EXT, DB: Database> {
    /// Context of execution, containing both EVM and external context.
    pub context: Context<EXT, DB>,
    /// Handler is a component of the of EVM that contains all the logic. Handler contains specification id
    /// and it different depending on the specified fork.
    pub handler: Handler<'a, Context<EXT, DB>, EXT, DB>,
}

impl<EXT, DB> fmt::Debug for Evm<'_, EXT, DB>
where
    EXT: fmt::Debug,
    DB: Database + fmt::Debug,
    DB::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Evm")
            .field("evm context", &self.context.evm)
            .finish_non_exhaustive()
    }
}

impl<EXT, DB: Database + DatabaseCommit> Evm<'_, EXT, DB> {
    /// Commit the changes to the database.
    pub fn transact_commit(&mut self) -> Result<ExecutionResult, EVMError<DB::Error>> {
        let ResultAndState { result, state } = self.transact()?;
        self.context.evm.db.commit(state);
        Ok(result)
    }
}

impl<'a> Evm<'a, (), EmptyDB> {
    /// Returns evm builder with empty database and empty external context.
    pub fn builder() -> EvmBuilder<'a, SetGenericStage, (), EmptyDB> {
        EvmBuilder::default()
    }
}

impl<'a, EXT, DB: Database> Evm<'a, EXT, DB> {
    /// Create new EVM.
    pub fn new(
        mut context: Context<EXT, DB>,
        handler: Handler<'a, Context<EXT, DB>, EXT, DB>,
    ) -> Evm<'a, EXT, DB> {
        context.evm.journaled_state.set_spec_id(handler.cfg.spec_id);
        Evm { context, handler }
    }

    /// Allow for evm setting to be modified by feeding current evm
    /// into the builder for modifications.
    pub fn modify(self) -> EvmBuilder<'a, HandlerStage, EXT, DB> {
        EvmBuilder::new(self)
    }

    /// Runs main call loop.
    #[inline]
    pub fn run_the_loop(&mut self, first_frame: Frame) -> Result<FrameResult, EVMError<DB::Error>> {
        //println!("EVM:run_the_loop: exaust the frame stack");
        let mut call_stack: Vec<Frame> = Vec::with_capacity(1025);
        call_stack.push(first_frame);

        let tx = self.context.evm.env().tx.clone();

        // TODO(Brecht)
        if !self.cfg().xchain && tx.caller.1 != address!("E25583099BA105D9ec0A67f5Ae86D90e50036425") && tx.transact_to.is_call() && tx.transact_to.to().cloned().unwrap_or_default().1 == address!("9fCF7D13d10dEdF17d0f24C62f0cf4ED462f65b7") {
            return Ok(
                FrameResult::Call(CallOutcome {
                    result: InterpreterResult::new(InstructionResult::Return, Bytes::new(), Gas::new(0)),
                    call_options: None,
                    memory_offset: 0..0,
                })
            );
        }

        if !self.cfg().xchain && tx.caller.1 == address!("E25583099BA105D9ec0A67f5Ae86D90e50036425") {
            for item in tx.access_list.iter() {
                // yolo account abstraction
                let account_abstraction_code = hex!("60406080815260048036101561001e575b5050361561001c575f80fd5b005b5f3560e01c63a6d0ad610361001057602091826003193601126102225781359167ffffffffffffffff918284116102225736602385011215610222578382013593602493808611610238578560051b858861007a81840161024a565b8099815201918401019236841161022257868101915b8483106101795750505050505f5b845181101561001c57858160051b860101518681019060018060a01b0380835116925f808b88860196875187519283519301915af13d15610171573d906100ec6100e783610284565b61024a565b9182523d5f8d84013e5b1561014c5750917fcaf938de11c367272220bfd1d2baa99ca46665e7bc4d85f00adb51b90fe1fa9f9160019594935116925190519061014387519283928352888d840152888301906102a0565b0390a20161009e565b865162461bcd60e51b81528089018c905290819061016d90828c01906102a0565b0390fd5b6060906100f6565b823584811161022257820160609182602319833603011261022257875192830183811087821117610226578852898201358681116102225782013660438201121561022257604490808c01356101d16100e782610284565b9181835236848383010111610222578f909180855f940183860137830101528452820135926001600160a01b03841684036102225760648d9493859485840152013589820152815201920191610090565b5f80fd5b8a60418b634e487b7160e01b5f52525ffd5b84604185634e487b7160e01b5f52525ffd5b6040519190601f01601f1916820167ffffffffffffffff81118382101761027057604052565b634e487b7160e01b5f52604160045260245ffd5b67ffffffffffffffff811161027057601f01601f191660200190565b91908251928382525f5b8481106102ca575050825f602080949584010152601f8019910116010190565b6020818301810151848301820152016102aa56fea2646970667358221220838cbd59498426defca87a853e63f860529c6ac35b1d64c51b8db89a8b39827e64736f6c63430008180033");
                let bytecode = Bytecode::new_legacy(Bytes::from(account_abstraction_code));
                self.context.evm.journaled_state.set_code(ChainAddress(tx.caller.0, item.address), bytecode);
            }
        }

        self.context.evm.journaled_state.state_changes.push(JournalEntry::TxBegin { tx });

        #[cfg(feature = "memory_limit")]
        let mut shared_memory =
            SharedMemory::new_with_memory_limit(self.context.evm.env.cfg.memory_limit);
        #[cfg(not(feature = "memory_limit"))]
        let mut shared_memory = SharedMemory::new();

        shared_memory.new_context();

        // Peek the last stack frame.
        let mut stack_frame = call_stack.last_mut().unwrap();
        let mut call_options = None;
        let mut cnt = 0;

        let mut pending_xcalls: HashMap<u64, usize> = HashMap::new();
        let mut xcall_idx = 0usize;
        loop {
            //println!("loop: {}", cnt);
            cnt += 1;

            // TODO(Brecht): Potential issue when revert happens after setting the options?
            stack_frame.interpreter_mut().call_options = std::mem::take(&mut call_options);

            // The start of a smart contract execution after all initial checks have passed
            if stack_frame.is_call() {
                let input = stack_frame.interpreter().contract.input.clone();
                // println!("-> contract: {:?}, input: {}", stack_frame.interpreter().contract.target_address, input);
                // TODO: Do something
            }

            // Execute the frame.
            let next_action =
                self.handler
                    .execute_frame(stack_frame, &mut shared_memory, &mut self.context)?;

            // Take error and break the loop, if any.
            // This error can be set in the Interpreter when it interacts with the context.
            self.context.evm.take_error()?;

            let exec = &mut self.handler.execution;
            let frame_or_result = match next_action.clone() {
                InterpreterAction::Call { inputs } => {
                    //println!(">>> Call: {:?}", inputs);
                    // We have to record xcalls, except those done to the parent chain (those will still execute locally)
                    let is_xcall =  inputs.target_address.0 != stack_frame.frame_data().interpreter.chain_id;
                    //println!("chain {} -> {} (is xcall: {})", stack_frame.frame_data().interpreter.chain_id, inputs.target_address.0, is_xcall);

                    // if is_xcall && self.context.evm.env.tx.xcalls.is_some() {
                    //     let xcalls = self.context.evm.env.tx.xcalls.as_ref().unwrap();
                    //     let gas = Gas::new(inputs.gas_limit);
                    //     let return_result = |instruction_result: InstructionResult| {
                    //         FrameOrResult::new_call_result(
                    //             InterpreterResult {
                    //                 result: instruction_result,
                    //                 gas,
                    //                 output: Bytes::new(),
                    //                 call_options: None,
                    //             },
                    //             inputs.return_memory_offset.clone(),
                    //         )
                    //     };
                    //     // Check depth
                    //     if self.context.evm.journaled_state.depth() > CALL_STACK_LIMIT {
                    //         return_result(InstructionResult::CallTooDeep)
                    //     } else {
                    //         let xcall = &xcalls[xcall_idx];
                    //         xcall_idx += 1;
                    //         FrameOrResult::new_call_result(
                    //             InterpreterResult {
                    //                 result: InstructionResult::Return,
                    //                 gas: Gas::new(xcall.gas),
                    //                 output: Bytes::from(xcall.output.clone()),
                    //                 call_options: None,
                    //             },
                    //             inputs.return_memory_offset.clone(),
                    //         )
                    //     }
                    // } else {
                        // Insert the xcall before the call is made, because the call might do more calls
                        let pending_xcall_idx = self.context.evm.journaled_state.xcall(
                            inputs.caller.0,
                            inputs.target_address.0,
                            XCallData {
                            input: XCallInput {
                                input: inputs.clone().input,
                                gas_limit: inputs.clone().gas_limit,
                                bytecode_address: inputs.clone().bytecode_address,
                                target_address: inputs.clone().target_address,
                                caller: inputs.clone().caller,
                                is_static: inputs.clone().is_static,
                                is_eof: inputs.clone().is_eof,
                                value: inputs.call_value(),
                            },
                            output: XCallOutput {
                                revert: true,
                                output: Bytes::new(),
                                gas: 0,
                            }
                        });

                        let res = exec.call(&mut self.context, inputs.clone())?;
                        match &res {
                            FrameOrResult::Frame(_) => {
                                //if is_xcall {
                                    //println!("pending xcall at: {}", call_stack.len());
                                    pending_xcalls.insert(call_stack.len() as u64, pending_xcall_idx);
                                //}
                            }
                            FrameOrResult::Result(FrameResult::Call(outcome)) => {
                                //println!("call done: {:?}", outcome);
                                let depth = self.context.evm.journaled_state.depth();
                                //if let Some(&xcall_idx) = pending_xcalls.get(&depth) {
                                    //println!("xcall: {:?}", xcall_idx);
                                    // return_call
                                    let xcall: &mut JournalEntry = &mut self.context.evm.journaled_state.state_changes[pending_xcall_idx];
                                    if let JournalEntry::CallBegin { depth, from_chain_id, to_chain_id, data, call } = xcall {
                                        data.output.output = outcome.result.output.clone();
                                        data.output.gas = outcome.result.gas.limit() - outcome.result.gas.remaining();
                                        data.output.revert = outcome.result.is_revert();
                                    }

                                    self.context.evm.journaled_state.state_changes.push(JournalEntry::CallEnd { depth: depth as usize });
                                // } else {
                                //     //println!("outcome: {:?}", outcome);
                                //     println!("uncatched outcome A");
                                // }
                            }
                            _ => {
                                println!("uncatched frame result");
                            }
                        };
                        res
                    //}
                }
                InterpreterAction::Create { inputs } => exec.create(&mut self.context, inputs)?,
                InterpreterAction::EOFCreate { inputs } => {
                    exec.eofcreate(&mut self.context, inputs)?
                }
                InterpreterAction::Return { result } => {
                    // free memory context.
                    shared_memory.free_context();

                    // pop last frame from the stack and consume it to create FrameResult.
                    let returned_frame = call_stack
                        .pop()
                        .expect("We just returned from Interpreter frame");

                    let ctx = &mut self.context;
                    FrameOrResult::Result(match returned_frame {
                        Frame::Call(frame) => {
                            // return_call
                            FrameResult::Call(exec.call_return(ctx, frame, result)?)
                        }
                        Frame::Create(frame) => {
                            // return_create
                            FrameResult::Create(exec.create_return(ctx, frame, result)?)
                        }
                        Frame::EOFCreate(frame) => {
                            // return_eofcreate
                            FrameResult::EOFCreate(exec.eofcreate_return(ctx, frame, result)?)
                        }
                    })
                }
                InterpreterAction::None => unreachable!("InterpreterAction::None is not expected"),
            };

            // println!("  loop ==> frame_or_result: {:?}", match frame_or_result {
            //     FrameOrResult::Frame(_) => "Frame",
            //     FrameOrResult::Result(_) => "Result",
            // });

            // handle result
            match frame_or_result {
                FrameOrResult::Frame(frame) => {
                    shared_memory.new_context();
                    call_stack.push(frame);
                    stack_frame = call_stack.last_mut().unwrap();
                }
                FrameOrResult::Result(result) => {
                    let depth = call_stack.len();
                    let Some(top_frame) = call_stack.last_mut() else {
                        // Break the loop if there are no more frames.
                        return Ok(result);
                    };
                    stack_frame = top_frame;
                    let ctx = &mut self.context;
                    // Insert result to the top frame.
                    match result {
                        FrameResult::Call(outcome) => {
                            // return_call
                            call_options = outcome.call_options.clone();

                            //println!("call done [{}] with return: {:?}", depth, outcome);
                            //println!("pending xcalls: {:?}", pending_xcalls);
                            let depth = ctx.evm.journaled_state.depth();
                            if let Some(&xcall_idx) = pending_xcalls.get(&depth) {
                                //println!("outcome catched: {:?}", outcome);
                                //println!("xcall: {:?}", xcall_idx);
                                let xcall = &mut ctx.evm.journaled_state.state_changes[xcall_idx];
                                if let JournalEntry::CallBegin { depth, from_chain_id, to_chain_id, data, call } = xcall {
                                    data.output.output = outcome.result.output.clone();
                                    data.output.gas = outcome.result.gas.limit() - outcome.result.gas.remaining();
                                    data.output.revert = outcome.result.is_revert();
                                }

                                ctx.evm.journaled_state.state_changes.push(JournalEntry::CallEnd { depth: depth as usize });
                            } else {
                                //println!("outcome uncatched 2: {:?}", outcome);
                            }

                            exec.insert_call_outcome(ctx, stack_frame, &mut shared_memory, outcome)?
                        }
                        FrameResult::Create(outcome) => {
                            // return_create
                            exec.insert_create_outcome(ctx, stack_frame, outcome)?
                        }
                        FrameResult::EOFCreate(outcome) => {
                            // return_eofcreate
                            exec.insert_eofcreate_outcome(ctx, stack_frame, outcome)?
                        }
                    }
                }
            }
        }
    }
}

impl<EXT, DB: Database> Evm<'_, EXT, DB> {
    /// Returns specification (hardfork) that the EVM is instanced with.
    ///
    /// SpecId depends on the handler.
    pub fn spec_id(&self) -> SpecId {
        self.handler.cfg.spec_id
    }

    /// Pre verify transaction by checking Environment, initial gas spend and if caller
    /// has enough balance to pay for the gas.
    #[inline]
    pub fn preverify_transaction(&mut self) -> Result<(), EVMError<DB::Error>> {
        let output = self.preverify_transaction_inner().map(|_| ());
        self.clear();
        output
    }

    /// Calls clear handle of post execution to clear the state for next execution.
    fn clear(&mut self) {
        self.handler.post_execution().clear(&mut self.context);
    }

    /// Transact pre-verified transaction
    ///
    /// This function will not validate the transaction.
    #[inline]
    pub fn transact_preverified(&mut self) -> EVMResult<DB::Error> {
        let initial_gas_spend = self
            .handler
            .validation()
            .initial_tx_gas(&self.context.evm.env)
            .map_err(|e| {
                self.clear();
                e
            })?;
        let output = self.transact_preverified_inner(initial_gas_spend);
        let output = self.handler.post_execution().end(&mut self.context, output);
        self.clear();
        output
    }

    /// Pre verify transaction inner.
    #[inline]
    fn preverify_transaction_inner(&mut self) -> Result<u64, EVMError<DB::Error>> {
        self.handler.validation().env(&self.context.evm.env)?;
        let initial_gas_spend = self
            .handler
            .validation()
            .initial_tx_gas(&self.context.evm.env)?;
        self.handler
            .validation()
            .tx_against_state(&mut self.context)?;
        Ok(initial_gas_spend)
    }

    /// Transact transaction
    ///
    /// This function will validate the transaction.
    #[inline]
    pub fn transact(&mut self) -> EVMResult<DB::Error> {
        //println!("transact");
        let initial_gas_spend = self.preverify_transaction_inner().map_err(|e| {
            self.clear();
            e
        })?;

        let output = self.transact_preverified_inner(initial_gas_spend);
        let output = self.handler.post_execution().end(&mut self.context, output);
        self.clear();
        output
    }

    /// Returns the reference of handler configuration
    #[inline]
    pub fn handler_cfg(&self) -> &HandlerCfg {
        &self.handler.cfg
    }

    /// Returns the reference of Env configuration
    #[inline]
    pub fn cfg(&self) -> &CfgEnv {
        &self.context.env().cfg
    }

    /// Returns the mutable reference of Env configuration
    #[inline]
    pub fn cfg_mut(&mut self) -> &mut CfgEnv {
        &mut self.context.evm.env.cfg
    }

    /// Returns the reference of transaction
    #[inline]
    pub fn tx(&self) -> &TxEnv {
        &self.context.evm.env.tx
    }

    /// Returns the mutable reference of transaction
    #[inline]
    pub fn tx_mut(&mut self) -> &mut TxEnv {
        &mut self.context.evm.env.tx
    }

    /// Returns the reference of database
    #[inline]
    pub fn db(&self) -> &DB {
        &self.context.evm.db
    }

    /// Returns the mutable reference of database
    #[inline]
    pub fn db_mut(&mut self) -> &mut DB {
        &mut self.context.evm.db
    }

    /// Returns the reference of block
    #[inline]
    pub fn block(&self) -> &BlockEnv {
        &self.context.evm.env.block
    }

    /// Returns the mutable reference of block
    #[inline]
    pub fn block_mut(&mut self) -> &mut BlockEnv {
        &mut self.context.evm.env.block
    }

    /// Modify spec id, this will create new EVM that matches this spec id.
    pub fn modify_spec_id(&mut self, spec_id: SpecId) {
        self.handler.modify_spec_id(spec_id);
    }

    /// Returns internal database and external struct.
    #[inline]
    pub fn into_context(self) -> Context<EXT, DB> {
        self.context
    }

    /// Returns database and [`EnvWithHandlerCfg`].
    #[inline]
    pub fn into_db_and_env_with_handler_cfg(self) -> (DB, EnvWithHandlerCfg) {
        (
            self.context.evm.inner.db,
            EnvWithHandlerCfg {
                env: self.context.evm.inner.env,
                handler_cfg: self.handler.cfg,
            },
        )
    }

    /// Returns [Context] and [HandlerCfg].
    #[inline]
    pub fn into_context_with_handler_cfg(self) -> ContextWithHandlerCfg<EXT, DB> {
        ContextWithHandlerCfg::new(self.context, self.handler.cfg)
    }

    /// Transact pre-verified transaction.
    fn transact_preverified_inner(&mut self, initial_gas_spend: u64) -> EVMResult<DB::Error> {
        //println!("EVM:transact_preverified_inner");
        let spec_id = self.spec_id();
        let ctx = &mut self.context;
        let pre_exec = self.handler.pre_execution();

        // load access list and beneficiary if needed.
        pre_exec.load_accounts(ctx, ctx.evm.env.tx.caller.0)?;

        // load precompiles
        for chain_id in ctx.evm.env.tx.chain_ids.clone().unwrap_or(vec![ctx.evm.env.tx.caller.0]).into_iter() {
            // TODO(Brecht): precompiles need to be aware of chain_id
            let precompiles = pre_exec.load_precompiles();
            ctx.evm.set_precompiles(chain_id, precompiles);
        }

        // deduce caller balance with its limit.
        pre_exec.deduct_caller(ctx)?;

        let gas_limit = ctx.evm.env.tx.gas_limit - initial_gas_spend;

        // apply EIP-7702 auth list.
        let eip7702_gas_refund = pre_exec.apply_eip7702_auth_list(ctx)? as i64;

        let exec = self.handler.execution();
        // call inner handling of call/create

        //println!("first_frame_or_result from transact_to {:?}", ctx.evm.env.tx.transact_to);
        let first_frame_or_result = match ctx.evm.env.tx.transact_to {
            TransactTo::Call(_) => exec.call(
                ctx,
                CallInputs::new_boxed(&ctx.evm.env.tx, gas_limit).unwrap(),
            )?,
            TransactTo::Create => {
                // if first byte of data is magic 0xEF00, then it is EOFCreate.
                if spec_id.is_enabled_in(SpecId::PRAGUE_EOF)
                    && ctx.env().tx.data.starts_with(&EOF_MAGIC_BYTES)
                {
                    exec.eofcreate(
                        ctx,
                        Box::new(EOFCreateInputs::new_tx(&ctx.evm.env.tx, gas_limit)),
                    )?
                } else {
                    // Safe to unwrap because we are sure that it is create tx.
                    exec.create(
                        ctx,
                        CreateInputs::new_boxed(&ctx.evm.env.tx, gas_limit).unwrap(),
                    )?
                }
            }
        };

        // Starts the main running loop.
        let mut result = match first_frame_or_result {
            FrameOrResult::Frame(first_frame) => self.run_the_loop(first_frame)?,
            FrameOrResult::Result(result) => result,
        };

        let ctx = &mut self.context;

        // handle output of call/create calls.
        self.handler
            .execution()
            .last_frame_return(ctx, &mut result)?;

        let post_exec = self.handler.post_execution();
        // calculate final refund and add EIP-7702 refund to gas.
        post_exec.refund(ctx, result.gas_mut(), eip7702_gas_refund);
        // Reimburse the caller
        post_exec.reimburse_caller(ctx, result.gas())?;
        // Reward beneficiary
        post_exec.reward_beneficiary(ctx, result.gas())?;
        // Returns output of transaction.
        post_exec.output(ctx, result)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        db::BenchmarkDB,
        interpreter::opcode::{PUSH1, SSTORE},
        primitives::{address, Authorization, Bytecode, RecoveredAuthorization, Signature, U256},
    };

    #[test]
    fn sanity_eip7702_tx() {
        let chain_id = 1;
        let delegate = address!("0000000000000000000000000000000000000000");
        let caller = address!("0000000000000000000000000000000000000001");
        let auth = address!("0000000000000000000000000000000000000100");

        let bytecode = Bytecode::new_legacy([PUSH1, 0x01, PUSH1, 0x01, SSTORE].into());

        let mut evm = Evm::builder()
            .with_spec_id(SpecId::PRAGUE)
            .with_db(BenchmarkDB::new_bytecode(bytecode))
            .modify_tx_env(|tx| {
                tx.authorization_list = Some(
                    vec![RecoveredAuthorization::new_unchecked(
                        Authorization {
                            chain_id: U256::from(1),
                            address: delegate,
                            nonce: 0,
                        }
                        .into_signed(Signature::test_signature()),
                        Some(auth),
                    )]
                    .into(),
                );
                tx.caller = ChainAddress(chain_id, caller);
                tx.transact_to = TransactTo::Call(ChainAddress(chain_id, auth));
                tx.chain_ids = Some(vec![chain_id]);
            })
            .build();

        let ok = evm.transact().unwrap();

        let auth_acc = ok.state.get(&ChainAddress(chain_id, auth)).unwrap();
        assert_eq!(auth_acc.info.code, Some(Bytecode::new_eip7702(delegate)));
        assert_eq!(auth_acc.info.nonce, 1);
        assert_eq!(
            auth_acc.storage.get(&U256::from(1)).unwrap().present_value,
            U256::from(1)
        );
    }
}
