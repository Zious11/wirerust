---
document_type: story
story_id: "STORY-044"
epic_id: "E-4"
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.013.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.014.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.015.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.016.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.017.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.018.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.020.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-041]
blocks: [STORY-045, STORY-046]
behavioral_contracts:
  - BC-2.06.013
  - BC-2.06.014
  - BC-2.06.015
  - BC-2.06.016
  - BC-2.06.017
  - BC-2.06.018
  - BC-2.06.020
verification_properties: []
priority: "P0"
cycle: v0.1.0-brownfield
wave: null
target_module: src/analyzer/http.rs
subsystems: [SS-06]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield
---

> **Execute:** `/vsdd-factory:deliver-story STORY-044`

# STORY-044: Parse-Error Isolation and Poisoning State Machine

## Narrative
- **As a** forensic analyst
- **I want to** have the HTTP analyzer correctly isolate parse errors per flow and per direction — counting consecutive errors, poisoning directions that exceed the threshold (3 errors), and silently absorbing subsequent bytes — so that non-HTTP traffic on HTTP-dispatched flows does not contaminate findings or cause false positives in other flows

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.06.013 | Non-HTTP Bytes Increment parse_errors; No Token-Error Findings |
| BC-2.06.014 | Too Many Headers Emits Anomaly/Inconclusive/Medium Finding (T1499.002) |
| BC-2.06.015 | After 3 Consecutive Parse Errors a Direction is Poisoned; Subsequent Bytes Skipped |
| BC-2.06.016 | Single Parse Error Does NOT Poison |
| BC-2.06.017 | Poisoning is Per-Direction; Poisoned Request Does Not Affect Response |
| BC-2.06.018 | non_http_flows Counts Flow Once Even if Both Directions Poisoned |
| BC-2.06.020 | HTTP Body Bytes After Header Completion Do Not Inflate parse_errors |

## Acceptance Criteria

### AC-001 (traces to BC-2.06.013 postcondition 1-5)
When httparse returns an Err (not TooManyHeaders) and `had_success == false`, `parse_errors` is incremented by 1, `request_error_count` is incremented by 1, `request_buf` is cleared, no finding is pushed to `all_findings`, and `try_parse_requests` returns early.
- **Test:** `test_parse_error_increments_counter`

### AC-002 (traces to BC-2.06.013 invariant 1)
`had_success` suppresses error counting for body bytes that follow a complete header. Body bytes after a parsed header do NOT increment `parse_errors`. `TooManyHeaders` is the only Err variant that also emits a finding.
- **Test:** `test_parse_error_in_response`

### AC-003 (traces to BC-2.06.014 postcondition 1-5)
When httparse returns `Err(httparse::Error::TooManyHeaders)` and `had_success == false`, a Finding is emitted with category=Anomaly, verdict=Inconclusive, confidence=Medium, mitre_technique=Some("T1499.002"), summary="Excessive HTTP headers exceeded parser limit (possible DoS or header-based attack)", and evidence containing "Direction: request" or "Direction: response" as a plain string (not a Direction enum).
- **Test:** `test_too_many_headers_generates_finding`

### AC-004 (traces to BC-2.06.014 postcondition 2-4)
The TooManyHeaders finding also increments `parse_errors` by 1 and `request_error_count` by 1, clears the buffer, and returns early — the usual error-count path is NOT bypassed.
- **Test:** `test_too_many_headers_in_response_generates_finding`

### AC-005 (traces to BC-2.06.014 invariant 4)
The TooManyHeaders finding evidence text is "Direction: request" or "Direction: response" — a plain hardcoded string, not derived from the Direction enum.
- **Test:** `test_too_many_headers_generates_finding`

### AC-006 (traces to BC-2.06.015 postcondition 1-4)
When `request_error_count >= POISON_THRESHOLD (3)`, `HttpFlowState.request_poisoned` is set to `true`, `non_http_flows` is incremented by 1 (if `counted_as_non_http` is false), the direction buffer is cleared, and all subsequent `on_data` calls for that direction count bytes in `poisoned_bytes_skipped` without parsing.
- **Test:** `test_parse_error_poisons_direction_after_threshold`

### AC-007 (traces to BC-2.06.015 invariant 1-3)
Poisoning is per-direction: `request_poisoned` and `response_poisoned` are independent booleans. The error counter is CONSECUTIVE, not cumulative — one successful parse resets the counter to 0. Poisoning is irreversible within a flow lifetime.
- **Test:** `test_poison_is_per_direction`

### AC-008 (traces to BC-2.06.016 postcondition 1-5)
A single parse error increments `request_error_count` to 1 and `parse_errors` to 1, but does NOT trigger poisoning. A subsequent valid HTTP request is parsed normally.
- **Test:** `test_single_error_does_not_poison`

### AC-009 (traces to BC-2.06.016 invariant 2)
`request_error_count` resets to 0 on a successful parse. One successful parse after two consecutive errors returns the counter to 0, so the flow requires 3 NEW consecutive errors to be poisoned.
- **Test:** `test_error_count_resets_on_success`

### AC-010 (traces to BC-2.06.017 postcondition 1-3)
When the request direction is poisoned, subsequent `on_data` calls with `Direction::ServerToClient` continue to parse responses normally. `response_error_count` is independent of `request_error_count`. `response_poisoned` remains false until its own threshold is reached independently.
- **Test:** `test_poison_request_does_not_affect_response`

### AC-011 (traces to BC-2.06.018 postcondition 1-3)
`non_http_flows` is incremented by 1 and `counted_as_non_http` is set to true when the first direction of a flow is poisoned. If the second direction subsequently reaches the poison threshold, `non_http_flows` is NOT incremented again because `counted_as_non_http` is already true.
- **Test:** `test_non_http_flows_counts_per_flow_not_direction`

### AC-012 (traces to BC-2.06.020 postcondition 1-4)
After a complete HTTP request header is successfully parsed (`had_success = true`), remaining body bytes in `request_buf` that fail parsing do NOT increment `parse_errors` or `request_error_count`. The buffer is cleared and the loop exits.
- **Test:** `test_body_bytes_do_not_inflate_parse_errors`

### AC-013 (traces to BC-2.06.020 invariant 3)
The TooManyHeaders finding check is inside the `if !had_success` block — TooManyHeaders on body bytes after a successful header is also suppressed.
- **Test:** `test_body_bytes_do_not_inflate_parse_errors`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| try_parse_requests Err arm | src/analyzer/http.rs:403-434 | effectful-shell |
| TooManyHeaders detection | src/analyzer/http.rs:416-428 (req), 475-487 (resp) | effectful-shell |
| Poison transition | src/analyzer/http.rs:408-409 (req), 467 (resp) | effectful-shell |
| counted_as_non_http latch | src/analyzer/http.rs:410-413 | effectful-shell |
| had_success guard | src/analyzer/http.rs:362-364, 403-408 | effectful-shell |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Binary garbage bytes | parse_errors=1; no finding |
| EC-002 | SSH protocol bytes (non-HTTP) | parse_errors=1; no finding |
| EC-003 | HTTP body bytes after complete header | NOT counted as error (had_success=true suppresses) |
| EC-004 | TooManyHeaders error | parse_errors=1 AND finding emitted |
| EC-005 | Three consecutive non-HTTP buffers | parse_errors=3; direction poisoned |
| EC-006 | 2 errors, then 1 success, then 2 more errors | NOT poisoned; count resets; need 3 more to poison |
| EC-007 | Request poisoned; response receives valid HTTP | Response parsed normally |
| EC-008 | Both directions poisoned | non_http_flows=1 (not 2) |
| EC-009 | Two separate flows, each poisoned | non_http_flows=2 |
| EC-010 | TooManyHeaders on 3rd consecutive attempt | Third error triggers poisoning AND emits a finding |
| EC-011 | TooManyHeaders after had_success=true | Finding NOT emitted (suppressed by had_success guard) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/analyzer/http.rs (error/poison path) | effectful-shell | Mutates parse_errors, error_count, poisoned flags, all_findings |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~4,000 |
| Referenced code (http.rs:362-434, 467-487, 509-523) | ~5,000 |
| Test files (http_analyzer_tests.rs) | ~4,000 |
| BC files (7 BCs) | ~7,000 |
| Tool outputs overhead | ~2,000 |
| **Total** | **~22,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~11%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-013 (test-writer)
2. [ ] Verify Red Gate: all tests fail before implementation
3. [ ] Implement parse-error counting (non-TooManyHeaders) per BC-2.06.013 (no finding; parse_errors++; error_count++; buf clear; early return)
4. [ ] Implement TooManyHeaders detection per BC-2.06.014 (finding + counter increment; direction evidence as plain string)
5. [ ] Implement poison transition per BC-2.06.015 (`POISON_THRESHOLD=3`; monotonic false->true; non_http_flows with counted_as_non_http latch)
6. [ ] Verify single-error non-poison per BC-2.06.016
7. [ ] Verify per-direction independence per BC-2.06.017
8. [ ] Implement non_http_flows latch per BC-2.06.018 (counted_as_non_http prevents double-count)
9. [ ] Implement had_success suppression per BC-2.06.020 (body bytes after success are silently discarded)
10. [ ] Run all tests; verify all pass
11. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-041 | `had_success` is a local bool per `try_parse_requests` invocation, initialized to false | Both request and response paths use symmetric loop structure | `request_error_count` is a u8 with saturating increment; ensure `>= POISON_THRESHOLD` not `== POISON_THRESHOLD` |
| STORY-042 | Findings push to `all_findings`; no finding is emitted for plain parse errors | TooManyHeaders is the ONLY parse error that emits a finding | `POISON_THRESHOLD = 3` is defined at `http.rs:80` as a const |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `POISON_THRESHOLD = 3` is a named constant, not a magic number | BC-2.06.015 | Code review: confirm const at http.rs:80 |
| Poisoning is monotonic: request_poisoned can only transition false->true | BC-2.06.015 invariant 3 / INV-8 | Code review: confirm no `= false` assignment after initial construction |
| `counted_as_non_http` latch prevents double-increment of non_http_flows | BC-2.06.018 invariant 3 | Unit test: AC-011 |
| `had_success` guard wraps ALL error-counting code, including TooManyHeaders finding | BC-2.06.020 invariant 3 | Code review: confirm TooManyHeaders check is inside `if !had_success` block |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| httparse | (as pinned in Cargo.toml) | `httparse::Error::TooManyHeaders` variant |
| Rust std | 2024 edition (stable) | u8 saturating arithmetic |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/http.rs | modify | Err arm (403-434 request, 475-487 response): had_success guard, TooManyHeaders finding, error_count++, poison transition, non_http_flows latch |
| tests/http_analyzer_tests.rs | modify | Add: test_parse_error_increments_counter, test_too_many_headers_generates_finding, test_parse_error_poisons_direction_after_threshold, test_single_error_does_not_poison, test_poison_request_does_not_affect_response, test_non_http_flows_counts_per_flow_not_direction, test_body_bytes_do_not_inflate_parse_errors |
