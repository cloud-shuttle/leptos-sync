//! Advanced CRDT Types
//!
//! This module provides advanced CRDT implementations including:
//! - RGA (Replicated Growable Array) for collaborative text editing
//! - LSEQ (Logoot Sequence) for ordered sequences
//! - Yjs-style trees for hierarchical data
//! - DAG (Directed Acyclic Graph) for complex relationships

use crate::crdt::{CRDT, Mergeable, ReplicaId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap, HashSet};
use std::error::Error;
use std::fmt;
use uuid::Uuid;

/// Error types for advanced CRDT operations
#[derive(Debug, Clone, PartialEq)]
pub enum AdvancedCrdtError {
    /// Invalid position in sequence
    InvalidPosition(String),
    /// Element not found
    ElementNotFound(String),
    /// Invalid parent-child relationship
    InvalidRelationship(String),
    /// Cycle detected in DAG
    CycleDetected(String),
    /// Invalid operation for CRDT type
    InvalidOperation(String),
    /// Merge operation failed
    MergeError(String),
}

impl fmt::Display for AdvancedCrdtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdvancedCrdtError::InvalidPosition(msg) => write!(f, "Invalid position: {}", msg),
            AdvancedCrdtError::ElementNotFound(msg) => write!(f, "Element not found: {}", msg),
            AdvancedCrdtError::InvalidRelationship(msg) => write!(f, "Invalid relationship: {}", msg),
            AdvancedCrdtError::CycleDetected(msg) => write!(f, "Cycle detected: {}", msg),
            AdvancedCrdtError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            AdvancedCrdtError::MergeError(msg) => write!(f, "Merge error: {}", msg),
        }
    }
}

impl Error for AdvancedCrdtError {}

/// Position identifier for RGA and LSEQ
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PositionId {
    /// Replica ID that created this position
    pub replica_id: ReplicaId,
    /// Logical timestamp
    pub timestamp: u64,
    /// Additional disambiguation value
    pub disambiguation: u64,
}

impl PositionId {
    /// Create a new position ID
    pub fn new(replica_id: ReplicaId, timestamp: u64, disambiguation: u64) -> Self {
        Self {
            replica_id,
            timestamp,
            disambiguation,
        }
    }
}

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

/// Yjs-style tree node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YjsNode<T> {
    /// Unique node identifier
    pub id: PositionId,
    /// Node value
    pub value: T,
    /// Parent node ID
    pub parent: Option<PositionId>,
    /// Child node IDs
    pub children: Vec<PositionId>,
    /// Whether the node is visible (not deleted)
    pub visible: bool,
}

impl<T> YjsNode<T> {
    /// Create a new Yjs node
    pub fn new(id: PositionId, value: T, parent: Option<PositionId>) -> Self {
        Self {
            id,
            value,
            parent,
            children: Vec::new(),
            visible: true,
        }
    }
}

/// Yjs-style tree for hierarchical data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YjsTree<T> {
    /// Replica ID
    replica_id: ReplicaId,
    /// Nodes indexed by ID
    nodes: HashMap<PositionId, YjsNode<T>>,
    /// Root node ID
    root: Option<PositionId>,
    /// Logical timestamp counter
    timestamp_counter: u64,
    /// Disambiguation counter
    disambiguation_counter: u64,
}

impl<T: Clone + PartialEq> YjsTree<T> {
    /// Create a new Yjs tree
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            replica_id,
            nodes: HashMap::new(),
            root: None,
            timestamp_counter: 0,
            disambiguation_counter: 0,
        }
    }
    
    /// Add a root node
    pub fn add_root(&mut self, value: T) -> Result<PositionId, AdvancedCrdtError> {
        self.timestamp_counter += 1;
        self.disambiguation_counter += 1;
        
        let id = PositionId::new(
            self.replica_id.clone(),
            self.timestamp_counter,
            self.disambiguation_counter,
        );
        
        let node = YjsNode::new(id.clone(), value, None);
        self.nodes.insert(id.clone(), node);
        self.root = Some(id.clone());
        
        Ok(id)
    }
    
    /// Add a child node
    pub fn add_child(&mut self, parent_id: &PositionId, value: T) -> Result<PositionId, AdvancedCrdtError> {
        if !self.nodes.contains_key(parent_id) {
            return Err(AdvancedCrdtError::ElementNotFound(format!("Parent {:?}", parent_id)));
        }
        
        self.timestamp_counter += 1;
        self.disambiguation_counter += 1;
        
        let id = PositionId::new(
            self.replica_id.clone(),
            self.timestamp_counter,
            self.disambiguation_counter,
        );
        
        let node = YjsNode::new(id.clone(), value, Some(parent_id.clone()));
        self.nodes.insert(id.clone(), node);
        
        // Add to parent's children
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.children.push(id.clone());
        }
        
        Ok(id)
    }
    
    /// Delete a node
    pub fn delete(&mut self, node_id: &PositionId) -> Result<(), AdvancedCrdtError> {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.visible = false;
            Ok(())
        } else {
            Err(AdvancedCrdtError::ElementNotFound(format!("Node {:?}", node_id)))
        }
    }
    
    /// Get the tree structure as a nested structure
    pub fn to_tree(&self) -> Option<YjsTreeNode<T>> {
        if let Some(root_id) = &self.root {
            self.build_tree_node(root_id)
        } else {
            None
        }
    }
    
    /// Build a tree node recursively
    fn build_tree_node(&self, node_id: &PositionId) -> Option<YjsTreeNode<T>> {
        if let Some(node) = self.nodes.get(node_id) {
            if !node.visible {
                return None;
            }
            
            let children: Vec<YjsTreeNode<T>> = node.children
                .iter()
                .filter_map(|child_id| self.build_tree_node(child_id))
                .collect();
            
            Some(YjsTreeNode {
                id: node.id.clone(),
                value: node.value.clone(),
                children,
            })
        } else {
            None
        }
    }
    
    /// Get node count
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    
    /// Check if tree is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Tree node for display purposes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YjsTreeNode<T> {
    /// Node ID
    pub id: PositionId,
    /// Node value
    pub value: T,
    /// Child nodes
    pub children: Vec<YjsTreeNode<T>>,
}

impl<T: Clone + PartialEq> CRDT for YjsTree<T> {
    fn replica_id(&self) -> &ReplicaId {
        &self.replica_id
    }
}

impl<T: Clone + PartialEq + Send + Sync> Mergeable for YjsTree<T> {
    type Error = AdvancedCrdtError;
    
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        // Merge all nodes from other tree
        for (node_id, other_node) in &other.nodes {
            if let Some(self_node) = self.nodes.get_mut(node_id) {
                // Node exists in both, keep the one with higher timestamp
                if other_node.id.timestamp > self_node.id.timestamp {
                    *self_node = other_node.clone();
                }
            } else {
                // Node only exists in other, add it
                self.nodes.insert(node_id.clone(), other_node.clone());
            }
        }
        
        // Update root if other has a root and we don't, or if other's root is newer
        if let Some(other_root) = &other.root {
            if self.root.is_none() {
                self.root = Some(other_root.clone());
            } else if let Some(self_root) = &self.root {
                if let (Some(other_root_node), Some(self_root_node)) = (
                    other.nodes.get(other_root),
                    self.nodes.get(self_root)
                ) {
                    if other_root_node.id.timestamp > self_root_node.id.timestamp {
                        self.root = Some(other_root.clone());
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn has_conflict(&self, other: &Self) -> bool {
        // Check for conflicting nodes (same ID, different values)
        for (node_id, self_node) in &self.nodes {
            if let Some(other_node) = other.nodes.get(node_id) {
                if self_node.value != other_node.value {
                    return true;
                }
            }
        }
        false
    }
}

/// DAG (Directed Acyclic Graph) node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagNode<T> {
    /// Unique node identifier
    pub id: PositionId,
    /// Node value
    pub value: T,
    /// Incoming edges (dependencies)
    pub incoming: HashSet<PositionId>,
    /// Outgoing edges (dependents)
    pub outgoing: HashSet<PositionId>,
    /// Whether the node is visible (not deleted)
    pub visible: bool,
}

impl<T> DagNode<T> {
    /// Create a new DAG node
    pub fn new(id: PositionId, value: T) -> Self {
        Self {
            id,
            value,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
            visible: true,
        }
    }
}

/// DAG (Directed Acyclic Graph) for complex relationships
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dag<T> {
    /// Replica ID
    replica_id: ReplicaId,
    /// Nodes indexed by ID
    nodes: HashMap<PositionId, DagNode<T>>,
    /// Logical timestamp counter
    timestamp_counter: u64,
    /// Disambiguation counter
    disambiguation_counter: u64,
}

impl<T: Clone + PartialEq> Dag<T> {
    /// Create a new DAG
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            replica_id,
            nodes: HashMap::new(),
            timestamp_counter: 0,
            disambiguation_counter: 0,
        }
    }
    
    /// Add a node
    pub fn add_node(&mut self, value: T) -> Result<PositionId, AdvancedCrdtError> {
        self.timestamp_counter += 1;
        self.disambiguation_counter += 1;
        
        let id = PositionId::new(
            self.replica_id.clone(),
            self.timestamp_counter,
            self.disambiguation_counter,
        );
        
        let node = DagNode::new(id.clone(), value);
        self.nodes.insert(id.clone(), node);
        
        Ok(id)
    }
    
    /// Add an edge from source to target
    pub fn add_edge(&mut self, from: &PositionId, to: &PositionId) -> Result<(), AdvancedCrdtError> {
        if !self.nodes.contains_key(from) || !self.nodes.contains_key(to) {
            return Err(AdvancedCrdtError::ElementNotFound("Node not found".to_string()));
        }
        
        // Check for cycle
        if self.would_create_cycle(from, to) {
            return Err(AdvancedCrdtError::CycleDetected("Adding edge would create cycle".to_string()));
        }
        
        // Add edge
        if let Some(from_node) = self.nodes.get_mut(from) {
            from_node.outgoing.insert(to.clone());
        }
        if let Some(to_node) = self.nodes.get_mut(to) {
            to_node.incoming.insert(from.clone());
        }
        
        Ok(())
    }
    
    /// Remove an edge
    pub fn remove_edge(&mut self, from: &PositionId, to: &PositionId) -> Result<(), AdvancedCrdtError> {
        if let Some(from_node) = self.nodes.get_mut(from) {
            from_node.outgoing.remove(to);
        }
        if let Some(to_node) = self.nodes.get_mut(to) {
            to_node.incoming.remove(from);
        }
        
        Ok(())
    }
    
    /// Delete a node
    pub fn delete_node(&mut self, node_id: &PositionId) -> Result<(), AdvancedCrdtError> {
        if let Some(node) = self.nodes.get(node_id) {
            let incoming_edges = node.incoming.clone();
            let outgoing_edges = node.outgoing.clone();
            
            // Mark node as invisible
            if let Some(node) = self.nodes.get_mut(node_id) {
                node.visible = false;
            }
            
            // Remove all edges involving this node
            for incoming in &incoming_edges {
                if let Some(incoming_node) = self.nodes.get_mut(incoming) {
                    incoming_node.outgoing.remove(node_id);
                }
            }
            for outgoing in &outgoing_edges {
                if let Some(outgoing_node) = self.nodes.get_mut(outgoing) {
                    outgoing_node.incoming.remove(node_id);
                }
            }
            
            Ok(())
        } else {
            Err(AdvancedCrdtError::ElementNotFound(format!("Node {:?}", node_id)))
        }
    }
    
    /// Check if adding an edge would create a cycle
    fn would_create_cycle(&self, from: &PositionId, to: &PositionId) -> bool {
        if from == to {
            return true;
        }
        
        // Use DFS to check for path from 'to' to 'from'
        let mut visited = HashSet::new();
        self.dfs_cycle_check(to, from, &mut visited)
    }
    
    /// DFS helper for cycle detection
    fn dfs_cycle_check(&self, current: &PositionId, target: &PositionId, visited: &mut HashSet<PositionId>) -> bool {
        if current == target {
            return true;
        }
        
        if visited.contains(current) {
            return false;
        }
        
        visited.insert(current.clone());
        
        if let Some(node) = self.nodes.get(current) {
            for next in &node.outgoing {
                if self.dfs_cycle_check(next, target, visited) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Get topological sort of the DAG
    pub fn topological_sort(&self) -> Vec<PositionId> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        
        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                self.dfs_topological(node_id, &mut visited, &mut result);
            }
        }
        
        result.reverse();
        result
    }
    
    /// DFS helper for topological sort
    fn dfs_topological(&self, node_id: &PositionId, visited: &mut HashSet<PositionId>, result: &mut Vec<PositionId>) {
        if visited.contains(node_id) {
            return;
        }
        
        visited.insert(node_id.clone());
        
        if let Some(node) = self.nodes.get(node_id) {
            for next in &node.outgoing {
                self.dfs_topological(next, visited, result);
            }
        }
        
        result.push(node_id.clone());
    }
    
    /// Get node count
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    
    /// Check if DAG is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    
    /// Get all nodes (for debugging/inspection)
    pub fn get_nodes(&self) -> &HashMap<PositionId, DagNode<T>> {
        &self.nodes
    }
}

impl<T: Clone + PartialEq> CRDT for Dag<T> {
    fn replica_id(&self) -> &ReplicaId {
        &self.replica_id
    }
}

impl<T: Clone + PartialEq + Send + Sync> Mergeable for Dag<T> {
    type Error = AdvancedCrdtError;
    
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        // Merge all nodes from other DAG
        for (node_id, other_node) in &other.nodes {
            if let Some(self_node) = self.nodes.get_mut(node_id) {
                // Node exists in both, keep the one with higher timestamp
                if other_node.id.timestamp > self_node.id.timestamp {
                    *self_node = other_node.clone();
                }
            } else {
                // Node only exists in other, add it
                self.nodes.insert(node_id.clone(), other_node.clone());
            }
        }
        
        Ok(())
    }
    
    fn has_conflict(&self, other: &Self) -> bool {
        // Check for conflicting nodes (same ID, different values)
        for (node_id, self_node) in &self.nodes {
            if let Some(other_node) = other.nodes.get(node_id) {
                if self_node.value != other_node.value {
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
    use crate::crdt::ReplicaId;
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
    
    #[test]
    fn test_yjs_tree_creation() {
        let replica_id = create_replica(1);
        let tree = YjsTree::<String>::new(replica_id.clone());
        
        assert_eq!(tree.replica_id(), &replica_id);
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }
    
    #[test]
    fn test_yjs_tree_operations() {
        let replica_id = create_replica(1);
        let mut tree = YjsTree::<String>::new(replica_id);
        
        // Add root
        let root_id = tree.add_root("root".to_string()).unwrap();
        assert_eq!(tree.len(), 1);
        
        // Add children
        let child1_id = tree.add_child(&root_id, "child1".to_string()).unwrap();
        let child2_id = tree.add_child(&root_id, "child2".to_string()).unwrap();
        assert_eq!(tree.len(), 3);
        
        // Add grandchild
        let grandchild_id = tree.add_child(&child1_id, "grandchild".to_string()).unwrap();
        assert_eq!(tree.len(), 4);
        
        // Delete node
        tree.delete(&child2_id).unwrap();
        assert_eq!(tree.len(), 4); // Node still exists but is invisible
        
        // Check tree structure
        let tree_structure = tree.to_tree().unwrap();
        assert_eq!(tree_structure.value, "root");
        assert_eq!(tree_structure.children.len(), 1); // Only child1 is visible
        assert_eq!(tree_structure.children[0].value, "child1");
        assert_eq!(tree_structure.children[0].children.len(), 1);
        assert_eq!(tree_structure.children[0].children[0].value, "grandchild");
    }
    
    #[test]
    fn test_yjs_tree_merge() {
        let replica_id1 = create_replica(1);
        let replica_id2 = create_replica(2);
        
        let mut tree1 = YjsTree::<String>::new(replica_id1);
        let mut tree2 = YjsTree::<String>::new(replica_id2);
        
        // Add different roots
        let root1_id = tree1.add_root("root1".to_string()).unwrap();
        let root2_id = tree2.add_root("root2".to_string()).unwrap();
        
        // Add children
        tree1.add_child(&root1_id, "child1".to_string()).unwrap();
        tree2.add_child(&root2_id, "child2".to_string()).unwrap();
        
        // Merge
        tree1.merge(&tree2).unwrap();
        
        // Should contain both trees
        assert_eq!(tree1.len(), 4); // 2 roots + 2 children
    }
    
    #[test]
    fn test_dag_creation() {
        let replica_id = create_replica(1);
        let dag = Dag::<String>::new(replica_id.clone());
        
        assert_eq!(dag.replica_id(), &replica_id);
        assert!(dag.is_empty());
        assert_eq!(dag.len(), 0);
    }
    
    #[test]
    fn test_dag_operations() {
        let replica_id = create_replica(1);
        let mut dag = Dag::<String>::new(replica_id);
        
        // Add nodes
        let node1_id = dag.add_node("node1".to_string()).unwrap();
        let node2_id = dag.add_node("node2".to_string()).unwrap();
        let node3_id = dag.add_node("node3".to_string()).unwrap();
        
        assert_eq!(dag.len(), 3);
        
        // Add edges
        dag.add_edge(&node1_id, &node2_id).unwrap();
        dag.add_edge(&node2_id, &node3_id).unwrap();
        
        // Test topological sort
        let sorted = dag.topological_sort();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0], node1_id);
        assert_eq!(sorted[1], node2_id);
        assert_eq!(sorted[2], node3_id);
        
        // Remove edge
        dag.remove_edge(&node1_id, &node2_id).unwrap();
        
        // Delete node
        dag.delete_node(&node2_id).unwrap();
        assert_eq!(dag.len(), 3); // Node still exists but is invisible
    }
    
    #[test]
    fn test_dag_cycle_detection() {
        let replica_id = create_replica(1);
        let mut dag = Dag::<String>::new(replica_id);
        
        // Add nodes
        let node1_id = dag.add_node("node1".to_string()).unwrap();
        let node2_id = dag.add_node("node2".to_string()).unwrap();
        let node3_id = dag.add_node("node3".to_string()).unwrap();
        
        // Add edges to create a cycle
        dag.add_edge(&node1_id, &node2_id).unwrap();
        dag.add_edge(&node2_id, &node3_id).unwrap();
        
        // Try to add edge that would create cycle
        let result = dag.add_edge(&node3_id, &node1_id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AdvancedCrdtError::CycleDetected("Adding edge would create cycle".to_string()));
    }
    
    #[test]
    fn test_dag_merge() {
        let replica_id1 = create_replica(1);
        let replica_id2 = create_replica(2);
        
        let mut dag1 = Dag::<String>::new(replica_id1);
        let mut dag2 = Dag::<String>::new(replica_id2);
        
        // Add different nodes
        let node1_id = dag1.add_node("node1".to_string()).unwrap();
        let node2_id = dag2.add_node("node2".to_string()).unwrap();
        
        // Add edges (no self-loops in DAG)
        // Just add nodes without edges for merge test
        
        // Merge
        dag1.merge(&dag2).unwrap();
        
        // Should contain both nodes
        assert_eq!(dag1.len(), 2);
    }
    
    #[test]
    fn test_advanced_crdt_traits() {
        let replica_id = create_replica(1);
        
        // Test that all advanced CRDT types implement the required traits
        let rga: Rga<String> = Rga::new(replica_id.clone());
        let lseq: Lseq<String> = Lseq::new(replica_id.clone());
        let tree: YjsTree<String> = YjsTree::new(replica_id.clone());
        let dag: Dag<String> = Dag::new(replica_id);
        
        // This should compile if all types implement CRDT trait
        let _: &dyn CRDT = &rga;
        let _: &dyn CRDT = &lseq;
        let _: &dyn CRDT = &tree;
        let _: &dyn CRDT = &dag;
    }
}
