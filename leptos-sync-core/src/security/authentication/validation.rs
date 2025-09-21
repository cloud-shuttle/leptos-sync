//! Password validation utilities

use super::config::AuthConfig;
use crate::SyncError;

/// Validate password strength
pub fn validate_password(password: &str, config: &AuthConfig) -> Result<(), SyncError> {
    if password.len() < config.password_min_length {
        return Err(SyncError::AuthenticationError(format!(
            "Password must be at least {} characters long",
            config.password_min_length
        )));
    }

    if config.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
        return Err(SyncError::AuthenticationError(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }

    if config.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
        return Err(SyncError::AuthenticationError(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }

    if config.require_numbers && !password.chars().any(|c| c.is_numeric()) {
        return Err(SyncError::AuthenticationError(
            "Password must contain at least one number".to_string(),
        ));
    }

    if config.require_special_chars
        && !password
            .chars()
            .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
    {
        return Err(SyncError::AuthenticationError(
            "Password must contain at least one special character".to_string(),
        ));
    }

    Ok(())
}
