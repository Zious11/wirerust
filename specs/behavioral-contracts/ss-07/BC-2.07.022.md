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

# BC-2.07.022: Empty SNI ServerNameList: No Count, No Finding, Handshake Counted

## Description

When a ClientHello contains an SNI extension but the ServerNameList is empty (the
extension has zero entries), `extract_sni` returns `None` (the `list.first()` call
returns `None`). No SNI is counted in `sni_counts`, no finding is emitted. The
handshake is still counted (via BC-2.07.001's `handshakes_seen++`). This handles the
degenerate case of an SNI extension header present but with an empty name list.

## Preconditions

1. A ClientHello contains a `TlsExtension::SNI(list)` extension.
2. `list.first()` returns `None` (empty list).

## Postconditions

1. `extract_sni` returns `None`.
2. `sni_counts` is unchanged (no entry added).
3. No finding is pushed to `all_findings`.
4. `handshakes_seen` is still incremented (by BC-2.07.001, not by SNI handling).

## Invariants

1. An SNI extension with empty ServerNameList is treated identically to a ClientHello
   with no SNI extension at all, from a finding/count perspective.
2. The `None` return from `extract_sni` short-circuits the entire SNI handling block
   (no match on `SniValue` variants is reached).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No SNI extension at all | extract_sni returns None (same outcome) |
| EC-002 | SNI extension with empty list + weak cipher | No SNI finding; weak-cipher finding still fires |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ClientHello with SNI extension containing 0 entries | sni_counts empty; no finding; handshakes_seen=1 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Empty SNI list produces no count and no finding | unit: test_sni_extension_with_empty_hostname_list |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- empty SNI list handling is an SNI edge case in TLS analysis |
| L2 Domain Invariants | INV-5 (SNI 4-way classification -- None return is outside the 4 arms) |
| Architecture Module | SS-07 (analyzer/tls.rs:247-249, C-13) |
| Stories | STORY-057 |
| Origin BC | BC-TLS-022 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.023 -- related to (empty hostname bytes is different from empty list)
- BC-2.07.024 -- related to (only first entry is processed)

## Architecture Anchors

- `src/analyzer/tls.rs:247-249` -- `list.first()` guard in extract_sni
- `tests/tls_analyzer_tests.rs` -- test_sni_extension_with_empty_hostname_list

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:247-249` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if let Some((_, hostname)) = list.first()`
- **assertion**: test_sni_extension_with_empty_hostname_list

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads only (no mutation when None) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self when Some) |
| **Overall classification** | pure (for None path) |
