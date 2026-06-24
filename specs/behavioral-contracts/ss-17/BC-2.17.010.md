---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-17
capability: CAP-17
lifecycle_status: active
introduced: v0.11.0-feature-enip
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
  - .factory/research/enip-mitre-ics-tagging.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/specs/verification-properties/vp-032-enip-parse-safety.md
input-hash: TBD
---

# BC-2.17.010: ListIdentity Command Observed Emits T0846 Network Enumeration Finding

## Description

When `classify_enip_command(header.command)` returns `EnipCommandClass::ListIdentity`
(command 0x0063), a `Finding` is emitted carrying `T0846` ("Remote System Discovery").
`ListIdentity` is the ENIP broadcast/unicast command used to enumerate all EtherNet/IP
devices on a network segment — it requests device identification information (vendor ID,
product type, product code, revision, serial number, product name) from all responsive
devices. This is the network-wide enumeration detection; single-device identity reads
(GetAttributeSingle/All targeting Identity Object) are covered by BC-2.17.014 (T0888).
Detection is per-occurrence (per ListIdentity frame seen on the flow).

## Preconditions

1. `classify_enip_command(header.command)` returns `EnipCommandClass::ListIdentity`.
2. `flow.is_non_enip == false`.
3. `self.all_findings.len() < MAX_FINDINGS` (checked before pushing finding only).

## Postconditions

1. `flow.command_counts.entry(0x0063).or_insert(0) += 1` (always, on every ListIdentity frame).
2. If `flow.list_identity_emitted == false` AND `self.all_findings.len() < MAX_FINDINGS`:
   - Push exactly ONE `Finding`:
     - `category: ThreatCategory::Reconnaissance`
     - `verdict: Verdict::Likely`
     - `confidence: Confidence::High`
     - `summary: "EtherNet/IP ListIdentity broadcast observed: network-wide device enumeration (T0846)"`
     - `evidence`: one entry — `"ENIP command=0x0063 (ListIdentity) src={src_ip} session={session_handle}"`
     - `mitre_techniques: vec!["T0846"]`
     - `source_ip: Some(<source endpoint>)` — resolved from flow_key
     - `timestamp: Some(...)` — pcap-relative capture timestamp
   - `flow.list_identity_emitted = true` (one-shot guard).
3. If `flow.list_identity_emitted == true`: `command_counts` updated; NO additional finding.

## Invariants

1. **Per-flow one-shot guard**: to prevent a single ListIdentity scan campaign from emitting
   up to MAX_FINDINGS near-identical T0846 findings on one flow, a per-flow one-shot guard
   (`flow.list_identity_emitted: bool`) is applied. The first ListIdentity frame per flow
   emits a T0846 finding and sets the guard; subsequent frames on the same flow increment
   `command_counts[0x0063]` but do not emit additional findings. The finding's evidence field
   includes the final `command_counts[0x0063]` value in `summarize()` for audit.
   Choice of one-shot-per-flow (over windowed) reflects that a scan is typically a single
   campaign per source; a new flow from the same source would reset the guard. [MEDIUM-confidence
   — un-calibrated; may be revisited in F3 if human feedback indicates windowed is preferable]
2. **T0846 is the correct v19.1 technique** [MITRE: enip-mitre-ics-tagging.md §4b]:
   T0846 "Remote System Discovery" (IcsDiscovery, TA0102) — ListIdentity is explicitly the
   network-wide device enumeration mechanism. Already seeded in `src/mitre.rs`; no new
   catalog entry required.
3. **High confidence**: ListIdentity is an explicit discovery broadcast. There is no
   legitimate reason to send this command in normal production operations (only during
   commissioning or troubleshooting).
4. **Distinct from T0888**: T0846 is for network-wide enumeration (ListIdentity broadcast
   returning a list of devices by IP). T0888 is for single-device profiling (GetAttributeSingle
   to Identity Object targeting a specific device). These are complementary and independent.
5. **ENIP encapsulation layer detection**: ListIdentity is detected at the ENIP command
   layer (before CPF parse). No CIP payload inspection is needed.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Single ListIdentity frame on a new flow | One T0846 finding emitted; `command_counts[0x0063] = 1`; `list_identity_emitted = true` |
| EC-002 | Multiple ListIdentity frames in sequence (same flow) | First frame: T0846 finding + guard set. Subsequent frames: `command_counts[0x0063]` incremented; NO additional findings (one-shot guard) |
| EC-003 | `all_findings.len() == MAX_FINDINGS` when ListIdentity arrives (guard false) | No finding pushed; command_counts still updated; guard remains false |
| EC-004 | ListIdentity with `session_handle = 0` (normal — no session needed) | Finding emitted; session_handle noted in evidence |
| EC-005 | ListIdentity followed by GetAttributeSingle to Identity Object | Two separate findings: T0846 (BC-2.17.010) + T0888 (BC-2.17.014); combined recon pattern |

## Canonical Test Vectors

**ListIdentity ENIP frame:**
```
ENIP header (hex): 63 00 04 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
command: 0x0063 (ListIdentity), length: 4, session: 0, status: 0
```
Expected: `Finding { mitre_techniques: ["T0846"], verdict: Likely, confidence: High,
summary: "EtherNet/IP ListIdentity broadcast observed: network-wide device enumeration (T0846)" }`

| Event | command_counts[0x0063] | Findings emitted | list_identity_emitted |
|-------|----------------------|-----------------|----------------------|
| 1st ListIdentity | 1 | 1 (T0846) | true (guard set) |
| 2nd ListIdentity | 2 | 0 (guard blocks) | true (unchanged) |
| 3rd ListIdentity | 3 | 0 (guard blocks) | true |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-032 | Sub-B (indirect): `classify_enip_command(0x0063)` returns `ListIdentity` — precondition verified | Kani Sub-B totality proof |
| (none) | Per-occurrence finding emission: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — detecting EtherNet/IP ListIdentity is a core ICS threat-detection requirement: this command is the canonical network-wide OT device enumeration primitive, routinely used in ICS attacks (e.g., TRITON/TRISIS initial reconnaissance) to identify targets before deploying payloads; T0846 (ics-attack-19.1) maps directly to this behavior |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 7 (T0846 technique mapping) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | T0846 — Remote System Discovery (ICS Discovery, TA0102; active in ics-attack-19.1; already seeded in src/mitre.rs; no new catalog entry required) |

## Related BCs

- BC-2.17.004 — depends on (ListIdentity classification via classify_enip_command Sub-B)
- BC-2.17.014 — composes with (T0888 single-device identity read is the follow-on recon)
- BC-2.17.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/enip.rs` — `process_pdu`: `if matches!(cmd_class, EnipCommandClass::ListIdentity) { flow.command_counts[0x0063] += 1; if !flow.list_identity_emitted { /* emit T0846 */ flow.list_identity_emitted = true; } }`
- `src/analyzer/enip.rs` — `EnipFlowState.command_counts: HashMap<u16, u64>`
- `src/analyzer/enip.rs` — `EnipFlowState.list_identity_emitted: bool` (per-flow one-shot guard for T0846)
- `src/mitre.rs` — `technique_info("T0846")` arm (existing; shared)
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 7` (T0846 active technique)
- `.factory/research/enip-mitre-ics-tagging.md §4b` (T0846 ListIdentity mapping)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-032 Sub-B (indirect — verifies classify_enip_command precondition)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 7 (T0846 active technique); enip-mitre-ics-tagging.md §4b (T0846 ListIdentity: "T0846 for network-wide ListIdentity broadcast/multicast enumeration that returns a list of systems by IP/identifier" — verified 2026-06-24 against attack.mitre.org) |
| **Confidence** | high — T0846 technique verified live (ics-attack-19.1 pin); ListIdentity is explicitly the network enumeration primitive |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings, command_counts |
| **Deterministic** | yes — same command sequence produces same findings |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (detection within process_pdu) |
