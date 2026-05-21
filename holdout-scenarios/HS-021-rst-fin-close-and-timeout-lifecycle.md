---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-013.md
  - .factory/stories/STORY-014.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.010.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.011.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.012.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.013.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
id: "HS-021"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.010
  - BC-2.04.011
  - BC-2.04.012
  - BC-2.04.013
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: TCP Flow Close Variants — RST, FIN, and Idle Timeout All Release Resources

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains three types of TCP connection close: one session closes with a proper
   FIN/FIN-ACK exchange, one is abruptly terminated by RST, and one simply stops (the pcap
   ends before any close).
2. All three connections are properly removed from the flow table after they close —
   no memory leak where terminated connections remain tracked indefinitely.
3. For the FIN-closed connection: both FINs must be seen before the flow is removed
   (half-close state handled correctly).
4. For the RST-closed connection: the RST immediately removes the flow regardless of
   which side sent it.
5. For the timed-out connection: when wirerust calls finalize at end-of-file, the remaining
   open flows are flushed with a "timeout" close reason and removed from tracking.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.010 | Postcondition 1 — RST closes flow immediately with CloseReason::Rst; total_memory zeroed | Step 4: RST close path |
| BC-2.04.011 | Postcondition 1 — both FINs close flow with CloseReason::Fin | Step 3: FIN close path |
| BC-2.04.012 | Postcondition 1 — finalize flushes all remaining flows with Timeout; idempotent | Step 5: timeout close via finalize |
| BC-2.04.013 | Postcondition 1 — expire_flows closes idle flows past flow_timeout_secs | Step 5 supplement: mid-capture timeout |

## Verification Approach

```
wirerust analyze --output-format json three_close_types.pcap | jq '{
  bytes_reassembled: .analyzers.reassembly.bytes_reassembled,
  flows_total: .analyzers.reassembly.flows_total,
  findings: (.findings | map(select(.category == "Anomaly")))
}'
```

For the FIN-closed flow: no findings expected from clean close.
For the RST-closed flow: no findings expected from RST itself (RST is not an anomaly in
normal operation).
For the timeout flow: no findings expected just from timeout (unless content triggered them).

Verify that running wirerust twice on the same file produces identical output (finalize
is idempotent — this tests BC-2.04.012 invariant 1).

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): All three close reasons handled; flow table
  cleaned up; no residual tracked flows after finalize.
- **Data integrity** (weight: 0.25): bytes_reassembled reflects data from all three
  connections combined correctly.
- **Edge case handling** (weight: 0.2): FIN half-close (one FIN seen, other missing):
  flow remains until second FIN or timeout; correct behavior.
- **Error quality** (weight: 0.1): Repeated finalize calls do not cause double-close
  errors or panics.

## Edge Conditions

- A RST during the SYN_SENT state (before handshake complete): flow removed, no data
  to flush.
- A FIN from only one side followed by more data from the other side (half-close): the
  data side continues until its FIN or RST.
- finalize called on an already-empty flow table: no panic.

## Failure Guidance

"HOLDOUT LOW: HS-021 (satisfaction: 0.XX) — one of the three TCP close paths (RST, FIN,
timeout) fails to clean up flow state or produces incorrect behavior."
