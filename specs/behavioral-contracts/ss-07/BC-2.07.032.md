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

# BC-2.07.032: TLS 1.3 ClientHello legacy_version Recorded as 0x0303

## Description

TLS 1.3 ClientHellos use `legacy_version = 0x0303` (TLS 1.2) in the record header for
backwards compatibility, with the actual TLS 1.3 version negotiated via the
`supported_versions` extension. The `tls_parser` crate exposes `ch.version` as the
`legacy_version` field. `TlsAnalyzer` records `ch.version.0` in `version_counts` and
uses it for JA3 computation. For TLS 1.3 ClientHellos, this is always 0x0303 (771
decimal). The JA3 spec uses this legacy version field, not the supported_versions
extension.

## Preconditions

1. A TLS 1.3 ClientHello is received where `legacy_version == 0x0303`.

## Postconditions

1. `version_counts[0x0303]` is incremented (771 decimal key in JSON).
2. JA3 is computed with `version = 0x0303 = 771`.
3. No deprecated-protocol finding is emitted (0x0303 > 0x0300).

## Invariants

1. `TlsAnalyzer` does not inspect the `supported_versions` extension for the version
   it stores; only `ch.version.0` is used.
2. This is intentional per the JA3 specification (version is the legacy_version field).
3. TLS 1.3 and TLS 1.2 ClientHellos both have `legacy_version = 0x0303` and are
   indistinguishable in `version_counts`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Genuine TLS 1.2 ClientHello | Also records 0x0303; indistinguishable from TLS 1.3 at version level |
| EC-002 | TLS 1.3 with supported_versions=[TLS 1.3] | version_counts[0x0303]++; supported_versions not separately tracked |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| TLS 1.3 ClientHello (legacy_version=0x0303) captured from real traffic | version_counts["771"]=1; ja3 starts with "771,"; no deprecated-protocol finding | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | TLS 1.3 legacy_version 0x0303 recorded in version_counts | integration: test_tls13_pcap_version_and_ja3 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- TLS 1.3 legacy_version handling is part of TLS analysis version recording |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:383-387, C-16) |
| Stories | S-TBD |
| Origin BC | BC-TLS-032 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.001 -- depends on (version recorded during ClientHello handling)
- BC-2.07.007 -- composes with (JA3 version field uses legacy_version)
- BC-2.07.011 -- related to (deprecated-protocol threshold is <= 0x0300; TLS 1.3 not affected)

## Architecture Anchors

- `src/analyzer/tls.rs:383-387` -- `version = ch.version.0; version_counts++`
- `tests/tls_integration_tests.rs` -- test_tls13_pcap_version_and_ja3

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:383-387` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: `ch.version.0` is the legacy_version field from tls_parser
- **assertion**: test_tls13_pcap_version_and_ja3

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates version_counts |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
