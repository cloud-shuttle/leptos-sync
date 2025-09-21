//! Test server utilities
//! 
//! Provides mock WebSocket server and test utilities for integration testing.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Mock WebSocket server for testing
pub struct MockWebSocketServer {
    pub url: String,
    pub port: u16,
    pub clients: Arc<RwLock<HashMap<String, MockClient>>>,
    pub messages: Arc<RwLock<Vec<ServerMessage>>>,
}

/// Mock client connection
pub struct MockClient {
    pub id: String,
    pub connected: bool,
    pub messages: Vec<ClientMessage>,
}

/// Message from client to server
#[derive(Debug, Clone)]
pub struct ClientMessage {
    pub client_id: String,
    pub message_type: String,
    pub data: Vec<u8>,
    pub timestamp: std::time::SystemTime,
}

/// Message from server to client
#[derive(Debug, Clone)]
pub struct ServerMessage {
    pub client_id: Option<String>,
    pub message_type: String,
    pub data: Vec<u8>,
    pub timestamp: std::time::SystemTime,
}

impl MockWebSocketServer {
    /// Create a new mock WebSocket server
    pub fn new(port: u16) -> Self {
        Self {
            url: format!("ws://localhost:{}", port),
            port,
            clients: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Start the mock server
    pub async fn start(&self) -> Result<(), String> {
        // In a real implementation, this would start an actual WebSocket server
        // For now, we'll just simulate the server being ready
        Ok(())
    }
    
    /// Stop the mock server
    pub async fn stop(&self) -> Result<(), String> {
        // In a real implementation, this would stop the WebSocket server
        // For now, we'll just simulate the server being stopped
        Ok(())
    }
    
    /// Get the server URL
    pub fn url(&self) -> &str {
        &self.url
    }
    
    /// Connect a mock client
    pub async fn connect_client(&self, client_id: Option<String>) -> String {
        let id = client_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        
        let client = MockClient {
            id: id.clone(),
            connected: true,
            messages: Vec::new(),
        };
        
        self.clients.write().await.insert(id.clone(), client);
        id
    }
    
    /// Disconnect a mock client
    pub async fn disconnect_client(&self, client_id: &str) -> Result<(), String> {
        if let Some(client) = self.clients.write().await.get_mut(client_id) {
            client.connected = false;
        }
        Ok(())
    }
    
    /// Send a message from client to server
    pub async fn receive_message(&self, client_id: &str, message: ClientMessage) -> Result<(), String> {
        if let Some(client) = self.clients.write().await.get_mut(client_id) {
            if client.connected {
                client.messages.push(message);
            } else {
                return Err("Client not connected".to_string());
            }
        } else {
            return Err("Client not found".to_string());
        }
        Ok(())
    }
    
    /// Send a message from server to client
    pub async fn send_message(&self, client_id: Option<&str>, message: ServerMessage) -> Result<(), String> {
        if let Some(client_id) = client_id {
            if let Some(client) = self.clients.read().await.get(client_id) {
                if !client.connected {
                    return Err("Client not connected".to_string());
                }
            } else {
                return Err("Client not found".to_string());
            }
        }
        
        self.messages.write().await.push(message);
        Ok(())
    }
    
    /// Broadcast a message to all connected clients
    pub async fn broadcast_message(&self, message: ServerMessage) -> Result<(), String> {
        let clients = self.clients.read().await;
        let connected_clients: Vec<String> = clients
            .values()
            .filter(|client| client.connected)
            .map(|client| client.id.clone())
            .collect();
        
        for client_id in connected_clients {
            let mut broadcast_message = message.clone();
            broadcast_message.client_id = Some(client_id);
            self.messages.write().await.push(broadcast_message);
        }
        
        Ok(())
    }
    
    /// Get all messages sent by the server
    pub async fn get_messages(&self) -> Vec<ServerMessage> {
        self.messages.read().await.clone()
    }
    
    /// Get messages for a specific client
    pub async fn get_client_messages(&self, client_id: &str) -> Vec<ClientMessage> {
        if let Some(client) = self.clients.read().await.get(client_id) {
            client.messages.clone()
        } else {
            Vec::new()
        }
    }
    
    /// Get connected clients
    pub async fn get_connected_clients(&self) -> Vec<String> {
        self.clients
            .read()
            .await
            .values()
            .filter(|client| client.connected)
            .map(|client| client.id.clone())
            .collect()
    }
    
    /// Check if a client is connected
    pub async fn is_client_connected(&self, client_id: &str) -> bool {
        if let Some(client) = self.clients.read().await.get(client_id) {
            client.connected
        } else {
            false
        }
    }
    
    /// Get server statistics
    pub async fn get_stats(&self) -> ServerStats {
        let clients = self.clients.read().await;
        let connected_count = clients.values().filter(|client| client.connected).count();
        let total_messages = clients.values().map(|client| client.messages.len()).sum();
        
        ServerStats {
            total_clients: clients.len(),
            connected_clients: connected_count,
            total_messages,
            server_messages: self.messages.read().await.len(),
        }
    }
}

/// Server statistics
#[derive(Debug, Clone)]
pub struct ServerStats {
    pub total_clients: usize,
    pub connected_clients: usize,
    pub total_messages: usize,
    pub server_messages: usize,
}

/// Test server configuration
#[derive(Debug, Clone)]
pub struct TestServerConfig {
    pub port: u16,
    pub max_clients: usize,
    pub message_timeout: std::time::Duration,
    pub heartbeat_interval: std::time::Duration,
}

impl Default for TestServerConfig {
    fn default() -> Self {
        Self {
            port: 3001,
            max_clients: 100,
            message_timeout: std::time::Duration::from_secs(30),
            heartbeat_interval: std::time::Duration::from_secs(10),
        }
    }
}

/// Helper function to create a test server
pub async fn create_test_server(port: u16) -> MockWebSocketServer {
    let server = MockWebSocketServer::new(port);
    server.start().await.unwrap();
    server
}

/// Helper function to create a test server with default configuration
pub async fn create_default_test_server() -> MockWebSocketServer {
    create_test_server(3001).await
}

/// Helper function to wait for server to be ready
pub async fn wait_for_server_ready(server: &MockWebSocketServer, timeout: std::time::Duration) -> Result<(), String> {
    let start = std::time::Instant::now();
    
    while start.elapsed() < timeout {
        if server.get_connected_clients().await.len() >= 0 {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    
    Err("Server not ready within timeout".to_string())
}

/// Helper function to create test messages
pub fn create_test_client_message(client_id: &str, message_type: &str, data: &[u8]) -> ClientMessage {
    ClientMessage {
        client_id: client_id.to_string(),
        message_type: message_type.to_string(),
        data: data.to_vec(),
        timestamp: std::time::SystemTime::now(),
    }
}

/// Helper function to create test server messages
pub fn create_test_server_message(client_id: Option<&str>, message_type: &str, data: &[u8]) -> ServerMessage {
    ServerMessage {
        client_id: client_id.map(|s| s.to_string()),
        message_type: message_type.to_string(),
        data: data.to_vec(),
        timestamp: std::time::SystemTime::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_server_creation() {
        let server = MockWebSocketServer::new(3001);
        assert_eq!(server.port, 3001);
        assert_eq!(server.url(), "ws://localhost:3001");
    }

    #[tokio::test]
    async fn test_client_connection() {
        let server = MockWebSocketServer::new(3001);
        let client_id = server.connect_client(None).await;
        
        assert!(server.is_client_connected(&client_id).await);
        assert_eq!(server.get_connected_clients().await.len(), 1);
    }

    #[tokio::test]
    async fn test_client_disconnection() {
        let server = MockWebSocketServer::new(3001);
        let client_id = server.connect_client(None).await;
        
        assert!(server.is_client_connected(&client_id).await);
        
        server.disconnect_client(&client_id).await.unwrap();
        assert!(!server.is_client_connected(&client_id).await);
    }

    #[tokio::test]
    async fn test_message_sending() {
        let server = MockWebSocketServer::new(3001);
        let client_id = server.connect_client(None).await;
        
        let message = create_test_client_message(&client_id, "test", b"test data");
        server.receive_message(&client_id, message.clone()).await.unwrap();
        
        let messages = server.get_client_messages(&client_id).await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message_type, "test");
    }

    #[tokio::test]
    async fn test_server_stats() {
        let server = MockWebSocketServer::new(3001);
        let client_id = server.connect_client(None).await;
        
        let message = create_test_client_message(&client_id, "test", b"test data");
        server.receive_message(&client_id, message).await.unwrap();
        
        let stats = server.get_stats().await;
        assert_eq!(stats.total_clients, 1);
        assert_eq!(stats.connected_clients, 1);
        assert_eq!(stats.total_messages, 1);
    }
}
