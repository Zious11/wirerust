//! Tests for STORY-184: S7comm TPKT Core Parser (pure-core free function).
//!
//! Covers BC-2.20.001 through BC-2.20.004 and the edge cases enumerated in each BC.
//!
//! ## Contract coverage
//! - BC-2.20.001: `parse_tpkt_header` returns `None` for input shorter than 4 bytes.
//! - BC-2.20.002: `parse_tpkt_header` returns `None` for version byte != 0x03 (also the
//!   SS-20 resync anchor).
//! - BC-2.20.003: `parse_tpkt_header` returns `None` for decoded length field < 4
//!   (malformed, includes zero-length).
//! - BC-2.20.004: `parse_tpkt_header` returns `Some(TpktHeader)` for valid input (happy
//!   path); reserved byte (`data[1]`) is never validated; `length == 65535` is a legal
//!   accept; the four BC-2.20.001-004 outcomes are jointly exhaustive and mutually
//!   exclusive (AC-184-005).
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
    /// decode as a legal `[4, 65535]` value must still return `None`.
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
    // BC-2.20.003: parse_tpkt_header returns None for length field < 4
    // AC-184-003
    // =========================================================================

    /// BC-2.20.003 canonical vector: length=0 (zero-length, most degenerate case) returns
    /// None.
    ///
    /// Canonical vector from BC-2.20.003: `[0x03, 0x00, 0x00, 0x00]` (length=0) -> None.
    /// Preconditions: data.len() >= 4, data[0] == 0x03 (version passes), decoded length
    /// (0) < 4.
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

    /// BC-2.20.003 canonical vector: length=3 (one below minimum) returns None.
    ///
    /// Canonical vector from BC-2.20.003: `[0x03, 0x00, 0x00, 0x03]` (length=3) -> None.
    /// This is the boundary immediately adjacent to the accept-path minimum (length=4,
    /// BC-2.20.004 EC-001).
    ///
    /// Traces: BC-2.20.003 postcondition 1; AC-184-003; EC-003; canonical test vector.
    #[test]
    fn test_BC_2_20_003_returns_none_for_length_three_off_by_one_canonical_vector() {
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x03];
        let result = parse_tpkt_header(data);
        assert!(
            result.is_none(),
            "length=3 (below minimum) must return None (BC-2.20.003 canonical vector)"
        );
    }

    /// BC-2.20.003 invariant: no overflow/panic for any `u16` length value below 4,
    /// including the all-zero length-field byte pattern.
    ///
    /// Traces: BC-2.20.003 invariant 2; AC-184-003; EC-005.
    #[test]
    fn test_BC_2_20_003_invariant_no_panic_across_sub_minimum_lengths() {
        for length_bytes in [[0x00u8, 0x00], [0x00, 0x01], [0x00, 0x02], [0x00, 0x03]] {
            let data: [u8; 4] = [0x03, 0xAB, length_bytes[0], length_bytes[1]];
            let result = parse_tpkt_header(&data);
            assert!(
                result.is_none(),
                "length bytes {length_bytes:?} (decoded < 4) must return None (BC-2.20.003)"
            );
        }
    }

    // =========================================================================
    // BC-2.20.004: parse_tpkt_header returns Some(TpktHeader) for valid input
    // AC-184-004
    // =========================================================================

    /// BC-2.20.004 canonical vector: length=4 (exactly minimum, header-only TPKT packet).
    ///
    /// Canonical vector from BC-2.20.004 / BC-2.20.003 EC-004:
    ///   `[0x03, 0x00, 0x00, 0x04]` -> `Some(TpktHeader { version: 3, length: 4 })`.
    ///
    /// Traces: BC-2.20.004 postconditions 1-3; AC-184-004; EC-001; canonical test vector.
    #[test]
    fn test_BC_2_20_004_valid_input_returns_some_header_length_4_canonical_vector() {
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x04];
        let result = parse_tpkt_header(data);
        let header = result.expect(
            "length=4 (exact minimum) must return Some (BC-2.20.004 canonical vector, \
             postcondition 1)",
        );
        assert_eq!(
            header,
            TpktHeader {
                version: 3,
                length: 4
            },
            "must decode version=3, length=4 exactly (BC-2.20.004 postcondition 1)"
        );
    }

    /// BC-2.20.004 canonical vector: length=7 (minimal CR/CC-carrying frame).
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
    /// Compares `[0x03, 0x00, 0x00, 0x04]` (reserved=0x00) against
    /// `[0x03, 0xFF, 0x00, 0x04]` (reserved=0xFF, EC-003): both must decode to the same
    /// `TpktHeader { version: 3, length: 4 }`.
    ///
    /// Traces: BC-2.20.004 postcondition 2, invariant 1; AC-184-004; EC-003.
    #[test]
    fn test_BC_2_20_004_reserved_byte_nonzero_parses_identically_to_zero() {
        let reserved_zero: &[u8] = &[0x03, 0x00, 0x00, 0x04];
        let reserved_nonzero: &[u8] = &[0x03, 0xFF, 0x00, 0x04];

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
                length: 4
            },
            "non-zero reserved byte must still decode version=3, length=4 (BC-2.20.004)"
        );
    }

    /// BC-2.20.004 EC-005: `data.len() == length as usize` exactly (single complete frame,
    /// no trailing bytes) is accepted.
    ///
    /// Traces: BC-2.20.004 postcondition 1; AC-184-004; EC-005.
    #[test]
    fn test_BC_2_20_004_exact_length_match_no_trailing_bytes() {
        // length = 6 (header + 2 payload bytes); data.len() == 6 exactly.
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x06, 0xAA, 0xBB];
        let result = parse_tpkt_header(data);
        let header =
            result.expect("exact-length-match input must return Some (BC-2.20.004 EC-005)");
        assert_eq!(
            header,
            TpktHeader {
                version: 3,
                length: 6
            },
            "must decode version=3, length=6 with data.len() == length exactly (EC-005)"
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
            // Class C: bad length (BC-2.20.003) — fires once len >= 4 and version == 0x03.
            (&[0x03, 0x00, 0x00, 0x00], None),
            (&[0x03, 0x00, 0x00, 0x03], None),
            // Class D: accept (BC-2.20.004) — len >= 4, version == 0x03, length in
            // [4, 65535].
            (
                &[0x03, 0x00, 0x00, 0x04],
                Some(TpktHeader {
                    version: 3,
                    length: 4,
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
    /// (4-byte header + 3-byte minimum COTP), NOT 4. This is the genuinely RFC-conformant
    /// minimum-length vector.
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

    /// DOCUMENTED DIVERGENCE (not RFC conformance): wirerust intentionally accepts
    /// length=4 (the TPKT header's own 4-byte structural floor), which is BELOW RFC 1006
    /// §6's stated min=7. This is a deliberate layering choice per ADR-014: the TPKT layer
    /// validates only structural framing; COTP-presence and semantic packet validity are
    /// enforced by the COTP layer (SS-21, STORY-185+). A length-4 TPKT parses here but is
    /// rejected downstream when the COTP parser receives 0 payload bytes.
    ///
    /// This test asserts wirerust's CURRENT lenient-framing behavior, not RFC conformance
    /// -- do not read `Some(length:4)` here as an RFC-valid vector.
    ///
    /// Traces: DF-CANONICAL-FRAME-HOLDOUT-001 (spec-independent holdout, not a BC vector);
    /// ADR-014.
    #[test]
    fn test_rfc1006_s6_length_four_wirerust_divergence_holdout() {
        let data: &[u8] = &[0x03, 0x00, 0x00, 0x04];
        let result = parse_tpkt_header(data);
        assert_eq!(
            result,
            Some(TpktHeader {
                version: 0x03,
                length: 4
            }),
            "wirerust intentionally accepts length=4, below RFC 1006 §6's stated min=7 \
             (documented layering divergence, ADR-014, not RFC conformance)"
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
        /// `test_rfc1006_s5_canonical_minimal_tpkt_holdout` below, whose vector is derived
        /// directly from RFC 1006 §5 rather than from this project's BCs
        /// (DF-CANONICAL-FRAME-HOLDOUT-001).
        fn oracle(data: &[u8]) -> Option<TpktHeader> {
            if data.len() < 4 {
                return None;
            }
            if data[0] != 0x03 {
                return None;
            }
            let length = u16::from_be_bytes([data[2], data[3]]);
            if length < 4 {
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
                prop_assume!(decoded >= 4);
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
