use anchor_lang::prelude::*;
use crate::config::protocol_config::ProtocolConfigTrait;
use crate::types::{ChainId, ProtocolType};
use std::collections::{HashSet, HashMap};

#[account]
#[derive(Default)]
pub struct WormholeConfig {
    pub owner: Pubkey,
    pub fee: u64,
    pub wormhole_bridge: Pubkey,
    pub wormhole_fee_collector: Pubkey,
    pub wormhole_emitter: Pubkey,
    pub wormhole_sequence: Pubkey,
    supported_chains: HashSet<ChainId>,
    additional_params: HashMap<String, String>,
}

impl WormholeConfig {
    pub const SPACE: usize = 32 + 8 + 32 + 32 + 32 + 32 + 64 + 64; // Adjust as needed

    pub fn new(
        owner: Pubkey,
        fee: u64,
        wormhole_bridge: Pubkey,
        wormhole_fee_collector: Pubkey,
        wormhole_emitter: Pubkey,
        wormhole_sequence: Pubkey,
    ) -> Self {
        Self {
            owner,
            fee,
            wormhole_bridge,
            wormhole_fee_collector,
            wormhole_emitter,
            wormhole_sequence,
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

impl ProtocolConfigTrait for WormholeConfig {
    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Wormhole
    }

    fn supported_chains(&self) -> &HashSet<ChainId> {
        &self.supported_chains
    }

    fn additional_params(&self) -> &HashMap<String, String> {
        &self.additional_params
    }
}