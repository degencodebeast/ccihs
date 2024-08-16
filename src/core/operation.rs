// src/core/operation.rs

use crate::types::{CrossChainMessage, ChainId, CCIHSResult};
use crate::config::{CCIHSConfig, TransportType};
use crate::error::CCIHSError;
use crate::hook::{HookManager, HookType};
use crate::transport::TransportAdapter;
use super::chain_management::ChainManager;
use super::error::CoreError;
use std::collections::HashMap;

pub struct CCIHSCore {
    config: CCIHSConfig,
    hook_manager: HookManager,
    transport_adapters: HashMap<TransportType, Box<dyn TransportAdapter>>,
    chain_manager: ChainManager,
}

impl CCIHSCore {
    pub fn new(
        config: CCIHSConfig,
        transport_adapters: HashMap<TransportType, Box<dyn TransportAdapter>>,
        supported_chains: Vec<ChainId>,
    ) -> CCIHSResult<Self> {
        config.validate()?;
        Ok(Self {
            config,
            hook_manager: HookManager::new(),
            transport_adapters,
            chain_manager: ChainManager::new(supported_chains),
        })
    }

    pub fn send_message(&self, message: &mut CrossChainMessage) -> CCIHSResult<()> {
        if !self.chain_manager.is_supported_chain(message.source_chain) {
            return Err(CoreError::UnsupportedChain(message.source_chain).into());
        }
        if !self.chain_manager.is_supported_chain(message.destination_chain) {
            return Err(CoreError::UnsupportedChain(message.destination_chain).into());
        }

        self.hook_manager.execute_hooks(HookType::PreDispatch, message, message.source_chain, message.destination_chain)?;

        // Convert addresses if necessary
        let converted_recipient = self.chain_manager.convert_address(
            message.source_chain,
            message.destination_chain,
            message.recipient.clone(),
        )?;
        message.recipient = converted_recipient;

        let adapter = self.transport_adapters.get(&self.config.default_transport)
            .ok_or(CCIHSError::TransportNotConfigured(self.config.default_transport.to_string()))?;

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

        let adapter = self.transport_adapters.get(&self.config.default_transport)
            .ok_or(CCIHSError::TransportNotConfigured(self.config.default_transport.to_string()))?;

        let mut message = adapter.receive_message(self.config.get_chain_config(&source_chain)?)?;

        self.hook_manager.execute_hooks(HookType::PreExecution, &mut message, source_chain, message.destination_chain)?;

        // Convert addresses if necessary
        let converted_sender = self.chain_manager.convert_address(
            message.source_chain,
            message.destination_chain,
            message.sender.to_vec(),
        )?;
        message.sender = converted_sender.try_into().map_err(|_| CCIHSError::AddressConversionError)?;

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

        let adapter = self.transport_adapters.get(&self.config.default_transport)
            .ok_or(CCIHSError::TransportNotConfigured(self.config.default_transport.to_string()))?;

        adapter.verify_message(message, 
            self.config.get_chain_config(&message.source_chain)?,
            self.config.get_chain_config(&message.destination_chain)?
        )
    }

    pub fn add_chain_conversion<F>(&mut self, from: ChainId, to: ChainId, conversion: F)
    where
        F: Fn(Vec<u8>) -> CCIHSResult<Vec<u8>> + 'static,
    {
        self.chain_manager.add_chain_conversion(from, to, conversion);
    }

    pub fn convert_address(&self, from: ChainId, to: ChainId, address: Vec<u8>) -> CCIHSResult<Vec<u8>> {
        self.chain_manager.convert_address(from, to, address)
    }

    pub fn supported_chains(&self) -> &[ChainId] {
        self.chain_manager.supported_chains()
    }
}