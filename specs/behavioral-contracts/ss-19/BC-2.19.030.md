---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-07-23T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-19
capability: CAP-19
lifecycle_status: active
introduced: wave-85-spec-evolution
modified:
  - "v1.1: CV-004 — Story Anchor filled: STORY-180 (draft, wave 85). 2026-07-23"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: []
input-hash: "d41d8cd"
---

# BC-2.19.030: Time-Tagged Set-Point + Bitstring Commands (TypeIDs 61–64, C_SE_TA_1/C_SE_TB_1/C_SE_TC_1/C_BO_TA_1) Emit T1692.001 + T0836

## Description

When a parsed ASDU has a TypeID in {61, 62, 63, 64} — the CP56Time2a time-tagged variants of
the plain set-point and bitstring write commands C_SE_NA_1 (48), C_SE_NB_1 (49), C_SE_NC_1 (50),
C_BO_NA_1 (51) — the analyzer emits two findings: T1692.001 "Unauthorized Message: Command
Message" (ICS) and T0836 "Modify Parameter" (ICS), both with Verdict Possible, Confidence Medium,
and ThreatCategory Impact. These four TypeIDs are the exact time-tagged twins of the untimed arm
`48..=51` already detected by BC-2.19.019; the only wire-format difference is a 7-byte CP56Time2a
timestamp appended to each information object. The set-point and bitstring write semantics —
writing a normalized value, a scaled value, a floating-point value, or a 32-bit bitstring to a
remote IED output — are identical to the untimed form.

T0836 is co-emitted because these commands modify ICS control parameters at the target RTU/IED
(set-point = parameter write; bitstring = output register write). This is the same rationale
applied to the untimed arm 48..=51 in BC-2.19.019 Postcondition 2.

This BC closes the second half of the detection gap documented in
`.factory/planning/iec104-timed-cmd-gap-validation.md` (IEC104-TIMED-CMD-GAP-001, CONFIRMED /
HIGH confidence, 2026-07-23). TypeIDs 58–64 fell silently through the `_` catch-all arm of
`detect_iec104_threats`. BC-2.19.029 covers 58–60; this BC covers 61–64.

## Preconditions

1. A valid I-format ASDU has been parsed.
2. `asdu.type_id` is in {61, 62, 63, 64}.
3. ASDU parse returned `Some(Asdu)`.

## Postconditions

1. Exactly one T1692.001 "Unauthorized Message: Command Message" finding is emitted with
   `Verdict::Possible`, `Confidence::Medium`, `ThreatCategory::Impact`. The evidence vector
   includes the command-message description, CASDU, and (when present) first_ioa.
2. Exactly one T0836 "Modify Parameter" finding is emitted with `Verdict::Possible`,
   `Confidence::Medium`, `ThreatCategory::Impact`. The evidence vector includes the
   parameter-modification description, CASDU, and (when present) first_ioa. T0836 is
   co-emitted because set-point and bitstring TypeIDs represent ICS parameter writes.
3. Both findings include `CASDU=<value>` and, when `asdu.first_ioa` is `Some(ioa)`, also
   `first_ioa=<ioa>` in their evidence vectors — identical evidence parity to the untimed arm
   (BC-2.19.019 Postcondition 3, lines 784–799 of the untimed implementation).
4. The T1692.001 finding's `summary` field distinguishes the time-tagged variant in
   human-readable form, naming the timed mnemonics (C_SE_TA/C_SE_TB/C_SE_TC or C_BO_TA), for
   example: "IEC-104 time-tagged control command TypeID=<N> (C_SE_TA/C_SE_TB/C_SE_TC/C_BO_TA):
   time-tagged set-point or bitstring write command observed on passive monitor (T1692.001
   unauthorized command message; BC-2.19.030)".
5. The T0836 finding's `summary` field also distinguishes the time-tagged variant, for
   example: "IEC-104 time-tagged parameter modification TypeID=<N> (C_SE_TA/C_SE_TB/C_SE_TC/
   C_BO_TA): time-tagged set-point or bitstring write modifies ICS control parameters (T0836
   modify parameter; BC-2.19.030 postcondition 2)".
6. The two findings are emitted once per ASDU frame regardless of the ASDU object count.
7. When `asdu.cot_test == true`, the ` [TEST]` suffix is appended to BOTH findings' `summary`
   fields by the post-emission loop at `detect_iec104_threats` lines 924–928 (BC-2.19.017
   Invariant 1 / AC-170-007). No extra wiring is needed.

## Invariants

1. **Two findings always emitted for 61–64**: T1692.001 AND T0836 are always co-emitted for
   TypeIDs in {61, 62, 63, 64}. It is never correct to emit only one of the two.
2. **Set-point/bitstring distinction preserved**: TypeIDs 61–63 are set-point commands (normalized,
   scaled, float); TypeID 64 is a bitstring write. Both sub-types emit both findings. The grouping
   mirrors the untimed arm which treats C_SE and C_BO as a single set for T0836 attribution.
3. **Count-independent**: both findings are emitted exactly once per ASDU regardless of the VSQ
   object count. (Parity: BC-2.19.019 Invariant 3.)
4. **Passive-only**: the analyzer does not block or delay the write command; it only records
   findings.
5. **Parity with untimed arm 48–51**: Verdict, Confidence, ThreatCategory, evidence field shapes
   (CASDU, first_ioa), technique IDs, and emission count are identical to the BC-2.19.019 arm
   for untimed TypeIDs 48–51. The only intentional difference is summary wording which names
   the timed mnemonics.
6. **Regression guard — neighbors stay silent**: TypeIDs 52–57 (reserved/unhandled) and
   TypeIDs 65–99 (also unhandled) produce no finding; they fall through to the `_` catch-all
   per BC-2.19.022 v1.1 (narrowed invariant). This arm MUST NOT expand detection beyond
   {61, 62, 63, 64}.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TypeID=61 (C_SE_TA_1, timed set-point normalized) | T1692.001 Possible + T0836 Possible |
| EC-002 | TypeID=62 (C_SE_TB_1, timed set-point scaled) | T1692.001 Possible + T0836 Possible |
| EC-003 | TypeID=63 (C_SE_TC_1, timed set-point short float) | T1692.001 Possible + T0836 Possible |
| EC-004 | TypeID=64 (C_BO_TA_1, timed bitstring of 32 bits) | T1692.001 Possible + T0836 Possible |
| EC-005 | TypeID=60 (just below range — C_RC_TA_1 timed switching) | T1692.001 only (BC-2.19.029) |
| EC-006 | TypeID=65 (just above range — unhandled) | No finding (silently logged per BC-2.19.022 v1.1) |
| EC-007 | TypeID=61, asdu.count=0 | Both findings still emitted — count-independent (Invariant 3) |
| EC-008 | TypeID=64, cot_test=true | Both findings emitted; both summaries end with " [TEST]" |
| EC-009 | TypeID=48 (untimed twin C_SE_NA_1) | T1692.001 + T0836 Possible (BC-2.19.019 handles; no regression) |
| EC-010 | TypeID=52 (neighbor below; RESERVED) | No finding (silently logged per BC-2.19.022 v1.1 — regression guard) |
| EC-011 | TypeID=65 (neighbor above; unhandled) | No finding (silently logged per BC-2.19.022 v1.1 — regression guard) |

## Canonical Test Vectors

| TypeID | CASDU | first_ioa | cot_test | Expected findings |
|--------|-------|-----------|----------|-------------------|
| 61 | 1 | Some(200) | false | T1692.001 Possible + T0836 Possible; both evidence vectors include "CASDU=1", "first_ioa=200" |
| 62 | 100 | None | false | T1692.001 Possible + T0836 Possible; evidence has "CASDU=100"; no first_ioa |
| 64 | 1 | Some(1) | true | T1692.001 Possible + T0836 Possible; both summaries end with " [TEST]" |
| 60 | 1 | None | false | T1692.001 only (BC-2.19.029, not this BC) |
| 65 | 1 | None | false | (no finding — silently logged) |
| 48 | 1 | None | false | T1692.001 + T0836 Possible (BC-2.19.019, not this BC — regression check) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic on TypeIDs 61–64 ASDU | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — time-tagged set-point and bitstring write command detection (TypeIDs 61–64) is a threat-detection function of the IEC-104 passive analyzer, closing the timed-variant evasion gap documented in IEC104-TIMED-CMD-GAP-001 |
| L2 Domain Invariants | INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs `detect_iec104_threats`); ADR-013 Decision 3 |
| Feature | wave-85-spec-evolution |
| MITRE Techniques | T1692.001 "Unauthorized Message: Command Message" (ICS) — Possible for TypeIDs 61–64; T0836 "Modify Parameter" (ICS) — Possible for TypeIDs 61–64 (parameter/value modification) |
| Validation Source | `.factory/planning/iec104-timed-cmd-gap-validation.md` (IEC104-TIMED-CMD-GAP-001, CONFIRMED / HIGH confidence) |

## Related BCs

- BC-2.19.019 — parity with (untimed set-point/bitstring arm 48–51; identical two-finding pattern, verdict, confidence, evidence shape; summary wording differs to name timed mnemonics)
- BC-2.19.022 — narrowed by (v1.1: silently-logged range 52–99 → 52–57 and 65–99 to exclude 58–64)
- BC-2.19.029 — sibling (time-tagged switching arm 58–60; emits T1692.001 only)
- BC-2.19.028 — depends on (MAX_IEC104_FINDINGS cap; both findings from this arm subject to cap at on_data extend step)

## Architecture Anchors

- `src/analyzer/iec104.rs` — new match arm in `detect_iec104_threats`, slotted ahead of `_` catch-all:
  `61..=64 => { /* C_SE_TA_1, C_SE_TB_1, C_SE_TC_1, C_BO_TA_1 — emit T1692.001 + T0836 Possible (BC-2.19.030) */ }`
- Evidence shape for T1692.001 finding mirrors lines 784–799 (untimed arm 48..=51): separate ev1 and ev2 vectors both with CASDU + conditional first_ioa.
- Summary wording mirrors lines 804–807 and 819–822 with "time-tagged" qualifier and timed mnemonics.
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3` (detection arm slot order)

## Story Anchor

STORY-180 (draft, wave 85)

## VP Anchors

- VP-047 — `fuzz_iec104_parser` (inherited; no-panic over all TypeIDs including 61–64)
