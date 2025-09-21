//! Remove-Wins Graph CRDT implementation
//!
//! This implementation completely removes deleted vertices and edges.
//! It's more memory-efficient but elements cannot be recovered.

use super::super::{CRDT, Mergeable, ReplicaId};
use super::add_wins::GraphConfig;
use super::edge::{Edge, EdgeId};
use super::vertex::{GraphError, Vertex, VertexId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Remove-Wins Graph CRDT implementation
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
    pub fn add_edge(
        &mut self,
        source: &VertexId,
        target: &VertexId,
        timestamp: u64,
        weight: Option<f64>,
    ) -> Result<EdgeId, GraphError> {
        // Check if vertices exist
        if !self.vertices.contains_key(source) || !self.vertices.contains_key(target) {
            return Err(GraphError::new(
                "Source or target vertex not found".to_string(),
            ));
        }

        // Check for self-loops
        if !self.config.allow_self_loops && source == target {
            return Err(GraphError::new("Self-loops are not allowed".to_string()));
        }

        // Check for multiple edges if not allowed
        if !self.config.allow_multiple_edges {
            for edge in self.edges.values() {
                if edge.source == *source && edge.target == *target {
                    return Err(GraphError::new(
                        "Multiple edges between same vertices not allowed".to_string(),
                    ));
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
    pub fn update_vertex(
        &mut self,
        id: &VertexId,
        value: T,
        timestamp: u64,
    ) -> Result<(), GraphError> {
        if let Some(vertex) = self.vertices.get_mut(id) {
            vertex.value = value;
            vertex.mark_modified(self.replica, timestamp);
            Ok(())
        } else {
            Err(GraphError::new("Vertex not found".to_string()))
        }
    }

    /// Update an existing edge
    pub fn update_edge(
        &mut self,
        id: &EdgeId,
        weight: f64,
        timestamp: u64,
    ) -> Result<(), GraphError> {
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
            self.edges
                .retain(|_, edge| edge.source != *id && edge.target != *id);
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
        self.edges.values().filter(|e| e.target == *id).collect()
    }

    /// Get outgoing edges from a vertex
    pub fn outgoing_edges(&self, id: &VertexId) -> Vec<&Edge> {
        self.edges.values().filter(|e| e.source == *id).collect()
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
    use super::super::super::ReplicaId;
    use super::*;
    use uuid::Uuid;

    fn create_replica(id: u64) -> ReplicaId {
        ReplicaId::from(Uuid::from_u64_pair(0, id))
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
    fn test_remove_wins_graph_merge() {
        let replica1 = create_replica(1);
        let replica2 = create_replica(2);

        let mut graph1 = RemoveWinsGraph::new(replica1);
        let mut graph2 = RemoveWinsGraph::new(replica2);

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
}
