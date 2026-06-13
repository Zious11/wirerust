---
document_type: behavioral-contract
level: L3
version: "1.12"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: "src/mitre.rs (catalog); emission-site evidence spans src/analyzer/tls.rs, src/analyzer/http.rs, src/reassembly/mod.rs, src/reassembly/lifecycle.rs, src/analyzer/modbus.rs (F2)"
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-10
capability: CAP-10
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: Wave 3 Ph3 pass-1 adversarial fix: M-1 add missing T1036 emission site src/reassembly/lifecycle.rs:111; correct total to 10 sites across 4 files; n-3 broaden extracted_from note — 2026-05-22 (product-owner)"
  - "v1.4: DF-SIBLING-SWEEP-001 ADV-IMPL-P03-HIGH-001 re-anchor: mod.rs:442 → mod.rs:471 (T1036 mitre_technique assignment site in check_anomaly_thresholds, shifted by HS-043 merge). — 2026-06-01"
  - "v1.5: ADR-006 / Decision 12+13 (F2 v0.3.0 BREAKING) — emitted-ID set grows from 6->13 (6 Enterprise + 7 ICS); grep pattern updated from 'mitre_technique: Some' to 'mitre_techniques: vec!'; T0888 replaces T0846 as Modbus recon emitter (Decision 12); 7 ICS IDs added to emitted set. ECs and canonical vectors updated. — 2026-06-09"
  - "v1.6: v19 remap: T0855 → T1692.001 per MITRE ATT&CK for ICS v19.0 revocation. All T0855 technique ID references in Description, Postcondition 1 ICS emitted list, EC-007, and Architecture Anchors updated to T1692.001. Tactic unchanged: IcsImpairProcessControl. Issue #222; audit: mitre-ics-v19-catalog-audit.md. — 2026-06-10"
  - "v1.7: Feature #8 DNP3 analyzer (F2). Added 2 new ICS emitted techniques: T1691.001 (Block Operational Technology Message: Command Message — DNP3 inferred block-command, IcsInhibitResponseFunction) + T0827 (Loss of Control — DNP3 derived correlated finding, IcsImpact). EMITTED count 13→15 (6 Enterprise + 9 ICS). Description, emission sites, Postcondition 1, Invariant 1, EC-014 and EC-015 added. — 2026-06-10"
  - "v1.8: Pass-1 adversarial fix C-1: corrected T1691.001 technique name from fabricated 'Unauthorized Message: Inhibit Response Function' to authoritative 'Block Operational Technology Message: Command Message' in changelog v1.7 and EC-014. — 2026-06-10"
  - "v1.9: Feature #9 ARP analyzer (F2). Added 2 new emitted techniques: T0830 (ICS: Adversary-in-the-Middle, LateralMovement) + T1557.002 (Enterprise: Adversary-in-the-Middle: ARP Cache Poisoning, CredentialAccess). EMITTED count 15→17 (7 Enterprise + 10 ICS). Description, emission sites, Postcondition 1, Invariant 1, EC-016 and EC-017 added. — 2026-06-12 (F-D-C1 pass-2 remediation)"
  - "v1.10: Pass-3 remediation F-C3/F-C4/F-C6/F-C1(b): EC-017 technique_name corrected to 'Adversary-in-the-Middle: ARP Cache Poisoning' (authoritative name from arch-delta §5 + mitre-arp-research.md); T1557.002 reclassified Enterprise (not ICS); Enterprise/ICS split corrected 6E+11I→7E+10I; Architecture Anchors re-anchored to current mitre.rs line numbers (T0885:158, _ => return None:179); 'all 13 emitted IDs' corrected to '17 emitted IDs'; PLANNED forward-declaration marker added. — 2026-06-12"
  - "v1.11: Pass-4 remediation F-C-P4-HIGH-002/F-D4-I2: Description reconciliation parenthetical added (pre-F2: 6E; Modbus: 7I; DNP3: +2I; ARP: +1E+1I → 7E+10I=17); PLANNED marker augmented with current→target values (23/15→25/17); Source Evidence path corrected 123-154→128-181. — 2026-06-12"
  - "v1.12: Pass-10 remediation F-C-P10-003: src/analyzer/arp.rs emission bullet lead-in changed from 'verified via grep' (implied current) to explicit PLANNED qualifier — 'Emission sites after F2 ARP (Modbus/DNP3 verified via grep; arp.rs PLANNED STORY-114)'; arp.rs bullet appended '(F2 Feature #9 PLANNED — STORY-114)'. arp.rs does not exist in develop HEAD until STORY-114 lands. — 2026-06-12"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.008: All Emitted Technique IDs Resolve in Lookup

<!--
  PREVIOUS VERSION SUMMARY (v1.4 -> v1.5):
  Grep pattern changed: 'mitre_technique: Some' -> 'mitre_techniques: vec!'
  Emitted-ID set expanded: 6 Enterprise -> 13 (6 Enterprise + 7 ICS)
  T0846 removed from emitted set; T0888 added (Decision 12 correctness fix)
  New ICS emitted IDs added: T0855, T0836, T0814, T0806, T0835, T0831, T0888
  Invariant 1: subset claim updated (13 of 21)
  Invariant 3: grep pattern updated
  EC-007 through EC-013 added for new ICS emitted IDs
  Canonical vectors extended with new ICS IDs
-->

## Description

Every technique ID that any analyzer or reassembly engine emits in `Finding.mitre_techniques`
must resolve to `Some(...)` when passed to `technique_name` or `technique_tactic`. After F2
(Feature #7 Modbus + Feature #8 DNP3 + Feature #9 ARP), the emitted-ID set grows to 17
distinct IDs: 7 Enterprise + 10 ICS. Reconciliation:
(pre-F2: 6 Enterprise; Modbus F2: 7 ICS; DNP3 F2: +2 ICS [T1691.001, T0827]; ARP F2: +2 = 1 Enterprise [T1557.002] + 1 ICS [T0830]) → 7 Enterprise + 10 ICS = 17.
No emitted ID may return None from the lookup — that would cause the terminal reporter to display
`<id> (unknown)` for a Finding produced by current analyzers.

PLANNED — implemented in STORY-114; current code 23 seeded / 15 emitted → target 25 seeded / 17 emitted after STORY-114 5-part atomic update. src/mitre.rs remains at SEEDED=23/EMITTED=15
until STORY-114 lands; vp007_catalog_drift_guard enforces consistency at implementation time.

Emission sites after F2 ARP (Modbus/DNP3 verified via grep; arp.rs PLANNED STORY-114):
- `src/analyzer/tls.rs` — `vec!["T1027"]` x3
- `src/analyzer/http.rs` — `vec!["T1083"]`, `vec!["T1505.003"]`, `vec!["T1046"]`, `vec!["T1499.002"]` x2
- `src/reassembly/mod.rs` — `vec!["T1036"]`
- `src/reassembly/lifecycle.rs` — `vec!["T1036"]`
- `src/analyzer/modbus.rs` (F2 Feature #7) — `vec!["T1692.001","T0836"]`, `vec!["T1692.001","T0835"]`,
  `vec!["T1692.001"]`, `vec!["T0806","T1692.001"]`, `vec!["T0814"]`, `vec!["T1692.001","T0836","T0831"]`, `vec!["T0888"]`
- `src/analyzer/dnp3.rs` (F2 Feature #8 new) — `vec!["T1692.001"]` (control threshold),
  `vec!["T0814"]` (restart DoS), `vec!["T0836"]` (write FC), `vec!["T1691.001"]` (block-command
  inferred, BC-2.15.014), `vec!["T0827"]` (derived loss-of-control, BC-2.15.015)
- `src/analyzer/arp.rs` (F2 Feature #9 PLANNED — STORY-114) — `vec!["T0830","T1557.002"]` (D1 spoof, D2 GARP,
  D12 mismatch paths; see BC-2.16.003, BC-2.16.004, BC-2.16.007, BC-2.16.014)

The emitted-ID set is 17 distinct IDs. Multi-element vecs contribute multiple IDs per emission;
all IDs in all vecs must resolve.

## Preconditions

1. `technique_name` or `technique_tactic` is called with one of the 17 emitted IDs.

## Postconditions

1. All 17 currently-emitted distinct IDs return `Some(...)`:
   - Enterprise (7): T1027, T1036, T1046, T1083, T1499.002, T1505.003, T1557.002
   - ICS (10): T1692.001, T0836, T0814, T0806, T0835, T0831, T0888, T1691.001, T0827, T0830
2. None of the 17 emitted IDs returns None.

## Invariants

1. The emitted set (17 IDs: 7 Enterprise + 10 ICS) is a strict subset of the catalogued set (25 IDs).
2. The invariant is enforced by convention: when an analyzer adds a new emission site, the
   developer must add the ID to `technique_info` first (or simultaneously). For multi-element
   `mitre_techniques` vecs, EVERY element must resolve.
3. The authoritative list of emitted IDs is `grep -rn 'mitre_techniques: vec!' src/` per
   mitre.rs comment (updated from pre-F2 `grep -rn 'mitre_technique: Some' src/`).
4. T0846 is SEEDED but NOT EMITTED. It was the intended Modbus recon emitter in pre-F2 plans
   but was corrected to T0888 (Remote System Information Discovery) per Decision 12. T0846
   remains catalogued for future use.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | T1027 (TLS SNI control-byte) | Some("Obfuscated Files or Information") |
| EC-002 | T1036 (TCP conflicting overlap) | Some("Masquerading") |
| EC-003 | T1046 (HTTP admin panel) | Some("Network Service Discovery") |
| EC-004 | T1083 (HTTP path traversal) | Some("File and Directory Discovery") |
| EC-005 | T1499.002 (HTTP too-many-headers) | Some("Service Exhaustion Flood") |
| EC-006 | T1505.003 (HTTP web shell) | Some("Web Shell") |
| EC-007 | T1692.001 (Modbus: unauthorized command, present in all write-class PDU findings; ICS sub-technique, v19 successor to revoked T0855) | Some("Unauthorized Message: Command Message") |
| EC-008 | T0836 (Modbus: register write — Modify Parameter) | Some("Modify Parameter") |
| EC-009 | T0814 (Modbus: Force Listen Only / Restart Comms) | Some("Denial of Service") |
| EC-010 | T0806 (Modbus: write burst or sustained rate) | Some("Brute Force I/O") |
| EC-011 | T0835 (Modbus: coil write — I/O Image) | Some("Manipulate I/O Image") |
| EC-012 | T0831 (Modbus: coordinated write sequence) | Some("Manipulation of Control") |
| EC-013 | T0888 (Modbus: recon FCs 0x11, 0x2B/0x0E) | Some("Remote System Information Discovery") |
| EC-014 | T1691.001 (DNP3: inferred block-command, control request without response; ICS sub-technique, v19) | Some("Block Operational Technology Message: Command Message") |
| EC-015 | T0827 (DNP3: derived loss-of-control correlated finding — N restart/block events in window) | Some("Loss of Control") |
| EC-016 | T0830 (ARP: D1 spoof and D12 mismatch paths; ICS Adversary-in-the-Middle, LateralMovement) | Some("Adversary-in-the-Middle") |
| EC-017 | T1557.002 (ARP: D1 spoof and D2 GARP-that-conflicts paths; Enterprise Adversary-in-the-Middle: ARP Cache Poisoning, CredentialAccess) | Some("Adversary-in-the-Middle: ARP Cache Poisoning") |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| technique_name("T1027") | Some("Obfuscated Files or Information") | happy-path |
| technique_name("T1505.003") | Some("Web Shell") | happy-path |
| technique_name("T1499.002") | Some("Service Exhaustion Flood") | happy-path |
| technique_name("T0888") | Some("Remote System Information Discovery") | happy-path (ICS, F2) |
| technique_name("T0836") | Some("Modify Parameter") | happy-path (ICS, F2) |
| technique_name("T0806") | Some("Brute Force I/O") | happy-path (ICS, F2) |
| technique_name("T1691.001") | Some("Block Operational Technology Message: Command Message") | happy-path (ICS, F2 DNP3) |
| technique_name("T0827") | Some("Loss of Control") | happy-path (ICS, F2 DNP3) |
| technique_name("T0830") | Some("Adversary-in-the-Middle") | happy-path (ICS, F2 ARP) |
| technique_name("T1557.002") | Some("Adversary-in-the-Middle: ARP Cache Poisoning") | happy-path (Enterprise, F2 ARP) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-007 | All 17 emitted IDs resolve in technique_name | unit: test each emitted ID |
| VP-007 | No new emission site uses an ID not in technique_info | manual: code review of analyzer PRs; every element in mitre_techniques vec must resolve |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per domain/capabilities/cap-10-mitre-mapping.md |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per domain/capabilities/cap-10-mitre-mapping.md -- emitted-ID resolution is the end-to-end correctness property of the MITRE mapping capability |
| L2 Domain Invariants | INV-9 (MITRE technique ID format) |
| Architecture Module | SS-10 (mitre.rs, C-16) |
| Stories | STORY-071 |
| Origin BC | BC-MIT-008 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.005 -- composes with (catalog lookup is the mechanism)
- BC-2.10.007 -- composes with (tactic resolution also applies to emitted IDs)

## Architecture Anchors

- `src/mitre.rs:128` -- `pub fn technique_info(id: &str)` function declaration
- `src/mitre.rs:129-181` -- technique_info match table covering all 17 emitted IDs (T0885 at :158; `_ => return None` at :179; T0830 and T1557.002 arms PLANNED in STORY-114 — not yet in source)
- Emitted sites (pre-F2 baseline; F2 sites to be added at implementation):
  - `src/analyzer/tls.rs:443` (T1027), `src/analyzer/tls.rs:463` (T1027), `src/analyzer/tls.rs:483` (T1027)
  - `src/analyzer/http.rs:198` (T1083), `src/analyzer/http.rs:228` (T1505.003), `src/analyzer/http.rs:244` (T1046), `src/analyzer/http.rs:423` (T1499.002), `src/analyzer/http.rs:482` (T1499.002)
  - `src/reassembly/mod.rs:471` (T1036)
  - `src/reassembly/lifecycle.rs:111` (T1036)
  - `src/analyzer/modbus.rs` — multiple sites (T1692.001, T0836, T0814, T0806, T0835, T0831, T0888; exact lines TBD at F3 implementation)
  - `src/analyzer/arp.rs` (F2 Feature #9 PLANNED in STORY-114) — `vec!["T0830","T1557.002"]` (D1 spoof, D2 GARP, D12 mismatch paths)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:128-181` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: technique_name_resolves_every_seeded_id covers the emitted subset
- **documentation**: mitre.rs comment "grep -rn 'mitre_techniques: vec!' src/" (updated from pre-F2 'mitre_technique: Some')

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

The grep pattern for emitted-ID discovery has changed from `grep -rn 'mitre_technique: Some' src/`
(pre-F2) to `grep -rn 'mitre_techniques: vec!' src/` (post-F2 ADR-006). The mitre.rs
VP-007 comment must be updated at implementation time. Multi-element vec sites contribute
multiple distinct IDs per call; all must be checked against the catalog.
