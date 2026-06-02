---
document_type: lessons-learned
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-06-02T00:00:00Z
cycle: "v0.1.0-greenfield-spec"
inputs: [STATE.md]
traces_to: STATE.md
---

# Lessons Learned — v0.1.0-greenfield-spec

Durable lessons from this cycle for future VSDD factory runs.
Organized by category. Each lesson tagged with phase/burst of discovery.

---

## Process-Level

### 1. [process-gap] PG-1: Mutation scope must cover ALL CRITICAL/HIGH modules, not just output modules

**Tag:** `[process-gap]`

**What happened:** `tooling-selection.md` scoped mutation testing to SS-06/07/08/09/10
(analyzers/mitre/findings — output/classification modules). The CRITICAL reassembly
modules SS-04 (flow.rs, segment.rs, mod.rs) were omitted entirely. These are the
anti-evasion core: segment.rs implements TCP reassembly overlap resolution, the exact
logic adversaries target. The `ranges_overlap` predicate in segment.rs had zero mutation
coverage for the entirety of Phase 6 until the gap was caught during human gate review
and a remediation pass was dispatched (PR #184).

**Why it matters:** The most security-critical modules are precisely those where mutants
survive longest, because they are often under-tested relative to their importance. Scoping
mutation testing to "what produces output" rather than "what is CRITICAL" inverts the
risk model.

**Recommendation:** In hardening-gate checklist, require a mutation-scope coverage table
keyed to module-criticality tiers (CRITICAL/HIGH/MEDIUM/LOW). Every CRITICAL and HIGH
module must appear in the mutation scope before the Phase-6 gate can PASS.

**Follow-up (spec-fix):** `tooling-selection.md` mutation scope section must be corrected
to enumerate ALL CRITICAL/HIGH tier modules from module-criticality.md. Status: OPEN (draft)
— assigned to architect for next inter-phase maintenance pass or cycle start.

_Discovered: Phase 6 hardening-gate review, 2026-06-02_

---

### 2. [process-gap] PG-2: CRITICAL-tier security invariants must default to PROVEN, not JUSTIFIED-via-debug-guard

**Tag:** `[process-gap]`

**What happened:** VP-002 (CRITICAL anti-evasion: winner-selection in segment overlap
resolution) was initially assessed as "justified" — meaning test-covered rather than
formally proven. The justification relied on a collision guard that is only active
in debug builds (`#[cfg(debug_assertions)]`). In release builds, the silent-overwrite
risk was real and untested. Human gate review caught this gap; a remediation burst
was dispatched to extract the pure `select_gaps` function and prove it with Kani
(PR #183, 2 harnesses, 180 checks SUCCESSFUL). VP-002 was upgraded from JUSTIFIED
to PROVEN.

**Why it matters:** A debug-only guard is NOT a correctness proof for a release build.
Any VP marked "justified" that relies on `#[cfg(debug_assertions)]` behavior as the
sole safety net for a CRITICAL invariant must be treated as unproven.

**Recommendation (hardening-gate checklist addition):**
- CRITICAL-tier VPs: default acceptance criterion is PROVEN (Kani/proptest oracle).
  JUSTIFIED is only acceptable if the justification cites production-path tests that
  cover the invariant independent of debug guards.
- At gate review: explicitly grep VP "justified" entries for `debug_assertions` in
  the referenced justification file. Flag any that rely on debug-only guards as
  requiring formal proof before closing.

**Follow-up disposition:** CLOSED — lesson recorded; checklist recommendation above
is the actionable output. No separate tracking item required; recommendation should
be folded into the Phase-6 entry checklist before next cycle's Phase 6 runs.

_Discovered: Phase 6 VP-002 hardening-gate review, 2026-06-02_

---

### 3. [process-gap] PG-3: Agents must branch off origin/develop, never local develop

**Tag:** `[process-gap]`

**What happened:** During the Phase-6 reassembly mutation pass, a sub-agent was
nearly run against a stale local `develop` branch. The local ref had not been
fast-forwarded after the most recent squash-merges to origin/develop. If the
mutation pass had executed against the stale HEAD, the results would have been
invalid (covering a codebase version missing several merged fixes), and the
discrepancy might not have been detected until gate review — wasting a full
mutation run.

The issue was caught via an explicit HEAD cross-check (`git log origin/develop -1`
vs `git log develop -1`).

**Why it matters:** In a factory pipeline with multiple bursts and sub-agents,
local refs silently lag origin. The stale-develop failure mode is subtle: `cargo test`
passes, mutation runs, results look valid — but the code under test is wrong.

**Recommendation:**
- Agents dispatched to run toolchain passes (mutation, fuzz, Kani) must:
  1. Run `git fetch origin` before branching.
  2. Branch from `origin/develop`, not `develop`.
  3. Assert `git rev-parse HEAD == git rev-parse origin/develop` before running any
     toolchain pass.
- Orchestrator: after every squash-merge to origin/develop, sync local develop:
  `git fetch origin && git checkout develop && git reset --hard origin/develop`
  before dispatching the next sub-agent.

**Note:** `DF-DEVELOP-FRESHNESS-001` policy already governs freshness at PR open time.
This recommendation extends freshness enforcement to toolchain-pass dispatch time
(pre-branch, not just pre-PR).

**Follow-up disposition:** CLOSED — lesson recorded; process recommendation above
is the actionable output. DF-DEVELOP-FRESHNESS-001 should be annotated to cover
toolchain-pass dispatch; add as a policy clarification item for next inter-phase
maintenance pass.

_Discovered: Phase 6 reassembly mutation pass dispatch, 2026-06-02_

---

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| PG-1 | DF-MUTATION-SCOPE-001: mutation coverage table must include all CRITICAL/HIGH modules before Phase-6 gate | Phase-6 hardening checklist | proposed |
| PG-2 | Extend hardening-gate checklist: CRITICAL VPs with debug-only justification require PROVEN before gate PASS | Phase-6 entry checklist | proposed |
| PG-3 | Extend DF-DEVELOP-FRESHNESS-001: freshness assertion at toolchain-pass dispatch time | DF-DEVELOP-FRESHNESS-001 amendment | proposed |
