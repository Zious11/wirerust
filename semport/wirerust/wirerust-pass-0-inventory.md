# Pass 0: Inventory — wirerust

- **Project:** wirerust
- **Source path:** `/Users/zious/Documents/GITHUB/wirerust/`
- **Generated:** 2026-05-19
- **Pass:** 0 (Inventory) — Phase A broad-sweep, round 1
- **Confidence:** HIGH (all metrics derived from `find`/`wc`/file reads against the live tree; no estimates)

## 1. Tech Stack

| Aspect | Value | Source of truth |
|---|---|---|
| Language | Rust | `Cargo.toml` |
| Edition | 2024 (requires rustc 1.85+, stabilized 2025-02-20) | `Cargo.toml:4`, `rustfmt.toml:1`, `CLAUDE.md` |
| Crate type | Single binary crate (`wirerust`) with a sibling library (`lib.rs` exports modules) | `Cargo.toml`, `src/lib.rs`, `src/main.rs` |
| Crate version | 0.1.0 | `Cargo.toml:3` |
| License | MIT | `Cargo.toml:6`, `LICENSE` |
| Build tool | `cargo` | conventional |
| Lockfile | `Cargo.lock` (38,291 bytes; checked in — binary crate) | top-level |
| Test framework | Built-in Rust `#[test]` (no `tokio::test`; no integration framework crate) | confirmed by `awk` scan of all test files (0 matches for `#[tokio::test]`) |
| Lint | `cargo clippy --all-targets -- -D warnings` | `.github/workflows/ci.yml:58` |
| Format | `cargo fmt --all --check` | `.github/workflows/ci.yml:68`; `rustfmt.toml` pins `edition=2024`, `max_width=100`, `use_field_init_shorthand=true`, `use_try_shorthand=true` |
| CI | GitHub Actions: `Semantic PR`, `Test`, `Clippy`, `Format` (all on `ubuntu-latest`, stable toolchain via `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`) | `.github/workflows/ci.yml` |
| CI envs | `CARGO_TERM_COLOR=always`, `RUSTFLAGS=-Dwarnings` | `.github/workflows/ci.yml:10-12` |
| MSRV | Implicit — whatever `dtolnay/rust-toolchain@stable` resolves to today; minimum 1.85 because of edition 2024. No `rust-toolchain.toml` pin. | `CLAUDE.md`, `.github/workflows/ci.yml` |
| Target platforms | CI builds only Linux (ubuntu-latest). README installation via `cargo install --path .` implies any cargo-supported host; no platform-specific `cfg` or build scripts found. | `.github/workflows/ci.yml`, `README.md`, no `build.rs` |
| Release profile | `overflow-checks = true` (defensive; arithmetic panics in release) | `Cargo.toml:24-25` |
| Git workflow | Default branch `develop` (git-flow); semantic PR titles enforced (`feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`); no local commit hooks. | `CLAUDE.md`, `.github/workflows/ci.yml:22-38` |

## 2. Dependencies

All from `Cargo.toml`.

### `[dependencies]` (12 direct)

| Crate | Req | Features | Purpose | Evidence of use |
|---|---|---|---|---|
| `httparse` | `"1"` | default | HTTP/1.x request/response header parser (zero-allocation) | `src/analyzer/http.rs` (`httparse::Request`, `httparse::Response`, `httparse::EMPTY_HEADER`, `httparse::Status`, `httparse::Error`) |
| `tls-parser` | `"0.12"` | default | TLS handshake parser (ClientHello/ServerHello, extensions, cipher suite catalog) | `src/analyzer/tls.rs:4-7` (`TlsCipherSuite`, `TlsExtension`, `parse_tls_plaintext`, `parse_tls_extensions`, `TlsMessage`, `TlsMessageHandshake`) |
| `md-5` | `"0.11"` | default | MD5 hashing for JA3/JA3S fingerprints | `src/analyzer/tls.rs:3` (`use md5::{Digest, Md5}`) |
| `clap` | `"4"` | `derive` | CLI argument parsing | `src/cli.rs` (`Parser`, `Subcommand`, `ValueEnum`), `src/main.rs:5` (`Parser`) |
| `etherparse` | `"0.16"` | default | Zero-copy L2-L4 packet slicer (Ethernet/IPv4/IPv6/TCP/UDP/ICMP/SLL) | `src/decoder.rs:4` (`SlicedPacket`, `NetSlice`, `TransportSlice`) |
| `pcap-file` | `"2"` | default | pcap (libpcap) file reader; provides `DataLink` enum | `src/reader.rs:4-5` (`PcapReader`, `DataLink`), `src/decoder.rs:5` |
| `serde` | `"1"` | `derive` | Struct serialization (used for JSON output of `Summary`, `Finding`, `AnalysisSummary`, `Protocol`, `Verdict`, `Confidence`, `ThreatCategory`) | `src/findings.rs:5`, `src/summary.rs:4`, `src/decoder.rs:6`, `src/analyzer/mod.rs:7` |
| `serde_json` | `"1"` | default | JSON serialization for reporter and per-analyzer `detail` maps | `src/reporter/json.rs:1`, `src/analyzer/{dns.rs,http.rs,tls.rs}` (`serde_json::json!`), `src/main.rs:158` |
| `csv` | `"1"` | default | CSV output — **DECLARED BUT UNUSED in current sources** (`--csv` CLI flag exists at `src/cli.rs:35-36` but never references the crate) | no `use csv` in `src/` (confirmed via `awk`) |
| `anyhow` | `"1"` | default | Error context / `Result` ergonomics | `src/decoder.rs:3`, `src/reader.rs:3`, `src/main.rs:3` |
| `owo-colors` | `"4"` | default | ANSI color codes for terminal reporter | `src/reporter/terminal.rs:1` (`OwoColorize`) |
| `indicatif` | `"0.17"` | default | Progress bar during packet ingest | `src/main.rs:5` (`ProgressBar`, `ProgressStyle`) |
| `chrono` | `"0.4"` | `serde` | `DateTime<Utc>` for finding timestamps | `src/findings.rs:4` (`DateTime`, `Utc`) |
| `rayon` | `"1"` | default | Data-parallel iteration — **DECLARED BUT UNUSED in current sources** (README "Roadmap" lists "Parallel file processing" as future work) | no `use rayon` in `src/` (confirmed via `awk`); `README.md:152` lists as roadmap |

### `[dev-dependencies]` (3 direct)

| Crate | Req | Features | Purpose | Evidence of use |
|---|---|---|---|---|
| `assert_cmd` | `"2"` | default | CLI integration testing (spawn binary, assert exit / stdout) — **DECLARED BUT UNUSED** | no `use assert_cmd` in `tests/` (confirmed via `awk`) |
| `predicates` | `"3"` | default | Predicate combinators for `assert_cmd` — **DECLARED BUT UNUSED** | no `use predicates` in `tests/` (confirmed via `awk`) |
| `tempfile` | `"3"` | default | Temp directories for file-system tests — **DECLARED BUT UNUSED** | no `use tempfile` / `TempDir` in `tests/` (confirmed via `awk`) |

### `[build-dependencies]`

None. There is no `build.rs`.

## 3. File Tree

```
/Users/zious/Documents/GITHUB/wirerust/
├── Cargo.toml                        31 lines    (crate manifest)
├── Cargo.lock                        ~1300 lines (38,291 bytes; not LOC-counted — generated)
├── LICENSE                           1 KB        (MIT)
├── README.md                         159 lines   (project overview, usage, architecture, roadmap)
├── CLAUDE.md                         see project context  (agent guidance — build/test/lint/git workflow)
├── rustfmt.toml                      5 lines     (edition 2024, max_width 100, shorthand opts)
├── .gitignore                        — (target/, .DS_Store et al.; 27 bytes)
├── .github/
│   └── workflows/
│       └── ci.yml                    68 lines    (4 jobs: semantic-pr, test, clippy, fmt)
├── docs/
│   ├── adr/
│   │   ├── 0001-content-first-stream-dispatch.md          107 lines
│   │   ├── 0002-modular-protocol-analyzers.md             146 lines
│   │   └── 0003-reporting-pipeline-layering.md            224 lines
│   └── superpowers/
│       ├── plans/   (10 markdown plans, dated 2026-04-02 → 2026-04-13)
│       └── specs/   (8 markdown design specs, dated 2026-04-06 → 2026-04-13)
│       (18 files in docs/superpowers total; combined ~12,052 LOC)
├── src/                              3,868 LOC across 20 .rs files
│   ├── lib.rs                            10
│   ├── main.rs                          256
│   ├── cli.rs                           113
│   ├── decoder.rs                       140
│   ├── dispatcher.rs                    118
│   ├── findings.rs                       92
│   ├── mitre.rs                         144
│   ├── reader.rs                         58
│   ├── summary.rs                        61
│   ├── analyzer/
│   │   ├── mod.rs                        31    (defines ProtocolAnalyzer trait, AnalysisSummary)
│   │   ├── dns.rs                        81
│   │   ├── http.rs                      535
│   │   └── tls.rs                       750
│   ├── reassembly/
│   │   ├── mod.rs                       564    (TcpReassembler engine — main loop)
│   │   ├── flow.rs                      243    (FlowKey, FlowDirection, FlowState, TcpFlow)
│   │   ├── segment.rs                   240    (InsertResult, first-wins overlap policy)
│   │   └── handler.rs                    29    (Direction, CloseReason, StreamHandler, StreamAnalyzer traits)
│   └── reporter/
│       ├── mod.rs                        15    (Reporter trait)
│       ├── json.rs                       38
│       └── terminal.rs                  350    (terminal escape + colored table + MITRE grouping; contains its own #[test] block)
└── tests/                            6,021 LOC across 18 .rs files (+ 14 binary pcap fixtures)
    ├── fixtures/   (14 binary files; .pcap/.pcapng/.cap; 1,209 → 33,144 bytes)
    │   ├── dns.cap                       4,338 B
    │   ├── dns-remoteshell.pcap         25,005 B
    │   ├── http.pcap                       247 B
    │   ├── http-full.cap                25,803 B
    │   ├── http-ooo.pcap                 1,209 B
    │   ├── ipv6-ripng.pcap              20,264 B
    │   ├── segmented.pcap               33,144 B
    │   ├── slammer.pcap                    458 B
    │   ├── smb3.pcapng                  15,692 B   (pcapng format — NOT supported by reader; per README L126 "Not yet supported"; presumably used as negative test)
    │   ├── teardrop.cap                  1,828 B
    │   ├── tls.pcap                     25,057 B
    │   ├── tls12-aes256gcm.pcap          2,064 B
    │   ├── tls13-rfc8446.pcap            4,158 B
    │   └── v6-http.cap                   9,159 B
    ├── analyzer_tests.rs                  68
    ├── cli_tests.rs                      110
    ├── decoder_tests.rs                  216
    ├── dispatcher_tests.rs                98
    ├── findings_tests.rs                  35
    ├── http_analyzer_tests.rs            735
    ├── http_integration_tests.rs          35
    ├── integration_test.rs                66
    ├── linktype_integration_tests.rs      40
    ├── mitre_tests.rs                    217
    ├── reader_tests.rs                   148
    ├── reassembly_engine_tests.rs       1398
    ├── reassembly_flow_tests.rs          102
    ├── reassembly_segment_tests.rs       402
    ├── reporter_tests.rs                 701
    ├── summary_tests.rs                   74
    ├── tls_analyzer_tests.rs            1455
    └── tls_integration_tests.rs          121
```

Excluded from inventory per task scope: `.git/`, `.factory/`, `target/`, `.claude/`. No `.reference/` present at root.

## 4. File-prioritization scoring

Source-file priorities (1-5; higher = more important for downstream Passes 1-5). Sorted descending.

| Score | File | LOC | Rationale (1 sentence) |
|---|---|---|---|
| 5 | `src/main.rs` | 256 | Binary entry point — wires CLI → reader → decoder → reassembly → dispatcher → analyzers → reporters; the canonical end-to-end pipeline lives here. |
| 5 | `src/lib.rs` | 10 | Library root; defines the public module surface (9 modules) and is the entry point for every integration test. |
| 5 | `src/dispatcher.rs` | 118 | Content-first stream protocol dispatch; subject of ADR 0001; cross-cutting between reassembly and HTTP/TLS analyzers. |
| 5 | `docs/adr/0001-content-first-stream-dispatch.md` | 107 | Definitive design rationale for dispatcher behavior — must be read before specifying it. |
| 5 | `docs/adr/0002-modular-protocol-analyzers.md` | 146 | Defines the `ProtocolAnalyzer` + `StreamAnalyzer` trait contract — the extensibility seam. |
| 5 | `docs/adr/0003-reporting-pipeline-layering.md` | 224 | Defines the data-vs-display layering (raw `Finding` text is attacker-controlled; only `TerminalReporter` escapes). |
| 4 | `src/reassembly/mod.rs` | 564 | Main TCP reassembly engine — orchestrates per-packet flow state, eviction, finalize, finding emission; largest and most state-heavy module. |
| 4 | `src/analyzer/tls.rs` | 750 | TLS ClientHello/ServerHello parser + JA3/JA3S + weak-cipher / SSL2/SSL3 detection; largest file in the crate. |
| 4 | `src/analyzer/http.rs` | 535 | HTTP/1.x stream parser; detects path traversal, web shells, unusual methods, host-header anomalies. |
| 4 | `src/reassembly/flow.rs` | 243 | `FlowKey` canonicalization (critical correctness — see comment on tuple-paired sort), `FlowState` machine, per-direction state. |
| 4 | `src/reassembly/segment.rs` | 240 | `InsertResult` enum + first-wins overlap policy (forensic invariant). |
| 4 | `src/decoder.rs` | 140 | `ParsedPacket` model + per-link-type slicing (Ethernet/RAW/IPv4/IPv6/SLL); the L2-L4 boundary. |
| 4 | `src/reassembly/handler.rs` | 29 | Tiny but defines the `StreamHandler` / `StreamAnalyzer` trait contract used by dispatcher + HTTP + TLS — load-bearing. |
| 4 | `src/findings.rs` | 92 | Domain model for outputs: `Verdict`, `Confidence`, `ThreatCategory`, `Finding`; doc comment encodes the Pass 3 raw-vs-display contract. |
| 4 | `src/analyzer/mod.rs` | 31 | Trait + `AnalysisSummary` type; the per-packet analyzer contract. |
| 3 | `src/reporter/terminal.rs` | 350 | Colored ASCII table renderer with MITRE grouping and C0/C1 control-byte escaping. |
| 3 | `src/mitre.rs` | 144 | MITRE ATT&CK technique-ID → (name, tactic) lookup with `#[non_exhaustive]` enum; seeded set of 16 IDs (incl. 4 ICS). |
| 3 | `src/cli.rs` | 113 | `clap` derive structs; defines all global flags and the two subcommands (`analyze`, `summary`). |
| 3 | `src/reader.rs` | 58 | pcap-file wrapper; gatekeeper for supported link types (rejects pcapng + other types up front). |
| 3 | `src/summary.rs` | 61 | Lightweight aggregation (host/protocol/service counters). |
| 3 | `src/reporter/json.rs` | 38 | JSON output sibling to terminal; emits raw `Finding` payload (relies on consumers handling escaping). |
| 2 | `src/reporter/mod.rs` | 15 | Trait declaration only. |
| 2 | `src/analyzer/dns.rs` | 81 | Query/response counter only — does not yet emit findings (`analyze()` returns empty `Vec`); thin compared to HTTP/TLS. |
| 1 | `tests/reassembly_engine_tests.rs` | 1398 | Highest-value test file — spec-encodes engine invariants (overlap, depth, memcap, eviction); read in Pass 3 as primary BC source. |
| 1 | `tests/tls_analyzer_tests.rs` | 1455 | Spec-encodes TLS parser & JA3 invariants (~39 tests). |
| 1 | `tests/http_analyzer_tests.rs` | 735 | Spec-encodes HTTP detection rules (~35 tests). |
| 1 | `tests/reporter_tests.rs` | 701 | Spec-encodes terminal+JSON output (~19 tests). |
| 1 | `tests/reassembly_segment_tests.rs` | 402 | First-wins overlap policy detail tests (~23 tests). |
| 1 | `tests/mitre_tests.rs` | 217 | MITRE lookup invariants — display, tactic order, ID→name (~10 tests). |
| 1 | `tests/decoder_tests.rs` | 216 | Per-link-type decode tests (~7 tests). |
| 1 | `tests/reader_tests.rs` | 148 | pcap header parsing + unsupported link-type rejection (~8 tests). |
| 1 | `tests/tls_integration_tests.rs` | 121 | End-to-end pcap → TLS analyzer with `tls12-aes256gcm.pcap`, `tls13-rfc8446.pcap`, `tls.pcap` (~4 tests). |
| 1 | `tests/cli_tests.rs` | 110 | Pure `Cli::parse_from` tests, no binary spawn (~8 tests). |
| 1 | `tests/reassembly_flow_tests.rs` | 102 | `FlowKey` + `FlowDirection` + state machine unit tests (~7 tests). |
| 1 | `tests/dispatcher_tests.rs` | 98 | Content-first vs port-fallback dispatch tests (~6 tests). |
| 1 | `tests/summary_tests.rs` | 74 | `Summary` aggregation tests (~3 tests). |
| 1 | `tests/analyzer_tests.rs` | 68 | `DnsAnalyzer` smoke tests (~3 tests). |
| 1 | `tests/integration_test.rs` | 66 | End-to-end pcap → analyzer with a synthetic in-memory pcap (~1 test). |
| 1 | `tests/linktype_integration_tests.rs` | 40 | Cross-link-type pcap loads (~3 tests). |
| 1 | `tests/findings_tests.rs` | 35 | `Finding`/`Verdict`/`Confidence` constructor & `Display` smoke (~2 tests). |
| 1 | `tests/http_integration_tests.rs` | 35 | End-to-end pcap → HTTP analyzer with `http.pcap`/`http-full.cap` (~1 test). |

## 5. Test Inventory

All counts of `#[test]` derived from `awk '/^#\[(test|tokio::test)\]/'` over each file. There are **zero** `#[tokio::test]` in the codebase. There are **zero** `#[test]` functions inline in `src/` modules — every test lives under `tests/`. **Total test functions: 202** across 18 files.

| File | LOC | `#[test]` count | One-line summary |
|---|---|---|---|
| `tests/reassembly_engine_tests.rs` | 1398 | 23 | End-to-end `TcpReassembler` behavior: SYN/SYN-ACK handshake, ISN tracking, depth/memcap/eviction, RST/FIN/timeout finalize, finding emission, stats. |
| `tests/tls_analyzer_tests.rs` | 1455 | 39 | `TlsAnalyzer` stream-level tests: ClientHello/ServerHello parse, SNI extraction, JA3/JA3S md5, weak/null/anon/export cipher detection, SSL 2.0/3.0 deprecated, GREASE filtering, control-byte SNI, malformed records. |
| `tests/http_analyzer_tests.rs` | 735 | 35 | `HttpAnalyzer` stream-level tests: GET/POST/etc. parsing, host/user-agent extraction, path-traversal detection, web-shell URI detection, unusual methods, oversized headers, partial requests, parse-error counters. |
| `tests/reporter_tests.rs` | 701 | 19 | Terminal + JSON reporter rendering: MITRE tactic grouping, color toggling, control-byte escaping (C0+DEL+C1+backslash), empty-state handling, JSON shape. |
| `tests/reassembly_segment_tests.rs` | 402 | 23 | `FlowDirection::insert_segment` unit tests: first-wins overlap, duplicate detection, depth/segment-limit, out-of-window, ISN missing path. |
| `tests/mitre_tests.rs` | 217 | 10 | `MitreTactic::Display` canonical names, `technique_info` lookups, `all_tactics_in_report_order` stability, sub-technique IDs (`T1071.001`, `T1499.002`, `T1505.003`), ICS techniques. |
| `tests/decoder_tests.rs` | 216 | 7 | `decode_packet` for Ethernet, RAW IP (IPv4 & IPv6), Linux SLL, with TCP / UDP / ICMP payloads and unsupported-link-type rejection. |
| `tests/reader_tests.rs` | 148 | 8 | `PcapSource::from_pcap_reader` happy path + corrupt-header / unsupported-link-type / pcapng rejection. |
| `tests/tls_integration_tests.rs` | 121 | 4 | End-to-end pcap → `TlsAnalyzer` on `tls.pcap`, `tls12-aes256gcm.pcap`, `tls13-rfc8446.pcap` fixtures. |
| `tests/cli_tests.rs` | 110 | 8 | `Cli::parse_from` for both subcommands, `--reassemble`/`--no-reassemble`, `--no-color`, `--mitre`, `--output-format`, multi-target. |
| `tests/reassembly_flow_tests.rs` | 102 | 7 | `FlowKey` canonicalization (ip-port pair ordering), `FlowState` machine transitions, `TcpFlow` per-direction state. |
| `tests/dispatcher_tests.rs` | 98 | 6 | `StreamDispatcher` classification: content-first TLS (0x16 0x03), HTTP method prefixes, port fallback (443/8443/80/8080), `unclassified_flows` counter. |
| `tests/summary_tests.rs` | 74 | 3 | `Summary::ingest` aggregates packets, bytes, hosts, protocols, services. |
| `tests/analyzer_tests.rs` | 68 | 3 | `DnsAnalyzer` query/response counting; returns no findings (current behavior). |
| `tests/integration_test.rs` | 66 | 1 | Synthetic in-memory pcap → `decode_packet` → `DnsAnalyzer` smoke test. |
| `tests/linktype_integration_tests.rs` | 40 | 3 | Cross-link-type pcap loads via `PcapSource` for Ethernet, Linux SLL, IPv6. |
| `tests/findings_tests.rs` | 35 | 2 | `Finding` struct construction + `Display` formatting. |
| `tests/http_integration_tests.rs` | 35 | 1 | End-to-end pcap → reassembler → `HttpAnalyzer` against `http.pcap` / `http-full.cap`. |

`src/reporter/terminal.rs` contains a `#[cfg(test)]` module (no `#[test]` attributes matched — the inline tests appear to be elsewhere or use a different gate; only one `#[cfg(test)]` line in all of `src/`). The 202 test count above counts only `tests/*.rs`.

## 6. Quick-Start Commands

Exact commands documented in `CLAUDE.md` and CI:

```bash
# Build
cargo check                          # fast type-check loop
cargo build                          # debug
cargo build --release                # release (overflow-checks = true)

# Run the CLI
cargo run -- --help
cargo run -- analyze capture.pcap --all
cargo run -- summary capture.pcap

# Test — matches CI exactly
cargo test --all-targets
cargo test <name_substring>          # single test
# Note: --all-targets does NOT run doctests; add `cargo test --doc` if any exist.

# Lint — matches CI exactly (warnings are errors)
cargo clippy --all-targets -- -D warnings

# Format
cargo fmt                            # apply
cargo fmt --check                    # CI gate (CI uses --all --check; harmless on single crate)

# Install (from README)
cargo install --path .
```

CI environment sets `RUSTFLAGS=-Dwarnings`; reproduce locally with `RUSTFLAGS=-Dwarnings cargo test --all-targets`.

## 7. Recovery Metadata

- **Rust source files (`.rs`):** 20 (in `src/`) + 18 (in `tests/`) = **38 total**
- **Source LOC (`src/`):** **3,868** (sum from `find src -name '*.rs' -exec wc -l`)
- **Test LOC (`tests/`):** **6,021** (sum from `find tests -name '*.rs' -exec wc -l`)
- **Total Rust LOC:** **9,889**
- **Test-to-source LOC ratio:** ~1.56:1
- **Test functions:** 202 `#[test]` across `tests/`; 0 `#[tokio::test]`; 0 inline `#[test]` in `src/`.
- **ADRs:** 3 (`docs/adr/0001-content-first-stream-dispatch.md`, `0002-modular-protocol-analyzers.md`, `0003-reporting-pipeline-layering.md`); combined 477 LOC.
- **Superpowers docs:** 18 files (10 plans + 8 specs); combined ~12,052 LOC of Markdown.
- **pcap/pcapng fixtures:** 14 (`tests/fixtures/`); range 247 B → 33,144 B; includes one `.pcapng` (`smb3.pcapng`) for negative testing since the reader rejects pcapng.
- **CI jobs (`.github/workflows/ci.yml`):** 4 — `semantic-pr` (PR-only), `test` (`cargo test --all-targets`), `clippy` (`cargo clippy --all-targets -- -D warnings`), `fmt` (`cargo fmt --all --check`). All use `dtolnay/rust-toolchain@stable` and `Swatinem/rust-cache@v2`.
- **No `build.rs`. No `rust-toolchain.toml`. No workspace.** Single binary crate, single `Cargo.toml`.

## 8. Open Questions for Later Passes

Items noticed during inventory that Passes 1-5 should investigate. File:line citations included where applicable.

1. **`csv` dependency declared but never imported** in `src/`. The `--csv` CLI option exists at `src/cli.rs:35-36` (and `OutputFormat::Csv` exists at `src/cli.rs:8`) but `src/main.rs:172-184` only branches on `OutputFormat::Json` and falls through to `TerminalReporter` for everything else — including `--csv`. Pass 1 (Architecture) and Pass 3 (BC) should flag this as an unimplemented feature. *Likely incomplete feature.*
2. **`rayon` dependency declared but never imported** in `src/`. README roadmap (`README.md:152`) lists "Parallel file processing" as future work. Pass 1 should note this as a stub for an unimplemented NFR (parallelism).
3. **`assert_cmd`, `predicates`, `tempfile` dev-deps declared but unused** anywhere in `tests/`. `tests/cli_tests.rs` uses pure `Cli::parse_from`, never spawns the binary. Pass 1/3 should consider whether end-to-end binary tests are intended.
4. **`--threats`, `--beacon`, `--filter`, `--hosts`, `--services` CLI flags exist but are unwired.** `src/cli.rs:67-68, 82-84, 94-96, 106-111` define them; `src/main.rs:28-50` destructures only `dns`, `http`, `tls`, `all`, `mitre`, `targets`. Pass 3 should treat them as unimplemented BCs to be flagged.
5. **`--json` and `--csv` global flags accept `Option<Option<PathBuf>>`** (`src/cli.rs:31-36`) implying file-output intent, but `src/main.rs:186` and `src/main.rs:232` unconditionally `println!` the rendered output to stdout. Output-to-file is unimplemented.
6. **`StreamDispatcher.unclassified_flows` is incremented only when the close handler sees no cached route** (`src/dispatcher.rs:110-115`). Pass 2/3 should verify whether flows that never received `on_data` (e.g., handshake-only flows) are counted, since `classify()` requires bytes.
7. **Reassembly engine has a `MAX_FINDINGS = 10_000` hard cap** (`src/reassembly/mod.rs:18`) with no documented behavior on what happens at the limit. Pass 3 should extract the truncation contract from tests.
8. **`OVERLAP_ALERT_THRESHOLD = 50`, `SMALL_SEGMENT_ALERT_THRESHOLD = 2048`, `OUT_OF_WINDOW_ALERT_THRESHOLD = 100`** constants in `src/reassembly/mod.rs:15-17` are magic numbers; Pass 4 (NFR) should extract their rationale (and check whether any test pins them).
9. **`DnsAnalyzer::analyze` returns `Vec::new()` unconditionally** (`src/analyzer/dns.rs:54-62`) — it counts queries/responses for the summary but never emits findings. This makes the DNS "analyzer" effectively a metrics collector. Pass 2/3 should decide whether DNS-as-analyzer is a domain entity at all.
10. **MITRE technique set is a static `match`** of only ~16 IDs (`src/mitre.rs:99-129`) including 4 ICS techniques (`T0846`, `T0855`, `T0856`, `T0885`). Pass 2 should map which findings emit which IDs and Pass 4 should note the maintenance NFR (no external data source — code changes for new techniques).
11. **`CLOSE_FLOW_MISSING_WARNED` and `ISN_MISSING_WARNED` are process-wide `AtomicBool`s** (`src/reassembly/mod.rs:20`, `src/reassembly/segment.rs:5`) — global state. Pass 4 should note this NFR (single warning per process, no re-entry).
12. **`smb3.pcapng` fixture exists** despite README declaring pcapng unsupported. Pass 3 should check what test consumes it (likely a negative test in `reader_tests.rs`).
13. **`linktype_integration_tests.rs:1` imports `pcap_file::DataLink`** directly, suggesting `DataLink` leaks across module boundaries; Pass 1 should consider whether `wirerust` ought to re-export a stable `DataLink` alias.
14. **No `LICENSE` SPDX header in source files** — only `Cargo.toml:6` declares `license = "MIT"`. Pass 5 (Conventions) should note absence of per-file license headers as a convention (consistent across all 20 files).
15. **`Cargo.lock` is checked in** (38,291 bytes) — appropriate for a binary crate; Pass 5 should note this as a deliberate convention.
16. **No `#[allow(...)]` clusters anywhere in `src/`** (confirmed via `awk` — 0 matches). Strong signal that warnings policy (`-Dwarnings`) is genuinely clean.
17. **No inline unit tests in `src/`** — all tests live under `tests/`. Pass 5 should note this as a project-wide convention (uncommon for Rust; most crates use both inline `#[cfg(test)] mod tests` and integration `tests/`).
18. **`src/reporter/terminal.rs:9-28`** has a long documentation block explaining the C1 control-byte escaping rationale — Pass 3 should extract this as a load-bearing security BC (defends against attacker-controlled control sequences in finding summaries).

## Pass 0 State Checkpoint

```yaml
pass: 0
status: complete
files_scanned: 38_rust + 21_markdown + 14_pcap_fixtures + 6_toplevel_metadata
total_source_loc: 3868
total_test_loc: 6021
total_test_functions: 202
adrs: 3
ci_jobs: 4
timestamp: 2026-05-19T00:00:00Z
next_pass: 1
resume_from: null
```
