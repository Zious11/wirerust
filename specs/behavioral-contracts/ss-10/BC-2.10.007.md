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

# BC-2.10.007: technique_tactic Returns Correct Tactic for Every Seeded ID

## Description

`technique_tactic(id)` returns `Some(MitreTactic)` for each of the 15 seeded technique IDs,
and the returned tactic is the correct parent tactic from MITRE ATT&CK. Like `technique_name`,
it is a thin projection over `technique_info`. The tactic assignments match the ATT&CK matrix
assignments (e.g., T1027 => DefenseEvasion, T1036 => DefenseEvasion, T1046 => Discovery).

## Preconditions

1. `technique_tactic` is called with a known technique ID string.

## Postconditions

1. Returns `Some(MitreTactic)` for all 15 seeded IDs.
2. The tactic assignments are:
   - T1027 => MitreTactic::DefenseEvasion
   - T1036 => MitreTactic::DefenseEvasion
   - T1040 => MitreTactic::CredentialAccess
   - T1046 => MitreTactic::Discovery
   - T1071 => MitreTactic::CommandAndControl
   - T1071.001 => MitreTactic::CommandAndControl
   - T1071.004 => MitreTactic::CommandAndControl
   - T1083 => MitreTactic::Discovery
   - T1499.002 => MitreTactic::Impact
   - T1505.003 => MitreTactic::Persistence
   - T1573 => MitreTactic::CommandAndControl
   - T0846 => MitreTactic::Discovery
   - T0855 => MitreTactic::IcsImpairProcessControl
   - T0856 => MitreTactic::IcsImpairProcessControl
   - T0885 => MitreTactic::CommandAndControl
3. Returns `None` for any ID not in the seeded set.

## Invariants

1. Tactic assignments are derived from the same `technique_info` match table as `technique_name`.
2. It is impossible for `technique_name` and `technique_tactic` to disagree for the same ID.
3. The ICS techniques (T0xxx) may map to ICS-specific tactics OR to Enterprise tactics that
   share a name (e.g., T0846 maps to Discovery, same name as Enterprise TA0007).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | T1027 | Some(DefenseEvasion) |
| EC-002 | T0846 | Some(Discovery) -- ICS technique maps to Discovery tactic |
| EC-003 | T9999 | None |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| technique_tactic("T1027") | Some(DefenseEvasion) | happy-path |
| technique_tactic("T1499.002") | Some(Impact) | happy-path |
| technique_tactic("T0885") | Some(CommandAndControl) | happy-path |
| technique_tactic("T9999") | None | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All 15 seeded IDs return correct tactic | unit: exhaustive tactic-assignment assertions |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 -- technique_tactic is the tactic-lookup function of the MITRE catalog used to group findings in reports |
| L2 Domain Invariants | INV-9 (MITRE technique ID format) |
| Architecture Module | SS-10 (mitre.rs, C-16) |
| Stories | S-TBD |
| Origin BC | BC-MIT-007 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.005 -- composes with (both are projections of technique_info)
- BC-2.10.008 -- composes with (all emitted IDs must resolve via this function)

## Architecture Anchors

- `src/mitre.rs:166-168` -- technique_tactic thin wrapper

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:166-168` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: technique_info returns (name, tactic) pair; tactic projection is lossless

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
