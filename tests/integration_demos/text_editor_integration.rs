//! Integration tests for Text Editor Demo using RGA CRDT

use leptos_sync_core::crdt::advanced::Rga;
use leptos_sync_core::crdt::{Mergeable, ReplicaId};
use uuid::Uuid;

#[test]
fn test_text_editor_rga_integration() {
    // Test basic RGA functionality
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut rga1 = Rga::new(replica1);
    let mut rga2 = Rga::new(replica2);
    
    // Insert characters from replica1
    let pos1 = rga1.insert_after(None, 'H').unwrap();
    let pos2 = rga1.insert_after(Some(pos1.clone()), 'e').unwrap();
    let pos3 = rga1.insert_after(Some(pos2.clone()), 'l').unwrap();
    let pos4 = rga1.insert_after(Some(pos3.clone()), 'l').unwrap();
    let pos5 = rga1.insert_after(Some(pos4.clone()), 'o').unwrap();
    
    // Insert characters from replica2
    let pos6 = rga2.insert_after(None, 'W').unwrap();
    let pos7 = rga2.insert_after(Some(pos6.clone()), 'o').unwrap();
    let pos8 = rga2.insert_after(Some(pos7.clone()), 'r').unwrap();
    let pos9 = rga2.insert_after(Some(pos8.clone()), 'l').unwrap();
    let pos10 = rga2.insert_after(Some(pos9.clone()), 'd').unwrap();
    
    // Merge the two RGAs
    rga1.merge(&rga2).unwrap();
    rga2.merge(&rga1).unwrap();
    
    // Both should have the same content
    let content1: String = rga1.to_vec().into_iter().collect();
    let content2: String = rga2.to_vec().into_iter().collect();
    
    assert_eq!(content1, content2);
    assert_eq!(content1.len(), 10); // "Hello" + "World"
}

#[test]
fn test_text_editor_concurrent_edits() {
    // Test concurrent editing scenario
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut rga1 = Rga::new(replica1);
    let mut rga2 = Rga::new(replica2);
    
    // Both start with "Hello"
    let pos1 = rga1.insert_after(None, 'H').unwrap();
    let pos2 = rga1.insert_after(Some(pos1.clone()), 'e').unwrap();
    let pos3 = rga1.insert_after(Some(pos2.clone()), 'l').unwrap();
    let pos4 = rga1.insert_after(Some(pos3.clone()), 'l').unwrap();
    let pos5 = rga1.insert_after(Some(pos4.clone()), 'o').unwrap();
    
    // Replica2 gets a copy
    rga2.merge(&rga1).unwrap();
    
    // Concurrent edits: replica1 adds " World", replica2 adds "!"
    let pos6 = rga1.insert_after(Some(pos5.clone()), ' ').unwrap();
    let pos7 = rga1.insert_after(Some(pos6.clone()), 'W').unwrap();
    let pos8 = rga1.insert_after(Some(pos7.clone()), 'o').unwrap();
    let pos9 = rga1.insert_after(Some(pos8.clone()), 'r').unwrap();
    let pos10 = rga1.insert_after(Some(pos9.clone()), 'l').unwrap();
    let pos11 = rga1.insert_after(Some(pos10.clone()), 'd').unwrap();
    
    let pos12 = rga2.insert_after(Some(pos5.clone()), '!').unwrap();
    
    // Merge both ways
    rga1.merge(&rga2).unwrap();
    rga2.merge(&rga1).unwrap();
    
    // Both should have the same final content
    let content1: String = rga1.to_vec().into_iter().collect();
    let content2: String = rga2.to_vec().into_iter().collect();
    
    assert_eq!(content1, content2);
    assert_eq!(content1, "Hello World!");
}

#[test]
fn test_text_editor_deletion() {
    // Test character deletion
    let replica = ReplicaId::from(Uuid::new_v4());
    let mut rga = Rga::new(replica);
    
    // Insert "Hello"
    let pos1 = rga.insert_after(None, 'H').unwrap();
    let pos2 = rga.insert_after(Some(pos1.clone()), 'e').unwrap();
    let pos3 = rga.insert_after(Some(pos2.clone()), 'l').unwrap();
    let pos4 = rga.insert_after(Some(pos3.clone()), 'l').unwrap();
    let pos5 = rga.insert_after(Some(pos4.clone()), 'o').unwrap();
    
    // Delete the second 'l'
    rga.delete(pos4).unwrap();
    
    let content: String = rga.to_vec().into_iter().collect();
    assert_eq!(content, "Helo");
}

#[test]
fn test_text_editor_merge_conflicts() {
    // Test that RGA handles merge conflicts correctly
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut rga1 = Rga::new(replica1);
    let mut rga2 = Rga::new(replica2);
    
    // Both start with "Hi"
    let pos1 = rga1.insert_after(None, 'H').unwrap();
    let pos2 = rga1.insert_after(Some(pos1.clone()), 'i').unwrap();
    rga2.merge(&rga1).unwrap();
    
    // Both insert at the same position concurrently
    let pos3 = rga1.insert_after(Some(pos2.clone()), '!').unwrap();
    let pos4 = rga2.insert_after(Some(pos2.clone()), '?').unwrap();
    
    // Merge both ways
    rga1.merge(&rga2).unwrap();
    rga2.merge(&rga1).unwrap();
    
    // Both should have the same content (order may vary but both chars present)
    let content1: String = rga1.to_vec().into_iter().collect();
    let content2: String = rga2.to_vec().into_iter().collect();
    
    assert_eq!(content1, content2);
    assert!(content1.contains('!'));
    assert!(content1.contains('?'));
    assert_eq!(content1.len(), 4); // "Hi" + "!" + "?"
}
