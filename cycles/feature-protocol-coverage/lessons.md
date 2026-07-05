---
document_type: lessons-learned
cycle: feature-protocol-coverage
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-07-05T12:00:00Z
traces_to: STATE.md
---

# Lessons Learned — feature-protocol-coverage

Captured per S-7.02 cycle-close requirement. Recurring [process-gap] findings are codified as follow-up
stories rather than hand-fixed each cycle.

---

## [codified] PG-INDEX-DRIFT-001 — STORY-INDEX status not updated on story PR merge

**Tag:** [process-gap, S-7.02, codified]
**Surfaced:** F7 consistency audit (D-381, 2026-07-05) as finding P0-001
**Codified as:** STORY-155 (E-11, wave TBD, 3 pts, draft)
**Filed:** 2026-07-05 (D-382)

### Description

When a story PR squash-merges to develop, the per-story-delivery flow does not update that story's
STORY-INDEX row from `status: draft` to `merged`. The index drifts out of sync with reality.

The E-21 F7 consistency audit caught this as P0-001: STORY-151, STORY-152, STORY-153, and STORY-154
all showed `status: draft` in the STORY-INDEX days after their respective PRs (#351/#352/#353/#355)
had merged to develop. The correction required a manual reconciliation pass (D-381) that updated four
Index Table rows, stamped PR numbers and merge SHAs, and closed the wave-67/68/69 delivery rows.

This is a recurring gap: every E-21 story needed hand-fixing after merge, and the same pattern was
present in earlier cycles (fix-tls-clienthello-frag, feature-enip-v0.11.0).

### Root Cause

The per-story delivery flow (pr-manager or state-manager post-merge step) has no standing obligation
to flip the delivered story's STORY-INDEX `status` cell and stamp the PR number and merge SHA.
Manual reconciliation is performed ad hoc at cycle close instead.

### Codification

STORY-155 filed to add a post-merge step to the per-story delivery workflow that writes these updates
atomically in the same state-manager commit: Index Table status → `merged`, PR# and merge SHA stamped,
wave delivery row updated (and closed if all stories in that wave are merged). Policy `DF-INDEX-DRIFT-001`
to be added to `.factory/policies.yaml` encoding the post-merge obligation.

---

## [carried] PG-F2-ARCHDELTA-SYNC-001 — Phase-delta working docs drift across adversary passes

**Tag:** [process-gap, carry]
**Surfaced:** F2 adversarial Pass-6 (F-F2P6-003)
**Status:** carry to maintenance sweep / process policy

Phase-delta working docs drift when multiple adversary passes amend them incrementally. Mitigated via
historical-snapshot disclaimer added to arch-delta. Consider codifying a policy that phase-delta docs
either stay synced or carry a snapshot disclaimer. Not codified as a story this cycle; route to
maintenance sweep.

---

## [carried] PG-F2-NARRATIVE-SWEEP-001 — Non-BC artifacts missed by BC-centric remediation sweeps

**Tag:** [process-gap, carry]
**Surfaced:** F2 adversarial (F-F2P7-001/002/003)

PRD §2.18 narrative + ARCH-INDEX subsystem registry are non-BC artifacts that BC-centric remediation
sweeps miss. DF-SIBLING-SWEEP / DF-CONSISTENCY-AUDIT sweeps should explicitly include PRD narrative
blocks + ARCH-INDEX subsystem registry counts as sweep targets.

---

## [carried] PG-F3-HOLDOUT-HASH-DUP-001 — Holdout authoring template leaves duplicate YAML keys

**Tag:** [process-gap, carry]
**Surfaced:** F3 Pass-2 (F-F3P2-003, HIGH)

Holdout authoring template appends computed input-hash without removing the 'tbd' seed → duplicate
YAML keys (9/10 E-21 holdout files). Recommend a lint (bin/compute-input-hash --scan variant or
pre-commit grep) failing on >1 `^input-hash:` key or literal 'tbd' in holdout files.

---

## [carried] PG-F3-SIBLING-UNDERSWEEP-001 — Least-changed sibling skipped by fix bursts

**Tag:** [process-gap, carry]
**Surfaced:** F3 Pass-8 (F-F3P8-001/002)

STORY-152 (least-changed E-21 sibling) was skipped by BOTH the F-F3P2-005 and F-F3P6-005 fix bursts
while STORY-151/153/154 received both. DF-SIBLING-SWEEP-001 fix bursts should explicitly enumerate ALL
same-epic siblings — including the least-changed one — as a checklist item.

---

## [carried] PG-F3-INTEGRATION-TEST-REACHABILITY-001 — Unreachable integration test red-gate

**Tag:** [process-gap, carry]
**Surfaced:** F3 Pass-11 (F-F3P11-001), Pass-14 (F-F3P14-001)

Story-writer decomposition should lint: gap-report/None-target integration tests must key on ports NOT
in classify()'s reserved set. Both F-F3P11-001 (port-502 gap-key) and F-F3P14-001 (port-502
unreachable test) rooted in this class of error.

---

## [carried] PG-F5-RECONCILE-INCOMPLETE-001 — Reconciliation burst missed clause types

**Tag:** [process-gap, carry]
**Surfaced:** F5 Pass-N (D-375/376/377)

First reconciliation burst (D-375) fixed PC-1/Anchor but missed Invariant 2 + phantom variant-shape +
BC-INDEX title-cell; required a second burst (D-376) and a third full-tree sweep (D-377). A
spec-reconciliation must exhaustively grep ALL clause types (PC/Inv/EC/Anchor/BC-INDEX-title/VP
phrasing) in one atomic sweep. Whole-tree grep checklist codified in D-377.

---

## [carried] PG-SPEC-FRESHNESS-ON-FIX-001 — No gate ties BC to new CLI behavior from a wave-level fix

**Tag:** [process-gap, carry]
**Surfaced:** F-W68-01 fix (D-370)

F-W68-01 fix added CSV-rejection + path-routing without a BC amendment; gap caught only by tests.
Consider a spec-freshness check at wave-gate close: if a fix PR adds a new CLI behavior, confirm BC
postconditions cover it before declaring WAVE GATE PASS.

---

## [carried] PG-HELP-PROVENANCE-CLI-DOC-001 — clap doc-comments must not contain factory IDs

**Tag:** [process-gap, carry]
**Surfaced:** STORY-152 tip d34a05f→c4b14f7 (D-368)

clap `///` doc-comments (which become `wirerust --help` text) MUST NOT contain internal factory IDs
(BC-/VP-/SS-/ADR-). Help-provenance CI gate catches this but neither local clippy/fmt nor adversary
axes do. Codify: (a) add "no internal IDs in clap `///` doc-comments" to implementer green-step
checklist + adversary CLI-story axes.
