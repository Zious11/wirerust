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
  - "v1.3: Wave 3 Ph3 pass-1 adversarial fix: m-1 correct technique_info line-anchor range to :122-156 (closing brace verified) in Architecture Anchors and Source Evidence — 2026-05-22 (product-owner)"
  - "v1.4: ADR-006 / Decision 12+13 (F2 v0.3.0) — SEEDED count updated 15 -> 21 (added 6 ICS: T0836,T0814,T0806,T0835,T0831,T0888; T0846 kept seeded, not Modbus-emitted; T0888 replaces T0846 in Modbus recon emission). Postconditions, Invariants, canonical vectors updated. — 2026-06-09"
  - "v1.5: v19 remap: T0855 → T1692.001, T0856 → T1692.002 per MITRE ATT&CK for ICS v19.0 revocation. ICS seeded ID list and Postcondition 1 updated. Seeded count remains 21. Issue #222; audit: mitre-ics-v19-catalog-audit.md. — 2026-06-10"
  - "v1.6: Feature #8 DNP3 analyzer (F2). Added 2 new ICS techniques: T1691.001 (Block Operational Technology Message: Command Message, IcsInhibitResponseFunction) + T0827 (Loss of Control, IcsImpact). SEEDED count 21→23 (11 Enterprise + 12 ICS). H1 title updated 21→23. — 2026-06-10"
  - "v1.7: Pass-1 adversarial fix C-1: corrected T1691.001 technique name from fabricated 'Unauthorized Message: Inhibit Response Function' to authoritative 'Block Operational Technology Message: Command Message' (parent T1691, tactic IcsInhibitResponseFunction) in changelog v1.6, EC-009, and canonical test vectors. Fixed duplicate invariant numbering (I-6): second invariant 4 renumbered to 5. — 2026-06-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.005: technique_name Returns Some for Every Seeded ID (23 Total)

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
-->

## Description

`technique_name(id: &str)` returns `Some(&'static str)` for all 23 technique IDs present in
the `technique_info` static match table. IDs not in the table return `None`. The 23-entry
catalog (post-F2 DNP3) includes 15 IDs emitted by analyzers and 8 staged IDs for future
analyzers. The catalog grows from 21 (post-Feature #7 Modbus, 11 Enterprise + 10 ICS) to 23
(11 Enterprise + 12 ICS) as part of Feature #8 (DNP3 analyzer, ADR-007).

## Preconditions

1. `technique_name` is called with a string argument.

## Postconditions

1. For each of the 23 seeded IDs, returns `Some(technique_name_string)`.
2. For any other string, returns `None`.
3. The 23 seeded IDs are:
   - Enterprise (11): T1027, T1036, T1040, T1046, T1071, T1071.001, T1071.004,
     T1083, T1499.002, T1505.003, T1573
   - ICS (12): T0846, T1692.001, T1692.002, T0885, T0836, T0814, T0806, T0835, T0831, T0888,
     T1691.001, T0827

## Invariants

1. IDs currently emitted (15): 6 Enterprise (T1027, T1036, T1046, T1083, T1499.002,
   T1505.003) + 9 ICS (T1692.001, T0836, T0814, T0806, T0835, T0831, T0888, T1691.001, T0827).
2. IDs catalogued but not emitted (8): T1040, T1071, T1071.001, T1071.004, T1573, T0846,
   T1692.002, T0885. T0846 was previously the Modbus recon technique but was corrected to T0888
   per Decision 12; T0846 remains seeded for future use (e.g., address-sweep detection).
   T1692.002 replaces revoked T0856 ("Spoof Reporting Message") per ATT&CK-ICS v19 remap.
3. The catalog count is 23 after Feature #8 DNP3 (F2). Post-Feature #7 count was 21.
   Any claim of 21 post-DNP3 is an error; any claim of 22 is an error (23 = 11 Enterprise + 12 ICS).
4. Arithmetic check: SEEDED=23, EMITTED=15, CATALOGUE-ONLY=23−15=8. These counts are
   mutually consistent.
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
| "T9999" | None | edge-case |
| "" | None | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-007 | All 23 seeded IDs return Some | unit: technique_name_resolves_every_seeded_id |
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

- `src/mitre.rs:122` -- `pub fn technique_info(id: &str)` function declaration
- `src/mitre.rs:123-156` -- static match table (T1027 at :125, T0885 at :152, `_ => return None` at :153, closing brace at :156)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:122-156` |
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
