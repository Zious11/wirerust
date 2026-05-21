---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-086.md
  - .factory/stories/STORY-087.md
  - .factory/stories/STORY-088.md
  - .factory/stories/STORY-089.md
  - .factory/stories/STORY-090.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.018.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.019.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.020.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.021.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-086.md
id: "HS-089"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-9"
behavioral_contracts:
  - BC-2.12.018
  - BC-2.12.019
  - BC-2.12.020
  - BC-2.12.021
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Summary Accumulates Correct Counts; unique_hosts Is Sorted and Deduplicated

## Scenario

The summary block that appears in every output format must accurately reflect the totals
from the ingested packets — and must not include skipped packets in any count. The unique-
hosts list must be sorted and deduplicated regardless of packet arrival order.

1. A pcap with exactly 10 packets is processed, where:
   - 3 packets are between host A and host B (bidirectional)
   - 2 packets are between host B and host C
   - 5 packets are between host A and host C
   - All packets use port 80 (HTTP-like)

2. After processing:
   - `total_packets` in the output is exactly 10.
   - `total_bytes` is the sum of all packet_len values (not the sum of capture lengths).
   - The unique hosts list contains exactly 3 entries: hosts A, B, and C — deduplicated.
   - The unique hosts list is sorted (not in insertion order).
   - `services["HTTP"]` appears in the output (port 80 implies HTTP service hint).
   - `skipped_packets` is 0 (no decode errors in this scenario).

3. A separate observation: `skipped_packets` is set by the pipeline AFTER the packet loop,
   not incremented during packet ingestion. Confirming this by adding decode errors to the
   scenario: even with 3 decode errors, `total_packets` counts only the successfully decoded
   packets, and `skipped_packets` = 3.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.12.018 | Postcondition 1-4: counts incremented correctly | total_packets, total_bytes, hosts, protocols |
| BC-2.12.018 | Invariant 2: skipped_packets not set by ingest | Decode errors do not inflate total_packets |
| BC-2.12.019 | Postcondition 1: port-based service hints | Port 80 → services["HTTP"] |
| BC-2.12.020 | Postcondition 1: unique_hosts sorted and deduplicated | 3 unique hosts in sorted order |
| BC-2.12.021 | Postcondition 1: JSON has total_packets, total_bytes, skipped_packets as u64 | JSON output fields |

## Verification Approach

Option 1 (unit test): Directly construct a `Summary`, call `ingest` 10 times with known
packet values, then assert:
- `summary.total_packets == 10`
- `summary.total_bytes == sum_of_packet_lens`
- `summary.unique_hosts() == [host_a, host_b, host_c].sorted()`
- `summary.service_counts()["HTTP"] == 10` (all packets on port 80)
- `summary.skipped_packets == 0`

Option 2 (integration): Run `wirerust analyze <10-packet.pcap> --json` and parse the JSON output.
Assert the `summary` object contains:
- `"total_packets": 10`
- `"total_bytes": <expected_sum>`
- `"skipped_packets": 0`
- `"services": {"HTTP": 10}` (or similar)

Option 3 (unique_hosts ordering): Ingest packets with IPs in non-sorted insertion order
(e.g., 10.0.0.3 before 10.0.0.1) and assert that `unique_hosts()` returns them in sorted order.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): All count fields match expected values; unique_hosts is sorted.
- **Edge case handling** (weight: 0.2): src_ip == dst_ip produces one host (not two); empty Summary gives empty unique_hosts.
- **Error quality** (weight: 0.1): No panic on edge cases.
- **Performance** (weight: 0.05): unique_hosts() sorting is reasonable even for large host sets.
- **Data integrity** (weight: 0.2): skipped_packets is set by caller after loop; ingest itself never modifies it.

## Edge Conditions

- Packet where `src_ip == dst_ip` (loopback): host appears once in `unique_hosts()`.
- Mix of IPv4 and IPv6: both appear in `unique_hosts()`; IPv4 sorts before IPv6 (Rust IpAddr ordering).
- Port not in the service hint table: `services` map entry is absent for that port.
- `packet_len = 0`: `total_bytes` unchanged (incremented by 0, not by 1).

## Failure Guidance

"HOLDOUT LOW: HS-089 (satisfaction: 0.XX) -- Summary counts were incorrect (wrong total_packets, wrong total_bytes), unique_hosts was not sorted or contained duplicates, or skipped_packets was incorrectly included in total_packets."
