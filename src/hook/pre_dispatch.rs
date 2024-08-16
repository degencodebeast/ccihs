// hook/pre_dispatch.rs

use super::Hook;
use crate::types::{CrossChainMessage, ChainId};
use crate::error::{CCIHSResult, CCIHSError};

pub struct PreDispatchHook;

impl Hook for PreDispatchHook {
    fn execute(&self, message: &mut CrossChainMessage, source_chain: ChainId, destination_chain: ChainId) -> CCIHSResult<()> {
        // Validate the message
        if message.payload.is_empty() {
            return Err(CCIHSError::InvalidPayload);
        }

        // Check if the destination chain is supported
        if !is_supported_chain(destination_chain) {
            return Err(CCIHSError::UnsupportedChain(destination_chain));
        }

        // Add a timestamp to the message
        message.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| CCIHSError::TimestampError)?
            .as_secs();

        Ok(())
    }
}

fn is_supported_chain(chain_id: ChainId) -> bool {
    // Implement your chain support logic here
    // For example:
    matches!(chain_id, ChainId::SOLANA | ChainId::ETHEREUM)
}