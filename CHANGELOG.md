## [0.14.2] - 2026-03-07

### 🐛 Bug Fixes

- Bound Vec::with_capacity to MAX_SEQUENCE_LENGTH (CWE-770) — v0.14.2

### ⚙️ Miscellaneous Tasks

- Remove auto-tag job — release process is now manual
## [0.14.1] - 2026-03-06

### 🐛 Bug Fixes

- Guard POST /signals against uncontrolled allocation (CWE-770) — v0.14.1

### 📚 Documentation

- Regenerate CHANGELOG for v0.14.1 [skip ci]

### ⚙️ Miscellaneous Tasks

- Use GH_PAT in auto-tag checkout to trigger release workflow
## [0.14.0] - 2026-03-06

### 🚀 Features

- Add POST /signals batch ingest endpoint — creates edges via HTTP (v0.14.0)

### 📚 Documentation

- Update CHANGELOG — close [Unreleased] → [0.13.1]
- Regenerate CHANGELOG for v0.14.0 [skip ci]

### ⚙️ Miscellaneous Tasks

- Add auto-tag job — GPG-signed CHANGELOG + tag after ci-success
- Add Ko-fi sponsor button via FUNDING.yml
## [0.13.1] - 2026-03-04

### 🐛 Bug Fixes

- Redirect tracing logs to stderr, suppress banner in --json-mode (v0.13.1)

### 📚 Documentation

- Regenerate CHANGELOG for v0.13.0
- Add missing env vars to installation and mcp/setup pages
- Regenerate CHANGELOG for v0.13.1, fix hash.mdx backend docs
## [0.13.0] - 2026-03-03

### 🚀 Features

- Add AppConfig TOML-based configuration system, v0.12.0
- Add config provenance log (ConfigReport), v0.13.0

### 💼 Other

- *(deps)* Bump tempfile in the rust-minor-patch group (#6)

### 📚 Documentation

- Regenerate CHANGELOG for v0.11.0, fix stale version in openapi.yml example
- Regenerate CHANGELOG for v0.12.0

### ⚙️ Miscellaneous Tasks

- Rename kremis.toml → kremis.example.toml, gitignore local config
- Update tempfile 3.25.0 → 3.26.0, remove stale wasm transitive deps
## [0.11.0] - 2026-03-01

### 🚀 Features

- Add top_k to TraverseFiltered, v0.11.0

### 📚 Documentation

- Regenerate CHANGELOG for v0.10.0
## [0.10.0] - 2026-02-28

### 🚀 Features

- Structured JSON logging (KREMIS_LOG_FORMAT=json), v0.10.0

### 📚 Documentation

- Update README MCP tool count 7 → 9 (kremis_retract, kremis_hash)
- Fix stale version examples in health.mdx and openapi.yml
## [0.9.0] - 2026-02-27

### 🚀 Features

- MCP parity — kremis_retract + kremis_hash tools, v0.9.0

### 📚 Documentation

- Regenerate CHANGELOG for v0.9.0
## [0.8.0] - 2026-02-26

### 🚀 Features

- Add decrement_edge + POST /signal/retract for edge invalidation

### 📚 Documentation

- Add CHANGELOG.md with git-cliff and update release workflow
- Regenerate CHANGELOG for v0.8.0
## [0.7.0] - 2026-02-26

### 🚀 Features

- Add GET /hash (BLAKE3), GET /metrics (Prometheus), bench-compile CI

### ⚙️ Miscellaneous Tasks

- *(ci)* Pin @redocly/cli@2 and cache npm in OpenAPI Lint job
## [0.6.1] - 2026-02-25

### 🐛 Bug Fixes

- Replace DefaultHasher with stable FNV-1a for PROPERTIES table key
## [0.6.0] - 2026-02-24

### 🚀 Features

- Add batch ingest to RedbGraph for O(1) transaction overhead

### 📚 Documentation

- Add honesty demo script and README section
- Add merge authority section and sanitize config files

### ⚙️ Miscellaneous Tasks

- Add doc-test step and --all-features flag to clippy in release workflow
- Add Claude Code project settings
## [0.5.0] - 2026-02-21

### 🚀 Features

- Add diagnostic field to QueryResponse (Honesty Protocol)

### 📚 Documentation

- Add OpenAPI 3.1.0 spec and CI lint job
- Update QueryResponse convention in CONTRIBUTING.md

### ⚙️ Miscellaneous Tasks

- Update Cargo.lock to version 0.4.0
## [0.4.0] - 2026-02-19

### 🚀 Features

- Add grounding field to QueryResponse (Honesty Protocol)

### 🐛 Bug Fixes

- Init --force now properly resets database, improve README quickstart flow

### 💼 Other

- Update clap 4.5.59, tempfile 3.25.0

### 📚 Documentation

- Add sample data and Try It quickstart section
- Add CONTRIBUTING.md and fix README inaccuracies
- Remove private dev-docs references from CONTRIBUTING.md
- Migrate documentation to Mintlify MDX format
- Add Mintlify docs badge to README
- Add grounding field to all query response examples

### ⚙️ Miscellaneous Tasks

- Align v0.3.0 references and centralize workspace deps
- Remove deleted labels from dependabot config
- Remove private doc references from codebase, update docs config
- Migrate alias from M2Dr3g0n to TyKolt
- Add brand identity — logo, favicon, OG image
- Replace NOTICE with LICENSE for brand assets, update README
## [0.3.0] - 2026-02-14

### 🚀 Features

- Add Dockerfile for containerized deployment
- Add kremis-mcp — MCP server bridge for AI assistants (beta)

### 💼 Other

- Expand benchmark suite with 8 new groups and larger scales

### 📚 Documentation

- Add Docker usage section to README
- Add MCP server section to API.md and CLI.md
- Add MCP Server to API.md table of contents

### 🎨 Styling

- Apply rustfmt to bench_increment_edge chain

### ⚙️ Miscellaneous Tasks

- Bump version to 0.2.1
- Bump version to 0.3.0
- Add social/, video/, nul to .gitignore
## [0.2.0] - 2026-02-11

### 🚀 Features

- Kremis v0.1.0 - deterministic graph cognitive substrate
- Persist signal attributes and values as node properties

### 🐛 Bug Fixes

- Preserve node properties across export/import and expose via API/CLI
- Add CLI related query and prevent dangling edges in RedbGraph
- Align docs with code, remove dead code, harden rate limiter
- Add checks:write permission to audit job in release workflow

### 💼 Other

- Bump bytes 1.11.0 -> 1.11.1 (security fix)
- Update dependencies

### 🚜 Refactor

- Unify GraphStore trait to eliminate Graph/RedbGraph duplication
- Remove related_subgraph duplicate of traverse

### 📚 Documentation

- Fix _archive reference and update dates
- Fix incorrect example outputs in API.md
- Add AI acknowledgment to README, fix docs accuracy
- Update project documentation.
- Clarify Facet trait as extension point
- Clarify stages are informational metrics only
- Simplify AI acknowledgment
- Clarify import command requires file backend
- Fix Rust version badge, document properties query
- Add contributing section to README
- Restructure README, add architecture doc

### 🎨 Styling

- Format api_tests.rs

### 🧪 Testing

- Improve API query tests to verify response data
- Add auth middleware integration tests

### ⚙️ Miscellaneous Tasks

- Trigger workflows
- Remove unused kremis.toml and docs.rs link
- Hide _archive from repository
- Hide dev-docs from repository
- Remove unused LRU cache module
- Add permissions and cross-platform tests
- Bump version to 0.2.0, upgrade to edition 2024, update deps
- Centralize clippy lints in workspace config
