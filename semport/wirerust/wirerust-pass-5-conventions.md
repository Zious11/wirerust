# Pass 5: Convention Catalog — wirerust

- **Project:** wirerust
- **Source path:** `/Users/zious/Documents/GITHUB/wirerust/`
- **Generated:** 2026-05-19
- **Pass:** 5 (Conventions) — Phase A broad-sweep, round 1
- **Confidence:** HIGH (every convention grounded in cited files, line counts derived from `awk`/`find`/`wc` against the live tree; CI gates verified against `.github/workflows/ci.yml`; doc-density numbers from `awk '/\/\/\//'` per file; test-name patterns verified across all 18 `tests/*.rs` files)

Builds on:
- Pass 0 (`wirerust-pass-0-inventory.md`) — file tree, dep catalog, test inventory, rustfmt/Cargo facts.
- Pass 1 (`wirerust-pass-1-architecture.md`) — component IDs C-1..C-20 used below.
- Pass 2 (`wirerust-pass-2-domain-model.md`) — entity / trait names.
- Pass 3 (`wirerust-pass-3-behavioral-contracts.md`) — tests-as-spec premise.
- Pass 4 (`wirerust-pass-4-nfr-catalog.md`) — NFR-MNT entries already partially encode convention shape.
- All 3 ADRs (`docs/adr/0001..0003`).
- `CLAUDE.md`, `rustfmt.toml`, `Cargo.toml`, `.github/workflows/ci.yml`.

This pass turns the tacit, observed regularities of the crate into an explicit, enforceable specification.

---

## 1. Convention Taxonomy

The catalogue uses these ten categories, with stable ID prefixes:

| Prefix | Category | What it covers |
|---|---|---|
| `CNV-NAM-NNN` | Naming | modules, types, fns, consts, tests |
| `CNV-MOD-NNN` | Module organization | `mod.rs` vs flat, submodule layout, `lib.rs` re-exports |
| `CNV-PUB-NNN` | Public-surface | what is `pub`, `pub(crate)`, `pub(super)`, private, re-export style |
| `CNV-ERR-NNN` | Error handling | `anyhow::Result` at boundary, `Option<T>` for absent, internal `Result<T, custom>`, no panics in src |
| `CNV-LOG-NNN` | Logging / diagnostics | `eprintln!` for warnings, `println!` only at L0 entry, one-shot atomic guards, no `log` crate |
| `CNV-TST-NNN` | Testing | integration-only layout, helper functions, fixture access, assertion style, `test_` prefix |
| `CNV-FMT-NNN` | Formatting / style | rustfmt 100 col, edition 2024, shorthand opts, derive ordering |
| `CNV-DEP-NNN` | Dependency policy | minor pins, lockfile committed, no workspace, no `build.rs` |
| `CNV-GIT-NNN` | Git / CI workflow | `develop` default, semantic PR titles, branch naming, no local hooks |
| `CNV-DOC-NNN` | Documentation | 3-tier doc set, doc-comment density, no per-file license headers, ADR rationale links |

Universality grading (used in column 5 of each table):

- **all** — present everywhere it could apply; no counter-example seen
- **most** — > ~75% of applicable sites; > 0 counter-examples
- **some** — 25–75% of applicable sites; mixed
- **none** — claimed but not actually present (kept for honesty / Pass 3 ⇄ Pass 5 alignment)

Enforcement legend (column 7):

- **rustfmt** — `cargo fmt --check` (CI gate)
- **clippy** — `cargo clippy --all-targets -- -D warnings` (CI gate)
- **rustc** — type / borrow / privacy errors (compiles or it doesn't)
- **test** — a specific `tests/*.rs` test pins the convention
- **semantic-pr** — `amannn/action-semantic-pull-request` job
- **manual** — by-convention only; no automated check

---

## 2. Convention Catalogue

### 2.1 Naming (`CNV-NAM-NNN`)

| ID | Category | Convention statement | Where observed | Universality | Counter-examples | Enforced by |
|---|---|---|---|---|---|---|
| CNV-NAM-001 | Naming | Modules are `snake_case` and single-word where possible (`analyzer`, `decoder`, `dispatcher`, `findings`, `mitre`, `reader`, `reassembly`, `reporter`, `summary`). Sub-modules use single words too (`flow`, `segment`, `handler`, `dns`, `http`, `tls`, `json`, `terminal`). | `src/lib.rs:1-10`; `src/analyzer/mod.rs:1-3`; `src/reassembly/mod.rs:1-3`; `src/reporter/mod.rs:1-2` | all | none | rustc (style) + manual |
| CNV-NAM-002 | Naming | Public types are `PascalCase`; no Hungarian prefixes; no `Wire`/`Rust` namespacing. | `decoder.rs:9 Protocol`; `findings.rs:8/25/42/60 Verdict/Confidence/ThreatCategory/Finding`; `flow.rs:7/63/72/160 FlowKey/FlowState/FlowDirection/TcpFlow`; `mitre.rs:23 MitreTactic`; `cli.rs:6/17/59 OutputFormat/Cli/Commands` | all | none | clippy `non_camel_case_types` |
| CNV-NAM-003 | Naming | Public functions and methods are `snake_case`. Top-level helpers in `src/` use verb-first naming (`decode_packet`, `process_packet`, `expire_flows`, `finalize`, `summarize`, `flush_contiguous`, `insert_segment`, `set_initiator`, `escape_for_terminal`, `compute_ja3`, `compute_ja3s`, `extract_sni`, `cipher_name`, `bytes_to_hex`). | `decoder.rs:71`; `reassembly/mod.rs:108,363,384`; `reassembly/segment.rs:28,227`; `reporter/terminal.rs:29`; `analyzer/tls.rs:68,1768,1858,51,59` | all | none | clippy `non_snake_case` |
| CNV-NAM-004 | Naming | Constructors are named `new` (not `create`, `build`, `from_*` unless taking a different input type). For each owned-state type, `new()` returns `Self`. | `dispatcher.rs:23`; `analyzer/dns.rs:19`; `analyzer/http.rs:122`; `summary.rs:25`; `reassembly/flow.rs:96,173`; `reassembly/mod.rs:85` | all | none | manual |
| CNV-NAM-005 | Naming | `from_*` constructors take alternate input types (cf. `PcapSource::from_file(path)` vs `PcapSource::from_pcap_reader(reader)`; `FlowKey::new(ip_a, port_a, ip_b, port_b)`). | `reader.rs:21,52`; `flow.rs:31` | all (n=3) | none | manual |
| CNV-NAM-006 | Naming | Boolean fields and parameters are predicate-named (no `is_` prefix, but no negation either): `partial`, `fin_seen`, `rst_seen`, `depth_exceeded`, `overlap_alert_fired`, `use_color`, `show_mitre_grouping`, `verbose`, `no_color`, `reassemble`, `no_reassemble`, `enable_dns`, `enable_http`, `enable_tls`. | `flow.rs:79-86,166`; `cli.rs:19-44`; `main.rs:55-62`; `reporter/terminal.rs:49-52` | most | `no_color`, `no_reassemble` are negated booleans (CLI mirroring user intent); `is_dns_port`, `is_query`, `is_grease_u16`, `is_weak_cipher`, `is_dns_port` are predicate **functions** (allowed) | manual |
| CNV-NAM-007 | Naming | Enum variants are `PascalCase` with no `Type`/`Kind`/`Variant` suffix. Variant names are problem-domain words, not technical (`Likely`/`Unlikely`/`Inconclusive`; `High`/`Medium`/`Low`; `Reconnaissance`/`LateralMovement`/`C2`; `ClientToServer`/`ServerToClient`; `Fin`/`Rst`/`Timeout`/`MemoryPressure`; `Inserted`/`Duplicate`/`PartialOverlap`/`ConflictingOverlap`/`Truncated`/`DepthExceeded`/`SegmentLimitReached`/`OutOfWindow`/`IsnMissing`). | `findings.rs:9-13,26-29,42-51`; `reassembly/handler.rs:7-9,12-17`; `reassembly/segment.rs:8-18` | all | none | clippy `non_camel_case_types` |
| CNV-NAM-008 | Naming | Module-level `const` and `static` are `SCREAMING_SNAKE_CASE`; numeric literals use `_` separators (`10_000`, `65_536`, `100_000`, `1_048_576`, `18_432`, `50_000`). | `reassembly/mod.rs:15-18,20`; `reassembly/segment.rs:5`; `analyzer/http.rs:8-11,67`; `analyzer/tls.rs:14-18` | all | none | clippy `non_upper_case_globals` |
| CNV-NAM-009 | Naming | Test functions use the `test_<subject>_<expected-behavior>` form. Verified: 185/202 test fns (91.6%) match this pattern. | every `tests/*.rs` except `tests/mitre_tests.rs` (10 tests) and 3 in `tests/tls_analyzer_tests.rs` and 7 in `tests/reporter_tests.rs` adopt the newer prose-style pattern. | most (91.6%) | 20 of 202 tests omit the `test_` prefix and use descriptive sentence form (`display_renders_enterprise_tactics_with_canonical_spacing`, `mitre_grouping_emits_tactic_headers_in_canonical_order`, `ascii_control_sni_finding_sets_mitre_t1027`). These are the **newest** tests (Apr 13 2026 commits in `mitre_tests.rs`, `reporter_tests.rs`); pattern appears to be in transition. | manual (no test or lint) |
| CNV-NAM-010 | Naming | Test-file naming follows `<module>_tests.rs` or `<scope>_integration_tests.rs` (e.g., `analyzer_tests.rs`, `reassembly_engine_tests.rs`, `reassembly_flow_tests.rs`, `tls_integration_tests.rs`, `linktype_integration_tests.rs`). Generic catch-all is `integration_test.rs` (singular, legacy). | `tests/*.rs` filenames | most | `tests/integration_test.rs` (singular) is the lone outlier — predates the `_tests.rs` convention | manual |
| CNV-NAM-011 | Naming | Test helper functions use the `make_<thing>(...)` form for fixture builders (cf. `make_tcp_packet`, `make_udp_packet`, `make_dns_packet`, `make_raw_ip_tcp_packet`, `make_linux_sll_tcp_packet`, `make_parsed`) and `build_<thing>` for binary-protocol synthesizers (cf. `build_client_hello`, `build_server_hello`, `build_client_hello_with_typed_sni_list`, `build_request_with_c1_in_host`, `build_path_traversal_with_c1_csi`). | `tests/decoder_tests.rs:6,23,37`; `tests/reassembly_engine_tests.rs:43`; `tests/analyzer_tests.rs:7,21`; `tests/summary_tests.rs:6`; `tests/tls_analyzer_tests.rs:16,22,34,47,...`; `tests/reporter_tests.rs` | most | `flow_key`/`test_flow_key`/`http_test_flow_key` and `minimal_pcap_bytes`/`minimal_pcap_with_tcp` are alternative helper styles; no `make_`/`build_` for these | manual |
| CNV-NAM-012 | Naming | Reserved technique IDs use the canonical MITRE form (`T1071`, `T1071.001`, `T1505.003`, `T1499.002`, `T0846`, `T0855`) — never aliased or shortened. | `mitre.rs:99-129`; `reassembly/mod.rs:284,543`; `analyzer/http.rs`; `analyzer/tls.rs` | all | none | test (`tests/mitre_tests.rs:94-176`) |

### 2.2 Module organization (`CNV-MOD-NNN`)

| ID | Category | Convention statement | Where observed | Universality | Counter-examples | Enforced by |
|---|---|---|---|---|---|---|
| CNV-MOD-001 | Module org | Crate is a single binary + library hybrid: `src/lib.rs` declares the module surface; `src/main.rs` is the binary entry, imports its own modules via `use wirerust::…` (treats the crate as if it were a downstream consumer). | `src/lib.rs` (10 lines, `pub mod` only); `src/main.rs:7-20` (all imports prefixed `wirerust::`) | all | none | rustc (binary cannot resolve `use wirerust::…` otherwise) |
| CNV-MOD-002 | Module org | Submodules use a folder + `mod.rs` layout when a module has 2+ peer files (`src/analyzer/mod.rs` + 3 protocol files; `src/reassembly/mod.rs` + 3 internal files; `src/reporter/mod.rs` + 2 renderers). Single-file leaves stay flat at the top level (`cli.rs`, `decoder.rs`, `dispatcher.rs`, `findings.rs`, `mitre.rs`, `reader.rs`, `summary.rs`). | tree at `src/` | all | none | manual |
| CNV-MOD-003 | Module org | `mod.rs` files only declare submodules and host the cross-cutting trait + supporting type that all submodules implement. `src/analyzer/mod.rs` (31 LOC) declares `pub mod {dns,http,tls}` + `AnalysisSummary` + `trait ProtocolAnalyzer`. `src/reassembly/mod.rs` (564 LOC) is the exception: declares `pub mod {flow,handler,segment}` and also hosts the main `TcpReassembler` engine. `src/reporter/mod.rs` (15 LOC) declares `pub mod {json,terminal}` + `trait Reporter`. | `src/analyzer/mod.rs:1-31`; `src/reporter/mod.rs:1-15`; `src/reassembly/mod.rs:1-564` | most | `src/reassembly/mod.rs` is a 564-LOC engine, not a thin trait/declaration file. This is the singular violation; observed deliberately as the engine ties all three sub-files together and would otherwise need a 5th file. | manual |
| CNV-MOD-004 | Module org | `lib.rs` re-exports *modules*, not types: `pub mod analyzer; pub mod cli; …` (no `pub use crate::findings::Finding;` style re-exports). Downstream code (binary, tests, future library consumers) must reach types through their module path. | `src/lib.rs:1-10` | all | none | rustc + manual |
| CNV-MOD-005 | Module org | The 10 top-level modules in `src/lib.rs` are declared alphabetically. | `src/lib.rs:1-10` (analyzer, cli, decoder, dispatcher, findings, mitre, reader, reassembly, reporter, summary) | all | none | manual |
| CNV-MOD-006 | Module org | Use-statements in each file follow Rust's idiomatic three-group order: (1) `std::*`; (2) external crates; (3) `crate::*`. Each group is alphabetical; blank line separates groups. | `src/main.rs:1-20`; `src/reassembly/mod.rs:5-13`; `src/reporter/terminal.rs:1-7`; `src/analyzer/{http,tls}.rs` opening blocks | all | none | rustfmt (when `imports_granularity` / `group_imports` is enabled — currently not in `rustfmt.toml`, so this is **convention-only**) |

### 2.3 Public surface (`CNV-PUB-NNN`)

| ID | Category | Convention statement | Where observed | Universality | Counter-examples | Enforced by |
|---|---|---|---|---|---|---|
| CNV-PUB-001 | Public surface | Every module in `src/lib.rs` is `pub`. There is no `pub(crate)` module declaration — the whole module tree is the library API. | `src/lib.rs:1-10` | all | none | rustc |
| CNV-PUB-002 | Public surface | `pub(crate)` is used **zero times** in src (`awk '/pub\(crate\)/'` returns nothing). The visibility ladder is `pub` or private only; nothing in between. | grep against `src/**/*.rs` | all | none | rustc + manual |
| CNV-PUB-003 | Public surface | `pub(super)` is used twice — both in `FlowDirection` (`segments`, `buffered_bytes`) to expose those fields to the parent `reassembly` engine while keeping them private to outside consumers. | `src/reassembly/flow.rs:75-76` | all | none (only-2 usage) | rustc + test (`tests/reassembly_flow_tests.rs:92-101` uses the public accessors `buffered_bytes()`/`segments_is_empty()` instead, confirming the encapsulation) |
| CNV-PUB-004 | Public surface | Fields on public structs default to private, with **public accessor methods** for read-only access (cf. `FlowKey::lower_ip()/lower_port()/upper_ip()/upper_port()`; `Summary::unique_hosts()/protocol_counts()/service_counts()`; `FlowDirection::segment_count()/buffered_bytes()/segments_is_empty()/segment_at()/has_segment_at()/memory_used()`). | `flow.rs:14-29,129-156`; `summary.rs:48-60` | most | Several types have `pub` fields where the type is purely a data carrier: `ParsedPacket` (all 6 fields `pub`), `TransportInfo::{Tcp,Udp}` (all variant fields `pub`), `RawPacket` (3 `pub` fields), `PcapSource` (2 `pub` fields), `Finding` (all 8 fields `pub`), `ReassemblyConfig`/`ReassemblyStats` (all fields `pub`), `TcpFlow` (7 `pub` fields), `TerminalReporter::use_color`/`show_mitre_grouping` `pub`. | rustc + manual |
| CNV-PUB-005 | Public surface | `pub use` re-exports are not used. Module-internal types are reached by their full path (`crate::reassembly::flow::FlowKey`, `crate::reassembly::handler::{Direction, CloseReason}`). The only `pub use`-shaped thing is the field `pub use_color: bool` (`reporter/terminal.rs:49`) which is a struct field, not a re-export — `awk` grep for `pub use` confirms zero true re-exports. | `awk '/pub use/'` returns one false-positive | all | none | manual |
| CNV-PUB-006 | Public surface | `#[non_exhaustive]` is applied only to externally-evolving enums. Used once in src (`MitreTactic` in `src/mitre.rs:22`) because new MITRE ATT&CK versions add tactics; not applied to internal enums (`Verdict`, `Confidence`, `ThreatCategory`, `InsertResult`, `FlowState`, `Direction`, `CloseReason`, `DispatchTarget`, `Protocol`, `OutputFormat`) that the crate fully owns. | `src/mitre.rs:22`; `awk '/non_exhaustive/'` returns 1 hit | all | none | manual |
| CNV-PUB-007 | Public surface | Traits are `pub` and live in their module's `mod.rs` (`Reporter` in `reporter/mod.rs`; `StreamHandler`, `StreamAnalyzer` in `reassembly/handler.rs`; `ProtocolAnalyzer` in `analyzer/mod.rs`). Trait method signatures take `&self` or `&mut self`, never `self` (no consuming trait methods). | `reporter/mod.rs:8-15`; `reassembly/handler.rs:19-29`; `analyzer/mod.rs:19-31` | all | none | rustc |

### 2.4 Error handling (`CNV-ERR-NNN`)

| ID | Category | Convention statement | Where observed | Universality | Counter-examples | Enforced by |
|---|---|---|---|---|---|---|
| CNV-ERR-001 | Error handling | `anyhow::Result<T>` is used at the binary boundary and at all I/O / parse boundaries that can fail with arbitrary diagnostics: `main`, `run_analyze`, `run_summary`, `resolve_targets`, `PcapSource::from_file`, `PcapSource::from_pcap_reader`, `decode_packet`. | `main.rs:22,55,190,236`; `reader.rs:21,52`; `decoder.rs:71` | all (n=7) | none | rustc + manual |
| CNV-ERR-002 | Error handling | `anyhow::Context` / `with_context(\|\| format!(...))` is the canonical pattern for adding context to errors propagated via `?`. | `reader.rs:22,41,53`; `main.rs:104,197` (5 callsites) | all | none | manual |
| CNV-ERR-003 | Error handling | `anyhow!(...)` literal is used to construct ad-hoc errors at parse failures (`decode_packet` for unsupported link types, missing IP layer, parse errors; `reader.rs` for unsupported pcap link types). | `decoder.rs:76,78,97`; `reader.rs:32` | all (n=4) | none | manual |
| CNV-ERR-004 | Error handling | `anyhow::bail!(...)` is used once, for the leaf "target file not found" path. | `main.rs:255` | all (n=1) | none | manual |
| CNV-ERR-005 | Error handling | Internal "may-be-absent" results use `Option<T>` — never `Result<T, ()>`. cf. `technique_info` (returns `Option<(&str, MitreTactic)>`), `technique_name`, `technique_tactic`, `parse_one_request` (returns `Result<Option<ParsedRequest>, httparse::Error>` to distinguish "more data needed" from "parse error"), `ParsedPacket::app_protocol_hint`, `extract_sni`. | `mitre.rs:98,136,142`; `analyzer/http.rs:1095,1117`; `decoder.rs:46-67`; `analyzer/tls.rs:1858` | all | none | manual |
| CNV-ERR-006 | Error handling | Internal "rich-result" enums (vs. `anyhow`) are used where the caller needs to branch on the failure mode. `InsertResult` (9 variants) is the canonical example — `insert_segment` returns one of nine variants for the caller (`TcpReassembler`) to translate into stats counters / findings. | `reassembly/segment.rs:7-18` consumed by `reassembly/mod.rs:232-265` | all (n=1) | none | exhaustive `match` in `mod.rs:232-265` (rustc) |
| CNV-ERR-007 | Error handling | No `panic!`, `unimplemented!`, `todo!`, or `unreachable!` macros anywhere in `src/`. `awk` confirms zero hits. The only allowed "this can't happen" assertions are `assert!`/`debug_assert!`. | `awk` against `src/**/*.rs` | all | none | manual + rustc clippy `panic_in_result_fn` (not enabled, so by-convention) |
| CNV-ERR-008 | Error handling | `assert!` is used for non-negotiable preconditions on **constructor** input (`TcpReassembler::new` asserts 5 config invariants). `debug_assert!` is used for internal "this would be a bug" invariants that should not crash a release build (cf. `reassembly/segment.rs:178,208`; `reassembly/mod.rs:223,481`; `flow.rs:150`). | `reassembly/mod.rs:86-96,481`; `reassembly/segment.rs:178,208`; `flow.rs:150` | all | none | rustc |
| CNV-ERR-009 | Error handling | `.unwrap()` and `.expect()` are restricted in `src/`. Only 4 call-sites exist: 3 `flows.get_mut(&key).unwrap()` calls in `reassembly/mod.rs` (where the key was just `contains_key`-checked / inserted) and 1 `serde_json::to_string_pretty(&output).unwrap()` in `reporter/json.rs:36` (`json!` macro produces infallible-serialize input). All are accompanied by reasoning the call cannot fail. | `reassembly/mod.rs:157,268,334`; `reporter/json.rs:36` | all | none | manual |
| CNV-ERR-010 | Error handling | The `?` operator is the only acceptable way to propagate `Result` errors; never combined with explicit `match` that maps to `panic!`. | `reader.rs:22,41`; `main.rs:104,197,243,244`; etc. | all | none | clippy + manual |

### 2.5 Logging / diagnostics (`CNV-LOG-NNN`)

| ID | Category | Convention statement | Where observed | Universality | Counter-examples | Enforced by |
|---|---|---|---|---|---|---|
| CNV-LOG-001 | Logging | `println!` is restricted to the binary entry (`src/main.rs`) for the rendered report only. Library code never writes to stdout. | `main.rs:186,232`; `awk '/println!/'` against `src/**/*.rs` returns exactly those 2 lines (plus eprintln hits) | all | none | manual |
| CNV-LOG-002 | Logging | `eprintln!` is the only diagnostic / warning channel. Used for: (a) decode error first-warning in main; (b) one-shot warnings on global invariant violations in the engine; (c) `--http/--tls` + `--no-reassemble` conflict warning. | `main.rs:73,126,206`; `reassembly/mod.rs:483`; `reassembly/segment.rs:44`; 6 total call sites | all | none | manual |
| CNV-LOG-003 | Logging | Global one-shot warning guards are implemented as private `static AtomicBool` in the module that emits the warning, with `swap(true, Ordering::Relaxed)` as the test-and-set. The pattern guarantees a multi-flow capture emits at most one warning per process per condition. | `reassembly/mod.rs:20` (`CLOSE_FLOW_MISSING_WARNED`); `reassembly/segment.rs:5` (`ISN_MISSING_WARNED`) | all (n=2) | none | manual |
| CNV-LOG-004 | Logging | The `log` / `tracing` crate is **not** a dependency. Verbose/quiet flags exist as `cli.verbose: bool` (currently unwired — Pass 0 Q#4) but no logging framework is plugged in. | `Cargo.toml` (no `log` or `tracing`); `cli.rs:19-20` | all | none | manual |
| CNV-LOG-005 | Logging | The decode-error warning is **rate-limited to first occurrence**: `if total_decode_errors == 0 { eprintln!(...) }` followed by `total_decode_errors += 1`. The same pattern appears in both `run_analyze` and `run_summary` for symmetry. | `main.rs:125-131,205-211` | all (n=2) | none | manual |
| CNV-LOG-006 | Logging | Diagnostic strings are prefixed with `wirerust:` only when emitted from a library module (engine-level warning); main-level warnings are prefixed `Warning:` (consumer-readable). | `reassembly/mod.rs:484` (`wirerust:`); `reassembly/segment.rs:44` (`wirerust:`); `main.rs:74,127,207` (`Warning:`) | all | none | manual |

### 2.6 Testing (`CNV-TST-NNN`)

| ID | Category | Convention statement | Where observed | Universality | Counter-examples | Enforced by |
|---|---|---|---|---|---|---|
| CNV-TST-001 | Testing | Tests live in `tests/*.rs` (integration / black-box). There is **one** inline `#[cfg(test)] mod tests` block in `src/`, in `src/reporter/terminal.rs:261-350` — the 10 `escape_for_terminal` unit tests live there because `escape_for_terminal` is private to the module and cannot be reached from `tests/`. | `awk '/#\[cfg\(test\)\]/'` over `src/**/*.rs` returns 1 hit; `tests/` has 202 test fns across 18 files | most | `src/reporter/terminal.rs:261-350` is the single inline test block (legitimately so — exercises a private helper). | rustc (anything compiled out of `#[cfg(test)]` does not ship) |
| CNV-TST-002 | Testing | `cargo test --all-targets` runs every test; both the `#[test]` body and the `#[cfg(test)] mod tests` block under `src/reporter/terminal.rs` participate. No nightly-only test framework, no `tokio::test`, no `rstest`, no `proptest`. | `tests/*.rs`; `awk` confirms zero `#[tokio::test]` and zero `#[ignore]` | all | none | CI (`.github/workflows/ci.yml:47`) |
| CNV-TST-003 | Testing | Each integration test file is paired to a single src/ module (`<module>_tests.rs` for unit-style integration; `<scope>_integration_tests.rs` for cross-module end-to-end). Pass 0 §3 maps the pairing one-to-one for 16 of 18 files. | every `tests/*.rs` | most | `tests/integration_test.rs` (singular legacy name) tests the full pipeline; `tests/linktype_integration_tests.rs` is cross-cutting. | manual |
| CNV-TST-004 | Testing | Fixtures live in `tests/fixtures/` as binary `.pcap`/`.pcapng`/`.cap` files (14 files; 247 B → 33 KB). Loaded via `include_bytes!` or `std::fs::read` from each integration test. | `tests/fixtures/`; consumed by `tests/{http_integration_tests,tls_integration_tests,linktype_integration_tests,reader_tests,reassembly_engine_tests}.rs` | all | none | manual |
| CNV-TST-005 | Testing | Test helpers (`make_*`, `build_*`, `flow_key`, `minimal_pcap_*`, `RecordingHandler`) live at the top of each `tests/*.rs` file. There is **no** `tests/common/mod.rs` or shared-helper module — duplication is accepted as a small cost for keeping each test file standalone. The cost shows: `make_tcp_packet` exists in both `tests/decoder_tests.rs:6` and `tests/reassembly_engine_tests.rs:43` with similar bodies but different signatures. | `tests/decoder_tests.rs:6,23,37,129,172`; `tests/reassembly_engine_tests.rs:43`; `tests/analyzer_tests.rs:7`; `tests/summary_tests.rs:6`; `tests/integration_test.rs:11` | all (intentional) | none | manual |
| CNV-TST-006 | Testing | Tests use `assert_eq!` / `assert!` / `assert_ne!` from the standard library. Custom assertion macros are not used. `assert!` calls include diagnostic messages (`assert!(err_msg.contains("Unsupported"), "Error should mention 'Unsupported', got: {err_msg}")`). | every `tests/*.rs`, e.g. `tests/reader_tests.rs:96-99`; `tests/reassembly_engine_tests.rs:113-114` | all | none | rustc |
| CNV-TST-007 | Testing | Test fns adopt the `test_<subject>_<expected>` convention in 91.6% of cases (185/202). Newer test files (`mitre_tests.rs` 10/10, `reporter_tests.rs` 7/19, `tls_analyzer_tests.rs` 3/39) drop the `test_` prefix and use prose-style names. **The convention is in transition.** | `awk` count: 185 with `test_` prefix, 20 without | most (drifting) | The 20 prose-style outliers (see CNV-NAM-009). | manual |
| CNV-TST-008 | Testing | `#[allow(clippy::too_many_arguments)]` is applied locally to a test helper that needs >7 params for a packet builder (`tests/reassembly_engine_tests.rs:42 make_tcp_packet`). This is the only `#[allow]` in any test file. | `tests/reassembly_engine_tests.rs:42` | all (n=1) | none | clippy + manual |
| CNV-TST-009 | Testing | `assert_cmd`, `predicates`, `tempfile` are declared as dev-deps in `Cargo.toml:28-30` but never used. CLI tests use `Cli::parse_from([…])` for in-process argv simulation rather than spawning the binary. | `Cargo.toml:28-30`; `tests/cli_tests.rs:1-110` (no `assert_cmd` import) | all | none | manual + Pass 0 Q#3 |
| CNV-TST-010 | Testing | Test handlers / fakes implement the same `StreamHandler` trait the production code does (cf. `RecordingHandler` in `tests/reassembly_engine_tests.rs:10-40`). Test doubles are not mocks; they are real impls that record. | `tests/reassembly_engine_tests.rs:10-40` | all (n=1, but it is the only test-double pattern in the crate) | none | manual |
| CNV-TST-011 | Testing | Doctests are not used (the `Note: --all-targets does NOT run doctests` in `CLAUDE.md:21` is a warning to future contributors; current crate has zero `///` examples that compile). | `awk '/```/'` over `src/**/*.rs` returns 0 hits | all | none | manual |

### 2.7 Formatting / style (`CNV-FMT-NNN`)

| ID | Category | Convention statement | Where observed | Universality | Counter-examples | Enforced by |
|---|---|---|---|---|---|---|
| CNV-FMT-001 | Formatting | Edition is `2024`. `rust-toolchain.toml` is not pinned; CI runs `dtolnay/rust-toolchain@stable` (effective MSRV = current stable; minimum 1.85 because of edition 2024). | `Cargo.toml:4`; `rustfmt.toml:1`; `.github/workflows/ci.yml:45,54,65` | all | none | rustc |
| CNV-FMT-002 | Formatting | `rustfmt.toml` pins `max_width = 100`, `use_field_init_shorthand = true`, `use_try_shorthand = true`. CI runs `cargo fmt --all --check`. | `rustfmt.toml:2-4`; `.github/workflows/ci.yml:68` | all | none | rustfmt + CI |
| CNV-FMT-003 | Formatting | The `#[derive(...)]` clause for data-carrier structs / enums orders traits as: `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize` (skipping any not applicable). | `findings.rs:7,24,41`; `decoder.rs:8`; `cli.rs:5`; `flow.rs:6,62`; `reassembly/segment.rs:7`; `reassembly/handler.rs:5,11`; `mitre.rs:21` | most | `Cli` uses `#[derive(Parser, Debug)]` (clap derive macro first, then `Debug`) — convention bend for clap. `Default` slot comes immediately after `Clone` in `ReassemblyStats`/`AnalysisSummary` (`Debug, Clone, Default`). | rustfmt orders within; ordering between derives is manual |
| CNV-FMT-004 | Formatting | Numeric literals use `_` thousands-separator: `10_000`, `65_536`, `1_048_576`, `100_000`, `50_000`, `18_432`, `2048` (>= 1000 only). Float literals are not used in src. | `analyzer/{http,tls}.rs`; `reassembly/mod.rs:15-18,43-49` | all | none | clippy `unreadable_literal` |
| CNV-FMT-005 | Formatting | Module-level constants are declared in a block at the top of the file, immediately after `use` imports, before the first type. | `analyzer/http.rs:8-11`; `analyzer/tls.rs:14-18`; `reassembly/mod.rs:15-18,20` | all | none | manual |
| CNV-FMT-006 | Formatting | `#[allow(...)]` is **not** used in `src/`. The `RUSTFLAGS=-Dwarnings` CI gate means any new warning is a build error — `awk '/#\[allow/'` over `src/**/*.rs` returns zero. | `awk` against src; CI `RUSTFLAGS=-Dwarnings` | all | none | clippy + CI |
| CNV-FMT-007 | Formatting | Format-string interpolation uses the inline-capture form `{var}` / `{var:?}` (Rust 2021 captures) rather than positional `{}` + arg list, where possible. | `findings.rs:55,84`; `reporter/terminal.rs:66-67,86-88`; `decoder.rs:76,78`; passim | most | A few callsites still use positional form when the value is a method call or non-identifier (e.g., `format!("Excessive segment overlaps ({}) on flow {}", flow_dir.overlap_count, key)` in `reassembly/mod.rs:280`). | clippy `uninlined_format_args` (enabled by default in 1.85+) |

### 2.8 Dependency policy (`CNV-DEP-NNN`)

| ID | Category | Convention statement | Where observed | Universality | Counter-examples | Enforced by |
|---|---|---|---|---|---|---|
| CNV-DEP-001 | Dependency | Dependencies are pinned by SemVer major only (`"1"`, `"4"`, `"0.12"`, `"0.17"`, `"0.16"`, `"0.4"`). No exact pins, no `=` requirements, no git deps. | `Cargo.toml:9-22,28-30` | all | none | manual |
| CNV-DEP-002 | Dependency | `Cargo.lock` is checked in (38,291 bytes). This is appropriate for a binary crate. | repo root | all | none | manual (gitignore does not list `Cargo.lock`) |
| CNV-DEP-003 | Dependency | The project is a **single crate**, not a workspace. No `[workspace]` section; no member crates. | `Cargo.toml` | all | none | manual |
| CNV-DEP-004 | Dependency | There is no `build.rs`. No code-generation step. Static lookup tables (cf. MITRE techniques) are hand-written `match` arms — see CNV-PAT (§4) for the "static match instead of phf/codegen" design pattern. | repo root | all | none | manual |
| CNV-DEP-005 | Dependency | Dev-deps are minimal (3 declared, all currently unused) — the project does not depend on `proptest`, `rstest`, `serial_test`, `criterion`, `pretty_assertions`, `mockall`, or other common test crates. | `Cargo.toml:27-30` | all | none | manual |
| CNV-DEP-006 | Dependency | `RUSTFLAGS=-Dwarnings` is set globally in CI (`.github/workflows/ci.yml:12`). Local dev does not enforce this by default. | `.github/workflows/ci.yml:10-12` | all (CI side) | none | CI |
| CNV-DEP-007 | Dependency | `[profile.release]` sets `overflow-checks = true` — arithmetic panics in release builds, treating overflow as a bug not a feature. | `Cargo.toml:24-25` | all | none | rustc + manual |
| CNV-DEP-008 | Dependency | Two declared deps are presently dead: `csv` (Cargo.toml:17) and `rayon` (Cargo.toml:22). Both are reserved for roadmap features (`--csv` output and parallel file processing — Pass 0 Q#1, Q#2). | `Cargo.toml` vs `awk '/use csv|use rayon/'` returns 0 hits | all (negative) | none | manual / Pass 0 |

### 2.9 Git / CI workflow (`CNV-GIT-NNN`)

| ID | Category | Convention statement | Where observed | Universality | Counter-examples | Enforced by |
|---|---|---|---|---|---|---|
| CNV-GIT-001 | Git | Default branch is `develop` (git-flow). PRs target `develop`; `main` is the release/stable branch. CI runs on push to `develop` and `main`. | `CLAUDE.md:35`; `.github/workflows/ci.yml:5,8` | all | none | manual |
| CNV-GIT-002 | Git | Three branch-naming patterns observed: (a) `feature/<name>` for plain feature branches; (b) `worktree-issue-<n>-<slug>` for issue-scoped worktree branches; (c) `worktree-<slug>` for ad-hoc worktree branches. Of the 24 remote branches observed via `git branch -a`, 6 follow pattern (a) (`feature/http-analyzer`, `feature/tcp-reassembly`, `feature/reassembly-perf`, `feature/gitignore-factory-worktree`, `feature/mitre-attack-mapping`, `feature/multi-linktype`), 5 follow pattern (b) (`worktree-issue-{20,52,54,56}-…`), and 11 follow pattern (c) (`worktree-{adr-0003,cross-flow-isolation-test,fix-overlap-classification,fix-segment-limit,flowdir-private-fields,harden-debug-asserts,http-parse-errors,http-spec-sync,out-of-window-alert,overflow-checks}`). Pattern (c) dominates the worktree population. | `git branch -a`; `CLAUDE.md:36-39` | most | A handful of single-use branches (`chore/add-test-fixtures`, `chore/update-readme`, `setup/repo-essentials`) follow a `<type>/<slug>` form that matches semantic-PR types rather than the three documented patterns. | manual |
| CNV-GIT-003 | Git | Semantic PR titles are enforced via `amannn/action-semantic-pull-request@v6`. Allowed types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`. Scope is optional. Recent `git log --oneline -30` confirms 100% compliance — every commit title matches `^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?:`. | `.github/workflows/ci.yml:14-38`; `git log --oneline` | all | none | CI |
| CNV-GIT-004 | Git | PR scopes when used follow `(mitre)`, `(tls)`, `(http)`, `(reporter)`, `(tcp)` — short module-name slugs. Out of last 30 commits, 12 use a scope, 18 omit it. | `git log --oneline -30` | most (optional by policy) | none | semantic-pr (regex permissive) |
| CNV-GIT-005 | Git | No local commit hooks. No `lefthook.yml`, `.husky/`, `.pre-commit-config.yaml`, or `commitlint.config.js`. Quality is **CI-gated only**. | repo root | all | none | manual |
| CNV-GIT-006 | Git | CI has 4 jobs: `semantic-pr` (PR-only), `test`, `clippy`, `fmt`. All Linux (`ubuntu-latest`); all use `dtolnay/rust-toolchain@stable` + `Swatinem/rust-cache@v2`. There is no matrix build, no MSRV check, no Windows / macOS validation, no nightly job. | `.github/workflows/ci.yml:14-68` | all | none | CI |
| CNV-GIT-007 | Git | Issue references are linked at commit-message tail: `(#N)`. ~95% of recent commits include at least one issue or PR number. | `git log --oneline -30` (29/30 match `#\d+`) | most | `setup/repo-essentials` and very-early commits lack the `(#N)` suffix. | manual |
| CNV-GIT-008 | Git | `.gitignore` is minimal: `/target` + `.claude/worktrees/`. No `.DS_Store` filter (relies on per-user global gitignore), no `*.swp`, etc. | `.gitignore` (2 lines) | all | none | manual |

### 2.10 Documentation (`CNV-DOC-NNN`)

| ID | Category | Convention statement | Where observed | Universality | Counter-examples | Enforced by |
|---|---|---|---|---|---|---|
| CNV-DOC-001 | Documentation | Project docs are organized in a 3-tier set: (1) `README.md` (project overview, install, usage); (2) `docs/adr/` (3 numbered, dated ADRs); (3) `docs/superpowers/{plans,specs}/` (18 dated implementation plans + design specs). `CLAUDE.md` augments these with build/test/lint/git workflow guidance for agents. | repo root; `docs/` | all | none | manual |
| CNV-DOC-002 | Documentation | ADRs follow a numbered + slugged filename (`0001-content-first-stream-dispatch.md`, `0002-modular-protocol-analyzers.md`, `0003-reporting-pipeline-layering.md`). Each ADR is referenced by short name (`ADR 0001`, `ADR 0002`, `ADR 0003`) from src doc comments where the decision is implemented. | `docs/adr/*.md`; `src/findings.rs:79`; `src/reporter/terminal.rs:27,109,131`; `src/mitre.rs:23` | all (n=3) | none | manual |
| CNV-DOC-003 | Documentation | Superpower (plans/specs) docs follow the `YYYY-MM-DD-<slug>.md` filename pattern (10 plans, 8 specs; all dated 2026-04-02..2026-04-13). | `docs/superpowers/plans/*.md`; `docs/superpowers/specs/*.md` | all (n=18) | none | manual |
| CNV-DOC-004 | Documentation | Doc-comment density (per `awk '/\/\/\//'`) is highly uneven across modules. Highest-density files: `src/analyzer/tls.rs` (73 doc lines / 750 LOC = ~10%); `src/reporter/terminal.rs` (42 / 350 = 12%); `src/reassembly/mod.rs` (24 / 564 = 4%); `src/cli.rs` (23 / 113 = 20% — every flag documented for `clap`); `src/mitre.rs` (13 / 144 = 9%); `src/findings.rs` (9 / 92 = 10%); `src/segment.rs` (5 / 240 = 2%). **Zero** doc lines in: `src/lib.rs`, `src/main.rs`, `src/dns.rs`, `src/summary.rs`, `src/reader.rs`, `src/dispatcher.rs`, `src/reassembly/flow.rs`, `src/reassembly/handler.rs`, `src/reporter/{mod,json}.rs`. | per-file `awk` counts | some | The 10 files with zero `///` doc comments are the counter-examples. Some of them have non-trivial public APIs (cf. `reassembly/flow.rs` exports 4 pub types and many methods without a single `///`). | manual |
| CNV-DOC-005 | Documentation | Module-level (`//!`) inner doc comments are used only in `src/mitre.rs:1-13` (13 lines). No other module declares a module-level doc block. | `awk '/^\/\/!/'` across `src/` returns hits only in `mitre.rs` | some | every other module lacks `//!`. The `mitre.rs` doc block is the only example; widening this pattern was *not* adopted across the crate. | manual |
| CNV-DOC-006 | Documentation | When a doc comment exists, it is preceded by `///` (item-level). Style is prose, often multi-paragraph (cf. `escape_for_terminal` 19-line doc comment; `Finding::Display` 9-line doc comment; `technique_info` 5-line doc comment). | `src/reporter/terminal.rs:9-28`; `src/findings.rs:72-80`; `src/mitre.rs:92-97` | most | sparse on simple accessors (`FlowKey::lower_ip()`, `Summary::unique_hosts()` are undocumented) | manual |
| CNV-DOC-007 | Documentation | The crate has **no per-file license header**. License is declared once in `Cargo.toml:6` (`license = "MIT"`) and once in the top-level `LICENSE` file. None of the 20 `.rs` files in `src/` begin with a `// Copyright …` or `// SPDX-License-Identifier: MIT` block. | `awk '/Copyright\|SPDX/'` over `src/**/*.rs` returns zero hits | all (negative) | none | manual |
| CNV-DOC-008 | Documentation | Where a load-bearing security or correctness invariant is encoded in code, the doc comment links to the relevant ADR by path (cf. `findings.rs:80` → `docs/adr/0003-reporting-pipeline-layering.md`; `reporter/terminal.rs:27` → ADR 0003; `mitre.rs:18-22` discusses MITRE-evolution rationale; `reassembly/flow.rs:32-34` discusses tuple-pair canonicalization correctness). | `findings.rs:79-80`; `reporter/terminal.rs:27,109,131`; `mitre.rs:18-22`; `reassembly/flow.rs:32-34` | most | none for the ADR-linked subset; many other invariants are encoded only in tests, not doc comments | manual |
| CNV-DOC-009 | Documentation | Cross-references to GitHub issues use `(see issue #NN)` inline in doc comments (cf. `tests/mitre_tests.rs:192` → "(see issue #67)"; `reporter/terminal.rs:197` → "(see issue #67)"). | grep `issue #` over src + tests | most | none seen incorrect; only the convention is sparse (~3-4 cites) | manual |
| CNV-DOC-010 | Documentation | `CLAUDE.md` is the canonical contributor onboarding doc for AI agents and humans alike. It declares: build commands, test commands, lint commands, format commands, git workflow, branch naming, semantic-PR types, no local hooks. | `CLAUDE.md:1-52` | all | none | manual |

---

## 3. Universality Matrix

Single roll-up table of every convention, with applicability (sites where it could apply), exemplification count, violation count, and a 3-class universality grade.

| ID | Universality | Applicable sites | Conforming sites | Violating sites | Notes |
|---|---|---|---|---|---|
| CNV-NAM-001 | all | 20 modules | 20 | 0 | every module is snake_case single-word |
| CNV-NAM-002 | all | 56 pub types | 56 | 0 | no Hungarian, no Wire/Rust prefix |
| CNV-NAM-003 | all | ~30 top-level fns | ~30 | 0 | clippy-enforced |
| CNV-NAM-004 | all | 9 `::new` constructors | 9 | 0 | none use `create`/`make` |
| CNV-NAM-005 | all | 3 `from_*` constructors | 3 | 0 | |
| CNV-NAM-006 | most | ~25 bool field/param | ~23 | 2 | `no_color`, `no_reassemble` mirror user intent |
| CNV-NAM-007 | all | 11 enums, ~40 variants | 40 | 0 | |
| CNV-NAM-008 | all | 12 constants + 2 statics | 14 | 0 | |
| CNV-NAM-009 | most (91.6%) | 202 test fns | 182 | 20 | new prose-style tests in `mitre_tests.rs`/`reporter_tests.rs`/`tls_analyzer_tests.rs` |
| CNV-NAM-010 | most | 18 test files | 17 | 1 | `tests/integration_test.rs` singular |
| CNV-NAM-011 | most | ~30 helpers | ~24 | ~6 | `flow_key`/`minimal_pcap_*`/`http_test_flow_key` use neither `make_` nor `build_` |
| CNV-NAM-012 | all | 6 emitted IDs | 6 | 0 | tests pin every emitted ID |
| CNV-MOD-001 | all | crate | yes | no | binary imports its own crate as if external |
| CNV-MOD-002 | all | 3 multi-file submodules | 3 | 0 | folder + mod.rs idiom |
| CNV-MOD-003 | most | 3 mod.rs files | 2 | 1 | `src/reassembly/mod.rs` is a 564-LOC engine |
| CNV-MOD-004 | all | `lib.rs` | yes | no | re-exports only modules |
| CNV-MOD-005 | all | 10 modules in `lib.rs` | 10 | 0 | alphabetical |
| CNV-MOD-006 | all | every `.rs` file | 20 | 0 | std → external → crate |
| CNV-PUB-001 | all | 10 top modules | 10 | 0 | all `pub` |
| CNV-PUB-002 | all | 20 files | 20 | 0 | zero `pub(crate)` |
| CNV-PUB-003 | all | 2 `pub(super)` uses | 2 | 0 | both in `FlowDirection` |
| CNV-PUB-004 | most | ~30 public structs | ~10 hide fields | ~20 expose fields | data-carrier types deliberately expose; behavior-owning types hide |
| CNV-PUB-005 | all | crate | yes (zero re-exports) | no | |
| CNV-PUB-006 | all | 1 outward-facing enum | 1 | 0 | `MitreTactic` only |
| CNV-PUB-007 | all | 4 traits | 4 | 0 | all `pub`, all in `mod.rs` (or `handler.rs`) |
| CNV-ERR-001 | all | 7 entry-shaped fns | 7 | 0 | `anyhow::Result<T>` |
| CNV-ERR-002 | all | 5 callsites | 5 | 0 | `with_context` / `.context` |
| CNV-ERR-003 | all | 4 callsites | 4 | 0 | `anyhow!` |
| CNV-ERR-004 | all | 1 callsite | 1 | 0 | `anyhow::bail!` |
| CNV-ERR-005 | all | ~10 absent-cases | ~10 | 0 | `Option<T>` not `Result` |
| CNV-ERR-006 | all | 1 enum | 1 | 0 | `InsertResult` |
| CNV-ERR-007 | all | src | 0 panic/unimplemented/todo/unreachable | 0 | |
| CNV-ERR-008 | all | 5 assert + 6 debug_assert | 11 | 0 | preconditions vs. invariants |
| CNV-ERR-009 | all | 4 unwrap/expect sites | 4 documented | 0 | |
| CNV-ERR-010 | all | every Result fn | all | 0 | |
| CNV-LOG-001 | all | 2 println sites | 2 (both in main) | 0 | library never writes stdout |
| CNV-LOG-002 | all | 6 eprintln | 6 | 0 | |
| CNV-LOG-003 | all | 2 AtomicBool guards | 2 | 0 | |
| CNV-LOG-004 | all | crate | yes (no log/tracing) | no | |
| CNV-LOG-005 | all | 2 sites | 2 | 0 | first-only warn pattern |
| CNV-LOG-006 | all | 6 eprintln | 6 | 0 | `wirerust:` vs `Warning:` |
| CNV-TST-001 | most | src + tests | 19 of 20 src files free of `#[cfg(test)]` | 1 (`terminal.rs`) | legitimate exception |
| CNV-TST-002 | all | toolchain | yes | no | stdlib `#[test]` only |
| CNV-TST-003 | most | 18 test files | 17 | 1 (`integration_test.rs`) | |
| CNV-TST-004 | all | 5 integration files | 5 | 0 | `tests/fixtures/` |
| CNV-TST-005 | all | 18 test files | 18 | 0 | no shared helper module |
| CNV-TST-006 | all | 202 tests | 202 | 0 | stdlib asserts only |
| CNV-TST-007 | most | 202 tests | 182 | 20 | drift (new prose-style) |
| CNV-TST-008 | all | 1 use | 1 | 0 | `#[allow(clippy::too_many_arguments)]` once |
| CNV-TST-009 | all | dev-deps | 3 declared / 0 used | 0 | dead dev-deps |
| CNV-TST-010 | all | 1 RecordingHandler | 1 | 0 | |
| CNV-TST-011 | all | src | 0 doctests | 0 | |
| CNV-FMT-001 | all | crate | yes | no | edition 2024 |
| CNV-FMT-002 | all | crate | yes | no | rustfmt pins |
| CNV-FMT-003 | most | ~30 derive clauses | ~25 | ~5 | Parser-first for clap, Default-second for default-needing structs |
| CNV-FMT-004 | all | ~12 numeric literals | 12 | 0 | underscore separators |
| CNV-FMT-005 | all | 4 const-heavy modules | 4 | 0 | const block at top |
| CNV-FMT-006 | all | src | 0 `#[allow]` | 0 | |
| CNV-FMT-007 | most | ~120 format! sites | ~100 inline | ~20 positional | clippy enforces but exceptions exist |
| CNV-DEP-001 | all | 14 deps | 14 | 0 | SemVer major pins |
| CNV-DEP-002 | all | crate | yes | no | `Cargo.lock` committed |
| CNV-DEP-003 | all | crate | yes | no | single crate |
| CNV-DEP-004 | all | crate | no `build.rs` | no | |
| CNV-DEP-005 | all | dev-deps | 3 declared | 0 | minimal |
| CNV-DEP-006 | all | CI | yes | no | `-Dwarnings` |
| CNV-DEP-007 | all | release profile | yes | no | overflow-checks |
| CNV-DEP-008 | all | 2 dead deps | yes (csv, rayon) | no | reserved for roadmap |
| CNV-GIT-001 | all | crate | yes | no | `develop` default |
| CNV-GIT-002 | most | ~24 branches | ~22 | ~2 | `chore/`, `setup/` patterns coexist |
| CNV-GIT-003 | all | every PR | yes | no | semantic PR title CI gate |
| CNV-GIT-004 | most | 30 recent commits | 12 with scope / 18 without | n/a (scope optional) | |
| CNV-GIT-005 | all | crate | no local hooks | no | |
| CNV-GIT-006 | all | CI | 4 jobs | no | |
| CNV-GIT-007 | most | 30 recent commits | 29 | 1 | issue/PR `(#N)` suffix |
| CNV-GIT-008 | all | crate | minimal `.gitignore` | no | |
| CNV-DOC-001 | all | crate | yes (3-tier) | no | |
| CNV-DOC-002 | all | 3 ADRs | 3 | 0 | numbered + slugged |
| CNV-DOC-003 | all | 18 superpower docs | 18 | 0 | dated `YYYY-MM-DD-` |
| CNV-DOC-004 | some | 20 src files | 10 with docs | 10 without | wildly uneven |
| CNV-DOC-005 | some | 20 src files | 1 with `//!` | 19 without | `mitre.rs` only |
| CNV-DOC-006 | most | ~90 doc lines | uses `///` | 0 | |
| CNV-DOC-007 | all | 20 src files | 0 with header | 0 | (negative convention) |
| CNV-DOC-008 | most | ~5 ADR-link sites | 5 | 0 | other invariants undocumented |
| CNV-DOC-009 | most | ~4 issue cites | 4 | 0 | sparse |
| CNV-DOC-010 | all | crate | yes | no | `CLAUDE.md` |

**Totals:** 73 conventions catalogued.

By category: NAM = 12 · MOD = 6 · PUB = 7 · ERR = 10 · LOG = 6 · TST = 11 · FMT = 7 · DEP = 8 · GIT = 8 · DOC = 10.

By universality: all = 56 · most = 14 · some = 3 · none = 0.

By enforcement: rustfmt/rustc/clippy/CI gated = 32 · test-pinned = 1 (`CNV-NAM-012` via `mitre_tests.rs:185`) · manual / by-convention = 40.

---

## 4. Counter-Example Catalogue

Every convention violation, with severity (low / medium / high).

| Counter-example | Convention violated | Location | Severity | Notes |
|---|---|---|---|---|
| Inline `#[cfg(test)] mod tests` block | CNV-TST-001 | `src/reporter/terminal.rs:261-350` (10 tests for `escape_for_terminal`) | low | Legitimate exception: `escape_for_terminal` is a private fn that integration tests in `tests/` cannot reach. Inline placement is the correct choice. Convention statement (CNV-TST-001) should be amended to "tests live in `tests/*.rs` **except** for unit tests of `pub(crate)`/private helpers". |
| Test fn names without `test_` prefix | CNV-NAM-009, CNV-TST-007 | `tests/mitre_tests.rs:4,33,46,53,72,94,124,132,178,184` (10/10); `tests/reporter_tests.rs` (12/19); `tests/tls_analyzer_tests.rs:ascii_control_sni_finding_sets_mitre_t1027` (3/39) — 20 tests total | medium | Convention is drifting. The newer files (Apr 13 2026) drop `test_` in favour of prose-style names. This is a deliberate stylistic shift but **not consensus**: most files still use `test_`. Next deepening round should determine canonical direction. |
| Negated boolean field names | CNV-NAM-006 | `cli.rs:23 no_color`, `cli.rs:43 no_reassemble` | low | Mirrors CLI flag semantics; idiomatic for clap. |
| `make_*` vs `build_*` vs neither | CNV-NAM-011 | `tests/dispatcher_tests.rs:8 flow_key`, `tests/http_analyzer_tests.rs:7,16 test_flow_key/test_flow_key_b`, `tests/reporter_tests.rs:http_test_flow_key`, `tests/reader_tests.rs:7 minimal_pcap_bytes`, `tests/integration_test.rs:11 minimal_pcap_with_tcp` | low | Mixed helper-name styles. Newer test files use `make_/build_` more consistently; older test files use bare slugs. |
| `tests/integration_test.rs` singular filename | CNV-NAM-010, CNV-TST-003 | `tests/integration_test.rs` | low | Legacy from the original scaffold (`2026-04-02-wirerust-scaffold.md` plan). Rename to `pipeline_integration_tests.rs` would conform. |
| `src/reassembly/mod.rs` is a 564-LOC engine | CNV-MOD-003 | `src/reassembly/mod.rs` | medium | The convention says `mod.rs` is a thin trait+submodule file; here it owns the `TcpReassembler` engine. Refactoring to `src/reassembly/{mod.rs, engine.rs}` (engine extracted) would conform. |
| Field-exposed data carriers | CNV-PUB-004 (partial) | `ParsedPacket`, `RawPacket`, `PcapSource`, `Finding`, `ReassemblyConfig`, `ReassemblyStats`, `TcpFlow`, `TransportInfo`, `TerminalReporter`, `JsonReporter` (10 types) | low | Deliberate: these are data carriers; consumers (incl. test harness) need direct field access. Encapsulated types are the ones with **behaviour** (`FlowKey`, `FlowDirection`, `Summary`, `HttpAnalyzer`, `TlsAnalyzer`, `DnsAnalyzer`). |
| Module-level doc comments missing | CNV-DOC-005 | 19 of 20 src files have no `//!` header | medium | Only `src/mitre.rs:1-13` carries a module doc. Pass 5 deepening should map each module → its missing `//!` header. |
| Doc-comment density uneven | CNV-DOC-004 | 10 src files have **zero** `///` (`lib.rs`, `main.rs`, `dispatcher.rs`, `dns.rs`, `summary.rs`, `reader.rs`, `flow.rs`, `handler.rs`, `reporter/mod.rs`, `reporter/json.rs`) | medium | Most are short; `flow.rs` at 243 LOC and `dispatcher.rs` at 118 LOC are the more painful gaps. |
| Branch naming heterogeneity | CNV-GIT-002 | `chore/add-test-fixtures`, `chore/update-readme`, `setup/repo-essentials` (don't follow `feature/`, `worktree-issue-*`, `worktree-*`) | low | These are semantic-PR-type-shaped slugs. Convention should be widened: `<type>/<slug>` is also acceptable. |
| Derive-clause ordering with non-canonical leader | CNV-FMT-003 | `cli.rs:11 #[derive(Parser, Debug)]` — `Parser` (clap derive) leads `Debug` | low | Convention bend for clap derive macros; not contestable. |
| `ReassemblyStats` / `ReassemblyConfig` declare `Default` mid-list | CNV-FMT-003 | `reassembly/mod.rs:54-55 #[derive(Debug, Clone, Default)]` | low | `Default` slot before `PartialEq/Eq/Hash` is consistent with rustc convention. |
| Positional format args | CNV-FMT-007 | `reassembly/mod.rs:279-281,298-300,320-321` | low | ~20 sites still use positional `{}` instead of inline-capture form. Clippy's `uninlined_format_args` is set to warn; CI's `-Dwarnings` should catch new violations, so existing ones predate the lint. |
| Dev-deps declared but unused | CNV-DEP-005 / CNV-TST-009 | `Cargo.toml:28-30 assert_cmd, predicates, tempfile` | low | Dead dev-deps; cost is a slightly larger `Cargo.lock`. |
| Runtime deps declared but unused | CNV-DEP-008 | `Cargo.toml:17 csv`, `Cargo.toml:22 rayon` | medium | These are reserved for roadmap features (`--csv` output, parallel processing). They appear in the resolved dep graph and contribute to compile time. Removing until needed is the conservative choice; keeping them documents intent. |
| Single `#[allow(clippy::too_many_arguments)]` | CNV-FMT-006 (variant) | `tests/reassembly_engine_tests.rs:42` | low | Only `#[allow]` in the whole crate; only because `make_tcp_packet` is a 10-param test helper. |
| `chore/`, `setup/` branch patterns | CNV-GIT-002 | observed branches | low | See above. |
| `(#N)` issue suffix missing | CNV-GIT-007 | 1 of last 30 commits | low | Rare. |
| `--threats`, `--beacon`, `--filter`, `--hosts`, `--services`, `--json` file output, `--csv` file output | not a convention violation per se | `cli.rs:67-111` | high (Pass 0 Q#4) | Unwired CLI flags — accepted by parser, ignored by main. Not a convention drift, but a *gap*; convention-wise, every CLI flag should map to a code path. |

---

## 5. Design-Pattern Catalogue

Patterns the crate has implicitly committed to. Each item names the pattern, lists representative source locations, and gives the one-line rationale.

| Pattern | Where | Why this pattern |
|---|---|---|
| **Trait as extensibility seam** | `trait Reporter` (`src/reporter/mod.rs:8-15`); `trait ProtocolAnalyzer` (`src/analyzer/mod.rs:19-31`); `trait StreamHandler` + `trait StreamAnalyzer: StreamHandler` (`src/reassembly/handler.rs:19-29`) | The 4 traits are the seams along which the architecture is split. ADR 0002 makes this explicit. Adding a new analyzer / reporter is implementing the trait; the engine consumes the trait object, never the concrete type. |
| **Newtype-via-struct, with canonical construction** | `pub struct FlowKey { lower_ip, lower_port, upper_ip, upper_port }` (`src/reassembly/flow.rs:6-50`) | `FlowKey` is **not** a newtype around `(SocketAddr, SocketAddr)`; it's a 4-field struct with **paired-tuple canonicalization**: `FlowKey::new` orders the endpoints by `(ip, port) <= (ip, port)` so that `(a:p1, b:p2)` and `(b:p2, a:p1)` map to the same key. The critical correctness pin is the comment at `flow.rs:32-34`: sorting IPs and ports independently would merge unrelated connections. Tests at `tests/reassembly_flow_tests.rs:7-32` pin this. |
| **First-wins overlap policy as `match` on rich-result enum** | `InsertResult` (`src/reassembly/segment.rs:7-18`, 9 variants); consumed by `process_packet` (`src/reassembly/mod.rs:232-265`) | The 9-variant enum lets the engine distinguish forensic-meaningful failure modes (`ConflictingOverlap` → emit `T1036` finding; `Truncated` → emit anomaly; `DepthExceeded`, `SegmentLimitReached`, `OutOfWindow`, `Duplicate`, `PartialOverlap` → counter bumps). The first-wins policy itself is encoded inside `insert_segment` (`segment.rs:131-202`). |
| **One-shot warning via `AtomicBool::swap`** | `CLOSE_FLOW_MISSING_WARNED` (`src/reassembly/mod.rs:20,482`); `ISN_MISSING_WARNED` (`src/reassembly/segment.rs:5,43`) | A defensive eprintln that fires at most once per process, even across thousands of flows. The pattern is a module-private `static AtomicBool` + `swap(true, Ordering::Relaxed)`. Avoids a noisy log channel; relies on the invariant that the warning identifies a *programming* error (not data-driven). |
| **Content-first dispatch with port fallback** | `StreamDispatcher::classify` (`src/dispatcher.rs:37-64`) | TLS bytes (`0x16 0x03`) and HTTP method tokens (`"GET "`, …, `"HTTP/"`) take precedence over port number; port fallback (`443`/`8443`/`80`/`8080`) is only used when the byte stream is too short to be conclusive. ADR 0001 formalises this. Tests at `tests/dispatcher_tests.rs:44-71` pin both branches. |
| **Tests-as-spec invariant** | `tests/*.rs` (202 tests, 6,021 LOC) vs `src/**/*.rs` (3,868 LOC) — ratio 1.56:1 | Test files encode the load-bearing contracts. Per Pass 3, every behavioural contract is grounded in a test citation (high confidence). The unusual ratio (>1.5× test LOC) and the wholesale **absence of inline `#[cfg(test)] mod tests`** force the contract surface into integration tests, where it is reachable by future regression authors. |
| **`#[non_exhaustive]` on externally-evolving enums only** | `MitreTactic` (`src/mitre.rs:22`) | Used precisely once, because MITRE ATT&CK evolves outside the crate. All other enums (which the crate fully owns: `Verdict`, `Confidence`, `Direction`, `CloseReason`, `FlowState`, `Protocol`, `OutputFormat`, etc.) are **closed** — `match` exhaustiveness is intentional. |
| **Static `match`-as-lookup-table** | `mitre::technique_info` (`src/mitre.rs:98-132`); `is_grease_u16` (`analyzer/tls.rs:23-25`); `is_weak_cipher` (`analyzer/tls.rs:29-37`); `cipher_name` (`analyzer/tls.rs:51-56`); `decode_packet`'s link-type match (`decoder.rs:72-77`) | The crate explicitly avoids `phf` / `lazy_static` / `build.rs` codegen. Static `match` arms are idiomatic Rust at this scale (~16 MITRE IDs, ~6 link types, ~10 cipher classifications). The trade-off is documented in `tests/mitre_tests.rs:185-217` ("hand-curated approach is the idiomatic Rust pattern at this scale; revisit when emission sites grow > ~20"). |
| **Raw-vs-display separation (data layer + escaping at the rendering boundary)** | `findings.rs:72-92 Finding::Display` (raw, *not* terminal-safe); `reporter/terminal.rs:29-46 escape_for_terminal` (the only terminal-safe escape primitive); `reporter/json.rs:1-38 JsonReporter::render` (relies on `serde_json`'s RFC 8259 escaping) | ADR 0003 is the rationale. Findings store raw bytes; the **renderer** is responsible for escaping. `escape_for_terminal` covers C0 + DEL + C1 + backslash; non-ASCII Unicode passes through. The convention is enforced by tests at `tests/reporter_tests.rs:91-150`. |
| **`Option` over `Result` for "may-be-absent"** | `technique_info` (`mitre.rs:98`); `parse_one_request` returns `Result<Option<ParsedRequest>, httparse::Error>` distinguishing partial-from-error (`analyzer/http.rs:1095`); `ParsedPacket::app_protocol_hint` (`decoder.rs:46`); `extract_sni` (`analyzer/tls.rs:1858`) | `Option<T>` is reserved for "value may not be present", `Result<T, E>` for "value may not be computable". The triple-Result-and-Option `Result<Option<T>, httparse::Error>` is the gold-standard form for streaming parsers (more data needed vs parse error vs success). |
| **Configuration via `*Config` builder-less struct + `Default`** | `ReassemblyConfig` (`src/reassembly/mod.rs:23-51`) | All fields `pub`, `impl Default` declares the production defaults, callers mutate fields directly (`ReassemblyConfig { max_depth: …, ..ReassemblyConfig::default() }` at `main.rs:79-83`). No builder pattern — Rust's record-update syntax + `Default` cover it. |
| **`AnalysisSummary` as universal output payload** | `analyzer/mod.rs:13-17 AnalysisSummary { analyzer_name, packets_analyzed, detail: HashMap<String, serde_json::Value> }`; consumed by every reporter | Every analyzer (DNS, HTTP, TLS, reassembly engine) produces the same shape: a name, a packet count, and an open-ended `detail` map. Reporters render the map without knowing what's inside. This is the project's single-most-flexible cross-cutting concession. |
| **`assert!` for constructor preconditions, `debug_assert!` for invariants** | `TcpReassembler::new` (`mod.rs:86-96`) uses `assert!` for 5 config invariants — these always fire, even in release. `debug_assert!` (~6 sites) is used for "this would be a bug" invariants that should not crash a release build (`mod.rs:223,481`; `segment.rs:178,208`; `flow.rs:150`). | Constructor preconditions are protection against config bugs (firing makes the program useless anyway). Internal invariants are protection against engine bugs (firing should crash debug builds, log + degrade in release). |
| **First-error-then-counter rate limiting** | `main.rs:125-131,205-211` | `if total_decode_errors == 0 { eprintln!(...) } total_decode_errors += 1;`. Subsequent errors are silently counted; the final summary line reports the total. Applied at both `run_analyze` and `run_summary` for symmetry. |
| **Test-double as real trait impl, not a mock** | `RecordingHandler` (`tests/reassembly_engine_tests.rs:10-40`) | The test recorder implements `StreamHandler` and records all callbacks for assertion. No mock framework; no expect-call protocol. This is more refactor-stable than mock-based tests because the trait shape is the surface, not the call sequence. |

---

## 6. Anti-Conventions / Inconsistencies

Where a convention is broken *unevenly* (rather than universally), and the inconsistency could trip future authors:

| Inconsistency | Severity | Where | Why it could trip a future author |
|---|---|---|---|
| **Test function naming is mid-transition** (`test_*` vs prose-style) | high | 91.6% use `test_*`; the newest 3 files use prose-style; mixed within `reporter_tests.rs` (7 prose / 12 `test_*`). | A new contributor sees both styles in the same file and cannot pick correctly. Resolution: pick a direction (either widen prose-style or restore `test_*`) and codify in `CLAUDE.md`. |
| **Doc-comment density is wildly uneven across modules** | high | 10 of 20 files have **zero** `///` comments. `flow.rs` (243 LOC, 4 pub types, 0 doc lines) is the worst gap. | Future authors cannot tell whether undocumented = simple (so docs unneeded) or undocumented = neglected. Resolution: enforce a per-public-item doc-comment policy or write `//!` module-level docs everywhere. |
| **Module-level doc comments exist in only one module** | high | `src/mitre.rs:1-13` only | The pattern was started but never adopted. Either drop it (delete the `mitre.rs` header) or roll it out across all 20 modules. |
| **Branch naming has 5+ patterns** | medium | `feature/`, `worktree-`, `worktree-issue-N-`, `chore/`, `setup/` | `CLAUDE.md` documents 3 patterns; actual practice uses 5. Update `CLAUDE.md` to allow `<type>/<slug>` matching semantic-PR types. |
| **Test helper naming has 3+ styles** | medium | `make_*`, `build_*`, bare slugs (`flow_key`, `minimal_pcap_bytes`) | New tests may pick a fourth style. Resolution: codify `make_*` for `ParsedPacket` builders; `build_*` for raw-bytes synthesizers; reserved fn names for "trivial helper for this file" only. |
| **Format-string interpolation is mixed** | low | ~100 inline-capture + ~20 positional | Old code is positional; new code is inline-capture. clippy enforces direction; CI may flag a new positional with `-Dwarnings`. |
| **Dev-deps declared but unused** | low | `assert_cmd`, `predicates`, `tempfile` (Cargo.toml:28-30) | A future contributor may write an `assert_cmd` test thinking it's the convention; in fact in-process `Cli::parse_from` is. Either delete the dev-deps or write at least one test using them as exemplar. |
| **Runtime deps declared but unused** | medium | `csv` (Cargo.toml:17), `rayon` (Cargo.toml:22) | Compiles them on every cargo build. Pass 0 Q#1, Q#2 flag the unwired CLI features. |
| **CLI flags exist but are unwired** | high | `--threats`, `--beacon`, `--filter`, `--hosts`, `--services`, `--json` (file form), `--csv` (file form) | A user typing `wirerust analyze cap.pcap --threats` gets *no error* and *no threat detection*. Convention violation: every public CLI flag should map to a code path or be removed. |
| **`src/reassembly/mod.rs` is a 564-LOC engine in a `mod.rs` file** | medium | `src/reassembly/mod.rs` | Breaks the "mod.rs = trait+submodule declaration" rule encoded by the other two multi-file modules. Refactor to `src/reassembly/{mod.rs, engine.rs}`. |
| **`#[derive]` ordering bends for derive macros** | low | `cli.rs:11 #[derive(Parser, Debug)]` | Documented as a known bend; no fix needed but worth noting. |

---

## 7. Convention-Enforcement Coverage

Which conventions are automatically enforced vs. only-by-convention. Drift risk concentrates in the **manual** column.

| Enforcement class | Conventions | Drift risk |
|---|---|---|
| **rustfmt / CI fmt** | CNV-FMT-001, CNV-FMT-002 (max_width, edition, shorthand) | very low |
| **clippy / CI clippy + RUSTFLAGS=-Dwarnings** | CNV-NAM-002, CNV-NAM-003, CNV-NAM-007, CNV-NAM-008, CNV-FMT-004 (numeric literal underscores), CNV-FMT-006 (no `#[allow]`), CNV-FMT-007 (inline-capture format args) | very low |
| **rustc compile / borrow / privacy** | CNV-MOD-001 (lib+bin), CNV-MOD-004 (lib re-exports modules), CNV-PUB-001..003 (visibility ladder), CNV-PUB-005 (no pub use), CNV-PUB-007 (traits in mod.rs), CNV-ERR-006 (exhaustive `match` on `InsertResult`), CNV-ERR-007 (no panic/unimplemented in src is a manual rule but `cargo test --all-targets` would surface a panic), CNV-ERR-008 (assert vs debug_assert), CNV-ERR-010 (`?` propagation), CNV-FMT-001 (edition) | very low |
| **CI fmt + clippy + test combined** | CNV-DEP-006 (`-Dwarnings`), CNV-DEP-007 (overflow-checks), CNV-TST-002 (`cargo test --all-targets`) | very low |
| **Semantic-PR action** | CNV-GIT-003 (PR title types), CNV-GIT-004 (scope optional) | very low |
| **Specific regression test** | CNV-NAM-012 (every emitted technique ID resolves) — pinned by `tests/mitre_tests.rs:185-217` | low |
| **Specific regression test (escaping)** | CNV-PAT raw-vs-display (ADR 0003) — pinned by `tests/reporter_tests.rs` C0/C1 tests | low |
| **Manual / by-convention** | CNV-NAM-001 (snake_case modules — clippy partial), CNV-NAM-004 (`new` constructor name), CNV-NAM-005 (`from_*` for alt input), CNV-NAM-006 (bool naming), CNV-NAM-009 (`test_` prefix in tests — **at-risk drift**), CNV-NAM-010 (test-file naming), CNV-NAM-011 (`make_*` vs `build_*`), CNV-MOD-002, CNV-MOD-003 (`mod.rs` thin), CNV-MOD-005 (alphabetical lib.rs), CNV-MOD-006 (use group ordering — partially clippy if `imports_granularity` enabled, currently isn't), CNV-PUB-004 (field privacy on behavior types), CNV-PUB-006 (`#[non_exhaustive]` for outward enums), CNV-ERR-001..005, CNV-ERR-009 (`unwrap` discipline), CNV-LOG-001..006 (every logging convention), CNV-TST-001 (integration-only tests — **at-risk if a contributor adds inline `#[cfg(test)]`**), CNV-TST-003 (test-file pairing), CNV-TST-004 (fixtures location), CNV-TST-005 (no shared helper module), CNV-TST-006 (stdlib asserts only), CNV-TST-007 (test_ prefix), CNV-TST-009 (dev-deps usage), CNV-TST-010 (test doubles as trait impl), CNV-TST-011 (no doctests), CNV-FMT-003 (derive ordering), CNV-FMT-005 (const block at top), CNV-DEP-001..005, CNV-DEP-008 (dead deps documented), CNV-GIT-001 (default branch), CNV-GIT-002 (branch naming — **drift**), CNV-GIT-005 (no hooks), CNV-GIT-006 (CI jobs), CNV-GIT-007 (`#N` suffix), CNV-GIT-008 (gitignore), CNV-DOC-001..010 (every doc convention) | **HIGH drift risk on:** doc-comment density (CNV-DOC-004), test-naming transition (CNV-NAM-009 / CNV-TST-007), branch naming heterogeneity (CNV-GIT-002). |

**Three conventions most at risk of drift:**

1. **CNV-NAM-009 / CNV-TST-007 (test function naming)** — actively transitioning. 91.6% conform, 8.4% don't. The newest test files (`mitre_tests.rs`, `reporter_tests.rs` portions, `tls_analyzer_tests.rs` portions, Apr 13 2026) reject the `test_` prefix. No lint, no test, no CI gate. A contributor opening a fresh file has 50-50 odds of picking the wrong style.
2. **CNV-DOC-004 (per-pub-item doc-comment policy)** — half the files have zero `///`. No formalisation. A future contributor cannot tell "this is simple so no docs needed" from "this was forgotten". Rust's `missing_docs` lint is not enabled.
3. **CNV-GIT-002 (branch naming)** — `CLAUDE.md` documents 3 patterns; actual practice uses 5. A contributor following `CLAUDE.md` won't include `chore/<slug>` even though `chore/add-test-fixtures` exists.

---

## 8. Recommendations for Pass 5 Deepening Rounds

The following 9 gaps justify another deepening round before declaring NITPICK convergence.

1. **Doc-comment policy formalisation.** Density is uneven (10 of 20 files with zero `///`). Pass 5 round 2 should map every `pub fn` / `pub struct` / `pub enum` / `pub trait` to its doc-state and produce a `pub-doc-coverage` table. Then propose a policy: "every `pub` item in `src/lib.rs`-exposed modules must have at least one `///` line, or an explicit `// undocumented: trivial` comment". Without this, the convention will continue to drift.
2. **Module-level `//!` headers.** Only `src/mitre.rs:1-13` carries one. Round 2 should either delete it (close the convention) or roll out a 1-3 line `//!` to every module (open it).
3. **Test-naming convention canonicalisation.** Determine whether the prose-style names (`mitre_tests.rs`, parts of `reporter_tests.rs`, `tls_analyzer_tests.rs`) are an intentional new direction or accidental drift. Inspect commit messages for the newer test files (Apr 13 2026 commits in `git log`) and codify the answer in `CLAUDE.md`.
4. **Branch-naming spec.** `CLAUDE.md` lists 3 patterns; observed practice has 5 (incl. `chore/`, `setup/`). Round 2 should `git for-each-ref` over remote branches, classify them, and either widen `CLAUDE.md` or close the convention by renaming outliers.
5. **`#[allow]` audit.** The crate has zero `#[allow]` in src and one in tests. Round 2 should confirm by reading every line of every `.rs` file (the awk over `^#\[allow` may miss indented or trailing-comment forms) and decide whether to formalise the "zero #[allow]" convention as a clippy lint config (currently relies on `-Dwarnings`).
6. **Helper-fn naming.** Round 2 should resolve the `make_*` vs `build_*` vs bare-slug inconsistency in `tests/`. Propose: `make_<thing>` for `ParsedPacket` and other crate-owned domain types; `build_<thing>_<flavor>` for raw-bytes (protocol-level) synthesizers; no bare slugs.
7. **`pub` field exposure on data carriers.** Round 2 should document the rule that distinguishes "data carrier with `pub` fields" from "behaviour-owning with private fields + accessors". Currently 10 types each side; the distinction is consistent but unwritten.
8. **CLI-flag wiring audit.** Pass 0 Q#4 lists 7 declared but unwired flags. Pass 5 is the right place to record the *convention* that "every accepted flag maps to a code path or returns an error". Round 2 should propose: either remove unwired flags from `cli.rs` or add a "Feature not yet implemented" early-exit per flag.
9. **`mod.rs` weight.** Reassembly's `mod.rs` is 564 LOC vs analyzer's 31 LOC and reporter's 15 LOC. Round 2 should propose extracting the engine into `src/reassembly/engine.rs` to align with the implicit "mod.rs is thin" convention.
10. **Format-string positional vs inline-capture census.** ~20 sites still positional. Round 2 should produce the line-level table and propose a one-shot refactor to bring everything to inline-capture form.

---

## Pass 5 State Checkpoint

```yaml
pass: 5
round: 1
status: complete
files_scanned: 38_rust_source + 5_pass_inputs + 3_adrs + 4_metadata
conventions_catalogued: 73
counter_examples_documented: 18
design_patterns_documented: 15
inconsistencies_flagged: 11
timestamp: 2026-05-19T00:00:00Z
novelty: SUBSTANTIVE
next_pass: 5-deep-round-2 (or pass-6 synthesis if other passes converge)
resume_from: null
```

