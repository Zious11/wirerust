# Evidence Report — STORY-042

**Story:** STORY-042 — URI-Based Threat Detections: Path Traversal, Web Shell, Admin Panel
**Implementation Strategy:** brownfield-formalization
**Test Module:** `bc_2_06_story042_formalization` in `tests/http_analyzer_tests.rs`
**Recording Tool:** VHS (CLI)
**Date:** 2026-05-28
**Branch:** feature/STORY-042

## Summary

All 10 acceptance criteria are covered by 11 named tests in `bc_2_06_story042_formalization`.
All tests pass: `test result: ok. 11 passed; 0 failed; 0 ignored`.
Evidence is captured as VHS terminal recordings (.gif + .webm) for each AC.

## Full-Suite Recording

| Artifact | Purpose |
|----------|---------|
| [AC-ALL-full-suite.gif](AC-ALL-full-suite.gif) | All 11 tests pass in a single run |
| [AC-ALL-full-suite.webm](AC-ALL-full-suite.webm) | Archival |
| [AC-ALL-full-suite.tape](AC-ALL-full-suite.tape) | VHS script |

**Command:** `cargo test --test http_analyzer_tests bc_2_06_story042_formalization -- --nocapture`
**Result:** `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 61 filtered out`

---

## Per-AC Evidence Mapping

### AC-001 — Path-traversal finding: all fields correct (BC-2.06.005 postcondition 1)

- **Test:** `test_BC_2_06_005_path_traversal_all_fields`
- **Path (success):** URI `/../etc/passwd` triggers a Finding with category=Reconnaissance, verdict=Likely, confidence=High, mitre_technique=Some("T1083"), direction=Some(ClientToServer)
- **Artifacts:**
  - [AC-001-002-path-traversal-detection.gif](AC-001-002-path-traversal-detection.gif)
  - [AC-001-002-path-traversal-detection.webm](AC-001-002-path-traversal-detection.webm)
  - [AC-001-002-path-traversal-detection.tape](AC-001-002-path-traversal-detection.tape)

---

### AC-002 — Exactly four path-traversal patterns; no backslash variant (BC-2.06.005 invariant 1)

- **Test:** `test_BC_2_06_005_encoded_traversal_four_patterns`
- **Path (success):** All four patterns (`../`, `..%2f`, `..%252f`, `....//`) each trigger a finding; backslash variant does not
- **Artifacts:**
  - [AC-002-encoded-patterns.gif](AC-002-encoded-patterns.gif)
  - [AC-002-encoded-patterns.webm](AC-002-encoded-patterns.webm)
  - [AC-002-encoded-patterns.tape](AC-002-encoded-patterns.tape)

---

### AC-003 — Path-traversal fires per-request, not per-flow-once (BC-2.06.005 postcondition 2)

- **Test:** `test_path_traversal_fires_per_request`
- **Path (success):** Two pipelined requests each containing `../` each emit a separate finding (2 total)
- **Artifacts:**
  - [AC-003-per-request-firing.gif](AC-003-per-request-firing.gif)
  - [AC-003-per-request-firing.webm](AC-003-per-request-firing.webm)
  - [AC-003-per-request-firing.tape](AC-003-per-request-firing.tape)

---

### AC-004 — Web-shell URI: all fields correct (BC-2.06.006 postcondition 1)

- **Test:** `test_BC_2_06_006_webshell_path_all_fields`
- **Path (success):** URI `/shell.php` triggers Finding with category=Execution, verdict=Likely, confidence=Medium, mitre_technique=Some("T1505.003")
- **Artifacts:**
  - [AC-004-005-webshell-detection.gif](AC-004-005-webshell-detection.gif)
  - [AC-004-005-webshell-detection.webm](AC-004-005-webshell-detection.webm)
  - [AC-004-005-webshell-detection.tape](AC-004-005-webshell-detection.tape)

---

### AC-005 — Web-shell detection is case-insensitive and substring-based (BC-2.06.006 invariants 1-2)

- **Test:** `test_webshell_case_insensitive`
- **Path (success):** URI `/uploads/C99.PHP?cmd=id` (uppercase) triggers a finding; lowercased match confirms `/c99.php` pattern
- **Artifacts:**
  - [AC-005-webshell-case-insensitive.gif](AC-005-webshell-case-insensitive.gif)
  - [AC-005-webshell-case-insensitive.webm](AC-005-webshell-case-insensitive.webm)
  - [AC-005-webshell-case-insensitive.tape](AC-005-webshell-case-insensitive.tape)

---

### AC-006 — Admin-panel path: all fields correct (BC-2.06.007 postcondition 1)

- **Test:** `test_BC_2_06_007_admin_panel_all_fields`
- **Path (success):** URI `/wp-admin/edit.php` triggers Finding with category=Reconnaissance, verdict=Inconclusive, confidence=Low, mitre_technique=Some("T1046")
- **Artifacts:**
  - [AC-006-007-admin-panel-detection.gif](AC-006-007-admin-panel-detection.gif)
  - [AC-006-007-admin-panel-detection.webm](AC-006-007-admin-panel-detection.webm)
  - [AC-006-007-admin-panel-detection.tape](AC-006-007-admin-panel-detection.tape)

---

### AC-007 — Admin-panel detection is case-insensitive and substring-based (BC-2.06.007 invariants 1-2)

- **Test:** `test_admin_panel_case_insensitive`
- **Path (success):** URI `/ADMIN` (uppercase) triggers a finding via the `/admin` pattern
- **Artifacts:**
  - [AC-007-admin-case-insensitive.gif](AC-007-admin-case-insensitive.gif)
  - [AC-007-admin-case-insensitive.webm](AC-007-admin-case-insensitive.webm)
  - [AC-007-admin-case-insensitive.tape](AC-007-admin-case-insensitive.tape)

---

### AC-008 — All URI detections are independent; a combined URI emits multiple findings (BC-2.06.005 invariant 3, BC-2.06.006 invariant 4)

- **Test:** `test_multiple_detections_fire_independently`
- **Path (success):** URI `/cmd.php/../etc/passwd` triggers both web-shell (T1505.003) and path-traversal (T1083) findings; neither suppresses the other
- **Artifacts:**
  - [AC-008-independent-detections.gif](AC-008-independent-detections.gif)
  - [AC-008-independent-detections.webm](AC-008-independent-detections.webm)
  - [AC-008-independent-detections.tape](AC-008-independent-detections.tape)

---

### AC-009 — Well-formed HTTP request produces zero findings (BC-2.06.012 postconditions 1-3)

- **Test:** `test_BC_2_06_012_normal_request_zero_findings`
- **Path (error/clean path):** `GET /index.html HTTP/1.1` with valid Host and User-Agent produces no findings; `all_findings` remains empty
- **Artifacts:**
  - [AC-009-010-zero-findings-clean.gif](AC-009-010-zero-findings-clean.gif)
  - [AC-009-010-zero-findings-clean.webm](AC-009-010-zero-findings-clean.webm)
  - [AC-009-010-zero-findings-clean.tape](AC-009-010-zero-findings-clean.tape)

---

### AC-010 — No anomaly detection fires on clean input; zero findings is the steady state (BC-2.06.012 invariant 1)

- **Test:** `test_BC_2_06_012_normal_request_no_parse_errors`
- **Path (error/clean path):** Clean GET request produces zero parse errors and zero findings — all detectors are independently gated and silent on legitimate traffic
- **Artifacts:** (shared with AC-009)
  - [AC-009-010-zero-findings-clean.gif](AC-009-010-zero-findings-clean.gif)
  - [AC-009-010-zero-findings-clean.webm](AC-009-010-zero-findings-clean.webm)
  - [AC-009-010-zero-findings-clean.tape](AC-009-010-zero-findings-clean.tape)

---

## Coverage Summary

| AC | BC | Test Name | Status | Recording |
|----|----|-----------|--------|-----------|
| AC-001 | BC-2.06.005 postcondition 1 | `test_BC_2_06_005_path_traversal_all_fields` | PASS | AC-001-002-path-traversal-detection |
| AC-002 | BC-2.06.005 invariant 1 | `test_BC_2_06_005_encoded_traversal_four_patterns` | PASS | AC-002-encoded-patterns |
| AC-003 | BC-2.06.005 postcondition 2 | `test_path_traversal_fires_per_request` | PASS | AC-003-per-request-firing |
| AC-004 | BC-2.06.006 postcondition 1 | `test_BC_2_06_006_webshell_path_all_fields` | PASS | AC-004-005-webshell-detection |
| AC-005 | BC-2.06.006 invariants 1-2 | `test_webshell_case_insensitive` | PASS | AC-005-webshell-case-insensitive |
| AC-006 | BC-2.06.007 postcondition 1 | `test_BC_2_06_007_admin_panel_all_fields` | PASS | AC-006-007-admin-panel-detection |
| AC-007 | BC-2.06.007 invariants 1-2 | `test_admin_panel_case_insensitive` | PASS | AC-007-admin-case-insensitive |
| AC-008 | BC-2.06.005 inv 3 / BC-2.06.006 inv 4 | `test_multiple_detections_fire_independently` | PASS | AC-008-independent-detections |
| AC-009 | BC-2.06.012 postconditions 1-3 | `test_BC_2_06_012_normal_request_zero_findings` | PASS | AC-009-010-zero-findings-clean |
| AC-010 | BC-2.06.012 invariant 1 | `test_BC_2_06_012_normal_request_no_parse_errors` | PASS | AC-009-010-zero-findings-clean |

**Additional bonus test also passing:** `test_BC_2_06_005_http10_path_traversal_not_exempt` (EC-011 edge case — HTTP/1.0 not exempt from path-traversal detection).

**Total:** 10/10 ACs covered. 11/11 tests pass. All recordings produced (.gif + .webm + .tape).
