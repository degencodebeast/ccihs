use anchor_lang::prelude::*;
use wormhole_anchor_sdk::{wormhole, token_bridge};
use crate::types::{CrossChainMessage, CCIHSResult};
use crate::utility::error::CCIHSError;
use crate::protocols::wormhole::state::{GeneralMessageConfig, WormholeEmitter};
use crate::protocols::wormhole::error::WormholeError;

 /// This instruction posts a Wormhole message of some arbitrary size
    /// in the form of bytes ([Vec<u8>]). The message is encoded as
    /// [HelloWorldMessage::Hello], which serializes a payload ID (1) before the message
    /// specified in the instruction. Instead of using the native borsh
    /// serialization of [Vec] length (little endian u32), length of the
    /// message is encoded as big endian u16 (in EVM, bytes for numerics are
    /// natively serialized as big endian).
    ///
    /// See [HelloWorldMessage] enum for serialization implementation.
    ///
    /// # Arguments
    ///
    /// * `message` - Arbitrary message to send out
    pub fn send_message_handler(ctx: Context<SendMessage>, message: Vec<u8>) -> Result<()> {
        // If Wormhole requires a fee before posting a message, we need to
        // transfer lamports to the fee collector. Otherwise
        // `wormhole::post_message` will fail.
        let fee = ctx.accounts.wormhole_bridge.fee();
        if fee > 0 {
            solana_program::program::invoke(
                &solana_program::system_instruction::transfer(
                    &ctx.accounts.payer.key(),
                    &ctx.accounts.wormhole_fee_collector.key(),
                    fee,
                ),
                &ctx.accounts.to_account_infos(),
            )?;
        }

        // Invoke `wormhole::post_message`.
        //
        // `wormhole::post_message` requires two signers: one for the emitter
        // and another for the wormhole message data. Both of these accounts
        // are owned by this program.
        //
        // There are two ways to handle the wormhole message data account:
        //   1. Using an extra keypair. You may to generate a keypair outside
        //      of this instruction and pass that keypair as an additional
        //      signer for the transaction. An integrator might use an extra
        //      keypair if the message can be "thrown away" (not easily
        //      retrievable without going back to this transaction hash to
        //      retrieve the message's pubkey).
        //   2. Generate a PDA. If we want some way to deserialize the message
        //      data written by the Wormhole program, we can use an account
        //      with an address derived by this program so we can use the PDA
        //      to access and deserialize the message data.
        //
        // In our example, we use method #2.
        let wormhole_emitter = &ctx.accounts.wormhole_emitter;
        let general_message_config = &ctx.accounts.general_message_config;

        // There is only one type of message that this example uses to
        // communicate with its foreign counterparts (payload ID == 1).
        let payload: Vec<u8> = CrossChainMessage::payload.try_to_vec()?;

        wormhole::post_message(
            CpiContext::new_with_signer(
                ctx.accounts.wormhole_program.to_account_info(),
                wormhole::PostMessage {
                    config: ctx.accounts.wormhole_bridge.to_account_info(),
                    message: ctx.accounts.wormhole_message.to_account_info(),
                    emitter: wormhole_emitter.to_account_info(),
                    sequence: ctx.accounts.wormhole_sequence.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    fee_collector: ctx.accounts.wormhole_fee_collector.to_account_info(),
                    clock: ctx.accounts.clock.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                },
                &[
                    &[
                        SEED_PREFIX_SENT,
                        &ctx.accounts.wormhole_sequence.next_value().to_le_bytes()[..],
                        &[ctx.bumps.wormhole_message],
                    ],
                    &[wormhole::SEED_PREFIX_EMITTER, &[wormhole_emitter.bump]],
                ],
            ),
            general_message_config.batch_id,
            payload,
            general_message_config.finality.try_into().unwrap(),
        )?;

        // Done.
        Ok(())
    }


/// AKA `b"sent"`.
pub const SEED_PREFIX_SENT: &[u8; 4] = b"sent";

#[derive(Accounts)]
pub struct SendMessage<'info> {
    #[account(mut)]
    /// Payer will pay Wormhole fee to post a message.
    pub payer: Signer<'info>,

    #[account(
        seeds = [GeneralMessageConfig::SEED_PREFIX],
        bump,
    )]
    /// Config account. Wormhole PDAs specified in the config are checked
    /// against the Wormhole accounts in this context. Read-only.
    pub general_message_config: Account<'info, GeneralMessageConfig>,

    /// Wormhole program.
    pub wormhole_program: Program<'info, wormhole::program::Wormhole>,

    #[account(
        mut,
        address = general_message_config.wormhole.bridge, //@ HelloWorldError::InvalidWormholeConfig
    )]
    /// Wormhole bridge data. [`wormhole::post_message`] requires this account
    /// be mutable.
    pub wormhole_bridge: Account<'info, wormhole::BridgeData>,

    #[account(
        mut,
        address = general_message_config.wormhole.fee_collector, //@ HelloWorldError::InvalidWormholeFeeCollector
    )]
    /// Wormhole fee collector. [`wormhole::post_message`] requires this
    /// account be mutable.
    pub wormhole_fee_collector: Account<'info, wormhole::FeeCollector>,

    #[account(
        seeds = [WormholeEmitter::SEED_PREFIX],
        bump,
    )]
    /// Program's emitter account. Read-only.
    pub wormhole_emitter: Account<'info, WormholeEmitter>,

    #[account(
        mut,
        address = general_message_config.wormhole.sequence, //@ HelloWorldError::InvalidWormholeSequence
    )]
    /// Emitter's sequence account. [`wormhole::post_message`] requires this
    /// account be mutable.
    pub wormhole_sequence: Account<'info, wormhole::SequenceTracker>,

    #[account(
        mut,
        seeds = [
            SEED_PREFIX_SENT,
            &wormhole_sequence.next_value().to_le_bytes()[..]
        ],
        bump,
    )]
    /// CHECK: Wormhole Message. [`wormhole::post_message`] requires this
    /// account be mutable.
    pub wormhole_message: UncheckedAccount<'info>,

    /// System program.
    pub system_program: Program<'info, System>,

    /// Clock sysvar.
    pub clock: Sysvar<'info, Clock>,

    /// Rent sysvar.
    pub rent: Sysvar<'info, Rent>,
}
