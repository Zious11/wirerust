---
document_type: story
story_id: "STORY-057"
epic_id: "E-5"
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.022.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.023.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.024.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.025.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.026.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.027.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.028.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-055, STORY-056]
blocks: []
behavioral_contracts:
  - BC-2.07.022
  - BC-2.07.023
  - BC-2.07.024
  - BC-2.07.025
  - BC-2.07.026
  - BC-2.07.027
  - BC-2.07.028
verification_properties: [VP-005]
priority: "P0"
cycle: v0.1.0-brownfield
wave: 19
target_module: src/analyzer/tls.rs
subsystems: [SS-07]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield
---

> **Execute:** `/vsdd-factory:deliver-story STORY-057`

# STORY-057: SNI Edge Cases — Empty Lists, Empty Hostnames, Multi-Name, NameType, Trailing Bytes, Large SNI, and Count-Cap Decoupling

## Narrative
- **As a** forensic analyst
- **I want** the TLS analyzer to handle all SNI degenerate cases correctly — empty ServerNameList, zero-length hostname bytes, multi-name SNI (first-only processing), non-zero NameType, trailing bytes in the extension, large SNIs near the record payload limit — and to guarantee that anomaly findings still fire even when the `sni_counts` map is at capacity
- **So that** the analyzer is robust against malformed, unusual, or adversarial TLS SNI extensions without missing findings or panicking

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.07.022 | Empty SNI ServerNameList: No Count, No Finding, Handshake Counted |
| BC-2.07.023 | Empty SNI Hostname Bytes Counted Under "" Key; No Finding |
| BC-2.07.024 | Only FIRST ServerName Entry Processed |
| BC-2.07.025 | Non-Zero NameType Entries Treated as Hostnames |
| BC-2.07.026 | Trailing Bytes in ServerNameList Tolerated |
| BC-2.07.027 | Large SNI (16 KB) Under MAX_RECORD_PAYLOAD Parses Successfully |
| BC-2.07.028 | sni_counts Cap: Finding Still Fires When Map at Capacity |

## Acceptance Criteria

### AC-001 (traces to BC-2.07.022 postcondition 1-4)
When a ClientHello contains an SNI extension but the `ServerNameList` is empty (`list.first()` returns `None`), `extract_sni` returns `None`. `sni_counts` is unchanged, no finding is emitted, and `handshakes_seen` is still incremented (by the outer `handle_client_hello` per BC-2.07.001).
- **Test:** `test_sni_extension_with_empty_hostname_list`

### AC-002 (traces to BC-2.07.022 invariant 1-2)
An SNI extension with empty ServerNameList is treated identically to a ClientHello with no SNI extension at all (from a finding/count perspective). The `None` return from `extract_sni` short-circuits the entire SNI handling block in `handle_client_hello`.
- **Test:** `test_sni_extension_with_empty_hostname_list` (compare with ClientHello without any SNI extension; assert identical state changes)

### AC-003 (traces to BC-2.07.023 postcondition 1-4)
When the SNI `ServerNameList` contains one entry with zero-length hostname bytes (`hostname == b""`), `extract_sni` classifies the empty bytes: `str::from_utf8(b"")` succeeds with `Ok("")`; `"".is_ascii() == true`; `contains_c0_or_del("") == false`. Arm 1 fires: `SniValue::Ascii("")`. `sni_counts[""]` is incremented. No finding is emitted.
- **Test:** `test_sni_with_empty_hostname_bytes`

### AC-004 (traces to BC-2.07.023 invariant 1-2)
The empty string satisfies arm 1 conditions vacuously. The `sni_counts` key for empty-byte SNI is `""` (empty string) — NOT `"<non-utf8:...>"` (that format is only for arm 4).
- **Test:** `test_sni_with_empty_hostname_bytes` (assert `sni_counts.contains_key("")`)

### AC-005 (traces to BC-2.07.024 postcondition 1-4)
When a ClientHello SNI extension contains a `ServerNameList` with 2+ entries, `extract_sni` uses `list.first()` to extract only the first entry. The second and subsequent entries are silently ignored. Only one `sni_counts` entry is inserted and at most one finding is emitted.
- **Test:** `test_multi_name_sni_list_only_first_entry_counted`

### AC-006 (traces to BC-2.07.024 invariant 1-2)
The multi-name SNI behavior is by design. The second+ entries are never inspected for anomalies. If the first entry is clean ASCII and the second entry has C0 bytes, no finding is emitted.
- **Test:** `test_multi_name_sni_list_only_first_entry_counted` (SNI list ["example.com", "evil\x01.com"]; assert no finding)

### AC-007 (traces to BC-2.07.025 postcondition 1-3)
When a ClientHello SNI extension has a `ServerNameList` where the first entry has a non-zero `NameType` byte (e.g., NameType=1), the NameType is discarded (pattern `let Some((_, hostname)) = list.first()`) and only the hostname bytes are passed to the 4-way classification. Behavior is identical to NameType=0 processing.
- **Test:** `test_non_zero_name_type_sni_entry`; `test_non_zero_name_type_with_valid_first_entry`

### AC-008 (traces to BC-2.07.025 invariant 1-3)
NameType validation is intentionally absent. The current implementation does not validate that NameType==0. No finding is emitted solely because of a non-zero NameType.
- **Test:** `test_non_zero_name_type_sni_entry` (NameType=1, hostname="example.com"; assert no finding, sni_counts has one entry)

### AC-009 (traces to BC-2.07.026 postcondition 1-3)
If a TLS ClientHello SNI extension has trailing bytes after the last valid hostname entry (but `parse_tls_extensions` succeeds with a non-empty list), `extract_sni` processes the first hostname entry normally. The trailing bytes are silently ignored. No `parse_errors` are incremented by `extract_sni` itself.
- **Test:** `test_trailing_bytes_in_server_name_list`

### AC-010 (traces to BC-2.07.027 postcondition 1-5)
A ClientHello with a clean ASCII SNI hostname of approximately 16 KB (payload_len <= 18,432) is accepted and parsed without error. `parse_errors` is NOT incremented. The large hostname is classified (arm 1 for clean ASCII) and counted in `sni_counts`. `handshakes_seen` is incremented.
- **Test:** `test_large_sni_near_record_payload_limit`

### AC-011 (traces to BC-2.07.027 invariant 1-2)
`MAX_RECORD_PAYLOAD = 18,432` is the binding size constraint, not `MAX_BUF = 65,536`. A single record of 16 KB fits comfortably in MAX_BUF. The system does not have an SNI-length-specific cap below MAX_RECORD_PAYLOAD.
- **Test:** `test_large_sni_near_record_payload_limit` (assert 16 KB SNI parses without `truncated_records` increment)

### AC-012 (traces to BC-2.07.028 postcondition 1-4)
When `sni_counts` is at `MAX_MAP_ENTRIES = 50,000` capacity and a new anomalous SNI arrives (not already in the map), the new SNI key is NOT inserted into `sni_counts` (count silently dropped), but the anomaly finding IS pushed to `all_findings`. `sni_counts.len()` remains at 50,000. `all_findings.len()` increases by 1.
- **Test:** `test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity`

### AC-013 (traces to BC-2.07.028 invariant 1-2)
Finding emission is decoupled from count insertion. The `Self::increment` call (which silently drops new keys when the map is full) and the `match sni { ... }` block that emits findings are sequential, not conditional on each other. `all_findings` in `TlsAnalyzer` has no cap.
- **Test:** `test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity` (fill to 50,000; then send new anomalous SNI; assert finding still fires despite count drop)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `extract_sni` (`list.first()` guard) | src/analyzer/tls.rs:247-249 | pure-core |
| `TlsAnalyzer::increment` helper | src/analyzer/tls.rs:372-376 | effectful-shell (mutates count map) |
| SNI count insertion in `handle_client_hello` | src/analyzer/tls.rs:402-416 | effectful-shell |
| SNI finding emission in `handle_client_hello` | src/analyzer/tls.rs:424-490 | effectful-shell |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | No SNI extension in ClientHello at all | `extract_sni` returns None; same as empty list |
| EC-002 | Empty SNI list + weak cipher | No SNI finding; weak-cipher finding still fires (independent) |
| EC-003 | First entry is empty bytes, second entry is non-empty | Only empty bytes processed (first-only); `sni_counts[""]++`; no finding |
| EC-004 | First entry has NameType=1, hostname is non-ASCII UTF-8 | Arm 3 fires (NameType discarded; hostname classified normally) |
| EC-005 | SNI hostname = 16,384 bytes of 'a' | Parsed; counted; no finding (all clean ASCII) |
| EC-006 | Map at capacity + clean ASCII SNI (arm 1) | Count dropped; NO finding (arm 1 never emits findings) |
| EC-007 | Map at capacity + anomalous SNI already in map | Count incremented (existing key); finding emitted |
| EC-008 | Map at capacity + anomalous SNI NOT in map | Count dropped; finding still emitted (decoupled) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/analyzer/tls.rs (extract_sni list.first() guard) | pure-core | Returns None without mutation when list is empty |
| src/analyzer/tls.rs (increment helper) | effectful-shell | Mutates count map conditionally |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~4,000 |
| Referenced code (tls.rs lines 247-249, 372-376, 402-490) | ~4,500 |
| Test files (tls_analyzer_tests.rs SNI edge-case tests) | ~5,500 |
| BC files (7 BCs) | ~7,500 |
| Tool outputs overhead | ~2,000 |
| **Total** | **~23,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~12%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-013 (test-writer)
2. [ ] Verify Red Gate: all AC tests fail before implementation
3. [ ] Implement `extract_sni` `list.first()` guard: return `None` when list is empty, per BC-2.07.022
4. [ ] Implement `extract_sni` NameType discard: `let Some((_, hostname)) = list.first()` — `_` pattern for NameType per BC-2.07.025
5. [ ] Implement `TlsAnalyzer::increment` helper with `MAX_MAP_ENTRIES` cap: insert if `map.len() < limit || map.contains_key(&key)`
6. [ ] Wire SNI count insertion BEFORE the `match sni { ... }` block in `handle_client_hello` to ensure finding emission is decoupled from count insertion
7. [ ] Write `test_sni_extension_with_empty_hostname_list` (empty ServerNameList)
8. [ ] Write `test_sni_with_empty_hostname_bytes` (one entry, zero-length bytes; arm 1; sni_counts[""]++)
9. [ ] Write `test_multi_name_sni_list_only_first_entry_counted` (multi-name SNI; only first processed)
10. [ ] Write `test_non_zero_name_type_sni_entry` (NameType=1; hostname treated normally)
11. [ ] Write `test_trailing_bytes_in_server_name_list` (trailing bytes; no parse_errors)
12. [ ] Write `test_large_sni_near_record_payload_limit` (16 KB clean ASCII SNI)
13. [ ] Write `test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity` (fill map; send new anomalous SNI; assert finding fires)
14. [ ] Run all tests; verify all pass
15. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-055 | `extract_sni` returns `Option<SniValue>` — `None` from `list.first()` short-circuits SNI processing in `handle_client_hello` | The `match sni { ... }` block only executes when `extract_sni` returns `Some` | `sni_counts` insertion happens BEFORE the match block to ensure finding emission is decoupled from count cap |
| STORY-056 | Non-UTF-8 SNI uses hex-tagged key `<non-utf8:hex>` in sni_counts | `increment` helper is shared by ja3_counts, ja3s_counts, version_counts, sni_counts | Finding emission and count insertion must be sequential, not conditional — cap on counts must not suppress findings |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| SNI count insertion (Self::increment) MUST occur BEFORE the match-sni-finding block | BC-2.07.028 invariant 1 | Code review: confirm ordering of count insertion and finding push in handle_client_hello |
| `all_findings` in TlsAnalyzer has NO cap (unlike TcpReassembler.findings) | BC-2.07.028 invariant 2 | Code review: confirm no MAX_FINDINGS guard on TlsAnalyzer.all_findings.push |
| NameType is discarded with `_` pattern — no NameType validation | BC-2.07.025 invariant 1-2 | Code review: confirm `let Some((_, hostname)) = list.first()` |
| `MAX_MAP_ENTRIES = 50,000` cap uses `map.len() < limit || map.contains_key(&key)` to allow increment of existing keys | BC-2.07.001 invariant 2 | Unit test: existing key increments even when map is full |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| tls-parser | 0.12 | `TlsExtension::SNI` list access, `(u8, &[u8])` tuple destructure |
| Rust std | 2024 edition (stable) | `HashMap::contains_key`, `HashMap::entry`, iterator `.first()` |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/tls.rs | modify | `extract_sni` guard (247-249), `increment` helper (372-376), count insertion before match-sni block (402-416), SNI finding emission (424-490) |
| tests/tls_analyzer_tests.rs | modify | Empty list, empty hostname, multi-name, NameType, trailing bytes, large SNI, count-cap decoupling tests |
