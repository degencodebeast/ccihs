use crate::core::CCIHSCore;
use crate::types::{CrossChainMessage, ChainId, CCIHSResult};
use crate::config::CCIHSConfig;
use super::endpoints;
use super::error::APIError;
use crate::protocol::ProtocolType;

pub struct CCIHSAPI {
    core: CCIHSCore,
}

impl CCIHSAPI {
    pub fn new(config: CCIHSConfig) -> CCIHSResult<Self> {
        // let core = CCIHSCore::new(config)?;
        // Ok(Self { core })
        let core = CCIHSCore::new(config, config.protocol_adapters, config.supported_chains).map_err(APIError::from)?;
        Ok(Self { core })
    }

    pub fn send_message(&self, message: CrossChainMessage) -> Result<String, APIError> {
        endpoints::send_message(&self.core, message)
    }

    pub fn receive_message(&self, source_chain: ChainId) -> Result<CrossChainMessage, APIError> {
        endpoints::receive_message(&self.core, source_chain)
    }

    pub fn verify_message(&self, message: CrossChainMessage) -> Result<bool, APIError> {
        endpoints::verify_message(&self.core, message)
    }

    pub fn get_supported_chains(&self) -> Vec<ChainId> {
        self.core.supported_chains().to_vec()
    }

    pub fn convert_address(&self, from: ChainId, to: ChainId, address: Vec<u8>) -> Result<Vec<u8>, APIError> {
        endpoints::convert_address(&self.core, from, to, address)
    }

    pub fn add_hook(&mut self, hook_type: crate::hook::HookType, hook: Box<dyn crate::hook::Hook>) {
        self.core.add_hook(hook_type, hook);
    }

    pub fn set_default_protocol(&mut self, protocol: ProtocolType) -> Result<(), APIError> {
        self.core.set_default_protocol(protocol).map_err(APIError::from)
    }

    pub fn update_config(&mut self, new_config: CCIHSConfig) -> Result<(), APIError> {
        self.core.update_config(new_config).map_err(APIError::from)
    }
}