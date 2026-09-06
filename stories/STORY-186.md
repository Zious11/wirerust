---
document_type: story
level: ops
story_id: STORY-186
title: "S7comm ISO-on-TCP Carry-Buffer Reassembly, Walk-First Frame Extraction, Resync, and the Frozen SS-20/SS-21 Module Boundary"
epic_id: E-23
version: "1.0"
status: ready
producer: story-writer
timestamp: 2026-09-06T00:00:00Z
phase: f3
traces_to: .factory/specs/prd.md
points: 5
priority: P1
cycle: feature-s7comm
wave: 89
target_module: analyzer/s7comm
subsystems: [SS-20, SS-21]
estimated_days: null
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
feature_id: feature-s7comm
depends_on: [STORY-185]
blocks: [STORY-187]
behavioral_contracts: [BC-2.20.013, BC-2.20.014, BC-2.20.015, BC-2.20.016, BC-2.21.003]
verification_properties: [VP-050]
inputs:
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.013.md
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.014.md
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.015.md
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.016.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.003.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
  - .factory/cycles/feature-s7comm/f2-pcap-fixture-sourcing.md
input-hash: "ce86f8c"
---

> **tdd_mode:** `strict` — full TDD Iron Law enforced.

# STORY-186: S7comm ISO-on-TCP Carry-Buffer Reassembly, Walk-First Frame Extraction, Resync, and the Frozen SS-20/SS-21 Module Boundary

## Narrative

**As a** security analyst using wirerust to inspect S7comm traffic spanning TCP segment
boundaries,
**I want** `S7commAnalyzer` to correctly reassemble TPKT frames split across `on_data`
calls via directional carry buffers, resync on malformed input without ever discarding
an already-complete frame, and enforce the architectural boundary that keeps SS-20
stateless and S7comm-agnostic,
**so that** multi-segment TPKT delivery is handled correctly, carry-overflow DoS attempts
are detected (T0814), and a future IEC 61850 MMS or ICCP/TASE.2 cycle can reuse SS-20
without modification.

This story creates `src/analyzer/s7comm.rs` (SS-21) for the first time: `S7commAnalyzer`,
a minimal `S7commFlowState` (carry fields only — extended with the remaining
classification/dedup fields in STORY-187), and the frame-walk loop in `on_data` that
drives STORY-184/185's `parse_tpkt_header`/`parse_cotp_header`. Protocol-specific
dispatch on the extracted `CotpHeader::protocol_id` (the protocol-ID dispatch contract
delivered by STORY-187) is **not** built here — this story only proves that complete
TPKT frames are correctly extracted, walked, and
carried across calls. STORY-187 wires the extraction output into S7comm-specific
classification.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.20.013 | TPKT Frames Spanning TCP Segments Reassembled via Directional Carry Buffers, Walk-First Residual-Bound Semantics | Frame-walk loop, no aggregate pre-check |
| BC-2.20.014 | Carry Buffer Bounded at MAX_S7_ISO_ON_TCP_CARRY_BYTES=65,535; Overflow Triggers Clear-and-Resync With One T0814 Per Direction | Overflow reaction + dedup |
| BC-2.20.015 | Resync Anchor Advances Exactly 1 Byte Per Iteration on a Bad TPKT Version Byte (Never 2) | Resync correctness |
| BC-2.20.016 | Frozen `iso_on_tcp.rs` Module Boundary — Pure Free Functions Only, No StreamAnalyzer Impl, No Per-Flow State of Its Own | Architectural boundary contract |
| BC-2.21.003 | `on_flow_close` Removes `S7commFlowState` and Discards All Carry Bytes | Flow lifecycle teardown (packaged here with flow-map creation, not with STORY-187's classification dispatch) |

## Acceptance Criteria

### AC-186-001: Frame-walk loop extracts every complete TPKT frame before any byte-count bound is applied
(traces to BC-2.20.013 postcondition 1)
- Given `working = carry[direction] ++ incoming_data` for a flow's `on_data` call
- When the frame-walk loop runs
- Then it repeatedly calls `parse_tpkt_header(&working[cursor..])`: a complete frame
  (`Some(header)` and `working.len() - cursor >= header.length as usize`) is extracted
  and dispatched to `parse_cotp_header`, `cursor += header.length as usize`, and the loop
  continues; a declared-but-incomplete frame or a `None` result breaks the loop and
  stashes `working[cursor..]` to `carry[direction]` (traces to BC-2.20.013
  postcondition 1, sub-clauses a/b/c)
- No aggregate `carry[direction].len() + incoming_data.len()` pre-check exists anywhere
  in the implementation — the walk always runs first (traces to BC-2.20.013
  postcondition 2, invariant 1)
- **Test:** `test_BC_2_20_013_walk_first_no_aggregate_precheck`

### AC-186-002: An adversarial burst with a complete frame at the head is never dropped
(traces to BC-2.20.013 invariant 1)
- Given one `on_data` call delivering `[complete 7-byte CR frame][60,000 bytes of
  trailing garbage]`
- When `on_data` processes this delivery
- Then the 7-byte CR frame is extracted regardless of the trailing garbage's size — this
  is the anti-evasion property (Ptacek/Newsham-class evasion channel) this design
  prevents, mirroring the IEC-104 F-172-001 and DNP3 F-B-002 rulings
- **Test:** `test_BC_2_20_013_adversarial_burst_head_frame_not_dropped`

### AC-186-003: Split-frame reassembly across two on_data calls
(traces to BC-2.20.013 edge case EC-002)
- Given call 1 delivers `[0x03, 0x00, 0x00, 0x0A]` (4-byte TPKT header declaring
  `length=10`) and call 2 delivers the remaining 6 bytes
- When both calls complete
- Then call 1 stashes the 4-byte header-only partial to carry (declared-but-incomplete);
  call 2's `working = carry ++ new_bytes` contains the complete 10-byte frame, which is
  extracted and `carry[direction]` is empty afterward
- **Test:** `test_BC_2_20_013_split_frame_across_two_calls`

### AC-186-004: Carry buffer bounded at 65,535 bytes; at-bound residual is legitimate, not overflow
(traces to BC-2.20.014 invariant 1) (traces to BC-2.20.014 edge case EC-001)
- Given a residual of exactly 65,535 bytes from a still-incomplete, conformant
  `length=65,535` frame
- When the residual-bound check runs
- Then no overflow is triggered (comparison is strict `>`, not `>=`); the residual is
  retained in carry unchanged
- **Test:** `test_BC_2_20_014_at_bound_residual_no_overflow`

### AC-186-005: Carry overflow clears the direction's carry, resyncs, and emits exactly one T0814 per direction
(traces to BC-2.20.014 postcondition 1) (traces to BC-2.20.014 postcondition 3)
- Given `residual.len() > MAX_S7_ISO_ON_TCP_CARRY_BYTES` (i.e. `> 65,535`) at the start of
  an `on_data` call, before the current delivery is appended and the walk begins
- When the overflow check fires
- Then `carry[direction]` is cleared (not truncated); the walk resyncs (BC-2.20.015); and
  exactly one T0814 finding (`ThreatCategory::Anomaly`, `Verdict::Possible`,
  `Confidence::Medium`) is emitted for this direction, guarded by
  `carry_overflow_reported_c2s`/`_s2c` (traces to BC-2.20.014 postcondition 4 — this
  dedup flag is distinct from any malformed-header dedup flag introduced in STORY-187)
- A second overflow event in the same direction on the same flow does not re-emit
  (traces to BC-2.20.014 edge case EC-004)
- **Test:** `test_BC_2_20_014_overflow_clear_resync_one_t0814_per_direction`,
  `test_BC_2_20_014_repeated_overflow_dedup_same_direction`

### AC-186-006: Overflow dedup flags are independent per direction
(traces to BC-2.20.014 edge case EC-005)
- Given an overflow event in the `c2s` direction on a flow
- When an independent overflow event subsequently occurs in the `s2c` direction on the
  same flow
- Then the `s2c` overflow emits its own T0814 finding — the `c2s` dedup flag has no
  bearing on `s2c`
- **Test:** `test_BC_2_20_014_overflow_dedup_independent_per_direction`

### AC-186-007: Resync advances exactly 1 byte per iteration on a bad version byte, never 2
(traces to BC-2.20.015 postcondition 1) (traces to BC-2.20.015 invariant 1)
- Given bytes `[0x01, 0x03, 0x00, 0x00, 0x04]` (a spurious `0x01` immediately followed by
  a valid frame at offset 1)
- When the frame-walk loop's resync sub-routine runs
- Then the valid frame at offset 1 is found; a 2-byte advance would have skipped it
  entirely (landing at offset 2, `0x00`)
- **Test:** `test_BC_2_20_015_resync_advances_exactly_one_byte`

### AC-186-008: Resync sub-routine is reused verbatim for both bad-version-byte and post-overflow conditions
(traces to BC-2.20.015 invariant 3)
- Given a bad-version-byte condition encountered mid-stream (per STORY-184's version-byte
  reject path) and a post-carry-overflow resync (BC-2.20.014)
- When either condition triggers a resync
- Then both invoke the same 1-byte-advance resync sub-routine — there is exactly one
  resync implementation, not two
- **Test:** `test_BC_2_20_015_single_resync_implementation_shared`

### AC-186-009: Resync always terminates for finite input
(traces to BC-2.20.015 invariant 2)
- Given a long run of non-`0x03` garbage bytes (e.g. 200 bytes) with no valid frame
  anywhere in the remaining input
- When the resync sub-routine runs
- Then it advances to the end of the input without an infinite loop; the (now sub-4-byte)
  remainder is stashed to carry per the ordinary incomplete-frame path
- **Test:** `test_BC_2_20_015_resync_terminates_no_valid_anchor`

### AC-186-010: `iso_on_tcp.rs` contains no `impl StreamAnalyzer` block
(traces to BC-2.20.016 postcondition 1)
- Given `src/analyzer/iso_on_tcp.rs`
- When a static regression-guard test inspects the module
- Then it contains zero `impl StreamAnalyzer for ...` blocks and zero
  `DispatchTarget::IsoOnTcp`-shaped references
- **Test:** `test_BC_2_20_016_iso_on_tcp_has_no_stream_analyzer_impl` (grep-equivalent
  static assertion)

### AC-186-011: TPKT/COTP carry buffers live on `S7commFlowState`, not a separate `IsoOnTcpFlowState`
(traces to BC-2.20.016 postcondition 3)
- Given `S7commFlowState` (created in this story)
- When its fields are inspected
- Then `carry_c2s: Vec<u8>`, `carry_s2c: Vec<u8>`, `carry_overflow_reported_c2s: bool`,
  `carry_overflow_reported_s2c: bool` are fields on `S7commFlowState` (SS-21); no
  `IsoOnTcpFlowState` type exists anywhere in the tree
- **Test:** `test_BC_2_20_016_no_iso_on_tcp_flow_state_type_exists` (grep-equivalent
  static assertion)

### AC-186-012: `on_flow_close` removes `S7commFlowState` and discards carry bytes with no finding
(traces to BC-2.21.003 postconditions 1-4, forward-referenced here as flow-lifecycle
infrastructure this story's `on_data`/`on_flow_close` pair requires to exist; the full
`S7commFlowState` struct is completed in STORY-187)
- Given a flow with active `S7commFlowState` (possibly non-empty carry buffers)
- When `S7commAnalyzer::on_flow_close(flow_key)` is called
- Then the flow's state is removed from the analyzer's per-flow map; carry bytes are
  dropped with no finding emitted; calling `on_flow_close` for an unknown `flow_key` is a
  no-op
- **Test:** `test_s7comm_on_flow_close_removes_state_discards_carry`

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `S7commAnalyzer` struct | SS-21 analyzer | `src/analyzer/s7comm.rs` | Effectful (owns `flows: HashMap<FlowKey, S7commFlowState>`) |
| `S7commFlowState` (minimal: carry fields only) | SS-21 per-flow state | `src/analyzer/s7comm.rs` | Mutable state |
| `MAX_S7_ISO_ON_TCP_CARRY_BYTES` const | SS-21 constants | `src/analyzer/s7comm.rs` | N/A |
| `S7commAnalyzer::on_data` (frame-walk loop only) | SS-21 effectful shell | `src/analyzer/s7comm.rs` | Effectful |
| `S7commAnalyzer::on_flow_close` | SS-21 lifecycle | `src/analyzer/s7comm.rs` | Effectful |
| `parse_tpkt_header`, `parse_cotp_header` (consumed, unchanged) | SS-20 | `src/analyzer/iso_on_tcp.rs` | Pure (unchanged by this story) |

Subsystem anchors:
- SS-21 owns `S7commAnalyzer`/`S7commFlowState`/`on_data`/`on_flow_close` per
  ARCH-INDEX.md §SS-21 — this is the first story to create `s7comm.rs`.
- SS-20 owns the frozen module-boundary contract (BC-2.20.016) this story verifies:
  `iso_on_tcp.rs` remains untouched, stateless, and the sole consumer relationship
  (SS-21 imports SS-20's pure functions) is established here for the first time.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `parse_tpkt_header`, `parse_cotp_header` (from STORY-184/185) | pure-core | Unchanged; consumed by value |
| `S7commAnalyzer::on_data` | effectful-shell | Mutates `S7commFlowState` (carry buffers, dedup flags), calls `emit_finding`-equivalent for T0814 on overflow |
| `S7commAnalyzer::on_flow_close` | effectful-shell | Removes map entry |

## VP-050 Proptest Obligation

**Harnesses:** `proptest_vp050_walk_first_residual_bound`,
`proptest_vp050_direction_isolation`, `proptest_vp050_resync_one_byte_advance`
(anchored in this story)
**Method:** proptest
**Priority:** P1

Skeleton (in `tests/s7comm_analyzer_tests.rs`):

```rust
proptest! {
    #[test]
    fn proptest_vp050_direction_isolation(
        c2s_data in prop::collection::vec(any::<u8>(), 0..300),
        s2c_data in prop::collection::vec(any::<u8>(), 0..300),
    ) {
        let mut analyzer = S7commAnalyzer::new();
        let flow_key = FlowKey::new(
            "127.0.0.1".parse().unwrap(), 1234,
            "127.0.0.2".parse().unwrap(), 102,
        );
        analyzer.on_data(flow_key.clone(), &c2s_data, 0, Direction::ClientToServer);
        analyzer.on_data(flow_key.clone(), &s2c_data, 0, Direction::ServerToClient);
        // carry_c2s must only ever contain bytes routed via C2S; carry_s2c only S2C;
        // each carry stays <= MAX_S7_ISO_ON_TCP_CARRY_BYTES (65,535).
    }
}
```

Full proptest run (including the walk-first equivalence property: splitting a byte
sequence into `carry + incoming` yields the identical result as running the walk once on
the concatenated bytes) is executed in STORY-194.

## Tasks

- [ ] Create `src/analyzer/s7comm.rs` with a module-level doc comment citing ADR-014
      Decisions 1, 2, 8 (SS-21 owns the frame-walk loop and all per-flow state; SS-20
      remains stateless)
- [ ] Define `const MAX_S7_ISO_ON_TCP_CARRY_BYTES: usize = 65_535;`
- [ ] Define a minimal `S7commFlowState` with exactly: `carry_c2s: Vec<u8>`,
      `carry_s2c: Vec<u8>`, `carry_overflow_reported_c2s: bool`,
      `carry_overflow_reported_s2c: bool` (STORY-187 extends this struct with the
      classification/dedup fields its own scope requires — do not pre-add those fields
      here)
- [ ] Implement `S7commAnalyzer` struct with `flows: HashMap<FlowKey, S7commFlowState>`
- [ ] Implement `S7commAnalyzer::on_data(&mut self, flow_key: FlowKey, data: &[u8],
      ts: u32, direction: Direction)`:
  - overflow check at entry on the directional carry (before appending/walking) per
    BC-2.20.014 walk-first-residual-bound semantics
  - frame-walk loop: extract complete TPKT frames via `iso_on_tcp::parse_tpkt_header`
    then `iso_on_tcp::parse_cotp_header`; for this story, dispatch is a no-op placeholder
    (classification lands in STORY-187) — the loop's job here is proven extraction and
    carry management only
  - bad-version-byte / post-overflow resync: 1-byte advance, shared sub-routine
    (BC-2.20.015)
- [ ] Implement `S7commAnalyzer::on_flow_close(&mut self, flow_key: FlowKey)`:
  `self.flows.remove(&flow_key)`
- [ ] Write `proptest_vp050_*` skeletons in `tests/s7comm_analyzer_tests.rs`
- [ ] Write unit tests: one per AC, named `test_BC_2_20_013_*` .. `test_BC_2_20_016_*`
- [ ] Write the two static regression-guard tests (AC-186-010, AC-186-011)
- [ ] Verify `cargo test` passes
- [ ] Add a CHANGELOG entry under `[Unreleased] > Added` describing the new
      `S7commAnalyzer` skeleton and carry-buffer reassembly, before creating the PR

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.20.013 | Single `on_data` call delivers exactly one complete frame, no carry before/after | Frame extracted; carry remains empty |
| EC-002 | BC-2.20.013 | TCP segment delivers two complete frames back-to-back plus a partial third | Both complete frames extracted in the same call; only the partial third stashed |
| EC-003 | BC-2.20.014 | `residual.len() == 65,535` exactly (at bound, legitimate) | No overflow; comparison is strict `>`, not `>=` |
| EC-004 | BC-2.20.014 | `residual.len() == 65,536` (one over bound, adversarial) | Carry cleared; resync; exactly one T0814 |
| EC-005 | BC-2.20.015 | `0x03` byte exists but starts a frame with an invalid length field (`< 4`) | Resync finds this `0x03`, `parse_tpkt_header` returns `None` again (different reject reason), walk continues advancing 1 byte past it — never stuck retrying the same offset |
| EC-006 | BC-2.20.016 | A future MMS/ICCP cycle wants to reuse `parse_tpkt_header`/`parse_cotp_header` | Imports them directly; defines its own analogous flow-state fields — zero lines of `iso_on_tcp.rs` change |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~5,200 |
| BC-2.20.013-016 (4 BCs, higher density) | ~6,000 |
| ADR-014 (Decisions 1, 2, 8, 9) | ~10,000 |
| src/analyzer/iso_on_tcp.rs (from STORY-184/185) | ~4,000 |
| Test file (new `s7comm_analyzer_tests.rs`) | ~2,500 |
| **Total** | **~27,700** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~14%** |

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-184/185 | `TpktHeader`/`CotpHeader` frozen structs; `parse_tpkt_header`/`parse_cotp_header` pure free fns | SS-20 is genuinely stateless and protocol-agnostic | `parse_cotp_header`'s `protocol_id` extraction is a total identity mapping — this story's frame-walk loop must not add any `0x32`/`0x72` interpretation; that belongs entirely to STORY-187's `S7commAnalyzer::on_data` dispatch extension |

This story is the S7comm/ISO-on-TCP analogue of IEC-104's STORY-172 (carry buffers +
frame-walk loop), but positioned *earlier* in the sequence relative to classification
work (IEC-104's STORY-172 came after its classification stories) because the SS-20/SS-21
split means TPKT/COTP frame extraction is architecturally prior to any S7comm-specific
dispatch. `S7commAnalyzer::on_data` is therefore built incrementally: this story proves
extraction/carry/resync; STORY-187 adds the four-way `protocol_id` dispatch; STORY-188/189
add classification; STORY-190 completes the dispatch table; STORY-191/192 add MITRE
emission.

## Architecture Compliance Rules

Extracted from `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md`:
- **ADR-014 Decision 1**: SS-20 (`iso_on_tcp.rs`) is deliberately stateless. The
  directional carry buffers required for TPKT reassembly are fields on `S7commFlowState`
  (SS-21), never on a hypothetical `IsoOnTcpFlowState`.
- **ADR-014 Decision 2**: No `DispatchTarget::IsoOnTcp` variant is introduced at any
  point — SS-20 is a parsing library consumed by `S7commAnalyzer`, not an independent
  dispatch target.
- **ADR-014 Decision 8 (WALK-FIRST-RESIDUAL-BOUND)**: `MAX_S7_ISO_ON_TCP_CARRY_BYTES =
  65,535` is derived from the TPKT `length` field's own maximum (`u16::MAX`), not COTP's
  254-byte LI maximum. The frame-walk loop runs unconditionally on carry + incoming data;
  the byte bound applies only to the leftover partial-frame residual. No aggregate
  carry-plus-delivery pre-check may exist anywhere (anti-evasion, mirrors IEC-104
  F-172-001 / DNP3 F-B-002).
- Resync anchor is the TPKT version byte (`0x03`); advance exactly 1 byte per iteration,
  never 2, on a bad-version-byte or post-overflow condition.
- Pure/effectful boundary: `parse_tpkt_header`/`parse_cotp_header` remain pure; `on_data`
  and `on_flow_close` are the effectful shell.

## Library & Framework Requirements

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | `Vec<u8>`, `HashMap`, bounds arithmetic |
| proptest | 1 (pinned in `Cargo.toml`) | VP-050 direction-isolation and walk-first-equivalence skeletons |

No new external crate dependencies.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `src/analyzer/s7comm.rs` | CREATE | `S7commAnalyzer`, minimal `S7commFlowState` (carry fields only), `MAX_S7_ISO_ON_TCP_CARRY_BYTES`, `on_data` frame-walk loop, `on_flow_close` |
| `src/analyzer/mod.rs` | MODIFY | Add `pub mod s7comm;` (module created but not yet registered with the dispatcher — that is STORY-193) |
| `tests/s7comm_analyzer_tests.rs` | CREATE | Unit tests for BC-2.20.013-016 + VP-050 proptest skeletons + static regression-guard tests |

## Forbidden Dependencies

- `rusty-cotp`, `rusty-tpkt`, `tpkt`, `copt`, `s7`, `s7-comm`, `s7-client`, Wireshark,
  Snap7, libnodave source — banned/avoid per ADR-014 Decision 4
- `src/analyzer/s7comm.rs` MUST NOT define a type named `IsoOnTcpFlowState` (BC-2.20.016
  postcondition 3) — carry buffers belong on `S7commFlowState` exclusively
- `carry_c2s` and `carry_s2c` MUST NOT be merged into a single shared `Vec<u8>` — this
  would violate RULING-DNP3-SIBLING-001's directional-isolation requirement

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-06 | story-writer | Initial authorship — `s7comm.rs` created, carry-buffer reassembly, walk-first frame extraction, resync, frozen SS-20/SS-21 boundary regression guards, VP-050 skeleton, AC-186-001..012. |
