use anchor_lang::prelude::*;
use wormhole_anchor_sdk::wormhole;

#[account]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ForeignEmitter {
  /// Emitter chain. Cannot equal `1` (Solana's Chain ID).
  pub chain: u16,
  /// Emitter address. Cannot be zero address.
  pub address: [u8; 32],

  //bump
}

impl ForeignEmitter {
    pub const MAXIMUM_SIZE: usize = 8 // discriminator
    + 2 // chain
    + 32 // address
    ;
    /// AKA `b"foreign_emitter"`.
    pub const SEED_PREFIX: &'static [u8; 15] = b"foreign_emitter";

    /// Convenience method to check whether an address equals the one saved in
    /// this account.
    pub fn verify(&self, address: &[u8; 32]) -> bool {
        *address == self.address
    }
}



pub const MESSAGE_MAX_LENGTH: usize = 1024;

#[account]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
//#[derive(Default)]
/// Received account.
pub struct Received {
    /// AKA nonce. Should always be zero in this example, but we save it anyway.
    pub batch_id: u32,
    /// Keccak256 hash of verified Wormhole message.
    pub wormhole_message_hash: [u8; 32],
    /// HelloWorldMessage from [HelloWorldMessage::Hello](crate::message::HelloWorldMessage).
    pub message: Vec<u8>,
}

impl Received {
    pub const MAXIMUM_SIZE: usize = 8 // discriminator
        + 4 // batch_id
        + 32 // wormhole_message_hash
        + 4 // Vec length
        + MESSAGE_MAX_LENGTH // message
    ;
    /// AKA `b"received"`.
    pub const SEED_PREFIX: &'static [u8; 8] = b"received";
}



#[account]
#[derive(Default)]
/// Wormhole emitter account.
pub struct WormholeEmitter {
    /// PDA bump.
    pub bump: u8,
}

impl WormholeEmitter {
    pub const MAXIMUM_SIZE: usize = 8 // discriminator
    + 1 // bump
    ;
    /// AKA `b"emitter` (see
    /// [`SEED_PREFIX_EMITTER`](wormhole::SEED_PREFIX_EMITTER)).
    pub const SEED_PREFIX: &'static [u8; 7] = wormhole::SEED_PREFIX_EMITTER;
}

