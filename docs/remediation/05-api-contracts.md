# API Contracts & Schema Definition - High Priority

## Overview
Establish formal API contracts with OpenAPI specifications and implement contract testing to ensure client-server compatibility.

## Current State
- No OpenAPI/Swagger specifications
- No formal schema definitions for messages
- No contract testing between client/server
- Message formats defined informally in code
- No API versioning strategy

## Objectives

### API Documentation
- OpenAPI 3.0 specifications for all endpoints
- JSON Schema definitions for CRDT messages
- Formal protocol documentation
- Interactive API explorer (Swagger UI)

### Contract Testing
- Server contract validation
- Client SDK contract compliance
- Message protocol compatibility testing
- Version compatibility matrices

## Implementation Plan

### Phase 1: Message Schema Definition (Week 1)
**File**: `docs/api/schemas/` (multiple files < 200 lines each)

Define JSON schemas for all message types:

#### CRDT Message Schema
```json
// schemas/crdt-message.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://leptos-sync.io/schemas/crdt-message.json",
  "title": "CRDT Message",
  "type": "object",
  "properties": {
    "type": {
      "type": "string",
      "enum": ["delta", "heartbeat", "peer_join", "peer_leave"]
    },
    "version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+\\.\\d+$"
    },
    "timestamp": {
      "type": "string",
      "format": "date-time"
    },
    "replica_id": {
      "type": "string",
      "format": "uuid"
    }
  },
  "required": ["type", "version", "timestamp", "replica_id"],
  "allOf": [
    {
      "if": { "properties": { "type": { "const": "delta" } } },
      "then": {
        "properties": {
          "collection_id": { "type": "string" },
          "crdt_type": { 
            "type": "string",
            "enum": ["lww_register", "lww_map", "g_counter", "rga", "lseq", "tree", "graph"]
          },
          "delta": {
            "type": "object",
            "additionalProperties": true
          }
        },
        "required": ["collection_id", "crdt_type", "delta"]
      }
    }
  ]
}
```

### Phase 2: WebSocket API Specification (Week 1)  
**File**: `docs/api/websocket-api.yaml` (< 300 lines)

```yaml
openapi: 3.0.3
info:
  title: Leptos-Sync WebSocket API
  version: 0.8.4
  description: Real-time synchronization API for CRDT operations

servers:
  - url: ws://localhost:3001
    description: Development server
  - url: wss://api.example.com
    description: Production server

paths:
  /sync:
    get:
      summary: WebSocket connection for CRDT synchronization
      description: Establishes real-time connection for bidirectional CRDT sync
      parameters:
        - name: room_id
          in: query
          required: false
          schema:
            type: string
            description: Optional room identifier for scoped synchronization
        - name: user_id  
          in: query
          required: false
          schema:
            type: string
            format: uuid
            description: User identifier for presence tracking
      responses:
        '101':
          description: WebSocket connection established
          headers:
            Upgrade:
              schema:
                type: string
                enum: [websocket]
            Connection:
              schema:
                type: string
                enum: [Upgrade]
        '400':
          description: Bad Request - Invalid parameters
        '403':
          description: Forbidden - Authentication required
        '429':
          description: Too Many Requests - Rate limit exceeded

components:
  schemas:
    CRDTMessage:
      $ref: './schemas/crdt-message.json'
    
    DeltaMessage:
      type: object
      properties:
        type:
          type: string
          enum: [delta]
        collection_id:
          type: string
          description: Unique identifier for the synchronized collection
        crdt_type:
          $ref: '#/components/schemas/CRDTType'
        delta:
          type: object
          description: CRDT-specific delta operation
          additionalProperties: true
        timestamp:
          type: string
          format: date-time
        replica_id:
          type: string
          format: uuid
      required: [type, collection_id, crdt_type, delta, timestamp, replica_id]

    CRDTType:
      type: string
      enum: 
        - lww_register
        - lww_map  
        - g_counter
        - rga
        - lseq
        - tree
        - graph
      description: Type of CRDT being synchronized

    HeartbeatMessage:
      type: object
      properties:
        type:
          type: string
          enum: [heartbeat]
        timestamp:
          type: string
          format: date-time
        replica_id:
          type: string
          format: uuid
        stats:
          type: object
          properties:
            messages_sent:
              type: integer
            messages_received:
              type: integer
            last_sync:
              type: string
              format: date-time
      required: [type, timestamp, replica_id]
```

### Phase 3: HTTP API Specification (Week 1)
**File**: `docs/api/rest-api.yaml` (< 250 lines)

```yaml
openapi: 3.0.3
info:
  title: Leptos-Sync REST API
  version: 0.8.4
  description: HTTP API for collection management and metadata

paths:
  /collections:
    get:
      summary: List collections
      responses:
        '200':
          description: List of collections
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/Collection'
    post:
      summary: Create collection
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateCollectionRequest'
      responses:
        '201':
          description: Collection created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Collection'

  /collections/{collection_id}:
    get:
      summary: Get collection details
      parameters:
        - name: collection_id
          in: path
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Collection details
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Collection'
        '404':
          description: Collection not found

  /collections/{collection_id}/export:
    get:
      summary: Export collection state
      parameters:
        - name: collection_id
          in: path
          required: true
          schema:
            type: string
        - name: format
          in: query
          schema:
            type: string
            enum: [json, binary]
            default: json
      responses:
        '200':
          description: Collection state export
          content:
            application/json:
              schema:
                type: object
            application/octet-stream:
              schema:
                type: string
                format: binary

components:
  schemas:
    Collection:
      type: object
      properties:
        id:
          type: string
        name:
          type: string
        crdt_type:
          $ref: '#/components/schemas/CRDTType'
        created_at:
          type: string
          format: date-time
        updated_at:
          type: string
          format: date-time
        replica_count:
          type: integer
        size_bytes:
          type: integer
      required: [id, name, crdt_type, created_at]

    CreateCollectionRequest:
      type: object
      properties:
        name:
          type: string
          minLength: 1
          maxLength: 100
        crdt_type:
          $ref: '#/components/schemas/CRDTType'
      required: [name, crdt_type]
```

### Phase 4: Contract Testing (Week 2)
**Files**: `tests/contracts/` (multiple files < 200 lines each)

#### Server Contract Tests
```rust
// tests/contracts/websocket_server_contract.rs (< 200 lines)
use serde_json::Value;
use jsonschema::{JSONSchema, Draft};
use leptos_sync_core::transport::SyncMessage;

#[tokio::test]
async fn test_delta_message_schema_compliance() {
    // Load JSON schema
    let schema_content = include_str!("../../docs/api/schemas/crdt-message.json");
    let schema: Value = serde_json::from_str(schema_content).unwrap();
    let compiled = JSONSchema::compile(&schema).unwrap();
    
    // Create test message
    let message = SyncMessage::Delta {
        collection_id: "test-collection".to_string(),
        crdt_type: CrdtType::LwwRegister,
        delta: vec![1, 2, 3, 4],
        timestamp: SystemTime::now(),
        replica_id: ReplicaId::new(),
    };
    
    // Serialize and validate against schema
    let json = serde_json::to_value(&message).unwrap();
    let result = compiled.validate(&json);
    
    if let Err(errors) = result {
        for error in errors {
            eprintln!("Schema validation error: {}", error);
        }
        panic!("Message does not comply with schema");
    }
}

#[tokio::test]
async fn test_websocket_server_accepts_valid_messages() {
    let server = start_test_server().await;
    let client = connect_websocket_client(&server.url()).await;
    
    // Test each message type from schema
    let test_cases = vec![
        create_test_delta_message(),
        create_test_heartbeat_message(), 
        create_test_peer_join_message(),
        create_test_peer_leave_message(),
    ];
    
    for message in test_cases {
        let result = client.send_message(message).await;
        assert!(result.is_ok(), "Server should accept valid message");
    }
}
```

#### Client Contract Tests  
```rust
// tests/contracts/client_contract.rs (< 200 lines)
use leptos_sync_core::client::LeptosSyncClient;

#[tokio::test]
async fn test_client_handles_all_server_messages() {
    let mock_server = MockWebSocketServer::new();
    
    // Configure server to send all message types from schema
    mock_server.send_delta_message().await;
    mock_server.send_heartbeat_message().await;
    mock_server.send_peer_join_message().await;
    mock_server.send_peer_leave_message().await;
    
    let client = LeptosSyncClient::connect(mock_server.url()).await.unwrap();
    
    // Verify client processes each message type correctly
    let received = client.collect_messages_for(Duration::from_secs(1)).await;
    assert_eq!(received.len(), 4);
    
    // Verify no errors or unhandled message types
    assert!(client.get_errors().is_empty());
}
```

### Phase 5: API Documentation Generation (Week 2)
**Setup**: Automated documentation pipeline

```yaml
# .github/workflows/api-docs.yml
name: API Documentation
on:
  push:
    branches: [main]
    paths: ['docs/api/**']

jobs:
  generate-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Generate OpenAPI docs
        uses: redocly/cli-action@v0.10.2
        with:
          args: 'build-docs docs/api/rest-api.yaml --output docs/generated/rest-api.html'
      - name: Generate WebSocket docs  
        uses: redocly/cli-action@v0.10.2
        with:
          args: 'build-docs docs/api/websocket-api.yaml --output docs/generated/websocket-api.html'
      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs/generated
```

## Schema Validation Integration

### Runtime Validation
```rust
// leptos-sync-core/src/validation/schema_validator.rs (< 200 lines)
use jsonschema::JSONSchema;
use serde_json::Value;

pub struct SchemaValidator {
    message_schema: JSONSchema,
}

impl SchemaValidator {
    pub fn new() -> Result<Self, ValidationError> {
        let schema_content = include_str!("../../../docs/api/schemas/crdt-message.json");
        let schema: Value = serde_json::from_str(schema_content)?;
        let message_schema = JSONSchema::compile(&schema)?;
        
        Ok(Self { message_schema })
    }
    
    pub fn validate_message(&self, message: &SyncMessage) -> Result<(), ValidationError> {
        let json = serde_json::to_value(message)?;
        self.message_schema.validate(&json)
            .map_err(|errors| ValidationError::SchemaViolation(errors.collect()))?;
        Ok(())
    }
}
```

### Development Mode Validation
```rust
// Only validate in debug builds to avoid performance impact
#[cfg(debug_assertions)]
fn send_message_with_validation(message: SyncMessage) -> Result<(), TransportError> {
    static VALIDATOR: OnceCell<SchemaValidator> = OnceCell::new();
    let validator = VALIDATOR.get_or_init(|| SchemaValidator::new().unwrap());
    
    validator.validate_message(&message)
        .map_err(|e| TransportError::InvalidMessage(e.to_string()))?;
    
    send_message_unchecked(message)
}

#[cfg(not(debug_assertions))]  
fn send_message_with_validation(message: SyncMessage) -> Result<(), TransportError> {
    send_message_unchecked(message)
}
```

## Acceptance Criteria

### API Documentation
- [ ] Complete OpenAPI 3.0 specifications for all APIs
- [ ] JSON Schema definitions for all message types
- [ ] Interactive documentation (Swagger UI) available
- [ ] Schema validation integrated into development builds
- [ ] API versioning strategy documented

### Contract Testing
- [ ] Server validates incoming messages against schemas
- [ ] Client handles all server message types correctly
- [ ] Contract tests run in CI/CD pipeline
- [ ] Version compatibility matrix maintained
- [ ] Breaking changes detected automatically

### Developer Experience
- [ ] Schema files under 300 lines each
- [ ] Clear error messages for schema violations
- [ ] IDE support with schema-based autocomplete
- [ ] Documentation automatically updated from schemas

## Time Estimate: 2 weeks
## Dependencies: WebSocket transport (02)
## Risk: Low - mostly documentation and validation work
## Benefits: Prevents integration issues, improves developer experience
