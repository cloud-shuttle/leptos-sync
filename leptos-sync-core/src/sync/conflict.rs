//! Advanced conflict resolution strategies for CRDT synchronization

use crate::crdt::{Mergeable, ReplicaId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConflictResolutionError {
    #[error("Unresolvable conflict: {0}")]
    Unresolvable(String),
    #[error("Strategy not applicable: {0}")]
    StrategyNotApplicable(String),
    #[error("Invalid conflict data: {0}")]
    InvalidData(String),
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictStrategy {
    /// Last-Write-Wins (default)
    LastWriteWins,
    /// First-Write-Wins
    FirstWriteWins,
    /// Custom merge function
    CustomMerge,
    /// Manual resolution required
    ManualResolution,
    /// Conflict avoidance (prevent conflicts)
    ConflictAvoidance,
}

/// Conflict metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictMetadata {
    pub replica_id: ReplicaId,
    pub timestamp: DateTime<Utc>,
    pub version: u64,
    pub conflict_type: String,
    pub resolution_strategy: ConflictStrategy,
}

/// Conflict resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution<T> {
    pub resolved_value: T,
    pub strategy_used: ConflictStrategy,
    pub metadata: ConflictMetadata,
    pub conflicts_resolved: usize,
}

/// Advanced conflict resolver
pub struct AdvancedConflictResolver {
    strategies: HashMap<String, Box<dyn ConflictResolutionStrategy + Send + Sync>>,
    default_strategy: ConflictStrategy,
    conflict_history: Vec<ConflictMetadata>,
}

impl AdvancedConflictResolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            strategies: HashMap::new(),
            default_strategy: ConflictStrategy::LastWriteWins,
            conflict_history: Vec::new(),
        };
        
        // Register default strategies
        resolver.register_strategy("lww", Box::new(LastWriteWinsStrategy));
        resolver.register_strategy("fww", Box::new(FirstWriteWinsStrategy));
        resolver.register_strategy("custom", Box::new(CustomMergeStrategy));
        
        resolver
    }

    pub fn with_default_strategy(mut self, strategy: ConflictStrategy) -> Self {
        self.default_strategy = strategy;
        self
    }

    pub fn register_strategy(&mut self, name: &str, strategy: Box<dyn ConflictResolutionStrategy + Send + Sync>) {
        self.strategies.insert(name.to_string(), strategy);
    }

    pub async fn resolve<T: Mergeable + Clone + Send + Sync>(
        &mut self,
        local: &T,
        remote: &T,
        metadata: Option<ConflictMetadata>,
    ) -> Result<ConflictResolution<T>, ConflictResolutionError> {
        let metadata = metadata.unwrap_or_else(|| ConflictMetadata {
            replica_id: ReplicaId::default(),
            timestamp: Utc::now(),
            version: 1,
            conflict_type: "default".to_string(),
            resolution_strategy: self.default_strategy.clone(),
        });

        // Check if there's actually a conflict
        if !self.has_conflict(local, remote, &metadata).await? {
            return Ok(ConflictResolution {
                resolved_value: local.clone(),
                strategy_used: ConflictStrategy::LastWriteWins,
                metadata,
                conflicts_resolved: 0,
            });
        }

        // Record conflict
        self.conflict_history.push(metadata.clone());

        // Apply resolution strategy
        let strategy = &metadata.resolution_strategy;
        let resolved_value = match strategy {
            ConflictStrategy::LastWriteWins => {
                self.resolve_last_write_wins(local, remote, &metadata).await?
            }
            ConflictStrategy::FirstWriteWins => {
                self.resolve_first_write_wins(local, remote, &metadata).await?
            }
            ConflictStrategy::CustomMerge => {
                self.resolve_custom_merge(local, remote, &metadata).await?
            }
            ConflictStrategy::ManualResolution => {
                return Err(ConflictResolutionError::Unresolvable(
                    "Manual resolution required".to_string()
                ));
            }
            ConflictStrategy::ConflictAvoidance => {
                self.resolve_conflict_avoidance(local, remote, &metadata).await?
            }
        };

        Ok(ConflictResolution {
            resolved_value,
            strategy_used: strategy.clone(),
            metadata,
            conflicts_resolved: 1,
        })
    }

    async fn has_conflict<T: Mergeable>(
        &self,
        local: &T,
        remote: &T,
        metadata: &ConflictMetadata,
    ) -> Result<bool, ConflictResolutionError> {
        // Use the CRDT's built-in conflict detection
        Ok(local.has_conflict(remote))
    }

    async fn resolve_last_write_wins<T: Mergeable + Clone>(
        &self,
        local: &T,
        remote: &T,
        metadata: &ConflictMetadata,
    ) -> Result<T, ConflictResolutionError> {
        // Simple timestamp-based resolution
        let mut result = local.clone();
        result.merge(remote).map_err(|e| {
            ConflictResolutionError::InvalidData(format!("Merge failed: {}", e))
        })?;
        Ok(result)
    }

    async fn resolve_first_write_wins<T: Mergeable + Clone>(
        &self,
        local: &T,
        remote: &T,
        metadata: &ConflictMetadata,
    ) -> Result<T, ConflictResolutionError> {
        // First-write-wins logic
        let mut result = local.clone();
        // In a real implementation, you'd check creation timestamps
        // For now, just merge and return
        result.merge(remote).map_err(|e| {
            ConflictResolutionError::InvalidData(format!("Merge failed: {}", e))
        })?;
        Ok(result)
    }

    async fn resolve_custom_merge<T: Mergeable + Clone>(
        &self,
        local: &T,
        remote: &T,
        metadata: &ConflictMetadata,
    ) -> Result<T, ConflictResolutionError> {
        // Custom merge logic based on conflict type
        match metadata.conflict_type.as_str() {
            "text" => self.merge_text_conflicts(local, remote).await,
            "numeric" => self.merge_numeric_conflicts(local, remote).await,
            "list" => self.merge_list_conflicts(local, remote).await,
            _ => self.resolve_last_write_wins(local, remote, metadata).await,
        }
    }

    async fn resolve_conflict_avoidance<T: Mergeable + Clone>(
        &self,
        local: &T,
        remote: &T,
        metadata: &ConflictMetadata,
    ) -> Result<T, ConflictResolutionError> {
        // Try to avoid conflicts by using more sophisticated merging
        let mut result = local.clone();
        
        // Attempt to merge without conflicts
        if let Ok(()) = result.merge(remote) {
            return Ok(result);
        }

        // If merge fails, fall back to last-write-wins
        self.resolve_last_write_wins(local, remote, metadata).await
    }

    async fn merge_text_conflicts<T: Mergeable + Clone>(
        &self,
        _local: &T,
        _remote: &T,
    ) -> Result<T, ConflictResolutionError> {
        // Text-specific merging logic would go here
        // For now, return an error
        Err(ConflictResolutionError::StrategyNotApplicable(
            "Text merging not implemented".to_string()
        ))
    }

    async fn merge_numeric_conflicts<T: Mergeable + Clone>(
        &self,
        _local: &T,
        _remote: &T,
    ) -> Result<T, ConflictResolutionError> {
        // Numeric-specific merging logic would go here
        // For now, return an error
        Err(ConflictResolutionError::StrategyNotApplicable(
            "Numeric merging not implemented".to_string()
        ))
    }

    async fn merge_list_conflicts<T: Mergeable + Clone>(
        &self,
        _local: &T,
        _remote: &T,
    ) -> Result<T, ConflictResolutionError> {
        // List-specific merging logic would go here
        // For now, return an error
        Err(ConflictResolutionError::StrategyNotApplicable(
            "List merging not implemented".to_string()
        ))
    }

    pub fn get_conflict_history(&self) -> &[ConflictMetadata] {
        &self.conflict_history
    }

    pub fn clear_conflict_history(&mut self) {
        self.conflict_history.clear();
    }
}

impl Default for AdvancedConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for custom conflict resolution strategies
pub trait ConflictResolutionStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn can_resolve(&self, conflict_type: &str) -> bool;
}

/// Last-Write-Wins strategy implementation
pub struct LastWriteWinsStrategy;

impl ConflictResolutionStrategy for LastWriteWinsStrategy {
    fn name(&self) -> &str {
        "last-write-wins"
    }

    fn can_resolve(&self, _conflict_type: &str) -> bool {
        true // Can resolve any conflict type
    }
}

/// First-Write-Wins strategy implementation
pub struct FirstWriteWinsStrategy;

impl ConflictResolutionStrategy for FirstWriteWinsStrategy {
    fn name(&self) -> &str {
        "first-write-wins"
    }

    fn can_resolve(&self, _conflict_type: &str) -> bool {
        true // Can resolve any conflict type
    }
}

/// Custom merge strategy implementation
pub struct CustomMergeStrategy;

impl ConflictResolutionStrategy for CustomMergeStrategy {
    fn name(&self) -> &str {
        "custom-merge"
    }

    fn can_resolve(&self, conflict_type: &str) -> bool {
        matches!(conflict_type, "text" | "numeric" | "list")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::LwwRegister;

    #[tokio::test]
    async fn test_advanced_conflict_resolver_creation() {
        let resolver = AdvancedConflictResolver::new();
        assert_eq!(resolver.default_strategy, ConflictStrategy::LastWriteWins);
    }

    #[tokio::test]
    async fn test_conflict_resolution_lww() {
        let mut resolver = AdvancedConflictResolver::new();
        let local = LwwRegister::new("local", ReplicaId::default());
        let remote = LwwRegister::new("remote", ReplicaId::default());
        
        let metadata = ConflictMetadata {
            replica_id: ReplicaId::default(),
            timestamp: Utc::now(),
            version: 1,
            conflict_type: "text".to_string(),
            resolution_strategy: ConflictStrategy::LastWriteWins,
        };

        let result = resolver.resolve(&local, &remote, Some(metadata)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_conflict_strategy_registration() {
        let mut resolver = AdvancedConflictResolver::new();
        let custom_strategy = Box::new(CustomMergeStrategy);
        
        resolver.register_strategy("custom", custom_strategy);
        assert!(resolver.strategies.contains_key("custom"));
    }

    #[tokio::test]
    async fn test_conflict_resolution_with_different_strategies() {
        let mut resolver = AdvancedConflictResolver::new();
        let local = LwwRegister::new("local", ReplicaId::default());
        let remote = LwwRegister::new("remote", ReplicaId::default());
        
        // Test Last-Write-Wins strategy
        let metadata_lww = ConflictMetadata {
            replica_id: ReplicaId::default(),
            timestamp: Utc::now(),
            version: 1,
            conflict_type: "text".to_string(),
            resolution_strategy: ConflictStrategy::LastWriteWins,
        };
        let result_lww = resolver.resolve(&local, &remote, Some(metadata_lww)).await;
        assert!(result_lww.is_ok());

        // Test First-Write-Wins strategy
        let metadata_fww = ConflictMetadata {
            replica_id: ReplicaId::default(),
            timestamp: Utc::now(),
            version: 1,
            conflict_type: "text".to_string(),
            resolution_strategy: ConflictStrategy::FirstWriteWins,
        };
        let result_fww = resolver.resolve(&local, &remote, Some(metadata_fww)).await;
        assert!(result_fww.is_ok());

        // Test Custom Merge strategy
        let metadata_custom = ConflictMetadata {
            replica_id: ReplicaId::default(),
            timestamp: Utc::now(),
            version: 1,
            conflict_type: "text".to_string(),
            resolution_strategy: ConflictStrategy::CustomMerge,
        };
        let result_custom = resolver.resolve(&local, &remote, Some(metadata_custom)).await;
        assert!(result_custom.is_ok());
    }

    #[tokio::test]
    async fn test_conflict_history_tracking() {
        let mut resolver = AdvancedConflictResolver::new();
        
        // Create registers with different replica IDs but same timestamp to force a conflict
        let local_replica = ReplicaId::default();
        let remote_replica = ReplicaId::default();
        
        // Create registers with the same timestamp to force a conflict
        let now = Utc::now();
        let local = LwwRegister::new("local", local_replica).with_timestamp(now);
        let remote = LwwRegister::new("remote", remote_replica).with_timestamp(now);
        
        let metadata = ConflictMetadata {
            replica_id: ReplicaId::default(),
            timestamp: Utc::now(),
            version: 1,
            conflict_type: "text".to_string(),
            resolution_strategy: ConflictStrategy::LastWriteWins,
        };

        // Initially no conflicts
        assert_eq!(resolver.get_conflict_history().len(), 0);

        // Resolve a conflict
        let _result = resolver.resolve(&local, &remote, Some(metadata)).await;

        // Should have one conflict in history
        assert_eq!(resolver.get_conflict_history().len(), 1);

        // Clear history
        resolver.clear_conflict_history();
        assert_eq!(resolver.get_conflict_history().len(), 0);
    }

    #[tokio::test]
    async fn test_conflict_strategy_validation() {
        let lww_strategy = LastWriteWinsStrategy;
        let fww_strategy = FirstWriteWinsStrategy;
        let custom_strategy = CustomMergeStrategy;

        // All strategies should be able to resolve any conflict type
        assert!(lww_strategy.can_resolve("text"));
        assert!(lww_strategy.can_resolve("numeric"));
        assert!(lww_strategy.can_resolve("list"));

        assert!(fww_strategy.can_resolve("text"));
        assert!(fww_strategy.can_resolve("numeric"));
        assert!(fww_strategy.can_resolve("list"));

        // Custom strategy should only resolve specific types
        assert!(custom_strategy.can_resolve("text"));
        assert!(custom_strategy.can_resolve("numeric"));
        assert!(custom_strategy.can_resolve("list"));
        assert!(!custom_strategy.can_resolve("unknown"));
    }

    #[tokio::test]
    async fn test_conflict_metadata_serialization() {
        let metadata = ConflictMetadata {
            replica_id: ReplicaId::default(),
            timestamp: Utc::now(),
            version: 1,
            conflict_type: "text".to_string(),
            resolution_strategy: ConflictStrategy::LastWriteWins,
        };

        // Test serialization
        let serialized = serde_json::to_string(&metadata);
        assert!(serialized.is_ok());

        // Test deserialization
        let deserialized: ConflictMetadata = serde_json::from_str(&serialized.unwrap()).unwrap();
        assert_eq!(deserialized.conflict_type, "text");
        assert_eq!(deserialized.resolution_strategy, ConflictStrategy::LastWriteWins);
    }
}
