---
document_type: process-gap-ledger
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-07-25T23:42:00Z
cycle: "wave-086"
inputs: []
input-hash: "[live-state]"
traces_to: STATE.md
---

# Process-Gap Ledger — wave-086

Process-gap candidates captured during wave-086 for cycle-close codification.
Each entry requires DF-VALIDATION-001 research-agent validation before filing
as a GitHub issue (product-local) or upstream drbothen/vsdd-factory issue.

---

## PG-W86-001 — story-writer lacks positive-coverage-assertion checklist for detector/gate stories

**Class:** Story-writer checklist gap / positive-coverage assertion discipline
**Caught by:** Wave-86 adversarial pass 1 (F-W86S-P1-007 HIGH) + pass 2 (F-W86S-P2-002 HIGH)
**Severity:** HIGH (pattern recurred identically in both STORY-182 and STORY-183 during the
  same wave; positive-coverage assertions were either absent or grounded against fabricated
  inputs rather than real stale sites)
**Occurrences:** 2 stories in wave-86 (same defect shape: pass-1 added positive-coverage ACs
  grounded against wrong/fabricated inputs; pass-2 found those ACs ineffective)
**Source finding:** F-W86S-P1-007 (HIGH, wave-86 pass 1) + F-W86S-P2-002 (HIGH, wave-86 pass 2)
**Vehicle:** Local carry-forward (DF-VALIDATION-001 required before filing upstream)

### Description

When story-writer drafts a "detector" or "gate" story (a story whose primary behavior is
detecting/rejecting a bad condition), it systematically omits or mislabels positive-coverage
acceptance criteria — ACs that assert the tool DETECTS a stale/bad artifact when one is
present (the "red path").

The pattern manifested twice in wave-86:

1. **STORY-183 (pass-1):** Pass-1 added AC-183-007/008 as positive-coverage ACs, but they
   were grounded against Patterns 30/31 (`currently falls through`, `is expected to`) which
   came from a lesson-summary that mislabeled the real phrase classes. Pass-2 found that
   these two patterns matched ZERO of 9 real stale sites from D-506 — the positive-coverage
   ACs were testing against fabricated inputs, not real ones (F-W86S-P2-001 CRIT,
   F-W86S-P2-002 HIGH).

2. **STORY-182 (pass-1 + pass-2):** Gate test (`fixture_manifest_all_present`) was specified
   without requiring it to actually fail on clean checkout — the test design made it possible
   to satisfy all ACs with a vacuously-passing implementation. Multiple medium findings
   (F-W86S-P2-007/008/009) revealed that gate tests lacked the failure-mode specification
   needed to make them truly gate-capable.

### Root Cause

Story-writer does not carry a mandatory "positive-coverage checklist" for detector/gate
stories:
- Does this story have an AC that asserts detection FIRES when a real stale input is present?
- Are the positive-coverage fixture inputs sourced from the real finding record (adversarial
  pass convergence reports, PR reviews) rather than from lesson summaries or description text?
- Does the gate test specify an explicit FAILURE mode (not just a success mode)?

### Proposed Fix

Add a **positive-coverage checklist** to the story-writer agent's `STORY.md` template for
`type: maintenance` + `epic: E-11` (tooling/detector) stories:

```
## Positive Coverage Checklist (detector/gate stories)
- [ ] AC asserting the tool exits non-zero on at least one real-world stale input
- [ ] Fixture text sourced from actual finding records (not lesson summaries or descriptions)
- [ ] Gate test specifies failure mode (not only success mode)
- [ ] Efficacy anchor: specific phrase/pattern from a named prior finding (D-NNN citation)
```

This checklist should be mandatory for any story whose primary AC is "tool rejects X" or
"test fails when Y absent".

### Disposition

Carry-forward to wave-086 cycle-close (S-7.02). DF-VALIDATION-001 research-agent validation
required before determining vehicle (local template change vs. upstream vsdd-factory issue).

---

## PG-W86-002 — ground efficacy ACs in source finding record, not lesson summaries

**Class:** Specification grounding discipline / finding-record citation mandate
**Caught by:** Wave-86 adversarial pass 2 (F-W86S-P2-001 CRIT, F-W86S-P2-005 HIGH)
**Severity:** HIGH (CRIT finding in pass-2; efficacy ACs tested against fabricated inputs)
**Occurrences:** 1 story (STORY-183) in wave-86; related to PG-W84-002 extension
**Source finding:** F-W86S-P2-001 (CRIT), F-W86S-P2-005 (HIGH)
**Vehicle:** Local carry-forward (extends PG-W84-010 scope; DF-VALIDATION-001 required)

### Description

STORY-183 v1.1 AC-183-009 specified `Expected RED: TypeID 58` as the D-506 efficacy fixture.
This came from the wave-85 lesson summary (`cycles/wave-085/lessons.md`) which described the
pass-1 adversary finding in lesson-summary prose. The lesson summary said "Expected RED:"
was the stale phrase class — but the actual convergence report
(`cycles/wave-085/STORY-180/convergence-report.md` lines 63-66) showed that the real stale
phrases observed were `currently asserts` and `is expected to`.

The lesson summary was an accurate description of the broader phrase gap, but the specific
finding-record evidence (convergence-report.md) showed different phrase text. Story-writer
cited the lesson summary without cross-referencing the primary finding record.

### Root Cause

When story-writer constructs efficacy ACs that cite prior findings (e.g., "D-506 showed that
pattern X occurs"), it must cite the **primary finding record** (adversarial pass convergence
report, PR review diff, or BC violation log) — not lesson summaries or STATE.md decision
entries, which are secondary digests.

Lesson summaries are intentionally condensed; they may rename or generalize phrase classes
for readability, which causes the derived AC to diverge from the actual stale text.

### Proposed Fix

Extend the story-writer's citation discipline (PG-W84-010 scope) to require that efficacy
ACs citing prior adversarial findings MUST include a direct path citation to the primary
finding record:

```
AC-NNN-YYY: Given input containing `{exact_phrase}` (per {source-doc}:{line}),
  tool exits non-zero.
  Source: cycles/wave-085/STORY-180/convergence-report.md:63-66
```

The `{source-doc}:{line}` citation must point to the primary record, not to STATE.md or
lessons.md. This prevents lesson-summary paraphrase from substituting for ground truth.

### Disposition

Carry-forward to wave-086 cycle-close (S-7.02). Extends PG-W84-010 scope (citation mandate
for story-writer). DF-VALIDATION-001 research-agent validation required.

---

## PG-W86-003 (adjacent) — bin-selftest CI gate gap (PG-W84-012 scope extension)

**Class:** CI gate coverage gap / required-status-checks
**Caught by:** Wave-84 S-7.02 (PG-W84-012, original) + wave-86 gate assessment
**Severity:** MEDIUM (bin/ Python self-tests pass in manual invocation but are not
  enforced as a required GitHub status check on PRs)
**Occurrences:** Persists from wave-84; STORY-183 delivery will add a new bin/ test
  (`bin/test_check_green_doc_tense.py`) that also lacks CI gate enforcement
**Source finding:** PG-W84-012 (original) + STORY-183 scope addition
**Vehicle:** Ops task (devops-engineer dispatch + human authorization) — NOT a story

### Description

`bin/test_compute_input_hash.py` and the forthcoming `bin/test_check_green_doc_tense.py`
both run during the wave-gate `Gate 1` manual CI verification but are NOT listed as required
status checks for `develop` branch protection. This means a PR that breaks a bin/ self-test
can be merged via GitHub UI without CI blocking it.

This gap is not new (PG-W84-012). It is noted here because STORY-183 adds a second bin/
self-test to the same surface, increasing the exposure.

### Cross-Reference

- PG-W84-012 (original finding, D-486): "bin-selftest required-status-check gap; bin/
  Python self-tests pass in Gate 1 but not enforced as required status check".
- STATE.md Active Carry-Forwards: PG-W84-012 row: "Ops task PENDING: bin-selftest →
  develop required-status-checks; devops-engineer + human authorization required for
  branch-protection mutation."

### Disposition

Separate from STORY-183 (different gate surface — branch protection, not tool logic).
Devops-engineer dispatch + human authorization required. Not a story. PG-W84-012 remains
the canonical tracking entry. This row notes the scope extension only.

---

## Summary

| ID | Severity | Status | Vehicle |
|----|----------|--------|---------|
| PG-W86-001 | HIGH | carry-forward, S-7.02 | Local (DF-VALIDATION-001 before filing) |
| PG-W86-002 | HIGH | carry-forward, S-7.02 | Local, extends PG-W84-010 (DF-VALIDATION-001 before filing) |
| PG-W86-003 | MEDIUM | adjacent, scope extension of PG-W84-012 | Ops task (devops-engineer, separate from STORY-183) |
