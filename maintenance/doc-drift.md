# Doc-Drift Scan — Maintenance Sweep

**Run ID:** maint-2026-07-06
**Date:** 2026-07-06
**Branch/HEAD:** develop @ f7460b4 (v0.11.4)
**Scope:** README.md, CLAUDE.md, docs/adr/0001–0011, src/lib.rs crate doc, CHANGELOG.md

---

## Summary

| Severity | Count |
|----------|-------|
| HIGH     | 1     |
| MEDIUM   | 4     |
| LOW      | 3     |
| **Total**| **8** |

---

## Prior Sweep Status (maint-2026-06-17, HEAD e1273c8 / v0.7.1)

The prior sweep filed 14 findings (4 HIGH, 6 MEDIUM, 4 LOW). Status as of this sweep:

| ID  | Description | Status |
|-----|-------------|--------|
| H-1 | ARP analyzer entirely absent from README | **FIXED** |
| H-2 | README "Multiple outputs" JSON-only claim | **FIXED** |
| H-3 | README Roadmap "CSV" still listed as future | **FIXED** |
| H-4 | ADR 0002 Existing Analyzers table stale (TLS/Modbus/DNP3/ARP missing) | **PARTIALLY FIXED** — EtherNet/IP row still missing (see N-4) |
| M-1 | Architecture diagram/table omit ARP, Modbus, DNP3 | **FIXED** |
| M-2 | ADR 005/006/007 files missing | **FIXED** |
| M-3 | lib.rs step 6 lists only DNS/HTTP/TLS | **PARTIALLY FIXED** — EtherNet/IP still absent from step 6 (see N-3) |
| M-4 | `--hosts` flag undocumented in README | **FIXED** |
| M-5 | README missing reassembly tuning flags | **FIXED** |
| M-6 | ~70 stale `RED:` comments in test files | **SUBSTANTIALLY FIXED** — 3 remain in `tests/enip_analyzer_tests.rs` (see N-7) |
| L-1 | ADR 0001 struct snippet missing Modbus/DNP3 | **PARTIALLY FIXED** — Modbus and DNP3 added; EtherNet/IP Rule 7 still absent (see N-6) |
| L-2 | Unused `rayon` dependency in Cargo.toml | **FIXED** |
| L-3 | CHANGELOG v0.4.0 T0855 annotation | **PERSISTS** (see N-8) |
| L-4 | ADR 0003 stale line number reference | **FIXED** |

**10 of 14** prior findings fully resolved. 4 partially resolved or persisting, carrying forward as new findings below.

---

## HIGH Severity

### N-1 — README: `protocols` subcommand entirely absent

**File:** `README.md`
**What's stale:** The binary has a third top-level subcommand `protocols` (`wirerust protocols`)
that prints a filterable table of all known ICS/IT protocols — supported and planned coverage —
with `--supported`, `--unsupported`, and `--all` filter flags. It also accepts the global `--json`
flag for machine-readable output.

The README "Options" block at lines 74–96 shows only two commands:
```
Commands:
  analyze   Analyze PCAP files for threats and anomalies
  summary   Generate a triage summary of PCAP files
```

Actual `wirerust --help` output (v0.11.4):
```
Commands:
  analyze    Analyze PCAP files for threats and anomalies
  summary    Generate a triage summary of PCAP files
  protocols  List the protocol coverage catalog
  help       Print this message or the help of the given subcommand(s)
```

A user reading the README has no way to discover `protocols` exists without running `--help`.

**Source:** `src/cli.rs` lines 123–134, 288–298.

**Automated fix feasible:** Yes — add a `### List protocol coverage` section with a usage
example and flag descriptions for `--supported` / `--unsupported`.

---

## MEDIUM Severity

### N-2 — README: `--json [FILE]` and `--csv [FILE]` standalone file-write flags undocumented

**File:** `README.md` lines 83–96 (Options block)
**What's stale:** The README documents `--output-format <FMT>` for switching between terminal,
JSON, and CSV output modes. The actual CLI additionally offers `--json [<JSON>]` and
`--csv [<CSV>]` as distinct global flags that write output directly to a file path (or to stdout
when no path is given), mutually exclusive with each other. These have different semantics from
`--output-format` — they are file-writing shortcuts with an optional path argument. Neither flag
appears in the README Options block.

**Source:** `wirerust --help` output; `src/cli.rs`.

**Automated fix feasible:** Yes — add entries to the Options block and a usage example such as
`wirerust analyze capture.pcap --all --json findings.json`.

---

### N-3 — lib.rs crate doc step 6 missing EtherNet/IP analyzer

**File:** `src/lib.rs` line 22
**What's stale:**
```rust
//! 6. **[`analyzer`]** (DNS / HTTP / TLS / Modbus / DNP3 / ARP) emits
```
EtherNet/IP (ENIP) — `src/analyzer/enip.rs`, enabled via `--enip`, documented at length in
README §EtherNet/IP CIP Analyzer and ADR-010 — is absent from the parenthetical list. The
same line's continuation also reads "stream-level (HTTP, TLS, Modbus, DNP3)" without listing
EtherNet/IP.

**Automated fix feasible:** Yes — add "/ EtherNet/IP" to both parentheticals on that line.

---

### N-4 — ADR 0002: Existing Analyzers table missing EtherNet/IP row

**File:** `docs/adr/0002-modular-protocol-analyzers.md` lines 147–156
**What's stale:** The table was updated in the prior sweep through ARP (v0.7.0), but the
EtherNet/IP CIP analyzer (`src/analyzer/enip.rs`, covered by ADR-010) is not listed. The
Deviations subsection at lines 158–178 documents DNP3 and ARP dispatch deviations but not
EtherNet/IP, which is also dispatched via inherent methods rather than the `StreamAnalyzer`
trait.

**Automated fix feasible:** Yes — add a row for EtherNet/IP and a deviation note matching the
pattern of the DNP3 entry.

---

### N-5 — README: New observability counters (PR #365/#366) undocumented

**Files:** `README.md` (ARP section ~lines 188–205, Modbus feature bullet ~line 13,
TLS/HTTP feature bullets)
**What's stale:** PR #365 and the PR #366 follow-up surfaced four new per-analyzer JSON output
counters that appear in each analyzer's JSON detail output. None are mentioned in the README
sections describing those analyzers:

- **`bindings_evicted`** (`src/analyzer/arp.rs:800`) — ARP analyzer JSON detail. Counts
  IP→MAC binding-table LRU evictions. Not mentioned in README ARP section.
- **`storm_counters_evicted`** (`src/analyzer/arp.rs:804`) — ARP analyzer JSON detail. Counts
  per-MAC storm-counter-table LRU evictions. Not mentioned in README ARP section.
- **`dropped_transactions`** (`src/analyzer/modbus.rs:977`) — Modbus analyzer JSON detail.
  Counts silently-dropped Modbus transactions when the per-flow transaction map hits its cap.
  Not mentioned in README.
- **`dropped_map_entries`** (`src/analyzer/http.rs:644`, `src/analyzer/tls.rs:1286`) — HTTP and
  TLS analyzer JSON detail. Counts silently-dropped map entries when per-analyzer counter maps
  hit their cap. Not mentioned in README.

The ENIP section's `enip_summary` seven-field list is accurate and is not affected by this
finding.

**Automated fix feasible:** Yes — add a brief JSON output fields note to the ARP, Modbus, HTTP,
and TLS sections, or a shared "Observability counters in JSON output" subsection.

---

## LOW Severity

### N-6 — ADR 0001: Classification rules and struct snippet missing EtherNet/IP Rule 7

**File:** `docs/adr/0001-content-first-stream-dispatch.md` lines 28–48, 50–58
**What's stale:** The `StreamDispatcher` struct snippet lists `modbus` (Rule 5) and `dnp3`
(Rule 6) but not `enip` (Rule 7, port 44818). The `DispatchTarget` enum shows `Http`, `Tls`,
`Modbus`, `Dnp3`, `None` — missing `Enip`. The classification rule list ends at 7 entries with
"No match → None" at position 7, but README states EtherNet/IP is Rule 7 (dispatched after
port-20000 DNP3), making "No match → None" Rule 8.

ADR-010 was created for the EtherNet/IP design decision but ADR-0001 was never amended.

**Automated fix feasible:** Yes — add `enip: Option<EnipAnalyzer>` / `Enip` variant to the
snippets, add Rule 7 (Port 44818 → Enip) to the classification table, and add a cross-reference
to ADR-010.

---

### N-7 — 3 residual stale `RED:` comments in enip_analyzer_tests.rs

**File:** `tests/enip_analyzer_tests.rs` lines 7478, 7488, 7498
**What's stale:** Three `// RED: pre-fix code returns 0/{}` comments remain. These describe
pre-fix behavior that no longer applies — all three tests currently pass. Down from ~70
occurrences across 5 files in the prior sweep.

**Example (line 7478):**
```rust
// RED: pre-fix code returns 0 (reads self.total_pdu_count == 0).
```

**Automated fix feasible:** Yes — relabel to `// WAS RED (pre-fix):` or remove entirely.

---

### N-8 — CHANGELOG v0.4.0 uses revoked MITRE ID T0855 without annotation (persists)

**File:** `CHANGELOG.md` line 703
**What's stale:** The v0.4.0 changelog entry lists `T0855 Unauthorized Command Message`
(write-class function codes). T0855 was remapped to T1692.001 in v0.5.0 (CHANGELOG line 688).
The v0.4.0 entry is historically accurate but no annotation links it to the remap.

This finding persisted unfixed from the prior sweep and remains unchanged.

**Automated fix feasible:** Yes — add `(→ remapped to T1692.001 in v0.5.0)` to the v0.4.0
T0855 line.

---

## No Issues Found

- **CLAUDE.md build/test/lint commands** — verified correct against Cargo.toml (edition 2024,
  `rust-version = "1.91"`, single-crate, `cargo test --all-targets`, clippy `-D warnings`).
- **CLAUDE.md git workflow** — branch naming, gitflow, semantic PR, SHA-pin policy all consistent
  with CI config.
- **CLAUDE.md input-hash section** — algorithm description matches `bin/compute-input-hash`.
- **README version references** — no stale explicit version numbers; `Cargo.toml` shows 0.11.4.
- **README feature bullets** — all seven protocol analyzers (DNS, HTTP, TLS, Modbus, DNP3, ARP,
  EtherNet/IP) present and accurately described including MITRE technique IDs.
- **README protocol coverage table** — all seven rows accurate (ports, flags, MITRE techniques).
- **README TLS `buffer_saturation_drops` telemetry mention** — accurate; this is a separate
  counter from the new `dropped_map_entries` (both exist in `src/analyzer/tls.rs`).
- **README ENIP `enip_summary` seven-field list** — verified against source; accurate.
- **README Supported Capture Formats / Link Types** — accurate.
- **README architecture diagram and component table** — accurate as of v0.11.4.
- **README DNP3/ENIP per-flow state purge (v0.11.3, PR #342)** — no README claim contradicts
  this fix; no stale "state not purged" language found.
- **ADR 0002 Deviations section** — DNP3 and ARP dispatch deviations correctly documented.
- **ADR 0003** — reporting-pipeline layering and escaping boundary accurate; prior L-4 fixed.
- **ADR 0004** — AtomicBool guards and test seams accurate.

---

## Remediation Priority

| Priority | Finding | Effort |
|----------|---------|--------|
| 1 | N-1: `protocols` subcommand absent from README | Low (add 1 section + example) |
| 2 | N-5: New observability counters undocumented | Medium (notes in 4 analyzer sections) |
| 3 | N-2: `--json`/`--csv` file-write flags missing | Low (add 2 lines + example) |
| 4 | N-4: ADR 0002 missing EtherNet/IP row | Low (1 table row + deviation note) |
| 5 | N-3: lib.rs step 6 missing EtherNet/IP | Trivial (1 line) |
| 6 | N-6: ADR 0001 missing ENIP Rule 7 | Low (update struct/enum/rules snippets) |
| 7 | N-7: 3 residual RED: comments | Trivial (3 comment relabels) |
| 8 | N-8: CHANGELOG T0855 annotation | Trivial (add footnote) |
