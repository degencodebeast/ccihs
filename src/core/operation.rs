use crate::types::{CrossChainMessage, ChainId, CCIHSResult, ProtocolType, HookType};
use crate::config::CCIHSConfig;
use crate::CCIHSError;
use crate::hooks::{HookManager, Hook};
use crate::protocols::ProtocolAdapter;
use super::chain_management::ChainManager;
use super::error::CoreError;
use std::collections::HashMap;

pub struct CCIHSCore {
    config: CCIHSConfig,
    hook_manager: HookManager,
    protocol_adapters: HashMap<ProtocolType, Box<dyn ProtocolAdapter>>,
    chain_manager: ChainManager,
}

impl CCIHSCore {
    pub fn new(
        config: CCIHSConfig,
        protocol_adapters: HashMap<ProtocolType, Box<dyn ProtocolAdapter>>,
        supported_chains: Vec<ChainId>,
    ) -> CCIHSResult<Self> {
        config.validate()?;
        Ok(Self {
            config,
            hook_manager: HookManager::new(),
            protocol_adapters,
            chain_manager: ChainManager::new(supported_chains),
        })
    }

    pub fn add_hook(&mut self, hook_type: HookType, hook: Box<dyn Hook>) {
        self.hook_manager.add_hook(hook_type, hook);
    }

    pub fn remove_hook(&mut self, hook_type: HookType, index: usize) -> CCIHSResult<()> {
        self.hook_manager.remove_hook(hook_type, index)
    }

    pub fn clear_hooks(&mut self, hook_type: HookType) {
        self.hook_manager.clear_hooks(hook_type);
    }

    pub fn send_message(&self, message: &mut CrossChainMessage) -> CCIHSResult<()> {
        if !self.chain_manager.is_supported_chain(message.source_chain) {
            return Err(CoreError::UnsupportedChain(message.source_chain).into());
        }
        if !self.chain_manager.is_supported_chain(message.destination_chain) {
            return Err(CoreError::UnsupportedChain(message.destination_chain).into());
        }

        self.hook_manager.execute_hooks(HookType::PreDispatch, message, message.source_chain, message.destination_chain)?;

        let converted_recipient = self.chain_manager.convert_address(
            message.source_chain,
            message.destination_chain,
            &message.recipient,
        )?;
        message.recipient = converted_recipient;

        let adapter = self.protocol_adapters.get(&self.config.default_protocol)
            .ok_or(CCIHSError::ProtocolNotConfigured(self.config.default_protocol.to_string()))?;

        adapter.send_message(message, 
            self.config.get_chain_config(&message.source_chain)?,
            self.config.get_chain_config(&message.destination_chain)?
        )?;

        self.hook_manager.execute_hooks(HookType::PostDispatch, message, message.source_chain, message.destination_chain)?;

        Ok(())
    }

    pub fn receive_message(&self, source_chain: ChainId) -> CCIHSResult<CrossChainMessage> {
        if !self.chain_manager.is_supported_chain(source_chain) {
            return Err(CoreError::UnsupportedChain(source_chain).into());
        }

        let adapter = self.protocol_adapters.get(&self.config.default_protocol)
            .ok_or(CCIHSError::ProtocolNotConfigured(self.config.default_protocol.to_string()))?;

        let mut message = adapter.receive_message(self.config.get_chain_config(&source_chain)?)?;

        self.hook_manager.execute_hooks(HookType::PreExecution, &mut message, source_chain, message.destination_chain)?;

        let converted_sender = self.chain_manager.convert_address(
            message.source_chain,
            message.destination_chain,
            &message.sender,
        )?;
        message.sender = converted_sender;

        self.hook_manager.execute_hooks(HookType::PostExecution, &mut message, source_chain, message.destination_chain)?;

        Ok(message)
    }

    pub fn verify_message(&self, message: &CrossChainMessage) -> CCIHSResult<bool> {
        if !self.chain_manager.is_supported_chain(message.source_chain) {
            return Err(CoreError::UnsupportedChain(message.source_chain).into());
        }
        if !self.chain_manager.is_supported_chain(message.destination_chain) {
            return Err(CoreError::UnsupportedChain(message.destination_chain).into());
        }

        let adapter = self.protocol_adapters.get(&self.config.default_protocol)
            .ok_or(CCIHSError::ProtocolNotConfigured(self.config.default_protocol.to_string()))?;

        adapter.verify_message(message, 
            self.config.get_chain_config(&message.source_chain)?,
            self.config.get_chain_config(&message.destination_chain)?
        )
    }

    pub fn add_chain_conversion<F>(&mut self, from: ChainId, to: ChainId, conversion: F)
    where
        F: Fn(&[u8]) -> CCIHSResult<Vec<u8>> + 'static + Send + Sync,
    {
        self.chain_manager.add_chain_conversion(from, to, conversion);
    }

    pub fn convert_address(&self, from: ChainId, to: ChainId, address: &[u8]) -> CCIHSResult<Vec<u8>> {
        self.chain_manager.convert_address(from, to, address)
    }

    pub fn supported_chains(&self) -> &[ChainId] {
        self.chain_manager.supported_chains()
    }

    pub fn get_config(&self) -> &CCIHSConfig {
        &self.config
    }

    pub fn update_config(&mut self, new_config: CCIHSConfig) -> CCIHSResult<()> {
        new_config.validate()?;
        self.config = new_config;
        Ok(())
    }

    pub fn get_protocol_adapter(&self, protocol_type: &ProtocolType) -> CCIHSResult<&Box<dyn ProtocolAdapter>> {
        self.protocol_adapters.get(protocol_type)
            .ok_or_else(|| CCIHSError::ProtocolNotConfigured(protocol_type.to_string()))
    }
}