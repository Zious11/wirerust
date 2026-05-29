---
document_type: story
story_id: "STORY-042"
epic_id: "E-4"
version: "1.4"
status: completed
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.005.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.006.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.007.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.012.md
input-hash: "9d85f8c"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-041]
blocks: [STORY-046]
behavioral_contracts:
  - BC-2.06.005
  - BC-2.06.006
  - BC-2.06.007
  - BC-2.06.012
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 16
target_module: src/analyzer/http.rs
subsystems: [SS-06]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **Execute:** `/vsdd-factory:deliver-story STORY-042`

# STORY-042: URI-Based Threat Detections — Path Traversal, Web Shell, Admin Panel

## Narrative
- **As a** forensic analyst
- **I want to** see structured findings when HTTP requests contain path-traversal patterns, web-shell URIs, or admin-panel paths — and to see zero findings for normal, well-formed requests
- **So that** I can quickly identify reconnaissance and execution-stage attacks from pcap captures

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.06.005 | Path Traversal in URI Emits Reconnaissance/Likely/High Finding Mapped to T1083 |
| BC-2.06.006 | Web-Shell URI Patterns Emit Execution/Likely/Medium Finding (T1505.003) |
| BC-2.06.007 | Admin Panel Paths Emit Reconnaissance/Inconclusive/Low Finding (T1046) |
| BC-2.06.012 | Well-Formed HTTP Request Produces Zero Findings |

## Acceptance Criteria

### AC-001 (traces to BC-2.06.005 postcondition 1)
When the parsed HTTP request URI (lowercased) contains any of `../`, `..%2f`, `..%252f`, or `....//`, a Finding is emitted with category=Reconnaissance, verdict=Likely, confidence=High, mitre_technique=Some("T1083"), summary containing the truncated URI (120 chars max), evidence containing the full raw URI, and direction=Some(Direction::ClientToServer).
- **Test:** `test_BC_2_06_005_path_traversal_all_fields`

### AC-002 (traces to BC-2.06.005 invariant 1)
The path-traversal check uses exactly four patterns: `../`, `..%2f`, `..%252f`, `....//`. There is NO backslash (`..\`) traversal variant in source. The URI is lowercased before matching.
- **Test:** `test_BC_2_06_005_encoded_traversal_four_patterns`

### AC-003 (traces to BC-2.06.005 postcondition 2)
The path-traversal detection fires per-request, not per-flow-once. Two pipelined requests each containing `../` emit two separate findings.
- **Test:** `test_path_traversal_fires_per_request`

### AC-004 (traces to BC-2.06.006 postcondition 1)
When the parsed HTTP request URI (lowercased) contains any of the 10 web-shell patterns (`/shell.php`, `/shell.asp`, `/shell.jsp`, `/cmd.php`, `/cmd.asp`, `/cmd.jsp`, `/c99.php`, `/r57.php`, `/webshell`, `/backdoor`), a Finding is emitted with category=Execution, verdict=Likely, confidence=Medium, mitre_technique=Some("T1505.003"), summary containing the truncated URI (120 chars max), and evidence containing the full raw URI.
- **Test:** `test_BC_2_06_006_webshell_path_all_fields`

### AC-005 (traces to BC-2.06.006 invariant 1-2)
Web-shell URI comparison is case-insensitive (lowercased before match) and substring-based — a URI like `/uploads/c99.php?cmd=id` triggers the finding. Pattern matching uses `shell_patterns.iter().any(|p| uri_lower.contains(p))`.
- **Test:** `test_webshell_case_insensitive`

### AC-006 (traces to BC-2.06.007 postcondition 1)
When the parsed HTTP request URI (lowercased) contains any of `/wp-admin`, `/admin`, `/phpmyadmin`, `/manager`, a Finding is emitted with category=Reconnaissance, verdict=Inconclusive, confidence=Low, mitre_technique=Some("T1046"), summary containing the truncated URI (120 chars max), and evidence containing the full raw URI.
- **Test:** `test_BC_2_06_007_admin_panel_all_fields`

### AC-007 (traces to BC-2.06.007 invariant 1-2)
Admin panel URI comparison is case-insensitive (lowercased before match) and substring-based — `/site/admin/settings` triggers the finding via `/admin`. Pattern matching uses `admin_patterns.iter().any(|p| uri_lower.contains(p))`.
- **Test:** `test_admin_panel_case_insensitive`

### AC-008 (traces to BC-2.06.005 invariant 3, BC-2.06.006 invariant 4, and BC-2.06.007 invariant 4)
All URI-based detections are independent: a request with a URI matching path-traversal, web-shell, and admin-panel patterns simultaneously emits all three findings. Detections do not suppress each other.
- **Test:** `test_multiple_detections_fire_independently`

### AC-009 (traces to BC-2.06.012 postcondition 1-3)
A syntactically valid HTTP/1.1 GET request with a standard method, URI length <= 2048, no traversal/shell/admin patterns, present non-empty Host, and absent or non-empty User-Agent produces zero findings. `all_findings` gains no new entries. Method/host/UA/URI counters update normally.
- **Test:** `test_BC_2_06_012_normal_request_zero_findings`

### AC-010 (traces to BC-2.06.012 invariant 1)
All anomaly detections are independently gated; none fires on clean input. Zero findings is the expected steady state for legitimate HTTP traffic.
- **Test:** `test_BC_2_06_012_normal_request_no_parse_errors`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| check_request_detections | src/analyzer/http.rs:183-357 | effectful-shell (pushes to all_findings) |
| path traversal detection | src/analyzer/http.rs:186-203 | effectful-shell |
| web shell detection | src/analyzer/http.rs:206-233 | effectful-shell |
| admin panel detection | src/analyzer/http.rs:235-249 | effectful-shell |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | URI = "/../etc/passwd" | Path-traversal finding emitted (T1083) |
| EC-002 | URI = "..%2fetc%2fpasswd" | Path-traversal finding emitted (URL-encoded) |
| EC-003 | URI = "..%252f" (double-encoded) | Path-traversal finding emitted |
| EC-004 | URI = "....//etc/passwd" | Path-traversal finding emitted |
| EC-005 | URI = "/shell.php" | Web-shell finding emitted (T1505.003) |
| EC-006 | URI = "/uploads/SHELL.PHP" | Web-shell finding emitted (case-insensitive) |
| EC-007 | URI = "/cmd.php/../etc/passwd" | Both web-shell and path-traversal findings emitted |
| EC-008 | URI = "/wp-admin/edit.php" | Admin-panel finding emitted (T1046) |
| EC-009 | URI = "/ADMIN" (uppercase) | Admin-panel finding emitted (case-insensitive) |
| EC-010 | URI = "/index.html" | Zero findings |
| EC-011 | HTTP/1.0 request with path traversal | Path-traversal finding emitted (HTTP/1.0 not exempt) |
| EC-012 | GET /index.html HTTP/1.1 with Host and UA | Zero findings |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/analyzer/http.rs (check_request_detections) | effectful-shell | Pushes to all_findings; reads self state |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| Referenced code (http.rs:183-357) | ~5,000 |
| Test files (http_analyzer_tests.rs) | ~3,000 |
| BC files (4 BCs) | ~4,000 |
| Tool outputs overhead | ~2,000 |
| **Total** | **~17,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~9%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-010 (test-writer)
2. [ ] Verify Red Gate: all tests fail before implementation
3. [ ] Implement path-traversal detection per BC-2.06.005 (four exact patterns, lowercase match, T1083, truncated summary, raw URI evidence)
4. [ ] Implement web-shell detection per BC-2.06.006 (10 patterns, lowercase match, T1505.003, truncated summary)
5. [ ] Implement admin-panel detection per BC-2.06.007 (4 patterns, lowercase match, T1046, Inconclusive/Low)
6. [ ] Confirm zero-findings on clean request (BC-2.06.012) — no detection fires on normal input
7. [ ] Confirm all three detections fire independently on a combined-pattern URI
8. [ ] Run all tests; verify all pass
9. [ ] Verify purity boundaries (all detection functions are effectful-shell)
10. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-041 | `check_request_detections` is invoked after every successful request parse; receives the ParsedRequest struct | URI is passed as raw bytes from httparse — do not re-encode before pattern match | `truncate_uri` must be UTF-8 char-boundary safe; truncation limit is 120 chars for summary, full URI for evidence |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Raw URI bytes preserved in evidence field; no escape at analyzer layer | ADR 0003 / INV-4 | Code review: no escape call between `parsed.uri` and `Finding.evidence` |
| Findings fire per-request, not per-flow-once | BC-2.06.005 postcondition 2 | Unit test: AC-003 (pipelined two traversal requests) |
| Exactly 4 path-traversal patterns; no backslash variant | BC-2.06.005 invariant 1 | Code review: confirm pattern list matches BC exactly |
| Exactly 10 web-shell patterns | BC-2.06.006 precondition 2 | Code review: count shell_patterns array |
| All URI comparisons are case-insensitive via lowercasing | BC-2.06.006 invariant 1, BC-2.06.007 invariant 1 | Unit test: AC-005, AC-007 |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust std | 2024 edition (stable) | String::to_lowercase, str::contains |
| (Finding type from findings.rs) | internal | Structured finding emission |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/http.rs | modify | Add/extend check_request_detections: path-traversal (186-203), web-shell (206-233), admin-panel (235-249) detection blocks |
| tests/http_analyzer_tests.rs | modify | Add tests: test_BC_2_06_005_path_traversal_all_fields, test_BC_2_06_005_encoded_traversal_four_patterns, test_BC_2_06_006_webshell_path_all_fields, test_BC_2_06_007_admin_panel_all_fields, test_BC_2_06_012_normal_request_zero_findings, test_BC_2_06_012_normal_request_no_parse_errors |

## Changelog

| Version | Date | Notes |
|---------|------|-------|
| v1.0 | 2026-05-21 | Initial story decomposition |
| v1.1 | 2026-05-21 | Pass-1/2 adversarial convergence; test citations added |
| v1.2 | 2026-05-28 | Sibling-sweep input-hash recomputation (DF-SIBLING-SWEEP-001): BC-2.06.005 bumped v1.3→v1.4 and BC-2.06.007 bumped v1.2→v1.3 by PO this burst — input-hash recomputed: `86f7fe0` → `7f9b0ab` (sha256 over sorted cited-BC files, first 7 chars). No AC citation changes required. |
| v1.3 | 2026-05-28 | Wave-16 Pass-4 sibling-sweep input-hash propagation (DF-SIBLING-SWEEP-001): BC-2.06.005 bumped v1.4→v1.5 by PO this burst — input-hash recomputed: `7f9b0ab` → `60e0389` (sha256 over sorted cited-BC files BC-2.06.005/006/007/012, first 7 chars). No AC citation changes required. |
| v1.4 | 2026-05-28 | DF-SIBLING-SWEEP-001 v4 propagation — BC-2.06.005 v1.6 (anchor 186-202→186-203 / 192-203), BC-2.06.006 v1.3 (line-precise invariant-1 anchor prose), BC-2.06.007 v1.4 (line-precise invariant-1 anchor prose); body propagations: Architecture Mapping row and File Structure Requirements row both updated `186-202` → `186-203`; input-hash recomputed: `60e0389` → `ea6d3cb` (sha256 over sorted cited-BC files BC-2.06.005/006/007/012, first 7 chars). |
| v1.5 | 2026-05-29 | input-hash corrected via canonical bin/compute-input-hash --update (prior value `ea6d3cb` was hand-computed sha256 over sorted inputs-file list; tool uses MD5 over inputs-order file list). New value: `9d85f8c`. |
