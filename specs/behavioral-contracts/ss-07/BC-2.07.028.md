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

# BC-2.07.028: sni_counts Cap: Finding Still Fires When Map at Capacity

## Description

`TlsAnalyzer.sni_counts` is capped at `MAX_MAP_ENTRIES = 50,000` entries. When the map
is full, the `TlsAnalyzer::increment` helper silently drops new keys. However, the
SNI anomaly finding emission is logically separate from the count insertion: the
`match sni { ... }` block that emits findings executes AFTER the count insertion. If
the count is dropped (key not inserted), the finding is still pushed to `all_findings`.
Finding emission and count insertion are independent operations.

This is a critical forensic property: an attacker flooding the analyzer with unique SNIs
to exhaust `sni_counts` capacity cannot suppress anomaly findings for subsequently
observed malicious SNIs.

## Preconditions

1. `sni_counts.len() == MAX_MAP_ENTRIES` (map is full).
2. A new ClientHello arrives with an anomalous SNI not already in the map
   (e.g., a non-UTF-8 SNI with a new hex key).

## Postconditions

1. The new SNI key is NOT inserted into `sni_counts` (map full; count silently dropped).
2. The anomaly finding IS pushed to `all_findings` regardless of the count outcome.
3. `sni_counts.len()` remains at `MAX_MAP_ENTRIES`.
4. `all_findings.len()` increases by 1.

## Invariants

1. Finding emission is decoupled from count insertion. The `Self::increment` call
   and the `match sni { ... }` block are sequential, not conditional on each other.
2. `all_findings` in `TlsAnalyzer` has no cap (unlike `TcpReassembler.findings` which
   has `MAX_FINDINGS = 10,000`).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Map at capacity + clean ASCII SNI (arm 1) | Count dropped; NO finding emitted (arm 1 never emits) |
| EC-002 | Map at capacity + anomalous SNI already in map | Count incremented (existing key); finding emitted |
| EC-003 | Map at capacity + anomalous SNI NOT in map | Count dropped; finding emitted (decoupled) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Fill sni_counts to 50,000 entries, then send non-UTF-8 SNI with new key | all_findings has one new finding; sni_counts still at 50,000 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Finding fires when sni_counts is at capacity | unit: test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- finding/count decoupling is a forensic-correctness property of TLS analysis |
| L2 Domain Invariants | INV-5 (SNI 4-way classification), INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:372-376, 402-490, C-13) |
| Stories | S-TBD |
| Origin BC | BC-TLS-028 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.019 -- composes with (non-UTF-8 SNI test vector for this cap test)
- BC-2.07.001 -- depends on (count cap and finding emission both in handle_client_hello)

## Architecture Anchors

- `src/analyzer/tls.rs:372-376` -- TlsAnalyzer::increment helper (cap logic)
- `src/analyzer/tls.rs:402-416` -- sni_counts insertion (before match sni)
- `src/analyzer/tls.rs:424-490` -- SNI finding emission (after, independent of count)
- `tests/tls_analyzer_tests.rs` -- test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:402-490` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if map.len() < limit || map.contains_key(&key)` in increment()
- **assertion**: test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates sni_counts, all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
