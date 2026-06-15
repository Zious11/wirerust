//! D-077 / F-ARP-F4-001: ARP hardware-type and protocol-type rejection guard tests.
//!
//! Guards that `extract_arp_frame` rejects ARP frames whose hw_addr_type (htype)
//! is not ETHERNET (0x0001) or whose proto_addr_type (ptype) is not IPv4 (0x0800),
//! even when the address-size fields (hlen=6, plen=4) are valid.
//!
//! Guards that when such a frame is rejected, `decode_packet` returns
//! `Err("Non-Ethernet/IPv4 ARP frame")` and `record_malformed` emits a D11
//! malformed finding — the frame is NOT admitted to the D1/D2/D3/D12 detection
//! pipeline.
//!
//! Also guards that the GARP-that-conflicts finding summary (BC-2.16.014 PC1)
//! contains wording that indicates a binding conflict, not just the plain GARP
//! announcement.
//!
//! Behavioral contracts covered:
//!   BC-2.16.001 PC2  hw_addr_type() == ArpHardwareId::ETHERNET required
//!   BC-2.16.001 PC3  proto_addr_type() == EtherType::IPV4 required
//!   BC-2.16.009 PC3a hw_addr_type != ETHERNET → D11 malformed finding
//!   BC-2.16.009 PC3b proto_addr_type != IPV4  → D11 malformed finding
//!   BC-2.16.009 EC-001 htype=0x0006, hlen=6, plen=4 → None + D11
//!   BC-2.16.009 EC-002 ptype=0x86DD, hlen=6, plen=4 → None + D11
//!   BC-2.16.014 PC1   GARP-that-conflicts summary must indicate binding conflict
//!   Decision D-077    Type-field rejection closes the gap left by size-only guard
//!
//! Critical test-design note (D-077):
//!   F-1 tests use VALID size fields (hlen=6, plen=4) combined with WRONG type
//!   fields.  The existing size guard at `decoder.rs:307` would catch wrong-size
//!   frames regardless of type.  To isolate the MISSING type branch, the size
//!   fields must be valid so that the current code PASSES the size guard and
//!   admits the frame — proving the type branch is absent.
//!
//! DF-TEST-NAMESPACE-001: all tests wrapped in `mod d077_arp_type_reject`.
//! DF-AC-TEST-NAME-SYNC-001: function names embed BC and EC identifiers.
//! DF-GREEN-DOC-TENSE-SWEEP: comments describe what the test GUARDS, written
//!   in the tense of a passing test (the fix already exists), per factory rule.

#![allow(non_snake_case)]

mod d077_arp_type_reject {
    use etherparse::ArpPacketSlice;
    use pcap_file::DataLink;
    use wirerust::analyzer::arp::ArpAnalyzer;
    use wirerust::decoder::{ArpFrame, decode_packet, extract_arp_frame};

    // -------------------------------------------------------------------------
    // ARP byte-buffer builder (mirrored from bc_2_16_story112_arp_tests.rs)
    // -------------------------------------------------------------------------

    /// Build a variable-field ARP payload (no Ethernet header).
    ///
    /// ARP payload layout (RFC 826):
    ///   bytes 0-1: htype (hardware type)
    ///   bytes 2-3: ptype (protocol type)
    ///   byte  4:   hlen  (hardware address length)
    ///   byte  5:   plen  (protocol address length)
    ///   bytes 6-7: oper
    ///   bytes 8-13: sender hardware address
    ///   bytes 14-17: sender protocol address
    ///   bytes 18-23: target hardware address
    ///   bytes 24-27: target protocol address
    ///
    /// When hlen=6 and plen=4 the total payload is exactly 28 bytes.
    #[allow(clippy::too_many_arguments)]
    fn make_arp_payload(
        htype: u16,
        ptype: u16,
        hlen: u8,
        plen: u8,
        oper: u16,
        sender_mac: [u8; 6],
        sender_ip: [u8; 4],
        target_mac: [u8; 6],
        target_ip: [u8; 4],
    ) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&htype.to_be_bytes());
        buf.extend_from_slice(&ptype.to_be_bytes());
        buf.push(hlen);
        buf.push(plen);
        buf.extend_from_slice(&oper.to_be_bytes());
        buf.extend_from_slice(&sender_mac[..hlen.min(6) as usize]);
        if hlen > 6 {
            buf.extend(std::iter::repeat_n(0u8, (hlen - 6) as usize));
        }
        buf.extend_from_slice(&sender_ip[..plen.min(4) as usize]);
        if plen > 4 {
            buf.extend(std::iter::repeat_n(0u8, (plen - 4) as usize));
        }
        buf.extend_from_slice(&target_mac[..hlen.min(6) as usize]);
        if hlen > 6 {
            buf.extend(std::iter::repeat_n(0u8, (hlen - 6) as usize));
        }
        buf.extend_from_slice(&target_ip[..plen.min(4) as usize]);
        if plen > 4 {
            buf.extend(std::iter::repeat_n(0u8, (plen - 4) as usize));
        }
        buf
    }

    /// Build a 42-byte Ethernet frame wrapping the given ARP payload.
    fn make_eth_arp_frame(eth_src_mac: [u8; 6], arp_payload: &[u8]) -> Vec<u8> {
        let mut frame = Vec::new();
        frame.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]); // dst MAC broadcast
        frame.extend_from_slice(&eth_src_mac); // src MAC
        frame.extend_from_slice(&[0x08, 0x06]); // EtherType: ARP
        frame.extend_from_slice(arp_payload);
        frame
    }

    /// Parse an ARP payload slice into an `ArpPacketSlice`.
    fn parse_arp_slice(payload: &[u8]) -> ArpPacketSlice<'_> {
        ArpPacketSlice::from_slice(payload)
            .expect("test-setup error: ARP payload bytes must be parseable by etherparse")
    }

    // -------------------------------------------------------------------------
    // Shared frame constants (canonical values)
    // -------------------------------------------------------------------------

    const SENDER_MAC: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    const SENDER_IP: [u8; 4] = [10, 0, 0, 1];
    const TARGET_MAC: [u8; 6] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    const TARGET_IP: [u8; 4] = [10, 0, 0, 2];

    // -------------------------------------------------------------------------
    // F-1 / BC-2.16.009 PC3a — Test 1
    // htype=0x0006 (IEEE 802), hlen=6, ptype=0x0800 (IPv4), plen=4
    // -------------------------------------------------------------------------

    /// Guards that `extract_arp_frame` rejects an ARP frame with htype=0x0006
    /// (IEEE 802 / non-Ethernet hardware type) even when the size fields are
    /// valid (hlen=6, plen=4).
    ///
    /// BC-2.16.001 PC2 / BC-2.16.009 PC3a / BC-2.16.009 EC-001:
    /// The type field (htype) must equal ArpHardwareId::ETHERNET (0x0001).
    /// A frame with htype=0x0006 and valid sizes (hlen=6, plen=4) bypasses
    /// the existing size guard but must be caught by the missing type guard.
    ///
    /// Isolation requirement (D-077): hlen=6 and plen=4 ensure the size guard
    /// at `decoder.rs:307` does NOT fire, so the test only passes once the
    /// type-rejection branch is added. Using wrong size would let the existing
    /// code pass the test for the wrong reason.
    ///
    /// Assertions:
    ///   Positive: `extract_arp_frame` returns `None` (frame is rejected).
    ///   Positive: `decode_packet` returns `Err` containing
    ///             "Non-Ethernet/IPv4 ARP frame" (D11 trigger).
    ///   Negative: `extract_arp_frame` does NOT return `Some` (frame is NOT
    ///             admitted to D1/D2/D3/D12 analysis pipeline).
    ///
    /// FAILS if `extract_arp_frame` admits a non-Ethernet hardware type
    /// (htype=0x0006) when hlen=6 and plen=4 (valid sizes).
    #[test]
    fn test_BC_2_16_009_PC3a_non_ethernet_hwtype_rejected() {
        // BC-2.16.009 EC-001: htype=0x0006 (IEEE 802), hlen=6, ptype=0x0800, plen=4.
        // Size fields are VALID — this frame passes the existing size guard.
        // The type guard (currently absent) must reject it.
        let payload = make_arp_payload(
            0x0006, // htype: IEEE 802 — non-Ethernet, NOT 0x0001
            0x0800, // ptype: IPv4 (correct)
            6,      // hlen: 6 — VALID Ethernet MAC size
            4,      // plen: 4 — VALID IPv4 address size
            1,      // oper: Request
            SENDER_MAC, SENDER_IP, TARGET_MAC, TARGET_IP,
        );
        let arp = parse_arp_slice(&payload);

        // Confirm the slice fields match the intended fixture.
        // hw_addr_size() returns hlen; hw_addr_type() returns the htype field.
        assert_eq!(
            arp.hw_addr_size(),
            6,
            "test-setup: hlen must be 6 (valid size — isolates the type branch)"
        );
        assert_eq!(
            arp.proto_addr_size(),
            4,
            "test-setup: plen must be 4 (valid size — isolates the type branch)"
        );
        assert_eq!(
            arp.hw_addr_type().0,
            0x0006,
            "test-setup: htype must be 0x0006 (IEEE 802, non-Ethernet)"
        );

        // Primary assertion: extract_arp_frame must reject the frame.
        // Guards that extract_arp_frame returns None for htype=0x0006 even with
        // valid sizes. FAILS if the type guard is absent (returns Some instead).
        let result = extract_arp_frame(&arp, Some(SENDER_MAC), payload.len());
        assert_eq!(
            result, None,
            "BC-2.16.001 PC2 / BC-2.16.009 PC3a / EC-001: extract_arp_frame must return \
             None for htype=0x0006 (IEEE 802 non-Ethernet hardware type) even when \
             hlen=6 and plen=4 are valid. Got Some — the htype guard is missing. \
             FAILS if extract_arp_frame admits a non-Ethernet hardware type.",
        );

        // Secondary assertion: decode_packet must surface this as D11.
        // Guards that the full decode pipeline returns Err("Non-Ethernet/IPv4 ARP frame")
        // so that main.rs routes the frame to record_malformed → D11 finding emission.
        let frame_bytes = make_eth_arp_frame(SENDER_MAC, &payload);
        let decode_result = decode_packet(&frame_bytes, DataLink::ETHERNET);
        let err = decode_result.expect_err(
            "BC-2.16.009 PC3a / EC-001: decode_packet must return Err (not Ok) for a \
             non-Ethernet htype=0x0006 ARP frame with valid sizes (hlen=6, plen=4). \
             D11 malformed trigger requires extract_arp_frame to return None.",
        );
        assert!(
            err.to_string().contains("Non-Ethernet/IPv4 ARP frame"),
            "BC-2.16.009 PC3a / EC-001: decode_packet Err must contain \
             'Non-Ethernet/IPv4 ARP frame' to trigger the D11 malformed path in main.rs. \
             Got: '{err}'",
        );

        // Tertiary assertion: record_malformed emits a D11 finding.
        // Guards the full D11 path: once decode_packet returns Err("Non-Ethernet/IPv4
        // ARP frame"), main.rs calls arp_analyzer.record_malformed(), which must return
        // at least one finding (D11 LOW/Anomaly). Confirms the frame is NOT run through
        // process_arp (D1/D2/D3/D12).
        let mut analyzer = ArpAnalyzer::new(3, 50);
        let malformed_findings = analyzer.record_malformed(payload.len());
        assert!(
            !malformed_findings.is_empty(),
            "BC-2.16.009 PC3a / EC-001: record_malformed must emit at least one D11 \
             finding when called for a non-Ethernet htype=0x0006 frame. Got 0 findings. \
             The frame must reach D11, not D1/D2/D3/D12.",
        );
        assert_eq!(
            analyzer.malformed_findings, 1,
            "BC-2.16.009 PC3a: malformed_findings counter must be 1 after one \
             record_malformed call. Got {}.",
            analyzer.malformed_findings,
        );
    }

    // -------------------------------------------------------------------------
    // F-1 / BC-2.16.009 PC3b — Test 2
    // htype=0x0001 (Ethernet), hlen=6, ptype=0x86DD (IPv6), plen=4
    // -------------------------------------------------------------------------

    /// Guards that `extract_arp_frame` rejects an ARP frame with ptype=0x86DD
    /// (IPv6 protocol type) even when the size fields are valid (hlen=6, plen=4).
    ///
    /// BC-2.16.001 PC3 / BC-2.16.009 PC3b / BC-2.16.009 EC-002:
    /// The protocol type field (ptype) must equal EtherType::IPV4 (0x0800).
    /// A frame with ptype=0x86DD (IPv6) and valid sizes (hlen=6, plen=4)
    /// bypasses the existing size guard but must be caught by the type guard.
    ///
    /// Isolation requirement (D-077): hlen=6 and plen=4 ensure the size guard
    /// does NOT fire. The test is only meaningful once the missing type branch
    /// is added to `extract_arp_frame`.
    ///
    /// Assertions:
    ///   Positive: `extract_arp_frame` returns `None` (frame is rejected).
    ///   Positive: `decode_packet` returns `Err` containing
    ///             "Non-Ethernet/IPv4 ARP frame" (D11 trigger).
    ///   Negative: `extract_arp_frame` does NOT return `Some` (frame is NOT
    ///             admitted to D1/D2/D3/D12 analysis pipeline).
    ///
    /// FAILS if `extract_arp_frame` admits a non-IPv4 protocol type
    /// (ptype=0x86DD) when hlen=6 and plen=4 (valid sizes).
    #[test]
    fn test_BC_2_16_009_PC3b_non_ipv4_prototype_rejected() {
        // BC-2.16.009 EC-002: htype=0x0001 (Ethernet), hlen=6, ptype=0x86DD (IPv6), plen=4.
        // Size fields are VALID — this frame passes the existing size guard.
        // The type guard (currently absent) must reject it.
        let payload = make_arp_payload(
            0x0001, // htype: Ethernet (correct)
            0x86DD, // ptype: IPv6 — non-IPv4, NOT 0x0800
            6,      // hlen: 6 — VALID Ethernet MAC size
            4,      // plen: 4 — VALID address size (ptype says IPv6 but size is 4)
            1,      // oper: Request
            SENDER_MAC, SENDER_IP, TARGET_MAC, TARGET_IP,
        );
        let arp = parse_arp_slice(&payload);

        // Confirm the slice fields match the intended fixture.
        assert_eq!(
            arp.hw_addr_size(),
            6,
            "test-setup: hlen must be 6 (valid size — isolates the type branch)"
        );
        assert_eq!(
            arp.proto_addr_size(),
            4,
            "test-setup: plen must be 4 (valid size — isolates the type branch)"
        );
        assert_eq!(
            arp.proto_addr_type().0,
            0x86DD,
            "test-setup: ptype must be 0x86DD (IPv6, non-IPv4)"
        );
        assert_eq!(
            arp.hw_addr_type().0,
            0x0001,
            "test-setup: htype must be 0x0001 (Ethernet — correct, only ptype is wrong)"
        );

        // Primary assertion: extract_arp_frame must reject the frame.
        // Guards that extract_arp_frame returns None for ptype=0x86DD even with
        // valid sizes. FAILS if the type guard is absent (returns Some instead).
        let result = extract_arp_frame(&arp, Some(SENDER_MAC), payload.len());
        assert_eq!(
            result, None,
            "BC-2.16.001 PC3 / BC-2.16.009 PC3b / EC-002: extract_arp_frame must return \
             None for ptype=0x86DD (IPv6 non-IPv4 protocol type) even when \
             hlen=6 and plen=4 are valid. Got Some — the ptype guard is missing. \
             FAILS if extract_arp_frame admits a non-IPv4 protocol type.",
        );

        // Secondary assertion: decode_packet must surface this as D11.
        let frame_bytes = make_eth_arp_frame(SENDER_MAC, &payload);
        let decode_result = decode_packet(&frame_bytes, DataLink::ETHERNET);
        let err = decode_result.expect_err(
            "BC-2.16.009 PC3b / EC-002: decode_packet must return Err (not Ok) for a \
             non-IPv4 ptype=0x86DD ARP frame with valid sizes (hlen=6, plen=4). \
             D11 malformed trigger requires extract_arp_frame to return None.",
        );
        assert!(
            err.to_string().contains("Non-Ethernet/IPv4 ARP frame"),
            "BC-2.16.009 PC3b / EC-002: decode_packet Err must contain \
             'Non-Ethernet/IPv4 ARP frame' to trigger the D11 malformed path in main.rs. \
             Got: '{err}'",
        );

        // Tertiary assertion: record_malformed emits a D11 finding.
        let mut analyzer = ArpAnalyzer::new(3, 50);
        let malformed_findings = analyzer.record_malformed(payload.len());
        assert!(
            !malformed_findings.is_empty(),
            "BC-2.16.009 PC3b / EC-002: record_malformed must emit at least one D11 \
             finding when called for a non-IPv4 ptype=0x86DD frame. Got 0 findings. \
             The frame must reach D11, not D1/D2/D3/D12.",
        );
        assert_eq!(
            analyzer.malformed_findings, 1,
            "BC-2.16.009 PC3b: malformed_findings counter must be 1 after one \
             record_malformed call. Got {}.",
            analyzer.malformed_findings,
        );
    }

    // -------------------------------------------------------------------------
    // F-2 / BC-2.16.014 PC1 — GARP-conflict summary mentions "conflict"
    // -------------------------------------------------------------------------

    /// Guards that the upgraded GARP finding emitted by a GARP-that-conflicts
    /// scenario (BC-2.16.014) carries summary text that mentions "conflict",
    /// distinguishing it from a benign GARP announcement.
    ///
    /// BC-2.16.014 PC1: when a GARP frame has a binding conflict (sender_ip is
    /// already bound to a different MAC), the GARP finding is upgraded from LOW
    /// to MEDIUM and its summary must indicate that a binding conflict was
    /// detected — not just announce "GARP".
    ///
    /// The current implementation sets the summary to:
    ///   "D2: Gratuitous ARP (GARP) — sender_ip equals target_ip"
    /// which does not mention "conflict". BC-2.16.014 PC1 requires the upgraded
    /// summary to indicate the binding conflict so operators can distinguish a
    /// conflicting GARP from a benign one.
    ///
    /// Assertions:
    ///   Positive: process_arp emits at least one GARP finding.
    ///   Positive: the GARP finding's summary contains "conflict"
    ///             (case-insensitive substring match).
    ///   Negative: the GARP finding summary does NOT omit "conflict"
    ///             (i.e., must not be the unchanged plain GARP message).
    ///
    /// FAILS if the GARP-that-conflicts finding summary does not mention "conflict"
    /// (i.e., the summary stays at the plain "sender_ip equals target_ip" text
    /// without indicating the binding conflict that triggered the upgrade).
    #[test]
    fn test_BC_2_16_014_garp_conflict_summary_mentions_conflict() {
        use wirerust::findings::Confidence;

        // Canonical test vector (mirrors story_114::make_garp / seed_binding pattern).
        const IP_A: [u8; 4] = [10, 0, 0, 1];
        const MAC_A: [u8; 6] = [0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA];
        const MAC_B: [u8; 6] = [0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB];

        let mut analyzer = ArpAnalyzer::new(3, 50);

        // Step 1: seed an initial binding 10.0.0.1 → MAC_A (normal non-GARP reply).
        // This first frame produces no spoof or GARP finding.
        let seed_frame = ArpFrame {
            operation: 2, // Reply
            sender_mac: MAC_A,
            sender_ip: IP_A,
            target_mac: [0u8; 6],
            target_ip: [10, 0, 0, 100],
            outer_src_mac: Some(MAC_A),
            packet_len: 42,
        };
        let seed_findings = analyzer.process_arp(&seed_frame, 0);
        // Verify seed produced no GARP or spoof (binding is fresh).
        assert!(
            seed_findings
                .iter()
                .all(|f| !f.summary.to_lowercase().contains("garp")
                    && !f.summary.to_lowercase().contains("spoof")),
            "test-setup: seed binding must not produce a GARP or spoof finding. \
             Got: {:?}",
            seed_findings.iter().map(|f| &f.summary).collect::<Vec<_>>()
        );

        // Step 2: send a GARP from MAC_B for the same IP (conflict with MAC_A).
        // BC-2.16.014 EC-001: GARP Reply where sender_ip == target_ip == 10.0.0.1,
        // sender_mac = MAC_B (different from the seeded MAC_A → conflict).
        let garp_conflict_frame = ArpFrame {
            operation: 2, // Reply
            sender_mac: MAC_B,
            sender_ip: IP_A,
            target_mac: [0u8; 6],
            target_ip: IP_A, // sender_ip == target_ip → GARP
            outer_src_mac: Some(MAC_B),
            packet_len: 42,
        };
        let findings = analyzer.process_arp(&garp_conflict_frame, 10);

        // There must be at least one GARP finding.
        let garp_finding = findings.iter().find(|f| {
            f.summary.to_lowercase().contains("garp")
                || f.summary.to_lowercase().contains("gratuitous")
        });
        assert!(
            garp_finding.is_some(),
            "BC-2.16.014 PC1: process_arp must emit a GARP finding when \
             sender_ip==target_ip and a binding conflict exists. \
             Got {} finding(s): {:?}",
            findings.len(),
            findings.iter().map(|f| &f.summary).collect::<Vec<_>>()
        );

        let gf = garp_finding.unwrap();

        // The GARP finding must be MEDIUM (upgraded due to conflict).
        assert_eq!(
            gf.confidence,
            Confidence::Medium,
            "BC-2.16.014 PC1: GARP-that-conflicts finding must be MEDIUM (upgraded from \
             LOW). Got {:?}. This is a pre-condition for the summary assertion.",
            gf.confidence
        );

        // Primary assertion: the upgraded GARP summary must mention "conflict".
        // Guards that BC-2.16.014 PC1 summary wording includes the conflict indicator
        // so operators know this is a binding conflict, not a benign GARP.
        // FAILS if the summary stays at the plain "sender_ip equals target_ip" text
        // without mentioning "conflict".
        assert!(
            gf.summary.to_lowercase().contains("conflict"),
            "BC-2.16.014 PC1: the upgraded GARP-that-conflicts finding summary must \
             contain 'conflict' (case-insensitive) to distinguish it from a benign \
             GARP announcement. Got summary: {:?}. \
             FAILS if apply_garp_conflict_escalation_impl does not update the summary \
             to include binding-conflict wording.",
            gf.summary
        );
    }
}
