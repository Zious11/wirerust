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
id: "HS-133"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "feature-iec104"
behavioral_contracts:
  - BC-2.19.029
  - BC-2.19.017
  - BC-2.19.028
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
fixture_note: "Requires crafted pcap fixture: TCP flow on port 2404 delivering three IEC-104 I-frames with TypeIDs 58, 59, 60. See Fixture Creation Obligation below."
---

# Holdout Scenario: IEC-104 Time-Tagged Switching Commands (TypeIDs 58–60) Emit T1692.001

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

> **IEC104-TIMED-CMD-GAP-001 CLOSURE:** This scenario verifies that the evasion gap where
> TypeIDs 58–64 fell silently through `detect_iec104_threats` is closed. An attacker using
> time-tagged TypeIDs 58–60 (C_SC_TA_1/C_DC_TA_1/C_RC_TA_1) to issue unauthorized switching
> commands MUST now produce T1692.001 findings. Prior to BC-2.19.029, zero findings were emitted.

## Scenario

A crafted PCAP file is presented containing a single TCP flow on port 2404. The TCP payload
delivers three IEC-104 I-frames, each carrying an ASDU with a distinct TypeID.

**Frame A** — TypeID=58 (C_SC_TA_1, time-tagged single command):
APCI: `68 13 00 00 00 00`; ASDU: TypeID=0x3A, VSQ=0x01, COT=0x0006, CASDU=0x0001,
IOA=0x000064 (100), SCS=0x01, CP56Time2a=7 zero bytes.

**Frame B** — TypeID=59 (C_DC_TA_1, time-tagged double command):
APCI: `68 13 02 00 00 00`; ASDU: TypeID=0x3B, VSQ=0x01, COT=0x0006, CASDU=0x0001,
IOA=0x000065 (101), DCS=0x02, CP56Time2a=7 zero bytes.

**Frame C** — TypeID=60 (C_RC_TA_1, time-tagged regulating step command):
APCI: `68 13 04 00 00 00`; ASDU: TypeID=0x3C, VSQ=0x01, COT=0x0006, CASDU=0x0001,
IOA=0x000066 (102), RCS=0x02, CP56Time2a=7 zero bytes.

### Case A — TypeID=58 Emits T1692.001 (Primary Evasion-Closure Guard)

1. Run: `wirerust analyze iec104_timed_switching.pcap --iec104 --json`
2. Exit code: 0.
3. The JSON `findings` array contains AT LEAST one finding where:
   - `mitre_techniques` includes `"T1692.001"`
   - `category` == `"impact"`
   - `verdict` == `"possible"`
   - `confidence` == `"medium"`
   - `summary` contains "time-tagged" OR "C_SC_TA" (distinguishing from untimed)
   - `evidence` contains an entry matching `CASDU=1`
4. A finding IS emitted. Zero findings = gap not closed.

### Case B — TypeID=59 Emits T1692.001

At least one finding with `mitre_techniques: ["T1692.001"]` for TypeID=59, with summary
containing "C_DC_TA" or "time-tagged".

### Case C — TypeID=60 Emits T1692.001

At least one finding for TypeID=60.

### Case D — T0836 NOT Emitted for TypeIDs 58–60

The `findings` array MUST NOT contain any finding with `mitre_techniques` containing
`"T0836"` for these frames. Switching commands are binary control, not parameter writes.

### Case E — Exactly 3 Findings for 3 Back-to-Back I-Frames

When three back-to-back I-frames are delivered (TypeIDs 58, 59, 60), the findings array
contains exactly 3 T1692.001 findings attributable to this flow (once per ASDU).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.19.029 | Postcondition 1 — T1692.001 Possible emitted for TypeIDs 58–60 | Cases A, B, C |
| BC-2.19.029 | Postcondition 2 — T0836 NOT emitted for TypeIDs 58–60 | Case D |
| BC-2.19.029 | Postcondition 3 — CASDU/first_ioa in evidence | Case A evidence check |
| BC-2.19.029 | Postcondition 4 — summary distinguishes "time-tagged" variant | Case A summary check |
| BC-2.19.029 | Invariant 3 — count-independent (once per ASDU) | Case E |
| BC-2.19.017 | Invariant 1 — cot_test [TEST] suffix (COT=6, no [TEST] expected here) | Case A (no [TEST] in summary) |

## Verification Approach

```bash
wirerust analyze iec104_timed_switching.pcap --iec104 --json | \
  jq '[.findings[] | select(.mitre_techniques[] == "T1692.001")] | length'
# Expect: 3 (one per TypeID 58/59/60)

wirerust analyze iec104_timed_switching.pcap --iec104 --json | \
  jq '[.findings[] | select(.mitre_techniques[] == "T0836")] | length'
# Expect: 0 (T0836 must NOT appear for switching commands)

wirerust analyze iec104_timed_switching.pcap --iec104 --json | \
  jq '.findings[] | select(.mitre_techniques[] == "T1692.001") | .summary' | \
  grep -c "time-tagged\|C_SC_TA\|C_DC_TA\|C_RC_TA"
# Expect: 3 (all three summaries distinguish the timed variant)
```

## Evaluation Rubric

- **T1692.001 emitted for TypeIDs 58–60** (weight: 0.50): All three TypeIDs produce T1692.001.
  This is the primary evasion-closure guard. Failure = gap not closed.
- **T0836 absent for TypeIDs 58–60** (weight: 0.20): No T0836 finding for switching commands.
- **Summary distinguishes timed variant** (weight: 0.15): Contains "time-tagged" or timed mnemonic.
- **CASDU/first_ioa evidence present** (weight: 0.15): Evidence vector includes CASDU entry.

## Edge Conditions

- TypeIDs 58–60 are valid standard IEC 60870-5-104 TypeIDs; no T0814 from BC-2.19.022.
- COT=6 (activation): cot_test=false; no [TEST] suffix expected.
- If pcap splits APCI across TCP segments, carry-buffer (BC-2.19.025/026) handles reassembly.

## Category: real-world-corpus

This is a synthetic security-probe scenario using crafted pcap data. Real-world ICS network
captures with time-tagged control commands are covered in HS-136 (known-good and
known-problematic IEC-104 corpus). See HS-136 for false-positive rate and real-world
evasion detection validation.

| Field | Description |
|-------|-------------|
| corpus_source | Crafted synthetic pcap (see Fixture Creation Obligation); real-world coverage in HS-136 |
| corpus_size | 3 I-frames, 1 TCP flow |
| known_edge_cases | CP56Time2a timestamp bytes (7 bytes) appended to each information object |
| false_positive_threshold | 0 T0836 findings for TypeIDs 58–60 (binary switching, not parameter writes) |
| false_negative_threshold | 3 T1692.001 findings required (one per TypeID 58, 59, 60) |

## Failure Guidance

"HOLDOUT FAIL: HS-133 (satisfaction: 0.XX) — IEC-104 timed switching evasion gap NOT closed.
T1692.001 absent for TypeIDs 58/59/60 means the match arm 58..=60 in detect_iec104_threats
is missing. These TypeIDs must produce T1692.001 Possible, same as untimed arm 45..=47.
See BC-2.19.029 Postcondition 1 and IEC104-TIMED-CMD-GAP-001."
