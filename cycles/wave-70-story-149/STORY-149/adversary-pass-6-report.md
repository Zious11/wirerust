---
document_type: adversary-pass-report
level: ops
version: "1.0"
story: STORY-149
cycle: wave-70-story-149
pass: 6
date: "2026-07-07"
worktree_head_reviewed: edb2b8c
story_spec_at_review: v1.4 (d68af34)
classification: NITPICK_ONLY
producer: adversary (wave-70)
traces_to: STORY-149
bc_gating: BC-5.39.001
---

# Adversary Pass 6 Report — STORY-149

## Checkout Guard

**Result: PASS**

Worktree head reviewed: `edb2b8c`. Checkout-guard attestation received:

| Check | Result |
|-------|--------|
| HEAD ref quoted | `edb2b8c` — PASS |
| Grep-counts (annotation site sweep) | 5/3 PASS — 5 marker instances confirmed (1 doc-paragraph + 3 inline + 1 test reference); 3 body-site markers correctly counted by enforcement test |
| Freshness | CONFIRM — worktree at `edb2b8c`, no stale refs |

## Classification

**NITPICK_ONLY** — 1 finding: 1 LOW doc-only item (F-S149P6-001). ZERO code findings.

This pass qualifies as a clean pass under convergence policy: NITPICK_ONLY classification
with zero code changes. Doc-only remediation committed to factory artifacts (`2418048`)
does not touch implementation or test code and does NOT void the clean pass.

Clean streak: **1 / 3** after pass 6.

## Pass-5 Finding Verification

### F-S149P5-001 — FIXED and NON-TAUTOLOGICAL

The Task-4 budget-annotation clause and CI-gate claim are both satisfied at `edb2b8c`:

- BORROW BUDGET doc paragraph is present in the implementation module.
- 3 inline `// BORROW BUDGET:` markers are present at the identified call sites.
- `test_BC_149_001_process_handshake_carry_budget_annotations_match_sites` is present
  and enforces marker presence and count equality.

**Non-tautological verification** — the enforcement test was independently probed for
extraction-region correctness:

1. **Doc-paragraph marker excluded from body count**: the test's grep region for counting
   body-site markers is bounded to exclude the doc-paragraph header line; a doc-paragraph
   marker does not inflate the body count. Confirmed: the header marker and body markers
   use distinct region delimiters in the test logic.

2. **Bidirectional failure modes confirmed**:
   - Removing any inline marker from the implementation causes the test to fail
     (count drops below the declared budget).
   - Removing the doc-paragraph header causes the test to fail (presence assertion fires).
   - Adding a spurious inline marker without updating the doc-paragraph count causes the
     test to fail (count exceeds declared budget).

The test is non-tautological: it cannot be trivially gamed by removing both the markers
and the doc-paragraph count together (the test would then fail on the presence assertion
for the doc-paragraph header).

### F-S149P5-002 — NO-ACTION UPHELD

Anti-gameability heuristic re-audited at `edb2b8c`. The layered defense has not changed.
No-action ruling upheld. Candidate passed to formal-hardening agent for Kani-level
borrow-count proof consideration.

## Re-Verified Axes

Pass 6 re-derived the following axes independently against `edb2b8c`:

1. **take/restore vs Decision-5 both-directions** — carry semantics unchanged from
   prior passes; `mem::take`/restore confirmed symmetric at both `client_hs_carry` and
   `server_hs_carry` independently. Decision-5 drain path confirmed for both directions.

2. **MAX_BUF boundary** — oversized-record rejection boundary confirmed correct;
   `client_hs_carry` and `server_hs_carry` cleared independently on per-direction
   oversize (consistent with Architecture Mapping corrected in v1.4).

3. **Cursor arithmetic** — per-direction cursor arithmetic confirmed; no cross-direction
   cursor aliasing introduced by the dual-field restructure.

4. **Bench methodology** — benchmark at `edb2b8c` uses `tls_carry_*` slug convention
   (consistent with pass-3 nitpick remediation); methodology consistent with pipeline.rs
   convention.

5. **Fixture determinism** — shared fixture helper produces identical byte-for-byte output
   across repeated invocations; `edb2b8c` did not change the fixture helper.

6. **Sibling sweeps clean** — grep sweep across implementation, test files, and bench
   files confirms no stale `TlsFlowState.carry` references (Architecture Mapping v1.4
   correction fully propagated); no stale `mem::replace swap pattern` comment residuals.

## Findings Detail

### LOW (NITPICK)

**F-S149P6-001** — Test module docstring missing 5th enforced invariant

The docstring for the `bc_149_budget_annotation_tests` test module enumerated 4 enforced
invariants:

1. BORROW BUDGET doc-paragraph header is present.
2. Inline marker count equals the declared budget.
3. Each expected call-site function name is listed in the doc paragraph.
4. Doc-paragraph region is bounded and distinct from body-site region.

The test in fact enforces a 5th invariant that was absent from the docstring:

5. The doc-paragraph marker line is excluded from the body-site grep region (the region
   delimiter logic is tested by a boundary sub-assertion).

The missing invariant does not affect test correctness — the test enforces it regardless
of whether the docstring describes it. However, a future maintainer reading only the
docstring would have an incomplete picture of what the test verifies, creating a
maintenance hazard if the region-delimiter logic were ever refactored.

Doc-only fix required. No code change to test logic.

Remediated in: `2418048` (doc-only; test module docstring updated to enumerate 5th
invariant). No code changes — does NOT void the clean pass.

## Orchestrator-Verified Gates at `edb2b8c`

| Gate | Result |
|------|--------|
| `cargo test --all-targets` | 2367 pass / 0 fail |
| `cargo clippy --all-targets -- -D warnings` | CLEAN |
| `cargo fmt --check` | CLEAN |

## Convergence State

Pass 6 complete. Classification: NITPICK_ONLY. Zero code findings.

Clean streak: **1 / 3** required. Not yet converged. Pass 7 pending.

Worktree head: `2418048` (doc-only resync after nitpick remediation; no code change from
`edb2b8c`). Story spec: v1.4 (`d68af34`).
