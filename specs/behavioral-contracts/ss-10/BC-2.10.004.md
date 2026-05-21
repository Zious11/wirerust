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

# BC-2.10.004: all_tactics_in_report_order Contains Every Variant Exactly Once

## Description

The static slice returned by `all_tactics_in_report_order()` contains all 16 `MitreTactic`
variants with no duplicates and no omissions. This no-duplicate, no-omission guarantee
ensures that a tactic-grouped report iterates over every possible bucket exactly once,
producing a complete and non-redundant output. Because `MitreTactic` is `#[non_exhaustive]`,
the completeness guarantee is by convention (manual inspection) rather than compiler
enforcement.

## Preconditions

1. `all_tactics_in_report_order()` is called.

## Postconditions

1. Each of the 16 `MitreTactic` variants appears in the slice exactly once.
2. No variant appears twice.
3. No variant is omitted.

## Invariants

1. The variant count is 16 (14 Enterprise + 2 ICS).
2. `#[non_exhaustive]` means the compiler cannot enforce completeness; human review and
   the duplicate-check test are the enforcement mechanism.
3. If a new variant is added to `MitreTactic`, the static slice in `all_tactics_in_report_order`
   must be updated manually; CI does not catch this automatically.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Collect slice into HashSet<MitreTactic> | Set size == 16 (no duplicates) |
| EC-002 | Iterate all_tactics and count occurrences of each variant | Every variant count == 1 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| HashSet from all_tactics_in_report_order() | len == 16 | happy-path |
| Count IcsImpairProcessControl appearances | 1 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-016 | No duplicates in all_tactics_in_report_order | unit: HashSet deduplication + len check |
| VP-016 | All 16 variants are present | unit: iterate expected set, assert membership |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 -- completeness of tactic enumeration is a correctness property of the MITRE mapping capability |
| L2 Domain Invariants | INV-9 (MITRE technique ID format) |
| Architecture Module | SS-10 (mitre.rs, C-16) |
| Stories | STORY-071 |
| Origin BC | BC-MIT-004 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.003 -- composes with (this BC is the completeness companion to BC-2.10.003's ordering)

## Architecture Anchors

- `src/mitre.rs:95-114` -- all_tactics_in_report_order static slice (16 elements)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:95-114` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: manual count of slice elements (16 verified by pass-8 synthesis)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed, but note the maintenance risk: `#[non_exhaustive]` means a new
variant added to MitreTactic without updating this slice will produce a silent omission
bug in tactic-grouped reports. A compile-time exhaustiveness check (via a `match` that
maps each variant) would catch this but is not currently present.
