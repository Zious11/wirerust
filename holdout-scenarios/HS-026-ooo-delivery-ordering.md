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
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.007.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.008.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.039.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-015.md
id: "HS-026"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.007
  - BC-2.04.008
  - BC-2.04.039
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Out-of-Order Segment Delivery Preserves Application Byte Order

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A pcap file contains a bidirectional TCP flow where one direction delivers
segments in a non-sequential order: the third chunk arrives first, the
second chunk arrives second, and the first chunk (covering the gap) arrives
last. The tool is run against this pcap.

1. The pcap contains a TCP flow where the client sends three 100-byte
   segments. Segment 3 (bytes 200-299) arrives in the capture first.
   Segment 2 (bytes 100-199) arrives second. Segment 1 (bytes 0-99) arrives
   last (the gap-filler).
2. The user runs: `wirerust analyze <pcap-file>`
3. The tool completes without error. The protocol analyzer attached to this
   flow receives the reassembled data in exactly the right byte order — first
   bytes 0-99, then 100-199, then 200-299. No bytes are delivered until the
   gap-filling segment arrives.
4. The tool's statistics show `bytes_reassembled` equal to the full payload
   length of the flow.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.007 | postcondition 1-2: flush consumes from base_offset only when segment present | Segments 1 and 2 must not be delivered before the gap-filler arrives |
| BC-2.04.008 | postcondition 5: gap-fill triggers delivery of all previously-buffered contiguous segments | The arrival of segment 1 should cause all three segments to flush in order |
| BC-2.04.039 | postcondition 1: ISN-relative offset arithmetic handles arbitrary arrival order | Offsets are computed correctly regardless of arrival order |

## Verification Approach

Craft or use a pcap with a three-segment out-of-order TCP flow. Run:

```bash
wirerust analyze <ooo-pcap> --output-format json
```

Confirm the exit code is 0. If a debugging hook is available (or via
tests), confirm the reassembled bytes are `[seg1_data, seg2_data, seg3_data]`
in that order. The key observable: if the tool emits protocol-level findings,
those findings must be consistent with correctly ordered data (not scrambled).

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Does the tool complete without
  error, and does the byte-count statistic match the total payload?
- **Edge case handling** (weight: 0.3): Are no spurious findings emitted for
  a normal out-of-order flow (which is common on the internet)?
- **Error quality** (weight: 0.1): No crash or panic on legitimate OOO traffic.
- **Performance** (weight: 0.05): Completes in a normal timeframe.
- **Data integrity** (weight: 0.05): `bytes_reassembled` matches expected total.

## Edge Conditions

- What happens when the gap is never filled (flow closed before segment 1 arrives)?
  The buffered segments at offsets 100+ should be dropped on flow close — the
  tool must not deliver them out of order.
- What happens with a single-segment flow? No buffering needed; should be
  delivered immediately.
- Three segments all arriving in-order should also work (baseline sanity).

## Failure Guidance

"HOLDOUT LOW: HS-026 (satisfaction: 0.XX) — out-of-order TCP segments were not
reassembled in correct byte order before delivery to the protocol layer."
