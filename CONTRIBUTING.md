# Contributing to Leptos-Sync

Thank you for your interest in contributing to Leptos-Sync! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites

- Rust 1.75+
- Nightly Rust (for Leptos 0.8.x)
- Node.js 18+ with PNPM
- Nix (optional, for reproducible environment)

### Development Setup

1. **Fork and clone the repository**
   ```bash
   git clone https://github.com/YOUR_USERNAME/leptos-sync.git
   cd leptos-sync
   ```

2. **Install dependencies**
   ```bash
   pnpm install
   ```

3. **Setup Rust toolchain**
   ```bash
   rustup toolchain install nightly
   rustup default nightly
   ```

4. **Run tests to ensure everything works**
   ```bash
   cargo test
   ```

## 🏗️ Project Structure

```
leptos-sync/
├── leptos-sync-core/          # Core synchronization library
├── leptos-sync-macros/        # Procedural macros
├── leptos-sync-components/    # Leptos UI components
├── leptos-sync-examples/      # Example applications
├── docs/                      # Documentation
├── deployment/                # Production deployment configs
├── infrastructure/            # Infrastructure as code
└── tests/                     # End-to-end tests
```

## 🧪 Testing

### Running Tests

```bash
# All tests
cargo test

# Core library only
cargo test --package leptos-sync-core

# Specific modules
cargo test --package leptos-sync-core --lib sync::conflict
cargo test --package leptos-sync-core --lib sync::realtime

# With output
cargo test -- --nocapture

# End-to-end tests
pnpm test:e2e
```

### Test Guidelines

- **Unit Tests**: Test individual functions and methods
- **Integration Tests**: Test module interactions
- **Property Tests**: Use `proptest` for CRDT correctness
- **WASM Tests**: Ensure browser compatibility
- **Coverage**: Aim for >90% test coverage

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_basic_functionality() {
        // Test implementation
    }

    proptest! {
        #[test]
        fn test_crdt_properties(a: u64, b: u64) {
            // Property-based testing
        }
    }
}
```

## 📝 Code Style

### Rust Code Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Document all public APIs with doc comments

### Documentation

```rust
/// Creates a new collection with the specified name and storage.
///
/// # Arguments
///
/// * `name` - The name of the collection
/// * `storage` - The storage backend to use
/// * `transport` - The transport layer for synchronization
///
/// # Examples
///
/// ```
/// use leptos_sync_core::{LocalFirstCollection, HybridStorage, HybridTransport};
///
/// let collection = LocalFirstCollection::new(
///     "todos".to_string(),
///     HybridStorage::new(),
///     HybridTransport::new()
/// );
/// ```
pub fn new(name: String, storage: HybridStorage, transport: HybridTransport) -> Self {
    // Implementation
}
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new conflict resolution strategy
fix: resolve Send/Sync compatibility issue
docs: update architecture documentation
test: add property tests for CRDT operations
refactor: simplify storage abstraction
```

## 🔧 Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Your Changes

- Write code following the style guidelines
- Add tests for new functionality
- Update documentation as needed
- Ensure all tests pass

### 3. Commit Your Changes

```bash
git add .
git commit -m "feat: add your feature description"
```

### 4. Push and Create PR

```bash
git push origin feature/your-feature-name
# Create PR on GitHub
```

## 🎯 Areas for Contribution

### High Priority

- **Performance Optimization**: Improve CRDT merge algorithms
- **Error Handling**: Enhance error recovery strategies
- **Testing**: Add more comprehensive test coverage
- **Documentation**: Improve examples and guides

### Medium Priority

- **New CRDT Types**: Implement additional CRDT algorithms
- **Transport Layer**: Add new transport protocols
- **Storage Backends**: Support for additional storage systems
- **Monitoring**: Add metrics and observability

### Low Priority

- **Language Bindings**: Python, JavaScript, etc.
- **IDE Support**: Language server, syntax highlighting
- **Benchmarks**: Performance comparison tools
- **Migration Tools**: Help users migrate from other systems

## 🐛 Bug Reports

### Before Reporting

1. Check if the issue is already reported
2. Try to reproduce with the latest version
3. Check if it's a platform-specific issue

### Bug Report Template

```markdown
**Description**
Brief description of the issue

**Steps to Reproduce**
1. Step 1
2. Step 2
3. Step 3

**Expected Behavior**
What should happen

**Actual Behavior**
What actually happens

**Environment**
- OS: [e.g., macOS, Linux, Windows]
- Rust Version: [e.g., 1.75.0]
- Leptos Version: [e.g., 0.8.0-rc2]
- Browser: [e.g., Chrome 108, Firefox 110]

**Additional Context**
Any other relevant information
```

## 💡 Feature Requests

### Feature Request Template

```markdown
**Description**
Brief description of the feature

**Use Case**
Why this feature would be useful

**Proposed Solution**
How you think it should work

**Alternatives Considered**
Other approaches you've considered

**Additional Context**
Any other relevant information
```

## 🔒 Security

### Reporting Security Issues

If you discover a security vulnerability, please:

1. **DO NOT** create a public issue
2. Email security@cloud-shuttle.com
3. Include detailed information about the vulnerability
4. Allow time for assessment and response

### Security Guidelines

- Never commit sensitive information
- Follow secure coding practices
- Validate all inputs
- Use secure random number generation
- Implement proper access controls

## 📚 Resources

### Learning Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Leptos Documentation](https://leptos.dev)
- [CRDT Research](https://crdt.tech)
- [WASM Guide](https://rustwasm.github.io/docs/book/)

### Community

- [GitHub Discussions](https://github.com/cloud-shuttle/leptos-sync/discussions)
- [Rust Community](https://www.rust-lang.org/community)
- [Leptos Discord](https://discord.gg/leptos)

## 🏆 Recognition

Contributors will be recognized in:

- Project README
- Release notes
- Contributor hall of fame
- GitHub contributors list

## 📄 License

By contributing to Leptos-Sync, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).

---

Thank you for contributing to Leptos-Sync! 🚀
