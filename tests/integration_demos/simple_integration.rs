//! Simple integration tests for collaborative application demos
//! 
//! These tests verify that the CRDT implementations work correctly
//! with the demo applications.

use leptos_sync_core::crdt::advanced::{Rga, Lseq, YjsTree};
use leptos_sync_core::crdt::graph::AddWinsGraph;
use leptos_sync_core::crdt::{Mergeable, ReplicaId};
use uuid::Uuid;

#[test]
fn test_rga_basic_functionality() {
    // Test basic RGA functionality for text editing
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut rga1 = Rga::new(replica1);
    let mut rga2 = Rga::new(replica2);
    
    // Insert characters from replica1
    let pos1 = rga1.insert_after('H', None).unwrap();
    let pos2 = rga1.insert_after('e', Some(pos1.clone())).unwrap();
    let pos3 = rga1.insert_after('l', Some(pos2.clone())).unwrap();
    let pos4 = rga1.insert_after('l', Some(pos3.clone())).unwrap();
    let pos5 = rga1.insert_after('o', Some(pos4.clone())).unwrap();
    
    // Insert characters from replica2
    let pos6 = rga2.insert_after('W', None).unwrap();
    let pos7 = rga2.insert_after('o', Some(pos6.clone())).unwrap();
    let pos8 = rga2.insert_after('r', Some(pos7.clone())).unwrap();
    let pos9 = rga2.insert_after('l', Some(pos8.clone())).unwrap();
    let pos10 = rga2.insert_after('d', Some(pos9.clone())).unwrap();
    
    // Merge the two RGAs
    rga1.merge(&rga2).unwrap();
    rga2.merge(&rga1).unwrap();
    
    // Both should have the same content
    let content1: Vec<char> = rga1.to_vec();
    let content2: Vec<char> = rga2.to_vec();
    
    assert_eq!(content1, content2);
    assert_eq!(content1.len(), 10); // "Hello" + "World"
}

#[test]
fn test_lseq_basic_functionality() {
    // Test basic LSEQ functionality for task management
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut lseq1 = Lseq::new(replica1);
    let mut lseq2 = Lseq::new(replica2);
    
    // Add tasks from replica1
    let task1 = "Buy groceries".to_string();
    let task2 = "Walk the dog".to_string();
    
    let pos1 = lseq1.insert(task1.clone(), None).unwrap();
    let pos2 = lseq1.insert(task2.clone(), Some(pos1.clone())).unwrap();
    
    // Add tasks from replica2
    let task3 = "Finish project".to_string();
    let task4 = "Call mom".to_string();
    
    let pos3 = lseq2.insert(task3.clone(), None).unwrap();
    let pos4 = lseq2.insert(task4.clone(), Some(pos3.clone())).unwrap();
    
    // Merge both ways
    lseq1.merge(&lseq2).unwrap();
    lseq2.merge(&lseq1).unwrap();
    
    // Both should have the same content
    let tasks1: Vec<String> = lseq1.to_vec();
    let tasks2: Vec<String> = lseq2.to_vec();
    
    assert_eq!(tasks1.len(), 4);
    assert_eq!(tasks2.len(), 4);
    
    // Verify all tasks are present
    assert!(tasks1.contains(&"Buy groceries".to_string()));
    assert!(tasks1.contains(&"Walk the dog".to_string()));
    assert!(tasks1.contains(&"Finish project".to_string()));
    assert!(tasks1.contains(&"Call mom".to_string()));
    
    assert_eq!(tasks1, tasks2);
}

#[test]
fn test_yjs_tree_basic_functionality() {
    // Test basic Yjs Tree functionality for document editing
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut tree1 = YjsTree::new(replica1);
    let mut tree2 = YjsTree::new(replica2);
    
    // Create document structure from replica1
    let section1 = "Introduction".to_string();
    let paragraph1 = "This is the introduction paragraph.".to_string();
    
    let pos1 = tree1.insert_after(section1.clone(), None).unwrap();
    let pos2 = tree1.insert_after(paragraph1.clone(), Some(pos1.clone())).unwrap();
    
    // Create document structure from replica2
    let section2 = "Conclusion".to_string();
    let paragraph2 = "This is the conclusion paragraph.".to_string();
    
    let pos3 = tree2.insert_after(section2.clone(), None).unwrap();
    let pos4 = tree2.insert_after(paragraph2.clone(), Some(pos3.clone())).unwrap();
    
    // Merge both ways
    tree1.merge(&tree2).unwrap();
    tree2.merge(&tree1).unwrap();
    
    // Both should have the same content
    let nodes1: Vec<String> = tree1.to_vec();
    let nodes2: Vec<String> = tree2.to_vec();
    
    assert_eq!(nodes1.len(), 4);
    assert_eq!(nodes2.len(), 4);
    assert_eq!(nodes1, nodes2);
}

#[test]
fn test_dag_basic_functionality() {
    // Test basic DAG functionality for project management
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut graph1 = AddWinsGraph::new(replica1);
    let mut graph2 = AddWinsGraph::new(replica2);
    
    // Add tasks from replica1
    let task1 = "Setup project".to_string();
    let task2 = "Write tests".to_string();
    
    let timestamp1 = 1000;
    let timestamp2 = 2000;
    
    let vertex1 = graph1.add_vertex(task1.clone(), timestamp1);
    let vertex2 = graph1.add_vertex(task2.clone(), timestamp2);
    
    // Add tasks from replica2
    let task3 = "Deploy to production".to_string();
    let task4 = "Monitor performance".to_string();
    
    let timestamp3 = 1500;
    let timestamp4 = 2500;
    
    let vertex3 = graph2.add_vertex(task3.clone(), timestamp3);
    let vertex4 = graph2.add_vertex(task4.clone(), timestamp4);
    
    // Merge both ways
    graph1.merge(&graph2).unwrap();
    graph2.merge(&graph1).unwrap();
    
    // Both graphs should have the same vertices
    assert!(graph1.get_vertex(&vertex3).is_some());
    assert!(graph1.get_vertex(&vertex4).is_some());
    assert!(graph2.get_vertex(&vertex1).is_some());
    assert!(graph2.get_vertex(&vertex2).is_some());
}

#[test]
fn test_crdt_merge_consistency() {
    // Test that all CRDT types maintain consistency after merging
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    // Test RGA merge consistency
    let mut rga1 = Rga::new(replica1.clone());
    let mut rga2 = Rga::new(replica2.clone());
    
    rga1.insert_after('A', None).unwrap();
    rga2.insert_after('B', None).unwrap();
    
    rga1.merge(&rga2).unwrap();
    rga2.merge(&rga1).unwrap();
    
    let content1: Vec<char> = rga1.to_vec();
    let content2: Vec<char> = rga2.to_vec();
    assert_eq!(content1, content2);
    
    // Test LSEQ merge consistency
    let mut lseq1 = Lseq::new(replica1.clone());
    let mut lseq2 = Lseq::new(replica2.clone());
    
    lseq1.insert("Item 1".to_string(), None).unwrap();
    lseq2.insert("Item 2".to_string(), None).unwrap();
    
    lseq1.merge(&lseq2).unwrap();
    lseq2.merge(&lseq1).unwrap();
    
    let items1: Vec<String> = lseq1.to_vec();
    let items2: Vec<String> = lseq2.to_vec();
    assert_eq!(items1, items2);
    
    // Test Yjs Tree merge consistency
    let mut tree1 = YjsTree::new(replica1.clone());
    let mut tree2 = YjsTree::new(replica2.clone());
    
    tree1.insert_after("Node 1".to_string(), None).unwrap();
    tree2.insert_after("Node 2".to_string(), None).unwrap();
    
    tree1.merge(&tree2).unwrap();
    tree2.merge(&tree1).unwrap();
    
    let nodes1: Vec<String> = tree1.to_vec();
    let nodes2: Vec<String> = tree2.to_vec();
    assert_eq!(nodes1, nodes2);
    
    // Test DAG merge consistency
    let mut graph1 = AddWinsGraph::new(replica1);
    let mut graph2 = AddWinsGraph::new(replica2);
    
    let vertex1 = graph1.add_vertex("Task 1".to_string(), 1000);
    let vertex2 = graph2.add_vertex("Task 2".to_string(), 2000);
    
    graph1.merge(&graph2).unwrap();
    graph2.merge(&graph1).unwrap();
    
    // Both graphs should have both vertices
    assert!(graph1.get_vertex(&vertex1).is_some());
    assert!(graph1.get_vertex(&vertex2).is_some());
    assert!(graph2.get_vertex(&vertex1).is_some());
    assert!(graph2.get_vertex(&vertex2).is_some());
}
