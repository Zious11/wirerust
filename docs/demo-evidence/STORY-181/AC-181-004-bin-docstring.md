# AC-181-004: parse_line() Docstring Clarified for Regex-Mismatch None Return

**AC:** AC-181-004  
**Story:** STORY-181 (ROUTE-W74 OBS-1 residual, bin/ housekeeping)  
**Date:** 2026-07-24  
**Branch:** feature/STORY-181-enip-sec001-split-borrow

---

## Verdict: PASS

---

## parse_line() Docstring (bin/validate-citations, lines 116–131)

The third return case (regex-mismatch None) is now documented:

```python
def parse_line(raw: str) -> tuple[str, int, int | None, str | None] | None:
    """Parse one line from a citations table.

    Returns (path, start_line, end_line_or_None, anchor_or_None) for a valid
    citation line, or None if the line should be skipped (blank or comment),
    or None if the line fails the citation regex (caller should treat as
    MALFORMED). Callers distinguish the skip case from the MALFORMED case by
    re-checking the stripped/comment status of the raw line, per F-S164P1-002.
    """
    stripped = raw.strip()
    if not stripped or stripped.startswith("#"):
        return None
    m = _CITATION_RE.match(raw)
    if not m:
        return None
    ...
```

OBS-1 (wave-74 gate-summary carry-forward) is resolved: the docstring now enumerates
all three `None` return cases:
1. Valid citation line → returns `(path, start_line, end_line_or_None, anchor_or_None)`
2. Blank or comment line → returns `None` (skip)
3. Regex mismatch → returns `None` (caller treats as MALFORMED)

---

## bin/test_validate_citations.py: 27/27 PASS

Command:
```
python3 bin/test_validate_citations.py
```

Output:
```
test_T01_valid_line_citation_passes:
  [PASS] T01 valid single-line citation: exit=0, out='PASS: 1 citations verified'

test_T02_valid_range_citation_passes:
  [PASS] T02 valid range citation: exit=0, out='PASS: 1 citations verified'

test_T03_nonexistent_file_rejected:
  [PASS] T03 nonexistent file rejected: exit=1

test_T04_out_of_range_single_line_rejected:
  [PASS] T04 out-of-range single line rejected: exit=1

test_T05_out_of_range_range_endpoint_rejected:
  [PASS] T05 out-of-range range endpoint rejected: exit=1

test_T06_comments_and_blanks_ignored:
  [PASS] T06 comments and blank lines ignored: exit=0, out='PASS: 1 citations verified'

test_T07_empty_input_passes:
  [PASS] T07 empty input: exit=0, out='PASS: 0 citations verified'

test_T08_invalid_range_start_gt_end:
  [PASS] T08 invalid range (start > end) rejected: exit=1

test_T09_bad_argument_exits_2:
  [PASS] T09 nonexistent citations file → exit 2: exit=2

test_T10_multiple_valid_citations_count:
  [PASS] T10 multiple valid citations: exit=0, out='PASS: 3 citations verified'

test_T11_mixed_valid_and_invalid:
  [PASS] T11 mixed valid+invalid → FAIL with count: exit=1

test_T12_malformed_line_reported:
  [PASS] T12 malformed citation reported: exit=1

test_T13_zero_line_number_rejected:
  [PASS] T13 zero line number rejected: exit=1

test_T14_zero_range_start_rejected:
  [PASS] T14 zero range start rejected: exit=1

test_T15_malformed_counts_in_fail_denominator:
  [PASS] T15 malformed counted in FAIL denominator: exit=1

test_T16_absolute_path_rejected:
  [PASS] T16 absolute path rejected as OUTSIDE REPO: exit=1

test_T17_parent_escape_rejected:
  [PASS] T17 parent-escape path rejected as OUTSIDE REPO: exit=1

test_T18_non_utf8_citations_file_exits_2:
  [PASS] T18 non-UTF-8 citations file → exit 2: exit=2

test_T19_unreadable_citations_file_exits_2:
  [PASS] T19 unreadable file → exit 2: exit=2

test_T20_non_utf8_stdin_exits_2:
  [PASS] T20 non-UTF-8 stdin → exit 2: exit=2

test_T21_directory_target_not_a_file:
  [PASS] T21 directory target → NOT A FILE, exit 1: exit=1

test_T22_unreadable_target_file:
  [PASS] T22 unreadable target → UNREADABLE, exit 1: exit=1

test_T23_anchor_present_passes:
  [PASS] T23 anchor present (+ EC-002 regex-special anchor): exit=0, out='PASS: 2 citations verified'

test_T24_anchor_absent_symbol_not_at_line:
  [PASS] T24 anchor absent -> SYMBOL NOT AT LINE, exit 1: exit=1

test_T25_bare_citation_still_passes:
  [PASS] T25 bare citation backward-compat control: exit=0, out='PASS: 1 citations verified'

test_T26_range_citation_anchor_asserts_start_line_only:
  [PASS] T26 range anchor asserts start-line-only: a_exit=0, b_exit=1

test_T27_symbol_failure_message_truncates_long_line:
  [PASS] T27 long-line SYMBOL NOT AT LINE message truncated to <=80 chars: exit=1, found_len=80

============================================================
Results: 27 passed, 0 failed
All tests passed.
```

All 27 tests pass. The docstring addition is behavior-neutral.
