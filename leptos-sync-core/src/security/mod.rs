//! Security features including encryption and compression

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod encryption;
pub mod compression;
pub mod hashing;
pub mod authentication;
pub mod gdpr;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Decryption error: {0}")]
    Decryption(String),
    #[error("Compression error: {0}")]
    Compression(String),
    #[error("Decompression error: {0}")]
    Decompression(String),
    #[error("Hash error: {0}")]
    Hash(String),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Invalid algorithm: {0}")]
    InvalidAlgorithm(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encryption_enabled: bool,
    pub compression_enabled: bool,
    pub encryption_algorithm: EncryptionAlgorithm,
    pub compression_algorithm: CompressionAlgorithm,
    pub key_derivation: KeyDerivationConfig,
    pub integrity_checking: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_enabled: true,
            compression_enabled: true,
            encryption_algorithm: EncryptionAlgorithm::Aes256,
            compression_algorithm: CompressionAlgorithm::Lz4,
            key_derivation: KeyDerivationConfig::default(),
            integrity_checking: true,
        }
    }
}

/// Re-export encryption algorithm from submodule
pub use encryption::EncryptionAlgorithm;

/// Re-export compression algorithm from submodule
pub use compression::CompressionAlgorithm;

/// Re-export authentication types from submodule
pub use authentication::{AuthenticationManager, AuthProvider, UserSession, User, AuthConfig};

/// Re-export GDPR types from submodule
pub use gdpr::{GDPRCompliance, DataSubject, DataProcessingPurpose, PersonalDataRecord, AuditLogEntry};

/// Key derivation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationConfig {
    pub algorithm: KeyDerivationAlgorithm,
    pub iterations: u32,
    pub salt_length: usize,
    pub key_length: usize,
}

impl Default for KeyDerivationConfig {
    fn default() -> Self {
        Self {
            algorithm: KeyDerivationAlgorithm::Argon2,
            iterations: 100_000,
            salt_length: 32,
            key_length: 32,
        }
    }
}

/// Key derivation algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyDerivationAlgorithm {
    Argon2,
    Pbkdf2,
    Scrypt,
}

/// Security manager for handling encryption and compression
pub struct SecurityManager {
    config: SecurityConfig,
    encryption: encryption::EncryptionManager,
    compression: compression::CompressionManager,
    hashing: hashing::HashManager,
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Result<Self, SecurityError> {
        let encryption = encryption::EncryptionManager::new(config.encryption_algorithm.clone());
        let compression = compression::CompressionManager::new(config.compression_algorithm.clone())?;
        let hashing = hashing::HashManager::new()?;

        Ok(Self {
            config,
            encryption,
            compression,
            hashing,
        })
    }

    /// Secure data (encrypt and compress)
    pub async fn secure_data(
        &self,
        data: &[u8],
        key: &encryption::EncryptionKey,
    ) -> Result<Vec<u8>, SecurityError> {
        let mut processed_data = data.to_vec();

        // Compress first (if enabled)
        if self.config.compression_enabled {
            processed_data = self.compression.compress(&processed_data)?;
        }

        // Encrypt (if enabled)
        if self.config.encryption_enabled {
            processed_data = self.encryption.encrypt(&processed_data, key).await
                .map_err(|e| SecurityError::Encryption(e.to_string()))?;
        }

        // Add integrity check (if enabled)
        if self.config.integrity_checking {
            let hash = self.hashing.hash(&processed_data)?;
            processed_data.extend_from_slice(&hash);
        }

        Ok(processed_data)
    }

    /// Unsecure data (decrypt and decompress)
    pub async fn unsecure_data(
        &self,
        data: &[u8],
        key: &encryption::EncryptionKey,
    ) -> Result<Vec<u8>, SecurityError> {
        let mut processed_data = data.to_vec();

        // Verify integrity check (if enabled)
        if self.config.integrity_checking {
            let expected_hash_len = 32; // SHA-256 hash length
            if processed_data.len() < expected_hash_len {
                return Err(SecurityError::InvalidData("Data too short for integrity check".to_string()));
            }

            let data_len = processed_data.len() - expected_hash_len;
            let (data_part, hash_part) = processed_data.split_at(data_len);
            
            let actual_hash = self.hashing.hash(data_part)?;
            if actual_hash != hash_part {
                return Err(SecurityError::Hash("Integrity check failed".to_string()));
            }

            processed_data = data_part.to_vec();
        }

        // Decrypt (if enabled)
        if self.config.encryption_enabled {
            processed_data = self.encryption.decrypt(&processed_data, key).await
                .map_err(|e| SecurityError::Decryption(e.to_string()))?;
        }

        // Decompress (if enabled)
        if self.config.compression_enabled {
            processed_data = self.compression.decompress(&processed_data)?;
        }

        Ok(processed_data)
    }

    /// Generate a secure key from a password
    pub async fn derive_key(
        &self,
        password: &str,
        salt: Option<&[u8]>,
    ) -> Result<Vec<u8>, SecurityError> {
        let salt = salt.unwrap_or_else(|| b"default_salt");
        
        match self.config.key_derivation.algorithm {
            KeyDerivationAlgorithm::Argon2 => {
                // Use Argon2 for key derivation
                self.hashing.derive_key_argon2(
                    password.as_bytes(),
                    salt,
                    self.config.key_derivation.iterations,
                    self.config.key_derivation.key_length,
                )
            }
            KeyDerivationAlgorithm::Pbkdf2 => {
                // Use PBKDF2 for key derivation
                self.hashing.derive_key_pbkdf2(
                    password.as_bytes(),
                    salt,
                    self.config.key_derivation.iterations,
                    self.config.key_derivation.key_length,
                )
            }
            KeyDerivationAlgorithm::Scrypt => {
                // Use Scrypt for key derivation
                self.hashing.derive_key_scrypt(
                    password.as_bytes(),
                    salt,
                    self.config.key_derivation.iterations,
                    self.config.key_derivation.key_length,
                )
            }
        }
    }

    /// Generate a random nonce (for compatibility)
    pub fn generate_nonce(&self) -> Result<Vec<u8>, SecurityError> {
        // Generate a 12-byte nonce for GCM
        use rand::{Rng, rngs::OsRng};
        let mut nonce = [0u8; 12];
        OsRng.fill(&mut nonce);
        Ok(nonce.to_vec())
    }

    /// Get security configuration
    pub fn config(&self) -> &SecurityConfig {
        &self.config
    }

    /// Update security configuration
    pub fn update_config(&mut self, config: SecurityConfig) -> Result<(), SecurityError> {
        let encryption = encryption::EncryptionManager::new(config.encryption_algorithm.clone());
        let compression = compression::CompressionManager::new(config.compression_algorithm.clone())?;
        
        self.config = config;
        self.encryption = encryption;
        self.compression = compression;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_manager_creation() {
        let config = SecurityConfig::default();
        let manager = SecurityManager::new(config);
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert!(config.encryption_enabled);
        assert!(config.compression_enabled);
        assert!(config.integrity_checking);
    }

    #[tokio::test]
    async fn test_key_derivation_config_default() {
        let config = KeyDerivationConfig::default();
        assert_eq!(config.iterations, 100_000);
        assert_eq!(config.salt_length, 32);
        assert_eq!(config.key_length, 32);
    }
}
