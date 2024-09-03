use super::{chain::ChainId, CCIHSResult};
//use solana_program::pubkey::Pubkey;
use anchor_lang::{prelude::Pubkey, AnchorDeserialize, AnchorSerialize};
use std::io;
use wormhole_io::Readable;
use wormhole_anchor_sdk::token_bridge;

#[cfg(feature = "native")]
use borsh::{BorshSerialize, BorshDeserialize};

#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;

//I need to come back to this payload ID

const PAYLOAD_ID_INITIALIZE: u8 = 0;
const PAYLOAD_ID_MESSAGE: u8 = 1;

pub const MAX_PAYLOAD_LENGTH: usize = 1024;

#[derive(Clone, Debug, PartialEq)]
/// Expected message types for this program. Only valid payloads are:
/// * `Initialize`: Payload ID == 0. Emitted when [`initialize`](crate::initialize)
///  is called).
/// * `Message`: Payload ID == 1. Emitted when
/// [`send_message`](crate::send_message) is called).
///
/// Payload IDs are encoded as u8.
pub enum CrossChainPayload {
    Initialize { program_id: Pubkey },
    Message { content: Vec<u8> },
}

#[cfg_attr(feature = "native", derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[derive(Clone, Debug, PartialEq)]
//#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct CrossChainMessage {
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub amount: u64,
    pub token_address: Option<Pubkey>,
    pub sender: CrossChainAddress,
    pub recipient: CrossChainAddress,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub nonce: u32,
    pub timestamp: u64,
    pub consistency_level: u8,
}

pub enum MessageType {
    General,
    TokenTransfer,
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
        message_type: MessageType,
        amount: u64,
        token_address: Option<Pubkey>,
        consistency_level: u8,
    ) -> Self {
        Self {
            message_type,
            payload,
            amount,
            token_address,
            sender,
            recipient,
            source_chain,
            destination_chain,
            nonce: 0, // This should be generated
            timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
            consistency_level,
        }
    }

    
// impl AnchorSerialize for CrossChainMessage {
//     fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
//         match self {
//             CrossChainMessage::Initialize { program_id } => {
//                 PAYLOAD_ID_INITIALIZE.serialize(writer)?;
//                 program_id.serialize(writer)
//             }
//             CrossChainMessage::Message { message } => {
//                 if message.len() > MAX_PAYLOAD_LENGTH {
//                     Err(io::Error::new(
//                         io::ErrorKind::InvalidInput,
//                         format!("message exceeds {MAX_PAYLOAD_LENGTH} bytes"),
//                     ))
//                 } else {
//                     PAYLOAD_ID_MESSAGE.serialize(writer)?;
//                     (message.len() as u16).to_be_bytes().serialize(writer)?;
//                     for item in message {
//                         item.serialize(writer)?;
//                     }
//                     Ok(())
//                 }
//             }
//         }
//     }
// }

// impl AnchorDeserialize for CrossChainMessage {
//     fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
//         match u8::read(reader)? {
//             PAYLOAD_ID_ALIVE => Ok(CrossChainMessage::Initialize {
//                 program_id: Pubkey::try_from(<[u8; 32]>::read(reader)?).unwrap(),
//             }),
//             PAYLOAD_ID_MESSAGE => {
//                 let length = u16::read(reader)? as usize;
//                 if length > MAX_PAYLOAD_LENGTH {
//                     Err(io::Error::new(
//                         io::ErrorKind::InvalidInput,
//                         format!("message exceeds {MAX_PAYLOAD_LENGTH} bytes"),
//                     ))
//                 } else {
//                     let mut buf = vec![0; length];
//                     reader.read_exact(&mut buf)?;
//                     Ok(CrossChainMessage::Message { message: buf })
//                 }
//             }
//             _ => Err(io::Error::new(
//                 io::ErrorKind::InvalidInput,
//                 "invalid payload ID",
//             )),
//         }
//     }
// }

    impl AnchorSerialize for CrossChainMessage {
        fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
            // Serialize message type
            match self.message_type {
                MessageType::General => 0u8.serialize(writer)?,
                MessageType::TokenTransfer => 1u8.serialize(writer)?,
            }

            // Serialize payload
            if self.payload.len() > MAX_PAYLOAD_LENGTH {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("payload exceeds {MAX_PAYLOAD_LENGTH} bytes"),
                ));
            }
            (self.payload.len() as u16).to_be_bytes().serialize(writer)?;
            writer.write_all(&self.payload)?;

            // Serialize other fields
            self.amount.serialize(writer)?;
            match &self.token_address {
                Some(addr) => {
                    1u8.serialize(writer)?; // Indicator for Some
                    addr.serialize(writer)?;
                }
                None => 0u8.serialize(writer)?, // Indicator for None
            }
            self.sender.serialize(writer)?;
            self.recipient.serialize(writer)?;
            self.source_chain.serialize(writer)?;
            self.destination_chain.serialize(writer)?;
            self.nonce.serialize(writer)?;
            self.timestamp.serialize(writer)?;
            self.consistency_level.serialize(writer)
        }
    }

    impl AnchorDeserialize for CrossChainMessage {
        fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
            let message_type = match u8::deserialize(buf)? {
                0 => MessageType::General,
                1 => MessageType::TokenTransfer,
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid message type")),
            };

            let payload_len = u16::deserialize(buf)? as usize;
            if payload_len > MAX_PAYLOAD_LENGTH {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("payload exceeds {MAX_PAYLOAD_LENGTH} bytes"),
                ));
            }
            let mut payload = vec![0u8; payload_len];
            buf.read_exact(&mut payload)?;

            let amount = u64::deserialize(buf)?;
            let token_address = match u8::deserialize(buf)? {
                0 => None,
                1 => Some(Pubkey::deserialize(buf)?),
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid token address indicator")),
            };

            Ok(CrossChainMessage {
                message_type,
                payload,
                amount,
                token_address,
                sender: CrossChainAddress::deserialize(buf)?,
                recipient: CrossChainAddress::deserialize(buf)?,
                source_chain: ChainId::deserialize(buf)?,
                destination_chain: ChainId::deserialize(buf)?,
                nonce: u32::deserialize(buf)?,
                timestamp: u64::deserialize(buf)?,
                consistency_level: u8::deserialize(buf)?,
            })
        }
    }
}


pub type PostedCrossChainMessage = token_bridge::PostedTransferWith<CrossChainMessage>;

#[cfg_attr(feature = "native", derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[derive(Clone, Debug, PartialEq)]
//#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct CrossChainTransaction {
    pub message: CrossChainMessage,
    pub status: MessageStatus,
    pub transaction_hash: Option<[u8; 32]>,
}