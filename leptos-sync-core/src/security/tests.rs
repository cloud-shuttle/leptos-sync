#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{
        encryption::{EncryptionManager, EncryptionAlgorithm, KeyManager},
        authentication::{AuthenticationManager, AuthProvider, UserSession},
        gdpr::{GDPRCompliance, DataSubject, DataProcessingPurpose},
    };
    use std::collections::HashMap;
    use tokio::test;

    // ============================================================================
    // ENCRYPTION TESTS
    // ============================================================================

    #[test]
    async fn test_aes_encryption_decryption() {
        let key_manager = KeyManager::new();
        let encryption_manager = EncryptionManager::new(EncryptionAlgorithm::Aes256);
        
        let plaintext = b"Hello, World! This is a test message.";
        let key = key_manager.generate_key(EncryptionAlgorithm::Aes256).await.unwrap();
        
        // Encrypt
        let encrypted = encryption_manager.encrypt(plaintext, &key).await.unwrap();
        assert_ne!(encrypted, plaintext);
        
        // Decrypt
        let decrypted = encryption_manager.decrypt(&encrypted, &key).await.unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    async fn test_rsa_encryption_decryption() {
        let key_manager = KeyManager::new();
        let encryption_manager = EncryptionManager::new(EncryptionAlgorithm::Rsa2048);
        
        let plaintext = b"Hello, World!";
        let (public_key, private_key) = key_manager.generate_rsa_keypair().await.unwrap();
        
        // Encrypt with public key
        let encrypted = encryption_manager.encrypt(plaintext, &public_key).await.unwrap();
        assert_ne!(encrypted, plaintext);
        
        // Decrypt with private key
        let decrypted = encryption_manager.decrypt(&encrypted, &private_key).await.unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    async fn test_key_rotation() {
        let key_manager = KeyManager::new();
        let encryption_manager = EncryptionManager::new(EncryptionAlgorithm::Aes256);
        
        let plaintext = b"Test data for key rotation";
        
        // Generate initial key
        let key1 = key_manager.generate_key(EncryptionAlgorithm::Aes256).await.unwrap();
        let encrypted1 = encryption_manager.encrypt(plaintext, &key1).await.unwrap();
        
        // Rotate key
        let key2 = key_manager.rotate_key(&key1).await.unwrap();
        let encrypted2 = encryption_manager.encrypt(plaintext, &key2).await.unwrap();
        
        // Verify both keys work
        let decrypted1 = encryption_manager.decrypt(&encrypted1, &key1).await.unwrap();
        let decrypted2 = encryption_manager.decrypt(&encrypted2, &key2).await.unwrap();
        
        assert_eq!(decrypted1, plaintext);
        assert_eq!(decrypted2, plaintext);
        assert_ne!(encrypted1, encrypted2);
    }

    #[test]
    async fn test_encryption_with_wrong_key() {
        let key_manager = KeyManager::new();
        let encryption_manager = EncryptionManager::new(EncryptionAlgorithm::Aes256);
        
        let plaintext = b"Test message";
        let key1 = key_manager.generate_key(EncryptionAlgorithm::Aes256).await.unwrap();
        let key2 = key_manager.generate_key(EncryptionAlgorithm::Aes256).await.unwrap();
        
        let encrypted = encryption_manager.encrypt(plaintext, &key1).await.unwrap();
        
        // Try to decrypt with wrong key
        let result = encryption_manager.decrypt(&encrypted, &key2).await;
        assert!(result.is_err());
    }

    #[test]
    async fn test_key_derivation() {
        let key_manager = KeyManager::new();
        let password = "test_password_123";
        let salt = b"test_salt";
        
        let derived_key = key_manager.derive_key_from_password(password, salt).await.unwrap();
        assert!(!derived_key.is_empty());
        
        // Same password and salt should produce same key
        let derived_key2 = key_manager.derive_key_from_password(password, salt).await.unwrap();
        assert_eq!(derived_key, derived_key2);
        
        // Different salt should produce different key
        let different_salt = b"different_salt";
        let derived_key3 = key_manager.derive_key_from_password(password, different_salt).await.unwrap();
        assert_ne!(derived_key, derived_key3);
    }

    // ============================================================================
    // AUTHENTICATION TESTS
    // ============================================================================

    #[test]
    async fn test_user_registration() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let password = "secure_password_123";
        let email = "test@example.com";
        
        let result = auth_manager.register_user(username, password, email).await;
        assert!(result.is_ok());
        
        let user_id = result.unwrap();
        assert!(!user_id.is_empty());
    }

    #[test]
    async fn test_user_login() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let password = "secure_password_123";
        let email = "test@example.com";
        
        // Register user
        let user_id = auth_manager.register_user(username, password, email).await.unwrap();
        
        // Login
        let session = auth_manager.login(username, password).await.unwrap();
        assert_eq!(session.user_id, user_id);
        assert!(!session.token.is_empty());
        assert!(session.expires_at > chrono::Utc::now());
    }

    #[test]
    async fn test_invalid_login() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let password = "secure_password_123";
        let wrong_password = "wrong_password";
        
        // Register user
        auth_manager.register_user(username, password, "test@example.com").await.unwrap();
        
        // Try to login with wrong password
        let result = auth_manager.login(username, wrong_password).await;
        assert!(result.is_err());
    }

    #[test]
    async fn test_session_validation() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let password = "secure_password_123";
        
        // Register and login
        auth_manager.register_user(username, password, "test@example.com").await.unwrap();
        let session = auth_manager.login(username, password).await.unwrap();
        
        // Validate session
        let is_valid = auth_manager.validate_session(&session.token).await.unwrap();
        assert!(is_valid);
        
        // Invalidate session
        auth_manager.logout(&session.token).await.unwrap();
        
        // Session should no longer be valid
        let is_valid_after_logout = auth_manager.validate_session(&session.token).await.unwrap();
        assert!(!is_valid_after_logout);
    }

    #[test]
    async fn test_password_reset() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let old_password = "old_password";
        let new_password = "new_secure_password";
        
        // Register user
        auth_manager.register_user(username, old_password, "test@example.com").await.unwrap();
        
        // Reset password
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

    #[test]
    async fn test_multi_factor_authentication() {
        let auth_manager = AuthenticationManager::new();
        let username = "test_user";
        let password = "secure_password_123";
        
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

    // ============================================================================
    // GDPR COMPLIANCE TESTS
    // ============================================================================

    #[test]
    async fn test_data_subject_registration() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: chrono::Utc::now(),
        };
        
        let result = gdpr.register_data_subject(subject.clone()).await;
        assert!(result.is_ok());
        
        // Verify subject is registered
        let retrieved = gdpr.get_data_subject("user_123").await.unwrap();
        assert_eq!(retrieved.email, subject.email);
    }

    #[test]
    async fn test_consent_management() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: chrono::Utc::now(),
        };
        
        gdpr.register_data_subject(subject).await.unwrap();
        
        // Withdraw consent
        gdpr.withdraw_consent("user_123").await.unwrap();
        
        // Verify consent is withdrawn
        let subject = gdpr.get_data_subject("user_123").await.unwrap();
        assert!(!subject.consent_given);
    }

    #[test]
    async fn test_right_to_be_forgotten() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: chrono::Utc::now(),
        };
        
        gdpr.register_data_subject(subject).await.unwrap();
        
        // Store some data
        let data = "sensitive_user_data".as_bytes();
        gdpr.store_personal_data("user_123", data, DataProcessingPurpose::ServiceProvision).await.unwrap();
        
        // Exercise right to be forgotten
        gdpr.delete_all_personal_data("user_123").await.unwrap();
        
        // Verify data is deleted
        let retrieved_data = gdpr.get_personal_data("user_123").await;
        assert!(retrieved_data.is_err());
        
        // Verify subject is deleted
        let retrieved_subject = gdpr.get_data_subject("user_123").await;
        assert!(retrieved_subject.is_err());
    }

    #[test]
    async fn test_data_portability() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: chrono::Utc::now(),
        };
        
        gdpr.register_data_subject(subject).await.unwrap();
        
        // Store some data
        let data1 = "data1".as_bytes();
        let data2 = "data2".as_bytes();
        gdpr.store_personal_data("user_123", data1, DataProcessingPurpose::ServiceProvision).await.unwrap();
        gdpr.store_personal_data("user_123", data2, DataProcessingPurpose::Analytics).await.unwrap();
        
        // Export all data
        let exported_data = gdpr.export_all_personal_data("user_123").await.unwrap();
        assert!(exported_data.contains(&data1.to_vec()));
        assert!(exported_data.contains(&data2.to_vec()));
    }

    #[test]
    async fn test_data_processing_purposes() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: chrono::Utc::now(),
        };
        
        gdpr.register_data_subject(subject).await.unwrap();
        
        // Store data for different purposes
        let service_data = "service_data".as_bytes();
        let analytics_data = "analytics_data".as_bytes();
        
        gdpr.store_personal_data("user_123", service_data, DataProcessingPurpose::ServiceProvision).await.unwrap();
        gdpr.store_personal_data("user_123", analytics_data, DataProcessingPurpose::Analytics).await.unwrap();
        
        // Withdraw consent for analytics only
        gdpr.withdraw_consent_for_purpose("user_123", DataProcessingPurpose::Analytics).await.unwrap();
        
        // Verify analytics data is deleted but service data remains
        let all_data = gdpr.export_all_personal_data("user_123").await.unwrap();
        assert!(all_data.contains(&service_data.to_vec()));
        assert!(!all_data.contains(&analytics_data.to_vec()));
    }

    #[test]
    async fn test_data_retention_policy() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: chrono::Utc::now(),
        };
        
        gdpr.register_data_subject(subject).await.unwrap();
        
        // Store data with retention period
        let data = "temporary_data".as_bytes();
        gdpr.store_personal_data_with_retention("user_123", data, DataProcessingPurpose::ServiceProvision, chrono::Duration::seconds(1)).await.unwrap();
        
        // Wait for retention period to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Trigger retention cleanup
        gdpr.cleanup_expired_data().await.unwrap();
        
        // Verify data is deleted
        let retrieved_data = gdpr.get_personal_data("user_123").await;
        assert!(retrieved_data.is_err());
    }

    #[test]
    async fn test_audit_logging() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: chrono::Utc::now(),
        };
        
        gdpr.register_data_subject(subject).await.unwrap();
        
        // Perform various operations
        gdpr.store_personal_data("user_123", b"test_data", DataProcessingPurpose::ServiceProvision).await.unwrap();
        gdpr.withdraw_consent("user_123").await.unwrap();
        gdpr.delete_all_personal_data("user_123").await.unwrap();
        
        // Get audit log
        let audit_log = gdpr.get_audit_log("user_123").await.unwrap();
        assert!(audit_log.len() >= 3); // At least 3 operations logged
        
        // Verify log entries contain required information
        for entry in audit_log {
            assert!(!entry.timestamp.is_empty());
            assert!(!entry.operation.is_empty());
            assert!(!entry.user_id.is_empty());
        }
    }

    // ============================================================================
    // INTEGRATION TESTS
    // ============================================================================

    #[test]
    async fn test_end_to_end_encrypted_authentication() {
        let key_manager = KeyManager::new();
        let encryption_manager = EncryptionManager::new(EncryptionAlgorithm::Aes256);
        let auth_manager = AuthenticationManager::new();
        
        let username = "test_user";
        let password = "secure_password_123";
        let email = "test@example.com";
        
        // Register user
        let user_id = auth_manager.register_user(username, password, email).await.unwrap();
        
        // Generate encryption key for user
        let user_key = key_manager.generate_key(EncryptionAlgorithm::Aes256).await.unwrap();
        
        // Login
        let session = auth_manager.login(username, password).await.unwrap();
        
        // Encrypt sensitive data
        let sensitive_data = b"user_sensitive_information";
        let encrypted_data = encryption_manager.encrypt(sensitive_data, &user_key).await.unwrap();
        
        // Decrypt data
        let decrypted_data = encryption_manager.decrypt(&encrypted_data, &user_key).await.unwrap();
        assert_eq!(decrypted_data, sensitive_data);
    }

    #[test]
    async fn test_gdpr_compliant_data_encryption() {
        let gdpr = GDPRCompliance::new();
        let key_manager = KeyManager::new();
        let encryption_manager = EncryptionManager::new(EncryptionAlgorithm::Aes256);
        
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: chrono::Utc::now(),
        };
        
        gdpr.register_data_subject(subject).await.unwrap();
        
        // Generate encryption key
        let key = key_manager.generate_key(EncryptionAlgorithm::Aes256).await.unwrap();
        
        // Encrypt and store personal data
        let personal_data = b"encrypted_personal_data";
        let encrypted_data = encryption_manager.encrypt(personal_data, &key).await.unwrap();
        
        gdpr.store_personal_data("user_123", &encrypted_data, DataProcessingPurpose::ServiceProvision).await.unwrap();
        
        // Retrieve and decrypt data
        let retrieved_encrypted = gdpr.get_personal_data("user_123").await.unwrap();
        let decrypted_data = encryption_manager.decrypt(&retrieved_encrypted, &key).await.unwrap();
        
        assert_eq!(decrypted_data, personal_data);
    }
}
