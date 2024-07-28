pub mod operation;
pub mod chain_management;
pub mod error;

pub use operation::*;
pub use chain_management::*;
pub use error::CoreError;

// The core folder in CCIHS is meant to contain the central, fundamental logic of the library. Let's review its purpose and what might be needed to complete it:
// Purpose of the core folder:

// 1. Define the main operational logic of CCIHS.
// 2. Implement the primary functionalities that tie together other components.
// 3. Provide the central interface that other parts of the library will use.