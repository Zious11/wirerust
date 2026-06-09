---
document_type: story
story_id: "STORY-080"
epic_id: "E-8"
version: "1.4"
status: completed
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.023.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.024.md
  - .factory/specs/prd.md
input-hash: "a53c09c"
traces_to: .factory/specs/prd.md
points: 3
depends_on: [STORY-079]
blocks: [STORY-086]
behavioral_contracts:
  - BC-2.11.023
  - BC-2.11.024
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 22
target_module: reporter/csv
subsystems: [SS-11]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
nfr:
  - NFR-COMPAT-002
implementation_strategy: brownfield-formalization
---

# STORY-080: CsvReporter — Reporter Trait Compliance and Optional Field Encoding

## Narrative
- **As a** security toolchain integrator
- **I want** CsvReporter to implement the Reporter trait producing exactly one data row per Finding with None optional fields encoded as empty strings, Direction as CamelCase Debug variant names, and timestamps as RFC 3339 strings — while completely ignoring Summary and AnalysisSummary inputs
- **So that** my downstream parser always sees a predictable row count and can detect absent optional data by empty cell (not sentinel strings like "null" or "N/A")

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.11.023 | CsvReporter Implements Reporter Trait and Emits One Row per Finding; Summary and AnalysisSummary Are Ignored |
| BC-2.11.024 | CsvReporter Encodes None Optional Fields as Empty Strings and Direction as Debug Variant Name |

## Acceptance Criteria

### AC-001 (traces to BC-2.11.023 postcondition 1)
The total row count in `CsvReporter::render` output is exactly `1 + findings.len()`: one header row plus one data row per finding.
- **Test:** `test_BC_2_11_023_row_count_equals_one_plus_findings_len()`

### AC-002 (traces to BC-2.11.023 postcondition 2)
The `summary` parameter is NOT reflected anywhere in the CSV output. A Summary with `total_packets = 9999` produces no row, column, or header entry related to 9999.
- **Test:** `test_BC_2_11_023_summary_not_in_output()`

### AC-003 (traces to BC-2.11.023 postcondition 3)
The `analyzer_summaries` parameter is NOT reflected anywhere in the CSV output. Non-empty analyzer_summaries produce no additional rows or columns.
- **Test:** `test_BC_2_11_023_analyzer_summaries_not_in_output()`

### AC-004 (traces to BC-2.11.023 postcondition 5)
Row order in the output matches the iteration order of the findings slice: first Finding maps to data row 2 (after header), second to data row 3, etc. No sorting or deduplication occurs.
- **Test:** `test_BC_2_11_023_row_order_matches_findings_slice()`

### AC-005 (traces to BC-2.11.023 invariant 1)
For an empty findings slice, the output is exactly the header row (valid 1-line CSV with no data rows).
- **Test:** `test_BC_2_11_023_empty_findings_header_only()`

### AC-006 (traces to BC-2.11.024 postcondition 1)
When `Finding.mitre_technique = None`, column 6 (`mitre_technique`) is an empty string `""`, not `"null"`, `"N/A"`, or any other sentinel.
- **Test:** `test_BC_2_11_024_none_mitre_technique_is_empty()`

### AC-007 (traces to BC-2.11.024 postcondition 2)
When `Finding.source_ip = None`, column 7 (`source_ip`) is an empty string. When `Some(192.168.1.1)`, it is `"192.168.1.1"`. When `Some(::1)`, it is `"::1"` (IpAddr::to_string compact form).
- **Test:** `test_BC_2_11_024_source_ip_encoding()`

### AC-008 (traces to BC-2.11.024 postcondition 3)
When `Finding.direction = Some(ClientToServer)`, column 8 is `"ClientToServer"`. When `Some(ServerToClient)`, column 8 is `"ServerToClient"`. The format is Debug (`{:?}`) — CamelCase, not lowercase.
- **Test:** `test_BC_2_11_024_direction_debug_camelcase()`

### AC-009 (traces to BC-2.11.024 postcondition 3)
When `Finding.direction = None`, column 8 is an empty string `""`.
- **Test:** `test_BC_2_11_024_none_direction_is_empty()`

### AC-010 (traces to BC-2.11.024 postcondition 4)
When `Finding.timestamp = None`, column 9 is an empty string `""`. When `Some(datetime)`, column 9 is the RFC 3339 string from `to_rfc3339()` (e.g., `"2024-01-15T12:34:56+00:00"`).
- **Test:** `test_BC_2_11_024_timestamp_rfc3339_encoding()`

### AC-011 (traces to BC-2.11.024 invariant 4)
None-to-empty-string encoding uses `unwrap_or("")` or `unwrap_or_default()`. No sentinel values `"null"`, `"N/A"`, or `"-"` appear for any None optional field.
- **Test:** `test_BC_2_11_024_no_sentinel_values_for_none()`

### AC-012 (traces to BC-2.11.024 postcondition 5)
All four optional-field-derived strings are individually passed through `neutralize_csv_injection` before the csv write. A `mitre_technique = Some("=HYPERLINK(...)")` produces `"'=HYPERLINK(...)"` in column 6.
- **Test:** `test_BC_2_11_024_optional_fields_neutralized()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| CsvReporter (impl Reporter) | src/reporter/csv.rs:51-106 | pure |
| Optional field encoding | src/reporter/csv.rs:82-85 | pure |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | findings = [] | Header row only; 0 data rows |
| EC-002 | findings = [f1, f2, f3] | Header + 3 data rows |
| EC-003 | All four Option fields = None | Columns 6-9 are all empty strings |
| EC-004 | direction = Some(ClientToServer) | Column 8 = `"ClientToServer"` (CamelCase) |
| EC-005 | source_ip = Some(2001:db8::1) | Column 7 = `"2001:db8::1"` |
| EC-006 | timestamp = Some(t) | Column 9 = RFC 3339 string |
| EC-007 | mitre_technique = Some("=cmd") | Column 6 = `"'=cmd"` (neutralized) |
| EC-008 | analyzer_summaries non-empty | Not in output; silently ignored |
| EC-009 | Duplicate findings in slice | Both emitted as separate rows; no deduplication |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reporter/csv.rs | pure | In-memory Vec<u8> buffer; `_summary` and `_analyzer_summaries` intentionally unused |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,000 |
| src/reporter/csv.rs (optional field section, csv.rs:82-97) | ~1,500 |
| BC files (2 BCs) | ~3,000 |
| tests/reporter_csv_tests.rs | ~800 |
| Tool outputs overhead | ~300 |
| **Total** | **~7,600** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~3.8%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-012 (test-writer)
2. [ ] Verify all tests fail at Red Gate
3. [ ] Verify `src/reporter/csv.rs` already satisfies all ACs (brownfield confirm)
4. [ ] Confirm `_summary` and `_analyzer_summaries` underscore-prefixed at csv.rs:53-56
5. [ ] Confirm `f.mitre_technique.as_deref().unwrap_or("")` at csv.rs:82
6. [ ] Confirm `f.source_ip.map(|ip| ip.to_string()).unwrap_or_default()` at csv.rs:83
7. [ ] Confirm `f.direction.map(|d| format!("{d:?}")).unwrap_or_default()` at csv.rs:84
8. [ ] Confirm `f.timestamp.map(|t| t.to_rfc3339()).unwrap_or_default()` at csv.rs:85
9. [ ] Confirm neutralize called on all four at csv.rs:94-97
10. [ ] Run `cargo test --all-targets` to confirm green

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-079 | CsvReporter schema (9 columns), neutralize_csv_injection, evidence join | CSV module pattern: in-memory Vec<u8> buffer via csv::WriterBuilder | Direction uses Debug format (`{:?}`), NOT Display — `Direction` has no Display impl; using Display would fail to compile |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `_summary` and `_analyzer_summaries` are intentionally unused (underscore prefix); no summary or analyzer data appears in output | BC-2.11.023 invariant 2 | Compiler: underscore prefix suppresses unused-variable warning; grep output for "total_packets" etc. must return empty |
| `Direction` is encoded via `format!("{d:?}")` (Debug), NOT Display | BC-2.11.024 invariant 2 | `Direction` enum must lack `Display` impl; code review of csv.rs:84 |
| None optional fields produce empty string via `unwrap_or("")` / `unwrap_or_default()`; never a sentinel value | BC-2.11.024 invariant 4 | Test: check output does not contain `"null"`, `"N/A"`, or `"-"` |
| All four optional field strings are individually neutralized before write | BC-2.11.024 postcondition 5 | Code review: csv.rs:94-97 |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| csv | (per Cargo.lock) | RFC 4180 row writing |
| chrono | (per Cargo.lock) | `DateTime::to_rfc3339()` for timestamp encoding |
| std::net::IpAddr | stdlib | `IpAddr::to_string()` for IPv4/IPv6 address encoding |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/reporter/csv.rs | verify/modify | Optional field encoding (csv.rs:82-85), neutralize (94-97), Reporter trait impl (51-106) |
| src/reporter/mod.rs | verify | `Reporter` trait definition (mod.rs:26-33) |
| tests/reporter_csv_tests.rs | create or modify | AC-001 through AC-012 tests |

## Revision History

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-05-21 | story-writer | Initial story |
| 1.1 | 2026-05-21 | story-writer | (prior revision) |
| 1.2 | 2026-05-30 | story-writer | v1.2: corrected test-file citation reporter_tests.rs → reporter_csv_tests.rs (FSR + Token Budget rows); Wave-22 P1/P2 finding |
| 1.4 | 2026-06-09 | story-writer | UPDATED (Feature #7 migration note): STORY-080 covers `CsvReporter` trait compliance and optional-field encoding, including `mitre_technique` `None` → empty string. After STORY-101 (v0.3.0), the field is `mitre_techniques: Vec<String>` and empty-vec encodes as empty string (identical external behavior). The `skip_serializing_if` contract for the field moves from `Option::is_none` to `Vec::is_empty`; CSV behavior is unchanged. Story status remains `completed`; no re-implementation required. |
