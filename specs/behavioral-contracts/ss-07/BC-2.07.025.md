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

# BC-2.07.025: Non-Zero NameType Entries Treated as Hostnames

## Description

The TLS SNI extension's `ServerNameList` is a list of `(NameType, hostname)` pairs.
RFC 6066 defines only NameType=0 (host_name). The `tls_parser` crate's
`TlsExtension::SNI` list uses `(u8, &[u8])` tuples where the first field is the
NameType byte. The `extract_sni` function destructures with `let Some((_, hostname)) =
list.first()`, discarding the NameType with `_`. Any NameType value (0 or non-zero) is
accepted; only the hostname bytes are used.

## Preconditions

1. A ClientHello SNI extension has a ServerNameList where the first entry has a
   non-zero NameType (e.g., NameType=1, which is "future use" per RFC 6066).

## Postconditions

1. The hostname bytes from the first entry (regardless of NameType) are passed to
   the 4-way classification.
2. Behavior is identical to NameType=0 processing.
3. No finding is emitted solely because of the non-zero NameType.

## Invariants

1. NameType is discarded (`_` in the destructure pattern).
2. The current implementation does not validate that NameType==0.
3. This is the `tls_parser` library's behavior: it passes through non-zero NameType
   entries without filtering.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First entry has NameType=1, hostname="example.com" | Arm 1; no finding; counted |
| EC-002 | First entry has NameType=255 + non-ASCII hostname | Arm 3; finding emitted (hostname classification only) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SNI entry with NameType=1 and hostname b"example.com" | sni_counts["example.com"]=1; no finding | happy-path |
| SNI entry with NameType=1 and non-ASCII hostname | Finding per arm 3 logic | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Non-zero NameType treated as hostname without rejection | unit: test_non_zero_name_type_sni_entry; test_non_zero_name_type_with_valid_first_entry |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- NameType handling is an SNI edge case in TLS analysis |
| L2 Domain Invariants | INV-5 (SNI 4-way classification) |
| Architecture Module | SS-07 (analyzer/tls.rs:247-249, C-13) |
| Stories | S-TBD |
| Origin BC | BC-TLS-025 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.024 -- composes with (first-entry-only; this BC clarifies NameType handling)

## Architecture Anchors

- `src/analyzer/tls.rs:247-249` -- `let Some((_, hostname)) = list.first()` (NameType discarded)
- `tests/tls_analyzer_tests.rs` -- test_non_zero_name_type_sni_entry

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:247-249` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: `_` wildcard in destructure pattern; NameType is ignored
- **assertion**: test_non_zero_name_type_sni_entry

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates sni_counts, potentially all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
