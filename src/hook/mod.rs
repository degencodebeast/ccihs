// hook/mod.rs

mod hook_manager;
mod post_dispatch;
mod post_execution;
mod pre_dispatch;
mod pre_execution;

pub use hook_manager::HookManager;
pub use post_dispatch::PostDispatchHook;
pub use post_execution::PostExecutionHook;
pub use pre_dispatch::PreDispatchHook;
pub use pre_execution::PreExecutionHook;

use crate::types::{CrossChainMessage, ChainId};
use crate::error::CCIHSResult;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HookType {
    PreDispatch,
    PostDispatch,
    PreExecution,
    PostExecution,
}

pub trait Hook: Send + Sync {
    fn execute(&self, message: &mut CrossChainMessage, source_chain: ChainId, destination_chain: ChainId) -> CCIHSResult<()>;
}