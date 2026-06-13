---
document_type: behavioral-contract
level: L3
version: "1.7"
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
  - "v1.3: ADR-006 / Decision 12 (F2 v0.3.0) — Added 6 new ICS seeded IDs with correct tactic assignments: T0836 (IcsImpairProcessControl), T0814 (IcsInhibitResponseFunction), T0806 (IcsImpairProcessControl), T0835 (IcsImpairProcessControl), T0831 (IcsImpairProcessControl), T0888 (Discovery). Seeded count 15->21. EC-004 added for T0888. — 2026-06-09"
  - "v1.4: v19 remap: T0855 → T1692.001, T0856 → T1692.002 per MITRE ATT&CK for ICS v19.0 revocation. Tactic table rows updated to use new ICS sub-technique IDs. Tactic assignment (IcsImpairProcessControl) unchanged. Issue #222; audit: mitre-ics-v19-catalog-audit.md. — 2026-06-10"
  - "v1.5: Feature #8 DNP3 analyzer (F2). Added 2 new tactic assignments: T1691.001 → IcsInhibitResponseFunction (same parent tactic as T0814 Denial of Service), T0827 → IcsImpact (new ICS-unique tactic variant). Postcondition 2 extended. Seeded count 21→23; all_tactics_in_report_order must include new IcsImpact variant (see BC-2.10.002 update). — 2026-06-10"
  - "v1.6: Pass-1 adversarial fix C-1: corrected T1691.001 technique name in EC-007 from fabricated 'Unauthorized Message: Inhibit Response Function' to authoritative 'Block Operational Technology Message: Command Message' (parent T1691, tactic IcsInhibitResponseFunction). — 2026-06-10"
  - "v1.7: Pass-12 corpus-cleanup F-C-P12-002/F-C-P12-003: technique_tactic src anchor re-anchored from stale :166-168 to current :192-194 (Architecture Anchors + Source Evidence). PLANNED forward-declaration marker added: STORY-114 adds T0830→LateralMovement and T1557.002→CredentialAccess arms, raising seeded count 23→25 (mirrors BC-2.10.005/008). — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.007: technique_tactic Returns Correct Tactic for Every Seeded ID

<!--
  PREVIOUS VERSION SUMMARY (v1.2 -> v1.3):
  Seeded count: 15 -> 21
  Added tactic assignments for: T0836, T0814, T0806, T0835, T0831, T0888
  EC-004 added: T0888 -> Some(Discovery)
  Canonical vectors: added T0888 and T0836 rows
  Invariant 3 updated to note T0888 maps to Discovery (same as T0846 pattern)
-->

## Description

`technique_tactic(id)` returns `Some(MitreTactic)` for each of the 23 seeded technique IDs
(post-F2 DNP3), and the returned tactic is the correct parent tactic from MITRE ATT&CK
(Enterprise or ICS). Like `technique_name`, it is a thin projection over `technique_info`. The
tactic assignments match the ATT&CK matrix assignments (e.g., T1027 => DefenseEvasion, T0888
=> Discovery, T0806 => IcsImpairProcessControl, T0827 => IcsImpact).

PLANNED — implemented in STORY-114; current code 23 seeded → target 25 seeded after STORY-114 5-part atomic update. T0830→LateralMovement and T1557.002→CredentialAccess tactic arms added in STORY-114. src/mitre.rs remains at SEEDED=23 until STORY-114 lands; vp007_catalog_drift_guard enforces consistency at implementation time.

## Preconditions

1. `technique_tactic` is called with a known technique ID string.

## Postconditions

1. Returns `Some(MitreTactic)` for all 23 seeded IDs.
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
   - T1692.001 => MitreTactic::IcsImpairProcessControl  [v19 remap; was T0855]
   - T1692.002 => MitreTactic::IcsImpairProcessControl  [v19 remap; was T0856]
   - T0885 => MitreTactic::CommandAndControl
   - T0836 => MitreTactic::IcsImpairProcessControl  [NEW F2]
   - T0814 => MitreTactic::IcsInhibitResponseFunction  [NEW F2]
   - T0806 => MitreTactic::IcsImpairProcessControl  [NEW F2]
   - T0835 => MitreTactic::IcsImpairProcessControl  [NEW F2]
   - T0831 => MitreTactic::IcsImpairProcessControl  [NEW F2]
   - T0888 => MitreTactic::Discovery  [NEW F2 — replaces T0846 as Modbus recon emitter per Decision 12]
   - T1691.001 => MitreTactic::IcsInhibitResponseFunction  [NEW F2 DNP3 — inferred block-command]
   - T0827 => MitreTactic::IcsImpact  [NEW F2 DNP3 — derived loss-of-control correlated finding]
3. Returns `None` for any ID not in the seeded set.

## Invariants

1. Tactic assignments are derived from the same `technique_info` match table as `technique_name`.
2. It is impossible for `technique_name` and `technique_tactic` to disagree for the same ID.
3. The ICS techniques (T0xxx/T1xxx ICS sub-techniques) may map to ICS-specific tactics OR to
   Enterprise tactics that share a name (e.g., T0846 and T0888 both map to Discovery, same
   name as Enterprise TA0007; T0814 and T1691.001 map to IcsInhibitResponseFunction which has
   no Enterprise equivalent; T0827 maps to IcsImpact, a third ICS-unique tactic variant).
4. With the addition of T0827 → IcsImpact, the MitreTactic enum gains a third ICS-unique
   variant. `all_tactics_in_report_order` must include IcsImpact after IcsImpairProcessControl
   (see BC-2.10.003 — slice grows from length 16 to 17, and BC-2.10.004 — count 16→17).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | T1027 | Some(DefenseEvasion) |
| EC-002 | T0846 | Some(Discovery) -- ICS technique maps to Discovery tactic; seeded but not Modbus-emitted |
| EC-003 | T9999 | None |
| EC-004 | T0888 | Some(Discovery) -- Remote System Information Discovery; Modbus recon emitter (replaces T0846 in emission) |
| EC-005 | T0806 | Some(IcsImpairProcessControl) -- Brute Force I/O; emitted by write-burst detector |
| EC-006 | T0814 | Some(IcsInhibitResponseFunction) -- Denial of Service; emitted by Force Listen Only FC |
| EC-007 | T1691.001 | Some(IcsInhibitResponseFunction) -- Block Operational Technology Message: Command Message; DNP3 inferred block-command |
| EC-008 | T0827 | Some(IcsImpact) -- Loss of Control; DNP3 derived correlated finding |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| technique_tactic("T1027") | Some(DefenseEvasion) | happy-path |
| technique_tactic("T1499.002") | Some(Impact) | happy-path |
| technique_tactic("T0885") | Some(CommandAndControl) | happy-path |
| technique_tactic("T0888") | Some(Discovery) | happy-path (new F2) |
| technique_tactic("T0836") | Some(IcsImpairProcessControl) | happy-path (new F2) |
| technique_tactic("T0814") | Some(IcsInhibitResponseFunction) | happy-path (new F2) |
| technique_tactic("T1691.001") | Some(IcsInhibitResponseFunction) | happy-path (new F2 DNP3) |
| technique_tactic("T0827") | Some(IcsImpact) | happy-path (new F2 DNP3) |
| technique_tactic("T9999") | None | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-007 | All 23 seeded IDs return correct tactic | unit: exhaustive tactic-assignment assertions |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per domain/capabilities/cap-10-mitre-mapping.md |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per domain/capabilities/cap-10-mitre-mapping.md -- technique_tactic is the tactic-lookup function of the MITRE catalog used to group findings in reports |
| L2 Domain Invariants | INV-9 (MITRE technique ID format) |
| Architecture Module | SS-10 (mitre.rs, C-16) |
| Stories | STORY-071 |
| Origin BC | BC-MIT-007 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.005 -- composes with (both are projections of technique_info)
- BC-2.10.008 -- composes with (all emitted IDs must resolve via this function)

## Architecture Anchors

- `src/mitre.rs:192-194` -- technique_tactic thin wrapper

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:192-194` |
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
