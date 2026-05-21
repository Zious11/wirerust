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

# BC-2.10.006: technique_name Returns None for Unknown IDs

## Description

`technique_name(id)` returns `None` for any string that is not in the 15-entry static match
table. This includes fabricated IDs, IDs with wrong formatting, IDs with extra whitespace,
and lowercase forms of valid IDs. The function never panics or returns a default string
for unknown IDs.

## Preconditions

1. `technique_name` is called with any string argument.
2. The argument is not one of the 15 seeded IDs.

## Postconditions

1. Returns `None`.
2. No error, no panic, no default string.

## Invariants

1. The match is exact string equality (case-sensitive, no trimming).
2. An ID not in the 15-entry table always returns None.
3. This applies to valid-looking IDs for real ATT&CK techniques that are not yet seeded
   (e.g., "T1059" is a real technique but not in the catalog; returns None).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | "T9999" (non-existent) | None |
| EC-002 | "" (empty string) | None |
| EC-003 | "t1027" (lowercase) | None (case-sensitive) |
| EC-004 | " T1027" (leading space) | None (no trim) |
| EC-005 | "T1059" (real ATT&CK but not seeded) | None |
| EC-006 | "T1046.001" (sub-technique of seeded parent; not itself seeded) | None |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| "T9999" | None | error-path |
| "" | None | error-path |
| "t1027" | None | error-path |
| "T1059" | None | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-007 | Unknown ID returns None | unit: technique_name_returns_none_for_unknown_ids |
| VP-007 | Lowercase form of known ID returns None | unit: assert None for "t1027" |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 -- the None path of technique_name is the explicit boundary of the MITRE catalog |
| L2 Domain Invariants | INV-9 (MITRE technique ID format) |
| Architecture Module | SS-10 (mitre.rs, C-16) |
| Stories | S-TBD |
| Origin BC | BC-MIT-006 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.005 -- related to (Some path; this BC is the None complement)
- BC-2.10.007 -- composes with (technique_tactic also returns None for unknown IDs via same lookup)

## Architecture Anchors

- `src/mitre.rs:153` -- `_ => return None` wildcard arm

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:153` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: wildcard match arm in technique_info

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
