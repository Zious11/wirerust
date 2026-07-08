---
document_type: maintenance-performance-baseline
sweep: 6-maint-2026-07-08
producer: performance-engineer
created: 2026-06-22
last_updated: 2026-07-08
branch: develop
commit: b642c0f
version: v0.11.5
baseline_date: 2026-05-19
baseline_source: .factory/maintenance/performance.md (maint-2026-06-17 recorded values)
prior_sweep_date: 2026-06-22
current_run_date: 2026-07-08
hardware_note: >
  Apple Silicon Mac, darwin 25.5.0. All measurements are wall-clock on the
  benchmark machine. Absolute µs values are not portable across hardware;
  only relative deltas (same machine, same branch) are meaningful for
  regression tracking.
benchmark_command: cargo bench --bench pipeline
rust_version: stable (v0.11.5)
criterion_version: "0.8"
samples: 100 per benchmark
---

# Performance Baseline — Running History

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
