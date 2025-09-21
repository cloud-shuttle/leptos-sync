# Leptos-Sync API Documentation

This directory contains the formal API specifications and schemas for the Leptos-Sync library.

## Overview

Leptos-Sync provides two main APIs:

1. **WebSocket API** - Real-time synchronization for CRDT operations
2. **REST API** - Collection management and metadata operations

## Files

### API Specifications

- [`websocket-api.yaml`](./websocket-api.yaml) - OpenAPI 3.0 specification for WebSocket API
- [`rest-api.yaml`](./rest-api.yaml) - OpenAPI 3.0 specification for REST API

### Schemas

- [`schemas/crdt-message.json`](./schemas/crdt-message.json) - JSON Schema for CRDT messages

## Contract Testing

The API contracts are validated through comprehensive test suites:

- **Server Contract Tests** - Validate server implementation against schemas
- **Client Contract Tests** - Validate client handling of all message types
- **Schema Compliance Tests** - Ensure all messages conform to JSON schemas

## Usage

### WebSocket API

The WebSocket API enables real-time bidirectional synchronization:

```javascript
const ws = new WebSocket('ws://localhost:3001/sync?room_id=document-123');

ws.onopen = () => {
  // Send a CRDT delta
  const deltaMessage = {
    type: 'delta',
    version: '0.8.4',
    timestamp: new Date().toISOString(),
    replica_id: '550e8400-e29b-41d4-a716-446655440000',
    collection_id: 'document-123',
    crdt_type: 'lww_register',
    delta: {
      operation: 'set',
      value: 'Hello, World!'
    }
  };
  
  ws.send(JSON.stringify(deltaMessage));
};
```

### REST API

The REST API provides collection management:

```bash
# List collections
curl -H "Authorization: Bearer <token>" \
     http://localhost:3000/api/v1/collections

# Create a new collection
curl -X POST \
     -H "Authorization: Bearer <token>" \
     -H "Content-Type: application/json" \
     -d '{"name": "My Document", "crdt_type": "lww_register"}' \
     http://localhost:3000/api/v1/collections
```

## Message Types

### Delta Message
Used for CRDT synchronization operations.

### Heartbeat Message
Used for connection health monitoring.

### Peer Join/Leave Messages
Used for presence tracking.

### Welcome Message
Sent by server to new clients.

### Presence Message
Used for peer status updates.

### Binary Acknowledgment
Used for binary data transfer confirmation.

## CRDT Types

The following CRDT types are supported:

- `lww_register` - Last-Write-Wins Register
- `lww_map` - Last-Write-Wins Map
- `g_counter` - Grow-only Counter
- `rga` - Replicated Growable Array
- `lseq` - Logoot Sequence
- `tree` - Tree CRDT
- `graph` - Graph CRDT

## Validation

All messages are validated against JSON schemas to ensure contract compliance. Validation is enabled in development builds and can be disabled in production for performance.

## Versioning

The API uses semantic versioning. The current version is 0.8.4.

## Documentation Generation

API documentation is automatically generated from the OpenAPI specifications and deployed to GitHub Pages when changes are pushed to the main branch.

## Contributing

When making changes to the API:

1. Update the relevant OpenAPI specification
2. Update the JSON schema if message structure changes
3. Run contract tests to ensure compatibility
4. Update this README if needed

## Links

- [Generated Documentation](https://leptos-sync.github.io/leptos-sync/)
- [Contract Tests](../tests/contracts/)
- [Schema Validation](../leptos-sync-core/src/validation/)
