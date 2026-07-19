---
document_type: story
id: STORY-143
title: "Harden Release Changelog Step: Full Prev-Tag..HEAD Range Enumeration"
epic: E-11
wave: "~"
points: 3
status: superseded
version: "1.1"
depends_on: []
input-hash: d41d8cd
inputs: []
---

# STORY-143 — Harden Release Changelog Step: Full Prev-Tag..HEAD Range Enumeration

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** superseded
**Wave:** TBD
**Points:** 3

## Background

In v0.11.0 the release CHANGELOG entry and GitHub release notes were initially
authored from a hand-summarized "what shipped" list scoped to the most recent
wave (EC-X1/EC-X2 fixes). The entire ENIP analyzer epic (STORY-130..138,
PRs #317–#334) was omitted from the initial entry because it had merged earlier
in the same release window and was invisible to a recollection-based approach.
Two post-release correction PRs (#339, #340) were required to author the complete
[0.11.0] entry.

Root cause and full narrative: cycles/feature-enip-v0.11.0/lessons.md §
RELEASE-CHANGELOG-FULL-RANGE-001.

## Goal

Encode lesson RELEASE-CHANGELOG-FULL-RANGE-001 into the release workflow so
that changelog omission at this class is mechanically prevented for v0.12.0 and
beyond. Specifically:

1. The devops-engineer release-prep step MUST run `git log <prev-tag>..HEAD
   --first-parent --oneline` to enumerate all merged PRs before authoring the
   CHANGELOG entry.
2. The CHANGELOG entry MUST cite the PR range (e.g., "PRs #317–#338") as a
   completeness anchor.
3. Either (a) add a policy DF-RELEASE-CHANGELOG-RANGE-001 to policies.yaml
   encoding this obligation, or (b) update the release-workflow documentation
   (CLAUDE.md or a release-runbook artifact) with these steps — whichever fits
   the project's enforcement model.
4. The release CHANGELOG entry is cross-checked against the enumerated log
   output before the release PR opens.

## Acceptance Criteria

AC-143-001: A policy DF-RELEASE-CHANGELOG-RANGE-001 exists in `.factory/policies.yaml`
  OR the CLAUDE.md "Releasing to main" section contains a "Changelog enumeration"
  sub-step that mandates `git log <prev-tag>..HEAD --first-parent --oneline` before
  authoring the CHANGELOG entry.

AC-143-002: The release workflow documentation (wherever the policy lives) explicitly
  prohibits hand-summarized or recollection-based CHANGELOG authoring as the sole
  source; requires the commit-range enumeration as the authoritative input.

AC-143-003: The documentation or policy cites the PR range as a mandatory completeness
  anchor field in the CHANGELOG entry (format: "Includes PRs #NNN–#MMM" or equivalent).

AC-143-004: The story includes a self-audit confirming that the correction PRs #339 and
  #340 are the last PRs requiring the "docs-only post-release correction" pattern for
  the changelog-omission class.

## Notes

- This is a documentation/policy story, not a code change. No Rust source changes.
- Wave assignment is TBD — schedule at v0.12.0 planning along with STORY-091 and
  STORY-121.
- Source lesson: cycles/feature-enip-v0.11.0/lessons.md §
  RELEASE-CHANGELOG-FULL-RANGE-001 (D-301, 2026-06-29).
- Related policy being proposed: DF-RELEASE-CHANGELOG-RANGE-001.

## Disposition

**Status:** superseded — routed upstream 2026-07-19.

The durable fix for this story is a **release-skill / devops-engineer behavior**:
release-prep MUST run `git log <prev-tag>..HEAD --first-parent --oneline` to enumerate
all merged PRs before authoring the CHANGELOG, citing the PR range as a completeness
anchor. `vsdd-factory:release` and `devops-engineer` are engine agents; this applies to
every VSDD-managed project, not just wirerust.

| AC | Upstream Disposition |
|----|----------------------|
| AC-143-001 (policy or CLAUDE.md changelog-enumeration sub-step) | drbothen/vsdd-factory#695 (NEW issue filed 2026-07-19, x-ref #580) |
| AC-143-002 (prohibit recollection-based authoring) | drbothen/vsdd-factory#695 |
| AC-143-003 (PR range as mandatory completeness anchor) | drbothen/vsdd-factory#695 |
| AC-143-004 (self-audit re: #339/#340 correction PRs) | drbothen/vsdd-factory#695 |

**Distinct from #580:** #580 is a per-story CHANGELOG delivery task (authoring-time-per-story);
this story's mechanism is release-time full-range reconciliation. Different mechanism,
cross-referenced in the filed issue.

This story file is retained on disk for traceability. No further wirerust delivery
expected.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-07-19 | story-writer | Status draft→superseded — engine-level release-changelog enumeration behavior routed upstream via new issue drbothen/vsdd-factory#695, x-ref #580. |
| 1.0 | 2026-07-08 | state-manager | Added `document_type: story` and `input-hash: d41d8cd` for scanner compatibility (STORY-157 TASK F; `inputs: []` → canonical empty-inputs hash). |
