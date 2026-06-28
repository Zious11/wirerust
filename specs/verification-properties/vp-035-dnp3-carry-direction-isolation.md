---
artifact: verification-property
vp_id: VP-035
title: "DNP3 Carry-Buffer Direction Isolation"
status: draft
phase: P1
tool: proptest
subsystem: SS-15
module: "src/analyzer/dnp3.rs"
producer: spec-steward
timestamp: 2026-06-27T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
feature_cycle: feature-enip-v0.11.0
issue: "#316"
bcs:
  - BC-2.15.016
ruling: RULING-DNP3-SIBLING-001
verification_lock: false
---

# VP-035: DNP3 Carry-Buffer Direction Isolation

## Property Statement

For any sequence of `on_data` calls with alternating `Direction::ClientToServer` (c2s,
master-to-outstation) and `Direction::ServerToClient` (s2c, outstation-to-master)
deliveries carrying partial-then-complete DNP3 link-layer frames, the `carry_c2s` and
`carry_s2c` buffers are **never mixed**:

- A partial master-to-outstation frame stashed in `carry_c2s` is NEVER prepended to a
  subsequent outstation-to-master delivery.
- A partial outstation-to-master frame stashed in `carry_s2c` is NEVER prepended to a
  subsequent master-to-outstation delivery.
- Per-direction frame counts (`frame_count` contributions from each direction) match the
  counts produced by independent same-direction control runs: injecting interleaved s2c
  deliveries between c2s partial-frame deliveries does NOT alter the c2s frame count, and
  vice versa.

Formally: for any (c2s_frames, s2c_frames) pair of frame sequences with segment boundaries,
running them interleaved through `on_data` with alternating directions produces the same
`frame_count` and `parse_errors` as running each direction's frames independently.

This directly prevents the cross-direction carry splice bug documented as
DRIFT-DNP3-DIRECTION-001 in RULING-DNP3-SIBLING-001: a partial master-to-outstation header
stashed in carry_c2s cannot corrupt the next outstation-to-master delivery's frame-walk loop.

The FIR=1 gate (`transport_is_fir`, transport octet bit 6 `0x40`) does not protect against
the splice: a spliced carry_c2s prepended to an outstation delivery may present a transport
octet with FIR coincidentally set, causing spurious FC classification; or with FIR clear,
suppressing a genuine first-fragment FC from the outstation stream. VP-035 directly verifies
the structural isolation that prevents both false-positive and false-negative FIR paths.

## Verified BCs

| BC-ID | Description | How VP-035 Covers It |
|-------|-------------|----------------------|
| BC-2.15.016 v2.0 | `Dnp3FlowState` frame-walk with per-direction carry buffers | VP-035 verifies Invariant 6 (direction isolation): `carry_c2s` and `carry_s2c` are never mixed; `on_data` selects exactly one buffer per `direction` argument on every call. Also covers EC-010 (partial master-to-outstation stash does not contaminate outstation-to-master delivery). |

### BC Version Note

BC-2.15.016 was amended from v1.6 to v2.0 by RULING-DNP3-SIBLING-001. The v2.0 amendments
relevant to VP-035 are:

- Postcondition 1 (carry prepend): incoming bytes appended to
  `(match direction { ClientToServer => flow.carry_c2s, ServerToClient => flow.carry_s2c })`
- Postcondition 2 (cap check): `flow.carry_c2s.len() <= 292` AND `flow.carry_s2c.len() <= 292`
  — each directional carry independently bounded at MAX_DNP3_FRAME_LEN
- Postcondition 3 (frame consume): `active_carry.drain(..frame_len)` where `active_carry` is
  `carry_c2s` or `carry_s2c` per direction
- Postcondition 4 (residual stash): directional carry (not unified)
- Invariant 1 (carry bounded): `flow.carry_c2s.len() <= 292` AND `flow.carry_s2c.len() <= 292`
- Invariant 6 (new): `carry_c2s` and `carry_s2c` are NEVER mixed. `on_data` selects exactly
  one of the two buffers based on the `direction` argument on every call
- EC-010 (new): Partial master-to-outstation frame stashed in carry_c2s; next call is
  outstation-to-master direction → `carry_s2c` is prepended to s2c data (`carry_c2s` NOT
  involved); s2c frame processes cleanly; `carry_c2s` retains partial c2s bytes

## Relationship to VP-033 (ENIP Analog)

VP-035 is structurally identical to VP-033 (EtherNet/IP Carry-Buffer Direction Isolation)
but targets `src/analyzer/dnp3.rs` instead of `src/analyzer/enip.rs`. The same two-harness
proptest strategy applies. Key differences:

- **Frame format:** DNP3 uses `[0x05, 0x64]` sync word + LENGTH byte (10-byte minimum header)
  vs. ENIP's 24-byte header. Minimal test frame: 10 bytes (sync + LENGTH=8 + DEST + SRC +
  CTRL + CRC).
- **FIR=1 gate:** DNP3 FIR=1 gating applies only to FC extraction, not to frame counting.
  VP-035 verifies `frame_count` (raw frame delivery count), not `pdu_count`. The direction
  isolation invariant holds at the frame-walk level regardless of FIR.
- **BC trace:** BC-2.15.016 v2.0 Invariant 6 (vs. BC-2.17.016 v2.0 Invariant 7 for ENIP).

## Purity Classification

**Pure-core with controlled state injection.** The proptest strategy drives `Dnp3FlowState`
directly without any I/O. The test constructs synthetic byte sequences (valid DNP3 link-layer
headers with known sync/length fields), calls `on_data` with alternating directions, and
asserts count invariants. No file I/O, no network, no global state.

**Why proptest and NOT Kani:** The direction-isolation invariant is trivially enforced by
the `match direction { ClientToServer => ..., ServerToClient => ... }` match arm in the
production code. The operationally meaningful check is the behavioral invariant: that
interleaving s2c deliveries between c2s partial-frame deliveries does NOT corrupt frame
counts or parse_errors. This is a state-machine property over sequences — the natural
domain of proptest. The DRIFT-DNP3-DIRECTION-001 repro was a sequence-based scenario;
proptest directly encodes the same evidence.

## Proof Harness Skeleton

```rust
#[cfg(test)]
mod vp035_dnp3_carry_direction_isolation {
    use super::*;
    use proptest::prelude::*;

    /// VP-035: carry_c2s and carry_s2c are never mixed across directions (DNP3).
    ///
    /// Strategy: generate a valid c2s partial DNP3 frame (bytes 0..split_offset,
    /// i.e., partial header but not yet complete), a valid complete s2c DNP3 frame
    /// (full 10-byte header with valid sync + minimal payload), and a completing c2s
    /// delivery (the remaining bytes of the c2s frame).
    ///
    /// Assert: after the interleaved run, frame_count == 2 (one c2s frame + one s2c
    /// frame) and parse_errors == 0. Compare against two independent same-direction
    /// control runs that together also produce frame_count == 2 and parse_errors == 0.
    ///
    /// This directly encodes the DRIFT-DNP3-DIRECTION-001 repro: before the fix, the
    /// interleaved run produced spurious parse_errors from the spliced carry.
    proptest! {
        #[test]
        fn proptest_vp035_direction_isolation_frame_count(
            // Segment split point: where to cut the c2s frame's first delivery
            // Must be in 1..9 (partial header, not yet complete 10-byte minimum)
            split_offset in 1usize..9,
            // s2c direction flag (always ServerToClient for the interleaved delivery)
            // s2c frame uses CTRL=0x44 (outstation response direction bit = bit7 clear for
            // standard response; actual direction bit is bit7 of CTRL per IEEE 1815)
            // _s2c_ctrl is declared but unused: s2c CTRL is hardcoded to 0x44 below because
            // ServerToClient frames always use the outstation-response value; varying it
            // would produce invalid frames outside this VP's scope.
            _s2c_ctrl in 0x00u8..=0xFFu8,
        ) {
            let mut state = Dnp3FlowState::default();
            let ts: u32 = 100;

            // Build a valid c2s (master-to-outstation) DNP3 frame:
            // sync=[0x05,0x64], LENGTH=5 (minimum: 10 bytes total), DEST=1, SRC=3,
            // CTRL=0xC4 (DIR=1, master frame), CRC=0x0000 (test only — CRC check
            // is not enforced in on_data frame-walk for this VP scope)
            let c2s_frame = build_minimal_dnp3_frame(0xC4u8); // DIR bit set = master
            // Build a valid s2c (outstation-to-master) DNP3 frame:
            // CTRL=0x44 (DIR=0, outstation response)
            let s2c_frame = build_minimal_dnp3_frame(0x44u8); // DIR bit clear = outstation

            // Delivery 1: partial c2s header (bytes 0..split_offset) → stashed in carry_c2s
            state.on_data_test(&c2s_frame[..split_offset], ts, Direction::ClientToServer);
            prop_assert_eq!(state.parse_errors, 0,
                "partial delivery must not produce parse errors");

            // Delivery 2: complete s2c frame → carry_s2c used (carry_c2s NOT involved)
            state.on_data_test(&s2c_frame, ts, Direction::ServerToClient);
            prop_assert_eq!(state.parse_errors, 0,
                "s2c delivery must not produce parse errors from c2s carry contamination");

            // Delivery 3: completing c2s bytes (split_offset..end) → carry_c2s prepended
            state.on_data_test(&c2s_frame[split_offset..], ts, Direction::ClientToServer);
            prop_assert_eq!(state.parse_errors, 0,
                "completing c2s delivery must not produce parse errors");

            // Verify carry buffers are both empty (all frames fully consumed)
            prop_assert!(state.carry_c2s.is_empty(),
                "carry_c2s must be drained after complete frame");
            prop_assert!(state.carry_s2c.is_empty(),
                "carry_s2c must be drained after complete frame");
        }

        #[test]
        fn proptest_vp035_independent_run_equivalence(
            split_offset in 1usize..9,
        ) {
            // Interleaved run
            let mut interleaved = Dnp3FlowState::default();
            let c2s_frame = build_minimal_dnp3_frame(0xC4u8);
            let s2c_frame = build_minimal_dnp3_frame(0x44u8);
            interleaved.on_data_test(&c2s_frame[..split_offset], 100, Direction::ClientToServer);
            interleaved.on_data_test(&s2c_frame, 100, Direction::ServerToClient);
            interleaved.on_data_test(&c2s_frame[split_offset..], 100, Direction::ClientToServer);

            // Independent c2s-only run
            let mut c2s_only = Dnp3FlowState::default();
            c2s_only.on_data_test(&c2s_frame[..split_offset], 100, Direction::ClientToServer);
            c2s_only.on_data_test(&c2s_frame[split_offset..], 100, Direction::ClientToServer);

            // Independent s2c-only run
            let mut s2c_only = Dnp3FlowState::default();
            s2c_only.on_data_test(&s2c_frame, 100, Direction::ServerToClient);

            // Invariant: interleaved frame_count == sum of independent frame_counts
            prop_assert_eq!(
                interleaved.frame_count,
                c2s_only.frame_count + s2c_only.frame_count,
                "interleaved frame_count must equal sum of independent runs"
            );
            prop_assert_eq!(
                interleaved.parse_errors,
                c2s_only.parse_errors + s2c_only.parse_errors,
                "interleaved parse_errors must equal sum of independent runs"
            );
        }
    }

    /// Build a minimal valid DNP3 link-layer frame with given CTRL byte.
    /// Returns a Vec<u8> of exactly 10 bytes:
    ///   [0x05, 0x64, LENGTH=0x05, CTRL, DEST_LO, DEST_HI, SRC_LO, SRC_HI, CRC_LO, CRC_HI]
    /// LENGTH=0x05 means 5 link-layer bytes (minimum), total frame = 10 bytes.
    /// CRC bytes are set to 0x00 — tests that skip CRC validation accept this.
    fn build_minimal_dnp3_frame(ctrl: u8) -> Vec<u8> {
        vec![
            0x05, 0x64, // sync bytes [0x05, 0x64]
            0x05,       // LENGTH = 5 (minimum; total 10 bytes)
            ctrl,       // CTRL (direction bit = bit7: 0x80 set = master frame)
            0x01, 0x00, // DEST = 1 (little-endian)
            0x03, 0x00, // SRC = 3 (little-endian)
            0x00, 0x00, // CRC (0x00 — not enforced in frame-walk for this VP)
        ]
    }
}
```

### Implementation Notes

- `on_data_test` is a thin test-visible wrapper around the production `on_data` logic
  (or `on_data` itself if made `pub(crate)` with a `#[cfg(test)]` companion). It mirrors
  the pattern used for VP-033's `on_data_test` wrapper in `EnipFlowState`.
- `build_minimal_dnp3_frame` produces a 10-byte DNP3 frame with valid sync bytes and
  LENGTH=5. The frame passes the sync gate (`[0x05, 0x64]`) and `compute_dnp3_frame_len`
  minimum check.
- The `split_offset in 1..9` range ensures the first delivery is always a partial header
  (not a complete 10-byte minimum), forcing the carry-stash path.
- CRC validation (`0x00` CRC bytes): the frame-walk loop's CRC check must either be
  bypassed in tests (via a `#[cfg(test)]` skip flag) or the harness must compute valid
  CRCs. The VP is valid either way; implementer chooses based on code structure.
- The `frame_count` field tracks raw frame deliveries at the frame-walk level, independent
  of FIR=1 gating (which affects FC extraction, not frame counting).
- Proptest default test count (100 cases) is sufficient; the direction-isolation property
  is structural, not numeric.

## Feasibility Assessment

**Assessment: FEASIBLE. Low complexity.**

1. **State-machine test over synthetic sequences:** The property is a state-machine test
   — Dnp3FlowState is driven by synthetic byte sequences, making it deterministic and
   reproducible. No external I/O or pcap dependency.

2. **Bounded input space:** `split_offset` ranges over 1..9 (8 values). The proptest
   strategy covers the full variation space in a modest number of cases.

3. **DRIFT-DNP3-DIRECTION-001 repro equivalence:** The two-harness proptest strategy
   directly encodes the DRIFT-DNP3-DIRECTION-001 repro scenario from RULING-DNP3-SIBLING-001
   §1.1. The first harness (isolation + no parse_errors) is the minimal correctness assertion;
   the second (independent-run equivalence) is the general direction-isolation invariant.

4. **Structural correctness:** After the carry split, the two-buffer match-arm makes this
   property trivially satisfied by construction. The proptest is a regression guard, not a
   discovery tool — it prevents carry-direction isolation from being accidentally violated
   in future refactors.

5. **Precedent:** VP-033 (EnipFlowState cross-flow isolation) uses the identical proptest
   pattern. VP-014 (HttpAnalyzer Cross-Flow Isolation) also uses the same pattern.

**Not Kani because:** The property is about behavioral isolation over sequences, not about
arithmetic safety of individual functions. The match arm enforcing direction isolation is
structurally trivial; what matters is that the full frame-walk state machine (carry
accumulation + frame counting) is correct end-to-end.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-035 produced, added to VP-INDEX | draft |
| F3 (story decomposition) | Proptest harnesses assigned to STORY-140 (DNP3 carry split) | draft |
| F4 (TDD implementation) | `proptest_vp035_direction_isolation_frame_count` + `proptest_vp035_independent_run_equivalence` authored and passing | draft → active |
| F6 (formal hardening) | Proptest suite confirmed in CI; no new failures | active → verified |

Lock gate: `status: verified` and `verification_lock: true` set by state-manager after
F6 confirmation.

## VP-INDEX Update Triggered by This VP

When VP-035 is added (after VP-034):
- `total_vps`: 34 → 35
- `p1_count`: 20 → 21
- `proptest_count`: 12 → 13
- `draft` count: 3 → 4 (VP-032, VP-033, VP-034, VP-035 all draft)
- Tool row in VP-INDEX summary: proptest VP-IDs list: append VP-035

These counts must be propagated in the same burst (by spec-steward) to:
1. `VP-INDEX.md` (authoritative source)
2. `verification-architecture.md` (Should Prove table + P1 list + Tooling Selection proptest row)
3. `verification-coverage-matrix.md` (VP-to-Module table + analyzer/dnp3.rs Per-Module row + Totals row)
