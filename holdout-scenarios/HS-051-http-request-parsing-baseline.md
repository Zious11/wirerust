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
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.001.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.002.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.003.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.004.md
input-hash: "5db4ba5"
traces_to: .factory/stories/STORY-041.md
id: "HS-051"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-4"
behavioral_contracts:
  - BC-2.06.001
  - BC-2.06.002
  - BC-2.06.003
  - BC-2.06.004
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: HTTP Pipelined Requests and Partial Buffering Correctness

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains a single TCP flow carrying two consecutive HTTP/1.1 GET requests sent back-to-back in a single segment, followed by a third request that is split across two segments — the second segment completing the headers.
2. The analyst runs wirerust on this pcap to obtain HTTP analysis output.
3. Each of the two complete requests produces independent statistics updates (method map, host map, URI list, request count) — the combined packet does not double-count on any single field.
4. The partial request does not produce any method, host, or URI entry after the first segment arrives; after the second segment completes the headers, all three fields are populated for that request.
5. The transaction count in the summary reflects only the number of parsed HTTP responses (which may be zero if no responses appear in this pcap), not the number of requests.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.06.001 | postcondition 1-7 | Complete request statistics correctly accumulated per request |
| BC-2.06.002 | postcondition 1-5; invariant 1 | Pipelined loop parses both complete requests independently; error count resets between them |
| BC-2.06.003 | postcondition 1-5; invariant 1 | Partial request produces no stats until fully received; partial is distinct from error |
| BC-2.06.004 | invariant 1 | transactions reflects response count only — request count does not bleed into this field |

## Verification Approach

Run wirerust with the `analyze` subcommand on a crafted pcap where:
- The TCP reassembler delivers two full `GET /a HTTP/1.1\r\nHost: h.test\r\n\r\n` requests in one on_data call to the HTTP analyzer.
- A third partial request `GET /b HTT` arrives in a subsequent on_data call, with `P/1.1\r\nHost: h.test\r\n\r\n` arriving in a third call.

Verification steps:
1. Inspect JSON output `analyzers` array for the HTTP analyzer entry.
2. Assert `methods["GET"] == 3` (all three requests parsed eventually).
3. Assert `top_hosts` contains `"h.test"` with count 3.
4. Assert `recent_uris` contains both `"/a"` (appearing twice) and `"/b"`.
5. Assert `transactions == 0` (no responses in pcap).
6. Assert `parse_errors == 0`.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Request count per method and per host equals 3; transaction count is 0; parse_errors is 0.
- **Edge case handling** (weight: 0.3): Partial buffering does not produce intermediate stats; third request counted only after both segments arrive.
- **Error quality** (weight: 0.1): No spurious error findings emitted.
- **Data integrity** (weight: 0.1): Summary BTreeMap keys present and ordered alphabetically.

## Edge Conditions

- Two complete requests in one segment — both must be counted, not just the first.
- Partial request split across segment boundary — no premature stats before completion.
- No responses — transaction counter must remain 0, not track requests.

## Failure Guidance

"HOLDOUT LOW: HS-051 (satisfaction: 0.XX) -- HTTP pipelined or partial-buffered request counting was incorrect; check method map cardinality, transaction semantics, or partial-request buffering logic."
