#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChainId(pub u16);

impl ChainId {
    pub const SOLANA: ChainId = ChainId(1);
    pub const ETHEREUM: ChainId = ChainId(2);
    // Add more chain IDs as needed
}