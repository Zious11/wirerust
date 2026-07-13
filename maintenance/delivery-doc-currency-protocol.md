# Delivery-Doc Currency Sweep Protocol

**Policy reference:** PG-W74-DELIVERY-DOC-CURRENCY  
**Finding references:** F-W74P1-001 (gate Pass 1) + F-W74P13-001 (gate Pass 13)  
**Codification story:** STORY-165 AC-165-003  
**Added:** 2026-07-13 (STORY-165 AC-165-003)

---

## Background

Multiple late adversarial passes at the wave-74 gate were consumed fixing
delivery-narrative artifacts that described pre-delivery state as current after
STORY-164 had already been merged:

- **Pass 1 (F-W74P1-001):** STORY-164 status field still read `ready` after PR #397
  merged (`.factory/stories/STORY-164.md:660`). The loci-agreement correction — updating
  frontmatter `status:`, body header, and STORY-INDEX index cell — is a mechanical check
  that does not require adversarial analysis; it is verifiable before any pass begins.
- **Pass 13 (F-W74P13-001):** The Background section described "Current gate
  implementation" using present tense for the pre-STORY-164 changelog-gate behavior
  (`.factory/stories/STORY-164.md:655`). Reframing as historical ("Pre-STORY-NNN
  implementation (as of develop SHA / vX.Y.Z): ...") after delivery is a tense-audit
  correction that a pre-gate sweep would have caught before any adversarial pass opened
  that section.

The 12-pass gap between a mechanical status correction (Pass 1) and a tense correction
(Pass 13) illustrates that staleness comes in layers. A pre-gate currency sweep catches
all layers at once rather than requiring a separate adversarial pass for each.

Root cause: no explicit wave-gate-entry step required a full currency sweep of delivery
documents before the adversarial pipeline started.

**W5 scheduler-vocabulary advisory (folded):** wave-74 gate Pass 5 noted a scheduler
vocabulary boundary observation (the distinction between "scheduler" and "orchestrator"
terminology in wave-gate-entry documentation). The currency sweep defined in this
protocol covers wave-gate-entry documentation terminology alongside story spec tense.

---

## Scope Trigger

This sweep is performed **once per wave**, before the first adversarial pass of the wave gate begins (per-story Step-4.5 convergence passes are NOT in scope).
Per-story Step-4.5 adversarial convergence (Perimeter 1) is explicitly out of scope; this sweep is a wave-gate-entry (Perimeter 2) obligation.
It applies to all delivery-narrative artifacts associated with the wave's stories:

- Story spec files (`.factory/stories/STORY-NNN.md`) for every story assigned to the wave
- Demo-evidence artifacts (`.factory/demo-evidence/`)
- Maintenance docs created or amended by the wave's stories
- Wave-gate-entry documentation (gate summaries, lessons, scheduler/orchestrator
  terminology in gate-entry prose)

---

## Mandatory Sweep Steps

Before the first adversarial pass of a wave gate, the operator MUST complete all three
steps:

### Step 1 — Status Loci Check

Verify that all three loci agree and reflect the current delivery state for every story
assigned to the wave:

1. Frontmatter `status:` field
2. Body status line (e.g., `**Status:** delivered`)
3. STORY-INDEX index cell

Recognized delivery-class values: `draft`, `ready`, `pending`, `delivered`, `merged`,
`completed`, `superseded` (per AC-164-001(c) loci agreement rule). All three loci MUST
show the same delivery-class category. A mismatch is a mechanical error — correct it
before opening the first adversarial pass of the wave gate.

### Step 2 — Tense Audit

Scan story Background and Acceptance Criteria sections for present-tense references to
implementation behavior that describe the pre-delivery state as "current" after delivery.
Examples of stale phrases:

- "Current gate implementation"
- "The gate currently..."
- Inline bash/code blocks copied from pre-delivery state that AC changes have superseded

Reframe any such references as historical. The canonical reframing pattern is:

> Pre-STORY-NNN implementation (as of develop COMMITSHA / vX.Y.Z): [original claim]

This pattern anchors the historical claim to the commit and version that preceded the
story's delivery, making the reframing unambiguous to future readers.

Also cover wave-gate-entry documentation for vocabulary accuracy — confirm that terms
like "scheduler" and "orchestrator" are used at their correct scope boundaries in any
gate-entry prose.

### Step 3 — Demo-Evidence Currency Notes

Review demo-evidence artifacts for any counts, code excerpts, or behavioral claims that
have been superseded by the wave's delivery. Add inline currency notes per the pattern
established in `.factory/demo-evidence/story-164/AC-164-001.md:67`:

> [Currency note — F-NNN, YYYY-MM-DD: brief description of what changed and why the
> original capture is now stale. New sed range or grep command if applicable.]

Zero stale items found is a valid outcome — record the sweep completion record (see Currency Sweep Record below)
and proceed. Do not add spurious currency notes where none are needed.

---

## Currency Sweep Record

Sweep completion MUST be recorded before the first adversarial pass of the wave gate. A single one-line
note is sufficient:

> **Currency sweep: COMPLETE (YYYY-MM-DD)**

Place this line in the wave gate summary or gate-entry checklist. Omitting the sweep
record is non-conforming. The first adversarial pass of the wave gate MUST verify that the record exists
before proceeding; if the record is absent, the pass MUST record a finding requiring the
sweep to be completed before review continues.

A wave with zero stale items MUST still include the sweep record — the record attests
that the sweep was performed, not merely that stale items were corrected.

---

## Non-Conformance Consequence

A wave gate opened without completing the currency sweep:

- **Burns adversarial-convergence capacity** on mechanical corrections — status
  mismatches and stale tense are not adversarial findings; they are editorial corrections
  that block the adversary from concluding its pass until they are resolved.
- **Forces re-review** of sections already approved once staleness is found, consuming
  additional passes on sections that do not contain behavioral gaps.
- **Is flagged as a process violation** in the wave lessons for S-7.02 cycle-close
  recording (the wave-74 precedent: two adversarial passes consumed on corrections
  avoidable by a pre-gate sweep).

---

## Wave-74 Evidence

| Item | Detail |
|------|--------|
| F-W74P1-001 | Pass 1 — STORY-164.md status `ready` → `delivered`; all loci corrected. Source: `.factory/stories/STORY-164.md:660` |
| F-W74P13-001 | Pass 13 — Background "Current gate implementation" reframed as `Pre-STORY-164 implementation (as of develop b5e1e15 / v0.12.0)`. Source: `.factory/stories/STORY-164.md:655` |
| Gap between passes | 12 passes separated a mechanical status correction from a tense correction — both were catchable by a pre-gate sweep |
| Currency note pattern | `.factory/demo-evidence/story-164/AC-164-001.md:67` — bracket note added to existing evidence artifact on superseded counts and stale sed ranges |

---

## Reference

- **PG-W74-DELIVERY-DOC-CURRENCY:** Root process-gap (wave-74 gate Passes 1 and 13,
  2026-07-11). Direct cause of this protocol.
- **F-W74P1-001:** Finding — status loci mismatch, wave-74 gate adversarial Pass 1.
  Source: `.factory/stories/STORY-164.md:660`.
- **F-W74P13-001:** Finding — stale "Current gate implementation" tense, wave-74 gate
  adversarial Pass 13. Source: `.factory/stories/STORY-164.md:655`.
- **AC-164-001(c):** Loci agreement rule — the three loci (frontmatter, body header,
  STORY-INDEX cell) must agree on delivery-class category.
- **`.factory/demo-evidence/story-164/AC-164-001.md:67`:** Currency note pattern example.
- **STORY-165 AC-165-003:** Codification story for this protocol.

---

## Correction Record

| Finding | Date | Change |
|---------|------|--------|
| F-S165P6-001 | 2026-07-13 | Scope trigger wording disambiguated at all loci: "before any adversarial convergence pass begins" → "before the first adversarial pass of the wave gate begins (per-story Step-4.5 passes NOT in scope)". Clarifying sentence added to Scope section. Lines 71, 110, 116 (original numbering) qualified with "of the wave gate". Sibling locus in STORY-165.md AC-165-003(a) fixed in same burst per DF-SIBLING-SWEEP-001. |
| F-S165P7-001 | 2026-07-13 | Editorial: dangling cross-reference "(Step 4)" at line 104 replaced with "(see Currency Sweep Record below)". The document declares three steps; no Step 4 exists. STORY-165.md confirmed clean (zero occurrences of "Step 4"). |
