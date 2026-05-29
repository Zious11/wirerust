---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-041.md
  - .factory/stories/STORY-042.md
  - .factory/stories/STORY-043.md
  - .factory/stories/STORY-044.md
  - .factory/stories/STORY-045.md
  - .factory/stories/STORY-046.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.013.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.015.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.016.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.017.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.018.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.020.md
input-hash: "d270d81"
traces_to: .factory/stories/STORY-041.md
id: "HS-054"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-4"
behavioral_contracts:
  - BC-2.06.013
  - BC-2.06.015
  - BC-2.06.016
  - BC-2.06.017
  - BC-2.06.018
  - BC-2.06.020
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: HTTP Poisoning Is Per-Direction and Counted Once Per Flow

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains two TCP flows. Flow A is a TLS tunnel where the HTTP analyzer receives binary TLS records, causing parse errors. Flow B is genuine HTTP/1.1 traffic.
2. Flow A's request direction receives three consecutive binary blobs (non-HTTP data). After these arrive, no further request-direction data arrives for flow A.
3. Flow A's response direction receives valid HTTP/1.1 response headers successfully after the request direction has been poisoned.
4. Flow B operates independently with well-formed HTTP requests and responses.
5. The analyst runs wirerust on this pcap.
6. Expected outcomes: `non_http_flows` in the HTTP summary equals 1 (flow A, counted once despite two possible poison events). Flow B's HTTP statistics are unaffected by flow A's errors. The HTTP summary `parse_errors` reflects flow A's 3+ errors but flow B's `transactions` and method counts are correct. No finding is emitted for plain parse errors (binary data, non-TooManyHeaders).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.06.013 | postcondition 1-5; invariant 1 | Binary non-HTTP bytes increment parse_errors with no finding |
| BC-2.06.015 | postcondition 1-4; invariant 1-3 | Three consecutive errors trigger poisoning; monotonic transition |
| BC-2.06.016 | postcondition 1-5 | Single error does not poison; contrast against flow B |
| BC-2.06.017 | postcondition 1-3 | Poisoned request direction does not affect response direction of same flow |
| BC-2.06.018 | postcondition 1-3; invariant 3 | non_http_flows counted once per flow regardless of how many directions are poisoned |
| BC-2.06.020 | postcondition 1-4 | HTTP body bytes after a successful header are silently dropped, not counted as errors |

## Verification Approach

Craft a pcap with flow A (binary data followed by a valid HTTP response) and flow B (clean HTTP request+response). Run wirerust with JSON output.

1. Assert `analyzers[HTTP].detail.non_http_flows == "1"` (flow A, not 2).
2. Assert `analyzers[HTTP].detail.parse_errors` is at least 3 (from flow A request direction).
3. Assert flow B contributes `transactions >= 1` (response from flow B counted).
4. Assert `findings` array contains NO finding with category="Anomaly" from plain binary parse errors (only TooManyHeaders would emit a finding, and that is not present here).
5. Assert `analyzers[HTTP].detail.poisoned_bytes_skipped` is greater than 0 if additional binary bytes were sent after poisoning.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): non_http_flows=1; parse_errors >= 3; response from clean flow counted; no spurious findings.
- **Edge case handling** (weight: 0.3): Response direction unaffected after request direction poisoned in same flow; flow B fully isolated.
- **Error quality** (weight: 0.15): No findings emitted for plain binary parse errors (only TooManyHeaders variety generates a finding).
- **Data integrity** (weight: 0.1): poisoned_bytes_skipped key present in summary even if zero.

## Edge Conditions

- One successful parse on the response direction of flow A after request direction is poisoned — the response direction must work normally.
- cross-flow isolation: flow B must have zero errors and correct stats regardless of what flow A does.
- The `non_http_flows` counter is a global aggregate, not per-flow — it should be exactly 1 in a 2-flow pcap where only one flow is poisoned.

## Failure Guidance

"HOLDOUT LOW: HS-054 (satisfaction: 0.XX) -- HTTP poisoning state machine miscounted non_http_flows, allowed cross-flow error contamination, or emitted findings for plain binary parse errors without TooManyHeaders."
