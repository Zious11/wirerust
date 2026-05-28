---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-014.md
  - .factory/stories/STORY-013.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.039.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.006.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.007.md
input-hash: "5b09f99"
traces_to: .factory/specs/prd.md
id: "HS-019"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.039
  - BC-2.04.006
  - BC-2.04.007
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: TCP Sequence Number Wraparound — Reassembly Correctness Across 32-Bit Boundary

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A forensic analyst has a pcap of a large TCP transfer whose sequence numbers wrap
   around the 32-bit boundary (sequence number reaches 0xFFFFFFFF and then continues
   from 0x00000000).
2. wirerust reassembles the stream correctly across the wraparound — the data delivered
   to the protocol analyzer is contiguous and in the correct order, not split at the
   wraparound boundary.
3. No spurious "conflicting overlap" or "out-of-window" findings are emitted due to
   the wraparound — the reassembly engine correctly handles modular 32-bit arithmetic.
4. The bytes_reassembled counter correctly accounts for data on both sides of the
   wraparound point.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.039 | Postcondition 1 — TCP sequence wraparound across 32-bit boundary reassembles correctly | Steps 2-3: core wraparound correctness |
| BC-2.04.006 | Postcondition 1 — bidirectional data delivered with correct Direction tag | Step 2: direction tagging survives wraparound |
| BC-2.04.007 | Postcondition 1 — in-order data flushes contiguously in segment order | Step 2: segments flushed in correct order at boundary |

## Verification Approach

Craft a synthetic pcap with:
- ISN set to 0xFFFFFF00 (sequence wraps after 256 bytes)
- Send two segments: bytes 0-127 at seq 0xFFFFFF00, bytes 128-255 at seq 0x00000000
- Both segments should be delivered contiguously in the correct order

```
wirerust analyze --output-format json seqwrap.pcap | jq '{
  bytes_reassembled: .analyzers.reassembly.bytes_reassembled,
  findings_count: (.findings | length)
}'
```

Expect:
- bytes_reassembled == 256 (all data delivered)
- findings_count == 0 (no false-positive conflict or out-of-window findings)

If any findings are present, they must NOT be T1036 (conflicting overlap) or out-of-window
findings triggered by the wraparound arithmetic.

## Evaluation Rubric

- **Functional correctness** (weight: 0.55): Both segments delivered in correct order;
  bytes_reassembled equals total payload size; no spurious findings.
- **Data integrity** (weight: 0.3): Data delivered to analyzer is the original two segments
  concatenated, not truncated or reordered.
- **Edge case handling** (weight: 0.15): The wraparound itself does not produce an error
  log or warning unless a genuine anomaly exists.

## Edge Conditions

- The exact boundary segment: last byte at seq 0xFFFFFFFF and first byte at seq 0x00000000
  — these must be treated as adjacent, not as a large gap.
- Out-of-order delivery where the second segment (post-wraparound) arrives before the first:
  the reordering logic must correctly interpret the wraparound in the offset calculation.

## Failure Guidance

"HOLDOUT LOW: HS-019 (satisfaction: 0.XX) — TCP sequence wraparound causes data loss,
wrong ordering, or spurious conflict/out-of-window findings at the 32-bit boundary."
