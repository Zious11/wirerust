# ADR 0004: Process-Wide Atomics for One-Shot Bug-Tripwire Warnings

**Status:** Accepted
**Date:** 2026-05-19
**Context:** This is a retroactive ADR. The reassembly engine accumulated three
`static AtomicBool` one-shot warning guards over several PRs (the
`close_flow`-on-missing-key guard, the `ISN`-missing guard, and the
`finalize`-not-called `Drop` tripwire added for LESSON-P0.03). The
brownfield-ingest Phase C synthesis flagged that this recurring pattern was
never written down (the "ADR 0004 for process-wide atomics" P3 lesson). This
ADR codifies the existing decision so future contributors apply it
consistently rather than re-deriving it.

## Problem

Several places in the reassembly engine detect a **programming error** — a
control-flow bug that should never happen if the engine is driven correctly:

- `close_flow` called for a `FlowKey` not present in the flow table
  (`src/reassembly/lifecycle.rs`).
- A segment insert reached with no ISN set for the direction
  (`src/reassembly/segment.rs`).
- A `TcpReassembler` dropped without `finalize()` having been called
  (`src/reassembly/mod.rs`, the LESSON-P0.03 `Drop` tripwire).

Each wants to emit a diagnostic to `stderr` so the bug is *loud* rather than
silent. But a naive `eprintln!` at each site has two failure modes:

1. **Flooding.** These sites sit on per-packet / per-flow paths. A bug that
   trips one of them typically trips it on *every* packet or *every* flow —
   thousands of identical lines that bury the signal and slow the run.
2. **Per-instance repetition.** wirerust creates many `TcpReassembler`
   instances within a single process — every integration test, and every
   multi-file analysis run, constructs fresh engines. A warning latch stored
   as a per-instance `bool` field would re-warn once *per instance*, so a
   test suite would print the same bug warning dozens of times.

The warning needs to fire **once per process**: enough to alert a developer,
never enough to flood.

## Decision

**One-shot bug-tripwire warnings use a process-wide `static AtomicBool`
guard, flipped with `AtomicBool::swap(true, Ordering::Relaxed)`.**

The canonical shape:

```rust
static SOME_BUG_WARNED: AtomicBool = AtomicBool::new(false);

// at the detection site:
if !SOME_BUG_WARNED.swap(true, Ordering::Relaxed) {
    eprintln!("wirerust: <description of the bug and its consequence>");
}
```

`swap` returns the *previous* value, so the very first caller sees `false`
and prints; every subsequent caller — in this instance or any other, on any
thread — sees `true` and stays silent.

This applies specifically to **programming-error tripwires**. It does NOT
apply to normal operational counters or to per-capture findings (those are
`ReassemblyStats` fields and `Finding`s respectively, and are per-instance by
design).

## Alternatives Considered

### Per-instance `bool` field on `TcpReassembler`

Rejected. It re-warns once per engine instance. A bug class is a property of
the *code*, not of one engine; seeing the warning once anywhere is the
signal. Per-instance state turns "once" into "once per test", which is the
flooding problem in a slower disguise. It also cannot serve the `Drop`
tripwire cleanly (see below) or the free-function ISN site in `segment.rs`,
which has no `self`.

### A logging framework (`log` + `env_logger`, `tracing`, etc.)

Rejected. wirerust deliberately carries a small dependency set, and these
warnings are rare bug tripwires, not an observability surface. A one-line
`eprintln!` gated by an atomic needs no crate. If wirerust later grows a
real logging story, these sites can migrate then.

### `std::sync::Once`

Workable — `Once::call_once` gives exactly-once semantics. Rejected as
slightly heavier at the call site: it takes a closure and is conceptually a
lazy-initialization primitive, whereas `AtomicBool::swap` keeps the call site
a plain `if` that reads naturally as "warn unless we already did". Both are
correct; the atomic is the simpler fit for a boolean latch.

## Rationale

- **`static`, not per-instance** — the latch tracks a *bug class*, and a bug
  class is process-global. This is the crux of the decision.
- **`swap` for the check-and-set** — a single atomic operation does
  test-and-latch with no separate load/store race window. The first caller
  is unambiguously the one that observed `false`.
- **`Ordering::Relaxed`** — the flag is a best-effort de-duplication hint,
  not a synchronization point. It guards no other memory. The only
  observable cost of the weakest ordering is that, under a genuine data race
  between two threads hitting the site for the first time simultaneously,
  the warning could print twice. Two identical lines instead of one is
  harmless; paying for `SeqCst` to prevent it would be cargo-culting.
- **Works without `&mut self`** — `swap` takes `&self` on the `static`, so
  the pattern works in `Drop::drop` (which the LESSON-P0.03 tripwire needs)
  and in free functions (the `segment.rs` ISN site), neither of which can
  rely on a mutable engine borrow.

## Consequences

- **Tests assert behavior, not warning text.** Because the latch is
  process-wide, a test cannot reliably assert "this run warned" — an earlier
  test in the same process may have already flipped the flag. Tests for
  these sites therefore assert the *real* contract (no panic, correct
  control flow, correct counters) and treat the `stderr` line as a
  developer-facing side effect only. The LESSON-P0.03 `Drop` tests are
  written exactly this way (`test_drop_without_finalize_does_not_panic`).
- **The warning is genuinely once-per-process.** Re-running an analysis in a
  long-lived process (e.g. a future daemon mode) would not re-warn for a
  recurring bug after the first occurrence. This is acceptable: the warning's
  job is to surface the bug to a developer once, not to provide ongoing
  telemetry. A recurring operational signal would be a counter, not a
  tripwire.
- **New tripwires must follow the pattern.** Any future "this should never
  happen" diagnostic on a hot path uses a new `static AtomicBool` named
  `<SUBJECT>_WARNED`, flipped with `swap(true, Ordering::Relaxed)`. Three
  guards currently follow it: `CLOSE_FLOW_MISSING_WARNED`,
  `ISN_MISSING_WARNED`, `FINALIZE_SKIPPED_WARNED`.

## Validation

- All three existing guards (`grep -rn '_WARNED' src/`) use the identical
  `static AtomicBool` + `swap(true, Relaxed)` shape.
- The behavior is exercised indirectly by the reassembly engine tests, which
  pass without warning floods, and directly by the LESSON-P0.03 `Drop` tests,
  which confirm the un-finalized path does not panic.

## Amendments

### 2026-05-25 — STORY-014 / BC-2.04.048 v1.3: test-seam exception for `ISN_MISSING_WARNED`

STORY-014 added two `#[doc(hidden)] pub fn` test seams in
`src/reassembly/segment.rs`:

- `pub fn isn_missing_warned_for_testing() -> bool` — reads `ISN_MISSING_WARNED`.
- `pub fn reset_isn_missing_warned_for_testing()` — stores `false` to `ISN_MISSING_WARNED`.

These accessors are formalized in BC-2.04.048 v1.3 PC4 and exist solely to
allow STORY-014's combined AC-013/AC-014/EC-007 integration test to
deterministically observe the BC-2.04.048 PC1 `false → true` swap
transition. The `reset_isn_missing_warned_for_testing` call at the top
of the combined test eliminates the original ADR guidance's "an earlier
test in the same process may have already flipped the flag" concern by
forcing a known precondition for the first observation.

**Hygiene constraints (mandatory):**

- Both functions carry `#[doc(hidden)]` to keep them out of public
  `cargo doc` output despite being on the `pub` API. They are `pub`
  (rather than `#[cfg(test)]`) because integration tests are separate
  crates and cannot see `#[cfg(test)]` items.
- Both function names end with the `_for_testing` suffix as a flag to
  readers of the public API.
- Neither function may be called from production code paths. Any future
  call site outside `tests/` is a defect.

**Scope of the exception:** the test-seam exception applies ONLY to
`ISN_MISSING_WARNED`. Sibling guards `CLOSE_FLOW_MISSING_WARNED`
(`src/reassembly/lifecycle.rs:31`) and `FINALIZE_SKIPPED_WARNED`
(`src/reassembly/mod.rs:70`) do NOT have test seams as of Wave 7, and
continue to follow the original ADR-0004 guidance: tests for those sites
assert the real contract (no panic, correct control flow, correct
counters) and treat the `stderr` line as a developer-facing side effect
only. Test seams will be added for those guards ONLY when a future BC
introduces an AC that requires deterministic observation of the
sibling's `false → true` swap transition. This asymmetry is
intentional — the seam is opt-in per-guard, gated by BC-driven need.

**Validation lemma refinement:** the original Validation section's
`grep -rn '_WARNED' src/` invariant ("all three guards use the
identical `static AtomicBool + swap(true, Relaxed)` shape") continues
to apply to the GUARD SITES themselves. The `_for_testing` accessor
functions introduced for `ISN_MISSING_WARNED` are read/reset wrappers,
not guard sites; they are expected to appear as additional matches and
do not violate the canonical-shape claim about the guards.

### 2026-05-26 — STORY-019 / BC-2.04.029 v1.4: test-seam expansion to CLOSE_FLOW_MISSING_WARNED + new state-injection seam class

STORY-019 (Wave 8) added three `#[doc(hidden)] pub fn` test seams for
`CLOSE_FLOW_MISSING_WARNED` in `src/reassembly/lifecycle.rs`:

- `close_flow_missing_warned_for_testing() -> bool` (lifecycle.rs:158-161) — reads the atomic.
- `reset_close_flow_missing_warned_for_testing()` (lifecycle.rs:167-170) — stores false.
- `trigger_close_flow_missing_key_for_testing(...)` (lifecycle.rs:196-217) — **replicate-body design**: directly executes the post-debug_assert body of the missing-key branch (atomic swap + one-shot eprintln). It does NOT call production `close_flow` because BC-2.04.029 PC6 defines a `debug_assert!(false, ...)` at lifecycle.rs:43 that fires in cargo's default debug-test profile BEFORE the swap at lifecycle.rs:44, making the post-swap atomic state unobservable via any `catch_unwind` wrapping of `close_flow`. The replicate-body design preserves PC6 in production code (debug_assert untouched) while allowing tests to deterministically observe BC-2.04.029 PC4/PC5 atomic-state behavior.

These three seams are formalized in BC-2.04.029 v1.4 PC7 and exercised by the STORY-019 combined AC-013+AC-014 test (`test_BC_2_04_029_close_flow_missing_key_warns_once`) and AC-015 test (`test_BC_2_04_029_close_flow_missing_key_does_not_modify_state`).

**Hygiene constraints** (same as the 2026-05-25 amendment):

- All three carry `#[doc(hidden)]`.
- All names end with the `_for_testing` suffix.
- None may be called from production code paths.

**New seam class: state-injection (`force_set_flow_state_for_testing`)**

STORY-019 also added a fourth `#[doc(hidden)] pub fn` at `src/reassembly/lifecycle.rs:232-244`:

```rust
pub fn force_set_flow_state_for_testing(
    reassembler: &mut TcpReassembler,
    key: &FlowKey,
    state: FlowState,
) -> bool
```

This seam is **NOT a warning-guard test accessor** — it directly mutates a flow's state via a `pub(crate) fn flows_mut(&mut self)` accessor on `TcpReassembler`. It exists for STORY-019 AC-012, which needs to construct a flow in `FlowState::Closed` with `last_seen` well within the timeout window to discriminate BC-2.04.013 PC1's state-based OR-clause from the time-based clause. Without this seam, the only way to reach `FlowState::Closed` is via two-FIN sequence (which advances `last_seen`). The seam allows the test to isolate the state-based clause.

The `force_set_flow_state_for_testing` seam represents a **new test-seam class** authorized under the original "opt-in per-guard" doctrine: state-injection seams enable BC-driven discrimination tests that cannot be expressed through the normal API. Same hygiene constraints apply (`#[doc(hidden)]`, `_for_testing` suffix, no production callers).

**Scope of the exception (updated as of Wave 8)**

The Wave-7 amendment's "Scope of the exception" lemma is now updated:

- `ISN_MISSING_WARNED` — has read + reset seams (STORY-014 / BC-2.04.048 v1.3 PC4).
- `CLOSE_FLOW_MISSING_WARNED` — has read + reset + trigger seams (STORY-019 / BC-2.04.029 v1.4 PC7).
- `FINALIZE_SKIPPED_WARNED` (`src/reassembly/mod.rs:70`) — has NO test seams **as of Wave 8**; continues to follow the original ADR-0004 guidance (assert behavior, not warning text).

The opt-in-per-guard, gated-by-BC-driven-need doctrine continues to apply. Future seams for `FINALIZE_SKIPPED_WARNED` are authorized when a BC introduces an AC requiring deterministic observation of its swap transition or its post-warning state.

**Validation lemma refinement (extended)**

- `grep -rn '_WARNED' src/` will match both guard sites AND the `_for_testing` accessor wrappers for both `ISN_MISSING_WARNED` and `CLOSE_FLOW_MISSING_WARNED`.
- `grep -rn '_for_testing' src/reassembly/` now matches six function signatures (2 ISN seams from STORY-014, 3 CLOSE_FLOW seams from STORY-019, plus 1 force_set_flow_state state-injection seam = 6 total).
- The canonical-shape claim ("static AtomicBool + swap(true, Relaxed)") still applies to the GUARD SITES themselves (lifecycle.rs:43-48 and segment.rs:51-58), not to the `_for_testing` wrappers or to the `trigger_close_flow_missing_key_for_testing` replicate-body (which mirrors the guard-site pattern internally).

**Module visibility widening**

STORY-019 widened `mod lifecycle;` (private) to `pub mod lifecycle;` to expose the `_for_testing` accessors to integration tests. This is necessary because integration tests are separate crates and cannot see `#[cfg(test)]` items. The visibility widening is recorded here for SemVer auditability — `flow_count()`, `flows_mut()`, and the four `_for_testing` accessors are now reachable via `wirerust::reassembly::lifecycle::*`.
