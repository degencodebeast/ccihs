use solana_program::pubkey::Pubkey;
use ethereum_types::Address as EthereumAddress;

#[derive(PartialEq, Debug)]
pub enum CrossChainAddress {
    Solana(Pubkey),
    Ethereum(EthereumAddress),
    // Add more as needed
}