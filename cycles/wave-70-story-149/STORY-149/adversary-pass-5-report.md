---
document_type: adversary-pass-report
level: ops
version: "1.0"
story: STORY-149
cycle: wave-70-story-149
pass: 5
date: "2026-07-07"
worktree_head_reviewed: 208b2d4
story_spec_at_review: v1.4 (d68af34)
classification: FINDINGS
producer: adversary (wave-70)
traces_to: STORY-149
bc_gating: BC-5.39.001
---

# Adversary Pass 5 Report — STORY-149

## Checkout Guard

**Result: PASS**

Worktree head reviewed: `208b2d4` (story now at v1.4, `d68af34`). Branch
`feature/STORY-149-tls-carry-perf` confirmed at correct head. No uncommitted changes.

## Classification

**FINDINGS** — 2 findings: 1 MEDIUM (F-S149P5-001), 1 LOW OBSERVATION (F-S149P5-002,
ruled no-action). Clean streak stays 0.

## Pass-4 Fix Verification

| Finding | Status | Notes |
|---------|--------|-------|
| F-S149P4-001 — phantom TlsFlowState.carry | FIXED | Architecture Mapping in v1.4 correctly uses `client_hs_carry` / `server_hs_carry`; old `TlsFlowState.carry` reference absent from story |

## Findings Detail

### MEDIUM

**F-S149P5-001** — Task-4 budget-annotation clause unimplemented + Compliance Rules
phantom CI gate

STORY-149 Task-4 required that every call site consuming the borrow budget carry a
`// BORROW BUDGET:` inline annotation marker, and that a BORROW BUDGET doc-paragraph
header in the relevant module enumerate the full budget. At `208b2d4`, no such markers
existed in the implementation and no BORROW BUDGET doc paragraph had been written.

The story's Compliance Rules section compounded this by asserting: "A CI test enforces
that annotation marker count equals the number of documented budget sites." No such test
existed at `208b2d4`. The Compliance Rules claim was therefore false — it asserted an
enforcement mechanism that had not been implemented.

This finding has two components which are coupled: the annotation scaffolding must exist
before the test can enforce it, and the test must exist for the Compliance Rules claim to
be true. Both components require remediation together.

Remediation route:
1. Add the BORROW BUDGET doc paragraph to the relevant module with enumeration of all
   budget-consuming call sites.
2. Add `// BORROW BUDGET:` inline markers at the 3 implementation call sites.
3. Add `test_BC_149_001_process_handshake_carry_budget_annotations_match_sites` — a test
   that greps the implementation for both the doc-paragraph marker and the inline-site
   markers, verifies all are present, and asserts that the inline-site count equals the
   value stated in the doc paragraph. This makes the Compliance Rules CI-gate claim true
   without requiring a story amendment.

Remediated in: `5b41eca` (BORROW BUDGET doc paragraph + 3 inline site markers) and
`edb2b8c` (new enforcement test). No story amendment required — the Compliance Rules
claim becomes retrospectively accurate once `edb2b8c` is present.

### LOW — OBSERVATION (no-action)

**F-S149P5-002** — Anti-gameability guard heuristic not exhaustive

The inspection test's anti-gameability defense relies on a grep-count heuristic: it
counts occurrences of `flows.get_mut(` in `try_parse_records` and asserts the count
equals exactly 1. A sufficiently motivated adversary with knowledge of this heuristic
could construct a two-borrow pattern where one borrow site uses a different method
spelling (e.g., `flows.get(` followed by an unwrap, or a rebind through a local
binding) that collapses to a single grep hit while still creating a second logical
borrow.

This is a theoretical attack surface. The adversary must:
1. Know the specific grep pattern used in the test.
2. Construct a non-obvious two-borrow pattern that bypasses it.
3. Pass all other companion tests including `_no_aliasing_patterns_hide_borrow_count`.

**Ruling: NO_ACTION.** The layered defense (exact-count assertion + aliasing-pattern
companion test + Kani structural verification of the borrow invariant) provides adequate
coverage for this risk level. The theoretical attack surface does not warrant a deeper
refactor — doing so would require either semantic proof of single-borrow (beyond the
scope of TDD phase) or a full AST-level analysis, both of which are formal-hardening
concerns, not adversarial-review findings.

This observation is recorded for the formal-hardening agent (Phase 6) as a candidate for
Kani-level borrow-count proof if warranted.

## Convergence State

Pass 5 complete. Clean streak: **0 / 3** required (FINDINGS raised, observation no-action).
Pass 6 pending against remediation head `edb2b8c`.
