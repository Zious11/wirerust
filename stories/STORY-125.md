---
document_type: story
story_id: STORY-125
epic_id: E-19
version: "1.0"
status: draft
# BC status: BCs authored and anchored below; all traces complete.
producer: story-writer
timestamp: 2026-06-20T00:00:00Z
phase: f3
points: 8
priority: P0
depends_on: [STORY-123, STORY-124]
blocks: [STORY-127]
behavioral_contracts:
  - BC-2.01.012
  - BC-2.01.014
verification_properties: [VP-025, VP-027]
tdd_mode: strict
target_module: reader
subsystems: [SS-01]
estimated_days: 3
feature_id: f3-pcapng-reader-support
wave: 53
inputs:
  - .factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.012.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.014.md
input-hash: ""
# Dependency anchor: STORY-125 depends on STORY-123 because the SHB parse and
#   pcapng routing are prerequisites for EPB parsing; STORY-125 depends on
#   STORY-124 because the interface table (Vec<InterfaceInfo> with if_tsresol)
#   must be populated before any EPB can look up its interface_id to retrieve
#   if_tsresol for timestamp conversion.
# Subsystem anchor: SS-01 owns this story's scope because BC-2.01.012 and
#   BC-2.01.014 are both SS-01 behavioral contracts per their traceability
#   tables, anchored to src/reader.rs (C-4) per ARCH-INDEX Subsystem Registry.
input-hash: "d81b9f3"
---

# STORY-125: EPB Parse and 64-bit Timestamp Normalization (Kani VP-025 + VP-027)

## Narrative

- **As a** security analyst using wirerust to analyze pcapng captures from modern Wireshark and
  tcpdump captures
- **I want** the pcapng reader to parse Enhanced Packet Blocks (EPBs) and correctly convert 64-bit
  split-tick timestamps using the per-interface `if_tsresol` exponent
- **So that** each EPB produces a `RawPacket` with correct `(timestamp_secs, timestamp_usecs)`
  values regardless of whether the capture uses microsecond (default), nanosecond, or base-2
  resolution; and the Kani formal proofs VP-025 and VP-027 are in place

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.01.012 | Parse pcapng Enhanced Packet Block (EPB): Packet Data and Timestamp |
| BC-2.01.014 | Pure-Core 64-bit pcapng Timestamp Normalization to (ts_sec, ts_usecs) |

## Acceptance Criteria

### AC-001 (traces to BC-2.01.012 postcondition 9 step i — body.len() >= 20 gate, E-INP-008)
Before reading any EPB fixed field, wirerust MUST check `body.len() >= 20`
(`EPB_FIXED_OVERHEAD_BYTES`). If false → `Err` mapping to **E-INP-008** (body too short; wirerust
body-decode path; crate already successfully framed the block). The crate does NOT run its
`EnhancedPacketBlock` parser on the raw path — wirerust owns this check (M-1 / BC-2.01.012 AC-003).
Constructible window: btl in [12, 32) → body in [0, 19] bytes → E-INP-008.

**Test:** `test_BC_2_01_012_no_panic_malformed`, `test_BC_2_01_012_epb_body_short_e_inp_008`

### AC-002 (traces to BC-2.01.012 postcondition 5a — empty-table → E-INP-009)
An EPB whose `interface_id` is evaluated when the interface table is EMPTY MUST return `Err`
mapping to **E-INP-009** with message
`"EPB references interface_id=<id> but interface table is empty — no IDB has been parsed"`.
This check occurs at step (iii) of the EPB evaluation order — AFTER the body.len() >= 20 gate
(step i) and `interface_id` read (step ii), BEFORE any captured_len arithmetic (step v). This check
is independent of the captured_len field value.

**Test:** `test_BC_2_01_012_interface_id_bounds_check` (must assert E-INP-009 for empty-table case)

### AC-003 (traces to BC-2.01.012 postcondition 5b — OOB-on-non-empty → E-INP-010)
An EPB whose `interface_id >= table.len()` on a NON-EMPTY table MUST return `Err` mapping to
**E-INP-010** with message `"EPB interface_id=<id> out of range (table size=<n>)"`. This code MUST
differ from the empty-table code (E-INP-009). Using the same code for both paths is an AC-001
violation per BC-2.01.012.

**Test:** `test_BC_2_01_012_interface_id_bounds_check` (must assert E-INP-010 for OOB-non-empty
case and confirm a different error code from E-INP-009)

### AC-004 (traces to BC-2.01.012 postcondition 9 step v — two-step captured_len validation)
Before any memory allocation for packet data, two checks are applied in order (both BEFORE any
allocation):
1. **PC6a** (unconditional bound-by-body, LIVE REACHABLE GUARD): `captured_len <= body.len()` →
   `Err` mapping to **E-INP-008** on failure (wirerust body-decode; crate already framed the block)
2. **PC6b** (padding-aware overhead, DEFENSE-IN-DEPTH; unreachable on a crate-framed 4-aligned
   block): `EPB_FIXED_OVERHEAD_BYTES(20) + captured_len + pad_len(captured_len) <= body.len()`
   where `pad_len(n) = (4 - n % 4) % 4` → `Err` mapping to **E-INP-008** on failure. MUST be
   coded even though unreachable via normal crate delivery.

**Test:** `test_BC_2_01_012_guard_before_allocate`

### AC-005 (traces to BC-2.01.012 postcondition 3 — packet data bounded by captured_len)
Packet data is copied from the EPB body bounded by `captured_len` bytes (NOT `original_len`).
`captured_len < original_len` means the packet was captured-length-truncated by the writing tool;
wirerust copies exactly `captured_len` bytes and does NOT compute or apply snaplen (Decision 9
amendment). `original_len` IS retained on `RawPacket`; `captured_len` is NOT retained (`data.len()`
recovers it).

**Test:** `test_BC_2_01_012_data_bounded_by_captured_len`

### AC-006 (traces to BC-2.01.012 AC-005 — zero-byte captured_len valid, EC-008)
When `captured_len = 0`, wirerust MUST produce `RawPacket { data: vec![] }`. The padding-aware
check passes: `20 + 0 + 0 <= body.len()` for any 20-byte-minimum body. Zero-byte packets are
valid; `data` is empty, not absent.

**Test:** `test_BC_2_01_012_zero_byte_captured_len`

### AC-007 (traces to BC-2.01.012 AC-006 — max-boundary captured_len, EC-009/010)
When `captured_len` equals the largest value satisfying the padding-aware bound
(`20 + captured_len + pad_len(captured_len) == body.len()`), wirerust MUST produce `Ok(RawPacket)`
with `data.len() == captured_len`. A `captured_len` one byte larger (causing padded total to exceed
`body.len()`) MUST return `Err` mapping to **E-INP-008** (wirerust body-decode: padding overrun).

**Test:** `test_BC_2_01_012_max_boundary_captured_len`

### AC-008 (traces to BC-2.01.012 AC-004 — raw split ticks from RawBlock body, NOT crate Duration)
The raw split-tick fields `ts_high: u32` and `ts_low: u32` MUST be read from `RawBlock` body bytes
at offsets 4-7 (`ts_high`) and 8-11 (`ts_low`) within the EPB body. wirerust MUST NOT consume
`EnhancedPacketBlock::timestamp` — the crate's `Duration` type hard-codes nanoseconds
(`enhanced_packet.rs:46-48,65`) and NEVER applies `if_tsresol`, making it WRONG for any
non-nanosecond capture. The EPB parser does NOT form the combined 64-bit ticks itself — that
combine is owned exclusively by the BC-2.01.014 helper.

**Test:** `test_BC_2_01_012_raw_block_path_not_crate_duration`

### AC-009 (traces to BC-2.01.014 postcondition 1 — timestamp helper owns ticks combine)
The EPB parser reads raw `(ts_high: u32, ts_low: u32)` from the body and passes them to
`pcapng_timestamp_to_secs_usecs(ts_high, ts_low, if_tsresol)`. The helper owns the
`ticks = (ts_high as u64) << 32 | ts_low as u64` combine. The `if_tsresol` value is retrieved
from `interfaces[interface_id].if_tsresol` (default `6u8` when absent from IDB, stored by
STORY-124).

**Test:** `test_BC_2_01_014_regression_1000x_bug` (integration: pcapng with known µs timestamps;
assert correct `(ts_sec, ts_usecs)` — proves 1000× crate ns-hardcode bug absent)

### AC-010 (traces to BC-2.01.014 postconditions 2 and 4 — base-10 saturating arithmetic, µs fast path)
For base-10 `if_tsresol` (bit 7 clear, e = if_tsresol & 0x7F):
- `ticks_per_sec` MUST use a precomputed lookup table for e∈[0,19], saturate to u64::MAX for e≥20
  (Option A — preferred; Kani-decidable without unwind annotation) OR `10u64.checked_pow(e as
  u32).unwrap_or(u64::MAX)` with `#[kani::unwind(128)]` (Option B)
- `ts_sec = (ticks / ticks_per_sec).min(u32::MAX as u64) as u32` — MANDATORY `.min()` saturation;
  bare `as u32` wraps for post-Y2106 ts_high (e.g., ts_high=4295 → ticks/1_000_000 > u32::MAX)
- µs fast path (if_tsresol == 6 exactly): `ts_sec = (ticks / 1_000_000).min(u32::MAX as u64) as
  u32`; `ts_usecs = (ticks % 1_000_000) as u32`. `.min()` saturation is MANDATORY in the fast path
  (M-3 / ADR-009 rev 8).
- Intermediate `(ticks % ticks_per_sec) * 1_000_000` MUST use u128 or `u64::saturating_mul` to
  prevent overflow for large ticks_per_sec values.

**Test:** `test_BC_2_01_014_usecs_default_matches_classic_pcap`,
`test_BC_2_01_014_fast_path_saturation_guard` (EC-013: ts_high=4295, if_tsresol=6 → ts_sec=u32::MAX)

### AC-011 (traces to BC-2.01.014 postcondition 3 — base-2 exponent: MANDATORY clamp e to [0,63])
For base-2 `if_tsresol` (bit 7 set, e = if_tsresol & 0x7F):
- `e` MUST be clamped to [0, 63] BEFORE any shift: `let e_clamped = (if_tsresol & 0x7F).min(63)`.
  Omitting the clamp causes `1u64 << 64` panic with `overflow-checks = true` (wirerust release
  profile sets `overflow-checks = true`). Clamping is NOT optional.
- `ticks_per_sec = 1u64.checked_shl(e_clamped as u32).unwrap_or(u64::MAX)` (saturate to u64::MAX;
  unreachable after clamp since max shift is 63, which is valid for u64)
- Same saturating `ts_sec` (.min(u32::MAX as u64) as u32) and u128-intermediate `ts_usecs` formulas
  as base-10

**Test:** `test_BC_2_01_014_e127_no_panic` (if_tsresol=0xFF, e=127 → e_clamped=63; no panic),
`test_BC_2_01_014_base2_e20_known_vector` (if_tsresol=0x94, ticks=1_048_576 → ts_sec=1, ts_usecs=0)

### AC-012 (traces to BC-2.01.014 invariant 1 + VP-025 — full u8 Kani proof, BOTH branches)
`pcapng_timestamp_to_secs_usecs(ts_high: u32, ts_low: u32, if_tsresol: u8) -> (u32, u32)` is a
pure-core function with no I/O, no global state, deterministic. The Kani proof VP-025 MUST cover
the FULL u8 if_tsresol space — BOTH the base-2 (high-bit-set) branch AND the base-10 branch —
not just one branch. A Kani harness that only covers base-10 is vacuous for half the input space.
The harness MUST include the fast-path saturation test vector (ts_high=4295, ts_low=0, if_tsresol=6
→ ts_sec=u32::MAX). Properties: `ts_usecs` always in [0, 999_999]; `ts_sec ≤ u32::MAX`; no
division by zero (ticks_per_sec >= 1 always per BC-2.01.014 PC7).

**Test:** VP-025 Kani proof (`#[kani::proof]` over full symbolic input space u32 × u32 × u8)

### AC-013 (traces to BC-2.01.012 postcondition 8 — N-packet encounter order and byte fidelity)
For `arp-baseline-16pkt.cap` fixture (16 EPBs, single section, LE, if_tsresol=6 default), the
resulting `PcapSource.packets` has exactly `packets.len() == 16`. Packets appear in EPB encounter
order (first EPB in block stream → `packets[0]`, last EPB → `packets[15]`). Each `packets[i].data`
is byte-for-byte identical to the captured bytes from that EPB body — no bytes added, dropped, or
reordered.

**Test:** `test_BC_2_01_012_happy_path_n_packet_order_and_byte_fidelity`

## Behavioral Contracts Table

| BC | Version | Clauses Covered |
|----|---------|-----------------|
| BC-2.01.012 | v1.9 | PC1 (raw ts_high/ts_low from RawBlock body), PC2 (timestamp helper call with if_tsresol), PC3 (captured_len for data slice; snaplen not applied — Decision 9 amendment), PC4 (RawPacket appended in encounter order), PC5a (empty-table → E-INP-009 exact message), PC5b (OOB-non-empty → E-INP-010 exact message; DIFFERENT codes), PC6a (unconditional bound-by-body → E-INP-008; LIVE REACHABLE), PC6b (padding-aware → E-INP-008; DEFENSE-IN-DEPTH), PC7 (no silent drop on parse error), PC8 (N-packet order + byte fidelity on arp-baseline-16pkt.cap), PC9 (5-step evaluation order: i body≥20, ii read iface_id, iii empty-table, iv OOB-non-empty, v captured_len), AC-001 (discriminant split: E-INP-009 ≠ E-INP-010), AC-002 (guard-before-allocate PC6a+PC6b), AC-003 (no-panic SEC-005; wirerust owns body≥20 check), AC-004 (raw split ticks, NOT crate Duration), AC-005 (zero-byte captured_len valid), AC-006 (max-boundary captured_len fidelity), Inv1 (encounter order), Inv2 (captured_len for slice not original_len), Inv3 (RawPacket identical structure), Inv5 (EPB_FIXED_OVERHEAD_BYTES=20) |
| BC-2.01.014 | v1.5 | PC1 (ticks combine in helper), PC2 (base-10: checked_pow/lookup, u128 intermediate, ts_sec saturation), PC3 (base-2: e clamped [0,63], checked_shl, u128 intermediate, ts_sec saturation), PC4 (µs fast path: .min(u32::MAX as u64) MANDATORY — M-3), PC5 (ns if_tsresol=9 formula), PC6 (ts_sec saturates at u32::MAX), PC7 (division by zero impossible), Inv1 (no panic — saturating arithmetic), Inv2 (µs default parity with classic-pcap for ts_high==0 domain), Inv3 (ts_usecs always in [0, 999_999]), Inv4 (division by zero impossible), Inv6 (base-2 e clamped [0,63]) |

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| EPB body decode (fixed fields: interface_id, ts_high, ts_low, captured_len, original_len) | `src/reader.rs` (block-walk dispatch) | Effectful shell (I/O: block reading) |
| EPB 5-step evaluation order (BC-2.01.012 PC9) | `src/reader.rs` | Effectful shell (sequential decode) |
| `pcapng_timestamp_to_secs_usecs(ts_high, ts_low, if_tsresol)` | `src/reader.rs` or dedicated pure-core module | Pure core (Kani VP-025 proof target) |
| Packet data slice extraction (bounded by captured_len, guard-before-allocate) | `src/reader.rs` | Pure core (slice operation) |

Architecture section references: `architecture/module-decomposition.md` (SS-01 C-4,
`src/reader.rs`); ADR-009 Decision 1 (raw-block path; `RawBlock` / `next_raw_block`), Decision 4
(pure-core timestamp helper, designated Kani target VP-025), Decision 20 (uniform error-code rule;
ALL wirerust body-decode failures → E-INP-008; crate framing rejection → E-INP-010).

## Forbidden Dependencies

- wirerust MUST NOT import or call `pcap_file::pcapng::blocks::enhanced_packet::EnhancedPacketBlock`
  or its `timestamp` field. The crate's `Duration::from_nanos` hard-codes ns resolution
  (`enhanced_packet.rs:46-48,65`) and is WRONG for non-nanosecond captures (the common case is
  microseconds). Any diff that imports this type in the EPB parse path MUST fail review.
- `pcapng_timestamp_to_secs_usecs` MUST be a pure function whose signature contains only Rust
  primitive integer types (`u8`, `u32`, `u64`, `u128`). No crate type or `std` struct may appear
  in its signature. It has no I/O and no global state.
- `src/reader.rs` MUST NOT gain any new crate dependency (+0 new crates, ADR-009 Decision 1).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `captured_len == original_len` | Data copied in full; normal case; no truncation |
| EC-002 | `captured_len < original_len` | Data bounded to `captured_len`; snaplen NOT applied (Decision 9 amendment) |
| EC-003 | `ts_high=0, ts_low=0` | `timestamp_secs=0, timestamp_usecs=0`; valid zero-epoch packet |
| EC-004 | `ts_high=u32::MAX, ts_low=u32::MAX` (maximum u64 ticks) | `ts_sec` saturates at u32::MAX; `ts_usecs` from remainder; NO PANIC |
| EC-005 | EPB `interface_id=0` with EMPTY interface table | E-INP-009 with exact message (empty-table discriminant) |
| EC-006 | EPB `interface_id=1` with one IDB (index 0 only) on non-empty table | E-INP-010 with exact message (OOB-non-empty discriminant; different code from E-INP-009) |
| EC-007 | EPB `interface_id=u32::MAX` with any non-empty table | E-INP-010 (OOB on non-empty) |
| EC-008 | `captured_len=0` (zero-length captured data) | `RawPacket { data: vec![] }`; zero-byte packet is valid |
| EC-009 | `captured_len` at max padding-aware boundary: `20 + captured_len + pad_len(captured_len) == body.len()` | `Ok(RawPacket)` with `data.len() == captured_len` |
| EC-010 | `captured_len` one byte over padding-aware boundary | E-INP-008 (padding overrun; wirerust body-decode; crate framed block successfully) |
| EC-011 | EPB body < 20 bytes — btl in [12, 32) | E-INP-008 (body-too-short; wirerust body-decode path; NOT E-INP-010) |
| EC-012 | btl < 12 or trailing-length mismatch | E-INP-010 (crate framing rejection; wirerust never sees body) |
| EC-013 | `if_tsresol=6`, `ts_high=4295`, `ts_low=0` (µs fast path saturation) | `ts_sec=u32::MAX` (ticks/1_000_000 > u32::MAX; `.min()` saturates); `ts_usecs=0`; NO PANIC |
| EC-014 | `if_tsresol=0xFF` (base-2, e=127) | e_clamped=63; ticks_per_sec=1u64<<63; ts_sec=0 for any realistic ticks; NO PANIC |
| EC-015 | `if_tsresol=0x94` (base-2, e=20) | ticks_per_sec=1<<20=1_048_576; correct division; ts_sec=ticks/1_048_576 |
| EC-016 | `arp-baseline-16pkt.cap` (16 EPBs, µs default, LE) | `packets.len()==16`; encounter order preserved; each `packets[i].data` byte-identical |

## Tasks

1. Implement the 5-step EPB evaluation order (BC-2.01.012 PC9) as a single EPB decode function:
   (i) `body.len() >= 20` check → E-INP-008 if false; (ii) read `interface_id: u32` @0-3;
   (iii) empty-table check → E-INP-009 with exact message; (iv) OOB-on-non-empty check →
   E-INP-010 with exact message; (v) read `ts_high @4-7`, `ts_low @8-11`, `captured_len @12-15`,
   `original_len @16-19`; apply PC6a + PC6b checks → E-INP-008 on failure; slice `body[20..20+captured_len]`.
2. Implement `pcapng_timestamp_to_secs_usecs(ts_high: u32, ts_low: u32, if_tsresol: u8) -> (u32, u32)`:
   - Combine: `ticks = (ts_high as u64) << 32 | ts_low as u64`
   - µs fast path (if_tsresol == 6): `ts_sec = (ticks / 1_000_000).min(u32::MAX as u64) as u32`;
     `ts_usecs = (ticks % 1_000_000) as u32`
   - Base-10 (bit7=0, e=if_tsresol & 0x7F): lookup table for e∈[0,19]; saturate u64::MAX for e≥20;
     `ts_sec = (ticks / ticks_per_sec).min(u32::MAX as u64) as u32`;
     `ts_usecs = ((ticks % ticks_per_sec) as u128 * 1_000_000 / ticks_per_sec as u128) as u32`
   - Base-2 (bit7=1, e=if_tsresol & 0x7F): `e_clamped = e.min(63)`;
     `ticks_per_sec = 1u64.checked_shl(e_clamped as u32).unwrap_or(u64::MAX)`;
     same ts_sec and ts_usecs formulas as base-10
3. Read `(ts_high, ts_low)` from EPB body bytes @4-7 and @8-11 via `u32::from_le_bytes` /
   `u32::from_be_bytes` per section byte-order (NOT from `EnhancedPacketBlock::timestamp`).
4. Copy `body[20..20+captured_len]` to `Vec<u8>` for `RawPacket.data`.
5. Write Kani proof harness for **VP-025**: full symbolic input space `(u32, u32, u8)`; assert
   `ts_usecs <= 999_999`; assert `ts_sec <= u32::MAX`; assert no panic for BOTH base-2 (bit7=1)
   AND base-10 (bit7=0) branches; include concrete assertion for fast-path saturation
   (ts_high=4295, ts_low=0, if_tsresol=6 → ts_sec=u32::MAX).
6. Write Kani proof harness for **VP-027**: EPB parse safety with symbolic EPB byte sequences;
   assert empty-table → E-INP-009 discriminant; assert OOB-non-empty → E-INP-010 discriminant;
   assert body.len()<20 → E-INP-008; assert PC6a bound-by-body → E-INP-008; assert PC6b
   padding-overrun → E-INP-008 (injected via synthetic non-4-aligned body); assert no panic.
7. Write integration test `test_BC_2_01_012_happy_path_n_packet_order_and_byte_fidelity` against
   `arp-baseline-16pkt.cap` fixture: assert `packets.len()==16`; encounter order; byte fidelity.
8. Run `cargo test --all-targets` and `cargo clippy --all-targets -- -D warnings`.

## Test Plan

| AC | Test | Type |
|----|------|------|
| AC-001 | `test_BC_2_01_012_no_panic_malformed`, `test_BC_2_01_012_epb_body_short_e_inp_008` | Unit |
| AC-002 | `test_BC_2_01_012_interface_id_bounds_check` (empty-table → E-INP-009 exact message) | Unit |
| AC-003 | `test_BC_2_01_012_interface_id_bounds_check` (OOB-non-empty → E-INP-010; assert != E-INP-009) | Unit |
| AC-004 | `test_BC_2_01_012_guard_before_allocate` | Unit |
| AC-005 | `test_BC_2_01_012_data_bounded_by_captured_len` | Unit |
| AC-006 | `test_BC_2_01_012_zero_byte_captured_len` | Unit |
| AC-007 | `test_BC_2_01_012_max_boundary_captured_len` | Unit |
| AC-008 | `test_BC_2_01_012_raw_block_path_not_crate_duration` | Unit |
| AC-009 | `test_BC_2_01_014_regression_1000x_bug` | Integration |
| AC-010 | `test_BC_2_01_014_usecs_default_matches_classic_pcap`, `test_BC_2_01_014_fast_path_saturation_guard` | Unit |
| AC-011 | `test_BC_2_01_014_e127_no_panic`, `test_BC_2_01_014_base2_e20_known_vector` | Unit |
| AC-012 | VP-025 Kani (`#[kani::proof]` full u8 space; both base-2 and base-10 branches; saturation vector) | Kani formal |
| AC-013 | `test_BC_2_01_012_happy_path_n_packet_order_and_byte_fidelity` (arp-baseline-16pkt.cap) | Integration |

## Previous Story Intelligence

- STORY-124 established `InterfaceInfo { linktype: DataLink, if_tsresol: u8 }` and
  `Vec<InterfaceInfo>` with correctly stored `if_tsresol` values (defaulting to `6u8` when absent
  from IDB options). STORY-125 consumes `interfaces[interface_id].if_tsresol` directly — no
  re-extraction needed.
- The `if_tsresol` value from STORY-124 is a raw `u8` byte as stored in the IDB options TLV. The
  BC-2.01.014 helper interprets bit 7 (base selector) and bits 0-6 (exponent) internally —
  STORY-125 passes the raw byte unchanged.
- VP-025 (Kani on `pcapng_timestamp_to_secs_usecs`) and VP-027 (Kani on EPB parse safety) are
  BOTH F3 deliverables per ADR-009 VP table (Phase P1). They are NOT deferred to F6 (VP-028
  cargo-fuzz is the F6 deliverable). Kani proofs must be submitted in the same PR as the
  implementation.
- The `arp-baseline-16pkt.cap` fixture is a pcapng file (pcapng with `.cap` extension) that
  STORY-123 already confirmed returns `Ok(PcapSource)` with 16 packets. STORY-125's integration
  test builds on that: it asserts the same 16 packets, encounter order, and byte fidelity.

## Architecture Compliance Rules

Derived from ADR-009 rev 9 and BC-2.01.012/014:

1. **Raw-block path ONLY** — `EnhancedPacketBlock` is forbidden (hard-codes ns resolution; wrong
   for any non-nanosecond capture). Use `RawBlock.body` bytes directly (`next_raw_block`).
2. **EPB_FIXED_OVERHEAD_BYTES = 20** (body-relative: interface_id:4 + ts_high:4 + ts_low:4 +
   captured_len:4 + original_len:4). Outer 12-byte block frame header is SEPARATE. Combined minimum
   block size = 32 bytes. E-INP-008 window for EPB: btl in [12, 32) → body in [0, 19] bytes.
3. **Five-step evaluation order is canonical and immutable** — any reordering of steps (i)–(v) from
   BC-2.01.012 PC9 is a correctness defect.
4. **Two discriminants MUST differ**: empty-table → E-INP-009, OOB-on-non-empty → E-INP-010.
   Same code for both paths = AC-001 violation.
5. **VP-025 Kani MUST cover full u8 if_tsresol space** — both base-2 (bit7 set) AND base-10
   (bit7 clear) branches. Limiting the harness to base-10 makes the proof vacuous for base-2.
6. **Saturation is mandatory everywhere**: `ts_sec` MUST use `.min(u32::MAX as u64) as u32`. Bare
   `as u32` truncation wraps for post-Y2106 captures and violates VP-025's totality claim.
7. **Base-2 e MUST be clamped to [0,63]** before any shift — `1u64 << 64` panics with
   `overflow-checks = true` (wirerust release profile). Clamping is NOT optional.
8. **u128 intermediate for ts_usecs** — `(ticks % ticks_per_sec) * 1_000_000` overflows u64 for
   base-2 e >= 43. Use `(ticks % ticks_per_sec) as u128 * 1_000_000u128 / ticks_per_sec as u128`.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `pcap_file` | 2.0.0 | `RawBlock` / `next_raw_block` only; NO `EnhancedPacketBlock` |
| `anyhow` | existing | `.context(...)` for all EPB errors; context strings match BC-2.01.017 PC1 |
| `kani` | existing | Formal proof harnesses for VP-025 (timestamp helper) and VP-027 (EPB parse) |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/reader.rs` | Modify | Add EPB decode function; add `pcapng_timestamp_to_secs_usecs` pure-core helper |
| `tests/reader_tests.rs` (or equivalent) | Modify | Add EPB parse tests; integration test for arp-baseline-16pkt.cap |
| `tests/kani_proofs.rs` (or equivalent) | Create/Modify | VP-025 and VP-027 Kani proof harnesses |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~6,500 |
| BC files (2 BCs: BC-2.01.012 v1.9, BC-2.01.014 v1.5) | ~14,000 |
| ADR-009 rev 9 (canonical constants + relevant decisions) | ~4,000 |
| `src/reader.rs` (post-STORY-124) | ~6,000 |
| Kani harness files (VP-025, VP-027) | ~3,000 |
| Test files | ~4,000 |
| Tool outputs | ~1,000 |
| **Total estimated** | **~38,500** |

Within 20-30% of agent context window.

## Dependency Rationale

- `depends_on: [STORY-123, STORY-124]` — STORY-125 requires STORY-123 (pcapng routing, SHB parse,
  section-wide endianness, block-walk loop) and STORY-124 (interface table `Vec<InterfaceInfo>` with
  `if_tsresol` populated and all three IDB checks in place). Without the interface table, EPB
  `interface_id` lookup is undefined behavior.
- `blocks: [STORY-127]` — STORY-127's E2E corpus wiring requires the full EPB parse stack
  (STORY-125) to be in place; the E2E tests assert packet counts and timestamps, which depend on
  correct EPB parsing. STORY-126 (SPB) has separate `depends_on: [STORY-123, STORY-124]` and does
  NOT block on STORY-125 — SPB and EPB are independent block types.
