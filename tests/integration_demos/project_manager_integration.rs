//! Integration tests for Project Manager Demo using DAG CRDT

use leptos_sync_core::crdt::graph::AddWinsGraph;
use leptos_sync_core::crdt::{Mergeable, ReplicaId};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Task {
    id: Uuid,
    title: String,
    description: String,
    status: TaskStatus,
    priority: TaskPriority,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TaskStatus {
    NotStarted,
    InProgress,
    Completed,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Task {
    fn new(title: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            status: TaskStatus::NotStarted,
            priority: TaskPriority::Medium,
        }
    }
}

#[test]
fn test_project_manager_dag_integration() {
    // Test basic DAG functionality for project management
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut graph1 = AddWinsGraph::new(replica1);
    let mut graph2 = AddWinsGraph::new(replica2);
    
    // Add tasks from replica1
    let task1 = Task::new("Setup project".to_string(), "Initialize the project structure".to_string());
    let task2 = Task::new("Write tests".to_string(), "Create unit tests for the project".to_string());
    
    let timestamp1 = 1000;
    let timestamp2 = 2000;
    
    let vertex1 = graph1.add_vertex(task1.clone(), timestamp1);
    let vertex2 = graph1.add_vertex(task2.clone(), timestamp2);
    
    // Add tasks from replica2
    let task3 = Task::new("Deploy to production".to_string(), "Deploy the application to production".to_string());
    let task4 = Task::new("Monitor performance".to_string(), "Set up monitoring and alerting".to_string());
    
    let timestamp3 = 1500;
    let timestamp4 = 2500;
    
    let vertex3 = graph2.add_vertex(task3.clone(), timestamp3);
    let vertex4 = graph2.add_vertex(task4.clone(), timestamp4);
    
    // Merge both ways
    graph1.merge(&graph2).unwrap();
    graph2.merge(&graph1).unwrap();
    
    // Both graphs should have the same vertices
    // Note: We can't easily iterate over vertices in the current API,
    // but we can verify the merge was successful by checking if we can
    // access vertices that were added by the other replica
    assert!(graph1.get_vertex(&vertex3).is_some());
    assert!(graph1.get_vertex(&vertex4).is_some());
    assert!(graph2.get_vertex(&vertex1).is_some());
    assert!(graph2.get_vertex(&vertex2).is_some());
}

#[test]
fn test_project_manager_task_dependencies() {
    // Test task dependency management
    let replica = ReplicaId::from(Uuid::new_v4());
    let mut graph = AddWinsGraph::new(replica);
    
    // Create tasks
    let task1 = Task::new("Design system".to_string(), "Create the system design".to_string());
    let task2 = Task::new("Implement features".to_string(), "Implement the core features".to_string());
    let task3 = Task::new("Test system".to_string(), "Test the implemented system".to_string());
    
    let timestamp1 = 1000;
    let timestamp2 = 2000;
    let timestamp3 = 3000;
    
    let vertex1 = graph.add_vertex(task1.clone(), timestamp1);
    let vertex2 = graph.add_vertex(task2.clone(), timestamp2);
    let vertex3 = graph.add_vertex(task3.clone(), timestamp3);
    
    // Add dependencies: task2 depends on task1, task3 depends on task2
    let edge1 = graph.add_edge(&vertex1, &vertex2, timestamp2 + 1, Some(1.0)).unwrap();
    let edge2 = graph.add_edge(&vertex2, &vertex3, timestamp3 + 1, Some(1.0)).unwrap();
    
    // Verify edges exist
    assert!(graph.get_edge(&edge1).is_some());
    assert!(graph.get_edge(&edge2).is_some());
}

#[test]
fn test_project_manager_concurrent_task_creation() {
    // Test concurrent task creation
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut graph1 = AddWinsGraph::new(replica1);
    let mut graph2 = AddWinsGraph::new(replica2);
    
    // Both start with one task
    let task1 = Task::new("Initial task".to_string(), "The first task".to_string());
    let vertex1 = graph1.add_vertex(task1.clone(), 1000);
    graph2.merge(&graph1).unwrap();
    
    // Concurrent task creation
    let task2 = Task::new("Task from replica1".to_string(), "Created by replica1".to_string());
    let task3 = Task::new("Task from replica2".to_string(), "Created by replica2".to_string());
    
    let vertex2 = graph1.add_vertex(task2.clone(), 2000);
    let vertex3 = graph2.add_vertex(task3.clone(), 2500);
    
    // Merge both ways
    graph1.merge(&graph2).unwrap();
    graph2.merge(&graph1).unwrap();
    
    // Both should have all three tasks
    assert!(graph1.get_vertex(&vertex1).is_some());
    assert!(graph1.get_vertex(&vertex2).is_some());
    assert!(graph1.get_vertex(&vertex3).is_some());
    
    assert!(graph2.get_vertex(&vertex1).is_some());
    assert!(graph2.get_vertex(&vertex2).is_some());
    assert!(graph2.get_vertex(&vertex3).is_some());
}

#[test]
fn test_project_manager_task_updates() {
    // Test updating existing tasks
    let replica = ReplicaId::from(Uuid::new_v4());
    let mut graph = AddWinsGraph::new(replica);
    
    // Add a task
    let mut task = Task::new("Incomplete task".to_string(), "This task needs work".to_string());
    let vertex = graph.add_vertex(task.clone(), 1000);
    
    // Update the task
    task.status = TaskStatus::InProgress;
    task.priority = TaskPriority::High;
    graph.update_vertex(&vertex, task.clone(), 2000).unwrap();
    
    // Verify the update
    let updated_vertex = graph.get_vertex(&vertex).unwrap();
    assert_eq!(updated_vertex.value.status, TaskStatus::InProgress);
    assert_eq!(updated_vertex.value.priority, TaskPriority::High);
}

#[test]
fn test_project_manager_task_deletion() {
    // Test deleting tasks
    let replica = ReplicaId::from(Uuid::new_v4());
    let mut graph = AddWinsGraph::new(replica);
    
    // Add multiple tasks
    let task1 = Task::new("Task 1".to_string(), "First task".to_string());
    let task2 = Task::new("Task 2".to_string(), "Second task".to_string());
    let task3 = Task::new("Task 3".to_string(), "Third task".to_string());
    
    let vertex1 = graph.add_vertex(task1.clone(), 1000);
    let vertex2 = graph.add_vertex(task2.clone(), 2000);
    let vertex3 = graph.add_vertex(task3.clone(), 3000);
    
    // Delete the middle task
    graph.remove_vertex(&vertex2, 4000).unwrap();
    
    // Verify deletion
    assert!(graph.get_vertex(&vertex1).is_some());
    assert!(graph.get_vertex(&vertex2).is_none()); // Should be deleted
    assert!(graph.get_vertex(&vertex3).is_some());
}

#[test]
fn test_project_manager_merge_with_deletions() {
    // Test merging when one replica has deletions
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut graph1 = AddWinsGraph::new(replica1);
    let mut graph2 = AddWinsGraph::new(replica2);
    
    // Both start with the same tasks
    let task1 = Task::new("Task 1".to_string(), "First task".to_string());
    let task2 = Task::new("Task 2".to_string(), "Second task".to_string());
    let task3 = Task::new("Task 3".to_string(), "Third task".to_string());
    
    let vertex1 = graph1.add_vertex(task1.clone(), 1000);
    let vertex2 = graph1.add_vertex(task2.clone(), 2000);
    let vertex3 = graph1.add_vertex(task3.clone(), 3000);
    
    graph2.merge(&graph1).unwrap();
    
    // Replica1 deletes a task, replica2 adds a new one
    graph1.remove_vertex(&vertex2, 4000).unwrap();
    let task4 = Task::new("Task 4".to_string(), "Fourth task".to_string());
    let vertex4 = graph2.add_vertex(task4.clone(), 5000);
    
    // Merge both ways
    graph1.merge(&graph2).unwrap();
    graph2.merge(&graph1).unwrap();
    
    // Both should have the same final state
    assert!(graph1.get_vertex(&vertex1).is_some());
    assert!(graph1.get_vertex(&vertex2).is_none()); // Deleted
    assert!(graph1.get_vertex(&vertex3).is_some());
    assert!(graph1.get_vertex(&vertex4).is_some());
    
    assert!(graph2.get_vertex(&vertex1).is_some());
    assert!(graph2.get_vertex(&vertex2).is_none()); // Deleted
    assert!(graph2.get_vertex(&vertex3).is_some());
    assert!(graph2.get_vertex(&vertex4).is_some());
}
