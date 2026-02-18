# Kremis

[![CI](https://github.com/M2Dr3g0n/kremis/actions/workflows/ci.yml/badge.svg)](https://github.com/M2Dr3g0n/kremis/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-mintlify-0D9373.svg)](https://kremis.mintlify.app)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
![Status](https://img.shields.io/badge/status-experimental-orange)

> **Work in Progress** — Features incomplete. Breaking changes expected.

**Kremis** is a minimal, deterministic, graph-based cognitive substrate implemented in Rust.

It functions solely as a mechanism to **record**, **associate**, and **retrieve** structural relationships derived from grounded experience.

> **The system does not *understand*.** It contains only the structure of the signals it has processed.

---

## Why Kremis

| Problem | How Kremis addresses it |
|---------|------------------------|
| **Hallucination** | No fabricated data. Every result traces back to real ingested signals. Explicit "not found" for missing data |
| **Opacity** | Fully inspectable state. No hidden layers, no black box. Every result traces back to a graph path |
| **Lack of grounding** | Zero pre-loaded knowledge. All structure emerges from real signals, not assumptions |
| **Non-determinism** | Same input, same output. No randomness, no floating-point arithmetic in core |
| **Data loss** | ACID transactions via `redb` embedded database. Crash-safe by design |

---

## Quick Start

Requires **Rust 1.85+** (stable, edition 2024) and Cargo.

```bash
git clone https://github.com/M2Dr3g0n/kremis.git
cd kremis
cargo build --release
cargo test --workspace
```

```bash
# Initialize database
cargo run -p kremis -- init

# Ingest sample data (9 signals: 3 entities with properties + relationships)
cargo run -p kremis -- ingest -f examples/sample_signals.json -t json

# Start HTTP server (in a separate terminal, or background with &)
cargo run -p kremis -- server

# Check health (in another terminal)
curl http://localhost:8080/health
```

> **Note:** CLI commands and the HTTP server cannot run simultaneously (redb holds an exclusive lock). Stop the server before using CLI commands like `ingest`, `status`, or `export`.

### Try It

With the server running, query the graph:

```bash
# Look up entity 1 (Alice)
curl -X POST http://localhost:8080/query \
     -H "Content-Type: application/json" \
     -d '{"type": "lookup", "entity_id": 1}'

# Traverse from node 0, depth 3
curl -X POST http://localhost:8080/query \
     -H "Content-Type: application/json" \
     -d '{"type": "traverse", "node_id": 0, "depth": 3}'

# Get properties of node 0 (name, role, etc.)
curl -X POST http://localhost:8080/query \
     -H "Content-Type: application/json" \
     -d '{"type": "properties", "node_id": 0}'

# Find common connections between nodes 0 and 1
curl -X POST http://localhost:8080/query \
     -H "Content-Type: application/json" \
     -d '{"type": "intersect", "nodes": [0, 1]}'

# Check graph status
curl http://localhost:8080/status
```

You can also ingest signals via HTTP:

```bash
curl -X POST http://localhost:8080/signal \
     -H "Content-Type: application/json" \
     -d '{"entity_id": 1, "attribute": "name", "value": "Alice"}'
# {"success":true,"node_id":0,"error":null}
```

The `examples/` directory contains sample data in both JSON and text formats.

### Docker

```bash
docker build -t kremis .
docker run -d -p 8080:8080 -v kremis-data:/data kremis
```

Pass configuration via environment variables:

```bash
docker run -d -p 8080:8080 \
  -v kremis-data:/data \
  -e KREMIS_API_KEY=your-secret \
  -e KREMIS_CORS_ORIGINS="https://example.com" \
  kremis
```

Multi-stage build (~136 MB image). Data persists in `/data` volume. Built-in healthcheck on `/health`.

---

## Usage

### CLI

```bash
# Show graph status
cargo run -p kremis -- status

# Show developmental stage
cargo run -p kremis -- stage --detailed

# Ingest signals from file
cargo run -p kremis -- ingest -f data.json -t json

# Query the graph
cargo run -p kremis -- query -t lookup --entity 1
cargo run -p kremis -- query -t traverse -s 0 -d 3
cargo run -p kremis -- query -t path -s 0 -e 5

# Export/Import
cargo run -p kremis -- export -o graph.bin -t canonical
cargo run -p kremis -- import -i graph.bin -B file
```

### HTTP API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/status` | GET | Graph statistics |
| `/stage` | GET | Developmental stage |
| `/signal` | POST | Ingest a signal |
| `/query` | POST | Execute a query |
| `/export` | POST | Export graph |

See the [full documentation](https://kremis.mintlify.app/api/overview) or browse the [source docs](docs/api/) for API reference.

### MCP Server

Kremis provides an MCP (Model Context Protocol) server that enables AI assistants like Claude to interact with the knowledge graph directly.

```bash
# Build the MCP server
cargo build -p kremis-mcp --release

# Run (requires a Kremis HTTP server running)
KREMIS_URL=http://localhost:8080 ./target/release/kremis-mcp
```

Configure in Claude Desktop (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "kremis": {
      "command": "/path/to/kremis-mcp",
      "env": {
        "KREMIS_URL": "http://localhost:8080",
        "KREMIS_API_KEY": "your-key-here"
      }
    }
  }
}
```

7 tools available: `kremis_ingest`, `kremis_lookup`, `kremis_traverse`, `kremis_path`, `kremis_intersect`, `kremis_status`, `kremis_properties`.

### Rust API

```rust
use kremis_core::{Session, Signal, EntityId, Attribute, Value};

let mut session = Session::new();

let signal = Signal::new(
    EntityId(1),
    Attribute::new("name"),
    Value::new("Alice"),
);

let node_id = session.ingest(&signal)?;
```

---

## Architecture

| Component | Description |
|-----------|-------------|
| **kremis-core** | Deterministic graph engine (pure Rust, no async) |
| **apps/kremis** | HTTP server + CLI (tokio, axum, clap) |
| **apps/kremis-mcp** | MCP server bridge for AI assistants (rmcp, stdio) |

See the [architecture docs](https://kremis.mintlify.app/architecture) or browse the [source](docs/architecture.mdx) for internal details (data flow, storage backends, algorithms, export formats).

---

## Testing

```bash
cargo test --workspace
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

---

## License

[Apache License 2.0](LICENSE)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines. The architecture is still evolving — open an [issue](https://github.com/M2Dr3g0n/kremis/issues) before submitting a PR.

## Acknowledgments

This project was developed with AI assistance.

---

<p align="center">
  <strong>Keep it minimal. Keep it deterministic. Keep it grounded. Keep it honest.</strong>
</p>
