//! D-078 / BC-2.16.009 + BC-2.16.015: lax-path malformed-ARP D11 routing tests.
//!
//! Guards that a malformed ARP frame (one whose header declares a BAD size or type
//! that would cause `extract_arp_frame` to return `None`) produces a **D11 malformed
//! finding** regardless of which decode path is taken.  The key distinction:
//!
//! - STRICT path (full ARP payload present): `SlicedPacket::from_ethernet` succeeds;
//!   the `NetSlice::Arp` arm calls `extract_arp_frame`, which returns `None` for
//!   bad fields; `decode_packet` returns `Err("Non-Ethernet/IPv4 ARP frame")`.
//!   `main.rs` line 271 matches this string and routes to `record_malformed` → D11. ✓
//!
//! - LAX path (ARP payload too short for its declared sizes): `SlicedPacket::from_ethernet`
//!   fails with `SliceError::Len`; `decode_packet` falls back to `LaxSlicedPacket`.
//!   The lax parser also cannot build an `ArpPacketSlice` (same minimum-size check),
//!   so `lax.net` is `None` with `stop_err = Layer::Arp`.  The lax `None` arm at
//!   `decoder.rs:265` returns `Err("truncated ARP frame")`.  Because `main.rs` only
//!   matches `"Non-Ethernet/IPv4 ARP frame"`, a malformed-and-short frame silently
//!   falls through to the generic decode-error bucket — **no D11** is emitted.
//!
//! ## What the tests exercise
//!
//! ### F5 finding O-A / D-078 (BC-2.16.009 + BC-2.16.015)
//!
//! `test_BC_2_16_009_D078_lax_malformed_arp_routes_to_d11`
//!   A frame with `hlen=8` (non-Ethernet hardware-address length) whose ARP payload
//!   is only 8 bytes (the fixed header — far shorter than the 32 bytes that `hlen=8,
//!   plen=4` requires).  The strict parser fails with `SliceError::Len`; the lax
//!   parser also fails to build an `ArpPacketSlice`, so the lax `None` arm fires.
//!   In the current (unfixed) code the frame is returned as `Err("truncated ARP frame")`
//!   and does **not** increment `malformed_findings`.  After the fix the lax `None` arm
//!   must distinguish "bad fields visible in the 8-byte fixed header" from "genuine
//!   truncation before the fixed header can be read", and route the former to
//!   `record_malformed` → D11.
//!
//!   **RED**: asserts `malformed_findings >= 1`; fails on the current unfixed code.
//!
//! ### Regression guard (BC-2.16.015)
//!
//! `test_BC_2_16_015_D078_unbuildable_truncated_arp_stays_decode_error`
//!   A frame with valid fields (`hlen=6, plen=4`) whose ARP payload is only 20 bytes
//!   (short by 8 bytes of the 28 needed).  The strict parser fails with `SliceError::Len`;
//!   the lax parser also cannot build an `ArpPacketSlice`.  This is genuine truncation —
//!   the frame was cut in transit before the variable section was fully captured.
//!   The lax `None` arm must continue to produce `Err("truncated ARP frame")` and must
//!   NOT route this frame to D11.
//!
//!   **GREEN**: asserts `malformed_findings == 0`; passes on the current unfixed code.
//!   Guards the boundary so the fix does not promote genuine truncation to D11.
//!
//! ## Fixture byte layout
//!
//! Both fixtures share the same Ethernet-over-ARP framing:
//!
//! ```text
//! Ethernet header (14 bytes)
//!   [0..6]  dst MAC: FF:FF:FF:FF:FF:FF (broadcast)
//!   [6..12] src MAC: AA:BB:CC:DD:EE:FF
//!   [12..14] EtherType: 0x0806 (ARP)
//! ARP fixed header (8 bytes) — always present in both fixtures
//!   [14..16] htype: 0x0001 (Ethernet) for the good-fields fixture
//!            htype: 0x0001 (Ethernet) for the bad-fields fixture too
//!   [16..18] ptype: 0x0800 (IPv4)
//!   [18]     hlen: 8 (BAD — non-Ethernet size) for test 1
//!            hlen: 6 (good) for test 2
//!   [19]     plen: 4
//!   [20..22] oper: 0x0001 (Request)
//! Variable data (present or absent determines which arm fires)
//!   test 1 (D078 RED):  NO variable data — total ARP = 8 bytes, total frame = 22 bytes
//!   test 2 (regression): 12 bytes present — total ARP = 20 bytes, total frame = 34 bytes
//!                         (28 bytes needed for hlen=6, plen=4 — 8 bytes short)
//! ```
//!
//! Both fixtures cause `SlicedPacket::from_ethernet` to fail with `SliceError::Len`
//! (ARP payload shorter than declared sizes require) and trigger the lax fallback arm
//! in `decode_packet`.  The lax parser also cannot build an `ArpPacketSlice` for either,
//! so `lax.net == None` with `stop_err = Some((_, Layer::Arp))` in both cases.
//! The current code treats them identically (both return `Err("truncated ARP frame")`).
//! After the fix, test 1 is distinguished by the bad `hlen` field visible in the 8-byte
//! fixed header; test 2 with good fields is left as a decode-error.
//!
//! ## How the test confirms the fixture reaches the right arm
//!
//! The analysis and probe verified:
//! - An 8-byte ARP payload with `hlen=8, plen=4` causes strict to fail (`Len`) and
//!   lax to also set `stop_err = Layer::Arp` (`net = None`).
//! - A 20-byte ARP payload with `hlen=6, plen=4` (28 needed) causes strict to fail
//!   (`Len`) and lax to also set `stop_err = Layer::Arp` (`net = None`).
//! - In both cases `decode_packet` currently returns `Err("truncated ARP frame")`.
//! - The current `main.rs` error-dispatch only matches `"Non-Ethernet/IPv4 ARP frame"`;
//!   `"truncated ARP frame"` falls through to `total_decode_errors += 1` with no D11.
//!
//! The `decode_packet` return values are verified inline before the D11-path assertions.
//!
//! Behavioral contracts covered:
//!   BC-2.16.009 PC3   malformed ARP MUST produce D11 finding regardless of decode path
//!   BC-2.16.015 PC3   lax-path malformed ARP must not silently absorb into decode-errors
//!   BC-2.16.009 PC4   malformed_frames counter must increment for the malformed frame
//!   BC-2.16.009 Inv3  mitre_techniques must be [] for D11 (T0814 withheld)
//!   Decision D-078    O-A finding: lax None arm does not distinguish bad-fields vs truncation
//!
//! DF-TEST-NAMESPACE-001: all tests wrapped in `mod d078_lax_malformed`.
//! DF-AC-TEST-NAME-SYNC-001: function names embed BC and decision identifiers.
//! DF-GREEN-DOC-TENSE-SWEEP (incl. sub-rule d): comments describe what the tests
//!   GUARD, written in the tense of a passing (fixed) test.

#![allow(non_snake_case)]

mod d078_lax_malformed {
    use pcap_file::DataLink;
    use wirerust::analyzer::arp::ArpAnalyzer;
    use wirerust::decoder::decode_packet;
    use wirerust::findings::ThreatCategory;

    // -------------------------------------------------------------------------
    // Fixture builders
    // -------------------------------------------------------------------------

    /// Build an Ethernet/ARP frame where the ARP header declares `hlen=8`
    /// (a non-Ethernet, non-standard hardware-address length) but only the
    /// 8-byte fixed ARP header is present — no variable-data section.
    ///
    /// Wire layout (22 bytes total):
    /// ```text
    /// [0..14]  Ethernet header (dst FF:FF:FF:FF:FF:FF, src AA:BB:CC:DD:EE:FF, EtherType 0x0806)
    /// [14..22] ARP fixed header only: htype=0x0001, ptype=0x0800, hlen=8, plen=4, oper=0x0001
    /// ```
    ///
    /// For `hlen=8, plen=4` the ARP spec needs `8 + 2*8 + 2*4 = 32` bytes of ARP payload.
    /// Only 8 bytes are present, so `ArpPacketSlice::from_slice` reports `required_len=32`
    /// vs `len=8`.  Both the strict and the lax etherparse parsers see this as a length error.
    ///
    /// The 8-byte fixed header is sufficient to read the `hlen` field at offset 4 — a
    /// correct fix can use this to distinguish "bad field visible" from "header truncated
    /// before hlen is readable".
    ///
    /// D-078 canonical fixture: `hlen=8, plen=4, ARP payload = 8 bytes (fixed header only)`.
    fn make_malformed_short_arp_hlen8() -> Vec<u8> {
        let mut frame = Vec::with_capacity(22);
        // Ethernet header (14 bytes)
        frame.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]); // dst MAC broadcast
        frame.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]); // src MAC
        frame.extend_from_slice(&[0x08, 0x06]); // EtherType: ARP
        // ARP fixed header (8 bytes) — hlen=8 (BAD), plen=4
        frame.extend_from_slice(&[0x00, 0x01]); // htype: 0x0001 (Ethernet type field, value valid)
        frame.extend_from_slice(&[0x08, 0x00]); // ptype: 0x0800 (IPv4)
        frame.push(8); // hlen: 8 — NON-ETHERNET hardware address size (should be 6)
        frame.push(4); // plen: 4 — IPv4 address size (correct)
        frame.extend_from_slice(&[0x00, 0x01]); // oper: Request
        // No variable data section — total ARP = 8 bytes (fixed header only)
        assert_eq!(
            frame.len(),
            22,
            "fixture pre-condition: total frame = 22 bytes"
        );
        frame
    }

    /// Build an Ethernet/ARP frame with valid fields (`hlen=6, plen=4`) but the ARP
    /// payload is only 20 bytes — 8 bytes short of the 28 bytes that `hlen=6, plen=4`
    /// requires.
    ///
    /// Wire layout (34 bytes total):
    /// ```text
    /// [0..14]  Ethernet header (dst FF:FF:FF:FF:FF:FF, src AA:BB:CC:DD:EE:FF, EtherType 0x0806)
    /// [14..22] ARP fixed header: htype=0x0001, ptype=0x0800, hlen=6, plen=4, oper=0x0001
    /// [22..34] Partial variable data (12 bytes — sender hw addr + sender proto addr + partial
    ///          target hw addr, cut mid-section)
    /// ```
    ///
    /// For `hlen=6, plen=4` the ARP spec needs `8 + 2*6 + 2*4 = 28` bytes.
    /// Only 20 bytes of ARP payload are present (12 bytes short).  This is genuine
    /// snaplen-induced truncation of an otherwise valid Ethernet/IPv4 ARP frame.
    ///
    /// Regression-guard fixture: `hlen=6, plen=4, ARP payload = 20 bytes (8 short)`.
    fn make_genuine_truncated_arp_hlen6() -> Vec<u8> {
        let mut frame = Vec::with_capacity(34);
        // Ethernet header (14 bytes)
        frame.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]); // dst MAC broadcast
        frame.extend_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]); // src MAC
        frame.extend_from_slice(&[0x08, 0x06]); // EtherType: ARP
        // ARP fixed header (8 bytes) — hlen=6 (valid Ethernet), plen=4 (valid IPv4)
        frame.extend_from_slice(&[0x00, 0x01]); // htype: 0x0001 (Ethernet)
        frame.extend_from_slice(&[0x08, 0x00]); // ptype: 0x0800 (IPv4)
        frame.push(6); // hlen: 6 — correct Ethernet MAC size
        frame.push(4); // plen: 4 — correct IPv4 address size
        frame.extend_from_slice(&[0x00, 0x01]); // oper: Request
        // Partial variable data (12 bytes — 8 short of the 20 needed for hlen=6,plen=4):
        // sender hw addr [6 bytes] + first 6 bytes of sender proto addr + target hw addr
        frame.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]); // sender hw addr (6)
        frame.extend_from_slice(&[0xc0, 0xa8, 0x01, 0x0a]); // sender proto: 192.168.1.10 (4)
        frame.extend_from_slice(&[0x00, 0x00]); // target hw addr (first 2 of 6 — truncated)
        // Total: 14 (eth) + 8 (arp fixed) + 12 (partial variable) = 34 bytes
        assert_eq!(
            frame.len(),
            34,
            "fixture pre-condition: total frame = 34 bytes"
        );
        frame
    }

    // -------------------------------------------------------------------------
    // F5 / D-078 — BC-2.16.009 + BC-2.16.015
    // RED test: lax-path malformed ARP must route to D11
    // -------------------------------------------------------------------------

    /// Guards that a malformed ARP frame (`hlen=8`, non-Ethernet hardware-address
    /// length) whose ARP payload is only 8 bytes (forcing both strict and lax parsers
    /// to encounter a length error) is routed to the D11 malformed-finding path rather
    /// than silently absorbed into generic decode-errors.
    ///
    /// BC-2.16.009 PC3 / BC-2.16.015 / D-078 O-A:
    ///
    /// The lax `None` arm in `decode_packet` (decoder.rs:262-272) currently returns
    /// `Err("truncated ARP frame")` for any `lax.net == None` case where `stop_err`
    /// indicates `Layer::Arp`.  `main.rs` only matches `"Non-Ethernet/IPv4 ARP frame"`
    /// for D11 routing; `"truncated ARP frame"` falls through to `total_decode_errors`.
    /// A malformed-and-short frame is thus miscounted as a generic decode error with no
    /// D11 finding emitted.
    ///
    /// The fix must distinguish "the 8-byte fixed ARP header is present and reveals a
    /// bad `hlen` / `plen` field" from "the frame was truncated before the fixed header
    /// is complete".  In the former case the lax `None` arm must return
    /// `Err("Non-Ethernet/IPv4 ARP frame")` (triggering D11 in main.rs); in the latter
    /// it continues to return `Err("truncated ARP frame")`.
    ///
    /// Assertions:
    ///   Positive: `decode_packet` returns an `Err` for this frame (not `Ok`).
    ///   Positive: when the fix is applied, the `Err` message changes from
    ///             `"truncated ARP frame"` to `"Non-Ethernet/IPv4 ARP frame"`,
    ///             so `main.rs` routes to `record_malformed`.
    ///   Positive: `record_malformed` increments `malformed_findings` to >= 1.
    ///   Positive: `record_malformed` increments `malformed_frames` to >= 1.
    ///   Positive: the D11 finding carries `category = Anomaly` and empty MITRE list.
    ///   Negative: the frame is NOT silently absorbed into `total_decode_errors` only
    ///             (malformed_findings must be >= 1, not 0).
    ///
    /// FAILS on the current (unfixed) code because the lax `None` arm returns
    /// `"truncated ARP frame"`, which `main.rs` does not route to `record_malformed`,
    /// so `malformed_findings` stays at 0.
    #[test]
    fn test_BC_2_16_009_D078_lax_malformed_arp_routes_to_d11() {
        // ---- Fixture ----
        // 22-byte Ethernet/ARP frame: hlen=8 (malformed), ARP payload = 8 bytes only.
        // The strict parser fails with SliceError::Len (needs 32 bytes of ARP, has 8).
        // The lax parser also fails at ArpPacketSlice::from_slice for the same reason.
        // Current code: lax None arm → Err("truncated ARP frame") → decode-error (no D11).
        // After fix: lax None arm reads hlen from the 8 available bytes → bad hlen=8 detected
        //            → returns Err("Non-Ethernet/IPv4 ARP frame") → D11 routed in main.rs.
        let frame = make_malformed_short_arp_hlen8();
        assert_eq!(
            frame.len(),
            22,
            "D-078 fixture pre-condition: frame must be 22 bytes \
             (14 Ethernet + 8 ARP fixed header, no variable data)"
        );

        // ---- Confirm decode_packet returns Err (neither arm produces Ok) ----
        let decode_result = decode_packet(&frame, DataLink::ETHERNET);
        assert!(
            decode_result.is_err(),
            "D-078 / BC-2.16.009: decode_packet must return Err for a malformed-and-short \
             ARP frame (hlen=8, ARP payload = 8 bytes). Got Ok — fixture or decode logic error."
        );

        // ---- Verify the frame is NOT currently routed to D11 (confirm RED state) ----
        // Current code returns "truncated ARP frame", which main.rs does NOT route to
        // record_malformed. After the fix it returns "Non-Ethernet/IPv4 ARP frame".
        // We assert the D11-path behavior (what the fix must produce), which currently fails.
        let err_msg = decode_result.unwrap_err().to_string();
        // Confirm it IS an ARP error (either the current wrong string or the future right one).
        let is_arp_error =
            err_msg.contains("truncated ARP frame") || err_msg.contains("Non-Ethernet/IPv4 ARP");
        assert!(
            is_arp_error,
            "D-078: decode_packet Err must be an ARP-related error (not a generic parse error). \
             Got: '{err_msg}'"
        );

        // ---- Simulate the main.rs D11 routing and assert D11 is emitted ----
        // Guards that when main.rs receives the error string from decode_packet and routes
        // to record_malformed (as the fix will ensure), a D11 finding IS emitted.
        //
        // This is the core RED assertion: on the unfixed code, the error string is
        // "truncated ARP frame", which main.rs does NOT match for D11 routing, so
        // record_malformed is never called. malformed_findings stays 0.
        //
        // After the fix: error string becomes "Non-Ethernet/IPv4 ARP frame";
        // main.rs calls record_malformed; malformed_findings becomes 1.
        //
        // We emulate the main.rs routing logic to make the test self-contained:
        // if the error message contains "Non-Ethernet/IPv4 ARP frame", route to D11.
        let mut analyzer = ArpAnalyzer::new(3, 50);

        // Route using the same condition as main.rs:271 (the condition the fix must satisfy).
        let routed_to_d11 = err_msg.contains("Non-Ethernet/IPv4 ARP frame");
        if routed_to_d11 {
            let d11_findings = analyzer.record_malformed(frame.len());
            assert!(
                !d11_findings.is_empty(),
                "D-078 / BC-2.16.009 PC3: record_malformed must emit at least one D11 finding \
                 when called for a malformed-and-short ARP frame (hlen=8). Got 0 findings."
            );
        }

        // ---- RED assertion: malformed_findings must be >= 1 (FAILS on current code) ----
        // On current (unfixed) code: routed_to_d11 = false → record_malformed not called
        //   → malformed_findings = 0 → this assert FAILS.
        // After fix: routed_to_d11 = true → record_malformed called → malformed_findings = 1
        //   → this assert PASSES.
        assert!(
            analyzer.malformed_findings >= 1,
            "D-078 / BC-2.16.009 PC3 / BC-2.16.015: malformed_findings must be >= 1 after \
             processing a malformed-and-short ARP frame (hlen=8, ARP payload = 8 bytes). \
             Got malformed_findings = {}. \
             \nCurrent decode_packet error: '{err_msg}'. \
             \nFAILS because the lax None arm returns 'truncated ARP frame' instead of \
             'Non-Ethernet/IPv4 ARP frame', so main.rs does NOT route to record_malformed \
             (D-078 O-A finding: lax path absorbs malformed frame as generic decode-error).",
            analyzer.malformed_findings
        );

        // ---- Additional D11 finding quality assertions (run only when routed to D11) ----
        // Guards that the D11 finding has the correct category and empty MITRE list.
        if routed_to_d11 {
            // Re-call to get the findings (record_malformed already incremented counter above)
            let mut analyzer2 = ArpAnalyzer::new(3, 50);
            let d11_findings = analyzer2.record_malformed(frame.len());

            let d11 = d11_findings.first().expect(
                "D-078 / BC-2.16.009 PC3: record_malformed must return at least one finding",
            );

            // BC-2.16.009 Invariant 1: category must be Anomaly
            assert_eq!(
                d11.category,
                ThreatCategory::Anomaly,
                "D-078 / BC-2.16.009 Inv1: D11 finding must have category Anomaly. Got {:?}",
                d11.category
            );

            // BC-2.16.009 Invariant 3: mitre_techniques must be empty (T0814 withheld)
            assert!(
                d11.mitre_techniques.is_empty(),
                "D-078 / BC-2.16.009 Inv3: D11 finding must have empty mitre_techniques \
                 (T0814 withheld per DF-VALIDATION-001). Got {:?}",
                d11.mitre_techniques
            );

            // BC-2.16.009 PC3: summary must mention "D11" or "malformed"
            let summary_lower = d11.summary.to_lowercase();
            assert!(
                summary_lower.contains("d11") || summary_lower.contains("malformed"),
                "D-078 / BC-2.16.009 PC3: D11 finding summary must mention 'D11' or 'malformed'. \
                 Got: {:?}",
                d11.summary
            );

            // malformed_frames counter must also increment (BC-2.16.009 PC4)
            assert_eq!(
                analyzer2.malformed_frames, 1,
                "D-078 / BC-2.16.009 PC4: malformed_frames must be 1 after one \
                 record_malformed call. Got {}.",
                analyzer2.malformed_frames
            );
        }
    }

    // -------------------------------------------------------------------------
    // Regression guard — BC-2.16.015
    // GREEN test: genuine truncation stays as decode-error (no D11)
    // -------------------------------------------------------------------------

    /// Guards that a genuinely-truncated ARP frame with valid header fields
    /// (`hlen=6, plen=4`) whose ARP payload is only 20 bytes (8 short of the
    /// 28 needed) continues to produce a `"truncated ARP frame"` decode-error
    /// and does NOT produce a D11 finding.
    ///
    /// BC-2.16.015 lax-path boundary / D-078 regression guard:
    ///
    /// This frame represents a legitimate snaplen-induced truncation: the sender's
    /// hardware and protocol fields indicate a normal Ethernet/IPv4 ARP Request, but
    /// the capture was cut before all variable data arrived.  The lax `None` arm
    /// (`stop_err = Layer::Arp`) correctly identifies this as a decode limitation,
    /// not a malformed-frame event.  After the D-078 fix is applied, the fix must
    /// NOT promote this frame to D11 (it has valid field values; the shortage is
    /// purely a capture artifact).
    ///
    /// Assertions:
    ///   Positive: `decode_packet` returns `Err` containing `"truncated ARP frame"`.
    ///   Positive: routing via the main.rs condition does NOT call `record_malformed`
    ///             (the error string is `"truncated ARP frame"`, not the D11 trigger).
    ///   Positive: `malformed_findings` remains 0.
    ///   Negative: `decode_packet` does NOT return `Err` containing
    ///             `"Non-Ethernet/IPv4 ARP frame"` (that would be a false D11).
    ///
    /// PASSES on the current (unfixed) code.  Must continue to pass after the fix.
    #[test]
    fn test_BC_2_16_015_D078_unbuildable_truncated_arp_stays_decode_error() {
        // ---- Fixture ----
        // 34-byte Ethernet/ARP frame: hlen=6 (valid), plen=4 (valid),
        // ARP payload = 20 bytes (28 needed — genuinely truncated variable section).
        // The strict parser fails with SliceError::Len (needs 28 bytes of ARP, has 20).
        // The lax parser also fails at ArpPacketSlice::from_slice for the same reason.
        // Both current and fixed code: lax None arm → Err("truncated ARP frame").
        // main.rs does NOT route "truncated ARP frame" to record_malformed → no D11. ✓
        let frame = make_genuine_truncated_arp_hlen6();
        assert_eq!(
            frame.len(),
            34,
            "D-078 regression fixture pre-condition: frame must be 34 bytes \
             (14 Ethernet + 20 ARP bytes — 8 short of the 28 needed for hlen=6,plen=4)"
        );

        // ---- Confirm decode_packet returns Err ----
        let decode_result = decode_packet(&frame, DataLink::ETHERNET);
        assert!(
            decode_result.is_err(),
            "D-078 regression: decode_packet must return Err for a genuinely-truncated \
             ARP frame (hlen=6, ARP payload = 20 of 28 bytes). Got Ok — fixture error."
        );

        // ---- Confirm the error is "truncated ARP frame" (not D11 trigger) ----
        // Guards that a genuinely truncated ARP with valid fields remains a decode-error
        // and is NOT promoted to D11 by the fix.
        let err_msg = decode_result.unwrap_err().to_string();
        assert!(
            err_msg.contains("truncated ARP frame"),
            "D-078 regression / BC-2.16.015: a genuinely-truncated ARP frame with valid \
             fields (hlen=6, plen=4) must produce Err containing 'truncated ARP frame', \
             not 'Non-Ethernet/IPv4 ARP frame'. \
             Got: '{err_msg}'. \
             FAILS if the D-078 fix incorrectly promotes genuine truncation to D11."
        );

        // Confirm it does NOT contain the D11 trigger string
        assert!(
            !err_msg.contains("Non-Ethernet/IPv4 ARP frame"),
            "D-078 regression / BC-2.16.015: genuine truncation (valid hlen=6, plen=4) \
             must NOT produce 'Non-Ethernet/IPv4 ARP frame'. Got: '{err_msg}'."
        );

        // ---- Emulate main.rs routing: confirm record_malformed is NOT called ----
        // Guards that the error string does not trigger the D11 route in main.rs.
        let mut analyzer = ArpAnalyzer::new(3, 50);
        let routed_to_d11 = err_msg.contains("Non-Ethernet/IPv4 ARP frame");
        if routed_to_d11 {
            analyzer.record_malformed(frame.len());
        }

        // malformed_findings must remain 0 for genuine truncation (no D11 emitted)
        assert_eq!(
            analyzer.malformed_findings, 0,
            "D-078 regression / BC-2.16.015: malformed_findings must be 0 for a genuinely \
             truncated ARP frame with valid fields (hlen=6, plen=4, ARP = 20 of 28 bytes). \
             Got malformed_findings = {}. \
             The D-078 fix must NOT promote genuine truncation to D11.",
            analyzer.malformed_findings
        );
    }
}
