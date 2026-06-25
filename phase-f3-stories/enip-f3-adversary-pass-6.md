---
document_type: adversarial-review
cycle: feature-enip-v0.11.0
phase: F3
pass: 6
verdict: FAIL
critical: 0
high: 1
medium: 1
low: 0
novelty: MODERATE
produced: 2026-06-24
producer: adversary
---

# Adversarial Story Review — feature-enip-v0.11.0 (SS-17 stories), Pass 6

## Verdict: FAIL — 0 CRITICAL, 1 HIGH, 1 MEDIUM

Novelty: MODERATE. All prior-pass fixes HELD (VP-032 Sub-A asserts + canonical
names, Sub-D ownership, carry-overflow T0814 ordering, 26-BC coverage, aggregates,
dep-graph canonical fields, etc.).

## Severity Trajectory

| Pass | C | H | M | L | Notes |
|------|---|---|---|---|-------|
| Pass 1 | 4 | 6 | 5 | 3 obs | Root cause: ACs written from assumption, not transcribed from BC postconditions |
| Pass 2 | 1 | 3 | 5 | 3 | New defects in STORY-130 + CIP seams |
| Pass 3 | 0 | 2 | 3 | 2 | Severity decay |
| Pass 4 | 2 | 2 | 2 | 0 | VP-032 Sub-D orphaned; pseudocode gaps |
| Pass 5 | 0 | 1 | 2 | 0 | Carry-overflow T0814 ordering; VP-032 Sub-A asserts |
| **Pass 6** | **0** | **1** | **1** | **0** | flows_analyzed dead counter; STORY-136 attribution |

## Confirmed HELD (Prior-Pass Fixes)

All prior-pass remediations remain intact:
- VP-032 Sub-A harness asserts is_none() for len<24 + field-offset equality per BC-2.17.002 (F5-002)
- VP-032 Sub-A/B/C canonical harness names match VP-032 spec (F5-003)
- VP-032 Sub-D ownership assigned to STORY-132 with AC-132-007 (F4-01)
- STORY-137 carry-overflow: check_t0814 called BEFORE is_non_enip latch (F5-001)
- 26-BC coverage across all 9 stories (BC-2.17.001..026)
- STORY-138 anchored to BC-2.17.025; out-of-scope ACs deleted (F2-01..F2-10)
- Aggregate fields canonical (write_window_start_ts, no phantom fields)
- Dependency-graph acyclic with canonical field references
- Wave/dependency ordering verified correct

## Findings

### F-P6-001 — HIGH — BC-level: `flows_analyzed` dead counter

**Severity:** HIGH
**Category:** BC-level postcondition violation
**Story:** STORY-138 (enip_summary emission) + BC-2.17.021 + BC-2.17.017
**Recurrence pattern:** Modbus total_flows_analyzed dead-counter (F-DELTA-002)

**Description:**
BC-2.17.021 requires `enip_summary` to include the field `flows_analyzed` with
the canonical vector value `flows_analyzed: 1` (one analyzed flow per summary
emission). However, no BC defines an increment site for this counter.
BC-2.17.017 specifies the fold/merge logic for `enip_summary` construction but
omits any increment of `flows_analyzed`. The result: `flows_analyzed` will always
emit `0` regardless of how many flows were analyzed — a postcondition violation
against BC-2.17.021.

**Root cause:** BC gap — no BC assigns ownership of the `flows_analyzed` increment
site. The field appears in the output schema (BC-2.17.021) but is never populated
(BC-2.17.017 fold omits it).

**Impact:** Any implementation following the stories as written will emit
`flows_analyzed: 0` in every `enip_summary` JSON output, violating BC-2.17.021
postconditions. Holdout scenario HS-122 (enip-real-world-corpus) checks
`enip_summary` fields and will fail on this defect.

**Required remediation (BC-level — PO action):**
1. Product owner adds an increment site to BC-2.17.017 (fold/merge step):
   `flows_analyzed += 1` per flow processed.
2. STORY-138 acceptance criteria updated to assert `flows_analyzed >= 1` in
   emitted `enip_summary`.
3. Perform dead-counter sweep of ALL `enip_summary` fields (BC-2.17.021 schema)
   to confirm no other fields share this omission pattern.

**Status:** REMEDIATION IN PROGRESS

---

### F-P6-002 — MEDIUM — STORY-136 CipServiceClass attribution mis-cite

**Severity:** MEDIUM
**Category:** Traceability / attribution error
**Story:** STORY-136
**BC:** BC-2.17.007

**Description:**
STORY-136 attributes its `CipServiceClass` enum definition as "From STORY-130".
The correct attribution is STORY-132, which owns the `CipServiceClass` type per
BC-2.17.007. STORY-130 owns the frame-level parsing (EnipHeader/EnipCommandClass)
not the CIP service classification enum.

**Impact:** Traceability defect — implementer reading STORY-136 will look in
STORY-130 for `CipServiceClass` and not find it. This creates confusion at F4
implementation and risks the implementer re-defining the type, producing a
duplicate type compilation error.

**Required remediation:**
Update STORY-136 to read "From STORY-132" (not "From STORY-130") for the
`CipServiceClass` enum reference.

**Status:** REMEDIATION IN PROGRESS

---

## Convergence Assessment

**Counter:** 0/3 (reset — FAIL in Pass 6)

Prior trajectory: 4C/6H → 1C/3H → 0C/2H → 2C/2H → 0C/1H → 0C/1H (P6).

Severity is not decaying (P5 and P6 both 0C/1H). F-P6-001 is a BC-level gap
(not a story transcription error) requiring PO action before Pass 7. Once
F-P6-001 is fixed at the BC level and propagated to STORY-138, and F-P6-002
attribution is corrected in STORY-136, Pass 7 may achieve PASS.

**Pass 7:** Pending F-P6-001 BC-level fix (PO) + STORY-138 AC update + STORY-136
attribution correction.
