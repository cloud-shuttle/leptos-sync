# Text Editor Demo - RGA CRDT

A collaborative text editor demonstrating the RGA (Replicated Growable Array) CRDT implementation.

## Overview

This demo showcases real-time collaborative text editing using the RGA CRDT, which provides conflict-free merging of concurrent text operations.

## Features

- **Real-time Collaboration**: Multiple users can edit text simultaneously
- **Conflict-free Merging**: Concurrent edits are automatically resolved
- **Character-level Operations**: Insert and delete individual characters
- **Position-based Ordering**: Characters maintain consistent ordering across replicas

## Running the Demo

```bash
cd examples/text_editor_demo
trunk serve
```

Access the demo at: http://localhost:3000/

## Architecture

### CRDT Type: RGA (Replicated Growable Array)

RGA is optimized for:
- Character-level text operations
- Efficient insertion and deletion
- Conflict-free merging of concurrent edits
- Maintaining text ordering across replicas

### Key Components

- **TextEditor**: Main application wrapper around RGA
- **RGA**: Core CRDT implementation for text operations
- **Leptos UI**: Reactive web interface

## API Usage

```rust
use leptos_sync_core::crdt::advanced::Rga;
use leptos_sync_core::crdt::{Mergeable, ReplicaId};
use uuid::Uuid;

// Create a new RGA instance
let replica = ReplicaId::from(Uuid::new_v4());
let mut rga = Rga::new(replica);

// Insert characters
let pos1 = rga.insert_after('H', None)?;
let pos2 = rga.insert_after('e', Some(pos1))?;
let pos3 = rga.insert_after('l', Some(pos2))?;
let pos4 = rga.insert_after('l', Some(pos3))?;
let pos5 = rga.insert_after('o', Some(pos4))?;

// Get the text content
let content: Vec<char> = rga.to_vec();
let text: String = content.into_iter().collect();
// text = "Hello"

// Delete a character
rga.delete(&pos3)?;

// Merge with another RGA
let mut other_rga = Rga::new(ReplicaId::from(Uuid::new_v4()));
other_rga.insert_after('!', None)?;
rga.merge(&other_rga)?;
```

## Testing

Run the demo tests:

```bash
cargo test -p text_editor_demo
```

## Performance

- **Insertion**: O(log n) average case
- **Deletion**: O(log n) average case  
- **Merge**: O(n) where n is the number of operations
- **Memory**: O(n) where n is the number of characters

## Use Cases

- Collaborative text editors
- Real-time document editing
- Chat applications
- Code collaboration tools
- Note-taking applications

## Limitations

- Character-level operations only (no word/line operations)
- Memory usage grows with document size
- No built-in formatting support

## Future Enhancements

- Word-level operations
- Rich text formatting
- Cursor position synchronization
- Undo/redo functionality
- Large document optimization
