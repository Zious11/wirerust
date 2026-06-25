---
document_type: story
story_id: STORY-132
title: "CPF Item Walk, CIP Header Parse, and CIP Request Path Extraction"
epic_id: E-20
wave: 59
points: 8
phase: f3
tdd_mode: strict
status: ready
feature_id: issue-316-enip-analyzer
github_issue: 316
subsystems: [SS-17]
target_module: analyzer/enip
depends_on: [STORY-130]
behavioral_contracts:
  - BC-2.17.005
  - BC-2.17.006
  - BC-2.17.007
  - BC-2.17.009
verification_properties: []
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.005.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.006.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.007.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.009.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
input-hash: "3343540"
---

# STORY-132: CPF Item Walk, CIP Header Parse, and CIP Request Path Extraction

## Narrative

**As a** security analyst inspecting EtherNet/IP traffic,
**I want** the analyzer to correctly walk CPF items, parse CIP headers from Unconnected Data Items
(type_id 0x00B2 only), and extract CIP request path segments (Class, Instance, Attribute),
**so that** threat detection BCs in Wave 60 have well-typed, bounds-safe inputs to work with.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.17.005 | `parse_cpf_items` walks CPF item list | Core implementation target |
| BC-2.17.006 | `parse_cip_header` parses CIP header from 0x00B2 item data | Core implementation target |
| BC-2.17.007 | `classify_cip_service` (effectful use) and service classification in analyzer context | In-scope via re-use |
| BC-2.17.009 | `parse_cip_request_path` extracts Class/Instance/Attribute path segments | Core implementation target |

## Acceptance Criteria

### AC-132-001: `parse_cpf_items` walks CPF items and returns typed item list
**Traces to:** BC-2.17.005 postconditions 1–4
- Given a CPF payload (bytes after the ENIP 24-byte header) with `item_count` at bytes 0–1 (LE u16)
- When `parse_cpf_items(cpf_data: &[u8]) -> Vec<CpfItem>` is called
- Then for each item in the CPF list: `type_id = LE u16 at cursor`, `length = LE u16 at cursor+2`, `data = cpf_data[cursor+4..cursor+4+length]`
- Returns `vec![]` if `cpf_data.len() < 2` (cannot read item_count)
- Stops iteration (returns partial list) on any bounds violation (cursor + 4 > len, or cursor + 4 + item_length > len)
- Never panics for any input
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_parse_cpf_items_single_item`
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_parse_cpf_items_two_items`
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_parse_cpf_items_empty`
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_parse_cpf_items_truncated`

### AC-132-002: `CpfItem` carries type_id and data slice reference
**Traces to:** BC-2.17.005 postcondition 2
- `CpfItem` struct: `type_id: u16`, `data: Vec<u8>` (2 fields; `length` is a TRANSIENT parse local equal to `data.len()`, NOT a struct field — BC-2.17.005 Architecture Anchors)
- `CpfItem { type_id: 0x00B2, data: <N bytes> }` for Unconnected Data Items
- `CpfItem { type_id: 0x00B1, data: <N bytes> }` for Connected Data Items
- `CpfItem { type_id: 0x0000, data: vec![] }` for null address items (length 0 is valid)
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_cpf_item_type_ids`

### AC-132-003: `parse_cip_header` parses CIP header from 0x00B2 item data
**Traces to:** BC-2.17.006 postconditions 1–7
- Given `item.data` from a `CpfItem` with `type_id == 0x00B2`
- When `parse_cip_header(item_data: &[u8]) -> Option<CipHeader>` is called
- Then returns `Some(CipHeader { service, request_path })` if `item_data.len() >= 2` (2-field struct; BC-2.17.006 Architecture Anchors)
  - `service: u8 = item_data[0]`
  - `request_path_size` (in 16-bit words) is a TRANSIENT parse local: `item_data[1] as usize`; NOT a struct field
  - `path_byte_count = request_path_size * 2`
  - Returns `None` if `item_data.len() < 2 + path_byte_count` (truncated path — BC-2.17.006 postcondition 5)
  - `request_path: Vec<u8>` = `item_data[2..2 + path_byte_count]`
  - `general_status` is NOT a CipHeader struct field. Per BC-2.17.008 Postcondition 1, it is extracted at the response CALL SITE (byte 2 of the 0x00B2 item_data, gated `len >= 4`), not stored in CipHeader
- Returns `None` if `item_data.len() < 2`
- Never panics
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_parse_cip_header_request`
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_parse_cip_header_response`
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_parse_cip_header_too_short`
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_parse_cip_header_truncated_path`

### AC-132-004: CIP header parse is 0x00B2-only; 0x00B1 items are skipped
**Traces to:** BC-2.17.006 Invariant 3 (F-P9-001 locked decision)
- When walking CPF items, CIP-layer parsing (calling `parse_cip_header`) is only performed on items where `type_id == 0x00B2`
- Items with `type_id == 0x00B1` (Connected Data Item) are stored in `CpfItem` list but NOT passed to `parse_cip_header` in v0.11.0
- The 2-byte sequence-count prefix on 0x00B1 items would corrupt CIP service/path parse (F-P9-001 rationale)
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_cip_parse_skips_0x00b1_items`
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_cip_parse_processes_0x00b2_items`

### AC-132-005: `parse_cip_request_path` extracts Class, Instance, Attribute segments
**Traces to:** BC-2.17.009 postconditions 1–4
- Given `cip_header.request_path` as `&[u8]`
- When `parse_cip_request_path(path: &[u8]) -> Vec<CipPathSegment>` is called
- Then for each 2-byte segment at cursor:
  - `0x20` + byte → `CipPathSegment::Class(byte)`
  - `0x24` + byte → `CipPathSegment::Instance(byte)`
  - `0x30` + byte → `CipPathSegment::Attribute(byte)`
  - Other → skip (advance by 2)
- Returns `vec![]` for empty path
- Stops at any bounds violation (`cursor + 2 > path.len()`)
- Never panics
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_parse_cip_path_empty`
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_parse_cip_path_class_only`
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_parse_cip_path_class_instance_attr`
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_parse_cip_path_unrecognized_skip`
- **Test:** `tests/enip_analyzer_tests.rs::cpf_cip::test_parse_cip_path_odd_length_safe`

### AC-132-006: F-P9-002 fuzz obligation stub exists for `parse_cip_header` and `parse_cpf_items`
**Traces to:** BC-2.17.007 Invariant 2 (F-P9-002 fuzz obligation)
- Both `parse_cip_header` and `parse_cpf_items` must have `#[cfg(fuzzing)]` fuzz harness stubs (empty bodies with `todo!()` replaced by minimal invocations) OR a `TODO: F-P9-002` comment in the function doc comment pointing to the fuzz obligation
- The fuzz stubs do not need to be functional in v0.11.0; their presence tracks the F-P9-002 obligation
- **Test:** (structural — verify via code review, not automated test)

## Architecture Mapping

| Component | Location | Role |
|-----------|----------|------|
| `CpfItem` struct | `src/analyzer/enip.rs` | `type_id: u16, data: Vec<u8>` (2 fields; BC-2.17.005 arch anchor; `length` is a transient parse local, NOT a field) |
| `parse_cpf_items` | `src/analyzer/enip.rs` | Pure-core free fn; CPF item walk |
| `CipHeader` struct | `src/analyzer/enip.rs` | `service: u8, request_path: Vec<u8>` (2 fields; BC-2.17.006 arch anchor; `request_path_size` and `general_status` are NOT fields) |
| `parse_cip_header` | `src/analyzer/enip.rs` | Pure-core free fn; 0x00B2 item data → CipHeader |
| `CipPathSegment` enum | `src/analyzer/enip.rs` | `Class(u8), Instance(u8), Attribute(u8)` |
| `parse_cip_request_path` | `src/analyzer/enip.rs` | Pure-core free fn; path bytes → segments |
| Test mod | `tests/enip_analyzer_tests.rs` | `mod cpf_cip { ... }` |

**F-P9-001 scope note:** `parse_cip_header` is called ONLY for `type_id == 0x00B2` items in v0.11.0. The `0x00B1` Connected Data Item gate is enforced in the analyzer's item-walk loop, not inside `parse_cip_header` itself. `parse_cip_header` is agnostic to the item type — the call-site gate provides the F-P9-001 restriction.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | CPF data with 0 items (`item_count=0`) | Returns `vec![]` |
| EC-002 | CPF data too short for item_count (0 or 1 byte) | Returns `vec![]` |
| EC-003 | CPF item declares `length=5` but only 3 bytes remain | Stops iteration; returns partial list (0 items in this case) |
| EC-004 | `parse_cip_header` with 1-byte input | Returns `None` |
| EC-005 | CIP header with `request_path_size=0` | `request_path = &[]`; still returns `Some(...)` |
| EC-006 | CIP header with `request_path_size=3` but only 4 bytes available (need 6) | Returns available bytes clamped; no panic |
| EC-007 | `parse_cip_request_path` with 1-byte path | `cursor+2 > 1` at first iteration; returns `vec![]` |
| EC-008 | Path `[0x40, 0x00, 0x20, 0x01]` (unknown 0x40 then Class 0x01) | `[Class(0x01)]` — unknown type skipped |
| EC-009 | Path `[0x24, 0x01]` (Instance only, no Class) | `[Instance(0x01)]` — Class presence not required |
| EC-010 | CPF with `type_id=0x00B1` and `type_id=0x00B2` items | `parse_cip_header` called only for 0x00B2; 0x00B1 stored in CpfItem but CIP parse skipped |

## Tasks

- [ ] Define `CpfItem` struct in `src/analyzer/enip.rs`: `pub struct CpfItem { pub type_id: u16, pub data: Vec<u8> }` (2 fields only; `length` is a transient parse local inside `parse_cpf_items`, NOT a struct field)
- [ ] Implement `pub fn parse_cpf_items(cpf_data: &[u8]) -> Vec<CpfItem>` — read `item_count = LE u16 at [0..2]`; walk items at `cursor = 2`; for each: read `type_id`, read transient `length` (LE u16 at cursor+2), extract `data = cpf_data[cursor+4..cursor+4+length as usize]`; advance `cursor += 4 + length as usize`; break on any bounds error; push `CpfItem { type_id, data }`
- [ ] Define `CipHeader` struct: `pub struct CipHeader { pub service: u8, pub request_path: Vec<u8> }` (2 fields only; `request_path_size` is a transient parse local; `general_status` is NOT a struct field — extracted at the response call site per BC-2.17.008)
- [ ] Implement `pub fn parse_cip_header(item_data: &[u8]) -> Option<CipHeader>` — check `item_data.len() >= 2`; read `service = item_data[0]`; transient `request_path_size = item_data[1] as usize`; `path_byte_count = request_path_size * 2`; return `None` if `item_data.len() < 2 + path_byte_count`; `request_path = item_data[2..2 + path_byte_count].to_vec()`; return `Some(CipHeader { service, request_path })`
- [ ] Define `CipPathSegment` enum: `pub enum CipPathSegment { Class(u8), Instance(u8), Attribute(u8) }` with `Debug, Clone, PartialEq`
- [ ] Implement `pub fn parse_cip_request_path(path: &[u8]) -> Vec<CipPathSegment>` — cursor walk with exact-match on `0x20/0x24/0x30`; `cursor += 2` per segment; break at bounds
- [ ] Add F-P9-002 doc comment `/// # Fuzz Obligation (F-P9-002)` to `parse_cip_header` and `parse_cpf_items`
- [ ] Add `mod cpf_cip { ... }` test wrapper to `tests/enip_analyzer_tests.rs` with all AC-132 unit tests
- [ ] Run `cargo check` — zero errors
- [ ] Run `cargo test enip` — all new tests pass
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings

## Test Plan

**Test file:** `tests/enip_analyzer_tests.rs`
**Test module:** `mod cpf_cip { ... }`

```
cpf_cip::test_parse_cpf_items_single_item
cpf_cip::test_parse_cpf_items_two_items
cpf_cip::test_parse_cpf_items_empty
cpf_cip::test_parse_cpf_items_truncated
cpf_cip::test_cpf_item_type_ids
cpf_cip::test_parse_cip_header_request
cpf_cip::test_parse_cip_header_response
cpf_cip::test_parse_cip_header_too_short
cpf_cip::test_parse_cip_header_truncated_path
cpf_cip::test_cip_parse_skips_0x00b1_items
cpf_cip::test_cip_parse_processes_0x00b2_items
cpf_cip::test_parse_cip_path_empty
cpf_cip::test_parse_cip_path_class_only
cpf_cip::test_parse_cip_path_class_instance_attr
cpf_cip::test_parse_cip_path_unrecognized_skip
cpf_cip::test_parse_cip_path_odd_length_safe
```

## Previous Story Intelligence

STORY-132 depends on STORY-130 for the foundational types:
- `EnipHeader` and `EnipCommandClass` (from STORY-130, BC-2.17.004) are in scope but not directly called in STORY-132's parse functions
- `CipServiceClass` (15 variants, BC-2.17.007) is defined by THIS story (STORY-132) — it is NOT from STORY-130
- `classify_cip_service` (BC-2.17.007) is defined in STORY-132 and used by the Wave 60 detection stories when they call `parse_cip_header` and classify the service; STORY-132 defines both the enum and the classifier

Key lesson from STORY-130: the `#[cfg(kani)]` guard is not needed for STORY-132 functions (BC-2.17.009 explicitly notes `parse_cip_request_path` is not a VP-032 Kani target in v0.11.0). Add doc comments noting the F-P9-002 fuzz obligation instead.

## Architecture Compliance Rules

From ADR-010 Decision 8 (CIP object model scope) and F-P9-001/002:

1. **0x00B2 gate is at the call site, not inside `parse_cip_header` (F-P9-001):** `parse_cip_header` itself is agnostic to item type. The caller (`EnipAnalyzer::process_pdu`) checks `item.type_id == 0x00B2` before calling `parse_cip_header`. This design keeps `parse_cip_header` reusable when v0.12.0 lifts the F-P9-001 restriction.
2. **8-bit logical segments only in `parse_cip_request_path` (ADR-010 Decision 8):** Exact-match on `0x20`, `0x24`, `0x30`. Do NOT use `& 0xE0` mask — it would incorrectly match Instance `0x24` as Class `0x20`. 16-bit extended segments (`0x21`, `0x25`, `0x31`) are deferred to v0.12.0.
3. **No panic on any input (pure-core obligation):** `parse_cpf_items`, `parse_cip_header`, and `parse_cip_request_path` must never panic. Use slice indexing with bounds checks or `get()` rather than direct index. The only allowed `unwrap()` is on `try_into()` after explicit length guards.
4. **CPF `length` field is the data length only (not including type_id + length fields):** The 4-byte item header (type_id 2 bytes + length 2 bytes) is NOT counted in `item.length`. The data occupies `cpf_data[cursor+4..cursor+4+item.length as usize]`.
5. **`general_status` is NOT a CipHeader field (BC-2.17.006 Postcondition 7):** `CipHeader` has exactly 2 fields: `service: u8` and `request_path: Vec<u8>`. `general_status` is extracted at the RESPONSE CALL SITE (byte 2 of the 0x00B2 `item_data`, gated `item_data.len() >= 4`) per BC-2.17.008 Postcondition 1. Do NOT add `general_status` to the `CipHeader` struct.
6. **`request_path_size` is a transient parse local in 16-bit words (ODVA CIP Vol 1):** `path_byte_count = request_path_size * 2`. The path bytes start at `item_data[2]` and span `path_byte_count` bytes. `request_path_size` is NOT a struct field — it is consumed during parsing and discarded.

## Library & Framework Requirements

No new external crate dependencies. All parsing uses stdlib slice ops. The `Vec<u8>` choice for `CpfItem.data` and `CipHeader.request_path` avoids lifetime complexity — the data is small (CPF items are bounded by `MAX_ENIP_CARRY_BYTES = 600`).

## File Structure Requirements

**Files to modify:**
- `src/analyzer/enip.rs` — add `CpfItem`, `parse_cpf_items`, `CipHeader`, `parse_cip_header`, `CipPathSegment`, `parse_cip_request_path`
- `tests/enip_analyzer_tests.rs` — add `mod cpf_cip { ... }` block

**Files NOT touched:**
- `src/dispatcher.rs` — already modified by STORY-131
- `src/cli.rs` — already modified by STORY-131
- `src/main.rs` — already modified by STORY-131
- `src/mitre.rs` — STORY-133 handles MITRE changes

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/analyzer/enip.rs` additions (3 structs + 3 fns) | ~500 |
| `tests/enip_analyzer_tests.rs` cpf_cip mod (16 tests) | ~600 |
| **Total** | **~1,100** |

## Dependency Rationale

STORY-132 is Wave 59 (depends on STORY-130 for `src/analyzer/enip.rs` existence). The CPF/CIP parse layer is the prerequisite for all Wave 60 detection stories (STORY-134, STORY-135, STORY-136, STORY-137) which call `parse_cpf_items`, `parse_cip_header`, and `parse_cip_request_path` to extract typed data for threat detection.
