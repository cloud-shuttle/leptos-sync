//! GDPR compliance system for data protection and privacy rights

use crate::SyncError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

/// Data subject information for GDPR compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSubject {
    pub id: String,
    pub email: String,
    pub name: String,
    pub consent_given: bool,
    pub consent_date: DateTime<Utc>,
    pub consent_withdrawn_date: Option<DateTime<Utc>>,
    pub data_retention_period: Option<Duration>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Data processing purpose for GDPR compliance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataProcessingPurpose {
    ServiceProvision,
    Analytics,
    Marketing,
    LegalCompliance,
    Research,
    Other(String),
}

/// Personal data record with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalDataRecord {
    pub id: String,
    pub subject_id: String,
    pub data: Vec<u8>,
    pub purpose: DataProcessingPurpose,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_encrypted: bool,
    pub encryption_key_id: Option<String>,
}

/// Audit log entry for GDPR compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub user_id: String,
    pub operation: String,
    pub timestamp: String,
    pub details: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// GDPR compliance manager
pub struct GDPRCompliance {
    data_subjects: RwLock<HashMap<String, DataSubject>>,
    personal_data: RwLock<HashMap<String, PersonalDataRecord>>,
    audit_log: RwLock<Vec<AuditLogEntry>>,
    consent_records: RwLock<HashMap<String, Vec<DataProcessingPurpose>>>,
}

impl GDPRCompliance {
    /// Create a new GDPR compliance manager
    pub fn new() -> Self {
        Self {
            data_subjects: RwLock::new(HashMap::new()),
            personal_data: RwLock::new(HashMap::new()),
            audit_log: RwLock::new(Vec::new()),
            consent_records: RwLock::new(HashMap::new()),
        }
    }

    /// Register a data subject
    pub async fn register_data_subject(&self, subject: DataSubject) -> Result<(), SyncError> {
        let subject_id = subject.id.clone();
        
        // Check if subject already exists
        {
            let subjects = self.data_subjects.read().await;
            if subjects.contains_key(&subject_id) {
                return Err(SyncError::GDPRError("Data subject already exists".to_string()));
            }
        }

        // Store data subject
        {
            let mut subjects = self.data_subjects.write().await;
            subjects.insert(subject_id.clone(), subject);
        }

        // Log the registration
        self.log_audit_event(&subject_id, "DATA_SUBJECT_REGISTERED", "Data subject registered").await?;

        Ok(())
    }

    /// Get data subject by ID
    pub async fn get_data_subject(&self, subject_id: &str) -> Result<DataSubject, SyncError> {
        let subjects = self.data_subjects.read().await;
        subjects.get(subject_id)
            .cloned()
            .ok_or_else(|| SyncError::GDPRError("Data subject not found".to_string()))
    }

    /// Update data subject information
    pub async fn update_data_subject(&self, subject_id: &str, updates: DataSubject) -> Result<(), SyncError> {
        {
            let mut subjects = self.data_subjects.write().await;
            if let Some(subject) = subjects.get_mut(subject_id) {
                *subject = updates;
                subject.updated_at = Utc::now();
            } else {
                return Err(SyncError::GDPRError("Data subject not found".to_string()));
            }
        }

        // Log the update
        self.log_audit_event(subject_id, "DATA_SUBJECT_UPDATED", "Data subject information updated").await?;

        Ok(())
    }

    /// Withdraw consent for data subject
    pub async fn withdraw_consent(&self, subject_id: &str) -> Result<(), SyncError> {
        {
            let mut subjects = self.data_subjects.write().await;
            if let Some(subject) = subjects.get_mut(subject_id) {
                subject.consent_given = false;
                subject.consent_withdrawn_date = Some(Utc::now());
                subject.updated_at = Utc::now();
            } else {
                return Err(SyncError::GDPRError("Data subject not found".to_string()));
            }
        }

        // Log the consent withdrawal
        self.log_audit_event(subject_id, "CONSENT_WITHDRAWN", "Data subject withdrew consent").await?;

        Ok(())
    }

    /// Withdraw consent for specific purpose
    pub async fn withdraw_consent_for_purpose(&self, subject_id: &str, purpose: DataProcessingPurpose) -> Result<(), SyncError> {
        // Remove data for the specific purpose
        {
            let mut personal_data = self.personal_data.write().await;
            personal_data.retain(|_, record| {
                !(record.subject_id == subject_id && record.purpose == purpose)
            });
        }

        // Update consent records
        {
            let mut consent_records = self.consent_records.write().await;
            if let Some(purposes) = consent_records.get_mut(subject_id) {
                purposes.retain(|p| p != &purpose);
            }
        }

        // Log the purpose-specific consent withdrawal
        self.log_audit_event(subject_id, "CONSENT_WITHDRAWN_FOR_PURPOSE", 
            &format!("Consent withdrawn for purpose: {:?}", purpose)).await?;

        Ok(())
    }

    /// Store personal data
    pub async fn store_personal_data(&self, subject_id: &str, data: &[u8], purpose: DataProcessingPurpose) -> Result<String, SyncError> {
        // Check if data subject exists and has given consent
        {
            let subjects = self.data_subjects.read().await;
            if let Some(subject) = subjects.get(subject_id) {
                if !subject.consent_given {
                    return Err(SyncError::GDPRError("No consent given for data processing".to_string()));
                }
            } else {
                return Err(SyncError::GDPRError("Data subject not found".to_string()));
            }
        }

        // Generate data record ID
        let record_id = self.generate_record_id();

        // Create personal data record
        let record = PersonalDataRecord {
            id: record_id.clone(),
            subject_id: subject_id.to_string(),
            data: data.to_vec(),
            purpose: purpose.clone(),
            created_at: Utc::now(),
            expires_at: None,
            is_encrypted: false,
            encryption_key_id: None,
        };

        // Store the record
        {
            let mut personal_data = self.personal_data.write().await;
            personal_data.insert(record_id.clone(), record);
        }

        // Update consent records
        {
            let mut consent_records = self.consent_records.write().await;
            consent_records.entry(subject_id.to_string())
                .or_insert_with(Vec::new)
                .push(purpose.clone());
        }

        // Log the data storage
        self.log_audit_event(subject_id, "PERSONAL_DATA_STORED", 
            &format!("Personal data stored for purpose: {:?}", purpose)).await?;

        Ok(record_id)
    }

    /// Store personal data with retention period
    pub async fn store_personal_data_with_retention(&self, subject_id: &str, data: &[u8], purpose: DataProcessingPurpose, retention_period: Duration) -> Result<String, SyncError> {
        // Check consent
        {
            let subjects = self.data_subjects.read().await;
            if let Some(subject) = subjects.get(subject_id) {
                if !subject.consent_given {
                    return Err(SyncError::GDPRError("No consent given for data processing".to_string()));
                }
            } else {
                return Err(SyncError::GDPRError("Data subject not found".to_string()));
            }
        }

        let record_id = self.generate_record_id();
        let expires_at = Utc::now() + retention_period;

        let record = PersonalDataRecord {
            id: record_id.clone(),
            subject_id: subject_id.to_string(),
            data: data.to_vec(),
            purpose: purpose.clone(),
            created_at: Utc::now(),
            expires_at: Some(expires_at),
            is_encrypted: false,
            encryption_key_id: None,
        };

        {
            let mut personal_data = self.personal_data.write().await;
            personal_data.insert(record_id.clone(), record);
        }

        {
            let mut consent_records = self.consent_records.write().await;
            consent_records.entry(subject_id.to_string())
                .or_insert_with(Vec::new)
                .push(purpose.clone());
        }

        self.log_audit_event(subject_id, "PERSONAL_DATA_STORED_WITH_RETENTION", 
            &format!("Personal data stored with retention period: {:?}", retention_period)).await?;

        Ok(record_id)
    }

    /// Get personal data for a subject
    pub async fn get_personal_data(&self, subject_id: &str) -> Result<Vec<u8>, SyncError> {
        let personal_data = self.personal_data.read().await;
        
        // Find the first record for the subject (simplified implementation)
        for record in personal_data.values() {
            if record.subject_id == subject_id {
                return Ok(record.data.clone());
            }
        }

        Err(SyncError::GDPRError("No personal data found for subject".to_string()))
    }

    /// Export all personal data for a subject (Right to Data Portability)
    pub async fn export_all_personal_data(&self, subject_id: &str) -> Result<Vec<Vec<u8>>, SyncError> {
        let personal_data = self.personal_data.read().await;
        
        let mut exported_data = Vec::new();
        for record in personal_data.values() {
            if record.subject_id == subject_id {
                exported_data.push(record.data.clone());
            }
        }

        // Log the data export
        self.log_audit_event(subject_id, "DATA_EXPORTED", "All personal data exported").await?;

        Ok(exported_data)
    }

    /// Delete all personal data for a subject (Right to be Forgotten)
    pub async fn delete_all_personal_data(&self, subject_id: &str) -> Result<(), SyncError> {
        // Delete all personal data records
        {
            let mut personal_data = self.personal_data.write().await;
            personal_data.retain(|_, record| record.subject_id != subject_id);
        }

        // Delete consent records
        {
            let mut consent_records = self.consent_records.write().await;
            consent_records.remove(subject_id);
        }

        // Delete data subject record
        {
            let mut subjects = self.data_subjects.write().await;
            subjects.remove(subject_id);
        }

        // Log the data deletion
        self.log_audit_event(subject_id, "ALL_DATA_DELETED", "All personal data deleted (Right to be Forgotten)").await?;

        Ok(())
    }

    /// Get audit log for a subject
    pub async fn get_audit_log(&self, subject_id: &str) -> Result<Vec<AuditLogEntry>, SyncError> {
        let audit_log = self.audit_log.read().await;
        let subject_logs: Vec<AuditLogEntry> = audit_log
            .iter()
            .filter(|entry| entry.user_id == subject_id)
            .cloned()
            .collect();

        Ok(subject_logs)
    }

    /// Clean up expired data
    pub async fn cleanup_expired_data(&self) -> Result<usize, SyncError> {
        let now = Utc::now();
        let mut expired_count = 0;

        {
            let mut personal_data = self.personal_data.write().await;
            personal_data.retain(|_, record| {
                if let Some(expires_at) = record.expires_at {
                    if now > expires_at {
                        expired_count += 1;
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            });
        }

        // Log the cleanup
        self.log_audit_event("SYSTEM", "EXPIRED_DATA_CLEANUP", 
            &format!("Cleaned up {} expired data records", expired_count)).await?;

        Ok(expired_count)
    }

    /// Log audit event
    async fn log_audit_event(&self, user_id: &str, operation: &str, details: &str) -> Result<(), SyncError> {
        let entry = AuditLogEntry {
            id: self.generate_audit_id(),
            user_id: user_id.to_string(),
            operation: operation.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            details: details.to_string(),
            ip_address: None,
            user_agent: None,
        };

        {
            let mut audit_log = self.audit_log.write().await;
            audit_log.push(entry);
        }

        Ok(())
    }

    /// Generate unique record ID
    fn generate_record_id(&self) -> String {
        use rand::{Rng, rngs::OsRng};
        use base64::{Engine as _, engine::general_purpose};
        let mut rng = OsRng;
        let random_bytes: [u8; 16] = rng.gen();
        format!("record_{}", general_purpose::STANDARD.encode(random_bytes))
    }

    /// Generate unique audit ID
    fn generate_audit_id(&self) -> String {
        use rand::{Rng, rngs::OsRng};
        use base64::{Engine as _, engine::general_purpose};
        let mut rng = OsRng;
        let random_bytes: [u8; 16] = rng.gen();
        format!("audit_{}", general_purpose::STANDARD.encode(random_bytes))
    }

    /// Get all data subjects
    pub async fn list_data_subjects(&self) -> Vec<DataSubject> {
        let subjects = self.data_subjects.read().await;
        subjects.values().cloned().collect()
    }

    /// Get data processing purposes for a subject
    pub async fn get_processing_purposes(&self, subject_id: &str) -> Result<Vec<DataProcessingPurpose>, SyncError> {
        let consent_records = self.consent_records.read().await;
        Ok(consent_records.get(subject_id).cloned().unwrap_or_default())
    }

    /// Check if subject has given consent for a specific purpose
    pub async fn has_consent_for_purpose(&self, subject_id: &str, purpose: &DataProcessingPurpose) -> Result<bool, SyncError> {
        let consent_records = self.consent_records.read().await;
        Ok(consent_records.get(subject_id)
            .map(|purposes| purposes.contains(purpose))
            .unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_data_subject_registration() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: Utc::now(),
            consent_withdrawn_date: None,
            data_retention_period: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let result = gdpr.register_data_subject(subject.clone()).await;
        assert!(result.is_ok());
        
        // Verify subject is registered
        let retrieved = gdpr.get_data_subject("user_123").await.unwrap();
        assert_eq!(retrieved.email, subject.email);
    }

    #[tokio::test]
    async fn test_consent_management() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: Utc::now(),
            consent_withdrawn_date: None,
            data_retention_period: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        gdpr.register_data_subject(subject).await.unwrap();
        
        // Withdraw consent
        gdpr.withdraw_consent("user_123").await.unwrap();
        
        // Verify consent is withdrawn
        let subject = gdpr.get_data_subject("user_123").await.unwrap();
        assert!(!subject.consent_given);
        assert!(subject.consent_withdrawn_date.is_some());
    }

    #[tokio::test]
    async fn test_right_to_be_forgotten() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: Utc::now(),
            consent_withdrawn_date: None,
            data_retention_period: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
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

    #[tokio::test]
    async fn test_data_portability() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: Utc::now(),
            consent_withdrawn_date: None,
            data_retention_period: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
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

    #[tokio::test]
    async fn test_data_processing_purposes() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: Utc::now(),
            consent_withdrawn_date: None,
            data_retention_period: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
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

    #[tokio::test]
    async fn test_data_retention_policy() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: Utc::now(),
            consent_withdrawn_date: None,
            data_retention_period: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        gdpr.register_data_subject(subject).await.unwrap();
        
        // Store data with retention period
        let data = "temporary_data".as_bytes();
        gdpr.store_personal_data_with_retention("user_123", data, DataProcessingPurpose::ServiceProvision, Duration::seconds(1)).await.unwrap();
        
        // Wait for retention period to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Trigger retention cleanup
        let cleaned_count = gdpr.cleanup_expired_data().await.unwrap();
        assert_eq!(cleaned_count, 1);
        
        // Verify data is deleted
        let retrieved_data = gdpr.get_personal_data("user_123").await;
        assert!(retrieved_data.is_err());
    }

    #[tokio::test]
    async fn test_audit_logging() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: Utc::now(),
            consent_withdrawn_date: None,
            data_retention_period: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
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

    #[tokio::test]
    async fn test_consent_validation() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: false, // No consent given
            consent_date: Utc::now(),
            consent_withdrawn_date: None,
            data_retention_period: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        gdpr.register_data_subject(subject).await.unwrap();
        
        // Try to store data without consent
        let result = gdpr.store_personal_data("user_123", b"test_data", DataProcessingPurpose::ServiceProvision).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No consent given"));
    }

    #[tokio::test]
    async fn test_duplicate_subject_registration() {
        let gdpr = GDPRCompliance::new();
        let subject = DataSubject {
            id: "user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
            consent_given: true,
            consent_date: Utc::now(),
            consent_withdrawn_date: None,
            data_retention_period: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Register first time
        let result1 = gdpr.register_data_subject(subject.clone()).await;
        assert!(result1.is_ok());
        
        // Try to register same subject again
        let result2 = gdpr.register_data_subject(subject).await;
        assert!(result2.is_err());
        assert!(result2.unwrap_err().to_string().contains("already exists"));
    }
}
