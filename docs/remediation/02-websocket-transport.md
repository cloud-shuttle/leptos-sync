# WebSocket Transport Implementation - Critical Priority

## Overview
Implement functional WebSocket transport to enable actual synchronization between clients.

## Current State
- `WebSocketTransport` exists but only logs messages
- `leptos_ws_pro_transport` module is mostly empty
- No real network communication happening
- Integration tests exist but can't verify real sync

## Design Requirements

### Core Functionality
- Establish WebSocket connections via leptos-ws-pro
- Send/receive CRDT delta messages between peers
- Handle connection lifecycle (connect, reconnect, disconnect)
- Message serialization/deserialization
- Error handling and retry logic

### Message Protocol
```rust
#[derive(Serialize, Deserialize, Debug)]
pub enum SyncMessage {
    Delta { 
        collection_id: String,
        crdt_type: CrdtType,
        delta: Vec<u8>,
        timestamp: SystemTime,
        replica_id: ReplicaId,
    },
    Heartbeat {
        replica_id: ReplicaId,
        timestamp: SystemTime,
    },
    PeerJoin {
        replica_id: ReplicaId,
        user_info: Option<UserInfo>,
    },
    PeerLeave {
        replica_id: ReplicaId,
    },
}
```

## Implementation Plan

### Phase 1: Basic Transport (Week 1)
**File**: `leptos-sync-core/src/transport/websocket_transport.rs` (< 300 lines)

Core WebSocket implementation:
- Connection establishment via leptos-ws-pro
- Send/receive raw messages  
- Basic error handling
- Connection state management

### Phase 2: Message Protocol (Week 1)
**File**: `leptos-sync-core/src/transport/message_protocol.rs` (< 250 lines)

Message handling:
- Serialize/deserialize `SyncMessage`
- Message routing and validation
- Protocol versioning support
- Compression (if enabled)

### Phase 3: Integration (Week 2)  
**File**: `leptos-sync-core/src/transport/websocket_integration.rs` (< 200 lines)

Connect transport to sync engine:
- Delta message forwarding
- Peer lifecycle management
- Reconnection logic
- Transport statistics

### Phase 4: Server Implementation (Week 2)
**File**: `leptos-sync-examples/src/websocket_server.rs` (< 300 lines)

Reference WebSocket server:
- Message broadcasting to peers
- Room/channel management
- Connection pooling
- Basic rate limiting

## Breaking Down Large Files

### Current Issues
- `websocket_server.rs` already 496 lines - needs splitting
- Mixing connection logic with message handling

### Proposed Structure
```
transport/websocket/
├── client.rs          (< 250 lines) - Client connection logic
├── server.rs          (< 250 lines) - Server implementation  
├── protocol.rs        (< 200 lines) - Message protocol
├── connection.rs      (< 200 lines) - Connection management
├── errors.rs          (< 150 lines) - Error types
└── mod.rs            (< 100 lines) - Public API
```

## Testing Strategy

### Unit Tests
- Message serialization/deserialization
- Connection state transitions
- Error handling scenarios
- Mock transport for deterministic testing

### Integration Tests
- Two-client sync via WebSocket
- Server broadcasting messages
- Connection failure/recovery
- Message ordering guarantees

### Browser Tests
- WASM WebSocket client
- Cross-browser compatibility
- Network condition simulation

## API Contract

### Client Configuration
```rust
pub struct WebSocketConfig {
    pub url: String,
    pub reconnect_attempts: u32,
    pub heartbeat_interval: Duration,
    pub message_timeout: Duration,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:3001/sync".to_string(),
            reconnect_attempts: 5,
            heartbeat_interval: Duration::from_secs(30),
            message_timeout: Duration::from_secs(10),
        }
    }
}
```

### Transport Interface
```rust
#[async_trait]
impl Transport for WebSocketTransport {
    async fn send(&self, message: SyncMessage) -> Result<(), TransportError>;
    async fn receive(&mut self) -> Result<SyncMessage, TransportError>;
    fn is_connected(&self) -> bool;
    async fn connect(&mut self) -> Result<(), TransportError>;
    async fn disconnect(&mut self) -> Result<(), TransportError>;
}
```

## Acceptance Criteria
- [ ] Two browser tabs can sync CRDT changes via WebSocket
- [ ] Connection recovery works after network interruption
- [ ] Message protocol handles all CRDT types (LWW, GCounter, etc.)
- [ ] Server can handle multiple concurrent clients
- [ ] Integration tests demonstrate end-to-end sync
- [ ] All files under 300 lines
- [ ] Comprehensive error handling

## Time Estimate: 2 weeks
## Dependencies: Compilation fixes (01)
## Risk: Medium - network programming complexity
