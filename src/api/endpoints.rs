use crate::core::CCIHSCore;
use crate::types::{CrossChainMessage, ChainId, CCIHSResult};
use super::error::APIError;

pub fn send_message(core: &CCIHSCore, mut message: CrossChainMessage) -> Result<String, APIError> {
    core.send_message(&mut message)
        .map_err(APIError::from)
        .map(|_| format!("Message sent successfully. Nonce: {}", message.nonce))
}

pub fn receive_message(core: &CCIHSCore, source_chain: ChainId) -> Result<CrossChainMessage, APIError> {
    core.receive_message(source_chain)
        .map_err(APIError::from)
}

pub fn verify_message(core: &CCIHSCore, message: CrossChainMessage) -> Result<bool, APIError> {
    core.verify_message(&message)
        .map_err(APIError::from)
}

pub fn convert_address(core: &CCIHSCore, from: ChainId, to: ChainId, address: Vec<u8>) -> Result<Vec<u8>, APIError> {
    core.convert_address(from, to, &address)
        .map_err(APIError::from)
}