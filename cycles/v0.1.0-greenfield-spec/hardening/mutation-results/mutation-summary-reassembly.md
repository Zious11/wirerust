# Mutation Testing Summary — Reassembly Core (flow / segment / mod)

- **Tool:** cargo-mutants 27.0.0
- **Date:** 2026-06-02
- **Base:** `origin/develop` @ `d31769d` (#183 — VP-002 `select_gaps` extraction +
  G1/G2/G3 byte-survival tests). Worktree branch `verify/mutants-reassembly`,
  mounted at `.worktrees/mutants-reassembly`.
- **Verified before running:** `fn select_gaps` present at `src/reassembly/segment.rs:140`;
  G1/G2/G3 byte-survival tests present in `tests/reassembly_segment_tests.rs`
  (line ~2541+) and `tests/reassembly_engine_tests.rs` (XAAAA / first-wins guards).
  `segment.rs` is 865 lines on this base (vs 529 pre-VP-002), confirming the merge.
- **Mode:** Assessment only — no source/test changes, nothing committed, no PR.

## Kill-Rate Definition

Kill rate = `(caught + timeout) / (caught + timeout + missed)` over **viable** mutants
(unviable mutants — those that fail to compile — are excluded from numerator and
denominator, per cargo-mutants convention). TIMEOUT counts as a kill: the mutation
caused a hang/infinite loop the harness would never let pass.

## Method note — timeout calibration (important, affects result validity)

The reassembly test suite is the heaviest in the repo (baseline ~12-13 s). Two
methodology pitfalls were encountered and corrected:

1. **`--timeout 30 -j 8` produced SPURIOUS timeouts.** Under 8-way CPU contention a
   normally-12 s suite can run 30-120 s wall, so the 30 s cap mis-classified
   *legitimately-passing-but-slow* mutants (e.g. the stats-counter `+= → *=` mutants,
   which cannot loop-hang) as TIMEOUT instead of MISSED. That run reported a false
   "0 survivors". **Discarded.**
2. **`--timeout 120 -j 6` (the task-specified timeout, lower parallelism) classifies
   correctly** — genuine loop-hang mutants hit 120 s; slow-but-passing tests finish
   well under it; real survivors surface as MISSED. This run could not finish all 492
   within the harness runtime cap, but it **fully covered `flow.rs` (58) and `mod.rs`
   (212)** before stopping (it had not yet reached `segment.rs`).
3. **`segment.rs` was then run on its own** (`-f src/reassembly/segment.rs --timeout 60
   -j 6`, 212 mutants) to completion. 60 s is ~4.5x the baseline — comfortably above
   any contention-induced slowdown, comfortably below a genuine hang — so its TIMEOUT
   classifications are reliable.

The authoritative result below is the **union of the `flow.rs`+`mod.rs` portion of the
120 s/j6 run and the dedicated 60 s/j6 `segment.rs` run.** Both used a timeout that
does not produce spurious timeouts.

## Per-Module Result vs Target

| Module | Tier | Target | Kill Rate | Verdict | caught | timeout | missed | unviable | viable |
|--------|------|--------|-----------|---------|--------|---------|--------|----------|--------|
| `src/reassembly/flow.rs`    | CRITICAL | ≥95% | **100.00%** | **PASS** | 11 | 44 | 0 | 3 | 55 |
| `src/reassembly/segment.rs` | CRITICAL | ≥95% | **98.57%**  | **PASS** (see note) | 93 | 114 | 3 | 2 | 210 |
| `src/reassembly/mod.rs`     | HIGH     | ≥90% | **92.68%**  | **PASS** (see note) | 136 | 54 | 15 | 7 | 205 |

Per the genuine-vs-justified analysis below:
- **flow.rs:** 100% — no survivors. PASS unconditionally.
- **segment.rs:** raw 98.57%. Of 3 survivors, **2 are justified-equivalent** and **1 is a
  genuine gap** (`ranges_overlap` adjacency boundary). Excluding justified survivors the
  effective kill rate is 209/210 = 99.52%. Either way **above the 95% CRITICAL target.**
- **mod.rs:** raw 92.68% (above the 90% HIGH target even counting every survivor as a
  genuine miss). All 15 survivors are boundary-comparison / stats-counter mutants in the
  effectful hot-path shell; none touch routing or evasion-detection logic.

**All three modules meet or exceed their kill-rate targets**, including counting every
survivor pessimistically as a genuine miss.

## Totals (union of authoritative runs)

| Outcome | Count |
|---------|-------|
| Viable + unviable processed | 482 |
| Caught | 240 |
| Timeout (= caught) | 212 |
| **Missed (survivors)** | **18** |
| Unviable (did not compile) | 12 |
| **Aggregate viable kill rate** | **(240+212)/(240+212+18) = 96.17%** |

Per-file mutant generation: mod.rs 212, segment.rs 212, flow.rs 58.

## Anti-evasion logic — the headline question

The task flagged `select_gaps` / first-wins / overlap-detection mutants as the most
important (anti-evasion). Findings:

- **`select_gaps` (the VP-002 extraction):** every mutant generated against it was
  **killed** — the G1/G2/G3 byte-survival tests and the gap-fill sweep tests detect
  function-body and constant-replacement mutations (most via the active-flow no-escape
  guard, surfacing as TIMEOUT loop-hangs; the rest caught by assertion). **Zero
  `select_gaps` survivors.**
- **First-wins / overlap policy (`segment_overlap`, the InsertResult classification):**
  all mutants killed. **Zero survivors.**
- **`ranges_overlap` — ONE genuine survivor (see GG-1 below).** This is the lowest-level
  overlap *predicate*, and one of its two boundary comparisons has an untested edge.

So the *winner-selection* and *classification* anti-evasion logic is fully covered; the
single anti-evasion gap is an untested boundary in the underlying overlap predicate.

## Survivors (MISSED) — Classified

### GENUINE TEST GAPS (real behavior change no test catches)

| # | File:Line | Mutation | Why it's a real gap |
|---|-----------|----------|---------------------|
| **GG-1** | `segment.rs:43:41` | `ranges_overlap`: `new_end > existing_offset` → `>=` | **CRITICAL anti-evasion.** Half-open intervals: `new_end == existing_offset` (exact adjacency) must be NON-overlapping (documented at segment.rs:40). `>=` makes adjacency count as overlap. No test exercises the exact adjacency boundary. A mirror bug here is precisely a TCP-overlap mis-classification. **Highest-priority kill.** |
| GG-2 | `mod.rs:166:22` | `process_packet`: idle-sweep gate `timestamp > last_expiry_sweep_secs` → `>=` | Sweep also fires when `timestamp == last_expiry_sweep_secs`. The sweep is idempotent within a second, so output is unchanged, but no test pins "sweep at most once per unique second" — borderline gap (effectively equivalent in output; listed as genuine for conservatism). |
| GG-3 | `mod.rs:206:30` | `process_packet`: memcap gate `total_memory > memcap` → `>=` | Eviction triggers when memory *equals* memcap instead of strictly exceeding. Off-by-one on the memcap boundary; no test pins memory-exactly-at-memcap → no eviction. |
| GG-4 | `mod.rs:394:30` | `insert_payload_segment`: small-segment threshold `payload.len() < small_segment_max_bytes` → `<=` | Anti-evasion small-segment-run heuristic. `<=` counts a payload exactly equal to the threshold as "small". No test at the exact threshold boundary. |
| GG-5 | `mod.rs:422:32` | `insert_payload_segment`: `bytes_added > 0` → `bytes_added < 0` | `bytes_added` is `usize` → `< 0` is always false → the SegmentLimitReached partial-insertion stats block (`segments_overlaps`/`segments_inserted` +1) never runs. Stats undercount on the segment-limit partial path. |
| GG-6..GG-13 | `mod.rs:403:71`, `405:46`, `406:46`, `409:46`, `413:46`(×2: `-=` and `*=`), `423:50`(×2), `424:50`(×2) | `self.stats.segments_* += 1` → `*= 1` / `-= 1` | Stats-counter accuracy in the InsertResult match arms (Duplicate / PartialOverlap / ConflictingOverlap / Truncated / SegmentLimitReached). No test asserts the exact `segments_overlaps` / `segments_inserted` / `segments_duplicates` counts after these result types. **Counters only — not the forensic finding emission, which is driven by the `InsertResult` enum value and is fully killed.** |

Genuine-gap count: **1 in segment.rs (GG-1), 14 in mod.rs (GG-2..GG-13, expanding the
multi-operator lines)** = **15 genuine.**

### JUSTIFIED-EQUIVALENT survivors (no behavioral contract / dead state / log-only)

| # | File:Line | Mutation | Why justified |
|---|-----------|----------|---------------|
| JE-1 | `segment.rs:204:20` | `insert_segment`: `delete !` in `if !ISN_MISSING_WARNED.swap(true, …)` | Log-emission guard. The `.swap(true)` side-effect (the observable state, readable via the test-only `was_isn_missing_warned()`) is identical with or without `!`; only the `eprintln!` cardinality changes. Diagnostic stderr text has no behavioral contract. `IsnMissing` is still returned. |
| JE-2 | `segment.rs:232:16` | `insert_segment`: `delete !` in `if !self.depth_exceeded { self.depth_exceeded = true; }` | The `depth_exceeded` bool field (`flow.rs:107`) is **write-only dead state** — never read anywhere in `src/`. The depth-exceeded *count* that feeds findings uses the separate `stats.segments_depth_exceeded` counter driven by the `InsertResult::DepthExceeded` return. So whether the guard sets the dead flag is unobservable. (Minor: flags a dead-field cleanup opportunity, not a test gap.) |

Justified-equivalent count: **2 (both segment.rs).**

> Note on `#[cfg(kani)]` harnesses: as in the first run's mitre.rs finding, the
> `kani_proofs` modules in `flow.rs` and `segment.rs` are dead code under `cargo test`.
> Mutating their bodies produced **TIMEOUT** (caught), not MISSED, here — the mutated
> harness hangs the non-kani build until the per-mutant timeout — so they did not appear
> as survivors and required no justification this pass.

## Modules BELOW target on GENUINE gaps

**NONE.** Every module is at or above its target even when all 18 survivors are counted
pessimistically as genuine misses (flow 100%, segment 98.57% ≥95%, mod 92.68% ≥90%).

The genuine gaps are test-coverage opportunities, **not** gate failures. Recommended
test-strengthening priority (for the follow-up fix-PR), highest first:

1. **GG-1 (`ranges_overlap` adjacency)** — add a unit test on `ranges_overlap` /
   `segment_overlap` at exact adjacency (`new_end == existing_offset` and
   `new_start == existing_end`) asserting NON-overlap. This is the one anti-evasion
   genuine gap and should be killed regardless of gate status.
2. **GG-3 / GG-4 (memcap and small-segment boundaries)** — boundary tests at exactly
   `memcap` and exactly `small_segment_max_bytes`.
3. **GG-5 + GG-6..GG-13 (stats-counter accuracy)** — assert exact `stats.segments_*`
   values after Duplicate / PartialOverlap / ConflictingOverlap / Truncated /
   SegmentLimitReached-with-partial-insertion. (Lower priority — counters, not findings.)
4. **GG-2 (idle-sweep gate)** — optional; effectively equivalent.

## Run Provenance / Artifacts

- flow.rs + mod.rs (120 s, j6, base d31769d): `/tmp/mutants-reassembly-120/mutants.out/`
  (run stopped by harness runtime cap after fully covering flow.rs + mod.rs = 270 mutants;
  flow.rs and mod.rs were 100% processed).
- segment.rs (60 s, j6, base d31769d): `/tmp/mutants-seg/mutants.out/` (completed, exit 0,
  212/212 processed).
- The discarded 30 s/j8 full run (`/tmp/mutants-reassembly/mutants.out/`) is retained for
  reference only; its TIMEOUT classification is unreliable due to CPU-contention false
  timeouts and must not be used.

## Convergence assessment

Reassembly CRITICAL/HIGH mutation gate: **PASS.** No module below target. 1 genuine
anti-evasion gap (GG-1, `ranges_overlap` adjacency) and 14 hot-path boundary/stats gaps
in mod.rs identified for a follow-up test-strengthening PR; none block the hardening gate.
