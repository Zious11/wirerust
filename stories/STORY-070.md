---
document_type: story
story_id: STORY-070
epic_id: E-7
version: "1.7"
status: completed
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.005.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.006.md
input-hash: "9266b29"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-069]
blocks: [STORY-071]
behavioral_contracts:
  - BC-2.09.005
  - BC-2.09.006
verification_properties: []
priority: P0
cycle: v0.1.0-greenfield-spec
wave: 2
target_module: findings
subsystems: [SS-09]
estimated_days: 2
tdd_mode: strict
nfr:
  - NFR-SEC-001
  - NFR-OBS-010
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — full TDD Iron Law enforced.

> **Execute:** `/vsdd-factory:deliver-story STORY-070`

# STORY-070: Raw-Data Contract and JSON Serialization Symmetry (skip_serializing_if)

## Narrative
- **As a** SIEM consumer or security analyst
- **I want** all Finding optional fields (mitre_technique, source_ip, timestamp, direction) to be completely absent from JSON when `None` (never `null`), and `Finding.summary`/`evidence` to carry raw attacker-controlled bytes without pre-escaping
- **So that** SIEM ingestion logic can use key-presence to detect optional fields, and forensic byte evidence is preserved intact for analysis

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.09.005 | Finding.summary and Evidence Store RAW Post-from_utf8_lossy Bytes per ADR 0003 |
| BC-2.09.006 | Finding JSON Serialization: All None Option Fields Omitted via skip_serializing_if |

## Acceptance Criteria

### AC-001 (traces to BC-2.09.005 postcondition 1)
`Finding.summary` contains the post-`from_utf8_lossy` bytes without any additional escaping at construction time. Given a URI containing C1 control byte U+009B (CSI), `finding.summary` contains the literal U+009B byte (not any escape form).
- **Test:** `test_finding_summary_preserves_raw_c1_bytes()`

### AC-002 (traces to BC-2.09.005 postcondition 3)
`escape_for_terminal` is NOT called at any `Finding` construction site. Grep-based assertion: `grep -rn 'escape_for_terminal' src/ | grep -v reporter/terminal` returns no output — confirming that no occurrence of `escape_for_terminal` exists outside `src/reporter/terminal.rs`.
- **Test:** `test_escape_for_terminal_contained_to_terminal_module()` (code-level assertion)

### AC-003 (traces to BC-2.09.005 postcondition 4)
When a `Finding` with an ESC byte in `summary` is serialized by `JsonReporter`, the JSON output contains `\u{1b}` (RFC 8259 serde encoding by `serde_json`), NOT the literal ESC byte.
- **Test:** `test_output_sanitization_layering_contract()`

### AC-004 (traces to BC-2.09.005 invariant 3)
Invalid UTF-8 sequences in `summary` or `evidence` are replaced by U+FFFD (replacement character) via `String::from_utf8_lossy`; no panic occurs.
- **Test:** `test_non_utf8_bytes_in_summary_replaced_with_fffd()`

### AC-005 (traces to BC-2.09.006 postcondition 2)
When `mitre_technique = None`, the JSON object for the Finding has NO `"mitre_technique"` key (not `"mitre_technique": null`).
- **Test:** `test_none_mitre_technique_absent_from_json()`

### AC-006 (traces to BC-2.09.006 postcondition 2)
When `source_ip = None`, the JSON object has NO `"source_ip"` key.
- **Test:** `test_none_source_ip_absent_from_json()`

### AC-007 (traces to BC-2.09.006 postcondition 2)
When `direction = None`, the JSON object has NO `"direction"` key.
- **Test:** `test_none_direction_absent_from_json()`

### AC-008 (traces to BC-2.09.006 postcondition 2)
When `timestamp = None` (always), the JSON object has NO `"timestamp"` key in any produced Finding.
- **Test:** `test_timestamp_absent_from_all_finding_json()`

### AC-009 (traces to BC-2.09.006 postcondition 1)
When `mitre_technique = Some("T1036")`, the JSON object contains `"mitre_technique": "T1036"`.
- **Test:** `test_some_mitre_technique_present_in_json()`

### AC-010 (traces to BC-2.09.006 postcondition 1)
When `direction = Some(ClientToServer)`, the JSON object contains `"direction": "ClientToServer"`.
- **Test:** `test_some_direction_present_in_json()`

### AC-011 (traces to BC-2.09.006 invariant 3)
Reassembly-engine findings with `direction: None` (lifecycle, segment-limit-summary) produce JSON with no `"direction"` key.
- **Test:** `test_reassembly_lifecycle_finding_no_direction_in_json()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `Finding` serde attributes | `src/findings.rs` | pure-core |
| `escape_for_terminal` call site | `src/reporter/terminal.rs:44` | pure-core (the function itself is pure; call site is effectful context) |
| `JsonReporter::render` | `src/reporter/json.rs` | pure-core (string construction) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Full pipeline: Finding with ESC in URI | JSON has `\u{1b}`; terminal output has escape form; Finding.summary has literal 0x1B |
| EC-002 | Finding with `source_ip = Some(IpAddr::V4(...))` | JSON has `"source_ip": "1.2.3.4"` |
| EC-003 | The three serializable Option fields (mitre_technique, source_ip, direction) are Some | Those three keys present in JSON; timestamp always absent (O-01 domain debt) — bound test: `test_story_070_ec003_three_some_option_fields_present_in_json` |
| EC-004 | All four Option fields are None | Zero of the four keys present in JSON |
| EC-005 | `evidence = vec!["raw\x00bytes"]` | JSON encodes null byte as the literal six-character string `\u0000` (RFC 8259); `finding.evidence[0]` contains the literal raw NUL byte at construction time |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `src/findings.rs` | pure-core | Serde attribute is a compile-time annotation; serialization is pure |
| `src/reporter/json.rs` | pure-core | Pure string construction via serde_json |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,800 |
| `src/findings.rs` | ~4,000 |
| `src/reporter/json.rs` | ~3,000 |
| `src/reporter/terminal.rs` (escape function) | ~1,500 |
| `tests/reporter_tests.rs` | ~3,000 |
| BC files (2 BCs) | ~3,500 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~18,800** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~9%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-011 (test-writer)
2. [ ] Verify Red Gate: all tests fail
3. [ ] Add `#[serde(skip_serializing_if = "Option::is_none")]` to all four Option fields on `Finding`
4. [ ] Verify the serde attribute is present on `mitre_technique`, `source_ip`, `timestamp`, and `direction`
5. [ ] Write `test_output_sanitization_layering_contract` to assert JSON output contains `\u{1b}` (not literal ESC)
6. [ ] Write a code-level test asserting `escape_for_terminal` has no occurrence/call outside `src/reporter/terminal.rs` (module-containment invariant per BC-2.09.005)
7. [ ] Write edge-case tests for EC-001 through EC-005
8. [ ] Verify: no analyzer file calls `escape_for_terminal` at Finding construction sites
9. [ ] Run `cargo test --all-targets` and `cargo clippy -- -D warnings`

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-069 | `Finding` struct defined with all fields | Display does not escape; raw bytes pass through | `timestamp` is always None — all 22 sites set `timestamp: None` per O-01 domain debt |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| All four Option fields use `skip_serializing_if = "Option::is_none"` (symmetric — fixed in P1.02) | BC-2.09.006 invariant (symmetric after P1.02 fix) | Test: parse JSON, assert key absent when value is None |
| `escape_for_terminal` is defined and invoked exclusively within `src/reporter/terminal.rs`; no occurrence exists outside that file | BC-2.09.005 invariant 1 | Grep: `grep -rn 'escape_for_terminal' src/ | grep -v reporter/terminal` returns no output |
| `from_utf8_lossy` is the only transformation at construction time | BC-2.09.005 invariant 3 | Code review: no other byte transformation in analyzer emission sites |
| Downstream consumers must use key-absence (not null check) for option detection | BC-2.09.006 invariant 1 | Test: JSON has no `null` values for Finding Option fields |

**Forbidden Dependencies:**
- `src/findings.rs` must NOT import `src/reporter/terminal.rs` (would allow calling `escape_for_terminal` in the data layer, violating ADR 0003)

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `serde` | workspace version | `#[serde(skip_serializing_if = "Option::is_none")]` on Option fields |
| `serde_json` | workspace version | JSON serialization for reporter tests |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/findings.rs` | modify | Add `skip_serializing_if` to all four Option fields |
| `tests/reporter_tests.rs` | modify | Add AC-001..AC-011 test functions |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.5 | 2026-05-22 | story-writer | Wave 2 Ph3 pass-4 adversarial fix: M-1 — removed embedded raw NUL bytes (0x00) from EC-005 Expected Behavior cell (and from changelog v1.4 row); EC-005 now reads with literal six ASCII characters \\u0000 (RFC 8259 JSON encoding of null byte); no binary NUL bytes remain in file (verified: grep -c $'\000' returns 0) |
| 1.4 | 2026-05-22 | story-writer | Wave 2 Ph3 pass-3 adversarial fixes: M-1 — corrected EC-005 Expected Behavior: null byte JSON-encodes as \u0000 (RFC 8259), not \x00 (invalid JSON escape); kept separate claim that finding.evidence[0] holds literal raw NUL at construction time. N-1 — AC-001 test reference updated from test_finding_summary_preserves_raw_c0_bytes to test_finding_summary_preserves_raw_c1_bytes; description corrected to C1 control byte U+009B (CSI) instead of C0/ESC byte 0x1B, reflecting httparse URI rejection of C0 bytes |
| 1.3 | 2026-05-22 | story-writer | Wave 2 Ph3 pass-2 adversarial fixes: M-1 Task 6 reworded to module-containment invariant per BC-2.09.005 (no longer implies single call site); m-1 raw ESC byte (0x1B) in AC-003, EC-001, and Task 5 replaced with literal \u{1b} (now readable); m-3 AC-002 test name updated to test_escape_for_terminal_contained_to_terminal_module |
| 1.2 | 2026-05-22 | story-writer | Wave 2 Ph3 adversarial fixes: AC-002 grep command kept, contradiction resolved by removing false "wc -l == 1" exclusivity from both AC and Architecture Compliance Rule; both now state the verifiable property (no escape_for_terminal outside terminal.rs); EC-003 relabeled to "three serializable Option fields" (timestamp always None per O-01 domain debt); bound test name updated to test_story_070_ec003_three_some_option_fields_present_in_json |
| 1.1 | 2026-05-21 | story-writer | Initial story decomposition |
| 1.7 | 2026-06-09 | story-writer | UPDATED (Feature #7 migration note): JSON serialization tests in the STORY-070 scope assert the `mitre_technique` field. STORY-100 (v0.3.0) renames this field to `mitre_techniques: Vec<String>`. The `skip_serializing_if` attribute changes from `Option::is_none` to `Vec::is_empty`. JSON assertions updated from scalar `"TXXXX"` to array `["TXXXX"]`. CSV and terminal output are behavior-preserving for singleton vecs. The old `"mitre_technique"` key is absent from all JSON output after STORY-100 lands. Story status remains `completed`; no re-implementation required. |
