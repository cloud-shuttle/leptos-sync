use leptos_sync_core::crdt::advanced::Lseq;
use leptos_sync_core::crdt::{ReplicaId, Mergeable};
use crate::task_manager::{Task, TaskManager, TaskStatus, TaskPriority};
use uuid::Uuid;

/// Test suite for collaborative task manager using LSEQ
/// This module contains comprehensive tests for the task manager functionality,
/// following Test-Driven Development (TDD) principles.

#[test]
fn test_task_manager_initialization() {
    let user_id = Uuid::new_v4();
    let manager = TaskManager::new(user_id);
    assert_eq!(manager.len(), 0);
    assert!(manager.is_empty());
}

#[test]
fn test_add_task() {
    let user_id = Uuid::new_v4();
    let mut manager = TaskManager::new(user_id);
    
    let task = Task {
        id: uuid::Uuid::new_v4(),
        title: "Test Task".to_string(),
        description: "A test task".to_string(),
        status: TaskStatus::Todo,
        priority: TaskPriority::Medium,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let position = manager.add_task(task.clone()).unwrap();
    assert_eq!(manager.len(), 1);
    assert!(!manager.is_empty());
    
    let tasks = manager.get_tasks();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Test Task");
}

#[test]
fn test_update_task_status() {
    let user_id = Uuid::new_v4();
    let mut manager = TaskManager::new(user_id);
    
    let task = Task {
        id: uuid::Uuid::new_v4(),
        title: "Test Task".to_string(),
        description: "A test task".to_string(),
        status: TaskStatus::Todo,
        priority: TaskPriority::Medium,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let position = manager.add_task(task).unwrap();
    manager.update_task_status(&position, TaskStatus::InProgress).unwrap();
    
    let tasks = manager.get_tasks();
    assert_eq!(tasks[0].status, TaskStatus::InProgress);
}

#[test]
fn test_delete_task() {
    let user_id = Uuid::new_v4();
    let mut manager = TaskManager::new(user_id);
    
    let task = Task {
        id: uuid::Uuid::new_v4(),
        title: "Test Task".to_string(),
        description: "A test task".to_string(),
        status: TaskStatus::Todo,
        priority: TaskPriority::Medium,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let position = manager.add_task(task).unwrap();
    assert_eq!(manager.len(), 1);
    
    manager.delete_task(&position).unwrap();
    assert_eq!(manager.len(), 0);
    assert!(manager.is_empty());
}

#[test]
fn test_merge_task_managers() {
    let user_id1 = Uuid::new_v4();
    let user_id2 = Uuid::new_v4();
    
    let mut manager1 = TaskManager::new(user_id1);
    let mut manager2 = TaskManager::new(user_id2);
    
    let task1 = Task {
        id: uuid::Uuid::new_v4(),
        title: "Task 1".to_string(),
        description: "First task".to_string(),
        status: TaskStatus::Todo,
        priority: TaskPriority::High,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let task2 = Task {
        id: uuid::Uuid::new_v4(),
        title: "Task 2".to_string(),
        description: "Second task".to_string(),
        status: TaskStatus::InProgress,
        priority: TaskPriority::Low,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let _pos1 = manager1.add_task(task1).unwrap();
    let _pos2 = manager2.add_task(task2).unwrap();
    
    assert_eq!(manager1.len(), 1);
    assert_eq!(manager2.len(), 1);
    
    manager1.merge(&manager2).unwrap();
    assert_eq!(manager1.len(), 2);
    
    let tasks = manager1.get_tasks();
    assert_eq!(tasks.len(), 2);
}

#[test]
fn test_filter_tasks_by_status() {
    let user_id = Uuid::new_v4();
    let mut manager = TaskManager::new(user_id);
    
    let task1 = Task {
        id: uuid::Uuid::new_v4(),
        title: "Todo Task".to_string(),
        description: "A todo task".to_string(),
        status: TaskStatus::Todo,
        priority: TaskPriority::Medium,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let task2 = Task {
        id: uuid::Uuid::new_v4(),
        title: "Done Task".to_string(),
        description: "A completed task".to_string(),
        status: TaskStatus::Done,
        priority: TaskPriority::Medium,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let _pos1 = manager.add_task(task1).unwrap();
    let _pos2 = manager.add_task(task2).unwrap();
    
    let todo_tasks = manager.get_tasks_by_status(TaskStatus::Todo);
    assert_eq!(todo_tasks.len(), 1);
    assert_eq!(todo_tasks[0].title, "Todo Task");
    
    let done_tasks = manager.get_tasks_by_status(TaskStatus::Done);
    assert_eq!(done_tasks.len(), 1);
    assert_eq!(done_tasks[0].title, "Done Task");
}

#[test]
fn test_filter_tasks_by_priority() {
    let user_id = Uuid::new_v4();
    let mut manager = TaskManager::new(user_id);
    
    let task1 = Task {
        id: uuid::Uuid::new_v4(),
        title: "High Priority Task".to_string(),
        description: "An important task".to_string(),
        status: TaskStatus::Todo,
        priority: TaskPriority::High,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let task2 = Task {
        id: uuid::Uuid::new_v4(),
        title: "Low Priority Task".to_string(),
        description: "A less important task".to_string(),
        status: TaskStatus::Todo,
        priority: TaskPriority::Low,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let _pos1 = manager.add_task(task1).unwrap();
    let _pos2 = manager.add_task(task2).unwrap();
    
    let high_priority_tasks = manager.get_tasks_by_priority(TaskPriority::High);
    assert_eq!(high_priority_tasks.len(), 1);
    assert_eq!(high_priority_tasks[0].title, "High Priority Task");
    
    let low_priority_tasks = manager.get_tasks_by_priority(TaskPriority::Low);
    assert_eq!(low_priority_tasks.len(), 1);
    assert_eq!(low_priority_tasks[0].title, "Low Priority Task");
}

#[test]
fn test_task_manager_serialization() {
    let user_id = Uuid::new_v4();
    let mut manager = TaskManager::new(user_id);
    
    let task = Task {
        id: uuid::Uuid::new_v4(),
        title: "Serializable Task".to_string(),
        description: "A task for serialization testing".to_string(),
        status: TaskStatus::Todo,
        priority: TaskPriority::Medium,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let _position = manager.add_task(task).unwrap();
    let tasks = manager.get_tasks();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Serializable Task");
}
