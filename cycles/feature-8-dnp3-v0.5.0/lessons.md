---
document_type: lessons-learned
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-06-11T00:00:00Z
cycle: "feature-8-dnp3-v0.5.0"
session_dates: "2026-06-10 to 2026-06-11"
decisions: "D-047..D-054"
inputs: [STATE.md]
traces_to: STATE.md
---

# Lessons Learned — Feature #8 DNP3 / v0.5.0 Release

Durable lessons from the 2026-06-10/11 session covering:
- v0.5.0 release (MITRE ATT&CK-ICS v19 revocation fix, issue #222)
- Feature #8 DNP3 F1 scope, F2 spec (SS-15, ADR-007, VP-023), F3 decomposition
  (STORY-106..110, E-15, waves 35-39)

---

## Process-Level

### PG-5 [process-gap] Sibling-Sweep Must Cover ADRs, domain-debt, cap-10, and Design Drafts

**Tag:** `[process-gap]`

**What happened:** When a technique-ID, enum variant, or count changes, multiple HIGH
adversarial findings (this session: ADR-005/006 emission tables, cap-10 counts) surfaced
because the sibling-sweep that accompanied the change did not include:
- `docs/architecture/decisions/ADR-*.md` emission tables
- `domain-debt.md` technique references
- `cap-10-*` capability files with counts
- `docs/superpowers/specs/` and `docs/superpowers/plans/` design drafts

Each of these is a mandatory sweep target when any technique ID or enum variant is
added, removed, or renamed.

**Rule (DF-SIBLING-SWEEP-001 expansion):** When a technique-ID/enum/count changes,
the sibling-sweep checklist MUST enumerate: `docs/architecture/decisions/ADR-*.md`,
`domain-debt.md`, `cap-10-*`, and `docs/superpowers/` design drafts. Missing any one
of these is a HIGH finding in the next adversarial pass.

_Discovered: F2 adversarial pass, 2026-06-10_

---

### PG-7 [process-gap] BC-INDEX, PRD, and ARCH-INDEX Titles Must Be Updated in the Same Burst as Spec Changes

**Tag:** `[process-gap]`

**What happened:** Multiple adversarial passes caught stale title/count fields in
BC-INDEX, PRD, and ARCH-INDEX that were not updated in the same burst that added or
renamed a field/variant. This became a recurring HIGH finding across F2 and F3.

**Rule:** Any burst that changes a domain-spec section title, BC count, story count,
or wave count MUST update BC-INDEX, PRD headings, and ARCH-INDEX in the same commit.
These index files are mandatory co-change targets — not optional sibling sweeps.

_Discovered: F2/F3 adversarial passes, 2026-06-10_

---

### PG-8 [process-gap] Orphaned-Struct-Field / Partial-Fix-Propagation — Most Recurring This Session

**Tag:** `[process-gap]`

**Frequency:** Recurred across F2 HIGH-1/2, must-add C-1/C-2, and F3 C-1/M-2/F-PG2.
Every remediation burst that added or renamed a flow-state field or a count triggered
a follow-up adversarial pass because the fix was partial.

**What a partial fix looks like:**
- A new/renamed field appears in one copy of the struct sketch (e.g., architecture-delta)
  but not in the companion ADR struct sketch, or vice versa.
- The single reset-owner BC is not updated to include the new field in its reset list.
- All index/PRD/spec-changelog entries are not updated in the same burst.

**Rule (in-burst sibling checklist for new/renamed flow-state field OR
release-target/count change):**
When any burst adds or renames a flow-state field or changes a release-target/count, the
burst MUST atomically update ALL of:
1. Both struct copies (architecture-delta + ADR struct sketch).
2. The single reset-owner BC's reset list.
3. BC-INDEX, PRD, and ARCH-INDEX title/count fields.
4. spec-changelog entry.
5. grep-to-exhaustion across the full repo for the old field/count string before closing.

**Global grep-to-exhaustion (orchestrator-run) is the mechanism that breaks the partial-fix
cycle.** It must be run after any multi-file sweep, not left as an optional step.

_Discovered: F2/F3 adversarial passes, 2026-06-10/11_

---

### PG-9 [process-gap] Research-Validation Passes Catch Real Shipped Defects and Self-Hallucinations

**Tag:** `[pattern-confirmation]`

**What happened (v0.5.0):** A DNP3 research pass incidentally discovered that the shipped
v0.4.0 release was emitting T0855/T0856 — both REVOKED in ATT&CK-ICS v19.0. This was a
genuine shipped-release defect that surfaced only because a fresh research pass questioned
the project's own pinned assumptions. Two independent research passes (DF-VALIDATION-001)
confirmed the revocation; the research agent also caught and discarded its own
deep-research hallucinations via primary-source cross-check.

Adversarial fresh-context passes throughout F2/F3 repeatedly caught real defects (fabricated
technique name in 5 sites, window-reset contradiction, panic-prone arithmetic, un-buildable
orphaned fields) that prior passes and the original author missed.

**Pattern confirmed:** Independent research-validation is not overhead — it is the mechanism
that surfaces defects in assumptions the project has held for multiple waves. DF-VALIDATION-001
(no issue from unvalidated finding) and the adversary-independence wall are working as intended.

_Confirmed: v0.5.0 MITRE v19 remap, F2/F3 adversarial passes, 2026-06-10/11_

---

## Open Follow-Ups (carry into next session)

| Item | Category | Priority |
|------|----------|----------|
| F3 human-gate review — 3 open questions (decomposition granularity, VP-023 placement, linear-chain parallelism) | gate | HIGH |
| F4 start: wave 35 STORY-106 parse-core + VP-023 Kani authoring | next-step | HIGH |
| 6 open Dependabot PRs | maintenance | MEDIUM |
| PCAP-CORPUS-001 (local E2E pcap corpus expansion) | maintenance | MEDIUM |
| Roadmap issues #3, #4, #6 | roadmap | MEDIUM |
| DRIFT-F2-COUNT-001 (F2 count field drift) | drift | LOW |
| DRIFT-SUPERPOWERS-001 (superpowers design draft sync) | drift | LOW |
| PG-5/7/8 codification into policy candidates (DF-SIBLING-SWEEP-001 expansion + new PG-8 rule) | policy | MEDIUM |

---

## Policy Candidates

| Lesson | Proposed Policy / Rule | Scope | Status |
|--------|------------------------|-------|--------|
| PG-5 | Expand DF-SIBLING-SWEEP-001: ADRs + domain-debt + cap-10 + superpowers drafts as mandatory sweep targets | Adversarial-pass entry checklist | proposed |
| PG-7 | BC-INDEX / PRD / ARCH-INDEX titles are mandatory co-change targets (same burst as spec changes) | All burst checklists | proposed |
| PG-8 | In-burst sibling checklist for new/renamed flow-state field or count change (both struct copies + reset-owner BC + indices + spec-changelog + grep-to-exhaustion) | Burst authoring rules | proposed |
| PG-F4-F5-001 | DF-CANONICAL-FRAME-HOLDOUT-001: protocol framing invariants require at least one holdout using canonical spec byte sequence | Holdout authorship, F3 decomposition | proposed |
| PG-F4-F5-002 | Expand DF-ADVERSARY-METHODOLOGY-001: F5 must include BC-completeness sweep (is every BC Invariant implemented?) | F5 holistic review checklist | proposed |
| PG-F4-F5-003 | Embed checkout-guard block verbatim in adversary dispatch template (DF-ADVERSARY-CHECKOUT-GUARD-001 enforcement) | Orchestrator dispatch template | proposed |
| PG-F4-F5-004 | Orchestrator must run cargo fmt before direct-committing agent-authored code | Orchestrator commit procedure | proposed |
| PG-F4-F5-005 | Update pr-manager prompt to emit consolidated report (PR#, pr-reviewer, security, CI) per DF-PR-MANAGER-COMPLETE-001 | pr-manager agent prompt | proposed |
| PG-F4-F5-006 | Pre-implementation agentic-sliced design review as default step for protocol/security-invariant features | F5 remediation playbook | proposed (IP-3) |

---

## F4-F6 Session Lessons (appended 2026-06-12)

_Session arc: F4 delivery (waves 37-39, STORY-108/109/110) + F5 scoped adversarial +
F5 remediation + F6 formal hardening. Full analysis: `session-review-f4-f6.md`._

---

### PG-F4-F5-001 [process-gap] Canonical-Frame Holdout Cross-Validation for Protocol Framing Invariants

**Tag:** `[process-gap]`

**What happened:** The `is_master_frame` DIR-bit mask was wrong (0x10 instead of 0x80,
where DIR is bit 7 per IEEE 1815 DNP3). The bug was latent since STORY-107 (wave 36) and
survived approximately 30 per-story adversarial passes across STORY-107, 108, 109, and 110.
Root cause: the per-story adversary reviews code against the project's own BC and tests.
When the BC (PC5) and the tests both used the same wrong mask (an internally self-consistent
error), the adversary had no external reference to detect it.

The fix was found by the F5 agentic-sliced pre-implementation review, which cross-checked
the mask against the DNP3 link-layer bit layout in IEEE 1815 independently.

**Lesson:** Per-story adversarial cannot detect protocol-spec level errors that are
internally self-consistent between code, tests, and project BCs. The only reliable
mechanism is cross-validation against the authoritative protocol specification, either
via a holdout scenario that uses a canonical spec byte sequence, or via a fresh-context
adversary given the spec as a reference.

**Rule:** For any protocol-framing invariant (direction bits, function codes, frame
types, magic bytes), at least one holdout scenario MUST use a canonical byte sequence
from the authoritative protocol specification — NOT a value derived from the project's
own BCs. The holdout scenario must cite the spec section and the canonical byte value
explicitly. Stories implementing protocol-framing invariants must require this in their
AC.

**Proposed policy:** `DF-CANONICAL-FRAME-HOLDOUT-001` in `.factory/policies.yaml`.

**Follow-up recommendation:** Add to F3 story decomposition checklist. Warrant: YES —
this is a self-improvement story candidate (or policy-file change) before the next
protocol feature begins.

_Discovered: F5 agentic-sliced pre-impl adversarial review, 2026-06-12_

---

### PG-F4-F5-002 [process-gap] BC-Set-Completeness Check in F5 Holistic Review

**Tag:** `[process-gap]`

**What happened:** BC-2.15.010 Invariant 5 (unexpected-source detection) was entirely
absent from the implementation after 5 stories were delivered. The per-story adversarial
passes correctly verified that existing detection branches satisfied their own BCs. None
of the 34 per-story passes checked whether Invariant 5 had any corresponding code path.
The F5 holistic adversarial review found the gap by scanning the full delta for coverage
of each BC Invariant.

**Lesson:** Per-story adversarial reviews correctness of what exists. Completeness of
what was specified (is every BC Invariant implemented?) requires a different check — a
scan of the BC corpus against the implementation, not a scan of the implementation
against each story's BCs.

**Rule:** F5 scoped adversarial review MUST include a mandatory first step: for each BC
in the feature's spec that specifies a detection invariant, guard condition, or emission
requirement, confirm that a corresponding implementation path EXISTS in the feature delta.
A BC invariant with no corresponding implementation path is a P0 BLOCKER finding.

**Proposed policy change:** Expand `DF-ADVERSARY-METHODOLOGY-001` to include the
completeness sweep as a numbered step.

**Follow-up recommendation:** Warrant: YES — add to DF-ADVERSARY-METHODOLOGY-001 scope
before next feature-level F5 review.

_Discovered: F5 holistic adversarial delta review, 2026-06-12_

---

### PG-F4-F5-003 [process-gap] Pre-Implementation Adversarial Design Review for High-Risk Features

**Tag:** `[codified]`

**What happened:** Before TDD implementation of F5 remediations, an agentic-sliced
adversarial review was run in 3 parallel slices (F-001 design, F-003 design,
spec-consistency). The review found 2 design BLOCKERs before any code was written:

1. F-A-001: DIR-bit mask error (described under PG-F4-F5-001 above)
2. F-B-002: Overflow `clear+return` in the proposed resync design created a data-loss
   path (Crain-Sistrunk evasion vector) that was not visible in the initial design sketch

Both required ARCHITECT REVISION-2 directives. The estimated savings: 5-8 convergence
passes that would have been needed to discover these flaws post-implementation.

**Lesson:** For designs touching protocol framing invariants, multi-path detection logic,
or security-invariant BCs, a pre-implementation adversarial design review is high-leverage.
The cost is one additional step before TDD; the benefit is avoiding design-rework in the
middle of the convergence cycle.

**Rule:** When F5 holistic review identifies issues involving (a) protocol framing
invariants, (b) detection logic with security-invariant BCs (T1692.001/T0827 class),
or (c) multi-path byte-walk algorithms, dispatch an agentic-sliced pre-implementation
adversarial design review (2-4 slices, one per concern area) BEFORE TDD authoring begins.
Each slice receives: the adjudication directive, the relevant BCs, and the authoritative
protocol spec section. Any BLOCKER finding triggers ARCHITECT REVISION before coding begins.

**Follow-up recommendation:** Warrant: YES — codify as a Step 0 in the F5 remediation
playbook template with trigger conditions listed above.

_Discovered/confirmed: F5 pre-impl adversarial design review, 2026-06-12_

---

### PG-F4-F5-004 [process-gap] Mandatory Adversary Checkout-Guard in Worktree Dispatch

**Tag:** `[process-gap]`

**What happened:** F5 convergence pass P9 reviewed `develop` HEAD (pre-fix state) instead
of the feature worktree (post-fix state). The adversary produced 3 BLOCKER findings
describing code that had already been fixed in the worktree. The orchestrator caught the
error by cross-checking the file paths cited; P10 was re-dispatched on the correct
worktree and returned CLEAN.

Impact: 1 wasted convergence pass (~20-30 minutes). If uncaught, would have triggered
unnecessary rework.

**Lesson:** An adversary reviewing the wrong codebase produces invalid findings. Policy
DF-ADVERSARY-CHECKOUT-GUARD-001 exists and is correct, but it must be embedded in the
dispatch template — not added post-hoc when a failure is suspected. The later passes
(P10) used the guard explicitly and worked correctly.

**Rule (operational):** The orchestrator's adversarial dispatch template MUST include
a checkout-guard block as the FIRST instruction in every worktree-based review pass:

1. Run `git -C <worktree_path> log -1 --format='%H %s'` and log the result.
2. Confirm the HEAD SHA matches the expected worktree HEAD provided by the orchestrator.
3. Spot-check: read the file that contained the most recently fixed BLOCKER finding;
   confirm the fix is present (cite the line checked).
4. If either check fails: STOP. Report "CHECKOUT GUARD FAILED — invalid context." Do
   NOT produce findings. The pass is invalid and must be re-dispatched.

**Follow-up recommendation:** Warrant: YES — update the adversarial dispatch template
to embed this block verbatim (not as a reminder, but as mandatory instructions).

_Discovered: F5 convergence pass P9, 2026-06-12_

---

### PG-F4-F5-005 [process-gap] Orchestrator Must Run cargo fmt Before Direct-Committing Agent-Authored Code

**Tag:** `[process-gap]`

**What happened:** CI Format gate failed on PRs #228 (STORY-109) and #231 (F6) because
the orchestrator committed agent-authored test changes via direct git, bypassing the
authoring agent's `cargo fmt` step. The authoring agent runs `cargo fmt` as part of its
delivery step; the direct-commit path does not. Frequency: 2/5 PRs (40%) in this session.

**Lesson:** When the orchestrator takes code from an agent's output and commits it
directly (without routing through the agent's normal delivery step), the format step is
skipped. This predictably produces CI Format gate failures.

**Rule:** When committing agent-authored code via direct git (not via the authoring
agent's own commit step), the orchestrator MUST run `cargo fmt` (or `cargo fmt --check`
as a gate) before the commit. Skipping this step WILL produce a CI Format gate failure.
The rule applies to: test changes received from the test-writer agent, fixup patches
received from the implementer agent, or any code received as text output rather than
via the agent's normal commit path.

**Follow-up recommendation:** Warrant: YES — add to orchestrator commit procedure as a
mandatory pre-commit check. This is a simple procedure change (1 line: `cargo fmt`
before `git commit`).

_Discovered: STORY-109 PR #228 and F6 PR #231 CI failures, 2026-06-12_

---

### PG-F4-F5-006 [process-gap] pr-manager Must Return Consolidated Report (Policy Non-Compliance)

**Tag:** `[process-gap]`

**What happened:** Across multiple PRs in this session (#227, #228, #229, #230),
pr-manager returned only the security review verdict. The pr-reviewer approval verdict,
PR number, and CI status were not included, requiring the orchestrator to dispatch
pr-reviewer separately and query CI status manually each time.

Policy DF-PR-MANAGER-COMPLETE-001 already requires a consolidated report. The agent is
not complying with an existing policy. This is an agent-prompt gap.

**Lesson:** Policies are only enforced when embedded in the agent prompt. DF-PR-MANAGER-
COMPLETE-001 exists in policies.yaml but the pr-manager agent prompt does not require
the consolidated output format.

**Rule (operational):** The pr-manager agent prompt MUST require the following
consolidated report as the mandatory final section of every run:

```
## Consolidated Gate Report
- PR number: #NNN
- pr-reviewer verdict: [APPROVE / REQUEST_CHANGES / REJECT]
- security verdict: [APPROVE_WITH_NOTES / APPROVE / REJECT] (N CRITICAL, N HIGH, N MED, N LOW)
- CI status: [GREEN / FAILING] (job list with pass/fail per job)
- Merge commit: <SHA> (if merged)
- Blocking items: <list or "none">
```

**Follow-up recommendation:** Warrant: YES — update pr-manager agent prompt immediately.
DF-PR-MANAGER-COMPLETE-001 already exists; this is a compliance fix, not a new policy.
