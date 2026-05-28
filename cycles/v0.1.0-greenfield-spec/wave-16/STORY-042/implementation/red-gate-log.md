# Red Gate Log — STORY-042

**Story:** STORY-042 URI-Based Threat Detections (Path Traversal, Web Shell, Admin Panel)
**Branch:** feature/STORY-042
**Commit:** 9ef5670
**Date:** 2026-05-28
**Mode:** brownfield-formalization (implementation already exists; tests confirm conformance)

## Summary

All 10 formal tests added in `mod bc_2_06_story042_formalization` (tests/http_analyzer_tests.rs)
PASSED against the existing brownfield source in `src/analyzer/http.rs`.

This is expected in brownfield-formalization mode: the tests are formalizing existing behavior,
not driving new implementation. The Red Gate condition (all tests FAIL before implementation)
does not apply here — instead, the gate is: **all tests PASS**, confirming the source
already conforms to the contracted postconditions.

## Test Results by AC

| AC | Test Function | BC | Result | Notes |
|----|--------------|-----|--------|-------|
| AC-001 | `test_BC_2_06_005_path_traversal_all_fields` | BC-2.06.005 pc-1 | PASS | Verifies all fields: category=Reconnaissance, verdict=Likely, confidence=High, mitre=T1083, summary prefix, evidence prefix+URI, direction=ClientToServer |
| AC-002 | `test_BC_2_06_005_encoded_traversal_four_patterns` | BC-2.06.005 inv-1 | PASS | Verifies all 4 patterns (../ ..%2f ..%252f ....//) trigger finding; verifies backslash (..\) does NOT trigger; verifies case-insensitivity |
| AC-003 | `test_path_traversal_fires_per_request` | BC-2.06.005 pc-2 | PASS | Two pipelined traversal requests emit two separate findings |
| AC-004 | `test_BC_2_06_006_webshell_path_all_fields` | BC-2.06.006 pc-1 | PASS | Verifies all fields: category=Execution, verdict=Likely, confidence=Medium, mitre=T1505.003, summary prefix, evidence prefix+URI, direction=ClientToServer |
| AC-005 | `test_webshell_case_insensitive` | BC-2.06.006 inv-1,2 | PASS | Uppercase /SHELL.PHP triggers; all 10 patterns individually verified; substring match verified |
| AC-006 | `test_BC_2_06_007_admin_panel_all_fields` | BC-2.06.007 pc-1 | PASS | All 4 patterns with full field assertions: category=Reconnaissance, verdict=Inconclusive, confidence=Low, mitre=T1046, summary prefix, evidence prefix+URI, direction=ClientToServer |
| AC-007 | `test_admin_panel_case_insensitive` | BC-2.06.007 inv-1,2 | PASS | /ADMIN uppercase triggers; /site/admin/settings triggers via substring; /WP-Admin triggers via lowercase |
| AC-008 | `test_multiple_detections_fire_independently` | BC-2.06.005 inv-3 + BC-2.06.006 inv-4 | PASS | /cmd.php/../etc/passwd emits both T1083 and T1505.003; /wp-admin/../shell.php emits all three (T1083, T1505.003, T1046) |
| AC-009 | `test_BC_2_06_012_normal_request_zero_findings` | BC-2.06.012 pc-1,2,3 | PASS | Zero findings; method/host/UA/URI counters updated normally; parse_errors=0 |
| AC-010 | `test_BC_2_06_012_normal_request_no_parse_errors` | BC-2.06.012 inv-1 | PASS | parse_errors=0; no T1083/T1505.003/T1046 findings; findings.is_empty() |

## Edge Cases Covered

| EC-ID | URI | Covered by |
|-------|-----|-----------|
| EC-001 | `/../etc/passwd` | test_BC_2_06_005_path_traversal_all_fields |
| EC-002 | `..%2fetc%2fpasswd` | test_BC_2_06_005_encoded_traversal_four_patterns |
| EC-003 | `..%252f` (double-encoded) | test_BC_2_06_005_encoded_traversal_four_patterns |
| EC-004 | `....//etc/passwd` | test_BC_2_06_005_encoded_traversal_four_patterns |
| EC-005 | `/shell.php` | test_webshell_case_insensitive (all-patterns loop) |
| EC-006 | `/uploads/SHELL.PHP` | test_webshell_case_insensitive |
| EC-007 | `/cmd.php/../etc/passwd` | test_multiple_detections_fire_independently |
| EC-008 | `/wp-admin/edit.php` | test_BC_2_06_007_admin_panel_all_fields |
| EC-009 | `/ADMIN` (uppercase) | test_admin_panel_case_insensitive |
| EC-010 | `/index.html` | test_BC_2_06_012_normal_request_zero_findings + test_BC_2_06_012_normal_request_no_parse_errors |

## Source Divergences Found

**None.** The existing brownfield implementation in `src/analyzer/http.rs` (lines 183–357)
fully conforms to all BC postconditions, invariants, and edge cases.

Specific conformance points verified:
- Exactly 4 path-traversal patterns at http.rs:187-191 (no backslash variant)
- Exactly 10 web-shell patterns at http.rs:206-217
- Exactly 4 admin-panel patterns at http.rs:236
- URI lowercased before all pattern checks (http.rs:184)
- `truncate_uri(&parsed.uri, 120)` in summary, full URI in evidence (http.rs:196-201, 223-227, 242-246)
- `direction: Some(Direction::ClientToServer)` on all three detection blocks
- `Finding::mitre_technique` correct per BC for each detection type

## Overall Gate Status

PASS — source already conforms to all BC clauses. Tests are formalized and committed.
Ready for handoff to implementer (no implementation needed; behavior already exists).
