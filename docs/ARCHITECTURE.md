# Kremis-Core Architecture

> Internal architecture of the graph engine. For HTTP API see [API.md](API.md), for CLI see [CLI.md](CLI.md).

---

## Data Flow

```
Signal (Entity | Attribute | Value)
        │
        ▼
   ┌─────────┐    validate attribute ≤256B, value ≤64KB
   │ Ingestor │    get_or_create node for entity
   └────┬────┘    store_property(node, attribute, value)
        │         link adjacent signals (ASSOCIATION_WINDOW = 1)
        ▼
   ┌─────────┐
   │ Session  │──── match backend ────┐
   └─────────┘                        │
        │                             │
   ┌────┴─────┐               ┌──────┴──────┐
   │ InMemory │               │ Persistent  │
   │  (Graph) │               │ (RedbGraph) │
   └──────────┘               └─────────────┘
        │                             │
        ▼                             ▼
   ┌──────────┐               ┌──────────────┐
   │Compositor│               │  Compositor  │
   │ (query)  │               │   (query)    │
   └────┬─────┘               └──────┬───────┘
        │                             │
        ▼                             ▼
     Artifact { path, subgraph }
```

---

## Session and Storage Backends

`Session` wraps a `StorageBackend` enum and a volatile `Buffer`:

```rust
pub struct Session {
    backend: StorageBackend,   // graph data (persistent or in-memory)
    buffer: Buffer,            // active context (always volatile, never saved)
}

pub enum StorageBackend {
    InMemory(Graph),
    Persistent(RedbGraph),
}
```

Every operation delegates via `match`:

```rust
match &mut self.backend {
    StorageBackend::InMemory(graph) => Ingestor::ingest_signal(graph, signal),
    StorageBackend::Persistent(redb) => Ingestor::ingest_signal(redb, signal),
}
```

Both backends implement the `GraphStore` trait (16 methods), ensuring identical behavior.

---

## In-Memory Graph

All fields use `BTreeMap` for deterministic iteration (no `HashMap`):

```rust
pub struct Graph {
    nodes:        BTreeMap<NodeId, Node>,
    edges:        BTreeMap<NodeId, BTreeMap<NodeId, EdgeWeight>>,  // adjacency list
    entity_index: BTreeMap<EntityId, NodeId>,                      // reverse lookup
    properties:   BTreeMap<NodeId, BTreeMap<Attribute, Vec<Value>>>,
    next_node_id: u64,
}
```

---

## Persistent Graph (RedbGraph)

Uses [redb](https://github.com/cberner/redb) embedded database with 5 tables:

| Table | Key | Value | Purpose |
|-------|-----|-------|---------|
| `NODES` | `u64` | `&[u8]` (postcard) | NodeId to serialized Node |
| `EDGES` | `(u64, u64)` | `i64` | (from, to) to weight |
| `ENTITY_INDEX` | `u64` | `u64` | EntityId to NodeId |
| `METADATA` | `&str` | `u64` | Counters (e.g. `"next_node_id"`) |
| `PROPERTIES` | `(u64, u64)` | `&[u8]` (postcard) | (node_id, attr_hash) to (Attribute, Vec\<Value\>) |

Properties: ACID transactions, crash-safe (copy-on-write B-trees), MVCC concurrent readers.

In-memory cache: `entity_cache: BTreeMap<EntityId, NodeId>` loaded on open for fast entity lookups.

---

## Signal Ingestion

```
ingest_signal(graph, signal):
  1. validate(signal)            → KremisError::InvalidSignal if bad
  2. node_id = insert_node(entity) or get existing
  3. store_property(node_id, attribute, value)
  4. return node_id

ingest_sequence(graph, [A, B, C]):
  Uses signals.windows(ASSOCIATION_WINDOW + 1) = windows(2)
  → ingest A, ingest B, edge A→B (weight +1)
  → ingest B, ingest C, edge B→C (weight +1)
  Repeated signals on same edge: weight increments (saturating)
```

---

## Traversal Algorithms

| Method | Algorithm | Details |
|--------|-----------|---------|
| `compose` | BFS | `VecDeque` queue, bounded by `depth` (max 100) |
| `compose_filtered` | BFS + weight filter | Skips edges below `min_weight` |
| `strongest_path` | Dijkstra | Cost = `i64::MAX - weight` (higher weight = preferred), `BTreeMap` for determinism |
| `intersect` | Set intersection | Neighbors of first node as `BTreeSet`, intersect with remaining |
| `related_context` | BFS | Same as `compose` (contextual alias) |

All traversals return `Artifact { path: Vec<NodeId>, subgraph: Option<Vec<(NodeId, NodeId, EdgeWeight)>> }`.

---

## Export Formats

### Canonical (bit-exact, for verification)

```
Byte layout:
[header_len: u32 LE] [CanonicalHeader: postcard] [CanonicalGraph: postcard]

Header: magic=b"KREX", version=2, node_count, edge_count, checksum
Data:   nodes (sorted), edges (sorted), next_node_id, properties (sorted)
```

- Checksum: XOR-based deterministic hash (not cryptographic)
- V1 backward compat: imports without properties field
- Import limits: 1M nodes, 10M edges (DoS protection)

### Persistence (binary, for disk storage)

```
Byte layout:
[magic: b"KREM" 4B] [version: 1B] [SerializableGraph: postcard]
```

- Payload limit: 500 MB
- `SerializableGraph`: JSON-compatible via serde (nodes, edges, next_node_id, properties)

---

## Stage System

Developmental stages based on **stable edge count** (edges with weight >= 10):

| Stage | Name | Threshold |
|-------|------|-----------|
| S0 | Signal Segmentation | 0 |
| S1 | Pattern Crystallization | 100 stable edges |
| S2 | Causal Chaining | 1,000 stable edges |
| S3 | Recursive Optimization | 5,000 stable edges |

Stages are **informational only** — they do not gate functionality. `StageCapability` is a reference pattern documenting which capabilities conceptually belong to each stage.

---

## MCP Server (kremis-mcp)

External process that translates MCP (Model Context Protocol) into HTTP calls to the Kremis REST API. Does not embed `kremis-core`.

```
Claude/GPT <--MCP (stdio)--> kremis-mcp <--HTTP--> kremis server
```

| Module | Responsibility |
|--------|---------------|
| `main.rs` | Entry point: env vars, tracing to stderr, stdio transport |
| `server.rs` | `KremisMcp` + `ServerHandler` + 7 MCP tools via `rmcp` |
| `client.rs` | `KremisClient`: HTTP wrapper (`reqwest`) to Kremis API |

### MCP Tools

| Tool | Kremis API Call | Description |
|------|----------------|-------------|
| `kremis_ingest` | `POST /signal` | Add entity or relation |
| `kremis_lookup` | `POST /query` (lookup) | Look up entity by ID |
| `kremis_traverse` | `POST /query` (traverse) | Traverse from node |
| `kremis_path` | `POST /query` (strongest_path) | Find strongest path |
| `kremis_intersect` | `POST /query` (intersect) | Find common connections |
| `kremis_status` | `GET /status` | Graph statistics |
| `kremis_properties` | `POST /query` (properties) | Node properties |

### Configuration

| Env Variable | Default | Description |
|-------------|---------|-------------|
| `KREMIS_URL` | `http://localhost:8080` | Kremis server URL |
| `KREMIS_API_KEY` | (none) | Optional Bearer token |

---

## Rust API Reference

Run `cargo doc --no-deps --package kremis-core --open` for the complete, compiler-generated API reference.
