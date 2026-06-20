---
document_type: story
story_id: "STORY-053"
epic_id: "E-5"
version: "1.4"
status: completed
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.002.md
input-hash: "f347e1c"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-051, STORY-052]
blocks: [STORY-054]
behavioral_contracts:
  - BC-2.07.002
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 17
target_module: src/analyzer/tls.rs
subsystems: [SS-07]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
nfr:
  - NFR-OBS-003
implementation_strategy: brownfield-formalization
---

<!-- changelog
## v1.3 (2026-06-01)
- Consistency-audit D-001: correct EC-004 expected-behavior to match BC-2.07.002 v1.3. Pre-correction text stated "Anomaly/Likely/High deprecated-protocol finding emitted (see STORY-054)". Post-correction: tls-parser rejects SSL 2.0 ServerHello at the record layer (parse_errors++), handle_server_hello NOT reached, NO finding emitted. Aligns with test_BC_2_07_002_ec004_ssl2_version_parse_behavior_pinned (tls_analyzer_tests.rs:5227). input-hash NOT recomputed (BC file changed prior to this story edit; hash intentionally preserved per governance instruction).

## v1.2 (2026-05-29)
- status: draft → completed. PR #149 squash-merged → develop a044144. Per-story adversarial convergence: 3/3 clean P3-P5, 5 passes total. Wave 17 partial merge — wave-level convergence pending.

## v1.1 (2026-05-29)
- F-W17-S053-P2-001 (MEDIUM): AC-001..007 `**Test:**` citations updated to discriminating BC-prefixed test names per DF-AC-TEST-NAME-SYNC-001 v2. Old citations pointed to under-asserting `test_parse_server_hello` or the wrong-polarity `test_weak_cipher_finding_server` (which exercises a KNOWN cipher and asserts no cipher_counts key — the opposite discriminant from AC-006).
- F-W17-S053-P2-002 (LOW): File Structure Requirements table and Tasks section updated; stale test names `test_parse_server_hello` / `test_weak_cipher_finding_server` replaced with full BC-prefixed test list.
- No input-hash recompute (no cited BC changed).
-->

> **Execute:** `/vsdd-factory:deliver-story STORY-053`

# STORY-053: ServerHello Parsing — JA3S Fingerprinting and Cipher/Version Tracking

## Narrative
- **As a** forensic analyst
- **I want** every TLS ServerHello to be parsed and JA3S-fingerprinted, with the negotiated cipher and version tracked in aggregate counts
- **So that** server-side TLS fingerprints appear in analysis summaries alongside client-side JA3 data, enabling complete TLS session profiling

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.07.002 | Parse Complete TLS ServerHello: JA3S Fingerprint Computed |

## Acceptance Criteria

### AC-001 (traces to BC-2.07.002 postcondition 1)
When a complete TLS ServerHello is processed by `handle_server_hello`, `flow.server_hello_seen` is set to `true`.
- **Test:** `test_BC_2_07_002_server_hello_seen_set_true`

### AC-002 (traces to BC-2.07.002 postcondition 2)
The ServerHello `version` field (u16) is inserted/incremented in `version_counts`. This version count is independent of any prior ClientHello version count on the same flow.
- **Test:** `test_BC_2_07_002_server_version_inserted_in_version_counts`

### AC-003 (traces to BC-2.07.002 postcondition 3)
A JA3S MD5 hex string (32 lowercase hex chars) is computed via `compute_ja3s` from `(version, selected_cipher, extensions)` and inserted/incremented in `ja3s_counts` (bounded at `MAX_MAP_ENTRIES`).
- **Test:** `test_BC_2_07_002_ja3s_hash_computed_and_inserted`; `compute_ja3s_is_deterministic_and_hex` (proptest, secondary)

### AC-004 (traces to BC-2.07.002 postcondition 4)
The cipher name (from `cipher_name(sh.cipher)`) is inserted/incremented in `cipher_counts` (bounded at `MAX_MAP_ENTRIES`).
- **Test:** `test_BC_2_07_002_cipher_name_inserted_in_cipher_counts`

### AC-005 (traces to BC-2.07.002 invariant 1)
JA3S is computed solely from `(version, selected_cipher, extension_ids)` using `compute_ja3s`. GREASE extension IDs are filtered using the same `is_grease_u16` bitmask `(val & 0x0F0F) == 0x0A0A` as JA3.
- **Test:** `test_BC_2_07_002_ja3s_grease_ext_filtered_cipher_not_filtered`

### AC-006 (traces to BC-2.07.002 invariant 2)
Unknown cipher IDs (where `TlsCipherSuite::from_id` returns `None`) are rendered as `"0x{id:04x}"` lowercase hex via `cipher_name`. This hex-formatted string is used as the `cipher_counts` map key.
- **Test:** `test_BC_2_07_002_unknown_cipher_id_renders_as_hex_in_cipher_counts`

### AC-007 (traces to BC-2.07.002 invariant 3)
`version_counts` receives the ServerHello version independently of any prior ClientHello version count. A flow where ClientHello and ServerHello have different version fields increments both version counts.
- **Test:** `test_BC_2_07_002_version_counts_client_and_server_versions_independent`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `handle_server_hello` | src/analyzer/tls.rs:542-604 | effectful-shell (mutates server_hello_seen, version_counts, ja3s_counts, cipher_counts, all_findings) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | ServerHello with no extensions (`sh.ext = None`) | JA3S computed with empty extensions field; `"version,cipher,"` |
| EC-002 | ServerHello with extensions that fail `parse_tls_extensions` | `parse_errors++`; JA3S computed with empty ext field |
| EC-003 | ServerHello cipher = `TLS_NULL_WITH_NULL_NULL` (0x0000) | `is_weak_server_cipher` returns true; `Anomaly/Likely/Medium` finding emitted (see STORY-054 for details) |
| EC-004 | ServerHello version = 0x0200 (SSL 2.0) — PARSE-REJECTION under tls-parser 0.12 | tls-parser rejects the record at the record layer before `handle_server_hello` is reached; `parse_errors` is incremented; `version_counts[0x0200]` remains 0; `ja3s_counts` is not updated; NO deprecated-protocol finding is produced. Pinned by `test_BC_2_07_002_ec004_ssl2_version_parse_behavior_pinned`. (BC-2.07.002 v1.3 EC-004) |
| EC-005 | ServerHello version = 0x0301 (TLS 1.0) | No deprecated-protocol finding; version counted only |
| EC-006 | ServerHello when `ja3s_counts` at MAX_MAP_ENTRIES with a new hash | New hash silently dropped |
| EC-007 | ClientHello has version 0x0301; ServerHello has version 0x0303 | `version_counts` has entry for both 0x0301 and 0x0303 |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/analyzer/tls.rs (handle_server_hello) | effectful-shell | Mutates server_hello_seen, version_counts, ja3s_counts, cipher_counts, all_findings |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| Referenced code (tls.rs lines 542-604) | ~2,500 |
| Test files (tls_analyzer_tests.rs ServerHello tests) | ~2,500 |
| BC files (1 BC) | ~1,500 |
| Tool outputs overhead | ~1,500 |
| **Total** | **~10,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~5%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-007 (test-writer)
2. [ ] Verify Red Gate: all AC tests fail before implementation
3. [ ] Implement `handle_server_hello`: set `server_hello_seen = true`, increment `version_counts`, compute JA3S via `compute_ja3s`, insert `ja3s_counts`, call `cipher_name` and insert `cipher_counts`
4. [ ] Wire `handle_server_hello` into `try_parse_records` (called when ServerHello message type is detected)
5. [ ] Run all tests; verify all pass
6. [ ] Verify purity boundaries: `handle_server_hello` is effectful-shell
7. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-051 | `compute_ja3s` is a pure function returning 32-char lowercase hex string | JA3S has 3 fields only (version, cipher, extension_ids) — no curves or point-formats | JA3S cipher field is a SINGLE value, not a list |
| STORY-052 | `handle_client_hello` sets `client_hello_seen`; `server_hello_seen` (this story) is the partner flag | Both flags needed for `done()` predicate | `version_counts` increments BOTH on ClientHello and ServerHello independently |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `version_counts` is shared between ClientHello and ServerHello — both contribute to the same map | BC-2.07.002 invariant 3 | Code review: confirm shared HashMap is used |
| `cipher_counts` uses `cipher_name(id)` output as key; unknown IDs render as `"0x{id:04x}"` | BC-2.07.002 invariant 2 / BC-2.07.036 | Unit test: AC-006 with an unrecognized cipher ID |
| JA3S has exactly 3 fields (no curves or point-formats); single cipher value | BC-2.07.008 invariant 1-2 | Unit test: count commas in JA3S string |
| `server_hello_seen` is set to `true` once (first ServerHello only; after done, on_data short-circuits) | BC-2.07.002 precondition 4 | Unit test: send two ServerHello bytes; assert server_hello_seen set once |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| tls-parser | 0.12 | `TlsServerHelloContents`, `TlsExtension`, `TlsCipherSuiteID` for ServerHello parsing |
| md-5 | 0.11 | MD5 hex computation (via `compute_ja3s` from STORY-051) |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/tls.rs | modify | `handle_server_hello` (lines 542-604); `cipher_name` function (lines 77-83) |
| tests/tls_analyzer_tests.rs | modify | `test_BC_2_07_002_server_hello_seen_set_true`, `test_BC_2_07_002_server_version_inserted_in_version_counts`, `test_BC_2_07_002_ja3s_hash_computed_and_inserted`, `test_BC_2_07_002_cipher_name_inserted_in_cipher_counts`, `test_BC_2_07_002_ja3s_grease_ext_filtered_cipher_not_filtered`, `test_BC_2_07_002_unknown_cipher_id_renders_as_hex_in_cipher_counts`, `test_BC_2_07_002_version_counts_client_and_server_versions_independent` (AC-001..007) |
