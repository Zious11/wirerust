---
document_type: behavioral-contract
level: L3
version: "1.12"
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
  - "v1.3: Wave 3 Ph3 pass-1 adversarial fix: m-1 correct technique_info line-anchor range to :122-156 (closing brace verified) in Architecture Anchors and Source Evidence — 2026-05-22 (product-owner)"
  - "v1.4: ADR-006 / Decision 12+13 (F2 v0.3.0) — SEEDED count updated 15 -> 21 (added 6 ICS: T0836,T0814,T0806,T0835,T0831,T0888; T0846 kept seeded, not Modbus-emitted; T0888 replaces T0846 in Modbus recon emission). Postconditions, Invariants, canonical vectors updated. — 2026-06-09"
  - "v1.5: v19 remap: T0855 → T1692.001, T0856 → T1692.002 per MITRE ATT&CK for ICS v19.0 revocation. ICS seeded ID list and Postcondition 1 updated. Seeded count remains 21. Issue #222; audit: mitre-ics-v19-catalog-audit.md. — 2026-06-10"
  - "v1.6: Feature #8 DNP3 analyzer (F2). Added 2 new ICS techniques: T1691.001 (Block Operational Technology Message: Command Message, IcsInhibitResponseFunction) + T0827 (Loss of Control, IcsImpact). SEEDED count 21→23 (11 Enterprise + 12 ICS). H1 title updated 21→23. — 2026-06-10"
  - "v1.7: Pass-1 adversarial fix C-1: corrected T1691.001 technique name from fabricated 'Unauthorized Message: Inhibit Response Function' to authoritative 'Block Operational Technology Message: Command Message' (parent T1691, tactic IcsInhibitResponseFunction) in changelog v1.6, EC-009, and canonical test vectors. Fixed duplicate invariant numbering (I-6): second invariant 4 renumbered to 5. — 2026-06-10"
  - "v1.8: Feature #9 ARP analyzer (F2). Added 2 new techniques: T0830 (ICS: Adversary-in-the-Middle, LateralMovement) + T1557.002 (Enterprise: Adversary-in-the-Middle: ARP Cache Poisoning, CredentialAccess). SEEDED count 23→25 (12 Enterprise + 13 ICS). EMITTED count 15→17. H1 title updated 23→25. — 2026-06-12 (F-D-C1 pass-2 remediation)"
  - "v1.9: Pass-3 remediation F-C4/F-C5/F-C6/F-C1(b): T1557.002 reclassified Enterprise (not ICS); Enterprise/ICS split corrected 11E+14I→12E+13I in Description, Postcondition 3, Invariants, and changelog; EC-011/012 added for T0830/T1557.002; canonical vectors for T0830/T1557.002 added; VP table 'All 23 seeded IDs'→'All 25 seeded IDs'; Architecture Anchors re-anchored to current mitre.rs line numbers (T0885:158, _ => return None:179); PLANNED forward-declaration marker added. — 2026-06-12"
  - "v1.10: Pass-4 remediation F-C-P4-HIGH-003: PLANNED marker augmented with current→target values (23/15→25/17 after STORY-114 5-part atomic update). — 2026-06-12"
  - "v1.11: Post-STORY-114-merge governance update: PLANNED marker resolved to landed status (PR #240, develop HEAD 7c0f453). SEEDED=25/EMITTED=17 confirmed in src/mitre.rs (SEEDED_TECHNIQUE_ID_COUNT=25; EMITTED_IDS array=17 entries; T0830+T1557.002 arms present). Architecture Anchors updated: T0830/T1557.002 arms no longer PLANNED. — 2026-06-15"
  - "v1.12: F2 EtherNet/IP (feature-enip-v0.11.0, ADR-010 Decision 7) — SEEDED count updated 25→28 (added 3 ICS: T0858 'Change Operating Mode' IcsExecution TA0104 new catalog entry; T0816 'Device Restart/Shutdown' IcsInhibitResponseFunction TA0107 new catalog entry; T1693.001 'Modify Firmware: System Firmware' staged-not-emitted). ICS split 13→16. H1 title updated 25→28. T0846 reclassified seeded+emitted (was catalogue-only). CATALOGUE-ONLY count remains 8 (T0846 leaves; T1693.001 enters; T0858/T0816 new seeds immediately emitted; net=0). Postconditions, Invariants, Edge Cases updated. PRD §2.10 O-04 SEEDED=28 authoritative reference. — 2026-06-24"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.005: technique_name Returns Some for Every Seeded ID (28 Total)

<!--
  PREVIOUS VERSION SUMMARY (v1.3 -> v1.4):
  Title: "15 Total" -> "21 Total"
  Seeded count: 15 -> 21 (11 Enterprise + 10 ICS)
  Added seeded ICS IDs: T0836, T0814, T0806, T0835, T0831, T0888
  Invariant 1: emitted count 6 -> 13 (6 Enterprise + 7 ICS)
  Invariant 2: catalogued-but-not-emitted updated (T0846 remains non-emitted; staged IDS updated)
  Invariant 3: count claim updated 15 -> 21

  PREVIOUS VERSION SUMMARY (v1.5 -> v1.6):
  Title: "21 Total" -> "23 Total"
  Seeded count: 21 -> 23 (11 Enterprise + 12 ICS)
  Added seeded ICS IDs: T1691.001, T0827 (Feature #8 DNP3)
  Invariant 1: emitted count 13 -> 15 (6 Enterprise + 9 ICS)
  Invariant 2: catalogued-but-not-emitted remains 8 (23 - 15 = 8)
  Invariant 3: count claim updated 21 -> 23

  PREVIOUS VERSION SUMMARY (v1.7 -> v1.8):
  Title: "23 Total" -> "25 Total"
  Seeded count: 23 -> 25 (12 Enterprise + 13 ICS)
  Added: T0830 (ICS: Adversary-in-the-Middle), T1557.002 (Enterprise: Adversary-in-the-Middle: ARP Cache Poisoning) — Feature #9 ARP
  Invariant 1: emitted count 15 -> 17 (7 Enterprise + 10 ICS)
  Invariant 2: catalogued-but-not-emitted remains 8 (25 - 17 = 8)
  Invariant 3: count claim updated 23 -> 25

  PREVIOUS VERSION SUMMARY (v1.11 -> v1.12):
  Title: "25 Total" -> "28 Total"
  Seeded count: 25 -> 28 (12 Enterprise + 16 ICS)
  Added ICS IDs: T0858 (Change Operating Mode, IcsExecution TA0104 — CIP Stop, new catalog entry),
                 T0816 (Device Restart/Shutdown, IcsInhibitResponseFunction TA0107 — CIP Reset, new catalog entry),
                 T1693.001 (Modify Firmware: System Firmware, staged-not-emitted v0.11.0)
  Note: T0846 was already seeded; it is NOW also emitted (by ENIP BC-2.17.010 ListIdentity)
  Invariant 1: emitted count 17 -> 20 (7 Enterprise + 13 ICS): adds T0858, T0816, T0846
  Invariant 2: catalogued-but-not-emitted remains 8 (28 - 20 = 8); T0846 exits; T1693.001 enters;
               T0858/T0816 new seeds immediately emitted
  Invariant 3: count claim updated 25 -> 28
  EC-013/014/015 added for T0858, T0816, T1693.001
-->

## Description

`technique_name(id: &str)` returns `Some(&'static str)` for all 28 technique IDs present in
the `technique_info` static match table. IDs not in the table return `None`. The 28-entry
catalog (post-F2 EtherNet/IP v0.11.0) includes 20 IDs emitted by analyzers and 8 staged IDs for
future analyzers. The catalog grows from 25 (post-Feature #9 ARP, 12 Enterprise + 13 ICS) to 28
(12 Enterprise + 16 ICS) as part of Feature #316 (EtherNet/IP + CIP analyzer, ADR-010):
T0858 "Change Operating Mode" (IcsExecution TA0104 — CIP Stop) and T0816 "Device Restart/Shutdown"
(IcsInhibitResponseFunction TA0107 — CIP Reset) are new catalog entries immediately emitted.
T1693.001 "Modify Firmware: System Firmware" is seeded-but-not-emitted (GetAndClear firmware
detection deferred to v0.12.0). T0846 "Remote System Discovery" was already seeded; it is NOW also
emitted by the EtherNet/IP analyzer (BC-2.17.010 ListIdentity).

PENDING — STORY-EIP-09 (VP-007 atomic burst for T0858+T0816 new technique_info arms + IcsExecution
MitreTactic variant). SEEDED=28/EMITTED=20 per ADR-010 Decision 7 + PRD §2.10 O-04 (v1.36).
vp007_catalog_drift_guard will enforce consistency at runtime once implemented.

## Preconditions

1. `technique_name` is called with a string argument.

## Postconditions

1. For each of the 28 seeded IDs, returns `Some(technique_name_string)`.
2. For any other string, returns `None`.
3. The 28 seeded IDs are:
   - Enterprise (12): T1027, T1036, T1040, T1046, T1071, T1071.001, T1071.004,
     T1083, T1499.002, T1505.003, T1573, T1557.002
   - ICS (16): T0846, T1692.001, T1692.002, T0885, T0836, T0814, T0806, T0835, T0831, T0888,
     T1691.001, T0827, T0830, T0858, T0816, T1693.001

## Invariants

1. IDs currently emitted (20): 7 Enterprise (T1027, T1036, T1046, T1083, T1499.002,
   T1505.003, T1557.002) + 13 ICS (T1692.001, T0836, T0814, T0806, T0835, T0831, T0888,
   T1691.001, T0827, T0830, T0858, T0816, T0846).
   T0858 (CIP Stop — IcsExecution TA0104) and T0816 (CIP Reset — IcsInhibitResponseFunction TA0107)
   are new catalog entries added for the EtherNet/IP analyzer (Feature #316, v0.11.0, ADR-010).
   T0846 (ListIdentity — IcsDiscovery TA0102) was already seeded; it is NOW also emitted by the
   EtherNet/IP analyzer (BC-2.17.010).
2. IDs catalogued but not emitted (8): T1040, T1071, T1071.001, T1071.004, T1573 (Enterprise);
   T1692.002 (ICS — IcsImpairProcessControl; replaces revoked T0856 per ATT&CK-ICS v19 remap),
   T0885 (ICS — CommandAndControl), T1693.001 (ICS — IcsInhibitResponseFunction; staged firmware
   detection, seeded-not-emitted v0.11.0; replaces revoked T0857 per ATT&CK-ICS v19 remap).
   T0846 was previously catalogue-only; it has been reclassified to emitted (EtherNet/IP
   ListIdentity, BC-2.17.010). T1693.001 enters catalogue-only as the staged firmware entry.
   The net catalogue-only count remains 8.
3. The catalog count is 28 after Feature #316 EtherNet/IP (F2, v0.11.0). Post-ARP (Feature #9)
   count was 25. Any claim of 25 post-ENIP is an error; any claim of 26 or 27 is an error
   (28 = 12 Enterprise + 16 ICS; T0858, T0816, T1693.001 are new ICS entries).
4. Arithmetic check: SEEDED=28 (12E+16I), EMITTED=20 (7E+13I), CATALOGUE-ONLY=28−20=8.
   These counts are mutually consistent.
5. The match is exact string equality; no prefix/suffix matching.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | "T9999" (unknown) | None |
| EC-002 | "" (empty string) | None |
| EC-003 | "T1046.999" (unknown sub-technique) | None |
| EC-004 | "garbage" | None |
| EC-005 | "t1027" (lowercase) | None (case-sensitive) |
| EC-006 | "T1071.001" (sub-technique) | Some("Web Protocols") |
| EC-007 | "T0888" (new ICS seeded — Remote System Information Discovery) | Some("Remote System Information Discovery") |
| EC-008 | "T0836" (new ICS seeded — Modify Parameter) | Some("Modify Parameter") |
| EC-009 | "T1691.001" (new ICS seeded F2 DNP3 — Block Operational Technology Message: Command Message) | Some("Block Operational Technology Message: Command Message") |
| EC-010 | "T0827" (new ICS seeded F2 DNP3 — Loss of Control) | Some("Loss of Control") |
| EC-011 | "T0830" (new ICS seeded F2 ARP — Adversary-in-the-Middle, ICS LateralMovement) | Some("Adversary-in-the-Middle") |
| EC-012 | "T1557.002" (new Enterprise seeded F2 ARP — Adversary-in-the-Middle: ARP Cache Poisoning, CredentialAccess) | Some("Adversary-in-the-Middle: ARP Cache Poisoning") |
| EC-013 | "T0858" (new ICS seeded F2 ENIP — Change Operating Mode, IcsExecution TA0104 — CIP Stop) | Some("Change Operating Mode") |
| EC-014 | "T0816" (new ICS seeded F2 ENIP — Device Restart/Shutdown, IcsInhibitResponseFunction TA0107 — CIP Reset) | Some("Device Restart/Shutdown") |
| EC-015 | "T1693.001" (new ICS seeded F2 ENIP — Modify Firmware: System Firmware, staged-not-emitted v0.11.0) | Some("Modify Firmware: System Firmware") |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| "T1027" | Some("Obfuscated Files or Information") | happy-path |
| "T1036" | Some("Masquerading") | happy-path |
| "T1071.001" | Some("Web Protocols") | happy-path |
| "T0885" | Some("Commonly Used Port") | happy-path |
| "T0888" | Some("Remote System Information Discovery") | happy-path (new F2) |
| "T0836" | Some("Modify Parameter") | happy-path (new F2) |
| "T0806" | Some("Brute Force I/O") | happy-path (new F2) |
| "T1691.001" | Some("Block Operational Technology Message: Command Message") | happy-path (new F2 DNP3) |
| "T0827" | Some("Loss of Control") | happy-path (new F2 DNP3) |
| "T0830" | Some("Adversary-in-the-Middle") | happy-path (new F2 ARP, ICS) |
| "T1557.002" | Some("Adversary-in-the-Middle: ARP Cache Poisoning") | happy-path (new F2 ARP, Enterprise) |
| "T0858" | Some("Change Operating Mode") | happy-path (new F2 ENIP — CIP Stop, IcsExecution TA0104) |
| "T0816" | Some("Device Restart/Shutdown") | happy-path (new F2 ENIP — CIP Reset, IcsInhibitResponseFunction TA0107) |
| "T1693.001" | Some("Modify Firmware: System Firmware") | happy-path (new F2 ENIP — staged-not-emitted v0.11.0) |
| "T9999" | None | edge-case |
| "" | None | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-007 | All 28 seeded IDs return Some | unit: technique_name_resolves_every_seeded_id |
| VP-007 | Non-seeded IDs return None | unit: technique_name_returns_none_for_unknown_ids |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per domain/capabilities/cap-10-mitre-mapping.md |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per domain/capabilities/cap-10-mitre-mapping.md -- technique_name is the primary lookup function of the MITRE catalog |
| L2 Domain Invariants | INV-9 (MITRE technique ID format) |
| Architecture Module | SS-10 (mitre.rs, C-16) |
| Stories | STORY-071 |
| Origin BC | BC-MIT-005 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.006 -- related to (None case for unknown IDs)
- BC-2.10.007 -- composes with (technique_tactic uses same lookup)
- BC-2.10.008 -- composes with (all emitted IDs must resolve)

## Architecture Anchors

- `src/mitre.rs:128` -- `pub fn technique_info(id: &str)` function declaration
- `src/mitre.rs:129-181` -- static match table (T1027 at :131, T0885 at :158, `_ => return None` at :179; T0830 and T1557.002 arms landed in STORY-114, PR #240)
- `src/mitre.rs` -- T0858 ("Change Operating Mode") arm: PENDING STORY-EIP-09 (new catalog entry; requires `MitreTactic::IcsExecution` variant addition)
- `src/mitre.rs` -- T0816 ("Device Restart/Shutdown") arm: PENDING STORY-EIP-09 (new catalog entry; IcsInhibitResponseFunction TA0107)
- `src/mitre.rs` -- T1693.001 ("Modify Firmware: System Firmware") arm: PENDING future story (staged-not-emitted; replaces revoked T0857)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:128-181` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **assertion**: technique_name_resolves_every_seeded_id

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (static match) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |
