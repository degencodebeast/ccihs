use solana_program::pubkey::Pubkey;

#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;

#[cfg(not(feature = "anchor"))]
use borsh::{BorshSerialize, BorshDeserialize};



#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
#[derive(Clone, Debug, PartialEq)]
//#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct CrossChainFee {
    pub amount: u64,
    pub token: Option<Pubkey>,  // None for native token, Some(Pubkey) for SPL tokens
}