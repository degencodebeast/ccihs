use anchor_lang::prelude::*;
use wormhole_anchor_sdk::{wormhole, token_bridge};
use crate::types::{CrossChainMessage, CCIHSResult};
use crate::utility::error::CCIHSError;
use crate::protocols::wormhole::state::general_message_config::GeneralMessageConfig;
use crate::protocols::wormhole::state::{ForeignEmitter, WormholeEmitter, Received};

#[derive(Accounts)]
#[instruction(chain: u16)]
pub struct RegisterEmitter<'info> {
    #[account(mut)]
    /// Owner of the program set in the [`Config`] account. Signer for creating
    /// the [`ForeignEmitter`] account.
    pub owner: Signer<'info>,

    #[account(
        has_one = owner, //@ HelloWorldError::OwnerOnly,
        seeds = [GeneralMessageConfig::SEED_PREFIX],
        bump
    )]
    /// Config account. This program requires that the `owner` specified in the
    /// context equals the pubkey specified in this account. Read-only.
    pub general_message_config: Account<'info, GeneralMessageConfig>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = [
            ForeignEmitter::SEED_PREFIX,
            &chain.to_le_bytes()[..]
        ],
        bump,
        space = ForeignEmitter::MAXIMUM_SIZE
    )]
    /// Foreign Emitter account. Create this account if an emitter has not been
    /// registered yet for this Wormhole chain ID. If there already is an
    /// emitter address saved in this account, overwrite it.
    pub foreign_emitter: Account<'info, ForeignEmitter>,

    /// System program.
    pub system_program: Program<'info, System>,
}

