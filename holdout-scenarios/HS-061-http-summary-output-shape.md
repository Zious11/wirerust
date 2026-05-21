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
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.023.md
input-hash: "5db4ba5"
traces_to: .factory/stories/STORY-041.md
id: "HS-061"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-4"
behavioral_contracts:
  - BC-2.06.023
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: HTTP Analyzer Summary Is Complete, Deterministic, and Reflects Response-Only Transaction Count

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains HTTP traffic with: 5 GET requests, 3 POST requests, 3 complete HTTP responses (200, 404, 200), 25 distinct host values, and 22 distinct URIs.
2. The analyst runs wirerust on this pcap with JSON output.
3. The `analyzers` array contains an HTTP entry. Its `detail` BTreeMap is inspected.
4. The analyst verifies:
   - Exactly 9 keys in the detail map (alphabetical order).
   - `transactions` equals the number of parsed RESPONSES (3), not the number of REQUESTS (8).
   - `top_hosts` contains exactly 20 entries (truncated from 25), sorted by count descending.
   - `recent_uris` contains the first 20 URIs in insertion order (not last 20, not sorted).
   - `methods` has two keys: "GET" with value 5 and "POST" with value 3.
   - `status_codes` has two keys: "200" with value 2 and "404" with value 1 (stringified u16 keys).
5. Running `summarize()` twice on the same analyzer produces identical output (determinism).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.06.023 | postcondition 1-3; invariant 1-4 | Complete detail map; exact key set; transactions=response count; top_hosts sorted; recent_uris first 20; deterministic; read-only |

## Verification Approach

Run wirerust on a pcap matching the described traffic. Parse JSON output.

1. Navigate to `analyzers[].detail` where `analyzer_name == "HTTP"`.
2. Assert `Object.keys(detail).sort()` equals `["methods","non_http_flows","parse_errors","poisoned_bytes_skipped","recent_uris","status_codes","top_hosts","transactions","user_agents"]` (9 keys, alphabetical).
3. Assert `detail.transactions == 3` (response count).
4. Assert `detail.top_hosts` is an array or object with at most 20 entries, sorted by count descending.
5. Assert `detail.recent_uris` is an array of at most 20 items in insertion order.
6. Assert `detail.status_codes["200"] == 2` and `detail.status_codes["404"] == 1` (string keys, not integer keys).
7. Assert `detail.methods["GET"] == 5` and `detail.methods["POST"] == 3`.

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): Exactly 9 keys; transactions reflects response count; method and status counts correct.
- **Edge case handling** (weight: 0.3): top_hosts truncated to 20 from 25; recent_uris is first 20 not last 20.
- **Error quality** (weight: 0.2): status_codes keys are string-typed, not integer-typed; BTreeMap ordering is alphabetical.
- **Data integrity** (weight: 0.1): summarize() is read-only; calling it multiple times produces identical output.

## Edge Conditions

- 22 URIs with MAX_URIS=10,000 — all 22 fit; recent_uris shows first 20 (the insertion-order first 20 of 22).
- 25 distinct hosts — top_hosts shows only the 20 most frequent.
- Status code 0 (from httparse returning None for code): stored as `"0"` key in status_codes.

## Failure Guidance

"HOLDOUT LOW: HS-061 (satisfaction: 0.XX) -- HTTP summarize output was wrong; check that transactions counts responses (not requests), that top_hosts is truncated to 20 and sorted descending, and that status_codes keys are strings not integers."
