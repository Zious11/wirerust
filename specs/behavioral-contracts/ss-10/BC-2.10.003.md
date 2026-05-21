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

# BC-2.10.003: all_tactics_in_report_order Returns Kill-Chain Order First Then ICS

## Description

`all_tactics_in_report_order()` returns a static slice of all 16 `MitreTactic` variants in
a fixed order: the 14 Enterprise ATT&CK tactics in canonical kill-chain order (Reconnaissance
through Impact), followed by the 2 ICS-unique tactics. This function provides the stable
iteration order used by the terminal reporter to generate tactic group headers in a
consistent, predictable sequence.

## Preconditions

1. `all_tactics_in_report_order()` is called (no preconditions; takes no arguments).

## Postconditions

1. Returns a `&'static [MitreTactic]` slice of length exactly 16.
2. The first 14 elements are the Enterprise tactics in this order:
   Reconnaissance, ResourceDevelopment, InitialAccess, Execution, Persistence,
   PrivilegeEscalation, DefenseEvasion, CredentialAccess, Discovery, LateralMovement,
   Collection, CommandAndControl, Exfiltration, Impact.
3. Elements [14] and [15] are: IcsInhibitResponseFunction, IcsImpairProcessControl.
4. The returned reference is `'static`; no heap allocation occurs.

## Invariants

1. The function is a `&'static` literal slice -- it never changes at runtime.
2. The length is always 16 (14 Enterprise + 2 ICS).
3. The order is the authoritative render order for the terminal reporter.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slice length | all_tactics_in_report_order().len() == 16 |
| EC-002 | First element | Reconnaissance |
| EC-003 | Last element | IcsImpairProcessControl |
| EC-004 | No duplicate elements | All 16 variants appear exactly once |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| all_tactics_in_report_order().len() | 16 | happy-path |
| all_tactics_in_report_order()[0] | Reconnaissance | happy-path |
| all_tactics_in_report_order()[13] | Impact | happy-path |
| all_tactics_in_report_order()[14] | IcsInhibitResponseFunction | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-016 | Slice length is 16 | unit: assert_eq!(all_tactics_in_report_order().len(), 16) |
| VP-016 | No duplicate variants in the slice | unit: HashSet dedup check |
| VP-016 | Kill-chain order for first 14 | unit: assert positions 0..14 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 -- all_tactics_in_report_order is the stable enumeration contract for the tactic-grouped reporter output |
| L2 Domain Invariants | INV-9 (MITRE technique ID format) |
| Architecture Module | SS-10 (mitre.rs, C-16) |
| Stories | STORY-071 |
| Origin BC | BC-MIT-003 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.004 -- composes with (no duplicates: every variant appears exactly once)
- BC-2.10.001 -- composes with (Enterprise Display strings)
- BC-2.10.002 -- composes with (ICS Display strings)

## Architecture Anchors

- `src/mitre.rs:95-114` -- all_tactics_in_report_order static slice literal

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:95-114` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: &'static [MitreTactic] return type; compile-time slice literal

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (static data) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed. If MitreTactic gains new variants, this function must be updated
to include them in the correct kill-chain position. The compiler will not warn about
missing variants in a `&[...]` literal.
