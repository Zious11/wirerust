# Phase F6 — Fuzz Testing Results (feature-iec104 delta)

**Feature:** IEC-104 passive analyzer (STORY-167..174)
**develop HEAD:** `b36b884`
**Date:** 2026-07-17
**Toolchain:** `cargo +nightly fuzz` (cargo-fuzz 0.13.1 / libFuzzer)
**Target:** `fuzz_iec104_parser` (fuzz/fuzz_targets/fuzz_iec104_parser.rs)

Re-run rationale (VP-047): `on_data` was modified by FIX-P4-001 / FIX-F5-001
enrichment (source_ip/timestamp/direction threaded through the frame-walk and
threat-detection paths). This run genuinely re-verifies no-panic / loop
termination / carry-bound on the current emit-site code, not a STORY-174 assumption.

---

## Summary

| Metric | Value |
|--------|-------|
| Command | `cargo +nightly fuzz run fuzz_iec104_parser -- -max_total_time=300` |
| Wall-clock | 301 s (DONE) |
| Total runs | 2,642,251 |
| exec/s (final) | 8,778 (steady-state ~8,800) |
| Final coverage | cov: 451, ft: 1398 |
| Corpus | 345 entries / 92 Kb |
| RSS | 730 Mb |
| **Crashes** | **0** |
| Timeouts / OOMs / leaks | 0 |
| Artifacts dir | `fuzz/artifacts/fuzz_iec104_parser/` empty (no crash inputs) |

## VP-047 properties confirmed

- **No-panic:** 2.64M inputs through `on_data` (post-enrichment) — 0 panics, 0 aborts.
- **Loop termination:** run completed cleanly at the 300 s budget; no libFuzzer
  timeout triggered (frame-walk loop terminates on all inputs).
- **Carry bound:** no OOM / RSS runaway (steady 730 Mb); carry buffer stays bounded.

## Verdict

Fuzz gate: **PASS** — 5-minute minimum satisfied (301 s), 2.64M executions,
0 crashes on current code b36b884.
