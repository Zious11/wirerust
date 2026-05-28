---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-012.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.002.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.028.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.030.md
input-hash: "136f5d1"
traces_to: .factory/specs/prd.md
id: "HS-012"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.002
  - BC-2.04.028
  - BC-2.04.030
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Non-TCP Packet Filtering, Reassembly Stats, and Byte Accounting

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A forensic analyst runs wirerust analyze on a mixed pcap containing:
   - 20 TCP packets (10 bidirectional in one connection)
   - 30 UDP DNS packets
   - 5 ICMP echo packets
2. The JSON output summary shows `packets_skipped_non_tcp` equal to 35 (30 UDP + 5 ICMP).
3. The TCP data is processed; bytes_reassembled in the reassembly summary equals the total
   payload bytes successfully delivered to the analyzer — not the raw packet lengths.
4. The `AnalysisSummary` for the reassembly engine includes a stats map with at minimum:
   `packets_skipped_non_tcp`, `bytes_reassembled`, and flow counts.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.002 | Postcondition 1 — non-TCP packets increment packets_skipped_non_tcp; not processed further | Step 2: skip counter accuracy |
| BC-2.04.028 | Postcondition 1 — summarize returns AnalysisSummary with reassembly stats detail map | Step 4: stats map structure |
| BC-2.04.030 | Postcondition 1 — bytes_reassembled equals total bytes delivered to handler | Step 3: byte accounting correctness |

## Verification Approach

```
wirerust analyze --output-format json mixed_protocols.pcap | jq '.analyzers.reassembly'
```

Check:
- `packets_skipped_non_tcp`: should equal count of UDP + ICMP packets in capture.
- `bytes_reassembled`: should equal the sum of TCP payload bytes successfully delivered
  to the stream handler (not full frame lengths).
- Presence of other expected keys in the reassembly stats map.

Cross-check `bytes_reassembled` by summing TCP data segment lengths from the pcap file
using a reference tool (tcpdump or Wireshark).

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): packets_skipped_non_tcp accurately counts
  all non-TCP packets; bytes_reassembled reflects only TCP payload bytes, not full frames.
- **Data integrity** (weight: 0.35): bytes_reassembled matches the sum of delivered TCP
  payload bytes verifiable from the pcap.
- **Edge case handling** (weight: 0.1): A capture with ONLY non-TCP packets produces
  bytes_reassembled=0 and packets_skipped_non_tcp = total packet count.
- **Performance** (weight: 0.1): No accumulation of non-TCP packets in memory.

## Edge Conditions

- A capture with zero TCP packets: all packets appear in packets_skipped_non_tcp; the
  reassembly engine still runs without error.
- A TCP packet with no data payload (pure ACK): does it increment bytes_reassembled? No —
  only delivered payload bytes count.
- bytes_reassembled must not count bytes buffered but not yet flushed; only delivered bytes.

## Failure Guidance

"HOLDOUT LOW: HS-012 (satisfaction: 0.XX) — packets_skipped_non_tcp count is wrong,
bytes_reassembled includes non-TCP bytes, or reassembly AnalysisSummary is missing expected keys."
