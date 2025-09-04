use crate::project_manager::{
    ProjectManager, Task, TaskStatus, TaskPriority, ProjectManagerError
};
use leptos_sync_core::crdt::Mergeable;
use uuid::Uuid;

#[test]
fn test_create_project_manager() {
    let user_id = Uuid::new_v4();
    let manager = ProjectManager::new(user_id);
    
    assert!(manager.is_empty());
    assert_eq!(manager.len(), 0);
}

#[test]
fn test_add_task() {
    let user_id = Uuid::new_v4();
    let mut manager = ProjectManager::new(user_id);
    
    let task = Task::new("Test Task".to_string(), "A test task".to_string());
    let task_id = manager.add_task(task.clone()).unwrap();
    
    assert_eq!(manager.len(), 1);
    assert!(!manager.is_empty());
    
    let retrieved_task = manager.get_task(task_id).unwrap();
    assert_eq!(retrieved_task.title, "Test Task");
    assert_eq!(retrieved_task.description, "A test task");
}

#[test]
fn test_add_multiple_tasks() {
    let user_id = Uuid::new_v4();
    let mut manager = ProjectManager::new(user_id);
    
    let task1 = Task::new("Task 1".to_string(), "First task".to_string());
    let task2 = Task::new("Task 2".to_string(), "Second task".to_string());
    
    let task1_id = manager.add_task(task1).unwrap();
    let task2_id = manager.add_task(task2).unwrap();
    
    assert_eq!(manager.len(), 2);
    
    let tasks = manager.get_tasks();
    assert_eq!(tasks.len(), 2);
    
    // Verify both tasks exist
    assert!(manager.get_task(task1_id).is_some());
    assert!(manager.get_task(task2_id).is_some());
}

#[test]
fn test_add_dependency() {
    let user_id = Uuid::new_v4();
    let mut manager = ProjectManager::new(user_id);
    
    let task1 = Task::new("Prerequisite".to_string(), "Must be done first".to_string());
    let task2 = Task::new("Dependent".to_string(), "Depends on task1".to_string());
    
    let task1_id = manager.add_task(task1).unwrap();
    let task2_id = manager.add_task(task2).unwrap();
    
    // Add dependency: task2 depends on task1
    let result = manager.add_dependency(task2_id, task1_id);
    assert!(result.is_ok());
}

#[test]
fn test_update_task() {
    let user_id = Uuid::new_v4();
    let mut manager = ProjectManager::new(user_id);
    
    let mut task = Task::new("Original Title".to_string(), "Original description".to_string());
    let task_id = manager.add_task(task.clone()).unwrap();
    
    // Update the task
    task.title = "Updated Title".to_string();
    task.description = "Updated description".to_string();
    task.status = TaskStatus::InProgress;
    task.priority = TaskPriority::High;
    
    manager.update_task(task_id, task.clone()).unwrap();
    
    // Verify the update
    let updated_task = manager.get_task(task_id).unwrap();
    assert_eq!(updated_task.title, "Updated Title");
    assert_eq!(updated_task.description, "Updated description");
    assert_eq!(updated_task.status, TaskStatus::InProgress);
    assert_eq!(updated_task.priority, TaskPriority::High);
}

#[test]
fn test_merge_project_managers() {
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();
    
    let mut manager1 = ProjectManager::new(user1_id);
    let mut manager2 = ProjectManager::new(user2_id);
    
    // Add tasks to manager1
    let task1 = Task::new("Task 1".to_string(), "From manager 1".to_string());
    let task1_id = manager1.add_task(task1).unwrap();
    
    // Add tasks to manager2
    let task2 = Task::new("Task 2".to_string(), "From manager 2".to_string());
    let task2_id = manager2.add_task(task2).unwrap();
    
    // Merge manager2 into manager1
    manager1.merge(&manager2).unwrap();
    
    // Verify both tasks exist in manager1
    assert_eq!(manager1.len(), 2);
    assert!(manager1.get_task(task1_id).is_some());
    assert!(manager1.get_task(task2_id).is_some());
}

#[test]
fn test_task_not_found_error() {
    let user_id = Uuid::new_v4();
    let mut manager = ProjectManager::new(user_id);
    
    let non_existent_id = Uuid::new_v4();
    let result = manager.update_task(non_existent_id, Task::new("Test".to_string(), "Test".to_string()));
    
    assert!(matches!(result, Err(ProjectManagerError::TaskNotFound(_))));
}

#[test]
fn test_project_structure() {
    let user_id = Uuid::new_v4();
    let mut manager = ProjectManager::new(user_id);
    
    // Create a simple project structure
    let task1 = Task::new("Setup".to_string(), "Project setup".to_string());
    let task2 = Task::new("Development".to_string(), "Main development".to_string());
    let task3 = Task::new("Testing".to_string(), "Testing phase".to_string());
    
    let _task1_id = manager.add_task(task1).unwrap();
    let _task2_id = manager.add_task(task2).unwrap();
    let _task3_id = manager.add_task(task3).unwrap();
    
    // Get project structure
    let structure = manager.get_project_structure();
    
    // Should have three tasks
    assert_eq!(structure.len(), 3);
}

#[test]
fn test_ready_tasks() {
    let user_id = Uuid::new_v4();
    let mut manager = ProjectManager::new(user_id);
    
    let task1 = Task::new("Independent Task".to_string(), "No dependencies".to_string());
    let task2 = Task::new("Dependent Task".to_string(), "Depends on task1".to_string());
    
    let _task1_id = manager.add_task(task1).unwrap();
    let _task2_id = manager.add_task(task2).unwrap();
    
    // All tasks are ready (simplified implementation)
    let ready_tasks = manager.get_ready_tasks();
    assert_eq!(ready_tasks.len(), 2);
}