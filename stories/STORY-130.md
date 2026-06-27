---
document_type: story
story_id: STORY-130
title: "EtherNet/IP Pure-Core Parse: ENIP Header, Command Classification, Frame Validity, and Kani VP-032"
epic_id: E-20
wave: 58
points: 8
phase: f3
tdd_mode: strict
status: ready
feature_id: issue-316-enip-analyzer
github_issue: 316
subsystems: [SS-17]
target_module: analyzer/enip
depends_on: []
behavioral_contracts:
  - BC-2.17.001
  - BC-2.17.002
  - BC-2.17.003
  - BC-2.17.004
verification_properties:
  - VP-032
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.001.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.002.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.003.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.004.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/specs/verification-properties/vp-032-enip-parse-safety.md
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
input-hash: "63fac3a"
---

# STORY-130: EtherNet/IP Pure-Core Parse: ENIP Header, Command Classification, Frame Validity, and Kani VP-032

## Narrative

**As a** security analyst using wirerust to inspect industrial network traffic,
**I want** the EtherNet/IP parser to correctly parse the 24-byte ENIP encapsulation header,
classify command codes, and validate frame boundaries,
**so that** all downstream CIP analysis and threat detection has a verified, bounds-safe
foundation proven correct by Kani formal verification (VP-032).

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.17.001 | `parse_enip_header` Returns None for Input Shorter Than 24 Bytes | Core implementation target (reject path) |
| BC-2.17.002 | EnipHeader Field Contracts — Fixed Little-Endian Offsets for 24-Byte Input | Core implementation target (accept path) |
| BC-2.17.003 | `is_valid_enip_frame` Validity Gate Biconditional — Known-Command Set | Core implementation target |
| BC-2.17.004 | `classify_enip_command` Total Classification with Unknown Arm Over All u16 Values | Core implementation target |

## Acceptance Criteria

### AC-130-001: `parse_enip_header` parses 24-byte header with little-endian fields (accept path)
**Traces to:** BC-2.17.002 postconditions 1–9
- Given a byte slice with `len >= 24`
- When `parse_enip_header(&bytes)` is called
- Then `Some(EnipHeader { command, length, session_handle, status, sender_context, options })` is returned
- With `command = u16::from_le_bytes([bytes[0], bytes[1]])` (BC-2.17.002 postcondition 2)
- With `length = u16::from_le_bytes([bytes[2], bytes[3]])` (BC-2.17.002 postcondition 3)
- With `session_handle = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])` (BC-2.17.002 postcondition 4)
- With `status = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]])` (BC-2.17.002 postcondition 5)
- With `sender_context = [bytes[12], bytes[13], bytes[14], bytes[15], bytes[16], bytes[17], bytes[18], bytes[19]]` as `[u8;8]` verbatim copy (BC-2.17.002 postcondition 6)
- With `options = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]])` (BC-2.17.002 postcondition 7)
- Bytes beyond index 23 are not read by this function (BC-2.17.002 postcondition 8)
- For any slice shorter than 24 bytes, returns `None` (BC-2.17.001 postcondition 1)
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_parse_enip_header_valid`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_parse_enip_header_too_short`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_parse_enip_header_exactly_24`

### AC-130-002: `parse_enip_header` returns None for any input shorter than 24 bytes (reject path)
**Traces to:** BC-2.17.001 postconditions 1–3; BC-2.17.002 postcondition 9 (purity/no panic)
- Given any `&[u8]` with `len < 24` (including empty slice, 1-byte, 23-byte)
- When `parse_enip_header(&bytes)` is called
- Then returns `None` without accessing any bytes — no panics, no UB (BC-2.17.001 postcondition 1–2)
- The 24-byte minimum is fixed by the ODVA EtherNet/IP specification and is not configurable (BC-2.17.001 invariant 1)
- For `len == 0`: returns `None` — zero bytes, no access (BC-2.17.001 EC-001)
- For `len == 23`: returns `None` — 23 bytes, none accessed (BC-2.17.001 EC-003)
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_parse_enip_header_no_panic_empty`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_parse_enip_header_no_panic_23_bytes`

### AC-130-003: `classify_enip_command` correctly maps all in-scope command codes and Unknown arm
**Traces to:** BC-2.17.004 postconditions 1–5
- `0x0004` → `EnipCommandClass::ListServices`
- `0x0063` → `EnipCommandClass::ListIdentity`
- `0x0064` → `EnipCommandClass::ListInterfaces`
- `0x0065` → `EnipCommandClass::RegisterSession`
- `0x0066` → `EnipCommandClass::UnRegisterSession`
- `0x006F` → `EnipCommandClass::SendRRData`
- `0x0070` → `EnipCommandClass::SendUnitData`
- `0x0072` → `EnipCommandClass::IndicateStatus`
- `0x0075` → `EnipCommandClass::Cancel`
- Any other value (e.g., `0x0000`, `0xFFFF`) → `EnipCommandClass::Unknown`
- The `Unknown` arm is reachable and non-vacuous (BC-2.17.004 postcondition 4)
- `EnipCommandClass` has exactly 10 variants (9 named + Unknown); the match is exhaustive (BC-2.17.004 invariant 1)
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_classify_enip_command_known`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_classify_enip_command_unknown`

### AC-130-004: `is_valid_enip_frame` returns true iff command is in the known-command set (biconditional)
**Traces to:** BC-2.17.003 postconditions 1–4
- Given an `EnipHeader` with `h.command` as any `u16` value
- When `is_valid_enip_frame(h)` is called
- Then returns `true` if and only if `h.command ∈ {0x0004, 0x0063, 0x0064, 0x0065, 0x0066, 0x006F, 0x0070, 0x0072, 0x0075}` — the 9-value ODVA known-command set (BC-2.17.003 postcondition 1)
- Returns `false` for any command value not in the known set (e.g., `0x0000`, `0x0062`, `0x0067`, `0xFFFF`) (BC-2.17.003 postcondition 2)
- The function does not inspect `h.length`, `h.status`, or `h.options` — command-only gate (BC-2.17.003 postcondition 3)
- The biconditional holds for all 65,536 possible `u16` values (BC-2.17.003 invariant 1)
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_is_valid_enip_frame_known_commands_true`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_is_valid_enip_frame_unknown_command_false`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_is_valid_enip_frame_boundary_commands`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_is_valid_enip_frame_all_fields_zeroed`

### AC-130-005: `classify_enip_command` is total — Unknown arm is reachable and non-vacuous; counter accumulation
**Traces to:** BC-2.17.004 postconditions 3–5, invariants 1–3
- The function never panics for any `u16` input (BC-2.17.004 postcondition 3); this property is formally verified by Kani VP-032 Sub-B
- `EnipCommandClass::Unknown` is reachable: `classify_enip_command(0x0000)` → `EnipCommandClass::Unknown` (BC-2.17.004 postcondition 4)
- `EnipCommandClass::Unknown` is also reachable at `0xFFFF` (BC-2.17.004 edge case EC-004)
- Gap values (e.g., `0x0067` — between UnRegisterSession and SendRRData) map to `Unknown` (BC-2.17.004 edge case EC-005)
- The `EnipCommandClass` enum has exactly 10 variants; the match is compiler-enforced exhaustive (BC-2.17.004 invariant 1)
- The caller MUST increment `flow.command_counts.entry(header.command).or_insert(0) += 1` after each
  classification — both named and Unknown commands are counted (BC-2.17.004 invariant 3). Per BC-2.17.016
  PC-0 (v1.1), this increment is placed in the frame-walk (STORY-137 / `on_data`) immediately after
  `parse_enip_header` returns `Some` and BEFORE `is_valid_enip_frame` is evaluated. This is the
  **single canonical increment site** — not in `process_pdu`. The placement guarantees the `Unknown`
  bucket is countable even for frames that are subsequently rejected by the validity gate.
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_classify_enip_command_unknown_zero`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_classify_enip_command_unknown_ffff`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_classify_enip_command_unknown_gap`

### AC-130-006: VP-032 Kani proof harnesses pass for Sub-A, Sub-B, and Sub-C
**Traces to:** VP-032 Sub-A, Sub-B, Sub-C
- Sub-A (`vp032_enip_header_parse_safety`): `parse_enip_header` never panics for any symbolic
  `&[u8]` up to 48 bytes; `unwind(49)` annotation required; asserts `result.is_none()` when
  `len < 24`, and asserts field-offset equality (`h.command`, `h.length`, `h.status` decoded
  little-endian from fixed offsets) when `len >= 24` — proving BC-2.17.002 field layout, not
  merely no-panic (VP-032 Sub-A skeleton; BC-2.17.001/002 purity invariants)
- Sub-B (`vp032_enip_command_classification_biconditional`): `classify_enip_command` biconditional
  — `Unknown` iff `cmd` not in `KNOWN_COMMANDS`, for all 65,536 `u16` values; simultaneously
  proves totality, Unknown reachability, and named-variant reachability (BC-2.17.004 invariant 1;
  DF-KANI-NONVACUITY-001)
- Sub-C (`vp032_enip_validity_gate_biconditional`): `is_valid_enip_frame(h: &EnipHeader) -> bool`
  biconditional for all 65,536 `u16` command values; asserts `gate_result == known_cmds.contains(&cmd)`
  (BC-2.17.003 postcondition 3, invariant 1)
- Sub-D (`vp032_cip_service_classification_totality`): OUT OF SCOPE for this story —
  `classify_cip_service` is defined in STORY-132 (BC-2.17.007)
- All harnesses live in `src/analyzer/enip.rs` under `#[cfg(kani)] mod kani_proofs { ... }`
- **Test:** `kani_proofs::vp032_enip_header_parse_safety` (Sub-A)
- **Test:** `kani_proofs::vp032_enip_command_classification_biconditional` (Sub-B)
- **Test:** `kani_proofs::vp032_enip_validity_gate_biconditional` (Sub-C)

## Architecture Mapping

| Component | Location | Role |
|-----------|----------|------|
| `parse_enip_header` | `src/analyzer/enip.rs` | Pure-core free fn, VP-032 Sub-A target |
| `classify_enip_command` | `src/analyzer/enip.rs` | Pure-core free fn, VP-032 Sub-B target |
| `is_valid_enip_frame` | `src/analyzer/enip.rs` | Pure-core free fn, VP-032 Sub-C target |
| `EnipHeader` struct | `src/analyzer/enip.rs` | 24-byte header model: command, length, session_handle, status, sender_context, options |
| `EnipCommandClass` enum | `src/analyzer/enip.rs` | 9 named variants + Unknown = 10 variants total (VP-032 Sub-B) |
| `kani_proofs` mod | `src/analyzer/enip.rs` | `#[cfg(kani)]` Kani harnesses (Sub-A through Sub-C in this story; Sub-D in STORY-132) |
| Test mod | `tests/enip_analyzer_tests.rs` | `mod parse_header { ... }` namespace |

**Pure/effectful boundary (ADR-010 Decision 2):** All four functions are pure-core free `fn`s — no `&self`, no `&mut self`, no global state, no I/O. They are the Kani verification targets and must remain pure to maintain VP-032 proof validity.

## VP Kani Obligation (VP-032)

VP-032 specifies 4 Kani proof harnesses for the pure-core parse functions. All harnesses live in `#[cfg(kani)] mod kani_proofs` inside `src/analyzer/enip.rs`. Sub-A through Sub-C are in scope for STORY-130 (which defines `parse_enip_header`, `classify_enip_command`, and `is_valid_enip_frame`). Sub-D (`vp032_cip_service_classification_totality`) is in scope for STORY-132 (which defines `classify_cip_service`).

**Sub-A — `vp032_enip_header_parse_safety` (VP-032 canonical name):**

Proves BC-2.17.001 (None for < 24 bytes) AND BC-2.17.002 (field layout) — not merely no-panic.
```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// VP-032 Sub-A: parse_enip_header never panics; returns None for <24 bytes;
    /// returns Some with correct field layout for >=24 bytes.
    ///
    /// BOUND/SOUNDNESS: 48-byte bound (2x minimum header) covers all length
    /// conditions; behavior is identical for any longer slice (fixed 24-byte read).
    /// Non-vacuity: both Some and None branches are reachable in the symbolic range.
    #[kani::proof]
    #[kani::unwind(49)]
    fn vp032_enip_header_parse_safety() {
        const BOUND: usize = 48;
        let data: [u8; BOUND] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= BOUND);
        let slice = &data[..len];
        let result = parse_enip_header(slice);
        if len < 24 {
            // BC-2.17.001 postcondition 1: must return None for any len < 24
            assert!(result.is_none());
        } else {
            // BC-2.17.002 postconditions 2/3/5: field offsets at fixed LE positions
            let h = result.expect("must be Some for len >= 24");
            let expected_cmd = u16::from_le_bytes([slice[0], slice[1]]);
            assert_eq!(h.command, expected_cmd);
            let expected_len = u16::from_le_bytes([slice[2], slice[3]]);
            assert_eq!(h.length, expected_len);
            let expected_status = u32::from_le_bytes([slice[8], slice[9], slice[10], slice[11]]);
            assert_eq!(h.status, expected_status);
        }
    }
```
- `unwind(49)` covers all symbolic lengths 0..=48 (covering both `< 24` and `>= 24` cases).

**Sub-B — `vp032_enip_command_classification_biconditional` (VP-032 canonical name):**

Proves the biconditional: `Unknown` iff `cmd` not in `KNOWN_COMMANDS`, for all 65,536 u16 values.
```rust
    /// VP-032 Sub-B: classify_enip_command(cmd) == Unknown iff cmd is not in KNOWN_COMMANDS.
    /// Biconditional simultaneously proves totality, Unknown reachability, and named-variant
    /// reachability (DF-KANI-NONVACUITY-001). No kani::assume on cmd.
    #[kani::proof]
    fn vp032_enip_command_classification_biconditional() {
        const KNOWN_COMMANDS: &[u16] = &[
            0x0004, 0x0063, 0x0064, 0x0065, 0x0066, 0x006F, 0x0070, 0x0072, 0x0075,
        ];
        let cmd: u16 = kani::any();
        let is_unknown = matches!(classify_enip_command(cmd), EnipCommandClass::Unknown);
        let not_in_known = !KNOWN_COMMANDS.contains(&cmd);
        assert_eq!(is_unknown, not_in_known);
    }
```

**Sub-C — `vp032_enip_validity_gate_biconditional` (VP-032 canonical name):**

Proves the biconditional: `is_valid_enip_frame` returns `true` iff `h.command` is in the
known-command set. BC-2.17.003 Postcondition 3 + Invariant 1.
```rust
    /// VP-032 Sub-C: is_valid_enip_frame iff h.command is in the known-command set.
    /// Biconditional proven for all 65,536 u16 command values.
    #[kani::proof]
    fn vp032_enip_validity_gate_biconditional() {
        let cmd: u16 = kani::any();
        let h = EnipHeader { command: cmd, length: 0, session_handle: 0,
                              status: 0, sender_context: [0u8; 8], options: 0 };
        let known_cmds: &[u16] = &[
            0x0004, 0x0063, 0x0064, 0x0065, 0x0066, 0x006F, 0x0070, 0x0072, 0x0075,
        ];
        let is_known = known_cmds.contains(&cmd);
        let gate_result = is_valid_enip_frame(&h);
        assert_eq!(gate_result, is_known);
    }
```

**Sub-D — `vp032_cip_service_classification_totality` (OUT OF SCOPE for STORY-130 — belongs to STORY-132):**

The Sub-D harness covers `classify_cip_service` (BC-2.17.007) which is defined in STORY-132 alongside `CipServiceClass`. It will live in the same `#[cfg(kani)] mod kani_proofs` block, added by STORY-132.

```rust
} // end kani_proofs (Sub-A, Sub-B, Sub-C only in STORY-130)
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `parse_enip_header` with 0 bytes | Returns `None` |
| EC-002 | `parse_enip_header` with 23 bytes | Returns `None` |
| EC-003 | `parse_enip_header` with 24 bytes all-zero | Returns `Some(EnipHeader { all zeros })` |
| EC-004 | `classify_enip_command(0x0000)` | Returns `EnipCommandClass::Unknown` (BC-2.17.004 EC-003) |
| EC-005 | `classify_enip_command(0xFFFF)` | Returns `EnipCommandClass::Unknown` (BC-2.17.004 EC-004) |
| EC-006 | `is_valid_enip_frame` with `command=0x0063` (ListIdentity), all other fields zeroed | Returns `true` — command-only gate; other fields ignored (BC-2.17.003 EC-008) |
| EC-007 | `is_valid_enip_frame` with `command=0x0000` | Returns `false` — not in known-command set (BC-2.17.003 EC-003) |
| EC-008 | `is_valid_enip_frame` with `command=0x0076` (one above Cancel, not assigned) | Returns `false` — above highest known command (BC-2.17.003 EC-007) |
| EC-009 | `classify_enip_command(0x0067)` (gap between 0x0066 and 0x006F) | Returns `EnipCommandClass::Unknown` — not in ODVA table (BC-2.17.004 EC-005) |
| EC-010 | `is_valid_enip_frame` with `command = 0x0075` (Cancel) | Returns `true` — Cancel is the highest boundary of the known-command set (BC-2.17.003 EC-006) |

## Tasks

- [ ] Create `src/analyzer/enip.rs` with module-level doc comment citing ADR-010
- [ ] Define `EnipHeader` struct with fields: `command: u16`, `length: u16`, `session_handle: u32`, `status: u32`, `sender_context: [u8; 8]`, `options: u32`
- [ ] Implement `fn parse_enip_header(buf: &[u8]) -> Option<EnipHeader>` — slice bounds check `buf.len() >= 24`, then LE reads via `u16::from_le_bytes` / `u32::from_le_bytes`; `sender_context` via `buf[12..20].try_into().unwrap()` (safe after length check)
- [ ] Define `EnipCommandClass` enum with 9 named variants + payloadless `Unknown`; derive `Debug, Clone, Copy, PartialEq, Eq` (BC-2.17.004 Architecture Anchor: exactly 10 variants)
- [ ] Implement `fn classify_enip_command(cmd: u16) -> EnipCommandClass` — exhaustive match with `_ => EnipCommandClass::Unknown` fallback (BC-2.17.004 postcondition 4; invariant 1)
- [ ] Implement `fn is_valid_enip_frame(h: &EnipHeader) -> bool` — returns `true` iff `h.command` is in the 9-value ODVA known-command set; inspects ONLY `h.command` (BC-2.17.003 postcondition 3); no buffer-length parameter
- [ ] Add `#[cfg(kani)] mod kani_proofs` block with Kani harnesses Sub-A `vp032_enip_header_parse_safety` (with `#[kani::unwind(49)]`), Sub-B `vp032_enip_command_classification_biconditional`, and Sub-C `vp032_enip_validity_gate_biconditional`; Sub-D (`vp032_cip_service_classification_totality`) belongs to STORY-132
- [ ] Create `tests/enip_analyzer_tests.rs` with top-level `mod parse_header { ... }` wrapper containing all AC-130-001 through AC-130-005 unit tests
- [ ] Run `cargo check` — zero errors
- [ ] Run `cargo test enip` — all new tests pass
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings

## Test Plan

**Test file:** `tests/enip_analyzer_tests.rs`
**Test module:** `mod parse_header { ... }`

```
parse_header::test_parse_enip_header_valid
parse_header::test_parse_enip_header_too_short
parse_header::test_parse_enip_header_exactly_24
parse_header::test_parse_enip_header_no_panic_empty
parse_header::test_parse_enip_header_no_panic_23_bytes
parse_header::test_classify_enip_command_known
parse_header::test_classify_enip_command_unknown
parse_header::test_classify_enip_command_unknown_zero
parse_header::test_classify_enip_command_unknown_ffff
parse_header::test_classify_enip_command_unknown_gap
parse_header::test_is_valid_enip_frame_known_commands_true
parse_header::test_is_valid_enip_frame_unknown_command_false
parse_header::test_is_valid_enip_frame_boundary_commands
parse_header::test_is_valid_enip_frame_all_fields_zeroed
```

Kani harnesses (not run in `cargo test`; run via `cargo kani`):
```
kani_proofs::vp032_enip_header_parse_safety                  [VP-032 Sub-A]
kani_proofs::vp032_enip_command_classification_biconditional [VP-032 Sub-B]
kani_proofs::vp032_enip_validity_gate_biconditional          [VP-032 Sub-C]
```
Note: `vp032_cip_service_classification_totality` (VP-032 Sub-D) is added by STORY-132.

## Previous Story Intelligence

No SS-17 predecessor stories exist yet; this is the first story in E-20. Reference stories for structural patterns:

- **STORY-106** (DNP3 pure-core parse, Wave 50): pattern for pure-core free functions with Kani proof harnesses — use same `#[cfg(kani)] mod kani_proofs` placement, same sub-property numbering scheme
- **STORY-110** (DNP3 dispatcher integration, Wave 52): shows VP-007 atomic-update and VP-004 oracle patterns — not directly relevant to this story but needed by STORY-131 and STORY-133

Key structural lesson from STORY-106: the `#[kani::unwind(N)]` annotation must set N = max_symbolic_len + 1 (here 48 + 1 = 49). The implementer must NOT set N to the exact loop bound; off-by-one causes Kani to fail to verify the boundary case.

## Architecture Compliance Rules

From ADR-010 and the enip-architecture-delta:

1. **Pure-core / effectful-shell boundary (ADR-010 Decision 2):** `parse_enip_header`, `classify_enip_command`, and `is_valid_enip_frame` MUST be free `fn`s — no `self` parameter, no global state, no I/O. They are Kani proof targets (Sub-A/B/C); any effectful dependency breaks VP-032 validity. `classify_cip_service` (Sub-D, BC-2.17.007) is defined in STORY-132 under the same constraint.
2. **Little-endian byte order (ADR-010 Decision 1):** All multi-byte integer fields in the ENIP header are little-endian. Use `u16::from_le_bytes` and `u32::from_le_bytes` — NOT `from_be_bytes`.
3. **No panic on any input (VP-032 contract):** Every function must be panic-free for arbitrary input. No `unwrap()` on slice indexing — use length checks before accessing fields. The only allowed `unwrap()` is `buf[12..20].try_into().unwrap()` which is safe only after the `buf.len() >= 24` guard.
4. **`Unknown` fallback required (BC-2.17.004 invariant 1):** `classify_enip_command` must have a catch-all `_ => EnipCommandClass::Unknown` arm to remain a total function (VP-032 Sub-B). The `Unknown` arm must be reachable and non-vacuous (BC-2.17.004 postcondition 4).
5. **`EnipCommandClass` / `CipServiceClass` scope boundary:** This story (STORY-130) defines `EnipCommandClass` (10 variants, BC-2.17.004) and the Kani Sub-B/Sub-C harnesses. `CipServiceClass` (15 variants, BC-2.17.007) and the Kani Sub-D harness are in scope of STORY-132. Do NOT define `classify_cip_service` or `CipServiceClass` in this story.
6. **Kani harness placement:** All harnesses MUST be inside `#[cfg(kani)] mod kani_proofs { use super::*; ... }`. Never use `#[cfg(test)]` for Kani harnesses — they are proof artifacts, not unit tests, and must not appear in `cargo test` output.
7. **`sender_context` is `[u8; 8]` NOT `Vec<u8>` (ADR-010 Decision 1):** The 8-byte opaque context field is a fixed-size array for zero-allocation parsing.

## Library & Framework Requirements

No new external crate dependencies. All parsing uses only Rust stdlib:
- `u16::from_le_bytes([buf[0], buf[1]])` — stdlib
- `u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]])` — stdlib
- `<[u8; 8]>::try_from(&buf[12..20]).unwrap()` — stdlib `TryFrom`, safe after bounds check

For Kani: `cargo kani` with Kani ≥ 0.55 (current project version from existing Kani CI job). Sub-A requires `#[kani::unwind(49)]`; other subs have no explicit unwind annotation (exhaustive on finite domains).

## File Structure Requirements

**Files to create:**
- `src/analyzer/enip.rs` — new file; entire SS-17 analyzer module starts here
- `tests/enip_analyzer_tests.rs` — new integration test file for all ENIP stories

**Files to modify:**
- `src/analyzer/mod.rs` — add `pub mod enip;` declaration
- `Cargo.toml` — add `[[test]] name = "enip_analyzer_tests" path = "tests/enip_analyzer_tests.rs"` if not already present (check existing pattern from `dnp3_analyzer_tests` or similar)

**Files NOT touched by this story:**
- `src/dispatcher.rs` — STORY-131 adds ENIP dispatch
- `src/cli.rs` — STORY-131 adds `--enip` CLI flags
- `src/main.rs` — STORY-131 wires the analyzer
- `src/mitre.rs` — STORY-133 adds new MITRE entries

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/analyzer/enip.rs` (structs + 4 fns + Kani harnesses) | ~600 |
| `tests/enip_analyzer_tests.rs` (14 unit tests) | ~500 |
| `src/analyzer/mod.rs` (1-line addition) | ~10 |
| `Cargo.toml` (test target entry) | ~20 |
| **Total** | **~1,130** |

## Dependency Rationale

This story has no SS-17 dependencies — it creates the foundational pure-core types (`EnipHeader`, `EnipCommandClass`) and parse functions (`parse_enip_header`, `classify_enip_command`, `is_valid_enip_frame`) that all subsequent ENIP stories depend on. It must be completed in Wave 58 before any Wave 59+ story can be implemented. `CipServiceClass` and `classify_cip_service` are defined in STORY-132.
