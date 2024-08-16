// src/core/error.rs

use thiserror::Error;
use crate::types::ChainId;

#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(ChainId),

    #[error("Invalid chain conversion: from {from} to {to}")]
    InvalidChainConversion { from: ChainId, to: ChainId },

    #[error("Operation not supported: {0}")]
    UnsupportedOperation(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}