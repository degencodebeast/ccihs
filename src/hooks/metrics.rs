use super::Hook;
use crate::types::{CrossChainMessage, ChainId, CCIHSResult};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MetricsHook {
    total_messages: AtomicUsize,
    total_bytes: AtomicUsize,
}

impl MetricsHook {
    pub fn new() -> Self {
        Self {
            total_messages: AtomicUsize::new(0),
            total_bytes: AtomicUsize::new(0),
        }
    }

    pub fn get_total_messages(&self) -> usize {
        self.total_messages.load(Ordering::Relaxed)
    }

    pub fn get_total_bytes(&self) -> usize {
        self.total_bytes.load(Ordering::Relaxed)
    }
}

impl Hook for MetricsHook {
    fn execute(&self, message: &mut CrossChainMessage, _source_chain: ChainId, _destination_chain: ChainId) -> CCIHSResult<()> {
        self.total_messages.fetch_add(1, Ordering::Relaxed);
        self.total_bytes.fetch_add(message.payload.len(), Ordering::Relaxed);
        Ok(())
    }
}

// use std::sync::atomic::{AtomicUsize, Ordering};

// pub struct MetricsHook {
//     total_messages: AtomicUsize,
//     total_bytes: AtomicUsize,
// }

// impl Hook for MetricsHook {
//     fn execute(&self, message: &CrossChainMessage, _source_chain: ChainId, _destination_chain: ChainId) -> CCIHSResult<()> {
//         self.total_messages.fetch_add(1, Ordering::SeqCst);
//         self.total_bytes.fetch_add(message.payload.len(), Ordering::SeqCst);
//         Ok(())
//     }
// }