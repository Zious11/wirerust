---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-001.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.002.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.003.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.006.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.007.md
input-hash: "a3ed987"
traces_to: .factory/specs/prd.md
id: "HS-002"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.01.002
  - BC-2.01.003
  - BC-2.01.006
  - BC-2.01.007
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Empty Capture and Corrupt-Header Behavior at Ingest

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A user runs wirerust on a valid pcap file that contains a well-formed pcap header but
   zero packet records (the capture was started and immediately stopped).
2. The tool completes successfully (exit code 0) and produces output that reflects zero
   packets — no crash, no spurious error.
3. A second user runs wirerust on a file where the pcap global header bytes are truncated
   or have a corrupted magic number.
4. The tool returns a non-zero exit code with a clear error message and does not panic
   or produce partial output.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.002 | Postcondition 1 — N=0 packets returned as empty Vec without error | Step 1-2: empty capture completes cleanly |
| BC-2.01.003 | Postcondition 1 — from_file returns Ok(PcapSource) with empty packets vec | Step 2: Ok path for header-only capture |
| BC-2.01.006 | Postcondition 1 — header parse error returns Err with anyhow context | Step 3-4: corrupt header triggers clear error |
| BC-2.01.007 | Postcondition 1 — per-packet read error returns Err with anyhow context | Step 4 edge: mid-stream truncation error path |

## Verification Approach

Construct a minimal valid pcap (just the 24-byte global header, no packets):

```
wirerust analyze header_only.pcap
```

Expect: exit 0, output shows zero findings, summary shows 0 total_packets.

Construct a corrupt file (first 4 bytes are not 0xd4c3b2a1 or 0xa1b2c3d4):

```
wirerust analyze corrupt_header.pcap
```

Expect: non-zero exit, stderr message references the parse failure context, no stdout
JSON or terminal output beyond the error.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Zero-packet file succeeds; corrupt header fails
  with non-zero exit and error message.
- **Error quality** (weight: 0.3): Error message for corrupt header is descriptive — not just
  "error" but contains context about what failed.
- **Edge case handling** (weight: 0.15): No panic in either case; output format remains valid.
- **Data integrity** (weight: 0.1): Zero-packet output contains valid (empty) summary structure.

## Edge Conditions

- Header-only pcap has magic number 0xd4c3b2a1 (big-endian or little-endian variants); both
  should be accepted if both link types are valid.
- Truncated mid-packet file (header valid, first packet record cut short) exercises BC-2.01.007.
- A 0-byte file is distinct from a header-only file and should also fail with a parse error.

## Failure Guidance

"HOLDOUT LOW: HS-002 (satisfaction: 0.XX) — zero-packet pcap handling or corrupt-header error
surfacing does not meet expectations."
