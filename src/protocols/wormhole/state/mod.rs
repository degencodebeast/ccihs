mod foreign_emitter;
mod received;
mod wormhole_emitter;
mod general_message_config;
mod token_sender_config;
mod token_redeemer_config;
mod foreign_token_emitter;

pub use foreign_emitter::ForeignEmitter;
pub use received::{Received, MESSAGE_MAX_LENGTH};
pub use wormhole_emitter::WormholeEmitter;
pub use general_message_config::GeneralMessageConfig; 
pub use token_sender_config::SenderConfig;
pub use token_redeemer_config::RedeemerConfig;
pub use foreign_token_emitter::ForeignTContract;


// This approach provides a balance between convenience and control.
// We explicitly re-export the main structs and important constants,
// which gives us better control over the public API of this module.
// It reduces the risk of unintentionally exposing internal details
// or causing name conflicts, while still providing easy access to
// the most commonly used items.



