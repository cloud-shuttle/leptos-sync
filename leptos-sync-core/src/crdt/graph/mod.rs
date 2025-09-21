//! Graph CRDT implementations
//!
//! This module provides graph-based CRDT implementations including:
//! - Add-Wins Graph: Preserves deleted elements for potential recovery
//! - Remove-Wins Graph: Completely removes deleted elements for memory efficiency
//! - Graph algorithms: Path finding, connectivity analysis, etc.

pub mod add_wins;
pub mod algorithms;
pub mod edge;
pub mod remove_wins;
pub mod vertex;

// Re-export main types for convenience
pub use add_wins::{AddWinsGraph, GraphConfig};
pub use algorithms::GraphAlgorithms;
pub use edge::{Edge, EdgeId, EdgeMetadata};
pub use remove_wins::RemoveWinsGraph;
pub use vertex::{GraphError, Vertex, VertexId, VertexMetadata};

/// Strategy for handling graph conflicts
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GraphStrategy {
    /// Add-Wins: Vertices and edges are never removed, only marked as deleted
    AddWins,
    /// Remove-Wins: Deleted vertices and edges are completely removed
    RemoveWins,
}

#[cfg(test)]
mod integration_tests {
    use super::super::{ReplicaId, basic::traits::Mergeable};
    use super::*;
    use uuid::Uuid;

    fn create_replica(id: u64) -> ReplicaId {
        ReplicaId::from(Uuid::from_u64_pair(0, id))
    }

    #[test]
    fn test_graph_module_integration() {
        let replica = create_replica(1);

        // Test Add-Wins Graph
        let mut add_wins_graph = AddWinsGraph::new(replica);
        let v1_id = add_wins_graph.add_vertex("vertex1", 1000);
        let v2_id = add_wins_graph.add_vertex("vertex2", 2000);
        let _edge_id = add_wins_graph.add_edge(&v1_id, &v2_id, 3000, None).unwrap();

        assert_eq!(add_wins_graph.vertex_count(), 2);
        assert_eq!(add_wins_graph.edge_count(), 1);

        // Test Remove-Wins Graph
        let mut remove_wins_graph = RemoveWinsGraph::new(replica);
        let v1_id = remove_wins_graph.add_vertex("vertex1", 1000);
        let v2_id = remove_wins_graph.add_vertex("vertex2", 2000);
        let edge_id = remove_wins_graph
            .add_edge(&v1_id, &v2_id, 3000, None)
            .unwrap();

        assert_eq!(remove_wins_graph.vertex_count(), 2);
        assert_eq!(remove_wins_graph.edge_count(), 1);

        // Test complete removal
        remove_wins_graph.remove_edge(&edge_id).unwrap();
        assert_eq!(remove_wins_graph.edge_count(), 0);
    }

    #[test]
    fn test_graph_algorithms_integration() {
        let replica = create_replica(1);
        let mut graph = AddWinsGraph::new(replica);

        // Create a simple path
        let v1_id = graph.add_vertex("vertex1", 1000);
        let v2_id = graph.add_vertex("vertex2", 2000);
        let v3_id = graph.add_vertex("vertex3", 3000);

        graph.add_edge(&v1_id, &v2_id, 4000, None).unwrap();
        graph.add_edge(&v2_id, &v3_id, 5000, None).unwrap();

        // Test shortest path algorithm
        let path = graph.shortest_path(&v1_id, &v3_id).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], v1_id);
        assert_eq!(path[1], v2_id);
        assert_eq!(path[2], v3_id);
    }

    #[test]
    fn test_graph_merge_integration() {
        let replica1 = create_replica(1);
        let replica2 = create_replica(2);

        let mut graph1 = AddWinsGraph::new(replica1);
        let mut graph2 = AddWinsGraph::new(replica2);

        // Add different vertices to each graph
        let v1_id = graph1.add_vertex("vertex1", 1000);
        let v2_id = graph2.add_vertex("vertex2", 2000);

        // Merge graphs
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
            preserve_deleted: false,
            max_vertices: Some(100),
            max_edges: Some(200),
            allow_self_loops: true,
            allow_multiple_edges: true,
        };

        let graph: AddWinsGraph<String> = AddWinsGraph::with_config(replica, config);
        assert_eq!(graph.config.max_vertices, Some(100));
        assert_eq!(graph.config.allow_self_loops, true);
        assert_eq!(graph.config.allow_multiple_edges, true);
    }
}
