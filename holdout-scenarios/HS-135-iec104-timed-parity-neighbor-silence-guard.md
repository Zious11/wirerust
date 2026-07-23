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
id: "HS-135"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "feature-iec104"
behavioral_contracts:
  - BC-2.19.029
  - BC-2.19.030
  - BC-2.19.022
  - BC-2.19.019
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
fixture_note: "Requires multiple crafted pcap fixtures: individual flows for each TypeID tested. See Fixture Creation Obligation below."
---

# Holdout Scenario: IEC-104 Timed/Untimed Parity and Neighbor Silence Regression Guard

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

> **REGRESSION GUARD:** This scenario verifies: (1) timed arms produce structurally identical
> findings to their untimed twins (parity), (2) neighboring TypeIDs 52–57 and 65–99 that are
> NOT in any detection arm remain silent after the new arms are added (no false-positive
> expansion), (3) cot_test=[TEST] suffix is correctly inherited by timed-command findings.

## Scenario

### Case A — Timed/Untimed Parity: TypeID=58 vs TypeID=45

Two separate pcap files are analyzed:
1. `iec104_typeids_45_58_parity.pcap` containing two TCP flows on port 2404:
   - Flow 1: one I-frame with TypeID=45 (C_SC_NA_1, untimed)
   - Flow 2: one I-frame with TypeID=58 (C_SC_TA_1, timed twin)

The evaluator verifies:
- Both flows produce exactly 1 finding with `mitre_techniques: ["T1692.001"]`.
- Both findings have `verdict=="possible"`, `confidence=="medium"`, `category=="impact"`.
- Both findings include `CASDU=<N>` in evidence.
- The TypeID=45 summary does NOT contain "time-tagged"; the TypeID=58 summary DOES contain
  "time-tagged" or "C_SC_TA". This is the parity check: same technique, different wording.
- Neither finding contains T0836 (switching commands, not parameter writes).

### Case B — Timed/Untimed Parity: TypeID=61 vs TypeID=48 (Two-Finding Set)

Two separate pcap flows:
- Flow 1: one I-frame with TypeID=48 (C_SE_NA_1, untimed set-point)
- Flow 2: one I-frame with TypeID=61 (C_SE_TA_1, timed twin)

Both flows must produce exactly 2 findings: T1692.001 + T0836 Possible.
The TypeID=61 summaries must contain "time-tagged" or "C_SE_TA".
The TypeID=48 summaries must NOT contain "time-tagged" (untimed).

### Case C — Neighbor Silence: TypeIDs 52, 53, 54, 55, 56, 57 Produce Zero Findings

A pcap with one I-frame per TypeID 52–57 (six frames total, all on port 2404).
The evaluator confirms: `findings` array is empty OR contains zero entries with
`mitre_techniques` containing T1692.001 or T0836. Only T0814 would be unexpected here
(52–57 are NOT in the reserved range 0 or 128–255 and thus fall to the `_` catch-all).

Expected: `[.findings] | length == 0` (no T1692.001, no T0836, no T0814 for TypeIDs 52–57).

### Case D — Neighbor Silence: TypeIDs 65, 66, 67, 99 Produce Zero Findings

A pcap with one I-frame per TypeID in {65, 66, 67, 99}.
Expected: `findings` array is empty — no T1692.001, no T0836, no T0814.

### Case E — cot_test [TEST] Suffix Inherited by Timed Arms

A pcap with one I-frame TypeID=58 and one I-frame TypeID=61, both with the COT test bit
set (cot_test=true, e.g., COT=0x0086 with bit 7 set).

Expected:
- TypeID=58 finding summary ends with " [TEST]"
- TypeID=61 T1692.001 finding summary ends with " [TEST]"
- TypeID=61 T0836 finding summary ends with " [TEST]"

### Case F — Untimed Arms Unaffected (Full Regression)

A pcap with TypeIDs 45, 46, 47, 48, 49, 50, 51 (all untimed arms). After adding the new
timed arms, these must still produce findings identical to pre-wave-85 behavior:
- TypeIDs 45–47: 1 × T1692.001 each (3 total)
- TypeIDs 48–51: 2 × (T1692.001 + T0836) each (8 total)
Total: 11 findings. No regression on the existing untimed detection.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.19.029 | Invariant 5 — parity with untimed arm 45–47 | Cases A, E |
| BC-2.19.029 | Invariant 6 — regression guard 52–57 and 65–99 silent | Cases C, D |
| BC-2.19.030 | Invariant 5 — parity with untimed arm 48–51 | Cases B, E |
| BC-2.19.030 | Invariant 6 — regression guard 52–57 and 65–99 silent | Cases C, D |
| BC-2.19.022 | Invariant 1 (v1.1) — silently-logged range is {52–57, 65–99} | Cases C, D |
| BC-2.19.019 | Postconditions 1–3 — untimed arms unchanged | Case F |
| BC-2.19.017 | Invariant 1 — cot_test [TEST] suffix propagated | Case E |

## Verification Approach

```bash
# Case A parity check
wirerust analyze typeids_45_58_parity.pcap --iec104 --json | \
  jq '[.findings[] | select(.mitre_techniques[] == "T1692.001")] | length'
# Expect: 2 (one per TypeID 45 and 58)

# Case C neighbor silence
wirerust analyze typeids_52_57.pcap --iec104 --json | jq '.findings | length'
# Expect: 0

# Case D neighbor silence
wirerust analyze typeids_65_66_67_99.pcap --iec104 --json | jq '.findings | length'
# Expect: 0

# Case E [TEST] suffix
wirerust analyze typeids_58_61_cot_test.pcap --iec104 --json | \
  jq '.findings[] | select(.summary | endswith(" [TEST]")) | .summary' | wc -l
# Expect: 3 (1 for TypeID=58, 2 for TypeID=61)

# Case F untimed regression
wirerust analyze typeids_45_51.pcap --iec104 --json | jq '.findings | length'
# Expect: 11
```

## Evaluation Rubric

- **Parity (same technique/verdict/confidence for timed and untimed)** (weight: 0.30):
  Cases A, B pass with matching field values. Failure = structural divergence from untimed arm.
- **Neighbor silence (TypeIDs 52–57, 65–99 produce zero findings)** (weight: 0.35):
  Cases C, D: `findings | length == 0` for both sets of neighbors.
- **[TEST] suffix inheritance** (weight: 0.15): Case E: all 3 findings have [TEST] suffix.
- **Untimed arm regression** (weight: 0.20): Case F: exactly 11 findings, matching pre-wave-85 counts.

## Edge Conditions

- TypeIDs 52–57 are RESERVED per IEC 60870-5-101 — they fall to the `_` catch-all (BC-2.19.022 v1.1).
- TypeID 65 is the first unhandled TypeID above the new timed set-point arm (61–64).
- The `_` catch-all comment in detect_iec104_threats was updated in v1.1 to name these ranges.
- Case F regression is critical: the new arms must slot AHEAD of the `_` catch-all without
  disrupting the existing arms (45..=47, 48..=51) which precede them in the match.

## Category: real-world-corpus

This is a synthetic edge-case-combinations scenario using crafted pcap data to verify parity
and non-expansion invariants. Real-world corpus validation for IEC-104 timed command detection
is covered in HS-136 (known-good and known-problematic ICS traffic corpus).

| Field | Description |
|-------|-------------|
| corpus_source | Crafted synthetic pcap fixtures (per-TypeID isolation); real-world in HS-136 |
| corpus_size | 1 I-frame per TypeID tested; multiple small pcaps |
| known_edge_cases | RESERVED TypeIDs 52–57 (defined by spec but have no ASDU meaning); CP56Time2a variant parsing |
| false_positive_threshold | 0 findings for TypeIDs 52–57 and 65–99 (strict — any finding is a bug) |
| false_negative_threshold | Cases A+B: 1 T1692.001 per timed arm; Case F: exactly 11 findings for untimed arms |

## Fixture Creation Obligation

This scenario requires multiple small crafted pcap fixtures. All fixtures are TCP flows on
port 2404, VSQ=0x01, COT=0x06 0x00 (activation), CASDU=0x01 0x00 (CASDU=1). LEN for untimed
commands: 4+(6+3+1)=14=0x0E (1-byte info element, no CP56Time2a). LEN for timed 1-byte
element (TypeIDs 58–60): 4+(6+3+1+7)=21=0x15. LEN for timed 2-byte+QOS element (TypeIDs
61–62): 4+(6+3+2+1+7)=23=0x17. LEN for timed 4-byte+QOS element (TypeID 63): 4+(6+3+4+1+7)=25=0x19.

**iec104_typeids_45_58_parity.pcap** (Case A — two separate TCP flows):
- Flow 1, TypeID=45 (C_SC_NA_1, untimed): `APCI: 68 0E 00 00 00 00; ASDU: 2D 01 06 00 01 00 64 00 00 01`
- Flow 2, TypeID=58 (C_SC_TA_1, timed): `APCI: 68 15 00 00 00 00; ASDU: 3A 01 06 00 01 00 64 00 00 01 00 00 00 00 00 00 00`

**typeids_52_57.pcap** (Case C — neighbor silence): Six I-frames, TypeIDs 52–57 (RESERVED).
These fall to the `_` catch-all; any plausible minimal frame structure suffices since no finding
is emitted. Example using a 1-byte placeholder info element per TypeID:
TypeID bytes: 0x34(52), 0x35(53), 0x36(54), 0x37(55), 0x38(56), 0x39(57).
Frame template: `APCI: 68 0B 00 00 00 00; ASDU: <TypeID> 01 06 00 01 00 64 00 00 00`

**typeids_65_66_67_99.pcap** (Case D — neighbor silence): Four I-frames, TypeIDs 65, 66, 67, 99
(unhandled above the new timed set-point arm). Same minimal frame template.
TypeID bytes: 0x41(65), 0x42(66), 0x43(67), 0x63(99).

**typeids_58_61_cot_test.pcap** (Case E — [TEST] suffix): Two I-frames with COT test bit set
(cot_test=true; COT low byte = 0x86 = 0x06 | 0x80):
- TypeID=58 (C_SC_TA_1): `APCI: 68 15 00 00 00 00; ASDU: 3A 01 86 00 01 00 64 00 00 01 00 00 00 00 00 00 00`
- TypeID=61 (C_SE_TA_1): `APCI: 68 17 02 00 00 00; ASDU: 3D 01 86 00 01 00 64 00 00 00 40 00 00 00 00 00 00 00 00`

**typeids_45_51.pcap** (Case F — untimed regression): Seven I-frames TypeIDs 45–51 per
BC-2.19.019 reference implementation. These match the pre-wave-85 baseline byte layouts
(no CP56Time2a).

## Failure Guidance

"HOLDOUT FAIL: HS-135 (satisfaction: 0.XX) — IEC-104 timed/untimed parity or neighbor-silence
regression. If TypeIDs 52–57 or 65–99 produce findings, detection overreach has occurred —
check that the match arms are strictly 58..=60 and 61..=64. If parity fails (mismatched verdict/
confidence), check arm construction mirrors BC-2.19.019 lines 752–758 and 784–799. If untimed
counts changed, a match arm ordering conflict exists. See BC-2.19.029 Invariants 5–6 and
BC-2.19.030 Invariants 5–6."
