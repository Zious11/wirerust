---
document_type: behavioral-contract
level: L3
version: "1.4"
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
modified:
  - "v1.1: F2 Burst-A remediation per ADR-009 rev 4 PO dispatch — (1) VP-025 added to Verification Properties. (2) Required SATURATING arithmetic throughout (H-1/SEC-001/SEC-006): base-10 uses checked_pow (saturate to u64::MAX on overflow); base-2 exponent e CLAMPED to [0,63] before shift (e >= 64 panics in Rust with overflow-checks=true — clamping is mandatory); intermediate product (ticks % ticks_per_sec) * 1_000_000 MUST use u128 or saturating_mul (overflows u64 for base-2 e >= 43). (3) Updated EC-006: added explicit statement that e >= 64 panics without clamping and that the spec mandates clamping; also added base-2 e=63 as a tested boundary. (4) Added new edge cases: EC-009 (if_tsresol=6 µs default), EC-010 (if_tsresol=9 ns), EC-011 (if_tsresol=0x94 base-2 e=20), EC-012 (if_tsresol=0xFF base-2 e=127, must not panic). (5) Updated Postcondition 2 and 3 to specify saturating formulas. (6) Updated Invariant 1: 'no panics' is NOW ACTUALLY TRUE with the saturating formula — prior version with `pow` could panic for large base-10 exponents. (7) Clarified that this helper is LOAD-BEARING (drives off raw split ticks from RawBlock; crate's high-level EPB is ns-hardcoded and wrong). — 2026-06-19"
  - "v1.2: Pass-2 remediation per ADR-009 rev 5 (I-9, I-2, I-11) — (I-9) EC-006 corrected: was if_tsresol=0x3F (bit7=0 → base-10, not base-2; e=63); fixed to if_tsresol=0xBF (bit7=1 → base-2, e=63). Panic counter-example changed from 0x40 (base-10 e=64, checked_pow saturates — no panic) to 0xC0 (base-2 e=64 — shift panic without clamp). EC-006 now correctly illustrates the base-2 e=63 boundary and the necessity of clamping for e=64. (I-2) Added Kani implementation note to VP-025 row: base-10 branch MUST use precomputed ticks_per_sec lookup table for e∈[0,19] (Option A, preferred, keeps Kani proof bounded) OR VP-025 harness carries #[kani::unwind(128)] (Option B). Without one of these, the Kani proof is vacuous. (I-11) Added Test: citations to all unit/integration VP rows. — 2026-06-19"
  - "v1.4: Pass-4 remediation R3a (ADR-009 rev 7) — (Decision 21) Added if_tsoffset limitation note to Description: this helper does NOT apply if_tsoffset (IDB option code 10); files with if_tsoffset set carry a timestamp bias of offset×(1/ticks_per_sec); this is an accepted limitation this cycle per ADR Decision 21. Function signature unchanged: (ts_high, ts_low, if_tsresol). — 2026-06-20"
  - "v1.3: Pass-3 remediation per ADR-009 rev 6 (M-4 / timestamp parity) — Corrected Invariant 2: the 'numerically identical to classic-pcap' claim is qualified to ts_high==0 AND ts_sec <= u32::MAX only; pcapng additionally represents the high 32 bits via ts_high, which classic pcap's wire format cannot encode. Updated regression-guard VP row and EC-009 to scope to the ts_high==0 domain. — 2026-06-19"
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
pair used by `RawPacket`. This helper is LOAD-BEARING on the raw-block path (ADR-009 Decision 1
rev 4): it drives off the raw split ticks from `RawBlock`; the crate's high-level
`EnhancedPacketBlock::timestamp` is ns-hardcoded (`Duration::from_nanos`) and never applies
`if_tsresol`, making it WRONG for any non-nanosecond capture (the common case — Wireshark
default — is microsecond, tsresol=6). The function has no I/O, no global state, and is
deterministic; it is the designated Kani proof target (VP-025, ADR-009 Decision 4).
ALL intermediate arithmetic MUST use saturating or checked operations (H-1/SEC-001/SEC-006).
When `if_tsresol` is absent at the IDB level, callers MUST pass `6` (10^-6, microseconds —
the pcapng specification default).

**Limitation (Decision 21):** This helper does NOT apply `if_tsoffset` (IDB option code 10).
Files with `if_tsoffset` set carry a timestamp bias of `offset × (1 / ticks_per_sec)`; the
resulting timestamps are biased by that amount and will not match the true wall-clock epoch.
This is an accepted limitation this cycle per ADR-009 Decision 21. The function signature
remains `(ts_high: u32, ts_low: u32, if_tsresol: u8)` — no `if_tsoffset` parameter is
accepted or applied.

## Preconditions

1. Function is called with `(ts_high: u32, ts_low: u32, if_tsresol: u8)`.
2. `if_tsresol` is either the value extracted from the IDB `if_tsresol` TLV option (code 9),
   or the constant `6u8` when that option is absent from the IDB.
3. No I/O is performed before or during this function.
4. Inputs are raw split ticks from `RawBlock` body bytes — NOT from `EnhancedPacketBlock::timestamp`
   (the crate's Duration, which is incorrect for non-nanosecond captures).

## Postconditions

1. The 64-bit timestamp in ticks is computed as `ticks: u64 = (ts_high as u64) << 32 | ts_low as u64`.
   This shift is safe: both operands are u64 (`u32 as u64`); the high half occupies bits 32-63
   exactly; adding the low u32 cannot overflow u64.

2. If bit 7 of `if_tsresol` is clear (base-10 exponent `e = if_tsresol & 0x7F`):
   - `ticks_per_sec: u64 = 10u64.checked_pow(e as u32).unwrap_or(u64::MAX)`
     (saturate to u64::MAX on overflow — prevents panic for large e; if saturated, ts_usecs = 0).
   - `ts_sec: u32 = (ticks / ticks_per_sec).min(u32::MAX as u64) as u32` (saturating cast,
     capped at u32::MAX — Y2106 boundary, same limitation as classic pcap).
   - The intermediate product `(ticks % ticks_per_sec) * 1_000_000` MUST be computed in u128
     or via `u64::saturating_mul` to avoid overflow (overflows u64 for large ticks_per_sec).
   - `ts_usecs: u32 = (((ticks % ticks_per_sec) as u128 * 1_000_000u128) / ticks_per_sec as u128) as u32`

3. If bit 7 of `if_tsresol` is set (base-2 exponent `e = if_tsresol & 0x7F`):
   - `e` MUST be CLAMPED to [0, 63] before the shift: `let e_clamped = e.min(63)`.
     Reason: Rust panics on `1u64.checked_shl(e as u32)` when `e >= 64` with
     `overflow-checks = true` (wirerust release profile has `overflow-checks = true`).
     Clamping is MANDATORY; omitting it makes the no-panic invariant false.
   - `ticks_per_sec: u64 = 1u64.checked_shl(e_clamped as u32).unwrap_or(u64::MAX)`
     (saturate to u64::MAX; if e_clamped = 63, `1u64 << 63` = 9_223_372_036_854_775_808 which
     fits u64 and is a valid result; `checked_shl` returns None only for shift >= 64, which
     is excluded by the clamp).
   - Same `ts_sec` and `ts_usecs` formulas as Postcondition 2 (using u128 intermediate).

4. `if_tsresol = 6` (default, base-10, 10^-6, microseconds):
   - `ticks_per_sec = 1_000_000`
   - `ts_sec = ticks / 1_000_000` (safe; u64 / u64)
   - `ts_usecs = (ticks % 1_000_000) as u32` (remainder < 1_000_000; fits u32; no intermediate overflow)
   - This is the canonical fast path. Mathematically equivalent to the general formula, but
     the simpler form avoids the u128 intermediate for this critical common case.

5. `if_tsresol = 9` (nanoseconds, base-10, 10^-9):
   - `ticks_per_sec = 1_000_000_000`
   - `ts_usecs = ((ticks % 1_000_000_000) as u128 * 1_000_000u128 / 1_000_000_000u128) as u32`
     (equivalently: `(ticks % 1_000_000_000) / 1_000` — converts ns remainder to µs)

6. When `ticks / ticks_per_sec > u32::MAX`, `ts_sec` saturates at `u32::MAX` (Y2106 boundary;
   same limitation as classic pcap). No panic; saturation is the documented behavior.

7. Division by zero is impossible: `ticks_per_sec` is always ≥ 1.
   - Base-10, e=0: `10^0 = 1`. `checked_pow(0) = Some(1)`.
   - Base-2, e=0 (or clamped to 0): `2^0 = 1`. `checked_shl(0) = Some(1)`.
   - Saturated u64::MAX: divisor is u64::MAX ≥ 1.
   - Therefore the denominator can never be 0.

## Invariants

1. The function is pure: no I/O, no global state reads/writes. With the saturating arithmetic
   prescribed above, NO PANIC occurs for ANY `(u32, u32, u8)` input. This invariant is NOW
   ACTUALLY TRUE (prior formulas using `10u64.pow()` without `checked_pow` or `1u64 << e`
   without clamping could panic for large exponents — those forms are prohibited).
2. `if_tsresol = 6` (default µs) produces results numerically identical to the classic-pcap
   microsecond-resolution path (`ts_frac` used as-is in BC-2.01.002 Postcondition 2) **for
   ts_high == 0 AND ts_sec <= u32::MAX**. Classic pcap stores `ts_sec` as a raw u32 with no
   high-half field; pcapng additionally encodes the high 32 bits of the tick counter via
   `ts_high`, representing timestamps beyond 2^32 / ticks_per_sec seconds (the Y2106 boundary
   for ts_high==0). For ts_high > 0 or ts_sec > u32::MAX (post-Y2106 captures), pcapng can
   represent the full epoch while classic pcap's wire format cannot — `ts_sec` saturates at
   u32::MAX on both formats, but the pcapng format retains the `ts_high` field on the wire.
   This invariant applies exclusively within the ts_high==0 / ts_sec-in-u32-range domain.
3. The returned `ts_usecs` is always in the range [0, 999_999]; it MUST NOT exceed 999_999.
4. Division by zero cannot occur (see Postcondition 7).
5. The function signature takes only scalar integers; it has no dependency on any crate types
   other than Rust primitives and the u128 builtin.
6. Base-2 exponent `e` is ALWAYS clamped to [0, 63] before any shift. The unclamped
   `if_tsresol & 0x7F` value MAY be in [0, 127]; values in [64, 127] are valid `if_tsresol`
   bytes per the pcapng spec but represent sub-femtosecond resolutions unreachable in practice.
   Clamping to 63 saturates `ticks_per_sec` at `u64::MAX` (via `checked_shl` returning None
   for shifts ≥ 64 after clamping makes this unreachable — alternatively, clamping to 63
   means `1u64 << 63` which is a valid u64). Either path: no panic.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `ts_high=0, ts_low=0` (epoch) | `(0, 0)` |
| EC-002 | Default µs (`if_tsresol=6`), `ticks=1_500_000_500` | `ts_sec=1500, ts_usecs=500` |
| EC-003 | Nanoseconds (`if_tsresol=9`), `ticks=1_500_000_000_500` | `ts_sec=1500, ts_usecs=0` (500 ns < 1 µs rounds down) |
| EC-004 | `if_tsresol=0` (base-10, 10^0=1 tick/sec, 1-second resolution) | `ticks_per_sec=1`; `ts_sec=(ticks).min(u32::MAX as u64) as u32`; `ts_usecs=0` |
| EC-005 | `if_tsresol=0x80` (base-2, 2^0=1 tick/sec, 1-second resolution) | Same output as EC-004 |
| EC-006 | `if_tsresol=0xBF` (base-2 [bit7=1], e=63) | `e_clamped=63`; `ticks_per_sec=1u64<<63`; ticks likely << ticks_per_sec; `ts_sec=0, ts_usecs=0`; NO PANIC. Without the e-clamp, `if_tsresol=0xC0` (base-2 [bit7=1], e=64) would panic on `1u64 << 64` with overflow-checks=true; clamping to [0,63] is mandatory. |
| EC-007 | `ts_high=u32::MAX, ts_low=u32::MAX` (maximum u64 ticks) | `ts_sec` saturates at u32::MAX; `ts_usecs` computed from remainder; NO PANIC |
| EC-008 | `if_tsresol=6` (default), `ticks=1_000_000` exactly | `ts_sec=1, ts_usecs=0` |
| EC-009 | `if_tsresol=6` (µs default), pcapng file with known packet timestamp and ts_high=0 | REGRESSION GUARD (ts_high==0 domain): confirms 1000× timestamp bug absent; a Wireshark-default µs capture (ts_high=0, ts_sec within u32 range) must NOT produce timestamps 1000× too large (crate's ns-hardcode bug). Scoped to ts_high==0 per Invariant 2 qualification — the classic-pcap parity claim applies only within this domain. |
| EC-010 | `if_tsresol=9` (nanoseconds), `ticks=2_000_000_500` | `ts_sec=2, ts_usecs=0` (500 ns rounds down to 0 µs) |
| EC-011 | `if_tsresol=0x94` (base-2, e=20) | `e_clamped=20`; `ticks_per_sec=1<<20=1_048_576`; `ts_sec=ticks/1_048_576`; `ts_usecs` via u128 intermediate; NO PANIC |
| EC-012 | `if_tsresol=0xFF` (base-2, e=127, out of u64-shift range) | `e_clamped=63`; `ticks_per_sec=1u64<<63=9_223_372_036_854_775_808`; `ts_sec=0` (any realistic ticks << this); `ts_usecs=0`; NO PANIC |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `ts_high=0, ts_low=1_000_000, if_tsresol=6` | `(1, 0)` — 1 second exactly | happy-path |
| `ts_high=0, ts_low=1_500_000, if_tsresol=6` | `(1, 500_000)` — 1.5 seconds | happy-path |
| `ts_high=0, ts_low=1_500_000_000, if_tsresol=9` | `(1, 500_000)` — 1.5 seconds from ns ticks | happy-path |
| `ts_high=0, ts_low=0, if_tsresol=6` | `(0, 0)` | edge-case |
| `ts_high=u32::MAX, ts_low=u32::MAX, if_tsresol=6` | `ts_sec=u32::MAX` (saturated), `ts_usecs` in [0,999_999], no panic | edge-case |
| `ts_high=1, ts_low=0, if_tsresol=6` | `ts_sec = (2^32 / 1_000_000)`, `ts_usecs = (2^32 % 1_000_000)` | happy-path |
| `ts_high=0, ts_low=0, if_tsresol=0xFF` | `(0, 0)`, no panic (e=127 clamped to 63) | edge-case (EC-012) |
| `ts_high=0, ts_low=1_048_576, if_tsresol=0x94` | `ts_sec=1`, `ts_usecs=0` (base-2 e=20: 1<<20=1_048_576 ticks/sec) | edge-case (EC-011) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-025 | `pcapng_timestamp_to_secs_usecs` totality: no panic for ALL (u32, u32, u8) inputs with saturating arithmetic; `ts_usecs` always in [0, 999_999]; `ts_sec` plausible (≤ u32::MAX); denominator never 0 | Kani: `#[kani::proof]` over full symbolic input space (u32 × u32 × u8 = 2^65 inputs; Kani explores exhaustively via bounded model checking). **Implementation note (I-2):** the base-10 branch MUST use a precomputed ticks_per_sec lookup table for e∈[0,19] (saturating to u64::MAX for e≥20) — **Option A (preferred)**: keeps the Kani proof bounded without unwind annotations; OR the VP-025 Kani harness carries `#[kani::unwind(128)]` — **Option B**. Without one of these, the Kani proof over the base-10 `checked_pow` loop is vacuous (Kani will not explore all paths). Option A is preferred. |
| — | `if_tsresol=6` result matches classic-pcap microsecond result for same epoch (ts_high==0, ts_sec <= u32::MAX domain only — Invariant 2 qualification) | unit: compare against classic-pcap timestamp extraction with ts_high=0 inputs; assert numerical parity within the ts_high==0 / u32-range domain **Test:** `test_BC_2_01_014_usecs_default_matches_classic_pcap` |
| — | `if_tsresol=0xFF` (e=127) does not panic | unit: assert result = (0, 0) or (0, small) without panic **Test:** `test_BC_2_01_014_e127_no_panic` |
| — | `if_tsresol=0x94` (e=20) produces correct result for known ticks | unit: `ticks=1_048_576, if_tsresol=0x94` → `(1, 0)` **Test:** `test_BC_2_01_014_base2_e20_known_vector` |
| — | ts_usecs regression guard for µs-default captures | integration: pcapng file with known ts values and `if_tsresol=6`; assert timestamp_secs and timestamp_usecs are correct (proves 1000× bug absent) **Test:** `test_BC_2_01_014_regression_1000x_bug` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- timestamp normalization is an internal computation within the ingestion pipeline; the `(ts_sec, ts_usecs)` output is embedded in `RawPacket`, which is CAP-01's primary output type |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs or dedicated pure-core module, C-4) |
| Stories | STORY-125 |
| ADR Reference | ADR-009 rev 4 Decision 4 (pure-core timestamp helper, LOAD-BEARING, saturating arithmetic, Kani-provable VP-025); Decision 1 (raw-block path — helper feeds off raw split ticks, not crate Duration) |

## Related BCs

- BC-2.01.011 -- depends on (if_tsresol value comes from IDB parsing in BC-2.01.011)
- BC-2.01.012 -- composed by (EPB parsing passes raw (ts_high, ts_low) to this function)
- BC-2.01.002 -- mirrors (classic-pcap analog uses ts_frac directly; this BC extends the pattern to pcapng)

## Architecture Anchors

- ADR-009 rev 4 Decision 4: saturating arithmetic mandate — `checked_pow`, e-clamp to [0,63], u128 intermediate for `(ticks % ticks_per_sec) * 1_000_000`
- ADR-009 rev 4: "BC-2.01.014 pure-core helper is LOAD-BEARING, not redundant. The H-1/SEC-001/SEC-006 overflow guards are real fixes that the Kani proof (VP-025) will verify."
- `enhanced_packet.rs:46-48,65` (pcap-file 2.0.0 source): confirms crate hard-codes ns, never applies `if_tsresol` — raw ticks not recoverable from `EnhancedPacketBlock::timestamp`
- pcapng spec IETF draft §if_tsresol: option code 9; bit 7 = base selector (0=base-10, 1=base-2); bits 0-6 = exponent; default = 6 (10^-6, microseconds) when option absent
- Kani: designated formal verification target per ADR-009 rev 4 (VP-025)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same inputs always produce same output |
| **Thread safety** | trivially safe (pure function, no shared state) |
| **Overall classification** | pure core — designated Kani verification target (VP-025) |
