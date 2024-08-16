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

    pub fn execute_hooks(&self, hook_type: HookType, message: &mut CrossChainMessage, source_chain: ChainId, destination_chain: ChainId) -> CCIHSResult<()> {
        let hooks = self.hooks.get(&hook_type).ok_or(CCIHSError::HookNotFound)?;
        for hook in hooks {
            hook.execute(message, source_chain, destination_chain)?;
        }
        Ok(())
    }

    pub fn remove_hook(&mut self, hook_type: HookType, index: usize) -> CCIHSResult<()> {
        let hooks = self.hooks.get_mut(&hook_type).ok_or(CCIHSError::HookNotFound)?;
        if index < hooks.len() {
            hooks.remove(index);
            Ok(())
        } else {
            Err(CCIHSError::HookIndexOutOfBounds)
        }
    }

    pub fn clear_hooks(&mut self, hook_type: HookType) {
        if let Some(hooks) = self.hooks.get_mut(&hook_type) {
            hooks.clear();
        }
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}


// This implementation provides a flexible and robust hook system for your CCIHS project. Here are some key features:

// Each hook type (PreDispatch, PostDispatch, PreExecution, PostExecution) has its own implementation with specific checks and actions.
// The HookManager allows adding, removing, and clearing hooks, providing flexibility in managing the hook system.
// Hooks receive not just the message, but also the source and destination chain IDs, allowing for chain-specific logic.
// Error handling is implemented throughout, using custom error types from your CCIHSError enum.
// The system is designed to be easily extensible - you can add new hook types or implement additional hooks as your project grows.

// To use this in your WormholeAdapter, you would typically:

// impl WormholeAdapter {
//     pub fn send_message<'info>(
//         &self,
//         ctx: Context<'_, '_, '_, 'info, SendMessage<'info>>,
//         message: &mut CrossChainMessage,
//     ) -> Result<()> {
//         self.hook_manager.execute_hooks(HookType::PreDispatch, message, message.source_chain, message.destination_chain)?;
        
//         // ... perform send operation ...

//         self.hook_manager.execute_hooks(HookType::PostDispatch, message, message.source_chain, message.destination_chain)?;

//         Ok(())
//     }

//     pub fn receive_message<'info>(
//         &self,
//         ctx: Context<'_, '_, '_, 'info, ReceiveMessage<'info>>,
//     ) -> Result<CrossChainMessage> {
//         // ... deserialize message ...

//         let source_chain = ChainId::SOLANA; // Or determine this dynamically
//         let destination_chain = ChainId::ETHEREUM; // Or determine this dynamically

//         self.hook_manager.execute_hooks(HookType::PreExecution, &mut message, source_chain, destination_chain)?;
        
//         // ... perform receive operation ...

//         self.hook_manager.execute_hooks(HookType::PostExecution, &mut message, source_chain, destination_chain)?;

//         Ok(message)
//     }
// }

// This implementation provides a solid foundation for your hook system in CCIHS. It allows for flexible message processing, chain-specific logic, and easy extensibility. You can expand on this by adding more specific hooks or by implementing more complex logic within each hook as your project requirements evolve.