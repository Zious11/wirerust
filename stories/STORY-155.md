---
document_type: story
id: STORY-155
title: "Auto-update STORY-INDEX status draft→merged on story PR merge"
epic: E-11
wave: "~"
points: 3
status: draft
depends_on: []
input-hash: d41d8cd
inputs: []
---

# STORY-155 — Auto-update STORY-INDEX status draft→merged on story PR merge

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** TBD
**Points:** 3

## Background

When a story PR squash-merges to develop, the per-story-delivery flow does not
update that story's STORY-INDEX row from `status: draft` to `merged`. The index
drifts out of sync with reality.

The E-21 F7 consistency audit (D-381, 2026-07-05) caught this as finding P0-001:
STORY-151, STORY-152, STORY-153, and STORY-154 all showed `status: draft` in the
STORY-INDEX days after their respective PRs (#351/#352/#353/#355) had merged to
develop. The correction required a manual reconciliation pass (D-380/D-381) that
updated four Index Table rows, stamped PR numbers and merge SHAs, and closed the
wave-67/68/69 delivery rows.

This is a recurring gap: every E-21 story needed hand-fixing after merge, and the
same pattern was present in earlier cycles. S-7.02 (cycle-close requirement) mandates
that recurring process gaps be codified as follow-up stories rather than hand-fixed
each cycle.

Root cause: the per-story delivery flow (pr-manager or state-manager post-merge step)
has no standing obligation to flip the delivered story's STORY-INDEX `status` cell
and stamp the PR number and merge SHA. Manual reconciliation is performed ad hoc at
cycle close instead.

## Goal

Automate the STORY-INDEX status update that currently requires manual reconciliation
at cycle close. Specifically:

1. **Add a post-merge step** to the per-story delivery workflow (pr-manager
   post-merge or state-manager cycle-state-update) that, after a story PR merges to
   develop, writes the following updates to STORY-INDEX.md in the same state-manager
   commit:
   - Index Table: status cell for the delivered story changes to `merged`.
   - Wave Delivery Progress: the wave row is updated with PR# and merge SHA; if
     all stories in that wave are now merged, the row is marked DELIVERED & CLOSED.

2. **The update is idempotent**: running the step twice on the same story produces no
   net diff (already-`merged` rows are left unchanged).

3. Either (a) add a policy `DF-INDEX-DRIFT-001` to `.factory/policies.yaml` encoding
   this post-merge obligation, or (b) update the per-story delivery workflow
   documentation (CLAUDE.md or a delivery-runbook artifact) with the step — whichever
   fits the project's enforcement model.

## Acceptance Criteria

AC-155-001: After any story PR merges to develop, the STORY-INDEX Index Table row for
  that story reflects `status=merged` with the merge PR# and SHA recorded (in the
  wave-delivery table or the index row) — with no additional manual reconciliation
  pass required.

AC-155-002: A fresh consistency audit (equivalent to the E-21 F7 P0-001 check)
  finds zero rows where `status=draft` but the corresponding PR has already merged to
  develop.

AC-155-003: The post-merge STORY-INDEX update is idempotent: applying it to an
  already-`merged` row produces no net diff to the file.

## Notes

- This is a process/tooling story. The deliverable is a workflow change (post-merge
  hook or state-manager step) and/or a policy/documentation addition, not a Rust
  source change.
- Wave assignment is TBD — schedule alongside STORY-091, STORY-121, STORY-143,
  STORY-147 (all E-11, wave-TBD tooling stories) at the next planning pass.
- Source finding: E-21 F7 consistency audit finding P0-001 (D-381, 2026-07-05).
  Manual fix applied in D-380/D-381 (STORY-151/152/153/154 status reconciliation,
  STORY-INDEX v3.13).
- Tag: [process-gap] follow-up per S-7.02.
- Precedent: STORY-143 (release-changelog enumeration hardening, D-301, 2026-06-29)
  and STORY-147 (mutation-testing parallelism hardening, PG-MUTANTS-JOBS-001,
  2026-07-01) — same E-11 pattern: a cycle process-gap follow-up encoding a lesson
  into project workflow/docs.
- S-7.02 disposition: creating this story at draft status codifies the
  PG-INDEX-DRIFT-001 process gap for S-7.02 E-21 cycle-close purposes.
- No behavioral contract required: E-11 convention (see epics.md E-11 note:
  "BCs: none authored yet — status: draft; pending PO authorship").

## Changelog

- 2026-07-08 (state-manager): Added `document_type: story` and `input-hash: d41d8cd` for scanner compatibility (STORY-157 TASK F; `inputs: []` → canonical empty-inputs hash).
