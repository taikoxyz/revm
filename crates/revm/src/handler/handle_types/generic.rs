use crate::{
    primitives::{db::SyncDatabase as Database, EVMResultGeneric},
    Context,
};
use std::sync::Arc;

/// Generic Handle that takes a mutable reference to the context and returns a result.
pub type GenericContextHandle<'a, EXT, DB> = GenericContextHandleRet<'a, EXT, DB, ()>;

/// Generic handle that takes a mutable reference to the context and returns a result.
pub type GenericContextHandleRet<'a, EXT, DB, ReturnT> =
    Arc<dyn Fn(&mut Context<EXT, DB>) -> EVMResultGeneric<ReturnT, <DB as Database>::Error> + 'a>;

/// Generic Handle that takes a mutable reference to the context and returns a result.
pub type GenericContextHandleChain<'a, EXT, DB> = GenericContextHandleRetChain<'a, EXT, DB, ()>;

/// Generic handle that takes a mutable reference to the context and returns a result.
pub type GenericContextHandleRetChain<'a, EXT, DB, ReturnT> =
    Arc<dyn Fn(&mut Context<EXT, DB>, u64) -> EVMResultGeneric<ReturnT, <DB as Database>::Error> + 'a>;
