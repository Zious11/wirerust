---
document_type: domain-capability
capability_id: CAP-17
title: "EtherNet/IP + CIP Analysis"
subsystem: SS-17
feature: feature-enip-v0.11.0
issue: "#316"
adr: ADR-010
introduced: v0.11.0
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
---

# CAP-17: EtherNet/IP + CIP Analysis

## Description

wirerust detects adversarial abuse of the EtherNet/IP (ENIP) and Common Industrial Protocol
(CIP) stack over TCP/44818. The analyzer performs two-level binary parsing: ENIP
encapsulation header (24-byte fixed, little-endian) and CPF item layer leading to CIP service
extraction. Detection covers six MITRE ATT&CK for ICS techniques: T0858 (CIP Stop — Change
Operating Mode), T0816 (CIP Reset — Device Restart/Shutdown), T0836 (CIP write-class burst
— Modify Parameter), T0846 (ListIdentity — Remote System Discovery), T0888 (CIP Identity
Object read / error burst — Remote System Information Discovery), and T0814 (malformed ENIP
frame threshold — Denial of Service). ForwardOpen connection-lifecycle anomalies are also
detected with an empty MITRE technique set per ADR-010 Decision 7 policy.

## Behavioral Contracts

BC-2.17.001 through BC-2.17.024 (24 BCs; see behavioral-contracts/ss-17/).

## Subsystem

SS-17: `analyzer/enip.rs` (EnipAnalyzer, EnipFlowState, parse_enip_header,
classify_enip_command, is_valid_enip_frame, classify_cip_service, parse_cip_header,
parse_cip_request_path, CPF item-walk).

## Scope Boundaries

- **In scope (v0.11.0):** TCP/44818 explicit messaging; ENIP encapsulation header;
  CPF item layer; CIP service extraction; 6 MITRE detections.
- **Deferred:** UDP/2222 implicit I/O; T1693.001 firmware detection (GetAndClear staged,
  not emitted); cross-BC ForwardOpen+Stop correlation (T1692.001 on ForwardOpen).

## MITRE Techniques

| Technique | Name | Tactic | Status |
|-----------|------|--------|--------|
| T0858 | Change Operating Mode | IcsExecution (TA0104) | new in v0.11.0 |
| T0816 | Device Restart/Shutdown | IcsInhibitResponseFunction (TA0107) | new in v0.11.0 |
| T0836 | Modify Parameter | IcsImpairProcessControl (TA0105) | already seeded |
| T0846 | Remote System Discovery | IcsDiscovery (TA0102) | already seeded |
| T0888 | Remote System Information Discovery | IcsDiscovery (TA0102) | already seeded |
| T0814 | Denial of Service | IcsInhibitResponseFunction (TA0107) | already seeded |
| T1693.001 | (GetAndClear firmware) | (future) | staged, not emitted |

## Open Items

- **OA-001:** --enip-write-burst-threshold default (20 writes/1s) requires human confirmation
  for high-write CIP environments. See BC-2.17.012 and BC-2.17.023.
