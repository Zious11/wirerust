---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-015.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.006.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-015.md
id: "HS-027"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.006
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Bidirectional Data Direction Tags Are Mutually Exclusive and Accurate

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A pcap contains a full HTTP exchange over TCP: the client sends an HTTP GET
request, and the server responds with an HTTP 200 response. The tool processes
this pcap and the HTTP analyzer is active.

1. The pcap contains a complete TCP handshake followed by an HTTP request from
   the client (SYN initiator) and an HTTP response from the server.
2. The user runs: `wirerust analyze <bidirectional-pcap> --output-format json`
3. The tool's internal reassembly correctly labels the HTTP request bytes as
   coming from the client direction and the HTTP response bytes as coming from
   the server direction.
4. Any HTTP-level statistics or findings produced reflect the correct
   directionality — for example, the HTTP method is seen in the client direction
   and the HTTP status code is seen in the server direction.
5. Flushing data from one direction has no effect on what has been buffered in
   the opposite direction.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.006 | postcondition 1: client-to-server bytes tagged ClientToServer | HTTP request bytes labeled correctly |
| BC-2.04.006 | postcondition 2: server-to-client bytes tagged ServerToClient | HTTP response bytes labeled correctly |
| BC-2.04.006 | invariant 2: directions are fully independent buffers | Flushing server data does not drain client buffer |

## Verification Approach

Use a real or synthetic pcap with a valid HTTP exchange. Run:

```bash
wirerust analyze <http-pcap> --output-format json
```

Inspect the JSON output for HTTP statistics or findings. Verify:
- The HTTP method (GET, POST, etc.) was observed — it comes from client direction.
- The HTTP status code was observed — it comes from server direction.
- No directionality inversion is evident (e.g., status code appearing before method).

For a stronger check: run with `--output-format json` and verify JSON fields that
indicate request/response detection are both populated correctly.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): HTTP request and response both
  recognized from the correct directions.
- **Edge case handling** (weight: 0.2): Simultaneous data in both directions
  does not cause one direction to corrupt the other.
- **Error quality** (weight: 0.1): No unexpected error messages.
- **Performance** (weight: 0.1): Normal completion time.
- **Data integrity** (weight: 0.1): `bytes_reassembled` accounts for both
  directions without double-counting.

## Edge Conditions

- What if the server sends data before the client does (server-push or
  interleaved writes)? Direction tags must still be accurate.
- What if both directions send data at the same time? Each direction's buffer
  must remain independent.
- An empty direction (client sends data, server sends nothing) should produce
  zero `bytes_reassembled` for the empty direction.

## Failure Guidance

"HOLDOUT LOW: HS-027 (satisfaction: 0.XX) — TCP direction tagging is incorrect;
client-side and server-side data are mixed or mislabeled in the reassembly output."
