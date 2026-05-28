---
document_type: story
story_id: "STORY-043"
epic_id: "E-4"
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.008.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.009.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.010.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.011.md
input-hash: "2189b42"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-041]
blocks: [STORY-046]
behavioral_contracts:
  - BC-2.06.008
  - BC-2.06.009
  - BC-2.06.010
  - BC-2.06.011
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

> **Execute:** `/vsdd-factory:deliver-story STORY-043`

# STORY-043: Header and Method Anomaly Detections — Method, Host, URI Length, User-Agent

## Narrative
- **As a** forensic analyst
- **I want to** receive structured findings when HTTP requests use unusual methods (CONNECT/TRACE/DELETE/OPTIONS), violate the HTTP/1.1 Host header requirement, present abnormally long URIs, or carry an empty User-Agent
- **So that** I can identify protocol-abuse, evasion, and attack-tooling signatures in pcap traffic

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.06.008 | Unusual HTTP Methods Emit Reconnaissance/Inconclusive/Medium Finding |
| BC-2.06.009 | HTTP/1.1 Missing or Empty Host Emits Anomaly/Inconclusive/Medium Finding |
| BC-2.06.010 | URI Greater Than 2048 Chars Emits Execution/Likely/Medium Finding |
| BC-2.06.011 | Empty UA Emits Anomaly/Inconclusive/Low; Absent UA Does NOT |

## Acceptance Criteria

### AC-001 (traces to BC-2.06.008 postcondition 1)
When `parsed.method` is exactly one of "CONNECT", "TRACE", "DELETE", or "OPTIONS", a Finding is emitted with category=Reconnaissance, verdict=Inconclusive, confidence=Medium, mitre_technique=None, summary="Unusual HTTP method: <method>", evidence=vec!["<method> <uri>"], and direction=Some(Direction::ClientToServer).
- **Test:** `test_detect_unusual_method`

### AC-002 (traces to BC-2.06.008 invariant 1-2)
Method matching is an exact slice comparison (`unusual_methods.contains(&parsed.method.as_str())`) and is case-sensitive. "delete" (lowercase) does NOT match "DELETE". Standard methods GET, POST, PUT, PATCH, HEAD do not trigger this detection.
- **Test:** `test_unusual_method_case_sensitive`

### AC-003 (traces to BC-2.06.009 postcondition 1)
For HTTP/1.1 requests (`parsed.version == 1`) where `host == None`, a Finding is emitted with category=Anomaly, verdict=Inconclusive, confidence=Medium, mitre_technique=None, and summary="HTTP/1.1 request without Host header".
- **Test:** `test_detect_missing_host_header`

### AC-004 (traces to BC-2.06.009 postcondition 1)
For HTTP/1.1 requests (`parsed.version == 1`) where `host == Some("")` (present but empty after trim), a Finding is emitted with summary="HTTP/1.1 request with empty Host header". The two cases produce distinct summary text.
- **Test:** `test_detect_empty_host_header`

### AC-005 (traces to BC-2.06.009 postcondition 3)
HTTP/1.0 requests (`parsed.version == 0`) are completely exempt from the Host check — neither absent nor empty Host triggers the finding. `Host:   ` (whitespace-only) produces `Some("")` after trim and triggers the finding for HTTP/1.1 only.
- **Test:** `test_http10_no_host_finding`

### AC-006 (traces to BC-2.06.010 postcondition 1)
When `parsed.uri.len() > 2048`, a Finding is emitted with category=Execution, verdict=Likely, confidence=Medium, mitre_technique=None, summary="Abnormally long URI (<N> chars)" where N=uri.len(), and evidence=vec!["URI prefix: <truncate_uri(uri, 200)>"].
- **Test:** `test_detect_long_uri`

### AC-007 (traces to BC-2.06.010 invariant 1-3)
The long-URI threshold is strictly greater-than: `uri.len() == 2048` does NOT fire; `uri.len() == 2049` does. The summary includes the exact byte count (not the truncated length). Evidence is truncated to 200 characters via `truncate_uri`.
- **Test:** `test_long_uri_boundary_exactly_2048`

### AC-008 (traces to BC-2.06.011 postcondition 1)
When `parsed.user_agent == Some("")` (header present, value empty after trim), a Finding is emitted with category=Anomaly, verdict=Inconclusive, confidence=Low, mitre_technique=None, and summary="Empty User-Agent header".
- **Test:** `test_detect_empty_user_agent`

### AC-009 (traces to BC-2.06.011 postcondition 2 and invariant 2)
When `parsed.user_agent == None` (header absent), NO finding is emitted. This asymmetry is intentional per the Kheir 2015 rationale (absent UA is common for cron jobs; empty UA is a stronger malware signal).
- **Test:** `test_missing_user_agent_no_finding`

### AC-010 (traces to BC-2.06.011 invariant 1)
`find_header` returns `Some("")` for `User-Agent: \r\n` (whitespace-only value after trim), triggering the empty-UA finding. `User-Agent:   ` (spaces only) also produces `Some("")` after trim.
- **Test:** `test_whitespace_user_agent_triggers_empty_ua_finding`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| unusual method detection | src/analyzer/http.rs:251-265 | effectful-shell |
| host anomaly detection | src/analyzer/http.rs:283-302 | effectful-shell |
| long URI detection | src/analyzer/http.rs:304-317 | effectful-shell |
| empty UA detection | src/analyzer/http.rs:344-356 | effectful-shell |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Method = "DELETE" | Finding emitted |
| EC-002 | Method = "OPTIONS" | Finding emitted |
| EC-003 | Method = "get" (lowercase) | No finding (case-sensitive) |
| EC-004 | HTTP/1.1 with no Host header | Finding: "without Host header" |
| EC-005 | HTTP/1.1 with Host: (empty) | Finding: "with empty Host header" |
| EC-006 | HTTP/1.1 with Host:   (whitespace only) | Finding: "with empty Host header" (trim produces "") |
| EC-007 | HTTP/1.0 with no Host | No finding (version==0 exempt) |
| EC-008 | URI length = 2048 | No finding |
| EC-009 | URI length = 2049 | Finding emitted |
| EC-010 | URI length = 10000 | Finding emitted; evidence shows first 200 chars |
| EC-011 | User-Agent: (empty) | Finding emitted |
| EC-012 | No User-Agent header | No finding |
| EC-013 | Empty UA + missing Host (HTTP/1.1) | Both findings emitted independently |
| EC-014 | Long URI + path traversal | Both findings emitted independently |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/analyzer/http.rs (method/host/uri/ua detections) | effectful-shell | All push to all_findings; read self state |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| Referenced code (http.rs:251-356) | ~4,000 |
| Test files (http_analyzer_tests.rs) | ~3,000 |
| BC files (4 BCs) | ~4,000 |
| Tool outputs overhead | ~2,000 |
| **Total** | **~16,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~8%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-010 (test-writer)
2. [ ] Verify Red Gate: all tests fail before implementation
3. [ ] Implement unusual-method detection per BC-2.06.008 (exact slice match; CONNECT/TRACE/DELETE/OPTIONS; mitre=None)
4. [ ] Implement Host anomaly detection per BC-2.06.009 (3-way match on None/Some("")/Some(_); HTTP/1.1 only; distinct summary text)
5. [ ] Implement long-URI detection per BC-2.06.010 (threshold > 2048; summary with exact byte count; evidence truncated to 200)
6. [ ] Implement empty-UA detection per BC-2.06.011 (only Some("") fires; None does not fire; Kheir asymmetry)
7. [ ] Confirm all four detections fire independently when multiple anomalies co-occur on the same request
8. [ ] Run all tests; verify all pass
9. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-041 | `find_header` returns None for absent headers, Some("") for present-but-empty-after-trim | Host check must use `as_deref()` to pattern-match on &str; version field is 0 for HTTP/1.0, 1 for HTTP/1.1 | `truncate_uri` is UTF-8 char-boundary safe — use it for all URI truncation |
| STORY-042 | All detections are per-request, not per-flow-once | Summary strings use `format!` with truncated URI; evidence uses full raw URI | Detections fire even when the request also triggers other findings |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| HTTP/1.0 (version==0) is exempt from Host header check | BC-2.06.009 postcondition 3, invariant 1 | Unit test: AC-005 |
| Empty UA fires; absent UA does NOT fire | BC-2.06.011 invariant 2-4 | Unit test: AC-009 |
| Long URI threshold is strictly greater-than 2048 (not >=) | BC-2.06.010 invariant 1 | Unit test: AC-007 |
| mitre_technique is None for unusual-method and host and UA findings | BC-2.06.008/009/011 | Code review + unit test |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust std | 2024 edition (stable) | String operations, slice contains |
| (Finding type from findings.rs) | internal | Structured finding emission |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/http.rs | modify | Add unusual-method (251-265), host anomaly (283-302), long-URI (304-317), empty-UA (344-356) detection blocks |
| tests/http_analyzer_tests.rs | modify | Add: test_detect_unusual_method, test_detect_missing_host_header, test_detect_long_uri, test_detect_empty_user_agent, test_missing_user_agent_no_finding |
