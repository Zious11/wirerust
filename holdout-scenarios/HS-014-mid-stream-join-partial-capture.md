---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-014.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.009.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.031.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.032.md
input-hash: "e83aa7b"
traces_to: .factory/specs/prd.md
id: "HS-014"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.009
  - BC-2.04.031
  - BC-2.04.032
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Mid-Stream Join — Partial Captures Analyzed Without Silent Data Corruption

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A forensic analyst receives a pcap that was started AFTER a TCP connection was already
   established — no SYN or SYN+ACK packets are present. The capture begins with a data
   segment mid-flow.
2. wirerust successfully processes the capture without crashing. The flow is marked as
   a partial capture in the reassembly statistics.
3. Data from the mid-stream capture is reassembled and passed to protocol analyzers.
   If the data contains detectable HTTP or TLS content, findings are still emitted — the
   absence of a handshake does not suppress analysis.
4. A second scenario: the analyst captures a connection from the beginning, but the pcap
   was truncated mid-flow (connection never closed). wirerust processes whatever data is
   present without hanging or crashing.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.009 | Postcondition 1-2 — mid-stream join: ISN inferred as seq-1; flow marked partial | Steps 1-2: partial capture processing |
| BC-2.04.031 | Postcondition 2 — ISN inferred as seq-1 on data-without-SYN | Step 1-2: ISN inference |
| BC-2.04.032 | Postcondition 1 — insert_segment with no ISN returns IsnMissing; inserts nothing | Step 1 edge: safe guard if ISN never set |

## Verification Approach

Craft or obtain a pcap where a TCP connection begins mid-capture:
```
wirerust analyze --output-format json mid_stream.pcap | jq '.analyzers.reassembly'
```

Check that the reassembly stats indicate partial flows (a `partial_flows` counter or
equivalent); no panic; exit code 0.

If the mid-stream data is HTTP: check that HTTP findings (if any anomalies present) are
emitted despite the missing handshake.

For a truncated capture (no close):
```
wirerust analyze --output-format json truncated_session.pcap
```

Expect: exit 0; bytes_reassembled > 0; no hang; finalize is called and flushes remaining
buffered segments.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Mid-stream captures are processed without
  error; partial flow is tracked; data is passed to analyzers.
- **Data integrity** (weight: 0.3): ISN inferred as seq-1 for first data segment; subsequent
  segments are in the correct order relative to the inferred ISN.
- **Edge case handling** (weight: 0.15): Truncated capture finalizes cleanly without hanging;
  no spurious IsnMissing errors printed for each packet.
- **Error quality** (weight: 0.1): If ISN_MISSING warning fires, it fires at most once
  per process (not once per packet).

## Edge Conditions

- A flow where the first two data packets arrive out of order mid-stream — both must be
  reassembled correctly relative to the inferred ISN.
- A TCP session that arrives with only ACK packets (no data) and no SYN: the flow should
  not produce a spurious finding or crash.
- Multiple concurrent mid-stream flows: each infers its own ISN independently.

## Failure Guidance

"HOLDOUT LOW: HS-014 (satisfaction: 0.XX) — mid-stream join crashes, marks flows as
corrupted instead of partial, or produces spurious IsnMissing warnings for every packet."
