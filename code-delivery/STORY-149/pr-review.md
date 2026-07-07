# PR #374 — Fresh-eyes Review (Cycle 1)

**Verdict:** APPROVE — no blocking findings.

**Scope reviewed:** full diff `develop...feature/STORY-149-tls-carry-perf`
(11 files, +1339/-393). Focus areas per task brief:
single-borrow invariant, `std::mem::take` carry swap, test enforcement quality,
benchmark fixture, and spec fidelity against AC-149-001..005.

---

## What I verified

### Correctness of the single-borrow restructure
- `try_parse_records` (src/analyzer/tls.rs:995–1031) acquires exactly one
  `self.flows.get_mut(flow_key)` per loop iteration and passes the resulting
  `&mut TlsFlowState` into the associated function `prepare_record_step`.
  Because `prepare_record_step` has no `self` receiver, it *cannot* re-hash
  the map — this makes the SINGLE-BORROW INVARIANT a compile-time guarantee,
  not just a comment claim. Solid.
- `prepare_record_step` (src/analyzer/tls.rs:787–847) does all
  buf-drain / carry-extend / `mem::take` work inside that one borrow,
  and returns owned data (`RecordStep::Handshake { carry, last_ts }`)
  so the outer `&mut self` dispatch is unencumbered.

### Correctness of the `std::mem::take` swap
Traced semantic equivalence against the pre-refactor per-direction loops:
- Empty-carry hot path: `mem::take` returns an empty Vec (allocation moved
  out and replaced with `Vec::new()`); after the drain loop, the (still-empty
  after `carry.drain(..consumed)`) local is assigned back — equivalent to
  the old in-place `state.client_hs_carry.drain(..consumed)` and preserves
  the allocation across take/restore.
- Decision-4 body-len spoof: `carry.clear()` on the local, drain skipped,
  then `state.client_hs_carry = carry` (empty). Equivalent to the old
  `state.client_hs_carry.clear()` + skipped drain.
- Decision-5 overflow-before-append: guard runs before `mem::take`; carry
  is cleared in-place on the still-borrowed state, buf drained,
  `RecordStep::CarryOverflow` returned. Equivalent.
- Fragmented multi-record accumulation (3×15-byte fixture): traced —
  record 1 leaves 15 bytes in state carry after restore, record 2 leaves
  30, record 3 completes the 45-byte ClientHello and drains. Matches
  `test_BC_149_002_carry_drain_loop_exercised_across_records`.

Flow-eviction concern (comment at src/analyzer/tls.rs:973–976): I confirmed
`flows.remove` only appears in `on_flow_close`
(src/analyzer/tls.rs:1104), which is never called during `on_data`.
`handle_client_hello` / `handle_server_hello` do not touch `self.flows`
(the `_flow_key` parameter is intentionally unused). So the assumption
holds — but see NIT #2 below.

### Test enforcement quality (AC-149-001)
5 source-inspection tests in `tests/bc_149_single_borrow_invariant_tests.rs`:
- `exactly_one_flows_borrow_in_try_parse_records` — asserts `total == 1`
  (not `<= 1`), guarding against accidental body hollowing.
- `single_borrow_invariant_comment_marker_present` — enforces the inline
  marker survives future diffs.
- `process_handshake_carry_borrow_budget` — asserts helper ≤ 3 AND
  grand total ≤ 4.
- `process_handshake_carry_budget_annotations_match_sites` — annotation
  count equals site count (both directions of drift caught).
- `no_aliasing_patterns_hide_borrow_count` — guards against
  `= &mut self.flows`, `= &self.flows`, `.entry(`, `.iter_mut(`.

`extract_fn_body` uses brace-depth counting; the limitation
(braces in string literals / comments are counted) is documented in the
helper doc. Reviewed both function bodies — no unbalanced braces in
string literals, so the extraction is sound for this codebase.

### Benchmark fixture (AC-149-002)
`benches/tls_fragmented.rs` + `tests/common/tls_fragmented_fixture.rs`:
- The 45-byte ClientHello split 15/15/15 correctly forces the carry-drain
  loop to enter twice with incomplete body_len (breaks at `carry.len() -
  consumed < 4 + body_len`) before the third fragment completes it.
- The `include!` sharing between the test file and the bench file
  eliminates duplication drift.
- `test_BC_149_002_carry_drain_loop_exercised_across_records` positively
  verifies both the partial (carry non-empty after N-1 segments) and
  completed (handshake_count == 1 after N) states — this is a stronger
  test than a simple "invoke the code path" smoke test.

### Local build gates
- `cargo test --test bc_149_single_borrow_invariant_tests` — 5/5 pass.
- `cargo test --test bc_149_fragmented_fixture_tests` — 3/3 pass.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo fmt --check` — clean.
- (A pre-existing `test_drop_without_finalize_does_not_panic` shared-
  atomic pollution surfaces under parallel test execution but reproduces
  on develop and is unrelated to this PR.)

---

## Findings

No BLOCKING findings. Four non-blocking observations:

### NIT-1: "site N of ≤3" wording could confuse future readers
**File:** `src/analyzer/tls.rs:924, 948, 977`
The `BORROW BUDGET (STORY-149): site 1 of ≤3 — flag-set (client_hello_seen)`
markers refer to *source-inspection* sites (all three counted by the CI
grep), not runtime call sites (only one of the flag-set sites fires per
invocation because the direction match selects one arm). Consider a
one-line clarification, e.g. `site 1 of ≤3 (source-count; runtime hits
1-of-2 flag-set sites)`. Non-blocking.

### NIT-2: Silent flow-eviction assumption in carry-restore
**File:** `src/analyzer/tls.rs:973–984`
The comment states carry is dropped silently if the flow was evicted
between `prepare_record_step` and the restore. Today this cannot happen,
but a future change to `handle_client_hello`/`handle_server_hello`
(or a new dispatch site) could silently break carry accumulation for
fragmented handshakes. A cheap guard would be:

```rust
if let Some(state) = self.flows.get_mut(flow_key) {
    match direction { ... }
} else {
    debug_assert!(false, "STORY-149: flow evicted between prepare_record_step and carry-restore");
}
```

Cost is zero in release builds; catches the invariant violation early
in debug/test. Non-blocking.

### SUGGESTION-1: Anti-gameability check misses `self.flows[key]` (Index syntax)
**File:** `tests/bc_149_single_borrow_invariant_tests.rs:235–273`
The security review's SEC-002 already flags this as a deferred follow-up.
Since `self.flows[flow_key]` (via `Index::index`) currently does not appear
anywhere in the file, adding one more assertion (`!body.contains("self.flows[")`)
would close the gap now at essentially zero cost. Non-blocking (deferred
per SEC-002).

### SUGGESTION-2: Benchmark includes `TlsAnalyzer::new()` per iteration
**File:** `benches/tls_fragmented.rs:46–52`
The Criterion iteration constructs a fresh analyzer every loop, so the
1.594 µs baseline includes analyzer setup overhead in addition to the
carry-drain path. For pure carry-drain isolation, `iter_batched` with a
setup closure (using `BatchSize::SmallInput`) would exclude construction
cost. Regression detection is still valid at the current baseline; this
is a precision improvement, not a correctness issue. Non-blocking.

---

## Spec fidelity check

| AC | Wording | Implementation |
|----|---------|----------------|
| AC-149-001 | Bounded-borrow budget ≤ 4 | Verified: 1 (try_parse_records) + 3 (process_handshake_carry) = 4. |
| AC-149-002 | Fragmented benchmark, ≥ 3 records | Verified: 3-record fixture, drain loop exercises 2+ times. |
| AC-149-003 | Perf within +5% of May-19 anchor | Not independently re-measured; trust PR-declared 23.841 µs (< 24.445 µs ceiling). |
| AC-149-005 | No regressions | 2367 tests reported PASS; local `cargo test --all-targets` matches (modulo pre-existing shared-atomic pollution unrelated to this PR). |

Implementation matches AC wording. No spec drift.

---

## Diff coherence & description accuracy

- All changes trace to STORY-149. No unrelated diffs.
- Diff size (+1339/-393) is above the 500-line "flag" threshold but the
  large numbers are dominated by (a) rewritten `tls.rs` (net loss of
  duplication between two direction arms) and (b) demo-evidence text
  files. The actual behavior-changing surface is ~200 lines of
  `try_parse_records` restructure plus the two helper functions.
- PR description accurately reflects the diff (helper split, `mem::take`
  rationale, borrow budget, benchmark fixture).
- Conventional commit format and story ID present on every commit.
- Demo evidence: 4 AC files + evidence-report.md present under
  `docs/demo-evidence/STORY-149/`. Files are `.txt`, not `.gif`/`.webm`
  — acceptable for a perf/refactor story where the observable outcomes
  are numeric (benchmark output, test-count deltas), not visual.

---

## Verdict

**APPROVE.** The refactor preserves all documented invariants, the tests
genuinely enforce what they claim, and the benchmark fixture faithfully
exercises the carry-drain path. Findings above are non-blocking polish
items; none of them block merge.
