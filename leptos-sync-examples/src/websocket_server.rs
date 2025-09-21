//! WebSocket server implementation for leptos-sync
//! 
//! This module provides a reference WebSocket server that can handle
//! multiple clients and broadcast CRDT synchronization messages.

use leptos_sync_core::transport::message_protocol::{SyncMessage, MessageCodec, UserInfo, PresenceAction};
use leptos_sync_core::crdt::ReplicaId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, broadcast};
use tokio::time::{interval, timeout};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum WebSocketServerError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Message error: {0}")]
    MessageError(String),
    #[error("Room error: {0}")]
    RoomError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Configuration for the WebSocket server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: Option<usize>,
    pub heartbeat_interval: Duration,
    pub connection_timeout: Duration,
    pub message_timeout: Duration,
    pub enable_compression: bool,
    pub enable_rate_limiting: bool,
    pub rate_limit_per_minute: usize,
}

impl Default for WebSocketServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3001,
            max_connections: Some(1000),
            heartbeat_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(60),
            message_timeout: Duration::from_secs(10),
            enable_compression: false,
            enable_rate_limiting: true,
            rate_limit_per_minute: 100,
        }
    }
}

/// Information about a connected client
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub replica_id: ReplicaId,
    pub user_info: Option<UserInfo>,
    pub connected_at: SystemTime,
    pub last_heartbeat: SystemTime,
    pub message_count: usize,
    pub room_id: Option<String>,
}

/// Information about a room/channel
#[derive(Debug, Clone)]
pub struct RoomInfo {
    pub id: String,
    pub name: String,
    pub clients: Vec<ReplicaId>,
    pub created_at: SystemTime,
    pub max_clients: Option<usize>,
}

/// WebSocket server for handling CRDT synchronization
pub struct WebSocketServer {
    config: WebSocketServerConfig,
    clients: Arc<RwLock<HashMap<ReplicaId, ClientInfo>>>,
    rooms: Arc<RwLock<HashMap<String, RoomInfo>>>,
    message_broadcaster: broadcast::Sender<SyncMessage>,
    server_info: ServerInfo,
}

#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub version: String,
    pub max_connections: Option<usize>,
    pub features: Vec<String>,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub fn new(config: WebSocketServerConfig) -> Self {
        let (message_broadcaster, _) = broadcast::channel(1000);
        
        let server_info = ServerInfo {
            version: "1.0.0".to_string(),
            max_connections: config.max_connections,
            features: vec![
                "crdt-sync".to_string(),
                "presence".to_string(),
                "rooms".to_string(),
            ],
        };

        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
            message_broadcaster,
            server_info,
        }
    }

    /// Start the WebSocket server
    pub async fn start(&self) -> Result<(), WebSocketServerError> {
        tracing::info!(
            "Starting WebSocket server on {}:{}",
            self.config.host,
            self.config.port
        );

        // Start background tasks
        self.start_heartbeat_monitor().await;
        self.start_cleanup_task().await;

        // TODO: Implement actual WebSocket server using a WebSocket library
        // For now, simulate server startup
        tracing::info!("WebSocket server started successfully");
        Ok(())
    }

    /// Stop the WebSocket server
    pub async fn stop(&self) -> Result<(), WebSocketServerError> {
        tracing::info!("Stopping WebSocket server");
        
        // Disconnect all clients
        let mut clients = self.clients.write().await;
        for (replica_id, _) in clients.iter() {
            self.send_peer_leave(*replica_id).await;
        }
        clients.clear();
        
        tracing::info!("WebSocket server stopped");
        Ok(())
    }

    /// Handle a new client connection
    pub async fn handle_client_connection(
        &self,
        replica_id: ReplicaId,
        user_info: Option<UserInfo>,
    ) -> Result<(), WebSocketServerError> {
        // Check connection limits
        if let Some(max_connections) = self.config.max_connections {
            let client_count = self.clients.read().await.len();
            if client_count >= max_connections {
                return Err(WebSocketServerError::ConnectionError(
                    "Server at maximum capacity".to_string(),
                ));
            }
        }

        // Add client to registry
        let client_info = ClientInfo {
            replica_id,
            user_info: user_info.clone(),
            connected_at: SystemTime::now(),
            last_heartbeat: SystemTime::now(),
            message_count: 0,
            room_id: None,
        };

        self.clients.write().await.insert(replica_id, client_info);

        // Send welcome message
        self.send_welcome_message(replica_id).await?;

        // Broadcast peer join
        self.broadcast_message(SyncMessage::PeerJoin {
            replica_id,
            user_info,
        }).await?;

        tracing::info!("Client connected: {:?}", replica_id);
        Ok(())
    }

    /// Handle client disconnection
    pub async fn handle_client_disconnection(&self, replica_id: ReplicaId) -> Result<(), WebSocketServerError> {
        // Remove from rooms
        if let Some(client_info) = self.clients.read().await.get(&replica_id) {
            if let Some(room_id) = &client_info.room_id {
                self.remove_client_from_room(replica_id, room_id.clone()).await?;
            }
        }

        // Remove client
        self.clients.write().await.remove(&replica_id);

        // Broadcast peer leave
        self.broadcast_message(SyncMessage::PeerLeave { replica_id }).await?;

        tracing::info!("Client disconnected: {:?}", replica_id);
        Ok(())
    }

    /// Handle incoming message from client
    pub async fn handle_client_message(
        &self,
        replica_id: ReplicaId,
        message: SyncMessage,
    ) -> Result<(), WebSocketServerError> {
        // Update client info
        if let Some(client_info) = self.clients.write().await.get_mut(&replica_id) {
            client_info.last_heartbeat = SystemTime::now();
            client_info.message_count += 1;
        }

        match message {
            SyncMessage::Heartbeat { .. } => {
                // Heartbeat received, no action needed
            }
            SyncMessage::Delta { .. } => {
                // Broadcast delta to all clients in the same room
                self.broadcast_delta_message(replica_id, message).await?;
            }
            SyncMessage::Presence { action, .. } => {
                // Handle presence updates
                self.handle_presence_update(replica_id, action).await?;
            }
            _ => {
                // For other message types, just broadcast
                self.broadcast_message(message).await?;
            }
        }

        Ok(())
    }

    /// Get server statistics
    pub async fn get_stats(&self) -> ServerStats {
        let clients = self.clients.read().await;
        let rooms = self.rooms.read().await;

        ServerStats {
            connected_clients: clients.len(),
            total_rooms: rooms.len(),
            uptime: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default(),
            total_messages: clients.values().map(|c| c.message_count).sum(),
        }
    }

    // Private methods

    async fn send_welcome_message(&self, replica_id: ReplicaId) -> Result<(), WebSocketServerError> {
        let welcome = SyncMessage::Welcome {
            peer_id: replica_id,
            timestamp: SystemTime::now(),
            server_info: Some(leptos_sync_core::transport::message_protocol::ServerInfo {
                version: self.server_info.version.clone(),
                max_connections: self.server_info.max_connections,
                features: self.server_info.features.clone(),
            }),
        };

        // TODO: Send message to specific client
        tracing::debug!("Sent welcome message to {:?}", replica_id);
        Ok(())
    }

    async fn send_peer_leave(&self, replica_id: ReplicaId) {
        let leave_message = SyncMessage::PeerLeave { replica_id };
        let _ = self.broadcast_message(leave_message).await;
    }

    async fn broadcast_message(&self, message: SyncMessage) -> Result<(), WebSocketServerError> {
        let serialized = MessageCodec::serialize(&message)
            .map_err(|e| WebSocketServerError::SerializationError(e.to_string()))?;

        // TODO: Broadcast to all connected clients
        tracing::debug!("Broadcasting message: {:?}", message);
        Ok(())
    }

    async fn broadcast_delta_message(
        &self,
        sender_id: ReplicaId,
        message: SyncMessage,
    ) -> Result<(), WebSocketServerError> {
        // Get sender's room
        let sender_room = {
            let clients = self.clients.read().await;
            clients.get(&sender_id).and_then(|c| c.room_id.clone())
        };

        if let Some(room_id) = sender_room {
            // Broadcast to all clients in the same room
            let room_clients = {
                let rooms = self.rooms.read().await;
                rooms.get(&room_id).map(|r| r.clients.clone()).unwrap_or_default()
            };

            for client_id in room_clients {
                if client_id != sender_id {
                    // TODO: Send message to specific client
                    tracing::debug!("Sending delta to client {:?} in room {}", client_id, room_id);
                }
            }
        } else {
            // Broadcast to all clients if not in a room
            self.broadcast_message(message).await?;
        }

        Ok(())
    }

    async fn handle_presence_update(
        &self,
        replica_id: ReplicaId,
        action: PresenceAction,
    ) -> Result<(), WebSocketServerError> {
        let presence_message = SyncMessage::Presence {
            peer_id: replica_id,
            action,
            timestamp: SystemTime::now(),
        };

        self.broadcast_message(presence_message).await?;
        Ok(())
    }

    async fn remove_client_from_room(
        &self,
        replica_id: ReplicaId,
        room_id: String,
    ) -> Result<(), WebSocketServerError> {
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            room.clients.retain(|&id| id != replica_id);
            
            // Remove room if empty
            if room.clients.is_empty() {
                rooms.remove(&room_id);
            }
        }
        Ok(())
    }

    async fn start_heartbeat_monitor(&self) {
        let clients = self.clients.clone();
        let config = self.config.clone();
        let message_broadcaster = self.message_broadcaster.clone();

        tokio::spawn(async move {
            let mut interval = interval(config.heartbeat_interval);

            loop {
                interval.tick().await;

                let now = SystemTime::now();
                let timeout_duration = config.connection_timeout;

                // Check for stale connections
                let stale_clients: Vec<ReplicaId> = {
                    let clients_guard = clients.read().await;
                    clients_guard
                        .iter()
                        .filter(|(_, info)| {
                            now.duration_since(info.last_heartbeat)
                                .unwrap_or_default()
                                > timeout_duration
                        })
                        .map(|(id, _)| *id)
                        .collect()
                };

                // Remove stale clients
                for replica_id in stale_clients {
                    tracing::warn!("Removing stale client: {:?}", replica_id);
                    clients.write().await.remove(&replica_id);
                    
                    // Broadcast peer leave
                    let leave_message = SyncMessage::PeerLeave { replica_id };
                    let _ = message_broadcaster.send(leave_message);
                }
            }
        });
    }

    async fn start_cleanup_task(&self) {
        let rooms = self.rooms.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Cleanup every minute

            loop {
                interval.tick().await;

                // Remove empty rooms
                let mut rooms_guard = rooms.write().await;
                rooms_guard.retain(|_, room| !room.clients.is_empty());
            }
        });
    }
}

/// Server statistics
#[derive(Debug, Clone)]
pub struct ServerStats {
    pub connected_clients: usize,
    pub total_rooms: usize,
    pub uptime: Duration,
    pub total_messages: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos_sync_core::crdt::ReplicaId;

    fn create_test_replica_id() -> ReplicaId {
        ReplicaId::from(uuid::Uuid::new_v4())
    }

    #[tokio::test]
    async fn test_server_creation() {
        let config = WebSocketServerConfig::default();
        let server = WebSocketServer::new(config);
        
        let stats = server.get_stats().await;
        assert_eq!(stats.connected_clients, 0);
        assert_eq!(stats.total_rooms, 0);
    }

    #[tokio::test]
    async fn test_client_connection() {
        let config = WebSocketServerConfig::default();
        let server = WebSocketServer::new(config);
        
        let replica_id = create_test_replica_id();
        let user_info = UserInfo {
            user_id: "user123".to_string(),
            username: Some("testuser".to_string()),
            display_name: Some("Test User".to_string()),
            avatar_url: None,
        };

        let result = server.handle_client_connection(replica_id, Some(user_info)).await;
        assert!(result.is_ok());

        let stats = server.get_stats().await;
        assert_eq!(stats.connected_clients, 1);
    }

    #[tokio::test]
    async fn test_client_disconnection() {
        let config = WebSocketServerConfig::default();
        let server = WebSocketServer::new(config);
        
        let replica_id = create_test_replica_id();
        
        // Connect client
        server.handle_client_connection(replica_id, None).await.unwrap();
        assert_eq!(server.get_stats().await.connected_clients, 1);
        
        // Disconnect client
        let result = server.handle_client_disconnection(replica_id).await;
        assert!(result.is_ok());
        assert_eq!(server.get_stats().await.connected_clients, 0);
    }

    #[tokio::test]
    async fn test_heartbeat_message() {
        let config = WebSocketServerConfig::default();
        let server = WebSocketServer::new(config);
        
        let replica_id = create_test_replica_id();
        server.handle_client_connection(replica_id, None).await.unwrap();
        
        let heartbeat = SyncMessage::Heartbeat {
            replica_id,
            timestamp: SystemTime::now(),
        };
        
        let result = server.handle_client_message(replica_id, heartbeat).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connection_limit() {
        let config = WebSocketServerConfig {
            max_connections: Some(1),
            ..Default::default()
        };
        let server = WebSocketServer::new(config);
        
        let replica_id1 = create_test_replica_id();
        let replica_id2 = create_test_replica_id();
        
        // First connection should succeed
        let result1 = server.handle_client_connection(replica_id1, None).await;
        assert!(result1.is_ok());
        
        // Second connection should fail
        let result2 = server.handle_client_connection(replica_id2, None).await;
        assert!(result2.is_err());
    }
}
