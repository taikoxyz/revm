use crate::{SyncDatabase, Inspector};
/// Dummy [Inspector], helpful as standalone replacement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoOpInspector;

impl<DB: SyncDatabase> Inspector<DB> for NoOpInspector {}
