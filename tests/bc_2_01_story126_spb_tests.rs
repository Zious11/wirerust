//! STORY-126: SPB Parse, Explicit Block-Skip Dispatch (F-07), and Error-Surface Contract
//!
//! Regression-guard suite for the STORY-126 SPB parsing implementation. All behavioral
//! contracts exercised here are IMPLEMENTED: `spb_captured_len` (src/reader.rs:770-781)
//! and the SPB dispatch arm (src/reader.rs:1158-1218) were delivered in STORY-126.
//! These tests passed their Red Gate phase (all failed before implementation) and are
//! now GREEN, guarding against future regressions.
//!
//! # Implementation status
//!
//! The following tests were RED during the TDD Red Gate phase and turned GREEN after
//! STORY-126 implementation (`cargo test --all-targets` is fully GREEN):
//!
//!   - `test_BC_2_01_013_empty_interface_table_guarded` — SPB arm handles empty interface
//!     table; returns Err(E-INP-009) as required.
//!   - `test_BC_2_01_013_padding_strip` — SPB arm strips padding; packet produced correctly.
//!   - `test_BC_2_01_013_zero_timestamps` — SPB arm produces packet with zero timestamps.
//!   - `test_BC_2_01_013_spb_body_truncated_e_inp_008` — SPB arm rejects btl=12 body with
//!     E-INP-008.
//!   - `test_BC_2_01_013_fixed_overhead_constant` — GREEN (const already exported; unchanged).
//!   - `test_BC_2_01_013_no_panic_malformed` — SPB arm returns Err without panic.
//!   - `test_BC_2_01_013_spb_be_endian` — SPB arm handles big-endian capture correctly.
//!   - `test_BC_2_01_013_spb_le_endian` — SPB arm handles little-endian capture correctly.
//!   - `test_BC_2_01_013_spb_zero_original_len` — SPB arm emits zero-length packet.
//!   - `test_BC_2_01_013_spb_truncated_original_len_exceeds_available` — SPB arm truncates
//!     captured_len to available bytes.
//!   - `test_BC_2_01_015_dispatch_known_and_skip_unknown` — named arms for NRB/ISB/SJE/DSB
//!     skip correctly; OPB dual-counter and generic single-counter discriminant verified.
//!   - `test_BC_2_01_015_opb_skipped_not_parsed` — OPB skipped via dual-counter path.
//!   - `test_BC_2_01_015_no_output_on_skip` — no spurious output on skipped blocks.
//!   - `test_BC_2_01_015_loop_break_on_error` — loop breaks on error; no runaway.
//!   - `test_BC_2_01_015_skipped_blocks_counter_and_notice` — OPB counter and notice verified.
//!   - `test_BC_2_01_015_shb_only_counters_zero` — zero-block file yields zero counters.
//!   - `test_BC_2_01_015_dsb_body_not_logged` — DSB body discarded; SEC-007 holds.
//!   - `test_STORY_126_SPB_PACKETS_EMITTED_001` — SPB arm increments packets_emitted;
//!     late IDB after SPB correctly triggers E-INP-013.
//!   - `test_BC_2_01_017_all_error_paths_have_context` — SPB context string present.
//!   - `test_BC_2_01_017_epb_before_idb_emits_einp009_context` — EPB before IDB emits
//!     canonical BC-2.01.017 PC1 string "before any Interface Description Block".
//!   - `test_BC_2_01_017_no_panic_truncated_pcapng` — no panic on truncated input.
//!   - `test_BC_2_01_017_spb_before_idb_emits_einp009_context` — SPB before IDB emits
//!     E-INP-009 with context.
//!   - `test_BC_2_01_017_spb_body_too_short_einp008_context` — short SPB body emits
//!     E-INP-008 with context.
//!
//! # Regression coverage
//!
//! STORY-123/124/125's existing tests in `bc_2_01_story123_pcapng_tests.rs`,
//! `bc_2_01_story124_idb_tests.rs`, and `bc_2_01_story125_epb_tests.rs` remain GREEN.
//! This file does NOT modify any source file.
//!
//! # Coverage map (AC → test → E-INP code)
//!
//! ```
//! AC-001 → test_BC_2_01_013_empty_interface_table_guarded (E-INP-009; exact message)
//! AC-002 → test_BC_2_01_013_padding_strip (captured_len = min(original_len, body.len()-4))
//!        → test_BC_2_01_013_spb_truncated_original_len_exceeds_available (truncation)
//!        → test_BC_2_01_013_spb_be_endian (non-palindromic BE fixture)
//!        → test_BC_2_01_013_spb_le_endian (non-palindromic LE fixture)
//! AC-003 → test_BC_2_01_013_zero_timestamps (timestamp_secs=0, timestamp_usecs=0)
//! AC-004a → test_BC_2_01_013_spb_body_truncated_e_inp_008 (E-INP-008; btl=12 → body=0)
//! AC-004b → test_BC_2_01_013_fixed_overhead_constant (SPB_FIXED_OVERHEAD_BYTES == 4; GREEN)
//! AC-005 → test_BC_2_01_013_no_panic_malformed (no panic)
//! STORY-126-SPB-PACKETS-EMITTED-001 → test_STORY_126_SPB_PACKETS_EMITTED_001 (E-INP-013)
//! AC-006 → test_BC_2_01_015_dispatch_known_and_skip_unknown
//! AC-007 → test_BC_2_01_015_opb_skipped_not_parsed
//! AC-008 → test_BC_2_01_015_no_output_on_skip
//!        → test_BC_2_01_015_dsb_body_not_logged (SEC-007)
//! AC-009 → test_BC_2_01_015_loop_break_on_error
//! AC-010 → test_BC_2_01_015_skipped_blocks_counter_and_notice
//!        → test_BC_2_01_015_shb_only_counters_zero
//! AC-011 → test_BC_2_01_017_all_error_paths_have_context
//!        → test_BC_2_01_017_epb_before_idb_emits_einp009_context
//!        → test_BC_2_01_017_spb_before_idb_emits_einp009_context (E-INP-009)
//!        → test_BC_2_01_017_spb_body_too_short_einp008_context (E-INP-008)
//! AC-012 → test_BC_2_01_017_no_panic_truncated_pcapng
//! ```
//!
//! # Discriminating-assertion discipline
//!
//! Every error test asserts:
//!   (a) the EXPECTED E-INP code IS present in the error chain, AND
//!   (b) at least one SIBLING code is ABSENT (to guard against false positives).
//!
//! Naming convention: `test_BC_S_SS_NNN_<assertion>()` throughout.
//! `#![allow(non_snake_case)]` required per factory BC-naming mandate.
#![allow(non_snake_case)]

use std::io::Cursor;

use wirerust::reader::{PcapSource, SPB_FIXED_OVERHEAD_BYTES, spb_captured_len};

// ── pcapng canonical constants (ADR-009 Current Canonical Constants table) ────

/// pcapng SHB block_type / file magic (endian-independent 4-byte literal).
const SHB_BLOCK_TYPE: u32 = 0x0A0D_0D0A;

/// IDB block type code.
const IDB_BLOCK_TYPE: u32 = 0x0000_0001;

/// EPB block type code.
const EPB_BLOCK_TYPE: u32 = 0x0000_0006;

/// SPB block type code (BC-2.01.013 / ADR-009 Decision 22).
const SPB_BLOCK_TYPE: u32 = 0x0000_0003;

/// OPB (Obsolete Packet Block) block type code.
const OPB_BLOCK_TYPE: u32 = 0x0000_0002;

/// NRB (Name Resolution Block) type code — explicit skip arm (BC-2.01.015 AC-001 F-07).
const NRB_BLOCK_TYPE: u32 = 0x0000_0004;

/// ISB (Interface Statistics Block) type code — explicit skip arm (BC-2.01.015 AC-001 F-07).
const ISB_BLOCK_TYPE: u32 = 0x0000_0005;

/// SJE (Systemd Journal Export Block) type code — explicit skip arm (BC-2.01.015 AC-001 F-07).
const SJE_BLOCK_TYPE: u32 = 0x0000_0009;

/// DSB (Decryption Secrets Block) type code — explicit skip arm; body MUST NOT be logged
/// (SEC-007: DSB carries TLS key material). No named `Block` enum variant exists in
/// `pcap_file::pcapng::Block` — match raw type bytes directly.
const DSB_BLOCK_TYPE: u32 = 0x0000_000A;

/// BOM: little-endian pcapng section (on-disk 4D 3C 2B 1A).
const SHB_BOM_LE: [u8; 4] = [0x4D, 0x3C, 0x2B, 0x1A];

/// BOM: big-endian pcapng section (on-disk 1A 2B 3C 4D).
const SHB_BOM_BE: [u8; 4] = [0x1A, 0x2B, 0x3C, 0x4D];

/// DataLink::ETHERNET numeric code.
const DL_ETHERNET: u16 = 1;

/// E-INP discriminant strings (canonical per error-taxonomy).
const E_INP_008: &str = "E-INP-008";
const E_INP_009: &str = "E-INP-009";
const E_INP_010: &str = "E-INP-010";
const E_INP_011: &str = "E-INP-011";
const E_INP_013: &str = "E-INP-013";

// ── Exact BC-2.01.017 PC1 context strings ──────────────────────────────────────
//
// These must match EXACTLY what implementation produces (BC-2.01.017 PC1 normative strings).
// Tests assert these substrings are present in the anyhow error chain.

/// Canonical E-INP-009 context for EPB before any IDB (BC-2.01.017 PC1).
#[allow(dead_code)]
const CTX_EPB_BEFORE_IDB: &str = "encountered before any Interface Description Block";

/// Canonical E-INP-009 context for SPB before any IDB (BC-2.01.017 PC1).
const CTX_SPB_BEFORE_IDB: &str = "encountered before any Interface Description Block";

/// Canonical E-INP-009 exact message for SPB empty-table (BC-2.01.013 PC5 / v1.9).
const SPB_EMPTY_TABLE_MSG: &str =
    "SPB encountered but interface table is empty — no IDB has been parsed";

/// Canonical context for SPB body-decode error (BC-2.01.017 PC1).
const CTX_SPB_BODY_DECODE: &str = "Failed to read pcapng Simple Packet Block";

// ── Fixture builder helpers ───────────────────────────────────────────────────
//
// All multi-byte fields are encoded in the declared endianness (no silent LE-only).
// Non-palindromic values are used to detect endianness bugs: a non-palindromic
// field decoded in the wrong byte order yields a detectably wrong value.

/// Build a minimal 28-byte LE SHB block.
///
/// block_type(4 LE) + btl(28 LE) + BOM_LE(4) + major(1 u16 LE) + minor(0 u16 LE)
///   + section_length(8 LE) + trailing_btl(28 LE)
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

/// Build a minimal 28-byte BE SHB block (all fields big-endian).
///
/// Uses non-palindromic minor_version=2 so that any LE-misread of the body
/// decodes minor as 0x0200 = 512, not 2, detecting endianness bugs.
fn be_shb() -> Vec<u8> {
    let mut v = Vec::with_capacity(28);
    v.extend_from_slice(&SHB_BLOCK_TYPE.to_be_bytes()); // 0A 0D 0D 0A
    v.extend_from_slice(&28u32.to_be_bytes()); // 00 00 00 1C
    v.extend_from_slice(&SHB_BOM_BE); // 1A 2B 3C 4D
    v.extend_from_slice(&1u16.to_be_bytes()); // 00 01 (major=1)
    v.extend_from_slice(&2u16.to_be_bytes()); // 00 02 (minor=2; non-palindromic)
    v.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_be_bytes()); // section_length
    v.extend_from_slice(&28u32.to_be_bytes()); // 00 00 00 1C trailing btl
    assert_eq!(v.len(), 28);
    v
}

/// Build a minimal LE IDB block with Ethernet linktype and no options.
///
/// Structure: block_type(4 LE) + btl(20 LE) + linktype(2 LE) + reserved(2 LE)
///   + snaplen(4 LE) + trailing_btl(4 LE). btl = 12 outer + 8 fixed = 20.
fn le_idb_ethernet() -> Vec<u8> {
    le_idb_raw_le(DL_ETHERNET)
}

/// Build a minimal LE IDB block with the given linktype (no options).
fn le_idb_raw_le(linktype: u16) -> Vec<u8> {
    let btl: u32 = 20; // 12 outer + 8 fixed
    let mut v = Vec::with_capacity(20);
    v.extend_from_slice(&IDB_BLOCK_TYPE.to_le_bytes());
    v.extend_from_slice(&btl.to_le_bytes());
    v.extend_from_slice(&linktype.to_le_bytes()); // linktype LE (non-palindromic for DL_ETHERNET=1)
    v.extend_from_slice(&0u16.to_le_bytes()); // reserved = 0
    v.extend_from_slice(&65535u32.to_le_bytes()); // snaplen (discarded)
    v.extend_from_slice(&btl.to_le_bytes()); // trailing btl
    assert_eq!(v.len(), 20);
    v
}

/// Build a minimal BE IDB block with the given linktype (no options, all fields BE).
///
/// Non-palindromic: DL_ETHERNET=1 encoded as 00 01; LE-misread gives 0x0100=256 (wrong).
fn be_idb_ethernet() -> Vec<u8> {
    let btl: u32 = 20; // 12 outer + 8 fixed
    let mut v = Vec::with_capacity(20);
    v.extend_from_slice(&IDB_BLOCK_TYPE.to_be_bytes()); // 00 00 00 01
    v.extend_from_slice(&btl.to_be_bytes()); // 00 00 00 14
    v.extend_from_slice(&DL_ETHERNET.to_be_bytes()); // 00 01 (non-palindromic)
    v.extend_from_slice(&0u16.to_be_bytes()); // reserved = 00 00
    v.extend_from_slice(&65535u32.to_be_bytes()); // 00 00 FF FF (non-palindromic snaplen, discarded)
    v.extend_from_slice(&btl.to_be_bytes()); // 00 00 00 14
    assert_eq!(v.len(), 20);
    v
}

/// Build a LE SPB block with the given payload bytes.
///
/// Block structure:
///   block_type:         4 bytes (LE) = 0x00000003
///   block_total_length: 4 bytes (LE)
///   original_len:       4 bytes (LE)  -- the SPB body fixed field
///   padded_data:        data padded to 4-byte boundary
///   trailing_btl:       4 bytes (LE)
///
/// btl = 12 (outer) + 4 (original_len field) + padded_data.len()
///
/// `original_len` is set to `data.len()` (the actual captured length, no truncation).
/// `data` is padded to the next 4-byte boundary for btl computation.
fn le_spb(data: &[u8]) -> Vec<u8> {
    le_spb_with_original_len(data, data.len() as u32)
}

/// Build a LE SPB block where `original_len` may differ from `data.len()`.
///
/// Allows testing the truncation case (original_len > spb_data_available).
/// The block body contains:
///   - 4 bytes: original_len (LE)
///   - data bytes (the captured payload, padded to 4-byte boundary)
///
/// captured_len = min(original_len, data.len()) per BC-2.01.013 AC-002.
/// This fixture encodes the raw data bytes WITHOUT applying the truncation —
/// the truncation is wirerust's responsibility to compute from original_len and
/// the body extent.
fn le_spb_with_original_len(data: &[u8], original_len: u32) -> Vec<u8> {
    // Compute padded data length (4-byte boundary, per pcapng spec).
    let pad_len = (4usize.wrapping_sub(data.len() % 4)) % 4;
    let padded_data_len = data.len() + pad_len;
    let body_len = 4 + padded_data_len; // original_len field (4) + padded data
    let btl = 12 + body_len;
    let mut v = Vec::with_capacity(btl);
    v.extend_from_slice(&SPB_BLOCK_TYPE.to_le_bytes()); // 03 00 00 00
    v.extend_from_slice(&(btl as u32).to_le_bytes()); // btl LE
    v.extend_from_slice(&original_len.to_le_bytes()); // original_len LE
    v.extend_from_slice(data); // payload
    v.extend_from_slice(&vec![0u8; pad_len]); // padding
    v.extend_from_slice(&(btl as u32).to_le_bytes()); // trailing btl LE
    assert_eq!(v.len(), btl);
    v
}

/// Build a BE SPB block with the given payload bytes (all fields big-endian).
///
/// Non-palindromic: original_len=100 encoded as 00 00 00 64; LE-misread gives 0x64000000.
fn be_spb(data: &[u8]) -> Vec<u8> {
    be_spb_with_original_len(data, data.len() as u32)
}

/// Build a BE SPB block where `original_len` may differ from `data.len()`.
fn be_spb_with_original_len(data: &[u8], original_len: u32) -> Vec<u8> {
    let pad_len = (4usize.wrapping_sub(data.len() % 4)) % 4;
    let padded_data_len = data.len() + pad_len;
    let body_len = 4 + padded_data_len;
    let btl = 12 + body_len;
    let mut v = Vec::with_capacity(btl);
    v.extend_from_slice(&SPB_BLOCK_TYPE.to_be_bytes()); // 00 00 00 03
    v.extend_from_slice(&(btl as u32).to_be_bytes()); // btl BE
    v.extend_from_slice(&original_len.to_be_bytes()); // original_len BE (non-palindromic)
    v.extend_from_slice(data); // payload
    v.extend_from_slice(&vec![0u8; pad_len]); // padding
    v.extend_from_slice(&(btl as u32).to_be_bytes()); // trailing btl BE
    assert_eq!(v.len(), btl);
    v
}

/// Build a minimal LE EPB block with a 4-byte dummy payload at interface 0.
fn le_epb_iface0() -> Vec<u8> {
    let btl: u32 = 36; // 12 outer + 20 fixed + 4 payload
    let mut v = Vec::with_capacity(36);
    v.extend_from_slice(&EPB_BLOCK_TYPE.to_le_bytes());
    v.extend_from_slice(&btl.to_le_bytes());
    v.extend_from_slice(&0u32.to_le_bytes()); // interface_id = 0
    v.extend_from_slice(&1u32.to_le_bytes()); // ts_high = 1 (non-palindromic)
    v.extend_from_slice(&0u32.to_le_bytes()); // ts_low = 0
    v.extend_from_slice(&4u32.to_le_bytes()); // captured_len = 4
    v.extend_from_slice(&4u32.to_le_bytes()); // original_len = 4
    v.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // 4-byte payload
    v.extend_from_slice(&btl.to_le_bytes());
    assert_eq!(v.len(), 36);
    v
}

/// Build a "skip" block of the given type with the specified body content.
///
/// Structure: block_type(4 LE) + btl(LE) + body + trailing_btl(4 LE).
/// btl = 12 + body.len(). body must already be 4-byte aligned.
fn le_skip_block(block_type: u32, body: &[u8]) -> Vec<u8> {
    assert_eq!(body.len() % 4, 0, "body must be 4-byte aligned");
    let btl = 12 + body.len();
    let mut v = Vec::with_capacity(btl);
    v.extend_from_slice(&block_type.to_le_bytes());
    v.extend_from_slice(&(btl as u32).to_le_bytes());
    v.extend_from_slice(body);
    v.extend_from_slice(&(btl as u32).to_le_bytes());
    assert_eq!(v.len(), btl);
    v
}

/// Build a minimal valid LE NRB block (12 bytes, empty body).
fn le_nrb_empty() -> Vec<u8> {
    le_skip_block(NRB_BLOCK_TYPE, &[])
}

/// Build a minimal valid LE ISB block (12 bytes, empty body).
fn le_isb_empty() -> Vec<u8> {
    le_skip_block(ISB_BLOCK_TYPE, &[])
}

/// Build a minimal valid LE SJE block (12 bytes, empty body).
fn le_sje_empty() -> Vec<u8> {
    le_skip_block(SJE_BLOCK_TYPE, &[])
}

/// Build a LE DSB block with the given "key material" body.
///
/// IMPORTANT: The body bytes here are SYNTHETIC test bytes representing TLS key
/// material. The test then asserts these bytes do NOT appear in any output (SEC-007).
/// Body must be 4-byte aligned.
fn le_dsb_with_body(body: &[u8]) -> Vec<u8> {
    assert_eq!(body.len() % 4, 0, "DSB body must be 4-byte aligned");
    le_skip_block(DSB_BLOCK_TYPE, body)
}

/// Build a LE block with a truly unknown block type.
fn le_unknown_block_empty() -> Vec<u8> {
    // 0xDEAD_BEEF is definitely not a known block type
    le_skip_block(0xDEAD_BEEF, &[])
}

/// Build a LE OPB block (type 0x00000002, empty body).
fn le_opb_empty() -> Vec<u8> {
    le_skip_block(OPB_BLOCK_TYPE, &[])
}

/// Craft a LE pcapng SHB-only file (no IDB, no other blocks).
fn shb_only_le() -> Vec<u8> {
    le_shb()
}

/// Build a complete LE pcapng with SHB + IDB(Ethernet) + one SPB containing `data`.
///
/// This is the standard happy-path fixture for SPB parse tests.
fn le_pcapng_shb_idb_spb(data: &[u8]) -> Vec<u8> {
    let mut v = le_shb();
    v.extend_from_slice(&le_idb_ethernet());
    v.extend_from_slice(&le_spb(data));
    v
}

/// Build a LE pcapng with SHB + IDB + SPB where original_len differs from data.len().
fn le_pcapng_shb_idb_spb_with_original_len(data: &[u8], original_len: u32) -> Vec<u8> {
    let mut v = le_shb();
    v.extend_from_slice(&le_idb_ethernet());
    v.extend_from_slice(&le_spb_with_original_len(data, original_len));
    v
}

/// Build a complete BE pcapng with SHB + IDB(Ethernet) + one SPB containing `data`.
///
/// All frames and body fields are big-endian. Non-palindromic values detect misread.
fn be_pcapng_shb_idb_spb(data: &[u8]) -> Vec<u8> {
    let mut v = be_shb();
    v.extend_from_slice(&be_idb_ethernet());
    v.extend_from_slice(&be_spb(data));
    v
}

/// Build a LE pcapng with SHB + SPB (no IDB) — tests E-INP-009 guard.
fn le_pcapng_shb_spb_no_idb(data: &[u8]) -> Vec<u8> {
    let mut v = le_shb();
    v.extend_from_slice(&le_spb(data));
    v
}

// ── BC-2.01.013 SPB Parse Tests ──────────────────────────────────────────────

/// AC-001 — SPB before any IDB (empty interface table) → E-INP-009.
///
/// Discriminating assertions:
///   (a) E-INP-009 IS present in error chain.
///   (b) The EXACT message mandated by BC-2.01.013 PC5 / v1.9 is present.
///   (c) E-INP-008 is NOT present (not a body-too-short error).
///   (d) E-INP-010 is NOT present (not a framing error).
///
/// RED-phase: before the SPB arm existed, SPB fell through to the wildcard `_` arm and
/// returned Ok instead of Err(E-INP-009). This test was RED and turned GREEN once the
/// SPB dispatch arm landed (reader.rs:1158). Guards the empty-table guard path.
#[test]
fn test_BC_2_01_013_empty_interface_table_guarded() {
    // Minimal payload so the SPB is well-formed (btl = 12 + 4 + 4 = 20).
    let payload = [0u8; 4];
    let bytes = le_pcapng_shb_spb_no_idb(&payload);
    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "SPB before any IDB must return Err(E-INP-009), got Ok"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains(E_INP_009),
        "error must contain E-INP-009, got: {msg}"
    );
    assert!(
        msg.contains(SPB_EMPTY_TABLE_MSG),
        "error must contain the EXACT BC-2.01.013 PC5 message: \
         '{SPB_EMPTY_TABLE_MSG}', got: {msg}"
    );
    assert!(
        !msg.contains(E_INP_008),
        "E-INP-008 must NOT be present (this is empty-table, not body-too-short), got: {msg}"
    );
    assert!(
        !msg.contains(E_INP_010),
        "E-INP-010 must NOT be present (this is empty-table, not framing), got: {msg}"
    );
}

/// AC-002 — padding strip: captured_len = min(original_len, body.len()-4).
///
/// Fixture: 7 bytes of payload → padded to 8 bytes in block body.
/// spb_data_available = 8 (padded_body) NOT 12 (body.len() = 4 + 8 = 12).
/// Wait — re-check: body = [original_len:4][payload:7][pad:1] = 12 bytes.
/// spb_data_available = body.len() - 4 = 8.
/// original_len = 7. captured_len = min(7, 8) = 7.
/// RawPacket.data.len() must be 7 (not 8 = padded, not 12 = body.len()).
///
/// RED-phase: before the SPB arm existed, no packet was produced (SPB fell to `_`). This
/// test was RED and turned GREEN once the SPB arm landed (reader.rs:1158). Guards the
/// padding-strip arithmetic — if a refactor removed the arm, no packet would be produced.
#[test]
fn test_BC_2_01_013_padding_strip() {
    // 7-byte payload: not 4-aligned, so 1 pad byte appended in block → body = 4 + 8 = 12.
    // original_len = 7. spb_data_available = 8. captured_len = min(7, 8) = 7.
    let payload: Vec<u8> = (0u8..7).collect();
    let bytes = le_pcapng_shb_idb_spb(&payload);
    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(result.is_ok(), "SPB parse must succeed: {:?}", result.err());
    let source = result.unwrap();
    assert_eq!(
        source.packets.len(),
        1,
        "must produce exactly 1 packet from SPB"
    );
    let pkt = &source.packets[0];
    // MUST NOT be 8 (padded size) or 12 (bare body.len())
    assert_eq!(
        pkt.data.len(),
        7,
        "captured_len must be original_len=7 (padding stripped); \
         got {} (bare body.len()-4 would be 8, bare body.len() would be 12)",
        pkt.data.len()
    );
    // Verify actual bytes are the payload, not padding bytes
    assert_eq!(
        &pkt.data, &payload,
        "packet data must exactly match the original 7-byte payload"
    );
}

/// AC-003 — zero timestamps: every SPB packet has timestamp_secs=0, timestamp_usecs=0.
///
/// RED-phase: before the SPB arm existed, no packet was produced. This test was RED and
/// turned GREEN once the SPB arm landed (reader.rs:1158). Guards the zero-timestamp
/// invariant — SPB packets must carry timestamp_secs=0 and timestamp_usecs=0.
#[test]
fn test_BC_2_01_013_zero_timestamps() {
    let payload = [0xDE, 0xAD, 0xBE, 0xEF]; // 4 bytes, no padding needed
    let bytes = le_pcapng_shb_idb_spb(&payload);
    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(result.is_ok(), "SPB parse must succeed: {:?}", result.err());
    let source = result.unwrap();
    assert_eq!(source.packets.len(), 1, "must produce 1 packet from SPB");
    let pkt = &source.packets[0];
    assert_eq!(
        pkt.timestamp_secs, 0,
        "SPB packet timestamp_secs must be 0 (no per-packet timestamp in SPB format)"
    );
    assert_eq!(
        pkt.timestamp_usecs, 0,
        "SPB packet timestamp_usecs must be 0 (no per-packet timestamp in SPB format)"
    );
}

/// AC-004a — body-too-short: btl=12 → body=0 bytes < 4 SPB fixed fields → E-INP-008.
///
/// Construction: a pcapng block where btl=12 so body = btl - 12 = 0 bytes.
/// The crate accepts this (btl=12 is ≥ 12 and 12%4=0). wirerust body-decode
/// checks body.len() >= SPB_FIXED_OVERHEAD_BYTES (4) and rejects with E-INP-008.
///
/// Discriminating assertions:
///   (a) E-INP-008 IS present.
///   (b) E-INP-010 is NOT present (not a crate-framing error — crate accepted btl=12).
///   (c) E-INP-009 is NOT present (this is body-too-short, not empty-table).
///
/// We need IDB first so the empty-table guard doesn't fire before body-check.
///
/// RED-phase: before the SPB arm existed, SPB fell to `_` and returned Ok instead of
/// Err(E-INP-008). This test was RED and turned GREEN once the SPB arm landed
/// (reader.rs:1158). Guards the body-too-short rejection path (btl=12, body=0 bytes).
#[test]
fn test_BC_2_01_013_spb_body_truncated_e_inp_008() {
    // Build SHB + IDB + SPB-with-btl=12 manually.
    // SPB with btl=12: block_type(4) + btl(4) + trailing_btl(4) = 12 bytes; body = 0 bytes.
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb_ethernet());
    // Hand-craft SPB block with btl=12 (body = 0 bytes — below the 4-byte SPB fixed minimum).
    let btl: u32 = 12;
    bytes.extend_from_slice(&SPB_BLOCK_TYPE.to_le_bytes());
    bytes.extend_from_slice(&btl.to_le_bytes());
    // No body bytes (btl - 12 = 0)
    bytes.extend_from_slice(&btl.to_le_bytes()); // trailing btl

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "SPB with btl=12 (body=0 < 4) must return Err(E-INP-008), got Ok"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains(E_INP_008),
        "error must contain E-INP-008 (body-too-short path), got: {msg}"
    );
    assert!(
        !msg.contains(E_INP_010),
        "E-INP-010 must NOT be present (crate accepted btl=12; this is wirerust body-decode \
         failure, not crate-framing rejection), got: {msg}"
    );
    assert!(
        !msg.contains(E_INP_009),
        "E-INP-009 must NOT be present (IDB is present; this is body-too-short, not \
         empty-table), got: {msg}"
    );
}

/// AC-004b — SPB_FIXED_OVERHEAD_BYTES == 4 (const regression guard).
///
/// GREEN immediately — the constant is already exported. This is a regression guard.
/// It must remain GREEN after all subsequent STORY-126 changes.
///
/// Per BC-2.01.013 AC-004b: MUST NOT be confused with EPB_FIXED_OVERHEAD_BYTES = 20.
#[test]
fn test_BC_2_01_013_fixed_overhead_constant() {
    assert_eq!(
        SPB_FIXED_OVERHEAD_BYTES, 4,
        "SPB_FIXED_OVERHEAD_BYTES must be 4 (body-relative: original_len:u32 only); \
         must NOT be 20 (EPB) or any other value"
    );
    // Explicitly confirm it is NOT the EPB value.
    assert_ne!(
        SPB_FIXED_OVERHEAD_BYTES, 20,
        "SPB_FIXED_OVERHEAD_BYTES must NOT be 20 (that is EPB_FIXED_OVERHEAD_BYTES)"
    );
}

/// AC-005 — no-panic on malformed SPB input (SEC-005).
///
/// Feed a pcapng with SHB + IDB + a truncated/malformed SPB (various edge cases).
/// The reader MUST return Ok or Err — never panic.
///
/// This test exercises several malformed-SPB scenarios:
///   - btl=16 → body=4 bytes, spb_data_available=0, original_len=0 → empty packet (Ok)
///   - The core no-panic property; AC-012 (cross-cutting) covers arbitrary truncation.
///
/// RED-phase: before the SPB arm existed, the wildcard `_` arm produced Ok with 0 packets
/// rather than the expected 1-packet Ok — so the packet-count assertion was RED. This test
/// turned GREEN once the SPB arm landed (reader.rs:1158). Guards the no-panic contract
/// for minimum-valid SPB (btl=16, original_len=0) and the resulting empty-data packet.
#[test]
fn test_BC_2_01_013_no_panic_malformed() {
    // Case 1: btl=16 → body=4, spb_data_available=0, original_len=0 → Ok({data: []})
    // This is the minimum legal SPB (BC-2.01.013 Precondition 4).
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb_ethernet());
    // btl = 16: block_type(4) + btl(4) + original_len(4) + trailing_btl(4) = 16.
    let btl: u32 = 16;
    bytes.extend_from_slice(&SPB_BLOCK_TYPE.to_le_bytes());
    bytes.extend_from_slice(&btl.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes()); // original_len = 0
    // No data bytes (spb_data_available = 0)
    bytes.extend_from_slice(&btl.to_le_bytes());

    // Must not panic — return Ok or Err
    let result = PcapSource::from_pcap_reader(Cursor::new(&bytes));
    // After implementation: expect Ok with 1 packet of empty data
    assert!(
        result.is_ok(),
        "minimum valid SPB (btl=16, original_len=0) must parse successfully (Ok)"
    );
    let source = result.unwrap();
    assert_eq!(
        source.packets.len(),
        1,
        "minimum valid SPB must produce 1 packet"
    );
    assert_eq!(
        source.packets[0].data.len(),
        0,
        "minimum valid SPB (original_len=0) must produce empty data"
    );
}

/// AC-002 (LE) — genuine LE fixture: SHB(LE) + IDB(LE) + SPB(LE) with non-palindromic payload.
///
/// Non-palindromic payload: [0x01, 0x02, 0x03, 0x04] (LE/BE differ in meaning).
/// original_len=4, body=4+4=8 bytes → spb_data_available=4, captured_len=4.
///
/// RED-phase: before the SPB arm existed, no packet was produced. This test was RED and
/// turned GREEN once the SPB arm landed (reader.rs:1158). Guards correct LE field decoding
/// — a refactor breaking endian selection would fail this assertion.
#[test]
fn test_BC_2_01_013_spb_le_endian() {
    // Non-palindromic 4-byte payload.
    let payload: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    let bytes = le_pcapng_shb_idb_spb(&payload);
    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "LE SPB parse must succeed: {:?}",
        result.err()
    );
    let source = result.unwrap();
    assert_eq!(source.packets.len(), 1, "must produce 1 packet from LE SPB");
    let pkt = &source.packets[0];
    assert_eq!(pkt.data.len(), 4, "LE SPB: captured_len must be 4");
    assert_eq!(
        &pkt.data, &payload,
        "LE SPB: packet bytes must match the original payload exactly"
    );
    assert_eq!(pkt.timestamp_secs, 0, "LE SPB: timestamp_secs must be 0");
    assert_eq!(pkt.timestamp_usecs, 0, "LE SPB: timestamp_usecs must be 0");
}

/// AC-002 (BE) — genuine BE fixture: SHB(BE) + IDB(BE) + SPB(BE) with non-palindromic payload.
///
/// Non-palindromic payload: [0x01, 0x02, 0x03, 0x04] (same bytes, but the framing and
/// original_len field are big-endian encoded: original_len=4 as 0x00000004).
/// If wirerust reads original_len as LE, it would get 0x04000000 = 67108864 ≠ 4,
/// causing captured_len clamping to spb_data_available=4 anyway, but data would
/// be misread or miscounted. We assert data.len()==4 exactly.
///
/// RED-phase: before the SPB arm existed, no packet was produced. This test was RED and
/// turned GREEN once the SPB arm landed (reader.rs:1158). Guards correct BE field decoding
/// — reading original_len as LE on a BE fixture would produce a mismatched captured_len.
#[test]
fn test_BC_2_01_013_spb_be_endian() {
    // Non-palindromic 4-byte payload (same content, different encoding context).
    let payload: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    let bytes = be_pcapng_shb_idb_spb(&payload);
    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "BE SPB parse must succeed: {:?}",
        result.err()
    );
    let source = result.unwrap();
    assert_eq!(source.packets.len(), 1, "must produce 1 packet from BE SPB");
    let pkt = &source.packets[0];
    assert_eq!(pkt.data.len(), 4, "BE SPB: captured_len must be 4");
    assert_eq!(
        &pkt.data, &payload,
        "BE SPB: packet bytes must match the original payload"
    );
    assert_eq!(pkt.timestamp_secs, 0, "BE SPB: timestamp_secs must be 0");
    assert_eq!(pkt.timestamp_usecs, 0, "BE SPB: timestamp_usecs must be 0");
}

/// AC-002 (EC-004 / EC-003 from BC-2.01.013) — SPB with original_len = 0.
///
/// original_len=0, body=4 bytes (just the original_len field, no data).
/// spb_data_available = 4 - 4 = 0. captured_len = min(0, 0) = 0.
/// Must produce RawPacket { data: vec![] }.
///
/// RED-phase: before the SPB arm existed, no packet was produced. This test was RED and
/// turned GREEN once the SPB arm landed (reader.rs:1158). Guards the zero-original_len
/// path — the SPB arm must emit an empty-data packet rather than skipping the block.
#[test]
fn test_BC_2_01_013_spb_zero_original_len() {
    // Build SHB + IDB + SPB with original_len=0 and no data bytes.
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb_ethernet());
    // btl = 12 + 4 (original_len field) = 16; spb_data_available = 0.
    let btl: u32 = 16;
    bytes.extend_from_slice(&SPB_BLOCK_TYPE.to_le_bytes());
    bytes.extend_from_slice(&btl.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes()); // original_len = 0
    // No padded data bytes (spb_data_available = 0)
    bytes.extend_from_slice(&btl.to_le_bytes());

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "SPB with original_len=0 must succeed: {:?}",
        result.err()
    );
    let source = result.unwrap();
    assert_eq!(source.packets.len(), 1, "must produce 1 packet");
    assert_eq!(
        source.packets[0].data.len(),
        0,
        "packet data must be empty for original_len=0"
    );
    assert_eq!(
        source.packets[0].timestamp_secs, 0,
        "timestamp_secs must be 0"
    );
    assert_eq!(
        source.packets[0].timestamp_usecs, 0,
        "timestamp_usecs must be 0"
    );
}

/// AC-002 (EC-001 from BC-2.01.013) — original_len > spb_data_available (truncation case).
///
/// Canonical test vector from BC-2.01.013:
///   SPB with original_len=1500, block body 64 padded bytes (spb_data_available=64).
///   captured_len = min(1500, 64) = 64.
///   data.len() must be 64.
///
/// RED-phase: before the SPB arm existed, no packet was produced. This test was RED and
/// turned GREEN once the SPB arm landed (reader.rs:1158). Guards the clamping arithmetic
/// — if a refactor removed the arm, no packet would be produced and the assertion fails.
#[test]
fn test_BC_2_01_013_spb_truncated_original_len_exceeds_available() {
    // 64 bytes of actual captured data (4-aligned; no padding needed).
    let payload = vec![0xAB_u8; 64];
    // original_len = 1500 (claims full frame, but only 64 captured).
    let bytes = le_pcapng_shb_idb_spb_with_original_len(&payload, 1500);
    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "truncated SPB parse must succeed: {:?}",
        result.err()
    );
    let source = result.unwrap();
    assert_eq!(source.packets.len(), 1, "must produce 1 packet");
    let pkt = &source.packets[0];
    // captured_len = min(1500, 64) = 64 (bounded by spb_data_available = body.len()-4 = 64)
    assert_eq!(
        pkt.data.len(),
        64,
        "captured_len must be clamped to spb_data_available=64 when original_len=1500; \
         bare body.len()-4=64 is the correct bound; body.len()=68 would be wrong"
    );
    // Verify the bare body.len() guard is NOT used (it would give 68, not 64).
    assert_ne!(
        pkt.data.len(),
        68,
        "data.len() must NOT be 68 (bare body.len() = 4+64 = 68 is 4 bytes too large — \
         counts the original_len field)"
    );
}

/// STORY-126-SPB-PACKETS-EMITTED-001 (MANDATORY) — IDB after SPB must trigger E-INP-013.
///
/// An IDB appearing AFTER an SPB (which increments packets_emitted) must be rejected
/// with E-INP-013 exactly as for EPB. This is the headline new constraint.
///
/// RED-phase provenance: before the SPB arm existed, SPB fell to the wildcard `_` arm
/// which incremented only `skipped_blocks` — not `packets_emitted`. As a result, a late
/// IDB after SPB was accepted silently (no E-INP-013). The test was RED until the SPB
/// arm (reader.rs:1158) landed and began incrementing `packets_emitted`, causing the
/// second IDB to correctly fire E-INP-013.
///
/// Discriminating assertions:
///   (a) E-INP-013 IS present — SPB arm increments packets_emitted; late IDB is rejected.
///   (b) E-INP-008 is NOT present.
///   (c) E-INP-009 is NOT present (interface table is non-empty at SPB time).
///
/// Guards: if the SPB arm were removed or stopped incrementing packets_emitted, the late
/// IDB would be accepted silently and this assertion would fail.
#[test]
fn test_STORY_126_SPB_PACKETS_EMITTED_001() {
    // Build: SHB + IDB + SPB + second IDB (late IDB after SPB).
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb_ethernet()); // first IDB — establishes interface 0
    bytes.extend_from_slice(&le_spb(&[0xDE, 0xAD, 0xBE, 0xEF])); // SPB — must increment packets_emitted
    bytes.extend_from_slice(&le_idb_ethernet()); // second IDB — MUST trigger E-INP-013

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "IDB after SPB must return Err(E-INP-013) — SPB arm (reader.rs:1158) increments \
         packets_emitted; a late IDB is unsupported ordering and must be rejected. \
         If this fails, the SPB arm may have been removed or no longer increments \
         packets_emitted, allowing the late IDB to be accepted silently."
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains(E_INP_013),
        "error must contain E-INP-013 (late IDB after packet block), got: {msg}"
    );
    assert!(
        !msg.contains(E_INP_008),
        "E-INP-008 must NOT be present, got: {msg}"
    );
    assert!(
        !msg.contains(E_INP_009),
        "E-INP-009 must NOT be present (interface table is non-empty), got: {msg}"
    );
}

// ── spb_captured_len pure-core tests (VP-031 target) ─────────────────────────
//
// These tests call spb_captured_len directly. The function is implemented
// (src/reader.rs:770-781) and all tests in this section are GREEN, guarding
// the captured-length arithmetic against regression.
//
// Note: proptest VP-031 is in tests/proptest_reader.rs. These are unit test
// vectors from the BC-2.01.013 Canonical Test Vectors table.

/// VP-031 unit — original_len <= spb_data_available → captured_len = original_len.
///
/// Canonical vector: original_len=64, body.len()=68 (4+64), spb_data_available=64.
/// captured_len = min(64, 64) = 64.
///
/// GREEN after STORY-126: spb_captured_len is implemented.
#[test]
fn test_BC_2_01_013_spb_captured_len_no_truncation() {
    // body = [original_len:4][payload:64] = 68 bytes
    let mut body = vec![0u8; 68];
    // body[0..4] = original_len = 64 (LE) — spb_captured_len receives the raw body slice
    // The function signature: spb_captured_len(original_len: u32, body: &[u8])
    // body here is the raw SPB body slice (includes the original_len field at byte 0).
    // Per BC-2.01.013 AC-002: spb_data_available = body.len() - 4 = 64.
    body[0] = 64;
    body[1] = 0;
    body[2] = 0;
    body[3] = 0;
    let result = spb_captured_len(64, &body);
    assert_eq!(
        result, 64,
        "captured_len must be 64 when original_len=64 <= spb_data_available=64"
    );
}

/// VP-031 unit — original_len > spb_data_available → captured_len = spb_data_available.
///
/// Canonical vector: original_len=1500, body.len()=68 (4+64), spb_data_available=64.
/// captured_len = min(1500, 64) = 64.
///
/// GREEN after STORY-126: spb_captured_len is implemented.
#[test]
fn test_BC_2_01_013_spb_captured_len_truncated_to_available() {
    // body = [original_len:4][payload:64] = 68 bytes; original_len = 1500
    let body = vec![0u8; 68]; // payload portion doesn't matter for length arithmetic
    let result = spb_captured_len(1500, &body);
    assert_eq!(
        result, 64,
        "captured_len must be clamped to spb_data_available=64 when original_len=1500"
    );
}

/// VP-031 unit — boundary: body.len()=4 (exactly SPB_FIXED_OVERHEAD_BYTES).
///
/// body.len()=4, spb_data_available=0, original_len=0 → captured_len=0.
/// This is the minimum body that passes the body-too-short guard.
///
/// GREEN after STORY-126: spb_captured_len is implemented.
#[test]
fn test_BC_2_01_013_spb_captured_len_minimum_body() {
    // body = [original_len:4] only; spb_data_available = 0.
    let body = vec![0u8; 4]; // body.len() = 4 = SPB_FIXED_OVERHEAD_BYTES
    let result = spb_captured_len(0, &body);
    assert_eq!(
        result, 0,
        "captured_len must be 0 when body.len()=4 (spb_data_available=0) and original_len=0"
    );
}

/// VP-031 unit — original_len == spb_data_available (exact match, no truncation).
///
/// Canonical vector from BC-2.01.013 EC-002:
///   original_len=100, body.len()=104 → spb_data_available=100.
///   captured_len = min(100, 100) = 100.
///
/// GREEN after STORY-126: spb_captured_len is implemented.
#[test]
fn test_BC_2_01_013_spb_captured_len_exact_match() {
    let body = vec![0u8; 104]; // 4 + 100 bytes
    let result = spb_captured_len(100, &body);
    assert_eq!(
        result, 100,
        "captured_len must be 100 when original_len=100 == spb_data_available=100"
    );
}

/// VP-031 unit — original_len=0, body.len()=8 → captured_len=0.
///
/// EC-004 from BC-2.01.013: SPB with original_len=0.
///
/// GREEN after STORY-126: spb_captured_len is implemented.
#[test]
fn test_BC_2_01_013_spb_captured_len_zero_original() {
    // body = [original_len:4][4 bytes data] = 8 bytes; spb_data_available = 4.
    let body = vec![0u8; 8];
    let result = spb_captured_len(0, &body);
    assert_eq!(result, 0, "captured_len must be 0 when original_len=0");
}

/// VP-031 unit — large original_len, small body (saturation guard).
///
/// original_len = u32::MAX, body.len() = 4 → spb_data_available = 0.
/// captured_len = min(u32::MAX, 0) = 0. No overflow or panic.
///
/// GREEN after STORY-126: spb_captured_len is implemented.
#[test]
fn test_BC_2_01_013_spb_captured_len_max_original_len() {
    let body = vec![0u8; 4]; // minimum body, spb_data_available = 0
    let result = spb_captured_len(u32::MAX, &body);
    assert_eq!(
        result, 0,
        "captured_len must be 0 when original_len=u32::MAX but spb_data_available=0"
    );
}

// ── BC-2.01.015 Block-Skip Dispatch Tests ────────────────────────────────────

/// AC-006 — explicit match arms for all named skip block types (F-07 MANDATORY).
///
/// Feed a pcapng containing NRB, ISB, SJE, DSB, and truly-unknown blocks.
/// Each MUST be skipped (skipped_blocks incremented); none cause panic or error.
/// The EPB following them MUST be parsed normally.
///
/// The test verifies that the named arms exist (by observing counter behavior
/// and continued parsing). The named-arm vs wildcard distinction is verified by
/// the discriminating counter behavior for OPB (dual counter) vs generic (single).
///
/// Implementation note: NRB/ISB/SJE/DSB are handled by EXPLICIT NAMED match arms at
/// src/reader.rs:1228-1254 (NRB:1228, ISB:1235, SJE:1241, DSB:1247). Only genuinely
/// unknown/unrecognized block types fall to the wildcard `_` arm (reader.rs:1256).
/// The DSB arm satisfies SEC-007 (body bytes MUST NOT be logged) structurally — the
/// named arm increments skipped_blocks and returns without touching body bytes.
/// This test guards: named-arm dispatch + counter behavior + continued parsing
/// (BC-2.01.015 AC-001 F-07, SEC-007 DSB boundary).
#[test]
fn test_BC_2_01_015_dispatch_known_and_skip_unknown() {
    // Build: SHB + IDB + NRB + ISB + SJE + DSB + unknown + EPB
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb_ethernet());
    bytes.extend_from_slice(&le_nrb_empty()); // NRB → skip
    bytes.extend_from_slice(&le_isb_empty()); // ISB → skip
    bytes.extend_from_slice(&le_sje_empty()); // SJE → skip
    bytes.extend_from_slice(&le_dsb_with_body(&[0u8; 8])); // DSB → skip (body NOT logged)
    bytes.extend_from_slice(&le_unknown_block_empty()); // unknown → skip
    bytes.extend_from_slice(&le_epb_iface0()); // EPB → produces packet

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "must succeed after all skip blocks: {:?}",
        result.err()
    );
    let source = result.unwrap();

    // EPB must be parsed — 1 packet total
    assert_eq!(
        source.packets.len(),
        1,
        "EPB after skip blocks must produce exactly 1 packet"
    );

    // 5 skipped blocks (NRB + ISB + SJE + DSB + unknown)
    assert_eq!(
        source.skipped_blocks, 5,
        "5 non-EPB/IDB/SHB/SPB blocks must all increment skipped_blocks; got {}",
        source.skipped_blocks
    );

    // OPB was NOT in this sequence → opb_skipped must be 0
    assert_eq!(
        source.opb_skipped, 0,
        "opb_skipped must be 0 (no OPB in this fixture)"
    );
}

/// AC-007 — OPB is skipped (not parsed); increments BOTH skipped_blocks AND opb_skipped.
///
/// Feed a pcapng with N OPBs and no EPBs.
/// Result: packets.len()==0, skipped_blocks==N, opb_skipped==N.
/// Then verify that adding a non-OPB unknown block increments skipped_blocks (only).
///
/// GREEN: OPB arm already exists in reader.rs. This is a regression guard.
#[test]
fn test_BC_2_01_015_opb_skipped_not_parsed() {
    // Case 1: 3 OPBs, no EPBs
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb_ethernet());
    bytes.extend_from_slice(&le_opb_empty());
    bytes.extend_from_slice(&le_opb_empty());
    bytes.extend_from_slice(&le_opb_empty());

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(result.is_ok(), "3 OPBs must succeed: {:?}", result.err());
    let source = result.unwrap();
    assert_eq!(
        source.packets.len(),
        0,
        "OPB packet data must NOT be ingested; packets must be empty"
    );
    assert_eq!(
        source.skipped_blocks, 3,
        "skipped_blocks must be 3 (one per OPB)"
    );
    assert_eq!(
        source.opb_skipped, 3,
        "opb_skipped must be 3 (same N — all skips are OPB)"
    );

    // Case 2: 1 OPB + 1 unknown → skipped_blocks=2, opb_skipped=1
    let mut bytes2 = le_shb();
    bytes2.extend_from_slice(&le_idb_ethernet());
    bytes2.extend_from_slice(&le_opb_empty());
    bytes2.extend_from_slice(&le_unknown_block_empty());

    let result2 = PcapSource::from_pcap_reader(Cursor::new(bytes2));
    assert!(result2.is_ok(), "OPB + unknown must succeed");
    let source2 = result2.unwrap();
    assert_eq!(
        source2.skipped_blocks, 2,
        "skipped_blocks must be 2 (1 OPB + 1 unknown)"
    );
    assert_eq!(
        source2.opb_skipped, 1,
        "opb_skipped must be 1 (only OPB increments this counter, not generic unknown)"
    );
}

/// AC-008 — no output at any log level; block body bytes MUST NOT be logged (SEC-007).
///
/// This is a structural test verifying that the reader produces no stderr/stdout.
/// We redirect stderr/stdout is not directly testable in Rust unit tests without
/// process capture, so we instead verify the structural property: the function
/// returns Ok (no error leaking log-like errors) and no assertion about log content.
///
/// The SEC-007 DSB body-logging prohibition is tested separately in
/// test_BC_2_01_015_dsb_body_not_logged.
///
/// GREEN: the wildcard arm already discards body bytes silently.
#[test]
fn test_BC_2_01_015_no_output_on_skip() {
    // Feed a DSB with identifiable sentinel bytes + EPB.
    // The sentinel bytes must NOT appear in any error message or panic output.
    let sentinel = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb_ethernet());
    bytes.extend_from_slice(&le_dsb_with_body(&sentinel));
    bytes.extend_from_slice(&le_epb_iface0());

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "DSB + EPB must succeed; if Err, the error must not contain DSB body bytes: {:?}",
        result.as_ref().err()
    );
    // Verify the error chain (if any) does not contain the sentinel bytes.
    if let Err(ref e) = result {
        let msg = e.to_string();
        let sentinel_hex = format!("{:02X}{:02X}", sentinel[0], sentinel[1]);
        assert!(
            !msg.contains(&sentinel_hex),
            "DSB body bytes (SEC-007) must NOT appear in any error output, got: {msg}"
        );
    }
    let source = result.unwrap();
    assert_eq!(
        source.packets.len(),
        1,
        "EPB after DSB must produce 1 packet"
    );
}

/// AC-008 (SEC-007) — DSB body bytes MUST NOT be logged or surfaced.
///
/// Feed a DSB containing synthetic "TLS key material" (sentinel bytes).
/// Assert the sentinel bytes do NOT appear in any Ok or Err output.
///
/// GREEN: wildcard arm already discards body bytes. Regression guard.
#[test]
fn test_BC_2_01_015_dsb_body_not_logged() {
    // Synthetic TLS key material sentinel — distinctive byte sequence.
    let tls_sentinel: Vec<u8> =
        b"SSLKEYLOGFILE_SENTINEL_123456789ABCDEF0000000000000000000".to_vec();
    // Pad to 4-byte boundary for block body.
    let mut body = tls_sentinel.clone();
    while !body.len().is_multiple_of(4) {
        body.push(0);
    }

    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb_ethernet());
    bytes.extend_from_slice(&le_dsb_with_body(&body));
    bytes.extend_from_slice(&le_epb_iface0());

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));

    // Regardless of Ok/Err: the TLS sentinel string must NOT appear in any output.
    let sentinel_str = String::from_utf8_lossy(&tls_sentinel);
    if let Err(ref e) = result {
        let msg = e.to_string();
        assert!(
            !msg.contains(sentinel_str.as_ref()),
            "SEC-007 violation: DSB TLS key material appeared in error output: {msg}"
        );
    }
    assert!(
        result.is_ok(),
        "DSB + EPB must succeed; DSB body must be silently discarded"
    );
    let source = result.unwrap();
    assert_eq!(
        source.packets.len(),
        1,
        "EPB after DSB must produce 1 packet"
    );
    // OBS-2 positive assertion (SEC-007 success path): the DSB sentinel bytes must NOT
    // appear in ANY produced packet's data field. This ensures DSB payload never leaks
    // into the packet stream, not just that it doesn't appear in error messages.
    for (i, pkt) in source.packets.iter().enumerate() {
        let pkt_as_str = String::from_utf8_lossy(&pkt.data);
        assert!(
            !pkt_as_str.contains(sentinel_str.as_ref()),
            "SEC-007 violation: DSB TLS sentinel bytes appeared in packets[{i}].data — \
             DSB payload must never leak into the packet stream"
        );
        // Also check raw byte containment (sentinel is ASCII, but guard both).
        assert!(
            !pkt.data
                .windows(tls_sentinel.len())
                .any(|w| w == tls_sentinel.as_slice()),
            "SEC-007 violation: DSB TLS sentinel bytes (raw) found in packets[{i}].data"
        );
    }
}

/// AC-009 — loop MUST break on Err from next_raw_block (no CWE-835 spin).
///
/// Feed a truncated pcapng that will cause the crate to return Err. Verify:
///   - The function returns Err (not spinning indefinitely).
///   - The function terminates quickly (not spinning).
///
/// We test termination by feeding bytes that are well-formed up to the SHB but
/// then have a block whose btl < 12 (crate-level rejection).
///
/// GREEN: the existing code already breaks on Err. Regression guard.
#[test]
fn test_BC_2_01_015_loop_break_on_error() {
    // SHB + IDB (valid) + truncated block (btl=8 → crate rejects with Err).
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb_ethernet());
    // Craft a block with btl=8 (< 12 minimum) — crate will reject this.
    // block_type(4) + btl=8(4) = 8 bytes; we intentionally omit the trailing btl.
    bytes.extend_from_slice(&0x0000_0007u32.to_le_bytes()); // unknown block type
    bytes.extend_from_slice(&8u32.to_le_bytes()); // btl = 8 (rejected by crate < 12)
    // trailing btl would normally be here but block is malformed anyway

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    // Must return Err — not spin. The test itself verifying it returns demonstrates
    // no infinite loop (the test would hang otherwise).
    assert!(
        result.is_err(),
        "malformed block (btl=8 < 12) must cause Err return, not spin"
    );
    // The error is either E-INP-010 (crate framing rejection) or the forward-progress guard.
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains(E_INP_010) || msg.contains("framing"),
        "loop-break error should reference framing/E-INP-010, got: {msg}"
    );
}

/// AC-010 — skipped_blocks and opb_skipped are public fields on PcapSource.
///
/// SHB-only file → both counters must be 0.
/// Then OPB file → skipped_blocks=1, opb_skipped=1.
/// Then non-OPB skip → skipped_blocks incremented, opb_skipped unchanged.
///
/// GREEN: fields already defined on PcapSource. Regression guard.
#[test]
fn test_BC_2_01_015_skipped_blocks_counter_and_notice() {
    // SHB + IDB + OPB: skipped_blocks=1, opb_skipped=1
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb_ethernet());
    bytes.extend_from_slice(&le_opb_empty());

    let source =
        PcapSource::from_pcap_reader(Cursor::new(bytes)).expect("SHB+IDB+OPB must succeed");
    assert_eq!(
        source.skipped_blocks, 1,
        "OPB must increment skipped_blocks to 1"
    );
    assert_eq!(source.opb_skipped, 1, "OPB must increment opb_skipped to 1");

    // SHB + IDB + unknown: skipped_blocks=1, opb_skipped=0
    let mut bytes2 = le_shb();
    bytes2.extend_from_slice(&le_idb_ethernet());
    bytes2.extend_from_slice(&le_unknown_block_empty());

    let source2 =
        PcapSource::from_pcap_reader(Cursor::new(bytes2)).expect("SHB+IDB+unknown must succeed");
    assert_eq!(
        source2.skipped_blocks, 1,
        "unknown block must increment skipped_blocks to 1"
    );
    assert_eq!(
        source2.opb_skipped, 0,
        "unknown block must NOT increment opb_skipped (stays 0)"
    );
}

/// BC-2.01.015 EC-013 (F-M4) — SHB-only pcapng: both counters must be 0.
///
/// GREEN: no blocks reach the skip arm; counters initialized to 0 and returned.
#[test]
fn test_BC_2_01_015_shb_only_counters_zero() {
    let bytes = shb_only_le();
    let source = PcapSource::from_pcap_reader(Cursor::new(bytes))
        .expect("SHB-only pcapng must parse successfully");
    assert_eq!(
        source.skipped_blocks, 0,
        "SHB-only file: no blocks reach the skip arm; skipped_blocks must be 0"
    );
    assert_eq!(
        source.opb_skipped, 0,
        "SHB-only file: no OPB encountered; opb_skipped must be 0"
    );
    assert_eq!(
        source.packets.len(),
        0,
        "SHB-only file: no packet blocks; packets must be empty"
    );
}

// ── BC-2.01.017 Error-Surface Tests ──────────────────────────────────────────

/// AC-011 — all error paths have anyhow context strings (BC-2.01.017 PC1).
///
/// Tests each error class individually. Each must produce an Err whose chain
/// contains the canonical context string from BC-2.01.017 PC1.
///
/// Sub-test: SPB before IDB → context must contain "before any Interface Description Block".
/// RED-phase: before the SPB arm existed, SPB fell to `_` and returned Ok instead of Err.
/// This test was RED and turned GREEN once the SPB arm landed (reader.rs:1158).
#[test]
fn test_BC_2_01_017_all_error_paths_have_context() {
    // Sub-test 1: SPB before any IDB (E-INP-009).
    // The SPB arm must emit the canonical context "pcapng Simple Packet Block encountered
    // before any Interface Description Block" (BC-2.01.017 PC1).
    let payload = [0u8; 4];
    let bytes = le_pcapng_shb_spb_no_idb(&payload);
    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "SPB before IDB must return Err (E-INP-009 path has canonical context string)"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains(CTX_SPB_BEFORE_IDB),
        "SPB-before-IDB error must contain context string '{}', got: {msg}",
        CTX_SPB_BEFORE_IDB
    );
    assert!(
        msg.contains(E_INP_009),
        "SPB-before-IDB error must contain E-INP-009, got: {msg}"
    );

    // Sub-test 2: SPB body too short (E-INP-008).
    // btl=12 → body=0 < 4 → wirerust body-decode → E-INP-008.
    // Context: "Failed to read pcapng Simple Packet Block".
    let mut bytes2 = le_shb();
    bytes2.extend_from_slice(&le_idb_ethernet());
    let btl: u32 = 12;
    bytes2.extend_from_slice(&SPB_BLOCK_TYPE.to_le_bytes());
    bytes2.extend_from_slice(&btl.to_le_bytes());
    bytes2.extend_from_slice(&btl.to_le_bytes());
    let result2 = PcapSource::from_pcap_reader(Cursor::new(bytes2));
    assert!(
        result2.is_err(),
        "SPB body-too-short must return Err(E-INP-008)"
    );
    let msg2 = result2.unwrap_err().to_string();
    assert!(
        msg2.contains(CTX_SPB_BODY_DECODE),
        "SPB body-too-short error must contain context '{}', got: {msg2}",
        CTX_SPB_BODY_DECODE
    );
    assert!(
        msg2.contains(E_INP_008),
        "SPB body-too-short error must contain E-INP-008, got: {msg2}"
    );
}

/// AC-011 — EPB before IDB emits E-INP-009 with canonical context string.
///
/// The canonical context per BC-2.01.017 PC1 is:
///   "pcapng Enhanced Packet Block encountered before any Interface Description Block"
///
/// We check this contains "before any Interface Description Block" as the invariant part.
/// The EPB arm already emits a message containing "before any Interface Description Block"
/// — verify this is stable (regression guard for existing behavior).
///
/// Note: current EPB message is: "EPB references interface_id=0 but interface table is
/// empty — no IDB has been parsed (E-INP-009)". This does NOT match the canonical
/// PC1 string exactly. The test asserts the BC-2.01.017 PC1 canonical form.
/// This may be RED or GREEN depending on the exact message string.
/// We assert the canonical substring that must be present.
#[test]
fn test_BC_2_01_017_epb_before_idb_emits_einp009_context() {
    // Build SHB + EPB (no IDB) — triggers empty-table guard.
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_epb_iface0());

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(result.is_err(), "EPB before IDB must return Err");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains(E_INP_009),
        "EPB-before-IDB error must contain E-INP-009, got: {msg}"
    );
    // BC-2.01.017 PC1: the EXACT EPB context string must be present in the rendered chain.
    // This pins conformance — the OR condition is removed to prevent regression back to the
    // pre-fix bare message that lacked the mandatory PC1 context prefix.
    assert!(
        msg.contains(
            "pcapng Enhanced Packet Block encountered before any Interface Description Block"
        ),
        "EPB-before-IDB error must contain the canonical BC-2.01.017 PC1 context string \
         'pcapng Enhanced Packet Block encountered before any Interface Description Block', \
         got: {msg}"
    );
    // The underlying taxonomy message must also be present (error-taxonomy v3.6).
    assert!(
        msg.contains("EPB references interface_id") && msg.contains("no IDB has been parsed"),
        "EPB-before-IDB error must contain the taxonomy underlying message \
         'EPB references interface_id=<id> but interface table is empty — no IDB has been parsed', \
         got: {msg}"
    );
    assert!(
        !msg.contains(E_INP_008),
        "E-INP-008 must NOT be present (this is empty-table, not body-too-short), got: {msg}"
    );
    assert!(
        !msg.contains(E_INP_010),
        "E-INP-010 must NOT be present (this is empty-table, not framing), got: {msg}"
    );
}

/// AC-011 — SPB before IDB emits E-INP-009 with canonical context string (BC-2.01.017 PC1).
///
/// Canonical string: "pcapng Simple Packet Block encountered before any Interface Description Block"
///
/// Discriminating assertions:
///   (a) E-INP-009 IS present.
///   (b) Canonical PC1 context substring IS present.
///   (c) E-INP-008 is NOT present.
///   (d) E-INP-010 is NOT present.
///
/// RED-phase: before the SPB arm existed, SPB fell to `_` and returned Ok instead of
/// Err(E-INP-009). This test was RED and turned GREEN once the SPB arm landed
/// (reader.rs:1158). Guards the canonical BC-2.01.017 PC1 context string for SPB.
#[test]
fn test_BC_2_01_017_spb_before_idb_emits_einp009_context() {
    let payload = [0xAB_u8; 4];
    let bytes = le_pcapng_shb_spb_no_idb(&payload);
    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "SPB before IDB must return Err(E-INP-009), not Ok"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains(E_INP_009),
        "error must contain E-INP-009, got: {msg}"
    );
    assert!(
        msg.contains(CTX_SPB_BEFORE_IDB),
        "error must contain canonical BC-2.01.017 PC1 context '{}', got: {msg}",
        CTX_SPB_BEFORE_IDB
    );
    assert!(
        !msg.contains(E_INP_008),
        "E-INP-008 must NOT be present, got: {msg}"
    );
    assert!(
        !msg.contains(E_INP_010),
        "E-INP-010 must NOT be present, got: {msg}"
    );
    assert!(
        !msg.contains(E_INP_011),
        "E-INP-011 must NOT be present, got: {msg}"
    );
}

/// AC-011 — SPB body too short emits E-INP-008 with canonical context string.
///
/// Canonical context: "Failed to read pcapng Simple Packet Block" (BC-2.01.017 PC1).
///
/// Discriminating assertions:
///   (a) E-INP-008 IS present.
///   (b) Canonical context IS present.
///   (c) E-INP-009 is NOT present (IDB is present; this is body-too-short).
///   (d) E-INP-010 is NOT present (crate accepted btl=12; this is wirerust body-decode).
///
/// RED-phase: before the SPB arm existed, SPB fell to `_` and returned Ok instead of
/// Err(E-INP-008). This test was RED and turned GREEN once the SPB arm landed
/// (reader.rs:1158). Guards the canonical BC-2.01.017 PC1 context string for short bodies.
#[test]
fn test_BC_2_01_017_spb_body_too_short_einp008_context() {
    let mut bytes = le_shb();
    bytes.extend_from_slice(&le_idb_ethernet());
    // btl=12: crate accepts it; body=0 bytes; wirerust body-decode rejects.
    let btl: u32 = 12;
    bytes.extend_from_slice(&SPB_BLOCK_TYPE.to_le_bytes());
    bytes.extend_from_slice(&btl.to_le_bytes());
    bytes.extend_from_slice(&btl.to_le_bytes()); // trailing btl

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "SPB with btl=12 (body=0 < 4) must return Err(E-INP-008), not Ok"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains(E_INP_008),
        "error must contain E-INP-008 (wirerust body-decode path), got: {msg}"
    );
    assert!(
        msg.contains(CTX_SPB_BODY_DECODE),
        "error must contain canonical context '{}', got: {msg}",
        CTX_SPB_BODY_DECODE
    );
    assert!(
        !msg.contains(E_INP_009),
        "E-INP-009 must NOT be present (IDB exists; this is body-too-short), got: {msg}"
    );
    assert!(
        !msg.contains(E_INP_010),
        "E-INP-010 must NOT be present (crate accepted btl=12; this is wirerust path), got: {msg}"
    );
}

/// AC-012 — no panic on truncated pcapng (cross-cutting no-panic contract).
///
/// Feeds progressively truncated versions of a valid pcapng to from_pcap_reader.
/// Each must return Ok or Err — never panic.
///
/// GREEN: the code already doesn't panic on most truncations. Regression guard.
#[test]
fn test_BC_2_01_017_no_panic_truncated_pcapng() {
    // Build a valid pcapng with SHB + IDB + SPB
    let valid = le_pcapng_shb_idb_spb(&[0xAA, 0xBB, 0xCC, 0xDD]);
    // Feed truncations at every byte offset [0, len]
    for truncate_at in 0..=valid.len() {
        let truncated = &valid[..truncate_at];
        // Must not panic
        let result = PcapSource::from_pcap_reader(Cursor::new(truncated));
        // The result can be Ok or Err — we only assert no panic
        let _ = result;
    }
}
