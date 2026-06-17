---
document_type: session-review
run_id: maint-2026-06-17
pipeline_path: "Path 10 — Maintenance Sweep"
baseline: "develop e1273c8 / v0.7.1"
date: 2026-06-17
verdict: GATE PASS
reviewer_model: claude-sonnet-4-6
---

# Session Review — maint-2026-06-17

## Run Summary

| Dimension | Value |
|-----------|-------|
| Sweeps applicable | 8 (Rust CLI) |
| Sweeps skipped | 3 (DTU, accessibility, design — correctly N/A) |
| Total findings | ~48 |
| CRITICAL | 0 |
| CVE | 0 |
| Fix PRs delivered | 2 (#261, #262) |
| CI status | 9/9 green on both PRs |
| Gate verdict | NON-BLOCKING PASS |
| Deferred to tech-debt | 5 manual items |

---

## 3-Bullet Lessons Summary

**L1 — Sweep signal quality is high; low false-positive rate justifies the sweep cost.** The
doc-drift and pattern-consistency sweeps surfaced 26 genuinely actionable findings (HIGH + MEDIUM)
out of ~48 total. Spec-coherence confirmed structural integrity across all 283 BC / 24 VP / 70
story files with no phantom breakage — a clean all-pass on structural integrity is itself a
valuable negative signal worth logging per run.

**L2 (PROCESS GAP) — Agents operating in worktrees must verify their working-tree path before
editing.** During PR #261 (RED-prose strip), edits to `src/analyzer/arp.rs` landed in the
main repo working tree rather than the assigned worktree. This was caught pre-fast-forward
by a `git stash`; the stash was verified byte-identical to the merged commit before a
`git stash drop` (no work lost). Root cause is path-ambiguity: an agent likely constructed
or resolved an absolute path that pointed at the main checkout instead of
`<worktree-root>/src/analyzer/arp.rs`. This is not unique to this run — any per-story
worktree delivery can hit the same failure mode.

**L3 — The 2-pass review loop on "documentation-only" PRs is justified and should remain
mandatory.** Pass 1 of the docs PR (#262) caught 3 HIGH doc-fidelity defects:
(a) false claim that DNP3 implements `StreamAnalyzer`,
(b) false claim that ARP implements `ProtocolAnalyzer`,
(c) README attributing T0830 to "D3 storm" (wrong threat).
All three were fixed; Pass 2 approved. These are the exact class of errors a human skimming
a docs-only diff would likely miss. The gate pays for itself.

---

## Dimension Analysis

### 1. Cost Analysis
No cost-summary.md available for this maintenance run. No baseline comparison possible.
Self-cost of this session review: estimated small (single read pass over sweep artifacts).
Flag: cost instrumentation should be added to maintenance sweep runs.

### 2. Timing Analysis
No wall-clock timing data captured. Both PRs achieved 9/9 CI green on first submission.
No timeouts observed. The parallel sweep execution (8 sweeps concurrent) is already the
correct pattern — no parallelization debt identified.

### 3. Convergence Analysis
Not applicable to maintenance path. Fix PRs did not require re-review loops;
PR #262 required Pass 2 after findings in Pass 1 (by design, not failure).
2-pass convergence on a docs PR is within normal range.

### 4. Agent Behavior Analysis
One tier/path violation detected: implementer (or delivery) agent edited main repo
working tree during #261 worktree delivery. See PROCESS GAP below.
No other out-of-scope edits observed. Sweep agents stayed on assigned sweep types.
N/A skips (DTU, accessibility, design) were correctly applied without requiring
human intervention — N/A detection logic is working.

### 5. Gate Outcome Analysis
Sweep gate: PASS (0 CRITICAL, 0 CVE). Human triage gate: 2 PRs approved for
delivery, 5 items deferred to tech-debt-register. No overrides of agent recommendations.
Both PRs merged without rollback. Gate thresholds appear correctly calibrated.

### 6. Wall Integrity Analysis
Code-reviewer on #262 referenced only the PR diff and public API surface — no leakage
of implementation internals into the review. Security-reviewer and pr-reviewer operated
independently and both APPROVE. No asymmetry violations detected.

### 7. Quality Signal Analysis
Spec-coherence: 283 BC / 24 VP / 70 story files — structural integrity ALL PASS.
3 label-lag items (stale counts) are pre-existing tracked drift, not new breakage.
No mutation or fuzz data for this maintenance run (maintenance sweeps do not re-run
formal verification by design).

### 8. Pattern Detection (Cross-Run)
| Pattern | Prior run | This run |
|---------|-----------|----------|
| PAT-001 pr-manager-shortstop | Active (6 occurrences) | Not triggered — maintenance sweep does not use pr-manager for story delivery |
| PAT-002 doc-tense recurrence | 7 occurrences (feature-arp) | PR #261 stripped ~70 stale RED-prose comments — this is the *closure* action for PAT-002's backlog, not a new recurrence |
| Worktree path ambiguity | Not previously recorded | NEW — see PROCESS GAP |
| Doc-only PR review catching fidelity errors | Not previously recorded | NEW — confirmed value of 2-pass gate |

---

## Process Gap Analysis

### PG-MAINT-001 — Worktree Path Ambiguity During Fix-PR Delivery

**Observed:** During PR #261 delivery, edits to `src/analyzer/arp.rs` appeared as
uncommitted modifications in the main repo working tree. Cleanup required:
`git stash` (pre-fast-forward), orchestrator verification that stash was
byte-identical to merged commit, then `git stash drop`.

**Root cause hypothesis:** Agent resolved or constructed the absolute edit path
against `git rev-parse --show-toplevel` of the main checkout rather than the
assigned worktree root. Worktree and main checkout share the same `HEAD` branch
name; path-concatenation bugs produce valid-looking absolute paths that point
to the wrong tree.

**Risk if unmitigated:** A future instance where the stash is NOT byte-identical
to the merged commit — meaning the main-tree edit diverged — would require a
manual recovery and could cause data loss if the stash were dropped blindly.

**Recommendation: FOLLOW-UP STORY (not deferral)**

This warrants a concrete checklist item baked into the per-story delivery sequence,
not just a backlog entry that may never surface. Proposed additions:

1. At the start of any file-edit step in a worktree delivery agent:
   assert `git rev-parse --show-toplevel` == `$WORKTREE_ROOT` before the first
   `Edit` or `Write` call. Fail loudly if they differ.
2. In the orchestrator cleanup sequence after worktree fast-forward:
   if `git stash list` is non-empty for the main repo, diff-verify
   (`git diff HEAD stash@{0}`) and assert it is empty before dropping.
   Never call `git stash drop` without this assertion.

**Filing:** Per DF-VALIDATION-001 and CLAUDE.md, this must be validated by the
research-agent before being filed as a GitHub issue. Recommend creating a
research-agent task to confirm the hypothesis (main-tree vs worktree path
resolution) against the actual git worktree documentation, then file.

---

## Improvement Proposals

### PROP-M01 — Worktree Path Guard in Per-Story Delivery
- **Category:** agent + workflow
- **Priority:** P1 HIGH
- **Evidence:** PG-MAINT-001 (maint-2026-06-17); main-tree edit during #261 worktree delivery
- **Recommendation:** Add `git rev-parse --show-toplevel` assertion as the first step
  of any file-edit sequence in per-story/fix-pr worktree delivery agents.
  Add diff-verify-before-drop guard to orchestrator worktree cleanup sequence.
- **Affected files:** orchestrator per-story-delivery sequence, fix-pr delivery template
- **Risk:** Low — assertion adds a read-only git call; no behavior change on correct paths
- **Disposition required:** STORY (needs research-agent validation first per DF-VALIDATION-001)

### PROP-M02 — Mandatory 2-Pass Review on Documentation PRs
- **Category:** gate
- **Priority:** P2 HIGH
- **Evidence:** PR #262 Pass 1 caught 3 HIGH fidelity defects (false trait claims, wrong
  threat attribution) that a single-pass skim would likely have missed
- **Recommendation:** Formalize that doc-only PRs are NOT exempt from the 2-pass
  review loop. Current policy may implicitly allow single-pass on "trivial" doc PRs.
  Add explicit note to pr-review gate criteria.
- **Affected files:** pr-review gate template / orchestrator-maintenance-sequence
- **Risk:** Minimal — adds one review pass only when Pass 1 returns findings

### PROP-M03 — Cost Instrumentation for Maintenance Runs
- **Category:** cost
- **Priority:** P3 MEDIUM
- **Evidence:** cost-summary.md absent for maint-2026-06-17; cannot compute cost-per-finding
  or track sweep ROI across runs
- **Recommendation:** Emit a cost-summary.md to `.factory/maintenance/cost-summary-<run-id>.md`
  at maintenance sweep completion, with per-sweep token estimates.
- **Affected files:** orchestrator-maintenance-sequence, state-manager
- **Risk:** None — additive instrumentation only

### PROP-M04 — Wall-Clock Timing Log for Maintenance Runs
- **Category:** timing
- **Priority:** P4 LOW
- **Evidence:** No timing data in maint-2026-06-17 artifacts; cannot identify sweep
  bottlenecks or track sweep duration trends
- **Recommendation:** Emit start/end timestamps per sweep to sweep-report.md frontmatter.
- **Affected files:** sweep-report template, maintenance sweep sequence
- **Risk:** None

---

## Deferred Items Status

5 items correctly deferred to tech-debt-register per human decision:
- B1 PC-001 DNP3 StreamHandler trait conformance (2-4 days, design)
- B2 PC-006 modbus analyzer_name casing (breaking output change)
- B3 PC-003 DNP3 dropped_findings counter
- B4 Performance regression investigation (re-run reassembly/tls first)
- B6 Risk/assumption registry backfill (structural, before next ICS protocol feature)

Dep bumps (A1) and label-lag fixes (A5) left for a future sweep per human decision —
not urgent, no correctness impact.

---

## Factory-Artifacts HEAD SHA

At time of this review, factory-artifacts HEAD: `06ca45f`
(commit: "chore(maintenance): finalize maint-2026-06-17 — 2 PRs merged (#261/#262), 5 deferred, gate PASS")

---

## Appendix: Pattern Database Updates Required

Add to `pattern-database.yaml` after this review:

- **PAT-007** `worktree-path-ambiguity` — first observed maint-2026-06-17; severity HIGH;
  proposal PROP-M01; status OPEN.
- **PAT-008** `doc-pr-fidelity-defects-caught-by-2pass` — first observed maint-2026-06-17;
  severity MEDIUM (positive signal); proposal PROP-M02; status DOCUMENTED-POSITIVE.
