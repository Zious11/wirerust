//! SHB Twin-Equivalence Trip-Wire for VP-026 Non-Staleness Guard
//!
//! # Purpose
//!
//! `parse_shb_body` (src/reader.rs) is the production SHB body parser.
//! `parse_shb_body_discriminant` (src/reader.rs) is its typed-error twin,
//! used as the Kani BMC target for the VP-026 proof. The twin is a verbatim
//! lift of the production decode logic (same guard order, same BOM table, same
//! `major_version == 1` gate) differing ONLY in the error channel: it returns
//! `ShbDecodeError` instead of an `anyhow::Error` string, so Kani's symbolic
//! state stays tractable.
//!
//! Like the VP-027 SEC-001 trip-wire (`tests/sec_001_twin_equivalence_tests.rs`),
//! this file mechanically guards against twin drift: it asserts that for ALL
//! inputs the two functions agree on Ok/Err parity, that every twin Err
//! discriminant maps to the production `E-INP-008` code, and that on success
//! the decoded `ShbInfo` fields (endianness, major/minor version) agree exactly.
//! If they diverge, the test fails loudly, alerting maintainers that the VP-026
//! Kani proof may be stale relative to production.
//!
//! # Confirmed discriminant ↔ E-INP-code mapping
//!
//! | `ShbDecodeError` discriminant | Production error string contains |
//! |-------------------------------|----------------------------------|
//! | `BodyTooShort`                | `"E-INP-008"`                   |
//! | `InvalidBom`                  | `"E-INP-008"`                   |
//! | `UnsupportedVersion`          | `"E-INP-008"`                   |
//!
//! All three SHB error sub-cases are E-INP-008 in production (BC-2.01.010 PC5b /
//! PC1 / PC2); the twin's three distinct discriminants exist so the Kani proof
//! can assert *which* guard fired, not to introduce new error codes.
//!
//! # References
//!
//! - VP-026 Kani proof (`reader::kani_proofs::vp026_shb_parse_safety`)
//! - BC-2.01.010 (SHB parse safety + byte-order detection)
//! - ADR-009 (pcapng design decisions)
//! - SEC-001 trip-wire (`tests/sec_001_twin_equivalence_tests.rs`) — sibling pattern
#![allow(non_snake_case)]
#![allow(unused_doc_comments)]

use proptest::prelude::*;
use wirerust::reader::{
    SectionEndianness, ShbDecodeError, ShbInfo, parse_shb_body, parse_shb_body_discriminant,
};

// ─── Helper: assert Ok/Err parity + error-class parity + success parity ─────

fn assert_shb_twin_equivalent(body: &[u8], context: &str) {
    let prod: anyhow::Result<ShbInfo> = parse_shb_body(body);
    let twin: Result<ShbInfo, ShbDecodeError> = parse_shb_body_discriminant(body);

    // (a) Ok/Err parity.
    assert_eq!(
        prod.is_ok(),
        twin.is_ok(),
        "SHB-twin Ok/Err parity violated [{context}]: parse_shb_body={} but \
         parse_shb_body_discriminant={}",
        if prod.is_ok() { "Ok" } else { "Err" },
        if twin.is_ok() { "Ok" } else { "Err" },
    );

    match (prod, twin) {
        (Ok(prod_info), Ok(twin_info)) => {
            // (c) Success field parity.
            assert_eq!(
                prod_info.endianness, twin_info.endianness,
                "SHB-twin endianness mismatch [{context}]"
            );
            assert_eq!(
                prod_info.major_version, twin_info.major_version,
                "SHB-twin major_version mismatch [{context}]"
            );
            assert_eq!(
                prod_info.minor_version, twin_info.minor_version,
                "SHB-twin minor_version mismatch [{context}]"
            );
        }
        (Err(prod_err), Err(twin_discriminant)) => {
            // (b) Error-class parity: all three SHB discriminants → E-INP-008.
            let prod_msg = format!("{prod_err}");
            let expected_code = match twin_discriminant {
                ShbDecodeError::BodyTooShort => "E-INP-008",
                ShbDecodeError::InvalidBom => "E-INP-008",
                ShbDecodeError::UnsupportedVersion => "E-INP-008",
            };
            assert!(
                prod_msg.contains(expected_code),
                "SHB-twin error-class mismatch [{context}]: twin discriminant={twin_discriminant:?} \
                 expects production error to contain '{expected_code}' but production error was: \
                 {prod_msg:?}"
            );
        }
        _ => unreachable!("parity already checked above"),
    }
}

// ─── Helper: build a 16-byte SHB body ───────────────────────────────────────

/// Build a canonical SHB body: BOM, major, minor (in BOM byte order), section_length.
fn shb_body(bom: [u8; 4], major: u16, minor: u16, big_endian: bool) -> Vec<u8> {
    let mut b = Vec::with_capacity(16);
    b.extend_from_slice(&bom);
    if big_endian {
        b.extend_from_slice(&major.to_be_bytes());
        b.extend_from_slice(&minor.to_be_bytes());
    } else {
        b.extend_from_slice(&major.to_le_bytes());
        b.extend_from_slice(&minor.to_le_bytes());
    }
    b.extend_from_slice(&[0u8; 8]); // section_length (ignored)
    b
}

const BOM_BIG: [u8; 4] = [0x1A, 0x2B, 0x3C, 0x4D];
const BOM_LITTLE: [u8; 4] = [0x4D, 0x3C, 0x2B, 0x1A];

// ─── Unit cases: deterministic anchors ──────────────────────────────────────

/// Body shorter than 16 bytes → both Err / BodyTooShort / E-INP-008.
#[test]
fn test_SHB_twin_body_too_short() {
    for len in [0usize, 1, 8, 15] {
        let body = vec![0u8; len];
        assert_shb_twin_equivalent(&body, "body-too-short");
    }
}

/// 16-byte body with a non-canonical BOM → both Err / InvalidBom / E-INP-008.
#[test]
fn test_SHB_twin_invalid_bom() {
    let body = shb_body([0xDE, 0xAD, 0xBE, 0xEF], 1, 0, false);
    assert_shb_twin_equivalent(&body, "invalid-bom");
}

/// Big-endian BOM, major==1 → both Ok with BigEndian + major 1.
#[test]
fn test_SHB_twin_valid_big_endian() {
    let body = shb_body(BOM_BIG, 1, 0, true);
    assert_shb_twin_equivalent(&body, "valid-big-endian");
    let info = parse_shb_body_discriminant(&body).expect("valid");
    assert_eq!(info.endianness, SectionEndianness::BigEndian);
    assert_eq!(info.major_version, 1);
}

/// Little-endian BOM, major==1, minor==42 → both Ok with LittleEndian + minor 42.
#[test]
fn test_SHB_twin_valid_little_endian() {
    let body = shb_body(BOM_LITTLE, 1, 42, false);
    assert_shb_twin_equivalent(&body, "valid-little-endian");
    let info = parse_shb_body_discriminant(&body).expect("valid");
    assert_eq!(info.endianness, SectionEndianness::LittleEndian);
    assert_eq!(info.minor_version, 42);
}

/// Valid BOM but major != 1 → both Err / UnsupportedVersion / E-INP-008.
#[test]
fn test_SHB_twin_unsupported_version() {
    for major in [0u16, 2, 0xFFFF] {
        let le = shb_body(BOM_LITTLE, major, 0, false);
        assert_shb_twin_equivalent(&le, "unsupported-version-le");
        let be = shb_body(BOM_BIG, major, 0, true);
        assert_shb_twin_equivalent(&be, "unsupported-version-be");
    }
}

// ─── Property test: random inputs ───────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn proptest_SHB_twin_equivalence_random_inputs(
        // Body bytes spanning <16, ==16, and >16.
        body in prop::collection::vec(any::<u8>(), 0usize..=40usize),
        // Force a canonical BOM into bytes 0-3 some of the time so valid paths are exercised.
        bom_choice in prop::sample::select(vec![0u8, 1u8, 2u8]),
        // Targeted major version to drive the gate.
        major_override in prop::sample::select(vec![0u16, 1u16, 2u16, 0xFFFFu16]),
        apply_overrides in any::<bool>(),
        use_be in any::<bool>(),
    ) {
        let mut body_mut = body.clone();
        if apply_overrides && body_mut.len() >= 16 {
            let bom = match bom_choice {
                0 => BOM_LITTLE,
                1 => BOM_BIG,
                _ => [0x00, 0x11, 0x22, 0x33], // non-canonical
            };
            body_mut[0..4].copy_from_slice(&bom);
            let big = bom == BOM_BIG;
            let major_bytes = if big { major_override.to_be_bytes() } else { major_override.to_le_bytes() };
            body_mut[4..6].copy_from_slice(&major_bytes);
            let _ = use_be; // endianness is BOM-derived in the decode path
        }

        let prod = parse_shb_body(&body_mut);
        let twin = parse_shb_body_discriminant(&body_mut);

        // (a) Ok/Err parity.
        prop_assert_eq!(
            prod.is_ok(),
            twin.is_ok(),
            "SHB-twin Ok/Err parity violated: body_len={} production={} twin={}",
            body_mut.len(),
            if prod.is_ok() { "Ok" } else { "Err" },
            if twin.is_ok() { "Ok" } else { "Err" },
        );

        match (&prod, &twin) {
            (Ok(prod_info), Ok(twin_info)) => {
                prop_assert_eq!(prod_info.endianness, twin_info.endianness,
                    "SHB-twin endianness mismatch: body_len={}", body_mut.len());
                prop_assert_eq!(prod_info.major_version, twin_info.major_version,
                    "SHB-twin major mismatch: body_len={}", body_mut.len());
                prop_assert_eq!(prod_info.minor_version, twin_info.minor_version,
                    "SHB-twin minor mismatch: body_len={}", body_mut.len());
            }
            (Err(prod_err), Err(twin_discriminant)) => {
                let prod_msg = format!("{prod_err}");
                let expected_code = match twin_discriminant {
                    ShbDecodeError::BodyTooShort => "E-INP-008",
                    ShbDecodeError::InvalidBom => "E-INP-008",
                    ShbDecodeError::UnsupportedVersion => "E-INP-008",
                };
                prop_assert!(
                    prod_msg.contains(expected_code),
                    "SHB-twin error-class mismatch: twin={twin_discriminant:?} expects \
                     '{expected_code}' but production error was: {prod_msg:?} [body_len={}]",
                    body_mut.len(),
                );
            }
            _ => unreachable!("parity already checked above"),
        }
    }
}
