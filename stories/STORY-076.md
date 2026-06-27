---
document_type: story
story_id: "STORY-076"
epic_id: "E-8"
version: "1.3"
status: completed
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.001.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.002.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.003.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.004.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.005.md
  - .factory/specs/prd.md
input-hash: "9abc907"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-046, STORY-057, STORY-058, STORY-066, STORY-071]
blocks: [STORY-077, STORY-079]
behavioral_contracts:
  - BC-2.11.001
  - BC-2.11.002
  - BC-2.11.003
  - BC-2.11.004
  - BC-2.11.005
verification_properties: [VP-017]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 20
target_module: reporter/json
subsystems: [SS-11]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
nfr:
  - NFR-SEC-004
implementation_strategy: brownfield-formalization
---

# STORY-076: JsonReporter — Structure, skipped_packets, and RFC 8259 Byte Handling

## Narrative
- **As a** security toolchain integrator consuming wirerust JSON output
- **I want** the JSON output to always have exactly three top-level keys (summary, findings, analyzers), always include `skipped_packets` in the summary (even when zero), and faithfully encode forensic bytes per RFC 8259 — C0 as `\uNNNN` escapes, non-ASCII Unicode readable, C1 as raw UTF-8
- **So that** downstream scripts can rely on a stable JSON schema, distinguish zero-error captures from field-absent errors, and safely process international hostnames and attacker-controlled payloads

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.11.001 | JsonReporter Renders JSON Object with summary/findings/analyzers Keys |
| BC-2.11.002 | JsonReporter Includes skipped_packets in Summary |
| BC-2.11.003 | JsonReporter Escapes C0 Control Bytes per RFC 8259 via serde |
| BC-2.11.004 | JsonReporter Preserves Non-ASCII Unicode in Readable Form |
| BC-2.11.005 | JsonReporter Passes C1 Codepoints Through as Raw UTF-8 |

## Acceptance Criteria

### AC-001 (traces to BC-2.11.001 postcondition 2)
`JsonReporter::render` returns a JSON string whose parsed top-level object contains exactly the keys `"summary"`, `"findings"`, and `"analyzers"`. No other top-level keys exist.
- **Test:** `test_BC_2_11_001_top_level_keys()`

### AC-002 (traces to BC-2.11.001 postcondition 3)
`"findings"` is a JSON array with one element per `Finding` in the input slice; an empty findings slice produces `"findings": []`.
- **Test:** `test_BC_2_11_001_findings_array_length()`

### AC-003 (traces to BC-2.11.001 postcondition 5)
The `"summary"` object contains sub-keys `total_packets`, `total_bytes`, `skipped_packets`, `unique_hosts`, `protocols`, and `services`.
- **Test:** `test_BC_2_11_001_summary_subkeys()`

### AC-004 (traces to BC-2.11.001 postcondition 6)
The output is pretty-printed (uses serde_json::to_string_pretty — indented with spaces, one key per line).
- **Test:** `test_BC_2_11_001_output_is_pretty_printed()`

### AC-005 (traces to BC-2.11.002 postcondition 2)
When `Summary.skipped_packets = 0`, the JSON output contains `"skipped_packets": 0` — the key is present with value 0, not absent.
- **Test:** `test_BC_2_11_002_skipped_packets_zero_present()`

### AC-006 (traces to BC-2.11.002 postcondition 3)
When `Summary.skipped_packets = 3`, the JSON output contains `"skipped_packets": 3`.
- **Test:** `test_BC_2_11_002_skipped_packets_nonzero()`

### AC-007 (traces to BC-2.11.003 postcondition 1)
A Finding with ESC (0x1B) in its `summary` field produces JSON where the ESC byte appears as the six-character sequence `` (backslash, u, 0, 0, 1, b), not as a raw 0x1B byte.
- **Test:** `test_BC_2_11_003_c0_esc_escaped_in_json()`

### AC-008 (traces to BC-2.11.003 postcondition 2)
DEL (0x7F) is NOT escaped by serde_json; it passes through as a raw 0x7F byte in the JSON output string.
- **Test:** `test_BC_2_11_003_del_not_escaped_in_json()`

### AC-009 (traces to BC-2.11.003 postcondition 4)
A round-trip (serialize `Finding` with C0 bytes, then deserialize the JSON) recovers the original byte sequence exactly.
- **Test:** `test_BC_2_11_003_c0_roundtrip()`

### AC-010 (traces to BC-2.11.004 postcondition 1)
A Finding with a Cyrillic hostname in `summary` produces JSON where the Cyrillic characters appear as raw UTF-8 bytes, NOT as `\u` escape sequences.
- **Test:** `test_BC_2_11_004_cyrillic_preserved_readable()`

### AC-011 (traces to BC-2.11.005 postcondition 1)
A Finding with U+009B (C1 CSI) in `summary` produces JSON where the CSI appears as the raw two-byte UTF-8 sequence 0xC2 0x9B, NOT as the text ``.
- **Test:** `test_BC_2_11_005_c1_passthrough_raw_utf8()`

### AC-012 (traces to BC-2.11.005 invariant 2)
A Finding with both ESC (C0, 0x1B) and U+009B (C1) in `summary` produces JSON where ESC is `` and C1 is raw 0xC2 0x9B — the two characters are treated differently.
- **Test:** `test_BC_2_11_005_c0_escaped_c1_passthrough_in_same_string()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| JsonReporter | src/reporter/json.rs | pure |
| JsonReporter::render | src/reporter/json.rs:23-60 | pure |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Empty findings slice | `"findings": []` |
| EC-002 | Empty analyzers slice | `"analyzers": []` |
| EC-003 | skipped_packets = u64::MAX | Serialized as JSON integer |
| EC-004 | ESC (0x1B) in summary | `` in JSON string |
| EC-005 | NUL (0x00) in summary | `\x00` in JSON string |
| EC-006 | DEL (0x7F) in summary | Raw 0x7F byte (NOT escaped) |
| EC-007 | C1 CSI (U+009B) in summary | Raw 0xC2 0x9B bytes in JSON |
| EC-008 | Cyrillic in summary | Readable UTF-8, no escapes |
| EC-009 | Backslash in summary | serde_json escapes to double-backslash |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reporter/json.rs | pure | Returns owned String; no I/O; serde_json::Value serialization cannot fail; unwrap at json.rs:60 is infallible |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| src/reporter/json.rs (full file) | ~1,500 |
| BC files (5 BCs) | ~5,000 |
| tests/reporter_tests.rs (relevant sections) | ~1,000 |
| Tool outputs overhead | ~500 |
| **Total** | **~10,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~5.3%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-012 (test-writer)
2. [ ] Verify all tests fail at Red Gate
3. [ ] Verify `src/reporter/json.rs` already satisfies all ACs (brownfield confirm)
4. [ ] Confirm `serde_json::to_string_pretty` is used (not `to_string`)
5. [ ] Confirm `unwrap()` at json.rs:60 is on a `serde_json::Value` (infallible)
6. [ ] Confirm no `escape_for_terminal` call in json.rs (ADR 0003 / INV-4)
7. [ ] Run `cargo test --all-targets` to confirm green
8. [ ] Add round-trip test for C0 bytes: serialize then deserialize recovers original

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A -- first story in E-8 | N/A | N/A | N/A |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `JsonReporter` NEVER calls `escape_for_terminal` — this is TerminalReporter-only per ADR 0003 / INV-4 | BC-2.11.003 invariant 1 | Code review: grep json.rs for `escape_for_terminal`; must be absent |
| `serde_json::to_string_pretty` is the serializer; no custom `Serialize` impl on Finding strings | BC-2.11.003 precondition 2 | Code review |
| `skipped_packets` is always included in the summary JSON object, even when zero | BC-2.11.002 invariant 1 | Test: assert key presence when value is 0 |
| Protocol keys use `{k:?}` (Debug) format in the summary.protocols map | BC-2.11.001 invariant 3 | Code review of json.rs summary serialization |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| serde_json | (per Cargo.lock) | JSON serialization; RFC 8259 C0 escaping; non-ASCII Unicode preservation |
| serde | (per Cargo.lock) | #[derive(Serialize)] on Summary and Finding |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/reporter/json.rs | verify/modify | All 5 BCs live here |
| tests/reporter_tests.rs | create or modify | AC-001 through AC-012 tests |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.2 | 2026-06-01 | story-writer | Corrected source anchor json.rs:59 → json.rs:60 in Purity Classification table and Tasks item 5 (ADV-IMPL-P07-LOW-001); no semantic, AC, BC, or test-name changes |
