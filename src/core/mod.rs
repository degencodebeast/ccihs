// src/core/mod.rs

mod operation;
mod chain_management;
mod error;

pub use operation::CCIHSCore;
pub use chain_management::ChainManager;
pub use error::CoreError;

// The core folder in CCIHS is meant to contain the central, fundamental logic of the library
// Purpose of the core folder:

// 1. Define the main operational logic of CCIHS.
// 2. Implement the primary functionalities that tie together other components.
// 3. Provide the central interface that other parts of the library will use.