---
document_type: lessons-learned
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-06-18T23:59:00Z
cycle: "feature-collapse-v0.8.0"
inputs: [STATE.md]
traces_to: STATE.md
---

# Lessons Learned — feature-collapse-v0.8.0 (E-8 / issue #62 F1..F3 cycle)

<!-- Durable lessons from this cycle for future VSDD factory runs.
     Organized by category: agent-level, process-level, infrastructure-level.
     Each lesson is numbered continuously and includes the pass/burst
     where it was discovered. -->

## Process-Level

1. **F1/F2 story-input analysis docs must pass a full numeric self-audit at authoring time** — During F3 for E-8 / issue #62, the F1 delta-analysis doc (declared as a STORY-120 input via `inputs:` frontmatter) accumulated stale sub-counts across 9 fix rounds. Every correction to F1 re-triggered STORY-120 input-hash recompute, spawning a fresh adversarial re-derivation cycle. The root cause (codified across D-099/D-100/D-101): F1/F2 phase analysis docs that are declared story inputs MUST be exhaustively self-audited for numeric accuracy vs grep ground-truth at authoring time. This is the most-churned phase of this cycle (10 fix rounds, majority documentation-hygiene).
   _Discovered: F3 Round-7 fix-burst, 2026-06-18 (D-099); extended D-100/D-101._
   **Follow-up:** STORY-121 (E-11, draft) filed (D-103, 2026-06-18) to codify this as a mandatory numeric self-audit checklist + consuming-surface sweep checklist for F1/F2 story-input docs. Satisfies Cycle-Closing Checklist step 3 for process-gap D-099/D-100/D-101.

2. **BC version bump must sweep ALL consuming surfaces together** — Each BC line-anchor correction or version bump in rounds 4–6 created propagation drift into: story body BC-table version cells, frontmatter `# BC status:` comments, STORY-119 forward-ref BC table, dep-graph BC-to-Stories matrix. The round-7 9-location sweep of the '35→28' headline count missed 1 additional occurrence (OQ-3 in F1). Root cause: no single-pass exhaustive sweep protocol. Policy candidate reinforces PG-62-F2-BOOKKEEPING-SWEEP-001: a BC bump must sweep BC file + BC-INDEX + spec-changelog + consuming-story body + frontmatter comment + dep-graph matrix in one pass.
   _Discovered: F3 Round-5 and Round-8 fix-bursts, 2026-06-18 (D-097/D-100)._

3. **AC code blocks must reference only variables provably in scope at cited file:line** — F3 Round-1 adversary caught CRITICAL: AC-005 prescribed vars `*mitre`/`no_collapse` that are out of scope at the `run_analyze` construction site. Root cause: story-writer did not verify variable scope at each cited file:line. Policy candidate PG-62-F3-AC-SCOPE.
   _Discovered: F3 Round-1 triple, 2026-06-18 (D-093)._

4. **AC trace descriptions for BC citations must be copied verbatim from BC postcondition text** — F3 Round-3 adversary caught CRITICAL: round-2 AC-trace descriptions for BC-2.11.015/016 were semantically INVERTED (BC-015 mislabeled "colorization", BC-016 mislabeled "uncategorized"). Root cause: story-writer paraphrased BC postconditions from memory rather than reading the actual BC file. Policy candidate PG-62-F3-AC-DESC-FROM-SOURCE.
   _Discovered: F3 Round-3 triple, 2026-06-18 (D-095)._

5. **DF-SIBLING-SWEEP-001 must cover same defect class across all sibling BCs** — BC-2.11.029 Architecture-Anchor carried the same out-of-scope wiring expression as BC-2.11.028 (fixed in Round-4), but the Round-4 sibling-sweep covered only the dispatch-anchor pattern, not the wiring-expression pattern. Reinforces DF-SIBLING-SWEEP-001: fix must sweep ALL siblings for the SAME defect class, not just the named instance.
   _Discovered: F3 Round-6 triple, 2026-06-18 (D-098)._

6. **Post-merge anchor drift is a consuming-surface gap** — F5 Pass-3 found that all 12 SS-11 BC line-anchors and ADR-0003 became stale the moment STORY-120 merged (shifting terminal.rs lines ~52-160). The anchor re-validation passes had been run against the spec-branch HEAD, not the post-merge feature SHA. Resolved by BC-INDEX v1.43 re-anchor by symbol + ADR-0003+CHANGELOG fix-PR #267. Additionally F7 consistency audit found VP-016 was also a missed consuming surface (stale struct-field snippets in test-spec code blocks). Root cause: the consuming-surface checklist for any story refactoring a file that other BCs/VPs anchor into must include a post-merge anchor-revalidation step. Codification via STORY-121 (PG-62-F5-POSTMERGE-ANCHOR-001).
   _Discovered: F5 Pass-3, 2026-06-18 (D-105/D-106); extended F7 consistency (D-108)._

## Cycle-Closing Record (2026-06-18)

**E-8 / #62 cycle status: CLOSED-PENDING-RELEASE (D-109).**

- All process-gap findings from this cycle have follow-up coverage:
  - **STORY-121 (E-11, draft)** covers D-099/D-100/D-101 (F1/F2 story-input numeric self-audit + consuming-surface sweep checklist) + PG-62-F5-POSTMERGE-ANCHOR-001 (post-merge anchor drift) + VP-016/consuming-surface gap (D-108 sub-note). Filed 2026-06-18.
  - STORY-121 is the designated vehicle for all F1-F7 #62 process-gap policy codification.
- **Release v0.9.0 HELD** per human (D-109). Cargo 0.9.0 on develop f851995. Holding until more work is bundled (specifically the upcoming STORY-119 grouped-mode collapse cycle).
- **Next cycle:** STORY-119 (grouped-mode finding-collapse). Feature-Mode F1 delta-analysis. depends_on [STORY-120] (unblocked).

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| 1 | Mandatory numeric self-audit for F1/F2 story-input docs (STORY-121) | F1/F2 story-input analysis docs declared in story `inputs:` frontmatter | proposed — STORY-121 filed |
| 2 | BC-bump consuming-surface sweep checklist | BC version bump + propagation to all consuming artifacts | proposed — codification pending |
| 3 | PG-62-F3-AC-SCOPE — AC code blocks scope-verified at cited file:line | Story AC authoring | proposed — policy codification pending |
| 4 | PG-62-F3-AC-DESC-FROM-SOURCE — AC trace descriptions verbatim from BC PC text | Story AC authoring | proposed — policy codification pending |
| 5 | DF-SIBLING-SWEEP-001 extension — sweep same defect class across all sibling BCs | Post-fix BC sweep | reinforces existing policy |
| 6 | PG-62-F5-POSTMERGE-ANCHOR-001 — post-merge anchor revalidation (BCs + VPs) against merged feature SHA | Cycle-closing checklist for stories refactoring files that other BCs/VPs anchor into | proposed — STORY-121 filed |
