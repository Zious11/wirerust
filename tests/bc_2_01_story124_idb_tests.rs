//! STORY-124: IDB Parse (Link Type + if_tsresol), Interface Whitelist,
//!            and Multi-IDB Agreement
//!
//! TDD Red Gate suite — ALL tests in this file target UNIMPLEMENTED behavior:
//!   - `parse_idb_options` is `todo!()` → all options-walk tests FAIL at runtime.
//!   - Reserved-field check (bytes 2-3) is NOT in the current IDB arm → FAIL.
//!   - E-INP-013 position check (IDB after first packet) is NOT implemented → FAIL.
//!   - The E-INP-011 conflict message format differs from BC-2.01.018's required
//!     format (interface 0/N explicit), → format-sensitive assertions FAIL.
//!   - The three-level precedence ordering (E-INP-013 → E-INP-001 → E-INP-011) is
//!     NOT enforced (no packets_emitted counter in the IDB arm) → FAIL.
//!   - `InterfaceInfo` and `Vec<InterfaceInfo>` table are declared but NOT populated
//!     or returned; tests asserting `InterfaceInfo` population FAIL.
//!   - VP-030 proptest fails because the E-INP-011 message format is wrong.
//!
//! STORY-123's existing tests in `bc_2_01_story123_pcapng_tests.rs` must remain
//! GREEN. Do NOT modify src/reader.rs or any other source file.
//!
//! Coverage map (AC → test → E-INP code):
//!   AC-001 → test_BC_2_01_011_linktype_ethernet (E-INP-001 path absent)
//!          → test_BC_2_01_011_interface_table_is_vec_indexed
//!          → test_BC_2_01_011_linktype_decoded_big_endian (non-palindromic)
//!   AC-002 → test_BC_2_01_011_if_tsresol_stored_in_interface_info
//!          → test_BC_2_01_011_if_tsresol_absent_default
//!   AC-003 → test_BC_2_01_011_body_truncated_e_inp_008 (E-INP-008)
//!          → test_BC_2_01_011_nonzero_reserved_e_inp_008 (E-INP-008)
//!   AC-004 → test_BC_2_01_011_options_malformed_length_e_inp_008 (E-INP-008)
//!          → test_BC_2_01_011_if_tsresol_wrong_length_e_inp_008 (E-INP-008)
//!   AC-005 → test_BC_2_01_011_late_idb_after_packet_rejected_e_inp_013 (E-INP-013)
//!   AC-006 → test_BC_2_01_011_idb_precedence_e_inp_013_wins_over_conflict (E-INP-013)
//!   AC-007 → test_BC_2_01_016_whitelist_mirrors_bc_2_01_001 (E-INP-001)
//!          → test_BC_2_01_016_non_whitelisted_linktype_returns_err_no_panic (E-INP-001)
//!   AC-008 → test_BC_2_01_018_two_idbs_different_linktype_e_inp_011 (E-INP-011)
//!          → test_BC_2_01_018_three_idbs_third_conflicts (E-INP-011)
//!   AC-009 → test_BC_2_01_018_two_idbs_same_linktype_ok
//!          → test_BC_2_01_011_two_idbs_same_linktype
//!   AC-010 → test_BC_2_01_011_no_panic_fuzz
//!   AC-011 → test_VP_030_multi_idb_agreement_totality (VP-030)
//!
//! SCOPE EXCLUSION (confirmed): this file does NOT test timestamp CONVERSION
//! (ts_sec / ts_usecs derivation from if_tsresol). That is BC-2.01.014 / STORY-125.
//! All if_tsresol tests only verify extraction and storage of the raw byte.
//!
//! Naming convention: `test_BC_S_SS_NNN_<assertion>()` throughout.
//! `#![allow(non_snake_case)]` required per factory BC-naming mandate.
#![allow(non_snake_case)]

use std::io::Cursor;

use proptest::prelude::*;
use wirerust::reader::{InterfaceInfo, PcapSource, SectionEndianness, parse_idb_options};

// ── pcapng canonical constants (mirrors ADR-009 / bc_2_01_story123_pcapng_tests) ─
//
// All byte values are taken from ADR-009 Current Canonical Constants table.
// Do NOT introduce magic byte values not listed there.

/// pcapng SHB block_type / file magic (endian-independent; canonical reference only).
#[allow(dead_code)]
const PCAPNG_MAGIC: [u8; 4] = [0x0A, 0x0D, 0x0D, 0x0A];

/// BOM: big-endian pcapng section (on-disk 1A 2B 3C 4D).
const SHB_BOM_BE: [u8; 4] = [0x1A, 0x2B, 0x3C, 0x4D];

/// BOM: little-endian pcapng section (on-disk 4D 3C 2B 1A).
const SHB_BOM_LE: [u8; 4] = [0x4D, 0x3C, 0x2B, 0x1A];

/// SHB block type (u32).
const SHB_BLOCK_TYPE: u32 = 0x0A0D_0D0A;

/// IDB block type (u32).
const IDB_BLOCK_TYPE: u32 = 0x0000_0001;

/// EPB block type (u32).
const EPB_BLOCK_TYPE: u32 = 0x0000_0006;

/// DataLink::ETHERNET numeric code.
const DL_ETHERNET: u16 = 1;

/// DataLink::LINUX_SLL numeric code.
const DL_LINUX_SLL: u16 = 113;

/// DataLink::RAW numeric code.
const DL_RAW: u16 = 101;

/// DataLink::IPV4 numeric code.
const DL_IPV4: u16 = 228;

/// DataLink::IPV6 numeric code.
const DL_IPV6: u16 = 229;

/// DataLink::IEEE802_11 — NOT whitelisted, fires E-INP-001.
const DL_IEEE802_11: u16 = 105;

/// pcapng if_tsresol option code.
const OPT_IF_TSRESOL: u16 = 9;

/// pcapng opt_endofopt code.
const OPT_ENDOFOPT: u16 = 0;

/// Default if_tsresol exponent when absent (ADR-009 canonical constant).
const DEFAULT_TSRESOL: u8 = 6;

// ── Fixture builder helpers ───────────────────────────────────────────────────
//
// These helpers follow the conventions established in bc_2_01_story123_pcapng_tests.rs.
// For genuine big-endian fixtures ALL multi-byte framing fields (block_type,
// block_total_length, trailing btl) PLUS all body fields are encoded big-endian,
// mirroring the approach documented for `genuine_be_pcapng_with_one_packet()`.

/// Build a minimal LE SHB block (28 bytes).
///
/// block_type + btl(28 LE) + BOM_LE + major(1 LE) + minor(0 LE)
///   + section_length(unspecified LE) + trailing btl(28 LE)
fn le_shb() -> Vec<u8> {
    let mut v = Vec::with_capacity(28);
    v.extend_from_slice(&SHB_BLOCK_TYPE.to_le_bytes()); // 0A 0D 0D 0A
    v.extend_from_slice(&28u32.to_le_bytes()); // btl = 28
    v.extend_from_slice(&SHB_BOM_LE); // 4D 3C 2B 1A
    v.extend_from_slice(&1u16.to_le_bytes()); // major = 1
    v.extend_from_slice(&0u16.to_le_bytes()); // minor = 0
    v.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes()); // section_length
    v.extend_from_slice(&28u32.to_le_bytes()); // trailing btl
    assert_eq!(v.len(), 28);
    v
}

/// Build a minimal BE SHB block (28 bytes, ALL fields big-endian).
///
/// Uses non-palindromic minor_version=2 so that any LE-misread of the body
/// decodes minor as 512 (0x0200), not 2 — detecting endianness bugs.
fn be_shb() -> Vec<u8> {
    let mut v = Vec::with_capacity(28);
    v.extend_from_slice(&SHB_BLOCK_TYPE.to_be_bytes()); // 0A 0D 0D 0A
    v.extend_from_slice(&28u32.to_be_bytes()); // 00 00 00 1C
    v.extend_from_slice(&SHB_BOM_BE); // 1A 2B 3C 4D
    v.extend_from_slice(&1u16.to_be_bytes()); // 00 01 (major=1)
    v.extend_from_slice(&2u16.to_be_bytes()); // 00 02 (minor=2; non-palindromic)
    v.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_be_bytes());
    v.extend_from_slice(&28u32.to_be_bytes()); // 00 00 00 1C
    assert_eq!(v.len(), 28);
    v
}

/// Build a minimal LE IDB block with the given linktype, reserved, and an optional
/// TLV options region.
///
/// Block structure (12-byte outer + 8-byte fixed + options):
///   block_type:          4 bytes = 0x00000001 (LE)
///   block_total_length:  4 bytes = btl (LE)
///   linktype:            2 bytes (LE)
///   reserved:            2 bytes (LE)
///   snaplen:             4 bytes = 65535 (LE)
///   [options]:           variable (the `opts` slice)
///   trailing btl:        4 bytes = btl (LE)
///
/// `btl = 12 (outer) + 8 (fixed) + options.len()`.
/// Returns the complete block bytes.
fn le_idb(linktype: u16, reserved: u16, opts: &[u8]) -> Vec<u8> {
    let body_len = 8 + opts.len();
    let btl = 12 + body_len;
    let mut v = Vec::with_capacity(btl);
    v.extend_from_slice(&IDB_BLOCK_TYPE.to_le_bytes());
    v.extend_from_slice(&(btl as u32).to_le_bytes()); // btl LE
    v.extend_from_slice(&linktype.to_le_bytes()); // linktype LE
    v.extend_from_slice(&reserved.to_le_bytes()); // reserved LE
    v.extend_from_slice(&65535u32.to_le_bytes()); // snaplen LE (discarded by wirerust)
    v.extend_from_slice(opts); // TLV options
    v.extend_from_slice(&(btl as u32).to_le_bytes()); // trailing btl LE
    assert_eq!(v.len(), btl);
    v
}

/// Build a minimal BE IDB block with the given linktype, reserved, and options.
///
/// All framing and body multi-byte fields are big-endian.
/// Non-palindromic linktype values (e.g., ETHERNET=1, rendered as 00 01)
/// produce LE-misread 0x0100 = 256 — detecting endianness bugs.
fn be_idb(linktype: u16, reserved: u16, opts: &[u8]) -> Vec<u8> {
    let body_len = 8 + opts.len();
    let btl = 12 + body_len;
    let mut v = Vec::with_capacity(btl);
    v.extend_from_slice(&IDB_BLOCK_TYPE.to_be_bytes()); // 00 00 00 01
    v.extend_from_slice(&(btl as u32).to_be_bytes()); // btl BE
    v.extend_from_slice(&linktype.to_be_bytes()); // linktype BE (non-palindromic)
    v.extend_from_slice(&reserved.to_be_bytes()); // reserved BE
    v.extend_from_slice(&65536u32.to_be_bytes()); // snaplen BE (non-palindromic, discarded)
    v.extend_from_slice(opts);
    v.extend_from_slice(&(btl as u32).to_be_bytes()); // trailing btl BE
    assert_eq!(v.len(), btl);
    v
}

/// Build a minimal LE EPB block (36 bytes) with a 4-byte dummy payload.
///
/// Layout: outer(12) + interface_id(4) + ts_high(4) + ts_low(4) +
///         captured_len(4) + original_len(4) + packet_data(4) = 36 bytes.
fn le_epb(interface_id: u32) -> Vec<u8> {
    let btl: u32 = 36;
    let mut v = Vec::with_capacity(36);
    v.extend_from_slice(&EPB_BLOCK_TYPE.to_le_bytes());
    v.extend_from_slice(&btl.to_le_bytes()); // btl = 36
    v.extend_from_slice(&interface_id.to_le_bytes()); // interface_id
    v.extend_from_slice(&1u32.to_le_bytes()); // ts_high = 1
    v.extend_from_slice(&0u32.to_le_bytes()); // ts_low = 0
    v.extend_from_slice(&4u32.to_le_bytes()); // captured_len = 4
    v.extend_from_slice(&4u32.to_le_bytes()); // original_len = 4
    v.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // 4-byte payload (4%4=0, no pad)
    v.extend_from_slice(&btl.to_le_bytes()); // trailing btl
    assert_eq!(v.len(), 36);
    v
}

/// Build a TLV option for if_tsresol (code 9, length 1) with the given value byte.
///
/// TLV: option_code(2 LE) + option_length(2 LE) + value(1) + padding(3 to align to 4).
fn opt_if_tsresol_le(value: u8) -> Vec<u8> {
    // code=9, length=1, value=value, pad=3 bytes to align to 4
    let mut v = Vec::with_capacity(8);
    v.extend_from_slice(&OPT_IF_TSRESOL.to_le_bytes()); // 09 00
    v.extend_from_slice(&1u16.to_le_bytes()); // length = 1
    v.push(value); // the exponent byte
    v.extend_from_slice(&[0u8; 3]); // 3 pad bytes (1 + 3 = 4, aligned)
    v
}

/// Build a malformed if_tsresol TLV with option_length = 4 (not 1).
///
/// AC-004 / EC-012: if_tsresol with option_length != 1 → E-INP-008 (F-M5).
fn opt_if_tsresol_wrong_length_le() -> Vec<u8> {
    // code=9, length=4 (wrong — must be 1), 4 value bytes, 0 pad (4 is already aligned)
    let mut v = Vec::with_capacity(8);
    v.extend_from_slice(&OPT_IF_TSRESOL.to_le_bytes()); // 09 00
    v.extend_from_slice(&4u16.to_le_bytes()); // WRONG length = 4
    v.extend_from_slice(&[0x00, 0x00, 0x00, 0x06]); // 4 value bytes
    v
}

/// Build an opt_endofopt terminator.
fn opt_endofopt_le() -> Vec<u8> {
    let mut v = Vec::with_capacity(4);
    v.extend_from_slice(&OPT_ENDOFOPT.to_le_bytes()); // 00 00
    v.extend_from_slice(&0u16.to_le_bytes()); // length = 0
    v
}

/// Build a malformed TLV option where the declared length overruns the body.
///
/// AC-004 / EC-011: option_length > remaining bytes → E-INP-008.
/// Creates a single TLV with length=100 but only 2 bytes of actual data.
fn opt_malformed_overrun_le() -> Vec<u8> {
    // code=42 (unknown, would be skipped except length overruns body),
    // length=100 (way past end of any reasonable IDB options region)
    let mut v = Vec::with_capacity(4);
    v.extend_from_slice(&42u16.to_le_bytes()); // code = 42 (unknown)
    v.extend_from_slice(&100u16.to_le_bytes()); // length = 100 (overruns body)
    // no value bytes — simulates a truncated option (body ends here)
    v
}

// ─── Pure-core helper tests: parse_idb_options ───────────────────────────────
//
// These tests exercise `parse_idb_options` as a pure function.
// The stub is `todo!()`, so ALL tests below FAIL with PanicInfo at runtime.

/// AC-002 / BC-2.01.011 PC2 / PC6 — if_tsresol present with code 9, length 1.
///
/// Canonical test vector: body = 8 fixed bytes + [09 00 01 00 06 00 00 00] + endofopt.
/// Expected: Ok(0x06) (base-10 nanosecond exponent = 6 is the example here; other
/// values tested in EC-002/EC-003 cases below).
///
/// AC mapping: AC-002 (if_tsresol stored from option TLV).
/// E-INP code pinned: none (happy path — successful extraction).
#[test]
fn test_BC_2_01_011_if_tsresol_stored_in_interface_info() {
    // Build a complete IDB body (8 fixed + if_tsresol TLV with value=9 + endofopt).
    // We test the pure helper directly; no full pcapng stream needed.
    // Body layout: [linktype:2 | reserved:2 | snaplen:4 | TLV...]
    let mut body = Vec::new();
    body.extend_from_slice(&DL_ETHERNET.to_le_bytes()); // linktype = 1 (LE)
    body.extend_from_slice(&0u16.to_le_bytes()); // reserved = 0
    body.extend_from_slice(&65535u32.to_le_bytes()); // snaplen (discarded)
    // if_tsresol TLV: code=9, length=1, value=0x09 (nanosecond), pad=3
    body.extend_from_slice(&opt_if_tsresol_le(0x09));
    // opt_endofopt terminator
    body.extend_from_slice(&opt_endofopt_le());

    let result = parse_idb_options(&body, SectionEndianness::LittleEndian);
    assert!(
        result.is_ok(),
        "parse_idb_options with if_tsresol=9 must return Ok; got: {:?}",
        result.unwrap_err()
    );
    assert_eq!(
        result.unwrap(),
        0x09,
        "if_tsresol option value 0x09 must be extracted verbatim"
    );
}

/// AC-002 / BC-2.01.011 EC-001 — if_tsresol absent from options → DEFAULT_TSRESOL.
///
/// AC mapping: AC-002 (absent if_tsresol defaults to 6).
/// E-INP code pinned: none (happy path).
#[test]
fn test_BC_2_01_011_if_tsresol_absent_default() {
    // Body with 8 fixed bytes only (no options region, no if_tsresol).
    let mut body = Vec::new();
    body.extend_from_slice(&DL_ETHERNET.to_le_bytes()); // linktype
    body.extend_from_slice(&0u16.to_le_bytes()); // reserved
    body.extend_from_slice(&65535u32.to_le_bytes()); // snaplen

    let result = parse_idb_options(&body, SectionEndianness::LittleEndian);
    assert!(
        result.is_ok(),
        "parse_idb_options with no options must return Ok; got: {:?}",
        result.unwrap_err()
    );
    assert_eq!(
        result.unwrap(),
        DEFAULT_TSRESOL,
        "absent if_tsresol must default to {} (10^-6 microseconds, pcapng spec default)",
        DEFAULT_TSRESOL
    );
}

/// AC-004 / BC-2.01.011 AC-005 / EC-012 — if_tsresol option with option_length = 4.
///
/// F-M5: if_tsresol (code 9) with option_length != 1 MUST return Err(E-INP-008).
/// MUST NOT silently default to 6 (ADR-009 rev 9).
///
/// AC mapping: AC-004 (if_tsresol wrong length).
/// E-INP code pinned: E-INP-008 (must contain "E-INP-008").
/// Sibling codes NOT present: E-INP-010, E-INP-011, E-INP-013.
#[test]
fn test_BC_2_01_011_if_tsresol_wrong_length_e_inp_008() {
    let mut body = Vec::new();
    body.extend_from_slice(&DL_ETHERNET.to_le_bytes());
    body.extend_from_slice(&0u16.to_le_bytes());
    body.extend_from_slice(&65535u32.to_le_bytes());
    body.extend_from_slice(&opt_if_tsresol_wrong_length_le());

    let result = parse_idb_options(&body, SectionEndianness::LittleEndian);
    assert!(
        result.is_err(),
        "parse_idb_options with if_tsresol option_length=4 MUST return Err (F-M5/E-INP-008)"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    // Discriminating assertion: must contain E-INP-008, must NOT contain sibling codes.
    assert!(
        err_msg.contains("E-INP-008"),
        "error for if_tsresol wrong length must contain E-INP-008; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-010"),
        "error must NOT contain E-INP-010 (wrong taxonomy); got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-011"),
        "error must NOT contain E-INP-011; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-013"),
        "error must NOT contain E-INP-013; got: {err_msg}"
    );
}

/// AC-004 / BC-2.01.011 AC-005 / EC-011 — option TLV length exceeds remaining bytes.
///
/// BC-2.01.011 PC6: bounds-check option-length BEFORE reading value or padding.
/// Malformed TLV with length=100 but only 4 bytes available → Err(E-INP-008).
///
/// AC mapping: AC-004 (options TLV overrun).
/// E-INP code pinned: E-INP-008.
/// Sibling codes NOT present: E-INP-010, E-INP-011, E-INP-013.
#[test]
fn test_BC_2_01_011_options_malformed_length_e_inp_008() {
    let mut body = Vec::new();
    body.extend_from_slice(&DL_ETHERNET.to_le_bytes());
    body.extend_from_slice(&0u16.to_le_bytes());
    body.extend_from_slice(&65535u32.to_le_bytes());
    body.extend_from_slice(&opt_malformed_overrun_le());

    let result = parse_idb_options(&body, SectionEndianness::LittleEndian);
    assert!(
        result.is_err(),
        "parse_idb_options with overrunning option_length MUST return Err (E-INP-008)"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    assert!(
        err_msg.contains("E-INP-008"),
        "error for TLV overrun must contain E-INP-008; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-010"),
        "must NOT contain E-INP-010; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-011"),
        "must NOT contain E-INP-011; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-013"),
        "must NOT contain E-INP-013; got: {err_msg}"
    );
}

/// BC-2.01.011 EC-002 — if_tsresol = 0x09 (base-10, nanoseconds) stored correctly.
///
/// Tests EXTRACTION ONLY. Does NOT test timestamp conversion (STORY-125 scope).
///
/// E-INP code: none (happy path).
#[test]
fn test_BC_2_01_011_if_tsresol_nanosecond() {
    let mut body = Vec::new();
    body.extend_from_slice(&DL_ETHERNET.to_le_bytes());
    body.extend_from_slice(&0u16.to_le_bytes());
    body.extend_from_slice(&65535u32.to_le_bytes());
    body.extend_from_slice(&opt_if_tsresol_le(0x09)); // nanosecond base-10
    body.extend_from_slice(&opt_endofopt_le());

    let result = parse_idb_options(&body, SectionEndianness::LittleEndian);
    assert!(
        result.is_ok(),
        "if_tsresol=0x09 must parse Ok; got: {:?}",
        result.unwrap_err()
    );
    assert_eq!(result.unwrap(), 0x09, "must extract 0x09 verbatim");
    // SCOPE EXCLUSION: we do NOT assert timestamp conversion here (STORY-125).
}

/// BC-2.01.011 EC-003 — if_tsresol = 0x8A (base-2, bit 7 set) stored as raw byte.
///
/// Tests EXTRACTION ONLY. Bit 7 interpretation (base-2 vs base-10) is STORY-125 scope.
///
/// E-INP code: none (happy path).
#[test]
fn test_BC_2_01_011_if_tsresol_base2() {
    let mut body = Vec::new();
    body.extend_from_slice(&DL_ETHERNET.to_le_bytes());
    body.extend_from_slice(&0u16.to_le_bytes());
    body.extend_from_slice(&65535u32.to_le_bytes());
    body.extend_from_slice(&opt_if_tsresol_le(0x8A)); // base-2 exponent 10 (bit7=1)
    body.extend_from_slice(&opt_endofopt_le());

    let result = parse_idb_options(&body, SectionEndianness::LittleEndian);
    assert!(
        result.is_ok(),
        "if_tsresol=0x8A must parse Ok; got: {:?}",
        result.unwrap_err()
    );
    assert_eq!(
        result.unwrap(),
        0x8A,
        "must extract 0x8A verbatim (raw byte, no interpretation)"
    );
    // SCOPE EXCLUSION: base-2 tick-per-second computation is STORY-125 / BC-2.01.014.
}

/// BC-2.01.010 Invariant 4 / BC-2.01.011 PC6 — pure-core BE path: parse_idb_options
/// correctly decodes option_code and option_length in big-endian byte order.
///
/// Pins the BE path at the unit level (mirrors the LE extraction tests above).
/// The if_tsresol value byte (0x09) is endianness-independent (single byte); only
/// the 2-byte option_code and option_length fields require BE decoding.
///
/// Fixture: body = 8 fixed bytes (BE fields) + BE if_tsresol TLV (value=9) + BE endofopt.
/// Expected: Ok(0x09).
///
/// E-INP code: none (happy path).
#[test]
fn test_BC_2_01_011_be_idb_options_pure_core() {
    // Build a body with BE-encoded fixed fields and a BE if_tsresol TLV.
    // The fixed-field bytes are endianness-irrelevant to parse_idb_options (skipped at
    // offset 0–7); we use BE encoding for documentary correctness.
    let mut body = Vec::new();
    body.extend_from_slice(&DL_ETHERNET.to_be_bytes()); // linktype BE (skipped by opts walk)
    body.extend_from_slice(&0u16.to_be_bytes()); // reserved BE (skipped)
    body.extend_from_slice(&65535u32.to_be_bytes()); // snaplen BE (skipped)
    // BE if_tsresol TLV: code=9 as 00 09, length=1 as 00 01, value=0x09, 3 pad bytes.
    body.extend_from_slice(&opt_if_tsresol_be(0x09));
    // BE opt_endofopt: 00 00 00 00.
    body.extend_from_slice(&opt_endofopt_be());

    let result = parse_idb_options(&body, SectionEndianness::BigEndian);
    assert!(
        result.is_ok(),
        "parse_idb_options(BigEndian) with BE if_tsresol TLV must return Ok; got: {:?}",
        result.unwrap_err()
    );
    assert_eq!(
        result.unwrap(),
        0x09,
        "BE parse_idb_options must extract if_tsresol value 0x09 verbatim"
    );
}

// ─── Integration tests via PcapSource::from_pcap_reader ──────────────────────
//
// These tests exercise the full block-walk path. They require SHB + IDB(s) [+ EPB]
// fixtures fed through `PcapSource::from_pcap_reader`.

// ── AC-001: linktype extraction ───────────────────────────────────────────────

/// AC-001 / BC-2.01.011 PC1 / EC-007 — linktype ETHERNET decoded from LE IDB.
///
/// Canonical test vector: SHB(LE) + IDB(linktype=ETHERNET, no options) → Ok
/// with `PcapSource.datalink == ETHERNET`.
///
/// Also partially covers AC-002 (default if_tsresol must be 6 once implemented).
/// E-INP code: none (happy path).
#[test]
fn test_BC_2_01_011_linktype_ethernet() {
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[]));

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "SHB+IDB(ETHERNET, no options) must return Ok; got: {:?}",
        result.unwrap_err()
    );
    let source = result.unwrap();
    assert_eq!(
        source.datalink,
        pcap_file::DataLink::ETHERNET,
        "datalink must be ETHERNET (DL code 1)"
    );
}

/// AC-001 / BC-2.01.011 PC1 — linktype decoded correctly from GENUINE big-endian IDB.
///
/// Non-palindromic value: linktype=ETHERNET=1 is encoded as [00 01] (BE).
/// A LE-reader misreads this as 0x0100 = 256 → wrong linktype.
/// Test proves byte-order correction uses section endianness from SHB BOM.
///
/// E-INP code: none (happy path).
#[test]
fn test_BC_2_01_011_linktype_decoded_big_endian() {
    let mut bytes = be_shb(); // ALL fields big-endian
    bytes.extend_from_slice(&be_idb(DL_ETHERNET, 0, &[])); // linktype=1 encoded as 00 01 BE

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "genuine BE SHB+IDB(ETHERNET) must return Ok (BE endianness applied to linktype); got: {:?}",
        result.unwrap_err()
    );
    let source = result.unwrap();
    assert_eq!(
        source.datalink,
        pcap_file::DataLink::ETHERNET,
        "linktype=1 (00 01 BE) must decode as ETHERNET, not 0x0100=256"
    );
}

/// AC-001 / BC-2.01.011 AC-002 / Invariant 1 — interface table is Vec indexed 0-based.
///
/// With two IDBs of the same linktype, both must be registered (interface indexes 0 and 1).
/// This test accesses `InterfaceInfo` through the public type. The actual Vec is internal
/// to the parse state; this test proves via the public API that two-IDB same-linktype
/// parses succeed and datalink is set to the agreed value.
///
/// NOTE: The test ALSO asserts `InterfaceInfo` is importable and has the expected fields.
/// If `InterfaceInfo` is not `pub` or lacks the `linktype`/`if_tsresol` fields,
/// the test fails at compile time.
///
/// E-INP code: none (happy path).
#[test]
fn test_BC_2_01_011_interface_table_is_vec_indexed() {
    // Two IDBs, same ETHERNET linktype — verifies Vec ordering (first index 0, then 1).
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[])); // interface 0
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[])); // interface 1

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "two same-linktype IDBs must return Ok; got: {:?}",
        result.unwrap_err()
    );
    // Struct-field compile check: InterfaceInfo must be importable with these fields.
    let _iface: InterfaceInfo = InterfaceInfo {
        linktype: pcap_file::DataLink::ETHERNET,
        if_tsresol: DEFAULT_TSRESOL,
    };
    // The public `datalink` field must reflect the agreed linktype.
    assert_eq!(
        result.unwrap().datalink,
        pcap_file::DataLink::ETHERNET,
        "two ETHERNET IDBs: datalink must be ETHERNET"
    );
}

// ── AC-003: IDB error routing (body truncated, nonzero reserved) ──────────────

/// AC-003 / BC-2.01.011 PC5 / EC-008 — IDB body too short (< 8 bytes) → E-INP-008.
///
/// Constructible window: 12 ≤ btl < 20 → body = 0-7 bytes < 8 fixed-field bytes.
/// Canonical fixture: btl=16 → body = 16-12 = 4 bytes.
/// The crate frames and returns the block; wirerust body-decode finds body<8 → E-INP-008.
///
/// AC mapping: AC-003 (body truncated).
/// E-INP code pinned: E-INP-008.
/// Sibling codes NOT present: E-INP-010, E-INP-011, E-INP-013, E-INP-001.
#[test]
fn test_BC_2_01_011_body_truncated_e_inp_008() {
    // Build a manually truncated IDB with btl=16 (body = 4 bytes, < 8 minimum).
    // Outer header (12 bytes): block_type(4) + btl(4) + trailing_btl(4)
    // body = btl - 12 = 16 - 12 = 4 bytes
    let btl: u32 = 16;
    let mut idb_bytes = Vec::new();
    idb_bytes.extend_from_slice(&IDB_BLOCK_TYPE.to_le_bytes()); // block_type
    idb_bytes.extend_from_slice(&btl.to_le_bytes()); // btl = 16 LE
    // 4 body bytes (< 8 minimum; linktype only, no reserved/snaplen)
    idb_bytes.extend_from_slice(&DL_ETHERNET.to_le_bytes()); // 2 bytes of linktype
    idb_bytes.extend_from_slice(&0u16.to_le_bytes()); // 2 bytes of reserved
    // trailing btl
    idb_bytes.extend_from_slice(&btl.to_le_bytes()); // trailing btl = 16 LE
    assert_eq!(
        idb_bytes.len(),
        16,
        "truncated IDB must be exactly 16 bytes"
    );

    let mut bytes = le_shb();
    bytes.extend_from_slice(&idb_bytes);

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "IDB with btl=16 (body=4 bytes < 8 minimum) MUST return Err (E-INP-008)"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    assert!(
        err_msg.contains("E-INP-008"),
        "truncated IDB error must contain E-INP-008; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-010"),
        "must NOT contain E-INP-010 (btl=16 ≥ 12, so crate accepts; body-decode fails); got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-011"),
        "must NOT contain E-INP-011; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-013"),
        "must NOT contain E-INP-013; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-001"),
        "must NOT contain E-INP-001; got: {err_msg}"
    );
}

/// AC-003 / BC-2.01.011 EC-010 — IDB reserved field non-zero → E-INP-008.
///
/// Mirrors crate enforcement at `interface_description.rs:48-49`.
/// A reserved field of 0xDEAD (non-zero) is a structural IDB error.
///
/// AC mapping: AC-003 (nonzero reserved).
/// E-INP code pinned: E-INP-008.
/// Sibling codes NOT present: E-INP-010, E-INP-011, E-INP-013.
#[test]
fn test_BC_2_01_011_nonzero_reserved_e_inp_008() {
    let mut bytes = le_shb();
    // reserved = 0xDEAD (non-zero) — must trigger E-INP-008.
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0xDEAD, &[]));

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "IDB with reserved=0xDEAD MUST return Err (E-INP-008)"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    assert!(
        err_msg.contains("E-INP-008"),
        "nonzero reserved error must contain E-INP-008; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-010"),
        "must NOT contain E-INP-010; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-011"),
        "must NOT contain E-INP-011; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-013"),
        "must NOT contain E-INP-013; got: {err_msg}"
    );
}

// ── AC-005: Late IDB rejection (E-INP-013) ───────────────────────────────────

/// AC-005 / BC-2.01.011 AC-004 / EC-009 — IDB after first EPB → E-INP-013.
///
/// Decision 17 check #1: `packets_emitted > 0` at IDB-parse time → E-INP-013.
/// The IDB body MUST NOT be decoded (interface table not updated).
/// Processing stops immediately.
///
/// Fixture: SHB + IDB(ETHERNET) + EPB(interface_id=0) + IDB(ETHERNET) [late IDB].
/// The second IDB triggers E-INP-013.
///
/// AC mapping: AC-005 (late IDB rejected).
/// E-INP code pinned: E-INP-013.
/// Sibling codes NOT present: E-INP-008, E-INP-010, E-INP-011, E-INP-001.
#[test]
fn test_BC_2_01_011_late_idb_after_packet_rejected_e_inp_013() {
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[])); // interface 0 (valid)
    bytes.extend_from_slice(&le_epb(0)); // EPB → first packet emitted
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[])); // LATE IDB → E-INP-013

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "IDB after first EPB MUST return Err (E-INP-013)"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    assert!(
        err_msg.contains("E-INP-013"),
        "late IDB error must contain E-INP-013; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-008"),
        "must NOT contain E-INP-008 (body not decoded); got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-010"),
        "must NOT contain E-INP-010; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-011"),
        "must NOT contain E-INP-011 (conflict check not evaluated); got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-001"),
        "must NOT contain E-INP-001; got: {err_msg}"
    );
}

// ── AC-006: Three-level precedence (E-INP-013 wins over E-INP-011) ───────────

/// AC-006 / BC-2.01.011 AC-006 / EC-012 — late IDB with conflicting linktype → E-INP-013.
///
/// Decision 17: E-INP-013 position check FIRST; E-INP-011 conflict check is THIRD and
/// NEVER evaluated for a late IDB. Even though the late IDB's linktype (LINUX_SLL)
/// differs from the established linktype (ETHERNET), only E-INP-013 fires.
///
/// Fixture: SHB + IDB(ETHERNET) + EPB + IDB(LINUX_SLL) [late AND conflicting].
/// Expected: Err containing E-INP-013 (NOT E-INP-011, NOT E-INP-001).
///
/// AC mapping: AC-006 (three-level precedence).
/// E-INP code pinned: E-INP-013.
/// Sibling codes NOT present: E-INP-011, E-INP-001, E-INP-008.
#[test]
fn test_BC_2_01_011_idb_precedence_e_inp_013_wins_over_conflict() {
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[])); // first IDB: ETHERNET
    bytes.extend_from_slice(&le_epb(0)); // packet: packets_emitted > 0 now
    // Late IDB with DIFFERENT whitelisted linktype — triggers both position AND conflict.
    // Decision 17: position check wins; E-INP-013 fires; conflict never evaluated.
    bytes.extend_from_slice(&le_idb(DL_LINUX_SLL, 0, &[]));

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "late IDB with conflicting linktype MUST Err with E-INP-013 (position wins)"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    assert!(
        err_msg.contains("E-INP-013"),
        "error must contain E-INP-013 (position wins); got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-011"),
        "E-INP-011 must NOT appear (conflict check is #3, never reached); got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-001"),
        "E-INP-001 must NOT appear; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-008"),
        "E-INP-008 must NOT appear (body not decoded); got: {err_msg}"
    );
}

// ── AC-007: Whitelist enforcement (BC-2.01.016) ──────────────────────────────

/// AC-007 / BC-2.01.016 AC-001 — whitelist covers exactly ETHERNET, RAW, IPV4, IPV6,
/// LINUX_SLL. All five must be accepted; IEEE802_11 (105) must be rejected.
///
/// Also confirms the whitelist is IDENTICAL between classic-pcap and pcapng paths
/// (BC-2.01.016 Invariant 1).
///
/// E-INP code: E-INP-001 on rejection; none on acceptance.
#[test]
fn test_BC_2_01_016_whitelist_mirrors_bc_2_01_001() {
    // All five whitelisted linktypes must succeed.
    let whitelisted = [
        (DL_ETHERNET, "ETHERNET"),
        (DL_RAW, "RAW"),
        (DL_IPV4, "IPV4"),
        (DL_IPV6, "IPV6"),
        (DL_LINUX_SLL, "LINUX_SLL"),
    ];
    for (code, name) in &whitelisted {
        let mut bytes = le_shb();
        bytes.extend_from_slice(&le_idb(*code, 0, &[]));
        let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
        assert!(
            result.is_ok(),
            "whitelisted linktype {name} (code {code}) must return Ok; got: {:?}",
            result.unwrap_err()
        );
    }

    // A non-whitelisted linktype must return Err containing E-INP-001.
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb(DL_IEEE802_11, 0, &[])); // IEEE802_11 = 105, not whitelisted

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "non-whitelisted IEEE802_11 (105) MUST return Err"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    // BC-2.01.016 PC2 / BC-2.01.001: exact message format required.
    assert!(
        err_msg.contains("Unsupported pcap link type"),
        "error must mention 'Unsupported pcap link type'; got: {err_msg}"
    );
    assert!(
        err_msg.contains("Ethernet (1)") && err_msg.contains("Raw IP (101)"),
        "error message must list all 5 supported types in BC-2.01.001 format; got: {err_msg}"
    );
}

/// AC-007 / BC-2.01.016 AC-002 — non-whitelisted IEEE802_11 in IDB returns Err, no panic.
///
/// EC-002: pcapng IDB linktype=IEEE802_11 (105) → Err with the specified message.
/// Message format: `"Unsupported pcap link type: {linktype:?}. Supported: ..."`
///
/// AC mapping: AC-007 (whitelist rejection at IDB-parse time).
/// E-INP code pinned: E-INP-001 (whitelist check #2 per Decision 17).
/// Sibling codes NOT present: E-INP-011, E-INP-013.
#[test]
fn test_BC_2_01_016_non_whitelisted_linktype_returns_err_no_panic() {
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb(DL_IEEE802_11, 0, &[]));

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "IDB with IEEE802_11 linktype MUST return Err (E-INP-001)"
    );
    let err_msg = format!("{:#}", result.unwrap_err());

    // BC-2.01.016 PC2 exact message format (mirrors BC-2.01.001).
    assert!(
        err_msg.contains("Unsupported pcap link type"),
        "must contain 'Unsupported pcap link type'; got: {err_msg}"
    );
    // Must name the rejected type.
    assert!(
        err_msg.contains("IEEE802_11") || err_msg.contains("105"),
        "error must name the rejected linktype (IEEE802_11 or 105); got: {err_msg}"
    );
    // Must list all 5 supported types per canonical format.
    assert!(
        err_msg.contains("Ethernet (1)"),
        "error must list 'Ethernet (1)'; got: {err_msg}"
    );
    assert!(
        err_msg.contains("Raw IP (101)"),
        "error must list 'Raw IP (101)'; got: {err_msg}"
    );
    assert!(
        err_msg.contains("Linux Cooked (113)"),
        "error must list 'Linux Cooked (113)'; got: {err_msg}"
    );
    assert!(
        err_msg.contains("IPv4 (228)"),
        "error must list 'IPv4 (228)'; got: {err_msg}"
    );
    assert!(
        err_msg.contains("IPv6 (229)"),
        "error must list 'IPv6 (229)'; got: {err_msg}"
    );

    // Discriminating: must NOT contain sibling error codes.
    assert!(
        !err_msg.contains("E-INP-011"),
        "must NOT contain E-INP-011 (conflict check #3 preempted by whitelist #2); got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-013"),
        "must NOT contain E-INP-013; got: {err_msg}"
    );
}

/// BC-2.01.016 EC-003 / BC-2.01.018 EC-008 — two IEEE802_11 IDBs: whitelist fires on FIRST.
///
/// The first IDB triggers E-INP-001 at whitelist check #2. The second IDB is never
/// parsed; the agreement/conflict between them is unobservable.
///
/// E-INP code pinned: E-INP-001 (NOT E-INP-011 — they never reach the conflict check).
#[test]
fn test_BC_2_01_016_two_non_whitelisted_idbs_first_fires() {
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb(DL_IEEE802_11, 0, &[])); // first: non-whitelisted
    bytes.extend_from_slice(&le_idb(DL_IEEE802_11, 0, &[])); // second: never parsed

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "Two IEEE802_11 IDBs: MUST Err on first (E-INP-001), second never parsed"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    assert!(
        err_msg.contains("Unsupported pcap link type"),
        "error must be whitelist rejection on first IDB; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-011"),
        "E-INP-011 must NOT appear (conflict check never reached); got: {err_msg}"
    );
}

/// BC-2.01.016 EC-004 / BC-2.01.018 EC-006 — ETHERNET then IEEE802_11.
///
/// First IDB (ETHERNET) passes whitelist. Second IDB (IEEE802_11) hits whitelist check
/// (#2) → E-INP-001. The conflict check (#3) is never reached because whitelist preempts.
///
/// E-INP code pinned: E-INP-001 (NOT E-INP-011).
#[test]
fn test_BC_2_01_016_ethernet_then_non_whitelisted_fires_whitelist() {
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[])); // first: ETHERNET (passes)
    bytes.extend_from_slice(&le_idb(DL_IEEE802_11, 0, &[])); // second: non-whitelisted

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "ETHERNET then IEEE802_11: MUST Err with E-INP-001 on second IDB"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    assert!(
        err_msg.contains("Unsupported pcap link type"),
        "error must be whitelist rejection (E-INP-001); got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-011"),
        "E-INP-011 must NOT appear (conflict check #3 preempted by whitelist #2); got: {err_msg}"
    );
}

// ── AC-008: Multi-IDB conflict → E-INP-011 (BC-2.01.018) ────────────────────

/// AC-008 / BC-2.01.018 PC2 / EC-003 — two IDBs with different whitelisted linktypes.
///
/// ETHERNET (first) vs LINUX_SLL (second) — both are whitelisted, but they differ.
/// The conflict check (#3 per Decision 17) fires on the second IDB.
///
/// Required message format (BC-2.01.018 PC2):
/// `"pcapng multi-interface link-type conflict: interface 0 has {first:?}, interface {n} has {other:?}"`
///
/// AC mapping: AC-008 (conflict detected).
/// E-INP code pinned: E-INP-011.
/// Sibling codes NOT present: E-INP-001, E-INP-008, E-INP-013.
#[test]
fn test_BC_2_01_018_two_idbs_different_linktype_e_inp_011() {
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[])); // interface 0: ETHERNET
    bytes.extend_from_slice(&le_idb(DL_LINUX_SLL, 0, &[])); // interface 1: LINUX_SLL — CONFLICT

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "ETHERNET then LINUX_SLL IDBs MUST return Err (E-INP-011 conflict)"
    );
    let err_msg = format!("{:#}", result.unwrap_err());

    // BC-2.01.018 PC2: exact message format required.
    assert!(
        err_msg.contains("pcapng multi-interface link-type conflict"),
        "error must contain 'pcapng multi-interface link-type conflict'; got: {err_msg}"
    );
    assert!(
        err_msg.contains("interface 0 has"),
        "error must name 'interface 0 has'; got: {err_msg}"
    );
    assert!(
        err_msg.contains("interface 1 has") || err_msg.contains("interface 1"),
        "error must name the conflicting interface index (1); got: {err_msg}"
    );
    assert!(
        err_msg.contains("E-INP-011"),
        "error must contain E-INP-011; got: {err_msg}"
    );

    // BC-2.01.018 AC-001(b) / error-taxonomy E-INP-011: mandatory hint must be present.
    // The hint identifies the common trigger and required remediation.
    assert!(
        err_msg.contains("tcpdump -i any"),
        "error must contain hint 'tcpdump -i any'; got: {err_msg}"
    );
    assert!(
        err_msg.contains("single link type"),
        "error must contain hint 'single link type'; got: {err_msg}"
    );

    // Discriminating: sibling codes must NOT appear.
    assert!(
        !err_msg.contains("E-INP-001"),
        "must NOT contain E-INP-001 (both linktypes are whitelisted); got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-008"),
        "must NOT contain E-INP-008; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-013"),
        "must NOT contain E-INP-013 (no packet before second IDB); got: {err_msg}"
    );
}

/// AC-008 / BC-2.01.018 PC4 / EC-004 — three IDBs: ETHERNET, ETHERNET, RAW.
///
/// Lazy check: first two agree (ETHERNET), third introduces conflict (RAW at index 2).
/// E-INP-011 fires on the third IDB, citing interface 0 vs interface 2.
///
/// AC mapping: AC-008 (three IDBs, third conflicts).
/// E-INP code pinned: E-INP-011.
#[test]
fn test_BC_2_01_018_three_idbs_third_conflicts() {
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[])); // interface 0: ETHERNET
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[])); // interface 1: ETHERNET (agrees)
    bytes.extend_from_slice(&le_idb(DL_RAW, 0, &[])); // interface 2: RAW (CONFLICTS)

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "ETHERNET, ETHERNET, RAW: MUST Err on third IDB (E-INP-011)"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    assert!(
        err_msg.contains("pcapng multi-interface link-type conflict"),
        "error must contain 'pcapng multi-interface link-type conflict'; got: {err_msg}"
    );
    assert!(
        err_msg.contains("interface 0 has"),
        "error must name 'interface 0 has'; got: {err_msg}"
    );
    // Third IDB is interface index 2.
    assert!(
        err_msg.contains("interface 2 has") || err_msg.contains("interface 2"),
        "error must cite interface 2 as the conflicting IDB; got: {err_msg}"
    );
    assert!(
        err_msg.contains("E-INP-011"),
        "must contain E-INP-011; got: {err_msg}"
    );
    // BC-2.01.018 AC-001(b) / error-taxonomy E-INP-011: mandatory hint must be present.
    assert!(
        err_msg.contains("tcpdump -i any"),
        "error must contain hint 'tcpdump -i any'; got: {err_msg}"
    );
    assert!(
        err_msg.contains("single link type"),
        "error must contain hint 'single link type'; got: {err_msg}"
    );
    assert!(
        !err_msg.contains("E-INP-001"),
        "must NOT contain E-INP-001; got: {err_msg}"
    );
}

// ── AC-009: Same-linktype multi-IDB succeeds (BC-2.01.018 PC1) ──────────────

/// AC-009 / BC-2.01.018 PC1 / EC-002 — two IDBs, both ETHERNET: agreement satisfied.
///
/// `PcapSource.datalink` is set to ETHERNET. Parse continues (no error).
///
/// AC mapping: AC-009 (same-linktype multi-IDB passes).
/// E-INP code: none (happy path).
#[test]
fn test_BC_2_01_018_two_idbs_same_linktype_ok() {
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[])); // interface 0
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[])); // interface 1 (same)

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "Two ETHERNET IDBs must return Ok (agreement satisfied); got: {:?}",
        result.unwrap_err()
    );
    assert_eq!(
        result.unwrap().datalink,
        pcap_file::DataLink::ETHERNET,
        "PcapSource.datalink must be ETHERNET when both IDBs agree"
    );
}

/// AC-009 / BC-2.01.011 EC-004 — two IDBs with identical ETHERNET linktype.
///
/// Distinct test from test_BC_2_01_018_two_idbs_same_linktype_ok to cover the BC-2.01.011
/// EC-004 test vector explicitly.
///
/// E-INP code: none (happy path).
#[test]
fn test_BC_2_01_011_two_idbs_same_linktype() {
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[]));
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[]));

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "BC-2.01.011 EC-004: two ETHERNET IDBs must succeed; got: {:?}",
        result.unwrap_err()
    );
    let source = result.unwrap();
    assert_eq!(source.datalink, pcap_file::DataLink::ETHERNET);
    // No conflict: E-INP-011 must NOT have fired.
    // (Verified by is_ok() above.)
}

// ── AC-010: No-panic over arbitrary IDB bytes (SEC-005) ─────────────────────

/// AC-010 / BC-2.01.011 AC-001 — IDB parse path MUST return Err for any malformed bytes.
///
/// Property test: feed arbitrary bytes as an IDB body (with valid SHB wrapping) and
/// verify no panic occurs. SEC-005: unwrap/expect/panic! are prohibited.
///
/// This test uses a fixed set of adversarial inputs rather than full proptest to keep
/// compile complexity low; the formal no-panic VP-028 (fuzz) is a Phase-6 deliverable.
#[test]
fn test_BC_2_01_011_no_panic_fuzz() {
    // A selection of adversarial IDB body byte sequences.
    // For each, we wrap in a valid SHB + a raw IDB block and verify: Ok or Err, never panic.
    let adversarial_bodies: &[&[u8]] = &[
        &[],          // 0 bytes
        &[0x00],      // 1 byte
        &[0xFF; 4],   // 4 bytes (< 8 minimum)
        &[0xFF; 7],   // 7 bytes (still < 8)
        &[0x00; 8],   // 8 bytes, all zero
        &[0xFF; 8],   // 8 bytes, all 0xFF (linktype=65535, reserved=65535)
        &[0x00; 100], // 100 bytes of zeros (options region all zeros)
        &[0xFF; 100], // 100 bytes of 0xFF (adversarial options)
        // Simulate option_length = 65535 (should be bounds-checked before read)
        &[
            0x01, 0x00, // linktype = ETHERNET (LE)
            0x00, 0x00, // reserved = 0
            0xFF, 0xFF, 0xFF, 0x00, // snaplen LE
            0x09, 0x00, // opt code = 9
            0xFF, 0xFF, // opt length = 65535 (overruns body)
        ],
    ];

    for (i, body) in adversarial_bodies.iter().enumerate() {
        // Build a raw IDB block with this body.
        // btl = 12 + body.len(); must be ≥ 12.
        let btl = 12 + body.len();
        let mut idb_bytes = Vec::new();
        idb_bytes.extend_from_slice(&IDB_BLOCK_TYPE.to_le_bytes());
        idb_bytes.extend_from_slice(&(btl as u32).to_le_bytes());
        idb_bytes.extend_from_slice(body);
        idb_bytes.extend_from_slice(&(btl as u32).to_le_bytes());

        let mut stream = le_shb();
        stream.extend_from_slice(&idb_bytes);

        // The block walker must not panic. It may return Ok or Err.
        let result = std::panic::catch_unwind(|| PcapSource::from_pcap_reader(Cursor::new(stream)));
        assert!(
            result.is_ok(), // is_ok() on the catch_unwind result means no panic
            "adversarial IDB body #{i} caused a panic (must return Ok or Err, never panic)"
        );
    }
}

// ── AC-011: VP-030 proptest over whitelisted DataLink values ─────────────────

// VP-030 / BC-2.01.018 VP section — multi-IDB agreement totality over whitelisted values.
// Property: any sequence of whitelisted DataLink values satisfies:
//   - all-equal   → Ok, `PcapSource.datalink` = that DataLink variant
//   - first-diff  → Err containing "pcapng multi-interface link-type conflict"
//                   AND "interface 0 has" AND "E-INP-011"
//   - zero-length → Ok (SHB-only; zero packets, no IDBs — trivial agreement)
// Domain: WHITELISTED values only: {ETHERNET=1, RAW=101, IPV4=228, IPV6=229, LINUX_SLL=113}.
// Non-whitelisted values short-circuit to E-INP-001 at check #2 and are OUT of VP-030 scope.
// Comparison unit: DataLink (typed enum), NOT raw u16.
// SCOPE EXCLUSION: does NOT test timestamp conversion (STORY-125).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn test_VP_030_multi_idb_agreement_totality(
        // Generate a sequence of whitelisted DataLink codes (u8 indexes into the whitelist).
        // 0=ETHERNET, 1=RAW, 2=IPV4, 3=IPV6, 4=LINUX_SLL
        dl_indices in proptest::collection::vec(0usize..5, 0..=6)
    ) {
        let whitelist_codes: [u16; 5] = [DL_ETHERNET, DL_RAW, DL_IPV4, DL_IPV6, DL_LINUX_SLL];
        let whitelist_dl: [pcap_file::DataLink; 5] = [
            pcap_file::DataLink::ETHERNET,
            pcap_file::DataLink::RAW,
            pcap_file::DataLink::IPV4,
            pcap_file::DataLink::IPV6,
            pcap_file::DataLink::LINUX_SLL,
        ];

        // Convert index sequence to (code, DataLink) pairs.
        let linktypes: Vec<(u16, pcap_file::DataLink)> = dl_indices
            .iter()
            .map(|&i| (whitelist_codes[i], whitelist_dl[i]))
            .collect();

        // Build the pcapng stream.
        let mut bytes = le_shb();
        for (code, _) in &linktypes {
            bytes.extend_from_slice(&le_idb(*code, 0, &[]));
        }

        let result = PcapSource::from_pcap_reader(Cursor::new(bytes));

        if linktypes.is_empty() {
            // Zero IDBs: SHB-only file → trivially Ok.
            prop_assert!(
                result.is_ok(),
                "zero IDBs (SHB-only) must return Ok; got: {:?}",
                result.unwrap_err()
            );
        } else {
            // Determine expected outcome.
            let first_dl = linktypes[0].1;
            let conflict_index = linktypes
                .iter()
                .enumerate()
                .skip(1)
                .find(|(_, (_, dl))| *dl != first_dl)
                .map(|(i, _)| i);

            match conflict_index {
                None => {
                    // All equal → Ok.
                    prop_assert!(
                        result.is_ok(),
                        "all-same whitelisted DataLink must return Ok; got: {:?}",
                        result.unwrap_err()
                    );
                    prop_assert_eq!(
                        result.unwrap().datalink,
                        first_dl,
                        "PcapSource.datalink must equal the agreed DataLink"
                    );
                }
                Some(conflict_at) => {
                    // First-differing → Err with E-INP-011.
                    prop_assert!(
                        result.is_err(),
                        "first-differing whitelisted DataLink at index {conflict_at} must Err (E-INP-011)"
                    );
                    let err_msg = format!("{:#}", result.unwrap_err());
                    prop_assert!(
                        err_msg.contains("pcapng multi-interface link-type conflict"),
                        "Err must contain 'pcapng multi-interface link-type conflict'; got: {err_msg}"
                    );
                    prop_assert!(
                        err_msg.contains("interface 0 has"),
                        "Err must contain 'interface 0 has'; got: {err_msg}"
                    );
                    prop_assert!(
                        err_msg.contains("E-INP-011"),
                        "Err must contain E-INP-011; got: {err_msg}"
                    );
                    // BC-2.01.018 AC-001(b) / error-taxonomy E-INP-011: mandatory hint (H-2 pin).
                    prop_assert!(
                        err_msg.contains("tcpdump -i any"),
                        "Err must contain hint 'tcpdump -i any'; got: {err_msg}"
                    );
                    prop_assert!(
                        err_msg.contains("single link type"),
                        "Err must contain hint 'single link type'; got: {err_msg}"
                    );
                }
            }
        }
    }
}

// ── Additional edge case tests ────────────────────────────────────────────────

/// BC-2.01.011 EC-006 — IDB with no options (options section empty).
///
/// if_tsresol absent → default; snaplen read-and-discarded (not stored).
///
/// E-INP code: none (happy path).
#[test]
fn test_BC_2_01_011_idb_no_options() {
    // IDB with exactly 8 bytes of body (fixed fields only, no options region).
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[])); // no options

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "IDB with no options must return Ok; got: {:?}",
        result.unwrap_err()
    );
    // if_tsresol defaults to 6. We cannot directly inspect `InterfaceInfo` through the
    // public `PcapSource` API yet (that would require the Vec to be exposed), but once
    // implemented the `parse_idb_options` call will return DEFAULT_TSRESOL for this fixture.
    // The integration test passes the Ok check; the pure-core test covers the value.
}

/// BC-2.01.011 EC-005 — two IDBs with different linktypes (pure linktype-parse check).
///
/// Canonical test vector: ETHERNET then LINUX_SLL → Err E-INP-011.
/// (Overlaps with AC-008 but directly covers BC-2.01.011 EC-005.)
#[test]
fn test_BC_2_01_011_two_idbs_different_linktype() {
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb(DL_ETHERNET, 0, &[]));
    bytes.extend_from_slice(&le_idb(DL_LINUX_SLL, 0, &[]));

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "BC-2.01.011 EC-005: ETHERNET then LINUX_SLL must return Err (E-INP-011)"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    assert!(
        err_msg.contains("E-INP-011") || err_msg.contains("conflict"),
        "error must mention conflict or E-INP-011; got: {err_msg}"
    );
}

// ── BC-2.01.010 Invariant 4 / BC-2.01.011 PC6 — BE options walk ──────────────
//
// The following helper and integration test expose the hardcoded-LE bug in
// parse_idb_options (lines 325-326 of reader.rs): opt_code and opt_len are both
// decoded with u16::from_le_bytes regardless of section endianness.
//
// For a genuine big-endian pcapng:
//   BE if_tsresol TLV on-disk (code=9, length=1, value=9):
//     00 09  <- opt_code in big-endian: 9
//     00 01  <- opt_length in big-endian: 1
//     09     <- value byte: 9 (nanosecond)
//     00 00 00 <- 3 pad bytes (1+3=4, 4-byte aligned)
//
//   LE-misread (current bug):
//     opt_code  = u16::from_le_bytes([0x00, 0x09]) = 0x0900 = 2304  (not 9)
//     opt_len   = u16::from_le_bytes([0x00, 0x01]) = 0x0100 = 256   (not 1)
//
//   The overrun guard then fires: cursor (4) + opt_len (256) > remaining.len() (8)
//   → E-INP-008, rejecting a structurally valid file.
//
// This violates BC-2.01.010 Invariant 4: the SHB-established section endianness MUST
// govern ALL multi-byte field decoding in ALL blocks, including option code/length.

/// Encode the if_tsresol option (code 9, length 1, value=`value`) in big-endian byte order.
///
/// On-disk layout (big-endian encoding of all multi-byte fields):
///   00 09          <- option_code = 9 as u16 big-endian
///   00 01          <- option_length = 1 as u16 big-endian
///   <value>        <- the single exponent byte
///   00 00 00       <- 3 pad bytes to reach 4-byte alignment (1 + 3 = 4)
///
/// A correct BE-aware options decoder reads opt_code=9, opt_len=1, value=`value`.
/// The current LE-only decoder misreads opt_code=0x0900=2304 and opt_len=0x0100=256,
/// then triggers a spurious E-INP-008 overrun on the 256-byte bounds check.
fn opt_if_tsresol_be(value: u8) -> Vec<u8> {
    let mut v = Vec::with_capacity(8);
    v.extend_from_slice(&OPT_IF_TSRESOL.to_be_bytes()); // 00 09 (big-endian)
    v.extend_from_slice(&1u16.to_be_bytes()); // 00 01 (length=1 big-endian)
    v.push(value); // the exponent byte
    v.extend_from_slice(&[0u8; 3]); // 3 pad bytes (1 + 3 = 4, aligned)
    v
}

/// Encode the opt_endofopt terminator (code 0, length 0) in big-endian byte order.
///
/// Both fields are 0, so the on-disk bytes are 00 00 00 00 — palindromic.
/// Encoded explicitly as BE for documentation completeness.
fn opt_endofopt_be() -> Vec<u8> {
    let mut v = Vec::with_capacity(4);
    v.extend_from_slice(&OPT_ENDOFOPT.to_be_bytes()); // 00 00
    v.extend_from_slice(&0u16.to_be_bytes()); // 00 00 (length=0)
    v
}

/// RED GATE — BC-2.01.010 Invariant 4 / BC-2.01.011 PC6: genuine BE pcapng with
/// an if_tsresol IDB option MUST be accepted; the current LE-only options decoder
/// rejects it with a spurious E-INP-008 overrun.
///
/// # What this tests
///
/// A fully valid big-endian pcapng stream is built:
///   - be_shb(): SHB with on-disk BOM 1A 2B 3C 4D (big-endian section)
///   - be_idb(ETHERNET, 0, opts): IDB with all framing and body fields big-endian,
///     options region = BE if_tsresol(value=9) + BE opt_endofopt
///
/// The options region bytes on-disk are:
///   00 09  <- opt_code=9 in BE
///   00 01  <- opt_len=1 in BE
///   09     <- value byte (nanosecond resolution, non-default → fix is observable)
///   00 00 00 <- 3 pad bytes
///   00 00 00 00 <- opt_endofopt in BE
///
/// A correct endianness-aware options walk (the fix) reads: opt_code=9, opt_len=1,
/// value=9 → Ok(0x09). Result: the parse succeeds.
///
/// The current LE-only walk misreads: opt_code=0x0900=2304, opt_len=0x0100=256.
/// The bounds check cursor(4)+256 > remaining_len(12) fires → Err(E-INP-008),
/// wrongly rejecting a structurally valid file.
///
/// # Violation
///
/// BC-2.01.010 Invariant 4: "The endianness established by the SHB BOM applies to
/// ALL multi-byte fields in ALL blocks of this section — block lengths, interface_id,
/// captured_len, timestamps, option code/length, and all other numeric fields."
///
/// # Contract
///
/// A correctly implemented parse_idb_options (or its successor that takes section
/// endianness) MUST accept this file and return Ok. This test MUST FAIL against the
/// current LE-only implementation and MUST PASS once the fix is applied.
///
/// # Red Gate confirmation
///
/// Current failure: Err("IDB options TLV overrun: option code 2304 declares length 256
/// but only N bytes remain in the options region (E-INP-008: malformed IDB options TLV)")
/// → the test assertion `result.is_ok()` fails.
///
/// BC contract pinned: BC-2.01.010 Invariant 4 (section-wide endianness authority).
/// BC-2.01.011 PC6 (options TLV walk uses section endianness for code/length decode).
/// E-INP code NOT expected: E-INP-008 (this file is structurally valid; the error is spurious).
#[test]
fn test_BC_2_01_011_be_idb_options_accepted() {
    // Build the genuine BE options region: if_tsresol(value=9) + endofopt.
    // Using value=9 (nanosecond) — a non-default value so the fix is meaningfully exercised
    // (default=6 would pass even if the option were silently skipped rather than correctly parsed).
    let mut opts = Vec::new();
    opts.extend_from_slice(&opt_if_tsresol_be(0x09)); // code=9, len=1, value=9 — all BE
    opts.extend_from_slice(&opt_endofopt_be()); // 00 00 00 00

    // Build the complete big-endian pcapng stream: SHB + IDB(ETHERNET, no reserved, opts).
    let mut bytes = be_shb();
    bytes.extend_from_slice(&be_idb(DL_ETHERNET, 0, &opts));

    // This file is structurally valid. A correct BE-aware parser MUST return Ok.
    // The current LE-only parse_idb_options WRONGLY returns Err(E-INP-008) because it
    // misreads opt_len as 256 (0x0100 LE-misread of 00 01 BE) and the overrun guard fires.
    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));

    // Assertion: the current code produces Err (spurious E-INP-008).
    // Once the fix is applied, result.is_ok() must hold instead.
    // This assertion documents the RED state — it MUST FAIL against the current code.
    assert!(
        result.is_ok(),
        "genuine BE pcapng with BE-encoded if_tsresol IDB option MUST be accepted \
         (BC-2.01.010 Invariant 4: section endianness governs option code/length decoding); \
         current LE-only parse_idb_options wrongly rejects it with E-INP-008 — \
         actual error: {:?}",
        result.unwrap_err()
    );
    let source = result.unwrap();
    assert_eq!(
        source.datalink,
        pcap_file::DataLink::ETHERNET,
        "BE IDB with ETHERNET linktype must decode correctly as ETHERNET (not 0x0100=256)"
    );
}

// SCOPE EXCLUSION (not a test):
// STORY-125 / BC-2.01.014 owns: ts_sec and ts_usecs derivation from (ts_high, ts_low, if_tsresol).
// This file only tests EXTRACTION of the if_tsresol byte — NOT its application.
// No timestamp conversion function is imported or called anywhere in this file.
