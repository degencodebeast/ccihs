// config/Protocol_config.rs

use crate::types::{ChainId, ProtocolType};
use std::collections::{HashSet, HashMap};

pub trait ProtocolConfigTrait {
    fn protocol_type(&self) -> ProtocolType;
    fn supported_chains(&self) -> &HashSet<ChainId>;
    fn additional_params(&self) -> &HashMap<String, String>;
}

#[derive(Clone, Debug)]
pub struct ProtocolConfig {
    pub protocol_type: ProtocolType,
    pub supported_chains: HashSet<ChainId>,
    pub additional_params: HashMap<String, String>,
}

impl ProtocolConfig {
    pub fn new(protocol_type: ProtocolType) -> Self {
        Self {
            protocol_type,
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

impl ProtocolConfigTrait for ProtocolConfig {
    fn protocol_type(&self) -> ProtocolType {
        self.protocol_type.clone()
    }

    fn supported_chains(&self) -> &HashSet<ChainId> {
        &self.supported_chains
    }

    fn additional_params(&self) -> &HashMap<String, String> {
        &self.additional_params
    }
}