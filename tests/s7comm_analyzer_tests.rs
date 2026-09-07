//! Tests for STORY-186: S7comm ISO-on-TCP Carry-Buffer Reassembly, Walk-First Frame
//! Extraction, Resync, and the Frozen SS-20/SS-21 Module Boundary.
//!
//! Covers BC-2.20.013, BC-2.20.014, BC-2.20.015, BC-2.20.016, BC-2.21.003, and the
//! VP-050 proptest obligation (walk-first residual bound, direction isolation, 1-byte
//! resync advance).
//!
//! ## Contract coverage
//! - BC-2.20.013: TPKT frames spanning TCP segment boundaries are reassembled via
//!   directional carry buffers using walk-first, residual-bound semantics. No aggregate
//!   `carry.len() + incoming.len()` pre-check exists anywhere.
//! - BC-2.20.014: carry buffer bounded at `MAX_S7_ISO_ON_TCP_CARRY_BYTES = 65,535`;
//!   overflow triggers clear-and-resync with exactly one T0814 per direction, guarded by
//!   a per-direction dedup flag.
//! - BC-2.20.015: resync anchor advances exactly 1 byte per iteration on a bad TPKT
//!   version byte (never 2); the same resync sub-routine is reused for both an ordinary
//!   bad-version-byte reject and a post-carry-overflow resync.
//! - BC-2.20.016: frozen `iso_on_tcp.rs` module boundary — no `impl StreamAnalyzer`, no
//!   `IsoOnTcpFlowState` type anywhere in the tree.
//! - BC-2.21.003: `on_flow_close` removes `S7commFlowState` and discards all carry bytes
//!   with no finding emitted; a no-op for an unknown `flow_key`.
//!
//! ## Test naming convention
//! Tests follow `test_BC_S_SS_NNN_xxx()` for BC-traceable tests. The non_snake_case
//! lint fires on uppercase BC IDs — suppressed intentionally, mirroring the
//! STORY-184/185/167-173 precedent in `iso_on_tcp_tests.rs` / `iec104_analyzer_tests.rs`.
//!
//! ## Provenance
//! Authored Red-first as TDD stubs (STORY-186 `tdd_mode: strict`) against the `todo!()`
//! bodies of `S7commAnalyzer::on_data`, `S7commAnalyzer::on_flow_close`, and
//! `S7commAnalyzer::resync_one_byte` in `src/analyzer/s7comm.rs`. The `todo!()` bodies
//! were replaced by the STORY-186 implementation (commit 34e9b435); these tests are
//! now GREEN against the landed STORY-186 implementation.
//! The two BC-2.20.016 static regression-guard tests are architectural/structural checks
//! (per BC-2.20.016's own "Verification Properties" note: "verified by code-review
//! inspection and the regression-guard greps ... not by a runtime proof harness") and are
//! expected to be green immediately, since the frozen module boundary is already
//! satisfied by the current `iso_on_tcp.rs` / `s7comm.rs` stub — they exist here as
//! permanent drift guards, not as Red Gate behavioral tests.
//!
//! Canonical test vectors from the BCs are used verbatim (DF-CANONICAL-FRAME-HOLDOUT-001)
//! where given; one clarification was applied per "BC is the source of truth" guidance
//! (see the `test_BC_2_20_015_resync_advances_exactly_one_byte` doc comment below) where
//! the story's inline AC-186-007 example byte sequence disagreed with BC-2.20.015's own
//! canonical vector table.

#![allow(non_snake_case)]

// Per DF-TEST-NAMESPACE-001: all STORY-186 tests are grouped inside a dedicated
// `mod story_186` wrapper to prevent test-function name collisions with other stories'
// BC-prefixed names.
mod story_186 {
    use wirerust::analyzer::s7comm::{MAX_S7_ISO_ON_TCP_CARRY_BYTES, S7commAnalyzer};
    use wirerust::findings::{Confidence, ThreatCategory, Verdict};
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    /// Canonical default flow key for these tests: an arbitrary client port against
    /// TCP/102, the registered ISO-on-TCP port (ADR-014).
    fn flow_key_default() -> FlowKey {
        FlowKey::new(
            "127.0.0.1".parse().unwrap(),
            1234,
            "127.0.0.2".parse().unwrap(),
            102,
        )
    }

    /// A minimal, complete 7-byte TPKT/COTP CR (Connect Request) frame:
    /// TPKT header `[0x03, 0x00, 0x00, 0x07]` (version=3, length=7, the RFC 1006 §6
    /// minimum) followed by a 3-byte COTP payload `[LI=1, code=0xE0 (CR), pad]`.
    /// `payload_offset = 1 + LI = 2 <= tpkt_payload.len() = 3`, so `parse_cotp_header`
    /// succeeds — content beyond that is irrelevant to this story's no-op dispatch.
    fn cr_frame_7() -> Vec<u8> {
        vec![0x03, 0x00, 0x00, 0x07, 0x01, 0xE0, 0x00]
    }

    /// A complete TPKT frame declaring the maximum representable length
    /// (`u16::MAX = 65,535`): 4-byte header + 65,531 zero payload bytes.
    fn max_length_frame() -> Vec<u8> {
        let mut frame = vec![0u8; 65_535];
        frame[0] = 0x03;
        frame[1] = 0x00;
        frame[2] = 0xFF;
        frame[3] = 0xFF;
        frame
    }

    // =========================================================================
    // BC-2.20.013: TPKT frames spanning TCP segment boundaries are reassembled via
    // directional carry buffers using walk-first, residual-bound semantics.
    // =========================================================================

    /// AC-186-001: the frame-walk loop extracts every complete TPKT frame before any
    /// byte-count bound is applied — there is no aggregate
    /// `carry[direction].len() + incoming_data.len()` pre-check anywhere.
    ///
    /// Setup: `carry_c2s` is seeded (directly, simulating a prior `on_data` call) with
    /// the first 65,000 bytes of a complete 65,535-byte max-length TPKT frame. The
    /// current `on_data` call then delivers the remaining 535 bytes of that frame PLUS
    /// a second complete 7-byte CR frame appended immediately after — total aggregate
    /// `carry.len() + incoming.len() = 65,000 + 542 = 65,542`, which is *greater* than
    /// `MAX_S7_ISO_ON_TCP_CARRY_BYTES = 65,535`.
    ///
    /// An implementation with an aggregate pre-check (the rejected
    /// PRE-CHECK-DISCARD-ALL alternative BC-2.20.013 invariant 1 explicitly forbids)
    /// would treat this as overflow: clear the carry, emit a T0814, and never dispatch
    /// either frame. The correct WALK-FIRST implementation extracts both complete
    /// frames (the 65,535-byte frame, then the 7-byte CR frame) leaving zero residual
    /// and zero findings, because the *directional carry alone* (65,000 bytes) never
    /// exceeded the bound at the start of this call (BC-2.20.014 precondition 2).
    ///
    /// Traces: BC-2.20.013 postconditions 1-2, invariant 1; AC-186-001.
    #[test]
    fn test_BC_2_20_013_walk_first_no_aggregate_precheck() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        let big_frame = max_length_frame(); // 65,535 bytes total
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.carry_c2s = big_frame[..65_000].to_vec();
        }
        // Sanity: 65,000 <= MAX_S7_ISO_ON_TCP_CARRY_BYTES (65,535), so the seeded carry
        // alone must not itself be an overflow.

        let mut delivery = big_frame[65_000..].to_vec(); // 535 completing bytes
        delivery.extend_from_slice(&cr_frame_7()); // + a second complete 7-byte frame
        assert_eq!(
            65_000 + delivery.len(),
            65_542,
            "aggregate carry+delivery must exceed MAX_S7_ISO_ON_TCP_CARRY_BYTES (65,535) \
             to exercise the anti-aggregate-precheck property"
        );

        analyzer.on_data(flow_key.clone(), &delivery, 0, Direction::ClientToServer);

        assert!(
            analyzer.findings.is_empty(),
            "walk-first semantics: an aggregate carry+incoming total that exceeds the \
             bound must NOT trigger the overflow reaction when the directional carry \
             ALONE never exceeded it at call entry (BC-2.20.013 postcondition 2, \
             invariant 1; BC-2.20.014 precondition 2) — a wrongly aggregate-pre-checked \
             implementation would emit a T0814 here"
        );
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.carry_c2s.is_empty(),
            "both complete frames (65,535-byte + 7-byte CR) must be fully extracted by \
             the walk, leaving no residual (BC-2.20.013 postcondition 3) — a wrongly \
             aggregate-pre-checked implementation would clear the carry via the overflow \
             path instead of consuming it via ordinary extraction, or would drop the \
             delivery outright"
        );
    }

    /// AC-186-002: an adversarial burst with a complete frame at the head is never
    /// dropped, regardless of the size of the trailing garbage that follows it
    /// (Ptacek/Newsham-class evasion channel; mirrors IEC-104 F-172-001 / DNP3 F-B-002).
    ///
    /// One `on_data` call delivers `[complete 7-byte CR frame][60,000 bytes of 0xAA
    /// garbage]`. The walk must extract the head frame (cursor advances past its 7
    /// bytes) and then resync through the garbage 1 byte at a time (no `0x03` byte
    /// appears anywhere in the 0xAA run). Since the resync loop stops precisely when
    /// fewer than 4 bytes remain (BC-2.20.015 postcondition 3(b)), and
    /// `60,000 mod 1 == 0` bytes are consumed one at a time from a starting remainder
    /// of exactly 60,000, the final residual is deterministically the last 3 garbage
    /// bytes.
    ///
    /// Traces: BC-2.20.013 postcondition 1, invariant 1; AC-186-002.
    #[test]
    fn test_BC_2_20_013_adversarial_burst_head_frame_not_dropped() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        let mut data = cr_frame_7();
        data.extend(std::iter::repeat_n(0xAAu8, 60_000));
        assert_eq!(data.len(), 60_007);

        analyzer.on_data(flow_key.clone(), &data, 0, Direction::ClientToServer);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.carry_c2s,
            vec![0xAAu8; 3],
            "the head 7-byte CR frame must be extracted (cursor advances past it, not \
             dropped by any evasion-style aggregate check) and the walk must then \
             resync through all 60,000 trailing garbage bytes down to the deterministic \
             3-byte remainder (BC-2.20.013 postcondition 1, invariant 1; AC-186-002)"
        );
    }

    /// AC-186-003: split-frame reassembly across two `on_data` calls (BC-2.20.013
    /// canonical vector, EC-002).
    ///
    /// Call 1 delivers `[0x03, 0x00, 0x00, 0x0A]` (a 4-byte TPKT header declaring
    /// `length=10`) with no trailing payload bytes — declared-but-incomplete. Call 2
    /// delivers the remaining 6 bytes. After call 1, `carry_c2s` must hold exactly the
    /// 4-byte header-only stash. After call 2, `working = carry ++ new_bytes` contains
    /// the complete 10-byte frame, which is extracted, leaving `carry_c2s` empty.
    ///
    /// Traces: BC-2.20.013 postcondition 1 (sub-clause b), edge case EC-002; AC-186-003.
    #[test]
    fn test_BC_2_20_013_split_frame_across_two_calls() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        let call_1 = [0x03u8, 0x00, 0x00, 0x0A];
        analyzer.on_data(flow_key.clone(), &call_1, 0, Direction::ClientToServer);
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert_eq!(
                state.carry_c2s,
                call_1.to_vec(),
                "call 1: the 4-byte header-only partial must be stashed verbatim to \
                 carry_c2s (declared-but-incomplete; BC-2.20.013 postcondition 1b)"
            );
        }
        assert!(
            analyzer.findings.is_empty(),
            "call 1: a declared-but-incomplete stash must not emit any finding"
        );

        let call_2 = [0x01u8, 0xE0, 0x00, 0x00, 0x00, 0x00]; // remaining 6 bytes -> total 10
        analyzer.on_data(flow_key.clone(), &call_2, 0, Direction::ClientToServer);
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.carry_c2s.is_empty(),
            "call 2: carry ++ new_bytes forms the complete 10-byte frame; it must be \
             extracted, leaving carry_c2s empty (BC-2.20.013 EC-002; AC-186-003)"
        );
        assert!(
            analyzer.findings.is_empty(),
            "call 2: ordinary split-frame completion must not emit any finding"
        );
    }

    // =========================================================================
    // BC-2.20.014: carry buffer bounded at MAX_S7_ISO_ON_TCP_CARRY_BYTES=65,535;
    // overflow triggers clear-and-resync with one T0814 per direction.
    // =========================================================================

    /// AC-186-004: a residual at exactly the 65,535-byte bound is legitimate, not
    /// overflow — the comparison is strict `>`, never `>=` (BC-2.20.014 edge case
    /// EC-001, invariant 1).
    ///
    /// `carry_c2s` is seeded with a complete, conformant 65,535-byte max-length TPKT
    /// frame (the largest frame the TPKT `length` field can ever represent). `on_data`
    /// is then called with an empty delivery: since `65,535 > 65,535` is false, the
    /// overflow check on entry does not fire, so the walk proceeds and extracts the
    /// frame in full — `carry_c2s` ends this call EMPTY, not retained unchanged. What
    /// the test actually verifies is the strict-`>` at-bound boundary itself: a
    /// complete, at-bound input must never trip the overflow reaction (clear + resync
    /// + T0814), which it confirms via empty findings and an unset overflow dedup flag.
    ///
    /// Traces: BC-2.20.014 precondition 2, invariant 1, edge case EC-001; AC-186-004.
    #[test]
    fn test_BC_2_20_014_at_bound_residual_no_overflow() {
        assert_eq!(
            MAX_S7_ISO_ON_TCP_CARRY_BYTES, 65_535,
            "MAX_S7_ISO_ON_TCP_CARRY_BYTES must be exactly u16::MAX (ADR-014 Decision 8)"
        );

        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.carry_c2s = max_length_frame(); // exactly 65,535 bytes
        }

        analyzer.on_data(flow_key.clone(), &[], 0, Direction::ClientToServer);

        assert!(
            analyzer.findings.is_empty(),
            "a residual of exactly 65,535 bytes must NOT trigger the overflow reaction \
             (comparison is strict '>', not '>='; BC-2.20.014 invariant 1, EC-001)"
        );
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            !state.carry_overflow_reported_c2s,
            "the carry-overflow dedup flag must remain unset when the bound is merely \
             met, not exceeded (BC-2.20.014 EC-001)"
        );
    }

    /// AC-186-005: carry overflow clears the direction's carry, resyncs, and emits
    /// exactly one T0814 (`ThreatCategory::Anomaly`, `Verdict::Possible`,
    /// `Confidence::Medium`) for this direction.
    ///
    /// `carry_c2s` is seeded (simulating prior accumulation) with 65,536 bytes of 0xAA
    /// garbage — one byte over the bound. This `on_data` call's own delivery is a
    /// complete 7-byte CR frame. Per BC-2.20.014 precondition 2, the overflow check
    /// examines the residual carry ALONE, before the current delivery is appended: the
    /// 65,536-byte garbage carry is cleared and exactly one T0814 finding is emitted;
    /// the walk then proceeds on the (now-empty-carry ++ CR-frame) working buffer,
    /// extracting the CR frame normally (BC-2.20.014 postcondition 2's "fresh-start
    /// resync, not a permanent desync latch").
    ///
    /// **BC-2.20.014 v1.1 (defense-in-depth reclassification, STORY-186 adversarial
    /// gate F-02/F-03, human ruling 2026-09-07 — Option B):** this test exercises the
    /// guard's mechanics via SYNTHETIC direct flow-state field injection
    /// (`state.carry_c2s = vec![...]` above), **not** via `on_data` — the
    /// `> 65,535`-byte residual seeded here is a state that is NOT reachable through
    /// the real `on_data` data path. Under the current BC-2.20.013 walk-first +
    /// BC-2.20.015 1-byte-resync design, the directional carry is bounded `<= 65,534`
    /// bytes by construction for all traffic (BC-2.20.014 v1.1 Invariant 1) — this is
    /// intentional and specified: the guard is retained as a structural
    /// defense-in-depth safety net against a future design regression, and its
    /// mechanics (clear-not-truncate, resync, one-T0814-per-direction) remain the
    /// binding specification IF the guard is ever reached, which this direct-injection
    /// harness exists solely to exercise in isolation (BC-2.20.014 v1.1 Canonical Test
    /// Vectors, "over-bound, guard-mechanics (SYNTHETIC ...)"). See
    /// `test_BC_2_20_014_overflow_unreachable_via_on_data` below for the positive proof
    /// that this precondition is never reached by feeding bytes through `on_data`.
    ///
    /// Traces: BC-2.20.014 postconditions 1, 3, 4; AC-186-005.
    #[test]
    fn test_BC_2_20_014_overflow_clear_resync_one_t0814_per_direction() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.carry_c2s = vec![0xAAu8; MAX_S7_ISO_ON_TCP_CARRY_BYTES + 1];
        }

        analyzer.on_data(
            flow_key.clone(),
            &cr_frame_7(),
            0,
            Direction::ClientToServer,
        );

        assert_eq!(
            analyzer.findings.len(),
            1,
            "exactly one T0814 must be emitted for this direction on overflow \
             (BC-2.20.014 postcondition 3)"
        );
        let finding = &analyzer.findings[0];
        assert_eq!(
            finding.category,
            ThreatCategory::Anomaly,
            "T0814 carry-overflow finding must have ThreatCategory::Anomaly \
             (BC-2.20.014 postcondition 3)"
        );
        assert_eq!(
            finding.verdict,
            Verdict::Possible,
            "T0814 carry-overflow finding must have Verdict::Possible \
             (BC-2.20.014 postcondition 3)"
        );
        assert_eq!(
            finding.confidence,
            Confidence::Medium,
            "T0814 carry-overflow finding must have Confidence::Medium \
             (BC-2.20.014 postcondition 3)"
        );
        assert!(
            finding.mitre_techniques.iter().any(|t| t == "T0814"),
            "the carry-overflow finding must cite T0814 (BC-2.20.014 postcondition 3)"
        );
        assert_eq!(
            finding.direction,
            Some(Direction::ClientToServer),
            "the finding must be attributed to the overflowing direction (C2S)"
        );

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.carry_overflow_reported_c2s,
            "carry_overflow_reported_c2s must be set after the first overflow \
             (BC-2.20.014 postcondition 3 dedup guard)"
        );
        assert!(
            !state.carry_overflow_reported_s2c,
            "carry_overflow_reported_s2c must remain false — independent per-direction \
             dedup flags (BC-2.20.014 postcondition 4)"
        );
        assert!(
            state.carry_c2s.is_empty(),
            "after the overflow clears carry_c2s, the fresh-start walk on the CR-frame \
             delivery alone must fully extract it, leaving no residual (BC-2.20.014 \
             postcondition 2: 'fresh-start resync, not a permanent desync latch')"
        );
    }

    /// AC-186-005: a second overflow event in the same direction on the same flow does
    /// not re-emit T0814 (BC-2.20.014 edge case EC-004) — the dedup flag suppresses
    /// repeated emission, though the carry is still cleared and resync still occurs
    /// each time.
    ///
    /// **BC-2.20.014 v1.1 (defense-in-depth reclassification):** like
    /// `test_BC_2_20_014_overflow_clear_resync_one_t0814_per_direction` above, both
    /// overflow events in this test are SYNTHETIC — directly injected onto
    /// `state.carry_c2s` — not reachable via the real `on_data` data path. Under the
    /// current walk-first (BC-2.20.013) + 1-byte-resync (BC-2.20.015) design the
    /// directional carry is bounded `<= 65,534` bytes for all traffic (BC-2.20.014 v1.1
    /// Invariant 1), so this `> 65,535` condition never arises through `on_data`. This
    /// is intentional per the reconciled spec (STORY-186 v1.1, human ruling
    /// 2026-09-07 — Option B: Defense-in-Depth): the dedup mechanics tested here remain
    /// the binding specification for the guard's behavior IF it is ever reached under a
    /// future design regression (BC-2.20.014 v1.1 Canonical Test Vectors, "repeated
    /// over-bound, same direction (SYNTHETIC ...)").
    ///
    /// Traces: BC-2.20.014 postcondition 3 dedup guard, edge case EC-004; AC-186-005.
    #[test]
    fn test_BC_2_20_014_repeated_overflow_dedup_same_direction() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // First overflow event.
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.carry_c2s = vec![0xAAu8; MAX_S7_ISO_ON_TCP_CARRY_BYTES + 1];
        }
        analyzer.on_data(flow_key.clone(), &[], 0, Direction::ClientToServer);
        assert_eq!(
            analyzer.findings.len(),
            1,
            "first overflow event must emit exactly one T0814"
        );
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert!(state.carry_overflow_reported_c2s);
            assert!(
                state.carry_c2s.is_empty(),
                "carry must be cleared on the first overflow (empty delivery leaves \
                 nothing to walk afterward)"
            );
        }

        // Second overflow event, same direction, same flow.
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.carry_c2s = vec![0xAAu8; MAX_S7_ISO_ON_TCP_CARRY_BYTES + 1];
        }
        analyzer.on_data(flow_key.clone(), &[], 0, Direction::ClientToServer);

        assert_eq!(
            analyzer.findings.len(),
            1,
            "a second overflow event in the same direction must NOT emit an additional \
             T0814 — the dedup flag suppresses re-emission (BC-2.20.014 EC-004)"
        );
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.carry_c2s.is_empty(),
            "the carry must still be cleared on the second overflow event even though \
             no new finding is emitted (BC-2.20.014 EC-004: 'carry is still cleared and \
             resync still occurs each time')"
        );
    }

    /// AC-186-006: overflow dedup flags are independent per direction (BC-2.20.014
    /// edge case EC-005). A C2S overflow does not suppress a subsequent, independent
    /// S2C overflow on the same flow.
    ///
    /// **BC-2.20.014 v1.1 (defense-in-depth reclassification):** both overflow events
    /// in this test (C2S and S2C) are SYNTHETIC — directly injected onto
    /// `state.carry_c2s`/`state.carry_s2c` — not reachable via the real `on_data` data
    /// path. Under the current walk-first (BC-2.20.013) + 1-byte-resync (BC-2.20.015)
    /// design the directional carry is bounded `<= 65,534` bytes for all traffic in
    /// both directions (BC-2.20.014 v1.1 Invariant 1), so this `> 65,535` condition
    /// never arises through `on_data` in either direction. This is intentional per the
    /// reconciled spec (STORY-186 v1.1, human ruling 2026-09-07 — Option B:
    /// Defense-in-Depth): the per-direction independence tested here remains the
    /// binding specification for the guard's dedup mechanics IF it is ever reached
    /// under a future design regression (BC-2.20.014 v1.1 Canonical Test Vectors).
    ///
    /// Traces: BC-2.20.014 postcondition 4, edge case EC-005; AC-186-006.
    #[test]
    fn test_BC_2_20_014_overflow_dedup_independent_per_direction() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // C2S overflow.
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.carry_c2s = vec![0xAAu8; MAX_S7_ISO_ON_TCP_CARRY_BYTES + 1];
        }
        analyzer.on_data(flow_key.clone(), &[], 0, Direction::ClientToServer);
        assert_eq!(analyzer.findings.len(), 1);

        // Independent S2C overflow on the same flow.
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.carry_s2c = vec![0xAAu8; MAX_S7_ISO_ON_TCP_CARRY_BYTES + 1];
        }
        analyzer.on_data(flow_key.clone(), &[], 0, Direction::ServerToClient);

        assert_eq!(
            analyzer.findings.len(),
            2,
            "the S2C overflow must emit its own T0814 independently of the C2S dedup \
             flag having already been set (BC-2.20.014 EC-005)"
        );
        let s2c_finding = &analyzer.findings[1];
        assert_eq!(
            s2c_finding.direction,
            Some(Direction::ServerToClient),
            "the second finding must be attributed to the S2C direction"
        );
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.carry_overflow_reported_c2s,
            "carry_overflow_reported_c2s must remain true from the earlier C2S event"
        );
        assert!(
            state.carry_overflow_reported_s2c,
            "carry_overflow_reported_s2c must now be set after the independent S2C \
             overflow (BC-2.20.014 EC-005)"
        );
    }

    /// **NEW (BC-2.20.014 v1.1 / STORY-186 v1.1 AC-186-005 new positive assertion,
    /// F-02 closure):** the positive, `on_data`-driven counterpart to the three
    /// SYNTHETIC direct-injection tests above. Proves that the overflow precondition
    /// (`residual.len() > 65,535`) is never reached when bytes are fed exclusively
    /// through the real `on_data` entry point — never touching the `pub carry_c2s` /
    /// `carry_s2c` fields directly — closing the adversarial F-02 concern that the
    /// SYNTHETIC guard-mechanics tests could mask an unreachable/untested real-traffic
    /// path.
    ///
    /// Two scenarios, both driven through `on_data` only:
    ///
    /// 1. **Garbage flood, single call:** one `on_data` call delivers 200,000 bytes of
    ///    non-`0x03`-anchored `0xAA` garbage. The frame-walk's resync sub-routine
    ///    (BC-2.20.015) drains this 1 byte at a time down to the deterministic 3-byte
    ///    remainder before the call returns — it never accumulates toward the
    ///    65,535-byte bound within a single call.
    /// 2. **Garbage flood, split across many calls:** the same total garbage volume is
    ///    redelivered in four separate 50,000-byte `on_data` calls on the same flow and
    ///    direction. Because BC-2.20.015's resync drains un-anchored garbage below 4
    ///    remaining bytes before *each* call's walk terminates, the ~3-byte remainder
    ///    from call N is not compounded by call N+1's fresh 50,000 bytes into anything
    ///    exceeding the bound — garbage never accumulates carry-to-carry across calls
    ///    (BC-2.20.013 Reconciliation Note).
    /// 3. **Dribbled, incomplete max-length frame:** a conformant TPKT frame declaring
    ///    `length = 65,535` (the maximum representable value) is delivered one byte
    ///    short of complete (65,534 of its 65,535 bytes), split across many small
    ///    `on_data` calls (500 bytes at a time) so the legitimate residual grows
    ///    incrementally, call by call, all the way up to the maximum legitimate
    ///    single-frame residual (65,534 bytes) without ever exceeding
    ///    `MAX_S7_ISO_ON_TCP_CARRY_BYTES` (65,535) — the walk-first residual bound
    ///    (BC-2.20.013 Reconciliation Note) holds at every observation point along the
    ///    way, not merely at the start and end.
    ///
    /// After every call in all three scenarios: zero T0814 findings are emitted, and
    /// `carry_c2s.len() < MAX_S7_ISO_ON_TCP_CARRY_BYTES` (i.e. `<= 65,534`) holds —
    /// the overflow guard's `> 65,535` precondition is never satisfied via the real
    /// `on_data` data path (BC-2.20.014 v1.1 Invariant 1 / VP-050 reachability
    /// property).
    ///
    /// Traces: BC-2.20.014 v1.1 Invariant 1, AC-186-005 (new positive assertion).
    #[test]
    fn test_BC_2_20_014_overflow_unreachable_via_on_data() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // --- Scenario 1: garbage flood delivered in one on_data call. ---
        let flood = vec![0xAAu8; 200_000];
        analyzer.on_data(flow_key.clone(), &flood, 0, Direction::ClientToServer);

        assert!(
            analyzer.findings.is_empty(),
            "a 200,000-byte non-0x03-anchored garbage flood delivered through on_data \
             alone must never emit a T0814 — the resync sub-routine (BC-2.20.015) \
             drains it to a sub-4-byte remainder within the same call, never \
             approaching the 65,535-byte bound (BC-2.20.014 v1.1 Invariant 1)"
        );
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert!(
                state.carry_c2s.len() < MAX_S7_ISO_ON_TCP_CARRY_BYTES,
                "carry_c2s ({} bytes) must stay < MAX_S7_ISO_ON_TCP_CARRY_BYTES \
                 (i.e. <= 65,534) after a single-call garbage flood driven via on_data \
                 only",
                state.carry_c2s.len()
            );
        }

        // --- Scenario 2: the same total garbage volume redelivered across four
        // separate on_data calls, to prove garbage never accumulates carry-to-carry
        // across calls (BC-2.20.013 Reconciliation Note). ---
        for _ in 0..4 {
            let chunk = vec![0xAAu8; 50_000];
            analyzer.on_data(flow_key.clone(), &chunk, 0, Direction::ClientToServer);

            assert!(
                analyzer.findings.is_empty(),
                "no T0814 must ever be emitted while redelivering non-0x03-anchored \
                 garbage across multiple on_data calls on the same flow/direction \
                 (BC-2.20.014 v1.1 Invariant 1)"
            );
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert!(
                state.carry_c2s.len() < MAX_S7_ISO_ON_TCP_CARRY_BYTES,
                "carry_c2s ({} bytes) must stay < 65,535 (i.e. <= 65,534) after each \
                 dribbled garbage on_data call — garbage never accumulates \
                 carry-to-carry across calls",
                state.carry_c2s.len()
            );
        }

        // --- Scenario 3: a conformant length=65,535 TPKT frame delivered one byte
        // short of complete, dribbled across many small on_data calls on a fresh flow,
        // so the legitimate residual grows incrementally all the way up to the maximum
        // single-frame residual (65,534 bytes) without ever exceeding the bound. ---
        let flow_key_2 = FlowKey::new(
            "127.0.0.1".parse().unwrap(),
            1235,
            "127.0.0.2".parse().unwrap(),
            102,
        );
        let mut analyzer_2 = S7commAnalyzer::new();
        let full_frame = max_length_frame(); // 65,535 bytes total (length field = 0xFFFF)
        let incomplete = &full_frame[..full_frame.len() - 1]; // 65,534 bytes: one short
        assert_eq!(incomplete.len(), MAX_S7_ISO_ON_TCP_CARRY_BYTES - 1);

        for chunk in incomplete.chunks(500) {
            analyzer_2.on_data(flow_key_2.clone(), chunk, 0, Direction::ClientToServer);

            assert!(
                analyzer_2.findings.is_empty(),
                "dribbling a still-incomplete, conformant max-length-frame residual via \
                 on_data must never emit a T0814, even as the residual grows \
                 incrementally toward the maximum legitimate single-frame size \
                 (BC-2.20.014 v1.1 Invariant 1 / EC-002)"
            );
            let state = analyzer_2.flows.get(&flow_key_2).unwrap();
            assert!(
                state.carry_c2s.len() < MAX_S7_ISO_ON_TCP_CARRY_BYTES,
                "carry_c2s ({} bytes) must stay < 65,535 (i.e. <= 65,534) at every \
                 observation point while dribbling the incomplete max-length frame via \
                 on_data",
                state.carry_c2s.len()
            );
        }
        let final_state = analyzer_2.flows.get(&flow_key_2).unwrap();
        assert_eq!(
            final_state.carry_c2s.len(),
            MAX_S7_ISO_ON_TCP_CARRY_BYTES - 1,
            "after dribbling all 65,534 available bytes of the still-incomplete \
             max-length frame, carry_c2s must hold exactly the maximum legitimate \
             single-frame residual (65,534 bytes) with no overflow ever triggered"
        );
    }

    // =========================================================================
    // BC-2.20.015: resync anchor advances exactly 1 byte per iteration on a bad TPKT
    // version byte (never 2).
    // =========================================================================

    /// AC-186-007: the resync sub-routine advances exactly 1 byte per iteration, never
    /// 2, on a bad TPKT version byte.
    ///
    /// Canonical vector taken from **BC-2.20.015's own table** (not the story's inline
    /// AC-186-007 example, which restates the byte sequence with a typo — `length=0x04`
    /// instead of `0x07` — that would make the trailing header fail BC-2.20.003's
    /// length-floor check and therefore never be a "valid frame" in the first place;
    /// per "the BC is the source of truth", the BC-2.20.015 canonical vector
    /// `[0x01, 0x03, 0x00, 0x00, 0x07]` is used here).
    ///
    /// Bytes: `[0x01, 0x03, 0x00, 0x00, 0x07]` — a spurious `0x01` immediately followed
    /// by a valid TPKT header at offset 1 declaring `length=7`. Only 4 bytes are
    /// available from offset 1 onward, so the header is declared-but-incomplete (not
    /// yet a complete frame) — but the anchor itself must be found. A correct 1-byte
    /// resync skips only the single spurious byte and stashes the anchored 4-byte
    /// partial header to carry. A buggy 2-byte advance would land on offset 2 (`0x00`,
    /// not `0x03`), fail to recognize the header, and never recover it.
    ///
    /// Traces: BC-2.20.015 postconditions 1-3, invariant 1; AC-186-007.
    #[test]
    fn test_BC_2_20_015_resync_advances_exactly_one_byte() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        let data = [0x01u8, 0x03, 0x00, 0x00, 0x07];
        analyzer.on_data(flow_key.clone(), &data, 0, Direction::ClientToServer);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.carry_c2s,
            vec![0x03u8, 0x00, 0x00, 0x07],
            "the 1-byte resync must skip only the single leading spurious byte (0x01) \
             and find + stash the anchored 4-byte partial TPKT header starting at \
             offset 1 (BC-2.20.015 postconditions 1-3); a 2-byte advance would have \
             landed on offset 2 (0x00) and missed the anchor entirely"
        );
        assert!(
            analyzer.findings.is_empty(),
            "an ordinary bad-version-byte resync (not an overflow condition) must not \
             emit any finding"
        );
    }

    /// AC-186-008: the resync sub-routine is reused verbatim for both the ordinary
    /// bad-version-byte condition and the post-carry-overflow condition — there is
    /// exactly one resync implementation, not two (BC-2.20.015 invariant 3).
    ///
    /// `carry_c2s` is seeded with an oversized (65,536-byte) garbage residual to force
    /// the carry-overflow path (BC-2.20.014). This call's own delivery is
    /// `[0x01, 0x03, 0x00, 0x00, 0x07]` — the exact same "spurious-byte-then-anchor"
    /// pattern used by `test_BC_2_20_015_resync_advances_exactly_one_byte` above, but
    /// now reached via the post-overflow code path (carry cleared, walk continues on
    /// the delivery alone). If the two conditions invoked different resync logic (e.g.
    /// a 2-byte advance for the post-overflow case), the anchor at offset 1 would be
    /// missed here even though the ordinary-path test above finds it — demonstrating a
    /// second, divergent implementation. Observing byte-for-byte identical resync
    /// behavior in both conditions is exactly the behavioral signature of BC-2.20.015
    /// invariant 3's "exactly one resync implementation, not two" claim.
    ///
    /// Traces: BC-2.20.015 invariant 3; AC-186-008.
    #[test]
    fn test_BC_2_20_015_single_resync_implementation_shared() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.carry_c2s = vec![0xAAu8; MAX_S7_ISO_ON_TCP_CARRY_BYTES + 1];
        }

        let data = [0x01u8, 0x03, 0x00, 0x00, 0x07];
        analyzer.on_data(flow_key.clone(), &data, 0, Direction::ClientToServer);

        // Exactly one finding — the overflow T0814 itself. The post-overflow resync
        // over `data` must not itself emit any additional finding.
        assert_eq!(
            analyzer.findings.len(),
            1,
            "only the carry-overflow T0814 must be emitted; the subsequent resync over \
             `data` is an ordinary (non-overflow) resync and must not itself add a \
             finding"
        );

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.carry_c2s,
            vec![0x03u8, 0x00, 0x00, 0x07],
            "the post-overflow resync must use the exact same exactly-1-byte-advance \
             sub-routine as the ordinary bad-version-byte resync (BC-2.20.015 invariant \
             3): it must find and stash the anchored 4-byte partial header at offset 1 \
             within `data`, identically to test_BC_2_20_015_resync_advances_exactly_one_byte"
        );
    }

    /// AC-186-009: the resync sub-routine always terminates for finite input, even
    /// with no valid anchor anywhere in the remaining bytes (BC-2.20.015 invariant 2).
    ///
    /// 200 bytes of `0xAA` garbage (never `0x03`) contain no valid frame boundary.
    /// The resync walk must advance to the end without an infinite loop, and the
    /// (now sub-4-byte) remainder is stashed to carry via the ordinary incomplete-frame
    /// path. Since the resync loop stops exactly when fewer than 4 bytes remain, and
    /// starts with exactly 200 bytes at cursor 0, the deterministic final residual is
    /// the last 3 garbage bytes.
    ///
    /// Traces: BC-2.20.015 postcondition 3(b), invariant 2; AC-186-009.
    #[test]
    fn test_BC_2_20_015_resync_terminates_no_valid_anchor() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        let data = vec![0xAAu8; 200];
        analyzer.on_data(flow_key.clone(), &data, 0, Direction::ClientToServer);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.carry_c2s,
            vec![0xAAu8; 3],
            "with no valid 0x03 anchor anywhere in 200 garbage bytes, the resync walk \
             must terminate (not infinite-loop) at the deterministic 3-byte remainder \
             (BC-2.20.015 postcondition 3(b), invariant 2)"
        );
        assert!(
            analyzer.findings.is_empty(),
            "an ordinary no-anchor-found resync must not emit any finding"
        );
    }

    // =========================================================================
    // BC-2.20.016: frozen `iso_on_tcp.rs` module boundary — pure free functions only,
    // no StreamAnalyzer impl, no per-flow state of its own.
    //
    // These are static/structural regression-guard tests (grep-equivalent), per
    // BC-2.20.016's own "Verification Properties" note: verified by code-review
    // inspection and static greps, not a runtime proof harness. They are expected to
    // already be green (the frozen boundary is satisfied by the current stub) and
    // exist here as permanent drift guards against a future violation.
    // =========================================================================

    /// Strips `//`, `///`, and `//!` line-comment content from Rust source text before
    /// a static grep-equivalent scan, so that architectural doc comments *describing*
    /// the absence of a construct (e.g. "no `impl StreamAnalyzer` block of any kind")
    /// are not themselves mistaken for the construct they document the absence of.
    /// A literal `grep -c` over the raw file (as BC-2.20.016's canonical vector table
    /// spells it) would false-positive on exactly this kind of self-documenting
    /// frozen-boundary prose; this scan targets the actual code, matching the BC's
    /// substantive intent (postconditions 1/3: no such *code construct* exists).
    fn strip_line_comments(content: &str) -> String {
        content
            .lines()
            .map(|line| match line.find("//") {
                Some(idx) => &line[..idx],
                None => line,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// AC-186-010: `iso_on_tcp.rs` contains zero `impl StreamAnalyzer` blocks and zero
    /// `DispatchTarget::IsoOnTcp`-shaped references.
    ///
    /// Traces: BC-2.20.016 postconditions 1-2; AC-186-010.
    #[test]
    fn test_BC_2_20_016_iso_on_tcp_has_no_stream_analyzer_impl() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/analyzer/iso_on_tcp.rs");
        let raw =
            std::fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
        let content = strip_line_comments(&raw);

        assert!(
            !content.contains("impl StreamAnalyzer"),
            "src/analyzer/iso_on_tcp.rs must contain zero `impl StreamAnalyzer` blocks in \
             actual code (BC-2.20.016 postcondition 1) — SS-20 is a stateless \
             pure-function parsing library, not a dispatcher-registered analyzer"
        );
        assert!(
            !content.contains("DispatchTarget::IsoOnTcp"),
            "src/analyzer/iso_on_tcp.rs must never reference a `DispatchTarget::IsoOnTcp` \
             variant in actual code (BC-2.20.016 postcondition 2) — the only new \
             dispatcher variant for this feature is `DispatchTarget::S7comm`"
        );
    }

    /// AC-186-011: no `IsoOnTcpFlowState` type exists anywhere in the tree — TPKT/COTP
    /// carry buffers live exclusively on `S7commFlowState` (SS-21).
    ///
    /// Traces: BC-2.20.016 postcondition 3; AC-186-011.
    #[test]
    fn test_BC_2_20_016_no_iso_on_tcp_flow_state_type_exists() {
        fn collect_rs_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            let entries = std::fs::read_dir(dir)
                .unwrap_or_else(|e| panic!("failed to read dir {}: {e}", dir.display()));
            for entry in entries {
                let entry = entry.expect("directory entry read failure");
                let path = entry.path();
                if path.is_dir() {
                    collect_rs_files(&path, out);
                } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    out.push(path);
                }
            }
        }

        let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        collect_rs_files(&src_dir, &mut files);
        assert!(
            !files.is_empty(),
            "sanity: expected to find at least one .rs file under src/"
        );

        for file in &files {
            let raw = std::fs::read_to_string(file)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", file.display()));
            let content = strip_line_comments(&raw);
            assert!(
                !content.contains("IsoOnTcpFlowState"),
                "found a reference to `IsoOnTcpFlowState` in actual code in {} — no such \
                 type may exist anywhere in the tree; TPKT/COTP carry buffers live \
                 exclusively on `S7commFlowState` (BC-2.20.016 postcondition 3)",
                file.display()
            );
        }
    }

    // =========================================================================
    // BC-2.21.003: on_flow_close removes S7commFlowState and discards all carry bytes.
    // =========================================================================

    /// AC-186-012: `on_flow_close` removes the flow's `S7commFlowState` (including
    /// non-empty carry buffers) with no finding emitted; calling it for an unknown
    /// `flow_key` is a no-op.
    ///
    /// Traces: BC-2.21.003 postconditions 1-4; AC-186-012.
    #[test]
    fn test_s7comm_on_flow_close_removes_state_discards_carry() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.carry_c2s = vec![0x03, 0x00, 0x00, 0x0A];
            state.carry_s2c = vec![0xAA, 0xAA];
        }
        assert!(
            analyzer.flows.contains_key(&flow_key),
            "precondition: flow state must exist before on_flow_close"
        );

        analyzer.on_flow_close(flow_key.clone());

        assert!(
            !analyzer.flows.contains_key(&flow_key),
            "on_flow_close must remove the flow's S7commFlowState from the analyzer's \
             per-flow map (BC-2.21.003 postcondition 1)"
        );
        assert!(
            analyzer.findings.is_empty(),
            "on_flow_close must not emit any finding, even with non-empty carry buffers \
             at closure (BC-2.21.003 postconditions 2-3)"
        );

        // Closing an unknown flow_key must be a no-op (BC-2.21.003 postcondition 4) —
        // must not panic, and must not disturb any other flow's state.
        //
        // NOTE: `flow_key` (above) was just removed by the on_flow_close call, so
        // re-inserting state under the *same* 4-tuple (`flow_key_default()`) here is
        // deliberately re-creating a flow that shares `flow_key`'s identity — this is
        // NOT a "different" flow for FlowKey-equality purposes, only a flow that
        // happens to be tracked *after* the original's closure. It is named
        // `reopened_flow_key` (not `other_key`) to make that byte-identity explicit and
        // avoid the false impression that it exercises a distinct, second flow.
        let reopened_flow_key = flow_key_default();
        {
            let state = analyzer.flows.entry(reopened_flow_key.clone()).or_default();
            state.carry_c2s = vec![0x11, 0x22];
        }
        let unknown_key = FlowKey::new(
            "10.0.0.1".parse().unwrap(),
            9999,
            "10.0.0.2".parse().unwrap(),
            102,
        );
        analyzer.on_flow_close(unknown_key);
        assert_eq!(
            analyzer.flows.get(&reopened_flow_key).unwrap().carry_c2s,
            vec![0x11, 0x22],
            "closing an unknown flow_key must not disturb any other tracked flow's state \
             (BC-2.21.003 postcondition 4 no-op)"
        );
    }

    /// EC-002 (BC-2.21.003): `on_flow_close` is called twice for the SAME `FlowKey`
    /// (defensive double-close, adversarial finding F-06). The first call removes the
    /// flow's `S7commFlowState` and emits no finding (BC-2.21.003 postconditions 1-3);
    /// the second call — against the same, now-already-removed `flow_key` — must be a
    /// harmless no-op per postcondition 4's "does not exist for the FlowKey" path: it
    /// must not panic, must not re-create state, and must not emit any finding. This is
    /// distinct from `test_s7comm_on_flow_close_removes_state_discards_carry`'s
    /// unknown-key no-op case above, which exercises a `flow_key` that was *never*
    /// tracked in the first place, rather than one tracked-then-closed-then-closed-again.
    ///
    /// Traces: BC-2.21.003 postcondition 4, edge case EC-002.
    #[test]
    fn test_BC_2_21_003_double_close_same_flow_key_is_idempotent_no_op() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.carry_c2s = vec![0x03, 0x00, 0x00, 0x0A];
            state.carry_s2c = vec![0xAA, 0xAA];
        }
        assert!(
            analyzer.flows.contains_key(&flow_key),
            "precondition: flow state must exist before the first on_flow_close"
        );

        // First close: removes the state, emits no finding.
        analyzer.on_flow_close(flow_key.clone());
        assert!(
            !analyzer.flows.contains_key(&flow_key),
            "first on_flow_close must remove the flow's S7commFlowState \
             (BC-2.21.003 postcondition 1)"
        );
        assert!(
            analyzer.findings.is_empty(),
            "first on_flow_close must not emit any finding (BC-2.21.003 postconditions \
             2-3), even with non-empty carry buffers at closure"
        );

        // Second close on the SAME flow_key: idempotent no-op (BC-2.21.003 EC-002).
        analyzer.on_flow_close(flow_key.clone());
        assert!(
            !analyzer.flows.contains_key(&flow_key),
            "second on_flow_close on the same, already-removed flow_key must remain a \
             no-op — state stays absent, it is not re-created (BC-2.21.003 \
             postcondition 4 / EC-002 'second call is a no-op')"
        );
        assert!(
            analyzer.findings.is_empty(),
            "the second (double) close must not emit any finding either — a harmless \
             no-op, not an error condition (BC-2.21.003 EC-002)"
        );
    }

    // =========================================================================
    // VP-050: TPKT/COTP Carry-Buffer Residual-Bound Reassembly, Overflow Isolation,
    // and 1-Byte Resync (proptest P1; traces BC-2.20.013..015).
    // =========================================================================

    mod vp050 {
        use proptest::prelude::*;
        use wirerust::analyzer::s7comm::{MAX_S7_ISO_ON_TCP_CARRY_BYTES, S7commAnalyzer};
        use wirerust::reassembly::flow::FlowKey;
        use wirerust::reassembly::handler::Direction;

        fn flow_key_default() -> FlowKey {
            FlowKey::new(
                "127.0.0.1".parse().unwrap(),
                1234,
                "127.0.0.2".parse().unwrap(),
                102,
            )
        }

        proptest! {
            /// VP-050 sub-property: walk-first residual bound (BC-2.20.014 invariant
            /// 1). For any pre-existing directional carry (including deliberately
            /// oversized, adversarial values well past the bound) and any incoming
            /// delivery, the residual left in `carry_c2s` after `on_data` returns must
            /// never exceed `MAX_S7_ISO_ON_TCP_CARRY_BYTES` — an overflow always clears
            /// the carry back down; it can never grow unbounded.
            #[test]
            fn proptest_vp050_walk_first_residual_bound(
                preexisting_carry in prop::collection::vec(any::<u8>(), 0..70_000),
                incoming in prop::collection::vec(any::<u8>(), 0..500),
            ) {
                let mut analyzer = S7commAnalyzer::new();
                let flow_key = flow_key_default();
                {
                    let state = analyzer.flows.entry(flow_key.clone()).or_default();
                    state.carry_c2s = preexisting_carry;
                }

                analyzer.on_data(flow_key.clone(), &incoming, 0, Direction::ClientToServer);

                let state = analyzer.flows.get(&flow_key).unwrap();
                prop_assert!(
                    state.carry_c2s.len() <= MAX_S7_ISO_ON_TCP_CARRY_BYTES,
                    "carry_c2s residual ({}) must never exceed MAX_S7_ISO_ON_TCP_CARRY_BYTES \
                     ({}) for any input (BC-2.20.014 invariant 1 / VP-050)",
                    state.carry_c2s.len(),
                    MAX_S7_ISO_ON_TCP_CARRY_BYTES
                );
            }

            /// VP-050 sub-property: direction isolation (BC-2.20.013 invariant 3). An
            /// `on_data` call in one direction must never mutate the other direction's
            /// carry buffer — `carry_c2s` after a C2S delivery followed by an
            /// unrelated S2C delivery must be byte-for-byte identical to `carry_c2s`
            /// immediately after the C2S delivery alone.
            #[test]
            fn proptest_vp050_direction_isolation(
                c2s_data in prop::collection::vec(any::<u8>(), 0..300),
                s2c_data in prop::collection::vec(any::<u8>(), 0..300),
            ) {
                let mut analyzer = S7commAnalyzer::new();
                let flow_key = flow_key_default();

                analyzer.on_data(flow_key.clone(), &c2s_data, 0, Direction::ClientToServer);
                let carry_c2s_after_c2s_only = analyzer
                    .flows
                    .get(&flow_key)
                    .map(|s| s.carry_c2s.clone())
                    .unwrap_or_default();

                analyzer.on_data(flow_key.clone(), &s2c_data, 0, Direction::ServerToClient);
                let state = analyzer.flows.get(&flow_key).unwrap();

                prop_assert_eq!(
                    &state.carry_c2s,
                    &carry_c2s_after_c2s_only,
                    "an S2C delivery must never mutate carry_c2s (BC-2.20.013 invariant 3 \
                     directional isolation / VP-050)"
                );
                prop_assert!(state.carry_c2s.len() <= MAX_S7_ISO_ON_TCP_CARRY_BYTES);
                prop_assert!(state.carry_s2c.len() <= MAX_S7_ISO_ON_TCP_CARRY_BYTES);
            }

            /// VP-050 sub-property: 1-byte resync advance (BC-2.20.015 postconditions
            /// 1-3, invariant 1). For any length of leading non-`0x03` garbage
            /// followed by a complete, valid 7-byte TPKT/COTP frame, the resync walk
            /// must find and fully extract that trailing frame — a resync that ever
            /// advanced by more than 1 byte could skip the anchor and leave the frame
            /// unextracted.
            #[test]
            fn proptest_vp050_resync_one_byte_advance(garbage_len in 0usize..50) {
                let mut data = vec![0xAAu8; garbage_len];
                data.extend_from_slice(&[0x03, 0x00, 0x00, 0x07, 0x01, 0xE0, 0x00]);

                let mut analyzer = S7commAnalyzer::new();
                let flow_key = flow_key_default();
                analyzer.on_data(flow_key.clone(), &data, 0, Direction::ClientToServer);

                let state = analyzer.flows.get(&flow_key).unwrap();
                prop_assert!(
                    state.carry_c2s.is_empty(),
                    "a 1-byte resync must find the anchor at offset {} (past the leading \
                     garbage) and fully extract the trailing complete 7-byte frame, \
                     leaving carry_c2s empty (BC-2.20.015 postconditions 1-3 / VP-050); \
                     carry_c2s = {:?}",
                    garbage_len,
                    state.carry_c2s
                );
            }
        }
    }
}
