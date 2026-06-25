---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.011.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.012.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.013.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.014.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.005.md
  - .factory/stories/STORY-135.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-119"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-20"
behavioral_contracts:
  - BC-2.17.011
  - BC-2.17.012
  - BC-2.17.013
  - BC-2.17.014
  - BC-2.17.005
lifecycle_status: active
introduced: v0.11.0-feature-enip
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Requires crafted pcap: TCP flow on port 44818 with ENIP SendRRData frame carrying CPF 0x00B1 (Connected Data Item, NOT 0x00B2) with CIP Stop service byte at position that would be read as service byte if it were 0x00B2. This tests that the 0x00B1 scope gate prevents the detection from firing."
---

# Holdout Scenario: CIP Stop in 0x00B1 Connected Item Does NOT Fire in v0.11.0 (Deferral Scope Gate)

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

In v0.11.0, all CIP service detections (T0858 Stop, T0816 Reset, T0836 Write Burst, T0888
Identity Read) apply ONLY to CIP services carried in CPF type_id 0x00B2 (Unconnected Data
Item). CIP services carried in CPF type_id 0x00B1 (Connected Data Item) are explicitly
deferred to v0.12.0 (F-P9-001 locked decision Option A).

The 0x00B1 scope gate exists because Connected Data Items prepend a 2-byte CIP
"connected sequence count" before the CIP PDU. Bytes 0-1 of the item data are the
sequence count, not the CIP service byte. If the analyzer attempted to read item_data[0]
as a CIP service byte on a 0x00B1 item, it would read the low byte of the sequence count
and produce a wrong or spurious classification.

This holdout verifies the scope gate: a CIP Stop (service=0x07) carried in a 0x00B1 item
must NOT produce a T0858 finding in v0.11.0.

### Case A — CIP Stop in 0x00B1 Does NOT Emit T0858 (v0.11.0 Scope Gate)

1. A crafted PCAP is presented: TCP flow on port 44818; ENIP SendRRData frame; CPF item
   with type_id=0x00B1 (Connected Data Item). The item data begins with:
   - Bytes 0-1: 0x01 0x00 (connected sequence count = 1, LE)
   - Byte 2: 0x07 (CIP service byte — this would be Stop if interpreted as CIP service,
     but it's at position 2, not 0 in item_data)
   NOTE: If the analyzer correctly applies the 0x00B2-only gate, it will NOT parse the CIP
   service from this 0x00B1 item, and no T0858 finding is emitted.
2. The user runs: `wirerust analyze enip_stop_in_b1.pcap --enip --json`
3. The tool exits 0.
4. The evaluator confirms: ZERO T0858 findings in the JSON output.
5. No panic. No crash. The 0x00B1 item is either silently skipped or logged without a
   CIP-service detection.

### Case B — CIP Reset in 0x00B1 Does NOT Emit T0816

1. Second PCAP: same structure but the byte at position 2 of item data is 0x05 (CIP Reset).
2. The evaluator confirms: ZERO T0816 findings.

### Case C — CIP SetAttributeSingle in 0x00B1 Does NOT Increment Write Counter

1. Third PCAP: 51 ENIP SendRRData frames, each with CPF 0x00B1 item; position 2 of item
   data = 0x10 (CIP SetAttributeSingle). All within 1 second.
2. The evaluator confirms: ZERO T0836 findings (write-burst counter must not be incremented
   for 0x00B1 items, so 51 frames does not trigger the burst even if bytes are read).
3. No panic.

### Case D — Positive Control: CIP Stop in 0x00B2 Still Fires T0858

A fourth PCAP uses the same CIP Stop byte (0x07) but now in a 0x00B2 item (byte 0 of item
data is 0x07, no sequence-count prefix). The evaluator confirms: ONE T0858 finding. This
confirms the 0x00B2 path is working and the 0x00B1 suppression is selective, not broken.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.17.011 | Precondition 3 — 0x00B1 items NOT processed for CIP Stop in v0.11.0 | Case A: 0x00B1 → zero T0858 |
| BC-2.17.013 | Precondition 3 — 0x00B1 items NOT processed for CIP Reset | Case B: 0x00B1 → zero T0816 |
| BC-2.17.012 | Precondition 3 — 0x00B1 items NOT processed for write-burst counting | Case C: 0x00B1 → zero T0836 |
| BC-2.17.005 | CPF item-walk: type_id gate distinguishes 0x00B1 from 0x00B2 | Cases A/B/C: 0x00B1 items not passed to CIP parser |
| BC-2.17.011 | Postcondition 1 — T0858 fires on 0x00B2 carrier | Case D: 0x00B2 → one T0858 (positive control) |

<!-- HIDDEN TRACEABILITY: F-P9-001 locked decision Option A (0x00B1 deferral to v0.12.0); all four detection BCs share Precondition 3 -->

## Fixture Creation Obligation

**F4 must create:**
1. `enip_stop_in_b1.pcap` — TCP flow dst port 44818; ENIP SendRRData + CPF 0x00B1 item
   (type_id=0x00B1), item data = [0x01, 0x00, 0x07, ...] (sequence count prefix + Stop byte).
2. `enip_reset_in_b1.pcap` — Same with byte at position 2 = 0x05 (Reset).
3. `enip_write_burst_51_b1.pcap` — 51 ENIP frames with 0x00B1 items (position 2 = 0x10,
   SetAttributeSingle) within 1 second.
4. `enip_stop_in_b2_positive.pcap` — ENIP SendRRData + CPF 0x00B2 item, item_data[0]=0x07
   (positive control). This is the same fixture as used in HS-111 Case A.

## Verification Approach

```bash
wirerust analyze enip_stop_in_b1.pcap --enip --json
# Expect: exit 0; ZERO T0858 findings. (0x00B1 scope gate active.)

wirerust analyze enip_reset_in_b1.pcap --enip --json
# Expect: exit 0; ZERO T0816 findings.

wirerust analyze enip_write_burst_51_b1.pcap --enip --json
# Expect: exit 0; ZERO T0836 findings (write counter not incremented for 0x00B1).

wirerust analyze enip_stop_in_b2_positive.pcap --enip --json
# Expect: exit 0; ONE T0858 finding. (Positive control.)
```

## Evaluation Rubric

- **0x00B1 scope gate: Stop** (weight: 0.30): Case A produces zero T0858.
- **0x00B1 scope gate: Reset** (weight: 0.20): Case B produces zero T0816.
- **0x00B1 scope gate: Write burst** (weight: 0.20): Case C produces zero T0836 even
  for 51 frames.
- **Positive control** (weight: 0.20): Case D produces one T0858 on 0x00B2.
- **No panic** (weight: 0.10): All cases exit 0 without panic.

## Failure Guidance

"HOLDOUT FAIL: HS-119 — 0x00B1 scope gate not applied. If Cases A/B/C produce T0858/
T0816/T0836 findings for 0x00B1 items, the CPF item-walk passes 0x00B1 item data to the
CIP service classifier without checking type_id first. The gate must check: if type_id !=
0x00B2, skip CIP service processing for v0.11.0. The 0x00B1 connected-item prefix contains
a 2-byte sequence count before the CIP PDU — misreading it as a CIP service byte produces
wrong service classification. See BC-2.17.011/012/013/014 Precondition 3, and BC-2.17.005
item-type gate. The 0x00B1 deferral is a mandatory v0.11.0 scope boundary (F-P9-001)."
