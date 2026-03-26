# Changelog

All notable changes to Kremis are documented in this file.

## [0.17.1] - 2026-03-26

### Bug Fixes

- From<SerializableGraph> silently discards dangling edges without diagnostics (#28) ([`c4bf707`](https://github.com/TyKolt/kremis/commit/c4bf707156970339a95c3becfd464e5701cf2b0b))

### Documentation

- Regenerate CHANGELOG for v0.17.0 [skip ci] ([`56b57dd`](https://github.com/TyKolt/kremis/commit/56b57ddbc74c16282806831bd2e5993d00c7a918))

## [0.17.0] - 2026-03-25

### Documentation

- Regenerate CHANGELOG for v0.16.10 [skip ci] ([`75372cf`](https://github.com/TyKolt/kremis/commit/75372cfafb74f1ebb7618f8630e133d9337e779f))

### Features

- Add --strict flag to ingest text mode (#29) ([`b3c15d2`](https://github.com/TyKolt/kremis/commit/b3c15d2d399db8f17d427f2d5ccafba4daaa5b0e))

## [0.16.10] - 2026-03-25

### Bug Fixes

- Properties query returns found:true for existing node without properties (#27) ([`ad6c237`](https://github.com/TyKolt/kremis/commit/ad6c23767de52d714288a6ad626dd880c82af9ea))

### Documentation

- Regenerate CHANGELOG for v0.16.9 [skip ci] ([`5cb4e60`](https://github.com/TyKolt/kremis/commit/5cb4e60ff318621837dc377fbf225d3a98646f0b))

## [0.16.9] - 2026-03-25

### Bug Fixes

- Exempt /health from rate limiter (#26) ([`cb449ba`](https://github.com/TyKolt/kremis/commit/cb449baa665dd016d3b24e7dbf094682238f7ea2))

### Dependencies

- Bump proptest in the rust-minor-patch group (#32) ([`5c534c2`](https://github.com/TyKolt/kremis/commit/5c534c2050d097a76c2df338fd929ed70a7ea56b))

### Documentation

- Regenerate CHANGELOG for v0.16.8 [skip ci] ([`9cc7d8d`](https://github.com/TyKolt/kremis/commit/9cc7d8daaf8408dfecaba45eb1002b20652675d1))

## [0.16.8] - 2026-03-25

### Bug Fixes

- Respect --json-mode flag on ingest command ([`31f939a`](https://github.com/TyKolt/kremis/commit/31f939aa758ffed8ae7f27cbd7743ab79558d471))

### Documentation

- Fix stale GraphStore method count in architecture.mdx ([`57ad733`](https://github.com/TyKolt/kremis/commit/57ad733640c0a1dbc247d4e653f81c493b89a619))

## [0.16.7] - 2026-03-21

### Bug Fixes

- Resolve 7 bugs in core, CLI, and MCP bridge ([`5f46892`](https://github.com/TyKolt/kremis/commit/5f4689278e5e53452b7347bcca19b4e53896821f))

### Documentation

- Regenerate CHANGELOG for v0.16.6 [skip ci] ([`3b528e7`](https://github.com/TyKolt/kremis/commit/3b528e710f184ae800b57deae2d54f1369d774b0))
- Fix 10 stale values in API/MCP docs post v0.16.2–v0.16.6 ([`5926419`](https://github.com/TyKolt/kremis/commit/5926419ce40a6804c375b8e4ff49e28557291294))
- Update query-path algorithm and add import size limit note ([`618ce57`](https://github.com/TyKolt/kremis/commit/618ce578a380ee7bd2d5f4a4f71f71506d97bdc1))
- Fix query-path algorithm description stale after v0.16.5 ([`193705c`](https://github.com/TyKolt/kremis/commit/193705c92122522338ec0458fe50d39455d10379))
- Regenerate CHANGELOG for v0.16.7 [skip ci] ([`12c4ef5`](https://github.com/TyKolt/kremis/commit/12c4ef516c8be6fc40e924a4a41880b3c0c0d90a))

## [0.16.6] - 2026-03-20

### Bug Fixes

- Detect hash collision and deserialization errors in redb property buckets ([`bc883e4`](https://github.com/TyKolt/kremis/commit/bc883e489d9e08d65237a482bf3719cc27d64eb5))

### Documentation

- Regenerate CHANGELOG for v0.16.5 [skip ci] ([`da50da8`](https://github.com/TyKolt/kremis/commit/da50da83c6a46254dfec77821cbe58f22d569ff1))

## [0.16.5] - 2026-03-20

### Bug Fixes

- Strongest_path now correctly prefers stronger multi-hop paths ([`0e6ad60`](https://github.com/TyKolt/kremis/commit/0e6ad60d78b129f925757d15988419fd7c335977))

### Documentation

- Regenerate CHANGELOG for v0.16.4 [skip ci] ([`1840c55`](https://github.com/TyKolt/kremis/commit/1840c5547fcc68ea4142122c6a15d3b717cf0c91))

## [0.16.4] - 2026-03-20

### Bug Fixes

- Bound properties payload size before deserialization in import_canonical ([`72a362f`](https://github.com/TyKolt/kremis/commit/72a362f8a13c44706a0ab11a51dbb347d9cf2aa8))

### Documentation

- Regenerate CHANGELOG for v0.16.3 [skip ci] ([`366d6fc`](https://github.com/TyKolt/kremis/commit/366d6fc92ee27679e811dd23fe8a83d6fd3638bb))

## [0.16.3] - 2026-03-20

### Bug Fixes

- Propagate backend query failures correctly in HTTP and MCP layers ([`ec96971`](https://github.com/TyKolt/kremis/commit/ec9697107a36df02a5c3a3a7e30a870be3328fc2))

### Documentation

- Regenerate CHANGELOG for v0.16.2 [skip ci] ([`c8a30af`](https://github.com/TyKolt/kremis/commit/c8a30af24210de7212c6faf8ae3c59638df9fa44))
- Use version placeholder x.y.z in bug_report.yml ([`1e7e0ae`](https://github.com/TyKolt/kremis/commit/1e7e0aee86e8397d7f1017a4f8f6e25ccf9f94e6))

## [0.16.2] - 2026-03-19

### Bug Fixes

- Clamp next_node_id in from_canonical to prevent node ID collision ([`5e67986`](https://github.com/TyKolt/kremis/commit/5e679866ebbdfb22f1edbb99a5d34a4877e0134a))

### Dependencies

- Bump the rust-minor-patch group with 3 updates (#19) ([`31f3426`](https://github.com/TyKolt/kremis/commit/31f342698b58d5e55c2d0cb343407bc88cd62278))

### Documentation

- Regenerate CHANGELOG for v0.16.1 [skip ci] ([`1a4d854`](https://github.com/TyKolt/kremis/commit/1a4d854747a0f92abb0c6f21d41e2ddbe043e029))

## [0.16.1] - 2026-03-14

### Bug Fixes

- Enforce consistent missing-node semantics for increment_edge ([`05a2f27`](https://github.com/TyKolt/kremis/commit/05a2f27ff81717d7de8f38a2130691aae7e28d0c))

### Documentation

- Regenerate CHANGELOG for v0.16.0 [skip ci] ([`670015d`](https://github.com/TyKolt/kremis/commit/670015d74749a234a7186fe6c722abe7a00d1294))

## [0.16.0] - 2026-03-13

### Bug Fixes

- Propagate storage errors from Session instead of silencing them ([`51473d8`](https://github.com/TyKolt/kremis/commit/51473d81fa3e706e0f4528c4e20dbe9b1eea1e06))

### Documentation

- Regenerate CHANGELOG for v0.15.3 [skip ci] ([`2d48bcd`](https://github.com/TyKolt/kremis/commit/2d48bcd4bb2017627a56bf0497fd704c6ab01305))

## [0.15.3] - 2026-03-13

### Bug Fixes

- Enforce set semantics for duplicate (attribute, value) properties ([`8259541`](https://github.com/TyKolt/kremis/commit/8259541d8aeba4feceacfc0fa6acb2d1ad661c32))

### Documentation

- Regenerate CHANGELOG for v0.15.2 [skip ci] ([`f641a9f`](https://github.com/TyKolt/kremis/commit/f641a9ff1de68df48802b5f331b2b81445ec5e55))

## [0.15.2] - 2026-03-13

### Bug Fixes

- Resolve CLI/config inconsistencies in json_mode, CORS origins, and error handling ([`1b0ae3c`](https://github.com/TyKolt/kremis/commit/1b0ae3c50bd4328f59769fd51167c848fcb827e8))

### Documentation

- Regenerate CHANGELOG for v0.15.1 [skip ci] ([`13d6f01`](https://github.com/TyKolt/kremis/commit/13d6f01fec57e552f3cb69f95568bb838ff4a823))
- Align OpenAPI health version with v0.15.1 ([`f6119f2`](https://github.com/TyKolt/kremis/commit/f6119f24194c2d2cb17b08f61e9d93c351c3bef1))
- Add collapsible table of contents to README ([`9a4c8a7`](https://github.com/TyKolt/kremis/commit/9a4c8a7c17b12e8f4ab77579081fbb876f5904f1))

## [0.15.1] - 2026-03-10

### Bug Fixes

- Align kremis_retract with HTTP retract contract, add MCP bridge tests ([`dbba8e0`](https://github.com/TyKolt/kremis/commit/dbba8e0a5e3c1bf688845c0641108add2eb435b8))

### Documentation

- Fix CLI overview, hash examples, and CONTRIBUTING version bump table ([`ae5bffa`](https://github.com/TyKolt/kremis/commit/ae5bffaf6df1506c54da4c17b7b773a0b9ea187c))
- Fix stage command link in CLI overview (use anchor /cli/status#stage) ([`6be822d`](https://github.com/TyKolt/kremis/commit/6be822d62e1fda643f9ab0acda44b810716a65c9))
- Use present tense in signal-batch.mdx (remove future 'will') ([`3a5cc29`](https://github.com/TyKolt/kremis/commit/3a5cc29ed70d49553ed6b291af451676ca8bd917))

## [0.15.0] - 2026-03-08

### Documentation

- Regenerate CHANGELOG for v0.14.3 [skip ci] ([`0875cf6`](https://github.com/TyKolt/kremis/commit/0875cf61a18103dbfc17337b181f97ab99e04877))
- Fix MSRV badge and Quick Start text from 1.85 to 1.89 ([`9d06bb7`](https://github.com/TyKolt/kremis/commit/9d06bb7488ec26a33216d65609c50dd4e1ea5563))
- Fix MSRV in CONTRIBUTING.md and quickstart.mdx from 1.85 to 1.89 ([`21b3e22`](https://github.com/TyKolt/kremis/commit/21b3e22e97efe74781cdaa24f1db7e59dc1ce2ef))
- Fix MSRV in installation.mdx from 1.85 to 1.89 ([`c39fede`](https://github.com/TyKolt/kremis/commit/c39fede523fb1a77543d6a52f5676439f54d5bb6))
- Add design philosophy page ([`9db4d85`](https://github.com/TyKolt/kremis/commit/9db4d85628e4d5ab0c148eb151cc81c281cf736d))
- Align Precision law wording with grounding field value ([`e3ee921`](https://github.com/TyKolt/kremis/commit/e3ee921d0ca356db077bde9bd55a81c13d8df50f))
- Add CODE_OF_CONDUCT ([`0f96e56`](https://github.com/TyKolt/kremis/commit/0f96e560148871bd8d9b7853a52e2fb2fc021955))
- Add Kremis project name to CODE_OF_CONDUCT ([`825f254`](https://github.com/TyKolt/kremis/commit/825f2541324155f9bfecddfb535aa8960a537e23))
- Document --from-stdin flag in kremis ingest CLI reference ([`f147524`](https://github.com/TyKolt/kremis/commit/f1475243b065be474037371ecc8508b26b45d18a))
- Regenerate CHANGELOG for v0.15.0 [skip ci] ([`b03e35b`](https://github.com/TyKolt/kremis/commit/b03e35b0ee1ff6747bf00a2d84b0e15634eab7e8))

### Features

- Add --from-stdin flag to kremis ingest (v0.15.0) ([`d31cd3a`](https://github.com/TyKolt/kremis/commit/d31cd3a04008f67b595af2e545cecef120244c3f))

## [0.14.3] - 2026-03-07

### Bug Fixes

- Remove user-derived Vec::with_capacity to satisfy CodeQL CWE-770 — v0.14.3 ([`e3d00b6`](https://github.com/TyKolt/kremis/commit/e3d00b6c4e97c1713ca033292db039dd4b329808))

## [0.14.2] - 2026-03-07

### Bug Fixes

- Bound Vec::with_capacity to MAX_SEQUENCE_LENGTH (CWE-770) — v0.14.2 ([`e70f727`](https://github.com/TyKolt/kremis/commit/e70f7274939019406cbe082dc8ebbbf0f24f408b))

### Documentation

- Regenerate CHANGELOG for v0.14.2 [skip ci] ([`5ff55f4`](https://github.com/TyKolt/kremis/commit/5ff55f4e0cc21c0bde5e554a028723898009a791))

## [0.14.1] - 2026-03-06

### Bug Fixes

- Guard POST /signals against uncontrolled allocation (CWE-770) — v0.14.1 ([`56801cd`](https://github.com/TyKolt/kremis/commit/56801cddfb81f1a55ce36d74937ff172accb9e32))

### Documentation

- Regenerate CHANGELOG for v0.14.1 [skip ci] ([`916fac4`](https://github.com/TyKolt/kremis/commit/916fac4b0522f320c4728546793695b207b4d618))

## [0.14.0] - 2026-03-06

### Documentation

- Update CHANGELOG — close [Unreleased] → [0.13.1] ([`50e7641`](https://github.com/TyKolt/kremis/commit/50e7641814d6507fa28885f2c9d3ee780ef61bf8))
- Regenerate CHANGELOG for v0.14.0 [skip ci] ([`c46d109`](https://github.com/TyKolt/kremis/commit/c46d109009076846b9ce09ac23acf03cf1b2cc01))

### Features

- Add POST /signals batch ingest endpoint — creates edges via HTTP (v0.14.0) ([`36abd70`](https://github.com/TyKolt/kremis/commit/36abd703b215fb67653e2f0503060a7cc7705052))

## [0.13.1] - 2026-03-04

### Bug Fixes

- Redirect tracing logs to stderr, suppress banner in --json-mode (v0.13.1) ([`8db6eb0`](https://github.com/TyKolt/kremis/commit/8db6eb05868cb75afbdc9f8823958173262fc2a7))

### Documentation

- Regenerate CHANGELOG for v0.13.0 ([`76e610c`](https://github.com/TyKolt/kremis/commit/76e610ca756a329a9233ee80b69031304320e9e1))
- Add missing env vars to installation and mcp/setup pages ([`118f612`](https://github.com/TyKolt/kremis/commit/118f6123d3d2d9dbf6cfcb2a7bc763b4f44098c4))
- Regenerate CHANGELOG for v0.13.1, fix hash.mdx backend docs ([`d7bc142`](https://github.com/TyKolt/kremis/commit/d7bc14292e47a35fb98e8f9a5b39b04e51c16152))

## [0.13.0] - 2026-03-03

### Dependencies

- Bump tempfile in the rust-minor-patch group (#6) ([`fe9ddf6`](https://github.com/TyKolt/kremis/commit/fe9ddf6805bfcb8af448d8ec28ef41d4c9aca4da))

### Documentation

- Regenerate CHANGELOG for v0.11.0, fix stale version in openapi.yml example ([`d873311`](https://github.com/TyKolt/kremis/commit/d87331185d2255be17b436bdac87ba2528954f64))
- Regenerate CHANGELOG for v0.12.0 ([`89a333e`](https://github.com/TyKolt/kremis/commit/89a333e4a8cd6413e3a4e332bd49e0c67cf603ba))

### Features

- Add AppConfig TOML-based configuration system, v0.12.0 ([`ff6e827`](https://github.com/TyKolt/kremis/commit/ff6e8272f377fdbd20f0692591dac1ad02d4c375))
- Add config provenance log (ConfigReport), v0.13.0 ([`ae18428`](https://github.com/TyKolt/kremis/commit/ae18428cad0462d06a14b9f879c65c4973500df5))

## [0.11.0] - 2026-03-01

### Documentation

- Regenerate CHANGELOG for v0.10.0 ([`c509522`](https://github.com/TyKolt/kremis/commit/c50952225c18636071a996220117b2017e2a3290))

### Features

- Add top_k to TraverseFiltered, v0.11.0 ([`37f083d`](https://github.com/TyKolt/kremis/commit/37f083dc110a7ce6e05e5fd0c196d9edc4eaa35b))

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
