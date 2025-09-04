//! Integration tests for Task Manager Demo using LSEQ CRDT

use leptos_sync_core::crdt::advanced::Lseq;
use leptos_sync_core::crdt::{Mergeable, ReplicaId};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Task {
    id: Uuid,
    title: String,
    completed: bool,
}

impl Task {
    fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            completed: false,
        }
    }
}

#[test]
fn test_task_manager_lseq_integration() {
    // Test basic LSEQ functionality for task management
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut lseq1 = Lseq::new(replica1);
    let mut lseq2 = Lseq::new(replica2);
    
    // Add tasks from replica1
    let task1 = Task::new("Buy groceries".to_string());
    let task2 = Task::new("Walk the dog".to_string());
    
    let pos1 = lseq1.insert_after(None, task1.clone()).unwrap();
    let pos2 = lseq1.insert_after(Some(pos1.clone()), task2.clone()).unwrap();
    
    // Add tasks from replica2
    let task3 = Task::new("Finish project".to_string());
    let task4 = Task::new("Call mom".to_string());
    
    let pos3 = lseq2.insert_after(None, task3.clone()).unwrap();
    let pos4 = lseq2.insert_after(Some(pos3.clone()), task4.clone()).unwrap();
    
    // Merge both ways
    lseq1.merge(&lseq2).unwrap();
    lseq2.merge(&lseq1).unwrap();
    
    // Both should have the same content
    let tasks1: Vec<Task> = lseq1.to_vec().into_iter().collect();
    let tasks2: Vec<Task> = lseq2.to_vec().into_iter().collect();
    
    assert_eq!(tasks1.len(), 4);
    assert_eq!(tasks2.len(), 4);
    
    // Verify all tasks are present
    let titles1: Vec<String> = tasks1.iter().map(|t| t.title.clone()).collect();
    let titles2: Vec<String> = tasks2.iter().map(|t| t.title.clone()).collect();
    
    assert!(titles1.contains(&"Buy groceries".to_string()));
    assert!(titles1.contains(&"Walk the dog".to_string()));
    assert!(titles1.contains(&"Finish project".to_string()));
    assert!(titles1.contains(&"Call mom".to_string()));
    
    assert_eq!(titles1, titles2);
}

#[test]
fn test_task_manager_concurrent_additions() {
    // Test concurrent task additions
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut lseq1 = Lseq::new(replica1);
    let mut lseq2 = Lseq::new(replica2);
    
    // Both start with one task
    let task1 = Task::new("Initial task".to_string());
    let pos1 = lseq1.insert_after(None, task1.clone()).unwrap();
    lseq2.merge(&lseq1).unwrap();
    
    // Concurrent additions at the same position
    let task2 = Task::new("Task from replica1".to_string());
    let task3 = Task::new("Task from replica2".to_string());
    
    let pos2 = lseq1.insert_after(Some(pos1.clone()), task2.clone()).unwrap();
    let pos3 = lseq2.insert_after(Some(pos1.clone()), task3.clone()).unwrap();
    
    // Merge both ways
    lseq1.merge(&lseq2).unwrap();
    lseq2.merge(&lseq1).unwrap();
    
    // Both should have all three tasks
    let tasks1: Vec<Task> = lseq1.to_vec().into_iter().collect();
    let tasks2: Vec<Task> = lseq2.to_vec().into_iter().collect();
    
    assert_eq!(tasks1.len(), 3);
    assert_eq!(tasks2.len(), 3);
    assert_eq!(tasks1, tasks2);
}

#[test]
fn test_task_manager_task_updates() {
    // Test updating existing tasks
    let replica = ReplicaId::from(Uuid::new_v4());
    let mut lseq = Lseq::new(replica);
    
    // Add a task
    let mut task = Task::new("Incomplete task".to_string());
    let pos = lseq.insert_after(None, task.clone()).unwrap();
    
    // Update the task
    task.completed = true;
    lseq.update(pos.clone(), task.clone()).unwrap();
    
    // Verify the update
    let tasks: Vec<Task> = lseq.to_vec().into_iter().collect();
    assert_eq!(tasks.len(), 1);
    assert!(tasks[0].completed);
    assert_eq!(tasks[0].title, "Incomplete task");
}

#[test]
fn test_task_manager_task_deletion() {
    // Test deleting tasks
    let replica = ReplicaId::from(Uuid::new_v4());
    let mut lseq = Lseq::new(replica);
    
    // Add multiple tasks
    let task1 = Task::new("Task 1".to_string());
    let task2 = Task::new("Task 2".to_string());
    let task3 = Task::new("Task 3".to_string());
    
    let pos1 = lseq.insert_after(None, task1.clone()).unwrap();
    let pos2 = lseq.insert_after(Some(pos1.clone()), task2.clone()).unwrap();
    let pos3 = lseq.insert_after(Some(pos2.clone()), task3.clone()).unwrap();
    
    // Delete the middle task
    lseq.delete(pos2).unwrap();
    
    // Verify deletion
    let tasks: Vec<Task> = lseq.to_vec().into_iter().collect();
    assert_eq!(tasks.len(), 2);
    
    let titles: Vec<String> = tasks.iter().map(|t| t.title.clone()).collect();
    assert!(titles.contains(&"Task 1".to_string()));
    assert!(titles.contains(&"Task 3".to_string()));
    assert!(!titles.contains(&"Task 2".to_string()));
}

#[test]
fn test_task_manager_merge_with_deletions() {
    // Test merging when one replica has deletions
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut lseq1 = Lseq::new(replica1);
    let mut lseq2 = Lseq::new(replica2);
    
    // Both start with the same tasks
    let task1 = Task::new("Task 1".to_string());
    let task2 = Task::new("Task 2".to_string());
    let task3 = Task::new("Task 3".to_string());
    
    let pos1 = lseq1.insert_after(None, task1.clone()).unwrap();
    let pos2 = lseq1.insert_after(Some(pos1.clone()), task2.clone()).unwrap();
    let pos3 = lseq1.insert_after(Some(pos2.clone()), task3.clone()).unwrap();
    
    lseq2.merge(&lseq1).unwrap();
    
    // Replica1 deletes a task, replica2 adds a new one
    lseq1.delete(pos2).unwrap();
    let task4 = Task::new("Task 4".to_string());
    let pos4 = lseq2.insert_after(Some(pos3.clone()), task4.clone()).unwrap();
    
    // Merge both ways
    lseq1.merge(&lseq2).unwrap();
    lseq2.merge(&lseq1).unwrap();
    
    // Both should have the same final state
    let tasks1: Vec<Task> = lseq1.to_vec().into_iter().collect();
    let tasks2: Vec<Task> = lseq2.to_vec().into_iter().collect();
    
    assert_eq!(tasks1.len(), 3); // Task 1, Task 3, Task 4
    assert_eq!(tasks2.len(), 3);
    assert_eq!(tasks1, tasks2);
}
