// hook/post_dispatch.rs

use super::Hook;
use crate::types::{CrossChainMessage, ChainId};
use crate::error::CCIHSResult;
use anchor_lang::solana_program::log::sol_log;

pub struct PostDispatchHook;

impl Hook for PostDispatchHook {
    fn execute(&self, message: &mut CrossChainMessage, source_chain: ChainId, destination_chain: ChainId) -> CCIHSResult<()> {
        // Log the dispatched message
        sol_log(&format!(
            "Message dispatched: from {} to {}, nonce: {}, timestamp: {}",
            source_chain, destination_chain, message.nonce, message.timestamp
        ));

        // You could update some on-chain statistics here if needed
        // For example, incrementing a counter for messages sent

        Ok(())
    }
}