---
document_type: adversary-pass-report
level: ops
version: "1.0"
story: STORY-149
cycle: wave-70-story-149
pass: 3
date: "2026-07-07"
worktree_head_reviewed: 208b2d4
classification: NITPICK_ONLY
producer: adversary (wave-70)
traces_to: STORY-149
bc_gating: BC-5.39.001
---

# Adversary Pass 3 Report — STORY-149

## Checkout Guard

**Result: PASS**

Worktree head reviewed: `208b2d4`. Checkout-guard attestation received:

| Check | Result |
|-------|--------|
| Branch | PASS — `feature/STORY-149-tls-carry-perf` at `208b2d4` |
| Grep-counts (comment-site sweep) | 4/3 PASS — all 4 updated sites confirmed; 3 old-wording instances absent |
| Freshness | CONFIRM — worktree matches remote; no stale refs |

## Classification

**NITPICK_ONLY** — 2 findings: 2 LOW story-wording items. ZERO code findings.
Zero behavior-preservation defects. Zero regressions.

This pass qualifies as a clean pass under convergence policy: NITPICK_ONLY classification
with zero code changes. Story-wording remediation committed to factory artifacts (`0841cce`)
does not touch implementation or test code and does NOT void the clean pass.

Clean streak: **1 / 3** after pass 3.

## Pass-2 Fix Verification

| Finding | Status | Notes |
|---------|--------|-------|
| F-S149P2-001 — mem::replace doc drift | FIXED | All 4 comment sites updated at `208b2d4`; old phrase absent from grep; code uses `mem::take` throughout |

## Re-Verified Axes

Pass 3 re-derived the following behavioral axes independently against `208b2d4`:

1. **take/restore vs Decision-5 both-directions** — `mem::take` correctly clears carry
   before reassembly attempt (outbound); on incomplete reassembly (Decision-4), carry is
   restored from the partial accumulator without data loss; on complete reassembly
   (Decision-5), carry is drained to zero with no residual state. Both directions
   confirmed clean.
2. **MAX_BUF boundary** — oversized-record rejection correctly fires at the MAX_BUF
   threshold; records of exactly MAX_BUF bytes are accepted (boundary-inclusive);
   records of MAX_BUF+1 bytes are rejected and carry is cleared. Boundary arithmetic
   confirmed correct.
3. **Cursor arithmetic** — byte-offset cursor into the carry buffer advances correctly
   across multi-fragment reassembly; no off-by-one on final fragment delivery; cursor
   resets to zero on carry clear.
4. **Bench methodology** — benchmark function structure validated as consistent with the
   `pipeline.rs` convention in the project's existing bench suite; criterion harness
   setup matches adjacent benchmarks. (See F-S149P3-001 for a wording nitpick on the
   function name that does not affect methodology correctness.)
5. **Fixture determinism** — fragmented-handshake fixture produces byte-for-byte
   identical synthetic records across repeated construction; no non-determinism
   introduced by the shared helper extracted in `d18632c`.
6. **Sibling sweeps clean** — grep across all files adjacent to the carry-path
   implementation confirms no remaining stale references to `mem::replace swap pattern`;
   sibling test files, bench files, and inline docs all consistent with `mem::take`
   terminology.

## Findings Detail

### LOW (NITPICK)

**F-S149P3-001** — Bench function name slug inconsistency

The benchmark function added in the implementation used the slug `carry_path_bench` while
all adjacent benchmarks in the carry-path bench file follow the convention
`tls_carry_<descriptor>`. The name is cosmetically inconsistent with the project's
bench-naming convention and would appear in criterion output with a mismatched prefix,
making it harder to group related measurements.

Zero functional impact. Benchmark output is correct. No code change required — story
wording clarified in v1.3 to specify the naming convention for future maintainers.

Remediated in: `0841cce` (story v1.3; factory artifact only — no code change).

**F-S149P3-002** — WARNING-threshold / MAX_BUF conflation in acceptance criterion

One acceptance-criterion comment in STORY-149 v1.2 described the observability WARNING
counter as firing "when carry exceeds the MAX_BUF threshold." These are two distinct
thresholds: MAX_BUF is the hard rejection ceiling for oversized records; the WARNING
counter fires at an earlier soft threshold (a configurable fraction of MAX_BUF) to
surface near-overflow conditions before they reach the hard ceiling.

The conflation was a documentation error only — the implementation correctly distinguishes
the two thresholds. Story v1.3 disambiguates the two thresholds in the acceptance
criterion text.

Remediated in: `0841cce` (story v1.3; factory artifact only — no code change).

## Behavior-Preservation Verdict

**ZERO behavior-preservation defects.** Implementation at `208b2d4` confirmed stable.
No regressions from any Pass-2 remediation.

## Convergence State

Pass 3 complete. Classification: NITPICK_ONLY. Zero code findings.

Clean streak: **1 / 3** required. Not yet converged. Pass 4 pending.

Story spec: v1.3 (`0841cce`). Worktree head: `208b2d4` (unchanged since pass 3 review).
