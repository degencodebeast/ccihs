use solana_program::pubkey::Pubkey;

pub struct CrossChainFee {
    pub amount: u64,
    pub token: Option<Pubkey>,  // None for native token, Some(Pubkey) for SPL tokens
}