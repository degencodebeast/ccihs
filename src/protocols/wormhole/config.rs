use anchor_lang::prelude::*;
use crate::config::protocol_config::ProtocolConfigTrait;
use crate::types::{ChainId, ProtocolType};
use std::collections::{HashSet, HashMap};
use std::collections::BTreeMap;

#[account]
//#[derive(Default)]
#[derive(Default, AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct WormholeConfig {
    pub owner: Pubkey,
    pub fee: u64,
    /// [BridgeData](wormhole_anchor_sdk::wormhole::BridgeData) address.
    pub wormhole_bridge: Pubkey,
    /// [FeeCollector](wormhole_anchor_sdk::wormhole::FeeCollector) address.
    pub wormhole_fee_collector: Pubkey,
    /// [SequenceTracker](wormhole_anchor_sdk::wormhole::SequenceTracker) address.
    pub wormhole_sequence: Pubkey,
    pub wormhole_emitter: Pubkey,
    pub foreign_emitters: BTreeMap<u16, Pubkey>,
    //pub foreign_emitters: BTreeMap<u16, ForeignEmitter>,
    pub bump: u8,
    supported_chains: HashSet<ChainId>,
    additional_params: HashMap<String, String>,
}

impl WormholeConfig {
    pub const SPACE: usize = 32 + 8 + 32 + 32 + 32 + 32 + 64 + 64 + 1 + 64 + 64;

    pub const SEED_PREFIX: &'static [u8; 6] = b"wormhole_config";

    pub fn new(
        owner: Pubkey,
        fee: u64,
        wormhole_bridge: Pubkey,
        wormhole_fee_collector: Pubkey,
        wormhole_emitter: Pubkey,
        wormhole_sequence: Pubkey,
        bump: u8,
    ) -> Self {
        Self {
            owner,
            fee,
            wormhole_bridge,
            wormhole_fee_collector,
            wormhole_emitter,
            wormhole_sequence,
            foreign_emitters: BTreeMap::new(),
            bump,
            supported_chains: HashSet::new(),
            additional_params: HashMap::new(),
        }
    }

    pub fn add_foreign_emitter(&mut self, chain: u16, emitter: Pubkey) {
        self.foreign_emitters.insert(chain, emitter);
    }

    pub fn get_foreign_emitter(&self, chain: u16) -> Option<&Pubkey> {
        self.foreign_emitters.get(&chain)
    }

    pub fn remove_foreign_emitter(&mut self, chain: u16) -> Option<Pubkey> {
        self.foreign_emitters.remove(&chain)
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
