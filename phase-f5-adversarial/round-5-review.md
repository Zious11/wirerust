---
document_type: f5-adversarial-round-review
producer: adversary-via-orchestrator
date: 2026-07-17
cycle: feature-iec104
round: 5
reviewed_sha: b36b884
base: fedcea4
---

# F5 Round 5 — feature-iec104

Attestation: develop @ b36b884 (FIX-F5-004 squash-merged, PR #415).

## Part A — Round 4 Finding Verification (F-B2)

**F-B2 [MEDIUM][doc-accuracy] — VERIFIED FIXED by FIX-F5-004 (PR #415 b36b884):**

Both corrections in FIX-F5-004 are accurate:

- Example-3 `mitre_techniques` corrected from `[]` to `["T0814"]` — matches actual
  serialized output from `iec104.rs:1214` (carry-packet detection finding; T0814 applies).
- FIX-F5-001 CHANGELOG intro sentence corrected from "8 function" to "10 total / 8 function"
  — accurately reflects the implementation: 8 named `emit_*` helper functions plus 2 inline
  `findings.push(Finding { ... })` call sites that were also enriched with `source_ip` and
  `timestamp` in FIX-F5-001.

## Part B — Comprehensive Cross-Check

**CHANGELOG accuracy sweep — ALL MATCH:**

All CHANGELOG entries for the feature-iec104 fix batch (FIX-F5-001, FIX-F5-002, FIX-F5-003,
FIX-F5-004) were verified against the actual source changes and serialized output. No
additional misstatements found.

**Demo-JSON cross-check — ALL MATCH:**

All four FIX-P4-001 demo-evidence artifacts now reference real enum variants and real
serialized JSON structure. Example JSON in FIX-F5-001/002/003/004 change descriptions also
checked; no fabricated values remain.

**BC-completeness sweep — 31/31 PASS:**

All 31 behavioral contracts in the feature-iec104 BC set (BC-2.19.001..031) have at least
one corresponding test assertion. No gaps introduced by any fix-PR. (Same result as R1 sweep
confirming no regression from the four fix batches.)

**Canonical-frame sweep — 19 byte-exact invariants UNDISTURBED:**

The 19 IEC 60870-5-104 canonical-frame invariants verified in R1 remain byte-exact. None
of the fix-PRs (FIX-F5-001..004) touched the frame-parsing or canonical-byte-position logic.
No regression.

**Kani non-vacuity — PASS:**

VP-044 (89 checks, 5 facets), VP-045/046 (non-vacuous proptests via interleaved generator),
VP-047 (fuzz 1.95M execs, 0 crashes). All non-vacuity requirements from D-462 remain
satisfied; fix-PRs made no changes to Kani harnesses or proptest strategies.

**Mutation score — 117/122 = 95.9% (unchanged):**

No fix-PR modified production logic that would affect mutant survival. Score carried forward
from F4 convergence (D-463).

## Findings

**Observation [LOW] — non-blocking, prose-only:**

TypeID 45 (C_SC_NA_1, Single Command / control command) is described as "monitoring
direction" in two documentation artifacts:

- `docs/demo-evidence/FIX-P4-001/evidence-report.md:46` — prose description of the TypeID
  45 detection finding calls it a "monitoring direction" command.
- `docs/demo-evidence/FIX-P4-001/AC-P4-001-test-results.txt:61` — same mislabeling in the
  test result narrative.

The production code is correct: `iec104.rs:744-748` classifies TypeIDs 45-47 as control
commands (T1692.001) with the correct direction annotation. This is a prose-only mislabel
in two demo-evidence files; it does not affect any behavior, test result, or BC.

Non-blocking. Recommended to fold into the next docs-currency sweep or cycle-close pass
rather than opening a new FIX-F5-005 PR for a single prose correction.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 5 |
| **New findings** | 0 |
| **Observations (LOW, non-blocking)** | 1 |
| **Novelty score** | 0.0 (0 CRITICAL/HIGH/MEDIUM) |
| **Median severity** | 0.0 |
| **Trajectory** | 4 (R1) → 2 (R2) → 1 HIGH (R3) → 1 MEDIUM (R4) → 0 (R5) |
| **Verdict** | CONVERGENCE_REACHED |

## Classification

NITPICK_ONLY → **F5 CONVERGED**.

0 CRITICAL / 0 HIGH / 0 MEDIUM findings. 1 LOW observation (TypeID 45 prose mislabel in
demo-evidence files; code correct). BC-completeness 31/31 PASS, canonical-frame 19
byte-exact PASS, Kani non-vacuity PASS, fuzz 1.95M execs 0 crashes, mutants 117/122=95.9%.

Feature-iec104 F5 scoped adversarial gate PASSED. F6 targeted hardening is next.
