use thiserror::Error;

#[derive(Error, Debug)]
pub enum WormholeError {
    #[error("Invalid emitter address")]
    InvalidEmitterAddress,

    #[error("Invalid sequence address")]
    InvalidSequenceAddress,

    #[error("Invalid VAA")]
    InvalidVAA,

    #[error("Wormhole error: {0}")]
    WormholeSDKError(String),

    #[error("Token bridge error: {0}")]
    TokenBridgeError(String),
}