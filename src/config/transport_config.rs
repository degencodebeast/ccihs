// config/transport_config.rs

use crate::types::ChainId;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TransportType {
    Wormhole,
    LayerZero,
    // Add other transport types as needed
}

#[derive(Clone, Debug)]
pub struct TransportConfig {
    pub transport_type: TransportType,
    pub supported_chains: HashSet<ChainId>,
    pub additional_params: HashMap<String, String>,
}

impl TransportConfig {
    pub fn new(transport_type: TransportType) -> Self {
        Self {
            transport_type,
            supported_chains: HashSet::new(),
            additional_params: HashMap::new(),
        }
    }

    pub fn add_supported_chain(&mut self, chain_id: ChainId) {
        self.supported_chains.insert(chain_id);
    }

    pub fn add_param(&mut self, key: &str, value: &str) {
        self.additional_params.insert(key.to_string(), value.to_string());
    }
}