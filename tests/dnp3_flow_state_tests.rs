//! Failing tests for STORY-107: DNP3 Per-Flow State + Carry Buffer + Pending-Request Bounds.
//!
//! Covers BC-2.15.016 AC-001..AC-006 and edge cases EC-001..EC-006.
//! All tests MUST FAIL before implementation — Red Gate per the strict-TDD contract
//! (STORY-107, tdd_mode: strict).
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

    use wirerust::analyzer::dnp3::{
        Dnp3Analyzer, MAX_DNP3_FRAME_LEN, MAX_MASTER_ADDRS, MAX_PENDING_REQUESTS,
    };
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

    /// Build a master-direction frame: control has DIR bit set (0x10).
    /// Uses nibble 0x04 (UNCONFIRMED_USER_DATA) with DIR+PRM bits: 0xD4.
    fn build_master_frame(dest: u16, src: u16) -> Vec<u8> {
        // 0xD4 = 1101 0100: DIR(1) PRM(1) FCB(0) FCV(0) FC(0100=UNCONF_USER_DATA)
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

        assert_eq!(
            flow.carry.len(),
            MAX_DNP3_FRAME_LEN,
            "carry.len() must be capped at MAX_DNP3_FRAME_LEN=292 after overflow"
        );
        assert_eq!(
            flow.parse_errors, 1,
            "parse_errors must be 1: carry overflow increments the lifetime parse_errors counter"
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
    /// Red Gate: on_data has NO eviction logic in STORY-106.  After delivering the 257th
    /// frame, the map will either stay at 256 (if STORY-107 eviction is implemented) or
    /// grow to 257 (if not).  This test asserts len==256 AND oldest evicted — both will
    /// FAIL until the implementation is complete.
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

        // Pre-populate pending_requests with 256 entries, ts 0..=255.
        // Entry (0u16, 0u8) → ts=0 is the oldest and must be evicted.
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.pending_requests.clear();
            for i in 0u32..(MAX_PENDING_REQUESTS as u32) {
                flow.pending_requests.insert((i as u16, 0u8), i);
            }
            assert_eq!(
                flow.pending_requests.len(),
                MAX_PENDING_REQUESTS,
                "pre-condition: pending_requests must have exactly 256 entries"
            );
            assert!(
                flow.pending_requests.contains_key(&(0u16, 0u8)),
                "pre-condition: oldest entry (0,0)→ts=0 must be present before eviction"
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
             triggers eviction of the oldest entry (ts=0)"
        );
        // The oldest entry (ts=0) must have been evicted.
        assert!(
            !flow.pending_requests.contains_key(&(0u16, 0u8)),
            "entry (0u16, 0u8) with ts=0 must be evicted before the 257th insert"
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
    /// Red Gate: STORY-106's on_data does not accumulate into carry and does not
    /// execute the carry-consume loop.  carry.len() will be 0 (not 1) and
    /// frame_count will reflect STORY-106 counting (not carry-consume counting).
    /// The assertion on carry.len()==1 will FAIL until implementation is complete.
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
        assert_eq!(
            flow.carry.len(),
            MAX_DNP3_FRAME_LEN,
            "carry must be capped at 292 after accepting 1 of 2 bytes"
        );
        assert_eq!(
            flow.parse_errors, 1,
            "parse_errors must be 1: 1 byte was discarded (overflow)"
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
    /// When two entries share ts=0, either may be evicted (implementation-defined per
    /// BC-2.15.016 postcondition 9).  After insert: map.len()==256 and at least one
    /// of the tied-oldest keys is no longer present.
    ///
    /// Red Gate: on_data has NO eviction logic in STORY-106.  This test will FAIL
    /// until the STORY-107 eviction implementation is in place.
    ///
    /// Traces to: STORY-107 EC-005; BC-2.15.016 postcondition 9 (tie-breaking impl-defined).
    #[test]
    fn test_EC_005_pending_requests_tie_break_eviction() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Create the flow entry.
        let seed_frame = build_frame(5, 0x0003, 0x0001, 0xC4);
        analyzer.on_data(key.clone(), &seed_frame, 0);

        // Pre-populate with 256 entries: two entries share ts=0 (tied oldest).
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.pending_requests.clear();
            flow.pending_requests.insert((0u16, 0u8), 0u32); // ts=0, tied oldest
            flow.pending_requests.insert((1u16, 0u8), 0u32); // ts=0, tied oldest
            for i in 2u32..(MAX_PENDING_REQUESTS as u32) {
                flow.pending_requests.insert((i as u16, 0u8), i - 1); // ts=1..=254
            }
            assert_eq!(
                flow.pending_requests.len(),
                MAX_PENDING_REQUESTS,
                "pre-condition: map must have exactly 256 entries"
            );
        }

        // Deliver a 257th Control-class frame (dest=300, seq=0, ts=500) via on_data.
        // The implementation must evict one of the tied-oldest (ts=0) entries.
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
        // F-P2-002 / EC-006 resync-progress assertion: the drain-1 resync policy must have
        // advanced the carry past the invalid LENGTH byte.  The carry must be SHORTER than
        // the 10 bytes originally delivered — proving the drain-1 resync occurred and the
        // implementation did not simply break without advancing (a no-op stuck-state).
        //
        // Derivation:
        //   bad_frame = [0x05, 0x64, 0x04, ...zeros...] (10 bytes)
        //   Iteration 1: carry[0..2] = [0x05, 0x64] — sync OK; compute_dnp3_frame_len(4)
        //     returns None → parse_errors++; carry.drain(..1) → carry now has 9 bytes.
        //   Iteration 2: carry[0] = 0x64 (not 0x05) → sync break.
        //   Final carry length = 9 (the 9 bytes starting from original index 1).
        assert!(
            flow.carry.len() < 10,
            "carry must have advanced: drain-1 resync must reduce carry below the 10 \
             originally delivered bytes (stuck carry == no-op, which is a liveness bug)"
        );
        assert_eq!(
            flow.carry.len(),
            9,
            "carry must be exactly 9 bytes: drain-1 resync consumed 1 byte, then the sync \
             gate broke on carry[0]=0x64, leaving bytes [1..9] of the original delivery"
        );
    }

    // ---------------------------------------------------------------------------
    // Sanity: is_master_frame recognizes DIR bit
    // BC-2.15.016 postconditions 5–6 depend on is_master_frame
    // ---------------------------------------------------------------------------

    /// Verify is_master_frame correctly identifies DIR=1 frames (control & 0x10 != 0).
    ///
    /// This is a direct unit test of the helper used by master-addr tracking.
    /// `is_master_frame` currently has `todo!()` — this test will panic until implemented.
    ///
    /// Traces to: BC-2.15.016 postconditions 5–6; STORY-107 Task 5.
    #[test]
    fn test_BC_2_15_016_is_master_frame_dir_bit() {
        use wirerust::analyzer::dnp3::is_master_frame;

        // DIR bit set (0x10) — master-direction frame.
        assert!(
            is_master_frame(0x10),
            "control=0x10 (DIR bit only) must return true"
        );
        assert!(
            is_master_frame(0xD4),
            "control=0xD4 (DIR+PRM+UNCONF_USER_DATA) must return true"
        );
        assert!(
            is_master_frame(0xFF),
            "control=0xFF (all bits set) must return true"
        );

        // DIR bit clear — outstation-direction frame.
        assert!(
            !is_master_frame(0x00),
            "control=0x00 (no DIR bit) must return false"
        );
        assert!(
            !is_master_frame(0x04),
            "control=0x04 (UNCONF_USER_DATA, no DIR) must return false"
        );
        assert!(
            !is_master_frame(0xEF),
            "control=0xEF (DIR bit clear: 0xEF & 0x10 == 0) must return false"
        );
    }
} // mod story_107
