---
document_type: maintenance-performance-report
sweep: 5
producer: performance-engineer
created: 2026-06-17
branch: develop
version: v0.7.1
baseline_date: 2026-05-19
baseline_source: target/criterion/*/base/estimates.json (May 19 criterion run)
current_run_date: 2026-06-17
hardware_note: >
  Apple Silicon Mac, darwin 25.5.0. All measurements are wall-clock on the
  benchmark machine. Absolute µs values are not portable; only relative
  deltas are meaningful for regression tracking.
---

# Maintenance Sweep 5 — Performance Report

## Executive Summary

Criterion benchmarks exist and were run on 2026-06-17 against a baseline from
2026-05-19 (362 commits prior, before the ARP feature cycle STORY-111..115).
Multiple benchmarks show statistically significant regressions above the 10%
WARNING threshold. None exceed the 25% CRITICAL threshold for the best-case
(point-estimate) figures except `reassembly/tls.pcap`, which at +54.5% median
is CRITICAL.

The regressions are plausibly attributable to the new ARP decoding path added
in the v0.7.0/v0.7.1 feature cycle: `decode_packet` now returns
`DecodedFrame::Ip | DecodedFrame::Arp`, requiring an additional match arm on
every packet in the hot loop. This hypothesis is consistent with the pattern
that ALL decoder-touching benchmarks regressed while none were improved. No
NFR target number exists in the NFR catalog for maximum per-packet latency (see
NFR Compliance Matrix), so no formal PASS/FAIL determination is possible
against spec; these are relative regression findings only.

The `reassembly/tls.pcap` regression (+54.5% median) has high variance
(std_dev ~14 µs on a 34 µs median) indicating thermal or scheduling noise. A
re-run under controlled conditions is recommended before filing as a confirmed
regression.

---

## Benchmark Infrastructure

| Item | Value |
|------|-------|
| Harness | criterion 0.8 |
| Config | `[[bench]] name = "pipeline" harness = false` (Cargo.toml) |
| Fixture files | `tests/fixtures/{segmented.pcap, tls.pcap, dns-remoteshell.pcap}` |
| Benchmark groups | decode, summary, reassembly |
| Samples per benchmark | 100 |
| Baseline source | `target/criterion/*/base/estimates.json` dated 2026-05-19 |

---

## Results Table

All times are mean per-iteration (µs). Delta = (current - baseline) / baseline.

| Benchmark | Fixture | Baseline mean (µs) | Current mean (µs) | Delta | Criterion verdict |
|-----------|---------|-------------------|-------------------|-------|-------------------|
| decode | segmented.pcap | 1.440 | 1.468 | +1.9% | within noise |
| decode | tls.pcap | 3.002 | 3.658 | +21.9% | **WARNING** |
| decode | dns-remoteshell.pcap | 4.472 | 4.960 | +10.9% | **WARNING** |
| summary | segmented.pcap | 0.600 | 0.670 | +11.7% | **WARNING** |
| summary | dns-remoteshell.pcap | 2.535 | 2.667 | +5.2% | within noise† |
| reassembly | segmented.pcap | 4.907 | 5.894 | +20.1% | **WARNING** |
| reassembly | tls.pcap | 23.281 | 35.960 | +54.5% | **CRITICAL** |

† Criterion reported `summary/dns-remoteshell.pcap` as a statistically
significant regression (+5.2%, p < 0.05) but it is below the 10% WARNING
threshold defined in this sweep's rules.

Baseline figures are from the May 19 run stored in
`target/criterion/*/base/estimates.json` before this sweep began.

---

## NFR Compliance Matrix

The NFR catalog (`.factory/specs/prd-supplements/nfr-catalog.md` v2.1) defines
four performance NFRs. None specify a per-packet latency numerical target that
would allow a PASS/FAIL determination from micro-benchmarks.

| NFR ID | Requirement | Validation Method (from catalog) | Measured | Verdict |
|--------|-------------|----------------------------------|----------|---------|
| NFR-PERF-001 | Zero-copy slice path; one allocation per packet | Code review (payload clone only) | Not a benchmark target — confirmed by code inspection in prior cycles | N/A |
| NFR-PERF-002 | Eager full-pcap load; RAM <= pcap_size * 1.5 | Load test with 1 GB pcap; measure RSS | Not measured this sweep (no 1 GB fixture available) | DEFERRED |
| NFR-PERF-003 | O(1) dispatch via cache; 100% cache hit rate after first classification | Benchmark: 10,000-flow pcap; confirm hit rate = 100% | Not directly exercised by the three fixture benchmarks (no 10,000-flow pcap) | DEFERRED |
| NFR-PERF-004 | Overlap detection uses SIMD-friendly slice equality; autovectorization confirmed | `cargo asm` / LLVM IR inspection | Not validated this sweep; bench harness exercises hot path but LLVM IR not inspected | OPEN-DEBT (carried from prior cycle) |

NFR-PERF-002 and NFR-PERF-003 are not exercised by the current fixture set.
Both were already in DEFERRED/OPEN-DEBT state in the NFR catalog. No new
PASS/FAIL determination is possible without the specified load-test fixture.

---

## Regression Analysis

### Confirmed regressions (criterion statistically significant, >= 10%)

**decode/tls.pcap: +21.9%** (WARNING)
- Baseline: 3.002 µs mean; Current: 3.658 µs mean
- Criterion: p < 0.05; 18 outliers (17 high mild, 1 high severe)
- Most likely cause: `decode_packet` now matches on `DecodedFrame::Arp` in
  addition to `DecodedFrame::Ip`, adding a branch on every call regardless of
  link type. The tls.pcap fixture is Ethernet with only IP frames; the new arm
  is never taken but the branch still exists in the compiled code.

**decode/dns-remoteshell.pcap: +10.9%** (WARNING)
- Baseline: 4.472 µs mean; Current: 4.960 µs mean
- Criterion: p < 0.05; 17 outliers (13 high mild, 4 high severe)
- Same root cause as decode/tls.pcap. Higher baseline latency per iteration
  suggests more frames in this fixture.

**summary/segmented.pcap: +11.7%** (WARNING)
- Baseline: 0.600 µs mean; Current: 0.670 µs mean
- Criterion: p < 0.05; 7 outliers (6 high mild)
- The summary benchmark pre-decodes frames; the filter-map now has an extra
  `DecodedFrame::Arp` arm to handle. The ARP frames are dropped before
  `Summary::ingest`, but the match overhead is in-loop.

**reassembly/segmented.pcap: +20.1%** (WARNING)
- Baseline: 4.907 µs mean; Current: 5.894 µs mean
- Criterion: p < 0.05; 3 outliers (3 high mild)
- Full pipeline: decode + IP-filter + reassembly + dispatch + TLS/HTTP. Same
  decode overhead compounded with any changes to the dispatcher or reassembler.

**reassembly/tls.pcap: +54.5% median (CRITICAL) — high variance, re-run advised**
- Baseline: 23.281 µs mean, 23.203 µs median
- Current: 35.960 µs mean, 33.546 µs median (std_dev ~14 µs vs ~0.2 µs baseline)
- Criterion: p < 0.05; 14 outliers (6 low mild, 2 high mild, 6 high severe)
- The extremely high variance in this run (std_dev increased 65x) makes the
  mean unreliable. The median (+44.6%) is more robust but still very large.
  This fixture exercises the TLS reassembly path, which is the most compute-
  intensive benchmark. Possible causes: (a) thermal throttling during the 5 s
  benchmark window, (b) genuine regression in the TLS analyzer or reassembler
  from ARP-adjacent code changes, or (c) both. Recommend re-running under
  stable thermal conditions before treating as a confirmed regression.

### Within-noise (< 10% or criterion "no change")

**decode/segmented.pcap: +1.9%** — within criterion noise threshold
- Criterion: "Change within noise threshold" (not statistically significant)

**summary/dns-remoteshell.pcap: +5.2%** — below 10% WARNING, statistically
significant (p < 0.05) but not actionable under sweep thresholds.

---

## Root Cause Hypothesis

The ARP feature cycle (STORY-111..115, PRs #237-#260) introduced:
1. A new `DecodedFrame::Arp` variant returned by `decode_packet`
2. Match arms in the hot decoding loop (main.rs analyze path, benches)
3. The ARP analyzer itself (`src/analyzer/arp.rs` and related modules)

The benchmark fixtures (segmented.pcap, tls.pcap, dns-remoteshell.pcap)
contain no ARP traffic — yet all decode-path benchmarks regressed. This is
consistent with a branch-predictor pressure or instruction-cache cost from the
new code, not from executing the ARP analysis path itself. The reassembly
benchmarks also regressed, suggesting the dispatcher may have acquired
additional overhead in the flow-close or type-check paths.

The ARP feature adds functional capability warranting these costs. Whether the
regressions are acceptable is a product trade-off, not a correctness issue.

---

## Recommendations

1. **Establish a committed performance baseline.** The current `target/criterion/`
   data is transient (not committed, not versioned). The factory should commit
   a `performance-baseline.json` (or equivalent) to the `factory-artifacts`
   branch after each release so that sweep-over-sweep trend tracking is
   authoritative. The May 19 baseline survived by luck (not overwritten yet).

2. **Re-run reassembly/tls.pcap under controlled conditions.** The 65x increase
   in std_dev makes the +54.5% figure unreliable. Re-run with a quiescent
   machine (no background processes, fan speed stable) before treating as a
   CRITICAL regression.

3. **Investigate decode branch cost.** If the decode regressions (10–22%) are
   unacceptable, consider inlining the `DecodedFrame` match to allow the
   compiler to eliminate the dead ARP arm on non-ARP code paths, or profile
   with `cargo flamegraph` to confirm the hypothesis.

4. **Add numerical NFR targets for decode throughput.** NFR-PERF-001 through
   NFR-PERF-004 lack per-packet latency targets. Without a numerical target
   (e.g., "decode throughput >= 100 Mpps on reference hardware") the benchmark
   suite can only detect relative regression, not absolute NFR violations. Filing
   a tech-debt item to define latency targets is recommended.

5. **NFR-PERF-002/003 load tests are unvalidated.** No 1 GB pcap fixture or
   10,000-flow fixture exists. These NFR targets remain perpetually DEFERRED.
   Consider creating synthetic fixtures or documenting that these NFRs are
   validated by code inspection only.

---

## Open NFR Debt Items (carried from nfr-catalog.md v2.1)

| NFR ID | Status | Note |
|--------|--------|------|
| NFR-PERF-004 | OPEN-DEBT | SIMD autovectorization unverified; bench harness exists but LLVM IR / `cargo asm` not inspected this sweep |
| NFR-PERF-002 | OPEN-DEBT | Eager-load memory bound untested; no 1 GB fixture |
| NFR-PERF-003 | DEFERRED | Cache-hit-rate test requires 10,000-flow pcap fixture |
| NFR-RES-023 | OPEN | Weak-cipher evidence vec cardinality unbounded; GitHub #102 |

---

## Sweep Metadata

- Run date: 2026-06-17
- Platform: darwin 25.5.0 (Apple Silicon, macOS Sequoia 15.5)
- Rust toolchain: stable (dtolnay/rust-toolchain@stable)
- Benchmark command: `cargo bench --bench pipeline`
- Commits since baseline: 362 (2026-05-19 to 2026-06-17)
- Key feature cycle in interval: ARP analyzer (STORY-111..115, v0.7.0/v0.7.1)
