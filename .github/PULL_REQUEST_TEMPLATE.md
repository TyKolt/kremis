## Type of change

- [ ] `feat:` — new feature (requires MINOR version bump)
- [ ] `fix:` — bug fix (requires PATCH version bump)
- [ ] `docs:` — documentation only
- [ ] `refactor:` — code restructuring, no behavior change
- [ ] `chore:` — tooling, CI, dependencies

## Description

<!-- What does this PR do and why? -->

## Pre-commit checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo check --all-targets` passes
- [ ] `cargo test --workspace` passes

## Version bump (feat/fix only)

- [ ] Not applicable (docs/refactor/chore)
- [ ] Version bumped in `Cargo.toml`, `docs/api/overview.mdx`, `apps/kremis/tests/types_tests.rs`, `docs/openapi.yml`

## Documentation

- [ ] Not applicable
- [ ] CLI docs updated (`docs/cli/`)
- [ ] API docs updated (`docs/api/`)
- [ ] MCP docs updated (`docs/mcp/tools.mdx`)
- [ ] README updated (if observable behavior changed)

## Testing

<!-- How was this change tested? -->

- [ ] Covered by existing tests
- [ ] New tests added
- [ ] Manually tested — describe steps below

## Breaking changes

- [ ] No breaking changes
- [ ] Breaking change — described below

<!-- If breaking: what breaks and how should users migrate? -->
