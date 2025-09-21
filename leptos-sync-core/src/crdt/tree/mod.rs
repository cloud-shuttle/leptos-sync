//! Tree CRDT implementations
//!
//! This module provides tree-based Conflict-free Replicated Data Types (CRDTs)
//! with different conflict resolution strategies.

pub mod add_wins;
pub mod config;
pub mod error;
pub mod remove_wins;
pub mod types;

// Re-export public types
pub use add_wins::AddWinsTree;
pub use config::{TreeConfig, TreeStrategy};
pub use error::TreeError;
pub use remove_wins::RemoveWinsTree;
pub use types::{NodeId, NodeMetadata, TreeNode};

#[cfg(test)]
mod tests {
    use super::super::{Mergeable, ReplicaId};
    use super::*;
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
        let _child2_id = tree.add_child(&child1_id, "child2", 3000).unwrap();

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
        assert_eq!(tree.config().strategy, TreeStrategy::RemoveWins);
        assert_eq!(tree.config().max_depth, Some(5));
        assert_eq!(tree.config().max_children, Some(10));
    }
}
