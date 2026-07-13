# Lessons Learned — wave-75

S-7.02 cycle-closing requirement: lessons recorded here for human review and
engine-improvement triage.

Wave: 75 | Gate passed: 2026-07-13 (D-435) | Wave CLOSED: 2026-07-13 (D-435, human-approved 2026-07-13) | Stories: STORY-165 (3 pts).
PRs merged: #398 (fa646ed, STORY-165; squash-merged 2026-07-13).
Adversarial convergence (wave-level): 7 passes; streak 3/3 (W5/W6/W7); trajectory 2→0→0→1→0→0→0.
Per-story adversarial convergence (STORY-165): 9 passes, streak 3/3 (P7/P8/P9), trajectory 1→0→0→2→0→1→0→0→0.

---

## Lesson 1 — [codified→STORY-166] Fabricated-Citation Defect Class Recurred Twice Per-Story

**Observation:**

The wave-75 per-story adversary surfaced two HIGH fabricated-citation findings despite
STORY-165 being explicitly a tooling story about citation validation:

**F-S165P1-001 HIGH (fabricated test name):** STORY-165 delivery doc cited
`test_validate_citations.py` with a specific test function name at an in-bounds line. The
function name did not exist at that line — `bin/validate-citations` validated the line was
in range but could not assert the named symbol was present. This is exactly the class of
gap STORY-165's AC-164-002 (PG-W73-CITATION-VALIDATOR) was designed to close in a future
delivery — the citation mandate's own evidence instantiated the defect it described.

**F-S165P4-001 HIGH (fabricated finding ID):** The delivery doc cross-referenced a prior
wave-74 finding as `F-W74P8-001` (G-less form, wrong pass number). The canonical ID was
`F-W74G-P3-001` (G-form). Both errors — non-canonical form AND wrong pass number — stem
from the dual-ID-scheme ambiguity. Codified as PG-W75-FINDING-ID-DUAL-SCHEME.

**Root causes:** (1) `bin/validate-citations` cannot assert symbol-at-line (PG-W75-VALIDATE-
CITATIONS-SYMBOL-GAP). (2) Two colliding wave-gate finding-ID schemes (`F-W<NN>G-P<n>` canonical
vs `F-W<NN>P<n>` G-less) exist repo-wide; authors default to the shorter non-canonical form
and misnumber passes without a naming-policy to enforce (PG-W75-FINDING-ID-DUAL-SCHEME).

**Codification:** STORY-166 AC-166-001 (symbol-at-line validator extension) + AC-166-002
(finding-ID naming policy in `.factory/policies.yaml`).

**Tags:** `codified`, `fabricated-citation`, `symbol-at-line`, `finding-id-scheme`, `recurrence`

---

## Lesson 2 — [observation] First Currency-Sweep Execution Itself Gate-Hardened

**Observation:**

Wave-75 was the first wave to execute the delivery-doc currency sweep (AC-165-003) as a
mandatory pre-adversary gate step. Despite running the sweep before adversary Pass 1, two
currency-class findings still reached the wave gate:

- F-W75G-P1-002 LOW: STORY-165.md line citation off by 2 (lines 81-82, not line 84).
- F-W75G-P4-001 MEDIUM: currency-sweep.md itself had a blanket-provenance claim aggregating
  all four demo-evidence files under a single worktree attribution — corrected to per-method headers.

This demonstrates that the currency sweep catches macro drift (version numbers, status
fields, major structural staleness) but fine-grained line citations and internal
sweep-document claims are a distinct sweep surface. The gate caught both findings
correctly; the sweep did not create false confidence.

Additionally, F-W75G-P3-002 (a finding against the process-gap-ledger) was research-
adjudicated as ledger-redundant. The research-agent validation correctly identified the
overlap with PG-W75-FINDING-ID-DUAL-SCHEME and dismissed F-W75G-P3-002 without streak
impact. The principle: the checker (currency sweep, ledger) must itself be checked by the
gate, and research-adjudication is the correct triage path for ledger-redundancy claims.

**Tags:** `observation`, `currency-sweep`, `gate-hardening`, `checker-gets-checked`

---

## Lesson 3 — [applied] Canonical G-Form Finding IDs Used Throughout Wave-75 Gate

**Observation:**

Wave-75 gate artifacts (findings.md, gate-summary.md, this lessons.md, and the
process-gap-ledger.md) use the canonical `F-W<NN>G-P<n>-<seq>` form throughout —
the same form that STORY-166 AC-166-002 will codify as policy. This was a deliberate
dogfood-fix ahead of the formal policy: by using only G-form IDs in wave-75 gate artifacts,
the gate itself becomes a concrete example of the naming convention it is codifying.

This practice eliminates the "two forms in live use" problem within wave-75's own artifact
set. Future waves will be able to reference wave-75 gate findings using only the canonical
form without ambiguity.

**Tags:** `applied`, `finding-id-canonical`, `dogfood`, `self-referential-codification`

---

## Lesson 4 — [codified→STORY-166] Mid-Gate Streak Persistence Gap Observed

**Observation:**

During wave-75 gate passes W6 and W7 (CLEAN passes, streak #2 and #3), the wave-gate
`findings.md` log was not updated to record those CLEAN passes. A reader checking mid-gate
progress could not confirm the streak had persisted through W6 and W7 without reading the
adversarial review artifacts directly — the findings.md provided no incremental CLEAN-pass
rows.

This is a distinct gap from the F-W75G-P4-001 currency fix (which corrected a provenance
claim within the log). The streak-persistence gap is about completeness of the gate-progress
log: every pass, CLEAN or not, should add a row so mid-gate state is legible without
cross-referencing multiple artifacts.

**Codification:** STORY-166 AC-166-004 (extend wave-gate and per-story findings logs to
record CLEAN-pass rows with running streak count).

**Tags:** `codified`, `streak-persistence`, `gate-log-completeness`, `w6-observation`

---

## Lesson 5 — [observation] Row-Verify Mandate First Compliant Execution on Its Own PR

**Observation:**

STORY-165's PR #398 was the first delivery PR executed under the row-verify mandate
(AC-165-002, PG-W74-PRDESC-ROW-VERIFY): 9 test-evidence table rows were cross-checked
against actual CI output before submission. The per-story gate confirmed all 9 rows were
accurate — no fabricated counts, no stale references.

A self-referential test: the mandate's own delivery PR was the first to run the mandate.
The mandate passed its own test. No meta-irony gap was found (unlike the citation-mandate
story STORY-164 where a fabricated citation was caught in the citation mandate's own evidence
at P5 — see wave-74 lessons). This is a positive data point but does not eliminate the
need for continued vigilance; fabricated-citation defects in the same wave at per-story
level (F-S165P1-001, F-S165P4-001) show the class remains live.

**Tags:** `observation`, `row-verify-mandate`, `first-compliant-execution`, `self-referential`
