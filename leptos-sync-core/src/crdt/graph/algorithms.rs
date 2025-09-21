//! Graph algorithms and utilities

use super::edge::Edge;
use super::vertex::{Vertex, VertexId};
use std::collections::{HashMap, HashSet, VecDeque};

/// Graph traversal algorithms
pub struct GraphAlgorithms;

impl GraphAlgorithms {
    /// Find shortest path between two vertices using BFS
    pub fn shortest_path<T>(
        vertices: &HashMap<VertexId, Vertex<T>>,
        edges: &HashMap<super::edge::EdgeId, Edge>,
        source: &VertexId,
        target: &VertexId,
    ) -> Option<Vec<VertexId>> {
        if !vertices.contains_key(source) || !vertices.contains_key(target) {
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

            for neighbor in Self::get_neighbors(vertices, edges, &current) {
                if !visited.contains(&neighbor.id) {
                    visited.insert(neighbor.id.clone());
                    parent.insert(neighbor.id.clone(), current.clone());
                    queue.push_back(neighbor.id.clone());
                }
            }
        }

        None
    }

    /// Get neighbors of a vertex (considering only visible elements)
    pub fn get_neighbors<'a, T>(
        vertices: &'a HashMap<VertexId, Vertex<T>>,
        edges: &'a HashMap<super::edge::EdgeId, Edge>,
        id: &'a VertexId,
    ) -> Vec<&'a Vertex<T>> {
        let mut neighbors = Vec::new();

        for edge in edges.values() {
            if !edge.metadata.deleted {
                if edge.source == *id {
                    if let Some(target) = vertices.get(&edge.target) {
                        if !target.metadata.deleted {
                            neighbors.push(target);
                        }
                    }
                } else if edge.target == *id {
                    if let Some(source) = vertices.get(&edge.source) {
                        if !source.metadata.deleted {
                            neighbors.push(source);
                        }
                    }
                }
            }
        }

        neighbors
    }

    /// Get incoming edges to a vertex (considering only visible elements)
    pub fn get_incoming_edges<'a>(
        edges: &'a HashMap<super::edge::EdgeId, Edge>,
        id: &'a VertexId,
    ) -> Vec<&'a Edge> {
        edges
            .values()
            .filter(|e| !e.metadata.deleted && e.target == *id)
            .collect()
    }

    /// Get outgoing edges from a vertex (considering only visible elements)
    pub fn get_outgoing_edges<'a>(
        edges: &'a HashMap<super::edge::EdgeId, Edge>,
        id: &'a VertexId,
    ) -> Vec<&'a Edge> {
        edges
            .values()
            .filter(|e| !e.metadata.deleted && e.source == *id)
            .collect()
    }

    /// Check if graph is connected (all visible vertices reachable)
    pub fn is_connected<T>(
        vertices: &HashMap<VertexId, Vertex<T>>,
        edges: &HashMap<super::edge::EdgeId, Edge>,
    ) -> bool {
        let visible_vertices: Vec<&VertexId> = vertices
            .values()
            .filter(|v| !v.metadata.deleted)
            .map(|v| &v.id)
            .collect();

        if visible_vertices.is_empty() {
            return true; // Empty graph is considered connected
        }

        let start = &visible_vertices[0];
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start.clone());
        visited.insert(start.clone());

        while let Some(current) = queue.pop_front() {
            for neighbor in Self::get_neighbors(vertices, edges, &current) {
                if !visited.contains(&neighbor.id) {
                    visited.insert(&neighbor.id);
                    queue.push_back(&neighbor.id);
                }
            }
        }

        // Check if all visible vertices were visited
        visible_vertices.iter().all(|v| visited.contains(v))
    }

    /// Find all connected components
    pub fn connected_components<T>(
        vertices: &HashMap<VertexId, Vertex<T>>,
        edges: &HashMap<super::edge::EdgeId, Edge>,
    ) -> Vec<Vec<VertexId>> {
        let mut components = Vec::new();
        let mut visited = HashSet::new();

        for vertex in vertices.values() {
            if !vertex.metadata.deleted && !visited.contains(&vertex.id) {
                let mut component = Vec::new();
                let mut queue = VecDeque::new();

                queue.push_back(vertex.id.clone());
                visited.insert(vertex.id.clone());
                component.push(vertex.id.clone());

                while let Some(current) = queue.pop_front() {
                    for neighbor in Self::get_neighbors(vertices, edges, &current) {
                        if !visited.contains(&neighbor.id) {
                            visited.insert(neighbor.id.clone());
                            queue.push_back(neighbor.id.clone());
                            component.push(neighbor.id.clone());
                        }
                    }
                }

                components.push(component);
            }
        }

        components
    }

    /// Calculate graph density (ratio of edges to maximum possible edges)
    pub fn density<T>(
        vertices: &HashMap<VertexId, Vertex<T>>,
        edges: &HashMap<super::edge::EdgeId, Edge>,
    ) -> f64 {
        let visible_vertices: Vec<_> = vertices.values().filter(|v| !v.metadata.deleted).collect();

        let visible_edges: Vec<_> = edges.values().filter(|e| !e.metadata.deleted).collect();

        let n = visible_vertices.len();
        if n <= 1 {
            return 0.0;
        }

        let max_edges = n * (n - 1) / 2; // For undirected graph
        visible_edges.len() as f64 / max_edges as f64
    }

    /// Find vertices with no incoming edges (sources)
    pub fn find_sources<'a, T>(
        vertices: &'a HashMap<VertexId, Vertex<T>>,
        edges: &'a HashMap<super::edge::EdgeId, Edge>,
    ) -> Vec<&'a VertexId> {
        let mut sources = Vec::new();

        for vertex in vertices.values() {
            if !vertex.metadata.deleted {
                let has_incoming = edges
                    .values()
                    .any(|e| !e.metadata.deleted && e.target == vertex.id);

                if !has_incoming {
                    sources.push(&vertex.id);
                }
            }
        }

        sources
    }

    /// Find vertices with no outgoing edges (sinks)
    pub fn find_sinks<'a, T>(
        vertices: &'a HashMap<VertexId, Vertex<T>>,
        edges: &'a HashMap<super::edge::EdgeId, Edge>,
    ) -> Vec<&'a VertexId> {
        let mut sinks = Vec::new();

        for vertex in vertices.values() {
            if !vertex.metadata.deleted {
                let has_outgoing = edges
                    .values()
                    .any(|e| !e.metadata.deleted && e.source == vertex.id);

                if !has_outgoing {
                    sinks.push(&vertex.id);
                }
            }
        }

        sinks
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::ReplicaId;
    use super::super::edge::EdgeId;
    use super::*;
    use uuid::Uuid;

    fn create_replica(id: u64) -> ReplicaId {
        ReplicaId::from(Uuid::from_u64_pair(0, id))
    }

    #[test]
    fn test_shortest_path() {
        let replica = create_replica(1);
        let mut vertices = HashMap::new();
        let mut edges = HashMap::new();

        // Create path: v1 -> v2 -> v3 -> v4
        let v1 = Vertex::new("vertex1", replica, 1000);
        let v2 = Vertex::new("vertex2", replica, 2000);
        let v3 = Vertex::new("vertex3", replica, 3000);
        let v4 = Vertex::new("vertex4", replica, 4000);

        let v1_id = v1.id.clone();
        let v2_id = v2.id.clone();
        let v3_id = v3.id.clone();
        let v4_id = v4.id.clone();

        vertices.insert(v1_id.clone(), v1);
        vertices.insert(v2_id.clone(), v2);
        vertices.insert(v3_id.clone(), v3);
        vertices.insert(v4_id.clone(), v4);

        let e1 = Edge::new(v1_id.clone(), v2_id.clone(), replica, 5000);
        let e2 = Edge::new(v2_id.clone(), v3_id.clone(), replica, 6000);
        let e3 = Edge::new(v3_id.clone(), v4_id.clone(), replica, 7000);

        edges.insert(e1.id.clone(), e1);
        edges.insert(e2.id.clone(), e2);
        edges.insert(e3.id.clone(), e3);

        // Find shortest path
        let path = GraphAlgorithms::shortest_path(&vertices, &edges, &v1_id, &v4_id).unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], v1_id);
        assert_eq!(path[1], v2_id);
        assert_eq!(path[2], v3_id);
        assert_eq!(path[3], v4_id);
    }

    #[test]
    fn test_connected_components() {
        let replica = create_replica(1);
        let mut vertices = HashMap::new();
        let mut edges = HashMap::new();

        // Create two disconnected components
        let v1 = Vertex::new("vertex1", replica, 1000);
        let v2 = Vertex::new("vertex2", replica, 2000);
        let v3 = Vertex::new("vertex3", replica, 3000);
        let v4 = Vertex::new("vertex4", replica, 4000);

        let v1_id = v1.id.clone();
        let v2_id = v2.id.clone();
        let v3_id = v3.id.clone();
        let v4_id = v4.id.clone();

        vertices.insert(v1_id.clone(), v1);
        vertices.insert(v2_id.clone(), v2);
        vertices.insert(v3_id.clone(), v3);
        vertices.insert(v4_id.clone(), v4);

        // Connect v1-v2 and v3-v4 separately
        let e1 = Edge::new(v1_id.clone(), v2_id.clone(), replica, 5000);
        let e2 = Edge::new(v3_id.clone(), v4_id.clone(), replica, 6000);

        edges.insert(e1.id.clone(), e1);
        edges.insert(e2.id.clone(), e2);

        let components = GraphAlgorithms::connected_components(&vertices, &edges);
        assert_eq!(components.len(), 2);

        // Each component should have 2 vertices
        assert_eq!(components[0].len(), 2);
        assert_eq!(components[1].len(), 2);
    }

    #[test]
    fn test_graph_density() {
        let replica = create_replica(1);
        let mut vertices = HashMap::new();
        let mut edges = HashMap::new();

        // Create triangle (3 vertices, 3 edges)
        let v1 = Vertex::new("vertex1", replica, 1000);
        let v2 = Vertex::new("vertex2", replica, 2000);
        let v3 = Vertex::new("vertex3", replica, 3000);

        let v1_id = v1.id.clone();
        let v2_id = v2.id.clone();
        let v3_id = v3.id.clone();

        vertices.insert(v1_id.clone(), v1);
        vertices.insert(v2_id.clone(), v2);
        vertices.insert(v3_id.clone(), v3);

        let e1 = Edge::new(v1_id.clone(), v2_id.clone(), replica, 4000);
        let e2 = Edge::new(v2_id.clone(), v3_id.clone(), replica, 5000);
        let e3 = Edge::new(v3_id.clone(), v1_id.clone(), replica, 6000);

        edges.insert(e1.id.clone(), e1);
        edges.insert(e2.id.clone(), e2);
        edges.insert(e3.id.clone(), e3);

        let density = GraphAlgorithms::density(&vertices, &edges);
        // For 3 vertices, max edges = 3, actual edges = 3, density = 1.0
        assert_eq!(density, 1.0);
    }
}
