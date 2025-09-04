# Task Manager Demo - LSEQ CRDT

A collaborative task manager demonstrating the LSEQ (Logoot Sequence) CRDT implementation.

## Overview

This demo showcases collaborative task management using the LSEQ CRDT, which provides ordered sequences with conflict-free merging of concurrent operations.

## Features

- **Ordered Task Lists**: Tasks maintain consistent ordering across replicas
- **Task Management**: Create, update, and delete tasks
- **Priority System**: High, Medium, Low, Critical priority levels
- **Status Tracking**: Not Started, In Progress, Completed, Blocked statuses
- **Collaborative Editing**: Multiple users can manage tasks simultaneously

## Running the Demo

```bash
cd examples/task_manager_demo
trunk serve
```

Access the demo at: http://localhost:3001/

## Architecture

### CRDT Type: LSEQ (Logoot Sequence)

LSEQ is optimized for:
- Ordered sequence operations
- Efficient insertion and deletion
- Conflict-free merging of concurrent edits
- Maintaining sequence ordering across replicas

### Key Components

- **TaskManager**: Main application wrapper around LSEQ
- **Task**: Task data structure with metadata
- **LSEQ**: Core CRDT implementation for ordered sequences
- **Leptos UI**: Reactive web interface

## API Usage

```rust
use leptos_sync_core::crdt::advanced::Lseq;
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

// Create a new LSEQ instance
let replica = ReplicaId::from(Uuid::new_v4());
let mut lseq = Lseq::new(replica);

// Add tasks
let task1 = Task {
    id: Uuid::new_v4(),
    title: "Buy groceries".to_string(),
    description: "Get milk, bread, and eggs".to_string(),
    status: TaskStatus::NotStarted,
    priority: TaskPriority::Medium,
    assignee: Some("Alice".to_string()),
    estimated_hours: Some(1),
    actual_hours: None,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
    due_date: Some(chrono::Utc::now() + chrono::Duration::days(1)),
};
let pos1 = lseq.insert(task1, None)?;

// Add another task
let task2 = Task {
    id: Uuid::new_v4(),
    title: "Walk the dog".to_string(),
    description: "Take the dog for a 30-minute walk".to_string(),
    status: TaskStatus::InProgress,
    priority: TaskPriority::High,
    assignee: Some("Bob".to_string()),
    estimated_hours: Some(1),
    actual_hours: Some(1),
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
    due_date: None,
};
let pos2 = lseq.insert(task2, Some(pos1))?;

// Get all tasks
let tasks: Vec<Task> = lseq.to_vec();

// Merge with another LSEQ
let mut other_lseq = Lseq::new(ReplicaId::from(Uuid::new_v4()));
let other_task = Task { /* ... */ };
other_lseq.insert(other_task, None)?;
lseq.merge(&other_lseq)?;
```

## Testing

Run the demo tests:

```bash
cargo test -p task_manager_demo
```

## Performance

- **Insertion**: O(log n) average case
- **Deletion**: O(log n) average case
- **Merge**: O(n) where n is the number of operations
- **Memory**: O(n) where n is the number of tasks

## Use Cases

- Project management tools
- Task tracking applications
- Collaborative to-do lists
- Workflow management
- Team coordination tools

## Task States

### Status
- **Not Started**: Task has been created but not begun
- **In Progress**: Task is currently being worked on
- **Completed**: Task has been finished
- **Blocked**: Task cannot proceed due to dependencies

### Priority
- **Low**: Low priority tasks
- **Medium**: Normal priority tasks
- **High**: Important tasks
- **Critical**: Urgent tasks that need immediate attention

## Limitations

- No built-in task dependencies
- No task templates or recurring tasks
- No file attachments
- No time tracking beyond estimated/actual hours

## Future Enhancements

- Task dependencies and relationships
- Task templates and recurring tasks
- File attachments and comments
- Advanced time tracking
- Task filtering and search
- Task assignment and notifications
- Integration with external tools
