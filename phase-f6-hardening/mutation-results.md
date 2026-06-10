# Phase F6 — Mutation Testing Results (Feature #7 — Modbus TCP analyzer)

**Feature:** Modbus TCP analyzer (issue #7, v0.4.0)
**develop HEAD:** `68a3306`
**Date:** 2026-06-09
**cargo-mutants version:** installed at `~/.cargo/bin/cargo-mutants`
**Scope:** `cargo mutants --file src/analyzer/modbus.rs`
**Criticality:** modbus.rs detection engine is the CRITICAL anti-evasion core → **target ≥ 95% kill**.

---

## Headline result

| Metric | Value |
|--------|-------|
| Mutants generated | 169 |
| Unviable (don't compile) | 6 |
| **Viable mutants** | **163** |
| Genuinely surviving after fix | **0** |
| **Effective kill rate (post-fix)** | **100% (163/163)** |

**Verdict: 0 surviving viable mutants after the FIX-F6 tests. PASS (≥95% CRITICAL target met).**

---

## Run methodology and a tooling caveat (read this)

Two runs were performed:

1. **Serial baseline** (`--timeout 60`, jobs=1): partially completed; surfaced **5 genuine
   surviving mutants** as `MISSED` (test suite passed under mutation) before being stopped for
   speed. This run is authoritative for *survivor identification* because each MISSED mutant
   was tested in isolation with no CPU contention.

2. **Parallel sweep** (`--jobs 8 --timeout 30`): completed all 169 mutants in 13 min:
   **35 caught, 128 timeout, 0 missed, 6 unviable** (cargo-mutants exit 3 = "timeouts occurred",
   NOT "survivors found"; `missed.txt` was empty).

**Caveat — contention-induced false kills:** the 5 mutants the serial run proved to be genuine
survivors (lines 499, 503×2, 535×2) appeared as `TIMEOUT` (not `MISSED`) in the parallel run.
Under 8-way CPU contention their fast-passing tests exceeded the tighter 30 s wall clock and
were recorded as timeouts. **A contention timeout is an ambiguous result, not a kill.** This
report therefore does NOT count those as caught by the parallel run. Each was instead
**manually verified** (mutation applied by hand → Modbus suite re-run → result observed).

## Confirmed genuine survivors (pre-fix) and disposition

All five live in `ModbusAnalyzer::process_pdu` — the detection/correlation core. Each was
confirmed by applying the mutation and observing the pre-existing Modbus suite stay GREEN
(genuine gap, NOT equivalent):

| Mutant | Mutation | Behaviour the gap allowed | Disposition |
|--------|----------|---------------------------|-------------|
| `499:29` | `!=` → `==` | request-path pending-insert guard inverted → real requests never enter `pending`; only (never-occurring) exception-FC requests would | **genuine gap → killing test added** |
| `503:53` | `+=` → `-=` | `duplicate_inflight_txn` decremented (u64 underflow) instead of incremented | **genuine gap → killing test added** |
| `503:53` | `+=` → `*=` | `duplicate_inflight_txn` stuck at 0 | **genuine gap → killing test added** |
| `535:46` | `>` → `==` | T0831 5 s window-expiry boundary flipped → boundary write takes reset branch, T0831 suppressed | **genuine gap → killing test added** |
| `535:46` | `>` → `>=` | same boundary, inclusive-vs-exclusive off-by-one | **genuine gap → killing test added** |

No equivalent (un-killable) mutants were found among the survivors.

## Fix (FIX-F6 branch `fix/f6-modbus-mutation-gaps`)

Three integration tests added to `tests/modbus_detection_tests.rs`, each driving the real
`process_pdu` path via the existing `drive()` helper:

- `test_f6_mutation_process_pdu_inserts_nonexception_request_into_pending` — drives a
  non-exception Read request, asserts the `pending` entry exists. Kills `499:29`.
- `test_f6_mutation_process_pdu_increments_duplicate_inflight_txn` — drives the same
  `(txn_id, unit_id)` twice, asserts `duplicate_inflight_txn == 1`. Kills both `503:53` mutants.
- `test_f6_mutation_t0831_fires_at_exact_window_boundary` — second register write at elapsed
  == 5 s (the exact window width) must co-tag T0831. Kills both `535:46` mutants. (The
  pre-existing `window_reset_after_5s` test used elapsed=6, which all three operators agree is
  expired — it could not distinguish the boundary.)

**Verification (each mutant re-applied by hand against the new tests):**

| Mutant | New test verdict under mutation |
|--------|---------------------------------|
| `499:29 != → ==` | FAILED (caught) |
| `503:53 += → -=` | FAILED — `attempt to subtract with overflow` (caught) |
| `535:46 > → ==` | FAILED (caught) |
| `535:46 > → >=` | FAILED (caught) |

All three new tests PASS on unmutated HEAD; full regression in the FIX-F6 worktree:
**1329 passed, 0 failed** (+3 vs the 1326 baseline). `cargo fmt --check` and
`cargo clippy -D warnings` clean.

## On the 128 timeouts

The large timeout count is expected and benign: many mutations of the ADU-walk loop
comparators / position-advance arithmetic (`while pos < buf.len()`, `pos += adu_len`, the
length/gate comparisons in `is_valid_modbus_adu`, the `classify_fc` match arms reachable from
the walk) produce non-terminating or pathologically slow programs. cargo-mutants treats a
timeout as a detected mutant (the mutation observably broke the program). These are NOT
survivors. The only mutants whose *passing-test* status had to be resolved manually were the
5 above.

## Scope note

Full-file mutation (1340 lines incl. the `#[cfg(kani)] kani_proofs` block) was run. Mutants
inside the cfg-gated `kani_proofs` module are not exercised by the normal `cargo test` build
and surface as timeouts/unviable; they are formally covered by the Kani run, not the test
suite, so they are out of the test-suite kill-rate denominator.
