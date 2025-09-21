//! IndexedDB schema migrations

use super::{connection::IndexedDbConnection, errors::{IndexedDbError, IndexedDbResult}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Migration definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub up: MigrationStep,
    pub down: Option<MigrationStep>,
}

/// Migration step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationStep {
    /// Create object store
    CreateObjectStore {
        name: String,
        key_path: Option<String>,
        auto_increment: bool,
    },
    /// Delete object store
    DeleteObjectStore { name: String },
    /// Create index
    CreateIndex {
        store_name: String,
        index_name: String,
        key_path: String,
        unique: bool,
        multi_entry: bool,
    },
    /// Delete index
    DeleteIndex {
        store_name: String,
        index_name: String,
    },
    /// Data transformation
    TransformData {
        store_name: String,
        transform: DataTransform,
    },
    /// Custom migration function
    Custom { name: String },
}

/// Data transformation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataTransform {
    /// Rename field in all records
    RenameField { old_name: String, new_name: String },
    /// Add default value to field
    AddDefaultField { field_name: String, default_value: serde_json::Value },
    /// Remove field from all records
    RemoveField { field_name: String },
    /// Transform field values using a function
    TransformField { field_name: String, transform_type: TransformType },
}

/// Transform types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformType {
    /// Convert string to number
    StringToNumber,
    /// Convert number to string
    NumberToString,
    /// Convert to uppercase
    ToUppercase,
    /// Convert to lowercase
    ToLowercase,
    /// Add prefix to string
    AddPrefix { prefix: String },
    /// Add suffix to string
    AddSuffix { suffix: String },
}

/// Migration manager
pub struct MigrationManager {
    connection: IndexedDbConnection,
    migrations: Vec<Migration>,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(connection: IndexedDbConnection) -> Self {
        Self {
            connection,
            migrations: Self::get_default_migrations(),
        }
    }

    /// Add a custom migration
    pub fn add_migration(&mut self, migration: Migration) {
        self.migrations.push(migration);
        self.migrations.sort_by_key(|m| m.version);
    }

    /// Run all pending migrations
    pub async fn migrate(&self) -> IndexedDbResult<()> {
        let current_version = self.get_current_version().await?;
        let target_version = self.connection.version();

        if current_version == target_version {
            tracing::info!("Database is already at version {}", target_version);
            return Ok(());
        }

        if current_version < target_version {
            // Upgrade
            self.upgrade(current_version, target_version).await?;
        } else {
            // Downgrade
            self.downgrade(current_version, target_version).await?;
        }

        Ok(())
    }

    /// Upgrade database to target version
    async fn upgrade(&self, from_version: u32, to_version: u32) -> IndexedDbResult<()> {
        tracing::info!("Upgrading database from version {} to {}", from_version, to_version);

        for version in (from_version + 1)..=to_version {
            if let Some(migration) = self.get_migration(version) {
                tracing::info!("Running migration: {} (v{})", migration.name, migration.version);
                self.run_migration(migration, true).await?;
            }
        }

        tracing::info!("Database upgrade completed successfully");
        Ok(())
    }

    /// Downgrade database to target version
    async fn downgrade(&self, from_version: u32, to_version: u32) -> IndexedDbResult<()> {
        tracing::info!("Downgrading database from version {} to {}", from_version, to_version);

        for version in (to_version + 1..=from_version).rev() {
            if let Some(migration) = self.get_migration(version) {
                if migration.down.is_some() {
                    tracing::info!("Rolling back migration: {} (v{})", migration.name, migration.version);
                    self.run_migration(migration, false).await?;
                } else {
                    return Err(IndexedDbError::Migration(format!(
                        "Cannot rollback migration {} (v{}) - no down migration defined",
                        migration.name, migration.version
                    )));
                }
            }
        }

        tracing::info!("Database downgrade completed successfully");
        Ok(())
    }

    /// Run a single migration
    async fn run_migration(&self, migration: &Migration, is_up: bool) -> IndexedDbResult<()> {
        let step = if is_up { &migration.up } else { 
            migration.down.as_ref().ok_or_else(|| {
                IndexedDbError::Migration(format!("No down migration for {}", migration.name))
            })?
        };

        match step {
            MigrationStep::CreateObjectStore { name, key_path, auto_increment } => {
                self.create_object_store(name, key_path.as_deref(), *auto_increment).await?;
            }
            MigrationStep::DeleteObjectStore { name } => {
                self.delete_object_store(name).await?;
            }
            MigrationStep::CreateIndex { store_name, index_name, key_path, unique, multi_entry } => {
                self.create_index(store_name, index_name, key_path, *unique, *multi_entry).await?;
            }
            MigrationStep::DeleteIndex { store_name, index_name } => {
                self.delete_index(store_name, index_name).await?;
            }
            MigrationStep::TransformData { store_name, transform } => {
                self.transform_data(store_name, transform).await?;
            }
            MigrationStep::Custom { name } => {
                self.run_custom_migration(name).await?;
            }
        }

        Ok(())
    }

    /// Get current database version
    async fn get_current_version(&self) -> IndexedDbResult<u32> {
        // In a real implementation, this would read from a version table
        // For now, we'll use the connection version
        Ok(self.connection.version())
    }

    /// Get migration by version
    fn get_migration(&self, version: u32) -> Option<&Migration> {
        self.migrations.iter().find(|m| m.version == version)
    }

    /// Create object store
    async fn create_object_store(&self, name: &str, key_path: Option<&str>, auto_increment: bool) -> IndexedDbResult<()> {
        // This would be implemented in the connection upgrade handler
        tracing::info!("Creating object store: {}", name);
        Ok(())
    }

    /// Delete object store
    async fn delete_object_store(&self, name: &str) -> IndexedDbResult<()> {
        // This would be implemented in the connection upgrade handler
        tracing::info!("Deleting object store: {}", name);
        Ok(())
    }

    /// Create index
    async fn create_index(&self, store_name: &str, index_name: &str, key_path: &str, unique: bool, multi_entry: bool) -> IndexedDbResult<()> {
        // This would be implemented in the connection upgrade handler
        tracing::info!("Creating index: {} on store {}", index_name, store_name);
        Ok(())
    }

    /// Delete index
    async fn delete_index(&self, store_name: &str, index_name: &str) -> IndexedDbResult<()> {
        // This would be implemented in the connection upgrade handler
        tracing::info!("Deleting index: {} from store {}", index_name, store_name);
        Ok(())
    }

    /// Transform data
    async fn transform_data(&self, store_name: &str, transform: &DataTransform) -> IndexedDbResult<()> {
        tracing::info!("Transforming data in store: {}", store_name);
        
        match transform {
            DataTransform::RenameField { old_name, new_name } => {
                tracing::info!("Renaming field {} to {}", old_name, new_name);
                // Implementation would iterate through all records and rename the field
            }
            DataTransform::AddDefaultField { field_name, default_value } => {
                tracing::info!("Adding default field {} with value {:?}", field_name, default_value);
                // Implementation would add the field to all records
            }
            DataTransform::RemoveField { field_name } => {
                tracing::info!("Removing field {}", field_name);
                // Implementation would remove the field from all records
            }
            DataTransform::TransformField { field_name, transform_type } => {
                tracing::info!("Transforming field {} with transform {:?}", field_name, transform_type);
                // Implementation would transform the field values
            }
        }

        Ok(())
    }

    /// Run custom migration
    async fn run_custom_migration(&self, name: &str) -> IndexedDbResult<()> {
        tracing::info!("Running custom migration: {}", name);
        
        match name {
            "add_compression_metadata" => {
                self.add_compression_metadata().await?;
            }
            "migrate_peer_format" => {
                self.migrate_peer_format().await?;
            }
            "add_delta_indexes" => {
                self.add_delta_indexes().await?;
            }
            _ => {
                return Err(IndexedDbError::Migration(format!("Unknown custom migration: {}", name)));
            }
        }

        Ok(())
    }

    /// Add compression metadata to existing records
    async fn add_compression_metadata(&self) -> IndexedDbResult<()> {
        tracing::info!("Adding compression metadata to existing records");
        // Implementation would add compression metadata to all existing records
        Ok(())
    }

    /// Migrate peer format to new structure
    async fn migrate_peer_format(&self) -> IndexedDbResult<()> {
        tracing::info!("Migrating peer format to new structure");
        // Implementation would transform peer records to new format
        Ok(())
    }

    /// Add additional indexes for delta queries
    async fn add_delta_indexes(&self) -> IndexedDbResult<()> {
        tracing::info!("Adding additional indexes for delta queries");
        // Implementation would add new indexes to the deltas store
        Ok(())
    }

    /// Get default migrations
    fn get_default_migrations() -> Vec<Migration> {
        vec![
            Migration {
                version: 1,
                name: "initial_schema".to_string(),
                description: "Create initial database schema".to_string(),
                up: MigrationStep::CreateObjectStore {
                    name: "collections".to_string(),
                    key_path: None,
                    auto_increment: false,
                },
                down: None,
            },
            Migration {
                version: 2,
                name: "add_conflicts_store".to_string(),
                description: "Add conflicts object store for conflict resolution".to_string(),
                up: MigrationStep::CreateObjectStore {
                    name: "conflicts".to_string(),
                    key_path: None,
                    auto_increment: false,
                },
                down: Some(MigrationStep::DeleteObjectStore {
                    name: "conflicts".to_string(),
                }),
            },
            Migration {
                version: 3,
                name: "add_compression_store".to_string(),
                description: "Add compression object store for compression metadata".to_string(),
                up: MigrationStep::CreateObjectStore {
                    name: "compression".to_string(),
                    key_path: None,
                    auto_increment: false,
                },
                down: Some(MigrationStep::DeleteObjectStore {
                    name: "compression".to_string(),
                }),
            },
        ]
    }

    /// Validate migration integrity
    pub fn validate_migrations(&self) -> IndexedDbResult<()> {
        let mut versions = Vec::new();
        
        for migration in &self.migrations {
            if versions.contains(&migration.version) {
                return Err(IndexedDbError::Migration(format!(
                    "Duplicate migration version: {}",
                    migration.version
                )));
            }
            versions.push(migration.version);
        }

        // Check for gaps in version numbers
        versions.sort();
        for i in 1..versions.len() {
            if versions[i] != versions[i - 1] + 1 {
                return Err(IndexedDbError::Migration(format!(
                    "Gap in migration versions: {} -> {}",
                    versions[i - 1], versions[i]
                )));
            }
        }

        Ok(())
    }

    /// Get migration history
    pub fn get_migration_history(&self) -> &[Migration] {
        &self.migrations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_validation() {
        let connection = IndexedDbConnection::open("test_migration_db", 1).await.unwrap();
        let manager = MigrationManager::new(connection);
        
        let result = manager.validate_migrations();
        assert!(result.is_ok());
    }

    #[test]
    fn test_migration_history() {
        let connection = IndexedDbConnection::open("test_history_db", 1).await.unwrap();
        let manager = MigrationManager::new(connection);
        
        let history = manager.get_migration_history();
        assert!(!history.is_empty());
        assert_eq!(history[0].version, 1);
        assert_eq!(history[0].name, "initial_schema");
    }

    #[test]
    fn test_custom_migration() {
        let connection = IndexedDbConnection::open("test_custom_db", 1).await.unwrap();
        let mut manager = MigrationManager::new(connection);
        
        let custom_migration = Migration {
            version: 4,
            name: "custom_test".to_string(),
            description: "Test custom migration".to_string(),
            up: MigrationStep::Custom { name: "test_custom".to_string() },
            down: None,
        };
        
        manager.add_migration(custom_migration);
        
        let history = manager.get_migration_history();
        assert_eq!(history.len(), 4);
        assert_eq!(history[3].version, 4);
        assert_eq!(history[3].name, "custom_test");
    }
}
