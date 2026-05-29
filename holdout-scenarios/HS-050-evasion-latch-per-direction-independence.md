---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-017.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.022.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.021.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.020.md
input-hash: "ba4dbfd"
traces_to: .factory/stories/STORY-017.md
id: "HS-050"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.022
  - BC-2.04.021
  - BC-2.04.020
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Anomaly Alert Latches Are Per-Direction — Both Can Fire Independently

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A bidirectional flow experiences anomalies in BOTH directions simultaneously
— for example, the client-to-server direction sends many small segments
AND the server-to-client direction sends out-of-window segments. Both directions
should generate their respective anomaly alerts independently, without one
direction's latch suppressing the other.

1. A pcap contains a bidirectional TCP flow where:
   - Client-to-server direction: 200 consecutive 1-byte segments (small segment
     alert should fire for this direction when threshold crossed).
   - Server-to-client direction: 50 segments beyond the receive window (OOW alert
     should fire for this direction when threshold crossed).
2. The user runs: `wirerust analyze <bidirectional-anomaly-pcap> --output-format json`
3. The tool emits:
   - Exactly one small-segment alert (one Anomaly/Inconclusive/Medium finding)
     for the client-to-server direction.
   - Exactly one OOW alert (one Anomaly/Inconclusive/Low finding) for the
     server-to-client direction.
4. The maximum possible threshold-based findings for this flow is 6 (3 alert types
   × 2 directions). This scenario exercises 2 of those 6 slots independently.
5. The client-to-server latch being set does NOT prevent the server-to-client latch
   from firing, and vice versa.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.022 | postcondition 1: latch set before cap check | Both latches set even if cap is full |
| BC-2.04.022 | invariant 3: max 6 threshold findings per bidirectional flow | Upper bound on threshold findings is verified |
| BC-2.04.021 | postcondition 1-2: OOW one-shot finding with window size in evidence | S2C OOW alert fires correctly |
| BC-2.04.020 | postcondition 1-2: small-segment one-shot finding | C2S small-segment alert fires correctly |

## Verification Approach

```bash
wirerust analyze <bidirectional-anomaly-pcap> --output-format json
```

Inspect findings:
- Count findings with confidence "Medium" whose evidence/summary mentions
  "small segment" or segment run: should be exactly 1.
- Count findings with confidence "Low" whose evidence mentions "max_receive_window"
  or out-of-window: should be exactly 1.
- No additional alerts from these two anomaly types after the first fires.

Verify the two findings reference different directions (the small-segment is
C2S; the OOW is S2C). They should have different flow-direction indicators
if such information is surfaced.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Both directions fire their respective
  alerts independently; neither suppresses the other.
- **Edge case handling** (weight: 0.3): After each latch fires, additional segments
  in that direction produce no new threshold alerts.
- **Error quality** (weight: 0.1): No panic from concurrent per-direction latch updates.
- **Performance** (weight: 0.05): Normal throughput.
- **Data integrity** (weight: 0.05): Finding counts bounded by 6 per flow.

## Edge Conditions

- All 3 alert types fire in both directions: 6 total threshold findings.
- One direction has all 3 latches set; the other direction still has fresh latches
  to fire.
- Latch set when findings cap is full: finding dropped; latch still set; subsequent
  threshold crossings are no-ops.
- The findings cap prevents the 7th threshold finding (past 6) even if it were
  theoretically possible.

## Failure Guidance

"HOLDOUT LOW: HS-050 (satisfaction: 0.XX) — per-direction anomaly latches
interfered with each other; setting a latch in one direction suppressed a
valid alert in the other direction, causing a missed anomaly detection."
