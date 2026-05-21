---
pass: 4
scope: story-decomposition
date: 2026-05-21
verdict: NOT_CONVERGED
findings: "0C/3H/5M/4L/3N"
total_findings: 15
convergence_counter: 0/3
---

# Story Adversary Pass 4 — Story Decomposition Review

**Date:** 2026-05-21
**Scope:** Story decomposition (STORY-*.md files) + holdout scenarios (HS-001..100.md, HS-INDEX.md)
+ dependency graph (dependency-graph.md) + BC-INDEX.md
**Verdict:** NOT_CONVERGED — 0C/3H/5M/4L/3N (15 findings total; 8 blocking HIGH/MEDIUM findings
all remediated)
**Convergence counter:** 0/3 (streak does not advance; Pass 5 next)

## Pass-3 Regressions Check

All Pass-3 blocking findings (F-1 HIGH epic-points table, F-2 MEDIUM transitive-dependency
inconsistency, F-4 LOW HS-094 filler BC cite) verified clean. F-3 INVALID finding confirmed
non-applicable. No regressions on any prior finding.

## Finding Summary

### HIGH

**H-1 — STORY-088 VP-018 AC trace annotation absent**
`STORY-088.md` AC-009 (the acceptance criterion covering the case-sensitivity behavior for
protocol-name fields governed by VP-018) lacked the `[VP-018]` trace annotation required for
any AC that is the primary behavioral-contract witness for a Verification Property. The annotation
protocol is established in the Phase-2 story-template and was applied consistently across all
other VP-bearing stories. Absence of the annotation breaks the forward traceability chain
VP-018 → STORY-088 → AC-NNN.
**Remediated:** `[VP-018]` trace annotation added to the relevant acceptance criterion in
`STORY-088.md`. Traceability chain restored.

**H-2 — BC-INDEX.md ingestion-count arithmetic self-contradiction**
The BC-INDEX.md header commentary contained two distinct derivations of the total BC count
(217) that were arithmetically inconsistent with each other. One derivation listed per-subsystem
subtotals that summed correctly to 217; a second, legacy derivation block cited an older
intermediate count that did not reconcile with the current per-subsystem table. This
self-contradiction had persisted through all 33 Phase-1 adversary passes because the two
derivation blocks appeared in different sections of the index header and were not co-located.
The finding demonstrates that cross-artifact propagation drift can survive an exhaustive
convergence run when inconsistent data occupies semantically-distinct document regions not
checked in the same review pass.
**Remediated:** BC-INDEX.md reconciled to a single canonical derivation consistent with the
217-BC per-subsystem table. The legacy arithmetic block replaced with the authoritative
per-subsystem sum.

**H-3 — dependency-graph.md stale STORY-086 provenance note**
`dependency-graph.md` contained a provenance note for STORY-086 that cited a superseded
rationale. The note referenced an earlier story-decomposition decision that was revised during
Pass-3 remediation (the transitive-dependency correction for the E-5 epic cluster), but the
dependency-graph provenance annotation was not updated in the same burst. The stale note
created a contradiction between the graph's own annotation and the current story frontmatter
state.
**Remediated:** STORY-086 provenance note in `dependency-graph.md` corrected to reflect the
current direct-edge graph state.

### MEDIUM

**M-1 — HS-051..100.md generic `inputs` fields (50 holdout scenarios)**
All 50 holdout scenarios in the `HS-051..100.md` file used generic placeholder text in their
`inputs` frontmatter field (e.g., `"sample_pcap"`, `"packet_sequence"`) and lacked a
`traces_to` field linking each scenario to the specific story or BC it exercises. Generic
inputs reduce the scenarios' value as independent audit instruments: a holdout scenario should
specify a concrete, named, scenario-specific input artifact (real PCAP, constructed byte
sequence, or crafted command) so that the scenario can be replayed mechanically. The `traces_to`
field is required by the HS-INDEX schema for wave-scheduling and coverage audits.
**Remediated:** All 50 HS-051..100.md files updated: `inputs` fields replaced with
scenario-specific concrete inputs (named PCAP fixtures, constructed byte sequences, or CLI
invocations appropriate to each scenario class), and `traces_to` fields added citing the
governing story and BC for each scenario.

**M-2 — HS-INDEX.md per-wave Count column ambiguity**
The per-wave summary table in `HS-INDEX.md` listed scenario counts per wave without clarifying
whether the count represented scenarios assigned to that wave or scenarios whose governing story
first appears in that wave. This ambiguity was actionable because several scenarios exercise
behaviors that span multiple waves (integration-boundary and real-world-corpus classes in
particular), and the count interpretation affected whether coverage was verifiably complete for
each wave gate.
**Remediated:** HS-INDEX.md header note corrected to clarify the Count column semantics
(scenarios whose primary governing story first appears in the listed wave) and a clarifying
note added to the per-wave table header.

**M-3 — STORY-088 missing AC-010 case-sensitivity criterion + ACs renumbered**
`STORY-088.md` was missing an explicit acceptance criterion covering the case-sensitivity
behavior for protocol-name fields (the behavior governed by VP-018 and BC-2.04.xxx). The
VP-018 trace added as H-1 remediation identified the gap: the relevant acceptance criterion
should explicitly state that protocol-name matching is case-insensitive and that both uppercase
and lowercase inputs produce identical classification results. ACs were renumbered after
insertion to maintain sequential ordering.
**Remediated:** AC-010 added to `STORY-088.md` with explicit case-sensitivity predicate.
Subsequent ACs renumbered. VP-018 trace annotation applied to AC-010.

**M-4 — STORY-096 tdd_mode field uses non-standard value + Red Gate task missing absence-proof**
`STORY-096.md` frontmatter contained `tdd_mode: tdd_mode` (a self-referential placeholder)
rather than the standard `facade` value appropriate for a story that exercises a public API
facade. Additionally, the Red Gate (failing-test) task description did not include the
absence-proof obligation: for facade stories, the Red Gate test must assert that the facade
method does not exist yet (compile error / unresolved symbol), not merely that it returns the
wrong value. The absence-proof formulation is required to prevent the Red Gate from being
trivially satisfied by a pre-existing method stub.
**Remediated:** `tdd_mode` corrected to `facade` in `STORY-096.md` frontmatter. Red Gate task
rewritten to specify the absence-proof formulation (assert compile error on missing facade
entry point).

**M-5 — STORY-003 cargo-fuzz harness absent from ACs; VP-008 harness promoted to mandatory**
`STORY-003.md` covered the behavioral contracts governing the packet-parsing subsystem (SS-04),
which is the primary target of VP-008 (fuzzing harness). The story's acceptance criteria listed
unit tests and integration tests but did not include an AC requiring the cargo-fuzz harness to
be wired, compiled, and runnable. VP-008 mandates a fuzz harness for the parsing surface; a
story that delivers the parsing subsystem without an AC requiring the harness creates a coverage
gap where the fuzz harness could be silently omitted during Phase 3 implementation.
**Remediated:** AC-011 added to `STORY-003.md` requiring the cargo-fuzz harness to compile and
run (at least one seed corpus cycle without UB/panic). A corresponding implementation task
added. VP-008 explicitly cited in AC-011.

### LOW

**L-1 — HS-INDEX.md header note inconsistency (pre-M-2 remnant)**
Minor wording inconsistency in the HS-INDEX.md header note block, distinct from the M-2
Count-column ambiguity, concerning the definition of "must-pass" vs "should-pass" scenario
classification. The note conflated the two terms in one paragraph.
**Disposition:** Corrected as part of M-2 remediation (HS-INDEX.md header note rewrite).

**L-2 — STORY-096 description inconsistent with facade pattern**
The description prose in `STORY-096.md` described the story objective in terms of an
"implementation detail" rather than the public API boundary that a facade story should document.
**Disposition:** Description updated to facade-pattern framing as part of M-4 remediation.

**L-3 — STORY-003 task list ordering (fuzz task position)**
The implementation task list in `STORY-003.md` placed the cargo-fuzz task at the end of the
list after integration-test tasks, which is not the standard ordering (fuzz harness wiring
should precede integration tests that depend on parser stability).
**Disposition:** Task order corrected as part of M-5 remediation (fuzz task moved before
integration-test tasks).

**L-4 — dependency-graph.md provenance note formatting**
The provenance note block for STORY-086 used inconsistent markdown formatting relative to
all other provenance note blocks in the file.
**Disposition:** Formatting normalized as part of H-3 remediation.

### NITPICK

**N-1 — Process gap: VP trace annotations not machine-validated across all stories**
H-1 (missing VP-018 annotation on STORY-088) and M-3 (missing AC-010) together reveal that
VP → story → AC traceability is not mechanically validated. A VP can be "covered" by a story
in the traceability matrix while the specific AC providing that coverage lacks the annotation.
**Disposition:** [process-gap] — deferred for cycle-close codification. Not a story content
defect.

**N-2 — Process gap: holdout-scenario `inputs` field not schema-validated**
M-1 (generic inputs in HS-051..100) could only be detected by reviewing each scenario file
individually. No schema validator asserts that `inputs` fields contain non-generic, scenario-
specific values.
**Disposition:** [process-gap] — deferred for cycle-close codification. Not a scenario content
defect once remediated.

**N-3 — Process gap: story tdd_mode values not enumeration-validated**
M-4 (self-referential `tdd_mode: tdd_mode` placeholder) reveals that frontmatter enum fields
are not validated against the allowed-values list. A linter that rejects unknown tdd_mode
values would have caught this at story-creation time.
**Disposition:** [process-gap] — deferred for cycle-close codification. Not a story content
defect once remediated.

## Remediation Status

| Finding | Severity | Remediated? |
|---------|----------|-------------|
| H-1 | HIGH | YES — VP-018 trace annotation added to STORY-088.md |
| H-2 | HIGH | YES — BC-INDEX.md reconciled to single canonical derivation |
| H-3 | HIGH | YES — STORY-086 provenance note in dependency-graph.md corrected |
| M-1 | MEDIUM | YES — all 50 HS-051..100.md files updated with concrete inputs + traces_to |
| M-2 | MEDIUM | YES — HS-INDEX.md Count column semantics clarified + header note corrected |
| M-3 | MEDIUM | YES — AC-010 case-sensitivity criterion added to STORY-088.md; ACs renumbered |
| M-4 | MEDIUM | YES — tdd_mode corrected to facade; Red Gate task rewritten for absence-proof |
| M-5 | MEDIUM | YES — AC-011 cargo-fuzz harness added to STORY-003.md with VP-008 cite |
| L-1 | LOW | YES — resolved as part of M-2 remediation |
| L-2 | LOW | YES — resolved as part of M-4 remediation |
| L-3 | LOW | YES — resolved as part of M-5 remediation |
| L-4 | LOW | YES — resolved as part of H-3 remediation |
| N-1 | NITPICK | DEFERRED — cycle-close process-gap codification |
| N-2 | NITPICK | DEFERRED — cycle-close process-gap codification |
| N-3 | NITPICK | DEFERRED — cycle-close process-gap codification |

**All 8 blocking findings (3H + 5M) remediated. All 4 LOW findings resolved as co-remediation
side-effects. 3 process-gap NITPICKs deferred for cycle-close codification.**
Pass 5 may be dispatched.

## Finding Trajectory Across Story-Review Passes

| Pass | Findings | Severity Breakdown | Verdict |
|------|----------|--------------------|---------|
| 1 | 11 | 1C/3H/3M/2L/2N | NOT_CONVERGED |
| 2 | 7 | 0C/1H/2M/2L/2N | NOT_CONVERGED |
| 3 | 6 | 0C/1H/1M/2L/2N | NOT_CONVERGED (1 invalid) |
| 4 | 15 | 0C/3H/5M/4L/3N | NOT_CONVERGED |

**Trajectory note — NON-MONOTONIC:** Pass 4 surfaced 15 findings against Pass 3's 6, a
regression in raw count. This is not a quality regression in the remediated artifacts; it
reflects breadth expansion: Pass 4 extended coverage to BC-INDEX.md cross-artifact propagation
drift (H-2), the full HS-051..100 holdout batch (M-1, 50 files), and previously un-audited
STORY-096/STORY-003 story internals. The H-2 finding is notable: the BC-INDEX arithmetic
self-contradiction had been present since before Phase 1 began and survived all 33 Phase-1
adversary passes because the inconsistent derivation blocks occupied different document sections
that were not co-located in any single review pass. Non-monotonic trajectory is expected when
pass scope expands to previously unreviewed artifact classes. Blocking severity is 0C, so
convergence trajectory remains valid.
