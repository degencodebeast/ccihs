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
            return Ok(address);
        }

        self.chain_conversions
            .get(&(from, to))
            .ok_or_else(|| CoreError::InvalidChainConversion { from, to })?
            (address)
    }

    pub fn supported_chains(&self) -> &[ChainId] {
        &self.supported_chains
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_manager() {
        let mut manager = ChainManager::new(vec![ChainId::SOLANA, ChainId::ETHEREUM]);

        assert!(manager.is_supported_chain(ChainId::SOLANA));
        assert!(manager.is_supported_chain(ChainId::ETHEREUM));
        assert!(!manager.is_supported_chain(ChainId::new(999)));

        manager.add_chain_conversion(ChainId::SOLANA, ChainId::ETHEREUM, |addr| {
            Ok(addr.into_iter().rev().collect())
        });

        let solana_addr = vec![1, 2, 3, 4];
        let eth_addr = manager.convert_address(ChainId::SOLANA, ChainId::ETHEREUM, solana_addr.clone()).unwrap();
        assert_eq!(eth_addr, vec![4, 3, 2, 1]);

        assert!(manager.convert_address(ChainId::SOLANA, ChainId::new(999), solana_addr).is_err());
    }
}