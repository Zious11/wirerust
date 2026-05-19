# Pass 8 Deep Synthesis -- wirerust

- **Project:** wirerust
- **Source path:** /Users/zious/Documents/GITHUB/wirerust/
- **Generated:** 2026-05-19
- **Pipeline:** brownfield-ingest (in-repo: target == reference)
- **Inputs ingested:** 20 prior artifacts (7 Phase A + 11 Phase B deepening + 2 Phase B.5/B.6 audits)
- **Supersedes:** wirerust-pass-6-synthesis.md (Phase A R1 synthesis -- carries uncorrected metrics)
- **Confidence summary:**
  - Inventory completeness: HIGH (P0 R2 fully re-verified)
  - Architecture accuracy: HIGH (P1 R3 converged; 10 smells catalogued)
  - Domain model accuracy: HIGH structural / HIGH-MEDIUM behavioral (P2 R3 converged)
  - Behavioral contract coverage: HIGH (218 BCs; 74% HIGH-confidence; all 10 ABS dispositioned)
  - NFR coverage: HIGH (79 NFRs; resource bounds + security HIGH; perf MEDIUM)
  - Convention coverage: HIGH (90 conventions)
  - Cross-pass consistency: HIGH (B.5 PASS / 0 blind spots; B.6 PASS / 18-20 confirmed, 0 hallucinated)

## 2. Project at a glance

### What wirerust does

wirerust is an offline, single-binary, single-pass forensic triage tool that ingests classic-pcap captures and emits structured findings about HTTP/TLS/DNS traffic plus TCP stream anomalies. It is a hybrid binary+library Rust 2024 crate (3,868 LOC of src + 6,021 LOC of integration tests; 213 #[test] functions total) with zero network I/O, zero `unsafe` blocks, zero `#[allow]` clusters, and zero use of tokio/async -- all tests use the stdlib `#[test]` runner. Its product identity is "trustworthy forensic data preservation + display-layer safety": raw attacker-controlled bytes survive intact through the analyzers and JSON output, while the terminal reporter is the sole owner of C0+DEL+C1+backslash escaping (ADR 0003).

### Architecture in one paragraph

The pipeline is a strict 5-layer stack of 20 components: **L0 Entry** (main.rs/lib.rs/cli.rs -- C-1..C-3) parses CLI, owns the per-target packet loop and stdout-only output; **L1 Ingest** (reader.rs/decoder.rs -- C-4..C-5) is the L2-L4 boundary that gates the 5-element link-type whitelist and produces ParsedPackets; **L2 Stream/Routing** (reassembly/{mod,flow,segment,handler}.rs + dispatcher.rs -- C-6..C-9, C-15) holds the most state-heavy code -- a 564-LOC reassembly engine with first-wins overlap policy, MAX_FINDINGS=10000 cap, content-first dispatch (ADR 0001), and the StreamHandler/StreamAnalyzer trait pair that is the only L2->L3 upward trait coupling in the tree; **L3 Domain Analysis** (analyzer/{dns,http,tls,mod}.rs + findings.rs + mitre.rs + summary.rs -- C-10..C-14, C-16..C-17) owns three analyzers plus the load-bearing Finding schema; **L4 Output** (reporter/{mod,json,terminal}.rs -- C-18..C-20) renders terminal-with-escaping and JSON-via-serde (CSV declared but unwired). File-level DAG is acyclic; a single advisory module-group cycle exists between analyzer and reassembly via the StreamAnalyzer trait returning AnalysisSummary/Finding, formalized by ADR 0002.

### The 5 most architecturally-significant invariants

1. **FlowKey canonical ordering (VO-1, flow.rs:34)** -- `(ip_a, port_a) <= (ip_b, port_b)` tuple-pair comparison merges directions of the same connection into one key. Sorting fields independently would silently merge unrelated flows. Tests: reassembly_flow_tests.rs (`test_flow_key_canonicalization`).
2. **Content-first dispatch precedence (ADR 0001 / BC-DSP-001..004)** -- TLS `0x16 0x03` and HTTP method tokens beat port numbers; ports 80/443/8080/8443 are pure fallback. `DispatchTarget::None` is uncached to allow reclassification on subsequent on_data. This is the dispatch identity.
3. **First-wins overlap policy (BC-RAS-036/037/018)** -- New TCP segments inserting only gap bytes; conflicting overlap (full-coverage, different bytes) emits Anomaly/Likely/High with MITRE T1036. This is the forensic-truth principle.
4. **Raw-data layer / display-layer escaping (ADR 0003 / BC-FND-005 / BC-RPT-001..012)** -- Finding.summary and Finding.evidence carry raw post-from_utf8_lossy bytes; escape_for_terminal is the only terminal-safe primitive; JSON delegates to serde_json RFC 8259. This is the security identity.
5. **SNI conformance 4-way classification (BC-TLS-014..020)** -- The SniValue enum (Ascii/AsciiWithControl/NonAsciiUtf8/NonUtf8) is the load-bearing covert-channel detector; 3 of 4 buckets emit T1027 findings. is_ascii() is the controlling gate (mixed ASCII+UTF-8 with controls routes to NonAsciiUtf8, not AsciiWithControl).

### Three ADRs and what each pins

- **ADR 0001 (Accepted 2026-04-07): Content-first stream dispatch.** Pins 5-byte content signature precedence over port number. Rejected alternatives: broadcast-to-all + lazy-buffer, or pure-port dispatch.
- **ADR 0002 (Accepted 2026-04-07): Modular protocol analyzer pattern.** Pins the two-trait design (ProtocolAnalyzer for packet-level; StreamHandler+StreamAnalyzer for stream-level), the universal AnalysisSummary payload, and MAX_FINDINGS/MAX_MAP_ENTRIES cardinality discipline. Rejected alternative: single mega-trait.
- **ADR 0003 (Accepted 2026-04-09): Reporting pipeline layering.** Pins "data layer raw, display layer formats" -- analyzers store raw bytes; only TerminalReporter escapes; JsonReporter delegates to serde. Rejected alternative: PR #49's construction-site sanitization which destroyed forensic data via {:?} Debug-format.

### Top 3 cross-cutting findings from deepening

1. **Smell #9 broadens beyond panic** (P1 R3) -- The lack of `impl Drop` on TcpReassembler means panic-unwind AND ordinary `?`-Err propagation skip finalize(); buffered findings (incl. BC-RAS-054 segment-limit summary) are silently lost on multi-target invocations.
2. **MAX_FINDINGS=10000 silently drops with no observability counter** (P3 R3) -- The engine drops findings at the cap with no observability; design for a `dropped_findings: u64` counter on ReassemblyStats is ready (~12 LOC).
3. **Finding.timestamp universally None despite data availability** (P2 R3) -- All 22 production emission sites pass `timestamp: None`; `RawPacket.timestamp_secs` is read but never threaded through to Finding construction. Forensic provenance gap.

## 3. Definitive metric table (authoritative)

| Metric | Value | Source |
|---|---|---|
| Rust source files (src/) | 20 | P0 confirmed |
| Test source files (tests/) | 18 | P0 confirmed |
| Source LOC | 3,868 | P0 confirmed |
| Test LOC | 6,021 | P0 confirmed |
| Total LOC | 9,889 | P0 confirmed |
| `#[test]` functions in tests/ | 202 | P0 |
| `#[test]` functions inline in src/ (reporter/terminal.rs:265-341) | 11 | P0 R2 (R1 missed these) |
| **Total `#[test]` functions** | **213** | **P0 R2 corrected from R1's 202** |
| Behavioral contracts total | **218** (216 R2 + BC-RAS-054 + BC-TLS-037) | P3 R2/R3/R4 corrected from R1's 137 |
| ... HIGH confidence | 162 (74%) | P3 R2 |
| ... MEDIUM confidence | 40 (with R3/R4 upgrades, ~35 net remaining) | P3 |
| ... LOW confidence | 4 | P3 |
| ... ABSENT (BC-ABS-001..010) | 10 (all dispositioned) | P3 R2/R3 |
| Domain entities | 41 | P2 |
| Domain enums (semantic) | 14 | P2 |
| Business rules in P2 | 101 | P2 |
| State machines | 5 | P2 |
| Components | 20 (C-1..C-20) | P1 |
| Layers | 5 | P1 |
| Architecture smells | 10 (P1 R1's 8 + R2's #9 no-Drop + #10 loose TLS gate) | P1 R2 |
| NFRs total | **79** (P4 R1's 76 + R2's 3 new: OBS-010, RES-022, RES-023) | P4 R2 |
| Magic numbers (de-duplicated) | ~31 | P4 R2 corrected from 28 |
| Saturating arithmetic sites | 12 | P4 R2 corrected from 13 |
| Conventions total | **90** (R1's 85 + 5 net new; CNV-FMT-008 subsumed) | P5 R3 corrected from R1's 73 and R2's 91 |
| MITRE techniques catalogued in technique_info | 15 | P2 R2 corrected from 16 |
| MITRE techniques emitted | 6 (T1027, T1036, T1046, T1083, T1499.002, T1505.003) | P2 R2 |
| MITRE techniques unused in catalogue | 9 (T1040, T1071, T1071.001, T1071.004, T1573, T0846, T0855, T0856, T0885) | P2 R2 corrected from 4 |
| MitreTactic enum variants | 16 (14 Enterprise + 2 ICS-unique) | P2 |
| pcap fixtures total | 14 | P0 |
| pcap fixtures consumed | **6** (43%) | B.5 corrected from P0 R2 prose "5" |
| pcap fixtures dead | 8 | P0 R2 / B.5 |
| `unsafe` blocks in src/ | 0 | P1 / P4 |
| `#[allow(...)]` in src/ | 0 | P5 R3 |
| `impl Drop` in src/ | 0 | P2 R3 / P5 R3 |
| ADRs | 3 (0001/0002/0003) | P0 |
| CI jobs | 4 (semantic-pr, test, clippy, fmt) | P0 |
| Cargo.lock bytes | 38,291 | P0 R2 |
| Direct production deps | 14 (11 active + 3 unused: csv, rayon, partial Csv) | P0 |
| Dev-deps | 3 (assert_cmd, predicates, tempfile -- all unused) | P0 |

## 4. Architecture overview

### Components by layer

| Layer | Component IDs | Files | Criticality (P1 R2) |
|---|---|---|---|
| L0 Entry | C-1 main, C-2 lib, C-3 cli | main.rs, lib.rs, cli.rs | HIGH (C-1, C-2); MEDIUM (C-3) |
| L1 Ingest | C-4 reader, C-5 decoder | reader.rs, decoder.rs | HIGH both |
| L2 Stream/Routing | C-6 reassembly engine, C-7 flow, C-8 segment, C-9 handler, C-15 dispatcher | reassembly/{mod,flow,segment,handler}.rs, dispatcher.rs | HIGH all |
| L3 Domain Analysis | C-10 findings, C-11 mitre, C-12 analyzer trait, C-13 dns, C-14 http, C-16 tls, C-17 summary | findings.rs, mitre.rs, analyzer/{mod,dns,http,tls}.rs, summary.rs | CRITICAL (C-10); HIGH (C-12); MEDIUM (C-11, C-14, C-16, C-17); LOW (C-13) |
| L4 Output | C-18 reporter trait, C-19 json, C-20 terminal | reporter/{mod,json,terminal}.rs | MEDIUM all |

### Module-group cycle (advisory)

analyzer <-> reassembly via the StreamAnalyzer trait. `reassembly::handler` (L2) returns `AnalysisSummary` (L3) and `Vec<Finding>` (L3); HttpAnalyzer/TlsAnalyzer (L3) import `reassembly::handler::{StreamAnalyzer,...}` and `reassembly::flow::FlowKey`. File-level DAG is acyclic; cycle is module-group only. Documented and accepted by ADR 0002.

### Architecture smells (10)

| # | Smell | Severity | Where |
|---|---|---|---|
| 1 | reassembly/mod.rs is a 565-LOC god-module | medium | mod.rs |
| 2 | Process-wide one-shot AtomicBool warning guards | low | mod.rs:20, segment.rs:5 |
| 3 | 7 CLI flags declared but unwired | high | cli.rs vs main.rs:28-50 |
| 4 | L2->L3 trait coupling (intrinsic to ADR 0002) | advisory | handler.rs:1-2 |
| 5 | DnsAnalyzer::analyze returns empty Vec unconditionally | low | dns.rs:54-62 |
| 6 | StreamDispatcher pub field exposure | low | dispatcher.rs:17-18 |
| 7 | pcap_file::DataLink leaks across crate boundary | low | decoder.rs/reader.rs/test imports |
| 8 | csv + rayon declared, never imported | low | Cargo.toml |
| 9 | **No `impl Drop` on TcpReassembler -- panic AND `?`-Err skip finalize** (P1 R2 + R3) | high | mod.rs |
| 10 | **Loose TLS gate (5-byte signature accepts data[2] up to 0xFF)** (P1 R2) | low (theoretical) | dispatcher.rs:39-41 |

## 5. Behavioral contract corpus summary

| Area | BC count | HIGH | MEDIUM | LOW | ABS |
|---|---|---|---|---|---|
| BC-RDR-* (reader) | 8 | 5 | 3 | 0 | 0 |
| BC-DEC-* (decoder) | 15 | 7 | 8 | 0 | 0 |
| BC-RAS-* (reassembly) -- includes new BC-RAS-054 | 54 | 41 | 11 | 2 | 0 |
| BC-DSP-* (dispatcher) | 9 | 4 | 5 | 0 | 0 |
| BC-HTTP-* (http analyzer) | 26 | 22 | 4 | 0 | 0 |
| BC-TLS-* (tls analyzer) -- includes new BC-TLS-037 | 37 | 33 | 4 | 0 | 0 |
| BC-DNS-* (dns analyzer) | 4 | 4 | 0 | 0 | 0 |
| BC-MIT-* (mitre) | 9 | 8 | 0 | 1 | 0 |
| BC-FND-* (findings) | 6 | 5 | 1 | 0 | 0 |
| BC-RPT-* (reporters) | 19 | 15 | 3 | 1 | 0 |
| BC-CLI-* (cli + main) | 17 | 7 | 10 | 0 | 0 |
| BC-SUM-* (summary) | 4 | 4 | 0 | 0 | 0 |
| BC-ABS-* (absent / unwired) | 10 | n/a | n/a | n/a | 10 |
| **Total** | **218** | **155** | **49** | **4** | **10** |

## 6. Most-significant findings (top 10 by impact)

1. **Module-group cycle at analyzer <-> reassembly via StreamAnalyzer trait.** Advisory; documented in ADR 0002.
2. **MAX_FINDINGS=10000 silently drops with no observability counter** (P3 R3). NFR-RES-022 design ready.
3. **Finding.timestamp is universally None despite data availability** (P2 R3). 22 emission sites verified.
4. **Asymmetric Option JSON serialization** (BC-FND-006 / NFR-OBS-010). Only timestamp skips.
5. **Inverted missing-Host vs missing-UA contracts** (P2 R3 / CNV-PAT-001).
6. **BC-RAS-022 latches are per-direction** -- up to 6 alerts/flow (3 types x 2 directions), not 3.
7. **smb3.pcapng added "for future pcapng support"** (P0 R2). R1 negative-test hypothesis refuted.
8. **MSRV effectively 1.86 but Cargo.toml silent** (NFR-VIO-009). floor_char_boundary at http.rs:97.
9. **10 absent BCs all dispositioned** (P3 R2/R3 -- wire/remove/error-with-msg).
10. **Zero `impl Drop` in src/** -- panic-safety is structural-only; `?`-Err propagation also skips finalize.

## 7. Convergence report

| Pass | Rounds | Trajectory | Final state |
|---|---|---|---|
| 0 | 3 (R1+R2+R3 via B.5/B.6) | broad -> SUBSTANTIVE -> NITPICK | CONVERGED |
| 1 | 3 (R1+R2+R3) | broad -> SUBSTANTIVE -> NITPICK | CONVERGED |
| 2 | 4 (R1+R2+R3+R4) | broad -> SUBSTANTIVE -> SUBSTANTIVE -> NITPICK | CONVERGED |
| 3 | 5 (R1+R2+R3+R4+R5) | broad -> SUBSTANTIVE -> SUBSTANTIVE -> SUBSTANTIVE -> NITPICK | CONVERGED |
| 4 | 3 (R1+R2+R3) | broad -> SUBSTANTIVE -> NITPICK | CONVERGED |
| 5 | 4 (R1+R2+R3+R4) | broad -> SUBSTANTIVE -> SUBSTANTIVE -> NITPICK | CONVERGED |

Total agent dispatches in Phase B: ~14 (plus 1 retry on P1 R2). Phase B.5 coverage audit: PASS, 0 blind spots. Phase B.6 extraction validation: PASS, 18/20 CONFIRMED, 0 hallucinated.

## 8. Priority-ordered lessons for wirerust (P0/P1/P2/P3)

### P0 -- Correctness gaps

**LESSON-P0.01: Declare effective MSRV in Cargo.toml.**
- **(a) What wirerust does today:** Cargo.toml declares `edition = "2024"` but no `rust-version`. http.rs:97 uses floor_char_boundary (Rust 1.86).
- **(b) What it should do:** Declare `rust-version = "1.86"` in `[package]` with comment citing http.rs:97.
- **(c) The gap:** rustc 1.85 users get confusing stdlib-pointing errors.
- **(d) Action items:** Single-line addition to /Users/zious/Documents/GITHUB/wirerust/Cargo.toml.

**LESSON-P0.02: Resolve pcapng glob vs reader rejection inconsistency.**
- **(a) What wirerust does today:** main.rs:245-247 collects both `*.pcap` AND `*.pcapng`, but reader.rs:22 rejects pcapng with generic message; error counter increments silently after first.
- **(b) What it should do:** Either remove pcapng from glob, OR emit a distinct "pcapng not yet supported" per-file error.
- **(c) The gap:** Directory targets with mixed formats produce confusing generic errors.
- **(d) Action items:** Edit /Users/zious/Documents/GITHUB/wirerust/src/main.rs:245-247 (remove `|| ext == "pcapng"`).

**LESSON-P0.03: Add Drop guard so finalize() runs on panic AND `?`-Err.**
- **(a) What wirerust does today:** TcpReassembler has no impl Drop. Three `?` sites in main.rs (lines 102, 105, 110) fire AFTER reassembler construction and BEFORE finalize. On multi-target runs, errors lose buffered findings AND the render never runs.
- **(b) What it should do:** Add `impl Drop for TcpReassembler` that flushes internal findings OR wrap the per-target loop in an RAII guard.
- **(c) The gap:** Smell #9 broadened to cover `?`-Err propagation (more common than panic).
- **(d) Action items:** Add impl Drop to /Users/zious/Documents/GITHUB/wirerust/src/reassembly/mod.rs (~10-15 LOC) + regression test.

**LESSON-P0.04: Wire `--csv <FILE>` and `--json <FILE>` to fs::write OR remove the FILE shape.**
- **(a) What wirerust does today:** cli.rs:31-36 declares Option<Option<PathBuf>> for both; main.rs:186, 232 unconditionally `println!`. BC-ABS-006/007. OutputFormat::Csv silently falls through to terminal (BC-CLI-016).
- **(b) What it should do:** Match `Some(Some(path)) => fs::write(path, rendered)?` (~6 LOC), OR remove inner Option.
- **(c) The gap:** User runs `wirerust --csv out.csv` and no file is written.
- **(d) Action items:** Edit /Users/zious/Documents/GITHUB/wirerust/src/main.rs:172-232; add tests in tests/cli_tests.rs.

**LESSON-P0.05: Fix inverted missing-Host vs missing-UA semantics.**
- **(a) What wirerust does today:** http.rs:250-262 fires only when host is None (empty Host: header passes); http.rs:278-290 fires only when UA is present-but-empty (absent UA passes).
- **(b) What it should do:** Adopt single canonical missing semantic; both checks should fire on `is_none() || value.trim().is_empty()`.
- **(c) The gap:** Attacker defeats missing-Host with `Host:` empty value; defeats missing-UA by omitting header.
- **(d) Action items:** Edit /Users/zious/Documents/GITHUB/wirerust/src/analyzer/http.rs:250-290.

### P1 -- High-ROI improvements

**LESSON-P1.01: Add `dropped_findings: u64` counter to ReassemblyStats.**
- **(a) What wirerust does today:** MAX_FINDINGS=10_000 cap at mod.rs:18; findings past cap silently dropped (guards at 272, 291, 310, 534, 550); no counter increments.
- **(b) What it should do:** Add dropped_findings field to ReassemblyStats; increment on each guarded path; surface in summarize() detail.
- **(c) The gap:** NFR-RES-022. Operators have no signal when adversarial input hits the cap.
- **(d) Action items:** Edit /Users/zious/Documents/GITHUB/wirerust/src/reassembly/mod.rs (~12 LOC) + test.

**LESSON-P1.02: Symmetrize Finding Option JSON serialization.**
- **(a) What wirerust does today:** findings.rs:60-70 -- only timestamp has skip_serializing_if; mitre_technique and source_ip always serialize, even as null.
- **(b) What it should do:** Add skip_serializing_if to mitre_technique and source_ip OR document the asymmetry deliberately.
- **(c) The gap:** Downstream JSON consumers handle both schemas for sibling Option fields.
- **(d) Action items:** Edit /Users/zious/Documents/GITHUB/wirerust/src/findings.rs:66-69 (2 attribute lines).

**LESSON-P1.03: Wire `--hosts` flag (data exists in Summary.host set).**
- **(a) What wirerust does today:** cli.rs:106-107 declares hosts: bool; main.rs:190-233 ignores it. BC-ABS-004. Summary.hosts already aggregates the data.
- **(b) What it should do:** When --hosts set, render the host set with per-host packet counts (upgrade Summary.hosts to HashMap<IpAddr, u64>).
- **(c) The gap:** Documented CLI feature returns no extra information.
- **(d) Action items:** Edit /Users/zious/Documents/GITHUB/wirerust/src/summary.rs:14 + src/main.rs:190-233 (~15 LOC).

**LESSON-P1.04: Codify "no unwired CLI flags" convention.**
- **(a) What wirerust does today:** 8 flags declared but unwired: `--threats`, `--beacon`, `--filter`, `--verbose`, `--hosts`, `--services`, `--json <FILE>`, `--csv <FILE>`. CNV-CLI-001 / Smell #3.
- **(b) What it should do:** Convention: every CLI flag is wired OR hidden with `#[arg(hide = true)]` + tracking issue.
- **(c) The gap:** --help advertises behavior the binary cannot deliver.
- **(d) Action items:** Edit /Users/zious/Documents/GITHUB/wirerust/src/cli.rs to hide unwired flags; document in CLAUDE.md.

**LESSON-P1.05: Add TLS truncation counter (CNV-PAT-002 follow-up).**
- **(a) What wirerust does today:** HttpAnalyzer has poisoned_bytes_skipped: u64; TlsAnalyzer has no analogous counter. At least one TLS silent-drop site exists.
- **(b) What it should do:** Add truncated_records: u64 to TlsAnalyzer struct; increment at each silent record-discard site.
- **(c) The gap:** CNV-PAT-002 P5 R3 retraction: TLS does not yet conform to silent-drop instrumentation.
- **(d) Action items:** Edit /Users/zious/Documents/GITHUB/wirerust/src/analyzer/tls.rs (~10 LOC + 1 test). NFR-RES-023.

**LESSON-P1.06: Enable `#![warn(missing_docs)]` on lib.rs with phased rollout.**
- **(a) What wirerust does today:** 10 of 20 src files have zero `///` doc lines; reassembly/flow.rs has 47 pub items with zero docs.
- **(b) What it should do:** Add #![warn(missing_docs)] + temporary #![allow(missing_docs)] to lib.rs; phase out the allow as coverage reaches 100%.
- **(c) The gap:** CNV-DOC-004 drift hotspot.
- **(d) Action items:** Edit /Users/zious/Documents/GITHUB/wirerust/src/lib.rs (2 lines); backfill incrementally.

**LESSON-P1.07: Add `//!` module headers to all 20 modules.**
- **(a) What wirerust does today:** Only mitre.rs:1-13 carries a //! header (CNV-DOC-005).
- **(b) What it should do:** Add 1-3 line //! headers to each module (P5 R2 drafted 20 templates).
- **(c) The gap:** Module-level intent is implicit.
- **(d) Action items:** Edit all 20 /Users/zious/Documents/GITHUB/wirerust/src/**/*.rs files.

### P2 -- Worth considering

**LESSON-P2.01: Refactor reassembly/mod.rs (565 LOC god-module).** Split into mod.rs(~40) + engine.rs(~440) + policy.rs(~85) per P5 R2 Target 7. Edit /Users/zious/Documents/GITHUB/wirerust/src/reassembly/mod.rs.

**LESSON-P2.02: Convert ~20 positional format-string sites to inline-capture form.** CNV-FMT-007 violations; one-shot rewrite across /Users/zious/Documents/GITHUB/wirerust/src/**.

**LESSON-P2.03: Implement CsvReporter OR remove OutputFormat::Csv + csv crate.** BC-CLI-016 / BC-ABS-007. Edit /Users/zious/Documents/GITHUB/wirerust/Cargo.toml + src/cli.rs + src/main.rs:172-184.

**LESSON-P2.04: Add property tests for JA3/JA3S GREASE filtering.** Add proptest = "1" dev-dep; new tests/tls_property_tests.rs.

**LESSON-P2.05: Calibrate alert thresholds against benign captures.** OVERLAP/SMALL_SEG/OUT_OF_WIN have no empirical justification (P4 R2). Calibrate against CTU-13 / CICIDS-2017 to find 99th-percentile floors. Edit /Users/zious/Documents/GITHUB/wirerust/src/reassembly/mod.rs:15-17.

**LESSON-P2.06: Add `cargo audit` / `cargo deny` CI job.** Edit /Users/zious/Documents/GITHUB/wirerust/.github/workflows/ci.yml.

**LESSON-P2.07: Add criterion benchmarks for reassembly hot paths.** Add /Users/zious/Documents/GITHUB/wirerust/benches/reassembly.rs.

**LESSON-P2.08: Add direction tag (c2s/s2c) to Finding payload.** Edit /Users/zious/Documents/GITHUB/wirerust/src/findings.rs + 22 emission sites (non-breaking via Option default).

**LESSON-P2.09: Sort JSON map keys deterministically (Q-A5).** Edit /Users/zious/Documents/GITHUB/wirerust/src/reporter/json.rs:18-22 to sort or switch upstream to BTreeMap.

**LESSON-P2.10: Make ThreatCategory `#[non_exhaustive]` OR close-and-document.** Edit /Users/zious/Documents/GITHUB/wirerust/src/findings.rs:41.

**LESSON-P2.11: Add `max_classification_attempts` knob to dispatcher (Q-A7).** Edit /Users/zious/Documents/GITHUB/wirerust/src/dispatcher.rs:15-117.

### P3 -- Known divergences to document

**LESSON-P3.01: Document `Summary.services` port-based vs dispatcher content-first split.** Add doc comment at /Users/zious/Documents/GITHUB/wirerust/src/summary.rs:8-16.

**LESSON-P3.02: Document the inline pluralization ternary at reassembly/mod.rs:415.** Either extract to helper or codify the form in CLAUDE.md.

**LESSON-P3.03: Document process-wide one-shot atomics in a new ADR 0004.** Multi-target visibility limitation. Add /Users/zious/Documents/GITHUB/wirerust/docs/adr/0004-process-wide-warning-guards.md.

**LESSON-P3.04: Document 9 catalogued-but-unused MITRE IDs.** Edit /Users/zious/Documents/GITHUB/wirerust/src/mitre.rs:92-129 doc comment listing T1040/T1071/T1071.001/T1071.004/T1573/T0846/T0855/T0856/T0885 as "staged for future analyzers."

**LESSON-P3.05: Codify the test-naming style transition (prose-style canonical).** Edit /Users/zious/Documents/GITHUB/wirerust/CLAUDE.md.

**LESSON-P3.06: Widen branch-naming patterns to `<type>/<slug>` in CLAUDE.md.** Per P5 R2 Target 4. Edit /Users/zious/Documents/GITHUB/wirerust/CLAUDE.md Git Workflow section.

**LESSON-P3.07: Document dead fixtures (8 of 14) explicitly.** Add /Users/zious/Documents/GITHUB/wirerust/tests/fixtures/README.md OR `git rm` the 8 unconsumed files.

## 9. Confidence assessment

| Dimension | Confidence | Basis |
|---|---|---|
| Architecture | HIGH | 20 components grounded in file:line; cycle scan verified |
| Domain model (structural) | HIGH | 41 entities + 14 enums + 12 VOs cited |
| Domain model (behavioral) | HIGH-MEDIUM | 101 BRs from code; P2 R3 fully traced 22 emission sites + SniValue precedence |
| Behavioral contracts | HIGH | 218 BCs; 74% HIGH-confidence; B.6 confirmed 18/20 sampled |
| NFRs (resource + security) | HIGH | 12 saturating sites, 31 magic numbers, 5 ADR-grounded; P4 R2 provenance research |
| NFRs (performance) | MEDIUM | No benchmarks exist; LESSON-P2.07 addresses |
| NFRs (observability) | MEDIUM | 9 + 1 NFR-OBS; no structured logging |
| Conventions | HIGH | 90 catalogued; 32 CI-gated, 58 manual |
| Cross-pass consistency | HIGH | B.5 PASS (0 blind spots); 7 R1->R2/R3 corrections durably present |
| Coverage of source files | HIGH | All 20 src files COVERED |
| Hallucination risk | LOW | B.6 PASS, ~97% extraction accuracy |

## 10. Known unknowns (18 items from P6 R1 re-verified)

1. csv dep declared unused -- RESOLVED (LESSON-P2.03).
2. rayon dep declared unused -- RESOLVED (recommend remove).
3. dev-deps unused -- RESOLVED (NFR-VIO-007; write E2E test or drop).
4. unwired CLI flags -- RESOLVED (BC-ABS-001..005; LESSON-P1.04).
5. --json / --csv file output -- RESOLVED (LESSON-P0.04).
6. unclassified_flows for handshake-only flows -- RESOLVED (P1 R2: counted as unclassified; metric is operator-misleading).
7. MAX_FINDINGS truncation contract -- RESOLVED (LESSON-P1.01).
8. Threshold rationale -- RESOLVED (LESSON-P2.05 covers calibration).
9. DnsAnalyzer emits no findings -- KNOWN (Smell #5; LOW severity).
10. MITRE technique set is 15 (not 16) IDs; 6 emitted, 9 unused -- RESOLVED.
11. Process-wide atomics -- RESOLVED (LESSON-P3.03 documents).
12. smb3.pcapng "negative test" -- REFUTED (added for future pcapng support).
13. pcap_file::DataLink leak -- KNOWN (Smell #7; advisory).
14. No per-file license headers -- KNOWN (CNV-DOC-007).
15. Cargo.lock checked in -- KNOWN (CNV-DEP-002; deliberate).
16. Zero #[allow] in src/ -- VERIFIED (P5 R3).
17. No inline unit tests -- REFUTED (P0 R2: 11 inline tests in reporter/terminal.rs).
18. C1 escape doc in terminal.rs -- KNOWN.

Resolution rate: 18/18.

## 11. Recommended next steps

**Direct paths forward:**

1. **`/create-brief`** -- product brief using §2 as seed.
2. **`/create-domain-spec`** -- L2 domain spec using §5 (218 BC corpus) + §4 (architecture) + §6 (cross-cutting).
3. **`/create-prd`** -- L3 PRD using §8 (priority-ordered lessons) directly as backlog.

**Direct-to-story alternative:** For wirerust specifically, many open engineering decisions (CNV-PAT-001, CNV-FMT-009, P0/P1 lessons) could be handled via direct story decomposition rather than full L2/L3 ceremony. The P0/P1 lesson set is essentially a ready-to-execute story backlog with file paths.

**Phase order:** (1) Address P0 lessons (5 stories; all S/M cost; CI-gated). (2) Apply P1 lessons (7 stories). (3) Then proceed to /create-brief and /create-prd for net-new features.

## 12. State checkpoint (final)

```yaml
pipeline: brownfield-ingest
mode: in-repo (target == reference)
phase: complete
phase_b_passes_converged: 6 of 6
phase_b5_coverage: PASS (0 blind spots)
phase_b6_validation: PASS (18/20 CONFIRMED, 0 HALLUCINATED)
phase_c_synthesis: complete
total_input_files_ingested: 20
total_artifact_count: 21 (this + 20 inputs)
total_commits_on_factory_artifacts: ~24
final_state: ready for /create-brief or /create-prd
```

