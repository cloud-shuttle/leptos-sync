//! Authentication system with comprehensive user management and security features

use crate::SyncError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use sha2::{Sha256, Digest};
use rand::{Rng, rngs::OsRng};
use base64::{Engine as _, engine::general_purpose};

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
    pub async fn register_user(&self, username: &str, password: &str, email: &str) -> Result<String, SyncError> {
        // Validate password strength
        self.validate_password(password)?;

        // Check if user already exists
        let users = self.users.read().await;
        if users.values().any(|u| u.username == username || u.email == email) {
            return Err(SyncError::AuthenticationError("User already exists".to_string()));
        }
        drop(users);

        // Generate user ID
        let user_id = self.generate_user_id();

        // Hash password
        let (password_hash, salt) = self.hash_password(password)?;

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
                return Err(SyncError::AuthenticationError("Account is locked".to_string()));
            }
        }

        // Check if user is active
        if !user.is_active {
            return Err(SyncError::AuthenticationError("Account is inactive".to_string()));
        }

        // Verify password
        if !self.verify_password(password, &user.password_hash, &user.salt)? {
            // Increment failed login attempts
            self.increment_failed_attempts(&user.id).await?;
            return Err(SyncError::AuthenticationError("Invalid credentials".to_string()));
        }

        // Check if MFA is enabled for user
        if user.mfa_enabled {
            return Err(SyncError::AuthenticationError("MFA required - use login_with_mfa method".to_string()));
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
    pub async fn login_with_mfa(&self, username: &str, password: &str, mfa_code: &str) -> Result<UserSession, SyncError> {
        // First authenticate with password
        let user = self.find_user_by_username(username).await?;

        // Check if user is locked
        if let Some(locked_until) = user.locked_until {
            if Utc::now() < locked_until {
                return Err(SyncError::AuthenticationError("Account is locked".to_string()));
            }
        }

        // Check if user is active
        if !user.is_active {
            return Err(SyncError::AuthenticationError("Account is inactive".to_string()));
        }

        // Verify password
        if !self.verify_password(password, &user.password_hash, &user.salt)? {
            self.increment_failed_attempts(&user.id).await?;
            return Err(SyncError::AuthenticationError("Invalid credentials".to_string()));
        }

        // Check if MFA is enabled
        if !user.mfa_enabled {
            return Err(SyncError::AuthenticationError("MFA not enabled for user".to_string()));
        }

        // Verify MFA code
        if !self.verify_mfa_code(&user.id, mfa_code).await? {
            self.increment_failed_attempts(&user.id).await?;
            return Err(SyncError::AuthenticationError("Invalid MFA code".to_string()));
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
        let token = self.generate_reset_token();
        let expires_at = Utc::now() + Duration::hours(1);

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
    pub async fn complete_password_reset(&self, token: &str, new_password: &str) -> Result<(), SyncError> {
        // Validate new password
        self.validate_password(new_password)?;

        // Find reset token
        let mut reset_tokens = self.reset_tokens.write().await;
        if let Some(reset_token) = reset_tokens.get_mut(token) {
            // Check if token is expired
            if Utc::now() > reset_token.expires_at {
                return Err(SyncError::AuthenticationError("Reset token has expired".to_string()));
            }

            // Check if token is already used
            if reset_token.used {
                return Err(SyncError::AuthenticationError("Reset token has already been used".to_string()));
            }

            // Mark token as used
            reset_token.used = true;

            // Update user password
            let (password_hash, salt) = self.hash_password(new_password)?;
            let mut users = self.users.write().await;
            if let Some(user) = users.get_mut(&reset_token.user_id) {
                user.password_hash = password_hash;
                user.salt = salt;
                user.failed_login_attempts = 0;
                user.locked_until = None;
            }

            Ok(())
        } else {
            Err(SyncError::AuthenticationError("Invalid reset token".to_string()))
        }
    }

    /// Enable MFA for user
    pub async fn enable_mfa(&self, user_id: &str) -> Result<(), SyncError> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(user_id) {
            user.mfa_enabled = true;
            user.mfa_secret = Some(self.generate_mfa_secret());
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
                let code = self.generate_totp_code(secret);
                Ok(code)
            } else {
                Err(SyncError::AuthenticationError("MFA secret not found".to_string()))
            }
        } else {
            Err(SyncError::AuthenticationError("User not found".to_string()))
        }
    }

    /// Find user by username
    async fn find_user_by_username(&self, username: &str) -> Result<User, SyncError> {
        let users = self.users.read().await;
        users.values()
            .find(|u| u.username == username)
            .cloned()
            .ok_or_else(|| SyncError::AuthenticationError("User not found".to_string()))
    }

    /// Hash password with salt
    fn hash_password(&self, password: &str) -> Result<(String, String), SyncError> {
        let salt = self.generate_salt();
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt.as_bytes());
        let hash = hasher.finalize();
        let hash_string = general_purpose::STANDARD.encode(hash);
        Ok((hash_string, salt))
    }

    /// Verify password
    fn verify_password(&self, password: &str, hash: &str, salt: &str) -> Result<bool, SyncError> {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt.as_bytes());
        let computed_hash = hasher.finalize();
        let computed_hash_string = general_purpose::STANDARD.encode(computed_hash);
        Ok(computed_hash_string == hash)
    }

    /// Validate password strength
    fn validate_password(&self, password: &str) -> Result<(), SyncError> {
        if password.len() < self.config.password_min_length {
            return Err(SyncError::AuthenticationError(format!(
                "Password must be at least {} characters long",
                self.config.password_min_length
            )));
        }

        if self.config.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(SyncError::AuthenticationError("Password must contain at least one uppercase letter".to_string()));
        }

        if self.config.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(SyncError::AuthenticationError("Password must contain at least one lowercase letter".to_string()));
        }

        if self.config.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(SyncError::AuthenticationError("Password must contain at least one number".to_string()));
        }

        if self.config.require_special_chars && !password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
            return Err(SyncError::AuthenticationError("Password must contain at least one special character".to_string()));
        }

        Ok(())
    }

    /// Generate user ID
    fn generate_user_id(&self) -> String {
        let mut rng = OsRng;
        let random_bytes: [u8; 16] = rng.gen();
        format!("user_{}", general_purpose::STANDARD.encode(random_bytes))
    }

    /// Generate salt
    fn generate_salt(&self) -> String {
        let mut rng = OsRng;
        let random_bytes: [u8; 16] = rng.gen();
        general_purpose::STANDARD.encode(random_bytes)
    }

    /// Generate session token
    fn generate_session_token(&self) -> String {
        let mut rng = OsRng;
        let random_bytes: [u8; 32] = rng.gen();
        general_purpose::STANDARD.encode(random_bytes)
    }

    /// Generate reset token
    fn generate_reset_token(&self) -> String {
        let mut rng = OsRng;
        let random_bytes: [u8; 32] = rng.gen();
        general_purpose::STANDARD.encode(random_bytes)
    }

    /// Generate MFA secret
    fn generate_mfa_secret(&self) -> String {
        let mut rng = OsRng;
        let random_bytes: [u8; 20] = rng.gen();
        general_purpose::STANDARD.encode(random_bytes)
    }

    /// Generate TOTP code (simplified implementation)
    fn generate_totp_code(&self, secret: &str) -> String {
        // Simplified TOTP implementation - in production, use a proper TOTP library
        let timestamp = Utc::now().timestamp() / 30;
        let mut hasher = Sha256::new();
        hasher.update(secret.as_bytes());
        hasher.update(timestamp.to_be_bytes());
        let hash = hasher.finalize();
        let code = (hash[0] as u32 % 1000000) as u32;
        format!("{:06}", code)
    }

    /// Create user session
    async fn create_session(&self, user_id: &str) -> Result<UserSession, SyncError> {
        let token = self.generate_session_token();
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
                let expected_code = self.generate_totp_code(secret);
                Ok(expected_code == code)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<User, SyncError> {
        let users = self.users.read().await;
        users.get(user_id)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_registration() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let password = "SecurePassword123!";
        let email = "test@example.com";
        
        let result = auth_manager.register_user(username, password, email).await;
        assert!(result.is_ok());
        
        let user_id = result.unwrap();
        assert!(!user_id.is_empty());
    }

    #[tokio::test]
    async fn test_user_login() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let password = "SecurePassword123!";
        let email = "test@example.com";
        
        // Register user
        let user_id = auth_manager.register_user(username, password, email).await.unwrap();
        
        // Login
        let session = auth_manager.login(username, password).await.unwrap();
        assert_eq!(session.user_id, user_id);
        assert!(!session.token.is_empty());
        assert!(session.expires_at > Utc::now());
    }

    #[tokio::test]
    async fn test_invalid_login() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let password = "SecurePassword123!";
        let wrong_password = "WrongPassword123!";
        
        // Register user
        auth_manager.register_user(username, password, "test@example.com").await.unwrap();
        
        // Try to login with wrong password
        let result = auth_manager.login(username, wrong_password).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_session_validation() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let password = "SecurePassword123!";
        
        // Register and login
        auth_manager.register_user(username, password, "test@example.com").await.unwrap();
        let session = auth_manager.login(username, password).await.unwrap();
        
        // Validate session
        let is_valid = auth_manager.validate_session(&session.token).await.unwrap();
        assert!(is_valid);
        
        // Logout
        auth_manager.logout(&session.token).await.unwrap();
        
        // Session should no longer be valid
        let is_valid_after_logout = auth_manager.validate_session(&session.token).await.unwrap();
        assert!(!is_valid_after_logout);
    }

    #[tokio::test]
    async fn test_password_reset() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let old_password = "OldPassword123!";
        let new_password = "NewPassword123!";
        
        // Register user
        auth_manager.register_user(username, old_password, "test@example.com").await.unwrap();
        
        // Initiate password reset
        let reset_token = auth_manager.initiate_password_reset(username).await.unwrap();
        assert!(!reset_token.is_empty());
        
        // Complete password reset
        auth_manager.complete_password_reset(&reset_token, new_password).await.unwrap();
        
        // Verify old password no longer works
        let old_login_result = auth_manager.login(username, old_password).await;
        assert!(old_login_result.is_err());
        
        // Verify new password works
        let new_login_result = auth_manager.login(username, new_password).await;
        assert!(new_login_result.is_ok());
    }

    #[tokio::test]
    async fn test_multi_factor_authentication() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let password = "SecurePassword123!";
        
        // Register user with MFA enabled
        let user_id = auth_manager.register_user(username, password, "test@example.com").await.unwrap();
        auth_manager.enable_mfa(&user_id).await.unwrap();
        
        // Login should require MFA
        let login_result = auth_manager.login(username, password).await;
        assert!(login_result.is_err()); // Should fail without MFA code
        
        // Generate MFA code
        let mfa_code = auth_manager.generate_mfa_code(&user_id).await.unwrap();
        
        // Login with MFA code
        let session = auth_manager.login_with_mfa(username, password, &mfa_code).await.unwrap();
        assert!(!session.token.is_empty());
    }

    #[tokio::test]
    async fn test_password_validation() {
        let auth_manager = AuthenticationManager::new();
        
        // Test weak password
        let weak_password = "123";
        let result = auth_manager.register_user("user1", weak_password, "test1@example.com").await;
        assert!(result.is_err());
        
        // Test password without uppercase
        let no_upper = "password123!";
        let result = auth_manager.register_user("user2", no_upper, "test2@example.com").await;
        assert!(result.is_err());
        
        // Test password without lowercase
        let no_lower = "PASSWORD123!";
        let result = auth_manager.register_user("user3", no_lower, "test3@example.com").await;
        assert!(result.is_err());
        
        // Test password without numbers
        let no_numbers = "Password!";
        let result = auth_manager.register_user("user4", no_numbers, "test4@example.com").await;
        assert!(result.is_err());
        
        // Test password without special characters
        let no_special = "Password123";
        let result = auth_manager.register_user("user5", no_special, "test5@example.com").await;
        assert!(result.is_err());
        
        // Test valid password
        let valid_password = "SecurePassword123!";
        let result = auth_manager.register_user("user6", valid_password, "test6@example.com").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_account_lockout() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let password = "SecurePassword123!";
        let wrong_password = "WrongPassword123!";
        
        // Register user
        auth_manager.register_user(username, password, "test@example.com").await.unwrap();
        
        // Try wrong password multiple times
        for _ in 0..5 {
            let _ = auth_manager.login(username, wrong_password).await;
        }
        
        // Account should be locked
        let result = auth_manager.login(username, password).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("locked"));
    }

    #[tokio::test]
    async fn test_duplicate_user_registration() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let password = "SecurePassword123!";
        let email = "test@example.com";
        
        // Register user first time
        let result1 = auth_manager.register_user(username, password, email).await;
        assert!(result1.is_ok());
        
        // Try to register same user again
        let result2 = auth_manager.register_user(username, password, email).await;
        assert!(result2.is_err());
        assert!(result2.unwrap_err().to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn test_session_cleanup() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let password = "SecurePassword123!";
        
        // Register and login
        auth_manager.register_user(username, password, "test@example.com").await.unwrap();
        let session = auth_manager.login(username, password).await.unwrap();
        
        // Verify session exists
        let is_valid = auth_manager.validate_session(&session.token).await.unwrap();
        assert!(is_valid);
        
        // Clean up expired sessions (should not remove our session)
        let cleaned = auth_manager.cleanup_expired_sessions().await;
        assert_eq!(cleaned, 0);
        
        // Verify session still exists
        let is_valid_after_cleanup = auth_manager.validate_session(&session.token).await.unwrap();
        assert!(is_valid_after_cleanup);
    }
}
