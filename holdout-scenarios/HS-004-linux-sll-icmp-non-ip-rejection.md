---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-003.md
  - .factory/stories/STORY-004.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.006.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.009.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.010.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.011.md
input-hash: "a9b8cc0"
traces_to: .factory/specs/prd.md
id: "HS-004"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.02.006
  - BC-2.02.009
  - BC-2.02.010
  - BC-2.02.011
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Linux SLL Cooked Capture, ICMP Classification, and Non-IP Frame Handling

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A user has a pcap captured on a Linux loopback interface (link type 113, Linux cooked / SLL).
   The capture contains a TCP connection.
2. wirerust processes the capture and extracts TCP flows correctly — the SLL header is stripped
   transparently and the TCP data is processed as normal.
3. A second capture contains a mix of TCP, UDP, and ICMP packets on an Ethernet link.
4. ICMP packets are consumed without error; they do not appear as TCP flows, do not crash
   the pipeline, and are counted in summary statistics as skipped by the reassembly engine.
5. A packet with an IP protocol byte other than TCP/UDP/ICMP (e.g., protocol 89 — OSPF)
   is processed without panic; the packet is skipped or classified as Protocol::Other.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.02.006 | Postcondition 1 — Linux SLL TCP decoded to ParsedPacket | Step 2: SLL-framed capture produces correct output |
| BC-2.02.009 | Postcondition 1 — "No IP layer found" error surfaces; no panic | Step 4-5 edge: frames without IP layer |
| BC-2.02.010 | Postcondition 1 — ICMP classified as Protocol::Icmp with TransportInfo::None | Step 4: ICMP classification |
| BC-2.02.011 | Postcondition 1 — other protocols classified as Protocol::Other(byte) | Step 5: OSPF or similar non-IP protocol handling |

## Verification Approach

```
wirerust analyze --output-format json loopback_linux_sll.pcap
```

Expect: TCP flows visible in JSON output; summary shows flows processed correctly.

```
wirerust analyze --output-format json mixed_tcp_udp_icmp.pcap
```

Expect: exit 0; ICMP packets appear in summary stats (packets_skipped_non_tcp > 0);
no ICMP-based findings; TCP flows present if any TCP packets exist.

For OSPF or protocol-89 packets: no panic; packet counted as skipped.

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): SLL capture produces correct TCP flow output;
  ICMP packets handled without error.
- **Edge case handling** (weight: 0.3): Non-IP and unknown-protocol frames are silently
  skipped or correctly classified without crashing.
- **Data integrity** (weight: 0.2): packets_skipped_non_tcp counter reflects the actual
  number of non-TCP packets in the mixed capture.
- **Error quality** (weight: 0.1): Any error messages are descriptive and scoped to the
  specific packet, not the entire file.

## Edge Conditions

- SLL header is 16 bytes, not 14 (Ethernet); if the parser uses a fixed offset, it will
  misread the IP layer — this is the core correctness risk.
- ICMP packets that look like TCP SYN on a naive byte-level check should not create spurious flows.
- IP packets with no transport layer (e.g., pure ICMP, OSPF) should not crash the TCP extractor.

## Failure Guidance

"HOLDOUT LOW: HS-004 (satisfaction: 0.XX) — Linux SLL decoding, ICMP classification, or
non-IP frame handling fails or panics."
