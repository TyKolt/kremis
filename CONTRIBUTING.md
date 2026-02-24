# Contributing to Kremis

Thank you for your interest in Kremis. This document explains how to contribute effectively.

> **Status:** Kremis is experimental. The architecture is still evolving. Contributions are welcome, but expect breaking changes.

---

## Prerequisites

- **Rust 1.85+** (stable, edition 2024)
- **Git** with signed commits (GPG recommended)
- Familiarity with graph data structures and Rust's type system

---

## Project Structure

```
kremis/
├── crates/kremis-core/   # Graph engine (pure Rust, no async, no network)
├── apps/kremis/           # HTTP server + CLI (axum, clap)
├── apps/kremis-mcp/       # MCP server bridge (rmcp, stdio)
└── docs/                  # Public documentation (Mintlify MDX format)
```

---

## The 4 Fundamental Laws

Every contribution must respect these laws. PRs that violate them will be rejected.

1. **Determinism** — Same input produces same output. No randomness, no `HashMap`/`HashSet` in core, no floating-point arithmetic in core, no timestamp-dependent logic in core.
2. **Precision** — Output is honest: Facts, Inferences, or "I don't know". No silent gap-filling.
3. **Security** — Constant-time auth comparison, input validation, path traversal protection, DoS limits.
4. **Separation** — `kremis-core` is pure (no async, no network, no IO). `apps/` handles all IO.

---

## Development Setup

```bash
git clone https://github.com/TyKolt/kremis.git
cd kremis
cargo build --workspace
cargo test --workspace
```

---

## Code Style

### Enforced by CI

- `cargo fmt --all -- --check` — Standard Rust formatting.
- `cargo clippy --all-targets --all-features -- -D warnings` — All warnings are errors.

### Workspace Lints

These are **denied** across the entire workspace:

| Lint | Reason |
|------|--------|
| `clippy::float_arithmetic` | Determinism: no floating-point in core |
| `clippy::unwrap_used` | Safety: handle all errors explicitly |
| `clippy::panic` | Safety: no panics in library code |

### Conventions

- Use `BTreeMap`/`BTreeSet` instead of `HashMap`/`HashSet` in `kremis-core`.
- All `QueryResponse` constructors must include `properties: vec![]`, `grounding: "unknown".to_string()`, and `diagnostic: None`. Use `.with_diagnostic("reason")` on `not_found()` responses to populate the Diagnostic Side-Channel.
- No `unsafe` in production code. Test-only `unsafe` (e.g., `std::env::set_var`) must include a `// SAFETY:` comment.
- No new dependencies without prior discussion in an issue.

---

## Testing

Run the full test suite before submitting a PR:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace
cargo test --doc --workspace
```

CI runs these checks on **3 operating systems** (Linux, Windows, macOS). All must pass.

### Writing Tests

- Place unit tests in the same file as the code they test (`#[cfg(test)]` module).
- Place integration tests in `tests/` directories within each crate.
- Use `proptest` for property-based testing where applicable.
- Use `tempfile` for tests that need filesystem access.

---

## Commit Conventions

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add batch signal ingestion
fix: correct edge weight overflow on increment
docs: update API reference with new query type
chore: update dependencies
test: add proptest for canonical roundtrip
```

### Version Bump Rule

This is **mandatory** — maintainers will reject commits that violate this:

- Every `feat:` commit **must** include a **MINOR** version bump in the same commit.
- Every `fix:` commit **must** include a **PATCH** version bump in the same commit.

Committed files that contain the version (all must be updated together):

| File | What to update |
|------|---------------|
| `Cargo.toml` | `[workspace.package] version` |
| `docs/api/overview.mdx` | Version in description |
| `apps/kremis/tests/types_tests.rs` | Version assertion |

---

## Pull Request Process

1. **Open an issue first.** Describe what you want to change and why. Wait for feedback before writing code.
2. **Fork and branch.** Create a branch from `main` with a descriptive name (`feat/batch-ingest`, `fix/edge-overflow`).
3. **Keep it focused.** One logical change per PR. Do not mix features, fixes, and refactoring.
4. **Pass all checks.** Run the full test suite locally before pushing.
5. **Write a clear description.** Explain what changed, why, and how to test it.
6. **Be patient.** This is a solo-maintained project. Reviews may take time.

### Merge Authority

**Only the maintainer can merge pull requests into `main`.**

- Contributors cannot merge their own PRs, regardless of CI status.
- No direct commits to `main` from external contributors — all changes go through a PR.
- The maintainer reviews every PR for compliance with the 4 Fundamental Laws before merging.
- A passing CI is necessary but not sufficient for merge approval.

### PR Checklist

- [ ] All tests pass (`cargo test --workspace`)
- [ ] No clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`)
- [ ] Code is formatted (`cargo fmt --all -- --check`)
- [ ] Version bumped if `feat:` or `fix:` (see table above)
- [ ] Documentation updated if behavior changed
- [ ] Commit messages follow conventional commits

---

## What NOT to Contribute

To keep the project focused, the following will not be accepted:

- **AI/ML algorithms in kremis-core.** The core is a pure graph engine.
- **Non-deterministic constructs in core.** No `HashMap`, `HashSet`, `rand`, `uuid`, floats.
- **New dependencies without discussion.** Open an issue first.
- **Speculative features.** Every change must solve a concrete, current problem.
- **Cosmetic refactoring.** Do not restructure working code for style preferences.
- **CHANGELOG, SECURITY files.** Not needed at this stage.

---

## Reporting Bugs

Open an [issue](https://github.com/TyKolt/kremis/issues) with:

- Kremis version (`cargo run -p kremis -- --version`)
- OS and Rust version (`rustc --version`)
- Steps to reproduce
- Expected vs actual behavior

---

## License

By contributing, you agree that your contributions will be licensed under the [Apache License 2.0](LICENSE), consistent with Section 5 of the Apache License.

---

<p align="center">
  <strong>Keep it minimal. Keep it deterministic. Keep it grounded. Keep it honest.</strong>
</p>
