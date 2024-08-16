// api/ccihs_api.rs

use crate::core::CCIHSCore;
use crate::types::{CrossChainMessage, ChainId, CCIHSResult};
use crate::config::CCIHSConfig;
use super::endpoints;
use super::error::APIError;


/// The main API interface for interacting with the Cross-Chain Interoperability Hooks for Solana (CCIHS) system.
///
/// This API provides methods for sending and receiving cross-chain messages, verifying messages,
/// getting supported chains, and converting addresses between different chains.
pub struct CCIHSAPI {
    core: CCIHSCore,
}

impl CCIHSAPI {
     /// Creates a new instance of the CCIHS API.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the CCIHS system.
    ///
    /// # Returns
    ///
    /// A `CCIHSResult` containing the new `CCIHSAPI` instance if successful, or an error if initialization fails.
    pub fn new(config: CCIHSConfig) -> CCIHSResult<Self> {
        let core = CCIHSCore::new(config.clone(), config.chain_id, config.contract_addresses)?;
        Ok(Self { core })
    }

    /// Sends a cross-chain message.
    ///
    /// # Arguments
    ///
    /// * `message` - The `CrossChainMessage` to be sent.
    ///
    /// # Returns
    ///
    /// A `Result` containing a success message with the nonce of the sent message if successful,
    /// or an `APIError` if the operation fails.
    pub fn send_message(&self, message: CrossChainMessage) -> Result<String, APIError> {
        endpoints::send_message(&self.core, message)
    }


    /// Receives a cross-chain message from the specified source chain.
    ///
    /// # Arguments
    ///
    /// * `source_chain` - The `ChainId` of the source chain.
    ///
    /// # Returns
    ///
    /// A `Result` containing the received `CrossChainMessage` if successful,
    /// or an `APIError` if the operation fails.
    pub fn receive_message(&self, source_chain: ChainId) -> Result<CrossChainMessage, APIError> {
        endpoints::receive_message(&self.core, source_chain)
    }

    /// Verifies a cross-chain message.
    ///
    /// # Arguments
    ///
    /// * `message` - The `CrossChainMessage` to be verified.
    ///
    /// # Returns
    ///
    /// A `Result` containing a boolean indicating whether the message is valid,
    /// or an `APIError` if the verification process fails.
    pub fn verify_message(&self, message: CrossChainMessage) -> Result<bool, APIError> {
        endpoints::verify_message(&self.core, message)
    }

    /// Returns a list of supported chains.
    ///
    /// # Returns
    ///
    /// A `Vec<ChainId>` containing the IDs of all supported chains.
    pub fn get_supported_chains(&self) -> Vec<ChainId> {
        self.core.supported_chains().to_vec()
    }

     /// Converts an address from one chain format to another.
    ///
    /// # Arguments
    ///
    /// * `from` - The `ChainId` of the source chain.
    /// * `to` - The `ChainId` of the destination chain.
    /// * `address` - The address to be converted, as a byte vector.
    ///
    /// # Returns
    ///
    /// A `Result` containing the converted address as a byte vector if successful,
    /// or an `APIError` if the conversion fails.
    pub fn convert_address(&self, from: ChainId, to: ChainId, address: Vec<u8>) -> Result<Vec<u8>, APIError> {
        endpoints::convert_address(&self.core, from, to, address)
    }
}


// This implementation provides a comprehensive API for interacting with the CCIHS system. It includes:

// A main CCIHSAPI struct that wraps the CCIHSCore and provides high-level methods for interacting with the system.
// Separate endpoint implementations for each major operation (send, receive, verify, convert address).
// Proper error handling with a custom APIError type that can wrap internal CCIHS errors.
// Documentation for each public method in the API.

// To use this API, a user would typically do something like this:
// let config = CCIHSConfig::new(/* ... */);
// let api = CCIHSAPI::new(config)?;

// // Send a message
// Create a message
// let message = CrossChainMessage::new(
//     ChainId::Solana,    // source chain
//     ChainId::Ethereum,  // destination chain
//     [0u8; 32],          // sender
//     vec![1, 2, 3, 4],   // recipient
//     vec![5, 6, 7, 8],   // payload
// );
// let result = api.send_message(message)?;
// println!("{}", result);

// // Receive a message
// let received_message = api.receive_message(ChainId::Solana)?;
// println!("Received message: {:?}", received_message);

// // Verify a message
// let is_valid = api.verify_message(received_message)?;
// println!("Message is valid: {}", is_valid);

// // Convert an address
// let solana_address = vec![/* ... */];
// let ethereum_address = api.convert_address(ChainId::Solana, ChainId::Ethereum, solana_address)?;
// println!("Converted address: {:?}", ethereum_address);
// This API provides a clean and easy-to-use interface for interacting with the CCIHS system, while encapsulating the complexity of the underlying operations.