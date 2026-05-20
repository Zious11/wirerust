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
