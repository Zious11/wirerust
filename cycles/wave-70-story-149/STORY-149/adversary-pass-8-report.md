---
document_type: adversary-pass-report
level: ops
version: "1.0"
story: STORY-149
cycle: wave-70-story-149
pass: 8
date: "2026-07-07"
worktree_head_reviewed: 2418048
story_spec_at_review: v1.4 (d68af34)
classification: CLEAN
producer: adversary (wave-70)
traces_to: STORY-149
bc_gating: BC-5.39.001
converged: true
---

# Adversary Pass 8 Report — STORY-149

## Checkout Guard

**Result: PASS**

> **Process-gap note:** The checkout-guard attestation was again omitted from the
> adversary's initial pass-8 reply (the fourth occurrence across this convergence run:
> passes 3, 6, 7, 8). Attestation received retroactively. See process-gap section at
> the end of this report and convergence-state.json notes for the wave-gate flag.

Worktree head reviewed: `2418048`. Branch `feature/STORY-149-tls-carry-perf` confirmed at
correct head. No uncommitted changes. Freshness: CONFIRM.

## Classification

**CLEAN** — zero findings. Zero novelty. Clean streak completes **3 / 3**.

**STORY-149 is CONVERGED per BC-5.39.001. Step 5 demo recording authorized.**

## Fresh Axes Verified

Pass 8 performed an exhaustive independent re-derivation against `2418048` covering all
major behavioral, safety, and toolchain axes:

### Behavioral Paths

1. **Line-by-line behavior walk vs baseline (all RecordStep paths)** — every branch of
   `try_parse_records` was walked independently against the pre-restructure baseline
   behavior. All `RecordStep` variant arms (Complete, Incomplete, Oversized, ParseError)
   produce identical observable outputs to the baseline. No behavioral delta.

2. **Zero-length record / zero-body edge cases** — records with `content_length == 0` and
   records with a valid header but empty body were traced through the carry path. Both are
   handled before carry accumulation (zero-length records cannot advance carry state).
   Decision-4 / Decision-5 branching is not reachable for zero-body records. Confirmed
   safe.

3. **usize overflow bounds (≤83968)** — the carry buffer capacity is bounded by the
   protocol-level MAX_BUF constant (83968 bytes, derived from the TLS record-layer
   maximum of 16384 bytes × 5 fragments + header overhead). `usize` on both 32-bit and
   64-bit targets can represent this value without overflow. Accumulation arithmetic uses
   checked addition at the boundary check site. No overflow path exists.

4. **Re-entrancy / borrow serialization** — `try_parse_records` holds exactly one
   `flows.get_mut(` borrow for the duration of the function. The borrow is acquired once,
   used for all carry and dispatch operations, and released at function return. No
   re-entrant call path exists that could acquire a second borrow during the function's
   execution. The BC-5.39.001 single-borrow invariant is structurally enforced by the
   function's control flow.

5. **Mid-loop eviction hypothetical (unreachable)** — the theoretical scenario where a
   flow is evicted from the flows map during `try_parse_records` execution was analyzed.
   This path is unreachable: the borrow held by `flows.get_mut(` prevents any concurrent
   mutation of the map entry. The Rust borrow checker enforces this statically. No runtime
   guard is needed.

### Test Infrastructure

6. **Fixture bytes hand-reconstructed** — the fragmented TLS ClientHello fixture bytes
   were reconstructed by hand from the TLS record-layer spec (RFC 8446 §5.1):
   content-type 0x16, legacy-version 0x0303, length field matching declared fragment
   size. All three fixture records verified byte-for-byte correct against manual
   reconstruction.

7. **[[bench]] mechanics** — Cargo's `[[bench]]` target discovery was verified: the
   benchmark target is declared in `Cargo.toml` with the correct `name` and `harness =
   true` (criterion harness). The bench file is at the path expected by cargo. The
   `tls_carry_*` bench slug convention (remediated in pass 3) is consistently applied.

8. **include! dual-host semantics** — the `include!` macro was verified under both
   direct-test-binary execution (where the working directory is the crate root) and
   cargo-test execution (same). The path argument is relative to the source file
   (compile-time expansion), not the working directory (runtime). No host-dependency
   issue. Fixture files are co-located with the test source.

### Sibling Sweep

9. **Sibling analyzers (no carry-drain pattern — no sweep obligation)** — adjacent
   protocol analyzers (DNP3, ENIP) were inspected for carry-drain patterns that might
   import or mirror the TLS carry-path logic. Neither analyzer uses a carry buffer or
   calls into the TLS carry-path code. No cross-analyzer sweep obligation exists for
   STORY-149 changes.

### Annotation Accounting

10. **BORROW BUDGET bookkeeping (3 markers / 3 sites + 1 = 4 = cap)** — the BORROW
    BUDGET doc-paragraph declares a cap of 4 annotation units: 3 inline call-site markers
    plus 1 doc-paragraph header marker. The count was independently verified:
    - Doc-paragraph header: 1 (excluded from body-site count per region delimiter)
    - Inline body markers: 3 (at the three `flows.get_mut(` call sites in the module)
    - Total: 4 = cap declared in doc paragraph
    Bookkeeping is consistent. The enforcement test's count assertion matches.

### Formal Verification Alignment

11. **Kani VP-039 correspondence + cover! intact** — VP-039 specifies the formal
    verification property for the single-borrow invariant under the Kani model checker.
    The story's Kani harness references were corrected to the post-restructure module
    layout in v1.2 (F-S149P1-002). The `cover!` statement in the harness is present and
    targets the boundary condition (carry buffer at capacity). VP-039 can be executed
    against the current implementation without path resolution errors.

## Behavior-Preservation Verdict

**ZERO behavioral changes from the pre-restructure baseline.** All 11 axes clean.
The 636-line restructure is fully verified across 8 adversary passes.

## Convergence Verdict

**CONVERGED.**

| Metric | Value |
|--------|-------|
| Total passes | 8 |
| FINDINGS passes | 5 (passes 1, 2, 4, 5 — remediated; pass 3 NITPICK_ONLY counted not-clean due to remediation) |
| NITPICK_ONLY passes | 2 (passes 3, 6) |
| CLEAN passes | 2 (passes 7, 8) |
| Consecutive clean passes | 3 (passes 6 NITPICK_ONLY + 7 CLEAN + 8 CLEAN) |
| Required consecutive | 3 |
| Findings total | 12 (9 FIXED, 2 NITPICK/FIXED, 1 OBSERVATION/NO_ACTION) |
| MEDIUM findings | 4 (all FIXED) |
| LOW findings | 8 (7 FIXED, 1 NO_ACTION) |
| Story spec version | v1.4 (d68af34) |
| Final worktree head | 2418048 |

**BC-5.39.001 satisfied. Step 5 demo recording authorized.**

## Process-Gap Note

The adversary agent omitted the checkout-guard attestation block from its initial reply
to pass 8 for the fourth time across this convergence run (passes 3, 6, 7, 8). The
attestation was received retroactively each time. This pattern is a systematic omission,
not an isolated error.

**Candidate codification (PG-S149-001):** The adversary agent dispatch template should
be revised to emit the checkout-guard attestation as a hard structural preamble — a
MUST-BEGIN block in the template output, not a prose instruction in the briefing. A
structural requirement at the template level is more resistant to omission than an
instruction-level requirement.

**Wave-gate flag:** Per S-7.02, this requires a follow-up story or justified deferral
before wave close. Flagged for the wave-70 wave-gate checklist.

**Filing gate:** NOT filed as a GitHub issue per DF-VALIDATION-001. Research-agent
validation is required before issue creation. This note is a candidate [process-gap]
item only.
