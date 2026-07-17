---
document_type: story
story_id: STORY-168
title: "IEC-104 Frame Format Discrimination + U-Format Session State Machine (STARTDT/STOPDT/TESTFR)"
epic_id: E-22
wave: 77
points: 5
phase: f3
tdd_mode: strict
status: delivered
feature_id: feature-iec104
subsystems: [SS-19]
target_module: analyzer/iec104
depends_on: [STORY-167]
blocks: [STORY-169, STORY-171]
behavioral_contracts:
  - BC-2.19.007
  - BC-2.19.008
  - BC-2.19.009
  - BC-2.19.010
  - BC-2.19.011
  - BC-2.19.012
  - BC-2.19.013
  - BC-2.19.014
verification_properties:
  - VP-046
  - VP-047
inputs:
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.007.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.008.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.009.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.010.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.011.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.012.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.013.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.014.md
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "de47ef5"
---

# STORY-168: IEC-104 Frame Format Discrimination + U-Format Session State Machine

## Narrative

**As a** security analyst using wirerust to inspect ICS/SCADA traffic,
**I want** the IEC-104 analyzer to correctly classify I/S/U frame formats and track the
STARTDT/STOPDT/TESTFR session state machine,
**so that** service-stop anomalies (T0881), non-canonical U-frame attacks (T0814/CVE-2026-1773),
and session lifecycle events are detected with correct confidence levels.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.19.007 | I-Format Frame: CF1 Bit 0 = 0 | Frame discrimination — I-format path |
| BC-2.19.008 | S-Format Frame: CF1 Bits1:0 = 0b01 | Frame discrimination — S-format path |
| BC-2.19.009 | U-Format Frame: CF1 Bits1:0 = 0b11; classify_frame_format Totality (VP-046) | Frame discrimination — U-format + VP-046 totality |
| BC-2.19.010 | STARTDT-act (CF1=0x07) Sets session_started=true | Session SM — STARTDT |
| BC-2.19.011 | STOPDT-act While session_started=true: T0881 Possible | Session SM — STOPDT detected (possible) |
| BC-2.19.012 | STOPDT-act While session_started=false: T0881 Likely | Session SM — STOPDT detected (likely) |
| BC-2.19.013 | TESTFR-act/con: No Finding; TESTFR Is Keepalive | Session SM — TESTFR no-op |
| BC-2.19.014 | Non-Canonical U-Frame CF1 → T0814 (CVE-2026-1773) | Session SM — non-canonical U-frame anomaly |

## Acceptance Criteria

### AC-168-001: `classify_frame_format` correctly classifies I-format frames
**Traces to:** BC-2.19.007 postconditions 1–2
- Given an `ApciHeader` where `cf1 & 0x01 == 0x00`
- When `classify_frame_format(cf1)` is called
- Then returns `FrameFormat::IFormat`
- I-format frames carry an ASDU payload and include N(S)/N(R) sequence numbers in CF1–CF4

### AC-168-002: `classify_frame_format` correctly classifies S-format frames
**Traces to:** BC-2.19.008 postconditions 1–2
- Given an `ApciHeader` where `cf1 & 0x03 == 0x01` (bit0=1, bit1=0)
- When `classify_frame_format(cf1)` is called
- Then returns `FrameFormat::SFormat`
- S-format frames are supervisory-only (N(R) only, no ASDU payload)

### AC-168-003: `classify_frame_format` correctly classifies U-format frames and is total over all 256 u8 values
**Traces to:** BC-2.19.009 postconditions 1–2 and invariant 1 (VP-046 totality)
- Given any `u8` CF1 value where `cf1 & 0x03 == 0x03`
- When `classify_frame_format(cf1)` is called
- Then returns `FrameFormat::UFormat`
- The `classify_frame_format` function is total over all 256 possible CF1 u8 values: every u8 maps
  to exactly one of IFormat, SFormat, or UFormat (no unhandled case, no panic); VP-046 proptest
  verifies this exhaustively

### AC-168-004: STARTDT-act sets session_started=true with no finding
**Traces to:** BC-2.19.010 postconditions 1–4 and invariant 1
- Given a U-format frame with CF1=0x07 (STARTDT-act)
- When the session state machine processes the frame
- Then `Iec104FlowState::session_started` is set to `true`
- No finding is emitted (STARTDT is expected operational behavior)
- Receiving STARTDT-act when already started is idempotent (state remains true, no finding)
- STARTDT-con (CF1=0x0B) is also recognized; sets session_started=true if not already set

### AC-168-005: STOPDT-act while session active emits T0881 with confidence Possible
**Traces to:** BC-2.19.011 postconditions 1–3
- Given `Iec104FlowState::session_started == true`
- When a U-format frame with CF1=0x13 (STOPDT-act) is received
- Then a T0881 "Service Stop" finding is emitted with confidence `Possible`
- `session_started` is set to `false` after emission

### AC-168-006: STOPDT-act without prior STARTDT emits T0881 with confidence Likely
**Traces to:** BC-2.19.012 postconditions 1–3
- Given `Iec104FlowState::session_started == false` (no STARTDT observed)
- When a U-format frame with CF1=0x13 (STOPDT-act) is received
- Then a T0881 "Service Stop" finding is emitted with confidence `Likely`
- A STOPDT without a prior STARTDT is anomalous; `Likely` confidence (stronger than Possible)

### AC-168-007: TESTFR-act and TESTFR-con produce no finding
**Traces to:** BC-2.19.013 postconditions 1–2 and invariant 1
- Given a U-format frame with CF1=0x43 (TESTFR-act) or CF1=0x83 (TESTFR-con)
- When the session state machine processes the frame
- Then no finding is emitted; session state is unchanged
- TESTFR is a keepalive mechanism; observation is normal IEC-104 behavior

### AC-168-008: Non-canonical U-frame CF1 emits T0814
**Traces to:** BC-2.19.014 postconditions 1–2 and invariant 1 (CVE-2026-1773)
- Given a U-format frame (CF1 bits1:0 = 0b11) where CF1 does not match any of
  STARTDT-act (0x07), STARTDT-con (0x0B), STOPDT-act (0x13), STOPDT-con (0x23),
  TESTFR-act (0x43), TESTFR-con (0x83)
- When the session state machine processes the frame
- Then a T0814 "Denial of Service" finding is emitted with confidence Possible
- This matches the CVE-2026-1773 non-canonical U-frame attack vector

### AC-168-009: VP-046 proptest skeleton compiles — classify_frame_format totality
**Traces to:** BC-2.19.009 invariant 1 (VP-046 totality obligation)
- Given the `classify_frame_format(cf1: u8) -> FrameFormat` pure-core free function
- When the VP-046 proptest harness is scaffolded in this story
- Then the proptest skeleton (`proptest_vp046_frame_format_totality`) compiles
- Full proptest run over all 256 u8 values is executed in STORY-174

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `classify_frame_format` | SS-19 frame classifier | `src/analyzer/iec104.rs` | Pure (free fn, VP-046 target) |
| `FrameFormat` enum | SS-19 data model | `src/analyzer/iec104.rs` | N/A (data type) |
| `process_u_frame` | SS-19 session SM | `src/analyzer/iec104.rs` | Effectful (emits findings, mutates state) |
| `Iec104FlowState::session_started` | SS-19 per-flow state | `src/analyzer/iec104.rs` | Mutable state field |
| VP-046 proptest | proptest harness | `tests/iec104_analyzer_tests.rs` | Test-only |

Subsystem anchor: SS-19 owns this story's scope because frame discrimination and the
STARTDT/STOPDT/TESTFR session state machine are core behavioral capabilities of the IEC-104
passive analyzer per ARCH-INDEX.md §SS-19.

## VP-046 Proptest Obligation

**Harness:** `proptest_vp046_frame_format_totality` (anchored in STORY-168)
**Method:** proptest exhaustive sweep
**Priority:** P1

Skeleton (in `tests/iec104_analyzer_tests.rs`):

```rust
proptest! {
    #[test]
    fn proptest_vp046_frame_format_totality(cf1 in 0u8..=255u8) {
        // Every u8 maps to exactly one FrameFormat — no unhandled case
        let fmt = classify_frame_format(cf1);
        // Partitioning assertion (property verified exhaustively over all 256 values):
        match cf1 & 0x03 {
            0x00 => prop_assert!(matches!(fmt, FrameFormat::IFormat)),
            0x01 => prop_assert!(matches!(fmt, FrameFormat::SFormat)),
            0x03 => prop_assert!(matches!(fmt, FrameFormat::UFormat)),
            _ => {} // 0x02 cannot occur (only bits1:0 matter for 2-bit classification)
        }
    }
}
```

Full proof execution is in STORY-174. Mirrors VP-032 Sub-B pattern (ENIP classify_enip_command).

## Tasks

- [ ] Define `FrameFormat` enum: `IFormat`, `SFormat`, `UFormat`
- [ ] Implement `classify_frame_format(cf1: u8) -> FrameFormat` as pure-core free fn
  - `cf1 & 0x01 == 0x00` → IFormat (BC-2.19.007)
  - `cf1 & 0x03 == 0x01` → SFormat (BC-2.19.008)
  - `cf1 & 0x03 == 0x03` → UFormat (BC-2.19.009)
- [ ] Add `session_started: bool` field to `Iec104FlowState` (initialized to `false`)
- [ ] Implement `process_u_frame` (or inline in `on_data` dispatcher) for U-format handling:
  - CF1=0x07 → session_started=true, no finding (BC-2.19.010)
  - CF1=0x0B → session_started=true (STARTDT-con), no finding (BC-2.19.010)
  - CF1=0x13 → T0881 Possible (if started) / Likely (if not started); session_started=false (BC-2.19.011/012)
  - CF1=0x43, 0x83 → no finding (BC-2.19.013)
  - Other U-frame CF1 → T0814 Possible (BC-2.19.014)
- [ ] Write VP-046 proptest skeleton in `tests/iec104_analyzer_tests.rs`
- [ ] Write unit tests: one per AC; named `test_BC_2_19_007_*`, etc.
- [ ] Verify `cargo test` passes for all new tests

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.19.007 | CF1=0x00 (I-format, N(S)=0) | IFormat classification |
| EC-002 | BC-2.19.008 | CF1=0x01 (S-format, minimal) | SFormat classification |
| EC-003 | BC-2.19.009 | CF1=0x03 (U-format, minimal) | UFormat classification |
| EC-004 | BC-2.19.010 | STARTDT-act then STARTDT-act again | Idempotent; no finding |
| EC-005 | BC-2.19.012 | STOPDT without prior STARTDT | T0881 Likely (not just Possible) |
| EC-006 | BC-2.19.014 | CF1=0x0F (non-canonical U-frame) | T0814 Possible |
| EC-007 | BC-2.19.014 | CF1=0xFF (non-canonical U-frame) | T0814 Possible |
| EC-008 | BC-2.19.013 | TESTFR-con (0x83) after TESTFR-act (0x43) | No finding for either |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~4,000 |
| BC-2.19.007–014 (8 BCs × ~600 each) | ~4,800 |
| ss-19-iec104-analysis.md (SS-19 shard) | ~8,000 |
| ADR-013 architecture decisions | ~12,000 |
| src/analyzer/iec104.rs (from STORY-167) | ~3,000 |
| Test file (new unit + proptest) | ~2,500 |
| TOTAL | ~34,300 |

Agent context window ~200k tokens. This story uses ~17% — within budget.

## Previous Story Intelligence

**Predecessor:** STORY-167 (APCI core parser)
- STORY-167 defines `ApciHeader`, `parse_apci_header`, `is_valid_iec104_frame`
- This story extends `src/analyzer/iec104.rs` with `FrameFormat` enum and `classify_frame_format`
- `Iec104FlowState` stub from STORY-167 gains a `session_started: bool` field here
- `T0881` technique ID must be referenced in `emit_finding` calls; the actual mitre.rs
  catalog registration is in STORY-173 (integration). This story uses the string identifier
  `"T0881"` — the catalog entry will be added atomically in STORY-173.

## Architecture Compliance Rules

Extracted from `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`:
- **ADR-013 Decision 4**: `classify_frame_format(cf1: u8) -> FrameFormat` is a pure-core
  free function. It must NOT mutate state or emit findings — that is the caller's role.
- **ADR-013 Decision 5**: U-format canonical values: STARTDT-act=0x07, STARTDT-con=0x0B,
  STOPDT-act=0x13, STOPDT-con=0x23, TESTFR-act=0x43, TESTFR-con=0x83. All other U-frame
  CF1 values are non-canonical → T0814.
- **ADR-013 Decision 9**: T0881 is an EMITTED technique. The mitre.rs catalog entry is
  added in STORY-173 (six-part atomic commit per BC-2.10.010). This story emits T0881 via
  `emit_finding(T0881, ...)` — the constant must be defined or stubbed before STORY-173 lands.
- Pure/effectful boundary: `classify_frame_format` is pure; `process_u_frame` is effectful.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | Language; match patterns, enum dispatch |
| proptest | latest (from Cargo.toml) | VP-046 frame format totality proptest skeleton |
| cargo-fuzz | latest | VP-047 fuzz coverage (extended in STORY-172) |

No new external crate dependencies beyond what STORY-167 introduced.

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/analyzer/iec104.rs` | MODIFY | Add `FrameFormat` enum, `classify_frame_format`, `process_u_frame`, `session_started` field on `Iec104FlowState` |
| `tests/iec104_analyzer_tests.rs` | MODIFY | Add BC-2.19.007–014 unit tests + VP-046 proptest skeleton |

## Forbidden Dependencies

- `iec60870-5`, `wireshark`, `lib60870` — banned (ADR-013 Decision 7)
- `session_started` field must NOT influence `classify_frame_format` — that function is pure
  and must not read flow state
