//! STORY-123: pcapng Format Detection (Magic-Byte Probe) and SHB Parse
//!
//! TDD test suite — ALL tests in this file MUST FAIL until the implementation
//! replaces the `todo!()` stubs in `src/reader.rs`. This is the Red Gate.
//!
//! Coverage map:
//!   AC-001 → test_BC_2_01_009_unbuffered_read_routes_correctly
//!             test_BC_2_01_009_pipe_stream_probe_observable
//!   AC-002 → test_BC_2_01_009_smb3_pcapng_accepted
//!             test_BC_2_01_009_pcapng_magic_routes_to_pcapng_path
//!   AC-003 → test_BC_2_01_009_classic_pcap_routing_unchanged
//!             test_BC_2_01_009_nanosecond_pcap_routing
//!   AC-004 → test_BC_2_01_009_unrecognized_magic
//!             test_BC_2_01_009_stream_under_4_bytes
//!   AC-005 → test_BC_2_01_010_bom_little_endian
//!             test_BC_2_01_010_bom_big_endian
//!   AC-006 → test_BC_2_01_010_major_version_not_1_rejected
//!   AC-007 → test_BC_2_01_010_shb_body_truncated_e_inp_008
//!             test_BC_2_01_010_shb_framing_rejection_e_inp_010
//!             test_BC_2_01_010_invalid_bom_e_inp_008
//!   AC-008 → test_BC_2_01_010_second_shb_rejected_e_inp_012
//!   AC-009 → test_BC_2_01_010_no_panic_fuzz
//!   AC-010 → test_BC_2_01_009_shb_only_zero_packet_notice
//!   AC-011 → covered by test_BC_2_01_009_smb3_pcapng_accepted (LE) and
//!             test_BC_2_01_010_bom_big_endian (BE)
//!   AC-012 → test_BC_2_01_009_arp_baseline_cap_accepted
//!
//! Additionally rewrites test_BC_2_01_004_rejects_pcapng (which existed as a
//! negative assertion in bc_2_01_story001_tests.rs) to a positive acceptance
//! assertion, per STORY-123 Previous Story Intelligence.
//!
//! Naming convention: `test_BC_S_SS_NNN_<assertion>()` throughout.
//! `#![allow(non_snake_case)]` is required per the factory naming mandate —
//! the BC-prefixed pattern uses uppercase identifiers (BC vs snake_case).
#![allow(non_snake_case)]

use std::io::{BufRead, BufReader, Cursor, Read};
use std::path::Path;

use proptest::prelude::*;
use wirerust::reader::{PcapSource, SectionEndianness, parse_shb_body};

// ── pcapng canonical constants (mirrors ADR-009 / BC-2.01.009 / BC-2.01.010) ─
// These are the ONLY normative byte values tests may use for constructing pcapng
// fixtures. Do not introduce additional magic values not listed in ADR-009.

/// pcapng SHB block_type / file magic: endian-independent 4-byte literal.
const PCAPNG_MAGIC: [u8; 4] = [0x0A, 0x0D, 0x0D, 0x0A];

/// BOM indicating big-endian section (on-disk bytes 1A 2B 3C 4D).
const SHB_BOM_BE: [u8; 4] = [0x1A, 0x2B, 0x3C, 0x4D];

/// BOM indicating little-endian section (on-disk bytes 4D 3C 2B 1A).
const SHB_BOM_LE: [u8; 4] = [0x4D, 0x3C, 0x2B, 0x1A];

/// pcapng SHB block type code (`0x0A0D0D0A` as a u32, same value in both byte orders).
const SHB_BLOCK_TYPE_U32: u32 = 0x0A0D_0D0A;

/// IDB block type code.
const IDB_BLOCK_TYPE_U32: u32 = 0x0000_0001;

/// EPB block type code.
const EPB_BLOCK_TYPE_U32: u32 = 0x0000_0006;

/// SHB body minimum: 16 bytes (BOM:4 + major:2 + minor:2 + section_length:8).
const SHB_BODY_FIXED_BYTES: usize = 16;

// ── Classic-pcap magic bytes (for regression tests) ──────────────────────────
//
// Convention: the pcap-file crate reads the 4 magic bytes as a big-endian u32.
// - On-disk [D4 C3 B2 A1] → BE read = 0xD4C3B2A1 → crate routes to LittleEndian
//   (the pcapng spec calls this the "LE" file; libpcap writes LE files this way).
// - On-disk [A1 B2 C3 D4] → BE read = 0xA1B2C3D4 → crate routes to BigEndian
//   (the pcapng spec calls this the "BE" file).
// - On-disk [D4 C3 B2 A1] is what `0xa1b2c3d4u32.to_le_bytes()` produces.
// - On-disk [A1 B2 C3 D4] is what `0xa1b2c3d4u32.to_be_bytes()` produces.
//
// The constants below use the on-disk byte sequences that produce the
// desired crate routing and which are consistent with existing reader_tests.rs.
const CLASSIC_MAGIC_LE_US: [u8; 4] = [0xD4, 0xC3, 0xB2, 0xA1]; // 0xa1b2c3d4.to_le_bytes()
const CLASSIC_MAGIC_BE_US: [u8; 4] = [0xA1, 0xB2, 0xC3, 0xD4]; // 0xa1b2c3d4.to_be_bytes()
const CLASSIC_MAGIC_LE_NS: [u8; 4] = [0x4D, 0x3C, 0xB2, 0xA1]; // 0xa1b23c4d.to_le_bytes()

// ── Fixture builder helpers ───────────────────────────────────────────────────

/// Build a 28-byte SHB-only pcapng file.
///
/// Structure (28 bytes total = 12-byte outer block header + 16-byte body):
/// - block_type:          4 bytes = 0x0A0D0D0A
/// - block_total_length:  4 bytes = 28 (little-endian)
/// - BOM:                 4 bytes (bom_bytes parameter)
/// - major_version:       2 bytes (little-endian)
/// - minor_version:       2 bytes (little-endian)
/// - section_length:      8 bytes = 0xFFFFFFFFFFFFFFFF (unspecified)
/// - trailing btl:        4 bytes = 28 (little-endian)
///
/// This is the minimal structurally valid SHB per pcapng spec §4.1.
/// The SHB magic (0x0A0D0D0A) occupies bytes 0-3, so the probe sees PCAPNG_MAGIC
/// and the pcapng branch is taken; the BufReader is NOT consumed.
fn shb_only_pcapng(bom_bytes: [u8; 4], major: u16, minor: u16) -> Vec<u8> {
    let mut buf = Vec::with_capacity(28);
    // block_type: SHB
    buf.extend_from_slice(&SHB_BLOCK_TYPE_U32.to_le_bytes());
    // block_total_length: 28
    buf.extend_from_slice(&28u32.to_le_bytes());
    // body (16 bytes):
    buf.extend_from_slice(&bom_bytes); // BOM
    buf.extend_from_slice(&major.to_le_bytes()); // major_version
    buf.extend_from_slice(&minor.to_le_bytes()); // minor_version
    buf.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes()); // section_length (unspecified)
    // trailing block_total_length
    buf.extend_from_slice(&28u32.to_le_bytes());
    assert_eq!(buf.len(), 28, "SHB fixture must be exactly 28 bytes");
    buf
}

/// Build a well-formed LE SHB-only pcapng (28 bytes, major=1, minor=0).
fn minimal_shb_pcapng_le() -> Vec<u8> {
    shb_only_pcapng(SHB_BOM_LE, 1, 0)
}

/// Build a spec-conforming BE SHB-only pcapng (28 bytes, major=1, minor=2).
///
/// In a genuine big-endian pcapng file ALL multi-byte fields — both the outer
/// block framing (block_type, block_total_length) AND the SHB body fields
/// (BOM, major, minor, section_length) — are encoded big-endian. This is the
/// postcondition of BC-2.01.010 PC1 / Invariant 4.
///
/// A fixture with a BE BOM but LE outer framing is MALFORMED for a crate-based
/// reader that correctly reads SHB btl as big-endian first (architect mandate).
///
/// # Fixture correction (architect-sanctioned, STORY-123)
///
/// Prior versions wrote outer framing (block_type, btl, trailing btl) as LE.
/// This was malformed: the pcap-file 2.0.0 crate reads the SHB btl as BE,
/// then swap_bytes() for LE sections. A BE BOM with LE-encoded btl of 28
/// (`1C 00 00 00`) reads as BE = 0x1C000000 = 469762048 → IncompleteBuffer.
/// The fix: write ALL framing fields in genuine big-endian (00 00 00 1C).
///
/// Old bytes (positions 4..8):  1C 00 00 00  (btl=28 LE — malformed for BE file)
/// New bytes (positions 4..8):  00 00 00 1C  (btl=28 BE — spec-conforming)
/// Old bytes (positions 24..28): 1C 00 00 00  (trailing btl LE — malformed)
/// New bytes (positions 24..28): 00 00 00 1C  (trailing btl BE — spec-conforming)
///
/// # Non-palindromic minor_version = 2
///
/// minor_version = 2 is chosen deliberately:
///   - On-disk (BE): `00 02`  → correctly decoded as 2
///   - LE misread:   `02 00`  → decoded as 512 (detects LE misread immediately)
///
/// major_version = 1 encoded BE: `00 01`
///   - LE misread: `01 00` → decoded as 256 ≠ 1 → triggers "unsupported version" error
///
/// Any `parse_shb_body` implementation that reads version fields unconditionally as LE
/// will see major = 256 and return Err("Unsupported pcapng major version: 256") — RED.
/// Only a correct BE-aware decode produces (major=1, minor=2) — GREEN after fix.
fn minimal_shb_pcapng_be() -> Vec<u8> {
    let mut buf = Vec::with_capacity(28);
    // Outer block framing: ALL fields big-endian (genuine BE file per pcapng spec).
    // block_type 0x0A0D0D0A is endian-independent (palindromic in bytes [0A 0D 0D 0A]).
    buf.extend_from_slice(&SHB_BLOCK_TYPE_U32.to_be_bytes()); // 0A 0D 0D 0A
    buf.extend_from_slice(&28u32.to_be_bytes()); // 00 00 00 1C (btl=28 BE)
    // SHB body — ALL fields big-endian (spec-conforming BE section):
    buf.extend_from_slice(&SHB_BOM_BE); // 1A 2B 3C 4D (big-endian BOM)
    buf.extend_from_slice(&1u16.to_be_bytes()); // 00 01 (major=1 BE; LE misread = 256)
    buf.extend_from_slice(&2u16.to_be_bytes()); // 00 02 (minor=2 BE; LE misread = 512)
    buf.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_be_bytes()); // FF FF FF FF FF FF FF FF
    // Trailing btl — big-endian (matching outer framing).
    buf.extend_from_slice(&28u32.to_be_bytes()); // 00 00 00 1C (trailing btl=28 BE)
    assert_eq!(buf.len(), 28, "BE SHB fixture must be exactly 28 bytes");
    buf
}

/// Build a minimal classic-pcap global header (24 bytes, LE microsecond).
fn classic_pcap_header_le_ethernet() -> Vec<u8> {
    let mut buf = Vec::with_capacity(24);
    buf.extend_from_slice(&CLASSIC_MAGIC_LE_US); // magic
    buf.extend_from_slice(&2u16.to_le_bytes()); // version major
    buf.extend_from_slice(&4u16.to_le_bytes()); // version minor
    buf.extend_from_slice(&0i32.to_le_bytes()); // thiszone
    buf.extend_from_slice(&0u32.to_le_bytes()); // sigfigs
    buf.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
    buf.extend_from_slice(&1u32.to_le_bytes()); // network (Ethernet)
    buf
}

/// Build a minimal classic nanosecond-resolution pcap header (24 bytes, LE ns).
fn classic_pcap_header_le_ns_ethernet() -> Vec<u8> {
    let mut buf = Vec::with_capacity(24);
    buf.extend_from_slice(&CLASSIC_MAGIC_LE_NS); // ns magic
    buf.extend_from_slice(&2u16.to_le_bytes());
    buf.extend_from_slice(&4u16.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&65535u32.to_le_bytes());
    buf.extend_from_slice(&1u32.to_le_bytes());
    buf
}

/// Build a pcapng file with 2 sections: SHB₁ + IDB + SHB₂ (no EPB between).
///
/// Structure (canonical fixture for AC-008 / BC-2.01.010 AC-002 EC-006):
///   SHB₁ (28 bytes, LE, major=1)
///   IDB  (20 bytes, LE, linktype=1 Ethernet, snaplen=65535)
///   SHB₂ (28 bytes, LE, major=1)  ← second SHB triggers E-INP-012
///
/// The IDB is included before SHB₂ to reflect the "realistic" 2-section layout
/// described in BC-2.01.010 AC-002 canonical fixture (SHB₁ + IDB + EPB + SHB₂).
/// For the purpose of triggering the rejection it is sufficient to have any block
/// before SHB₂; we use an IDB to keep the fixture structurally valid up to SHB₂.
fn two_section_pcapng() -> Vec<u8> {
    let mut buf = Vec::new();

    // Section 1 SHB (28 bytes, LE)
    buf.extend_from_slice(&SHB_BLOCK_TYPE_U32.to_le_bytes()); // block_type
    buf.extend_from_slice(&28u32.to_le_bytes()); // block_total_length
    buf.extend_from_slice(&SHB_BOM_LE); // BOM: little-endian
    buf.extend_from_slice(&1u16.to_le_bytes()); // major_version = 1
    buf.extend_from_slice(&0u16.to_le_bytes()); // minor_version = 0
    buf.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes()); // section_length
    buf.extend_from_slice(&28u32.to_le_bytes()); // trailing btl

    // IDB (Interface Description Block, 20 bytes, LE)
    // block_type=0x00000001, btl=20, body=8 bytes (linktype:2 + reserved:2 + snaplen:4)
    buf.extend_from_slice(&IDB_BLOCK_TYPE_U32.to_le_bytes()); // block_type
    buf.extend_from_slice(&20u32.to_le_bytes()); // block_total_length = 20
    buf.extend_from_slice(&1u16.to_le_bytes()); // linktype = 1 (ETHERNET)
    buf.extend_from_slice(&0u16.to_le_bytes()); // reserved
    buf.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
    buf.extend_from_slice(&20u32.to_le_bytes()); // trailing btl

    // Section 2 SHB — this triggers E-INP-012 (second SHB rejected).
    buf.extend_from_slice(&SHB_BLOCK_TYPE_U32.to_le_bytes()); // block_type
    buf.extend_from_slice(&28u32.to_le_bytes()); // block_total_length
    buf.extend_from_slice(&SHB_BOM_LE); // BOM
    buf.extend_from_slice(&1u16.to_le_bytes()); // major_version = 1
    buf.extend_from_slice(&0u16.to_le_bytes()); // minor_version = 0
    buf.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes()); // section_length
    buf.extend_from_slice(&28u32.to_le_bytes()); // trailing btl

    buf
}

/// Build an SHB with a truncated body (btl=16 → body=4 bytes, < 16 SHB fixed-field bytes).
///
/// AC-007 case (b): crate parses the block outer frame (btl=16, body=4 bytes), then
/// wirerust body-decode finds body < 16 SHB_BODY_FIXED_BYTES → E-INP-008 (body-too-short).
///
/// Structure: block_type + btl(16 LE) + 4 body bytes + trailing btl(16 LE).
/// Total = 4 + 4 + 4 + 4 = 16 bytes.
///
/// # Fixture correction (architect-sanctioned, STORY-123)
///
/// Prior version put [1A 2B 3C 4D] (BE BOM bytes) as the 4-byte body in an
/// otherwise LE-framed SHB. This is endianness-inconsistent: the crate reads
/// the SHB btl as BE first → 0x10000000 (not 16) → IncompleteBuffer → E-INP-010,
/// masking the intended E-INP-008 body-too-short signal.
///
/// Fix: use LE BOM bytes [4D 3C 2B 1A] as the 4-byte body, consistent with a
/// LE-outer-framed SHB. The crate now correctly reads btl=16 LE (via swap_bytes
/// after seeing LE BOM), yields body=[4D 3C 2B 1A] (4 bytes), and passes the
/// block to wirerust. Wirerust checks body.len()=4 < 16 → E-INP-008.
///
/// Old body bytes: 1A 2B 3C 4D  (BE BOM — inconsistent with LE outer framing)
/// New body bytes: 4D 3C 2B 1A  (LE BOM — consistent with LE outer framing)
fn shb_body_truncated_btl16() -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&SHB_BLOCK_TYPE_U32.to_le_bytes()); // block_type
    buf.extend_from_slice(&16u32.to_le_bytes()); // block_total_length = 16
    // body = btl - 12 = 16 - 12 = 4 bytes (LE BOM bytes — consistent with LE outer framing)
    buf.extend_from_slice(&[0x4D, 0x3C, 0x2B, 0x1A]); // only 4 body bytes (LE BOM)
    buf.extend_from_slice(&16u32.to_le_bytes()); // trailing btl
    assert_eq!(buf.len(), 16);
    buf
}

/// Build an SHB with btl=8 (crate framing rejection → E-INP-010).
///
/// AC-007 case (a): btl < 12 → crate rejects BEFORE returning block.
/// wirerust never sees the body. E-INP-010.
///
/// Note: because the crate reads btl from the first 8 bytes and then rejects
/// btl=8 < 12, the "body" content after the btl field is irrelevant; we just
/// need enough bytes for the crate to read the header and see btl=8.
fn shb_framing_btl8() -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&SHB_BLOCK_TYPE_U32.to_le_bytes()); // block_type
    buf.extend_from_slice(&8u32.to_le_bytes()); // btl = 8 (< 12 → crate rejects)
    // No body bytes needed (crate rejects before consuming body)
    // Include trailing btl for completeness even though crate will reject first.
    buf.extend_from_slice(&8u32.to_le_bytes());
    buf
}

/// Build a well-formed SHB (btl=28) but with an invalid BOM (`DE AD BE EF`).
///
/// AC-007 case (c): btl ≥ 28, body ≥ 16, but BOM bytes match neither row of
/// the canonical BOM table → E-INP-008 (invalid BOM).
///
/// The canonical BOM table (BC-2.01.010 PC1):
///   1A 2B 3C 4D → big-endian
///   4D 3C 2B 1A → little-endian
///   any other    → E-INP-008
fn shb_with_invalid_bom() -> Vec<u8> {
    shb_only_pcapng([0xDE, 0xAD, 0xBE, 0xEF], 1, 0)
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-012 Fixture Discovery and arp-baseline-16pkt.cap reconciliation
// ──────────────────────────────────────────────────────────────────────────────
//
// F3 FOLLOW-UP ITEM: arp-baseline-16pkt.cap fixture reconciliation
//
// The story (AC-012) specifies that `tests/fixtures/arp-baseline-16pkt.cap`
// is a "pcapng file with a .cap extension" that must yield Ok(PcapSource)
// with 16 packets.
//
// FINDING: The file `tests/fixtures/arp-baseline-16pkt.cap` does NOT EXIST
// in the fixture directory at the time of this test suite authorship. The
// fixture directory contains: dns.cap, http-full.cap, nfs_bad_stalls.cap,
// teardrop.cap, v6-http.cap (all classic .cap files) and smb3.pcapng (pcapng).
// There is no arp-baseline-16pkt.cap file.
//
// ADR-009 Decision 11 lists `arp-baseline-16pkt.cap` as the ADR's lead motivator
// file ("captured on PacketLife") and specifies it is stored as pcapng with a
// .cap extension. It was NOT present in the fixture directory when this test suite
// was authored.
//
// DECISION (Test Writer): Rather than silently omit the AC-012 test or create a
// synthetic fixture that may not match the intended capture, this test suite:
//   (a) Creates a synthetic minimal pcapng fixture with exactly 16 EPB packets
//       at `tests/fixtures/arp-baseline-16pkt.cap`, programmatically generated
//       using canonical constants. This fixture satisfies AC-012's 16-packet
//       assertion and enables Red Gate verification.
//   (b) Documents this as a FIXTURE SUBSTITUTION that the implementer and PO
//       must replace with the authentic PacketLife fixture before Phase-4
//       holdout evaluation (HS-103 etc.). The synthetic fixture has .cap
//       extension and starts with PCAPNG_MAGIC, which is the AC-012 proof
//       scenario: magic-byte probe routes by content not extension.
//   (c) The test `test_BC_2_01_009_arp_baseline_cap_accepted` uses
//       `tests/fixtures/arp-baseline-16pkt.cap` via `PcapSource::from_file`;
//       the from_file → from_pcap_reader → magic-probe routing is the
//       behavior under test (AC-012, BC-2.01.009 PC5 / ADR-009 Decision 11).
//
// This reconciliation must be recorded as a DONE_WITH_CONCERNS item:
// the fixture substitution is explicit and logged here, not silently fudged.

/// Build a minimal pcapng file with exactly `n` valid EPB packets.
///
/// Layout: SHB(28) + IDB(20) + n×EPB(32 each, with 4-byte dummy payload + 0 pad)
/// Each EPB has a minimal 4-byte Ethernet-style payload (not a real frame, but
/// sufficient for the reader to count it as a packet via captured_len > 0).
///
/// EPB structure (body = 20 + captured_data + pad; btl = 12 + body):
///   interface_id:   4 bytes = 0
///   ts_high:        4 bytes = 1 (arbitrary)
///   ts_low:         4 bytes = 0
///   captured_len:   4 bytes = 4
///   original_len:   4 bytes = 4
///   packet_data:    4 bytes (captured_len = 4, pad = 0 because 4%4==0)
/// body = 20 + 4 = 24 bytes; btl = 12 + 24 = 36 bytes
fn minimal_pcapng_with_n_packets(n: u32) -> Vec<u8> {
    let mut buf = Vec::new();

    // SHB (28 bytes, LE, major=1, minor=0)
    buf.extend_from_slice(&SHB_BLOCK_TYPE_U32.to_le_bytes());
    buf.extend_from_slice(&28u32.to_le_bytes());
    buf.extend_from_slice(&SHB_BOM_LE);
    buf.extend_from_slice(&1u16.to_le_bytes()); // major
    buf.extend_from_slice(&0u16.to_le_bytes()); // minor
    buf.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes());
    buf.extend_from_slice(&28u32.to_le_bytes());

    // IDB (20 bytes, LE, ETHERNET linktype=1)
    buf.extend_from_slice(&IDB_BLOCK_TYPE_U32.to_le_bytes());
    buf.extend_from_slice(&20u32.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes()); // linktype = ETHERNET
    buf.extend_from_slice(&0u16.to_le_bytes()); // reserved
    buf.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
    buf.extend_from_slice(&20u32.to_le_bytes());

    // EPB blocks (36 bytes each)
    // btl = 12 (outer) + 20 (EPB fixed) + 4 (packet data) = 36
    let epb_btl: u32 = 36;
    for i in 0..n {
        buf.extend_from_slice(&EPB_BLOCK_TYPE_U32.to_le_bytes());
        buf.extend_from_slice(&epb_btl.to_le_bytes()); // btl = 36
        buf.extend_from_slice(&0u32.to_le_bytes()); // interface_id = 0
        buf.extend_from_slice(&1u32.to_le_bytes()); // ts_high = 1
        buf.extend_from_slice(&i.to_le_bytes()); // ts_low = packet index
        buf.extend_from_slice(&4u32.to_le_bytes()); // captured_len = 4
        buf.extend_from_slice(&4u32.to_le_bytes()); // original_len = 4
        buf.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // 4-byte payload (4%4=0, no pad)
        buf.extend_from_slice(&epb_btl.to_le_bytes()); // trailing btl
    }

    buf
}

/// Build a GENUINE big-endian pcapng file where ALL multi-byte framing fields
/// are encoded big-endian.
///
/// # Big-endian pcapng wire format (ADR-009 / BC-2.01.010 / pcapng spec §4.1)
///
/// In a genuine BE pcapng file, the SHB BOM field on-disk is `1A 2B 3C 4D`.
/// Per the pcapng spec, this BOM ALSO governs the outer block_type and
/// block_total_length framing fields of the SHB itself and all subsequent
/// blocks. An implementation that reads outer framing fields as LE (always)
/// will misinterpret them and either fail with a "stream too short" error or
/// decode the wrong block boundary.
///
/// # Non-palindromic multi-byte values used (why they detect LE-misreads)
///
/// | Field                         | BE bytes          | Logical value | LE-misread value             |
/// |-------------------------------|-------------------|---------------|------------------------------|
/// | SHB block_total_length        | 00 00 00 1C = 28  | 28            | 1C 00 00 00 = 469762048      |
/// | IDB block_total_length        | 00 00 00 14 = 20  | 20            | 14 00 00 00 = 335544320      |
/// | IDB linktype                  | 00 01 = 1         | 1 (ETHERNET)  | 01 00 = 256 (wrong type)     |
/// | EPB block_total_length        | 00 00 00 24 = 36  | 36            | 24 00 00 00 = 603979776      |
/// | EPB captured_len              | 00 00 00 04 = 4   | 4             | 04 00 00 00 = 67108864 (OOB) |
///
/// A LE-always reader will encounter btl=469762048 for the SHB and immediately
/// fail (stream too short). If a future LE-reader skips the SHB check, the IDB
/// btl=335544320 fails next. The test therefore asserts that the correct result
/// is `Ok(PcapSource)` with exactly 1 packet containing `[BE EF CA FE]` data
/// and `datalink == ETHERNET` — conditions that are all wrong under LE misread.
///
/// # Structure (all framing fields big-endian)
///
/// ```text
/// SHB  (28 bytes BE):
///   block_type:         0A 0D 0D 0A  (endian-independent magic)
///   block_total_length: 00 00 00 1C  (= 28 BE)
///   BOM:                1A 2B 3C 4D  (big-endian section)
///   major_version:      00 01        (= 1 BE)
///   minor_version:      00 00        (= 0 BE)
///   section_length:     FF FF FF FF FF FF FF FF (unspecified)
///   trailing btl:       00 00 00 1C  (= 28 BE)
///
/// IDB  (20 bytes BE):
///   block_type:         00 00 00 01  (IDB type)
///   block_total_length: 00 00 00 14  (= 20 BE)
///   linktype:           00 01        (= 1 ETHERNET, BE)
///   reserved:           00 00
///   snaplen:            00 01 00 00  (= 65536 BE, non-zero, non-palindromic)
///   trailing btl:       00 00 00 14  (= 20 BE)
///
/// EPB  (36 bytes BE):
///   block_type:         00 00 00 06  (EPB type)
///   block_total_length: 00 00 00 24  (= 36 BE)
///   interface_id:       00 00 00 00  (= 0 BE)
///   ts_high:            00 00 00 01  (= 1 BE)
///   ts_low:             00 00 00 00  (= 0 BE)
///   captured_len:       00 00 00 04  (= 4 BE)
///   original_len:       00 00 00 04  (= 4 BE)
///   packet_data:        BE EF CA FE  (4 bytes, 4%4=0 no pad)
///   trailing btl:       00 00 00 24  (= 36 BE)
/// ```
///
/// FIXTURE-HYGIENE NOTE: this function returns a `Vec<u8>` built in-memory.
/// It does NOT write to the source tree.
fn genuine_be_pcapng_with_one_packet() -> Vec<u8> {
    let mut buf = Vec::new();

    // ── SHB (28 bytes, ALL fields big-endian) ────────────────────────────────
    // block_type: SHB magic (0x0A0D0D0A) — endian-independent 4-byte literal
    buf.extend_from_slice(&SHB_BLOCK_TYPE_U32.to_be_bytes()); // 0A 0D 0D 0A (same in both)
    // block_total_length = 28 in BE
    buf.extend_from_slice(&28u32.to_be_bytes()); // 00 00 00 1C
    // BOM: big-endian section (on-disk 1A 2B 3C 4D per BC-2.01.010 PC1)
    buf.extend_from_slice(&SHB_BOM_BE); // 1A 2B 3C 4D
    // major_version = 1 in BE
    buf.extend_from_slice(&1u16.to_be_bytes()); // 00 01
    // minor_version = 0 in BE
    buf.extend_from_slice(&0u16.to_be_bytes()); // 00 00
    // section_length = unspecified (all 0xFF)
    buf.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_be_bytes()); // FF FF FF FF FF FF FF FF
    // trailing block_total_length = 28 in BE
    buf.extend_from_slice(&28u32.to_be_bytes()); // 00 00 00 1C
    assert_eq!(buf.len(), 28, "SHB must be 28 bytes");

    // ── IDB (20 bytes, ALL fields big-endian) ────────────────────────────────
    // block_type = 1 (IDB) in BE: 00 00 00 01 (misread as LE → 0x01000000 = 16777216)
    buf.extend_from_slice(&IDB_BLOCK_TYPE_U32.to_be_bytes()); // 00 00 00 01
    // block_total_length = 20 in BE: 00 00 00 14 (misread as LE → 335544320)
    buf.extend_from_slice(&20u32.to_be_bytes()); // 00 00 00 14
    // linktype = 1 (ETHERNET) in BE: 00 01 (misread as LE → 256 = wrong type)
    buf.extend_from_slice(&1u16.to_be_bytes()); // 00 01
    // reserved = 0
    buf.extend_from_slice(&0u16.to_be_bytes()); // 00 00
    // snaplen = 65536 in BE: 00 01 00 00 (non-palindromic)
    buf.extend_from_slice(&65536u32.to_be_bytes()); // 00 01 00 00
    // trailing block_total_length = 20 in BE
    buf.extend_from_slice(&20u32.to_be_bytes()); // 00 00 00 14
    assert_eq!(buf.len(), 48, "SHB(28) + IDB(20) = 48 bytes");

    // ── EPB (36 bytes, ALL fields big-endian) ────────────────────────────────
    // EPB layout: outer(12) + interface_id(4) + ts_high(4) + ts_low(4) +
    //             captured_len(4) + original_len(4) + packet_data(4) = 36 bytes
    // block_type = 6 (EPB) in BE: 00 00 00 06 (misread as LE → 0x06000000)
    buf.extend_from_slice(&EPB_BLOCK_TYPE_U32.to_be_bytes()); // 00 00 00 06
    // block_total_length = 36 in BE: 00 00 00 24 (misread as LE → 603979776)
    buf.extend_from_slice(&36u32.to_be_bytes()); // 00 00 00 24
    // interface_id = 0 in BE
    buf.extend_from_slice(&0u32.to_be_bytes()); // 00 00 00 00
    // ts_high = 1 in BE: 00 00 00 01 (misread as LE → 0x01000000)
    buf.extend_from_slice(&1u32.to_be_bytes()); // 00 00 00 01
    // ts_low = 0 in BE
    buf.extend_from_slice(&0u32.to_be_bytes()); // 00 00 00 00
    // captured_len = 4 in BE: 00 00 00 04 (misread as LE → 0x04000000 = OOB!)
    buf.extend_from_slice(&4u32.to_be_bytes()); // 00 00 00 04
    // original_len = 4 in BE
    buf.extend_from_slice(&4u32.to_be_bytes()); // 00 00 00 04
    // packet_data: 4 bytes, non-palindromic sentinel value
    buf.extend_from_slice(&[0xBE, 0xEF, 0xCA, 0xFE]); // distinctive payload
    // trailing block_total_length = 36 in BE
    buf.extend_from_slice(&36u32.to_be_bytes()); // 00 00 00 24
    assert_eq!(buf.len(), 84, "SHB(28) + IDB(20) + EPB(36) = 84 bytes");

    buf
}

/// Build a pcapng file with an EPB that appears BEFORE any IDB.
///
/// This is the E-INP-009 trigger: EPB/SPB before the interface table is populated.
/// Structure: SHB (28 bytes LE) + EPB (36 bytes LE, interface_id=0) — NO IDB.
///
/// Per BC-2.01.009 PC3 / ADR-009 Decision 20 Error-Code table:
///   "EPB/SPB arrives with empty interface table (no IDB parsed yet) → E-INP-009"
///
/// The EPB here is structurally valid (btl=36, body ≥ 20), so E-INP-008 body-too-short
/// is ruled out; the failure must be E-INP-009 (no IDB parsed yet).
fn pcapng_epb_before_idb() -> Vec<u8> {
    let mut buf = Vec::new();

    // SHB (28 bytes, LE)
    buf.extend_from_slice(&SHB_BLOCK_TYPE_U32.to_le_bytes());
    buf.extend_from_slice(&28u32.to_le_bytes());
    buf.extend_from_slice(&SHB_BOM_LE);
    buf.extend_from_slice(&1u16.to_le_bytes()); // major = 1
    buf.extend_from_slice(&0u16.to_le_bytes()); // minor = 0
    buf.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes());
    buf.extend_from_slice(&28u32.to_le_bytes());

    // EPB (36 bytes, LE) — NO IDB precedes this.
    // btl = 36: outer(12) + EPB_fixed(20) + packet_data(4) = 36
    let epb_btl: u32 = 36;
    buf.extend_from_slice(&EPB_BLOCK_TYPE_U32.to_le_bytes()); // block_type = 6
    buf.extend_from_slice(&epb_btl.to_le_bytes()); // btl = 36
    buf.extend_from_slice(&0u32.to_le_bytes()); // interface_id = 0
    buf.extend_from_slice(&1u32.to_le_bytes()); // ts_high = 1
    buf.extend_from_slice(&0u32.to_le_bytes()); // ts_low = 0
    buf.extend_from_slice(&4u32.to_le_bytes()); // captured_len = 4
    buf.extend_from_slice(&4u32.to_le_bytes()); // original_len = 4
    buf.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // packet_data (4 bytes, no pad)
    buf.extend_from_slice(&epb_btl.to_le_bytes()); // trailing btl

    buf
}

/// Create the arp-baseline-16pkt.cap fixture in the system temp directory.
///
/// # M-1 FIXTURE-HYGIENE FIX
///
/// The ORIGINAL implementation wrote the synthetic fixture to
/// `tests/fixtures/arp-baseline-16pkt.cap` (inside the source tree) using
/// `std::fs::write`. This is a test-hygiene defect: tests MUST NOT mutate
/// the source tree at runtime. Multiple parallel test runs will race on
/// the same path; CI workers may have read-only source trees.
///
/// This replacement writes to `std::env::temp_dir()` instead, which is
/// always writable, isolated per-process by a unique prefix, and cleaned
/// up by the OS.
///
/// # FIXTURE-SUBSTITUTION NOTE (AC-012 follow-up required before Phase-4)
///
/// This function produces a SYNTHETIC 16-packet pcapng fixture, NOT the
/// authentic PacketLife `arp-baseline-16pkt.cap` capture referenced in
/// ADR-009 Decision 11. The synthetic fixture satisfies AC-012's 16-packet
/// count assertion and proves content-based routing (.cap extension, pcapng
/// magic), but it does not contain real ARP traffic.
///
/// BEFORE Phase-4 holdout evaluation (HS-103 and related scenarios), the
/// implementer or PO MUST replace the synthetic bytes with the authentic
/// PacketLife capture so the holdout tests exercise real packet content.
/// Track this as a follow-up item: "Replace synthetic arp-baseline-16pkt.cap
/// with the authentic PacketLife capture before Phase-4 holdout."
fn ensure_arp_baseline_fixture() -> std::path::PathBuf {
    // Write to the OS temp directory, NOT the source tree.
    // Use a deterministic name within the temp dir so the same test run reuses it,
    // but different processes get different paths via the temp dir's own isolation.
    let mut path = std::env::temp_dir();
    path.push("wirerust-test-arp-baseline-16pkt.cap");

    // Write (or overwrite) the synthetic fixture.
    // This is idempotent: writing the same bytes again is harmless.
    let bytes = minimal_pcapng_with_n_packets(16);
    std::fs::write(&path, &bytes).expect(
        "should be able to write synthetic arp-baseline-16pkt.cap to temp dir \
         (M-1 hygiene fix: writes to temp_dir() not tests/fixtures/)",
    );
    path
}

// ──────────────────────────────────────────────────────────────────────────────
// PART 1: Rewrite of test_BC_2_01_004_rejects_pcapng (BC-2.01.004 superseded)
//
// BC-2.01.004 was RETIRED in F2 spec evolution; its postcondition inverted.
// This test existed in bc_2_01_story001_tests.rs as a NEGATIVE assertion.
// STORY-123 Previous Story Intelligence: the test must become a POSITIVE
// acceptance assertion. The original negative test should be removed from
// bc_2_01_story001_tests.rs by the implementer as part of the PR.
//
// This test IS named per the old BC (BC-2.01.004) because we are REWRITING it
// (not adding a new one). The new intent is the inverse of the old BC.
// ──────────────────────────────────────────────────────────────────────────────

/// Rewrite of test_BC_2_01_004_rejects_pcapng → now a positive acceptance test.
///
/// BC-2.01.009 PC1 + AC-002: smb3.pcapng MUST return Ok(PcapSource).
/// This directly inverts the former BC-2.01.004 postcondition (rejection → acceptance).
/// The original test asserted `result.is_err()` containing "Failed to parse pcap header";
/// this replacement asserts `result.is_ok()` and correct packet count.
///
/// Covers: AC-002, AC-011 (LE pcapng → endian-independent magic detection).
#[test]
fn test_BC_2_01_004_pcapng_accepted_positive_rewrite() {
    let path = Path::new("tests/fixtures/smb3.pcapng");
    assert!(path.exists(), "smb3.pcapng fixture must exist");

    let result = PcapSource::from_file(path);
    assert!(
        result.is_ok(),
        "smb3.pcapng must now return Ok(PcapSource) — BC-2.01.009 inverts BC-2.01.004; got: {:?}",
        result.unwrap_err()
    );
    let source = result.unwrap();
    // smb3.pcapng contains EPBs; packets must not be empty for this specific fixture
    // (BC-2.01.009 canonical test vector — fixture-specific assertion).
    assert!(
        !source.packets.is_empty(),
        "smb3.pcapng must contain at least 1 packet; got 0"
    );
    // The old error chain "Failed to parse pcap header" must NOT appear.
    // (No assertion needed here — is_ok() above already proves it.)
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-001: Probe consumes no bytes (BC-2.01.009 PC3 / Invariant 1 / AC-007)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-001 / BC-2.01.009 AC-007: from_pcap_reader must work with an unbuffered
/// Cursor<&[u8]>, proving the internal BufReader wrap is present.
///
/// If the BufReader wrap is absent, fill_buf() is unavailable and the probe
/// cannot be performed without consuming bytes; the call would either panic
/// or misroute. The test also verifies the pcapng branch is taken (Ok result),
/// not the classic-pcap or error branch.
#[test]
fn test_BC_2_01_009_unbuffered_read_routes_correctly() {
    // Pass an unbuffered `Cursor<Vec<u8>>` (not a BufReader) directly.
    // AC-007 (M-4): from_pcap_reader wraps it internally.
    let bytes = minimal_shb_pcapng_le();
    let cursor = Cursor::new(bytes); // unbuffered — no BufRead impl
    let result = PcapSource::from_pcap_reader(cursor);
    assert!(
        result.is_ok(),
        "unbuffered Cursor with valid pcapng SHB must return Ok (proves internal BufReader wrap); got: {:?}",
        result.err()
    );
    let source = result.unwrap();
    // SHB-only file → zero packets (AC-010)
    assert_eq!(
        source.packets.len(),
        0,
        "SHB-only pcapng must yield 0 packets"
    );
    // skipped_blocks and opb_skipped must both be 0 for SHB-only file.
    assert_eq!(
        source.skipped_blocks, 0,
        "SHB-only file: skipped_blocks must be 0"
    );
    assert_eq!(
        source.opb_skipped, 0,
        "SHB-only file: opb_skipped must be 0"
    );
}

/// AC-001 / BC-2.01.009 PC3 / Invariant 1: the probe peeks 4 bytes without
/// consuming them — the next read on the BufReader returns the byte at offset 0.
///
/// Observable invariant (I-12): after the probe, `fill_buf()[0]` still equals
/// the byte at offset 0 of the original input. Works on non-seekable streams
/// (Cursor is used here; fill_buf semantics are the same).
///
/// We exercise this by wrapping the input ourselves in a BufReader (the same
/// type from_pcap_reader uses internally) and calling fill_buf before and after
/// constructing a PcapSource. Since from_pcap_reader takes ownership, we use
/// a separate BufReader to observe the invariant from the outside:
/// the first 4 bytes of the pcapng data must remain observable as the magic.
#[test]
fn test_BC_2_01_009_pipe_stream_probe_observable() {
    // Build a minimal SHB-only pcapng (28 bytes, LE).
    let bytes = minimal_shb_pcapng_le();

    // A custom reader that records the bytes read after the probe.
    // We use a Cursor here because Cursor<Vec<u8>>: Read but not BufRead,
    // so from_pcap_reader must wrap it in BufReader. After from_pcap_reader
    // returns, we cannot observe the internal BufReader's state directly.
    //
    // Instead we verify the observable invariant indirectly: from_pcap_reader
    // with the pcapng bytes returns Ok AND the pcapng branch is taken (not the
    // classic branch, which would require byte-0 to be 0xA1 not 0x0A).
    // The pcapng branch being taken proves byte-0 was still 0x0A after the probe.
    let cursor = Cursor::new(bytes.clone());
    let result = PcapSource::from_pcap_reader(cursor);
    assert!(
        result.is_ok(),
        "probe-observable invariant test: pcapng SHB must route correctly; got: {:?}",
        result.err()
    );

    // Direct observability: build a BufReader and verify the first byte is
    // still 0x0A after calling fill_buf (simulating what from_pcap_reader does).
    let mut br = BufReader::new(Cursor::new(bytes.clone()));
    let buf = br.fill_buf().expect("fill_buf must succeed");
    assert_eq!(
        buf[0], 0x0A,
        "byte-0 of pcapng stream is 0x0A (first byte of PCAPNG_MAGIC)"
    );
    assert_eq!(buf[1], 0x0D, "byte-1 of pcapng stream is 0x0D");
    assert_eq!(buf[2], 0x0D, "byte-2 of pcapng stream is 0x0D");
    assert_eq!(buf[3], 0x0A, "byte-3 of pcapng stream is 0x0A");
    // fill_buf did NOT advance the reader — we did NOT call consume().
    // A second fill_buf must return the same bytes (probe-idempotent).
    let buf2 = br.fill_buf().expect("second fill_buf must succeed");
    assert_eq!(
        buf2[0], 0x0A,
        "byte-0 unchanged after second fill_buf — probe is non-destructive"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-002: pcapng magic routes to pcapng branch (BC-2.01.009 PC1)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-002 / BC-2.01.009 PC1: smb3.pcapng fixture (formerly negative assertion
/// for BC-2.01.004) MUST return Ok(PcapSource) with correct packet count.
///
/// Canonical test vector (BC-2.01.009 test vectors):
///   Input:  tests/fixtures/smb3.pcapng
///   Output: Ok(PcapSource); packets.len() > 0 (fixture-specific: smb3 has EPBs)
///
/// Also covers AC-011 (LE pcapng, BOM=4D 3C 2B 1A → endian-independent magic probe).
#[test]
fn test_BC_2_01_009_smb3_pcapng_accepted() {
    let path = Path::new("tests/fixtures/smb3.pcapng");
    assert!(
        path.exists(),
        "smb3.pcapng fixture must exist in tests/fixtures/"
    );

    let result = PcapSource::from_file(path);
    assert!(
        result.is_ok(),
        "BC-2.01.009 PC1: smb3.pcapng must return Ok(PcapSource); got: {:?}",
        result.err()
    );
    let source = result.unwrap();
    // smb3.pcapng has EPBs: fixture-specific assertion (not a general postcondition).
    assert!(
        !source.packets.is_empty(),
        "smb3.pcapng contains EPBs; packets must not be empty for this fixture"
    );
}

/// AC-002 / BC-2.01.009 PC1: inline minimal pcapng bytes (PCAPNG_MAGIC →
/// pcapng branch → Ok). Proves the routing decision, not the fixture.
///
/// Uses a synthetic SHB-only pcapng (28 bytes, LE, major=1, minor=0).
#[test]
fn test_BC_2_01_009_pcapng_magic_routes_to_pcapng_path() {
    let bytes = minimal_shb_pcapng_le();
    assert_eq!(
        &bytes[0..4],
        &PCAPNG_MAGIC,
        "fixture sanity: first 4 bytes must be PCAPNG_MAGIC"
    );

    let cursor = Cursor::new(bytes);
    let result = PcapSource::from_pcap_reader(cursor);
    assert!(
        result.is_ok(),
        "PCAPNG_MAGIC in bytes 0-3 must route to pcapng branch and return Ok; got: {:?}",
        result.err()
    );
    // Zero-packet SHB-only result (AC-010 combined).
    let source = result.unwrap();
    assert_eq!(
        source.packets.len(),
        0,
        "SHB-only pcapng must yield 0 packets"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-003: Classic-pcap routing unchanged (BC-2.01.009 PC2)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-003 / BC-2.01.009 PC2: classic-pcap magic bytes still route to the
/// classic-pcap path. Regression guard: all four classic-pcap magic values
/// must be accepted correctly after the probe insertion.
///
/// This test covers all four classic-pcap magics (LE/BE microsecond, LE nanosecond).
/// The BE nanosecond magic (0x4D3CB2A1) is tested separately as it's less common.
#[test]
fn test_BC_2_01_009_classic_pcap_routing_unchanged() {
    // LE microsecond: 0xA1B2C3D4
    {
        let buf = classic_pcap_header_le_ethernet();
        let result = PcapSource::from_pcap_reader(Cursor::new(buf));
        assert!(
            result.is_ok(),
            "classic LE microsecond magic must route to classic-pcap path; got: {:?}",
            result.err()
        );
        let source = result.unwrap();
        assert_eq!(
            source.datalink,
            pcap_file::DataLink::ETHERNET,
            "classic-pcap LE must yield ETHERNET linktype"
        );
        // skipped_blocks and opb_skipped are always 0 for classic pcap
        assert_eq!(
            source.skipped_blocks, 0,
            "classic pcap: skipped_blocks must be 0"
        );
        assert_eq!(source.opb_skipped, 0, "classic pcap: opb_skipped must be 0");
    }

    // BE microsecond magic [A1 B2 C3 D4]: the pcap-file crate reads this as
    // BE u32 = 0xA1B2C3D4 → routes to BigEndian parsing for remaining fields.
    {
        let mut buf = Vec::new();
        buf.extend_from_slice(&CLASSIC_MAGIC_BE_US); // [A1 B2 C3 D4]
        buf.extend_from_slice(&2u16.to_be_bytes()); // remaining header is BE
        buf.extend_from_slice(&4u16.to_be_bytes());
        buf.extend_from_slice(&0i32.to_be_bytes());
        buf.extend_from_slice(&0u32.to_be_bytes());
        buf.extend_from_slice(&65535u32.to_be_bytes());
        buf.extend_from_slice(&1u32.to_be_bytes()); // ETHERNET
        let result = PcapSource::from_pcap_reader(Cursor::new(buf));
        assert!(
            result.is_ok(),
            "classic BE magic [A1 B2 C3 D4] must route to classic-pcap path; got: {:?}",
            result.err()
        );
    }

    // LE nanosecond: 0xA1B23C4D (EC-006)
    {
        let buf = classic_pcap_header_le_ns_ethernet();
        let result = PcapSource::from_pcap_reader(Cursor::new(buf));
        assert!(
            result.is_ok(),
            "classic LE nanosecond magic must route to classic-pcap path; got: {:?}",
            result.err()
        );
    }
}

/// AC-003 / BC-2.01.009 PC2 / EC-006: nanosecond-resolution pcap routing.
///
/// The nanosecond magic (0xA1B23C4D) must be recognized and routed to the
/// classic-pcap path, not confused with pcapng.
#[test]
fn test_BC_2_01_009_nanosecond_pcap_routing() {
    let buf = classic_pcap_header_le_ns_ethernet();
    assert_eq!(
        &buf[0..4],
        &CLASSIC_MAGIC_LE_NS,
        "fixture sanity: nanosecond magic must be [A1 B2 3C 4D]"
    );

    let result = PcapSource::from_pcap_reader(Cursor::new(buf));
    assert!(
        result.is_ok(),
        "BC-2.01.009 EC-006: nanosecond-resolution pcap must route to classic-pcap path; got: {:?}",
        result.err()
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-004: Unrecognized magic → Err (BC-2.01.009 PC4)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-004 / BC-2.01.009 PC4: four bytes that match neither pcapng nor classic-pcap
/// magic must return Err with context indicating unrecognized magic.
///
/// Canonical test vector: 4 bytes `[0xDE, 0xAD, 0xBE, 0xEF]` → Err.
/// Error must mention "unrecognized" or "pcap magic" (BC-2.01.009 PC4).
#[test]
fn test_BC_2_01_009_unrecognized_magic() {
    let unrecognized: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
    let result = PcapSource::from_pcap_reader(Cursor::new(unrecognized));
    assert!(
        result.is_err(),
        "BC-2.01.009 PC4: [DE AD BE EF] must return Err (unrecognized magic)"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    // The error must mention the unrecognized magic (not just "failed to parse").
    assert!(
        err_msg.contains("unrecognized") || err_msg.contains("magic"),
        "error must mention 'unrecognized' or 'magic'; got: {err_msg}"
    );
}

/// AC-004 / BC-2.01.009 EC-003: stream under 4 bytes must return Err.
///
/// The probe requires 4 bytes; a 2-byte stream is a short-read (not a
/// precondition violation — the BC explicitly removed the "at least 4 bytes"
/// precondition in v1.5 and made this a graceful Err, not a panic).
#[test]
fn test_BC_2_01_009_stream_under_4_bytes() {
    // 2 bytes — too short to read the magic.
    let short: [u8; 2] = [0x0A, 0x0D];
    let result = PcapSource::from_pcap_reader(Cursor::new(short));
    assert!(
        result.is_err(),
        "BC-2.01.009 EC-003: stream < 4 bytes must return Err (short-read); got Ok"
    );
    // Must not panic; return Err gracefully.
    // Empty stream: 0 bytes.
    let empty: [u8; 0] = [];
    let result_empty = PcapSource::from_pcap_reader(Cursor::new(empty));
    assert!(
        result_empty.is_err(),
        "0-byte stream must return Err; got Ok"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-005: BOM detection via canonical table (BC-2.01.010 PC1)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-005 / BC-2.01.010 PC1: parse_shb_body with LE BOM (4D 3C 2B 1A) →
/// SectionEndianness::LittleEndian.
///
/// Canonical test vector: on-disk 4D 3C 2B 1A → little-endian.
#[test]
fn test_BC_2_01_010_bom_little_endian() {
    // SHB body: BOM(4) + major(2) + minor(2) + section_length(8) = 16 bytes.
    let mut body = Vec::with_capacity(16);
    body.extend_from_slice(&SHB_BOM_LE); // 4D 3C 2B 1A = little-endian
    body.extend_from_slice(&1u16.to_le_bytes()); // major = 1
    body.extend_from_slice(&0u16.to_le_bytes()); // minor = 0
    body.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes()); // section_length
    assert_eq!(
        body.len(),
        SHB_BODY_FIXED_BYTES,
        "body must be exactly 16 bytes"
    );

    let result = parse_shb_body(&body);
    assert!(
        result.is_ok(),
        "valid LE BOM must return Ok(ShbInfo); got: {:?}",
        result.err()
    );
    let info = result.unwrap();
    assert_eq!(
        info.endianness,
        SectionEndianness::LittleEndian,
        "on-disk 4D 3C 2B 1A → SectionEndianness::LittleEndian (BC-2.01.010 PC1 canonical BOM table)"
    );
    assert_eq!(info.major_version, 1, "major_version must be 1");
    assert_eq!(info.minor_version, 0, "minor_version must be 0");
}

/// AC-005 / BC-2.01.010 PC1: parse_shb_body with BE BOM (1A 2B 3C 4D) →
/// SectionEndianness::BigEndian, major=1, minor=2.
///
/// Canonical test vector: on-disk 1A 2B 3C 4D → big-endian.
/// Also covers AC-011 (SHB magic is endian-independent).
///
/// # Spec-conforming body layout
///
/// When the BOM declares big-endian, ALL subsequent multi-byte body fields MUST be
/// big-endian encoded (BC-2.01.010 PC1 / Invariant 4). This test uses:
///   - major_version = 1  encoded BE: `00 01`  (LE misread → 256 ≠ 1 → wrong)
///   - minor_version = 2  encoded BE: `00 02`  (LE misread → 512 ≠ 2 → wrong)
///
/// A `parse_shb_body` that reads version fields unconditionally as LE will interpret
/// `00 01` (BE major=1) as `0x0100 = 256`, triggering "Unsupported pcapng major
/// version: 256" and returning Err — making this test RED against the current impl
/// (reader.rs lines 206-207 use `u16::from_le_bytes` unconditionally).
///
/// # RED GATE EXPECTATION
///
/// Current reader.rs implementation (lines 206-207):
///   `let major_version = u16::from_le_bytes([body[4], body[5]]);`
///   `let minor_version = u16::from_le_bytes([body[6], body[7]]);`
///
/// With BE body bytes `[..., 00 01, 00 02, ...]`:
///   - major decoded as LE: body[4]=0x00, body[5]=0x01 → u16::from_le_bytes([0,1]) = 256
///   - major=256 ≠ 1 → Err("Unsupported pcapng major version: 256")
///   - parse_shb_body returns Err, not Ok
///   - `result.is_ok()` assertion FAILS → RED
///
/// The test becomes GREEN only when `parse_shb_body` applies section endianness
/// (established by the BOM) to subsequent field decoding.
#[test]
fn test_BC_2_01_010_bom_big_endian() {
    // Spec-conforming big-endian SHB body: BOM says BE, so ALL fields are BE-encoded.
    let mut body = Vec::with_capacity(16);
    body.extend_from_slice(&SHB_BOM_BE); // 1A 2B 3C 4D (big-endian BOM)
    body.extend_from_slice(&1u16.to_be_bytes()); // 00 01 (major=1 BE; LE misread = 256)
    body.extend_from_slice(&2u16.to_be_bytes()); // 00 02 (minor=2 BE; LE misread = 512)
    body.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_be_bytes()); // section_length (BE)
    assert_eq!(
        body.len(),
        SHB_BODY_FIXED_BYTES,
        "body must be exactly 16 bytes"
    );

    let result = parse_shb_body(&body);

    // PRIMARY ASSERTION — currently RED:
    // parse_shb_body reads major via u16::from_le_bytes([0x00, 0x01]) = 256 ≠ 1 → Err.
    // Correct BE-aware decode: u16::from_be_bytes([0x00, 0x01]) = 1 → Ok.
    assert!(
        result.is_ok(),
        "spec-conforming BE body (BOM=1A2B3C4D, major/minor BE-encoded) must return Ok(ShbInfo); \
         current impl reads version LE → major=256 → Err (RED GATE); got: {:?}",
        result.err()
    );

    let info = result.unwrap();

    // ENDIANNESS
    assert_eq!(
        info.endianness,
        SectionEndianness::BigEndian,
        "on-disk BOM 1A 2B 3C 4D → SectionEndianness::BigEndian (BC-2.01.010 PC1 canonical BOM table)"
    );

    // MAJOR VERSION — non-palindromic detection:
    // BE bytes `00 01` → correct decode = 1; LE misread = 256.
    assert_eq!(
        info.major_version, 1,
        "BE-encoded major_version (`00 01`) must decode to 1; LE misread gives 256 (RED)"
    );

    // MINOR VERSION — non-palindromic detection:
    // BE bytes `00 02` → correct decode = 2; LE misread = 512.
    // This assertion makes any LE-fallback detectable: a LE read of `00 02` = 512 ≠ 2.
    assert_eq!(
        info.minor_version, 2,
        "BE-encoded minor_version (`00 02`) must decode to 2; LE misread gives 512 (detects LE fallback)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-006: Major version validation (BC-2.01.010 PC2)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-006 / BC-2.01.010 PC2 / AC-003: pcapng major version != 1 must return Err.
///
/// Canonical test vector: major_version=2 → Err with "unsupported" context.
/// Minor version is irrelevant; a non-1 major is always rejected.
#[test]
fn test_BC_2_01_010_major_version_not_1_rejected() {
    // Test with major=2 (EC-004: future version)
    let mut body = Vec::with_capacity(16);
    body.extend_from_slice(&SHB_BOM_LE);
    body.extend_from_slice(&2u16.to_le_bytes()); // major = 2 (future, unsupported)
    body.extend_from_slice(&0u16.to_le_bytes());
    body.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes());

    let result = parse_shb_body(&body);
    assert!(
        result.is_err(),
        "major_version=2 must return Err (BC-2.01.010 PC2); got Ok"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    assert!(
        err_msg.to_lowercase().contains("unsupported")
            || err_msg.to_lowercase().contains("version"),
        "error must mention 'unsupported' or 'version' for major!=1; got: {err_msg}"
    );

    // Also test via from_pcap_reader (integration path): craft an SHB-only
    // pcapng with major=2 in the body.
    {
        let shb_bytes = shb_only_pcapng(SHB_BOM_LE, 2, 0);
        let result = PcapSource::from_pcap_reader(Cursor::new(shb_bytes));
        assert!(
            result.is_err(),
            "from_pcap_reader: major_version=2 SHB must return Err; got Ok"
        );
    }

    // major=0: also invalid.
    {
        let mut body0 = Vec::with_capacity(16);
        body0.extend_from_slice(&SHB_BOM_LE);
        body0.extend_from_slice(&0u16.to_le_bytes()); // major = 0 (invalid)
        body0.extend_from_slice(&0u16.to_le_bytes());
        body0.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes());
        let result0 = parse_shb_body(&body0);
        assert!(result0.is_err(), "major_version=0 must return Err; got Ok");
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-007: SHB error routing 4-way split (BC-2.01.010 PC5 / Decision 20)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-007 case (b) / BC-2.01.010 AC-004a / EC-005: SHB with btl=16 → body=4 bytes
/// (< 16 SHB fixed-field bytes) → E-INP-008 (body-too-short).
///
/// ADR-009 rev 7 Decision 20 Tier 2: crate accepts the block (btl=16 ≥ 12,
/// btl%4==0); wirerust body-decode finds body < 16 → E-INP-008.
/// Constructible window: 12 ≤ btl < 28; canonical fixture: btl=16 → body=4.
///
/// The error must mention E-INP-008 or equivalent context ("body" or "truncated"
/// or "too short"). It must NOT be E-INP-010 (which is only for crate framing
/// rejections that never reach wirerust's body-decode).
#[test]
fn test_BC_2_01_010_shb_body_truncated_e_inp_008() {
    let bytes = shb_body_truncated_btl16();
    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "BC-2.01.010 AC-004a: SHB btl=16 (body=4 < 16 fixed bytes) must return Err (E-INP-008); got Ok"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    // Must be E-INP-008 (body-too-short), not E-INP-010 (framing rejection).
    // E-INP-010 is the framing code; E-INP-008 is the wirerust body-decode code.
    // The exact error string may not include "E-INP-008" literally; we check for
    // context that indicates a body-length or parse failure.
    assert!(
        err_msg.to_lowercase().contains("e-inp-008")
            || err_msg.to_lowercase().contains("body")
            || err_msg.to_lowercase().contains("truncat")
            || err_msg.to_lowercase().contains("too short")
            || err_msg.to_lowercase().contains("fixed")
            || err_msg.to_lowercase().contains("shb"),
        "E-INP-008 path: error must mention body parse failure context; got: {err_msg}"
    );
    // Must explicitly NOT contain E-INP-010 (which would indicate wrong routing).
    assert!(
        !err_msg.to_lowercase().contains("e-inp-010"),
        "E-INP-008 path must not be misrouted to E-INP-010; got: {err_msg}"
    );
}

/// AC-007 case (a) / BC-2.01.010 AC-004b / EC-008: SHB with btl=8 → crate
/// framing rejection (btl < 12) → E-INP-010.
///
/// ADR-009 rev 7 Decision 20 Tier 1: crate rejects BEFORE returning block to
/// wirerust. wirerust never sees the body. All crate framing Errs → E-INP-010.
///
/// The bytes include pcapng magic in the first 4 bytes so the probe routes to
/// the pcapng branch. The framing-rejection Err must be surfaced as E-INP-010
/// (not E-INP-008 which is a wirerust body-decode code).
#[test]
fn test_BC_2_01_010_shb_framing_rejection_e_inp_010() {
    let bytes = shb_framing_btl8();
    // Prepend PCAPNG_MAGIC so the probe routes to the pcapng branch.
    // Actually, shb_framing_btl8() already starts with SHB_BLOCK_TYPE which
    // equals PCAPNG_MAGIC as bytes — the block_type IS the SHB magic.
    // Let's verify:
    assert_eq!(
        &bytes[0..4],
        &PCAPNG_MAGIC,
        "SHB_BLOCK_TYPE bytes == PCAPNG_MAGIC for the probe to route correctly"
    );

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_err(),
        "BC-2.01.010 AC-004b: SHB btl=8 (< 12, crate framing rejection) must return Err (E-INP-010); got Ok"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    // E-INP-010 is the crate framing error code. The error may say "framing",
    // "invalid", "block", "length", etc. depending on implementation. We check
    // for the distinguishing phrase not being E-INP-008 (which would be wrong).
    assert!(
        !err_msg.to_lowercase().contains("e-inp-008"),
        "E-INP-010 path must not be misrouted to E-INP-008 (body-decode); got: {err_msg}"
    );
    // The error must be present (is_err() above ensures this).
}

/// AC-007 case (c) / BC-2.01.010 AC-001 / EC-007: SHB with valid framing
/// (btl=28, body=16) but invalid BOM → E-INP-008.
///
/// Canonical holdout fixture: BOM = DE AD BE EF (neither row of PC1 table).
/// ADR-009 rev 7 Decision 20 Tier 2 semantic invalid: body ≥ 16 but BOM invalid.
#[test]
fn test_BC_2_01_010_invalid_bom_e_inp_008() {
    // BOM = DE AD BE EF: not in the canonical BOM table.
    let body_with_invalid_bom: [u8; 16] = [
        0xDE, 0xAD, 0xBE, 0xEF, // invalid BOM
        0x01, 0x00, // major = 1 (valid)
        0x00, 0x00, // minor = 0
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // section_length
    ];

    // Test via parse_shb_body directly (pure-core helper).
    let result = parse_shb_body(&body_with_invalid_bom);
    assert!(
        result.is_err(),
        "BC-2.01.010 AC-001: invalid BOM (DE AD BE EF) must return Err (E-INP-008); got Ok"
    );
    let err_msg = format!("{:#}", result.unwrap_err());
    assert!(
        err_msg.to_lowercase().contains("bom")
            || err_msg.to_lowercase().contains("byte-order")
            || err_msg.to_lowercase().contains("e-inp-008")
            || err_msg.to_lowercase().contains("invalid")
            || err_msg.to_lowercase().contains("unrecognized"),
        "E-INP-008 BOM path: error must identify BOM mismatch; got: {err_msg}"
    );

    // Also test via from_pcap_reader (integration path).
    let shb_bytes = shb_with_invalid_bom();
    let result2 = PcapSource::from_pcap_reader(Cursor::new(shb_bytes));
    assert!(
        result2.is_err(),
        "from_pcap_reader: SHB with invalid BOM must return Err; got Ok"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-008: Multi-section rejection (BC-2.01.010 AC-002 / EC-006 → E-INP-012)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-008 / BC-2.01.010 AC-002: a SECOND SHB encountered after the first
/// MUST be rejected with Err mapping to E-INP-012.
///
/// Canonical fixture: SHB₁ + IDB + SHB₂ (BC-2.01.010 AC-002 canonical fixture).
/// Requirement: rejection fires BEFORE any byte-order reset from SHB₂.
/// Error message MUST include the mergecap remediation hint per BC-2.01.010 AC-002.
///
/// Key invariants checked:
/// - result.is_err() (second SHB rejected)
/// - error message contains "multi-section" or "second" (identifies cause)
/// - error message contains "mergecap" (remediation hint per ADR-009 Decision 7)
/// - error message does NOT indicate success
#[test]
fn test_BC_2_01_010_second_shb_rejected_e_inp_012() {
    let bytes = two_section_pcapng();
    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));

    assert!(
        result.is_err(),
        "BC-2.01.010 AC-002: second SHB in stream must return Err (E-INP-012); got Ok"
    );
    let err_msg = format!("{:#}", result.unwrap_err());

    // Must identify the multi-section cause.
    assert!(
        err_msg.to_lowercase().contains("multi-section")
            || err_msg.to_lowercase().contains("second")
            || err_msg.to_lowercase().contains("e-inp-012"),
        "E-INP-012: error must identify second SHB / multi-section; got: {err_msg}"
    );

    // Must include the remediation hint (mergecap or editcap per ADR-009 Decision 7).
    assert!(
        err_msg.to_lowercase().contains("mergecap") || err_msg.to_lowercase().contains("editcap"),
        "E-INP-012: error must include 'mergecap' or 'editcap' remediation hint; got: {err_msg}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-009: No-panic fuzz over arbitrary SHB body bytes (BC-2.01.010 AC-005 / SEC-005)
// ──────────────────────────────────────────────────────────────────────────────

// AC-009 / BC-2.01.010 AC-005 / VP-026 (proptest coverage): parse_shb_body
// MUST return Err for any malformed input and MUST NOT panic.
//
// BC-2.01.010 AC-005 (SEC-005): `unwrap()`, `expect()`, `panic!()`, and
// `unreachable!()` are prohibited in the SHB parse path.
//
// This property test generates 1000+ random byte slices (0..=64 bytes) and
// asserts that parse_shb_body never panics. The proptest runner catches panics
// as failures, so any panic surfaces here. A Result::Err is acceptable and
// expected for inputs shorter than 16 bytes or with invalid BOM/version.
//
// VP-026: Kani formally verifies parse_shb_body totality; this proptest
// provides complementary coverage at the integration test level.
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn test_BC_2_01_010_no_panic_fuzz(
        // Generate arbitrary byte slices of length 0..=64 (covers all truncation points
        // including the SHB fixed-field minimum of 16 and the minimum total btl of 28).
        body in proptest::collection::vec(any::<u8>(), 0..=64)
    ) {
        // parse_shb_body must not panic regardless of input.
        // It should return Ok only for inputs with valid BOM, major=1, body≥16.
        // For all other inputs it must return Err.
        // The proptest runner itself catches panics as test failures (prop_assert is
        // not needed here — the act of calling the function without panicking is
        // the safety property we are testing).
        let _ = parse_shb_body(&body); // must not panic
    }

    #[test]
    fn test_BC_2_01_010_no_panic_fuzz_full_pcapng_stream(
        // Generate arbitrary byte slices of length 4..=256.
        // Prepend PCAPNG_MAGIC so the probe routes to the pcapng branch.
        // Any arbitrary bytes after the magic must not cause a panic.
        rest in proptest::collection::vec(any::<u8>(), 4..=252)
    ) {
        let mut stream = Vec::with_capacity(4 + rest.len());
        stream.extend_from_slice(&PCAPNG_MAGIC);
        stream.extend_from_slice(&rest);
        // from_pcap_reader on arbitrary pcapng-magic-prefixed bytes must not panic.
        // It MUST return Ok or Err; panics are failures here (proptest catches them).
        let _ = PcapSource::from_pcap_reader(Cursor::new(stream));
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-010: Zero-packet notice on SHB-only file (BC-2.01.009 PC6 / EC-010 / F-M4)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-010 / BC-2.01.009 EC-010 / F-M4: SHB-only pcapng (no IDB, no subsequent
/// blocks) is structurally valid and yields Ok(PcapSource) with:
///   - packets.len() == 0
///   - skipped_blocks == 0
///   - opb_skipped == 0
///
/// Note: main.rs emits the zero-packet notice (not from_pcap_reader).
/// This test verifies only the PcapSource fields; main.rs emission is
/// tested in CLI integration tests (out of scope for this reader test file).
#[test]
fn test_BC_2_01_009_shb_only_zero_packet_notice() {
    let bytes = minimal_shb_pcapng_le();
    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "BC-2.01.009 EC-010: SHB-only pcapng must return Ok (structurally valid); got: {:?}",
        result.err()
    );
    let source = result.unwrap();
    assert_eq!(
        source.packets.len(),
        0,
        "SHB-only pcapng: packets.len() must be 0 (no EPB/SPB blocks)"
    );
    assert_eq!(
        source.skipped_blocks, 0,
        "SHB-only pcapng: skipped_blocks must be 0 (no blocks to skip after SHB)"
    );
    assert_eq!(
        source.opb_skipped, 0,
        "SHB-only pcapng: opb_skipped must be 0"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-012: arp-baseline-16pkt.cap fixture (BC-2.01.009 PC5 / ADR-009 Decision 11)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-012 / BC-2.01.009 PC5 / ADR-009 Decision 11: `arp-baseline-16pkt.cap`
/// (pcapng file with .cap extension) MUST return Ok(PcapSource) with 16 packets.
///
/// This proves that format detection is by CONTENT (magic-byte probe), NOT by
/// file extension — a .cap file is correctly routed to the pcapng branch if
/// its first 4 bytes are PCAPNG_MAGIC.
///
/// FIXTURE NOTE: arp-baseline-16pkt.cap did NOT exist at test-suite authorship
/// time. A synthetic fixture is created by ensure_arp_baseline_fixture().
/// See the F3 FOLLOW-UP ITEM note at the top of this file.
/// The synthetic fixture has exactly 16 EPB packets and a .cap extension.
/// The implementer / PO must replace it with the authentic PacketLife capture
/// before Phase-4 holdout evaluation.
#[test]
fn test_BC_2_01_009_arp_baseline_cap_accepted() {
    let path = ensure_arp_baseline_fixture();
    assert!(
        path.exists(),
        "arp-baseline-16pkt.cap fixture must exist (either original or synthetic)"
    );

    // Verify it starts with pcapng magic (not classic pcap).
    let first4 = {
        let mut f = std::fs::File::open(&path).expect("fixture must be openable");
        let mut buf = [0u8; 4];
        f.read_exact(&mut buf).expect("fixture must have ≥ 4 bytes");
        buf
    };
    assert_eq!(
        first4, PCAPNG_MAGIC,
        "arp-baseline-16pkt.cap must start with PCAPNG_MAGIC (content-based detection)"
    );

    let result = PcapSource::from_file(&path);
    assert!(
        result.is_ok(),
        "BC-2.01.009 PC5: arp-baseline-16pkt.cap (.cap extension, pcapng content) must return Ok; got: {:?}",
        result.err()
    );
    let source = result.unwrap();
    assert_eq!(
        source.packets.len(),
        16,
        "BC-2.01.009 PC5: arp-baseline-16pkt.cap must yield exactly 16 packets"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-011: pcapng SHB magic is endian-independent (BC-2.01.009 Invariant 4)
// ──────────────────────────────────────────────────────────────────────────────
//
// AC-011 is covered by:
//   - test_BC_2_01_009_smb3_pcapng_accepted (LE pcapng: BOM=4D3C2B1A)
//   - test_BC_2_01_010_bom_big_endian (BE SHB body: BOM=1A2B3C4D)
//
// The SHB magic 0x0A0D0D0A is the same 4 bytes in both byte orders (it is
// deliberately designed to be palindromic in its byte pattern after the first 2
// bytes, and its value is the same regardless of byte order interpretation).
// Detection by the probe does NOT depend on byte order — only BOM detection
// (inside the SHB body) determines endianness.
//
// Additional direct test:
/// AC-011 / BC-2.01.009 Invariant 4: PCAPNG_MAGIC [0A 0D 0D 0A] is the same
/// 4-byte literal regardless of byte order; probe routes identically for BE and LE files.
#[test]
fn test_BC_2_01_009_pcapng_magic_endian_independent() {
    // Both LE SHB and BE SHB files start with the same 4 bytes (PCAPNG_MAGIC).
    let le_bytes = minimal_shb_pcapng_le();
    let be_bytes = minimal_shb_pcapng_be();

    // First 4 bytes of both must be identical (PCAPNG_MAGIC).
    assert_eq!(
        &le_bytes[0..4],
        &PCAPNG_MAGIC,
        "LE SHB: first 4 bytes must be PCAPNG_MAGIC"
    );
    assert_eq!(
        &be_bytes[0..4],
        &PCAPNG_MAGIC,
        "BE SHB: first 4 bytes must be PCAPNG_MAGIC"
    );

    // Both route to the pcapng branch (probe is endian-independent).
    let le_result = PcapSource::from_pcap_reader(Cursor::new(le_bytes));
    let be_result = PcapSource::from_pcap_reader(Cursor::new(be_bytes));

    assert!(
        le_result.is_ok(),
        "LE SHB must route to pcapng branch; got: {:?}",
        le_result.err()
    );
    assert!(
        be_result.is_ok(),
        "BE SHB must route to pcapng branch; got: {:?}",
        be_result.err()
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// parse_shb_body: additional contract tests (BC-2.01.010 PC3)
// ──────────────────────────────────────────────────────────────────────────────

/// BC-2.01.010 PC3: section_length = 0xFFFFFFFFFFFFFFFF (unspecified) is accepted;
/// the reader does not use section_length for bounds checking.
#[test]
fn test_BC_2_01_010_section_length_unspecified_accepted() {
    let mut body = Vec::with_capacity(16);
    body.extend_from_slice(&SHB_BOM_LE);
    body.extend_from_slice(&1u16.to_le_bytes());
    body.extend_from_slice(&0u16.to_le_bytes());
    body.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes()); // unspecified

    let result = parse_shb_body(&body);
    assert!(
        result.is_ok(),
        "section_length=0xFFFFFFFFFFFFFFFF must be accepted (BC-2.01.010 PC3); got: {:?}",
        result.err()
    );
}

/// BC-2.01.010 PC3: section_length = 0 is also accepted (any value ok).
#[test]
fn test_BC_2_01_010_section_length_zero_accepted() {
    let mut body = Vec::with_capacity(16);
    body.extend_from_slice(&SHB_BOM_LE);
    body.extend_from_slice(&1u16.to_le_bytes());
    body.extend_from_slice(&0u16.to_le_bytes());
    body.extend_from_slice(&0u64.to_le_bytes()); // section_length = 0

    let result = parse_shb_body(&body);
    assert!(
        result.is_ok(),
        "section_length=0 must be accepted (BC-2.01.010 PC3: any value ok); got: {:?}",
        result.err()
    );
}

/// BC-2.01.010 body-too-short: body.len() < 16 → Err (E-INP-008).
/// Tests the boundary at 15 bytes (one byte short of the 16-byte minimum).
#[test]
fn test_BC_2_01_010_body_too_short_15_bytes() {
    // 15 bytes: one byte short of the 16-byte SHB body minimum.
    let short_body = vec![0u8; 15];
    let result = parse_shb_body(&short_body);
    assert!(
        result.is_err(),
        "body.len()=15 (< 16 SHB_BODY_FIXED_BYTES) must return Err (E-INP-008); got Ok"
    );
}

/// BC-2.01.010 body-too-short: empty body → Err.
#[test]
fn test_BC_2_01_010_body_empty_returns_err() {
    let result = parse_shb_body(&[]);
    assert!(
        result.is_err(),
        "empty body must return Err (E-INP-008 body-too-short); got Ok"
    );
}

/// BC-2.01.010 PC2: minor version may be any value ≥ 0; arbitrary minor versions are accepted.
#[test]
fn test_BC_2_01_010_minor_version_arbitrary_accepted() {
    for minor in [0u16, 1, 42, 100, u16::MAX] {
        let mut body = Vec::with_capacity(16);
        body.extend_from_slice(&SHB_BOM_LE);
        body.extend_from_slice(&1u16.to_le_bytes()); // major = 1 (valid)
        body.extend_from_slice(&minor.to_le_bytes());
        body.extend_from_slice(&0u64.to_le_bytes());

        let result = parse_shb_body(&body);
        assert!(
            result.is_ok(),
            "minor_version={minor} must be accepted (any value ≥ 0 per BC-2.01.010 PC2); got: {:?}",
            result.err()
        );
        let info = result.unwrap();
        assert_eq!(
            info.minor_version, minor,
            "minor_version must be parsed and returned"
        );
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// skipped_blocks and opb_skipped field initialization (BC-2.01.009 PC6 / Decision 19)
// ──────────────────────────────────────────────────────────────────────────────

/// BC-2.01.009 PC6 / Decision 19: classic-pcap path must always yield
/// skipped_blocks=0 and opb_skipped=0 (there are no blocks to skip in classic pcap).
#[test]
fn test_BC_2_01_009_classic_pcap_skipped_blocks_zero() {
    let buf = classic_pcap_header_le_ethernet();
    let source = PcapSource::from_pcap_reader(Cursor::new(buf)).unwrap();
    assert_eq!(
        source.skipped_blocks, 0,
        "classic pcap: skipped_blocks must always be 0"
    );
    assert_eq!(
        source.opb_skipped, 0,
        "classic pcap: opb_skipped must always be 0"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// C-1 REGRESSION GUARD: Genuine big-endian pcapng end-to-end decode
//
// ADR-009 rev 9 Decision 1 — the raw-block path MUST handle both byte orders.
// BC-2.01.010 PC1 Invariant 4 — the SHB BOM establishes endianness ONCE for the
// entire section; ALL subsequent multi-byte fields (block_type, btl, linktype,
// captured_len, timestamps) MUST be decoded with that endianness.
//
// The CRITICAL BUG this test exposes: the current implementation reads the outer
// block framing (block_type, block_total_length) always as little-endian even in
// a big-endian section. For a genuine BE pcapng file:
//   - SHB btl = 00 00 00 1C (28 in BE), misread as LE → 0x1C000000 = 469762048
//   - Reader immediately fails with "stream too short for btl=469762048"
//
// This makes the test RED against the current LE-always implementation and GREEN
// only when the implementer re-implements using pcap-file 2.0.0's
// PcapNgParser/RawBlock API (which handles both endiannesses correctly) per the
// architect's binding ruling.
//
// Non-palindromic multi-byte values used (why LE-misread produces a wrong result):
//
//   SHB btl = 28  (BE: 00 00 00 1C) → LE misread: 469762048  (stream too short)
//   IDB btl = 20  (BE: 00 00 00 14) → LE misread: 335544320  (stream too short)
//   IDB linktype = 1 (BE: 00 01)    → LE misread: 256 (not ETHERNET; wrong type)
//   EPB btl = 36  (BE: 00 00 00 24) → LE misread: 603979776  (stream too short)
//   EPB captured_len = 4 (BE: 00 00 00 04) → LE misread: 67108864 (OOB error)
//
// Any ONE of these misreads causes a different, wrong outcome than the correct
// decoded result (Ok, 1 packet, ETHERNET linktype, data = [BE EF CA FE]).
// ──────────────────────────────────────────────────────────────────────────────

/// C-1 / BC-2.01.010 PC1 / BC-2.01.010 Invariant 4 / HS-103: feed a GENUINE
/// big-endian pcapng file (all outer framing fields in BE) through
/// `PcapSource::from_pcap_reader` and assert the decoded output is correct.
///
/// The file is: SHB (BE, BOM=1A 2B 3C 4D) + IDB (linktype=ETHERNET, BE) +
///              EPB (1 packet, 4 bytes `[BE EF CA FE]`, BE).
///
/// Correct result (after endianness-correct decode):
///   - `Ok(PcapSource)`
///   - `packets.len() == 1`
///   - `packets[0].data == [0xBE, 0xEF, 0xCA, 0xFE]`
///   - `datalink == DataLink::ETHERNET`
///
/// Under the current LE-always implementation the SHB btl is misread as
/// 0x1C000000 = 469762048, causing an immediate "stream too short" error.
/// This test is therefore RED now and becomes GREEN only when the implementer
/// applies section-endianness to outer block framing fields.
///
/// See `genuine_be_pcapng_with_one_packet()` doc-comment for the full byte-by-byte
/// breakdown of non-palindromic field values and their LE-misread consequences.
#[test]
fn test_BC_2_01_010_genuine_be_section_end_to_end() {
    let bytes = genuine_be_pcapng_with_one_packet();

    // Sanity: first 4 bytes are PCAPNG_MAGIC (endian-independent) so the probe routes.
    assert_eq!(
        &bytes[0..4],
        &PCAPNG_MAGIC,
        "BE fixture: first 4 bytes must still be PCAPNG_MAGIC (endian-independent)"
    );
    // Sanity: BOM bytes at offset 8..12 (after 4 block_type + 4 btl) are BE BOM.
    assert_eq!(
        &bytes[8..12],
        &SHB_BOM_BE,
        "BE fixture: BOM at body offset 0 must be 1A 2B 3C 4D (big-endian)"
    );

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));

    // ── PRIMARY ASSERTION: the entire file decodes without error ─────────────
    //
    // CURRENT EXPECTED STATE: FAIL (RED GATE)
    //
    // The current LE-always implementation reads the SHB btl at bytes [4..8] as
    // u32::from_le_bytes([0x00, 0x00, 0x00, 0x1C]) = 0x1C000000 = 469762048.
    // This triggers the "stream too short for declared btl=469762048" error path
    // before any body field is decoded. The test therefore currently returns
    // Err(...) instead of Ok(PcapSource).
    //
    // PASS CONDITION: The implementer re-implements using the pcap-file 2.0.0
    // PcapNgParser/RawBlock path, which detects BE endianness from the BOM and
    // correctly decodes all subsequent framing fields as big-endian.
    assert!(
        result.is_ok(),
        "BC-2.01.010 Invariant 4 / C-1: genuine BE pcapng (all fields BE) must decode \
         without error; current LE-always impl fails here (RED); got: {:?}",
        result.as_ref().err()
    );

    let source = result.unwrap();

    // ── PACKET COUNT ─────────────────────────────────────────────────────────
    assert_eq!(
        source.packets.len(),
        1,
        "BE pcapng must yield exactly 1 packet (EPB decoded with BE captured_len=4); \
         LE misread would give captured_len=67108864 → OOB error, not 1 packet"
    );

    // ── DECODED PACKET DATA (the key non-palindromic check) ──────────────────
    //
    // The EPB payload is [BE EF CA FE]. A correctly decoded packet returns exactly
    // these bytes. Under LE misread, captured_len would be 0x04000000 = 67108864
    // which exceeds the body, causing E-INP-008 — the packet would not be returned.
    assert_eq!(
        source.packets[0].data,
        [0xBE, 0xEF, 0xCA, 0xFE],
        "BE pcapng: EPB payload must decode to [BE EF CA FE] (4-byte sentinel); \
         LE-misread of captured_len=4 as 67108864 causes OOB error instead"
    );

    // ── LINKTYPE (non-palindromic: BE [00 01] vs LE misread [01 00] = 256) ───
    assert_eq!(
        source.datalink,
        pcap_file::DataLink::ETHERNET,
        "BE pcapng: IDB linktype=1 (BE bytes 00 01) must decode as ETHERNET; \
         LE misread reads 01 00 = 256 = unknown type (would error E-INP-001 before here)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// E-INP-009 COVERAGE: EPB before any IDB → structural error
//
// ADR-009 Error-Code table: "EPB/SPB arrives with empty interface table
// (no IDB parsed yet) → E-INP-009"
//
// BC-2.01.009 PC3 disambiguation rule (H-4): "An EPB or SPB encountered before
// any IDB has been parsed is a structural error (E-INP-009, exit 1) — it is NOT
// classified as zero-packet success; 'parses to EOF with no error' excludes this case."
//
// The current implementation contains the E-INP-009 path (reader.rs ~line 558-562:
// `if datalink.is_none() → Err("E-INP-009: no interface table entry")`).
// However, this path has ZERO test coverage — no test exercises it.
// This test adds coverage per the remediation mandate.
// ──────────────────────────────────────────────────────────────────────────────

/// E-INP-009 / BC-2.01.009 PC3 (H-4 disambiguation) / ADR-009 error-code table:
/// An EPB that appears before any IDB has been parsed MUST return `Err` mapping
/// to E-INP-009.
///
/// The fixture is: SHB (28 bytes LE) + EPB (36 bytes LE, no IDB before it).
///
/// This must NOT be classified as "zero-packet success" (exit 0).
/// It is a structural error (exit 1, E-INP-009).
///
/// The error message must contain "E-INP-009" or "interface" or "IDB" context
/// to identify the cause as "packet block before interface description."
#[test]
fn test_BC_2_01_009_epb_before_idb_e_inp_009() {
    let bytes = pcapng_epb_before_idb();

    // Sanity: starts with PCAPNG_MAGIC → pcapng branch taken.
    assert_eq!(
        &bytes[0..4],
        &PCAPNG_MAGIC,
        "E-INP-009 fixture: first 4 bytes must be PCAPNG_MAGIC"
    );

    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));

    assert!(
        result.is_err(),
        "E-INP-009: EPB before any IDB must return Err (structural error, NOT zero-packet \
         success); BC-2.01.009 PC3 H-4 disambiguation rule; got Ok"
    );

    let err_msg = format!("{:#}", result.unwrap_err());
    // The error must identify the E-INP-009 condition.
    assert!(
        err_msg.to_lowercase().contains("e-inp-009")
            || err_msg.to_lowercase().contains("interface")
            || err_msg.to_lowercase().contains("idb")
            || err_msg.to_lowercase().contains("no interface")
            || err_msg.to_lowercase().contains("before"),
        "E-INP-009: error must identify 'no interface table' / IDB-before-EPB context; \
         got: {err_msg}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// SHB-ONLY DATALINK: architect-mandated NULL sentinel (not ETHERNET)
//
// Architect ruling: SHB-only file (no IDB) → PcapSource.datalink MUST be
// DataLink::from(0) (reserved/NULL sentinel), NOT the fabricated DataLink::ETHERNET
// it currently returns.
//
// The current implementation (reader.rs line 653):
//   let final_datalink = datalink.unwrap_or(DataLink::ETHERNET);
// This fabricates ETHERNET when no IDB was seen — which is wrong. The correct
// sentinel for "no interface description seen" is DataLink::from(0) (the
// reserved/NULL value in the IANA linktype registry).
//
// This test augments test_BC_2_01_009_shb_only_zero_packet_notice to additionally
// assert the correct NULL sentinel. The existing test does NOT check datalink;
// this test does.
//
// CURRENT STATE: RED (returns ETHERNET, not from(0)).
// PASS CONDITION: implementer changes `unwrap_or(DataLink::ETHERNET)` to
//   `unwrap_or(DataLink::from(0))` (the M-3 fix per the architect ruling).
// ──────────────────────────────────────────────────────────────────────────────

/// BC-2.01.009 EC-010 / architect ruling (M-3): SHB-only pcapng (no IDB, no
/// packet blocks) must yield `PcapSource.datalink == DataLink::from(0)` — the
/// reserved/NULL sentinel — NOT `DataLink::ETHERNET`.
///
/// Rationale: when no IDB has been parsed, there is no linktype information.
/// Fabricating ETHERNET (code 1) is a tautological lie that would mislead
/// downstream analyzers. The reserved/NULL value (code 0) correctly signals
/// "no interface description was present in this file."
///
/// This test is currently RED because reader.rs uses:
///   `datalink.unwrap_or(DataLink::ETHERNET)`
/// The implementer must change this to:
///   `datalink.unwrap_or(DataLink::from(0))`
///
/// This test does NOT conflict with test_BC_2_01_009_shb_only_zero_packet_notice;
/// that test checks packets/skipped_blocks/opb_skipped only. This test checks
/// the datalink field specifically.
#[test]
fn test_BC_2_01_009_shb_only_datalink_null_sentinel() {
    let bytes = minimal_shb_pcapng_le();
    let result = PcapSource::from_pcap_reader(Cursor::new(bytes));

    assert!(
        result.is_ok(),
        "SHB-only pcapng must return Ok (structurally valid per EC-010); got: {:?}",
        result.err()
    );
    let source = result.unwrap();

    // Packets and skipped_blocks are already covered in the sibling test.
    // This test focuses exclusively on the datalink field.
    assert_eq!(
        source.packets.len(),
        0,
        "SHB-only: packets.len() must be 0 (no EPB/SPB)"
    );

    // ── THE CRITICAL ASSERTION (currently RED) ────────────────────────────────
    //
    // DataLink::from(0) is the reserved/NULL sentinel per the IANA linktype
    // registry. It signals "no interface description was seen in this file."
    //
    // The current impl returns DataLink::ETHERNET (DataLink::from(1)) because
    // it uses `datalink.unwrap_or(DataLink::ETHERNET)`. This is wrong.
    //
    // EXPECTED: DataLink::from(0)   (NULL sentinel — no IDB was present)
    // ACTUAL:   DataLink::ETHERNET  (DataLink::from(1) — fabricated, incorrect)
    let null_sentinel = pcap_file::DataLink::from(0);
    assert_eq!(
        source.datalink, null_sentinel,
        "SHB-only pcapng (no IDB): datalink must be DataLink::from(0) (NULL/reserved sentinel); \
         current impl returns DataLink::ETHERNET which is wrong — there is no linktype \
         information when no IDB is present (architect ruling M-3)"
    );
}
