//! Conflict-free Replicated Data Types (CRDTs) for distributed systems
//! 
//! This module provides various CRDT implementations that can be used
//! to build eventually consistent distributed applications.

mod crdt_basic;
pub mod list;
pub mod tree;
pub mod graph;
pub mod builder;
pub mod advanced;

// Re-export basic CRDTs
pub use crdt_basic::{LwwRegister, LwwMap, GCounter, ReplicaId, Mergeable, CRDT};

pub use list::{
    ElementId, ElementMetadata, ListElement, ListStrategy, ListConfig,
    AddWinsList, RemoveWinsList, LwwList,
};

pub use tree::{
    NodeId, NodeMetadata, TreeNode, TreeStrategy, TreeConfig,
    AddWinsTree, RemoveWinsTree,
};

pub use graph::{
    VertexId, EdgeId, VertexMetadata, EdgeMetadata, Vertex, Edge,
    GraphStrategy, GraphConfig, AddWinsGraph, RemoveWinsGraph,
};

// Re-export builder functionality
pub use builder::{
    CrdtBuilder, CrdtBuilderConfig, FieldConfig, CrdtStrategy, 
    CustomCrdt, GenericCrdtField, CrdtField, BuilderError
};

// Re-export advanced CRDT types
pub use advanced::{
    Rga, RgaElement, Lseq, LseqElement, YjsTree, YjsNode, YjsTreeNode,
    Dag, DagNode, PositionId, AdvancedCrdtError
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::ReplicaId;
    use uuid::Uuid;

    fn create_replica(id: u64) -> ReplicaId {
        ReplicaId::from(Uuid::from_u64_pair(0, id))
    }

    #[test]
    fn test_list_crdt_integration() {
        let replica = create_replica(1);
        let mut list = AddWinsList::new(replica);
        
        let id1 = list.add("item1", 1000);
        let id2 = list.add("item2", 2000);
        
        assert_eq!(list.len(), 2);
        assert!(list.contains(&id1));
        assert!(list.contains(&id2));
    }

    #[test]
    fn test_tree_crdt_integration() {
        let replica = create_replica(1);
        let mut tree = AddWinsTree::new(replica);
        
        let root_id = tree.add_root("root", 1000);
        let child_id = tree.add_child(&root_id, "child", 2000).unwrap();
        
        assert_eq!(tree.len(), 2);
        assert!(tree.contains(&root_id));
        assert!(tree.contains(&child_id));
    }

    #[test]
    fn test_graph_crdt_integration() {
        let replica = create_replica(1);
        let mut graph = AddWinsGraph::new(replica);
        
        let v1_id = graph.add_vertex("vertex1", 1000);
        let v2_id = graph.add_vertex("vertex2", 2000);
        let edge_id = graph.add_edge(&v1_id, &v2_id, 3000, None).unwrap();
        
        assert_eq!(graph.vertex_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert!(graph.contains_vertex(&v1_id));
        assert!(graph.contains_edge(&edge_id));
    }

    #[test]
    fn test_crdt_traits() {
        let replica = create_replica(1);
        
        // Test that all CRDT types implement the required traits
        let list: AddWinsList<String> = AddWinsList::new(replica);
        let tree: AddWinsTree<String> = AddWinsTree::new(replica);
        let graph: AddWinsGraph<String> = AddWinsGraph::new(replica);
        
        // This should compile if all types implement CRDT trait
        let _: &dyn CRDT = &list;
        let _: &dyn CRDT = &tree;
        let _: &dyn CRDT = &graph;
    }

    #[test]
    fn test_custom_crdt_builder_integration() {
        let replica = create_replica(1);
        
        // Create a custom CRDT using the builder
        let config = CrdtBuilder::new("UserProfile".to_string())
            .add_field("name".to_string(), CrdtStrategy::Lww)
            .add_field("age".to_string(), CrdtStrategy::Lww)
            .add_field("friends".to_string(), CrdtStrategy::AddWins)
            .add_optional_field("bio".to_string(), CrdtStrategy::Lww, 
                serde_json::Value::String("No bio yet".to_string()))
            .build();
        
        let mut profile = CustomCrdt::new(config, replica);
        
        // Set field values
        profile.set_field("name", serde_json::Value::String("Alice".to_string())).unwrap();
        profile.set_field("age", serde_json::Value::Number(serde_json::Number::from(25))).unwrap();
        profile.set_field("friends", serde_json::Value::Array(vec![
            serde_json::Value::String("Bob".to_string()),
            serde_json::Value::String("Charlie".to_string()),
        ])).unwrap();
        
        // Test field access
        assert_eq!(profile.get_field("name"), Some(&serde_json::Value::String("Alice".to_string())));
        assert_eq!(profile.get_field("age"), Some(&serde_json::Value::Number(serde_json::Number::from(25))));
        assert_eq!(profile.get_field("bio"), Some(&serde_json::Value::String("No bio yet".to_string())));
        
        // Test CRDT trait implementation
        let _: &dyn CRDT = &profile;
        
        // Test mergeable trait
        let mut profile2 = profile.clone();
        
        // Small delay to ensure different timestamp for LWW
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        profile2.set_field("name", serde_json::Value::String("Alice Updated".to_string())).unwrap();
        profile2.set_field("friends", serde_json::Value::Array(vec![
            serde_json::Value::String("David".to_string()),
        ])).unwrap();
        
        // Merge profiles
        profile.merge(&profile2).unwrap();
        
        // Check merged values
        assert_eq!(profile.get_field("name"), Some(&serde_json::Value::String("Alice Updated".to_string())));
        // Friends should be combined (AddWins strategy)
        if let Some(friends) = profile.get_field("friends") {
            if let Some(friends_array) = friends.as_array() {
                assert_eq!(friends_array.len(), 3); // Bob, Charlie, David
            }
        }
    }

    #[test]
    fn test_advanced_crdt_integration() {
        let replica1 = create_replica(1);
        let replica2 = create_replica(2);
        
        // Test RGA integration
        let mut rga1 = Rga::new(replica1.clone());
        let mut rga2 = Rga::new(replica2.clone());
        
        let _pos1 = rga1.insert_after("hello".to_string(), None).unwrap();
        let _pos2 = rga2.insert_after("world".to_string(), None).unwrap();
        
        rga1.merge(&rga2).unwrap();
        let elements = rga1.to_vec();
        assert!(elements.contains(&"hello".to_string()));
        assert!(elements.contains(&"world".to_string()));
        
        // Test LSEQ integration
        let mut lseq1 = Lseq::new(replica1.clone());
        let mut lseq2 = Lseq::new(replica2.clone());
        
        lseq1.insert("item1".to_string(), None).unwrap();
        lseq2.insert("item2".to_string(), None).unwrap();
        
        lseq1.merge(&lseq2).unwrap();
        let elements = lseq1.to_vec();
        assert!(elements.contains(&"item1".to_string()));
        assert!(elements.contains(&"item2".to_string()));
        
        // Test Yjs Tree integration
        let mut tree1 = YjsTree::new(replica1.clone());
        let mut tree2 = YjsTree::new(replica2.clone());
        
        let root1_id = tree1.add_root("root1".to_string()).unwrap();
        let root2_id = tree2.add_root("root2".to_string()).unwrap();
        
        tree1.add_child(&root1_id, "child1".to_string()).unwrap();
        tree2.add_child(&root2_id, "child2".to_string()).unwrap();
        
        tree1.merge(&tree2).unwrap();
        assert_eq!(tree1.len(), 4); // 2 roots + 2 children
        
        // Test DAG integration
        let mut dag1 = Dag::new(replica1);
        let mut dag2 = Dag::new(replica2);
        
        let node1_id = dag1.add_node("node1".to_string()).unwrap();
        let node2_id = dag2.add_node("node2".to_string()).unwrap();
        
        // No edges needed for merge test
        
        dag1.merge(&dag2).unwrap();
        assert_eq!(dag1.len(), 2);
    }
}
