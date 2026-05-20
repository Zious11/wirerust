---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/findings.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-09
capability: CAP-09
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

# BC-2.09.004: Confidence Display: Uppercase Tokens

## Description

`Confidence` implements `fmt::Display` with uppercase string representations: `High` => "HIGH",
`Medium` => "MEDIUM", `Low` => "LOW". These tokens appear in `Finding::Display` output and
in the terminal reporter's colorized output. The uppercase convention is part of the wire-
visible output contract.

## Preconditions

1. A `Confidence` value is formatted via Display.

## Postconditions

1. `Confidence::High` displays as "HIGH".
2. `Confidence::Medium` displays as "MEDIUM".
3. `Confidence::Low` displays as "LOW".
4. No other strings are produced.

## Invariants

1. The strings are hardcoded in the match arms.
2. `Confidence` is `#[non_exhaustive]`; future variants must add Display arms.
3. `Confidence` is also used as a sort key by the terminal reporter (the sort order is
   separate from the Display string; Display is presentation only).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Confidence::High | "HIGH" |
| EC-002 | Confidence::Medium | "MEDIUM" |
| EC-003 | Confidence::Low | "LOW" |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| format!("{}", Confidence::High) | "HIGH" | happy-path |
| format!("{}", Confidence::Medium) | "MEDIUM" | happy-path |
| format!("{}", Confidence::Low) | "LOW" | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All Confidence variants produce expected uppercase strings | unit: exhaustive assert on each variant |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-09 ("Forensic finding emission") per capabilities.md §CAP-09 |
| Capability Anchor Justification | CAP-09 ("Forensic finding emission") per capabilities.md §CAP-09 -- Confidence display is part of the Finding output vocabulary defined in CAP-09 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-09 (findings.rs, C-14) |
| Stories | S-TBD |
| Origin BC | BC-FND-004 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.09.002 -- composes with (Confidence Display is used in Finding Display template)

## Architecture Anchors

- `src/findings.rs:68-76` -- impl fmt::Display for Confidence

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/findings.rs:68-76` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: hardcoded string literals in match arms

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
