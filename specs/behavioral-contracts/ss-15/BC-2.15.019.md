---
document_type: behavioral-contract
level: L3
version: "1.0"
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
introduced: v0.5.0-feature-008
modified: []
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

# BC-2.15.019: Unsolicited Response Anomaly — UNS Bit Set or FC 0x82 From Unexpected Pattern

## Description

DNP3 UNSOLICITED_RESPONSE (FC=0x82) is a legitimate outstation-initiated event-reporting
mechanism. It becomes anomalous when: (a) it appears without a prior ENABLE_UNSOLICITED
(FC=0x14) exchange in the capture, OR (b) it originates from a source address not previously
seen issuing RESPONSE (FC=0x81) on this flow, OR (c) the Application Control byte has the
UNS bit (bit 4, mask 0x10) set unexpectedly in a non-response context. This BC detects the
first occurrence per flow when neither ENABLE_UNSOLICITED nor a prior RESPONSE has been
observed — a signal of unsolicited-response injection or spoofing. T0814 is the co-technique
for unsolicited-flood injection (DoS-class attack).

## Preconditions

1. The validity gate (BC-2.15.004) returned `true`.
2. `has_user_data(control)` is `true`; FIR=1 (BC-2.15.008).
3. App FC = 0x82 (UNSOLICITED_RESPONSE) OR App Control `app_ctrl & 0x10 != 0` (UNS bit set).
4. `flow.enable_unsolicited_seen == false` (no ENABLE_UNSOLICITED FC=0x14 was observed in this capture's flow).
5. `flow.response_seen == false` (no solicited RESPONSE FC=0x81 from this outstation was previously seen on this flow).
6. `flow.unsolicited_anomaly_emitted == false` (one-shot guard — anomaly fires once per flow).
7. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

1. ONE anomaly `Finding` is pushed:
   - `category: ThreatCategory::Suspicious`
   - `verdict: Verdict::Possible`
   - `confidence: Confidence::Low`
   - `summary`: `"DNP3 unexpected unsolicited response: UNSOLICITED_RESPONSE from src={src:#06X} with no prior ENABLE_UNSOLICITED or solicited exchange on this flow"`
   - `evidence`: `"FC=0x82 src={src:#06X} dest={dest:#06X} UNS_bit={uns_bit}"`
   - `mitre_techniques: vec!["T0814"]`
   - `source_ip: Some(...)`, `timestamp: Some(...)`
2. `flow.unsolicited_anomaly_emitted = true` (one-shot guard set).

**When ENABLE_UNSOLICITED (0x14) is observed:**
3. `flow.enable_unsolicited_seen = true` — subsequent UNSOLICITED_RESPONSE frames on this flow are NOT anomalous.

**When RESPONSE (0x81) is observed:**
4. `flow.response_seen = true` — this flow has a known-legitimate solicited exchange pattern;
   subsequent unsolicited responses are NOT flagged as anomalous (the outstation is known to respond).

## Invariants

1. **One-shot guard**: `unsolicited_anomaly_emitted` prevents repeated anomaly findings for the
   same flow. The first unsolicited-without-context event is the signal; subsequent ones are noise.
2. **ENABLE_UNSOLICITED flag**: legitimate unsolicited reporting always follows an
   ENABLE_UNSOLICITED (0x14) exchange. If the capture shows ENABLE_UNSOLICITED before the
   UNSOLICITED_RESPONSE, no anomaly is raised. [SPEC: dnp3-research.md §3.2, §5.1]
3. **T0814 (DoS) tactic**: unsolicited-response injection can be used to flood the master with
   spurious events, masking legitimate events or exhausting master-station resources. [SPEC: dnp3-research.md §5]
4. **Confidence::Low**: a capture that started mid-flow may have missed the ENABLE_UNSOLICITED
   exchange. This is an inherent false-positive risk for passive analyzers. The low confidence
   communicates this uncertainty.
5. **UNS bit in non-response context**: if UNS bit is set in the App Control of a request
   (from a master, DIR=1), this is protocol-unusual (UNS is meant for outstation responses);
   the same logic applies.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ENABLE_UNSOLICITED observed first, then UNSOLICITED_RESPONSE | No anomaly (`enable_unsolicited_seen=true`) |
| EC-002 | UNSOLICITED_RESPONSE at start of capture (no prior context) | Anomaly finding: T0814, Possible, Low |
| EC-003 | Second UNSOLICITED_RESPONSE (same flow, no prior context) | No additional anomaly (one-shot guard; first already emitted) |
| EC-004 | RESPONSE (0x81) observed first, then UNSOLICITED_RESPONSE | No anomaly (`response_seen=true`; outstation is known-legitimate) |
| EC-005 | Capture starts mid-session (ENABLE_UNSOLICITED was in pre-capture traffic) | False positive likely; this is accepted per passive-analyzer caveat |

## Canonical Test Vectors

| Scenario | Flow events | Expected outcome |
|----------|------------|-----------------|
| Clean unsolicited (ENABLE first) | FC=0x14, then FC=0x82 | No anomaly; `enable_unsolicited_seen=true` |
| Injection (no ENABLE) | FC=0x82 immediately | Anomaly finding: `{mitre_techniques:["T0814"], verdict:Possible, confidence:Low}` |
| Mid-capture start | FC=0x82 (no prior context) | Anomaly finding (accepted FP for passive analyzer) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | State-flag logic: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — unsolicited-response anomaly detection is a protocol-specific ICS analysis capability; forged or injected UNSOLICITED_RESPONSE messages can falsify process state at the master SCADA station, a technique observed in advanced ICS attacks |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — findings emitted only on valid DNP3 port-20000 flows) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-23); ADR-007 Decision 5 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | T0814 — Denial of Service (unsolicited flood is a DoS-class technique; tactic: IcsInhibitResponseFunction) |

## Related BCs

- BC-2.15.006 — depends on (Response-class FC classification; FC 0x82 maps to Response)
- BC-2.15.008 — depends on (FIR=1 gate)
- BC-2.15.022 — depends on (MAX_FINDINGS cap)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `Dnp3FlowState.enable_unsolicited_seen: bool`, `.response_seen: bool`, `.unsolicited_anomaly_emitted: bool`
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §8` — "Unsolicited response anomaly: UNS bit set / FC 0x82 from unexpected source"
- `.factory/research/dnp3-research.md §3.1` (UNS bit, mask 0x10); §5.1 (unsolicited threshold rationale [JUDGMENT])

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

(none — effectful shell; state-flag logic)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | dnp3-research.md §3.1 (UNS bit 0x10 confirmed [SPEC]); §3.2 (FC 0x82 UNSOLICITED_RESPONSE confirmed [SPEC]); §5 (T0814 unsolicited-flood mapping [MITRE]); §5.1 (anomaly threshold rationale [JUDGMENT]) |
| **Confidence** | medium — T0814 confirmed [MITRE]; unsolicited anomaly logic is [JUDGMENT] for passive analyzer |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads/writes enable_unsolicited_seen, response_seen, unsolicited_anomaly_emitted, all_findings |
| **Deterministic** | yes — same exchange sequence produces same state |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell |
