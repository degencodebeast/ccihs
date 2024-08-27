// Chain-related constants
pub const SOLANA_CHAIN_ID: u16 = 1;
pub const ETHEREUM_CHAIN_ID: u16 = 2;

// Protocol-related constants
pub const WORMHOLE_PROTOCOL_ID: u8 = 1;
pub const LAYERZERO_PROTOCOL_ID: u8 = 2;

// Message-related constants
pub const MAX_PAYLOAD_SIZE: usize = 1024; // Example value, adjust as needed
pub const MAX_MESSAGE_AGE: u64 = 3600; // 1 hour in seconds

// Fee-related constants
pub const DEFAULT_FEE_AMOUNT: u64 = 1000; // Example value, adjust as needed

// State-related constants
pub const STATE_SEED_PREFIX: &[u8] = b"cross_chain_state";

// Timing constants
pub const DEFAULT_TIMEOUT: u64 = 300; // 5 minutes in seconds

// Error codes
pub const ERROR_INVALID_CHAIN: u32 = 1001;
pub const ERROR_INVALID_PROTOCOL: u32 = 1002;
pub const ERROR_MESSAGE_TOO_LARGE: u32 = 1003;
pub const ERROR_MESSAGE_EXPIRED: u32 = 1004;

// Feature flags
pub const ENABLE_RATE_LIMITING: bool = true;
pub const ENABLE_ENCRYPTION: bool = false; // Example, enable when implemented