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
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.001.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.004.md
input-hash: "db1dde0"
traces_to: .factory/specs/prd.md
id: "HS-001"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.01.001
  - BC-2.01.004
lifecycle_status: stale
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: "pcapng-rejection expectation (Step 5, BC-2.01.004) inverted by F2 pcapng-reader-support (BC-2.01.009 now accepts pcapng). Scenario rewrite is F3 scope (STORY-127)."
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: PCAP Link-Type Boundary — Accepted vs. Rejected at File Open

> **[STALE — 2026-06-19] This scenario's pcapng-rejection expectation (Step 5 below, and the
> BC-2.01.004 table row) is INVERTED by the F2 pcapng-reader-support feature (ADR-009). As of
> BC-2.01.009, pcapng is now an ACCEPTED input format — the `sample.pcapng` file in Step 5
> should be ACCEPTED (exit 0), not rejected. BC-2.01.004 is RETIRED. This scenario must be
> fully rewritten in F3 when STORY-127 is decomposed. Until then, do NOT use this scenario as
> a gate for pcapng rejection behavior.**

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A user has three capture files: one classic `.pcap` with Ethernet framing (link type 1),
   one classic `.pcap` with an IEEE 802.11 WiFi link type (link type 105), and one file in
   pcapng format (link type embedded in pcapng block headers).
2. The user runs `wirerust analyze` on each file in turn.
3. The Ethernet capture is accepted; analysis proceeds and the tool exits cleanly.
4. The 802.11 capture is rejected immediately with a human-readable error message identifying
   the link type as unsupported; the tool exits non-zero without reading packet data.
5. The pcapng file is rejected at the reader level with a human-readable error before any
   packet decoding begins; the tool exits non-zero.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.001 | Postcondition 2 — unsupported link type returns Err containing "Unsupported pcap link type" | Steps 3-4: verified that the 802.11 file is rejected with the right error signal |
| BC-2.01.004 | Postcondition 1 — pcapng format rejected at reader level before packet loop | Step 5: verifies pcapng is distinguished from classic pcap and rejected early |

## Verification Approach

Run each of the three files through the CLI:

```
wirerust analyze ethernet.pcap
wirerust analyze wifi80211.pcap
wirerust analyze sample.pcapng
```

For `ethernet.pcap`: observe exit code 0, findings/summary present.
For `wifi80211.pcap`: observe non-zero exit code, stderr message contains text that
communicates "unsupported" or "link type"; no findings emitted to stdout.
For `sample.pcapng`: observe non-zero exit code, error on stderr before any packet data;
no findings emitted to stdout.

A crafted test pcap with link-type field set to 105 bytes is sufficient for the 802.11 case.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Ethernet accepted, 802.11 and pcapng both rejected.
- **Edge case handling** (weight: 0.2): pcapng rejection happens before packet-loop entry (not
  after attempting to parse malformed packets).
- **Error quality** (weight: 0.2): Rejection error messages are human-readable and reference
  the problematic format/link type.
- **Data integrity** (weight: 0.1): No partial output emitted before rejection; exit code
  correctly reflects error.

## Edge Conditions

- pcapng magic number differs from classic pcap; tool must not fall into the packet-read loop.
- Link type 105 (IEEE 802.11) must be rejected even though the bytes are otherwise valid pcap.
- Link type 101 (RAW) and link type 228 (IPV4) must both be accepted; tested separately.

## Failure Guidance

"HOLDOUT LOW: HS-001 (satisfaction: 0.XX) — link-type gating or pcapng rejection is not
working correctly at the file-open boundary."
