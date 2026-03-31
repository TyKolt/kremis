# Changelog

All notable changes to Kremis are documented in this file.

## [0.17.6] - 2026-03-31

### Bug Fixes

- Accept UTF-8 BOM in CLI ingest JSON files ([#36](https://github.com/TyKolt/kremis/issues/36)) ([`9b69c62`](https://github.com/TyKolt/kremis/commit/9b69c62d143030a8c03c68d89cc5d1fe20f03e70))

## [0.17.5] - 2026-03-31

### Bug Fixes

- Reject CLI intersect queries with fewer than 2 nodes ([#37](https://github.com/TyKolt/kremis/issues/37)) ([`186afb2`](https://github.com/TyKolt/kremis/commit/186afb26007b0bed96245be444e04ca9c8c58726))

### Documentation

- Update community health files and GitHub templates to 2026 best practices ([`0518aa9`](https://github.com/TyKolt/kremis/commit/0518aa9ca90b4d7fb82606bae9e6781cbe0be382))

## [0.17.4] - 2026-03-30

### Bug Fixes

- Reject CLI traversal depth over limit instead of silent clamp ([#34](https://github.com/TyKolt/kremis/issues/34)) ([`fdc7a4d`](https://github.com/TyKolt/kremis/commit/fdc7a4d2080c4c7337c134ccf5a97718556214bd))

## [0.17.3] - 2026-03-30

### Bug Fixes

- Reject intersect queries with fewer than 2 nodes ([#33](https://github.com/TyKolt/kremis/issues/33)) ([`449a27c`](https://github.com/TyKolt/kremis/commit/449a27c2fd12f5cd38a8b69adad6b4bb6b6588fc))

### Documentation

- Mark project status as alpha ([`f41b6d1`](https://github.com/TyKolt/kremis/commit/f41b6d1f75aa3eada1c146387aa72d32e104c4c7))

## [0.17.2] - 2026-03-26

### Bug Fixes

- Add visit budget to strongest_path DFS to prevent exponential blowup ([#25](https://github.com/TyKolt/kremis/issues/25)) ([`6cd9a4a`](https://github.com/TyKolt/kremis/commit/6cd9a4a05e9f84c9a92ca4f233b125418db08849))

## [0.17.1] - 2026-03-26

### Bug Fixes

- From<SerializableGraph> silently discards dangling edges without diagnostics ([#28](https://github.com/TyKolt/kremis/issues/28)) ([`c4bf707`](https://github.com/TyKolt/kremis/commit/c4bf707156970339a95c3becfd464e5701cf2b0b))

## [0.17.0] - 2026-03-25

### Features

- Add --strict flag to ingest text mode ([#29](https://github.com/TyKolt/kremis/issues/29)) ([`b3c15d2`](https://github.com/TyKolt/kremis/commit/b3c15d2d399db8f17d427f2d5ccafba4daaa5b0e))

## [0.16.10] - 2026-03-25

### Bug Fixes

- Properties query returns found:true for existing node without properties ([#27](https://github.com/TyKolt/kremis/issues/27)) ([`ad6c237`](https://github.com/TyKolt/kremis/commit/ad6c23767de52d714288a6ad626dd880c82af9ea))

## [0.16.9] - 2026-03-25

### Bug Fixes

- Exempt /health from rate limiter ([#26](https://github.com/TyKolt/kremis/issues/26)) ([`cb449ba`](https://github.com/TyKolt/kremis/commit/cb449baa665dd016d3b24e7dbf094682238f7ea2))

### Dependencies

- *(deps)* Bump proptest in the rust-minor-patch group ([#32](https://github.com/TyKolt/kremis/issues/32)) ([`5c534c2`](https://github.com/TyKolt/kremis/commit/5c534c2050d097a76c2df338fd929ed70a7ea56b))

### Miscellaneous

- Update GitHub Actions to v6 (Node.js 24) ([`d86ec3c`](https://github.com/TyKolt/kremis/commit/d86ec3c64b8080134a2589164cb7061dd8ed2b0c))
- Fix actions/cache to v5 and download-artifact to v7 ([`b705693`](https://github.com/TyKolt/kremis/commit/b7056933f9044a9c34fb3935a3e762954235921f))
- Allow major updates for GitHub Actions in Dependabot ([`04b6945`](https://github.com/TyKolt/kremis/commit/04b6945deda0212c11dffd95e4395865a7e92823))

## [0.16.8] - 2026-03-25

### Bug Fixes

- Respect --json-mode flag on ingest command ([`31f939a`](https://github.com/TyKolt/kremis/commit/31f939aa758ffed8ae7f27cbd7743ab79558d471))

### Refactoring

- Remove dead code, extract helpers, deduplicate GraphStore ([`923982a`](https://github.com/TyKolt/kremis/commit/923982a90a507edb7b7c5075b4a881cf14c6358b))

### Documentation

- Fix stale GraphStore method count in architecture.mdx ([`57ad733`](https://github.com/TyKolt/kremis/commit/57ad733640c0a1dbc247d4e653f81c493b89a619))

## [0.16.7] - 2026-03-21

### Bug Fixes

- Resolve 7 bugs in core, CLI, and MCP bridge ([`5f46892`](https://github.com/TyKolt/kremis/commit/5f4689278e5e53452b7347bcca19b4e53896821f))

### Documentation

- Fix 10 stale values in API/MCP docs post v0.16.2–v0.16.6 ([`5926419`](https://github.com/TyKolt/kremis/commit/5926419ce40a6804c375b8e4ff49e28557291294))
- Update query-path algorithm and add import size limit note ([`618ce57`](https://github.com/TyKolt/kremis/commit/618ce578a380ee7bd2d5f4a4f71f71506d97bdc1))
- Fix query-path algorithm description stale after v0.16.5 ([`193705c`](https://github.com/TyKolt/kremis/commit/193705c92122522338ec0458fe50d39455d10379))

### Security

- Update rustls-webpki 0.103.9 → 0.103.10 (RUSTSEC-2026-0049) ([`8436bf3`](https://github.com/TyKolt/kremis/commit/8436bf31d852328b5052078ff3e03646c48e5424))

## [0.16.6] - 2026-03-20

### Bug Fixes

- Detect hash collision and deserialization errors in redb property buckets ([`bc883e4`](https://github.com/TyKolt/kremis/commit/bc883e489d9e08d65237a482bf3719cc27d64eb5))

## [0.16.5] - 2026-03-20

### Bug Fixes

- Strongest_path now correctly prefers stronger multi-hop paths ([`0e6ad60`](https://github.com/TyKolt/kremis/commit/0e6ad60d78b129f925757d15988419fd7c335977))

## [0.16.4] - 2026-03-20

### Bug Fixes

- Bound properties payload size before deserialization in import_canonical ([`72a362f`](https://github.com/TyKolt/kremis/commit/72a362f8a13c44706a0ab11a51dbb347d9cf2aa8))

## [0.16.3] - 2026-03-20

### Bug Fixes

- Propagate backend query failures correctly in HTTP and MCP layers ([`ec96971`](https://github.com/TyKolt/kremis/commit/ec9697107a36df02a5c3a3a7e30a870be3328fc2))

### Documentation

- Use version placeholder x.y.z in bug_report.yml ([`1e7e0ae`](https://github.com/TyKolt/kremis/commit/1e7e0aee86e8397d7f1017a4f8f6e25ccf9f94e6))

## [0.16.2] - 2026-03-19

### Bug Fixes

- Clamp next_node_id in from_canonical to prevent node ID collision ([`5e67986`](https://github.com/TyKolt/kremis/commit/5e679866ebbdfb22f1edbb99a5d34a4877e0134a))

### Dependencies

- *(deps)* Bump the rust-minor-patch group with 3 updates ([#19](https://github.com/TyKolt/kremis/issues/19)) ([`31f3426`](https://github.com/TyKolt/kremis/commit/31f342698b58d5e55c2d0cb343407bc88cd62278))

### Miscellaneous

- Add issue templates, PR template, and security policy ([`9bcedb9`](https://github.com/TyKolt/kremis/commit/9bcedb958110dc9ade1c9667e0b538ca59a659a2))

## [0.16.1] - 2026-03-14

### Bug Fixes

- Enforce consistent missing-node semantics for increment_edge ([`05a2f27`](https://github.com/TyKolt/kremis/commit/05a2f27ff81717d7de8f38a2130691aae7e28d0c))

## [0.16.0] - 2026-03-13

### Bug Fixes

- Propagate storage errors from Session instead of silencing them ([`51473d8`](https://github.com/TyKolt/kremis/commit/51473d81fa3e706e0f4528c4e20dbe9b1eea1e06))

## [0.15.3] - 2026-03-13

### Bug Fixes

- Enforce set semantics for duplicate (attribute, value) properties ([`8259541`](https://github.com/TyKolt/kremis/commit/8259541d8aeba4feceacfc0fa6acb2d1ad661c32))

### Miscellaneous

- Update Cargo.lock to v0.15.2 and fix README paragraph order ([`8f52164`](https://github.com/TyKolt/kremis/commit/8f52164e840babeb426abb6d2e40b713b625e2d9))

## [0.15.2] - 2026-03-13

### Bug Fixes

- Resolve CLI/config inconsistencies in json_mode, CORS origins, and error handling ([`1b0ae3c`](https://github.com/TyKolt/kremis/commit/1b0ae3c50bd4328f59769fd51167c848fcb827e8))

### Documentation

- Align OpenAPI health version with v0.15.1 ([`f6119f2`](https://github.com/TyKolt/kremis/commit/f6119f24194c2d2cb17b08f61e9d93c351c3bef1))
- Add collapsible table of contents to README ([`9a4c8a7`](https://github.com/TyKolt/kremis/commit/9a4c8a7c17b12e8f4ab77579081fbb876f5904f1))

### Miscellaneous

- Replace CodeRabbit with Greptile for AI code review ([`a588f93`](https://github.com/TyKolt/kremis/commit/a588f932e63269558077602afd2fc4805e276c15))
- Add CONTRIBUTING.md as Greptile custom context file reference ([`cd2e14a`](https://github.com/TyKolt/kremis/commit/cd2e14a9dcf8679b29fdc2f099de45d5f8753c9c))
- Add animated honesty demo SVG to README ([`73b8370`](https://github.com/TyKolt/kremis/commit/73b8370e5ecca069e36605eb4d776efddec2a778))

## [0.15.1] - 2026-03-10

### Bug Fixes

- *(mcp)* Align kremis_retract with HTTP retract contract, add MCP bridge tests ([`dbba8e0`](https://github.com/TyKolt/kremis/commit/dbba8e0a5e3c1bf688845c0641108add2eb435b8))

### Documentation

- Fix CLI overview, hash examples, and CONTRIBUTING version bump table ([`ae5bffa`](https://github.com/TyKolt/kremis/commit/ae5bffaf6df1506c54da4c17b7b773a0b9ea187c))
- Fix stage command link in CLI overview (use anchor /cli/status#stage) ([`6be822d`](https://github.com/TyKolt/kremis/commit/6be822d62e1fda643f9ab0acda44b810716a65c9))
- Use present tense in signal-batch.mdx (remove future 'will') ([`3a5cc29`](https://github.com/TyKolt/kremis/commit/3a5cc29ed70d49553ed6b291af451676ca8bd917))

### Miscellaneous

- *(deps)* Update redb 3.1.0→3.1.1, tokio 1.49.0→1.50.0 ([`f4e5459`](https://github.com/TyKolt/kremis/commit/f4e54593a2f03fa634e9950493be4f6d428b3ed3))
- Add CodeRabbit AI code review configuration ([`6e9110d`](https://github.com/TyKolt/kremis/commit/6e9110d22044556536824955ee2aadb42e3e22af))
- *(deps)* Patch quinn-proto 0.11.13→0.11.14, update wasm-bindgen/js-sys ([`b1c3cc8`](https://github.com/TyKolt/kremis/commit/b1c3cc84cfb840c2e8e4e008ba5ebbb0d5814572))

## [0.15.0] - 2026-03-08

### Features

- Add --from-stdin flag to kremis ingest (v0.15.0) ([`d31cd3a`](https://github.com/TyKolt/kremis/commit/d31cd3a04008f67b595af2e545cecef120244c3f))

### Documentation

- Fix MSRV badge and Quick Start text from 1.85 to 1.89 ([`9d06bb7`](https://github.com/TyKolt/kremis/commit/9d06bb7488ec26a33216d65609c50dd4e1ea5563))
- Fix MSRV in CONTRIBUTING.md and quickstart.mdx from 1.85 to 1.89 ([`21b3e22`](https://github.com/TyKolt/kremis/commit/21b3e22e97efe74781cdaa24f1db7e59dc1ce2ef))
- Fix MSRV in installation.mdx from 1.85 to 1.89 ([`c39fede`](https://github.com/TyKolt/kremis/commit/c39fede523fb1a77543d6a52f5676439f54d5bb6))
- Add design philosophy page ([`9db4d85`](https://github.com/TyKolt/kremis/commit/9db4d85628e4d5ab0c148eb151cc81c281cf736d))
- Align Precision law wording with grounding field value ([`e3ee921`](https://github.com/TyKolt/kremis/commit/e3ee921d0ca356db077bde9bd55a81c13d8df50f))
- Add CODE_OF_CONDUCT ([`0f96e56`](https://github.com/TyKolt/kremis/commit/0f96e560148871bd8d9b7853a52e2fb2fc021955))
- Add Kremis project name to CODE_OF_CONDUCT ([`825f254`](https://github.com/TyKolt/kremis/commit/825f2541324155f9bfecddfb535aa8960a537e23))
- Document --from-stdin flag in kremis ingest CLI reference ([`f147524`](https://github.com/TyKolt/kremis/commit/f1475243b065be474037371ecc8508b26b45d18a))

## [0.14.3] - 2026-03-07

### Bug Fixes

- Remove user-derived Vec::with_capacity to satisfy CodeQL CWE-770 — v0.14.3 ([`e3d00b6`](https://github.com/TyKolt/kremis/commit/e3d00b6c4e97c1713ca033292db039dd4b329808))

## [0.14.2] - 2026-03-07

### Bug Fixes

- Bound Vec::with_capacity to MAX_SEQUENCE_LENGTH (CWE-770) — v0.14.2 ([`e70f727`](https://github.com/TyKolt/kremis/commit/e70f7274939019406cbe082dc8ebbbf0f24f408b))

## [0.14.1] - 2026-03-06

### Bug Fixes

- Guard POST /signals against uncontrolled allocation (CWE-770) — v0.14.1 ([`56801cd`](https://github.com/TyKolt/kremis/commit/56801cddfb81f1a55ce36d74937ff172accb9e32))

## [0.14.0] - 2026-03-06

### Features

- Add POST /signals batch ingest endpoint — creates edges via HTTP (v0.14.0) ([`36abd70`](https://github.com/TyKolt/kremis/commit/36abd703b215fb67653e2f0503060a7cc7705052))

### Documentation

- Update CHANGELOG — close [Unreleased] → [0.13.1] ([`50e7641`](https://github.com/TyKolt/kremis/commit/50e7641814d6507fa28885f2c9d3ee780ef61bf8))

### Miscellaneous

- Add Ko-fi sponsor button via FUNDING.yml ([`387add5`](https://github.com/TyKolt/kremis/commit/387add584f1caaddd7609c44470601ecfdd06074))

## [0.13.1] - 2026-03-04

### Bug Fixes

- Redirect tracing logs to stderr, suppress banner in --json-mode (v0.13.1) ([`8db6eb0`](https://github.com/TyKolt/kremis/commit/8db6eb05868cb75afbdc9f8823958173262fc2a7))

### Documentation

- Add missing env vars to installation and mcp/setup pages ([`118f612`](https://github.com/TyKolt/kremis/commit/118f6123d3d2d9dbf6cfcb2a7bc763b4f44098c4))

## [0.13.0] - 2026-03-03

### Features

- Add AppConfig TOML-based configuration system, v0.12.0 ([`ff6e827`](https://github.com/TyKolt/kremis/commit/ff6e8272f377fdbd20f0692591dac1ad02d4c375))
- Add config provenance log (ConfigReport), v0.13.0 ([`ae18428`](https://github.com/TyKolt/kremis/commit/ae18428cad0462d06a14b9f879c65c4973500df5))

### Dependencies

- *(deps)* Bump tempfile in the rust-minor-patch group ([#6](https://github.com/TyKolt/kremis/issues/6)) ([`fe9ddf6`](https://github.com/TyKolt/kremis/commit/fe9ddf6805bfcb8af448d8ec28ef41d4c9aca4da))

### Miscellaneous

- Rename kremis.toml → kremis.example.toml, gitignore local config ([`9ae2e9c`](https://github.com/TyKolt/kremis/commit/9ae2e9c0db882dc2271b295f9fcaf82f0b08c9d6))
- Update tempfile 3.25.0 → 3.26.0, remove stale wasm transitive deps ([`d057e78`](https://github.com/TyKolt/kremis/commit/d057e786ae4d92f7579d811d4175ecbdf5f6efc6))

## [0.11.0] - 2026-03-01

### Features

- Add top_k to TraverseFiltered, v0.11.0 ([`37f083d`](https://github.com/TyKolt/kremis/commit/37f083dc110a7ce6e05e5fd0c196d9edc4eaa35b))

## [0.10.0] - 2026-02-28

### Features

- Structured JSON logging (KREMIS_LOG_FORMAT=json), v0.10.0 ([`f2895c5`](https://github.com/TyKolt/kremis/commit/f2895c5f74cfbd3b82547b557373d5037ebb9d2d))

### Documentation

- Update README MCP tool count 7 → 9 (kremis_retract, kremis_hash) ([`fb45f5a`](https://github.com/TyKolt/kremis/commit/fb45f5a62dd380ae2f533c0e0919fedd8acabe8a))
- Fix stale version examples in health.mdx and openapi.yml ([`5c60794`](https://github.com/TyKolt/kremis/commit/5c60794c34fc3e46378ba0ec78295b4febb108ec))

## [0.9.0] - 2026-02-27

### Features

- MCP parity — kremis_retract + kremis_hash tools, v0.9.0 ([`e265e69`](https://github.com/TyKolt/kremis/commit/e265e69d8b358f5dcc11ffef398bda55b70dfdbb))

### Documentation

- Docs: fix stale example values and cross-page inconsistency                                                                                                                                                                          - openapi.yml: update GET /health example response version 0.5.0 -> 0.8.0                                                   (inline example and HealthResponse schema example were not updated on release)                                          - query-traverse.mdx: align min_weight example with stable edge threshold (5 -> 10)                                         and add Tip linking to Developmental Stages for semantic context                                                        - cli/query.mdx: same alignment (--min-weight 5 -> 10)
  Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com> ([`be692dc`](https://github.com/TyKolt/kremis/commit/be692dc4dc432aeb02b8291d0a922114f2c88432))

## [0.8.0] - 2026-02-26

### Features

- Add decrement_edge + POST /signal/retract for edge invalidation ([`e579c97`](https://github.com/TyKolt/kremis/commit/e579c970335b557cfb71b509c76d06ccb0ad2092))

### Documentation

- Add CHANGELOG.md with git-cliff and update release workflow ([`da5b949`](https://github.com/TyKolt/kremis/commit/da5b94906f0f8d7c9efa398a329e61ea6bab1b86))

## [0.7.0] - 2026-02-26

### Features

- Add GET /hash (BLAKE3), GET /metrics (Prometheus), bench-compile CI ([`17ce060`](https://github.com/TyKolt/kremis/commit/17ce060b6ade75beda572900eed26405988c5a12))

### Miscellaneous

- *(ci)* Pin @redocly/cli@2 and cache npm in OpenAPI Lint job ([`4bb67c2`](https://github.com/TyKolt/kremis/commit/4bb67c298472715d32cffe49734ab58d1d55b3ea))

## [0.6.1] - 2026-02-25

### Bug Fixes

- Replace DefaultHasher with stable FNV-1a for PROPERTIES table key ([`8294454`](https://github.com/TyKolt/kremis/commit/82944546343bdb0902712aa77e1b2e327a0343b7))

## [0.6.0] - 2026-02-24

### Features

- Add batch ingest to RedbGraph for O(1) transaction overhead ([`c5ceae5`](https://github.com/TyKolt/kremis/commit/c5ceae54e4a9826720ae148c228ebefa3df6936f))

### Documentation

- Add honesty demo script and README section ([`0a72768`](https://github.com/TyKolt/kremis/commit/0a7276831561a637f838e6c0e73a3d657f270b7d))
- Add merge authority section and sanitize config files ([`da0de66`](https://github.com/TyKolt/kremis/commit/da0de6670cd3f31b47616c7f63970e6e7b317fe2))

### Miscellaneous

- Add Claude Code project settings ([`2ac299a`](https://github.com/TyKolt/kremis/commit/2ac299abe77b45617c0f98adef7ec52e2e5b16e6))
- *(deps)* Update Rust dependencies ([`29c4801`](https://github.com/TyKolt/kremis/commit/29c48017621f74a0085cd9eb5180a70ddd5d5983))

## [0.5.0] - 2026-02-21

### Features

- Add diagnostic field to QueryResponse (Honesty Protocol) ([`7712eab`](https://github.com/TyKolt/kremis/commit/7712eab003d5b8010d74a729f8fbe7631ae23ad6))

### Documentation

- Add OpenAPI 3.1.0 spec and CI lint job ([`94289e4`](https://github.com/TyKolt/kremis/commit/94289e4e3693cfb42f5aec1c974daf67e0bc5c5b))
- Update QueryResponse convention in CONTRIBUTING.md ([`7ceb063`](https://github.com/TyKolt/kremis/commit/7ceb063060d862192fcf1833ff400f2f35bb3eb8))

### Miscellaneous

- Update Cargo.lock to version 0.4.0 ([`94aa596`](https://github.com/TyKolt/kremis/commit/94aa5963b85b02086475d836d2d2b8b29ebdae07))

## [0.4.0] - 2026-02-19

### Features

- Add grounding field to QueryResponse (Honesty Protocol) ([`2cc5388`](https://github.com/TyKolt/kremis/commit/2cc5388f9a2820f2ddf15e36b1e9ccef5296686d))

### Bug Fixes

- Init --force now properly resets database, improve README quickstart flow ([`adf13fe`](https://github.com/TyKolt/kremis/commit/adf13fe795a73456f91f3e500632bf2c62e03053))

### Documentation

- Add sample data and Try It quickstart section ([`f474987`](https://github.com/TyKolt/kremis/commit/f474987a4dee7e4491a977ecfa2293cfc735add4))
- Add CONTRIBUTING.md and fix README inaccuracies ([`4fa0ca6`](https://github.com/TyKolt/kremis/commit/4fa0ca62f047a894a00e20c78d603df7399d8f32))
- Remove private dev-docs references from CONTRIBUTING.md ([`7dbed05`](https://github.com/TyKolt/kremis/commit/7dbed05e157230305363d90ec27328a145ee97df))
- Migrate documentation to Mintlify MDX format ([`ec45858`](https://github.com/TyKolt/kremis/commit/ec45858f3760827081ee8c8fe5319f3daa64b810))
- Add Mintlify docs badge to README ([`c84815e`](https://github.com/TyKolt/kremis/commit/c84815e819aaf5226ddc1354dc7ab193f201db9e))
- Add grounding field to all query response examples ([`97217ce`](https://github.com/TyKolt/kremis/commit/97217ce0ad99c7c70290fb3617a08b4f80501c5f))

### Dependencies

- Update clap 4.5.59, tempfile 3.25.0 ([`35f4ada`](https://github.com/TyKolt/kremis/commit/35f4ada3ffddefb3d799dd3627f23ff1d188be55))

### Miscellaneous

- Align v0.3.0 references and centralize workspace deps ([`7b08e64`](https://github.com/TyKolt/kremis/commit/7b08e64608b7b0a4a332a14fd86fb69779dc7b03))
- Remove deleted labels from dependabot config ([`408c011`](https://github.com/TyKolt/kremis/commit/408c011250d3694c64e1002fd9edc36ed5170fac))
- Remove private doc references from codebase, update docs config ([`318a40a`](https://github.com/TyKolt/kremis/commit/318a40abf571d5e12e3bfa0da6fbe62f00dab242))
- Migrate alias from M2Dr3g0n to TyKolt ([`dd0de12`](https://github.com/TyKolt/kremis/commit/dd0de125df170d6e9e2adc9e17e1051d0d422050))
- Add brand identity — logo, favicon, OG image ([`28cc022`](https://github.com/TyKolt/kremis/commit/28cc02242c7dfe4a441594760fee96bae97e5fa6))
- Replace NOTICE with LICENSE for brand assets, update README ([`aa3b1f2`](https://github.com/TyKolt/kremis/commit/aa3b1f27940e3554bf3109f71790663e3f5ff62f))

## [0.3.0] - 2026-02-14

### Features

- Add Dockerfile for containerized deployment ([`c961dc7`](https://github.com/TyKolt/kremis/commit/c961dc71d1565363ae66f2148ca6fa5dbf0de0cf))
- Add kremis-mcp — MCP server bridge for AI assistants (beta) ([`161003a`](https://github.com/TyKolt/kremis/commit/161003ab7e58a6846d732faa85c450209fa97726))

### Documentation

- Add Docker usage section to README ([`17870aa`](https://github.com/TyKolt/kremis/commit/17870aa96a8c2156dc1caedc789077f86f78c36f))
- Add MCP server section to API.md and CLI.md ([`b56a002`](https://github.com/TyKolt/kremis/commit/b56a002a2df3080bd0375bc138ef57bedd5639ff))
- Add MCP Server to API.md table of contents ([`50542a4`](https://github.com/TyKolt/kremis/commit/50542a4e2760ca81291c8f9ebaecc31c99c499fa))

### Miscellaneous

- Bump version to 0.2.1 ([`44986d4`](https://github.com/TyKolt/kremis/commit/44986d495ede58edb59b80e8ffa5f186e6b660e1))
- Bump version to 0.3.0 ([`3d05679`](https://github.com/TyKolt/kremis/commit/3d05679b36c67e10e179663f3ffc67ba405c50cb))
- Add social/, video/, nul to .gitignore ([`e0d7f04`](https://github.com/TyKolt/kremis/commit/e0d7f0421a2a2d78ecd4d2b6702aeb0a6681b3a0))

## [0.2.0] - 2026-02-11

### Features

- Kremis v0.1.0 - deterministic graph cognitive substrate ([`9f20443`](https://github.com/TyKolt/kremis/commit/9f20443ec7a43987356bde8fca0307a1ea095df9))
- Persist signal attributes and values as node properties ([`f1c51bf`](https://github.com/TyKolt/kremis/commit/f1c51bf28ce2e62be92019f9dc545d998855e376))

### Bug Fixes

- Preserve node properties across export/import and expose via API/CLI ([`8d52eb3`](https://github.com/TyKolt/kremis/commit/8d52eb33c0692192b5c0352df0d7a7c675f6afba))
- Add CLI related query and prevent dangling edges in RedbGraph ([`bb143a0`](https://github.com/TyKolt/kremis/commit/bb143a05a6b00f458e9bfe239a5c00b36804307a))
- Align docs with code, remove dead code, harden rate limiter ([`bc09c60`](https://github.com/TyKolt/kremis/commit/bc09c60a9f3d46ed997de6fc509eb37364b11507))
- Add checks:write permission to audit job in release workflow ([`f375afc`](https://github.com/TyKolt/kremis/commit/f375afc22a7ea4335386c1d10fe295efc6d38dfe))

### Refactoring

- Unify GraphStore trait to eliminate Graph/RedbGraph duplication ([`5504c9a`](https://github.com/TyKolt/kremis/commit/5504c9aaafc191dc23ec077b6f0573ee5888510e))
- Remove related_subgraph duplicate of traverse ([`f7e9be5`](https://github.com/TyKolt/kremis/commit/f7e9be5dd100f016b3e8ef650ccb2c5d9dca1a72))

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

### Dependencies

- Bump bytes 1.11.0 -> 1.11.1 (security fix) ([`69dd68c`](https://github.com/TyKolt/kremis/commit/69dd68cc3aa7c5b1d118b28432c9357673248fda))
- Update dependencies ([`b1dc01f`](https://github.com/TyKolt/kremis/commit/b1dc01f6e5297a2ae8f6d9ea522806218c30dada))

### Miscellaneous

- Remove unused kremis.toml and docs.rs link ([`505b77b`](https://github.com/TyKolt/kremis/commit/505b77b15a0c0c9283c13de07a71c9bc370d1679))
- Hide _archive from repository ([`ee9c22d`](https://github.com/TyKolt/kremis/commit/ee9c22d06736deccf812f2114247cd216966c1e2))
- Hide dev-docs from repository ([`8689fe4`](https://github.com/TyKolt/kremis/commit/8689fe4471da33e9fd769d09cb71e6a2306d20f5))
- Remove unused LRU cache module ([`c37c9c9`](https://github.com/TyKolt/kremis/commit/c37c9c977dc6db30f4230836eaf00a489cdd6251))
- Bump version to 0.2.0, upgrade to edition 2024, update deps ([`2c11661`](https://github.com/TyKolt/kremis/commit/2c116613f42af7c75bf1befd5650a9e83cca24a9))
- Centralize clippy lints in workspace config ([`b670bdd`](https://github.com/TyKolt/kremis/commit/b670bdd957e6c09c180313fb64f0f57e7a2b7888))

---
*Generated by [git-cliff](https://git-cliff.org)*
