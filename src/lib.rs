pub mod core;
pub mod protocols;
pub mod api;
pub mod hooks;
pub mod utility;
pub mod types;
pub mod state;
pub mod config;
pub mod constants;

pub use utility::{CCIHSError, serialization};

// pub use types::{
//     ChainId, 
//     CrossChainMessage, 
//     CrossChainTransaction, 
//     MessageStatus,
//     CCIHSResult,
//     CrossChainResult,
// };

pub use types::*;

pub use hooks::{Hook, HookType, HookManager};

pub use state::*;
pub use core::CCIHSCore;  // Export CCIHSCore for easy access
pub use config::CCIHSConfig;  
pub use api::CCIHSAPI; 
pub use protocols::*;
pub use constants::*;
