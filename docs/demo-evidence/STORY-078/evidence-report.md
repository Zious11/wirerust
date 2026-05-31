# Demo Evidence Report — STORY-078

**Story:** STORY-078 — TerminalReporter: MITRE grouping, section order, and colorization
**Wave:** 22
**Epic:** E-8 (Reporter formalization)
**Strategy:** brownfield-formalization (zero src/ changes; tests only)
**Evidence Date:** 2026-05-30
**Branch:** test/story-078-terminal-grouping

---

## Summary

16/16 AC → test PASS. All 16 acceptance criteria are covered by passing tests in
`tests/reporter_terminal_tests.rs` (mod `story_078`). Zero source changes — the production
implementation in `src/reporter/terminal.rs` already satisfies all ACs (confirmed by
brownfield formalization).

Full test suite: **958 passed / 0 failed / 0 ignored**.

---

## Per-AC Evidence

| AC | Test Name | BC | Result |
|----|-----------|-----|--------|
| AC-001 | `test_BC_2_11_013_tactic_headers_in_canonical_order` | BC-2.11.013 pc2, inv3 | PASS |
| AC-002 | `test_BC_2_11_013_uncategorized_last` | BC-2.11.013 pc4 | PASS |
| AC-003 | `test_BC_2_11_014_sort_by_verdict_within_bucket` | BC-2.11.014 pc1 | PASS |
| AC-004 | `test_BC_2_11_014_sort_by_confidence_within_same_verdict` | BC-2.11.014 pc2 | PASS |
| AC-005 | `test_BC_2_11_014_stable_emission_order_on_tie` | BC-2.11.014 pc3, inv3 | PASS |
| AC-006 | `test_BC_2_11_015_none_technique_uncategorized` | BC-2.11.015 pc1, pc4 | PASS |
| AC-007 | `test_BC_2_11_015_unknown_id_uncategorized_with_label` | BC-2.11.015 pc2, pc3 | PASS |
| AC-008 | `test_BC_2_11_016_known_id_em_dash_and_name` | BC-2.11.016 pc1, inv1 | PASS |
| AC-009 | `test_BC_2_11_016_separator_is_em_dash_not_ascii_hyphen` | BC-2.11.016 inv1 | PASS |
| AC-010 | `test_BC_2_11_017_default_mode_bare_mitre_id` | BC-2.11.017 pc1, inv1, inv2 | PASS |
| AC-011 | `test_BC_2_11_017_default_mode_no_tactic_headers` | BC-2.11.017 pc3 | PASS |
| AC-012 | `test_BC_2_11_018_no_ansi_codes_when_color_disabled` | BC-2.11.018 pc5 | PASS |
| AC-013 | `test_BC_2_11_019_header_is_first_section` | BC-2.11.019 pc1 | PASS |
| AC-014 | `test_BC_2_11_019_findings_section_absent_when_empty` | BC-2.11.019 pc4, inv2 | PASS |
| AC-015 | `test_BC_2_11_019_analyzer_sections_last_in_slice_order` | BC-2.11.019 pc5 | PASS |
| AC-016 | `test_BC_2_11_019_services_section_absent_when_empty` | BC-2.11.019 inv3 | PASS |

---

## Traceability Chain

```
BC-2.11.013 → AC-001, AC-002
BC-2.11.014 → AC-003, AC-004, AC-005
BC-2.11.015 → AC-006, AC-007
BC-2.11.016 → AC-008, AC-009
BC-2.11.017 → AC-010, AC-011
BC-2.11.018 → AC-012
BC-2.11.019 → AC-013, AC-014, AC-015, AC-016
```

---

## Key Behavioral Properties Verified

1. **Canonical MITRE tactic grouping order**: When `show_mitre_grouping = true`, tactic
   section headers appear in the order returned by `all_tactics_in_report_order()` (ATT&CK
   kill-chain order). Only tactics with at least one finding are emitted (inv3). Verified:
   DefenseEvasion (index 6) precedes CommandAndControl (index 11).

2. **Uncategorized-last**: The `## Uncategorized` bucket is always emitted after all named
   tactic sections. Findings with `mitre_technique = None` and findings with unrecognized
   technique IDs both land in this bucket.

3. **Verdict-then-confidence stable sort within tactic bucket**: Within each tactic section,
   findings sort by verdict rank ascending (Likely=0, Inconclusive=1, Unlikely=2), then by
   confidence rank ascending (High=0, Medium=1, Low=2), with original slice index as the
   stable tiebreaker (Rust `sort_by_key` is stable).

4. **Em-dash (U+2014) technique separator**: In grouped mode, known technique IDs produce
   `MITRE: T1036 — Masquerading` (U+2014 literal, UTF-8 bytes 0xE2 0x80 0x94). ASCII `--`
   is never used as the separator.

5. **Unknown-ID label**: Unrecognized technique IDs produce `MITRE: T9999 (unknown)` in
   grouped mode. The `(unknown)` label is absent in flat mode.

6. **NO ANSI when color disabled**: When `use_color = false`, the ANSI CSI introducer
   `\x1b[` is entirely absent from output for all verdict/confidence combinations (Likely/High,
   Likely/Medium, Inconclusive/High, Unlikely/Low).

7. **Section ordering — header first**: The `WIRERUST TRIAGE REPORT` header is always the
   first content in the output (byte offset 0 / `starts_with` assertion).

8. **FINDINGS absent when empty**: The FINDINGS section is entirely absent (not just empty)
   when the findings slice is empty (inv2). It appears when findings are non-empty (pc4).

9. **SERVICES absent when empty**: The SERVICES section is entirely absent when
   `service_counts()` returns an empty map (inv3). The PROTOCOLS section remains present
   regardless (always-on inv5).

10. **Analyzer sections last in slice order**: ANALYZER sections appear after FINDINGS, one
    per `AnalysisSummary` element, in the order they appear in the input slice (pc5).

---

## Flat Mode vs. Grouped Mode Contrast

| Property | Flat mode (`show_mitre_grouping = false`) | Grouped mode (`show_mitre_grouping = true`) |
|---|---|---|
| Tactic section headers | Absent | Present, in `all_tactics_in_report_order()` order |
| `## Uncategorized` | Absent | Present (last), if any uncategorized findings |
| MITRE line format | `MITRE: T1036` (bare ID only) | `MITRE: T1036 — Masquerading` (em-dash + name) |
| Unknown-ID label | Absent | `MITRE: T9999 (unknown)` |
| Within-tactic sort | N/A | verdict → confidence → stable emission index |

---

## Test Execution Evidence

```
running 16 tests
test story_078::test_BC_2_11_014_sort_by_confidence_within_same_verdict ... ok
test story_078::test_BC_2_11_013_uncategorized_last ... ok
test story_078::test_BC_2_11_014_sort_by_verdict_within_bucket ... ok
test story_078::test_BC_2_11_013_tactic_headers_in_canonical_order ... ok
test story_078::test_BC_2_11_014_stable_emission_order_on_tie ... ok
test story_078::test_BC_2_11_015_unknown_id_uncategorized_with_label ... ok
test story_078::test_BC_2_11_015_none_technique_uncategorized ... ok
test story_078::test_BC_2_11_017_default_mode_bare_mitre_id ... ok
test story_078::test_BC_2_11_016_separator_is_em_dash_not_ascii_hyphen ... ok
test story_078::test_BC_2_11_016_known_id_em_dash_and_name ... ok
test story_078::test_BC_2_11_017_default_mode_no_tactic_headers ... ok
test story_078::test_BC_2_11_018_no_ansi_codes_when_color_disabled ... ok
test story_078::test_BC_2_11_019_findings_section_absent_when_empty ... ok
test story_078::test_BC_2_11_019_analyzer_sections_last_in_slice_order ... ok
test story_078::test_BC_2_11_019_header_is_first_section ... ok
test story_078::test_BC_2_11_019_services_section_absent_when_empty ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
```

---

## CI Gate Status (at evidence collection)

| Check | Status |
|-------|--------|
| cargo test --test reporter_terminal_tests story_078 | 16 passed / 0 failed |
| cargo test --all-targets | 958 passed / 0 failed |
| src/ diff vs develop | empty (zero source changes) |
