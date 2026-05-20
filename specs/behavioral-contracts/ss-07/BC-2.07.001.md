---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/tls.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.001: Parse Complete TLS ClientHello: Version, Ciphers, Extensions, SNI, JA3

## Description

When a complete TLS ClientHello record arrives on a flow, `TlsAnalyzer` extracts the
protocol version, cipher suite list, and extensions. From extensions it derives the SNI
hostname (classified via `extract_sni`) and computes the JA3 MD5 fingerprint. The JA3
hash is counted in `ja3_counts`; the SNI is counted in `sni_counts`; the version is
counted in `version_counts`. `handshakes_seen` is incremented once per ClientHello
processed.

## Preconditions

1. `TlsAnalyzer::on_data` has been called with bytes for the client direction.
2. The accumulated client-direction buffer contains a complete TLS record with
   `record_type == 0x16` (Handshake) and a complete ClientHello message.
3. `payload_len <= MAX_RECORD_PAYLOAD` (18,432 bytes); oversized records are rejected
   before parsing (see BC-2.07.004).
4. The flow has not yet been marked `done()` (both hellos already seen).

## Postconditions

1. `handshakes_seen` is incremented by 1.
2. The ClientHello `version` field value (u16) is inserted/incremented in
   `version_counts` (bounded by `MAX_MAP_ENTRIES = 50,000`).
3. A JA3 MD5 hex string (32 lowercase hex chars) is computed via `compute_ja3` and
   inserted/incremented in `ja3_counts` (bounded by `MAX_MAP_ENTRIES`).
4. If the ClientHello extensions include a non-empty SNI list, the first hostname is
   classified and its string key is inserted/incremented in `sni_counts`.
5. If the classified SNI is not `SniValue::Ascii`, a Finding is pushed to
   `all_findings` (see BC-2.07.014..019).
6. If the ClientHello cipher list contains any weak cipher (NULL/ANON/EXPORT), a
   Finding is pushed to `all_findings` (see BC-2.07.009).
7. If the version is <= 0x0300 (SSL 3.0), a Finding is pushed to `all_findings`
   (see BC-2.07.011).
8. The consumed record bytes are drained from `client_buf`.

## Invariants

1. `handshakes_seen` increments exactly once per ClientHello, regardless of how
   many SNI entries or weak ciphers are present.
2. All counter maps are bounded at `MAX_MAP_ENTRIES`; new keys are silently dropped
   when the map is full.
3. Raw SNI bytes are not escaped at this layer (ADR 0003 / INV-4).
4. GREASE cipher/extension/group values are filtered before JA3 computation (INV-2
   of JA3 spec; see BC-2.07.006).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ClientHello with no extensions (ch.ext = None) | JA3 computed with empty strings for ext/curves/pf; no SNI counting |
| EC-002 | ClientHello with extensions that fail `parse_tls_extensions` | `parse_errors++`; JA3 computed with empty ext fields; SNI not extracted |
| EC-003 | ClientHello with SNI extension but empty ServerNameList | No SNI count; no finding; handshake still counted (see BC-2.07.022) |
| EC-004 | ClientHello with all GREASE ciphers | JA3 cipher field is empty string after filtering; no weak-cipher finding |
| EC-005 | ClientHello version = 0x0303 (TLS 1.2) | Version counted; no deprecated-protocol finding |
| EC-006 | ClientHello when `ja3_counts` is at MAX_MAP_ENTRIES with a new JA3 hash | New hash silently dropped; count unchanged |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Full ClientHello with "example.com" SNI, TLS 1.2 version, strong ciphers | handshakes_seen=1; sni_counts["example.com"]=1; ja3_counts has one entry; no findings | happy-path |
| ClientHello with GREASE cipher 0x0a0a in cipher list alongside one real cipher | JA3 string excludes 0x0a0a; same JA3 as without GREASE cipher | happy-path |
| ClientHello with null extensions field | ja3 computed with 4 empty comma-separated fields after version; sni_counts empty | edge-case |
| ClientHello with version 0x0300 (SSL 3.0) | findings has one deprecated-protocol finding | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | handshakes_seen increments exactly once per ClientHello | unit: test_parse_client_hello |
| VP-TBD | JA3 is 32 lowercase hex chars | proptest: compute_ja3_has_five_fields_and_hex_hash |
| VP-TBD | GREASE values do not change JA3 hash | proptest: compute_ja3_is_grease_invariant |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- this BC is the primary ClientHello processing entry point for all TLS analysis |
| L2 Domain Invariants | INV-5 (SNI 4-way classification), INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:379-540, C-13) |
| Stories | S-TBD |
| Origin BC | BC-TLS-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.002 -- related to (ServerHello counterpart)
- BC-2.07.003 -- related to (short-circuit after both hellos seen)
- BC-2.07.006 -- composes with (GREASE filtering in JA3)
- BC-2.07.007 -- composes with (JA3 string format)
- BC-2.07.009 -- composes with (weak cipher detection)
- BC-2.07.011 -- composes with (deprecated version detection)

## Architecture Anchors

- `src/analyzer/tls.rs:379-540` -- `handle_client_hello` implementation
- `src/analyzer/tls.rs:379-387` -- `handshakes_seen` increment and version count
- `src/analyzer/tls.rs:493` -- JA3 computation and count
- `src/analyzer/tls.rs:402-490` -- SNI extraction and finding emission
- `tests/tls_analyzer_tests.rs` -- test_parse_client_hello

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:379-540` (`handle_client_hello`) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: TlsClientHelloContents struct fields from tls_parser crate
- **guard clause**: `ch.ext` None/Some branch; `parse_tls_extensions` Ok/Err branch
- **assertion**: test_parse_client_hello exercises the full path

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates handshakes_seen, version_counts, ja3_counts, sni_counts, all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
