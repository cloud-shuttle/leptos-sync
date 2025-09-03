//! Real-time synchronization engine for live collaboration

use crate::crdt::{Mergeable, ReplicaId};
use crate::storage::{Storage, LocalStorage};
use crate::transport::SyncTransport;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RealtimeSyncError {
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Real-time synchronization event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealtimeEvent {
    /// Document changed
    DocumentChanged {
        key: String,
        replica_id: ReplicaId,
        timestamp: DateTime<Utc>,
        change_type: ChangeType,
    },
    /// User joined
    UserJoined {
        replica_id: ReplicaId,
        timestamp: DateTime<Utc>,
        user_info: Option<UserInfo>,
    },
    /// User left
    UserLeft {
        replica_id: ReplicaId,
        timestamp: DateTime<Utc>,
    },
    /// Sync started
    SyncStarted {
        replica_id: ReplicaId,
        timestamp: DateTime<Utc>,
    },
    /// Sync completed
    SyncCompleted {
        replica_id: ReplicaId,
        timestamp: DateTime<Utc>,
        changes_synced: usize,
    },
    /// Conflict detected
    ConflictDetected {
        key: String,
        replica_id: ReplicaId,
        timestamp: DateTime<Utc>,
        conflict_type: String,
    },
}

/// Type of change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Updated,
    Deleted,
    Merged,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub color: Option<String>,
}

/// Real-time synchronization manager
pub struct RealtimeSyncManager<Tr>
where
    Tr: SyncTransport + Clone + Send + Sync + 'static,
{
    replica_id: ReplicaId,
    transport: Tr,
    storage: Arc<Storage>,
    event_sender: broadcast::Sender<RealtimeEvent>,
    subscriptions: Arc<RwLock<HashMap<String, Subscription>>>,
    active_users: Arc<RwLock<HashMap<ReplicaId, UserInfo>>>,
    sync_state: Arc<RwLock<SyncState>>,
    heartbeat_interval: std::time::Duration,
    presence_timeout: std::time::Duration,
}

/// Subscription to real-time events
pub struct Subscription {
    pub id: String,
    pub event_types: Vec<String>,
    pub callback: Box<dyn Fn(RealtimeEvent) + Send + Sync>,
}

/// Synchronization state
#[derive(Debug, Clone)]
pub struct SyncState {
    pub is_syncing: bool,
    pub last_sync: Option<DateTime<Utc>>,
    pub connected_users: usize,
    pub pending_changes: usize,
    pub sync_errors: Vec<String>,
}

impl<Tr> RealtimeSyncManager<Tr>
where
    Tr: SyncTransport + Clone + Send + Sync + 'static,
{
    pub fn new(
        replica_id: ReplicaId,
        transport: Tr,
        storage: Arc<Storage>,
    ) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            replica_id,
            transport,
            storage,
            event_sender,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            active_users: Arc::new(RwLock::new(HashMap::new())),
            sync_state: Arc::new(RwLock::new(SyncState {
                is_syncing: false,
                last_sync: None,
                connected_users: 0,
                pending_changes: 0,
                sync_errors: Vec::new(),
            })),
            heartbeat_interval: std::time::Duration::from_secs(30),
            presence_timeout: std::time::Duration::from_secs(120),
        }
    }

    /// Start real-time synchronization
    pub async fn start(&mut self) -> Result<(), RealtimeSyncError> {
        let mut state = self.sync_state.write().await;
        state.is_syncing = true;
        drop(state);

        // Announce presence
        self.announce_presence().await?;

        // Start heartbeat
        self.start_heartbeat().await;

        // Start presence monitoring
        self.start_presence_monitoring().await;

        // Emit sync started event
        self.emit_event(RealtimeEvent::SyncStarted {
            replica_id: self.replica_id,
            timestamp: Utc::now(),
        }).await;

        Ok(())
    }

    /// Stop real-time synchronization
    pub async fn stop(&mut self) -> Result<(), RealtimeSyncError> {
        let mut state = self.sync_state.write().await;
        state.is_syncing = false;
        drop(state);

        // Announce departure
        self.announce_departure().await?;

        // Emit sync completed event
        self.emit_event(RealtimeEvent::SyncCompleted {
            replica_id: self.replica_id,
            timestamp: Utc::now(),
            changes_synced: 0,
        }).await;

        Ok(())
    }

    /// Subscribe to real-time events
    pub async fn subscribe(
        &self,
        event_types: Vec<String>,
        callback: Box<dyn Fn(RealtimeEvent) + Send + Sync>,
    ) -> Result<String, RealtimeSyncError> {
        let subscription_id = uuid::Uuid::new_v4().to_string();
        
        let subscription = Subscription {
            id: subscription_id.clone(),
            event_types,
            callback,
        };

        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription_id.clone(), subscription);

        Ok(subscription_id)
    }

    /// Unsubscribe from real-time events
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<(), RealtimeSyncError> {
        let mut subscriptions = self.subscriptions.write().await;
        
        if subscriptions.remove(subscription_id).is_some() {
            Ok(())
        } else {
            Err(RealtimeSyncError::SubscriptionNotFound(subscription_id.to_string()))
        }
    }

    /// Broadcast a change to all connected peers
    pub async fn broadcast_change<T: Mergeable + Serialize + Clone>(
        &self,
        key: &str,
        value: &T,
        change_type: ChangeType,
    ) -> Result<(), RealtimeSyncError> {
        // Store locally first
        self.storage.set(key, value).await
            .map_err(|e| RealtimeSyncError::Storage(e.to_string()))?;

        // Serialize and send via transport
        let change_message = ChangeMessage {
            key: key.to_string(),
            data: serde_json::to_vec(value)
                .map_err(|e| RealtimeSyncError::Serialization(e.to_string()))?,
            replica_id: self.replica_id,
            timestamp: Utc::now(),
            change_type: change_type.clone(),
        };

        let message_bytes = serde_json::to_vec(&change_message)
            .map_err(|e| RealtimeSyncError::Serialization(e.to_string()))?;

        self.transport.send(&message_bytes).await
            .map_err(|e| RealtimeSyncError::Transport(e.to_string()))?;

        // Emit local event
        self.emit_event(RealtimeEvent::DocumentChanged {
            key: key.to_string(),
            replica_id: self.replica_id,
            timestamp: Utc::now(),
            change_type,
        }).await;

        Ok(())
    }

    /// Process incoming changes from peers
    pub async fn process_incoming_changes(&mut self) -> Result<usize, RealtimeSyncError> {
        let messages = self.transport.receive().await
            .map_err(|e| RealtimeSyncError::Transport(e.to_string()))?;

        let mut changes_processed = 0;

        for message_bytes in messages {
            if let Ok(change_message) = serde_json::from_slice::<ChangeMessage>(&message_bytes) {
                // Process the change
                self.process_change(change_message).await?;
                changes_processed += 1;
            }
        }

        // Update sync state
        let mut state = self.sync_state.write().await;
        state.last_sync = Some(Utc::now());
        state.pending_changes = state.pending_changes.saturating_sub(changes_processed);

        Ok(changes_processed)
    }

    /// Get current synchronization state
    pub async fn get_sync_state(&self) -> SyncState {
        self.sync_state.read().await.clone()
    }

    /// Get active users
    pub async fn get_active_users(&self) -> HashMap<ReplicaId, UserInfo> {
        self.active_users.read().await.clone()
    }

    /// Announce presence to peers
    async fn announce_presence(&self) -> Result<(), RealtimeSyncError> {
        let presence_message = PresenceMessage {
            replica_id: self.replica_id,
            timestamp: Utc::now(),
            user_info: None, // Could be populated with actual user info
        };

        let message_bytes = serde_json::to_vec(&presence_message)
            .map_err(|e| RealtimeSyncError::Serialization(e.to_string()))?;

        self.transport.send(&message_bytes).await
            .map_err(|e| RealtimeSyncError::Transport(e.to_string()))?;

        Ok(())
    }

    /// Announce departure to peers
    async fn announce_departure(&self) -> Result<(), RealtimeSyncError> {
        let departure_message = DepartureMessage {
            replica_id: self.replica_id,
            timestamp: Utc::now(),
        };

        let message_bytes = serde_json::to_vec(&departure_message)
            .map_err(|e| RealtimeSyncError::Serialization(e.to_string()))?;

        self.transport.send(&message_bytes).await
            .map_err(|e| RealtimeSyncError::Transport(e.to_string()))?;

        Ok(())
    }

    /// Start heartbeat mechanism
    async fn start_heartbeat(&self) {
        let transport = self.transport.clone();
        let replica_id = self.replica_id;
        let interval = self.heartbeat_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                // Send heartbeat
                let heartbeat_message = HeartbeatMessage {
                    replica_id,
                    timestamp: Utc::now(),
                };

                if let Ok(message_bytes) = serde_json::to_vec(&heartbeat_message) {
                    let _ = transport.send(&message_bytes).await;
                }
            }
        });
    }

    /// Start presence monitoring
    async fn start_presence_monitoring(&self) {
        let active_users = self.active_users.clone();
        let timeout = self.presence_timeout;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(std::time::Duration::from_secs(60));
            
            loop {
                interval_timer.tick().await;
                
                let now = Utc::now();
                let mut users = active_users.write().await;
                
                // Remove users who haven't sent heartbeat recently
                users.retain(|_, user_info| {
                    // This is a simplified check - in reality you'd track last heartbeat
                    true
                });
            }
        });
    }

    /// Process an incoming change
    async fn process_change(&mut self, change_message: ChangeMessage) -> Result<(), RealtimeSyncError> {
        // Emit event for the change
        self.emit_event(RealtimeEvent::DocumentChanged {
            key: change_message.key.clone(),
            replica_id: change_message.replica_id,
            timestamp: change_message.timestamp,
            change_type: change_message.change_type,
        }).await;

        Ok(())
    }

    /// Emit an event to all subscribers
    async fn emit_event(&self, event: RealtimeEvent) {
        let subscriptions = self.subscriptions.read().await;
        
        for subscription in subscriptions.values() {
            // Check if subscription is interested in this event type
            let event_type = match &event {
                RealtimeEvent::DocumentChanged { .. } => "document_changed",
                RealtimeEvent::UserJoined { .. } => "user_joined",
                RealtimeEvent::UserLeft { .. } => "user_left",
                RealtimeEvent::SyncStarted { .. } => "sync_started",
                RealtimeEvent::SyncCompleted { .. } => "sync_completed",
                RealtimeEvent::ConflictDetected { .. } => "conflict_detected",
            };

            if subscription.event_types.contains(&event_type.to_string()) 
                || subscription.event_types.contains(&"*".to_string()) {
                (subscription.callback)(event.clone());
            }
        }
    }
}

/// Change message for broadcasting updates
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChangeMessage {
    key: String,
    data: Vec<u8>,
    replica_id: ReplicaId,
    timestamp: DateTime<Utc>,
    change_type: ChangeType,
}

/// Presence message for announcing user presence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PresenceMessage {
    replica_id: ReplicaId,
    timestamp: DateTime<Utc>,
    user_info: Option<UserInfo>,
}

/// Departure message for announcing user departure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DepartureMessage {
    replica_id: ReplicaId,
    timestamp: DateTime<Utc>,
}

/// Heartbeat message for keeping connections alive
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HeartbeatMessage {
    replica_id: ReplicaId,
    timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::memory::MemoryStorage;
    use crate::transport::memory::InMemoryTransport;

    #[tokio::test]
    async fn test_realtime_sync_manager_creation() {
        let storage = Arc::new(Storage::memory());
        let transport = InMemoryTransport::new();
        let replica_id = ReplicaId::default();
        
        let manager = RealtimeSyncManager::new(replica_id, transport, storage);
        assert_eq!(manager.replica_id, replica_id);
    }

    #[tokio::test]
    async fn test_subscription_management() {
        let storage = Arc::new(Storage::memory());
        let transport = InMemoryTransport::new();
        let replica_id = ReplicaId::default();
        
        let manager = RealtimeSyncManager::new(replica_id, transport, storage);
        
        // Subscribe to events
        let callback = Box::new(|_event: RealtimeEvent| {});
        let subscription_id = manager.subscribe(
            vec!["document_changed".to_string()],
            callback
        ).await.unwrap();
        
        // Unsubscribe
        let result = manager.unsubscribe(&subscription_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_state_management() {
        let storage = Arc::new(Storage::memory());
        let transport = InMemoryTransport::new();
        let replica_id = ReplicaId::default();
        
        let manager = RealtimeSyncManager::new(replica_id, transport, storage);
        
        let state = manager.get_sync_state().await;
        assert!(!state.is_syncing);
        assert_eq!(state.connected_users, 0);
        assert_eq!(state.pending_changes, 0);
    }
}
