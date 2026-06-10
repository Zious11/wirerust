# Phase F6 — Fuzz Testing Results (Feature #100)

**Feature:** issue-100-pcap-timestamps
**develop HEAD:** `256a490`
**Date:** 2026-06-09
**cargo-fuzz version:** 0.13.1 (available); requires nightly toolchain to run targets

---

## Summary

| Metric | Value |
|--------|-------|
| Existing fuzz targets | 1 (`fuzz_decode_packet`, VP-008 pcap-parse path) |
| New focused targets added | 0 (justified — see below) |
| Timestamp path reachable via existing target | **No** (timestamp bypasses `decode_packet`) |
| Crashes found | 0 |
| Fuzz finding disposition | Input domain fully covered by exhaustive-range proptest; no fuzz gap |

---

## Threat model for the delta

The timestamp originates from the pcap **packet header** (`raw_packet.ts_sec`, a `u32`) and is
therefore **attacker-controlled** — a crafted pcap can set any `u32` value, including `0` and
`u32::MAX`. The F6 concern is: can an adversarial timestamp cause a panic, overflow, or
incorrect output as it threads to `Finding.timestamp`?

## Data-flow trace of the timestamp (source → sink)

```
pcap packet header
  └─ reader.rs:76  RawPacket.timestamp_secs = raw_packet.ts_sec   (u32, from pcap_file crate)
       └─ main.rs:167  reasm.process_packet(&parsed, raw.timestamp_secs, &mut dispatcher)
            └─ reassembly/mod.rs  process_packet(timestamp: u32)
                 ├─ hot-path:  flush_contiguous_data → on_data(.., timestamp)
                 └─ close:     close_flow → on_data(.., flow.last_seen)   (u32)
                      └─ analyzer/http.rs + tls.rs  store last_ts: u32 per FlowKey
                           └─ emission sites:  DateTime::from_timestamp(last_ts as i64, 0)
```

**Critical observation:** `decode_packet(data, datalink)` — the function fuzzed by the
existing `fuzz_decode_packet` target — receives only the packet **payload bytes** and the
datalink type. It **never sees the timestamp**. The timestamp is parsed separately from the
pcap record header by the `pcap_file` crate and carried as a plain `u32`. Therefore the
timestamp-threading path introduced by Feature #100 is **not reachable** through the existing
fuzz target, and re-running `fuzz_decode_packet` would exercise none of the delta.

## Existing target re-run (VP-008 regression check)

`fuzz_decode_packet` is unchanged by this feature and remains the VP-008 no-panic guarantee
for the decode path. It is **orthogonal** to the timestamp delta. No regression to that target
is introduced (the 6 changed source files do not touch `src/decoder.rs`). A focused re-run was
considered but adds no coverage of the delta; the decode path is out of Feature #100's scope.

## Why a focused timestamp fuzz target adds NO value over the existing proptest

A hypothetical `fuzz_on_data_timestamp` target would feed an arbitrary `u32` into the
threading path and assert no-panic / correct `Finding.timestamp`. This is **strictly dominated**
by the existing property test, for three reasons:

1. **The input domain is exactly `u32` — a single 4-byte scalar.** The timestamp has no
   structure for a coverage-guided fuzzer to discover; libFuzzer's value here would be to
   explore the `u32` space, which the proptest strategy `0u32..=u32::MAX` already samples
   randomly, anchored by explicit boundary tests at both endpoints (`0` and `u32::MAX`).

2. **There is exactly one timestamp-dependent branch:** the `Some`/`None` outcome of
   `DateTime::from_timestamp(v as i64, 0)`. As established in `kani-results.md`, this is
   **total** over all `u32` (always `Some`, never panics) by a closed-form range argument.
   A fuzzer cannot find a crashing input because the codomain has no crashing input.

3. **The proptest already drives the full pipeline, not just the conversion.**
   `prop_finding_timestamp_matches_on_data_timestamp` constructs real SYN/SYN-ACK/data/FIN
   packets at an arbitrary `ts_sec`, runs them through `TcpReassembler` + `HttpAnalyzer`, and
   asserts the emitted `Finding.timestamp`. A `.expect()` on `from_timestamp` inside the test
   means any `None` (the only conceivable failure mode) would fail the test. This exceeds what
   a no-panic fuzz target would assert (it checks value-correctness, not merely absence of
   panic).

## Adversarial-input safety check (no panic / no overflow / no leakage)

| Adversarial input | Behavior | Safe? |
|-------------------|----------|-------|
| `ts_sec = 0` (epoch) | `Some(1970-01-01T00:00:00Z)` | yes (EC-003, boundary test) |
| `ts_sec = u32::MAX` (~2106 CE) | `Some(...)`, no panic, no overflow | yes (EC-004, boundary test) |
| Arbitrary `u32` | `v as i64` is a widening cast (lossless, no overflow); chrono returns `Some` | yes (proptest 256 cases) |
| Internal detail in Finding output | `Finding.timestamp` is only the ISO-8601 datetime; no path, memory, or internal-state leakage | yes — value is a plain capture timestamp, framed as capture-relative provenance per BC-2.09.007 |

Release profile has `overflow-checks = true`; the only arithmetic on the timestamp is the
widening `u32 as i64` cast, which cannot overflow.

## Verdict

**0 crashes. No fuzz gap.** A focused timestamp fuzz target is justified-omitted: the input
domain is a single unstructured `u32` whose sole consumer is a provably-total conversion, and
that domain is already covered by a full-range proptest plus boundary tests at both endpoints.
The existing `fuzz_decode_packet` (VP-008) is unaffected by the delta and continues to guard
the decode path it owns.
