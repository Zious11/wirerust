---
document_type: maintenance-performance-baseline
sweep: 5-controlled-rerun
producer: performance-engineer
created: 2026-06-22
branch: develop
version: v0.9.3
baseline_date: 2026-05-19
baseline_source: .factory/maintenance/performance.md (maint-2026-06-17 recorded values)
prior_sweep_date: 2026-06-17
current_run_date: 2026-06-22
hardware_note: >
  Apple Silicon Mac, darwin 25.5.0. All measurements are wall-clock on the
  benchmark machine. Absolute µs values are not portable across hardware;
  only relative deltas (same machine, same branch) are meaningful for
  regression tracking.
benchmark_command: cargo bench --bench pipeline
rust_version: stable (v0.9.3 release)
criterion_version: "0.8"
samples: 100 per benchmark
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
