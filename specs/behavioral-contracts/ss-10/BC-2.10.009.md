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

# BC-2.10.009: MitreTactic is #[non_exhaustive]

## Description

The `MitreTactic` enum is annotated `#[non_exhaustive]`. This means external code matching
on `MitreTactic` must include a wildcard arm (`_ => ...`), making it non-breaking to add
new `MitreTactic` variants in future ATT&CK version updates. This is the standard
extensibility pattern for enums representing evolving external standards.

## Preconditions

1. Code outside the defining crate matches on a `MitreTactic` value.

## Postconditions

1. The compiler requires a wildcard arm in any external `match` expression over `MitreTactic`.
2. Adding a new variant to `MitreTactic` is a non-breaking change for downstream crate consumers.
3. Within the wirerust crate itself, `match` statements can still be exhaustive (the
   `#[non_exhaustive]` restriction only applies to external crates).

## Invariants

1. `#[non_exhaustive]` is applied at the enum definition level (not per-variant).
2. Any future ATT&CK tactic (e.g., "Resource Development" was added in ATT&CK v9) can be
   added as a new variant without breaking downstream consumers that have a wildcard arm.
3. The `all_tactics_in_report_order` function must be manually updated when new variants are added.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Crate-external match without wildcard | Compile error: non-exhaustive pattern |
| EC-002 | Adding new variant (e.g., PreCompromise) | Downstream code with wildcard arm compiles without changes |
| EC-003 | Within wirerust crate: match on MitreTactic | Exhaustive match allowed; no wildcard required |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| External crate: match tactic { Reconnaissance => ..., _ => ... } | Compiles | happy-path |
| External crate: match tactic { Reconnaissance => ... } (no wildcard) | Compile error | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | MitreTactic has #[non_exhaustive] attribute | code: grep src/mitre.rs for non_exhaustive |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 -- #[non_exhaustive] is the extensibility contract for the MITRE tactic enum |
| L2 Domain Invariants | INV-9 (MITRE technique ID format) |
| Architecture Module | SS-10 (mitre.rs, C-11) |
| Stories | S-TBD |
| Origin BC | BC-MIT-009 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.001 -- related to (Display impl must be updated when new variants are added)
- BC-2.10.003 -- related to (all_tactics_in_report_order slice must be updated when new variants are added)

## Architecture Anchors

- `src/mitre.rs:46` -- `#[non_exhaustive]` on MitreTactic enum

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:46` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: Rust #[non_exhaustive] attribute enforced by the compiler

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure (attribute-only; no runtime behavior) |

## Refactoring Notes

No refactoring needed. This is a deliberately simple attribute-level contract that
provides forward compatibility without runtime overhead.
