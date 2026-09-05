---
document_type: lessons-learned
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-09-04T00:00:00Z
cycle: "wave-086"
inputs: [STATE.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Lessons Learned — wave-086

Captured during the wave-86 story-level adversarial convergence loop (in-progress). Final
S-7.02 cycle-closing consolidation happens at wave-86 gate close; this file accumulates
in-cycle process insights as they occur.

Wave: 86 | Status: IN-PROGRESS (streak 0/3, pass 25 next as of D-541, 2026-09-04).

---

## Agent-Level

_(none captured yet this wave — see `cycles/wave-086/process-gap-ledger.md` for agent-behavior
process gaps PG-W86-005/006/009/010/ADVERSARY-WRITE-PROFILE, which are tracked there rather
than duplicated here.)_

## Process-Level

1. **A pass-N NIT accepted as "documented residual" to preserve a clean-pass streak was
   independently ESCALATED to MEDIUM by the pass-N+1 fresh-context adversary
   (PG-W86-RESIDUAL-MISQUOTE-ESCALATION)** — Pass 23 found a markdown-emphasis mismatch
   between a STORY-183 Task-10 bullet and its FSR row and rated it NIT, disposing it as an
   accepted residual specifically to let the wave's first clean-pass streak accumulate.
   Pass 24's fresh-context adversary — with no visibility into pass 23's disposition
   rationale — independently found the same locus and rated it MEDIUM, because the
   "cosmetic" framing had concealed a live-source misquote (bold markdown that does not
   exist in `bin/check-green-doc-tense:4`) plus an intra-document contradiction (the Task-10
   rewrite text disagreed with the FSR row's plain-text prescription).

   **LESSON/RULE:** Do not accept as a documented residual any finding that involves (a) a
   quote that does not match live source, or (b) two loci of the same document contradicting
   each other, even when its functional impact is nil. Remediate such items immediately
   rather than deferring, since a fresh adversary will (correctly) re-raise them at higher
   severity and reset the streak the deferral was meant to protect. The discriminator for
   "safe to defer as residual" is whether the finding is purely a preference/stylistic choice
   with no live-source or intra-document factual conflict.

   _Discovered: D-541, 2026-09-04 (pass 24, F-W86S-P24-001). Disposition: non-blocking for
   delivery (STORY-183 remediated same burst, v2.12→v2.13); codification-tracking only —
   requires DF-VALIDATION-001 research-agent validation before filing as a GitHub issue. See
   `cycles/wave-086/process-gap-ledger.md` § PG-W86-RESIDUAL-MISQUOTE-ESCALATION._

2. **Edit/Write tool calls resolved to the main repo checkout instead of the story
   worktree during STORY-183 implementation, leaking a stray uncommitted edit to
   `bin/check-green-doc-tense` onto `develop` (PG-W86-EDIT-WORKTREE-PATH-HAZARD)** —
   during STORY-183 implementation, one or more Edit/Write tool invocations resolved
   against the MAIN repo checkout (branch `develop`) rather than the story's dedicated
   worktree, leaving a stray uncommitted modification to `bin/check-green-doc-tense` on
   `develop`. The implementer detected the mismatch via `git status`, reverted the
   stray change (`git -C <main-repo> checkout -- bin/check-green-doc-tense`), confirmed
   `develop` clean, and completed the remaining edits via Bash-executed scripts
   targeting the worktree path directly.

   **LESSON/RULE:** Implementers working in a worktree MUST verify Edit/Write path
   resolution — run `git -C <main-repo> status` after edits to confirm no leakage to
   the main checkout; prefer worktree-absolute paths, or Bash-verified writes, when the
   Edit tool's cwd resolution is ambiguous.

   _Discovered: STORY-183 implementation, 2026-09-05. Disposition: codification-tracking
   only — STORY-183 shipped correct; no defect reached `develop`. Requires
   DF-VALIDATION-001 research-agent validation before filing as a GitHub issue. See
   `cycles/wave-086/process-gap-ledger.md` § PG-W86-EDIT-WORKTREE-PATH-HAZARD._

## Infrastructure-Level

_(none captured yet this wave.)_

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| 1 | PG-W86-RESIDUAL-MISQUOTE-ESCALATION | Adversary/orchestrator residual-acceptance discipline: forbid deferring a finding as a documented residual if it involves a live-source misquote or an intra-document contradiction, regardless of apparent stylistic framing | proposed |

---

Prior wave-86 process gaps (PG-W86-001 through PG-W86-AUDIT-SEAM-PIPEFAIL) are tracked in
`cycles/wave-086/process-gap-ledger.md`, not duplicated here. This file records the narrative
lesson; the ledger records the codification-tracking entry.
