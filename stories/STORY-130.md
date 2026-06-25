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
input-hash: "d709bd4"
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
| BC-2.17.001 | `parse_enip_header` parses 24-byte ENIP encapsulation header | Core implementation target |
| BC-2.17.002 | `classify_enip_command` maps command u16 to `EnipCommand` enum | Core implementation target |
| BC-2.17.003 | `is_valid_enip_frame` validates header-declared length against buffer length | Core implementation target |
| BC-2.17.004 | `classify_cip_service` maps CIP service byte to `CipServiceClass` enum | Core implementation target |

## Acceptance Criteria

### AC-130-001: `parse_enip_header` parses 24-byte header with little-endian fields
**Traces to:** BC-2.17.001 postconditions 1–3
- Given a byte slice of exactly 24 bytes with known field values
- When `parse_enip_header(&bytes)` is called
- Then `Some(EnipHeader { command, length, session_handle, status, sender_context, options })` is returned
- With `command = LE u16 bytes[0..2]`, `length = LE u16 bytes[2..4]`, `session_handle = LE u32 bytes[4..8]`, `status = LE u32 bytes[8..12]`, `sender_context = bytes[12..20]` as `[u8;8]`, `options = LE u32 bytes[20..24]`
- For any slice shorter than 24 bytes, returns `None`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_parse_enip_header_valid`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_parse_enip_header_too_short`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_parse_enip_header_exactly_24`

### AC-130-002: `parse_enip_header` never panics for any byte slice input
**Traces to:** BC-2.17.001 postcondition 4 (no panic guarantee)
- Given any `&[u8]` of any length (0 to large)
- When `parse_enip_header(&bytes)` is called
- Then returns `Some(...)` if `bytes.len() >= 24`, `None` otherwise — no panic, no UB
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_parse_enip_header_no_panic_empty`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_parse_enip_header_no_panic_23_bytes`

### AC-130-003: `classify_enip_command` correctly maps all in-scope command codes
**Traces to:** BC-2.17.002 postconditions 1–2
- `0x0065` → `EnipCommand::RegisterSession`
- `0x0066` → `EnipCommand::UnRegisterSession`
- `0x0063` → `EnipCommand::ListIdentity`
- `0x0004` → `EnipCommand::ListServices`
- `0x0064` → `EnipCommand::ListInterfaces`
- `0x0072` → `EnipCommand::SendRRData`
- `0x006F` → `EnipCommand::SendUnitData`
- `0x0070` → `EnipCommand::IndicateStatus`
- `0x0075` → `EnipCommand::Cancel`
- Any other value → `EnipCommand::Unknown(value)`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_classify_enip_command_known`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_classify_enip_command_unknown`

### AC-130-004: `is_valid_enip_frame` validates length field against actual buffer
**Traces to:** BC-2.17.003 postconditions 1–3
- Given a header with `length` field value `L` and a total buffer of `B` bytes
- When `is_valid_enip_frame(&header, buffer_len)` is called
- Then returns `true` iff `(24 + L as usize) <= buffer_len` (i.e., the payload declared by the header fits within the buffer)
- Returns `false` if `header.length == 0 && buffer_len < 24` (undersized)
- Returns `true` if `header.length == 0 && buffer_len >= 24` (zero-payload frame is valid)
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_is_valid_enip_frame_exact_fit`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_is_valid_enip_frame_buffer_too_small`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_is_valid_enip_frame_zero_payload`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_is_valid_enip_frame_overflow_guard`

### AC-130-005: `classify_cip_service` maps service byte to `CipServiceClass`
**Traces to:** BC-2.17.004 postconditions 1–2
- `service & 0x7F` (low 7 bits) determines the class; `service & 0x80` indicates response (but classification uses the low bits only)
- `0x0E` or `0x8E` → `CipServiceClass::GetAttributeSingle`
- `0x01` or `0x81` → `CipServiceClass::GetAttributesAll`
- `0x55` or `0xD5` → `CipServiceClass::GetAttributeList`
- `0x10` or `0x90` → `CipServiceClass::SetAttributeSingle`
- `0x02` or `0x82` → `CipServiceClass::SetAttributeList`
- `0x54` or `0xD4` → `CipServiceClass::ForwardOpen`
- `0x4E` or `0xCE` → `CipServiceClass::ForwardClose`
- `0x4B` or `0xCB` → `CipServiceClass::UnconnectedSend`
- Any other low-7-bit value → `CipServiceClass::Unknown(service)`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_classify_cip_service_request`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_classify_cip_service_response`
- **Test:** `tests/enip_analyzer_tests.rs::parse_header::test_classify_cip_service_unknown`

### AC-130-006: VP-032 Kani proof harnesses pass for all 4 sub-properties
**Traces to:** VP-032 Sub-A through Sub-D
- Sub-A (`kani_parse_enip_header_no_panic`): `parse_enip_header` never panics for any symbolic `&[u8]` up to 49 bytes; `unwind(49)` annotation required
- Sub-B (`kani_classify_enip_command_total`): `classify_enip_command` is total — returns for every `u16` input with no unreachable branch
- Sub-C (`kani_is_valid_enip_frame_no_overflow`): `is_valid_enip_frame` `(24 + L as usize)` never overflows for any `L: u16`
- Sub-D (`kani_classify_cip_service_total`): `classify_cip_service` is total — returns for every `u8` input
- All harnesses live in `src/analyzer/enip.rs` under `#[cfg(kani)] mod kani_proofs { ... }`
- **Test:** `kani::kani_parse_enip_header_no_panic` (Sub-A)
- **Test:** `kani::kani_classify_enip_command_total` (Sub-B)
- **Test:** `kani::kani_is_valid_enip_frame_no_overflow` (Sub-C)
- **Test:** `kani::kani_classify_cip_service_total` (Sub-D)

## Architecture Mapping

| Component | Location | Role |
|-----------|----------|------|
| `parse_enip_header` | `src/analyzer/enip.rs` | Pure-core free fn, VP-032 Sub-A target |
| `classify_enip_command` | `src/analyzer/enip.rs` | Pure-core free fn, VP-032 Sub-B target |
| `is_valid_enip_frame` | `src/analyzer/enip.rs` | Pure-core free fn, VP-032 Sub-C target |
| `classify_cip_service` | `src/analyzer/enip.rs` | Pure-core free fn, VP-032 Sub-D target |
| `EnipHeader` struct | `src/analyzer/enip.rs` | 24-byte header model: command, length, session_handle, status, sender_context, options |
| `EnipCommand` enum | `src/analyzer/enip.rs` | 9 named variants + Unknown(u16) |
| `CipServiceClass` enum | `src/analyzer/enip.rs` | 8 named variants + Unknown(u8) |
| `kani_proofs` mod | `src/analyzer/enip.rs` | `#[cfg(kani)]` Kani harnesses |
| Test mod | `tests/enip_analyzer_tests.rs` | `mod parse_header { ... }` namespace |

**Pure/effectful boundary (ADR-010 Decision 2):** All four functions are pure-core free `fn`s — no `&self`, no `&mut self`, no global state, no I/O. They are the Kani verification targets and must remain pure to maintain VP-032 proof validity.

## VP Kani Obligation (VP-032)

VP-032 specifies 4 Kani proof harnesses for the pure-core parse functions. All harnesses live in `#[cfg(kani)] mod kani_proofs` inside `src/analyzer/enip.rs`.

**Sub-A — `kani_parse_enip_header_no_panic`:**
```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;
    #[kani::proof]
    #[kani::unwind(49)]
    fn kani_parse_enip_header_no_panic() {
        let len: usize = kani::any();
        kani::assume(len <= 48);
        let bytes: Vec<u8> = (0..len).map(|_| kani::any()).collect();
        let _ = parse_enip_header(&bytes);
    }
```
- `unwind(49)` covers all symbolic lengths 0..=48 (covering both `< 24` and `>= 24` cases).

**Sub-B — `kani_classify_enip_command_total`:**
```rust
    #[kani::proof]
    fn kani_classify_enip_command_total() {
        let cmd: u16 = kani::any();
        let _ = classify_enip_command(cmd);
    }
```

**Sub-C — `kani_is_valid_enip_frame_no_overflow`:**
```rust
    #[kani::proof]
    fn kani_is_valid_enip_frame_no_overflow() {
        let length: u16 = kani::any();
        let buf_len: usize = kani::any();
        // Must not overflow: 24usize + (length as usize)
        // Proof: u16::MAX = 65535; 24 + 65535 = 65559 < usize::MAX on all targets
        let header = EnipHeader { command: 0, length, session_handle: 0,
                                   status: 0, sender_context: [0u8;8], options: 0 };
        let _ = is_valid_enip_frame(&header, buf_len);
    }
```

**Sub-D — `kani_classify_cip_service_total`:**
```rust
    #[kani::proof]
    fn kani_classify_cip_service_total() {
        let svc: u8 = kani::any();
        let _ = classify_cip_service(svc);
    }
} // end kani_proofs
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `parse_enip_header` with 0 bytes | Returns `None` |
| EC-002 | `parse_enip_header` with 23 bytes | Returns `None` |
| EC-003 | `parse_enip_header` with 24 bytes all-zero | Returns `Some(EnipHeader { all zeros })` |
| EC-004 | `classify_enip_command(0x0000)` | Returns `EnipCommand::Unknown(0x0000)` |
| EC-005 | `classify_enip_command(0xFFFF)` | Returns `EnipCommand::Unknown(0xFFFF)` |
| EC-006 | `is_valid_enip_frame` with `length=0xFFFF, buf_len=usize::MAX` | `24 + 65535 = 65559 <= usize::MAX` → `true`; no overflow |
| EC-007 | `is_valid_enip_frame` with `length=0, buf_len=24` | `24 + 0 = 24 <= 24` → `true` |
| EC-008 | `is_valid_enip_frame` with `length=1, buf_len=24` | `24 + 1 = 25 > 24` → `false` |
| EC-009 | `classify_cip_service(0x8E)` (GetAttributeSingle response) | Returns `CipServiceClass::GetAttributeSingle` |
| EC-010 | `classify_cip_service(0xFF)` (unknown response bit set) | Returns `CipServiceClass::Unknown(0xFF)` |

## Tasks

- [ ] Create `src/analyzer/enip.rs` with module-level doc comment citing ADR-010
- [ ] Define `EnipHeader` struct with fields: `command: u16`, `length: u16`, `session_handle: u32`, `status: u32`, `sender_context: [u8; 8]`, `options: u32`
- [ ] Implement `fn parse_enip_header(buf: &[u8]) -> Option<EnipHeader>` — slice bounds check `buf.len() >= 24`, then LE reads via `u16::from_le_bytes` / `u32::from_le_bytes`; `sender_context` via `buf[12..20].try_into().unwrap()` (safe after length check)
- [ ] Define `EnipCommand` enum with 9 named variants + `Unknown(u16)`; derive `Debug, Clone, Copy, PartialEq, Eq`
- [ ] Implement `fn classify_enip_command(cmd: u16) -> EnipCommand` — exhaustive match with `_ => Unknown(cmd)` fallback
- [ ] Implement `fn is_valid_enip_frame(header: &EnipHeader, buffer_len: usize) -> bool` — `(24usize + header.length as usize) <= buffer_len`; note `u16 as usize` cast is widening, no overflow possible
- [ ] Define `CipServiceClass` enum with 8 named variants + `Unknown(u8)`; derive `Debug, Clone, Copy, PartialEq, Eq`
- [ ] Implement `fn classify_cip_service(service: u8) -> CipServiceClass` — match on `service & 0x7F` (strip response bit), `_ => Unknown(service)` fallback
- [ ] Add `#[cfg(kani)] mod kani_proofs` block with all 4 Kani harnesses (Sub-A with `#[kani::unwind(49)]`, Sub-B, Sub-C, Sub-D)
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
parse_header::test_is_valid_enip_frame_exact_fit
parse_header::test_is_valid_enip_frame_buffer_too_small
parse_header::test_is_valid_enip_frame_zero_payload
parse_header::test_is_valid_enip_frame_overflow_guard
parse_header::test_classify_cip_service_request
parse_header::test_classify_cip_service_response
parse_header::test_classify_cip_service_unknown
```

Kani harnesses (not run in `cargo test`; run via `cargo kani`):
```
kani_proofs::kani_parse_enip_header_no_panic     [VP-032 Sub-A]
kani_proofs::kani_classify_enip_command_total    [VP-032 Sub-B]
kani_proofs::kani_is_valid_enip_frame_no_overflow [VP-032 Sub-C]
kani_proofs::kani_classify_cip_service_total     [VP-032 Sub-D]
```

## Previous Story Intelligence

No SS-17 predecessor stories exist yet; this is the first story in E-20. Reference stories for structural patterns:

- **STORY-106** (DNP3 pure-core parse, Wave 50): pattern for pure-core free functions with Kani proof harnesses — use same `#[cfg(kani)] mod kani_proofs` placement, same sub-property numbering scheme
- **STORY-110** (DNP3 dispatcher integration, Wave 52): shows VP-007 atomic-update and VP-004 oracle patterns — not directly relevant to this story but needed by STORY-131 and STORY-133

Key structural lesson from STORY-106: the `#[kani::unwind(N)]` annotation must set N = max_symbolic_len + 1 (here 48 + 1 = 49). The implementer must NOT set N to the exact loop bound; off-by-one causes Kani to fail to verify the boundary case.

## Architecture Compliance Rules

From ADR-010 and the enip-architecture-delta:

1. **Pure-core / effectful-shell boundary (ADR-010 Decision 2):** `parse_enip_header`, `classify_enip_command`, `is_valid_enip_frame`, and `classify_cip_service` MUST be free `fn`s — no `self` parameter, no global state, no I/O. They are Kani proof targets; any effectful dependency breaks VP-032 validity.
2. **Little-endian byte order (ADR-010 Decision 1):** All multi-byte integer fields in the ENIP header are little-endian. Use `u16::from_le_bytes` and `u32::from_le_bytes` — NOT `from_be_bytes`.
3. **No panic on any input (VP-032 contract):** Every function must be panic-free for arbitrary input. No `unwrap()` on slice indexing — use length checks before accessing fields. The only allowed `unwrap()` is `buf[12..20].try_into().unwrap()` which is safe only after the `buf.len() >= 24` guard.
4. **`Unknown` fallback required (BC-2.17.002, BC-2.17.004):** Both `classify_enip_command` and `classify_cip_service` must have a catch-all `_ => Unknown(value)` arm to remain total functions (VP-032 Sub-B and Sub-D).
5. **CIP response bit stripping (BC-2.17.004):** `classify_cip_service` MUST match on `service & 0x7F`, NOT on `service` directly. Matching on the raw byte would miss response variants (e.g., `0x8E` for GetAttributeSingle response).
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

This story has no SS-17 dependencies — it creates the foundational pure-core types (`EnipHeader`, `EnipCommand`, `CipServiceClass`) and parse functions that all subsequent ENIP stories depend on. It must be completed in Wave 58 before any Wave 59+ story can be implemented.
