use crate::utility::CCIHSError;
use super::message::CrossChainTransaction;

pub type CCIHSResult<T> = Result<T, CCIHSError>;

#[derive(Debug, Clone)]
pub enum CrossChainResult {
    Success(CrossChainTransaction),
    Failure(CCIHSError),
}