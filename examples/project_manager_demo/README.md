# Project Manager Demo - DAG CRDT

A collaborative project manager demonstrating the DAG (Directed Acyclic Graph) CRDT implementation.

## Overview

This demo showcases project management with task dependencies using the DAG CRDT, which provides graph-based data structures with conflict-free merging of concurrent operations.

## Features

- **Task Dependencies**: Create and manage task dependencies
- **Project Organization**: Organize tasks in hierarchical structures
- **Dependency Visualization**: Visualize task relationships and dependencies
- **Collaborative Management**: Multiple users can manage projects simultaneously
- **Task Tracking**: Track task status, priority, and progress

## Running the Demo

```bash
cd examples/project_manager_demo
trunk serve
```

Access the demo at: http://localhost:8083/

## Architecture

### CRDT Type: DAG (Directed Acyclic Graph)

DAG is optimized for:
- Graph-based data structures
- Dependency management
- Conflict-free merging of concurrent edits
- Maintaining graph structure across replicas

### Key Components

- **ProjectManager**: Main application wrapper around DAG
- **Task**: Task data structure with metadata
- **AddWinsGraph**: Core CRDT implementation for graph structures
- **Leptos UI**: Reactive web interface

## API Usage

```rust
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
    assignee: Option<String>,
    estimated_hours: Option<u32>,
    actual_hours: Option<u32>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
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

// Create a new DAG instance
let replica = ReplicaId::from(Uuid::new_v4());
let mut graph = AddWinsGraph::new(replica);

// Add tasks (vertices)
let task1 = Task {
    id: Uuid::new_v4(),
    title: "Setup project".to_string(),
    description: "Initialize the project structure".to_string(),
    status: TaskStatus::NotStarted,
    priority: TaskPriority::High,
    assignee: Some("Alice".to_string()),
    estimated_hours: Some(8),
    actual_hours: None,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
    due_date: Some(chrono::Utc::now() + chrono::Duration::days(3)),
};
let vertex1 = graph.add_vertex(task1, 1000);

let task2 = Task {
    id: Uuid::new_v4(),
    title: "Write tests".to_string(),
    description: "Create unit tests for the project".to_string(),
    status: TaskStatus::NotStarted,
    priority: TaskPriority::Medium,
    assignee: Some("Bob".to_string()),
    estimated_hours: Some(16),
    actual_hours: None,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
    due_date: Some(chrono::Utc::now() + chrono::Duration::days(7)),
};
let vertex2 = graph.add_vertex(task2, 2000);

// Add dependencies (edges)
let edge1 = graph.add_edge(&vertex1, &vertex2, 3000, Some(1.0))?;

// Update task status
let mut updated_task = task1.clone();
updated_task.status = TaskStatus::InProgress;
graph.update_vertex(&vertex1, updated_task, 4000)?;

// Merge with another graph
let mut other_graph = AddWinsGraph::new(ReplicaId::from(Uuid::new_v4()));
let other_task = Task { /* ... */ };
let other_vertex = other_graph.add_vertex(other_task, 5000);
graph.merge(&other_graph)?;
```

## Testing

Run the demo tests:

```bash
cargo test -p project_manager_demo
```

## Performance

- **Vertex Operations**: O(1) average case
- **Edge Operations**: O(1) average case
- **Merge**: O(n) where n is the number of vertices/edges
- **Memory**: O(n) where n is the number of vertices/edges

## Task Management

### Task States
- **Not Started**: Task has been created but not begun
- **In Progress**: Task is currently being worked on
- **Completed**: Task has been finished
- **Blocked**: Task cannot proceed due to dependencies

### Priority Levels
- **Low**: Low priority tasks
- **Medium**: Normal priority tasks
- **High**: Important tasks
- **Critical**: Urgent tasks that need immediate attention

### Dependencies
- **Blocking Dependencies**: Task B cannot start until Task A is completed
- **Soft Dependencies**: Task B can start but benefits from Task A being completed
- **Dependency Weight**: Strength of the dependency relationship

## Use Cases

- Project management tools
- Workflow management
- Task scheduling
- Resource planning
- Team coordination
- Agile project management
- Gantt chart applications

## Limitations

- No built-in resource management
- No time tracking beyond estimated/actual hours
- No task templates or recurring tasks
- No file attachments
- No advanced scheduling algorithms

## Future Enhancements

- Resource management and allocation
- Advanced time tracking
- Task templates and recurring tasks
- File attachments and comments
- Gantt chart visualization
- Critical path analysis
- Resource leveling
- Integration with external tools
- Advanced scheduling algorithms
- Risk management features
