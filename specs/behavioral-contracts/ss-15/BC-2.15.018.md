---
document_type: behavioral-contract
level: L3
version: "1.2"
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
  - "v1.1: Pass-1 adversarial fix I-4: added explicit ordering/dedup rule to Invariant 4 — broadcast anomaly finding (BC-2.15.018) is pushed FIRST; BC-2.15.010 burst finding pushed second if/when threshold crossed in same call. Both T1692.001 findings are RETAINED (distinct observations). Implementation must not deduplicate by technique ID alone. Cross-reference BC-2.15.013 added to Related BCs. — 2026-06-10"
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

# BC-2.15.018: Broadcast Destination Anomaly — DEST in 0xFFFD/0xFFFE/0xFFFF Emits Anomaly Finding

## Description

When a DNP3 frame's decoded DESTINATION address is in the broadcast range 0xFFFD..=0xFFFF
AND the frame carries a Control-class application FC (SELECT/OPERATE/DIRECT_OPERATE), the
analyzer emits an anomaly finding. Broadcast control commands (addressed to all outstations
simultaneously) are operationally abnormal — production control is always unicast to a specific
outstation. Time-sync and read commands to broadcast are legitimate; only Control-class FCs to
broadcast are anomalous. The three broadcast addresses are: 0xFFFD (broadcast, confirmation
required), 0xFFFE (broadcast, confirmation optional), 0xFFFF (broadcast, no confirmation).
[SPEC for broadcast range; per-address confirm-semantics are partially UNVERIFIED — see OQ-1
disposition below.]

**OQ-1 disposition (broadcast confirm-semantics — IN SCOPE for anomaly, NOT required for v1):**
Treating any destination in 0xFFFD..=0xFFFF as broadcast is safe and spec-supported regardless
of exact per-address confirm semantics. The per-address confirm nuance (0xFFFD requires confirm,
0xFFFE optional, 0xFFFF none) is NOT required for v1 detection — the anomaly fires on any
Control-class FC to any of the three addresses. The exact confirm-semantics can be encoded in
a future cycle once confirmed against IEEE 1815-2012 primary text.

**OQ-2 (self-address 0xFFFC) and OQ-3 (reserved range 0xFFF0–0xFFFB): OUT OF v1 SCOPE.**
Detection BCs for self-address and reserved-range destinations are deferred pending confirmation
of exact IEEE 1815-2012 reserved-address boundaries. These are not part of the issue #8
anomaly list.

## Preconditions

1. The validity gate (BC-2.15.004) returned `true`.
2. `h.destination >= 0xFFFD` (broadcast range) — `is_broadcast_destination(h.destination)` returns `true`.
3. The frame carries a Control-class application FC (`classify_dnp3_fc(app_fc) == Control`) on a FIR=1 frame.
4. `flow.is_non_dnp3 == false`.
5. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

1. Exactly ONE anomaly `Finding` is pushed:
   - `category: ThreatCategory::Suspicious`
   - `verdict: Verdict::Possible`
   - `confidence: Confidence::Medium`
   - `summary`: `"DNP3 broadcast control command: Control FC 0x{fc:02X} sent to broadcast destination {dest:#06X}"`
   - `evidence`: `"FC=0x{fc:02X} dest={dest:#06X} (broadcast) src={src:#06X}"`
   - `mitre_techniques: vec!["T1692.001"]`
     (Broadcast control to all outstations is an unauthorized/anomalous command; T1692.001
     is the appropriate technique — "Unauthorized Message: Command Message")
   - `source_ip: Some(...)`, `timestamp: Some(...)`
2. The per-flow Control-class counter (`direct_operate_count`) is ALSO incremented — this frame
   contributes to the threshold check in BC-2.15.010 in addition to the broadcast anomaly finding.
3. `flow.fc_counts` and `self.fn_code_counts` are updated.

## Invariants

1. **Broadcast range is 0xFFFD..=0xFFFF** [SPEC: dnp3-research.md §4; is_broadcast_destination helper]:
   `is_broadcast_destination(dest: u16) = dest >= 0xFFFD`. This is a simple `>=` comparison,
   proof of correctness is trivial.
2. **Only Control-class FCs trigger this BC**: READ, WRITE, and Management FCs to broadcast
   destinations are not anomalous (time-sync, global reads, and some management operations use
   broadcast legitimately). [dnp3-research.md §8 FP caveat 5]
3. **T1692.001 tag**: broadcast Control FCs are a form of unauthorized command message —
   sending a control command to all outstations simultaneously is functionally equivalent to
   an unauthorized bulk actuation.
4. **Does NOT suppress the BC-2.15.010 finding**: a broadcast Control FC both increments
   `direct_operate_count` (potentially triggering BC-2.15.010 threshold) AND emits this
   anomaly finding. The two are not mutually exclusive. Ordering (per BC-2.15.013 Invariant 4):
   the BC-2.15.018 anomaly finding is pushed FIRST (at the point of observation within the
   `on_data` call), then the BC-2.15.010 burst finding is pushed when/if the count threshold
   is crossed in the same call. Both findings carry T1692.001 but have different
   category/verdict/summary fields and represent distinct detection observations — they are
   NOT deduplicated by the implementation. A session with broadcast Control FCs may legitimately
   contain multiple T1692.001 findings from these two different rules.
5. **OQ-1 deferred**: per-address confirm-semantics (0xFFFD vs 0xFFFE vs 0xFFFF) are NOT
   required for v1 detection. The v1 anomaly fires uniformly on dest >= 0xFFFD with a Control FC.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DIRECT_OPERATE to dest=0xFFFF | Broadcast anomaly finding: T1692.001, Possible, Medium |
| EC-002 | DIRECT_OPERATE to dest=0xFFFD | Broadcast anomaly finding: T1692.001 |
| EC-003 | DIRECT_OPERATE to dest=0xFFFE | Broadcast anomaly finding: T1692.001 |
| EC-004 | READ (FC=0x01) to dest=0xFFFF | No anomaly finding (READ to broadcast is legitimate, e.g., global Class 0 poll) |
| EC-005 | WRITE (FC=0x02) to dest=0xFFFF | No broadcast anomaly (WRITE is not Control-class); T0836 finding from BC-2.15.012 still fires |
| EC-006 | COLD_RESTART to dest=0xFFFF | No broadcast anomaly (Restart-class, not Control-class); T0814 from BC-2.15.011 still fires |
| EC-007 | DIRECT_OPERATE to dest=0xFFFC (self-address) | OUT OF v1 SCOPE (OQ-2); no detection |
| EC-008 | DIRECT_OPERATE to dest=0xFFF0 (reserved) | OUT OF v1 SCOPE (OQ-3); no detection |

## Canonical Test Vectors

**Broadcast DIRECT_OPERATE frame:**
```
DNP3 link frame:  05 64 0E C4 FF FF 01 00 [hdr-crc]  C0 81 05 [objects]  [data-crc]
DEST bytes:       [0xFF, 0xFF] → dest = 0xFFFF (broadcast no-confirm)
App FC:           0x05 → DIRECT_OPERATE → Control class
```
Expected: `Finding { category: Suspicious, verdict: Possible, confidence: Medium, mitre_techniques: ["T1692.001"], summary: "DNP3 broadcast control command: Control FC 0x05 sent to broadcast destination 0xFFFF" }`

| Dest (hex) | FC | Is broadcast? | Is Control? | Anomaly? |
|-----------|-----|-------------|------------|---------|
| 0xFFFF | 0x05 DIRECT_OPERATE | yes | yes | YES — finding emitted |
| 0xFFFD | 0x03 SELECT | yes | yes | YES |
| 0xFFFE | 0x04 OPERATE | yes | yes | YES |
| 0xFFFF | 0x01 READ | yes | no | NO (legitimate broadcast read) |
| 0x0003 | 0x05 DIRECT_OPERATE | no | yes | NO (unicast; BC-2.15.010 applies) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | `is_broadcast_destination(dest >= 0xFFFD)` is a trivial comparison; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — broadcast control command detection is an anomaly detection capability of the DNP3/ICS analyzer; sending SELECT/OPERATE/DIRECT_OPERATE to all outstations simultaneously is operationally abnormal and a signal of adversarial ICS manipulation |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — anomaly findings emitted only on valid DNP3 port-20000 flows) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24); ADR-007 Decision 5 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | T1692.001 — Unauthorized Message: Command Message (broadcast control is an unauthorized command by nature) |

## Related BCs

- BC-2.15.003 — depends on (correct LE DEST decode determines is_broadcast_destination result)
- BC-2.15.010 — composes with (broadcast Control FC also increments direct_operate_count; both detections can fire; this BC's finding is pushed FIRST per BC-2.15.013 Invariant 4)
- BC-2.15.013 — composes with (ordering and dedup rule for broadcast-anomaly + burst-threshold T1692.001 co-occurrence documented in BC-2.15.013 Invariant 4 and EC-005)
- BC-2.15.022 — depends on (MAX_FINDINGS cap)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `fn is_broadcast_destination(dest: u16) -> bool { dest >= 0xFFFD }`
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §7` — `is_broadcast_destination` helper
- `.factory/research/dnp3-research.md §4` — broadcast addresses 0xFFFD/0xFFFE/0xFFFF confirmed [SPEC]; per-address confirm-semantics partially [UNVERIFIED]

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

(none — trivial comparison; no formal proof needed)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | dnp3-research.md §4 (broadcast range confirmed [SPEC]; per-address semantics [UNVERIFIED]); dnp3-architecture-delta.md §7 (is_broadcast_destination helper); ADR-007 Decision 5 (broadcast anomaly detection) |
| **Confidence** | high (0xFFFD..=0xFFFF as broadcast range — SPEC-confirmed); medium (per-address confirm-semantics deferred per OQ-1) |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads h.destination; writes all_findings, fc_counts |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell |
