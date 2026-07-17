---
document_type: story
level: ops
story_id: STORY-169
title: "IEC-104 ASDU Header Extraction: parse_asdu / Asdu with Broken-Out DUI Fields"
epic_id: E-22
version: "1.1"
status: delivered
producer: story-writer
timestamp: 2026-07-14T23:00:00Z
phase: f4
inputs:
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.015.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.016.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.017.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.018.md
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "1f1b917"
traces_to: .factory/specs/prd.md
points: 3
depends_on: [STORY-168]
blocks: [STORY-170]
behavioral_contracts: [BC-2.19.015, BC-2.19.016, BC-2.19.017, BC-2.19.018]
verification_properties: [VP-047]
priority: P1
cycle: feature-iec104
wave: 78
target_module: analyzer/iec104
subsystems: [SS-19]
estimated_days: null
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
feature_id: feature-iec104
---

> **v1.1 — F3-drift correction (2026-07-14):** Realigned to BC-2.19.015-018 (F2-converged).
> Function renamed `parse_asdu`; struct renamed `Asdu`; broken-out DUI fields
> (`sq`/`count`/`cot_cause`/`cot_pn`/`cot_test`/`cot_originator`/`casdu`/`first_ioa Option`);
> min-length guard corrected to `< 6` (was `< 10`); `first_ioa` is `Option<u32>` (was always-present `u32`).
> Pre-delivery reconciliation.

# STORY-169: IEC-104 ASDU Header Extraction: parse_asdu / Asdu with Broken-Out DUI Fields

## Narrative

**As a** security analyst using wirerust to inspect ICS/SCADA traffic,
**I want** the IEC-104 analyzer to correctly extract all ASDU DUI header fields from
I-format frames into a semantically broken-out `Asdu` struct,
**so that** TypeID-based threat detection (control commands, system resets, reconnaissance)
in STORY-170 has a verified, field-level extraction foundation with no packed-byte ambiguity.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.19.015 | ASDU Minimum-Length Guard Rejects I-Frame ASDU Body Shorter Than 6 Bytes | Guard — `asdu_body.len() < 6` → `None` |
| BC-2.19.016 | TypeID and VSQ Extraction from ASDU Bytes 0–1 | Core extraction — `type_id`, `sq`, `count` |
| BC-2.19.017 | COT Extraction (Cause of Transmission) from ASDU Bytes 2–3 | Core extraction — `cot_cause`, `cot_pn`, `cot_test`, `cot_originator` |
| BC-2.19.018 | CASDU and First IOA Extraction from ASDU Bytes 4–8 | Core extraction — `casdu`, `first_ioa: Option<u32>` |

## Acceptance Criteria

### AC-169-001 (traces to BC-2.19.015 postconditions 1–3)
`parse_asdu(asdu_body)` returns `None` when `asdu_body.len() < 6`; no ASDU field is accessed; no panic.
- Given an I-format APCI frame whose extracted ASDU body has fewer than 6 bytes
  (TypeID(1) + VSQ(1) + COT(2) + CASDU(2) = 6-byte DUI minimum)
- When `parse_asdu(asdu_body)` is called
- Then it returns `None`; no TypeID, VSQ, COT, CASDU, or IOA fields are accessed; no panic occurs
- The caller (STORY-170 effectful shell) emits T0814 on `None`

### AC-169-002 (traces to BC-2.19.016 postconditions 1–4)
TypeID and VSQ broken-out into `type_id`, `sq`, `count` from bytes 0–1.
- Given `asdu_body.len() >= 6`
- When `parse_asdu(asdu_body)` returns `Some(asdu)`
- Then:
  - `asdu.type_id == asdu_body[0]` (u8, any value 0–255; TypeID 0 is undefined, passed through for caller handling)
  - `asdu.sq == (asdu_body[1] & 0x80) != 0` (SQ bit: sequence qualifier — true = contiguous sequence)
  - `asdu.count == asdu_body[1] & 0x7F` (number of information objects, 0–127)
  - If `asdu.count == 0`, no Information Objects are present (valid, unusual)

### AC-169-003 (traces to BC-2.19.017 postconditions 1–4)
COT broken-out into `cot_cause`, `cot_pn`, `cot_test`, `cot_originator` from bytes 2–3.
- Given `asdu_body.len() >= 6`
- When `parse_asdu(asdu_body)` returns `Some(asdu)`
- Then:
  - `asdu.cot_cause == asdu_body[2] & 0x3F` (6-bit cause code, 0–63)
  - `asdu.cot_pn == (asdu_body[2] & 0x40) != 0` (P/N flag: positive/negative confirmation)
  - `asdu.cot_test == (asdu_body[2] & 0x80) != 0` (T flag: test transmission)
  - `asdu.cot_originator == asdu_body[3]` (u8; 0 = no originator)

### AC-169-004 (traces to BC-2.19.018 postcondition 1)
CASDU extracted as 16-bit LE from bytes 4–5.
- Given `asdu_body.len() >= 6`
- When `parse_asdu(asdu_body)` returns `Some(asdu)`
- Then `asdu.casdu == u16::from_le_bytes([asdu_body[4], asdu_body[5]])` (Common Address of ASDU, RTU/IED identity)

### AC-169-005 (traces to BC-2.19.018 postconditions 2–3)
`first_ioa` is `Some(24-bit LE zero-extended)` when `count > 0` and `len >= 9`; `None` otherwise.
- Given `asdu_body.len() >= 9` AND `asdu.count > 0`
- When `parse_asdu(asdu_body)` returns `Some(asdu)`
- Then `asdu.first_ioa == Some(u32::from_le_bytes([asdu_body[6], asdu_body[7], asdu_body[8], 0]))` (24-bit LE IOA, zero-extended to u32)
- Given `asdu.count == 0` OR `asdu_body.len() < 9`
- Then `asdu.first_ioa == None` (no IOA bytes available or no objects declared)

### AC-169-006 (traces to BC-2.19.015 invariant 2)
`parse_asdu` is a pure free function — no side effects, no finding emission, no state mutation.
- `parse_asdu` emits no findings, mutates no shared state, and performs no I/O
- The extraction layer returns an `Option<Asdu>` data value; the calling effectful layer (STORY-170) emits findings

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `parse_asdu` | SS-19 ASDU parser | `src/analyzer/iec104.rs` | Pure (free fn) |
| `Asdu` struct | SS-19 data model | `src/analyzer/iec104.rs` | N/A (data type) |

Subsystem anchor: SS-19 owns this story's scope because ASDU field extraction is the core
data-layer foundation for IEC-104 TypeID-based detection per ARCH-INDEX.md §SS-19.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `parse_asdu` (free fn) | pure-core | No I/O, no state mutation, no finding emission; returns `Option<Asdu>` by value |
| `Asdu` struct | pure-core | Plain data; no methods that perform I/O or mutation |
| `on_data` caller (STORY-170) | effectful-shell | Calls `parse_asdu`, emits T0814 finding on `None`, drives TypeID dispatch |

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.19.015 | ASDU body 5 bytes (one short of 6-byte DUI minimum) | `None`; caller emits T0814 |
| EC-002 | BC-2.19.015 | ASDU body exactly 6 bytes (minimum valid DUI, no IOA bytes) | `Some(Asdu{...})` with `first_ioa=None` |
| EC-003 | BC-2.19.015 | ASDU body 6–8 bytes, `count > 0` | `Some(Asdu{...})` with `first_ioa=None` (insufficient bytes for 3-byte IOA) |
| EC-004 | BC-2.19.018 | ASDU body 9+ bytes, `count > 0` | `first_ioa=Some(...)` — 24-bit LE zero-extended to u32 |
| EC-005 | BC-2.19.018 | `count == 0` regardless of body length | `first_ioa=None` (no objects declared) |
| EC-006 | BC-2.19.018 | IOA = 0xFFFFFF (max 24-bit) | Extracted correctly via 3-byte LE + zero-pad |
| EC-007 | BC-2.19.016 | TypeID=0 (undefined per spec) | Extracted as `type_id=0`; `parse_asdu` passes through; caller handles anomaly |
| EC-008 | BC-2.19.017 | T-bit set (`cot_test=true`) | Extracted; caller may suppress or tag findings `[TEST]` per BC-2.19.017 invariant 1 |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~2,800 |
| BC-2.19.015–018 (4 BC files × ~600 each) | ~2,400 |
| `ss-19-iec104-analysis.md` (SS-19 shard) | ~8,000 |
| ADR-013 architecture decisions | ~12,000 |
| `src/analyzer/iec104.rs` (from STORY-168) | ~5,000 |
| Test file delta (6 new tests) | ~1,800 |
| TOTAL | ~32,000 |

Agent context window ~200k tokens. This story uses ~16% — within budget.

## Tasks

- [ ] Define `Asdu` struct with broken-out DUI fields:
  ```rust
  pub struct Asdu {
      pub type_id: u8,           // asdu_body[0]
      pub sq: bool,              // (asdu_body[1] & 0x80) != 0
      pub count: u8,             // asdu_body[1] & 0x7F  (0–127)
      pub cot_cause: u8,         // asdu_body[2] & 0x3F  (0–63)
      pub cot_pn: bool,          // (asdu_body[2] & 0x40) != 0
      pub cot_test: bool,        // (asdu_body[2] & 0x80) != 0
      pub cot_originator: u8,    // asdu_body[3]
      pub casdu: u16,            // u16::from_le_bytes([asdu_body[4], asdu_body[5]])
      pub first_ioa: Option<u32>,// Some(24-bit LE) if count>0 && len>=9, else None
  }
  ```
- [ ] Implement `parse_asdu(asdu_body: &[u8]) -> Option<Asdu>` as a pure free function:
  - Return `None` if `asdu_body.len() < 6` (6-byte DUI minimum guard; caller emits T0814)
  - Extract all nine fields exactly per AC-169-002 through AC-169-005
  - `first_ioa` logic:
    ```rust
    let first_ioa = if count > 0 && asdu_body.len() >= 9 {
        Some(u32::from_le_bytes([asdu_body[6], asdu_body[7], asdu_body[8], 0]))
    } else {
        None
    };
    ```
  - Return `Some(Asdu { type_id, sq, count, cot_cause, cot_pn, cot_test, cot_originator, casdu, first_ioa })`
- [ ] Write unit tests: one per AC, named `test_BC_2_19_015_*`, `test_BC_2_19_016_*`, `test_BC_2_19_017_*`, `test_BC_2_19_018_*`
- [ ] Verify `cargo test` passes

## Previous Story Intelligence

**Predecessor:** STORY-168 (frame format discrimination + session state machine)
- STORY-168 defines `FrameFormat` enum and `classify_frame_format`
- STORY-168 adds `session_started: bool` to `Iec104FlowState`
- This story adds the `Asdu` struct and `parse_asdu` pure free function — called only for I-format frames
- The ASDU body in an I-format frame is `&apci_data[4..]` where `apci_data.len() == header.len as usize`
  (APCI header = 6 bytes: START + LEN + CF1–CF4; LEN covers CF1..CF4 + ASDU; ASDU body = LEN − 4 bytes starting at offset 4 within APCI data)

## Architecture Compliance Rules

Extracted from `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`:
- **ADR-013 Decision 8**: ASDU parsing (`parse_asdu`) is pure-core, outside VP-044 Kani scope.
  VP-047 (`fuzz_iec104_parser`) covers all `parse_asdu` paths via the top-level `on_data` harness.
- **ASDU payload offset**: ASDU body starts at byte 4 of `apci_data` (bytes 0–3 are CF1–CF4 control
  octets). Pass `&apci_data[4..]` to `parse_asdu`.
- **6-byte DUI minimum**: TypeID(1) + VSQ(1) + COT(2) + CASDU(2) = 6 bytes; the guard is `< 6`, NOT `< 10`.
- **Pure/effectful boundary**: `parse_asdu` is pure; finding emission (T0814 on `None`) lives in
  the STORY-170 effectful caller.
- **Broken-out fields only**: Do NOT define a packed `vsq: u8` or packed `cot: u16`. Each semantic
  sub-field must be its own `bool` or `u8` per BC-2.19.016/017 postconditions.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | `u16::from_le_bytes`, `u32::from_le_bytes`, slice indexing, `bool` bit-masking |

No new crate dependencies.

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/analyzer/iec104.rs` | MODIFY | Add `Asdu` struct (9 broken-out fields) and `parse_asdu` pure free fn |
| `tests/iec104_analyzer_tests.rs` | MODIFY | Add BC-2.19.015–018 unit tests (`test_BC_2_19_015_*`, etc.) |

## Forbidden Dependencies

- `iec60870-5`, `wireshark`, `lib60870` — banned (ADR-013 Decision 7)
- `parse_asdu` MUST NOT emit findings, mutate `Iec104FlowState`, or have any observable side effect
- Do NOT define a packed `vsq: u8` or `cot: u16` field on `Asdu` — only the broken-out semantic
  sub-fields (`sq`, `count`, `cot_cause`, `cot_pn`, `cot_test`, `cot_originator`) are permitted
