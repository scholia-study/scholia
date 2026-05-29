use rand::RngCore;
use sha2::{Digest, Sha256};

/// Generate a random token and its SHA-256 hash.
/// Returns (raw_token, token_hash). Store the hash in DB, send the raw token to the user.
pub fn generate_token() -> (String, String) {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let raw_token = hex::encode(bytes);
    let hash = hash_token(&raw_token);
    (raw_token, hash)
}

/// Hash a raw token with SHA-256.
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
