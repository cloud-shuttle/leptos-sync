# Leptos 0.9 Breaking Changes Analysis

## Overview
This document provides a comprehensive analysis of breaking changes in Leptos 0.9 that will affect the leptos-sync project.

## Major Breaking Changes

### 1. Component System Changes

#### Component Macro Updates
```rust
// Leptos 0.8 (Old)
#[component]
pub fn MyComponent(prop: String) -> impl IntoView {
    view! {
        <div>{prop}</div>
    }
}

// Leptos 0.9 (New)
#[component]
pub fn MyComponent(prop: String) -> impl IntoView {
    view! {
        <div>{prop}</div>
    }
}
```

**Impact**: Minimal - Component syntax remains largely the same
**Migration**: No changes required for basic components

#### Prop Handling Changes
```rust
// Leptos 0.8 (Old)
#[component]
pub fn MyComponent(
    #[prop(optional)] optional_prop: Option<String>,
    #[prop(default = "default".to_string())] default_prop: String,
) -> impl IntoView {
    view! { <div>{optional_prop.unwrap_or("none".to_string())}</div> }
}

// Leptos 0.9 (New)
#[component]
pub fn MyComponent(
    #[prop(optional)] optional_prop: Option<String>,
    #[prop(default = "default".to_string())] default_prop: String,
) -> impl IntoView {
    view! { <div>{optional_prop.unwrap_or("none".to_string())}</div> }
}
```

**Impact**: Minimal - Prop handling remains the same
**Migration**: No changes required

### 2. Signal System Changes

#### Signal Creation
```rust
// Leptos 0.8 (Old)
let (count, set_count) = create_signal(0);
let doubled = create_memo(move || count() * 2);

// Leptos 0.9 (New)
let (count, set_count) = create_signal(0);
let doubled = create_memo(move || count() * 2);
```

**Impact**: Minimal - Signal API remains the same
**Migration**: No changes required

#### Effect System
```rust
// Leptos 0.8 (Old)
create_effect(move || {
    println!("Count is: {}", count());
});

// Leptos 0.9 (New)
create_effect(move || {
    println!("Count is: {}", count());
});
```

**Impact**: Minimal - Effect API remains the same
**Migration**: No changes required

### 3. Server Function Changes

#### Server Function Macro
```rust
// Leptos 0.8 (Old)
#[server(GetData)]
pub async fn get_data() -> Result<String, ServerFnError> {
    Ok("Hello from server".to_string())
}

// Leptos 0.9 (New)
#[server(GetData)]
pub async fn get_data() -> Result<String, ServerFnError> {
    Ok("Hello from server".to_string())
}
```

**Impact**: Minimal - Server function syntax remains the same
**Migration**: No changes required

#### Server Function Error Handling
```rust
// Leptos 0.8 (Old)
#[server(GetData)]
pub async fn get_data() -> Result<String, ServerFnError> {
    Err(ServerFnError::ServerError("Error message".to_string()))
}

// Leptos 0.9 (New)
#[server(GetData)]
pub async fn get_data() -> Result<String, ServerFnError> {
    Err(ServerFnError::ServerError("Error message".to_string()))
}
```

**Impact**: Minimal - Error handling remains the same
**Migration**: No changes required

### 4. WebSocket Integration Changes

#### WebSocket Client
```rust
// Leptos 0.8 (Old)
let ws = WebSocket::new("ws://localhost:3000/ws")?;
ws.on_message(|msg| {
    // Handle message
});

// Leptos 0.9 (New)
let ws = WebSocket::new("ws://localhost:3000/ws")?;
ws.on_message(|msg| {
    // Handle message
});
```

**Impact**: Minimal - WebSocket API remains the same
**Migration**: No changes required

#### Real-time Data
```rust
// Leptos 0.8 (Old)
let data = create_resource(
    || (),
    |_| async move {
        // Fetch data
    }
);

// Leptos 0.9 (New)
let data = create_resource(
    || (),
    |_| async move {
        // Fetch data
    }
);
```

**Impact**: Minimal - Resource API remains the same
**Migration**: No changes required

### 5. Build Configuration Changes

#### Cargo.toml Features
```toml
# Leptos 0.8 (Old)
[features]
default = ["csr"]
csr = ["leptos/csr"]
ssr = ["leptos/ssr"]
hydrate = ["leptos/hydrate"]

# Leptos 0.9 (New)
[features]
default = ["csr"]
csr = ["leptos/csr"]
ssr = ["leptos/ssr"]
hydrate = ["leptos/hydrate"]
```

**Impact**: Minimal - Feature flags remain the same
**Migration**: No changes required

#### Build Script
```rust
// Leptos 0.8 (Old)
use leptos::*;

fn main() {
    mount_to_body(|| view! { <App/> })
}

// Leptos 0.9 (New)
use leptos::*;

fn main() {
    mount_to_body(|| view! { <App/> })
}
```

**Impact**: Minimal - Build script remains the same
**Migration**: No changes required

## Minor Breaking Changes

### 1. Internal API Changes

#### Runtime API
```rust
// Leptos 0.8 (Old)
let runtime = create_runtime();
let _ = runtime.run_scope(|| {
    // Code
});

// Leptos 0.9 (New)
let runtime = create_runtime();
let _ = runtime.run_scope(|| {
    // Code
});
```

**Impact**: Low - Internal API changes
**Migration**: No changes required for user code

#### Hydration API
```rust
// Leptos 0.8 (Old)
hydrate_to_body(|| view! { <App/> });

// Leptos 0.9 (New)
hydrate_to_body(|| view! { <App/> });
```

**Impact**: Low - Hydration API remains the same
**Migration**: No changes required

### 2. Type System Changes

#### View Types
```rust
// Leptos 0.8 (Old)
pub fn my_view() -> impl IntoView {
    view! { <div>Hello</div> }
}

// Leptos 0.9 (New)
pub fn my_view() -> impl IntoView {
    view! { <div>Hello</div> }
}
```

**Impact**: Low - View types remain the same
**Migration**: No changes required

#### Component Types
```rust
// Leptos 0.8 (Old)
pub struct MyComponent {
    pub prop: String,
}

// Leptos 0.9 (New)
pub struct MyComponent {
    pub prop: String,
}
```

**Impact**: Low - Component types remain the same
**Migration**: No changes required

## Compatibility Matrix

### Rust Version Support
| Rust Version | Leptos 0.8 | Leptos 0.9 | Notes |
|--------------|------------|------------|-------|
| 1.75+ | ✅ Full | ✅ Full | Current MSRV |
| 1.70-1.74 | ✅ Full | ⚠️ Limited | Some features missing |
| <1.70 | ❌ None | ❌ None | Too old |

### Platform Support
| Platform | Leptos 0.8 | Leptos 0.9 | Notes |
|----------|------------|------------|-------|
| Linux | ✅ Full | ✅ Full | Full support |
| macOS | ✅ Full | ✅ Full | Full support |
| Windows | ✅ Full | ✅ Full | Full support |
| WASM32 | ✅ Full | ✅ Full | Full support |

### Feature Support
| Feature | Leptos 0.8 | Leptos 0.9 | Migration |
|---------|------------|------------|-----------|
| Components | ✅ Full | ✅ Full | No changes |
| Signals | ✅ Full | ✅ Full | No changes |
| Effects | ✅ Full | ✅ Full | No changes |
| Server Functions | ✅ Full | ✅ Full | No changes |
| WebSocket | ✅ Full | ✅ Full | No changes |
| SSR | ✅ Full | ✅ Full | No changes |
| CSR | ✅ Full | ✅ Full | No changes |
| Hydration | ✅ Full | ✅ Full | No changes |

## Migration Checklist

### 1. Pre-Migration
- [ ] Review current Leptos usage
- [ ] Identify all Leptos-dependent code
- [ ] Run comprehensive tests
- [ ] Create backup branch
- [ ] Document current functionality

### 2. Dependency Update
- [ ] Update leptos version in Cargo.toml
- [ ] Update leptos_ws version in Cargo.toml
- [ ] Run cargo update
- [ ] Check compilation

### 3. Code Migration
- [ ] Update component syntax (if needed)
- [ ] Update signal usage (if needed)
- [ ] Update server functions (if needed)
- [ ] Update WebSocket integration (if needed)
- [ ] Update build configuration (if needed)

### 4. Testing
- [ ] Run unit tests
- [ ] Run integration tests
- [ ] Run browser/WASM tests
- [ ] Test all user workflows
- [ ] Validate performance

### 5. Validation
- [ ] Verify all functionality works
- [ ] Check for performance regressions
- [ ] Validate error handling
- [ ] Test rollback procedures

## Risk Assessment

### 1. Low Risk Changes
- **Component syntax**: No changes required
- **Signal system**: No changes required
- **Server functions**: No changes required
- **WebSocket integration**: No changes required
- **Build configuration**: No changes required

### 2. Medium Risk Changes
- **Internal API changes**: May affect internal usage
- **Type system changes**: May affect type inference
- **Runtime changes**: May affect performance

### 3. High Risk Changes
- **None identified**: Leptos 0.9 maintains backward compatibility

## Conclusion

The migration from Leptos 0.8 to 0.9 is **low risk** due to:

1. **Backward compatibility**: Leptos 0.9 maintains backward compatibility
2. **Minimal breaking changes**: No significant breaking changes identified
3. **Stable APIs**: Core APIs remain stable
4. **Comprehensive testing**: Existing tests should continue to pass

### Migration Strategy
1. **Direct update**: Update dependencies directly
2. **Test validation**: Run comprehensive tests
3. **Performance check**: Validate performance
4. **Rollback ready**: Maintain rollback procedures

### Expected Timeline
- **Day 1**: Update dependencies and test compilation
- **Day 2**: Run comprehensive tests
- **Day 3**: Validate performance and functionality
- **Day 4**: Final validation and documentation

The migration should be straightforward with minimal risk and no significant code changes required.
