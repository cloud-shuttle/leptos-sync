#!/bin/bash
# Script to rollback medium-risk dependency updates

set -e

echo "Rolling back medium-risk dependency updates..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Not in project root directory"
    exit 1
fi

# Backup current Cargo.toml
echo "Creating backup of current Cargo.toml..."
cp Cargo.toml Cargo.toml.medium-risk-backup

# Rollback leptos-ws-pro from 0.11.0 to 0.10.0
echo "Rolling back leptos-ws-pro from 0.11.0 to 0.10.0..."
sed -i.bak 's/leptos-ws-pro = "0.11.0"/leptos-ws-pro = "0.10.0"/g' Cargo.toml

# Rollback sqlx from 0.8 to 0.7
echo "Rolling back sqlx from 0.8 to 0.7..."
sed -i.bak 's/sqlx = { version = "0.8"/sqlx = { version = "0.7"/g' Cargo.toml

# Rollback redis from 0.26 to 0.23
echo "Rolling back redis from 0.26 to 0.23..."
sed -i.bak 's/redis = { version = "0.26"/redis = { version = "0.23"/g' Cargo.toml

# Clean up backup files
echo "Cleaning up backup files..."
find . -name "*.bak" -delete

# Update Cargo.lock
echo "Updating Cargo.lock..."
cargo update --package leptos-ws-pro --package sqlx --package redis

# Check compilation
echo "Checking compilation..."
if cargo check --workspace; then
    echo "✅ Compilation successful"
else
    echo "❌ Compilation failed. Please review the errors above."
    echo "Restoring backup..."
    cp Cargo.toml.medium-risk-backup Cargo.toml
    exit 1
fi

# Run tests
echo "Running tests..."
if cargo test --workspace; then
    echo "✅ Tests passed"
else
    echo "❌ Tests failed. Please review the errors above."
    echo "Restoring backup..."
    cp Cargo.toml.medium-risk-backup Cargo.toml
    exit 1
fi

# Clean up backup
rm Cargo.toml.medium-risk-backup

echo "✅ Medium-risk dependency rollback complete!"
echo ""
echo "Summary of rollbacks:"
echo "- leptos-ws-pro: 0.11.0 → 0.10.0"
echo "- sqlx: 0.8 → 0.7"
echo "- redis: 0.26 → 0.23"
echo ""
echo "All dependencies have been rolled back to their previous versions."
echo "Compilation and tests are passing."
echo ""
echo "Next steps:"
echo "1. Review any issues that caused the rollback"
echo "2. Fix any problems before attempting updates again"
echo "3. Test thoroughly before re-applying updates"
echo "4. Consider updating dependencies one at a time"
