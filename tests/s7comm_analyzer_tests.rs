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

    /// AC-186-004(a): [SYNTHETIC, unreachable via `on_data`] a residual at exactly the
    /// literal 65,535-byte bound is legitimate, not overflow — the comparison is
    /// strict `>`, never `>=` (BC-2.20.014 edge case EC-006, invariant 1).
    ///
    /// **SYNTHETIC direct field injection (BC-2.20.014 v1.2 / STORY-186 v1.2
    /// consistency audit finding #2 — same synthetic direct-flow-state-injection
    /// labeling convention as AC-186-005/006):** `carry_c2s` is seeded, by direct
    /// field assignment, with a complete, conformant 65,535-byte max-length TPKT
    /// frame constructed by `max_length_frame()` — bypassing the `on_data` walk-first
    /// path entirely. Under the walk-first design (BC-2.20.013), a residual of
    /// exactly 65,535 bytes is UNREALIZABLE via real `on_data` traffic: nothing the
    /// walk actually stashes to carry can be longer than 65,534 bytes (a declared
    /// `length = 65,535` frame that is fully available is extracted whole on the
    /// walk, not stashed), so a 65,535-byte residual can only arise via this kind of
    /// direct synthetic injection, never through real frame walking (BC-2.20.014
    /// v1.2 Invariant 1). No on_data call sequence can ever produce this
    /// precondition; this test exists purely to pin the guard's strict-`>`
    /// comparison operator at the literal boundary value.
    ///
    /// `on_data` is then called with an empty delivery: since `65,535 > 65,535` is
    /// false, the overflow check on entry does not fire, so the walk proceeds and
    /// extracts the frame in full — `carry_c2s` ends this call EMPTY, not retained
    /// unchanged. What the test actually verifies is the strict-`>` at-bound boundary
    /// itself: a complete, at-bound input must never trip the overflow reaction
    /// (clear + resync + T0814), which it confirms via empty findings and an unset
    /// overflow dedup flag.
    ///
    /// See `test_BC_2_20_014_live_near_bound_residual_reachable` below for the LIVE,
    /// `on_data`-reachable counterpart at the actual maximum reachable residual
    /// (65,534 bytes).
    ///
    /// Traces: BC-2.20.014 invariant 1, edge case EC-006; AC-186-004(a).
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
             (comparison is strict '>', not '>='; BC-2.20.014 invariant 1, EC-006)"
        );
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            !state.carry_overflow_reported_c2s,
            "the carry-overflow dedup flag must remain unset when the bound is merely \
             met, not exceeded (BC-2.20.014 EC-006)"
        );
    }

    /// AC-186-004(b): [LIVE, reachable via real `on_data`] the actual maximum residual
    /// reachable via real traffic — 65,534 bytes, one byte short of the guard
    /// constant — never triggers overflow, and completing the frame with its final
    /// byte extracts it and empties the carry.
    ///
    /// Driven ONLY through public `on_data` calls (no direct field injection for
    /// setup, unlike `test_BC_2_20_014_at_bound_residual_no_overflow` above). A TPKT
    /// frame declaring `length = 65,535` (`max_length_frame()`) is delivered minus its
    /// final byte, split across multiple `on_data` calls as progressive accumulation
    /// (BC-2.20.014 edge case EC-002): the directional carry grows call by call, and
    /// at every observation point — including every intermediate accumulation step —
    /// it must hold exactly the bytes delivered so far, never trip the overflow
    /// reaction, and leave the overflow dedup flag unset. Once the final byte is then
    /// delivered via a subsequent `on_data` call, the frame is complete and must be
    /// extracted, leaving the carry empty.
    ///
    /// Since this story's frame-walk loop dispatches extracted frames to
    /// `iso_on_tcp::parse_cotp_header` and discards the result (`let _ = ...` in
    /// `S7commAnalyzer::on_data` — STORY-187 wires classification/findings from this
    /// dispatch), `self.findings` in this story's scope can only ever contain the
    /// BC-2.20.014 T0814 carry-overflow finding. Asserting `analyzer.findings.is_empty()`
    /// after final-byte extraction is therefore already the precise
    /// carry-overflow/T0814-absence assertion, not a weaker "no findings of any kind"
    /// check that happens to coincide.
    ///
    /// See `test_BC_2_20_014_live_near_bound_residual_single_call` below for the
    /// single-call variant of this same maximum-residual case (BC-2.20.014 edge case
    /// EC-001).
    ///
    /// Traces: BC-2.20.014 invariant 1, edge case EC-001, edge case EC-002;
    /// BC-2.20.013; AC-186-004(b); VP-050 clause (c) (REACHABLE-BOUND INVARIANT).
    #[test]
    fn test_BC_2_20_014_live_near_bound_residual_reachable() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        let full_frame = max_length_frame(); // 65,535 bytes total (length field = 0xFFFF)
        let incomplete = &full_frame[..full_frame.len() - 1]; // 65,534 bytes: one short
        assert_eq!(incomplete.len(), MAX_S7_ISO_ON_TCP_CARRY_BYTES - 1);

        // Progressive multi-call accumulation (EC-002): deliver the 65,534-byte
        // near-bound residual across several segments/on_data calls, checking the
        // carry length, empty findings, and unset dedup flag at every step.
        let mut delivered = 0usize;
        for chunk in incomplete.chunks(20_000) {
            analyzer.on_data(flow_key.clone(), chunk, 0, Direction::ClientToServer);
            delivered += chunk.len();

            assert!(
                analyzer.findings.is_empty(),
                "no finding may be emitted while progressively accumulating a still- \
                 incomplete, conformant max-length-frame residual via on_data \
                 (BC-2.20.014 invariant 1, edge case EC-002)"
            );
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert_eq!(
                state.carry_c2s.len(),
                delivered,
                "carry_c2s must hold exactly the bytes delivered so far at every \
                 intermediate accumulation step (BC-2.20.014 edge case EC-002)"
            );
            assert!(
                !state.carry_overflow_reported_c2s,
                "the carry-overflow dedup flag must remain unset at every intermediate \
                 accumulation step, including the peak of 65,534 bytes (BC-2.20.014 \
                 edge case EC-002)"
            );
        }
        assert_eq!(
            delivered,
            MAX_S7_ISO_ON_TCP_CARRY_BYTES - 1,
            "sanity: the full 65,534-byte near-bound residual must have been delivered \
             across the progressive on_data calls above"
        );
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert_eq!(
                state.carry_c2s.len(),
                65_534,
                "after all-but-the-final-byte has been delivered, carry_c2s must hold \
                 exactly 65,534 bytes — the maximum residual reachable via real \
                 on_data traffic (BC-2.20.014 invariant 1, edge case EC-001)"
            );
        }

        // Deliver the final byte: the frame completes and must be extracted, leaving
        // carry_c2s empty (BC-2.20.014 edge case EC-002's final-byte-completion step).
        let final_byte = &full_frame[full_frame.len() - 1..];
        analyzer.on_data(flow_key.clone(), final_byte, 0, Direction::ClientToServer);

        assert!(
            analyzer.findings.is_empty(),
            "completing the near-bound frame with its final byte must not emit a \
             carry-overflow/T0814 finding — the only finding type reachable in this \
             story's scope (BC-2.20.014 edge case EC-002)"
        );
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.carry_c2s.is_empty(),
            "the completed 65,535-byte frame must be extracted in full once its final \
             byte arrives, leaving carry_c2s empty (BC-2.20.014 edge case EC-002; \
             BC-2.20.013 postcondition 1a)"
        );
        assert!(
            !state.carry_overflow_reported_c2s,
            "the overflow dedup flag must remain unset throughout — this near-bound \
             residual never overflows at any point (BC-2.20.014 invariant 1)"
        );
    }

    /// AC-186-004(b): [LIVE, reachable via real `on_data`] single-call variant of
    /// `test_BC_2_20_014_live_near_bound_residual_reachable` above — the 65,534-byte
    /// near-bound residual arrives in one `on_data` call rather than progressively
    /// accumulated across several (BC-2.20.014 edge case EC-001).
    ///
    /// Traces: BC-2.20.014 invariant 1, edge case EC-001; AC-186-004(b).
    #[test]
    fn test_BC_2_20_014_live_near_bound_residual_single_call() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        let full_frame = max_length_frame(); // 65,535 bytes total (length field = 0xFFFF)
        let incomplete = &full_frame[..full_frame.len() - 1]; // 65,534 bytes: one short

        analyzer.on_data(flow_key.clone(), incomplete, 0, Direction::ClientToServer);

        assert!(
            analyzer.findings.is_empty(),
            "a single-call delivery of the 65,534-byte near-bound residual must not \
             emit any finding (BC-2.20.014 invariant 1, edge case EC-001)"
        );
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert_eq!(
                state.carry_c2s.len(),
                65_534,
                "carry_c2s must hold exactly 65,534 bytes after this single-call \
                 delivery — the maximum residual reachable via real on_data traffic \
                 (BC-2.20.014 edge case EC-001)"
            );
            assert!(
                !state.carry_overflow_reported_c2s,
                "the carry-overflow dedup flag must remain unset (BC-2.20.014 edge \
                 case EC-001)"
            );
        }

        // Deliver the final byte: the frame completes and must be extracted, leaving
        // carry_c2s empty.
        let final_byte = &full_frame[full_frame.len() - 1..];
        analyzer.on_data(flow_key.clone(), final_byte, 0, Direction::ClientToServer);

        assert!(
            analyzer.findings.is_empty(),
            "completing the near-bound frame with its final byte must not emit a \
             carry-overflow/T0814 finding (BC-2.20.014 edge case EC-001)"
        );
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.carry_c2s.is_empty(),
            "the completed 65,535-byte frame must be extracted in full once its final \
             byte arrives, leaving carry_c2s empty (BC-2.20.013 postcondition 1a)"
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

/// Tests for STORY-187: S7comm Flow State Completion, Four-Way `protocol_id` Dispatch
/// Skeleton, and `parse_s7comm_header` Pure-Core Parser.
///
/// Covers BC-2.21.001, BC-2.21.002, BC-2.21.004 through BC-2.21.009, and the
/// VP-051 Kani skeleton / VP-053 proptest skeleton obligations (both partial per
/// STORY-187's own VP Obligation notes — full proof execution/non-vacuous run is
/// deferred to STORY-194, and VP-053's Some(0x72)/unclassified dissection completeness
/// is deferred to STORY-190).
///
/// ## Contract coverage
/// - BC-2.21.001: `S7commFlowState` carries the full STORY-187 field set
///   (`session_established`, `classified_protocol`, `malformed_header_reported_c2s`/
///   `_s2c`), created lazily on first `on_data`, with the carry-overflow and
///   malformed-header dedup flags tracked independently.
/// - BC-2.21.002: `S7commAnalyzer::on_data`'s four-way dispatch on
///   `CotpHeader::protocol_id` — CR/CC session tracking (no classification), classic
///   (`Some(0x32)`) dissection entry, and sticky first-classification-wins.
/// - BC-2.21.004: `parse_s7comm_header` returns `None` for `data.len() < 10`.
/// - BC-2.21.005: `parse_s7comm_header` defensively rejects `data[0] != 0x32`.
/// - BC-2.21.006: `parse_s7comm_header` extracts the common header (Job/Ack_Data/
///   Userdata happy path).
/// - BC-2.21.007: `parse_s7comm_header` returns `None` for an unrecognized ROSCTR byte.
/// - BC-2.21.008: `parse_s7comm_header` for ROSCTR=Ack requires 12 bytes and extracts
///   `error_class`/`error_code`.
/// - BC-2.21.009: the caller-side `header_len + param_length + data_length` bounds
///   check precedes any parameter/data-block slice.
///
/// ## Test naming convention
/// `test_BC_S_SS_NNN_xxx()`, matching `story_186`'s established convention in this
/// file. `#![allow(non_snake_case)]` at the file top already covers this module.
///
/// ## Provenance
/// Authored Red-first as TDD stubs (STORY-187 `tdd_mode: strict`) against the
/// `todo!()` bodies of `parse_s7comm_header` and
/// `S7commAnalyzer::dispatch_classic_s7comm`, and the `todo!()`-free no-op stub of
/// `S7commAnalyzer::classify_first_dt_frame` and the CR/CC session-tracking branch of
/// `S7commAnalyzer::dispatch_cotp_frame`, all in `src/analyzer/s7comm.rs` (commit
/// 3be2730a). Per DF-TEST-NAMESPACE-001, all STORY-187 tests are grouped inside a
/// dedicated `mod story_187` wrapper.
///
/// Canonical test vectors from BC-2.21.004/005/006/007/008/009 are used verbatim
/// (DF-CANONICAL-FRAME-HOLDOUT-001) where given.
mod story_187 {
    use wirerust::analyzer::s7comm::{
        Rosctr, S7Protocol, S7commAnalyzer, S7commFlowState, S7commHeader, parse_s7comm_header,
    };
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    /// Canonical default flow key for these tests: an arbitrary client port against
    /// TCP/102, the registered ISO-on-TCP port (ADR-014). Mirrors `story_186`'s helper
    /// of the same shape (separate module scope, so redefined here).
    fn flow_key_default() -> FlowKey {
        FlowKey::new(
            "127.0.0.1".parse().unwrap(),
            1234,
            "127.0.0.2".parse().unwrap(),
            102,
        )
    }

    /// Wraps a COTP payload in a 4-byte TPKT header (RFC 1006 §6): `[0x03, 0x00,
    /// len_hi, len_lo]` where `len = 4 + cotp_payload.len()`.
    fn tpkt_frame(cotp_payload: &[u8]) -> Vec<u8> {
        let total_len = 4 + cotp_payload.len();
        assert!(
            total_len <= u16::MAX as usize,
            "test helper: frame too large for the TPKT u16 length field"
        );
        let len = total_len as u16;
        let mut frame = vec![0x03u8, 0x00, (len >> 8) as u8, (len & 0xFF) as u8];
        frame.extend_from_slice(cotp_payload);
        frame
    }

    /// A complete COTP Connect Request (CR) frame wrapped in its TPKT header: LI=1,
    /// code=0xE0, one pad byte — mirrors `story_186::cr_frame_7()`'s exact shape (a
    /// complete, minimal 7-byte TPKT/COTP frame; `payload_offset = 1 + LI = 2 <=
    /// tpkt_payload.len() = 3`).
    fn cr_frame() -> Vec<u8> {
        tpkt_frame(&[0x01u8, 0xE0, 0x00])
    }

    /// A complete COTP Connect Confirm (CC) frame — same shape as `cr_frame()` but
    /// with the CC high-nibble code (`0xD0`).
    fn cc_frame() -> Vec<u8> {
        tpkt_frame(&[0x01u8, 0xD0, 0x00])
    }

    /// A complete COTP Data Transfer (DT) frame carrying `upper_payload` as its
    /// upper-layer payload: LI=1 (so `payload_offset = 2`), DT code `0xF0`, then
    /// `upper_payload` verbatim (its first byte is `protocol_id` per BC-2.20.009, when
    /// non-empty).
    fn dt_frame(upper_payload: &[u8]) -> Vec<u8> {
        let mut cotp = vec![0x01u8, 0xF0];
        cotp.extend_from_slice(upper_payload);
        tpkt_frame(&cotp)
    }

    /// A complete COTP DT frame with an EMPTY upper-layer payload (`protocol_id:
    /// None`, BC-2.20.010) — the minimum legal DT frame: LI=1, DT code only, total TPKT
    /// length = 4 + 2 = 6, one byte short of RFC 1006 §6's length-floor of 7. Per
    /// BC-2.20.003 this is actually rejected by `parse_tpkt_header` (`length < 7`), so
    /// an empty-payload DT frame is not directly constructible at the minimum TPKT
    /// size; callers needing BC-2.20.010 coverage pad with a trailing filler byte
    /// instead (kept out of `dt_frame`'s general-purpose contract).
    #[allow(dead_code)]
    fn dt_frame_empty_payload() -> Vec<u8> {
        tpkt_frame(&[0x01u8, 0xF0])
    }

    /// A minimal classic S7comm common header for Job/Ack_Data/Userdata (10 bytes):
    /// `[0x32, rosctr, 0x00, 0x00, pdu_ref(2 BE), param_len(2 BE), data_len(2 BE)]`.
    fn classic_header_bytes(rosctr: u8, pdu_ref: u16, param_len: u16, data_len: u16) -> Vec<u8> {
        let mut v = vec![0x32u8, rosctr, 0x00, 0x00];
        v.extend_from_slice(&pdu_ref.to_be_bytes());
        v.extend_from_slice(&param_len.to_be_bytes());
        v.extend_from_slice(&data_len.to_be_bytes());
        v
    }

    /// A minimal classic S7comm Ack (0x02) header (12 bytes): the 10-byte common
    /// header plus `error_class`/`error_code`.
    fn ack_header_bytes(
        pdu_ref: u16,
        param_len: u16,
        data_len: u16,
        error_class: u8,
        error_code: u8,
    ) -> Vec<u8> {
        let mut v = classic_header_bytes(0x02, pdu_ref, param_len, data_len);
        v.push(error_class);
        v.push(error_code);
        v
    }

    /// Asserts a single malformed-header T0814 finding (BC-2.21.004/007/008/009: the
    /// shared `malformed_header_reported_c2s`/`_s2c` dedup class) has exactly the
    /// expected category/verdict/confidence/technique/direction.
    fn assert_malformed_header_t0814(finding: &Finding, expected_direction: Direction) {
        assert_eq!(
            finding.category,
            ThreatCategory::Anomaly,
            "malformed classic-S7comm-header T0814 must have ThreatCategory::Anomaly \
             (BC-2.21.004 postcondition 4)"
        );
        assert_eq!(
            finding.verdict,
            Verdict::Possible,
            "malformed classic-S7comm-header T0814 must have Verdict::Possible \
             (BC-2.21.004 postcondition 4)"
        );
        assert_eq!(
            finding.confidence,
            Confidence::Medium,
            "malformed classic-S7comm-header T0814 must have Confidence::Medium \
             (BC-2.21.004 postcondition 4)"
        );
        assert!(
            finding.mitre_techniques.iter().any(|t| t == "T0814"),
            "malformed classic-S7comm-header finding must cite T0814"
        );
        assert_eq!(
            finding.direction,
            Some(expected_direction),
            "the finding must be attributed to the direction the malformed frame arrived on"
        );
    }

    // =========================================================================
    // BC-2.21.001: `S7commFlowState` owns TPKT/COTP carry buffers, S7comm
    // classification state, and per-direction dedup flags.
    // =========================================================================

    /// AC-187-001: `S7commFlowState` carries the full STORY-187 field set —
    /// `session_established: bool`, `classified_protocol: Option<S7Protocol>`,
    /// `malformed_header_reported_c2s: bool`, `malformed_header_reported_s2c: bool` —
    /// in addition to STORY-186's carry fields, and the first `on_data` call for a
    /// newly classified flow with zero bytes delivered leaves every field at its
    /// documented default (BC-2.21.001's own canonical test vector).
    ///
    /// NOTE (expected pass against the stub): this test exercises only the struct's
    /// field defaults (`#[derive(Default)]`) and `on_data`'s pre-existing
    /// STORY-186 lazy-creation/frame-walk logic on an EMPTY delivery — neither path
    /// touches any `todo!()`-stubbed function (the frame-walk loop breaks immediately
    /// for `data.len() < 4` remaining, before any COTP/S7comm dispatch is reached), so
    /// this test is a legitimate, currently-green regression guard for AC-187-001's
    /// field-set/default-value contract, not a Red Gate violation.
    ///
    /// Traces: BC-2.21.001 postcondition 1, Canonical Test Vectors row 1; AC-187-001.
    #[test]
    fn test_BC_2_21_001_flow_state_field_set() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        analyzer.on_data(flow_key.clone(), &[], 0, Direction::ClientToServer);

        let state = analyzer
            .flows
            .get(&flow_key)
            .expect("S7commFlowState must be created on the first on_data call");
        assert!(
            state.carry_c2s.is_empty(),
            "carry_c2s must default to empty"
        );
        assert!(
            state.carry_s2c.is_empty(),
            "carry_s2c must default to empty"
        );
        assert!(
            !state.carry_overflow_reported_c2s,
            "carry_overflow_reported_c2s must default to false"
        );
        assert!(
            !state.carry_overflow_reported_s2c,
            "carry_overflow_reported_s2c must default to false"
        );
        assert!(
            !state.session_established,
            "session_established must default to false (BC-2.21.001 postcondition 1)"
        );
        assert_eq!(
            state.classified_protocol, None,
            "classified_protocol must default to None (BC-2.21.001 postcondition 1)"
        );
        assert!(
            !state.malformed_header_reported_c2s,
            "malformed_header_reported_c2s must default to false (BC-2.21.001 postcondition 1)"
        );
        assert!(
            !state.malformed_header_reported_s2c,
            "malformed_header_reported_s2c must default to false (BC-2.21.001 postcondition 1)"
        );
    }

    /// AC-187-002: `S7commFlowState` is created lazily on the first `on_data` call for
    /// a newly classified flow and stored in the analyzer's per-flow map, keyed by
    /// `FlowKey`.
    ///
    /// NOTE (expected pass against the stub): same rationale as
    /// `test_BC_2_21_001_flow_state_field_set` above — lazy creation via
    /// `self.flows.entry(flow_key).or_default()` was already implemented in STORY-186
    /// and is exercised here on an empty delivery that never reaches any
    /// `todo!()`-stubbed dispatch path.
    ///
    /// Traces: BC-2.21.001 postcondition 3; AC-187-002.
    #[test]
    fn test_BC_2_21_001_lazy_flow_state_creation() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        assert!(
            !analyzer.flows.contains_key(&flow_key),
            "precondition: no S7commFlowState exists before the first on_data call"
        );

        analyzer.on_data(flow_key.clone(), &[], 0, Direction::ClientToServer);

        assert!(
            analyzer.flows.contains_key(&flow_key),
            "S7commFlowState must be created lazily on the first on_data call and stored \
             keyed by FlowKey (BC-2.21.001 postcondition 3)"
        );
    }

    /// EC-001 (BC-2.21.001): a flow that never sends any bytes before close never gets
    /// an `S7commFlowState` created — `on_flow_close` is a no-op for it.
    ///
    /// NOTE (expected pass against the stub): a flow_key that `on_data` was never
    /// called for cannot appear in `analyzer.flows` (a plain `HashMap` starts empty);
    /// this is a structural guarantee, not stub-dependent behavior.
    ///
    /// Traces: BC-2.21.001 edge case EC-001.
    #[test]
    fn test_BC_2_21_001_never_touched_flow_state_never_created() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        assert!(
            !analyzer.flows.contains_key(&flow_key),
            "a flow_key never passed to on_data must never appear in the per-flow map \
             (BC-2.21.001 edge case EC-001)"
        );
        analyzer.on_flow_close(flow_key.clone());
        assert!(
            !analyzer.flows.contains_key(&flow_key),
            "closing a flow that never received on_data must remain a no-op"
        );
        assert!(analyzer.findings.is_empty());
    }

    /// EC-003 (BC-2.21.001) / Invariant 2: `carry_overflow_reported_c2s` and
    /// `malformed_header_reported_c2s` are independent dedup flags — setting one does
    /// not set or reset the other. `carry_overflow_reported_c2s` is set via SYNTHETIC
    /// direct field injection (mirroring the `story_186` SYNTHETIC convention for the
    /// carry-overflow guard, which is unreachable via real `on_data` traffic per
    /// BC-2.20.014 v1.1); `malformed_header_reported_c2s` is then driven to `true`
    /// via a genuine `on_data`-triggered malformed classic-S7comm-header condition
    /// (a `Some(0x32)` DT frame with `data.len() < 10`, BC-2.21.004) on the SAME flow
    /// direction, and both flags must end up independently `true`.
    ///
    /// Traces: BC-2.21.001 invariant 2, edge case EC-003.
    #[test]
    fn test_BC_2_21_001_dedup_flags_independent_c2s() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.carry_overflow_reported_c2s = true;
        }

        // A Some(0x32) DT frame whose payload is one byte (just the protocol-ID byte)
        // -- data.len() == 1 < 10 inside parse_s7comm_header -- triggers the
        // malformed-header T0814/dedup path (BC-2.21.004).
        let frame = dt_frame(&[0x32]);
        analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.carry_overflow_reported_c2s,
            "the SYNTHETIC pre-set carry_overflow_reported_c2s flag must remain true"
        );
        assert!(
            state.malformed_header_reported_c2s,
            "malformed_header_reported_c2s must independently become true from the \
             genuine on_data-driven malformed-header condition (BC-2.21.001 invariant 2)"
        );
    }

    // =========================================================================
    // BC-2.21.002: `S7commAnalyzer::on_data` four-way dispatch on
    // `CotpHeader::protocol_id`.
    // =========================================================================

    /// AC-187-003: a COTP CR frame followed by a matching CC frame updates
    /// `session_established`; no protocol classification occurs (classification is
    /// deferred to the first DT frame).
    ///
    /// Traces: BC-2.21.002 postcondition 2; AC-187-003.
    #[test]
    fn test_BC_2_21_002_cr_cc_updates_session_no_classification() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        analyzer.on_data(flow_key.clone(), &cr_frame(), 0, Direction::ClientToServer);
        analyzer.on_data(flow_key.clone(), &cc_frame(), 1, Direction::ServerToClient);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.session_established,
            "a CR followed by a matching CC must set session_established \
             (BC-2.21.002 postcondition 2, BC-2.21.001 postcondition 1)"
        );
        assert_eq!(
            state.classified_protocol, None,
            "CR/CC frames must never trigger protocol classification -- classification \
             is deferred to the first DT frame regardless of session_established's value \
             (BC-2.21.002 postcondition 2)"
        );
        assert!(
            analyzer.findings.is_empty(),
            "ordinary CR/CC session establishment must not emit any finding"
        );
    }

    /// EC-001 (BC-2.21.002): a flow that observes only CR/CC frames, never a DT frame,
    /// leaves `classified_protocol` as `None` for the flow's lifetime; no dissection of
    /// any kind occurs.
    ///
    /// NOTE (expected pass against the stub): `classify_first_dt_frame` is only ever
    /// invoked from the `DataTransfer` arm of `dispatch_cotp_frame`'s match -- a flow
    /// that sends only CR/CC frames never reaches that arm at all, in the current stub
    /// AND in the correct final implementation alike. This test asserts the absence of
    /// a code path being taken, which already holds true structurally; it is a
    /// legitimate regression guard for AC-187's CR/CC-only edge case, not a Red Gate
    /// violation.
    ///
    /// Traces: BC-2.21.002 edge case EC-001.
    #[test]
    fn test_BC_2_21_002_cr_cc_only_never_classifies() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        analyzer.on_data(flow_key.clone(), &cr_frame(), 0, Direction::ClientToServer);
        analyzer.on_data(flow_key.clone(), &cc_frame(), 1, Direction::ServerToClient);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.classified_protocol, None,
            "a flow observing only CR/CC frames must never classify \
             (BC-2.21.002 edge case EC-001)"
        );
    }

    /// AC-187-004: a DT frame with `protocol_id: Some(0x32)` dispatches to classic
    /// S7comm dissection -- `parse_s7comm_header` is called on the slice beginning at
    /// `payload_offset`. Exercised end-to-end via a minimal, fully valid Job PDU
    /// (empty parameter/data blocks, BC-2.21.006 EC-001 shape): once implemented, this
    /// must produce no malformed-header finding at all (the header parses cleanly and
    /// the BC-2.21.009 bounds check passes trivially for `param_length == data_length
    /// == 0`).
    ///
    /// Traces: BC-2.21.002 postcondition 3; AC-187-004.
    #[test]
    fn test_BC_2_21_002_classic_s7comm_dispatch() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        let header = classic_header_bytes(0x01, 0x0001, 0x0000, 0x0000); // Job, empty blocks
        let frame = dt_frame(&header);
        analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

        assert!(
            analyzer.findings.is_empty(),
            "a well-formed, fully-parseable classic Job PDU with empty parameter/data \
             blocks must not emit any malformed-header finding once parse_s7comm_header \
             and the BC-2.21.009 bounds check are correctly wired (BC-2.21.002 \
             postcondition 3)"
        );
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            !state.malformed_header_reported_c2s,
            "a well-formed classic Job PDU must not trip the malformed-header dedup flag"
        );
    }

    /// AC-187-005 / EC-002: on the first DT frame observed for a flow (protocol_id:
    /// `Some(0x99)`, an unrecognized/unclassified value), `classified_protocol` is set
    /// exactly once (to `Unclassified`); a LATER DT frame on the SAME flow carrying
    /// `Some(0x32)` (a fully valid classic Job PDU) does NOT overwrite it, even though
    /// the classic dispatch branch itself still runs for that later frame
    /// (BC-2.21.002's dispatch match is per-frame; only the `classified_protocol`
    /// *assignment* is sticky).
    ///
    /// Uses BC-2.21.001 EC-002's exact scenario ("First DT frame classifies
    /// Unclassified, later frame on same flow carries Some(0x32)").
    ///
    /// Traces: BC-2.21.002 postcondition 6; BC-2.21.001 edge case EC-002; AC-187-005.
    #[test]
    fn test_BC_2_21_002_sticky_first_classification() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // First DT frame: protocol_id Some(0x99), unrecognized -> Unclassified.
        let first = dt_frame(&[0x99]);
        analyzer.on_data(flow_key.clone(), &first, 0, Direction::ClientToServer);
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert_eq!(
                state.classified_protocol,
                Some(S7Protocol::Unclassified),
                "the first DT frame (protocol_id Some(0x99)) must classify the flow \
                 Unclassified exactly once (BC-2.21.002 postcondition 6)"
            );
        }

        // Second DT frame: protocol_id Some(0x32), a fully valid classic Job PDU.
        let header = classic_header_bytes(0x01, 0x0002, 0x0000, 0x0000);
        let second = dt_frame(&header);
        analyzer.on_data(flow_key.clone(), &second, 1, Direction::ClientToServer);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.classified_protocol,
            Some(S7Protocol::Unclassified),
            "a later DT frame with a DIFFERENT protocol_id (Some(0x32)) must NOT \
             overwrite the sticky first classification (BC-2.21.002 postcondition 6, \
             BC-2.21.001 edge case EC-002)"
        );
    }

    /// EC-002 (BC-2.21.002): a flow's very first observed frame is a DT frame with
    /// `protocol_id: Some(0x32)` (no prior CR/CC observed). Classification proceeds
    /// normally from the DT frame alone; `session_established` remains `false`.
    ///
    /// Traces: BC-2.21.002 edge case EC-002 (BC's own numbering; distinct from the
    /// BC-2.21.001 EC-002 referenced above).
    #[test]
    fn test_BC_2_21_002_dt_first_frame_no_prior_cr_cc() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        let header = classic_header_bytes(0x01, 0x0001, 0x0000, 0x0000);
        let frame = dt_frame(&header);
        analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.classified_protocol,
            Some(S7Protocol::Classic),
            "a DT frame with protocol_id Some(0x32) as the flow's very first observed \
             frame must classify Classic normally, without any prior CR/CC \
             (BC-2.21.002 edge case EC-002)"
        );
        assert!(
            !state.session_established,
            "session_established must remain false when no CR/CC was ever observed \
             (BC-2.21.002 edge case EC-002) -- this must not block classic dissection"
        );
    }

    // =========================================================================
    // BC-2.21.004: `parse_s7comm_header` returns None for input shorter than 10 bytes.
    // =========================================================================

    /// BC-2.21.004 canonical test vectors: `[]` (0 bytes), an 8-byte input, and a
    /// 9-byte input must all return `None` from `parse_s7comm_header` directly.
    ///
    /// Traces: BC-2.21.004 postconditions 1-2, Canonical Test Vectors rows 1-3.
    #[test]
    fn test_BC_2_21_004_parse_returns_none_for_len_lt_10() {
        assert_eq!(parse_s7comm_header(&[]), None, "0 bytes must return None");
        assert_eq!(
            parse_s7comm_header(&[0x32, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02]),
            None,
            "8 bytes (canonical vector) must return None"
        );
        assert_eq!(
            parse_s7comm_header(&[0x32, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00]),
            None,
            "9 bytes (canonical vector, one byte short) must return None"
        );
    }

    /// AC-187-006: the on_data-driven malformed-header path (a `Some(0x32)` DT frame
    /// whose payload is shorter than the 10-byte common-header minimum) returns
    /// `None` from `parse_s7comm_header` and causes `S7commAnalyzer` to emit exactly
    /// one T0814 on first occurrence per flow direction; a second occurrence on the
    /// SAME direction does not re-emit (BC-2.21.004 edge case EC-004 dedup).
    ///
    /// Traces: BC-2.21.004 postcondition 4, edge case EC-004; AC-187-006.
    #[test]
    fn test_BC_2_21_004_len_shorter_than_10_returns_none_and_emits_t0814_once() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // Payload = just the protocol-ID byte -> data.len() == 1 inside
        // parse_s7comm_header, well under the 10-byte minimum.
        let frame = dt_frame(&[0x32]);
        analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

        assert_eq!(
            analyzer.findings.len(),
            1,
            "the first malformed-length classic-S7comm-header condition on this \
             direction must emit exactly one T0814 (BC-2.21.004 postcondition 4)"
        );
        assert_malformed_header_t0814(&analyzer.findings[0], Direction::ClientToServer);
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert!(
                state.malformed_header_reported_c2s,
                "malformed_header_reported_c2s must be set after the first occurrence"
            );
        }

        // A second occurrence of the SAME malformed condition, same direction, same
        // flow -- must not re-emit (BC-2.21.004 edge case EC-004).
        analyzer.on_data(flow_key.clone(), &frame, 1, Direction::ClientToServer);
        assert_eq!(
            analyzer.findings.len(),
            1,
            "a second malformed-length occurrence on the same flow direction must NOT \
             emit an additional T0814 -- the dedup flag suppresses re-emission \
             (BC-2.21.004 edge case EC-004)"
        );
    }

    // =========================================================================
    // BC-2.21.005: `parse_s7comm_header` defensively rejects `data[0] != 0x32`.
    // =========================================================================

    /// AC-187-007: `data[0] == 0x72` (would only occur via a caller defect) is
    /// rejected with `None`; this is a pure-function hygiene contract exercised via a
    /// direct call, never reachable via BC-2.21.002's real dispatch, and emits no
    /// finding.
    ///
    /// Traces: BC-2.21.005 postconditions 1-3, Canonical Test Vectors; AC-187-007.
    #[test]
    fn test_BC_2_21_005_defensive_reject_wrong_protocol_id_byte() {
        let data = [0x72u8, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00];
        assert_eq!(
            parse_s7comm_header(&data),
            None,
            "data[0] == 0x72 must be rejected defensively (BC-2.21.005 postcondition 1)"
        );
    }

    /// BC-2.21.005 canonical test vector: `data[0] == 0x00`.
    ///
    /// Traces: BC-2.21.005 edge case EC-002, Canonical Test Vectors.
    #[test]
    fn test_BC_2_21_005_defensive_reject_zero_byte() {
        let data = [0x00u8, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00];
        assert_eq!(
            parse_s7comm_header(&data),
            None,
            "data[0] == 0x00 must be rejected defensively (BC-2.21.005 edge case EC-002)"
        );
    }

    // =========================================================================
    // BC-2.21.006: `parse_s7comm_header` extracts the common header (Job/Ack_Data/
    // Userdata happy path).
    // =========================================================================

    /// AC-187-008: the three BC-2.21.006 canonical test vectors (Job, Ack_Data,
    /// Userdata) each extract the exact expected `S7commHeader` -- full structural
    /// equality, not merely `is_some()`.
    ///
    /// Traces: BC-2.21.006 postconditions 1-5, Canonical Test Vectors; AC-187-008.
    #[test]
    fn test_BC_2_21_006_common_header_field_extraction() {
        // Job, minimal: 32 01 00 00 00 01 00 02 00 00
        let job = [0x32u8, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00];
        assert_eq!(
            parse_s7comm_header(&job),
            Some(S7commHeader {
                rosctr: Rosctr::Job,
                pdu_reference: 1,
                param_length: 2,
                data_length: 0,
                error_class: None,
                error_code: None,
                header_len: 10,
            }),
            "canonical Job vector must extract the exact expected S7commHeader"
        );

        // Ack_Data with response data: 32 03 00 00 00 01 00 02 00 04
        let ack_data = [0x32u8, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x04];
        assert_eq!(
            parse_s7comm_header(&ack_data),
            Some(S7commHeader {
                rosctr: Rosctr::AckData,
                pdu_reference: 1,
                param_length: 2,
                data_length: 4,
                error_class: None,
                error_code: None,
                header_len: 10,
            }),
            "canonical Ack_Data vector must extract the exact expected S7commHeader"
        );

        // Userdata: 32 07 00 00 00 05 00 08 00 00
        let userdata = [0x32u8, 0x07, 0x00, 0x00, 0x00, 0x05, 0x00, 0x08, 0x00, 0x00];
        assert_eq!(
            parse_s7comm_header(&userdata),
            Some(S7commHeader {
                rosctr: Rosctr::Userdata,
                pdu_reference: 5,
                param_length: 8,
                data_length: 0,
                error_class: None,
                error_code: None,
                header_len: 10,
            }),
            "canonical Userdata vector must extract the exact expected S7commHeader"
        );
    }

    /// EC-003 (BC-2.21.006): non-zero Reserved bytes (`data[2..4]`) are extracted and
    /// discarded, never causing rejection -- the field is read-but-unvalidated.
    ///
    /// Traces: BC-2.21.006 postcondition 5, edge case EC-003.
    #[test]
    fn test_BC_2_21_006_nonzero_reserved_bytes_do_not_reject() {
        let data = [0x32u8, 0x01, 0xAB, 0xCD, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00];
        assert_eq!(
            parse_s7comm_header(&data),
            Some(S7commHeader {
                rosctr: Rosctr::Job,
                pdu_reference: 1,
                param_length: 2,
                data_length: 0,
                error_class: None,
                error_code: None,
                header_len: 10,
            }),
            "non-zero Reserved bytes must be extracted-and-discarded, never causing \
             rejection or altering any other field (BC-2.21.006 postcondition 5, \
             edge case EC-003)"
        );
    }

    // =========================================================================
    // BC-2.21.007: `parse_s7comm_header` returns None for an unrecognized ROSCTR byte.
    // =========================================================================

    /// AC-187-009 (test name per story): the three BC-2.21.007 canonical
    /// unrecognized-ROSCTR test vectors (`0x00`, `0x04`, `0xFF`) each return `None`.
    ///
    /// Traces: BC-2.21.007 postconditions 1-2, Canonical Test Vectors; AC-187-009.
    #[test]
    fn test_BC_2_21_007_unrecognized_rosctr_returns_none() {
        let base = |rosctr: u8| {
            [
                0x32u8, rosctr, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00,
            ]
        };
        assert_eq!(
            parse_s7comm_header(&base(0x00)),
            None,
            "ROSCTR 0x00 (canonical vector) must return None"
        );
        assert_eq!(
            parse_s7comm_header(&base(0x04)),
            None,
            "ROSCTR 0x04 (canonical vector -- not to be confused with FC 0x04) must \
             return None"
        );
        assert_eq!(
            parse_s7comm_header(&base(0xFF)),
            None,
            "ROSCTR 0xFF (canonical vector) must return None"
        );
    }

    /// AC-187-009: an unrecognized-ROSCTR malformed condition shares the SAME
    /// per-direction dedup flag as BC-2.21.004's too-short condition -- a
    /// too-short frame followed by an unrecognized-ROSCTR frame on the SAME
    /// direction/flow must emit only the FIRST T0814, not a second one.
    ///
    /// Traces: BC-2.21.007 postcondition 3.
    #[test]
    fn test_BC_2_21_007_shares_dedup_flag_with_004_malformed_header() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // First malformed condition: too-short (BC-2.21.004).
        let too_short = dt_frame(&[0x32]);
        analyzer.on_data(flow_key.clone(), &too_short, 0, Direction::ClientToServer);
        assert_eq!(
            analyzer.findings.len(),
            1,
            "the first malformed condition must emit one T0814"
        );

        // Second, DISTINCT malformed condition on the same direction: unrecognized
        // ROSCTR (BC-2.21.007) -- must NOT emit a second finding, since both share the
        // same malformed_header_reported_c2s dedup flag (BC-2.21.007 postcondition 3).
        let unrecognized = dt_frame(&[0x32, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00]);
        analyzer.on_data(
            flow_key.clone(),
            &unrecognized,
            1,
            Direction::ClientToServer,
        );
        assert_eq!(
            analyzer.findings.len(),
            1,
            "an unrecognized-ROSCTR malformed condition sharing the dedup flag with an \
             already-reported too-short condition on the same direction must NOT emit a \
             second T0814 (BC-2.21.007 postcondition 3)"
        );
    }

    // =========================================================================
    // BC-2.21.008: `parse_s7comm_header` for ROSCTR=Ack requires 12 bytes.
    // =========================================================================

    /// AC-187-010: the three BC-2.21.008 canonical Ack test vectors -- 10 bytes
    /// (`None`, truncated), 11 bytes (`None`, truncated), 12 bytes (`Some`, exact
    /// structural equality including `error_class`/`error_code`).
    ///
    /// Traces: BC-2.21.008 postconditions 1-2, Canonical Test Vectors; AC-187-010.
    #[test]
    fn test_BC_2_21_008_ack_rosctr_12_byte_minimum_and_error_fields() {
        // 10 bytes: common header only, no error class/code -- truncated Ack.
        let ten = [0x32u8, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            parse_s7comm_header(&ten),
            None,
            "10-byte Ack (canonical vector) must return None -- truncated"
        );

        // 11 bytes: one byte short of the 12-byte Ack minimum.
        let eleven = [
            0x32u8, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(
            parse_s7comm_header(&eleven),
            None,
            "11-byte Ack (canonical vector) must return None -- one byte short"
        );

        // 12 bytes exactly: complete Ack (built via the ack_header_bytes helper).
        let twelve = ack_header_bytes(1, 0, 0, 0x00, 0x00);
        assert_eq!(twelve.len(), 12);
        assert_eq!(
            parse_s7comm_header(&twelve),
            Some(S7commHeader {
                rosctr: Rosctr::Ack,
                pdu_reference: 1,
                param_length: 0,
                data_length: 0,
                error_class: Some(0),
                error_code: Some(0),
                header_len: 12,
            }),
            "12-byte Ack (canonical vector, minimal happy path) must extract the exact \
             expected S7commHeader with error_class/error_code Some(0)"
        );
    }

    /// BC-2.21.008 postcondition 3: `error_class`/`error_code` are `Some` ONLY when
    /// `rosctr == Ack` -- cross-checked here against a non-Ack (Job) header, which
    /// must have both fields `None` (already asserted structurally within
    /// `test_BC_2_21_006_common_header_field_extraction`'s Job case; restated here as
    /// its own dedicated, BC-2.21.008-scoped assertion for direct traceability).
    ///
    /// Traces: BC-2.21.008 postcondition 3, invariant 2.
    #[test]
    fn test_BC_2_21_008_error_fields_none_for_non_ack_rosctr() {
        let job = [0x32u8, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00];
        let header = parse_s7comm_header(&job).expect("valid Job header must parse");
        assert_eq!(
            header.error_class, None,
            "error_class must be None for a non-Ack ROSCTR (BC-2.21.008 postcondition 3)"
        );
        assert_eq!(
            header.error_code, None,
            "error_code must be None for a non-Ack ROSCTR (BC-2.21.008 postcondition 3)"
        );
    }

    // =========================================================================
    // BC-2.21.009: declared param_length/data_length are bounds-checked against
    // remaining bytes before parameter/data block access.
    // =========================================================================

    /// AC-187-011: a header whose declared `param_length`/`data_length` exceed the
    /// bytes actually remaining (canonical vector `2 / 0 / 1` -- one byte short) is
    /// treated as malformed: one T0814 per flow direction (sharing the dedup flag with
    /// BC-2.21.004/007/008), and no out-of-bounds slice is ever attempted (proven
    /// indirectly here by the absence of a panic once implemented).
    ///
    /// Traces: BC-2.21.009 postconditions 1-2, Canonical Test Vectors; AC-187-011.
    #[test]
    fn test_BC_2_21_009_bounds_check_before_parameter_data_slice() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // Job header declaring param_length=2, data_length=0, but only 1 byte follows
        // the 10-byte header (canonical vector "2 / 0 / 1").
        let mut header = classic_header_bytes(0x01, 0x0001, 0x0002, 0x0000);
        header.push(0xAA); // only 1 of the declared 2 parameter bytes present
        let frame = dt_frame(&header);

        analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

        assert_eq!(
            analyzer.findings.len(),
            1,
            "declared param_length exceeding the actually-available bytes must emit \
             exactly one malformed-header T0814 (BC-2.21.009 postcondition 2)"
        );
        assert_malformed_header_t0814(&analyzer.findings[0], Direction::ClientToServer);
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.malformed_header_reported_c2s,
            "malformed_header_reported_c2s must be set (BC-2.21.009 postcondition 2 \
             shares the dedup flag with BC-2.21.004/007/008)"
        );
    }

    /// BC-2.21.009 canonical test vector: `param_length=2, data_length=0`, exactly 2
    /// bytes remaining -- bounds check passes, no finding.
    ///
    /// Traces: BC-2.21.009 Canonical Test Vectors row 1 (happy-path).
    #[test]
    fn test_BC_2_21_009_bounds_check_passes_exact_match() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        let mut header = classic_header_bytes(0x01, 0x0001, 0x0002, 0x0000);
        header.extend_from_slice(&[0xAA, 0xBB]); // exactly the declared 2 param bytes
        let frame = dt_frame(&header);

        analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

        assert!(
            analyzer.findings.is_empty(),
            "an exact declared-length-to-available-bytes match must pass the bounds \
             check cleanly, with no malformed-header finding (BC-2.21.009 Canonical \
             Test Vectors row 1)"
        );
    }

    /// EC-001 (BC-2.21.009): `param_length == 0`, `data_length == 0`, and
    /// `data.len() == header_len` exactly -- the bounds check passes trivially (an
    /// empty parameter block is legitimate, not itself an error).
    ///
    /// Traces: BC-2.21.009 edge case EC-001.
    #[test]
    fn test_BC_2_21_009_empty_parameter_and_data_blocks_trivial_pass() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        let header = classic_header_bytes(0x01, 0x0001, 0x0000, 0x0000);
        let frame = dt_frame(&header);

        analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

        assert!(
            analyzer.findings.is_empty(),
            "param_length == data_length == 0 with data.len() == header_len exactly \
             must pass the bounds check trivially, with no finding \
             (BC-2.21.009 edge case EC-001)"
        );
    }

    /// EC-007 (story-level edge case table): `param_length == 0xFFFF`, `data_length ==
    /// 0xFFFF`, `data.len() == 10` (i.e. only the bare 10-byte header, no
    /// parameter/data bytes at all) -- the bounds check must fail cleanly, with no
    /// arithmetic overflow in the `header_len + param_length + data_length` sum and no
    /// slice ever attempted (no panic).
    ///
    /// Traces: BC-2.21.009 invariant 1, STORY-187 Edge Case EC-007.
    #[test]
    fn test_BC_2_21_009_overflow_free_arithmetic_max_values() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // Job header declaring the maximum representable param_length/data_length,
        // with NO trailing bytes at all beyond the bare 10-byte header.
        let header = classic_header_bytes(0x01, 0x0001, 0xFFFF, 0xFFFF);
        assert_eq!(header.len(), 10);
        let frame = dt_frame(&header);

        // Must not panic (no arithmetic overflow, no out-of-bounds slice).
        analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

        assert_eq!(
            analyzer.findings.len(),
            1,
            "maximum-representable declared param_length/data_length against a bare \
             10-byte header must cleanly fail the bounds check and emit exactly one \
             malformed-header T0814, never panic via arithmetic overflow \
             (BC-2.21.009 invariant 1)"
        );
        assert_malformed_header_t0814(&analyzer.findings[0], Direction::ClientToServer);
    }

    // =========================================================================
    // VP-051 (Kani P0, skeleton): S7comm Header Bounds-Before-Slice Safety.
    // Traces BC-2.21.004, BC-2.21.009. Full proof execution deferred to STORY-194
    // (formal-hardening) per this story's VP-051 Kani Obligation note.
    // =========================================================================

    /// `#[cfg(kani)]` skeleton, compiled only under `cargo kani` -- under a normal
    /// `cargo test`/`cargo check` build this module compiles to nothing (mirrors the
    /// `tests/kani_proofs.rs` VP-025/VP-027 pattern and the in-source
    /// `#[cfg(kani)] mod kani_proofs` pattern used by `src/analyzer/iso_on_tcp.rs`'s
    /// VP-048/VP-049 harnesses). Located in this test file per STORY-187's own File
    /// Structure Requirements table (`tests/s7comm_analyzer_tests.rs` MODIFY: "+
    /// VP-051 Kani skeleton"), rather than inside `src/analyzer/s7comm.rs`, since
    /// `parse_s7comm_header` is `pub` and fully exercisable from the test crate.
    #[cfg(kani)]
    mod vp051_kani {
        use wirerust::analyzer::s7comm::parse_s7comm_header;

        /// VP-051: `parse_s7comm_header` must not panic for any input up to a bounded
        /// length, including every `data.len() < 10` case (BC-2.21.004) and the
        /// `data[0] != 0x32` defensive-reject case (BC-2.21.005). Also proves the
        /// BC-2.21.009 arithmetic-safety half of VP-051's obligation: for any `Some`
        /// result, `header_len + param_length + data_length` (each summed as `u128`
        /// to make the no-overflow check itself trivially safe to state) never exceeds
        /// `usize::MAX` on any wirerust target -- `header_len` is 10 or 12,
        /// `param_length`/`data_length` are each bounded by `u16::MAX` (65,535), so the
        /// sum is bounded well within `usize::MAX` on both 32-bit and 64-bit targets.
        ///
        /// The analyzer-side call-site bounds check itself
        /// (`S7commAnalyzer::dispatch_classic_s7comm`, a private fn) is exercised by
        /// this file's `test_BC_2_21_009_*` integration tests above, not by this Kani
        /// harness directly (it is not `pub`, so not reachable from the test crate for
        /// symbolic execution) -- STORY-194 re-verifies the full obligation once that
        /// bounds check is wired.
        #[kani::proof]
        fn verify_parse_s7comm_header_bounds_safety() {
            let len: usize = kani::any();
            kani::assume(len <= 300);
            let mut data = vec![0u8; len];
            for b in data.iter_mut() {
                *b = kani::any();
            }

            // Must not panic for any input (BC-2.21.004/005/006/007/008 guards).
            let result = parse_s7comm_header(&data);

            if let Some(header) = result {
                let bound = header.header_len as u128
                    + header.param_length as u128
                    + header.data_length as u128;
                assert!(
                    bound <= usize::MAX as u128,
                    "VP-051 / BC-2.21.009 invariant 1: header_len + param_length + \
                     data_length must never overflow usize"
                );
            }
        }
    }

    // =========================================================================
    // VP-053 (proptest P0, PARTIAL skeleton): `protocol_id` Four-Way Dispatch
    // Totality and Unclassified Never-Force-Fit. Traces BC-2.21.002, BC-2.21.027,
    // BC-2.21.028. This story wires the CR/CC and Some(0x32) branches fully;
    // Some(0x72)/Some(other) route to a todo!()-free structural no-op completed in
    // STORY-190. The full non-vacuous run is deferred to STORY-194.
    // =========================================================================

    mod vp053 {
        use proptest::prelude::*;
        use wirerust::analyzer::s7comm::{S7Protocol, S7commAnalyzer};
        use wirerust::reassembly::flow::FlowKey;
        use wirerust::reassembly::handler::Direction;

        fn flow_key_for(salt: u16) -> FlowKey {
            FlowKey::new(
                "127.0.0.1".parse().unwrap(),
                20_000u16.wrapping_add(salt),
                "127.0.0.2".parse().unwrap(),
                102,
            )
        }

        fn tpkt_frame(cotp_payload: &[u8]) -> Vec<u8> {
            let total_len = 4 + cotp_payload.len();
            let len = total_len as u16;
            let mut frame = vec![0x03u8, 0x00, (len >> 8) as u8, (len & 0xFF) as u8];
            frame.extend_from_slice(cotp_payload);
            frame
        }

        /// Builds a complete DT frame carrying `byte` as `protocol_id`. When `byte ==
        /// 0x32`, a fully valid minimal classic Job header (empty parameter/data
        /// blocks) follows, so that VP-053's dispatch-TOTALITY property is exercised
        /// independently of BC-2.21.004-009's header-parse REJECTION paths (those are
        /// this file's dedicated `test_BC_2_21_004/007/008/009_*` tests).
        fn dt_frame_for_protocol_id(byte: u8) -> Vec<u8> {
            let mut upper_payload = vec![byte];
            if byte == 0x32 {
                upper_payload
                    .extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]);
            }
            let mut cotp = vec![0x01u8, 0xF0];
            cotp.extend_from_slice(&upper_payload);
            tpkt_frame(&cotp)
        }

        proptest! {
            /// VP-053 (partial skeleton): for any `protocol_id` byte value, the first
            /// DT frame observed on a fresh flow must classify it exactly per
            /// BC-2.21.002's four-way table: `0x32` -> `Classic`, `0x72` -> `Plus`,
            /// any other byte -> `Unclassified` -- and a `S7commFlowState` must always
            /// be created (BC-2.21.001 postcondition 3), regardless of `protocol_id`.
            #[test]
            fn proptest_vp053_protocol_id_dispatch_totality(
                protocol_id_byte in any::<u8>(),
                salt in any::<u16>(),
            ) {
                let mut analyzer = S7commAnalyzer::new();
                let flow_key = flow_key_for(salt);

                let frame = dt_frame_for_protocol_id(protocol_id_byte);
                analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

                let state = analyzer.flows.get(&flow_key).expect(
                    "VP-053: S7commFlowState must be created lazily on the first \
                     on_data call regardless of protocol_id (BC-2.21.001 postcondition 3)"
                );

                let expected = match protocol_id_byte {
                    0x32 => S7Protocol::Classic,
                    0x72 => S7Protocol::Plus,
                    _ => S7Protocol::Unclassified,
                };
                prop_assert_eq!(
                    state.classified_protocol,
                    Some(expected),
                    "VP-053 LOAD-BEARING property: the first DT frame's protocol_id \
                     must classify the flow exactly per BC-2.21.002's four-way table \
                     -- protocol_id_byte={:#04x}",
                    protocol_id_byte
                );
            }
        }
    }

    /// A trivial compile-time/type-level sanity check that `S7commFlowState`'s
    /// STORY-187 field set is exactly what BC-2.21.001 postcondition 1 requires --
    /// this is a construction-only check (no `on_data` call), kept separate from
    /// `test_BC_2_21_001_flow_state_field_set` above for documentation clarity.
    #[test]
    fn test_BC_2_21_001_fields_constructible_with_expected_types() {
        let state = S7commFlowState {
            carry_c2s: Vec::new(),
            carry_s2c: Vec::new(),
            carry_overflow_reported_c2s: false,
            carry_overflow_reported_s2c: false,
            session_established: false,
            classified_protocol: None,
            malformed_header_reported_c2s: false,
            malformed_header_reported_s2c: false,
        };
        assert_eq!(state.classified_protocol, None);
    }
}
