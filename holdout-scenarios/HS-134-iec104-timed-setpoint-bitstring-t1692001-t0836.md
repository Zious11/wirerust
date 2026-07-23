---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-23T00:00:00Z
phase: f3
inputs: []
input-hash: "d41d8cd"
traces_to: .factory/specs/prd.md
id: "HS-134"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "feature-iec104"
behavioral_contracts:
  - BC-2.19.030
  - BC-2.19.017
verification_properties:
  - VP-047
lifecycle_status: active
introduced: wave-85-spec-evolution
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: IEC104-TIMED-CMD-GAP-001
fixture_needed: true
fixture_note: "Requires crafted pcap fixture: TCP flow on port 2404 delivering four IEC-104 I-frames with TypeIDs 61, 62, 63, 64. See Fixture Creation Obligation below."
---

# Holdout Scenario: IEC-104 Time-Tagged Set-Point + Bitstring Commands (TypeIDs 61–64) Emit T1692.001 + T0836

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

> **IEC104-TIMED-CMD-GAP-001 CLOSURE:** This scenario verifies that time-tagged set-point and
> bitstring TypeIDs 61–64 (C_SE_TA_1/C_SE_TB_1/C_SE_TC_1/C_BO_TA_1) emit both T1692.001 and
> T0836 — exactly matching the untimed arm 48..=51 in BC-2.19.019. Prior to BC-2.19.030, zero
> findings were emitted for these TypeIDs, providing an evasion channel for parameter writes.

## Scenario

A crafted PCAP file is presented containing a single TCP flow on port 2404. The TCP payload
delivers four IEC-104 I-frames, each carrying an ASDU with a distinct TypeID.

**Frame A** — TypeID=61 (C_SE_TA_1, timed set-point normalized value):
APCI: `68 17 00 00 00 00`; ASDU: TypeID=0x3D, VSQ=0x01, COT=0x0006, CASDU=0x0001,
IOA=0x000064 (100), NVA=0x4000 (normalized value), QOS=0x00, CP56Time2a=7 zero bytes.

**Frame B** — TypeID=62 (C_SE_TB_1, timed set-point scaled value):
APCI: `68 17 02 00 00 00`; ASDU: TypeID=0x3E, VSQ=0x01, COT=0x0006, CASDU=0x0001,
IOA=0x000065 (101), SVA=0x0064 (100), QOS=0x00, CP56Time2a=7 zero bytes.

**Frame C** — TypeID=63 (C_SE_TC_1, timed set-point short float):
APCI: `68 19 04 00 00 00`; ASDU: TypeID=0x3F, VSQ=0x01, COT=0x0006, CASDU=0x0001,
IOA=0x000066 (102), R32-IEEE754=4 bytes (e.g., 0x00 0x00 0x48 0x42 = 50.0f), QOS=0x00,
CP56Time2a=7 zero bytes.

**Frame D** — TypeID=64 (C_BO_TA_1, timed bitstring 32 bits):
APCI: `68 18 06 00 00 00`; ASDU: TypeID=0x40, VSQ=0x01, COT=0x0006, CASDU=0x0001,
IOA=0x000067 (103), BSI=0xDEADBEEF (32 bits), CP56Time2a=7 zero bytes.
(No QOS field for C_BO_TA_1 per IEC 60870-5-101 Table 8 — bitstring commands carry no QOS.)

### Case A — TypeID=61 Emits T1692.001 + T0836

1. Run: `wirerust analyze iec104_timed_setpoint.pcap --iec104 --json`
2. Exit code: 0.
3. The JSON `findings` array contains AT LEAST one finding with `mitre_techniques: ["T1692.001"]`
   for TypeID=61, where `category=="impact"`, `verdict=="possible"`, `confidence=="medium"`.
4. The `findings` array ALSO contains AT LEAST one finding with `mitre_techniques: ["T0836"]`
   for TypeID=61, with the same verdict/confidence/category.
5. Both findings include `CASDU=1` in their `evidence` vectors.
6. Both summaries contain "time-tagged" OR "C_SE_TA" to distinguish from the untimed form.

### Case B — TypeID=62 Emits T1692.001 + T0836

Two findings emitted for TypeID=62: T1692.001 and T0836.

### Case C — TypeID=63 Emits T1692.001 + T0836

Two findings emitted for TypeID=63 (float set-point).

### Case D — TypeID=64 Emits T1692.001 + T0836

Two findings emitted for TypeID=64 (C_BO_TA_1 bitstring).

### Case E — Exactly 8 Findings for 4 Frames (2 Per ASDU)

When four back-to-back I-frames are delivered (TypeIDs 61, 62, 63, 64), the findings array
contains exactly 8 findings: 4 × T1692.001 + 4 × T0836, one pair per TypeID.

### Case F — Summaries Distinguish Timed Variant

All T1692.001 and T0836 summary strings contain "time-tagged" or a timed mnemonic
(C_SE_TA / C_SE_TB / C_SE_TC / C_BO_TA), NOT the untimed form (C_SE / C_BO without TA/TB/TC).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.19.030 | Postcondition 1 — T1692.001 Possible emitted | Cases A–D |
| BC-2.19.030 | Postcondition 2 — T0836 Possible co-emitted | Cases A–D |
| BC-2.19.030 | Postcondition 3 — CASDU/first_ioa in both evidence vectors | Case A evidence |
| BC-2.19.030 | Postconditions 4–5 — summaries distinguish timed variants | Case F |
| BC-2.19.030 | Invariant 1 — both findings always emitted for 61–64 | Cases A–D |
| BC-2.19.030 | Invariant 3 — count-independent | Case E (8 total) |
| BC-2.19.017 | Invariant 1 — cot_test [TEST] (COT=6, no [TEST] expected) | Cases A–D |

## Verification Approach

```bash
wirerust analyze iec104_timed_setpoint.pcap --iec104 --json | \
  jq '[.findings[] | select(.mitre_techniques[] == "T1692.001")] | length'
# Expect: 4 (one per TypeID 61/62/63/64)

wirerust analyze iec104_timed_setpoint.pcap --iec104 --json | \
  jq '[.findings[] | select(.mitre_techniques[] == "T0836")] | length'
# Expect: 4 (one per TypeID 61/62/63/64)

wirerust analyze iec104_timed_setpoint.pcap --iec104 --json | \
  jq '[.findings[]] | length'
# Expect: 8 (4 T1692.001 + 4 T0836)

wirerust analyze iec104_timed_setpoint.pcap --iec104 --json | \
  jq '.findings[] | .summary' | \
  grep -c "time-tagged\|C_SE_TA\|C_SE_TB\|C_SE_TC\|C_BO_TA"
# Expect: 8 (all summaries distinguish timed variant)
```

## Evaluation Rubric

- **T1692.001 emitted for TypeIDs 61–64** (weight: 0.35): 4 T1692.001 findings present.
- **T0836 emitted for TypeIDs 61–64** (weight: 0.35): 4 T0836 findings present.
- **Summary distinguishes timed variant** (weight: 0.15): All 8 summaries contain timed mnemonic.
- **CASDU/first_ioa evidence** (weight: 0.15): Evidence vectors include CASDU for both findings per TypeID.

## Edge Conditions

- TypeIDs 61–64 are valid standard IEC 60870-5-104 TypeIDs; no T0814 from BC-2.19.022.
- CP56Time2a adds 7 bytes per information object vs untimed variants — does not affect finding count.
- TypeID=64 (C_BO_TA_1 bitstring) must co-emit T0836 just like its untimed twin TypeID=51.

## Category: real-world-corpus

This is a synthetic security-probe scenario using crafted pcap data. Real-world ICS network
captures with time-tagged set-point commands are covered in HS-136 (known-good and
known-problematic IEC-104 corpus).

| Field | Description |
|-------|-------------|
| corpus_source | Crafted synthetic pcap (see Fixture Creation Obligation); real-world coverage in HS-136 |
| corpus_size | 4 I-frames, 1 TCP flow |
| known_edge_cases | CP56Time2a timestamp bytes appended; variable information-object sizes per TypeID |
| false_positive_threshold | 0 findings for TypeID=60 or TypeID=65 (neighbors must stay silent per BC-2.19.022 v1.1) |
| false_negative_threshold | 8 findings required (4 T1692.001 + 4 T0836, one pair per TypeID 61–64) |

## Fixture Creation Obligation

The evaluator must produce `iec104_timed_setpoint.pcap`: a single TCP flow on port 2404
delivering four back-to-back IEC-104 I-frames (TypeIDs 61, 62, 63, 64), one ASDU per frame,
VSQ=0x01, COT=6 activation (0x06 0x00), CASDU=1 (0x01 0x00).

**LEN arithmetic** (LEN = 4 control-field octets + ASDU length):
- Frame A/B: ASDU = TypeID(1)+VSQ(1)+COT(2)+CASDU(2)+IOA(3)+value(2)+QOS(1)+CP56Time2a(7) = 19 → LEN=4+19=23=**0x17**
- Frame C: ASDU = TypeID(1)+VSQ(1)+COT(2)+CASDU(2)+IOA(3)+R32(4)+QOS(1)+CP56Time2a(7) = 21 → LEN=4+21=25=**0x19**
- Frame D: ASDU = TypeID(1)+VSQ(1)+COT(2)+CASDU(2)+IOA(3)+BSI(4)+CP56Time2a(7) = 20 → LEN=4+20=24=**0x18** (no QOS for C_BO_TA_1)

**Canonical byte layout per frame:**

Frame A — TypeID=61 (C_SE_TA_1, normalized value), Send-sequence=0:
```
APCI: 68 17 00 00 00 00
ASDU: 3D 01 06 00 01 00 64 00 00 00 40 00 00 00 00 00 00 00 00
```
(TypeID=0x3D, IOA=0x000064=100, NVA=0x4000 LE, QOS=0x00, 7×0x00)

Frame B — TypeID=62 (C_SE_TB_1, scaled value), Send-sequence=1:
```
APCI: 68 17 02 00 00 00
ASDU: 3E 01 06 00 01 00 65 00 00 64 00 00 00 00 00 00 00 00 00
```
(IOA=0x000065=101, SVA=0x0064=100 LE, QOS=0x00)

Frame C — TypeID=63 (C_SE_TC_1, short float), Send-sequence=2:
```
APCI: 68 19 04 00 00 00
ASDU: 3F 01 06 00 01 00 66 00 00 00 00 48 42 00 00 00 00 00 00 00 00
```
(IOA=0x000066=102, R32=0x42480000 LE = 50.0f, QOS=0x00)

Frame D — TypeID=64 (C_BO_TA_1, bitstring 32 bits), Send-sequence=3:
```
APCI: 68 18 06 00 00 00
ASDU: 40 01 06 00 01 00 67 00 00 EF BE AD DE 00 00 00 00 00 00 00
```
(IOA=0x000067=103, BSI=0xDEADBEEF LE; no QOS per IEC 60870-5-101 Table 8)

These four frames are TCP-encapsulated in a single flow. Back-to-back delivery means exactly
8 findings are expected (4 × T1692.001 + 4 × T0836, one pair per TypeID 61–64, Case E).

## Failure Guidance

"HOLDOUT FAIL: HS-134 (satisfaction: 0.XX) — IEC-104 timed set-point/bitstring evasion gap
NOT closed. T1692.001 and/or T0836 absent for TypeIDs 61–64 means match arm 61..=64 in
detect_iec104_threats is missing. These TypeIDs must produce T1692.001 + T0836 Possible,
same as untimed arm 48..=51. See BC-2.19.030 Postconditions 1–2 and IEC104-TIMED-CMD-GAP-001."
