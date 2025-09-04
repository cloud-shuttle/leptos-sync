use super::{CRDT, Mergeable, ReplicaId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;

/// Custom error type for list operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListError {
    message: String,
}

impl ListError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for ListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ListError: {}", self.message)
    }
}

impl Error for ListError {}

/// Unique identifier for a list element
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementId {
    /// Unique identifier for the element
    pub id: Uuid,
    /// Replica that created the element
    pub replica: ReplicaId,
}

impl ElementId {
    /// Create a new element ID
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            id: Uuid::new_v4(),
            replica,
        }
    }

    /// Create an element ID from existing UUID and replica
    pub fn from_parts(id: Uuid, replica: ReplicaId) -> Self {
        Self { id, replica }
    }
}

/// Metadata for a list element
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElementMetadata {
    /// When the element was created
    pub created_at: u64,
    /// When the element was last modified
    pub modified_at: u64,
    /// Whether the element is marked as deleted
    pub deleted: bool,
    /// Replica that last modified the element
    pub last_modified_by: ReplicaId,
}

impl ElementMetadata {
    /// Create new metadata
    pub fn new(replica: ReplicaId, timestamp: u64) -> Self {
        Self {
            created_at: timestamp,
            modified_at: timestamp,
            deleted: false,
            last_modified_by: replica,
        }
    }

    /// Mark as modified
    pub fn mark_modified(&mut self, replica: ReplicaId, timestamp: u64) {
        self.modified_at = timestamp;
        self.last_modified_by = replica;
    }

    /// Mark as deleted
    pub fn mark_deleted(&mut self, replica: ReplicaId, timestamp: u64) {
        self.deleted = true;
        self.mark_modified(replica, timestamp);
    }
}

/// A list element with its metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListElement<T> {
    /// Unique identifier
    pub id: ElementId,
    /// The actual value
    pub value: T,
    /// Metadata
    pub metadata: ElementMetadata,
}

impl<T> ListElement<T> {
    /// Create a new list element
    pub fn new(value: T, replica: ReplicaId, timestamp: u64) -> Self {
        Self {
            id: ElementId::new(replica),
            value,
            metadata: ElementMetadata::new(replica, timestamp),
        }
    }

    /// Mark as modified
    pub fn mark_modified(&mut self, replica: ReplicaId, timestamp: u64) {
        self.metadata.mark_modified(replica, timestamp);
    }

    /// Mark as deleted
    pub fn mark_deleted(&mut self, replica: ReplicaId, timestamp: u64) {
        self.metadata.mark_deleted(replica, timestamp);
    }
}

/// Strategy for handling list conflicts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListStrategy {
    /// Add-Wins: Elements are never removed, only marked as deleted
    AddWins,
    /// Remove-Wins: Deleted elements are completely removed
    RemoveWins,
    /// Last-Write-Wins: Most recent modification wins
    LastWriteWins,
}

/// Configuration for list CRDTs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListConfig {
    /// Conflict resolution strategy
    pub strategy: ListStrategy,
    /// Whether to preserve deleted elements in metadata
    pub preserve_deleted: bool,
    /// Maximum number of elements to keep in memory
    pub max_elements: Option<usize>,
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            strategy: ListStrategy::AddWins,
            preserve_deleted: true,
            max_elements: None,
        }
    }
}

/// Add-Wins List CRDT implementation
/// 
/// This implementation ensures that elements are never completely lost.
/// Deleted elements are marked as deleted but preserved for potential recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddWinsList<T> {
    /// Configuration
    config: ListConfig,
    /// Elements in the list
    elements: HashMap<ElementId, ListElement<T>>,
    /// Replica ID for this instance
    replica: ReplicaId,
}

impl<T: Clone + PartialEq + Eq + Send + Sync> AddWinsList<T> {
    /// Create a new Add-Wins list
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            config: ListConfig::default(),
            elements: HashMap::new(),
            replica,
        }
    }

    /// Create with custom configuration
    pub fn with_config(replica: ReplicaId, config: ListConfig) -> Self {
        Self {
            config,
            elements: HashMap::new(),
            replica,
        }
    }

    /// Add an element to the list
    pub fn add(&mut self, value: T, timestamp: u64) -> ElementId {
        let element = ListElement::new(value, self.replica, timestamp);
        let id = element.id.clone();
        self.elements.insert(id.clone(), element);
        id
    }

    /// Update an existing element
    pub fn update(&mut self, id: &ElementId, value: T, timestamp: u64) -> Result<(), ListError> {
        if let Some(element) = self.elements.get_mut(id) {
            element.value = value;
            element.mark_modified(self.replica, timestamp);
            Ok(())
        } else {
            Err(ListError::new("Element not found".to_string()))
        }
    }

    /// Mark an element as deleted
    pub fn remove(&mut self, id: &ElementId, timestamp: u64) -> Result<(), ListError> {
        if let Some(element) = self.elements.get_mut(id) {
            element.mark_deleted(self.replica, timestamp);
            Ok(())
        } else {
            Err(ListError::new("Element not found".to_string()))
        }
    }

    /// Get an element by ID
    pub fn get(&self, id: &ElementId) -> Option<&ListElement<T>> {
        self.elements.get(id)
    }

    /// Get all visible elements (not deleted)
    pub fn visible_elements(&self) -> Vec<&ListElement<T>> {
        self.elements
            .values()
            .filter(|e| !e.metadata.deleted)
            .collect()
    }

    /// Get all elements including deleted ones
    pub fn all_elements(&self) -> Vec<&ListElement<T>> {
        self.elements.values().collect()
    }

    /// Check if the list contains an element
    pub fn contains(&self, id: &ElementId) -> bool {
        self.elements.contains_key(id)
    }

    /// Get the length of visible elements
    pub fn len(&self) -> usize {
        self.visible_elements().len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.elements.clear();
    }
}

impl<T: Clone + PartialEq + Eq + Send + Sync> CRDT for AddWinsList<T> {
    fn replica_id(&self) -> &ReplicaId {
        &self.replica
    }
}

impl<T: Clone + PartialEq + Eq + Send + Sync> Mergeable for AddWinsList<T> {
    type Error = ListError;

    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        for (id, element) in &other.elements {
            match self.elements.get(id) {
                Some(existing) => {
                    // Conflict resolution: later timestamp wins
                    if element.metadata.modified_at > existing.metadata.modified_at {
                        self.elements.insert(id.clone(), element.clone());
                    }
                }
                None => {
                    // New element, add it
                    self.elements.insert(id.clone(), element.clone());
                }
            }
        }
        Ok(())
    }

    fn has_conflict(&self, other: &Self) -> bool {
        for (id, element) in &other.elements {
            if let Some(existing) = self.elements.get(id) {
                // Check for conflicts: same timestamp but different replica
                if element.metadata.modified_at == existing.metadata.modified_at
                    && element.metadata.last_modified_by != existing.metadata.last_modified_by
                {
                    return true;
                }
            }
        }
        false
    }
}

/// Remove-Wins List CRDT implementation
/// 
/// This implementation completely removes deleted elements.
/// It's more memory-efficient but elements cannot be recovered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoveWinsList<T> {
    /// Configuration
    config: ListConfig,
    /// Elements in the list
    elements: HashMap<ElementId, ListElement<T>>,
    /// Replica ID for this instance
    replica: ReplicaId,
}

impl<T: Clone + PartialEq + Eq + Send + Sync> RemoveWinsList<T> {
    /// Create a new Remove-Wins list
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            config: ListConfig {
                strategy: ListStrategy::RemoveWins,
                preserve_deleted: false,
                max_elements: None,
            },
            elements: HashMap::new(),
            replica,
        }
    }

    /// Create with custom configuration
    pub fn with_config(replica: ReplicaId, config: ListConfig) -> Self {
        Self {
            config,
            elements: HashMap::new(),
            replica,
        }
    }

    /// Add an element to the list
    pub fn add(&mut self, value: T, timestamp: u64) -> ElementId {
        let element = ListElement::new(value, self.replica, timestamp);
        let id = element.id.clone();
        self.elements.insert(id.clone(), element);
        id
    }

    /// Update an existing element
    pub fn update(&mut self, id: &ElementId, value: T, timestamp: u64) -> Result<(), ListError> {
        if let Some(element) = self.elements.get_mut(id) {
            element.value = value;
            element.mark_modified(self.replica, timestamp);
            Ok(())
        } else {
            Err(ListError::new("Element not found".to_string()))
        }
    }

    /// Remove an element completely
    pub fn remove(&mut self, id: &ElementId) -> Result<(), ListError> {
        if self.elements.remove(id).is_some() {
            Ok(())
        } else {
            Err(ListError::new("Element not found".to_string()))
        }
    }

    /// Get an element by ID
    pub fn get(&self, id: &ElementId) -> Option<&ListElement<T>> {
        self.elements.get(id)
    }

    /// Get all elements
    pub fn elements(&self) -> Vec<&ListElement<T>> {
        self.elements.values().collect()
    }

    /// Check if the list contains an element
    pub fn contains(&self, id: &ElementId) -> bool {
        self.elements.contains_key(id)
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.elements.clear();
    }
}

impl<T: Clone + PartialEq + Eq + Send + Sync> CRDT for RemoveWinsList<T> {
    fn replica_id(&self) -> &ReplicaId {
        &self.replica
    }
}

impl<T: Clone + PartialEq + Eq + Send + Sync> Mergeable for RemoveWinsList<T> {
    type Error = ListError;

    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        for (id, element) in &other.elements {
            match self.elements.get(id) {
                Some(existing) => {
                    // Conflict resolution: later timestamp wins
                    if element.metadata.modified_at > existing.metadata.modified_at {
                        self.elements.insert(id.clone(), element.clone());
                    }
                }
                None => {
                    // New element, add it
                    self.elements.insert(id.clone(), element.clone());
                }
            }
        }
        Ok(())
    }

    fn has_conflict(&self, other: &Self) -> bool {
        for (id, element) in &other.elements {
            if let Some(existing) = self.elements.get(id) {
                // Check for conflicts: same timestamp but different replica
                if element.metadata.modified_at == existing.metadata.modified_at
                    && element.metadata.last_modified_by != existing.metadata.last_modified_by
                {
                    return true;
                }
            }
        }
        false
    }
}

/// Last-Write-Wins List CRDT implementation
/// 
/// This implementation uses timestamps to resolve conflicts.
/// The most recent modification always wins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LwwList<T> {
    /// Configuration
    config: ListConfig,
    /// Elements in the list
    elements: HashMap<ElementId, ListElement<T>>,
    /// Replica ID for this instance
    replica: ReplicaId,
}

impl<T: Clone + PartialEq + Eq + Send + Sync> LwwList<T> {
    /// Create a new LWW list
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            config: ListConfig {
                strategy: ListStrategy::LastWriteWins,
                preserve_deleted: true,
                max_elements: None,
            },
            elements: HashMap::new(),
            replica,
        }
    }

    /// Create with custom configuration
    pub fn with_config(replica: ReplicaId, config: ListConfig) -> Self {
        Self {
            config,
            elements: HashMap::new(),
            replica,
        }
    }

    /// Add an element to the list
    pub fn add(&mut self, value: T, timestamp: u64) -> ElementId {
        let element = ListElement::new(value, self.replica, timestamp);
        let id = element.id.clone();
        self.elements.insert(id.clone(), element);
        id
    }

    /// Update an existing element
    pub fn update(&mut self, id: &ElementId, value: T, timestamp: u64) -> Result<(), ListError> {
        if let Some(element) = self.elements.get_mut(id) {
            element.value = value;
            element.mark_modified(self.replica, timestamp);
            Ok(())
        } else {
            Err(ListError::new("Element not found".to_string()))
        }
    }

    /// Mark an element as deleted
    pub fn remove(&mut self, id: &ElementId, timestamp: u64) -> Result<(), ListError> {
        if let Some(element) = self.elements.get_mut(id) {
            element.mark_deleted(self.replica, timestamp);
            Ok(())
        } else {
            Err(ListError::new("Element not found".to_string()))
        }
    }

    /// Get an element by ID
    pub fn get(&self, id: &ElementId) -> Option<&ListElement<T>> {
        self.elements.get(id)
    }

    /// Get all visible elements (not deleted)
    pub fn visible_elements(&self) -> Vec<&ListElement<T>> {
        self.elements
            .values()
            .filter(|e| !e.metadata.deleted)
            .collect()
    }

    /// Get all elements including deleted ones
    pub fn all_elements(&self) -> Vec<&ListElement<T>> {
        self.elements.values().collect()
    }

    /// Check if the list contains an element
    pub fn contains(&self, id: &ElementId) -> bool {
        self.elements.contains_key(id)
    }

    /// Get the length of visible elements
    pub fn len(&self) -> usize {
        self.visible_elements().len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.elements.clear();
    }
}

impl<T: Clone + PartialEq + Eq + Send + Sync> CRDT for LwwList<T> {
    fn replica_id(&self) -> &ReplicaId {
        &self.replica
    }
}

impl<T: Clone + PartialEq + Eq + Send + Sync> Mergeable for LwwList<T> {
    type Error = ListError;

    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        for (id, element) in &other.elements {
            match self.elements.get(id) {
                Some(existing) => {
                    // Conflict resolution: later timestamp wins
                    if element.metadata.modified_at > existing.metadata.modified_at {
                        self.elements.insert(id.clone(), element.clone());
                    }
                }
                None => {
                    // New element, add it
                    self.elements.insert(id.clone(), element.clone());
                }
            }
        }
        Ok(())
    }

    fn has_conflict(&self, other: &Self) -> bool {
        for (id, element) in &other.elements {
            if let Some(existing) = self.elements.get(id) {
                // Check for conflicts: same timestamp but different replica
                if element.metadata.modified_at == existing.metadata.modified_at
                    && element.metadata.last_modified_by != existing.metadata.last_modified_by
                {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ReplicaId;
    use uuid::Uuid;

    fn create_replica(id: u64) -> ReplicaId {
        ReplicaId::from(Uuid::from_u64_pair(0, id))
    }

    #[test]
    fn test_element_id_creation() {
        let replica = create_replica(1);
        let element_id = ElementId::new(replica);
        
        assert_eq!(element_id.replica, replica);
        assert_ne!(element_id.id, Uuid::nil());
    }

    #[test]
    fn test_list_element_creation() {
        let replica = create_replica(1);
        let timestamp = 1234567890;
        let element = ListElement::new("test_value", replica, timestamp);
        
        assert_eq!(element.value, "test_value");
        assert_eq!(element.metadata.created_at, timestamp);
        assert_eq!(element.metadata.modified_at, timestamp);
        assert_eq!(element.metadata.deleted, false);
        assert_eq!(element.metadata.last_modified_by, replica);
    }

    #[test]
    fn test_add_wins_list_basic_operations() {
        let replica = create_replica(1);
        let mut list = AddWinsList::new(replica);
        
        // Add elements
        let id1 = list.add("first", 1000);
        let id2 = list.add("second", 2000);
        
        assert_eq!(list.len(), 2);
        assert!(list.contains(&id1));
        assert!(list.contains(&id2));
        
        // Update element
        list.update(&id1, "updated_first", 3000).unwrap();
        assert_eq!(list.get(&id1).unwrap().value, "updated_first");
        
        // Remove element (marks as deleted)
        list.remove(&id2, 4000).unwrap();
        assert_eq!(list.len(), 1); // Only visible elements
        assert!(list.get(&id2).unwrap().metadata.deleted);
    }

    #[test]
    fn test_remove_wins_list_basic_operations() {
        let replica = create_replica(1);
        let mut list = RemoveWinsList::new(replica);
        
        // Add elements
        let id1 = list.add("first", 1000);
        let id2 = list.add("second", 2000);
        
        assert_eq!(list.len(), 2);
        
        // Remove element completely
        list.remove(&id2).unwrap();
        assert_eq!(list.len(), 1);
        assert!(!list.contains(&id2));
    }

    #[test]
    fn test_lww_list_basic_operations() {
        let replica = create_replica(1);
        let mut list = LwwList::new(replica);
        
        // Add elements
        let id1 = list.add("first", 1000);
        let id2 = list.add("second", 2000);
        
        assert_eq!(list.len(), 2);
        
        // Update element
        list.update(&id1, "updated_first", 3000).unwrap();
        assert_eq!(list.get(&id1).unwrap().value, "updated_first");
        
        // Remove element (marks as deleted)
        list.remove(&id2, 4000).unwrap();
        assert_eq!(list.len(), 1);
        assert!(list.get(&id2).unwrap().metadata.deleted);
    }

    #[test]
    fn test_list_merge() {
        let replica1 = create_replica(1);
        let replica2 = create_replica(2);
        
        let mut list1 = AddWinsList::new(replica1);
        let mut list2 = AddWinsList::new(replica2);
        
        // Add elements to both lists
        let id1 = list1.add("from_replica1", 1000);
        let id2 = list2.add("from_replica2", 2000);
        
        // Merge list2 into list1
        list1.merge(&list2).unwrap();
        
        // Both elements should be present
        assert_eq!(list1.len(), 2);
        assert!(list1.contains(&id1));
        assert!(list1.contains(&id2));
    }

    #[test]
    fn test_list_merge_conflict_resolution() {
        let replica1 = create_replica(1);
        let replica2 = create_replica(2);
        
        let mut list1 = AddWinsList::new(replica1);
        let mut list2 = AddWinsList::new(replica2);
        
        // Create same element with different values
        let id = list1.add("value1", 1000);
        list2.elements.insert(id.clone(), ListElement::new("value2", replica2, 2000));
        
        // Merge - later timestamp should win
        list1.merge(&list2).unwrap();
        assert_eq!(list1.get(&id).unwrap().value, "value2");
    }

    #[test]
    fn test_list_configuration() {
        let replica = create_replica(1);
        let config = ListConfig {
            strategy: ListStrategy::RemoveWins,
            preserve_deleted: false,
            max_elements: Some(100),
        };
        
        let list: AddWinsList<String> = AddWinsList::with_config(replica, config);
        assert_eq!(list.config.strategy, ListStrategy::RemoveWins);
        assert_eq!(list.config.max_elements, Some(100));
    }
}
