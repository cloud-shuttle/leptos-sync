//! Yjs-style tree for hierarchical data

use super::common::{PositionId, AdvancedCrdtError};
use super::super::{CRDT, Mergeable, ReplicaId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::ReplicaId;
    use uuid::Uuid;
    
    fn create_replica(id: u64) -> ReplicaId {
        ReplicaId::from(Uuid::from_u64_pair(0, id))
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
}
