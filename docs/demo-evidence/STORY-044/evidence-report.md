# Demo Evidence Report — STORY-044

**Story:** STORY-044 — Parse-Error Isolation and Poisoning State Machine
**Story ID (frontmatter):** `STORY-044`
**Date Recorded:** 2026-05-28
**Branch:** `feature/STORY-044`
**Implementation Strategy:** brownfield-formalization (no runtime change; deliverable is `bc_2_06_044_formalization` test module)
**Toolchain:** VHS (Menlo font, Dracula theme, 1200px wide)

## Summary

All 13 acceptance criteria are covered by the `bc_2_06_044_formalization` module in
`tests/http_analyzer_tests.rs`. The overview recording shows all 29 formalization tests
passing in a single run. Each AC below links to the specific tape and the specific named
test(s) mandated by the story.

---

## Overview Recording

| Artifact | Description |
|----------|-------------|
| `AC-ALL-formalization-module.gif` | Full `bc_2_06_044_formalization` module — 29 tests, all pass |
| `AC-ALL-formalization-module.webm` | Same (archival) |
| `AC-ALL-formalization-module.tape` | VHS source |

---

## AC-by-AC Coverage

### AC-001 — parse_errors incremented, no finding on non-TooManyHeaders error

**BC:** BC-2.06.013 postcondition 1-5
**Named test:** `test_parse_error_increments_counter`

| Artifact | Description |
|----------|-------------|
| `AC-001-parse-error-increments-counter.gif` | Test passes; parse_errors=1, no finding |
| `AC-001-parse-error-increments-counter.webm` | Same (archival) |
| `AC-001-parse-error-increments-counter.tape` | VHS source |

---

### AC-002 — had_success suppresses body-byte error counting

**BC:** BC-2.06.013 invariant 1
**Named test:** `test_BC_2_06_013_invariant_had_success_suppresses_body_byte_errors`

| Artifact | Description |
|----------|-------------|
| `AC-002-had-success-suppresses-body-byte-errors.gif` | Body bytes after parsed header do NOT increment parse_errors |
| `AC-002-had-success-suppresses-body-byte-errors.webm` | Same (archival) |
| `AC-002-had-success-suppresses-body-byte-errors.tape` | VHS source |

---

### AC-003 + AC-005 — TooManyHeaders emits Anomaly/Inconclusive/Medium finding with plain-string evidence

**BC:** BC-2.06.014 postcondition 1-5, invariant 4
**Named test:** `test_too_many_headers_generates_finding`

| Artifact | Description |
|----------|-------------|
| `AC-003-005-too-many-headers-finding.gif` | Finding: category=Anomaly, verdict=Inconclusive, confidence=Medium, mitre=T1499.002 |
| `AC-003-005-too-many-headers-finding.webm` | Same (archival) |
| `AC-003-005-too-many-headers-finding.tape` | VHS source |

**Notes:** AC-005 (evidence is plain string "Direction: request", not enum-derived) is verified
by the same test `test_too_many_headers_generates_finding` which asserts the evidence string
directly. Also covered by `test_BC_2_06_014_invariant_evidence_is_plain_string_not_enum_derived`
in the formalization module (visible in `AC-ALL-formalization-module.gif`).

---

### AC-004 — TooManyHeaders on response arm also emits finding, increments counters

**BC:** BC-2.06.014 postcondition 2-4 (direction-symmetric)
**Named test:** `test_too_many_headers_in_response_generates_finding`

| Artifact | Description |
|----------|-------------|
| `AC-004-too-many-headers-response-arm.gif` | Response direction: finding emitted, response_error_count=1, parse_errors=1 |
| `AC-004-too-many-headers-response-arm.webm` | Same (archival) |
| `AC-004-too-many-headers-response-arm.tape` | VHS source |

---

### AC-006 — 3 consecutive parse errors poison direction; subsequent bytes skipped

**BC:** BC-2.06.015 postcondition 1-4
**Named tests (story):** `test_parse_error_poisons_direction_after_threshold`
**Also in formalization:** `test_BC_2_06_015_three_consecutive_errors_trigger_poisoning`, `test_BC_2_06_015_non_http_flows_incremented_on_first_poison`

| Artifact | Description |
|----------|-------------|
| `AC-006-007-poison-after-threshold.gif` | request_poisoned=true after 3 errors; non_http_flows=1; subsequent bytes counted in poisoned_bytes_skipped |
| `AC-006-007-poison-after-threshold.webm` | Same (archival) |
| `AC-006-007-poison-after-threshold.tape` | VHS source |

---

### AC-007 — Poisoning is per-direction; irreversible

**BC:** BC-2.06.015 invariant 1-3
**Named test:** `test_poison_is_per_direction`
**Also in formalization:** `test_BC_2_06_015_invariant_poisoning_is_irreversible`

| Artifact | Description |
|----------|-------------|
| `AC-006-007-poison-after-threshold.gif` | Same recording covers both AC-006 and AC-007 (test_poison_is_per_direction run together) |
| `AC-006-007-poison-after-threshold.webm` | Same (archival) |
| `AC-006-007-poison-after-threshold.tape` | VHS source |

---

### AC-008 — Single parse error does NOT poison; next valid parse succeeds

**BC:** BC-2.06.016 postcondition 1-5
**Named test:** `test_single_error_does_not_poison`

| Artifact | Description |
|----------|-------------|
| `AC-008-009-single-error-no-poison-reset.gif` | error_count=1, request_poisoned=false; subsequent valid request parsed |
| `AC-008-009-single-error-no-poison-reset.webm` | Same (archival) |
| `AC-008-009-single-error-no-poison-reset.tape` | VHS source |

---

### AC-009 — error_count resets to 0 on successful parse

**BC:** BC-2.06.016 invariant 2
**Named test:** `test_error_count_resets_on_success`

| Artifact | Description |
|----------|-------------|
| `AC-008-009-single-error-no-poison-reset.gif` | Same recording; 2 errors then success resets count to 0 |
| `AC-008-009-single-error-no-poison-reset.webm` | Same (archival) |
| `AC-008-009-single-error-no-poison-reset.tape` | VHS source |

---

### AC-010 — Poisoned request direction does not affect response direction

**BC:** BC-2.06.017 postcondition 1-3
**Named test:** `test_poison_request_does_not_affect_response`
**Also in formalization:** `test_BC_2_06_017_poisoned_request_does_not_affect_response_parsing`

| Artifact | Description |
|----------|-------------|
| `AC-010-poison-request-does-not-affect-response.gif` | After request poisoned: response parses valid HTTP normally, response_poisoned=false |
| `AC-010-poison-request-does-not-affect-response.webm` | Same (archival) |
| `AC-010-poison-request-does-not-affect-response.tape` | VHS source |

---

### AC-011 — non_http_flows counts per-flow (both directions poisoned = count 1, not 2)

**BC:** BC-2.06.018 postcondition 1-3
**Named test:** `test_non_http_flows_counts_per_flow_not_direction`
**Also in formalization:** `test_BC_2_06_018_both_directions_poisoned_counts_one_flow_not_two`, `test_BC_2_06_018_invariant_counted_as_non_http_latch_prevents_double_count`

| Artifact | Description |
|----------|-------------|
| `AC-011-non-http-flows-counts-once.gif` | Both directions poisoned: non_http_flows=1 (not 2); counted_as_non_http latch holds |
| `AC-011-non-http-flows-counts-once.webm` | Same (archival) |
| `AC-011-non-http-flows-counts-once.tape` | VHS source |

---

### AC-012 — Body bytes after complete header do NOT inflate parse_errors

**BC:** BC-2.06.020 postcondition 1-4
**Named test:** `test_body_bytes_do_not_inflate_parse_errors`
**Also in formalization:** `test_BC_2_06_020_post_with_body_does_not_inflate_parse_errors`, `test_BC_2_06_020_response_with_body_does_not_inflate_parse_errors`

| Artifact | Description |
|----------|-------------|
| `AC-012-body-bytes-no-inflate.gif` | After header success: remaining body bytes do not increment parse_errors or error_count |
| `AC-012-body-bytes-no-inflate.webm` | Same (archival) |
| `AC-012-body-bytes-no-inflate.tape` | VHS source |

---

### AC-013 — TooManyHeaders on body bytes after had_success is suppressed

**BC:** BC-2.06.020 invariant 3
**Named test:** `test_BC_2_06_020_invariant_real_too_many_headers_after_success_suppressed`
(covers both request and response arms via sibling test `..._suppressed_response`)

| Artifact | Description |
|----------|-------------|
| `AC-013-too-many-headers-after-success-suppressed.gif` | Real TooManyHeaders input after had_success: no finding emitted, parse_errors unchanged |
| `AC-013-too-many-headers-after-success-suppressed.webm` | Same (archival) |
| `AC-013-too-many-headers-after-success-suppressed.tape` | VHS source |

---

## Formalization Module Coverage Summary

The `bc_2_06_044_formalization` module in `tests/http_analyzer_tests.rs` contains 29 tests
covering all 7 behavioral contracts (BC-2.06.013 through BC-2.06.020, excluding BC-2.06.019).

| BC | Tests in Formalization Module |
|----|-------------------------------|
| BC-2.06.013 | `test_BC_2_06_013_non_http_bytes_increment_parse_errors_no_finding`, `test_BC_2_06_013_binary_garbage_increments_parse_errors`, `test_BC_2_06_013_invariant_had_success_suppresses_body_byte_errors`, `test_BC_2_06_013_invariant_token_error_does_not_emit_finding` |
| BC-2.06.014 | `test_BC_2_06_014_too_many_headers_request_emits_anomaly_finding`, `test_BC_2_06_014_too_many_headers_response_emits_anomaly_finding`, `test_BC_2_06_014_invariant_evidence_is_plain_string_not_enum_derived`, `test_BC_2_06_014_invariant_too_many_headers_contributes_to_poison_threshold` |
| BC-2.06.015 | `test_BC_2_06_015_three_consecutive_errors_trigger_poisoning`, `test_BC_2_06_015_invariant_poisoning_is_irreversible`, `test_BC_2_06_015_invariant_error_count_is_consecutive_not_cumulative`, `test_BC_2_06_015_non_http_flows_incremented_on_first_poison` |
| BC-2.06.016 | `test_BC_2_06_016_single_error_does_not_poison_direction`, `test_BC_2_06_016_invariant_single_error_then_success_resets_count`, `test_BC_2_06_016_ec003_two_errors_success_one_error_count_one` |
| BC-2.06.017 | `test_BC_2_06_017_poisoned_request_does_not_affect_response_parsing`, `test_BC_2_06_017_invariant_request_poisoned_gates_only_client_to_server`, `test_BC_2_06_017_ec003_poisoned_response_does_not_affect_request` |
| BC-2.06.018 | `test_BC_2_06_018_both_directions_poisoned_counts_one_flow_not_two`, `test_BC_2_06_018_only_request_poisoned_counts_one_flow`, `test_BC_2_06_018_invariant_counted_as_non_http_latch_prevents_double_count`, `test_BC_2_06_018_invariant_two_separate_flows_count_two` |
| BC-2.06.020 | `test_BC_2_06_020_post_with_body_does_not_inflate_parse_errors`, `test_BC_2_06_020_invariant_had_success_is_local_per_call`, `test_BC_2_06_020_response_with_body_does_not_inflate_parse_errors`, `test_BC_2_06_020_invariant_too_many_headers_after_success_suppressed`, `test_BC_2_06_020_invariant_real_too_many_headers_after_success_suppressed`, `test_BC_2_06_020_invariant_real_too_many_headers_after_success_suppressed_response`, `test_BC_2_06_020_pre_success_errors_counted_body_errors_not` |

**Result: 29/29 passed. 0 failed.**

---

## Artifact Index

| File | Type | AC Coverage |
|------|------|-------------|
| `AC-ALL-formalization-module.gif` | GIF | ALL (overview, 29 tests) |
| `AC-ALL-formalization-module.webm` | WebM | ALL (overview, 29 tests) |
| `AC-ALL-formalization-module.tape` | VHS source | ALL |
| `AC-001-parse-error-increments-counter.gif` | GIF | AC-001 |
| `AC-001-parse-error-increments-counter.webm` | WebM | AC-001 |
| `AC-001-parse-error-increments-counter.tape` | VHS source | AC-001 |
| `AC-002-had-success-suppresses-body-byte-errors.gif` | GIF | AC-002 |
| `AC-002-had-success-suppresses-body-byte-errors.webm` | WebM | AC-002 |
| `AC-002-had-success-suppresses-body-byte-errors.tape` | VHS source | AC-002 |
| `AC-003-005-too-many-headers-finding.gif` | GIF | AC-003, AC-005 |
| `AC-003-005-too-many-headers-finding.webm` | WebM | AC-003, AC-005 |
| `AC-003-005-too-many-headers-finding.tape` | VHS source | AC-003, AC-005 |
| `AC-004-too-many-headers-response-arm.gif` | GIF | AC-004 |
| `AC-004-too-many-headers-response-arm.webm` | WebM | AC-004 |
| `AC-004-too-many-headers-response-arm.tape` | VHS source | AC-004 |
| `AC-006-007-poison-after-threshold.gif` | GIF | AC-006, AC-007 |
| `AC-006-007-poison-after-threshold.webm` | WebM | AC-006, AC-007 |
| `AC-006-007-poison-after-threshold.tape` | VHS source | AC-006, AC-007 |
| `AC-008-009-single-error-no-poison-reset.gif` | GIF | AC-008, AC-009 |
| `AC-008-009-single-error-no-poison-reset.webm` | WebM | AC-008, AC-009 |
| `AC-008-009-single-error-no-poison-reset.tape` | VHS source | AC-008, AC-009 |
| `AC-010-poison-request-does-not-affect-response.gif` | GIF | AC-010 |
| `AC-010-poison-request-does-not-affect-response.webm` | WebM | AC-010 |
| `AC-010-poison-request-does-not-affect-response.tape` | VHS source | AC-010 |
| `AC-011-non-http-flows-counts-once.gif` | GIF | AC-011 |
| `AC-011-non-http-flows-counts-once.webm` | WebM | AC-011 |
| `AC-011-non-http-flows-counts-once.tape` | VHS source | AC-011 |
| `AC-012-body-bytes-no-inflate.gif` | GIF | AC-012 |
| `AC-012-body-bytes-no-inflate.webm` | WebM | AC-012 |
| `AC-012-body-bytes-no-inflate.tape` | VHS source | AC-012 |
| `AC-013-too-many-headers-after-success-suppressed.gif` | GIF | AC-013 |
| `AC-013-too-many-headers-after-success-suppressed.webm` | WebM | AC-013 |
| `AC-013-too-many-headers-after-success-suppressed.tape` | VHS source | AC-013 |
| `evidence-report.md` | Report | All ACs |
