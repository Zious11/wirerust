---
document_type: maintenance-performance-baseline
sweep: maint-2026-07-11
producer: performance-engineer
created: 2026-07-11
branch: develop
commit: b5e1e15
version: v0.12.0
run_date: 2026-07-11
baseline_source: .factory/maintenance/performance-baseline.md (maint-2026-06-22 controlled re-run values)
register_item: PERF-RERUN-001
hardware_note: >
  Apple Silicon Mac, darwin 25.5.0. All measurements are wall-clock on the
  benchmark machine. Absolute µs values are not portable across hardware;
  only relative deltas (same machine, same branch) are meaningful for
  regression tracking.
benchmark_command: cargo bench --bench pipeline && cargo bench --bench tls_fragmented
criterion_version: "0.8"
samples: 100 per benchmark
---

# Performance Baseline — maint-2026-07-11 (Sweep 5 PERF-RERUN-001)

## Executive Summary

**PERF-RERUN-001 status: CANNOT CLOSE — machine severely loaded.**

This run was the intended PERF-RERUN-001 controlled re-run to replace the
thermal-noise-contaminated maint-2026-07-08 baseline. The machine measured load
averages of 52.57 / 46.80 / 35.41 at run time, with OrbStack (VM manager, ~480%
CPU), multiple node processes (8+ processes, 17–27% CPU each), and Chrome GPU
processes all active. This is the most severely loaded machine state across all
documented maintenance sweeps.

All benchmarks show CRITICAL apparent regressions vs Jun-22 anchor (54–144%), but
ALL flags are NOISE-SUSPECT. The wave-72 delta (b642c0f..b5e1e15) is cold-path
only — no hot-path code changed since b642c0f. No confirmed regressions. No fix
PRs warranted.

The Jun-22 controlled re-run values remain the sole authoritative anchor. Do not
use this run's data as a new anchor. PERF-RERUN-001 remains open.

---

## System Load Assessment

| Item | Value |
|------|-------|
| Load averages at run time | 52.57 / 46.80 / 35.41 (1m / 5m / 15m) |
| Top consumer | OrbStack Helper (vmgr) ~480% CPU |
| Other significant load | 8+ node processes @ 17–27% CPU each; Chrome GPU/renderer; iTerm2 |
| Machine state | SEVERELY LOADED — NOT quiescent |
| Reliability | ALL benchmark values UNRELIABLE — NOISE-SUSPECT |

The noise signature here differs from prior contaminated runs (Jul-08, Jul-09):
those showed 9–21 severe outliers per 100 samples with moderately elevated medians.
This run shows fewer severe outliers (0–7) but extreme CI widths and median inflation
(54–144% above anchor). This is consistent with uniform CPU starvation across all
iterations from the extreme system load — the entire measurement distribution is
pulled up rather than just the tail.

---

## Wave Delta Cold-Path Assessment

**Commits in delta (b642c0f..b5e1e15):**

| Commit | Type | Files changed | Hot-path impact |
|--------|------|--------------|-----------------|
| b5e1e15 | docs | docs/ only | None |
| f1e0c36 | merge | release/0.12.0 | None |
| 72a2842, 795fc9d, c8488be | release/chore/docs | Cargo.toml version, docs | None (version string only) |
| e3ca2bc | docs | docs/ only | None |
| 716054a | chore(deps) | Cargo.toml/Cargo.lock (indicatif bump) | None |
| 44f8c9c | ci | .github/workflows only | None |
| 80fbb64 | docs | docs/ only | None |
| 704fd2e | feat(reporter) | src/reporter/json.rs, src/findings.rs, src/analyzer/arp.rs (comment) | None — cold serialization path only |
| d410b8d | docs | docs/ only | None |
| 75c5ba5 | ci | .github/workflows only | None |
| c4eb1f4 | refactor | src/ (109 saturating_add conversions on diagnostic counters) | None — overflow-only paths not exercised by benchmark fixtures |

**Conclusion: The full delta b642c0f..b5e1e15 is cold-path for all benchmarks.
No hot-path code changed. All apparent regressions in this run are attributable
entirely to machine load/thermal noise.**

---

## Results Table — pipeline bench

All times are Criterion point estimates (median of CI) in µs.
Authoritative anchors: May-19 original (23.281 µs for reassembly/tls.pcap) and
Jun-22 controlled re-run (the column labeled "Jun-22 anchor").

| Benchmark | Fixture | Jun-22 anchor (µs) | Today 2026-07-11 (µs) | Criterion CI | Outliers (severe) | vs Jun-22 | Classification |
|-----------|---------|-------------------|----------------------|-------------|------------------|-----------|----------------|
| decode | segmented.pcap | 1.459 | 3.3310 | [2.931–3.795] | 4 (0) | +128.3% | CRITICAL (NOISE-SUSPECT) |
| decode | tls.pcap | 3.369 | 8.2342 | [6.588–10.035] | 9 (4) | +144.4% | CRITICAL (NOISE-SUSPECT) |
| decode | dns-remoteshell.pcap | 4.840 | 9.6365 | [8.494–10.911] | 12 (7) | +99.1% | CRITICAL (NOISE-SUSPECT) |
| summary | segmented.pcap | 0.639 | 1.1149 | [0.998–1.233] | 8 (4) | +74.5% | CRITICAL (NOISE-SUSPECT) |
| summary | dns-remoteshell.pcap | 2.589 | 4.3494 | [3.855–4.875] | 5 (1) | +68.0% | CRITICAL (NOISE-SUSPECT) |
| reassembly | segmented.pcap | 5.858 | 9.0131 | [8.223–9.910] | 6 (1) | +53.9% | CRITICAL (NOISE-SUSPECT) |
| reassembly | tls.pcap | 24.429 | 40.388 | [36.225–44.784] | 5 (3) | +65.3% | CRITICAL (NOISE-SUSPECT) |

Criterion verdicts vs stored base (prior noisy Jul-09 run):
- `decode/segmented.pcap`: Performance has regressed (+93.3%, p<0.05)
- `decode/tls.pcap`: Performance has regressed (+33.4%, p<0.05)
- `decode/dns-remoteshell.pcap`: Performance has regressed (+75.1%, p<0.05)
- `summary/segmented.pcap`: Performance has improved (−44.0%, p<0.05) — prior stored base was itself noisy
- `summary/dns-remoteshell.pcap`: No change detected (p=0.20) — wide CI straddles stored base
- `reassembly/segmented.pcap`: Performance has regressed (+26.4%, p<0.05)
- `reassembly/tls.pcap`: Performance has regressed (+66.7%, p<0.05)

The `summary/segmented.pcap` showing "improved" vs a prior noisy base exemplifies why
criterion verdicts from noisy runs are meaningless — the stored base it compared against
was itself a noise inflation. All criterion verdicts this run are non-actionable.

---

## Results Table — tls_fragmented bench

| Benchmark | Fixture | Jul-08 initial baseline (µs) | Today 2026-07-11 (µs) | Criterion CI | Outliers (severe) | vs Jul-08 | Classification |
|-----------|---------|------------------------------|----------------------|-------------|------------------|-----------|----------------|
| tls_fragmented | 3-record-carry-drain | 2.0662 | 3.4825 | [2.908–4.083] | 13 (3) | +68.5% | CRITICAL (NOISE-SUSPECT) |

Note: The Jul-08 initial baseline (2.0662 µs) was itself recorded under noisy conditions
(18/100 severe outliers) and has never been validated as a reliable anchor. The Jul-09
runs measured 1.7004 µs and 1.8823 µs respectively — both below the Jul-08 initial,
showing the Jul-08 anchor is unreliable in either direction. Today's 3.4825 µs is
inflated by machine load. No tls_fragmented anchor value is reliable until a
quiescent run is completed.

---

## AC-149-003 Metric (reassembly/tls.pcap)

The primary regression-tracking metric. AC-149-003 requires ≤ 24.445 µs (May-19 × 1.05).

| Measurement | Value (µs) | vs May-19 anchor | vs AC-149-003 ceiling | AC-149-003 |
|-------------|-----------|-----------------|----------------------|------------|
| May-19 anchor | 23.281 | 0.0% | −5.0% (below ceiling) | PASS |
| AC-149-003 ceiling | 24.445 | +5.0% | — | — |
| Jun-22 controlled re-run | 24.429 | +4.9% | −0.07% (below ceiling) | PASS |
| story149-pre (2026-07-07) | 25.880 | +11.2% | +5.9% (above) | FAIL |
| maint-2026-07-08 (b642c0f) | 26.353 | +13.2% | +7.8% (above) | FAIL (NOISE-SUSPECT) |
| maint-2026-07-09 run 1 | 25.698 | +10.4% | +5.1% (above) | FAIL (NOISE-SUSPECT) |
| maint-2026-07-09 run 2 | 24.075 | +3.4% | −1.5% (below) | PASS (NOISE-SUSPECT) |
| Today 2026-07-11 | 40.388 | +73.5% | +65.3% (above) | FAIL (NOISE-SUSPECT) |

**Finding:** Today's 40.388 µs result is far above the AC-149-003 ceiling, but is
entirely attributable to the extreme machine load (load avg 52.57). No code touching
the TLS reassembly hot path changed since b642c0f. AC-149-003 status remains
INDETERMINATE — the Jun-22 controlled re-run (24.429 µs) and the maint-2026-07-09
run 2 (24.075 µs) both passed the ceiling under less-noisy conditions, but neither
represents a fully quiescent, reproducible controlled run.

---

## PERF-002 Status (reassembly/tls.pcap vs Jun-22 anchor of 24.429 µs)

PERF-002 was RESOLVED by the Jun-22 controlled re-run confirming the TLS reassembly
path performance near the May-19 anchor. The question for this sweep was: does it
stay resolved?

**Finding: PERF-002 resolution cannot be confirmed or reversed from this noisy run.**
Today's 40.388 µs is clearly machine-load noise. The Jun-22 controlled re-run (24.429 µs)
and prior maint measurements (24.075–26.353 µs band) are the relevant reference range.
There is no evidence of a code change causing a real regression on the TLS path.
PERF-002 remains RESOLVED based on the Jun-22 evidence. Confirm with next quiescent run.

---

## Noise Diagnosis: Why All CRITICAL Flags Are NOISE-SUSPECT

1. **Extreme machine load is the root cause.** Load averages 52.57/46.80/35.41 mean the
   CPU is ~52× oversubscribed relative to idle. Every benchmark iteration experiences
   context switches and CPU scheduler delays, uniformly inflating the entire distribution.

2. **Low severe-outlier counts mask the problem.** Prior noisy runs (Jul-08, Jul-09) had
   9–21 severe outliers per 100 samples. This run has 0–7 severe outliers — but the
   median is far higher (65–144% above anchor). Uniform CPU starvation lifts the entire
   distribution rather than creating tail outliers; the Criterion outlier detector cannot
   flag the baseline inflation.

3. **Cold-path delta.** No hot-path code changed since b642c0f (the Jul-08 baseline
   commit). Any regression is noise.

4. **Wide confidence intervals confirm instability.** decode/tls.pcap CI width: 42%
   ([6.59–10.04 µs]). reassembly/tls.pcap CI width: 21% ([36.23–44.78 µs]).
   Clean runs produce CI widths of 0.5–3%.

---

## PERF-RERUN-001 Assessment

**PERF-RERUN-001: CANNOT CLOSE — machine severely loaded at run time.**

PERF-RERUN-001 was registered after the maint-2026-07-08 run (11–20 severe outliers,
thermal/scheduling contamination) as requiring a controlled re-run with the machine
quiescent. This sweep's run conditions are worse: load average 52.57 vs the
Jul-08/Jul-09 runs which showed 9–21 severe outliers. The present run exhibits
a different noise mode (uniform distribution inflation vs tail outliers) but is
equally or more unreliable.

Requirements to close PERF-RERUN-001:
- Load averages below ~3.0 (one-minute load)
- OrbStack VM manager idle or stopped
- Node processes in website-cbb worktrees terminated or suspended
- Chrome and other heavy background processes closed
- Criterion outlier count ≤ 6 per 100 samples across all pipeline groups

Until those conditions are met, Jun-22 (24.429 µs) remains the sole authoritative
reassembly/tls.pcap anchor.

---

## Open Tidy Candidates (not addressed this sweep)

PERF-003, PERF-004, PERF-005: Open allocation pattern improvements in tls.rs.
These were identified in prior maintenance sweeps as potential optimizations in the
TLS reassembly path. Not addressed this sweep per task scope. No code changes made.

---

## Long-Run Trend Table (reassembly/tls.pcap — primary regression-tracking metric)

| Date | Commit | µs (slope/mean) | vs May-19 | Notes |
|------|--------|-----------------|-----------|-------|
| 2026-05-19 | (anchor) | 23.281 | 0.0% | Original baseline |
| 2026-06-17 | (maint) | 35.960 | +54.5% | NOISE — confirmed thermal spike |
| 2026-06-22 | (maint) | 24.429 | +4.9% | Controlled re-run; authoritative anchor |
| 2026-07-06 | f7460b4 | 27.842 | +19.6% | v0.11.4 (maint-2026-07-06) |
| 2026-07-07 | 19569ae | 25.880 | +11.2% | STORY-149 pre-story anchor (v0.11.5) |
| 2026-07-08 | b642c0f | 26.353 | +13.2% | maint-2026-07-08; high outliers — NOISE-SUSPECT |
| 2026-07-09 run 1 | 716054a | 25.698 | +10.4% | maint-2026-07-09; high outliers — NOISE-SUSPECT |
| 2026-07-09 run 2 | 716054a | 24.075 | +3.4% | maint-2026-07-09; moderate outliers — NOISE-SUSPECT |
| 2026-07-11 | b5e1e15 | 40.388 | +73.5% | This sweep; machine load avg 52.57 — NOISE-SUSPECT |

The Jul-09 run 2 (24.075 µs) is the closest data point to a passing AC-149-003 result
since Jun-22. It and the Jun-22 run bracket the expected quiescent performance in the
24.0–24.5 µs range. The 25–26 µs values in other runs likely reflect mild machine
contention. The 35–40 µs values (Jun-17, today) reflect severe contamination.

---

## Confirmed Real Regressions (persisting from Jun-22 controlled re-run)

These were confirmed in the Jun-22 controlled re-run and remain valid regardless of today's noisy data:

| Metric | May-19 anchor (µs) | Jun-22 confirmed (µs) | Delta | Classification |
|--------|-------------------|-----------------------|-------|----------------|
| decode/tls.pcap | 3.002 | 3.369 | +12.2% | REGRESSION-MINOR (real, stable, ARP overhead) |
| reassembly/segmented.pcap | 4.907 | 5.858 | +19.4% | REGRESSION-MINOR (real, stable, ARP overhead) |

Both are attributable to the ARP feature cycle. Neither worsened in any confirmed clean
run. No fix PRs warranted — no numerical NFR latency target exists.

---

## NFR Compliance Matrix

| NFR ID | Requirement | Status |
|--------|-------------|--------|
| NFR-PERF-001 | Zero-copy slice path; one allocation per packet | DEFERRED — not measured by microbenchmarks |
| NFR-PERF-002 | Eager full-pcap load; RAM <= pcap_size * 1.5 | DEFERRED — no 1 GB fixture |
| NFR-PERF-003 | O(1) dispatch; 100% cache hit rate after first classification | DEFERRED — no 10,000-flow fixture |
| NFR-PERF-004 | SIMD autovectorization in overlap detection | OPEN-DEBT — LLVM IR not inspected this sweep |

---

## Summary Table

| Metric | Classification | Reason |
|--------|----------------|--------|
| All pipeline benchmarks vs Jun-22 | CRITICAL (NOISE-SUSPECT) | Machine load avg 52.57; cold-path delta |
| reassembly/tls.pcap vs Jun-22 (24.429 µs) | CRITICAL (NOISE-SUSPECT) | 40.388 µs; machine noise |
| AC-149-003 (reassembly/tls.pcap ≤ 24.445 µs) | INDETERMINATE | Cannot adjudicate under these conditions |
| PERF-002 (tls.pcap stays near Jun-22) | CANNOT CONFIRM — noise run | Resolution stands from Jun-22 evidence |
| PERF-RERUN-001 | OPEN — cannot close | Machine load avg 52.57; conditions not met |
| tls_fragmented anchor | UNRELIABLE | No clean run in history; Jul-08 initial was noisy; today noisy |
| Confirmed real regressions (decode/tls, rea/seg) | REGRESSION-MINOR (unchanged) | Stable ARP overhead from prior sweeps |
| New regressions from wave delta | NONE | Delta is cold-path only |

---

## Recommendations

1. **Do not update the authoritative anchor.** Jun-22 controlled re-run values remain
   the only reliable baseline. This data must not displace them.

2. **PERF-RERUN-001 remains open.** To close it, run `cargo bench` with: OrbStack
   stopped (or VM powered off), all website-cbb node processes terminated, Chrome
   closed. Target: load avg < 3.0, severe outliers ≤ 6 per 100 samples.

3. **AC-149-003 status is INDETERMINATE.** The best available evidence (Jul-09 run 2 at
   24.075 µs, Jun-22 at 24.429 µs) is consistent with the ceiling being met, but no
   fully clean run has confirmed it since Jun-22. The next quiescent run will settle this.

4. **No fix PRs warranted.** All CRITICAL flags are machine-noise artifacts. The wave
   delta is cold-path.

5. **PERF-003/004/005 (tls.rs allocation tidy candidates) remain open** and were not
   addressed this sweep.

---

## Sweep Metadata

| Field | Value |
|-------|-------|
| Run date | 2026-07-11 |
| Platform | darwin 25.5.0 (Apple Silicon, macOS) |
| Rust toolchain | stable (v0.12.0) |
| HEAD commit | b5e1e15 (develop, v0.12.0) |
| Benchmark command | `cargo bench --bench pipeline && cargo bench --bench tls_fragmented` |
| Severe outlier range | 0–7 per 100 samples (lower count but uniform distribution inflation) |
| Machine load at run time | 52.57 / 46.80 / 35.41 (1m / 5m / 15m) — SEVERELY LOADED |
| Thermal state | NOT quiescent — heavy competing load (OrbStack, Node, Chrome) |
| PERF-RERUN-001 | OPEN — machine conditions not met for controlled re-run |
