use anchor_lang::prelude::*;
use wormhole_anchor_sdk::{wormhole, token_bridge};
use crate::types::{CrossChainMessage, CCIHSResult, PostedCrossChainMessage};
use crate::utility::error::CCIHSError;
use crate::wormhole::GeneralMessageConfig;
use crate::wormhole::WormholeError;
use crate::protocols::wormhole::state::{ForeignEmitter, WormholeEmitter, Received, ForeignTokenEmitter, RedeemerConfig, SenderConfig};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

/// AKA `b"bridged"`.
pub const SEED_PREFIX_BRIDGED: &[u8; 7] = b"bridged";
/// AKA `b"tmp"`.
pub const SEED_PREFIX_TMP: &[u8; 3] = b"tmp";

/// AKA `b"general_message_config"`.
pub const SEED_PREFIX_GENERAL_MESSAGE_CONFIG: &[u8; 18] = b"general_message_config";

pub fn initialize_handler(ctx: Context<Initialize>, relayer_fee: u64, relayer_fee_precision: u32) -> Result<()> {
    
    require!(
        relayer_fee < relayer_fee_precision,
        WormholeError::InvalidRelayerFee,
    );

    let general_message_config = &mut ctx.accounts.general_message_config;

    // Set the owner of the config
    general_message_config.owner = ctx.accounts.owner.key();

    // Set Wormhole related addresses.
    {
        let wormhole = &mut general_message_config.wormhole;

        // wormhole::BridgeData (Wormhole's program data).
        wormhole.bridge = ctx.accounts.wormhole_bridge.key();

        // wormhole::FeeCollector (lamports collector for posting
        // messages).
        wormhole.fee_collector = ctx.accounts.wormhole_fee_collector.key();

        // wormhole::SequenceTracker (tracks # of messages posted by this
        // program).
        wormhole.sequence = ctx.accounts.wormhole_sequence.key();
    }

    // Set default values
    general_message_config.batch_id = 0;
    general_message_config.finality = wormhole::Finality::Confirmed as u8;

    // Initialize our Wormhole emitter account. It is not required by the
    // Wormhole program that there is an actual account associated with the
    // emitter PDA. The emitter PDA is just a mechanism to have the program
    // sign for the `wormhole::post_message` instruction.
    //
    // But for fun, we will store our emitter's bump for convenience.
    // Initialize Wormhole emitter account
    ctx.accounts.wormhole_emitter.bump = *ctx.bumps.get("wormhole_emitter").unwrap();

    // Initialize program's sender config
    let sender_config = &mut ctx.accounts.sender_config;

    // Set the owner of the sender config (effectively the owner of the
    // program).
    sender_config.owner = ctx.accounts.owner.key();
    sender_config.bump = ctx.bumps.sender_config;

      // Set Token Bridge related addresses.
      {
        let token_bridge = &mut sender_config.token_bridge;
        token_bridge.config = ctx.accounts.token_bridge_config.key();
        token_bridge.authority_signer = ctx.accounts.token_bridge_authority_signer.key();
        token_bridge.custody_signer = ctx.accounts.token_bridge_custody_signer.key();
        token_bridge.emitter = ctx.accounts.token_bridge_emitter.key();
        token_bridge.sequence = ctx.accounts.token_bridge_sequence.key();
        token_bridge.wormhole_bridge = ctx.accounts.wormhole_bridge.key();
        token_bridge.wormhole_fee_collector = ctx.accounts.wormhole_fee_collector.key();
    }

    // Initialize program's redeemer config
    let redeemer_config = &mut ctx.accounts.redeemer_config;

    // Set the owner of the redeemer config (effectively the owner of the
    // program).
    redeemer_config.owner = ctx.accounts.owner.key();
    redeemer_config.bump = ctx.bumps.redeemer_config;
    redeemer_config.relayer_fee = relayer_fee;
    redeemer_config.relayer_fee_precision = relayer_fee_precision;

    // Set Token Bridge related addresses.
    {
        let token_bridge = &mut redeemer_config.token_bridge;
        token_bridge.config = ctx.accounts.token_bridge_config.key();
        token_bridge.custody_signer = ctx.accounts.token_bridge_custody_signer.key();
        token_bridge.mint_authority = ctx.accounts.token_bridge_mint_authority.key();
    }

    // Post initial Wormhole message
    let fee = ctx.accounts.wormhole_bridge.fee();
    if fee > 0 {
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.owner.key(),
                &ctx.accounts.wormhole_fee_collector.key(),
                fee,
            ),
            &ctx.accounts.to_account_infos(),
        )?;
    }

    let wormhole_emitter = &ctx.accounts.wormhole_emitter;

    let payload = CrossChainMessage::Alive {
        program_id: *ctx.program_id,
    }.try_to_vec()?;

    wormhole::post_message(
        CpiContext::new_with_signer(
            ctx.accounts.wormhole_program.to_account_info(),
            wormhole::PostMessage {
                config: ctx.accounts.wormhole_bridge.to_account_info(),
                message: ctx.accounts.wormhole_message.to_account_info(),
                emitter: wormhole_emitter.to_account_info(),
                sequence: ctx.accounts.wormhole_sequence.to_account_info(),
                payer: ctx.accounts.owner.to_account_info(),
                fee_collector: ctx.accounts.wormhole_fee_collector.to_account_info(),
                clock: ctx.accounts.clock.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            },
            &[
                &[
                    SEED_PREFIX_SENT,
                    &wormhole::INITIAL_SEQUENCE.to_le_bytes()[..],
                    &[*ctx.bumps.get("wormhole_message").unwrap()],
                ],
                &[wormhole::SEED_PREFIX_EMITTER, &[wormhole_emitter.bump]],
            ],
        ),
        general_message_config.batch_id,
        payload,
        general_message_config.finality.try_into().unwrap(),
    )?;

    Ok(())
}

#[derive(Accounts)]
/// Context used to initialize program data (i.e. config).
pub struct Initialize<'info> {
    #[account(mut)]
    /// Whoever initializes the configs will be the owner of the program. Signer
    /// for creating the [`Config`] accounts and posting a Wormhole message and token transfers
    /// indicating that the program is alive.
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        seeds = [GeneralMessageConfig::SEED_PREFIX],
        bump,
        space = GeneralMessageConfig::MAXIMUM_SIZE,
    )]
    /// General message config account, which saves program data useful for other instructions.
    /// Also saves the payer of the [`initialize`](crate::initialize) instruction
    /// as the program's owner.
    pub general_message_config: Account<'info, GeneralMessageConfig>,

    #[account(
        init,
        payer = owner,
        seeds = [SenderConfig::SEED_PREFIX],
        bump,
        space = SenderConfig::MAXIMUM_SIZE,
    )]
    /// Sender Config account, which saves program data useful for other
    /// instructions, specifically for outbound transfers. Also saves the payer
    /// of the [`initialize`](crate::initialize) instruction as the program's
    /// owner.
    pub sender_config: Box<Account<'info, SenderConfig>>,

    #[account(
        init,
        payer = owner,
        seeds = [RedeemerConfig::SEED_PREFIX],
        bump,
        space = RedeemerConfig::MAXIMUM_SIZE,
    )]
    /// Redeemer Config account, which saves program data useful for other
    /// instructions, specifically for inbound transfers. Also saves the payer
    /// of the [`initialize`](crate::initialize) instruction as the program's
    /// owner.
    pub redeemer_config: Box<Account<'info, RedeemerConfig>>,

    /// Wormhole program.
    pub wormhole_program: Program<'info, wormhole::program::Wormhole>,

    /// Token Bridge program.
    pub token_bridge_program: Program<'info, token_bridge::program::TokenBridge>,

    // #[account(
    //     mut,
    //     seeds = [wormhole::BridgeData::SEED_PREFIX],
    //     bump,
    //     seeds::program = wormhole_program,
    // )]
    // /// Wormhole bridge data account (a.k.a. its config).
    // /// [`wormhole::post_message`] requires this account be mutable.
    // pub wormhole_bridge: Account<'info, wormhole::BridgeData>,

    #[account(
        seeds = [wormhole::BridgeData::SEED_PREFIX],
        bump,
        seeds::program = wormhole_program,
    )]
    /// Wormhole bridge data account (a.k.a. its config).
    pub wormhole_bridge: Box<Account<'info, wormhole::BridgeData>>,

    #[account(
        seeds = [token_bridge::Config::SEED_PREFIX],
        bump,
        seeds::program = token_bridge_program,
    )]
    /// Token Bridge config. Token Bridge program needs this account to
    /// invoke the Wormhole program to post messages. Even though it is a
    /// required account for redeeming token transfers, it is not actually
    /// used for completing these transfers.
    pub token_bridge_config: Account<'info, token_bridge::Config>,

    #[account(
        seeds = [token_bridge::SEED_PREFIX_AUTHORITY_SIGNER],
        bump,
        seeds::program = token_bridge_program,
    )]
    /// CHECK: Token Bridge authority signer. This isn't an account that holds
    /// data; it is purely just a signer for SPL tranfers when it is delegated
    /// spending approval for the SPL token.
    pub token_bridge_authority_signer: UncheckedAccount<'info>,

    #[account(
        seeds = [token_bridge::SEED_PREFIX_CUSTODY_SIGNER],
        bump,
        seeds::program = token_bridge_program,
    )]
    /// CHECK: Token Bridge custody signer. This isn't an account that holds
    /// data; it is purely just a signer for Token Bridge SPL tranfers.
    pub token_bridge_custody_signer: UncheckedAccount<'info>,


    #[account(
        seeds = [token_bridge::SEED_PREFIX_MINT_AUTHORITY],
        bump,
        seeds::program = token_bridge_program,
    )]
    /// CHECK: Token Bridge mint authority. This isn't an account that holds
    /// data; it is purely just a signer (SPL mint authority) for Token Bridge
    /// wrapped assets.
    pub token_bridge_mint_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [token_bridge::SEED_PREFIX_EMITTER],
        bump,
        seeds::program = token_bridge_program
    )]
    /// CHECK: Token Bridge program's emitter account. This isn't an account
    /// that holds data; it is purely just a signer for posting Wormhole
    /// messages on behalf of the Token Bridge program.
    pub token_bridge_emitter: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [wormhole::FeeCollector::SEED_PREFIX],
        bump,
        seeds::program = wormhole_program
    )]
    /// Wormhole fee collector account, which requires lamports before the
    /// program can post a message (if there is a fee).
    /// [`wormhole::post_message`] requires this account be mutable.
    pub wormhole_fee_collector: Account<'info, wormhole::FeeCollector>,

    #[account(
        init,
        payer = owner,
        seeds = [WormholeEmitter::SEED_PREFIX],
        bump,
        space = WormholeEmitter::MAXIMUM_SIZE
    )]
    /// This program's emitter account. We create this account in the
    /// [`initialize`](crate::initialize) instruction, but
    /// [`wormhole::post_message`] only needs it to be read-only.
    pub wormhole_emitter: Account<'info, WormholeEmitter>,

    #[account(
        mut,
        seeds = [
            wormhole::SequenceTracker::SEED_PREFIX,
            wormhole_emitter.key().as_ref()
        ],
        bump,
        seeds::program = wormhole_program
    )]
    /// CHECK: Emitter's sequence account. This is not created until the first
    /// message is posted, so it needs to be an [UncheckedAccount] for the
    /// [`initialize`](crate::initialize) instruction.
    /// [`wormhole::post_message`] requires this account be mutable.
    pub wormhole_sequence: UncheckedAccount<'info>,

    #[account(
        seeds = [
            wormhole::SequenceTracker::SEED_PREFIX,
            token_bridge_emitter.key().as_ref()
        ],
        bump,
        seeds::program = wormhole_program
    )]
    /// Token Bridge emitter's sequence account. Like with all Wormhole
    /// emitters, this account keeps track of the sequence number of the last
    /// posted message.
    pub token_bridge_sequence: Account<'info, wormhole::SequenceTracker>,

    #[account(
        mut,
        seeds = [
            SEED_PREFIX_SENT,
            &wormhole::INITIAL_SEQUENCE.to_le_bytes()[..]
        ],
        bump,
    )]
    /// CHECK: Wormhole message account. The Wormhole program writes to this
    /// account, which requires this program's signature.
    /// [`wormhole::post_message`] requires this account be mutable.
    pub wormhole_message: UncheckedAccount<'info>,

    /// Clock sysvar.
    pub clock: Sysvar<'info, Clock>,

    /// Rent sysvar.
    pub rent: Sysvar<'info, Rent>,

    /// System program.
    pub system_program: Program<'info, System>,
}