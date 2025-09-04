use super::{CRDT, Mergeable, ReplicaId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;

/// Custom error type for tree operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeError {
    message: String,
}

impl TreeError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for TreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TreeError: {}", self.message)
    }
}

impl Error for TreeError {}

/// Unique identifier for a tree node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId {
    /// Unique identifier for the node
    pub id: Uuid,
    /// Replica that created the node
    pub replica: ReplicaId,
}

impl NodeId {
    /// Create a new node ID
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            id: Uuid::new_v4(),
            replica,
        }
    }

    /// Create a node ID from existing UUID and replica
    pub fn from_parts(id: Uuid, replica: ReplicaId) -> Self {
        Self { id, replica }
    }
}

/// Metadata for a tree node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// When the node was created
    pub created_at: u64,
    /// When the node was last modified
    pub modified_at: u64,
    /// Whether the node is marked as deleted
    pub deleted: bool,
    /// Replica that last modified the node
    pub last_modified_by: ReplicaId,
}

impl NodeMetadata {
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

/// A tree node with its metadata and relationships
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeNode<T> {
    /// Unique identifier
    pub id: NodeId,
    /// The actual value
    pub value: T,
    /// Metadata
    pub metadata: NodeMetadata,
    /// Parent node ID (None for root)
    pub parent: Option<NodeId>,
    /// Child node IDs
    pub children: Vec<NodeId>,
}

impl<T> TreeNode<T> {
    /// Create a new tree node
    pub fn new(value: T, replica: ReplicaId, timestamp: u64) -> Self {
        Self {
            id: NodeId::new(replica),
            value,
            metadata: NodeMetadata::new(replica, timestamp),
            parent: None,
            children: Vec::new(),
        }
    }

    /// Create a child node
    pub fn new_child(value: T, replica: ReplicaId, timestamp: u64, parent: NodeId) -> Self {
        Self {
            id: NodeId::new(replica),
            value,
            metadata: NodeMetadata::new(replica, timestamp),
            parent: Some(parent),
            children: Vec::new(),
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

    /// Add a child
    pub fn add_child(&mut self, child_id: NodeId) {
        self.children.push(child_id);
    }

    /// Remove a child
    pub fn remove_child(&mut self, child_id: &NodeId) -> bool {
        if let Some(pos) = self.children.iter().position(|id| id == child_id) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }
}

/// Strategy for handling tree conflicts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeStrategy {
    /// Add-Wins: Nodes are never removed, only marked as deleted
    AddWins,
    /// Remove-Wins: Deleted nodes are completely removed
    RemoveWins,
}

/// Configuration for tree CRDTs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeConfig {
    /// Conflict resolution strategy
    pub strategy: TreeStrategy,
    /// Whether to preserve deleted nodes in metadata
    pub preserve_deleted: bool,
    /// Maximum depth of the tree
    pub max_depth: Option<usize>,
    /// Maximum number of children per node
    pub max_children: Option<usize>,
}

impl Default for TreeConfig {
    fn default() -> Self {
        Self {
            strategy: TreeStrategy::AddWins,
            preserve_deleted: true,
            max_depth: None,
            max_children: None,
        }
    }
}

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
        let mut to_visit = vec![id.clone()];
        
        while let Some(current_id) = to_visit.pop() {
            if let Some(node) = self.nodes.get(&current_id) {
                if !node.metadata.deleted {
                    descendants.push(node);
                    to_visit.extend(node.children.iter().cloned());
                }
            }
        }
        
        descendants
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

/// Remove-Wins Tree CRDT implementation
/// 
/// This implementation completely removes deleted nodes.
/// It's more memory-efficient but nodes cannot be recovered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoveWinsTree<T> {
    /// Configuration
    config: TreeConfig,
    /// Nodes in the tree
    nodes: HashMap<NodeId, TreeNode<T>>,
    /// Replica ID for this instance
    replica: ReplicaId,
}

impl<T: Clone + PartialEq + Eq + Send + Sync> RemoveWinsTree<T> {
    /// Create a new Remove-Wins tree
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            config: TreeConfig {
                strategy: TreeStrategy::RemoveWins,
                preserve_deleted: false,
                max_depth: None,
                max_children: None,
            },
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

    /// Remove a node completely
    pub fn remove(&mut self, id: &NodeId) -> Result<(), TreeError> {
        // Get the parent ID first
        let parent_id = self.nodes.get(id).and_then(|node| node.parent.clone());
        
        // Remove from parent's children list
        if let Some(parent_id) = parent_id {
            if let Some(parent) = self.nodes.get_mut(&parent_id) {
                parent.remove_child(id);
            }
        }
        
        // Remove the node
        if self.nodes.remove(id).is_some() {
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

    /// Get all nodes
    pub fn nodes(&self) -> Vec<&TreeNode<T>> {
        self.nodes.values().collect()
    }

    /// Get root nodes
    pub fn roots(&self) -> Vec<&TreeNode<T>> {
        self.nodes
            .values()
            .filter(|n| n.parent.is_none())
            .collect()
    }

    /// Get children of a node
    pub fn children(&self, id: &NodeId) -> Vec<&TreeNode<T>> {
        if let Some(node) = self.nodes.get(id) {
            node.children
                .iter()
                .filter_map(|child_id| self.nodes.get(child_id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get descendants of a node (recursive)
    pub fn descendants(&self, id: &NodeId) -> Vec<&TreeNode<T>> {
        let mut descendants = Vec::new();
        let mut to_visit = vec![id.clone()];
        
        while let Some(current_id) = to_visit.pop() {
            if let Some(node) = self.nodes.get(&current_id) {
                descendants.push(node);
                to_visit.extend(node.children.iter().cloned());
            }
        }
        
        descendants
    }

    /// Check if the tree contains a node
    pub fn contains(&self, id: &NodeId) -> bool {
        self.nodes.contains_key(id)
    }

    /// Get the number of nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Clear all nodes
    pub fn clear(&mut self) {
        self.nodes.clear();
    }
}

impl<T: Clone + PartialEq + Eq + Send + Sync> CRDT for RemoveWinsTree<T> {
    fn replica_id(&self) -> &ReplicaId {
        &self.replica
    }
}

impl<T: Clone + PartialEq + Eq + Send + Sync> Mergeable for RemoveWinsTree<T> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ReplicaId;
    use uuid::Uuid;

    fn create_replica(id: u64) -> ReplicaId {
        ReplicaId::from(Uuid::from_u64_pair(0, id))
    }

    #[test]
    fn test_node_id_creation() {
        let replica = create_replica(1);
        let node_id = NodeId::new(replica);
        
        assert_eq!(node_id.replica, replica);
        assert_ne!(node_id.id, Uuid::nil());
    }

    #[test]
    fn test_tree_node_creation() {
        let replica = create_replica(1);
        let timestamp = 1234567890;
        let node = TreeNode::new("test_value", replica, timestamp);
        
        assert_eq!(node.value, "test_value");
        assert_eq!(node.metadata.created_at, timestamp);
        assert_eq!(node.metadata.modified_at, timestamp);
        assert_eq!(node.metadata.deleted, false);
        assert_eq!(node.metadata.last_modified_by, replica);
        assert!(node.parent.is_none());
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_add_wins_tree_basic_operations() {
        let replica = create_replica(1);
        let mut tree = AddWinsTree::new(replica);
        
        // Add root
        let root_id = tree.add_root("root", 1000);
        assert_eq!(tree.len(), 1);
        assert!(tree.contains(&root_id));
        
        // Add child
        let child_id = tree.add_child(&root_id, "child", 2000).unwrap();
        assert_eq!(tree.len(), 2);
        assert!(tree.contains(&child_id));
        
        // Check hierarchy
        let root = tree.get(&root_id).unwrap();
        assert!(root.children.contains(&child_id));
        
        let child = tree.get(&child_id).unwrap();
        assert_eq!(child.parent, Some(root_id));
    }

    #[test]
    fn test_remove_wins_tree_basic_operations() {
        let replica = create_replica(1);
        let mut tree = RemoveWinsTree::new(replica);
        
        // Add root and child
        let root_id = tree.add_root("root", 1000);
        let child_id = tree.add_child(&root_id, "child", 2000).unwrap();
        
        assert_eq!(tree.len(), 2);
        
        // Remove child completely
        tree.remove(&child_id).unwrap();
        assert_eq!(tree.len(), 1);
        assert!(!tree.contains(&child_id));
        
        // Root should have no children
        let root = tree.get(&root_id).unwrap();
        assert!(root.children.is_empty());
    }

    #[test]
    fn test_tree_move_operation() {
        let replica = create_replica(1);
        let mut tree = AddWinsTree::new(replica);
        
        // Create tree: root -> child1 -> child2
        let root_id = tree.add_root("root", 1000);
        let child1_id = tree.add_child(&root_id, "child1", 2000).unwrap();
        let child2_id = tree.add_child(&child1_id, "child2", 3000).unwrap();
        
        // Move child2 to root
        tree.move_node(&child2_id, &root_id).unwrap();
        
        let child2 = tree.get(&child2_id).unwrap();
        assert_eq!(child2.parent, Some(root_id.clone()));
        
        let root = tree.get(&root_id).unwrap();
        assert!(root.children.contains(&child2_id));
        
        let child1 = tree.get(&child1_id).unwrap();
        assert!(!child1.children.contains(&child2_id));
    }

    #[test]
    fn test_tree_traversal() {
        let replica = create_replica(1);
        let mut tree = AddWinsTree::new(replica);
        
        // Create tree: root -> child1 -> child2
        let root_id = tree.add_root("root", 1000);
        let child1_id = tree.add_child(&root_id, "child1", 2000).unwrap();
        let child2_id = tree.add_child(&child1_id, "child2", 3000).unwrap();
        
        // Test descendants
        let descendants = tree.descendants(&root_id);
        assert_eq!(descendants.len(), 2);
        
        // Test children
        let root_children = tree.children(&root_id);
        assert_eq!(root_children.len(), 1);
        assert_eq!(root_children[0].id, child1_id);
    }

    #[test]
    fn test_tree_merge() {
        let replica1 = create_replica(1);
        let replica2 = create_replica(2);
        
        let mut tree1 = AddWinsTree::new(replica1);
        let mut tree2 = AddWinsTree::new(replica2);
        
        // Add nodes to both trees
        let root1_id = tree1.add_root("root1", 1000);
        let root2_id = tree2.add_root("root2", 2000);
        
        // Merge tree2 into tree1
        tree1.merge(&tree2).unwrap();
        
        // Both roots should be present
        assert_eq!(tree1.len(), 2);
        assert!(tree1.contains(&root1_id));
        assert!(tree1.contains(&root2_id));
    }

    #[test]
    fn test_tree_configuration() {
        let replica = create_replica(1);
        let config = TreeConfig {
            strategy: TreeStrategy::RemoveWins,
            preserve_deleted: false,
            max_depth: Some(5),
            max_children: Some(10),
        };
        
        let tree: AddWinsTree<String> = AddWinsTree::with_config(replica, config);
        assert_eq!(tree.config.strategy, TreeStrategy::RemoveWins);
        assert_eq!(tree.config.max_depth, Some(5));
        assert_eq!(tree.config.max_children, Some(10));
    }
}
