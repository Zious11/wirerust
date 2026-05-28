---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-004.md
  - .factory/stories/STORY-005.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.012.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.014.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.015.md
input-hash: "f08c837"
traces_to: .factory/specs/prd.md
id: "HS-005"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.02.012
  - BC-2.02.014
  - BC-2.02.015
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: App Protocol Hints, Frame Length Accounting, and TCP Flag Extraction

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A user runs wirerust summary on a pcap containing DNS traffic on port 53 (UDP), HTTP
   traffic on port 80, HTTPS traffic on port 443, and an unrecognized port (port 9999).
2. The summary output lists service labels ("dns", "http", "https") for the recognized ports;
   port 9999 traffic appears without a service label or under a generic category.
3. The total_bytes reported in the summary matches the sum of the full frame lengths (including
   Ethernet header, IP header, and transport header) — NOT just the payload sizes.
4. A separate capture contains a TCP SYN packet with the SYN flag set. Another packet is a
   SYN+ACK. When analyzed, the reassembly engine correctly tracks the handshake (no finding
   about a missing handshake for a normal TCP connection).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.02.012 | Postcondition 1 — app_protocol_hint returns "dns" for port 53, "http" for 80, "https" for 443 | Step 2: service labeling in summary |
| BC-2.02.014 | Postcondition 1 — packet_len is total frame length, not just payload | Step 3: total_bytes accounting uses frame length |
| BC-2.02.015 | Postcondition 1 — TCP control flags (SYN, ACK, RST, FIN) extracted into TransportInfo::Tcp | Step 4: handshake flags drive state machine correctly |

## Verification Approach

```
wirerust summary --output-format json mixed_services.pcap
```

Check JSON output `services` map contains keys "dns", "http", "https" with non-zero counts.
Check `total_bytes` equals the sum of pcap `incl_len` fields (full frame lengths).

```
wirerust analyze --output-format json normal_handshake.pcap
```

Check that a standard TCP connection produces zero TCP-reassembly findings for a clean session.

Manually verify: pick one frame from the pcap, compute its frame length, confirm that value
is included in `total_bytes` (not the IP payload length).

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): Known service ports produce correct labels in
  summary; unrecognized ports do not produce spurious labels.
- **Data integrity** (weight: 0.35): total_bytes equals the sum of full frame lengths, not
  just payload bytes — this is verifiable by computing the sum from the pcap file directly.
- **Edge case handling** (weight: 0.15): Unknown port (9999) handled without error; no label
  or "unknown" label is both acceptable.
- **Performance** (weight: 0.1): Summary completes in reasonable time for a moderately sized
  capture (>1000 packets).

## Edge Conditions

- A packet with src_port=53 AND dst_port=80: which hint wins? App_protocol_hint resolves
  the first matching port in the table — the exact tie-breaking rule is an implementation detail.
- A UDP packet to port 443: may hint as "https" even though it's UDP; the hint is based
  purely on port, not transport protocol.
- A packet where IP total_length < actual bytes captured: packet_len should reflect the
  captured (incl_len) value, not the inner IP length.

## Failure Guidance

"HOLDOUT LOW: HS-005 (satisfaction: 0.XX) — service hint labeling, frame-length byte
accounting, or TCP flag extraction produces incorrect results."
