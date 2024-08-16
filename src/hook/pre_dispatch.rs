// hook/post_dispatch.rs

use super::Hook;
use crate::types::CrossChainMessage;
use crate::error::CCIHSResult;

pub struct PostDispatchHook;

impl Hook for PostDispatchHook {
    fn execute(&self, message: &mut CrossChainMessage) -> CCIHSResult<()> {
        // Implement post-dispatch logic here
        // For example, you might want to log the message or update some statistics
        Ok(())
    }
}