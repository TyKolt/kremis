//! Frozen certificate vectors.
//!
//! These cases exercise the published certificate contract
//! (`docs/concepts/certificate-spec.mdx`) against the public API, so the
//! reproducibility and proof-of-absence claims are checked, not assumed.
//!
//! `state_hash` is supplied as a fixed input here (feature-independent): the
//! vectors verify the certificate format and the absence path, not the BLAKE3
//! derivation, which is covered by the `crypto-hash` unit test.

// Top-level fixture helpers use `.expect()`; this is test code, not production.
#![allow(clippy::expect_used)]

use kremis_core::graph::GraphStore;
use kremis_core::{Artifact, EdgeWeight, EntityId, Graph, QueryCertificate};

/// Deterministic fixture graph: 1 -> 2 -> 3.
fn fixture_graph() -> Graph {
    let mut g = Graph::new();
    let a = g.insert_node(EntityId(1)).expect("node 1");
    let b = g.insert_node(EntityId(2)).expect("node 2");
    let c = g.insert_node(EntityId(3)).expect("node 3");
    g.insert_edge(a, b, EdgeWeight::new(10)).expect("edge a-b");
    g.insert_edge(b, c, EdgeWeight::new(20)).expect("edge b-c");
    g
}

const FIXED_HASH: [u8; 32] = [7u8; 32];

#[test]
fn vector_fact_is_reproducible() {
    let g = fixture_graph();
    let artifact = Artifact::with_subgraph(vec![], g.edges().collect());

    let c1 = QueryCertificate::new(FIXED_HASH, "traverse:1:2", "fact", &g, &artifact);
    let c2 = QueryCertificate::new(FIXED_HASH, "traverse:1:2", "fact", &g, &artifact);

    let b1 = c1.to_canonical_bytes().expect("encode 1");
    let b2 = c2.to_canonical_bytes().expect("encode 2");
    assert_eq!(b1, b2, "fact certificate must be byte-reproducible");

    let decoded = QueryCertificate::from_canonical_bytes(&b1).expect("decode");
    assert_eq!(decoded, c1);
    assert!(!decoded.is_proof_of_absence());
    assert_eq!(decoded.evidence_edges.len(), 2);
}

/// Frozen bytes: these are the exact encodings the published format produces.
///
/// Reproducibility alone (two runs agreeing) cannot catch a change that alters
/// the bytes *consistently*. These literals pin the wire format itself, so any
/// refactor of the evidence projection has to reproduce it byte for byte.
#[test]
fn vector_bytes_are_frozen() {
    let g = fixture_graph();
    let ids: Vec<_> = g.nodes().map(|n| n.id).collect();

    // subgraph = Some (traverse): evidence edges come from the artifact.
    let traverse = Artifact::with_subgraph(ids.clone(), g.edges().collect());
    let bytes = QueryCertificate::new(FIXED_HASH, "traverse:1:2", "fact", &g, &traverse)
        .to_canonical_bytes()
        .expect("encode traverse");
    assert_eq!(
        hex(&bytes),
        "250000004b5651430107070707070707070707070707070707070707070707070707070707070707070c74726176657273653a313a320300010102020302000114010228030001020466616374"
    );

    // subgraph = None (path only): evidence edges are induced by the path.
    let path = Artifact::with_path(vec![ids[0], ids[2]]);
    let bytes = QueryCertificate::new(FIXED_HASH, "path:1:3", "inference", &g, &path)
        .to_canonical_bytes()
        .expect("encode path");
    assert_eq!(
        hex(&bytes),
        "250000004b56514301070707070707070707070707070707070707070707070707070707070707070708706174683a313a3302000102030002000209696e666572656e6365"
    );
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn vector_proof_of_absence_is_reproducible() {
    let g = fixture_graph();

    // A query with no result: empty artifact + grounding "unknown".
    let c1 = QueryCertificate::new(FIXED_HASH, "lookup:999", "unknown", &g, &Artifact::new());
    let c2 = QueryCertificate::new(FIXED_HASH, "lookup:999", "unknown", &g, &Artifact::new());

    let b1 = c1.to_canonical_bytes().expect("encode 1");
    let b2 = c2.to_canonical_bytes().expect("encode 2");
    assert_eq!(b1, b2, "absence certificate must be byte-reproducible");

    let decoded = QueryCertificate::from_canonical_bytes(&b1).expect("decode");
    assert_eq!(decoded, c1);
    assert!(
        decoded.is_proof_of_absence(),
        "an empty 'unknown' result must certify absence"
    );
}
