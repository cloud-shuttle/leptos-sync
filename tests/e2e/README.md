# End-to-End Test Suite

This directory contains comprehensive end-to-end tests for the Leptos-Sync application using Playwright.

## Test Structure

```
tests/e2e/
├── core/           # Core functionality tests
├── components/     # UI component tests
├── integration/    # Workflow integration tests
├── performance/    # Performance and load tests
├── accessibility/  # Accessibility compliance tests
├── utils/          # Test utility functions
├── config/         # Test environment configurations
└── README.md       # This file
```

## Running Tests

### Basic Commands

```bash
# Run all E2E tests
pnpm run test:e2e

# Run tests with UI (interactive)
pnpm run test:e2e:ui

# Run tests in headed mode (visible browser)
pnpm run test:e2e:headed

# Run tests in debug mode
pnpm run test:e2e:debug

# Show test report
pnpm run test:e2e:report

# Run all tests (unit + E2E)
pnpm run test:all
```

### Make Commands

```bash
# Run E2E tests
make test-e2e

# Run E2E tests with UI
make test-e2e-ui

# Run E2E tests in headed mode
make test-e2e-headed

# Run E2E tests in debug mode
make test-e2e-debug

# Run all tests
make test-all
```

## Test Categories

### 1. Core Functionality Tests (`core/`)
- **Storage Layer**: Tests local storage, offline functionality, data persistence
- **CRDT Sync**: Tests conflict resolution, network handling, reconnection

### 2. Component Tests (`components/`)
- **UI Components**: Tests component rendering, interactions, responsiveness
- **Accessibility**: Tests keyboard navigation, focus management, ARIA compliance

### 3. Integration Tests (`integration/`)
- **User Workflows**: Tests complete user journeys, offline-to-online transitions
- **Concurrent Operations**: Tests multi-user scenarios, data consistency

### 4. Performance Tests (`performance/`)
- **Load Testing**: Tests page load times, performance budgets
- **Stress Testing**: Tests large datasets, memory management

### 5. Accessibility Tests (`accessibility/`)
- **WCAG Compliance**: Tests color contrast, keyboard navigation
- **Screen Reader**: Tests ARIA labels, semantic structure

## Test Configuration

### Environment Variables

```bash
# Set test environment
TEST_ENV=dev          # Development
TEST_ENV=staging      # Staging
TEST_ENV=production   # Production
TEST_ENV=ci           # CI/CD
```

### Browser Support

- **Desktop**: Chrome, Firefox, Safari
- **Mobile**: Chrome (Android), Safari (iOS)
- **Headless**: All browsers support headless mode

## Writing Tests

### Test Structure

```typescript
import { test, expect } from '@playwright/test';
import { TestHelpers } from '../utils/test-helpers';

test.describe('Feature Name', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await TestHelpers.waitForAppLoad(page);
  });

  test('should perform expected behavior', async ({ page }) => {
    // Arrange
    const element = page.locator('selector');
    
    // Act
    await element.click();
    
    // Assert
    await expect(element).toBeVisible();
  });
});
```

### Best Practices

1. **Use TestHelpers**: Leverage utility functions for common operations
2. **Descriptive Names**: Use clear, descriptive test names
3. **Setup/Teardown**: Use beforeEach/afterEach for test isolation
4. **Assertions**: Use specific, meaningful assertions
5. **Error Handling**: Test both success and failure scenarios

## Continuous Integration

### GitHub Actions

Tests run automatically on:
- Pull requests
- Main branch pushes
- Scheduled runs (nightly)

### Test Reports

- **HTML Reports**: Interactive test results
- **JSON Reports**: Machine-readable results
- **JUnit Reports**: CI/CD integration

## Troubleshooting

### Common Issues

1. **Tests Hanging**: Check web server status
2. **Flaky Tests**: Increase timeouts, add retries
3. **Browser Issues**: Update Playwright browsers
4. **Performance**: Check resource usage, optimize tests

### Debug Mode

```bash
# Run single test in debug mode
npx playwright test test-name.spec.ts --debug

# Run with UI for step-by-step debugging
npx playwright test --ui
```

## Performance Benchmarks

### Target Metrics

- **Page Load**: < 3 seconds
- **Interaction**: < 100ms
- **Sync Operations**: < 5 seconds
- **Memory Usage**: < 100MB

### Monitoring

- Performance metrics are collected automatically
- Reports include timing breakdowns
- Alerts for performance regressions

## Contributing

### Adding New Tests

1. Create test file in appropriate category
2. Follow naming convention: `feature.spec.ts`
3. Use TestHelpers for common operations
4. Add comprehensive assertions
5. Update this README if needed

### Test Review

- All tests must pass before merging
- Performance tests must meet benchmarks
- Accessibility tests must pass compliance checks
- Integration tests must cover critical paths
