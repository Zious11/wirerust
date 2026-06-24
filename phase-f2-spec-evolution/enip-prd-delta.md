---
document_type: prd-delta
feature: feature-enip-v0.11.0
issue: "#316"
prd_version_before: "1.35"
prd_version_after: "1.36"
bc_index_version_before: "1.73"
bc_index_version_after: "1.74"
timestamp: 2026-06-24T00:00:00Z
producer: product-owner
phase: f2
adr: ADR-010
vp: VP-032
---

# EtherNet/IP + CIP PRD Delta — Feature Mode F2

## Summary

This delta records the additions to the PRD for the EtherNet/IP + CIP analyzer
(feature-enip-v0.11.0, issue #316, v0.11.0). 24 new BCs (BC-2.17.001..024) are
integrated under the new subsystem SS-17 (CAP-17).

## PRD Changes Applied

### §2 Behavioral Contracts — New Section Added

**Section `§2.17 EtherNet/IP + CIP Analysis (CAP-17)`** added after §2.16.
See prd.md §2.17 for the full section including release notes, MITRE analysis, CLI flags,
formal verification note, and grouped BC tables.

### §7 Requirements Traceability Matrix — 24 New Rows Added

24 RTM rows added for BC-2.17.001..024 mapping to CAP-17 / SS-17 / analyzer/enip.rs.
See prd.md §7 for the updated RTM.

### §1.3 Competitive Differentiators — Traceability Update

KD-005 (ICS/OT Protocol Deep Inspection) extended: BC-2.17.010..015/018 trace to this
differentiator (EtherNet/IP + CIP adds T0858/T0816/T0836/T0846/T0888/T0814 to the
ICS detection surface alongside existing Modbus/DNP3 coverage).

### §8 Domain Debt Index — O-04 Updated

O-04 SEEDED/EMITTED counts updated: SEEDED grows 25→28 (new T0858 + T0816 + T1693.001
seeded but not emitted; T1693.001 is future staged firmware detection per BC-2.17.007
GetAndClear note). EMITTED grows 17→19 (T0858 + T0816 now emitted by ENIP analyzer).
CATALOGUE-ONLY drops 8→7 (T0858 and T0816 move from catalogue-only to emitted;
T1693.001 added as catalogue-only). Note: BC-2.10.005 and BC-2.10.008 must be updated
in the next BC version-bump pass.

## Traceability Summary

| CAP ID | Subsystem | BC Range | VP | ADR | Feature |
|--------|-----------|----------|----|-----|---------|
| CAP-17 | SS-17 | BC-2.17.001..024 | VP-032 | ADR-010 | feature-enip-v0.11.0 (issue #316) |

## MITRE Catalog Delta

| Technique | Name | Tactic | Status | Catalog Action |
|-----------|------|--------|--------|----------------|
| T0858 | Change Operating Mode | IcsExecution (TA0104) | new → emitted | Add `technique_info("T0858")` arm + `MitreTactic::IcsExecution` variant |
| T0816 | Device Restart/Shutdown | IcsInhibitResponseFunction (TA0107) | new → emitted | Add `technique_info("T0816")` arm in src/mitre.rs |
| T0836 | Modify Parameter | IcsImpairProcessControl (TA0105) | already seeded | No catalog change |
| T0846 | Remote System Discovery | IcsDiscovery (TA0102) | already seeded | No catalog change |
| T0888 | Remote System Information Discovery | IcsDiscovery (TA0102) | already seeded | No catalog change |
| T0814 | Denial of Service | IcsInhibitResponseFunction (TA0107) | already seeded | No catalog change |
| T1693.001 | Modify Firmware: System Firmware | (future) | staged not emitted | Seed but do not emit in v0.11.0; trigger: CIP firmware download service (0x4B or vendor-specific); staged per ADR-010 Decision 7 |

## Open Items

| ID | Description | Owner | Status |
|----|-------------|-------|--------|
| OA-001 | --enip-write-burst-threshold default (20 writes/1s) needs human confirmation for high-write CIP environments. BC-2.17.012 (confidence: medium) and BC-2.17.023 flag this. | Human | OPEN |

## Domain Capability Registration

CAP-17 registered in `.factory/specs/domain/capabilities/cap-17-enip-cip-analysis.md`.

## Architect Verification

ARCH-INDEX v1.7 (SS-17 Subsystem Registry entry, ADR-010 Architecture Decisions row,
Bounded-Resource Design note) was produced by architect in the create sub-burst.
No ARCH-INDEX changes needed from this integrate sub-burst.

VP-INDEX v2.11 (VP-032 added, Kani count 14→15, total 31→32) was produced by architect
in the create sub-burst. No VP-INDEX changes needed from this integrate sub-burst.
