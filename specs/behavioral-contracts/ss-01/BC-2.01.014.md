---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: F2
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-01
capability: CAP-01
lifecycle_status: active
introduced: v0.10.0-pcapng
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.014: Pure-Core 64-bit pcapng Timestamp Normalization to (ts_sec, ts_usecs)

## Description

A pure-core function `pcapng_timestamp_to_secs_usecs(ts_high: u32, ts_low: u32, if_tsresol: u8) -> (u32, u32)`
converts a pcapng EPB 64-bit timestamp to the `(timestamp_secs: u32, timestamp_usecs: u32)`
pair used by `RawPacket`. The function has no I/O, no global state, and is deterministic; it
is the designated target for Kani property verification (ADR-009 Decision 4). The `if_tsresol`
byte encodes either a base-10 exponent (bit 7 clear) or a base-2 exponent (bit 7 set) per the
pcapng specification. When `if_tsresol` is absent at the IDB level, callers MUST pass `6`
(the default 10^-6 microsecond resolution) to this function.

## Preconditions

1. Function is called with `(ts_high: u32, ts_low: u32, if_tsresol: u8)`.
2. `if_tsresol` is either the value extracted from the IDB `if_tsresol` TLV option, or
   the constant `6u8` when that option is absent.
3. No I/O is performed before or during this function.

## Postconditions

1. The 64-bit timestamp in ticks is computed as `ticks: u64 = (ts_high as u64) << 32 | ts_low as u64`.
2. If bit 7 of `if_tsresol` is clear (base-10 exponent `e = if_tsresol & 0x7F`):
   - `ticks_per_sec: u64 = 10u64.pow(e as u32)`
   - `ts_sec: u32 = (ticks / ticks_per_sec) as u32` (saturating cast, capped at u32::MAX)
   - `ts_usecs: u32 = ((ticks % ticks_per_sec) * 1_000_000 / ticks_per_sec) as u32`
3. If bit 7 of `if_tsresol` is set (base-2 exponent `e = if_tsresol & 0x7F`):
   - `ticks_per_sec: u64 = 1u64 << e` (2^e ticks per second)
   - Same `ts_sec` and `ts_usecs` formulas as above.
4. `if_tsresol = 6` (default, base-10, 10^-6):
   - `ticks_per_sec = 1_000_000`
   - `ts_sec = ticks / 1_000_000`
   - `ts_usecs = ticks % 1_000_000`
   - This is the canonical fast path; it is mathematically equivalent to the general formula.
5. `if_tsresol = 9` (nanoseconds, base-10, 10^-9):
   - `ticks_per_sec = 1_000_000_000`
   - `ts_usecs = ((ticks % 1_000_000_000) / 1_000) as u32` (convert ns remainder to µs)
6. When `ticks / ticks_per_sec > u32::MAX`, `ts_sec` saturates at `u32::MAX` (Y2106 boundary; same limitation as classic pcap).
7. Division by zero is impossible: `ticks_per_sec` is always ≥ 1 (minimum: base-10 e=0 → 10^0=1; base-2 e=0 → 2^0=1).

## Invariants

1. The function is pure: no I/O, no global state reads/writes, no panics for any u8 `if_tsresol` value.
2. `if_tsresol = 6` (default µs) produces results numerically identical to the classic-pcap
   microsecond-resolution path (`ts_frac` used as-is in BC-2.01.002 Postcondition 2).
3. The returned `ts_usecs` is always in the range [0, 999_999]; it MUST NOT exceed 999_999.
4. Division by zero cannot occur (see Postcondition 7).
5. The function signature takes only scalar integers; it has no dependency on any crate types
   other than Rust primitives.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `ts_high=0, ts_low=0` (epoch) | `(0, 0)` |
| EC-002 | Default µs (`if_tsresol=6`), `ticks=1_500_000_500` | `ts_sec=1500, ts_usecs=500` |
| EC-003 | Nanoseconds (`if_tsresol=9`), `ticks=1_500_000_000_500` | `ts_sec=1500, ts_usecs=0` (500 ns < 1 µs rounds down) |
| EC-004 | `if_tsresol=0` (base-10, 10^0=1 tick/sec, 1-second resolution) | `ticks_per_sec=1`; `ts_sec=ticks as u32`; `ts_usecs=0` |
| EC-005 | `if_tsresol=0x80` (base-2, 2^0=1 tick/sec, 1-second resolution) | Same output as EC-004 |
| EC-006 | `if_tsresol=0x3F` (base-2, 2^63 ticks/sec — sub-femtosecond) | `ticks_per_sec=2^63`; ticks likely < ticks_per_sec; `ts_sec=0, ts_usecs=0` |
| EC-007 | `ts_high=u32::MAX, ts_low=u32::MAX` (maximum u64 ticks) | `ts_sec` saturates at u32::MAX; no panic |
| EC-008 | `if_tsresol=6` (default), `ticks=1_000_000` exactly | `ts_sec=1, ts_usecs=0` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `ts_high=0, ts_low=1_000_000, if_tsresol=6` | `(1, 0)` — 1 second exactly | happy-path |
| `ts_high=0, ts_low=1_500_000, if_tsresol=6` | `(1, 500_000)` — 1.5 seconds | happy-path |
| `ts_high=0, ts_low=1_500_000_000, if_tsresol=9` | `(1, 500_000)` — 1.5 seconds from ns ticks | happy-path |
| `ts_high=0, ts_low=0, if_tsresol=6` | `(0, 0)` | edge-case |
| `ts_high=u32::MAX, ts_low=u32::MAX, if_tsresol=6` | `ts_sec=u32::MAX` (saturated), no panic | edge-case |
| `ts_high=1, ts_low=0, if_tsresol=6` | `ts_sec = (2^32 / 1_000_000)`, `ts_usecs = (2^32 % 1_000_000)` | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | No panic for any (u32, u32, u8) input combination | Kani: `#[kani::proof]` over full input space |
| — | `ts_usecs` always in [0, 999_999] | Kani: assert `ts_usecs < 1_000_000` |
| — | `if_tsresol=6` result == classic-pcap microsecond result for same epoch | unit: compare against classic-pcap timestamp extraction |
| — | Division by zero impossible | Kani: prove denominator always >= 1 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- timestamp normalization is an internal computation within the ingestion pipeline; the `(ts_sec, ts_usecs)` output is embedded in `RawPacket`, which is CAP-01's primary output type |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs or dedicated pure-core module, C-4) |
| Stories | STORY-125 |
| ADR Reference | ADR-009 Decision 4 (pure-core timestamp-conversion helper, Kani-provable, free function with no I/O) |

## Related BCs

- BC-2.01.011 -- depends on (if_tsresol value comes from IDB parsing in BC-2.01.011)
- BC-2.01.012 -- composed by (EPB parsing calls this function for each packet)
- BC-2.01.002 -- mirrors (classic-pcap analog uses ts_frac directly; this BC extends the pattern to pcapng)

## Architecture Anchors

- ADR-009 Decision 4: "pure-core timestamp-conversion helper MUST be extracted (free function or standalone module, no I/O), taking (ts_high: u32, ts_low: u32, if_tsresol: u8) and returning (ts_sec: u32, ts_usecs: u32)"
- pcapng spec IETF draft §if_tsresol: option code 9; bit 7 = base selector (0=10, 1=2); bits 0-6 = exponent
- Kani: designated formal verification target per ADR-009

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same inputs always produce same output |
| **Thread safety** | trivially safe (pure function, no shared state) |
| **Overall classification** | pure core — designated Kani verification target |
