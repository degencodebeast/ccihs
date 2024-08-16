// hook/post_execution.rs

use super::Hook;
use crate::types::{CrossChainMessage, ChainId};
use crate::error::CCIHSResult;
use anchor_lang::solana_program::log::sol_log;

pub struct PostExecutionHook;

impl Hook for PostExecutionHook {
    fn execute(&self, message: &mut CrossChainMessage, source_chain: ChainId, destination_chain: ChainId) -> CCIHSResult<()> {
        // Log the executed message
        sol_log(&format!(
            "Message executed: from {} to {}, nonce: {}, timestamp: {}",
            source_chain, destination_chain, message.nonce, message.timestamp
        ));

        // You could update some on-chain statistics here if needed
        // For example, incrementing a counter for messages received and executed

        // If you're implementing a request-response pattern, you could trigger a response here

        Ok(())
    }
}