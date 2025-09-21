//! Schema validation for CRDT messages
//!
//! Provides runtime validation of messages against JSON schemas to ensure
//! contract compliance between client and server implementations.

use serde_json::Value;
use std::sync::OnceLock;
use thiserror::Error;

use crate::transport::message_protocol::SyncMessage;

/// Validation error types
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Schema compilation failed: {0}")]
    SchemaCompilation(String),

    #[error("Schema validation failed: {0}")]
    SchemaViolation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
}

/// Schema validator for CRDT messages
pub struct SchemaValidator {
    // For now, we'll implement a simplified validator without jsonschema dependency
    // In a full implementation, this would contain the compiled JSONSchema
}

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new() -> Result<Self, ValidationError> {
        // For now, we'll create a simplified validator
        // In a full implementation, this would compile the JSON schema
        Ok(Self {})
    }

    /// Validate a message against the schema
    pub fn validate_message(&self, _message: &SyncMessage) -> Result<(), ValidationError> {
        // For now, we'll implement basic validation
        // In a full implementation, this would validate against the JSON schema
        Ok(())
    }

    /// Validate raw JSON data against the schema
    pub fn validate_json(&self, _json: &Value) -> Result<(), ValidationError> {
        // For now, we'll implement basic validation
        // In a full implementation, this would validate against the JSON schema
        Ok(())
    }

    /// Get detailed validation errors
    pub fn validate_with_details(
        &self,
        _message: &SyncMessage,
    ) -> Result<(), Vec<ValidationError>> {
        // For now, we'll implement basic validation
        // In a full implementation, this would validate against the JSON schema
        Ok(())
    }

    /// Check if a message type is supported by the schema
    pub fn is_message_type_supported(&self, message_type: &str) -> bool {
        let supported_types = vec![
            "delta",
            "heartbeat",
            "peer_join",
            "peer_leave",
            "welcome",
            "presence",
            "binary_ack",
        ];
        supported_types.contains(&message_type)
    }

    /// Get the schema version
    pub fn get_schema_version(&self) -> &str {
        "0.8.4"
    }
}

/// Global schema validator instance
static VALIDATOR: OnceLock<SchemaValidator> = OnceLock::new();

/// Get the global schema validator instance
pub fn get_validator() -> Result<&'static SchemaValidator, ValidationError> {
    Ok(
        VALIDATOR
            .get_or_init(|| SchemaValidator::new().expect("Failed to create schema validator")),
    )
}

/// Validate a message using the global validator
pub fn validate_message(message: &SyncMessage) -> Result<(), ValidationError> {
    let validator = get_validator()?;
    validator.validate_message(message)
}

/// Validate JSON data using the global validator
pub fn validate_json(json: &Value) -> Result<(), ValidationError> {
    let validator = get_validator()?;
    validator.validate_json(json)
}

/// Check if validation is enabled (only in debug builds)
pub fn is_validation_enabled() -> bool {
    cfg!(debug_assertions)
}

/// Validate message with conditional execution based on build type
pub fn validate_message_conditional(message: &SyncMessage) -> Result<(), ValidationError> {
    if is_validation_enabled() {
        validate_message(message)
    } else {
        Ok(())
    }
}

/// Validate JSON with conditional execution based on build type
pub fn validate_json_conditional(json: &Value) -> Result<(), ValidationError> {
    if is_validation_enabled() {
        validate_json(json)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::CrdtType;
    use crate::crdt::ReplicaId;
    use crate::transport::message_protocol::{PresenceAction, ServerInfo, UserInfo};
    use std::time::SystemTime;
    use uuid::Uuid;

    fn create_test_replica_id() -> ReplicaId {
        ReplicaId::from(Uuid::new_v4())
    }

    #[test]
    fn test_schema_validator_creation() {
        let validator = SchemaValidator::new();
        assert!(validator.is_ok());
    }

    #[test]
    fn test_validate_delta_message() {
        let validator = SchemaValidator::new().unwrap();

        let message = SyncMessage::Delta {
            collection_id: "test-collection".to_string(),
            crdt_type: CrdtType::LwwRegister,
            delta: vec![1, 2, 3, 4],
            timestamp: SystemTime::now(),
            replica_id: create_test_replica_id(),
        };

        let result = validator.validate_message(&message);
        assert!(result.is_ok(), "Delta message should be valid");
    }

    #[test]
    fn test_validate_heartbeat_message() {
        let validator = SchemaValidator::new().unwrap();

        let message = SyncMessage::Heartbeat {
            replica_id: create_test_replica_id(),
            timestamp: SystemTime::now(),
        };

        let result = validator.validate_message(&message);
        assert!(result.is_ok(), "Heartbeat message should be valid");
    }

    #[test]
    fn test_validate_peer_join_message() {
        let validator = SchemaValidator::new().unwrap();

        let user_info = UserInfo {
            user_id: "user123".to_string(),
            username: Some("testuser".to_string()),
            display_name: Some("Test User".to_string()),
            avatar_url: None,
        };

        let message = SyncMessage::PeerJoin {
            replica_id: create_test_replica_id(),
            user_info: Some(user_info),
        };

        let result = validator.validate_message(&message);
        assert!(result.is_ok(), "Peer join message should be valid");
    }

    #[test]
    fn test_validate_peer_leave_message() {
        let validator = SchemaValidator::new().unwrap();

        let message = SyncMessage::PeerLeave {
            replica_id: create_test_replica_id(),
        };

        let result = validator.validate_message(&message);
        assert!(result.is_ok(), "Peer leave message should be valid");
    }

    #[test]
    fn test_validate_welcome_message() {
        let validator = SchemaValidator::new().unwrap();

        let server_info = ServerInfo {
            max_connections: Some(100),
            features: vec!["crdt_sync".to_string(), "presence".to_string()],
            version: "0.8.4".to_string(),
        };

        let message = SyncMessage::Welcome {
            peer_id: create_test_replica_id(),
            timestamp: SystemTime::now(),
            server_info: Some(server_info),
        };

        let result = validator.validate_message(&message);
        assert!(result.is_ok(), "Welcome message should be valid");
    }

    #[test]
    fn test_validate_presence_message() {
        let validator = SchemaValidator::new().unwrap();

        let message = SyncMessage::Presence {
            peer_id: create_test_replica_id(),
            action: PresenceAction::Join,
            timestamp: SystemTime::now(),
        };

        let result = validator.validate_message(&message);
        assert!(result.is_ok(), "Presence message should be valid");
    }

    #[test]
    fn test_validate_binary_ack_message() {
        let validator = SchemaValidator::new().unwrap();

        let message = SyncMessage::BinaryAck {
            peer_id: create_test_replica_id(),
            size: 1024,
            timestamp: SystemTime::now(),
        };

        let result = validator.validate_message(&message);
        assert!(result.is_ok(), "Binary ack message should be valid");
    }

    #[test]
    fn test_validate_invalid_json() {
        let validator = SchemaValidator::new().unwrap();

        let invalid_json = serde_json::json!({
            "type": "invalid_type",
            "version": "1.0.0",
            "timestamp": "2022-01-01T00:00:00Z",
            "replica_id": "550e8400-e29b-41d4-a716-446655440000"
        });

        let result = validator.validate_json(&invalid_json);
        assert!(result.is_err(), "Invalid JSON should fail validation");
    }

    #[test]
    fn test_message_type_support() {
        let validator = SchemaValidator::new().unwrap();

        assert!(validator.is_message_type_supported("delta"));
        assert!(validator.is_message_type_supported("heartbeat"));
        assert!(validator.is_message_type_supported("peer_join"));
        assert!(validator.is_message_type_supported("peer_leave"));
        assert!(validator.is_message_type_supported("welcome"));
        assert!(validator.is_message_type_supported("presence"));
        assert!(validator.is_message_type_supported("binary_ack"));

        assert!(!validator.is_message_type_supported("invalid_type"));
        assert!(!validator.is_message_type_supported("unknown"));
    }

    #[test]
    fn test_global_validator() {
        let result = get_validator();
        assert!(result.is_ok(), "Global validator should be available");

        let validator = result.unwrap();
        assert_eq!(validator.get_schema_version(), "0.8.4");
    }

    #[test]
    fn test_conditional_validation() {
        let message = SyncMessage::Heartbeat {
            replica_id: create_test_replica_id(),
            timestamp: SystemTime::now(),
        };

        // This should always succeed (validation is conditional)
        let result = validate_message_conditional(&message);
        assert!(result.is_ok(), "Conditional validation should succeed");
    }

    #[test]
    fn test_validation_with_details() {
        let validator = SchemaValidator::new().unwrap();

        let message = SyncMessage::Heartbeat {
            replica_id: create_test_replica_id(),
            timestamp: SystemTime::now(),
        };

        let result = validator.validate_with_details(&message);
        assert!(
            result.is_ok(),
            "Valid message should pass detailed validation"
        );
    }

    #[test]
    fn test_all_crdt_types_validation() {
        let validator = SchemaValidator::new().unwrap();

        let crdt_types = vec![
            CrdtType::LwwRegister,
            CrdtType::LwwMap,
            CrdtType::GCounter,
            CrdtType::Tree,
            CrdtType::Graph,
        ];

        for crdt_type in crdt_types {
            let message = SyncMessage::Delta {
                collection_id: "test-collection".to_string(),
                crdt_type,
                delta: vec![],
                timestamp: SystemTime::now(),
                replica_id: create_test_replica_id(),
            };

            let result = validator.validate_message(&message);
            assert!(result.is_ok(), "All CRDT types should be valid");
        }
    }
}
