---
document_type: maintenance-sweep-output
sweep: pattern-consistency
sweep_id: maint-2026-07-09 / Sweep-3
producer: code-reviewer
develop_head: 716054a
date: 2026-07-09
---

# Pattern Consistency Scan — maint-2026-07-09

Run: **maint-2026-07-09**, Sweep 3.
Scope: `src/` + `tests/`, codebase at `716054a` (develop).
Delta: `c4eb1f4..716054a` (wave-72 PRs #387–#391 + dep bump #386).
Clippy result: **CLEAN** (`cargo clippy --all-targets -- -D warnings`, 0 warnings).
Fmt result: **CLEAN** (`cargo fmt --check`, 0 diffs).

---

## Prior Finding Dispositions (PF-001 through PF-008)

| ID | Description | Severity | Status |
|----|-------------|----------|--------|
| PF-001 | Counter discipline: plain `+=` on diagnostic counters | MEDIUM | **RESOLVED** (PR #384, c4eb1f4) |
| PF-002 | dnp3.rs free-function naming: 4 functions lack `dnp3_` prefix | LOW | **STILL-OPEN** |
| PF-003 | enip.rs `check_t0814` lacks `enip_` module prefix | LOW | **STILL-OPEN** |
| PF-004 | Trait gap: `Dnp3Analyzer` and `EnipAnalyzer` do not implement `StreamHandler`/`StreamAnalyzer` | LOW | **STILL-OPEN** |
| PF-005 | Error handling style: all analyzers consistent — no finding | — | **INFO (CLEAN)** |
| PF-006 | Clippy gate | — | **INFO (CLEAN)** |
| PF-007 | PG-HELP-PROVENANCE-CLI-DOC-001 | — | **INFO (CLEAN)** |
| PF-008 | Wave-71 CR-001 (MINOR + 3 NITs) | LOW | **INFO (no regression)** |

---

## PF-001 Resolution Evidence

PR #384 (`c4eb1f4 refactor: PF-001 — convert 109 diagnostic-counter sites to saturating_add`)
resolved the entire PF-001 scope. Post-conversion verification at `716054a`:

**`src/dispatcher.rs:477`** (the prior lone outlier): now `self.unclassified_flows = self.unclassified_flows.saturating_add(1)`.

**`src/main.rs`**: Lines 443, 452, 838 — all three sites (`malformed_frames`, `total_decode_errors` ×2) now use `saturating_add`.

**`src/analyzer/dns.rs`** (prior 2 sites), **`src/analyzer/arp.rs`** (prior 5 sites), **`src/analyzer/tls.rs`** (prior 6 sites), **`src/analyzer/enip.rs`** (prior 4 sites), **`src/analyzer/dnp3.rs`** (prior 25 sites), **`src/reassembly/lifecycle.rs`** (prior 3 sites): all converted.

Remaining `+=` sites in `src/` are exclusively in three excluded categories:
- **HashMap entry value increments** (`*flow.command_counts.entry(header.command).or_insert(0) += 1`) — excluded by PF-001 scope definition
- **Loop cursor / index variables** (`cursor += 1`, `i += 1`, `j += 1`, `dispatched += 1`, etc.)
- **Local computation accumulators** (`storm_count += 1`, `len += 1`, `count += 1` in bounded local scopes)

**PF-001: FULLY RESOLVED. No regression. Wave-72 introduced zero new diagnostic-counter `+=` sites.**

---

## Wave-72 Delta Scan (c4eb1f4..716054a)

### Files changed in src/

| File | PR | Change |
|------|----|--------|
| `src/findings.rs` | #389 | Added `#[serde(rename_all = "lowercase")]` to `Verdict` and `Confidence`; `#[serde(rename_all = "snake_case")]` to `ThreatCategory` |
| `src/reporter/json.rs` | #389 | Added `SCHEMA_VERSION: &str = "2"` constant and `"schema_version"` envelope field |
| `src/reporter/terminal.rs` | #389 | Two `for (_, items) in buckets.iter_mut()` → `for items in buckets.values_mut()` (clippy cleanup) |

No changes to `src/analyzer/`, `src/dispatcher.rs`, `src/reassembly/`, `src/decoder.rs`, `src/reader.rs`, or `src/mitre.rs`.

---

### Scan Category: New plain `+=` diagnostic-counter sites

**Result: CLEAN.** No new `+=` sites on named diagnostic counter fields introduced by wave-72.
The only `+=` additions in src/ are the `buckets.values_mut()` refactors in `terminal.rs`, which contain
no counter fields.

---

### Scan Category: New `as` numeric casts in src/

**Result: CLEAN.** Zero new `as` numeric casts introduced by wave-72 src/ changes. The existing casts
in `src/reader.rs` (timestamp arithmetic) and `src/reporter/terminal.rs` (unicode codepoint rendering)
are all pre-wave-72 and unmodified.

---

### Scan Category: `self.flows[` index syntax (SEC-011 anti-gameability convention)

**Result: CLEAN (no actual code usage).** Three grep hits in `src/analyzer/enip.rs:980,983,986`
are all inside `//` inline comments documenting the split-borrow safety argument — e.g.,
`// flow we pass is from self.flows[flow_key], and process_pdu only mutates`. The actual code
at lines 992–994 uses `.get_mut(&flow_key)` (the preferred keyed-lookup pattern). No `self.flows[index]`
indexing in production code.

---

### Scan Category: Error-handling pattern divergence

**Result: CLEAN.** Wave-72 src/ changes introduce no new `unwrap()`, `expect()`, or `panic!()` in
production paths. The `serde_json::to_string_pretty(...).unwrap()` in `src/reporter/json.rs` is
pre-wave-72 and unmodified (infallible in practice — memory-only serialization). The new
`SCHEMA_VERSION` constant is a `&str` literal with no fallible path.

---

### Scan Category: Naming convention drift

**Result: CLEAN.** The new `SCHEMA_VERSION` constant follows the existing ALL_CAPS `&str` constant
naming pattern established by `MITRE_DOMAIN` and `MITRE_ATTACK_VERSION` in the same file.
The `#[serde(rename_all = ...)]` attribute ordering (after `#[derive(..)]`, before `#[non_exhaustive]`)
is consistent across all three enum additions in `findings.rs`.

---

### Scan Category: Test structure inconsistency

**Result: CLEAN.** The 357 new lines added to `tests/reporter_json_tests.rs` follow house style:
- `test_BC_2_11_036_*` and `test_BC_2_11_037_*` naming matches the `PG-W17-001` convention
- Section headers as `// ─── comment ───` separating contract groups
- Per-variant iteration with clear per-assertion messages
- `#![allow(non_snake_case)]` already present at file scope (pre-wave-72); new tests inherit it

The `bc_2_09_100_multitag_tests.rs` update correctly changes the six-key sort test to include `schema_version`
and updates the assertion message to the v1.9 BC version. No structure drift.

---

### Stale-assertion check (BC-2.11.036 casing change)

Wave-72 changes `Verdict` and `Confidence` to serialize lowercase and `ThreatCategory` to serialize
snake_case. Checked all test files for assertions on JSON enum values:

- `tests/enip_e2e_real_pcaps_tests.rs:204,336,474`: Uses `format!("{:?}", f.category)` (Debug repr →
  PascalCase) — unaffected by `#[serde(rename_all)]` which only controls `Serialize`. No staleness.
- `tests/reporter_csv_tests.rs:240`: Asserts CSV row contains `"Anomaly"` — CSV reporter uses
  `f.category.to_string()` (Display → Debug → PascalCase). Unaffected. No staleness.
- `tests/bc_2_16_story115_arp_tests.rs:291-296`: Calls `.to_lowercase()` on the JSON `confidence`
  field before comparing to `"medium"`. The comparison is deliberately case-insensitive; a comment
  at line 307 explicitly documents this design ("JSON casing is a serialisation detail not specified
  by BC-2.16.008 AC-015"). No staleness.
- `tests/reporter_json_tests.rs`: No remaining assertions on PascalCase serialized enum values;
  the new BC-2.11.036 tests assert the lowercase/snake_case forms exclusively.

**No stale test assertions found across any test file.**

---

## Summary Table

| ID | Description | Severity | Status |
|----|-------------|----------|--------|
| PF-001 | Counter discipline: `+=` on diagnostic counters (~48 sites) | MEDIUM | **RESOLVED** PR #384 |
| PF-002 | dnp3.rs free-function naming: 4 functions lack `dnp3_` prefix | LOW | **STILL-OPEN** |
| PF-003 | enip.rs `check_t0814` lacks `enip_` module prefix | LOW | **STILL-OPEN** |
| PF-004 | Trait gap: `Dnp3Analyzer`/`EnipAnalyzer` missing `StreamHandler`/`StreamAnalyzer` | LOW | **STILL-OPEN** |
| PF-005 | Error handling style | — | **INFO (CLEAN)** |
| PF-006 | Clippy gate | — | **INFO (CLEAN)** |
| PF-007 | PG-HELP-PROVENANCE-CLI-DOC-001 | — | **INFO (CLEAN)** |
| PF-008 | Wave-71 CR-001 gate-artifact gap | LOW | **INFO (no regression)** |

**No new PF-NNN findings from wave-72 delta.** Wave-72 is consistent with all house conventions.

---

## Counts

| Category | Count |
|----------|-------|
| RESOLVED this sweep | 1 (PF-001) |
| STILL-OPEN (carry forward) | 3 (PF-002, PF-003, PF-004) |
| INFO / no-action | 4 (PF-005 through PF-008) |
| NEW findings from wave-72 | 0 |
| Clippy | CLEAN |
| fmt | CLEAN |

---

## Carry-Forward Open Items (for next sweep)

### PF-002 — dnp3.rs Free-Function Naming [STILL-OPEN]

**Severity:** LOW
**Category:** naming-convention
**Files:** `src/analyzer/dnp3.rs:2053,2066,2086,2103`

Four public free functions lack the `dnp3_` module prefix used by the file's other four (`parse_dnp3_dl_header`,
`is_valid_dnp3_frame_header`, `classify_dnp3_fc`, `compute_dnp3_frame_len`):

| Line | Current name | Target name |
|------|-------------|-------------|
| 2053 | `transport_is_fir` | `dnp3_transport_is_fir` |
| 2066 | `has_user_data` | `dnp3_has_user_data` |
| 2086 | `is_broadcast_destination` | `dnp3_is_broadcast_destination` |
| 2103 | `is_master_frame` | `dnp3_is_master_frame` |

**Classification:** FIXABLE-AUTO — mechanical rename + call-site update in the same file and tests.

---

### PF-003 — enip.rs `check_t0814` Naming [STILL-OPEN]

**Severity:** LOW
**Category:** naming-convention
**Files:** `src/analyzer/enip.rs:447`

`pub fn check_t0814(...)` lacks the `enip_` prefix used by peer functions
(`parse_enip_header`, `classify_enip_command`, `is_valid_enip_frame`).

**Classification:** MANUAL — the T0814 threat-tag suffix is domain-specific; evaluate
`enip_check_t0814` vs. `check_enip_t0814` before renaming.

---

### PF-004 — Trait Implementation Gap: EnipAnalyzer, Dnp3Analyzer [STILL-OPEN]

**Severity:** LOW
**Category:** architecture-alignment
**Files:** `src/analyzer/dnp3.rs`, `src/analyzer/enip.rs`, `src/dispatcher.rs`

`Dnp3Analyzer` and `EnipAnalyzer` expose `on_data`/`on_flow_close`/`summarize` as bare `impl` methods
rather than `StreamHandler`/`StreamAnalyzer` trait implementations. The dispatcher holds them as
concrete `Option<T>` fields. This is an intentional structural choice per ADR-007 and ADR-010.

**Classification:** ARCH-REVIEW — no code change required now; flag for next ADR revision if a third
concrete ICS analyzer is added.
