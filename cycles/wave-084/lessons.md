---
document_type: lessons-learned
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-07-21T05:30:00Z
cycle: "wave-084"
inputs: [STATE.md, process-gap-ledger.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Lessons Learned — wave-084

S-7.02 cycle-closing requirement. All entries post-date D-448(b); no pre-D-448(b) exemptions apply.

Wave: 84 | Gate CLOSED: 2026-07-21 (D-486) | Stories: STORY-147 (2 pts) + STORY-166 (3 pts) + STORY-176 (2 pts) = 7 pts.
PRs merged: #421 (f0cb7374 STORY-147) + #426 (fa9be701 STORY-166) + #427 (595cdba8 STORY-176)
+ #428 (82105d02 gate-fix 1) + #429 (39b30cb1 gate-fix 2) + #430 (1e967bad gate-fix 3).
Wave-level adversarial: 6 passes; streak 3/3 (P4/P5/P6); trajectory 1M → M/L-batch → 1L → 0 → 0 → 0.
Process-gaps: 12 entries (PG-W84-001..012); 3 FIXED in-cycle; 9 deferred to DF-VALIDATION-001 batch.

---

## Agent-Level

### L-W84-002 — [codified] Story-writer ACs for script-gated behavior must cite the actual script source

**Observation:** STORY-176 v2.2 AC-176-001 described a `# green-doc-tense-gate: allow`
inline comment allowlist that does not exist in `bin/check-green-doc-tense`; cited `ci.yml`
as the gate locus (wrong — gate lives in the Python script); and inverted the CHANGELOG
obligation. Research-validated HIGH confidence INVALID (D-484;
`.factory/planning/story-176-ac001-validation.md`). The stub-architect's Step-2 pre-condition
probe caught the discrepancy before Red Gate; remediated via spec-route v2.2→v2.3.
_Discovered: STORY-176 Step 2, 2026-07-20_

**Closes:** PG-W84-009

---

### L-W84-005 — [deferred] Sub-agent message-routing breakage causes missing artifacts and backfill commits

**Observation:** During STORY-147 delivery (D-481), a sub-agent result did not reach the
orchestrator, requiring relay-through-orchestrator workaround and a separate backfill commit
(f2b5dcfe) for the security-review.md artifact. Sub-agents must relay all results through
the orchestrator; direct cross-agent messaging is unreliable.
_Discovered: STORY-147 delivery, 2026-07-20_

**Closes:** PG-W84-002

---

### L-W84-006 — [deferred] validate-pr-review-posted hook false-positive for self-authored PRs

**Observation:** The hook blocked valid STORY-166 delivery (D-482) because it does not
distinguish "no review posted" from "self-authored PR with COMMENTED review event + artifact
= review of record." Required orchestrator escalation. DF-VALIDATION-001 required before
upstream filing.
_Discovered: STORY-166 delivery (PR #426), 2026-07-20_

**Closes:** PG-W84-005

---

### L-W84-007 — [deferred] pr-manager-completion-guard pressures step-9 fabrication before merge is confirmed

**Observation:** The pr-manager-completion-guard applied merge-completion pressure before
the PR was merged, inappropriately pressuring the agent to fabricate step-9 (HIGH severity;
agent correctly refused). Guard must not fire until `gh pr view --json mergeStateStatus`
confirms merged state.
_Discovered: STORY-166 delivery, 2026-07-20_

**Closes:** PG-W84-006

---

## Process-Level

### L-W84-001 — [codified] Governance-doc CI examples must be execution-verified against branch topology

**Observation:** `demo-evidence-scrub-gate.md` used a `grep` example that exits 2 (not 1)
when `.factory/` is absent on develop CI (factory-artifacts worktree not mounted), producing
a false-green even when leaks were present. Caught by STORY-166 adversarial pass 7
(F-S166P7-001 HIGH). Fixed by adding `|| true` guard and explicit exit-code documentation.
Every CI-guard example in a governance doc must be execution-verified against the actual
branch topology where it will run (develop CI does not mount `.factory/` unless explicitly
fetched).
_Discovered: STORY-166 Step-4.5 pass 7, 2026-07-20_

**Closes:** PG-W84-007

---

### L-W84-003 — [codified] New bin/test_*.py must be wired into bin-selftest CI job at delivery time

**Observation:** `bin/test_gitignore_mutants_glob.py` was delivered in STORY-176 Steps 1–4
without being wired into the existing `bin-selftest` CI job (established by STORY-165
AC-165-001). Adversary caught this at pass 4 (F-S176P4-001). Fixed in-cycle at commit
`ea4bcd8e`. Direct recurrence of PG-W74-CI-BIN-SELFTEST. Per-story delivery checklist
item needed: for every new `bin/test_*.py` added — extend the bin-selftest CI job step
before Step-4.5 dispatch.
_Discovered: STORY-176 Step-4.5 pass 4, 2026-07-20_

**Closes:** PG-W84-011

---

### L-W84-004 — [deferred] Stale inline version markers recur; automated lint check candidate

**Observation:** Story-writer and implementer agents left stale `## Story v<N.M>` markers
after spec evolution across STORY-147 and STORY-166 (3+ occurrences in wave-84 alone;
pattern recurs from prior waves). Candidate fix: lint check comparing `## Story v<N.M>`
header vs frontmatter `version:` field. Upstream engine candidate.
_Discovered: PR reviews STORY-147 and STORY-166, 2026-07-20_

**Closes:** PG-W84-001

---

### L-W84-008 — [deferred] bin/check-green-doc-tense Rust-only scan blind spot for bin/*.py prose

**Observation:** The gate scans only `*.rs` files; adversary caught stale RED-phase prose
in `bin/test_check_green_doc_tense.py` (F-S176P1-003) that the gate itself missed.
Candidate: extend scan glob to cover `bin/*.py`. Product-local; DF-VALIDATION-001 required
before filing as GitHub issue.
_Discovered: STORY-176 Step-4.5 pass 1, 2026-07-20_

**Closes:** PG-W84-010

---

### L-W84-012 — [deferred] PR description commit-count composed before final fixup commit

**Observation:** PR #426 (STORY-166) description claimed 10 commits but the squash base
contained 11. The pr-manager composed the count before the final fixup commit was added.
Re-count immediately before posting, or note counts as approximate pre-squash estimates.
_Discovered: Post-merge review PR #426, 2026-07-20_

**Closes:** PG-W84-008

---

## Infrastructure-Level

### L-W84-009 — [deferred] bin-selftest CI job not in develop required-status-checks

**Observation:** `bin-selftest` CI job (established STORY-165; extended STORY-176) is not
a required status check in develop branch protection. Self-test guards do not block merges
if the job fails. Pre-existing pattern (STORY-164/165 also landed without wiring). Adversary
surfaced as Obs-P7-2 at STORY-176 pass 7. Product-local; DF-VALIDATION-001 required before
filing as GitHub issue.
_Discovered: STORY-176 Step-4.5 pass 7, 2026-07-20_

**Closes:** PG-W84-012

---

### L-W84-010 — [deferred] Burst-log Dim-1 file-count template understates ride-along files

**Observation:** Burst-log template Dim-1 "files touched" section does not guide agents to
count ride-along files (session-checkpoints.md, process-gap-ledger.md), causing cardinality
mismatches caught by the validate-burst-log hook during STORY-147 delivery.
_Discovered: STORY-147 delivery, 2026-07-20_

**Closes:** PG-W84-003

---

### L-W84-011 — [deferred] STATE.md write-path hook cascade lacks unified error reporting

**Observation:** Three simultaneous PostToolUse hooks on STATE.md writes
(verify-state-timestamp-refresh, validate-dispatch-advance, validate-state-pin-freshness)
can pass two and fail the third. Error message identifies only the blocking hook, not the
full cascade status, requiring trial-and-error diagnosis.
_Discovered: D-484 burst, 2026-07-21_

**Closes:** PG-W84-004

---

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| L-W84-001 | Governance-doc CI examples must be execution-verified against branch topology before publication | CLAUDE.md / docs/ governance authoring checklist | proposed (product-local; upstream variant needs DF-VALIDATION-001) |
| L-W84-002 | Story-writer must read actual script source before authoring ACs for script-gated behavior; fabricated mechanism references are HIGH-severity spec defects | story-writer agent delivery checklist | proposed (upstream engine candidate; DF-VALIDATION-001 required) |
| L-W84-003 | Per-story delivery checklist: for every new `bin/test_*.py` added — extend bin-selftest CI job step before Step-4.5 dispatch | per-story-delivery orchestrator workflow | proposed (upstream engine candidate; DF-VALIDATION-001 required) |
| L-W84-005 | Sub-agents must relay all results through the orchestrator; direct cross-agent messaging is unreliable | agent message-routing protocol | proposed (upstream engine candidate; DF-VALIDATION-001 required) |
| L-W84-006 | validate-pr-review-posted hook must distinguish self-authored PRs (COMMENTED + artifact = review of record) from no-review | hook logic | proposed (upstream engine candidate; DF-VALIDATION-001 required) |
| L-W84-007 | pr-manager-completion-guard must not fire step-9 pressure until `gh pr view --json mergeStateStatus` confirms merged | hook timing | proposed (upstream engine candidate; DF-VALIDATION-001 required) |
