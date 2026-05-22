//! STORY-001 Phase 3 TDD — PCAP File Ingestion
//!
//! This file formalizes the behavioral contracts BC-2.01.001 through BC-2.01.008.
//! Strategy: brownfield-formalization. Every test matches an AC from STORY-001.md and
//! a clause from the named BC. On the first run these are expected to PASS (the
//! implementation already satisfies the BCs); if any fail, a gap in `src/reader.rs`
//! is flagged in the Phase 3 report.
//!
//! Test naming convention: test_BC_S_SS_NNN_<assertion>()
//!
//! ## Coverage overlap note (F-m4)
//!
//! `tests/reader_tests.rs` and `tests/linktype_integration_tests.rs` contain
//! pre-existing ad-hoc tests of the same `src/reader.rs` code paths. Those
//! tests are retained unchanged under the brownfield-formalization strategy:
//! they provide regression continuity and are not superseded by this file.
//! This file adds BC-anchored, traceability-tagged coverage that maps every
//! behavioral-contract clause to a named test; the two sets are complementary.
//!
//! The BC-based naming pattern uses uppercase letters (BC-S.SS.NNN) which
//! violates Rust's snake_case convention. `#![allow(non_snake_case)]` is
//! necessary to satisfy both the factory naming mandate and CI's `-D warnings`.
#![allow(non_snake_case)]

use std::io::Cursor;
use std::path::Path;

use pcap_file::DataLink;
use proptest::prelude::*;
use wirerust::reader::PcapSource;

// ---------------------------------------------------------------------------
// Shared helper: build a minimal classic-pcap global header (24 bytes).
//
// magic: 0xa1b2c3d4 = microsecond LE
//        0xa1b23c4d = nanosecond  LE
// ---------------------------------------------------------------------------
fn pcap_header(link_type: u32, magic: u32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(24);
    buf.extend_from_slice(&magic.to_le_bytes()); // magic number
    buf.extend_from_slice(&2u16.to_le_bytes()); // version major
    buf.extend_from_slice(&4u16.to_le_bytes()); // version minor
    buf.extend_from_slice(&0i32.to_le_bytes()); // thiszone
    buf.extend_from_slice(&0u32.to_le_bytes()); // sigfigs
    buf.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
    buf.extend_from_slice(&link_type.to_le_bytes()); // network (DataLink)
    buf
}

/// Build a microsecond-resolution global header for the given link type.
fn us_header(link_type: u32) -> Vec<u8> {
    pcap_header(link_type, 0xa1b2c3d4)
}

/// Build a nanosecond-resolution global header for the given link type.
fn ns_header(link_type: u32) -> Vec<u8> {
    pcap_header(link_type, 0xa1b23c4d)
}

/// Append one packet record to an existing header buffer.
///
/// * `ts_sec`   — timestamp seconds
/// * `ts_frac`  — timestamp fraction (microseconds or nanoseconds, depending on magic)
/// * `data`     — captured bytes (`incl_len` and `orig_len` are both set to `data.len()`)
fn append_packet(buf: &mut Vec<u8>, ts_sec: u32, ts_frac: u32, data: &[u8]) {
    let len = data.len() as u32;
    buf.extend_from_slice(&ts_sec.to_le_bytes());
    buf.extend_from_slice(&ts_frac.to_le_bytes());
    buf.extend_from_slice(&len.to_le_bytes()); // incl_len
    buf.extend_from_slice(&len.to_le_bytes()); // orig_len
    buf.extend_from_slice(data);
}

/// A minimal 14-byte Ethernet frame body (enough to satisfy the reader's
/// `into_owned()` call; downstream decoders are not exercised here).
const DUMMY_ETH_FRAME: &[u8] = &[
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst mac
    0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src mac
    0x08, 0x00, // EtherType IPv4
];

// ---------------------------------------------------------------------------
// AC-001 / BC-2.01.001 postcondition 1
//
// Calling PcapSource::from_file on a pcap with any of the five accepted link
// types returns Ok(PcapSource) with `datalink` set to the accepted variant.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_01_001_accepts_all_five_link_types() {
    // Numeric values from BC-2.01.001 invariant 3.
    let cases: &[(u32, DataLink)] = &[
        (1, DataLink::ETHERNET),
        (101, DataLink::RAW),
        (113, DataLink::LINUX_SLL),
        (228, DataLink::IPV4),
        (229, DataLink::IPV6),
    ];

    for (numeric, expected_variant) in cases {
        let buf = us_header(*numeric);
        let result = PcapSource::from_pcap_reader(Cursor::new(&buf));
        assert!(
            result.is_ok(),
            "link type {numeric} ({expected_variant:?}) should be accepted, got: {:?}",
            result.unwrap_err()
        );
        let source = result.unwrap();
        assert_eq!(
            source.datalink, *expected_variant,
            "datalink field must be set to the accepted variant for link type {numeric}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-002 / BC-2.01.001 postcondition 2
//
// A pcap with link type not in the accepted set returns Err containing
// "Unsupported pcap link type" without panicking. The error message must
// also identify the offending link type (F-003: strengthen assertion).
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_01_001_rejects_unsupported_link_type() {
    // IEEE 802.11 = numeric 105 (BC-2.01.001 EC-001).
    // The implementation formats the rejected variant via its Debug repr,
    // which for link type 105 is "IEEE802_11" (from pcap_file::DataLink).
    let buf = us_header(105);
    let result = PcapSource::from_pcap_reader(Cursor::new(&buf));
    assert!(result.is_err(), "link type 105 must be rejected");
    let msg = format!("{:#}", result.unwrap_err());
    assert!(
        msg.contains("Unsupported pcap link type"),
        "error must contain 'Unsupported pcap link type', got: {msg}"
    );
    // The offending variant must be identifiable in the message so callers
    // can diagnose which link type caused the rejection.
    assert!(
        msg.contains("IEEE802_11"),
        "error must identify the rejected variant 'IEEE802_11', got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// AC-003 / BC-2.01.002 postcondition 1 + postcondition 2 (data field)
//
// For a pcap with N packet records the returned PcapSource.packets contains
// exactly N RawPacket entries in file order. Each entry's data field must
// equal the exact bytes that were written into the pcap record (F-002).
//
// F-n1: each of the three packets carries a distinct payload so that the
// data-equality assertion also catches any reordering — identical payloads
// would pass even if the reader permuted the packet Vec.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_01_002_packet_count_and_order() {
    let mut buf = us_header(1); // ETHERNET

    // Three packets: distinct payloads AND distinct timestamps so both the
    // order and the byte-preservation properties are independently verifiable.
    // Last two bytes of each payload differ to make packets distinguishable.
    let frame_a: &[u8] = &[
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x08, 0x01,
    ];
    let frame_b: &[u8] = &[
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x08, 0x02,
    ];
    let frame_c: &[u8] = &[
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x08, 0x03,
    ];

    let frames: &[(&[u8], u32)] = &[(frame_a, 1000), (frame_b, 2000), (frame_c, 3000)];
    for (data, ts) in frames {
        append_packet(&mut buf, *ts, 0, data);
    }

    let source = PcapSource::from_pcap_reader(Cursor::new(&buf)).unwrap();
    assert_eq!(source.packets.len(), 3, "packet count must be exactly 3");
    for (i, (expected_data, expected_ts)) in frames.iter().enumerate() {
        assert_eq!(
            source.packets[i].timestamp_secs, *expected_ts,
            "packet {i} timestamp_secs must match file order"
        );
        // BC-2.01.002 postcondition 2: raw frame bytes are preserved unmodified.
        // Because each packet has a unique payload, a match here also confirms
        // that packet i in the output corresponds to packet i from the file.
        assert_eq!(
            source.packets[i].data.as_slice(),
            *expected_data,
            "packet {i} data must exactly match the bytes written into the pcap record"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-004 (microsecond) / BC-2.01.002 postcondition 2 + BC-2.01.005 postcondition 1
//
// Each RawPacket has timestamp_secs == ts_sec and timestamp_usecs == ts_frac
// for a microsecond-resolution file.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_01_002_timestamp_preserved_microsecond() {
    // Canonical test vector from BC-2.01.005: ts_sec=1000, ts_frac=500
    let mut buf = us_header(1);
    append_packet(&mut buf, 1000, 500, DUMMY_ETH_FRAME);

    let source = PcapSource::from_pcap_reader(Cursor::new(&buf)).unwrap();
    assert_eq!(source.packets.len(), 1);
    assert_eq!(
        source.packets[0].timestamp_secs, 1000,
        "timestamp_secs must equal ts_sec"
    );
    assert_eq!(
        source.packets[0].timestamp_usecs, 500,
        "timestamp_usecs must equal ts_frac for microsecond-resolution file"
    );
}

// ---------------------------------------------------------------------------
// AC-004 (nanosecond) / BC-2.01.002 postcondition 2 + BC-2.01.005 postcondition 2
//
// For a nanosecond-resolution file, timestamp_usecs == ts_frac / 1_000.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_01_002_timestamp_preserved_nanosecond() {
    // ns magic: 0xa1b23c4d
    let mut buf = ns_header(1);
    // ts_frac = 900_000 ns → timestamp_usecs = 900 µs
    append_packet(&mut buf, 5000, 900_000, DUMMY_ETH_FRAME);

    let source = PcapSource::from_pcap_reader(Cursor::new(&buf)).unwrap();
    assert_eq!(source.packets.len(), 1);
    assert_eq!(source.packets[0].timestamp_secs, 5000);
    assert_eq!(
        source.packets[0].timestamp_usecs, 900,
        "ns ts_frac 900_000 must be divided by 1_000 to yield 900 µs"
    );
}

// ---------------------------------------------------------------------------
// AC-005 / BC-2.01.003 postcondition 1
//
// A pcap containing only the global header (zero packet records) returns
// Ok(PcapSource { packets: vec![], datalink }) without error or panic.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_01_003_zero_packet_pcap() {
    // Exactly 24 bytes: global header only (BC-2.01.003 EC-001)
    let buf = us_header(1); // ETHERNET, no packets
    assert_eq!(buf.len(), 24, "pre-condition: header is exactly 24 bytes");

    let result = PcapSource::from_pcap_reader(Cursor::new(&buf));
    assert!(result.is_ok(), "zero-packet pcap must not return Err");
    let source = result.unwrap();
    assert!(
        source.packets.is_empty(),
        "zero-packet pcap must yield empty Vec, got {} packets",
        source.packets.len()
    );
    assert_eq!(source.datalink, DataLink::ETHERNET);
}

// ---------------------------------------------------------------------------
// AC-006 / BC-2.01.004 postcondition 1
//
// A pcapng-format file returns Err with message containing
// "Failed to parse pcap header"; no packets are returned.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_01_004_rejects_pcapng() {
    // Use the existing smb3.pcapng fixture (BC-2.01.004 invariant 2).
    let path = Path::new("tests/fixtures/smb3.pcapng");
    assert!(
        path.exists(),
        "fixture tests/fixtures/smb3.pcapng must exist"
    );

    let result = PcapSource::from_file(path);
    assert!(result.is_err(), "pcapng input must return Err");
    let chain = format!("{:#}", result.unwrap_err());
    assert!(
        chain.contains("Failed to parse pcap header"),
        "error chain must contain 'Failed to parse pcap header', got: {chain}"
    );
}

// ---------------------------------------------------------------------------
// AC-007 / BC-2.01.005 postcondition 2 (nanosecond edge case)
//
// For a nanosecond-resolution pcap record with ts_frac = 500_000,
// the resulting RawPacket.timestamp_usecs equals 500 (integer division,
// sub-microsecond precision is discarded).
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_01_005_nanosecond_resolution_conversion() {
    // Canonical test vector from BC-2.01.005 EC-003 / story AC-007.
    let mut buf = ns_header(1); // nanosecond magic
    append_packet(&mut buf, 42, 500_000, DUMMY_ETH_FRAME);

    let source = PcapSource::from_pcap_reader(Cursor::new(&buf)).unwrap();
    assert_eq!(source.packets.len(), 1);
    assert_eq!(
        source.packets[0].timestamp_usecs, 500,
        "ts_frac 500_000 ns ÷ 1_000 must equal 500 µs (integer division)"
    );
}

// ---------------------------------------------------------------------------
// AC-008 / BC-2.01.006 postcondition 1
//
// Passing a zero-byte file or a file with invalid pcap magic bytes to
// from_file returns Err whose error chain contains "Failed to parse pcap header".
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_01_006_corrupt_header_error_message() {
    // Sub-case 1: empty byte slice (0 bytes, BC-2.01.006 EC-001)
    {
        let result = PcapSource::from_pcap_reader(Cursor::new([] as [u8; 0]));
        assert!(result.is_err(), "empty input must return Err");
        let chain = format!("{:#}", result.unwrap_err());
        assert!(
            chain.contains("Failed to parse pcap header"),
            "empty input error must contain 'Failed to parse pcap header', got: {chain}"
        );
    }

    // Sub-case 2: 10 garbage bytes (BC-2.01.006 test vector)
    {
        let garbage = [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55u8];
        let result = PcapSource::from_pcap_reader(Cursor::new(garbage));
        assert!(result.is_err(), "garbage bytes must return Err");
        let chain = format!("{:#}", result.unwrap_err());
        assert!(
            chain.contains("Failed to parse pcap header"),
            "garbage error must contain 'Failed to parse pcap header', got: {chain}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-009 / BC-2.01.007 postcondition 1 + invariant 1
//
// When a pcap has a valid header but a truncated packet record mid-stream,
// from_pcap_reader returns Err with context "Failed to read packet".
//
// BC-2.01.007 invariant 1 (all-or-nothing: previously-read packets are NOT
// returned as a partial Vec on error) is structurally guaranteed by the
// Result<PcapSource> return type: Err and Ok(partial) are mutually exclusive
// shapes, so there is no API surface through which a partial packet Vec could
// escape an Err return. This test does not and cannot make that invariant
// "more observable" — it is guaranteed by the type system regardless.
//
// The test writes one complete valid packet before the truncated record to
// exercise a realistic multi-packet mid-stream failure scenario (BC-2.01.007
// EC-001), and asserts postcondition 1: the call returns Err with the
// expected "Failed to read packet" context string.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_01_007_truncated_packet_error() {
    let mut buf = us_header(1); // ETHERNET

    // Packet 1: complete, valid record — this must NOT appear in any Ok result.
    append_packet(&mut buf, 100, 0, DUMMY_ETH_FRAME);

    // Packet 2: record header claims 20 bytes but only 4 are present.
    // BC-2.01.007 EC-001: file truncated in the middle of a packet record.
    let claimed_len: u32 = 20;
    buf.extend_from_slice(&200u32.to_le_bytes()); // ts_sec
    buf.extend_from_slice(&0u32.to_le_bytes()); // ts_frac
    buf.extend_from_slice(&claimed_len.to_le_bytes()); // incl_len = 20
    buf.extend_from_slice(&claimed_len.to_le_bytes()); // orig_len = 20
    buf.extend_from_slice(&[0xAAu8; 4]); // only 4 bytes, not 20 (truncated)

    let result = PcapSource::from_pcap_reader(Cursor::new(&buf));

    // The call must fail: the valid first packet is NOT returned as a partial result.
    assert!(
        result.is_err(),
        "truncated mid-stream packet must return Err, not Ok with partial data"
    );
    let chain = format!("{:#}", result.unwrap_err());
    assert!(
        chain.contains("Failed to read packet"),
        "error must contain 'Failed to read packet', got: {chain}"
    );
}

// ---------------------------------------------------------------------------
// AC-010 / BC-2.01.008 postcondition 2
//
// Calling from_file on a path that does not exist returns Err with context
// "Failed to open" and the path in the message.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_01_008_file_not_found_error() {
    let nonexistent = Path::new("/tmp/wirerust-test-does-not-exist-bc-2-01-008.pcap");
    // Ensure it really doesn't exist
    let _ = std::fs::remove_file(nonexistent);

    let result = PcapSource::from_file(nonexistent);
    assert!(result.is_err(), "non-existent path must return Err");

    let chain = format!("{:#}", result.unwrap_err());
    assert!(
        chain.contains("Failed to open"),
        "error must contain 'Failed to open', got: {chain}"
    );
    assert!(
        chain.contains("wirerust-test-does-not-exist-bc-2-01-008.pcap"),
        "error must contain the file path, got: {chain}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.01.008 EC-002 — file exists but is not readable (permission denied)
//
// F-004: from_file on an unreadable file must return Err with context
// "Failed to open". Guarded with #[cfg(unix)] because chmod 000 semantics
// are platform-specific and meaningless on Windows.
// ---------------------------------------------------------------------------
#[test]
#[cfg(unix)]
fn test_BC_2_01_008_permission_denied_error() {
    use std::os::unix::fs::PermissionsExt;

    // Create a real file so the path exists, then remove all permissions.
    let tmp = tempfile::NamedTempFile::new().expect("tempfile creation must succeed");
    let path = tmp.path().to_owned();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000))
        .expect("chmod 000 must succeed on the temp file");

    let result = PcapSource::from_file(&path);

    // Restore permissions before asserting so the NamedTempFile destructor
    // can clean up regardless of assertion outcome.
    let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));

    assert!(result.is_err(), "unreadable file must return Err");
    let chain = format!("{:#}", result.unwrap_err());
    assert!(
        chain.contains("Failed to open"),
        "permission-denied error must contain 'Failed to open', got: {chain}"
    );
}

// ---------------------------------------------------------------------------
// Story Task 8 / BC-2.01.001 verification property
//
// Property-based test: any 32-bit link-type value NOT in the 5-variant
// whitelist produces Err without panic. Generates 1000 cases. Uses proptest.
//
// The generator covers the full u32 range (0..=u32::MAX) because the pcap
// global header's network field is a 32-bit LE integer — any bit pattern is
// a valid raw input. The no-panic property must hold for all of them, not
// just the subset of known DataLink enum variants (0..=296) or a narrower
// range.
//
// The whitelist is {1=ETHERNET, 101=RAW, 113=LINUX_SLL, 228=IPV4, 229=IPV6}.
// ---------------------------------------------------------------------------
const WHITELIST: &[u32] = &[1, 101, 113, 228, 229];

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn test_BC_2_01_001_proptest_non_whitelist_link_type_rejected(
        link_type in 0u32..=u32::MAX
    ) {
        // Skip whitelisted types — those must succeed, not fail.
        prop_assume!(!WHITELIST.contains(&link_type));

        let buf = us_header(link_type);
        let result = PcapSource::from_pcap_reader(Cursor::new(buf));

        // Must return Err — the proptest runner also catches panics as failures,
        // so a panic on any generated value would surface here.
        prop_assert!(
            result.is_err(),
            "link type {link_type} (arbitrary u32, not in whitelist) returned Ok"
        );
        // The error must come from the link-type rejection branch, not from
        // an incidental header-parse failure. Without this assertion, any Err
        // (including one from a corrupt-header path) would satisfy the check
        // above, leaving the ingestion gate untested for large link-type values.
        let chain = format!("{:#}", result.unwrap_err());
        prop_assert!(
            chain.contains("Unsupported pcap link type"),
            "link type {link_type} rejected but not by the link-type guard: {chain}"
        );
    }

    #[test]
    fn test_BC_2_01_001_proptest_whitelist_link_type_accepted(
        link_type in proptest::sample::select(vec![1u32, 101u32, 113u32, 228u32, 229u32])
    ) {
        let buf = us_header(link_type);
        let result = PcapSource::from_pcap_reader(Cursor::new(buf));
        prop_assert!(
            result.is_ok(),
            "whitelisted link type {link_type} returned Err: {:?}",
            result.unwrap_err()
        );
    }
}

// ---------------------------------------------------------------------------
// Additional edge-case tests derived from the BC edge-case catalogs
// ---------------------------------------------------------------------------

/// EC-005 from STORY-001: Packet ts_sec = u32::MAX is stored as-is.
/// (BC-2.01.002 EC-006, BC-2.01.005 EC-002)
#[test]
fn test_BC_2_01_005_ts_sec_u32_max_stored_as_is() {
    let mut buf = us_header(1);
    append_packet(&mut buf, u32::MAX, 0, DUMMY_ETH_FRAME);

    let source = PcapSource::from_pcap_reader(Cursor::new(&buf)).unwrap();
    assert_eq!(source.packets.len(), 1);
    assert_eq!(
        source.packets[0].timestamp_secs,
        u32::MAX,
        "ts_sec u32::MAX must be stored without wrapping or error"
    );
}

/// EC-008 from STORY-001: RAW and IPV4 are both accepted individually.
/// (BC-2.01.001 EC-004)
#[test]
fn test_BC_2_01_001_raw_and_ipv4_both_accepted() {
    for (num, expected) in [(101u32, DataLink::RAW), (228u32, DataLink::IPV4)] {
        let buf = us_header(num);
        let source = PcapSource::from_pcap_reader(Cursor::new(&buf)).unwrap();
        assert_eq!(
            source.datalink, expected,
            "link type {num} must be accepted as {expected:?}"
        );
    }
}

/// Zero-packet pcap with LINUX_SLL link type must still be accepted.
/// (BC-2.01.003 EC-002)
#[test]
fn test_BC_2_01_003_zero_packet_linux_sll() {
    let buf = us_header(113); // LINUX_SLL
    let source = PcapSource::from_pcap_reader(Cursor::new(&buf)).unwrap();
    assert!(source.packets.is_empty());
    assert_eq!(source.datalink, DataLink::LINUX_SLL);
}

/// Nanosecond-resolution zero-fractional-second: ts_frac = 0 → timestamp_usecs = 0.
/// (BC-2.01.005 test vector: ts_sec=0, ts_frac=0)
#[test]
fn test_BC_2_01_005_zero_timestamp_nanosecond() {
    let mut buf = ns_header(1);
    append_packet(&mut buf, 0, 0, DUMMY_ETH_FRAME);

    let source = PcapSource::from_pcap_reader(Cursor::new(&buf)).unwrap();
    assert_eq!(source.packets[0].timestamp_secs, 0);
    assert_eq!(source.packets[0].timestamp_usecs, 0);
}

/// Truncated pcap global header — valid magic but < 24 bytes total — must return
/// "Failed to parse pcap header". (BC-2.01.006 EC-002)
///
/// This is DISTINCT from `test_BC_2_01_006_corrupt_header_error_message`:
/// - That test exercises wrong/garbage magic bytes (the header is invalid from byte 0).
/// - This test exercises a header that STARTS with the correct classic-pcap magic
///   (0xa1b2c3d4 LE) and valid subsequent fields, but is cut short before the
///   required 24 bytes are complete. The pcap_file crate recognises the magic and
///   attempts to parse the remaining header fields but finds insufficient bytes,
///   producing a parse error that is distinct in origin from a magic-mismatch.
#[test]
fn test_BC_2_01_006_truncated_header_error_message() {
    // Build 17 bytes: the full microsecond LE magic (4 bytes) + version major/minor
    // (4 bytes) + thiszone (4 bytes) + sigfigs (4 bytes) + the first byte of the
    // snaplen field — stopping 7 bytes short of the complete 24-byte global header.
    // The magic is valid classic-pcap (0xa1b2c3d4); the truncation happens mid-header.
    let mut partial = Vec::with_capacity(17);
    partial.extend_from_slice(&0xa1b2c3d4u32.to_le_bytes()); // microsecond LE magic
    partial.extend_from_slice(&2u16.to_le_bytes()); // version major = 2
    partial.extend_from_slice(&4u16.to_le_bytes()); // version minor = 4
    partial.extend_from_slice(&0i32.to_le_bytes()); // thiszone = 0
    partial.extend_from_slice(&0u32.to_le_bytes()); // sigfigs = 0
    partial.extend_from_slice(&0xFFu8.to_le_bytes()); // first byte of snaplen — truncated
    assert_eq!(
        partial.len(),
        17,
        "pre-condition: input is 17 bytes, 7 short of a complete header"
    );

    let result = PcapSource::from_pcap_reader(Cursor::new(&partial));
    assert!(
        result.is_err(),
        "valid-magic-but-truncated header must return Err"
    );
    let chain = format!("{:#}", result.unwrap_err());
    assert!(
        chain.contains("Failed to parse pcap header"),
        "truncated-valid-magic error must contain 'Failed to parse pcap header', got: {chain}"
    );
}

/// from_file on a valid pcap file returns the same result as from_pcap_reader
/// on the same bytes. (BC-2.01.008 postcondition 1 — delegation equivalence)
#[test]
fn test_BC_2_01_008_from_file_delegates_to_from_pcap_reader() {
    // Use an existing, known-good fixture.
    let path = Path::new("tests/fixtures/tls.pcap");
    assert!(path.exists(), "tls.pcap fixture must exist");

    let from_file_result = PcapSource::from_file(path).unwrap();

    let bytes = std::fs::read(path).unwrap();
    let from_reader_result = PcapSource::from_pcap_reader(Cursor::new(&bytes)).unwrap();

    assert_eq!(
        from_file_result.packets.len(),
        from_reader_result.packets.len(),
        "from_file and from_pcap_reader must yield the same packet count"
    );
    // BC-2.01.008 PC1: datalink is part of the returned struct and must match.
    assert_eq!(
        from_file_result.datalink, from_reader_result.datalink,
        "from_file and from_pcap_reader must yield the same datalink"
    );
    for (i, (a, b)) in from_file_result
        .packets
        .iter()
        .zip(from_reader_result.packets.iter())
        .enumerate()
    {
        assert_eq!(
            a.timestamp_secs, b.timestamp_secs,
            "packet {i}: timestamp_secs must match between from_file and from_pcap_reader"
        );
        assert_eq!(
            a.timestamp_usecs, b.timestamp_usecs,
            "packet {i}: timestamp_usecs must match"
        );
        assert_eq!(a.data, b.data, "packet {i}: data bytes must match");
    }
}
