---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified: ["v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.030: Normal Handshake with Strong Cipher Produces Zero Findings

## Description

When a TLS flow completes a ClientHello + ServerHello handshake using a modern strong
cipher (e.g., TLS_AES_256_GCM_SHA384) and a modern TLS version (>= TLS 1.0, i.e.,
version > 0x0300), and the SNI hostname is clean ASCII without C0/DEL bytes, the
analyzer emits zero findings. All counters are incremented normally (handshakes_seen,
version_counts, ja3_counts, ja3s_counts, sni_counts, cipher_counts), but `all_findings`
remains empty.

## Preconditions

1. ClientHello: clean ASCII SNI (arm 1), version > 0x0300, no weak ciphers.
2. ServerHello: selected cipher is not weak (no NULL/ANON/EXPORT/RC4); version > 0x0300.

## Postconditions

1. `all_findings` is empty after processing both hellos.
2. `handshakes_seen == 1`.
3. `sni_counts`, `ja3_counts`, `ja3s_counts`, `version_counts`, `cipher_counts` each
   have exactly one entry.
4. `parse_errors == 0`.

## Invariants

1. A well-formed modern TLS handshake is the baseline zero-finding case.
2. Any deviation from the baseline (weak cipher, deprecated version, anomalous SNI)
   is covered by other BCs; this BC defines the baseline.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TLS 1.3 handshake (version 0x0303 with supported_versions extension) | Zero findings; version_counts[0x0303]++ |
| EC-002 | Handshake on non-standard port (e.g., 8443) | Zero findings (port not consulted by TlsAnalyzer) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ClientHello(version=0x0303, SNI="example.com", ciphers=[AES-256-GCM]) + ServerHello(0x0303, AES-256-GCM) | all_findings is empty; handshakes_seen=1 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Normal strong-cipher handshake produces zero findings | unit: test_normal_request_no_parse_errors; test_normal_handshake_no_findings |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- the zero-finding baseline is the most important correctness property of TLS analysis (no false positives) |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs, C-13) |
| Stories | S-TBD |
| Origin BC | BC-TLS-030 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.009 -- related to (weak client cipher breaks the zero-finding property)
- BC-2.07.011 -- related to (deprecated version breaks it)
- BC-2.07.014 -- related to (anomalous SNI breaks it)

## Architecture Anchors

- `src/analyzer/tls.rs` -- all analysis paths; zero-findings is the absence of all conditions
- `tests/tls_analyzer_tests.rs` -- test_normal_handshake_no_findings

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs` (holistic) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_normal_handshake_no_findings; test_normal_request_no_parse_errors

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates handshakes_seen, all count maps |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
