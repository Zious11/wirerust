---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - "v1.1: F-W85S-P7-001 — PC5 field-path corrected: backticked vsq.count replaced with (the VSQ object count / asdu.count); Asdu struct has a flat count: u8 field with no vsq subfield. 2026-07-23"
  - "v1.2: CV-004 — Story Anchor filled: STORY-180 (draft, wave 85). 2026-07-23"
  - "v1.3: F-180-P4-001 — Story Anchor status label refreshed draft→ready (D-505 gate). 2026-07-24"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: []
input-hash: "d41d8cd"
---

# BC-2.19.029: Time-Tagged Switching Control Commands (TypeIDs 58–60, C_SC_TA_1/C_DC_TA_1/C_RC_TA_1) Emit T1692.001

## Description

When a parsed ASDU has a TypeID in {58, 59, 60} — the CP56Time2a time-tagged variants of
the plain switching control commands C_SC_NA_1 (45), C_DC_NA_1 (46), C_RC_NA_1 (47) — the
analyzer emits T1692.001 "Unauthorized Message: Command Message" (ICS) with Verdict Possible,
Confidence Medium, and ThreatCategory Impact. These three TypeIDs are the exact time-tagged
twins of the untimed arm `45..=47` already detected by BC-2.19.019; the only wire-format
difference is a 7-byte CP56Time2a timestamp appended to each information object. The control
semantics — actuating a switch, driving a relay, issuing a regulating step — are identical to
the untimed form.

No T0836 "Modify Parameter" is emitted for TypeIDs 58–60: these are binary switching commands
(on/off, up/down), not parameter or set-point writes. This parity rule mirrors BC-2.19.019
Invariant 2, which withholds T0836 from untimed TypeIDs 45–47.

This BC closes the detection gap documented in
`.factory/planning/iec104-timed-cmd-gap-validation.md` (IEC104-TIMED-CMD-GAP-001, CONFIRMED /
HIGH confidence, 2026-07-23). Prior to this BC, TypeIDs 58–64 fell silently through the `_`
catch-all arm of `detect_iec104_threats`, providing an evasion channel for attackers who
selected the time-tagged TypeID instead of its untimed twin.

## Preconditions

1. A valid I-format ASDU has been parsed.
2. `asdu.type_id` is in {58, 59, 60}.
3. ASDU parse returned `Some(Asdu)`.

## Postconditions

1. Exactly one T1692.001 "Unauthorized Message: Command Message" finding is emitted with
   `Verdict::Possible`, `Confidence::Medium`, `ThreatCategory::Impact`.
2. T0836 "Modify Parameter" is NOT emitted — TypeIDs 58–60 are binary switching commands, not
   parameter writes. (This mirrors BC-2.19.019 Invariant 2 for the untimed arm 45–47.)
3. The finding's evidence vector includes `CASDU=<value>` and, when `asdu.first_ioa` is
   `Some(ioa)`, also includes `first_ioa=<ioa>` — identical evidence parity to the untimed arm
   (BC-2.19.019 Postcondition 3).
4. The finding's `summary` field distinguishes the time-tagged variant in human-readable form,
   for example: "IEC-104 time-tagged control command TypeID=<N> (C_SC_TA/C_DC_TA/C_RC_TA):
   time-tagged switching control command observed on passive monitor (T1692.001 unauthorized
   command message; BC-2.19.029)". The summary MUST NOT be identical to the BC-2.19.019 summary
   for the untimed arm; analysts must be able to distinguish timed from untimed in output.
5. The finding is emitted once per ASDU frame regardless of the ASDU object count (the VSQ object count / `asdu.count`).
6. When `asdu.cot_test == true`, the ` [TEST]` suffix is appended to the finding's `summary`
   field by the post-emission loop at `detect_iec104_threats` lines 924–928 (BC-2.19.017
   Invariant 1 / AC-170-007). No extra wiring is needed in this arm — the loop runs over all
   findings added during the call, including those from this arm.

## Invariants

1. **T1692.001 for all timed switching TypeIDs**: T1692.001 is emitted for every TypeID in
   {58, 59, 60} without exception; it is the command-message indicator for all IEC-104 control
   direction messages.
2. **T0836 never emitted for 58–60**: Switching commands (timed or untimed) are binary control
   signals, not ICS parameter modifications. T0836 is reserved for set-point and bitstring
   writes. (Parity: BC-2.19.019 Invariant 2.)
3. **Count-independent**: the finding is emitted exactly once per ASDU regardless of the VSQ
   object count. (Parity: BC-2.19.019 Invariant 3.)
4. **Passive-only**: the analyzer does not block or delay the control command; it only records
   the finding.
5. **Parity with untimed arm 45–47**: Verdict, Confidence, ThreatCategory, evidence fields
   (CASDU, first_ioa), technique IDs, and emission count are identical to the BC-2.19.019
   arm for untimed TypeIDs 45–47. The only intentional difference is the summary wording,
   which names the timed mnemonics (C_SC_TA / C_DC_TA / C_RC_TA).
6. **Regression guard — neighbors stay silent**: TypeIDs 52–57 (reserved/unhandled) and
   TypeIDs 65–99 (also unhandled) produce no finding; they fall through to the `_` catch-all
   per BC-2.19.022 v1.1 (narrowed invariant). This arm MUST NOT expand detection beyond
   {58, 59, 60}.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TypeID=58 (C_SC_TA_1, timed single command) | T1692.001 Possible only (no T0836) |
| EC-002 | TypeID=59 (C_DC_TA_1, timed double command) | T1692.001 Possible only |
| EC-003 | TypeID=60 (C_RC_TA_1, timed regulating step command) | T1692.001 Possible only |
| EC-004 | TypeID=57 (just below range — RESERVED in IEC 60870-5-101) | No finding; silently logged per BC-2.19.022 v1.1 |
| EC-005 | TypeID=61 (just above range — C_SE_TA_1 timed set-point) | T1692.001 + T0836 Possible (BC-2.19.030) |
| EC-006 | TypeID=58, asdu.count=0 (ASDU with zero information objects) | T1692.001 still emitted — count-independent (Invariant 3) |
| EC-007 | TypeID=60, cot_test=true | T1692.001 emitted; summary has " [TEST]" suffix appended by post-emission loop |
| EC-008 | TypeID=45 (untimed twin C_SC_NA_1) | T1692.001 Possible (BC-2.19.019 handles; no regression) |
| EC-009 | TypeID=52 (neighbor below; RESERVED) | No finding (silently logged per BC-2.19.022 v1.1 — regression guard) |
| EC-010 | TypeID=65 (neighbor above; unhandled) | No finding (silently logged per BC-2.19.022 v1.1 — regression guard) |

## Canonical Test Vectors

| TypeID | CASDU | first_ioa | cot_test | Expected findings |
|--------|-------|-----------|----------|-------------------|
| 58 | 1 | Some(100) | false | T1692.001 Possible; evidence includes "TypeID=58 is a time-tagged switching control command (C_SC_TA/C_DC_TA/C_RC_TA)", "CASDU=1", "first_ioa=100" |
| 59 | 200 | None | false | T1692.001 Possible; evidence includes "CASDU=200"; no first_ioa entry |
| 60 | 1 | Some(1) | true | T1692.001 Possible; summary ends with " [TEST]" |
| 57 | 1 | None | false | (no finding — silently logged) |
| 61 | 1 | None | false | T1692.001 + T0836 (BC-2.19.030, not this BC) |
| 45 | 1 | None | false | T1692.001 Possible (BC-2.19.019, not this BC — regression check) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic on TypeIDs 58–60 ASDU | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — time-tagged switching control command detection (TypeIDs 58–60) is a threat-detection function of the IEC-104 passive analyzer, closing the timed-variant evasion gap documented in IEC104-TIMED-CMD-GAP-001 |
| L2 Domain Invariants | INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs `detect_iec104_threats`); ADR-013 Decision 3 |
| Feature | wave-85-spec-evolution |
| MITRE Techniques | T1692.001 "Unauthorized Message: Command Message" (ICS) — Possible for TypeIDs 58–60; T0836 NOT emitted (switching commands are binary control, not parameter writes) |
| Validation Source | `.factory/planning/iec104-timed-cmd-gap-validation.md` (IEC104-TIMED-CMD-GAP-001, CONFIRMED / HIGH confidence) |

## Related BCs

- BC-2.19.019 — parity with (untimed switching arm 45–47; identical technique, verdict, confidence, evidence shape; summary wording differs to name timed mnemonics)
- BC-2.19.022 — narrowed by (v1.1: silently-logged range 52–99 → 52–57 and 65–99 to exclude 58–64)
- BC-2.19.030 — sibling (time-tagged set-point/bitstring arm 61–64; emits T1692.001 + T0836)
- BC-2.19.028 — depends on (MAX_IEC104_FINDINGS cap; findings from this arm subject to cap at on_data extend step)

## Architecture Anchors

- `src/analyzer/iec104.rs` — new match arm in `detect_iec104_threats`, slotted ahead of `_` catch-all:
  `58..=60 => { /* C_SC_TA_1, C_DC_TA_1, C_RC_TA_1 — emit T1692.001 Possible; no T0836 (BC-2.19.029) */ }`
- Evidence shape mirrors lines 752–758 (untimed arm 45..=47): CASDU + conditional first_ioa.
- Summary wording mirrors lines 763–766 with "time-tagged" qualifier and "C_SC_TA/C_DC_TA/C_RC_TA" mnemonic.
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3` (detection arm slot order)

## Story Anchor

STORY-180 (ready, wave 85)

## VP Anchors

- VP-047 — `fuzz_iec104_parser` (inherited; no-panic over all TypeIDs including 58–60)
