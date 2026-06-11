---
document_type: story
story_id: "STORY-079"
epic_id: "E-8"
version: "1.5"
status: completed
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.020.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.021.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.022.md
  - .factory/specs/prd.md
input-hash: "1d0e3b9"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-076]
blocks: [STORY-080]
behavioral_contracts:
  - BC-2.11.020
  - BC-2.11.021
  - BC-2.11.022
verification_properties: [VP-020]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 21
target_module: reporter/csv
subsystems: [SS-11]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
nfr:
  - NFR-COMPAT-002
implementation_strategy: brownfield-formalization
---

# STORY-079: CsvReporter — Fixed 9-Column Schema, CSV-Injection Neutralization, and Evidence Join

## Narrative
- **As a** security analyst or SIEM integration engineer importing wirerust findings into a spreadsheet or pipeline
- **I want** CsvReporter to produce a fixed 9-column RFC 4180 CSV with a stable header, neutralize formula-injection trigger characters by prepending a single-quote, and flatten multi-evidence findings into a single semicolon-separated cell
- **So that** I can rely on column positions without schema discovery, trust that no attacker-supplied content can execute as a spreadsheet formula, and import all evidence lines without losing any

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.11.020 | CsvReporter Emits Exactly Nine Columns in Fixed Header Order |
| BC-2.11.021 | CsvReporter Neutralizes CSV-Injection Trigger Characters with a Leading Single Quote |
| BC-2.11.022 | CsvReporter Joins Evidence Vec Elements with "; " into a Single Cell |

## Acceptance Criteria

### AC-001 (traces to BC-2.11.020 postcondition 1)
The first line of `CsvReporter::render` output is the header row:
`category,verdict,confidence,summary,evidence,mitre_technique,source_ip,direction,timestamp`
(with LF `\n` line terminator — the `csv` crate default; RFC 4180 CRLF is NOT configured).
- **Test:** `test_BC_2_11_020_header_row_first_and_exact()`

### AC-002 (traces to BC-2.11.020 postcondition 2)
Every data row in the output contains exactly 9 comma-separated fields, in the same column order as the header.
- **Test:** `test_BC_2_11_020_every_row_has_nine_columns()`

### AC-003 (traces to BC-2.11.020 invariant 1)
Column count is exactly 9 in every row including when a field value contains a comma (the `csv` crate wraps the field in double-quotes per RFC 4180, but the column count remains 9).
- **Test:** `test_BC_2_11_020_comma_in_field_does_not_change_column_count()`

### AC-004 (traces to BC-2.11.020 postcondition 4)
The output is valid RFC 4180 CSV: a field containing commas appears double-quoted; a field containing double-quotes uses `""` escaping per RFC 4180.
- **Test:** `test_BC_2_11_020_rfc4180_quoting()`

### AC-005 (traces to BC-2.11.021 postcondition 1)
`neutralize_csv_injection` prepends `'` to any cell value whose first character is `=`, `+`, `-`, `@`, TAB (U+0009), or CR (U+000D).
- **Test:** `test_BC_2_11_021_neutralize_all_six_trigger_chars()`

### AC-006 (traces to BC-2.11.021 postcondition 2)
`neutralize_csv_injection` returns the input unchanged when the first character is any character NOT in the trigger set (including printable ASCII letters, digits, and non-trigger punctuation).
- **Test:** `test_BC_2_11_021_no_trigger_no_change()`

### AC-007 (traces to BC-2.11.021 postcondition 4)
An empty string input to `neutralize_csv_injection` is returned unchanged (no prefix added).
- **Test:** `test_BC_2_11_021_empty_string_unchanged()`

### AC-008 (traces to BC-2.11.021 invariant 2)
Only the FIRST character is inspected; a trigger character at position 2+ does NOT cause a prefix to be added (e.g., `"a=formula"` is returned unchanged).
- **Test:** `test_BC_2_11_021_trigger_at_position_2_no_prefix()`

### AC-009 (traces to BC-2.11.021 invariant 1)
`neutralize_csv_injection` is applied to ALL nine column values for every data row without exception.
- **Test:** `test_BC_2_11_021_applied_to_all_nine_columns()`

### AC-010 (traces to BC-2.11.022 postcondition 1)
For a Finding with `evidence = ["first", "second", "third"]`, the evidence CSV cell value is `"first; second; third"` (elements joined with `"; "` — semicolon then space).
- **Test:** `test_BC_2_11_022_evidence_joined_with_semicolon_space()`

### AC-011 (traces to BC-2.11.022 postcondition 2)
For a Finding with `evidence = []` (empty), the evidence cell is an empty string `""`.
- **Test:** `test_BC_2_11_022_empty_evidence_is_empty_cell()`

### AC-012 (traces to BC-2.11.022 postcondition 3)
For a Finding with `evidence = ["single item"]`, the evidence cell is `"single item"` with no separator.
- **Test:** `test_BC_2_11_022_single_evidence_no_separator()`

### AC-013 (traces to BC-2.11.022 postcondition 4)
The joined evidence string is subsequently processed by `neutralize_csv_injection`. If the first element starts with `=`, the entire joined string (starting with `=`) receives a `'` prefix.
- **Test:** `test_BC_2_11_022_evidence_join_then_neutralize()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| CsvReporter | src/reporter/csv.rs | pure |
| neutralize_csv_injection | src/reporter/csv.rs:40-45 | pure |
| CsvReporter::render (header + loop) | src/reporter/csv.rs:51-106 | pure |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | findings is empty | Header row only; valid 1-line CSV |
| EC-002 | Field value contains comma | csv crate wraps in double-quotes; column count remains 9 |
| EC-003 | Field value contains double-quote | csv crate escapes as `""`; RFC 4180 |
| EC-004 | Cell starts with `=` | `'=...` prefix added |
| EC-005 | Cell starts with `+` | `'+...` prefix |
| EC-006 | Cell starts with TAB | `'\t...` prefix (literal tab still present) |
| EC-007 | Cell starts with `'` | No change; single-quote is not a trigger |
| EC-008 | evidence = [] | Empty cell (no separator) |
| EC-009 | evidence = ["a", "b"] | `"a; b"` |
| EC-010 | First evidence starts with `=` | Joined string starts with `=`; neutralized to `'=...` |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reporter/csv.rs | pure | In-memory Vec<u8> buffer; returns owned String; no I/O |
| neutralize_csv_injection | pure | Pure string transformation; no side effects |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| src/reporter/csv.rs (full file) | ~2,000 |
| BC files (3 BCs) | ~4,500 |
| tests/reporter_csv_tests.rs (CSV tests) | ~1,000 |
| Tool outputs overhead | ~400 |
| **Total** | **~10,400** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~5.2%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-013 (test-writer)
2. [ ] Verify all tests fail at Red Gate
3. [ ] Verify `src/reporter/csv.rs` already satisfies all ACs (brownfield confirm)
4. [ ] Confirm header row at csv.rs:62-73 lists exactly the 9 column names in order
5. [ ] Confirm `neutralize_csv_injection` at csv.rs:40-45 matches all 6 trigger chars
6. [ ] Confirm `neutralize_csv_injection` applied to all 9 columns at csv.rs:89-97
7. [ ] Confirm evidence join is `f.evidence.join("; ")` at csv.rs:81
8. [ ] Verify via unit tests that non-trigger-prefixed inputs pass through unchanged (`test_BC_2_11_021_no_trigger_no_change` + `test_BC_2_11_021_trigger_at_position_2_no_prefix`)
9. [ ] Run `cargo test --all-targets` to confirm green

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-076 | JsonReporter is the model for the `Reporter` trait; summary+findings+analyzers | Reporter trait signature: render(summary, findings, analyzer_summaries) -> String | CsvReporter intentionally ignores summary and analyzer_summaries |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Column count is ALWAYS exactly 9; no row may have 8 or 10 columns | BC-2.11.020 invariant 1 | Unit test: parse every row with the csv crate, assert column count == 9 (`test_BC_2_11_020_every_row_has_nine_columns`) |
| `neutralize_csv_injection` is called on ALL 9 column values per row — no bypass | BC-2.11.021 invariant 1 | Code review: csv.rs:89-97 calls it on each field individually |
| Evidence join separator is exactly `"; "` (semicolon + space, two characters) | BC-2.11.022 invariant 1 | Code review: csv.rs:81 join literal |
| `csv::WriterBuilder` default separator is a comma; no alternative separator configured | BC-2.11.020 invariant 3 | Code review: csv.rs:58 WriterBuilder initialization |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| csv | (per Cargo.lock) | `WriterBuilder::new().from_writer(Vec::new())` — RFC 4180 CSV field formatting and quoting; LF line terminators (not CRLF — no `.terminator(Terminator::CRLF)` configured) |
| serde | (per Cargo.lock) | If used for record serialization |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/reporter/csv.rs | verify/modify | neutralize_csv_injection (40-45), header (62-73), per-row render (76-100) |
| tests/reporter_csv_tests.rs | create or modify | AC-001 through AC-013 tests |

## Revision History

| Version | Date | Change |
|---------|------|--------|
| v1.3 | 2026-05-30 | proptest→unit in Task 8 + Compliance Rules enforcement to match VP-020 proof_method:unit + realized unit test suite (STORY-079 P6 finding; sibling-sweep of 2026-05-30 VP-020 correction) |
| v1.5 | 2026-06-09 | story-writer | UPDATED (Feature #7 migration note): STORY-079 covers `CsvReporter` fixed 9-column schema. STORY-101 (v0.3.0) renames column 6 header from `mitre_technique` to `mitre_techniques` and changes the value encoding from `Option::as_deref().unwrap_or("")` to `Vec::join(";")`. CSV output is behavior-preserving for singleton vecs (single technique renders identically). Multi-technique vecs render as `"T1692.001;T0836"` (semicolon-joined). The VP-020 csv-injection-neutralization harness is updated by STORY-100 to use `Finding { mitre_techniques: vec![...] }`. Story status remains `completed`; no re-implementation required. |
