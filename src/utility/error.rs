// src/utility/error.rs

use thiserror::Error;
use solana_program::program_error::ProgramError;

#[derive(Debug, Error, Clone)]
pub enum CCIHSError {
    #[error("Invalid chain ID")]
    InvalidChainId,

    #[error("Message too large")]
    MessageTooLarge,

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Unsupported operation")]
    UnsupportedOperation,

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Hook execution error: {0}")]
    HookExecutionError(String),

    #[error("Solana program error: {0}")]
    SolanaProgramError(#[from] ProgramError),

    // Add more error types as needed
}