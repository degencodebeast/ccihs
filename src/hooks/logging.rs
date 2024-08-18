use super::Hook;
use crate::types::{CrossChainMessage, ChainId, CCIHSResult};
use log;

pub struct LoggingHook;

impl Hook for LoggingHook {
    fn execute(&self, message: &mut CrossChainMessage, source_chain: ChainId, destination_chain: ChainId) -> CCIHSResult<()> {
        log::info!(
            "Processing message: source: {:?}, destination: {:?}, nonce: {}, payload size: {}",
            source_chain,
            destination_chain,
            message.nonce,
            message.payload.len()
        );
        Ok(())
    }
}


// use super::Hook;
// use crate::types::{CrossChainMessage, ChainId, CCIHSResult};
// use async_trait::async_trait;
// pub struct LoggingHook;

// impl Hook for LoggingHook {
//     fn execute(&self, message: &mut CrossChainMessage, source_chain: ChainId, destination_chain: ChainId) -> CCIHSResult<()> {
//         log::info!("Processing message: source: {:?}, destination: {:?}, nonce: {}",
//                    source_chain, destination_chain, message.nonce);
//         Ok(())
//     }
// }