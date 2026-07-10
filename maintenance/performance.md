---
document_type: maintenance-performance-report
sweep: 7
run_id: maint-2026-07-09
producer: performance-engineer
created: 2026-07-09
branch: develop
commit: 716054a
version: v0.11.5
baseline_source: .factory/maintenance/performance-baseline.md (maint-2026-06-22 controlled re-run values)
current_run_date: 2026-07-09
hardware_note: >
  Apple Silicon Mac, darwin 25.5.0. All measurements are wall-clock on the
  benchmark machine. Absolute µs values are not portable across hardware;
  only relative deltas (same machine, same branch) are meaningful for
  regression tracking.
benchmark_command: cargo bench --bench pipeline && cargo bench --bench tls_fragmented
criterion_version: "0.8"
samples: 100 per benchmark
---

# Performance Regression Scan — maint-2026-07-09

## Executive Summary

Two passes of both benchmark suites (`pipeline` and `tls_fragmented`) were run at develop
HEAD (716054a, v0.11.5). The machine exhibited high severe-outlier counts across all
benchmarks (9–21 severe outliers per 100 samples) and extreme run-to-run point-estimate
swings (up to 74% between consecutive runs). This is the same noise signature observed in
the maint-2026-07-08 run (Sweep 6). The machine was not quiescent during either run.

**Overall verdict: NOISE-SUSPECT — no actionable regressions from the wave-72 delta.**

The wave-72 delta (c4eb1f4..716054a) is **provably cold-path** for all benchmarks in scope
(see Wave-72 Cold-Path Assessment below). No hot-path code changed between the maint-2026-07-08
baseline commit (b642c0f) and HEAD (716054a). All apparent regressions and improvements in
this run reflect machine scheduling/thermal noise, not code changes.

The primary AC-149-003 metric (`reassembly/tls.pcap`) measured 25.698 µs in run 1 and
24.075 µs in run 2 — a 6.3% spread between consecutive runs. Run 2 is just below the
AC-149-003 ceiling (24.445 µs); run 1 is 5.1% above it. The ambiguity cannot be resolved
under current machine conditions. A quiescent controlled re-run remains the prerequisite for
any definitive AC-149-003 assessment.

---

## Wave-72 Cold-Path Assessment

**Commits in wave-72 delta (c4eb1f4..716054a):**

| Commit | Type | Files changed |
|--------|------|--------------|
| 716054a | chore(deps) | indicatif version bump (Cargo.toml/Cargo.lock) |
| 44f8c9c | ci | .github/workflows only |
| 80fbb64 | docs | docs/ only |
| 704fd2e | feat(reporter) | `src/reporter/json.rs` (+SCHEMA_VERSION const, envelope wiring), `src/findings.rs` (+3 serde annotations), `src/analyzer/arp.rs` (comment-only, 2 lines) |
| d410b8d | docs | docs/ only |
| 75c5ba5 | ci | .github/workflows only |

**Assessment:** None of the wave-72 commits touch the decode, reassembly, or summary
hot paths exercised by the pipeline benchmark groups. The reporter JSON path (`src/reporter/json.rs`)
is a cold serialization path invoked only at output time, not inside the per-packet benchmark
loop. The `src/findings.rs` serde annotations affect only JSON serialization. The `src/analyzer/arp.rs`
change is a code comment update with zero runtime impact.

**Conclusion: The wave-72 delta is cold-path only. Any regression or improvement signal in
this run is attributable entirely to machine noise, not to code changes in this delta.**

---

## Benchmark Infrastructure

| Item | Value |
|------|-------|
| Harness | criterion 0.8 |
| Config | `[[bench]] name = "pipeline" harness = false` and `[[bench]] name = "tls_fragmented" harness = false` (Cargo.toml) |
| Fixture files | `tests/fixtures/{segmented.pcap, tls.pcap, dns-remoteshell.pcap}` |
| Benchmark groups | decode, summary, reassembly (pipeline); tls_fragmented (tls_fragmented) |
| Samples per benchmark | 100 |
| Baseline source | performance-baseline.md (2026-06-22 controlled re-run) |
| AC-149-003 anchor | 23.281 µs (May-19); target ceiling 24.445 µs |

---

## Results Table — pipeline bench (both runs)

All times are Criterion point estimates (median of CI) in µs.
Baselines: May-19 original (23.281 / 1.440 / 3.002 / 4.472 / 0.600 / 2.535 / 4.907 for rea/tls, dec/seg, dec/tls, dec/dns, sum/seg, sum/dns, rea/seg respectively) and Jun-22 controlled re-run (authoritative anchor).

| Benchmark | Fixture | Jun-22 anchor (µs) | Run 1 2026-07-09 (µs) | Run 2 2026-07-09 (µs) | vs Jun-22 R1 | vs Jun-22 R2 | R1 Outliers | R2 Outliers | Verdict |
|-----------|---------|-------------------|----------------------|----------------------|-------------|-------------|-------------|-------------|---------|
| decode | segmented.pcap | 1.459 | 1.6286 | 1.5773 | +11.6% | +8.1% | 13 (9 sev) | 13 (9 sev) | NOISE-SUSPECT |
| decode | tls.pcap | 3.369 | 3.9916 | 5.9107 | +18.5% | +75.5% | 14 (12 sev) | 10 (5 sev) | NOISE-SUSPECT |
| decode | dns-remoteshell.pcap | 4.840 | 5.0631 | 6.7779 | +4.6% | +40.0% | 16 (12 sev) | 19 (14 sev) | NOISE-SUSPECT |
| summary | segmented.pcap | 0.639 | 0.9114 | 1.5860 | +42.6% | +148.2% | 21 (17 sev) | 17 (12 sev) | NOISE-SUSPECT |
| summary | dns-remoteshell.pcap | 2.589 | 3.4875 | 4.3876 | +34.7% | +69.4% | 18 (16 sev) | 12 (4 sev) | NOISE-SUSPECT |
| reassembly | segmented.pcap | 5.858 | 7.1911 | 6.9739 | +22.8% | +19.0% | 19 (17 sev) | 16 (11 sev) | NOISE-SUSPECT |
| reassembly | tls.pcap | 24.429 | 25.698 | 24.075 | +5.2% | −1.4% | 13 (11 sev) | 9 (5 sev) | NOISE-SUSPECT |

**Criterion verdicts (run 1 vs run 2 stored base):**

The run-2 criterion comparisons are against the run-1 Criterion stored base, producing
nonsensical results that confirm machine noise:
- `decode/tls.pcap`: +40.4% regressed (run-2 vs run-1)
- `summary/segmented.pcap`: +146% regressed (run-2 vs run-1)
- `decode/segmented.pcap`: no change (p=0.26)
- `reassembly/tls.pcap`: −11.3% improved (run-2 vs run-1)

Opposite sign signals on consecutive runs with no code change are the definitive
noise fingerprint. No criterion verdict from this sweep is actionable.

---

## Results Table — tls_fragmented bench (both runs)

| Benchmark | Fixture | Jul-08 initial baseline (µs) | Run 1 2026-07-09 (µs) | Run 2 2026-07-09 (µs) | vs Jul-08 R1 | vs Jul-08 R2 | R1 Outliers | R2 Outliers | Verdict |
|-----------|---------|------------------------------|----------------------|----------------------|-------------|-------------|-------------|-------------|---------|
| tls_fragmented | 3-record-carry-drain | 2.0662 | 1.7004 | 1.8823 | −17.7% | −8.9% | 16 (9 sev) | 13 (13 sev) | NOISE-SUSPECT |

The Jul-08 initial baseline (2.0662 µs) was itself recorded under the same high-noise
conditions (18/100 severe outliers) and was flagged as unreliable for use as a regression
comparator. Both today's runs fall below it, but the 10.7% run-to-run spread makes the
absolute values unreliable. A quiescent re-run is needed to establish a clean tls_fragmented
anchor.

---

## AC-149-003 Status (reassembly/tls.pcap target: ≤ 24.445 µs)

| Measurement | Value (µs) | vs May-19 | vs Target | AC-149-003 |
|-------------|-----------|-----------|-----------|------------|
| May-19 anchor | 23.281 | 0.0% | −5.0% (below) | PASS |
| AC-149-003 ceiling | 24.445 | +5.0% | — | — |
| story149-pre (2026-07-07) | 25.880 | +11.2% | +5.9% (above) | FAIL |
| Jul-08 run (b642c0f) | 26.353 | +13.2% | +7.8% (above) | FAIL |
| Today Run 1 (716054a) | 25.698 | +10.4% | +5.1% (above) | FAIL |
| Today Run 2 (716054a) | 24.075 | +3.4% | −1.5% (below) | PASS |

The ambiguity (run 1 FAIL at 25.698 µs, run 2 PASS at 24.075 µs) cannot be resolved
under current machine conditions. The 6.3% run-to-run spread on the most stable benchmark
in this suite confirms the machine is not suitable for AC-149-003 adjudication today.

---

## Noise Diagnosis

The key diagnostics that confirm this is machine noise rather than a real regression:

1. **Run-to-run spread exceeds the regression threshold.** `decode/tls.pcap` swung from
   3.99 µs (run 1) to 5.91 µs (run 2) — a 48% difference with no code change between runs.
   `summary/segmented.pcap` swung from 0.91 µs to 1.59 µs (+74%). Genuine regressions do
   not produce 48–74% swings between back-to-back runs.

2. **All benchmark groups affected simultaneously.** The decode, summary, and reassembly
   groups are independent code paths. Simultaneous elevation across all three, combined with
   high severe-outlier counts in every group, is the canonical OS scheduling or thermal
   interference fingerprint — identical to the maint-2026-07-08 run.

3. **Wave-72 delta is cold-path only.** No hot-path code changed since b642c0f (the
   maint-2026-07-08 baseline commit). Any regression must be noise.

4. **Reassembly/tls.pcap is the most stable across both runs** (−6.3% spread), consistent
   with prior observations that TLS bench outlier counts are lower when the machine is
   less degraded.

---

## Non-TLS Path Controlled Re-run Assessment

The maint-2026-07-08 report (Sweep 6) recommended a "controlled re-run for non-TLS paths."
This sweep executed that re-run under the same noisy-machine conditions. The non-TLS
paths (`decode/segmented.pcap`, `decode/dns-remoteshell.pcap`, `summary/segmented.pcap`,
`summary/dns-remoteshell.pcap`) all showed extreme run-to-run variance (26–74% swings)
that cannot be attributed to code changes. The controlled re-run recommendation stands:
the machine must be quiescent (background processes minimized, thermal state stable)
before non-TLS path results can be trusted.

---

## Long-Run Trend Table (reassembly/tls.pcap — primary regression-tracking metric)

| Date | Commit | µs (slope/mean) | vs May-19 | Notes |
|------|--------|-----------------|-----------|-------|
| 2026-05-19 | (anchor) | 23.281 | 0.0% | Original baseline |
| 2026-06-17 | (maint) | 35.960 | +54.5% | NOISE — confirmed thermal spike |
| 2026-06-22 | (maint) | 24.429 | +4.9% | Controlled re-run; noise resolved |
| 2026-07-06 | f7460b4 | 27.842 | +19.6% | v0.11.4 (maint-2026-07-06) |
| 2026-07-07 | 19569ae | 25.880 | +11.2% | STORY-149 pre-story anchor (v0.11.5) |
| 2026-07-08 | b642c0f | 26.353 | +13.2% | maint-2026-07-08; high outliers — NOISE-SUSPECT |
| 2026-07-09 run 1 | 716054a | 25.698 | +10.4% | This sweep; high outliers — NOISE-SUSPECT |
| 2026-07-09 run 2 | 716054a | 24.075 | +3.4% | This sweep; moderate outliers — NOISE-SUSPECT |

The long-run trend for `reassembly/tls.pcap` remains in the 24–26 µs band, with no step-change
attributable to any specific commit in the b642c0f–716054a range. The ARP-cycle overhead
(+~11% over May-19) persists as the baseline elevation identified in prior sweeps.

---

## NFR Compliance Matrix

| NFR ID | Requirement | Status |
|--------|-------------|--------|
| NFR-PERF-001 | Zero-copy slice path; one allocation per packet | DEFERRED — not measured by microbenchmarks |
| NFR-PERF-002 | Eager full-pcap load; RAM <= pcap_size * 1.5 | DEFERRED — no 1 GB fixture |
| NFR-PERF-003 | O(1) dispatch; 100% cache hit rate after first classification | DEFERRED — no 10,000-flow fixture |
| NFR-PERF-004 | SIMD autovectorization in overlap detection | OPEN-DEBT — LLVM IR not inspected this sweep |

---

## Recommendations

1. **No fix PRs warranted from this sweep.** Wave-72 delta is provably cold-path; all
   apparent regressions are machine noise. Do not open performance fix tickets based on
   this data.

2. **A quiescent controlled re-run is the prerequisite for AC-149-003 adjudication.**
   The 6.3% run-to-run spread on `reassembly/tls.pcap` straddles the AC-149-003 ceiling
   (24.445 µs). Until a clean run is available, AC-149-003 status is INDETERMINATE.

3. **tls_fragmented baseline remains unreliable.** Both the Jul-08 initial recording
   and today's runs were noisy. A quiescent run is needed to establish a clean anchor
   before this bench can function as a regression gate.

4. **Do not update performance-baseline.md.** Noisy data must not become the new anchor.
   The Jun-22 controlled re-run values remain the authoritative baseline until a clean
   quiescent run produces stable measurements.

5. **Prior REGRESSION-MINOR findings (decode/tls.pcap +12.2%, reassembly/segmented.pcap
   +19.4% vs May-19) remain unchanged.** Both were confirmed real and attributable to the
   ARP feature cycle. Nothing in the wave-72 delta affects these paths.

---

## Sweep Metadata

| Field | Value |
|-------|-------|
| Run date | 2026-07-09 |
| Platform | darwin 25.5.0 (Apple Silicon, macOS) |
| Rust toolchain | stable (v0.11.5) |
| Benchmark command | `cargo bench --bench pipeline` (×2), `cargo bench --bench tls_fragmented` (×2) |
| Commits since Jun-22 baseline | 716054a (v0.11.5, wave-72 delta over b642c0f) |
| Outlier profile | 9–21 severe outliers per 100 samples across all groups; highest in summary/* |
| Thermal state | Machine not quiescent; high noise — all verdicts NOISE-SUSPECT |
| Wave-72 hot-path impact | None — delta is cold-path only (reporter/json.rs, findings.rs serde, arp.rs comment) |
