// src/utility/serialization/native.rs

use borsh::{BorshSerialize, BorshDeserialize};
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
use ethereum_types::Address as EthereumAddress;

impl BorshSerialize for ChainId {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        BorshSerialize::serialize(&self.0, writer)
    }
}

impl BorshDeserialize for ChainId {
    
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let id = u16::deserialize(buf)?;
        Ok(ChainId(id))
    }

    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let id = u16::deserialize_reader(reader)?;
        Ok(ChainId(id))
    }
}

// Implement BorshSerialize and BorshDeserialize for CrossChainAddress
impl BorshSerialize for CrossChainAddress {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            CrossChainAddress::Solana(pubkey) => {
                0u8.serialize(writer)?;
                pubkey.serialize(writer)
            },
            CrossChainAddress::Ethereum(addr) => {
                1u8.serialize(writer)?;
                addr.0.serialize(writer)
            },
        }
    }
}

impl BorshDeserialize for CrossChainAddress {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let variant = u8::deserialize(buf)?;
        match variant {
            0 => Ok(CrossChainAddress::Solana(Pubkey::deserialize(buf)?)),
            1 => {
                let mut eth_addr = [0u8; 20];
                eth_addr.copy_from_slice(&buf[..20]);
                *buf = &buf[20..];
                Ok(CrossChainAddress::Ethereum(EthereumAddress::from(eth_addr)))
            },
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid CrossChainAddress variant")),
        }
    }

    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let variant = u8::deserialize_reader(reader)?;
        match variant {
            0 => Ok(CrossChainAddress::Solana(Pubkey::deserialize_reader(reader)?)),
            1 => {
                let mut eth_addr = [0u8; 20];
                reader.read_exact(&mut eth_addr)?;
                Ok(CrossChainAddress::Ethereum(EthereumAddress::from(eth_addr)))
            },
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid CrossChainAddress variant")),
        }
    }
}


// Helper functions for serialization and deserialization
pub fn serialize<T: BorshSerialize>(value: &T) -> CCIHSResult<Vec<u8>> {
    let mut buffer = Vec::new();
    value.serialize(&mut buffer)
        .map_err(|e| CCIHSError::SerializationError(e.to_string()))?;
    Ok(buffer)
}

pub fn deserialize<T: BorshDeserialize>(buffer: &[u8]) -> CCIHSResult<T> {
    T::try_from_slice(buffer)
        .map_err(|e| CCIHSError::DeserializationError(e.to_string()))
}


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

        let eth_address = CrossChainAddress::Ethereum(EthereumAddress::from_low_u64_be(1));
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