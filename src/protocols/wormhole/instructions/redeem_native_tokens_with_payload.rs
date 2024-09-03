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

/// AKA `b"bridged"`.
pub const SEED_PREFIX_BRIDGED: &[u8; 7] = b"bridged";
/// AKA `b"tmp"`.
pub const SEED_PREFIX_TMP: &[u8; 3] = b"tmp";

/// AKA `b"general_message_config"`.
pub const SEED_PREFIX_GENERAL_MESSAGE_CONFIG: &[u8; 18] = b"general_message_config";


#[derive(Accounts)]
#[instruction(vaa_hash: [u8; 32])]
pub struct RedeemNativeTransferWithPayload<'info> {
    #[account(mut)]
    /// Payer will pay Wormhole fee to transfer tokens and create temporary
    /// token account.
    pub payer: Signer<'info>,

    #[account(
        mut,
        constraint = payer.key() == recipient.key() || payer_token_account.key() == anchor_spl::associated_token::get_associated_token_address(&payer.key(), &mint.key()) @ HelloTokenError::InvalidPayerAta
    )]
    /// CHECK: Payer's token account. If payer != recipient, must be an
    /// associated token account. Mutable.
    pub payer_token_account: UncheckedAccount<'info>,

    #[account(
        seeds = [RedeemerConfig::SEED_PREFIX],
        bump
    )]
    /// Redeemer Config account. Acts as the Token Bridge redeemer, which signs
    /// for the complete transfer instruction. Read-only.
    pub config: Box<Account<'info, RedeemerConfig>>,

    #[account(
        seeds = [
            ForeignContract::SEED_PREFIX,
            &vaa.emitter_chain().to_le_bytes()[..]
        ],
        bump,
        constraint = foreign_contract.verify(&vaa) @ HelloTokenError::InvalidForeignContract
    )]
    /// Foreign Contract account. The registered contract specified in this
    /// account must agree with the target address for the Token Bridge's token
    /// transfer. Read-only.
    pub foreign_contract: Box<Account<'info, ForeignContract>>,

    #[account(
        address = vaa.data().mint()
    )]
    /// Mint info. This is the SPL token that will be bridged over from the
    /// foreign contract. This must match the token address specified in the
    /// signed Wormhole message. Read-only.
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = recipient
    )]
    /// Recipient associated token account.
    pub recipient_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    /// CHECK: Recipient may differ from payer if a relayer paid for this
    /// transaction.
    pub recipient: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [
            SEED_PREFIX_TMP,
            mint.key().as_ref(),
        ],
        bump,
        token::mint = mint,
        token::authority = config
    )]
    /// Program's temporary token account. This account is created before the
    /// instruction is invoked to temporarily take custody of the payer's
    /// tokens. When the tokens are finally bridged in, the tokens will be
    /// transferred to the destination token accounts. This account will have
    /// zero balance and can be closed.
    pub tmp_token_account: Box<Account<'info, TokenAccount>>,

    /// Wormhole program.
    pub wormhole_program: Program<'info, wormhole::program::Wormhole>,

    /// Token Bridge program.
    pub token_bridge_program: Program<'info, token_bridge::program::TokenBridge>,

    #[account(
        address = config.token_bridge.config @ HelloTokenError::InvalidTokenBridgeConfig
    )]
    /// Token Bridge config. Read-only.
    pub token_bridge_config: Account<'info, token_bridge::Config>,

    #[account(
        seeds = [
            wormhole::SEED_PREFIX_POSTED_VAA,
            &vaa_hash
        ],
        bump,
        seeds::program = wormhole_program,
        constraint = vaa.data().to() == crate::ID || vaa.data().to() == config.key() @ HelloTokenError::InvalidTransferToAddress,
        constraint = vaa.data().to_chain() == wormhole::CHAIN_ID_SOLANA @ HelloTokenError::InvalidTransferToChain,
        constraint = vaa.data().token_chain() == wormhole::CHAIN_ID_SOLANA @ HelloTokenError::InvalidTransferTokenChain
    )]
    /// Verified Wormhole message account. The Wormhole program verified
    /// signatures and posted the account data here. Read-only.
    pub vaa: Box<Account<'info, PostedHelloTokenMessage>>,

    #[account(mut)]
    /// CHECK: Token Bridge claim account. It stores a boolean, whose value
    /// is true if the bridged assets have been claimed. If the transfer has
    /// not been redeemed, this account will not exist yet.
    pub token_bridge_claim: UncheckedAccount<'info>,

    #[account(
        address = foreign_contract.token_bridge_foreign_endpoint @ HelloTokenError::InvalidTokenBridgeForeignEndpoint
    )]
    /// Token Bridge foreign endpoint. This account should really be one
    /// endpoint per chain, but the PDA allows for multiple endpoints for each
    /// chain! We store the proper endpoint for the emitter chain.
    pub token_bridge_foreign_endpoint: Account<'info, token_bridge::EndpointRegistration>,

    #[account(
        mut,
        seeds = [mint.key().as_ref()],
        bump,
        seeds::program = token_bridge_program
    )]
    /// CHECK: Token Bridge custody. This is the Token Bridge program's token
    /// account that holds this mint's balance.
    pub token_bridge_custody: Account<'info, TokenAccount>,

    #[account(
        address = config.token_bridge.custody_signer @ HelloTokenError::InvalidTokenBridgeCustodySigner
    )]
    /// CHECK: Token Bridge custody signer. Read-only.
    pub token_bridge_custody_signer: UncheckedAccount<'info>,

    /// System program.
    pub system_program: Program<'info, System>,

    /// Token program.
    pub token_program: Program<'info, Token>,

    /// Associated Token program.
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Rent sysvar.
    pub rent: Sysvar<'info, Rent>,
}
