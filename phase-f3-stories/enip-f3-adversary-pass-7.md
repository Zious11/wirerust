# Adversarial Story Review — feature-enip-v0.11.0 (SS-17 stories), Pass 7

## Verdict

**PASS** — 0 CRITICAL, 0 HIGH, 2 MEDIUM, 1 LOW. Novelty: LOW.

FIRST CLEAN story pass (zero HIGH/CRITICAL). All prior-pass fixes HELD. Dead-counter
sweep fully closed — all 7 enip_summary fields have verified increment sites.

## Finding Summary

| ID | Severity | Title | Disposition |
|----|----------|-------|-------------|
| F-P7-001 | MEDIUM | dependency-graph STORY-133→STORY-136 edge justification wrongly says STORY-136 emits IcsExecution tactic (BC-2.17.015 = vec![]) | REMEDIATED — rewrote justification to vec![]/catalog-prereq |
| F-P7-002 | MEDIUM | STORY-134 command_counts double-increment ambiguity vs STORY-138's single generic site | REMEDIATED — disambiguation note added |
| F-P7-003 | LOW | FALSE ALARM — adversary glob missed slugged filenames; all 13 holdouts HS-110..122 exist + git-tracked (verified) | NO ACTION |

## Finding Detail

### F-P7-001 (MEDIUM) — Dependency-graph STORY-133→STORY-136 justification tactic error

**Location:** `.factory/specs/architecture/dependency-graph-extended.md`, STORY-133→STORY-136 edge

**Finding:** The dependency edge justification claimed STORY-136 emits the IcsExecution MITRE
tactic. BC-2.17.015 specifies `emitted_tactic_ids: vec![]` — STORY-136 emits no findings of
its own; it provides the CipServiceClass enum catalog that STORY-130/132 reference. The
incorrect tactic claim made the edge justification factually wrong and could mislead the
implementer into adding spurious tactic emissions in STORY-136.

**Remediation:** Edge justification rewritten to accurately reflect: STORY-136 builds no
findings (vec![]), but STORY-133 depends on the CipServiceClass catalog for variant
classification. Justification now reads: "catalog-prereq — CipServiceClass enum consumed
by STORY-130/132 CIP request classification".

### F-P7-002 (MEDIUM) — STORY-134 command_counts double-increment ambiguity

**Location:** STORY-134 AC-134-009 pseudocode; STORY-138 aggregate pseudocode

**Finding:** STORY-134's per-flow pseudocode shows `command_counts[cmd] += 1` at the
dispatch site. STORY-138's aggregate AC also increments `command_counts` at the summary
rollup. Without an explicit disambiguation note, an implementer could read these as two
independent increment sites for the same logical counter, producing double-counting.
BC-2.17.021 Invariant 2 specifies one increment site per counter; dual-path
increment would violate the invariant.

**Remediation:** Disambiguation note added to STORY-134 AC-134-009 and STORY-138 aggregate
AC: STORY-134 owns the per-flow per-command increment; STORY-138 reads (not increments)
the per-flow value and sums it into the session aggregate for reporting. The single
increment site is in STORY-134; STORY-138 aggregates by reading only.

### F-P7-003 (LOW) — FALSE ALARM: holdout glob missed slugged filenames

**Location:** Adversary's file-existence sweep

**Finding:** Adversary glob pattern (`HS-1[0-9][0-9].md`) did not match the actual slugged
filenames (`HS-110-enip-canonical-frame-le-header-decode.md`, etc.). Initial report
claimed 0 of 13 holdouts found on disk.

**Disposition:** FALSE ALARM. Manual verification confirmed all 13 holdout scenario files
exist at `.factory/holdout-scenarios/HS-110*.md` through `HS-122*.md` and are
git-tracked on factory-artifacts. No action required.

## Severity Trajectory

| Pass | CRITICAL | HIGH | Result |
|------|----------|------|--------|
| P1 | 4 | 6 | FAIL |
| P2 | 1 | 3 | FAIL |
| P3 | 0 | 2 | FAIL |
| P4 | 2 | 2 | FAIL |
| P5 | 0 | 1 | FAIL |
| P6 | 0 | 1 | FAIL |
| **P7** | **0** | **0** | **PASS** |

## Convergence Counter

1/3 consecutive clean passes. Passes 8 and 9 pending.

## Coverage Checks (all PASS)

- 26-BC coverage: all BC-2.17.001..026 assigned across STORY-130..138 — VERIFIED
- VP-032 harness fidelity: Sub-A/B/C/D + vp032_cip_service_request_partition — VERIFIED
- Wave/dependency: dependency-graph E-20 chain acyclic post-remediation — VERIFIED
- Holdout coverage: 13 holdouts HS-110..122 (all must-pass) present and git-tracked — VERIFIED
- Dead-counter sweep: all 7 enip_summary fields have confirmed increment sites — VERIFIED
