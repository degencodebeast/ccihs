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

//I could remove the state folder later tho
// The `state` folder in CCIHS is primarily used to define structures and utilities for managing cross-chain message state. Its main purposes are:

// 1. Define State Structures: It provides templates for state structures that developers can use to track cross-chain message information in their Solana programs.

// 2. Compatibility: It offers implementations compatible with both Anchor and native Solana development, allowing flexibility for different developer preferences.

// 3. State Management Utilities: It includes helper functions and methods for managing and updating the state of cross-chain messages.

// 4. PDA (Program Derived Address) Utilities: It provides functions to derive addresses for state accounts, which is crucial for Solana's account model.

// 5. Trait Definitions: It defines traits that ensure consistent state management across different implementations.

// The `state` folder doesn't maintain a global state for CCIHS itself (since CCIHS is a library, not an on-chain program). Instead, it provides tools and structures for developers using CCIHS to manage state in their own Solana programs that utilize cross-chain functionality.

// For example, a developer using CCIHS might use these state structures to:
// - Keep track of the last processed nonce for cross-chain messages
// - Count the number of messages sent or received
// - Store timestamps of the last message interactions

// This approach allows CCIHS to remain flexible and usable in various contexts while providing developers with the necessary tools to manage cross-chain state effectively in their Solana programs.
