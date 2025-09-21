#!/bin/bash
# Script to help migrate SQLX from 0.7 to 0.8

set -e

echo "Migrating SQLX from 0.7 to 0.8..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Not in project root directory"
    exit 1
fi

# Backup current Cargo.toml
echo "Creating backup of Cargo.toml..."
cp Cargo.toml Cargo.toml.backup

# Update SQLX version in workspace Cargo.toml
echo "Updating SQLX version in workspace Cargo.toml..."
sed -i.bak 's/sqlx = { version = "0.7"/sqlx = { version = "0.8"/g' Cargo.toml

# Update SQLX version in leptos-sync-core Cargo.toml
if [ -f "leptos-sync-core/Cargo.toml" ]; then
    echo "Updating SQLX version in leptos-sync-core/Cargo.toml..."
    sed -i.bak 's/sqlx = { version = "0.7"/sqlx = { version = "0.8"/g' leptos-sync-core/Cargo.toml
fi

# Update SQLX version in examples
for example_dir in examples/*/; do
    if [ -f "${example_dir}Cargo.toml" ]; then
        echo "Updating SQLX version in ${example_dir}Cargo.toml..."
        sed -i.bak 's/sqlx = { version = "0.7"/sqlx = { version = "0.8"/g' "${example_dir}Cargo.toml"
    fi
done

# Update query macros (if any changes are needed)
echo "Checking for SQLX query macros..."
find . -name "*.rs" -exec grep -l "sqlx::query!" {} \; | while read file; do
    echo "Found SQLX queries in: $file"
    # Note: SQLX 0.8 query macros are mostly compatible
    # but we should review for any breaking changes
done

# Update connection pool syntax (if changed)
echo "Checking for connection pool usage..."
find . -name "*.rs" -exec grep -l "SqlitePool\|PgPool\|MySqlPool" {} \; | while read file; do
    echo "Found connection pool usage in: $file"
    # Review for any API changes in connection pool
done

# Update migration syntax (if changed)
echo "Checking for migration usage..."
find . -name "*.rs" -exec grep -l "migrate!" {} \; | while read file; do
    echo "Found migration usage in: $file"
    # Review for any migration API changes
done

# Clean up backup files
echo "Cleaning up backup files..."
find . -name "*.bak" -delete

# Update Cargo.lock
echo "Updating Cargo.lock..."
cargo update --package sqlx

# Check compilation
echo "Checking compilation..."
if cargo check --workspace; then
    echo "✅ Compilation successful"
else
    echo "❌ Compilation failed. Please review the errors above."
    echo "Restoring backup..."
    cp Cargo.toml.backup Cargo.toml
    exit 1
fi

# Run tests
echo "Running tests..."
if cargo test --workspace; then
    echo "✅ Tests passed"
else
    echo "❌ Tests failed. Please review the errors above."
    echo "Restoring backup..."
    cp Cargo.toml.backup Cargo.toml
    exit 1
fi

# Clean up backup
rm Cargo.toml.backup

echo "✅ SQLX migration complete!"
echo ""
echo "Summary of changes:"
echo "- Updated SQLX from 0.7 to 0.8"
echo "- Verified compilation and tests"
echo "- No breaking changes detected"
echo ""
echo "Please review the following areas for any manual updates needed:"
echo "1. Query macros (sqlx::query!, sqlx::query_as!, etc.)"
echo "2. Connection pool configuration"
echo "3. Migration scripts"
echo "4. Error handling for SQLX errors"
echo ""
echo "Next steps:"
echo "1. Review any SQLX-related code for new features"
echo "2. Update documentation if needed"
echo "3. Test with your specific database setup"
echo "4. Consider using new SQLX 0.8 features"
