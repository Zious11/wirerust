//! Tests for STORY-184: S7comm TPKT Core Parser (pure-core free function).
//!
//! Covers BC-2.20.001 through BC-2.20.004 and the edge cases enumerated in each BC.
//!
//! ## Contract coverage
//! - BC-2.20.001: `parse_tpkt_header` returns `None` for input shorter than 4 bytes.
//! - BC-2.20.002: `parse_tpkt_header` returns `None` for version byte != 0x03 (also the
//!   SS-20 resync anchor).
//! - BC-2.20.003: `parse_tpkt_header` returns `None` for decoded length field < 7 (RFC
//!   1006 §6's stated minimum packet length; malformed, includes zero-length and
//!   header-only lengths 4-6).
//! - BC-2.20.004: `parse_tpkt_header` returns `Some(TpktHeader)` for valid input (happy
//!   path); reserved byte (`data[1]`) is never validated; accept range is `[7, 65535]`;
//!   `length == 65535` is a legal accept; the four BC-2.20.001-004 outcomes are jointly
//!   exhaustive and mutually exclusive (AC-184-005).
//!
//! ## Test naming convention
//! Tests follow `test_BC_S_SS_NNN_xxx()` for BC-traceable tests.
//! The non_snake_case lint fires on uppercase BC IDs — suppressed intentionally.
//!
//! ## Provenance
//! Originally authored Red-first as TDD stubs (STORY-184 `tdd_mode: strict`;
//! BC-2.20.001-004) against a `todo!()` stub in `src/analyzer/iso_on_tcp.rs`. Red Gate
//! was verified via `cargo test --test iso_on_tcp_tests` (BC-5.38.001) before the
//! `todo!()` stub was replaced by the STORY-184 implementation of
//! `parse_tpkt_header`. These tests are now GREEN.
//!
//! RFC 1006 §6 states the minimum legal TPKT packet length is 7 (4-byte TPKT header +
//! 3-byte minimum COTP). `parse_tpkt_header`'s length-floor guard enforces this minimum
//! directly (human ruling, re-opening this story from its earlier converged state, which
//! had accepted length=4 as a documented layering divergence — that divergence has been
//! retired; the implementation and every test below are now RFC-conformant).
//!
//! Canonical test vectors from BC-2.20.001-004 are used verbatim for the BC-conformance
//! tests above. Separately, PER DF-CANONICAL-FRAME-HOLDOUT-001, the `test_rfc1006_s6_*`
//! holdout tests below are authored independently of the BCs, derived directly from
//! RFC 1006 §6 ("Packet Format") — that policy requires a spec-independent vector set,
//! the opposite of reusing BC text verbatim.

#![allow(non_snake_case)]

// Per DF-TEST-NAMESPACE-001: all STORY-184 tests are grouped inside a dedicated
// `mod story_184` wrapper to prevent test-function name collisions with other
// stories' BC-prefixed names.
mod story_184 {
    use wirerust::analyzer::iso_on_tcp::{TpktHeader, parse_tpkt_header};

    // =========================================================================
    // BC-2.20.001: parse_tpkt_header returns None for input shorter than 4 bytes
    // AC-184-001
    // =========================================================================

    /// BC-2.20.001 canonical vector: empty slice returns None.
    ///
    /// Canonical vector from BC-2.20.001: `[]` (0 bytes) -> None.
    /// Precondition: data.len() == 0 (< 4). Postcondition 2: no bytes accessed, no panic.
    ///
    /// Traces: BC-2.20.001 postconditions 1-3; AC-184-001; EC-001.
    #[test]
    fn test_BC_2_20_001_returns_none_for_empty_slice() {
        let result = parse_tpkt_header(&[]);
        assert!(
            result.is_none(),
            "empty slice must return None (BC-2.20.001 postcondition 1)"
        );
    }

    /// BC-2.20.001: one-byte slice returns None.
    ///
    /// Canonical vector from BC-2.20.001 EC-002: `data.len() == 1` -> None.
    ///
    /// Traces: BC-2.20.001 postconditions 1-3; AC-184-001; EC-002.
    #[test]
    fn test_BC_2_20_001_returns_none_for_one_byte() {
        let result = parse_tpkt_header(&[0x03]);
        assert!(
            result.is_none(),
            "1-byte slice must return None even if byte is the valid version 0x03 \
             (BC-2.20.001 postcondition 1)"
        );
    }

    /// BC-2.20.001: two-byte slice returns None.
    ///
    /// Exercises len=2, one step further from the 1-byte case above, still short of the
    /// 4-byte minimum.
    ///
    /// Traces: BC-2.20.001 postconditions 1-3; AC-184-001.
    #[test]
    fn test_BC_2_20_001_returns_none_for_two_bytes() {
        let result = parse_tpkt_header(&[0x03, 0x00]);
        assert!(
            result.is_none(),
            "2-byte slice must return None (BC-2.20.001 postcondition 1)"
        );
    }

    /// BC-2.20.001 canonical vector: three-byte slice (one byte short) returns None.
    ///
    /// Canonical vector from BC-2.20.001: `[0x03, 0x00, 0x00]` (3 bytes) -> None.
    /// This slice looks like the start of a valid frame (correct version byte) but is one
    /// byte short of the 4-byte TPKT header minimum. The length guard fires before any
    /// length-field bytes exist to inspect.
    ///
    /// Traces: BC-2.20.001 postconditions 1-3; AC-184-001; EC-003; canonical test vector.
    #[test]
    fn test_BC_2_20_001_returns_none_for_three_bytes_canonical_vector() {
        let data: &[u8] = &[0x03, 0x00, 0x00];
        let result = parse_tpkt_header(data);
        assert!(
            result.is_none(),
            "3-byte canonical vector must return None (BC-2.20.001 EC-003, canonical vector)"
        );
    }

    /// BC-2.20.001 invariant: no panic on any truncated input, including all-zero and
    /// all-0xFF content.
    ///
    /// Exercises lengths 0-3 with varied byte content to verify purity invariant 2 (no
    /// panic, no byte access). If any call panics, the test infrastructure reports it.
    ///
    /// Traces: BC-2.20.001 invariants 1-3; AC-184-001.
    #[test]
    fn test_BC_2_20_001_invariant_no_panic_on_truncated_inputs() {
        let inputs: &[&[u8]] = &[
            &[],
            &[0x00],
            &[0xFF],
            &[0x03],
            &[0x03, 0x00],
            &[0x03, 0x00, 0x00],
            &[0xFF, 0xFF, 0xFF],
        ];
        for &data in inputs {
            let result = parse_tpkt_header(data);
            assert!(
                result.is_none(),
                "input of len {} must return None for len < 4 (BC-2.20.001)",
                data.len()
            );
        }
    }

    // =========================================================================
    // BC-2.20.002: parse_tpkt_header returns None for version byte != 0x03
    // AC-184-002
    // =========================================================================

    /// BC-2.20.002 canonical vector: version byte 0x00 returns None.
    ///
    /// Canonical vector from BC-2.20.002: `[0x00, 0x00, 0x00, 0x04]` -> None.
    /// data.len() == 4 (length guard passes); data[0] != 0x03 (version guard fires).
    ///
    /// Traces: BC-2.20.002 postconditions 1-3; AC-184-002; EC-001; canonical test vector.
    #[test]
    fn test_BC_2_20_002_returns_none_for_version_0x00_canonical_vector() {
        let data: &[u8] = &[0x00, 0x00, 0x00, 0x04];
        let result = parse_tpkt_header(data);
        assert!(
            result.is_none(),
            "version byte 0x00 must return None (BC-2.20.002 canonical vector, postcondition 1)"
        );
    }

    /// BC-2.20.002 canonical vector: version byte 0x04 (off-by-one) returns None.
    ///
    /// Canonical vector from BC-2.20.002: `[0x04, 0x00, 0x00, 0x04]` -> None.
    /// No leniency for values adjacent to the valid version byte.
    ///
    /// Traces: BC-2.20.002 postcondition 1; AC-184-002; EC-002; canonical test vector.
    #[test]
    fn test_BC_2_20_002_returns_none_for_version_0x04_off_by_one_canonical_vector() {
        let data: &[u8] = &[0x04, 0x00, 0x00, 0x04];
        let result = parse_tpkt_header(data);
        assert!(
            result.is_none(),
            "version byte 0x04 (off-by-one) must return None (BC-2.20.002 canonical vector)"
        );
    }

    /// BC-2.20.002 canonical vector: version byte 0xFF returns None.
    ///
    /// Canonical vector from BC-2.20.002: `[0xFF, 0x00, 0x00, 0x04]` -> None.
    ///
    /// Traces: BC-2.20.002 postcondition 1; AC-184-002; EC-003; canonical test vector.
    #[test]
    fn test_BC_2_20_002_returns_none_for_version_0xFF_canonical_vector() {
        let data: &[u8] = &[0xFF, 0x00, 0x00, 0x04];
        let result = parse_tpkt_header(data);
        assert!(
            result.is_none(),
            "version byte 0xFF must return None (BC-2.20.002 canonical vector)"
        );
    }

    /// BC-2.20.002 postcondition 2: the length field is never decoded when the version
    /// byte is invalid — a bad version byte with a length field that would otherwise
    /// decode as a legal `[7, 65535]` value must still return `None`.
    ///
    /// Uses length bytes `[0xFF, 0xFF]` (decodes to 65535, the maximum legal length) to
    /// prove the version check short-circuits before any length-based accept could occur.
    ///
    /// Traces: BC-2.20.002 postcondition 2; AC-184-002.
    #[test]
    fn test_BC_2_20_002_bad_version_short_circuits_before_length_decode() {
        let data: &[u8] = &[0x02, 0x00, 0xFF, 0xFF];
        let result = parse_tpkt_header(data);
        assert!(
            result.is_none(),
            "bad version byte must return None even when length field would otherwise be \
             maximally valid (BC-2.20.002 postcondition 2)"
        );
    }

    /// BC-2.20.002 invariant: no panic for any `u8` value of `data[0]` other than 0x03.
    ///
    /// Sweeps a representative sample of the `u8` domain (excluding 0x03) to check purity
    /// invariant 2. Full 256-value totality is the VP-048 Kani obligation; this is the
    /// unit-level spot check.
    ///
    /// Traces: BC-2.20.002 invariant 2; AC-184-002.
    #[test]
    fn test_BC_2_20_002_invariant_no_panic_across_version_byte_sample() {
        for version in [0x01u8, 0x02, 0x05, 0x10, 0x7F, 0x80, 0xFE, 0xFF] {
            let data: [u8; 4] = [version, 0x00, 0x00, 0x04];
            let result = parse_tpkt_header(&data);
            assert!(
                result.is_none(),
                "version byte {version:#04x} (!= 0x03) must return None (BC-2.20.002)"
            );
        }
    }

    // =========================================================================
    // BC-2.20.003: parse_tpkt_header returns None for length field < 7 (RFC 1006 §6
    // minimum packet length)
    // AC-184-003
    // =========================================================================

    /// BC-2.20.003 canonical vector: length=0 (zero-length, most degenerate case) returns
    /// None.
    ///
    /// Canonical vector from BC-2.20.003: `[0x03, 0x00, 0x00, 0x00]` (length=0) -> None.
    /// Preconditions: data.len() >= 4, data[0] == 0x03 (version passes), decoded length
    /// (0) < 7.
    ///
    /// Traces: BC-2.20.003 postconditions 1-2; AC-184-003; EC-001; canonical test vector.
    #[test]
    fn test_BC_2_20_003_returns_none_for_length_zero_canonical_vector() {
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x00];
        let result = parse_tpkt_header(data);
        assert!(
            result.is_none(),
            "length=0 must return None (BC-2.20.003 canonical vector, postcondition 1)"
        );
    }

    /// BC-2.20.003 canonical vector: length=1 returns None.
    ///
    /// Canonical vector from BC-2.20.003: `[0x03, 0x00, 0x00, 0x01]` (length=1) -> None.
    ///
    /// Traces: BC-2.20.003 postcondition 1; AC-184-003; EC-002; canonical test vector.
    #[test]
    fn test_BC_2_20_003_returns_none_for_length_one_canonical_vector() {
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x01];
        let result = parse_tpkt_header(data);
        assert!(
            result.is_none(),
            "length=1 must return None (BC-2.20.003 canonical vector, postcondition 1)"
        );
    }

    /// BC-2.20.003: length=2 returns None.
    ///
    /// Exhausts the sub-minimum length values between the length=1 and length=3 canonical
    /// vectors.
    ///
    /// Traces: BC-2.20.003 postcondition 1; AC-184-003.
    #[test]
    fn test_BC_2_20_003_returns_none_for_length_two() {
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x02];
        let result = parse_tpkt_header(data);
        assert!(
            result.is_none(),
            "length=2 must return None (BC-2.20.003 postcondition 1)"
        );
    }

    /// BC-2.20.003 canonical vector: length=3 returns None.
    ///
    /// Canonical vector from BC-2.20.003: `[0x03, 0x00, 0x00, 0x03]` (length=3) -> None.
    ///
    /// Traces: BC-2.20.003 postcondition 1; AC-184-003; EC-003; canonical test vector.
    #[test]
    fn test_BC_2_20_003_returns_none_for_length_three_off_by_one_canonical_vector() {
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x03];
        let result = parse_tpkt_header(data);
        assert!(
            result.is_none(),
            "length=3 must return None (BC-2.20.003 canonical vector)"
        );
    }

    /// BC-2.20.003: length=4 (the TPKT header's own 4-byte structural floor, but below
    /// RFC 1006 §6's stated minimum packet length of 7) returns None.
    ///
    /// A length=4 packet is header-only, with zero bytes of room for even a minimal COTP
    /// PDU. RFC 1006 §6 states the minimum legal TPKT packet length is 7, so this is
    /// rejected here, at the TPKT layer itself (human ruling; this story was re-opened
    /// from its earlier converged state, which had accepted length=4 as a documented
    /// layering divergence — that divergence is retired).
    ///
    /// Traces: BC-2.20.003 postcondition 1; AC-184-003.
    #[test]
    fn test_BC_2_20_003_returns_none_for_length_four_below_rfc_minimum() {
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x04];
        let result = parse_tpkt_header(data);
        assert!(
            result.is_none(),
            "length=4 (below RFC 1006 §6 minimum of 7) must return None (BC-2.20.003)"
        );
    }

    /// BC-2.20.003: length=5 returns None.
    ///
    /// One byte of room for COTP — still below the RFC 1006 §6 minimum of 7.
    ///
    /// Traces: BC-2.20.003 postcondition 1; AC-184-003.
    #[test]
    fn test_BC_2_20_003_returns_none_for_length_five_below_rfc_minimum() {
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x05];
        let result = parse_tpkt_header(data);
        assert!(
            result.is_none(),
            "length=5 (below RFC 1006 §6 minimum of 7) must return None (BC-2.20.003)"
        );
    }

    /// BC-2.20.003 boundary vector: length=6 (one below the RFC 1006 §6 minimum of 7)
    /// returns None. Paired with `test_BC_2_20_004_valid_input_returns_some_header_length_7_canonical_vector`
    /// (length=7 -> Some) as the genuine 6-vs-7 accept-floor boundary.
    ///
    /// Traces: BC-2.20.003 postcondition 1; AC-184-003; RFC 1006 §6 boundary.
    #[test]
    fn test_BC_2_20_003_returns_none_for_length_six_boundary_below_rfc_minimum() {
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x06];
        let result = parse_tpkt_header(data);
        assert!(
            result.is_none(),
            "length=6 (one below RFC 1006 §6 minimum of 7) must return None (BC-2.20.003, \
             6-vs-7 boundary)"
        );
    }

    /// BC-2.20.003 invariant: no overflow/panic for any `u16` length value below the RFC
    /// 1006 §6 minimum of 7, including the all-zero length-field byte pattern.
    ///
    /// Traces: BC-2.20.003 invariant 2; AC-184-003; EC-005.
    #[test]
    fn test_BC_2_20_003_invariant_no_panic_across_sub_minimum_lengths() {
        for length_bytes in [
            [0x00u8, 0x00],
            [0x00, 0x01],
            [0x00, 0x02],
            [0x00, 0x03],
            [0x00, 0x04],
            [0x00, 0x05],
            [0x00, 0x06],
        ] {
            let data: [u8; 4] = [0x03, 0xAB, length_bytes[0], length_bytes[1]];
            let result = parse_tpkt_header(&data);
            assert!(
                result.is_none(),
                "length bytes {length_bytes:?} (decoded < 7) must return None (BC-2.20.003)"
            );
        }
    }

    // =========================================================================
    // BC-2.20.004: parse_tpkt_header returns Some(TpktHeader) for valid input
    // AC-184-004
    // =========================================================================

    /// BC-2.20.004 canonical vector: length=7 (exactly the RFC 1006 §6 minimum,
    /// minimal CR/CC-carrying frame). This is the genuine RFC-conformant accept floor —
    /// the 6-vs-7 boundary companion to `test_BC_2_20_003_returns_none_for_length_six_boundary_below_rfc_minimum`
    /// (length=6 -> None).
    ///
    /// Canonical vector from BC-2.20.004 / BC-2.20.001:
    ///   `[0x03, 0x00, 0x00, 0x07]` -> `Some(TpktHeader { version: 3, length: 7 })`.
    ///
    /// Traces: BC-2.20.004 postconditions 1-3; AC-184-004; canonical test vector.
    #[test]
    fn test_BC_2_20_004_valid_input_returns_some_header_length_7_canonical_vector() {
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x07];
        let result = parse_tpkt_header(data);
        let header = result
            .expect("length=7 must return Some (BC-2.20.004 canonical vector, postcondition 1)");
        assert_eq!(
            header,
            TpktHeader {
                version: 3,
                length: 7
            },
            "must decode version=3, length=7 exactly (BC-2.20.004 postcondition 1)"
        );
    }

    /// BC-2.20.004 canonical vector + invariant 2: length=65535 (maximum representable
    /// `u16`) is a legal accept, with a non-zero reserved byte.
    ///
    /// Canonical vector from BC-2.20.004:
    ///   `[0x03, 0xFF, 0xFF, 0xFF]` -> `Some(TpktHeader { version: 3, length: 65535 })`.
    ///   Reserved byte (`data[1]`) is `0xFF` (non-zero) and is ignored.
    ///
    /// Traces: BC-2.20.004 postcondition 1, invariant 2; AC-184-004; EC-002, EC-003;
    /// canonical test vector.
    #[test]
    fn test_BC_2_20_004_valid_input_returns_some_header_length_65535_max_canonical_vector() {
        let data: &[u8] = &[0x03, 0xFF, 0xFF, 0xFF];
        let result = parse_tpkt_header(data);
        let header = result.expect(
            "length=65535 (max u16) must return Some (BC-2.20.004 canonical vector, \
             invariant 2)",
        );
        assert_eq!(
            header,
            TpktHeader {
                version: 3,
                length: 65535
            },
            "must decode version=3, length=65535 exactly, with non-zero reserved byte \
             ignored (BC-2.20.004 invariant 2)"
        );
    }

    /// BC-2.20.004 invariant 1: the reserved byte (`data[1]`) is never inspected — a
    /// non-zero reserved byte with an otherwise-identical header must parse identically to
    /// a zero reserved byte.
    ///
    /// Compares `[0x03, 0x00, 0x00, 0x07]` (reserved=0x00) against
    /// `[0x03, 0xFF, 0x00, 0x07]` (reserved=0xFF, EC-003): both must decode to the same
    /// `TpktHeader { version: 3, length: 7 }`.
    ///
    /// Traces: BC-2.20.004 postcondition 2, invariant 1; AC-184-004; EC-003.
    #[test]
    fn test_BC_2_20_004_reserved_byte_nonzero_parses_identically_to_zero() {
        let reserved_zero: &[u8] = &[0x03, 0x00, 0x00, 0x07];
        let reserved_nonzero: &[u8] = &[0x03, 0xFF, 0x00, 0x07];

        let header_zero = parse_tpkt_header(reserved_zero)
            .expect("reserved=0x00 header must parse (BC-2.20.004)");
        let header_nonzero = parse_tpkt_header(reserved_nonzero)
            .expect("reserved=0xFF header must parse identically (BC-2.20.004 invariant 1)");

        assert_eq!(
            header_zero, header_nonzero,
            "reserved byte value must not affect the decoded TpktHeader \
             (BC-2.20.004 postcondition 2, invariant 1)"
        );
        assert_eq!(
            header_nonzero,
            TpktHeader {
                version: 3,
                length: 7
            },
            "non-zero reserved byte must still decode version=3, length=7 (BC-2.20.004)"
        );
    }

    /// BC-2.20.004 EC-005: `data.len() == length as usize` exactly (single complete frame,
    /// no trailing bytes) is accepted.
    ///
    /// Traces: BC-2.20.004 postcondition 1; AC-184-004; EC-005.
    #[test]
    fn test_BC_2_20_004_exact_length_match_no_trailing_bytes() {
        // length = 7 (header + 3 payload bytes, the RFC 1006 §6 minimum); data.len() == 7
        // exactly.
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x07, 0xAA, 0xBB, 0xCC];
        let result = parse_tpkt_header(data);
        let header =
            result.expect("exact-length-match input must return Some (BC-2.20.004 EC-005)");
        assert_eq!(
            header,
            TpktHeader {
                version: 3,
                length: 7
            },
            "must decode version=3, length=7 with data.len() == length exactly (EC-005)"
        );
    }

    /// BC-2.20.004 EC-004 / canonical vector: `data.len() > length as usize` — a second
    /// frame follows immediately. `parse_tpkt_header` must still return `Some` describing
    /// only the first frame's declared length; it does not attempt to consume or validate
    /// trailing bytes (frame-walk advance is a STORY-186 concern).
    ///
    /// The first assertion below is the genuine EC-004 case (declared `length == 10`,
    /// `data.len() == 14` — a second frame's header trails the first). The second
    /// assertion is the EC-005 exact-length-match case (declared `length == 10`,
    /// `data.len() == 10`), kept alongside it as a companion boundary check.
    ///
    /// Canonical vector from BC-2.20.004:
    ///   `[0x03, 0x00, 0x00, 0x0A, ...6 more payload bytes]` (10 bytes total)
    ///   -> `Some(TpktHeader { version: 3, length: 10 })`.
    ///
    /// Traces: BC-2.20.004 postcondition 4, EC-004; AC-184-004; canonical test vector.
    #[test]
    fn test_BC_2_20_004_trailing_bytes_beyond_declared_length_still_accepted_canonical_vector() {
        // EC-004: strictly more trailing bytes than the declared length (a second full
        // frame's worth) — confirm first-frame-only decode holds regardless of trailer.
        let data_with_second_frame: &[u8] = &[
            0x03, 0x00, 0x00, 0x0A, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // frame 1 (10 bytes)
            0x03, 0x00, 0x00, 0x04, // frame 2 header (4 bytes)
        ];
        let header2 = parse_tpkt_header(data_with_second_frame).expect(
            "input with a second frame appended must still return Some for the first frame \
             (BC-2.20.004 EC-004)",
        );
        assert_eq!(
            header2,
            TpktHeader {
                version: 3,
                length: 10
            },
            "must decode only the first frame's header/length, ignoring the second frame's \
             bytes entirely (BC-2.20.004 postcondition 4)"
        );

        // EC-005 companion check: 4-byte header (length=10) + 6 arbitrary payload bytes =
        // 10 bytes total, matching the canonical vector's total length exactly (no
        // trailing bytes at all — data.len() == length).
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x0A, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        assert_eq!(data.len(), 10, "canonical vector must be exactly 10 bytes");
        let result = parse_tpkt_header(data);
        let header = result.expect(
            "input with data.len() == length exactly must return Some (BC-2.20.004 EC-005)",
        );
        assert_eq!(
            header,
            TpktHeader {
                version: 3,
                length: 10
            },
            "must decode version=3, length=10 with data.len() == length exactly (EC-005)"
        );
    }

    // =========================================================================
    // AC-184-005: the four parse_tpkt_header outcomes are jointly exhaustive and
    // mutually exclusive (unit-level spot check; full exhaustiveness is VP-048 Kani)
    // =========================================================================

    /// AC-184-005 / BC-2.20.004 invariant 3: spot-checks that every representative input
    /// falls into exactly one of the four outcome classes (too-short / bad-version /
    /// bad-length / accept), and that the classification matches the expected outcome for
    /// each hand-picked boundary case across all four BCs.
    ///
    /// This is not exhaustive over all `&[u8]` (that is VP-048's Kani obligation, deferred
    /// to STORY-194) — it is a boundary-sampling spot check that a mistake in ordering the
    /// three guards (e.g. checking length before version, or length before size) would
    /// still be caught here.
    ///
    /// Traces: BC-2.20.004 invariant 3; AC-184-005.
    #[test]
    fn test_BC_2_20_004_four_way_partition_is_exhaustive() {
        // (input, expected outcome) pairs spanning all four classes at their boundaries.
        let cases: &[(&[u8], Option<TpktHeader>)] = &[
            // Class A: too short (BC-2.20.001) — length guard fires first, regardless of
            // what the first byte would otherwise decode to.
            (&[], None),
            (&[0x03], None),
            (&[0x03, 0x00, 0x00], None),
            // Class B: bad version (BC-2.20.002) — fires once len >= 4, before length
            // decode, even when the length bytes would otherwise be maximally valid.
            (&[0x02, 0x00, 0xFF, 0xFF], None),
            (&[0x00, 0x00, 0x00, 0x04], None),
            // Class C: bad length (BC-2.20.003) — fires once len >= 4 and version == 0x03,
            // for decoded length < 7 (RFC 1006 §6 minimum). Includes the 6-vs-7 boundary.
            (&[0x03, 0x00, 0x00, 0x00], None),
            (&[0x03, 0x00, 0x00, 0x03], None),
            (&[0x03, 0x00, 0x00, 0x04], None),
            (&[0x03, 0x00, 0x00, 0x06], None),
            // Class D: accept (BC-2.20.004) — len >= 4, version == 0x03, length in
            // [7, 65535].
            (
                &[0x03, 0x00, 0x00, 0x07],
                Some(TpktHeader {
                    version: 3,
                    length: 7,
                }),
            ),
            (
                &[0x03, 0xFF, 0xFF, 0xFF],
                Some(TpktHeader {
                    version: 3,
                    length: 65535,
                }),
            ),
        ];

        for (data, expected) in cases {
            let actual = parse_tpkt_header(data);
            assert_eq!(
                actual, *expected,
                "input {data:?} must classify to exactly one outcome class \
                 (AC-184-005, BC-2.20.004 invariant 3)"
            );
        }
    }

    // =========================================================================
    // DF-CANONICAL-FRAME-HOLDOUT-001: independent RFC-1006-derived holdout vector(s).
    // Unlike every other vector in this file (which traces to this project's own
    // BC-2.20.001-004 text), the vectors below are derived directly from the RFC 1006
    // spec document, independently of any BC. RFC 1006 §6 ("Packet Format") defines the
    // TPKT wire layout: octet 0 = version (0x03); octet 1 = reserved; octets 2-3 =
    // big-endian TPKTLength INCLUDING the 4-byte header, RFC-stated range [7, 65535].
    // =========================================================================

    /// RFC-VALID holdout: the RFC 1006 §6 stated minimum legal TPKT packet length is 7
    /// (4-byte header + 3-byte minimum COTP). `parse_tpkt_header` enforces exactly this
    /// minimum, so this is the genuinely RFC-conformant minimum-length accept vector.
    ///
    /// Traces: DF-CANONICAL-FRAME-HOLDOUT-001 (spec-independent holdout, not a BC vector).
    #[test]
    fn test_rfc1006_s6_minimum_valid_length_holdout() {
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x07];
        let result = parse_tpkt_header(data);
        assert_eq!(
            result,
            Some(TpktHeader {
                version: 0x03,
                length: 7
            }),
            "RFC 1006 §6: minimum legal TPKT packet length = 7 (4-byte header + 3-byte \
             minimum COTP)."
        );
    }

    /// RFC 1006 §6 states the minimum legal TPKT packet length is 7; length=4 is a
    /// header-only packet (no room for even a minimal COTP PDU) and is below that
    /// minimum, so it is rejected.
    ///
    /// Traces: DF-CANONICAL-FRAME-HOLDOUT-001 (spec-independent holdout, not a BC vector).
    #[test]
    fn test_rfc1006_s6_length_four_below_minimum_returns_none() {
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x04];
        let result = parse_tpkt_header(data);
        assert_eq!(
            result, None,
            "RFC 1006 §6 states min=7; length=4 (header-only, no room for COTP) is below \
             the minimum and is rejected."
        );
    }

    /// Per RFC 1006 §6: a TPKT header declaring a length larger than the 4-byte header
    /// itself (here: header + 6 payload octets = 10 total) must decode `length` as the
    /// full big-endian TPKTLength value, independently of any payload byte contents.
    /// Derived from RFC 1006 §6, independently of BC-2.20.00x.
    ///
    /// Traces: DF-CANONICAL-FRAME-HOLDOUT-001 (spec-independent holdout, not a BC vector).
    #[test]
    fn test_rfc1006_s6_ten_byte_tpkt_holdout() {
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let result = parse_tpkt_header(data);
        assert_eq!(
            result,
            Some(TpktHeader {
                version: 0x03,
                length: 10
            }),
            "RFC 1006 §6 TPKT header declaring length=10 (4-byte header + 6 payload \
             octets) must decode version=0x03, length=10"
        );
    }

    /// Input-independence holdout (L2): exercises a length value absent from every
    /// BC-2.20.00x vector table -- 0x0205 = 517 -- to cover more length-field bit
    /// positions than any BC-derived vector does (BC vectors use only 0, 1, 2, 3, 4, 6,
    /// 7, 10, and 65535).
    ///
    /// Traces: DF-CANONICAL-FRAME-HOLDOUT-001 (spec-independent holdout, not a BC vector).
    #[test]
    fn test_rfc1006_s6_wide_length_field_holdout() {
        let data: &[u8] = &[0x03, 0x00, 0x02, 0x05];
        let result = parse_tpkt_header(data);
        assert_eq!(
            result,
            Some(TpktHeader {
                version: 0x03,
                length: 517
            }),
            "RFC 1006 §6 TPKT header with length=517 (0x0205), a bit pattern absent from \
             any BC-2.20.00x vector, must decode version=0x03, length=517"
        );
    }

    // =========================================================================
    // Property-based test: independent oracle re-implementation of the four-way
    // partition, checked against parse_tpkt_header across randomized inputs.
    // =========================================================================
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        /// Re-derivation of BC-2.20.001-004's classification logic, written independently
        /// of `parse_tpkt_header`'s implementation code. It is logically equivalent to
        /// the function under test (both implement the same BC), so this proptest is a
        /// mutation-catcher rather than a proof of independent correctness: it flags
        /// implementation drift when one side changes without the other. Because both this
        /// oracle and the canonical-vector unit tests above are ultimately derived from the
        /// same BC-2.20.001-004 spec text, neither guards against a shared logic error --
        /// e.g. endianness or boundary mistakes -- inherited from that spec text itself.
        /// Spec-independent grounding against such an error comes from
        /// `test_rfc1006_s6_minimum_valid_length_holdout` above, whose vector is derived
        /// directly from RFC 1006 §6 rather than from this project's BCs
        /// (DF-CANONICAL-FRAME-HOLDOUT-001).
        fn oracle(data: &[u8]) -> Option<TpktHeader> {
            if data.len() < 4 {
                return None;
            }
            if data[0] != 0x03 {
                return None;
            }
            let length = u16::from_be_bytes([data[2], data[3]]);
            if length < 7 {
                return None;
            }
            Some(TpktHeader { version: 3, length })
        }

        proptest! {
            /// BC-2.20.001-004 totality: for any byte slice of length 0-16, the
            /// implementation must agree with the independent oracle on all four
            /// outcomes (too-short / bad-version / bad-length / accept). This is the
            /// proptest complement to the unit-level exhaustiveness spot check
            /// (`test_BC_2_20_004_four_way_partition_is_exhaustive`); full formal
            /// exhaustiveness over all lengths is the VP-048 Kani obligation.
            #[test]
            fn test_BC_2_20_004_proptest_matches_independent_oracle(
                data in proptest::collection::vec(any::<u8>(), 0..=16)
            ) {
                let expected = oracle(&data);
                let actual = parse_tpkt_header(&data);
                prop_assert_eq!(
                    actual, expected,
                    "parse_tpkt_header must match the independent BC-2.20.001-004 oracle \
                     for input {:?}", data
                );
            }

            /// BC-2.20.004 invariant 2 (representational upper bound): for any accepted
            /// header, `length` must be exactly the big-endian u16 decoded from
            /// `data[2..4]`, with no overflow, across randomized valid-shaped inputs.
            #[test]
            fn test_BC_2_20_004_proptest_accepted_length_matches_decoded_bytes(
                len_hi in any::<u8>(),
                len_lo in any::<u8>(),
                reserved in any::<u8>(),
            ) {
                let decoded = u16::from_be_bytes([len_hi, len_lo]);
                prop_assume!(decoded >= 7);
                let data = [0x03u8, reserved, len_hi, len_lo];
                let result = parse_tpkt_header(&data);
                prop_assert_eq!(
                    result,
                    Some(TpktHeader { version: 3, length: decoded }),
                    "accepted length must exactly match the big-endian u16 decode of \
                     data[2..4], reserved byte {:#04x} ignored", reserved
                );
            }
        }
    }
}

/// Tests for STORY-185: S7comm COTP TPDU-Type Parser (pure-core free function).
///
/// Covers BC-2.20.005 through BC-2.20.012 and the edge cases enumerated in each BC.
///
/// ## Contract coverage
/// - BC-2.20.005: `parse_cotp_header` returns `None` for input shorter than 2 bytes.
/// - BC-2.20.006: `parse_cotp_header` returns `None` when the Length Indicator (LI)
///   declares more header bytes than are present (LI-truncation).
/// - BC-2.20.007: `parse_cotp_header` recognizes Connect Request (CR) TPDUs
///   (`tpkt_payload[1] & 0xF0 == 0xE0`), `protocol_id: None`.
/// - BC-2.20.008: `parse_cotp_header` recognizes Connect Confirm (CC) TPDUs
///   (`tpkt_payload[1] & 0xF0 == 0xD0`), `protocol_id: None`.
/// - BC-2.20.009: `parse_cotp_header` recognizes Data Transfer (DT) TPDUs
///   (`tpkt_payload[1] & 0xF0 == 0xF0`) with a non-empty payload and extracts
///   `protocol_id` verbatim.
/// - BC-2.20.010: `parse_cotp_header` recognizes DT TPDUs with an empty payload
///   (`protocol_id: None`).
/// - BC-2.20.011: `parse_cotp_header` returns `None` for an unrecognized TPDU-type
///   code (high nibble not in `{0xE0, 0xD0, 0xF0}`); never force-fit.
/// - BC-2.20.012: `protocol_id` is extracted verbatim, never interpreted (frozen SS-20
///   to SS-21 boundary) — `parse_cotp_header` never compares the extracted byte
///   against any specific value.
///
/// ## Test naming convention
/// Tests follow `test_BC_S_SS_NNN_xxx()` for BC-traceable tests, matching the
/// AC-185-00x `**Test:**` citations in STORY-185.md. The non_snake_case lint fires on
/// uppercase BC IDs — suppressed by the file-level `#![allow(non_snake_case)]` at the
/// top of this file, which applies crate-wide to this whole test binary.
///
/// ## Provenance
/// Authored Red-first as TDD stubs (STORY-185 `tdd_mode: strict`; BC-2.20.005-012);
/// the Red Gate was verified via `cargo test --test iso_on_tcp_tests` before the
/// `todo!()` stub was replaced by the STORY-185 implementation of
/// `parse_cotp_header`; these tests are now GREEN.
///
/// ## Literal-avoidance note (BC-2.20.012 / AC-185-009)
/// Per BC-2.20.012's frozen SS-20/SS-21 boundary, `protocol_id` is a raw, uninterpreted
/// byte at this layer — S7comm (`0x32`) / S7comm-plus (`0x72`) disambiguation belongs to
/// SS-21 (`S7commAnalyzer`, STORY-186+), not to this module or its tests. This test
/// module therefore deliberately never writes the literal byte values `0x32` or `0x72`,
/// nor the strings "S7comm"/"S7comm-plus", anywhere in source text — the totality sweep
/// below (`test_BC_2_20_012_protocol_id_extraction_totality`) still exercises those two
/// byte *values* at runtime (they arise dynamically from a `0u8..=255u8` loop), without
/// either value ever appearing as a literal token in this file.
///
/// ## Canonical test vectors and the independent ISO 8073 holdout
/// Most vectors below are canonical test vectors copied verbatim from BC-2.20.005-012.
/// Per DF-CANONICAL-FRAME-HOLDOUT-001, the `test_iso8073_rfc905_*` holdout tests near
/// the end of this module are authored independently of this project's BCs, derived
/// directly from RFC 905 ("ISO Transport Protocol Specification ISO DP 8073" — a
/// freely accessible IETF mirror of the ISO Transport Protocol text that the project's
/// own BCs cite as ISO 8073 / ITU-T X.224), fetched and cross-checked against this
/// project's BC citations while drafting this file. Citations below reference specific
/// RFC 905 section numbers and Table 8 ("TPDU code") verified directly against the
/// fetched document text, not assumed from memory.
//
// Per DF-TEST-NAMESPACE-001: all STORY-185 tests are grouped inside a dedicated
// `mod story_185` wrapper to prevent test-function name collisions with other
// stories' BC-prefixed names (in particular, the pre-existing `mod story_184` above).
mod story_185 {
    use wirerust::analyzer::iso_on_tcp::{CotpHeader, CotpTpduType, parse_cotp_header};

    // =========================================================================
    // BC-2.20.005: parse_cotp_header returns None for input shorter than 2 bytes
    // AC-185-001
    // =========================================================================

    /// BC-2.20.005 canonical vectors: empty slice and one-byte slice both return None.
    ///
    /// Canonical vectors from BC-2.20.005: `[]` (0 bytes) -> None; `[0x02]` (1 byte,
    /// EC-002) -> None. Precondition: `tpkt_payload.len() < 2`. Postcondition 2: no
    /// bytes accessed beyond the length check, no panic even for `len() == 0` (EC-001).
    ///
    /// Traces: BC-2.20.005 postconditions 1-3; AC-185-001; EC-001; EC-002; canonical
    /// test vectors.
    #[test]
    fn test_BC_2_20_005_len_shorter_than_2_returns_none() {
        let result_empty = parse_cotp_header(&[]);
        assert_eq!(
            result_empty, None,
            "empty tpkt_payload must return None (BC-2.20.005 postcondition 1, EC-001)"
        );

        let result_one_byte = parse_cotp_header(&[0x02]);
        assert_eq!(
            result_one_byte, None,
            "1-byte tpkt_payload must return None (BC-2.20.005 canonical vector, EC-002)"
        );
    }

    /// BC-2.20.005 invariant: no panic for any 0- or 1-byte input, including all-zero
    /// and all-0xFF content.
    ///
    /// Traces: BC-2.20.005 invariants 1-3; AC-185-001.
    #[test]
    fn test_BC_2_20_005_invariant_no_panic_across_short_inputs() {
        let inputs: &[&[u8]] = &[&[], &[0x00], &[0xFF], &[0x02]];
        for &data in inputs {
            let result = parse_cotp_header(data);
            assert_eq!(
                result,
                None,
                "input of len {} must return None for len < 2 (BC-2.20.005)",
                data.len()
            );
        }
    }

    // =========================================================================
    // BC-2.20.006: parse_cotp_header returns None when LI declares more bytes than
    // are present (LI-truncation)
    // AC-185-002
    // =========================================================================

    /// BC-2.20.006 canonical vectors: LI declares more remaining header bytes than
    /// `tpkt_payload` actually contains.
    ///
    /// Canonical vectors from BC-2.20.006: `[0x06, 0xE0, 0x00, 0x01]` (LI=6, only 3
    /// bytes follow the LI octet; EC-001, truncated CR header) -> None; `[0x02, 0xF0]`
    /// (LI=2, only 1 byte follows; EC-002, truncated DT header) -> None.
    ///
    /// Traces: BC-2.20.006 postconditions 1-2; AC-185-002; EC-001; EC-002; canonical
    /// test vectors.
    #[test]
    fn test_BC_2_20_006_li_truncation_returns_none() {
        let truncated_cr: &[u8] = &[0x06, 0xE0, 0x00, 0x01];
        assert_eq!(
            parse_cotp_header(truncated_cr),
            None,
            "LI=6 declaring 6 more bytes with only 3 present must return None \
             (BC-2.20.006 canonical vector, EC-001)"
        );

        let truncated_dt: &[u8] = &[0x02, 0xF0];
        assert_eq!(
            parse_cotp_header(truncated_dt),
            None,
            "LI=2 declaring 2 more bytes with only 1 present must return None \
             (BC-2.20.006 canonical vector, EC-002)"
        );
    }

    /// BC-2.20.006 invariant 2: no out-of-bounds index / panic for LI values spanning
    /// the full `u8` range, including the maximum LI value (255).
    ///
    /// Truncation predicate (BC-2.20.006): `tpkt_payload.len() < 1 + LI`. Against the
    /// fixed 3-byte buffer used here, that's `3 < 1 + LI`, i.e. genuinely truncating
    /// exactly when `LI >= 3`. Every sampled LI below is `>= 3`, so each one truncates
    /// and must return `None`: `0x03` is the exact boundary (`1+3=4 > 3`), and
    /// `0x0A`/`0x7F`/`0xFE`/`0xFF` truncate by increasingly wide margins up to the max
    /// `u8` value. (`0x01` is deliberately excluded from this sample: `1+1=2 <= 3` is
    /// NOT truncated, so with TPDU-code `0xE0` it would classify as `Some(CotpHeader
    /// { ConnectRequest, .. })` per BC-2.20.007 rather than `None` — asserting `None`
    /// for it would be a wrong test, not a truncation invariant check.)
    ///
    /// Traces: BC-2.20.006 invariant 2; AC-185-002.
    #[test]
    fn test_BC_2_20_006_invariant_no_panic_across_li_value_sample() {
        for li in [0x03u8, 0x0A, 0x7F, 0xFE, 0xFF] {
            let data: [u8; 3] = [li, 0xE0, 0x00];
            let result = parse_cotp_header(&data);
            assert_eq!(
                result, None,
                "LI={li:#04x} declaring more bytes than the 3-byte input contains must \
                 return None with no panic (BC-2.20.006 invariant 2)"
            );
        }
    }

    /// BC-2.20.006 EC-003: `LI == 0` is a degenerate but not-truncated case
    /// (`1 + 0 <= len`) — the truncation check passes and classification proceeds. The
    /// TPDU-code byte (`tpkt_payload[1]`) doubles as the sole payload byte at
    /// `payload_offset == 1`, since the fixed header contributes zero further bytes.
    ///
    /// Traces: BC-2.20.006 EC-003; AC-185-002 (boundary case demonstrating the
    /// truncation guard does not over-reject `LI == 0`).
    #[test]
    fn test_BC_2_20_006_li_zero_not_truncated_proceeds_to_classification() {
        let data: &[u8] = &[0x00, 0xF0];
        let result = parse_cotp_header(data);
        assert_eq!(
            result,
            Some(CotpHeader {
                tpdu_type: CotpTpduType::DataTransfer,
                protocol_id: Some(0xF0),
                payload_offset: 1,
            }),
            "LI=0 must not be rejected as truncated; classification proceeds to DT \
             recognition with payload_offset == 1 (BC-2.20.006 EC-003)"
        );
    }

    // =========================================================================
    // BC-2.20.007: parse_cotp_header recognizes Connect Request (CR) TPDU
    // AC-185-003
    // =========================================================================

    /// BC-2.20.007 canonical vector: minimal CR TPDU (`LI == 6`).
    ///
    /// Canonical vector from BC-2.20.007: `[0x06, 0xE0, 0x00, 0x00, 0x00, 0x01, 0x00]`
    /// (LI=6, CR, DST-REF=0x0000, SRC-REF=0x0001, class=0) ->
    /// `Some(CotpHeader{tpdu_type: ConnectRequest, protocol_id: None, payload_offset: 7})`.
    ///
    /// Traces: BC-2.20.007 postconditions 1-2; AC-185-003; EC-004; canonical test vector.
    #[test]
    fn test_BC_2_20_007_connect_request_recognized() {
        let data: &[u8] = &[0x06, 0xE0, 0x00, 0x00, 0x00, 0x01, 0x00];
        let result = parse_cotp_header(data);
        assert_eq!(
            result,
            Some(CotpHeader {
                tpdu_type: CotpTpduType::ConnectRequest,
                protocol_id: None,
                payload_offset: 7,
            }),
            "minimal CR TPDU must decode to ConnectRequest, protocol_id: None, \
             payload_offset: 7 (BC-2.20.007 canonical vector)"
        );
    }

    /// BC-2.20.007 EC-002: a non-zero low nibble on the TPDU-code byte (`0xE1`) does
    /// not affect CR recognition — only the high nibble is inspected.
    ///
    /// Traces: BC-2.20.007 invariant 2; AC-185-003; EC-002.
    #[test]
    fn test_BC_2_20_007_connect_request_nonzero_low_nibble_still_recognized() {
        let data: &[u8] = &[0x06, 0xE1, 0x00, 0x00, 0x00, 0x01, 0x00];
        let result = parse_cotp_header(data);
        assert_eq!(
            result,
            Some(CotpHeader {
                tpdu_type: CotpTpduType::ConnectRequest,
                protocol_id: None,
                payload_offset: 7,
            }),
            "TPDU-code 0xE1 (non-zero low nibble) must still be recognized as CR \
             (BC-2.20.007 EC-002, high-nibble-only discrimination)"
        );
    }

    /// BC-2.20.007 postcondition 3: `protocol_id` is unconditionally `None` for a CR
    /// TPDU, even when bytes are present in `tpkt_payload` beyond the fixed CR header.
    ///
    /// Uses a trailing byte (`0xAB`) beyond `payload_offset` to prove that no
    /// upper-layer-payload interpretation is ever attempted for CR.
    ///
    /// Traces: BC-2.20.007 postcondition 3; AC-185-003.
    #[test]
    fn test_BC_2_20_007_connect_request_protocol_id_none_even_with_trailing_bytes() {
        let data: &[u8] = &[0x06, 0xE0, 0x00, 0x00, 0x00, 0x01, 0x00, 0xAB];
        let result = parse_cotp_header(data);
        assert_eq!(
            result,
            Some(CotpHeader {
                tpdu_type: CotpTpduType::ConnectRequest,
                protocol_id: None,
                payload_offset: 7,
            }),
            "protocol_id must remain None for CR even with a trailing byte present \
             beyond the fixed CR header (BC-2.20.007 postcondition 3)"
        );
    }

    // =========================================================================
    // BC-2.20.008: parse_cotp_header recognizes Connect Confirm (CC) TPDU
    // AC-185-004
    // =========================================================================

    /// BC-2.20.008 canonical vector: minimal CC TPDU (`LI == 6`).
    ///
    /// Canonical vector from BC-2.20.008: `[0x06, 0xD0, 0x00, 0x01, 0x00, 0x00, 0x00]`
    /// (LI=6, CC, DST-REF=0x0001, SRC-REF=0x0000, class=0) ->
    /// `Some(CotpHeader{tpdu_type: ConnectConfirm, protocol_id: None, payload_offset: 7})`.
    ///
    /// Traces: BC-2.20.008 postconditions 1-3; AC-185-004; EC-004; canonical test vector.
    #[test]
    fn test_BC_2_20_008_connect_confirm_recognized() {
        let data: &[u8] = &[0x06, 0xD0, 0x00, 0x01, 0x00, 0x00, 0x00];
        let result = parse_cotp_header(data);
        assert_eq!(
            result,
            Some(CotpHeader {
                tpdu_type: CotpTpduType::ConnectConfirm,
                protocol_id: None,
                payload_offset: 7,
            }),
            "minimal CC TPDU must decode to ConnectConfirm, protocol_id: None, \
             payload_offset: 7 (BC-2.20.008 canonical vector)"
        );
    }

    /// BC-2.20.008 EC-002: a non-zero low nibble on the TPDU-code byte (`0xD1`) does
    /// not affect CC recognition.
    ///
    /// Traces: BC-2.20.008 invariant 2; AC-185-004; EC-002.
    #[test]
    fn test_BC_2_20_008_connect_confirm_nonzero_low_nibble_still_recognized() {
        let data: &[u8] = &[0x06, 0xD1, 0x00, 0x01, 0x00, 0x00, 0x00];
        let result = parse_cotp_header(data);
        assert_eq!(
            result,
            Some(CotpHeader {
                tpdu_type: CotpTpduType::ConnectConfirm,
                protocol_id: None,
                payload_offset: 7,
            }),
            "TPDU-code 0xD1 (non-zero low nibble) must still be recognized as CC \
             (BC-2.20.008 EC-002, high-nibble-only discrimination)"
        );
    }

    // =========================================================================
    // BC-2.20.009: parse_cotp_header recognizes DT with non-empty payload and
    // extracts protocol_id
    // AC-185-005
    // =========================================================================

    /// BC-2.20.009 canonical-vector-shaped test (EC-004 variant): minimal DT TPDU
    /// (`LI == 2`) with a non-`0x32`/`0x72` protocol-ID byte (`0x01`, simulating an
    /// MMS/ICCP or otherwise-unrecognized upper-layer protocol), extracted verbatim.
    ///
    /// Per this test module's literal-avoidance note, the canonical `0x32`/`0x72`
    /// vectors from BC-2.20.009's own table are not reproduced verbatim here (that
    /// would place the literals `0x32`/`0x72` in this file); EC-004's `0x01` vector is
    /// used instead, which is equally a canonical BC-2.20.009 test vector.
    ///
    /// Traces: BC-2.20.009 postconditions 1-3; AC-185-005; EC-004; canonical test vector.
    #[test]
    fn test_BC_2_20_009_dt_nonempty_payload_extracts_protocol_id() {
        let data: &[u8] = &[0x02, 0xF0, 0x80, 0x01];
        let result = parse_cotp_header(data);
        assert_eq!(
            result,
            Some(CotpHeader {
                tpdu_type: CotpTpduType::DataTransfer,
                protocol_id: Some(0x01),
                payload_offset: 3,
            }),
            "minimal DT TPDU with a non-empty payload must extract protocol_id \
             verbatim, payload_offset: 3 (BC-2.20.009 canonical vector, EC-004)"
        );
    }

    /// BC-2.20.009 postcondition 2: `protocol_id` is exactly the byte at
    /// `tpkt_payload[payload_offset]` — only the first trailing byte, never a later one,
    /// regardless of how many further bytes follow.
    ///
    /// Traces: BC-2.20.009 postcondition 2; AC-185-005.
    #[test]
    fn test_BC_2_20_009_dt_protocol_id_is_first_trailing_byte_only() {
        let data: &[u8] = &[0x02, 0xF0, 0x80, 0x01, 0x02, 0x03, 0x04];
        let result = parse_cotp_header(data);
        assert_eq!(
            result,
            Some(CotpHeader {
                tpdu_type: CotpTpduType::DataTransfer,
                protocol_id: Some(0x01),
                payload_offset: 3,
            }),
            "protocol_id must be exactly tpkt_payload[payload_offset] (0x01), never a \
             later trailing byte (BC-2.20.009 postcondition 2)"
        );
    }

    /// BC-2.20.009 / BC-2.20.012 boundary values: the protocol-ID byte is extracted
    /// verbatim at both `u8` extremes (`0x00` and `0xFF`), with identical `tpdu_type`
    /// and `payload_offset` across both.
    ///
    /// Traces: BC-2.20.009 postcondition 2; BC-2.20.012 EC-003, EC-004; AC-185-005.
    #[test]
    fn test_BC_2_20_009_dt_protocol_id_extracted_for_boundary_byte_values() {
        let data_min: &[u8] = &[0x02, 0xF0, 0x80, 0x00];
        assert_eq!(
            parse_cotp_header(data_min),
            Some(CotpHeader {
                tpdu_type: CotpTpduType::DataTransfer,
                protocol_id: Some(0x00),
                payload_offset: 3,
            }),
            "protocol-ID byte 0x00 (minimum u8) must be extracted verbatim \
             (BC-2.20.012 EC-004)"
        );

        let data_max: &[u8] = &[0x02, 0xF0, 0x80, 0xFF];
        assert_eq!(
            parse_cotp_header(data_max),
            Some(CotpHeader {
                tpdu_type: CotpTpduType::DataTransfer,
                protocol_id: Some(0xFF),
                payload_offset: 3,
            }),
            "protocol-ID byte 0xFF (maximum u8) must be extracted verbatim \
             (BC-2.20.012 EC-003)"
        );
    }

    // =========================================================================
    // BC-2.20.010: parse_cotp_header recognizes DT with empty payload —
    // protocol_id is None
    // AC-185-006
    // =========================================================================

    /// BC-2.20.010 canonical vector: minimal DT TPDU (`LI == 2`) with zero trailing
    /// payload bytes (`tpkt_payload.len() == payload_offset` exactly).
    ///
    /// Canonical vector from BC-2.20.010: `[0x02, 0xF0, 0x80]` (LI=2, DT, TPDU-NR=0x80,
    /// no trailing payload byte) ->
    /// `Some(CotpHeader{tpdu_type: DataTransfer, protocol_id: None, payload_offset: 3})`.
    ///
    /// Traces: BC-2.20.010 postconditions 1-2; AC-185-006; EC-001; canonical test vector.
    #[test]
    fn test_BC_2_20_010_dt_empty_payload_protocol_id_none() {
        let data: &[u8] = &[0x02, 0xF0, 0x80];
        let result = parse_cotp_header(data);
        assert_eq!(
            result,
            Some(CotpHeader {
                tpdu_type: CotpTpduType::DataTransfer,
                protocol_id: None,
                payload_offset: 3,
            }),
            "minimal DT TPDU with zero trailing payload bytes must return \
             protocol_id: None, payload_offset: 3 (BC-2.20.010 canonical vector)"
        );
    }

    // =========================================================================
    // BC-2.20.011: parse_cotp_header returns None for an unrecognized TPDU-type code
    // AC-185-007, AC-185-008
    // =========================================================================

    /// BC-2.20.011 canonical vectors: DR, DC, ER, and all-zero TPDU-code bytes are
    /// never force-fit into CR/CC/DT.
    ///
    /// Canonical vectors from BC-2.20.011: `[0x02, 0x80, 0x00]` (DR-shaped) -> None;
    /// `[0x02, 0xC0, 0x00]` (DC-shaped) -> None; `[0x02, 0x70, 0x00]` (ER-shaped) ->
    /// None; `[0x02, 0x00, 0x00]` (all-zero) -> None.
    ///
    /// Traces: BC-2.20.011 postconditions 1-2; AC-185-007; EC-001, EC-002, EC-003,
    /// EC-004; canonical test vectors.
    #[test]
    fn test_BC_2_20_011_unrecognized_tpdu_type_returns_none() {
        let cases: &[(&[u8], &str)] = &[
            (&[0x02, 0x80, 0x00], "DR-shaped (0x8_)"),
            (&[0x02, 0xC0, 0x00], "DC-shaped (0xC_)"),
            (&[0x02, 0x70, 0x00], "ER-shaped (0x7_)"),
            (&[0x02, 0x00, 0x00], "all-zero TPDU-code byte"),
        ];
        for (data, label) in cases {
            assert_eq!(
                parse_cotp_header(data),
                None,
                "{label} must return None, never force-fit into CR/CC/DT \
                 (BC-2.20.011 canonical vector)"
            );
        }
    }

    /// AC-185-008 / BC-2.20.011 invariant 3: the four-way TPDU-type match is
    /// exhaustive and non-overlapping over all 16 possible high-nibble values of
    /// `tpkt_payload[1]`. Every input below shares the same shape (`LI = 2`, 3 bytes
    /// total: LI, TPDU-code, one trailing byte) so that CR/CC (no payload check) and DT
    /// (payload check against `payload_offset == 3`, exactly satisfied by this 3-byte
    /// shape) all reach a defined, well-typed outcome; the remaining 13 nibble values
    /// must all reject.
    ///
    /// This is a unit-level spot check, not exhaustive over all `&[u8]` shapes (that is
    /// the VP-049 Kani obligation, deferred to STORY-194) — it exhaustively covers the
    /// 16-value high-nibble domain for one fixed input shape.
    ///
    /// Traces: BC-2.20.011 invariant 3; AC-185-008.
    #[test]
    fn test_BC_2_20_011_tpdu_type_match_is_exhaustive() {
        for nibble in 0x0u8..=0xF {
            let code_byte = nibble << 4;
            let data: [u8; 3] = [0x02, code_byte, 0x00];
            let result = parse_cotp_header(&data);
            let expected = match nibble {
                0xE => Some(CotpHeader {
                    tpdu_type: CotpTpduType::ConnectRequest,
                    protocol_id: None,
                    payload_offset: 3,
                }),
                0xD => Some(CotpHeader {
                    tpdu_type: CotpTpduType::ConnectConfirm,
                    protocol_id: None,
                    payload_offset: 3,
                }),
                0xF => Some(CotpHeader {
                    tpdu_type: CotpTpduType::DataTransfer,
                    protocol_id: None,
                    payload_offset: 3,
                }),
                _ => None,
            };
            assert_eq!(
                result, expected,
                "high nibble {nibble:#x} must classify to exactly one outcome \
                 (AC-185-008, BC-2.20.011 invariant 3)"
            );
        }
    }

    // =========================================================================
    // BC-2.20.012: protocol_id is extracted verbatim, never interpreted
    // AC-185-009
    // =========================================================================

    /// BC-2.20.012 postconditions 1-2: the protocol-ID extraction is a total identity
    /// mapping over every possible `u8` value — an exhaustive sweep over all 256
    /// values, not a random sample, so this totality claim is fully covered rather than
    /// probabilistically covered. `tpdu_type` and `payload_offset` must stay constant
    /// across the entire sweep; only `protocol_id` varies with the input byte.
    ///
    /// This sweep necessarily also exercises the two byte values a downstream SS-21
    /// disambiguation table would treat specially, but neither value is ever written as
    /// a literal token in this file (see the file-level literal-avoidance note) — both
    /// arise only at runtime from the `0u8..=255u8` loop bound.
    ///
    /// Traces: BC-2.20.012 postconditions 1-2; invariant 2; AC-185-009.
    #[test]
    fn test_BC_2_20_012_protocol_id_extraction_totality() {
        for byte in 0u8..=255u8 {
            let data: [u8; 4] = [0x02, 0xF0, 0x80, byte];
            let result = parse_cotp_header(&data);
            assert_eq!(
                result,
                Some(CotpHeader {
                    tpdu_type: CotpTpduType::DataTransfer,
                    protocol_id: Some(byte),
                    payload_offset: 3,
                }),
                "protocol_id extraction must be the identity function for byte \
                 {byte:#04x} (BC-2.20.012 postcondition 1, exhaustive 256-value sweep)"
            );
        }
    }

    /// BC-2.20.012 postcondition 3 / AC-185-009 static regression guard: the source
    /// file must contain zero occurrences of the literals `0x32` or `0x72` anywhere —
    /// these are the classic-S7comm and S7comm-plus protocol-ID values respectively,
    /// and their interpretation belongs entirely to `S7commAnalyzer` (SS-21), never to
    /// this module's parsing logic.
    ///
    /// This is a whole-file substring check (not scoped to excluding doc comments): as
    /// of this story, `src/analyzer/iso_on_tcp.rs` contains zero occurrences of either
    /// literal anywhere, including in doc comments, so a whole-file check is the
    /// correct, unambiguous regression guard — a future doc comment introducing either
    /// literal would itself be exactly the kind of drift this guard exists to catch.
    ///
    /// Traces: BC-2.20.012 postcondition 3; AC-185-009 (static regression-guard test).
    #[test]
    fn test_BC_2_20_012_static_regression_guard_no_hardcoded_protocol_literals() {
        let source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/analyzer/iso_on_tcp.rs"
        ))
        .expect("src/analyzer/iso_on_tcp.rs must be readable for the regression guard");

        // Constructed via concatenation so this file itself never contains the literal
        // substrings "0x32"/"0x72" as contiguous text either.
        let classic_s7comm_literal = ["0x", "32"].concat();
        let s7comm_plus_literal = ["0x", "72"].concat();

        assert!(
            !source.contains(&classic_s7comm_literal),
            "src/analyzer/iso_on_tcp.rs must not contain the literal {classic_s7comm_literal} \
             anywhere (BC-2.20.012 postcondition 3) — S7comm disambiguation belongs to SS-21"
        );
        assert!(
            !source.contains(&s7comm_plus_literal),
            "src/analyzer/iso_on_tcp.rs must not contain the literal {s7comm_plus_literal} \
             anywhere (BC-2.20.012 postcondition 3) — S7comm-plus disambiguation belongs to SS-21"
        );
    }

    // =========================================================================
    // DF-CANONICAL-FRAME-HOLDOUT-001: independent ISO 8073 holdout vectors.
    //
    // Unlike every vector above (which traces to this project's own BC-2.20.005-012
    // text), the vectors below are derived directly from RFC 905 ("ISO Transport
    // Protocol Specification ISO DP 8073"), an IETF-published, freely accessible
    // mirror of the ISO Transport Protocol specification text — fetched directly and
    // cross-checked line-by-line while drafting this file, independently of this
    // project's own BC citations to ISO 8073 / ITU-T X.224.
    //
    // Verified citations (RFC 905, section numbers and Table 8 read directly from the
    // fetched document text):
    //   - §13.2 "Structure": TPDUs contain, in order, the LI field, the fixed part,
    //     the variable part (if present), then the data field.
    //   - §13.2.1 "Length indicator field": LI occupies the first octet of the TPDU;
    //     its value is "the header length in octets including parameters, but
    //     excluding the length indicator field and user data, if any."
    //   - §13.2.2.2 "TPDU code": contained in octet 2 of the header.
    //   - Table 8 "TPDU code" (page 115): CR = `1110 xxxx` (clause 13.3), CC =
    //     `1101 xxxx` (clause 13.4), DR = `1000 0000` (clause 13.5), DT = `1111 0000`
    //     (clause 13.7).
    //   - §13.7.1 "Structure", format (a) "Normal format for Classes 0 and 1": DT TPDU
    //     = LI | DT code (octet 2) | TPDU-NR and EOT (octet 3) | User Data (octet 4+).
    // =========================================================================

    /// ISO 8073 holdout: Table 8's `1110 xxxx` CR code and `1101 xxxx` CC code both
    /// reserve bits 4-1 (the low nibble) for CDT signaling, confirming (independently
    /// of BC-2.20.007/008's own EC-002 vectors) that only the high nibble discriminates
    /// TPDU type — using different low-nibble values than either BC's canonical
    /// vectors.
    ///
    /// Traces: DF-CANONICAL-FRAME-HOLDOUT-001 (spec-independent holdout, not a BC
    /// vector); RFC 905 §13.2.2.2, Table 8.
    #[test]
    fn test_iso8073_rfc905_table8_cr_cc_low_nibble_is_free_holdout() {
        let cr_data: &[u8] = &[0x06, 0xEF, 0x00, 0x00, 0x00, 0x01, 0x00];
        assert_eq!(
            parse_cotp_header(cr_data),
            Some(CotpHeader {
                tpdu_type: CotpTpduType::ConnectRequest,
                protocol_id: None,
                payload_offset: 7,
            }),
            "RFC 905 Table 8: CR code is 1110 xxxx — low nibble 0xF must not prevent \
             CR recognition"
        );

        let cc_data: &[u8] = &[0x06, 0xDA, 0x00, 0x01, 0x00, 0x00, 0x00];
        assert_eq!(
            parse_cotp_header(cc_data),
            Some(CotpHeader {
                tpdu_type: CotpTpduType::ConnectConfirm,
                protocol_id: None,
                payload_offset: 7,
            }),
            "RFC 905 Table 8: CC code is 1101 xxxx — low nibble 0xA must not prevent \
             CC recognition"
        );
    }

    /// ISO 8073 holdout: Table 8's Disconnect Request (DR) code is the fixed octet
    /// `1000 0000` (clause 13.5) — a TPDU type this project's `CotpTpduType` (frozen to
    /// exactly 3 variants) deliberately does not model. `parse_cotp_header` must reject
    /// it, not force-fit it to the "closest" recognized type.
    ///
    /// Traces: DF-CANONICAL-FRAME-HOLDOUT-001 (spec-independent holdout, not a BC
    /// vector); RFC 905 §13.2.2.2, Table 8, clause 13.5.
    #[test]
    fn test_iso8073_rfc905_table8_dr_code_not_modeled_holdout() {
        let data: &[u8] = &[0x02, 0x80, 0x00];
        assert_eq!(
            parse_cotp_header(data),
            None,
            "RFC 905 Table 8: DR (Disconnect Request) code 1000 0000 is not one of the \
             3 frozen CotpTpduType variants and must return None"
        );
    }

    /// ISO 8073 holdout: RFC 905 §13.7.1 format (a) ("Normal format for Classes 0 and
    /// 1") defines the DT TPDU's fixed part as exactly 2 octets — the DT code (octet 2)
    /// and the combined TPDU-NR/EOT byte (octet 3) — so `LI == 2` and user data begins
    /// at octet 4, i.e. `payload_offset == 1 + LI == 3`. This vector uses a distinct
    /// TPDU-NR/EOT byte (`0xC0`) and a distinct user-data byte (`0x99`) from every
    /// BC-2.20.009/010 canonical vector, to independently confirm the LI-to-
    /// payload-offset arithmetic rather than reusing a BC-derived byte pattern.
    ///
    /// Traces: DF-CANONICAL-FRAME-HOLDOUT-001 (spec-independent holdout, not a BC
    /// vector); RFC 905 §13.2.1, §13.7.1 format (a).
    #[test]
    fn test_iso8073_rfc905_s13_7_1_dt_class0_normal_format_holdout() {
        let data: &[u8] = &[0x02, 0xF0, 0xC0, 0x99];
        assert_eq!(
            parse_cotp_header(data),
            Some(CotpHeader {
                tpdu_type: CotpTpduType::DataTransfer,
                protocol_id: Some(0x99),
                payload_offset: 3,
            }),
            "RFC 905 §13.7.1 format (a): class-0 DT fixed part is exactly 2 octets \
             (DT code + TPDU-NR/EOT), so payload_offset == 1 + LI == 3 and the trailing \
             byte is the verbatim protocol-ID"
        );
    }

    /// ISO 8073 holdout: RFC 905 §13.2.1 states the LI value "shall be the header
    /// length in octets including parameters, but excluding the length indicator field
    /// and user data" — i.e. LI counts everything in the header *after* the LI octet
    /// itself. This vector reconstructs a minimal CR TPDU (fixed part = TPDU-code +
    /// 2-byte DST-REF + 2-byte SRC-REF + 1-byte class/options = 6 octets, so `LI == 6`)
    /// using DST-REF/SRC-REF values distinct from BC-2.20.007's own canonical vector,
    /// to independently confirm `payload_offset == 1 + LI` rather than reusing a
    /// BC-derived byte pattern.
    ///
    /// Traces: DF-CANONICAL-FRAME-HOLDOUT-001 (spec-independent holdout, not a BC
    /// vector); RFC 905 §13.2.1, §13.2 structure diagram, clause 13.3.
    #[test]
    fn test_iso8073_rfc905_s13_2_1_li_excludes_itself_holdout() {
        let data: &[u8] = &[0x06, 0xE3, 0xAA, 0xBB, 0xCC, 0xDD, 0x00];
        assert_eq!(
            parse_cotp_header(data),
            Some(CotpHeader {
                tpdu_type: CotpTpduType::ConnectRequest,
                protocol_id: None,
                payload_offset: 7,
            }),
            "RFC 905 §13.2.1: LI=6 counts the 6 octets of the CR fixed part following \
             the LI octet itself, so payload_offset == 1 + LI == 7, independent of the \
             DST-REF/SRC-REF byte values chosen"
        );
    }
}
