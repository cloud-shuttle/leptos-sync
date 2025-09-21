//! Cryptographic utilities for authentication

use crate::SyncError;
use base64::{Engine as _, engine::general_purpose};
use rand::{Rng, rngs::OsRng};
use sha2::{Digest, Sha256};

/// Hash password with salt
pub fn hash_password(password: &str) -> Result<(String, String), SyncError> {
    let salt = generate_salt();
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let hash = hasher.finalize();
    let hash_string = general_purpose::STANDARD.encode(hash);
    Ok((hash_string, salt))
}

/// Verify password
pub fn verify_password(password: &str, hash: &str, salt: &str) -> Result<bool, SyncError> {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let computed_hash = hasher.finalize();
    let computed_hash_string = general_purpose::STANDARD.encode(computed_hash);
    Ok(computed_hash_string == hash)
}

/// Generate salt
pub fn generate_salt() -> String {
    let mut rng = OsRng;
    let random_bytes: [u8; 16] = rng.r#gen();
    general_purpose::STANDARD.encode(random_bytes)
}

/// Generate session token
pub fn generate_session_token() -> String {
    let mut rng = OsRng;
    let random_bytes: [u8; 32] = rng.r#gen();
    general_purpose::STANDARD.encode(random_bytes)
}

/// Generate reset token
pub fn generate_reset_token() -> String {
    let mut rng = OsRng;
    let random_bytes: [u8; 32] = rng.r#gen();
    general_purpose::STANDARD.encode(random_bytes)
}

/// Generate MFA secret
pub fn generate_mfa_secret() -> String {
    let mut rng = OsRng;
    let random_bytes: [u8; 20] = rng.r#gen();
    general_purpose::STANDARD.encode(random_bytes)
}

/// Generate user ID
pub fn generate_user_id() -> String {
    let mut rng = OsRng;
    let random_bytes: [u8; 16] = rng.r#gen();
    format!("user_{}", general_purpose::STANDARD.encode(random_bytes))
}

/// Generate TOTP code (simplified implementation)
pub fn generate_totp_code(secret: &str) -> String {
    use chrono::Utc;
    // Simplified TOTP implementation - in production, use a proper TOTP library
    let timestamp = Utc::now().timestamp() / 30;
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(timestamp.to_be_bytes());
    let hash = hasher.finalize();
    let code = (hash[0] as u32 % 1000000) as u32;
    format!("{:06}", code)
}
