// src/core/chain_management.rs

use crate::types::{ChainId, CCIHSResult};
use super::error::CoreError;
use std::collections::HashMap;

pub struct ChainManager {
    supported_chains: Vec<ChainId>,
    chain_conversions: HashMap<(ChainId, ChainId), Box<dyn Fn(Vec<u8>) -> CCIHSResult<Vec<u8>>>>,
}

impl ChainManager {
    pub fn new(supported_chains: Vec<ChainId>) -> Self {
        Self {
            supported_chains,
            chain_conversions: HashMap::new(),
        }
    }

    pub fn is_supported_chain(&self, chain_id: ChainId) -> bool {
        self.supported_chains.contains(&chain_id)
    }

    pub fn add_chain_conversion<F>(&mut self, from: ChainId, to: ChainId, conversion: F)
    where
        F: Fn(Vec<u8>) -> CCIHSResult<Vec<u8>> + 'static,
    {
        self.chain_conversions.insert((from, to), Box::new(conversion));
    }

    pub fn convert_address(&self, from: ChainId, to: ChainId, address: Vec<u8>) -> CCIHSResult<Vec<u8>> {
        if !self.is_supported_chain(from) {
            return Err(CoreError::UnsupportedChain(from).into());
        }
        if !self.is_supported_chain(to) {
            return Err(CoreError::UnsupportedChain(to).into());
        }

        if from == to {
            log::debug!("No conversion needed for address from {:?} to {:?}", from, to);
            return Ok(address);
        }
    
        match self.chain_conversions.get(&(from, to)) {
            Some(conversion_fn) => {
                log::info!("Converting address from {:?} to {:?}", from, to);
                conversion_fn(address)
            }
            None => {
                log::warn!("No conversion function found for {:?} to {:?}", from, to);
                Err(CoreError::InvalidChainConversion { from, to }.into())
            }
        }
    }

    pub fn supported_chains(&self) -> &[ChainId] {
        &self.supported_chains
    }
}