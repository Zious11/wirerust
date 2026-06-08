# Demo Evidence Report — STORY-077

**Story:** STORY-077 — TerminalReporter: escape_for_terminal, skipped_packets, and End-to-End C1 Safety
**Wave:** 21
**Epic:** E-8 (Reporter formalization)
**Strategy:** brownfield-formalization (zero src/ changes; tests only)
**Evidence Date:** 2026-05-30
**Branch:** test/story-077-terminal-reporter

---

## Summary

14/14 AC → test PASS. All 14 acceptance criteria are covered by passing tests in
`tests/reporter_terminal_tests.rs` (mod `story_077`). Zero source changes — the production
implementation in `src/reporter/terminal.rs` already satisfies all ACs (confirmed by
brownfield formalization).

Full test suite: **929 passed / 0 failed / 0 ignored**.

---

## Per-AC Evidence

| AC | Test Name | BC | Result |
|----|-----------|-----|--------|
| AC-001 | `test_BC_2_11_006_skipped_packets_zero_no_line` | BC-2.11.006 pc2 | PASS |
| AC-002 | `test_BC_2_11_006_skipped_packets_nonzero_line_present` | BC-2.11.006 pc1 | PASS |
| AC-003 | `test_BC_2_11_007_esc_byte_escaped` | BC-2.11.007 pc1 | PASS |
| AC-004 | `test_BC_2_11_007_del_escaped` | BC-2.11.007 pc2 | PASS |
| AC-005 | `test_BC_2_11_007_backslash_escaped` | BC-2.11.007 pc4 | PASS |
| AC-006 | `test_BC_2_11_008_printable_ascii_preserved` | BC-2.11.008 pc1 | PASS |
| AC-007 | `test_BC_2_11_008_cyrillic_and_emoji_preserved` | BC-2.11.008 pc2 | PASS |
| AC-008 | `test_BC_2_11_009_c1_range_escaped` | BC-2.11.009 pc1 | PASS |
| AC-009 | `test_BC_2_11_009_nbsp_u00a0_preserved` | BC-2.11.009 pc2 | PASS |
| AC-010 | `test_BC_2_11_009_c1_boundary_inclusive` | BC-2.11.009 inv2 | PASS |
| AC-011 | `test_BC_2_11_010_summary_is_escaped` | BC-2.11.010 pc1 | PASS |
| AC-012 | `test_BC_2_11_010_evidence_each_entry_is_escaped` | BC-2.11.010 pc2 | PASS |
| AC-013 | `test_BC_2_11_011_analyzer_detail_c1_escaped` | BC-2.11.011 pc1 | PASS |
| AC-014 | `test_BC_2_11_012_http_finding_c1_end_to_end` | BC-2.11.012 pc1 | PASS |

---

## Traceability Chain

```
BC-2.11.006 → AC-001, AC-002
BC-2.11.007 → AC-003, AC-004, AC-005
BC-2.11.008 → AC-006, AC-007
BC-2.11.009 → AC-008, AC-009, AC-010
BC-2.11.010 → AC-011, AC-012
BC-2.11.011 → AC-013
BC-2.11.012 → AC-014
```

---

## Key Behavioral Properties Verified

1. **Terminal injection prevention**: No attacker-controlled C0/DEL/C1/backslash bytes pass
   through to terminal output.
2. **Unicode preservation**: Cyrillic, emoji, NBSP (U+00A0) pass through unchanged — only the
   dangerous control range is escaped.
3. **C1 range inclusive**: U+0080 and U+009F both escape; U+00A0 does not.
4. **Analyzer detail escaping**: C1 bytes in `AnalysisSummary.detail` values are also escaped.
5. **End-to-end safety**: HTTP path-traversal findings with C1 CSI in summary are safely rendered.
6. **Forensic preservation**: `Finding.summary` holds raw C1 bytes pre-render (INV-4); only the
   terminal renderer escapes at display time.

---

## Intentional Asymmetry (Design Decision)

Terminal vs. JSON escaping is intentionally asymmetric per ADR 0003 / BC-2.11.005 inv2:
- `JsonReporter` passes C1 through as raw UTF-8 (RFC 8259 scope — JSON validators handle it).
- `TerminalReporter` escapes C1 (terminal safety — prevents CSI injection).
This is NOT a contradiction. Both behaviors are correct and intentional.

---

## Deferred Items

- **VP-012** (escape_for_terminal proptest — property-based verification of the escape predicate):
  Deferred to Phase-6 formal hardening. Requires symbolic execution tooling not yet in pipeline.

---

## CI Gate Status (at evidence collection)

| Check | Status |
|-------|--------|
| cargo test --all-targets | 929 passed / 0 failed |
| cargo clippy --all-targets -D warnings | clean |
| cargo fmt --check | clean |
| src/ diff vs develop | empty (zero source changes) |
