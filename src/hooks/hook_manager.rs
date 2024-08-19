// hook/hook_manager.rs

use super::{Hook, HookType, PreDispatchHook, PostDispatchHook, PreExecutionHook, PostExecutionHook};
use crate::types::{CrossChainMessage, ChainId};
use crate::error::{CCIHSResult, CCIHSError};
use std::collections::HashMap;

pub struct HookManager {
    hooks: HashMap<HookType, Vec<Box<dyn Hook>>>,
}

impl HookManager {
    pub fn new() -> Self {

        Self {
            hooks: HashMap::new(),
        }
    }

    pub fn add_hook(&mut self, hook_type: HookType, hook: Box<dyn Hook>) {
        self.hooks.entry(hook_type).or_default().push(hook);
    }

    // pub fn add_hook(&mut self, hook_type: HookType, hook: Box<dyn Hook>) {
    //     self.hooks.entry(hook_type).or_insert_with(Vec::new).push(hook);
    // }

    pub fn remove_hook(&mut self, hook_type: HookType, index: usize) -> CCIHSResult<()> {
        if let Some(hooks) = self.hooks.get_mut(&hook_type) {
            if index < hooks.len() {
                hooks.remove(index);
                Ok(())
            } else {
                Err(CCIHSError::HookIndexOutOfBounds)
            }
        } else {
            Err(CCIHSError::HookTypeNotFound)
        }
    }

    // pub fn remove_hook(&mut self, hook_type: HookType, index: usize) -> CCIHSResult<()> {
    //     if let Some(hooks) = self.hooks.get_mut(&hook_type) {
    //         if index < hooks.len() {
    //             hooks.remove(index);
    //             Ok(())
    //         } else {
    //             Err(CCIHSError::HookIndexOutOfBounds)
    //         }
    //     } else {
    //         Err(CCIHSError::HookTypeNotFound)
    //     }
    // }

    pub fn clear_hooks(&mut self, hook_type: HookType) {
        if let Some(hooks) = self.hooks.get_mut(&hook_type) {
            hooks.clear();
        }
    }

    pub fn execute_hooks(&self, hook_type: HookType, message: &mut CrossChainMessage, source_chain: ChainId, destination_chain: ChainId) -> CCIHSResult<()> {
        // Execute default behavior first
        hook_type.execute_default(message, source_chain, destination_chain)?;

        // Then execute custom hooks
        if let Some(hooks) = self.hooks.get(&hook_type) {
            for hook in hooks {
                hook.execute(message, source_chain, destination_chain)?;
            }
        }

        Ok(())
    }

    // pub async fn execute_hooks(&self, hook_type: HookType, message: &mut CrossChainMessage) -> CCIHSResult<()> {
    //     if let Some(hooks) = self.hooks.get(&hook_type) {
    //         for hook in hooks {
    //             hook.execute(message).await?;
    //         }
    //     }
    //     Ok(())
    // }

}



// When setting up hooks, specify the stage at which each hook should be executed:

// let mut hook_manager = HookManager::new();
// hook_manager.add_hook(HookType::PreDispatch, Box::new(RateLimitingHook::new(100, Duration::from_secs(60))));
// hook_manager.add_hook(HookType::PreDispatch, Box::new(ValidationHook::new(1024)));
// hook_manager.add_hook(HookType::PostDispatch, Box::new(LoggingHook));
// hook_manager.add_hook(HookType::PreExecution, Box::new(EncryptionHook::new([0u8; 32])));
// hook_manager.add_hook(HookType::PostExecution, Box::new(MetricsHook::new()))