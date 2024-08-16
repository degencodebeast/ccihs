// api/mod.rs

mod ccihs_api;
mod endpoints;
mod error;

pub use ccihs_api::CCIHSAPI;
pub use error::APIError;