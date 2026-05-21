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
  - .factory/stories/STORY-003.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.007.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.008.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.009.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
id: "HS-022"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.02.007
  - BC-2.02.008
  - BC-2.02.009
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Decoder No-Panic Safety — Malformed and Truncated Packets

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. An analyst receives a pcap from an unknown source that may contain adversarially crafted
   packets with malformed headers: truncated Ethernet frames (fewer than 14 bytes), IPv4
   headers with impossible IHL values, and TCP segments with no payload.
2. wirerust processes the entire pcap without panicking at any point.
3. Each malformed packet is counted in the skipped_packets or decode-error counter — the
   analyst can see how many packets were unprocessable.
4. Valid packets interspersed with the malformed ones are still processed correctly.
5. The tool exits with exit code 0 if any valid analysis was possible, or a non-zero code
   only if the entire file was unprocessable.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.02.007 | Postcondition 1 — malformed input bytes return Err (no panic); never reaches unwrap/panic branch | Steps 2-3: no panic on malformed bytes |
| BC-2.02.008 | Postcondition 1 — unsupported link type rejected in decode_packet | Step 3 edge: known link type, unknown sub-frame format |
| BC-2.02.009 | Postcondition 1 — "No IP layer found" error surfaced without panic | Step 3: non-IP Ethernet frames handled |

## Verification Approach

Craft or use a fuzzing-derived pcap with intentionally malformed packets:

```
wirerust analyze --output-format json malformed_mix.pcap | jq '{
  total_packets: .summary.total_packets,
  skipped_packets: .summary.skipped_packets,
  valid_findings: (.findings | length)
}'
```

Verify:
- skipped_packets > 0 (some packets could not be decoded)
- total_packets > 0 (file was opened successfully)
- No Rust panic message on stderr (no "thread 'main' panicked")
- If any valid TCP packets existed: some analysis output present

For the all-malformed case (every packet is garbage):
```
wirerust analyze all_malformed.pcap
```
Expect: exit non-zero OR exit 0 with skipped_packets == total_packets.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): No panic; malformed packets counted as skipped;
  valid packets still analyzed.
- **Error quality** (weight: 0.3): skipped_packets counter accurately reflects the number
  of undecodable packets; the difference between total_packets and skipped_packets is the
  number of successfully decoded packets.
- **Edge case handling** (weight: 0.15): Truncated file (last packet cut mid-bytes) handled
  without panic.
- **Data integrity** (weight: 0.1): Valid packets interleaved with malformed ones are still
  processed; no "contamination" from a malformed packet corrupts subsequent ones.

## Edge Conditions

- A packet with valid Ethernet and IP headers but a truncated TCP segment (fewer bytes than
  TCP header IHL claims): should be skipped, not panic.
- A packet with an IPv4 IHL field that claims more bytes than the frame length provides:
  etherparse or similar library should return Err, which should be counted as skipped.
- A zero-length pcap packet record: skip without crash.

## Failure Guidance

"HOLDOUT LOW: HS-022 (satisfaction: 0.XX) — wirerust panics on malformed or truncated
packets, or fails to count skipped packets accurately."
