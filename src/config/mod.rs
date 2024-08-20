// config/mod.rs

mod ccihs_config;
mod chain_config;
mod protocol_config;

pub use ccihs_config::CCIHSConfig;
pub use chain_config::ChainConfig;
pub use protocol_config::ProtocolConfig;

// If you want to re-export everything from these modules, you can use:
// pub use ccihs_config::*;
// pub use chain_config::*;
// pub use protocol_config::*;