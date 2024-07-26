
//This could help in managing nonces for cross-chain messages to prevent replay attacks.
pub struct Nonce(pub u64);

impl Nonce {
    pub fn new() -> Self {
        // Implementation to generate a unique nonce
        // This is a placeholder; actual implementation would be more sophisticated
        Nonce(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64)
    }
}