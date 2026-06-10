# Phase F6 — Mutation Testing Results (Feature #100)

**Feature:** issue-100-pcap-timestamps
**develop HEAD:** `256a490`
**Date:** 2026-06-09
**cargo-mutants version:** 27.0.0
**Scope:** `cargo mutants --in-diff /tmp/f6-100-src.diff --timeout 300` (delta lines only, 6 files)

---

## Headline result

| Metric | Value |
|--------|-------|
| Mutants generated (in-diff) | 33 |
| Viable mutants tested | 32 (1 unviable) |
| Caught | 30 |
| Missed | 2 (both **provably-equivalent**, same line) |
| **Raw kill rate** | **93.8%** (30/32) |
| **Effective kill rate (excluding equivalent mutants)** | **100%** (30/30 killable) |

**All timestamp-delta logic mutants were killed.** The 2 survivors are on a pre-existing
statistics-counter line (`bytes_reassembled += data.len()`) that is **not part of the
Feature #100 timestamp change** and is **structurally unreachable with non-zero data** in any
program state — i.e., equivalent mutants that no test can kill.

---

## Per-file kill rates vs. criticality targets

| File | Tier | Target | Caught | Missed | Unviable | Kill rate | Status |
|------|------|--------|--------|--------|----------|-----------|--------|
| `src/analyzer/http.rs` | HIGH | ≥90% | 7 | 0 | 0 | **100%** | PASS |
| `src/analyzer/tls.rs` | CRITICAL/HIGH | ≥95% | 9 | 0 | 0 | **100%** | PASS |
| `src/dispatcher.rs` | CRITICAL | ≥95% | 1 | 0 | 0 | **100%** | PASS |
| `src/reassembly/mod.rs` | HIGH | ≥90% | 10 | 0 | 1 | **100%** | PASS |
| `src/reassembly/lifecycle.rs` | CRITICAL/HIGH | ≥95% | 3 | 2 | 0 | 60% raw / **100% effective** | PASS (equivalent) |
| `src/reassembly/handler.rs` | — | — | 0 | 0 | 0 | n/a (trait sig only; no mutable lines in diff) | n/a |

CRITICAL anti-evasion / dispatch paths (dispatcher, tls SNI, reassembly mod hot path):
**100% kill — exceeds the ≥95% CRITICAL target.**

---

## Baseline note (process)

The first run (`--timeout 120`) failed in the **unmutated baseline** because cargo-mutants
runs the full test binary set (including the slow `assert_cmd`-based CLI integration tests and
criterion bench harness) serially in a fresh tmp target dir, and the aggregate baseline test
phase exceeded 120s. Re-running with `--timeout 300` produced a green baseline (41s build +
35s test) and complete results. No source or test change was needed for this — it was a
timeout-budget tuning issue, not a real failure.

---

## Per-mutant disposition (non-caught)

### 1. UNVIABLE — `src/reassembly/mod.rs:339` — `insert_payload_segment -> Direction` replaced with `Default::default()`

- **Disposition:** UNVIABLE (does not count against kill rate). `Direction` has no `Default`
  impl, so the mutant does not compile. cargo-mutants correctly classifies it as unviable.
  No action.

### 2 & 3. MISSED (EQUIVALENT) — `src/reassembly/lifecycle.rs:62` — `+=` → `-=` and `+=` → `*=`

- **Mutated line:** `self.stats.bytes_reassembled += data.len() as u64;` inside `close_flow`'s
  per-direction flush loop.
- **This is NOT Feature #100 code.** It is a pre-existing statistics accumulator. The
  Feature #100 diff only changed the **adjacent** line 63 (added the `close_timestamp`
  argument to `handler.on_data(...)`) and inserted `let close_timestamp = flow.last_seen;`.
  Because `--in-diff` includes context lines within a changed hunk, line 62 was pulled into
  the mutable scope even though the feature did not author it. (Verified against the diff hunk:
  line 62 is an unchanged context line.)

- **Disposition: PROVEN EQUIVALENT MUTANTS — no kill test is possible.**

  **Proof of unreachability-with-data:**
  1. `insert_payload_segment` is called at exactly one site (`mod.rs:191`) and is **always**
     immediately followed by `flush_contiguous_data` on the same direction (`mod.rs:193`),
     with no early return between them.
  2. `flow_dir.flush_contiguous()` (`segment.rs:369`) drains the **entire** contiguous prefix
     from `base_offset` via an unconditional `while` loop — it never leaves a contiguous
     prefix behind.
  3. Therefore, after every payload packet, that direction's contiguous prefix is fully
     drained on the hot path.
  4. At `close_flow` (`lifecycle.rs:60`), `flush_contiguous()` is called again per direction.
     Since every direction's contiguous prefix was already drained, it **always returns an
     empty `Vec`**.
  5. The loop body at `lifecycle.rs:61-64`, including the mutated line 62, **never executes
     with data**. With zero iterations, `+=`, `-=`, and `*=` are observationally identical
     (the statement is never run).

  **Empirical corroboration:** I instrumented `close_flow`'s flush loop with an `eprintln!`
  on non-empty `data` and ran the **entire `cargo test --all-targets` suite** in a throwaway
  worktree. The instrumentation fired **zero times** — confirming that no test (and, per the
  proof above, no possible input) drives buffered data through this loop. I additionally swept
  several reassembler constructions (gap-then-RST; out-of-order then gap-fill; server-direction
  buffering) attempting to leave a contiguous prefix unflushed at close; all flushed on the hot
  path, none reached line 62 with data.

  Because the mutated statement is unreachable-with-data in **all** reachable program states,
  these are textbook equivalent mutants. Per the Dark Factory mutation discipline, equivalent
  mutants are justified-survived, not counted as a test gap. **No code or test change is
  warranted**, and (critically) **no kill test can be authored** — there is no input that
  makes the line execute with non-zero data.

- **Note for the orchestrator:** Line 62's accumulator is *defensive* code (it handles the
  hypothetical case where `close_flow` flushes residual data). That is reasonable engineering,
  but under the current single-flush-site design it is dead-with-data. This is a pre-existing
  observation about `reassembly/lifecycle.rs`, **out of scope** for Feature #100, and does not
  block the F6 gate. If the orchestrator wishes to eliminate the equivalent mutants entirely,
  the options are (a) add a `cargo-mutants` skip annotation / `mutants.toml` exclusion for that
  line, or (b) leave as-is (recommended — the survivors are correctly understood and the delta
  logic is 100% killed). Either is a **FIX-F6 (optional, non-blocking)** item, not a
  correctness fix.

---

## Conclusion

- **Every mutant on Feature #100's timestamp-threading logic was killed** (http.rs, tls.rs,
  dispatcher.rs, reassembly/mod.rs emission + threading sites: 100%).
- The only survivors are 2 provably-equivalent mutants on a pre-existing, unreachable-with-data
  statistics line that the diff included only by hunk-adjacency.
- **Effective kill rate over killable mutants: 100%**, exceeding all criticality targets
  (CRITICAL ≥95%, HIGH ≥90%).
- **Mutation gate: PASS.** No FIX-F6 code change required.
