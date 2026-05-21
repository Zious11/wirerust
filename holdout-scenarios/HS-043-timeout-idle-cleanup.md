---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-019.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.013.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.029.md
input-hash: "de4f3ef"
traces_to: .factory/stories/STORY-019.md
id: "HS-043"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.013
  - BC-2.04.029
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Idle Flow Timeout Cleans Up Long-Silent Connections

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A pcap captures a mix of active flows and flows that go silent for a very
long time before the next packet. The tool should expire flows that have been
idle for longer than the configured timeout to prevent unbounded memory growth
from long-lived stale sessions.

1. A pcap captures two TCP flows:
   - Flow A: active throughout the capture, packets every few seconds.
   - Flow B: exchanges data at t=0, then goes completely silent until t=3600
     seconds (1 hour). The tool's `flow_timeout_secs` is configured to 300
     seconds (5 minutes).
2. The user runs: `wirerust analyze <idle-flow-pcap> --output-format json`
3. The tool completes with exit code 0.
4. Flow B is expired at some point during the 1-hour gap (when the next packet
   from any flow arrives after 300 seconds of silence for flow B).
5. Flow A continues to be tracked and analyzed normally.
6. The `flows_expired` statistic is at least 1 (flow B was expired).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.013 | postcondition 1-2: expire_flows closes flows idle past timeout; flows_expired increments | Flow B expired after 300s; flows_expired >= 1 |
| BC-2.04.013 | postcondition 4: active flows within timeout NOT closed | Flow A not affected by expire_flows |
| BC-2.04.013 | invariant 1: underflow-safe subtraction | No arithmetic errors from timestamp comparison |
| BC-2.04.029 | postcondition 4-5: close_flow for missing key emits one-shot warning | Should not occur in normal operation; defensive guard works |

## Verification Approach

```bash
wirerust analyze <idle-flow-pcap> --output-format json
```

Verify:
- `flows_expired` is 1 (flow B was expired) in statistics.
- Flow A's analysis (HTTP or TLS findings) is complete and not truncated.
- The tool does not crash from the long-idle scenario.
- `flows_expired` is 0 for a pcap where all flows close cleanly within the timeout.

For a stricter test: run on a pcap where the idle gap is exactly at the timeout
boundary. A gap of exactly `flow_timeout_secs` should NOT trigger expiry (strict `>`).

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Idle flow is expired; active flow
  is not affected.
- **Edge case handling** (weight: 0.2): Exactly-at-timeout flows are NOT expired
  (boundary condition).
- **Error quality** (weight: 0.1): No crash from long idle gaps or timestamp
  wraparound.
- **Performance** (weight: 0.1): Idle flow cleanup does not degrade processing
  speed for other flows.
- **Data integrity** (weight: 0.1): Statistics accurately reflect expiry count;
  bytes_reassembled not affected by expired flows.

## Edge Conditions

- `current_time < last_seen` (clock jitter or out-of-order timestamps): no expiry
  fires (underflow guard).
- Exactly `flow_timeout_secs` of idle: no expiry (strict `>`, not `>=`).
- One second past `flow_timeout_secs`: expiry fires.
- A flow already in state `FlowState::Closed` (FIN-closed but not yet evicted)
  is also cleaned up by expire_flows.

## Failure Guidance

"HOLDOUT LOW: HS-043 (satisfaction: 0.XX) — idle flow timeout did not clean up
long-silent TCP flows; the tool retained stale flows beyond the configured
timeout threshold, causing unbounded flow table growth."
