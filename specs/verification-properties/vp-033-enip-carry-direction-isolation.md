---
artifact: verification-property
vp_id: VP-033
title: "EtherNet/IP Carry-Buffer Direction Isolation"
status: draft
phase: P1
tool: proptest
subsystem: SS-17
module: "src/analyzer/enip.rs"
producer: architect
timestamp: 2026-06-27T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
feature_cycle: feature-enip-v0.11.0
issue: "#316"
bcs:
  - BC-2.17.016
ruling: RULING-EDGECASE-001
verification_lock: false
---

# VP-033: EtherNet/IP Carry-Buffer Direction Isolation

## Property Statement

For any sequence of `on_data` calls with alternating `Direction::ClientToServer` (c2s) and
`Direction::ServerToClient` (s2c) deliveries carrying partial-then-complete ENIP frames,
the `carry_c2s` and `carry_s2c` buffers are **never mixed**:

- A partial c2s frame stashed in `carry_c2s` is NEVER prepended to a subsequent s2c delivery.
- A partial s2c frame stashed in `carry_s2c` is NEVER prepended to a subsequent c2s delivery.
- Per-direction PDU counts (`pdu_count` contributions from each direction) match the counts
  produced by independent same-direction control runs: injecting interleaved s2c deliveries
  between c2s deliveries does NOT alter the c2s PDU count, and vice versa.

Formally: for any (c2s_frames, s2c_frames) pair of frame sequences with segment boundaries,
running them interleaved through `on_data` with alternating directions produces the same
`pdu_count` and `parse_errors` as running each direction's frames independently.

This directly prevents the cross-direction carry splice bug documented as EC-X1 in
RULING-EDGECASE-001: a partial c2s header stashed in carry cannot corrupt the next s2c
delivery's frame-walk loop.

## Verified BCs

| BC-ID | Description | How VP-033 Covers It |
|-------|-------------|----------------------|
| BC-2.17.016 v2.0 | `EnipFlowState` frame-walk with per-direction carry buffers | VP-033 verifies Invariant 7 (direction isolation): `carry_c2s` and `carry_s2c` are never mixed; `on_data` selects exactly one buffer per `direction` argument on every call. Also covers EC-010 (partial c2s stash does not contaminate s2c delivery). |

### BC Version Note

BC-2.17.016 was amended from v1.0 to v2.0 by RULING-EDGECASE-001. The v2.0 amendments
relevant to VP-033 are:

- Precondition 2: buf = `(match direction { ClientToServer => flow.carry_c2s, ServerToClient => flow.carry_s2c }) ++ data`
- Postcondition 3: carry stash-back selects the same directional carry
- Postcondition 4: cap check applies to the directional carry
- Invariant 1: both `carry_c2s.len() <= 600` AND `carry_s2c.len() <= 600`
- Invariant 7 (new): `carry_c2s` and `carry_s2c` are never mixed
- EC-010 (new): partial c2s stash does not contaminate s2c delivery

## Purity Classification

**Pure-core with controlled state injection.** The proptest strategy drives `EnipFlowState`
directly without any I/O. The test constructs synthetic byte sequences (valid ENIP headers
with known command/length fields), calls `on_data` with alternating directions, and asserts
count invariants. No file I/O, no network, no global state.

**Why proptest and NOT Kani:** The direction-isolation invariant is trivially enforced by
the `match direction { ClientToServer => ..., ServerToClient => ... }` match arm in the
production code. Kani model-checking of a two-arm match would prove the invariant by
inspection but would add no additional value beyond reading the code. The operationally
meaningful check is the behavioral invariant: that interleaving s2c deliveries between c2s
partial-frame deliveries does NOT corrupt PDU counts or parse_errors. This is a state-
machine property over sequences, which is the natural domain of proptest. The EC-X1 repro
was itself a sequence-based scenario; proptest directly encodes the same evidence.

## Proof Harness Skeleton

```rust
#[cfg(test)]
mod vp033_carry_direction_isolation {
    use super::*;
    use proptest::prelude::*;

    /// VP-033: carry_c2s and carry_s2c are never mixed across directions.
    ///
    /// Strategy: generate a valid c2s partial frame (bytes 0..23, i.e. one byte
    /// short of a complete 24-byte ENIP header), a valid complete s2c frame (full
    /// 24-byte header + declared payload), and a completing c2s delivery (the
    /// remaining bytes of the c2s header + a minimal payload).
    ///
    /// Assert: after the interleaved run, pdu_count == 2 (one c2s PDU + one s2c PDU)
    /// and parse_errors == 0. Compare against two independent same-direction control
    /// runs that together also produce pdu_count == 2 and parse_errors == 0.
    ///
    /// This directly encodes the EC-X1 repro: before the fix, the interleaved run
    /// produced pdu_count != 2 (spurious T0858 finding, missed error_count).
    proptest! {
        #[test]
        fn proptest_vp033_direction_isolation_pdu_count(
            // Segment split point: where to cut the c2s frame's first delivery
            // Must be in 1..23 (partial header, not yet complete)
            split_offset in 1usize..23,
            // s2c command (any valid ENIP command for the s2c frame)
            s2c_cmd in prop::sample::select(vec![
                0x0065u16, // RegisterSession
                0x0066u16, // UnRegisterSession
                0x0063u16, // ListIdentity
            ]),
        ) {
            let mut state = EnipFlowState::default();
            let ts: u32 = 100;

            // Build a valid c2s frame: ListIdentity (0x0063), 0-byte payload
            let c2s_frame = build_minimal_enip_frame(0x0063u16, 0u16);
            // Build a valid s2c frame: s2c_cmd, 0-byte payload
            let s2c_frame = build_minimal_enip_frame(s2c_cmd, 0u16);

            // Delivery 1: partial c2s header (bytes 0..split_offset) → stashed in carry_c2s
            state.on_data_test(&c2s_frame[..split_offset], ts, Direction::ClientToServer);
            prop_assert_eq!(state.pdu_count, 0, "partial delivery must not complete a PDU");
            prop_assert_eq!(state.parse_errors, 0, "partial delivery must not produce parse errors");

            // Delivery 2: complete s2c frame → carry_s2c used (carry_c2s NOT involved)
            state.on_data_test(&s2c_frame, ts, Direction::ServerToClient);
            prop_assert_eq!(state.pdu_count, 1, "s2c frame must complete exactly one PDU");
            prop_assert_eq!(state.parse_errors, 0, "s2c delivery must not produce parse errors");

            // Delivery 3: completing c2s bytes (split_offset..end) → carry_c2s prepended
            state.on_data_test(&c2s_frame[split_offset..], ts, Direction::ClientToServer);
            prop_assert_eq!(state.pdu_count, 2, "c2s frame must complete the second PDU; total == 2");
            prop_assert_eq!(state.parse_errors, 0, "completing c2s delivery must not produce parse errors");

            // Verify carry buffers are both empty (all frames fully consumed)
            prop_assert!(state.carry_c2s.is_empty(), "carry_c2s must be drained after complete frame");
            prop_assert!(state.carry_s2c.is_empty(), "carry_s2c must be drained after complete frame");
        }

        #[test]
        fn proptest_vp033_independent_run_equivalence(
            split_offset in 1usize..23,
            c2s_cmd in prop::sample::select(vec![0x0063u16, 0x0065u16, 0x006Fu16]),
            s2c_cmd in prop::sample::select(vec![0x0065u16, 0x0066u16, 0x0063u16]),
        ) {
            // Interleaved run
            let mut interleaved = EnipFlowState::default();
            let c2s_frame = build_minimal_enip_frame(c2s_cmd, 0u16);
            let s2c_frame = build_minimal_enip_frame(s2c_cmd, 0u16);
            interleaved.on_data_test(&c2s_frame[..split_offset], 100, Direction::ClientToServer);
            interleaved.on_data_test(&s2c_frame, 100, Direction::ServerToClient);
            interleaved.on_data_test(&c2s_frame[split_offset..], 100, Direction::ClientToServer);

            // Independent c2s-only run
            let mut c2s_only = EnipFlowState::default();
            c2s_only.on_data_test(&c2s_frame[..split_offset], 100, Direction::ClientToServer);
            c2s_only.on_data_test(&c2s_frame[split_offset..], 100, Direction::ClientToServer);

            // Independent s2c-only run
            let mut s2c_only = EnipFlowState::default();
            s2c_only.on_data_test(&s2c_frame, 100, Direction::ServerToClient);

            // Invariant: interleaved counts == sum of independent counts
            prop_assert_eq!(
                interleaved.pdu_count,
                c2s_only.pdu_count + s2c_only.pdu_count,
                "interleaved pdu_count must equal sum of independent runs"
            );
            prop_assert_eq!(
                interleaved.parse_errors,
                c2s_only.parse_errors + s2c_only.parse_errors,
                "interleaved parse_errors must equal sum of independent runs"
            );
        }
    }

    /// Build a minimal valid ENIP frame with given command and payload_len.
    /// Returns a Vec<u8> of length 24 + payload_len with a valid ENIP header
    /// (little-endian, status=0, session_handle=0, options=0).
    fn build_minimal_enip_frame(command: u16, payload_len: u16) -> Vec<u8> {
        let mut frame = vec![0u8; 24 + payload_len as usize];
        frame[0..2].copy_from_slice(&command.to_le_bytes());
        frame[2..4].copy_from_slice(&payload_len.to_le_bytes());
        // All other header fields zero (valid: status=0, session_handle=0, etc.)
        frame
    }
}
```

### Implementation Notes

- `on_data_test` is a thin test-visible wrapper around the production `on_data` logic
  (or `on_data` itself if made `pub(crate)` with a `#[cfg(test)]` companion).
- `build_minimal_enip_frame` produces a frame that passes `is_valid_enip_frame` (known
  command code, zero payload length, standard zero fields).
- The `split_offset in 1..23` range ensures the first delivery is always a partial header
  (not a complete one), forcing the carry-stash path.
- The `s2c_cmd` values are drawn from the known-command set so they pass the validity gate.
- Proptest default test count (100 cases) is sufficient; the direction-isolation property
  is structural, not numeric.

## Feasibility Assessment

**Assessment: FEASIBLE. Low complexity.**

1. **State-machine test over synthetic sequences:** The property is a state-machine test
   — EnipFlowState is driven by synthetic byte sequences, making it deterministic and
   reproducible. No external I/O or pcap dependency.

2. **Bounded input space:** `split_offset` ranges over 1..23 (22 values); `s2c_cmd` is
   selected from a small set (3 choices). The proptest strategy covers the full variation
   space in a modest number of cases.

3. **EC-X1 repro equivalence:** The two-harness proptest strategy directly encodes the
   EC-X1 repro scenario from `tests/scratch_ecx1_ecx2_repro.rs`. The first harness
   (pdu_count == 2) is the minimal correctness assertion; the second (independent-run
   equivalence) is the general direction-isolation invariant. Both are low-complexity.

4. **Structural correctness:** After the EC-X1 fix, the two-buffer match-arm makes this
   property trivially satisfied by construction. The proptest is a regression guard,
   not a discovery tool — it prevents carry-direction isolation from being accidentally
   violated in future refactors.

5. **Precedent:** VP-014 (HttpAnalyzer Cross-Flow Isolation) uses the same proptest
   pattern (drive state with synthetic sequences; assert count invariants across runs).

**Not Kani because:** The property is about behavioral isolation over sequences, not
about the arithmetic safety of individual function calls. Kani's symbolic execution is
the right tool for arithmetic/bounds properties (VP-032); proptest is the right tool
for sequence/state-machine properties. The match arm that enforces direction isolation
is structurally trivial; what matters is that the full frame-walk state-machine (carry
accumulation + PDU counting) is correct end-to-end.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-033 produced, added to VP-INDEX | draft |
| F3 (story decomposition) | Proptest harnesses assigned to EC-X1 fix story | draft |
| F4 (TDD implementation) | `proptest_vp033_direction_isolation_pdu_count` + `proptest_vp033_independent_run_equivalence` authored and passing | draft → active |
| F6 (formal hardening) | Proptest suite confirmed in CI; no new failures | active → verified |

Lock gate: `status: verified` and `verification_lock: true` set by state-manager after
F6 confirmation.

## VP-INDEX Update Triggered by This VP

When VP-033 is added:
- `total_vps`: 32 → 33
- `p1_count`: 18 → 19
- `proptest_count`: 10 → 11
- `draft` count: 1 → 2 (VP-032 and VP-033 both draft)
- Tool row in VP-INDEX summary: proptest VP-IDs list: append VP-033

These counts must be propagated in the same burst (by spec-steward) to:
1. `VP-INDEX.md` (authoritative source)
2. `verification-architecture.md` (Should Prove table + P1 list + Tooling Selection proptest row)
3. `verification-coverage-matrix.md` (VP-to-Module table + Per-Module table + Totals row)
