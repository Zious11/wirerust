# BREAKING-Change Holdout-Expectation Sweep Protocol

**Policy reference:** PG-W72-BREAKING-HOLDOUT-SWEEP  
**Finding reference:** `.factory/cycles/wave-72/lessons.md` Lesson 2 (lines 46-79)  
**Evidence story:** STORY-160 (wave-72, PR #389, squash 704fd2e)  
**Codification story:** STORY-164 AC-164-005  
**Added:** 2026-07-11 (STORY-164 AC-164-005)

---

## Background

STORY-160 (wave-72) introduced a BREAKING JSON change: enum casing from PascalCase to
lowercase/snake_case, plus a `schema_version` envelope. The implementation was correctly
delivered and tested against the story's own test suite. However, 13 holdout scenarios
in `.factory/holdout-scenarios/` had stale `expected_output` fields hard-coded to the old
enum names and structure. None were identified during per-story delivery.

The 13 scenarios — HS-021/024/032/033/034/035/050/054/059/064/065/074/075 — were repaired
by the product-owner at the wave-72 integration gate holdout re-evaluation step, a
significant unplanned gate-time work item. The root cause: no delivery-protocol step
required a sweep of holdout-scenario expectations against the new output format. Holdout
scenarios live in `.factory/holdout-scenarios/`, outside the story's own test suite, so
the standard per-story TDD cycle did not cover them.

Source: `.factory/cycles/wave-72/lessons.md` Lesson 2 (lines 46-79), tag
PG-W72-BREAKING-HOLDOUT-SWEEP.

---

## Scope Trigger

This sweep obligation applies to any story satisfying **at least one** of the following:

1. **BREAKING tag:** the story frontmatter, title, or CHANGELOG entry contains the
   term `BREAKING`.
2. **Observable JSON output schema change:** the story changes field names, field types,
   enum values, enum casing (e.g., PascalCase → lowercase or snake_case), or adds/removes
   a structural envelope (e.g., `schema_version` wrapper).
3. **Observable text output layout change:** the story changes column ordering, header
   format, separator characters, or field labels in terminal or text-format output.

If any trigger applies, the mandatory delivery gate below MUST be completed before the
PR is opened.

---

## Mandatory Delivery Gate (Pre-PR)

Before opening the PR for an in-scope story, the implementer MUST complete all four steps:

1. **Run the holdout evaluator** against the story's output changes: dispatch the
   `vsdd-factory:holdout-evaluator` agent against `.factory/holdout-scenarios/` with the
   story's output changes in scope; evaluations are recorded under the wave's gate
   artifacts.
2. **Identify all stale holdout-scenario expectations** in `.factory/holdout-scenarios/`
   — any scenario whose `expected_output` references old enum names, old JSON schema
   fields, or old text layout.
3. **Repair all stale expectations** to match the new output format.
4. **Record `holdout-expectations-sweep: COMPLETE`** in the story's delivery checklist
   (Tasks section).

A PR opened for an in-scope story without step 4 completed is **non-conforming** per
this protocol (PG-W72-BREAKING-HOLDOUT-SWEEP). The integration gate holdout re-evaluation
will surface stale scenarios as failures; repairing them at gate time is an unplanned
work item that this protocol prevents.

---

## Non-Conformance Consequence

A story PR opened without `holdout-expectations-sweep: COMPLETE` for an in-scope story:

- **Fails the wave integration gate holdout evaluation** — stale scenarios will emit
  failures against the new output format, blocking gate closure.
- **Is subject to gate-time remediation by the product-owner**, an unplanned work item
  charged against gate time rather than story delivery time.
- **Is flagged as a process violation** in the wave lessons for S-7.02 cycle-close
  recording.

The wave-72 precedent (STORY-160, 13 stale scenarios) demonstrates that even a single
BREAKING change can produce a dozen or more stale scenarios spanning many holdout suite
areas.

---

## Wave-72 Evidence

| Item | Detail |
|------|--------|
| Story | STORY-160 (wave-72, PR #389, squash 704fd2e) |
| Change | BREAKING JSON: enum casing PascalCase → lowercase/snake_case + `schema_version` envelope |
| Stale scenarios found at gate | 13: HS-021/024/032/033/034/035/050/054/059/064/065/074/075 |
| HS-INDEX version at repair | v2.13 |
| Repaired by | Product-owner at wave-72 integration gate holdout re-evaluation |
| Source | `.factory/cycles/wave-72/lessons.md` Lesson 2 (lines 46-79) |
| Tag | PG-W72-BREAKING-HOLDOUT-SWEEP |

---

## Reference

- **PG-W72-BREAKING-HOLDOUT-SWEEP:** Root process-gap tag (wave-72 Lesson 2, 2026-07-09).
- **`.factory/cycles/wave-72/lessons.md` Lesson 2 (lines 46-79):** Source observation and
  candidate-codification record, human-approved 2026-07-11. Codified by STORY-164
  AC-164-005.
- **STORY-160:** The BREAKING story whose 13 stale holdout scenarios motivated this
  protocol.
- **STORY-164 AC-164-005:** Codification story for this protocol.
