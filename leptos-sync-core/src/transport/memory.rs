//! In-memory transport for testing

use super::{SyncTransport, TransportError};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory transport for testing
#[derive(Clone)]
pub struct InMemoryTransport {
    connected: bool,
    message_queue: Arc<RwLock<VecDeque<Vec<u8>>>>,
}

impl InMemoryTransport {
    pub fn new() -> Self {
        Self {
            connected: true,
            message_queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub fn with_connection_status(connected: bool) -> Self {
        Self {
            connected,
            message_queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }
}

impl SyncTransport for InMemoryTransport {
    type Error = TransportError;

    async fn send(&self, data: &[u8]) -> Result<(), Self::Error> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }
        
        let mut queue = self.message_queue.write().await;
        queue.push_back(data.to_vec());
        Ok(())
    }

    async fn receive(&self) -> Result<Vec<Vec<u8>>, Self::Error> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }
        
        let mut queue = self.message_queue.write().await;
        let messages: Vec<Vec<u8>> = queue.drain(..).collect();
        Ok(messages)
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

impl Default for InMemoryTransport {
    fn default() -> Self {
        Self::new()
    }
}
