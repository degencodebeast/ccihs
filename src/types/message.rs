use super::{chain::ChainId, CCIHSResult};
//use solana_program::pubkey::Pubkey;
use anchor_lang::{prelude::Pubkey, AnchorDeserialize, AnchorSerialize};
use std::io;
use wormhole_io::Readable;


#[cfg(feature = "native")]
use borsh::{BorshSerialize, BorshDeserialize};

#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;


// const PAYLOAD_ID_INITIALIZE: u8 = 0;
// const PAYLOAD_ID_MESSAGE: u8 = 1;

// pub const MAX_PAYLOAD_LENGTH: usize = 1024;

// #[derive(Clone, Debug, PartialEq)]
// pub enum CrossChainPayload {
//     Initialize { program_id: Pubkey },
//     Message { content: Vec<u8> },
// }

#[cfg_attr(feature = "native", derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[derive(Clone, Debug, PartialEq)]
//#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct CrossChainMessage {
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub sender: Pubkey,
    pub recipient: Vec<u8>,
    pub payload: Vec<u8>,
    pub nonce: u64,
    pub timestamp: u64,
}


#[cfg_attr(feature = "native", derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[derive(Clone, Debug, PartialEq)]
//#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum MessageStatus {
    Pending,
    Sent,
    Delivered,
    Executed,
    Failed,
}

impl CrossChainMessage {
    pub fn new(
        source_chain: ChainId,
        destination_chain: ChainId,
        sender: Pubkey,
        recipient: Vec<u8>,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            source_chain,
            destination_chain,
            sender,
            recipient,
            payload,
            nonce: 0, // This should be generated
            timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        }
    }

    
impl AnchorSerialize for CrossChainMessage {
    fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.source_chain.serialize(writer)?;
        self.destination_chain.serialize(writer)?;
        self.sender.serialize(writer)?;
        self.recipient.serialize(writer)?;
        
        if self.payload.len() > MAX_PAYLOAD_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("payload exceeds {MAX_PAYLOAD_LENGTH} bytes"),
            ));
        }
        self.payload.serialize(writer)?;
        
        self.nonce.serialize(writer)?;
        self.timestamp.serialize(writer)
    }
}

impl AnchorDeserialize for CrossChainMessage {
    fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
        let source_chain = ChainId::deserialize(buf)?;
        let destination_chain = ChainId::deserialize(buf)?;
        let sender = Pubkey::deserialize(buf)?;
        let recipient = Vec::<u8>::deserialize(buf)?;
        let payload = Vec::<u8>::deserialize(buf)?;
        
        if payload.len() > MAX_PAYLOAD_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("payload exceeds {MAX_PAYLOAD_LENGTH} bytes"),
            ));
        }
        
        let nonce = u64::deserialize(buf)?;
        let timestamp = u64::deserialize(buf)?;

        Ok(CrossChainMessage {
            source_chain,
            destination_chain,
            sender,
            recipient,
            payload,
            nonce,
            timestamp,
        })
    }
}


    pub fn validate(&self) -> CCIHSResult<()> {
        // Implement validation logic
        // e.g., check payload size, validate chains, etc.
        Ok(())
    }

    // Add other methods as needed
}
    
#[cfg_attr(feature = "native", derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[derive(Clone, Debug, PartialEq)]
//#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct CrossChainTransaction {
    pub message: CrossChainMessage,
    pub status: MessageStatus,
    pub transaction_hash: Option<[u8; 32]>,
}