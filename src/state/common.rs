use solana_program::pubkey::Pubkey;

pub const STATE_SEED_PREFIX: &[u8] = b"cross_chain_state";

pub trait CrossChainMessageStateTrait {
    fn new() -> Self;
    fn update_with_message(&mut self, nonce: u64, timestamp: i64);
    fn last_nonce(&self) -> u64;
    fn message_count(&self) -> u64;
    fn last_message_timestamp(&self) -> i64;
}

pub fn derive_state_address(program_id: &Pubkey, sender: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[STATE_SEED_PREFIX, sender.as_ref()],
        program_id
    )
}