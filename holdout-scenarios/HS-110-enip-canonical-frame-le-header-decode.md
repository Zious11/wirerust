---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.001.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.002.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.003.md
  - .factory/stories/STORY-130.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-110"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-20"
behavioral_contracts:
  - BC-2.17.001
  - BC-2.17.002
  - BC-2.17.003
verification_properties:
  - VP-032
lifecycle_status: active
introduced: v0.11.0-feature-enip
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
canonical_frame_holdout: true
fixture_needed: true
fixture_note: "Requires crafted pcap fixture: one TCP flow on port 44818 delivering the 28-byte byte sequence in Case A below. Flag as fixture-creation obligation for F4."
---

# Holdout Scenario: EtherNet/IP Canonical Frame — 24-Byte Header Little-Endian Decode and Endianness Regression Guard

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

> **DF-CANONICAL-FRAME-HOLDOUT-001 COMPLIANCE:** This is the mandatory canonical-frame
> holdout for the EtherNet/IP analyzer. It pins the little-endian decode of the 24-byte
> ENIP encapsulation header for a real-world SendRRData frame on TCP/44818. A big-endian
> regression would cause command=0x6F00 (unknown) instead of 0x006F, silently discarding
> the frame. This scenario catches that regression.

## Scenario

A crafted PCAP file is presented containing a single TCP flow on port 44818. The TCP payload
carries a minimal but protocol-correct EtherNet/IP SendRRData frame encoded as the following
28-byte byte sequence:

```
Bytes 0-23 (ENIP 24-byte encapsulation header, all fields LE):
  6F 00   -- command = 0x006F (SendRRData), LE: byte0=0x6F, byte1=0x00
  04 00   -- length  = 0x0004 (4 bytes payload after header), LE
  01 00 00 00  -- session_handle = 0x00000001, LE (bytes 4-7: 01 00 00 00 → 0x00000001)
  00 00 00 00  -- status = 0x00000000 (success), LE (bytes 8-11)
  AA BB CC DD EE FF 00 11  -- sender_context (bytes 12-19, opaque 8 bytes)
  00 00 00 00  -- options = 0x00000000, LE (bytes 20-23)

Bytes 24-27 (minimal CPF payload — 4 bytes as declared by length field):
  00 00 00 00
```

### Case A — Correct Little-Endian Decode (Primary Guard)

1. The user runs: `wirerust analyze enip_canonical_sendrr.pcap --enip --json`
2. The tool exits 0.
3. The JSON output contains ENIP analyzer summary data. The evaluator confirms:
   - The tool did not emit any parse error referencing this frame.
   - At minimum 1 EtherNet/IP PDU was processed (the summary `pdu_count` or equivalent
     reflects at least 1 frame analyzed — it must NOT be 0).
   - No finding of type "structural anomaly" (T0814 or equivalent) is emitted for this
     flow — this is a well-formed frame and must not be treated as malformed.
4. **Endianness guard:** If the ENIP command bytes `[0x6F, 0x00]` were decoded as
   big-endian, the result would be command=0x6F00 (unknown), which would fail the
   validity gate and increment `parse_errors` without incrementing `pdu_count`. The
   evaluator MUST verify that pdu_count >= 1 (not 0) AND that no structural-anomaly
   finding is emitted for this frame. These two conditions together confirm the
   little-endian decode path is taken.

### Case B — session_handle Little-Endian Confirmation

The ENIP header bytes 4-7 are `01 00 00 00`. Correct LE decode: session_handle =
`u32::from_le_bytes([0x01, 0x00, 0x00, 0x00])` = 0x00000001. Big-endian decode would
produce session_handle = 0x01000000 (≠ 0x00000001). While the session handle is not
directly visible in the CLI JSON summary output, the evaluator uses the Case A pdu_count
evidence as the proxy: if the LE decode is correct, the frame is accepted; if BE, it is
rejected. The session_handle value disambiguation is a hidden traceability note for our
records.

### Case C — Truncated Header Rejected (Negative Control)

1. A second crafted PCAP is presented: a TCP flow on port 44818 carrying a 23-byte payload
   (one byte short of the minimum 24-byte ENIP header).
2. The user runs: `wirerust analyze enip_truncated_23.pcap --enip --json`
3. The tool exits 0 (no panic, no crash).
4. The evaluator confirms: pdu_count = 0 (the partial header is stashed in carry or
   discarded; no frame is emitted as analyzed). No panic. No structural-anomaly finding
   (a single short fragment stashed in carry does not trigger T0814 — the T0814 threshold
   requires >= 3 malformed-frame events, not a single partial-header stash).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.17.001 | Postcondition 1 — parse_enip_header returns None for data.len() < 24 | Case C: 23-byte input rejected; no panic |
| BC-2.17.002 | Postcondition 1 — parse_enip_header returns Some for data.len() >= 24 | Case A: 28-byte input accepted; pdu_count >= 1 |
| BC-2.17.002 | Postcondition 2 — command = u16::from_le_bytes([data[0], data[1]]) | Case A: bytes [0x6F, 0x00] → command=0x006F (LE), not 0x6F00 (BE) |
| BC-2.17.002 | Postcondition 4 — session_handle = u32::from_le_bytes(data[4..8]) | Case B: bytes [0x01,0x00,0x00,0x00] → 0x00000001 (LE), not 0x01000000 (BE) |
| BC-2.17.003 | is_valid_enip_frame returns true for command 0x006F | Case A: SendRRData is a known-valid command; passes validity gate |

<!-- HIDDEN TRACEABILITY: BC-2.17.002 EC-003 (command=0x006F, length=0x0004 canonical vector) -->

## Fixture Creation Obligation

**F4 must create two pcap fixtures:**

1. `enip_canonical_sendrr.pcap` — single TCP flow, source port arbitrary, destination port
   44818; one TCP data segment containing the 28 bytes above (24-byte header + 4-byte CPF
   payload); standard Ethernet/IPv4 encapsulation; pcap link type 1.

2. `enip_truncated_23.pcap` — single TCP flow, destination port 44818; one TCP data
   segment containing exactly 23 bytes (e.g., the first 23 bytes of the header above:
   `6F 00 04 00 01 00 00 00 00 00 00 00 AA BB CC DD EE FF 00 11 00 00 00`).

Both fixtures can be crafted with scapy or a raw pcap writer. No live network capture is
required.

## Verification Approach

```bash
wirerust analyze enip_canonical_sendrr.pcap --enip --json
# Observe: exit 0; JSON output contains "pdu_count": N where N >= 1;
# no "T0814" or "structural anomaly" finding in the findings array.

wirerust analyze enip_truncated_23.pcap --enip --json
# Observe: exit 0; no panic; JSON output contains "pdu_count": 0 (or field absent);
# no T0814 finding (single carry-stash does not trigger the 3-event threshold).
```

If `wirerust analyze` does not expose `pdu_count` in JSON, the evaluator uses:
- Case A: zero findings of category `structural_anomaly` for the flow, AND no
  error message referencing this flow on stderr.
- Case C: tool completes without panic; no crash report.

## Evaluation Rubric

- **Endianness correctness** (weight: 0.50): Case A pdu_count >= 1 AND no structural-anomaly
  finding — confirms LE decode accepted the frame. Failure here is a CRITICAL regression.
- **Truncation safety** (weight: 0.20): Case C exits 0 with no panic and pdu_count = 0.
- **Validity gate correctness** (weight: 0.20): Case A produces no T0814 finding — the frame
  is structurally valid and the validity gate accepted it.
- **JSON coherence** (weight: 0.10): JSON output is well-formed and parseable.

## Edge Conditions

- The `[0x6F, 0x00]` command bytes are the canonical big-endian vs. little-endian
  discriminant: 0x006F (LE, SendRRData — valid) vs. 0x6F00 (BE — unknown command, rejected).
  No other command has this property as cleanly.
- The 4-byte CPF payload (all zeros) is deliberately minimal. It will likely result in a CPF
  parse error or empty item walk — but this does NOT produce a T0814 finding (T0814 requires
  the structural-reject count to reach MALFORMED_ANOMALY_THRESHOLD = 3). The evaluator must
  not confuse a CPF-layer parse issue with a T0814 structural anomaly.
- Case C (23-byte payload) exercises the carry-buffer stash path. Since the flow ends with
  the partial frame (no subsequent data arrives), the carry contents are discarded at
  flow close — this is correct behavior and produces no finding.

## Failure Guidance

"HOLDOUT FAIL: HS-110 (satisfaction: 0.XX) — EtherNet/IP canonical frame endianness
regression. If Case A pdu_count = 0 or a T0814 structural-anomaly finding is emitted for
the canonical frame, the ENIP header command field is being decoded big-endian (0x6F00 is
an unknown command). Verify parse_enip_header uses u16::from_le_bytes([data[0], data[1]]),
not u16::from_be_bytes. See BC-2.17.002 Postcondition 2. If Case C panics, see BC-2.17.001
Postcondition 2 (no bytes accessed for len < 24)."
