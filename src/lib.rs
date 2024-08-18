pub mod core;
pub mod protocol;
pub mod api;
pub mod hook;
pub mod utility;
pub mod types;
pub mod state;
// pub mod config;  // Add this if you haven't already
// pub mod transport;  // Add this for TransportAdapter



pub use utility::{CCIHSError, serialization};

pub use types::{
    ChainId, 
    CrossChainMessage, 
    CrossChainTransaction, 
    MessageStatus,
    CCIHSResult,
    CrossChainResult,
};

pub use state::*;
// pub use core::CCIHSCore;  // Export CCIHSCore for easy access
// pub use config::CCIHSConfig;  // Export CCIHSConfig
// pub use api::CCIHSAPI;  // Export CCIHSAPI for users of your library