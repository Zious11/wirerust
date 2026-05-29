---
document_type: story
story_id: "STORY-052"
epic_id: "E-5"
version: "1.4"
status: completed
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.001.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.003.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.032.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.034.md
input-hash: "39b997a"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-051]
blocks: [STORY-053, STORY-054, STORY-055, STORY-058]
behavioral_contracts:
  - BC-2.07.001
  - BC-2.07.003
  - BC-2.07.032
  - BC-2.07.034
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 16
target_module: src/analyzer/tls.rs
subsystems: [SS-07]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **Execute:** `/vsdd-factory:deliver-story STORY-052`

# STORY-052: ClientHello Parsing — Handshake Counting, Version/JA3 Tracking, and Done Short-Circuit

## Narrative
- **As a** forensic analyst
- **I want** every TLS ClientHello to be parsed, counted, version-tracked, and JA3-fingerprinted — and for the analyzer to short-circuit gracefully once both hellos have been processed
- **So that** all downstream analysis (SNI classification, cipher/protocol anomaly detection) operates on correctly structured state and no memory is wasted on post-handshake application data

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.07.001 | Parse Complete TLS ClientHello: Version, Ciphers, Extensions, SNI, JA3 |
| BC-2.07.003 | After Both Hellos Seen, Subsequent Records Are Silently Skipped |
| BC-2.07.032 | TLS 1.3 ClientHello legacy_version Recorded as 0x0303 |
| BC-2.07.034 | After Both Hellos Seen, on_data Short-Circuits |

## Acceptance Criteria

### AC-001 (traces to BC-2.07.001 postcondition 1)
When a complete TLS ClientHello is processed by `handle_client_hello`, `handshakes_seen` is incremented by exactly 1.
- **Test:** `test_parse_client_hello`

### AC-002 (traces to BC-2.07.001 postcondition 2)
The ClientHello `version` field value (u16) is inserted/incremented in `version_counts` (bounded at `MAX_MAP_ENTRIES = 50,000`).
- **Test:** `test_parse_client_hello`; `test_BC_2_07_001_inv2_version_counts_bounded_at_max_map_entries`

### AC-003 (traces to BC-2.07.001 postcondition 3)
A JA3 MD5 hex string (32 lowercase hex chars) is computed via `compute_ja3` and inserted/incremented in `ja3_counts` (bounded at `MAX_MAP_ENTRIES`).
- **Test:** `test_parse_client_hello`; `compute_ja3_has_five_fields_and_hex_hash` (proptest)

### AC-004 (traces to BC-2.07.001 postcondition 4)
If the ClientHello extensions include a non-empty SNI list, the first hostname is classified and its string key is inserted/incremented in `sni_counts`.
- **Test:** `test_parse_client_hello`

### AC-005 (traces to BC-2.07.001 postcondition 8)
The consumed ClientHello record bytes are drained from `client_buf` after processing.
- **Test:** `test_parse_client_hello` (assert buffer length decreases)

### AC-006 (traces to BC-2.07.001 invariant 1)
`handshakes_seen` increments exactly once per ClientHello, regardless of how many SNI entries or weak ciphers are present in that ClientHello.
- **Test:** `test_parse_client_hello`

### AC-007 (traces to BC-2.07.001 invariant 2)
All counter maps (`ja3_counts`, `version_counts`, `sni_counts`) are bounded at `MAX_MAP_ENTRIES = 50,000`. New keys are silently dropped when the map is full; no error or finding is emitted for the drop.
- **Test:** `test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity`; `test_BC_2_07_001_inv2_version_counts_bounded_at_max_map_entries`; `test_BC_2_07_001_inv2_ja3_counts_bounded_at_max_map_entries`

### AC-008 (traces to BC-2.07.003 postcondition 1-5)
After both `client_hello_seen` and `server_hello_seen` are true (`TlsFlowState::done() == true`), any further `on_data` call returns immediately. No bytes are appended to buffers, no counters are incremented, no findings are emitted, and the flow state remains in the `flows` HashMap.
- **Test:** `test_stop_after_handshake`

### AC-009 (traces to BC-2.07.003 invariant 1-2)
The `done()` check is the first operation in `on_data` after the HashMap lookup. Once `done()` is true, it is permanent — no future call can re-enter processing for that flow.
- **Test:** `test_stop_after_handshake` (send retransmitted ClientHello after done; assert `handshakes_seen` unchanged)

### AC-010 (traces to BC-2.07.032 postcondition 1-3)
A TLS 1.3 ClientHello with `legacy_version = 0x0303` records `version_counts[0x0303]++` and computes JA3 with `version = 771`. No deprecated-protocol finding is emitted (0x0303 > 0x0300).
- **Test:** `test_tls13_pcap_version_and_ja3` (integration)

### AC-011 (traces to BC-2.07.032 invariant 1-2)
`TlsAnalyzer` uses only `ch.version.0` (the `legacy_version` field from tls-parser) for version counting and JA3 — it does NOT inspect the `supported_versions` extension to determine the actual negotiated TLS version.
- **Test:** `test_BC_2_07_032_inv1_supported_versions_not_inspected` (unit, primary — constructs a synthetic ClientHello with `legacy_version=0x0303` and `supported_versions=[0x0304]`; asserts `version_counts` contains `0x0303` and does NOT contain `0x0304`, directly pinning the invariant); `test_tls13_pcap_version_and_ja3` (integration, companion — confirms the real-world TLS 1.3 pcap path records `0x0303`, but is vacuous for the isolation invariant because both paths produce `0x0303` for a real capture)

### AC-012 (traces to BC-2.07.034 postcondition 1-3)
When `on_data` is called for a done flow, it returns without modifying any state, without appending bytes to any buffer, and without calling `try_parse_records`. A 1 MB burst of application data after both hellos leaves all counters at their post-handshake values.
- **Test:** `test_stop_after_handshake`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `handle_client_hello` | src/analyzer/tls.rs:379-540 | effectful-shell (mutates handshakes_seen, version_counts, ja3_counts, sni_counts, all_findings) |
| `TlsFlowState::done()` | src/analyzer/tls.rs:290-293 | pure-core (reads two booleans) |
| `on_data` (done-check) | src/analyzer/tls.rs:718-724 | effectful-shell (reads flows HashMap; early return) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | ClientHello with no extensions (`ch.ext = None`) | JA3 computed with empty ext/curves/pf fields; no SNI counting |
| EC-002 | ClientHello with extensions that fail `parse_tls_extensions` | `parse_errors++`; JA3 computed with empty ext fields; SNI not extracted |
| EC-003 | ClientHello version = 0x0303 (TLS 1.2) | Version counted; no deprecated-protocol finding |
| EC-004 | ClientHello when `ja3_counts` is at MAX_MAP_ENTRIES with a new JA3 hash | New hash silently dropped; count unchanged |
| EC-005 | Large burst (1 MB) of app data after both hellos | All bytes discarded; state unchanged |
| EC-006 | Empty slice `on_data([], dir)` after hellos done | Returns immediately; no effect |
| EC-007 | Flow not in `self.flows` map | `is_some_and` returns false; `done = false`; flow created fresh |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/analyzer/tls.rs (handle_client_hello) | effectful-shell | Mutates 4 count maps and all_findings |
| src/analyzer/tls.rs (TlsFlowState::done()) | pure-core | Reads two boolean fields; no mutation |
| src/analyzer/tls.rs (on_data done-path) | pure-core (done path only) | Reads flows HashMap; returns immediately without mutation |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,500 |
| Referenced code (tls.rs lines 280-540, 718-724) | ~6,000 |
| Test files (tls_analyzer_tests.rs ClientHello tests) | ~4,000 |
| BC files (4 BCs) | ~5,000 |
| Tool outputs overhead | ~2,000 |
| **Total** | **~20,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~10%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-012 (test-writer)
2. [ ] Verify Red Gate: all AC tests fail before implementation
3. [ ] Implement `TlsFlowState::done()` predicate per BC-2.07.003 invariant 1 (reads `client_hello_seen && server_hello_seen`)
4. [ ] Implement `on_data` done-check at entry point per BC-2.07.034 precondition 1 (uses `is_some_and`)
5. [ ] Implement `handle_client_hello`: increment `handshakes_seen`, insert `version_counts`, compute JA3 via `compute_ja3`, insert `ja3_counts`, call SNI extraction, drain `client_buf`
6. [ ] Wire the done-check short-circuit: return immediately if `done() == true`
7. [ ] Write integration test for TLS 1.3 `legacy_version = 0x0303` per AC-010
8. [ ] Run all tests; verify all pass
9. [ ] Verify purity boundaries: `done()` is pure; `handle_client_hello` is effectful-shell
10. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-051 | `compute_ja3` and `compute_ja3s` are pure functions; use `is_grease_u16` bitmask `(v & 0x0F0F) == 0x0A0A` | GREASE filtering is bitmask-based, not allowlist | `compute_ja3` returns `(md5_hex, ja3_string)` pair; only md5_hex is stored |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `done()` check is the FIRST operation in `on_data`, before any mutable borrow | BC-2.07.034 invariant 1 | Code review: confirm position of done-check in on_data |
| `handshakes_seen` increments exactly once per ClientHello (not once per SNI or cipher) | BC-2.07.001 invariant 1 | Unit test: AC-006 with multiple weak ciphers |
| Raw SNI bytes are not escaped at the TlsAnalyzer layer | BC-2.07.001 invariant 3 (ADR 0003 / INV-4) | Code review: no escape call in handle_client_hello |
| `MAX_MAP_ENTRIES = 50,000` is the bound for ALL count maps | BC-2.07.001 invariant 2 | Code review: confirm constant and usage in increment() helper |
| TLS 1.3: only `ch.version.0` is used for version tracking; `supported_versions` extension is ignored | BC-2.07.032 invariant 1 | Code review: confirm no supported_versions extension parsing |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| tls-parser | 0.12 | `TlsClientHelloContents`, `TlsExtension`, `TlsCipherSuiteID` for ClientHello parsing |
| md-5 | 0.11 | MD5 hex computation (via `compute_ja3` from STORY-051) |
| Rust std | 2024 edition (stable) | `HashMap::or_insert_with`, `Vec::drain`, `str::from_utf8` |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/tls.rs | modify | `handle_client_hello` (lines 379-540), `TlsFlowState::done()` (lines 280-293), `on_data` done-check (lines 718-724) |
| tests/tls_analyzer_tests.rs | modify | ClientHello parsing tests (AC-001..009), stop-after-handshake test (AC-008, AC-012) |
| tests/tls_integration_tests.rs | modify | TLS 1.3 pcap integration test (AC-010, AC-011) |

## Changelog

| Version | Date | Notes |
|---------|------|-------|
| v1.0 | 2026-05-21 | Initial story decomposition |
| v1.1 | 2026-05-21 | Pass-1 adversarial convergence |
| v1.2 | 2026-05-21 | Pass-2 adversarial convergence; test citations added |
| v1.3 | 2026-05-28 | Pass-2 retroactive remediation (F-W16-S052-P2-001): AC-011 primary test changed from vacuous integration test to discriminating unit test `test_BC_2_07_032_inv1_supported_versions_not_inspected` (tests/tls_analyzer_tests.rs:2677); integration test `test_tls13_pcap_version_and_ja3` demoted to companion citation. BC-2.07.032 bumped to v1.3 and BC-2.07.001 bumped to v1.3 by PO this burst — input-hash recomputed: `5847cf5` → `09f5faa` (sha256 over sorted cited-BC files, first 7 chars). |
| v1.4 | 2026-05-28 | Wave-16 Pass-4 sibling-sweep input-hash propagation (DF-SIBLING-SWEEP-001): BC-2.07.001 bumped v1.3→v1.4 by PO this burst — input-hash recomputed: `09f5faa` → `39b997a` (sha256 over sorted cited-BC files BC-2.07.001/003/032/034, first 7 chars). No AC citation changes required. |
