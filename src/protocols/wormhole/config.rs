use anchor_lang::prelude::*;
use crate::config::protocol_config::ProtocolConfigTrait;
use crate::types::{ChainId, ProtocolType};
use std::collections::{HashSet, HashMap};
use std::collections::BTreeMap;

pub struct WormholeAddresses {
    pub bridge: Pubkey,
    pub fee_collector: Pubkey,
    pub sequence: Pubkey,
}

impl WormholeAddresses {
    pub const LEN: usize = 32 + 32 + 32;
}

#[account]
//#[derive(Default)]
#[derive(Default, AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct WormholeConfig {
    pub owner: Pubkey,
    pub wormhole: WormholeAddresses,
    pub batch_id: u32,
    pub finality: u8,
    pub foreign_emitters: BTreeMap<u16, Pubkey>,
    pub supported_chains: HashSet<ChainId>,
    pub additional_params: HashMap<String, String>,
}

impl WormholeConfig {
  
    pub const SPACE: usize = 8 // discriminator
    + 32 // owner
    + WormholeAddresses::LEN
    + 4 // batch_id
    + 1 // finality
    + 64 // Estimate for foreign_emitters
    + 64 // Estimate for supported_chains
    + 64 // Estimate for additional_params
    ;

    pub const SEED_PREFIX: &'static [u8; 15] = b"wormhole_config";

    pub fn new(
        owner: Pubkey,
        wormhole_bridge: Pubkey,
        wormhole_fee_collector: Pubkey,
        //wormhole_emitter: Pubkey,
        wormhole_sequence: Pubkey,
    ) -> Self {
        Self {
            owner,
            wormhole: WormholeAddresses {
                bridge: wormhole_bridge,
                fee_collector: wormhole_fee_collector,
                sequence: wormhole_sequence,
            },
            batch_id: 0,
            finality: 0,
            foreign_emitters: BTreeMap::new(),
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

    pub fn set_parameter(&mut self, key: &str, value: &str) {
        self.additional_params.insert(key.to_string(), value.to_string());
    }

    fn get_parameter(&self, key: &str) -> Option<&str> {
        self.additional_params.get(key)
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
