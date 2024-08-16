// hook/hook_manager.rs

use super::{Hook, HookType, PreDispatchHook, PostDispatchHook, PreExecutionHook, PostExecutionHook};
use crate::types::CrossChainMessage;
use crate::error::CCIHSResult;
use std::collections::HashMap;

pub struct HookManager {
    hooks: HashMap<HookType, Vec<Box<dyn Hook>>>,
}

impl HookManager {
    pub fn new() -> Self {
        let mut hooks = HashMap::new();
        hooks.insert(HookType::PreDispatch, vec![Box::new(PreDispatchHook)]);
        hooks.insert(HookType::PostDispatch, vec![Box::new(PostDispatchHook)]);
        hooks.insert(HookType::PreExecution, vec![Box::new(PreExecutionHook)]);
        hooks.insert(HookType::PostExecution, vec![Box::new(PostExecutionHook)]);
        Self { hooks }
    }

    pub fn add_hook(&mut self, hook_type: HookType, hook: Box<dyn Hook>) {
        self.hooks.entry(hook_type).or_default().push(hook);
    }

    pub fn execute_hooks(&self, hook_type: HookType, message: &mut CrossChainMessage) -> CCIHSResult<()> {
        if let Some(hooks) = self.hooks.get(&hook_type) {
            for hook in hooks {
                hook.execute(message)?;
            }
        }
        Ok(())
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}


// This implementation provides a flexible hook system that allows you to:

// Define different types of hooks (PreDispatch, PostDispatch, PreExecution, PostExecution).
// Easily add new hooks to the system.
// Execute all hooks of a specific type when needed.

// The HookManager is initialized with default hooks for each type, but you can add more hooks as needed. Each hook type has its own implementation, allowing you to customize the behavior for different stages of the cross-chain message processing.
// To use this in your WormholeAdapter, you would typically:

// Create a HookManager when initializing the WormholeAdapter.
// Call execute_hooks at the appropriate points in your send_message and receive_message functions.

// For example:

// impl WormholeAdapter {
//     pub fn send_message<'info>(
//         &self,
//         ctx: Context<'_, '_, '_, 'info, SendMessage<'info>>,
//         message: &mut CrossChainMessage,
//     ) -> Result<()> {
//         self.hook_manager.execute_hooks(HookType::PreDispatch, message)?;
        
//         // ... perform send operation ...

//         self.hook_manager.execute_hooks(HookType::PostDispatch, message)?;

//         Ok(())
//     }

//     pub fn receive_message<'info>(
//         &self,
//         ctx: Context<'_, '_, '_, 'info, ReceiveMessage<'info>>,
//     ) -> Result<CrossChainMessage> {
//         // ... deserialize message ...

//         self.hook_manager.execute_hooks(HookType::PreExecution, &mut message)?;
        
//         // ... perform receive operation ...

//         self.hook_manager.execute_hooks(HookType::PostExecution, &mut message)?;

//         Ok(message)
//     }
// }
// This implementation provides a solid foundation for your hook system. You can expand on this by adding more specific hooks or by implementing more complex logic within each hook as your project requirements evolve.