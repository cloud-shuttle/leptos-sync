//! Add-Wins Tree CRDT implementation

use super::super::{CRDT, Mergeable, ReplicaId};
use super::{config::TreeConfig, error::TreeError, types::{NodeId, TreeNode}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Add-Wins Tree CRDT implementation
/// 
/// This implementation ensures that nodes are never completely lost.
/// Deleted nodes are marked as deleted but preserved for potential recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddWinsTree<T> {
    /// Configuration
    config: TreeConfig,
    /// Nodes in the tree
    nodes: HashMap<NodeId, TreeNode<T>>,
    /// Replica ID for this instance
    replica: ReplicaId,
}

impl<T: Clone + PartialEq + Eq + Send + Sync> AddWinsTree<T> {
    /// Create a new Add-Wins tree
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            config: TreeConfig::default(),
            nodes: HashMap::new(),
            replica,
        }
    }

    /// Create with custom configuration
    pub fn with_config(replica: ReplicaId, config: TreeConfig) -> Self {
        Self {
            config,
            nodes: HashMap::new(),
            replica,
        }
    }

    /// Add a root node to the tree
    pub fn add_root(&mut self, value: T, timestamp: u64) -> NodeId {
        let node = TreeNode::new(value, self.replica, timestamp);
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);
        id
    }

    /// Add a child node
    pub fn add_child(&mut self, parent_id: &NodeId, value: T, timestamp: u64) -> Result<NodeId, TreeError> {
        if !self.nodes.contains_key(parent_id) {
            return Err(TreeError::new("Parent node not found".to_string()));
        }

        let node = TreeNode::new_child(value, self.replica, timestamp, parent_id.clone());
        let id = node.id.clone();
        
        // Add the child node
        self.nodes.insert(id.clone(), node);
        
        // Update parent's children list
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.add_child(id.clone());
        }
        
        Ok(id)
    }

    /// Update an existing node
    pub fn update(&mut self, id: &NodeId, value: T, timestamp: u64) -> Result<(), TreeError> {
        if let Some(node) = self.nodes.get_mut(id) {
            node.value = value;
            node.mark_modified(self.replica, timestamp);
            Ok(())
        } else {
            Err(TreeError::new("Node not found".to_string()))
        }
    }

    /// Mark a node as deleted
    pub fn remove(&mut self, id: &NodeId, timestamp: u64) -> Result<(), TreeError> {
        if let Some(node) = self.nodes.get_mut(id) {
            node.mark_deleted(self.replica, timestamp);
            Ok(())
        } else {
            Err(TreeError::new("Node not found".to_string()))
        }
    }

    /// Move a node to a new parent
    pub fn move_node(&mut self, id: &NodeId, new_parent_id: &NodeId) -> Result<(), TreeError> {
        if !self.nodes.contains_key(id) || !self.nodes.contains_key(new_parent_id) {
            return Err(TreeError::new("Node not found".to_string()));
        }

        // Get the old parent ID first
        let old_parent_id = self.nodes.get(id).and_then(|node| node.parent.clone());
        
        // Remove from old parent
        if let Some(old_parent_id) = old_parent_id {
            if let Some(old_parent) = self.nodes.get_mut(&old_parent_id) {
                old_parent.remove_child(id);
            }
        }
        
        // Update the node's parent reference
        if let Some(node) = self.nodes.get_mut(id) {
            node.parent = Some(new_parent_id.clone());
        }
        
        // Add to new parent
        if let Some(new_parent) = self.nodes.get_mut(new_parent_id) {
            new_parent.add_child(id.clone());
        }
        
        Ok(())
    }

    /// Get a node by ID
    pub fn get(&self, id: &NodeId) -> Option<&TreeNode<T>> {
        self.nodes.get(id)
    }

    /// Get all visible nodes (not deleted)
    pub fn visible_nodes(&self) -> Vec<&TreeNode<T>> {
        self.nodes
            .values()
            .filter(|n| !n.metadata.deleted)
            .collect()
    }

    /// Get all nodes including deleted ones
    pub fn all_nodes(&self) -> Vec<&TreeNode<T>> {
        self.nodes.values().collect()
    }

    /// Get root nodes
    pub fn roots(&self) -> Vec<&TreeNode<T>> {
        self.nodes
            .values()
            .filter(|n| n.parent.is_none() && !n.metadata.deleted)
            .collect()
    }

    /// Get children of a node
    pub fn children(&self, id: &NodeId) -> Vec<&TreeNode<T>> {
        if let Some(node) = self.nodes.get(id) {
            node.children
                .iter()
                .filter_map(|child_id| self.nodes.get(child_id))
                .filter(|n| !n.metadata.deleted)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get descendants of a node (recursive)
    pub fn descendants(&self, id: &NodeId) -> Vec<&TreeNode<T>> {
        let mut descendants = Vec::new();
        self.collect_descendants(id, &mut descendants);
        descendants
    }
    
    fn collect_descendants<'a>(&'a self, id: &NodeId, descendants: &mut Vec<&'a TreeNode<T>>) {
        if let Some(node) = self.nodes.get(id) {
            if !node.metadata.deleted {
                for child_id in &node.children {
                    if let Some(child_node) = self.nodes.get(child_id) {
                        if !child_node.metadata.deleted {
                            descendants.push(child_node);
                            self.collect_descendants(child_id, descendants);
                        }
                    }
                }
            }
        }
    }

    /// Check if the tree contains a node
    pub fn contains(&self, id: &NodeId) -> bool {
        self.nodes.contains_key(id)
    }

    /// Get the number of visible nodes
    pub fn len(&self) -> usize {
        self.visible_nodes().len()
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all nodes
    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    /// Get the configuration
    pub fn config(&self) -> &TreeConfig {
        &self.config
    }
}

impl<T: Clone + PartialEq + Eq + Send + Sync> CRDT for AddWinsTree<T> {
    fn replica_id(&self) -> &ReplicaId {
        &self.replica
    }
}

impl<T: Clone + PartialEq + Eq + Send + Sync> Mergeable for AddWinsTree<T> {
    type Error = TreeError;

    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        for (id, node) in &other.nodes {
            match self.nodes.get(id) {
                Some(existing) => {
                    // Conflict resolution: later timestamp wins
                    if node.metadata.modified_at > existing.metadata.modified_at {
                        self.nodes.insert(id.clone(), node.clone());
                    }
                }
                None => {
                    // New node, add it
                    self.nodes.insert(id.clone(), node.clone());
                }
            }
        }
        Ok(())
    }

    fn has_conflict(&self, other: &Self) -> bool {
        for (id, node) in &other.nodes {
            if let Some(existing) = self.nodes.get(id) {
                // Check for conflicts: same timestamp but different replica
                if node.metadata.modified_at == existing.metadata.modified_at
                    && node.metadata.last_modified_by != existing.metadata.last_modified_by
                {
                    return true;
                }
            }
        }
        false
    }
}
