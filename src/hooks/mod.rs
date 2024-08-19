mod encryption;
mod fee_calculation;
mod logging;
mod metrics;
mod rate_limiting;
mod validation;
mod hook_manager;

pub use encryption::EncryptionHook;
pub use fee_calculation::FeeCalculationHook;
pub use logging::LoggingHook;
pub use metrics::MetricsHook;
pub use rate_limiting::RateLimitingHook;
pub use validation::ValidationHook;
pub use hook_manager::HookManager;

use crate::types::{CrossChainMessage, ChainId, CCIHSResult, MessageStatus};
pub use anchor_lang::solana_program::log::sol_log;

pub trait Hook: Send + Sync {
    fn execute(&self, message: &mut CrossChainMessage, source_chain: ChainId, destination_chain: ChainId) -> CCIHSResult<()>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HookType {
    PreDispatch,
    PostDispatch,
    PreExecution,
    PostExecution,
}

impl HookType {

    pub fn execute_default(&self, message: &mut CrossChainMessage, source_chain: ChainId, destination_chain: ChainId) -> CCIHSResult<()> {
        match self {
            HookType::PreDispatch => self.default_pre_dispatch(message, source_chain, destination_chain),
            HookType::PostDispatch => self.default_post_dispatch(message, source_chain, destination_chain),
            HookType::PreExecution => self.default_pre_execution(message, source_chain, destination_chain),
            HookType::PostExecution => self.default_post_execution(message, source_chain, destination_chain),
        }
    }

    fn default_pre_dispatch(&self, message: &mut CrossChainMessage, _source_chain: ChainId, _destination_chain: ChainId) -> CCIHSResult<()> {
        log::info!("Performing default pre-dispatch checks");
        
        // Validate the message
        if message.payload.is_empty() {
            return Err(CCIHSError::EmptyPayload);
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

    fn default_post_dispatch(&self, message: &mut CrossChainMessage, _source_chain: ChainId, _destination_chain: ChainId) -> CCIHSResult<()> {
        log::info!("Performing default post-dispatch checks");

        if message.nonce == 0 {
            return Err(CCIHSError::InvalidNonce);
        }

          // Log the dispatched message
          sol_log(&format!(
            "Message dispatched: from {} to {}, nonce: {}, timestamp: {}",
            source_chain, destination_chain, message.nonce, message.timestamp
        ));

        // You could update some on-chain statistics here if needed
        // For example, incrementing a counter for messages sent

        Ok(())
        //Ok(())
    }

    fn default_pre_execution(&self, message: &mut CrossChainMessage, _source_chain: ChainId, _destination_chain: ChainId) -> CCIHSResult<()> {
        log::info!("Performing default pre-execution checks");
        
        // Verify that the message hasn't expired
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| CCIHSError::TimestampError)?
            .as_secs();

        if current_time - message.timestamp > MAX_MESSAGE_AGE {
            return Err(CCIHSError::MessageExpired);
        }

            // Verify that the source chain matches the expected chain
        if message.source_chain != source_chain {
            return Err(CCIHSError::ChainMismatch);
        }

        // You could perform additional checks here, such as:
        // - Verifying the message format is correct for the specific cross-chain operation
        // - Checking if the sender has the necessary permissions
        Ok(())
    }

    fn default_post_execution(&self, message: &mut CrossChainMessage, _source_chain: ChainId, _destination_chain: ChainId) -> CCIHSResult<()> {
        log::info!("Performing default post-execution checks");
        if message.status != MessageStatus::Executed {
            return Err(CCIHSError::MessageNotExecuted);
        }

         // Log the executed message
         sol_log(&format!(
            "Message executed: from {} to {}, nonce: {}, timestamp: {}",
            source_chain, destination_chain, message.nonce, message.timestamp
        ));

        // You could update some on-chain statistics here if needed
        // For example, incrementing a counter for messages received and executed

        // If you're implementing a request-response pattern, you could trigger a response here

        Ok(())
        //Ok(())
    }
    
}

const MAX_MESSAGE_AGE: u64 = 3600; // 1 hour, adjust as needed

fn is_supported_chain(chain_id: ChainId) -> bool {
    // Implement your chain support logic here
    // For example:
    matches!(chain_id, ChainId::SOLANA | ChainId::ETHEREUM)
}