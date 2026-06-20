---
document_type: holdout-scenario
level: ops
version: "2.0"  # 2026-06-20 — FULL REWRITE per F3/STORY-127. Prior v1.0 encoded pcapng-REJECTION (BC-2.01.004, retired). Now encodes pcapng-ACCEPTANCE per BC-2.01.009 (ADR-009 rev 9): a valid pcapng file is detected via magic-byte probe, routed to the pcapng reader, and its packets are analyzed to completion. The 802.11 link-type rejection (BC-2.01.001 Step 4 — unsupported link-type) is preserved unchanged. BC-2.01.004 reference removed (retired). BC-2.01.009 added to behavioral_contracts and inputs.
status: draft
producer: product-owner
timestamp: 2026-06-20T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-001.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.001.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.009.md
  - .factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md
input-hash: "946cb06"
traces_to: .factory/specs/prd.md
id: "HS-001"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.01.001
  - BC-2.01.009
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: PCAP Link-Type Boundary — Accepted vs. Rejected at File Open

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A user has three capture files: one classic `.pcap` with Ethernet framing (link type 1),
   one classic `.pcap` with an IEEE 802.11 WiFi link type (link type 105), and one file in
   pcapng format containing at least one EPB with an Ethernet payload.
2. The user runs `wirerust analyze` on each file in turn.
3. The Ethernet classic-pcap capture is accepted; analysis proceeds and the tool exits
   cleanly with a summary including packet counts and any findings.
4. The 802.11 classic-pcap capture is rejected immediately with a human-readable error
   message identifying the link type as unsupported; the tool exits non-zero without reading
   packet data.
5. The pcapng file is ACCEPTED: the tool detects the pcapng magic bytes
   (`0x0A 0x0D 0x0D 0x0A`) via a non-destructive peek probe, routes to the pcapng reader,
   parses the file to completion, extracts the Ethernet packets contained in EPBs, and exits
   0 with analysis output. No error is emitted for the pcapng file.

The key behavioral inversion from the old (pre-F2) expectation: pcapng is now a
first-class accepted capture format. The tool must NOT reject a pcapng file at the reader
boundary; it must NOT produce a "format not supported" or similar error for pcapng input.
BC-2.01.004 (which specified pcapng rejection) is retired and superseded by BC-2.01.009.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.001 | Postcondition 2 — unsupported link type returns Err containing "Unsupported pcap link type" | Step 4: 802.11 classic-pcap file is rejected with the right error signal |
| BC-2.01.009 | Postcondition 1 — pcapng magic bytes route to pcapng parse path; returns Ok(PcapSource); packets contains RawPackets from EPBs in encounter order | Step 5: pcapng file is ACCEPTED, parsed, packets extracted; exit 0 |
| BC-2.01.009 | Postcondition 3 — probe consumes no bytes; both downstream parsers receive the full un-consumed stream from byte 0 | Step 5: pcapng reader receives the full stream; no "partial read" error before packet loop |

## Verification Approach

Run each of the three files through the CLI:

```
wirerust analyze ethernet.pcap
wirerust analyze wifi80211.pcap
wirerust analyze sample.pcapng
```

For `ethernet.pcap`: observe exit code 0, findings/summary present. This is the unchanged
baseline for classic-pcap Ethernet.

For `wifi80211.pcap`: observe non-zero exit code, stderr message contains text that
communicates "unsupported" or "link type"; no findings emitted to stdout. No change from
pre-F2 behavior.

For `sample.pcapng`: observe exit code 0. Stdout contains analysis output (JSON summary,
terminal report, or findings — depending on flags used). No error on stderr. The file
contains at least one EPB with an Ethernet payload; `total_packets` in the output is >= 1.
The tool must NOT produce any error about "format not supported," "pcapng not supported,"
or equivalent.

A crafted test file for the pcapng case: SHB (LE, 28 bytes) + IDB (linktype=1, 20 bytes) +
EPB with a minimal 14-byte Ethernet frame (block_total_length=48, captured_len=16 with 2
padding bytes). This is the simplest valid pcapng with one Ethernet packet.

## Evaluation Rubric

- **Functional correctness — pcapng acceptance** (weight: 0.45): pcapng file exits 0 with
  analysis output; no rejection error on stderr; `total_packets >= 1` in output confirms the
  pcapng reader ran and extracted packets. This is the primary gate for BC-2.01.009.
- **Link-type rejection preserved** (weight: 0.30): 802.11 classic-pcap is still rejected
  with a human-readable error; no regression in classic-pcap link-type gating.
- **Error quality** (weight: 0.15): The 802.11 rejection message is human-readable and
  references the problematic link type. The pcapng success path produces no spurious
  warnings on stderr.
- **Data integrity** (weight: 0.10): No partial output emitted before rejection (802.11
  case); pcapng output includes at least one packet from the EPB; exit codes correctly
  reflect success or error.

## Edge Conditions

- The pcapng magic `0x0A 0x0D 0x0D 0x0A` is distinct from all four classic-pcap magics
  (`0xA1B2C3D4`, `0xD4C3B2A1`, `0xA1B23C4D`, `0x4D3CB2A1`). The probe branch is
  deterministic: any file starting with `0A 0D 0D 0A` must take the pcapng path.
- The peek probe MUST NOT advance the stream. Both the classic-pcap and pcapng parsers
  receive the stream from byte 0. An implementation that consumes the first 4 bytes before
  branching will corrupt the stream and produce a parse error — the evaluator checks for
  exit 0 and at least one packet in output, which would fail if the stream was corrupted.
- Link type 105 (IEEE 802.11) must be rejected even though the bytes are otherwise valid
  classic pcap. Link type 101 (RAW) and link type 228 (IPV4) must both be accepted; tested
  separately in other scenarios.
- The pcapng file in this scenario uses little-endian byte order (BOM = `4D 3C 2B 1A`);
  big-endian pcapng behavior is covered by HS-103.

## Failure Guidance

"HOLDOUT LOW: HS-001 (satisfaction: 0.XX) — link-type gating or pcapng acceptance is not
working correctly at the file-open boundary.
pcapng case exit non-zero: the pcapng reader is absent or the magic-byte probe branches
incorrectly; BC-2.01.009 not implemented. Check that the probe detects 0A 0D 0D 0A and
routes to the pcapng parse path.
pcapng case exit 0 but total_packets = 0: pcapng reader runs but no packets extracted;
EPB parsing is absent or the IDB linktype is not registered before the EPB is processed.
pcapng case error on stderr: pcapng is still being rejected; BC-2.01.004 behavior has not
been inverted; verify BC-2.01.009 supersedes BC-2.01.004.
802.11 rejection absent (exit 0): link-type gate regressed; BC-2.01.001 postcondition 2
not enforced."
