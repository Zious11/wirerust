# Doc-Drift Scan — Maintenance Sweep 2

**Date:** 2026-06-17
**Branch/HEAD:** develop @ e1273c8 (v0.7.1)
**Scope:** README.md, docs/adr/0001–0004, CLAUDE.md, CHANGELOG.md, lib.rs module docs

---

## Summary

| Severity | Count |
|----------|-------|
| HIGH     | 4     |
| MEDIUM   | 6     |
| LOW      | 4     |
| **Total**| **14**|

All findings are documentation-only; no runtime behavior is affected. Automated-fix
feasibility is noted per finding.

---

## HIGH Severity

### H-1 — README: ARP analyzer entirely absent

**File:** `README.md`
**What's stale:** The ARP security analyzer (`--arp`) shipped in v0.7.0 (STORY-111–115) and is
the most recent major feature. It is completely absent from:
- The "Features" bullet list (line 10 says "DNS, HTTP, TLS, Modbus, and DNP3" — ARP missing)
- The "Multiple outputs / Fast" summary line 18 still says "JSON export" not "JSON / CSV export"
- The "Analyze flags" block (lines 87–97) has no `--arp`, `--arp-spoof-threshold`,
  `--arp-storm-rate` entries
- The "Supported Protocol Analyzers" table (lines 123–129) has no ARP row
- There is no "ARP Analyzer" subsection equivalent to the DNP3 subsection at lines 131–155
- The architecture component table (lines 111–119) has no ARP parser/analyzer entry

**Actual CLI flags not documented:**
```
--arp
--arp-spoof-threshold N   (default: 3)
--arp-storm-rate N        (default: 50)
```
**MITRE techniques not documented:** T0830 (Adversary-in-the-Middle), T1557.002 (ARP Cache
Poisoning)

**Automated fix feasible:** Yes — add ARP row to protocol table, add ARP bullet to Features,
add three flags to the Analyze flags block, and add a subsection for ARP detections.

---

### H-2 — README: "Multiple outputs" claims only "JSON export"; CSV shipped in v0.1.0

**File:** `README.md` line 18
**What's stale:**
```
- **Multiple outputs** — colored terminal, JSON export
```
CSV output has been fully implemented since v0.1.0 (CSV reporter at
`src/reporter/csv.rs`; `--csv [FILE]` and `--output-format csv` both functional).
The options block on line 75 already shows `--output-format <FMT>  Output format: json, csv`
which contradicts line 18.

**Automated fix feasible:** Yes — change "JSON export" to "JSON and CSV export".

---

### H-3 — README: Roadmap item "CSV and SQLite export" is partially shipped

**File:** `README.md` lines 219–222
```
## Roadmap
- C2 beaconing detection
- CSV and SQLite export
- Parallel file processing
- pcapng format support
```
CSV export shipped in v0.1.0. The roadmap item should be split: "SQLite export" remains
future work; "CSV export" is done and should be removed. Leaving "CSV and SQLite export"
as a combined item implies CSV is still unshipped, misleading users scanning the roadmap.

**Automated fix feasible:** Yes — replace "CSV and SQLite export" with "SQLite export".

---

### H-4 — ADR 0002: "Existing Analyzers" table is 4 releases out of date

**File:** `docs/adr/0002-modular-protocol-analyzers.md` lines 142–146
```
| Analyzer | Trait | File | Since |
|----------|-------|------|-------|
| DNS | `ProtocolAnalyzer` | `src/analyzer/dns.rs` | v0.1.0 |
| HTTP | `StreamAnalyzer` | `src/analyzer/http.rs` | v0.1.0 |
| TLS | `StreamAnalyzer` | `src/analyzer/tls.rs` | Issue #2 (planned) |
```
Reality as of v0.7.1:
- TLS: shipped in v0.1.0 (not "planned"; `src/analyzer/tls.rs` exists)
- Modbus: shipped in v0.4.0, `StreamAnalyzer`, `src/analyzer/modbus.rs`
- DNP3: shipped in v0.6.0, `StreamAnalyzer`, `src/analyzer/dnp3.rs`
- ARP: shipped in v0.7.0, `ProtocolAnalyzer`, `src/analyzer/arp.rs`

The "Issue #2 (planned)" annotation for TLS is the most misleading — it actively tells
a reader that TLS is not yet implemented when it has been for the entire project lifetime.

**Automated fix feasible:** Yes — add rows for TLS (fix "planned" to version), Modbus,
DNP3, and ARP.

---

## MEDIUM Severity

### M-1 — README: Architecture diagram and component table omit ARP, Modbus, DNP3 dispatching

**File:** `README.md` lines 101–119
The architecture ASCII diagram (lines 101–108) shows:
```
Reassembly Engine → StreamDispatcher → StreamAnalyzers (HTTP, TLS)
```
The actual dispatcher routes to HTTP, TLS, Modbus (port 502), and DNP3 (port 20000).
ARP is a packet-level analyzer (not in the dispatcher) but is also absent.

The component table (lines 111–119) lists `tls-parser` but has no entries for the Modbus
or DNP3 parsers (both implemented in-tree). The Output row says "Terminal + JSON" but
CSV is also a supported output.

**Automated fix feasible:** Yes — update diagram and table.

---

### M-2 — ADR 0005 and ADR 0006 referenced in code/CHANGELOG but files are missing

**Files:** `src/dispatcher.rs` lines 9, 24–25, 213, 231, 236, 450, 455;
           `src/findings.rs` line 142; `src/reporter/csv.rs` line 82;
           `CHANGELOG.md` lines 202–203
**What's stale:** ADR-005 ("Binary ICS protocol integration strategy") and ADR-006
("Multi-technique Finding attribution model") are cited by name throughout the codebase
and were recorded as "Added" in the CHANGELOG v0.4.0 entry. However, the files
`docs/adr/0005-*.md` and `docs/adr/0006-*.md` do not exist. Similarly, ADR-007 is
referenced in `src/dispatcher.rs` (lines 9, 25, 231, 236) for the DNP3 dispatching
decision, but `docs/adr/0007-*.md` does not exist either.

The CLAUDE.md "Project References" table lists only `docs/adr/` without noting that
ADRs 005–007 are referenced but unwritten.

**Impact:** A contributor following a `// ADR-005` comment cannot find the decision record.
The CHANGELOG entry for v0.4.0 claims these ADRs were added.

**Automated fix feasible:** No — the ADRs need to be authored. This is a documentation
debt item, not a simple text correction.

---

### M-3 — lib.rs module doc: step 6 lists only "DNS / HTTP / TLS"; Modbus, DNP3, ARP missing

**File:** `src/lib.rs` line 22
```
6. **[`analyzer`]** (DNS / HTTP / TLS) emits per-flow [`findings::Finding`]s.
```
As of v0.7.1, the analyzer module contains DNS, HTTP, TLS, Modbus, DNP3, and ARP analyzers.
The pipeline description in the crate-level doc is incomplete.

**Automated fix feasible:** Yes — update the parenthetical to list all six analyzers.

---

### M-4 — README: Analyze flags block has no `--hosts` flag for `summary` subcommand

**File:** `README.md` lines 58–63 ("Generate a summary" section)
The summary usage example shows only `wirerust summary capture.pcap` and
`wirerust summary /path/to/pcaps/ --output-format json`. The `--hosts` flag (wired per
LESSON-P1.03, `src/cli.rs` line 219) is not documented anywhere in the README.

**Automated fix feasible:** Yes — add a usage example and flag description for `--hosts`.

---

### M-5 — README: "Analyze flags" options block omits all reassembly threshold overrides

**File:** `README.md` lines 84–97 ("Analyze flags" block)
The "Analyze flags" block documents protocol-specific flags but omits the reassembly
tuning flags (`--overlap-threshold`, `--small-segment-threshold`,
`--small-segment-max-bytes`, `--small-segment-ignore-ports`, `--out-of-window-threshold`,
`--flow-timeout`) which are shared global options. The global "Options" block on lines
66–81 documents `--reassemble`/`--no-reassemble`/`--reassembly-depth`/`--reassembly-memcap`
but also omits the finer-grained reassembly thresholds.

This is a completeness gap rather than an error. The flags exist and work; the README
just leaves them undiscovered without `--help`.

**Automated fix feasible:** Yes — add a "Reassembly tuning" subsection or extend the Options block.

---

### M-6 — Stale "RED:" comments in passing test files (issue #254 scope)

**Files (with RED: occurrence counts):**
- `tests/dnp3_f5_remediation_tests.rs`: 50 occurrences
- `tests/bc_2_15_110_dnp3_dispatcher_tests.rs`: 8 occurrences
- `tests/dnp3_detection_tests.rs`: 6 occurrences
- `tests/cli_story_087_tests.rs`: 4 occurrences
- `tests/dnp3_flow_state_tests.rs`: 2 occurrences

**Total:** ~70 "RED:" comments across 5 files. All these tests currently pass (confirmed:
`cargo test --all-targets` is green). The "RED:" inline comments describe behavior that
was buggy before the corresponding fix PR; they are now stale because the code has been
corrected and the tests pass GREEN.

**Example from `dnp3_f5_remediation_tests.rs` line 80:**
```rust
// RED: currently is_master_frame(0xC4) returns false (buggy 0x10 mask).
```
The `0x10` mask bug has been fixed; `is_master_frame(0xC4)` now returns true, and the
test passes. The "RED:" label is misleading — it implies the test is currently failing.

**CLAUDE.md note:** The task description mentions "issue #254 (repo-wide stale RED-gate
prose in passing tests) is already known." This finding confirms the scope: ~70 occurrences
across 5 files. The comments describe the pre-fix broken behavior as if it is still
current. A reader trying to understand a test failure would be confused.

**Automated fix feasible:** Partially — "RED:" labels in completed remediation tests can
be relabeled to "WAS RED:" or "RED (pre-fix):" to clarify historical context vs. current
state. However, this requires careful review to distinguish legitimate in-progress RED
tests from historical ones. Scope is confirmed at approximately 70 occurrences.

---

## LOW Severity

### L-1 — ADR 0001: StreamDispatcher struct code snippet is out of date

**File:** `docs/adr/0001-content-first-stream-dispatch.md` lines 28–56
The `StreamDispatcher` struct in ADR 0001 shows only `http` and `tls` fields. The actual
struct in `src/dispatcher.rs` also has `modbus` and `dnp3` fields, and the
`DispatchTarget` enum has `Modbus` and `Dnp3` variants. The ADR documents the original
design; subsequent extensions (ADR-005, ADR-007) should have amended it or the ADR
should note it is a baseline description only.

This is LOW because the ADR records a design decision that was correct at the time, and
the "Consequences" section's extension mechanism ("Future analyzers register content
signatures in the dispatcher") is what happened. The struct snippet is illustrative, not
normative. Still, a contributor reading ADR 0001 without reading ADR-005/007 (which are
missing, see M-2) would see an incomplete struct.

**Automated fix feasible:** Yes — add an amendment section noting the struct has grown;
or note that the snippet is a baseline and point to the source file for the current shape.

---

### L-2 — `rayon` dependency in Cargo.toml is unused in source

**File:** `Cargo.toml`
`rayon = "1"` is listed as a dependency, but no `use rayon::` or `rayon::*` call exists
anywhere in `src/`. The "Parallel file processing" item appears in the README Roadmap,
suggesting rayon was added in anticipation of that feature. Carrying an unused dependency
slightly bloats compile times and the binary.

**Automated fix feasible:** Yes — remove the `rayon` dependency until parallel processing
is actually implemented. This would also resolve any `cargo deny` advisory risk if rayon
acquires a known vulnerability before it is used.

---

### L-3 — CHANGELOG v0.4.0 uses the pre-remap technique ID T0855 for Modbus

**File:** `CHANGELOG.md` lines 168–169
```
- T0855 Unauthorized Command Message (write-class function codes)
```
T0855 was remapped to T1692.001 in v0.5.0 (CHANGELOG lines 151–154). The v0.4.0 changelog
entry uses the revoked ID, which is historically correct for what was shipped in v0.4.0
but could confuse readers scanning for T0855 references. The v0.5.0 entry correctly
documents the remap. This is a LOW finding because historical changelogs describe what
was shipped at the time; it is not incorrect, just potentially confusing.

**Automated fix feasible:** Yes — add a footnote or `→ remapped to T1692.001 in v0.5.0`
annotation to the v0.4.0 entry.

---

### L-4 — ADR 0003 consequence table references `src/analyzer/tls.rs` line ~349

**File:** `docs/adr/0003-reporting-pipeline-layering.md` lines 180–183
```
| `src/analyzer/tls.rs` | Replace `{hostname:?}` (line ~349) ...
```
The "~" acknowledges approximate line numbers at writing time, but the file has evolved
since. Line 349 of `src/analyzer/tls.rs` no longer contains `{hostname:?}` — the fix was
applied in the introducing PR. The consequence table is a historical record of what the PR
changed; the `~` prefix signals approximation. This is a LOW informational note only — the
fix was applied; the ADR is just slightly misleading to someone checking "has this been
done?" by looking at that line number.

**Automated fix feasible:** Yes — remove or update the line number annotation, or add a
note "applied" to indicate the consequence was realized.

---

## No Issues Found

- **CLAUDE.md build/test/lint commands** — all verified correct against the current Cargo
  setup (edition 2024, `rust-version = "1.91"`, single-crate).
- **CLAUDE.md git workflow** — branch naming, gitflow, semantic PR enforcement, SHA-pin
  policy — all consistent with CI config.
- **CHANGELOG completeness** — v0.7.1, v0.7.0, v0.6.0 entries accurately describe shipped
  changes. v0.5.0 MITRE remap and v0.3.0 scalar→array migration are correctly documented.
- **CLAUDE.md input-hash section** — algorithm description matches `bin/compute-input-hash`
  implementation.
- **ADR 0004** — accurately describes the three AtomicBool guards, the two Wave-7/8
  amendments (STORY-014, STORY-019 test seams), and the force_set_flow_state seam class.
  The `_for_testing` function names and `#[doc(hidden)]` constraints are current.
- **CLAUDE.md public-API tracking note (W7.1)** — deferred status is correctly recorded;
  no drift.
- **Link types in README** — Ethernet/Raw IP/SLL/IPv4/IPv6 table is accurate.
- **CLAUDE.md CI action-pin policy** — described correctly; dtolnay exemption matches
  actual CI config.

---

## Remediation Priority

| Priority | Finding | Effort |
|----------|---------|--------|
| 1 | H-1: ARP analyzer missing from README | Medium (add ~30 lines) |
| 2 | H-4: ADR 0002 Existing Analyzers table | Low (4 table rows) |
| 3 | H-2: README "Multiple outputs" JSON-only claim | Trivial (1 word change) |
| 4 | H-3: README Roadmap "CSV" still listed as future | Trivial (1 line) |
| 5 | M-2: ADR 005/006/007 files missing | High (author 3 ADRs) |
| 6 | M-3: lib.rs crate doc lists only DNS/HTTP/TLS | Trivial (1 line) |
| 7 | M-4: `--hosts` flag undocumented in README | Low (add 2 lines) |
| 8 | M-1: README architecture diagram/table stale | Low (update 2 sections) |
| 9 | M-5: README missing reassembly tuning flags | Medium (add subsection) |
| 10 | M-6: ~70 stale "RED:" comments (issue #254) | Medium (batch relabel) |
| 11 | L-2: Unused `rayon` dependency | Trivial (remove 1 line) |
| 12 | L-1: ADR 0001 struct snippet incomplete | Low (add amendment note) |
| 13 | L-3: CHANGELOG v0.4.0 T0855 annotation | Trivial (add footnote) |
| 14 | L-4: ADR 0003 stale line number reference | Trivial (add "applied" note) |
