use crate::types::{CrossChainMessage, ChainId, CCIHSResult};

pub trait Hook: Send + Sync {
    fn execute(&self, message: &mut CrossChainMessage, source_chain: ChainId, destination_chain: ChainId) -> CCIHSResult<()>;
}

mod logging;
mod fee_calculation;
mod encryption;
mod rate_limiting;
mod validation;
mod metrics;

pub use logging::LoggingHook;
pub use fee_calculation::FeeCalculationHook;
pub use encryption::EncryptionHook;
pub use rate_limiting::RateLimitingHook;
pub use validation::MessageValidationHook;
pub use metrics::MetricsHook;