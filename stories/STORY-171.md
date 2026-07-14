---
document_type: story
story_id: STORY-171
title: "IEC-104 N(S)/N(R) Sequence Tracking: Option<u16> First-Frame Guard + Desync Detection"
epic_id: E-22
wave: 80
points: 3
phase: f3
tdd_mode: strict
status: draft
feature_id: feature-iec104
subsystems: [SS-19]
target_module: analyzer/iec104
depends_on: [STORY-168, STORY-170]
blocks: [STORY-172]
behavioral_contracts:
  - BC-2.19.023
  - BC-2.19.024
verification_properties:
  - VP-045
  - VP-047
inputs:
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.023.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.024.md
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "4ed11eb"
---

# STORY-171: IEC-104 N(S)/N(R) Sequence Tracking: Option<u16> First-Frame Guard + Desync Detection

## Narrative

**As a** security analyst using wirerust to detect adversarial IEC-104 sequence manipulation,
**I want** the analyzer to track per-direction N(S) sequence numbers with an `Option<u16>`
first-frame guard,
**so that** sequence-number desynchronization attacks (gap > k=12, T1692.001 Possible) are
detected without false positives on mid-capture starts where the first observed N(S) is arbitrary.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.19.023 | N(S)/N(R) 15-Bit Sequence Numbers Extracted Correctly from CF1–CF4 | Core extraction — N(S) from CF1/CF2, N(R) from CF3/CF4 |
| BC-2.19.024 | N(S) Gap > k=12 Emits T1692.001 Sequence-Desync Finding | Gap detection + Option<u16> first-frame guard |

## Acceptance Criteria

### AC-171-001: N(S) correctly extracted from I-format CF1/CF2
**Traces to:** BC-2.19.023 postconditions 1 and 3
- Given an I-format APCI frame with CF1 and CF2 control field bytes
- When `extract_ns(cf1: u8, cf2: u8) -> u16` is called
- Then `ns = ((cf1 as u16) >> 1) | ((cf2 as u16) << 7)` — range [0, 32767]
- Canonical test vectors: CF1=0x02/CF2=0x00 → N(S)=1; CF1=0xFE/CF2=0xFF → N(S)=32767

### AC-171-002: N(R) correctly extracted from I/S-format CF3/CF4
**Traces to:** BC-2.19.023 postconditions 2 and 4
- Given CF3 and CF4 control field bytes from an I- or S-format frame
- When `extract_nr(cf3: u8, cf4: u8) -> u16` is called
- Then `nr = ((cf3 as u16) >> 1) | ((cf4 as u16) << 7)` — range [0, 32767]
- N(R) is computed and available but NOT stored in `Iec104FlowState` (no `last_nr` field)

### AC-171-003: First I-frame sets Option<u16> baseline with no finding
**Traces to:** BC-2.19.024 postcondition (Path A) and invariant 3
- Given `Iec104FlowState::last_ns_c2s` (or `last_ns_s2c`) is `None` (fresh flow or mid-capture start)
- When the first I-format frame with any N(S) value is received
- Then the directional field is set to `Some(ns)`; NO finding is emitted unconditionally
- This is the mid-capture correctness guard: first observed N(S) may be arbitrary (e.g., 5000)
  and must never generate a desync finding regardless of its value

### AC-171-004: Subsequent frame with gap ≤ 12 updates state with no finding
**Traces to:** BC-2.19.024 postcondition (Path B)
- Given `last_ns_dir` is `Some(prev)` and `(current_ns.wrapping_sub(prev) & 0x7FFF) <= 12`
- When the next I-frame is processed
- Then the directional field is updated to `Some(current_ns)` and no finding is emitted
- Test vectors: prev=5000, current=5001 (gap=1) → no finding

### AC-171-005: Subsequent frame with gap > 12 emits T1692.001 Possible
**Traces to:** BC-2.19.024 postcondition (Path C) and invariant 1
- Given `last_ns_dir` is `Some(prev)` and `(current_ns.wrapping_sub(prev) & 0x7FFF) > 12`
- When the next I-frame is processed
- Then T1692.001 "Unauthorized Message: Command Message" finding is emitted with confidence Possible
- The finding message includes: current N(S), previous N(S) (prev), and the gap value
- The directional field is updated to `Some(current_ns)`
- Test vectors: prev=5001, current=5020 (gap=19) → T1692.001 Possible

### AC-171-006: 15-bit modular arithmetic applied correctly — wrapping_sub & 0x7FFF
**Traces to:** BC-2.19.024 invariant 1 (15-bit modular arithmetic)
- Given `last_ns_dir = Some(32767)` and `current_ns = 1`
- When gap is computed: `1u16.wrapping_sub(32767) & 0x7FFF`
  = `(1u16.wrapping_sub(32767)) & 0x7FFF`
  = `32770u16 & 0x7FFF` = `2`
- Then gap = 2 ≤ 12 → no finding; state → `Some(1)`
- This MUST use `wrapping_sub` with `& 0x7FFF` mask; plain subtraction would overflow

### AC-171-007: Directional isolation — C2S and S2C tracked independently
**Traces to:** BC-2.19.023 postcondition 3 (direction parameter selects field)
- Given different N(S) sequences in C2S and S2C directions
- When I-frames arrive alternating directions
- Then `last_ns_c2s` and `last_ns_s2c` are updated independently; no cross-direction mixing
- VP-045 proptest verifies this directional isolation property

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `extract_ns` | SS-19 sequence extraction | `src/analyzer/iec104.rs` | Pure (free fn) |
| `extract_nr` | SS-19 sequence extraction | `src/analyzer/iec104.rs` | Pure (free fn) |
| `last_ns_c2s: Option<u16>` | SS-19 per-flow state | `src/analyzer/iec104.rs` | Mutable state |
| `last_ns_s2c: Option<u16>` | SS-19 per-flow state | `src/analyzer/iec104.rs` | Mutable state |
| N(S) gap check | SS-19 gap detector | `src/analyzer/iec104.rs` | Effectful (emits finding) |

Subsystem anchor: SS-19 owns this story's scope because N(S) sequence tracking and
desynchronization detection are core IEC-104 behavioral capabilities per ARCH-INDEX.md §SS-19.

## Tasks

- [ ] Implement `extract_ns(cf1: u8, cf2: u8) -> u16` as pure free fn (BC-2.19.023)
- [ ] Implement `extract_nr(cf3: u8, cf4: u8) -> u16` as pure free fn (BC-2.19.023)
- [ ] Add `last_ns_c2s: Option<u16>` and `last_ns_s2c: Option<u16>` to `Iec104FlowState`
  (initialized to `None`; replaces any prior u16 default-0 approach)
- [ ] Implement gap check logic inside I-format frame processing (effectful, inside `on_data`):
  ```rust
  match last_ns_dir {
      None => { *last_ns_dir = Some(current_ns); /* baseline — no finding */ }
      Some(prev) => {
          let gap = current_ns.wrapping_sub(prev) & 0x7FFF;
          if gap > 12 {
              emit_finding(T1692_001, Possible, /* current_ns, prev, gap */);
          }
          *last_ns_dir = Some(current_ns);
      }
  }
  ```
- [ ] Write unit tests: one per AC, named `test_BC_2_19_023_*` and `test_BC_2_19_024_*`
- [ ] Include test for RETRANSMIT-NS-FALSEPOS-001 edge case (see Edge Cases below)
- [ ] Verify `cargo test` passes

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | BC-2.19.024 | First I-frame, N(S)=0 (fresh flow) | `None → Some(0)`; no finding |
| EC-002 | BC-2.19.024 | First I-frame, N(S)=5000 (mid-capture start) | `None → Some(5000)`; no finding |
| EC-003 | BC-2.19.024 | Gap = 12 exactly | No finding (≤ k) |
| EC-004 | BC-2.19.024 | Gap = 13 (k+1) | T1692.001 Possible |
| EC-005 | BC-2.19.024 | Wrap: Some(32767) → current=1 | Gap=2 via `wrapping_sub & 0x7FFF`; no finding |
| EC-006 | BC-2.19.024 | Massive gap (32767) | T1692.001 Possible |
| EC-007 | RETRANSMIT-NS-FALSEPOS-001 | TCP retransmission delivers older N(S) | Gap computed from older N(S) may exceed k=12 even when benign; this is a known false-positive risk for TCP retransmissions that re-deliver I-frames with lower N(S) values. The analyzer cannot distinguish TCP retransmits from adversarial replays. Document in test comments; do not suppress the finding (fail-closed per INV-3) |

**RETRANSMIT-NS-FALSEPOS-001 note:** TCP retransmissions that re-deliver I-frames with a lower
N(S) than the last seen value will produce a false-positive T1692.001 Possible. This is the
expected behavior (fail-closed per INV-3). The risk is documented here for operator awareness;
future mitigation via TCP deduplication is deferred.

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~3,000 |
| BC-2.19.023–024 (2 BCs × ~700 each) | ~1,400 |
| ss-19-iec104-analysis.md (SS-19 shard) | ~8,000 |
| ADR-013 architecture decisions | ~12,000 |
| src/analyzer/iec104.rs (from STORY-168) | ~7,000 |
| Test file delta | ~2,000 |
| TOTAL | ~33,400 |

Agent context window ~200k tokens. This story uses ~17% — within budget.

## Previous Story Intelligence

**Predecessor:** STORY-168 (frame format discrimination + session state machine)
- STORY-168 added `FrameFormat` enum and `session_started: bool` to `Iec104FlowState`
- This story adds `last_ns_c2s: Option<u16>` and `last_ns_s2c: Option<u16>` to `Iec104FlowState`
- The `Option<u16>` type is critical: do NOT use `u16` with default 0 (causes false positives
  on mid-capture starts where first N(S) may be non-zero — this was a pre-correction bug)
- `extract_ns` and `extract_nr` are pure functions called BEFORE state mutation

## Architecture Compliance Rules

Extracted from `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`:
- **ADR-013 Decision 6**: N(S) is tracked as `Option<u16>` (not bare `u16`). The `None`
  sentinel handles mid-capture starts where first N(S) is arbitrary. First I-frame unconditionally
  sets `Some(ns)` with NO finding.
- **ADR-013 Decision 6**: k=12 window is fixed for MVP. Future: configurable via `--iec104-k-window`.
- **ADR-013 Decision 6**: 15-bit modular gap MUST use `wrapping_sub(prev) & 0x7FFF`. Plain
  subtraction would wrap at u16::MAX (65535), not 32767, giving wrong results.
- N(R) is computed but NOT stored — `Iec104FlowState` has no `last_nr_*` field.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| Rust stdlib | 1.91+ | `u16::wrapping_sub`, `Option<u16>` |
| proptest | latest | VP-045 carry direction isolation (STORY-172 anchors proptest; N(S) direction isolation verified here) |

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/analyzer/iec104.rs` | MODIFY | Add `extract_ns`, `extract_nr` free fns; add `last_ns_c2s: Option<u16>`, `last_ns_s2c: Option<u16>` to `Iec104FlowState`; add gap check logic in I-frame processing path |
| `tests/iec104_analyzer_tests.rs` | MODIFY | Add BC-2.19.023–024 unit tests including RETRANSMIT-NS-FALSEPOS-001 documentation test |

## Forbidden Dependencies

- `iec60870-5`, `wireshark`, `lib60870` — banned (ADR-013 Decision 7)
- `extract_ns` / `extract_nr` MUST NOT access `Iec104FlowState` — they are pure extraction fns
- Gap check MUST use `wrapping_sub & 0x7FFF`; plain `>=` or `-` arithmetic is WRONG for 15-bit
