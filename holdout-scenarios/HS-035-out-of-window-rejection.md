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
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.042.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.021.md
input-hash: "9c5b099"
traces_to: .factory/stories/STORY-018.md
id: "HS-035"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.042
  - BC-2.04.021
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Out-of-Window Segments Are Rejected and Counted

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

An adversary or a misconfigured endpoint sends TCP segments far beyond the
receiver's advertised window. These segments cannot be legitimately buffered
and should be rejected. However, excessive out-of-window traffic warrants
an anomaly finding.

1. A pcap contains a TCP flow where one side sends 50 segments with sequence
   numbers that place them beyond the current receive window. These segments
   arrive sequentially, not as retransmissions.
2. The user runs: `wirerust analyze <oow-pcap> --output-format json`
3. The tool completes with exit code 0.
4. The JSON output contains an anomaly finding for excessive out-of-window
   segments. The finding's evidence mentions the `max_receive_window` value.
5. The finding fires only once per direction (it does not flood with 50 findings).
6. A segment arriving at exactly the boundary of `base_offset + max_receive_window`
   is accepted — only segments strictly beyond the window are rejected.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.042 | postcondition 1: OutOfWindow returned for segment beyond window; no bytes stored | Rejected segments leave no bytes in buffer |
| BC-2.04.042 | postcondition 4: out_of_window_count increments per rejected segment | Counter tracks rejections |
| BC-2.04.021 | postcondition 1-2: one-shot finding when count exceeds threshold; evidence contains window size | One finding emitted with window size in evidence |

## Verification Approach

```bash
wirerust analyze <oow-pcap> --output-format json
```

Inspect JSON findings:
- At least one finding whose summary or evidence references the receive window
  or out-of-window behavior.
- The finding's confidence is "Low" (it signals possible misconfiguration or
  evasion, not a confirmed attack).
- Only one such finding per direction (the latch prevents flooding).
- Segments exactly at the window boundary: their bytes appear in the reassembled
  stream (no rejection at the boundary).

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Exactly one OOW finding per direction;
  boundary segments accepted; beyond-boundary segments rejected.
- **Edge case handling** (weight: 0.2): Boundary condition (exactly at window edge)
  is accepted, not rejected.
- **Error quality** (weight: 0.1): No panic from large sequence numbers.
- **Performance** (weight: 0.1): Rejection is fast; no quadratic behavior.
- **Data integrity** (weight: 0.1): `bytes_reassembled` does not include
  out-of-window bytes.

## Edge Conditions

- `base_offset` near `u64::MAX`: saturating addition prevents overflow in the
  window check.
- OOW alert fires when findings cap is full: `dropped_findings` increments;
  latch is still set.
- Both directions independently exceed OOW threshold: each direction fires
  its own one-shot alert.

## Failure Guidance

"HOLDOUT LOW: HS-035 (satisfaction: 0.XX) — out-of-window segments were accepted
into the reassembly buffer, or the OOW anomaly finding was not emitted (or was
emitted more than once per direction)."
