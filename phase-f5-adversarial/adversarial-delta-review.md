# Phase F5 Adversarial Delta Review — Feature #100 (pcap timestamp → Finding.timestamp)

## Metadata

| Field | Value |
|-------|-------|
| Scope delta | `afe93a1^..48cbc05` |
| develop HEAD | `48cbc05` |
| Reviewer | vsdd-factory:adversary (Claude, fresh context) |
| Date | 2026-06-08 |
| Stories | STORY-097, STORY-098, STORY-099 |

---

## Findings

### CRITICAL

None.

---

### HIGH

#### ADV-F5-HIGH-001 — Spec-Fidelity: BC-2.09.007 canonical test vector maps ts_sec=1_000_000 to wrong human-readable date

**Severity:** HIGH
**Category:** spec-fidelity
**Status:** Never-deferrable

BC-2.09.007's canonical test vector maps `ts_sec=1_000_000` to the human-readable date `2001-09-08T21:46:40Z`. This is WRONG. 1_000_000 seconds after the Unix epoch is `1970-01-12T13:46:40Z`. The date `2001-09-08T21:46:40Z` corresponds to `ts_sec=1_000_000_000` (one billion, not one million).

**Locations (blast radius — 6 files):**

- `.factory/specs/behavioral-contracts/BC-2.09.007.md` lines 105, 113, 114
- `.factory/specs/behavioral-contracts/BC-2.09.006.md` lines 64, 77, 86 (sibling propagation)
- `.factory/stories/STORY-098.md` line 67
- `.factory/stories/STORY-099.md` lines 67, 137, 140
- `.factory/feature-delta/issue-100-pcap-timestamps/delta-analysis.md` line 354

**Implementation and tests are CORRECT.** The actual VP-021 tests at `tests/timestamp_threading_tests.rs:117-119, 781-782` correctly use `1970-01-12T13:46:40Z`. `reassembly_engine_tests.rs:16982` correctly asserts `1970-01-12` for the 1M case and uses `1_000_000_000` for the 2001-era JSON test. The defect is entirely in the spec/story corpus — the code and tests accurately reflect reality.

**Carry-over verdict:** ts_sec=1M wrong-date CONFIRMED spec defect (carry-over item #2 resolved as REAL).

**Policy:** DF-SIBLING-SWEEP-001 requires a one-burst fix across all 6 files — no partial commits.

**Fix:** Correct all spec/story text so `ts_sec=1_000_000` maps to `1970-01-12T13:46:40Z`. If a 2001-era example is desired, add a distinct vector with `ts_sec=1_000_000_000` → `2001-09-08T21:46:40Z`. After correcting BC-2.09.007 content, recompute input-hashes for stories that list it as an input (STORY-098, STORY-099) via `bin/compute-input-hash --write .factory/stories/STORY-098.md .factory/stories/STORY-099.md`.

---

### MEDIUM

#### ADV-F5-MED-001 — Spec-Fidelity: STORY-098 emission-site count "4" should be "3"

**Severity:** MEDIUM
**Category:** spec-fidelity

`STORY-098.md:58-59` (AC-003) and `:136` (Task 9) state "4 anomaly finding-emission sites in `src/reassembly/mod.rs`." The actual `Some(…)`-emitting count in `mod.rs` is **3**:

- `mod.rs:493` — overlap anomaly
- `mod.rs:533` — small-segment anomaly
- `mod.rs:559` — out-of-window anomaly

Total reassembly Some sites = 3 (mod.rs) + 2 (lifecycle.rs:125, lifecycle.rs:155) = 5, plus 1 None site (`mod.rs:673` segment-limit summary). The BC-2.09.007 "21 of 22" invariant is itself correct (9 http + 7 tls + 3 mod + 2 lifecycle = 21 Some, +1 None = 22). Only STORY-098's "4" claim is wrong.

**Fix:** Change "4" to "3" in AC-003 and Task 9 in STORY-098.md. Body-only change; does not affect the BC input content.

---

#### ADV-F5-MED-002 — Spec/Test Divergence: STORY-099 AC-002 / Task 4 describe a test that does not exist as described

**Severity:** MEDIUM
**Category:** spec/test divergence

`STORY-099.md:59` (AC-002) and `:138` (Task 4) describe a close-flush test using `ts_sec=2_000_000`, asserting the finding carries `Some(from_timestamp(2_000_000, 0))`. The implemented `test_finding_timestamp_close_flush` (`tests/timestamp_threading_tests.rs:383-665`) uses `TS_DATA=1_500_000` / `TS_FIN=1_500_100` and **never asserts a close-flush finding timestamp** — because contiguous-only flush drains all data on the hot-path, leaving the close buffer empty (test comments lines 483-491). The PR description also repeats the false `2_000_000` claim.

**Fix:** Rewrite AC-002 and Task 4 to accurately describe what is actually verified: `on_flow_close` fires once; `flow.last_seen` is read at `lifecycle.rs:56`; close-flush `on_data` is unreachable with residual data under contiguous-only flush, so timestamp correctness is established by code inspection of `lifecycle.rs:56,63`. Align cited `ts` values to the actual `TS_DATA=1_500_000` / `TS_FIN=1_500_100` used in the test.

---

### LOW

#### ADV-F5-LOW-001 — Coverage Gap (NOT a defect): close-flush flow.last_seen timestamp unverified at runtime

**Severity:** LOW
**Category:** coverage-gap

The close-flush `flow.last_seen` timestamp (`lifecycle.rs:56, 63`) is never verified by a runtime observable — only by code inspection. This is carry-over item #1.

**Verdict: REAL + ACCEPTABLE.** Traced all paths to `close_flow` → `flush_contiguous`: the hot-path flush drains contiguous bytes after every payload packet; a permanent gap is unreachable by `flush_contiguous`; RST/timeout close paths have drained/non-contiguous buffers. Timestamp on this path is correct (`flow.last_seen` read before `flows.remove`). A crafted capture cannot reach it with a wrong or None timestamp. The residual risk is that a future non-contiguous flush mode would have no regression test.

**Optional fix:** Add a `#[doc(hidden)]` test seam that injects a contiguous-at-close segment to make the path directly observable at runtime.

---

#### ADV-F5-LOW-002 — Test Quality: STORY-098 emission-site tests assert only `timestamp.is_some()`

**Severity:** LOW
**Category:** test-quality

The STORY-098 emission-site tests at `reassembly_engine_tests.rs:16465, 16506, 16680` and `tls_analyzer_tests.rs:1115, 1693` assert only `timestamp.is_some()`. A hardcoded `Some(epoch)` bug would pass these assertions. The assertions are non-vacuous (they verify findings are non-empty first) but weak on value-binding.

**Mitigation:** Substantially mitigated by the STORY-099 exact-value tests at `timestamp_threading_tests.rs:201, 312`, the AC-005 all-u32 proptest at `:949`, and `dispatcher_tests.rs:1568, 1601`, which do bind exact timestamp values.

**Optional fix:** Strengthen the 3 `is_some()` checks in reassembly_engine_tests.rs and 2 in tls_analyzer_tests.rs to `assert_eq!` with the exact expected timestamp value.

---

#### ADV-F5-LOW-003 — Coverage Gap: VP-021 cross-flow isolation test uses separate reassemblers

**Severity:** LOW
**Category:** coverage-gap

`prop_cross_flow_timestamp_isolation` (`timestamp_threading_tests.rs:1039-1056`) drives flow B through a **separate** reassembler/analyzer instance, so it cannot detect within-single-analyzer cross-flow contamination — the real VP-014/BC-2.09.007-inv-4 risk. That risk is covered by STORY-098's `test_cross_flow_timestamp_isolation` (`reassembly_engine_tests.rs:17060`, same analyzer), but that test asserts only that the timestamp set is `{ts_a, ts_b}`, not that the finding from flow-A carries `ts_a` specifically (a swap would still pass).

**Optional fix:** Feed both flows into one analyzer and bind each finding to its source flow via evidence (e.g., `flow-a.com` / `flow-b.com` Host header) to catch cross-flow timestamp swaps.

---

### OBSERVATIONS

#### ADV-F5-OBS-001 — Process Gap: ts_sec=1_000_000 ↔ 2001 mispairing survived multiple review layers

**Type:** process-gap

The `ts_sec=1_000_000` ↔ `2001-09-08` mispairing survived BC authoring, two story authorings, and per-PR reviews. This is the cross-story integration-gap class the combined F5 pass is designed to catch.

**Recommendation:** Consider a lightweight lint that evaluates `ts_sec=N` ↔ ISO-8601 pairings in BC canonical-vector tables (assert `DateTime::from_timestamp(N, 0)` matches the recorded human-readable string). Candidate for a new policy axis under DF-SIBLING-SWEEP-001.

---

### Clean Observations

- **Convention adherence:** Clean. Underscore-prefixed unused params, uniform `from_timestamp` pattern, per-flow `FlowKey` keying.
- **Security:** Clean. `from_timestamp` is total over all `u32` values — no overflow, panic, or timestamp leakage path.
- **Regression:** Clean. All 5 `on_data` implementors updated. Error paths and cap paths intact. Kani harnesses untouched.

---

### Carry-Over Item Verdicts

| # | Carry-over | Verdict |
|---|------------|---------|
| 1 | close-flush `flow.last_seen` unverified at runtime | REAL + ACCEPTABLE |
| 2 | `ts_sec=1_000_000` mapped to 2001 date in spec | REAL — CONFIRMED spec defect (ADV-F5-HIGH-001) |
| 3 | 1 omitted-of-22 site (`mod.rs:673` segment-limit summary) | CORRECT — `timestamp: None` BY DESIGN per BC-2.09.007 PC4/inv-1, verified by `test_segment_limit_summary_timestamp_is_none_and_absent_from_json` |

---

## Severity Summary

| Severity | Count | IDs |
|----------|-------|-----|
| CRITICAL | 0 | — |
| HIGH | 1 | ADV-F5-HIGH-001 |
| MEDIUM | 2 | ADV-F5-MED-001, ADV-F5-MED-002 |
| LOW | 3 | ADV-F5-LOW-001, ADV-F5-LOW-002, ADV-F5-LOW-003 |
| OBSERVATION | 4 | ADV-F5-OBS-001 (1 process-gap) + 3 clean observations |

## Verdict

**NOT-CONVERGED** — blocked on ADV-F5-HIGH-001 (mis-anchored canonical test vector in spec corpus). Implementation code is sound. Spec corpus requires a one-burst 6-file fix sweep before re-pass.
