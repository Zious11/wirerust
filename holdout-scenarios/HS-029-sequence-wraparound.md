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
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.039.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-015.md
id: "HS-029"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
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

# Holdout Scenario: TCP Sequence Number Wraparound Across 32-bit Boundary

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A long-running TCP flow transfers enough data that the 32-bit TCP sequence
number wraps around from near `4294967295` (2^32 - 1) back to near zero.
The pcap contains segments on both sides of this wrap boundary.

1. A pcap contains a TCP flow whose Initial Sequence Number (ISN) is set very
   close to the maximum 32-bit unsigned value (e.g., ISN = 4294967200). The
   flow transfers data across the wraparound point: some segments have sequence
   numbers near `4294967295` and subsequent segments have sequence numbers
   near `0`, `1`, `2`, etc.
2. The user runs: `wirerust analyze <wraparound-pcap> --format json`
3. The tool completes with exit code 0. The reassembled byte stream is
   contiguous and correctly ordered — bytes from the high-sequence segments
   come before bytes from the low-sequence (wrapped) segments.
4. No spurious overlap or conflict findings are emitted for the wrapped segments;
   the wraparound is treated as normal in-order continuation of the stream.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.039 | postcondition 1: wrapping_sub arithmetic produces monotonically-increasing u64 offsets | Segments post-wraparound have correct ISN-relative offsets |
| BC-2.04.039 | postcondition 3: flush delivers wrapped segments in correct byte order | The reassembled data is in proper order across the wrap boundary |

## Verification Approach

Use a crafted or real-world pcap where TCP sequence numbers wrap. Run:

```bash
wirerust analyze <wraparound-pcap> --format json
```

Verify:
- Exit code is 0.
- No anomaly findings mentioning "overlap" or "conflict" for the wraparound
  transition (wraparound is not an evasion attack).
- `bytes_reassembled` equals the total payload bytes across all segments.
- If any protocol-level content is present (e.g., HTTP), it is correctly
  recognized — meaning the byte order is right.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Wraparound handled transparently;
  bytes delivered in correct order.
- **Edge case handling** (weight: 0.3): No false-positive findings for the
  wrap transition; no missing bytes at the boundary.
- **Error quality** (weight: 0.1): No panic or arithmetic overflow.
- **Performance** (weight: 0.05): No performance degradation for long flows.
- **Data integrity** (weight: 0.05): Byte count correct; no duplication.

## Edge Conditions

- ISN at exactly `u32::MAX - 1`: first post-ISN segment has seq=u32::MAX,
  second has seq=0. Both offsets must be computed correctly.
- Out-of-order arrival across the boundary: a segment with wrapped seq arrives
  before the bridging segment. Must buffer correctly.
- What if ISN itself is 0? No wraparound; standard path. Must still work.

## Failure Guidance

"HOLDOUT LOW: HS-029 (satisfaction: 0.XX) — TCP sequence number wraparound
caused incorrect byte ordering or spurious overlap findings near the 32-bit
boundary."
