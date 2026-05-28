# Demo Evidence Report — STORY-043

**Story:** STORY-043 — Header and Method Anomaly Detections (Method, Host, URI Length, User-Agent)
**Module:** `tests/http_analyzer_tests.rs :: bc_2_06_043_formalization`
**Branch:** `feature/STORY-043`
**Date:** 2026-05-28
**Implementation strategy:** brownfield-formalization (no runtime change; deliverable is the test module)
**Recording tool:** VHS 0.11.0
**Font:** Menlo (system)

## Test Suite Summary

```
running 14 tests
test bc_2_06_043_formalization::test_BC_2_06_011_empty_ua_and_missing_host_both_fire_independently ... ok
test bc_2_06_043_formalization::test_detect_empty_host_header ... ok
test bc_2_06_043_formalization::test_BC_2_06_008_all_four_unusual_methods_emit_finding ... ok
test bc_2_06_043_formalization::test_BC_2_06_010_long_uri_and_path_traversal_both_fire_independently ... ok
test bc_2_06_043_formalization::test_BC_2_06_010_very_long_uri_evidence_truncated_to_200 ... ok
test bc_2_06_043_formalization::test_detect_empty_user_agent ... ok
test bc_2_06_043_formalization::test_missing_user_agent_no_finding ... ok
test bc_2_06_043_formalization::test_detect_unusual_method ... ok
test bc_2_06_043_formalization::test_detect_missing_host_header ... ok
test bc_2_06_043_formalization::test_http10_no_host_finding ... ok
test bc_2_06_043_formalization::test_whitespace_user_agent_triggers_empty_ua_finding ... ok
test bc_2_06_043_formalization::test_detect_long_uri ... ok
test bc_2_06_043_formalization::test_unusual_method_case_sensitive ... ok
test bc_2_06_043_formalization::test_long_uri_boundary_exactly_2048 ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 61 filtered out; finished in 0.00s
```

## AC-to-Evidence Mapping

| AC | BC | Test Name | Path Type | Tape | GIF | WebM |
|----|----|-----------|-----------|------|-----|------|
| AC-000 (full module) | all | `bc_2_06_043_formalization` (14 tests) | success | [AC-000-full-suite.tape](AC-000-full-suite.tape) | [AC-000-full-suite.gif](AC-000-full-suite.gif) | [AC-000-full-suite.webm](AC-000-full-suite.webm) |
| AC-001 | BC-2.06.008 post-1 | `test_detect_unusual_method` | success (CONNECT/TRACE/DELETE/OPTIONS each emit Reconnaissance/Inconclusive/Medium) | [AC-001-detect-unusual-method.tape](AC-001-detect-unusual-method.tape) | [AC-001-detect-unusual-method.gif](AC-001-detect-unusual-method.gif) | [AC-001-detect-unusual-method.webm](AC-001-detect-unusual-method.webm) |
| AC-002 | BC-2.06.008 inv-1/2 | `test_unusual_method_case_sensitive` | boundary/"no-finding" (lowercase "delete" and standard methods GET/POST/PUT/PATCH/HEAD do not fire) | [AC-002-method-case-sensitive.tape](AC-002-method-case-sensitive.tape) | [AC-002-method-case-sensitive.gif](AC-002-method-case-sensitive.gif) | [AC-002-method-case-sensitive.webm](AC-002-method-case-sensitive.webm) |
| AC-003 | BC-2.06.009 post-1 | `test_detect_missing_host_header` | success (HTTP/1.1 + host=None → "without Host header" finding) | [AC-003-missing-host-header.tape](AC-003-missing-host-header.tape) | [AC-003-missing-host-header.gif](AC-003-missing-host-header.gif) | [AC-003-missing-host-header.webm](AC-003-missing-host-header.webm) |
| AC-004 | BC-2.06.009 post-1 | `test_detect_empty_host_header` | success (HTTP/1.1 + host=Some("") → "with empty Host header" finding, distinct summary) | [AC-004-empty-host-header.tape](AC-004-empty-host-header.tape) | [AC-004-empty-host-header.gif](AC-004-empty-host-header.gif) | [AC-004-empty-host-header.webm](AC-004-empty-host-header.webm) |
| AC-005 | BC-2.06.009 post-3 | `test_http10_no_host_finding` | boundary/"no-finding" (HTTP/1.0 version==0 is fully exempt; absent and whitespace-only Host both suppressed) | [AC-005-http10-no-host.tape](AC-005-http10-no-host.tape) | [AC-005-http10-no-host.gif](AC-005-http10-no-host.gif) | [AC-005-http10-no-host.webm](AC-005-http10-no-host.webm) |
| AC-006 | BC-2.06.010 post-1 | `test_detect_long_uri` | success (uri.len() > 2048 → Execution/Likely/Medium; summary includes exact byte count; evidence truncated to 200 chars) | [AC-006-long-uri.tape](AC-006-long-uri.tape) | [AC-006-long-uri.gif](AC-006-long-uri.gif) | [AC-006-long-uri.webm](AC-006-long-uri.webm) |
| AC-007 | BC-2.06.010 inv-1/3 | `test_long_uri_boundary_exactly_2048` | boundary (uri.len()==2048 → NO finding; uri.len()==2049 → finding fires) | [AC-007-long-uri-boundary-2048.tape](AC-007-long-uri-boundary-2048.tape) | [AC-007-long-uri-boundary-2048.gif](AC-007-long-uri-boundary-2048.gif) | [AC-007-long-uri-boundary-2048.webm](AC-007-long-uri-boundary-2048.webm) |
| AC-008 | BC-2.06.011 post-1 | `test_detect_empty_user_agent` | success (user_agent=Some("") → Anomaly/Inconclusive/Low "Empty User-Agent header") | [AC-008-empty-user-agent.tape](AC-008-empty-user-agent.tape) | [AC-008-empty-user-agent.gif](AC-008-empty-user-agent.gif) | [AC-008-empty-user-agent.webm](AC-008-empty-user-agent.webm) |
| AC-009 | BC-2.06.011 post-2/inv-2 | `test_missing_user_agent_no_finding` | no-finding (user_agent=None → NO finding emitted; Kheir 2015 asymmetry) | [AC-009-missing-user-agent-no-finding.tape](AC-009-missing-user-agent-no-finding.tape) | [AC-009-missing-user-agent-no-finding.gif](AC-009-missing-user-agent-no-finding.gif) | [AC-009-missing-user-agent-no-finding.webm](AC-009-missing-user-agent-no-finding.webm) |
| AC-010 | BC-2.06.011 inv-1 | `test_whitespace_user_agent_triggers_empty_ua_finding` | boundary (whitespace-only UA after trim → Some("") → finding fires) | [AC-010-whitespace-user-agent.tape](AC-010-whitespace-user-agent.tape) | [AC-010-whitespace-user-agent.gif](AC-010-whitespace-user-agent.gif) | [AC-010-whitespace-user-agent.webm](AC-010-whitespace-user-agent.webm) |

## Coverage Classification

| Path Type | ACs Covered |
|-----------|------------|
| Success / detection-firing | AC-001, AC-003, AC-004, AC-006, AC-008 |
| No-finding / absent-does-not-fire | AC-002, AC-005, AC-009 |
| Boundary (strictly-greater-than threshold) | AC-007, AC-010 |
| Full module (all 14 tests) | AC-000 |

## Additional Tests in Module (beyond the 10 named ACs)

The `bc_2_06_043_formalization` module contains 14 tests total. Four tests cover compound / co-occurrence scenarios beyond the 10 AC-named tests:

| Test | Behavioral Contract | What It Proves |
|------|---------------------|----------------|
| `test_BC_2_06_008_all_four_unusual_methods_emit_finding` | BC-2.06.008 | All four unusual methods (CONNECT, TRACE, DELETE, OPTIONS) independently trigger findings |
| `test_BC_2_06_010_very_long_uri_evidence_truncated_to_200` | BC-2.06.010 | Evidence field is capped at 200 chars via `truncate_uri`; exact byte count in summary |
| `test_BC_2_06_010_long_uri_and_path_traversal_both_fire_independently` | BC-2.06.010 | Long URI and path-traversal detections co-occur independently on the same request |
| `test_BC_2_06_011_empty_ua_and_missing_host_both_fire_independently` | BC-2.06.011 | Empty UA and missing-Host findings both fire independently on the same HTTP/1.1 request |

## Artifacts

All files are in `docs/demo-evidence/STORY-043/`:

```
AC-000-full-suite.tape / .gif / .webm
AC-001-detect-unusual-method.tape / .gif / .webm
AC-002-method-case-sensitive.tape / .gif / .webm
AC-003-missing-host-header.tape / .gif / .webm
AC-004-empty-host-header.tape / .gif / .webm
AC-005-http10-no-host.tape / .gif / .webm
AC-006-long-uri.tape / .gif / .webm
AC-007-long-uri-boundary-2048.tape / .gif / .webm
AC-008-empty-user-agent.tape / .gif / .webm
AC-009-missing-user-agent-no-finding.tape / .gif / .webm
AC-010-whitespace-user-agent.tape / .gif / .webm
evidence-report.md
```

Total: 33 recording artifacts (11 tapes, 11 GIFs, 11 WebMs) + this report.
