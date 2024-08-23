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

use crate::types::{CrossChainMessage, ChainId, CCIHSResult, MessageStatus, HookType};
pub use anchor_lang::solana_program::log::sol_log;

pub trait Hook: Send + Sync {
    fn execute(&self, message: &mut CrossChainMessage, source_chain: ChainId, destination_chain: ChainId) -> CCIHSResult<()>;
}