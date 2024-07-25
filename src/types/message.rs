use super::chain::ChainId;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, PartialEq)]
pub struct CrossChainMessage {
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub sender: Pubkey,
    pub recipient: Vec<u8>,
    pub payload: Vec<u8>,
    pub nonce: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageStatus {
    Pending,
    Sent,
    Delivered,
    Executed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct CrossChainTransaction {
    pub message: CrossChainMessage,
    pub status: MessageStatus,
    pub transaction_hash: Option<[u8; 32]>,
}