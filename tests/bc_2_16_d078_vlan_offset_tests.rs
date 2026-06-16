//! F5 finding F-1 / D-078 VLAN-offset bug: lax-path ARP malformed-vs-truncation
//! classification uses a HARDCODED offset of 14 (Ethernet2 header length) and
//! IGNORES `lax.link_exts` (VLAN tags). For a VLAN-tagged frame the real ARP
//! fixed header starts at offset 18 (14 Ethernet + 4 VLAN), so the peek at offset
//! 14 reads the 4-byte 802.1Q tag region (TCI + inner EtherType bytes) as the
//! ARP `htype`/`ptype` and MISCLASSIFIES the frame.
//!
//! ## What these tests exercise
//!
//! ### Test 1 — PRIMARY RED
//!
//! `test_F1_vlan_tagged_truncated_benign_arp_no_false_positive_d11`
//!
//!   A VLAN-tagged, snaplen-TRUNCATED, otherwise BENIGN Ethernet/802.1Q/ARP frame:
//!   the inner ARP fixed header has valid Ethernet/IPv4 fields (`htype=0x0001,
//!   ptype=0x0800, hlen=6, plen=4`), but no variable section is present.  This is
//!   genuine truncation — the ARP payload was cut short by snaplen before the sender
//!   MAC / IP fields were captured.  There is NO malformed field; the correct outcome
//!   is a generic decode-error ("truncated ARP frame"), NOT a D11 finding.
//!
//!   GUARDS: `malformed_findings == 0` after processing.
//!
//!   FAILS on the current code because the buggy offset-14 peek reads
//!   `data[14..16] = 0x00, 0x64` (the TCI for VID=100) as `htype = 0x0064 ≠ 0x0001`,
//!   detects a "bad field", and returns `"Non-Ethernet/IPv4 ARP frame"` — a spurious
//!   D11 false positive.
//!
//! ### Test 2 — VLAN + malformed (routes to D11)
//!
//! `test_F1_vlan_tagged_truncated_malformed_arp_routes_to_d11`
//!
//!   A VLAN-tagged, truncated, GENUINELY MALFORMED ARP frame: same VLAN framing but
//!   the inner ARP fixed header has `hlen=8` (non-Ethernet, bad).  After the fix, the
//!   peek at the correct offset (18) reads `hlen=8`, correctly identifies a malformed
//!   field, and routes to D11.
//!
//!   GUARDS: `malformed_findings >= 1` after processing.
//!
//! ### Regression guards (must stay GREEN)
//!
//! `test_F1_nonvlan_truncated_benign_unchanged`
//!   Non-VLAN truncated benign ARP (hlen=6, plen=4) with short variable section.
//!   Offset 14 is correct for plain Ethernet. Must continue to produce 0 D11 findings.
//!
//! `test_F1_nonvlan_truncated_malformed_unchanged`
//!   Non-VLAN truncated malformed ARP (hlen=8, plen=4, ARP payload = 8 bytes).
//!   The existing D-078 fix correctly classifies this as malformed. Must continue to
//!   produce >= 1 D11 finding.
//!
//! ## Fixture byte layouts
//!
//! ### VLAN-tagged benign truncated (Test 1) — 26 bytes total
//! ```text
//! [0..6]   dst MAC: FF:FF:FF:FF:FF:FF
//! [6..12]  src MAC: AA:BB:CC:DD:EE:FF
//! [12..14] outer EtherType: 0x8100 (IEEE 802.1Q)
//! [14..16] TCI: 0x0064 (PCP=0, DEI=0, VID=100)
//! [16..18] inner EtherType: 0x0806 (ARP)
//! [18..26] ARP fixed header: htype=0x0001, ptype=0x0800, hlen=6, plen=4, oper=0x0001
//! --- NO variable section ---
//! ```
//! For `hlen=6, plen=4` the ARP spec needs 28 bytes of ARP payload; only 8 are present.
//! The strict parser fails with `SliceError::Len` triggering the lax fallback.
//! The lax parser sets `net = None` with `stop_err = Layer::Arp`.
//! `lax.link_exts` contains one `LaxLinkExtSlice::Vlan(_)` entry (the 802.1Q header).
//!
//! Bug evidence at offset 14:
//!   `htype = u16::from_be([data[14], data[15]]) = u16::from_be([0x00, 0x64]) = 0x0064 ≠ 0x0001`
//!   → current code classifies as malformed → false-positive D11.
//!
//! Correct offset 18:
//!   `htype = 0x0001, ptype = 0x0800, hlen = 6, plen = 4` — all valid.
//!   → correct code classifies as genuine truncation → no D11.
//!
//! ### VLAN-tagged malformed truncated (Test 2) — 26 bytes total
//! ```text
//! [0..6]   dst MAC: FF:FF:FF:FF:FF:FF
//! [6..12]  src MAC: AA:BB:CC:DD:EE:FF
//! [12..14] outer EtherType: 0x8100 (IEEE 802.1Q)
//! [14..16] TCI: 0x0064 (PCP=0, DEI=0, VID=100)
//! [16..18] inner EtherType: 0x0806 (ARP)
//! [18..26] ARP fixed header: htype=0x0001, ptype=0x0800, hlen=8(BAD), plen=4, oper=0x0001
//! --- NO variable section ---
//! ```
//! Inner `hlen=8` is a non-Ethernet hardware-address length — genuinely malformed.
//! At correct offset 18: `hlen=8 ≠ 6` → malformed → D11.
//! At buggy offset 14: `htype=0x0064 ≠ 0x0001` → also "malformed" for wrong reason.
//! (Both offsets trigger D11 for this fixture; correctness matters for Test 1.)
//!
//! Behavioral contracts covered:
//!   BC-2.16.009 PC3   malformed ARP MUST produce D11 regardless of VLAN framing
//!   BC-2.16.015 PC-7b genuine truncation behind VLAN tag MUST NOT produce D11
//!   D-078             O-A finding: lax None arm ignores `link_exts` in offset computation
//!
//! DF-TEST-NAMESPACE-001: all tests wrapped in `mod d078_vlan_offset`.
//! DF-AC-TEST-NAME-SYNC-001: function names embed finding and bug identifiers.
//! DF-GREEN-DOC-TENSE-SWEEP: comments describe what the tests GUARD, written in the
//!   tense of a passing (fixed) test.

#![allow(non_snake_case)]

mod d078_vlan_offset {
    use etherparse::LaxLinkExtSlice;
    use etherparse::LaxSlicedPacket;
    use etherparse::err::Layer;
    use pcap_file::DataLink;
    use wirerust::analyzer::arp::ArpAnalyzer;
    use wirerust::decoder::decode_packet;
    use wirerust::findings::ThreatCategory;

    // -------------------------------------------------------------------------
    // Shared constants
    // -------------------------------------------------------------------------

    /// Ethernet2 header length in bytes (6 dst + 6 src + 2 EtherType).
    const ETH2_LEN: usize = 14;

    /// IEEE 802.1Q single VLAN tag length in bytes (2 TCI + 2 inner EtherType).
    const VLAN_TAG_LEN: usize = 4;

    /// Correct ARP payload offset for a VLAN-tagged Ethernet frame.
    const ARP_OFFSET_WITH_VLAN: usize = ETH2_LEN + VLAN_TAG_LEN; // 18

    /// Buggy ARP payload offset hard-coded in the current lax None arm.
    const ARP_OFFSET_WITHOUT_VLAN: usize = ETH2_LEN; // 14

    // -------------------------------------------------------------------------
    // Fixture builders
    // -------------------------------------------------------------------------

    /// Build a VLAN-tagged Ethernet/ARP frame with BENIGN inner ARP fields
    /// (`htype=0x0001, ptype=0x0800, hlen=6, plen=4`) but NO variable section.
    ///
    /// Wire layout (26 bytes total):
    /// ```text
    /// [0..6]   dst MAC: FF:FF:FF:FF:FF:FF
    /// [6..12]  src MAC: AA:BB:CC:DD:EE:FF
    /// [12..14] outer EtherType: 0x8100 (IEEE 802.1Q)
    /// [14..16] TCI: 0x0064 (PCP=0, DEI=0, VID=100)
    /// [16..18] inner EtherType: 0x0806 (ARP)
    /// [18..26] ARP fixed header: htype=0x0001, ptype=0x0800, hlen=6, plen=4, oper=0x0001
    /// ```
    ///
    /// For `hlen=6, plen=4` the ARP spec requires 28 bytes of ARP payload.
    /// Only the 8-byte fixed header is present — 20 bytes short.
    /// This is genuine snaplen truncation of a valid Ethernet/IPv4 ARP frame.
    ///
    /// Primary RED fixture: VLAN tag causes offset-14 peek to read TCI bytes as
    /// `htype=0x0064`, triggering a false-positive D11 in the current (unfixed) code.
    fn make_vlan_tagged_benign_truncated_arp() -> Vec<u8> {
        let mut frame = Vec::with_capacity(26);
        // Ethernet header (14 bytes)
        frame.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]); // dst MAC broadcast
        frame.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]); // src MAC
        frame.extend_from_slice(&[0x81, 0x00]); // EtherType: 802.1Q VLAN
        // 802.1Q tag (4 bytes)
        frame.extend_from_slice(&[0x00, 0x64]); // TCI: PCP=0, DEI=0, VID=100 (0x0064)
        frame.extend_from_slice(&[0x08, 0x06]); // inner EtherType: ARP
        // ARP fixed header (8 bytes) — BENIGN: hlen=6 (Ethernet), plen=4 (IPv4)
        frame.extend_from_slice(&[0x00, 0x01]); // htype: 0x0001 (Ethernet)
        frame.extend_from_slice(&[0x08, 0x00]); // ptype: 0x0800 (IPv4)
        frame.push(6); // hlen: 6 — correct Ethernet MAC size
        frame.push(4); // plen: 4 — correct IPv4 address size
        frame.extend_from_slice(&[0x00, 0x01]); // oper: ARP Request
        // No variable data section (sender/target hw+proto addrs absent).
        // hlen=6, plen=4 requires 8 + 2*6 + 2*4 = 28 bytes; only 8 present.
        assert_eq!(
            frame.len(),
            26,
            "VLAN benign truncated fixture: total frame must be 26 bytes \
             (14 Ethernet + 4 VLAN tag + 8 ARP fixed header)"
        );
        frame
    }

    /// Build a VLAN-tagged Ethernet/ARP frame with a MALFORMED inner ARP fixed
    /// header (`hlen=8`, non-Ethernet hardware-address length) and NO variable section.
    ///
    /// Wire layout (26 bytes total):
    /// ```text
    /// [0..6]   dst MAC: FF:FF:FF:FF:FF:FF
    /// [6..12]  src MAC: AA:BB:CC:DD:EE:FF
    /// [12..14] outer EtherType: 0x8100 (IEEE 802.1Q)
    /// [14..16] TCI: 0x0064 (PCP=0, DEI=0, VID=100)
    /// [16..18] inner EtherType: 0x0806 (ARP)
    /// [18..26] ARP fixed header: htype=0x0001, ptype=0x0800, hlen=8(BAD), plen=4, oper=0x0001
    /// ```
    ///
    /// `hlen=8` is a non-Ethernet hardware-address length — genuinely malformed.
    /// After the fix, the peek at offset 18 reads `hlen=8 ≠ 6` and correctly routes
    /// to D11.
    fn make_vlan_tagged_malformed_truncated_arp() -> Vec<u8> {
        let mut frame = Vec::with_capacity(26);
        // Ethernet header (14 bytes)
        frame.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]); // dst MAC broadcast
        frame.extend_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]); // src MAC
        frame.extend_from_slice(&[0x81, 0x00]); // EtherType: 802.1Q VLAN
        // 802.1Q tag (4 bytes)
        frame.extend_from_slice(&[0x00, 0x64]); // TCI: PCP=0, DEI=0, VID=100
        frame.extend_from_slice(&[0x08, 0x06]); // inner EtherType: ARP
        // ARP fixed header (8 bytes) — MALFORMED: hlen=8 (non-Ethernet)
        frame.extend_from_slice(&[0x00, 0x01]); // htype: 0x0001 (Ethernet — type is OK)
        frame.extend_from_slice(&[0x08, 0x00]); // ptype: 0x0800 (IPv4 — type is OK)
        frame.push(8); // hlen: 8 — BAD (non-Ethernet hardware-address size)
        frame.push(4); // plen: 4 — correct IPv4 address size
        frame.extend_from_slice(&[0x00, 0x01]); // oper: ARP Request
        // No variable data section — hlen=8, plen=4 requires 8 + 2*8 + 2*4 = 32 bytes ARP.
        assert_eq!(
            frame.len(),
            26,
            "VLAN malformed truncated fixture: total frame must be 26 bytes \
             (14 Ethernet + 4 VLAN tag + 8 ARP fixed header)"
        );
        frame
    }

    /// Build a NON-VLAN Ethernet/ARP frame with BENIGN fields (`hlen=6, plen=4`)
    /// but only 20 bytes of ARP payload (8 bytes short of the 28 needed).
    ///
    /// Regression guard: offset 14 is correct for plain Ethernet. The existing
    /// D-078 fix correctly classifies this as genuine truncation (no D11).
    ///
    /// Mirrors `make_genuine_truncated_arp_hlen6` from bc_2_16_d078_lax_malformed_tests.rs.
    fn make_nonvlan_benign_truncated_arp() -> Vec<u8> {
        let mut frame = Vec::with_capacity(34);
        // Ethernet header (14 bytes)
        frame.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]); // dst MAC broadcast
        frame.extend_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]); // src MAC
        frame.extend_from_slice(&[0x08, 0x06]); // EtherType: ARP
        // ARP fixed header (8 bytes) — hlen=6, plen=4 (valid)
        frame.extend_from_slice(&[0x00, 0x01]); // htype: 0x0001 (Ethernet)
        frame.extend_from_slice(&[0x08, 0x00]); // ptype: 0x0800 (IPv4)
        frame.push(6); // hlen: 6
        frame.push(4); // plen: 4
        frame.extend_from_slice(&[0x00, 0x01]); // oper: Request
        // Partial variable data (12 bytes — 8 short of 20 needed for hlen=6,plen=4)
        frame.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]); // sender hw addr
        frame.extend_from_slice(&[0xc0, 0xa8, 0x01, 0x0a]); // sender proto: 192.168.1.10
        frame.extend_from_slice(&[0x00, 0x00]); // target hw addr (first 2 — truncated)
        assert_eq!(
            frame.len(),
            34,
            "non-VLAN benign truncated fixture: total frame must be 34 bytes"
        );
        frame
    }

    /// Build a NON-VLAN Ethernet/ARP frame with BAD `hlen=8` and only the 8-byte
    /// ARP fixed header present (no variable section).
    ///
    /// Regression guard: the existing D-078 fix correctly classifies this as malformed
    /// at offset 14. Must continue to produce >= 1 D11 finding.
    ///
    /// Mirrors `make_malformed_short_arp_hlen8` from bc_2_16_d078_lax_malformed_tests.rs.
    fn make_nonvlan_malformed_arp_hlen8() -> Vec<u8> {
        let mut frame = Vec::with_capacity(22);
        // Ethernet header (14 bytes)
        frame.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]); // dst MAC broadcast
        frame.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]); // src MAC
        frame.extend_from_slice(&[0x08, 0x06]); // EtherType: ARP
        // ARP fixed header (8 bytes) — hlen=8 (BAD)
        frame.extend_from_slice(&[0x00, 0x01]); // htype: 0x0001
        frame.extend_from_slice(&[0x08, 0x00]); // ptype: 0x0800
        frame.push(8); // hlen: 8 — BAD
        frame.push(4); // plen: 4
        frame.extend_from_slice(&[0x00, 0x01]); // oper: Request
        assert_eq!(
            frame.len(),
            22,
            "non-VLAN malformed (hlen=8) fixture: total frame must be 22 bytes"
        );
        frame
    }

    // -------------------------------------------------------------------------
    // Fixture probe helpers
    //
    // These helpers verify that each fixture reaches the correct lax arm before
    // the D11-path assertions are checked.  They confirm:
    //   1. decode_packet returns Err (not Ok).
    //   2. LaxSlicedPacket has net=None, stop_err=Layer::Arp.
    //   3. For VLAN fixtures: link_exts contains at least one Vlan entry.
    // -------------------------------------------------------------------------

    /// Confirm a frame goes through the lax `None` arm with `stop_err=Layer::Arp`.
    /// Returns `(lax_net_is_none, stop_is_arp, link_exts_has_vlan)`.
    fn probe_lax_arm(frame: &[u8]) -> (bool, bool, bool) {
        let lax = LaxSlicedPacket::from_ethernet(frame)
            .expect("LaxSlicedPacket::from_ethernet must succeed for any Ethernet-framed input");

        let lax_net_is_none = lax.net.is_none();
        let stop_is_arp = lax
            .stop_err
            .as_ref()
            .is_some_and(|(_, layer)| *layer == Layer::Arp);
        let link_exts_has_vlan = lax
            .link_exts
            .iter()
            .any(|ext| matches!(ext, LaxLinkExtSlice::Vlan(_)));

        (lax_net_is_none, stop_is_arp, link_exts_has_vlan)
    }

    // -------------------------------------------------------------------------
    // Test 1 — PRIMARY RED
    // VLAN-tagged, truncated, BENIGN ARP → NO D11 (false-positive guard)
    // -------------------------------------------------------------------------

    /// Guards that a VLAN-tagged, snaplen-truncated, otherwise-benign ARP frame
    /// (valid Ethernet/IPv4 inner ARP fixed header, no variable section) does NOT
    /// produce a D11 malformed finding.
    ///
    /// BC-2.16.015 PC-7b / D-078 VLAN-offset bug:
    ///
    /// The lax `None` arm in `decode_packet` (decoder.rs) computes `arp_offset = 14`
    /// from `lax.link` (Ethernet2 header length) but ignores `lax.link_exts`.  For a
    /// VLAN-tagged frame the ARP payload starts at offset 18 (14 Ethernet + 4 VLAN);
    /// the peek at offset 14 reads the 4-byte 802.1Q TCI region as `htype`/`ptype`.
    /// With `VID=100` (TCI bytes `0x00, 0x64`), `htype = 0x0064 ≠ 0x0001`, so the
    /// current code classifies this as malformed and returns
    /// `"Non-Ethernet/IPv4 ARP frame"` → spurious D11 false positive.
    ///
    /// After the fix, the lax `None` arm adds the total `link_exts` byte-length
    /// (4 bytes for each single-VLAN tag) to `arp_offset`, so the peek is at offset 18
    /// where `htype=0x0001, ptype=0x0800, hlen=6, plen=4` — all valid — and the frame
    /// is correctly classified as genuine truncation (`"truncated ARP frame"`, no D11).
    ///
    /// FAILS on the current (unfixed) code: `malformed_findings == 1` (false positive).
    #[test]
    fn test_F1_vlan_tagged_truncated_benign_arp_no_false_positive_d11() {
        // ---- Fixture ----
        // 26-byte frame: 14 Ethernet + 4 VLAN tag (TCI=0x0064, VID=100) + 8 ARP fixed
        // header (htype=0x0001, ptype=0x0800, hlen=6, plen=4, oper=0x0001).
        // No variable section → lax parser sets net=None, stop_err=Layer::Arp.
        let frame = make_vlan_tagged_benign_truncated_arp();
        assert_eq!(
            frame.len(),
            26,
            "VLAN benign truncated fixture: frame must be 26 bytes \
             (14 Ethernet + 4 VLAN tag + 8 ARP fixed header)"
        );

        // ---- Confirm the fixture reaches the lax None arm with link_exts=VLAN ----
        // The probe confirms:
        //   • lax.net == None (the lax parser cannot build an ArpPacketSlice due to
        //     the short ARP payload — only 8 bytes of ARP present, 28 needed)
        //   • lax.stop_err == (_, Layer::Arp) (failure is at the ARP layer)
        //   • lax.link_exts contains a Vlan entry (the 802.1Q tag is parsed)
        let (lax_net_is_none, stop_is_arp, link_exts_has_vlan) = probe_lax_arm(&frame);

        assert!(
            lax_net_is_none,
            "D-078 VLAN-offset / BC-2.16.015: lax.net must be None for a truncated ARP \
             frame (only 8 bytes of ARP payload, 28 needed for hlen=6,plen=4). \
             Got Some — fixture or etherparse behavior changed."
        );
        assert!(
            stop_is_arp,
            "D-078 VLAN-offset / BC-2.16.015: lax.stop_err must be Layer::Arp for a \
             VLAN-tagged truncated ARP frame. Got different stop_err — \
             the frame is not reaching the correct lax arm."
        );
        assert!(
            link_exts_has_vlan,
            "D-078 VLAN-offset / BC-2.16.015: lax.link_exts must contain a Vlan entry \
             for a VLAN-tagged frame (EtherType 0x8100). Got no Vlan in link_exts. \
             The probe confirms the 802.1Q tag is NOT being parsed — frame may be wrong."
        );

        // ---- Verify offset mismatch that causes the bug ----
        // At the buggy offset (14): reads the TCI bytes as ARP htype.
        // TCI for VID=100: [0x00, 0x64] → htype = 0x0064 ≠ 0x0001.
        // At the correct offset (18): reads the real ARP htype.
        // Real ARP htype: [0x00, 0x01] → htype = 0x0001 = 0x0001 ✓.
        let buggy_htype = u16::from_be_bytes([
            frame[ARP_OFFSET_WITHOUT_VLAN],
            frame[ARP_OFFSET_WITHOUT_VLAN + 1],
        ]);
        let correct_htype =
            u16::from_be_bytes([frame[ARP_OFFSET_WITH_VLAN], frame[ARP_OFFSET_WITH_VLAN + 1]]);

        assert_ne!(
            buggy_htype, 0x0001,
            "Offset-14 htype must NOT equal 0x0001 for a VLAN-tagged frame — \
             the TCI bytes at [14..16] encode VID=100 (0x0064), not a valid ARP htype. \
             Got 0x{buggy_htype:04x}. Fixture layout may be wrong."
        );
        assert_eq!(
            correct_htype, 0x0001,
            "Offset-18 htype must equal 0x0001 (Ethernet ARP) for a benign VLAN-tagged \
             ARP frame. Got 0x{correct_htype:04x}. Fixture layout may be wrong."
        );

        // ---- decode_packet must return Err (either path) ----
        let decode_result = decode_packet(&frame, DataLink::ETHERNET);
        assert!(
            decode_result.is_err(),
            "D-078 VLAN-offset: decode_packet must return Err for a truncated VLAN-tagged \
             ARP frame (only 8 of 28 required ARP bytes present). Got Ok — \
             etherparse or decoder behavior changed."
        );

        let err_msg = decode_result.unwrap_err().to_string();

        // Confirm the error is ARP-related (either truncation or the wrong D11 trigger).
        let is_arp_error =
            err_msg.contains("truncated ARP frame") || err_msg.contains("Non-Ethernet/IPv4 ARP");
        assert!(
            is_arp_error,
            "D-078 VLAN-offset / BC-2.16.015: decode_packet Err must be an ARP-related \
             error string. Got: '{err_msg}'. Fixture may have hit an unexpected code path."
        );

        // ---- Simulate main.rs routing and assert NO D11 is emitted ----
        // After the fix: decode_packet returns "truncated ARP frame" (not the D11 trigger).
        // main.rs does NOT call record_malformed → malformed_findings stays 0.
        let mut analyzer = ArpAnalyzer::new(3, 50);
        let routed_to_d11 = err_msg.contains("Non-Ethernet/IPv4 ARP frame");
        if routed_to_d11 {
            analyzer.record_malformed(frame.len());
        }

        // ---- PRIMARY RED ASSERTION ----
        // Guards that a benign VLAN-tagged truncated ARP does NOT produce a D11 finding.
        //
        // FAILS on the current (unfixed) code:
        //   • Current code: arp_offset = 14 (ignores VLAN tag)
        //   • data[14..16] = TCI bytes [0x00, 0x64] → htype = 0x0064 ≠ 0x0001
        //   • Misclassified as "Non-Ethernet/IPv4 ARP frame" → routed_to_d11 = true
        //   • record_malformed called → malformed_findings = 1
        //   • ASSERTION FAILS: malformed_findings == 0 is violated
        //
        // PASSES after the fix:
        //   • Fixed code: arp_offset = 14 + link_exts_byte_len (VLAN) = 14 + 4 = 18
        //   • data[18..26] = real ARP fixed header → htype=0x0001, ptype=0x0800,
        //     hlen=6, plen=4 — all valid
        //   • Classified as "truncated ARP frame" → routed_to_d11 = false
        //   • record_malformed NOT called → malformed_findings = 0
        //   • ASSERTION PASSES
        assert_eq!(
            analyzer.malformed_findings,
            0,
            "D-078 VLAN-offset / BC-2.16.015 PC-7b: a VLAN-tagged truncated benign ARP \
             frame (htype=0x0001, ptype=0x0800, hlen=6, plen=4 at offset 18, no variable \
             section) must NOT produce a D11 malformed finding. \
             Got malformed_findings = {}. \
             \nCurrent decode_packet error: '{err_msg}'. \
             \nBug: arp_offset = {} (hardcoded ETH2 length) ignores lax.link_exts. \
             At offset {}: htype = 0x{:04x} (TCI bytes misread as ARP htype) ≠ 0x0001 \
             → false-positive D11. \
             Fix: include link_exts byte-length in offset → arp_offset = {} \
             → htype = 0x{:04x} ✓ → genuine truncation → no D11.",
            analyzer.malformed_findings,
            ARP_OFFSET_WITHOUT_VLAN,
            ARP_OFFSET_WITHOUT_VLAN,
            buggy_htype,
            ARP_OFFSET_WITH_VLAN,
            correct_htype,
        );
    }

    // -------------------------------------------------------------------------
    // Test 2 — VLAN + malformed (routes to D11 correctly)
    // -------------------------------------------------------------------------

    /// Guards that a VLAN-tagged, truncated, GENUINELY MALFORMED ARP frame
    /// (`hlen=8`, non-Ethernet hardware-address length) is routed to D11.
    ///
    /// BC-2.16.009 PC3 / D-078 VLAN-offset:
    ///
    /// After the fix, the lax `None` arm peeks the real ARP fixed header at
    /// offset 18 (accounting for the 4-byte VLAN tag).  For this fixture,
    /// `hlen=8 ≠ 6` is correctly identified at offset 18 and the frame routes
    /// to `"Non-Ethernet/IPv4 ARP frame"` → D11.
    ///
    /// Note on current behavior: the current buggy code also routes this fixture
    /// to D11, but for the wrong reason (reads TCI bytes as `htype=0x0064 ≠ 0x0001`).
    /// The test asserts the correct post-fix outcome (D11 is emitted). The important
    /// distinction is Test 1, where the correct vs. buggy offsets diverge.
    #[test]
    fn test_F1_vlan_tagged_truncated_malformed_arp_routes_to_d11() {
        // ---- Fixture ----
        // 26-byte frame: 14 Ethernet + 4 VLAN tag (TCI=0x0064, VID=100) + 8 ARP fixed
        // header (htype=0x0001, ptype=0x0800, hlen=8(BAD), plen=4, oper=0x0001).
        // No variable section → lax parser sets net=None, stop_err=Layer::Arp.
        let frame = make_vlan_tagged_malformed_truncated_arp();
        assert_eq!(
            frame.len(),
            26,
            "VLAN malformed truncated fixture: frame must be 26 bytes \
             (14 Ethernet + 4 VLAN tag + 8 ARP fixed header with hlen=8)"
        );

        // ---- Confirm the fixture reaches the lax None arm with link_exts=VLAN ----
        let (lax_net_is_none, stop_is_arp, link_exts_has_vlan) = probe_lax_arm(&frame);

        assert!(
            lax_net_is_none,
            "D-078 VLAN-offset / BC-2.16.009: lax.net must be None for a truncated \
             malformed VLAN-tagged ARP frame. Got Some — fixture error."
        );
        assert!(
            stop_is_arp,
            "D-078 VLAN-offset / BC-2.16.009: lax.stop_err must be Layer::Arp. \
             Got different stop_err — frame is not reaching the lax None arm."
        );
        assert!(
            link_exts_has_vlan,
            "D-078 VLAN-offset / BC-2.16.009: lax.link_exts must contain a Vlan entry \
             for a VLAN-tagged frame. Got no Vlan — fixture may be wrong."
        );

        // ---- Verify the inner ARP field at the correct offset ----
        // At offset 18 (correct): hlen byte is at data[22] (18+4), value = 8.
        let inner_hlen = frame[ARP_OFFSET_WITH_VLAN + 4]; // ARP hlen byte
        assert_eq!(
            inner_hlen,
            8,
            "Fixture pre-condition: inner ARP hlen at offset {} must be 8 (BAD). \
             Got {}. Fixture layout may be wrong.",
            ARP_OFFSET_WITH_VLAN + 4,
            inner_hlen
        );

        // ---- decode_packet must return Err ----
        let decode_result = decode_packet(&frame, DataLink::ETHERNET);
        assert!(
            decode_result.is_err(),
            "D-078 VLAN-offset / BC-2.16.009: decode_packet must return Err for a \
             malformed VLAN-tagged ARP frame. Got Ok."
        );

        let err_msg = decode_result.unwrap_err().to_string();
        let is_arp_error =
            err_msg.contains("truncated ARP frame") || err_msg.contains("Non-Ethernet/IPv4 ARP");
        assert!(
            is_arp_error,
            "D-078 VLAN-offset / BC-2.16.009: decode_packet Err must be an ARP-related \
             error string. Got: '{err_msg}'."
        );

        // ---- Simulate main.rs routing and assert D11 IS emitted ----
        // After the fix: decode_packet returns "Non-Ethernet/IPv4 ARP frame"
        // (hlen=8 detected at offset 18) → routed_to_d11 = true → record_malformed called.
        let mut analyzer = ArpAnalyzer::new(3, 50);
        let routed_to_d11 = err_msg.contains("Non-Ethernet/IPv4 ARP frame");
        if routed_to_d11 {
            let findings = analyzer.record_malformed(frame.len());
            assert!(
                !findings.is_empty(),
                "D-078 VLAN-offset / BC-2.16.009 PC3: record_malformed must emit at \
                 least one D11 finding for a genuinely malformed VLAN-tagged ARP frame \
                 (hlen=8 at offset 18). Got 0 findings."
            );
        }

        // ---- Guard: D11 must be emitted (malformed_findings >= 1) ----
        assert!(
            analyzer.malformed_findings >= 1,
            "D-078 VLAN-offset / BC-2.16.009 PC3: a VLAN-tagged truncated ARP frame with \
             genuinely malformed inner fields (hlen=8 at offset {}, vs. correct field at \
             offset {}) must produce a D11 malformed finding. \
             Got malformed_findings = {}. \
             \nCurrent decode_packet error: '{err_msg}'. \
             \nAfter the fix, the lax None arm reads hlen=8 at the correct offset ({}) \
             and routes to record_malformed → D11.",
            ARP_OFFSET_WITH_VLAN + 4,
            ARP_OFFSET_WITH_VLAN,
            analyzer.malformed_findings,
            ARP_OFFSET_WITH_VLAN,
        );

        // ---- D11 finding quality assertions ----
        if routed_to_d11 {
            let mut analyzer2 = ArpAnalyzer::new(3, 50);
            let d11_findings = analyzer2.record_malformed(frame.len());
            let d11 = d11_findings
                .first()
                .expect("record_malformed must return at least one finding");

            assert_eq!(
                d11.category,
                ThreatCategory::Anomaly,
                "D-078 VLAN-offset / BC-2.16.009 Inv1: D11 finding must have category \
                 Anomaly. Got {:?}",
                d11.category
            );
            assert!(
                d11.mitre_techniques.is_empty(),
                "D-078 VLAN-offset / BC-2.16.009 Inv3: D11 finding must have empty \
                 mitre_techniques (T0814 withheld per DF-VALIDATION-001). Got {:?}",
                d11.mitre_techniques
            );
        }
    }

    // -------------------------------------------------------------------------
    // Regression guard 1 — non-VLAN benign truncated ARP (must stay GREEN)
    // -------------------------------------------------------------------------

    /// Guards that a NON-VLAN truncated benign ARP frame continues to produce
    /// 0 D11 findings after the VLAN-offset fix is applied.
    ///
    /// BC-2.16.015 regression guard / D-078 VLAN-offset:
    ///
    /// For plain Ethernet frames, `arp_offset = 14` is correct.  The fix adds
    /// the `link_exts` byte-length to the offset; for a non-VLAN frame
    /// `link_exts` is empty and the offset stays at 14.  This test guards that
    /// the fix does not regress the non-VLAN path.
    ///
    /// GREEN on current code; must remain GREEN after the fix.
    #[test]
    fn test_F1_nonvlan_truncated_benign_unchanged() {
        // ---- Fixture ----
        // 34-byte non-VLAN frame: hlen=6, plen=4, ARP payload = 20 of 28 bytes.
        let frame = make_nonvlan_benign_truncated_arp();
        assert_eq!(
            frame.len(),
            34,
            "non-VLAN benign truncated fixture: 34 bytes"
        );

        // ---- Confirm no VLAN in lax link_exts ----
        let (lax_net_is_none, stop_is_arp, link_exts_has_vlan) = probe_lax_arm(&frame);
        assert!(lax_net_is_none, "non-VLAN benign: lax.net must be None");
        assert!(stop_is_arp, "non-VLAN benign: stop_err must be Layer::Arp");
        assert!(
            !link_exts_has_vlan,
            "non-VLAN benign: lax.link_exts must NOT contain a Vlan entry \
             for a plain Ethernet frame."
        );

        // ---- decode_packet must return Err("truncated ARP frame") ----
        let decode_result = decode_packet(&frame, DataLink::ETHERNET);
        assert!(
            decode_result.is_err(),
            "non-VLAN benign: decode_packet must return Err. Got Ok."
        );
        let err_msg = decode_result.unwrap_err().to_string();
        assert!(
            err_msg.contains("truncated ARP frame"),
            "non-VLAN benign / BC-2.16.015: decode_packet must return \
             'truncated ARP frame' for a genuinely-truncated plain Ethernet ARP \
             frame with valid fields. Got: '{err_msg}'."
        );
        assert!(
            !err_msg.contains("Non-Ethernet/IPv4 ARP frame"),
            "non-VLAN benign regression: genuine truncation (hlen=6, plen=4) must NOT \
             produce 'Non-Ethernet/IPv4 ARP frame'. Got: '{err_msg}'."
        );

        // ---- No D11 emitted ----
        let mut analyzer = ArpAnalyzer::new(3, 50);
        if err_msg.contains("Non-Ethernet/IPv4 ARP frame") {
            analyzer.record_malformed(frame.len());
        }
        assert_eq!(
            analyzer.malformed_findings, 0,
            "non-VLAN benign regression / BC-2.16.015: malformed_findings must be 0 \
             for a genuinely-truncated plain Ethernet ARP frame. Got {}.",
            analyzer.malformed_findings
        );
    }

    // -------------------------------------------------------------------------
    // Regression guard 2 — non-VLAN malformed ARP, hlen=8 (must stay GREEN)
    // -------------------------------------------------------------------------

    /// Guards that a NON-VLAN malformed ARP frame (`hlen=8`, fixed header only)
    /// continues to produce >= 1 D11 findings after the VLAN-offset fix.
    ///
    /// BC-2.16.009 PC3 regression guard / D-078:
    ///
    /// The existing D-078 fix correctly classifies this as malformed at offset 14.
    /// The VLAN-offset fix must not regress this: for a non-VLAN frame `link_exts`
    /// is empty, so `arp_offset` stays at 14 and the behavior is unchanged.
    ///
    /// GREEN on current code; must remain GREEN after the fix.
    #[test]
    fn test_F1_nonvlan_truncated_malformed_unchanged() {
        // ---- Fixture ----
        // 22-byte non-VLAN frame: hlen=8 (BAD), ARP payload = 8 bytes (fixed header only).
        let frame = make_nonvlan_malformed_arp_hlen8();
        assert_eq!(
            frame.len(),
            22,
            "non-VLAN malformed (hlen=8) fixture: 22 bytes"
        );

        // ---- Confirm no VLAN in lax link_exts ----
        let (lax_net_is_none, stop_is_arp, link_exts_has_vlan) = probe_lax_arm(&frame);
        assert!(lax_net_is_none, "non-VLAN malformed: lax.net must be None");
        assert!(
            stop_is_arp,
            "non-VLAN malformed: stop_err must be Layer::Arp"
        );
        assert!(
            !link_exts_has_vlan,
            "non-VLAN malformed: lax.link_exts must NOT contain a Vlan entry."
        );

        // ---- decode_packet must return Err("Non-Ethernet/IPv4 ARP frame") ----
        let decode_result = decode_packet(&frame, DataLink::ETHERNET);
        assert!(
            decode_result.is_err(),
            "non-VLAN malformed: decode_packet must return Err. Got Ok."
        );
        let err_msg = decode_result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Non-Ethernet/IPv4 ARP frame"),
            "non-VLAN malformed regression / BC-2.16.009 PC3: a non-VLAN malformed ARP \
             frame (hlen=8, ARP payload = 8 bytes) must produce \
             'Non-Ethernet/IPv4 ARP frame' (D11 trigger). Got: '{err_msg}'."
        );

        // ---- D11 is emitted ----
        let mut analyzer = ArpAnalyzer::new(3, 50);
        if err_msg.contains("Non-Ethernet/IPv4 ARP frame") {
            analyzer.record_malformed(frame.len());
        }
        assert!(
            analyzer.malformed_findings >= 1,
            "non-VLAN malformed regression / BC-2.16.009 PC3: malformed_findings must \
             be >= 1 for a non-VLAN malformed ARP frame (hlen=8). Got {}.",
            analyzer.malformed_findings
        );
    }
}
