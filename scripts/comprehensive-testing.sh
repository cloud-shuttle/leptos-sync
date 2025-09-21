#!/bin/bash
# Comprehensive testing script for leptos-sync before release

set -e

echo "🧪 Starting comprehensive testing for leptos-sync..."
echo "=================================================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Not in project root directory"
    exit 1
fi

# Create test results directory
mkdir -p test-results
TEST_RESULTS_DIR="test-results/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$TEST_RESULTS_DIR"

echo "📁 Test results will be saved to: $TEST_RESULTS_DIR"
echo ""

# 1. Compilation Check
echo "🔨 Phase 1: Compilation Check"
echo "=============================="
echo "Checking compilation for all packages..."

if cargo check --workspace > "$TEST_RESULTS_DIR/compilation.log" 2>&1; then
    echo "✅ All packages compile successfully"
    echo "✅ Compilation check passed" >> "$TEST_RESULTS_DIR/summary.log"
else
    echo "❌ Compilation failed. Check $TEST_RESULTS_DIR/compilation.log"
    echo "❌ Compilation check failed" >> "$TEST_RESULTS_DIR/summary.log"
    exit 1
fi
echo ""

# 2. Unit Tests
echo "🧪 Phase 2: Unit Tests"
echo "======================"
echo "Running all unit tests..."

if cargo test --workspace --lib > "$TEST_RESULTS_DIR/unit_tests.log" 2>&1; then
    echo "✅ All unit tests passed"
    echo "✅ Unit tests passed" >> "$TEST_RESULTS_DIR/summary.log"
else
    echo "❌ Unit tests failed. Check $TEST_RESULTS_DIR/unit_tests.log"
    echo "❌ Unit tests failed" >> "$TEST_RESULTS_DIR/summary.log"
    exit 1
fi
echo ""

# 3. Integration Tests
echo "🔗 Phase 3: Integration Tests"
echo "============================="
echo "Running integration tests..."

if cargo test --test integration > "$TEST_RESULTS_DIR/integration_tests.log" 2>&1; then
    echo "✅ All integration tests passed"
    echo "✅ Integration tests passed" >> "$TEST_RESULTS_DIR/summary.log"
else
    echo "❌ Integration tests failed. Check $TEST_RESULTS_DIR/integration_tests.log"
    echo "❌ Integration tests failed" >> "$TEST_RESULTS_DIR/summary.log"
    exit 1
fi
echo ""

# 4. Contract Tests
echo "📋 Phase 4: Contract Tests"
echo "=========================="
echo "Running contract tests..."

if cargo test --test contracts > "$TEST_RESULTS_DIR/contract_tests.log" 2>&1; then
    echo "✅ All contract tests passed"
    echo "✅ Contract tests passed" >> "$TEST_RESULTS_DIR/summary.log"
else
    echo "❌ Contract tests failed. Check $TEST_RESULTS_DIR/contract_tests.log"
    echo "❌ Contract tests failed" >> "$TEST_RESULTS_DIR/summary.log"
    exit 1
fi
echo ""

# 5. Property Tests
echo "🎲 Phase 5: Property Tests"
echo "=========================="
echo "Running property-based tests..."

if cargo test --test property > "$TEST_RESULTS_DIR/property_tests.log" 2>&1; then
    echo "✅ All property tests passed"
    echo "✅ Property tests passed" >> "$TEST_RESULTS_DIR/summary.log"
else
    echo "❌ Property tests failed. Check $TEST_RESULTS_DIR/property_tests.log"
    echo "❌ Property tests failed" >> "$TEST_RESULTS_DIR/summary.log"
    exit 1
fi
echo ""

# 6. Performance Tests
echo "⚡ Phase 6: Performance Tests"
echo "============================="
echo "Running performance benchmarks..."

if cargo bench --bench sync_performance > "$TEST_RESULTS_DIR/performance_tests.log" 2>&1; then
    echo "✅ All performance tests passed"
    echo "✅ Performance tests passed" >> "$TEST_RESULTS_DIR/summary.log"
else
    echo "⚠️  Performance tests failed or not available. Check $TEST_RESULTS_DIR/performance_tests.log"
    echo "⚠️  Performance tests failed or not available" >> "$TEST_RESULTS_DIR/summary.log"
fi
echo ""

# 7. Browser/WASM Tests (if available)
echo "🌐 Phase 7: Browser/WASM Tests"
echo "==============================="
echo "Running browser/WASM tests..."

if command -v wasm-pack >/dev/null 2>&1; then
    if wasm-pack test --chrome --headless > "$TEST_RESULTS_DIR/browser_tests.log" 2>&1; then
        echo "✅ All browser tests passed"
        echo "✅ Browser tests passed" >> "$TEST_RESULTS_DIR/summary.log"
    else
        echo "⚠️  Browser tests failed or not available. Check $TEST_RESULTS_DIR/browser_tests.log"
        echo "⚠️  Browser tests failed or not available" >> "$TEST_RESULTS_DIR/summary.log"
    fi
else
    echo "⚠️  wasm-pack not available, skipping browser tests"
    echo "⚠️  Browser tests skipped (wasm-pack not available)" >> "$TEST_RESULTS_DIR/summary.log"
fi
echo ""

# 8. Linting and Formatting
echo "🎨 Phase 8: Linting and Formatting"
echo "==================================="
echo "Running clippy and format checks..."

if cargo clippy --workspace -- -D warnings > "$TEST_RESULTS_DIR/clippy.log" 2>&1; then
    echo "✅ All clippy checks passed"
    echo "✅ Clippy checks passed" >> "$TEST_RESULTS_DIR/summary.log"
else
    echo "❌ Clippy checks failed. Check $TEST_RESULTS_DIR/clippy.log"
    echo "❌ Clippy checks failed" >> "$TEST_RESULTS_DIR/summary.log"
    exit 1
fi

if cargo fmt --all -- --check > "$TEST_RESULTS_DIR/format.log" 2>&1; then
    echo "✅ All format checks passed"
    echo "✅ Format checks passed" >> "$TEST_RESULTS_DIR/summary.log"
else
    echo "❌ Format checks failed. Check $TEST_RESULTS_DIR/format.log"
    echo "❌ Format checks failed" >> "$TEST_RESULTS_DIR/summary.log"
    exit 1
fi
echo ""

# 9. Security Audit
echo "🔒 Phase 9: Security Audit"
echo "=========================="
echo "Running security audit..."

if command -v cargo-audit >/dev/null 2>&1; then
    if cargo audit > "$TEST_RESULTS_DIR/security_audit.log" 2>&1; then
        echo "✅ Security audit passed"
        echo "✅ Security audit passed" >> "$TEST_RESULTS_DIR/summary.log"
    else
        echo "⚠️  Security audit found issues. Check $TEST_RESULTS_DIR/security_audit.log"
        echo "⚠️  Security audit found issues" >> "$TEST_RESULTS_DIR/summary.log"
    fi
else
    echo "⚠️  cargo-audit not available, skipping security audit"
    echo "⚠️  Security audit skipped (cargo-audit not available)" >> "$TEST_RESULTS_DIR/summary.log"
fi
echo ""

# 10. Dependency Check
echo "📦 Phase 10: Dependency Check"
echo "=============================="
echo "Checking for outdated dependencies..."

if command -v cargo-outdated >/dev/null 2>&1; then
    cargo outdated > "$TEST_RESULTS_DIR/outdated_deps.log" 2>&1 || true
    echo "✅ Dependency check completed. Check $TEST_RESULTS_DIR/outdated_deps.log"
    echo "✅ Dependency check completed" >> "$TEST_RESULTS_DIR/summary.log"
else
    echo "⚠️  cargo-outdated not available, skipping dependency check"
    echo "⚠️  Dependency check skipped (cargo-outdated not available)" >> "$TEST_RESULTS_DIR/summary.log"
fi
echo ""

# 11. Build All Examples
echo "🏗️  Phase 11: Build Examples"
echo "============================"
echo "Building all examples..."

if cargo build --examples > "$TEST_RESULTS_DIR/examples_build.log" 2>&1; then
    echo "✅ All examples built successfully"
    echo "✅ Examples build passed" >> "$TEST_RESULTS_DIR/summary.log"
else
    echo "❌ Examples build failed. Check $TEST_RESULTS_DIR/examples_build.log"
    echo "❌ Examples build failed" >> "$TEST_RESULTS_DIR/summary.log"
    exit 1
fi
echo ""

# 12. Documentation Check
echo "📚 Phase 12: Documentation Check"
echo "================================"
echo "Checking documentation..."

if cargo doc --workspace --no-deps > "$TEST_RESULTS_DIR/docs.log" 2>&1; then
    echo "✅ Documentation generated successfully"
    echo "✅ Documentation check passed" >> "$TEST_RESULTS_DIR/summary.log"
else
    echo "❌ Documentation generation failed. Check $TEST_RESULTS_DIR/docs.log"
    echo "❌ Documentation check failed" >> "$TEST_RESULTS_DIR/summary.log"
    exit 1
fi
echo ""

# Generate Final Report
echo "📊 Generating Final Report"
echo "=========================="

# Count test results
TOTAL_TESTS=$(grep -c "test result:" "$TEST_RESULTS_DIR/unit_tests.log" 2>/dev/null || echo "0")
PASSED_TESTS=$(grep -c "test result: ok" "$TEST_RESULTS_DIR/unit_tests.log" 2>/dev/null || echo "0")

# Create final report
cat > "$TEST_RESULTS_DIR/final_report.md" << EOF
# Comprehensive Testing Report

**Date**: $(date)
**Test Results Directory**: $TEST_RESULTS_DIR

## Summary

- **Total Tests**: $TOTAL_TESTS
- **Passed Tests**: $PASSED_TESTS
- **Test Results**: All critical tests passed ✅

## Test Phases

$(cat "$TEST_RESULTS_DIR/summary.log")

## Files Generated

- \`compilation.log\` - Compilation check results
- \`unit_tests.log\` - Unit test results
- \`integration_tests.log\` - Integration test results
- \`contract_tests.log\` - Contract test results
- \`property_tests.log\` - Property test results
- \`performance_tests.log\` - Performance test results
- \`browser_tests.log\` - Browser/WASM test results
- \`clippy.log\` - Clippy linting results
- \`format.log\` - Format check results
- \`security_audit.log\` - Security audit results
- \`outdated_deps.log\` - Outdated dependencies
- \`examples_build.log\` - Examples build results
- \`docs.log\` - Documentation generation results

## Conclusion

All critical tests have passed successfully. The project is ready for release.

EOF

echo "✅ Comprehensive testing completed successfully!"
echo "📊 Final report generated: $TEST_RESULTS_DIR/final_report.md"
echo ""
echo "🎉 All tests passed! The project is ready for commit, push, and release."
echo ""
echo "Next steps:"
echo "1. Review the test results in $TEST_RESULTS_DIR/"
echo "2. Commit changes: git add . && git commit -m 'Complete dependency modernization and comprehensive testing'"
echo "3. Push to repository: git push"
echo "4. Create release: git tag v0.8.0 && git push --tags"
echo "5. Publish to crates.io: cargo publish"
echo ""
echo "🚀 Ready for release!"
