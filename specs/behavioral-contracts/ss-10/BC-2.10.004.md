---
document_type: behavioral-contract
level: L3
version: "1.4"
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
  - "v1.3: Feature #8 DNP3 analyzer (F2). MitreTactic gains third ICS-unique variant IcsImpact. All postconditions and invariants updated: variant count 16→17 (14 Enterprise + 3 ICS). Edge case EC-001 and test vectors updated. — 2026-06-10"
  - "v1.4: Pass-1 adversarial fix F3: corrected two stale informative lines — Architecture Anchors '16 elements' → '17 elements'; Evidence Types Used '16 verified' → '17 verified'. Normative body already said 17; only these two informative lines were stale. — 2026-06-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.004: all_tactics_in_report_order Contains Every Variant Exactly Once

## Description

The static slice returned by `all_tactics_in_report_order()` contains all 17 `MitreTactic`
variants with no duplicates and no omissions. This no-duplicate, no-omission guarantee
ensures that a tactic-grouped report iterates over every possible bucket exactly once,
producing a complete and non-redundant output. Because `MitreTactic` is `#[non_exhaustive]`,
the completeness guarantee is by convention (manual inspection) rather than compiler
enforcement. The count grew from 16 (14 Enterprise + 2 ICS) to 17 (14 Enterprise + 3 ICS)
with the addition of `IcsImpact` in Feature #8 (DNP3).

## Preconditions

1. `all_tactics_in_report_order()` is called.

## Postconditions

1. Each of the 17 `MitreTactic` variants appears in the slice exactly once.
2. No variant appears twice.
3. No variant is omitted.

## Invariants

1. The variant count is 17 (14 Enterprise + 3 ICS).
2. `#[non_exhaustive]` means the compiler cannot enforce completeness; human review and
   the duplicate-check test are the enforcement mechanism.
3. If a new variant is added to `MitreTactic`, the static slice in `all_tactics_in_report_order`
   must be updated manually; CI does not catch this automatically.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Collect slice into HashSet<MitreTactic> | Set size == 17 (no duplicates) |
| EC-002 | Iterate all_tactics and count occurrences of each variant | Every variant count == 1 |
| EC-003 | IcsImpact present in slice | Confirmed at position [16] |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| HashSet from all_tactics_in_report_order() | len == 17 | happy-path |
| Count IcsImpairProcessControl appearances | 1 | edge-case |
| Count IcsImpact appearances | 1 | edge-case (new F2 DNP3) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-016 | No duplicates in all_tactics_in_report_order | unit: HashSet deduplication + len check |
| VP-016 | All 17 variants are present | unit: iterate expected set, assert membership |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per domain/capabilities/cap-10-mitre-mapping.md |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per domain/capabilities/cap-10-mitre-mapping.md -- completeness of tactic enumeration is a correctness property of the MITRE mapping capability |
| L2 Domain Invariants | INV-9 (MITRE technique ID format) |
| Architecture Module | SS-10 (mitre.rs, C-16) |
| Stories | STORY-071 |
| Origin BC | BC-MIT-004 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.003 -- composes with (this BC is the completeness companion to BC-2.10.003's ordering)

## Architecture Anchors

- `src/mitre.rs:95-114` -- all_tactics_in_report_order static slice (17 elements)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:95-114` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: manual count of slice elements (17 verified post-F2 DNP3; was 16 pre-F2)

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
