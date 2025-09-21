//! Authentication system with comprehensive user management and security features

pub mod config;
pub mod crypto;
pub mod manager;
pub mod types;
pub mod validation;

// Re-export public types
pub use config::AuthConfig;
pub use manager::AuthenticationManager;
pub use types::{AuthProvider, PasswordResetToken, User, UserSession};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

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
        let user_id = auth_manager
            .register_user(username, password, email)
            .await
            .unwrap();

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
        auth_manager
            .register_user(username, password, "test@example.com")
            .await
            .unwrap();

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
        auth_manager
            .register_user(username, password, "test@example.com")
            .await
            .unwrap();
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
        auth_manager
            .register_user(username, old_password, "test@example.com")
            .await
            .unwrap();

        // Initiate password reset
        let reset_token = auth_manager
            .initiate_password_reset(username)
            .await
            .unwrap();
        assert!(!reset_token.is_empty());

        // Complete password reset
        auth_manager
            .complete_password_reset(&reset_token, new_password)
            .await
            .unwrap();

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
        let user_id = auth_manager
            .register_user(username, password, "test@example.com")
            .await
            .unwrap();
        auth_manager.enable_mfa(&user_id).await.unwrap();

        // Login should require MFA
        let login_result = auth_manager.login(username, password).await;
        assert!(login_result.is_err()); // Should fail without MFA code

        // Generate MFA code
        let mfa_code = auth_manager.generate_mfa_code(&user_id).await.unwrap();

        // Login with MFA code
        let session = auth_manager
            .login_with_mfa(username, password, &mfa_code)
            .await
            .unwrap();
        assert!(!session.token.is_empty());
    }

    #[tokio::test]
    async fn test_password_validation() {
        let auth_manager = AuthenticationManager::new();

        // Test weak password
        let weak_password = "123";
        let result = auth_manager
            .register_user("user1", weak_password, "test1@example.com")
            .await;
        assert!(result.is_err());

        // Test password without uppercase
        let no_upper = "password123!";
        let result = auth_manager
            .register_user("user2", no_upper, "test2@example.com")
            .await;
        assert!(result.is_err());

        // Test password without lowercase
        let no_lower = "PASSWORD123!";
        let result = auth_manager
            .register_user("user3", no_lower, "test3@example.com")
            .await;
        assert!(result.is_err());

        // Test password without numbers
        let no_numbers = "Password!";
        let result = auth_manager
            .register_user("user4", no_numbers, "test4@example.com")
            .await;
        assert!(result.is_err());

        // Test password without special characters
        let no_special = "Password123";
        let result = auth_manager
            .register_user("user5", no_special, "test5@example.com")
            .await;
        assert!(result.is_err());

        // Test valid password
        let valid_password = "SecurePassword123!";
        let result = auth_manager
            .register_user("user6", valid_password, "test6@example.com")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_account_lockout() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let password = "SecurePassword123!";
        let wrong_password = "WrongPassword123!";

        // Register user
        auth_manager
            .register_user(username, password, "test@example.com")
            .await
            .unwrap();

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
        auth_manager
            .register_user(username, password, "test@example.com")
            .await
            .unwrap();
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
