# Phase F6 — Fuzz Testing Results (Feature #7 — Modbus TCP analyzer)

**Feature:** Modbus TCP analyzer (issue #7, v0.4.0)
**develop HEAD:** `68a3306`
**Date:** 2026-06-09
**Toolchain:** `cargo +nightly fuzz` (cargo-fuzz / libFuzzer), nightly `rustc 1.97.0-nightly`

---

## Summary

| Metric | Value |
|--------|-------|
| Fuzz target | `fuzz_modbus_parse` (NEW — added this phase) |
| Wall-clock | 301 s (`-max_total_time=300`) |
| Total executions | **3,716,084** |
| exec/s (steady state) | ~14,300 |
| **Crashes / panics / OOM / timeouts** | **0** |
| Crash artifacts written | none (`fuzz/artifacts/fuzz_modbus_parse/` empty) |
| Coverage reached | cov 803, ft 3207, corpus 538 entries |

**Verdict: 0 crashes in 5 minutes (3.7M execs). PASS.**

---

## Why a new target was added

The pre-existing target `fuzz_decode_packet` covers VP-008 (`decoder::decode_packet`) only —
it never reaches the Modbus parse path. The Modbus analyzer consumes attacker-controlled pcap
bytes through two surfaces that were previously unfuzzed:

1. `parse_mbap_header(&[u8])` — the pure-core fixed-offset MBAP decoder (VP-022 sub-property A
   is Kani-proven for `len <= 12`; the fuzzer is an **unbounded** dynamic cross-check).
2. `ModbusAnalyzer::on_data(...)` — the effectful `StreamHandler` shell: the ADU walk loop,
   the F-105-001 partial-ADU **carry buffer (260-byte cap)**, the **pending table (256-cap)**,
   the 3-point validity gate, the desync latch, and the full `process_pdu` detection engine.
   Kani cannot drive `on_data` (its `HashMap` `RandomState` seed is an FFI the model checker
   cannot symbolically execute), so fuzzing is the **primary dynamic safety check** for this
   shell.

## Target design (`fuzz/fuzz_targets/fuzz_modbus_parse.rs`)

For each arbitrary input the harness:
- calls `parse_mbap_header(data)` over the raw unbounded bytes;
- splits `data` in half and feeds the two halves as two successive `on_data` calls on the
  SAME port-502 flow key — deliberately exercising the carry-buffer **cross-segment ADU
  reassembly** path (a partial ADU straddling a TCP segment boundary);
- alternates `Direction::ClientToServer` (request-insert path) and `ServerToClient`
  (response-match / exception-attribution path), with advancing timestamps to drive the
  time-windowed burst / sustained / exception detectors;
- feeds the whole buffer a third time to drive duplicate-inflight and pending-cap accounting;
- calls `on_flow_close(.., CloseReason::Fin)`.

Registered in `fuzz/Cargo.toml` as a `[[bin]]`. Build: `cargo +nightly fuzz build
fuzz_modbus_parse` — clean.

## Run

```
cargo +nightly fuzz run fuzz_modbus_parse -- -max_total_time=300
...
Done 3716084 runs in 301 second(s)
```

No `ERROR: libFuzzer`, no `panicked`, no `SUMMARY: ` crash line, no `crash-*` / `oom-*` /
`timeout-*` artifact. The DoS guards held: the 260-byte carry cap and 256-entry pending cap
bounded memory; no unbounded growth, no OOB index, no `unwrap()` on attacker bytes fired.

## Existing target (regression)

`fuzz_decode_packet` (VP-008) remains registered and unchanged. Not re-run this phase
(out of Modbus delta scope; covered by prior phases).
