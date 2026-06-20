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
id: "HS-102"
category: "security-probes"
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

# Holdout Scenario: pcapng Timestamp Tsresol Overflow — Saturating Guards for Extreme and Adversarial Resolution Values

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

This scenario exercises the saturating-arithmetic guards in the BC-2.01.014 timestamp helper
for adversarially-crafted if_tsresol values that would overflow 64-bit arithmetic without the
mandated clamp and saturating operations. These are the safety-critical paths required by
H-1 / SEC-001 / SEC-006.

### Case A — if_tsresol=0xFF (base-2, exponent e=127: must not panic, saturating behavior)

1. A crafted pcapng file is presented with:
   - SHB with LE byte-order magic 0x1A2B3C4D
   - IDB with if_tsresol=0xFF (bit7=1 → base-2; e = 0xFF & 0x7F = 127) and linktype=1
   - One EPB with any valid ts_high/ts_low pair (e.g., ts_high=0x00000001, ts_low=0x00000000)
   - This if_tsresol value encodes ticks_per_sec = 2^127, which overflows u64 (max ~1.8×10^19)
   - With the mandated clamp: e is clamped from 127 to 63 before the shift; or the
     checked_shl returns None and saturates to u64::MAX; the saturating path sets ts_usecs=0
2. The user runs `wirerust analyze ts_overflow_b2_e127.pcapng --json`.
3. The tool MUST exit 0 or exit with a non-panic error. Under NO circumstances may the
   process panic, terminate with SIGILL, or produce a Rust overflow panic. The public-
   observable postcondition is: **no panic; exit code is 0 (packet processed with saturated
   timestamp) or a non-zero graceful error**. A panic or SIGABRT is an automatic FAIL.

### Case B — if_tsresol=0x94 (base-2, exponent e=20: intermediate-overflow territory)

1. A crafted pcapng file is presented with:
   - SHB with LE byte-order magic 0x1A2B3C4D
   - IDB with if_tsresol=0x94 (bit7=1 → base-2; e = 0x94 & 0x7F = 20) and linktype=1
   - ticks_per_sec = 2^20 = 1,048,576
   - One EPB with ts_high=0x00000000, ts_low=0x00100000 (1,048,576 raw ticks).
     Correct result: ts_sec = 1,048,576 / 1,048,576 = 1; ts_usecs = 0.
     Intermediate product: (ticks % ticks_per_sec) * 1,000,000 = 0 * 1,000,000 = 0 (no overflow here).
   - A second EPB with ts_high=0x00000000, ts_low=0x00180000 (1,572,864 raw ticks).
     Correct result: ts_sec = 1; remainder = 1,572,864 - 1,048,576 = 524,288;
     ts_usecs = 524,288 * 1,000,000 / 1,048,576 = 500,000. (Exactly half a second.)
     The intermediate product 524,288 * 1,000,000 = 524,288,000,000 exceeds u32::MAX but
     fits in u64; the u128 intermediate or saturating_mul mandated by BC-2.01.014 must be used.
2. The user runs `wirerust analyze ts_base2_e20.pcapng --json`.
3. The tool exits 0. Two packets are ingested. No panic occurs. The second packet's
   sub-second component (ts_usecs) is in [0, 999,999] — observable if the field is
   exposed; otherwise the no-panic + exit-0 postcondition is the gate.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.014 | Invariant — no panic for any (u32, u32, u8) input | Case A: e=127 exceeds u64 shift width; clamped or saturated to avoid panic |
| BC-2.01.014 | Postcondition — ts_usecs in [0, 999,999] | Case A: saturated path produces ts_usecs=0; Case B: intermediate overflow path produces ts_usecs=500,000 |
| BC-2.01.014 | Edge case — base-2 e >= 64 clamp (EC mandated by H-1/SEC-001/SEC-006) | Case A: e=127 must be clamped; MUST NOT shift by 127 in Rust debug/overflow-checked mode |
| BC-2.01.014 | Edge case — intermediate-product u64 overflow for e >= 43 | Case B: e=20 intermediate product fits in u64 but is large; e=43 and above would overflow; the u128 or saturating_mul path must be in place |

## Verification Approach

These are adversarial probe inputs. The primary verification metric is **no panic**:

```
wirerust analyze ts_overflow_b2_e127.pcapng --json
echo "Exit code: $?"
```

Expect: process completes (does not panic or abort). Exit code is either 0 (packet
processed with saturated/degraded timestamp) or a non-zero graceful error message on
stderr. An output like `thread 'main' panicked at 'attempt to shift left with overflow'`
is an automatic FAIL.

```
wirerust analyze ts_base2_e20.pcapng --json
echo "Exit code: $?"
```

Expect: exit 0, two packets in output. No error on stderr.

If `wirerust` is built in release mode (which enables overflow-checks=true per Cargo.toml),
a shift by 127 without clamping would produce a `SIGILL` trap on x86 or a Rust abort
(not a graceful error). The absence of such a trap is the primary observable.

## Evaluation Rubric

- **No-panic safety** (weight: 0.60): No panic, SIGABRT, SIGILL, or overflow trap for
  either input. This is the critical safety property.
- **Functional correctness** (weight: 0.25): Case B processes both packets successfully
  with exit 0.
- **Error quality** (weight: 0.15): If Case A results in a graceful error (non-zero exit),
  the stderr message does not leak internal values; it states the file could not be
  processed without panicking.

## Edge Conditions

- if_tsresol=0xFF: bit7=1 selects base-2 mode; raw e=127 must be clamped to [0,63] before
  any shift. The test fails if the clamp is absent and overflow-checks=true causes a panic.
- if_tsresol=0x94: bit7=1; raw e=20. The critical edge is the intermediate product
  (remainder * 1_000_000) for large remainder values; e=20 is safely within u64 range for
  this product but e >= 43 would overflow. The u128 intermediate or saturating_mul must
  be used universally (not only for e >= 43) to satisfy the formal Kani proof (VP-025).
- The Kani proof (VP-025) verifies this property over the full (u32, u32, u8) input space.
  This holdout covers the same edge cases end-to-end to confirm the proof-target function
  is the one actually called on the live code path.

## Fixture Construction Note

For the holdout evaluator: constructing a valid pcapng file for Case A requires writing raw
bytes. The SHB (block_type=0x0A0D0D0A, LE), IDB with options TLV (code=9, len=1,
value=0xFF, padded to 4 bytes, followed by opt_endofopt code=0), and one EPB are
sufficient. The EPB payload can be any 14-byte Ethernet frame (6+6+2 bytes, e.g., all
zeros). All block lengths must be consistent (block_total_length = initial_len = trailing_len).

## Failure Guidance

"HOLDOUT LOW: HS-102 (satisfaction: 0.XX) — pcapng timestamp saturating-arithmetic guards
are absent. Case A panic indicates the base-2 e-clamp (e > 63 → saturate) is missing.
Case B intermediate-overflow error indicates the u128 intermediate or saturating_mul is
absent. See BC-2.01.014, VP-025, and ADR-009 Decision 4 / H-1 / SEC-001 / SEC-006."
