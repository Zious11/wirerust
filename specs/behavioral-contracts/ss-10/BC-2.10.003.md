---
document_type: behavioral-contract
level: L3
version: "1.5"
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
  - "v1.3: Feature #8 DNP3 analyzer (F2). MitreTactic gains third ICS-unique variant IcsImpact. Slice length 16→17. Element [16] = IcsImpact. Description, Postconditions, Invariants, Edge Cases, and Canonical Test Vectors updated. — 2026-06-10"
  - "v1.4: Pass-12 corpus-cleanup F-C-P12-004: all_tactics_in_report_order anchor re-anchored from stale :95-114 to current :100-120 (matching sibling BC-2.10.004). Architecture Anchors and Source Evidence updated. — 2026-06-13"
  - "v1.5: F5 ICS tactic-ID correctness fix. Slice length 17→20. Three new ICS variants appended after IcsImpact: IcsDiscovery [17], IcsCollection [18], IcsCommandAndControl [19]. Description, Postconditions 1/3/4, Invariant 2, Edge Cases EC-001/EC-003/EC-004/EC-006, and Canonical Test Vectors updated. — 2026-06-23"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.003: all_tactics_in_report_order Returns Kill-Chain Order First Then ICS

## Description

`all_tactics_in_report_order()` returns a static slice of all 20 `MitreTactic` variants in
a fixed order: the 14 Enterprise ATT&CK tactics in canonical kill-chain order (Reconnaissance
through Impact), followed by the 6 ICS-unique variants. This function provides the stable
iteration order used by the terminal reporter to generate tactic group headers in a
consistent, predictable sequence. The third ICS variant `IcsImpact` was added in Feature #8
(DNP3) to support T0827 "Loss of Control" (ICS Impact tactic TA0105). Three additional ICS
variants were added in F5 to emit correct ICS-matrix TA-ids for techniques that previously
merged into Enterprise variants: `IcsDiscovery` (TA0102), `IcsCollection` (TA0100), and
`IcsCommandAndControl` (TA0101).

## Preconditions

1. `all_tactics_in_report_order()` is called (no preconditions; takes no arguments).

## Postconditions

1. Returns a `&'static [MitreTactic]` slice of length exactly 20.
2. The first 14 elements are the Enterprise tactics in this order:
   Reconnaissance, ResourceDevelopment, InitialAccess, Execution, Persistence,
   PrivilegeEscalation, DefenseEvasion, CredentialAccess, Discovery, LateralMovement,
   Collection, CommandAndControl, Exfiltration, Impact.
3. Elements [14], [15], and [16] are: IcsInhibitResponseFunction, IcsImpairProcessControl,
   IcsImpact.
4. Elements [17], [18], and [19] are: IcsDiscovery, IcsCollection, IcsCommandAndControl.
5. The returned reference is `'static`; no heap allocation occurs.

## Invariants

1. The function is a `&'static` literal slice -- it never changes at runtime.
2. The length is always 20 (14 Enterprise + 6 ICS).
3. The order is the authoritative render order for the terminal reporter.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slice length | all_tactics_in_report_order().len() == 20 |
| EC-002 | First element | Reconnaissance |
| EC-003 | Last element | IcsCommandAndControl |
| EC-004 | No duplicate elements | All 20 variants appear exactly once |
| EC-005 | Element at index [15] | IcsImpairProcessControl |
| EC-006 | Element at index [16] | IcsImpact (added F2 DNP3) |
| EC-007 | Element at index [17] | IcsDiscovery (added F5) |
| EC-008 | Element at index [18] | IcsCollection (added F5) |
| EC-009 | Element at index [19] | IcsCommandAndControl (added F5) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| all_tactics_in_report_order().len() | 20 | happy-path |
| all_tactics_in_report_order()[0] | Reconnaissance | happy-path |
| all_tactics_in_report_order()[13] | Impact | happy-path |
| all_tactics_in_report_order()[14] | IcsInhibitResponseFunction | happy-path |
| all_tactics_in_report_order()[15] | IcsImpairProcessControl | happy-path |
| all_tactics_in_report_order()[16] | IcsImpact | happy-path (added F2 DNP3) |
| all_tactics_in_report_order()[17] | IcsDiscovery | happy-path (added F5) |
| all_tactics_in_report_order()[18] | IcsCollection | happy-path (added F5) |
| all_tactics_in_report_order()[19] | IcsCommandAndControl | happy-path (added F5) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-016 | Slice length is 20 | unit: assert_eq!(all_tactics_in_report_order().len(), 20) |
| VP-016 | No duplicate variants in the slice | unit: HashSet dedup check |
| VP-016 | Kill-chain order for first 14 | unit: assert positions 0..14 |
| VP-016 | IcsImpact at position [16] | unit: assert_eq!(slice[16], IcsImpact) |
| VP-016 | IcsDiscovery at position [17] | unit: assert_eq!(slice[17], IcsDiscovery) |
| VP-016 | IcsCollection at position [18] | unit: assert_eq!(slice[18], IcsCollection) |
| VP-016 | IcsCommandAndControl at position [19] | unit: assert_eq!(slice[19], IcsCommandAndControl) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per domain/capabilities/cap-10-mitre-mapping.md |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per domain/capabilities/cap-10-mitre-mapping.md -- all_tactics_in_report_order is the stable enumeration contract for the tactic-grouped reporter output |
| L2 Domain Invariants | INV-9 (MITRE technique ID format) |
| Architecture Module | SS-10 (mitre.rs, C-16) |
| Stories | STORY-071 |
| Origin BC | BC-MIT-003 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.004 -- composes with (no duplicates: every variant appears exactly once)
- BC-2.10.001 -- composes with (Enterprise Display strings)
- BC-2.10.002 -- composes with (ICS Display strings)

## Architecture Anchors

- `src/mitre.rs:100-120` -- all_tactics_in_report_order static slice literal

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:100-120` |
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
to include them in the correct kill-chain position (Enterprise) or ICS append position
(ICS-unique variants). The compiler will not warn about missing variants in a `&[...]` literal;
the `vp007_catalog_drift_guard` test / VP-016 unit tests are the runtime enforcement gate.
