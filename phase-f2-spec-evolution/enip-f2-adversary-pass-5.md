# Adversarial Spec Review — feature-enip-v0.11.0 (SS-17), Pass 5

**Cycle:** feature-enip-v0.11.0
**Phase:** F2 — Spec Evolution
**Pass:** 5
**Date:** 2026-06-24
**Verdict:** FAIL — 1 HIGH, 3 MEDIUM, 1 LOW (0 CRITICAL)
**Novelty:** MEDIUM — Core ENIP spec CONFIRMED CLEAN; all drift in ARCH-INDEX + PRD §6.5
(index/accounting lag of LOCKED decisions).

---

## Summary

Pass 5 found 5 findings (0C/1H/3M/1L), all in the index/accounting layer. The core
normative SS-17 spec (BCs, VP-032, ADR-010 decisions, SS-17 subsystem design) is
confirmed clean across all 5 passes. All findings are index/anchor-layer propagation
lag — a known pattern tracked as PROPAGATION-LAG-001. All 5 findings REMEDIATED.

**Severity trajectory:** Pass1 4C/7H → Pass2 4C/3H → Pass3 3C/4H → Pass4 0C/1H →
Pass5 0C/1H. Monotone decay confirmed; residue confined to index/anchor layer.

---

## Findings

### F-P5-001 (HIGH) — ARCH-INDEX ADR-010 row EMITTED count stale

**Location:** `ARCH-INDEX.md` line 213, ADR-010 row
**Description:** ADR-010 row showed `EMITTED 17→19` (19 was the Pass-2 intermediate
value). Pass-4 remediation added T0846 as a third new emission (T0858 + T0816 + T0846),
bringing the total to 20 emitted. The ARCH-INDEX row was not updated to reflect this.
**Expected:** `EMITTED 17→20`
**Actual:** `EMITTED 17→19`
**Status:** REMEDIATED — corrected to `EMITTED 17→20` in ARCH-INDEX v1.8.

---

### F-P5-002 (MEDIUM) — ARCH-INDEX O-04 debt row catalogue-only count stale

**Location:** `ARCH-INDEX.md` line 230, O-04 Architecture Debt row
**Description:** O-04 row stated `SEEDED 28 − EMITTED 19 = 9 catalogue-only`. With
EMITTED corrected to 20 (F-P5-001), the catalogue-only count is 28 − 20 = 8.
**Expected:** `SEEDED 28 − EMITTED 20 = 8 catalogue-only`
**Actual:** `SEEDED 28 − EMITTED 19 = 9 catalogue-only`
**Status:** REMEDIATED — corrected to `EMITTED 20 = 8 catalogue-only` in ARCH-INDEX v1.8.

---

### F-P5-003 (MEDIUM) — PRD §6.5 BC-2.10.005 annotation still described T0846 as seeded-not-emitted

**Location:** `prd.md` §6.5, line ~1555, BC-2.10.005 annotation
**Description:** PRD §6.5 RTM still contained a note describing T0846 as
`seeded-not-emitted` for BC-2.10.005. T0846 was moved to `now emitted` in BC-2.17.010
(Identity Object read detection) during F2 spec evolution. The RTM annotation was a
propagation lag from before BC-2.17.010 was authored.
**Expected:** PRD annotation updated to reflect T0846 emitted via BC-2.17.010
**Actual:** T0846 listed under seeded-not-emitted for BC-2.10.005
**Status:** REMEDIATED — PRD §6.5 updated; T0846 annotated as `now emitted (BC-2.17.010)`.

---

### F-P5-004 (LOW) — ARCH-INDEX MALFORMED_ANOMALY_THRESHOLD anchor cited wrong decision number

**Location:** `ARCH-INDEX.md` line 184 (MALFORMED_ANOMALY_THRESHOLD entry)
**Description:** ARCH-INDEX cited `Decision 3` as the anchor for
`MALFORMED_ANOMALY_THRESHOLD`. ADR-010 was restructured: Decision 3 became the
600-byte carry-buffer cap (MAX_ENIP_CARRY_BYTES). The malformed-anomaly threshold
rationale lives in architecture-delta §4.2 / ADR-010 Decision 4.
**Expected:** Anchor updated to `architecture-delta §4.2 / Decision 4`
**Actual:** `Decision 3`
**Status:** REMEDIATED — anchor re-pointed to `architecture-delta §4.2/Decision 4` in
ARCH-INDEX v1.8.

---

### F-P5-005 (MEDIUM) — ARCH-INDEX SS-17 BC count and ADR-010 range still showed 24

**Location:** `ARCH-INDEX.md` line 130 (SS-17 subsystem row); `ADR-010` line 686
**Description:** SS-17 subsystem row in ARCH-INDEX showed `BC count: 24` and BC range
`..024`. BC-2.17.025 (session-handshake circuit-break) was added during F2 Pass-1
remediation, bringing the true count to 25. The ARCH-INDEX row and the ADR-010 BC range
reference were not updated during Pass-1 remediation.
**Expected:** BC count 25, range `BC-2.17.001..025`
**Actual:** BC count 24, range `..024`
**Status:** REMEDIATED — ARCH-INDEX SS-17 row updated to 25 BCs + range `..025`; ADR-010
line 686 updated to `..025`. ARCH-INDEX version bumped v1.7→v1.8.

---

## Remediation Commits

All 5 findings remediated in a single fix burst on `factory-artifacts` branch.
Files changed:
- `.factory/specs/architecture/ARCH-INDEX.md` (F-P5-001, F-P5-002, F-P5-004, F-P5-005)
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md` (F-P5-005)
- `.factory/specs/prd.md` (F-P5-003)

Note: `docs/adr/0010-*.md` is on the `develop` working tree — ships with F4 code
delivery, not committed here.

---

## Convergence Status

**Pass 5:** FAIL (0C/1H/3M/1L) — all REMEDIATED.
**Convergence counter:** 0/3 (reset; Pass 6 required).
**Core spec:** CONFIRMED CLEAN (SS-17 BCs, VP-032, ADR-010 decisions, CAP-17 normative
content — no findings across any of passes 1-5).
**Residue pattern:** PROPAGATION-LAG-001 — index/accounting layer only.
**Next:** Dispatch Pass 6. If Pass 6 returns 0 findings or index-only LOW, increment
counter toward 3/3.
