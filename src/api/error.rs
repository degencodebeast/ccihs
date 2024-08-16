// api/error.rs

use thiserror::Error;
use crate::error::CCIHSError;

#[derive(Error, Debug)]
pub enum APIError {
    #[error("Internal CCIHS error: {0}")]
    Internal(#[from] CCIHSError),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

impl From<CCIHSError> for APIError {
    fn from(error: CCIHSError) -> Self {
        APIError::Internal(error)
    }
}