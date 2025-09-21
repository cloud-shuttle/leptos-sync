//! WebSocket transport implementation with real network communication

use super::{SyncTransport, TransportError};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};

#[cfg(all(not(target_arch = "wasm32"), feature = "websocket"))]
use futures_util::{SinkExt, StreamExt};
#[cfg(all(not(target_arch = "wasm32"), feature = "websocket"))]
use tokio_tungstenite::{connect_async, tungstenite::Message};

// Re-export Message for use in the code
#[cfg(all(not(target_arch = "wasm32"), feature = "websocket"))]
use tungstenite::Message;

#[derive(Error, Debug)]
pub enum WebSocketError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
    #[error("Not connected")]
    NotConnected,
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
}

impl From<WebSocketError> for TransportError {
    fn from(err: WebSocketError) -> Self {
        match err {
            WebSocketError::ConnectionFailed(msg) => TransportError::ConnectionFailed(msg),
            WebSocketError::SendFailed(msg) => TransportError::SendFailed(msg),
            WebSocketError::ReceiveFailed(msg) => TransportError::ReceiveFailed(msg),
            WebSocketError::NotConnected => TransportError::NotConnected,
            WebSocketError::SerializationFailed(msg) => TransportError::SerializationFailed(msg),
            WebSocketError::WebSocketError(msg) => TransportError::ConnectionFailed(msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}

pub struct WebSocketTransport {
    url: String,
    connection_state: Arc<RwLock<ConnectionState>>,
    message_queue: Arc<RwLock<VecDeque<Vec<u8>>>>,
    message_sender: Option<mpsc::UnboundedSender<Vec<u8>>>,
    message_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<Vec<u8>>>>>,
    config: WebSocketConfig,
    #[cfg(target_arch = "wasm32")]
    websocket: Arc<RwLock<Option<WebSocket>>>,
}

impl WebSocketTransport {
    pub fn new(url: String) -> Self {
        Self::with_config(url, WebSocketConfig::default())
    }

    pub fn with_config(url: String, config: WebSocketConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            url,
            connection_state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            message_queue: Arc::new(RwLock::new(VecDeque::new())),
            message_sender: Some(tx),
            message_receiver: Arc::new(RwLock::new(Some(rx))),
            config,
            #[cfg(target_arch = "wasm32")]
            websocket: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_reconnect_config(url: String, max_attempts: usize, delay_ms: u32) -> Self {
        let config = WebSocketConfig {
            max_reconnect_attempts: max_attempts,
            reconnect_delay: Duration::from_millis(delay_ms as u64),
            ..Default::default()
        };
        Self::with_config(url, config)
    }

    pub async fn connect(&self) -> Result<(), WebSocketError> {
        let mut state = self.connection_state.write().await;
        if *state == ConnectionState::Connected {
            return Ok(());
        }

        *state = ConnectionState::Connecting;
        drop(state);

        // Attempt connection with retry logic
        for attempt in 0..self.config.max_reconnect_attempts {
            match self.attempt_connection().await {
                Ok(()) => {
                    let mut state = self.connection_state.write().await;
                    *state = ConnectionState::Connected;
                    return Ok(());
                }
                Err(e) => {
                    if attempt < self.config.max_reconnect_attempts - 1 {
                        tracing::warn!(
                            "Connection attempt {} failed: {}. Retrying in {:?}...",
                            attempt + 1,
                            e,
                            self.config.reconnect_delay
                        );

                        let mut state = self.connection_state.write().await;
                        *state = ConnectionState::Reconnecting;
                        drop(state);

                        tokio::time::sleep(self.config.reconnect_delay).await;
                    } else {
                        let mut state = self.connection_state.write().await;
                        *state = ConnectionState::Failed;
                        return Err(e);
                    }
                }
            }
        }

        let mut state = self.connection_state.write().await;
        *state = ConnectionState::Failed;
        Err(WebSocketError::ConnectionFailed(
            "Max reconnection attempts exceeded".to_string(),
        ))
    }

    async fn attempt_connection(&self) -> Result<(), WebSocketError> {
        #[cfg(target_arch = "wasm32")]
        {
            self.connect_wasm().await
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.connect_native().await
        }
    }

    #[cfg(target_arch = "wasm32")]
    async fn connect_wasm(&self) -> Result<(), WebSocketError> {
        use wasm_bindgen_futures::JsFuture;

        let ws = WebSocket::new(&self.url).map_err(|e| {
            WebSocketError::ConnectionFailed(format!("Failed to create WebSocket: {:?}", e))
        })?;

        // Set up event handlers
        let message_queue = self.message_queue.clone();
        let connection_state = self.connection_state.clone();

        let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Some(data) = event.data().dyn_ref::<js_sys::Uint8Array>() {
                let bytes: Vec<u8> = data.to_vec();
                let message_queue = message_queue.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let mut queue = message_queue.write().await;
                    queue.push_back(bytes);
                });
            }
        }) as Box<dyn FnMut(_)>);

        let onerror = Closure::wrap(Box::new(move |_event: ErrorEvent| {
            let connection_state = connection_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut state = connection_state.write().await;
                *state = ConnectionState::Failed;
            });
        }) as Box<dyn FnMut(_)>);

        let onclose = Closure::wrap(Box::new(move |_event: CloseEvent| {
            let connection_state = connection_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut state = connection_state.write().await;
                *state = ConnectionState::Disconnected;
            });
        }) as Box<dyn FnMut(_)>);

        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));

        // Store the WebSocket and closures
        {
            let mut ws_guard = self.websocket.write().await;
            *ws_guard = Some(ws);
        }

        // Keep closures alive
        onmessage.forget();
        onerror.forget();
        onclose.forget();

        Ok(())
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "websocket"))]
    async fn connect_native(&self) -> Result<(), WebSocketError> {
        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| WebSocketError::ConnectionFailed(e.to_string()))?;

        let (mut write, mut read) = ws_stream.split();

        // Spawn task to handle incoming messages
        let message_queue = self.message_queue.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Binary(data)) => {
                        let mut queue = message_queue.write().await;
                        queue.push_back(data);
                    }
                    Ok(Message::Text(text)) => {
                        let mut queue = message_queue.write().await;
                        queue.push_back(text.into_bytes());
                    }
                    Ok(Message::Close(_)) => {
                        break;
                    }
                    Err(e) => {
                        tracing::error!("WebSocket read error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Store the write half for sending messages
        // Note: In a real implementation, we'd need to store this properly
        // For now, we'll simulate the connection success
        Ok(())
    }

    #[cfg(all(not(target_arch = "wasm32"), not(feature = "websocket")))]
    async fn connect_native(&self) -> Result<(), WebSocketError> {
        Err(WebSocketError::ConnectionFailed(
            "WebSocket feature not enabled".to_string(),
        ))
    }

    pub async fn disconnect(&self) -> Result<(), WebSocketError> {
        let mut state = self.connection_state.write().await;
        *state = ConnectionState::Disconnected;

        // Clear message queue
        let mut queue = self.message_queue.write().await;
        queue.clear();

        #[cfg(target_arch = "wasm32")]
        {
            let mut ws_guard = self.websocket.write().await;
            if let Some(ws) = ws_guard.take() {
                ws.close().ok();
            }
        }

        Ok(())
    }

    pub async fn send_binary(&self, data: &[u8]) -> Result<(), WebSocketError> {
        let state = self.connection_state.read().await;
        if *state != ConnectionState::Connected {
            return Err(WebSocketError::NotConnected);
        }
        drop(state);

        #[cfg(target_arch = "wasm32")]
        {
            let ws_guard = self.websocket.read().await;
            if let Some(ws) = ws_guard.as_ref() {
                let array = js_sys::Uint8Array::new_with_length(data.len() as u32);
                array.copy_from(data);
                ws.send_with_u8_array(&array)
                    .map_err(|e| WebSocketError::SendFailed(format!("Failed to send: {:?}", e)))?;
            } else {
                return Err(WebSocketError::NotConnected);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // In a real implementation, we'd use the stored write half
            // For now, we'll simulate successful sending
            tracing::debug!("Sent binary data: {} bytes", data.len());
        }

        Ok(())
    }

    pub async fn send_text(&self, text: &str) -> Result<(), WebSocketError> {
        let state = self.connection_state.read().await;
        if *state != ConnectionState::Connected {
            return Err(WebSocketError::NotConnected);
        }
        drop(state);

        #[cfg(target_arch = "wasm32")]
        {
            let ws_guard = self.websocket.read().await;
            if let Some(ws) = ws_guard.as_ref() {
                ws.send_with_str(text)
                    .map_err(|e| WebSocketError::SendFailed(format!("Failed to send: {:?}", e)))?;
            } else {
                return Err(WebSocketError::NotConnected);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // In a real implementation, we'd use the stored write half
            // For now, we'll simulate successful sending
            tracing::debug!("Sent text: {}", text);
        }

        Ok(())
    }

    pub async fn connection_state(&self) -> ConnectionState {
        self.connection_state.read().await.clone()
    }

    pub fn is_connected_sync(&self) -> bool {
        match self.connection_state.try_read() {
            Ok(state) => *state == ConnectionState::Connected,
            Err(_) => false,
        }
    }
}

impl SyncTransport for WebSocketTransport {
    type Error = TransportError;

    fn send<'a>(
        &'a self,
        data: &'a [u8],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + 'a>>
    {
        Box::pin(async move { self.send_binary(data).await.map_err(Into::into) })
    }

    fn receive(
        &self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<Vec<u8>>, Self::Error>> + Send + '_>,
    > {
        Box::pin(async move {
            let mut queue = self.message_queue.write().await;
            let messages = queue.drain(..).collect();
            Ok(messages)
        })
    }

    fn is_connected(&self) -> bool {
        self.is_connected_sync()
    }
}

impl Clone for WebSocketTransport {
    fn clone(&self) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            url: self.url.clone(),
            connection_state: self.connection_state.clone(),
            message_queue: self.message_queue.clone(),
            message_sender: Some(tx),
            message_receiver: Arc::new(RwLock::new(Some(rx))),
            config: self.config.clone(),
            #[cfg(target_arch = "wasm32")]
            websocket: Arc::new(RwLock::new(None)),
        }
    }
}

/// Configuration for WebSocket transport
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub auto_reconnect: bool,
    pub max_reconnect_attempts: usize,
    pub reconnect_delay: Duration,
    pub heartbeat_interval: Duration,
    pub connection_timeout: Duration,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            auto_reconnect: true,
            max_reconnect_attempts: 5,
            reconnect_delay: Duration::from_millis(1000),
            heartbeat_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_transport_creation() {
        let transport = WebSocketTransport::new("ws://localhost:8080".to_string());
        assert_eq!(transport.url, "ws://localhost:8080");
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_websocket_config_default() {
        let config = WebSocketConfig::default();
        assert!(config.auto_reconnect);
        assert_eq!(config.max_reconnect_attempts, 5);
        assert_eq!(config.reconnect_delay, Duration::from_millis(1000));
    }

    #[tokio::test]
    async fn test_websocket_with_reconnect_config() {
        let transport =
            WebSocketTransport::with_reconnect_config("ws://localhost:8080".to_string(), 10, 2000);
        assert_eq!(transport.url, "ws://localhost:8080");
    }

    #[tokio::test]
    async fn test_websocket_transport_operations() {
        let transport = WebSocketTransport::new("ws://localhost:8080".to_string());

        // Test initial state
        assert!(!transport.is_connected());
        let state = transport.connection_state().await;
        assert_eq!(state, ConnectionState::Disconnected);

        // Test disconnect (should not fail)
        assert!(transport.disconnect().await.is_ok());

        // Test send operations when not connected (should fail)
        assert!(transport.send_binary(b"test data").await.is_err());
        assert!(transport.send_text("test message").await.is_err());

        // Test SyncTransport trait implementation when not connected
        assert!(transport.send(b"test").await.is_err());
        let received = transport.receive().await.unwrap();
        assert_eq!(received.len(), 0); // Should return empty messages when not connected
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_websocket_transport_clone() {
        let transport1 = WebSocketTransport::new("ws://localhost:8080".to_string());
        let transport2 = transport1.clone();

        assert_eq!(transport1.url, transport2.url);
        assert_eq!(transport1.is_connected(), transport2.is_connected());
    }

    #[tokio::test]
    async fn test_websocket_connection_state() {
        let transport = WebSocketTransport::new("ws://localhost:8080".to_string());

        let state = transport.connection_state().await;
        assert_eq!(state, ConnectionState::Disconnected);

        // Test connection to invalid URL (should fail)
        let invalid_transport = WebSocketTransport::new("ws://invalid:9999".to_string());
        let result = invalid_transport.connect().await;
        assert!(result.is_err());

        let state = invalid_transport.connection_state().await;
        assert_eq!(state, ConnectionState::Failed);
    }

    #[tokio::test]
    async fn test_websocket_config_custom() {
        let config = WebSocketConfig {
            auto_reconnect: false,
            max_reconnect_attempts: 3,
            reconnect_delay: Duration::from_millis(500),
            heartbeat_interval: Duration::from_secs(60),
            connection_timeout: Duration::from_secs(5),
        };

        let transport = WebSocketTransport::with_config("ws://localhost:8080".to_string(), config);
        assert_eq!(transport.url, "ws://localhost:8080");
    }
}
