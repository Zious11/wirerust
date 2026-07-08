# Lessons Learned — wave-71

S-7.02 cycle-closing requirement: lessons recorded here for human review and
engine-improvement triage.

Wave: 71 | Closed: 2026-07-08 (D-404) | Stories: STORY-150, STORY-156, STORY-157 (13 pts).
PRs merged: #378 (e2c2b33) / #379 (9d0d175) / #380 (11c37b6) / #381 gate-fix (b642c0f).
Adversarial convergence: 7 passes; streak 3/3 (P5/P6/P7); trajectory 1→0→0→1→0→0→0.

---

## Lesson 1 — [codified] Wave-70 Process Gaps Delivered via STORY-157

**Observation:**

Wave-70's three process gaps (PG-S149-001 adversary checkout-guard omission,
PG-W70-DEMO-SCRUB absolute-path policy, PG-HASH-EMPTY-INPUTS empty-inputs
documentation) were drafted as STORY-157 at wave-70 gate close and fully delivered
during wave-71. STORY-157 merged as PR #380 (11c37b6, 2026-07-08).

The adversary checkout-guard (GUARD-002, codified in STORY-157) was live on all
wave-71 dispatches. No stale-state adversary recurrence occurred — confirming the
PG-S149-001 fix is effective. The canonical hash scan achieved MATCH=110 STALE=0
for the first time, validating the empty-inputs and inline-comment handling fixes.

**Lesson:**

The wave-close S-7.02 → draft-story loop is working. Process gaps codified in
wave N are delivered in wave N+1 and are operationally verified at the N+1 gate.
No new meta-lesson needed; reinforce the existing pattern.

---

## Lesson 2 — [codified] Wave-71 Process Gaps Drafted as STORY-158

**Observation:**

Wave-71 adversarial passes surfaced three new process gaps:
- PG-W71-CHANGELOG (P1 catch): unreleased changelog entries not updated before
  gate close — caught by adversary, required PR #381 gate-fix.
- PG-W71-CYCLE-ARTIFACT-IDENTITY (P4 catch): implementation evidence.md files in
  cycles/ lack story-identity headers — required a factory-artifacts remediation commit.
- PG-W71-CI-SCAN-GUARDS (P3 LOW): no CI guards for `cargo test --doc` and clippy on
  bin/ targets — LOW severity, no gate-fix required but identified as a hardening gap.

These three gaps are codified in STORY-158 (E-11, 3 pts, wave-TBD, STORY-INDEX v3.23).
STORY-158 is the S-7.02 deliverable for wave-71, mirroring the STORY-157 role in wave-70.

**Lesson:**

Consistent with Lesson 1: the S-7.02 loop produces one S-NNN story per wave capturing
that wave's process-gap class. Deferred-LOW PG findings that are not gate-blocking are
intentionally carried into STORY-158 rather than requiring a same-wave fix — this avoids
prolonging the gate on LOW findings once streak is achievable.

---

## Lesson 3 — Partial-Fix Propagation Recurred; Sibling-Sweep Grep Discipline Required

**Observation:**

P4 adversary catch (F-W71-P4-001 MEDIUM) found that the wave-71 implementation
evidence.md files lacked story-identity headers. The initial remediation fixed one
file; the subsequent PROC-OBS-002 review found that a single-file fix would have
missed the STORY-156 and STORY-157 sibling directories. The fix required an explicit
grep-across-all-wave-71-subdirs sweep.

This recurrence pattern — partial-fix propagation requiring a follow-up sweep to
catch siblings — has appeared in multiple prior cycles (wave-68 F-W68 bc-anchor
propagation, feature-protocol-coverage Pass-5 remediation undersweep). Each time,
the root cause is the same: the remediation dispatch described only the triggered
file, not the full sibling set.

**Lesson:**

Every remediation dispatch for any finding that touches a per-story artifact MUST
include an explicit grep-list enumerating ALL same-wave (or same-epic) sibling
directories. Do not rely on "repair the reported file and check neighbors manually."
The authoritative procedure is DF-SIBLING-SWEEP-001 v4; the grep-list is the
enforcement mechanism. STORY-158 carries PG-W71-CYCLE-ARTIFACT-IDENTITY to
codify this for the evidence.md identity case specifically.

---

## Lesson 4 — Audit-Artifact Identity Lens Is a Durable Adversarial Axis

**Observation:**

Pass-4 found a MEDIUM finding (F-W71-P4-001) by applying what can be called the
"audit-artifact identity lens": examining whether cycle-scoped factory artifacts
(implementation/evidence.md, FINDINGS.md, session records) carry enough internal
identification to be unambiguous when read out-of-context. Files lacked story-ID
headers, making them indistinguishable from each other in isolation.

This adversarial axis — "can this artifact be correctly attributed if read cold?" —
is distinct from the standard behavioral-contract and spec-completeness axes.
It is effective because factory artifacts are written by the delivery pipeline under
time pressure and are rarely reviewed for identity metadata.

**Lesson:**

Add "audit-artifact identity" as a named axis in the wave adversary charter: for any
cycle-scoped file produced during delivery (evidence.md, red-gate-log.md, FINDINGS.md),
verify that the first non-YAML block contains an explicit story reference
(e.g., `# STORY-NNN: …`). This is a lightweight structural check, not a content review.
Candidate for codification in the adversary dispatch template at the next charter update.

---

## Lesson 5 — VHS Wait+Screen Limitation with Box-Drawing Output; Transcript Fallback

**Observation:**

All four wave-71 VHS demo recordings failed with `Wait+Screen` timeout. Root cause:
wirerust's unicode box-drawing separator characters (U+2500 × 40, `────────────────────`)
fill VHS 0.11.0's visible viewport before the matched pattern scrolls into view.
`Wait+Screen` checks only the visible viewport (not scrollback buffer), so the match
never fires.

The wave-70 demo-evidence conventions (PG-W70-DEMO-SCRUB, STORY-157) document the
scrub policy but do not address the VHS Wait+Screen limitation. This gap was observed
and documented in the wave-evidence-report.md.

**Mitigation applied:** Transcript fallback (.txt files) used as primary evidence;
tape files retained as runnable reference scripts with `<repo>` placeholders and
inline comments explaining the limitation.

**Lesson:**

For future wirerust VHS recordings of full-output commands, pipe through `grep` or
use `--json` with short-field extraction (`jq .field`) to keep visible output under
~20 lines per command, which keeps the match-target within the VHS viewport.
Alternatively, use `Screen` with a pattern that appears early in output rather than
waiting for the summary line. Document this in the tape templates for wave-72+.
