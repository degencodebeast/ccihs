use super::Hook;
use crate::types::{CrossChainMessage, ChainId, CCIHSResult};
use crate::CCIHSError;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use parking_lot::Mutex;

pub struct RateLimitingHook {
    max_messages: usize,
    time_window: Duration,
    message_times: Mutex<VecDeque<Instant>>,
}

impl RateLimitingHook {
    pub fn new(max_messages: usize, time_window: Duration) -> Self {
        Self {
            max_messages,
            time_window,
            message_times: Mutex::new(VecDeque::new()),
        }
    }
}

impl Hook for RateLimitingHook {
    fn execute(&self, _message: &mut CrossChainMessage, _source_chain: ChainId, _destination_chain: ChainId) -> CCIHSResult<()> {
        let now = Instant::now();
        let mut message_times = self.message_times.lock();

        message_times.push_back(now);

        if message_times.len() > self.max_messages {
            let oldest = message_times.pop_front().unwrap();
            if now.duration_since(oldest) < self.time_window {
                return Err(CCIHSError::RateLimitExceeded);
            }
        }

        Ok(())
    }
}


// use std::collections::VecDeque;
// use std::time::{Duration, Instant};

// pub struct RateLimitingHook {
//     max_messages: usize,
//     time_window: Duration,
//     message_times: VecDeque<Instant>,
// }

// impl Hook for RateLimitingHook {
//     fn execute(&self, _message: &CrossChainMessage, _source_chain: ChainId, _destination_chain: ChainId) -> CCIHSResult<()> {
//         let now = Instant::now();
//         self.message_times.push_back(now);

//         if self.message_times.len() > self.max_messages {
//             let oldest = self.message_times.pop_front().unwrap();
//             if now.duration_since(oldest) < self.time_window {
//                 return Err(CCIHSError::RateLimitExceeded);
//             }
//         }

//         Ok(())
//     }
// }