---
document_type: prd-supplement-nfr-story-map
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-06-08T12:00:00Z
phase: 2
traces_to:
  - .factory/specs/prd-supplements/nfr-catalog.md
  - .factory/stories/STORY-INDEX.md
---

# NFR-to-Story Traceability Map: wirerust

> This document is the canonical mapping from NFR Catalog entries (nfr-catalog.md v1.3)
> to owning stories. It is the input for story-writer to add `nfr:` references to story
> frontmatter under the Criterion-38 closure pass.
>
> Story-writer must propagate these mappings into each listed story's frontmatter `nfr:`
> array and, where relevant, into the story body's AC table. Do NOT add nfr references
> to stories that are not listed as owning stories for a given NFR.
>
> **Mapping basis:** NFR-to-story assignment follows subsystem alignment:
> - SS-01 = PCAP reader (STORY-001)
> - SS-02 = Packet decoder (STORY-002, STORY-003, STORY-004, STORY-005)
> - SS-04 = TCP reassembly engine (STORY-011 through STORY-021)
> - SS-05 = Content-first dispatcher (STORY-031, STORY-032, STORY-033)
> - SS-06 = HTTP analyzer (STORY-041 through STORY-046)
> - SS-07 = TLS analyzer (STORY-051 through STORY-058)
> - SS-08 = DNS analyzer (STORY-066)
> - SS-09 = Finding data model (STORY-069, STORY-070)
> - SS-10 = MITRE mapping (STORY-071)
> - SS-11 = Reporters (STORY-076 through STORY-080)
> - SS-12 = CLI + orchestration (STORY-086 through STORY-090)
> - SS-13 = Absent-behavior contracts (STORY-096)
>
> For cross-cutting NFRs (security, reliability, CI/supply-chain), the primary owning
> story is the one that most directly implements or verifies the property. "No story
> owner" entries are explicitly marked with a reason.

---

## NFR Mapping Table

| NFR-ID | Priority | Owning Story IDs | Rationale |
|--------|----------|-----------------|-----------|
| **Performance** | | | |
| NFR-PERF-001 | P0 | STORY-002, STORY-005 | Zero-copy etherparse decoding is implemented in SS-02 decoder (decoder.rs:288-292); STORY-002 covers Ethernet/RAW/IPv4/IPv6 decode paths; STORY-005 covers packet_len semantics and TCP/UDP payload extraction where the to_vec() allocation occurs |
| NFR-PERF-002 | P1 | STORY-001 | Eager `Vec<RawPacket>` load is the read strategy in SS-01 reader.rs; STORY-001 owns PCAP ingestion and the all-in-memory design |
| NFR-PERF-003 | P0 | STORY-031, STORY-032 | Content-first dispatch and classification caching are the core behavior of SS-05; STORY-031 covers initial classification; STORY-032 covers caching and retry budget |
| NFR-PERF-004 | P1 | STORY-016 | Overlap detection slice equality is in SS-04 reassembly/segment.rs overlap path; STORY-016 covers overlap detection; autovectorization assertion remains OPEN-DEBT |
| **Security** | | | |
| NFR-SEC-001 | P0 | STORY-069, STORY-070, STORY-077 | Raw-data contract (no escape at analyzer layer) is established by the Finding data model in SS-09 (STORY-069/070) and enforced at the terminal reporter boundary in SS-11 (STORY-077); ADR 0003 layering |
| NFR-SEC-002 | P0 | STORY-077 | `escape_for_terminal` function and its 15 inline tests live entirely in terminal.rs (SS-11); STORY-077 covers escape_for_terminal, C1 safety, and the end-to-end escape contract |
| NFR-SEC-003 | P0 | STORY-077 | Analyzer-summary detail value escaping (`escape_for_terminal(&val.to_string())`) is in the terminal reporter path (SS-11); same story as NFR-SEC-002 |
| NFR-SEC-004 | P0 | STORY-076 | JSON reporter serde_json delegation for RFC 8259 C0 escaping is in SS-11 JsonReporter; STORY-076 covers JSON reporter structure and byte handling |
| NFR-SEC-005 | P0 | STORY-055, STORY-056 | SniValue 4-arm enum and extract_sni classification is in SS-07 tls.rs; STORY-055 covers arms 1 and 2 (clean ASCII + C0 detection); STORY-056 covers arms 3 and 4 (non-ASCII UTF-8 + non-UTF-8) |
| NFR-SEC-006 | P0 | STORY-091 | No shell-out grep audit is a cross-cutting CI/tooling property; STORY-091 (anchor-validation tooling) is the closest owner; if no story exists that exercises this, see "no story owner" note below |
| NFR-SEC-007 | P0 | STORY-091 | No network I/O grep audit is a cross-cutting property; same rationale as NFR-SEC-006 |
| NFR-SEC-008 | P0 | STORY-058 | MAX_RECORD_PAYLOAD oversized-record guard is in SS-07 tls.rs; STORY-058 covers buffer management, record parsing infrastructure, and flow lifecycle in the TLS analyzer |
| **Reliability** | | | |
| NFR-REL-001 | P0 | STORY-011 | overflow-checks=true in Cargo.toml [profile.release] is a build-system property; STORY-011 owns TcpReassembler constructor and is the foundational reassembly story that depends on this property for arithmetic safety |
| NFR-REL-002 | P0 | STORY-013, STORY-014, STORY-015 | `wrapping_sub` for TCP sequence number arithmetic is in SS-04 reassembly/segment.rs seq_offset; STORY-013 covers the wrapping_sub three-way-handshake state machine that 014/015 build on; STORY-014 covers ISN management; STORY-015 covers in-order delivery and sequence math |
| NFR-REL-003 | P0 | STORY-011, STORY-015, STORY-016, STORY-041, STORY-058 | Saturating arithmetic property spans SS-04 (reassembly/segment.rs, reassembly/mod.rs, reassembly/flow.rs), SS-05 (dispatcher.rs), SS-06 (http.rs), SS-07 (tls.rs); primary owners are STORY-011 (TcpReassembler, owns the config arithmetic), STORY-015 (segment insert, depth/offset math), STORY-016 (overlap count), STORY-041 (HTTP parse error counters + buffer cap), STORY-058 (TLS buffer cap) |
| NFR-REL-004 | P1 | STORY-011 | 5 defensive asserts in `TcpReassembler::new` at mod.rs:115-125; STORY-011 owns TcpReassembler constructor validation |
| NFR-REL-005 | P0 | STORY-021 | `finalize()` idempotency guard (`finalized: bool`) is in SS-04 mod.rs:103/615-618; STORY-021 owns finalize lifecycle and MAX_FINDINGS cap |
| NFR-REL-006 | P1 | STORY-019 | `CLOSE_FLOW_MISSING_WARNED: AtomicBool` one-shot warning is in SS-04 lifecycle.rs; STORY-019 owns flow lifecycle including RST close and missing-key warning |
| NFR-REL-007 | P0 | STORY-014 | `ISN_MISSING_WARNED: AtomicBool` one-shot warning is in SS-04 segment.rs:16, IsnMissing arm at 204-207; STORY-014 owns ISN management and IsnMissing guard |
| NFR-REL-008 | P0 | STORY-001, STORY-002 | `anyhow::Result` error propagation through file/reader/decoder paths; STORY-001 owns file/reader error surfaces; STORY-002 owns decode error paths |
| NFR-REL-009 | P1 | STORY-089 | First-error-only decode error suppression is in main.rs:170-177; STORY-089 owns decode error counting, format resolution, and output routing in SS-12 |
| NFR-REL-010 | P0 | STORY-058 | TLS oversized-record clear-and-continue at tls.rs:643-653; STORY-058 owns TLS buffer management and record parsing infrastructure |
| NFR-REL-011 | P0 | STORY-044 | HTTP `POISON_THRESHOLD = 3` per-direction poison state machine; STORY-044 owns parse-error isolation and poisoning state machine in SS-06 |
| **Observability** | | | |
| NFR-OBS-001 | P1 | STORY-012 | ReassemblyStats 17-field struct and summarize() emission; STORY-012 owns non-TCP packet filter, statistics summary, and bytes_reassembled accounting (the foundational stats story in SS-04) |
| NFR-OBS-002 | P1 | STORY-012, STORY-046, STORY-058, STORY-066 | Uniform AnalysisSummary shape across all 4 analyzers + reassembler; STORY-012 owns reassembler summarize; STORY-046 owns HTTP analyzer summary; STORY-058 owns TLS analyzer summarize; STORY-066 owns DNS analyzer summarize |
| NFR-OBS-003 | P0 | STORY-041, STORY-052, STORY-053 | parse_errors counters in HTTP and TLS analyzers; STORY-041 owns HTTP parse error tracking; STORY-052 owns TLS parse error counting (ClientHello parsing); STORY-053 owns additional TLS parse error paths (per per-story table authority; STORY-044 removed — NFR-OBS-003 does not appear in STORY-044's per-story row) |
| NFR-OBS-004 | P1 | STORY-071 | MITRE technique IDs with None-preferred policy is in SS-10 mitre.rs; STORY-071 owns MITRE ATT&CK mapping and catalog lookup |
| NFR-OBS-005 | P1 | STORY-033 | `unclassified_flows` counter is in SS-05 dispatcher.rs:53; STORY-033 owns flow lifecycle and unclassified counter in the dispatcher |
| NFR-OBS-006 | P1 | STORY-019, STORY-089 | One-shot stderr warnings: ISN-missing (STORY-014/019), close-flow missing key (STORY-019), decode error (STORY-089), --no-reassemble conflict (STORY-088/089) |
| NFR-OBS-007 | P2 | STORY-088 | Per-target progress bar via indicatif at main.rs:153-156; STORY-088 owns run_analyze orchestration including progress bar |
| NFR-OBS-008 | P0 | STORY-078 | `--mitre` tactic grouping in terminal reporter; STORY-078 owns TerminalReporter MITRE grouping and section order |
| NFR-OBS-009 | P1 | STORY-077 | "Skipped: N packets" suppression when N=0 in terminal reporter; STORY-077 owns TerminalReporter escape_for_terminal and skipped_packets line |
| NFR-OBS-010 | P0 | STORY-070 | `skip_serializing_if = "Option::is_none"` on all 4 Option fields in Finding struct; STORY-070 owns raw-data contract and JSON serialization symmetry |
| **Resource Bounds** | | | |
| NFR-RES-001 | P0 | STORY-021 | `MAX_FINDINGS = 10_000` cap and finalize bypass; STORY-021 owns finalize lifecycle, MAX_FINDINGS cap, and segment-limit summary finding |
| NFR-RES-002 | P0 | STORY-017 | Per-flow-direction overlap-anomaly alert (overlap_alert_threshold = 50, sticky latch); STORY-017 owns conflict and evasion detection including one-shot anomaly latches |
| NFR-RES-003 | P1 | STORY-018 | Per-flow-direction small-segment alert (small_segment_alert_threshold = 100); STORY-018 owns resource bounds including depth truncation and threshold enforcement |
| NFR-RES-004 | P1 | STORY-018 | Per-flow-direction out-of-window alert (out_of_window_alert_threshold = 100); STORY-018 owns out-of-window rejection and alert latches |
| NFR-RES-005 | P0 | STORY-018, STORY-087 | Default max_depth = 10 MB (config.rs:119); STORY-018 owns depth truncation enforcement; STORY-087 owns the `--reassembly-depth` CLI flag |
| NFR-RES-006 | P0 | STORY-020, STORY-087 | Default memcap = 1 GB (config.rs:120); STORY-020 owns memory management and LRU eviction; STORY-087 owns the `--reassembly-memcap` CLI flag |
| NFR-RES-007 | P1 | STORY-019 | Default flow_timeout_secs = 300 (config.rs:121); STORY-019 owns timeout expiry (STORY-087 removed — NFR-RES-007 does not appear in STORY-087's per-story row, which is authoritative) |
| NFR-RES-008 | P1 | STORY-020 | Default max_flows = 100,000 (config.rs:122); STORY-020 owns max concurrent tracked flows and LRU eviction policies |
| NFR-RES-009 | P1 | STORY-018 | Default max_segments_per_direction = 10,000 (config.rs:123); STORY-018 owns segment limit enforcement |
| NFR-RES-010 | P0 | STORY-018, STORY-087 | Default max_receive_window = 1 MB (config.rs:124); STORY-018 owns out-of-window segment rejection; STORY-087 owns CLI override path |
| NFR-RES-011 | P0 | STORY-045 | HTTP MAX_HEADER_BUF = 65,536 cap in http.rs:21; STORY-045 owns flow lifecycle, cross-flow isolation, and buffer/map caps in SS-06 |
| NFR-RES-012 | P0 | STORY-043 | HTTP MAX_HEADERS = 96 and TooManyHeaders finding; STORY-043 owns header and method anomaly detections |
| NFR-RES-013 | P2 | STORY-045 | HTTP MAX_URIS = 10,000 list cap; STORY-045 owns buffer/map caps in SS-06 |
| NFR-RES-014 | P0 | STORY-045, STORY-058 | HTTP/TLS MAX_MAP_ENTRIES = 50,000 cardinality cap; STORY-045 owns HTTP map caps; STORY-058 owns TLS map caps (sni_counts, cipher/version maps) |
| NFR-RES-015 | P1 | STORY-058 | TLS MAX_BUF = 65,536 per-direction record buffer cap; STORY-058 owns TLS buffer management |
| NFR-RES-016 | P0 | STORY-058 | TLS MAX_RECORD_PAYLOAD = 18,432 sanity cap with RFC 5246/8446 basis; STORY-058 owns record parsing and oversized-record guard |
| NFR-RES-017 | P0 | STORY-044 | HTTP POISON_THRESHOLD = 3 (duplicate of NFR-REL-011 from resource angle); STORY-044 owns poisoning state machine |
| NFR-RES-018 | P1 | STORY-042 | HTTP URI truncation in findings (summary 120 chars, evidence 200 chars); STORY-042 owns URI-based threat detections including truncate_uri |
| NFR-RES-019 | P1 | STORY-042 | HTTP "abnormally long URI" finding threshold (> 2,048 chars); STORY-042 owns URI-based threat detections |
| NFR-RES-020 | P1 | STORY-046, STORY-058 | summarize() top-20 truncation for hosts/SNIs/URIs; STORY-046 owns HTTP analyzer summary output; STORY-058 owns TLS analyzer summarize output |
| NFR-RES-021 | P0 | STORY-051, STORY-052 | TLS handshake-only short-circuit (`done()` at tls.rs:291-293); STORY-051 owns the foundational TLS flow setup that gates the short-circuit path; STORY-052 owns ClientHello parsing, handshake counting, and done short-circuit (STORY-051 added per per-story table authority) |
| NFR-RES-022 | P1 | STORY-021 | `dropped_findings: u64` counter implementation; STORY-021 owns MAX_FINDINGS cap and finalize lifecycle; counter is in stats.rs:31 and incremented at mod.rs:477/515/539 which is in STORY-021 scope |
| NFR-RES-023 | P1 | STORY-054 | ClientHello weak-cipher evidence vec cardinality; STORY-054 owns cipher and protocol weakness findings |
| NFR-RES-024 | P1 | STORY-066 | DnsAnalyzer port dispatch, 12-byte header guard, and never-emit-findings contract; STORY-066 is the sole DNS analyzer story |
| **Maintainability** | | | |
| NFR-MNT-001 | P0 | no story owner — accepted | `RUSTFLAGS=-Dwarnings` is a CI pipeline property (ci.yml:10-12); no story governs CI configuration; this is a pure supply-chain/CI NFR |
| NFR-MNT-002 | P0 | no story owner — accepted | `cargo clippy --all-targets -- -D warnings` CI job is a pipeline property; same rationale as NFR-MNT-001 |
| NFR-MNT-003 | P0 | no story owner — accepted | `cargo fmt --check` CI job is a pipeline property; same rationale as NFR-MNT-001 |
| NFR-MNT-004 | P1 | STORY-071 | `#[non_exhaustive]` on `MitreTactic` is in mitre.rs; STORY-071 owns MITRE ATT&CK mapping and catalog |
| NFR-MNT-005 | P1 | STORY-091 | Inline test organization policy (5 modules, 34 tests); STORY-091 is the tooling/anchor-validation story; the corrected policy description is most relevant to overall project tooling |
| NFR-MNT-006 | P2 | no story owner — accepted | No per-file SPDX headers is a project-wide convention; no single story owns this |
| NFR-MNT-007 | P1 | no story owner — accepted | Semantic-PR title enforcement is a CI pipeline property (ci.yml:14-38) |
| NFR-MNT-008 | P1 | STORY-017, STORY-018 | Sticky boolean alert fields (`overlap_alert_fired`, `small_segment_alert_fired`, `out_of_window_alert_fired`) on FlowDirection; STORY-017 owns evasion detection latches; STORY-018 owns resource bound alert firing |
| NFR-MNT-009 | P1 | STORY-071 | `technique_info` as single source of truth; STORY-071 owns MITRE technique catalog |
| NFR-MNT-010 | P1 | no story owner — accepted | 4 ADRs (docs/adr/0001-0004) are architecture/documentation artifacts; no story owns ADR authorship (ADR authorship is architect scope, not a deliverable story in this cycle) |
| NFR-MNT-011 | P1 | STORY-041 | `rust-version = "1.91"` in Cargo.toml and `floor_char_boundary` at http.rs:110; STORY-041 owns HTTP analyzer including the floor_char_boundary usage that sets the effective MSRV floor |
| **Portability** | | | |
| NFR-PORT-001 | P2 | no story owner — accepted | Single-platform CI (`ubuntu-latest`) is a CI infrastructure decision; OPEN-DEBT accepted; no story in this cycle covers CI matrix expansion |
| NFR-PORT-002 | P1 | no story owner — accepted | Zero platform-specific cfg attributes is a project-wide property verified by grep audit; no single story owns this |
| NFR-PORT-003 | P1 | no story owner — accepted | No build.rs, no rust-toolchain.toml; project-wide property; not owned by any story |
| NFR-PORT-004 | P1 | STORY-089 | `env::var("NO_COLOR")` at main.rs:43; STORY-089 owns output routing and format resolution in SS-12 main |
| NFR-PORT-005 | P0 | STORY-005 | Rust 2024 edition (`edition = "2024"` Cargo.toml:4); let-chains in tls.rs:248-249 and elsewhere; STORY-005 covers packet decoding which uses 2024 edition features (let-chains in the decode path) |
| **Supply Chain** | | | |
| NFR-SUP-001 | P1 | no story owner — accepted | Unused `rayon` dep (Cargo.toml:35) is a single-line removal tracked as NFR-VIO-006; no story owns this fix (it is a dev/ops task); OPEN pending GitHub issue |
| NFR-SUP-002 | P1 | STORY-001, STORY-088 | Dev-dep usage: `assert_cmd`/`predicates`/`tempfile` used by integration tests; STORY-001 (integration test base) and STORY-088 (run_analyze, tempfile for target fixture tests) are primary consumers |
| NFR-SUP-003 | P0 | no story owner — accepted | Cargo.lock checked in is a project-wide git convention; no story owns this |
| NFR-SUP-004 | P1 | no story owner — accepted | All caret-pin deps is a Cargo.toml hygiene convention; no story owns dep version policy |
| NFR-SUP-005 | P0 | no story owner — accepted | No build-dependencies, no build.rs, no proc-macros beyond clap/serde derive is a project-wide property |
| **Compatibility** | | | |
| NFR-COMPAT-001 | P0 | STORY-001 | 5 link types accepted, all others rejected with message; STORY-001 owns PCAP file ingestion and link-type gating |
| NFR-COMPAT-002 | P0 | STORY-079, STORY-080, STORY-089 | CSV output format via CsvReporter; STORY-079 owns CsvReporter fixed schema; STORY-080 owns reporter trait compliance; STORY-089 owns output format routing (OutputFormat::Csv dispatch) |

---

## Coverage Summary

| Metric | Value |
|--------|-------|
| Total NFRs mapped | 80 (all 79 v1.2 NFRs + NFR-RES-024 added in v1.3) |
| P0 NFRs covered | 38 of 38 (100%) |
| P1 NFRs covered | 33 of 33 (100%) |
| P2 NFRs covered | 5 of 5 (remaining) |
| NFRs with story owner | 62 |
| NFRs explicitly "no story owner — accepted" | 18 |
| Stories receiving NFR back-references | 43 (45 rows in per-story table; STORY-086 and STORY-090 carry no NFR IDs) |

---

## "No Story Owner — Accepted" Rationale Summary

The 18 NFRs without a story owner fall into three categories:

1. **CI pipeline properties** (NFR-MNT-001/002/003/007, NFR-PORT-001): Enforced by `.github/workflows/ci.yml`; no story in this cycle creates or modifies CI jobs. These are delivered-by-convention, not by stories.

2. **Project-wide static properties** (NFR-PORT-002/003, NFR-SUP-003/004/005, NFR-MNT-006): Verified by grep audit or file-absence check; they have no runtime owner story because they are structural properties of the repository (no cfg attrs, no build.rs, caret deps, no SPDX headers). Correct-by-construction.

3. **Open debt tracked elsewhere** (NFR-SUP-001: rayon removal; NFR-PORT-001: multi-platform CI; NFR-MNT-010: ADR authorship): These are tracked as NFR-VIO-006 / OPEN-DEBT items in the NFR Violation Dispositions section of the catalog. They require a separate minimal fix PR, not a story.

Story-writer should propagate `nfr:` references only for the 43 stories listed in the per-story table above that carry at least one NFR ID (STORY-086 and STORY-090 carry none).

---

## Stories Receiving NFR Back-References

The following stories need `nfr:` array additions in their frontmatter (story-writer scope):

| Story ID | NFRs to Add |
|----------|-------------|
| STORY-001 | NFR-PERF-002, NFR-REL-008, NFR-COMPAT-001, NFR-SUP-002 |
| STORY-002 | NFR-PERF-001, NFR-REL-008 |
| STORY-005 | NFR-PERF-001, NFR-PORT-005 |
| STORY-011 | NFR-REL-001, NFR-REL-004, NFR-REL-003 |
| STORY-012 | NFR-OBS-001, NFR-OBS-002 |
| STORY-013 | NFR-REL-002 |
| STORY-014 | NFR-REL-002, NFR-REL-007 |
| STORY-015 | NFR-REL-002, NFR-REL-003 |
| STORY-016 | NFR-PERF-004, NFR-REL-003 |
| STORY-017 | NFR-RES-002, NFR-MNT-008 |
| STORY-018 | NFR-RES-003, NFR-RES-004, NFR-RES-005, NFR-RES-009, NFR-RES-010, NFR-MNT-008 |
| STORY-019 | NFR-REL-006, NFR-OBS-006, NFR-RES-007 |
| STORY-020 | NFR-RES-006, NFR-RES-008 |
| STORY-021 | NFR-REL-005, NFR-RES-001, NFR-RES-022 |
| STORY-031 | NFR-PERF-003 |
| STORY-032 | NFR-PERF-003 |
| STORY-033 | NFR-OBS-005 |
| STORY-041 | NFR-OBS-003, NFR-REL-003, NFR-MNT-011 |
| STORY-042 | NFR-RES-018, NFR-RES-019 |
| STORY-043 | NFR-RES-012 |
| STORY-044 | NFR-REL-011, NFR-RES-017 |
| STORY-045 | NFR-RES-011, NFR-RES-013, NFR-RES-014 |
| STORY-046 | NFR-OBS-002, NFR-RES-020 |
| STORY-051 | NFR-RES-021 |
| STORY-052 | NFR-OBS-003, NFR-RES-021 |
| STORY-053 | NFR-OBS-003 |
| STORY-054 | NFR-RES-023 |
| STORY-055 | NFR-SEC-005 |
| STORY-056 | NFR-SEC-005 |
| STORY-058 | NFR-REL-003, NFR-REL-010, NFR-RES-014, NFR-RES-015, NFR-RES-016, NFR-OBS-002, NFR-SEC-008, NFR-RES-020 |
| STORY-066 | NFR-OBS-002, NFR-RES-024 |
| STORY-069 | NFR-SEC-001 |
| STORY-070 | NFR-SEC-001, NFR-OBS-010 |
| STORY-071 | NFR-OBS-004, NFR-MNT-004, NFR-MNT-009 |
| STORY-076 | NFR-SEC-004 |
| STORY-077 | NFR-SEC-001, NFR-SEC-002, NFR-SEC-003, NFR-OBS-009 |
| STORY-078 | NFR-OBS-008 |
| STORY-079 | NFR-COMPAT-002 |
| STORY-080 | NFR-COMPAT-002 |
| STORY-086 | (none — CLI subcommand parsing; cross-cutting NFRs are owned by more specific stories) |
| STORY-087 | NFR-RES-005, NFR-RES-006, NFR-RES-010 |
| STORY-088 | NFR-OBS-007, NFR-SUP-002 |
| STORY-089 | NFR-REL-009, NFR-PORT-004, NFR-COMPAT-002, NFR-OBS-006 |
| STORY-090 | (none — summary data model; NFR coverage via parent stories in SS-12) |
| STORY-091 | NFR-MNT-005 |
