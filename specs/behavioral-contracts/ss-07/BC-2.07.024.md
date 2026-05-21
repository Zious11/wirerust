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

# BC-2.07.024: Only FIRST ServerName Entry Processed

## Description

`extract_sni` uses `list.first()` to extract exactly one ServerName entry from the
`TlsExtension::SNI(list)`. If the ServerNameList contains multiple entries, only the
first is classified and counted. Subsequent entries are ignored. Multi-name SNI is
rare in practice; this matches the prior behavior.

## Preconditions

1. A ClientHello has an SNI extension with a ServerNameList containing 2+ entries.

## Postconditions

1. Only the first entry's hostname bytes are passed to the classification logic.
2. Only one sni_counts entry is inserted (for the first hostname).
3. Only zero or one finding is emitted (based on the first hostname's classification).
4. Subsequent entries are silently ignored.

## Invariants

1. The multi-name SNI behavior is by design (comment in tls.rs: "we ignore additional
   entries -- multi-name SNI is rare and treating only the first matches the prior
   behavior").
2. The second+ entries are never inspected for anomalies.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First entry is clean ASCII; second has C0 bytes | No finding (second entry never inspected) |
| EC-002 | First entry has non-ASCII; second is clean | Finding from first entry only |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SNI list with ["example.com", "evil\x01.com"] | sni_counts["example.com"]=1; no finding; second entry ignored | happy-path |
| SNI list with ["evil\x01.com", "example.com"] | Finding from "evil\x01.com"; second entry ignored | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Only first SNI entry counted and inspected | unit: test_multi_name_sni_list_only_first_entry_counted |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- first-entry-only SNI processing is part of TLS analysis SNI classification design |
| L2 Domain Invariants | INV-5 (SNI 4-way classification) |
| Architecture Module | SS-07 (analyzer/tls.rs:247-249, C-13) |
| Stories | S-TBD |
| Origin BC | BC-TLS-024 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.022 -- related to (empty list handled at same guard)
- BC-2.07.025 -- related to (non-zero NameType entries also treated as hostnames)

## Architecture Anchors

- `src/analyzer/tls.rs:247-249` -- `list.first()` in extract_sni
- `tests/tls_analyzer_tests.rs` -- test_multi_name_sni_list_only_first_entry_counted

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:247-249` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `.first()` iterator method; comment in source code confirms design intent
- **assertion**: test_multi_name_sni_list_only_first_entry_counted

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates sni_counts, potentially all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
