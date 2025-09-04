//! Custom CRDT Builder Framework
//!
//! This module provides a framework for users to define their own CRDT types
//! using declarative macros and trait implementations.

use crate::crdt::{CRDT, Mergeable, ReplicaId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Error types for CRDT builder operations
#[derive(Debug, Clone, PartialEq)]
pub enum BuilderError {
    /// Invalid field configuration
    InvalidFieldConfig(String),
    /// Missing required field
    MissingField(String),
    /// Type mismatch in field
    TypeMismatch(String),
    /// Strategy not supported
    UnsupportedStrategy(String),
    /// Serialization error
    SerializationError(String),
    /// Merge operation failed
    MergeError(String),
}

impl fmt::Display for BuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuilderError::InvalidFieldConfig(msg) => write!(f, "Invalid field config: {}", msg),
            BuilderError::MissingField(field) => write!(f, "Missing required field: {}", field),
            BuilderError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            BuilderError::UnsupportedStrategy(strategy) => write!(f, "Unsupported strategy: {}", strategy),
            BuilderError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            BuilderError::MergeError(msg) => write!(f, "Merge error: {}", msg),
        }
    }
}

impl Error for BuilderError {}

/// CRDT field strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrdtStrategy {
    /// Last-Write-Wins strategy
    Lww,
    /// Add-Wins strategy
    AddWins,
    /// Remove-Wins strategy
    RemoveWins,
    /// Grow-Only Counter
    GCounter,
    /// Multi-Value Register
    MvRegister,
    /// Replicated Growable Array
    Rga,
    /// Logoot Sequence
    Lseq,
    /// Yjs-style tree
    YjsTree,
    /// Directed Acyclic Graph
    Dag,
}

/// Field configuration for CRDT builder
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldConfig {
    /// Field name
    pub name: String,
    /// CRDT strategy to use
    pub strategy: CrdtStrategy,
    /// Whether the field is optional
    pub optional: bool,
    /// Default value for optional fields
    pub default: Option<serde_json::Value>,
}

/// CRDT builder configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrdtBuilderConfig {
    /// CRDT type name
    pub type_name: String,
    /// Field configurations
    pub fields: Vec<FieldConfig>,
    /// Replica ID field name (optional, defaults to auto-generated)
    pub replica_id_field: Option<String>,
}

/// Trait for CRDT field operations
pub trait CrdtField: Clone + Send + Sync {
    /// Get the field value
    fn get_value(&self) -> serde_json::Value;
    
    /// Set the field value
    fn set_value(&mut self, value: serde_json::Value) -> Result<(), BuilderError>;
    
    /// Merge with another field
    fn merge(&mut self, other: &Self) -> Result<(), BuilderError>;
    
    /// Check if there's a conflict with another field
    fn has_conflict(&self, other: &Self) -> bool;
    
    /// Get the field strategy
    fn strategy(&self) -> CrdtStrategy;
}

/// Generic CRDT field implementation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericCrdtField {
    /// Field name
    pub name: String,
    /// Field value
    pub value: serde_json::Value,
    /// CRDT strategy
    pub strategy: CrdtStrategy,
    /// Field metadata (timestamps, replica IDs, etc.)
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CrdtField for GenericCrdtField {
    fn get_value(&self) -> serde_json::Value {
        self.value.clone()
    }
    
    fn set_value(&mut self, value: serde_json::Value) -> Result<(), BuilderError> {
        self.value = value;
        Ok(())
    }
    
    fn merge(&mut self, other: &Self) -> Result<(), BuilderError> {
        if self.strategy != other.strategy {
            return Err(BuilderError::TypeMismatch(
                format!("Cannot merge fields with different strategies: {:?} vs {:?}", 
                        self.strategy, other.strategy)
            ));
        }
        
        match self.strategy {
            CrdtStrategy::Lww => self.merge_lww(other),
            CrdtStrategy::AddWins => self.merge_add_wins(other),
            CrdtStrategy::RemoveWins => self.merge_remove_wins(other),
            CrdtStrategy::GCounter => self.merge_gcounter(other),
            CrdtStrategy::MvRegister => self.merge_mv_register(other),
            CrdtStrategy::Rga => self.merge_rga(other),
            CrdtStrategy::Lseq => self.merge_lseq(other),
            CrdtStrategy::YjsTree => self.merge_yjs_tree(other),
            CrdtStrategy::Dag => self.merge_dag(other),
        }
    }
    
    fn has_conflict(&self, other: &Self) -> bool {
        if self.strategy != other.strategy {
            return true;
        }
        
        match self.strategy {
            CrdtStrategy::Lww => self.has_lww_conflict(other),
            CrdtStrategy::AddWins => self.has_add_wins_conflict(other),
            CrdtStrategy::RemoveWins => self.has_remove_wins_conflict(other),
            CrdtStrategy::GCounter => false, // G-Counters never conflict
            CrdtStrategy::MvRegister => self.has_mv_register_conflict(other),
            CrdtStrategy::Rga => self.has_rga_conflict(other),
            CrdtStrategy::Lseq => self.has_lseq_conflict(other),
            CrdtStrategy::YjsTree => self.has_yjs_tree_conflict(other),
            CrdtStrategy::Dag => self.has_dag_conflict(other),
        }
    }
    
    fn strategy(&self) -> CrdtStrategy {
        self.strategy.clone()
    }
}

impl GenericCrdtField {
    /// Create a new generic CRDT field
    pub fn new(name: String, value: serde_json::Value, strategy: CrdtStrategy) -> Self {
        Self {
            name,
            value,
            strategy,
            metadata: HashMap::new(),
        }
    }
    
    /// Merge using Last-Write-Wins strategy
    fn merge_lww(&mut self, other: &Self) -> Result<(), BuilderError> {
        let self_timestamp = self.metadata.get("timestamp")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let other_timestamp = other.metadata.get("timestamp")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        if other_timestamp >= self_timestamp {
            self.value = other.value.clone();
            self.metadata = other.metadata.clone();
        }
        
        Ok(())
    }
    
    /// Merge using Add-Wins strategy
    fn merge_add_wins(&mut self, other: &Self) -> Result<(), BuilderError> {
        // For add-wins, we keep all values that were added
        if let (Some(self_set), Some(other_set)) = (
            self.value.as_array(),
            other.value.as_array()
        ) {
            let mut combined: Vec<serde_json::Value> = self_set.clone();
            for item in other_set {
                if !combined.contains(item) {
                    combined.push(item.clone());
                }
            }
            self.value = serde_json::Value::Array(combined);
        } else {
            // For non-array values, use LWW as fallback
            self.merge_lww(other)?;
        }
        
        Ok(())
    }
    
    /// Merge using Remove-Wins strategy
    fn merge_remove_wins(&mut self, other: &Self) -> Result<(), BuilderError> {
        // For remove-wins, we remove items that were explicitly removed
        if let (Some(self_set), Some(other_set)) = (
            self.value.as_array(),
            other.value.as_array()
        ) {
            let mut combined: Vec<serde_json::Value> = self_set.clone();
            for item in other_set {
                if !combined.contains(item) {
                    combined.push(item.clone());
                }
            }
            
            // Remove items marked as removed
            combined.retain(|item| {
                !other.metadata.get("removed")
                    .and_then(|v| v.as_array())
                    .map(|removed| removed.contains(item))
                    .unwrap_or(false)
            });
            
            self.value = serde_json::Value::Array(combined);
        } else {
            // For non-array values, use LWW as fallback
            self.merge_lww(other)?;
        }
        
        Ok(())
    }
    
    /// Merge using G-Counter strategy
    fn merge_gcounter(&mut self, other: &Self) -> Result<(), BuilderError> {
        if let (Some(self_count), Some(other_count)) = (
            self.value.as_u64(),
            other.value.as_u64()
        ) {
            self.value = serde_json::Value::Number(serde_json::Number::from(
                self_count.max(other_count)
            ));
        }
        
        Ok(())
    }
    
    /// Merge using Multi-Value Register strategy
    fn merge_mv_register(&mut self, other: &Self) -> Result<(), BuilderError> {
        // For MV-Register, we keep all concurrent values
        if let (Some(self_values), Some(other_values)) = (
            self.value.as_array(),
            other.value.as_array()
        ) {
            let mut combined: Vec<serde_json::Value> = self_values.clone();
            for value in other_values {
                if !combined.contains(value) {
                    combined.push(value.clone());
                }
            }
            self.value = serde_json::Value::Array(combined);
        } else {
            // For non-array values, use LWW as fallback
            self.merge_lww(other)?;
        }
        
        Ok(())
    }
    
    /// Merge using RGA strategy (simplified)
    fn merge_rga(&mut self, other: &Self) -> Result<(), BuilderError> {
        // Simplified RGA merge - in practice, this would be much more complex
        self.merge_add_wins(other)
    }
    
    /// Merge using LSEQ strategy (simplified)
    fn merge_lseq(&mut self, other: &Self) -> Result<(), BuilderError> {
        // Simplified LSEQ merge - in practice, this would be much more complex
        self.merge_add_wins(other)
    }
    
    /// Merge using Yjs-style tree strategy (simplified)
    fn merge_yjs_tree(&mut self, other: &Self) -> Result<(), BuilderError> {
        // Simplified Yjs tree merge - in practice, this would be much more complex
        self.merge_add_wins(other)
    }
    
    /// Merge using DAG strategy (simplified)
    fn merge_dag(&mut self, other: &Self) -> Result<(), BuilderError> {
        // Simplified DAG merge - in practice, this would be much more complex
        self.merge_add_wins(other)
    }
    
    // Conflict detection methods
    fn has_lww_conflict(&self, other: &Self) -> bool {
        let self_timestamp = self.metadata.get("timestamp")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let other_timestamp = other.metadata.get("timestamp")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        self.value != other.value && self_timestamp == other_timestamp
    }
    
    fn has_add_wins_conflict(&self, _other: &Self) -> bool {
        false // Add-wins never conflicts
    }
    
    fn has_remove_wins_conflict(&self, _other: &Self) -> bool {
        false // Remove-wins never conflicts
    }
    
    fn has_mv_register_conflict(&self, other: &Self) -> bool {
        self.value != other.value
    }
    
    fn has_rga_conflict(&self, other: &Self) -> bool {
        self.value != other.value
    }
    
    fn has_lseq_conflict(&self, other: &Self) -> bool {
        self.value != other.value
    }
    
    fn has_yjs_tree_conflict(&self, other: &Self) -> bool {
        self.value != other.value
    }
    
    fn has_dag_conflict(&self, other: &Self) -> bool {
        self.value != other.value
    }
}

/// Custom CRDT built using the builder framework
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomCrdt {
    /// CRDT configuration
    pub config: CrdtBuilderConfig,
    /// Field values
    pub fields: HashMap<String, GenericCrdtField>,
    /// Replica ID
    pub replica_id: ReplicaId,
}

impl CRDT for CustomCrdt {
    fn replica_id(&self) -> &ReplicaId {
        &self.replica_id
    }
}

impl Mergeable for CustomCrdt {
    type Error = BuilderError;
    
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        if self.config.type_name != other.config.type_name {
            return Err(BuilderError::TypeMismatch(
                format!("Cannot merge CRDTs of different types: {} vs {}", 
                        self.config.type_name, other.config.type_name)
            ));
        }
        
        // Merge each field
        for (field_name, other_field) in &other.fields {
            if let Some(self_field) = self.fields.get_mut(field_name) {
                self_field.merge(other_field)?;
            } else {
                // Add new field from other CRDT
                self.fields.insert(field_name.clone(), other_field.clone());
            }
        }
        
        Ok(())
    }
    
    fn has_conflict(&self, other: &Self) -> bool {
        if self.config.type_name != other.config.type_name {
            return true;
        }
        
        // Check for conflicts in each field
        for (field_name, self_field) in &self.fields {
            if let Some(other_field) = other.fields.get(field_name) {
                if self_field.has_conflict(other_field) {
                    return true;
                }
            }
        }
        
        false
    }
}

impl CustomCrdt {
    /// Create a new custom CRDT
    pub fn new(config: CrdtBuilderConfig, replica_id: ReplicaId) -> Self {
        let mut fields = HashMap::new();
        
        // Initialize fields with default values
        for field_config in &config.fields {
            let default_value = field_config.default.clone()
                .unwrap_or_else(|| serde_json::Value::Null);
            
            let field = GenericCrdtField::new(
                field_config.name.clone(),
                default_value,
                field_config.strategy.clone(),
            );
            
            fields.insert(field_config.name.clone(), field);
        }
        
        Self {
            config,
            fields,
            replica_id,
        }
    }
    
    /// Get a field value
    pub fn get_field(&self, field_name: &str) -> Option<&serde_json::Value> {
        self.fields.get(field_name).map(|f| &f.value)
    }
    
    /// Set a field value
    pub fn set_field(&mut self, field_name: &str, value: serde_json::Value) -> Result<(), BuilderError> {
        if let Some(field) = self.fields.get_mut(field_name) {
            field.set_value(value)?;
            // Update timestamp for LWW fields
            if field.strategy == CrdtStrategy::Lww {
                field.metadata.insert("timestamp".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64
                    )));
            }
            Ok(())
        } else {
            Err(BuilderError::MissingField(field_name.to_string()))
        }
    }
    
    /// Get all field names
    pub fn field_names(&self) -> Vec<String> {
        self.fields.keys().cloned().collect()
    }
    
    /// Get field configuration
    pub fn get_field_config(&self, field_name: &str) -> Option<&FieldConfig> {
        self.config.fields.iter().find(|f| f.name == field_name)
    }
}

/// CRDT Builder for creating custom CRDT types
pub struct CrdtBuilder {
    config: CrdtBuilderConfig,
}

impl CrdtBuilder {
    /// Create a new CRDT builder
    pub fn new(type_name: String) -> Self {
        Self {
            config: CrdtBuilderConfig {
                type_name,
                fields: Vec::new(),
                replica_id_field: None,
            },
        }
    }
    
    /// Add a field to the CRDT
    pub fn add_field(mut self, name: String, strategy: CrdtStrategy) -> Self {
        self.config.fields.push(FieldConfig {
            name,
            strategy,
            optional: false,
            default: None,
        });
        self
    }
    
    /// Add an optional field with default value
    pub fn add_optional_field(mut self, name: String, strategy: CrdtStrategy, default: serde_json::Value) -> Self {
        self.config.fields.push(FieldConfig {
            name,
            strategy,
            optional: true,
            default: Some(default),
        });
        self
    }
    
    /// Set the replica ID field name
    pub fn replica_id_field(mut self, field_name: String) -> Self {
        self.config.replica_id_field = Some(field_name);
        self
    }
    
    /// Build the CRDT configuration
    pub fn build(self) -> CrdtBuilderConfig {
        self.config
    }
    
    /// Create a new CRDT instance
    pub fn create_crdt(self, replica_id: ReplicaId) -> CustomCrdt {
        CustomCrdt::new(self.config, replica_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::ReplicaId;
    use uuid::Uuid;
    
    #[test]
    fn test_crdt_builder_creation() {
        let config = CrdtBuilder::new("TestCRDT".to_string())
            .add_field("name".to_string(), CrdtStrategy::Lww)
            .add_field("count".to_string(), CrdtStrategy::GCounter)
            .add_optional_field("tags".to_string(), CrdtStrategy::AddWins, 
                serde_json::Value::Array(vec![]))
            .build();
        
        assert_eq!(config.type_name, "TestCRDT");
        assert_eq!(config.fields.len(), 3);
        assert_eq!(config.fields[0].name, "name");
        assert_eq!(config.fields[0].strategy, CrdtStrategy::Lww);
        assert_eq!(config.fields[1].name, "count");
        assert_eq!(config.fields[1].strategy, CrdtStrategy::GCounter);
        assert_eq!(config.fields[2].name, "tags");
        assert_eq!(config.fields[2].strategy, CrdtStrategy::AddWins);
        assert!(config.fields[2].optional);
    }
    
    #[test]
    fn test_custom_crdt_creation() {
        let replica_id = ReplicaId::from(Uuid::new_v4());
        let config = CrdtBuilder::new("TestCRDT".to_string())
            .add_field("name".to_string(), CrdtStrategy::Lww)
            .add_field("count".to_string(), CrdtStrategy::GCounter)
            .build();
        
        let crdt = CustomCrdt::new(config, replica_id.clone());
        
        assert_eq!(crdt.replica_id(), &replica_id);
        assert_eq!(crdt.field_names().len(), 2);
        assert!(crdt.get_field("name").is_some());
        assert!(crdt.get_field("count").is_some());
    }
    
    #[test]
    fn test_custom_crdt_field_operations() {
        let replica_id = ReplicaId::from(Uuid::new_v4());
        let config = CrdtBuilder::new("TestCRDT".to_string())
            .add_field("name".to_string(), CrdtStrategy::Lww)
            .add_field("count".to_string(), CrdtStrategy::GCounter)
            .build();
        
        let mut crdt = CustomCrdt::new(config, replica_id);
        
        // Set field values
        crdt.set_field("name", serde_json::Value::String("test".to_string())).unwrap();
        crdt.set_field("count", serde_json::Value::Number(serde_json::Number::from(42))).unwrap();
        
        // Get field values
        assert_eq!(crdt.get_field("name"), Some(&serde_json::Value::String("test".to_string())));
        assert_eq!(crdt.get_field("count"), Some(&serde_json::Value::Number(serde_json::Number::from(42))));
    }
    
    #[test]
    fn test_custom_crdt_merge() {
        let replica_id1 = ReplicaId::from(Uuid::new_v4());
        let replica_id2 = ReplicaId::from(Uuid::new_v4());
        
        let config = CrdtBuilder::new("TestCRDT".to_string())
            .add_field("name".to_string(), CrdtStrategy::Lww)
            .add_field("count".to_string(), CrdtStrategy::GCounter)
            .build();
        
        let mut crdt1 = CustomCrdt::new(config.clone(), replica_id1);
        let mut crdt2 = CustomCrdt::new(config, replica_id2);
        
        // Set different values with a small delay to ensure different timestamps
        crdt1.set_field("name", serde_json::Value::String("alice".to_string())).unwrap();
        crdt1.set_field("count", serde_json::Value::Number(serde_json::Number::from(10))).unwrap();
        
        // Small delay to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        crdt2.set_field("name", serde_json::Value::String("bob".to_string())).unwrap();
        crdt2.set_field("count", serde_json::Value::Number(serde_json::Number::from(20))).unwrap();
        
        // Merge crdt2 into crdt1
        crdt1.merge(&crdt2).unwrap();
        
        // Check merged values
        // Name should be "bob" (LWW with later timestamp)
        assert_eq!(crdt1.get_field("name"), Some(&serde_json::Value::String("bob".to_string())));
        // Count should be 20 (GCounter takes max)
        assert_eq!(crdt1.get_field("count"), Some(&serde_json::Value::Number(serde_json::Number::from(20))));
    }
    
    #[test]
    fn test_custom_crdt_conflict_detection() {
        let replica_id1 = ReplicaId::from(Uuid::new_v4());
        let replica_id2 = ReplicaId::from(Uuid::new_v4());
        
        let config = CrdtBuilder::new("TestCRDT".to_string())
            .add_field("name".to_string(), CrdtStrategy::Lww)
            .add_field("count".to_string(), CrdtStrategy::GCounter)
            .build();
        
        let mut crdt1 = CustomCrdt::new(config.clone(), replica_id1);
        let mut crdt2 = CustomCrdt::new(config, replica_id2);
        
        // Set same timestamp for LWW conflict
        crdt1.set_field("name", serde_json::Value::String("alice".to_string())).unwrap();
        crdt2.set_field("name", serde_json::Value::String("bob".to_string())).unwrap();
        
        // Manually set same timestamp to create conflict
        if let Some(field1) = crdt1.fields.get_mut("name") {
            field1.metadata.insert("timestamp".to_string(), serde_json::Value::Number(serde_json::Number::from(1000)));
        }
        if let Some(field2) = crdt2.fields.get_mut("name") {
            field2.metadata.insert("timestamp".to_string(), serde_json::Value::Number(serde_json::Number::from(1000)));
        }
        
        // Should detect conflict
        assert!(crdt1.has_conflict(&crdt2));
    }
    
    #[test]
    fn test_generic_field_merge_strategies() {
        // Test LWW merge
        let mut field1 = GenericCrdtField::new(
            "test".to_string(),
            serde_json::Value::String("alice".to_string()),
            CrdtStrategy::Lww,
        );
        let field2 = GenericCrdtField::new(
            "test".to_string(),
            serde_json::Value::String("bob".to_string()),
            CrdtStrategy::Lww,
        );
        
        // Set timestamps
        field1.metadata.insert("timestamp".to_string(), serde_json::Value::Number(serde_json::Number::from(1000)));
        let mut field2_with_timestamp = field2.clone();
        field2_with_timestamp.metadata.insert("timestamp".to_string(), serde_json::Value::Number(serde_json::Number::from(2000)));
        
        // Merge should take the later timestamp
        field1.merge(&field2_with_timestamp).unwrap();
        assert_eq!(field1.value, serde_json::Value::String("bob".to_string()));
        
        // Test GCounter merge
        let mut counter1 = GenericCrdtField::new(
            "count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(10)),
            CrdtStrategy::GCounter,
        );
        let counter2 = GenericCrdtField::new(
            "count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(20)),
            CrdtStrategy::GCounter,
        );
        
        counter1.merge(&counter2).unwrap();
        assert_eq!(counter1.value, serde_json::Value::Number(serde_json::Number::from(20)));
    }
    
    #[test]
    fn test_builder_error_handling() {
        let replica_id = ReplicaId::from(Uuid::new_v4());
        let config = CrdtBuilder::new("TestCRDT".to_string())
            .add_field("name".to_string(), CrdtStrategy::Lww)
            .build();
        
        let mut crdt = CustomCrdt::new(config, replica_id);
        
        // Test missing field error
        let result = crdt.set_field("nonexistent", serde_json::Value::String("test".to_string()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BuilderError::MissingField("nonexistent".to_string()));
    }
}
