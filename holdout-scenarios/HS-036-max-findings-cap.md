---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-021.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.024.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.054.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.025.md
input-hash: "a25eba6"
traces_to: .factory/stories/STORY-021.md
id: "HS-036"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.024
  - BC-2.04.054
  - BC-2.04.025
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Findings Cap Prevents Memory Exhaustion Under Adversarial Load

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

An adversary crafts a pcap designed to generate an enormous number of anomaly
findings — for example, a pcap with 50,000 conflicting overlapping segments.
The tool must not OOM or produce an unbounded output file; findings must be
capped.

1. A pcap contains TCP flows with more than 10,000 total events that would
   each normally generate a finding.
2. The user runs: `wirerust analyze <flood-pcap> --output-format json`
3. The tool completes with exit code 0 and a reasonable memory footprint.
4. The JSON output's findings array has at most 10,001 entries (the hard cap
   is 10,000 for normal findings, plus at most 1 unconditional segment-limit
   summary finding).
5. If any segments were dropped due to the segment-count-per-direction limit,
   the segment-limit summary finding IS present in the output regardless of
   whether the normal findings cap was hit.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.024 | postcondition 1-2: findings capped at 10,000; dropped_findings counter tracks excess | Output array bounded even under adversarial load |
| BC-2.04.054 | postcondition 1-2: segment-limit summary finding unconditionally bypasses the cap | The one intentional exception to the cap |
| BC-2.04.025 | postcondition 1 and invariant 3: summary uses correct singular/plural grammar | Evidence string format is correct |

## Verification Approach

Use a pcap engineered to flood the findings queue. Run:

```bash
wirerust analyze <flood-pcap> --output-format json | jq '.findings | length'
```

Verify the output is at most 10001. Verify the `dropped_findings` field in the
output statistics is > 0 (indicating findings were dropped).

If the pcap also causes segment-limit events (many flows with > `max_segments`
worth of gaps), verify the segment-limit summary finding appears even though the
regular findings array is at capacity.

Check the segment-limit finding's summary string: if only 1 segment was dropped,
the summary should say "1 segment dropped"; if multiple, "N segments dropped".

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): findings.len() <= 10001 after any run.
- **Edge case handling** (weight: 0.3): Segment-limit summary finding bypasses cap
  correctly; it appears even at cap.
- **Error quality** (weight: 0.2): No OOM crash; exit code 0.
- **Performance** (weight: 0.05): Tool completes in reasonable time even for adversarial pcaps.
- **Data integrity** (weight: 0.05): `dropped_findings` accurately reflects
  how many findings were silently discarded.

## Edge Conditions

- Findings array at exactly 9,999 entries: next finding is accepted (bringing to 10,000).
- Findings array at exactly 10,000 entries: next NORMAL finding is dropped; `dropped_findings++`.
- `dropped_findings` itself never exceeds u64::MAX (saturating arithmetic).
- Finalize with zero segment-limit events: no segment-limit summary finding;
  findings.len() <= 10,000 exactly.

## Failure Guidance

"HOLDOUT LOW: HS-036 (satisfaction: 0.XX) — the findings array exceeded 10,001
entries (unbounded growth under adversarial load), or the segment-limit summary
finding was suppressed by the cap when it should have been unconditionally emitted."
