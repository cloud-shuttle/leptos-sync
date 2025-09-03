# Testing Documentation

This directory contains all testing-related files and documentation for Leptos-Sync.

## 🧪 Test Structure

```
tests/
├── README.md                 # This file - test overview
├── e2e/                     # End-to-end tests
│   ├── README.md            # E2E test documentation
│   ├── run-all.spec.ts      # Main E2E test runner
│   ├── accessibility/        # Accessibility tests
│   ├── components/          # Component tests
│   ├── core/                # Core functionality tests
│   ├── fixtures/            # Test fixtures and data
│   ├── integration/         # Integration tests
│   ├── performance/         # Performance tests
│   ├── config/              # Test configuration
│   └── utils/               # Test utilities
└── test-results/            # Test execution results (gitignored)
```

## 🎯 Test Categories

### Unit Tests
- **Location**: `leptos-sync-core/src/**/tests/`
- **Framework**: Rust built-in testing
- **Coverage**: Individual functions and methods
- **Run with**: `cargo test --package leptos-sync-core`

### Integration Tests
- **Location**: `tests/integration/`
- **Framework**: Playwright
- **Coverage**: Module interactions and workflows
- **Run with**: `pnpm test:integration`

### End-to-End Tests
- **Location**: `tests/e2e/`
- **Framework**: Playwright
- **Coverage**: Full user workflows and scenarios
- **Run with**: `pnpm test:e2e`

### Performance Tests
- **Location**: `tests/performance/`
- **Framework**: Playwright + custom metrics
- **Coverage**: Response times, memory usage, bundle size
- **Run with**: `pnpm test:performance`

### Accessibility Tests
- **Location**: `tests/accessibility/`
- **Framework**: Playwright + axe-core
- **Coverage**: WCAG compliance, screen reader support
- **Run with**: `pnpm test:accessibility`

## 🚀 Running Tests

### All Tests
```bash
# Run all tests (unit + integration + e2e)
pnpm test

# Run all tests with coverage
pnpm test:coverage
```

### Unit Tests Only
```bash
# All packages
cargo test --workspace

# Core library only
cargo test --package leptos-sync-core

# Specific module
cargo test --package leptos-sync-core --lib sync::conflict
```

### Integration Tests
```bash
# Run integration tests
pnpm test:integration

# Run with specific browser
pnpm test:integration --project=chromium
```

### End-to-End Tests
```bash
# Run all E2E tests
pnpm test:e2e

# Run specific test file
pnpm test:e2e --grep "user authentication"

# Run in headed mode (see browser)
pnpm test:e2e --headed
```

### Performance Tests
```bash
# Run performance tests
pnpm test:performance

# Generate performance report
pnpm test:performance --reporter=html
```

## 📊 Test Results

### Current Status
- **Total Tests**: 44
- **Passing**: 42 (95.5%)
- **Failing**: 2 (expected IndexedDB failures on native targets)

### Expected Failures
Some tests are expected to fail in certain environments:

- **IndexedDB Tests**: Fail on native targets (expected - IndexedDB is web-only)
- **WebSocket Tests**: Limited functionality on native targets (expected - WASM-optimized)

### Test Coverage
- **Core Library**: 95.5% pass rate
- **CRDT Operations**: 100% pass rate
- **Conflict Resolution**: 100% pass rate
- **Real-time Sync**: 100% pass rate
- **Security Features**: 100% pass rate
- **Error Handling**: 100% pass rate

## 🔧 Test Configuration

### Playwright Configuration
- **File**: `playwright.config.ts`
- **Browsers**: Chromium, Firefox, WebKit
- **Viewport**: 1280x720
- **Timeout**: 30 seconds
- **Retries**: 2

### Test Environment
- **Node.js**: 18+
- **Rust**: 1.75+ (nightly for Leptos 0.8.x)
- **WASM Target**: wasm32-unknown-unknown
- **OS**: Linux, macOS, Windows

## 📝 Writing Tests

### Unit Test Guidelines
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function(input);
        
        // Assert
        assert_eq!(result, "expected");
    }

    #[tokio::test]
    async fn test_async_function() {
        // Async test implementation
    }
}
```

### E2E Test Guidelines
```typescript
import { test, expect } from '@playwright/test';

test('user can create todo item', async ({ page }) => {
  // Navigate to app
  await page.goto('/');
  
  // Perform action
  await page.fill('[data-testid="todo-input"]', 'Buy groceries');
  await page.click('[data-testid="add-todo"]');
  
  // Verify result
  await expect(page.locator('[data-testid="todo-item"]')).toContainText('Buy groceries');
});
```

### Test Naming Conventions
- **Unit Tests**: `test_function_name_description`
- **Integration Tests**: `test_integration_scenario`
- **E2E Tests**: `test_user_can_perform_action`
- **Performance Tests**: `test_performance_metric`
- **Accessibility Tests**: `test_accessibility_requirement`

## 🐛 Debugging Tests

### Debug Mode
```bash
# Run tests in debug mode
pnpm test:e2e --debug

# Run specific test in debug mode
pnpm test:e2e --grep "test name" --debug
```

### Logs and Output
```bash
# Verbose output
cargo test -- --nocapture

# Show test output
pnpm test:e2e --reporter=list
```

### Common Issues
1. **IndexedDB Tests Failing**: Expected on native targets
2. **WebSocket Tests Limited**: Expected on native targets
3. **WASM Compilation**: Ensure nightly Rust toolchain
4. **Browser Compatibility**: Check Playwright configuration

## 📈 Continuous Integration

### GitHub Actions
- **Unit Tests**: Run on every push and PR
- **Integration Tests**: Run on every push and PR
- **E2E Tests**: Run on every push and PR
- **Performance Tests**: Run on release tags
- **Accessibility Tests**: Run on every push and PR

### Test Matrix
- **Operating Systems**: Ubuntu, macOS, Windows
- **Rust Versions**: Stable, Nightly
- **Node.js Versions**: 18, 20
- **Browsers**: Chromium, Firefox, WebKit

## 🤝 Contributing Tests

### Adding New Tests
1. **Unit Tests**: Add to appropriate module in source code
2. **Integration Tests**: Add to `tests/integration/`
3. **E2E Tests**: Add to `tests/e2e/`
4. **Update Documentation**: Update this README if needed

### Test Requirements
- **Coverage**: Aim for >90% test coverage
- **Performance**: Tests should complete in <30 seconds
- **Reliability**: Tests should be deterministic
- **Documentation**: Include clear test descriptions

### Test Review Process
1. **Write Tests**: Following established patterns
2. **Run Locally**: Ensure all tests pass
3. **Submit PR**: Include test files and documentation updates
4. **CI Validation**: GitHub Actions will run all tests
5. **Review**: Tests are reviewed along with code changes

---

**Need help with testing?** Check our [Testing Strategy](../docs/guides/testing-strategy.md) or [GitHub Issues](https://github.com/cloud-shuttle/leptos-sync/issues).
