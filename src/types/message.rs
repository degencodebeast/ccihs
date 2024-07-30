use super::{chain::ChainId, CCIHSResult};
use solana_program::pubkey::Pubkey;


#[cfg(not(feature = "anchor"))]
use borsh::{BorshSerialize, BorshDeserialize};

#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;


#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
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

#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
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
            timestamp: 0, // This should be set to current time
        }
    }

    pub fn validate(&self) -> CCIHSResult<()> {
        // Implement validation logic
        // e.g., check payload size, validate chains, etc.
        Ok(())
    }

    // Add other methods as needed
}
    

#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[derive(Clone, Debug, PartialEq)]
//#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct CrossChainTransaction {
    pub message: CrossChainMessage,
    pub status: MessageStatus,
    pub transaction_hash: Option<[u8; 32]>,
}