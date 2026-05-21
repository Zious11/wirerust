---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-018.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.041.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.027.md
input-hash: "41ea40b"
traces_to: .factory/stories/STORY-018.md
id: "HS-048"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.041
  - BC-2.04.027
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Depth Truncation in One Direction Leaves Other Direction Intact

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A TCP flow has an asymmetric data volume: the client uploads a very large file
(exceeding `max_depth`) while the server responds with small acknowledgment
messages. The depth limit on the client direction should not affect the server
direction — the server's responses should continue to be processed normally
and contribute to protocol-level analysis.

1. A pcap contains a TCP flow where:
   - Client-to-server direction: 50 MB of file upload data (far exceeding `max_depth`
     of, say, 10 MB).
   - Server-to-client direction: 100 small 200-byte HTTP response headers.
2. The user runs: `wirerust analyze <asymmetric-upload-pcap> --output-format json`
3. The tool completes with exit code 0.
4. A truncation finding is emitted for the client-to-server direction.
5. The server-to-client HTTP response headers are still analyzed normally —
   HTTP status codes and headers from the server direction appear in output.
6. `bytes_reassembled` for the server direction is accurate (all 100 × 200 bytes).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.041 | invariant 1: depth_exceeded is per-direction, permanent once set | Client direction truncated; server direction unaffected |
| BC-2.04.027 | edge case EC-004: depth exceedance is per-direction | Server direction continues normally after client truncation |

## Verification Approach

```bash
wirerust analyze <asymmetric-upload-pcap> --output-format json
```

Verify:
- A truncation finding exists for the client-to-server direction (evidenced by
  a finding mentioning "Stream depth exceeded").
- HTTP findings for the server-to-client direction are present (status codes,
  headers, or response statistics visible).
- `bytes_reassembled` for the server direction is the full 20,000 bytes (100 × 200).
- No truncation finding for the server-to-client direction.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Server direction fully processed
  despite client direction being truncated.
- **Edge case handling** (weight: 0.2): Both directions can independently reach
  their own depth limits without cross-contamination.
- **Error quality** (weight: 0.1): No crash from asymmetric depth scenario.
- **Performance** (weight: 0.1): Server direction bytes processed at normal speed;
  client depth-exceeded path is fast (no byte storage).
- **Data integrity** (weight: 0.1): `bytes_reassembled` correctly reflects only
  the non-truncated server bytes.

## Edge Conditions

- Both directions simultaneously hit their depth limit: two truncation findings,
  one per direction. Both subsequent directions return DepthExceeded.
- Client direction truncated at exactly `max_depth` bytes: no truncation (the
  limit is inclusive — exactly `max_depth` is allowed).
- Server direction's responses interleaved with client's uploads: per-direction
  independence means no interaction between the two buffers.

## Failure Guidance

"HOLDOUT LOW: HS-048 (satisfaction: 0.XX) — depth truncation in the client
direction incorrectly affected the server direction; server-to-client data was
not fully processed when client-to-server direction exceeded max_depth."
