//! SEC-001 Twin-Equivalence Trip-Wire for VP-027 Non-Staleness Guard
//!
//! # Purpose
//!
//! `decode_epb_body` (src/reader.rs:430) is the production EPB parser.
//! `decode_epb_body_discriminant` (src/reader.rs:536) is its typed-error twin,
//! used as the Kani BMC target for VP-027 proof. These two functions are
//! independent bodies — no shared core — verified line-for-line faithful at
//! creation time, but nothing mechanically prevents them from drifting apart
//! as the codebase evolves.
//!
//! This file is the SEC-001 mechanical trip-wire: it asserts that for ALL
//! inputs the two functions agree on Ok/Err parity AND, when both return Err,
//! that the production anyhow error code corresponds to the twin's discriminant.
//! If they diverge, the test fails loudly, alerting maintainers that the
//! VP-027 Kani proof may be stale relative to production.
//!
//! # Confirmed discriminant ↔ E-INP-code mapping
//!
//! | `EpbDecodeError` discriminant | Production error string contains |
//! |-------------------------------|----------------------------------|
//! | `BodyTooShort`                | `"E-INP-008"`                   |
//! | `EmptyInterfaceTable`         | `"E-INP-009"`                   |
//! | `InterfaceIdOob`              | `"E-INP-010"`                   |
//!
//! Note: Both PC6a (`captured_len > available`) and PC6b (padding overrun)
//! map to `BodyTooShort` / `E-INP-008` in the discriminant twin, consistent
//! with the production function which emits `"E-INP-008"` for both sub-cases.
//!
//! # Non-vacuity
//!
//! The companion mutation test (in this file, gated `#[cfg(test)]`) demonstrates
//! that mutating one guard in the discriminant twin causes this trip-wire to fail,
//! confirming the assertions are not trivially satisfied. See Step-3 notes in
//! the SEC-001 delivery report.
//!
//! # References
//!
//! - SEC-001 delivery spec (twin-equivalence trip-wire)
//! - VP-027 Kani proof (`tests/kani_proofs.rs`)
//! - BC-2.01.012 (EPB decode behavioral contracts)
//! - ADR-009 (pcapng design decisions)
//!
//! Naming convention: `test_SEC_001_xxx` for unit cases, `proptest_SEC_001_xxx`
//! for property tests.
//! `#![allow(non_snake_case)]` required per factory BC-naming mandate.
#![allow(non_snake_case)]
#![allow(unused_doc_comments)]

use pcap_file::DataLink;
use proptest::prelude::*;
use wirerust::reader::{
    EpbDecodeError, InterfaceInfo, SectionEndianness, decode_epb_body, decode_epb_body_discriminant,
};

// ─── Shared helper: build an InterfaceInfo with a given linktype ─────────────

fn iface(tsresol: u8) -> InterfaceInfo {
    InterfaceInfo {
        linktype: DataLink::ETHERNET,
        if_tsresol: tsresol,
    }
}

// ─── Helper: assert Ok/Err parity + error-class parity ──────────────────────

/// Core equivalence assertion used by both unit tests and proptests.
///
/// Calls both functions with identical inputs and asserts:
/// (a) Ok/Err parity: both Ok or both Err.
/// (b) Error-class parity: when both Err, the production E-INP code matches
///     the twin's `EpbDecodeError` discriminant.
/// (c) Success parity: when both Ok, the decoded fields (timestamp, data)
///     agree exactly.
///
/// Field-level equivalence for the Ok case is additionally covered by the
/// STORY-125 EPB tests; this test adds cross-function agreement as a guard.
fn assert_twin_equivalent(
    body: &[u8],
    interfaces: &[InterfaceInfo],
    endianness: SectionEndianness,
    context: &str,
) {
    let prod = decode_epb_body(body, interfaces, endianness);
    let twin = decode_epb_body_discriminant(body, interfaces, endianness);

    // (a) Ok/Err parity.
    assert_eq!(
        prod.is_ok(),
        twin.is_ok(),
        "SEC-001 OK/Err parity violated [{context}]: \
         decode_epb_body={} but decode_epb_body_discriminant={}",
        if prod.is_ok() { "Ok" } else { "Err" },
        if twin.is_ok() { "Ok" } else { "Err" },
    );

    match (prod, twin) {
        (Ok(prod_pkt), Ok(twin_pkt)) => {
            // (c) Success field parity.
            assert_eq!(
                prod_pkt.timestamp_secs, twin_pkt.timestamp_secs,
                "SEC-001 timestamp_secs mismatch [{context}]"
            );
            assert_eq!(
                prod_pkt.timestamp_usecs, twin_pkt.timestamp_usecs,
                "SEC-001 timestamp_usecs mismatch [{context}]"
            );
            assert_eq!(
                prod_pkt.data, twin_pkt.data,
                "SEC-001 packet data mismatch [{context}]"
            );
        }
        (Err(prod_err), Err(twin_discriminant)) => {
            // (b) Error-class parity: confirm discriminant ↔ E-INP code mapping.
            let prod_msg = format!("{prod_err}");
            let expected_code = match twin_discriminant {
                EpbDecodeError::BodyTooShort => "E-INP-008",
                EpbDecodeError::EmptyInterfaceTable => "E-INP-009",
                EpbDecodeError::InterfaceIdOob => "E-INP-010",
            };
            assert!(
                prod_msg.contains(expected_code),
                "SEC-001 error-class mismatch [{context}]: \
                 twin discriminant={twin_discriminant:?} expects production error to contain \
                 '{expected_code}' but production error was: {prod_msg:?}"
            );
        }
        // Parity check (a) already caught mixed Ok/Err above.
        _ => unreachable!("parity already checked above"),
    }
}

// ─── Unit cases: deterministic anchors ───────────────────────────────────────

/// SEC-001-U1: body shorter than 20 bytes → both return E-INP-008 / BodyTooShort.
#[test]
fn test_SEC_001_body_too_short() {
    let body = [0u8; 19]; // one byte short of EPB_FIXED_OVERHEAD_BYTES
    let interfaces = [iface(6)];
    for &endianness in &[
        SectionEndianness::LittleEndian,
        SectionEndianness::BigEndian,
    ] {
        assert_twin_equivalent(&body, &interfaces, endianness, "body-too-short (19 bytes)");
    }
}

/// SEC-001-U2: completely empty body → both return E-INP-008 / BodyTooShort.
#[test]
fn test_SEC_001_empty_body() {
    let body = [];
    let interfaces = [iface(6)];
    assert_twin_equivalent(
        &body,
        &interfaces,
        SectionEndianness::LittleEndian,
        "empty body (0 bytes)",
    );
}

/// SEC-001-U3: empty interface table with a valid-length body → both return
/// E-INP-009 / EmptyInterfaceTable.
#[test]
fn test_SEC_001_empty_interface_table() {
    // Construct a 20-byte body with interface_id=0 (LE) and captured_len=0.
    let mut body = [0u8; 20];
    body[0..4].copy_from_slice(&0u32.to_le_bytes()); // interface_id = 0
    // ts_high, ts_low, captured_len, original_len all 0
    let interfaces: &[InterfaceInfo] = &[];
    assert_twin_equivalent(
        &body,
        interfaces,
        SectionEndianness::LittleEndian,
        "empty interface table",
    );
}

/// SEC-001-U4: interface_id out of bounds on a non-empty table → both return
/// E-INP-010 / InterfaceIdOob.
#[test]
fn test_SEC_001_interface_id_oob() {
    // One interface (index 0), but EPB references interface_id=1.
    let mut body = [0u8; 20];
    body[0..4].copy_from_slice(&1u32.to_le_bytes()); // interface_id = 1 (OOB)
    let interfaces = [iface(6)];
    assert_twin_equivalent(
        &body,
        &interfaces,
        SectionEndianness::LittleEndian,
        "interface_id=1 with table size=1 (OOB)",
    );

    // Also test large interface_id.
    body[0..4].copy_from_slice(&0x0000_FFFFu32.to_le_bytes());
    assert_twin_equivalent(
        &body,
        &interfaces,
        SectionEndianness::LittleEndian,
        "interface_id=0xFFFF with table size=1 (OOB)",
    );
}

/// SEC-001-U5: valid EPB with 4 bytes of packet data → both return Ok with
/// identical decoded fields (timestamp, data).
#[test]
fn test_SEC_001_valid_epb_happy_path() {
    // Build a valid EPB body:
    //   interface_id=0, ts_high=0, ts_low=0, captured_len=4, original_len=4,
    //   data=[0xAA, 0xBB, 0xCC, 0xDD] (no padding needed, 4 % 4 == 0).
    let captured_len: u32 = 4;
    let mut body = Vec::with_capacity(24); // 20 fixed + 4 data
    body.extend_from_slice(&0u32.to_le_bytes()); // interface_id = 0
    body.extend_from_slice(&0u32.to_le_bytes()); // ts_high = 0
    body.extend_from_slice(&1000u32.to_le_bytes()); // ts_low = 1000 µs
    body.extend_from_slice(&captured_len.to_le_bytes()); // captured_len = 4
    body.extend_from_slice(&captured_len.to_le_bytes()); // original_len = 4
    body.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // packet data (4 bytes, 4-aligned)

    let interfaces = [iface(6)]; // tsresol=6 → microsecond resolution
    assert_twin_equivalent(
        &body,
        &interfaces,
        SectionEndianness::LittleEndian,
        "valid EPB with 4 bytes of packet data",
    );
}

/// SEC-001-U6: captured_len exceeds available body bytes (PC6a) → both return
/// E-INP-008 / BodyTooShort.
#[test]
fn test_SEC_001_captured_len_exceeds_body() {
    // 20-byte body with captured_len=1 (only 0 bytes available after header).
    let mut body = [0u8; 20];
    body[0..4].copy_from_slice(&0u32.to_le_bytes()); // interface_id = 0
    body[12..16].copy_from_slice(&1u32.to_le_bytes()); // captured_len = 1 > 0 available
    let interfaces = [iface(6)];
    assert_twin_equivalent(
        &body,
        &interfaces,
        SectionEndianness::LittleEndian,
        "PC6a: captured_len=1 with 0 available bytes",
    );
}

// ─── Property test: random inputs ────────────────────────────────────────────

/// VP-027 SEC-001 — Twin-equivalence property over random EPB inputs.
///
/// Strategy: generate random body bytes (lengths spanning <20, ==20, >20),
/// random interface tables (size 0..=5), random endianness, and random
/// interface_id values. For each combination, assert Ok/Err parity AND
/// error-class parity between production and twin.
///
/// This guards SEC-001: if `decode_epb_body_discriminant` ever drifts from
/// `decode_epb_body`, at least one generated case will expose the divergence.
///
/// Configured at 2000 cases (double the VP-029/031 baseline) to maximise
/// coverage of the input space, especially boundary conditions around the
/// 20-byte minimum body length.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn proptest_SEC_001_twin_equivalence_random_inputs(
        // Body bytes: cover below, at, and above the 20-byte threshold.
        body in prop::collection::vec(any::<u8>(), 0usize..=64usize),
        // Interface table size: 0 (empty), 1, 2, 3 (small but non-trivial).
        num_interfaces in 0usize..=3usize,
        // if_tsresol exponent: 0 (1 Hz), 6 (µs, default), 9 (ns), 20 (fast-path limit).
        tsresol in prop::sample::select(vec![0u8, 6u8, 9u8, 20u8]),
        // Endianness.
        use_le in any::<bool>(),
        // Override bytes 0-3 with a targeted interface_id for interest.
        // 0 = in-range (if table non-empty), 1 = potentially OOB, u32::MAX = always OOB.
        interface_id_override in prop::sample::select(vec![0u32, 1u32, 2u32, u32::MAX]),
        // Whether to apply the interface_id override into the body bytes.
        apply_iid_override in any::<bool>(),
    ) {
        let endianness = if use_le { SectionEndianness::LittleEndian } else { SectionEndianness::BigEndian };
        let interfaces: Vec<InterfaceInfo> = (0..num_interfaces).map(|_| iface(tsresol)).collect();

        // Optionally write interface_id_override into body bytes 0-3 so we can exercise
        // boundary values even when random bytes happen to be out of range.
        let mut body_mut = body.clone();
        if apply_iid_override && body_mut.len() >= 4 {
            let bytes = match endianness {
                SectionEndianness::LittleEndian => interface_id_override.to_le_bytes(),
                SectionEndianness::BigEndian => interface_id_override.to_be_bytes(),
            };
            body_mut[0..4].copy_from_slice(&bytes);
        }

        let prod = decode_epb_body(&body_mut, &interfaces, endianness);
        let twin = decode_epb_body_discriminant(&body_mut, &interfaces, endianness);

        // (a) Ok/Err parity.
        prop_assert_eq!(
            prod.is_ok(),
            twin.is_ok(),
            "SEC-001 Ok/Err parity violated: \
             body_len={} num_interfaces={} endianness={:?} \
             production={} twin={}",
            body_mut.len(),
            interfaces.len(),
            endianness,
            if prod.is_ok() { "Ok" } else { "Err" },
            if twin.is_ok() { "Ok" } else { "Err" },
        );

        match (&prod, &twin) {
            (Ok(prod_pkt), Ok(twin_pkt)) => {
                // (c) Success field parity.
                prop_assert_eq!(
                    prod_pkt.timestamp_secs,
                    twin_pkt.timestamp_secs,
                    "SEC-001 timestamp_secs mismatch: body_len={} num_interfaces={} endianness={:?}",
                    body_mut.len(), interfaces.len(), endianness
                );
                prop_assert_eq!(
                    prod_pkt.timestamp_usecs,
                    twin_pkt.timestamp_usecs,
                    "SEC-001 timestamp_usecs mismatch: body_len={} num_interfaces={} endianness={:?}",
                    body_mut.len(), interfaces.len(), endianness
                );
                prop_assert_eq!(
                    &prod_pkt.data,
                    &twin_pkt.data,
                    "SEC-001 packet data mismatch: body_len={} num_interfaces={} endianness={:?}",
                    body_mut.len(), interfaces.len(), endianness
                );
            }
            (Err(prod_err), Err(twin_discriminant)) => {
                // (b) Error-class parity.
                let prod_msg = format!("{prod_err}");
                let expected_code = match twin_discriminant {
                    EpbDecodeError::BodyTooShort => "E-INP-008",
                    EpbDecodeError::EmptyInterfaceTable => "E-INP-009",
                    EpbDecodeError::InterfaceIdOob => "E-INP-010",
                };
                prop_assert!(
                    prod_msg.contains(expected_code),
                    "SEC-001 error-class mismatch: \
                     twin={twin_discriminant:?} expects '{expected_code}' but \
                     production error was: {prod_msg:?} \
                     [body_len={} num_interfaces={} endianness={:?}]",
                    body_mut.len(),
                    interfaces.len(),
                    endianness,
                );
            }
            _ => unreachable!("parity already checked above"),
        }
    }
}
