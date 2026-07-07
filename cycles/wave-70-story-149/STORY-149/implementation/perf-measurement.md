---
document_type: story-performance-measurement
story: STORY-149
wave: 70
producer: performance-engineer
created: 2026-07-07
branch: feature/STORY-149-tls-carry-perf
commit: 923fac0
pre_baseline_commit: 19569ae
pre_baseline_name: story149-pre
post_baseline_name: story149-post
benchmark_commands:
  - cargo bench --bench pipeline -- --baseline story149-pre reassembly
  - cargo bench --bench pipeline -- --save-baseline story149-post reassembly
  - cargo bench --bench tls_fragmented -- --save-baseline story149-post
criterion_version: "0.8"
rust_version: "1.96.0 (ac68faa20 2026-05-25)"
samples: 100
hardware_note: >
  Apple Silicon Mac, darwin 25.5.0. Same machine as pre-story baseline (story149-pre).
  All measurements are wall-clock. Absolute µs values are not portable across hardware;
  only relative deltas (same machine, same branch) are meaningful for regression tracking.
---

# STORY-149 AC-149-003 Performance Measurement

## Purpose

Verify that the post-story `reassembly/tls.pcap` Criterion slope recovers to within +5% of
the May-19 anchor (23.281 µs), satisfying AC-149-003. The pre-story baseline was captured at
develop HEAD 19569ae (v0.11.5) as named Criterion baseline `story149-pre` (slope 25.880 µs).

---

## Commit and Branch

| Field | Value |
|-------|-------|
| Measured commit | 923fac0 |
| Branch | feature/STORY-149-tls-carry-perf |
| Pre-story baseline commit | 19569ae (develop, v0.11.5) |
| Pre-story baseline name | story149-pre |
| Post-story baseline name | story149-post |
| Measurement date | 2026-07-07 |

---

## Run 1: Baseline Comparison (reassembly/tls.pcap vs story149-pre)

Command: `cargo bench --bench pipeline -- --baseline story149-pre reassembly`

Criterion terminal output (verbatim):
```
reassembly/tls.pcap     time:   [23.602 µs 23.841 µs 24.141 µs]
                        change: [−7.9532% −6.2847% −4.3644%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high severe
```

This run is the **primary AC-149-003 measurement**. The Criterion slope (point estimate) of
23.841 µs is the canonical value used for delta and verdict computations.

---

## Run 2: story149-post Save-Baseline (reassembly)

Command: `cargo bench --bench pipeline -- --save-baseline story149-post reassembly`

All values in nanoseconds from `estimates.json`, converted to µs for readability.

### reassembly/tls.pcap

| Statistic | Value (µs) | 95% CI Lower (µs) | 95% CI Upper (µs) |
|-----------|-----------|-------------------|-------------------|
| Slope     | 23.184    | 22.917            | 23.534            |
| Mean      | 23.974    | 23.511            | 24.477            |
| Median    | 22.734    | 22.674            | 22.814            |
| Std Dev   | 2.491     | 1.983             | 2.862             |
| MAD       | 0.303     | 0.173             | 0.570             |

Criterion terminal output (verbatim):
```
reassembly/tls.pcap     time:   [22.917 µs 23.184 µs 23.534 µs]
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high severe
```

### reassembly/segmented.pcap (context only)

| Statistic | Value (µs) |
|-----------|-----------|
| Time      | [6.101 µs 6.222 µs 6.383 µs] |

---

## Run 3: tls_fragmented Bench — Initial Baseline

Command: `cargo bench --bench tls_fragmented -- --save-baseline story149-post`

This is the **initial baseline** for the carry-drain criterion introduced in STORY-149.
No prior baseline existed; this run establishes story149-post as the regression anchor
for future carry-path changes.

### tls_fragmented/3-record-carry-drain

| Statistic | Value (µs) | 95% CI Lower (µs) | 95% CI Upper (µs) |
|-----------|-----------|-------------------|-------------------|
| Slope     | 1.594     | 1.588             | 1.601             |
| Mean      | 1.604     | 1.601             | 1.608             |
| Median    | 1.611     | 1.608             | 1.612             |
| Std Dev   | 0.019     | 0.016             | 0.022             |
| MAD       | 0.013     | 0.009             | 0.016             |

Criterion terminal output (verbatim):
```
tls_fragmented/3-record-carry-drain
                        time:   [1.5883 µs 1.5942 µs 1.6007 µs]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) low mild
  1 (1.00%) high mild
```

The tls_fragmented bench is very tight (MAD 13 ns, < 1% of mean), confirming the
carry-drain path is a stable, low-variance hotspot. Future changes must regress ≤ +10%
vs story149-post (warning threshold) or ≤ +25% (critical threshold).

---

## Delta Analysis

Primary measurement: Run 1 comparison slope = **23.841 µs**.

| Anchor | Date | Anchor (µs) | Measured (µs) | Delta (µs) | Delta (%) | Classification |
|--------|------|------------|---------------|-----------|-----------|----------------|
| story149-pre | 2026-07-07 | 25.880 | 23.841 | −2.039 | **−7.88%** | IMPROVEMENT |
| May-19 original | 2026-05-19 | 23.281 | 23.841 | +0.560 | **+2.41%** | within +5% window |
| Jun-22 controlled re-run | 2026-06-22 | 24.429 | 23.841 | −0.588 | **−2.41%** | IMPROVEMENT |

95% CI upper bound of Run 1: 24.141 µs (also below target ceiling).

---

## AC-149-003 Verdict

| Item | Value |
|------|-------|
| AC | AC-149-003 |
| Requirement | reassembly/tls.pcap slope ≤ 24.445 µs (May-19 anchor 23.281 µs × 1.05) |
| Measured slope (Run 1 point estimate) | **23.841 µs** |
| Measured 95% CI upper | **24.141 µs** |
| Target ceiling | **24.445 µs** |
| Margin (ceiling − measured) | **0.604 µs (2.47%)** |
| Criterion comparison verdict | "Performance has improved." (−6.28% vs story149-pre) |
| **AC-149-003 verdict** | **PASS** |

Both the point estimate (23.841 µs) and the 95% CI upper bound (24.141 µs) are below
the 24.445 µs target ceiling. The STORY-149 single-borrow refactor of `try_parse_records`
eliminates the clone-then-extend anti-pattern, recovering 2.039 µs (7.88%) vs pre-story
and placing the slope within +2.41% of the May-19 anchor — well within the +5% window.

---

## Post-story Baselines Saved

| Baseline name | Bench | Path in worktree |
|---------------|-------|-----------------|
| story149-post | pipeline (reassembly/*) | target/criterion/reassembly/*/story149-post/ |
| story149-post | tls_fragmented | target/criterion/tls_fragmented/3-record-carry-drain/story149-post/ |

To run future carry-path regression checks against story149-post:
```bash
cargo bench --bench tls_fragmented -- --baseline story149-post
cargo bench --bench pipeline -- --baseline story149-post reassembly
```
