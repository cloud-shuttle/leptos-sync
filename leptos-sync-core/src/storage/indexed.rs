//! Indexed storage implementation for faster lookups and queries

use crate::storage::StorageError;
use crate::storage::Storage as StorageEnum;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;

/// Index-related errors
#[derive(Error, Debug)]
pub enum IndexError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Index not found: {0}")]
    IndexNotFound(String),
    #[error("Index already exists: {0}")]
    IndexAlreadyExists(String),
    #[error("Invalid index value")]
    InvalidIndexValue,
}

/// Index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Index name
    pub name: String,
    /// Index type
    pub index_type: IndexType,
    /// Whether the index is unique
    pub unique: bool,
    /// Whether the index is sparse (allows null values)
    pub sparse: bool,
}

/// Index types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IndexType {
    /// Hash index for exact matches
    Hash,
    /// B-tree index for range queries and sorting
    BTree,
    /// Full-text index for text search
    FullText,
}

impl Default for IndexType {
    fn default() -> Self {
        Self::Hash
    }
}

/// Index metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    /// Index configuration
    pub config: IndexConfig,
    /// Number of entries in the index
    pub entry_count: usize,
    /// Index size in bytes
    pub size_bytes: usize,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    /// Indexed value
    pub value: String,
    /// Associated document IDs
    pub document_ids: Vec<String>,
    /// Timestamp when this entry was last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Indexed storage implementation
pub struct IndexedStorage {
    /// Primary storage backend
    primary: Arc<StorageEnum>,
    /// Secondary indices
    indices: Arc<RwLock<HashMap<String, Box<dyn Index>>>>,
    /// Index metadata
    metadata: Arc<RwLock<HashMap<String, IndexMetadata>>>,
}

/// Index trait for different index implementations
#[async_trait::async_trait]
pub trait Index: Send + Sync {
    /// Get the index name
    fn name(&self) -> &str;
    
    /// Get the index type
    fn index_type(&self) -> IndexType;
    
    /// Insert a value with associated document ID
    async fn insert(&mut self, value: &str, document_id: &str) -> Result<(), IndexError>;
    
    /// Remove a value-document association
    async fn remove(&mut self, value: &str, document_id: &str) -> Result<(), IndexError>;
    
    /// Get document IDs for a value
    async fn get(&self, value: &str) -> Result<Vec<String>, IndexError>;
    
    /// Get all values in the index
    async fn values(&self) -> Result<Vec<String>, IndexError>;
    
    /// Get index statistics
    async fn stats(&self) -> Result<IndexStats, IndexError>;
    
    /// Clear the index
    async fn clear(&mut self) -> Result<(), IndexError>;
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Number of entries
    pub entry_count: usize,
    /// Index size in bytes
    pub size_bytes: usize,
    /// Average entries per value
    pub avg_entries_per_value: f64,
    /// Most common values
    pub top_values: Vec<(String, usize)>,
}

/// Hash index implementation
pub struct HashIndex {
    name: String,
    data: HashMap<String, Vec<String>>,
    unique: bool,
    sparse: bool,
}

impl HashIndex {
    pub fn new(name: String, config: &IndexConfig) -> Self {
        Self {
            name,
            data: HashMap::new(),
            unique: config.unique,
            sparse: config.sparse,
        }
    }
}

#[async_trait::async_trait]
impl Index for HashIndex {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn index_type(&self) -> IndexType {
        IndexType::Hash
    }
    
    async fn insert(&mut self, value: &str, document_id: &str) -> Result<(), IndexError> {
        if value.is_empty() && !self.sparse {
            return Err(IndexError::InvalidIndexValue);
        }
        
        let entry = self.data.entry(value.to_string()).or_insert_with(Vec::new);
        
        if self.unique && !entry.is_empty() {
            return Err(IndexError::InvalidIndexValue);
        }
        
        if !entry.contains(&document_id.to_string()) {
            entry.push(document_id.to_string());
        }
        
        Ok(())
    }
    
    async fn remove(&mut self, value: &str, document_id: &str) -> Result<(), IndexError> {
        if let Some(entry) = self.data.get_mut(value) {
            entry.retain(|id| id != document_id);
            if entry.is_empty() {
                self.data.remove(value);
            }
        }
        Ok(())
    }
    
    async fn get(&self, value: &str) -> Result<Vec<String>, IndexError> {
        Ok(self.data.get(value).cloned().unwrap_or_default())
    }
    
    async fn values(&self) -> Result<Vec<String>, IndexError> {
        Ok(self.data.keys().cloned().collect())
    }
    
    async fn stats(&self) -> Result<IndexStats, IndexError> {
        let entry_count = self.data.len();
        let total_docs: usize = self.data.values().map(|v| v.len()).sum();
        let avg_entries = if entry_count > 0 {
            total_docs as f64 / entry_count as f64
        } else {
            0.0
        };
        
        let mut top_values: Vec<_> = self.data.iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();
        top_values.sort_by(|a, b| b.1.cmp(&a.1));
        top_values.truncate(10);
        
        Ok(IndexStats {
            entry_count,
            size_bytes: std::mem::size_of_val(&self.data),
            avg_entries_per_value: avg_entries,
            top_values,
        })
    }
    
    async fn clear(&mut self) -> Result<(), IndexError> {
        self.data.clear();
        Ok(())
    }
}

/// B-tree index implementation
pub struct BTreeIndex {
    name: String,
    data: BTreeMap<String, Vec<String>>,
    unique: bool,
    sparse: bool,
}

impl BTreeIndex {
    pub fn new(name: String, config: &IndexConfig) -> Self {
        Self {
            name,
            data: BTreeMap::new(),
            unique: config.unique,
            sparse: config.sparse,
        }
    }
}

#[async_trait::async_trait]
impl Index for BTreeIndex {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn index_type(&self) -> IndexType {
        IndexType::BTree
    }
    
    async fn insert(&mut self, value: &str, document_id: &str) -> Result<(), IndexError> {
        if value.is_empty() && !self.sparse {
            return Err(IndexError::InvalidIndexValue);
        }
        
        let entry = self.data.entry(value.to_string()).or_insert_with(Vec::new);
        
        if self.unique && !entry.is_empty() {
            return Err(IndexError::InvalidIndexValue);
        }
        
        if !entry.contains(&document_id.to_string()) {
            entry.push(document_id.to_string());
        }
        
        Ok(())
    }
    
    async fn remove(&mut self, value: &str, document_id: &str) -> Result<(), IndexError> {
        if let Some(entry) = self.data.get_mut(value) {
            entry.retain(|id| id != document_id);
            if entry.is_empty() {
                self.data.remove(value);
            }
        }
        Ok(())
    }
    
    async fn get(&self, value: &str) -> Result<Vec<String>, IndexError> {
        Ok(self.data.get(value).cloned().unwrap_or_default())
    }
    
    async fn values(&self) -> Result<Vec<String>, IndexError> {
        Ok(self.data.keys().cloned().collect())
    }
    
    async fn stats(&self) -> Result<IndexStats, IndexError> {
        let entry_count = self.data.len();
        let total_docs: usize = self.data.values().map(|v| v.len()).sum();
        let avg_entries = if entry_count > 0 {
            total_docs as f64 / entry_count as f64
        } else {
            0.0
        };
        
        let mut top_values: Vec<_> = self.data.iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();
        top_values.sort_by(|a, b| b.1.cmp(&a.1));
        top_values.truncate(10);
        
        Ok(IndexStats {
            entry_count,
            size_bytes: std::mem::size_of_val(&self.data),
            avg_entries_per_value: avg_entries,
            top_values,
        })
    }
    
    async fn clear(&mut self) -> Result<(), IndexError> {
        self.data.clear();
        Ok(())
    }
}

impl IndexedStorage {
    /// Create a new indexed storage
    pub fn new(primary: Arc<StorageEnum>) -> Self {
        Self {
            primary,
            indices: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create an index
    pub async fn create_index(&self, config: IndexConfig) -> Result<(), IndexError> {
        let index_name = config.name.clone();
        
        // Check if index already exists
        if self.indices.read().await.contains_key(&index_name) {
            return Err(IndexError::IndexAlreadyExists(index_name));
        }
        
        // Create index based on type
        let index: Box<dyn Index> = match config.index_type {
            IndexType::Hash => Box::new(HashIndex::new(index_name.clone(), &config)),
            IndexType::BTree => Box::new(BTreeIndex::new(index_name.clone(), &config)),
            IndexType::FullText => {
                // TODO: Implement full-text index
                return Err(IndexError::InvalidIndexValue);
            }
        };
        
        // Add index
        self.indices.write().await.insert(index_name.clone(), index);
        
        // Add metadata
        let metadata = IndexMetadata {
            config,
            entry_count: 0,
            size_bytes: 0,
            last_updated: chrono::Utc::now(),
        };
        self.metadata.write().await.insert(index_name, metadata);
        
        Ok(())
    }
    
    /// Drop an index
    pub async fn drop_index(&self, name: &str) -> Result<(), IndexError> {
        if !self.indices.read().await.contains_key(name) {
            return Err(IndexError::IndexNotFound(name.to_string()));
        }
        
        self.indices.write().await.remove(name);
        self.metadata.write().await.remove(name);
        
        Ok(())
    }
    
    /// Get all index names
    pub async fn list_indices(&self) -> Vec<String> {
        self.indices.read().await.keys().cloned().collect()
    }
    
    /// Get index metadata
    pub async fn get_index_metadata(&self, name: &str) -> Option<IndexMetadata> {
        self.metadata.read().await.get(name).cloned()
    }
    
    /// Query by index
    pub async fn query_by_index(&self, index_name: &str, value: &str) -> Result<Vec<String>, IndexError> {
        let indices = self.indices.read().await;
        let index = indices.get(index_name)
            .ok_or_else(|| IndexError::IndexNotFound(index_name.to_string()))?;
        
        index.get(value).await
    }
    
    /// Range query (for B-tree indices)
    pub async fn range_query(&self, index_name: &str, _start: &str, _end: &str) -> Result<Vec<String>, IndexError> {
        let indices = self.indices.read().await;
        let index = indices.get(index_name)
            .ok_or_else(|| IndexError::IndexNotFound(index_name.to_string()))?;
        
        if index.index_type() != IndexType::BTree {
            return Err(IndexError::InvalidIndexValue);
        }
        
        // For now, return empty result
        // TODO: Implement proper range query
        Ok(Vec::new())
    }
    
    /// Get index statistics
    pub async fn get_index_stats(&self, name: &str) -> Result<IndexStats, IndexError> {
        let indices = self.indices.read().await;
        let index = indices.get(name)
            .ok_or_else(|| IndexError::IndexNotFound(name.to_string()))?;
        
        index.stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::memory::MemoryStorage;

    #[tokio::test]
    async fn test_index_creation() {
        let primary = Arc::new(StorageEnum::Memory(MemoryStorage::new()));
        let indexed = IndexedStorage::new(primary);
        
        let config = IndexConfig {
            name: "test_index".to_string(),
            index_type: IndexType::Hash,
            unique: false,
            sparse: false,
        };
        
        assert!(indexed.create_index(config).await.is_ok());
        assert!(indexed.list_indices().await.contains(&"test_index".to_string()));
    }

    #[tokio::test]
    async fn test_duplicate_index() {
        let primary = Arc::new(StorageEnum::Memory(MemoryStorage::new()));
        let indexed = IndexedStorage::new(primary);
        
        let config = IndexConfig {
            name: "test_index".to_string(),
            index_type: IndexType::Hash,
            unique: false,
            sparse: false,
        };
        
        assert!(indexed.create_index(config.clone()).await.is_ok());
        assert!(indexed.create_index(config).await.is_err());
    }

    #[tokio::test]
    async fn test_index_drop() {
        let primary = Arc::new(StorageEnum::Memory(MemoryStorage::new()));
        let indexed = IndexedStorage::new(primary);
        
        let config = IndexConfig {
            name: "test_index".to_string(),
            index_type: IndexType::Hash,
            unique: false,
            sparse: false,
        };
        
        assert!(indexed.create_index(config).await.is_ok());
        assert!(indexed.drop_index("test_index").await.is_ok());
        assert!(!indexed.list_indices().await.contains(&"test_index".to_string()));
    }
}
