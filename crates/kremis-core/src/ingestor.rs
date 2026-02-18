//! # Ingestor Module
//!
//! Signal validation and ingestion protocol for Kremis CORE.
//!
//! - Validate signals before graph mutation
//! - Reject malformed input
//! - Deduplicate identical signals
//! - No semantic inference or enrichment

use crate::graph::GraphStore;
use crate::primitives::{
    ASSOCIATION_WINDOW, MAX_ATTRIBUTE_LENGTH, MAX_SEQUENCE_LENGTH, MAX_VALUE_LENGTH,
};
use crate::{KremisError, NodeId, Signal};

/// The Ingestor handles signal validation and graph ingestion.
///
/// The Ingestor:
/// - Accepts raw input from Facets
/// - Sanitizes and validates signals
/// - Reduces input to strict Logical Primitives
pub struct Ingestor;

impl Ingestor {
    /// Validate a signal.
    ///
    /// A signal is valid if:
    /// - Entity ID is present
    /// - Attribute is non-empty and within length limits
    /// - Value is non-empty and within length limits
    ///
    /// Returns `KremisError::InvalidSignal` if validation fails.
    pub fn validate(signal: &Signal) -> Result<(), KremisError> {
        let attr = signal.attribute.as_str();
        let val = signal.value.as_str();

        // Attribute must be non-empty
        if attr.is_empty() {
            return Err(KremisError::InvalidSignal);
        }

        // Attribute length check
        if attr.len() > MAX_ATTRIBUTE_LENGTH {
            return Err(KremisError::InvalidSignal);
        }

        // Value must be non-empty
        if val.is_empty() {
            return Err(KremisError::InvalidSignal);
        }

        // Value length check
        if val.len() > MAX_VALUE_LENGTH {
            return Err(KremisError::InvalidSignal);
        }

        Ok(())
    }

    /// Ingest a single signal into any graph store.
    ///
    /// Works with both in-memory Graph and persistent RedbGraph.
    /// Returns the NodeId of the entity node.
    ///
    /// The signal's attribute and value are stored as properties of the node,
    /// preserving the full signal data for later retrieval.
    pub fn ingest_signal<G: GraphStore>(
        graph: &mut G,
        signal: &Signal,
    ) -> Result<NodeId, KremisError> {
        Self::validate(signal)?;

        // Get or create node for the entity
        let node_id = graph.insert_node(signal.entity)?;

        // Store the attribute and value as properties
        graph.store_property(node_id, signal.attribute.clone(), signal.value.clone())?;

        Ok(node_id)
    }

    /// Ingest a sequence of signals with automatic edge creation.
    ///
    /// Works with both in-memory Graph and persistent RedbGraph.
    /// Edges are formed between adjacent signals
    /// within the ASSOCIATION_WINDOW (= 1).
    ///
    /// Returns the list of NodeIds created/updated.
    ///
    /// # Errors
    /// Returns `KremisError::InvalidSignal` if:
    /// - The sequence exceeds `MAX_SEQUENCE_LENGTH`
    /// - Any signal in the sequence is invalid
    pub fn ingest_sequence<G: GraphStore>(
        graph: &mut G,
        signals: &[Signal],
    ) -> Result<Vec<NodeId>, KremisError> {
        if signals.is_empty() {
            return Ok(Vec::new());
        }

        // Sequence length check
        if signals.len() > MAX_SEQUENCE_LENGTH {
            return Err(KremisError::InvalidSignal);
        }

        let mut node_ids = Vec::with_capacity(signals.len());

        // Ingest first signal
        let first_node = Self::ingest_signal(graph, &signals[0])?;
        node_ids.push(first_node);

        // Ingest remaining signals with edge creation
        for window in signals.windows(ASSOCIATION_WINDOW + 1) {
            let current_signal = &window[window.len() - 1];
            let current_node = Self::ingest_signal(graph, current_signal)?;
            node_ids.push(current_node);

            // Create edges from all previous signals in window to current
            for prev_signal in window.iter().take(window.len() - 1) {
                if let Some(prev_node) = graph.get_node_by_entity(prev_signal.entity) {
                    graph.increment_edge(prev_node, current_node)?;
                }
            }
        }

        Ok(node_ids)
    }

    /// Check if a signal would be a duplicate.
    ///
    /// A signal is a duplicate if:
    /// - The entity already exists as a node
    /// - (Optional: same attribute/value combination exists)
    #[must_use]
    pub fn is_duplicate<G: GraphStore>(graph: &G, signal: &Signal) -> bool {
        graph.get_node_by_entity(signal.entity).is_some()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    use crate::{Attribute, EntityId, Value};

    fn make_signal(entity_id: u64, attr: &str, val: &str) -> Signal {
        Signal::new(EntityId(entity_id), Attribute::new(attr), Value::new(val))
    }

    #[test]
    fn validate_rejects_empty_attribute() {
        let signal = make_signal(1, "", "value");
        assert!(Ingestor::validate(&signal).is_err());
    }

    #[test]
    fn validate_rejects_empty_value() {
        let signal = make_signal(1, "attr", "");
        assert!(Ingestor::validate(&signal).is_err());
    }

    #[test]
    fn validate_accepts_valid_signal() {
        let signal = make_signal(1, "name", "Alice");
        assert!(Ingestor::validate(&signal).is_ok());
    }

    #[test]
    fn ingest_signal_creates_node() {
        let mut graph = Graph::new();
        let signal = make_signal(42, "name", "Bob");

        let node_id = Ingestor::ingest_signal(&mut graph, &signal).expect("ingest");

        assert!(graph.lookup(node_id).expect("lookup").is_some());
        assert_eq!(graph.node_count().expect("count"), 1);
    }

    #[test]
    fn ingest_sequence_creates_edges() {
        let mut graph = Graph::new();
        let signals = vec![
            make_signal(1, "type", "word"),
            make_signal(2, "type", "word"),
            make_signal(3, "type", "word"),
        ];

        let nodes = Ingestor::ingest_sequence(&mut graph, &signals).expect("ingest");

        assert_eq!(nodes.len(), 3);
        // Edge from node 0 to node 1
        assert!(graph.get_edge(nodes[0], nodes[1]).expect("get").is_some());
        // Edge from node 1 to node 2
        assert!(graph.get_edge(nodes[1], nodes[2]).expect("get").is_some());
    }

    #[test]
    fn is_duplicate_detects_existing_entity() {
        let mut graph = Graph::new();
        let signal = make_signal(1, "name", "Alice");

        assert!(!Ingestor::is_duplicate(&graph, &signal));

        Ingestor::ingest_signal(&mut graph, &signal).expect("ingest");

        assert!(Ingestor::is_duplicate(&graph, &signal));
    }

    #[test]
    fn ingest_signal_stores_properties() {
        use crate::graph::GraphStore;

        let mut graph = Graph::new();
        let signal = make_signal(1, "name", "Alice");

        let node_id = Ingestor::ingest_signal(&mut graph, &signal).expect("ingest");

        // Verify the property was stored
        let props = graph.get_properties(node_id).expect("get properties");
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].0.as_str(), "name");
        assert_eq!(props[0].1.as_str(), "Alice");
    }

    #[test]
    fn ingest_sequence_stores_all_properties() {
        use crate::graph::GraphStore;

        let mut graph = Graph::new();
        let signals = vec![
            make_signal(1, "name", "Alice"),
            make_signal(2, "name", "Bob"),
        ];

        let nodes = Ingestor::ingest_sequence(&mut graph, &signals).expect("ingest");

        // Verify properties for first node
        let props1 = graph.get_properties(nodes[0]).expect("get properties");
        assert_eq!(props1.len(), 1);
        assert_eq!(props1[0].0.as_str(), "name");
        assert_eq!(props1[0].1.as_str(), "Alice");

        // Verify properties for second node
        let props2 = graph.get_properties(nodes[1]).expect("get properties");
        assert_eq!(props2.len(), 1);
        assert_eq!(props2[0].0.as_str(), "name");
        assert_eq!(props2[0].1.as_str(), "Bob");
    }
}
