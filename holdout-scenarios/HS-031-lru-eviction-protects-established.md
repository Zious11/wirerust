---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-020.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.015.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.016.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.017.md
input-hash: "028f0b1"
traces_to: .factory/stories/STORY-020.md
id: "HS-031"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.015
  - BC-2.04.016
  - BC-2.04.017
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Memory Eviction Discards Incomplete Flows Before Established Sessions

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A pcap contains a mix of TCP traffic: many incomplete (SYN-only or
SYN/SYN-ACK only) connection attempts that never complete their handshake,
plus several fully-established flows that are actively exchanging data.
The number of these half-open connections exceeds the reassembler's
configured `max_flows` limit.

1. A pcap contains: 100 half-open connections (SYN sent, no SYN-ACK) and
   10 fully established connections actively transferring data. The reassembler
   is configured with `max_flows = 50`.
2. The user runs: `wirerust analyze <mixed-flows-pcap> --output-format json`
3. The tool completes with exit code 0.
4. The established flows are preferentially preserved; the half-open SYN-only
   flows are evicted first when the limit is hit.
5. The evicted half-open flows receive a MemoryPressure close reason in the
   tool's internal accounting. Established flows are not evicted until all
   incomplete flows have been evicted.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.015 | postcondition 1 and 3: non-Established flows evicted before Established; stats.evictions increments | Half-open SYNs evicted first |
| BC-2.04.016 | postcondition 1: memcap eviction triggers after total_memory exceeds threshold | Eviction fires when limits are hit |
| BC-2.04.017 | postcondition 1-4: sort places non-Established before Established; within group, oldest first | Eviction order is deterministic and correct |

## Verification Approach

Run the tool against a crafted pcap with mixed half-open and established flows:

```bash
wirerust analyze <mixed-flows-pcap> --output-format json
```

Verify:
- The tool completes without crash or panic.
- JSON output shows data from the established flows (HTTP/TLS findings, etc.).
- If `evictions` is surfaced in stats, it should be > 0.
- Established flows produce their expected findings; they were not prematurely
  evicted (no MemoryPressure close reason for established sessions).

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): Established flows are not
  prematurely evicted; established-flow findings are present in output.
- **Edge case handling** (weight: 0.3): Half-open flows are correctly evicted
  with MemoryPressure; no data corruption from eviction.
- **Error quality** (weight: 0.1): No panic on eviction path.
- **Performance** (weight: 0.1): Eviction does not cause excessive overhead.
- **Data integrity** (weight: 0.1): Memory accounting remains consistent
  (no leak or undercount) after evictions.

## Edge Conditions

- All flows are Established when eviction fires: oldest-by-last-seen is
  evicted (the LRU tie-breaker within the Established group).
- Single flow in table at max_flows=1: the one flow is evicted to make room
  for a new SYN.
- total_memory at exactly `memcap` bytes: no eviction fires (strict `>` check).
- total_memory at `memcap + 1` byte: eviction fires immediately.

## Failure Guidance

"HOLDOUT LOW: HS-031 (satisfaction: 0.XX) — established TCP flows were evicted
before half-open/SYN-only flows; the LRU non-established-first policy was not
applied correctly."
