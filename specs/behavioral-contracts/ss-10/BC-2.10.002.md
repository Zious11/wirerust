---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.002: ICS Tactics Render Unprefixed

## Description

The two ICS-unique `MitreTactic` variants render WITHOUT an "ICS:" prefix or any other
matrix qualifier: `IcsInhibitResponseFunction` => "Inhibit Response Function" and
`IcsImpairProcessControl` => "Impair Process Control". This mirrors the MITRE ICS ATT&CK
display convention for tactic names (which are used directly in grouped reports alongside
Enterprise tactic names). The design intention per mitre.rs is to merge Enterprise and ICS
findings into a single tactic-grouped report with one section per tactic name.

## Preconditions

1. A `MitreTactic::IcsInhibitResponseFunction` or `MitreTactic::IcsImpairProcessControl`
   value is formatted via Display.

## Postconditions

1. `IcsInhibitResponseFunction` displays as "Inhibit Response Function".
2. `IcsImpairProcessControl` displays as "Impair Process Control".
3. No "ICS:" or other matrix prefix is prepended.

## Invariants

1. The ICS tactic names appear in `all_tactics_in_report_order()` AFTER all 14 Enterprise tactics.
2. The Display strings are the MITRE ICS ATT&CK canonical tactic names (unprefixed).
3. A consumer matching on tactic names must account for these ICS names appearing in the
   same list as Enterprise tactic names; there is no structural separation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | IcsInhibitResponseFunction formatted | "Inhibit Response Function" (no prefix) |
| EC-002 | IcsImpairProcessControl formatted | "Impair Process Control" (no prefix) |
| EC-003 | both ICS tactics in report order | appear as positions [14] and [15] (0-indexed) after all 14 Enterprise tactics |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| format!("{}", MitreTactic::IcsInhibitResponseFunction) | "Inhibit Response Function" | happy-path |
| format!("{}", MitreTactic::IcsImpairProcessControl) | "Impair Process Control" | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | ICS tactic names render without prefix | unit: assert_eq on both ICS variants |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 -- ICS tactic Display is part of the MITRE mapping capability's output |
| L2 Domain Invariants | INV-9 (MITRE technique ID format) |
| Architecture Module | SS-10 (mitre.rs, C-16) |
| Stories | S-TBD |
| Origin BC | BC-MIT-002 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.001 -- related to (Enterprise tactic Display uses same impl)
- BC-2.10.003 -- composes with (ICS tactics appear last in all_tactics_in_report_order)

## Architecture Anchors

- `src/mitre.rs:85-87` -- ICS tactic Display arms

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:85-87` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: hardcoded &'static str for ICS variants

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed.
