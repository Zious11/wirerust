//! MACsec ARP offset regression guards — synthetic frame harness.
//!
//! These tests guard that the lax-path ARP offset formula in `decode_packet`
//! (src/decoder.rs) correctly handles IEEE 802.1AE (MACsec) link extensions
//! for all reachable payload variants, and that Modified/Encrypted variants
//! (opaque payload) are structurally unreachable from the ARP truncation path.
//!
//! ## Synthetic frames
//!
//! No public MACsec-over-ARP pcap exists.  All frames are constructed by hand
//! using byte-exact layouts and etherparse's `MacsecHeader` builder.
//! The empirical results (offset values, reachability) back the documented
//! offset constants in BC-2.16.009 v1.8 and BC-2.16.015 v1.7 EC-009.
//!
//! ## What these tests guard
//!
//! ### V1 — no-SCI Unmodified, ARP truncated benign
//!
//! `test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22`
//!
//!   MACsec header without SCI, Unmodified ptype: `header_len() == 8`
//!   (6 SecTag + 2 next_EtherType).  ARP offset formula yields 14 + 8 = **22**.
//!   ARP fixed header at offset 22 has `hlen=6` → classified as
//!   `"truncated ARP frame"` (no false D11).
//!
//! ### V2 — no-SCI Unmodified, ARP malformed hlen=8
//!
//! `test_BC_2_16_009_macsec_no_sci_unmodified_arp_malformed_hlen8_routes_to_d11`
//!
//!   Same MACsec framing (offset 22).  ARP fixed header has `hlen=8` → formula
//!   reads the malformed byte at the correct offset → D11 fires.
//!
//! ### V3 — SCI-present Unmodified, ARP truncated benign (spec backing test)
//!
//! `test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30`
//!
//!   MACsec header with SCI, Unmodified ptype: `header_len() == 16`
//!   (6 SecTag + 8 SCI + 2 next_EtherType).  ARP offset formula yields
//!   14 + 16 = **30**.
//!
//!   THIS IS THE SPEC-BACKING TEST.  It empirically confirms the documented
//!   offset=30 for SCI-present MACsec frames (BC-2.16.015 v1.7 EC-009).
//!   A former off-by-8 risk existed: if `LaxLinkExtSlice::Macsec` returned
//!   `header_len() == 8` (omitting the 8 SCI bytes), the decoder would compute
//!   offset 22, peeking 8 bytes INTO the SCI field and producing a false D11.
//!   This test guards that `header_len()` returns 16, making offset 30 correct.
//!
//! ### V4 — SCI-present Unmodified, ARP malformed hlen=8
//!
//! `test_BC_2_16_009_macsec_sci_present_unmodified_arp_malformed_hlen8_routes_to_d11`
//!
//!   Same SCI-present framing (offset 30).  ARP `hlen=8` → D11 fires at the
//!   correct offset.
//!
//! ### V5 — no-SCI Modified, opaque payload (security-property guard)
//!
//! `test_BC_2_16_015_macsec_no_sci_modified_opaque_payload_unreachable`
//!
//!   Modified ptype (no inner EtherType): the lax parser does NOT reach
//!   `stop_err==Layer::Arp`.  The decoder's ARP-truncation path is structurally
//!   unreachable; the offset code never executes on ciphertext.
//!
//! ### V6 — SCI-present Modified, opaque payload (security-property guard)
//!
//! `test_BC_2_16_015_macsec_sci_present_modified_opaque_payload_unreachable`
//!
//!   Same security-property guard for the SCI-present Modified variant.
//!
//! ## Wire layouts
//!
//! ### 802.1AE SecTag (Unmodified, no SCI) — 8 bytes
//! ```text
//! [0]   TCI-AN byte: SC=0 (no SCI), E=0, C=0, AN=0
//! [1]   SL byte: short_len = 0
//! [2..6] PN: packet number (4 bytes, big-endian)
//! [6..8] next_EtherType: 0x0806 (ARP)
//! ```
//!
//! ### 802.1AE SecTag (Unmodified, SCI present) — 16 bytes
//! ```text
//! [0]   TCI-AN byte: SC=1 (SCI present), E=0, C=0, AN=0
//! [1]   SL byte: short_len = 0
//! [2..6] PN: packet number (4 bytes, big-endian)
//! [6..14] SCI: 8 bytes
//! [14..16] next_EtherType: 0x0806 (ARP)
//! ```
//!
//! Behavioral contracts covered:
//!   BC-2.16.009 PC3   malformed ARP MUST produce D11 regardless of MACsec framing
//!   BC-2.16.015 EC-009 SCI-present MACsec ARP offset documented as 30
//!
//! DF-TEST-NAMESPACE-001: all tests wrapped in `mod macsec_offset`.
//! DF-GREEN-DOC-TENSE-SWEEP: comments describe what the tests GUARD, written in
//!   the tense of a passing (already-correct) test — no RED-phase stub language.

#![allow(non_snake_case)]

mod macsec_offset {
    use etherparse::{
        EtherType, LaxLinkExtSlice, LaxMacsecPayloadSlice, LaxSlicedPacket, MacsecAn, MacsecHeader,
        MacsecPType, MacsecShortLen, err::Layer,
    };
    use pcap_file::DataLink;
    use wirerust::analyzer::arp::ArpAnalyzer;
    use wirerust::decoder::decode_packet;
    use wirerust::findings::ThreatCategory;

    // -------------------------------------------------------------------------
    // Shared constants
    // -------------------------------------------------------------------------

    /// Ethernet2 header length in bytes.
    const ETH2_LEN: usize = 14;

    /// ARP fixed header length in bytes.
    const ARP_FIXED_HDR_LEN: usize = 8;

    /// ARP payload offset for a MACsec frame without SCI (Unmodified ptype).
    ///
    /// `header_len()` for no-SCI Unmodified = 6 SecTag + 2 next_EtherType = 8.
    /// Offset = 14 + 8 = 22.  Backed empirically by V1 and V2.
    const ARP_OFFSET_MACSEC_NO_SCI: usize = 22;

    /// ARP payload offset for a MACsec frame with SCI present (Unmodified ptype).
    ///
    /// `header_len()` for SCI-present Unmodified = 6 SecTag + 8 SCI + 2 next_EtherType = 16.
    /// Offset = 14 + 16 = 30.  This is the spec-documented value (BC-2.16.015 v1.7 EC-009),
    /// empirically backed by V3 and V4.
    const ARP_OFFSET_MACSEC_SCI: usize = 30;

    // -------------------------------------------------------------------------
    // Fixture builders
    // -------------------------------------------------------------------------

    /// Build a MACsec header using etherparse and serialize it.
    ///
    /// Returns `(serialized_bytes, declared_header_len)` where `declared_header_len`
    /// is what `MacsecHeader::header_len()` reports and `serialized_bytes.len()`
    /// equals that value.
    fn build_macsec_header(ptype: MacsecPType, sci: Option<u64>) -> (Vec<u8>, usize) {
        let hdr = MacsecHeader {
            ptype,
            endstation_id: false,
            scb: false,
            an: MacsecAn::ZERO,
            short_len: MacsecShortLen::ZERO,
            packet_nr: 1,
            sci,
        };
        let declared_len = hdr.header_len();
        (hdr.to_bytes().to_vec(), declared_len)
    }

    /// ARP fixed header with BENIGN fields — no variable section.
    ///
    /// `htype=0x0001` (Ethernet), `ptype=0x0800` (IPv4), `hlen=6`, `plen=4`,
    /// `oper=1` (ARP Request).  Genuine snaplen truncation of a valid ARP frame:
    /// the 20-byte variable section is absent.
    fn arp_fixed_benign_truncated() -> [u8; ARP_FIXED_HDR_LEN] {
        [
            0x00, 0x01, // htype: 0x0001 (Ethernet)
            0x08, 0x00, // ptype: 0x0800 (IPv4)
            6,    // hlen: 6 — correct Ethernet MAC size
            4,    // plen: 4 — correct IPv4 address size
            0x00, 0x01, // oper: 1 (ARP Request)
        ]
    }

    /// ARP fixed header with MALFORMED `hlen=8` — no variable section.
    ///
    /// `hlen=8` is not a valid Ethernet hardware-address length; it triggers D11
    /// when read at the correct ARP offset.
    fn arp_fixed_malformed_hlen8() -> [u8; ARP_FIXED_HDR_LEN] {
        [
            0x00, 0x01, // htype: 0x0001 (Ethernet — type field is valid)
            0x08, 0x00, // ptype: 0x0800 (IPv4 — type field is valid)
            8,    // hlen: 8 — BAD (non-Ethernet hardware-address length)
            4,    // plen: 4 — correct IPv4 address size
            0x00, 0x01, // oper: 1 (ARP Request)
        ]
    }

    /// Build a complete Ethernet/MACsec/ARP frame.
    ///
    /// Returns `(frame_bytes, actual_arp_offset)` where `actual_arp_offset` is
    /// the byte index in the returned slice at which the ARP fixed header begins.
    fn build_macsec_arp_frame(macsec_hdr_bytes: &[u8], inner_arp: &[u8]) -> (Vec<u8>, usize) {
        let mut frame = Vec::with_capacity(ETH2_LEN + macsec_hdr_bytes.len() + inner_arp.len());
        // Ethernet header (EtherType 0x88E5 = MACsec / IEEE 802.1AE)
        frame.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]); // dst MAC broadcast
        frame.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]); // src MAC
        frame.extend_from_slice(&[0x88, 0xE5]); // EtherType: 0x88E5 (MACsec)
        let actual_arp_offset = ETH2_LEN + macsec_hdr_bytes.len();
        frame.extend_from_slice(macsec_hdr_bytes);
        frame.extend_from_slice(inner_arp);
        (frame, actual_arp_offset)
    }

    /// Build a complete Ethernet/MACsec frame with an OPAQUE payload.
    ///
    /// Used for Modified/Encrypted variants where no inner EtherType is present.
    /// Returns `(frame_bytes, notional_payload_offset)` — the notional offset is
    /// provided for documentation only; the ARP-truncation path is unreachable
    /// for these variants.
    fn build_macsec_opaque_frame(macsec_hdr_bytes: &[u8]) -> (Vec<u8>, usize) {
        let opaque_payload = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let mut frame =
            Vec::with_capacity(ETH2_LEN + macsec_hdr_bytes.len() + opaque_payload.len());
        frame.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]); // dst MAC broadcast
        frame.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]); // src MAC
        frame.extend_from_slice(&[0x88, 0xE5]); // EtherType: 0x88E5 (MACsec)
        let notional_payload_offset = ETH2_LEN + macsec_hdr_bytes.len();
        frame.extend_from_slice(macsec_hdr_bytes);
        frame.extend_from_slice(&opaque_payload);
        (frame, notional_payload_offset)
    }

    // -------------------------------------------------------------------------
    // Internal helper: confirm lax parse is reachable at Layer::Arp
    // -------------------------------------------------------------------------

    /// Returns `(reachable, macsec_header_len_sum, macsec_payload_is_unmodified)`.
    ///
    /// `reachable` is true when `lax.stop_err == Layer::Arp`, meaning the lax
    /// cursor walked through the MACsec header to an inner ARP layer but could
    /// not fully decode it — exactly the condition that triggers the offset-peek
    /// code in decoder.rs.
    fn probe_lax_macsec(frame: &[u8]) -> (bool, usize, bool) {
        let lax = LaxSlicedPacket::from_ethernet(frame)
            .expect("LaxSlicedPacket::from_ethernet must not panic on any Ethernet-framed input");

        let reachable = lax
            .stop_err
            .as_ref()
            .is_some_and(|(_, layer)| *layer == Layer::Arp);

        let header_len_sum: usize = lax.link_exts.iter().map(|e| e.header_len()).sum();

        let payload_is_unmodified = lax.link_exts.iter().any(|ext| {
            matches!(
                ext,
                LaxLinkExtSlice::Macsec(m)
                    if matches!(m.payload, LaxMacsecPayloadSlice::Unmodified(_))
            )
        });

        (reachable, header_len_sum, payload_is_unmodified)
    }

    // -------------------------------------------------------------------------
    // V1 — no-SCI Unmodified, ARP truncated benign
    // -------------------------------------------------------------------------

    /// Guards that a MACsec frame without SCI (Unmodified ptype) carrying a
    /// benign, snaplen-truncated ARP payload is classified as
    /// `"truncated ARP frame"` and does NOT produce a false D11 finding.
    ///
    /// Wire layout (30 bytes total):
    /// ```text
    /// [0..14]  Ethernet header (EtherType=0x88E5)
    /// [14..22] MACsec SecTag no-SCI Unmodified (6 SecTag + 2 next_EtherType(ARP))
    /// [22..30] ARP fixed header: htype=0x0001, ptype=0x0800, hlen=6, plen=4, oper=1
    /// ```
    ///
    /// `header_len()` = 8 → `arp_offset = 14 + 8 = 22`.  ARP fixed header at
    /// offset 22 has valid `htype=0x0001`, `hlen=6` → genuine truncation, no D11.
    ///
    /// BC-2.16.015 PC-7b: genuine truncation behind MACsec must NOT produce D11.
    #[test]
    fn test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22() {
        // ---- Pre-conditions: MACsec header geometry ----
        let (macsec_hdr_bytes, declared_len) =
            build_macsec_header(MacsecPType::Unmodified(EtherType::ARP), None);
        assert_eq!(
            declared_len, 8,
            "V1 pre-condition: MacsecHeader without SCI and Unmodified ptype must have \
             header_len == 8 (6 SecTag + 2 next_EtherType). Got {}. \
             etherparse changed MacsecHeader::header_len() — all MACsec offset \
             assumptions in decoder.rs must be re-verified.",
            declared_len
        );
        assert_eq!(
            macsec_hdr_bytes.len(),
            8,
            "V1 pre-condition: serialized MACsec header must be 8 bytes. Got {}.",
            macsec_hdr_bytes.len()
        );

        // ---- Build frame ----
        let arp = arp_fixed_benign_truncated();
        let (frame, actual_arp_offset) = build_macsec_arp_frame(&macsec_hdr_bytes, &arp);

        assert_eq!(
            frame.len(),
            30,
            "V1 fixture: frame must be 30 bytes (14 + 8 + 8). Got {}.",
            frame.len()
        );
        assert_eq!(
            actual_arp_offset, ARP_OFFSET_MACSEC_NO_SCI,
            "V1 fixture: actual ARP offset must be {} (= 14 Ethernet + 8 MACsec). Got {}.",
            ARP_OFFSET_MACSEC_NO_SCI, actual_arp_offset
        );

        // ---- Assert lax parse properties and offset formula ----
        let (reachable, header_len_sum, payload_is_unmodified) = probe_lax_macsec(&frame);

        assert!(
            payload_is_unmodified,
            "V1: MACsec payload must be Unmodified (inner EtherType readable). \
             If Modified/Encrypted, the ARP truncation path would be unreachable — \
             frame construction is wrong."
        );
        assert!(
            reachable,
            "V1: no-SCI Unmodified MACsec+ARP must be REACHABLE (lax.stop_err=Layer::Arp). \
             The decoder's ARP offset-peek code requires this condition."
        );
        assert_eq!(
            header_len_sum,
            8,
            "V1: Σ link_exts.header_len() must be 8 for no-SCI Unmodified MACsec. \
             Got {}. Decoder would compute arp_offset = {} instead of {}.",
            header_len_sum,
            ETH2_LEN + header_len_sum,
            ARP_OFFSET_MACSEC_NO_SCI
        );

        // ---- Assert the offset formula yields the spec constant ----
        let computed_arp_offset = ETH2_LEN + header_len_sum;
        assert_eq!(
            computed_arp_offset, ARP_OFFSET_MACSEC_NO_SCI,
            "V1: computed arp_offset must be {} (= 14 + 8). Got {}.",
            ARP_OFFSET_MACSEC_NO_SCI, computed_arp_offset
        );
        assert_eq!(
            computed_arp_offset, actual_arp_offset,
            "V1: computed arp_offset {} must equal actual ARP byte position {} in the frame.",
            computed_arp_offset, actual_arp_offset
        );

        // ---- Assert ARP bytes are readable at the computed offset ----
        let htype =
            u16::from_be_bytes([frame[computed_arp_offset], frame[computed_arp_offset + 1]]);
        let hlen = frame[computed_arp_offset + 4];
        assert_eq!(
            htype, 0x0001,
            "V1: ARP htype at computed offset {} must be 0x0001 (Ethernet). Got 0x{:04X}.",
            computed_arp_offset, htype
        );
        assert_eq!(
            hlen, 6,
            "V1: ARP hlen at computed offset {} must be 6. Got {}. \
             Offset formula is pointing at the wrong bytes.",
            computed_arp_offset, hlen
        );

        // ---- decode_packet must return "truncated ARP frame", not D11 ----
        let decode_result = decode_packet(&frame, DataLink::ETHERNET);
        assert!(
            decode_result.is_err(),
            "V1: decode_packet must return Err for a truncated MACsec ARP frame. Got Ok."
        );
        let err_msg = decode_result.unwrap_err().to_string();

        let mut analyzer = ArpAnalyzer::new(3, 50);
        if err_msg.contains("Non-Ethernet/IPv4 ARP frame") {
            analyzer.record_malformed(frame.len());
        }
        assert_eq!(
            analyzer.malformed_findings, 0,
            "BC-2.16.015 PC-7b V1: a no-SCI Unmodified MACsec frame carrying a benign \
             truncated ARP (hlen=6 at offset {}) must NOT produce a D11 malformed finding. \
             Got malformed_findings = {}. decode_packet error: '{}'. \
             Expected: 'truncated ARP frame'.",
            ARP_OFFSET_MACSEC_NO_SCI, analyzer.malformed_findings, err_msg
        );
        assert!(
            err_msg.contains("truncated ARP frame"),
            "BC-2.16.015 PC-7b V1: decode_packet must return 'truncated ARP frame'. \
             Got: '{err_msg}'."
        );

        // ---- Negative-offset diagnostic: guards against off-by-8 under-count bug ----
        //
        // If the decoder's header_len()-sum were wrong by 8 (e.g., MACsec header
        // counted as 0 bytes instead of 8), it would compute arp_offset = 14 + 0 = 14
        // and read frame[14..] as the ARP header — that is the SecTag's TCI-AN and SL
        // bytes (both 0x00 for no-SCI, AN=0, short_len=0).
        //
        // By construction: frame[14] = TCI-AN = 0x00, frame[15] = SL = 0x00
        //   → u16 BE = 0x0000, which is NOT a valid ARP htype (0x0001).
        //   → frame[18] = PN byte 4 = 0x00, which is NOT a valid hlen (6).
        //
        // This assertion independently rules out an off-by-8 offset bug: if the
        // decoder read at the wrong offset, it would NOT see valid ARP header fields
        // and would return a different error (not "truncated ARP frame"), causing
        // the primary assert above to fail.  This diagnostic makes V1 independently
        // sufficient to pin the correct offset without relying on other variants.
        const WRONG_OFFSET_V1: usize = ETH2_LEN; // = 14: off-by-8 under-count (no MACsec header)
        let htype_at_wrong_offset_v1 =
            u16::from_be_bytes([frame[WRONG_OFFSET_V1], frame[WRONG_OFFSET_V1 + 1]]);
        let hlen_at_wrong_offset_v1 = frame[WRONG_OFFSET_V1 + 4];
        assert_ne!(
            htype_at_wrong_offset_v1,
            0x0001,
            "V1 negative-offset diagnostic: frame bytes at wrong offset {} (off-by-8 \
             under-count) must NOT read as ARP htype=0x0001. By construction, frame[{}..{}] \
             are the SecTag TCI-AN (0x00) and SL (0x00) bytes — u16 BE = 0x{:04X}. \
             If this assertion fails, the diagnostic is inconclusive and the SCI fixture \
             bytes changed — review the frame construction.",
            WRONG_OFFSET_V1,
            WRONG_OFFSET_V1,
            WRONG_OFFSET_V1 + 2,
            htype_at_wrong_offset_v1
        );
        assert_ne!(
            hlen_at_wrong_offset_v1,
            6,
            "V1 negative-offset diagnostic: frame bytes at wrong offset {}+4={} (off-by-8 \
             under-count) must NOT read as valid hlen=6. By construction, frame[{}] is the \
             PN byte (0x00). Got {}.",
            WRONG_OFFSET_V1,
            WRONG_OFFSET_V1 + 4,
            WRONG_OFFSET_V1 + 4,
            hlen_at_wrong_offset_v1
        );
    }

    // -------------------------------------------------------------------------
    // V2 — no-SCI Unmodified, ARP malformed hlen=8
    // -------------------------------------------------------------------------

    /// Guards that a MACsec frame without SCI (Unmodified ptype) carrying an ARP
    /// fixed header with `hlen=8` is classified as `"Non-Ethernet/IPv4 ARP frame"`
    /// and produces a D11 malformed finding.
    ///
    /// Wire layout: identical to V1 but ARP `hlen=8` at offset 22.
    ///
    /// BC-2.16.009 PC3: malformed ARP MUST produce D11 regardless of MACsec framing.
    #[test]
    fn test_BC_2_16_009_macsec_no_sci_unmodified_arp_malformed_hlen8_routes_to_d11() {
        // ---- Pre-conditions ----
        let (macsec_hdr_bytes, declared_len) =
            build_macsec_header(MacsecPType::Unmodified(EtherType::ARP), None);
        assert_eq!(
            declared_len, 8,
            "V2 pre-condition: no-SCI Unmodified header_len must be 8. Got {}.",
            declared_len
        );

        // ---- Build frame ----
        let arp = arp_fixed_malformed_hlen8();
        let (frame, actual_arp_offset) = build_macsec_arp_frame(&macsec_hdr_bytes, &arp);

        assert_eq!(
            actual_arp_offset, ARP_OFFSET_MACSEC_NO_SCI,
            "V2 fixture: actual ARP offset must be {}. Got {}.",
            ARP_OFFSET_MACSEC_NO_SCI, actual_arp_offset
        );

        // ---- Assert lax parse properties and offset formula ----
        let (reachable, header_len_sum, _) = probe_lax_macsec(&frame);

        assert!(
            reachable,
            "V2: no-SCI Unmodified MACsec+ARP must be REACHABLE (lax.stop_err=Layer::Arp)."
        );
        assert_eq!(
            header_len_sum, 8,
            "V2: Σ link_exts.header_len() must be 8. Got {}.",
            header_len_sum
        );

        let computed_arp_offset = ETH2_LEN + header_len_sum;
        assert_eq!(
            computed_arp_offset, ARP_OFFSET_MACSEC_NO_SCI,
            "V2: computed arp_offset must be {}. Got {}.",
            ARP_OFFSET_MACSEC_NO_SCI, computed_arp_offset
        );
        assert_eq!(
            computed_arp_offset, actual_arp_offset,
            "V2: computed arp_offset {} must equal actual ARP byte position {}.",
            computed_arp_offset, actual_arp_offset
        );

        // ---- Assert hlen=8 is visible at the computed offset ----
        let hlen = frame[computed_arp_offset + 4];
        assert_eq!(
            hlen, 8,
            "V2: ARP hlen at computed offset {} must be 8 (malformed). Got {}. \
             Offset formula is pointing at the wrong bytes.",
            computed_arp_offset, hlen
        );

        // ---- decode_packet must route to D11 ----
        let decode_result = decode_packet(&frame, DataLink::ETHERNET);
        assert!(
            decode_result.is_err(),
            "V2: decode_packet must return Err for a malformed MACsec ARP frame. Got Ok."
        );
        let err_msg = decode_result.unwrap_err().to_string();

        let routed_to_d11 = err_msg.contains("Non-Ethernet/IPv4 ARP frame");
        let mut analyzer = ArpAnalyzer::new(3, 50);
        if routed_to_d11 {
            let findings = analyzer.record_malformed(frame.len());
            assert!(
                !findings.is_empty(),
                "BC-2.16.009 PC3 V2: record_malformed must emit at least one D11 finding \
                 for a genuinely malformed MACsec ARP frame (hlen=8 at offset {}). \
                 Got 0 findings.",
                computed_arp_offset + 4
            );
        }

        assert!(
            analyzer.malformed_findings >= 1,
            "BC-2.16.009 PC3 V2: no-SCI Unmodified MACsec with ARP hlen=8 at offset {} \
             must produce a D11 malformed finding. Got malformed_findings = {}. \
             decode_packet error: '{}'.",
            computed_arp_offset + 4,
            analyzer.malformed_findings,
            err_msg
        );

        // ---- D11 finding quality (parity with bc_2_16_qinq_macsec_offset_tests.rs) ----
        if routed_to_d11 {
            let mut analyzer2 = ArpAnalyzer::new(3, 50);
            let d11_findings = analyzer2.record_malformed(frame.len());
            let d11 = d11_findings
                .first()
                .expect("record_malformed must return at least one finding");
            assert_eq!(
                d11.category,
                ThreatCategory::Anomaly,
                "BC-2.16.009 QinQ Inv1: D11 finding must have category Anomaly. Got {:?}.",
                d11.category
            );
            assert!(
                d11.mitre_techniques.is_empty(),
                "BC-2.16.009 QinQ Inv3: D11 finding must have empty mitre_techniques \
                 (T0814 withheld per DF-VALIDATION-001). Got {:?}.",
                d11.mitre_techniques
            );
        }
    }

    // -------------------------------------------------------------------------
    // V3 — SCI-present Unmodified, ARP truncated benign (spec-backing test)
    // -------------------------------------------------------------------------

    /// Guards that a MACsec frame WITH SCI present (Unmodified ptype) carrying a
    /// benign, snaplen-truncated ARP payload is classified as `"truncated ARP frame"`
    /// and does NOT produce a false D11 finding.
    ///
    /// Wire layout (38 bytes total):
    /// ```text
    /// [0..14]  Ethernet header (EtherType=0x88E5)
    /// [14..30] MACsec SecTag SCI-present Unmodified (6 SecTag + 8 SCI + 2 next_EtherType(ARP))
    /// [30..38] ARP fixed header: htype=0x0001, ptype=0x0800, hlen=6, plen=4, oper=1
    /// ```
    ///
    /// THIS IS THE SPEC-BACKING TEST for BC-2.16.015 v1.7 EC-009 (offset=30).
    ///
    /// `header_len()` for SCI-present Unmodified = 16 → `arp_offset = 14 + 16 = 30`.
    /// The off-by-8 risk: if `LaxLinkExtSlice::Macsec` returned `header_len() == 8`
    /// (omitting the 8 SCI bytes), the decoder would compute offset 22, peeking
    /// into the SCI field and producing a false D11.  This test asserts
    /// `header_len() == 16`, making offset 30 correct, and confirms that ARP bytes
    /// at offset 30 are valid (htype=0x0001, hlen=6).
    ///
    /// BC-2.16.015 PC-7b: genuine truncation behind MACsec must NOT produce D11.
    #[test]
    fn test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30() {
        let sci_value: u64 = 0x0102_0304_0506_0708;

        // ---- Pre-conditions: MACsec header geometry (SCI-present Unmodified = 16 bytes) ----
        let (macsec_hdr_bytes, declared_len) =
            build_macsec_header(MacsecPType::Unmodified(EtherType::ARP), Some(sci_value));
        assert_eq!(
            declared_len, 16,
            "V3 pre-condition: MacsecHeader with SCI and Unmodified ptype must have \
             header_len == 16 (6 SecTag + 8 SCI + 2 next_EtherType). Got {}. \
             If 8 is returned, the SCI bytes are not counted — this is the off-by-8 risk.",
            declared_len
        );
        assert_eq!(
            macsec_hdr_bytes.len(),
            16,
            "V3 pre-condition: serialized SCI-present Unmodified MACsec header must be \
             16 bytes. Got {}.",
            macsec_hdr_bytes.len()
        );

        // ---- Build frame ----
        let arp = arp_fixed_benign_truncated();
        let (frame, actual_arp_offset) = build_macsec_arp_frame(&macsec_hdr_bytes, &arp);

        assert_eq!(
            frame.len(),
            38,
            "V3 fixture: frame must be 38 bytes (14 + 16 + 8). Got {}.",
            frame.len()
        );
        assert_eq!(
            actual_arp_offset, ARP_OFFSET_MACSEC_SCI,
            "V3 fixture: actual ARP offset must be {} (= 14 Ethernet + 16 MACsec). Got {}. \
             Frame construction is wrong — ARP bytes are not where expected.",
            ARP_OFFSET_MACSEC_SCI, actual_arp_offset
        );

        // ---- Assert lax parse properties and offset formula ----
        let (reachable, header_len_sum, payload_is_unmodified) = probe_lax_macsec(&frame);

        assert!(
            payload_is_unmodified,
            "V3: MACsec payload must be Unmodified (inner EtherType readable for ARP parsing)."
        );
        assert!(
            reachable,
            "V3: SCI-present Unmodified MACsec+ARP must be REACHABLE (lax.stop_err=Layer::Arp). \
             The ARP truncation path in decoder.rs requires this condition."
        );

        // CRITICAL ASSERTION: header_len_sum must be 16, not 8.
        // If 8 is returned, the SCI bytes are omitted from the offset computation,
        // producing arp_offset = 22 and reading 8 bytes into the SCI field as ARP
        // header bytes — a false D11.  This assertion is the primary empirical
        // backing for BC-2.16.015 v1.7 EC-009 offset=30.
        assert_eq!(
            header_len_sum, 16,
            "V3 CRITICAL: Σ link_exts.header_len() must be 16 for SCI-present Unmodified \
             MACsec (6 SecTag + 8 SCI + 2 next_EtherType). Got {}. \
             If 8 is returned, the decoder computes arp_offset = 14 + 8 = 22 instead of \
             14 + 16 = 30, reads into the SCI field as ARP bytes, and produces a false D11. \
             This directly invalidates the spec-documented offset=30 in BC-2.16.015 v1.7 EC-009.",
            header_len_sum
        );

        // ---- Assert the offset formula yields the spec constant ----
        let computed_arp_offset = ETH2_LEN + header_len_sum;
        assert_eq!(
            computed_arp_offset, ARP_OFFSET_MACSEC_SCI,
            "V3: computed arp_offset must be {} (= 14 + 16). Got {}. \
             BC-2.16.015 v1.7 EC-009 documents offset=30 for SCI-present MACsec.",
            ARP_OFFSET_MACSEC_SCI, computed_arp_offset
        );
        assert_eq!(
            computed_arp_offset, actual_arp_offset,
            "V3: computed arp_offset {} must equal actual ARP byte position {} in the frame. \
             This confirms the offset formula is correct for SCI-present MACsec frames.",
            computed_arp_offset, actual_arp_offset
        );

        // ---- Assert ARP bytes are readable at the computed offset ----
        // Diagnostic: show what would be read at the WRONG offset (22) if SCI were omitted.
        // At frame[22..30]: the 8 SCI bytes — not ARP bytes.  Reading those as ARP htype
        // would produce a garbage value and a false D11.
        //
        // OBS-1 HARDENING: The SCI value is sci_value = 0x0102_0304_0506_0708.
        // SecTag occupies frame[14..20] (6 bytes: TCI-AN, SL, PN[4]).
        // SCI occupies frame[20..28] (8 bytes, big-endian): 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08.
        // Therefore frame[22] = SCI byte 2 = 0x03, frame[23] = SCI byte 3 = 0x04.
        // u16 BE of frame[22..24] = 0x0304, which is pinned != 0x0001 by construction.
        // The next_EtherType bytes at frame[26..28] = 0x08, 0x06.
        //
        // If a future test author changes sci_value, they MUST ensure frame[22..24] still
        // does not equal 0x0001 (i.e., the SCI high bytes [2..4] must not be 0x00, 0x01).
        // The assertion below is unconditional; there is no "inconclusive" path.
        let sci_bytes_at_22_23: [u8; 2] = [frame[22], frame[23]];
        assert_eq!(
            sci_bytes_at_22_23,
            [0x03, 0x04],
            "V3 OBS-1 pre-condition: frame[22..24] must be SCI bytes 2-3 = [0x03, 0x04] \
             (from sci_value=0x0102_0304_0506_0708). Got {:?}. \
             If sci_value changed, verify that the new SCI bytes at positions [22..24] \
             are still != 0x00, 0x01 (which would make the negative-offset diagnostic \
             inconclusive). Recheck the frame layout and update this assertion.",
            sci_bytes_at_22_23
        );
        let htype_at_wrong_offset = u16::from_be_bytes([frame[22], frame[23]]);
        let htype_at_correct_offset = u16::from_be_bytes([
            frame[ARP_OFFSET_MACSEC_SCI],
            frame[ARP_OFFSET_MACSEC_SCI + 1],
        ]);
        // This assert_ne is now unconditionally non-trivial: 0x0304 != 0x0001 is guaranteed
        // by the sci_bytes_at_22_23 pre-condition assertion above.
        assert_ne!(
            htype_at_wrong_offset, 0x0001,
            "V3 negative-offset diagnostic: frame bytes at wrong offset 22 (SCI bytes) \
             must NOT read as ARP htype=0x0001. By construction frame[22..24] = [0x03, 0x04] \
             (SCI value 0x0102_0304_0506_0708 bytes 2-3), giving htype=0x0304. \
             Got 0x{:04X}. This is non-trivial: the SCI byte pre-condition above ensures \
             this diagnostic is never inconclusive.",
            htype_at_wrong_offset
        );
        assert_eq!(
            htype_at_correct_offset, 0x0001,
            "V3: ARP htype at correct offset {} must be 0x0001 (Ethernet). Got 0x{:04X}. \
             The offset formula is pointing at the wrong bytes.",
            ARP_OFFSET_MACSEC_SCI, htype_at_correct_offset
        );

        let hlen = frame[ARP_OFFSET_MACSEC_SCI + 4];
        assert_eq!(
            hlen, 6,
            "V3: ARP hlen at correct offset {} must be 6. Got {}. \
             The offset formula is pointing at the wrong bytes.",
            ARP_OFFSET_MACSEC_SCI, hlen
        );

        // ---- decode_packet must return "truncated ARP frame", not D11 ----
        let decode_result = decode_packet(&frame, DataLink::ETHERNET);
        assert!(
            decode_result.is_err(),
            "V3: decode_packet must return Err for a truncated MACsec ARP frame. Got Ok."
        );
        let err_msg = decode_result.unwrap_err().to_string();

        let mut analyzer = ArpAnalyzer::new(3, 50);
        if err_msg.contains("Non-Ethernet/IPv4 ARP frame") {
            analyzer.record_malformed(frame.len());
        }
        assert_eq!(
            analyzer.malformed_findings, 0,
            "BC-2.16.015 PC-7b V3: a SCI-present Unmodified MACsec frame carrying a benign \
             truncated ARP (hlen=6 at offset {}) must NOT produce a D11 malformed finding. \
             Got malformed_findings = {}. decode_packet error: '{}'. \
             Expected: 'truncated ARP frame'. \
             If 'Non-Ethernet/IPv4 ARP frame' was returned, the ARP offset formula is \
             reading the SCI bytes at offset 22 instead of the ARP bytes at offset 30.",
            ARP_OFFSET_MACSEC_SCI, analyzer.malformed_findings, err_msg
        );
        assert!(
            err_msg.contains("truncated ARP frame"),
            "BC-2.16.015 PC-7b V3: decode_packet must return 'truncated ARP frame' for \
             a genuinely-truncated SCI-present MACsec ARP frame. Got: '{err_msg}'."
        );
    }

    // -------------------------------------------------------------------------
    // V4 — SCI-present Unmodified, ARP malformed hlen=8
    // -------------------------------------------------------------------------

    /// Guards that a MACsec frame with SCI present (Unmodified ptype) carrying an
    /// ARP fixed header with `hlen=8` is classified as `"Non-Ethernet/IPv4 ARP frame"`
    /// and produces a D11 malformed finding.
    ///
    /// Wire layout: identical to V3 but ARP `hlen=8` at offset 30.
    ///
    /// BC-2.16.009 PC3: malformed ARP MUST produce D11 regardless of MACsec framing.
    #[test]
    fn test_BC_2_16_009_macsec_sci_present_unmodified_arp_malformed_hlen8_routes_to_d11() {
        let sci_value: u64 = 0xDEAD_BEEF_CAFE_BABE;

        // ---- Pre-conditions ----
        let (macsec_hdr_bytes, declared_len) =
            build_macsec_header(MacsecPType::Unmodified(EtherType::ARP), Some(sci_value));
        assert_eq!(
            declared_len, 16,
            "V4 pre-condition: SCI-present Unmodified header_len must be 16. Got {}.",
            declared_len
        );

        // ---- Build frame ----
        let arp = arp_fixed_malformed_hlen8();
        let (frame, actual_arp_offset) = build_macsec_arp_frame(&macsec_hdr_bytes, &arp);

        assert_eq!(
            actual_arp_offset, ARP_OFFSET_MACSEC_SCI,
            "V4 fixture: actual ARP offset must be {}. Got {}.",
            ARP_OFFSET_MACSEC_SCI, actual_arp_offset
        );

        // ---- Assert lax parse properties and offset formula ----
        let (reachable, header_len_sum, _) = probe_lax_macsec(&frame);

        assert!(
            reachable,
            "V4: SCI-present Unmodified MACsec+ARP must be REACHABLE (lax.stop_err=Layer::Arp)."
        );
        assert_eq!(
            header_len_sum, 16,
            "V4: Σ link_exts.header_len() must be 16 for SCI-present Unmodified. Got {}.",
            header_len_sum
        );

        let computed_arp_offset = ETH2_LEN + header_len_sum;
        assert_eq!(
            computed_arp_offset, ARP_OFFSET_MACSEC_SCI,
            "V4: computed arp_offset must be {}. Got {}.",
            ARP_OFFSET_MACSEC_SCI, computed_arp_offset
        );
        assert_eq!(
            computed_arp_offset, actual_arp_offset,
            "V4: computed arp_offset {} must equal actual ARP byte position {}.",
            computed_arp_offset, actual_arp_offset
        );

        // ---- Assert hlen=8 is visible at the computed offset ----
        let hlen = frame[computed_arp_offset + 4];
        assert_eq!(
            hlen, 8,
            "V4: ARP hlen at computed offset {} must be 8 (malformed). Got {}. \
             Offset formula is pointing at the wrong bytes.",
            computed_arp_offset, hlen
        );

        // ---- decode_packet must route to D11 ----
        let decode_result = decode_packet(&frame, DataLink::ETHERNET);
        assert!(
            decode_result.is_err(),
            "V4: decode_packet must return Err for a malformed SCI-present MACsec ARP frame. \
             Got Ok."
        );
        let err_msg = decode_result.unwrap_err().to_string();

        let routed_to_d11 = err_msg.contains("Non-Ethernet/IPv4 ARP frame");
        let mut analyzer = ArpAnalyzer::new(3, 50);
        if routed_to_d11 {
            let findings = analyzer.record_malformed(frame.len());
            assert!(
                !findings.is_empty(),
                "BC-2.16.009 PC3 V4: record_malformed must emit at least one D11 finding. \
                 Got 0 findings."
            );
        }

        assert!(
            analyzer.malformed_findings >= 1,
            "BC-2.16.009 PC3 V4: SCI-present Unmodified MACsec with ARP hlen=8 at offset {} \
             must produce a D11 malformed finding. Got malformed_findings = {}. \
             decode_packet error: '{}'.",
            computed_arp_offset + 4,
            analyzer.malformed_findings,
            err_msg
        );

        // ---- D11 finding quality (parity with bc_2_16_qinq_macsec_offset_tests.rs) ----
        if routed_to_d11 {
            let mut analyzer2 = ArpAnalyzer::new(3, 50);
            let d11_findings = analyzer2.record_malformed(frame.len());
            let d11 = d11_findings
                .first()
                .expect("record_malformed must return at least one finding");
            assert_eq!(
                d11.category,
                ThreatCategory::Anomaly,
                "BC-2.16.009 Inv1 V4: D11 finding must have category Anomaly. Got {:?}.",
                d11.category
            );
            assert!(
                d11.mitre_techniques.is_empty(),
                "BC-2.16.009 Inv3 V4: D11 finding must have empty mitre_techniques \
                 (T0814 withheld per DF-VALIDATION-001). Got {:?}.",
                d11.mitre_techniques
            );
        }
    }

    // -------------------------------------------------------------------------
    // V5 — no-SCI Modified, opaque payload (security-property guard)
    // -------------------------------------------------------------------------

    /// Guards that a MACsec Modified frame (no-SCI, opaque payload) does NOT
    /// reach `lax.stop_err == Layer::Arp`.  The decoder's ARP truncation path is
    /// structurally unreachable; the offset-peek code never executes on ciphertext.
    ///
    /// Wire layout:
    /// ```text
    /// [0..14]  Ethernet header (EtherType=0x88E5)
    /// [14..20] MACsec SecTag no-SCI Modified (6 bytes, no next_EtherType)
    /// [20..28] Opaque payload (8 bytes — inner content not an ARP header)
    /// ```
    ///
    /// Modified ptype omits the inner EtherType; `LaxMacsecPayloadSlice::Modified`
    /// has an opaque payload, so the lax cursor cannot continue into an inner ARP
    /// layer.  `stop_err` will NOT be `Layer::Arp`.
    ///
    /// This is a SECURITY-PROPERTY regression guard: if etherparse ever incorrectly
    /// exposed the opaque payload as a readable inner ARP layer, the decoder would
    /// attempt to interpret ciphertext as ARP bytes.  This test guards against that.
    #[test]
    fn test_BC_2_16_015_macsec_no_sci_modified_opaque_payload_unreachable() {
        // ---- Pre-conditions ----
        let (macsec_hdr_bytes, declared_len) = build_macsec_header(MacsecPType::Modified, None);
        assert_eq!(
            declared_len, 6,
            "V5 pre-condition: no-SCI Modified header_len must be 6 (SecTag only, no \
             next_EtherType). Got {}.",
            declared_len
        );

        // ---- Build frame ----
        let (frame, _notional_offset) = build_macsec_opaque_frame(&macsec_hdr_bytes);

        // ---- Assert Modified payload is NOT reachable as Layer::Arp ----
        let lax = LaxSlicedPacket::from_ethernet(&frame)
            .expect("LaxSlicedPacket::from_ethernet must not panic on any Ethernet-framed input");

        let reachable_as_arp = lax
            .stop_err
            .as_ref()
            .is_some_and(|(_, layer)| *layer == Layer::Arp);

        // Confirm payload is Modified/opaque.
        let payload_is_modified = lax.link_exts.iter().any(|ext| {
            matches!(
                ext,
                LaxLinkExtSlice::Macsec(m)
                    if matches!(m.payload, LaxMacsecPayloadSlice::Modified { .. })
            )
        });
        assert!(
            payload_is_modified,
            "V5: MACsec payload must be Modified/opaque. Got Unmodified — frame construction \
             is wrong."
        );

        // SECURITY PROPERTY: the parser must NOT reach Layer::Arp for an opaque payload.
        // If this assertion fails, etherparse has changed its behavior in a way that would
        // allow the decoder to read opaque ciphertext bytes as ARP fields.
        assert!(
            !reachable_as_arp,
            "BC-2.16.015 V5 SECURITY GUARD: Modified MACsec (opaque payload, no inner \
             EtherType) must NOT reach lax.stop_err == Layer::Arp. \
             If reachable, the decoder would attempt to read ciphertext as ARP bytes. \
             lax.stop_err = {:?}. \
             etherparse behavior change — all MACsec decoder safety assumptions must \
             be re-verified.",
            lax.stop_err
        );

        // Confirm offset_match semantics: no offset assertion is possible or needed
        // for an unreachable variant.
        assert_eq!(
            lax.link_exts.len(),
            1,
            "V5: must have 1 Macsec link_ext. Got {}.",
            lax.link_exts.len()
        );
        let header_len_sum: usize = lax.link_exts.iter().map(|e| e.header_len()).sum();
        assert_eq!(
            header_len_sum, 6,
            "V5: Σ link_exts.header_len() must be 6 for no-SCI Modified. Got {}.",
            header_len_sum
        );
    }

    // -------------------------------------------------------------------------
    // V6 — SCI-present Modified, opaque payload (security-property guard)
    // -------------------------------------------------------------------------

    /// Guards that a MACsec Modified frame WITH SCI present (opaque payload) does
    /// NOT reach `lax.stop_err == Layer::Arp`.  Same security property as V5 but
    /// for the SCI-present case.
    ///
    /// Wire layout:
    /// ```text
    /// [0..14]  Ethernet header (EtherType=0x88E5)
    /// [14..28] MACsec SecTag SCI-present Modified (6 SecTag + 8 SCI = 14 bytes, no next_EtherType)
    /// [28..36] Opaque payload (8 bytes)
    /// ```
    ///
    /// BC-2.16.015: SCI-present Modified MACsec must remain unreachable from the
    /// ARP truncation path.
    #[test]
    fn test_BC_2_16_015_macsec_sci_present_modified_opaque_payload_unreachable() {
        let sci_value: u64 = 0x1122_3344_5566_7788;

        // ---- Pre-conditions ----
        let (macsec_hdr_bytes, declared_len) =
            build_macsec_header(MacsecPType::Modified, Some(sci_value));
        assert_eq!(
            declared_len, 14,
            "V6 pre-condition: SCI-present Modified header_len must be 14 (6 SecTag + \
             8 SCI, no next_EtherType). Got {}.",
            declared_len
        );

        // ---- Build frame ----
        let (frame, _notional_offset) = build_macsec_opaque_frame(&macsec_hdr_bytes);

        // ---- Assert SCI-present Modified payload is NOT reachable as Layer::Arp ----
        let lax = LaxSlicedPacket::from_ethernet(&frame)
            .expect("LaxSlicedPacket::from_ethernet must not panic on any Ethernet-framed input");

        let reachable_as_arp = lax
            .stop_err
            .as_ref()
            .is_some_and(|(_, layer)| *layer == Layer::Arp);

        let payload_is_modified = lax.link_exts.iter().any(|ext| {
            matches!(
                ext,
                LaxLinkExtSlice::Macsec(m)
                    if matches!(m.payload, LaxMacsecPayloadSlice::Modified { .. })
            )
        });
        assert!(
            payload_is_modified,
            "V6: MACsec payload must be Modified/opaque. Got Unmodified — frame construction \
             is wrong."
        );

        // SECURITY PROPERTY: SCI-present Modified must also not reach Layer::Arp.
        assert!(
            !reachable_as_arp,
            "BC-2.16.015 V6 SECURITY GUARD: SCI-present Modified MACsec (opaque payload) must \
             NOT reach lax.stop_err == Layer::Arp. \
             lax.stop_err = {:?}. \
             etherparse behavior change — all MACsec decoder safety assumptions must \
             be re-verified.",
            lax.stop_err
        );

        assert_eq!(
            lax.link_exts.len(),
            1,
            "V6: must have 1 Macsec link_ext. Got {}.",
            lax.link_exts.len()
        );
        let header_len_sum: usize = lax.link_exts.iter().map(|e| e.header_len()).sum();
        assert_eq!(
            header_len_sum, 14,
            "V6: Σ link_exts.header_len() must be 14 for SCI-present Modified. Got {}.",
            header_len_sum
        );
    }
}
