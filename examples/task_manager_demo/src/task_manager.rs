use leptos_sync_core::crdt::advanced::Lseq;
use leptos_sync_core::crdt::{PositionId, ReplicaId, Mergeable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::error::Error;

/// Task status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "Todo"),
            TaskStatus::InProgress => write!(f, "In Progress"),
            TaskStatus::Done => write!(f, "Done"),
            TaskStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Task priority enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "Low"),
            TaskPriority::Medium => write!(f, "Medium"),
            TaskPriority::High => write!(f, "High"),
            TaskPriority::Critical => write!(f, "Critical"),
        }
    }
}

/// Task structure representing a collaborative task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier
    pub id: Uuid,
    /// Task title
    pub title: String,
    /// Task description
    pub description: String,
    /// Current task status
    pub status: TaskStatus,
    /// Task priority
    pub priority: TaskPriority,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Task {
    /// Create a new task
    pub fn new(title: String, description: String, priority: TaskPriority) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            status: TaskStatus::Todo,
            priority,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Update the task status
    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
    
    /// Update the task priority
    pub fn update_priority(&mut self, priority: TaskPriority) {
        self.priority = priority;
        self.updated_at = Utc::now();
    }
    
    /// Update the task title
    pub fn update_title(&mut self, title: String) {
        self.title = title;
        self.updated_at = Utc::now();
    }
    
    /// Update the task description
    pub fn update_description(&mut self, description: String) {
        self.description = description;
        self.updated_at = Utc::now();
    }
}

/// Task update event for real-time collaboration
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskUpdate {
    pub user_id: Uuid,
    pub task_id: Uuid,
    pub position: Option<PositionId>,
    pub update_type: TaskUpdateType,
}

/// Types of task updates
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskUpdateType {
    Created,
    StatusChanged(TaskStatus),
    PriorityChanged(TaskPriority),
    TitleChanged(String),
    DescriptionChanged(String),
    Deleted,
}

/// Collaborative Task Manager using LSEQ CRDT
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskManager {
    /// LSEQ for storing tasks in order
    tasks: Lseq<Task>,
    /// User ID for this manager instance
    user_id: Uuid,
    /// Position to task ID mapping for quick lookups
    position_to_task_id: HashMap<PositionId, Uuid>,
    /// Task ID to position mapping for quick lookups
    task_id_to_position: HashMap<Uuid, PositionId>,
}

impl TaskManager {
    /// Create a new task manager
    pub fn new(user_id: Uuid) -> Self {
        let replica_id = ReplicaId::default();
        Self {
            tasks: Lseq::new(replica_id),
            user_id,
            position_to_task_id: HashMap::new(),
            task_id_to_position: HashMap::new(),
        }
    }
    
    /// Add a new task
    pub fn add_task(&mut self, task: Task) -> Result<PositionId, String> {
        let position = self.tasks.insert(task.clone(), None)
            .map_err(|e| e.to_string())?;
        
        self.position_to_task_id.insert(position.clone(), task.id);
        self.task_id_to_position.insert(task.id, position.clone());
        
        Ok(position)
    }
    
    /// Update task status
    pub fn update_task_status(&mut self, position: &PositionId, status: TaskStatus) -> Result<(), String> {
        let task_id = if let Some(task_id) = self.position_to_task_id.get(position) {
            *task_id
        } else {
            return Ok(());
        };
        
        // Find the task in the LSEQ and update it
        if let Some(element) = self.tasks.get_elements().get(position) {
            if element.visible {
                let mut updated_task = element.value.clone();
                updated_task.update_status(status);
                
                // Remove old task and add updated one
                self.tasks.delete(position).map_err(|e| e.to_string())?;
                let new_position = self.tasks.insert(updated_task, None)
                    .map_err(|e| e.to_string())?;
                
                // Update mappings
                self.position_to_task_id.remove(position);
                self.task_id_to_position.remove(&task_id);
                self.position_to_task_id.insert(new_position.clone(), task_id);
                self.task_id_to_position.insert(task_id, new_position);
            }
        }
        Ok(())
    }
    
    /// Update task priority
    pub fn update_task_priority(&mut self, position: &PositionId, priority: TaskPriority) -> Result<(), String> {
        let task_id = if let Some(task_id) = self.position_to_task_id.get(position) {
            *task_id
        } else {
            return Ok(());
        };
        
        if let Some(element) = self.tasks.get_elements().get(position) {
            if element.visible {
                let mut updated_task = element.value.clone();
                updated_task.update_priority(priority);
                
                self.tasks.delete(position).map_err(|e| e.to_string())?;
                let new_position = self.tasks.insert(updated_task, None)
                    .map_err(|e| e.to_string())?;
                
                self.position_to_task_id.remove(position);
                self.task_id_to_position.remove(&task_id);
                self.position_to_task_id.insert(new_position.clone(), task_id);
                self.task_id_to_position.insert(task_id, new_position);
            }
        }
        Ok(())
    }
    
    /// Delete a task
    pub fn delete_task(&mut self, position: &PositionId) -> Result<(), String> {
        let task_id = if let Some(task_id) = self.position_to_task_id.get(position) {
            *task_id
        } else {
            return Ok(());
        };
        
        self.tasks.delete(position).map_err(|e| e.to_string())?;
        self.position_to_task_id.remove(position);
        self.task_id_to_position.remove(&task_id);
        Ok(())
    }
    
    /// Get all tasks
    pub fn get_tasks(&self) -> Vec<Task> {
        self.tasks.to_vec()
    }
    
    /// Get tasks by status
    pub fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<Task> {
        self.tasks.to_vec()
            .into_iter()
            .filter(|task| task.status == status)
            .collect()
    }
    
    /// Get tasks by priority
    pub fn get_tasks_by_priority(&self, priority: TaskPriority) -> Vec<Task> {
        self.tasks.to_vec()
            .into_iter()
            .filter(|task| task.priority == priority)
            .collect()
    }
    
    /// Get task count
    pub fn len(&self) -> usize {
        self.tasks.to_vec().len()
    }
    
    /// Check if task manager is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.to_vec().is_empty()
    }
    
    /// Get user ID
    pub fn get_user_id(&self) -> Uuid {
        self.user_id
    }
    
    /// Get task updates for real-time collaboration
    pub fn get_task_updates(&self) -> Vec<TaskUpdate> {
        // Simplified implementation - in a real app, this would track actual updates
        vec![]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TaskManagerError {
    #[error("LSEQ error: {0}")]
    LseqError(String),
}

impl Mergeable for TaskManager {
    type Error = TaskManagerError;
    
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        // Merge the underlying LSEQ
        self.tasks.merge(&other.tasks).map_err(|e| TaskManagerError::LseqError(e.to_string()))?;
        
        // Rebuild position mappings
        self.position_to_task_id.clear();
        self.task_id_to_position.clear();
        
        for (position, element) in self.tasks.get_elements() {
            if element.visible {
                self.position_to_task_id.insert(position.clone(), element.value.id);
                self.task_id_to_position.insert(element.value.id, position.clone());
            }
        }
        
        Ok(())
    }
    
    fn has_conflict(&self, other: &Self) -> bool {
        self.tasks.has_conflict(&other.tasks)
    }
}
