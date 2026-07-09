# Lessons Learned — wave-72

S-7.02 cycle-closing requirement: lessons recorded here for human review and
engine-improvement triage.

Wave: 72 | Gate passed: 2026-07-09 (D-415) | Wave CLOSED: 2026-07-09 (D-416, human-approved) | Stories: STORY-158, STORY-159, STORY-160, STORY-161 (29 pts).
PRs merged: #387 (75c5ba5) / #388 (d410b8d) / #389 (704fd2e) / #390 (80fbb64) / #391 gate-fix (44f8c9c).
Adversarial convergence: 4 passes; streak 3/3 (P2/P3/P4); trajectory 1→0→0→0.

---

## Lesson 1 — [codified] via PR #391 CI hardening — Wave-Level Sibling-Sweep Catches What Per-Story Scope Cannot

**Observation:**

Wave-72 adversary Pass 1 (F-W72G-P1-001 HIGH) found that the `action-pin-gate` CI
job scanned 0 workflow files silently — a missing existence guard on the scan-target
path made the entire scan a no-op. This was structurally identical to the STORY-158
trust-boundary gap (AC-158-004: scan-guard on cycle-artifact bins), yet it was not
caught during STORY-158 delivery.

The root cause is scope: per-story adversary passes are scoped to the story's own
deliverables. STORY-158's adversary passes correctly verified that lint-cycle-artifact
had proper existence guards — but the action-pin-gate job in `.github/workflows/ci.yml`
was outside STORY-158's declared deliverable scope, so no per-story pass reviewed it.

The wave-level integration adversary, by contrast, reviews the full develop tree with
no scope restriction. It caught the gap immediately.

**Lesson:**

Wave-level adversarial integration is structurally necessary for catching gaps in
files that are adjacent to — but not declared in — a story's file list. Per-story
adversary passes provide fine-grained verification within scope; wave-level passes
provide holistic verification across all stories and their interactions. Neither
substitutes for the other.

Codification: already implicit in the wave-gate procedure (wave-level adversary
dispatch on the full develop tree). No new story needed; reinforce the existing
wave-gate adversary dispatch scope.

**Tags:** `process-gap-caught`, `scope-boundary`, `wave-gate-value`

---

## Lesson 2 — [candidate-codification — next maintenance] Breaking JSON Changes Cascade to Holdout Expectations

**Observation:**

STORY-160 introduced a BREAKING JSON change (enum casing from PascalCase to
lowercase/snake_case + schema_version envelope). This was correctly implemented,
tested, and delivered. However, 13 holdout scenarios had stale expectations
hard-coded to the old enum casing and structure — none of them were identified
during per-story delivery.

The 13 scenarios were repaired by the product-owner at the integration gate holdout
re-evaluation step (HS-021/024/032/033/034/035/050/054/059/064/065/074/075 +
HS-INDEX v2.13). This was a significant unplanned work item at the gate.

The root cause: the BREAKING-change story's delivery protocol had no step requiring
a sweep of holdout-scenario expectations against the new output format. Holdout
scenarios are in `.factory/holdout-scenarios/`, outside the story's own test suite,
so the standard per-story TDD cycle did not cover them.

**Lesson:**

Any story tagged as BREAKING (or any story that changes observable output format —
JSON schema, enum casing, text layout) MUST include a holdout-expectation sweep
as an explicit delivery step: before the PR is opened, run the holdout evaluator
against the story's output changes and repair any stale expectations.

This is a new delivery-protocol obligation. Candidate addition: a mandatory checklist
item in the BREAKING-change story template (or the implementer delivery checklist)
requiring `holdout-expectations-sweep: COMPLETE` before PR creation.

Note: STORY-162 may carry the codification of this obligation if scope permits. If
not, it should be a wave-73 STORY-163.

**Tags:** `breaking-change`, `holdout-sweep`, `process-gap`, `flag-for-codification`

---

## Lesson 3 — [codified] STORY-162 — Hook-Forced Template Fields on Locked Docs Need Governance

**Observation:**

STORY-161 delivered VP-024 proof_file_hash codification and re-locked the VP with
the new hash. During delivery, the plugin hook forced `inputs:[]` / `input-hash:
d41d8cd` template fields onto VP-024 — a document that already had an active
`verification_lock`. This created a governance conflict: the hook automation assumes
all documents with its template fields should have them populated, but VP-024's
`verification_lock` semantics imply those fields should not be modified post-lock
without a deliberate re-lock procedure.

The STORY-161 delivery resolved this correctly (the hook-forced fields were
intentionally included as part of the re-lock; proof_file_hash was computed and
verified by three independent methods). However, the hook's behavior on locked
documents is undefined in the current governance documentation — a future delivery
agent might interpret the hook-forced fields as overriding the verification_lock.

**Lesson:**

Phase-5 (adversarial refinement) process documentation must clarify which VP fields
are in scope for hook-forced template enforcement and which are protected by
verification_lock. Specifically: `inputs:[]` and `input-hash:` fields should be
hook-governed; `proof_file_hash` and the `verification_lock` block should require
a deliberate re-lock procedure (as done in STORY-161), not automatic hook-forcing.

STORY-162 (E-11, wave-TBD) codifies the governance boundary
(PG-W72-LMR003-TEMPLATE-CONFORMANCE / F-S161P1-001).

**Tags:** `codified`, `governance`, `verification-lock`, `hook-automation`

---

## Lesson 4 — [candidate-codification — next maintenance] Triple-Verification for LMR-001-Permanent Hash Writes

**Observation:**

STORY-161 required computing `proof_file_hash` — a SHA-256 hash that would be written
as a permanent (LMR-001) value into VP-024. The orchestrator executed a triple-
verification protocol: (1) Python hashlib computation, (2) bash shasum/xxd
recomputation, (3) independent orchestrator recomputation. All three methods agreed
on `48296b21a5bbce59750e6210da8d55be8bf7d3d4a1ed6719088dd4ef59a2c8a5`.

This triple-verification produced a durable, independently-verified value. The
LMR-001 annotation on `proof_file_hash` (meaning: this value is set once and must
never be changed) makes correctness at write-time critical — there is no recovery
path for a wrong hash short of breaking the lock.

**Lesson:**

Any write governed by LMR-001-permanent semantics (a value that is set once and
frozen for the lifetime of the document) MUST use at least two independent
computation methods before committing. The three-method protocol used in STORY-161
(language-stdlib, shell-utility, independent-orchestrator-recomputation) is the
recommended template for hash-value writes. Record the computation evidence in the
implementation log before committing the value.

This discipline has no dedicated codification story; it is reinforced here as an
explicit lessons-learned entry for future VP proof_file_hash writes.

**Tags:** `discipline`, `lmr-001`, `hash-verification`, `correctness`

---

## Gate-Close Confirmation — Wave-72 CLOSED (D-416, 2026-07-09)

Wave-72 officially CLOSED by human approval at orchestrator gate (D-416, 2026-07-09). All dimensions green per D-415 gate-summary.

**Codification status summary (S-7.02):**

| Lesson | Tag | Codification vehicle |
|--------|-----|---------------------|
| Lesson 1 (a) — Wave-Level Sibling-Sweep | `[codified] via PR #391 CI hardening` | PR #391 action-pin-gate existence-guard fix codifies the wave-level adversary scope value |
| Lesson 2 (b) — Breaking JSON Holdout Sweep | `[candidate-codification — next maintenance]` | Needs a BREAKING-change delivery-protocol checklist item; route to next maintenance sweep or wave-73 story |
| Lesson 3 (c) — Hook-Forced Template on Locked Docs | `[codified] STORY-162` | STORY-162 (wave-TBD) codifies PG-W72-LMR003-TEMPLATE-CONFORMANCE governance boundary |
| Lesson 4 (d) — Triple-Verification LMR-001 Writes | `[candidate-codification — next maintenance]` | No dedicated story; reinforce via implementer checklist update in next maintenance sweep |
