//! DAG (Directed Acyclic Graph) for complex relationships

use super::common::{PositionId, AdvancedCrdtError};
use super::super::{CRDT, Mergeable, ReplicaId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// DAG (Directed Acyclic Graph) node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagNode<T> {
    /// Unique node identifier
    pub id: PositionId,
    /// Node value
    pub value: T,
    /// Incoming edges (dependencies)
    pub incoming: HashSet<PositionId>,
    /// Outgoing edges (dependents)
    pub outgoing: HashSet<PositionId>,
    /// Whether the node is visible (not deleted)
    pub visible: bool,
}

impl<T> DagNode<T> {
    /// Create a new DAG node
    pub fn new(id: PositionId, value: T) -> Self {
        Self {
            id,
            value,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
            visible: true,
        }
    }
}

/// DAG (Directed Acyclic Graph) for complex relationships
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dag<T> {
    /// Replica ID
    replica_id: ReplicaId,
    /// Nodes indexed by ID
    nodes: HashMap<PositionId, DagNode<T>>,
    /// Logical timestamp counter
    timestamp_counter: u64,
    /// Disambiguation counter
    disambiguation_counter: u64,
}

impl<T: Clone + PartialEq> Dag<T> {
    /// Create a new DAG
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            replica_id,
            nodes: HashMap::new(),
            timestamp_counter: 0,
            disambiguation_counter: 0,
        }
    }
    
    /// Add a node
    pub fn add_node(&mut self, value: T) -> Result<PositionId, AdvancedCrdtError> {
        self.timestamp_counter += 1;
        self.disambiguation_counter += 1;
        
        let id = PositionId::new(
            self.replica_id.clone(),
            self.timestamp_counter,
            self.disambiguation_counter,
        );
        
        let node = DagNode::new(id.clone(), value);
        self.nodes.insert(id.clone(), node);
        
        Ok(id)
    }
    
    /// Add an edge from source to target
    pub fn add_edge(&mut self, from: &PositionId, to: &PositionId) -> Result<(), AdvancedCrdtError> {
        if !self.nodes.contains_key(from) || !self.nodes.contains_key(to) {
            return Err(AdvancedCrdtError::ElementNotFound("Node not found".to_string()));
        }
        
        // Check for cycle
        if self.would_create_cycle(from, to) {
            return Err(AdvancedCrdtError::CycleDetected("Adding edge would create cycle".to_string()));
        }
        
        // Add edge
        if let Some(from_node) = self.nodes.get_mut(from) {
            from_node.outgoing.insert(to.clone());
        }
        if let Some(to_node) = self.nodes.get_mut(to) {
            to_node.incoming.insert(from.clone());
        }
        
        Ok(())
    }
    
    /// Remove an edge
    pub fn remove_edge(&mut self, from: &PositionId, to: &PositionId) -> Result<(), AdvancedCrdtError> {
        if let Some(from_node) = self.nodes.get_mut(from) {
            from_node.outgoing.remove(to);
        }
        if let Some(to_node) = self.nodes.get_mut(to) {
            to_node.incoming.remove(from);
        }
        
        Ok(())
    }
    
    /// Delete a node
    pub fn delete_node(&mut self, node_id: &PositionId) -> Result<(), AdvancedCrdtError> {
        if let Some(node) = self.nodes.get(node_id) {
            let incoming_edges = node.incoming.clone();
            let outgoing_edges = node.outgoing.clone();
            
            // Mark node as invisible
            if let Some(node) = self.nodes.get_mut(node_id) {
                node.visible = false;
            }
            
            // Remove all edges involving this node
            for incoming in &incoming_edges {
                if let Some(incoming_node) = self.nodes.get_mut(incoming) {
                    incoming_node.outgoing.remove(node_id);
                }
            }
            for outgoing in &outgoing_edges {
                if let Some(outgoing_node) = self.nodes.get_mut(outgoing) {
                    outgoing_node.incoming.remove(node_id);
                }
            }
            
            Ok(())
        } else {
            Err(AdvancedCrdtError::ElementNotFound(format!("Node {:?}", node_id)))
        }
    }
    
    /// Check if adding an edge would create a cycle
    fn would_create_cycle(&self, from: &PositionId, to: &PositionId) -> bool {
        if from == to {
            return true;
        }
        
        // Use DFS to check for path from 'to' to 'from'
        let mut visited = HashSet::new();
        self.dfs_cycle_check(to, from, &mut visited)
    }
    
    /// DFS helper for cycle detection
    fn dfs_cycle_check(&self, current: &PositionId, target: &PositionId, visited: &mut HashSet<PositionId>) -> bool {
        if current == target {
            return true;
        }
        
        if visited.contains(current) {
            return false;
        }
        
        visited.insert(current.clone());
        
        if let Some(node) = self.nodes.get(current) {
            for next in &node.outgoing {
                if self.dfs_cycle_check(next, target, visited) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Get topological sort of the DAG
    pub fn topological_sort(&self) -> Vec<PositionId> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        
        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                self.dfs_topological(node_id, &mut visited, &mut result);
            }
        }
        
        result.reverse();
        result
    }
    
    /// DFS helper for topological sort
    fn dfs_topological(&self, node_id: &PositionId, visited: &mut HashSet<PositionId>, result: &mut Vec<PositionId>) {
        if visited.contains(node_id) {
            return;
        }
        
        visited.insert(node_id.clone());
        
        if let Some(node) = self.nodes.get(node_id) {
            for next in &node.outgoing {
                self.dfs_topological(next, visited, result);
            }
        }
        
        result.push(node_id.clone());
    }
    
    /// Get node count
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    
    /// Check if DAG is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    
    /// Get all nodes (for debugging/inspection)
    pub fn get_nodes(&self) -> &HashMap<PositionId, DagNode<T>> {
        &self.nodes
    }
}

impl<T: Clone + PartialEq> CRDT for Dag<T> {
    fn replica_id(&self) -> &ReplicaId {
        &self.replica_id
    }
}

impl<T: Clone + PartialEq + Send + Sync> Mergeable for Dag<T> {
    type Error = AdvancedCrdtError;
    
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        // Merge all nodes from other DAG
        for (node_id, other_node) in &other.nodes {
            if let Some(self_node) = self.nodes.get_mut(node_id) {
                // Node exists in both, keep the one with higher timestamp
                if other_node.id.timestamp > self_node.id.timestamp {
                    *self_node = other_node.clone();
                }
            } else {
                // Node only exists in other, add it
                self.nodes.insert(node_id.clone(), other_node.clone());
            }
        }
        
        Ok(())
    }
    
    fn has_conflict(&self, other: &Self) -> bool {
        // Check for conflicting nodes (same ID, different values)
        for (node_id, self_node) in &self.nodes {
            if let Some(other_node) = other.nodes.get(node_id) {
                if self_node.value != other_node.value {
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
    use super::super::super::ReplicaId;
    use uuid::Uuid;
    
    fn create_replica(id: u64) -> ReplicaId {
        ReplicaId::from(Uuid::from_u64_pair(0, id))
    }
    
    #[test]
    fn test_dag_creation() {
        let replica_id = create_replica(1);
        let dag = Dag::<String>::new(replica_id.clone());
        
        assert_eq!(dag.replica_id(), &replica_id);
        assert!(dag.is_empty());
        assert_eq!(dag.len(), 0);
    }
    
    #[test]
    fn test_dag_operations() {
        let replica_id = create_replica(1);
        let mut dag = Dag::<String>::new(replica_id);
        
        // Add nodes
        let node1_id = dag.add_node("node1".to_string()).unwrap();
        let node2_id = dag.add_node("node2".to_string()).unwrap();
        let node3_id = dag.add_node("node3".to_string()).unwrap();
        
        assert_eq!(dag.len(), 3);
        
        // Add edges
        dag.add_edge(&node1_id, &node2_id).unwrap();
        dag.add_edge(&node2_id, &node3_id).unwrap();
        
        // Test topological sort
        let sorted = dag.topological_sort();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0], node1_id);
        assert_eq!(sorted[1], node2_id);
        assert_eq!(sorted[2], node3_id);
        
        // Remove edge
        dag.remove_edge(&node1_id, &node2_id).unwrap();
        
        // Delete node
        dag.delete_node(&node2_id).unwrap();
        assert_eq!(dag.len(), 3); // Node still exists but is invisible
    }
    
    #[test]
    fn test_dag_cycle_detection() {
        let replica_id = create_replica(1);
        let mut dag = Dag::<String>::new(replica_id);
        
        // Add nodes
        let node1_id = dag.add_node("node1".to_string()).unwrap();
        let node2_id = dag.add_node("node2".to_string()).unwrap();
        let node3_id = dag.add_node("node3".to_string()).unwrap();
        
        // Add edges to create a cycle
        dag.add_edge(&node1_id, &node2_id).unwrap();
        dag.add_edge(&node2_id, &node3_id).unwrap();
        
        // Try to add edge that would create cycle
        let result = dag.add_edge(&node3_id, &node1_id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AdvancedCrdtError::CycleDetected("Adding edge would create cycle".to_string()));
    }
    
    #[test]
    fn test_dag_merge() {
        let replica_id1 = create_replica(1);
        let replica_id2 = create_replica(2);
        
        let mut dag1 = Dag::<String>::new(replica_id1);
        let mut dag2 = Dag::<String>::new(replica_id2);
        
        // Add different nodes
        let node1_id = dag1.add_node("node1".to_string()).unwrap();
        let node2_id = dag2.add_node("node2".to_string()).unwrap();
        
        // Add edges (no self-loops in DAG)
        // Just add nodes without edges for merge test
        
        // Merge
        dag1.merge(&dag2).unwrap();
        
        // Should contain both nodes
        assert_eq!(dag1.len(), 2);
    }
}
