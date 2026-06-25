# Adversarial Spec Review — feature-enip-v0.11.0 (SS-17), Pass 7

**Date:** 2026-06-24
**Cycle:** feature-enip-v0.11.0
**Phase:** F2 — Spec Evolution
**Pass:** 7 of N (convergence target: 3 consecutive clean passes)
**Verdict:** FAIL — 1 HIGH, 0 CRITICAL (1 LOW). Novelty: LOW-MODERATE.

---

## Verdict Summary

8 of 9 axes CLEAN. Single HIGH is a partial-fix propagation gap: Pass 6
remediated BC-2.17.014 to strict `>` semantics (6th error fires T0888, 5th
does not), but ADR-010 prose and OA-005 were not updated to match. An
implementer reading ADR-010 Decision 4 doc-comment or OA-005 would be misled
to emit T0888 one error early (>= semantics). Constant value 5 is CORRECT —
prose-only fix required.

Pass-6 clean streak broken. Convergence counter RESET to 0/3.

---

## Findings

### F-P7-001 (HIGH) — ADR-010 / OA-005 strict-`>` propagation gap

**Status:** REMEDIATED (architect, 2026-06-24) — awaiting Pass 8 verification.

**Location:**
- `ADR-010-ethernet-ip-cip-stream-dispatch.md` — Decision 4 doc-comment, line
  303: "5 CIP error responses ... triggers T0888"
- `ADR-010-ethernet-ip-cip-stream-dispatch.md` — line 305: "LOCKED value: 5
  errors/10s"
- `OA-005` — line 674 (same implied >= semantics)

**Issue:** Pass 6 locked BC-2.17.014 to strict `>` semantics: 6 consecutive
errors within window trigger T0888; the 5th error does NOT. ADR-010 Decision 4
prose and OA-005 still describe the threshold in ambiguous or >= language
("triggers on 5 errors", "LOCKED value: 5 errors/10s"). An implementer reading
only ADR-010 would write the comparator as `>= 5` instead of `> 5`, emitting
T0888 one error too early. This contradicts the Pass-6-locked BC.

**Required fix:** Reword ADR-010 Decision 4 prose and any OA-005 references to
make the strict `>` relationship explicit: "the 6th consecutive error (count
exceeds threshold of 5) fires T0888; exactly 5 errors do NOT fire." The
constant VALUE 5 is correct and must not change (see F-P7-002).

**Cited BC:** BC-2.17.014 (strict `>`, locked Pass 6).

---

### F-P7-002 (LOW) — Constant value 5 is correct; prose-only fix

**Status:** CONFIRMED-CLEAN (no code change required).

**Note:** The threshold CONSTANT `ENIP_ERROR_BURST_THRESHOLD = 5` is correct.
The comparator in implementation will be `count > 5` (fires on 6th). The
integer literal 5 must NOT be changed to 6. This finding documents that
F-P7-001 is prose-only and guards against an over-eager "fix" that changes the
constant.

---

## Axis-by-Axis Results

| Axis | Result | Notes |
|------|--------|-------|
| Endianness (LE) | CLEAN | All multi-byte fields LE throughout ADR-010, BC files, and VP-032. No BE residues found. |
| CIP service table (13/15 services) | CLEAN | 0x0A MSP confirmed present. Table at 13 listed + 2 reserved slots = 15 total. |
| MITRE/EMITTED | CLEAN | 17→20→28 accounting correct. T0888/T0846 emitted; T1693.001 (not T0857 revoked) confirmed. T0858 present. 8 techniques in holdout-eval scope. |
| Frame-walk | CLEAN | Carry-buffer cap 600 bytes enforced. Frame-split logic consistent across BC-2.17.005/008. |
| Canonical-frame holdout | CLEAN | No new contradictions vs. HS-INDEX entries for SS-17. |
| Kani non-vacuity (5 harnesses) | CLEAN | VP-032 sub-A/B/C/D + vp032_cip_service_request_partition all have non-vacuous witness obligation. |
| Testability | CLEAN | BC-2.17.014 strict `>` is internally consistent; BC-2.17.025 pdu_count via process_pdu matches BC-2.17.024 session-handshake semantics. |
| Cross-doc consistency | FAIL (F-P7-001) | ADR-010 Decision 4 prose contradicts BC-2.17.014 strict `>`. OA-005 line 674 same. |
| Semantic anchoring | CLEAN | cli.rs = SS-12, enip.rs = SS-17 anchors intact. ARCH-INDEX v1.8 subsystem table correct. |

---

## New-Defect Sweep (Pass-6 Fixes Regression Check)

Pass 6 remediated:
- F6-01: BC-2.17.025 pdu_count anchor
- F6-02: BC-2.17.014 error-burst `>` off-by-one (the BC itself)
- F6-03: OA-001 label hygiene

**Sweep result:** Pass-6 fixes did NOT introduce regressions in BC-2.17.008 or
BC-2.17.025. Only the ADR back-propagation lag was identified (F-P7-001). The
BC-2.17.014 strict `>` semantics are correctly locked in the BC file itself;
the gap is documentation prose only.

---

## Severity Trajectory (Full)

| Pass | Critical | High | Medium | Low | Verdict |
|------|----------|------|--------|-----|---------|
| P1 | 4 | 7 | 3 | 3 | FAIL |
| P2 | 4 | 3 | 3 | 2 | FAIL |
| P3 | 3 | 4 | 4 | 0 | FAIL |
| P4 | 0 | 1 | 4 | 2 | FAIL |
| P5 | 0 | 1 | 3 | 1 | FAIL |
| P6 | 0 | 0 | 2 | 1 | PASS (first clean) |
| P7 | 0 | 1 | 0 | 1 | FAIL (ADR prose lag) |

Convergence counter before Pass 7: 1/3 (Pass 6 clean streak).
Convergence counter after Pass 7: RESET to 0/3.

---

## Next Step

Architect remediation of F-P7-001 in progress. Pass 8 to be dispatched after
confirmation that ADR-010 Decision 4 prose and OA-005 line 674 have been
updated to strict `>` / "6th error fires" language.
