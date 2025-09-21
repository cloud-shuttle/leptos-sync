//! RGA (Replicated Growable Array) for collaborative text editing

use super::common::{PositionId, AdvancedCrdtError};
use super::super::{CRDT, Mergeable, ReplicaId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RGA (Replicated Growable Array) element
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RgaElement<T> {
    /// Unique position identifier
    pub position: PositionId,
    /// Element value
    pub value: T,
    /// Whether the element is visible (not deleted)
    pub visible: bool,
    /// Reference to previous element
    pub prev: Option<PositionId>,
}

impl<T> RgaElement<T> {
    /// Create a new RGA element
    pub fn new(position: PositionId, value: T, prev: Option<PositionId>) -> Self {
        Self {
            position,
            value,
            visible: true,
            prev,
        }
    }
}

/// RGA (Replicated Growable Array) for collaborative text editing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rga<T> {
    /// Replica ID
    replica_id: ReplicaId,
    /// Elements indexed by position
    elements: HashMap<PositionId, RgaElement<T>>,
    /// Logical timestamp counter
    timestamp_counter: u64,
    /// Disambiguation counter
    disambiguation_counter: u64,
}

impl<T: Clone + PartialEq> Rga<T> {
    /// Create a new RGA
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            replica_id,
            elements: HashMap::new(),
            timestamp_counter: 0,
            disambiguation_counter: 0,
        }
    }
    
    /// Insert an element after the given position
    pub fn insert_after(&mut self, value: T, after: Option<PositionId>) -> Result<PositionId, AdvancedCrdtError> {
        self.timestamp_counter += 1;
        self.disambiguation_counter += 1;
        
        let position = PositionId::new(
            self.replica_id.clone(),
            self.timestamp_counter,
            self.disambiguation_counter,
        );
        
        let element = RgaElement::new(position.clone(), value, after);
        self.elements.insert(position.clone(), element);
        
        Ok(position)
    }
    
    /// Delete an element at the given position
    pub fn delete(&mut self, position: &PositionId) -> Result<(), AdvancedCrdtError> {
        if let Some(element) = self.elements.get_mut(position) {
            element.visible = false;
            Ok(())
        } else {
            Err(AdvancedCrdtError::ElementNotFound(format!("Position {:?}", position)))
        }
    }
    
    /// Get the visible elements in order
    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::new();
        
        // For RGA, we need to handle multiple root elements (elements with prev: None)
        // We'll collect all visible elements and sort them by position
        let mut elements: Vec<_> = self.elements.values()
            .filter(|e| e.visible)
            .collect();
        
        // Sort by position (replica_id, timestamp, disambiguation)
        elements.sort_by(|a, b| a.position.cmp(&b.position));
        
        // Add values to result
        for element in elements {
            result.push(element.value.clone());
        }
        
        result
    }
    
    /// Find the first element in the sequence
    fn find_first_element(&self) -> Option<PositionId> {
        // Find element with no previous element
        self.elements.values()
            .find(|e| e.prev.is_none())
            .map(|e| e.position.clone())
    }
    
    /// Find the next element after the given position
    fn find_next_element(&self, position: &PositionId) -> Option<PositionId> {
        self.elements.values()
            .find(|e| e.prev.as_ref() == Some(position))
            .map(|e| e.position.clone())
    }
    
    /// Get element count
    pub fn len(&self) -> usize {
        self.elements.len()
    }
    
    /// Check if RGA is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl<T: Clone + PartialEq> CRDT for Rga<T> {
    fn replica_id(&self) -> &ReplicaId {
        &self.replica_id
    }
}

impl<T: Clone + PartialEq + Send + Sync> Mergeable for Rga<T> {
    type Error = AdvancedCrdtError;
    
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        // Merge all elements from other RGA
        for (position, other_element) in &other.elements {
            if let Some(self_element) = self.elements.get_mut(position) {
                // Element exists in both, keep the one with higher timestamp
                if other_element.position.timestamp > self_element.position.timestamp {
                    *self_element = other_element.clone();
                }
            } else {
                // Element only exists in other, add it
                self.elements.insert(position.clone(), other_element.clone());
            }
        }
        
        Ok(())
    }
    
    fn has_conflict(&self, other: &Self) -> bool {
        // Check for conflicting elements (same position, different values)
        for (position, self_element) in &self.elements {
            if let Some(other_element) = other.elements.get(position) {
                if self_element.value != other_element.value {
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
    use super::super::super::ReplicaId;
    use uuid::Uuid;
    
    fn create_replica(id: u64) -> ReplicaId {
        ReplicaId::from(Uuid::from_u64_pair(0, id))
    }
    
    #[test]
    fn test_rga_creation() {
        let replica_id = create_replica(1);
        let rga = Rga::<String>::new(replica_id.clone());
        
        assert_eq!(rga.replica_id(), &replica_id);
        assert!(rga.is_empty());
        assert_eq!(rga.len(), 0);
    }
    
    #[test]
    fn test_rga_insert_and_delete() {
        let replica_id = create_replica(1);
        let mut rga = Rga::<String>::new(replica_id);
        
        // Insert elements
        let pos1 = rga.insert_after("hello".to_string(), None).unwrap();
        let pos2 = rga.insert_after("world".to_string(), Some(pos1.clone())).unwrap();
        let pos3 = rga.insert_after("!".to_string(), Some(pos2.clone())).unwrap();
        
        assert_eq!(rga.len(), 3);
        assert_eq!(rga.to_vec(), vec!["hello", "world", "!"]);
        
        // Delete middle element
        rga.delete(&pos2).unwrap();
        assert_eq!(rga.to_vec(), vec!["hello", "!"]);
        
        // Delete first element
        rga.delete(&pos1).unwrap();
        assert_eq!(rga.to_vec(), vec!["!"]);
        
        // Delete last element
        rga.delete(&pos3).unwrap();
        assert_eq!(rga.to_vec(), Vec::<String>::new());
    }
    
    #[test]
    fn test_rga_merge() {
        let replica_id1 = create_replica(1);
        let replica_id2 = create_replica(2);
        
        let mut rga1 = Rga::<String>::new(replica_id1);
        let mut rga2 = Rga::<String>::new(replica_id2);
        
        // Insert different elements
        let _pos1 = rga1.insert_after("hello".to_string(), None).unwrap();
        let _pos2 = rga2.insert_after("world".to_string(), None).unwrap();
        
        // Merge
        rga1.merge(&rga2).unwrap();
        
        // Should contain both elements (RGA merge adds all elements)
        let elements = rga1.to_vec();
        assert!(elements.contains(&"hello".to_string()));
        assert!(elements.contains(&"world".to_string()));
    }
}
