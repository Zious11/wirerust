---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-016.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.035.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.043.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.047.md
input-hash: "6e9a81c"
traces_to: .factory/stories/STORY-016.md
id: "HS-030"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.035
  - BC-2.04.043
  - BC-2.04.047
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Normal TCP Retransmissions Do Not Produce False-Positive Findings

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A pcap is captured from a network with moderate packet loss. As a result,
several TCP segments are retransmitted by the sender — the same bytes at the
same sequence numbers appear twice in the capture. This is a routine occurrence
in production networks and should not be flagged as an attack.

1. A pcap contains a legitimate TCP flow with 5 duplicate retransmissions
   (identical bytes at identical offsets, which is normal TCP behavior under
   packet loss conditions).
2. The user runs: `wirerust analyze <retransmit-pcap> --output-format json`
3. The tool completes with exit code 0. No anomaly findings with MITRE T1036
   are emitted for the duplicate retransmissions.
4. The `bytes_reassembled` statistic reflects the actual data bytes, not
   counting the duplicate retransmissions twice.
5. Adjacent segments (where one starts exactly where another ends) are also
   NOT flagged as overlapping.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.035 | postcondition 1-2: identical retransmission returns Duplicate; segments and buffered_bytes unchanged | No data duplication; no spurious conflict |
| BC-2.04.043 | postcondition 1-2: adjacent segments are not overlapping | Back-to-back segments in order don't trip overlap detection |
| BC-2.04.047 | postcondition 1: buffered_bytes mirrors actual stored bytes | Memory accounting stays correct after deduplication |

## Verification Approach

This is a real-world corpus scenario. Use a pcap from a production network
or from a known-good repository (e.g., Wireshark sample captures) that
contains TCP retransmissions:

```bash
wirerust analyze <production-retransmit-pcap> --output-format json
```

Inspect the JSON output:
- Count findings tagged T1036: should be 0 for a clean retransmission-only pcap.
- Verify `bytes_reassembled` is not inflated by duplicate segments.
- Verify no error about "conflicting bytes" in any finding summary.

### Known-good corpus

**Corpus source:** Wireshark sample captures — `http.pcap` or `smtp.pcap`
from https://wiki.wireshark.org/SampleCaptures (well-maintained reference pcaps
known to contain normal retransmissions on real networks).

**Expected result:** Zero T1036 findings. Low or zero `dropped_findings`.
`bytes_reassembled` matches the actual application payload.

**False positive threshold:** 0 T1036 findings for purely duplicate retransmissions.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): No T1036 findings for identical retransmissions.
- **Edge case handling** (weight: 0.2): Adjacent segments also not flagged.
- **Error quality** (weight: 0.1): Tool completes cleanly without error.
- **Performance** (weight: 0.1): Normal throughput on a typical production pcap.
- **Data integrity** (weight: 0.1): `bytes_reassembled` accurate; no double-counting.

## Edge Conditions

- A segment retransmitted 10 times: still a Duplicate each time; no finding.
- An exactly-adjacent segment: starts where previous ends; not an overlap.
- A segment retransmitted with one changed byte: THAT is a conflict and should produce a finding.

## Failure Guidance

"HOLDOUT LOW: HS-030 (satisfaction: 0.XX) — normal TCP retransmissions
produced spurious T1036/ConflictingOverlap findings; this is a false positive."
