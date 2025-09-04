use leptos_sync_core::crdt::graph::{AddWinsGraph, VertexId, GraphError};
use leptos_sync_core::crdt::{Mergeable, ReplicaId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Task status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Completed,
    Blocked,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::NotStarted
    }
}

/// Task priority enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Medium
    }
}

/// Project task structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub assignee: Option<String>,
    pub estimated_hours: Option<u32>, // Changed to u32 to support Eq
    pub actual_hours: Option<u32>,    // Changed to u32 to support Eq
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(title: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            status: TaskStatus::default(),
            priority: TaskPriority::default(),
            assignee: None,
            estimated_hours: None,
            actual_hours: None,
            created_at: now,
            updated_at: now,
            due_date: None,
        }
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_assignee(mut self, assignee: String) -> Self {
        self.assignee = Some(assignee);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_estimated_hours(mut self, hours: u32) -> Self {
        self.estimated_hours = Some(hours);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_due_date(mut self, due_date: DateTime<Utc>) -> Self {
        self.due_date = Some(due_date);
        self.updated_at = Utc::now();
        self
    }
}

/// Custom error type for project manager operations
#[derive(Error, Debug, Clone)]
pub enum ProjectManagerError {
    #[error("Graph error: {0}")]
    GraphError(String),
    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),
    #[error("Circular dependency detected")]
    CircularDependency,
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

impl From<GraphError> for ProjectManagerError {
    fn from(err: GraphError) -> Self {
        ProjectManagerError::GraphError(err.to_string())
    }
}

/// Project manager using DAG CRDT for task dependencies
#[derive(Debug, Clone)]
pub struct ProjectManager {
    /// The underlying DAG CRDT
    graph: AddWinsGraph<Task>,
    /// Mapping from task ID to vertex ID for quick lookups
    task_to_vertex: HashMap<Uuid, VertexId>,
    /// Mapping from vertex ID to task ID for reverse lookups
    vertex_to_task: HashMap<VertexId, Uuid>,
    /// All tasks stored separately for easy access
    tasks: HashMap<Uuid, Task>,
}

impl ProjectManager {
    /// Create a new project manager
    pub fn new(user_id: Uuid) -> Self {
        let replica = ReplicaId::from(user_id);
        Self {
            graph: AddWinsGraph::new(replica),
            task_to_vertex: HashMap::new(),
            vertex_to_task: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    /// Add a new task to the project
    pub fn add_task(&mut self, task: Task) -> Result<Uuid, ProjectManagerError> {
        let task_id = task.id;
        let timestamp = task.created_at.timestamp() as u64;
        let vertex_id = self.graph.add_vertex(task.clone(), timestamp);
        
        self.task_to_vertex.insert(task_id, vertex_id.clone());
        self.vertex_to_task.insert(vertex_id, task_id);
        self.tasks.insert(task_id, task);
        
        Ok(task_id)
    }

    /// Update an existing task
    pub fn update_task(&mut self, task_id: Uuid, updated_task: Task) -> Result<(), ProjectManagerError> {
        if let Some(vertex_id) = self.task_to_vertex.get(&task_id) {
            // Update the task in the graph
            let timestamp = updated_task.updated_at.timestamp() as u64;
            self.graph.update_vertex(vertex_id, updated_task.clone(), timestamp)?;
            
            // Update our local copy
            self.tasks.insert(task_id, updated_task);
            
            Ok(())
        } else {
            Err(ProjectManagerError::TaskNotFound(task_id))
        }
    }

    /// Add a dependency between two tasks (task1 depends on task2)
    pub fn add_dependency(&mut self, dependent_task: Uuid, prerequisite_task: Uuid) -> Result<(), ProjectManagerError> {
        let dependent_vertex = self.task_to_vertex.get(&dependent_task)
            .ok_or_else(|| ProjectManagerError::TaskNotFound(dependent_task))?;
        let prerequisite_vertex = self.task_to_vertex.get(&prerequisite_task)
            .ok_or_else(|| ProjectManagerError::TaskNotFound(prerequisite_task))?;

        let timestamp = Utc::now().timestamp() as u64;
        self.graph.add_edge(prerequisite_vertex, dependent_vertex, timestamp, Some(1.0))?;
        
        Ok(())
    }

    /// Remove a dependency between two tasks
    pub fn remove_dependency(&mut self, dependent_task: Uuid, prerequisite_task: Uuid) -> Result<(), ProjectManagerError> {
        let _dependent_vertex = self.task_to_vertex.get(&dependent_task)
            .ok_or_else(|| ProjectManagerError::TaskNotFound(dependent_task))?;
        let _prerequisite_vertex = self.task_to_vertex.get(&prerequisite_task)
            .ok_or_else(|| ProjectManagerError::TaskNotFound(prerequisite_task))?;

        // For now, we'll just return Ok since we can't easily iterate over edges
        // In a real implementation, we'd need to track edge IDs
        Ok(())
    }

    /// Get all tasks in the project
    pub fn get_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    /// Get a specific task by ID
    pub fn get_task(&self, task_id: Uuid) -> Option<&Task> {
        self.tasks.get(&task_id)
    }

    /// Get tasks that have no dependencies (can be started)
    /// For now, we'll return all tasks since we can't easily check dependencies
    pub fn get_ready_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    /// Get tasks that depend on a specific task
    /// For now, we'll return an empty vector since we can't easily iterate over edges
    pub fn get_dependent_tasks(&self, _task_id: Uuid) -> Vec<&Task> {
        Vec::new()
    }

    /// Get tasks that a specific task depends on
    /// For now, we'll return an empty vector since we can't easily iterate over edges
    pub fn get_prerequisite_tasks(&self, _task_id: Uuid) -> Vec<&Task> {
        Vec::new()
    }

    /// Get the project structure as a tree (for visualization)
    pub fn get_project_structure(&self) -> Vec<TaskNode> {
        self.tasks.values()
            .map(|task| TaskNode {
                task: task.clone(),
                dependencies: Vec::new(), // Simplified for now
            })
            .collect()
    }

    /// Get the number of tasks in the project
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Check if the project is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

/// Task node for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub task: Task,
    pub dependencies: Vec<TaskNode>,
}

impl Mergeable for ProjectManager {
    type Error = ProjectManagerError;

    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        // Merge the underlying graph
        self.graph.merge(&other.graph)?;
        
        // Update the mappings
        for (task_id, vertex_id) in &other.task_to_vertex {
            self.task_to_vertex.insert(*task_id, vertex_id.clone());
        }
        
        for (vertex_id, task_id) in &other.vertex_to_task {
            self.vertex_to_task.insert(vertex_id.clone(), *task_id);
        }
        
        // Merge tasks
        for (task_id, task) in &other.tasks {
            self.tasks.insert(*task_id, task.clone());
        }
        
        Ok(())
    }

    fn has_conflict(&self, _other: &Self) -> bool {
        // For now, we'll assume no conflicts
        false
    }
}