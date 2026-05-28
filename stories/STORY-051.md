---
document_type: story
story_id: "STORY-051"
epic_id: "E-5"
version: "1.1"
status: in-progress
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.006.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.007.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.008.md
input-hash: "6efccbd"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-033, STORY-071]
blocks: [STORY-052, STORY-053]
behavioral_contracts:
  - BC-2.07.006
  - BC-2.07.007
  - BC-2.07.008
verification_properties: [VP-013]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 15
target_module: src/analyzer/tls.rs
subsystems: [SS-07]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **Execute:** `/vsdd-factory:deliver-story STORY-051`

# STORY-051: JA3 and JA3S Computation — GREASE Filtering and String Format

## Narrative
- **As a** malware researcher
- **I want** the JA3 and JA3S fingerprint computation functions to produce deterministic, GREASE-filtered MD5 hex strings in the correct 5-field and 3-field formats
- **So that** TLS fingerprints are identical to those produced by other JA3-compliant tools, enabling correlation with known malware signatures

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.07.006 | JA3 Computation Filters GREASE Values per RFC 8701 |
| BC-2.07.007 | JA3 String Format: version,ciphers,...; MD5 Hex |
| BC-2.07.008 | JA3S String Format: version,cipher,extensions; MD5 Hex |

## Acceptance Criteria

### AC-001 (traces to BC-2.07.006 postcondition 1)
All values `v` where `(v & 0x0F0F) == 0x0A0A` are excluded from JA3/JA3S cipher, extension, and curve fields. A ClientHello with cipher list `[0x0a0a, 0x002f]` produces the same JA3 hash as a ClientHello with cipher list `[0x002f]` only.
- **Test:** `test_BC_2_07_006_grease_cipher_excluded_same_hash_as_without_grease` (tests/tls_analyzer_tests.rs)
- **Companion test:** `test_BC_2_07_006_all_grease_cipher_list_produces_empty_cipher_field` (tests/tls_analyzer_tests.rs)

### AC-002 (traces to BC-2.07.006 postcondition 3)
Non-GREASE values are preserved in their original order. Inserting a canonical GREASE value at any position in the cipher list does not change the resulting JA3 hash.
- **Test:** `compute_ja3_is_grease_invariant` (proptest, src/analyzer/tls.rs::ja3_property_tests — VP-013 anchor, do not rename)
- **Companion test:** `test_BC_2_07_006_grease_inserted_at_front_middle_end_same_hash` (tests/tls_analyzer_tests.rs)

### AC-003 (traces to BC-2.07.006 invariant 1)
The GREASE `is_grease_u16` bitmask `(val & 0x0F0F) == 0x0A0A` is applied to cipher IDs, extension type IDs, and named group IDs. EC point format bytes are NOT filtered (they are single bytes, not u16 values).
- **Test:** `is_grease_u16_matches_all_canonical_grease_values` (proptest, src/analyzer/tls.rs::ja3_property_tests — VP-013 anchor, do not rename)
- **Also covered by:** `is_grease_u16_matches_nibble_bitmask_contract` (proptest, src/analyzer/tls.rs::ja3_property_tests — VP-013 anchor, do not rename)
- **Companion tests:** `test_BC_2_07_006_all_16_canonical_grease_ciphers_produce_empty_cipher_field`, `test_BC_2_07_006_non_canonical_grease_pattern_0x0a1a_is_filtered`, `test_BC_2_07_006_ec_point_format_bytes_are_not_filtered` (tests/tls_analyzer_tests.rs)

### AC-004 (traces to BC-2.07.007 postcondition 1-2)
The JA3 string has exactly 4 commas (5 fields). The first field is the decimal representation of `version` (e.g., `771` for TLS 1.2 / 0x0303).
- **Test:** `compute_ja3_has_five_fields_and_hex_hash` (proptest, src/analyzer/tls.rs::ja3_property_tests — VP-013 anchor, do not rename)
- **Companion tests:** `test_BC_2_07_007_ja3_string_has_exactly_four_commas_five_fields`, `test_BC_2_07_007_version_zero_companion_no_cipher_produces_known_hash` (tests/tls_analyzer_tests.rs)

### AC-005 (traces to BC-2.07.007 postcondition 3-6)
The cipher field is decimal IDs of non-GREASE ciphers joined by `-`; if all ciphers are GREASE or none exist, the cipher field is `""`. The extension field is decimal type IDs of non-GREASE extensions joined by `-`. The curves field is decimal group IDs of non-GREASE named groups joined by `-`. The point-format field is decimal bytes from EcPointFormats joined by `-`.
- **Test:** `compute_ja3_has_five_fields_and_hex_hash` (proptest, src/analyzer/tls.rs::ja3_property_tests — VP-013 anchor, do not rename)
- **Companion tests:** `test_BC_2_07_007_cipher_field_is_decimal_not_hex`, `test_BC_2_07_007_empty_cipher_field_when_all_grease_or_none` (tests/tls_analyzer_tests.rs)

### AC-006 (traces to BC-2.07.007 postcondition 7-8)
The MD5 hash is computed over the UTF-8 bytes of the assembled JA3 string, and the returned hash is exactly 32 lowercase hex characters.
- **Test:** `compute_ja3_has_five_fields_and_hex_hash` (proptest, src/analyzer/tls.rs::ja3_property_tests — VP-013 anchor, do not rename)
- **Companion test:** `test_BC_2_07_007_ja3_hash_is_32_lowercase_hex_chars` (tests/tls_analyzer_tests.rs)

### AC-007 (traces to BC-2.07.007 invariant 2)
JA3 hash is order-sensitive: ciphers `[A, B]` and `[B, A]` produce different hashes.
- **Test:** `compute_ja3_is_order_sensitive` (proptest, src/analyzer/tls.rs::ja3_property_tests — VP-013 anchor, do not rename)
- **Companion test:** `test_BC_2_07_007_cipher_order_produces_different_hashes` (tests/tls_analyzer_tests.rs)

### AC-008 (traces to BC-2.07.008 postcondition 1-4)
The JA3S string has exactly 2 commas (3 fields). Field 1 is decimal `version`. Field 2 is decimal `cipher.0` (single selected cipher). Field 3 is GREASE-filtered extension type IDs joined by `-`, or empty string if none.
- **Test:** `compute_ja3s_is_deterministic_and_hex` (proptest, src/analyzer/tls.rs::ja3_property_tests — VP-013 anchor, do not rename)
- **Companion tests:** `test_BC_2_07_008_ja3s_has_exactly_two_commas_three_fields`, `test_BC_2_07_008_ja3s_grease_extension_filtered_from_ext_field` (tests/tls_analyzer_tests.rs)

### AC-009 (traces to BC-2.07.008 postcondition 5-6)
The JA3S MD5 hex digest is 32 lowercase hex characters and is deterministic: same inputs always produce the same 32-char string.
- **Test:** `compute_ja3s_is_deterministic_and_hex` (proptest, src/analyzer/tls.rs::ja3_property_tests — VP-013 anchor, do not rename)
- **Companion test:** `test_BC_2_07_008_ja3s_hash_is_32_lowercase_hex_and_deterministic` (tests/tls_analyzer_tests.rs)

### AC-010 (traces to BC-2.07.008 invariant 1-2)
The JA3S cipher field is a SINGLE value (server selects ONE cipher, not a list). GREASE filtering applies only to extension IDs in JA3S, not to the cipher field.
- **Test:** `compute_ja3s_is_deterministic_and_hex` (proptest, src/analyzer/tls.rs::ja3_property_tests — VP-013 anchor, do not rename)
- **Companion tests:** `test_BC_2_07_008_ja3s_cipher_field_is_single_value_not_filtered`, `test_BC_2_07_008_ja3s_grease_extension_filtered_but_grease_cipher_preserved` (tests/tls_analyzer_tests.rs)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `is_grease_u16` | src/analyzer/tls.rs:50-52 | pure-core |
| `compute_ja3` | src/analyzer/tls.rs:94-151 | pure-core |
| `compute_ja3s` | src/analyzer/tls.rs:153-173 | pure-core |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Cipher list is all GREASE | JA3 cipher field is empty string `""` after filtering |
| EC-002 | Non-canonical GREASE-pattern value `0x0a1a` | Filtered by bitmask; treated as GREASE |
| EC-003 | All 16 canonical RFC 8701 GREASE values in cipher list | All filtered; same JA3 as empty cipher list |
| EC-004 | No extensions, no curves, no point formats | JA3 string = `"771,,,,"` (4 trailing commas for empty fields) |
| EC-005 | version = 0 | JA3 string starts with `"0,"` |
| EC-006 | JA3S with no extensions | JA3S string = `"version,cipher,"` (trailing comma for empty ext field) |
| EC-007 | JA3S all extensions are GREASE | JA3S extension field = `""` |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/analyzer/tls.rs (is_grease_u16, compute_ja3, compute_ja3s) | pure-core | No I/O, no global state access; deterministic functions with no side effects |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| Referenced code (tls.rs lines 50-173) | ~3,000 |
| Test files (tls_analyzer_tests.rs JA3 section) | ~3,000 |
| BC files (3 BCs) | ~3,500 |
| Tool outputs overhead | ~1,500 |
| **Total** | **~13,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~7%** |

## Tasks (MANDATORY)

1. [x] Write tests for AC-001 through AC-010 (test-writer): 17 BC-prefixed deterministic tests added to tests/tls_analyzer_tests.rs + existing proptests reused as VP-013 anchors (no new proptest authoring needed)
2. [ ] Verify Red Gate: all 17 BC-prefixed deterministic tests fail before implementation (existing proptests already in place)
3. [ ] Implement `is_grease_u16` bitmask function per BC-2.07.006 invariant 1
4. [ ] Implement `compute_ja3` with GREASE filtering and 5-field format per BC-2.07.007
5. [ ] Implement `compute_ja3s` with GREASE filtering and 3-field format per BC-2.07.008
6. [ ] Verify existing proptest `compute_ja3_is_grease_invariant` (src/analyzer/tls.rs::ja3_property_tests) passes — do NOT rename (VP-013 anchor)
7. [ ] Verify existing proptest `compute_ja3_is_order_sensitive` (src/analyzer/tls.rs::ja3_property_tests) passes — do NOT rename (VP-013 anchor)
8. [ ] Verify existing proptest `compute_ja3s_is_deterministic_and_hex` (src/analyzer/tls.rs::ja3_property_tests) passes — do NOT rename (VP-013 anchor)
9. [ ] Run all tests; verify all pass
10. [ ] Verify purity boundaries (all three functions are pure-core; no state mutation)
11. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A — first story in E-5 | N/A | N/A | N/A |
| STORY-033 | BC-prefixed test naming convention established; brownfield-formalization delivery pattern (zero src/ changes, tests-only additions) | `test_BC_S_SS_NNN_*` canonical naming for deterministic companion tests; existing inline proptests preserved as VP anchors | Proptest VP anchors must not be renamed — DF-AC-TEST-NAME-SYNC-001 v1 adds BC-prefixed deterministic companions alongside, not replacing, proptests |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| JA3 string must NOT be stored or emitted; only the MD5 hash is surfaced | BC-2.07.007 invariant 4 | Code review: confirm no field stores the pre-hash string |
| GREASE filter bitmask is `(val & 0x0F0F) == 0x0A0A` — NOT the RFC 8701 16-value allowlist | BC-2.07.006 description | Code review: confirm bitmask expression at tls.rs:50-52 |
| Decimal encoding of cipher/extension IDs in JA3 string, not hex or names | BC-2.07.007 invariant 3 | Unit test: confirm "47" not "0x002f" in cipher field |
| JA3S has exactly 3 fields (no curves, no point formats) | BC-2.07.008 description | Unit test: confirm 2 commas in JA3S string |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| tls-parser | 0.12 | `TlsCipherSuiteID`, `TlsExtension` types for JA3 computation |
| md-5 | 0.11 | MD5 digest computation over JA3/JA3S string bytes |
| proptest | 1 | Property-based tests for GREASE invariance and order-sensitivity |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/tls.rs | no change | `is_grease_u16` (line 50-52), `compute_ja3` (lines 94-151), `compute_ja3s` (lines 153-173) already exist; brownfield invariant — ZERO src/ modifications |
| tests/tls_analyzer_tests.rs | modify | 17 BC-prefixed deterministic tests + 2 test helpers (`build_client_hello_no_extensions`, `build_server_hello_with_grease_ext`); +719 lines |

## Changelog

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| v1.0 | 2026-05-21 | story-writer | Initial story decomposition |
| v1.1 | 2026-05-28 | story-writer | DF-AC-TEST-NAME-SYNC-001 v1 sync — AC Test lines bound to actual `fn test_BC_2_07_*` names (17 tests); proptest VP-013 anchors preserved as primary; companion tests added; Tasks 1/6/7/8 updated to reflect existing proptest reuse; FSR updated (src/analyzer/tls.rs no-change, brownfield invariant); Previous Story Intelligence row added (STORY-033); status draft→in-progress |
