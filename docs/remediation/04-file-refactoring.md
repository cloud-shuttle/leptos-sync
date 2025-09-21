# File Size Refactoring - High Priority

## Overview
Break down oversized files (>300 lines) into smaller, focused modules for better maintainability and LLM comprehension.

## Files Requiring Refactoring

### Critical Priority (>1000 lines)
1. `leptos-sync-core/src/crdt/graph.rs` (1,160 lines)
2. `leptos-sync-core/src/crdt/advanced.rs` (1,123 lines)

### High Priority (800-1000 lines)  
3. `leptos-sync-core/src/reliability/monitoring.rs` (977 lines)
4. `leptos-sync-core/src/reliability/data_integrity.rs` (895 lines)
5. `leptos-sync-core/src/reliability/error_recovery.rs` (838 lines)
6. `leptos-sync-core/src/crdt/tree.rs` (814 lines)
7. `leptos-sync-core/src/security/authentication.rs` (810 lines)
8. `leptos-sync-core/src/crdt/crdt_basic.rs` (799 lines)

### Medium Priority (600-800 lines)
9. `leptos-sync-core/src/devtools/mod.rs` (770 lines)
10. `leptos-sync-core/src/crdt/list.rs` (724 lines)
... (40+ additional files)

## Refactoring Strategy

### 1. CRDT Graph Module (1,160 lines → 6 files < 250 lines each)

**Current Issues:**
- Vertex and Edge logic mixed together
- Add/Remove wins strategies in same file
- Test code mixed with implementation
- Multiple graph algorithms in one place

**Proposed Structure:**
```
crdt/graph/
├── vertex.rs         (< 200 lines) - Vertex operations
├── edge.rs           (< 200 lines) - Edge operations  
├── add_wins.rs       (< 250 lines) - Add-wins strategy
├── remove_wins.rs    (< 250 lines) - Remove-wins strategy
├── algorithms.rs     (< 200 lines) - Graph algorithms
└── mod.rs           (< 100 lines) - Public API + tests
```

### 2. Advanced CRDTs Module (1,123 lines → 5 files < 250 lines each)

**Current Issues:**  
- RGA, LSEQ, Yjs Tree, and DAG all in one file
- Different CRDT types have different concerns
- Complex position tracking mixed with data operations

**Proposed Structure:**
```
crdt/advanced/
├── rga.rs           (< 250 lines) - Replicated Growable Array
├── lseq.rs          (< 250 lines) - LSEQ implementation
├── yjs_tree.rs      (< 250 lines) - Yjs Tree CRDT
├── dag.rs           (< 250 lines) - Directed Acyclic Graph
└── mod.rs          (< 100 lines) - Public API + common types
```

### 3. Reliability Monitoring (977 lines → 4 files < 300 lines each)

**Current Issues:**
- Metrics collection, alerting, and health checks mixed
- Different monitoring concerns not separated
- Large test suites in same file

**Proposed Structure:**
```
reliability/monitoring/
├── metrics.rs       (< 250 lines) - Metrics collection
├── alerts.rs        (< 200 lines) - Alert management  
├── health.rs        (< 200 lines) - Health reporting
├── config.rs        (< 150 lines) - Configuration types
└── mod.rs          (< 100 lines) - Public API
```

## Refactoring Process Template

### Step 1: Analyze Current File Structure
- Identify logical boundaries (structs, traits, implementations)
- Map dependencies between components
- Identify shared types and utilities
- Note test coverage for each component

### Step 2: Plan Module Boundaries  
- Group related functionality together
- Minimize cross-module dependencies
- Keep public API surface small
- Ensure each module has single responsibility

### Step 3: Create New Module Structure
- Start with shared types and utilities
- Move independent components first
- Update imports and re-exports
- Maintain public API compatibility

### Step 4: Validate Refactoring
- All tests continue to pass
- No performance regression
- Documentation still accurate
- API consumers unaffected

## Example: CRDT Basic Refactoring (799 lines → 4 files)

### Before (crdt_basic.rs - 799 lines)
```rust
// Everything mixed together:
// - ReplicaId definition
// - LwwRegister implementation + tests
// - LwwMap implementation + tests  
// - GCounter implementation + tests
// - Common utilities
```

### After Structure
```
crdt/basic/
├── replica_id.rs    (< 100 lines) - ReplicaId type + utilities
├── lww_register.rs  (< 200 lines) - LWW Register + tests
├── lww_map.rs       (< 250 lines) - LWW Map + tests
├── counter.rs       (< 200 lines) - GCounter + tests
└── mod.rs          (< 50 lines)  - Public API
```

### Implementation Plan
```rust
// replica_id.rs
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReplicaId(pub Uuid);

impl ReplicaId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

// lww_register.rs  
use super::replica_id::ReplicaId;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LwwRegister<T> {
    value: T,
    timestamp: SystemTime,
    replica_id: ReplicaId,
}

impl<T> LwwRegister<T> {
    // Implementation...
}

#[cfg(test)]
mod tests {
    // Tests specific to LwwRegister
}

// mod.rs
pub mod replica_id;
pub mod lww_register;
pub mod lww_map;
pub mod counter;

pub use replica_id::ReplicaId;
pub use lww_register::LwwRegister;
pub use lww_map::LwwMap;
pub use counter::GCounter;
```

## Testing Strategy During Refactoring

### Test Preservation
- Move tests with their related code
- Maintain all existing test coverage
- Add integration tests to verify module boundaries
- Ensure no test duplication or gaps

### Regression Prevention
- Run full test suite after each module split
- Use feature flags to enable gradual migration
- Keep old file as deprecated during transition
- Document any API changes clearly

## Automation Support

### Refactoring Tools
```bash
# Script to analyze file structure and suggest splits
cargo expand src/crdt/graph.rs | grep "impl\|struct\|enum" > analysis.txt

# Script to validate line counts after refactoring  
find . -name "*.rs" -exec wc -l {} + | awk '$1 > 300 {print $1 " " $2}'

# Script to run subset of tests during refactoring
cargo test --package leptos-sync-core crdt::graph --
```

## Acceptance Criteria

### Per File
- [ ] No file exceeds 300 lines of code
- [ ] Each module has single, clear responsibility  
- [ ] Public API remains unchanged (or properly deprecated)
- [ ] All tests continue to pass
- [ ] Documentation updated to reflect new structure

### Overall
- [ ] Total codebase complexity reduced
- [ ] Module dependencies are clear and minimal
- [ ] Import statements cleaned up
- [ ] No circular dependencies introduced
- [ ] Performance maintained or improved

## Time Estimate: 3-4 weeks (can be parallelized)

### Priority Order
1. Week 1: Critical files (graph.rs, advanced.rs) 
2. Week 2: High priority reliability modules
3. Week 3: Security and authentication modules
4. Week 4: Remaining medium priority files + validation

## Dependencies: Compilation fixes (01)
## Risk: Medium - potential for introducing bugs during large refactors
## Mitigation: Incremental approach with continuous testing
