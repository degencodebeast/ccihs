// use crate::types::{CrossChainMessage, CCIHSResult, ChainId};
// use crate::protocol::ProtocolAdapter;

// pub struct CCIHSCore {
//     protocol: Box<dyn ProtocolAdapter>,
// }

// impl CCIHSCore {
//     pub fn new(protocol: Box<dyn ProtocolAdapter>) -> Self {
//         Self { protocol }
//     }

//     pub fn send_message(&self, message: &CrossChainMessage) -> CCIHSResult<()> {
//         message.validate()?;
//         self.protocol.send_message(message)
//     }

//     // ... rest of the implementation remains the same
// }


use crate::types::{CrossChainMessage, ChainId, CCIHSResult};
use crate::protocol::ProtocolAdapter;
use super::chain_management::ChainManager;
use super::error::CoreError;
use crate::utility::error::CCIHSError;

pub struct CCIHSCore {
    protocol: Box<dyn ProtocolAdapter>,
    chain_manager: ChainManager,
}

impl CCIHSCore {
    pub fn new(protocol: Box<dyn ProtocolAdapter>, supported_chains: Vec<ChainId>) -> Self {
        Self {
            protocol,
            chain_manager: ChainManager::new(supported_chains),
        }
    }

    pub fn send_message(&self, message: &CrossChainMessage) -> CCIHSResult<()> {
        if !self.chain_manager.is_supported_chain(message.source_chain) {
            return Err(CoreError::UnsupportedChain(message.source_chain).into());
        }
        if !self.chain_manager.is_supported_chain(message.destination_chain) {
            return Err(CoreError::UnsupportedChain(message.destination_chain).into());
        }

        message.validate()?;
        self.protocol.send_message(message)
    }

    pub fn receive_message(&self, source_chain: ChainId) -> CCIHSResult<CrossChainMessage> {
        if !self.chain_manager.is_supported_chain(source_chain) {
            return Err(CoreError::UnsupportedChain(source_chain).into());
        }

        self.protocol.receive_message(source_chain)
    }

    pub fn verify_message(&self, message: &CrossChainMessage) -> CCIHSResult<bool> {
        if !self.chain_manager.is_supported_chain(message.source_chain) {
            return Err(CoreError::UnsupportedChain(message.source_chain).into());
        }
        if !self.chain_manager.is_supported_chain(message.destination_chain) {
            return Err(CoreError::UnsupportedChain(message.destination_chain).into());
        }

        self.protocol.verify_message(message)
    }

    pub fn supported_chains(&self) -> &[ChainId] {
        self.chain_manager.supported_chains()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct MockProtocolAdapter {
        sent_messages: Rc<RefCell<Vec<CrossChainMessage>>>,
    }

    impl ProtocolAdapter for MockProtocolAdapter {
        fn send_message(&self, message: &CrossChainMessage) -> CCIHSResult<()> {
            self.sent_messages.borrow_mut().push(message.clone());
            Ok(())
        }

        fn receive_message(&self, _source_chain: ChainId) -> CCIHSResult<CrossChainMessage> {
            unimplemented!()
        }

        fn verify_message(&self, _message: &CrossChainMessage) -> CCIHSResult<bool> {
            Ok(true)
        }

        fn supported_chains(&self) -> Vec<ChainId> {
            vec![ChainId::SOLANA, ChainId::ETHEREUM]
        }
    }

    #[test]
    fn test_ccihs_core() {
        let sent_messages = Rc::new(RefCell::new(Vec::new()));
        let protocol = Box::new(MockProtocolAdapter {
            sent_messages: sent_messages.clone(),
        });

        let mut core = CCIHSCore::new(protocol, vec![ChainId::SOLANA, ChainId::ETHEREUM]);

        let message = CrossChainMessage {
            source_chain: ChainId::SOLANA,
            destination_chain: ChainId::ETHEREUM,
            sender: [0u8; 32].into(),
            recipient: vec![1, 2, 3, 4],
            payload: vec![5, 6, 7, 8],
            nonce: 1,
            timestamp: 12345,
        };

        assert!(core.send_message(&message).is_ok());
        assert_eq!(sent_messages.borrow().len(), 1);

        assert!(core.verify_message(&message).unwrap());

        core.add_chain_conversion(ChainId::SOLANA, ChainId::ETHEREUM, |addr| {
            Ok(addr.into_iter().rev().collect())
        });

        let converted = core.convert_address(ChainId::SOLANA, ChainId::ETHEREUM, vec![1, 2, 3, 4]).unwrap();
        assert_eq!(converted, vec![4, 3, 2, 1]);

        assert!(core.send_message(&CrossChainMessage {
            source_chain: ChainId::new(999),
            ..message
        }).is_err());
    }
}