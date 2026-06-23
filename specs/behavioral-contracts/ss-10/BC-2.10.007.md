---
document_type: behavioral-contract
level: L3
version: "1.9"
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
  - "v1.8: Post-STORY-114-merge governance update (F7 follow-up item 5, validated by research DF-VALIDATION-001; report .factory/research/arp-followups-validation.md item 5): PLANNED marker resolved to landed status (PR #240, develop HEAD 7c0f453). SEEDED=25/EMITTED=17 confirmed in src/mitre.rs (SEEDED_TECHNIQUE_ID_COUNT=25; T0830→LateralMovement and T1557.002→CredentialAccess arms present, Kani-proven T0830/T1557.002). Postcondition 1 count 23→25. Tactic rows for T0830 and T1557.002 added. VP-007 count 23→25. EC-009/EC-010 added. Canonical vectors for T0830/T1557.002 added. Architecture Anchors updated. — 2026-06-16"
  - "v1.9: F5 ICS tactic-ID correctness fix (issue #64 follow-up, f5-ics-technique-tactic-authoritative.md + f5-ics-catalog-fix-scope.md). Four ICS techniques remapped to new dedicated ICS variants: T0846→IcsDiscovery (TA0102), T0888→IcsDiscovery (TA0102), T0830→IcsCollection (TA0100), T0885→IcsCommandAndControl (TA0101). T0831 corrected from IcsImpairProcessControl→IcsImpact (TA0105) per MITRE ICS v19.1 page verification. Description, Postcondition 2 tactic assignment rows, Invariants 3–4, Edge Cases EC-002/EC-004/EC-009, Canonical Test Vectors updated. — 2026-06-23"
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

`technique_tactic(id)` returns `Some(MitreTactic)` for each of the 25 seeded technique IDs
(post-F2 ARP), and the returned tactic is the correct parent tactic from MITRE ATT&CK
(Enterprise or ICS). Like `technique_name`, it is a thin projection over `technique_info`. The
tactic assignments match the ATT&CK matrix assignments (e.g., T1027 => DefenseEvasion,
T0888 => IcsDiscovery, T0806 => IcsImpairProcessControl, T0827 => IcsImpact,
T0830 => IcsCollection, T0831 => IcsImpact, T1557.002 => CredentialAccess). ICS techniques
that collide with Enterprise tactic names use dedicated ICS variants (IcsDiscovery TA0102,
IcsCollection TA0100, IcsCommandAndControl TA0101) to emit the authoritative ICS-matrix TA-id
per f5-ics-technique-tactic-authoritative.md (MITRE ATT&CK for ICS v19.1, verified).

LANDED — STORY-114 merged (PR #240, develop HEAD 7c0f453). src/mitre.rs is now at SEEDED=25/EMITTED=17.
T0830 (ICS Collection) and T1557.002 (Enterprise CredentialAccess) tactic arms are present in technique_info;
vp007_catalog_drift_guard enforces consistency at runtime.

## Preconditions

1. `technique_tactic` is called with a known technique ID string.

## Postconditions

1. Returns `Some(MitreTactic)` for all 25 seeded IDs.
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
   - T0846 => MitreTactic::IcsDiscovery  [F5: ICS Discovery TA0102; was Enterprise Discovery TA0007]
   - T1692.001 => MitreTactic::IcsImpairProcessControl  [v19 remap; was T0855]
   - T1692.002 => MitreTactic::IcsImpairProcessControl  [v19 remap; was T0856]
   - T0885 => MitreTactic::IcsCommandAndControl  [F5: ICS Command and Control TA0101; was Enterprise CommandAndControl TA0011]
   - T0836 => MitreTactic::IcsImpairProcessControl  [NEW F2]
   - T0814 => MitreTactic::IcsInhibitResponseFunction  [NEW F2]
   - T0806 => MitreTactic::IcsImpairProcessControl  [NEW F2]
   - T0835 => MitreTactic::IcsImpairProcessControl  [NEW F2]
   - T0831 => MitreTactic::IcsImpact  [F5: ICS Impact TA0105; was IcsImpairProcessControl TA0106 — WRONG per MITRE ICS v19.1]
   - T0888 => MitreTactic::IcsDiscovery  [F5: ICS Discovery TA0102; was Enterprise Discovery TA0007; Modbus recon emitter]
   - T1691.001 => MitreTactic::IcsInhibitResponseFunction  [NEW F2 DNP3 — inferred block-command]
   - T0827 => MitreTactic::IcsImpact  [NEW F2 DNP3 — derived loss-of-control correlated finding]
   - T0830 => MitreTactic::IcsCollection  [F5: ICS Collection TA0100; was LateralMovement TA0008 — WRONG per MITRE ICS v19.1; ARP spoof emitted]
   - T1557.002 => MitreTactic::CredentialAccess  [NEW F2 ARP — Adversary-in-the-Middle: ARP Cache Poisoning, Enterprise]
3. Returns `None` for any ID not in the seeded set.

## Invariants

1. Tactic assignments are derived from the same `technique_info` match table as `technique_name`.
2. It is impossible for `technique_name` and `technique_tactic` to disagree for the same ID.
3. ICS techniques use dedicated ICS-matrix tactic variants, NOT Enterprise variants that share a
   tactic name. T0846 and T0888 map to IcsDiscovery (TA0102), not Enterprise Discovery (TA0007);
   T0885 maps to IcsCommandAndControl (TA0101), not Enterprise CommandAndControl (TA0011);
   T0830 maps to IcsCollection (TA0100), not Enterprise Collection (TA0009) nor LateralMovement.
   T0814 and T1691.001 map to IcsInhibitResponseFunction (TA0107); T0827, T0831 map to
   IcsImpact (TA0105) — both the same ICS Impact variant. T1557.002 is an Enterprise-only
   technique and correctly uses the Enterprise CredentialAccess (TA0006) variant.
4. With F5, the MitreTactic enum gains three additional ICS variants (IcsDiscovery,
   IcsCollection, IcsCommandAndControl), growing from 17 to 20 total variants.
   `all_tactics_in_report_order` grows from length 17 to 20 (see BC-2.10.003 v1.5).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | T1027 | Some(DefenseEvasion) |
| EC-002 | T0846 | Some(IcsDiscovery) -- ICS Remote System Discovery maps to ICS Discovery tactic TA0102; seeded but not Modbus-emitted |
| EC-003 | T9999 | None |
| EC-004 | T0888 | Some(IcsDiscovery) -- Remote System Information Discovery; ICS Discovery TA0102; Modbus recon emitter (replaces T0846 in emission) |
| EC-005 | T0806 | Some(IcsImpairProcessControl) -- Brute Force I/O; emitted by write-burst detector |
| EC-006 | T0814 | Some(IcsInhibitResponseFunction) -- Denial of Service; emitted by Force Listen Only FC |
| EC-007 | T1691.001 | Some(IcsInhibitResponseFunction) -- Block Operational Technology Message: Command Message; DNP3 inferred block-command |
| EC-008 | T0827 | Some(IcsImpact) -- Loss of Control; DNP3 derived correlated finding |
| EC-009 | T0830 | Some(IcsCollection) -- Adversary-in-the-Middle; ICS Collection TA0100; ARP spoof/mismatch paths (F5 correction: was LateralMovement TA0008, WRONG per MITRE ICS v19.1) |
| EC-010 | T1557.002 | Some(CredentialAccess) -- Adversary-in-the-Middle: ARP Cache Poisoning; Enterprise CredentialAccess |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| technique_tactic("T1027") | Some(DefenseEvasion) | happy-path |
| technique_tactic("T1499.002") | Some(Impact) | happy-path |
| technique_tactic("T0885") | Some(IcsCommandAndControl) | happy-path (F5: ICS C2 TA0101) |
| technique_tactic("T0888") | Some(IcsDiscovery) | happy-path (F5: ICS Discovery TA0102) |
| technique_tactic("T0846") | Some(IcsDiscovery) | happy-path (F5: ICS Discovery TA0102; seeded-only) |
| technique_tactic("T0831") | Some(IcsImpact) | happy-path (F5: ICS Impact TA0105; was IcsImpairProcessControl — corrected) |
| technique_tactic("T0836") | Some(IcsImpairProcessControl) | happy-path (new F2) |
| technique_tactic("T0814") | Some(IcsInhibitResponseFunction) | happy-path (new F2) |
| technique_tactic("T1691.001") | Some(IcsInhibitResponseFunction) | happy-path (new F2 DNP3) |
| technique_tactic("T0827") | Some(IcsImpact) | happy-path (new F2 DNP3) |
| technique_tactic("T0830") | Some(IcsCollection) | happy-path (F5: ICS Collection TA0100; was LateralMovement — corrected) |
| technique_tactic("T1557.002") | Some(CredentialAccess) | happy-path (new F2 ARP, Enterprise) |
| technique_tactic("T9999") | None | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-007 | All 25 seeded IDs return correct tactic | unit: exhaustive tactic-assignment assertions |

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

- `src/mitre.rs:192-194` -- technique_tactic thin wrapper (T0830 and T1557.002 tactic arms landed in STORY-114, PR #240)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:192-194` (T0830 and T1557.002 arms landed in STORY-114, PR #240) |
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
