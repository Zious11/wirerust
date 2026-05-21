---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-002.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.001.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.003.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.005.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.007.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
id: "HS-003"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.02.001
  - BC-2.02.003
  - BC-2.02.005
  - BC-2.02.007
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Ethernet, RAW IPv4, and IPv6 Link-Layer Decode Correctness

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A user has three pcap captures: one Ethernet-framed IPv4 TCP capture, one RAW-link-layer
   IPv4 TCP capture (link type 101), and one RAW-link-layer IPv6 TCP capture.
2. All three are processed by wirerust analyze.
3. For the Ethernet file, the source and destination IPv4 addresses visible in the
   output match what a hex dump of the pcap frame shows.
4. For the RAW IPv6 file, the output shows IPv6 addresses (colon-separated) rather than
   IPv4-dotted-decimal for those flows.
5. A fourth pcap contains deliberately truncated/malformed bytes at the Ethernet layer. The
   tool does not panic; the packet increments skipped_packets rather than crashing.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.02.001 | Postcondition 1 — Ethernet IPv4 TCP decoded to ParsedPacket with correct src/dst IP | Step 3: IP address fidelity |
| BC-2.02.003 | Postcondition 1 — RAW link-layer IPv4 TCP decoded via from_ip | Step 2-3: RAW path accepts and decodes correctly |
| BC-2.02.005 | Postcondition 1 — RAW IPv6 TCP surfaces IPv6 addresses | Step 4: IPv6 address display |
| BC-2.02.007 | Postcondition 1 — malformed bytes return Err (no panic) | Step 5: malformed input safety |

## Verification Approach

Use crafted or captured pcap files for each link-layer type:

```
wirerust analyze --output-format json ethernet_ipv4.pcap
wirerust analyze --output-format json raw_ipv4.pcap
wirerust analyze --output-format json raw_ipv6.pcap
wirerust analyze --output-format json malformed_ethernet.pcap
```

For IPv4 captures: verify that source/destination IPs in JSON output match expected
dotted-decimal addresses. For IPv6 capture: verify addresses contain colons and match
the capture content. For malformed capture: verify exit is non-zero or skipped_packets > 0,
no panic.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): All three valid link-layer types decode without
  error; addresses match the capture content.
- **Edge case handling** (weight: 0.25): Malformed input does not panic; packet is counted
  as skipped.
- **Data integrity** (weight: 0.2): IPv6 addresses are formatted correctly (colon notation);
  IPv4 addresses are dotted-decimal.
- **Error quality** (weight: 0.1): Any decode error message is descriptive.

## Edge Conditions

- RAW (link type 101) and IPV4 (link type 228) should produce identical decoded output for
  the same IPv4 payload (BC-2.02.004 related).
- An IPv6 extension-header packet should not crash the decoder even if not fully supported.
- A frame shorter than the minimum Ethernet header should be counted as skipped, not crash.

## Failure Guidance

"HOLDOUT LOW: HS-003 (satisfaction: 0.XX) — one or more link-layer decode paths (Ethernet,
RAW IPv4, RAW IPv6) does not produce correct output or panics on malformed input."
