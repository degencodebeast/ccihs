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

pub fn update_relayer_fee_handler(
    ctx: Context<UpdateRelayerFee>,
    relayer_fee: u32,
    relayer_fee_precision: u32,
) -> Result<()> {
    require!(
        relayer_fee < relayer_fee_precision,
        WormholeError::InvalidRelayerFee,
    );
    let config = &mut ctx.accounts.config;
    config.relayer_fee = relayer_fee;
    config.relayer_fee_precision = relayer_fee_precision;

    // Done.
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateRelayerFee<'info> {
    #[account(mut)]
    /// CHECK: Owner of the program set in the [`RedeemerConfig`] account.
    pub owner: UncheckedAccount<'info>,

    #[account(
        mut,
        has_one = owner @ WormholeError::OwnerOnly,
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
