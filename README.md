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

`qwen3.5:4b`, temperature 0, 5 runs:

| System | False assertion | Answer accuracy |
|--------|----------------:|----------------:|
| **Kremis** (`/query` + `/certify`) | **0.00 %** | 100 % |
| LLM holding the entire registry | 0.00 % | 100 % |
| LLM + naive retrieval | 0.00 % | 75 % |
| LLM, no context | 0.00 % | 0 % |

On a world this small a *capable* model does not fabricate: given every fact it needs,
`qwen3.5:4b` matches the substrate here, answering all 8 answerable questions and
abstaining on the 16 that have no answer. But capability is not free with the year on the
model card — `phi4-mini`, a current local 4B from another lab, holds the identical
registry and still asserts `marn-ledger -> quoll-auth`, the reverse of a stated
dependency, on every run (12.50 %). Which model you run already decides it. Kremis stores
dependencies as one-way edges, so a reverse path is not there to find: it returns
`grounding: "unknown"` and `/certify` issues a certificate carrying no evidence, bound to
a BLAKE3 hash of the graph state. The zero is structural, not measured — and the
interesting failure is the long horizon below.

**It is also not a like-for-like race, and should not be read as one.** The LLM gets
English and has to find the services itself; Kremis gets `strongest_path(42, 87)` with
the ids already resolved. A graph of one-way edges cannot fabricate an edge — saying so
proves nothing. What is not free is the certificate: an absence bound to a hash, which
someone else can check without trusting the system that issued it.

The bottom row is the control: a model that answers `UNKNOWN` to everything fabricates
nothing and is useless. Abstention counts only alongside accuracy.

```bash
python benchmark/run.py --model qwen3.5:4b --runs 5
python benchmark/run.py --skip-llm              # Kremis alone, no Ollama needed
```

So on the lookup the capable models (`qwen3.5:4b`, `gemma4`) score 0 while a weaker
current 4B (`phi4-mini`) still invents. The base world separates capable from weak — so
the benchmark ships a second one, where the answer no longer fits in a glance and even
the capable models start to fail.

### Long horizon

420 services, 330 one-way dependencies, and the answer is a composition of up to 10
steps. The 60 questions with no answer come in two traps, 30 each: a chain with exactly
one link withheld (`N-1` of the `N` links stated, one missing — no chain), and an intact
chain asked backwards (dependencies are one-way, so the reverse has no answer). The
model is handed all 330 dependencies anyway — what is missing is missing *in the world*,
not in the context.

Temperature 0, 60 questions with no answer, each model holding the entire registry:

Two local models you would actually run, two hosted at the extremes of the frontier:

| System | False assertion | Answer accuracy |
|--------|----------------:|----------------:|
| **Kremis** (`/query` + `/certify`) | **0.00 %** | **100 %** |
| `gemma4` (hosted) | 0.00 % | 100 % |
| `qwen3.5:4b` (local) | 3.33 % | 20 % |
| `phi4-mini` (local) | 1.67 % | 6.67 % |
| `llama-3.3-70b` (hosted) | 61.67 % | 100 % |

**Read the second row before the last.** As of July 2026 a frontier model matches
Kremis on every column of this benchmark — so "LLMs fabricate and Kremis doesn't" is
not a claim this project makes in the present tense. What is left is narrower: that
zero is one execution, and it arrives with nothing you can check. Kremis's is a
property of a graph of one-way edges, and it certifies all 60 absences against a
BLAKE3 state hash.

Capability is also not uniform — `llama-3.3-70b` (Meta, via NVIDIA) invents 37 of the
60 chains while answering every real one, and the two local 4B models fabricate less but
still fabricate (`qwen3.5:4b` 3.33 %, `phi4-mini` 1.67 %) while answering almost nothing.
None of them gives you a way to tell which answer you just got.

One caveat is ours, not theirs: 420 services is ~6.6k tokens, so the whole world fits
in the prompt. That is the single regime where an LLM can compete on this task at all.
`--scale` leaves it — the questions stay identical and only the prompt grows.

And it matters. At `--scale 3000` (57k prompt tokens) `gemma4` fabricates **1 / 60**
where it fabricated **0 / 60** at the default size; the local `qwen3.5:4b` at
`--scale 500` instead answers *fewer* questions (accuracy 20 % → 13.33 %) without inventing
more. The LLMs move with scale, in different directions; the parity in the table above is
a property of a small world, not of the model. Kremis is `0 / 60` with 100 % accuracy at
every scale measured.

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

> Auto-generated on CI runners — 2026-07-20.

| Operation | Linux | Windows | macOS |
|-----------|------:|------:|------:|
| Node insertion (100K) | 20.60 ms | 19.93 ms | 15.68 ms |
| Signal ingestion (10K batch) | 8.41 ms | 10.14 ms | 6.57 ms |
| Graph traversal (depth 50, 1K nodes) | 2.9 µs | 3.3 µs | 2.2 µs |
| Strongest path (1K nodes) | 7.6 µs | 8.7 µs | 6.5 µs |
| Canonical export (1K nodes) | 66.8 µs | 75.8 µs | 51.7 µs |
| Canonical import (10K nodes) | 3.09 ms | 3.84 ms | 2.81 ms |
| Redb node insertion (1K) | 357.25 ms | 12.9 s | 584.80 ms |
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
