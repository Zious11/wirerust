---
document_type: maintenance-performance-baseline
sweep: 7-maint-2026-09-05
producer: performance-engineer
created: 2026-06-22
last_updated: 2026-09-05
branch: develop
commit: 0b1ea806
version: v0.13.3
baseline_date: 2026-05-19
baseline_source: .factory/maintenance/performance.md (maint-2026-06-17 recorded values)
prior_sweep_date: 2026-07-08
current_run_date: 2026-09-05
hardware_note: >
  Apple Silicon Mac, darwin 25.5.0. All measurements are wall-clock on the
  benchmark machine. Absolute µs values are not portable across hardware;
  only relative deltas (same machine, same branch) are meaningful for
  regression tracking.
benchmark_command: cargo bench --bench pipeline && cargo bench --bench tls_fragmented
rust_version: stable (rustc 1.98.1, cargo 1.98.1)
criterion_version: "0.8"
samples: 100 per benchmark
---

# Performance Baseline — Running History

---

## maint-2026-09-05 (Sweep 7)

**Commit:** 0b1ea806 (develop, v0.13.3)
**Requested run:** maint-2026-09-05, Maintenance Sweep 5 (Performance Regression Detection).

### Baseline-anchor discrepancy (flagged, not fabricated)

The dispatch instructions for this sweep asked to compare against "the last recorded
baseline ... from prior run maint-2026-07-21." **No `maint-2026-07-21` entry exists in
this file, in `.factory/maintenance/`, or in this file's git history** (`git log
--follow` on this path returns no results beyond the current tracked blob). The most
recent entry actually recorded in this document is **Sweep 6 / maint-2026-07-08**
(commit b642c0f, v0.11.5), which itself carried a **NOISE-SUSPECT** annotation (11–20
severe outliers per 100 samples, wide CIs) and an explicit recommendation not to treat
it as a clean regression comparator. No numbers for a "maint-2026-07-21" run are
fabricated here; both real anchors below (Sweep 6 and the June-22 controlled re-run)
are used instead, and this discrepancy should be raised with whoever scheduled this
sweep.

### Benchmarks found: YES

Two `harness = false` criterion targets registered in `Cargo.toml`:
`benches/pipeline.rs` (groups: `decode`, `summary`, `reassembly`; fixtures
`segmented.pcap`, `tls.pcap`, `dns-remoteshell.pcap`) and `benches/tls_fragmented.rs`
(`tls_fragmented/3-record-carry-drain`). Both were run via `cargo bench` (not
fabricated — full criterion stdout captured for this run).

### Results Table — vs Sweep 6 (2026-07-08, NOISE-SUSPECT anchor) and vs June-22 controlled anchor

| Benchmark | Fixture | Jun-22 controlled anchor (µs) | Jul-08 Sweep 6 (µs, NOISE-SUSPECT) | Today 2026-09-05 (µs) | Today's outliers/100 | vs Jun-22 | vs Jul-08 | Verdict |
|-----------|---------|-------------------------------|-------------------------------------|------------------------|----------------------|-----------|-----------|---------|
| decode | segmented.pcap | 1.459 | 2.0145 | 1.4626 | 0 | +0.25% | -27.4% | NOISE (matches Jun-22 anchor) |
| decode | tls.pcap | 3.369 | 4.3810 | 3.4387 | 6 (1 severe) | +2.07% | -21.5% | NOISE |
| decode | dns-remoteshell.pcap | 4.840 | 5.9408 | 4.7362 | 8 (2 severe) | -2.14% | -20.3% | NOISE |
| summary | segmented.pcap | 0.639 | 0.9003 | 0.6543 | 4 | +2.41% | -27.3% | NOISE |
| summary | dns-remoteshell.pcap | 2.589 | 3.5304 | 2.6342 | 1 | +1.75% | -25.4% | NOISE |
| reassembly | segmented.pcap | 5.858 | 8.8937 | 5.8572 | 4 | -0.01% | -34.1% | NOISE (essentially identical to Jun-22) |
| reassembly | tls.pcap | 24.429 | 26.353 | 22.244 | 5 (1 severe) | -8.86% | -15.6% | NOISE (below 10% threshold either direction) |
| tls_fragmented | 3-record-carry-drain | N/A (no clean anchor — Jul-08 was itself the noisy initial baseline) | 2.0662 | 1.5141 | 5 | N/A | -26.7% | No clean prior anchor; this run establishes the first low-noise baseline |

Criterion's own stored-base verdicts from today's run (vs whatever was previously in
`target/criterion/` on this machine, dated May-19/Jul-07 — not this document's anchors):
- `decode/segmented.pcap`: No change detected (p=0.09)
- `decode/tls.pcap`: Performance has improved (-8.4%, p<0.05)
- `decode/dns-remoteshell.pcap`: Performance has improved (-15.1%, p<0.05)
- `summary/segmented.pcap`: No change detected (p=0.44)
- `summary/dns-remoteshell.pcap`: Performance has regressed (+2.6%, p<0.05) — well under the 10% WARNING threshold, informational only
- `reassembly/segmented.pcap`: Performance has improved (-6.2%, p<0.05)
- `reassembly/tls.pcap`: Performance has improved (-4.3%, p<0.05)
- `tls_fragmented/3-record-carry-drain`: Performance has improved (-11.2%, p<0.05)

### Interpretation

**No regression >10% or >25% found against any anchor.** Every "vs Jul-08" delta is a
large *improvement* (-15.6% to -34.1%), which is exactly what you'd expect if the
Jul-08 numbers were noise-inflated (as that sweep's own report suspected) rather than
evidence of two months of genuine speedups. The more trustworthy comparison is against
the **June-22 controlled re-run** (machine quiescent, low outlier count, explicitly
established as the reliable anchor): today's run lands within **±2.5%** of that anchor
on 6 of 7 pipeline metrics, and `reassembly/tls.pcap` is *better* by 8.9% (still inside
the informational NOISE band, not a flagged improvement/regression). Today's outlier
counts (0–8 per 100, 0–2 severe) are also far lower than Jul-08's (11–20 per 100,
7–16 severe), corroborating that this run — like Jun-22 — was captured on a quiescent
machine and is a valid comparator; Jul-08 was not.

**Conclusion: no regression >10% (WARNING) or >25% (CRITICAL) detected. The codebase's
hot-path performance across STORY-150 → STORY-183 / v0.11.5 → v0.13.3 is stable and
consistent with the June-22 controlled baseline.** `tls_fragmented` now has its first
low-noise measurement (1.5141 µs, 5 outliers) and should be used as its baseline going
forward in preference to the noisy Jul-08 initial value.

### New Baseline Snapshot (recorded for next sweep)

| Benchmark | Fixture | New baseline (µs) |
|-----------|---------|--------------------|
| decode | segmented.pcap | 1.4626 |
| decode | tls.pcap | 3.4387 |
| decode | dns-remoteshell.pcap | 4.7362 |
| summary | segmented.pcap | 0.6543 |
| summary | dns-remoteshell.pcap | 2.6342 |
| reassembly | segmented.pcap | 5.8572 |
| reassembly | tls.pcap | 22.244 |
| tls_fragmented | 3-record-carry-drain | 1.5141 |

### Build-health check (lightweight, no deep analysis)

| Check | Result |
|-------|--------|
| `cargo build --release` | SUCCESS. 6.61s wall (incremental — dependency crates already built in `target/release`; only the `wirerust` crate itself recompiled. Not a clean-tree timing; recorded for trend only). |
| `cargo test --all-targets` | SUCCESS. 94 test binaries, **2668 passed / 0 failed**, 5 ignored (all in `silent_resource_caps`, documented as intentionally slow/`--ignored`-gated MAX_MAP_ENTRIES/MAX_ARP_BINDINGS cap tests — not a gap). Total wall time ~part of the same invocation that smoke-ran both bench binaries in debug/1-iteration mode (not the criterion measurement run). |

### NFR Compliance Matrix

Same as prior sweeps — no per-packet latency NFR-NNN target exists in the NFR catalog
to gate against; this sweep's numbers are informational/trend-tracking only.

| NFR ID | Requirement | Validation Method | Measured | Verdict |
|--------|-------------|------------------|----------|---------|
| NFR-PERF-001 | Zero-copy slice path; one allocation per packet | Code review | Not re-validated this sweep (no allocation-path changes observed since last check) | N/A |
| NFR-PERF-002 | Eager full-pcap load; RAM <= pcap_size * 1.5 | Load test with 1 GB pcap | Not measured — no 1 GB fixture | DEFERRED |
| NFR-PERF-003 | O(1) dispatch via cache; 100% cache hit rate after first classification | Benchmark: 10,000-flow pcap | Not measured — no 10,000-flow fixture | DEFERRED |
| NFR-PERF-004 | Overlap detection uses SIMD-friendly slice equality | cargo asm / LLVM IR inspection | Not validated this sweep | OPEN-DEBT |

### Recommendations

1. **No fix PR triggered.** No metric degraded >10% against either the Sweep-6 or
   June-22 anchors.
2. **Resolve the maint-2026-07-21 anchor discrepancy** with whoever scheduled this
   sweep — no such baseline entry exists in this repo's history; confirm whether it
   was recorded elsewhere (a different branch/artifact store) or the date is a
   mis-reference to Jul-08 / Jun-22.
3. **Retire Jul-08 (Sweep 6) as a regression comparator.** Its own report flagged it
   NOISE-SUSPECT; this sweep confirms that suspicion (all deltas vs it are large
   improvements with no plausible code-level cause). Use Jun-22 and this sweep
   (Sep-05) as the reliable anchors going forward.
4. **Adopt `tls_fragmented/3-record-carry-drain` = 1.5141 µs** as its first low-noise
   baseline; the Jul-08 value (2.0662 µs, 15 severe outliers) should no longer be used
   as a comparator.
5. NFR-PERF-002/003 remain DEFERRED — no large fixtures exist to validate them
   (unchanged from prior sweeps).

### Sweep Metadata

| Field | Value |
|-------|-------|
| Run date | 2026-09-05 |
| Platform | darwin 25.5.0 (Apple Silicon, macOS 26.5.2, build 25F84) |
| Rust toolchain | stable (rustc 1.98.1, cargo 1.98.1) |
| Benchmark command | `cargo bench --bench pipeline` then `cargo bench --bench tls_fragmented` |
| `cargo build --release` | SUCCESS, 6.61s (incremental) |
| `cargo test --all-targets` | SUCCESS, 2668 passed / 0 failed / 5 ignored across 94 binaries |
| Outliers this run | decode/segmented 0; decode/tls 6 (1 severe); decode/dns-remoteshell 8 (2 severe); summary/segmented 4; summary/dns-remoteshell 1; reassembly/segmented 4; reassembly/tls 5 (1 severe); tls_fragmented 5 |
| Thermal state | Not explicitly controlled, but outlier counts are consistent with a quiescent machine (comparable to the Jun-22 controlled run, far below Jul-08's noise signature) |

---

## maint-2026-07-08 (Sweep 6)

**Commit:** b642c0f (develop, v0.11.5)
**Context:** Post-STORY-150 (PR #379, merged 2026-07-08) — refactored TLS drain-loop dispatch
arms. Paying particular attention to TLS-path benches for regression from that refactor.
Also first run to include the `tls_fragmented` bench (added in STORY-149).

### Run Quality Assessment

This run exhibited high severe-outlier counts across all pipeline benchmarks
(11–20 severe outliers per 100 samples). Wide confidence intervals (e.g.
decode/segmented.pcap: [1.67–2.41 µs]) indicate the machine was not fully quiescent
during measurement. This is the same noise signature observed in the June-17 run
(which showed a +54.5% CRITICAL later confirmed as thermal/scheduling noise in the
June-22 controlled re-run).

**Recommendation: treat CRITICAL/WARNING flags on non-TLS paths as NOISE-SUSPECT pending a
controlled re-run with the machine quiescent. Do not open fix PRs based solely on this run.**

### Results Table — pipeline bench

All times are Criterion point estimates (median of CI). Delta vs Jun-22 anchor (maint-2026-06-22).

| Benchmark | Fixture | Jun-22 anchor (µs) | Today 2026-07-08 (µs) | Criterion CI | Outliers | vs Jun-22 | Verdict |
|-----------|---------|-------------------|----------------------|-------------|---------|-----------|---------|
| decode | segmented.pcap | 1.459 | 2.0145 | [1.671–2.417] | 12 (11 severe) | +38.1% | CRITICAL (NOISE-SUSPECT) |
| decode | tls.pcap | 3.369 | 4.3810 | [4.091–4.706] | 12 (8 severe) | +30.0% | CRITICAL (NOISE-SUSPECT) |
| decode | dns-remoteshell.pcap | 4.840 | 5.9408 | [5.500–6.544] | 15 (13 severe) | +22.7% | WARNING (NOISE-SUSPECT) |
| summary | segmented.pcap | 0.639 | 0.9003 | [0.820–0.990] | 11 (7 severe) | +40.9% | CRITICAL (NOISE-SUSPECT) |
| summary | dns-remoteshell.pcap | 2.589 | 3.5304 | [3.148–3.974] | 20 (16 severe) | +36.4% | CRITICAL (NOISE-SUSPECT) |
| reassembly | segmented.pcap | 5.858 | 8.8937 | [7.929–10.069] | 15 (9 severe) | +51.8% | CRITICAL (NOISE-SUSPECT) |
| reassembly | tls.pcap | 24.429 | 26.353 | [25.101–27.850] | 18 (13 severe) | +7.9% | NOISE |

Criterion verdicts vs stored base (prior criterion run):
- `decode/segmented.pcap`: Performance has regressed (+15.9% vs stored base, p<0.05)
- `decode/tls.pcap`: Performance has regressed (+25.2% vs stored base, p<0.05)
- `decode/dns-remoteshell.pcap`: Performance has regressed (+27.8% vs stored base, p<0.05)
- `summary/segmented.pcap`: Performance has regressed (+20.3% vs stored base, p<0.05)
- `summary/dns-remoteshell.pcap`: Performance has regressed (+35.5% vs stored base, p<0.05)
- `reassembly/segmented.pcap`: Performance has regressed (+34.9% vs stored base, p<0.05)
- `reassembly/tls.pcap`: No change in performance detected (p=0.37)

### Results Table — tls_fragmented bench (NEW — first appearance in maintenance sweep)

This bench was added in STORY-149 (AC-149-002) to isolate the TLS carry-drain loop.
No prior maintenance sweep anchor exists; this run establishes the initial baseline.

| Benchmark | Fixture | Today 2026-07-08 (µs) | Criterion CI | Outliers | vs prior | Verdict |
|-----------|---------|----------------------|-------------|---------|---------|---------|
| tls_fragmented | 3-record-carry-drain | 2.0662 | [1.902–2.252] | 18 (15 severe) | N/A (initial) | INITIAL BASELINE |

Criterion: +22.6% vs stored base (p<0.05). High outlier count (18/100 severe) same as other benches,
consistent with machine-noise run. **Initial baseline value recorded; a controlled re-run should
anchor this before using as a regression comparator.**

### STORY-150 TLS Drain-Loop Regression Check

STORY-150 (PR #379, merged 2026-07-08) refactored TLS drain-loop dispatch arms in the
`drain_carry` / stream-dispatch path. Relevant measurements:

| Bench | Today (µs) | STORY-149 pre-story anchor (µs) | Delta | vs May-19 (23.281 µs) |
|-------|-----------|--------------------------------|-------|----------------------|
| reassembly/tls.pcap | 26.353 | 25.880 | +1.8% (NOISE) | +13.2% (WARNING vs long-run anchor) |
| tls_fragmented/3-record-carry-drain | 2.0662 | N/A (no prior) | — | — |

**Finding: No evidence of regression from the STORY-150 TLS drain-loop refactor.**
`reassembly/tls.pcap` at +1.8% vs the STORY-149 pre-story anchor is well within noise;
criterion reports no change (p=0.37). The tls_fragmented path also shows the same
high-outlier noise pattern as all other benches, meaning its absolute value this run is
unreliable as an anchor.

### Noise Diagnosis: Why CRITICAL flags are NOISE-SUSPECT

The key diagnostic is that decode and summary paths are completely unchanged by STORY-150's
TLS drain-loop refactor. If their regressions were real code regressions, there would need
to be an unrelated code change touching decode/summary paths — but there is none (b642c0f is
the STORY-150 merge commit). The simultaneous regression across all independent paths, combined
with high severe-outlier counts in every group, is the canonical fingerprint of OS scheduling
or thermal interference — identical to the June-17 run (later confirmed noise in the Jun-22
controlled re-run).

### Long-Run Trend Table (reassembly/tls.pcap — primary regression-tracking metric)

| Date | Commit | µs (slope/mean) | vs May-19 | Notes |
|------|--------|-----------------|-----------|-------|
| 2026-05-19 | (anchor) | 23.281 | 0.0% | Original baseline |
| 2026-06-17 | (maint) | 35.960 | +54.5% | NOISE — confirmed thermal spike |
| 2026-06-22 | (maint) | 24.429 | +4.9% | Controlled re-run; noise resolved |
| 2026-07-06 | f7460b4 | 27.842 | +19.6% | v0.11.4 (maint-2026-07-06) |
| 2026-07-07 | 19569ae | 25.880 | +11.2% | STORY-149 pre-story anchor (v0.11.5) |
| 2026-07-08 | b642c0f | 26.353 | +13.2% | This run; high outliers — noise-suspect |

The long-run trend for reassembly/tls.pcap shows a stable elevation of ~11–13% above the
May-19 anchor, consistent with the ARP feature cycle overhead identified in prior sweeps.
No step-change attributable to STORY-150.

---

# Maintenance Sweep 5 — Controlled Re-run Performance Baseline

## Purpose

The maint-2026-06-17 sweep flagged:
- Five WARNING-level regressions (10–22% vs May-19 baseline)
- One CRITICAL-level regression (`reassembly/tls.pcap` +54.5% median, high variance)
- Tech-debt item TD-MAINT-PERF-ARP-HOTPATH: attributed to `DecodedFrame::Arp` match
  variant in the hot decode loop, recommended controlled re-run to distinguish
  measurement noise from a real regression.

This document records the results of that controlled re-run (2026-06-22, machine
quiescent, background processes minimized) and assesses whether the prior regressions
were real or noise.

---

## Results Table

All times are mean per-iteration. Delta columns compare against two anchors:

- **vs May-19**: Original baseline before the ARP feature cycle (STORY-111..115)
- **vs June-17**: Prior sweep run (what criterion's "base" pointed to this run)

| Benchmark | Fixture | May-19 baseline (µs) | June-17 prior sweep (µs) | Today 2026-06-22 (µs) | vs May-19 | vs June-17 | Verdict |
|-----------|---------|----------------------|--------------------------|----------------------|-----------|------------|---------|
| decode | segmented.pcap | 1.440 | 1.468 | 1.459 | +1.3% | -0.7% | NOISE |
| decode | tls.pcap | 3.002 | 3.658 | 3.369 | +12.2% | -7.9% | REGRESSION-MINOR |
| decode | dns-remoteshell.pcap | 4.472 | 4.960 | 4.840 | +8.2% | -2.4% | NOISE |
| summary | segmented.pcap | 0.600 | 0.670 | 0.639 | +6.5% | -4.6% | NOISE |
| summary | dns-remoteshell.pcap | 2.535 | 2.667 | 2.589 | +2.1% | -2.9% | NOISE |
| reassembly | segmented.pcap | 4.907 | 5.894 | 5.858 | +19.4% | -0.6% | REGRESSION-MINOR |
| reassembly | tls.pcap | 23.281 | 35.960 | 24.429 | +4.9% | -32.1% | NOISE |

Criterion verdicts from this run (vs June-17 base):
- `decode/segmented.pcap`: No change in performance detected (p=0.20)
- `decode/tls.pcap`: Performance has improved (-7.9%, p<0.05)
- `decode/dns-remoteshell.pcap`: Performance has improved (-2.4%, p<0.05)
- `summary/segmented.pcap`: Performance has improved (-4.5%, p<0.05)
- `summary/dns-remoteshell.pcap`: Performance has improved (-2.9%, p<0.05)
- `reassembly/segmented.pcap`: No change in performance detected (p=0.36), 6 outliers
- `reassembly/tls.pcap`: Performance has improved (-32.1%, p<0.05), 10 outliers

---

## ARP Hotpath Regression: Real or Noise?

### TD-MAINT-PERF-ARP-HOTPATH assessment

**Conclusion: PARTIALLY NOISE — the CRITICAL `reassembly/tls.pcap` finding was thermal/scheduling noise. Two REGRESSION-MINOR findings are real but stable.**

#### reassembly/tls.pcap — WAS CRITICAL (+54.5%), NOW NOISE (+4.9% vs May-19)

The June-17 run measured 35.960 µs mean with std_dev ~14 µs (65x higher than the
0.2 µs baseline std_dev). This run measures 24.429 µs mean — virtually identical to
the May-19 baseline of 23.281 µs (delta +4.9%, well within noise). Criterion reports
a -32.1% improvement vs the prior run (p<0.05).

**Finding: The CRITICAL regression in `reassembly/tls.pcap` was measurement noise
(thermal throttling or OS scheduling spike during the June-17 run window). It is
NOT a real regression. The TLS reassembly path is performing at baseline levels.**

#### decode/tls.pcap — WAS WARNING (+21.9%), REMAINS REGRESSION-MINOR (+12.2% vs May-19)

This run measures 3.369 µs vs the May-19 baseline of 3.002 µs (+12.2%). The June-17
run was 3.658 µs; today's value is lower (-7.9%) but still above the original baseline
by more than the 10% WARNING threshold. Criterion: statistically significant improvement
vs June-17 (p<0.05), but the May-19 anchor shows a real ~0.37 µs permanent overhead.

**Finding: A genuine minor regression exists in `decode/tls.pcap`. It is stable
(not worsening), consistent with the `DecodedFrame::Arp` match overhead hypothesis.
Classification: REGRESSION-MINOR.**

#### reassembly/segmented.pcap — WAS WARNING (+20.1%), REMAINS REGRESSION-MINOR (+19.4% vs May-19)

This run measures 5.858 µs vs the May-19 baseline of 4.907 µs (+19.4%). The June-17
value was 5.894 µs; today is essentially the same (-0.6%, p=0.36, no change detected).
This is the most stable measurement across both runs and confirms a real, persistent
regression of ~0.95 µs per iteration in the full reassembly pipeline.

**Finding: A genuine minor regression exists in `reassembly/segmented.pcap`. Stable
across two independent runs. Classification: REGRESSION-MINOR.**

#### All other benchmarks — NOISE

`decode/segmented.pcap` (+1.3%), `decode/dns-remoteshell.pcap` (+8.2%),
`summary/segmented.pcap` (+6.5%), `summary/dns-remoteshell.pcap` (+2.1%) all fell
below 10% vs the May-19 baseline this run, and are within or near criterion noise
thresholds. The June-17 readings for these (10.9%, 11.7%, 5.2%) were elevated by the
same thermal/scheduling conditions that caused the CRITICAL tls.pcap reading.

---

## Summary: Prior Findings Disposition

| June-17 Finding | June-17 Verdict | This Run (vs May-19) | Disposition |
|-----------------|-----------------|----------------------|-------------|
| decode/tls.pcap +21.9% | WARNING | +12.2% | REAL — stable regression, reduced from 22% to 12%. REGRESSION-MINOR. |
| decode/dns-remoteshell.pcap +10.9% | WARNING | +8.2% | NOISE — fell below 10% threshold this run. Borderline; monitor. |
| summary/segmented.pcap +11.7% | WARNING | +6.5% | NOISE — fell below 10% threshold this run. |
| reassembly/segmented.pcap +20.1% | WARNING | +19.4% | REAL — stable across both runs. REGRESSION-MINOR. |
| reassembly/tls.pcap +54.5% | CRITICAL | +4.9% | NOISE — thermal spike in June-17 run. NOT a real regression. |
| decode/segmented.pcap +1.9% | within noise | +1.3% | NOISE |
| summary/dns-remoteshell.pcap +5.2% | within noise | +2.1% | NOISE |

**Net: 2 confirmed real regressions (decode/tls.pcap +12.2%, reassembly/segmented.pcap
+19.4% vs original May-19 baseline). Both attributable to the ARP feature cycle. No
CRITICAL regressions confirmed. The June-17 CRITICAL was noise.**

---

## NFR Compliance Matrix

No per-packet latency target exists in the NFR catalog. Compliance determination is
informational only.

| NFR ID | Requirement | Validation Method | Measured | Verdict |
|--------|-------------|------------------|----------|---------|
| NFR-PERF-001 | Zero-copy slice path; one allocation per packet | Code review | Not re-measured (no change to allocation path since June-17) | N/A |
| NFR-PERF-002 | Eager full-pcap load; RAM <= pcap_size * 1.5 | Load test with 1 GB pcap | Not measured — no 1 GB fixture | DEFERRED |
| NFR-PERF-003 | O(1) dispatch via cache; 100% cache hit rate after first classification | Benchmark: 10,000-flow pcap | Not measured — no 10,000-flow fixture | DEFERRED |
| NFR-PERF-004 | Overlap detection uses SIMD-friendly slice equality | cargo asm / LLVM IR inspection | Not validated this sweep | OPEN-DEBT |

---

## Recommendations

1. **Close TD-MAINT-PERF-ARP-HOTPATH as PARTIALLY-CONFIRMED.** The CRITICAL finding
   was noise. Two REGRESSION-MINOR findings (+12% decode/tls, +19% reassembly/segmented)
   are real and stable. Neither worsened between June-17 and today, indicating the
   regressions are fixed overhead from the ARP feature, not a growing leak.

2. **No immediate action required.** Since no numerical NFR latency target exists, these
   regressions are informational. The ARP decode overhead is ~0.37 µs per tls.pcap
   iteration and ~0.95 µs per segmented.pcap reassembly iteration. At these absolute
   values, the impact on user-facing throughput is negligible for typical pcap sizes.

3. **Establish a committed baseline.** Update `performance-baseline.md` with the
   2026-06-22 values (this document) as the new anchor for future sweeps. Prior May-19
   and June-17 values are preserved in `.factory/maintenance/performance.md`.

4. **Monitor decode/dns-remoteshell.pcap.** It measured +10.9% (WARNING) in June-17
   and +8.2% (NOISE) today — borderline. One more data point will clarify trend.

5. **NFR-PERF-002/003 remain DEFERRED** — no large fixtures exist to validate them.

---

## Sweep Metadata

| Field | Value |
|-------|-------|
| Run date | 2026-06-22 |
| Platform | darwin 25.5.0 (Apple Silicon, macOS Sequoia 15.5) |
| Rust toolchain | stable (v0.9.3) |
| Benchmark command | `cargo bench --bench pipeline` |
| Commits since May-19 baseline | ~367 (includes v0.9.3 release) |
| Criterion "base" compared against | maint-2026-06-17 stored target/criterion values |
| Outliers this run | segmented.pcap: 6 (2 high mild, 4 high severe); tls.pcap: 10 (5 high mild, 5 high severe) |
| Thermal state | Machine quiescent; background apps minimized |
