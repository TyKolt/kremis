<p align="center">
  <img src="docs/logo/icon.svg" alt="Kremis" width="120" height="120">
</p>

<h1 align="center">Kremis</h1>

<p align="center">
  <strong>A deterministic knowledge graph MCP server. Local, single binary, no LLM in the loop.</strong>
</p>

<p align="center">
  A minimal, graph-based cognitive substrate in Rust.<br>
  Records, associates, retrieves — but never invents.
</p>

<p align="center">
  <a href="https://github.com/TyKolt/kremis/actions/workflows/ci.yml"><img src="https://github.com/TyKolt/kremis/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://crates.io/crates/kremis-core"><img src="https://img.shields.io/crates/v/kremis-core.svg" alt="crates.io"></a>
  <a href="https://kremis.mintlify.app"><img src="https://img.shields.io/badge/docs-mintlify-0D9373.svg" alt="Docs"></a>
  <a href="https://dev.to/tykolt/i-spent-months-trying-to-stop-llm-hallucinations-prompt-engineering-wasnt-enough-so-i-wrote-a-4872"><img src="https://img.shields.io/badge/story-dev.to-0A0A0A.svg" alt="Background & Story"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.89%2B-orange.svg" alt="Rust"></a>
  <img src="https://img.shields.io/badge/status-alpha-orange" alt="Status">
</p>

> **Alpha** — Functional and tested. Breaking changes may still occur before v1.0.

<p align="center">
  <img src="assets/demo.svg" alt="Kremis fabrication benchmark" width="800">
</p>

---

## Why Kremis

| Problem | How Kremis addresses it |
|---------|------------------------|
| **Hallucination** | Every result traces back to a real ingested signal. Missing data returns explicit "not found" — never fabricated |
| **Opacity** | Fully inspectable graph state. No hidden layers, no black box |
| **Lack of grounding** | Zero pre-loaded knowledge. All structure emerges from real signals, not assumptions |
| **Non-determinism** | Same input, same output. No randomness, no floating-point arithmetic in core |
| **Data loss** | ACID transactions via `redb` embedded database. Crash-safe by design |

> [Design Philosophy](https://kremis.mintlify.app/philosophy) — why these constraints exist.

---

## Features

- **Deterministic graph engine** — Pure Rust, no async in core, no floating-point. Same input always produces the same output
- **CLI + HTTP API + MCP bridge** — Three interfaces to the same engine: terminal, REST, and AI assistants
- **BLAKE3 hashing** — Cryptographic hash of the full graph state for integrity verification at any point
- **Canonical export (KREX)** — Deterministic binary snapshot for provenance, audit trails, and reproducibility
- **Proof-carrying knowledge (KVQC)** — `POST /certify` returns a reproducible Verifiable Query Certificate: a portable proof of a fact, or proof of its absence
- **Zero baked-in knowledge** — Kremis starts empty. Every node comes from a real signal
- **ACID persistence** — Default `redb` backend with crash-safe transactions

---

## Use Cases

### AI agent memory via MCP

Give Claude, Cursor, or any MCP-compatible assistant a verifiable memory layer. Kremis stores facts as graph nodes — the agent queries them, and every answer traces back to a real data point. No embeddings, no probabilistic retrieval.

### LLM fact-checking

Ingest your data, let an LLM generate claims, then check each claim against the graph. Every response carries a `grounding` field — `fact`, `inference`, or `unknown` — and `POST /certify` turns an `unknown` into a certificate bound to a BLAKE3 hash of the graph state. No confidence scores, no ambiguity.

### Provenance and audit trail

Export the full graph as a deterministic binary snapshot, compute its BLAKE3 hash, and verify integrity at any point. Every node links to the signal that created it. Useful for compliance workflows where you need to prove what data was present and when.

---

## Fabrication Benchmark

A closed registry of 9 fictional services and 5 one-way dependencies. 24 questions of
the form *"does A depend on B, directly or transitively?"* — 8 have an answer, 16 do
not, and no answer exists for them anywhere. Nothing in the prompt asks any model to
invent: the facts are supplied and `UNKNOWN` is offered.

`qwen2.5:3b`, temperature 0, 3 runs:

| System | False assertion | Answer accuracy |
|--------|----------------:|----------------:|
| **Kremis** (`/query` + `/certify`) | **0.00 %** | 100 % |
| LLM holding the entire registry | 12.50 % | 50 % |
| LLM + naive retrieval | 6.25 % | 75 % |
| LLM, no context | 0.00 % | 0 % |

Given every fact it needs, the model still asserts `marn-ledger -> quoll-auth` — the
reverse of a stated dependency — on every run. Kremis stores dependencies as one-way
edges, so the reverse path is not there to find: it returns `grounding: "unknown"` and
`/certify` issues a certificate carrying no evidence, bound to a BLAKE3 hash of the
graph state. The zero is structural, not measured.

**It is also not a like-for-like race, and should not be read as one.** The LLM gets
English and has to find the services itself; Kremis gets `strongest_path(42, 87)` with
the ids already resolved. A graph of one-way edges cannot fabricate an edge — saying so
proves nothing. What is not free is the certificate: an absence bound to a hash, which
someone else can check without trusting the system that issued it.

The bottom row is the control: a model that answers `UNKNOWN` to everything fabricates
nothing and is useless. Abstention counts only alongside accuracy.

```bash
python benchmark/run.py --model qwen2.5:3b --runs 3
python benchmark/run.py --skip-llm              # Kremis alone, no Ollama needed
```

**A larger model resists this partition.** `qwen3-coder-next` (80B), holding the same
registry, scored 0 % across 3 runs. A world you can hold in your head is a world a big
model can hold in its head — so the benchmark ships a second one.

### Long horizon

420 services, 330 one-way dependencies, and the answer is a composition of up to 10
steps. Half the chains have exactly one link withheld: `N-1` of the `N` links are in
the registry, one is not, and there is no chain. The model is handed all 330
dependencies anyway — the link is missing from the *world*, not from the context.

`qwen3-coder-next` (80B), temperature 0, 60 questions with no answer:

| System | False assertion | Answer accuracy |
|--------|----------------:|----------------:|
| **Kremis** (`/query` + `/certify`) | **0.00 %** | **100 %** |
| LLM holding the entire registry | 21.67 % | 83 % |
| LLM + naive retrieval | 0.00 % | 20 % |
| LLM, no context | 31.67 % | 0 % |

At `N = 10` the model asserts 4 of the 6 chains that do not exist, and recovers 2 of
the 6 that do — more fabricated chains than correct ones. Kremis is `0/6` and `6/6` at
every horizon, and certifies all 60 absences against a BLAKE3 state hash.

A second capable model on a different vendor collapses the same way, harder:
`llama-3.3-70b` (Meta, via NVIDIA) fabricates **37 / 60 (61.67 %)** while answering
every real chain correctly (100 %). Two families, two providers, one result.

```bash
python benchmark/run.py --world horizon
```

Caveats, the counter-experiment, the noise in the curve, and the ground truth are in
[`benchmark/README.md`](benchmark/README.md).

---

## Quick Start

Requires **Rust 1.89+** and Cargo.

```bash
git clone https://github.com/TyKolt/kremis.git
cd kremis
cargo build --release
cargo test --workspace
```

```bash
cargo run -p kremis -- init                                          # initialize database
cargo run -p kremis -- ingest -f examples/sample_signals.json -t json # ingest sample data
cargo run -p kremis -- server                                        # start HTTP server
```

In a second terminal:

```bash
curl http://localhost:8080/health
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"type":"lookup","entity_id":1}'
```

> **Note:** CLI commands and the HTTP server cannot run simultaneously (`redb` holds an exclusive lock). Stop the server before using CLI commands.

### Docker

```bash
docker build -t kremis .

# MCP server (default) — pipe MCP stdio JSON-RPC; suitable for any MCP client
docker run -i --rm kremis

# HTTP API only — override the entrypoint
docker run -d -p 8080:8080 -v kremis-data:/data \
  --entrypoint kremis kremis server -H 0.0.0.0 -D /data/kremis.db
```

---

## Architecture

| Component | Description |
|-----------|-------------|
| **kremis-core** | Deterministic graph engine (pure Rust, no async) |
| **apps/kremis** | HTTP server + CLI (tokio, axum, clap) |
| **apps/kremis-mcp** | MCP server bridge for AI assistants (rmcp, stdio) |

See the [architecture docs](https://kremis.mintlify.app/architecture) for internals: data flow, storage backends, algorithms, export formats.

---

## Documentation

Full reference at **[kremis.mintlify.app](https://kremis.mintlify.app)**:

| Topic | Link |
|-------|------|
| Introduction | [kremis.mintlify.app/introduction](https://kremis.mintlify.app/introduction) |
| Installation | [kremis.mintlify.app/installation](https://kremis.mintlify.app/installation) |
| Quick Start | [kremis.mintlify.app/quickstart](https://kremis.mintlify.app/quickstart) |
| Configuration | [kremis.mintlify.app/configuration](https://kremis.mintlify.app/configuration) |
| CLI Reference | [kremis.mintlify.app/cli/overview](https://kremis.mintlify.app/cli/overview) |
| API Reference | [kremis.mintlify.app/api/overview](https://kremis.mintlify.app/api/overview) |
| MCP Server | [kremis.mintlify.app/mcp/overview](https://kremis.mintlify.app/mcp/overview) |
| Philosophy | [kremis.mintlify.app/philosophy](https://kremis.mintlify.app/philosophy) |
| The Name | [kremis.mintlify.app/the-name](https://kremis.mintlify.app/the-name) |

---

## Testing

```bash
cargo test --workspace
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

---

<!-- BENCHMARK-START -->
## Benchmarks

> Auto-generated on CI runners — 2026-07-13.

| Operation | Linux | Windows | macOS |
|-----------|------:|------:|------:|
| Node insertion (100K) | 20.61 ms | 22.85 ms | 17.72 ms |
| Signal ingestion (10K batch) | 8.09 ms | 14.04 ms | 7.18 ms |
| Graph traversal (depth 50, 1K nodes) | 2.8 µs | 3.4 µs | 1.9 µs |
| Strongest path (1K nodes) | 7.4 µs | 8.2 µs | 5.8 µs |
| Canonical export (1K nodes) | 67.1 µs | 77.1 µs | 50.9 µs |
| Canonical import (10K nodes) | 3.09 ms | 3.64 ms | 3.24 ms |
| Redb node insertion (1K) | 359.12 ms | 12.8 s | 339.53 ms |
<!-- BENCHMARK-END -->

---

## License

[Apache License 2.0](LICENSE)

The brand assets in [`docs/logo/`](docs/logo/) (logo, icon, favicon) are proprietary and not covered by the Apache 2.0 license. See [`docs/logo/LICENSE`](docs/logo/LICENSE).

## Contributing

See [CONTRIBUTING.md](.github/CONTRIBUTING.md) for guidelines. The architecture is still evolving — open an [issue](https://github.com/TyKolt/kremis/issues) before submitting a PR.

## Acknowledgments

This project was developed with AI assistance.

---

<p align="center">
  <strong>Keep it minimal. Keep it deterministic. Keep it grounded. Keep it honest.</strong>
</p>
