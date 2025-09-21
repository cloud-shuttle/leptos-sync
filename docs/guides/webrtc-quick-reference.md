# WebRTC Integration Quick Reference

## 🚀 Quick Start

### Basic WebRTC Setup
```rust
use leptos_sync_core::{LocalFirstCollection, HybridTransport};
use leptos_sync_webrtc::WebRTCTransport;
use leptos_sync_core::transport::LeptosWsProTransport;

// 1. Create WebRTC transport
let webrtc_transport = WebRTCTransport::new()
    .with_stun_servers(vec!["stun:stun.l.google.com:19302"])
    .with_turn_servers(vec![/* TURN servers */]);

// 2. Create WebSocket fallback
let websocket_transport = LeptosWsProTransport::new();

// 3. Create hybrid transport
let hybrid_transport = HybridTransport::new()
    .with_primary(webrtc_transport)
    .with_fallback(websocket_transport);

// 4. Use with collection
let collection = LocalFirstCollection::<TodoItem>::new(
    "todos".to_string(),
    storage,
    hybrid_transport
);
```

## 📦 Package Dependencies

### Main Project
```toml
[dependencies]
leptos-sync-core = "0.8.4"
leptos-sync-webrtc = "0.1.0"  # Optional WebRTC support
```

### WebRTC Repository
```toml
[dependencies]
leptos-sync-core = "0.8.4"
webrtc = "0.7"
tokio = { version = "1.0", features = ["rt", "net", "time"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
```

## 🔧 Configuration Options

### WebRTC Configuration
```rust
let config = WebRTCConfig::new()
    .with_stun_servers(vec![
        "stun:stun.l.google.com:19302",
        "stun:stun1.l.google.com:19302"
    ])
    .with_turn_servers(vec![
        TurnServer {
            urls: vec!["turn:turn.server.com:3478"],
            username: "user".to_string(),
            credential: "pass".to_string(),
        }
    ])
    .with_ice_policy(IceTransportPolicy::All)
    .with_bundle_policy(BundlePolicy::Balanced);
```

### Hybrid Transport Configuration
```rust
let hybrid = HybridTransport::new()
    .with_primary(webrtc_transport)
    .with_fallback(websocket_transport)
    .with_fallback_threshold(Duration::from_secs(5))
    .with_quality_monitoring(true);
```

## 🌐 STUN/TURN Servers

### Free STUN Servers
```rust
let free_stun_servers = vec![
    "stun:stun.l.google.com:19302",
    "stun:stun1.l.google.com:19302",
    "stun:stun2.l.google.com:19302",
    "stun:stun3.l.google.com:19302",
    "stun:stun4.l.google.com:19302",
];
```

### Commercial TURN Servers
- **Twilio**: `turn:global.turn.twilio.com:3478`
- **Xirsys**: `turn:turn.xirsys.com:80`
- **Open Relay Project**: `turn:openrelay.metered.ca:80`

## 🔒 Security Configuration

### Encryption Setup
```rust
let encrypted_transport = EncryptedWebRTCTransport::new(
    webrtc_transport,
    encryption_key
);
```

### Authentication
```rust
let authenticated_transport = AuthenticatedWebRTCTransport::new(
    webrtc_transport,
    auth_token,
    token_validator
);
```

## 🧪 Testing

### Unit Tests
```rust
#[tokio::test]
async fn test_webrtc_connection() {
    let transport = WebRTCTransport::new()
        .with_stun_servers(vec!["stun:stun.l.google.com:19302"]);
    
    assert!(transport.connect().await.is_ok());
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_hybrid_fallback() {
    let webrtc = WebRTCTransport::new();
    let websocket = LeptosWsProTransport::new();
    let hybrid = HybridTransport::new()
        .with_primary(webrtc)
        .with_fallback(websocket);
    
    // Test fallback behavior
}
```

## 📊 Monitoring

### Metrics Collection
```rust
let metrics = WebRTCMetrics::new();
metrics.record_connection();
metrics.record_message_sent(1024);
metrics.record_latency(Duration::from_millis(45));
```

### Logging
```rust
use tracing::{info, warn, error};

info!("WebRTC connection established");
warn!("High latency detected: {}ms", latency);
error!("Connection failed: {}", error);
```

## 🚨 Error Handling

### Common Errors
```rust
match transport.connect().await {
    Ok(_) => info!("Connected successfully"),
    Err(TransportError::WebRTC(WebRTCError::PeerConnectionFailed(msg))) => {
        error!("Peer connection failed: {}", msg);
        // Try fallback transport
    }
    Err(TransportError::WebRTC(WebRTCError::IceGatheringFailed(msg))) => {
        error!("ICE gathering failed: {}", msg);
        // Check STUN/TURN configuration
    }
    Err(e) => error!("Unexpected error: {}", e),
}
```

### Recovery Strategies
```rust
impl WebRTCTransport {
    async fn connect_with_retry(&self, max_retries: usize) -> Result<(), TransportError> {
        for attempt in 1..=max_retries {
            match self.connect().await {
                Ok(_) => return Ok(()),
                Err(e) if attempt == max_retries => return Err(e),
                Err(e) => {
                    warn!("Connection attempt {} failed: {}", attempt, e);
                    tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt))).await;
                }
            }
        }
        unreachable!()
    }
}
```

## 🔄 Migration Guide

### From WebSocket Only
```rust
// Before
let transport = LeptosWsProTransport::new();
let collection = LocalFirstCollection::<TodoItem>::new(
    "todos".to_string(),
    storage,
    transport
);

// After
let webrtc_transport = WebRTCTransport::new()
    .with_stun_servers(vec!["stun:stun.l.google.com:19302"]);
let websocket_transport = LeptosWsProTransport::new();
let hybrid_transport = HybridTransport::new()
    .with_primary(webrtc_transport)
    .with_fallback(websocket_transport);

let collection = LocalFirstCollection::<TodoItem>::new(
    "todos".to_string(),
    storage,
    hybrid_transport
);
```

## 📚 Common Patterns

### Peer-to-Peer Collaboration
```rust
// Create peer connection
let peer_connection = webrtc_transport.create_peer_connection().await?;

// Create data channel
let data_channel = peer_connection.create_data_channel("sync").await?;

// Handle incoming data
data_channel.on_message(|data| {
    // Process incoming synchronization data
    collection.apply_remote_operation(data);
});

// Send local changes
collection.on_change(|change| {
    data_channel.send(change);
});
```

### Multi-Peer Mesh
```rust
let mut peer_connections = HashMap::new();

// Connect to multiple peers
for peer_id in peer_ids {
    let connection = webrtc_transport.connect_to_peer(peer_id).await?;
    peer_connections.insert(peer_id, connection);
}

// Broadcast changes to all peers
collection.on_change(|change| {
    for (peer_id, connection) in &peer_connections {
        connection.send(change.clone());
    }
});
```

## 🎯 Performance Tips

### Optimization
```rust
// Use binary data channels for better performance
let config = DataChannelConfig {
    ordered: true,
    max_retransmits: Some(3),
    max_retransmit_time: Some(Duration::from_secs(5)),
    protocol: "binary".to_string(),
};

// Enable compression
let transport = WebRTCTransport::new()
    .with_compression(true)
    .with_compression_level(6);

// Optimize ICE gathering
let config = WebRTCConfig::new()
    .with_ice_transport_policy(IceTransportPolicy::All)
    .with_bundle_policy(BundlePolicy::MaxBundle);
```

### Monitoring
```rust
// Monitor connection quality
let quality_monitor = ConnectionQualityMonitor::new();
quality_monitor.on_quality_change(|quality| {
    match quality {
        ConnectionQuality::Excellent => {
            // Use high-quality settings
        }
        ConnectionQuality::Poor => {
            // Use low-quality settings or fallback
        }
    }
});
```

## 🔧 Troubleshooting

### Common Issues

#### Connection Fails
```rust
// Check STUN/TURN configuration
let config = WebRTCConfig::new()
    .with_stun_servers(vec!["stun:stun.l.google.com:19302"])
    .with_turn_servers(vec![/* TURN servers */]);

// Enable debugging
env_logger::init();
```

#### High Latency
```rust
// Use local STUN servers
let config = WebRTCConfig::new()
    .with_stun_servers(vec!["stun:local.stun.server:3478"]);

// Optimize data channel settings
let data_channel_config = DataChannelConfig {
    ordered: false,  // Allow out-of-order delivery
    max_retransmits: Some(0),  // Disable retransmission
};
```

#### NAT Traversal Issues
```rust
// Use TURN servers for strict NAT
let turn_servers = vec![
    TurnServer {
        urls: vec!["turn:turn.server.com:3478"],
        username: "user".to_string(),
        credential: "pass".to_string(),
    }
];
```

## 📖 Additional Resources

- [WebRTC Integration Strategy](./webrtc-integration-strategy.md)
- [Technical Decision Matrix](./webrtc-technical-decision-matrix.md)
- [WebRTC.org Documentation](https://webrtc.org/)
- [MDN WebRTC API](https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API)
- [Rust WebRTC Library](https://github.com/webrtc-rs/webrtc)

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/leptos-sync/leptos-sync-webrtc/issues)
- **Discussions**: [GitHub Discussions](https://github.com/leptos-sync/leptos-sync-webrtc/discussions)
- **Documentation**: [Docs](https://docs.leptos-sync.dev/webrtc)
- **Examples**: [Examples Repository](https://github.com/leptos-sync/leptos-sync-webrtc/tree/main/examples)
