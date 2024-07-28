use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]

pub struct ChainId(pub u16);

impl ChainId {
    pub const SOLANA: ChainId = ChainId(1);
    pub const ETHEREUM: ChainId = ChainId(2);
    // Add more chain IDs as needed

    pub fn new(id: u16) -> Self {
        ChainId(id)
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChainId({})", self.0)
    }
}

