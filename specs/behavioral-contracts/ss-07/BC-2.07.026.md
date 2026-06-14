---
document_type: behavioral-contract
level: L3
version: "1.3"
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
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: PG-ARP-F2-007 ss-07 full re-anchor — extract_sni fn 247-270→247-270 — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.026: Trailing Bytes in ServerNameList Tolerated

## Description

If a TLS ClientHello's SNI ServerNameList extension has trailing bytes after the last
valid hostname (i.e., the extension's declared length exceeds the actual content),
`tls_parser`'s SNI extension parsing may leave those bytes unconsumed. The `extract_sni`
function uses `list.first()` on the parsed list, so as long as at least one
`(NameType, hostname)` entry is present, the first hostname is extracted normally.
Trailing bytes do not cause a panic or a parse error at the `extract_sni` level.

## Preconditions

1. The `TlsExtension::SNI(list)` has been successfully parsed by `parse_tls_extensions`.
2. The `list` contains at least one entry.
3. The raw extension bytes had trailing content beyond the parsed entries.

## Postconditions

1. The first hostname entry is processed normally (counted and classified).
2. No `parse_errors` are incremented by `extract_sni` itself.
3. The trailing bytes are silently ignored.

## Invariants

1. Tolerance of trailing bytes is a property of the `tls_parser` crate; the
   TlsAnalyzer behavior depends on it but does not itself validate extension length.
2. If `parse_tls_extensions` fails due to malformed data, `parse_errors` would
   increment (BC-2.07.001 extension parse failure path), but this BC covers the
   success-with-trailing-bytes case.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SNI extension with valid hostname + 10 zero-padding bytes | First hostname extracted; trailing bytes ignored |
| EC-002 | SNI extension where trailing bytes happen to be another hostname-shaped sequence | Only first entry from parsed list is used |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SNI extension bytes with valid "example.com" + 4 trailing 0x00 bytes | sni_counts["example.com"]=1; no error; no extra finding | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Trailing bytes in ServerNameList do not cause error | unit: test_trailing_bytes_in_server_name_list |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md -- trailing-bytes tolerance is part of robust TLS analysis SNI parsing |
| L2 Domain Invariants | INV-5 (SNI 4-way classification) |
| Architecture Module | SS-07 (analyzer/tls.rs:247-270, C-13) |
| Stories | STORY-057 |
| Origin BC | BC-TLS-026 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.024 -- composes with (first-entry-only)
- BC-2.07.029 -- related to (bad record body increments parse_errors -- different path)

## Architecture Anchors

- `src/analyzer/tls.rs:247-270` -- extract_sni function (tls_parser handles the parse)
- `tests/tls_analyzer_tests.rs` -- test_trailing_bytes_in_server_name_list

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:247-270` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_trailing_bytes_in_server_name_list

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates sni_counts, potentially all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
