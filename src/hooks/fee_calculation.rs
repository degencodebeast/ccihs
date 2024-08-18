use super::Hook;
use crate::types::{CrossChainMessage, ChainId, CCIHSResult};
use crate::error::CCIHSError;

pub struct FeeCalculationHook {
    fee_percentage: f64,
}

impl FeeCalculationHook {
    pub fn new(fee_percentage: f64) -> Self {
        Self { fee_percentage }
    }
}

impl Hook for FeeCalculationHook {
    fn execute(&self, message: &mut CrossChainMessage, _source_chain: ChainId, _destination_chain: ChainId) -> CCIHSResult<()> {
        if let Some(amount) = message.amount {
            let fee = (amount as f64 * self.fee_percentage).round() as u64;
            if fee > amount {
                return Err(CCIHSError::InsufficientFunds);
            }
            message.amount = Some(amount - fee);
            message.fee = Some(fee);
            Ok(())
        } else {
            Err(CCIHSError::MissingAmount)
        }
    }
}

// pub struct FeeCalculationHook {
//     fee_percentage: f64,
// }

// impl Hook for FeeCalculationHook {
//     fn execute(&self, message: &mut CrossChainMessage, _source_chain: ChainId, _destination_chain: ChainId) -> CCIHSResult<()> {
//         let fee = (message.amount as f64 * self.fee_percentage).round() as u64;
//         message.amount -= fee;
//         log::info!("Applied fee of {} to message", fee);
//         Ok(())
//     }
// }