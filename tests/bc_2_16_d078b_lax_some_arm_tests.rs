//! D-078b / BC-2.16.009 v1.6: lax `Some(LaxNetSlice::Arp(arp))` arm — reachability
//! investigation and defensive-fix test for the sibling of D-078.
//!
//! ## Background
//!
//! `decode_packet` (decoder.rs) has two ARP dispatch arms after falling back to lax
//! parsing when strict fails with `SliceError::Len`:
//!
//!   (A) `Some(LaxNetSlice::Arp(arp))` at decoder.rs:236 — lax parser built an
//!       `ArpPacketSlice`.  Calls `extract_arp_frame`.  On `None` (bad type/size):
//!       **Currently** returns `Err("truncated ARP frame")` at decoder.rs:248.
//!       **Required** per BC-2.16.009 v1.6 PC3: `Err("Non-Ethernet/IPv4 ARP frame")`.
//!
//!   (B) `None` at decoder.rs:287 — lax parser could NOT build an `ArpPacketSlice`.
//!       Already fixed by D-078 to distinguish malformed vs genuine truncation.
//!
//! The D-078b fix is a one-line change: decoder.rs:248
//!   from: `None => Err(anyhow!("truncated ARP frame"))`
//!   to:   `None => Err(anyhow!("Non-Ethernet/IPv4 ARP frame"))`
//!
//! ## Reachability Verdict: NOT REACHABLE via integration-level construction
//!
//! For arm (A) to fire with `extract_arp_frame → None`, two conditions must hold
//! simultaneously:
//!
//!   1. Strict parse fails with `SliceError::Len` (lax fallback triggered).
//!   2. Lax parse succeeds in building `ArpPacketSlice` (`net = Some(Arp)`).
//!
//! These conditions are mutually exclusive for all supported datalinks:
//!
//!   ETHERNET: Both strict and lax call `Ethernet2Slice::from_slice_without_fcs`
//!   (same function) then `ArpPacketSlice::from_slice` on the same ARP payload bytes.
//!   No length field in the ARP header that strict validates and lax ignores.
//!   If the payload meets the byte-count minimum → both succeed → strict arm handles it.
//!   If the payload is too short → both fail → arm (B) fires, not arm (A).
//!
//!   LINUX_SLL: The custom `lax_parse` path reads `data[14..16]` as EtherType then
//!   calls `LaxSlicedPacket::from_ether_type(ARP, &data[16..])`.  The strict path calls
//!   `SlicedPacket::from_linux_sll(data)` which eventually calls `ArpPacketSlice::from_slice`
//!   on the same `data[16..]`.  Same bytes, same result.
//!
//!   RAW/IPV4/IPV6: No ARP dispatch; produces `"No IP layer found"`.
//!
//! ## What this file tests
//!
//! ### Reachability probes (permanently GREEN)
//!
//! Two empirical probes confirm the unreachability via etherparse API calls:
//!
//! `test_BC_2_16_009_D078b_empirical_full_size_bad_htype_reaches_strict_arm`
//!   42-byte frame with htype=0x0006, hlen=6, plen=4 → strict SUCCEEDS (strict arm
//!   fires) → lax arm (A) NOT entered.  Pins `decode_packet` → `"Non-Ethernet/IPv4 ARP frame"`.
//!
//! `test_BC_2_16_009_D078b_empirical_short_bad_htype_reaches_lax_none_arm_not_some`
//!   30-byte truncation of the above → strict fails Len, lax ALSO fails (`net = None`,
//!   `stop_err = Layer::Arp`) → arm (B) fires, NOT arm (A).
//!
//! ### Defensive source-inspection test (RED on unfixed code)
//!
//! `test_BC_2_16_009_D078b_lax_some_arm_malformed_routes_to_d11`
//!   The arm is integration-unreachable, so the RED assertion is made by reading the
//!   decoder.rs source and asserting that decoder.rs:248 does NOT contain the broken
//!   string `"truncated ARP frame"` at the location of the `None =>` branch inside
//!   `Some(LaxNetSlice::Arp(arp))`.  On current (unfixed) code this string is present
//!   → the test FAILS (RED).  After the fix, the string is `"Non-Ethernet/IPv4 ARP frame"`
//!   → the test passes.
//!
//! Behavioral contracts covered:
//!   BC-2.16.009 v1.6 PC3    malformed ARP MUST produce D11 regardless of decode path
//!   BC-2.16.009 v1.6 PC3a   hw_addr_type != ETHERNET → D11 malformed finding
//!
//! DF-TEST-NAMESPACE-001: tests wrapped in `mod d078b_lax_some_arm`.
//! DF-AC-TEST-NAME-SYNC-001: names embed BC and decision identifiers.
//! DF-GREEN-DOC-TENSE-SWEEP (sub-rule d): regression-guard tests use "guards" framing.

#![allow(non_snake_case)]

mod d078b_lax_some_arm {
    use etherparse::err::packet::SliceError;
    use etherparse::{ArpPacketSlice, LaxSlicedPacket};
    use pcap_file::DataLink;
    use wirerust::decoder::{decode_packet, extract_arp_frame};

    // -------------------------------------------------------------------------
    // Fixture builders
    // -------------------------------------------------------------------------

    /// Build a 42-byte Ethernet/ARP frame with htype=0x0006 (IEEE 802 non-Ethernet)
    /// and valid hlen=6, plen=4, with a complete 28-byte ARP variable section.
    ///
    /// `ArpPacketSlice::from_slice` succeeds (28 ≥ 8 + 6×2 + 4×2).
    /// `extract_arp_frame` returns `None` (htype ≠ ETHERNET).
    fn make_eth_arp_htype0006_full_size() -> Vec<u8> {
        let mut frame = Vec::with_capacity(42);
        frame.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]); // dst: broadcast
        frame.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]); // src MAC
        frame.extend_from_slice(&[0x08, 0x06]); // EtherType: ARP
        frame.extend_from_slice(&[0x00, 0x06]); // htype: 0x0006 (IEEE 802 — BAD)
        frame.extend_from_slice(&[0x08, 0x00]); // ptype: IPv4
        frame.push(6); // hlen: 6 (valid)
        frame.push(4); // plen: 4 (valid)
        frame.extend_from_slice(&[0x00, 0x01]); // oper: Request
        frame.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]); // sender hw (6)
        frame.extend_from_slice(&[10, 0, 0, 1]); // sender ip (4)
        frame.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // target hw (6)
        frame.extend_from_slice(&[10, 0, 0, 2]); // target ip (4)
        assert_eq!(frame.len(), 42, "D-078b fixture: 42 bytes");
        frame
    }

    /// Build a 28-byte ARP payload (no Ethernet header) with htype=0x0006 (bad type)
    /// and valid hlen=6, plen=4.  Used to call `extract_arp_frame` directly.
    fn make_arp_payload_htype0006() -> Vec<u8> {
        let mut p = Vec::with_capacity(28);
        p.extend_from_slice(&[0x00, 0x06]); // htype: IEEE 802 (BAD)
        p.extend_from_slice(&[0x08, 0x00]); // ptype: IPv4
        p.push(6); // hlen: 6
        p.push(4); // plen: 4
        p.extend_from_slice(&[0x00, 0x01]); // oper: Request
        p.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]); // sender hw
        p.extend_from_slice(&[10, 0, 0, 1]); // sender ip
        p.extend_from_slice(&[0x00; 6]); // target hw
        p.extend_from_slice(&[10, 0, 0, 2]); // target ip
        assert_eq!(p.len(), 28, "D-078b ARP payload: 28 bytes");
        p
    }

    // -------------------------------------------------------------------------
    // Reachability Probe A — permanently GREEN
    // Full-size bad-htype ARP → strict arm fires, lax arm (A) not entered.
    // -------------------------------------------------------------------------

    /// Guards the reachability verdict (half A): a full-size Ethernet/ARP frame with
    /// htype=0x0006 (bad type) and valid hlen=6/plen=4 is handled entirely by the
    /// strict parse arm — the lax arm (A) is never entered.
    ///
    /// `ArpPacketSlice::from_slice` validates byte-count only (not htype/ptype).
    /// With 28 bytes of ARP payload, both strict and lax parsers succeed.
    /// Strict succeeds first → strict `Some(NetSlice::Arp)` arm intercepts →
    /// `extract_arp_frame → None` → `Err("Non-Ethernet/IPv4 ARP frame")`.
    /// The lax fallback is never triggered.
    #[test]
    fn test_BC_2_16_009_D078b_empirical_full_size_bad_htype_reaches_strict_arm() {
        let frame = make_eth_arp_htype0006_full_size();

        // Strict parse must succeed (28 bytes satisfies the byte-count minimum).
        let strict_result = etherparse::SlicedPacket::from_ethernet(&frame);
        assert!(
            strict_result.is_ok(),
            "D-078b probe A: strict parse must succeed for a 42-byte Ethernet/ARP frame \
             with htype=0x0006 and valid hlen=6, plen=4. \
             ArpPacketSlice::from_slice validates byte-count only. Got Err."
        );
        assert!(
            matches!(
                strict_result.unwrap().net,
                Some(etherparse::NetSlice::Arp(_))
            ),
            "D-078b probe A: strict net must be Some(Arp)"
        );

        // decode_packet takes the strict arm → extract_arp_frame → None → correct string.
        let err = decode_packet(&frame, DataLink::ETHERNET)
            .expect_err("D-078b probe A: must return Err for bad-htype ARP");
        assert!(
            err.to_string().contains("Non-Ethernet/IPv4 ARP frame"),
            "D-078b probe A: strict arm must produce 'Non-Ethernet/IPv4 ARP frame'. \
             Got: '{err}'"
        );
        // Lax arm (A) NOT entered. Probe A complete.
    }

    // -------------------------------------------------------------------------
    // Reachability Probe B — permanently GREEN
    // Truncated bad-htype ARP → lax arm (B) fires, NOT lax arm (A).
    // -------------------------------------------------------------------------

    /// Guards the reachability verdict (half B): a truncated Ethernet/ARP frame with
    /// htype=0x0006 and hlen=6/plen=4, but only 16 bytes of ARP payload (28 needed),
    /// reaches the lax `None` arm (B) — NOT the lax `Some(Arp)` arm (A).
    ///
    /// Both parsers call `ArpPacketSlice::from_slice` on the same bytes.
    /// Both fail (16 < 28) → lax `net = None`, `stop_err = Layer::Arp` → arm (B) fires.
    /// Arm (A) (`Some(Arp)`) is NOT entered.
    ///
    /// Together with Probe A: when the payload is full-sized → strict handles it (A);
    /// when too short → arm (B) fires (B).  No construction reaches arm (A) with `extract → None`.
    #[test]
    fn test_BC_2_16_009_D078b_empirical_short_bad_htype_reaches_lax_none_arm_not_some() {
        let full_frame = make_eth_arp_htype0006_full_size();
        // Truncate to 30 bytes: 14 Ethernet + 16 ARP (28 needed).
        let short_frame = &full_frame[..30];
        assert_eq!(
            short_frame.len(),
            30,
            "D-078b probe B pre-condition: 30 bytes"
        );

        // Strict must fail with SliceError::Len.
        assert!(
            matches!(
                etherparse::SlicedPacket::from_ethernet(short_frame),
                Err(SliceError::Len(_))
            ),
            "D-078b probe B: strict must fail with SliceError::Len for 16-byte ARP payload"
        );

        // Lax must also fail to build ArpPacketSlice (same bytes → same outcome).
        let lax = LaxSlicedPacket::from_ethernet(short_frame)
            .expect("D-078b probe B: Ethernet header parse must not error");
        assert!(
            lax.net.is_none(),
            "D-078b probe B: lax net must be None — both parsers call ArpPacketSlice::from_slice \
             on the same bytes. Arm (B) fires, NOT arm (A). Got net = Some."
        );
        let (_, stop_layer) = lax
            .stop_err
            .expect("D-078b probe B: stop_err must be Some for failed ARP parse");
        assert_eq!(
            stop_layer,
            etherparse::err::Layer::Arp,
            "D-078b probe B: stop_err layer must be Arp. Got {:?}",
            stop_layer
        );
        // Arm (A) NOT entered. Probe B complete.
    }

    // -------------------------------------------------------------------------
    // D-078b Defensive source-level test — RED on unfixed code
    // -------------------------------------------------------------------------

    /// Guards that decoder.rs does NOT use `"truncated ARP frame"` as the error string
    /// for the `None` branch of the lax `Some(LaxNetSlice::Arp(arp))` arm.
    ///
    /// BC-2.16.009 v1.6 PC3 (path-independence): a malformed ARP MUST produce a D11
    /// finding regardless of decode path.  The lax `Some(Arp)` arm's `None` branch
    /// (decoder.rs:248) currently violates this: it returns `Err("truncated ARP frame")`
    /// — which `main.rs` routes to `total_decode_errors`, NOT to `record_malformed` → D11.
    ///
    /// The strict arm correctly returns `Err("Non-Ethernet/IPv4 ARP frame")` for `None`,
    /// which `main.rs` matches for D11 routing.  The lax arm MUST produce the same string.
    ///
    /// **Reachability:** The lax `Some(Arp)` arm with `extract → None` is NOT reachable
    /// at the integration level (see probes A and B above).  The fix is defensive.
    ///
    /// **RED mechanism (source inspection):**
    ///   This test reads `src/decoder.rs` and asserts that the `None =>` branch inside
    ///   the `Some(LaxNetSlice::Arp(arp))` match arm does NOT contain the string
    ///   `"truncated ARP frame"`.  On current (unfixed) code this string IS present at
    ///   line 248, so the test FAILS.  After the fix — changing that line to
    ///   `None => Err(anyhow!("Non-Ethernet/IPv4 ARP frame"))` — the test passes.
    ///
    /// Additionally guards `extract_arp_frame` behavior and the strict arm via integration:
    ///   Part 1 (GREEN): `extract_arp_frame` returns `None` for a bad-htype slice.
    ///   Part 2 (GREEN): strict arm produces `"Non-Ethernet/IPv4 ARP frame"` via `decode_packet`.
    ///   Part 3 (RED):   decoder.rs source must NOT contain `"truncated ARP frame"` at the
    ///                   lax `Some(Arp)` arm's `None` branch.
    ///
    /// FAILS on current (unfixed) code.  Passes after the one-line fix to decoder.rs:248.
    #[test]
    fn test_BC_2_16_009_D078b_lax_some_arm_malformed_routes_to_d11() {
        // ---- Part 1: extract_arp_frame returns None for bad htype (GREEN) ----
        let payload = make_arp_payload_htype0006();
        let arp = ArpPacketSlice::from_slice(&payload).expect(
            "D-078b test-setup: ArpPacketSlice::from_slice must succeed for 28-byte htype=0x0006",
        );
        assert_eq!(arp.hw_addr_type().0, 0x0006, "test-setup: htype = 0x0006");
        assert_eq!(arp.hw_addr_size(), 6, "test-setup: hlen = 6 (valid)");
        assert_eq!(arp.proto_addr_size(), 4, "test-setup: plen = 4 (valid)");

        let extract_result =
            extract_arp_frame(&arp, Some([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]), 42);
        assert_eq!(
            extract_result, None,
            "BC-2.16.009 D-078b part 1: extract_arp_frame must return None for htype=0x0006. \
             Type guard must fire regardless of valid hlen/plen. Got Some."
        );

        // ---- Part 2: strict arm produces the correct D11 trigger string (GREEN) ----
        let full_frame = make_eth_arp_htype0006_full_size();
        let strict_err = decode_packet(&full_frame, DataLink::ETHERNET)
            .expect_err("D-078b part 2: decode_packet must return Err for bad-htype ARP");
        let strict_err_str = strict_err.to_string();
        assert!(
            strict_err_str.contains("Non-Ethernet/IPv4 ARP frame"),
            "D-078b part 2: strict arm must produce 'Non-Ethernet/IPv4 ARP frame'. \
             Got: '{strict_err_str}'. Regression — re-examine strict arm."
        );

        // ---- Part 3: decoder.rs source must not use "truncated ARP frame" in the
        //              lax Some(Arp) arm's None branch (RED on unfixed code) ----
        //
        // The lax Some(Arp) arm at decoder.rs:236-249 currently ends with:
        //   None => Err(anyhow!("truncated ARP frame"))
        //
        // BC-2.16.009 v1.6 PC3 requires that branch to instead produce:
        //   None => Err(anyhow!("Non-Ethernet/IPv4 ARP frame"))
        //
        // We read decoder.rs and locate the section of source that contains the
        // lax Some(LaxNetSlice::Arp) match arm.  We assert that within the scope
        // of that arm, "truncated ARP frame" does NOT appear as the None-branch string.
        //
        // The source region to inspect: from `Some(LaxNetSlice::Arp(arp)) =>` to the
        // closing `}` of that arm (before `Some(net) =>`).  In the current file this
        // spans roughly lines 236-249.  We locate it by looking for the unique context
        // string `"Some(LaxNetSlice::Arp(arp))"` and checking the following text.
        let decoder_src = std::fs::read_to_string("src/decoder.rs")
            .expect("D-078b part 3: must be able to read src/decoder.rs");

        // Locate the lax Some(LaxNetSlice::Arp(arp)) arm in the source.
        let arm_marker = "Some(LaxNetSlice::Arp(arp))";
        let arm_start = decoder_src.find(arm_marker).expect(
            "D-078b part 3: src/decoder.rs must contain 'Some(LaxNetSlice::Arp(arp))' — \
             the arm exists per the code review",
        );

        // Extract the text from the arm start to the next `Some(net)` arm (the IP arm).
        // This isolates the Some(Arp) arm body where the None branch lives.
        let after_arm = &decoder_src[arm_start..];
        let arm_end = after_arm.find("Some(net) =>").unwrap_or(after_arm.len());
        let arm_body = &after_arm[..arm_end];

        // The arm body must contain the correct None-branch string, not the broken one.
        let broken_string = "truncated ARP frame";
        let correct_string = "Non-Ethernet/IPv4 ARP frame";

        // RED assertion: the broken string must NOT appear in the arm body.
        // FAILS on current (unfixed) decoder.rs:248 which has: Err(anyhow!("truncated ARP frame"))
        // PASSES after the fix: Err(anyhow!("Non-Ethernet/IPv4 ARP frame"))
        assert!(
            !arm_body.contains(broken_string),
            "D-078b / BC-2.16.009 v1.6 PC3: the lax `Some(LaxNetSlice::Arp(arp))` arm's \
             `None =>` branch must NOT return '{}'. \
             This string is semantically wrong for a bad-type/bad-size ARP frame: \
             main.rs only matches '{}' for D11 routing; '{}' falls through to \
             total_decode_errors with no D11 finding. \
             \nFix: change decoder.rs:248 from\n  \
               None => Err(anyhow!(\"{}\"))\n  \
             to\n  \
               None => Err(anyhow!(\"{}\"))\n  \
             \nThe arm body currently contains:\n{}\n  \
             \nFAILS on current (unfixed) code.",
            broken_string,
            correct_string,
            broken_string,
            broken_string,
            correct_string,
            arm_body.trim(),
        );

        // Also assert the correct string IS present in the arm body (after the fix).
        // This guards against a fix that removes the None branch entirely.
        assert!(
            arm_body.contains(correct_string),
            "D-078b / BC-2.16.009 v1.6 PC3: the lax `Some(LaxNetSlice::Arp(arp))` arm's \
             `None =>` branch MUST return '{}' — the D11 trigger string. \
             After the fix, decoder.rs:248 must read: \
             `None => Err(anyhow!(\"{}\"))`. \
             Arm body: {}",
            correct_string,
            correct_string,
            arm_body.trim(),
        );
    }
}
