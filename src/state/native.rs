use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CrossChainMessageState {
    pub last_nonce: u64,
    pub message_count: u64,
    pub last_message_timestamp: i64,
}

impl crate::state::common::CrossChainMessageStateTrait for CrossChainMessageState {
    fn new() -> Self {
        Self {
            last_nonce: 0,
            message_count: 0,
            last_message_timestamp: 0,
        }
    }

    fn update_with_message(&mut self, nonce: u64, timestamp: i64) {
        if nonce > self.last_nonce {
            self.last_nonce = nonce;
        }
        self.message_count += 1;
        self.last_message_timestamp = timestamp;
    }

    fn last_nonce(&self) -> u64 { self.last_nonce }
    fn message_count(&self) -> u64 { self.message_count }
    fn last_message_timestamp(&self) -> i64 { self.last_message_timestamp }
}