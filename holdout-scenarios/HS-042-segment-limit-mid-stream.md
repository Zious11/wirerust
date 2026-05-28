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
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.044.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.045.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.046.md
input-hash: "9c5b099"
traces_to: .factory/stories/STORY-018.md
id: "HS-042"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.044
  - BC-2.04.045
  - BC-2.04.046
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Per-Direction Segment Map Cap Prevents BTreeMap Overhead

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

An attacker attempts to exhaust the reassembler's memory by sending thousands
of small, non-contiguous TCP segments (each with a gap between them), forcing
the BTreeMap to grow without bound. The tool must enforce a per-direction
segment-count cap and signal when this protection triggers.

1. A pcap contains a TCP flow where one direction sends 5,000 individual 1-byte
   segments with large gaps between them — each segment arrives in-order but
   is spaced 1,000 bytes apart. This maximizes BTreeMap entries without triggering
   depth truncation.
2. The user runs: `wirerust analyze <many-gaps-pcap> --output-format json`
3. The tool completes with exit code 0 and reasonable memory usage.
4. After the `max_segments_per_direction` limit is hit, further non-contiguous
   segments are dropped and counted in `segments_segment_limit`.
5. At end of processing (finalize), the segment-limit summary finding is emitted
   in the output, noting how many segments were dropped.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.044 | postcondition 1: SegmentLimitReached for non-overlapping insert when map is full | Post-limit segments dropped |
| BC-2.04.045 | postcondition 1: SegmentLimitReached also returned for overlapping path when map is full | Overlap path also checked |
| BC-2.04.046 | postcondition 1-3: partial insertion when limit hit mid-loop | Some gap bytes in, later gap bytes dropped |

## Verification Approach

```bash
wirerust analyze <many-gaps-pcap> --output-format json
```

Verify:
- Exit code is 0 (no OOM or panic).
- The JSON findings array contains a segment-limit summary finding at the end.
- The summary mentions the number of segments dropped (in correct singular/plural form).
- The evidence strings in the summary finding reference "segment count limit" and
  "segmentation-based evasion."
- Memory usage during the run is bounded (does not grow proportionally to the
  5,000-segment input).

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): Segment-limit summary finding is
  present and accurate; tool terminates cleanly.
- **Edge case handling** (weight: 0.3): The cap is per-direction; the other
  direction continues to work normally.
- **Error quality** (weight: 0.2): No OOM; exit code 0.
- **Performance** (weight: 0.05): Memory usage stays bounded after cap is hit.
- **Data integrity** (weight: 0.05): `segments_segment_limit` counter accurately
  reflects dropped count.

## Edge Conditions

- Segment map at `max_segments - 1`: next segment accepted.
- Segment map at `max_segments`: next non-overlapping segment returns SegmentLimitReached.
- Segment map at `max_segments`, incoming segment is a pure duplicate (fully covered
  by existing segments): returns Duplicate, NOT SegmentLimitReached — the limit check
  only gates new gap insertion.
- Segment-limit finding must be unconditionally pushed even if normal findings cap
  is already full.

## Failure Guidance

"HOLDOUT LOW: HS-042 (satisfaction: 0.XX) — the per-direction segment-map cap
was not enforced; either OOM occurred from unbounded BTreeMap growth, or the
segment-limit summary finding was not emitted at end of processing."
