---
document_type: story
story_id: STORY-102
epic_id: E-14
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-06-09T00:00:00Z
phase: 4
inputs:
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.001.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.002.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.003.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.004.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.005.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.006.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.007.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.008.md
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/phase-f2-spec-evolution/f2-fix-directives.md
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-100]
blocks: [STORY-103]
behavioral_contracts:
  - BC-2.14.001
  - BC-2.14.002
  - BC-2.14.003
  - BC-2.14.004
  - BC-2.14.005
  - BC-2.14.006
  - BC-2.14.007
  - BC-2.14.008
verification_properties:
  - VP-022
priority: P0
cycle: v0.4.0-modbus
wave: 32
target_module: analyzer
subsystems: [SS-14]
estimated_days: 3
tdd_mode: strict
feature_id: issue-007-modbus-analyzer
github_issue: 7
# BC status: all 8 BCs authored at v1.0/v2.0 as of 2026-06-09
modified:
  - "v1.1 (F4/STORY-102 adversarial finding reconciliation): Length gate upper bound corrected from 253 to 254 in AC-005, AC-004 body, EC-007/EC-008, Task 5, Architecture Compliance table, and BC title row for BC-2.14.004. BC-2.14.004 is authoritative: valid range [2, 254]; Length = 254 is true (valid max); Length = 255 is false (over max). Earlier v1.0 stale value 253 was a pre-F2-fix residual."
input-hash: "cfe3fbe"
---

# STORY-102: Modbus MBAP Parse + FC Classification (Pure Core)

## Narrative

- **As a** ICS/OT security analyst using wirerust against Modbus TCP captures
- **I want** the Modbus analyzer to correctly parse MBAP headers and classify all 256 function codes, with formal Kani proofs verifying no panic paths
- **So that** the pure-core parsing and classification layer is formally verified before the stateful flow-correlation layer is built on top of it in STORY-103

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.14.001 | MBAP Header Accepted for Well-Formed 8-Byte-Minimum ADU |
| BC-2.14.002 | MBAP Header Rejected for ADU Shorter Than 8 Bytes |
| BC-2.14.003 | MBAP Header Rejected When Protocol ID is Not 0x0000 |
| BC-2.14.004 | MBAP Header Rejected When Length is Outside [2, 254] |
| BC-2.14.005 | classify_fc Is Total Over All 256 FC Values |
| BC-2.14.006 | Exception Response Detection — FC High Bit Set Identifies Exception and Recovers Original FC |
| BC-2.14.007 | Write-Class FC Classification — State-Changing Function Codes Identified as Elevated-Risk |
| BC-2.14.008 | Diagnostic-Class FC Classification and Sub-Function Dispatch (0x08 and 0x2B) |

## Acceptance Criteria

### AC-001 (traces to BC-2.14.001 postcondition 1 — parse returns Some for >=8 bytes)
`parse_mbap_header(data: &[u8]) -> Option<MbapHeader>` returns `Some(MbapHeader { ... })` when `data.len() >= 8`. All five fields are decoded big-endian: `transaction_id = u16::from_be_bytes([data[0], data[1]])`, `protocol_id = u16::from_be_bytes([data[2], data[3]])`, `length = u16::from_be_bytes([data[4], data[5]])`, `unit_id = data[6]`, `function_code = data[7]`.
- **Test:** `test_parse_mbap_header_returns_some_for_minimum_8_bytes()` — input canonical vector `00 01 00 00 00 06 01 03 00 00 00 0A`; assert `Some(MbapHeader { transaction_id: 0x0001, protocol_id: 0x0000, length: 6, unit_id: 0x01, function_code: 0x03 })`.

### AC-002 (traces to BC-2.14.002 — returns None for <8 bytes)
`parse_mbap_header(data)` returns `None` when `data.len() < 8` (i.e., 0 through 7 bytes inclusive). No panic on any input length.
- **Test:** `test_parse_mbap_header_returns_none_for_short_slices()` — assert `None` for lengths 0, 1, 4, 7; assert `Some(_)` for length 8 and 260.

### AC-003 (traces to BC-2.14.001 postcondition 8 — ADU boundary: offset advances by 6 + length)
The caller's ADU-boundary loop advances the offset by `6 + header.length` after a successful parse. The `6` is the fixed MBAP prefix (TxnID=2 + ProtoID=2 + Length=2 bytes that are NOT included in the `length` field itself). This offset computation is used by the `on_data` parsing loop implemented in STORY-103.
- **Test:** `test_adu_offset_advance_is_6_plus_length()` — parse a 260-byte slice where the first ADU has `length=6` (total 12 bytes); assert the next ADU starts at offset 12.

### AC-004 (traces to BC-2.14.003 — protocol ID gate)
`is_valid_modbus_adu(header: &MbapHeader) -> bool` returns `false` when `header.protocol_id != 0x0000`. The function is a pure 3-point validity gate: protocol ID check, length lower bound (`length >= 2`), length upper bound (`length <= 254`).
- **Test:** `test_is_valid_modbus_adu_rejects_non_zero_protocol_id()` — `MbapHeader { protocol_id: 0x0001, length: 6, ... }` → `false`. `MbapHeader { protocol_id: 0x0000, length: 6, ... }` → `true`.

### AC-005 (traces to BC-2.14.004 — length gate: range [2, 254])
`is_valid_modbus_adu` returns `false` when `header.length < 2` or `header.length > 254`. Length = 0 → `false`. Length = 1 → `false`. Length = 2 → `true`. Length = 254 → `true` (valid max). Length = 255 → `false` (over max).
- **Test:** `test_is_valid_modbus_adu_length_boundary_values()` — assert the boundary cases above.

### AC-006 (traces to BC-2.14.005 — classify_fc is total over all 256 values)
`classify_fc(fc: u8) -> FunctionCodeClass` is a pure, total function: it returns one of `{Read, Write, Diagnostic, Exception, Unknown}` for all 256 possible `u8` values, never panics, and has no unreachable arm.
- **Test:** `test_classify_fc_is_total()` — iterate all 256 u8 values; assert each returns a valid `FunctionCodeClass` variant (no panic). Implemented as `for fc in 0u8..=255 { let _ = classify_fc(fc); }`.

### AC-007 (traces to BC-2.14.006 — exception FC: high bit set)
`classify_fc(fc)` returns `FunctionCodeClass::Exception` when `fc >= 0x80` (high bit set). The original FC is recovered via `fc & 0x7F`.
- **Test:** `test_classify_fc_exception_when_high_bit_set()` — assert `classify_fc(0x83) == Exception`; assert `classify_fc(0x80) == Exception`; assert `classify_fc(0xFF) == Exception`. Assert `0x83 & 0x7F == 0x03` (original FC recovered).

### AC-008 (traces to BC-2.14.007 — write-class FCs: {0x05, 0x06, 0x0F, 0x10, 0x15, 0x16, 0x17})
`classify_fc` returns `FunctionCodeClass::Write` for exactly the set {0x05, 0x06, 0x0F, 0x10, 0x15, 0x16, 0x17}. No other FC (with high bit clear) maps to `Write`.
- **Test:** `test_classify_fc_write_class_completeness()` — assert `Write` for all 7 write FCs; assert `!= Write` for a representative sample of non-write FCs (e.g., 0x01, 0x03, 0x08, 0x2B, 0x00, 0x7F).

### AC-009 (traces to BC-2.14.008 — diagnostic FCs: {0x08, 0x2B})
`classify_fc(0x08)` returns `FunctionCodeClass::Diagnostic`. `classify_fc(0x2B)` returns `FunctionCodeClass::Diagnostic`. All other FCs with clear high bit and not in the Write or Read sets return `Unknown`.
- **Test:** `test_classify_fc_diagnostic_class()` — assert `Diagnostic` for 0x08 and 0x2B; assert `Unknown` for representative unclassified FCs.

### AC-010 (traces to BC-2.14.001 invariant 2 — VP-022 Kani harness: sub-property A)
A Kani harness `#[cfg_attr(kani, kani::proof)]` named `verify_parse_mbap_no_panic` verifies: for all symbolic `&[u8]` inputs of length 0..16, `parse_mbap_header` never panics and returns `None` iff `data.len() < 8`. This is VP-022 sub-property A.
- **Test:** `#[kani::proof] fn verify_parse_mbap_no_panic()` in `src/analyzer/modbus.rs` gated by `#[cfg(kani)]`. `cargo test` passes the standard unit tests; Kani is run separately in the formal-hardening wave.

### AC-011 (traces to BC-2.14.005 — VP-022 Kani harness: sub-property B)
A Kani harness `verify_classify_fc_no_panic` verifies: for all 256 symbolic `u8` inputs, `classify_fc` never panics and always returns a valid `FunctionCodeClass` variant.
- **Test:** `#[kani::proof] fn verify_classify_fc_no_panic()` gated by `#[cfg(kani)]`.

### AC-012 (traces to BC-2.14.001 invariant 4 — parse does not gate on protocol_id or length)
`parse_mbap_header` does NOT check `protocol_id` or `length` internally. Those validity checks belong to `is_valid_modbus_adu`. A parse of `00 00 00 01 00 01 01 01` (protocol_id=1, length=1 — invalid) returns `Some(MbapHeader { ... })` with the raw values parsed; `is_valid_modbus_adu` then returns `false`.
- **Test:** `test_parse_mbap_header_does_not_gate_on_protocol_or_length()` — parse an invalid PDU; assert `Some(_)` from `parse_mbap_header`; assert `false` from `is_valid_modbus_adu`.

### AC-013 (traces to BC-2.14.001 invariant 2 — is_non_modbus desync bail)
`ModbusAnalyzer` carries an `is_non_modbus: bool` flag on `ModbusFlowState`. When set to `true` (flow has been identified as non-Modbus due to parse failures), subsequent `on_data` calls bail immediately without attempting to parse further PDUs. This flag is set when a well-formed-length PDU fails the 3-point validity gate — indicating the flow is likely not carrying Modbus TCP data.
- **Test:** `test_is_non_modbus_bail_after_desync()` — deliver an invalid PDU (wrong protocol_id) followed by a valid Modbus PDU; assert the second PDU is not processed (no finding emitted, no pdu_count increment after desync).

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `parse_mbap_header(data: &[u8]) -> Option<MbapHeader>` | `src/analyzer/modbus.rs` | Pure core (VP-022 Kani target) |
| `MbapHeader` struct | `src/analyzer/modbus.rs` | Pure (data) |
| `is_valid_modbus_adu(header: &MbapHeader) -> bool` | `src/analyzer/modbus.rs` | Pure core |
| `classify_fc(fc: u8) -> FunctionCodeClass` | `src/analyzer/modbus.rs` | Pure core (VP-022 Kani target) |
| `FunctionCodeClass` enum | `src/analyzer/modbus.rs` | Pure (data) |
| `ModbusFlowState.is_non_modbus: bool` | `src/analyzer/modbus.rs` | Effectful (per-flow flag) |
| VP-022 Kani harnesses (sub-property A and B) | `src/analyzer/modbus.rs` (`#[cfg(kani)]`) | Pure (proofs) |

**Subsystem anchor justification:** SS-14 owns this story's complete scope because `parse_mbap_header`, `classify_fc`, and all supporting types live in `src/analyzer/modbus.rs` — SS-14 (Modbus/ICS Analysis, C-22) per ARCH-INDEX Subsystem Registry.

**Dependency anchor justification:** STORY-102 depends on STORY-100 because `ModbusAnalyzer` will emit `Finding { mitre_techniques: vec![...] }` starting in STORY-104. The `mitre_techniques: Vec<String>` field must exist before `src/analyzer/modbus.rs` can be written (it imports `src/findings.rs`). STORY-102 creates the file and the pure core functions; those functions reference no `Finding` type, but having the migration already done prevents a mid-stream type conflict.

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | 8-byte slice (minimum valid ADU) | `parse_mbap_header` returns `Some(_)`; offset advance = `6 + header.length` |
| EC-002 | 260-byte slice (maximum valid ADU) | `parse_mbap_header` returns `Some(_)` for first 8 bytes; trailing 252 bytes not consumed |
| EC-003 | All-zeros 8-byte input | `parse_mbap_header` returns `Some(MbapHeader { 0, 0, 0, 0, 0 })`; `is_valid_modbus_adu` returns `false` (length=0 < 2) |
| EC-004 | FC = 0x7F (no high bit, undefined) | `classify_fc(0x7F)` returns `Unknown` |
| EC-005 | FC = 0xFF (all bits set) | `classify_fc(0xFF)` returns `Exception` (high bit set; original FC = 0x7F) |
| EC-006 | Protocol ID = 0xFFFF (non-Modbus) | `parse_mbap_header` parses it as `Some(MbapHeader { protocol_id: 0xFFFF, ... })`; `is_valid_modbus_adu` returns `false` → `is_non_modbus = true` on the flow |
| EC-007 | Length = 254 (upper bound) | `is_valid_modbus_adu` returns `true` (254 <= 254); ADU boundary: `6 + 254 = 260` bytes |
| EC-008 | Length = 255 (first above upper bound) | `is_valid_modbus_adu` returns `false` |
| EC-009 | FC = 0x01 (Read Coils — Read class) | `classify_fc(0x01)` returns `Read` |
| EC-010 | FC = 0x2B sub-function 0x0E (MEI Read Device ID) | Diagnostic class; sub-function dispatch in STORY-104 |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~4,500 |
| `src/analyzer/modbus.rs` (new file — pure core functions) | ~4,000 |
| BC files (8 BCs: BC-2.14.001 through BC-2.14.008) | ~12,000 |
| `f2-fix-directives.md` (Decision 12/13 for context) | ~3,000 |
| `architecture-delta.md` (§2.4 MBAP parse model) | ~2,000 |
| `tests/modbus_tests.rs` (new test file — pure core tests) | ~5,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~31,500** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~16%** |

## Tasks (MANDATORY)

1. [ ] Create `tests/modbus_tests.rs` (or a new `#[cfg(test)]` module in `src/analyzer/modbus.rs`). Write failing tests for AC-001 through AC-009 and AC-012. Red Gate: `src/analyzer/modbus.rs` does not exist yet; tests fail to compile.
2. [ ] **Red Gate:** Confirm `cargo build` fails because `src/analyzer/modbus.rs` is missing.
3. [ ] Create `src/analyzer/modbus.rs`. Define: `MbapHeader` struct (5 fields, all `u16`/`u8`/`u8`/`u8`), `FunctionCodeClass` enum (`Read`, `Write`, `Diagnostic`, `Exception`, `Unknown`).
4. [ ] Implement `parse_mbap_header(data: &[u8]) -> Option<MbapHeader>`: check `data.len() >= 8`; decode 5 fields big-endian; return `Some` or `None`.
5. [ ] Implement `is_valid_modbus_adu(header: &MbapHeader) -> bool`: 3-point gate — `protocol_id == 0x0000 && length >= 2 && length <= 254`.
6. [ ] Implement `classify_fc(fc: u8) -> FunctionCodeClass`: match arm for `>=0x80` → `Exception`; match sets for Write (`{0x05,0x06,0x0F,0x10,0x15,0x16,0x17}`), Diagnostic (`{0x08,0x2B}`), Read (`{0x01,0x02,0x03,0x04,0x07,0x0B,0x0C,0x11,0x14}`); default → `Unknown`. The function MUST be total (no unreachable panic).
7. [ ] Add `ModbusFlowState` stub (empty struct with `is_non_modbus: bool` field) to anchor the desync-bail contract (AC-013). Full field list in STORY-103.
8. [ ] Add VP-022 Kani harnesses: `verify_parse_mbap_no_panic` (sub-property A) and `verify_classify_fc_no_panic` (sub-property B), both gated by `#[cfg(kani)]`.
9. [ ] Register `modbus` module in `src/analyzer/mod.rs` (add `pub mod modbus;`).
10. [ ] **Green Gate:** `cargo build --all-targets` exits 0. `cargo test --all-targets` green. AC-001 through AC-013 pass.
11. [ ] `cargo clippy --all-targets -- -D warnings` clean.
12. [ ] `cargo fmt --check` clean.

## Previous Story Intelligence (MANDATORY)

N/A — first story in E-14 (Modbus analyzer epic). No previous Modbus stories exist.

**Design reference from f2-fix-directives.md and architecture-delta.md:**
- `parse_mbap_header` is pure and does NOT check validity (`protocol_id`, `length`) internally — separation of concerns enables Kani formal proofs over all 2^(8*8) possible 8-byte inputs.
- `is_valid_modbus_adu` is the 3-point gate called by the `on_data` loop (STORY-103). It is also pure.
- `classify_fc` uses the high-bit rule for exceptions first (`fc >= 0x80`), then the write-class set, then diagnostic, then read, then `Unknown` — this order ensures totality.
- The `is_non_modbus` desync flag prevents false findings on flows that happen to use port 502 but carry non-Modbus data (per BC-2.14.025 EC-001/EC-004).

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `parse_mbap_header` is a PURE function — no I/O, no global state, no mutation | BC-2.14.001 invariant 2 | VP-022 Kani harness; code review |
| `classify_fc` is a TOTAL function — returns a value for all 256 u8 inputs, never panics | BC-2.14.005 | AC-006 loop test; VP-022 Kani sub-property B |
| `parse_mbap_header` does NOT check `protocol_id` or `length` — those belong to `is_valid_modbus_adu` | BC-2.14.001 invariant 4 | AC-012 test; code review |
| Exception FCs (>= 0x80) are classified BEFORE write/read/diagnostic in `classify_fc` | BC-2.14.006 — high bit takes priority | Code review: match arm order |
| `src/analyzer/modbus.rs` must NOT import `src/reporter/` | Architecture layer rule (L3 analyzer must not depend on L4 reporters) | Compiler module system |
| `MbapHeader.length` range [2, 254] is enforced in `is_valid_modbus_adu`, NOT in `parse_mbap_header` | BC-2.14.004; BC-2.14.001 invariant 4 | AC-005 + AC-012 tests |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `kani` | workspace version (nightly, gated by `#[cfg(kani)]`) | VP-022 Kani formal proofs — sub-properties A and B |
| No other new dependencies | — | Pure core functions use only standard Rust primitives |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/analyzer/modbus.rs` | **create** | `MbapHeader`, `FunctionCodeClass`, `parse_mbap_header`, `is_valid_modbus_adu`, `classify_fc`, `ModbusFlowState` stub, VP-022 Kani harnesses |
| `src/analyzer/mod.rs` | **modify** | Add `pub mod modbus;` |
| `tests/modbus_tests.rs` | **create** | Unit tests for AC-001 through AC-013 |

## Forbidden Dependencies

`src/analyzer/modbus.rs` MUST NOT import:
- `src/reporter/` (L3 must not depend on L4)
- `src/reassembly/` internals (only the `StreamHandler` trait via `src/reassembly/handler.rs` — but that import comes in STORY-105)
- Any external crates beyond `serde` (already a workspace dependency) — specifically NOT `nom`, `byteorder`, or any parse-combinator library. Parsing is done via direct `u16::from_be_bytes` and index access per BC-2.14.001.
