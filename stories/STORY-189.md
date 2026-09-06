---
document_type: story
level: ops
story_id: STORY-189
title: "S7comm Userdata Structural Parse and Function-Group Classification: Load-Bearing Group 0x03/0x04/0x07 Correction"
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
wave: 92
target_module: analyzer/s7comm
subsystems: [SS-21]
estimated_days: null
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
feature_id: feature-s7comm
depends_on: [STORY-188]
blocks: [STORY-190]
behavioral_contracts: [BC-2.21.018, BC-2.21.019, BC-2.21.020, BC-2.21.021, BC-2.21.022, BC-2.21.023]
verification_properties: [VP-052]
inputs:
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.018.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.019.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.020.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.021.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.022.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.023.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
  - .factory/research/s7comm-mitre-ics-tagging.md
input-hash: "1d59de6"
---

> **tdd_mode:** `strict` — full TDD Iron Law enforced.

# STORY-189: S7comm Userdata Structural Parse and Function-Group Classification

## Narrative

**As a** security analyst using wirerust to inspect classic S7comm Userdata (ROSCTR
`0x07`) traffic,
**I want** the S7comm analyzer to structurally parse the Userdata parameter head and
correctly classify function groups — with the load-bearing correction that group `0x03`
is Block functions and group `0x07` is Time functions (the reverse of a common
documentation error) —
**so that** downstream MITRE T0888 (Remote System Information Discovery) emission
(STORY-192) keys on the correct group/subfunction combination.

This story is purely classification (part B1) — no `Finding` is emitted here. It extends
`S7commAnalyzer::on_data`'s classic-S7comm branch for `header.rosctr == Userdata`,
completing the `S7ClassicFunction` enum's `Userdata(...)` arm left open by STORY-188.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.21.018 | Userdata Parameter Block Structural Parse — Parameter Head, Group/Subfunction Extraction, Bounds-Safe Reject | Structural parse, 7-byte minimum |
| BC-2.21.019 | Userdata Block Functions (Group 0x03) Classified — Load-Bearing Correction | Block functions, NOT group 0x07 |
| BC-2.21.020 | Userdata CPU Functions (Group 0x04) Subfunction 0x01 Classified as Read SZL | Primary discovery signal |
| BC-2.21.021 | Userdata CPU Functions (Group 0x04) Other Subfunctions Classified `CpuOther` | No force-fit |
| BC-2.21.022 | Userdata Time Functions (Group 0x07) Classified — Corrected, NOT Block Functions | Negative-space counterpart to BC-2.21.019 |
| BC-2.21.023 | Unrecognized Userdata Function Group — Totality Anchor, No Invented Group ID | Terminal fallback arm |

## Acceptance Criteria

### AC-189-001: Userdata parameter block structural parse rejects param_length < 7
(traces to BC-2.21.018 postcondition 1)
- Given `header.rosctr == Userdata` and `param_length < 7`
- When the Userdata structural parser runs
- Then the Userdata parameter block is treated as malformed: one T0814
  (`ThreatCategory::Anomaly`/`Verdict::Possible`/`Confidence::Medium`) per flow
  direction, sharing the `malformed_header_reported_c2s`/`_s2c` dedup flag; no
  function-group classification is attempted
- **Test:** `test_BC_2_21_018_param_length_below_7_malformed`

### AC-189-002: Group and subfunction extracted correctly for param_length >= 7
(traces to BC-2.21.018 postcondition 2)
- Given `param_length >= 7`
- When the Userdata structural parser runs
- Then the function group is the low nibble of `data[header_len + 4]` and the
  subfunction is `data[header_len + 5]`
- The Parameter Head (`data[header_len..header_len+3]`) is read for presence but its
  exact bytes are not validated against the conventional `0x00 0x01 0x12` marker —
  informational only, never a hard gate (traces to BC-2.21.018 postcondition 3)
- The Sequence Number (`data[header_len + 6]`) is extracted but never compared or
  branched on (traces to BC-2.21.018 postcondition 4)
- **Test:** `test_BC_2_21_018_group_subfunction_extraction`

### AC-189-003: Group 0x03 classified as Block Functions (load-bearing correction)
(traces to BC-2.21.019 postcondition 1)
- Given the function-group nibble equals `0x03`
- When the Userdata classifier runs
- Then the frame is classified `Userdata(BlockFunctions(subfn))`; `subfn == 0x01` ->
  "List Blocks" (traces to BC-2.21.019 postcondition 2), `subfn == 0x02` -> "List Blocks
  of Type" (postcondition 3), `subfn == 0x03` -> "Get Block Info" (postcondition 4), any
  other `subfn` -> `BlockFunctions(subfn)` with the raw byte preserved, never force-fit
  to one of the three named operations (traces to BC-2.21.019 postcondition 5)
- **Test:** `test_BC_2_21_019_group_0x03_block_functions_classified` — this test MUST
  assert group `0x03` specifically, not group `0x07` (regression guard for the
  documentation-error class this BC exists to prevent)

### AC-189-004: Group 0x04 subfunction 0x01 classified as CPU Read SZL
(traces to BC-2.21.020 postcondition 1)
- Given the function-group nibble equals `0x04` and `subfn == 0x01`
- When the Userdata classifier runs
- Then the frame is classified `Userdata(CpuReadSzl)`; no SZL-ID-specific decoding is
  performed at this layer (traces to BC-2.21.020 postcondition 2)
- **Test:** `test_BC_2_21_020_group_0x04_subfn_0x01_read_szl`

### AC-189-005: Group 0x04 other subfunctions classified CpuOther, no force-fit
(traces to BC-2.21.021 postcondition 1)
- Given the function-group nibble equals `0x04` and `subfn != 0x01`
- When the Userdata classifier runs
- Then the frame is classified `Userdata(CpuOther(subfn))`, preserving the raw
  subfunction byte; no specific named operation is asserted for any `CpuOther` value
  (traces to BC-2.21.021 postcondition 2)
- **Test:** `test_BC_2_21_021_group_0x04_other_subfunctions_cpu_other`

### AC-189-006: Group 0x07 classified as Time Functions — NOT Block Functions (corrected)
(traces to BC-2.21.022 postcondition 1)
- Given the function-group nibble equals `0x07`
- When the Userdata classifier runs
- Then the frame is classified `Userdata(TimeFunctions(subfn))`; this classification is
  never conflated with, aliased to, or co-classified as `BlockFunctions` (traces to
  BC-2.21.022 postcondition 2)
- **Test:** `test_BC_2_21_022_group_0x07_time_functions_classified_not_block_functions`
  — this is the matched-pair regression guard against BC-2.21.019's negative-space
  counterpart; both tests MUST pass together to prove the group `0x03`/`0x07` swap has
  not occurred

### AC-189-007: Unrecognized function group classified OtherGroup, no invented group ID
(traces to BC-2.21.023 postcondition 1)
- Given the function-group nibble is not `0x03`, `0x04`, or `0x07`
- When the Userdata classifier runs
- Then the frame is classified `Userdata(OtherGroup(group, subfn))` with both raw values
  preserved; no `Finding` is emitted for an unrecognized-but-parseable group value
  (traces to BC-2.21.023 postcondition 2)
- No group ID beyond `0x03`/`0x04`/`0x07` is ever asserted with a specific named
  semantic (e.g. no invented "Security functions" group) — this feature deliberately
  does not assert an unverified group-ID mapping
- **Test:** `test_BC_2_21_023_unrecognized_group_other_group_no_invented_id`

### AC-189-008: The Userdata function-group match is total over all 16 nibble values
(traces to BC-2.21.023 invariant — VP-052 totality obligation, Userdata sub-part)
- Given any 4-bit function-group nibble value
- When the Userdata classifier runs
- Then exactly one of BC-2.21.019 (`0x03`), BC-2.21.020/021 (`0x04`), BC-2.21.022
  (`0x07`), or BC-2.21.023 (the remaining 13 values) applies
- The harness MUST specifically assert group `0x03` classifies as `BlockFunctions` and
  group `0x07` classifies as `TimeFunctions` (not vice versa) — a transposed
  implementation would otherwise pass a naive totality check without this explicit
  non-vacuity assertion
- **Test:** `proptest_vp052_userdata_group_totality` (skeleton in this story, full
  non-vacuous run in STORY-194)

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `S7UserdataFunction` enum | SS-21 data model | `src/analyzer/s7comm.rs` | N/A (`BlockFunctions`, `CpuReadSzl`, `CpuOther`, `TimeFunctions`, `OtherGroup`) |
| `S7ClassicFunction::Userdata(...)` arm | SS-21 data model | `src/analyzer/s7comm.rs` | N/A (completes the enum from STORY-188) |
| `parse_userdata_parameter_block` | SS-21 structural parser | `src/analyzer/s7comm.rs` | Pure (free fn) |
| `classify_userdata_function` | SS-21 classifier | `src/analyzer/s7comm.rs` | Pure (free fn, VP-052 target) |

Subsystem anchor: SS-21 owns this story's scope — Userdata structural parse and
function-group classification are core dissection capabilities of the S7comm analyzer
per ARCH-INDEX.md §SS-21, and this story carries the feature's single most
research-flagged correctness risk (the group `0x03`/`0x07` documentation-error class).

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `parse_userdata_parameter_block`, `classify_userdata_function` | pure-core | Return plain data by value; no mutation, no finding emission, no I/O |
| `S7UserdataFunction` | pure-core | Plain data type |

## VP-052 Proptest Obligation (Userdata sub-part — completes the harness started in STORY-188)

**Harness:** `proptest_vp052_userdata_group_totality` (this story)
**Method:** proptest
**Priority:** P1

Non-vacuity requirement (per the VP-052 registration): the harness MUST specifically
assert group `0x03` -> `BlockFunctions` and group `0x07` -> `TimeFunctions`, not merely
that some totality property holds — a transposed implementation must fail this harness.
Full non-vacuous run (combined with STORY-188's FC totality sub-part) is in STORY-194.

## Tasks

- [ ] Define `pub enum S7UserdataFunction { BlockFunctions(u8), CpuReadSzl,
      CpuOther(u8), TimeFunctions(u8), OtherGroup(u8, u8) }`
- [ ] Extend `S7ClassicFunction` (from STORY-188) with the `Userdata(S7UserdataFunction)`
      arm
- [ ] Implement `fn parse_userdata_parameter_block(data: &[u8], header_len: usize,
      param_length: u16) -> Option<(u8, u8)>` returning `(group, subfn)` or `None` if
      `param_length < 7` (BC-2.21.018)
- [ ] Implement `fn classify_userdata_function(group: u8, subfn: u8) ->
      S7UserdataFunction`:
  - `group == 0x03` -> `BlockFunctions(subfn)` (BC-2.21.019) — **verify against the
    corrected mapping: group 0x03 = Block functions, NOT group 0x07**
  - `group == 0x04, subfn == 0x01` -> `CpuReadSzl` (BC-2.21.020)
  - `group == 0x04, subfn != 0x01` -> `CpuOther(subfn)` (BC-2.21.021)
  - `group == 0x07` -> `TimeFunctions(subfn)` (BC-2.21.022) — **verify against the
    corrected mapping: group 0x07 = Time functions, NOT Block functions**
  - any other group -> `OtherGroup(group, subfn)` (BC-2.21.023)
- [ ] Wire the Userdata structural parse + classifier into `S7commAnalyzer::on_data`'s
      classic-S7comm branch for `header.rosctr == Userdata`, emitting the shared T0814
      malformed-header anomaly on `param_length < 7` (reusing STORY-187's dedup flags)
- [ ] Write `proptest_vp052_userdata_group_totality` skeleton with the explicit
      non-vacuous group-`0x03`/`0x07` assertion
- [ ] Write unit tests: one per AC, named `test_BC_2_21_018_*` .. `test_BC_2_21_023_*`
- [ ] Extend `tests/fixtures/mk_s7comm_pcap.py` with Userdata/SZL-read frames
      (group `0x04` subfn `0x01`) and a Block-functions frame (group `0x03`)
- [ ] Verify `cargo test` passes
- [ ] Add a CHANGELOG entry under `[Unreleased] > Added` describing the Userdata
      structural parse and function-group classification, before creating the PR

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.21.018 | `param_length == 6` (one short of the 7-byte minimum) | Malformed; T0814 |
| EC-002 | BC-2.21.018 | Parameter Head bytes are not `0x00 0x01 0x12` (non-conventional) | Not itself a rejection — informational only |
| EC-003 | BC-2.21.019 | `subfn == 0xFF` under group `0x03` | `BlockFunctions(0xFF)` — recognized group, unnamed subfunction, raw byte preserved |
| EC-004 | BC-2.21.022 | A test implementation accidentally swaps groups `0x03` and `0x07` | MUST be caught by AC-189-003/006's paired regression-guard tests and the VP-052 non-vacuous totality proptest |
| EC-005 | BC-2.21.023 | Function-group nibble `0x02` (a "Security functions" group per some secondary sources) | Classified `OtherGroup(0x02, subfn)` — no invented "Security" semantic asserted without independent verification |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~5,400 |
| BC-2.21.018-023 (6 BCs) | ~6,500 |
| ADR-014 Decision 5 (group correction), s7comm-mitre-ics-tagging.md | ~5,000 |
| src/analyzer/s7comm.rs (from STORY-187/188) | ~6,000 |
| Test file delta + fixture generator extension | ~3,500 |
| **Total** | **~26,400** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~13%** |

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-188 | `S7ClassicFunction` enum established for Job/Ack_Data FCs; `classify_job_ack_function` pure classifier | Classification-only pattern: no finding emission until part B2 stories | This story's group `0x03`/`0x07` distinction is the single highest-value correctness property in the whole feature per ADR-014's own explicit framing — implement the two match arms as textually adjacent in the source and cross-reference each other in a comment, to make an accidental swap maximally visible in code review |

## Architecture Compliance Rules

Extracted from `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md`:
- **ADR-014 Decision 5 (load-bearing correction)**: Userdata function group `0x03` is
  Block functions (subfunctions `0x01` List Blocks, `0x02` List Blocks of Type, `0x03`
  Get Block Info); group `0x07` is Time functions (clock read/set) — the reverse of a
  common documentation error some secondary sources make. This mapping is load-bearing
  for STORY-192's T0888 emission call-site.
- This feature does not assert an unverified group-ID semantic (e.g. a "Security"
  functions group) — `OtherGroup(group, subfn)` preserves raw values without inventing
  meaning the source research does not support.
- Pure/effectful boundary: `parse_userdata_parameter_block` and
  `classify_userdata_function` are pure; the `on_data` call site remains the effectful
  shell.

## Library & Framework Requirements

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | Bitwise nibble extraction, match patterns |
| proptest | 1 (pinned in `Cargo.toml`) | VP-052 Userdata-group totality skeleton with non-vacuous group-0x03/0x07 assertion |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `src/analyzer/s7comm.rs` | MODIFY | Add `S7UserdataFunction`, extend `S7ClassicFunction::Userdata(...)`, `parse_userdata_parameter_block`, `classify_userdata_function`; wire into `on_data` |
| `tests/s7comm_analyzer_tests.rs` | MODIFY | Add BC-2.21.018-023 unit tests + VP-052 Userdata-group proptest skeleton (with explicit 0x03/0x07 non-vacuity assertion) |
| `tests/fixtures/mk_s7comm_pcap.py` | MODIFY | Add Userdata/SZL-read and Block-functions synthetic frames |

## Forbidden Dependencies

- Wireshark, Snap7, libnodave source, and any `s7`/`s7-comm`/`s7-client` crate — banned/
  avoid per ADR-014 Decision 4
- `classify_userdata_function` MUST NOT invent a named semantic for any function group
  beyond `0x03`/`0x04`/`0x07` without independent research verification

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-06 | story-writer | Initial authorship — Userdata structural parse, function-group classification with the load-bearing group 0x03/0x07 correction, VP-052 Userdata sub-part skeleton, AC-189-001..008. |
