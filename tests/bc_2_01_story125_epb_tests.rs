//! STORY-125: EPB Parse and 64-bit Timestamp Normalization
//!
//! TDD Red Gate suite — targets UNIMPLEMENTED or WRONG behavior.
//!
//! # Red Gate contract
//!
//! ALL tests in the timestamp-helper group and the end-to-end F-3 nanosecond
//! regression test MUST FAIL before implementation begins:
//!
//!   - `pcapng_timestamp_to_secs_usecs` is a `todo!()` stub → panics at runtime.
//!   - The EPB arm hard-codes `DEFAULT_TSRESOL=6` for ALL interfaces; it does NOT
//!     call `pcapng_timestamp_to_secs_usecs` or look up `if_tsresol` per interface
//!     → the nanosecond (if_tsresol=9) end-to-end test produces a 1000× wrong value.
//!   - The EPB arm lacks the E-INP-010 OOB-on-non-empty discriminant (interface_id
//!     OOB on a non-empty table) → that path currently produces no error.
//!   - The EPB arm E-INP-009 message does not match the BC-required exact format.
//!   - PC6b (padding-aware overhead check) is not coded → cannot fire.
//!
//! # What STAYS GREEN
//!
//! STORY-123/124's existing tests in `bc_2_01_story123_pcapng_tests.rs` and
//! `bc_2_01_story124_idb_tests.rs` MUST remain GREEN. This file does NOT touch
//! any previously implemented path.
//!
//! # Coverage map (AC → test → E-INP code)
//!
//! ```
//! AC-001 → test_BC_2_01_012_epb_body_short_e_inp_008  (E-INP-008)
//!        → test_BC_2_01_012_no_panic_malformed         (no panic)
//! AC-002 → test_BC_2_01_012_interface_id_bounds_check  (E-INP-009, empty-table path)
//! AC-003 → test_BC_2_01_012_interface_id_bounds_check  (E-INP-010, OOB-non-empty path)
//! AC-004 → test_BC_2_01_012_guard_before_allocate      (E-INP-008, PC6a + PC6b)
//! AC-005 → test_BC_2_01_012_data_bounded_by_captured_len
//! AC-006 → test_BC_2_01_012_zero_byte_captured_len
//! AC-007 → test_BC_2_01_012_max_boundary_captured_len  (E-INP-008 at one-over)
//! AC-008 → test_BC_2_01_012_raw_block_path_not_crate_duration
//! AC-009 → test_BC_2_01_014_regression_1000x_bug       (integration; fails on hardcoded-6)
//! AC-010 → test_BC_2_01_014_usecs_default_matches_classic_pcap
//!        → test_BC_2_01_014_fast_path_saturation_guard (EC-013; E-INP-none; ts_sec=u32::MAX)
//! AC-011 → test_BC_2_01_014_e127_no_panic              (EC-014; base-2 e=127 clamp)
//!        → test_BC_2_01_014_base2_e20_known_vector      (EC-015; ticks=1_048_576)
//! AC-012 → VP-025 Kani harness in tests/kani_proofs.rs (formal; gated #[cfg(kani)])
//! AC-013 → test_BC_2_01_012_happy_path_n_packet_order_and_byte_fidelity
//! ```
//!
//! # Discriminating-assertion discipline
//!
//! Every error test asserts:
//!   (a) the EXPECTED E-INP code IS present in the error message, AND
//!   (b) at least one SIBLING code is ABSENT (to guard against false positives).
//!
//! Naming convention: `test_BC_S_SS_NNN_<assertion>()` throughout.
//! `#![allow(non_snake_case)]` required per factory BC-naming mandate.
#![allow(non_snake_case)]
#![allow(clippy::needless_range_loop)]

use std::io::Cursor;

use wirerust::reader::{PcapSource, pcapng_timestamp_to_secs_usecs};

// ── pcapng canonical constants (mirrors ADR-009 / bc_2_01_story123_pcapng_tests) ─

/// pcapng SHB block type / file magic (endian-independent 4-byte literal).
const SHB_BLOCK_TYPE: u32 = 0x0A0D_0D0A;

/// IDB block type code.
const IDB_BLOCK_TYPE: u32 = 0x0000_0001;

/// EPB block type code.
const EPB_BLOCK_TYPE: u32 = 0x0000_0006;

/// BOM: little-endian pcapng section (on-disk 4D 3C 2B 1A).
const SHB_BOM_LE: [u8; 4] = [0x4D, 0x3C, 0x2B, 0x1A];

/// BOM: big-endian pcapng section (on-disk 1A 2B 3C 4D).
const SHB_BOM_BE: [u8; 4] = [0x1A, 0x2B, 0x3C, 0x4D];

/// EPB body minimum: 20 bytes (interface_id:4 + ts_high:4 + ts_low:4 +
/// captured_len:4 + original_len:4).
const EPB_BODY_FIXED_BYTES: usize = 20;

/// pcapng if_tsresol option code (IDB option).
const OPT_IF_TSRESOL: u16 = 9;

/// pcapng opt_endofopt code.
const OPT_ENDOFOPT: u16 = 0;

// ── Error-code discriminant strings (canonical per error-taxonomy) ────────────
//
// These are the ONLY string fragments tests may use to identify E-INP codes.
// They MUST match the error messages produced by `src/reader.rs` after implementation.

const E_INP_008: &str = "E-INP-008";
const E_INP_009: &str = "E-INP-009";
const E_INP_010: &str = "E-INP-010";

// ── Fixture builder helpers ───────────────────────────────────────────────────

/// Build a minimal 28-byte SHB-only pcapng (LE, major=1, minor=0).
fn shb_only_le() -> Vec<u8> {
    let mut buf = Vec::with_capacity(28);
    buf.extend_from_slice(&SHB_BLOCK_TYPE.to_le_bytes());
    buf.extend_from_slice(&28u32.to_le_bytes());
    buf.extend_from_slice(&SHB_BOM_LE);
    buf.extend_from_slice(&1u16.to_le_bytes()); // major = 1
    buf.extend_from_slice(&0u16.to_le_bytes()); // minor = 0
    buf.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes());
    buf.extend_from_slice(&28u32.to_le_bytes());
    assert_eq!(buf.len(), 28);
    buf
}

/// Build a minimal 20-byte IDB block (LE, ETHERNET, no options, snaplen=65535).
///
/// No if_tsresol TLV → if_tsresol defaults to 6 (microseconds, pcapng spec default).
fn idb_le_ethernet_default_tsresol() -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&IDB_BLOCK_TYPE.to_le_bytes());
    buf.extend_from_slice(&20u32.to_le_bytes()); // btl = 20
    buf.extend_from_slice(&1u16.to_le_bytes()); // linktype = ETHERNET (1)
    buf.extend_from_slice(&0u16.to_le_bytes()); // reserved = 0
    buf.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
    buf.extend_from_slice(&20u32.to_le_bytes()); // trailing btl
    assert_eq!(buf.len(), 20);
    buf
}

/// Build an IDB block (LE, ETHERNET) with an explicit if_tsresol TLV option.
///
/// Block layout:
///   outer header (12):  block_type(4) + btl(4) + trailing btl(4)
///   IDB fixed body (8): linktype(2) + reserved(2) + snaplen(4)
///   TLV options (8):    opt_code(2) + opt_len(2) + value(1) + pad(3)
///   opt_endofopt (4):   0x0000(2) + 0x0000(2)
///
/// body = 8 + 8 + 4 = 20 bytes; btl = 12 + 20 = 32 bytes.
fn idb_le_ethernet_with_tsresol(if_tsresol: u8) -> Vec<u8> {
    let btl: u32 = 32;
    let mut buf = Vec::new();
    buf.extend_from_slice(&IDB_BLOCK_TYPE.to_le_bytes());
    buf.extend_from_slice(&btl.to_le_bytes());
    // IDB fixed body (8 bytes)
    buf.extend_from_slice(&1u16.to_le_bytes()); // linktype = ETHERNET
    buf.extend_from_slice(&0u16.to_le_bytes()); // reserved
    buf.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
    // if_tsresol TLV (8 bytes: code(2) + len(2) + value(1) + pad(3))
    buf.extend_from_slice(&OPT_IF_TSRESOL.to_le_bytes()); // opt_code = 9
    buf.extend_from_slice(&1u16.to_le_bytes()); // opt_len = 1
    buf.push(if_tsresol); // value byte
    buf.extend_from_slice(&[0u8; 3]); // 3-byte pad to 4-byte alignment
    // opt_endofopt (4 bytes)
    buf.extend_from_slice(&OPT_ENDOFOPT.to_le_bytes()); // code = 0
    buf.extend_from_slice(&0u16.to_le_bytes()); // len = 0
    // trailing btl
    buf.extend_from_slice(&btl.to_le_bytes());
    assert_eq!(buf.len(), 32, "IDB with if_tsresol TLV must be 32 bytes");
    buf
}

/// Build an EPB block (LE) with the given fields and packet data.
///
/// EPB body (body_len = 20 + data.len() + pad):
///   interface_id(4) + ts_high(4) + ts_low(4) + captured_len(4) + original_len(4) + data + pad
///
/// btl = 12 + body_len (padded to 4-byte alignment).
fn epb_le(
    interface_id: u32,
    ts_high: u32,
    ts_low: u32,
    captured_len: u32,
    original_len: u32,
    data: &[u8],
) -> Vec<u8> {
    let data_len = data.len();
    let pad_len = (4usize.wrapping_sub(data_len % 4)) % 4;
    let body_len = EPB_BODY_FIXED_BYTES + data_len + pad_len;
    let btl: u32 = (12 + body_len) as u32;

    let mut buf = Vec::new();
    buf.extend_from_slice(&EPB_BLOCK_TYPE.to_le_bytes());
    buf.extend_from_slice(&btl.to_le_bytes());
    // EPB fixed body fields
    buf.extend_from_slice(&interface_id.to_le_bytes());
    buf.extend_from_slice(&ts_high.to_le_bytes());
    buf.extend_from_slice(&ts_low.to_le_bytes());
    buf.extend_from_slice(&captured_len.to_le_bytes());
    buf.extend_from_slice(&original_len.to_le_bytes());
    // packet data + padding
    buf.extend_from_slice(data);
    buf.extend_from_slice(&vec![0u8; pad_len]);
    // trailing btl
    buf.extend_from_slice(&btl.to_le_bytes());
    assert_eq!(buf.len(), btl as usize, "EPB block must be btl bytes");
    buf
}

/// Build a genuine BE EPB block with given fields and data.
///
/// Used for BC-2.01.010 Inv4 endianness coverage: interface_id and timestamp
/// fields decoded as BE (non-palindromic values detect LE misreads).
fn epb_be(
    interface_id: u32,
    ts_high: u32,
    ts_low: u32,
    captured_len: u32,
    original_len: u32,
    data: &[u8],
) -> Vec<u8> {
    let data_len = data.len();
    let pad_len = (4usize.wrapping_sub(data_len % 4)) % 4;
    let body_len = EPB_BODY_FIXED_BYTES + data_len + pad_len;
    let btl: u32 = (12 + body_len) as u32;

    let mut buf = Vec::new();
    buf.extend_from_slice(&EPB_BLOCK_TYPE.to_be_bytes());
    buf.extend_from_slice(&btl.to_be_bytes());
    buf.extend_from_slice(&interface_id.to_be_bytes());
    buf.extend_from_slice(&ts_high.to_be_bytes());
    buf.extend_from_slice(&ts_low.to_be_bytes());
    buf.extend_from_slice(&captured_len.to_be_bytes());
    buf.extend_from_slice(&original_len.to_be_bytes());
    buf.extend_from_slice(data);
    buf.extend_from_slice(&vec![0u8; pad_len]);
    buf.extend_from_slice(&btl.to_be_bytes());
    assert_eq!(buf.len(), btl as usize);
    buf
}

/// Build a minimal SHB-only pcapng (genuine BE, all fields BE-encoded).
fn shb_only_be() -> Vec<u8> {
    let mut buf = Vec::with_capacity(28);
    buf.extend_from_slice(&SHB_BLOCK_TYPE.to_be_bytes()); // 0A 0D 0D 0A (endian-indep)
    buf.extend_from_slice(&28u32.to_be_bytes()); // 00 00 00 1C
    buf.extend_from_slice(&SHB_BOM_BE); // 1A 2B 3C 4D
    buf.extend_from_slice(&1u16.to_be_bytes()); // 00 01
    buf.extend_from_slice(&0u16.to_be_bytes()); // 00 00
    buf.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_be_bytes());
    buf.extend_from_slice(&28u32.to_be_bytes()); // 00 00 00 1C
    assert_eq!(buf.len(), 28);
    buf
}

/// Build a minimal IDB (genuine BE, ETHERNET).
fn idb_be_ethernet_default_tsresol() -> Vec<u8> {
    let btl: u32 = 20;
    let mut buf = Vec::new();
    buf.extend_from_slice(&IDB_BLOCK_TYPE.to_be_bytes());
    buf.extend_from_slice(&btl.to_be_bytes());
    buf.extend_from_slice(&1u16.to_be_bytes()); // linktype = ETHERNET (BE)
    buf.extend_from_slice(&0u16.to_be_bytes()); // reserved
    buf.extend_from_slice(&65535u32.to_be_bytes()); // snaplen (BE)
    buf.extend_from_slice(&btl.to_be_bytes());
    assert_eq!(buf.len(), 20);
    buf
}

/// Build an EPB with a body that is shorter than EPB_BODY_FIXED_BYTES (20 bytes).
///
/// This exercises EC-011 / AC-001: btl in [12, 32) → body < 20 bytes → E-INP-008.
///
/// btl = 12 + body_len; to get body < 20, use btl ∈ [12, 32).
/// We pick btl = 28 → body = 16 bytes (12 bytes for outer + 16 body = 28 total).
fn epb_le_body_short(body_bytes: &[u8]) -> Vec<u8> {
    let btl: u32 = (12 + body_bytes.len()) as u32;
    let mut buf = Vec::new();
    buf.extend_from_slice(&EPB_BLOCK_TYPE.to_le_bytes());
    buf.extend_from_slice(&btl.to_le_bytes());
    buf.extend_from_slice(body_bytes);
    buf.extend_from_slice(&btl.to_le_bytes()); // trailing btl
    buf
}

/// Build a complete LE pcapng: SHB + IDB (default tsresol) + one EPB.
fn le_pcapng_with_one_epb(
    ts_high: u32,
    ts_low: u32,
    captured_len: u32,
    original_len: u32,
    data: &[u8],
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&shb_only_le());
    buf.extend_from_slice(&idb_le_ethernet_default_tsresol());
    buf.extend_from_slice(&epb_le(
        0,
        ts_high,
        ts_low,
        captured_len,
        original_len,
        data,
    ));
    buf
}

/// Build a complete LE pcapng: SHB + IDB (explicit if_tsresol) + one EPB.
fn le_pcapng_with_tsresol_and_one_epb(
    if_tsresol: u8,
    ts_high: u32,
    ts_low: u32,
    captured_len: u32,
    original_len: u32,
    data: &[u8],
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&shb_only_le());
    buf.extend_from_slice(&idb_le_ethernet_with_tsresol(if_tsresol));
    buf.extend_from_slice(&epb_le(
        0,
        ts_high,
        ts_low,
        captured_len,
        original_len,
        data,
    ));
    buf
}

/// Build a complete BE pcapng: SHB + IDB (default tsresol) + one EPB.
///
/// All multi-byte fields are genuine big-endian (non-palindromic values detect
/// LE misreads — BC-2.01.010 Invariant 4 / BC-2.01.012 coverage for endianness).
fn be_pcapng_with_one_epb(
    ts_high: u32,
    ts_low: u32,
    captured_len: u32,
    original_len: u32,
    data: &[u8],
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&shb_only_be());
    buf.extend_from_slice(&idb_be_ethernet_default_tsresol());
    buf.extend_from_slice(&epb_be(
        0,
        ts_high,
        ts_low,
        captured_len,
        original_len,
        data,
    ));
    buf
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.01.014 UNIT TESTS (pure-core timestamp helper)
// These call pcapng_timestamp_to_secs_usecs directly.
// ALL fail with todo!() panic until implementation exists.
// ─────────────────────────────────────────────────────────────────────────────

/// AC-010 / BC-2.01.014 PC4 / EC-008: default µs fast path (if_tsresol=6).
///
/// Canonical test vector from BC-2.01.014:
///   ts_high=0, ts_low=1_000_000, if_tsresol=6 → (1, 0) — 1 second exactly
///   ts_high=0, ts_low=1_500_000, if_tsresol=6 → (1, 500_000) — 1.5 seconds
///
/// This also verifies that the helper produces the same result as classic-pcap
/// microsecond conversion for the ts_high==0 domain (BC-2.01.014 Invariant 2).
///
/// FAILS: todo!() panics.
#[test]
fn test_BC_2_01_014_usecs_default_matches_classic_pcap() {
    // Vector 1: 1 second exactly
    let (ts_sec, ts_usecs) = pcapng_timestamp_to_secs_usecs(0, 1_000_000, 6);
    assert_eq!(
        ts_sec, 1,
        "BC-2.01.014 PC4: ts_high=0, ts_low=1_000_000, if_tsresol=6 → ts_sec=1; got {ts_sec}"
    );
    assert_eq!(
        ts_usecs, 0,
        "BC-2.01.014 PC4: ts_high=0, ts_low=1_000_000, if_tsresol=6 → ts_usecs=0; got {ts_usecs}"
    );

    // Vector 2: 1.5 seconds
    let (ts_sec, ts_usecs) = pcapng_timestamp_to_secs_usecs(0, 1_500_000, 6);
    assert_eq!(
        ts_sec, 1,
        "BC-2.01.014 PC4: ts_high=0, ts_low=1_500_000, if_tsresol=6 → ts_sec=1; got {ts_sec}"
    );
    assert_eq!(
        ts_usecs, 500_000,
        "BC-2.01.014 PC4: ts_high=0, ts_low=1_500_000, if_tsresol=6 → ts_usecs=500_000; got \
         {ts_usecs}"
    );

    // Vector 3: zero-epoch (EC-001 / EC-003)
    let (ts_sec, ts_usecs) = pcapng_timestamp_to_secs_usecs(0, 0, 6);
    assert_eq!(ts_sec, 0, "epoch: ts_sec must be 0; got {ts_sec}");
    assert_eq!(ts_usecs, 0, "epoch: ts_usecs must be 0; got {ts_usecs}");

    // Invariant 3: ts_usecs must always be in [0, 999_999]
    assert!(
        ts_usecs <= 999_999,
        "BC-2.01.014 Invariant 3: ts_usecs must be in [0, 999_999]; got {ts_usecs}"
    );
}

/// AC-010 / BC-2.01.014 PC4 / EC-013: µs fast path saturation guard (M-3).
///
/// CORRECTED test vector (BC-2.01.014 EC-013 canonical vector contained an arithmetic
/// error — see PO follow-up note below):
///   ts_high=2_000_000, ts_low=0, if_tsresol=6
///   ticks = 2_000_000u64 << 32 = 2_000_000 * 4_294_967_296 = 8_589_934_592_000_000
///   ticks / 1_000_000 = 8_589_934_592 which exceeds u32::MAX (4_294_967_295) → saturates ✓
///   ts_sec = u32::MAX (saturation via .min(u32::MAX as u64) — mandatory)
///   ts_usecs = 8_589_934_592_000_000 % 1_000_000 = 0 (8_589_934_592_000_000 is an exact
///              multiple of 1_000_000; lower 32 bits are 0 and 2_000_000 * 967_296 mod
///              1_000_000 = 0 since 2_000_000 * 967_296 = 1_934_592_000_000 mod 1_000_000 = 0)
///
/// A bare `as u32` cast would WRAP (not saturate), producing a value < u32::MAX.
/// This is the FAST-PATH SATURATION test that VP-025 Kani harness must also cover.
///
/// NOTE FOR PO — BC-2.01.014 CANONICAL VECTOR ERROR (do NOT fix here; route to PO):
///   The BC-2.01.014 EC-013 canonical vector table claims:
///     ts_high=4295, ticks = 4295 * 2^32 = 18_448_744_073_709_551_616
///   That claimed ticks value (18_448_744_073_709_551_616) EXCEEDS u64::MAX
///   (18_446_744_073_709_551_615) — it is impossible as a u64. The actual value of
///   4295u64 << 32 is only 18_446_884_536_320, which divided by 1_000_000 yields
///   18_446_884 — far below u32::MAX (4_294_967_295). Saturation does NOT trigger for
///   ts_high=4295. The BC must be updated to use a vector where ts_high > ~1_000_000
///   (e.g. ts_high=2_000_000 as used here, or ts_high=4_295 is wrong by 3 orders of
///   magnitude — the author appears to have confused 4295 with 4_295_000 or similar).
///   Corrected vector: ts_high=2_000_000, ts_low=0, if_tsresol=6 → ts_sec=u32::MAX, ts_usecs=0.
#[test]
fn test_BC_2_01_014_fast_path_saturation_guard() {
    // ts_high=2_000_000, ts_low=0, if_tsresol=6
    // ticks = 2_000_000u64 << 32 = 2_000_000 * 4_294_967_296 = 8_589_934_592_000_000
    // ticks / 1_000_000 = 8_589_934_592
    // 8_589_934_592 > u32::MAX (4_294_967_295) → .min(u32::MAX as u64) clamps to u32::MAX ✓
    // ticks % 1_000_000 = 8_589_934_592_000_000 % 1_000_000 = 0
    let (ts_sec, ts_usecs) = pcapng_timestamp_to_secs_usecs(2_000_000, 0, 6);
    assert_eq!(
        ts_sec,
        u32::MAX,
        "BC-2.01.014 EC-013 / M-3: ts_high=2_000_000 → ticks/1_000_000=8_589_934_592 > u32::MAX; \
         ts_sec MUST saturate at u32::MAX (not wrap); got {ts_sec}"
    );
    // ts_usecs: ticks = 2_000_000u64 << 32 = 8_589_934_592_000_000
    // 8_589_934_592_000_000 % 1_000_000 = 0 (exact multiple — ts_low=0 contributes no remainder,
    // and 2_000_000 * (4_294_967_296 % 1_000_000) = 2_000_000 * 967_296 = 1_934_592_000_000,
    // 1_934_592_000_000 % 1_000_000 = 0).
    assert_eq!(
        ts_usecs, 0,
        "BC-2.01.014 EC-013: ts_high=2_000_000, ts_low=0, if_tsresol=6 → ts_usecs=0 \
         (ticks is exact multiple of 1_000_000); got {ts_usecs}"
    );
    // No panic: the function returned normally (if it panicked this assert is unreachable).
}

/// AC-009 / BC-2.01.014 PC5: nanosecond resolution (if_tsresol=9).
///
/// This is the 1000×-bug guard. The current EPB arm hard-codes DEFAULT_TSRESOL=6
/// for ALL interfaces. A pcapng with if_tsresol=9 (nanoseconds) produces ticks
/// that are 1000× finer than microsecond ticks. The hardcoded-6 path treats them
/// as microseconds, producing timestamps that are 1000× too large.
///
/// Canonical test vector from BC-2.01.014:
///   ts_high=0, ts_low=1_500_000_000, if_tsresol=9 → (1, 500_000)
///   (1.5 billion nanosecond ticks = 1.5 seconds = 1 sec + 500_000 µs)
///
/// WRONG result with hardcoded if_tsresol=6:
///   ticks / 1_000_000 = 1_500_000_000 / 1_000_000 = 1_500 seconds  ← 1000× too large!
///   ts_sec = 1500, ts_usecs = 0  ← WRONG
///
/// CORRECT result with if_tsresol=9:
///   ticks_per_sec = 1_000_000_000
///   ts_sec = 1_500_000_000 / 1_000_000_000 = 1
///   ts_usecs = (1_500_000_000 % 1_000_000_000) / 1000 = 500_000_000 / 1000 = 500_000
///
/// FAILS: todo!() panics.
#[test]
fn test_BC_2_01_014_nanosecond_resolution_correct() {
    // Canonical test vector (BC-2.01.014 EC-010 and canonical table):
    let (ts_sec, ts_usecs) = pcapng_timestamp_to_secs_usecs(0, 1_500_000_000, 9);

    // Expected correct output: 1.5 seconds
    assert_eq!(
        ts_sec, 1,
        "BC-2.01.014 PC5 nanosecond guard: ts_high=0, ts_low=1_500_000_000, if_tsresol=9 \
         → ts_sec MUST be 1 (not 1500 as hardcoded-6 would produce); got {ts_sec}"
    );
    assert_eq!(
        ts_usecs, 500_000,
        "BC-2.01.014 PC5 nanosecond guard: ts_high=0, ts_low=1_500_000_000, if_tsresol=9 \
         → ts_usecs MUST be 500_000; got {ts_usecs}"
    );

    // Additional vector from BC-2.01.014 EC-003:
    // ts_high=0, ts_low=1_500_000_000_500, if_tsresol=9 → ts_sec=1500, ts_usecs=0
    // (500 ns rounds down to 0 µs)
    // Note: ts_low only holds u32 (max 4_294_967_295), so this ticks value requires ts_high.
    // Use a representable value: ts_high=0, ts_low=2_000_000_500, if_tsresol=9
    // → ts_sec = 2, ts_usecs = 0 (500 ns < 1 µs → rounds down)
    let (ts_sec2, ts_usecs2) = pcapng_timestamp_to_secs_usecs(0, 2_000_000_500, 9);
    assert_eq!(
        ts_sec2, 2,
        "BC-2.01.014 EC-010: ts_low=2_000_000_500, if_tsresol=9 → ts_sec=2; got {ts_sec2}"
    );
    assert_eq!(
        ts_usecs2, 0,
        "BC-2.01.014 EC-010: 500 ns remainder rounds DOWN to 0 µs; got {ts_usecs2}"
    );
}

/// AC-010 / BC-2.01.014 PC2: base-10 if_tsresol=0 (1 tick/sec, 1-second resolution).
///
/// EC-004: if_tsresol=0 → ticks_per_sec = 10^0 = 1.
///   ts_sec = ticks / 1 = ticks (saturated at u32::MAX).
///   ts_usecs = 0 (no sub-second remainder in 1-tick/sec resolution).
///
/// FAILS: todo!() panics.
#[test]
fn test_BC_2_01_014_base10_e0_one_tick_per_sec() {
    // if_tsresol=0 → base-10, e=0, ticks_per_sec=1.
    let (ts_sec, ts_usecs) = pcapng_timestamp_to_secs_usecs(0, 5, 0);
    assert_eq!(
        ts_sec, 5,
        "BC-2.01.014 EC-004: if_tsresol=0, ts_low=5 → ts_sec=5; got {ts_sec}"
    );
    assert_eq!(
        ts_usecs, 0,
        "BC-2.01.014 EC-004: 1 tick/sec → no sub-second component; ts_usecs=0; got {ts_usecs}"
    );
}

/// AC-011 / BC-2.01.014 PC3 / EC-014: base-2 e=127 (if_tsresol=0xFF) must NOT panic.
///
/// if_tsresol=0xFF: bit7=1 (base-2), e = 0xFF & 0x7F = 127.
/// Without clamping: 1u64 << 127 panics with overflow-checks=true.
/// With clamping: e_clamped = 127.min(63) = 63; ticks_per_sec = 1u64 << 63.
/// For any realistic ticks value, ts_sec = 0, ts_usecs = 0.
///
/// FAILS: todo!() panics (the outer todo!() fires before the inner shift panic).
#[test]
fn test_BC_2_01_014_e127_no_panic() {
    // ts_high=0, ts_low=0, if_tsresol=0xFF (e=127, base-2)
    // Expected: (0, 0) — no panic (e clamped to 63, ticks_per_sec = 2^63, ticks = 0)
    let (ts_sec, ts_usecs) = pcapng_timestamp_to_secs_usecs(0, 0, 0xFF);
    assert_eq!(
        ts_sec, 0,
        "BC-2.01.014 EC-014: if_tsresol=0xFF, ticks=0 → ts_sec=0; got {ts_sec}"
    );
    assert_eq!(
        ts_usecs, 0,
        "BC-2.01.014 EC-014: if_tsresol=0xFF, ticks=0 → ts_usecs=0; got {ts_usecs}"
    );
    // Non-zero ticks but still 0 relative to 2^63 ticks/sec.
    // Any ts_low value < 2^63 → ts_sec = 0.
    let (ts_sec2, ts_usecs2) = pcapng_timestamp_to_secs_usecs(0, 1_000_000, 0xFF);
    assert_eq!(
        ts_sec2, 0,
        "BC-2.01.014 EC-014: if_tsresol=0xFF, 1_000_000 ticks with 2^63 ticks/sec → ts_sec=0; \
         got {ts_sec2}"
    );
    assert!(
        ts_usecs2 <= 999_999,
        "BC-2.01.014 Invariant 3: ts_usecs must be in [0, 999_999]; got {ts_usecs2}"
    );
}

/// AC-011 / BC-2.01.014 PC3 / EC-015: base-2 e=20 (if_tsresol=0x94) known vector.
///
/// Canonical test vector from BC-2.01.014:
///   ts_high=0, ts_low=1_048_576, if_tsresol=0x94 (base-2, e=20)
///   ticks_per_sec = 1 << 20 = 1_048_576
///   ts_sec = 1_048_576 / 1_048_576 = 1
///   ts_usecs = 0 (remainder = 0)
///
/// FAILS: todo!() panics.
#[test]
fn test_BC_2_01_014_base2_e20_known_vector() {
    // 0x94 = 0b10010100 → bit7=1 (base-2), e = 0x94 & 0x7F = 0x14 = 20.
    let (ts_sec, ts_usecs) = pcapng_timestamp_to_secs_usecs(0, 1_048_576, 0x94);
    assert_eq!(
        ts_sec, 1,
        "BC-2.01.014 EC-015: if_tsresol=0x94 (base-2 e=20), ts_low=1_048_576 → ts_sec=1; \
         got {ts_sec}"
    );
    assert_eq!(
        ts_usecs, 0,
        "BC-2.01.014 EC-015: 1_048_576 / 1_048_576 = 1 exactly; ts_usecs=0; got {ts_usecs}"
    );
}

/// AC-011 / BC-2.01.014 Invariant 1: saturation / totality for extreme u64 ticks.
///
/// EC-004/007: ts_high=u32::MAX, ts_low=u32::MAX (maximum 64-bit tick value).
/// The function MUST NOT panic regardless of if_tsresol value.
/// ts_sec MUST saturate at u32::MAX (BC-2.01.014 PC6).
///
/// FAILS: todo!() panics.
#[test]
fn test_BC_2_01_014_saturation_extreme_ticks() {
    // Maximum u64 ticks (ts_high=u32::MAX, ts_low=u32::MAX).
    // With if_tsresol=6: ticks = 0xFFFF_FFFF_FFFF_FFFF / 1_000_000 >> u32::MAX → saturate.
    let (ts_sec, ts_usecs) = pcapng_timestamp_to_secs_usecs(u32::MAX, u32::MAX, 6);
    assert_eq!(
        ts_sec,
        u32::MAX,
        "BC-2.01.014 PC6: max ticks with µs resolution → ts_sec saturates at u32::MAX; \
         got {ts_sec}"
    );
    assert!(
        ts_usecs <= 999_999,
        "BC-2.01.014 Invariant 3: ts_usecs must be in [0, 999_999]; got {ts_usecs}"
    );

    // With if_tsresol=0 (1 tick/sec): ticks=u64::MAX → ts_sec saturates at u32::MAX.
    let (ts_sec2, ts_usecs2) = pcapng_timestamp_to_secs_usecs(u32::MAX, u32::MAX, 0);
    assert_eq!(
        ts_sec2,
        u32::MAX,
        "BC-2.01.014 PC6: max ticks with 1 tick/sec → ts_sec saturates at u32::MAX; \
         got {ts_sec2}"
    );
    assert_eq!(
        ts_usecs2, 0,
        "BC-2.01.014: 1 tick/sec → no sub-second component; ts_usecs=0; got {ts_usecs2}"
    );

    // With if_tsresol=0xFF (base-2, e=127 → clamped to 63): ticks/2^63 ≤ u32::MAX for
    // u64 max ticks (u64::MAX / 2^63 = 1). → ts_sec=1.
    let (ts_sec3, ts_usecs3) = pcapng_timestamp_to_secs_usecs(u32::MAX, u32::MAX, 0xFF);
    // u64::MAX / (1u64 << 63) = u64::MAX / 9_223_372_036_854_775_808 = 1 (integer division).
    assert_eq!(
        ts_sec3, 1,
        "BC-2.01.014 EC-014: max ticks / 2^63 = 1; ts_sec=1; got {ts_sec3}"
    );
    assert!(
        ts_usecs3 <= 999_999,
        "BC-2.01.014 Invariant 3: ts_usecs must be in [0, 999_999]; got {ts_usecs3}"
    );
}

/// BC-2.01.014 Invariant 3: ts_usecs always in [0, 999_999] for representative inputs.
///
/// Tests a range of representative (ts_low, if_tsresol) pairs to verify the invariant
/// holds. Complements the Kani VP-025 proof which covers the full symbolic space.
///
/// FAILS: todo!() panics.
#[test]
fn test_BC_2_01_014_invariant_ts_usecs_in_range() {
    let test_vectors: &[(u32, u32, u8)] = &[
        (0, 999_999, 6),       // 999_999 µs ticks — just under 1 sec
        (0, 1_000_001, 6),     // just over 1 second boundary
        (0, 1_500_000_499, 9), // 1.5 seconds + 499 ns (rounds down to 499 µs)
        (0, 1_048_575, 0x94),  // one tick under 1 second at e=20
        (1, 0, 6),             // ts_high=1 (2^32 µs ticks)
        (0, u32::MAX, 6),      // max ts_low with µs resolution
    ];
    for &(ts_high, ts_low, if_tsresol) in test_vectors {
        let (ts_sec, ts_usecs) = pcapng_timestamp_to_secs_usecs(ts_high, ts_low, if_tsresol);
        assert!(
            ts_usecs <= 999_999,
            "BC-2.01.014 Invariant 3: ts_usecs must be in [0, 999_999]; \
             ts_high={ts_high}, ts_low={ts_low}, if_tsresol={if_tsresol:#04X} \
             → ts_sec={ts_sec}, ts_usecs={ts_usecs}"
        );
        // ts_sec must not exceed u32::MAX (trivially true since it is u32, but verifies
        // the saturation path did not panic).
        let _ = ts_sec; // consume
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.01.012 EPB PARSE TESTS (via from_pcap_reader)
// These exercise the EPB arm of the block-walk loop.
// ─────────────────────────────────────────────────────────────────────────────

/// AC-001 / BC-2.01.012 PC9 step (i) / EC-011: EPB body shorter than 20 bytes → E-INP-008.
///
/// Canonical test vector: EPB with btl=28 → body = 28 - 12 = 16 bytes < 20 EPB fixed bytes.
/// wirerust MUST return E-INP-008 (body-too-short; wirerust body-decode path).
/// MUST NOT return E-INP-010 (that is for crate framing rejection).
///
/// Discriminating assertion:
///   assert E-INP-008 IS in error message
///   assert E-INP-010 IS NOT in error message
///
/// This test targets the existing wirerust body-len check at lines 802-809 of reader.rs.
/// This test is CURRENTLY GREEN because the check is already implemented — but we
/// include it here as the AC-001 anchor and to verify the correct error code.
/// (The RED Gate for STORY-125 is the timestamp tests and the OOB-non-empty test below.)
#[test]
fn test_BC_2_01_012_epb_body_short_e_inp_008() {
    // btl = 28 → body = 16 bytes (< 20 EPB_BODY_FIXED_BYTES) → E-INP-008.
    // Build: SHB + IDB + EPB-with-16-byte-body
    let mut buf = shb_only_le();
    buf.extend_from_slice(&idb_le_ethernet_default_tsresol());
    // EPB with 16-byte body (NOT a valid EPB — body < EPB_BODY_FIXED_BYTES).
    // btl = 12 + 16 = 28; body = 16 bytes (all zeros).
    let body_16 = [0u8; 16];
    buf.extend_from_slice(&epb_le_body_short(&body_16));

    let result = PcapSource::from_pcap_reader(Cursor::new(buf));
    assert!(
        result.is_err(),
        "AC-001 / EC-011: EPB body=16 < 20 bytes must return Err; got Ok"
    );
    let err = format!("{:#}", result.unwrap_err());
    assert!(
        err.contains(E_INP_008),
        "AC-001: error MUST contain E-INP-008 (body-too-short; wirerust body-decode); got: {err}"
    );
    assert!(
        !err.contains(E_INP_010),
        "AC-001: error MUST NOT contain E-INP-010 (that is for crate framing rejection); \
         got: {err}"
    );
}

/// AC-001 / BC-2.01.012 AC-003: no panic on any malformed EPB body.
///
/// Tests multiple malformed body lengths (0, 4, 8, 12, 16, 19 bytes) — all
/// shorter than EPB_BODY_FIXED_BYTES (20). wirerust MUST return Err, never panic.
///
/// Note: for body lengths < 12 (btl < 24), the crate may reject with its own
/// framing error before wirerust sees the block. This is acceptable — crate
/// framing rejection is E-INP-010. The test just verifies no panic.
#[test]
fn test_BC_2_01_012_no_panic_malformed() {
    // Test body lengths from 0 to 19 (all < EPB_BODY_FIXED_BYTES).
    // btl = 12 + body_len; the crate requires btl >= 12 and trailing btl match.
    // For body_len = 0: btl = 12. For body_len=19: btl=31.
    // The crate may reject very short btl values with IncompleteBuffer.
    // We only assert: no panic; result is Err.
    for body_len in [0usize, 4, 8, 12, 16, 19] {
        let mut buf = shb_only_le();
        buf.extend_from_slice(&idb_le_ethernet_default_tsresol());
        // Build a raw EPB with the given body_len.
        // btl = 12 + body_len.
        let btl = (12 + body_len) as u32;
        let mut epb = Vec::new();
        epb.extend_from_slice(&EPB_BLOCK_TYPE.to_le_bytes());
        epb.extend_from_slice(&btl.to_le_bytes());
        epb.extend_from_slice(&vec![0u8; body_len]);
        epb.extend_from_slice(&btl.to_le_bytes());
        buf.extend_from_slice(&epb);

        let result = PcapSource::from_pcap_reader(Cursor::new(buf));
        assert!(
            result.is_err(),
            "AC-003: EPB body_len={body_len} < 20 must return Err (no panic); got Ok"
        );
        // Error must be E-INP-008 (wirerust body-decode) or E-INP-010 (crate framing).
        // Either is acceptable here — the discriminant is between crate-framing and
        // wirerust-body-decode; both are valid rejection codes for body < 20.
        let err = format!("{:#}", result.unwrap_err());
        assert!(
            err.contains(E_INP_008) || err.contains(E_INP_010),
            "AC-003: malformed EPB must produce E-INP-008 or E-INP-010; body_len={body_len}; \
             got: {err}"
        );
    }
}

/// AC-002 + AC-003 / BC-2.01.012 PC5a + PC5b / EC-005 + EC-006:
/// interface_id discriminant split — E-INP-009 (empty-table) vs E-INP-010 (OOB-non-empty).
///
/// TWO discriminating sub-tests in one function (per STORY-125 Test Plan AC-002/AC-003):
///
/// Sub-test A (empty-table → E-INP-009):
///   EPB with interface_id=0, NO IDB in stream → must return E-INP-009 (not E-INP-010).
///
/// Sub-test B (OOB-on-non-empty → E-INP-010):
///   EPB with interface_id=5, ONE IDB (index 0 only) → must return E-INP-010 (not E-INP-009).
///   The two error codes MUST be different (BC-2.01.012 AC-001 / AC-003).
///
/// RED: The OOB-non-empty path (sub-test B) is not implemented:
///   The current code only checks `interfaces.is_empty()` (E-INP-009), but does NOT
///   check `interface_id >= interfaces.len()` for a non-empty table (should be E-INP-010).
///   Sub-test B will return Ok or panic; it will NOT return E-INP-010.
#[test]
fn test_BC_2_01_012_interface_id_bounds_check() {
    // ── Sub-test A: empty interface table → E-INP-009 ─────────────────────────
    // pcapng: SHB + EPB (no IDB → interface table is empty).
    {
        let mut buf = shb_only_le();
        // NO IDB — interface table will be empty when EPB is encountered.
        // EPB: interface_id=0 (valid value, but table is empty).
        buf.extend_from_slice(&epb_le(0, 0, 1_000_000, 4, 4, &[0xAA, 0xBB, 0xCC, 0xDD]));

        let result = PcapSource::from_pcap_reader(Cursor::new(buf));
        assert!(
            result.is_err(),
            "AC-002 / EC-005: EPB before any IDB (empty table) must return Err; got Ok"
        );
        let err_a = format!("{:#}", result.unwrap_err());
        assert!(
            err_a.contains(E_INP_009),
            "AC-002 / PC5a: empty-table path MUST return E-INP-009 (not E-INP-010); \
             got: {err_a}"
        );
        assert!(
            !err_a.contains(E_INP_010),
            "AC-002 discriminant: empty-table MUST NOT return E-INP-010; \
             E-INP-009 ≠ E-INP-010 (AC-001 / BC-2.01.012 PC5a); got: {err_a}"
        );
        // BC-2.01.012 PC5a: exact message fragment check.
        assert!(
            err_a.contains("interface table is empty"),
            "AC-002 / PC5a: error message must indicate empty interface table; got: {err_a}"
        );
    }

    // ── Sub-test B: OOB-on-non-empty → E-INP-010 ─────────────────────────────
    // pcapng: SHB + IDB(index=0, ETHERNET) + EPB(interface_id=5, which is OOB).
    // The interface table has 1 entry (index 0); interface_id=5 >= 1 → OOB on non-empty.
    {
        let mut buf = shb_only_le();
        buf.extend_from_slice(&idb_le_ethernet_default_tsresol()); // one IDB → table[0]
        // EPB with interface_id=5 (OOB: only index 0 exists).
        buf.extend_from_slice(&epb_le(5, 0, 1_000_000, 4, 4, &[0xAA, 0xBB, 0xCC, 0xDD]));

        let result = PcapSource::from_pcap_reader(Cursor::new(buf));
        assert!(
            result.is_err(),
            "AC-003 / EC-006: interface_id=5 with table_size=1 must return Err; got Ok"
        );
        let err_b = format!("{:#}", result.unwrap_err());
        assert!(
            err_b.contains(E_INP_010),
            "AC-003 / PC5b: OOB-on-non-empty MUST return E-INP-010 (not E-INP-009); \
             got: {err_b}"
        );
        assert!(
            !err_b.contains(E_INP_009),
            "AC-003 discriminant: OOB-on-non-empty MUST NOT return E-INP-009; \
             E-INP-009 ≠ E-INP-010 (AC-001 / BC-2.01.012 PC5b); got: {err_b}"
        );
        // BC-2.01.012 PC5b: exact message fragment check.
        assert!(
            err_b.contains("out of range"),
            "AC-003 / PC5b: error message must indicate 'out of range'; got: {err_b}"
        );
    }

    // ── EC-006 additional: interface_id=1 with one IDB also → E-INP-010 ──────
    {
        let mut buf = shb_only_le();
        buf.extend_from_slice(&idb_le_ethernet_default_tsresol()); // table[0]
        // EPB with interface_id=1 (OOB on a 1-entry table).
        buf.extend_from_slice(&epb_le(1, 0, 0, 4, 4, &[0xAA, 0xBB, 0xCC, 0xDD]));

        let result = PcapSource::from_pcap_reader(Cursor::new(buf));
        assert!(
            result.is_err(),
            "EC-006: interface_id=1 with table_size=1 must return Err"
        );
        let err_c = format!("{:#}", result.unwrap_err());
        assert!(
            err_c.contains(E_INP_010),
            "EC-006: interface_id=1, table_size=1 → E-INP-010; got: {err_c}"
        );
        assert!(
            err_c.contains("interface_id=1"),
            "EC-006: error must include interface_id=1 in message; got: {err_c}"
        );
    }

    // ── EC-007: interface_id=u32::MAX with non-empty table → E-INP-010 ────────
    {
        let mut buf = shb_only_le();
        buf.extend_from_slice(&idb_le_ethernet_default_tsresol()); // table[0]
        buf.extend_from_slice(&epb_le(u32::MAX, 0, 0, 4, 4, &[0xAA, 0xBB, 0xCC, 0xDD]));

        let result = PcapSource::from_pcap_reader(Cursor::new(buf));
        assert!(
            result.is_err(),
            "EC-007: interface_id=u32::MAX must return Err (OOB on non-empty table)"
        );
        let err_d = format!("{:#}", result.unwrap_err());
        assert!(
            err_d.contains(E_INP_010),
            "EC-007: interface_id=u32::MAX OOB → E-INP-010; got: {err_d}"
        );
    }
}

/// AC-004 / BC-2.01.012 PC6a (live reachable) + PC6b (defense-in-depth): guard-before-allocate.
///
/// PC6a: captured_len > body.len() → E-INP-008 (live reachable guard).
///   We craft an EPB whose captured_len field in the body is larger than the total
///   remaining body bytes after the fixed fields. This is the directly reachable path.
///
/// PC6b: 20 + captured_len + pad_len(captured_len) > body.len() → E-INP-008 (defense-in-depth).
///   As noted in BC-2.01.012 PC6b, this overrun condition is unreachable via normal
///   crate delivery (the crate rejects 4-misaligned blocks before wirerust sees them).
///   The note in the story prescribes: "if genuinely unreachable through from_pcap_reader,
///   test the bound logic as the story prescribes and note it."
///
///   Approach: We build an EPB where captured_len is exactly body_size - 20 (so PC6a passes)
///   but then add +1 to captured_len (so PC6a fires instead). Since PC6b is defense-in-depth
///   and unreachable via crate delivery, we test PC6a as the live guard and note PC6b's status.
///
/// Both paths must produce E-INP-008 (not E-INP-009 or E-INP-010).
///
/// This test is PARTIALLY RED: PC6a is already implemented (lines 861-867 of reader.rs),
/// but the check uses `available = body.len().saturating_sub(20)` and `captured_len > available`
/// which is equivalent to PC6a. PC6b (padding-aware overhead) is NOT coded. We test
/// the PC6a path (already implemented) and document PC6b as defense-in-depth.
#[test]
fn test_BC_2_01_012_guard_before_allocate() {
    // ── PC6a: captured_len > body.len() - 20 → E-INP-008 ────────────────────
    // Build an EPB where captured_len in the body claims more bytes than available.
    // Strategy: EPB with 4-byte data zone but captured_len claims 100 bytes.
    // body = 20 (fixed) + 4 (data) = 24 bytes; available = 4 bytes.
    // captured_len = 100 > 4 → PC6a fires → E-INP-008.
    {
        // Build a raw EPB that claims captured_len=100 but only has 4 data bytes.
        // We build this manually rather than through epb_le() because epb_le()
        // uses captured_len for actual data allocation.
        let data = [0xAA, 0xBB, 0xCC, 0xDD]; // 4 data bytes (4-aligned, 0 padding)
        let body_len = EPB_BODY_FIXED_BYTES + data.len(); // 20 + 4 = 24 (no pad: 4%4=0)
        let btl: u32 = (12 + body_len) as u32;
        let captured_len: u32 = 100; // LYING: claims 100 but only 4 bytes present

        let mut epb = Vec::new();
        epb.extend_from_slice(&EPB_BLOCK_TYPE.to_le_bytes());
        epb.extend_from_slice(&btl.to_le_bytes());
        epb.extend_from_slice(&0u32.to_le_bytes()); // interface_id = 0
        epb.extend_from_slice(&0u32.to_le_bytes()); // ts_high = 0
        epb.extend_from_slice(&0u32.to_le_bytes()); // ts_low = 0
        epb.extend_from_slice(&captured_len.to_le_bytes()); // claimed captured_len = 100
        epb.extend_from_slice(&100u32.to_le_bytes()); // original_len
        epb.extend_from_slice(&data); // only 4 bytes of data (not 100)
        // no padding bytes needed (4-aligned)
        epb.extend_from_slice(&btl.to_le_bytes()); // trailing btl

        let mut buf = shb_only_le();
        buf.extend_from_slice(&idb_le_ethernet_default_tsresol());
        buf.extend_from_slice(&epb);

        let result = PcapSource::from_pcap_reader(Cursor::new(buf));
        assert!(
            result.is_err(),
            "AC-004 / PC6a: captured_len=100 > available=4 must return Err; got Ok"
        );
        let err = format!("{:#}", result.unwrap_err());
        assert!(
            err.contains(E_INP_008),
            "AC-004 / PC6a: bound-by-body overflow MUST return E-INP-008 (wirerust body-decode); \
             got: {err}"
        );
        assert!(
            !err.contains(E_INP_009),
            "AC-004 / PC6a: MUST NOT be E-INP-009; got: {err}"
        );
        assert!(
            !err.contains(E_INP_010),
            "AC-004 / PC6a: MUST NOT be E-INP-010 (that is crate framing); got: {err}"
        );
    }

    // ── PC6b NOTE: defense-in-depth (unreachable via crate-framed 4-aligned blocks) ──
    //
    // BC-2.01.012 PC6b: the padding-aware check is required to be CODED even though
    // it cannot be triggered via normal crate block delivery. The crate rejects any
    // block_total_length that is not 4-byte aligned before returning the block to
    // wirerust. On a 4-aligned block, body.len() (= btl - 12) is always a multiple of 4,
    // so the maximum valid captured_len (= body.len() - 20) is also aligned, requiring
    // zero padding — the padded total never exceeds the data zone.
    //
    // To reach PC6b via from_pcap_reader, we would need a non-4-aligned block to bypass
    // the crate gate, which the crate's btl alignment check prevents (→ E-INP-010).
    //
    // The Kani VP-027 proof harness (in tests/kani_proofs.rs) exercises PC6b directly
    // by calling the EPB decode function with a synthetic non-aligned body bypassing the
    // crate gate. The implementation MUST code PC6b even though this integration test
    // cannot reach it via from_pcap_reader.
    //
    // Confirmation test: verify that a legitimately well-framed (4-aligned) EPB with
    // exactly the maximum-boundary captured_len succeeds (shows PC6b does not fire on
    // valid 4-aligned blocks).
    {
        // data_len = 8 (4-aligned); body = 20 + 8 = 28; btl = 40.
        // captured_len = 8 = body.len() - 20 = maximum valid.
        // 20 + 8 + pad(8) = 20 + 8 + 0 = 28 = body.len(). ✓ (PC6b passes exactly)
        let data = [0x11u8; 8];
        let buf = le_pcapng_with_one_epb(0, 1_000_000, 8, 8, &data);
        // This should succeed (but the timestamp will be wrong — ts uses DEFAULT_TSRESOL=6,
        // not pcapng_timestamp_to_secs_usecs). We only test that it does not fail with
        // E-INP-008 from PC6b.
        let result = PcapSource::from_pcap_reader(Cursor::new(buf));
        // The current EPB arm doesn't call pcapng_timestamp_to_secs_usecs (todo!() stub),
        // but does compute the timestamp inline — so this may PASS (no todo!() on this path).
        // If it panics from todo!(), that means pcapng_timestamp_to_secs_usecs was called.
        // We need to handle either case:
        // - If Ok: PC6b correctly did not fire on a valid 4-aligned block. Good.
        // - If Err: must NOT be E-INP-008 (that would mean PC6b or PC6a incorrectly fired).
        match result {
            Ok(source) => {
                assert_eq!(
                    source.packets.len(),
                    1,
                    "PC6b confirmation: valid EPB at exact boundary must yield 1 packet"
                );
            }
            Err(e) => {
                let err = format!("{:#}", e);
                assert!(
                    !err.contains(E_INP_008),
                    "PC6b confirmation: valid 4-aligned EPB must NOT return E-INP-008; \
                     got: {err}"
                );
            }
        }
    }
}

/// AC-005 / BC-2.01.012 PC3: packet data bounded by captured_len (NOT original_len).
///
/// EC-002: captured_len < original_len — packet is captured-length-truncated by the
/// writing tool. wirerust MUST copy exactly captured_len bytes. It MUST NOT compute
/// or apply snaplen (Decision 9 amendment).
///
/// canonical test vector from BC-2.01.012:
///   captured_len=64, original_len=1500 → data.len() == 64.
#[test]
fn test_BC_2_01_012_data_bounded_by_captured_len() {
    // 64 bytes of payload data (non-palindromic for detection)
    let payload: Vec<u8> = (0u8..64).collect();
    let buf = le_pcapng_with_one_epb(0, 1_000_000, 64, 1500, &payload);

    let result = PcapSource::from_pcap_reader(Cursor::new(buf));
    // This test may PASS (EPB arm computes timestamp inline, not via todo!() stub).
    // If it panics, pcapng_timestamp_to_secs_usecs was called.
    match result {
        Ok(source) => {
            assert_eq!(
                source.packets.len(),
                1,
                "AC-005: must yield exactly 1 packet"
            );
            assert_eq!(
                source.packets[0].data.len(),
                64,
                "AC-005 / PC3: data.len() MUST be captured_len (64), not original_len (1500); \
                 got {}",
                source.packets[0].data.len()
            );
            // Verify byte fidelity: data must be exactly the first 64 bytes of payload.
            assert_eq!(
                source.packets[0].data, payload,
                "AC-005 / PC3: data bytes must be byte-identical to the captured bytes"
            );
        }
        Err(e) => {
            // This happens if pcapng_timestamp_to_secs_usecs (todo!()) was called.
            // Until implementation is done, this is expected. Fail with helpful message.
            panic!(
                "AC-005: from_pcap_reader returned Err (may be todo!() panic in timestamp helper): \
                 {:#}",
                e
            );
        }
    }
}

/// AC-006 / BC-2.01.012 AC-005 / EC-008: zero-byte captured_len → data = vec![].
///
/// When captured_len = 0, wirerust MUST produce RawPacket { data: vec![] }.
/// Zero-byte packets are valid; `data` is empty, not absent.
/// The padding-aware check passes: 20 + 0 + 0 = 20 <= body.len() (≥ 20). ✓
#[test]
fn test_BC_2_01_012_zero_byte_captured_len() {
    // EPB: captured_len=0, original_len=0, no data bytes, no padding.
    // body = 20 (fixed fields only); btl = 32.
    let buf = le_pcapng_with_one_epb(0, 0, 0, 0, &[]);

    let result = PcapSource::from_pcap_reader(Cursor::new(buf));
    match result {
        Ok(source) => {
            assert_eq!(
                source.packets.len(),
                1,
                "AC-006: must yield exactly 1 packet"
            );
            assert_eq!(
                source.packets[0].data.len(),
                0,
                "AC-006 / EC-008: zero captured_len → data must be empty (data.len()=0); \
                 got {}",
                source.packets[0].data.len()
            );
            assert!(
                source.packets[0].data.is_empty(),
                "AC-006: data must be empty vec![]"
            );
        }
        Err(e) => {
            panic!(
                "AC-006: from_pcap_reader returned Err (may be todo!() panic): {:#}",
                e
            );
        }
    }
}

/// AC-007 / BC-2.01.012 AC-006 / EC-009 + EC-010: max-boundary captured_len fidelity.
///
/// EC-009: captured_len at exact padding-aware boundary → Ok(RawPacket).
/// EC-010: captured_len one byte over → E-INP-008 (padding overrun).
///
/// The exact boundary: 20 + captured_len + pad_len(captured_len) == body.len().
///
/// We use a 4-byte-aligned data_len for EC-009 to keep the boundary clean:
///   data_len=8 (4-aligned, pad=0): body=28, btl=40.
///   20 + 8 + 0 = 28 = body.len(). ✓ — valid.
///
/// For EC-010: captured_len = 9 with the same body (data zone = 8 bytes, pad_len(9)=3):
///   20 + 9 + 3 = 32 > 28 = body.len(). ✗ — padding overrun → E-INP-008.
///
/// Since PC6b (padding overrun) is defense-in-depth and unreachable via crate-framed
/// 4-aligned blocks (see AC-004 note), we test EC-010 by crafting an EPB where
/// captured_len exceeds available bytes (PC6a fires for captured_len > 8):
///   captured_len=9 > available=8 → PC6a fires → E-INP-008. ✓
///
/// This tests the live-reachable guard (PC6a) which subsumes EC-010 behavior for
/// crate-framed blocks.
#[test]
fn test_BC_2_01_012_max_boundary_captured_len() {
    // ── EC-009: exact boundary (captured_len = 8, data = 8 bytes, pad = 0) ────
    {
        let data = [0xAAu8; 8]; // 8 bytes, 4-aligned (pad = 0)
        let buf = le_pcapng_with_one_epb(0, 0, 8, 8, &data);
        let result = PcapSource::from_pcap_reader(Cursor::new(buf));
        match result {
            Ok(source) => {
                assert_eq!(source.packets.len(), 1, "EC-009: must yield 1 packet");
                assert_eq!(
                    source.packets[0].data.len(),
                    8,
                    "EC-009: data.len() must be captured_len=8; got {}",
                    source.packets[0].data.len()
                );
            }
            Err(e) => {
                panic!("EC-009: valid max-boundary EPB returned Err: {:#}", e);
            }
        }
    }

    // ── EC-010: one over boundary (captured_len=9 > available=8) → E-INP-008 ─
    // We build an EPB with 8 bytes of data but captured_len=9 (claims 9 of 8 available).
    // PC6a fires: captured_len(9) > available(8) → E-INP-008.
    {
        let data = [0xAAu8; 8]; // 8 bytes in body, but captured_len claims 9
        let pad_len = 0; // data.len()=8, 8%4=0
        let body_len = EPB_BODY_FIXED_BYTES + data.len() + pad_len; // 20 + 8 + 0 = 28
        let btl: u32 = (12 + body_len) as u32; // 40
        let captured_len: u32 = 9; // LYING: claims 9 but only 8 bytes present

        let mut epb = Vec::new();
        epb.extend_from_slice(&EPB_BLOCK_TYPE.to_le_bytes());
        epb.extend_from_slice(&btl.to_le_bytes());
        epb.extend_from_slice(&0u32.to_le_bytes()); // interface_id
        epb.extend_from_slice(&0u32.to_le_bytes()); // ts_high
        epb.extend_from_slice(&0u32.to_le_bytes()); // ts_low
        epb.extend_from_slice(&captured_len.to_le_bytes()); // 9 (LYING)
        epb.extend_from_slice(&9u32.to_le_bytes()); // original_len
        epb.extend_from_slice(&data); // 8 bytes (not 9)
        epb.extend_from_slice(&btl.to_le_bytes());

        let mut buf = shb_only_le();
        buf.extend_from_slice(&idb_le_ethernet_default_tsresol());
        buf.extend_from_slice(&epb);

        let result = PcapSource::from_pcap_reader(Cursor::new(buf));
        assert!(
            result.is_err(),
            "AC-007 / EC-010: captured_len=9 > available=8 must return Err"
        );
        let err = format!("{:#}", result.unwrap_err());
        assert!(
            err.contains(E_INP_008),
            "AC-007 / EC-010: MUST return E-INP-008 (wirerust body-decode — captured_len OOB); \
             got: {err}"
        );
        assert!(
            !err.contains(E_INP_010),
            "AC-007 / EC-010: MUST NOT return E-INP-010 (that is crate framing); got: {err}"
        );
    }
}

/// AC-008 / BC-2.01.012 AC-004: raw block path (not crate Duration).
///
/// The EPB parser MUST read (ts_high, ts_low) from RawBlock body bytes at
/// offsets 4-7 and 8-11 (relative to EPB body start). It MUST NOT use
/// EnhancedPacketBlock::timestamp (the crate's Duration type, which hard-codes
/// nanoseconds and discards if_tsresol).
///
/// This is verified indirectly: a pcapng with if_tsresol=6 (µs) and a known
/// ts value must produce the CORRECT timestamp (not 1000× off as the crate's
/// ns-hardcode would produce). After implementation, this test proves the
/// raw-path is used. Before implementation, it may fail due to todo!() or
/// the wrong timestamp.
///
/// We use a minimal in-line test here (full regression in test_BC_2_01_014_regression_1000x_bug).
#[test]
fn test_BC_2_01_012_raw_block_path_not_crate_duration() {
    // pcapng with if_tsresol absent (default=6, µs) and ts=1_000_000 µs ticks.
    // Correct: ts_sec=1, ts_usecs=0.
    // Wrong (crate ns-hardcode): ts_sec=0, ts_usecs=1000 (1_000_000 ns = 1 ms).
    // The crate's Duration::from_nanos(1_000_000) → 1 ms, ts_sec=0, ts_usecs=1000.
    let buf = le_pcapng_with_one_epb(0, 1_000_000, 4, 4, &[0xAA, 0xBB, 0xCC, 0xDD]);
    let result = PcapSource::from_pcap_reader(Cursor::new(buf));
    match result {
        Ok(source) => {
            assert_eq!(source.packets.len(), 1, "AC-008: must yield 1 packet");
            assert_ne!(
                source.packets[0].timestamp_secs, 0,
                "AC-008: ts_sec=0 would indicate wrong crate-Duration path (which produces \
                 1_000_000 ns = 1 ms ← not 1 sec); correct raw path → ts_sec=1"
            );
            // The raw path with if_tsresol=6: ticks=1_000_000, ts_sec=1, ts_usecs=0.
            assert_eq!(
                source.packets[0].timestamp_secs, 1,
                "AC-008: raw block path: 1_000_000 µs ticks → ts_sec=1 (not crate-Duration \
                 nanosecond path); got {}",
                source.packets[0].timestamp_secs
            );
        }
        Err(e) => {
            panic!(
                "AC-008: from_pcap_reader returned Err (may be todo!() panic): {:#}",
                e
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// F-3 END-TO-END TIMESTAMP TESTS (via from_pcap_reader, with non-default tsresol)
// These exercise the complete path: IDB with explicit if_tsresol + EPB timestamp decode.
// ALL FAIL until pcapng_timestamp_to_secs_usecs is implemented AND the EPB arm
// calls it with interfaces[interface_id].if_tsresol (not DEFAULT_TSRESOL=6).
// ─────────────────────────────────────────────────────────────────────────────

/// AC-009 / BC-2.01.014 Invariant 2 / VP proof target: 1000×-bug regression guard.
///
/// This is the F-3 correctness test: a pcapng file with if_tsresol=9 (nanoseconds)
/// MUST produce the CORRECT timestamp, NOT the 1000×-too-large value that the
/// current hardcoded-DEFAULT_TSRESOL=6 implementation produces.
///
/// # Current wrong behavior (before implementation):
///   The EPB arm hard-codes DEFAULT_TSRESOL=6 for all interfaces.
///   For ts_low=1_500_000_000 (1.5 billion ns ticks):
///     ticks = 1_500_000_000
///     ticks_per_sec = 10^6 = 1_000_000 (WRONG — treating ns ticks as µs)
///     ts_sec = 1_500_000_000 / 1_000_000 = 1500  ← 1000× too large!
///     ts_usecs = 0
///   Packets[0].timestamp_secs = 1500  ← WRONG
///
/// # Correct behavior (after implementation with per-interface if_tsresol):
///   The EPB arm must call pcapng_timestamp_to_secs_usecs(ts_high, ts_low, 9).
///     ticks_per_sec = 1_000_000_000 (nanoseconds)
///     ts_sec = 1_500_000_000 / 1_000_000_000 = 1
///     ts_usecs = 500_000 (500 ms in µs)
///   Packets[0].timestamp_secs = 1  ← CORRECT
///
/// FAILS: The EPB arm does NOT call pcapng_timestamp_to_secs_usecs and does NOT
/// look up interfaces[interface_id].if_tsresol. Current result: ts_sec=1500 (wrong).
#[test]
fn test_BC_2_01_014_regression_1000x_bug() {
    // pcapng: SHB + IDB(if_tsresol=9, nanoseconds) + EPB(ts=1_500_000_000 ns ticks)
    let buf = le_pcapng_with_tsresol_and_one_epb(
        9,             // if_tsresol=9 (nanoseconds)
        0,             // ts_high=0
        1_500_000_000, // ts_low=1.5 billion ns ticks
        4,             // captured_len=4
        4,             // original_len=4
        &[0xAA, 0xBB, 0xCC, 0xDD],
    );

    let result = PcapSource::from_pcap_reader(Cursor::new(buf));
    assert!(
        result.is_ok(),
        "AC-009 regression: pcapng with if_tsresol=9 must parse without error; got: {:?}",
        result.err()
    );
    let source = result.unwrap();
    assert_eq!(
        source.packets.len(),
        1,
        "AC-009 regression: must yield 1 packet"
    );

    let ts_sec = source.packets[0].timestamp_secs;
    let ts_usecs = source.packets[0].timestamp_usecs;

    // CURRENT WRONG VALUE (hardcoded DEFAULT_TSRESOL=6):
    //   ts_sec = 1_500_000_000 / 1_000_000 = 1500 ← 1000× too large
    // CORRECT VALUE (if_tsresol=9 → nanoseconds):
    //   ts_sec = 1, ts_usecs = 500_000

    assert_ne!(
        ts_sec, 1500,
        "AC-009 F-3 1000× BUG DETECTED: ts_sec=1500 means if_tsresol=9 was treated as \
         µs (hardcoded DEFAULT_TSRESOL=6). The EPB arm MUST call \
         pcapng_timestamp_to_secs_usecs with interfaces[interface_id].if_tsresol=9. \
         Current wrong ts_sec=1500, expected correct ts_sec=1."
    );
    assert_eq!(
        ts_sec, 1,
        "AC-009 F-3 regression guard: 1.5 billion ns ticks with if_tsresol=9 → ts_sec MUST be 1 \
         (not 1500 which is 1000× too large from hardcoded µs treatment); got {ts_sec}"
    );
    assert_eq!(
        ts_usecs, 500_000,
        "AC-009 F-3 regression guard: 1_500_000_000 ns = 1 sec + 500_000 µs; \
         ts_usecs MUST be 500_000; got {ts_usecs}"
    );
}

/// End-to-end: pcapng with IDB if_tsresol=6 (µs default) + EPB uses that interface's tsresol.
///
/// Verifies that the EPB arm correctly looks up interfaces[interface_id].if_tsresol
/// even when the value equals DEFAULT_TSRESOL (6). This is the happy path.
///
/// FAILS if pcapng_timestamp_to_secs_usecs is a todo!() stub.
/// PASSES after implementation with correct per-interface lookup.
#[test]
fn test_BC_2_01_014_e2e_le_microsecond_correct_timestamp() {
    // pcapng: SHB + IDB(if_tsresol=6, explicit) + EPB(ts_low=1_000_000)
    let buf = le_pcapng_with_tsresol_and_one_epb(
        6,         // if_tsresol=6 (µs, explicit — not absent-defaulted)
        0,         // ts_high=0
        1_000_000, // ts_low=1_000_000 µs ticks → 1 second
        4,         // captured_len
        4,         // original_len
        &[0x01, 0x02, 0x03, 0x04],
    );

    let result = PcapSource::from_pcap_reader(Cursor::new(buf));
    assert!(
        result.is_ok(),
        "E2E LE µs: must parse without error; got: {:?}",
        result.err()
    );
    let source = result.unwrap();
    assert_eq!(source.packets.len(), 1, "E2E LE µs: must yield 1 packet");
    assert_eq!(
        source.packets[0].timestamp_secs, 1,
        "E2E LE µs: 1_000_000 µs ticks → ts_sec=1; got {}",
        source.packets[0].timestamp_secs
    );
    assert_eq!(
        source.packets[0].timestamp_usecs, 0,
        "E2E LE µs: 1_000_000 µs ticks → ts_usecs=0; got {}",
        source.packets[0].timestamp_usecs
    );
}

/// End-to-end: genuine BE pcapng with EPB — endianness coverage (BC-2.01.010 Inv4).
///
/// All multi-byte fields are genuine BE (non-palindromic). An LE-always reader
/// would misread interface_id and timestamps.
///
/// ts_high=1 (BE: 00 00 00 01), ts_low=0 (BE: 00 00 00 00), if_tsresol=6 (default).
/// ticks = (1u64 << 32) | 0 = 4_294_967_296
/// ts_sec = 4_294_967_296 / 1_000_000 = 4294 (with saturation guard)
/// ts_usecs = 4_294_967_296 % 1_000_000 = 967_296
///
/// A LE-always reader would decode ts_high=0x01000000 (big BE value misread as LE).
/// ticks = (0x01000000u64 << 32) = a much larger value → different ts_sec.
///
/// FAILS if pcapng_timestamp_to_secs_usecs is a todo!() stub.
#[test]
fn test_BC_2_01_012_endianness_be_interface_id_and_timestamp() {
    // Genuine BE pcapng: ts_high=1, ts_low=0, captured_len=4.
    // All framing fields and EPB body fields encoded big-endian.
    let buf = be_pcapng_with_one_epb(1, 0, 4, 4, &[0xBE, 0xEF, 0xCA, 0xFE]);

    let result = PcapSource::from_pcap_reader(Cursor::new(buf));
    assert!(
        result.is_ok(),
        "BE endianness: genuine BE pcapng with 1 EPB must parse OK; got: {:?}",
        result.err()
    );
    let source = result.unwrap();
    assert_eq!(
        source.packets.len(),
        1,
        "BE endianness: must yield 1 packet"
    );

    // Verify data byte-fidelity (BE bodies decoded correctly).
    assert_eq!(
        source.packets[0].data,
        &[0xBE, 0xEF, 0xCA, 0xFE],
        "BE endianness: packet data must be byte-identical to the 4-byte BE payload"
    );

    // Verify timestamp: ts_high=1 (BE), ts_low=0 → ticks = 4_294_967_296 µs ticks.
    // ts_sec = 4_294_967_296 / 1_000_000 = 4294 (truncated integer division).
    // A LE-always misread of ts_high=1 (BE 00 00 00 01) gives ts_high=0x01000000 → much larger ts.
    let ts_sec = source.packets[0].timestamp_secs;
    assert_eq!(
        ts_sec, 4294,
        "BE endianness: ts_high=1 (BE), ts_low=0, if_tsresol=6 → ts_sec=4294; \
         a LE-always misread of ts_high would give a radically different value; got {ts_sec}"
    );
}

/// AC-013 / BC-2.01.012 PC8: N-packet encounter order and byte fidelity.
///
/// Tests a 16-packet synthetic pcapng fixture (arp-baseline-16pkt.cap equivalent):
///   - packets.len() == 16
///   - Packets appear in EPB encounter order (EPB[0] → packets[0], EPB[15] → packets[15]).
///   - Each packets[i].data is byte-for-byte identical to the EPB's captured bytes.
///
/// The fixture is the same synthetic 16-EPB pcapng used by test_BC_2_01_009_arp_baseline_cap_accepted
/// in bc_2_01_story123_pcapng_tests.rs (built inline here for independence).
///
/// This test may FAIL if pcapng_timestamp_to_secs_usecs (todo!()) is called during EPB parse.
/// After implementation: PASSES (should be GREEN).
#[test]
fn test_BC_2_01_012_happy_path_n_packet_order_and_byte_fidelity() {
    // Build a 16-EPB pcapng fixture inline.
    // Each EPB has a 4-byte payload: [packet_index, 0xBB, 0xCC, 0xDD].
    // ts_low = packet_index (so ts values are strictly increasing for order verification).
    let n: u32 = 16;
    let mut buf = Vec::new();

    // SHB (28 bytes LE)
    buf.extend_from_slice(&SHB_BLOCK_TYPE.to_le_bytes());
    buf.extend_from_slice(&28u32.to_le_bytes());
    buf.extend_from_slice(&SHB_BOM_LE);
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&0u16.to_le_bytes());
    buf.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes());
    buf.extend_from_slice(&28u32.to_le_bytes());

    // IDB (20 bytes LE, ETHERNET, no if_tsresol TLV → default 6)
    buf.extend_from_slice(&IDB_BLOCK_TYPE.to_le_bytes());
    buf.extend_from_slice(&20u32.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes()); // linktype = ETHERNET
    buf.extend_from_slice(&0u16.to_le_bytes()); // reserved
    buf.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
    buf.extend_from_slice(&20u32.to_le_bytes());

    // 16 EPBs — each with a 4-byte payload encoding the packet index
    let epb_btl: u32 = 36; // 12 outer + 20 EPB fixed + 4 data + 0 pad
    let mut expected_payloads: Vec<[u8; 4]> = Vec::new();
    for i in 0..n {
        let payload = [i as u8, 0xBB, 0xCC, 0xDD];
        expected_payloads.push(payload);
        buf.extend_from_slice(&EPB_BLOCK_TYPE.to_le_bytes());
        buf.extend_from_slice(&epb_btl.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes()); // interface_id = 0
        buf.extend_from_slice(&0u32.to_le_bytes()); // ts_high = 0
        buf.extend_from_slice(&i.to_le_bytes()); // ts_low = i (increasing)
        buf.extend_from_slice(&4u32.to_le_bytes()); // captured_len = 4
        buf.extend_from_slice(&4u32.to_le_bytes()); // original_len = 4
        buf.extend_from_slice(&payload); // packet_data (4 bytes, 4%4=0 pad)
        buf.extend_from_slice(&epb_btl.to_le_bytes()); // trailing btl
    }

    let result = PcapSource::from_pcap_reader(Cursor::new(buf));
    assert!(
        result.is_ok(),
        "AC-013: 16-EPB pcapng must parse without error; got: {:?}",
        result.err()
    );
    let source = result.unwrap();

    // PC8: exactly 16 packets
    assert_eq!(
        source.packets.len(),
        16,
        "AC-013 / PC8: packets.len() MUST be 16; got {}",
        source.packets.len()
    );

    // PC8: encounter order + byte fidelity
    for i in 0..16usize {
        // Encounter order (PC8 / Invariant 1)
        // Each EPB[i] has ts_low=i; the packet at position i must have ts from EPB[i].
        // (We don't assert exact ts values here since those depend on implementation;
        //  the data byte at position 0 encodes the order index.)
        assert_eq!(
            source.packets[i].data[0], i as u8,
            "AC-013 / PC8 encounter order: packets[{i}].data[0] must be {i} \
             (first byte encodes the EPB index); got {} — packets are out of order or \
             byte-fidelity is broken",
            source.packets[i].data[0]
        );

        // Byte fidelity (PC8): data must be byte-identical to the EPB payload
        assert_eq!(
            source.packets[i].data.as_slice(),
            &expected_payloads[i],
            "AC-013 / PC8 byte fidelity: packets[{i}].data must be byte-identical \
             to the EPB payload; got {:?}",
            source.packets[i].data
        );
    }
}
