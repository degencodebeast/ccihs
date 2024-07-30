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

use crate::types::{CrossChainMessage, ChainId, CCIHSResult}; // Import necessary types from the types module
use crate::protocol::ProtocolAdapter; // Import the ProtocolAdapter trait
use super::chain_management::ChainManager; // Import ChainManager from the parent module
use super::error::CoreError; // Import CoreError from the parent module
use crate::utility::error::CCIHSError; // Import CCIHSError from the utility module

pub struct CCIHSCore { // Define the main structure for the CCIHS core
    protocol: Box<dyn ProtocolAdapter>, // Store a boxed trait object for the protocol adapter
    chain_manager: ChainManager, // Store an instance of the ChainManager
}

impl CCIHSCore { // Implement methods for CCIHSCore
    pub fn new(protocol: Box<dyn ProtocolAdapter>, supported_chains: Vec<ChainId>) -> Self { // Constructor
        Self {
            protocol, // Store the provided protocol adapter
            chain_manager: ChainManager::new(supported_chains), // Create a new ChainManager with supported chains
        }
    }

    pub fn send_message(&self, message: &CrossChainMessage) -> CCIHSResult<()> { // Method to send a cross-chain message
        if !self.chain_manager.is_supported_chain(message.source_chain) { // Check if source chain is supported
            return Err(CoreError::UnsupportedChain(message.source_chain).into()); // Return error if not supported
        }
        if !self.chain_manager.is_supported_chain(message.destination_chain) { // Check if destination chain is supported
            return Err(CoreError::UnsupportedChain(message.destination_chain).into()); // Return error if not supported
        }

        message.validate()?; // Validate the message, propagate any error
        self.protocol.send_message(message) // Send the message using the protocol adapter
    }

    pub fn receive_message(&self, source_chain: ChainId) -> CCIHSResult<CrossChainMessage> { // Method to receive a message
        if !self.chain_manager.is_supported_chain(source_chain) { // Check if source chain is supported
            return Err(CoreError::UnsupportedChain(source_chain).into()); // Return error if not supported
        }

        self.protocol.receive_message(source_chain) // Receive message using the protocol adapter
    }

    pub fn verify_message(&self, message: &CrossChainMessage) -> CCIHSResult<bool> { // Method to verify a message
        if !self.chain_manager.is_supported_chain(message.source_chain) { // Check if source chain is supported
            return Err(CoreError::UnsupportedChain(message.source_chain).into()); // Return error if not supported
        }
        if !self.chain_manager.is_supported_chain(message.destination_chain) { // Check if destination chain is supported
            return Err(CoreError::UnsupportedChain(message.destination_chain).into()); // Return error if not supported
        }

        self.protocol.verify_message(message) // Verify the message using the protocol adapter
    }

    pub fn supported_chains(&self) -> &[ChainId] { // Method to get supported chains
        self.chain_manager.supported_chains() // Return supported chains from the chain manager
    }

    pub fn add_chain_conversion<F>(&mut self, from: ChainId, to: ChainId, conversion: F) // Method to add a chain conversion
    where
        F: Fn(Vec<u8>) -> CCIHSResult<Vec<u8>> + 'static, // F is a function that converts addresses
    {
        self.chain_manager.add_chain_conversion(from, to, conversion); // Add the conversion to the chain manager
    }

    pub fn convert_address(&self, from: ChainId, to: ChainId, address: Vec<u8>) -> CCIHSResult<Vec<u8>> { // Method to convert addresses
        self.chain_manager.convert_address(from, to, address) // Use the chain manager to convert the address
    }
}

#[cfg(test)] // Start of test module
mod tests {
    use super::*; // Import all items from the parent module
    use std::cell::RefCell; // Import RefCell for interior mutability
    use std::rc::Rc; // Import Rc for reference counting

    struct MockProtocolAdapter { // Define a mock protocol adapter for testing
        sent_messages: Rc<RefCell<Vec<CrossChainMessage>>>, // Store sent messages
    }

    impl ProtocolAdapter for MockProtocolAdapter { // Implement ProtocolAdapter for the mock
        fn send_message(&self, message: &CrossChainMessage) -> CCIHSResult<()> { // Mock send_message
            self.sent_messages.borrow_mut().push(message.clone()); // Store the sent message
            Ok(()) // Return success
        }

        fn receive_message(&self, _source_chain: ChainId) -> CCIHSResult<CrossChainMessage> { // Mock receive_message
            unimplemented!() // Not implemented for this test
        }

        fn verify_message(&self, _message: &CrossChainMessage) -> CCIHSResult<bool> { // Mock verify_message
            Ok(true) // Always return true for this test
        }

        fn supported_chains(&self) -> Vec<ChainId> { // Mock supported_chains
            vec![ChainId::SOLANA, ChainId::ETHEREUM] // Return two supported chains
        }
    }

    #[test] // Test function
    fn test_ccihs_core() {
        let sent_messages = Rc::new(RefCell::new(Vec::new())); // Create a shared, mutable vector for sent messages
        let protocol = Box::new(MockProtocolAdapter { // Create a boxed MockProtocolAdapter
            sent_messages: sent_messages.clone(), // Clone the Rc of sent_messages
        });

        let mut core = CCIHSCore::new(protocol, vec![ChainId::SOLANA, ChainId::ETHEREUM]); // Create a new CCIHSCore instance

        let message = CrossChainMessage { // Create a test message
            source_chain: ChainId::SOLANA,
            destination_chain: ChainId::ETHEREUM,
            sender: [0u8; 32].into(),
            recipient: vec![1, 2, 3, 4],
            payload: vec![5, 6, 7, 8],
            nonce: 1,
            timestamp: 12345,
        };

        assert!(core.send_message(&message).is_ok()); // Test sending a message
        assert_eq!(sent_messages.borrow().len(), 1); // Check that one message was sent

        assert!(core.verify_message(&message).unwrap()); // Test verifying a message

        core.add_chain_conversion(ChainId::SOLANA, ChainId::ETHEREUM, |addr| { // Add a test chain conversion
            Ok(addr.into_iter().rev().collect()) // Reverse the address bytes
        });

        let converted = core.convert_address(ChainId::SOLANA, ChainId::ETHEREUM, vec![1, 2, 3, 4]).unwrap(); // Test address conversion
        assert_eq!(converted, vec![4, 3, 2, 1]); // Check that the address was reversed

        assert!(core.send_message(&CrossChainMessage { // Test sending a message with an unsupported chain
            source_chain: ChainId::new(999), // Use an unsupported chain ID
            ..message // Use the rest of the fields from the previous message
        }).is_err()); // This should result in an error
    }
}