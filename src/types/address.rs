use solana_program::pubkey::Pubkey;


pub enum CrossChainAddress {
    Solana(Pubkey),
    Ethereum([u8; 20]),
    // Add more as needed
}