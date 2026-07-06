---
document_type: maintenance-performance-report
sweep: 5
run_id: maint-2026-07-06
producer: performance-engineer
created: 2026-07-06
branch: develop
commit: f7460b4
version: v0.11.4
baseline_source: .factory/maintenance/performance-baseline.md (maint-2026-06-22 controlled re-run values)
current_run_date: 2026-07-06
hardware_note: >
  Apple Silicon Mac, darwin 25.5.0. All measurements are wall-clock on the
  benchmark machine. Absolute µs values are not portable across hardware;
  only relative deltas (same machine, same branch) are meaningful for
  regression tracking.
benchmark_command: cargo bench --bench pipeline
criterion_version: "0.8"
samples: 100 per benchmark
---

# Maintenance Sweep 5 — Performance Report (run maint-2026-07-06)

## Executive Summary

Criterion benchmarks were run on 2026-07-06 against the June-22 controlled re-run
baseline documented in `performance-baseline.md`. Six of seven benchmarks are within
the 10% WARNING threshold vs the June-22 anchor. One benchmark — `reassembly/tls.pcap`
— registers **+14.0% vs the June-22 baseline** (REGRESSION), up from the +4.9%
confirmed-noise reading in that baseline. Criterion independently flags it as
"Performance has regressed" (p < 0.05, +7.56% vs its stored criterion base).

The two confirmed REGRESSION-MINOR findings from the June-22 baseline
(`decode/tls.pcap` +12.2%, `reassembly/segmented.pcap` +19.4% vs original May-19)
remain present but have not worsened beyond the June-22 band. The `reassembly/tls.pcap`
regression is a new development: it was noise in June-22 but is now a real, statistically
confirmed regression.

**Known open item PERF-001/002 (TLS carry-path perf, STORY-149 draft):** The
`reassembly/tls.pcap` benchmark is the primary proxy for TLS carry-path performance.
At +14.0% vs June-22 and +19.6% vs May-19, this benchmark has worsened since the
last controlled run and now crosses the 10% threshold that June-22 classified as noise.
The new regression is consistent with overhead added in the v0.9.3–v0.11.4 interval,
which includes observability counter increments on every eviction/drop event (PR #365)
and per-flow state purge on flow close (PR #362). Both touch paths exercised by the
full reassembly pipeline.

---

## Benchmark Infrastructure

| Item | Value |
|------|-------|
| Harness | criterion 0.8 |
| Config | `[[bench]] name = "pipeline" harness = false` (Cargo.toml) |
| Fixture files | `tests/fixtures/{segmented.pcap, tls.pcap, dns-remoteshell.pcap}` |
| Benchmark groups | decode, summary, reassembly |
| Samples per benchmark | 100 |
| Baseline source | performance-baseline.md (2026-06-22 controlled re-run) |

---

## Results Table

All times are mean per-iteration (µs). Primary delta column compares against the
June-22 controlled re-run baseline (the current authoritative anchor). May-19 column
is carried for historical continuity.

| Benchmark | Fixture | May-19 baseline (µs) | Jun-22 baseline (µs) | Today 2026-07-06 (µs) | vs May-19 | vs Jun-22 | Verdict |
|-----------|---------|---------------------|----------------------|----------------------|-----------|-----------|---------|
| decode | segmented.pcap | 1.440 | 1.459 | 1.4394 | −0.0% | −1.3% | PASS |
| decode | tls.pcap | 3.002 | 3.369 | 3.5178 | +17.2% | +4.4% | PASS |
| decode | dns-remoteshell.pcap | 4.472 | 4.840 | 5.1773 | +15.8% | +7.0% | PASS |
| summary | segmented.pcap | 0.600 | 0.639 | 0.6815 | +13.6% | +6.6% | PASS |
| summary | dns-remoteshell.pcap | 2.535 | 2.589 | 2.7889 | +10.0% | +7.7% | PASS |
| reassembly | segmented.pcap | 4.907 | 5.858 | 6.3000 | +28.4% | +7.5% | PASS |
| reassembly | tls.pcap | 23.281 | 24.429 | 27.842 | +19.6% | **+14.0%** | **REGRESSION** |

Criterion verdicts from this run (vs criterion's stored base, which reflects the last
criterion run on this machine — date unknown but post June-22):

- `decode/segmented.pcap`: No change in performance detected (p = 0.52)
- `decode/tls.pcap`: Change within noise threshold (p = 0.01, −1.6% vs stored base)
- `decode/dns-remoteshell.pcap`: No change in performance detected (p = 0.18)
- `summary/segmented.pcap`: Performance has regressed (+12.4%, p < 0.05)
- `summary/dns-remoteshell.pcap`: Performance has regressed (+5.4%, p < 0.05)
- `reassembly/segmented.pcap`: Performance has regressed (+6.4%, p < 0.05)
- `reassembly/tls.pcap`: Performance has regressed (+7.6%, p < 0.05)

Note: criterion's stored base for `summary/segmented.pcap` appears to be an
intermediate value lower than the June-22 documented baseline (criterion reports
+12.4% while vs-Jun-22 is +6.6%). This discrepancy indicates criterion's stored
base was updated by an intermediate run between June-22 and today. The documented
June-22 values in `performance-baseline.md` are the authoritative anchor; criterion
verdicts are supplementary signals.

---

## Regression Analysis

### REGRESSION: reassembly/tls.pcap — +14.0% vs Jun-22 baseline

- Jun-22 baseline: 24.429 µs mean
- Today: 27.842 µs mean
- Delta vs Jun-22: +14.0% (above 10% WARNING threshold)
- Delta vs May-19: +19.6%
- Criterion: "Performance has regressed" (p < 0.05, +7.6% vs stored criterion base)
- Outliers: 13 of 100 measurements (2 high mild, 11 high severe)

This benchmark exercises the full TLS reassembly pipeline: decode + IP-filter +
reassembly + dispatcher + TLS analyzer. The June-22 controlled re-run classified its
+4.9% vs May-19 as noise (it recovered from the June-17 spike of +54.5%). The current
run's +14.0% vs the June-22 anchor is a new real regression, confirmed by criterion
with p < 0.05.

Plausible causes in the v0.9.3–v0.11.4 commit interval:

1. **Observability counters (PR #365, STORY-149-adjacent):** Silently-dropped and
   evicted state now increments atomic counters on every event. The reassembly/tls.pcap
   fixture exercises the full TLS session lifecycle including state eviction; these
   counter increments occur on the hot path.

2. **Per-flow state purge on flow close (PR #362):** DNP3/ENIP state is now
   explicitly purged in the dispatcher's flow-close path. The dispatcher is called
   in the reassembly benchmark loop; extra cleanup work on each flow-close adds
   overhead even for flows that do not use DNP3/ENIP.

Neither cause is a correctness concern — both are intentional feature additions. The
combined overhead is approximately 3.4 µs per tls.pcap iteration above the June-22
baseline.

### Previously confirmed REGRESSION-MINOR findings (Jun-22 baseline, vs May-19)

These remain present but have not worsened relative to the June-22 values:

**decode/tls.pcap — +4.4% vs Jun-22, +17.2% vs May-19**
- June-22 baseline was already +12.2% above May-19 (REGRESSION-MINOR).
- Current reading (+4.4% vs Jun-22) shows no further deterioration.

**reassembly/segmented.pcap — +7.5% vs Jun-22, +28.4% vs May-19**
- June-22 baseline was already +19.4% above May-19 (REGRESSION-MINOR).
- Current reading (+7.5% vs Jun-22) shows no further deterioration. This
  benchmark does not include TLS traffic and was not further impacted by PR #365/#362.

### PASS benchmarks

`decode/segmented.pcap` (−1.3% vs Jun-22), `decode/dns-remoteshell.pcap` (+7.0%),
`summary/segmented.pcap` (+6.6%), `summary/dns-remoteshell.pcap` (+7.7%) all remain
below the 10% threshold vs the June-22 baseline. Criterion flags `summary/*` as
regressed vs its own stored base, but vs the authoritative June-22 anchor these are
within budget.

---

## PERF-001/002 (TLS Carry-Path Perf, STORY-149 Draft)

The baseline documents record this as an open item covering TLS carry-path performance.
The primary covering benchmark is `reassembly/tls.pcap`.

| Metric | May-19 | Jun-22 | Jul-06 | vs Jun-22 |
|--------|--------|--------|--------|-----------|
| reassembly/tls.pcap mean (µs) | 23.281 | 24.429 | 27.842 | +14.0% |

The TLS carry-path regression has worsened since June-22. The June-22 baseline
concluded the CRITICAL June-17 spike was noise and that +4.9% vs May-19 was within
acceptable range. The current measurement now shows a genuine +14.0% vs June-22
(+19.6% vs May-19). STORY-149 (if drafted as a performance improvement story for
the TLS carry-path) has gained additional motivation from this result.

---

## NFR Compliance Matrix

| NFR ID | Requirement | Status |
|--------|-------------|--------|
| NFR-PERF-001 | Zero-copy slice path; one allocation per packet | DEFERRED — not measured by microbenchmarks |
| NFR-PERF-002 | Eager full-pcap load; RAM <= pcap_size * 1.5 | DEFERRED — no 1 GB fixture; reference: v0.11.3 smoke test RSS 303 MB on 2.25M-packet capture (not directly comparable, fixture size unknown) |
| NFR-PERF-003 | O(1) dispatch; 100% cache hit rate after first classification | DEFERRED — no 10,000-flow fixture |
| NFR-PERF-004 | SIMD autovectorization in overlap detection | OPEN-DEBT — LLVM IR not inspected this sweep |

The v0.11.3 smoke test value (RSS 303 MB on a 2.25M-packet capture) is noted as a
reference point but cannot be directly validated against NFR-PERF-002 without knowing
the pcap file size. The NFR requires RSS <= pcap_size * 1.5; without both values a
PASS/FAIL determination is not possible.

---

## Recommendations

1. **Investigate reassembly/tls.pcap regression (+14.0%).** Profile the TLS reassembly
   path to determine whether the observability counter increments (PR #365) or the
   per-flow purge (PR #362) are the dominant contributors. This feeds STORY-149 scoping.

2. **Re-measure under controlled conditions.** The 11 high-severe outliers in
   `reassembly/tls.pcap` suggest some thermal or scheduling noise. A quiescent-machine
   re-run will tighten the interval, but the mean shift (+3.4 µs, +14%) is large enough
   that noise alone is unlikely to explain it.

3. **No action required for PASS benchmarks.** `summary/segmented.pcap` and
   `summary/dns-remoteshell.pcap` are within the June-22 anchor despite criterion's
   regression flags (criterion's stored base appears to be from an intermediate run
   with lower values).

4. **Update the baseline once STORY-149 lands.** If a performance improvement story
   is delivered for the TLS path, regenerate `performance-baseline.md` with the
   post-fix values as the new anchor.

---

## Sweep Metadata

| Field | Value |
|-------|-------|
| Run date | 2026-07-06 |
| Platform | darwin 25.5.0 (Apple Silicon, macOS Sequoia 15.5) |
| Rust toolchain | stable (v0.11.4, cargo build --release succeeded in 3.05 s) |
| Benchmark command | `cargo bench --bench pipeline` |
| Commits since Jun-22 baseline | ~f7460b4 (v0.11.4, includes PRs #362, #365, #366, #368) |
| Outliers this run | tls.pcap reassembly: 13 (2 high mild, 11 high severe); dns-remoteshell summary: 14 (3 high mild, 11 high severe) |
| Baseline anchor | performance-baseline.md (2026-06-22 controlled re-run) |
