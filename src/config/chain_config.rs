// config/chain_config.rs

use crate::types::ChainId;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ChainConfig {
    pub chain_id: ChainId,
    pub rpc_url: String,
    pub contract_addresses: HashMap<String, String>,
}

impl ChainConfig {
    pub fn new(chain_id: ChainId, rpc_url: String) -> Self {
        Self {
            chain_id,
            rpc_url,
            contract_addresses: HashMap::new(),
        }
    }

    pub fn add_contract_address(&mut self, name: &str, address: &str) {
        self.contract_addresses.insert(name.to_string(), address.to_string());
    }
}