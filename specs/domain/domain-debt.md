---
artifact: L2-domain-debt
traces_to: domain-spec.md
title: Known Limitations and Domain Debt
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
version: "1.2"
modified:
  - date: 2026-06-10
    actor: architect
    reason: "O-04 staged IDs: remove T0855 (now T1692.001, emitted) and T0856 (now T1692.002, still staged); update staged list and catalog count to reflect F2 Modbus expansion (issue #222)."
  - date: 2026-06-13
    actor: product-owner
    reason: "ARP-F2 Pass-14 remediation C-04: O-01 moved to RETIRED section — STORY-097 (thread capture-relative timestamp through StreamHandler::on_data), STORY-098, and STORY-099 wired timestamp at all http.rs/tls.rs/reassembly emission sites; STORY-102..110 wired timestamp in modbus/dnp3 analyzers. O-01 is fully resolved (Option A completed). Version 1.1→1.2."
---

# Known Limitations / Domain Debt

This shard documents observable gaps, known bugs, and technical debt in the shipped wirerust
codebase as of develop HEAD (post remediation-cycle PRs #69-#98). Items are presented
honestly. None are silently omitted or presented as intended behavior.

**Remediation status:** The brownfield-ingest Phase C produced 30 prioritized lessons
(5 P0, 7 P1, 11 P2, 7 P3). All 30 were delivered in PRs #69-#98. Many debt items from
the initial spec draft have been retired. The remaining open items below reflect the state
of develop today.

---

## RETIRED ITEMS (closed by remediation; recorded for traceability)

| Former ID | Description | Closed by |
|---|---|---|
| D-01 | No impl Drop on TcpReassembler | #72: impl Drop tripwire + run_analyze IIFE so finalize() always runs |
| D-02 | MAX_FINDINGS silent truncation with no counter | #73: dropped_findings: u64 added to ReassemblyStats |
| D-04 | --json/--csv flags do not write files | #70: write_output() wired; #84: CSV reporter fully implemented |
| D-05 (partial) | Missing-Host empty-value evasion | #71: both None and Some("") Host values now detected |
| D-09 | TlsAnalyzer has no truncation counter | #73: truncated_records: u64 added to TlsAnalyzer |
| D-10 | CsvReporter unwired; csv dep unused | #84: CsvReporter implemented with CSV-injection neutralization; csv dep now used. NOTE: rayon = "1" was NOT removed -- it remains in Cargo.toml (line 28) but is not imported in src/. See open item O-07. |
| D-11 | JSON map key ordering non-deterministic | #76: BTreeMap used in JsonReporter for deterministic key order |
| D-12 | ThreatCategory not #[non_exhaustive] | #76: #[non_exhaustive] added to ThreatCategory, Verdict, and Confidence |
| D-13 | Unwired CLI flags advertised in --help | #74: 5 dead flags removed; #93/#96: threshold flags added and wired |
| D-14 | pcapng files matched by glob but rejected by reader | #69: *.pcapng removed from resolve_targets directory glob |
| D-15 | MSRV undeclared in Cargo.toml | #69: rust-version = "1.91" declared |
| D-16 | 8 of 14 pcap test fixtures unused | #86: tests/fixtures/README.md added; nfs_bad_stalls.cap re-added (#90) |
| D-17 | Zero //! module headers in 19 of 20 modules | #75: //! headers on all 20 modules; #![warn(missing_docs)] added |
| D-07 (partial) | Anomaly thresholds unjustified round numbers | #88/#92/#93/#96: thresholds moved to ReassemblyConfig fields, CLI-overridable, research-documented |
| O-01 | Finding.timestamp universally None (forensic gap) | STORY-097 (thread capture-relative timestamp through StreamHandler::on_data) + STORY-098/099 wired http/tls/reassembly; STORY-102..110 wired modbus/dnp3. Option A fully complete. |


---

## OPEN ITEMS (genuine debt on develop today)

### O-02: Absent User-Agent Intentionally Not Detected (documented asymmetry)

**What exists:** The empty-Host evasion was closed by #71 -- both `None` and `Some("")`
Host values now fire findings (with different summary text: "without Host header" vs.
"with empty Host header"). The UA detection is intentionally asymmetric: only `Some("")`
(present-empty) fires; absent UA (`None`) does not.

**Rationale (research-cited in http.rs:319-343):** Many legitimate clients omit UA entirely
(cron jobs, internal microservices, healthchecks, embedded libraries). Snort's missing-UA
rule (sid 1:38130) ships disabled by default. Kheir (2015) reports ~24% of malware samples
emit an empty UA string rather than omitting the header. This is a documented design
decision, not a bug.

**References:** P0.05 (#71); http.rs:319-343 inline doc.


### O-03: Anomaly Thresholds Not Empirically Calibrated Against Labelled Traffic (P2)

**What exists:** `ReassemblyConfig` thresholds (`overlap_alert_threshold=50`,
`small_segment_alert_threshold=100`, `small_segment_max_bytes=16`,
`out_of_window_alert_threshold=100`) are CLI-overridable and research-documented in
config.rs (P2.05 via #88, #92, #93, #96). However, no labelled capture corpus (benign +
adversarial) exists to measure FP/TP rates and validate the defaults empirically.

**Remaining follow-up (STATE.md drift item 2):** A port-independent directional-symmetry
discriminator for the small-segment detector would make `small_segment_ignore_ports`
advisory rather than load-bearing. Research flagged as sound but not yet implemented.

**References:** LESSON-P2.05; config.rs doc comments; STATE.md drift items 1 and 2.


### O-04: 8 MITRE Techniques Catalogued but Never Emitted (documentation debt)

**What exists:** `technique_info` in mitre.rs contains 21 IDs (15 brownfield + 6 Modbus ICS
techniques added in Feature #7); 8 are never referenced by any current analyzer. The mitre.rs
doc comment now says "staged for future analyzers" (P3.04, #89). No analyzer wiring was added
for these 8 IDs.

**Staged IDs:** T1040, T1071, T1071.001, T1071.004, T1573, T0846, T1692.002, T0885.

Note: T1692.001 (formerly T0855, revoked in ATT&CK-ICS v19) IS actively emitted by the Modbus
analyzer (Feature #7) and is NOT in this staged list. T1692.002 (formerly T0856) remains
catalogued-only and stays in this list.

**References:** LESSON-P3.04; mitre.rs doc comment; CAP-10.


### O-05: reassembly/mod.rs Still ~691 LOC (partial split)

**What exists:** P2.01 (#85) split the original 565-LOC `mod.rs` into `config.rs`,
`stats.rs`, and `lifecycle.rs`. `mod.rs` now measures ~691 LOC because the
`process_packet` decomposition added named helper structs and sub-steps. Config and stats
are cleanly extracted; the engine hot path remains in `mod.rs` by design.

**References:** Smell #1 (partially closed); P2.01 (#85).


### O-06: Weak-Cipher Finding Evidence Vec Has Unbounded Cardinality (NFR-RES-023)

**What exists:** The ClientHello weak-cipher Finding at `src/analyzer/tls.rs` uses
`evidence: weak` where `weak: Vec<String>` is built by filtering ClientHello cipher suites.
Upper bound: ~9,216 cipher names (MAX_RECORD_PAYLOAD / 2 bytes per cipher). Worst-case
Finding heap ~270-500 KB. No per-cipher truncation cap exists.

**References:** NFR-RES-023; pass-4 R2 Target 10.3.


### O-07: rayon Declared in Cargo.toml but Never Imported (unused dependency)

**What exists:** `rayon = "1"` appears in `[dependencies]` at Cargo.toml:28 but no module
in `src/` imports it. The D-10 retirement entry incorrectly stated rayon was removed; it was
not. The dep contributes a transitive closure of crates to the build graph without providing
any functionality.

**Observable consequence:** Cargo will not warn about unused dependencies by default
(unlike unused imports in src/). The dep will be included in lock-file resolution and may
appear in `cargo tree` output. There is no runtime impact.

**Fix:** Remove `rayon = "1"` from `[dependencies]` in Cargo.toml, or add a code comment
explaining why it is retained (e.g., planned future parallelism). Estimated effort: trivial.

**References:** Cargo.toml:28; architecture smell #8 (partially closed by #84).


### O-08: dns.rs Module Doc-Comment Describes Unimplemented Detection (stale aspirational docs)

**What exists:** `src/analyzer/dns.rs` lines 1-7 carry a module doc-comment that asserts the
analyzer "pars[es] the question section to extract qnames" and tracks "DGA-class entropy on
labels, unusually long subdomains, NXDOMAIN spikes, and rare-TLD lookups" with "findings
carry[ing] confidence levels". None of this is implemented. `DnsAnalyzer` (lines 62-70) only
inspects the QR bit (byte 2, bit 7) via `is_query()`, increments one of two counters
(`query_count` / `response_count`), and unconditionally returns `Vec::new()`. The struct
(lines 15-18) has exactly two fields and no qname buffer, entropy accumulator, TLD set, or
NXDOMAIN counter. `analyze()` never constructs a `Finding`.

**Severity:** Low -- documentation debt only. The spec is correct: BC-2.08.001-004, VP-019,
and CAP-08 all accurately describe DNS as statistics-only. This is a stale/aspirational
SOURCE doc-comment, not a spec defect. Architecture Smell #5 ("DnsAnalyzer::analyze returns
empty Vec -- statistics-only by design") already acknowledges the statistics-only design; it
does not record the discrepancy between that design and the misleading doc-comment text.

**Observable consequence:** A developer reading only the module-level doc-comment will expect
qname extraction, entropy scoring, NXDOMAIN detection, and confidence-level findings. None of
these are present. The discrepancy can cause misaligned expectations during code review, onboarding,
or when adding future DNS detection capabilities.

**Source location:** `src/analyzer/dns.rs` lines 1-7 (//! module doc-comment).

**Fix:** Replace the module doc-comment with text that accurately describes the actual behavior:
statistics-only QR-bit discrimination producing `dns_queries` and `dns_responses` counters,
always returning an empty findings Vec. If DGA/entropy/NXDOMAIN detection is planned,
document it explicitly as future work rather than present behavior.

**References:** Adversarial-review pass-29 observation O-1; Architecture Smell #5; BC-2.08.001-004; VP-019; CAP-08.


---

## Architecture Smell Status (updated, develop HEAD)

| # | Smell | Original severity | Current status |
|---|---|---|---|
| 1 | reassembly/mod.rs god-module | medium | PARTIALLY CLOSED (#85): config/stats/lifecycle extracted; mod.rs still 691 LOC (see O-05) |
| 2 | Process-wide one-shot AtomicBool warning guards | low | DOCUMENTED (#89): ADR 0004 added; pattern is intentional |
| 3 | Unwired CLI flags | high | CLOSED (#74/#93/#96): all current flags are wired |
| 4 | L2->L3 trait coupling (intrinsic to ADR 0002) | advisory | UNCHANGED: accepted by ADR 0002 |
| 5 | DnsAnalyzer::analyze returns empty Vec | low | UNCHANGED: statistics-only by design |
| 6 | StreamDispatcher pub field exposure | low | UNCHANGED |
| 7 | pcap_file::DataLink leaks across crate boundary | low | UNCHANGED |
| 8 | csv + rayon declared, never imported | low | PARTIALLY CLOSED (#84): csv now used by CsvReporter. rayon = "1" remains in Cargo.toml but is not imported in src/. See O-07. |
| 9 | No impl Drop on TcpReassembler | high | CLOSED (#72): impl Drop tripwire + IIFE finalize guarantee |
| 10 | Loose TLS gate (byte[2] unchecked) | low | UNCHANGED: theoretical; zero misroute tests |
