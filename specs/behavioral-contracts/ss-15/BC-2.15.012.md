---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-06-10T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-15
capability: CAP-15
lifecycle_status: active
introduced: v0.6.0-feature-008
modified:
  - "v1.3: F3 story-anchor back-fill. — 2026-06-14"
  - "v1.4: F3-convergence FC-name normalization — changed FC 0x13 label from SAVE_CONFIG to SAVE_CONFIGURATION (IEEE 1815-2012 canonical name) in body line ~71 and canonical test vector table line ~102. Sibling FCs 0x14/0x15 already unabbreviated (ENABLE_UNSOLICITED/DISABLE_UNSOLICITED). No shipped SAVE_CONFIG symbol in src/ (grep confirmed zero hits). Behavioral classification (Management/out-of-scope for v1) unchanged. — 2026-06-14"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/dnp3-architecture-delta.md
  - .factory/research/dnp3-research.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
input-hash: TBD
---

# BC-2.15.012: WRITE FC Observed — Emits T0836 Modify-Parameter Finding Per-Occurrence

## Description

When a DNP3 application function code WRITE (0x02) is observed on a FIR=1 fragment, a
`Finding` is emitted immediately carrying `T0836` ("Modify Parameter"). WRITE commands
change setpoints, limits, or configuration stored in the outstation's data objects — a
common attacker technique to corrupt the process safety envelope without triggering alarms.
Detection is per-occurrence: one finding per observed WRITE FC. ADR-007 Decision 5.

## Preconditions

1. The validity gate (BC-2.15.004) returned `true`.
2. `has_user_data(control)` is `true`.
3. `transport_is_fir(transport_octet)` is `true` (FIR=1, BC-2.15.008).
4. `classify_dnp3_fc(app_fc)` returns `Dnp3FcClass::Write` (BC-2.15.006: FC 0x02 only).
5. `flow.is_non_dnp3 == false`.
6. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

1. Exactly ONE `Finding` is pushed to `self.all_findings`:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::Medium`
   - `summary`: `"DNP3 WRITE command observed: parameter modification from src={src:#06X} to dest={dest:#06X}"`
   - `evidence`: one entry — `"FC=0x02 (WRITE) dest={dest:#06X} src={src:#06X}"`
   - `mitre_techniques: vec!["T0836"]`
   - `source_ip: Some(<source endpoint>)` — resolved from flow_key
   - `timestamp: Some(...)` — pcap-relative capture timestamp
2. `flow.fc_counts.entry(0x02).or_insert(0) += 1`.
3. `self.fn_code_counts.entry(0x02).or_insert(0) += 1`.

## Invariants

1. **Per-occurrence detection**: one T0836 finding per observed WRITE FC. WRITE commands are
   stealthy (often single occurrences) and individually significant. No burst threshold. [ADR-007 Decision 5]
2. **T0836 is the correct v19.1 technique** [MITRE: dnp3-research.md §6]: T0836 "Modify
   Parameter" is active and unchanged in ics-attack-19.1. Tactic: IcsImpairProcessControl.
3. **FC 0x02 only**: only WRITE maps to `Dnp3FcClass::Write`. Other write-like operations
   (IMMED_FREEZE, SAVE_CONFIGURATION) map to `Management` and do NOT trigger T0836 in v1.
4. **No T1692.001 co-tag on WRITE**: unlike Modbus where write-class FCs get both T1692.001
   and T0836, DNP3 WRITE emits T0836 only. T1692.001 is emitted by Control-class FCs
   (BC-2.15.010). This separation is intentional — DNP3's WRITE is a configuration/setpoint
   operation distinct from real-time actuation commands. [ADR-007 Decision 5 detection table]

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First WRITE on a new flow | Finding: T0836, confidence=Medium |
| EC-002 | Second WRITE (same flow, same window) | Second T0836 finding emitted (no dedup in v1) |
| EC-003 | `all_findings.len() == MAX_FINDINGS` | No finding pushed; FC counter still incremented |
| EC-004 | FC 0x14 (ENABLE_UNSOLICITED) — config-change adjacent | NOT T0836; `classify_dnp3_fc(0x14)` = `Management`; no finding |
| EC-005 | WRITE to broadcast destination | T0836 emitted normally (broadcast WRITE is even more anomalous; v1 does not add extra tag) |
| EC-006 | WRITE (0x02) in response direction (outstation→master) | Response direction WRITE is unusual; detection applies regardless of DIR bit in v1 — the FC drives detection, not direction |

## Canonical Test Vectors

**WRITE frame (outstation 3, master 1):**
```
DNP3 link frame:  05 64 0E C4 03 00 01 00 [hdr-crc]  C0 81 02 [app-objects]  [data-crc]
Transport:        0xC0 (FIR=1, FIN=1)
App FC:           0x02 → WRITE → Dnp3FcClass::Write
```
Expected: `Finding { mitre_techniques: ["T0836"], confidence: Medium, summary: "DNP3 WRITE command observed: parameter modification from src=0x0001 to dest=0x0003" }`

| FC (hex) | Name | Expected `Finding.mitre_techniques` |
|----------|------|-------------------------------------|
| `0x02` | WRITE | `["T0836"]` |
| `0x14` | ENABLE_UNSOLICITED | (no T0836 finding) |
| `0x13` | SAVE_CONFIGURATION | (no T0836 finding; Management class) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-023 | Sub-property B (correctness): `classify_dnp3_fc(0x02)` returns `Write` | Kani (Sub-B set membership) |
| (none) | Per-occurrence finding: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — detecting WRITE-command parameter modification is a core capability of the DNP3/ICS analyzer; T0836 is the MITRE technique for attackers writing arbitrary values to ICS device setpoints, limits, or configuration |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — findings emitted only on valid DNP3 port-20000 flows) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24); ADR-007 Decision 5 |
| Stories | STORY-108 |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | T0836 — Modify Parameter (ICS; Impair Process Control tactic TA0106; active in v19.1) |

## Related BCs

- BC-2.15.006 — depends on (Write-class FC classification)
- BC-2.15.008 — depends on (FIR=1 gate)
- BC-2.15.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `Dnp3Analyzer::on_data` — Write-class branch
- `src/mitre.rs` — `technique_info("T0836")` arm (existing; shared with Modbus WRITE FCs)
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §8` (detection table: "WRITE 0x02 → T0836")
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 5`

## Story Anchor

STORY-108

## VP Anchors

- VP-023 — Sub-property B (verifies Write-class classification precondition)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-007 Decision 5; dnp3-research.md §3.2 (WRITE=0x02 [SPEC]); §5 (T0836 mapping: "WRITE 0x02 → T0836 Modify Parameter, confirmed active in v19.1") |
| **Confidence** | high — T0836 confirmed [MITRE] active; FC 0x02 confirmed [SPEC] |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings, fc_counts, fn_code_counts |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell |
