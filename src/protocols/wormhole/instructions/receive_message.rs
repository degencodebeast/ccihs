use anchor_lang::prelude::*;
use solana_program::entrypoint::HEAP_LENGTH;
use wormhole_anchor_sdk::{wormhole, token_bridge};
use crate::types::{CrossChainMessage, CCIHSResult};
use crate::utility::error::CCIHSError;
use crate::protocols::wormhole::state::{ForeignEmitter, Received};
use crate::wormhole::GeneralMessageConfig;
use crate::protocols::wormhole::state::MESSAGE_MAX_LENGTH;
use crate::protocols::wormhole::WormholeError;
use crate::MAX_PAYLOAD_SIZE;


    /// This instruction reads a posted verified Wormhole message and verifies
    /// that the payload is of type [HelloWorldMessage::Hello] (payload ID == 1). HelloWorldMessage
    /// data is stored in a [Received] account.
    ///
    /// See [HelloWorldMessage] enum for deserialization implementation.
    ///
    /// # Arguments
    ///
    /// * `vaa_hash` - Keccak256 hash of verified Wormhole message
    pub fn receive_message_handler(ctx: Context<ReceiveMessage>, vaa_hash: [u8; 32]) -> Result<()> {
        let posted_message = &ctx.accounts.posted;

        if let message = &posted_message.payload {
        // CrossChainMessage payload cannot be larger than the maximum size allowed.
        require!(
            message.len() <= MAX_PAYLOAD_SIZE,
            CCIHSError::PayloadTooLarge
        );

            // Save batch ID, keccak256 hash and message payload.
            let received = &mut ctx.accounts.received;
            received.batch_id = posted_message.batch_id();
            received.wormhole_message_hash = vaa_hash;
            received.message = message.clone();

            // Done
            Ok(())
        } else {
            Err(WormholeError::InvalidMessage.into())
        }
    }

#[derive(Accounts)]
#[instruction(vaa_hash: [u8; 32])]
pub struct ReceiveMessage<'info> {
    #[account(mut)]
    /// Payer will initialize an account that tracks his own message IDs.
    pub payer: Signer<'info>,

    #[account(
        seeds = [GeneralMessageConfig::SEED_PREFIX],
        bump,
    )]
    /// Config account. Wormhole PDAs specified in the config are checked
    /// against the Wormhole accounts in this context. Read-only.
    pub general_message_config: Account<'info, GeneralMessageConfig>,

    // Wormhole program.
    pub wormhole_program: Program<'info, wormhole::program::Wormhole>,

    #[account(
        seeds = [
            wormhole::SEED_PREFIX_POSTED_VAA,
            &vaa_hash
        ],
        bump,
        seeds::program = wormhole_program
    )]
    /// Verified Wormhole message account. The Wormhole program verified
    /// signatures and posted the account data here. Read-only.
    pub posted: Account<'info, wormhole::PostedVaa<CrossChainMessage>>,//you might need to check this line tho 
    //to make sure types are compatible, you need to make sure crossChainMessage aligns with the HelloWorldMessage example

    #[account(
        seeds = [
            ForeignEmitter::SEED_PREFIX,
            &posted.emitter_chain().to_le_bytes()[..]
        ],
        bump,
        constraint = foreign_emitter.verify(posted.emitter_address()), //@ HelloWorldError::InvalidForeignEmitter
    )]
    /// Foreign emitter account. The posted message's `emitter_address` must
    /// agree with the one we have registered for this message's `emitter_chain`
    /// (chain ID). Read-only.
    pub foreign_emitter: Account<'info, ForeignEmitter>,

    #[account(
        init,
        payer = payer,
        seeds = [
            Received::SEED_PREFIX,
            &posted.emitter_chain().to_le_bytes()[..],
            &posted.sequence().to_le_bytes()[..]
        ],
        bump,
        space = Received::MAXIMUM_SIZE
    )]
    /// Received account. [`receive_message`](crate::receive_message) will
    /// deserialize the Wormhole message's payload and save it to this account.
    /// This account cannot be overwritten, and will prevent Wormhole message
    /// replay with the same sequence.
    pub received: Account<'info, Received>,

    /// System program.
    pub system_program: Program<'info, System>,
}


