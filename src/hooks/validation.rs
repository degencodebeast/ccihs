use super::Hook;
use crate::types::{CrossChainMessage, ChainId, CCIHSResult};
use crate::error::CCIHSError;

pub struct ValidationHook {
    max_payload_size: usize,
}

impl ValidationHook {
    pub fn new(max_payload_size: usize) -> Self {
        Self { max_payload_size }
    }
}

impl Hook for ValidationHook {
    fn execute(&self, message: &mut CrossChainMessage, source_chain: ChainId, destination_chain: ChainId) -> CCIHSResult<()> {
        if message.payload.len() > self.max_payload_size {
            return Err(CCIHSError::PayloadTooLarge);
        }
        if source_chain == destination_chain {
            return Err(CCIHSError::InvalidChainPair);
        }
        // Add more validation checks as needed
        Ok(())
    }
}

// pub struct MessageValidationHook;

// impl Hook for MessageValidationHook {
//     fn execute(&self, message: &CrossChainMessage, _source_chain: ChainId, _destination_chain: ChainId) -> CCIHSResult<()> {
//         if message.payload.len() > 1000 {
//             return Err(CCIHSError::PayloadTooLarge);
//         }
//         if message.source_chain == message.destination_chain {
//             return Err(CCIHSError::InvalidChainPair);
//         }
//         Ok(())
//     }
// }