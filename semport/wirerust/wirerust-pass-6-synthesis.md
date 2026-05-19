# Pass 6: Unified Synthesis & Gap Report -- wirerust

- **Project:** wirerust
- **Source path:** `/Users/zious/Documents/GITHUB/wirerust/`
- **Generated:** 2026-05-19
- **Pass:** 6 (Synthesis) -- Phase A broad-sweep, FINAL convergence anchor (round 1)
- **Inputs ingested:** 5 prior pass files + 3 ADRs = 8 source documents
  - `wirerust-pass-0-inventory.md` (310 LOC; 38 .rs files; 9889 LOC; 202 tests; 18 carryover items)
  - `wirerust-pass-1-architecture.md` (782 LOC; 20 components C-1..C-20; 5 layers; 8 architecture smells; 8 architecture-level open questions Q-A1..Q-A8)
  - `wirerust-pass-2-domain-model.md` (724 LOC; 41 entities; 14 enums; 12 value objects; 10 state containers; 4 traits; 101 business rules; 5 state machines; 8 unenforced invariants, 3 flagged top-priority)
  - `wirerust-pass-3-behavioral-contracts.md` (1722 LOC; 137 BCs across 13 areas; 10 absent BCs; 81% HIGH confidence)
  - `wirerust-pass-4-nfr-catalog.md` (394 LOC; 76 NFRs across 9 categories; 28 magic numbers indexed; 10 NFR violations recorded incl. hidden MSRV 1.86)
  - `wirerust-pass-5-conventions.md` (426 LOC; 73 conventions across 10 categories; 18 counter-examples; 15 design patterns; 11 inconsistencies)
  - ADR 0001 (content-first stream dispatch), ADR 0002 (modular analyzer pattern), ADR 0003 (reporting pipeline layering)
- **Confidence summary** (full table at §9):
  - Inventory completeness: **HIGH**
  - Architecture accuracy: **HIGH**
  - Domain model accuracy: **HIGH** (structural) / **MEDIUM-HIGH** (behavioral)
  - BC coverage: **MEDIUM-HIGH** (137 BCs; 81% HIGH-confidence; 10 absent BCs)
  - NFR coverage: **HIGH** (resource bounds, security) / **MEDIUM** (perf, observability)
  - Convention coverage: **HIGH** (73 conventions; 32 CI-enforced; 14 drifting)
  - Cross-cutting consistency: **HIGH** with minor drift (see §3)

---

## 1. Project at a Glance (1 page)

### What wirerust does (3 sentences)

wirerust is an offline, single-binary, single-pass forensic triage tool that ingests classic-pcap captures and emits structured findings about HTTP/TLS/DNS traffic plus TCP stream anomalies. It is a hybrid binary+library Rust 2024 crate (3,868 LOC of `src/` + 6,021 LOC of integration tests) with zero network I/O, zero `unsafe` blocks, and zero use of `tokio`/async -- all 202 tests use the stdlib `#[test]` runner. Its product identity is "trustworthy forensic data preservation + display-layer safety": raw attacker-controlled bytes survive intact through the analyzers and JSON output, while the terminal reporter is the sole owner of C0+DEL+C1+backslash escaping (ADR 0003).

### Architecture in one paragraph

The pipeline is a strict 5-layer stack: **L0 Entry** (`main.rs`/`lib.rs`/`cli.rs` -- C-1..C-3) parses CLI, owns the per-target packet loop and stdout-only output; **L1 Ingest** (`reader.rs`/`decoder.rs` -- C-4..C-5) is the L2-L4 boundary that gates the 5-element link-type whitelist and produces `ParsedPacket`s; **L2 Stream/Routing** (`reassembly/{mod,flow,segment,handler}.rs` + `dispatcher.rs` -- C-6..C-9, C-15) holds the most state-heavy code -- a 564-LOC reassembly engine with first-wins overlap policy, `MAX_FINDINGS=10000` cap, content-first dispatch (ADR 0001), and the `StreamHandler`/`StreamAnalyzer` trait pair that is the only L2-->L3 upward trait coupling in the tree; **L3 Domain Analysis** (`analyzer/{dns,http,tls,mod}.rs` + `findings.rs` + `mitre.rs` + `summary.rs` -- C-10..C-14, C-16..C-17) owns three analyzers (DNS counter-only, HTTP stream-level with poison threshold, TLS stream-level with JA3/JA3S) plus the load-bearing `Finding` schema; **L4 Output** (`reporter/{mod,json,terminal}.rs` -- C-18..C-20) renders to two implemented formats (terminal-with-escaping, JSON-via-serde) with `--csv` declared but unwired. File-level DAG is acyclic (CI is green); a single advisory module-group "soft cycle" exists between `analyzer` and `reassembly` via the `StreamAnalyzer` trait returning `AnalysisSummary`/`Finding`, formalized and accepted by ADR 0002.

### The 5 most important behavioral contracts (cross-pass identity)

If any of these were changed, product identity changes. All are HIGH-confidence and test-pinned.

1. **BC-DSP-001..004 (content-first dispatch, ADR 0001):** TLS `0x16 0x03` and HTTP method tokens beat port numbers; port 80/443/8080/8443 are pure fallback. Reclassification on subsequent `on_data` is allowed for `DispatchTarget::None` (uncached). Tests: `dispatcher_tests.rs`. *This is the dispatch identity of the tool.*
2. **BC-RAS-036/037/018 (first-wins overlap policy, forensic invariant):** When a new TCP segment overlaps existing committed bytes, only gap bytes are inserted; existing bytes are preserved. Conflicting overlap (full coverage, different bytes) emits an `Anomaly/Likely/High` finding with MITRE T1036. Tests: `reassembly_segment_tests.rs`. *This is the forensic-truth principle.*
3. **BC-RAS-003 / VO-1 (FlowKey canonicalization):** The `(ip_a, port_a) <= (ip_b, port_b)` tuple-pair comparison merges directions of the same connection into one key. Sorting fields independently would silently merge unrelated flows -- the inline comment at `flow.rs:32-34` calls this out. *This is the only correctness pin against a class of silent merging defects.*
4. **BC-FND-005 / BC-RPT-001..012 (raw-data layer / display-layer escaping, ADR 0003):** `Finding.summary` and `Finding.evidence` carry raw post-`from_utf8_lossy` bytes; `escape_for_terminal` (the only terminal-safe primitive) escapes C0+DEL+C1+`\` while preserving valid Unicode; JSON reporter delegates to `serde_json` RFC 8259. *This is the security identity.*
5. **BC-TLS-014..020 (SNI conformance 4-way classification + MITRE T1027 tagging):** The `SniValue` enum (Ascii / AsciiWithControl / NonAsciiUtf8 / NonUtf8) is the load-bearing covert-channel/log-poisoning detector; 3 of the 4 buckets emit findings. Tests: `tls_analyzer_tests.rs` covers all four paths. *This is the differentiating threat-detection identity.*

### The top 3 NFRs and top 3 conventions scoping future work

**Top 3 NFRs:**
- **NFR-RES-001 / -002 / -003 / -004 (finding caps + threshold latches):** `MAX_FINDINGS=10000`, `OVERLAP_ALERT_THRESHOLD=50`, `SMALL_SEGMENT_ALERT_THRESHOLD=2048`, `OUT_OF_WINDOW_ALERT_THRESHOLD=100`, all per-flow-direction sticky `*_alert_fired` latches. Bounds the worst-case output cardinality.
- **NFR-SEC-001..005 (raw-data layer + escape primitive + SNI control flagging):** The security NFR group is the most explicitly documented (ADR 0003 + inline tests + doc comments). It is what the tool's value depends on.
- **NFR-REL-001 (overflow-checks = true in release):** Combined with 13 explicit `saturating_*` callsites, this makes the tool panic-free under adversarial inputs while still surfacing arithmetic bugs.

**Top 3 conventions:**
- **CNV-LOG-001..003 (eprintln-only diagnostics + AtomicBool one-shot warning guards):** Sets the constraint on future observability work: no `log`/`tracing` framework is in the dependency tree.
- **CNV-ERR-001..010 (`anyhow::Result` at boundaries, `Option<T>` for absent, no `panic!`/`unimplemented!`/`todo!`/`unreachable!` in src):** Enforces "forensic tools must keep running through malformed input."
- **CNV-PUB-006 / VO-5 (`#[non_exhaustive]` ONLY on `MitreTactic`):** Closed enums everywhere else (Verdict/Confidence/ThreatCategory/InsertResult/etc.). Adding variants is a breaking change by design.

### The 3 ADRs and what each pins

- **ADR 0001 (Accepted 2026-04-07): Content-first stream dispatch.** Pins that the 5-byte content signature beats the port number for HTTP/TLS classification. Alternative considered and rejected: broadcast-to-all + lazy-buffer or pure-port dispatch.
- **ADR 0002 (Accepted 2026-04-07): Modular protocol analyzer pattern.** Pins the two-trait design (`ProtocolAnalyzer` for packet-level, `StreamHandler`+`StreamAnalyzer` for stream-level), the `AnalysisSummary{name, packets_analyzed, detail: HashMap<String, serde_json::Value>}` universal payload, and the `MAX_FINDINGS`/`MAX_MAP_ENTRIES` cardinality discipline. Alternative considered: single mega-trait, or per-protocol bespoke shapes.
- **ADR 0003 (Accepted 2026-04-09): Reporting pipeline layering.** Pins "data layer is raw, display layer formats" -- analyzers store raw post-`from_utf8_lossy` bytes in `Finding.summary`/`evidence`; only `TerminalReporter` escapes (C0+DEL+C1+`\`); `JsonReporter` delegates to serde RFC 8259. Alternative considered and rejected: PR #49's construction-site sanitization (destroyed forensic data, required tribal-knowledge propagation).

---

## 2. Cross-Pass Consistency Check

Every claim made by Passes 0-5 was checked against the others. The table below is a non-exhaustive but representative cross-section -- consistency is HIGH overall.

| Pass-A | Pass-B | Subject | Pass-A claim | Pass-B claim | Verdict |
|---|---|---|---|---|---|
| P0 | P1 | Source LOC | 3868 LOC src/ (P0 §7) | "20 components total" + per-file LOC matches (P1 §1) | consistent |
| P0 | P4 | Source LOC | 3868 LOC | Not re-counted; P4 cites P0 by reference | consistent |
| P0 | P1 | Test count | 202 #[test] (P0 §5) | "tests pin every emitted ID" (P1 §10) | consistent |
| P0 | P5 | Test count | 202 | "202 tests" (P5 CNV-TST-007) -- 185/202 follow `test_` prefix | consistent |
| P0 | P1 | Active deps | 11 active + 2 declared-unused (csv, rayon) (P0 §2) | "11 actively-used direct deps, 2 declared-but-unused" (P1 §4) | consistent |
| P0 | P4 | Deps | csv + rayon unused | NFR-VIO-005/006 lists same | consistent |
| P0 | P5 | Deps | dev-deps unused | CNV-DEP-008 same | consistent |
| P1 | P2 | Component IDs | C-1..C-20 (P1 §1) | "Component IDs match Pass 1" (P2 prelude) -- BRs cite C-4..C-17 | consistent |
| P1 | P3 | Component IDs | C-1..C-20 | BCs cite C-1, C-3..C-6, C-7..C-19 (P3 §1) | consistent |
| P1 | P4 | Component IDs | C-1..C-20 | NFR-IDs reference C-1..C-20 indirectly via file paths | consistent |
| P1 | P5 | Component IDs | C-1..C-20 | "C-1..C-20 used below" (P5 prelude) | consistent |
| P2 | P3 | Enum count | 14 enums w/ semantic meaning (P2 §4) | BCs cover Verdict/Confidence/ThreatCategory/MitreTactic/FlowState/CloseReason/Direction/InsertResult/DispatchTarget/OutputFormat/Commands/SniValue/Protocol/TransportInfo | consistent |
| P2 | P3 | DnsAnalyzer behavior | "DNS analyzer NEVER emits findings" (BR-DNS-3) | BC-DNS-004 same | consistent |
| P2 | P4 | MAX_FINDINGS=10000 | BR-RE-24 cites mod.rs:18 | NFR-RES-001 cites same line + numeric value | consistent |
| P2 | P3 | Conflicting overlap finding | BR-RE-11 → T1036 Anomaly/Likely/High | BC-RAS-018 same | consistent |
| P2 | P3 | TLS SNI 4-way classification | BR-TL-11 (4 buckets, 3 emit findings) | BC-TLS-014/017/019/013 covers all 4 paths | consistent |
| P2 | P5 | Raw-vs-display contract | INV-3 (load-bearing-unchecked) + BR-RP-1/2 | CNV-PAT "raw-vs-display separation" + ADR 0003 references | consistent |
| P3 | P4 | Magic numbers | BCs reference 10_000 / 65_536 / 50_000 / 2048 / 18_432 / 96 / 100 / 50 / 8 / 3 | NFR-RES-001..017 indexes the same constants with file:line | consistent |
| P3 | P4 | OVERLAP_ALERT_THRESHOLD=50 | BC-RAS-019 (HIGH-confidence test) | NFR-RES-002 "inferred -- no comment" | consistent (but P4 flags the lack of comment; P3 confirms a test still pins the behavior) |
| P3 | P4 | SMALL_SEGMENT_ALERT_THRESHOLD=2048 | BC-RAS-020 MEDIUM (no direct test of the threshold) | NFR-RES-003 same | consistent |
| P3 | P5 | Absent BCs (CLI unwired flags) | BC-ABS-001..010 | CNV "CLI flags exist but are unwired" inconsistency (high severity) | consistent (both flag the gap) |
| P2 | P5 | Unenforced invariants | INV-1 / INV-2 / INV-3 (load-bearing) | CNV-PUB-006 (only MitreTactic is non_exhaustive); CNV-PAT raw-vs-display | consistent |
| P1 | P2 | L2→L3 trait coupling | Smell #4 (analyzer ↔ reassembly via StreamAnalyzer) | "The only intentional layer crossing" (P2 §6 trait coupling note) | consistent |
| P1 | P5 | mod.rs weight | Smell #1 ("god-module risk on reassembly::mod.rs" 564 LOC) | CNV-MOD-003 (1 violation: reassembly/mod.rs) | consistent |
| P4 | P3 | NFR-VIO-001 (eager pcap load) | reader.rs:38-48 | BR-R-3 / BC-RDR-002 confirm eager load | consistent |
| P4 | P3 | NFR-VIO-002 (pcapng glob mismatch) | main.rs collects *.pcapng but reader rejects | BC-CLI-011 + BR-CLI-9 same | consistent |
| P4 | P3 | NFR-VIO-009 hidden MSRV 1.86 | http.rs:97 floor_char_boundary | not in P3 directly (P3 doesn't cover MSRV) | consistent-but-overlapping (P4 is the deeper source) |
| P0 | P2 | Cli/Commands/OutputFormat names | Pass 0 used "Cli", "Commands", "OutputFormat" | P2 §A explicitly corrects the orchestrator's "Command/MitreMode" terms to actual "Commands"/no-MitreMode (--mitre is bool) | consistent (P2 documents the correction) |
| P2 | P3 | DispatchTarget privacy | Module-private (P2 E-22) | BC-DSP-005/006 references the cache without exposing the enum | consistent |
| P2 | P5 | SniValue privacy | Module-private (P2 E-35) | Not in P5 (which only covers public conventions) | consistent (deliberate scope) |
| P1 | P4 | unsafe count | 0 (P1 §3 layer rules) | NFR-SEC §5.2 audit confirms 0 | consistent |
| P1 | P4 | std::net usage | only `IpAddr` value type (P1 §7) | NFR-SEC-007 + §5.3 audit confirms | consistent |

**Net verdict:** All 30+ checked claims are consistent. The only "drift" is documentation density of the magic-number rationale -- P3 may say "test pins behavior" while P4 says "no comment on why the value was chosen", which is internally consistent (a test pins the symptom; the rationale is missing). No contradictions found.

---

## 3. Subsystem Complexity Ranking (by "specification cost")

Cost = (BC count) × (LOC) × (NFR count) × (state-heaviness). Top 5 ranked.

| Rank | Subsystem | BC count | LOC | NFR count | State machines | Specification cost | Notes |
|---|---|---|---|---|---|---|---|
| 1 | **Reassembly engine (C-6/C-7/C-8/C-9)** | 53 (BC-RAS-001..053) | 1076 (564+243+240+29) | 13 (NFR-RES-001..010, REL-002..007, OBS-001, MNT-008) | 2 explicit (FlowState, InsertResult decision tree) + 1 implicit (engine main-loop) | **VERY HIGH** | Largest LOC, most state, most enums (InsertResult 9 variants), most findings emitted (5 distinct anomalies), the `OVERLAP_ALERT_THRESHOLD`/`SMALL_SEGMENT_ALERT_THRESHOLD`/`OUT_OF_WINDOW_ALERT_THRESHOLD` policy lives here. Subject of Smell #1 (god-module). |
| 2 | **TLS analyzer (C-16)** | 36 (BC-TLS-001..036) | 750 (largest single file) | 6 (NFR-RES-014/015/016/021, NFR-SEC-005/008) | 1 implicit (TlsFlowState: Idle→ClientSeen/ServerSeen→BothSeen→Dormant) | **HIGH** | JA3/JA3S, GREASE filter (RFC 8701), 4-way SNI conformance with MITRE T1027 tagging, weak/null/anon/export cipher detection, deprecated-version detection per RFC 7568, MAX_RECORD_PAYLOAD per RFC 5246/8446. |
| 3 | **HTTP analyzer (C-14)** | 26 (BC-HTTP-001..026) | 535 | 4 (NFR-RES-011/012/013/017, REL-011) | 1 implicit (HttpFlowState per direction: Buffering→Drained→ErrCount→Poisoned) | **HIGH** | Path traversal, web-shell, admin panel, unusual method, long URI, empty UA, too-many-headers, poison-threshold-3-cleared-on-flow-close mechanism, hidden MSRV 1.86 dependency (floor_char_boundary at http.rs:97). |
| 4 | **Dispatcher (C-15)** | 9 (BC-DSP-001..009) | 118 | 1 (NFR-PERF-003 + ADR 0001) | 0 explicit (decision logic is per-call) | **MEDIUM** (high importance, low LOC) | Subject of ADR 0001. Content signature precedence, port fallback, None-not-cached reclassification rule, unclassified_flows counter is a soft KPI. |
| 5 | **Terminal reporter (C-20)** | 14 (BC-RPT-006..019) | 350 | 5 (NFR-SEC-002/003, NFR-OBS-008/009) | 0 | **MEDIUM** | Subject of ADR 0003. Sole owner of `escape_for_terminal` (C0+DEL+C1+`\`). MITRE-tactic grouping, color toggle, skipped-packets suppression, em-dash vs hyphen formatting. The 10 inline `#[cfg(test)] mod tests` (the only inline tests in src/) live here for the private escape primitive. |

**Decreasing complexity for context:**

| Rank | Subsystem | BC count | LOC | NFR count | Cost |
|---|---|---|---|---|---|
| 6 | MITRE (C-11) | 9 (BC-MIT-001..009) | 144 | 1 (NFR-MNT-004) | MEDIUM |
| 7 | CLI + main (C-1/C-3) | 27 (BC-CLI-001..017 + BC-ABS-001..010) | 369 (256+113) | 4 (CLI defaults table) | MEDIUM (mostly absent BCs) |
| 8 | Decoder (C-5) | 15 (BC-DEC-001..015) | 140 | 1 (NFR-PERF-001) | MEDIUM |
| 9 | Reporter::json (C-19) + Findings (C-10) | 6 + 6 = 12 (BC-RPT-001..005 + BC-FND-001..006) | 38 + 92 | 1 (NFR-SEC-004) | LOW |
| 10 | Reader (C-4) | 8 (BC-RDR-001..008) | 58 | 1 (NFR-COMPAT-001) | LOW |
| 11 | Summary (C-17) | 4 (BC-SUM-001..004) | 61 | 0 dedicated | LOW |
| 12 | DNS analyzer (C-13) | 4 (BC-DNS-001..004) | 81 | 0 (metrics-only) | LOW |

---

## 4. Critical Design Decisions Captured by ADRs

### ADR 0001 -- Content-First Stream Protocol Dispatch (Accepted 2026-04-07)

**Pins:** The first 5 bytes of reassembled stream data classify the flow's analyzer. TLS = `data[0]==0x16 && data[1]==0x03 && len>=5`; HTTP = method-prefix match including `HTTP/` for responses; fallback = port {443,8443}→TLS, {80,8080}→HTTP; cache the result per-FlowKey unless target is None (allow reclassification on more bytes).

**Alternative that would have been:** Pure port-based dispatch (broken for HTTPS on port 80) or broadcast-to-all-analyzers (HTTP would consume TLS bytes as garbage, polluting parse_errors). The ADR explicitly rejected both.

### ADR 0002 -- Modular Protocol Analyzer Pattern (Accepted 2026-04-07)

**Pins:** Two-trait design. `ProtocolAnalyzer` for packet-level (DNS-only today); `StreamHandler`+`StreamAnalyzer` (the latter extends the former) for stream-level (HTTP+TLS). Universal `AnalysisSummary{analyzer_name, packets_analyzed, detail: HashMap<String, serde_json::Value>}` payload so reporters don't need analyzer-specific knowledge. Bounded counter maps via `MAX_MAP_ENTRIES` to prevent cardinality explosion.

**Alternative that would have been:** Single mega-trait with both packet and stream methods (would force every analyzer to no-op one of the methods), or bespoke per-protocol output shapes (would couple reporters to each analyzer).

### ADR 0003 -- Reporting Pipeline Layering (Accepted 2026-04-09)

**Pins:** The data layer (analyzers writing into `Finding.summary` / `Finding.evidence`) holds **raw** post-`from_utf8_lossy` bytes; the display layer (each reporter) is the sole owner of escape policy. Terminal reporter escapes C0+DEL+C1+`\` via `char::escape_default`; JSON delegates to `serde_json` RFC 8259. The doc comment on `Finding`'s `Display` impl explicitly warns it is NOT terminal-safe.

**Alternative that would have been:** PR #49's construction-site sanitization (the attempted fix that prompted this ADR). It used `{:?}` Debug-format in analyzers, which permanently mangled forensic data -- a Cyrillic SNI became `\u{43f}\u{440}...` in JSON output, unreadable to Russian-speaking analysts. The ADR documents the audit finding that 7 unprotected HTTP analyzer interpolations had the same vulnerability class, proving the construction-site rule never propagates.

---

## 5. Anti-Patterns / Smells (Cross-Pass De-Duplicated)

Items appearing in 2+ passes are merged. Severity tagged.

| # | Smell | Severity | Passes that flagged it |
|---|---|---|---|
| 1 | **`reassembly/mod.rs` is a 564-LOC god-module (engine + config + stats + 5 anomaly findings + eviction policy in one file)** | medium | P1 Smell #1, P5 Counter-Example "reassembly/mod.rs is 564-LOC engine", P5 CNV-MOD-003 violation |
| 2 | **7 CLI flags declared but unwired (`--threats`, `--beacon`, `--filter`, `--verbose`, `--hosts`, `--services`, `--json <FILE>`, `--csv <FILE>`)** | high | P0 Q#1/Q#4/Q#5, P1 Smell #3, P3 BC-ABS-001..010, P4 NFR-VIO-003/004, P5 anti-conv "CLI flags exist but are unwired" |
| 3 | **`OutputFormat::Csv` parses but falls through to TerminalReporter; `csv` crate declared but never imported** | medium | P0 Q#1, P1 Smell #8, P3 BC-ABS-007, P4 NFR-VIO-005, P5 CNV-DEP-008 |
| 4 | **`rayon` declared in Cargo.toml but never imported** | low | P0 Q#2, P1 Smell #8, P3 BC-ABS-008, P4 NFR-VIO-006, P5 CNV-DEP-008 |
| 5 | **Dev-deps `assert_cmd`, `predicates`, `tempfile` declared but unused; no end-to-end binary test exists** | low | P0 Q#3, P1 Smell #8, P3 BC-ABS-009, P4 NFR-VIO-007, P5 CNV-TST-009 |
| 6 | **Hidden effective MSRV is 1.86 (because of `floor_char_boundary` in http.rs:97), but Cargo.toml declares no `rust-version`** | high (silent failure mode) | P4 NFR-MNT-011 + NFR-VIO-009 (P3/P5 don't directly cover MSRV) |
| 7 | **`OVERLAP_ALERT_THRESHOLD=50`, `SMALL_SEGMENT_ALERT_THRESHOLD=2048`, `OUT_OF_WINDOW_ALERT_THRESHOLD=100` are bare numeric literals with no rationale comment** | medium | P0 Q#8, P4 NFR-RES-002/003/004 ("inferred -- no comment"), P3 BC-RAS-019/020/021 (HIGH conf for the value, MEDIUM for the rationale) |
| 8 | **`MAX_HEADERS=96`, `MAX_BUF=65536`, `MAX_HEADER_BUF=65536`, long-URI=2048, summarize top-N=20 are also bare literals with no rationale** | medium | P4 NFR-RES-011..020, P5 "magic number duplication" |
| 9 | **`JsonReporter::render` ends with `.unwrap()` on `serde_json::to_string_pretty`** | low (infallible by construction but documentation paper-cut) | P4 NFR-VIO-008, P5 CNV-ERR-009, P3 indirect via BC-RPT-001 |
| 10 | **`DnsAnalyzer::analyze` returns `Vec::new()` unconditionally -- silently violates the "analyzers produce findings" contract** | low | P0 Q#9, P1 Smell #5, P2 BR-DNS-3, P3 BC-DNS-004 |
| 11 | **Process-wide one-shot AtomicBool warning guards (CLOSE_FLOW_MISSING_WARNED, ISN_MISSING_WARNED) -- multi-flow capture emits ≤1 warning per process** | low | P0 Q#11, P1 Smell #2, P4 NFR-REL-006/007, P5 CNV-LOG-003 |
| 12 | **`StreamDispatcher.{http, tls}` are pub fields; future analyzers added to the dispatcher would also have to be pub** | low | P0 Q#13 (DataLink leak similar pattern), P1 Smell #6 |
| 13 | **`pcap_file::DataLink` leaks across the crate boundary in tests and in `decode_packet` signature** | low | P0 Q#13, P1 Smell #7, P1 Q-A3 |
| 14 | **L2→L3 trait coupling: `reassembly::handler` imports `analyzer::AnalysisSummary` + `findings::Finding`** | advisory (not a defect; documented by ADR 0002) | P1 Smell #4, P2 §6 "single intentional layer crossing", P5 CNV-PAT |
| 15 | **`main.rs:245-247` collects `*.pcapng` files into the target list, but `PcapSource::from_pcap_reader` rejects pcapng at header-parse time -- silent swallowing into the first-error-only stderr path** | medium | P4 NFR-VIO-002, P3 BR-CLI-9 / BC-CLI-011 |
| 16 | **Test function naming is mid-transition: 91.6% use `test_<subject>_<expected>` form; the newest files (Apr 13 2026) use prose-style names; mixed within `reporter_tests.rs`** | high (drift hotspot, no automated enforcement) | P5 CNV-NAM-009 / CNV-TST-007, P5 anti-conv #1 |
| 17 | **Doc-comment density is wildly uneven: 10 of 20 src files have zero `///` (`lib.rs`, `main.rs`, `dispatcher.rs`, `dns.rs`, `summary.rs`, `reader.rs`, `flow.rs`, `handler.rs`, `reporter/mod.rs`, `reporter/json.rs`); module-level `//!` exists in 1 of 20 files (`mitre.rs` only)** | high (drift hotspot) | P5 CNV-DOC-004 / CNV-DOC-005 |
| 18 | **CI matrix runs only `ubuntu-latest`; macOS/Windows install paths (README L21-23) are untested** | low (latent regression risk) | P4 NFR-VIO-010, P5 CNV-GIT-006 |
| 19 | **`Summary.services` uses port-based hint, but the dispatcher uses content-first; a TLS-on-port-80 flow shows up under "HTTP" in summary while being correctly routed to TLS** | low | P1 Q-A8, P2 INV-7 |
| 20 | **Three load-bearing invariants are NOT mechanically enforced: INV-1 (emitted MITRE IDs must exist in `technique_info`), INV-2 (MITRE catalog matches upstream), INV-3 (`Finding.summary/evidence` are raw bytes, not pre-escaped)** | high (the BCs they protect are HIGH-confidence but the invariant itself is only protected by test fixtures, not types) | P2 §12 invariants table |

---

## 6. Gap Report -- by Category

The single most important section. Lists what we DON'T know after broad sweep.

### 6.1 Orphaned modules / files

After cross-checking `src/lib.rs:1-10` (10 `pub mod`s, alphabetical) against every component's dependency edges in P1 §4 and every `use crate::…` line in the tree: **no orphan source files exist.** Every file in `src/` is reachable from `lib.rs` (and thus from `main.rs` and tests). No "dead code" hiding in src/ that wasn't already flagged as an unwired CLI flag.

However, two near-orphans exist:
- **`csv` (Cargo.toml:17) and `rayon` (Cargo.toml:22)** -- runtime deps with zero `use` sites in `src/`. They compile transitively but cost nothing functionally.
- **`assert_cmd` / `predicates` / `tempfile` (Cargo.toml:28-30)** -- dev-deps with zero `use` sites in `tests/`.

### 6.2 Under-documented subsystems (BC/LOC density < 1.0)

Formula: `coverage_density = BC_count / LOC × 100`. Components with density < 1.0 are spec'd primarily by code reading, not by tests.

| Component | LOC | BC count | Density | Verdict |
|---|---|---|---|---|
| C-1 main | 256 | 17 (incl. absent) | 6.6 | OK |
| C-3 cli | 113 | 7 | 6.2 | OK |
| C-4 reader | 58 | 8 | 13.8 | OK |
| C-5 decoder | 140 | 15 | 10.7 | OK |
| C-6 reassembly (mod.rs only) | 564 | ~30 (BC-RAS-001..030) | 5.3 | OK |
| C-7 reassembly::flow | 243 | ~10 (BC-RAS-031..053 partial) | 4.1 | OK |
| C-8 reassembly::segment | 240 | ~17 (BC-RAS-031..047) | 7.1 | OK |
| C-9 reassembly::handler | 29 | 0 (interface-only) | 0.0 | **FLAGGED** |
| C-10 findings | 92 | 6 | 6.5 | OK |
| C-11 mitre | 144 | 9 | 6.3 | OK |
| C-12 analyzer (mod.rs) | 31 | 0 (trait-only) | 0.0 | **FLAGGED** |
| C-13 dns | 81 | 4 | 4.9 | OK |
| C-14 http | 535 | 26 | 4.9 | OK |
| C-15 dispatcher | 118 | 9 | 7.6 | OK |
| C-16 tls | 750 | 36 | 4.8 | OK |
| C-17 summary | 61 | 4 | 6.6 | OK |
| C-18 reporter (mod.rs) | 15 | 0 (trait-only) | 0.0 | **FLAGGED** |
| C-19 reporter::json | 38 | 5 | 13.2 | OK |
| C-20 reporter::terminal | 350 | 14 | 4.0 | OK |

**Flagged:** C-9, C-12, C-18 are trait-only definitions (no behavior to test directly); their BC absence is expected. **No component has a problematic density gap.** Coverage is strong across the board.

### 6.3 Subsystem-level BCs needing function-level depth

These BCs are spec'd at "the engine does X" level; deepening rounds (Pass 2/3 round 2) need to drill into "exactly what bytes, exactly which counters, exactly which fields of the emitted Finding."

| Subsystem BC | Function-level question for deepening |
|---|---|
| BC-RAS-018 (Conflicting overlap emits T1036 finding) | Exact `Finding.summary` string format; exact `evidence` payload contents (does it carry the conflicting bytes? the offsets? both?); is the source_ip the initiator or null? |
| BC-RAS-023 (Truncated segment emits Stream-depth-exceeded finding) | Exact bytes that are *kept* vs *truncated* (how is the boundary computed -- `max_depth - reassembled_bytes - buffered_bytes`?); exact summary string; whether `depth_exceeded=true` happens BEFORE or AFTER the finding push |
| BC-RAS-025 (`finalize()` emits summary-level segment-limit finding) | Exact pluralization rule for "1 segment dropped" vs "N segments dropped"; verify `MAX_FINDINGS` bypass works even at cap+1 |
| BC-TLS-014..020 (SNI conformance findings) | For each of the 4 SniValue variants: the exact `evidence` array shape (does Ascii emit no evidence, or is it always `["hex: ..."]`?); the `summary` template string; how the `<non-utf8:HEX>` count key is computed for collisions |
| BC-HTTP-005..011 (HTTP detection rules) | For each detection rule: the exact substring match (case-sensitive? URL-decode? `.find()` vs prefix?); the `evidence` array shape (raw URI vs truncated URI vs both); how `source_ip` is populated (or always None?) |
| BC-DSP-001..003 (content-first dispatch) | What happens when data is exactly 5 bytes and starts with `GET ` -- does HTTP win? What about a 5-byte buffer of `\x16\x03\x01GET` -- does TLS win? (Implementation detail of branch ordering.) |
| BC-RAS-015..017 (eviction) | When `max_flows` is reached AND `total_memory > memcap` simultaneously, which path wins? Is eviction global or per-trigger? |
| BC-RAS-022 (alert latches per direction) | If a flow's client_to_server direction emits the overlap alert, does the server_to_client direction emit independently? (Tests imply yes, but no test directly verifies both directions on the same flow.) |

### 6.4 Missing entity detail (struct-shape only; private parsing types)

These were captured by Pass 2 only at "shape level"; deepening should formalize their contracts.

| Entity | What's missing |
|---|---|
| `ParsedRequest` (E-40, private in http.rs:13-21) | Exact lifetime of borrows from `httparse::Request`; whether the `host`/`user_agent` Options are header-presence or header-non-empty |
| `ParsedResponse` (E-41, private in http.rs:39-42) | Just `bytes_consumed` and `status_code` -- but what about response headers? Are they ever extracted? |
| `HttpFlowState` (E-32) | Exact reset rules when `_poisoned` becomes true; whether `counted_as_non_http` is per-direction or per-flow |
| `TlsFlowState` (E-34) | The implicit state machine (Idle→ClientSeen/ServerSeen→BothSeen→Dormant) isn't an enum -- it's a `(bool, bool)` pair; documenting it as an explicit machine would help future authors |
| `SniValue` (E-35, private in tls.rs:173-195) | Exact disambiguation rules among AsciiWithControl / NonAsciiUtf8 / NonUtf8 when an SNI has *both* control bytes *and* non-ASCII UTF-8 |
| `DispatchTarget` (E-22, private in dispatcher.rs:9-13) | The `None` variant's "not-cached" semantics deserves a dedicated test that exercises a flow whose first byte arrives in a 4-byte segment, then a 6-byte segment that DOES match TLS |

### 6.5 Tests-without-BCs

P3 mapped 137 BCs against 202 tests. That implies ~65 tests are not 1:1 mapped to a BC ID -- many because a single BC is multiply-pinned (e.g., BC-TLS-014 has 7 named tests) and some because the test exercises a helper or a composition. Specific cases worth tabulating in deepening:

- The 10 inline `#[cfg(test)] mod tests` in `src/reporter/terminal.rs:261-350` -- pinned to BC-RPT-007/008/009 but not individually mapped.
- `tests/integration_test.rs` (single 66-LOC test) -- end-to-end smoke covers many BCs at once; not in P3's index by name.
- `tests/linktype_integration_tests.rs` (3 tests) -- cross-link-type pcap loads; loosely mapped to BC-DEC-003..006.
- `tests/http_integration_tests.rs` and `tests/tls_integration_tests.rs` -- end-to-end pcap fixtures; loosely mapped to BC-HTTP-001 / BC-TLS-001 / etc.

A deepening pass should produce a `test_id -> BC_id` matrix (grep-driven from test function names) and flag any test with zero BC mapping.

### 6.6 BCs-without-tests (MEDIUM/LOW confidence)

These are the 26 BCs (137 - 111 HIGH = 26 lower-confidence) that are spec'd by code reading or ADR-implication only. Spec-on-paper risk; would need new tests to pin.

**MEDIUM-confidence BCs** (from P3 §1; complete list):
- BC-RDR-004 (pcapng rejection - no direct test)
- BC-RDR-006/007/008 (anyhow context lines)
- BC-DEC-008/009/010/011/013 (decoder edge cases)
- BC-RAS-001 (constructor panic-asserts)
- BC-RAS-002 (non-TCP skip counter)
- BC-RAS-020 (small-segment threshold)
- BC-RAS-023 (truncated segment finding)
- BC-RAS-024 (MAX_FINDINGS cap)
- BC-DSP-005/006/009
- BC-HTTP-024/025
- BC-TLS-005/007/008/012/033/034/035/036
- BC-CLI-007/008/009/010/011/012/015/017
- BC-FND-006
- BC-RPT-018/019
- BC-SUM (none MEDIUM)
- BC-MIT (none MEDIUM)

**LOW-confidence BCs** (ADR/comment only, no test, hard to test):
- BC-RAS-029 (CLOSE_FLOW_MISSING_WARNED one-shot)
- BC-RAS-048 (ISN_MISSING_WARNED one-shot)
- BC-CLI-013 (indicatif progress bar template)
- BC-MIT-009 (`#[non_exhaustive]` invariant -- not testable as behavior)

### 6.7 NFR-violations-without-fix

From P4 §7. Each becomes a candidate work item.

| NFR-VIO ID | Description | Disposition for future work |
|---|---|---|
| NFR-VIO-001 | Eager pcap load contradicts README "multi-GB captures" claim | Either fix README language ("one-pass" not "streaming") OR implement streaming reader |
| NFR-VIO-002 | `*.pcapng` glob collects files the reader rejects | Either filter to `*.pcap` only OR support pcapng in reader |
| NFR-VIO-003 | 7 CLI flags unwired (--threats, --beacon, --filter, --verbose, --hosts, --services, file-output forms of --json/--csv) | Either wire them up or hide via `#[arg(hide = true)]` with tracking issue |
| NFR-VIO-004 | `--json <FILE>` / `--csv <FILE>` accept paths but always stdout | Wire to file or remove path argument |
| NFR-VIO-005 | `OutputFormat::Csv` falls through to TerminalReporter | Implement CSV reporter or remove the variant |
| NFR-VIO-006 | `rayon` declared but unused | Remove from Cargo.toml or implement parallel file processing |
| NFR-VIO-007 | `assert_cmd`/`predicates`/`tempfile` dev-deps unused | Add at least one end-to-end binary test or remove |
| NFR-VIO-008 | `JsonReporter::render` ends with `.unwrap()` | Convert to `.expect("infallible by construction")` or `Result<String>` |
| NFR-VIO-009 | Effective MSRV is 1.86 (floor_char_boundary) but unstated | Add `rust-version = "1.86"` to Cargo.toml |
| NFR-VIO-010 | CI only `ubuntu-latest`; macOS/Windows install paths untested | Add a CI matrix dimension |

### 6.8 Convention drift hotspots (specific files)

From P5 §7 "three conventions most at risk of drift":

1. **Test function naming (CNV-NAM-009 / CNV-TST-007):** 91.6% follow `test_<subject>_<expected>`. The 20 outliers are concentrated in:
   - `tests/mitre_tests.rs` (10/10 outliers; ALL prose-style)
   - `tests/reporter_tests.rs` (7/19 prose-style: `mitre_grouping_*`, `display_renders_*`)
   - `tests/tls_analyzer_tests.rs` (3/39 prose-style: `ascii_control_sni_finding_sets_mitre_t1027`, `non_ascii_utf8_sni_finding_sets_mitre_t1027`, `non_utf8_sni_finding_sets_mitre_t1027`)

2. **Doc-comment density (CNV-DOC-004):** 10 of 20 src files have zero `///` comments:
   - `src/lib.rs` (10 LOC, no docs)
   - `src/main.rs` (256 LOC, no docs)
   - `src/dispatcher.rs` (118 LOC, no docs -- subject of ADR 0001!)
   - `src/analyzer/dns.rs` (81 LOC, no docs)
   - `src/summary.rs` (61 LOC, no docs)
   - `src/reader.rs` (58 LOC, no docs)
   - `src/reassembly/flow.rs` (243 LOC, 4 pub types, no docs -- WORST GAP)
   - `src/reassembly/handler.rs` (29 LOC, declares 2 of 4 traits in the crate, no docs)
   - `src/reporter/mod.rs` (15 LOC, declares the Reporter trait, no docs)
   - `src/reporter/json.rs` (38 LOC, no docs)

3. **Branch naming (CNV-GIT-002):** `CLAUDE.md` documents 3 patterns (`feature/`, `worktree-issue-N-`, `worktree-`) but observed practice uses 5 (adds `chore/`, `setup/`).

---

## 7. Deepening Plan Recommendation

For EACH of the 6 passes needing convergence, 5-10 concrete questions for round 2. Anchored to file:line or BC-ID.

### Pass 0 (Inventory) -- Round 2

P0 was the broadest. Round 2 deepening priorities:

1. **[P1]** Re-verify the 38 .rs file count by reading `find` output against P0 §3 file tree -- any new files added since 2026-05-19? (low confidence drift)
2. **[P2]** Audit `tests/fixtures/` (14 files) to determine which test consumes each one. P0 noted `smb3.pcapng` "presumably used as negative test" -- confirm or refute.
3. **[P2]** Count `///` doc lines per src file and produce the `pub-doc-coverage` table for Pass 5's drift-hotspot recommendation.
4. **[P2]** Audit `.github/workflows/ci.yml` for any new jobs (e.g., security audit, codecov) added after 2026-05-19.
5. **[P3]** Confirm Cargo.lock is unchanged (38,291 B) -- if it changed, dep graph may have shifted.
6. **[P3]** Verify the 18 `docs/superpowers/` files are still 10 plans + 8 specs -- no new specs added.

### Pass 1 (Architecture) -- Round 2

P1 was thorough on the 20-component model. Round 2 deepening:

1. **[P1]** Resolve open question Q-A2: when `evict_flows` removes a flow before `on_data` was ever delivered, does the dispatcher's `routes` map still see it via `on_flow_close`? Add a test using `make_tcp_packet` + a small `max_flows` config to drive eviction.
2. **[P1]** Resolve Q-A5: are JSON `summary.protocols` / `summary.services` keys deliberately HashMap-ordered (non-deterministic), or should they sort like `unique_hosts`? Forensic reproducibility matters. Cite `src/reporter/json.rs:18-22` and `:30-31`.
3. **[P1]** Resolve Q-A6: should the 3 per-direction reassembly alerts (overlap, small-seg, OOW) be summed per-flow instead of per-direction? Affects cardinality and the operator's mental model.
4. **[P1]** Resolve Q-A7: classification cost ceiling -- a flow with hundreds of 4-byte segments that never match. Should there be a `max_classification_attempts` bound? Cite `src/dispatcher.rs:72-79`.
5. **[P2]** Resolve Q-A8: `Summary.services` is port-based; the dispatcher is content-first. Document the deliberate inconsistency or unify.
6. **[P1]** Audit each of the 20 components for inline `#[cfg(test)]` blocks -- P1 noted only `reporter/terminal.rs` has one; re-verify post any new commits.
7. **[P2]** Inventory the 4 traits (`Reporter`, `ProtocolAnalyzer`, `StreamHandler`, `StreamAnalyzer`) -- list every default method (currently zero); confirm trait stability.

### Pass 2 (Domain Model) -- Round 2 (TIER 1)

P2 round 2 is the highest-value deepening. Round 2 should:

1. **[P1]** For each of the 6 active analyzers (DNS, HTTP, TLS), drill into the *exact `Finding` payload shape* emitted by each detection rule: summary template string, evidence array element-by-element, source_ip population rule, timestamp population rule (currently always None). Anchor to BC-HTTP-005..011, BC-TLS-007..028, BC-RAS-018/023/025.
2. **[P1]** Formalize the implicit `TlsFlowState` state machine (P2 §9c) -- promote the `(client_hello_seen, server_hello_seen)` pair to a named domain state machine in the spec.
3. **[P1]** Formalize the implicit `HttpFlowState` per-direction parser state machine (P2 §9d) -- name the states explicitly (Buffering / Drained / ErrCount / Poisoned).
4. **[P1]** Make the 3 load-bearing-unenforced invariants from P2 §12 (INV-1, INV-2, INV-3) into explicit test fixtures or doc-comments. INV-3 in particular (raw-vs-display) is the security invariant.
5. **[P1]** Catalog the exact MITRE technique IDs *emitted* (6: T1027, T1036, T1046, T1083, T1499.002, T1505.003) vs *defined but unused* (4: T1040, T1071, T1071.004, T1573) and propose: remove the unused IDs or document why they're staged.
6. **[P2]** Document the exact disambiguation rules for `SniValue` when an SNI has both control bytes AND non-ASCII UTF-8 -- which variant wins? Cite tls.rs:173-242 and test which case is tested.
7. **[P2]** Resolve the implicit "FlowState transitions are monotonic toward Closed" claim (VO-11) -- verify with a state-machine test that out-of-spec ordering (FIN-before-SYN-ACK) doesn't leave inconsistent state.
8. **[P2]** Document the exact rule for `set_initiator` "first writer wins" -- which call wins under what packet ordering (BC-RAS-004/005)?
9. **[P2]** Audit the 10 state containers (P2 §5) for `Drop` impls -- are any relying on explicit cleanup (finalize, on_flow_close) for resource correctness? Document any that are.

### Pass 3 (Behavioral Contracts) -- Round 2 (TIER 1)

P3 round 2 is also highest-value. Round 2 should:

1. **[P1]** For each of the 26 MEDIUM/LOW-confidence BCs (P3 §1 + listed in §6.6 above), either find an indirect test that pins it, or document the test gap with a specific recommended test.
2. **[P1]** Pin the 5 anomaly findings emitted by reassembly (BC-RAS-018/019/020/021/023/025) with *function-level* depth: exact summary string, exact evidence shape, exact MITRE ID, exact verdict/confidence combinations.
3. **[P1]** Audit the "BC-ABS-001..010" absent BCs and propose disposition for each: either wire the flag (with a follow-up plan) or hide it (`#[arg(hide=true)]`).
4. **[P1]** Drill into BC-RAS-024 (MAX_FINDINGS=10000 cap) and BC-RAS-025 (finalize bypass): produce a regression test that exercises the cap-at-10001 case and verifies the finalize bypass adds exactly one more.
5. **[P2]** Map every test in `tests/integration_test.rs`, `tests/http_integration_tests.rs`, `tests/tls_integration_tests.rs`, `tests/linktype_integration_tests.rs` to specific BC IDs.
6. **[P2]** For each of the 10 inline tests in `src/reporter/terminal.rs:261-350`, map to BC-RPT-007/008/009.
7. **[P2]** Resolve the BC-DSP-001/002 branch-ordering question: when a 5-byte buffer starts with `\x16\x03\x01GET`, which dispatch wins? Add a test or document.
8. **[P2]** Catalogue every `Finding` emitted by the codebase (estimate: 14 emission sites across http.rs, tls.rs, reassembly/mod.rs). Build a `emission_site -> BC_id -> finding_shape` matrix.

### Pass 4 (NFR Catalog) -- Round 2

Round 2 should:

1. **[P1]** Provenance research on the 4 alert thresholds (OVERLAP_ALERT_THRESHOLD=50, SMALL_SEGMENT_ALERT_THRESHOLD=2048, OUT_OF_WINDOW_ALERT_THRESHOLD=100, POISON_THRESHOLD=3): git blame + PR history to find justification. If none exists, propose calibration-against-benign-traffic as a follow-up.
2. **[P1]** Add `rust-version = "1.86"` recommendation to `Cargo.toml` (or document why not).
3. **[P2]** Audit the 10 NFR violations (NFR-VIO-001..010) for actionability; assign each to "fix" or "document/accept" disposition.
4. **[P2]** Recommend consolidating `MAX_BUF=65_536` (TLS) + `MAX_HEADER_BUF=65_536` (HTTP) into a shared constant if the duplication is accidental.
5. **[P2]** Catalog every saturating arithmetic site (13 reported); verify each is necessary by checking the upstream input bound.
6. **[P3]** Audit time policy (NFR-OBS § timestamp): no analyzer currently populates `Finding.timestamp`. Decide: deprecate the field, or wire it to `RawPacket.timestamp_secs`.
7. **[P3]** Survey for benchmarks -- none exist in the repo today; recommend adding `criterion` benches for reassembly hot paths if perf claims are to be load-bearing.
8. **[P3]** Recommend a `cargo audit` / `cargo deny` CI job (NFR-SUP-001..005 currently lack supply-chain automation).

### Pass 5 (Conventions) -- Round 2

Round 2 should:

1. **[P1]** Pick a direction on test-function naming (CNV-NAM-009 / CNV-TST-007). The newer prose-style names (Apr 13 2026 commits) reject the `test_` prefix; codify the direction in `CLAUDE.md`.
2. **[P1]** Decide on doc-comment policy: either enable `#![warn(missing_docs)]` on `lib.rs`, OR explicitly document "no `///` required for trivial accessors" rule.
3. **[P2]** Decide on module-level `//!` policy: roll out 1-3 line headers to all 20 modules OR delete the lone header in `mitre.rs`.
4. **[P2]** Widen branch-naming patterns in `CLAUDE.md` to acknowledge `<type>/<slug>` (semantic-PR-aligned) as a 4th valid pattern.
5. **[P2]** Codify the helper-naming convention: `make_*` for `ParsedPacket`-shaped domain builders; `build_*` for raw-bytes protocol synthesizers; no bare slugs.
6. **[P2]** Document the explicit pub-field-on-data-carrier vs private-field-on-behavior-owner rule (10 types each side).
7. **[P3]** Refactor `reassembly/mod.rs` to extract the 564-LOC engine into `src/reassembly/engine.rs` to align with the "mod.rs is thin" convention (Smell #1).
8. **[P3]** Format-string positional-vs-inline-capture census across ~20 remaining positional callsites; one-shot refactor.
9. **[P3]** Decide disposition of the 7 unwired CLI flags (NFR-VIO-003) -- either wire each to "Feature not yet implemented" early-exit OR remove from clap.

---

## 8. Confidence Assessment

| Dimension | Confidence | Rationale |
|---|---|---|
| **Inventory completeness** | HIGH | Pass 0 grounded every file/dep/test count in `find`/`wc`/manifest reads against the live tree. Re-verifiable in seconds. |
| **Architecture accuracy** | HIGH | Every component, edge, and exported symbol grounded in `src/*.rs` with file:line citations. Cycle scan done. File-level DAG verified acyclic against green CI. |
| **Domain model accuracy (structural)** | HIGH | All 41 entities cited by file:line; 14 enums fully variant-listed; 4 traits fully method-listed. Cross-checked against P1 component IDs. |
| **Domain model accuracy (behavioral)** | MEDIUM-HIGH | 101 business rules; HIGH-confidence (test-pinned) where flagged, MEDIUM where derived from code only. 3 load-bearing invariants are explicitly unenforced (INV-1/2/3). |
| **BC coverage (137 BCs)** | MEDIUM-HIGH | 81% HIGH-confidence (test-pinned). 26 MEDIUM/LOW need test additions to escape spec-on-paper-only status. 10 BC-ABS-* are absent-behaviors (gaps in the product, not gaps in coverage). |
| **NFR coverage (76 NFRs)** | HIGH (resource bounds + security) / MEDIUM (perf + observability) | Resource bounds and security NFRs are well-grounded in code + ADRs + tests. Performance is "fast" by README claim with zero benchmarks. Observability uses eprintln + counters in lieu of structured logs. |
| **Convention coverage (73 conventions)** | HIGH | 56 of 73 are "all" universality (no counter-examples). 14 are "most" (some counter-examples). 3 drift hotspots (test naming, doc comments, branch naming) explicitly flagged for round 2. |
| **Cross-cutting consistency** | HIGH | All 30+ checked claims across passes are consistent (§2 above). One acceptable variance: passes-3-confidence-on-symptom vs passes-4-confidence-on-rationale for the same magic number (e.g., OVERLAP_ALERT_THRESHOLD=50 -- test pins behavior, but no comment explains the value). |

---

## 9. Known Unknowns (Original-Team-Only Questions)

Things inherently NOT discoverable from the codebase alone -- the original authors are the only resolution path.

1. **Why `OVERLAP_ALERT_THRESHOLD = 50`?** Did 50 match an observed evasion pattern in benign vs malware traffic, or was it "high enough to be unusual"?
2. **Why `SMALL_SEGMENT_ALERT_THRESHOLD = 2048` small-segment count and `< 8 bytes` for "small"?** No RFC or industry doc cited.
3. **Why `OUT_OF_WINDOW_ALERT_THRESHOLD = 100`?** "A small number is normal noise, 100 is a problem" is the inferred rationale but no calibration data exists.
4. **Why `MAX_HEADERS = 96` (HTTP)?** Suricata default? IIS limit? RFC reference? The number "96" has no obvious provenance.
5. **Why `MAX_BUF = 65_536` for TLS and `MAX_HEADER_BUF = 65_536` for HTTP?** Same value, two constants, two analyzers. Deliberate decoupling for future-proofing or accidental drift?
6. **Why "long URI" threshold is 2048 chars?** RFC 7230 §3.1.1 recommends "at least 8000 octets" support; 2048 is more aggressive. Where does 2048 come from? (IIS default is 16,384.)
7. **Why is `truncate_uri` summary cap 120 chars and evidence cap 200 chars?** ADR 0003 flags these as "should be at display layer"; original authors chose to leave them at construction.
8. **Why is `summarize()` top-N truncation hard-coded to 20?** Operator-friendly but should this be CLI-tunable?
9. **Why are `--threats`, `--beacon`, `--filter`, etc. declared but unwired?** Roadmap stubs, deferred implementation, or abandoned plans?
10. **Why is the dispatcher's classification cache "None is not cached" rule the right semantic?** It enables reclassification on more bytes, but it also enables unbounded classification cost on adversarial short-segment traffic.
11. **Why `CLOSE_FLOW_MISSING_WARNED` and `ISN_MISSING_WARNED` use process-wide atomics rather than per-`TcpReassembler`-instance state?** For a multi-target run (which the CLI supports), only the first target's anomalies trigger the warning.
12. **Why was `OutputFormat::Csv` added to the enum but never wired?** Was a CSV reporter planned and dropped, or is it staged for a future PR?
13. **Why `MitreTactic` is `#[non_exhaustive]` but `ThreatCategory` is closed?** Both are vocabularies that could evolve. ThreatCategory has 8 variants today; would adding a 9th be intentional or accidental?
14. **Why does the system have 4 defined-but-unemitted MITRE technique IDs (T1040, T1071, T1071.004, T1573)?** Future analyzers staged?
15. **Why does the test suite have a `smb3.pcapng` fixture (25,692 B) when the reader rejects pcapng?** Confirmed-negative-test, or staged for future SMB analyzer?
16. **Why does `--reassembly-depth` default to 10 MB but `max_segments_per_direction` defaults to 10000?** A 10 MB stream with 10000 segments averages 1 KB/segment -- is that the design point, or arbitrary?
17. **Are the (currently unused) `LateralMovement` and `C2` ThreatCategory variants intentional placeholders, or vestigial?**
18. **Is the 7-port `app_protocol_hint` table (53/80/443/22/445/502/20000) deliberately limited to those 7 services, or just initial coverage?**

---

## 10. Recommended Next Steps After This Synthesis

Specific, ordered:

**Step 1 (TIER 1 deepening):** Run **Pass 2 round 2** and **Pass 3 round 2** in parallel. These are highest-value because:
- They have the most BCs/entities at function-level depth still pending (§6.3).
- The 3 load-bearing-unenforced invariants (INV-1/INV-2/INV-3) need formalization.
- The 26 MEDIUM/LOW-confidence BCs need test gap analysis.
- These passes are the dominant inputs for downstream `/create-domain-spec` and `/create-brief`.

**Step 2 (TIER 2 deepening, parallel):** Run **Pass 0 round 2**, **Pass 1 round 2**, **Pass 4 round 2**, **Pass 5 round 2** in parallel. These benefit from any new subsystems / files discovered during Pass 2/3 round 2. Specifically:
- Pass 0 round 2: re-verify file count, fixture usage, doc-comment density table.
- Pass 1 round 2: resolve the 8 architectural open questions Q-A1..Q-A8.
- Pass 4 round 2: provenance research on the 4 magic-number thresholds; disposition the 10 NFR violations.
- Pass 5 round 2: pick a direction on the 3 drift hotspots (test naming, doc density, branch naming); refactor recommendation for `reassembly/mod.rs`.

**Step 3 (coverage audit B.5):** With all rounds 2 in hand, run a grep-driven `test_id -> BC_id` matrix audit. Any test in `tests/` that doesn't map to a BC, OR any BC without a test, gets a dedicated row in the gap log.

**Step 4 (extraction validation B.6):** Spot-check 5-10 random BCs by re-reading the cited source code and tests. Verify the BC statement accurately reflects code reality. If drift is found, escalate to round 3 of the relevant pass.

**Step 5 (final synthesis C):** Produce `wirerust-pass-8-deep-synthesis.md` consuming P0/P1/P2/P3/P4/P5 broad-sweep + all round 2/3 outputs. This becomes the canonical reference for `/create-brief`, `/create-domain-spec`, `/create-prd`, `/semport-analyze`. Pass 6 (this document) is the Phase A anchor; Pass 8 is the Phase B+C anchor.

**Throughout:** Treat the 3 load-bearing-unenforced invariants (INV-1/INV-2/INV-3) and the 5 most-important BCs (§1) as "do-not-regress" guardrails for any spec-crystallization work.

---

## State Checkpoint

```yaml
pass: 6
status: complete
sub_pass: synthesis_anchor
inputs_ingested: 8  # 5 prior pass files + 3 ADRs
bcs_cross_referenced: 137
nfrs_cross_referenced: 76
conventions_cross_referenced: 73
consistency_checks_performed: 30
contradictions_found: 0
drift_items_found: 14  # most/some-universality conventions
load_bearing_invariants_carried_forward: 3  # INV-1, INV-2, INV-3
unwired_features_consolidated: 7  # CLI flags
nfr_violations_carried_forward: 10
anti_patterns_de_duplicated: 20
gap_categories: 8
known_unknowns: 18
deepening_questions_authored: 51  # 6 P0 + 7 P1 + 9 P2 + 8 P3 + 8 P4 + 9 P5
recommended_next_step: phase_B_pass_2_round_2_and_pass_3_round_2_parallel
timestamp: 2026-05-19T00:00:00Z
novelty: SUBSTANTIVE
next_action: phase_B_tier_1_deepening
resume_from: null
```

