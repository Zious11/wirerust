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
