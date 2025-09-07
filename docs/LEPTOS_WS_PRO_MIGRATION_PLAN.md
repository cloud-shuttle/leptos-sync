# leptos-ws-pro Migration Plan

## Overview

This document outlines the Test-Driven Development (TDD) approach for migrating leptos-sync from the current stub WebSocket implementation to using `leptos-ws-pro`, a production-ready WebSocket library for Leptos applications.

## Current State Analysis

### Problems with Current Implementation

1. **Non-functional WebSocket Transport**: The current `WebSocketTransport` in `leptos-sync-core/src/transport/websocket.rs` is essentially a stub that only logs operations
2. **Disconnected Architecture**: The production WebSocket server exists separately from the transport layer
3. **Missing Features**: No reconnection logic, heartbeat mechanism, or proper error handling
4. **Limited Testing**: No comprehensive test coverage for WebSocket functionality

### Benefits of leptos-ws-pro

1. **Production-Ready**: Battle-tested WebSocket implementation with comprehensive features
2. **Leptos-Native**: Built specifically for Leptos with reactive integration
3. **Advanced Features**: Reconnection strategies, heartbeat, connection pooling, multiple transport protocols
4. **Type Safety**: RPC system with compile-time type checking
5. **Comprehensive Testing**: Extensive test suite and performance metrics

## Migration Strategy

### Phase 1: Foundation (✅ COMPLETED)

**Objective**: Set up the foundation for leptos-ws-pro integration

**Tasks Completed**:
- [x] Add `leptos-ws-pro = "0.2.0-beta"` to workspace dependencies
- [x] Create comprehensive TDD test suite (`leptos_ws_pro_tests.rs`)
- [x] Implement `LeptosWsProTransport` with basic functionality
- [x] Create compatibility layer for existing message protocol
- [x] Add proper error handling and type conversions

**Files Created/Modified**:
- `Cargo.toml` - Added leptos-ws-pro dependency
- `leptos-sync-core/Cargo.toml` - Added leptos-ws-pro dependency
- `leptos-sync-core/src/transport/leptos_ws_pro_tests.rs` - TDD test suite
- `leptos-sync-core/src/transport/leptos_ws_pro_transport.rs` - Core implementation
- `leptos-sync-core/src/transport/compatibility_layer.rs` - Protocol compatibility
- `leptos-sync-core/src/transport/mod.rs` - Module exports

### Phase 2: Integration Testing (🔄 IN PROGRESS)

**Objective**: Validate the integration works correctly

**Tasks**:
- [ ] Run the TDD test suite to verify implementation
- [ ] Fix any compilation errors or test failures
- [ ] Validate compatibility with existing message protocol
- [ ] Test error handling and edge cases

**Commands to Run**:
```bash
# Run the leptos-ws-pro integration tests
cargo test leptos_ws_pro_integration_tests

# Run all transport tests
cargo test -p leptos-sync-core transport

# Run with ignored tests (requires WebSocket server)
cargo test -- --ignored
```

### Phase 3: Real WebSocket Integration (📋 PLANNED)

**Objective**: Replace stub implementation with real leptos-ws-pro functionality

**Tasks**:
- [ ] Implement actual WebSocket connection using leptos-ws-pro APIs
- [ ] Add proper message handling and serialization
- [ ] Implement reconnection logic with exponential backoff
- [ ] Add heartbeat mechanism
- [ ] Integrate with Leptos reactive system

**Implementation Notes**:
```rust
// Example of real leptos-ws-pro integration
use leptos_ws_pro::*;

impl LeptosWsProTransport {
    async fn attempt_connection(&self) -> Result<(), LeptosWsProError> {
        // Create WebSocket context
        let ws_context = use_websocket(&self.config.url);
        
        // Set up message handlers
        let codec = JsonCodec::new();
        
        // Implement connection logic
        // ...
    }
}
```

### Phase 4: Server Compatibility (📋 PLANNED)

**Objective**: Ensure compatibility with existing WebSocket server

**Tasks**:
- [ ] Test compatibility with existing server in `leptos-sync/src/websocket_server.rs`
- [ ] Verify message protocol compatibility
- [ ] Test presence, heartbeat, and sync message handling
- [ ] Ensure binary message support

**Server Integration Points**:
- Welcome messages
- Presence updates
- Sync message broadcasting
- Heartbeat handling
- Binary message acknowledgments

### Phase 5: Hybrid Transport Integration (📋 PLANNED)

**Objective**: Integrate with existing hybrid transport system

**Tasks**:
- [ ] Update `HybridTransport` to use leptos-ws-pro as primary WebSocket transport
- [ ] Maintain fallback to in-memory transport
- [ ] Test transport switching logic
- [ ] Ensure backward compatibility

**Code Changes**:
```rust
// Update HybridTransport enum
#[derive(Clone)]
pub enum HybridTransport {
    LeptosWsPro(LeptosWsProTransport),  // New primary
    WebSocket(WebSocketTransport),       // Keep for compatibility
    InMemory(InMemoryTransport),
    Fallback {
        primary: LeptosWsProTransport,   // New primary
        fallback: InMemoryTransport,
    },
}
```

### Phase 6: Performance Optimization (📋 PLANNED)

**Objective**: Optimize performance and add advanced features

**Tasks**:
- [ ] Implement connection pooling
- [ ] Add message batching
- [ ] Optimize serialization/deserialization
- [ ] Add compression support
- [ ] Implement message queuing for offline scenarios

### Phase 7: Migration and Cleanup (📋 PLANNED)

**Objective**: Complete migration and remove old code

**Tasks**:
- [ ] Update all examples to use new transport
- [ ] Update documentation
- [ ] Remove old stub WebSocket implementation
- [ ] Clean up unused dependencies
- [ ] Update version and release notes

## Testing Strategy

### Unit Tests
- Transport creation and configuration
- Connection establishment and failure handling
- Message sending and receiving
- Error handling and edge cases
- Reconnection logic

### Integration Tests
- Full WebSocket server integration
- Message protocol compatibility
- Hybrid transport functionality
- Performance characteristics

### End-to-End Tests
- Complete sync workflow
- Multi-client scenarios
- Network failure recovery
- Offline/online transitions

## Risk Mitigation

### Technical Risks
1. **API Compatibility**: leptos-ws-pro API changes
   - **Mitigation**: Pin to specific version, comprehensive testing
2. **Performance Regression**: Slower than expected performance
   - **Mitigation**: Benchmarking, performance tests
3. **Breaking Changes**: Incompatible with existing code
   - **Mitigation**: Compatibility layer, gradual migration

### Project Risks
1. **Timeline Delays**: Migration takes longer than expected
   - **Mitigation**: Phased approach, parallel development
2. **Team Learning Curve**: Team needs to learn new APIs
   - **Mitigation**: Documentation, examples, training

## Success Criteria

### Functional Requirements
- [ ] All existing WebSocket functionality works with leptos-ws-pro
- [ ] Message protocol compatibility maintained
- [ ] Server integration works seamlessly
- [ ] Hybrid transport system updated

### Non-Functional Requirements
- [ ] Performance equal or better than current implementation
- [ ] Comprehensive test coverage (>90%)
- [ ] Documentation updated
- [ ] No breaking changes for existing users

### Quality Metrics
- [ ] All TDD tests pass
- [ ] No compilation warnings
- [ ] Clippy lints resolved
- [ ] Security audit clean

## Timeline

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1: Foundation | ✅ Complete | None |
| Phase 2: Integration Testing | 1-2 days | Phase 1 |
| Phase 3: Real WebSocket Integration | 3-5 days | Phase 2 |
| Phase 4: Server Compatibility | 2-3 days | Phase 3 |
| Phase 5: Hybrid Transport Integration | 2-3 days | Phase 4 |
| Phase 6: Performance Optimization | 3-5 days | Phase 5 |
| Phase 7: Migration and Cleanup | 2-3 days | Phase 6 |

**Total Estimated Duration**: 2-3 weeks

## Next Steps

1. **Immediate**: Run the TDD test suite to validate current implementation
2. **Short-term**: Implement real WebSocket functionality in Phase 3
3. **Medium-term**: Complete server integration and hybrid transport updates
4. **Long-term**: Performance optimization and full migration

## Resources

- [leptos-ws-pro Documentation](https://docs.rs/leptos-ws-pro/latest/leptos_ws_pro/)
- [leptos-ws-pro GitHub Repository](https://github.com/leptos-rs/leptos-ws-pro)
- [Leptos WebSocket Guide](https://leptos.dev/guide/websockets.html)
- [Current WebSocket Server Implementation](./leptos-sync/src/websocket_server.rs)

## Conclusion

This migration plan provides a structured, test-driven approach to integrating leptos-ws-pro into leptos-sync. The phased approach minimizes risk while ensuring comprehensive testing and validation at each step. The compatibility layer ensures existing functionality continues to work during the migration process.

The foundation has been laid with comprehensive tests and basic implementation. The next step is to validate the current implementation and proceed with real WebSocket integration.
