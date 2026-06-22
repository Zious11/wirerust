# Documentation Drift Findings — Maintenance Sweep 2

**Date:** 2026-06-22
**Branch/HEAD:** develop @ dd3b069 (v0.9.3)
**Scope:** README.md, docs/adr/0001–0007, CLAUDE.md, src/ module structure, src/ TODO/FIXME scan
**Prior sweep reference:** `.factory/maintenance/doc-drift.md` (Sweep 1, 2026-06-17, develop @ e1273c8 / v0.7.1)

---

## Prior Sweep Status

All HIGH and MEDIUM findings from Sweep 1 (doc-drift.md) were resolved between v0.7.1 and v0.9.3:

| Prior ID | Prior Summary | Status |
|----------|--------------|--------|
| H-1 | ARP entirely absent from README | FIXED — ARP now fully documented |
| H-2 | "Multiple outputs" claimed only JSON export | FIXED — now reads "JSON export, CSV export" |
| H-3 | Roadmap listed "CSV and SQLite export" (CSV shipped) | FIXED — now reads "SQLite export" only |
| H-4 | ADR 0002 Existing Analyzers table 4 releases out of date | FIXED — table updated with all 6 analyzers and Deviations section |
| M-1 | README Architecture diagram/table omitted ARP, Modbus, DNP3, CSV | FIXED — diagram and table updated |
| M-2 | ADR 0005, 0006, 0007 referenced but files missing | FIXED — all three ADR files now exist |
| M-3 | lib.rs step 6 listed only DNS/HTTP/TLS | FIXED — now lists DNS/HTTP/TLS/Modbus/DNP3/ARP |
| M-4 | --hosts flag undocumented | FIXED — documented in README Usage section |
| M-5 | Reassembly threshold flags missing from README | FIXED — flags appear in Options block |
| M-6 | Stale "RED:" comments in test files | OPEN — not re-checked in this sweep (no source code scope) |
| L-1 | ADR 0001 StreamDispatcher struct snippet out of date | PARTIALLY — ADR updated with rule list but struct snippet still shows original 2-field form |
| L-2 | rayon dependency unused | OPEN — rayon = "1" still in Cargo.toml, no rayon:: usage found in src/ |
| L-3 | CHANGELOG v0.4.0 uses revoked T0855 technique ID | OPEN — not in scope for this sweep |
| L-4 | ADR 0003 consequence table references stale line ~349 in tls.rs | SUPERSEDED — ADR 0003 has grown significantly; new line number drift exists (see DOC-007 below) |

---

## Summary

| Severity | Count | Auto-Fixable |
|----------|-------|-------------|
| HIGH     | 2     | 1           |
| MED      | 4     | 2           |
| LOW      | 4     | 3           |
| **Total**| **10**| **6**       |

All findings are documentation-only. No runtime behavior is affected.

---

## Findings

| ID | File | Severity | Auto-Fixable | Issue |
|----|------|----------|-------------|-------|
| DOC-001 | `CLAUDE.md` | HIGH | Yes | STATE.md described as "not yet initialized" — file is fully populated |
| DOC-002 | `docs/adr/` | HIGH | No | ADR-009 referenced 40+ times in reader.rs but no docs/adr/0009-*.md file exists |
| DOC-003 | `README.md` | MED | Yes | Architecture component table says Reader parses "pcap files" — omits pcapng |
| DOC-004 | `README.md` | MED | Yes | Features bullet line 17 says "pcap formats" — pcapng not mentioned |
| DOC-005 | `docs/adr/0002-modular-protocol-analyzers.md` | MED | Yes | AnalysisSummary `detail` field described as `HashMap` — actual type is `BTreeMap` |
| DOC-006 | `docs/adr/0002-modular-protocol-analyzers.md` | MED | Yes | `StreamHandler::on_data` signature in ADR omits `timestamp: u32` parameter |
| DOC-007 | `docs/adr/0003-reporting-pipeline-layering.md` | LOW | Yes | main.rs line numbers in Grouped-Mode Collapse section are stale by ~60-100 lines |
| DOC-008 | `docs/adr/0002-modular-protocol-analyzers.md` | LOW | No | `parse_error_count()` listed as "Required" method in the table, but it is not part of either `ProtocolAnalyzer` or `StreamAnalyzer` traits — it is an inherent method convention only |
| DOC-009 | `docs/adr/0001-content-first-stream-dispatch.md` | LOW | Yes | StreamDispatcher struct code snippet still shows original 2-field (http, tls) form; modbus and dnp3 fields and their variants are missing from the snippet |
| DOC-010 | `Cargo.toml` | LOW | Yes | `rayon = "1"` declared but no `rayon::` usage exists in src/ (carried over from Sweep 1 L-2; still unresolved) |

---

## Detailed Findings

### DOC-001 — CLAUDE.md: STATE.md described as "not yet initialized" (HIGH, auto-fixable)

**File:** `CLAUDE.md`, Project References table, line 178

**Stale text:**
```
| `.factory/` | VSDD factory artifacts (logs only; STATE.md not yet initialized) |
```

**Reality:** `.factory/STATE.md` is fully populated (204 lines). It records the complete
pipeline state through v0.9.3, including phase completions from Phase 0 through F7, release
chain metadata, and the D-203 SAFE-TO-CLEAR checkpoint. The file has been populated since
at minimum Phase 0 (2026-05-19).

**Suggested fix:** Update the table row to:
```
| `.factory/` | VSDD factory artifacts (STATE.md, stories, specs, research, maintenance logs) |
```

---

### DOC-002 — docs/adr/: ADR-009 referenced in source but file does not exist (HIGH, not auto-fixable)

**File:** `src/reader.rs` (40 occurrences of `ADR-009`); `docs/adr/` contains only 0001–0007.

**Issue:** The pcapng reader module (`src/reader.rs`) was introduced in v0.9.3 and makes
extensive references to `ADR-009` throughout its constants, field doc comments, and inline
comments. Example occurrences:
- Line 32: `// ─── pcapng canonical constants (BC-2.01.009 / ADR-009) ────────────────────`
- Line 57: `/// SPB (Simple Packet Block) type code (BC-2.01.013 / ADR-009 Decision 22).`
- Line 86: `/// (BC-2.01.009 PC3 + EC-011 / ADR-009 Decision 27).`
- Line 159: `/// # Field constraints (BC-2.01.011 / ADR-009 rev 9)`

No `docs/adr/0008-*.md` or `docs/adr/0009-*.md` file exists. A contributor following any
`ADR-009` comment cannot find the decision record. This is the same class of gap that was
HIGH finding M-2 in Sweep 1 (ADRs 005–007 missing before they were written).

**Suggested fix:** Author `docs/adr/0009-pcapng-reader-design.md` capturing the pcapng
reader design decisions referenced by the 40+ inline citations. This requires authoring,
not a simple text change — hence not auto-fixable.

---

### DOC-003 — README.md: Architecture component table omits pcapng (MED, auto-fixable)

**File:** `README.md`, line 131

**Stale text:**
```
| Reader | `pcap-file` | Parse pcap files (5 link types) |
```

**Reality:** Since v0.9.3 the reader parses both classic pcap and pcapng files. The
`pcap-file` crate's `PcapNgParser` is used for pcapng (confirmed in `src/reader.rs`
lines 1009–1010). The phrase "pcap files" is now incomplete.

**Suggested fix:**
```
| Reader | `pcap-file` | Parse classic pcap and pcapng files (5 link types; both formats) |
```

---

### DOC-004 — README.md: Features bullet omits pcapng support (MED, auto-fixable)

**File:** `README.md`, line 17

**Stale text:**
```
- **Multi-link-type support** — Ethernet, Raw IP, IPv4, IPv6, and Linux Cooked (SLL) pcap formats
```

The phrase "pcap formats" implies classic pcap only. The "Supported Capture Formats" section
later in the README correctly describes both pcap and pcapng (lines 205–219), but the Features
bullet at line 17 is the first thing a reader sees and does not mention pcapng support at all.
v0.9.3 added pcapng as a first-class supported format.

**Suggested fix:**
```
- **Multi-link-type support** — Ethernet, Raw IP, IPv4, IPv6, and Linux Cooked (SLL) in both
  classic pcap and pcapng captures
```

---

### DOC-005 — ADR 0002: AnalysisSummary `detail` type is BTreeMap, not HashMap (MED, auto-fixable)

**File:** `docs/adr/0002-modular-protocol-analyzers.md`, lines 73 and 97

**Stale text (line 73, Required Methods table):**
```
| `summarize()` | Returns `AnalysisSummary` with `detail: HashMap<String, serde_json::Value>` | Yes |
```
**Stale text (line 97, code snippet):**
```rust
pub detail: HashMap<String, serde_json::Value>,
```

**Reality (`src/analyzer/mod.rs` line 52):**
```rust
pub detail: BTreeMap<String, serde_json::Value>,
```
The type was changed from `HashMap` to `BTreeMap` per LESSON-P2.09 / NFR DET-001 to ensure
deterministic JSON serialization. The ADR was not updated when this change was made.

**Suggested fix:** Replace both `HashMap` occurrences with `BTreeMap` in the ADR. Also update
the imports note to reference `std::collections::BTreeMap`.

---

### DOC-006 — ADR 0002: StreamHandler::on_data signature omits `timestamp: u32` parameter (MED, auto-fixable)

**File:** `docs/adr/0002-modular-protocol-analyzers.md`, line 28

**Stale text:**
```rust
fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], offset: u64);
```

**Reality (`src/reassembly/handler.rs`):**
```rust
fn on_data(
    &mut self,
    flow_key: &FlowKey,
    direction: Direction,
    data: &[u8],
    offset: u64,
    timestamp: u32,
);
```
A `timestamp: u32` parameter was added to `StreamHandler::on_data` (used by the DNP3 analyzer
for its windowed detection logic). The ADR code snippet does not reflect this addition.

**Suggested fix:** Add `timestamp: u32,` as the final parameter to the `on_data` signature in
the ADR code snippet.

---

### DOC-007 — ADR 0003: main.rs line numbers in Grouped-Mode Collapse section are stale (LOW, auto-fixable)

**File:** `docs/adr/0003-reporting-pipeline-layering.md`, Grouped-Mode Collapse subsection

**Issue:** The ADR contains multiple specific `src/main.rs` line number references that are now
stale because main.rs has grown from its v0.9.0 form (745 lines currently) since the subsection
was written:

| ADR says | Actual location |
|---------|----------------|
| `src/main.rs lines 79-80` (collapse/mitre setup) | now ~line 128 |
| `src/main.rs lines 107-108` (run_analyze signature) | now `fn run_analyze` at line 142 |
| `src/main.rs ~line 373` (TerminalReporter construction) | now ~line 438 |
| `src/main.rs:383-390` (FindingsRender construction block) | now ~lines 448–452 |
| `src/main.rs:451-454` (run_summary inert site) | `fn run_summary` now at line 472 |
| `src/main.rs:511` (grouping_from_flag) | `grouping_from_flag` now at line 610 |
| `src/main.rs:502` (collapse_findings_from_flag) | now at line 601 |

The structural descriptions and behavioral claims in the ADR are accurate; only the line
numbers are stale. The ADR's prose can be corrected by replacing the specific line numbers
with the function names they label (e.g., "inside `run_analyze`" rather than "line 373").

**Suggested fix:** Replace pinned line numbers with function-name anchors throughout the
Grouped-Mode Collapse and Render-Mode Enum subsections. Example: replace
"`src/main.rs:383-390`" with "the `TerminalReporter` construction site in `run_analyze`".

---

### DOC-008 — ADR 0002: parse_error_count() listed as Required trait method, but is not in any trait (LOW, not auto-fixable)

**File:** `docs/adr/0002-modular-protocol-analyzers.md`, lines 74–76 (Required Methods table)

**Stale text:**
```
| `parse_error_count()` | Returns `u64` | Yes |
```

**Reality:** Neither `ProtocolAnalyzer` (in `src/analyzer/mod.rs`) nor `StreamAnalyzer`
(in `src/reassembly/handler.rs`) includes `parse_error_count()`. It exists only as an
inherent method on specific analyzer structs (`HttpAnalyzer`, `TlsAnalyzer`) for test
access. The "Required" designation in the ADR is incorrect — it is a convention, not a
trait requirement.

A contributor implementing a new analyzer who reads the ADR's Required Methods table would
expect to implement `parse_error_count()` as part of the trait contract and would then get
no compiler error for omitting it, which could produce confusion.

**Suggested fix:** Rename the section to "Conventional Methods and Accessors" (or similar)
and change "Yes" in the Required column to "Convention" for `parse_error_count()`. Note that
it is an inherent method for testing, not a trait obligation.

---

### DOC-009 — ADR 0001: StreamDispatcher struct code snippet is missing modbus/dnp3 fields (LOW, auto-fixable)

**File:** `docs/adr/0001-content-first-stream-dispatch.md`, lines 28–48

**Issue:** The struct and enum code snippet shows only `http` and `tls` fields, and only
`Http`, `Tls`, and `None` variants in `DispatchTarget`. This was flagged as LOW in Sweep 1
(L-1) and has not been updated. The actual struct has `modbus` and `dnp3` optional fields,
and the `DispatchTarget` enum has `Modbus` and `Dnp3` variants. The rule list earlier in the
ADR (lines 50–58) was updated to show all 7 rules, but the struct snippet was not.

This gap is mild because ADR-005 and ADR-007 document the extensions, but a reader of ADR-001
alone still sees a structurally incomplete picture.

**Suggested fix:** Add an amendment note below the struct snippet: "As of v0.4.0 (ADR-005)
and v0.6.0 (ADR-007), the struct also carries `modbus: Option<ModbusAnalyzer>` and
`dnp3: Option<Dnp3Analyzer>`, and `DispatchTarget` has `Modbus` and `Dnp3` variants.
See ADR-005 §Consequences and ADR-007 §Consequences for the current shape."

---

### DOC-010 — Cargo.toml: rayon dependency declared but unused (LOW, auto-fixable)

**File:** `Cargo.toml`, line 37

**Text:** `rayon = "1"`

**Issue:** No `use rayon` or `rayon::` call exists anywhere in `src/`. This was flagged as
L-2 in Sweep 1 and remains unresolved. The "Parallel file processing" Roadmap item suggests
rayon was added in anticipation; it has not been implemented.

An unused dependency adds compile-time cost and introduces a supply-chain surface that could
acquire vulnerabilities before the dependency is actually used.

**Suggested fix:** Remove `rayon = "1"` from `Cargo.toml`. Re-add when parallel processing
is implemented.

---

## Items Confirmed Accurate

The following were checked and found to be correct at HEAD dd3b069:

- All 7 ADR files (0001–0007) exist and the CLAUDE.md Project References table lists them correctly.
- `bin/compute-input-hash`, `bin/test_compute_input_hash.py`, and `.github/workflows/ci.yml` all exist.
- `docs/superpowers/plans/` and `docs/superpowers/specs/` both exist with content.
- `src/analyzer/` contains: arp.rs, dnp3.rs, dns.rs, http.rs, mod.rs, modbus.rs, tls.rs — matching the 6 analyzers documented in README and ADR 0002.
- `src/reassembly/` module structure is intact: config.rs, flow.rs, handler.rs, lifecycle.rs, mod.rs, segment.rs, stats.rs.
- `src/reporter/` contains csv.rs, json.rs, mod.rs, terminal.rs — matching README output claims.
- README Supported Capture Formats section accurately describes pcapng block support (SHB, IDB, EPB, SPB parsed; NRB, ISB, SJE, DSB, OPB skipped), 4 GiB per-file cap, and multi-section limitation.
- README CLI usage examples (analyze, summary subcommands, all flags) match current implementation.
- README Features section ARP, DNP3, Modbus, HTTP, TLS, DNS bullets are accurate.
- No TODO/FIXME/HACK comments found anywhere in `src/`.
- No `todo!()` or `unimplemented!()` macro calls found in `src/`.
- ADR 0004 atomics pattern (3 guards + test seams) is documented accurately and matches code.
- ADR 0005 port-502/port-20000 rule order (Rules 5 and 6) matches `src/dispatcher.rs` struct.
- ADR 0006 mitre_techniques Vec<String> / CSV semicolon join / JSON skip_serializing_if matches code.
- ADR 0007 DNP3 frame constants (MAX_DNP3_FRAME_LEN=292, CORRELATION_WINDOW_SECS=300, etc.) match code.
- StreamDispatcher actual struct fields (http, tls, modbus, dnp3, routes, classification_attempts, max_classification_attempts, unclassified_flows) match ADR 0001 extended description.
- ProtocolAnalyzer and StreamAnalyzer trait method signatures are accurate in README Extending section.
- FindingsRender struct-of-enums (Grouping × Collapse) exists in terminal.rs as documented in ADR 0003.
- `--no-collapse` and `--mitre` flags behave as documented in README and ADR 0003.
- CLAUDE.md rust-version 1.91 / Rust 2024 edition / single-crate notes remain accurate.
- CLAUDE.md git workflow, branch naming, and semantic PR type list are accurate.
- CLAUDE.md action-pin-gate and dtolnay/rust-toolchain exemption notes are accurate.
- CLAUDE.md Input Hash Algorithm section accurately describes bin/compute-input-hash.
