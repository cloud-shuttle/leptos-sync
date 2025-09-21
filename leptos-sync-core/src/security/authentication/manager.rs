//! Authentication manager implementation

use super::{
    config::AuthConfig,
    crypto::{
        generate_mfa_secret, generate_reset_token, generate_session_token, generate_totp_code,
        generate_user_id, hash_password, verify_password,
    },
    types::{PasswordResetToken, User, UserSession},
    validation::validate_password,
};
use crate::SyncError;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Authentication manager for handling user authentication and sessions
pub struct AuthenticationManager {
    config: AuthConfig,
    users: RwLock<HashMap<String, User>>,
    sessions: RwLock<HashMap<String, UserSession>>,
    reset_tokens: RwLock<HashMap<String, PasswordResetToken>>,
}

impl AuthenticationManager {
    /// Create a new authentication manager
    pub fn new() -> Self {
        Self {
            config: AuthConfig::default(),
            users: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
            reset_tokens: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new authentication manager with custom configuration
    pub fn with_config(config: AuthConfig) -> Self {
        Self {
            config,
            users: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
            reset_tokens: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new user
    pub async fn register_user(
        &self,
        username: &str,
        password: &str,
        email: &str,
    ) -> Result<String, SyncError> {
        // Validate password strength
        validate_password(password, &self.config)?;

        // Check if user already exists
        let users = self.users.read().await;
        if users
            .values()
            .any(|u| u.username == username || u.email == email)
        {
            return Err(SyncError::AuthenticationError(
                "User already exists".to_string(),
            ));
        }
        drop(users);

        // Generate user ID
        let user_id = generate_user_id();

        // Hash password
        let (password_hash, salt) = hash_password(password)?;

        // Create user
        let user = User {
            id: user_id.clone(),
            username: username.to_string(),
            email: email.to_string(),
            password_hash,
            salt,
            created_at: Utc::now(),
            last_login: None,
            is_active: true,
            is_verified: false,
            mfa_enabled: false,
            mfa_secret: None,
            failed_login_attempts: 0,
            locked_until: None,
        };

        // Store user
        let mut users = self.users.write().await;
        users.insert(user_id.clone(), user);

        Ok(user_id)
    }

    /// Authenticate user with username and password
    pub async fn login(&self, username: &str, password: &str) -> Result<UserSession, SyncError> {
        // Find user
        let user = self.find_user_by_username(username).await?;

        // Check if user is locked
        if let Some(locked_until) = user.locked_until {
            if Utc::now() < locked_until {
                return Err(SyncError::AuthenticationError(
                    "Account is locked".to_string(),
                ));
            }
        }

        // Check if user is active
        if !user.is_active {
            return Err(SyncError::AuthenticationError(
                "Account is inactive".to_string(),
            ));
        }

        // Verify password
        if !verify_password(password, &user.password_hash, &user.salt)? {
            // Increment failed login attempts
            self.increment_failed_attempts(&user.id).await?;
            return Err(SyncError::AuthenticationError(
                "Invalid credentials".to_string(),
            ));
        }

        // Check if MFA is enabled for user
        if user.mfa_enabled {
            return Err(SyncError::AuthenticationError(
                "MFA required - use login_with_mfa method".to_string(),
            ));
        }

        // Reset failed login attempts on successful login
        self.reset_failed_attempts(&user.id).await?;

        // Update last login
        self.update_last_login(&user.id).await?;

        // Create session
        let session = self.create_session(&user.id).await?;

        Ok(session)
    }

    /// Authenticate user with MFA
    pub async fn login_with_mfa(
        &self,
        username: &str,
        password: &str,
        mfa_code: &str,
    ) -> Result<UserSession, SyncError> {
        // First authenticate with password
        let user = self.find_user_by_username(username).await?;

        // Check if user is locked
        if let Some(locked_until) = user.locked_until {
            if Utc::now() < locked_until {
                return Err(SyncError::AuthenticationError(
                    "Account is locked".to_string(),
                ));
            }
        }

        // Check if user is active
        if !user.is_active {
            return Err(SyncError::AuthenticationError(
                "Account is inactive".to_string(),
            ));
        }

        // Verify password
        if !verify_password(password, &user.password_hash, &user.salt)? {
            self.increment_failed_attempts(&user.id).await?;
            return Err(SyncError::AuthenticationError(
                "Invalid credentials".to_string(),
            ));
        }

        // Check if MFA is enabled
        if !user.mfa_enabled {
            return Err(SyncError::AuthenticationError(
                "MFA not enabled for user".to_string(),
            ));
        }

        // Verify MFA code
        if !self.verify_mfa_code(&user.id, mfa_code).await? {
            self.increment_failed_attempts(&user.id).await?;
            return Err(SyncError::AuthenticationError(
                "Invalid MFA code".to_string(),
            ));
        }

        // Reset failed login attempts
        self.reset_failed_attempts(&user.id).await?;

        // Update last login
        self.update_last_login(&user.id).await?;

        // Create session
        let session = self.create_session(&user.id).await?;

        Ok(session)
    }

    /// Validate user session
    pub async fn validate_session(&self, token: &str) -> Result<bool, SyncError> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(token) {
            // Check if session is expired
            if Utc::now() > session.expires_at {
                drop(sessions);
                self.logout(token).await?;
                return Ok(false);
            }

            // Update last activity
            drop(sessions);
            self.update_session_activity(token).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Logout user
    pub async fn logout(&self, token: &str) -> Result<(), SyncError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(token);
        Ok(())
    }

    /// Initiate password reset
    pub async fn initiate_password_reset(&self, username: &str) -> Result<String, SyncError> {
        let user = self.find_user_by_username(username).await?;

        // Generate reset token
        let token = generate_reset_token();
        let expires_at = Utc::now() + chrono::Duration::hours(1);

        let reset_token = PasswordResetToken {
            token: token.clone(),
            user_id: user.id.clone(),
            expires_at,
            created_at: Utc::now(),
            used: false,
        };

        // Store reset token
        let mut reset_tokens = self.reset_tokens.write().await;
        reset_tokens.insert(token.clone(), reset_token);

        Ok(token)
    }

    /// Complete password reset
    pub async fn complete_password_reset(
        &self,
        token: &str,
        new_password: &str,
    ) -> Result<(), SyncError> {
        // Validate new password
        validate_password(new_password, &self.config)?;

        // Find reset token
        let mut reset_tokens = self.reset_tokens.write().await;
        if let Some(reset_token) = reset_tokens.get_mut(token) {
            // Check if token is expired
            if Utc::now() > reset_token.expires_at {
                return Err(SyncError::AuthenticationError(
                    "Reset token has expired".to_string(),
                ));
            }

            // Check if token is already used
            if reset_token.used {
                return Err(SyncError::AuthenticationError(
                    "Reset token has already been used".to_string(),
                ));
            }

            // Mark token as used
            reset_token.used = true;

            // Update user password
            let (password_hash, salt) = hash_password(new_password)?;
            let mut users = self.users.write().await;
            if let Some(user) = users.get_mut(&reset_token.user_id) {
                user.password_hash = password_hash;
                user.salt = salt;
                user.failed_login_attempts = 0;
                user.locked_until = None;
            }

            Ok(())
        } else {
            Err(SyncError::AuthenticationError(
                "Invalid reset token".to_string(),
            ))
        }
    }

    /// Enable MFA for user
    pub async fn enable_mfa(&self, user_id: &str) -> Result<(), SyncError> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(user_id) {
            user.mfa_enabled = true;
            user.mfa_secret = Some(generate_mfa_secret());
        } else {
            return Err(SyncError::AuthenticationError("User not found".to_string()));
        }
        Ok(())
    }

    /// Generate MFA code for user
    pub async fn generate_mfa_code(&self, user_id: &str) -> Result<String, SyncError> {
        let users = self.users.read().await;
        if let Some(user) = users.get(user_id) {
            if let Some(secret) = &user.mfa_secret {
                // Generate TOTP code (simplified implementation)
                let code = generate_totp_code(secret);
                Ok(code)
            } else {
                Err(SyncError::AuthenticationError(
                    "MFA secret not found".to_string(),
                ))
            }
        } else {
            Err(SyncError::AuthenticationError("User not found".to_string()))
        }
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<User, SyncError> {
        let users = self.users.read().await;
        users
            .get(user_id)
            .cloned()
            .ok_or_else(|| SyncError::AuthenticationError("User not found".to_string()))
    }

    /// List all users
    pub async fn list_users(&self) -> Vec<User> {
        let users = self.users.read().await;
        users.values().cloned().collect()
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let now = Utc::now();
        let expired_tokens: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| session.expires_at < now)
            .map(|(token, _)| token.clone())
            .collect();

        for token in &expired_tokens {
            sessions.remove(token);
        }

        expired_tokens.len()
    }

    /// Clean up expired reset tokens
    pub async fn cleanup_expired_reset_tokens(&self) -> usize {
        let mut reset_tokens = self.reset_tokens.write().await;
        let now = Utc::now();
        let expired_tokens: Vec<String> = reset_tokens
            .iter()
            .filter(|(_, token)| token.expires_at < now)
            .map(|(token, _)| token.clone())
            .collect();

        for token in &expired_tokens {
            reset_tokens.remove(token);
        }

        expired_tokens.len()
    }

    // Private helper methods

    /// Find user by username
    async fn find_user_by_username(&self, username: &str) -> Result<User, SyncError> {
        let users = self.users.read().await;
        users
            .values()
            .find(|u| u.username == username)
            .cloned()
            .ok_or_else(|| SyncError::AuthenticationError("User not found".to_string()))
    }

    /// Create user session
    async fn create_session(&self, user_id: &str) -> Result<UserSession, SyncError> {
        let token = generate_session_token();
        let now = Utc::now();
        let expires_at = now + self.config.session_timeout;

        let session = UserSession {
            user_id: user_id.to_string(),
            token: token.clone(),
            expires_at,
            created_at: now,
            last_activity: now,
            ip_address: None,
            user_agent: None,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(token, session.clone());

        Ok(session)
    }

    /// Update session activity
    async fn update_session_activity(&self, token: &str) -> Result<(), SyncError> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(token) {
            session.last_activity = Utc::now();
        }
        Ok(())
    }

    /// Increment failed login attempts
    async fn increment_failed_attempts(&self, user_id: &str) -> Result<(), SyncError> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(user_id) {
            user.failed_login_attempts += 1;
            if user.failed_login_attempts >= self.config.max_failed_attempts {
                user.locked_until = Some(Utc::now() + self.config.lockout_duration);
            }
        }
        Ok(())
    }

    /// Reset failed login attempts
    async fn reset_failed_attempts(&self, user_id: &str) -> Result<(), SyncError> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(user_id) {
            user.failed_login_attempts = 0;
            user.locked_until = None;
        }
        Ok(())
    }

    /// Update last login
    async fn update_last_login(&self, user_id: &str) -> Result<(), SyncError> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(user_id) {
            user.last_login = Some(Utc::now());
        }
        Ok(())
    }

    /// Verify MFA code
    async fn verify_mfa_code(&self, user_id: &str, code: &str) -> Result<bool, SyncError> {
        let users = self.users.read().await;
        if let Some(user) = users.get(user_id) {
            if let Some(secret) = &user.mfa_secret {
                let expected_code = generate_totp_code(secret);
                Ok(expected_code == code)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
}
