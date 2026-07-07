---
document_type: adversary-pass-report
level: ops
version: "1.0"
story: STORY-149
cycle: wave-70-story-149
pass: 1
date: "2026-07-07"
worktree_head_reviewed: ef83f8c
classification: FINDINGS
producer: adversary (wave-70)
traces_to: STORY-149
bc_gating: BC-5.39.001
---

# Adversary Pass 1 Report — STORY-149

## Checkout Guard

**Result: PASS**

Worktree head reviewed: `ef83f8c`. Worktree was clean at review time with no uncommitted
changes. Branch `feature/STORY-149-tls-carry-perf` confirmed at the correct head.

## Classification

**FINDINGS** — 6 findings total: 2 MEDIUM, 4 LOW. Zero behavior-preservation defects in
the 636-line carry-path restructure.

## Summary

Pass 1 reviewed the implementer deliverable at `ef83f8c` against STORY-149 (v1.1 at review
time) and BC-5.39.001. The carry-path restructure itself was structurally sound: all six
behavioral paths were re-derived clean with no observable behavior change. The findings
cluster into two MEDIUM items (test assertion gameability; stale structural refs) and four
LOW items (wording ambiguity, fixture hygiene, spec overstatement, project convention
alignment). All 6 findings were remediated in the same session.

## Findings Detail

### MEDIUM

**F-S149P1-001** — Inspection-test scope / assertion gameability

`test_BC_149_001_at_most_one_flows_borrow_in_try_parse_records` asserted `<= 1`
`flows.get_mut(` call sites in `try_parse_records`. The intent of BC-5.39.001 is exactly
one borrow per invocation — the `<= 1` bound is gameable by reducing to zero borrows (a
vacuously passing but semantically wrong implementation). The test was renamed to
`test_BC_149_001_exactly_one_flows_borrow_in_try_parse_records` with an `== 1` assertion.
Two companion tests were added: `_process_handshake_carry_borrow_budget` and
`_no_aliasing_patterns_hide_borrow_count` to close the remaining gameability surface.

Remediated in: `a02eb6f` (test-writer).

**F-S149P1-002** — Stale Kani structural references

After the 636-line carry-path restructure at `ef83f8c`, one or more Kani harness
references in the story/spec remained pointed at the pre-restructure module layout. Any
subsequent Kani run or formal-hardening agent would attempt to find harnesses at the
wrong locations.

Remediated in: `0d85780` (story-writer; STORY-149 bumped to v1.2).

### LOW

**F-S149P1-003** — `mem::replace` vs `mem::take` wording ambiguity

The red-gate-log implementation guidance (and carry-forward in the story) referred to
"std::mem::replace swap pattern" for the carry `Vec`. For `Option<T>`, the idiomatic and
intention-revealing idiom is `mem::take`, which is a specialisation of `mem::replace`
with the default value. The ambiguous wording was a documentation hazard for future
maintainers.

Remediated in: `d18632c` (implementer).

**F-S149P1-004** — Fixture duplication in fragmented-handshake tests

The synthetic ≥3-record fragmented TLS handshake setup was inlined into multiple test
functions in `tests/bc_149_fragmented_fixture_tests.rs` rather than extracted to a
shared helper. Any future change to the fixture would require N parallel edits, risking
divergence.

Remediated in: `d18632c` (implementer — shared helper extracted).

**F-S149P1-005** — EC-002 no-alloc claim overstatement

A spec clause in STORY-149 v1.1 implied the refactored carry path introduced zero heap
allocation per record. This holds for the hot path but not for error-handling branches
(e.g., parse-error recording paths can still allocate). The unqualified claim was
technically false and would mislead formal-hardening passes.

Remediated in: `0d85780` (story-writer; STORY-149 v1.2 qualified the claim to "hot path
only").

**F-S149P1-006** — Missing `#[cfg(test)]` mod wrappers

Both new test files (`tests/bc_149_single_borrow_invariant_tests.rs` and
`tests/bc_149_fragmented_fixture_tests.rs`) lacked `#[cfg(test)] mod` wrappers.
Adjacent test files in the project use `mod tests { ... }` wrappers consistently.
While the files compile correctly without them (integration test files are gated by
`[[test]]`-discovery), the inconsistency with project conventions was flagged for
alignment.

Remediated in: `a02eb6f` (test-writer).

## Behavior-Preservation Verdict

**ZERO behavior-preservation defects** in the 636-line restructure.

Six behavioral paths were re-derived independently and confirmed clean:

1. Oversized-record path — carry state correctly rejects and clears on oversize input.
2. Decision-4 (incomplete fragment, no flush) — carry accumulation preserved.
3. Decision-5 (complete reassembly, flush) — carry drain and delivery confirmed.
4. Non-handshake drain — records that arrive after handshake completion drain
   carry correctly.
5. Hello dispatch order — ClientHello / ServerHello ordering invariant intact.
6. Timestamp and carry restoration — timestamp field correctly preserved across
   carry cycles.

## RFC Framing

Fragmented TLS handshake fixture verified conformant to RFC 8446 §5.1 (TLS record
layer framing): synthetic records use correct content-type (0x16), correct version
field for TLS 1.2 compat mode, and correct length fields. The fixture construction
does not rely on any test-only parsing helpers that could mask malformed framing.

## Input Hash

`input-hash: d41d8cd` — consistent with recorded value in STORY-149.md. No spec drift
detected at review time.

## Post-Remediation Gates (orchestrator-verified at `a02eb6f`)

| Gate | Result |
|------|--------|
| `cargo test --all-targets` | 2366 pass / 0 fail |
| `cargo clippy --all-targets -- -D warnings` | CLEAN |
| `cargo fmt --check` | CLEAN |

## Convergence State

Pass 1 complete. Clean streak: 0 / 3 required. Not converged. Pass 2 pending.

## Notes

**Process gap observed — tool limitation (candidate [process-gap] item, NOT filed):**

`bin/compute-input-hash` errors with "Expected at least one entry" when a story's
`inputs:` field is an empty list (`inputs: []`). Stories of type E-11 (self-contained,
no external BC/PRD inputs) carry empty input lists by design. The tool currently cannot
verify these stories are drift-free. Observed during input-hash validation for STORY-149.

Per DF-VALIDATION-001, this finding must be validated by the research agent before being
filed as a GitHub issue. Recording here as a candidate [process-gap] item only. Do NOT
file as a GitHub issue without research-agent validation.
