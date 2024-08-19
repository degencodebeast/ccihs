// src/utility/error.rs

use thiserror::Error;
use solana_program::program_error::ProgramError;
use crate::core::error::CoreError;

#[derive(Debug, Error, Clone)]
pub enum CCIHSError {

    #[error("Core error: {0}")]
    Core(#[from] CoreError),

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

    #[error("Protocol not configured: {0}")]
    ProtocolNotConfigured(String),

    #[error("Empty payload")]
    EmptyPayload,

    #[error("Invalid nonce")]
    InvalidNonce,

    #[error("Message expired")]
    MessageExpired,

    #[error("Message not executed")]
    MessageNotExecuted,
    
    #[error("Timestamp error")]
    TimestampError,

    // Add more error types as needed
}