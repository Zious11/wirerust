---
document_type: lessons-learned
level: ops
version: "1.0"
status: final
producer: state-manager
timestamp: 2026-06-24T00:00:00Z
cycle: "fix-pc-013-014-015"
inputs: [STATE.md]
traces_to: STATE.md
---

# Lessons Learned — fix-pc-013-014-015

Durable lessons from the fix-pc-013-014-015 cycle (PC-013: ARP HashMap .expect() invariants;
PC-014: dnp3 key rename total_parse_errors → parse_errors; PC-015: ARP findings unbounded).
Cycle delivered 2026-06-23..2026-06-24.

## Process-Level

1. **Backlog labels must be verified against current code before committing to a fix approach.**
   All three STATE.md open-item descriptions were factually inaccurate when checked against the
   actual code: PC-013 "panic-on-malformed risk" was actually provably-unreachable internal
   invariants (not packet input panics); PC-014 "key missing" was actually key MISNAMED
   (`total_parse_errors` existed, just named wrong); PC-015 "cap not documented" was actually
   NO cap exists at all (intentional unbounded design). The F1 scoping pass + research agent
   caught all three inaccuracies before any wrong fix shipped. Practice: the scoping/F1 pass
   MUST read the actual code before endorsing or modifying a backlog label.
   _Discovered: F1 scoping pass, 2026-06-23. Category: process-practice._

## Design-Level

2. **Fail-OPEN is the wrong fix for internal invariant `.expect()` sites — keep the loud tripwire.**
   PC-013 was originally framed as "change `.expect()` → silent `if let` skip (fail-safe
   degradation)." Research (codebase survey of arp.rs state machine + external best-practice
   survey) showed this is a fail-OPEN anti-pattern: silently skipping a HashMap miss on an
   internal invariant hides bugs and corrupts results without any signal. The correct fix was
   to retain the loud `.expect()` (which will panic-crash in tests, not silently drop findings)
   and add white-box regression-guard tests that assert the invariants hold for all reachable
   inputs. The label "fail-safe degradation: silently skip" is a misnomer for internal
   invariants — it applies only to truly external inputs (network data, user input), not
   post-construction state-machine guarantees.
   _Discovered: PC-013 research + BC-2.16.004 v1.10 spec correction, 2026-06-24. Category: design-lesson._

## Non-Blocking Backlog Items (DF-VALIDATION-001 — do NOT file as GitHub issues without research-agent validation)

The following items were surfaced during this cycle and are tracked here as non-blocking
backlog items only. They are NOT filed as GitHub issues — DF-VALIDATION-001 requires
research-agent validation first.

- **(a) Demo .tape scripts hardcode ephemeral worktree `cd` path.** The VHS `.tape` scripts
  recorded during this cycle (AC-010/AC-011 demo evidence in STORY-108) reference a specific
  worktree path that is ephemeral. Should reference a stable, repo-relative path or
  parameterize the `cd` command. Low priority; no correctness impact.

- **(b) Demo-evidence binary media lacks a checksum manifest.** Binary `.gif`/`.mp4` demo
  evidence committed to factory-artifacts has no accompanying SHA-256 manifest. Adds an
  integrity gap if demo files are later modified without being re-recorded. Low priority.

- **(c) Dependabot PR #311 (actions/checkout bump) open and unreviewed.** Opened during this
  cycle; not merged. Requires human triage and approval.

## Engine/Process-Gap Assessment (S-7.02 checklist)

No lesson in this cycle rises to the level of a true engine/process-gap requiring a
follow-up self-improvement story. Both lessons (L1 and L2) are practice notes that
reinforce existing VSDD principles (verify before fix; prefer loud failure for internal
invariants). The existing policies (DF-VALIDATION-001, the scoping/F1 phase gate) already
encode the correct behavior — the gap was in the backlog labels, not in the engine design.

**No follow-up self-improvement story required.** Lessons recorded here as practice
lessons only. Satisfies S-7.02 checklist requirement.

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| L1 (backlog label accuracy) | Policy candidate: BACKLOG-LABEL-VERIFY-001 — require code read before endorsing or closing any backlog label during F1/scoping | Fix cycles and maintenance scoping | proposed (not yet adopted — human review required) |
| L2 (internal invariant fail-open) | Practice note only — existing fail-safe degradation guidance already distinguishes external vs internal. No new policy needed. | n/a | rejected as new policy |
