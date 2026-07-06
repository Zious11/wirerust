# Lessons Learned — maint-2026-07-06

S-7.02 cycle-closing requirement: lessons recorded here for human review and
engine-improvement triage.

Run: maint-2026-07-06 | Completed: 2026-07-06 | 8 sweeps, 39 findings, 0 CRITICAL.
Fix routes A–D + 1 out-of-band dep fix. PRs merged: #369 (docs), #370 (dnp3 counters), #371 (crossbeam-epoch RUSTSEC-2026-0204).

No `[process-gap]` tagged adversary findings this run — maintenance mode carries no adversarial
passes, so S-7.02 checklist item for codification follow-ups is satisfied with no new stories
required beyond existing STORY-155/156.

---

## Lesson 1 — Advisory-Race: RUSTSEC advisory published between sweep and PR CI

**Observation:**

The morning dependency sweep (Sweep 1, ~19:09) ran `cargo audit` against 1157 advisory DB
entries and returned CLEAN. Approximately 3 hours later, PR #371 CI ran and failed `cargo audit`
— the advisory DB had grown to 1158 entries. RUSTSEC-2026-0204 (crossbeam-epoch 0.9.18) had
been published in the interval, affecting a dev-only dependency (zero runtime footprint).

The fix was same-day: bump crossbeam-epoch 0.9.18→0.9.20 via `cargo update`, PR #371 merged.

**Root cause:**

`cargo audit` fetches the live advisory DB at run time. A CLEAN scan at time T does not
guarantee a CLEAN scan at time T+Δ, even on an unchanged codebase.

**Lesson:**

When `cargo audit` fails on a PR whose diff does not touch Cargo.toml or Cargo.lock
(or touches only unrelated crates), check the advisory DB delta first before blaming
the diff. The pattern is: new advisory published between sweep and CI run, not a
regression introduced by the PR. Fix is `cargo update <affected-crate>` rather than
reverting the PR.

Operationally: treat Audit failures on fresh PRs as possible time-bombs, not
diff-caused regressions.

---

## Lesson 2 — API Mid-Stream Stalls in Heavy Read+Write Single-Agent Tasks

**Observation:**

Sweep 8 (tech-debt register update) stalled twice mid-run, producing zero writes in both
attempts. The third attempt — prompted with explicit incremental-write and short-reply
instructions — succeeded on the first try.

**Root cause:**

Heavy read-then-write single-agent tasks (large file read + multi-section rewrite) appear
prone to mid-stream API stalls that silently truncate the response before any writes
occur. The agent produces no output and returns empty-handed.

**Lesson:**

For single-agent tasks that combine large file reads with multi-section writes:
- Instruct the agent to write incrementally (one section at a time) rather than
  accumulating all changes for a single large write.
- Keep the reply short; do not ask for explanatory prose alongside the writes.
- On second stall, diagnose first: check whether partial state was written before
  retrying (partial writes can corrupt the file).

Codification note: first occurrence of this class; STORY-155 (index-drift automation)
covers related tooling. No new story required for the agent-instruction pattern itself.

---

## Lesson 3 — Register/Canonical-ID Drift Caught by DF-VALIDATION-001

**Observation:**

Sweep 8 submitted tech-debt register entries using non-canonical PC IDs (e.g., PC-019,
PC-020 as new when PC-020 already existed with a different canonical ID assignment).
The content was correct; only the IDs drifted from the canonical numbering established
earlier in the same run.

DF-VALIDATION-001 validation (pass before filing) caught this before any GitHub issues
were filed. The entries were corrected and re-registered under their canonical IDs
(PC-016, PC-017, etc.) as established in the pattern-consistency sweep report.

**Root cause:**

The agent producing the register entries did not re-read the sweep report's canonical
ID assignments before writing the register row. ID drift is a content defect, not a
process-gap (the validation gate worked correctly).

**Lesson:**

When adding register entries from sweep reports, always re-read the sweep report's
canonical ID assignments immediately before writing the register row. Do not rely on
memory of IDs established earlier in the run; the canonical source is the sweep report
artifact, not the agent's working memory.

This is the first occurrence of this class. STORY-155 (automated BC-index drift
detection) covers related index-consistency automation and is the closest codification
candidate. No additional story required for this specific ID-drift pattern at this time.
