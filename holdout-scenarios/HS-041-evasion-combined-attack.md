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
  - .factory/stories/STORY-016.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.018.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.019.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.022.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.036.md
input-hash: "c513ef9"
traces_to: .factory/stories/STORY-017.md
id: "HS-041"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.018
  - BC-2.04.019
  - BC-2.04.022
  - BC-2.04.036
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Combined Evasion — Conflicting Bytes Plus Cumulative Overlap Threshold

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A sophisticated attacker uses a combination of TCP evasion techniques in the
same flow: some segments have conflicting bytes (attempting byte substitution)
and many additional segments overlap without conflict (building up the cumulative
overlap count). The tool should detect both the per-event conflict and the
cumulative threshold crossing, with bounded finding generation.

1. A pcap contains one TCP flow with:
   - 3 conflicting-byte overlaps (different bytes at same offsets): each produces
     a T1036/High finding.
   - 25 duplicate retransmissions (same bytes, same offsets) which accumulate
     `overlap_count` toward the threshold.
   - When `overlap_count` exceeds the configured threshold, a cumulative
     T1036/Medium finding fires exactly once per direction.
2. The user runs: `wirerust analyze <combined-evasion-pcap> --output-format json`
3. The tool emits 3 High confidence T1036 findings (one per conflict) plus at
   most 1 Medium confidence T1036 finding per direction (cumulative threshold).
4. The total number of T1036 findings is bounded: not 25 findings for 25 retransmissions.
5. Exit code is 0.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.018 | postcondition 2: one finding per ConflictingOverlap event | 3 conflicts = 3 High findings |
| BC-2.04.019 | postcondition 1: cumulative threshold alert fires once when overlap_count > threshold | One additional Medium finding |
| BC-2.04.022 | postcondition 3: once latch set, re-evaluation is no-op | No additional Medium findings after latch |
| BC-2.04.036 | postcondition 2-3: partial overlap gap bytes added; existing bytes preserved | PartialOverlap path correct in mixed scenario |

## Verification Approach

```bash
wirerust analyze <combined-evasion-pcap> --output-format json
```

Count findings by confidence level:
- High confidence T1036 findings: expect exactly 3 (one per conflict).
- Medium confidence T1036 findings: expect at most 1 per direction (cumulative latch).
- Total T1036 findings: expect at most 5 (3 High + 1 Medium per direction for bidirectional flow).
- The 25 pure-duplicate retransmissions produce 0 additional findings beyond their
  contribution to `overlap_count`.

Verify the cumulative latch behavior: if overlap_count just crosses the threshold
on the 26th overlap, the Medium finding fires then, not earlier.

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): Exactly the right number of findings
  per attack technique.
- **Edge case handling** (weight: 0.3): Latch prevents Medium alert flooding
  despite 25+ additional duplicates.
- **Error quality** (weight: 0.1): No crash or panic from complex overlap patterns.
- **Performance** (weight: 0.1): Normal throughput even with 25+ overlap segments.
- **Data integrity** (weight: 0.1): First-wins policy consistently applied;
  conflicting bytes never inserted into the stream.

## Edge Conditions

- All 3 conflicts happen before the cumulative threshold is crossed: findings arrive
  in the correct order (per-event Highs first, then cumulative Medium when threshold hit).
- Findings cap (10,000) is already full when the cumulative alert fires: the latch
  is still set, but the Medium finding is dropped; `dropped_findings` increments.
- Both C2S and S2C directions independently accumulate overlaps: each direction
  fires its own cumulative latch.

## Failure Guidance

"HOLDOUT LOW: HS-041 (satisfaction: 0.XX) — combined evasion scenario produced
incorrect finding counts: either per-event conflicts were not individually detected,
or the cumulative overlap threshold alert fired more than once per direction."
