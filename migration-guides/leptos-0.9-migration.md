# Leptos 0.8 → 0.9 Migration Guide

## Overview
This guide covers the migration from Leptos 0.8 to 0.9, including breaking changes and required code updates.

## Breaking Changes

### 1. Component API Changes

#### Before (Leptos 0.8)
```rust
use leptos::*;

#[component]
pub fn MyComponent(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);
    
    view! { cx,
        <div>
            <button on:click=move |_| set_count.update(|n| *n += 1)>
                "Count: " {count}
            </button>
        </div>
    }
}
```

#### After (Leptos 0.9)
```rust
use leptos::*;

#[component]
pub fn MyComponent() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    
    view! {
        <div>
            <button on:click=move |_| set_count.update(|n| *n += 1)>
                "Count: " {count}
            </button>
        </div>
    }
}
```

#### Key Changes:
- **Removed `cx: Scope` parameter** from component functions
- **Removed `cx` parameter** from `create_signal` and other reactive functions
- **Removed `cx` parameter** from `view!` macro

### 2. Server Function Updates

#### Before (Leptos 0.8)
```rust
use leptos::*;

#[server(UpdateCollection, "/api")]
pub async fn update_collection(
    cx: Scope,
    collection_id: String,
    data: String
) -> Result<(), ServerFnError> {
    // Implementation
    Ok(())
}
```

#### After (Leptos 0.9)
```rust
use leptos::*;

#[server(UpdateCollection, "/api")]
pub async fn update_collection(
    collection_id: String,
    data: String
) -> Result<(), ServerFnError> {
    // Implementation
    Ok(())
}
```

#### Key Changes:
- **Removed `cx: Scope` parameter** from server functions
- **Simplified function signatures** for better ergonomics

### 3. Reactive System Updates

#### Before (Leptos 0.8)
```rust
use leptos::*;

#[component]
pub fn ReactiveComponent(cx: Scope) -> impl IntoView {
    let (value, set_value) = create_signal(cx, 0);
    let doubled = create_memo(cx, move |_| value() * 2);
    
    view! { cx,
        <div>
            <p>"Value: " {value}</p>
            <p>"Doubled: " {doubled}</p>
        </div>
    }
}
```

#### After (Leptos 0.9)
```rust
use leptos::*;

#[component]
pub fn ReactiveComponent() -> impl IntoView {
    let (value, set_value) = create_signal(0);
    let doubled = create_memo(move |_| value() * 2);
    
    view! {
        <div>
            <p>"Value: " {value}</p>
            <p>"Doubled: " {doubled}</p>
        </div>
    }
}
```

#### Key Changes:
- **Removed `cx` parameter** from `create_memo`, `create_effect`, etc.
- **Simplified reactive function calls**

### 4. WebSocket API Updates

#### Before (Leptos 0.8)
```rust
use leptos::*;
use leptos_ws::*;

#[component]
pub fn WebSocketComponent(cx: Scope) -> impl IntoView {
    let (ws, set_ws) = create_signal(cx, None::<WebSocket>);
    
    let connect = move |_| {
        let ws = WebSocket::new("ws://localhost:3001").unwrap();
        set_ws.set(Some(ws));
    };
    
    view! { cx,
        <button on:click=connect>
            "Connect"
        </button>
    }
}
```

#### After (Leptos 0.9)
```rust
use leptos::*;
use leptos_ws::*;

#[component]
pub fn WebSocketComponent() -> impl IntoView {
    let (ws, set_ws) = create_signal(None::<WebSocket>);
    
    let connect = move |_| {
        let ws = WebSocket::new("ws://localhost:3001").unwrap();
        set_ws.set(Some(ws));
    };
    
    view! {
        <button on:click=connect>
            "Connect"
        </button>
    }
}
```

#### Key Changes:
- **Removed `cx` parameter** from WebSocket components
- **Simplified WebSocket integration**

### 5. SSR/CSR Configuration Updates

#### Before (Leptos 0.8)
```rust
use leptos::*;

#[cfg(feature = "ssr")]
pub fn main() {
    leptos::mount_to_body(|cx| {
        view! { cx, <App/> }
    });
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    leptos::mount_to_body(|cx| {
        view! { cx, <App/> }
    });
}
```

#### After (Leptos 0.9)
```rust
use leptos::*;

pub fn main() {
    leptos::mount_to_body(|| {
        view! { <App/> }
    });
}
```

#### Key Changes:
- **Simplified mounting** without `cx` parameter
- **Unified SSR/CSR** mounting API

## Migration Steps

### Step 1: Update Dependencies
```toml
# Cargo.toml
[dependencies]
leptos = "0.9.0"
leptos_ws = "0.9.0"
leptos-ws-pro = "0.11.0"
```

### Step 2: Update Component Functions
1. Remove `cx: Scope` parameter from all component functions
2. Remove `cx` parameter from all reactive function calls
3. Remove `cx` parameter from all `view!` macro calls

### Step 3: Update Server Functions
1. Remove `cx: Scope` parameter from all server functions
2. Update function signatures accordingly

### Step 4: Update Reactive System
1. Remove `cx` parameter from `create_signal`, `create_memo`, etc.
2. Update all reactive function calls

### Step 5: Update WebSocket Integration
1. Remove `cx` parameter from WebSocket components
2. Update WebSocket event handlers

### Step 6: Update SSR/CSR Configuration
1. Simplify mounting functions
2. Remove `cx` parameter from mount calls

## Automated Migration Script

### Migration Script for Components
```bash
#!/bin/bash
# migrate-components.sh

echo "Migrating Leptos components from 0.8 to 0.9..."

# Remove cx: Scope parameter from component functions
find . -name "*.rs" -exec sed -i 's/#\[component\]\s*$/#[component]/g' {} \;
find . -name "*.rs" -exec sed -i 's/pub fn \([^(]*\)(cx: Scope)/pub fn \1()/g' {} \;

# Remove cx parameter from create_signal calls
find . -name "*.rs" -exec sed -i 's/create_signal(cx, /create_signal(/g' {} \;

# Remove cx parameter from view! macro calls
find . -name "*.rs" -exec sed -i 's/view! { cx,/view! {/g' {} \;

echo "Component migration complete. Please review changes and run tests."
```

### Migration Script for Server Functions
```bash
#!/bin/bash
# migrate-server-functions.sh

echo "Migrating Leptos server functions from 0.8 to 0.9..."

# Remove cx: Scope parameter from server functions
find . -name "*.rs" -exec sed -i 's/pub async fn \([^(]*\)(cx: Scope, /pub async fn \1(/g' {} \;

echo "Server function migration complete. Please review changes and run tests."
```

## Testing After Migration

### 1. Compilation Check
```bash
cargo check --workspace
```

### 2. Unit Tests
```bash
cargo test --workspace --lib
```

### 3. Integration Tests
```bash
cargo test --test integration
```

### 4. Browser Tests
```bash
wasm-pack test --chrome --headless
```

### 5. Performance Tests
```bash
cargo bench --bench sync_performance
```

## Common Issues and Solutions

### Issue 1: Missing `cx` Parameter
**Error**: `error[E0061]: this function takes 1 argument but 0 were supplied`

**Solution**: Remove the `cx` parameter from the function call.

### Issue 2: Scope Not Found
**Error**: `error[E0425]: cannot find value `cx` in this scope`

**Solution**: Remove all references to `cx` in the component.

### Issue 3: View Macro Issues
**Error**: `error[E0425]: cannot find value `cx` in this scope`

**Solution**: Remove `cx` parameter from `view!` macro calls.

## Rollback Plan

If migration causes issues:

1. **Revert Cargo.toml changes**:
   ```bash
   git checkout HEAD~1 -- Cargo.toml
   ```

2. **Revert code changes**:
   ```bash
   git checkout HEAD~1 -- src/
   ```

3. **Restore working state**:
   ```bash
   cargo check --workspace
   cargo test --workspace
   ```

## Benefits of Migration

### Performance Improvements
- **Reduced overhead** from scope parameter passing
- **Simplified reactive system** with better performance
- **Optimized WebSocket** integration

### Developer Experience
- **Cleaner component syntax** without scope parameters
- **Simplified server function** definitions
- **Better error messages** and debugging

### Future Compatibility
- **Access to latest features** in Leptos 0.9+
- **Better integration** with modern Rust patterns
- **Improved ecosystem** compatibility

## Timeline

- **Week 1**: Update dependencies and run migration scripts
- **Week 2**: Manual review and testing
- **Week 3**: Performance testing and optimization
- **Week 4**: Documentation updates and release

## Support

For migration issues:
1. Check the [Leptos 0.9 changelog](https://github.com/leptos-rs/leptos/releases)
2. Review the [migration guide](https://leptos.dev/guide/migration)
3. Ask questions in the [Leptos Discord](https://discord.gg/leptos)
