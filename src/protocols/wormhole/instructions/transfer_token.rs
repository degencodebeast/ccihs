use anchor_lang::prelude::*;
use wormhole_anchor_sdk::{wormhole, token_bridge};
use crate::protocols::wormhole::config::WormholeConfig;
use crate::protocols::wormhole::state::{ForeignEmitter, Received};


#[derive(Accounts)]
pub struct TransferToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub config: Account<'info, WormholeConfig>,
    pub wormhole_program: Program<'info, Wormhole>,
    pub token_bridge_program: Program<'info, TokenBridge>,
    #[account(mut)]
    pub token_bridge_config: Account<'info, token_bridge::Config>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub custody: Account<'info, TokenAccount>,
    /// CHECK: Wormhole message
    #[account(mut)]
    pub wormhole_message: UncheckedAccount<'info>,
    pub wormhole_emitter: Account<'info, WormholeEmitter>,
    #[account(mut)]
    pub wormhole_sequence: Account<'info, wormhole::SequenceTracker>,
    #[account(mut)]
    pub wormhole_fee_collector: Account<'info, wormhole::FeeCollector>,
    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
