---
document_type: story
story_id: STORY-169
title: "IEC-104 ASDU Header Extraction: TypeID, VSQ, COT, CASDU, IOA"
epic_id: E-22
wave: 78
points: 3
phase: f3
tdd_mode: strict
status: draft
feature_id: feature-iec104
subsystems: [SS-19]
target_module: analyzer/iec104
depends_on: [STORY-168]
blocks: [STORY-170]
behavioral_contracts:
  - BC-2.19.015
  - BC-2.19.016
  - BC-2.19.017
  - BC-2.19.018
verification_properties:
  - VP-047
inputs:
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.015.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.016.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.017.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.018.md
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "5a970ca"
---

# STORY-169: IEC-104 ASDU Header Extraction: TypeID, VSQ, COT, CASDU, IOA

## Narrative

**As a** security analyst using wirerust to inspect ICS/SCADA traffic,
**I want** the IEC-104 analyzer to correctly extract ASDU header fields from I-format frames,
**so that** TypeID-based threat detection (control commands, system resets, reconnaissance)
in STORY-170 has a verified field-extraction foundation.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.19.015 | TypeID Extracted from ASDU Byte 0 (Range 1–255) | Core extraction — TypeID |
| BC-2.19.016 | VSQ (Variable Structure Qualifier) Extracted from ASDU Byte 1 | Core extraction — VSQ |
| BC-2.19.017 | COT (Cause of Transmission) Extracted from ASDU Bytes 2–3 | Core extraction — COT |
| BC-2.19.018 | CASDU and IOA Extracted from ASDU Bytes 4–9 | Core extraction — CASDU + IOA |

## Acceptance Criteria

### AC-169-001: TypeID extracted correctly from ASDU byte 0
**Traces to:** BC-2.19.015 postconditions 1–2
- Given an I-format APCI frame with a valid ASDU payload (at least 10 bytes after CF4)
- When `extract_asdu_header(asdu: &[u8])` is called
- Then `type_id = asdu[0]` — a u8 in range [1, 255]
- TypeID=0 is reserved and treated as anomalous; TypeID=0 is out of IEC-104 specification range

### AC-169-002: VSQ extracted correctly from ASDU byte 1
**Traces to:** BC-2.19.016 postconditions 1–2
- Given a valid ASDU slice `asdu.len() >= 2`
- When `extract_asdu_header` is called
- Then `vsq = asdu[1]` — Variable Structure Qualifier (number of information objects + SQ flag)
- Bit 7 of VSQ is the SQ (Sequence) flag; bits 6:0 are the number of information objects

### AC-169-003: COT extracted correctly from ASDU bytes 2–3
**Traces to:** BC-2.19.017 postconditions 1–2
- Given a valid ASDU slice `asdu.len() >= 4`
- When `extract_asdu_header` is called
- Then `cot = u16::from_le_bytes([asdu[2], asdu[3]])` — Cause of Transmission (2 bytes, LE)
- COT encodes why the ASDU was sent; relevant values: 3=spontaneous, 6=act, 7=actcon, 10=actterm

### AC-169-004: CASDU and IOA extracted correctly from ASDU bytes 4–9
**Traces to:** BC-2.19.018 postconditions 1–3
- Given a valid ASDU slice `asdu.len() >= 10`
- When `extract_asdu_header` is called
- Then `casdu = u16::from_le_bytes([asdu[4], asdu[5]])` — Common Address of ASDU (2 bytes, LE)
- Then `ioa = u32::from_le_bytes([asdu[6], asdu[7], asdu[8], 0])` — Information Object Address
  (3 bytes LE, padded to u32; byte [9] is the first element of the first information object body)
- IOA range: [0, 16777215] (24-bit)

### AC-169-005: `extract_asdu_header` returns None for ASDU too short
**Traces to:** BC-2.19.015 invariant 1 (minimum ASDU length guard)
- Given an ASDU slice with `asdu.len() < 10`
- When `extract_asdu_header(asdu)` is called
- Then returns `None`; no bytes beyond the slice length are accessed; no panic

### AC-169-006: ASDU extraction is pure — no side effects
**Traces to:** BC-2.19.018 invariant 2 (purity gate)
- `extract_asdu_header` is a pure-core free function: no I/O, no finding emission, no state mutation
- The ASDU extraction layer returns a data structure; the CALLING layer (STORY-170) emits findings

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `extract_asdu_header` | SS-19 ASDU parser | `src/analyzer/iec104.rs` | Pure (free fn) |
| `AsduHeader` struct | SS-19 data model | `src/analyzer/iec104.rs` | N/A (data type) |

Subsystem anchor: SS-19 owns this story's scope because ASDU field extraction is the core
data-layer foundation for IEC-104 TypeID-based detection per ARCH-INDEX.md §SS-19.

## Tasks

- [ ] Define `AsduHeader` struct: `type_id: u8, vsq: u8, cot: u16, casdu: u16, ioa: u32`
- [ ] Implement `extract_asdu_header(asdu: &[u8]) -> Option<AsduHeader>` as pure free fn
  - Return `None` if `asdu.len() < 10`
  - `type_id = asdu[0]`
  - `vsq = asdu[1]`
  - `cot = u16::from_le_bytes([asdu[2], asdu[3]])`
  - `casdu = u16::from_le_bytes([asdu[4], asdu[5]])`
  - `ioa = u32::from_le_bytes([asdu[6], asdu[7], asdu[8], 0])`
- [ ] Write unit tests: one per AC, named `test_BC_2_19_015_*`, etc.
- [ ] Verify `cargo test` passes

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.19.015 | TypeID=0 (reserved) | Extracted; caller (STORY-170) handles anomaly |
| EC-002 | BC-2.19.015 | TypeID=255 (max) | Extracted; caller handles unknown/reserved range |
| EC-003 | BC-2.19.018 | IOA=0xFFFFFF (max 24-bit) | Extracted correctly via 3-byte LE + zero pad |
| EC-004 | BC-2.19.015 | ASDU exactly 10 bytes | All fields extracted; no out-of-bounds |
| EC-005 | BC-2.19.015 | ASDU 9 bytes (too short) | Returns None |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~2,500 |
| BC-2.19.015–018 (4 BCs × ~500 each) | ~2,000 |
| ss-19-iec104-analysis.md (SS-19 shard) | ~8,000 |
| ADR-013 architecture decisions | ~12,000 |
| src/analyzer/iec104.rs (from STORY-168) | ~5,000 |
| Test file delta (4 new tests) | ~1,500 |
| TOTAL | ~31,000 |

Agent context window ~200k tokens. This story uses ~15% — within budget.

## Previous Story Intelligence

**Predecessor:** STORY-168 (frame format discrimination + session state machine)
- STORY-168 defines `FrameFormat` enum and `classify_frame_format`
- STORY-168 adds `session_started: bool` to `Iec104FlowState`
- This story adds `AsduHeader` struct and `extract_asdu_header` — called only for I-format frames
- The ASDU payload in an I-format frame starts after the 6-byte APCI header; the slice to pass is
  `&data[6..6+len as usize]` where `len` comes from `ApciHeader::len`

## Architecture Compliance Rules

Extracted from `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`:
- **ADR-013 Decision 8**: ASDU parsing (`extract_asdu_header`) is pure-core, not the VP-044 Kani
  target. VP-047 (fuzz) covers ASDU extraction paths via the top-level `on_data` harness.
- **ASDU payload offset**: ASDU starts at byte 6 of the raw APCI frame (bytes 0–5 are APCI header).
  The ASDU length is `ApciHeader::len - 4` bytes (LEN includes CF1–CF4, so subtract 4).
- Pure/effectful boundary: `extract_asdu_header` is pure; finding emission is in STORY-170.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | `u16::from_le_bytes`, `u32::from_le_bytes`, slice indexing |

No new crate dependencies.

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/analyzer/iec104.rs` | MODIFY | Add `AsduHeader` struct, `extract_asdu_header` free fn |
| `tests/iec104_analyzer_tests.rs` | MODIFY | Add BC-2.19.015–018 unit tests |

## Forbidden Dependencies

- `iec60870-5`, `wireshark`, `lib60870` — banned (ADR-013 Decision 7)
- `extract_asdu_header` must NOT emit findings or mutate `Iec104FlowState`
