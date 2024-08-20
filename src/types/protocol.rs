use crate::types::ChainId;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ProtocolType {
    Wormhole,
    LayerZero,
    // Add more as needed
}