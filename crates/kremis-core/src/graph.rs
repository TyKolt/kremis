//! # Graph Engine
//!
//! The deterministic graph storage for Kremis CORE.
//!
//! This module implements the `GraphStore` trait.
//! All data structures use `BTreeMap` for deterministic ordering.

use crate::{Artifact, Attribute, EdgeWeight, EntityId, KremisError, Node, NodeId, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

// =============================================================================
// GRAPHSTORE TRAIT
// =============================================================================

/// The GraphStore trait defines the core graph operations.
///
/// All queries must be computationally bounded.
///
/// All fallible operations return `Result<T, KremisError>` to support both
/// in-memory and persistent storage backends uniformly.
pub trait GraphStore {
    /// Insert a node for the given entity. Returns the NodeId.
    /// If the entity already exists, returns the existing NodeId.
    fn insert_node(&mut self, entity: EntityId) -> Result<NodeId, KremisError>;

    /// Insert or update an edge with the given weight.
    /// If the edge exists, the weight is updated (not added).
    fn insert_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        weight: EdgeWeight,
    ) -> Result<(), KremisError>;

    /// Increment the weight of an edge by 1 using saturating arithmetic.
    /// Creates the edge with weight 1 if it doesn't exist.
    fn increment_edge(&mut self, from: NodeId, to: NodeId) -> Result<(), KremisError>;

    /// Lookup a node by its NodeId. Returns owned Node for storage compatibility.
    fn lookup(&self, id: NodeId) -> Result<Option<Node>, KremisError>;

    /// Get a node by its EntityId. This is infallible (uses in-memory cache).
    fn get_node_by_entity(&self, entity: EntityId) -> Option<NodeId>;

    /// Get the weight of an edge.
    fn get_edge(&self, from: NodeId, to: NodeId) -> Result<Option<EdgeWeight>, KremisError>;

    /// Get all neighbors of a node (outgoing edges).
    fn neighbors(&self, node: NodeId) -> Result<Vec<(NodeId, EdgeWeight)>, KremisError>;

    /// Check if a node exists in the graph.
    fn contains_node(&self, id: NodeId) -> Result<bool, KremisError>;

    /// Traverse the graph from a starting node up to a depth limit.
    fn traverse(&self, start: NodeId, depth: usize) -> Result<Option<Artifact>, KremisError>;

    /// Traverse with minimum weight filter.
    fn traverse_filtered(
        &self,
        start: NodeId,
        depth: usize,
        min_weight: EdgeWeight,
    ) -> Result<Option<Artifact>, KremisError>;

    /// Find nodes connected to ALL input nodes (intersection).
    fn intersect(&self, nodes: &[NodeId]) -> Result<Vec<NodeId>, KremisError>;

    /// Find the strongest path between two nodes.
    /// Cost = i64::MAX - weight, so higher weights = lower cost = preferred.
    fn strongest_path(
        &self,
        start: NodeId,
        end: NodeId,
    ) -> Result<Option<Vec<NodeId>>, KremisError>;

    /// Get the total number of nodes.
    fn node_count(&self) -> Result<usize, KremisError>;

    /// Get the total number of edges.
    fn edge_count(&self) -> Result<usize, KremisError>;

    /// Store a property (attribute, value) for a node.
    ///
    /// This persists the full signal data beyond just the entity.
    /// Multiple values can be stored for the same attribute.
    fn store_property(
        &mut self,
        node: NodeId,
        attribute: Attribute,
        value: Value,
    ) -> Result<(), KremisError>;

    /// Get all properties for a node.
    ///
    /// Returns a list of (Attribute, Value) pairs associated with this node.
    fn get_properties(&self, node: NodeId) -> Result<Vec<(Attribute, Value)>, KremisError>;
}

// =============================================================================
// GRAPH IMPLEMENTATION
// =============================================================================

/// The main Graph structure.
///
/// Uses `BTreeMap` exclusively for deterministic ordering.
/// No `HashMap` allowed.
#[derive(Debug, Clone, Default)]
pub struct Graph {
    /// Node storage: NodeId -> Node
    nodes: BTreeMap<NodeId, Node>,

    /// Adjacency list: from_node -> (to_node -> weight)
    edges: BTreeMap<NodeId, BTreeMap<NodeId, EdgeWeight>>,

    /// Reverse lookup: EntityId -> NodeId
    entity_index: BTreeMap<EntityId, NodeId>,

    /// Node properties: NodeId -> Attribute -> [Values]
    /// Stores the full signal data (attribute, value) for each node.
    properties: BTreeMap<NodeId, BTreeMap<Attribute, Vec<Value>>>,

    /// Next available NodeId
    next_node_id: u64,
}

impl Graph {
    /// Create a new empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reconstruct a graph from a canonical representation, preserving original NodeIds.
    #[must_use]
    pub fn from_canonical(canonical: &crate::export::CanonicalGraph) -> Self {
        let mut graph = Self {
            next_node_id: canonical.next_node_id,
            ..Self::default()
        };

        for cn in &canonical.nodes {
            let node_id = NodeId(cn.id);
            let entity = EntityId(cn.entity);
            let node = Node::new(node_id, entity);
            graph.nodes.insert(node_id, node);
            graph.entity_index.insert(entity, node_id);
        }

        for ce in &canonical.edges {
            let from = NodeId(ce.from);
            let to = NodeId(ce.to);
            if graph.nodes.contains_key(&from) && graph.nodes.contains_key(&to) {
                graph
                    .edges
                    .entry(from)
                    .or_default()
                    .insert(to, EdgeWeight::new(ce.weight));
            }
        }

        for cp in &canonical.properties {
            let node_id = NodeId(cp.node_id);
            if graph.nodes.contains_key(&node_id) {
                let _ = graph.store_property(
                    node_id,
                    Attribute::new(&cp.attribute),
                    Value::new(&cp.value),
                );
            }
        }

        graph
    }

    /// Get all nodes in deterministic order.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    /// Get all edges in deterministic order.
    pub fn edges(&self) -> impl Iterator<Item = (NodeId, NodeId, EdgeWeight)> + '_ {
        self.edges.iter().flat_map(|(from, targets)| {
            targets
                .iter()
                .map(move |(to, weight)| (*from, *to, *weight))
        })
    }

    /// Get the next node ID that would be assigned.
    #[must_use]
    pub fn next_node_id(&self) -> u64 {
        self.next_node_id
    }

    /// Check if the graph contains a node (internal, non-Result version).
    #[must_use]
    pub fn contains_node_internal(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    /// Check if the graph contains an edge.
    #[must_use]
    pub fn contains_edge(&self, from: NodeId, to: NodeId) -> bool {
        self.edges
            .get(&from)
            .is_some_and(|targets| targets.contains_key(&to))
    }

    /// Get neighbors (internal, iterator version for efficiency in algorithms).
    pub fn neighbors_internal(
        &self,
        node: NodeId,
    ) -> impl Iterator<Item = (NodeId, EdgeWeight)> + '_ {
        self.edges
            .get(&node)
            .into_iter()
            .flat_map(|targets| targets.iter().map(|(k, v)| (*k, *v)))
    }

    /// Get edge weight (internal, non-Result version).
    #[must_use]
    pub fn get_edge_internal(&self, from: NodeId, to: NodeId) -> Option<EdgeWeight> {
        self.edges.get(&from)?.get(&to).copied()
    }

    /// Import a node with its original NodeId (for export/import operations).
    ///
    /// # M3 Fix
    ///
    /// This method is used when rebuilding a graph from persistent storage
    /// for export purposes. It preserves the original NodeId instead of
    /// assigning a new one.
    pub fn import_node(&mut self, node: Node) {
        // Update next_node_id if necessary
        if node.id.0 >= self.next_node_id {
            self.next_node_id = node.id.0.saturating_add(1);
        }
        self.entity_index.insert(node.entity, node.id);
        self.nodes.insert(node.id, node);
    }
}

impl GraphStore for Graph {
    fn insert_node(&mut self, entity: EntityId) -> Result<NodeId, KremisError> {
        // Return existing node if entity already mapped
        if let Some(&node_id) = self.entity_index.get(&entity) {
            return Ok(node_id);
        }

        // Create new node
        let node_id = NodeId(self.next_node_id);
        self.next_node_id = self.next_node_id.saturating_add(1);

        let node = Node::new(node_id, entity);
        self.nodes.insert(node_id, node);
        self.entity_index.insert(entity, node_id);

        Ok(node_id)
    }

    fn insert_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        weight: EdgeWeight,
    ) -> Result<(), KremisError> {
        if !self.nodes.contains_key(&from) || !self.nodes.contains_key(&to) {
            return Ok(());
        }
        self.edges.entry(from).or_default().insert(to, weight);
        Ok(())
    }

    fn increment_edge(&mut self, from: NodeId, to: NodeId) -> Result<(), KremisError> {
        let targets = self.edges.entry(from).or_default();
        let current = targets.get(&to).copied().unwrap_or(EdgeWeight::new(0));
        targets.insert(to, current.increment());
        Ok(())
    }

    fn lookup(&self, id: NodeId) -> Result<Option<Node>, KremisError> {
        Ok(self.nodes.get(&id).cloned())
    }

    fn get_node_by_entity(&self, entity: EntityId) -> Option<NodeId> {
        self.entity_index.get(&entity).copied()
    }

    fn get_edge(&self, from: NodeId, to: NodeId) -> Result<Option<EdgeWeight>, KremisError> {
        Ok(self
            .edges
            .get(&from)
            .and_then(|targets| targets.get(&to).copied()))
    }

    fn neighbors(&self, node: NodeId) -> Result<Vec<(NodeId, EdgeWeight)>, KremisError> {
        Ok(self
            .edges
            .get(&node)
            .into_iter()
            .flat_map(|targets| targets.iter().map(|(k, v)| (*k, *v)))
            .collect())
    }

    fn contains_node(&self, id: NodeId) -> Result<bool, KremisError> {
        Ok(self.nodes.contains_key(&id))
    }

    fn traverse(&self, start: NodeId, depth: usize) -> Result<Option<Artifact>, KremisError> {
        let depth = depth.min(crate::primitives::MAX_TRAVERSAL_DEPTH);
        if !self.contains_node_internal(start) {
            return Ok(None);
        }

        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        let mut path = Vec::new();
        let mut subgraph_edges = Vec::new();

        queue.push_back((start, 0usize));
        visited.insert(start);

        while let Some((current, current_depth)) = queue.pop_front() {
            path.push(current);

            if current_depth >= depth {
                continue;
            }

            for (neighbor, weight) in self.neighbors_internal(current) {
                subgraph_edges.push((current, neighbor, weight));

                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back((neighbor, current_depth.saturating_add(1)));
                }
            }
        }

        Ok(Some(Artifact::with_subgraph(path, subgraph_edges)))
    }

    fn traverse_filtered(
        &self,
        start: NodeId,
        depth: usize,
        min_weight: EdgeWeight,
    ) -> Result<Option<Artifact>, KremisError> {
        let depth = depth.min(crate::primitives::MAX_TRAVERSAL_DEPTH);
        if !self.contains_node_internal(start) {
            return Ok(None);
        }

        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        let mut path = Vec::new();
        let mut subgraph_edges = Vec::new();

        queue.push_back((start, 0usize));
        visited.insert(start);

        while let Some((current, current_depth)) = queue.pop_front() {
            path.push(current);

            if current_depth >= depth {
                continue;
            }

            for (neighbor, weight) in self.neighbors_internal(current) {
                // Filter by minimum weight
                if weight.value() >= min_weight.value() {
                    subgraph_edges.push((current, neighbor, weight));

                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back((neighbor, current_depth.saturating_add(1)));
                    }
                }
            }
        }

        Ok(Some(Artifact::with_subgraph(path, subgraph_edges)))
    }

    fn intersect(&self, nodes: &[NodeId]) -> Result<Vec<NodeId>, KremisError> {
        if nodes.is_empty() {
            return Ok(Vec::new());
        }

        // Get neighbors of first node
        let first_neighbors: BTreeSet<_> =
            self.neighbors_internal(nodes[0]).map(|(n, _)| n).collect();

        if first_neighbors.is_empty() {
            return Ok(Vec::new());
        }

        // Intersect with neighbors of remaining nodes
        let mut result = first_neighbors;
        for &node in &nodes[1..] {
            let neighbors: BTreeSet<_> = self.neighbors_internal(node).map(|(n, _)| n).collect();
            result = result.intersection(&neighbors).copied().collect();
        }

        Ok(result.into_iter().collect())
    }

    fn strongest_path(
        &self,
        start: NodeId,
        end: NodeId,
    ) -> Result<Option<Vec<NodeId>>, KremisError> {
        if !self.contains_node_internal(start) || !self.contains_node_internal(end) {
            return Ok(None);
        }

        if start == end {
            return Ok(Some(vec![start]));
        }

        // Dijkstra with cost = i64::MAX - weight (to find maximum weight path)
        // Using BTreeMap for deterministic ordering
        let mut dist: BTreeMap<NodeId, i64> = BTreeMap::new();
        let mut prev: BTreeMap<NodeId, NodeId> = BTreeMap::new();
        let mut visited = BTreeSet::new();

        dist.insert(start, 0);

        loop {
            // Find unvisited node with minimum distance
            let current = dist
                .iter()
                .filter(|(n, _)| !visited.contains(*n))
                .min_by_key(|(_, d)| *d)
                .map(|(n, _)| *n);

            let Some(current) = current else {
                break;
            };

            if current == end {
                break;
            }

            visited.insert(current);
            let current_dist = dist[&current];

            for (neighbor, weight) in self.neighbors_internal(current) {
                if visited.contains(&neighbor) {
                    continue;
                }

                // Cost = i64::MAX - weight (higher weight = lower cost = preferred)
                // Clamp negative weights to 0 to maintain Dijkstra invariant
                let clamped_weight = weight.value().max(0);
                let edge_cost = i64::MAX.saturating_sub(clamped_weight);
                let new_dist = current_dist.saturating_add(edge_cost);

                if !dist.contains_key(&neighbor) || new_dist < dist[&neighbor] {
                    dist.insert(neighbor, new_dist);
                    prev.insert(neighbor, current);
                }
            }
        }

        // Reconstruct path
        if !prev.contains_key(&end) && start != end {
            return Ok(None);
        }

        let mut path = Vec::new();
        let mut current = end;
        while current != start {
            path.push(current);
            current = match prev.get(&current) {
                Some(&p) => p,
                None => return Ok(None),
            };
        }
        path.push(start);
        path.reverse();

        Ok(Some(path))
    }

    fn node_count(&self) -> Result<usize, KremisError> {
        Ok(self.nodes.len())
    }

    fn edge_count(&self) -> Result<usize, KremisError> {
        Ok(self.edges.values().map(BTreeMap::len).sum())
    }

    fn store_property(
        &mut self,
        node: NodeId,
        attribute: Attribute,
        value: Value,
    ) -> Result<(), KremisError> {
        if !self.nodes.contains_key(&node) {
            return Err(KremisError::NodeNotFound(node));
        }
        self.properties
            .entry(node)
            .or_default()
            .entry(attribute)
            .or_default()
            .push(value);
        Ok(())
    }

    fn get_properties(&self, node: NodeId) -> Result<Vec<(Attribute, Value)>, KremisError> {
        if !self.nodes.contains_key(&node) {
            return Err(KremisError::NodeNotFound(node));
        }
        let mut result = Vec::new();
        if let Some(attrs) = self.properties.get(&node) {
            for (attr, values) in attrs {
                for value in values {
                    result.push((attr.clone(), value.clone()));
                }
            }
        }
        Ok(result)
    }
}

// =============================================================================
// ADDITIONAL TRAVERSAL METHODS
// =============================================================================

impl Graph {
    /// Depth-first traversal from a starting node.
    ///
    /// DFS is an alternative to BFS
    /// with deterministic ordering via BTreeMap.
    pub fn traverse_dfs(&self, start: NodeId, depth: usize) -> Option<Artifact> {
        use crate::primitives::MAX_TRAVERSAL_DEPTH;

        if !self.contains_node_internal(start) {
            return None;
        }

        // Enforce computational bound
        let bounded_depth = depth.min(MAX_TRAVERSAL_DEPTH);

        let mut visited = BTreeSet::new();
        let mut path = Vec::new();
        let mut subgraph_edges = Vec::new();

        self.dfs_recursive(
            start,
            0,
            bounded_depth,
            &mut visited,
            &mut path,
            &mut subgraph_edges,
        );

        Some(Artifact::with_subgraph(path, subgraph_edges))
    }

    /// Recursive DFS helper.
    fn dfs_recursive(
        &self,
        current: NodeId,
        current_depth: usize,
        max_depth: usize,
        visited: &mut BTreeSet<NodeId>,
        path: &mut Vec<NodeId>,
        subgraph_edges: &mut Vec<(NodeId, NodeId, EdgeWeight)>,
    ) {
        if visited.contains(&current) || current_depth > max_depth {
            return;
        }

        visited.insert(current);
        path.push(current);

        if current_depth < max_depth {
            for (neighbor, weight) in self.neighbors_internal(current) {
                subgraph_edges.push((current, neighbor, weight));

                if !visited.contains(&neighbor) {
                    self.dfs_recursive(
                        neighbor,
                        current_depth.saturating_add(1),
                        max_depth,
                        visited,
                        path,
                        subgraph_edges,
                    );
                }
            }
        }
    }

    /// Bounded traverse that enforces MAX_TRAVERSAL_DEPTH.
    pub fn traverse_bounded(
        &self,
        start: NodeId,
        depth: usize,
    ) -> Result<Option<Artifact>, KremisError> {
        use crate::primitives::MAX_TRAVERSAL_DEPTH;
        self.traverse(start, depth.min(MAX_TRAVERSAL_DEPTH))
    }
}

// =============================================================================
// SERIALIZATION SUPPORT
// =============================================================================

use serde::{Deserialize, Serialize};

/// Serializable representation of the graph for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<(NodeId, NodeId, EdgeWeight)>,
    pub next_node_id: u64,
    #[serde(default)]
    pub properties: Vec<(u64, String, String)>,
}

impl From<&Graph> for SerializableGraph {
    fn from(graph: &Graph) -> Self {
        let mut properties = Vec::new();
        for node in graph.nodes.values() {
            if let Ok(props) = graph.get_properties(node.id) {
                for (attr, val) in props {
                    properties.push((
                        node.id.0,
                        attr.as_str().to_string(),
                        val.as_str().to_string(),
                    ));
                }
            }
        }
        Self {
            nodes: graph.nodes.values().cloned().collect(),
            edges: graph.edges().collect(),
            next_node_id: graph.next_node_id,
            properties,
        }
    }
}

impl From<SerializableGraph> for Graph {
    fn from(sg: SerializableGraph) -> Self {
        let mut graph = Graph::new();
        graph.next_node_id = sg.next_node_id;

        for node in sg.nodes {
            graph.nodes.insert(node.id, node.clone());
            graph.entity_index.insert(node.entity, node.id);
        }

        for (from, to, weight) in sg.edges {
            let _ = graph.insert_edge(from, to, weight);
        }

        for (node_id, attr, val) in sg.properties {
            let _ = graph.store_property(NodeId(node_id), Attribute::new(&attr), Value::new(&val));
        }

        graph
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_lookup_node() {
        let mut graph = Graph::new();
        let entity = EntityId(42);

        let node_id = graph.insert_node(entity).expect("insert");
        let node = graph.lookup(node_id).expect("lookup");

        assert!(node.is_some());
        assert_eq!(node.map(|n| n.entity), Some(entity));
    }

    #[test]
    fn insert_duplicate_entity_returns_same_node() {
        let mut graph = Graph::new();
        let entity = EntityId(42);

        let first = graph.insert_node(entity).expect("insert");
        let second = graph.insert_node(entity).expect("insert");

        assert_eq!(first, second);
        assert_eq!(graph.node_count().expect("count"), 1);
    }

    #[test]
    fn increment_edge_creates_and_increments() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");

        // First increment creates edge with weight 1
        graph.increment_edge(a, b).expect("increment");
        assert_eq!(graph.get_edge(a, b).expect("get"), Some(EdgeWeight::new(1)));

        // Second increment increases to 2
        graph.increment_edge(a, b).expect("increment");
        assert_eq!(graph.get_edge(a, b).expect("get"), Some(EdgeWeight::new(2)));
    }

    #[test]
    fn neighbors_in_deterministic_order() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        let c = graph.insert_node(EntityId(3)).expect("insert");

        // Insert edges in non-sorted order
        graph.insert_edge(a, c, EdgeWeight::new(1)).expect("insert");
        graph.insert_edge(a, b, EdgeWeight::new(2)).expect("insert");

        let neighbors: Vec<_> = graph
            .neighbors(a)
            .expect("neighbors")
            .into_iter()
            .map(|(n, _)| n)
            .collect();

        // Should be sorted by NodeId
        assert_eq!(neighbors, vec![b, c]);
    }

    #[test]
    fn traverse_respects_depth() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        let c = graph.insert_node(EntityId(3)).expect("insert");

        graph.insert_edge(a, b, EdgeWeight::new(1)).expect("insert");
        graph.insert_edge(b, c, EdgeWeight::new(1)).expect("insert");

        // Depth 1: should reach a and b
        let artifact = graph.traverse(a, 1).expect("traverse");
        assert!(artifact.is_some());

        let path = artifact.as_ref().map(|a| &a.path);
        assert!(path.is_some());
        assert!(path.map(|p| p.contains(&a)).unwrap_or(false));
        assert!(path.map(|p| p.contains(&b)).unwrap_or(false));
    }

    #[test]
    fn traverse_missing_node_returns_none() {
        let graph = Graph::new();
        let result = graph.traverse(NodeId(999), 5).expect("traverse");
        assert!(result.is_none());
    }

    #[test]
    fn strongest_path_finds_route() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        let c = graph.insert_node(EntityId(3)).expect("insert");

        graph
            .insert_edge(a, b, EdgeWeight::new(10))
            .expect("insert");
        graph
            .insert_edge(b, c, EdgeWeight::new(10))
            .expect("insert");

        let path = graph.strongest_path(a, c).expect("path");
        assert_eq!(path, Some(vec![a, b, c]));
    }

    #[test]
    fn intersect_finds_common_neighbors() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        let common = graph.insert_node(EntityId(100)).expect("insert");

        graph
            .insert_edge(a, common, EdgeWeight::new(1))
            .expect("insert");
        graph
            .insert_edge(b, common, EdgeWeight::new(1))
            .expect("insert");

        let result = graph.intersect(&[a, b]).expect("intersect");
        assert_eq!(result, vec![common]);
    }

    #[test]
    fn serialization_roundtrip() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        graph.insert_edge(a, b, EdgeWeight::new(5)).expect("insert");

        let serializable = SerializableGraph::from(&graph);
        let restored = Graph::from(serializable);

        assert_eq!(
            graph.node_count().expect("count"),
            restored.node_count().expect("count")
        );
        assert_eq!(
            graph.edge_count().expect("count"),
            restored.edge_count().expect("count")
        );
        assert_eq!(
            restored.get_edge(a, b).expect("get"),
            Some(EdgeWeight::new(5))
        );
    }

    #[test]
    fn store_and_get_properties() {
        let mut graph = Graph::new();
        let node = graph.insert_node(EntityId(1)).expect("insert");

        graph
            .store_property(node, Attribute::new("name"), Value::new("Alice"))
            .expect("store");
        graph
            .store_property(node, Attribute::new("age"), Value::new("30"))
            .expect("store");

        let props = graph.get_properties(node).expect("get");
        assert_eq!(props.len(), 2);
        assert!(props.contains(&(Attribute::new("name"), Value::new("Alice"))));
        assert!(props.contains(&(Attribute::new("age"), Value::new("30"))));
    }

    #[test]
    fn store_multiple_values_same_attribute() {
        let mut graph = Graph::new();
        let node = graph.insert_node(EntityId(1)).expect("insert");

        graph
            .store_property(node, Attribute::new("knows"), Value::new("Bob"))
            .expect("store");
        graph
            .store_property(node, Attribute::new("knows"), Value::new("Charlie"))
            .expect("store");

        let props = graph.get_properties(node).expect("get");
        assert_eq!(props.len(), 2);
        assert!(props.contains(&(Attribute::new("knows"), Value::new("Bob"))));
        assert!(props.contains(&(Attribute::new("knows"), Value::new("Charlie"))));
    }

    #[test]
    fn store_property_nonexistent_node_fails() {
        let mut graph = Graph::new();
        let result = graph.store_property(NodeId(999), Attribute::new("name"), Value::new("Test"));
        assert!(result.is_err());
    }

    #[test]
    fn get_properties_nonexistent_node_fails() {
        let graph = Graph::new();
        let result = graph.get_properties(NodeId(999));
        assert!(result.is_err());
    }

    #[test]
    fn get_properties_empty_returns_empty_vec() {
        let mut graph = Graph::new();
        let node = graph.insert_node(EntityId(1)).expect("insert");

        let props = graph.get_properties(node).expect("get");
        assert!(props.is_empty());
    }

    #[test]
    fn insert_edge_ignores_dangling_nodes() {
        let mut graph = Graph::new();
        let node1 = graph.insert_node(EntityId(1)).expect("insert");
        let dangling = NodeId(999);

        // Edge from existing to non-existing: silently ignored
        graph
            .insert_edge(node1, dangling, EdgeWeight::new(5))
            .expect("insert edge");
        assert_eq!(graph.edge_count().expect("count"), 0);

        // Edge from non-existing to existing: silently ignored
        graph
            .insert_edge(dangling, node1, EdgeWeight::new(5))
            .expect("insert edge");
        assert_eq!(graph.edge_count().expect("count"), 0);
    }

    #[test]
    fn serializable_graph_roundtrip_with_properties() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        graph.insert_edge(a, b, EdgeWeight::new(5)).expect("insert");

        graph
            .store_property(a, Attribute::new("name"), Value::new("Alice"))
            .expect("store");
        graph
            .store_property(a, Attribute::new("role"), Value::new("admin"))
            .expect("store");
        graph
            .store_property(b, Attribute::new("name"), Value::new("Bob"))
            .expect("store");

        let serializable = SerializableGraph::from(&graph);
        assert_eq!(serializable.properties.len(), 3);

        let restored = Graph::from(serializable);

        assert_eq!(
            graph.node_count().expect("count"),
            restored.node_count().expect("count")
        );
        assert_eq!(
            graph.edge_count().expect("count"),
            restored.edge_count().expect("count")
        );

        let props_a = restored.get_properties(a).expect("get");
        assert_eq!(props_a.len(), 2);
        assert!(props_a.contains(&(Attribute::new("name"), Value::new("Alice"))));
        assert!(props_a.contains(&(Attribute::new("role"), Value::new("admin"))));

        let props_b = restored.get_properties(b).expect("get");
        assert_eq!(props_b.len(), 1);
        assert!(props_b.contains(&(Attribute::new("name"), Value::new("Bob"))));
    }
}
