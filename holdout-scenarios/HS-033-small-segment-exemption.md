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
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.020.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-017.md
id: "HS-033"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
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

# Holdout Scenario: Small-Segment Alert Respects Port Exemption List

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

Some protocols naturally use very small TCP segments — for example, interactive
shell sessions (SSH on port 22, Telnet on port 23) send individual keystrokes
as 1-byte TCP segments. The tool must not flag these protocols as suspicious
simply because they use many small segments.

1. A pcap contains a long Telnet session on port 23 where 500 consecutive
   1-byte TCP segments are exchanged (every keystroke is its own segment —
   normal Telnet behavior).
2. The user runs: `wirerust analyze <telnet-pcap> --output-format json`
3. The tool completes with exit code 0. No small-segment anomaly finding is
   emitted for the Telnet flow.
4. A second pcap contains a suspicious flow on a non-exempt port (e.g., port
   8080) with 500 consecutive 1-byte segments. The tool DOES emit a small-segment
   anomaly finding for that flow.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.020 | invariant 2: if either endpoint port is in small_segment_ignore_ports, no alert | Telnet port is exempt; no finding |
| BC-2.04.020 | postcondition 1-2: alert fires for non-exempt ports exceeding run threshold | Port 8080 with many small segments does trigger |

## Verification Approach

```bash
wirerust analyze <telnet-pcap> --output-format json   # Should: no small-segment finding
wirerust analyze <port8080-small-pcap> --output-format json  # Should: small-segment anomaly finding
```

For the Telnet pcap: scan the JSON findings for any finding whose summary
mentions "small segment" or "segment" and verify there are none.

For the port 8080 pcap: verify at least one finding exists with category
"Anomaly" and whose summary or evidence references small segments or
segment counts.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Telnet (port 23) produces zero
  small-segment findings; non-exempt port produces at least one.
- **Edge case handling** (weight: 0.2): The exemption applies to EITHER endpoint
  port — a flow with dst=23 but src=54321 is still exempt.
- **Error quality** (weight: 0.1): No crash or parse errors.
- **Performance** (weight: 0.1): Normal throughput even for 500-segment flows.
- **Data integrity** (weight: 0.1): Findings are consistent between runs.

## Edge Conditions

- Port 22 (SSH) is also a common small-segment protocol: should be exempt.
- A flow with one endpoint on port 23 and one on a high-numbered ephemeral
  port: both endpoints should be checked; port 23 exempts the flow.
- The small-segment run resets when a normally-sized segment arrives; a run
  of 3 small segments, then 1 large, then 3 small should require two full runs
  to trigger (depending on threshold).

## Failure Guidance

"HOLDOUT LOW: HS-033 (satisfaction: 0.XX) — small-segment alert was emitted
for a port-exempt flow (e.g., Telnet on port 23) or was not emitted for a
non-exempt flow with excessive small segments."
