# Document Editor Demo - Yjs Tree CRDT

A collaborative document editor demonstrating the Yjs Tree CRDT implementation.

## Overview

This demo showcases hierarchical document editing using the Yjs Tree CRDT, which provides tree-based data structures with conflict-free merging of concurrent operations.

## Features

- **Hierarchical Structure**: Organize content in tree-like structures
- **Multiple Node Types**: Sections, paragraphs, headings, lists, and code blocks
- **Node Management**: Create, update, and delete document nodes
- **Tree Navigation**: Navigate through document hierarchy
- **Collaborative Editing**: Multiple users can edit documents simultaneously

## Running the Demo

```bash
cd examples/document_editor_demo
trunk serve
```

Access the demo at: http://localhost:8082/

## Architecture

### CRDT Type: Yjs Tree

Yjs Tree is optimized for:
- Hierarchical data structures
- Tree-based operations
- Conflict-free merging of concurrent edits
- Maintaining tree structure across replicas

### Key Components

- **DocumentEditor**: Main application wrapper around Yjs Tree
- **DocumentNode**: Document node data structure
- **YjsTree**: Core CRDT implementation for tree structures
- **Leptos UI**: Reactive web interface

## API Usage

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
    items: Vec<String>,
    language: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NodeType {
    Section,
    Paragraph,
    Heading,
    List,
    CodeBlock,
}

// Create a new Yjs Tree instance
let replica = ReplicaId::from(Uuid::new_v4());
let mut tree = YjsTree::new(replica);

// Add root nodes
let section = DocumentNode {
    id: Uuid::new_v4(),
    title: "Introduction".to_string(),
    content: "".to_string(),
    node_type: NodeType::Section,
    items: vec![],
    language: None,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
};
let pos1 = tree.add_root(section)?;

// Add child nodes
let paragraph = DocumentNode {
    id: Uuid::new_v4(),
    title: "".to_string(),
    content: "This is the introduction paragraph.".to_string(),
    node_type: NodeType::Paragraph,
    items: vec![],
    language: None,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
};
let pos2 = tree.insert_after(paragraph, Some(pos1))?;

// Add a code block
let code_block = DocumentNode {
    id: Uuid::new_v4(),
    title: "".to_string(),
    content: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
    node_type: NodeType::CodeBlock,
    items: vec![],
    language: Some("rust".to_string()),
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
};
let pos3 = tree.insert_after(code_block, Some(pos2))?;

// Merge with another tree
let mut other_tree = YjsTree::new(ReplicaId::from(Uuid::new_v4()));
let other_node = DocumentNode { /* ... */ };
other_tree.add_root(other_node)?;
tree.merge(&other_tree)?;
```

## Testing

Run the demo tests:

```bash
cargo test -p document_editor_demo
```

## Performance

- **Node Operations**: O(log n) average case
- **Tree Traversal**: O(n) where n is the number of nodes
- **Merge**: O(n) where n is the number of nodes
- **Memory**: O(n) where n is the number of nodes

## Node Types

### Section
- Container for other nodes
- Used for organizing content hierarchically
- Can contain multiple child nodes

### Paragraph
- Text content nodes
- Used for regular text content
- Supports rich text formatting

### Heading
- Section headers
- Used for document structure
- Different heading levels (H1, H2, H3, etc.)

### List
- Ordered or unordered lists
- Contains list items
- Supports nested lists

### Code Block
- Code snippets
- Supports syntax highlighting
- Language-specific formatting

## Use Cases

- Collaborative document editing
- Technical documentation
- Knowledge management systems
- Content management systems
- Academic paper collaboration
- Wiki systems

## Limitations

- No built-in rich text formatting
- No image or media support
- No table support
- No comment system
- No version history

## Future Enhancements

- Rich text formatting (bold, italic, etc.)
- Image and media support
- Table support
- Comment system
- Version history and diff
- Export to various formats (PDF, HTML, etc.)
- Advanced search and indexing
- Template system
- Collaborative cursors
