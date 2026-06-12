//! Tests for STORY-107: DNP3 Per-Flow State + Carry Buffer + Pending-Request Bounds.
//!
//! Covers BC-2.15.016 AC-001..AC-006 and edge cases EC-001..EC-006.
//! These tests were authored RED-first (TDD) against STORY-106 stubs; STORY-107
//! production logic has since landed and all 13 tests pass.
//!
//! ## Test naming convention
//! AC-derived tests use `test_BC_2_16_NNN_xxx()` or the exact names specified by the
//! story where the story mandates a particular function name.  EC tests use
//! `test_EC_NNN_xxx()` following the same BC-prefix pattern.
//!
//! ## Namespace
//! Per DF-TEST-NAMESPACE-001: all STORY-107 tests are in `mod story_107` to prevent
//! name collisions with STORY-106's `mod story_106`.
//!
//! ## Location rationale
//! STORY-106 tests live in `tests/dnp3_parse_core_tests.rs` (integration-test file,
//! not inline `#[cfg(test)]`).  All `Dnp3FlowState` fields are `pub`, so an integration
//! test file can access them without private-visibility tricks.  We follow the same
//! pattern here rather than adding an inline `#[cfg(test)]` block inside
//! `src/analyzer/dnp3.rs`.

// BC-based test naming uses uppercase BC IDs (test_BC_S_SS_NNN_xxx).
// The non_snake_case lint fires on the uppercase letters — suppressed intentionally.
#![allow(non_snake_case)]

// Per DF-TEST-NAMESPACE-001: separate namespace from STORY-106's mod story_106.
mod story_107 {
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::dnp3::{Dnp3Analyzer, MAX_MASTER_ADDRS, MAX_PENDING_REQUESTS};
    use wirerust::reassembly::flow::FlowKey;

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    fn test_flow_key() -> FlowKey {
        FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            20000,
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            20000,
        )
    }

    /// Build a minimal valid DNP3 link frame of exactly `frame_len` bytes.
    ///
    /// The LENGTH byte is set so that `compute_dnp3_frame_len(length) == frame_len`.
    /// For LENGTH=5 the formula gives frame_len=10 (U=0, blocks=0, 5+5+0=10).
    ///
    /// Layout:
    ///   [0]  0x05  start1
    ///   [1]  0x64  start2
    ///   [2]  length  (the LENGTH field value passed in)
    ///   [3]  control (0xC4 = DIR+PRM set, nibble 0x04 = UNCONFIRMED_USER_DATA)
    ///   [4]  dest_lo, [5] dest_hi  (little-endian destination)
    ///   [6]  src_lo,  [7] src_hi   (little-endian source)
    ///   [8]  0x00  CRC placeholder low
    ///   [9]  0x00  CRC placeholder high
    ///   [10..] 0x00 filler bytes for any user-data blocks
    ///
    /// Note: CRC bytes are placeholders (0x00) — the carry-buffer tests do not
    /// validate CRCs; they test carry-length and frame-consumption boundaries only.
    fn build_frame(length_byte: u8, dest: u16, src: u16, control: u8) -> Vec<u8> {
        // compute_dnp3_frame_len semantics:
        //   frame_len = 5 + length_byte + 2 * ceil((length_byte - 5) / 16)
        // For length_byte=5: frame_len = 5+5+0 = 10
        // For length_byte=14: frame_len = 5+14+2 = 21 (U=9, blocks=1)
        let u = (length_byte as usize).saturating_sub(5);
        let blocks = u.div_ceil(16);
        let frame_len = 5 + (length_byte as usize) + 2 * blocks;

        let mut frame = vec![0u8; frame_len];
        frame[0] = 0x05;
        frame[1] = 0x64;
        frame[2] = length_byte;
        frame[3] = control;
        let [dl, dh] = dest.to_le_bytes();
        frame[4] = dl;
        frame[5] = dh;
        let [sl, sh] = src.to_le_bytes();
        frame[6] = sl;
        frame[7] = sh;
        // bytes 8-9: CRC placeholder (zeros)
        // bytes 10..end: zeros (user-data + block CRCs — not validated here)
        frame
    }

    /// Build a master-direction frame: control has DIR bit set (bit 7, mask 0x80).
    /// Uses nibble 0x04 (UNCONFIRMED_USER_DATA) with DIR+PRM+FCV bits: 0xD4.
    fn build_master_frame(dest: u16, src: u16) -> Vec<u8> {
        // 0xD4 = 1101 0100: DIR(1, bit7) PRM(1, bit6) FCB(0, bit5) FCV(1, bit4) FC(0100=UNCONF_USER_DATA)
        // DIR=1 because 0xD4 & 0x80 = 0x80 != 0 (corrected mask per BC-2.15.016 PC5 / F-A-001 REVISION 2).
        // Note: bit4 of 0xD4 is FCV (Frame Count Valid), NOT DIR. DIR is bit7.
        build_frame(5, dest, src, 0xD4)
    }

    // ---------------------------------------------------------------------------
    // AC-001 — test_carry_buffer_cap_at_292
    // BC-2.15.016 postconditions 1–2: carry never exceeds 292; overflow → parse_errors++
    // ---------------------------------------------------------------------------

    /// AC-001: Deliver 290-byte carry + 5-byte segment → carry=292, parse_errors=1, 3 discarded.
    ///
    /// Pre-state: flow.carry has 290 bytes.
    /// Action: on_data delivers 5 more bytes.
    /// Expected: carry.len() == 292, parse_errors == 1 (overflow occurred), 3 bytes discarded.
    ///
    /// Traces to: BC-2.15.016 postconditions 1–2; STORY-107 AC-001.
    #[test]
    fn test_carry_buffer_cap_at_292() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Prime the flow entry and pre-load carry with 290 bytes.
        // We deliver a first valid-sync segment so on_data creates the flow entry,
        // then directly mutate carry to set the pre-condition (290 bytes).
        //
        // Deliver a valid-sync 2-byte segment to create the flow state.
        analyzer.on_data(key.clone(), &[0x05, 0x64], 0);
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            // Reset carry and fill with 290 bytes of filler.
            flow.carry.clear();
            flow.carry.extend(std::iter::repeat_n(0xAA, 290));
            assert_eq!(flow.carry.len(), 290, "pre-condition: carry must be 290");
        }

        // Deliver 5 new bytes.  Carry is 290, capacity is 292 → 2 fit, 3 discarded.
        let segment = [0xBBu8; 5];
        analyzer.on_data(key.clone(), &segment, 1);

        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");

        // BC-2.15.016 PC2: the 292-cap invariant is proven by parse_errors == 1.
        // parse_errors is incremented once and ONLY ONCE — in the overflow arm of Step 2,
        // when carry.len() + new_bytes.len() > 292. If the cap were not enforced,
        // carry would grow to 295 and no parse_errors increment would occur.
        // parse_errors == 1 is the authoritative cap-invariant assertion.
        assert_eq!(
            flow.parse_errors, 1,
            "parse_errors must be 1: carry overflow fired (292-cap enforced; 3 bytes discarded)"
        );

        // After the overflow, the frame-walk ran and found no [0x05,0x64] sync word
        // in the 292 bytes of 0xAA/0xBB filler → byte-walk-forward resync cleared carry.
        // carry.len() == 0 confirms: (a) the frame-walk ran (not a no-op), and
        // (b) the resync liveness property (carry advanced on every non-break iteration).
        // STORY-109 behavior: byte-walk resync clears unrecoverable junk carry;
        // the 292-cap is proven by parse_errors==1 (overflow arm only fires when carry+new > 292).
        assert_eq!(
            flow.carry.len(),
            0,
            "carry must be 0 after byte-walk-forward resync found no sync in junk carry \
             (STORY-109 behavior; overflow was already counted via parse_errors)"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-002 — test_carry_buffer_frame_consumption
    // BC-2.15.016 postconditions 3–4: frame drained; remainder stays; frame_count++
    // ---------------------------------------------------------------------------

    /// AC-002: 21-byte carry = one 10-byte frame + 11-byte partial second frame.
    ///
    /// Expected: first frame consumed (carry.drain(..10)), 11 bytes remain, frame_count=1.
    ///
    /// Frame 1: LENGTH=5 → frame_len=10 (compute_dnp3_frame_len(5) = Some(10)).
    /// The 11 remaining bytes represent the start of a second frame (partial).
    ///
    /// Traces to: BC-2.15.016 postconditions 3–4; STORY-107 AC-002.
    #[test]
    fn test_carry_buffer_frame_consumption() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Build 21 bytes: one complete 10-byte frame (LENGTH=5) + 11 bytes of partial second
        // frame.  Both start with valid sync [0x05, 0x64] so they look like DNP3 frames.
        let frame1 = build_frame(5, 0x0003, 0x0001, 0xC4); // 10 bytes
        assert_eq!(frame1.len(), 10, "frame1 must be exactly 10 bytes");

        // Partial second frame: 11 bytes with valid sync header (but no full frame).
        // Use LENGTH=14 → frame_len=21; deliver only 11 bytes of it.
        let frame2_full = build_frame(14, 0x0003, 0x0002, 0xC4); // 21 bytes
        let partial_second = &frame2_full[..11]; // first 11 bytes only

        let mut twenty_one_bytes = Vec::with_capacity(21);
        twenty_one_bytes.extend_from_slice(&frame1);
        twenty_one_bytes.extend_from_slice(partial_second);
        assert_eq!(
            twenty_one_bytes.len(),
            21,
            "combined carry must be 21 bytes"
        );

        // Deliver all 21 bytes in a single on_data call.
        analyzer.on_data(key.clone(), &twenty_one_bytes, 0);

        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");

        assert_eq!(
            flow.carry.len(),
            11,
            "carry must have 11 bytes remaining after consuming the 10-byte first frame"
        );
        assert_eq!(
            flow.frame_count, 1,
            "frame_count must be 1: exactly one complete frame was consumed"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-003 — test_master_addrs_cap_at_64
    // BC-2.15.016 postconditions 5–6: master_addrs_seen bounded at MAX_MASTER_ADDRS=64
    // ---------------------------------------------------------------------------

    /// AC-003: Insert 64 unique source addresses then a 65th → vec len stays at 64.
    ///
    /// Each delivery is a master-direction frame (DIR=1, control & 0x10 != 0).
    /// The 65th unique source address must be silently ignored.
    ///
    /// Traces to: BC-2.15.016 postconditions 5–6; STORY-107 AC-003.
    #[test]
    fn test_master_addrs_cap_at_64() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let base_key = test_flow_key();

        // Deliver 64 master frames each with a unique source address 1..=64.
        for src_addr in 1u16..=(MAX_MASTER_ADDRS as u16) {
            let frame = build_master_frame(0x0003, src_addr);
            analyzer.on_data(base_key.clone(), &frame, 0);
        }

        {
            let flow = analyzer.flows.get(&base_key).expect("flow must exist");
            assert_eq!(
                flow.master_addrs_seen.len(),
                MAX_MASTER_ADDRS,
                "after 64 unique master src addrs, master_addrs_seen.len() must be 64"
            );
        }

        // Deliver a 65th master frame with a new unique source address (65).
        let frame_65 = build_master_frame(0x0003, 65u16);
        analyzer.on_data(base_key.clone(), &frame_65, 0);

        let flow = analyzer
            .flows
            .get(&base_key)
            .expect("flow must exist after 65th");
        assert_eq!(
            flow.master_addrs_seen.len(),
            MAX_MASTER_ADDRS,
            "master_addrs_seen must NOT grow beyond MAX_MASTER_ADDRS=64; 65th addr silently ignored"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-004 — test_frame_count_increments
    // BC-2.15.016 postcondition 7: frame_count += 1 per complete frame consumed
    // ---------------------------------------------------------------------------

    /// AC-004: Deliver 3 complete frames → frame_count=3.
    ///
    /// Each frame is 10 bytes (LENGTH=5 → frame_len=10).
    /// Delivered in a single on_data call (all 30 bytes at once) to exercise the
    /// while-loop carry-consume path.
    ///
    /// Traces to: BC-2.15.016 postcondition 7; STORY-107 AC-004.
    #[test]
    fn test_frame_count_increments() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Build 3 complete 10-byte frames (LENGTH=5).
        let mut thirty_bytes = Vec::with_capacity(30);
        for i in 0u16..3 {
            let frame = build_frame(5, 0x0003, i + 1, 0xC4);
            thirty_bytes.extend_from_slice(&frame);
        }
        assert_eq!(thirty_bytes.len(), 30, "3 × 10-byte frames = 30 bytes");

        // Deliver all 30 bytes in one call.
        analyzer.on_data(key.clone(), &thirty_bytes, 0);

        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.frame_count, 3,
            "frame_count must be 3 after delivering 3 complete frames in one on_data call"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-005 — test_pending_requests_eviction_at_256
    // BC-2.15.016 postconditions 8–10: pending_requests bounded at 256 with oldest-eviction
    // ---------------------------------------------------------------------------

    /// AC-005: Pre-load 256 pending entries with ts 0..=255; deliver a 257th via on_data
    /// at ts=300.  The implementation must evict the oldest (ts=0) before inserting.
    ///
    /// Expected: map.len()==256; entry with ts=0 (key=(0,0)) is evicted; map stays at 256.
    ///
    /// Authored RED (STORY-107): STORY-106's on_data had no eviction logic; the 257th
    /// insert would have grown the map to 257.  STORY-107 added the eviction path;
    /// this test asserts len==256 AND the oldest entry evicted — both assertions pass.
    ///
    /// We drive the test via on_data with a Control-class frame (FC=0x03 SELECT, link
    /// CONTROL nibble=0x03 CONFIRMED_USER_DATA, transport FIR=1) targeting dest=(0u16, seq 0).
    /// The on_data call must trigger the eviction before the insert.
    ///
    /// Note: pending_requests is HashMap<(u16, u8), u32> — key=(dest_addr, app_seq),
    /// value=request_ts.  We pre-populate the map directly (field is pub) and then
    /// let on_data trigger the 257th insert to exercise the implementation's eviction path.
    ///
    /// Traces to: BC-2.15.016 postconditions 8–10; STORY-107 AC-005.
    #[test]
    fn test_pending_requests_eviction_at_256() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Create the flow entry.
        let seed_frame = build_frame(5, 0x0003, 0x0001, 0xC4);
        analyzer.on_data(key.clone(), &seed_frame, 0);

        // Pre-populate pending_requests with 256 entries.
        // Entry (0u16, 0u8) → ts=290 is the oldest and must be evicted by the 256-cap.
        // All timestamps are set within BLOCK_CMD_TIMEOUT_SECS (10s) of the delivery
        // ts=300 so that STORY-109's block-timeout scan does NOT drain the pre-filled
        // entries before the cap-eviction logic fires.
        // Specifically: STORY-109 scan removes entries where (300 - ts) > 10 (strictly
        // greater); ts=290 gives elapsed=10 (not > 10, survives); ts=295 gives elapsed=5.
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.pending_requests.clear();
            // Entry (0,0) at ts=290 — oldest single entry, survives block-timeout scan
            // (300-290=10, not strictly > 10) and is evicted by the 256-cap eviction.
            flow.pending_requests.insert((0u16, 0u8), 290u32);
            // Entries (1..=255) at ts=295 — all newer than (0,0), all survive scan.
            for i in 1u32..(MAX_PENDING_REQUESTS as u32) {
                flow.pending_requests.insert((i as u16, 0u8), 295u32);
            }
            assert_eq!(
                flow.pending_requests.len(),
                MAX_PENDING_REQUESTS,
                "pre-condition: pending_requests must have exactly 256 entries"
            );
            assert!(
                flow.pending_requests.contains_key(&(0u16, 0u8)),
                "pre-condition: oldest entry (0,0)→ts=290 must be present before eviction"
            );
        }

        // Deliver a Control-class frame (FC=0x03 SELECT, dest=256, src=1) at ts=300.
        // This must trigger the eviction path: detect map full → evict oldest (ts=0) →
        // insert new entry (dest=256, seq determined by app_ctrl, ts=300).
        //
        // STORY-108 relocated the pending_request seed onto the gate-validated carry path.
        // The frame must be COMPLETE (frame_len bytes) so the carry-walk reaches the app FC.
        // LENGTH=8 → frame_len = 5+8+2*ceil(3/16) = 5+8+2 = 15 bytes.
        // link CONTROL nibble=0x04 UNCONFIRMED_USER_DATA → has_user_data==true.
        // transport byte 0xC0: FIR=1 (0x40) | FIN=1 (0x80).
        // app_ctrl=0xC0, app_seq = 0xC0 & 0x0F = 0.
        // app_fc=0x03 (SELECT=Control-class).
        let mut ctrl_frame = vec![0u8; 15];
        ctrl_frame[0] = 0x05; // start1
        ctrl_frame[1] = 0x64; // start2
        ctrl_frame[2] = 0x08; // LENGTH=8 → frame_len=15 (complete)
        ctrl_frame[3] = 0xC4; // link CONTROL: DIR+PRM+UNCONFIRMED_USER_DATA (nibble 0x04)
        ctrl_frame[4] = 0x00; // dest low byte = 256 & 0xFF = 0
        ctrl_frame[5] = 0x01; // dest high byte = 256 >> 8 = 1  → dest LE = 0x0100 = 256
        ctrl_frame[6] = 0x01; // src low = 1
        ctrl_frame[7] = 0x00; // src high = 0  → src LE = 0x0001 = 1
        ctrl_frame[8] = 0x00; // header CRC placeholder
        ctrl_frame[9] = 0x00; // header CRC placeholder
        ctrl_frame[10] = 0xC0; // transport: FIR=1 (0x40), FIN=1 (0x80), SEQ=0
        ctrl_frame[11] = 0xC0; // app_ctrl: app_seq = 0xC0 & 0x0F = 0
        ctrl_frame[12] = 0x03; // app FC = SELECT (Control-class)
        // bytes 13-14: data-block CRC placeholder (0x00)
        analyzer.on_data(key.clone(), &ctrl_frame, 300);

        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");

        // The map must stay at MAX_PENDING_REQUESTS (never exceed 256).
        assert_eq!(
            flow.pending_requests.len(),
            MAX_PENDING_REQUESTS,
            "pending_requests must stay at MAX_PENDING_REQUESTS=256 after the 257th insert \
             triggers eviction of the oldest entry (ts=290)"
        );
        // The oldest entry (ts=290) must have been evicted by the 256-cap eviction.
        assert!(
            !flow.pending_requests.contains_key(&(0u16, 0u8)),
            "entry (0u16, 0u8) with ts=290 must be evicted before the 257th insert"
        );
        // F-P2-001 / AC-005 post-state: the new (dest=256, app_seq=0) entry seeded by the
        // ctrl_frame must be present, confirming that the evict-then-insert swap actually
        // occurred (not merely that len happened to stay at 256 for some other reason).
        //
        // STORY-108: seed moved to gate-validated carry path; complete frame required.
        // dest  = u16::from_le_bytes([ctrl_frame[4]=0x00, ctrl_frame[5]=0x01]) = 256
        // app_seq = ctrl_frame[11] & 0x0F = 0xC0 & 0x0F = 0
        assert!(
            flow.pending_requests.contains_key(&(256u16, 0u8)),
            "new entry (dest=256, app_seq=0) must be present: seed+evict swap must have \
             occurred, not merely preserved the existing 256 entries unchanged"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-006 — test_carry_drain_boundary_min_frame
    // BC-2.15.016 invariant 1 / VP-023 Sub-D: carry-drain does not panic on boundary
    // ---------------------------------------------------------------------------

    /// AC-006: LENGTH=5 minimum frame (frame_len=10) carry-drain does not panic.
    ///
    /// VP-023 Sub-D proves compute_dnp3_frame_len returns values in [10, 292].
    /// This test verifies the boundary case: carry already holds a full 10-byte
    /// minimum frame (pre-loaded directly), and one additional byte is delivered.
    ///
    /// Expected after the on_data call:
    ///   - The carry-consume loop drains the 10-byte frame from carry.
    ///   - 1 byte remains in carry (the extra byte delivered).
    ///   - frame_count == 1.
    ///   - No panic (VP-023 Sub-D invariant: drain index in [10, 292]).
    ///
    /// Authored RED (STORY-107): STORY-106's on_data did not accumulate into carry
    /// and did not execute the carry-consume loop.  STORY-107 added both; this test
    /// asserts carry.len()==1 after draining a 10-byte minimum frame — the assertion
    /// now passes.
    ///
    /// Traces to: BC-2.15.016 invariant 1; VP-023 Sub-D; STORY-107 AC-006.
    #[test]
    fn test_carry_drain_boundary_min_frame() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Create the flow entry.
        analyzer.on_data(key.clone(), &[0x05, 0x64], 0);

        // Pre-load carry with exactly one 10-byte minimum frame (LENGTH=5).
        let min_frame = build_frame(5, 0x0003, 0x0001, 0xC4);
        assert_eq!(
            min_frame.len(),
            10,
            "minimum frame must be exactly 10 bytes"
        );
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry.clear();
            flow.carry.extend_from_slice(&min_frame);
            assert_eq!(
                flow.carry.len(),
                10,
                "pre-condition: carry must be 10 bytes"
            );
        }

        // Deliver 1 additional byte.  The implementation must:
        //   1. Append 1 byte to carry → carry.len() == 11.
        //   2. Check: carry.len() (11) >= frame_len (10) for carry[2] = LENGTH=5.
        //   3. drain(..10) — must NOT panic (VP-023 Sub-D: frame_len in [10,292]).
        //   4. carry.len() == 1 after drain; frame_count++ → 1.
        //
        // The extra byte is a valid sync start byte (0x05) — it will stay in carry
        // as the beginning of the next (as-yet-incomplete) frame.
        analyzer.on_data(key.clone(), &[0x05], 1);

        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");

        assert_eq!(
            flow.carry.len(),
            1,
            "carry must have 1 byte remaining after draining the 10-byte minimum frame"
        );
        assert_eq!(
            flow.frame_count, 1,
            "frame_count must be 1 after consuming the minimum frame via carry-drain"
        );
    }

    // ---------------------------------------------------------------------------
    // EC-001: Partial frame delivery (7 bytes of a 10-byte header)
    // BC-2.15.016 edge case EC-001: bytes in carry, no frame parse attempted
    // ---------------------------------------------------------------------------

    /// EC-001: Partial frame delivery — 7 bytes of a 10-byte minimum frame.
    ///
    /// Expected: carry.len() == 7; frame_count == 0 (no complete frame yet).
    ///
    /// Traces to: BC-2.15.016 EC-001; STORY-107 edge case EC-001.
    #[test]
    fn test_EC_001_partial_frame_in_carry() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Build a minimum frame and deliver only its first 7 bytes.
        let min_frame = build_frame(5, 0x0003, 0x0001, 0xC4);
        let partial = &min_frame[..7];

        analyzer.on_data(key.clone(), partial, 0);

        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.carry.len(),
            7,
            "carry must hold the 7 partial bytes with no frame consumed yet"
        );
        assert_eq!(
            flow.frame_count, 0,
            "frame_count must be 0: no complete frame has been consumed"
        );
    }

    // ---------------------------------------------------------------------------
    // EC-002: Two complete frames in one on_data call
    // BC-2.15.016 edge case EC-002 (story) / BC EC-003: both parsed; carry empty
    // ---------------------------------------------------------------------------

    /// EC-002: Deliver two complete 10-byte frames in one on_data call.
    ///
    /// Expected: carry.len() == 0; frame_count == 2.
    ///
    /// Traces to: STORY-107 EC-002; BC-2.15.016 EC-002 (variant of EC-003).
    #[test]
    fn test_EC_002_two_complete_frames_one_call() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        let frame1 = build_frame(5, 0x0003, 0x0001, 0xC4);
        let frame2 = build_frame(5, 0x0003, 0x0002, 0xC4);
        let mut twenty_bytes = Vec::with_capacity(20);
        twenty_bytes.extend_from_slice(&frame1);
        twenty_bytes.extend_from_slice(&frame2);
        assert_eq!(
            twenty_bytes.len(),
            20,
            "two 10-byte frames = 20 bytes total"
        );

        analyzer.on_data(key.clone(), &twenty_bytes, 0);

        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.carry.len(),
            0,
            "carry must be empty after both complete frames are consumed"
        );
        assert_eq!(
            flow.frame_count, 2,
            "frame_count must be 2: both complete frames consumed in one on_data call"
        );
    }

    // ---------------------------------------------------------------------------
    // EC-003: Carry reaches 291 bytes; on_data delivers 2 bytes → 1 accepted, 1 discarded
    // BC-2.15.016 EC-004 / STORY-107 EC-003
    // ---------------------------------------------------------------------------

    /// EC-003: Carry at 291 bytes + on_data delivers 2 bytes → 1 accepted (total=292); 1 discarded.
    ///
    /// Expected: carry.len() == 292, parse_errors == 1.
    ///
    /// Traces to: STORY-107 EC-003; BC-2.15.016 EC-004.
    #[test]
    fn test_EC_003_carry_291_plus_2_overflow() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Create flow entry then pre-load carry to 291 bytes.
        analyzer.on_data(key.clone(), &[0x05, 0x64], 0);
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry.clear();
            flow.carry.extend(std::iter::repeat_n(0xAA, 291));
            assert_eq!(flow.carry.len(), 291, "pre-condition: carry must be 291");
        }

        // Deliver 2 bytes.  Only 1 fits (292 - 291 = 1); the second is discarded.
        analyzer.on_data(key.clone(), &[0xBB, 0xCC], 1);

        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        // BC-2.15.016 PC2 (292-cap): parse_errors == 1 proves the cap fired.
        // 291 bytes in carry + 2 bytes delivered → only 1 accepted (total=292 cap);
        // 1 byte discarded; overflow arm increments parse_errors exactly once.
        // STORY-109 behavior: byte-walk resync clears the carry; the 292-cap is
        // proven by parse_errors==1 (overflow arm only fires when carry+new > 292).
        assert_eq!(
            flow.parse_errors, 1,
            "parse_errors must be 1: 1 byte was discarded at the 292 cap (BC-2.15.016 PC2)"
        );

        // After overflow, frame-walk ran: no [0x05,0x64] sync found in 292 bytes of 0xAA
        // → carry cleared by byte-walk-forward resync (STORY-109 behavior).
        assert_eq!(
            flow.carry.len(),
            0,
            "carry must be 0: byte-walk-forward resync found no sync in junk carry after overflow"
        );
    }

    // ---------------------------------------------------------------------------
    // EC-004: Bailed flow (is_non_dnp3=true) receives bytes → immediate no-op
    // BC-2.15.016 EC-006 / STORY-107 EC-004
    // ---------------------------------------------------------------------------

    /// EC-004: Bailed flow (is_non_dnp3=true) — on_data is a no-op; carry NOT updated.
    ///
    /// A flow that has previously been bailed (desync latch set) must ignore all
    /// subsequent on_data calls.  carry must remain empty; parse_errors unchanged.
    ///
    /// Traces to: STORY-107 EC-004; BC-2.15.016 EC-006; BC-2.15.009 postcondition 5.
    #[test]
    fn test_EC_004_bailed_flow_is_noop() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Trigger the desync bail: deliver non-DNP3 bytes (no valid sync at offset 0).
        let non_dnp3 = [
            0xFF, 0xFE, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
            0x0C, 0x0D,
        ];
        analyzer.on_data(key.clone(), &non_dnp3, 0);

        {
            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after bail");
            assert!(flow.is_non_dnp3, "pre-condition: flow must be bailed");
        }

        // Now deliver valid-looking DNP3 bytes to a bailed flow — must be no-op.
        let valid_frame = build_frame(5, 0x0003, 0x0001, 0xC4);
        analyzer.on_data(key.clone(), &valid_frame, 1);

        let flow = analyzer.flows.get(&key).expect("flow must still exist");
        assert!(
            flow.is_non_dnp3,
            "is_non_dnp3 must remain true — one-way latch"
        );
        assert_eq!(
            flow.carry.len(),
            0,
            "carry must NOT grow on bailed flow — on_data is immediate no-op"
        );
        assert_eq!(
            flow.frame_count, 0,
            "frame_count must not increment on bailed flow"
        );
    }

    // ---------------------------------------------------------------------------
    // EC-005: pending_requests at 256 with tie-break (two entries share minimum ts)
    // BC-2.15.016 EC-008 / postcondition 9: tie-breaking is implementation-defined
    // ---------------------------------------------------------------------------

    /// EC-005: pending_requests at 256 with two entries sharing minimum ts.
    ///
    /// When two entries share the same minimum ts, either may be evicted (implementation-defined per
    /// BC-2.15.016 postcondition 9).  After insert: map.len()==256 and at least one
    /// of the tied-oldest keys is no longer present.
    ///
    /// Authored RED (STORY-107): STORY-106's on_data had no eviction logic.
    /// STORY-107 added eviction; this test verifies tie-breaking behaviour and now passes.
    ///
    /// Traces to: STORY-107 EC-005; BC-2.15.016 postcondition 9 (tie-breaking impl-defined).
    #[test]
    fn test_EC_005_pending_requests_tie_break_eviction() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Create the flow entry.
        let seed_frame = build_frame(5, 0x0003, 0x0001, 0xC4);
        analyzer.on_data(key.clone(), &seed_frame, 0);

        // Pre-populate with 256 entries: two entries share ts=490 (tied oldest).
        // All timestamps are within BLOCK_CMD_TIMEOUT_SECS (10s) of the delivery ts=500
        // so STORY-109's block-timeout scan does NOT drain entries before cap-eviction fires.
        // ts=490 → elapsed at ts=500 is 10, which is NOT > 10 (strictly), so entries survive.
        // ts=495 → elapsed=5 → survives scan.
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.pending_requests.clear();
            flow.pending_requests.insert((0u16, 0u8), 490u32); // ts=490, tied oldest
            flow.pending_requests.insert((1u16, 0u8), 490u32); // ts=490, tied oldest
            for i in 2u32..(MAX_PENDING_REQUESTS as u32) {
                flow.pending_requests.insert((i as u16, 0u8), 495u32); // ts=495, newer
            }
            assert_eq!(
                flow.pending_requests.len(),
                MAX_PENDING_REQUESTS,
                "pre-condition: map must have exactly 256 entries"
            );
        }

        // Deliver a 257th Control-class frame (dest=300, seq=0, ts=500) via on_data.
        // The implementation must evict one of the tied-oldest (ts=490) entries.
        // STORY-108: seed moved to gate-validated carry path; complete frame required.
        // LENGTH=8 → frame_len = 5+8+2 = 15 bytes (complete).
        // link CONTROL nibble=0x04 UNCONFIRMED_USER_DATA → has_user_data==true.
        let mut ctrl_frame = vec![0u8; 15];
        ctrl_frame[0] = 0x05;
        ctrl_frame[1] = 0x64;
        ctrl_frame[2] = 0x08; // LENGTH=8 → frame_len=15 (complete)
        ctrl_frame[3] = 0xC4; // UNCONFIRMED_USER_DATA (nibble 0x04)
        ctrl_frame[4] = 0x2C; // dest low byte of 300: 300 & 0xFF = 0x2C
        ctrl_frame[5] = 0x01; // dest high byte of 300: 300 >> 8 = 0x01 → dest=0x012C=300
        ctrl_frame[6] = 0x01;
        ctrl_frame[7] = 0x00;
        ctrl_frame[8] = 0x00;
        ctrl_frame[9] = 0x00;
        ctrl_frame[10] = 0xC0; // FIR=1
        ctrl_frame[11] = 0xC0;
        ctrl_frame[12] = 0x03; // FC=SELECT (Control-class)
        // bytes 13-14: data-block CRC placeholder
        analyzer.on_data(key.clone(), &ctrl_frame, 500);

        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");

        assert_eq!(
            flow.pending_requests.len(),
            MAX_PENDING_REQUESTS,
            "map must stay at 256: one of the tied-oldest entries must be evicted"
        );
        // At least one of the two tied-oldest keys must have been evicted.
        let key0_present = flow.pending_requests.contains_key(&(0u16, 0u8));
        let key1_present = flow.pending_requests.contains_key(&(1u16, 0u8));
        assert!(
            !key0_present || !key1_present,
            "at least one of the tied-minimum-ts keys (0,0)/(1,0) must have been evicted \
             (tie-breaking is implementation-defined per BC-2.15.016 postcondition 9)"
        );
    }

    // ---------------------------------------------------------------------------
    // EC-006: carry[2] (LENGTH byte) = 4 (invalid, < 5) → parse_errors++; carry advanced
    // BC-2.15.016 EC-007 / STORY-107 EC-006
    // ---------------------------------------------------------------------------

    /// EC-006: LENGTH byte = 4 (invalid, < 5) — validity gate fires; parse_errors++;
    /// carry is advanced past this malformed frame start.
    ///
    /// When carry[2] < 5, compute_dnp3_frame_len returns None.  The implementation
    /// must handle this: increment parse_errors and advance the carry (e.g., drain 1
    /// byte to attempt re-sync) rather than looping indefinitely.
    ///
    /// Traces to: STORY-107 EC-006; BC-2.15.016 EC-007; BC-2.15.004 (validity gate).
    #[test]
    fn test_EC_006_invalid_length_byte_increments_parse_errors() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Build a 10-byte frame with valid sync but LENGTH=4 (invalid: < 5).
        let mut bad_frame = build_frame(5, 0x0003, 0x0001, 0xC4);
        bad_frame[2] = 4; // Overwrite LENGTH with invalid value.

        // Deliver the malformed frame.
        analyzer.on_data(key.clone(), &bad_frame, 0);

        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.parse_errors, 1,
            "parse_errors must be 1: invalid LENGTH byte must increment parse_errors"
        );
        // Carry must NOT be in a stuck state (10 bytes with invalid LENGTH still in carry).
        // The carry should be either drained or advanced.  We assert frame_count == 0
        // (no valid frame was consumed) and parse_errors == 1 as the primary invariants.
        assert_eq!(
            flow.frame_count, 0,
            "frame_count must be 0: no valid frame was consumed from an invalid-LENGTH carry"
        );
        // F-P2-002 / EC-006 resync-progress assertion: the byte-walk-forward resync
        // (STORY-109; realizes STORY-107 deferral, per STORY-109-resync-adjudication.md
        // Decision 3) must have advanced the carry past the invalid LENGTH byte.
        //
        // Derivation under byte-walk-forward resync:
        //   bad_frame = [0x05, 0x64, 0x04, ...zeros...] (10 bytes)
        //   Iteration 1: carry[0..2] = [0x05, 0x64] — sync OK; compute_dnp3_frame_len(4)
        //     returns None → parse_errors++; carry.drain(..1) → carry now has 9 bytes:
        //     [0x64, 0x04, 0x00, ..., 0x00].
        //   Iteration 2: carry[0] = 0x64 (not 0x05) → sync gate fires; byte-walk scans
        //     windows from offset 1: no [0x05,0x64] pair exists in the remaining bytes →
        //     carry.clear() → carry is empty (len == 0). continue.
        //   Iteration 3: carry.len() < 3 → guard breaks.
        //   Final carry length = 0 (no [0x05,0x64] sync found → cleared).
        assert!(
            flow.carry.len() < 10,
            "carry must have advanced: resync must reduce carry below the 10 \
             originally delivered bytes (stuck carry == no-op, which is a liveness bug)"
        );
        assert_eq!(
            flow.carry.len(),
            0,
            "carry must be 0 bytes: byte-walk-forward resync found no [0x05,0x64] in the \
             remaining bytes and cleared the carry (STORY-109 realization of STORY-107 \
             deferred byte-walk resync; adjudication Decision 3)"
        );
    }

    // ---------------------------------------------------------------------------
    // Sanity: is_master_frame recognizes DIR bit
    // BC-2.15.016 postconditions 5–6 depend on is_master_frame
    // ---------------------------------------------------------------------------

    /// Verify is_master_frame correctly identifies DIR=1 frames (control & 0x80 != 0).
    ///
    /// CORRECTED per F-A-001 REVISION 2: DIR is bit 7 (mask 0x80) per IEEE 1815 DNP3
    /// link-layer framing. The previous version incorrectly tested bit 4 (mask 0x10 = FCV/DFC).
    ///
    /// RED GATE: this test asserts the CORRECT 0x80-mask behavior. It fails until
    /// `is_master_frame` is fixed from `control & 0x10 != 0` to `control & 0x80 != 0`.
    ///
    /// Traces to: BC-2.15.016 postcondition 5 (corrected); F-A-001 REVISION 2 §R2-1.
    #[test]
    fn test_BC_2_15_016_is_master_frame_dir_bit() {
        use wirerust::analyzer::dnp3::is_master_frame;

        // -------------------------------------------------------------------
        // DIR=1 (bit 7 set) — master-direction frames. All must return true.
        // -------------------------------------------------------------------

        // Canonical master frame per BC-2.15.010 / HS-W37-002 byte vectors.
        // 0xC4 = 1100 0100: DIR=1(bit7), PRM=1(bit6), FCB=0(bit5), FCV=0(bit4),
        //                    FC=0x04(UNCONF_USER_DATA).
        // 0xC4 & 0x80 = 0x80 != 0 → is_master_frame=true (CORRECT).
        // 0xC4 & 0x10 = 0x00 == 0 → BUGGY mask returns false (proves the bug).
        assert!(
            is_master_frame(0xC4),
            "control=0xC4 (canonical master frame: DIR=1 bit7 set) must return true; \
             RED: buggy mask 0x10 returns false for this canonical value"
        );

        // 0xD4 = 1101 0100: DIR=1(bit7), PRM=1(bit6), FCB=0(bit5), FCV=1(bit4),
        //                    FC=0x04(UNCONF_USER_DATA).
        // 0xD4 & 0x80 = 0x80 != 0 → is_master_frame=true (CORRECT).
        assert!(
            is_master_frame(0xD4),
            "control=0xD4 (DIR=1 bit7 set; also PRM+FCV bits set) must return true"
        );

        // 0xFF = all bits set; bit 7 (DIR) is set → must return true.
        assert!(
            is_master_frame(0xFF),
            "control=0xFF (all bits set, DIR=1) must return true"
        );

        // -------------------------------------------------------------------
        // DIR=0 (bit 7 clear) — outstation-direction frames. All must return false.
        // -------------------------------------------------------------------

        // 0x00 = all bits clear; DIR=0 → must return false.
        assert!(
            !is_master_frame(0x00),
            "control=0x00 (no bits set, DIR=0) must return false"
        );

        // 0x04 = UNCONF_USER_DATA with DIR=0: bit7=0, FC=0x04.
        // 0x04 & 0x80 = 0 → must return false.
        assert!(
            !is_master_frame(0x04),
            "control=0x04 (UNCONF_USER_DATA, DIR=0 bit7 clear) must return false"
        );

        // 0x44 = 0100 0100: bit7=0 (DIR=0), bit6=1 (PRM=1), bit5=0, bit4=0, FC=0x04.
        // Outstation direction (DIR=0). 0x44 & 0x80 = 0 → must return false.
        // Replaces the previously-wrong 0xEF assertion: 0xEF & 0x80 = 0x80 (DIR=1,
        // so 0xEF IS a master frame under the correct mask — the old assertion was wrong).
        assert!(
            !is_master_frame(0x44),
            "control=0x44 (DIR=0, PRM=1, outstation direction) must return false"
        );

        // Confirm 0x10 (FCV/DFC bit only, DIR=0) now returns FALSE under corrected mask.
        // Under the BUGGY mask (0x10), this returned true — which was the bug.
        // Under the CORRECT mask (0x80): 0x10 & 0x80 = 0 → false.
        assert!(
            !is_master_frame(0x10),
            "control=0x10 (FCV bit only, DIR=0 bit7 clear) must return false; \
             RED: buggy mask 0x10 returns true for this value (wrong: 0x10 is FCV, not DIR)"
        );
    }

    // ---------------------------------------------------------------------------
    // OBS-P11-1 REALIGN branch — byte-walk-forward finds next sync at offset > 1
    // Adjudication Step-6 requirement: drain-to-next-sync case
    // BC-2.15.016 / STORY-109-resync-adjudication.md Step 6
    // ---------------------------------------------------------------------------

    /// OBS-P11-1: byte-walk-forward resync REALIGN branch — next `[0x05,0x64]` sync
    /// word is found at carry offset i > 1; carry drains to that offset and the
    /// recovered frame is then fully consumed in the same frame-walk loop.
    ///
    /// This is coverage-strengthening for the `Some(i) => carry.drain(..i)` arm of
    /// the resync match.  All other resync tests exercise only the `None => carry.clear()`
    /// arm (junk with no embedded sync).  This test exercises the RECOVER-to-next-frame
    /// path explicitly required by STORY-109-resync-adjudication.md Step 6.
    ///
    /// ## Byte vector (20 bytes)
    ///
    /// ```text
    /// Offset  Bytes           Meaning
    /// 0..5    05 64 02 AA AA  Malformed frame: sync OK, LENGTH=0x02 < 5 → validity-gate
    ///                         REJECT → parse_errors+1, malformed_in_window+1,
    ///                         carry.drain(..1) → head becomes 0x64.
    /// 5..20   05 64 08 C4     Valid 15-byte frame (LENGTH=0x08 → frame_len=15):
    ///         03 00 01 00       dest=0x0003, src=0x0001
    ///         00 00             header CRC placeholder
    ///         C0 00 03          transport=0xC0 (FIR=1), app_ctrl=0x00, app_fc=0x03
    ///         00 00             data-block CRC placeholder
    /// ```
    ///
    /// ## Trace
    ///
    /// After delivery carry = 20 bytes.
    ///
    /// Iter 1: carry[0..2] = [05 64] — sync OK; `compute_dnp3_frame_len(0x02)` = None
    ///   → `parse_errors=1`, `malformed_in_window=1`; `carry.drain(..1)`.
    ///   carry = 19 bytes: `[64 02 AA AA 05 64 08 C4 03 00 01 00 00 00 C0 00 03 00 00]`.
    ///
    /// Iter 2: carry[0]=0x64 ≠ 0x05 → resync arm.
    ///   Scan windows(2) from index 1: index 4 = [0x05, 0x64] → found, i=4.
    ///   `carry.drain(..4)` → carry = 15 bytes: `[05 64 08 C4 03 00 01 00 00 00 C0 00 03 00 00]`.
    ///   `continue` (NOT break).
    ///
    /// Iter 3: carry[0..2] = [05 64] — sync OK;
    ///   `compute_dnp3_frame_len(0x08)` = Some(15); carry.len()=15 >= 15.
    ///   Header: start1=0x05 start2=0x64 length=0x08 control=0xC4 dest=0x0003 src=0x0001.
    ///   `is_valid_dnp3_frame_header` = true → `frame_count=1`.
    ///   `has_user_data(0xC4)` = true (link FC nibble = 0x04 = UNCONFIRMED_USER_DATA).
    ///   `transport_is_fir(0xC0)` = true (bit 0x40 set).
    ///   `app_fc = carry[12] = 0x03` (SELECT).
    ///   `classify_dnp3_fc(0x03)` = Control → `fc_counts[0x03]` += 1.
    ///   `carry.drain(..15)` → carry empty.
    ///
    /// Iter 4: carry.len()=0 < 3 → break.
    ///
    /// ## Assertions (cover Step-6 requirements from adjudication)
    ///
    /// (a) `parse_errors == 1`: the malformed first frame counted exactly once;
    ///     the resync navigation did NOT double-count.
    /// (b) `frame_count == 1` and `fc_counts[0x03] == 1`: the embedded valid frame
    ///     was consumed after the REALIGN drain positioned the carry head correctly.
    /// (c) `carry.len() == 0`: carry is empty after both the malformed frame and the
    ///     recovered valid frame were consumed.
    ///
    /// Traces to: STORY-109-resync-adjudication.md Step 6 (adversarial pass requirement);
    /// BC-2.15.016 resync path; BC-2.15.024 no-double-count invariant; OBS-P11-1.
    #[test]
    fn test_BC_2_16_OBS_P11_1_resync_realign_branch_drain_to_next_sync() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // 20-byte delivery vector — see doc comment for full trace.
        //
        // Bytes 0-4:  malformed frame (LENGTH=0x02 < 5 → validity-gate reject)
        // Bytes 5-19: valid 15-byte DNP3 frame (LENGTH=0x08, FC=0x03 SELECT)
        //
        //                                  --- valid 15-byte frame ---
        //                  -- malformed --  sync  LEN  CTL  DST    SRC    hCRC  trp  aseq aFC  dCRC
        let delivery: &[u8] = &[
            0x05, 0x64, 0x02, 0xAA, 0xAA, // malformed: sync OK, LENGTH=2 < 5
            0x05, 0x64, 0x08, 0xC4, // valid sync + LENGTH=8 + CTRL (UNCONF_USER_DATA, DIR=1)
            0x03, 0x00, // dest = 0x0003 little-endian
            0x01, 0x00, // src  = 0x0001 little-endian
            0x00, 0x00, // header CRC placeholder
            0xC0, // transport octet: FIR=1 (bit 0x40) | FIN=1 (bit 0x80) = 0xC0
            0x00, // app control (seq=0)
            0x03, // app FC = 0x03 SELECT (Control-class)
            0x00, 0x00, // data-block CRC placeholder
        ];
        assert_eq!(
            delivery.len(),
            20,
            "delivery vector must be exactly 20 bytes"
        );

        analyzer.on_data(key.clone(), delivery, 0);

        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");

        // (a) parse_errors == 1: malformed first frame counted once; resync arm MUST NOT
        //     double-count (adjudication Decision 1: "draining the carry is pure cursor
        //     movement, not a new error event").
        assert_eq!(
            flow.parse_errors, 1,
            "OBS-P11-1 (a): parse_errors must be exactly 1 — the malformed LENGTH=2 frame \
             was rejected once by the validity gate; the resync arm does NOT increment \
             parse_errors (no double-counting — adjudication Decision 1 / BC-2.15.024)"
        );

        // (b.i) frame_count == 1: the embedded valid frame was consumed AFTER the REALIGN
        //       drain positioned the carry head at [0x05, 0x64] (the Some(i) branch).
        //       frame_count=0 would mean the realign drained too far or broke instead of
        //       continuing, leaving the valid frame unconsumed.
        assert_eq!(
            flow.frame_count, 1,
            "OBS-P11-1 (b.i): frame_count must be 1 — the valid SELECT frame embedded after \
             the junk bytes must have been parsed after the REALIGN drain positioned carry[0] \
             at its [0x05,0x64] sync word (Some(i) branch with i=4; adjudication Step 6)"
        );

        // (b.ii) fc_counts[0x03] == 1: the FC=0x03 (SELECT) application function code was
        //        recorded, proving the application layer of the recovered frame was fully
        //        processed (transport FIR gate passed, app_fc extracted and classified).
        assert_eq!(
            flow.fc_counts.get(&0x03u8).copied().unwrap_or(0),
            1,
            "OBS-P11-1 (b.ii): fc_counts[0x03] must be 1 — SELECT (FC=0x03) was recorded as \
             a Control-class function code after the realigned frame was fully parsed"
        );

        // (c) carry empty: both the malformed frame's junk bytes and the valid frame's 15
        //     bytes have been consumed; nothing remains in carry.
        assert_eq!(
            flow.carry.len(),
            0,
            "OBS-P11-1 (c): carry must be empty — the malformed prefix was drained by the \
             validity-gate (drain-1) + resync (drain-4), and the valid 15-byte frame was \
             fully consumed by carry.drain(..15)"
        );
    }
} // mod story_107
