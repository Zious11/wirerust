---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.014.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-101"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.01.014
verification_properties:
  - VP-025
lifecycle_status: active
introduced: v0.9.x-pcapng-reader
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: pcapng Timestamp Tsresol — Microsecond Default and Nanosecond Fast-Path Regression Guard

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

This scenario is the primary regression guard against the `pcap-file` 2.0.0 timestamp bug
(hard-codes `Duration::from_nanos`, never applies `if_tsresol`). For a pcapng file using the
specification-default microsecond resolution (if_tsresol=6), the crate would silently produce
timestamps 1000× too large if wirerust used the high-level API. This scenario proves wirerust
does NOT make that error on the raw-block path.

### Case A — if_tsresol=6 (microsecond default, the pcapng spec default and Wireshark default)

1. A crafted pcapng file is presented containing:
   - One SHB (LE byte-order magic 0x1A2B3C4D)
   - One IDB with if_tsresol option (TLV code 9, value 0x06) and linktype=1 (Ethernet)
   - One EPB carrying a minimal Ethernet frame with a known ts_high/ts_low pair such that
     the correct microsecond-interpretation timestamp is T_expected_us, and the erroneous
     nanosecond interpretation would yield a value 1000× larger (T_expected_us × 1000)
   - Concrete known values: ts_high=0x00000000, ts_low=0x000F4240 (1,000,000 raw ticks).
     With if_tsresol=6: ticks_per_sec = 10^6 = 1,000,000; ts_sec = 1,000,000 / 1,000,000 = 1;
     ts_usecs = (1,000,000 % 1,000,000) * 1,000,000 / 1,000,000 = 0. Expected: ts_sec=1, ts_usecs=0.
     The crate's wrong path would produce ts_sec=1,000,000 (treating as nanoseconds, dividing by 10^9).
2. The user runs `wirerust analyze ts_usec_default.pcapng --json`.
3. The tool exits 0 and JSON output is present. The evaluator confirms that the packet's
   timestamp (reflected in the summary or any per-packet field if exposed) is consistent
   with ts_sec=1 (i.e., approximately 1 second since the epoch), NOT 0 seconds (nanosecond
   interpretation of 1,000,000 ns = 0.001 seconds, rounded down) or 1,000,000 seconds
   (microseconds misread as nanoseconds). The correct result is that one packet is ingested
   without error.

### Case B — if_tsresol=9 (nanosecond resolution, fast-path)

1. A crafted pcapng file is presented containing:
   - One SHB (LE byte-order magic 0x1A2B3C4D)
   - One IDB with if_tsresol option code 9, value 0x09 (10^-9, nanosecond resolution)
     and linktype=1 (Ethernet)
   - One EPB with ts_high=0x00000000, ts_low=0x3B9ACA00 (1,000,000,000 raw ticks).
     With if_tsresol=9: ticks_per_sec = 10^9; ts_sec = 1,000,000,000 / 1,000,000,000 = 1;
     ts_usecs = (1,000,000,000 % 1,000,000,000) * 1,000,000 / 1,000,000,000 = 0.
     Expected: ts_sec=1, ts_usecs=0.
2. The user runs `wirerust analyze ts_nsec.pcapng --json`.
3. The tool exits 0 and JSON output is present. One packet is ingested without error or panic.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.014 | Postcondition 1 — ts_sec and ts_usecs computed correctly from raw ticks and if_tsresol | Case A: microsecond resolution (default, spec-mandated); must NOT produce 1000× inflated timestamp |
| BC-2.01.014 | Postcondition 1 — if_tsresol=9 produces correct nanosecond-to-us conversion | Case B: nanosecond resolution fast-path |
| BC-2.01.014 | Invariant — ts_usecs is always in [0, 999,999] | Both cases: no usec overflow above one-million |
| BC-2.01.014 | Edge case EC-001 — if_tsresol absent defaults to 6 | Case A exercises the if_tsresol=6 explicit path; the absent-defaults-to-6 behavior is exercised by any pcapng file lacking the TLV option |

## Verification Approach

Construct or use reference pcapng fixtures:

```
wirerust analyze ts_usec_default.pcapng --json
```

Observe: exit code 0; no error on stderr; JSON `total_packets` >= 1. The presence of
the packet in the output (no decode error) confirms the timestamp did not trigger an
overflow or underflow that caused rejection.

For the regression-guard proof: if wirerust internally exposed per-packet timestamps in
JSON output, the evaluator would assert ts_sec == 1. Since current JSON output does not
expose per-packet timestamps, the evaluator verifies the weaker but still meaningful
postcondition: **the packet is processed without error**. A 1000× inflated timestamp
(e.g., ts_sec = 1,000,000 seconds = ~11.6 days since epoch) would not cause a visible
error at the analysis layer, but the Kani proof (VP-025) formally covers the arithmetic.
The integration test role of this holdout is to confirm the raw-block path is taken at
all — the per-packet processing without decode error is the observable gate.

```
wirerust analyze ts_nsec.pcapng --json
```

Observe: exit code 0; no error on stderr; JSON `total_packets` >= 1.

## Evaluation Rubric

- **Functional correctness** (weight: 0.50): Both Case A and Case B pcapng files are
  processed without error (exit 0, at least one packet in output).
- **No-panic safety** (weight: 0.30): No panic occurs for either resolution value. The
  Kani proof (VP-025) formally covers arithmetic overflow; this holdout covers the
  end-to-end code path.
- **Error quality** (weight: 0.10): No spurious error messages referencing timestamp
  overflow, underflow, or resolution parsing.
- **Data integrity** (weight: 0.10): JSON output is well-formed and parseable.

## Edge Conditions

- Case A (if_tsresol=6) is the regression guard for the most common real-world case
  (Wireshark default capture format). Failure here means the high-level API was used
  instead of the raw-block path.
- Case B (if_tsresol=9) exercises the nanosecond fast-path. A pcap-classic nanosecond
  file uses a different mechanism; this tests the pcapng-native path.
- The EPB ts_high=0 is deliberately chosen to simplify the verification: all tick
  arithmetic fits in 32-bit space, eliminating 64-bit reconstruction as a source of
  test variance.
- If_tsresol absent from IDB: the raw-block path defaults to 6 (the pcapng specification
  mandates this). This is covered by any pcapng fixture lacking the TLV option, and
  is the same arithmetic path as Case A once the default is applied by the caller.

## Failure Guidance

"HOLDOUT LOW: HS-101 (satisfaction: 0.XX) — pcapng timestamp resolution path is
incorrect. If Case A fails with a decode error or panic, the raw-block path is likely
taking a wrong branch or the if_tsresol default (6) is not applied. If Case B fails,
the nanosecond path is broken. See BC-2.01.014 and VP-025."
