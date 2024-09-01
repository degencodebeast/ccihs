mod foreign_emitter;
mod received;
mod wormhole_emitter;

pub use foreign_emitter::ForeignEmitter;
pub use received::{Received, MESSAGE_MAX_LENGTH};
pub use wormhole_emitter::WormholeEmitter;

// This approach provides a balance between convenience and control.
// We explicitly re-export the main structs and important constants,
// which gives us better control over the public API of this module.
// It reduces the risk of unintentionally exposing internal details
// or causing name conflicts, while still providing easy access to
// the most commonly used items.



