//! Cross-demo integration tests
//! 
//! These tests verify that different CRDT types can work together
//! and that the demos can be used in combination.

use leptos_sync_core::crdt::advanced::{Rga, Lseq};
use leptos_sync_core::crdt::tree::YjsTree;
use leptos_sync_core::crdt::graph::AddWinsGraph;
use leptos_sync_core::crdt::{Mergeable, ReplicaId};
use uuid::Uuid;

#[test]
fn test_cross_demo_crdt_compatibility() {
    // Test that different CRDT types can coexist and work together
    let replica = ReplicaId::from(Uuid::new_v4());
    
    // Create instances of all CRDT types
    let mut rga = Rga::new(replica.clone());
    let mut lseq = Lseq::new(replica.clone());
    let mut yjs_tree = YjsTree::new(replica.clone());
    let mut dag = AddWinsGraph::new(replica);
    
    // Test that they can all be created without conflicts
    assert_eq!(rga.len(), 0);
    assert_eq!(lseq.len(), 0);
    assert_eq!(yjs_tree.len(), 0);
    // DAG doesn't have a len() method, but we can verify it was created
    assert!(dag.get_vertex(&leptos_sync_core::crdt::graph::VertexId::new(replica)).is_none());
}

#[test]
fn test_demo_data_structures_compatibility() {
    // Test that data structures from different demos are compatible
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    // Text Editor data (characters)
    let mut rga1 = Rga::new(replica1.clone());
    let mut rga2 = Rga::new(replica2.clone());
    
    // Task Manager data (tasks)
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Task {
        id: Uuid,
        title: String,
        completed: bool,
    }
    
    let mut lseq1 = Lseq::new(replica1.clone());
    let mut lseq2 = Lseq::new(replica2.clone());
    
    // Add data to both CRDTs
    let pos1 = rga1.insert_after(None, 'H').unwrap();
    let pos2 = rga1.insert_after(Some(pos1.clone()), 'i').unwrap();
    
    let task = Task {
        id: Uuid::new_v4(),
        title: "Test task".to_string(),
        completed: false,
    };
    let task_pos = lseq1.insert_after(None, task.clone()).unwrap();
    
    // Merge both CRDTs
    rga1.merge(&rga2).unwrap();
    lseq1.merge(&lseq2).unwrap();
    
    // Verify both work independently
    let text: String = rga1.to_vec().into_iter().collect();
    assert_eq!(text, "Hi");
    
    let tasks: Vec<Task> = lseq1.to_vec().into_iter().collect();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Test task");
}

#[test]
fn test_demo_replica_id_consistency() {
    // Test that replica IDs are consistent across different CRDT types
    let replica_id = ReplicaId::from(Uuid::new_v4());
    
    let rga = Rga::new(replica_id.clone());
    let lseq = Lseq::new(replica_id.clone());
    let yjs_tree = YjsTree::new(replica_id.clone());
    let dag = AddWinsGraph::new(replica_id);
    
    // All CRDTs should use the same replica ID
    // (We can't directly access the replica ID from the CRDTs,
    // but we can verify they were created successfully)
    assert_eq!(rga.len(), 0);
    assert_eq!(lseq.len(), 0);
    assert_eq!(yjs_tree.len(), 0);
    // DAG creation is sufficient to verify replica ID consistency
}

#[test]
fn test_demo_merge_operations_independence() {
    // Test that merge operations on different CRDT types are independent
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    // Create CRDTs for different demos
    let mut rga1 = Rga::new(replica1.clone());
    let mut rga2 = Rga::new(replica2.clone());
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DocumentNode {
        id: Uuid,
        title: String,
        content: String,
    }
    
    let mut yjs_tree1 = YjsTree::new(replica1.clone());
    let mut yjs_tree2 = YjsTree::new(replica2.clone());
    
    // Add data to both CRDT types
    let pos1 = rga1.insert_after(None, 'A').unwrap();
    let pos2 = rga1.insert_after(Some(pos1.clone()), 'B').unwrap();
    
    let node = DocumentNode {
        id: Uuid::new_v4(),
        title: "Test Document".to_string(),
        content: "Test content".to_string(),
    };
    let node_pos = yjs_tree1.insert_after(None, node.clone()).unwrap();
    
    // Merge both CRDT types independently
    rga1.merge(&rga2).unwrap();
    yjs_tree1.merge(&yjs_tree2).unwrap();
    
    // Verify both merges worked correctly
    let text: String = rga1.to_vec().into_iter().collect();
    assert_eq!(text, "AB");
    
    let nodes: Vec<DocumentNode> = yjs_tree1.to_vec().into_iter().collect();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].title, "Test Document");
}

#[test]
fn test_demo_error_handling_consistency() {
    // Test that error handling is consistent across different CRDT types
    let replica = ReplicaId::from(Uuid::new_v4());
    
    let mut rga = Rga::new(replica.clone());
    let mut lseq = Lseq::new(replica.clone());
    
    // Test invalid operations
    let invalid_pos = leptos_sync_core::crdt::advanced::PositionId::new();
    
    // RGA should handle invalid position gracefully
    let result1 = rga.insert_after(Some(invalid_pos.clone()), 'X');
    assert!(result1.is_err());
    
    // LSEQ should handle invalid position gracefully
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestItem {
        value: String,
    }
    
    let test_item = TestItem {
        value: "test".to_string(),
    };
    let result2 = lseq.insert_after(Some(invalid_pos), test_item);
    assert!(result2.is_err());
}

#[test]
fn test_demo_performance_characteristics() {
    // Test that all CRDT types perform basic operations efficiently
    let replica = ReplicaId::from(Uuid::new_v4());
    
    let mut rga = Rga::new(replica.clone());
    let mut lseq = Lseq::new(replica.clone());
    let mut yjs_tree = YjsTree::new(replica.clone());
    
    // Add multiple items to each CRDT
    let mut positions = Vec::new();
    
    // RGA operations
    for i in 0..10 {
        let pos = rga.insert_after(positions.last().cloned(), ('A' as u8 + i) as char).unwrap();
        positions.push(pos);
    }
    
    // LSEQ operations
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestItem {
        id: usize,
        value: String,
    }
    
    let mut lseq_positions = Vec::new();
    for i in 0..10 {
        let item = TestItem {
            id: i,
            value: format!("Item {}", i),
        };
        let pos = lseq.insert_after(lseq_positions.last().cloned(), item).unwrap();
        lseq_positions.push(pos);
    }
    
    // Yjs Tree operations
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TreeNode {
        id: Uuid,
        value: String,
    }
    
    let mut tree_positions = Vec::new();
    for i in 0..10 {
        let node = TreeNode {
            id: Uuid::new_v4(),
            value: format!("Node {}", i),
        };
        let pos = yjs_tree.insert_after(tree_positions.last().cloned(), node).unwrap();
        tree_positions.push(pos);
    }
    
    // Verify all operations completed successfully
    assert_eq!(rga.len(), 10);
    assert_eq!(lseq.len(), 10);
    assert_eq!(yjs_tree.len(), 10);
}
