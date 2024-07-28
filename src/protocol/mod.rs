use crate::types::{CrossChainMessage, ChainId, CCIHSResult};

pub trait ProtocolAdapter {
    fn send_message(&self, message: &CrossChainMessage) -> CCIHSResult<()>;
    fn receive_message(&self, source_chain: ChainId) -> CCIHSResult<CrossChainMessage>;
    fn verify_message(&self, message: &CrossChainMessage) -> CCIHSResult<bool>;
    fn supported_chains(&self) -> Vec<ChainId>;
}

mod wormhole;
pub use wormhole::WormholeAdapter;