//! WebSocket server contract tests
//! 
//! Tests that validate the server correctly implements the WebSocket API contract
//! as defined in the OpenAPI specification.

use serde_json::Value;
use jsonschema::{JSONSchema, Draft};
use leptos_sync_core::transport::message_protocol::{SyncMessage, MessageCodec, UserInfo, ServerInfo};
use leptos_sync_core::crdt::{ReplicaId, CrdtType};
use std::time::SystemTime;
use uuid::Uuid;

/// Load and compile the CRDT message schema
fn load_message_schema() -> JSONSchema {
    let schema_content = include_str!("../../../docs/api/schemas/crdt-message.json");
    let schema: Value = serde_json::from_str(schema_content)
        .expect("Failed to parse CRDT message schema");
    JSONSchema::compile(&schema)
        .expect("Failed to compile CRDT message schema")
}

/// Create a test delta message
fn create_test_delta_message() -> SyncMessage {
    let replica_id = ReplicaId::from(Uuid::new_v4());
    SyncMessage::Delta {
        collection_id: "test-collection".to_string(),
        crdt_type: CrdtType::LwwRegister,
        delta: vec![1, 2, 3, 4],
        timestamp: SystemTime::now(),
        replica_id,
    }
}

/// Create a test heartbeat message
fn create_test_heartbeat_message() -> SyncMessage {
    let replica_id = ReplicaId::from(Uuid::new_v4());
    SyncMessage::Heartbeat {
        replica_id,
        timestamp: SystemTime::now(),
    }
}

/// Create a test peer join message
fn create_test_peer_join_message() -> SyncMessage {
    let replica_id = ReplicaId::from(Uuid::new_v4());
    let user_info = UserInfo {
        user_id: "user123".to_string(),
        username: Some("testuser".to_string()),
        display_name: Some("Test User".to_string()),
        avatar_url: None,
    };
    SyncMessage::PeerJoin {
        replica_id,
        user_info: Some(user_info),
    }
}

/// Create a test peer leave message
fn create_test_peer_leave_message() -> SyncMessage {
    let replica_id = ReplicaId::from(Uuid::new_v4());
    SyncMessage::PeerLeave { replica_id }
}

/// Create a test welcome message
fn create_test_welcome_message() -> SyncMessage {
    let replica_id = ReplicaId::from(Uuid::new_v4());
    let server_info = ServerInfo {
        server_id: "server-001".to_string(),
        version: "0.8.4".to_string(),
        capabilities: vec!["crdt_sync".to_string(), "presence".to_string()],
    };
    SyncMessage::Welcome {
        peer_id: replica_id,
        timestamp: SystemTime::now(),
        server_info,
    }
}

/// Create a test presence message
fn create_test_presence_message() -> SyncMessage {
    let replica_id = ReplicaId::from(Uuid::new_v4());
    SyncMessage::Presence {
        peer_id: replica_id,
        action: leptos_sync_core::transport::message_protocol::PresenceAction::Join,
        timestamp: SystemTime::now(),
    }
}

/// Create a test binary acknowledgment message
fn create_test_binary_ack_message() -> SyncMessage {
    let replica_id = ReplicaId::from(Uuid::new_v4());
    SyncMessage::BinaryAck {
        peer_id: replica_id,
        size: 1024,
        timestamp: SystemTime::now(),
    }
}

#[tokio::test]
async fn test_delta_message_schema_compliance() {
    let schema = load_message_schema();
    
    // Create test message
    let message = create_test_delta_message();
    
    // Serialize and validate against schema
    let serialized = MessageCodec::serialize(&message)
        .expect("Failed to serialize delta message");
    let json: Value = serde_json::from_slice(&serialized)
        .expect("Failed to deserialize message");
    
    let result = schema.validate(&json);
    
    if let Err(errors) = result {
        for error in errors {
            eprintln!("Schema validation error: {}", error);
        }
        panic!("Delta message does not comply with schema");
    }
}

#[tokio::test]
async fn test_heartbeat_message_schema_compliance() {
    let schema = load_message_schema();
    
    // Create test message
    let message = create_test_heartbeat_message();
    
    // Serialize and validate against schema
    let serialized = MessageCodec::serialize(&message)
        .expect("Failed to serialize heartbeat message");
    let json: Value = serde_json::from_slice(&serialized)
        .expect("Failed to deserialize message");
    
    let result = schema.validate(&json);
    
    if let Err(errors) = result {
        for error in errors {
            eprintln!("Schema validation error: {}", error);
        }
        panic!("Heartbeat message does not comply with schema");
    }
}

#[tokio::test]
async fn test_peer_join_message_schema_compliance() {
    let schema = load_message_schema();
    
    // Create test message
    let message = create_test_peer_join_message();
    
    // Serialize and validate against schema
    let serialized = MessageCodec::serialize(&message)
        .expect("Failed to serialize peer join message");
    let json: Value = serde_json::from_slice(&serialized)
        .expect("Failed to deserialize message");
    
    let result = schema.validate(&json);
    
    if let Err(errors) = result {
        for error in errors {
            eprintln!("Schema validation error: {}", error);
        }
        panic!("Peer join message does not comply with schema");
    }
}

#[tokio::test]
async fn test_peer_leave_message_schema_compliance() {
    let schema = load_message_schema();
    
    // Create test message
    let message = create_test_peer_leave_message();
    
    // Serialize and validate against schema
    let serialized = MessageCodec::serialize(&message)
        .expect("Failed to serialize peer leave message");
    let json: Value = serde_json::from_slice(&serialized)
        .expect("Failed to deserialize message");
    
    let result = schema.validate(&json);
    
    if let Err(errors) = result {
        for error in errors {
            eprintln!("Schema validation error: {}", error);
        }
        panic!("Peer leave message does not comply with schema");
    }
}

#[tokio::test]
async fn test_welcome_message_schema_compliance() {
    let schema = load_message_schema();
    
    // Create test message
    let message = create_test_welcome_message();
    
    // Serialize and validate against schema
    let serialized = MessageCodec::serialize(&message)
        .expect("Failed to serialize welcome message");
    let json: Value = serde_json::from_slice(&serialized)
        .expect("Failed to deserialize message");
    
    let result = schema.validate(&json);
    
    if let Err(errors) = result {
        for error in errors {
            eprintln!("Schema validation error: {}", error);
        }
        panic!("Welcome message does not comply with schema");
    }
}

#[tokio::test]
async fn test_presence_message_schema_compliance() {
    let schema = load_message_schema();
    
    // Create test message
    let message = create_test_presence_message();
    
    // Serialize and validate against schema
    let serialized = MessageCodec::serialize(&message)
        .expect("Failed to serialize presence message");
    let json: Value = serde_json::from_slice(&serialized)
        .expect("Failed to deserialize message");
    
    let result = schema.validate(&json);
    
    if let Err(errors) = result {
        for error in errors {
            eprintln!("Schema validation error: {}", error);
        }
        panic!("Presence message does not comply with schema");
    }
}

#[tokio::test]
async fn test_binary_ack_message_schema_compliance() {
    let schema = load_message_schema();
    
    // Create test message
    let message = create_test_binary_ack_message();
    
    // Serialize and validate against schema
    let serialized = MessageCodec::serialize(&message)
        .expect("Failed to serialize binary ack message");
    let json: Value = serde_json::from_slice(&serialized)
        .expect("Failed to deserialize message");
    
    let result = schema.validate(&json);
    
    if let Err(errors) = result {
        for error in errors {
            eprintln!("Schema validation error: {}", error);
        }
        panic!("Binary ack message does not comply with schema");
    }
}

#[tokio::test]
async fn test_all_message_types_serialization() {
    let test_cases = vec![
        ("delta", create_test_delta_message()),
        ("heartbeat", create_test_heartbeat_message()),
        ("peer_join", create_test_peer_join_message()),
        ("peer_leave", create_test_peer_leave_message()),
        ("welcome", create_test_welcome_message()),
        ("presence", create_test_presence_message()),
        ("binary_ack", create_test_binary_ack_message()),
    ];
    
    for (message_type, message) in test_cases {
        // Test serialization
        let serialized = MessageCodec::serialize(&message)
            .expect(&format!("Failed to serialize {} message", message_type));
        
        // Test deserialization
        let deserialized = MessageCodec::deserialize(&serialized)
            .expect(&format!("Failed to deserialize {} message", message_type));
        
        // Verify round-trip consistency
        match (&message, &deserialized) {
            (SyncMessage::Delta { collection_id: id1, crdt_type: type1, delta: delta1, replica_id: rid1, .. },
             SyncMessage::Delta { collection_id: id2, crdt_type: type2, delta: delta2, replica_id: rid2, .. }) => {
                assert_eq!(id1, id2, "Collection ID mismatch in {} message", message_type);
                assert_eq!(type1, type2, "CRDT type mismatch in {} message", message_type);
                assert_eq!(delta1, delta2, "Delta data mismatch in {} message", message_type);
                assert_eq!(rid1, rid2, "Replica ID mismatch in {} message", message_type);
            }
            (SyncMessage::Heartbeat { replica_id: rid1, .. },
             SyncMessage::Heartbeat { replica_id: rid2, .. }) => {
                assert_eq!(rid1, rid2, "Replica ID mismatch in {} message", message_type);
            }
            (SyncMessage::PeerJoin { replica_id: rid1, user_info: info1 },
             SyncMessage::PeerJoin { replica_id: rid2, user_info: info2 }) => {
                assert_eq!(rid1, rid2, "Replica ID mismatch in {} message", message_type);
                assert_eq!(info1, info2, "User info mismatch in {} message", message_type);
            }
            (SyncMessage::PeerLeave { replica_id: rid1 },
             SyncMessage::PeerLeave { replica_id: rid2 }) => {
                assert_eq!(rid1, rid2, "Replica ID mismatch in {} message", message_type);
            }
            (SyncMessage::Welcome { peer_id: pid1, server_info: info1, .. },
             SyncMessage::Welcome { peer_id: pid2, server_info: info2, .. }) => {
                assert_eq!(pid1, pid2, "Peer ID mismatch in {} message", message_type);
                assert_eq!(info1, info2, "Server info mismatch in {} message", message_type);
            }
            (SyncMessage::Presence { peer_id: pid1, action: action1, .. },
             SyncMessage::Presence { peer_id: pid2, action: action2, .. }) => {
                assert_eq!(pid1, pid2, "Peer ID mismatch in {} message", message_type);
                assert_eq!(action1, action2, "Action mismatch in {} message", message_type);
            }
            (SyncMessage::BinaryAck { peer_id: pid1, size: size1, .. },
             SyncMessage::BinaryAck { peer_id: pid2, size: size2, .. }) => {
                assert_eq!(pid1, pid2, "Peer ID mismatch in {} message", message_type);
                assert_eq!(size1, size2, "Size mismatch in {} message", message_type);
            }
            _ => panic!("Message type mismatch in {} message", message_type),
        }
    }
}

#[tokio::test]
async fn test_message_codec_error_handling() {
    // Test invalid JSON
    let invalid_json = b"invalid json";
    let result = MessageCodec::deserialize(invalid_json);
    assert!(result.is_err(), "Should fail to deserialize invalid JSON");
    
    // Test empty data
    let empty_data = b"";
    let result = MessageCodec::deserialize(empty_data);
    assert!(result.is_err(), "Should fail to deserialize empty data");
    
    // Test malformed message structure
    let malformed_message = r#"{"type": "invalid_type", "version": "1.0.0"}"#;
    let result = MessageCodec::deserialize(malformed_message.as_bytes());
    assert!(result.is_err(), "Should fail to deserialize malformed message");
}

#[tokio::test]
async fn test_crdt_type_enum_compliance() {
    let schema = load_message_schema();
    
    // Test all CRDT types from the schema
    let crdt_types = vec![
        "lww_register",
        "lww_map", 
        "g_counter",
        "rga",
        "lseq",
        "tree",
        "graph",
    ];
    
    for crdt_type in crdt_types {
        let message = SyncMessage::Delta {
            collection_id: "test-collection".to_string(),
            crdt_type: match crdt_type {
                "lww_register" => CrdtType::LwwRegister,
                "lww_map" => CrdtType::LwwMap,
                "g_counter" => CrdtType::GCounter,
                "rga" => CrdtType::Rga,
                "lseq" => CrdtType::Lseq,
                "tree" => CrdtType::Tree,
                "graph" => CrdtType::Graph,
                _ => panic!("Unknown CRDT type: {}", crdt_type),
            },
            delta: vec![],
            timestamp: SystemTime::now(),
            replica_id: ReplicaId::from(Uuid::new_v4()),
        };
        
        let serialized = MessageCodec::serialize(&message)
            .expect(&format!("Failed to serialize {} message", crdt_type));
        let json: Value = serde_json::from_slice(&serialized)
            .expect(&format!("Failed to deserialize {} message", crdt_type));
        
        let result = schema.validate(&json);
        if let Err(errors) = result {
            for error in errors {
                eprintln!("Schema validation error for {}: {}", crdt_type, error);
            }
            panic!("{} message does not comply with schema", crdt_type);
        }
    }
}
