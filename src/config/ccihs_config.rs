// config/ccihs_config.rs

use super::{ChainConfig, TransportConfig, TransportType};
use crate::types::ChainId;
use crate::error::{CCIHSResult, CCIHSError};
use std::collections::HashMap;
use std::env;

#[derive(Clone, Debug)]
pub struct CCIHSConfig {
    pub chains: HashMap<ChainId, ChainConfig>,
    pub transports: HashMap<TransportType, TransportConfig>,
    pub default_transport: TransportType,
    pub max_retries: u32,
    pub retry_delay: u64,
}

impl CCIHSConfig {
    pub fn new() -> Self {
        let mut config = Self {
            chains: HashMap::new(),
            transports: HashMap::new(),
            default_transport: TransportType::Wormhole,
            max_retries: 3,
            retry_delay: 1000,
        };

        config.load_from_env();
        config
    }

    pub fn add_chain(&mut self, chain_config: ChainConfig) {
        self.chains.insert(chain_config.chain_id, chain_config);
    }

    pub fn add_transport(&mut self, transport_config: TransportConfig) {
        self.transports.insert(transport_config.transport_type.clone(), transport_config);
    }

    pub fn set_default_transport(&mut self, transport_type: TransportType) -> CCIHSResult<()> {
        if self.transports.contains_key(&transport_type) {
            self.default_transport = transport_type;
            Ok(())
        } else {
            Err(CCIHSError::TransportNotConfigured)
        }
    }

    pub fn get_chain_config(&self, chain_id: &ChainId) -> Option<&ChainConfig> {
        self.chains.get(chain_id)
    }

    pub fn get_transport_config(&self, transport_type: &TransportType) -> Option<&TransportConfig> {
        self.transports.get(transport_type)
    }

    fn load_from_env(&mut self) {
        if let Ok(transport) = env::var("CCIHS_DEFAULT_TRANSPORT") {
            match transport.as_str() {
                "wormhole" => self.default_transport = TransportType::Wormhole,
                "layerzero" => self.default_transport = TransportType::LayerZero,
                _ => {},
            }
        }

        if let Ok(retries) = env::var("CCIHS_MAX_RETRIES") {
            if let Ok(retries) = retries.parse() {
                self.max_retries = retries;
            }
        }

        if let Ok(delay) = env::var("CCIHS_RETRY_DELAY") {
            if let Ok(delay) = delay.parse() {
                self.retry_delay = delay;
            }
        }

        // Add more environment variable loads as needed
    }

    pub fn validate(&self) -> CCIHSResult<()> {
        if self.chains.is_empty() {
            return Err(CCIHSError::NoConfiguredChains);
        }
        if self.transports.is_empty() {
            return Err(CCIHSError::NoConfiguredTransports);
        }
        if !self.transports.contains_key(&self.default_transport) {
            return Err(CCIHSError::InvalidDefaultTransport);
        }
        Ok(())
    }
}

impl Default for CCIHSConfig {
    fn default() -> Self {
        Self::new()
    }
}

// This implementation provides a flexible configuration system for CCIHS with the following features:

// It supports multiple chains and transport networks (like Wormhole and LayerZero).
// Configurations are stored in-memory during runtime.
// It allows for easy addition of new chains and transport networks.
// It provides methods to retrieve specific chain and transport configurations.
// It includes a validation method to ensure the configuration is valid before use.
// It loads some basic configurations from environment variables, allowing for easy overrides without changing the code.
// It uses sensible defaults where possible.

// To use this configuration in your CCIHS core, you would typically:
// rustCopylet mut config = CCIHSConfig::new();

// // Add chain configurations
// config.add_chain(ChainConfig::new(ChainId::Solana, "https://api.solana.com".to_string()));
// config.add_chain(ChainConfig::new(ChainId::Ethereum, "https://mainnet.infura.io/v3/YOUR-PROJECT-ID".to_string()));

// // Add transport configurations
// let mut wormhole_config = TransportConfig::new(TransportType::Wormhole);
// wormhole_config.add_supported_chain(ChainId::Solana);
// wormhole_config.add_supported_chain(ChainId::Ethereum);
// config.add_transport(wormhole_config);

// // Validate the configuration
// config.validate().expect("Invalid configuration");

// // Use the configuration to initialize CCIHS
// let ccihs = CCIHS::new(config);
// This approach provides a balance between flexibility and simplicity, allowing CCIHS to be configured for different use cases while keeping the configuration process straightforward.