# Kremis

[![CI](https://github.com/M2Dr3g0n/kremis/actions/workflows/ci.yml/badge.svg)](https://github.com/M2Dr3g0n/kremis/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
![Status](https://img.shields.io/badge/status-experimental-orange)
> 🚧 **Work in Progress**  
> Features incomplete. Breaking changes expected.

**Kremis** is a minimal, deterministic, graph-based cognitive substrate implemented in Rust.

It functions solely as a mechanism to **record**, **associate**, and **retrieve** structural relationships derived from grounded experience.

> **The system does not *understand*.** It contains only the structure of the signals it has processed.

---

## Why Kremis

Current AI systems suffer from three fundamental problems:

| Problem | How Kremis addresses it |
|---------|------------------------|
| **Hallucination** | Output is always honest: Facts, Inferences, or explicit "I don't know". No silent gap-filling. |
| **Opacity** | Fully inspectable state. No hidden layers, no black box. Every result traces back to a graph path. |
| **Lack of grounding** | Zero pre-loaded knowledge. All structure emerges from real signals, not assumptions. |

---

## Features

| Feature | Description |
|---------|-------------|
| **Deterministic** | Same input, same output. No randomness, no floats in Core |
| **Grounded** | Zero pre-loaded knowledge. All structure emerges from signals |
| **Transparent** | Fully inspectable state machine. No hidden state |
| **Crash-Safe** | ACID transactions via `redb` embedded database |
| **Honest Output** | Facts, Inferences, or "I don't know" - never hallucination |

---

## Requirements

- Rust 1.75+ (stable)
- Cargo

---

## Quick Start

```bash
git clone https://github.com/M2Dr3g0n/kremis.git
cd kremis
cargo build --release
cargo test --workspace
```

### Initialize and Run

```bash
# Initialize database
cargo run -p kremis -- init

# Start HTTP server
cargo run -p kremis -- server

# Check status
curl http://localhost:8080/health
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    KREMIS BINARY (apps/kremis)              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                    HTTP SERVER                       │    │
│  │   POST /signal   POST /query   GET /status          │    │
│  └─────────────────────────────────────────────────────┘    │
│                            │                                 │
│                            ▼                                 │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              KREMIS-CORE (THE LOGIC)                 │    │
│  │  INGESTOR ───▶ GRAPH ENGINE ───▶ COMPOSITOR         │    │
│  │    (RX)          (STORE)            (TX)            │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

| Component | Description |
|-----------|-------------|
| **kremis-core** | THE LOGIC - Deterministic graph engine (pure Rust, no async) |
| **apps/kremis** | THE BINARY - HTTP server + CLI (tokio, axum, clap) |

---

## Project Structure

```
kremis/
├── Cargo.toml              # Workspace Root
├── crates/
│   └── kremis-core/         # THE LOGIC - Graph Engine
│       └── src/
│           ├── types/       # Core types (EntityId, Signal, etc.)
│           ├── graph.rs     # BTreeMap-based graph
│           ├── formats/     # Persistence formats
│           ├── system/      # Stage assessment (S0-S3)
│           └── storage/     # redb backend
├── apps/
│   └── kremis/              # THE BINARY
│       └── src/
│           ├── main.rs
│           ├── api/         # HTTP handlers
│           └── cli/         # CLI commands
└── docs/                    # Documentation
```

---

## Usage

### CLI

```bash
# Initialize database
cargo run -p kremis -- init

# Start HTTP server
cargo run -p kremis -- server --port 8080

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

See [docs/API.md](docs/API.md) for full API documentation with examples.

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

## Testing

```bash
cargo test --workspace
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

---

## License

[Apache License 2.0](LICENSE)

## Acknowledgments

This project was developed with the assistance of AI (Claude by Anthropic).

---

<p align="center">
  <strong>Keep it minimal. Keep it deterministic. Keep it grounded. Keep it honest.</strong>
</p>
