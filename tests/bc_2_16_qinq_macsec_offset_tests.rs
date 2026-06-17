//! QinQ double-tagged and MACsec ARP frame offset regression guards.
//!
//! These tests guard that the lax-path ARP offset formula in `decode_packet`
//! (src/decoder.rs) correctly handles stacked link extensions beyond the
//! single-VLAN case already covered by bc_2_16_d078_vlan_offset_tests.rs.
//!
//! ## What these tests exercise
//!
//! ### Test 1 — QinQ benign truncated: no D11 false positive
//!
//! `test_BC_2_16_015_qinq_truncated_benign_arp_no_false_positive_d11`
//!
//!   A QinQ double-tagged (0x88a8 outer + 0x8100 inner) ARP frame with a valid
//!   ARP fixed header (htype=0x0001, ptype=0x0800, hlen=6, plen=4) but NO
//!   variable section (truncated before sender/target addresses).
//!   ARP starts at offset 22 (14 Ethernet + 4 outer 802.1Q + 4 inner 802.1Q).
//!
//!   GUARDS: `lax.link_exts` contains TWO `Vlan` entries, each `header_len()==4`,
//!   `Σ header_len() == 8`, decode result is `Err("truncated ARP frame")`, and
//!   `malformed_findings == 0` (no false D11).
//!
//! ### Test 2 — QinQ malformed hlen=8: routes to D11
//!
//! `test_BC_2_16_009_qinq_malformed_hlen8_routes_to_d11`
//!
//!   Same QinQ double-tagged framing but with `hlen=8` (non-Ethernet
//!   hardware-address length) in the ARP fixed header.
//!
//!   GUARDS: decode result is `Err("Non-Ethernet/IPv4 ARP frame")`,
//!   `malformed_findings >= 1`, finding has `category == Anomaly`, and
//!   `mitre_techniques` is empty (D11 quality parity with d078 tests).
//!
//! ### Test 3 — Offset-formula probe
//!
//! `test_BC_2_16_015_qinq_link_exts_offset_formula_pin`
//!
//!   Directly asserts the etherparse representation of QinQ frames: two separate
//!   `Vlan` entries (no `VlanDouble` variant exists in etherparse 0.20.2), each
//!   reporting `header_len() == 4`, summing to 8.  Also confirms single-VLAN
//!   control gives `Σ == 4`.  This pins the etherparse data-model assumption so
//!   a future version bump that changes the representation fails loudly.
//!
//! ### Test 4 — MACsec parse probe (observe-only for this file; offset asserted in e17 file)
//!
//! `test_BC_2_16_015_macsec_arp_lax_parse_probe`
//!
//!   Builds an Ethernet / MACsec(Unmodified, ptype=ARP) / ARP frame using
//!   etherparse's `MacsecHeader` builder.  Runs `LaxSlicedPacket::from_ethernet`
//!   and records the observed `link_exts` shape and `header_len()` value.
//!
//!   ASSERTS: no panic, exactly one `Macsec` entry in `link_exts`, and the
//!   observed `header_len()` equals the pre-computed `MacsecHeader::header_len()`
//!   value (8 without SCI, 16 with SCI).  Offset correctness — that
//!   `arp_offset = 14 + link_exts_sum` correctly positions the ARP payload —
//!   is synthetically asserted in `bc_2_16_e17_macsec_offset_tests.rs`
//!   (`test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22` and
//!   `test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30`).
//!
//!   What remains unverified is the existence/behavior of MACsec-over-ARP in
//!   REAL on-wire captured traffic (BC-2.16.009/015 EC-009 part c).  That gap
//!   is a real-world fixture gap, not an offset-formula gap.  This probe is
//!   intentionally observe-only within this file so that MACsec shape/no-panic
//!   coverage lives here and offset-correctness assertions live exclusively in
//!   the e17 file, keeping the two concerns cleanly separated.
//!
//! ## Wire layouts
//!
//! ### QinQ benign truncated — 30 bytes total
//! ```text
//! [0..6]   dst MAC: FF:FF:FF:FF:FF:FF
//! [6..12]  src MAC: AA:BB:CC:DD:EE:FF
//! [12..14] outer EtherType: 0x88A8 (802.1ad S-Tag)
//! [14..16] outer TCI: 0x0020 (VID=32)
//! [16..18] inner EtherType: 0x8100 (802.1Q C-Tag)
//! [18..20] inner TCI: 0x0064 (VID=100)
//! [20..22] inner EtherType: 0x0806 (ARP)
//! [22..30] ARP fixed header: htype=0x0001, ptype=0x0800, hlen=6, plen=4, oper=0x0001
//! --- NO variable section ---
//! ```
//!
//! ### QinQ malformed hlen=8 — 30 bytes total
//! Identical framing but ARP fixed header has hlen=8 (BAD).
//!
//! Behavioral contracts covered:
//!   BC-2.16.009 PC3   malformed ARP MUST produce D11 regardless of tag framing
//!   BC-2.16.015 PC-7b genuine truncation behind QinQ tags MUST NOT produce D11
//!
//! DF-TEST-NAMESPACE-001: all tests wrapped in `mod qinq_macsec_offset`.
//! DF-GREEN-DOC-TENSE-SWEEP: comments describe what the tests GUARD, written in
//!   the tense of a passing (already-correct) test — no RED-phase stub language.

#![allow(non_snake_case)]

mod qinq_macsec_offset {
    use etherparse::err::Layer;
    use etherparse::{
        EtherType, LaxLinkExtSlice, LaxSlicedPacket, MacsecAn, MacsecHeader, MacsecPType,
        MacsecShortLen,
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

    /// IEEE 802.1Q / 802.1ad single tag length in bytes (2 TCI + 2 inner EtherType).
    const VLAN_TAG_LEN: usize = 4;

    /// QinQ double-tag total length: outer (4) + inner (4) = 8.
    const QINQ_TAGS_LEN: usize = VLAN_TAG_LEN * 2; // 8

    /// ARP payload offset for a QinQ double-tagged Ethernet frame.
    const ARP_OFFSET_QINQ: usize = ETH2_LEN + QINQ_TAGS_LEN; // 22

    // -------------------------------------------------------------------------
    // Fixture builders
    // -------------------------------------------------------------------------

    /// Build a QinQ double-tagged ARP frame with BENIGN inner ARP fixed header
    /// (`htype=0x0001, ptype=0x0800, hlen=6, plen=4`) and NO variable section.
    ///
    /// Wire layout (30 bytes total):
    /// ```text
    /// [0..6]   dst MAC: FF:FF:FF:FF:FF:FF
    /// [6..12]  src MAC: AA:BB:CC:DD:EE:FF
    /// [12..14] outer EtherType: 0x88A8 (802.1ad S-Tag / Provider Bridging)
    /// [14..16] outer TCI: 0x0020 (PCP=0, DEI=0, VID=32)
    /// [16..18] inner EtherType: 0x8100 (802.1Q C-Tag)
    /// [18..20] inner TCI: 0x0064 (PCP=0, DEI=0, VID=100)
    /// [20..22] inner EtherType: 0x0806 (ARP)
    /// [22..30] ARP fixed header: htype=0x0001, ptype=0x0800, hlen=6, plen=4, oper=0x0001
    /// ```
    ///
    /// For hlen=6, plen=4 the ARP spec requires 28 bytes of ARP payload; only 8
    /// are present (the fixed header only).  This is genuine snaplen truncation
    /// of a valid Ethernet/IPv4 ARP frame behind two VLAN tags.
    ///
    /// etherparse 0.20.2 represents QinQ as TWO separate `LaxLinkExtSlice::Vlan`
    /// entries (there is no `VlanDouble` variant), so `lax.link_exts.len() == 2`.
    fn make_qinq_benign_truncated_arp() -> Vec<u8> {
        let mut frame = Vec::with_capacity(30);
        // Ethernet header (14 bytes)
        frame.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]); // dst MAC broadcast
        frame.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]); // src MAC
        frame.extend_from_slice(&[0x88, 0xA8]); // EtherType: 802.1ad (S-Tag / QinQ outer)
        // Outer 802.1ad tag (4 bytes)
        frame.extend_from_slice(&[0x00, 0x20]); // outer TCI: PCP=0, DEI=0, VID=32
        frame.extend_from_slice(&[0x81, 0x00]); // inner EtherType: 802.1Q C-Tag
        // Inner 802.1Q tag (4 bytes)
        frame.extend_from_slice(&[0x00, 0x64]); // inner TCI: PCP=0, DEI=0, VID=100
        frame.extend_from_slice(&[0x08, 0x06]); // payload EtherType: ARP
        // ARP fixed header (8 bytes) — BENIGN: hlen=6 (Ethernet), plen=4 (IPv4)
        frame.extend_from_slice(&[0x00, 0x01]); // htype: Ethernet
        frame.extend_from_slice(&[0x08, 0x00]); // ptype: IPv4
        frame.push(6); // hlen: 6 — correct Ethernet MAC size
        frame.push(4); // plen: 4 — correct IPv4 address size
        frame.extend_from_slice(&[0x00, 0x01]); // oper: ARP Request
        // No variable data section — 20 variable bytes missing.
        assert_eq!(
            frame.len(),
            30,
            "QinQ benign truncated fixture: total frame must be 30 bytes \
             (14 Ethernet + 4 outer VLAN + 4 inner VLAN + 8 ARP fixed header)"
        );
        frame
    }

    /// Build a QinQ double-tagged ARP frame with a MALFORMED inner ARP fixed
    /// header (`hlen=8`, non-Ethernet hardware-address length) and NO variable section.
    ///
    /// Wire layout (30 bytes total): identical framing as above, but ARP hlen=8(BAD).
    fn make_qinq_malformed_arp_hlen8() -> Vec<u8> {
        let mut frame = Vec::with_capacity(30);
        // Ethernet header (14 bytes)
        frame.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]); // dst MAC broadcast
        frame.extend_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]); // src MAC
        frame.extend_from_slice(&[0x88, 0xA8]); // EtherType: 802.1ad (S-Tag)
        // Outer 802.1ad tag (4 bytes)
        frame.extend_from_slice(&[0x00, 0x20]); // outer TCI: VID=32
        frame.extend_from_slice(&[0x81, 0x00]); // inner EtherType: 802.1Q C-Tag
        // Inner 802.1Q tag (4 bytes)
        frame.extend_from_slice(&[0x00, 0x64]); // inner TCI: VID=100
        frame.extend_from_slice(&[0x08, 0x06]); // payload EtherType: ARP
        // ARP fixed header (8 bytes) — MALFORMED: hlen=8 (non-Ethernet)
        frame.extend_from_slice(&[0x00, 0x01]); // htype: Ethernet (type field is OK)
        frame.extend_from_slice(&[0x08, 0x00]); // ptype: IPv4 (type field is OK)
        frame.push(8); // hlen: 8 — BAD (non-Ethernet hardware-address size)
        frame.push(4); // plen: 4 — correct IPv4 address size
        frame.extend_from_slice(&[0x00, 0x01]); // oper: ARP Request
        // No variable data section.
        assert_eq!(
            frame.len(),
            30,
            "QinQ malformed truncated fixture: total frame must be 30 bytes \
             (14 Ethernet + 4 outer VLAN + 4 inner VLAN + 8 ARP fixed header with hlen=8)"
        );
        frame
    }

    // -------------------------------------------------------------------------
    // Probe helper
    // -------------------------------------------------------------------------

    /// Confirm a frame goes through the lax `None` arm with `stop_err=Layer::Arp`.
    /// Returns `(lax_net_is_none, stop_is_arp, vlan_count)`.
    fn probe_lax_arm_qinq(frame: &[u8]) -> (bool, bool, usize) {
        let lax = LaxSlicedPacket::from_ethernet(frame)
            .expect("LaxSlicedPacket::from_ethernet must succeed for any Ethernet-framed input");

        let lax_net_is_none = lax.net.is_none();
        let stop_is_arp = lax
            .stop_err
            .as_ref()
            .is_some_and(|(_, layer)| *layer == Layer::Arp);
        let vlan_count = lax
            .link_exts
            .iter()
            .filter(|ext| matches!(ext, LaxLinkExtSlice::Vlan(_)))
            .count();

        (lax_net_is_none, stop_is_arp, vlan_count)
    }

    // -------------------------------------------------------------------------
    // Test 1 — QinQ benign truncated: no false-positive D11
    // -------------------------------------------------------------------------

    /// Guards that a QinQ double-tagged, snaplen-truncated, otherwise-benign ARP
    /// frame (valid Ethernet/IPv4 inner ARP fixed header, no variable section) is
    /// classified as `"truncated ARP frame"` and does NOT produce a D11 malformed
    /// finding.
    ///
    /// BC-2.16.015 PC-7b / D-078 VLAN-offset extension to QinQ:
    ///
    /// The lax `None` arm in `decode_packet` computes the ARP offset as
    /// `14 + lax.link_exts.iter().map(|ext| ext.header_len()).sum()`.
    /// For QinQ (outer 0x88A8 + inner 0x8100), etherparse 0.20.2 produces two
    /// separate `LaxLinkExtSlice::Vlan` entries, each with `header_len() == 4`,
    /// summing to 8 → `arp_offset = 22`.  The ARP fixed header at offset 22 has
    /// valid fields (htype=0x0001, ptype=0x0800, hlen=6, plen=4), so the frame
    /// is correctly classified as genuine truncation and no D11 is emitted.
    #[test]
    fn test_BC_2_16_015_qinq_truncated_benign_arp_no_false_positive_d11() {
        // ---- Fixture ----
        // 30-byte frame: 14 Ethernet + 4 outer VLAN (0x88A8, VID=32) +
        // 4 inner VLAN (0x8100, VID=100) + 8 ARP fixed header (benign).
        let frame = make_qinq_benign_truncated_arp();
        assert_eq!(
            frame.len(),
            30,
            "QinQ benign truncated fixture: frame must be 30 bytes"
        );

        // ---- Confirm lax parse properties ----
        // The lax parser must see: net=None, stop_err=Layer::Arp, two Vlan link_exts.
        let (lax_net_is_none, stop_is_arp, vlan_count) = probe_lax_arm_qinq(&frame);

        assert!(
            lax_net_is_none,
            "BC-2.16.015 QinQ: lax.net must be None for a truncated QinQ ARP frame \
             (only 8 bytes of ARP payload, 28 needed). Got Some — fixture or etherparse \
             behavior changed."
        );
        assert!(
            stop_is_arp,
            "BC-2.16.015 QinQ: lax.stop_err must be Layer::Arp for a QinQ truncated ARP \
             frame. Got different stop_err."
        );
        assert_eq!(
            vlan_count, 2,
            "BC-2.16.015 QinQ: lax.link_exts must contain exactly TWO Vlan entries for \
             a QinQ double-tagged frame (outer 0x88A8 + inner 0x8100). \
             etherparse 0.20.2 has no VlanDouble variant — QinQ is represented as two \
             separate Vlan entries. Got {vlan_count} Vlan entries. \
             If etherparse was upgraded and changed its representation, the offset \
             formula in decoder.rs must be re-verified."
        );

        // ---- Verify the offset-formula sum directly ----
        // This is the same computation the decoder performs; we check it here so a
        // failure produces a precise diagnostic rather than an indirect D11 mismatch.
        let lax = LaxSlicedPacket::from_ethernet(&frame).expect("lax parse must succeed");
        let link_exts_sum: usize = lax.link_exts.iter().map(|e| e.header_len()).sum();
        assert_eq!(
            link_exts_sum,
            QINQ_TAGS_LEN,
            "BC-2.16.015 QinQ offset formula: Σ link_exts.header_len() must equal 8 \
             for a QinQ double-tagged frame (2 × 4-byte VLAN tags). \
             Got {link_exts_sum}. Decoder arp_offset would be {} instead of {}.",
            ETH2_LEN + link_exts_sum,
            ARP_OFFSET_QINQ
        );

        // ---- Verify the ARP fixed header is readable at the correct offset ----
        let correct_htype =
            u16::from_be_bytes([frame[ARP_OFFSET_QINQ], frame[ARP_OFFSET_QINQ + 1]]);
        let correct_ptype =
            u16::from_be_bytes([frame[ARP_OFFSET_QINQ + 2], frame[ARP_OFFSET_QINQ + 3]]);
        let correct_hlen = frame[ARP_OFFSET_QINQ + 4];
        let correct_plen = frame[ARP_OFFSET_QINQ + 5];
        assert_eq!(
            correct_htype, 0x0001,
            "QinQ ARP htype at offset {ARP_OFFSET_QINQ} must be 0x0001"
        );
        assert_eq!(
            correct_ptype, 0x0800,
            "QinQ ARP ptype at offset {ARP_OFFSET_QINQ} must be 0x0800"
        );
        assert_eq!(
            correct_hlen, 6,
            "QinQ ARP hlen at offset {ARP_OFFSET_QINQ} must be 6"
        );
        assert_eq!(
            correct_plen, 4,
            "QinQ ARP plen at offset {ARP_OFFSET_QINQ} must be 4"
        );

        // ---- decode_packet must return Err ----
        let decode_result = decode_packet(&frame, DataLink::ETHERNET);
        assert!(
            decode_result.is_err(),
            "BC-2.16.015 QinQ: decode_packet must return Err for a truncated QinQ ARP \
             frame. Got Ok."
        );

        let err_msg = decode_result.unwrap_err().to_string();

        // ---- PRIMARY GUARD: no false-positive D11 ----
        // After the D-078 VLAN-offset fix (which handles QinQ via the generic
        // link_exts sum), decode_packet returns "truncated ARP frame" (not
        // the D11 trigger) for a benign QinQ truncated ARP frame.
        let mut analyzer = ArpAnalyzer::new(3, 50);
        let routed_to_d11 = err_msg.contains("Non-Ethernet/IPv4 ARP frame");
        if routed_to_d11 {
            analyzer.record_malformed(frame.len());
        }

        assert_eq!(
            analyzer.malformed_findings, 0,
            "BC-2.16.015 PC-7b QinQ: a QinQ double-tagged truncated benign ARP frame \
             (htype=0x0001, ptype=0x0800, hlen=6, plen=4 at offset {ARP_OFFSET_QINQ}, \
             no variable section) must NOT produce a D11 malformed finding. \
             Got malformed_findings = {}. \
             decode_packet error: '{err_msg}'. \
             Expected: 'truncated ARP frame' (genuine truncation, no D11). \
             If 'Non-Ethernet/IPv4 ARP frame' was returned, the ARP offset formula \
             in decoder.rs is NOT summing link_exts correctly for QinQ — \
             THIS IS A PRODUCTION BUG.",
            analyzer.malformed_findings
        );

        assert!(
            err_msg.contains("truncated ARP frame"),
            "BC-2.16.015 PC-7b QinQ: decode_packet must return 'truncated ARP frame' \
             for a genuinely-truncated QinQ benign ARP frame. Got: '{err_msg}'. \
             If 'Non-Ethernet/IPv4 ARP frame' was returned, the ARP offset formula \
             is reading the wrong bytes — THIS IS A PRODUCTION BUG."
        );
    }

    // -------------------------------------------------------------------------
    // Test 2 — QinQ malformed hlen=8: routes to D11
    // -------------------------------------------------------------------------

    /// Guards that a QinQ double-tagged, truncated, GENUINELY MALFORMED ARP frame
    /// (`hlen=8`, non-Ethernet hardware-address length) is classified as
    /// `"Non-Ethernet/IPv4 ARP frame"` and produces a D11 malformed finding.
    ///
    /// BC-2.16.009 PC3 / D-078 VLAN-offset extension to QinQ:
    ///
    /// The lax `None` arm reads the ARP fixed header at offset 22 (14 + 8 for two
    /// 4-byte VLAN tags) and finds `hlen=8 ≠ 6`, correctly routing to D11.
    /// Finding quality assertions mirror the single-VLAN malformed case in
    /// bc_2_16_d078_vlan_offset_tests.rs.
    #[test]
    fn test_BC_2_16_009_qinq_malformed_hlen8_routes_to_d11() {
        // ---- Fixture ----
        // 30-byte frame: 14 Ethernet + 4 outer VLAN + 4 inner VLAN +
        // 8 ARP fixed header with hlen=8 (BAD).
        let frame = make_qinq_malformed_arp_hlen8();
        assert_eq!(
            frame.len(),
            30,
            "QinQ malformed fixture: frame must be 30 bytes"
        );

        // ---- Confirm lax parse properties ----
        let (lax_net_is_none, stop_is_arp, vlan_count) = probe_lax_arm_qinq(&frame);

        assert!(
            lax_net_is_none,
            "BC-2.16.009 QinQ: lax.net must be None for a malformed QinQ ARP frame."
        );
        assert!(
            stop_is_arp,
            "BC-2.16.009 QinQ: lax.stop_err must be Layer::Arp for a QinQ ARP frame."
        );
        assert_eq!(
            vlan_count, 2,
            "BC-2.16.009 QinQ: lax.link_exts must contain exactly TWO Vlan entries \
             for a QinQ double-tagged frame. Got {vlan_count}."
        );

        // ---- Verify inner ARP hlen at the correct QinQ offset ----
        let inner_hlen = frame[ARP_OFFSET_QINQ + 4]; // ARP hlen byte
        assert_eq!(
            inner_hlen,
            8,
            "Fixture pre-condition: inner ARP hlen at offset {} must be 8 (BAD). \
             Got {}. Fixture layout may be wrong.",
            ARP_OFFSET_QINQ + 4,
            inner_hlen
        );

        // ---- decode_packet must return Err ----
        let decode_result = decode_packet(&frame, DataLink::ETHERNET);
        assert!(
            decode_result.is_err(),
            "BC-2.16.009 QinQ: decode_packet must return Err for a malformed QinQ ARP \
             frame. Got Ok."
        );

        let err_msg = decode_result.unwrap_err().to_string();

        // ---- Simulate main.rs routing and assert D11 IS emitted ----
        let mut analyzer = ArpAnalyzer::new(3, 50);
        let routed_to_d11 = err_msg.contains("Non-Ethernet/IPv4 ARP frame");
        if routed_to_d11 {
            let findings = analyzer.record_malformed(frame.len());
            assert!(
                !findings.is_empty(),
                "BC-2.16.009 PC3 QinQ: record_malformed must emit at least one D11 finding \
                 for a genuinely malformed QinQ ARP frame (hlen=8 at offset {}). \
                 Got 0 findings.",
                ARP_OFFSET_QINQ + 4
            );
        }

        assert!(
            analyzer.malformed_findings >= 1,
            "BC-2.16.009 PC3 QinQ: a QinQ double-tagged truncated ARP frame with genuinely \
             malformed inner fields (hlen=8 at offset {}) must produce a D11 malformed \
             finding. Got malformed_findings = {}. \
             decode_packet error: '{err_msg}'.",
            ARP_OFFSET_QINQ + 4,
            analyzer.malformed_findings
        );

        // ---- D11 finding quality assertions (parity with d078 malformed-case tests) ----
        if routed_to_d11 {
            let mut analyzer2 = ArpAnalyzer::new(3, 50);
            let d11_findings = analyzer2.record_malformed(frame.len());
            let d11 = d11_findings
                .first()
                .expect("record_malformed must return at least one finding");

            assert_eq!(
                d11.category,
                ThreatCategory::Anomaly,
                "BC-2.16.009 QinQ Inv1: D11 finding must have category Anomaly. \
                 Got {:?}",
                d11.category
            );
            assert!(
                d11.mitre_techniques.is_empty(),
                "BC-2.16.009 QinQ Inv3: D11 finding must have empty mitre_techniques \
                 (T0814 withheld per DF-VALIDATION-001). Got {:?}",
                d11.mitre_techniques
            );
        }
    }

    // -------------------------------------------------------------------------
    // Test 3 — Offset-formula probe: pin etherparse QinQ representation
    // -------------------------------------------------------------------------

    /// Guards that etherparse 0.20.2 represents QinQ (outer 0x88A8 + inner 0x8100)
    /// as two separate `LaxLinkExtSlice::Vlan` entries — NOT a single `VlanDouble`
    /// variant (which does not exist in this version) — and that the per-entry
    /// `header_len()` is 4, summing to 8.  Also confirms the single-VLAN
    /// control case gives `Σ == 4`.
    ///
    /// This test pins the etherparse data-model assumption that the ARP offset
    /// formula in decoder.rs relies on.  If a future etherparse version introduces
    /// a `VlanDouble` variant (with a different `header_len()`) or changes the
    /// per-tag length, this test will fail loudly before any production bug can
    /// appear silently.
    #[test]
    fn test_BC_2_16_015_qinq_link_exts_offset_formula_pin() {
        // ---- QinQ frame: two Vlan entries ----
        let qinq_frame = make_qinq_benign_truncated_arp();
        let lax_qinq =
            LaxSlicedPacket::from_ethernet(&qinq_frame).expect("QinQ lax parse must not fail");

        // Assert etherparse represents QinQ as exactly two Vlan entries.
        assert_eq!(
            lax_qinq.link_exts.len(),
            2,
            "etherparse 0.20.2 QinQ representation: lax.link_exts must have exactly 2 entries \
             (one per VLAN tag). Got {}. If etherparse changed its representation, the offset \
             formula in decoder.rs must be re-verified.",
            lax_qinq.link_exts.len()
        );

        // Assert both entries are Vlan (not any hypothetical VlanDouble).
        for (i, ext) in lax_qinq.link_exts.iter().enumerate() {
            assert!(
                matches!(ext, LaxLinkExtSlice::Vlan(_)),
                "etherparse 0.20.2 QinQ: link_exts[{i}] must be Vlan variant. \
                 Got a different variant. If a VlanDouble or similar was introduced, \
                 decoder.rs header_len() delegation must be re-verified."
            );
        }

        // Assert per-entry header_len == 4 (2-byte TCI + 2-byte inner EtherType).
        for (i, ext) in lax_qinq.link_exts.iter().enumerate() {
            assert_eq!(
                ext.header_len(),
                VLAN_TAG_LEN,
                "etherparse 0.20.2 QinQ: link_exts[{i}].header_len() must be 4. \
                 Got {}. This would invalidate the offset formula sum in decoder.rs.",
                ext.header_len()
            );
        }

        // Assert the sum equals QINQ_TAGS_LEN == 8.
        let qinq_sum: usize = lax_qinq.link_exts.iter().map(|e| e.header_len()).sum();
        assert_eq!(
            qinq_sum,
            QINQ_TAGS_LEN,
            "etherparse 0.20.2 QinQ offset formula: Σ link_exts.header_len() must equal \
             8 for a QinQ double-tagged frame. Got {qinq_sum}. \
             This DIRECTLY affects the ARP offset computation in decoder.rs: \
             arp_offset = 14 + {qinq_sum} = {} instead of the correct {}.",
            ETH2_LEN + qinq_sum,
            ARP_OFFSET_QINQ
        );

        // ---- Single-VLAN control: confirm Σ == 4 ----
        // Build a minimal single-VLAN frame (same as d078 tests) to confirm the
        // control case is unchanged by this test suite.
        let mut single_vlan_frame = Vec::with_capacity(26);
        single_vlan_frame.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        single_vlan_frame.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        single_vlan_frame.extend_from_slice(&[0x81, 0x00]); // EtherType: 802.1Q
        single_vlan_frame.extend_from_slice(&[0x00, 0x64]); // TCI: VID=100
        single_vlan_frame.extend_from_slice(&[0x08, 0x06]); // inner EtherType: ARP
        // ARP fixed header (benign, truncated)
        single_vlan_frame.extend_from_slice(&[0x00, 0x01, 0x08, 0x00, 6, 4, 0x00, 0x01]);

        let lax_single = LaxSlicedPacket::from_ethernet(&single_vlan_frame)
            .expect("single-VLAN lax parse must not fail");
        let single_sum: usize = lax_single.link_exts.iter().map(|e| e.header_len()).sum();

        assert_eq!(
            single_sum, VLAN_TAG_LEN,
            "etherparse 0.20.2 single-VLAN control: Σ link_exts.header_len() must equal \
             4 for a single-tagged frame. Got {single_sum}."
        );

        assert_eq!(
            lax_single.link_exts.len(),
            1,
            "etherparse 0.20.2 single-VLAN control: lax.link_exts must have exactly \
             1 entry. Got {}.",
            lax_single.link_exts.len()
        );
    }

    // -------------------------------------------------------------------------
    // Test 4 — MACsec probe (observe-only shape guard; offset asserted in e17 file)
    // -------------------------------------------------------------------------

    /// MACsec ARP lax parse probe — no-panic and shape regression guard.
    ///
    /// Builds an Ethernet / MACsec(Unmodified, ptype=ARP) / ARP frame using
    /// etherparse's `MacsecHeader` builder with a minimal SecTag (no SCI, ptype
    /// Unmodified(ARP)) and guards:
    ///   1. `LaxSlicedPacket::from_ethernet` does not panic.
    ///   2. `lax.link_exts` contains exactly one `Macsec` entry.
    ///   3. The observed `header_len()` matches `MacsecHeader::header_len()` (8 without SCI).
    ///
    /// ### Offset arithmetic — synthetic assertions live in the e17 file
    ///
    /// The MACsec `header_len()` for an Unmodified payload without SCI is 8:
    ///
    ///   - 6 bytes: SecTag (TCI-AN + SL + PN, 4-byte packet number)
    ///   - 2 bytes: next EtherType (present only for Unmodified payloads)
    ///   - Total: 6 + 2 = 8 bytes (no SCI; with SCI: 6 + 8 + 2 = 16)
    ///
    /// The decoder therefore computes `arp_offset = 14 + 8 = 22` (no SCI) or
    /// `arp_offset = 14 + 16 = 30` (SCI present).  These offset values are
    /// synthetically asserted in `bc_2_16_e17_macsec_offset_tests.rs`:
    ///   - `test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22`
    ///   - `test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30`
    ///
    /// This probe is intentionally observe-only for offset within this file so that
    /// offset-correctness assertions are owned by the e17 file, not duplicated here.
    ///
    /// ### Remaining real-world gap (EC-009 part c)
    ///
    /// What is NOT yet verified is the existence and behavior of MACsec-over-ARP
    /// in REAL on-wire captured traffic (BC-2.16.009/015 EC-009 part c).  That is
    /// a real-world fixture gap, not an offset-formula gap.  Modified/encrypted
    /// MACsec payloads are opaque — no inner ARP is readable — so the decoder's
    /// truncation fallback applies there regardless of offset; `lax.net` would be
    /// None with a MACsec stop_err, not an ARP stop_err.
    ///
    /// If constructing a MACsec frame that etherparse parses as a Macsec
    /// link_ext proves impossible (e.g., etherparse rejects the byte sequence),
    /// the frame-construction failure is itself a diagnostic finding and the test
    /// documents it in the assertion message.
    #[test]
    fn test_BC_2_16_015_macsec_arp_lax_parse_probe() {
        // ---- Build the MACsec header bytes ----
        // Minimal SecTag: no SCI, ptype=Unmodified(ARP), short_len=0 (unknown length),
        // packet_nr=1, an=0, endstation_id=false, scb=false.
        //
        // For Unmodified ptype without SCI:
        //   header_len = 6 (SecTag) + 2 (next EtherType) = 8 bytes.
        // The 2-byte next EtherType field is ARP (0x0806) and is part of the header.
        // After the header, the ARP payload begins immediately.
        let macsec_hdr = MacsecHeader {
            ptype: MacsecPType::Unmodified(EtherType::ARP),
            endstation_id: false,
            scb: false,
            an: MacsecAn::ZERO,
            short_len: MacsecShortLen::ZERO,
            packet_nr: 1,
            sci: None, // no SCI → header is 6 + 2 = 8 bytes
        };
        let macsec_hdr_bytes = macsec_hdr.to_bytes();

        // Computed expected header_len for documentation purposes.
        // With sci=None and ptype=Unmodified: 6 + 0 + 2 = 8.
        let expected_hdr_len = macsec_hdr.header_len();
        assert_eq!(
            expected_hdr_len, 8,
            "MACsec probe pre-condition: MacsecHeader without SCI and Unmodified ptype \
             must have header_len == 8 (6 SecTag + 2 next EtherType). Got {}. \
             etherparse changed its MacsecHeader::header_len() formula — all MACsec \
             offset assumptions in decoder.rs must be re-verified.",
            expected_hdr_len
        );

        // ---- Build the truncated ARP fixed header ----
        // Valid benign fields, no variable section.
        let arp_fixed: [u8; 8] = [
            0x00, 0x01, // htype: Ethernet
            0x08, 0x00, // ptype: IPv4
            6,    // hlen: 6
            4,    // plen: 4
            0x00, 0x01, // oper: Request
        ];

        // ---- Assemble the full frame ----
        // Layout: 14 Ethernet + macsec_hdr (8 bytes) + arp_fixed (8 bytes) = 30 bytes.
        // Note: the outer EtherType in the Ethernet header is 0x88E5 (MACsec).
        // The MACsec header's Unmodified ptype encodes the inner EtherType (ARP=0x0806)
        // as the last 2 bytes of the MACsec header, not as a separate field.
        let mut frame = Vec::new();
        // Ethernet header (14 bytes)
        frame.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]); // dst MAC broadcast
        frame.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]); // src MAC
        frame.extend_from_slice(&[0x88, 0xE5]); // EtherType: 0x88E5 (MACsec / IEEE 802.1AE)
        // MACsec SecTag + next EtherType (8 bytes)
        frame.extend_from_slice(&macsec_hdr_bytes);
        // ARP fixed header — truncated (no variable section)
        frame.extend_from_slice(&arp_fixed);

        let expected_frame_len = ETH2_LEN + expected_hdr_len + arp_fixed.len();
        assert_eq!(
            frame.len(),
            expected_frame_len,
            "MACsec probe: frame must be {expected_frame_len} bytes. Got {}.",
            frame.len()
        );

        // ---- Run lax parse — assert no panic ----
        // SAFETY GUARANTEE ONLY: this test asserts the parser does not panic.
        // It does NOT assert offset correctness or D11 classification.
        let lax = LaxSlicedPacket::from_ethernet(&frame)
            .expect("LaxSlicedPacket::from_ethernet must not panic on any Ethernet-framed input");

        // ---- Observe and record the link_exts shape ----
        let macsec_count = lax
            .link_exts
            .iter()
            .filter(|ext| matches!(ext, LaxLinkExtSlice::Macsec(_)))
            .count();

        let observed_macsec_hdr_len: Option<usize> = lax.link_exts.iter().find_map(|ext| {
            if let LaxLinkExtSlice::Macsec(m) = ext {
                Some(m.header.header_len())
            } else {
                None
            }
        });

        let link_exts_sum: usize = lax.link_exts.iter().map(|e| e.header_len()).sum();

        // Assert the MACsec frame produces exactly one Macsec entry in link_exts.
        // If this assertion fails, document why (e.g., etherparse rejected the
        // MACsec header entirely and set stop_err instead of populating link_exts).
        assert_eq!(
            macsec_count,
            1,
            "MACsec probe: lax.link_exts must contain exactly ONE Macsec entry for \
             a frame with EtherType 0x88E5 and a valid MacsecHeader (Unmodified, no SCI). \
             Got {macsec_count} Macsec entries. \
             lax.stop_err = {:?}. \
             lax.link_exts.len() = {}. \
             This may indicate that etherparse rejected the constructed MACsec frame — \
             check the frame bytes and MacsecHeader serialization.",
            lax.stop_err,
            lax.link_exts.len()
        );

        // Assert the observed header_len matches the expected value (8).
        // This pins the formula: 6 SecTag + 2 next_EtherType (Unmodified, no SCI).
        //
        // This assertion guards the HEADER parse.  The decoder uses
        // `lax.link_exts.header_len()` to compute `arp_offset = 14 + 8 = 22`,
        // which is consistent with the actual ARP position in this synthetic frame.
        // The full end-to-end offset correctness (offset 22 no-SCI / offset 30 SCI)
        // is asserted in bc_2_16_e17_macsec_offset_tests.rs.  What remains an open
        // gap is validation against REAL on-wire MACsec+ARP captures (EC-009 part c).
        let observed = observed_macsec_hdr_len.unwrap_or(0);
        assert_eq!(
            observed, expected_hdr_len,
            "MACsec probe: observed LaxMacsecSlice header_len ({observed}) must equal \
             the pre-computed MacsecHeader::header_len ({expected_hdr_len}). \
             If they differ, LaxMacsecSlice and MacsecHeader disagree on how many \
             bytes the SecTag occupies — this is a critical etherparse invariant \
             violation that would break the decoder offset formula."
        );

        // Record the full observed state for diagnostic purposes.
        // Synthetic offset assertions (22 no-SCI / 30 SCI) are owned by
        // bc_2_16_e17_macsec_offset_tests.rs and confirmed green there.
        //
        //   Observed link_exts_sum  : {link_exts_sum}
        //   Observed macsec hdr_len : {observed}
        //   lax.net                 : {:?}
        //   lax.stop_err            : {:?}
        //
        // Remaining gap: real on-wire MACsec+ARP captures (EC-009 part c) are not
        // yet available.  Encrypted/modified MACsec payloads are always opaque and
        // would never produce an ARP stop_err, so decoder case (c) handles those
        // safely regardless.
        let _ = (link_exts_sum, &lax.net, &lax.stop_err); // suppress unused-variable warnings
    }
}
