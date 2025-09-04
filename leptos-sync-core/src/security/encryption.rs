//! Encryption functionality with comprehensive algorithm support

use crate::SyncError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use rand::{Rng, rngs::OsRng};
use base64::{Engine as _, engine::general_purpose};

/// Encryption algorithms supported by the system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256,
    Aes128,
}

/// Encryption key with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKey {
    pub id: String,
    pub algorithm: EncryptionAlgorithm,
    pub key_data: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub version: u32,
}

/// Encryption manager for handling various encryption algorithms
pub struct EncryptionManager {
    algorithm: EncryptionAlgorithm,
}

impl EncryptionManager {
    /// Create a new encryption manager
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        Self { algorithm }
    }

    /// Encrypt data with the specified key
    pub async fn encrypt(&self, data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, SyncError> {
        if key.algorithm != self.algorithm {
            return Err(SyncError::EncryptionError(format!(
                "Key algorithm {:?} does not match manager algorithm {:?}",
                key.algorithm, self.algorithm
            )));
        }

        // Check if key is expired
        if let Some(expires_at) = key.expires_at {
            if Utc::now() > expires_at {
                return Err(SyncError::EncryptionError("Key has expired".to_string()));
            }
        }

        match self.algorithm {
            EncryptionAlgorithm::Aes256 => self.encrypt_aes256(data, &key.key_data).await,
            EncryptionAlgorithm::Aes128 => self.encrypt_aes128(data, &key.key_data).await,
        }
    }

    /// Decrypt data with the specified key
    pub async fn decrypt(&self, encrypted_data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, SyncError> {
        if key.algorithm != self.algorithm {
            return Err(SyncError::EncryptionError(format!(
                "Key algorithm {:?} does not match manager algorithm {:?}",
                key.algorithm, self.algorithm
            )));
        }

        // Check if key is expired
        if let Some(expires_at) = key.expires_at {
            if Utc::now() > expires_at {
                return Err(SyncError::EncryptionError("Key has expired".to_string()));
            }
        }

        match self.algorithm {
            EncryptionAlgorithm::Aes256 => self.decrypt_aes256(encrypted_data, &key.key_data).await,
            EncryptionAlgorithm::Aes128 => self.decrypt_aes128(encrypted_data, &key.key_data).await,
        }
    }

    /// Encrypt data using AES-256-GCM
    async fn encrypt_aes256(&self, data: &[u8], key_data: &[u8]) -> Result<Vec<u8>, SyncError> {
        use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
        
        let key = Key::<Aes256Gcm>::from_slice(key_data);
        let cipher = Aes256Gcm::new(key);
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt data
        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| SyncError::EncryptionError(format!("AES-256 encryption failed: {}", e)))?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    /// Decrypt data using AES-256-GCM
    async fn decrypt_aes256(&self, encrypted_data: &[u8], key_data: &[u8]) -> Result<Vec<u8>, SyncError> {
        use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
        
        if encrypted_data.len() < 12 {
            return Err(SyncError::EncryptionError("Invalid encrypted data length".to_string()));
        }

        let key = Key::<Aes256Gcm>::from_slice(key_data);
        let cipher = Aes256Gcm::new(key);
        
        // Extract nonce and ciphertext
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];
        
        // Decrypt data
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| SyncError::EncryptionError(format!("AES-256 decryption failed: {}", e)))?;
        
        Ok(plaintext)
    }

    /// Encrypt data using AES-128-GCM
    async fn encrypt_aes128(&self, data: &[u8], key_data: &[u8]) -> Result<Vec<u8>, SyncError> {
        use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
        
        // Use AES-256-GCM with 128-bit key (first 16 bytes)
        let key = Key::<Aes256Gcm>::from_slice(&key_data[..16]);
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| SyncError::EncryptionError(format!("AES-128 encryption failed: {}", e)))?;
        
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    /// Decrypt data using AES-128-GCM
    async fn decrypt_aes128(&self, encrypted_data: &[u8], key_data: &[u8]) -> Result<Vec<u8>, SyncError> {
        use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
        
        if encrypted_data.len() < 12 {
            return Err(SyncError::EncryptionError("Invalid encrypted data length".to_string()));
        }

        let key = Key::<Aes256Gcm>::from_slice(&key_data[..16]);
        let cipher = Aes256Gcm::new(key);
        
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];
        
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| SyncError::EncryptionError(format!("AES-128 decryption failed: {}", e)))?;
        
        Ok(plaintext)
    }

}

/// Key manager for generating, rotating, and managing encryption keys
pub struct KeyManager {
    key_store: RwLock<HashMap<String, EncryptionKey>>,
}

impl KeyManager {
    /// Create a new key manager
    pub fn new() -> Self {
        Self {
            key_store: RwLock::new(HashMap::new()),
        }
    }

    /// Generate a new encryption key
    pub async fn generate_key(&self, algorithm: EncryptionAlgorithm) -> Result<EncryptionKey, SyncError> {
        let key_id = self.generate_key_id();
        let key_data = match algorithm {
            EncryptionAlgorithm::Aes256 => self.generate_aes256_key().await?,
            EncryptionAlgorithm::Aes128 => self.generate_aes128_key().await?,
        };

        let key = EncryptionKey {
            id: key_id.clone(),
            algorithm,
            key_data,
            created_at: Utc::now(),
            expires_at: None,
            version: 1,
        };

        // Store key
        let mut store = self.key_store.write().await;
        store.insert(key_id, key.clone());

        Ok(key)
    }


    /// Rotate an existing key
    pub async fn rotate_key(&self, old_key: &EncryptionKey) -> Result<EncryptionKey, SyncError> {
        let new_key = self.generate_key(old_key.algorithm.clone()).await?;
        
        // Mark old key as expired
        let mut store = self.key_store.write().await;
        if let Some(existing_key) = store.get_mut(&old_key.id) {
            existing_key.expires_at = Some(Utc::now());
        }

        Ok(new_key)
    }

    /// Derive key from password using SHA-256
    pub async fn derive_key_from_password(&self, password: &str, salt: &[u8]) -> Result<Vec<u8>, SyncError> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        let derived_key = hasher.finalize().to_vec();
        
        Ok(derived_key)
    }

    /// Generate AES-256 key
    async fn generate_aes256_key(&self) -> Result<Vec<u8>, SyncError> {
        let mut key = [0u8; 32];
        OsRng.fill(&mut key);
        Ok(key.to_vec())
    }

    /// Generate AES-128 key
    async fn generate_aes128_key(&self) -> Result<Vec<u8>, SyncError> {
        let mut key = [0u8; 16];
        OsRng.fill(&mut key);
        Ok(key.to_vec())
    }


    /// Generate unique key ID
    fn generate_key_id(&self) -> String {
        let mut rng = OsRng;
        let random_bytes: [u8; 16] = rng.gen();
        format!("key_{}", general_purpose::STANDARD.encode(random_bytes))
    }

    /// Get key by ID
    pub async fn get_key(&self, key_id: &str) -> Option<EncryptionKey> {
        let store = self.key_store.read().await;
        store.get(key_id).cloned()
    }

    /// Revoke key
    pub async fn revoke_key(&self, key_id: &str) -> Result<(), SyncError> {
        let mut store = self.key_store.write().await;
        if let Some(key) = store.get_mut(key_id) {
            key.expires_at = Some(Utc::now());
            Ok(())
        } else {
            Err(SyncError::EncryptionError("Key not found".to_string()))
        }
    }

    /// List all keys
    pub async fn list_keys(&self) -> Vec<EncryptionKey> {
        let store = self.key_store.read().await;
        store.values().cloned().collect()
    }

    /// Clean up expired keys
    pub async fn cleanup_expired_keys(&self) -> usize {
        let mut store = self.key_store.write().await;
        let now = Utc::now();
        let expired_keys: Vec<String> = store
            .iter()
            .filter(|(_, key)| key.expires_at.map_or(false, |expires| expires < now))
            .map(|(id, _)| id.clone())
            .collect();
        
        for key_id in &expired_keys {
            store.remove(key_id);
        }
        
        expired_keys.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_aes256_encryption_decryption() {
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

    #[tokio::test]
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

    #[tokio::test]
    async fn test_password_derivation() {
        let key_manager = KeyManager::new();
        let password = "test_password_123";
        let salt = b"test_salt";
        
        let derived_key = key_manager.derive_key_from_password(password, salt).await.unwrap();
        assert!(!derived_key.is_empty());
        
        // Same password and salt should produce same key
        let derived_key2 = key_manager.derive_key_from_password(password, salt).await.unwrap();
        assert_eq!(derived_key, derived_key2);
    }
}
