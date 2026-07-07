---
document_type: story-performance-baseline
story: STORY-149
wave: 70
producer: performance-engineer
created: 2026-07-07
branch: develop
commit: 19569ae
version: v0.11.5
baseline_name: story149-pre
benchmark_command: cargo bench --bench pipeline -- --save-baseline story149-pre reassembly
criterion_version: "0.8"
rust_version: "1.96.0 (ac68faa20 2026-05-25)"
samples: 100
hardware_note: >
  Apple Silicon Mac, darwin 25.5.0. All measurements are wall-clock on the
  benchmark machine. Absolute µs values are not portable across hardware;
  only relative deltas (same machine, same branch) are meaningful for
  regression tracking.
---

# STORY-149 Pre-Story Criterion Baseline

## Purpose

Capture the `reassembly/tls.pcap` Criterion benchmark at develop HEAD (19569ae,
v0.11.5) before STORY-149 restructures `try_parse_records` in
`src/analyzer/tls.rs`. AC-149-003 requires showing that the post-story benchmark
recovers to within +5% of the May-19 anchor (23.281 µs). This file provides the
reference point for that comparison.

---

## Benchmark Run

| Field | Value |
|-------|-------|
| Bench target | `pipeline` (`[[bench]] name = "pipeline" harness = false`) |
| Benchmark group | `reassembly` |
| Fixture | `tests/fixtures/tls.pcap` |
| Criterion name | `reassembly/tls.pcap` |
| Saved baseline | `story149-pre` |
| Baseline path | `target/criterion/reassembly/tls.pcap/story149-pre/` |
| Commit | 19569ae (develop, v0.11.5) |
| Date | 2026-07-07 |

---

## Results: reassembly/tls.pcap

All values in microseconds (µs).

| Statistic | Value (µs) | Source |
|-----------|-----------|--------|
| Slope (point estimate) | 25.880 | Criterion terminal output; `estimates.json` slope.point_estimate |
| Slope 95% CI lower | 25.823 | estimates.json slope.confidence_interval.lower_bound |
| Slope 95% CI upper | 25.941 | estimates.json slope.confidence_interval.upper_bound |
| Mean (statistical) | 25.941 | estimates.json mean.point_estimate |
| Mean 95% CI | [25.866, 26.020] | estimates.json mean.confidence_interval |
| Median | 25.887 | estimates.json median.point_estimate |
| Median 95% CI | [25.793, 25.956] | estimates.json median.confidence_interval |
| Std Dev | 0.398 | estimates.json std_dev.point_estimate |
| MAD | 0.305 | estimates.json median_abs_dev.point_estimate |

Criterion terminal output (verbatim):
```
reassembly/tls.pcap     time:   [25.823 µs 25.880 µs 25.941 µs]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild
```

---

## Delta Analysis

### Reference anchors (from performance.md and performance-baseline.md)

| Anchor | Date | Value (µs) | Source |
|--------|------|-----------|--------|
| May-19 original | 2026-05-19 | 23.281 | performance-baseline.md, May-19 baseline column |
| Jun-22 controlled re-run | 2026-06-22 | 24.429 | performance-baseline.md, maint-2026-06-22 run |
| maint-2026-07-06 | 2026-07-06 | 27.842 | performance.md, maint-2026-07-06 run (commit f7460b4, v0.11.4) |

### Current (19569ae, v0.11.5) vs anchors

Using slope estimate (25.880 µs) as primary metric — consistent with prior reports:

| Anchor | Anchor (µs) | Current (µs) | Delta | Classification |
|--------|------------|--------------|-------|----------------|
| May-19 | 23.281 | 25.880 | **+11.16%** | REGRESSION (>10% WARNING threshold) |
| Jun-22 | 24.429 | 25.880 | **+5.94%** | NOISE/BORDERLINE (<10% WARNING threshold) |
| maint-2026-07-06 | 27.842 | 25.880 | **-7.05%** | IMPROVEMENT (v0.11.5 vs v0.11.4) |

Note: The v0.11.5 run (25.880 µs) is 1.962 µs lower than the maint-2026-07-06 run
(27.842 µs at f7460b4). The difference likely reflects a combination of machine
state variation and the v0.11.5 merge not touching the TLS analysis hot path. The
canonical regression vs May-19 is +11.16%.

---

## AC-149-003 Target

AC-149-003 requires the post-story benchmark to recover to within +5% of the
May-19 anchor.

| Item | Value |
|------|-------|
| May-19 anchor | 23.281 µs |
| AC-149-003 target ceiling | 24.445 µs (23.281 × 1.05) |
| Pre-story slope | 25.880 µs |
| Required reduction | ≥ 1.435 µs (5.5% of current) |

STORY-149 must bring `reassembly/tls.pcap` slope to ≤ 24.445 µs to satisfy
AC-149-003. Verify post-story by running:

```bash
cargo bench --bench pipeline -- --baseline story149-pre reassembly/tls
```

The post-story run should show no regression vs `story149-pre` AND the absolute
mean must land ≤ 24.445 µs.

---

## Verification Steps Used

1. `git log -1 --oneline` confirmed HEAD = 19569ae (develop, v0.11.5, clean tree).
2. Confirmed single `[[bench]]` entry in Cargo.toml: `name = "pipeline"`.
3. Confirmed `bench_reassembly` function in `benches/pipeline.rs` creates group
   `"reassembly"` with fixture loop including `"tls.pcap"`.
4. Ran `cargo bench --bench pipeline -- --save-baseline story149-pre reassembly`
   (release profile; Criterion default 100 samples, 3 s warmup).
5. Extracted estimates from
   `target/criterion/reassembly/tls.pcap/story149-pre/estimates.json`.
