//! Authentication configuration

use chrono::Duration;
use serde::{Deserialize, Serialize};

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub session_timeout: Duration,
    pub max_failed_attempts: u32,
    pub lockout_duration: Duration,
    pub password_min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub mfa_required: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            session_timeout: Duration::hours(24),
            max_failed_attempts: 5,
            lockout_duration: Duration::minutes(30),
            password_min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            mfa_required: false,
        }
    }
}
