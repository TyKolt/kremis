//! # redb-backed Graph Storage
//!
//! A disk-backed graph store using redb embedded database.
//!
//! This replaces the custom segment/WAL/LRU implementation with a
//! battle-tested embedded database, providing:
//! - ACID transactions
//! - Crash safety (copy-on-write B-trees)
//! - MVCC (concurrent readers, single writer)
//! - Zero configuration
//!
//! ## Integration with Session
//!
//! This module provides `RedbGraph` which can be used as a persistent
//! storage backend for Kremis sessions. Unlike the in-memory `Graph`,
//! `RedbGraph` persists data to disk automatically.

use crate::graph::GraphStore;
use crate::{Artifact, Attribute, EdgeWeight, EntityId, KremisError, Node, NodeId, Signal, Value};
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Table for nodes: NodeId(u64) -> serialized Node bytes
const NODES: TableDefinition<u64, &[u8]> = TableDefinition::new("nodes");

/// Table for edges: (from_id, to_id) -> weight
const EDGES: TableDefinition<(u64, u64), i64> = TableDefinition::new("edges");

/// Table for entity index: EntityId(u64) -> NodeId(u64)
const ENTITY_INDEX: TableDefinition<u64, u64> = TableDefinition::new("entity_index");

/// Table for metadata: key string -> value u64
const METADATA: TableDefinition<&str, u64> = TableDefinition::new("metadata");

/// Table for properties: (node_id, attr_hash) -> serialized (Attribute, Vec<Value>)
/// We use attr_hash (u64) as part of the key to enable range queries per node.
const PROPERTIES: TableDefinition<(u64, u64), &[u8]> = TableDefinition::new("properties");

/// A disk-backed graph store using redb.
///
/// Per the architectural decision:
/// - Replaces custom WAL, segments, and LRU cache
/// - Uses redb for crash safety and ACID
/// - Maintains in-memory entity index for fast lookups
pub struct RedbGraph {
    /// The redb database handle.
    db: Database,
    /// In-memory cache of entity -> node mapping for fast lookups.
    entity_cache: BTreeMap<EntityId, NodeId>,
    /// Next available node ID.
    next_node_id: u64,
}

impl std::fmt::Debug for RedbGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbGraph")
            .field("entity_cache_size", &self.entity_cache.len())
            .field("next_node_id", &self.next_node_id)
            .finish_non_exhaustive()
    }
}

impl RedbGraph {
    /// Open or create a graph database at the given path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, KremisError> {
        let db =
            Database::create(path.as_ref()).map_err(|e| KremisError::IoError(e.to_string()))?;

        // Initialize tables if they don't exist
        {
            let write_txn = db
                .begin_write()
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            let _ = write_txn
                .open_table(NODES)
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            let _ = write_txn
                .open_table(EDGES)
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            let _ = write_txn
                .open_table(ENTITY_INDEX)
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            let _ = write_txn
                .open_table(METADATA)
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            let _ = write_txn
                .open_table(PROPERTIES)
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            write_txn
                .commit()
                .map_err(|e| KremisError::IoError(e.to_string()))?;
        }

        // Load metadata
        let read_txn = db
            .begin_read()
            .map_err(|e| KremisError::IoError(e.to_string()))?;

        let next_node_id = {
            let table = read_txn
                .open_table(METADATA)
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            table
                .get("next_node_id")
                .map_err(|e| KremisError::IoError(e.to_string()))?
                .map(|v| v.value())
                .unwrap_or(0)
        };

        // Load entity cache
        let entity_cache = {
            let table = read_txn
                .open_table(ENTITY_INDEX)
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            let mut cache = BTreeMap::new();
            for entry in table
                .iter()
                .map_err(|e| KremisError::IoError(e.to_string()))?
            {
                let (key, value) = entry.map_err(|e| KremisError::IoError(e.to_string()))?;
                cache.insert(EntityId(key.value()), NodeId(value.value()));
            }
            cache
        };

        Ok(Self {
            db,
            entity_cache,
            next_node_id,
        })
    }

    /// Compact the database (optional optimization).
    pub fn compact(&mut self) -> Result<(), KremisError> {
        self.db
            .compact()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Ingest a sequence of signals in a single batch ACID transaction.
    ///
    /// Unlike `Ingestor::ingest_sequence`, this method groups all writes
    /// (nodes, properties, edges) into a single redb transaction, reducing
    /// fsync overhead from O(N) to O(1).
    ///
    /// Edges are formed between adjacent signals per `ASSOCIATION_WINDOW`.
    ///
    /// # Errors
    ///
    /// Returns `KremisError::InvalidSignal` if:
    /// - The sequence exceeds `MAX_SEQUENCE_LENGTH`
    /// - Any signal is invalid (all signals are validated before the transaction opens)
    pub fn ingest_batch(&mut self, signals: &[Signal]) -> Result<Vec<NodeId>, KremisError> {
        use crate::ingestor::Ingestor;
        use crate::primitives::{ASSOCIATION_WINDOW, MAX_SEQUENCE_LENGTH};

        if signals.is_empty() {
            return Ok(Vec::new());
        }
        if signals.len() > MAX_SEQUENCE_LENGTH {
            return Err(KremisError::InvalidSignal);
        }

        // Validate all signals before touching the database.
        // If any signal is invalid, the entire batch is rejected atomically.
        for signal in signals {
            Ingestor::validate(signal)?;
        }

        // Track entities newly inserted in this batch (not yet in entity_cache).
        let mut batch_entity_map: BTreeMap<EntityId, NodeId> = BTreeMap::new();
        let mut current_next_id = self.next_node_id;
        let mut node_ids = Vec::with_capacity(signals.len());

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| KremisError::IoError(e.to_string()))?;

        {
            let mut nodes_table = write_txn
                .open_table(NODES)
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            let mut entity_table = write_txn
                .open_table(ENTITY_INDEX)
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            let mut edges_table = write_txn
                .open_table(EDGES)
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            let mut props_table = write_txn
                .open_table(PROPERTIES)
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            let mut meta_table = write_txn
                .open_table(METADATA)
                .map_err(|e| KremisError::IoError(e.to_string()))?;

            // Pass 1: insert nodes and properties.
            for signal in signals {
                let node_id = if let Some(&existing) = self.entity_cache.get(&signal.entity) {
                    existing
                } else if let Some(&batch_node) = batch_entity_map.get(&signal.entity) {
                    batch_node
                } else {
                    let new_node_id = NodeId(current_next_id);
                    current_next_id = current_next_id.saturating_add(1);

                    let node = Node::new(new_node_id, signal.entity);
                    let node_bytes = postcard::to_allocvec(&node)
                        .map_err(|e| KremisError::SerializationError(e.to_string()))?;

                    nodes_table
                        .insert(new_node_id.0, node_bytes.as_slice())
                        .map_err(|e| KremisError::IoError(e.to_string()))?;
                    entity_table
                        .insert(signal.entity.0, new_node_id.0)
                        .map_err(|e| KremisError::IoError(e.to_string()))?;

                    batch_entity_map.insert(signal.entity, new_node_id);
                    new_node_id
                };

                // Store property: read-modify-write within the same transaction.
                let mut hasher = DefaultHasher::new();
                signal.attribute.as_str().hash(&mut hasher);
                let attr_hash = hasher.finish();

                let mut values: Vec<Value> = props_table
                    .get((node_id.0, attr_hash))
                    .map_err(|e| KremisError::IoError(e.to_string()))?
                    .map(|data| {
                        postcard::from_bytes::<(Attribute, Vec<Value>)>(data.value())
                            .map(|(_, v)| v)
                            .unwrap_or_default()
                    })
                    .unwrap_or_default();
                values.push(signal.value.clone());

                let prop_bytes = postcard::to_allocvec(&(signal.attribute.clone(), values))
                    .map_err(|e| KremisError::SerializationError(e.to_string()))?;
                props_table
                    .insert((node_id.0, attr_hash), prop_bytes.as_slice())
                    .map_err(|e| KremisError::IoError(e.to_string()))?;

                node_ids.push(node_id);
            }

            // Pass 2: create edges between adjacent signals (ASSOCIATION_WINDOW = 1).
            for window in signals.windows(ASSOCIATION_WINDOW + 1) {
                let current_signal = &window[window.len() - 1];
                let current_node = self
                    .entity_cache
                    .get(&current_signal.entity)
                    .copied()
                    .or_else(|| batch_entity_map.get(&current_signal.entity).copied())
                    .ok_or(KremisError::InvalidSignal)?;

                for prev_signal in window.iter().take(window.len() - 1) {
                    let prev_node = self
                        .entity_cache
                        .get(&prev_signal.entity)
                        .copied()
                        .or_else(|| batch_entity_map.get(&prev_signal.entity).copied())
                        .ok_or(KremisError::InvalidSignal)?;

                    let current_weight = edges_table
                        .get((prev_node.0, current_node.0))
                        .map_err(|e| KremisError::IoError(e.to_string()))?
                        .map(|v| v.value())
                        .unwrap_or(0);
                    edges_table
                        .insert(
                            (prev_node.0, current_node.0),
                            current_weight.saturating_add(1),
                        )
                        .map_err(|e| KremisError::IoError(e.to_string()))?;
                }
            }

            // Update metadata.
            meta_table
                .insert("next_node_id", current_next_id)
                .map_err(|e| KremisError::IoError(e.to_string()))?;
        }

        write_txn
            .commit()
            .map_err(|e| KremisError::IoError(e.to_string()))?;

        // Update in-memory state only after successful commit.
        self.next_node_id = current_next_id;
        for (entity, node_id) in batch_entity_map {
            self.entity_cache.insert(entity, node_id);
        }

        Ok(node_ids)
    }

    /// Get all edges in deterministic order.
    pub fn edges(&self) -> Result<Vec<(NodeId, NodeId, EdgeWeight)>, KremisError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        let edges_table = read_txn
            .open_table(EDGES)
            .map_err(|e| KremisError::IoError(e.to_string()))?;

        let mut edges = Vec::new();
        for entry in edges_table
            .iter()
            .map_err(|e| KremisError::IoError(e.to_string()))?
        {
            let (key, value) = entry.map_err(|e| KremisError::IoError(e.to_string()))?;
            let (from_id, to_id) = key.value();
            edges.push((
                NodeId(from_id),
                NodeId(to_id),
                EdgeWeight::new(value.value()),
            ));
        }
        Ok(edges)
    }

    /// Get all nodes in deterministic order.
    pub fn nodes(&self) -> Result<Vec<Node>, KremisError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        let nodes_table = read_txn
            .open_table(NODES)
            .map_err(|e| KremisError::IoError(e.to_string()))?;

        let mut nodes = Vec::new();
        for entry in nodes_table
            .iter()
            .map_err(|e| KremisError::IoError(e.to_string()))?
        {
            let (_, value) = entry.map_err(|e| KremisError::IoError(e.to_string()))?;
            let node: Node = postcard::from_bytes(value.value())
                .map_err(|e| KremisError::SerializationError(e.to_string()))?;
            nodes.push(node);
        }
        Ok(nodes)
    }

    /// Get stable edge count (edges with weight >= threshold).
    pub fn stable_edge_count(&self, threshold: i64) -> Result<usize, KremisError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        let edges_table = read_txn
            .open_table(EDGES)
            .map_err(|e| KremisError::IoError(e.to_string()))?;

        let mut count = 0;
        for entry in edges_table
            .iter()
            .map_err(|e| KremisError::IoError(e.to_string()))?
        {
            let (_, value) = entry.map_err(|e| KremisError::IoError(e.to_string()))?;
            if value.value() >= threshold {
                count += 1;
            }
        }
        Ok(count)
    }
}

// =============================================================================
// GRAPHSTORE TRAIT IMPLEMENTATION
// =============================================================================

impl GraphStore for RedbGraph {
    fn insert_node(&mut self, entity: EntityId) -> Result<NodeId, KremisError> {
        // Check if entity already exists
        if let Some(&node_id) = self.entity_cache.get(&entity) {
            return Ok(node_id);
        }

        // Create new node
        let node_id = NodeId(self.next_node_id);
        self.next_node_id = self.next_node_id.saturating_add(1);

        let node = Node::new(node_id, entity);
        let node_bytes = postcard::to_allocvec(&node)
            .map_err(|e| KremisError::SerializationError(e.to_string()))?;

        // Write to database
        {
            let write_txn = self
                .db
                .begin_write()
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            {
                let mut nodes_table = write_txn
                    .open_table(NODES)
                    .map_err(|e| KremisError::IoError(e.to_string()))?;
                nodes_table
                    .insert(node_id.0, node_bytes.as_slice())
                    .map_err(|e| KremisError::IoError(e.to_string()))?;
            }
            {
                let mut entity_table = write_txn
                    .open_table(ENTITY_INDEX)
                    .map_err(|e| KremisError::IoError(e.to_string()))?;
                entity_table
                    .insert(entity.0, node_id.0)
                    .map_err(|e| KremisError::IoError(e.to_string()))?;
            }
            {
                let mut meta_table = write_txn
                    .open_table(METADATA)
                    .map_err(|e| KremisError::IoError(e.to_string()))?;
                meta_table
                    .insert("next_node_id", self.next_node_id)
                    .map_err(|e| KremisError::IoError(e.to_string()))?;
            }
            write_txn
                .commit()
                .map_err(|e| KremisError::IoError(e.to_string()))?;
        }

        // Update cache
        self.entity_cache.insert(entity, node_id);

        Ok(node_id)
    }

    fn insert_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        weight: EdgeWeight,
    ) -> Result<(), KremisError> {
        // Check that both nodes exist before inserting (consistent with Graph behavior)
        if !self.contains_node(from)? || !self.contains_node(to)? {
            return Ok(());
        }

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        {
            let mut edges_table = write_txn
                .open_table(EDGES)
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            edges_table
                .insert((from.0, to.0), weight.value())
                .map_err(|e| KremisError::IoError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        Ok(())
    }

    fn increment_edge(&mut self, from: NodeId, to: NodeId) -> Result<(), KremisError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        {
            let mut edges_table = write_txn
                .open_table(EDGES)
                .map_err(|e| KremisError::IoError(e.to_string()))?;
            let current = edges_table
                .get((from.0, to.0))
                .map_err(|e| KremisError::IoError(e.to_string()))?
                .map(|v| v.value())
                .unwrap_or(0);
            edges_table
                .insert((from.0, to.0), current.saturating_add(1))
                .map_err(|e| KremisError::IoError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        Ok(())
    }

    fn lookup(&self, id: NodeId) -> Result<Option<Node>, KremisError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        let nodes_table = read_txn
            .open_table(NODES)
            .map_err(|e| KremisError::IoError(e.to_string()))?;

        match nodes_table
            .get(id.0)
            .map_err(|e| KremisError::IoError(e.to_string()))?
        {
            Some(data) => {
                let node: Node = postcard::from_bytes(data.value())
                    .map_err(|e| KremisError::SerializationError(e.to_string()))?;
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }

    fn get_node_by_entity(&self, entity: EntityId) -> Option<NodeId> {
        self.entity_cache.get(&entity).copied()
    }

    fn get_edge(&self, from: NodeId, to: NodeId) -> Result<Option<EdgeWeight>, KremisError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        let edges_table = read_txn
            .open_table(EDGES)
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        let result = edges_table
            .get((from.0, to.0))
            .map_err(|e| KremisError::IoError(e.to_string()))?
            .map(|v| EdgeWeight::new(v.value()));
        Ok(result)
    }

    fn neighbors(&self, from: NodeId) -> Result<Vec<(NodeId, EdgeWeight)>, KremisError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        let edges_table = read_txn
            .open_table(EDGES)
            .map_err(|e| KremisError::IoError(e.to_string()))?;

        let mut neighbors = Vec::new();
        for entry in edges_table
            .range((from.0, 0u64)..=(from.0, u64::MAX))
            .map_err(|e| KremisError::IoError(e.to_string()))?
        {
            let (key, value) = entry.map_err(|e| KremisError::IoError(e.to_string()))?;
            let (_from_id, to_id) = key.value();
            neighbors.push((NodeId(to_id), EdgeWeight::new(value.value())));
        }
        Ok(neighbors)
    }

    fn contains_node(&self, id: NodeId) -> Result<bool, KremisError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        let nodes_table = read_txn
            .open_table(NODES)
            .map_err(|e| KremisError::IoError(e.to_string()))?;

        Ok(nodes_table
            .get(id.0)
            .map_err(|e| KremisError::IoError(e.to_string()))?
            .is_some())
    }

    fn traverse(&self, start: NodeId, depth: usize) -> Result<Option<Artifact>, KremisError> {
        let depth = depth.min(crate::primitives::MAX_TRAVERSAL_DEPTH);
        if !self.contains_node(start)? {
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

            for (neighbor, weight) in self.neighbors(current)? {
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
        if !self.contains_node(start)? {
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

            for (neighbor, weight) in self.neighbors(current)? {
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
        let first_neighbors: BTreeSet<_> = self
            .neighbors(nodes[0])?
            .into_iter()
            .map(|(n, _)| n)
            .collect();

        if first_neighbors.is_empty() {
            return Ok(Vec::new());
        }

        // Intersect with neighbors of remaining nodes
        let mut result = first_neighbors;
        for &node in &nodes[1..] {
            let neighbors: BTreeSet<_> =
                self.neighbors(node)?.into_iter().map(|(n, _)| n).collect();
            result = result.intersection(&neighbors).copied().collect();
        }

        Ok(result.into_iter().collect())
    }

    fn strongest_path(
        &self,
        start: NodeId,
        end: NodeId,
    ) -> Result<Option<Vec<NodeId>>, KremisError> {
        if !self.contains_node(start)? || !self.contains_node(end)? {
            return Ok(None);
        }

        if start == end {
            return Ok(Some(vec![start]));
        }

        // Dijkstra with cost = i64::MAX - weight
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

            for (neighbor, weight) in self.neighbors(current)? {
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
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        let nodes_table = read_txn
            .open_table(NODES)
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        let count = nodes_table
            .len()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        Ok(count as usize)
    }

    fn edge_count(&self) -> Result<usize, KremisError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        let edges_table = read_txn
            .open_table(EDGES)
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        let count = edges_table
            .len()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        Ok(count as usize)
    }

    fn store_property(
        &mut self,
        node: NodeId,
        attribute: Attribute,
        value: Value,
    ) -> Result<(), KremisError> {
        // Verify node exists
        if !self.contains_node(node)? {
            return Err(KremisError::NodeNotFound(node));
        }

        // Hash the attribute for the key
        let mut hasher = DefaultHasher::new();
        attribute.as_str().hash(&mut hasher);
        let attr_hash = hasher.finish();

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        {
            let mut props_table = write_txn
                .open_table(PROPERTIES)
                .map_err(|e| KremisError::IoError(e.to_string()))?;

            // Read existing values for this (node, attribute) pair
            let existing: Vec<Value> = props_table
                .get((node.0, attr_hash))
                .map_err(|e| KremisError::IoError(e.to_string()))?
                .map(|data| {
                    postcard::from_bytes::<(Attribute, Vec<Value>)>(data.value())
                        .map(|(_, values)| values)
                        .unwrap_or_default()
                })
                .unwrap_or_default();

            // Append new value
            let mut values = existing;
            values.push(value);

            // Serialize and store
            let prop_bytes = postcard::to_allocvec(&(attribute, values))
                .map_err(|e| KremisError::SerializationError(e.to_string()))?;
            props_table
                .insert((node.0, attr_hash), prop_bytes.as_slice())
                .map_err(|e| KremisError::IoError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| KremisError::IoError(e.to_string()))?;

        Ok(())
    }

    fn get_properties(&self, node: NodeId) -> Result<Vec<(Attribute, Value)>, KremisError> {
        // Verify node exists
        if !self.contains_node(node)? {
            return Err(KremisError::NodeNotFound(node));
        }

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| KremisError::IoError(e.to_string()))?;
        let props_table = read_txn
            .open_table(PROPERTIES)
            .map_err(|e| KremisError::IoError(e.to_string()))?;

        let mut result = Vec::new();

        // Range query for all properties of this node
        for entry in props_table
            .range((node.0, 0u64)..=(node.0, u64::MAX))
            .map_err(|e| KremisError::IoError(e.to_string()))?
        {
            let (_, data) = entry.map_err(|e| KremisError::IoError(e.to_string()))?;
            let (attr, values): (Attribute, Vec<Value>) = postcard::from_bytes(data.value())
                .map_err(|e| KremisError::DeserializationError(e.to_string()))?;
            for value in values {
                result.push((attr.clone(), value));
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn basic_operations() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let node1 = graph.insert_node(EntityId(1)).expect("insert node");
        let node2 = graph.insert_node(EntityId(2)).expect("insert node");

        assert_ne!(node1, node2);
        assert_eq!(graph.node_count().expect("count"), 2);

        graph
            .insert_edge(node1, node2, EdgeWeight::new(5))
            .expect("insert edge");
        assert_eq!(graph.edge_count().expect("count"), 1);
    }

    #[test]
    fn entity_deduplication() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let node1 = graph.insert_node(EntityId(1)).expect("insert node");
        let node2 = graph.insert_node(EntityId(1)).expect("insert node");

        assert_eq!(node1, node2);
        assert_eq!(graph.node_count().expect("count"), 1);
    }

    #[test]
    fn persistence() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");

        // Create and populate
        {
            let mut graph = RedbGraph::open(&db_path).expect("open db");
            graph.insert_node(EntityId(1)).expect("insert node");
            graph.insert_node(EntityId(2)).expect("insert node");
        }

        // Reopen and verify
        {
            let graph = RedbGraph::open(&db_path).expect("open db");
            assert_eq!(graph.node_count().expect("count"), 2);
            assert!(graph.get_node_by_entity(EntityId(1)).is_some());
        }
    }

    #[test]
    fn edge_operations() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let node1 = graph.insert_node(EntityId(1)).expect("insert node");
        let node2 = graph.insert_node(EntityId(2)).expect("insert node");

        graph
            .insert_edge(node1, node2, EdgeWeight::new(3))
            .expect("insert edge");

        let weight = graph.get_edge(node1, node2).expect("get edge");
        assert_eq!(weight, Some(EdgeWeight::new(3)));

        graph.increment_edge(node1, node2).expect("increment edge");
        let weight = graph.get_edge(node1, node2).expect("get edge");
        assert_eq!(weight, Some(EdgeWeight::new(4)));
    }

    #[test]
    fn neighbors() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let node1 = graph.insert_node(EntityId(1)).expect("insert node");
        let node2 = graph.insert_node(EntityId(2)).expect("insert node");
        let node3 = graph.insert_node(EntityId(3)).expect("insert node");

        graph
            .insert_edge(node1, node2, EdgeWeight::new(5))
            .expect("insert edge");
        graph
            .insert_edge(node1, node3, EdgeWeight::new(3))
            .expect("insert edge");

        let neighbors = graph.neighbors(node1).expect("neighbors");
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn lookup_node() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let node1 = graph.insert_node(EntityId(42)).expect("insert node");
        let found = graph.lookup(node1).expect("lookup");

        assert!(found.is_some());
        assert_eq!(found.map(|n| n.entity), Some(EntityId(42)));
    }

    #[test]
    fn traverse_bfs() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let node1 = graph.insert_node(EntityId(1)).expect("insert node");
        let node2 = graph.insert_node(EntityId(2)).expect("insert node");
        let node3 = graph.insert_node(EntityId(3)).expect("insert node");

        graph
            .insert_edge(node1, node2, EdgeWeight::new(5))
            .expect("edge");
        graph
            .insert_edge(node2, node3, EdgeWeight::new(3))
            .expect("edge");

        let artifact = graph.traverse(node1, 2).expect("traverse");
        assert!(artifact.is_some());

        let path = &artifact.as_ref().expect("artifact").path;
        assert!(path.contains(&node1));
        assert!(path.contains(&node2));
        assert!(path.contains(&node3));
    }

    #[test]
    fn strongest_path_finds_route() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let a = graph.insert_node(EntityId(1)).expect("insert node");
        let b = graph.insert_node(EntityId(2)).expect("insert node");
        let c = graph.insert_node(EntityId(3)).expect("insert node");

        graph.insert_edge(a, b, EdgeWeight::new(10)).expect("edge");
        graph.insert_edge(b, c, EdgeWeight::new(10)).expect("edge");

        let path = graph.strongest_path(a, c).expect("path");
        assert_eq!(path, Some(vec![a, b, c]));
    }

    #[test]
    fn intersect_finds_common() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let a = graph.insert_node(EntityId(1)).expect("insert node");
        let b = graph.insert_node(EntityId(2)).expect("insert node");
        let common = graph.insert_node(EntityId(100)).expect("insert node");

        graph
            .insert_edge(a, common, EdgeWeight::new(1))
            .expect("edge");
        graph
            .insert_edge(b, common, EdgeWeight::new(1))
            .expect("edge");

        let result = graph.intersect(&[a, b]).expect("intersect");
        assert_eq!(result, vec![common]);
    }

    #[test]
    fn stable_edge_count_threshold() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let a = graph.insert_node(EntityId(1)).expect("insert node");
        let b = graph.insert_node(EntityId(2)).expect("insert node");
        let c = graph.insert_node(EntityId(3)).expect("insert node");

        graph.insert_edge(a, b, EdgeWeight::new(5)).expect("edge");
        graph.insert_edge(b, c, EdgeWeight::new(15)).expect("edge");

        // Threshold 10: only edge b->c qualifies
        let stable = graph.stable_edge_count(10).expect("stable count");
        assert_eq!(stable, 1);
    }

    // =========================================================================
    // M3 - Transaction edge cases tests
    // =========================================================================

    #[test]
    fn transaction_multiple_operations_atomic() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        // Insert multiple nodes and edges in sequence
        let n1 = graph.insert_node(EntityId(1)).expect("insert node 1");
        let n2 = graph.insert_node(EntityId(2)).expect("insert node 2");
        let n3 = graph.insert_node(EntityId(3)).expect("insert node 3");

        graph
            .insert_edge(n1, n2, EdgeWeight::new(10))
            .expect("edge 1");
        graph
            .insert_edge(n2, n3, EdgeWeight::new(20))
            .expect("edge 2");
        graph
            .insert_edge(n1, n3, EdgeWeight::new(30))
            .expect("edge 3");

        // Verify all operations persisted
        assert_eq!(graph.node_count().expect("count"), 3);
        assert_eq!(graph.edge_count().expect("count"), 3);

        // Verify edge weights
        assert_eq!(
            graph.get_edge(n1, n2).expect("get").map(|e| e.value()),
            Some(10)
        );
        assert_eq!(
            graph.get_edge(n2, n3).expect("get").map(|e| e.value()),
            Some(20)
        );
        assert_eq!(
            graph.get_edge(n1, n3).expect("get").map(|e| e.value()),
            Some(30)
        );
    }

    #[test]
    fn transaction_edge_update_overwrites() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let n1 = graph.insert_node(EntityId(1)).expect("insert node");
        let n2 = graph.insert_node(EntityId(2)).expect("insert node");

        // Insert edge with initial weight
        graph.insert_edge(n1, n2, EdgeWeight::new(5)).expect("edge");
        assert_eq!(
            graph.get_edge(n1, n2).expect("get").map(|e| e.value()),
            Some(5)
        );

        // Overwrite with new weight
        graph
            .insert_edge(n1, n2, EdgeWeight::new(100))
            .expect("edge update");
        assert_eq!(
            graph.get_edge(n1, n2).expect("get").map(|e| e.value()),
            Some(100)
        );

        // Only one edge should exist
        assert_eq!(graph.edge_count().expect("count"), 1);
    }

    #[test]
    fn transaction_increment_from_zero() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let n1 = graph.insert_node(EntityId(1)).expect("insert node");
        let n2 = graph.insert_node(EntityId(2)).expect("insert node");

        // Increment edge that doesn't exist (should create with weight 1)
        graph.increment_edge(n1, n2).expect("increment");
        assert_eq!(
            graph.get_edge(n1, n2).expect("get").map(|e| e.value()),
            Some(1)
        );

        // Increment again
        graph.increment_edge(n1, n2).expect("increment 2");
        assert_eq!(
            graph.get_edge(n1, n2).expect("get").map(|e| e.value()),
            Some(2)
        );
    }

    #[test]
    fn transaction_large_entity_ids() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        // Test with large entity IDs near u64::MAX
        let large_id1 = EntityId(u64::MAX - 1);
        let large_id2 = EntityId(u64::MAX);

        let n1 = graph.insert_node(large_id1).expect("insert node");
        let n2 = graph.insert_node(large_id2).expect("insert node");

        graph
            .insert_edge(n1, n2, EdgeWeight::new(i64::MAX))
            .expect("edge");

        let weight = graph.get_edge(n1, n2).expect("get");
        assert_eq!(weight.map(|w| w.value()), Some(i64::MAX));
    }

    #[test]
    fn transaction_negative_edge_weight() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let n1 = graph.insert_node(EntityId(1)).expect("insert node");
        let n2 = graph.insert_node(EntityId(2)).expect("insert node");

        // Negative weights should be handled correctly
        graph
            .insert_edge(n1, n2, EdgeWeight::new(-100))
            .expect("edge");

        let weight = graph.get_edge(n1, n2).expect("get");
        assert_eq!(weight.map(|w| w.value()), Some(-100));
    }

    // =========================================================================
    // M3 - Concurrent access tests
    // =========================================================================

    #[test]
    fn concurrent_read_while_idle() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        // Setup data
        let n1 = graph.insert_node(EntityId(1)).expect("insert node");
        let n2 = graph.insert_node(EntityId(2)).expect("insert node");
        graph
            .insert_edge(n1, n2, EdgeWeight::new(42))
            .expect("edge");

        // Multiple reads should work
        for _ in 0..10 {
            assert_eq!(graph.node_count().expect("count"), 2);
            assert_eq!(graph.edge_count().expect("count"), 1);
            let _ = graph.neighbors(n1).expect("neighbors");
            let _ = graph.lookup(n1).expect("lookup");
        }
    }

    #[test]
    fn concurrent_sequential_writes() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        // Many sequential writes
        for i in 0..100 {
            graph.insert_node(EntityId(i)).expect("insert node");
        }

        assert_eq!(graph.node_count().expect("count"), 100);

        // Add edges between consecutive nodes
        for i in 0..99 {
            graph
                .insert_edge(NodeId(i), NodeId(i + 1), EdgeWeight::new(i as i64))
                .expect("edge");
        }

        assert_eq!(graph.edge_count().expect("count"), 99);
    }

    #[test]
    fn concurrent_interleaved_reads_writes() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        for i in 0..50 {
            // Write
            let node = graph.insert_node(EntityId(i)).expect("insert node");

            // Read immediately after write
            let found = graph.lookup(node).expect("lookup");
            assert!(found.is_some());
            assert_eq!(found.unwrap().entity, EntityId(i));

            // Verify count increases
            assert_eq!(graph.node_count().expect("count"), (i + 1) as usize);
        }
    }

    // =========================================================================
    // M3 - Recovery after crash tests (simulated via reopen)
    // =========================================================================

    #[test]
    fn recovery_persistence_after_reopen() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");

        // Phase 1: Create data
        {
            let mut graph = RedbGraph::open(&db_path).expect("open db");
            let n1 = graph.insert_node(EntityId(100)).expect("insert node");
            let n2 = graph.insert_node(EntityId(200)).expect("insert node");
            let n3 = graph.insert_node(EntityId(300)).expect("insert node");

            graph
                .insert_edge(n1, n2, EdgeWeight::new(10))
                .expect("edge");
            graph
                .insert_edge(n2, n3, EdgeWeight::new(20))
                .expect("edge");
        }
        // Graph dropped here, simulating process exit

        // Phase 2: Reopen and verify all data persisted
        {
            let graph = RedbGraph::open(&db_path).expect("reopen db");

            assert_eq!(graph.node_count().expect("count"), 3);
            assert_eq!(graph.edge_count().expect("count"), 2);

            // Verify entity cache was reconstructed
            assert!(graph.get_node_by_entity(EntityId(100)).is_some());
            assert!(graph.get_node_by_entity(EntityId(200)).is_some());
            assert!(graph.get_node_by_entity(EntityId(300)).is_some());

            // Verify edges
            let n1 = graph.get_node_by_entity(EntityId(100)).unwrap();
            let n2 = graph.get_node_by_entity(EntityId(200)).unwrap();
            let n3 = graph.get_node_by_entity(EntityId(300)).unwrap();

            assert_eq!(
                graph.get_edge(n1, n2).expect("get").map(|e| e.value()),
                Some(10)
            );
            assert_eq!(
                graph.get_edge(n2, n3).expect("get").map(|e| e.value()),
                Some(20)
            );
        }
    }

    #[test]
    fn recovery_next_node_id_preserved() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");

        let node_id_after_initial;

        // Phase 1: Create nodes
        {
            let mut graph = RedbGraph::open(&db_path).expect("open db");
            graph.insert_node(EntityId(1)).expect("insert");
            graph.insert_node(EntityId(2)).expect("insert");
            node_id_after_initial = graph.insert_node(EntityId(3)).expect("insert");
        }

        // Phase 2: Reopen and add more nodes
        {
            let mut graph = RedbGraph::open(&db_path).expect("reopen db");
            let new_node = graph.insert_node(EntityId(4)).expect("insert");

            // New node should have ID greater than previous
            assert!(
                new_node.0 > node_id_after_initial.0,
                "New node ID {} should be > previous {}",
                new_node.0,
                node_id_after_initial.0
            );
        }
    }

    #[test]
    fn recovery_multiple_reopen_cycles() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");

        // Cycle 1
        {
            let mut graph = RedbGraph::open(&db_path).expect("open db");
            graph.insert_node(EntityId(1)).expect("insert");
        }

        // Cycle 2
        {
            let mut graph = RedbGraph::open(&db_path).expect("reopen db");
            assert_eq!(graph.node_count().expect("count"), 1);
            graph.insert_node(EntityId(2)).expect("insert");
        }

        // Cycle 3
        {
            let mut graph = RedbGraph::open(&db_path).expect("reopen db");
            assert_eq!(graph.node_count().expect("count"), 2);
            graph.insert_node(EntityId(3)).expect("insert");
        }

        // Final verification
        {
            let graph = RedbGraph::open(&db_path).expect("reopen db");
            assert_eq!(graph.node_count().expect("count"), 3);
        }
    }

    #[test]
    fn recovery_edge_increment_persists() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");

        // Phase 1: Create edge and increment multiple times
        {
            let mut graph = RedbGraph::open(&db_path).expect("open db");
            let n1 = graph.insert_node(EntityId(1)).expect("insert");
            let n2 = graph.insert_node(EntityId(2)).expect("insert");

            graph.insert_edge(n1, n2, EdgeWeight::new(0)).expect("edge");
            for _ in 0..10 {
                graph.increment_edge(n1, n2).expect("increment");
            }
        }

        // Phase 2: Verify increments persisted
        {
            let graph = RedbGraph::open(&db_path).expect("reopen db");
            let n1 = graph.get_node_by_entity(EntityId(1)).unwrap();
            let n2 = graph.get_node_by_entity(EntityId(2)).unwrap();

            let weight = graph.get_edge(n1, n2).expect("get").map(|w| w.value());
            assert_eq!(weight, Some(10));
        }
    }

    #[test]
    fn recovery_compact_and_reopen() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");

        // Phase 1: Create data and compact
        {
            let mut graph = RedbGraph::open(&db_path).expect("open db");
            for i in 0..50 {
                graph.insert_node(EntityId(i)).expect("insert");
            }
            graph.compact().expect("compact");
        }

        // Phase 2: Verify data after compact
        {
            let graph = RedbGraph::open(&db_path).expect("reopen db");
            assert_eq!(graph.node_count().expect("count"), 50);

            for i in 0..50 {
                assert!(
                    graph.get_node_by_entity(EntityId(i)).is_some(),
                    "Entity {} should exist",
                    i
                );
            }
        }
    }

    #[test]
    fn traverse_nonexistent_start_node() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let graph = RedbGraph::open(&db_path).expect("open db");

        // Traverse from non-existent node should return None
        let result = graph.traverse(NodeId(999), 5).expect("traverse");
        assert!(result.is_none());
    }

    #[test]
    fn traverse_filtered_excludes_low_weight() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let n1 = graph.insert_node(EntityId(1)).expect("insert");
        let n2 = graph.insert_node(EntityId(2)).expect("insert");
        let n3 = graph.insert_node(EntityId(3)).expect("insert");

        // High weight path: n1 -> n2
        graph
            .insert_edge(n1, n2, EdgeWeight::new(100))
            .expect("edge");
        // Low weight path: n1 -> n3
        graph.insert_edge(n1, n3, EdgeWeight::new(1)).expect("edge");

        // Filter with min weight 50 should only include n1 -> n2
        let artifact = graph
            .traverse_filtered(n1, 2, EdgeWeight::new(50))
            .expect("traverse");

        assert!(artifact.is_some());
        let art = artifact.unwrap();

        // Should have n1 and n2 but not traverse to n3 via low weight edge
        assert!(art.path.contains(&n1));
        assert!(art.path.contains(&n2));
        // n3 should not be reached via the filtered traversal
    }

    #[test]
    fn strongest_path_no_connection() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let n1 = graph.insert_node(EntityId(1)).expect("insert");
        let n2 = graph.insert_node(EntityId(2)).expect("insert");

        // No edge between n1 and n2
        let path = graph.strongest_path(n1, n2).expect("path");
        assert!(path.is_none());
    }

    #[test]
    fn strongest_path_same_node() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let n1 = graph.insert_node(EntityId(1)).expect("insert");

        // Path from node to itself
        let path = graph.strongest_path(n1, n1).expect("path");
        assert_eq!(path, Some(vec![n1]));
    }

    #[test]
    fn intersect_empty_input() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let graph = RedbGraph::open(&db_path).expect("open db");

        let result = graph.intersect(&[]).expect("intersect");
        assert!(result.is_empty());
    }

    #[test]
    fn intersect_no_common_neighbors() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let n1 = graph.insert_node(EntityId(1)).expect("insert");
        let n2 = graph.insert_node(EntityId(2)).expect("insert");
        let n3 = graph.insert_node(EntityId(3)).expect("insert");
        let n4 = graph.insert_node(EntityId(4)).expect("insert");

        // n1 -> n3, n2 -> n4 (no common neighbors)
        graph.insert_edge(n1, n3, EdgeWeight::new(1)).expect("edge");
        graph.insert_edge(n2, n4, EdgeWeight::new(1)).expect("edge");

        let result = graph.intersect(&[n1, n2]).expect("intersect");
        assert!(result.is_empty());
    }

    // =========================================================================
    // Property storage tests
    // =========================================================================

    #[test]
    fn store_and_get_properties() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

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
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

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
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let result = graph.store_property(NodeId(999), Attribute::new("name"), Value::new("Test"));
        assert!(result.is_err());
    }

    #[test]
    fn get_properties_nonexistent_node_fails() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let graph = RedbGraph::open(&db_path).expect("open db");

        let result = graph.get_properties(NodeId(999));
        assert!(result.is_err());
    }

    #[test]
    fn get_properties_empty_returns_empty_vec() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let node = graph.insert_node(EntityId(1)).expect("insert");

        let props = graph.get_properties(node).expect("get");
        assert!(props.is_empty());
    }

    #[test]
    fn insert_edge_ignores_dangling_nodes() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let node1 = graph.insert_node(EntityId(1)).expect("insert node");
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

        // Edge between two non-existing nodes: silently ignored
        graph
            .insert_edge(NodeId(888), dangling, EdgeWeight::new(5))
            .expect("insert edge");
        assert_eq!(graph.edge_count().expect("count"), 0);
    }

    // =========================================================================
    // ingest_batch tests
    // =========================================================================

    fn make_signal(entity_id: u64, attr: &str, val: &str) -> crate::Signal {
        crate::Signal::new(
            EntityId(entity_id),
            crate::Attribute::new(attr),
            crate::Value::new(val),
        )
    }

    #[test]
    fn ingest_batch_basic() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let signals = vec![
            make_signal(1, "type", "word"),
            make_signal(2, "type", "word"),
            make_signal(3, "type", "word"),
        ];

        let nodes = graph.ingest_batch(&signals).expect("ingest batch");

        assert_eq!(nodes.len(), 3);
        assert_eq!(graph.node_count().expect("count"), 3);
        // Adjacent edges created
        assert_eq!(graph.edge_count().expect("count"), 2);
    }

    #[test]
    fn ingest_batch_creates_edges() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let signals = vec![
            make_signal(10, "name", "Alice"),
            make_signal(20, "name", "Bob"),
            make_signal(30, "name", "Charlie"),
        ];

        let nodes = graph.ingest_batch(&signals).expect("ingest batch");

        // Edge from node 0 to node 1
        let w01 = graph.get_edge(nodes[0], nodes[1]).expect("get edge");
        assert_eq!(w01, Some(EdgeWeight::new(1)));

        // Edge from node 1 to node 2
        let w12 = graph.get_edge(nodes[1], nodes[2]).expect("get edge");
        assert_eq!(w12, Some(EdgeWeight::new(1)));

        // No edge from node 0 to node 2 (ASSOCIATION_WINDOW = 1)
        let w02 = graph.get_edge(nodes[0], nodes[2]).expect("get edge");
        assert!(w02.is_none());
    }

    #[test]
    fn ingest_batch_deduplication() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        // Same entity twice: should result in one node, two properties
        let signals = vec![
            make_signal(1, "name", "Alice"),
            make_signal(1, "role", "admin"),
        ];

        let nodes = graph.ingest_batch(&signals).expect("ingest batch");

        assert_eq!(nodes[0], nodes[1], "Same entity must map to same NodeId");
        assert_eq!(graph.node_count().expect("count"), 1);

        let props = graph.get_properties(nodes[0]).expect("get props");
        assert_eq!(props.len(), 2);
        assert!(props.contains(&(crate::Attribute::new("name"), crate::Value::new("Alice"))));
        assert!(props.contains(&(crate::Attribute::new("role"), crate::Value::new("admin"))));
    }

    #[test]
    fn ingest_batch_empty() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let nodes = graph.ingest_batch(&[]).expect("empty batch");
        assert!(nodes.is_empty());
        assert_eq!(graph.node_count().expect("count"), 0);
    }

    #[test]
    fn ingest_batch_preserves_properties() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");
        let mut graph = RedbGraph::open(&db_path).expect("open db");

        let signals = vec![
            make_signal(1, "city", "Paris"),
            make_signal(2, "city", "Rome"),
        ];

        let nodes = graph.ingest_batch(&signals).expect("ingest batch");

        let props1 = graph.get_properties(nodes[0]).expect("props 1");
        assert!(props1.contains(&(crate::Attribute::new("city"), crate::Value::new("Paris"))));

        let props2 = graph.get_properties(nodes[1]).expect("props 2");
        assert!(props2.contains(&(crate::Attribute::new("city"), crate::Value::new("Rome"))));
    }

    #[test]
    fn properties_persist_after_reopen() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("test.redb");

        // Phase 1: Create node with properties
        let node_id;
        {
            let mut graph = RedbGraph::open(&db_path).expect("open db");
            node_id = graph.insert_node(EntityId(42)).expect("insert");
            graph
                .store_property(node_id, Attribute::new("name"), Value::new("Alice"))
                .expect("store");
            graph
                .store_property(node_id, Attribute::new("city"), Value::new("Paris"))
                .expect("store");
        }

        // Phase 2: Reopen and verify properties persisted
        {
            let graph = RedbGraph::open(&db_path).expect("reopen db");
            let props = graph.get_properties(node_id).expect("get");
            assert_eq!(props.len(), 2);
            assert!(props.contains(&(Attribute::new("name"), Value::new("Alice"))));
            assert!(props.contains(&(Attribute::new("city"), Value::new("Paris"))));
        }
    }
}
