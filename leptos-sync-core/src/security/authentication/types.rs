//! Authentication types and data structures

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Authentication provider types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuthProvider {
    Local,
    OAuth2,
    JWT,
    LDAP,
}

/// User session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub salt: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub is_verified: bool,
    pub mfa_enabled: bool,
    pub mfa_secret: Option<String>,
    pub failed_login_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
}

/// Password reset token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub token: String,
    pub user_id: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub used: bool,
}
