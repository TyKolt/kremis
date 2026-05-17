//! # Verifiable Query Certificate (VQC)
//!
//! A certificate is a pure, deterministic function of `(graph state, query)`.
//! It carries the minimal evidence needed to re-derive a query result, plus a
//! cryptographic hash of the canonical graph state, so a third party can
//! re-verify the answer offline — including the proof-of-absence case where
//! the answer is `unknown`.
//!
//! This module is part of the pure CORE: no async, no network, no clock, no
//! randomness, no floating-point. The serialized form reuses the canonical
//! (`KREX`) primitives from [`crate::export`], so an independent
//! implementation reproducing those bytes reproduces the certificate.
//!
//! Specification: `docs/concepts/certificate-spec.mdx`.

use crate::export::{CanonicalEdge, CanonicalGraph, CanonicalNode};
use crate::graph::Graph;
use crate::{Artifact, KremisError, NodeId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Magic bytes for the certificate format ("Kremis Verifiable Query Cert").
pub const CERT_MAGIC: [u8; 4] = *b"KVQC";

/// Current certificate format version.
pub const CERT_VERSION: u8 = 1;

/// Header for a serialized certificate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CertHeader {
    /// Magic bytes identifying the format.
    pub magic: [u8; 4],
    /// Format version for compatibility.
    pub version: u8,
    /// BLAKE3 hash of the canonical (`KREX`) export of the graph state.
    pub state_hash: [u8; 32],
}

/// Body of a serialized certificate.
///
/// `evidence_nodes` and `evidence_edges` are sorted (canonical order).
/// `traversal_trace` preserves traversal order — it is **not** sorted, because
/// the order is itself part of the evidence and is deterministic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CertBody {
    /// The query in canonical descriptor form (caller-normalized).
    pub query: String,
    /// Minimal evidence nodes, sorted by NodeId.
    pub evidence_nodes: Vec<CanonicalNode>,
    /// Minimal evidence edges, sorted by (from, to).
    pub evidence_edges: Vec<CanonicalEdge>,
    /// Ordered node trace of the traversal.
    pub traversal_trace: Vec<u64>,
    /// Honest verdict: `fact`, `inference`, or `unknown`.
    pub grounding: String,
}

/// A Verifiable Query Certificate.
///
/// Construct it from a graph and a query result, then serialize with
/// [`QueryCertificate::to_canonical_bytes`]. The same `(state, query, result)`
/// always produces byte-identical output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueryCertificate {
    /// BLAKE3 hash of the canonical export of the graph state.
    pub state_hash: [u8; 32],
    /// The query in canonical descriptor form.
    pub query: String,
    /// Minimal evidence nodes, sorted.
    pub evidence_nodes: Vec<CanonicalNode>,
    /// Minimal evidence edges, sorted.
    pub evidence_edges: Vec<CanonicalEdge>,
    /// Ordered traversal trace (node ids).
    pub traversal_trace: Vec<u64>,
    /// Honest verdict: `fact`, `inference`, or `unknown`.
    pub grounding: String,
}

impl QueryCertificate {
    /// Create a certificate from a precomputed state hash and a query result.
    ///
    /// The evidence is derived deterministically from `artifact` and the
    /// graph's canonical form: only nodes/edges the result depends on are
    /// included, in canonical (sorted) order. An empty artifact with
    /// `grounding = "unknown"` yields a proof of absence.
    #[must_use]
    pub fn new(
        state_hash: [u8; 32],
        query: impl Into<String>,
        grounding: impl Into<String>,
        graph: &Graph,
        artifact: &Artifact,
    ) -> Self {
        // Evidence node-id set: traversal path plus any subgraph endpoints.
        let mut id_set: BTreeSet<u64> = artifact.path.iter().map(|n| n.0).collect();
        if let Some(sub) = &artifact.subgraph {
            for (from, to, _w) in sub {
                id_set.insert(from.0);
                id_set.insert(to.0);
            }
        }

        // Reuse the canonical projection: nodes carry their entity id and are
        // already sorted deterministically.
        let canonical = CanonicalGraph::from_graph(graph);

        let evidence_nodes: Vec<CanonicalNode> = canonical
            .nodes
            .iter()
            .filter(|n| id_set.contains(&n.id))
            .cloned()
            .collect();

        let evidence_edges: Vec<CanonicalEdge> = match &artifact.subgraph {
            Some(sub) => {
                let mut edges: Vec<CanonicalEdge> = sub
                    .iter()
                    .map(|(from, to, w)| CanonicalEdge::new(*from, *to, *w))
                    .collect();
                edges.sort();
                edges
            }
            None => canonical
                .edges
                .iter()
                .filter(|e| id_set.contains(&e.from) && id_set.contains(&e.to))
                .cloned()
                .collect(),
        };

        let traversal_trace: Vec<u64> = artifact.path.iter().map(|n: &NodeId| n.0).collect();

        Self {
            state_hash,
            query: query.into(),
            evidence_nodes,
            evidence_edges,
            traversal_trace,
            grounding: grounding.into(),
        }
    }

    /// Serialize to the canonical certificate format.
    ///
    /// Layout: `[cert_len: u32 LE] [CertHeader: postcard] [CertBody: postcard]`.
    /// Deterministic: identical inputs produce identical bytes.
    ///
    /// # Errors
    ///
    /// Returns [`KremisError::SerializationError`] if `postcard` encoding fails.
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, KremisError> {
        let header = CertHeader {
            magic: CERT_MAGIC,
            version: CERT_VERSION,
            state_hash: self.state_hash,
        };
        let body = CertBody {
            query: self.query.clone(),
            evidence_nodes: self.evidence_nodes.clone(),
            evidence_edges: self.evidence_edges.clone(),
            traversal_trace: self.traversal_trace.clone(),
            grounding: self.grounding.clone(),
        };

        let header_bytes = postcard::to_allocvec(&header)
            .map_err(|e| KremisError::SerializationError(format!("Cert header: {}", e)))?;
        let body_bytes = postcard::to_allocvec(&body)
            .map_err(|e| KremisError::SerializationError(format!("Cert body: {}", e)))?;

        let mut out = Vec::with_capacity(4 + header_bytes.len() + body_bytes.len());
        out.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(&header_bytes);
        out.extend_from_slice(&body_bytes);
        Ok(out)
    }

    /// Parse a certificate from its canonical bytes.
    ///
    /// Validates magic and version. Used by independent verifiers and by the
    /// test vectors.
    ///
    /// # Errors
    ///
    /// Returns [`KremisError::SerializationError`] if the data is too short,
    /// the magic/version is unrecognized, or `postcard` decoding fails.
    pub fn from_canonical_bytes(data: &[u8]) -> Result<Self, KremisError> {
        if data.len() < 4 {
            return Err(KremisError::SerializationError(
                "Certificate too short".to_string(),
            ));
        }
        let header_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        if data.len() < 4 + header_len {
            return Err(KremisError::SerializationError(
                "Certificate too short for header".to_string(),
            ));
        }

        let header: CertHeader = postcard::from_bytes(&data[4..4 + header_len])
            .map_err(|e| KremisError::SerializationError(format!("Cert header: {}", e)))?;
        if header.magic != CERT_MAGIC {
            return Err(KremisError::SerializationError(
                "Invalid certificate format".to_string(),
            ));
        }
        if header.version != CERT_VERSION {
            return Err(KremisError::SerializationError(
                "Unsupported certificate version".to_string(),
            ));
        }

        let body: CertBody = postcard::from_bytes(&data[4 + header_len..])
            .map_err(|e| KremisError::SerializationError(format!("Cert body: {}", e)))?;

        Ok(Self {
            state_hash: header.state_hash,
            query: body.query,
            evidence_nodes: body.evidence_nodes,
            evidence_edges: body.evidence_edges,
            traversal_trace: body.traversal_trace,
            grounding: body.grounding,
        })
    }

    /// Whether this certificate proves absence (a certified `unknown`).
    #[must_use]
    pub fn is_proof_of_absence(&self) -> bool {
        self.grounding == "unknown"
            && self.evidence_nodes.is_empty()
            && self.evidence_edges.is_empty()
    }
}

/// Derive the BLAKE3 state hash from a graph's canonical export.
///
/// This is the hash to pass as `state_hash` to [`QueryCertificate::new`].
///
/// # Errors
///
/// Returns [`KremisError::SerializationError`] if the canonical export fails.
///
/// # Requires
///
/// Only available with the `crypto-hash` feature, matching
/// [`crate::export::canonical_crypto_hash`].
#[cfg(feature = "crypto-hash")]
pub fn state_hash(graph: &Graph) -> Result<[u8; 32], KremisError> {
    let data = crate::export::export_canonical(graph)?;
    Ok(*blake3::hash(&data).as_bytes())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::graph::GraphStore;
    use crate::{EdgeWeight, EntityId};
    use proptest::prelude::*;

    fn graph_from(spec: &[(u64, u64, i64)]) -> Graph {
        let mut g = Graph::new();
        let mut ids = std::collections::BTreeMap::new();
        for (from_e, to_e, w) in spec {
            let a = *ids
                .entry(*from_e)
                .or_insert_with(|| g.insert_node(EntityId(*from_e)).expect("insert"));
            let b = *ids
                .entry(*to_e)
                .or_insert_with(|| g.insert_node(EntityId(*to_e)).expect("insert"));
            g.insert_edge(a, b, EdgeWeight::new(*w)).expect("edge");
        }
        g
    }

    #[test]
    fn proof_of_absence_is_detected() {
        let g = graph_from(&[(1, 2, 10)]);
        let cert = QueryCertificate::new([0u8; 32], "lookup:999", "unknown", &g, &Artifact::new());
        assert!(cert.is_proof_of_absence());
        let bytes = cert.to_canonical_bytes().unwrap();
        let back = QueryCertificate::from_canonical_bytes(&bytes).unwrap();
        assert_eq!(cert, back);
        assert!(back.is_proof_of_absence());
    }

    #[test]
    fn bad_magic_is_rejected() {
        let mut bytes =
            QueryCertificate::new([1u8; 32], "q", "fact", &Graph::new(), &Artifact::new())
                .to_canonical_bytes()
                .unwrap();
        // Corrupt the magic (first header byte after the 4-byte length prefix).
        bytes[4] ^= 0xFF;
        assert!(QueryCertificate::from_canonical_bytes(&bytes).is_err());
    }

    proptest! {
        // The core paradigm property: same (state, query, result) => identical
        // bytes, and bytes round-trip through the canonical decoder.
        #[test]
        fn serialization_is_deterministic_and_roundtrips(
            spec in proptest::collection::vec((0u64..6, 0u64..6, -50i64..50), 0..12),
            query in "[a-z:0-9]{0,16}",
            hash_seed in any::<u8>(),
        ) {
            let g = graph_from(&spec);
            let art = Artifact::with_path(g.nodes().map(|n| n.id).collect());
            let h = [hash_seed; 32];

            let c1 = QueryCertificate::new(h, query.clone(), "inference", &g, &art);
            let c2 = QueryCertificate::new(h, query, "inference", &g, &art);

            let b1 = c1.to_canonical_bytes().unwrap();
            let b2 = c2.to_canonical_bytes().unwrap();
            prop_assert_eq!(&b1, &b2, "certificate bytes must be deterministic");

            let decoded = QueryCertificate::from_canonical_bytes(&b1).unwrap();
            prop_assert_eq!(decoded, c1);
        }
    }

    #[cfg(feature = "crypto-hash")]
    #[test]
    fn state_hash_is_deterministic() {
        let g = graph_from(&[(1, 2, 10), (2, 3, 20)]);
        assert_eq!(state_hash(&g).unwrap(), state_hash(&g).unwrap());
    }
}
