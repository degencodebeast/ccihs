mod encryption;
mod fee_calculation;
mod logging;
mod metrics;
mod rate_limiting;
mod validation;

pub use encryption::EncryptionHook;
pub use fee_calculation::FeeCalculationHook;
pub use logging::LoggingHook;
pub use metrics::MetricsHook;
pub use rate_limiting::RateLimitingHook;
pub use validation::ValidationHook;

use crate::types::{CrossChainMessage, ChainId, CCIHSResult};

pub trait Hook: Send + Sync {
    fn execute(&self, message: &mut CrossChainMessage, source_chain: ChainId, destination_chain: ChainId) -> CCIHSResult<()>;
}