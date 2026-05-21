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
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.004.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.005.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.050.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.051.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.053.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
id: "HS-013"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.004
  - BC-2.04.005
  - BC-2.04.050
  - BC-2.04.051
  - BC-2.04.053
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Three-Way Handshake Completion and RST Abrupt Close

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A forensic analyst processes a pcap of a complete TCP session: SYN from client,
   SYN+ACK from server, ACK (completing handshake), data exchange, FIN close.
2. wirerust processes this without emitting any anomaly findings for the TCP layer —
   it is a normal, expected connection.
3. The analyst also processes a second pcap where a TCP RST arrives mid-session.
4. The RST immediately terminates the flow tracking — subsequent packets on the same
   5-tuple do not generate new findings or accumulate in memory.
5. A third pcap has traffic where the RST arrives from the server side (not the client).
   The flow is still closed immediately from the RST, regardless of direction.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.004 | Postcondition 1 — first SYN sets client ISN and initiator | Step 1: SYN initiates the handshake |
| BC-2.04.005 | Postcondition 1 — SYN+ACK transitions state to Established | Step 1: second step of three-way handshake |
| BC-2.04.050 | Postcondition 1 — full state machine: New->SynSent->Established->Closing->Closed | Steps 1-2: normal lifecycle |
| BC-2.04.051 | Postcondition 1 — RST transitions to Closed from any state | Steps 3-5: RST termination |
| BC-2.04.053 | Postcondition 1 — direction returns ClientToServer when src matches initiator | Steps 1-2: direction tagging |

## Verification Approach

```
wirerust analyze --output-format json complete_session.pcap | jq '.findings | length'
```

For a clean complete TCP session: expect 0 reassembly-level findings.

```
wirerust analyze --output-format json rst_mid_session.pcap | jq '.analyzers.reassembly'
```

Check that flows_closed (or equivalent) counter shows the RST-closed flow; no pending
buffered data for that flow remains after the RST.

For server-side RST: same behavior — the flow is closed and removed from the flow table.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Clean session produces zero TCP anomaly findings;
  RST closes the flow immediately regardless of which side sends it.
- **Data integrity** (weight: 0.25): After RST, no buffered data remains for that flow;
  subsequent packets on the same 5-tuple start a new flow.
- **Edge case handling** (weight: 0.15): RST during handshake (before Established) also
  closes the flow cleanly.
- **Error quality** (weight: 0.1): No spurious warnings for normal session teardown.

## Edge Conditions

- A RST packet with no data (bare RST): closes flow immediately.
- A RST + data in same packet: the RST takes effect; the data is discarded.
- Multiple RST packets for the same flow (retransmitted RST): second RST should be silently
  ignored (flow already closed).

## Failure Guidance

"HOLDOUT LOW: HS-013 (satisfaction: 0.XX) — TCP three-way handshake tracking produces
spurious findings, RST does not close the flow, or direction tagging is wrong."
