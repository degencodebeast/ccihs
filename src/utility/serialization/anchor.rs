// src/utility/serialization/anchor.rs

use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;
use crate::types::{
    CrossChainMessage, 
    ChainId, 
    CrossChainAddress, 
    CCIHSResult,
    MessageStatus,
    CrossChainTransaction,
    CrossChainFee,
};
use crate::utility::error::CCIHSError;

// Implement AnchorSerialize and AnchorDeserialize for ChainId
impl AnchorSerialize for ChainId {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.0.serialize(writer)
    }
}

impl AnchorDeserialize for ChainId {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let id = u16::deserialize(buf)?;
        Ok(ChainId(id))
    }
}

// Implement AnchorSerialize and AnchorDeserialize for CrossChainAddress
impl AnchorSerialize for CrossChainAddress {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            CrossChainAddress::Solana(pubkey) => {
                0u8.serialize(writer)?;
                pubkey.serialize(writer)
            },
            CrossChainAddress::Ethereum(addr) => {
                1u8.serialize(writer)?;
                addr.serialize(writer)
            },
        }
    }
}

impl AnchorDeserialize for CrossChainAddress {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let variant = u8::deserialize(buf)?;
        match variant {
            0 => Ok(CrossChainAddress::Solana(Pubkey::deserialize(buf)?)),
            1 => {
                let mut eth_addr = [0u8; 20];
                eth_addr.copy_from_slice(&buf[..20]);
                *buf = &buf[20..];
                Ok(CrossChainAddress::Ethereum(eth_addr))
            },
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid CrossChainAddress variant")),
        }
    }
}

// // Derive AnchorSerialize and AnchorDeserialize for other types
// #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
// pub struct CrossChainMessage {
//     pub source_chain: ChainId,
//     pub destination_chain: ChainId,
//     pub sender: Pubkey,
//     pub recipient: Vec<u8>,
//     pub payload: Vec<u8>,
//     pub nonce: u64,
//     pub timestamp: i64,
// }

// #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
// pub enum MessageStatus {
//     Pending,
//     Sent,
//     Delivered,
//     Executed,
//     Failed,
// }

// #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
// pub struct CrossChainTransaction {
//     pub message: CrossChainMessage,
//     pub status: MessageStatus,
//     pub transaction_hash: Option<[u8; 32]>,
// }

// #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
// pub struct CrossChainFee {
//     pub amount: u64,
//     pub token: Option<Pubkey>,
// }

// Helper functions for serialization and deserialization
pub fn serialize<T: AnchorSerialize>(value: &T) -> CCIHSResult<Vec<u8>> {
    let mut buffer = Vec::new();
    value.serialize(&mut buffer)
        .map_err(|e| CCIHSError::SerializationError(e.to_string()))?;
    Ok(buffer)
}

pub fn deserialize<T: AnchorDeserialize>(buffer: &[u8]) -> CCIHSResult<T> {
    T::deserialize(&mut &buffer[..])
        .map_err(|e| CCIHSError::DeserializationError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id_serialization() {
        let chain_id = ChainId::SOLANA;
        let serialized = serialize(&chain_id).unwrap();
        let deserialized: ChainId = deserialize(&serialized).unwrap();
        assert_eq!(chain_id, deserialized);
    }

    #[test]
    fn test_cross_chain_address_serialization() {
        let solana_address = CrossChainAddress::Solana(Pubkey::new_unique());
        let serialized = serialize(&solana_address).unwrap();
        let deserialized: CrossChainAddress = deserialize(&serialized).unwrap();
        assert_eq!(solana_address, deserialized);

        let eth_address = CrossChainAddress::Ethereum([1; 20]);
        let serialized = serialize(&eth_address).unwrap();
        let deserialized: CrossChainAddress = deserialize(&serialized).unwrap();
        assert_eq!(eth_address, deserialized);
    }

    #[test]
    fn test_cross_chain_message_serialization() {
        let message = CrossChainMessage {
            source_chain: ChainId::SOLANA,
            destination_chain: ChainId::ETHEREUM,
            sender: Pubkey::new_unique(),
            recipient: vec![1, 2, 3, 4],
            payload: vec![5, 6, 7, 8],
            nonce: 1,
            timestamp: 12345,
        };
        let serialized = serialize(&message).unwrap();
        let deserialized: CrossChainMessage = deserialize(&serialized).unwrap();
        assert_eq!(message, deserialized);
    }

    #[test]
    fn test_message_status_serialization() {
        let status = MessageStatus::Delivered;
        let serialized = serialize(&status).unwrap();
        let deserialized: MessageStatus = deserialize(&serialized).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_cross_chain_transaction_serialization() {
        let transaction = CrossChainTransaction {
            message: CrossChainMessage {
                source_chain: ChainId::SOLANA,
                destination_chain: ChainId::ETHEREUM,
                sender: Pubkey::new_unique(),
                recipient: vec![1, 2, 3, 4],
                payload: vec![5, 6, 7, 8],
                nonce: 1,
                timestamp: 12345,
            },
            status: MessageStatus::Sent,
            transaction_hash: Some([1; 32]),
        };
        let serialized = serialize(&transaction).unwrap();
        let deserialized: CrossChainTransaction = deserialize(&serialized).unwrap();
        assert_eq!(transaction, deserialized);
    }

    #[test]
    fn test_cross_chain_fee_serialization() {
        let fee = CrossChainFee {
            amount: 1000,
            token: Some(Pubkey::new_unique()),
        };
        let serialized = serialize(&fee).unwrap();
        let deserialized: CrossChainFee = deserialize(&serialized).unwrap();
        assert_eq!(fee, deserialized);
    }
}