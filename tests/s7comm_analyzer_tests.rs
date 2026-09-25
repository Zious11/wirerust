//! Tests for STORY-186: S7comm ISO-on-TCP Carry-Buffer Reassembly, Walk-First Frame
//! Extraction, Resync, and the Frozen SS-20/SS-21 Module Boundary; and for STORY-187:
//! S7comm Flow State Completion, Four-Way `protocol_id` Dispatch Skeleton, and
//! `parse_s7comm_header` Pure-Core Parser.
//!
//! Covers BC-2.20.013, BC-2.20.014, BC-2.20.015, BC-2.20.016, BC-2.21.003, the
//! VP-050 proptest obligation (walk-first residual bound, direction isolation, 1-byte
//! resync advance) (STORY-186); and BC-2.21.001, BC-2.21.002, BC-2.21.004 through
//! BC-2.21.009, the VP-051 Kani skeleton, the VP-053 proptest skeleton, the
//! independently-sourced canonical-frame holdout tests required by policy
//! DF-CANONICAL-FRAME-HOLDOUT-001, and the committed-fixture pcap end-to-end test
//! (`test_BC_2_21_002_setup_comm_fixture_pcap_well_formed_no_findings`) (STORY-187).
//! Each story's tests are grouped in their own `mod story_186` / `mod story_187`
//! wrapper below, each with its own module-level doc comment giving full detail.
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
/// - BC-2.21.001: `S7commFlowState` carries the full STORY-187 v1.1 field set
///   (`session_established`, `cr_observed_dir`, `classified_protocol`,
///   `malformed_header_reported_c2s`/`_s2c`), created lazily on first `on_data`, with
///   the carry-overflow and malformed-header dedup flags tracked independently.
/// - BC-2.21.002: `S7commAnalyzer::on_data`'s four-way dispatch on
///   `CotpHeader::protocol_id` — CR/CC session tracking (no classification), classic
///   (`Some(0x32)`) dissection entry gated on sticky classification, and sticky
///   first-classification-wins over `Some(byte)`-only DT frames.
/// - BC-2.21.004: `parse_s7comm_header` returns `None` for `data.len() < 10`.
/// - BC-2.21.005: `parse_s7comm_header` defensively rejects `data[0] != 0x32`.
/// - BC-2.21.006: `parse_s7comm_header` extracts the common header (Job/Userdata
///   happy path only, as of the round-4 canonical-frame holdout ruling below —
///   Ack_Data moved to BC-2.21.008).
/// - BC-2.21.007: `parse_s7comm_header` returns `None` for an unrecognized ROSCTR byte.
/// - BC-2.21.008: `parse_s7comm_header` for ROSCTR=Ack AND Ack_Data (0x02/0x03) both
///   require 12 bytes and extract `error_class`/`error_code`.
/// - BC-2.21.009: the caller-side `header_len + param_length + data_length` bounds
///   check precedes any parameter/data-block slice; extracted as the pure helper
///   `s7comm_bounds_ok` (F-14).
///
/// ## Test naming convention
/// `test_BC_S_SS_NNN_xxx()`, matching `story_186`'s established convention in this
/// file. `#![allow(non_snake_case)]` at the file top already covers this module.
///
/// ## Provenance
/// The original STORY-187 test suite originated Red-first (`tdd_mode: strict`)
/// against `todo!()` bodies in `src/analyzer/s7comm.rs`; that original scope landed
/// GREEN in commit 3be2730a and later commits. This module's round-2 addendum
/// (2026-09-24) added coverage for the human-ratified rulings from STORY-187's
/// per-story adversarial pass 1 (F-01 opposite-direction CR/CC matching, F-02
/// `protocol_id: None` never classifies, F-08 256-value ROSCTR totality, F-12 sticky
/// classification gates re-dissection, F-14 bounded VP-051 Kani harness plus the
/// `s7comm_bounds_ok` pure helper) — see STORY-187.md v1.1's Changelog for the full
/// ruling list; ALL of that round-2 coverage (F-01/F-02/F-08/F-12/F-14) is now GREEN
/// against the current implementation (see `src/analyzer/s7comm.rs`'s
/// `dispatch_cotp_frame`/`classify_first_dt_frame`/`dispatch_classic_s7comm` for the
/// F-01/F-02/F-12 logic and `s7comm_bounds_ok` for F-14) — none of it is pending
/// follow-up implementation work.
///
/// This module's round-3 addendum (per-story adversarial pass 2, STORY-187.md v1.2)
/// adds: the AC-187-013 canonical, independently-sourced classic S7comm frame test
/// required by policy DF-CANONICAL-FRAME-HOLDOUT-001 (F-19) — which originally
/// surfaced a genuine spec/implementation discrepancy over the Ack_Data (`0x03`)
/// header length (see the round-4 note below for its resolution); reason-specific
/// malformed-header evidence-text assertions and additional Ack/bounds
/// `on_data`-level coverage (F-21); a fifth AC-187-003 negative case, repeated
/// same-direction CR (F-22); and the VP-051 bounds-check-half Kani skeleton plus a
/// non-vacuity wording correction (F-18/F-24).
///
/// This module's round-4 addendum (STORY-187.md v1.3, the 2026-09-24 canonical-frame
/// holdout human ruling, DF-CANONICAL-FRAME-HOLDOUT-001) resolves the round-3
/// discrepancy: ROSCTR `0x02` (Ack) AND `0x03` (Ack_Data) BOTH require the 12-byte
/// header (`error_class = data[10]`, `error_code = data[11]`, `header_len == 12`);
/// Job (`0x01`)/Userdata (`0x07`) remain 10-byte with no error fields; a truncated
/// Ack_Data frame (10 or 11 bytes) returns `None`, exactly like a truncated Ack. The
/// former `test_BC_2_21_006_canonical_setup_communication_job_frame_on_data`
/// "DISCREPANCY -- EXPECTED TO FAIL" Ack_Data assertions are now split into their own
/// `test_BC_2_21_008_canonical_ack_data_setup_communication_response_on_data` test,
/// asserting the corrected (not discrepant) 12-byte shape as ordinary, non-vacuous
/// GREEN-gate assertions against the current implementation. Per DF-TEST-NAMESPACE-001,
/// all STORY-187 tests are grouped inside this dedicated `mod story_187` wrapper.
///
/// Canonical test vectors from BC-2.21.004/005/006/007/008/009 are used verbatim
/// where the BCs themselves specify them — these are ordinary, internally-authored
/// regression vectors and do NOT satisfy policy DF-CANONICAL-FRAME-HOLDOUT-001's
/// independent-sourcing requirement (their byte values ARE derived from this
/// project's own BCs, by design — the opposite of what that policy requires). Only
/// `test_BC_2_21_006_canonical_setup_communication_job_frame_on_data` and
/// `test_BC_2_21_008_canonical_ack_data_setup_communication_response_on_data` (F-19,
/// AC-187-013) satisfy DF-CANONICAL-FRAME-HOLDOUT-001 — their byte sequences are
/// sourced independently of this project's BCs/ADR-014, per those tests' own doc
/// comments.
mod story_187 {
    use wirerust::analyzer::s7comm::{
        Rosctr, S7Protocol, S7commAnalyzer, S7commFlowState, S7commHeader, parse_s7comm_header,
        s7comm_bounds_ok,
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

    /// A complete, genuinely constructible COTP DT frame with an EMPTY upper-layer
    /// payload (`protocol_id: None`, BC-2.20.010): TPKT header `03 00 00 07` (version
    /// 3, `length = 7`, exactly RFC 1006 §6's length-floor) followed by a 3-byte
    /// class-0 COTP DT fixed part `02 F0 80` — `LI = 2`, code `0xF0`, TPDU-NR+EOT
    /// `0x80` (EOT set, TPDU-NR 0). `payload_offset = 1 + LI = 3 == tpkt_payload.len()`
    /// (`[0x02, 0xF0, 0x80]`, 3 bytes) exactly, so `parse_cotp_header` yields
    /// `protocol_id: None` per BC-2.20.010's "no out-of-bounds index at
    /// `tpkt_payload[payload_offset]`" rule — unlike the earlier revision of this
    /// helper (which under-declared `LI = 1` and omitted the separate TPDU-NR+EOT
    /// byte, making the resulting frame one byte short of the RFC 1006 §6 floor and
    /// therefore rejected by `parse_tpkt_header` before ever reaching COTP dispatch),
    /// this is a real, dispatchable minimum-legal DT frame (BC-187 round-2 F-03 fix).
    fn dt_frame_empty_payload() -> Vec<u8> {
        vec![0x03u8, 0x00, 0x00, 0x07, 0x02, 0xF0, 0x80]
    }

    /// A minimal classic S7comm common header for Job/Userdata (10 bytes):
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

    /// A minimal classic S7comm Ack_Data (0x03) header (12 bytes): the 10-byte common
    /// header plus `error_class`/`error_code` — same shape as `ack_header_bytes`
    /// above, per the 2026-09-24 canonical-frame holdout ruling
    /// (DF-CANONICAL-FRAME-HOLDOUT-001): Ack_Data requires the SAME 12-byte header as
    /// Ack, not the plain 10-byte common header the pre-ruling (v1.0/v1.1)
    /// implementation assumed.
    fn ack_data_header_bytes(
        pdu_ref: u16,
        param_len: u16,
        data_len: u16,
        error_class: u8,
        error_code: u8,
    ) -> Vec<u8> {
        let mut v = classic_header_bytes(0x03, pdu_ref, param_len, data_len);
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

    /// F-21: asserts a malformed classic-S7comm-header T0814 finding's evidence text
    /// contains `expected_substring` -- the reason-specific evidence requirement that
    /// distinguishes the five (too short; unrecognized ROSCTR; truncated Ack;
    /// truncated Ack_Data; declared lengths exceed available, the BC-2.21.009
    /// bounds-check failure) malformed-header conditions sharing the
    /// `malformed_header_reported_c2s`/`_s2c`
    /// dedup flag from the emitted `Finding` alone. Matched against
    /// `classify_malformed_header_reason`'s / `dispatch_classic_s7comm`'s exact
    /// reason strings in `src/analyzer/s7comm.rs`.
    fn assert_reason_specific_evidence(finding: &Finding, expected_substring: &str) {
        assert!(
            finding
                .evidence
                .iter()
                .any(|e| e.contains(expected_substring)),
            "expected the malformed classic-S7comm-header finding's evidence to \
             contain {expected_substring:?} (F-21 reason-specific evidence \
             requirement, distinguishing the BC-2.21.004/007/008/009 malformed-header \
             conditions from each other) -- got {:?}",
            finding.evidence
        );
    }

    // =========================================================================
    // BC-2.21.001: `S7commFlowState` owns TPKT/COTP carry buffers, S7comm
    // classification state, and per-direction dedup flags.
    // =========================================================================

    /// AC-187-001: `S7commFlowState` carries the full STORY-187 field set —
    /// `session_established: bool`, `cr_observed_dir: Option<Direction>`,
    /// `classified_protocol: Option<S7Protocol>`, `malformed_header_reported_c2s: bool`,
    /// `malformed_header_reported_s2c: bool` — in addition to STORY-186's carry
    /// fields, and the first `on_data` call for a newly classified flow with zero
    /// bytes delivered leaves every field at its documented default (BC-2.21.001's
    /// own canonical test vector).
    ///
    /// NOTE: this test exercises only the struct's field defaults
    /// (`#[derive(Default)]`) and `on_data`'s lazy-creation/frame-walk logic on an
    /// EMPTY delivery — the frame-walk loop breaks immediately for `data.len() < 4`
    /// remaining, before any COTP/S7comm dispatch is reached — so this is a
    /// regression guard for AC-187-001's field-set/default-value contract,
    /// independent of the F-01/F-02/F-12 CR/CC-and-classification dispatch logic
    /// other tests in this module exercise directly.
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
            state.cr_observed_dir, None,
            "cr_observed_dir (the F-01 pending-CR-direction tracking field) must \
             default to None -- no CR has been observed yet (BC-2.21.001 postcondition \
             1, F-01 ruling, human-ratified 2026-09-24)"
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
    /// NOTE: same rationale as `test_BC_2_21_001_flow_state_field_set` above — lazy
    /// creation via `self.flows.entry(flow_key).or_default()` originated in STORY-186
    /// and is exercised here on an empty delivery that never reaches the COTP/S7comm
    /// dispatch path this round-2 addendum otherwise targets.
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
    /// NOTE: a flow_key that `on_data` was never called for cannot appear in
    /// `analyzer.flows` (a plain `HashMap` starts empty) — a structural guarantee
    /// independent of any dispatch-logic behavior.
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
    /// v1.1 F-01 consistency note: this test's CR (c2s) then CC (s2c) sequence is
    /// already the F-01 opposite-direction "matching CC" case, so it does not
    /// conflict with the v1.1 opposite-direction-CR->CC ruling; it remains a valid
    /// positive case and is not renamed, alongside the new, more explicitly-named
    /// `test_BC_2_21_001_cr_then_opposite_cc_sets_session_established` below (which
    /// additionally covers the negative F-01 cases this test does not).
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

    // =========================================================================
    // AC-187-003 (v1.1, F-01): session_established is set ONLY by a CC observed in
    // the direction OPPOSITE a previously-observed CR on the same flow. CR-only,
    // CC-only (no prior CR), CC-before-CR, and same-direction CC must all leave
    // session_established false (human ruling, STORY-187 per-story adversarial pass
    // 1, F-01, 2026-09-24; BC-2.21.001 postcondition 1, edge cases EC-004..EC-007).
    // =========================================================================

    /// AC-187-003 positive case: CR observed in direction A (c2s), then CC observed
    /// in direction B (s2c), opposite of A -- session_established becomes true.
    ///
    /// Traces: BC-2.21.001 postcondition 1, Canonical Test Vectors row 2; AC-187-003.
    #[test]
    fn test_BC_2_21_001_cr_then_opposite_cc_sets_session_established() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        analyzer.on_data(flow_key.clone(), &cr_frame(), 0, Direction::ClientToServer);
        analyzer.on_data(flow_key.clone(), &cc_frame(), 1, Direction::ServerToClient);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.session_established,
            "a CC observed in the direction opposite a previously-observed CR must set \
             session_established (F-01 ruling, BC-2.21.001 postcondition 1)"
        );
    }

    /// AC-187-003 negative case (a): a CR is observed, with no CC ever following on
    /// this flow (CR-only) -- session_established must remain false. Scoped
    /// EXCLUSIVELY to case (a) -- this test does NOT claim BC-2.21.001 EC-007
    /// (repeated same-direction CR), which is covered exclusively by
    /// `test_BC_2_21_001_repeated_same_direction_cr_session_not_established` below
    /// (F-22 ruling, human-ratified 2026-09-24).
    ///
    /// Traces: BC-2.21.001 postcondition 1; AC-187-003.
    #[test]
    fn test_BC_2_21_001_cr_only_session_not_established() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        analyzer.on_data(flow_key.clone(), &cr_frame(), 0, Direction::ClientToServer);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            !state.session_established,
            "a CR with no CC ever following must leave session_established false \
             (F-01 ruling; a CR alone never sets the flag)"
        );
    }

    /// AC-187-003 negative case (b): the flow's first observed COTP frame is a CC,
    /// with no prior CR (CC-only, e.g. a mid-flow capture start) --
    /// session_established must remain false: a CC with no preceding CR cannot
    /// "match" anything.
    ///
    /// Traces: BC-2.21.001 edge case EC-004; AC-187-003.
    #[test]
    fn test_BC_2_21_001_cc_only_no_prior_cr_session_not_established() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        analyzer.on_data(flow_key.clone(), &cc_frame(), 0, Direction::ServerToClient);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            !state.session_established,
            "a CC observed with no prior CR on this flow must leave \
             session_established false (F-01 ruling, BC-2.21.001 edge case EC-004)"
        );
        assert_eq!(
            state.cr_observed_dir, None,
            "a CC with no prior CR must never populate cr_observed_dir -- only a CR \
             frame writes this field, and none has been observed on this flow \
             (P11-F-3, BC-2.21.001 edge case EC-004)"
        );
    }

    /// P12-F-1: `cr_observed_dir` reflects the MOST RECENT CR on the flow, not the
    /// first -- a subsequent CR in a different direction overwrites the field
    /// (`dispatch_cotp_frame`'s `ConnectRequest` arm always writes
    /// `state.cr_observed_dir = Some(direction)`, unconditionally, never guarded by
    /// "only if unset"). Verified both ways: a CC opposite the MOST RECENT CR sets
    /// `session_established`, and a CC matching the MOST RECENT CR's direction (which
    /// happens to be the SAME direction as the now-stale first CR) does not.
    ///
    /// Traces: BC-2.21.001 postcondition 1's "most recently observed COTP CR"
    /// framing; F-01 ruling.
    #[test]
    fn test_BC_2_21_001_most_recent_cr_direction_wins() {
        // CR(c2s), CR(s2c), CC(c2s) -- the MOST RECENT CR is s2c, and c2s is opposite
        // it, so session_established must become true. Under a (buggy)
        // first-CR-wins implementation, cr_observed_dir would still read Some(c2s)
        // after the second CR, making this CC SAME-direction (not opposite),
        // leaving session_established false -- the two implementations disagree
        // here.
        {
            let mut analyzer = S7commAnalyzer::new();
            let flow_key = flow_key_default();

            analyzer.on_data(flow_key.clone(), &cr_frame(), 0, Direction::ClientToServer);
            analyzer.on_data(flow_key.clone(), &cr_frame(), 1, Direction::ServerToClient);
            analyzer.on_data(flow_key.clone(), &cc_frame(), 2, Direction::ClientToServer);

            let state = analyzer.flows.get(&flow_key).unwrap();
            assert!(
                state.session_established,
                "a CC opposite the MOST RECENT CR's direction (s2c) must set \
                 session_established, even though it matches the FIRST CR's \
                 direction (c2s) -- most-recent-CR-wins, not first-CR-wins \
                 (P12-F-1, BC-2.21.001 postcondition 1)"
            );
        }

        // CR(c2s), CR(s2c), CC(s2c) -- the MOST RECENT CR is s2c, and this CC is
        // SAME-direction, so session_established must remain false. Under a
        // first-CR-wins implementation, cr_observed_dir would still read Some(c2s),
        // making this CC opposite-direction and incorrectly setting
        // session_established true -- the two implementations disagree here too.
        {
            let mut analyzer = S7commAnalyzer::new();
            let flow_key = flow_key_default();

            analyzer.on_data(flow_key.clone(), &cr_frame(), 0, Direction::ClientToServer);
            analyzer.on_data(flow_key.clone(), &cr_frame(), 1, Direction::ServerToClient);
            analyzer.on_data(flow_key.clone(), &cc_frame(), 2, Direction::ServerToClient);

            let state = analyzer.flows.get(&flow_key).unwrap();
            assert!(
                !state.session_established,
                "a CC in the SAME direction as the MOST RECENT CR (s2c) must leave \
                 session_established false, even though it is OPPOSITE the FIRST \
                 CR's direction (c2s) -- most-recent-CR-wins, not first-CR-wins \
                 (P12-F-1, BC-2.21.001 postcondition 1)"
            );
        }
    }

    /// AC-187-003 negative case (c): a CC is observed BEFORE any CR (out-of-order
    /// arrival) -- session_established must remain false when the CC arrives, and a
    /// CR that subsequently arrives (in either direction) does not retroactively set
    /// it from the earlier CC: matching is forward-looking from the CR only.
    ///
    /// Traces: BC-2.21.001 edge case EC-005; AC-187-003.
    #[test]
    fn test_BC_2_21_001_cc_before_cr_session_not_established() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // CC arrives first (s2c), then a CR arrives afterward (c2s) -- opposite
        // direction of the CC, which per F-01 is still not a retroactive match.
        analyzer.on_data(flow_key.clone(), &cc_frame(), 0, Direction::ServerToClient);
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert!(
                !state.session_established,
                "session_established must remain false immediately after an \
                 out-of-order CC with no CR yet observed (BC-2.21.001 edge case EC-005)"
            );
        }

        analyzer.on_data(flow_key.clone(), &cr_frame(), 1, Direction::ClientToServer);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            !state.session_established,
            "a CR arriving AFTER an out-of-order CC must not retroactively set \
             session_established from that earlier CC -- matching is forward-looking \
             from the CR only, never backward-looking from the CC (F-01 ruling, \
             BC-2.21.001 edge case EC-005)"
        );
    }

    /// AC-187-003 negative case (d): a CR is observed in direction A, then a CC is
    /// observed also in direction A (same direction, not opposite) --
    /// session_established must remain false: a same-direction CC is never
    /// "matching."
    ///
    /// Traces: BC-2.21.001 edge case EC-006, Canonical Test Vectors row 3; AC-187-003.
    #[test]
    fn test_BC_2_21_001_same_direction_cc_session_not_established() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        analyzer.on_data(flow_key.clone(), &cr_frame(), 0, Direction::ClientToServer);
        analyzer.on_data(flow_key.clone(), &cc_frame(), 1, Direction::ClientToServer);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            !state.session_established,
            "a CC observed in the SAME direction as the flow's already-observed CR \
             must leave session_established false (F-01 ruling, BC-2.21.001 edge case \
             EC-006, Canonical Test Vectors row 3)"
        );
    }

    /// AC-187-003 negative case (e), F-22 (human-ratified 2026-09-24): a CR is
    /// observed in direction A, then a SECOND CR is also observed in direction A,
    /// with no intervening CC (repeated same-direction CR) -- session_established
    /// must remain false, and `cr_observed_dir` must still equal `Some(A)`: the
    /// repeated CR does not clear or corrupt the pending direction, and does not
    /// itself set session_established.
    ///
    /// Traces: BC-2.21.001 edge case EC-007; AC-187-003.
    #[test]
    fn test_BC_2_21_001_repeated_same_direction_cr_session_not_established() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        analyzer.on_data(flow_key.clone(), &cr_frame(), 0, Direction::ClientToServer);
        analyzer.on_data(flow_key.clone(), &cr_frame(), 1, Direction::ClientToServer);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            !state.session_established,
            "a repeated CR in the same direction, with no intervening CC, must leave \
             session_established false (F-22 ruling, BC-2.21.001 edge case EC-007)"
        );
        assert_eq!(
            state.cr_observed_dir,
            Some(Direction::ClientToServer),
            "the repeated CR must not clear or corrupt the pending CR direction -- \
             cr_observed_dir must still reflect the (shared) direction A after the \
             second CR (F-22 ruling, BC-2.21.001 edge case EC-007's \"reflects the \
             most recent CR (or first-observed value)\" permission is satisfied \
             identically by either choice since both CRs share direction A)"
        );
    }

    /// EC-001 (BC-2.21.002): a flow that observes only CR/CC frames, never a DT frame,
    /// leaves `classified_protocol` as `None` for the flow's lifetime; no dissection of
    /// any kind occurs.
    ///
    /// NOTE: `classify_first_dt_frame` is only ever invoked from the `DataTransfer`
    /// arm of `dispatch_cotp_frame`'s match -- a flow that sends only CR/CC frames
    /// never reaches that arm at all, structurally, independent of the F-01/F-02/F-12
    /// CR/CC-and-classification dispatch logic exercised elsewhere in this module.
    /// This is a regression guard for AC-187's CR/CC-only edge case.
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

    /// AC-187-004 (v1.1): a DT frame with `protocol_id: Some(0x32)` dispatches to
    /// classic S7comm dissection -- `parse_s7comm_header` is called on the slice
    /// beginning at `payload_offset`, and after dispatch
    /// `S7commFlowState.classified_protocol == Some(S7Protocol::Classic)`. Exercised
    /// end-to-end via a minimal, fully valid Job PDU (empty parameter/data blocks,
    /// BC-2.21.006 EC-001 shape): this produces no malformed-header finding at all
    /// (the header parses cleanly and the BC-2.21.009 bounds check passes trivially
    /// for `param_length == data_length == 0`).
    ///
    /// Traces: BC-2.21.002 postcondition 3; AC-187-004.
    #[test]
    fn test_BC_2_21_002_classic_s7comm_dispatch_asserts_classified_protocol_classic() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        let header = classic_header_bytes(0x01, 0x0001, 0x0000, 0x0000); // Job, empty blocks
        let frame = dt_frame(&header);
        analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

        assert!(
            analyzer.findings.is_empty(),
            "a well-formed, fully-parseable classic Job PDU with empty parameter/data \
             blocks must not emit any malformed-header finding (BC-2.21.002 \
             postcondition 3)"
        );
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            !state.malformed_header_reported_c2s,
            "a well-formed classic Job PDU must not trip the malformed-header dedup flag"
        );
        assert_eq!(
            state.classified_protocol,
            Some(S7Protocol::Classic),
            "after a Some(0x32) DT frame is dispatched, classified_protocol must be \
             Some(S7Protocol::Classic) (AC-187-004, BC-2.21.002 postcondition 3)"
        );
    }

    /// AC-187-004 (v1.1): a malformed (too-short, `data.len() < 10`) `0x32`-leading DT
    /// frame arriving on a flow whose sticky `classified_protocol` is ALREADY
    /// `Some(S7Protocol::Classic)` (established by an earlier, well-formed `0x32` DT
    /// frame) emits exactly one T0814 for the malformed-header condition -- not more
    /// than one, and not zero. Distinct from
    /// `test_BC_2_21_004_len_shorter_than_10_returns_none_and_emits_t0814_once`, which
    /// exercises the malformed frame as the flow's FIRST DT frame; here the flow is
    /// already Classic-classified before the malformed frame arrives.
    ///
    /// Traces: BC-2.21.002 postcondition 3, BC-2.21.004 postcondition 4; AC-187-004.
    #[test]
    fn test_BC_2_21_002_malformed_0x32_frame_on_classic_flow_emits_exactly_one_t0814() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // First: a well-formed classic Job PDU establishes Some(Classic).
        let good_header = classic_header_bytes(0x01, 0x0001, 0x0000, 0x0000);
        let good_frame = dt_frame(&good_header);
        analyzer.on_data(flow_key.clone(), &good_frame, 0, Direction::ClientToServer);
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert_eq!(
                state.classified_protocol,
                Some(S7Protocol::Classic),
                "precondition: the flow must already be sticky-classified Classic \
                 before the malformed frame below arrives"
            );
        }
        assert!(
            analyzer.findings.is_empty(),
            "precondition: no findings yet"
        );

        // Second: a malformed (too-short) 0x32-leading DT frame on the SAME
        // already-Classic-classified flow.
        let malformed_frame = dt_frame(&[0x32]);
        analyzer.on_data(
            flow_key.clone(),
            &malformed_frame,
            1,
            Direction::ClientToServer,
        );

        assert_eq!(
            analyzer.findings.len(),
            1,
            "a malformed 0x32-leading DT frame on an already-Classic-classified flow \
             must emit exactly one T0814 -- not more than one, and not zero \
             (AC-187-004, BC-2.21.004 postcondition 4)"
        );
        assert_malformed_header_t0814(&analyzer.findings[0], Direction::ClientToServer);
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.classified_protocol,
            Some(S7Protocol::Classic),
            "classified_protocol must remain Classic across the malformed frame -- \
             classification is sticky and unaffected by a later malformed-header \
             condition"
        );
    }

    /// AC-187-005 / EC-002: on the first DT frame observed for a flow (protocol_id:
    /// `Some(0x99)`, an unrecognized/unclassified value), `classified_protocol` is set
    /// exactly once (to `Unclassified`); a LATER DT frame on the SAME flow carrying
    /// `Some(0x32)` (a fully valid classic Job PDU) does NOT overwrite it. Per the
    /// F-12 sticky-classification gate (AC-187-012), the classic dissection branch
    /// does NOT run for this later frame either, since the flow's sticky
    /// `classified_protocol` is already `Some(Unclassified)` -- BC-2.21.002's
    /// dispatch match is per-frame, but BOTH the `classified_protocol` assignment AND
    /// classic-dissection eligibility are gated on the STICKY value, never on the
    /// current frame's raw `protocol_id` byte alone.
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
    // AC-187-005 (v1.1, F-02): a DT frame whose protocol_id is None carries no
    // protocol evidence and never classifies -- it does not consume the flow's
    // "first DT frame" status. Classification remains deferred to a later DT frame
    // (if any) that carries Some(byte) (human ruling, STORY-187 per-story
    // adversarial pass 1, F-02, 2026-09-24; BC-2.21.002 postconditions 5-6, edge
    // case EC-004; BC-2.21.001 postcondition 1).
    // =========================================================================

    /// AC-187-005: a flow's first DT frame carries `protocol_id: None` (empty
    /// payload, via the genuinely-constructible `dt_frame_empty_payload()` helper) --
    /// `classified_protocol` remains `None` after it. A second DT frame on the same
    /// flow then carries `protocol_id: Some(0x32)` (a fully valid classic Job PDU) --
    /// THIS frame is the one that sets `classified_protocol =
    /// Some(S7Protocol::Classic)` and is dissected (since `classified_protocol` was
    /// still unset at the moment this frame was dispatched).
    ///
    /// Traces: BC-2.21.002 postcondition 6, edge case EC-004; BC-2.21.001 edge case
    /// EC-004 (parallel "no protocol evidence" framing); AC-187-005.
    #[test]
    fn test_BC_2_21_002_none_protocol_id_dt_first_then_0x32_dt_classifies_classic() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // First DT frame: protocol_id None (empty upper-layer payload).
        analyzer.on_data(
            flow_key.clone(),
            &dt_frame_empty_payload(),
            0,
            Direction::ClientToServer,
        );
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert_eq!(
                state.classified_protocol, None,
                "a protocol_id: None DT frame carries no protocol evidence and must \
                 NOT classify the flow, even as the flow's first DT frame (F-02 \
                 ruling, BC-2.21.002 postcondition 6)"
            );
        }
        assert!(
            analyzer.findings.is_empty(),
            "a protocol_id: None DT frame must not itself emit any finding"
        );

        // Second DT frame: protocol_id Some(0x32), a fully valid classic Job PDU.
        let header = classic_header_bytes(0x01, 0x0001, 0x0000, 0x0000);
        let second = dt_frame(&header);
        analyzer.on_data(flow_key.clone(), &second, 1, Direction::ClientToServer);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.classified_protocol,
            Some(S7Protocol::Classic),
            "the SECOND DT frame (the first to carry Some(byte)) must be the one that \
             sets classified_protocol -- classification was still deferred after the \
             first, None-protocol_id frame (F-02 ruling, BC-2.21.002 postcondition 6, \
             edge case EC-004)"
        );
        assert!(
            analyzer.findings.is_empty(),
            "the well-formed classic Job PDU that classifies the flow must not itself \
             emit any malformed-header finding"
        );
    }

    /// AC-187-005: two DT frames with different `protocol_id` values arrive
    /// back-to-back within a SINGLE `on_data` call (mirrors BC-2.20.013's
    /// multi-frame walk) -- `classified_protocol`'s first-write-wins rule applies
    /// across the pair in arrival order: the first frame within the delivery sets
    /// it, the second does not overwrite it, even though the frame-walk loop
    /// dispatches both within the same `on_data` invocation.
    ///
    /// Traces: BC-2.21.002 edge case EC-003; AC-187-005.
    #[test]
    fn test_BC_2_21_002_two_frames_one_delivery_first_write_wins() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // First frame: protocol_id Some(0x72) (Plus). Second frame, concatenated in
        // the SAME delivery: protocol_id Some(0x32), a fully valid classic Job PDU.
        let mut delivery = dt_frame(&[0x72]);
        let classic_header = classic_header_bytes(0x01, 0x0001, 0x0000, 0x0000);
        delivery.extend_from_slice(&dt_frame(&classic_header));

        analyzer.on_data(flow_key.clone(), &delivery, 0, Direction::ClientToServer);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.classified_protocol,
            Some(S7Protocol::Plus),
            "within a single on_data delivery carrying two DT frames, the FIRST \
             frame's protocol_id (Some(0x72) -> Plus) must set classified_protocol; \
             the second frame (Some(0x32)) must not overwrite it -- first-write-wins \
             applies within a delivery, not merely across separate on_data calls \
             (BC-2.21.002 edge case EC-003)"
        );
    }

    // =========================================================================
    // AC-187-012 (v1.1, F-12): classic S7comm dissection is gated on the flow's
    // STICKY classified_protocol == Classic, never on the current frame's raw
    // protocol_id byte alone. A 0x32-leading DT frame on a flow already
    // sticky-classified Plus or Unclassified by an earlier DT frame is NOT
    // dissected -- no parse_s7comm_header call, no finding (human ruling, STORY-187
    // per-story adversarial pass 1, F-12, 2026-09-24; BC-2.21.002 postcondition 3,
    // invariant 4, edge case EC-005).
    // =========================================================================

    /// AC-187-012: a flow's first DT frame carries `protocol_id: Some(0x72)`, sticky
    /// classifying it `Some(S7Protocol::Plus)`. A LATER DT frame on the same flow
    /// carries `protocol_id: Some(0x32)` with a MALFORMED (too-short) classic-S7comm
    /// payload -- per the F-12 sticky gate, it must NOT be dissected at all: no
    /// `parse_s7comm_header` call is ever made, so NO finding of any kind is emitted
    /// (deliberately not a well-formed payload here: a well-formed `0x32` payload
    /// would emit zero findings regardless of whether the gate is correctly wired,
    /// since a successful parse never raises a T0814 either way -- only a MALFORMED
    /// payload distinguishes "correctly gated, never dissected" from "incorrectly
    /// ungated, dissected and rejected as malformed"). `classified_protocol` remains
    /// `Some(Plus)`.
    ///
    /// Also covers the parallel Unclassified case (first DT frame `Some(0x99)`, an
    /// unrecognized byte) in the same test, per BC-2.21.002 edge case EC-005's
    /// framing ("a flow sticky-classified Plus/Unclassified must never fall into
    /// classic dissection").
    ///
    /// Traces: BC-2.21.002 postcondition 3, invariant 4, edge case EC-005; AC-187-012.
    #[test]
    fn test_BC_2_21_002_0x32_dt_frame_not_dissected_when_sticky_classified_plus_or_unclassified() {
        // Malformed classic-S7comm payload (too short: data.len() == 1 inside
        // parse_s7comm_header) -- see the doc comment above for why the negative
        // case must use a malformed, not well-formed, payload.
        let malformed_classic = vec![0x32u8];

        // --- Plus-classified flow. ---
        {
            let mut analyzer = S7commAnalyzer::new();
            let flow_key = flow_key_default();

            analyzer.on_data(
                flow_key.clone(),
                &dt_frame(&[0x72]),
                0,
                Direction::ClientToServer,
            );
            {
                let state = analyzer.flows.get(&flow_key).unwrap();
                assert_eq!(state.classified_protocol, Some(S7Protocol::Plus));
            }

            let later = dt_frame(&malformed_classic);
            analyzer.on_data(flow_key.clone(), &later, 1, Direction::ClientToServer);

            assert!(
                analyzer.findings.is_empty(),
                "a later, MALFORMED 0x32-leading DT frame on an already-Plus-classified \
                 flow must emit NO finding at all -- the gate must prevent \
                 parse_s7comm_header from ever being called for it, so the malformed \
                 payload never reaches the malformed-header T0814 path either (F-12 \
                 ruling, BC-2.21.002 postcondition 3, edge case EC-005)"
            );
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert_eq!(
                state.classified_protocol,
                Some(S7Protocol::Plus),
                "classified_protocol must remain Some(Plus) -- a flow, once \
                 classified, is never re-interpreted under a different protocol's \
                 parser (F-12 ruling, ADR-014 Decision 2 no-misattribution guarantee)"
            );
            assert!(
                !state.malformed_header_reported_c2s,
                "the malformed-header dedup flag must NOT be set -- the gated frame \
                 was never passed to parse_s7comm_header at all (F-12 ruling)"
            );
        }

        // --- Unclassified-classified flow. ---
        {
            let mut analyzer = S7commAnalyzer::new();
            let flow_key = flow_key_default();

            analyzer.on_data(
                flow_key.clone(),
                &dt_frame(&[0x99]),
                0,
                Direction::ClientToServer,
            );
            {
                let state = analyzer.flows.get(&flow_key).unwrap();
                assert_eq!(state.classified_protocol, Some(S7Protocol::Unclassified));
            }

            let later = dt_frame(&malformed_classic);
            analyzer.on_data(flow_key.clone(), &later, 1, Direction::ClientToServer);

            assert!(
                analyzer.findings.is_empty(),
                "a later, MALFORMED 0x32-leading DT frame on an already- \
                 Unclassified-classified flow must emit NO finding at all (F-12 \
                 ruling, BC-2.21.002 edge case \
                 EC-005 / STORY-187 story-level EC-009)"
            );
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert_eq!(
                state.classified_protocol,
                Some(S7Protocol::Unclassified),
                "classified_protocol must remain Some(Unclassified) (F-12 ruling)"
            );
        }
    }

    // =========================================================================
    // P12-F-3: dissection must be bounded to the CURRENT TPKT frame's own payload
    // (`&frame[4..]`, `frame == &working[cursor..cursor + total]`) -- never to
    // `&working[cursor + 4..]`, which would (with no upper bound) leak bytes from
    // any SUBSEQUENT frame present later in the same `working` buffer/delivery into
    // the current frame's `tpkt_payload`/COTP-payload/S7comm-payload slice.
    // =========================================================================

    /// P12-F-3: a single `on_data` delivery contains TWO complete, back-to-back TPKT
    /// frames: first a DT frame whose classic S7comm Job header declares
    /// `param_length == 2` but has ZERO parameter bytes actually present WITHIN ITS
    /// OWN TPKT frame (the frame's declared TPKT `length` covers exactly the 10-byte
    /// common header, nothing more); second, a complete COTP CR frame.
    ///
    /// Under the correct implementation, the DT frame's dissection is bounded to its
    /// own frame -- `payload.len() == 10`, so `s7comm_bounds_ok` correctly fails
    /// (`declared_total == 12 > 10`) and exactly one T0814 is emitted. Under the
    /// `tpkt_payload = &working[cursor + 4..]` mutation (no upper bound), the DT
    /// frame's `tpkt_payload` would instead extend all the way to the end of
    /// `working`, absorbing the entire trailing CR frame's 7 bytes into what should
    /// have been a 10-byte payload -- `payload.len()` would become 17, which
    /// incorrectly PASSES the bounds check (`17 >= 12`), so the mutant emits ZERO
    /// findings instead of one.
    ///
    /// Traces: BC-2.21.009 postcondition 2; BC-2.20.013 postcondition 1a (frame
    /// boundary discipline).
    #[test]
    fn test_BC_2_21_009_dissection_bounded_to_own_tpkt_frame() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // Job header (exactly 10 bytes, no trailing parameter/data bytes) declaring
        // param_length=2, data_length=0 -- so the DT frame's own TPKT-declared
        // length covers only these 10 bytes, none of the (falsely) declared 2
        // parameter bytes.
        let header = classic_header_bytes(0x01, 0x0001, 0x0002, 0x0000);
        assert_eq!(header.len(), 10);
        let mut delivery = dt_frame(&header);
        delivery.extend_from_slice(&cr_frame());

        analyzer.on_data(flow_key.clone(), &delivery, 0, Direction::ClientToServer);

        assert_eq!(
            analyzer.findings.len(),
            1,
            "the DT frame's bounds-check failure (param_length=2 declared, 0 bytes \
             present WITHIN ITS OWN FRAME) must emit exactly one T0814, undiminished \
             by the trailing CR frame present later in the same delivery (P12-F-3)"
        );
        assert_malformed_header_t0814(&analyzer.findings[0], Direction::ClientToServer);
        assert_reason_specific_evidence(&analyzer.findings[0], "exceed available bytes");

        // The trailing CR frame must still have been walked and dispatched normally
        // (session-tracking state updated), confirming the cursor advanced correctly
        // past the DT frame using its OWN declared TPKT length, not consumed by an
        // over-wide tpkt_payload slice.
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.cr_observed_dir,
            Some(Direction::ClientToServer),
            "the trailing CR frame must still be walked and dispatched (its own \
             cr_observed_dir side effect observed), confirming the cursor advanced \
             past the DT frame by exactly its own declared TPKT length (P12-F-3)"
        );
    }

    /// P12-F-3 (mirror case): a single `on_data` delivery contains a DT frame with an
    /// EMPTY upper-layer payload (`protocol_id: None`, via `dt_frame_empty_payload()`
    /// -- `payload_offset == tpkt_payload.len()` exactly, within its OWN frame),
    /// immediately followed by a complete CR frame in the SAME delivery.
    ///
    /// Under the correct implementation, the DT frame's `tpkt_payload` is bounded to
    /// its own 3-byte COTP fixed part, so `header.protocol_id` correctly evaluates to
    /// `None` (no byte exists at `payload_offset` within this frame) and
    /// `classified_protocol` stays `None` (F-02: a `None`-protocol_id DT frame never
    /// classifies). Under the `tpkt_payload = &working[cursor + 4..]` mutation, the
    /// DT frame's `tpkt_payload` would instead leak into the trailing CR frame's
    /// bytes, so `tpkt_payload[payload_offset]` would read the CR frame's leading
    /// TPKT version byte (`0x03`) instead of being out-of-bounds -- producing
    /// `protocol_id: Some(0x03)`, which classifies the flow `Unclassified` instead of
    /// leaving it `None`.
    ///
    /// Traces: BC-2.21.002 postcondition 6; BC-2.20.010 (`protocol_id: None` no
    /// out-of-bounds read); BC-2.20.013 postcondition 1a (frame boundary discipline).
    #[test]
    fn test_BC_2_21_002_empty_dt_followed_by_frame_same_delivery_stays_unclassified() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        let mut delivery = dt_frame_empty_payload();
        delivery.extend_from_slice(&cr_frame());

        analyzer.on_data(flow_key.clone(), &delivery, 0, Direction::ClientToServer);

        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.classified_protocol, None,
            "a protocol_id: None DT frame (empty payload WITHIN ITS OWN FRAME) \
             followed by a trailing CR frame in the same delivery must leave \
             classified_protocol at None -- the DT frame's protocol_id evaluation \
             must never read into the trailing frame's bytes (P12-F-3, F-02)"
        );
        assert!(
            analyzer.findings.is_empty(),
            "neither the empty-payload DT frame nor the trailing CR frame emits any \
             finding"
        );
    }

    // =========================================================================
    // AC-187-013 (F-19, policy DF-CANONICAL-FRAME-HOLDOUT-001): a canonical,
    // independently-sourced classic S7comm Setup Communication frame pair.
    // =========================================================================

    /// AC-187-013 / F-19 / policy DF-CANONICAL-FRAME-HOLDOUT-001: a canonical,
    /// INDEPENDENTLY-SOURCED classic S7comm Setup Communication (function code
    /// `0xF0`) Job request, wrapped in real ISO-on-TCP framing (RFC 1006 TPKT
    /// header + standard class-0 COTP DT `02 F0 80`). Its companion Ack_Data
    /// response frame is covered by
    /// `test_BC_2_21_008_canonical_ack_data_setup_communication_response_on_data`
    /// below (split out per STORY-187 v1.3's per-BC test naming).
    ///
    /// **Primary source** (byte sequence taken verbatim): cnblogs,
    /// "西门子S7通讯协议引用整理" ("Siemens S7 Communication Protocol Reference
    /// Compilation"), <https://www.cnblogs.com/crcce-dncs/p/10659087.html> --
    /// Setup Communication request/response example.
    ///
    /// **Corroborating sources** (same field layout, different client-chosen PDU
    /// reference/PDU-size values -- confirming the layout recurs across
    /// independently-authored examples, not an artifact of the one primary source):
    /// - Yiqisoft blog, "PLC｜Golang 连接西门子Siemens S7/TCP 协议读取数据"
    ///   (2023-03-22), <https://www.yiqisoft.cn/blogs/IoT-Gateway/363.html>
    ///   (gos7-generated log: PDU reference `0x0400`, PDU size `480`).
    /// - Inductive Automation Support KB, "Loggers - Device Connections: Siemens",
    ///   <https://support.inductiveautomation.com/hc/en-us/articles/5497668078349-Loggers-Device-Connections-Siemens>
    ///   (PDU reference `0x0002`, PDU size `240`).
    ///
    /// The PDU reference value (`0xFFFF` in the primary source's example) is a
    /// client-chosen, connection-scoped sequence value, not a protocol constant --
    /// so the corroborating sources' differing PDU-reference/PDU-size values do not
    /// conflict with the primary source; they confirm the same FIELD LAYOUT
    /// (rosctr / pdu_reference / param_length / data_length / [error_class /
    /// error_code] / parameter block) recurs across independently-authored
    /// real-world examples.
    ///
    /// Per policy DF-CANONICAL-FRAME-HOLDOUT-001: the TPKT length field, the
    /// `02 F0 80` COTP DT header bytes, and the S7comm common-header/Setup-
    /// Communication-parameter bytes below are NOT derived from this project's own
    /// BCs, ADR-014, or any other project artifact -- they are copied verbatim from
    /// the primary source cited above.
    ///
    /// These publicly posted wire-capture byte examples are used here strictly as
    /// TEST-VECTOR SOURCES, as permitted by the ADR-014 Decision 4 reconciliation
    /// note (2026-09-24, DF-CANONICAL-FRAME-HOLDOUT-001); parser design and field
    /// semantics continue to derive from Decision 4's prose sources and permitted
    /// design references, never from these wire-capture test vectors.
    ///
    /// Traces: BC-2.21.002 postcondition 3, BC-2.21.006; AC-187-013.
    #[test]
    fn test_BC_2_21_006_canonical_setup_communication_job_frame_on_data() {
        // Canonical Setup Communication Job request (25 bytes total): RFC 1006 TPKT
        // header (03 00 00 19) + class-0 COTP DT (02 F0 80) + classic S7comm Job
        // header/params, verbatim from the cnblogs primary source cited above.
        const JOB_FRAME: [u8; 25] = [
            0x03, 0x00, 0x00, 0x19, 0x02, 0xF0, 0x80, 0x32, 0x01, 0x00, 0x00, 0xFF, 0xFF, 0x00,
            0x08, 0x00, 0x00, 0xF0, 0x00, 0x00, 0x01, 0x00, 0x01, 0x07, 0x80,
        ];

        // --- on_data-level assertions: the Job frame dispatches cleanly. ---
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        analyzer.on_data(flow_key.clone(), &JOB_FRAME, 0, Direction::ClientToServer);

        assert!(
            analyzer.findings.is_empty(),
            "the canonical Setup Communication Job request frame must produce no \
             findings -- it is well-formed under the BC-2.21.009 bounds check \
             (AC-187-013)"
        );
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.classified_protocol,
            Some(S7Protocol::Classic),
            "the canonical Job frame's protocol_id (0x32) must sticky-classify the \
             flow Classic (AC-187-013, BC-2.21.002 postcondition 3)"
        );

        // --- Direct parse_s7comm_header assertion on the isolated S7 slice. ---
        // TPKT (4 bytes) + COTP DT fixed part (3 bytes) = 7-byte prefix.
        let job_s7 = &JOB_FRAME[7..];
        let job_header =
            parse_s7comm_header(job_s7).expect("canonical Job header must parse (AC-187-013)");
        assert_eq!(job_header.rosctr, Rosctr::Job);
        assert_eq!(
            job_header.pdu_reference, 0xFFFF,
            "PDU reference is client-chosen (0xFFFF in the primary source's example) \
             -- not a protocol constant"
        );
        assert_eq!(job_header.param_length, 8);
        assert_eq!(job_header.data_length, 0);
        assert_eq!(job_header.header_len, 10);
        assert_eq!(job_header.error_class, None);
        assert_eq!(job_header.error_code, None);
    }

    /// AC-187-013 / F-19 / canonical-frame holdout ruling
    /// (DF-CANONICAL-FRAME-HOLDOUT-001, human-ratified 2026-09-24): the companion
    /// canonical, independently-sourced classic S7comm Setup Communication
    /// **Ack_Data (`0x03`) response** frame, wrapped in the same RFC 1006 TPKT +
    /// standard class-0 COTP DT (`02 F0 80`) framing as
    /// `test_BC_2_21_006_canonical_setup_communication_job_frame_on_data` above.
    ///
    /// **Primary source** (byte sequence taken verbatim): cnblogs,
    /// "西门子S7通讯协议引用整理" ("Siemens S7 Communication Protocol Reference
    /// Compilation"), <https://www.cnblogs.com/crcce-dncs/p/10659087.html> -- full
    /// 27-byte frame `03 00 00 1B 02 F0 80 32 03 00 00 FF FF 00 08 00 00 00 00 F0
    /// 00 00 01 00 01 00 F0`.
    ///
    /// **Corroborating sources** (same field layout -- an Ack_Data Setup
    /// Communication response ALSO carries Error Class (`data[10]`) / Error Code
    /// (`data[11]`), with the parameter block starting at `data[12]`, exactly as
    /// documented in BC-2.21.008's Canonical Test Vectors -- these are byte-level
    /// TEST-VECTOR corroboration only):
    /// - Yiqisoft blog (2023-03-22), <https://www.yiqisoft.cn/blogs/IoT-Gateway/363.html>
    /// - Inductive Automation Support KB, "Loggers - Device Connections: Siemens"
    ///
    /// **Field-semantics grounding** (separate from the test-vector corroboration
    /// above; see ADR-014 Decision 4 / Decision 9 notes for the full citations):
    /// Kleinmann & Wool 2014 (JDFSL 9(2), Fig. 2) is a PROSE source attesting the
    /// Ack_Data (`0x03`) 12-byte header only. The Ack (`0x02`) 12-byte layout and the
    /// 1-byte `error_class`/`error_code` split rest on Decision 4's PERMITTED DESIGN
    /// REFERENCES -- cisagov/icsnpp-s7comm and gijzelaerr/python-snap7 (kprovost/libs7comm
    /// consistent in aggregate) -- not on Kleinmann & Wool, which does not attest Ack.
    ///
    /// ## Resolution of the round-3 discrepancy (DO NOT relitigate by relaxing
    /// ## assertions)
    ///
    /// The primary source's Ack_Data response byte layout is only self-consistent
    /// under a 12-byte Ack_Data header -- i.e. `rosctr == AckData` carrying
    /// `error_class`/`error_code` at `data[10]`/`data[11]`, with the
    /// Setup-Communication response parameter block (`F0 00 00 01 00 01 00 F0`:
    /// function code `0xF0` + reserved + max-AMQ-calling + max-AMQ-called +
    /// PDU-length, exactly matching the declared `param_length == 8`) beginning at
    /// byte 12. This round-3 discrepancy (an earlier revision of this test asserted
    /// this same 12-byte shape as "EXPECTED TO FAIL") is now the RATIFIED,
    /// human-ratified behavior per DF-CANONICAL-FRAME-HOLDOUT-001 (2026-09-24):
    /// BC-2.21.008 / `parse_s7comm_header` extends the 12-byte header
    /// (`error_class`/`error_code`) to BOTH `rosctr == Ack` (`0x02`) AND
    /// `rosctr == AckData` (`0x03`) -- the assertions below are ordinary, load-bearing
    /// GREEN-gate assertions, not a documented discrepancy.
    ///
    /// These publicly posted wire-capture byte examples are used here strictly as
    /// TEST-VECTOR SOURCES, as permitted by the ADR-014 Decision 4 reconciliation
    /// note (2026-09-24, DF-CANONICAL-FRAME-HOLDOUT-001); parser design and field
    /// semantics continue to derive from Decision 4's prose sources and permitted
    /// design references (see the field-semantics grounding note above), never from
    /// these wire-capture test vectors.
    ///
    /// Traces: BC-2.21.002 postcondition 3, BC-2.21.008 postconditions 1-2;
    /// AC-187-013.
    #[test]
    fn test_BC_2_21_008_canonical_ack_data_setup_communication_response_on_data() {
        // Canonical Setup Communication Ack_Data response (27 bytes total),
        // verbatim from the cnblogs primary source cited above.
        const ACK_DATA_FRAME: [u8; 27] = [
            0x03, 0x00, 0x00, 0x1B, 0x02, 0xF0, 0x80, 0x32, 0x03, 0x00, 0x00, 0xFF, 0xFF, 0x00,
            0x08, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0xF0,
        ];

        // --- on_data-level assertions: the Ack_Data frame dispatches cleanly. ---
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // Ack_Data is the server's response frame, so it is fed as ServerToClient
        // (the response direction) -- corrected from an earlier revision that fed
        // it as ClientToServer; classification and the findings assertion are
        // direction-independent, so this does not change what the test proves.
        analyzer.on_data(
            flow_key.clone(),
            &ACK_DATA_FRAME,
            0,
            Direction::ServerToClient,
        );

        assert!(
            analyzer.findings.is_empty(),
            "the canonical Setup Communication Ack_Data response frame must produce \
             no findings -- it is well-formed under the corrected 12-byte-header \
             BC-2.21.009 bounds check (AC-187-013)"
        );
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.classified_protocol,
            Some(S7Protocol::Classic),
            "the canonical Ack_Data frame's protocol_id (0x32) must sticky-classify \
             the flow Classic (AC-187-013, BC-2.21.002 postcondition 3)"
        );

        // --- Direct parse_s7comm_header assertion on the isolated S7 slice. ---
        // TPKT (4 bytes) + COTP DT fixed part (3 bytes) = 7-byte prefix.
        let ack_data_s7 = &ACK_DATA_FRAME[7..];
        let ack_data_header = parse_s7comm_header(ack_data_s7)
            .expect("canonical Ack_Data header must parse (AC-187-013)");
        assert_eq!(ack_data_header.rosctr, Rosctr::AckData);
        assert_eq!(
            ack_data_header.pdu_reference, 0xFFFF,
            "PDU reference is client-chosen (0xFFFF in the primary source's example) \
             -- not a protocol constant"
        );
        assert_eq!(ack_data_header.param_length, 8);
        assert_eq!(ack_data_header.data_length, 0);
        assert_eq!(
            ack_data_header.header_len, 12,
            "Ack_Data (0x03) requires the 12-byte header per the 2026-09-24 \
             canonical-frame holdout ruling (DF-CANONICAL-FRAME-HOLDOUT-001) -- the \
             real-world parameter block only aligns at byte 12, not byte 10 (the \
             superseded v1.0/v1.1 assumption)"
        );
        assert_eq!(
            ack_data_header.error_class,
            Some(0x00),
            "error_class must be extracted from data[10] for Ack_Data \
             (BC-2.21.008 postcondition 2, DF-CANONICAL-FRAME-HOLDOUT-001)"
        );
        assert_eq!(
            ack_data_header.error_code,
            Some(0x00),
            "error_code must be extracted from data[11] for Ack_Data \
             (BC-2.21.008 postcondition 2, DF-CANONICAL-FRAME-HOLDOUT-001)"
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
        assert_reason_specific_evidence(&analyzer.findings[0], "header too short");
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

    /// AC-187-006 (v1.1): per-direction dedup verified independently for the s2c
    /// direction -- a c2s malformed-length occurrence and an independent s2c
    /// malformed-length occurrence on the SAME flow must each emit their own T0814
    /// (two total), and a REPEATED s2c occurrence must not emit a third.
    ///
    /// Traces: BC-2.21.004 postcondition 4, edge case EC-004; BC-2.21.001 invariant 2
    /// (per-direction dedup independence); AC-187-006.
    #[test]
    fn test_BC_2_21_004_len_shorter_than_10_emits_t0814_once_s2c() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();
        let frame = dt_frame(&[0x32]); // data.len() == 1 inside parse_s7comm_header

        // c2s malformed occurrence.
        analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);
        assert_eq!(
            analyzer.findings.len(),
            1,
            "the first c2s malformed-length occurrence must emit one T0814"
        );

        // First s2c malformed occurrence -- independent of the c2s dedup flag.
        analyzer.on_data(flow_key.clone(), &frame, 1, Direction::ServerToClient);
        assert_eq!(
            analyzer.findings.len(),
            2,
            "the first s2c malformed-length occurrence must emit its OWN T0814, \
             independently of the c2s direction's dedup flag already being set \
             (BC-2.21.001 invariant 2)"
        );
        assert_malformed_header_t0814(&analyzer.findings[1], Direction::ServerToClient);
        assert_reason_specific_evidence(&analyzer.findings[1], "header too short");
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert!(
                state.malformed_header_reported_s2c,
                "malformed_header_reported_s2c must be set after the first s2c \
                 occurrence"
            );
        }

        // A repeated s2c occurrence must not re-emit.
        analyzer.on_data(flow_key.clone(), &frame, 2, Direction::ServerToClient);
        assert_eq!(
            analyzer.findings.len(),
            2,
            "a repeated s2c malformed-length occurrence must NOT emit a third T0814 \
             (BC-2.21.004 edge case EC-004, s2c direction)"
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
    // BC-2.21.006: `parse_s7comm_header` extracts the common header (Job/Userdata
    // happy path; Ack/Ack_Data -> BC-2.21.008).
    // =========================================================================

    /// AC-187-008: the two BC-2.21.006 canonical test vectors (Job, Userdata) each
    /// extract the exact expected `S7commHeader` -- full structural equality, not
    /// merely `is_some()`. Ack_Data (`0x03`) is NOT part of this BC's happy path as
    /// of the 2026-09-24 canonical-frame holdout ruling
    /// (DF-CANONICAL-FRAME-HOLDOUT-001) -- it now requires the 12-byte header
    /// covered by BC-2.21.008 (see
    /// `test_BC_2_21_008_ack_data_12_byte_header_and_error_fields`); a 10-byte
    /// Ack_Data header is truncated and returns `None` (BC-2.21.006 v1.1's
    /// Canonical Test Vectors note).
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

        // Ack_Data at 10 bytes (the pre-ruling v1.0/v1.1 "happy path" shape) is now
        // truncated -- it requires the 12-byte header (BC-2.21.008).
        let ack_data_10_bytes = [0x32u8, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x04];
        assert_eq!(
            parse_s7comm_header(&ack_data_10_bytes),
            None,
            "a 10-byte Ack_Data header must return None -- Ack_Data (0x03) is no \
             longer part of BC-2.21.006's 10-byte happy-path group as of the \
             2026-09-24 canonical-frame holdout ruling (DF-CANONICAL-FRAME-HOLDOUT-001); \
             see BC-2.21.008 for the 12-byte Ack_Data shape"
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

    /// AC-187-009 (v1.1): per-direction dedup verified independently for the s2c
    /// direction, using an unrecognized-ROSCTR malformed condition (BC-2.21.007)
    /// rather than BC-2.21.004's too-short condition -- mirrors
    /// `test_BC_2_21_004_len_shorter_than_10_emits_t0814_once_s2c`'s c2s-then-s2c
    /// pattern.
    ///
    /// Traces: BC-2.21.007 postcondition 3; AC-187-009.
    #[test]
    fn test_BC_2_21_007_unrecognized_rosctr_emits_t0814_once_s2c() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();
        let unrecognized = dt_frame(&[0x32, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00]);

        // c2s malformed occurrence.
        analyzer.on_data(
            flow_key.clone(),
            &unrecognized,
            0,
            Direction::ClientToServer,
        );
        assert_eq!(
            analyzer.findings.len(),
            1,
            "the first c2s unrecognized-ROSCTR occurrence must emit one T0814"
        );
        assert_reason_specific_evidence(&analyzer.findings[0], "unrecognized ROSCTR");

        // First s2c malformed occurrence -- independent of the c2s dedup flag.
        analyzer.on_data(
            flow_key.clone(),
            &unrecognized,
            1,
            Direction::ServerToClient,
        );
        assert_eq!(
            analyzer.findings.len(),
            2,
            "the first s2c unrecognized-ROSCTR occurrence must emit its OWN T0814, \
             independently of the c2s direction's dedup flag already being set"
        );
        assert_malformed_header_t0814(&analyzer.findings[1], Direction::ServerToClient);
        assert_reason_specific_evidence(&analyzer.findings[1], "unrecognized ROSCTR");
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert!(state.malformed_header_reported_s2c);
        }

        // A repeated s2c occurrence must not re-emit.
        analyzer.on_data(
            flow_key.clone(),
            &unrecognized,
            2,
            Direction::ServerToClient,
        );
        assert_eq!(
            analyzer.findings.len(),
            2,
            "a repeated s2c unrecognized-ROSCTR occurrence must NOT emit a third \
             T0814"
        );
    }

    /// AC-187-009 (v1.1, F-08): `parse_s7comm_header` returns `Some` iff `data[1]` is
    /// one of the four recognized ROSCTR values (`0x01`/`0x02`/`0x03`/`0x07`),
    /// exhaustively verified over all 256 possible `data[1]` byte values at a
    /// length (12 bytes) sufficient for EVERY recognized ROSCTR, including the
    /// Ack/Ack_Data 12-byte minimum -- i.e. this test isolates BC-2.21.007's own
    /// "recognized vs. unrecognized ROSCTR" concern from BC-2.21.006/008's
    /// length-conditional concern (covered separately by
    /// `proptest_bc_2_21_006_008_some_iff_rosctr_and_length_conditional` below). No
    /// value panics.
    ///
    /// Traces: BC-2.21.007 postconditions 1-2, Canonical Test Vectors; AC-187-009.
    #[test]
    fn proptest_bc_2_21_007_rosctr_byte_totality_over_all_256_values() {
        for rosctr in 0u8..=255u8 {
            let mut data = classic_header_bytes(rosctr, 1, 0, 0);
            data.push(0x00); // error_class (only consumed when rosctr is Ack/AckData)
            data.push(0x00); // error_code
            assert_eq!(data.len(), 12);
            let result = parse_s7comm_header(&data);
            let expected_some = matches!(rosctr, 0x01 | 0x02 | 0x03 | 0x07);
            assert_eq!(
                result.is_some(),
                expected_some,
                "rosctr={rosctr:#04x}, len=12: parse_s7comm_header must return Some \
                 iff rosctr is one of the four recognized ROSCTR values \
                 (0x01/0x02/0x03/0x07) -- every other byte is unrecognized \
                 regardless of length (BC-2.21.007 postconditions 1-2), verified \
                 across all 256 possible u8 values"
            );
        }
    }

    /// AC-187-009 / F-08 (corrected 2026-09-24 per the canonical-frame holdout
    /// ruling, DF-CANONICAL-FRAME-HOLDOUT-001): the joint, LENGTH-CONDITIONAL
    /// totality across BC-2.21.006/007/008, exhaustively over all 256 possible
    /// `data[1]` byte values AND a representative span of lengths
    /// (`[9, 10, 11, 12, 13]`, straddling both the 10-byte Job/Userdata minimum and
    /// the 12-byte Ack/Ack_Data minimum). `parse_s7comm_header(data)` returns
    /// `Some` **iff** (`data[1] ∈ {0x01, 0x07}` and `data.len() >= 10`) **or**
    /// (`data[1] ∈ {0x02, 0x03}` and `data.len() >= 12`) -- i.e. Ack_Data (`0x03`)
    /// is grouped with Ack (`0x02`) under the 12-byte minimum, NOT with Job/Userdata
    /// under the 10-byte minimum (the superseded v1.0/v1.1/v1.2 assumption). Every
    /// other `data[1]` byte value returns `None` at every length, and no value
    /// panics.
    ///
    /// Traces: BC-2.21.006 postcondition 1, BC-2.21.007 postconditions 1-2,
    /// BC-2.21.008 postconditions 1-2; AC-187-009.
    #[test]
    fn proptest_bc_2_21_006_008_some_iff_rosctr_and_length_conditional() {
        for rosctr in 0u8..=255u8 {
            for len in 9usize..=13usize {
                let mut data = classic_header_bytes(rosctr, 1, 0, 0); // 10 bytes
                if len < data.len() {
                    data.truncate(len);
                } else {
                    while data.len() < len {
                        data.push(0x00);
                    }
                }
                assert_eq!(data.len(), len);

                let result = parse_s7comm_header(&data);
                let job_or_userdata = matches!(rosctr, 0x01 | 0x07);
                let ack_or_ack_data = matches!(rosctr, 0x02 | 0x03);
                let expected_some =
                    (len >= 10 && job_or_userdata) || (len >= 12 && ack_or_ack_data);
                assert_eq!(
                    result.is_some(),
                    expected_some,
                    "rosctr={rosctr:#04x}, len={len}: parse_s7comm_header must return \
                     Some iff (rosctr in {{0x01, 0x07}} and len>=10) or (rosctr in \
                     {{0x02, 0x03}} and len>=12) -- length-conditional joint totality \
                     across BC-2.21.006/007/008, corrected 2026-09-24 per the \
                     canonical-frame holdout ruling (DF-CANONICAL-FRAME-HOLDOUT-001): \
                     Ack_Data (0x03) is grouped with Ack (0x02) under the 12-byte \
                     minimum, not with Job/Userdata under the 10-byte minimum"
                );
            }
        }
    }

    // =========================================================================
    // BC-2.21.008: `parse_s7comm_header` for ROSCTR=Ack (0x02) AND Ack_Data (0x03)
    // both require 12 bytes (2026-09-24 canonical-frame holdout ruling,
    // DF-CANONICAL-FRAME-HOLDOUT-001).
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
        // Uses DISTINCT, non-zero error_class/error_code values (0x81/0x04, per
        // P12-F-2) rather than Some(0)/Some(0) -- a same-value pair cannot
        // distinguish a correct data[10]/data[11] extraction from a mutation that
        // swaps the two offsets, or from a mutation that hard-codes `Some(0)` for
        // either field.
        let twelve = ack_header_bytes(1, 0, 0, 0x81, 0x04);
        assert_eq!(twelve.len(), 12);
        assert_eq!(
            parse_s7comm_header(&twelve),
            Some(S7commHeader {
                rosctr: Rosctr::Ack,
                pdu_reference: 1,
                param_length: 0,
                data_length: 0,
                error_class: Some(0x81),
                error_code: Some(0x04),
                header_len: 12,
            }),
            "12-byte Ack (canonical vector, minimal happy path) must extract the exact \
             expected S7commHeader with error_class == data[10] == Some(0x81) and \
             error_code == data[11] == Some(0x04) -- distinct values (P12-F-2) so an \
             offset swap or a hard-coded Some(0) is caught"
        );
    }

    /// AC-187-010 (2026-09-24 canonical-frame holdout ruling,
    /// DF-CANONICAL-FRAME-HOLDOUT-001): the mirror-image test of
    /// `test_BC_2_21_008_ack_rosctr_12_byte_minimum_and_error_fields` above, for
    /// Ack_Data (`0x03`) instead of Ack (`0x02`) -- both ROSCTR values require the
    /// SAME 12-byte header shape. Uses BC-2.21.008's own Ack_Data canonical test
    /// vectors: 10 bytes (`None`, truncated), 11 bytes (`None`, truncated), and the
    /// cnblogs-sourced Setup Communication response header slice at 12 bytes
    /// (`Some`, exact structural equality including `error_class`/`error_code`).
    ///
    /// Traces: BC-2.21.008 postconditions 1-2, Canonical Test Vectors; AC-187-010.
    #[test]
    fn test_BC_2_21_008_ack_data_12_byte_header_and_error_fields() {
        // 10 bytes: common header only, no error class/code -- truncated Ack_Data.
        let ten = [0x32u8, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            parse_s7comm_header(&ten),
            None,
            "10-byte Ack_Data (BC-2.21.008 canonical vector) must return None -- \
             truncated (DF-CANONICAL-FRAME-HOLDOUT-001: Ack_Data is NOT a \
             10-byte-only ROSCTR)"
        );

        // 11 bytes: one byte short of the 12-byte Ack_Data minimum.
        let eleven = [
            0x32u8, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(
            parse_s7comm_header(&eleven),
            None,
            "11-byte Ack_Data (BC-2.21.008 canonical vector) must return None -- one \
             byte short"
        );

        // 12 bytes exactly: the cnblogs Setup Communication response header slice
        // (BC-2.21.008's canonical happy-path Ack_Data vector) -- built via the
        // ack_data_header_bytes helper for structural equality with a distinct set
        // of field values (pdu_reference/param_length) than the 10/11-byte vectors
        // above.
        // Distinct, non-zero error_class/error_code (0x81/0x04, per P12-F-2) --
        // mirrors the Ack-rosctr test's rationale above.
        let twelve = ack_data_header_bytes(0xFFFF, 8, 0, 0x81, 0x04);
        assert_eq!(twelve.len(), 12);
        assert_eq!(
            parse_s7comm_header(&twelve),
            Some(S7commHeader {
                rosctr: Rosctr::AckData,
                pdu_reference: 0xFFFF,
                param_length: 8,
                data_length: 0,
                error_class: Some(0x81),
                error_code: Some(0x04),
                header_len: 12,
            }),
            "12-byte Ack_Data (BC-2.21.008 canonical vector, Setup Communication \
             response shape) must extract the exact expected S7commHeader with \
             error_class == data[10] == Some(0x81), error_code == data[11] == \
             Some(0x04), and header_len == 12 (DF-CANONICAL-FRAME-HOLDOUT-001; \
             P12-F-2: distinct values catch an offset swap or a hard-coded Some(0))"
        );
    }

    /// AC-187-010 (2026-09-24 canonical-frame holdout ruling,
    /// DF-CANONICAL-FRAME-HOLDOUT-001), BC-2.21.008 edge cases EC-005/EC-006: a
    /// dedicated, direct-call regression test isolating the truncated-Ack_Data
    /// `None` behavior at exactly 10 and 11 bytes -- distinct from the full
    /// canonical-vector-plus-happy-path coverage in
    /// `test_BC_2_21_008_ack_data_12_byte_header_and_error_fields` above.
    ///
    /// Traces: BC-2.21.008 postcondition 1, edge cases EC-005/EC-006; AC-187-010.
    #[test]
    fn test_BC_2_21_008_truncated_ack_data_returns_none() {
        // EC-005: data[1] == 0x03 (Ack_Data), data.len() == 10 (only the common
        // header present).
        let ten = ack_data_header_bytes(1, 0, 0, 0x00, 0x00);
        let ten_truncated = &ten[..10];
        assert_eq!(
            parse_s7comm_header(ten_truncated),
            None,
            "a 10-byte Ack_Data header (only the common header present) must return \
             None (BC-2.21.008 edge case EC-005)"
        );

        // EC-006: data[1] == 0x03 (Ack_Data), data.len() == 11 (one byte short of
        // the 12-byte minimum).
        let eleven = ack_data_header_bytes(1, 0, 0, 0x00, 0x00);
        let eleven_truncated = &eleven[..11];
        assert_eq!(
            parse_s7comm_header(eleven_truncated),
            None,
            "an 11-byte Ack_Data header (one byte short of the 12-byte minimum) must \
             return None (BC-2.21.008 edge case EC-006)"
        );
    }

    /// BC-2.21.008 postcondition 3: `error_class`/`error_code` are `Some` ONLY when
    /// `rosctr ∈ {Ack, AckData}` -- cross-checked here against Job AND Userdata
    /// headers, both of which must have both fields `None` (the Job case is already
    /// asserted structurally within `test_BC_2_21_006_common_header_field_extraction`;
    /// restated here, alongside Userdata, as a dedicated, BC-2.21.008-scoped
    /// assertion for direct traceability). Renamed from
    /// `test_BC_2_21_008_error_fields_none_for_non_ack_rosctr` per the 2026-09-24
    /// canonical-frame holdout ruling (DF-CANONICAL-FRAME-HOLDOUT-001) -- the old
    /// name's "non-Ack" framing is now misleading, since Ack_Data (also
    /// "non-Ack") no longer has `None` error fields.
    ///
    /// Traces: BC-2.21.008 postcondition 3, invariant 2.
    #[test]
    fn test_BC_2_21_008_error_fields_none_for_job_and_userdata_rosctr() {
        let job = [0x32u8, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00];
        let job_header = parse_s7comm_header(&job).expect("valid Job header must parse");
        assert_eq!(
            job_header.error_class, None,
            "error_class must be None for Job (BC-2.21.008 postcondition 3)"
        );
        assert_eq!(
            job_header.error_code, None,
            "error_code must be None for Job (BC-2.21.008 postcondition 3)"
        );

        let userdata = [0x32u8, 0x07, 0x00, 0x00, 0x00, 0x05, 0x00, 0x08, 0x00, 0x00];
        let userdata_header =
            parse_s7comm_header(&userdata).expect("valid Userdata header must parse");
        assert_eq!(
            userdata_header.error_class, None,
            "error_class must be None for Userdata (BC-2.21.008 postcondition 3)"
        );
        assert_eq!(
            userdata_header.error_code, None,
            "error_code must be None for Userdata (BC-2.21.008 postcondition 3)"
        );
    }

    /// AC-187-010 (F-21b): an `on_data`-delivered classic-S7comm DT frame whose
    /// ROSCTR byte (`data[1]`) is `0x02` (Ack) and whose total S7comm payload length
    /// is exactly 10 bytes, and separately a variant whose total payload length is
    /// exactly 11 bytes (both one and two bytes short of the 12-byte Ack minimum),
    /// must each cause `parse_s7comm_header` to return `None` and emit exactly one
    /// T0814 `Finding` with evidence text identifying the reason as "truncated Ack
    /// header" -- the exact string `classify_malformed_header_reason` produces for
    /// `data[1] == 0x02` (distinct from the "header too short", "unrecognized
    /// ROSCTR", and "truncated Ack_Data header" reasons; per pass-3 F-29, the
    /// assertion below uses the full "truncated Ack header" substring rather than
    /// the shorter "truncated Ack" prefix, which is also a substring of
    /// "truncated Ack_Data header" and so would not actually distinguish the two
    /// reasons). Each case uses a fresh analyzer/flow, so no priming frame is needed: the
    /// frame's own `protocol_id: Some(0x32)` sticky-classifies the flow Classic on
    /// this very frame (AC-187-004's "or, by this very frame's own
    /// first-classification, becomes" clause), and dissection then proceeds.
    ///
    /// Traces: BC-2.21.008 postcondition 1; AC-187-010.
    #[test]
    fn test_BC_2_21_008_truncated_ack_on_data_emits_t0814_once() {
        // 10-byte case: the bare common header only (rosctr = 0x02, Ack) -- two
        // bytes short of the 12-byte Ack minimum.
        {
            let mut analyzer = S7commAnalyzer::new();
            let flow_key = flow_key_default();
            let ten = classic_header_bytes(0x02, 1, 0, 0);
            assert_eq!(ten.len(), 10);
            let frame = dt_frame(&ten);
            analyzer.on_data(flow_key, &frame, 0, Direction::ClientToServer);

            assert_eq!(
                analyzer.findings.len(),
                1,
                "a 10-byte Ack (rosctr=0x02) DT frame must emit exactly one T0814 \
                 (AC-187-010, BC-2.21.008 postcondition 1)"
            );
            assert_malformed_header_t0814(&analyzer.findings[0], Direction::ClientToServer);
            assert_reason_specific_evidence(&analyzer.findings[0], "truncated Ack header");
        }

        // 11-byte case: one byte short of the 12-byte Ack minimum.
        {
            let mut analyzer = S7commAnalyzer::new();
            let flow_key = flow_key_default();
            let mut eleven = classic_header_bytes(0x02, 1, 0, 0);
            eleven.push(0x00);
            assert_eq!(eleven.len(), 11);
            let frame = dt_frame(&eleven);
            analyzer.on_data(flow_key, &frame, 0, Direction::ClientToServer);

            assert_eq!(
                analyzer.findings.len(),
                1,
                "an 11-byte Ack (rosctr=0x02) DT frame must emit exactly one T0814 \
                 (AC-187-010, BC-2.21.008 postcondition 1)"
            );
            assert_malformed_header_t0814(&analyzer.findings[0], Direction::ClientToServer);
            assert_reason_specific_evidence(&analyzer.findings[0], "truncated Ack header");
        }
    }

    /// AC-187-010 (2026-09-24 canonical-frame holdout ruling,
    /// DF-CANONICAL-FRAME-HOLDOUT-001): the mirror-image `on_data`-level test of
    /// `test_BC_2_21_008_truncated_ack_on_data_emits_t0814_once` above, for
    /// Ack_Data (`0x03`) instead of Ack (`0x02`) -- a 10-byte and, separately, an
    /// 11-byte Ack_Data DT frame must each emit exactly one T0814 `Finding` with
    /// evidence text identifying the reason as "truncated Ack_Data", DISTINCT from
    /// the "truncated Ack" reason (the assertion below requires the evidence to
    /// contain the full "truncated Ack_Data" substring, so an implementation that
    /// mistakenly reused the plain "truncated Ack" string for this ROSCTR would
    /// fail this test). Each case uses a fresh analyzer/flow: the frame's own
    /// `protocol_id: Some(0x32)` sticky-classifies the flow Classic on this very
    /// frame (AC-187-004's "or, by this very frame's own first-classification,
    /// becomes" clause), and dissection then proceeds.
    ///
    /// Traces: BC-2.21.008 postcondition 1, edge cases EC-005/EC-006; AC-187-010.
    #[test]
    fn test_BC_2_21_008_truncated_ack_data_on_data_emits_t0814_once() {
        // 10-byte case: the bare common header only (rosctr = 0x03, Ack_Data) --
        // two bytes short of the 12-byte Ack_Data minimum.
        {
            let mut analyzer = S7commAnalyzer::new();
            let flow_key = flow_key_default();
            let ten = classic_header_bytes(0x03, 1, 0, 0);
            assert_eq!(ten.len(), 10);
            let frame = dt_frame(&ten);
            analyzer.on_data(flow_key, &frame, 0, Direction::ClientToServer);

            assert_eq!(
                analyzer.findings.len(),
                1,
                "a 10-byte Ack_Data (rosctr=0x03) DT frame must emit exactly one \
                 T0814 (AC-187-010, BC-2.21.008 postcondition 1)"
            );
            assert_malformed_header_t0814(&analyzer.findings[0], Direction::ClientToServer);
            assert_reason_specific_evidence(&analyzer.findings[0], "truncated Ack_Data");
        }

        // 11-byte case: one byte short of the 12-byte Ack_Data minimum.
        {
            let mut analyzer = S7commAnalyzer::new();
            let flow_key = flow_key_default();
            let mut eleven = classic_header_bytes(0x03, 1, 0, 0);
            eleven.push(0x00);
            assert_eq!(eleven.len(), 11);
            let frame = dt_frame(&eleven);
            analyzer.on_data(flow_key, &frame, 0, Direction::ClientToServer);

            assert_eq!(
                analyzer.findings.len(),
                1,
                "an 11-byte Ack_Data (rosctr=0x03) DT frame must emit exactly one \
                 T0814 (AC-187-010, BC-2.21.008 postcondition 1)"
            );
            assert_malformed_header_t0814(&analyzer.findings[0], Direction::ClientToServer);
            assert_reason_specific_evidence(&analyzer.findings[0], "truncated Ack_Data");
        }
    }

    // =========================================================================
    // BC-2.21.009: declared param_length/data_length are bounds-checked against
    // remaining bytes before parameter/data block access.
    // =========================================================================

    /// AC-187-011: a header whose declared `param_length`/`data_length` exceed the
    /// bytes actually remaining (canonical vector `2 / 0 / 1` -- one byte short) is
    /// treated as malformed: one T0814 per flow direction (sharing the dedup flag with
    /// BC-2.21.004/007/008), and no out-of-bounds slice is ever attempted (proven
    /// indirectly here by the absence of a panic).
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
        assert_reason_specific_evidence(&analyzer.findings[0], "exceed available bytes");
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.malformed_header_reported_c2s,
            "malformed_header_reported_c2s must be set (BC-2.21.009 postcondition 2 \
             shares the dedup flag with BC-2.21.004/007/008)"
        );
    }

    /// AC-187-011 (v1.1): per-direction dedup verified independently for the s2c
    /// direction, using a bounds-check (declared-length-exceeds-available-bytes)
    /// malformed condition (BC-2.21.009) -- mirrors the c2s-then-s2c pattern used by
    /// `test_BC_2_21_004_len_shorter_than_10_emits_t0814_once_s2c` and
    /// `test_BC_2_21_007_unrecognized_rosctr_emits_t0814_once_s2c`.
    ///
    /// Traces: BC-2.21.009 postcondition 2; AC-187-011.
    #[test]
    fn test_BC_2_21_009_bounds_check_dedup_s2c() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        let mut header = classic_header_bytes(0x01, 0x0001, 0x0002, 0x0000);
        header.push(0xAA); // only 1 of the declared 2 parameter bytes present
        let frame = dt_frame(&header);

        // c2s bounds-check-failure occurrence.
        analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);
        assert_eq!(
            analyzer.findings.len(),
            1,
            "the first c2s bounds-check-failure occurrence must emit one T0814"
        );

        // First s2c bounds-check-failure occurrence -- independent of the c2s dedup
        // flag.
        analyzer.on_data(flow_key.clone(), &frame, 1, Direction::ServerToClient);
        assert_eq!(
            analyzer.findings.len(),
            2,
            "the first s2c bounds-check-failure occurrence must emit its OWN T0814, \
             independently of the c2s direction's dedup flag already being set"
        );
        assert_malformed_header_t0814(&analyzer.findings[1], Direction::ServerToClient);
        assert_reason_specific_evidence(&analyzer.findings[1], "exceed available bytes");
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert!(state.malformed_header_reported_s2c);
        }

        // A repeated s2c occurrence must not re-emit.
        analyzer.on_data(flow_key.clone(), &frame, 2, Direction::ServerToClient);
        assert_eq!(
            analyzer.findings.len(),
            2,
            "a repeated s2c bounds-check-failure occurrence must NOT emit a third \
             T0814"
        );
    }

    /// F-14 / AC-187-011: the extracted pure helper `s7comm_bounds_ok` (BC-2.21.009)
    /// must agree with `dispatch_classic_s7comm`'s bounds decision: it returns `true`
    /// exactly when `data_len >= header.header_len + header.param_length as usize +
    /// header.data_length as usize`, callable directly (independent of the effectful
    /// `on_data` call site) so the VP-051 Kani harness can exercise it too.
    ///
    /// Traces: BC-2.21.009 postcondition 1, invariant 1; STORY-187 VP-051 Kani
    /// Obligation note (F-14).
    #[test]
    fn test_BC_2_21_009_s7comm_bounds_ok_helper_matches_bounds_decision() {
        // Exact match: header_len=10, param_length=2, data_length=0, data_len=12.
        let header = parse_s7comm_header(&classic_header_bytes(0x01, 1, 2, 0))
            .expect("10-byte common header must parse");
        assert!(
            s7comm_bounds_ok(&header, 12),
            "data_len == header_len + param_length + data_length exactly must pass \
             (BC-2.21.009 Canonical Test Vectors row 1)"
        );
        assert!(
            !s7comm_bounds_ok(&header, 11),
            "data_len one byte short of the declared total must fail \
             (BC-2.21.009 Canonical Test Vectors row 2)"
        );

        // Empty parameter/data blocks: trivially passes at data_len == header_len.
        let empty_header = parse_s7comm_header(&classic_header_bytes(0x01, 1, 0, 0))
            .expect("10-byte common header with empty blocks must parse");
        assert!(
            s7comm_bounds_ok(&empty_header, 10),
            "param_length == data_length == 0 with data_len == header_len exactly \
             must pass trivially (BC-2.21.009 edge case EC-001)"
        );

        // Overflow-free at maximum u16 values: must return false, never panic.
        let max_header = parse_s7comm_header(&classic_header_bytes(0x01, 1, 0xFFFF, 0xFFFF))
            .expect("10-byte common header must parse regardless of declared lengths");
        assert!(
            !s7comm_bounds_ok(&max_header, 10),
            "maximum-representable declared param_length/data_length against a bare \
             10-byte data_len must cleanly fail, never panic (BC-2.21.009 invariant 1, \
             STORY-187 Edge Case EC-007)"
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

    /// AC-187-011 (F-21c): the Ack-header (`header.header_len == 12`) bounds case,
    /// verified both via `on_data` (12 bytes total -- the declared 1-byte parameter
    /// is ABSENT -> T0814; 13 bytes -- the declared byte PRESENT -> clean) and via
    /// direct `s7comm_bounds_ok` assertions, to verify the check correctly
    /// incorporates the Ack/Ack_Data-specific `header_len == 12` base rather than
    /// assuming the 10-byte Job/Userdata default (corrected 2026-09-24 per the
    /// canonical-frame holdout ruling, DF-CANONICAL-FRAME-HOLDOUT-001 -- Ack_Data is
    /// no longer assumed to use the 10-byte default; this test exercises the
    /// `header_len == 12` mechanism via `rosctr == Ack`, which is shared identically
    /// with `rosctr == AckData` since both flow through the same
    /// `parse_s7comm_header` match arm and the same `s7comm_bounds_ok` helper).
    ///
    /// Traces: BC-2.21.009 postcondition 2; AC-187-011.
    #[test]
    fn test_BC_2_21_009_ack_header_len_12_bounds_check() {
        // 12 bytes total: Ack header (header_len=12) declaring param_length=1,
        // data_length=0, but the declared parameter byte is ABSENT.
        {
            let mut analyzer = S7commAnalyzer::new();
            let flow_key = flow_key_default();
            let header_bytes = ack_header_bytes(1, 1, 0, 0x00, 0x00);
            assert_eq!(header_bytes.len(), 12);
            let frame = dt_frame(&header_bytes);
            analyzer.on_data(flow_key, &frame, 0, Direction::ClientToServer);

            assert_eq!(
                analyzer.findings.len(),
                1,
                "a 12-byte Ack header declaring param_length=1 with the parameter \
                 byte ABSENT must fail the bounds check and emit one T0814 \
                 (AC-187-011)"
            );
            assert_malformed_header_t0814(&analyzer.findings[0], Direction::ClientToServer);
        }

        // 13 bytes total: same Ack header, this time WITH the declared parameter
        // byte present -- bounds check passes cleanly.
        {
            let mut analyzer = S7commAnalyzer::new();
            let flow_key = flow_key_default();
            let mut header_bytes = ack_header_bytes(1, 1, 0, 0x00, 0x00);
            header_bytes.push(0xAA); // the declared parameter byte
            assert_eq!(header_bytes.len(), 13);
            let frame = dt_frame(&header_bytes);
            analyzer.on_data(flow_key, &frame, 0, Direction::ClientToServer);

            assert!(
                analyzer.findings.is_empty(),
                "a 13-byte Ack header declaring param_length=1 WITH the parameter \
                 byte present must pass the bounds check cleanly, no finding \
                 (AC-187-011)"
            );
        }

        // Direct s7comm_bounds_ok assertions incorporating the Ack-specific
        // header_len == 12 base rather than assuming the 10-byte default.
        let header = parse_s7comm_header(&ack_header_bytes(1, 1, 0, 0x00, 0x00))
            .expect("12-byte Ack header must parse");
        assert_eq!(header.header_len, 12);
        assert!(
            !s7comm_bounds_ok(&header, 12),
            "header_len(12) + param_length(1) + data_length(0) == 13 > data_len(12) \
             must fail the bounds check (AC-187-011)"
        );
        assert!(
            s7comm_bounds_ok(&header, 13),
            "header_len(12) + param_length(1) + data_length(0) == 13 == data_len(13) \
             must pass the bounds check exactly (AC-187-011)"
        );
    }

    /// P12-F-1: unlike every other BC-2.21.009 `on_data` test in this module (which
    /// all use `data_length == 0` and vary `param_length`), this test holds
    /// `param_length == 0` fixed and overruns exclusively on `data_length` -- the
    /// declared 2 data bytes are only partially (1 byte) or exactly (2 bytes)
    /// present, with NO parameter bytes at all. A mutation that drops
    /// `header.data_length` from `s7comm_bounds_ok`'s summed total (leaving only
    /// `header_len + param_length`) would compute `declared_total == 10` here
    /// instead of the correct `12`, so the 1-trailing-byte case (`data_len == 11 >=
    /// 10`) would incorrectly PASS the bounds check and emit zero findings --
    /// diverging from this test's expectation of exactly one T0814.
    ///
    /// Traces: BC-2.21.009 postcondition 1 (declared total includes data_length).
    #[test]
    fn test_BC_2_21_009_data_length_overrun_on_data_emits_t0814() {
        // 1 of the declared 2 data bytes present -> bounds check fails, one T0814.
        {
            let mut analyzer = S7commAnalyzer::new();
            let flow_key = flow_key_default();

            let mut header = classic_header_bytes(0x01, 0x0001, 0x0000, 0x0002);
            header.push(0xAA); // only 1 of the declared 2 data bytes present
            let frame = dt_frame(&header);
            analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

            assert_eq!(
                analyzer.findings.len(),
                1,
                "declared data_length exceeding the actually-available bytes (with \
                 param_length == 0) must emit exactly one malformed-header T0814 \
                 (P12-F-1, BC-2.21.009 postcondition 1)"
            );
            assert_malformed_header_t0814(&analyzer.findings[0], Direction::ClientToServer);
            assert_reason_specific_evidence(&analyzer.findings[0], "exceed available bytes");
        }

        // Both declared data bytes present -> bounds check passes, no finding.
        {
            let mut analyzer = S7commAnalyzer::new();
            let flow_key = flow_key_default();

            let mut header = classic_header_bytes(0x01, 0x0001, 0x0000, 0x0002);
            header.extend_from_slice(&[0xAA, 0xBB]); // exactly the declared 2 data bytes
            let frame = dt_frame(&header);
            analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

            assert!(
                analyzer.findings.is_empty(),
                "an exact declared-data_length-to-available-bytes match (with \
                 param_length == 0) must pass the bounds check cleanly, with no \
                 malformed-header finding (P12-F-1)"
            );
        }
    }

    /// P12-F-1: direct `s7comm_bounds_ok` assertions isolating `data_length`'s
    /// contribution to the summed bound -- `header_len == 10`, `param_length == 0`,
    /// `data_length == 2`. Kills the same "drop data_length from the sum" mutation
    /// as the `on_data`-level test above, without going through `on_data` or
    /// `parse_s7comm_header` at all. Also covers a MIXED `param_length ==
    /// 3`/`data_length == 2` case, so a mutation that reads `data_length` but drops
    /// `param_length` (or vice versa) is caught too.
    ///
    /// Traces: BC-2.21.009 postcondition 1, invariant 1.
    #[test]
    fn test_BC_2_21_009_s7comm_bounds_ok_data_length_only_overrun() {
        let header = S7commHeader {
            rosctr: Rosctr::Job,
            pdu_reference: 0,
            param_length: 0,
            data_length: 2,
            error_class: None,
            error_code: None,
            header_len: 10,
        };
        assert!(
            !s7comm_bounds_ok(&header, 11),
            "header_len(10) + param_length(0) + data_length(2) == 12 > data_len(11) \
             must fail -- if data_length were dropped from the sum, declared_total \
             would be 10 and data_len(11) would incorrectly pass (P12-F-1)"
        );
        assert!(
            s7comm_bounds_ok(&header, 12),
            "header_len(10) + param_length(0) + data_length(2) == 12 == data_len(12) \
             must pass the bounds check exactly (P12-F-1)"
        );

        // Mixed case: both param_length and data_length are non-zero and distinct.
        let mixed = S7commHeader {
            param_length: 3,
            data_length: 2,
            ..header
        };
        assert!(
            !s7comm_bounds_ok(&mixed, 14),
            "header_len(10) + param_length(3) + data_length(2) == 15 > data_len(14) \
             must fail (P12-F-1 mixed case)"
        );
        assert!(
            s7comm_bounds_ok(&mixed, 15),
            "header_len(10) + param_length(3) + data_length(2) == 15 == data_len(15) \
             must pass exactly (P12-F-1 mixed case)"
        );
    }

    /// cargo-mutants (27.1.0) survivor kill: `dispatch_classic_s7comm`'s
    /// declared-vs-available evidence computation (`declared = header.header_len as
    /// u64 + header.param_length as u64 + header.data_length as u64;`, F-11) had no
    /// test asserting the exact numeric `declared`/`available` values it renders
    /// into the T0814 evidence text, so three arithmetic mutations at that line
    /// survived: `+` -> `*` on the first `+` (`header_len * param_length +
    /// data_length`, giving 35 for this test's inputs instead of 18), `+` -> `-` on
    /// the second `+` (`header_len + param_length - data_length`, giving 8), and `+`
    /// -> `*` on the second `+` (`header_len + param_length * data_length`, giving
    /// 25). All three mutants still fail the bounds check and still emit one T0814
    /// (the boolean accept/reject decision comes from `s7comm_bounds_ok`, which is
    /// untouched by these mutations) -- only the evidence text's numbers differ, so
    /// asserting the exact `declared 18 (header_len=10 + param_length=3 +
    /// data_length=5)` / `available 12` substrings is required to distinguish
    /// correct behavior from all three surviving mutants (35, 8, 25).
    ///
    /// Job header (`rosctr = 0x01`, `header_len = 10`): `param_length = 3`,
    /// `data_length = 5` -> declared total `10 + 3 + 5 = 18`; the frame's classic
    /// S7comm payload (the bytes passed to `dispatch_classic_s7comm`) is exactly 12
    /// bytes (the 10-byte header plus 2 trailing bytes), 6 short of the declared 18,
    /// so the bounds check fails and exactly one T0814 is emitted with `available ==
    /// 12`.
    ///
    /// Traces: BC-2.21.009 postcondition 2, F-11; AC-187-011.
    #[test]
    fn test_BC_2_21_009_bounds_failure_evidence_reports_declared_and_available() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // Job header: header_len=10, param_length=3, data_length=5 -> declared 18.
        let mut header = classic_header_bytes(0x01, 0x0001, 0x0003, 0x0005);
        header.extend_from_slice(&[0xAA, 0xBB]); // only 2 trailing bytes -> 12 total
        assert_eq!(
            header.len(),
            12,
            "test setup: 10-byte header + 2 trailing bytes"
        );
        let frame = dt_frame(&header);

        analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

        assert_eq!(
            analyzer.findings.len(),
            1,
            "a Job header declaring a total of 18 bytes against only 12 available \
             must emit exactly one malformed-header T0814 (BC-2.21.009 postcondition 2)"
        );
        assert_malformed_header_t0814(&analyzer.findings[0], Direction::ClientToServer);
        assert!(
            analyzer.findings[0]
                .evidence
                .iter()
                .any(|e| e.contains(
                    "declared 18 (header_len=10 + param_length=3 + data_length=5)"
                )),
            "evidence must report the correct declared total (18) and its exact \
             header_len/param_length/data_length breakdown -- distinguishes the \
             correct `+`/`+` computation from the surviving `*`/`+` mutant (35), got \
             {:?}",
            analyzer.findings[0].evidence
        );
        assert!(
            analyzer.findings[0]
                .evidence
                .iter()
                .any(|e| e.contains("available 12 (BC-2.21.009)")),
            "evidence must report the correct available byte count (12) -- \
             distinguishes the correct `+`/`+` computation from the surviving \
             `+`/`-` mutant (8) and `+`/`*` mutant (25), got {:?}",
            analyzer.findings[0].evidence
        );
    }

    /// Companion to
    /// `test_BC_2_21_009_bounds_failure_evidence_reports_declared_and_available`
    /// above, using the Ack_Data (`header_len = 12`) variant rather than Job
    /// (`header_len = 10`) -- confirms the declared/available evidence text is
    /// correct across BOTH `header_len` bases (2026-09-24 canonical-frame holdout
    /// ruling, DF-CANONICAL-FRAME-HOLDOUT-001), not merely coincidentally correct for
    /// the 10-byte common-header case.
    ///
    /// Ack_Data header (`rosctr = 0x03`, `header_len = 12`): `param_length = 3`,
    /// `data_length = 5` -> declared total `12 + 3 + 5 = 20`; the frame's classic
    /// S7comm payload is exactly 14 bytes (the 12-byte header plus 2 trailing
    /// bytes), 6 short of the declared 20, so the bounds check fails and exactly one
    /// T0814 is emitted with `available == 14`.
    ///
    /// Traces: BC-2.21.009 postcondition 2, F-11; AC-187-011.
    #[test]
    fn test_BC_2_21_009_bounds_failure_evidence_ack_data_header_len_12() {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = flow_key_default();

        // Ack_Data header: header_len=12, param_length=3, data_length=5 -> declared 20.
        let mut header = ack_data_header_bytes(0x0001, 0x0003, 0x0005, 0x00, 0x00);
        header.extend_from_slice(&[0xAA, 0xBB]); // only 2 trailing bytes -> 14 total
        assert_eq!(
            header.len(),
            14,
            "test setup: 12-byte header + 2 trailing bytes"
        );
        let frame = dt_frame(&header);

        analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

        assert_eq!(
            analyzer.findings.len(),
            1,
            "an Ack_Data header declaring a total of 20 bytes against only 14 \
             available must emit exactly one malformed-header T0814 (BC-2.21.009 \
             postcondition 2)"
        );
        assert_malformed_header_t0814(&analyzer.findings[0], Direction::ClientToServer);
        assert!(
            analyzer.findings[0]
                .evidence
                .iter()
                .any(|e| e.contains(
                    "declared 20 (header_len=12 + param_length=3 + data_length=5)"
                )),
            "evidence must report the correct declared total (20) and its exact \
             header_len/param_length/data_length breakdown for the header_len=12 \
             (Ack_Data) case, got {:?}",
            analyzer.findings[0].evidence
        );
        assert!(
            analyzer.findings[0]
                .evidence
                .iter()
                .any(|e| e.contains("available 14 (BC-2.21.009)")),
            "evidence must report the correct available byte count (14) for the \
             header_len=12 (Ack_Data) case, got {:?}",
            analyzer.findings[0].evidence
        );
    }

    /// P12-F-1: `pdu_reference`, `param_length`, and `data_length` are each decoded
    /// via `u16::from_be_bytes` (big-endian) -- never `from_le_bytes`. Uses
    /// byte-asymmetric values (each field's high byte != its low byte, and all three
    /// fields are pairwise distinct) so that a mutation swapping big-endian for
    /// little-endian decoding on ANY one of the three fields produces a different
    /// value than expected, and is caught.
    ///
    /// `0x0102` decoded little-endian would read as `0x0201`; `0x0304`
    /// little-endian as `0x0403`; `0x0506` little-endian as `0x0605` -- all three
    /// diverge from the correct big-endian values asserted below.
    ///
    /// Traces: BC-2.21.006 postconditions 2-4.
    #[test]
    fn test_BC_2_21_006_byte_asymmetric_big_endian_decode() {
        let data = classic_header_bytes(0x01, 0x0102, 0x0304, 0x0506);
        assert_eq!(data.len(), 10);

        let parsed = parse_s7comm_header(&data).expect("valid Job header must parse");
        assert_eq!(
            parsed.pdu_reference, 0x0102,
            "pdu_reference (data[4..6]) must be decoded big-endian (P12-F-1)"
        );
        assert_eq!(
            parsed.param_length, 0x0304,
            "param_length (data[6..8]) must be decoded big-endian (P12-F-1)"
        );
        assert_eq!(
            parsed.data_length, 0x0506,
            "data_length (data[8..10]) must be decoded big-endian (P12-F-1)"
        );
    }

    // =========================================================================
    // VP-051 (Kani P0, skeleton, v1.1 / F-14): S7comm Header Bounds-Before-Slice
    // Safety. Traces BC-2.21.004, BC-2.21.006, BC-2.21.007, BC-2.21.008, BC-2.21.009.
    // Full proof execution deferred to STORY-194 (formal-hardening) per this
    // story's VP-051 Kani Obligation note.
    // =========================================================================

    /// `#[cfg(kani)]` skeleton, compiled only under `cargo kani` -- under a normal
    /// `cargo test`/`cargo check` build this module compiles to nothing (mirrors the
    /// `tests/kani_proofs.rs` VP-025/VP-027 pattern and the in-source
    /// `#[cfg(kani)] mod kani_proofs` pattern used by `src/analyzer/iso_on_tcp.rs`'s
    /// VP-048/VP-049 harnesses). Located in this test file per STORY-187's own File
    /// Structure Requirements table (`tests/s7comm_analyzer_tests.rs` MODIFY: "+
    /// VP-051 Kani skeleton"), rather than inside `src/analyzer/s7comm.rs`, since
    /// `parse_s7comm_header` and `s7comm_bounds_ok` are both `pub` and fully
    /// exercisable from the test crate.
    ///
    /// v1.1 (F-14, human-ratified 2026-09-24): rewritten to use BOUNDED symbolic
    /// input -- a fixed-size `[u8; 16]` array via `kani::any()` plus an
    /// assumed-bounded `len <= 16` -- rather than an unbounded `Vec<u8>`, and to
    /// assert the non-vacuous properties VP-051's Kani Obligation note requires
    /// (`len < 10` implies `None`; `Some` implies `header_len ∈ {10, 12}` and
    /// `data.len() >= header_len`; `error_class.is_some() == (rosctr == Ack ||
    /// rosctr == AckData)`, identically for `error_code`). Also exercises
    /// `s7comm_bounds_ok`, the pure crate-visible helper BC-2.21.009's caller-side
    /// bounds check was extracted into for exactly this purpose.
    ///
    /// v1.3 (round 4, human-ratified 2026-09-24, canonical-frame holdout ruling
    /// DF-CANONICAL-FRAME-HOLDOUT-001): the non-vacuous assertion 3 below was
    /// corrected from the narrower `error_class.is_some() == (rosctr == Ack)` to
    /// `(rosctr == Ack || rosctr == AckData)` -- the narrower form would vacuously
    /// PASS a `header_len`/`error_class` mismatch on every Ack_Data input (an
    /// Ack_Data header with `header_len == 12` but `error_class == None` would
    /// satisfy the narrower equation, since `rosctr == Ack` is `false` for
    /// `AckData` too), silently hiding exactly the kind of bug this harness exists
    /// to catch. `header_len` selection is `12` when `rosctr ∈ {Ack, AckData}` and
    /// `10` otherwise (Job, Userdata).
    #[cfg(kani)]
    mod vp051_kani {
        use wirerust::analyzer::s7comm::{Rosctr, parse_s7comm_header, s7comm_bounds_ok};

        /// VP-051: `parse_s7comm_header` must not panic for any input up to a bounded
        /// length (a fixed-size `[u8; 16]` array, `len <= 16`), including every
        /// `data.len() < 10` case (BC-2.21.004) and the `data[0] != 0x32`
        /// defensive-reject case (BC-2.21.005).
        ///
        /// `Some(header)` positive assertions (pass-6 F-47): beyond the header_len
        /// shape check, the `Some` branch now asserts the full field-mapping contract
        /// so header_len is linked to ROSCTR by an `assert_eq!`, not merely by a
        /// `kani::cover!`:
        /// - `header.header_len == 12` iff `rosctr ∈ {Ack, AckData}`, else `10`
        ///   (BC-2.21.006 postcondition 1 / BC-2.21.008 postcondition 2).
        /// - When `rosctr ∈ {Ack, AckData}`: `error_class == Some(slice[10])` and
        ///   `error_code == Some(slice[11])` (BC-2.21.008 postcondition 2 -- exact
        ///   byte values; presence alone is postcondition 3).
        /// - `rosctr` maps exactly from `slice[1]`: `0x01 -> Job`, `0x02 -> Ack`,
        ///   `0x03 -> AckData`, `0x07 -> Userdata` (BC-2.21.006 postcondition 1 /
        ///   BC-2.21.007 postcondition 1 -- the recognized-ROSCTR set).
        /// - `pdu_reference`, `param_length`, `data_length` equal the big-endian reads
        ///   of `slice[4..6]`, `slice[6..8]`, `slice[8..10]` respectively
        ///   (BC-2.21.006 postcondition 2/3/4).
        ///
        /// Non-vacuous per DF-KANI-NONVACUITY-001 (F-24 wording correction,
        /// human-ratified 2026-09-24): the `kani::cover!` checks below record whether
        /// BOTH the `None` and `Some` return paths, and both the `header_len == 10`
        /// and `header_len == 12` shapes, are reachable from the bounded symbolic
        /// input. `kani::cover!` does NOT, by itself, fail verification when a
        /// covered condition turns out unreachable -- it is reported as an
        /// UNSATISFIABLE cover in the Kani output, not a hard verification failure.
        /// STORY-194 (the full non-vacuous proof run) MUST invoke `cargo kani` with
        /// `--fail-uncoverable` (or otherwise inspect each cover's
        /// satisfied/unsatisfied status in the Kani report) so that a vacuous
        /// harness is actually caught as a failure, not silently accepted.
        #[kani::proof]
        fn verify_parse_s7comm_header_bounds_safety() {
            let data: [u8; 16] = kani::any();
            let len: usize = kani::any();
            kani::assume(len <= 16);
            let slice = &data[..len];

            // Must not panic for any bounded input (BC-2.21.004/005/006/007/008
            // guards).
            let result = parse_s7comm_header(slice);

            // Non-vacuous assertion 1 (BC-2.21.004 postcondition 1): len < 10 implies
            // None.
            if len < 10 {
                assert!(
                    result.is_none(),
                    "VP-051 / BC-2.21.004 postcondition 1: data.len() < 10 must always \
                     return None"
                );
            }

            if let Some(header) = result {
                // Non-vacuous assertion 2: header_len is always 10 or 12, and the
                // input slice is always at least that long.
                assert!(
                    header.header_len == 10 || header.header_len == 12,
                    "VP-051: header_len must be exactly 10 or 12"
                );
                assert!(
                    slice.len() >= header.header_len,
                    "VP-051 / BC-2.21.004 postcondition 1 / BC-2.21.008 postcondition 1: \
                     a Some(header) result's own header_len must never exceed the input \
                     slice's length -- this is the parser's own length guard (>= 10 for \
                     Job/Userdata, >= 12 for Ack/Ack_Data), not BC-2.21.009's caller-side \
                     param_length/data_length bounds check"
                );

                // Non-vacuous assertion 3 (BC-2.21.008 postcondition 3, corrected
                // 2026-09-24 per DF-CANONICAL-FRAME-HOLDOUT-001): error_class/
                // error_code are Some iff rosctr is Ack OR AckData, identically for
                // both fields -- NOT the narrower (rosctr == Ack) alone, which would
                // vacuously pass a header_len/error_class mismatch on every Ack_Data
                // input.
                let is_ack_or_ack_data =
                    header.rosctr == Rosctr::Ack || header.rosctr == Rosctr::AckData;
                assert_eq!(
                    header.error_class.is_some(),
                    is_ack_or_ack_data,
                    "VP-051 / BC-2.21.008 postcondition 3: error_class.is_some() must \
                     equal (rosctr == Ack || rosctr == AckData)"
                );
                assert_eq!(
                    header.error_code.is_some(),
                    is_ack_or_ack_data,
                    "VP-051 / BC-2.21.008 postcondition 3: error_code.is_some() must \
                     equal (rosctr == Ack || rosctr == AckData)"
                );

                // Positive assertion (pass-6 F-47, BC-2.21.006 postcondition 1 /
                // BC-2.21.008 postcondition 2): header_len is LINKED to ROSCTR by an
                // assert_eq!, not merely observed via kani::cover! below -- 12 iff
                // Ack/AckData, else 10.
                assert_eq!(
                    header.header_len,
                    if is_ack_or_ack_data { 12 } else { 10 },
                    "VP-051 / BC-2.21.006 postcondition 1 / BC-2.21.008 postcondition 2: \
                     header_len must be 12 iff rosctr is Ack or AckData, else 10"
                );

                // Positive assertion (pass-6 F-47, BC-2.21.008 postcondition 2): when
                // the extended header is present, error_class/error_code carry the
                // EXACT byte values from slice[10]/slice[11], not merely Some(_) --
                // postcondition 3 covers presence alone; the exact-value equality
                // asserted here is postcondition 2.
                if is_ack_or_ack_data {
                    assert_eq!(
                        header.error_class,
                        Some(slice[10]),
                        "VP-051 / BC-2.21.008 postcondition 2: error_class must equal \
                         Some(slice[10]) for Ack/AckData"
                    );
                    assert_eq!(
                        header.error_code,
                        Some(slice[11]),
                        "VP-051 / BC-2.21.008 postcondition 2: error_code must equal \
                         Some(slice[11]) for Ack/AckData"
                    );
                }

                // Positive assertion (pass-6 F-47, BC-2.21.006 postcondition 1 /
                // BC-2.21.007 postcondition 1): rosctr must map EXACTLY from slice[1]
                // -- 0x01 -> Job, 0x02 -> Ack, 0x03 -> AckData, 0x07 -> Userdata (the
                // BC-2.21.007 postcondition 1 recognized-ROSCTR set). No other
                // slice[1] value reaches this Some(header) branch (BC-2.21.007
                // safe-reject).
                let expected_rosctr = match slice[1] {
                    0x01 => Rosctr::Job,
                    0x02 => Rosctr::Ack,
                    0x03 => Rosctr::AckData,
                    0x07 => Rosctr::Userdata,
                    other => panic!(
                        "VP-051 / BC-2.21.007 postcondition 1: unexpected slice[1] = \
                         {other:#x} reached a Some(header) result; only \
                         0x01/0x02/0x03/0x07 may produce Some"
                    ),
                };
                assert_eq!(
                    header.rosctr, expected_rosctr,
                    "VP-051 / BC-2.21.006 postcondition 1 / BC-2.21.007 postcondition 1: \
                     rosctr must map exactly from slice[1]"
                );

                // Positive assertion (pass-6 F-47, BC-2.21.006 postcondition 2/3/4):
                // pdu_reference/param_length/data_length are exactly the big-endian
                // reads of slice[4..6]/[6..8]/[8..10] -- the common fields are always
                // present and unaffected by header_len shape.
                assert_eq!(
                    header.pdu_reference,
                    u16::from_be_bytes([slice[4], slice[5]]),
                    "VP-051 / BC-2.21.006 postcondition 2: pdu_reference must equal \
                     u16::from_be_bytes([slice[4], slice[5]])"
                );
                assert_eq!(
                    header.param_length,
                    u16::from_be_bytes([slice[6], slice[7]]),
                    "VP-051 / BC-2.21.006 postcondition 3: param_length must equal \
                     u16::from_be_bytes([slice[6], slice[7]])"
                );
                assert_eq!(
                    header.data_length,
                    u16::from_be_bytes([slice[8], slice[9]]),
                    "VP-051 / BC-2.21.006 postcondition 4: data_length must equal \
                     u16::from_be_bytes([slice[8], slice[9]])"
                );

                // F-14: also exercise the extracted BC-2.21.009 pure helper directly,
                // independent of the effectful on_data call site -- must not panic for
                // any bounded data_len.
                let _ = s7comm_bounds_ok(&header, slice.len());

                // NON-VACUITY: both header_len shapes, and both Ack and AckData, must
                // be reachable.
                kani::cover!(header.header_len == 10);
                kani::cover!(header.header_len == 12);
                kani::cover!(header.rosctr == Rosctr::Ack);
                kani::cover!(header.rosctr == Rosctr::AckData);
            }

            // NON-VACUITY: both the None and Some return paths must be reachable from
            // the bounded symbolic input.
            kani::cover!(result.is_none());
            kani::cover!(result.is_some());
        }

        /// VP-051 bounds-check half (F-18, human-ratified 2026-09-24): a SECOND,
        /// INDEPENDENT symbolic input `data_len: usize` -- decoupled from the small
        /// fixed-size symbolic array used for header-field extraction above --
        /// representing the length of the hypothetical full delivery the header's
        /// declared `param_length`/`data_length` are checked against. Proves
        /// `s7comm_bounds_ok` returns EXACTLY the right answer (the equality
        /// asserted directly, both directions, not merely one direction of an
        /// implication) and that a `true` result implies the resulting
        /// parameter/data sub-slice access is always `Some(..)`, never `None` and
        /// never a construct that could panic.
        ///
        /// Bounded per the VP-051 Kani Obligation note's own justification:
        /// `data_len <= u16::MAX as usize * 3` represents any hypothetical
        /// full-delivery length up to three times the maximum single `u16` field,
        /// comfortably covering `header_len (<= 12) + param_length (<= u16::MAX) +
        /// data_length (<= u16::MAX)` without requiring an unbounded symbolic value.
        #[kani::proof]
        fn verify_s7comm_bounds_ok_bounds_safety() {
            let data: [u8; 16] = kani::any();
            let len: usize = kani::any();
            kani::assume(len <= 16);
            let slice = &data[..len];

            let Some(header) = parse_s7comm_header(slice) else {
                return;
            };

            let data_len: usize = kani::any();
            kani::assume(data_len <= u16::MAX as usize * 3);

            let declared_total =
                header.header_len as u64 + header.param_length as u64 + header.data_length as u64;
            let expected = data_len as u64 >= declared_total;

            // Exact equality (both directions): no false-accept, no false-reject.
            assert_eq!(
                s7comm_bounds_ok(&header, data_len),
                expected,
                "VP-051 F-18: s7comm_bounds_ok must return EXACTLY (data_len >= \
                 header_len + param_length + data_length) -- no false-accept and no \
                 false-reject"
            );

            if expected {
                // A symbolic buffer of length data_len -- the hypothetical full
                // delivery. When the bounds check passes, the parameter/data
                // sub-slice access must always succeed.
                let full: Vec<u8> = vec![0u8; data_len];
                let param_data_end =
                    header.header_len + header.param_length as usize + header.data_length as usize;
                assert!(
                    full.get(header.header_len..param_data_end).is_some(),
                    "VP-051 F-18: when s7comm_bounds_ok is true, the parameter/data \
                     sub-slice access data.get(header_len..header_len+param_length+ \
                     data_length) must be Some(..), never None, never a panic"
                );
            }

            // NON-VACUITY (F-18, DF-KANI-NONVACUITY-001): both the bounds-ok-true and
            // bounds-ok-false outcomes must be reachable under the symbolic inputs,
            // independent of the header-extraction half's None/Some cover
            // obligations above.
            kani::cover!(expected);
            kani::cover!(!expected);
        }
    }

    // =========================================================================
    // VP-053 (proptest P0, PARTIAL skeleton, v1.1 / F-02): `protocol_id` Four-Way
    // Dispatch Totality and Unclassified Never-Force-Fit. Traces BC-2.21.002,
    // BC-2.21.027, BC-2.21.028. This story wires the CR/CC and Some(0x32) branches
    // fully; Some(0x72)/Some(other) route to a structural no-op completed in
    // STORY-190. The full non-vacuous run is deferred to STORY-194.
    //
    // v1.1 (F-14/VP-INDEX.md v2.51's "Sticky-Classification Gating" reword,
    // human-ratified 2026-09-24): the strategy now generates `protocol_id: None` as
    // its OWN distinct case (`Option<u8>`, not a bare `u8`), rather than only
    // implicitly via absence -- the property under test ("protocol_id: None does not
    // participate in sticky first-classification", F-02) is untestable unless the
    // strategy can generate it explicitly alongside Some(0x32)/Some(0x72)/Some(other).
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

        /// Generates `protocol_id` as `Option<u8>`, an explicit, equally-weighted
        /// case split between `None` (empty DT payload, no protocol evidence, F-02)
        /// and `Some(byte)` (any of the 256 possible protocol-ID byte values) --
        /// rather than `None` arising only implicitly/by absence.
        fn protocol_id_strategy() -> impl Strategy<Value = Option<u8>> {
            prop_oneof![Just(None), any::<u8>().prop_map(Some)]
        }

        /// Builds a complete DT frame carrying `protocol_id`. When `protocol_id ==
        /// Some(0x32)`, a fully valid minimal classic Job header (empty parameter/data
        /// blocks) follows, so that VP-053's dispatch-TOTALITY property is exercised
        /// independently of BC-2.21.004-009's header-parse REJECTION paths (those are
        /// this file's dedicated `test_BC_2_21_004/007/008/009_*` tests). When
        /// `protocol_id == None`, the COTP fixed part uses the genuinely-constructible
        /// class-0 DT shape (`LI=2`, code `0xF0`, TPDU-NR+EOT `0x80`, no upper-layer
        /// bytes at all) -- `payload_offset == tpkt_payload.len()` exactly, matching
        /// BC-2.20.010's `protocol_id: None` case (mirrors the fix applied to this
        /// file's top-level `dt_frame_empty_payload()` helper, F-03).
        fn dt_frame_for_protocol_id(protocol_id: Option<u8>) -> Vec<u8> {
            let cotp = match protocol_id {
                Some(0x32) => {
                    let mut cotp = vec![0x01u8, 0xF0, 0x32];
                    cotp.extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]);
                    cotp
                }
                Some(byte) => vec![0x01u8, 0xF0, byte],
                None => vec![0x02u8, 0xF0, 0x80],
            };
            tpkt_frame(&cotp)
        }

        proptest! {
            /// VP-053 (partial skeleton, v1.1): for any `protocol_id` (`None` or
            /// `Some(byte)`), the first DT frame observed on a fresh flow must
            /// classify it exactly per BC-2.21.002's four-way table: `Some(0x32)` ->
            /// `Classic`, `Some(0x72)` -> `Plus`, any other `Some(byte)` ->
            /// `Unclassified`, and `None` -> NO classification at all (`None` stays
            /// `None`, F-02) -- and a `S7commFlowState` must always be created
            /// (BC-2.21.001 postcondition 3), regardless of `protocol_id`.
            ///
            /// Sticky-gating (F-12) is NOT re-exercised in this single-frame-per-flow
            /// skeleton -- it is covered by the dedicated
            /// `test_BC_2_21_002_0x32_dt_frame_not_dissected_when_sticky_classified_plus_or_unclassified`
            /// test above; extending this proptest to a multi-frame-per-flow
            /// sequence is deferred to STORY-194's full non-vacuous run alongside the
            /// Some(0x72)/Some(other) dissection completeness deferred to STORY-190.
            #[test]
            fn proptest_vp053_protocol_id_dispatch_totality(
                protocol_id in protocol_id_strategy(),
                salt in any::<u16>(),
            ) {
                let mut analyzer = S7commAnalyzer::new();
                let flow_key = flow_key_for(salt);

                let frame = dt_frame_for_protocol_id(protocol_id);
                analyzer.on_data(flow_key.clone(), &frame, 0, Direction::ClientToServer);

                let state = analyzer.flows.get(&flow_key).expect(
                    "VP-053: S7commFlowState must be created lazily on the first \
                     on_data call regardless of protocol_id (BC-2.21.001 postcondition 3)"
                );

                let expected = match protocol_id {
                    Some(0x32) => Some(S7Protocol::Classic),
                    Some(0x72) => Some(S7Protocol::Plus),
                    Some(_) => Some(S7Protocol::Unclassified),
                    None => None,
                };
                prop_assert_eq!(
                    state.classified_protocol,
                    expected,
                    "VP-053 LOAD-BEARING property (v1.1, F-02): the first DT frame's \
                     protocol_id must classify the flow exactly per BC-2.21.002's \
                     four-way table, with protocol_id: None carrying NO protocol \
                     evidence and never classifying -- protocol_id={:?}",
                    protocol_id
                );

                // P11-F-4: `dt_frame_for_protocol_id` never generates a MALFORMED
                // classic-S7comm payload -- the `Some(0x32)` branch always builds a
                // complete, well-formed 10-byte Job header with empty
                // parameter/data blocks (verified above the `proptest!` block), and
                // every other `protocol_id` value (`None`, `Some(0x72)`,
                // `Some(other)`) never reaches `dispatch_classic_s7comm` at all (the
                // F-12 gate only fires for `Some(0x32)`). So across every value this
                // strategy can generate, zero findings must ever be emitted --
                // scoped to this proptest's single-frame-per-flow generator only,
                // not a general claim about all possible classic S7comm frames.
                prop_assert!(
                    analyzer.findings.is_empty(),
                    "VP-053: this strategy generates only a well-formed classic Job \
                     header (protocol_id Some(0x32)) or frames that never reach \
                     classic dissection at all -- no scenario this generator can \
                     produce should ever emit a finding, but got {:?} for \
                     protocol_id={:?}",
                    analyzer.findings,
                    protocol_id
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
            cr_observed_dir: None,
            classified_protocol: None,
            malformed_header_reported_c2s: false,
            malformed_header_reported_s2c: false,
        };
        assert_eq!(state.classified_protocol, None);
    }

    // =========================================================================
    // Pass-3 F-25: the committed `tests/fixtures/s7comm-setup-comm.pcap` fixture
    // (generated by `tests/fixtures/mk_s7comm_pcap.py`) must actually be
    // exercised by a test, not merely generated and left unread. This also
    // regression-guards the F-25 fix: `mk_s7comm_pcap.py`'s Ack_Data (`0x03`)
    // PDU builder previously emitted a 10-byte common header for
    // `setup_communication_response`/`minimal_ack_data_pdu`, which is malformed
    // under the 2026-09-24 canonical-frame holdout ruling
    // (DF-CANONICAL-FRAME-HOLDOUT-001, BC-2.21.008): Ack_Data requires the same
    // 12-byte header as Ack (`error_class`/`error_code` at bytes 10/11, parameter
    // block starting at byte 12).
    // =========================================================================

    /// Minimal, stdlib-only Ethernet II + IPv4 + TCP walk sufficient to pull a
    /// packet's source/destination ports and TCP payload slice out of a raw
    /// captured frame (`RawPacket.data` — a full link-layer frame, since the
    /// fixture pcap's link-type is Ethernet, `LINKTYPE_ETHERNET = 1`). Returns
    /// `None` for anything that isn't a well-formed IPv4/TCP frame (not expected
    /// to occur in this fixture, since every packet is Ethernet/IPv4/TCP by
    /// construction). Deliberately narrow — just enough to drive
    /// `S7commAnalyzer::on_data` from the committed fixture below, not a
    /// general-purpose packet parser.
    fn extract_tcp_payload(
        frame: &[u8],
    ) -> Option<(std::net::IpAddr, std::net::IpAddr, u16, u16, &[u8])> {
        use std::net::{IpAddr, Ipv4Addr};

        const ETH_HEADER_LEN: usize = 14;
        if frame.len() < ETH_HEADER_LEN + 20 {
            return None;
        }
        let ethertype = u16::from_be_bytes([frame[12], frame[13]]);
        if ethertype != 0x0800 {
            // Not IPv4 — none of this fixture's frames take this branch.
            return None;
        }

        let ip_start = ETH_HEADER_LEN;
        let version_ihl = frame[ip_start];
        if version_ihl >> 4 != 4 {
            return None;
        }
        let ihl_bytes = usize::from(version_ihl & 0x0F) * 4;
        if ihl_bytes < 20 || frame.len() < ip_start + ihl_bytes {
            return None;
        }
        let protocol = frame[ip_start + 9];
        if protocol != 6 {
            // Not TCP — none of this fixture's frames take this branch.
            return None;
        }
        let src_ip = IpAddr::V4(Ipv4Addr::new(
            frame[ip_start + 12],
            frame[ip_start + 13],
            frame[ip_start + 14],
            frame[ip_start + 15],
        ));
        let dst_ip = IpAddr::V4(Ipv4Addr::new(
            frame[ip_start + 16],
            frame[ip_start + 17],
            frame[ip_start + 18],
            frame[ip_start + 19],
        ));
        let total_len = u16::from_be_bytes([frame[ip_start + 2], frame[ip_start + 3]]) as usize;
        let ip_end = (ip_start + total_len).min(frame.len());

        let tcp_start = ip_start + ihl_bytes;
        if frame.len() < tcp_start + 20 || tcp_start > ip_end {
            return None;
        }
        let src_port = u16::from_be_bytes([frame[tcp_start], frame[tcp_start + 1]]);
        let dst_port = u16::from_be_bytes([frame[tcp_start + 2], frame[tcp_start + 3]]);
        let data_offset_bytes = usize::from(frame[tcp_start + 12] >> 4) * 4;
        let payload_start = tcp_start + data_offset_bytes;
        if payload_start > ip_end {
            return None;
        }
        Some((
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            &frame[payload_start..ip_end],
        ))
    }

    /// Reads the committed `tests/fixtures/s7comm-setup-comm.pcap` capture via
    /// [`wirerust::reader::PcapSource::from_file`] (the same public entry point
    /// `tests/e2e_corpus_smoke_tests.rs` uses for its committed-fixture walk),
    /// extracts each packet's TCP payload with [`extract_tcp_payload`] above, and
    /// feeds every non-empty payload through [`S7commAnalyzer::on_data`] in the
    /// correct direction (destination port 102 -- the ISO-on-TCP/S7comm server
    /// port, ADR-014 -- is `ClientToServer`; the reverse is `ServerToClient`).
    ///
    /// Asserts the fixture is entirely well-formed under the corrected 12-byte
    /// Ack/Ack_Data header (pass-3 F-25): zero findings, `session_established ==
    /// true` (from the fixture's COTP CR/CC handshake), and `classified_protocol
    /// == Some(S7Protocol::Classic)` (from the fixture's `0x32` protocol-ID
    /// frames).
    #[test]
    fn test_BC_2_21_002_setup_comm_fixture_pcap_well_formed_no_findings() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/s7comm-setup-comm.pcap");
        let source = wirerust::reader::PcapSource::from_file(&path)
            .expect("the committed s7comm-setup-comm.pcap fixture must parse cleanly");

        const SERVER_PORT: u16 = 102;

        let mut analyzer = S7commAnalyzer::new();
        let mut flow_key: Option<FlowKey> = None;
        let mut payload_frames_seen = 0usize;

        for packet in &source.packets {
            let Some((src_ip, dst_ip, src_port, dst_port, payload)) =
                extract_tcp_payload(&packet.data)
            else {
                continue;
            };
            if payload.is_empty() {
                // TCP handshake (SYN/SYN-ACK/ACK) and the trailing FIN-ACK carry no
                // upper-layer payload -- nothing to feed to `on_data`.
                continue;
            }
            payload_frames_seen += 1;

            let direction = if dst_port == SERVER_PORT {
                Direction::ClientToServer
            } else {
                Direction::ServerToClient
            };
            // FlowKey::new canonicalizes its (ip, port) pairs internally, so
            // passing this packet's own src/dst (in either direction) yields the
            // same key for the whole bidirectional flow.
            let key = flow_key
                .get_or_insert_with(|| FlowKey::new(src_ip, src_port, dst_ip, dst_port))
                .clone();

            analyzer.on_data(key, payload, packet.timestamp_secs, direction);
        }

        assert_eq!(
            payload_frames_seen, 6,
            "expected exactly 6 TCP-payload-bearing packets in the fixture (COTP CR, \
             COTP CC, Setup Comm Job, Setup Comm Ack_Data, minimal Job, minimal \
             Ack_Data) -- got {payload_frames_seen}; the fixture's packet sequence may \
             have drifted from mk_s7comm_pcap.py's documented Packet sequence"
        );

        assert!(
            analyzer.findings.is_empty(),
            "the committed s7comm-setup-comm.pcap fixture must be entirely well-formed \
             under the corrected 12-byte Ack_Data header (pass-3 F-25) -- got {:?}",
            analyzer.findings
        );

        let flow_key =
            flow_key.expect("the fixture must contain at least one TCP-payload-bearing packet");
        let state = analyzer
            .flows
            .get(&flow_key)
            .expect("S7commFlowState must exist for the fixture's single flow");
        assert!(
            state.session_established,
            "the fixture's COTP Connect Request / Connect Confirm handshake must set \
             session_established (BC-2.21.001 postcondition 1, BC-2.21.002 \
             postcondition 2)"
        );
        assert_eq!(
            state.classified_protocol,
            Some(S7Protocol::Classic),
            "the fixture's classic S7comm (protocol_id 0x32) DT frames must \
             sticky-classify the flow Classic (BC-2.21.002 postcondition 3)"
        );
    }
}
