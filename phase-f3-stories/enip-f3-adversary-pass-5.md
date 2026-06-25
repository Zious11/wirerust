# Adversarial Story Review — feature-enip-v0.11.0 (SS-17 stories), Pass 5

**Date:** 2026-06-24
**Cycle:** feature-enip-v0.11.0
**Pass:** 5
**Scope:** STORY-130..138 (9 stories, 26 BCs BC-2.17.001..026, VP-032)
**Verdict:** FAIL — 0 CRITICAL, 1 HIGH, 2 MEDIUM
**Novelty:** MODERATE
**Severity trajectory:** P1 4C/6H → P2 1C/3H → P3 0C/2H → P4 2C/2H → P5 0C/1H

## Prior-Pass Fix Verification

All prior-pass fixes HELD:

- VP-032 Sub-D ownership → STORY-132 (F4-01 CRITICAL): HELD
- STORY-137 validity-gate pseudocode (F4-02 CRITICAL): HELD
- STORY-138 `write_window_start_ts` rename (F4-03 HIGH): HELD
- Dep-graph E-20 canonical field names + STORY-132↔137 relationship (F4-04 HIGH): HELD
- STORY-134 AC ordering 006-before-005 (F4-05 MED): HELD
- STORY-138 phantom `cip_service_counts` field removed (F4-06 MED): HELD
- STORY-130 VP-032 Sub-A/B/C harness bodies (F2): HELD
- AC→BC fidelity across STORY-130/134/135/136/137/138 (F1 pass): HELD
- 26-BC coverage across all 9 stories: HELD
- Aggregates: HELD
- Dep-graph canonical fields: HELD

---

## Findings

### F5-001 (HIGH) — STORY-137 carry-overflow path sets `is_non_enip=true` BEFORE `check_t0814`

**Story:** STORY-137
**Affected BCs:** BC-2.17.018 EC-007/Precond 6; BC-2.17.018 story EC-004
**Status:** REMEDIATED

**Description:**
The carry-overflow handling path in STORY-137's pseudocode latches `is_non_enip = true`
BEFORE calling `check_t0814`. The guard for `check_t0814` is `!is_non_enip`. Therefore,
on a carry-overflow event, T0814 can NEVER fire — the latch poisons its own precondition.

This directly contradicts:
- BC-2.17.018 EC-007/Precond 6: the T0814 structural-anomaly check is required to evaluate
  the oversize frame before the non-ENIP classification is applied.
- STORY-137's own EC-004: oversize frame MAY still be checked for T0814 before carry is
  abandoned.

**Remediation:**
Reorder pseudocode: invoke `check_t0814` BEFORE latching `is_non_enip = true` on the
overflow path. Add test: overflow event where frame header is structurally anomalous
asserts BOTH T0814 fired AND `is_non_enip = true` on the returned state.

---

### F5-002 (MEDIUM) — STORY-130 VP-032 Sub-A harness asserts nothing

**Story:** STORY-130
**Affected BCs:** BC-2.17.002 (mapped to VP-032 Sub-A)
**Status:** REMEDIATED

**Description:**
The VP-032 Sub-A Kani harness body in STORY-130 (vp032_enip_header_parse_safety) performs
no-panic verification only (harness runs, no panic = pass). It must additionally assert:
- `is_none()` for all inputs with `len < 24` (the minimum valid ENIP header length per
  BC-2.17.002)
- Field-offset equality assertions for valid frames: command field at offset 0, length at
  offset 2, session handle at offset 4, etc. (per BC-2.17.002 postconditions)

Without these assertions the harness provides zero functional verification — a stub that
always panics would also pass the current harness.

**Remediation:**
Add `assert!(result.is_none())` branch for `len < 24` path. Add field-equality assertions
for the valid-frame branch. Confirm harness matches VP-032 Sub-A specification text.

---

### F5-003 (MEDIUM) — STORY-130 Sub-A/B/C harness names drift from VP-032 canonical

**Story:** STORY-130
**Affected BCs:** VP-032 Sub-A, Sub-B, Sub-C harness name canonicalization
**Status:** REMEDIATED

**Description:**
VP-032 Sub-D harness name `vp032_enip_session_ownership_biconditional` already matches
the canonical form (added in STORY-132, F4-01). However Sub-A/B/C harnesses in STORY-130
use non-canonical names:

| Sub-property | STORY-130 name (non-canonical) | VP-032 canonical name |
|---|---|---|
| Sub-A | `vp032_parse_safety` | `vp032_enip_header_parse_safety` |
| Sub-B | `vp032_cmd_class` | `vp032_enip_command_classification_biconditional` |
| Sub-C | `vp032_validity_gate` | `vp032_enip_validity_gate_biconditional` |

Sub-D already correct: `vp032_enip_session_ownership_biconditional`.

**Remediation:**
Rename all three harnesses in STORY-130 AC list and pseudocode to match VP-032 canonical
names exactly. Verify `vp032_cip_service_request_partition` (fifth harness) is also present
in the appropriate story AC and matches VP-032 text.

---

## Convergence Assessment

- Clean passes required: 3 consecutive
- Current clean count: 0/3 (reset — this pass is FAIL)
- All findings REMEDIATED — Pass 6 is running

**Severity trajectory:**
```
P1: 4C/6H  → P2: 1C/3H  → P3: 0C/2H  → P4: 2C/2H  → P5: 0C/1H
```

Trajectory is net-converging. P4 regression (0C/2H → 2C/2H) was a depth-audit artifact
(VP-032 Sub-D orphan + pseudocode completeness), not a fix regression. P5 finding F5-001
(T0814 guard ordering) is a pure-pseudocode logical defect — not a recurrence of any prior
finding class. Moderate novelty is consistent with approaching convergence.
