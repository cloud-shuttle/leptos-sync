# Collaborative Application Demos

> **⚠️ Generated Code Disclaimer**: This documentation and the associated demo applications contain code generated with the assistance of AI tools. While the core functionality has been thoroughly tested and validated, please review all code before use in production environments.

This document provides comprehensive documentation for the four collaborative application demos built with leptos-sync-core CRDTs.

## Overview

The collaborative demos showcase different CRDT (Conflict-free Replicated Data Type) implementations in real-world applications:

1. **Text Editor Demo** - Uses RGA (Replicated Growable Array) for collaborative text editing
2. **Task Manager Demo** - Uses LSEQ (Logoot Sequence) for ordered task management
3. **Document Editor Demo** - Uses Yjs Tree for hierarchical document editing
4. **Project Manager Demo** - Uses DAG (Directed Acyclic Graph) for project task dependencies

## Demo Applications

### 1. Text Editor Demo (RGA)

**Location**: `examples/text_editor_demo/`

**CRDT Type**: RGA (Replicated Growable Array)

**Purpose**: Demonstrates collaborative text editing with real-time synchronization.

#### Features
- Real-time character insertion and deletion
- Conflict-free merging of concurrent edits
- Position-based character ordering
- Live collaboration between multiple users

#### Usage
```bash
cd examples/text_editor_demo
trunk serve
```

**Access**: http://localhost:3000/

#### API Example
```rust
use leptos_sync_core::crdt::advanced::Rga;
use leptos_sync_core::crdt::{Mergeable, ReplicaId};
use uuid::Uuid;

let replica = ReplicaId::from(Uuid::new_v4());
let mut rga = Rga::new(replica);

// Insert characters
let pos1 = rga.insert_after('H', None)?;
let pos2 = rga.insert_after('i', Some(pos1))?;

// Get content
let content: Vec<char> = rga.to_vec();
```

### 2. Task Manager Demo (LSEQ)

**Location**: `examples/task_manager_demo/`

**CRDT Type**: LSEQ (Logoot Sequence)

**Purpose**: Demonstrates collaborative task management with ordered sequences.

#### Features
- Ordered task lists
- Task creation, updating, and deletion
- Priority and status management
- Conflict-free task ordering

#### Usage
```bash
cd examples/task_manager_demo
trunk serve
```

**Access**: http://localhost:3001/

#### API Example
```rust
use leptos_sync_core::crdt::advanced::Lseq;
use leptos_sync_core::crdt::{Mergeable, ReplicaId};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Task {
    id: Uuid,
    title: String,
    completed: bool,
}

let replica = ReplicaId::from(Uuid::new_v4());
let mut lseq = Lseq::new(replica);

// Add tasks
let task = Task {
    id: Uuid::new_v4(),
    title: "Buy groceries".to_string(),
    completed: false,
};
let pos = lseq.insert(task, None)?;

// Get all tasks
let tasks: Vec<Task> = lseq.to_vec();
```

### 3. Document Editor Demo (Yjs Tree)

**Location**: `examples/document_editor_demo/`

**CRDT Type**: Yjs Tree

**Purpose**: Demonstrates hierarchical document editing with tree structures.

#### Features
- Hierarchical document structure
- Node creation, updating, and deletion
- Tree-based content organization
- Support for different node types (sections, paragraphs, headings, lists, code blocks)

#### Usage
```bash
cd examples/document_editor_demo
trunk serve
```

**Access**: http://localhost:8082/

#### API Example
```rust
use leptos_sync_core::crdt::advanced::YjsTree;
use leptos_sync_core::crdt::{Mergeable, ReplicaId};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DocumentNode {
    id: Uuid,
    title: String,
    content: String,
    node_type: NodeType,
}

let replica = ReplicaId::from(Uuid::new_v4());
let mut tree = YjsTree::new(replica);

// Add root node
let node = DocumentNode {
    id: Uuid::new_v4(),
    title: "Introduction".to_string(),
    content: "".to_string(),
    node_type: NodeType::Section,
};
let pos = tree.add_root(node)?;
```

### 4. Project Manager Demo (DAG)

**Location**: `examples/project_manager_demo/`

**CRDT Type**: DAG (Directed Acyclic Graph)

**Purpose**: Demonstrates project management with task dependencies and relationships.

#### Features
- Task dependency management
- Project task organization
- Priority and status tracking
- Dependency graph visualization
- Conflict-free dependency resolution

#### Usage
```bash
cd examples/project_manager_demo
trunk serve
```

**Access**: http://localhost:8083/

#### API Example
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
}

let replica = ReplicaId::from(Uuid::new_v4());
let mut graph = AddWinsGraph::new(replica);

// Add tasks
let task = Task {
    id: Uuid::new_v4(),
    title: "Setup project".to_string(),
    description: "Initialize the project structure".to_string(),
    status: TaskStatus::NotStarted,
    priority: TaskPriority::High,
};
let vertex = graph.add_vertex(task, 1000);

// Add dependencies
let edge = graph.add_edge(&vertex1, &vertex2, 2000, Some(1.0))?;
```

## Running All Demos

To run all demos simultaneously:

```bash
# Terminal 1 - Text Editor
cd examples/text_editor_demo && trunk serve

# Terminal 2 - Task Manager  
cd examples/task_manager_demo && trunk serve

# Terminal 3 - Document Editor
cd examples/document_editor_demo && trunk serve

# Terminal 4 - Project Manager
cd examples/project_manager_demo && trunk serve
```

## Demo URLs

- **Text Editor**: http://localhost:3000/
- **Task Manager**: http://localhost:3001/
- **Document Editor**: http://localhost:8082/
- **Project Manager**: http://localhost:8083/

## Architecture

### CRDT Selection Rationale

Each demo uses a specific CRDT type optimized for its use case:

1. **RGA for Text Editing**: Provides efficient character-level operations with conflict-free merging
2. **LSEQ for Task Lists**: Maintains ordered sequences with efficient insertion and deletion
3. **Yjs Tree for Documents**: Supports hierarchical structures with parent-child relationships
4. **DAG for Project Management**: Handles complex dependencies and relationships between tasks

### Data Flow

```
User Input → Demo Application → CRDT Operations → State Update → UI Refresh
                ↓
            Merge Operations ← Network Sync ← Other Replicas
```

### Synchronization

All demos support:
- Real-time synchronization between multiple users
- Conflict-free merging of concurrent operations
- Eventual consistency guarantees
- Offline-capable operations

## Testing

### Integration Tests

Comprehensive integration tests verify CRDT functionality:

```bash
cargo test -p integration_demos
```

### Demo-Specific Tests

Each demo includes unit tests:

```bash
# Text Editor tests
cargo test -p text_editor_demo

# Task Manager tests  
cargo test -p task_manager_demo

# Document Editor tests
cargo test -p document_editor_demo

# Project Manager tests
cargo test -p project_manager_demo
```

## Performance Characteristics

### RGA (Text Editor)
- **Insertion**: O(log n) average case
- **Deletion**: O(log n) average case
- **Merge**: O(n) where n is the number of operations

### LSEQ (Task Manager)
- **Insertion**: O(log n) average case
- **Deletion**: O(log n) average case
- **Merge**: O(n) where n is the number of operations

### Yjs Tree (Document Editor)
- **Node Operations**: O(log n) average case
- **Tree Traversal**: O(n) where n is the number of nodes
- **Merge**: O(n) where n is the number of nodes

### DAG (Project Manager)
- **Vertex Operations**: O(1) average case
- **Edge Operations**: O(1) average case
- **Merge**: O(n) where n is the number of vertices/edges

## Best Practices

### For Text Editing
- Use RGA for character-level operations
- Implement proper cursor position management
- Handle large documents with pagination

### For Task Management
- Use LSEQ for ordered task lists
- Implement proper task state transitions
- Handle task dependencies carefully

### For Document Editing
- Use Yjs Tree for hierarchical content
- Implement proper node type validation
- Handle large document trees efficiently

### For Project Management
- Use DAG for task dependencies
- Implement cycle detection
- Handle complex dependency graphs

## Troubleshooting

### Common Issues

1. **Port Conflicts**: Ensure each demo runs on a different port
2. **Build Errors**: Run `cargo clean` and rebuild
3. **Sync Issues**: Check network connectivity and replica IDs
4. **Performance**: Monitor memory usage for large datasets

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug trunk serve
```

## Contributing

When adding new demos:

1. Choose the appropriate CRDT type
2. Implement comprehensive tests
3. Add documentation
4. Update this guide
5. Ensure integration tests pass

## Future Enhancements

Planned improvements:

- Real-time collaboration between demos
- Advanced conflict resolution strategies
- Performance optimizations
- Mobile-responsive interfaces
- Accessibility improvements
- Local storage persistence
- Advanced visualization features
