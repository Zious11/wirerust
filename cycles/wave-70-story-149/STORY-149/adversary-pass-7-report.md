---
document_type: adversary-pass-report
level: ops
version: "1.0"
story: STORY-149
cycle: wave-70-story-149
pass: 7
date: "2026-07-07"
worktree_head_reviewed: 2418048
story_spec_at_review: v1.4 (d68af34)
classification: CLEAN
producer: adversary (wave-70)
traces_to: STORY-149
bc_gating: BC-5.39.001
---

# Adversary Pass 7 Report — STORY-149

## Checkout Guard

**Result: PASS**

> **Process-gap note:** The checkout-guard attestation was omitted from the adversary's
> initial pass-7 reply and was received retroactively upon request. This is the third
> consecutive pass (passes 3, 6, 7) where the attestation required a follow-up prompt.
> See notes section for candidate codification and wave-gate flag.

Worktree head reviewed: `2418048`. Branch `feature/STORY-149-tls-carry-perf` confirmed at
correct head. No uncommitted changes. Grep-counts: 5/3 PASS. Freshness: CONFIRM.

## Classification

**CLEAN** — zero findings. Clean streak advances to **2 / 3**.

## Pass-6 Fix Verification

| Finding | Status | Verification detail |
|---------|--------|---------------------|
| F-S149P6-001 — test module docstring missing 5th invariant | FIXED | Docstring↔test 1:1 mapping confirmed: no over-claim (docstring does not assert invariants the test does not enforce) and no under-claim (test does not enforce invariants absent from docstring). All 5 invariants enumerated and matched. |

## Fresh Axes Verified

Pass 7 re-derived the following axes independently against `2418048`:

1. **Sibling-marker consistency** — `// BORROW BUDGET:` markers in the carry-path
   implementation are consistent with the doc-paragraph header; all three inline markers
   reference the same budget cap stated in the paragraph. No orphaned markers in sibling
   files. Consistency confirmed across the full `src/` subtree touched by STORY-149.

2. **Anti-gameability scope boundary** — the out-of-scope `flows` sites were enumerated:
   two call sites to `flows.get_mut(` exist outside `try_parse_records` (one in the
   dispatching shim, one in the timeout path). Both are correctly excluded from the
   inspection-test grep region by the bounded-scope delimiter. The scope boundary is
   tight and non-gameable by adding sites outside the delimited region.

3. **Fixture split math re-derived** — the fragmented-handshake fixture splits a synthetic
   TLS ClientHello across three records. The byte lengths were independently verified:
   the first two fragments are intentionally undersized (below the declared
   `content_length` in the TLS record header), and the third fragment completes the
   assembly. The split arithmetic is deterministic and correct.

4. **include! path resolution** — the `include!` macro in the test suite resolves at
   compile time to the fixture file path relative to the source file's directory (Rust
   standard behavior). Path is stable across developer workstations and CI. No
   platform-specific path separator issues.

5. **extract_fn_body precondition (no free braces in strings)** — the grep-based
   `extract_fn_body` helper used by the budget-annotation test relies on balanced-brace
   counting to delimit function bodies. This breaks if a string literal contains `{` or
   `}`. The carry-path implementation was scanned: no string literals containing free
   braces are present in the bounded extraction region. Precondition satisfied.

6. **Input-hash canonical** — STORY-149.md `input-hash:` frontmatter field is consistent
   with the canonical algorithm in `bin/compute-input-hash`. The `inputs:` list is
   populated (non-empty); the 7-char MD5 prefix matches a recomputation against the
   declared input files.

## Behavior-Preservation Verdict

No behavioral changes in this pass interval. Implementation at `2418048` is identical to
`edb2b8c` (doc-only delta). All previously verified behavioral paths remain clean.

## Convergence State

Pass 7 complete. Classification: CLEAN. Zero findings.

Clean streak: **2 / 3** required. Not yet converged. Pass 8 pending against `2418048`.

## Process-Gap Note

The adversary agent omitted the checkout-guard attestation block from its initial reply
to this pass dispatch, requiring a follow-up request. This is the third consecutive
occurrence (passes 3, 6, 7). The attestation was received retroactively and is recorded
above. See convergence-state.json `notes.process_gap_observation` for the candidate
codification and wave-gate flag. Per DF-VALIDATION-001, no GitHub issue is filed without
research-agent validation.
