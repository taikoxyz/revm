use crate::cfg::CfgExt;
use revm::context::ContextTr;

// Type alias for Gwyneth context
pub trait GwynethContextTr: ContextTr<Cfg: CfgExt, Chain = GwynethContext> {}

/// Gwyneth special context
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GwynethContext {
    /// xcall options setting by the precompile
    pub xcall_options: Option<crate::xcall::XCallOptions>,
}
