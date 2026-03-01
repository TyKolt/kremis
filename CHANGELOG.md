# Changelog

All notable changes to Kremis are documented in this file.

## [Unreleased]

### Documentation

- Regenerate CHANGELOG for v0.10.0 ([`c509522`](https://github.com/TyKolt/kremis/commit/c50952225c18636071a996220117b2017e2a3290))

## [0.10.0] - 2026-02-28

### Documentation

- Update README MCP tool count 7 → 9 (kremis_retract, kremis_hash) ([`fb45f5a`](https://github.com/TyKolt/kremis/commit/fb45f5a62dd380ae2f533c0e0919fedd8acabe8a))
- Fix stale version examples in health.mdx and openapi.yml ([`5c60794`](https://github.com/TyKolt/kremis/commit/5c60794c34fc3e46378ba0ec78295b4febb108ec))

### Features

- Structured JSON logging (KREMIS_LOG_FORMAT=json), v0.10.0 ([`f2895c5`](https://github.com/TyKolt/kremis/commit/f2895c5f74cfbd3b82547b557373d5037ebb9d2d))

## [0.9.0] - 2026-02-27

### Documentation

- Docs: fix stale example values and cross-page inconsistency                                                                                                                                                                          - openapi.yml: update GET /health example response version 0.5.0 -> 0.8.0                                                   (inline example and HealthResponse schema example were not updated on release)                                          - query-traverse.mdx: align min_weight example with stable edge threshold (5 -> 10)                                         and add Tip linking to Developmental Stages for semantic context                                                        - cli/query.mdx: same alignment (--min-weight 5 -> 10)
  Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com> ([`be692dc`](https://github.com/TyKolt/kremis/commit/be692dc4dc432aeb02b8291d0a922114f2c88432))
- Regenerate CHANGELOG for v0.9.0 ([`816555c`](https://github.com/TyKolt/kremis/commit/816555cc434a39a3cced4a3c3a28ca801e079dc7))

### Features

- MCP parity — kremis_retract + kremis_hash tools, v0.9.0 ([`e265e69`](https://github.com/TyKolt/kremis/commit/e265e69d8b358f5dcc11ffef398bda55b70dfdbb))

## [0.8.0] - 2026-02-26

### Documentation

- Add CHANGELOG.md with git-cliff and update release workflow ([`da5b949`](https://github.com/TyKolt/kremis/commit/da5b94906f0f8d7c9efa398a329e61ea6bab1b86))
- Regenerate CHANGELOG for v0.8.0 ([`99822bd`](https://github.com/TyKolt/kremis/commit/99822bdd93c7da226419d250222e0d8052aff866))

### Features

- Add decrement_edge + POST /signal/retract for edge invalidation ([`e579c97`](https://github.com/TyKolt/kremis/commit/e579c970335b557cfb71b509c76d06ccb0ad2092))

## [0.7.0] - 2026-02-26

### Features

- Add GET /hash (BLAKE3), GET /metrics (Prometheus), bench-compile CI ([`17ce060`](https://github.com/TyKolt/kremis/commit/17ce060b6ade75beda572900eed26405988c5a12))

## [0.6.1] - 2026-02-25

### Bug Fixes

- Replace DefaultHasher with stable FNV-1a for PROPERTIES table key ([`8294454`](https://github.com/TyKolt/kremis/commit/82944546343bdb0902712aa77e1b2e327a0343b7))

## [0.6.0] - 2026-02-24

### Documentation

- Add honesty demo script and README section ([`0a72768`](https://github.com/TyKolt/kremis/commit/0a7276831561a637f838e6c0e73a3d657f270b7d))
- Add merge authority section and sanitize config files ([`da0de66`](https://github.com/TyKolt/kremis/commit/da0de6670cd3f31b47616c7f63970e6e7b317fe2))

### Features

- Add batch ingest to RedbGraph for O(1) transaction overhead ([`c5ceae5`](https://github.com/TyKolt/kremis/commit/c5ceae54e4a9826720ae148c228ebefa3df6936f))

## [0.5.0] - 2026-02-21

### Documentation

- Add OpenAPI 3.1.0 spec and CI lint job ([`94289e4`](https://github.com/TyKolt/kremis/commit/94289e4e3693cfb42f5aec1c974daf67e0bc5c5b))
- Update QueryResponse convention in CONTRIBUTING.md ([`7ceb063`](https://github.com/TyKolt/kremis/commit/7ceb063060d862192fcf1833ff400f2f35bb3eb8))

### Features

- Add diagnostic field to QueryResponse (Honesty Protocol) ([`7712eab`](https://github.com/TyKolt/kremis/commit/7712eab003d5b8010d74a729f8fbe7631ae23ad6))

## [0.4.0] - 2026-02-19

### Bug Fixes

- Init --force now properly resets database, improve README quickstart flow ([`adf13fe`](https://github.com/TyKolt/kremis/commit/adf13fe795a73456f91f3e500632bf2c62e03053))

### Dependencies

- Update clap 4.5.59, tempfile 3.25.0 ([`35f4ada`](https://github.com/TyKolt/kremis/commit/35f4ada3ffddefb3d799dd3627f23ff1d188be55))

### Documentation

- Add sample data and Try It quickstart section ([`f474987`](https://github.com/TyKolt/kremis/commit/f474987a4dee7e4491a977ecfa2293cfc735add4))
- Add CONTRIBUTING.md and fix README inaccuracies ([`4fa0ca6`](https://github.com/TyKolt/kremis/commit/4fa0ca62f047a894a00e20c78d603df7399d8f32))
- Remove private dev-docs references from CONTRIBUTING.md ([`7dbed05`](https://github.com/TyKolt/kremis/commit/7dbed05e157230305363d90ec27328a145ee97df))
- Migrate documentation to Mintlify MDX format ([`ec45858`](https://github.com/TyKolt/kremis/commit/ec45858f3760827081ee8c8fe5319f3daa64b810))
- Add Mintlify docs badge to README ([`c84815e`](https://github.com/TyKolt/kremis/commit/c84815e819aaf5226ddc1354dc7ab193f201db9e))
- Add grounding field to all query response examples ([`97217ce`](https://github.com/TyKolt/kremis/commit/97217ce0ad99c7c70290fb3617a08b4f80501c5f))

### Features

- Add grounding field to QueryResponse (Honesty Protocol) ([`2cc5388`](https://github.com/TyKolt/kremis/commit/2cc5388f9a2820f2ddf15e36b1e9ccef5296686d))

## [0.3.0] - 2026-02-14

### Documentation

- Add Docker usage section to README ([`17870aa`](https://github.com/TyKolt/kremis/commit/17870aa96a8c2156dc1caedc789077f86f78c36f))
- Add MCP server section to API.md and CLI.md ([`b56a002`](https://github.com/TyKolt/kremis/commit/b56a002a2df3080bd0375bc138ef57bedd5639ff))
- Add MCP Server to API.md table of contents ([`50542a4`](https://github.com/TyKolt/kremis/commit/50542a4e2760ca81291c8f9ebaecc31c99c499fa))

### Features

- Add Dockerfile for containerized deployment ([`c961dc7`](https://github.com/TyKolt/kremis/commit/c961dc71d1565363ae66f2148ca6fa5dbf0de0cf))
- Add kremis-mcp — MCP server bridge for AI assistants (beta) ([`161003a`](https://github.com/TyKolt/kremis/commit/161003ab7e58a6846d732faa85c450209fa97726))

## [0.2.0] - 2026-02-11

### Bug Fixes

- Preserve node properties across export/import and expose via API/CLI ([`8d52eb3`](https://github.com/TyKolt/kremis/commit/8d52eb33c0692192b5c0352df0d7a7c675f6afba))
- Add CLI related query and prevent dangling edges in RedbGraph ([`bb143a0`](https://github.com/TyKolt/kremis/commit/bb143a05a6b00f458e9bfe239a5c00b36804307a))
- Align docs with code, remove dead code, harden rate limiter ([`bc09c60`](https://github.com/TyKolt/kremis/commit/bc09c60a9f3d46ed997de6fc509eb37364b11507))
- Add checks:write permission to audit job in release workflow ([`f375afc`](https://github.com/TyKolt/kremis/commit/f375afc22a7ea4335386c1d10fe295efc6d38dfe))

### Dependencies

- Bump bytes 1.11.0 -> 1.11.1 (security fix) ([`69dd68c`](https://github.com/TyKolt/kremis/commit/69dd68cc3aa7c5b1d118b28432c9357673248fda))
- Update dependencies ([`b1dc01f`](https://github.com/TyKolt/kremis/commit/b1dc01f6e5297a2ae8f6d9ea522806218c30dada))

### Documentation

- Fix _archive reference and update dates ([`330f1e9`](https://github.com/TyKolt/kremis/commit/330f1e9d30eb165f08bb10cafc2178353b04d450))
- Fix incorrect example outputs in API.md ([`3667e12`](https://github.com/TyKolt/kremis/commit/3667e12956177c3b0dc450044d5fa8631d35c1ad))
- Add AI acknowledgment to README, fix docs accuracy ([`426c1a8`](https://github.com/TyKolt/kremis/commit/426c1a8a9dc3c1a679d1e3586a946ac7ba510199))
- Update project documentation. ([`57b12ff`](https://github.com/TyKolt/kremis/commit/57b12ffae5a0b5d8578522404c4eab1624081a2c))
- Clarify Facet trait as extension point ([`34deb9a`](https://github.com/TyKolt/kremis/commit/34deb9a31ea2d3dc046359b34091d0199c08f8e8))
- Clarify stages are informational metrics only ([`3de7e48`](https://github.com/TyKolt/kremis/commit/3de7e4830f86cf7566e0825687d5e5ba7f169c13))
- Simplify AI acknowledgment ([`69f8640`](https://github.com/TyKolt/kremis/commit/69f86408c9885f0b9791de7dabe4bc9dac3b02a2))
- Clarify import command requires file backend ([`424f5f8`](https://github.com/TyKolt/kremis/commit/424f5f8eb839df37e87bebc6dea804b16258b9ab))
- Fix Rust version badge, document properties query ([`b5fc0a5`](https://github.com/TyKolt/kremis/commit/b5fc0a581032d55a0c2e4707c1f5f5120e0e99af))
- Add contributing section to README ([`5d24bc5`](https://github.com/TyKolt/kremis/commit/5d24bc5c910ca6202e06ba5c0c03a99eab201557))
- Restructure README, add architecture doc ([`34e9085`](https://github.com/TyKolt/kremis/commit/34e90851f123b11dc38f91ba5f9d1c69fac3d1c9))

### Features

- Kremis v0.1.0 - deterministic graph cognitive substrate ([`9f20443`](https://github.com/TyKolt/kremis/commit/9f20443ec7a43987356bde8fca0307a1ea095df9))
- Persist signal attributes and values as node properties ([`f1c51bf`](https://github.com/TyKolt/kremis/commit/f1c51bf28ce2e62be92019f9dc545d998855e376))

---
*Generated by [git-cliff](https://git-cliff.org)*
