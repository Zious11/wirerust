# Demo Evidence Report — STORY-079

**Story:** STORY-079 — CsvReporter: 9-Column Schema, RFC 4180 Quoting, Injection Neutralization, Evidence Join
**Wave:** 21
**Epic:** E-8 (Reporter formalization)
**Strategy:** brownfield-formalization (zero src/ changes; tests only)
**Evidence Date:** 2026-05-30
**Branch:** test/story-079-csv-reporter

---

## Summary

13/13 AC → test PASS. All 13 acceptance criteria are covered by passing tests in
`tests/reporter_csv_tests.rs` (mod `story_079`). Zero source changes — the production
implementation in `src/reporter/csv.rs` already satisfies all ACs (confirmed by
brownfield formalization).

Full test suite: **942 passed / 0 failed / 0 ignored**.

---

## Per-AC Evidence

| AC | Test Name | BC | Result |
|----|-----------|-----|--------|
| AC-001 | `test_BC_2_11_020_header_row_first_and_exact` | BC-2.11.020 pc1 | PASS |
| AC-002 | `test_BC_2_11_020_every_row_has_nine_columns` | BC-2.11.020 pc2/inv1 | PASS |
| AC-003 | `test_BC_2_11_020_comma_in_field_does_not_change_column_count` | BC-2.11.020 inv1 | PASS |
| AC-004 | `test_BC_2_11_020_rfc4180_quoting` | BC-2.11.020 pc4 | PASS |
| AC-005 | `test_BC_2_11_021_neutralize_all_six_trigger_chars` | BC-2.11.021 pc1/VP-020 | PASS |
| AC-006 | `test_BC_2_11_021_no_trigger_no_change` | BC-2.11.021 pc2 | PASS |
| AC-007 | `test_BC_2_11_021_empty_string_unchanged` | BC-2.11.021 pc4 | PASS |
| AC-008 | `test_BC_2_11_021_trigger_at_position_2_no_prefix` | BC-2.11.021 inv2 | PASS |
| AC-009 | `test_BC_2_11_021_applied_to_all_nine_columns` | BC-2.11.021 inv1 | PASS |
| AC-010 | `test_BC_2_11_022_evidence_joined_with_semicolon_space` | BC-2.11.022 pc1/inv1 | PASS |
| AC-011 | `test_BC_2_11_022_empty_evidence_is_empty_cell` | BC-2.11.022 pc2 | PASS |
| AC-012 | `test_BC_2_11_022_single_evidence_no_separator` | BC-2.11.022 pc3 | PASS |
| AC-013 | `test_BC_2_11_022_evidence_join_then_neutralize` | BC-2.11.022 pc4 | PASS |

---

## Traceability Chain

```
BC-2.11.020 → AC-001, AC-002, AC-003, AC-004
BC-2.11.021 → AC-005, AC-006, AC-007, AC-008, AC-009
BC-2.11.022 → AC-010, AC-011, AC-012, AC-013
```

---

## Key Behavioral Properties Verified

1. **9-column fixed schema**: Every row (header and data) emits exactly nine fields in order:
   `category, verdict, confidence, summary, evidence, mitre_technique, source_ip, direction, timestamp`.
   Column count is invariant regardless of field content.

2. **RFC 4180 quoting**: Fields containing commas are double-quoted per RFC 4180 (e.g.
   `"attack,payload,here"`). Fields containing double-quotes use `""` escaping. Column count
   remains exactly 9 in both cases when parsed by an RFC 4180-compliant reader.

3. **LF line terminator**: `csv::WriterBuilder::new()` default — LF (`\n`) not CRLF. RFC 4180
   readers (including the `csv` crate) accept LF as a valid record terminator.

4. **CSV formula-injection neutralization**: `neutralize_csv_injection` prepends a leading
   single quote (`'`) to any cell whose first character is one of the six trigger characters:
   `=`, `+`, `-`, `@`, TAB (U+0009), CR (U+000D). This prevents spreadsheet applications from
   interpreting cell content as a formula.

5. **Neutralization applied to ALL nine columns**: Not just the summary column — every string
   field (columns 4-9, where non-None) passes through `neutralize_csv_injection` before
   being written.

6. **Trigger-position exclusivity**: Only the FIRST character triggers neutralization. A
   trigger character at position 2+ passes through unchanged (e.g. `"a=formula"` → `"a=formula"`).

7. **Empty-string safety**: An empty string input produces an empty cell with no `'` prefix
   added and no panic. Row column count remains 9.

8. **Evidence join order**: Multiple evidence entries are joined with `"; "` (semicolon then
   space) into a single cell BEFORE neutralization. Neutralization is applied to the joined
   string — not to individual elements — so only the first character of the joined result
   determines whether a prefix is added.

---

## Verification Property Coverage

| VP | Proof Method | Status |
|----|-------------|--------|
| VP-020 | unit: parametric test over all 6 trigger chars (`test_BC_2_11_021_neutralize_all_six_trigger_chars`) | SATISFIED IN-STORY |

VP-020 is an in-story unit proof, not deferred to Phase-6 formal hardening. The BC-2.11.021
Verification Properties table cites VP-020 with proof_method "unit: parametric test over trigger
set" — this story's parametric test satisfies that requirement.

---

## Deferred Items

None. All 13 ACs are fully covered with no deferred proof obligations.

---

## CI Gate Status (at evidence collection)

| Check | Status |
|-------|--------|
| `cargo test --test reporter_csv_tests -- --nocapture` | 13 passed / 0 failed |
| `cargo test --all-targets` | 942 passed / 0 failed |
| src/ diff vs develop | empty (zero source changes) |
