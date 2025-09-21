//! LSEQ (Logoot Sequence) for ordered sequences

use super::common::{PositionId, AdvancedCrdtError};
use super::super::{CRDT, Mergeable, ReplicaId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// LSEQ (Logoot Sequence) element
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LseqElement<T> {
    /// Unique position identifier
    pub position: PositionId,
    /// Element value
    pub value: T,
    /// Whether the element is visible (not deleted)
    pub visible: bool,
}

impl<T> LseqElement<T> {
    /// Create a new LSEQ element
    pub fn new(position: PositionId, value: T) -> Self {
        Self {
            position,
            value,
            visible: true,
        }
    }
}

/// LSEQ (Logoot Sequence) for ordered sequences
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lseq<T> {
    /// Replica ID
    replica_id: ReplicaId,
    /// Elements indexed by position
    elements: BTreeMap<PositionId, LseqElement<T>>,
    /// Logical timestamp counter
    timestamp_counter: u64,
    /// Disambiguation counter
    disambiguation_counter: u64,
}

impl<T: Clone + PartialEq> Lseq<T> {
    /// Create a new LSEQ
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            replica_id,
            elements: BTreeMap::new(),
            timestamp_counter: 0,
            disambiguation_counter: 0,
        }
    }
    
    /// Insert an element at the given position
    pub fn insert(&mut self, value: T, _position: Option<PositionId>) -> Result<PositionId, AdvancedCrdtError> {
        self.timestamp_counter += 1;
        self.disambiguation_counter += 1;
        
        let new_position = PositionId::new(
            self.replica_id.clone(),
            self.timestamp_counter,
            self.disambiguation_counter,
        );
        
        let element = LseqElement::new(new_position.clone(), value);
        self.elements.insert(new_position.clone(), element);
        
        Ok(new_position)
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
        self.elements.values()
            .filter(|e| e.visible)
            .map(|e| e.value.clone())
            .collect()
    }
    
    /// Get element count
    pub fn len(&self) -> usize {
        self.elements.len()
    }
    
    /// Check if LSEQ is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
    
    /// Get all elements (for debugging/inspection)
    pub fn get_elements(&self) -> &BTreeMap<PositionId, LseqElement<T>> {
        &self.elements
    }
}

impl<T: Clone + PartialEq> CRDT for Lseq<T> {
    fn replica_id(&self) -> &ReplicaId {
        &self.replica_id
    }
}

impl<T: Clone + PartialEq + Send + Sync> Mergeable for Lseq<T> {
    type Error = AdvancedCrdtError;
    
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        // Merge all elements from other LSEQ
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
    fn test_lseq_creation() {
        let replica_id = create_replica(1);
        let lseq = Lseq::<String>::new(replica_id.clone());
        
        assert_eq!(lseq.replica_id(), &replica_id);
        assert!(lseq.is_empty());
        assert_eq!(lseq.len(), 0);
    }
    
    #[test]
    fn test_lseq_insert_and_delete() {
        let replica_id = create_replica(1);
        let mut lseq = Lseq::<String>::new(replica_id);
        
        // Insert elements
        let pos1 = lseq.insert("hello".to_string(), None).unwrap();
        let pos2 = lseq.insert("world".to_string(), None).unwrap();
        let pos3 = lseq.insert("!".to_string(), None).unwrap();
        
        assert_eq!(lseq.len(), 3);
        let elements = lseq.to_vec();
        assert!(elements.contains(&"hello".to_string()));
        assert!(elements.contains(&"world".to_string()));
        assert!(elements.contains(&"!".to_string()));
        
        // Delete element
        lseq.delete(&pos2).unwrap();
        let elements = lseq.to_vec();
        assert!(elements.contains(&"hello".to_string()));
        assert!(!elements.contains(&"world".to_string()));
        assert!(elements.contains(&"!".to_string()));
    }
    
    #[test]
    fn test_lseq_merge() {
        let replica_id1 = create_replica(1);
        let replica_id2 = create_replica(2);
        
        let mut lseq1 = Lseq::<String>::new(replica_id1);
        let mut lseq2 = Lseq::<String>::new(replica_id2);
        
        // Insert different elements
        lseq1.insert("hello".to_string(), None).unwrap();
        lseq2.insert("world".to_string(), None).unwrap();
        
        // Merge
        lseq1.merge(&lseq2).unwrap();
        
        // Should contain both elements
        let elements = lseq1.to_vec();
        assert!(elements.contains(&"hello".to_string()));
        assert!(elements.contains(&"world".to_string()));
    }
}
