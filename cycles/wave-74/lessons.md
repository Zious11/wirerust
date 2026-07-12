# Lessons Learned — wave-74

S-7.02 cycle-closing requirement: lessons recorded here for human review and
engine-improvement triage.

Wave: 74 | Gate passed: 2026-07-12 (D-432) | Wave CLOSED: 2026-07-12 (D-432, human-approved 2026-07-12) | Stories: STORY-164 (4 pts).
PRs merged: #397 (d6e3be8, STORY-164; squash-merged 2026-07-11T23:04:56Z by human).
Adversarial convergence (wave-level): 13 passes; streak 3/3 (W11/W12/W13); trajectory 2→0→1→1→0→1→0→3→1→1→0→2n→1n.
Per-story adversarial convergence (STORY-164): 8 passes, streak 3/3 (P6/P7/P8), trajectory 6→4→3→2n→2→1n→2n→1n.

---

## Lesson 1 — [codified→STORY-165] Four PG-W74 Process Gaps Recorded and Codified

**Observation:**

The wave-74 gate identified four process gaps across its 13 adversarial passes, each surfacing
a recurring class of quality failure that was not covered by existing gating mechanisms:

**PG-W74-CI-BIN-SELFTEST (AC-165-001):** The CI pipeline does not run self-tests for
`bin/` Python tooling as part of the standard test suite. `bin/validate-citations`
ships with a 22-test suite (`bin/test_validate_citations.py`) and
`bin/changelog-gate-check` ships with a 10-test suite (`bin/test_changelog_gate_content.py`),
but neither is executed by `cargo test --all-targets`. During wave-74 gate, adversarial
passes W3/W6/W10 caught test-infrastructure drift (fabricated PR test table, stale doc
claims) that a CI-integrated bin/ self-test would have caught automatically at every PR.

Evidence: W3 finding F-W74G-P3-001 (fabricated PR test table reference in delivery doc);
W6 finding F-W74G-P6-001 (stale Python test count claim); W10 finding F-W74G-P10-001
(delivery doc assertions diverging from shipped test behavior).

**PG-W74-PRDESC-ROW-VERIFY (AC-165-002):** PR descriptions containing a "Test evidence"
or similar table are not verified against the actual CI run output. Wave-74 W3 caught a
fabricated test-count row in a delivery doc that claimed a count that did not match any
real test run. The absence of a mechanical cross-check between claimed counts and CI
artifacts creates a class of misleading delivery evidence.

Evidence: F-W74G-P3-001 caught a delivery-doc row claiming "22 + 10 = 32 tests pass" in a
context where the count had drifted. The fix required aligning the doc to the actual
`python3 -m pytest` output.

**PG-W74-DELIVERY-DOC-CURRENCY (AC-165-003):** Delivery documents (pr-description.md,
demo-evidence/*/AC-*.md) carry timestamp and version assertions that can become stale
when subsequent wave-gate passes amend the story spec. Wave-74 W1, W4, W8 all found
currency drift: W1 caught two delivery-doc locations asserting STORY-164 v1.10 status
while the story had been amended to v1.11 at gate time; W4/W8 found demo evidence
paths and STORY-INDEX currency dates lagging behind gate-pass amendments.

Evidence: F-W74G-P1-001 (v1.10 status assertions stale), F-W74G-P4-001 (demo index
date stale), F-W74G-P8-001/P8-002 (two additional currency locations not swept by P4 fix).

**PG-W74-GROUND-TRUTH-AUDIT-FIRST (AC-165-004):** Multiple wave-74 gate passes contained
adversarial claims that were refuted by ground-truth audit (actual file contents, actual
test counts, actual status values) — yet the adversary had reached a MEDIUM or HIGH
severity verdict before the audit step. The pattern of "claim first, check second" at the
adversary layer was the root cause of the P9/P10 accepted findings (adversary's MEDIUM
claim was refuted by orchestrator reading the file directly). A "ground-truth audit first"
discipline — where the adversary reads the exact file section before filing a finding —
would reduce false positives and avoid the "fix-then-refute" sequence.

Evidence: W9 finding F-W74G-P9-001 (adversary claimed stale status at a locus; orchestrator
ground-truth audit showed status was already updated; ACCEPTED-REFUTED). W10 finding
F-W74G-P10-001 (adversary claimed missing coverage annotation; ground-truth read showed
annotation was present in a different section; ACCEPTED-REFUTED).

**Codification:** STORY-165 v1.0 drafted (wave-TBD, E-11, 3 pts) codifying all four process
gaps as AC-165-001 through AC-165-004.

**Tags:** `codified`, `ci-bin-selftest`, `doc-currency`, `ground-truth-audit`, `wave-gate-value`

---

## Lesson 2 — [register] Engine Relay Silent-Idle Pattern: ADVERSARY-RELAY-UNRELIABLE-001 Update

**Observation:**

During wave-74's 13-pass gate, five additional silent-idle incidents were observed where
adversary agent instances wrote their output artifact but did not relay findings back to
the orchestrator session, requiring synchronous re-dispatch. This session (2026-07-11/12)
produced the highest concentration of relay-silent incidents observed in a single session.

All five incidents followed the same pattern: adversary writes a cycle-file artifact (e.g.,
`cycles/wave-74/adversarial-reviews/pass-N.md`), reports a brief artifact-written
confirmation, then goes idle before delivering the structured finding list to the
orchestrator. The orchestrator must detect the silence (usually within 2-3 minutes), then
re-dispatch the same adversary synchronously (`run_in_background: false`) to recover the
findings.

The pattern reliably resolves on synchronous re-dispatch. The artifact written in the
first (silent-idle) pass is valid; the re-dispatch reads it and produces the summary. The
workaround is stable but inefficient — each silent-idle incident costs one extra dispatch
per pass.

Updated count: 5+ silent-idle incidents this session (2026-07-11/12) added to prior count.
ADVERSARY-RELAY-UNRELIABLE-001 tech-debt entry updated accordingly (now 8+ lifetime
occurrences across maint-2026-07-08 [2], maint-2026-07-11 [1], wave-74 gate [5+]).

**Mitigation (already in effect):** Synchronous adversary dispatch (`run_in_background:
false`) for all adversary passes during wave-74 gate, per PROP-V0.12.0-03 recommendation
(D-423). Eliminates most incidents but not all — some synchronous dispatches also silenced
before returning the structured summary.

**Tags:** `engine-note`, `adversary-relay`, `workaround-stable`, `recurrence-5+`

---

## Lesson 3 — [observation] Wave-Gate Cost Profile for Single-Story Governance Waves

**Observation:**

Wave-74 required 13 adversarial passes at the wave-level gate for a 1-story governance
wave. The per-story adversarial convergence required 8 passes. Combined: 21 adversarial
passes total for a single 4-point story.

By comparison, wave-73 (2-story, 5 pts, both E-11) required 6 wave-level passes. Wave-72
(4-story, larger scope) required 10+ passes total. The governance-story class (E-11) appears
to generate a higher per-point adversarial cost than implementation stories because governance
stories produce documentation artifacts that are dense with verifiable claims (line anchors,
version numbers, status fields, test counts) — each claim is a potential adversary target.

**Wave-74 gate value was real:** 8 substantive defects were caught post-merge that would
have been left in the factory-artifacts tree (status-legend corpus contradictions ×3,
fabricated PR test table, demo/currency staleness ×3, historical-framing inversion). These
were not latent issues — they would have caused confusion at the next session start. The
gate cost was commensurate with the value for this cycle.

**Human accepted this observation. Future single-story governance waves may warrant a
gate-profiling discussion at session review** — not to reduce rigor, but to explore whether
a pre-gate self-validation sweep (e.g., running bin/validate-citations across all delivery
docs before dispatching wave adversary Pass 1) could compress the early-pass churn and
reduce the total pass count.

**Tags:** `observation`, `cost-profile`, `governance-wave`, `gate-efficiency`

---

## Lesson 4 — [codified-in-wave] Guidance §4 Usability and Protocol Step-1 Operability Fixed

**Observation:**

During the wave-74 gate pass W3 review, two quality gaps in wave-level guidance documents
were identified and corrected during the gate (not deferred to the next wave):

1. **docs-writer-dispatch-guidance.md §4 usability note:** The guidance document lacked a
   concrete example showing how a docs-writer agent should resolve ambiguous citation
   anchors (the most common cause of fabricated citations caught at P1). A worked example
   was added during gate remediation (W3 observation, fixed before W4).

2. **breaking-change-delivery-protocol.md step-1 operability:** Step 1 of the protocol
   had an underspecified "locate stale holdouts" instruction that did not give a concrete
   grep command. The protocol was amended to include the canonical `grep -r "Old.*Value"
   .factory/holdout-scenarios/` command (with the appropriate pattern), making step 1
   mechanically executable rather than interpretive.

Both fixes were applied directly to `.factory/maintenance/` artifacts during the wave-74
gate bursts. These are W3-observed issues, corrected pre-W4 dispatch.

**Tags:** `codified-in-wave`, `guidance-quality`, `protocol-operability`, `w3-observation`
