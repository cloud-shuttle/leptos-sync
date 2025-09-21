#!/bin/bash
# Script to rollback high-risk dependency updates

set -e

echo "Rolling back high-risk dependency updates..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Not in project root directory"
    exit 1
fi

# Backup current Cargo.toml
echo "Creating backup of current Cargo.toml..."
cp Cargo.toml Cargo.toml.high-risk-backup

# Rollback leptos from 0.9.0 to 0.8.6
echo "Rolling back leptos from 0.9.0 to 0.8.6..."
sed -i.bak 's/leptos = "0.9.0"/leptos = "0.8.6"/g' Cargo.toml

# Rollback leptos_ws from 0.9.0 to 0.8.0-rc2
echo "Rolling back leptos_ws from 0.9.0 to 0.8.0-rc2..."
sed -i.bak 's/leptos_ws = "0.9.0"/leptos_ws = "0.8.0-rc2"/g' Cargo.toml

# Clean up backup files
echo "Cleaning up backup files..."
find . -name "*.bak" -delete

# Update Cargo.lock
echo "Updating Cargo.lock..."
cargo update --package leptos --package leptos_ws

# Check compilation
echo "Checking compilation..."
if cargo check --workspace; then
    echo "✅ Compilation successful"
else
    echo "❌ Compilation failed. Please review the errors above."
    echo "Restoring backup..."
    cp Cargo.toml.high-risk-backup Cargo.toml
    exit 1
fi

# Run tests
echo "Running tests..."
if cargo test --workspace; then
    echo "✅ Tests passed"
else
    echo "❌ Tests failed. Please review the errors above."
    echo "Restoring backup..."
    cp Cargo.toml.high-risk-backup Cargo.toml
    exit 1
fi

# Clean up backup
rm Cargo.toml.high-risk-backup

echo "✅ High-risk dependency rollback complete!"
echo ""
echo "Summary of rollbacks:"
echo "- leptos: 0.9.0 → 0.8.6"
echo "- leptos_ws: 0.9.0 → 0.8.0-rc2"
echo ""
echo "All dependencies have been rolled back to their previous versions."
echo "Compilation and tests are passing."
echo ""
echo "Next steps:"
echo "1. Review any issues that caused the rollback"
echo "2. Fix any problems before attempting updates again"
echo "3. Test thoroughly before re-applying updates"
echo "4. Consider updating dependencies one at a time"
echo "5. Review the migration plan and breaking changes analysis"
