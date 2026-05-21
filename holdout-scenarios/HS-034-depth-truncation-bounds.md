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
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.041.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.023.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.027.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-018.md
id: "HS-034"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.041
  - BC-2.04.023
  - BC-2.04.027
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Stream Depth Limit Prevents Memory Exhaustion on Oversized Flows

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

An adversary or a malfunctioning endpoint sends a TCP flow with a massive
amount of data — far more than the reassembler's configured `max_depth`
limit. The tool must handle this gracefully: cap the buffered bytes,
emit a finding to signal the truncation, and continue processing other
flows without memory exhaustion.

1. A pcap contains a single TCP flow that transfers 100 MB of data on one
   direction, but the tool is configured with a `max_depth` of 10 MB
   (per direction).
2. The user runs: `wirerust analyze <deep-flow-pcap> --format json`
3. The tool completes with exit code 0. It does NOT run out of memory or
   slow to a crawl.
4. The JSON output contains a finding with category "Anomaly" and evidence
   mentioning "Stream depth exceeded" or similar.
5. The finding's evidence references the configured max depth value.
6. After the first truncation, subsequent segments for that direction are
   silently dropped (the tool does not emit a new finding for each additional
   segment past the limit).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.041 | postcondition 1-5: Truncated result caps bytes; depth_exceeded flag set | Only `max_depth` bytes are stored per direction |
| BC-2.04.023 | postcondition 1: Anomaly/Inconclusive/Low finding emitted with "Stream depth exceeded" | One finding signals the truncation event |
| BC-2.04.027 | postcondition 1-2: segments_depth_exceeded counter increments for each post-limit segment | Post-truncation segments tracked in stats |

## Verification Approach

Use a pcap with a very large TCP payload (or a synthetic one generated with
a traffic generation tool). The key check is that the tool terminates and
does not OOM:

```bash
wirerust analyze <large-flow-pcap> --format json
```

Verify:
- Exit code is 0 (no OOM, no panic).
- At least one finding exists with `category == "Anomaly"` whose evidence
  mentions stream depth or max depth.
- The finding confidence is "Low" (it is a resource-limit signal, not a
  confirmed attack).
- The tool's memory usage does not grow without bound during processing.

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): Exactly one truncation finding
  emitted per truncated direction; subsequent over-limit segments do not
  generate additional findings.
- **Edge case handling** (weight: 0.3): Depth truncation is per-direction;
  the other direction continues to accept data normally up to its own limit.
- **Error quality** (weight: 0.2): No OOM crash; graceful completion.
- **Performance** (weight: 0.05): Does not degrade catastrophically.
- **Data integrity** (weight: 0.05): The truncation counter in stats accurately
  reflects how many segments were rejected after the limit was hit.

## Edge Conditions

- Exactly at `max_depth`: no truncation occurs (the limit is inclusive).
- One byte over `max_depth`: truncation fires; the extra byte is dropped.
- Both directions independently exceed `max_depth`: each direction gets its
  own truncation event and finding.
- The truncation finding is blocked by MAX_FINDINGS cap (very busy pcap):
  `dropped_findings` increments instead; the depth_exceeded flag is still set.

## Failure Guidance

"HOLDOUT LOW: HS-034 (satisfaction: 0.XX) — the tool did not correctly enforce
the per-direction stream depth limit; either no truncation finding was emitted,
memory was not bounded, or additional findings were emitted for each post-limit
segment."
