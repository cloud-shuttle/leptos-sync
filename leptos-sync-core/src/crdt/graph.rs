use super::{CRDT, Mergeable, ReplicaId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use uuid::Uuid;

/// Custom error type for graph operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphError {
    message: String,
}

impl GraphError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GraphError: {}", self.message)
    }
}

impl Error for GraphError {}

/// Unique identifier for a graph vertex
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexId {
    /// Unique identifier for the vertex
    pub id: Uuid,
    /// Replica that created the vertex
    pub replica: ReplicaId,
}

impl VertexId {
    /// Create a new vertex ID
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            id: Uuid::new_v4(),
            replica,
        }
    }

    /// Create a vertex ID from existing UUID and replica
    pub fn from_parts(id: Uuid, replica: ReplicaId) -> Self {
        Self { id, replica }
    }
}

/// Unique identifier for a graph edge
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId {
    /// Unique identifier for the edge
    pub id: Uuid,
    /// Replica that created the edge
    pub replica: ReplicaId,
}

impl EdgeId {
    /// Create a new edge ID
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            id: Uuid::new_v4(),
            replica,
        }
    }

    /// Create an edge ID from existing UUID and replica
    pub fn from_parts(id: Uuid, replica: ReplicaId) -> Self {
        Self { id, replica }
    }
}

/// Metadata for a graph vertex
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexMetadata {
    /// When the vertex was created
    pub created_at: u64,
    /// When the vertex was last modified
    pub modified_at: u64,
    /// Whether the vertex is marked as deleted
    pub deleted: bool,
    /// Replica that last modified the vertex
    pub last_modified_by: ReplicaId,
}

impl VertexMetadata {
    /// Create new metadata
    pub fn new(replica: ReplicaId, timestamp: u64) -> Self {
        Self {
            created_at: timestamp,
            modified_at: timestamp,
            deleted: false,
            last_modified_by: replica,
        }
    }

    /// Mark as modified
    pub fn mark_modified(&mut self, replica: ReplicaId, timestamp: u64) {
        self.modified_at = timestamp;
        self.last_modified_by = replica;
    }

    /// Mark as deleted
    pub fn mark_deleted(&mut self, replica: ReplicaId, timestamp: u64) {
        self.deleted = true;
        self.mark_modified(replica, timestamp);
    }
}

/// Metadata for a graph edge
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeMetadata {
    /// When the edge was created
    pub created_at: u64,
    /// When the edge was last modified
    pub modified_at: u64,
    /// Whether the edge is marked as deleted
    pub deleted: bool,
    /// Replica that last modified the edge
    pub last_modified_by: ReplicaId,
}

impl EdgeMetadata {
    /// Create new metadata
    pub fn new(replica: ReplicaId, timestamp: u64) -> Self {
        Self {
            created_at: timestamp,
            modified_at: timestamp,
            deleted: false,
            last_modified_by: replica,
        }
    }

    /// Mark as modified
    pub fn mark_modified(&mut self, replica: ReplicaId, timestamp: u64) {
        self.modified_at = timestamp;
        self.last_modified_by = replica;
    }

    /// Mark as deleted
    pub fn mark_deleted(&mut self, replica: ReplicaId, timestamp: u64) {
        self.deleted = true;
        self.mark_modified(replica, timestamp);
    }
}

/// A graph vertex with its metadata
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Vertex<T> {
    /// Unique identifier
    pub id: VertexId,
    /// The actual value
    pub value: T,
    /// Metadata
    pub metadata: VertexMetadata,
}

impl<T> Vertex<T> {
    /// Create a new vertex
    pub fn new(value: T, replica: ReplicaId, timestamp: u64) -> Self {
        Self {
            id: VertexId::new(replica),
            value,
            metadata: VertexMetadata::new(replica, timestamp),
        }
    }

    /// Mark as modified
    pub fn mark_modified(&mut self, replica: ReplicaId, timestamp: u64) {
        self.metadata.mark_modified(replica, timestamp);
    }

    /// Mark as deleted
    pub fn mark_deleted(&mut self, replica: ReplicaId, timestamp: u64) {
        self.metadata.mark_deleted(replica, timestamp);
    }
}

/// A graph edge connecting two vertices
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    /// Unique identifier
    pub id: EdgeId,
    /// Source vertex ID
    pub source: VertexId,
    /// Target vertex ID
    pub target: VertexId,
    /// Optional edge weight
    pub weight: Option<f64>,
    /// Metadata
    pub metadata: EdgeMetadata,
}

impl Edge {
    /// Create a new edge
    pub fn new(source: VertexId, target: VertexId, replica: ReplicaId, timestamp: u64) -> Self {
        Self {
            id: EdgeId::new(replica),
            source,
            target,
            weight: None,
            metadata: EdgeMetadata::new(replica, timestamp),
        }
    }

    /// Create a new edge with weight
    pub fn with_weight(source: VertexId, target: VertexId, weight: f64, replica: ReplicaId, timestamp: u64) -> Self {
        Self {
            id: EdgeId::new(replica),
            source,
            target,
            weight: Some(weight),
            metadata: EdgeMetadata::new(replica, timestamp),
        }
    }

    /// Mark as modified
    pub fn mark_modified(&mut self, replica: ReplicaId, timestamp: u64) {
        self.metadata.mark_modified(replica, timestamp);
    }

    /// Mark as deleted
    pub fn mark_deleted(&mut self, replica: ReplicaId, timestamp: u64) {
        self.metadata.mark_deleted(replica, timestamp);
    }
}

/// Strategy for handling graph conflicts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphStrategy {
    /// Add-Wins: Vertices and edges are never removed, only marked as deleted
    AddWins,
    /// Remove-Wins: Deleted vertices and edges are completely removed
    RemoveWins,
}

/// Configuration for graph CRDTs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphConfig {
    /// Conflict resolution strategy
    pub strategy: GraphStrategy,
    /// Whether to preserve deleted vertices and edges in metadata
    pub preserve_deleted: bool,
    /// Maximum number of vertices
    pub max_vertices: Option<usize>,
    /// Maximum number of edges
    pub max_edges: Option<usize>,
    /// Whether to allow self-loops
    pub allow_self_loops: bool,
    /// Whether to allow multiple edges between the same vertices
    pub allow_multiple_edges: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            strategy: GraphStrategy::AddWins,
            preserve_deleted: true,
            max_vertices: None,
            max_edges: None,
            allow_self_loops: false,
            allow_multiple_edges: false,
        }
    }
}

/// Add-Wins Graph CRDT implementation
/// 
/// This implementation ensures that vertices and edges are never completely lost.
/// Deleted elements are marked as deleted but preserved for potential recovery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddWinsGraph<T> {
    /// Configuration
    config: GraphConfig,
    /// Vertices in the graph
    vertices: HashMap<VertexId, Vertex<T>>,
    /// Edges in the graph
    edges: HashMap<EdgeId, Edge>,
    /// Replica ID for this instance
    replica: ReplicaId,
}

impl<T: Clone + PartialEq + Eq + Send + Sync> AddWinsGraph<T> {
    /// Create a new Add-Wins graph
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            config: GraphConfig::default(),
            vertices: HashMap::new(),
            edges: HashMap::new(),
            replica,
        }
    }

    /// Create with custom configuration
    pub fn with_config(replica: ReplicaId, config: GraphConfig) -> Self {
        Self {
            config,
            vertices: HashMap::new(),
            edges: HashMap::new(),
            replica,
        }
    }

    /// Add a vertex to the graph
    pub fn add_vertex(&mut self, value: T, timestamp: u64) -> VertexId {
        let vertex = Vertex::new(value, self.replica, timestamp);
        let id = vertex.id.clone();
        self.vertices.insert(id.clone(), vertex);
        id
    }

    /// Add an edge between two vertices
    pub fn add_edge(&mut self, source: &VertexId, target: &VertexId, timestamp: u64, weight: Option<f64>) -> Result<EdgeId, GraphError> {
        // Check if vertices exist
        if !self.vertices.contains_key(source) || !self.vertices.contains_key(target) {
            return Err(GraphError::new("Source or target vertex not found".to_string()));
        }

        // Check for self-loops
        if !self.config.allow_self_loops && source == target {
            return Err(GraphError::new("Self-loops are not allowed".to_string()));
        }

        // Check for multiple edges if not allowed
        if !self.config.allow_multiple_edges {
            for edge in self.edges.values() {
                if !edge.metadata.deleted && edge.source == *source && edge.target == *target {
                    return Err(GraphError::new("Multiple edges between same vertices not allowed".to_string()));
                }
            }
        }

        let edge = if let Some(w) = weight {
            Edge::with_weight(source.clone(), target.clone(), w, self.replica, timestamp)
        } else {
            Edge::new(source.clone(), target.clone(), self.replica, timestamp)
        };
        
        let id = edge.id.clone();
        self.edges.insert(id.clone(), edge);
        Ok(id)
    }

    /// Update an existing vertex
    pub fn update_vertex(&mut self, id: &VertexId, value: T, timestamp: u64) -> Result<(), GraphError> {
        if let Some(vertex) = self.vertices.get_mut(id) {
            vertex.value = value;
            vertex.mark_modified(self.replica, timestamp);
            Ok(())
        } else {
            Err(GraphError::new("Vertex not found".to_string()))
        }
    }

    /// Update an existing edge
    pub fn update_edge(&mut self, id: &EdgeId, weight: f64, timestamp: u64) -> Result<(), GraphError> {
        if let Some(edge) = self.edges.get_mut(id) {
            edge.weight = Some(weight);
            edge.mark_modified(self.replica, timestamp);
            Ok(())
        } else {
            Err(GraphError::new("Edge not found".to_string()))
        }
    }

    /// Mark a vertex as deleted
    pub fn remove_vertex(&mut self, id: &VertexId, timestamp: u64) -> Result<(), GraphError> {
        if let Some(vertex) = self.vertices.get_mut(id) {
            vertex.mark_deleted(self.replica, timestamp);
            
            // Mark all incident edges as deleted
            for edge in self.edges.values_mut() {
                if !edge.metadata.deleted && (edge.source == *id || edge.target == *id) {
                    edge.mark_deleted(self.replica, timestamp);
                }
            }
            Ok(())
        } else {
            Err(GraphError::new("Vertex not found".to_string()))
        }
    }

    /// Mark an edge as deleted
    pub fn remove_edge(&mut self, id: &EdgeId, timestamp: u64) -> Result<(), GraphError> {
        if let Some(edge) = self.edges.get_mut(id) {
            edge.mark_deleted(self.replica, timestamp);
            Ok(())
        } else {
            Err(GraphError::new("Edge not found".to_string()))
        }
    }

    /// Get a vertex by ID
    pub fn get_vertex(&self, id: &VertexId) -> Option<&Vertex<T>> {
        self.vertices.get(id)
    }

    /// Get an edge by ID
    pub fn get_edge(&self, id: &EdgeId) -> Option<&Edge> {
        self.edges.get(id)
    }

    /// Get all visible vertices (not deleted)
    pub fn visible_vertices(&self) -> Vec<&Vertex<T>> {
        self.vertices
            .values()
            .filter(|v| !v.metadata.deleted)
            .collect()
    }

    /// Get all visible edges (not deleted)
    pub fn visible_edges(&self) -> Vec<&Edge> {
        self.edges
            .values()
            .filter(|e| !e.metadata.deleted)
            .collect()
    }

    /// Get all vertices including deleted ones
    pub fn all_vertices(&self) -> Vec<&Vertex<T>> {
        self.vertices.values().collect()
    }

    /// Get all edges including deleted ones
    pub fn all_edges(&self) -> Vec<&Edge> {
        self.edges.values().collect()
    }

    /// Get neighbors of a vertex
    pub fn neighbors(&self, id: &VertexId) -> Vec<&Vertex<T>> {
        let mut neighbors = Vec::new();
        
        for edge in self.edges.values() {
            if !edge.metadata.deleted {
                if edge.source == *id {
                    if let Some(target) = self.vertices.get(&edge.target) {
                        if !target.metadata.deleted {
                            neighbors.push(target);
                        }
                    }
                } else if edge.target == *id {
                    if let Some(source) = self.vertices.get(&edge.source) {
                        if !source.metadata.deleted {
                            neighbors.push(source);
                        }
                    }
                }
            }
        }
        
        neighbors
    }

    /// Get incoming edges to a vertex
    pub fn incoming_edges(&self, id: &VertexId) -> Vec<&Edge> {
        self.edges
            .values()
            .filter(|e| !e.metadata.deleted && e.target == *id)
            .collect()
    }

    /// Get outgoing edges from a vertex
    pub fn outgoing_edges(&self, id: &VertexId) -> Vec<&Edge> {
        self.edges
            .values()
            .filter(|e| !e.metadata.deleted && e.source == *id)
            .collect()
    }

    /// Find shortest path between two vertices using BFS
    pub fn shortest_path(&self, source: &VertexId, target: &VertexId) -> Option<Vec<VertexId>> {
        if !self.vertices.contains_key(source) || !self.vertices.contains_key(target) {
            return None;
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<VertexId, VertexId> = HashMap::new();
        
        queue.push_back(source.clone());
        visited.insert(source.clone());
        
        while let Some(current) = queue.pop_front() {
            if current == *target {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current_id = current;
                
                while current_id != *source {
                    path.push(current_id.clone());
                    current_id = parent[&current_id].clone();
                }
                path.push(source.clone());
                path.reverse();
                return Some(path);
            }
            
            for neighbor in self.neighbors(&current) {
                if !visited.contains(&neighbor.id) {
                    visited.insert(neighbor.id.clone());
                    parent.insert(neighbor.id.clone(), current.clone());
                    queue.push_back(neighbor.id.clone());
                }
            }
        }
        
        None
    }

    /// Check if the graph contains a vertex
    pub fn contains_vertex(&self, id: &VertexId) -> bool {
        self.vertices.contains_key(id)
    }

    /// Check if the graph contains an edge
    pub fn contains_edge(&self, id: &EdgeId) -> bool {
        self.edges.contains_key(id)
    }

    /// Get the number of visible vertices
    pub fn vertex_count(&self) -> usize {
        self.visible_vertices().len()
    }

    /// Get the number of visible edges
    pub fn edge_count(&self) -> usize {
        self.visible_edges().len()
    }

    /// Check if the graph is empty
    pub fn is_empty(&self) -> bool {
        self.vertex_count() == 0
    }

    /// Clear all vertices and edges
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.edges.clear();
    }
}

impl<T: Clone + PartialEq + Eq + Send + Sync> CRDT for AddWinsGraph<T> {
    fn replica_id(&self) -> &ReplicaId {
        &self.replica
    }
}

impl<T: Clone + PartialEq + Eq + Send + Sync> Mergeable for AddWinsGraph<T> {
    type Error = GraphError;

    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        // Merge vertices
        for (id, vertex) in &other.vertices {
            match self.vertices.get(id) {
                Some(existing) => {
                    // Conflict resolution: later timestamp wins
                    if vertex.metadata.modified_at > existing.metadata.modified_at {
                        self.vertices.insert(id.clone(), vertex.clone());
                    }
                }
                None => {
                    // New vertex, add it
                    self.vertices.insert(id.clone(), vertex.clone());
                }
            }
        }

        // Merge edges
        for (id, edge) in &other.edges {
            match self.edges.get(id) {
                Some(existing) => {
                    // Conflict resolution: later timestamp wins
                    if edge.metadata.modified_at > existing.metadata.modified_at {
                        self.edges.insert(id.clone(), edge.clone());
                    }
                }
                None => {
                    // New edge, add it
                    self.edges.insert(id.clone(), edge.clone());
                }
            }
        }
        
        Ok(())
    }

    fn has_conflict(&self, other: &Self) -> bool {
        // Check for conflicts in overlapping vertices
        for (id, vertex) in &other.vertices {
            if let Some(existing) = self.vertices.get(id) {
                if vertex.metadata.modified_at == existing.metadata.modified_at
                    && vertex.metadata.last_modified_by != existing.metadata.last_modified_by
                {
                    return true;
                }
            }
        }

        // Check for conflicts in overlapping edges
        for (id, edge) in &other.edges {
            if let Some(existing) = self.edges.get(id) {
                if edge.metadata.modified_at == existing.metadata.modified_at
                    && edge.metadata.last_modified_by != existing.metadata.last_modified_by
                {
                    return true;
                }
            }
        }
        false
    }
}

/// Remove-Wins Graph CRDT implementation
/// 
/// This implementation completely removes deleted vertices and edges.
/// It's more memory-efficient but elements cannot be recovered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveWinsGraph<T> {
    /// Configuration
    config: GraphConfig,
    /// Vertices in the graph
    vertices: HashMap<VertexId, Vertex<T>>,
    /// Edges in the graph
    edges: HashMap<EdgeId, Edge>,
    /// Replica ID for this instance
    replica: ReplicaId,
}

impl<T: Clone + PartialEq + Eq + Send + Sync> RemoveWinsGraph<T> {
    /// Create a new Remove-Wins graph
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            config: GraphConfig {
                strategy: GraphStrategy::RemoveWins,
                preserve_deleted: false,
                max_vertices: None,
                max_edges: None,
                allow_self_loops: false,
                allow_multiple_edges: false,
            },
            vertices: HashMap::new(),
            edges: HashMap::new(),
            replica,
        }
    }

    /// Create with custom configuration
    pub fn with_config(replica: ReplicaId, config: GraphConfig) -> Self {
        Self {
            config,
            vertices: HashMap::new(),
            edges: HashMap::new(),
            replica,
        }
    }

    /// Add a vertex to the graph
    pub fn add_vertex(&mut self, value: T, timestamp: u64) -> VertexId {
        let vertex = Vertex::new(value, self.replica, timestamp);
        let id = vertex.id.clone();
        self.vertices.insert(id.clone(), vertex);
        id
    }

    /// Add an edge between two vertices
    pub fn add_edge(&mut self, source: &VertexId, target: &VertexId, timestamp: u64, weight: Option<f64>) -> Result<EdgeId, GraphError> {
        // Check if vertices exist
        if !self.vertices.contains_key(source) || !self.vertices.contains_key(target) {
            return Err(GraphError::new("Source or target vertex not found".to_string()));
        }

        // Check for self-loops
        if !self.config.allow_self_loops && source == target {
            return Err(GraphError::new("Self-loops are not allowed".to_string()));
        }

        // Check for multiple edges if not allowed
        if !self.config.allow_multiple_edges {
            for edge in self.edges.values() {
                if edge.source == *source && edge.target == *target {
                    return Err(GraphError::new("Multiple edges between same vertices not allowed".to_string()));
                }
            }
        }

        let edge = if let Some(w) = weight {
            Edge::with_weight(source.clone(), target.clone(), w, self.replica, timestamp)
        } else {
            Edge::new(source.clone(), target.clone(), self.replica, timestamp)
        };
        
        let id = edge.id.clone();
        self.edges.insert(id.clone(), edge);
        Ok(id)
    }

    /// Update an existing vertex
    pub fn update_vertex(&mut self, id: &VertexId, value: T, timestamp: u64) -> Result<(), GraphError> {
        if let Some(vertex) = self.vertices.get_mut(id) {
            vertex.value = value;
            vertex.mark_modified(self.replica, timestamp);
            Ok(())
        } else {
            Err(GraphError::new("Vertex not found".to_string()))
        }
    }

    /// Update an existing edge
    pub fn update_edge(&mut self, id: &EdgeId, weight: f64, timestamp: u64) -> Result<(), GraphError> {
        if let Some(edge) = self.edges.get_mut(id) {
            edge.weight = Some(weight);
            edge.mark_modified(self.replica, timestamp);
            Ok(())
        } else {
            Err(GraphError::new("Edge not found".to_string()))
        }
    }

    /// Remove a vertex completely
    pub fn remove_vertex(&mut self, id: &VertexId) -> Result<(), GraphError> {
        if self.vertices.remove(id).is_some() {
            // Remove all incident edges
            self.edges.retain(|_, edge| edge.source != *id && edge.target != *id);
            Ok(())
        } else {
            Err(GraphError::new("Vertex not found".to_string()))
        }
    }

    /// Remove an edge completely
    pub fn remove_edge(&mut self, id: &EdgeId) -> Result<(), GraphError> {
        if self.edges.remove(id).is_some() {
            Ok(())
        } else {
            Err(GraphError::new("Edge not found".to_string()))
        }
    }

    /// Get a vertex by ID
    pub fn get_vertex(&self, id: &VertexId) -> Option<&Vertex<T>> {
        self.vertices.get(id)
    }

    /// Get an edge by ID
    pub fn get_edge(&self, id: &EdgeId) -> Option<&Edge> {
        self.edges.get(id)
    }

    /// Get all vertices
    pub fn vertices(&self) -> Vec<&Vertex<T>> {
        self.vertices.values().collect()
    }

    /// Get all edges
    pub fn edges(&self) -> Vec<&Edge> {
        self.edges.values().collect()
    }

    /// Get neighbors of a vertex
    pub fn neighbors(&self, id: &VertexId) -> Vec<&Vertex<T>> {
        let mut neighbors = Vec::new();
        
        for edge in self.edges.values() {
            if edge.source == *id {
                if let Some(target) = self.vertices.get(&edge.target) {
                    neighbors.push(target);
                }
            } else if edge.target == *id {
                if let Some(source) = self.vertices.get(&edge.source) {
                    neighbors.push(source);
                }
            }
        }
        
        neighbors
    }

    /// Get incoming edges to a vertex
    pub fn incoming_edges(&self, id: &VertexId) -> Vec<&Edge> {
        self.edges
            .values()
            .filter(|e| e.target == *id)
            .collect()
    }

    /// Get outgoing edges from a vertex
    pub fn outgoing_edges(&self, id: &VertexId) -> Vec<&Edge> {
        self.edges
            .values()
            .filter(|e| e.source == *id)
            .collect()
    }

    /// Find shortest path between two vertices using BFS
    pub fn shortest_path(&self, source: &VertexId, target: &VertexId) -> Option<Vec<VertexId>> {
        if !self.vertices.contains_key(source) || !self.vertices.contains_key(target) {
            return None;
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<VertexId, VertexId> = HashMap::new();
        
        queue.push_back(source.clone());
        visited.insert(source.clone());
        
        while let Some(current) = queue.pop_front() {
            if current == *target {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current_id = current;
                
                while current_id != *source {
                    path.push(current_id.clone());
                    current_id = parent[&current_id].clone();
                }
                path.push(source.clone());
                path.reverse();
                return Some(path);
            }
            
            for neighbor in self.neighbors(&current) {
                if !visited.contains(&neighbor.id) {
                    visited.insert(neighbor.id.clone());
                    parent.insert(neighbor.id.clone(), current.clone());
                    queue.push_back(neighbor.id.clone());
                }
            }
        }
        
        None
    }

    /// Check if the graph contains a vertex
    pub fn contains_vertex(&self, id: &VertexId) -> bool {
        self.vertices.contains_key(id)
    }

    /// Check if the graph contains an edge
    pub fn contains_edge(&self, id: &EdgeId) -> bool {
        self.edges.contains_key(id)
    }

    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get the number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Check if the graph is empty
    pub fn is_empty(&self) -> bool {
        self.vertex_count() == 0
    }

    /// Clear all vertices and edges
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.edges.clear();
    }
}

impl<T: Clone + PartialEq + Eq + Send + Sync> CRDT for RemoveWinsGraph<T> {
    fn replica_id(&self) -> &ReplicaId {
        &self.replica
    }
}

impl<T: Clone + PartialEq + Eq + Send + Sync> Mergeable for RemoveWinsGraph<T> {
    type Error = GraphError;

    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        // Merge vertices
        for (id, vertex) in &other.vertices {
            match self.vertices.get(id) {
                Some(existing) => {
                    // Conflict resolution: later timestamp wins
                    if vertex.metadata.modified_at > existing.metadata.modified_at {
                        self.vertices.insert(id.clone(), vertex.clone());
                    }
                }
                None => {
                    // New vertex, add it
                    self.vertices.insert(id.clone(), vertex.clone());
                }
            }
        }

        // Merge edges
        for (id, edge) in &other.edges {
            match self.edges.get(id) {
                Some(existing) => {
                    // Conflict resolution: later timestamp wins
                    if edge.metadata.modified_at > existing.metadata.modified_at {
                        self.edges.insert(id.clone(), edge.clone());
                    }
                }
                None => {
                    // New edge, add it
                    self.edges.insert(id.clone(), edge.clone());
                }
            }
        }
        
        Ok(())
    }

    fn has_conflict(&self, other: &Self) -> bool {
        // Check for conflicts in overlapping vertices
        for (id, vertex) in &other.vertices {
            if let Some(existing) = self.vertices.get(id) {
                if vertex.metadata.modified_at == existing.metadata.modified_at
                    && vertex.metadata.last_modified_by != existing.metadata.last_modified_by
                {
                    return true;
                }
            }
        }

        // Check for conflicts in overlapping edges
        for (id, edge) in &other.edges {
            if let Some(existing) = self.edges.get(id) {
                if edge.metadata.modified_at == existing.metadata.modified_at
                    && edge.metadata.last_modified_by != existing.metadata.last_modified_by
                {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ReplicaId;
    use uuid::Uuid;

    fn create_replica(id: u64) -> ReplicaId {
        ReplicaId::from(Uuid::from_u64_pair(0, id))
    }

    #[test]
    fn test_vertex_id_creation() {
        let replica = create_replica(1);
        let vertex_id = VertexId::new(replica);
        
        assert_eq!(vertex_id.replica, replica);
        assert_ne!(vertex_id.id, Uuid::nil());
    }

    #[test]
    fn test_edge_id_creation() {
        let replica = create_replica(1);
        let edge_id = EdgeId::new(replica);
        
        assert_eq!(edge_id.replica, replica);
        assert_ne!(edge_id.id, Uuid::nil());
    }

    #[test]
    fn test_vertex_creation() {
        let replica = create_replica(1);
        let timestamp = 1234567890;
        let vertex = Vertex::new("test_value", replica, timestamp);
        
        assert_eq!(vertex.value, "test_value");
        assert_eq!(vertex.metadata.created_at, timestamp);
        assert_eq!(vertex.metadata.modified_at, timestamp);
        assert_eq!(vertex.metadata.deleted, false);
        assert_eq!(vertex.metadata.last_modified_by, replica);
    }

    #[test]
    fn test_edge_creation() {
        let replica = create_replica(1);
        let timestamp = 1234567890;
        let source = VertexId::new(replica);
        let target = VertexId::new(replica);
        let edge = Edge::new(source.clone(), target.clone(), replica, timestamp);
        
        assert_eq!(edge.source, source);
        assert_eq!(edge.target, target);
        assert_eq!(edge.weight, None);
        assert_eq!(edge.metadata.created_at, timestamp);
        assert_eq!(edge.metadata.deleted, false);
    }

    #[test]
    fn test_edge_with_weight() {
        let replica = create_replica(1);
        let timestamp = 1234567890;
        let source = VertexId::new(replica);
        let target = VertexId::new(replica);
        let weight = 5.5;
        let edge = Edge::with_weight(source.clone(), target.clone(), weight, replica, timestamp);
        
        assert_eq!(edge.weight, Some(weight));
    }

    #[test]
    fn test_add_wins_graph_basic_operations() {
        let replica = create_replica(1);
        let mut graph = AddWinsGraph::new(replica);
        
        // Add vertices
        let v1_id = graph.add_vertex("vertex1", 1000);
        let v2_id = graph.add_vertex("vertex2", 2000);
        
        assert_eq!(graph.vertex_count(), 2);
        assert!(graph.contains_vertex(&v1_id));
        assert!(graph.contains_vertex(&v2_id));
        
        // Add edge
        let edge_id = graph.add_edge(&v1_id, &v2_id, 3000, None).unwrap();
        assert_eq!(graph.edge_count(), 1);
        assert!(graph.contains_edge(&edge_id));
        
        // Update vertex
        graph.update_vertex(&v1_id, "updated_vertex1", 4000).unwrap();
        assert_eq!(graph.get_vertex(&v1_id).unwrap().value, "updated_vertex1");
    }

    #[test]
    fn test_remove_wins_graph_basic_operations() {
        let replica = create_replica(1);
        let mut graph = RemoveWinsGraph::new(replica);
        
        // Add vertices and edge
        let v1_id = graph.add_vertex("vertex1", 1000);
        let v2_id = graph.add_vertex("vertex2", 2000);
        let edge_id = graph.add_edge(&v1_id, &v2_id, 3000, None).unwrap();
        
        assert_eq!(graph.vertex_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        
        // Remove edge completely
        graph.remove_edge(&edge_id).unwrap();
        assert_eq!(graph.edge_count(), 0);
        assert!(!graph.contains_edge(&edge_id));
        
        // Remove vertex completely
        graph.remove_vertex(&v1_id).unwrap();
        assert_eq!(graph.vertex_count(), 1);
        assert!(!graph.contains_vertex(&v1_id));
    }

    #[test]
    fn test_graph_neighbors() {
        let replica = create_replica(1);
        let mut graph = AddWinsGraph::new(replica);
        
        // Create triangle: v1 -> v2 -> v3 -> v1
        let v1_id = graph.add_vertex("vertex1", 1000);
        let v2_id = graph.add_vertex("vertex2", 2000);
        let v3_id = graph.add_vertex("vertex3", 3000);
        
        graph.add_edge(&v1_id, &v2_id, 4000, None).unwrap();
        graph.add_edge(&v2_id, &v3_id, 5000, None).unwrap();
        graph.add_edge(&v3_id, &v1_id, 6000, None).unwrap();
        
        // Check neighbors
        let v1_neighbors = graph.neighbors(&v1_id);
        assert_eq!(v1_neighbors.len(), 2); // v2 and v3
        
        let v2_neighbors = graph.neighbors(&v2_id);
        assert_eq!(v2_neighbors.len(), 2); // v1 and v3
        
        let v3_neighbors = graph.neighbors(&v3_id);
        assert_eq!(v3_neighbors.len(), 2); // v1 and v2
    }

    #[test]
    fn test_graph_shortest_path() {
        let replica = create_replica(1);
        let mut graph = AddWinsGraph::new(replica);
        
        // Create path: v1 -> v2 -> v3 -> v4
        let v1_id = graph.add_vertex("vertex1", 1000);
        let v2_id = graph.add_vertex("vertex2", 2000);
        let v3_id = graph.add_vertex("vertex3", 3000);
        let v4_id = graph.add_vertex("vertex4", 4000);
        
        graph.add_edge(&v1_id, &v2_id, 5000, None).unwrap();
        graph.add_edge(&v2_id, &v3_id, 6000, None).unwrap();
        graph.add_edge(&v3_id, &v4_id, 7000, None).unwrap();
        
        // Find shortest path
        let path = graph.shortest_path(&v1_id, &v4_id).unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], v1_id);
        assert_eq!(path[1], v2_id);
        assert_eq!(path[2], v3_id);
        assert_eq!(path[3], v4_id);
    }

    #[test]
    fn test_graph_merge() {
        let replica1 = create_replica(1);
        let replica2 = create_replica(2);
        
        let mut graph1 = AddWinsGraph::new(replica1);
        let mut graph2 = AddWinsGraph::new(replica2);
        
        // Add vertices to both graphs
        let v1_id = graph1.add_vertex("vertex1", 1000);
        let v2_id = graph2.add_vertex("vertex2", 2000);
        
        // Merge graph2 into graph1
        graph1.merge(&graph2).unwrap();
        
        // Both vertices should be present
        assert_eq!(graph1.vertex_count(), 2);
        assert!(graph1.contains_vertex(&v1_id));
        assert!(graph1.contains_vertex(&v2_id));
    }

    #[test]
    fn test_graph_configuration() {
        let replica = create_replica(1);
        let config = GraphConfig {
            strategy: GraphStrategy::RemoveWins,
            preserve_deleted: false,
            max_vertices: Some(100),
            max_edges: Some(200),
            allow_self_loops: true,
            allow_multiple_edges: true,
        };
        
        let graph: AddWinsGraph<String> = AddWinsGraph::with_config(replica, config);
        assert_eq!(graph.config.strategy, GraphStrategy::RemoveWins);
        assert_eq!(graph.config.max_vertices, Some(100));
        assert_eq!(graph.config.allow_self_loops, true);
        assert_eq!(graph.config.allow_multiple_edges, true);
    }

    #[test]
    fn test_graph_validation() {
        let replica = create_replica(1);
        let mut graph = AddWinsGraph::new(replica);
        
        // Add vertices
        let v1_id = graph.add_vertex("vertex1", 1000);
        let v2_id = graph.add_vertex("vertex2", 2000);
        
        // Try to add edge with non-existent vertex
        let fake_id = VertexId::new(create_replica(999));
        let result = graph.add_edge(&v1_id, &fake_id, 3000, None);
        assert!(result.is_err());
        
        // Try to add edge to itself (self-loop not allowed by default)
        let result = graph.add_edge(&v1_id, &v1_id, 3000, None);
        assert!(result.is_err());
    }
}