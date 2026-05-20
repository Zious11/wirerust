---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/mitre.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-10
capability: CAP-10
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

# BC-2.10.005: technique_name Returns Some for Every Seeded ID (15 Total)

## Description

`technique_name(id: &str)` returns `Some(&'static str)` for all 15 technique IDs present in
the `technique_info` static match table. IDs not in the table return `None`. The 15-entry
catalog includes 6 IDs currently emitted by analyzers and 9 staged IDs for future analyzers
(domain-debt O-04).

## Preconditions

1. `technique_name` is called with a string argument.

## Postconditions

1. For each of the 15 seeded IDs, returns `Some(technique_name_string)`.
2. For any other string, returns `None`.
3. The 15 seeded IDs are: T1027, T1036, T1040, T1046, T1071, T1071.001, T1071.004,
   T1083, T1499.002, T1505.003, T1573, T0846, T0855, T0856, T0885.

## Invariants

1. IDs currently emitted (6): T1027, T1036, T1046, T1083, T1499.002, T1505.003.
2. IDs catalogued but never emitted (9): T1040, T1071, T1071.001, T1071.004, T1573,
   T0846, T0855, T0856, T0885. These are staged for future analyzers (O-04).
3. The catalog count is 15 as verified by pass-2 R2 and pass-8. Any count claiming 16 is
   an error (pass-8 correction of a pass-6 claim).
4. The match is exact string equality; no prefix/prefix matching.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | "T9999" (unknown) | None |
| EC-002 | "" (empty string) | None |
| EC-003 | "T1046.999" (unknown sub-technique) | None |
| EC-004 | "garbage" | None |
| EC-005 | "t1027" (lowercase) | None (case-sensitive) |
| EC-006 | "T1071.001" (sub-technique) | Some("Web Protocols") |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| "T1027" | Some("Obfuscated Files or Information") | happy-path |
| "T1036" | Some("Masquerading") | happy-path |
| "T1071.001" | Some("Web Protocols") | happy-path |
| "T0885" | Some("Commonly Used Port") | happy-path |
| "T9999" | None | edge-case |
| "" | None | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All 15 seeded IDs return Some | unit: technique_name_resolves_every_seeded_id |
| VP-TBD | Non-seeded IDs return None | unit: technique_name_returns_none_for_unknown_ids |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 -- technique_name is the primary lookup function of the MITRE catalog |
| L2 Domain Invariants | INV-9 (MITRE technique ID format) |
| Architecture Module | SS-10 (mitre.rs, C-11) |
| Stories | S-TBD |
| Origin BC | BC-MIT-005 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.006 -- related to (None case for unknown IDs)
- BC-2.10.007 -- composes with (technique_tactic uses same lookup)
- BC-2.10.008 -- composes with (all emitted IDs must resolve)

## Architecture Anchors

- `src/mitre.rs:122` -- `pub fn technique_info(id: &str)` function declaration
- `src/mitre.rs:123-155` -- static match table (T1027 at :125, T0885 at :152, `_ => return None` at :153)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:122-155` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **assertion**: technique_name_resolves_every_seeded_id

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (static match) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |
