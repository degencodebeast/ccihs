use anchor_lang::prelude::*;
use wormhole_anchor_sdk::{wormhole, token_bridge};
use crate::types::CCIHSResult;
use crate::utility::error::CCIHSError;
use crate::wormhole::GeneralMessageConfig;
use crate::wormhole::WormholeError;
use crate::protocols::wormhole::state::{ForeignEmitter, WormholeEmitter, Received, ForeignTokenEmitter, RedeemerConfig, SenderConfig};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use crate::protocols::wormhole::CrossChainMessage;

#[derive(Accounts)]
pub struct UpdateRelayerFee<'info> {
    #[account(mut)]
    /// CHECK: Owner of the program set in the [`RedeemerConfig`] account.
    pub owner: UncheckedAccount<'info>,

    #[account(
        mut,
        has_one = owner @ HelloTokenError::OwnerOnly,
        seeds = [RedeemerConfig::SEED_PREFIX],
        bump
    )]
    /// Redeemer Config account. This program requires that the `owner`
    /// specified in the context equals the pubkey specified in this account.
    /// Mutable.
    pub config: Box<Account<'info, RedeemerConfig>>,

    /// System program.
    pub system_program: Program<'info, System>,
}
