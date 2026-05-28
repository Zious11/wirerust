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
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.014.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.016.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.020.md
input-hash: "a6a15cb"
traces_to: .factory/stories/STORY-041.md
id: "HS-065"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-4"
behavioral_contracts:
  - BC-2.06.014
  - BC-2.06.016
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

# Holdout Scenario: TooManyHeaders Emits Exactly One Finding and Contributes to Poison Counter

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains an HTTP flow where the request direction sends three consecutive HTTP requests, each with more than 96 headers (exceeding the httparse MAX_HEADERS limit of 96).
2. Each oversized request triggers a `TooManyHeaders` parse error.
3. After the third such request, the request direction is poisoned (3 consecutive errors).
4. The response direction of the same flow receives valid HTTP responses and continues to parse normally.
5. The analyst runs wirerust on this pcap.
6. Expected outcomes: exactly 3 findings with MITRE technique T1499.002 (one per TooManyHeaders error). `non_http_flows == 1` (the flow is counted once as non-HTTP after poisoning). `parse_errors == 3`. The response direction is unaffected.
7. A subsequent HTTP body (bytes after a complete header) encountering TooManyHeaders is NOT counted if `had_success` is already true for that iteration — the `had_success` guard wraps the finding emission.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.06.014 | postcondition 1-5; invariant 4 | TooManyHeaders emits Anomaly/Inconclusive/Medium T1499.002 finding; plain string evidence |
| BC-2.06.016 | postcondition 1-5; invariant 2 | Single error does not poison; need 3 consecutive errors; reset on success |
| BC-2.06.020 | postcondition 1-4; invariant 3 | had_success guard: body bytes after successful header do not count as TooManyHeaders error |

## Verification Approach

Craft a pcap with an HTTP flow where the request direction sends three back-to-back 97-header requests. Run wirerust.

1. Assert `findings` contains exactly 3 entries with `mitre_technique == "T1499.002"`.
2. Assert each finding has `category == "Anomaly"`, `verdict == "Inconclusive"`, `confidence == "Medium"`.
3. Assert each finding's `evidence` contains `"Direction: request"` as a plain string.
4. Assert `analyzers[HTTP].detail.parse_errors >= 3`.
5. Assert `analyzers[HTTP].detail.non_http_flows == "1"` (flow counted once after third error triggers poisoning).
6. Assert `analyzers[HTTP].detail.transactions` reflects any valid responses from the response direction.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Exactly 3 T1499.002 findings; parse_errors counts include TooManyHeaders; non_http_flows=1 after poisoning.
- **Edge case handling** (weight: 0.3): had_success guard prevents TooManyHeaders from firing on body bytes after a successful header; response direction unaffected.
- **Error quality** (weight: 0.15): Evidence text is hardcoded plain string "Direction: request" (not enum Debug format); MITRE code is T1499.002.
- **Data integrity** (weight: 0.1): parse_errors and non_http_flows are both present in the summary detail map.

## Edge Conditions

- TooManyHeaders on the 3rd consecutive error: the finding fires AND poisoning triggers simultaneously on the same error.
- Body bytes after a successful header: if httparse returns TooManyHeaders while `had_success == true`, NO finding is emitted and error count is NOT incremented.
- Response direction of the poisoned flow: must continue to parse normally and count transactions.

## Failure Guidance

"HOLDOUT LOW: HS-065 (satisfaction: 0.XX) -- TooManyHeaders handling was incorrect; verify exactly one T1499.002 finding per TooManyHeaders error, had_success guard prevents body-byte false positives, and poisoning triggers after 3 consecutive errors."
