//! Advanced CRDT Types
//!
//! This module provides advanced CRDT implementations including:
//! - RGA (Replicated Growable Array) for collaborative text editing
//! - LSEQ (Logoot Sequence) for ordered sequences
//! - Yjs-style trees for hierarchical data
//! - DAG (Directed Acyclic Graph) for complex relationships

pub mod common;
pub mod rga;
pub mod lseq;
pub mod yjs_tree;
pub mod dag;

// Re-export main types for convenience
pub use common::{PositionId, AdvancedCrdtError};
pub use rga::{Rga, RgaElement};
pub use lseq::{Lseq, LseqElement};
pub use yjs_tree::{YjsTree, YjsNode, YjsTreeNode};
pub use dag::{Dag, DagNode};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use super::super::{ReplicaId, basic::traits::Mergeable};
    use uuid::Uuid;
    
    fn create_replica(id: u64) -> ReplicaId {
        ReplicaId::from(Uuid::from_u64_pair(0, id))
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
        let _: &dyn super::super::CRDT = &rga;
        let _: &dyn super::super::CRDT = &lseq;
        let _: &dyn super::super::CRDT = &tree;
        let _: &dyn super::super::CRDT = &dag;
    }

    #[test]
    fn test_advanced_crdt_merge_integration() {
        let replica_id1 = create_replica(1);
        let replica_id2 = create_replica(2);
        
        // Test RGA merge
        let mut rga1: Rga<String> = Rga::new(replica_id1.clone());
        let mut rga2: Rga<String> = Rga::new(replica_id2.clone());
        
        let _pos1 = rga1.insert_after("hello".to_string(), None).unwrap();
        let _pos2 = rga2.insert_after("world".to_string(), None).unwrap();
        
        rga1.merge(&rga2).unwrap();
        let elements = rga1.to_vec();
        assert!(elements.contains(&"hello".to_string()));
        assert!(elements.contains(&"world".to_string()));
        
        // Test LSEQ merge
        let mut lseq1 = Lseq::new(replica_id1.clone());
        let mut lseq2 = Lseq::new(replica_id2.clone());
        
        lseq1.insert("item1".to_string(), None).unwrap();
        lseq2.insert("item2".to_string(), None).unwrap();
        
        lseq1.merge(&lseq2).unwrap();
        let elements = lseq1.to_vec();
        assert!(elements.contains(&"item1".to_string()));
        assert!(elements.contains(&"item2".to_string()));
        
        // Test Yjs Tree merge
        let mut tree1 = YjsTree::new(replica_id1.clone());
        let mut tree2 = YjsTree::new(replica_id2.clone());
        
        let root1_id = tree1.add_root("root1".to_string()).unwrap();
        let root2_id = tree2.add_root("root2".to_string()).unwrap();
        
        tree1.add_child(&root1_id, "child1".to_string()).unwrap();
        tree2.add_child(&root2_id, "child2".to_string()).unwrap();
        
        tree1.merge(&tree2).unwrap();
        assert_eq!(tree1.len(), 4); // 2 roots + 2 children
        
        // Test DAG merge
        let mut dag1 = Dag::new(replica_id1);
        let mut dag2 = Dag::new(replica_id2);
        
        let node1_id = dag1.add_node("node1".to_string()).unwrap();
        let node2_id = dag2.add_node("node2".to_string()).unwrap();
        
        // No edges needed for merge test
        
        dag1.merge(&dag2).unwrap();
        assert_eq!(dag1.len(), 2);
    }

    #[test]
    fn test_position_id_ordering() {
        let replica1 = create_replica(1);
        let replica2 = create_replica(2);
        
        let pos1 = PositionId::new(replica1, 100, 10);
        let pos2 = PositionId::new(replica1, 200, 10);
        let pos3 = PositionId::new(replica2, 100, 10);
        
        assert!(pos1 < pos2);
        assert!(pos1 < pos3);
        assert!(pos2 > pos3);
    }

    #[test]
    fn test_error_handling() {
        let replica = create_replica(1);
        let mut rga: Rga<String> = Rga::new(replica);
        
        // Test invalid position error
        let fake_pos = PositionId::new(create_replica(999), 1, 1);
        let result = rga.delete(&fake_pos);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdvancedCrdtError::ElementNotFound(_)));
    }
}
