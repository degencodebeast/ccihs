// src/utility/serialization/mod.rs

#[cfg(feature = "anchor")]
mod anchor;
#[cfg(feature = "anchor")]
pub use self::anchor::*;

#[cfg(not(feature = "anchor"))]
mod native;
#[cfg(not(feature = "anchor"))]
pub use self::native::*;




// Common types and traits can be defined here if needed