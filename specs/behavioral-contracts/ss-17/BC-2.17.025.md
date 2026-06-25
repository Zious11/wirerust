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

# BC-2.17.025: RegisterSession (0x0065) and UnRegisterSession (0x0066) Classified and PDU-Counted; No Finding Emitted

## Description

EtherNet/IP session establishment (RegisterSession, command 0x0065) and session teardown
(UnRegisterSession, command 0x0066) are normal protocol handshake operations required before
any explicit messaging (SendRRData 0x006F) can occur. When `classify_enip_command(header.command)`
returns `EnipCommandClass::RegisterSession` or `EnipCommandClass::UnRegisterSession`, the frame
is treated as a classified PDU: `pdu_count` is incremented and `command_counts[cmd]` is updated,
but NO finding is emitted. Session-handle anomaly validation (e.g., mismatched session handles,
replayed session handles) is explicitly deferred to v0.12.0.

## Preconditions

1. `classify_enip_command(header.command)` returns `EnipCommandClass::RegisterSession`
   (command 0x0065) or `EnipCommandClass::UnRegisterSession` (command 0x0066).
2. `flow.is_non_enip == false`.
3. `is_valid_enip_frame(&header)` returns `true` (the frame passed the validity gate).

## Postconditions

1. `flow.pdu_count += 1` — the frame is counted as a processed PDU (per BC-2.17.024: incremented in `process_pdu` at start of call).
2. `flow.command_counts.entry(header.command).or_insert(0) += 1` — command occurrence logged.
3. NO `Finding` is pushed to `self.all_findings`. The session handshake is a normal protocol
   operation and does not indicate a threat in isolation.
4. No one-shot guard is set; no window counter is affected.
5. `flow.is_non_enip` is not modified.

## Invariants

1. **Session handshake is not a finding**: RegisterSession and UnRegisterSession are required
   EtherNet/IP protocol steps. Emitting a finding for every session establishment would produce
   extremely high false-positive rates in normal SCADA/DCS environments.
2. **PDU-counted**: the frame is a valid ENIP PDU and contributes to `pdu_count` and
   `command_counts` for `summarize()` output, providing visibility into session activity.
3. **Session-handle anomaly deferred**: validation of session handles (e.g., detecting replayed
   or mismatched handles that may indicate session hijacking) is explicitly out-of-scope for
   v0.11.0 and deferred to v0.12.0. No state tracking for session handles is added in this BC.
4. **MITRE techniques empty**: `mitre_techniques: vec![]` — no MITRE technique tag applies to
   the bare session handshake in v0.11.0.
5. **Symmetric treatment**: both RegisterSession (establishment) and UnRegisterSession (teardown)
   follow the same policy: classify, count, do not emit.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | RegisterSession (0x0065) on a new flow | `pdu_count += 1`; `command_counts[0x0065] = 1`; no finding |
| EC-002 | UnRegisterSession (0x0066) after a RegisterSession on the same flow | `pdu_count += 1`; `command_counts[0x0066] = 1`; no finding |
| EC-003 | Multiple RegisterSession frames on the same flow (e.g., re-registration) | Each increments `pdu_count` and `command_counts[0x0065]`; no findings |
| EC-004 | RegisterSession followed immediately by SendRRData (normal usage) | RegisterSession: no finding; SendRRData: processed normally per BC-2.17.003/005; CIP detections fire as applicable |
| EC-005 | UnRegisterSession with non-zero status field (protocol error) | Frame classified as UnRegisterSession; `pdu_count++`; `command_counts++`; no finding (session-handle anomaly validation deferred) |

## Canonical Test Vectors

**RegisterSession frame:**
```
ENIP header (hex, little-endian): 65 00 04 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
command: 0x0065 (RegisterSession, LE: bytes [65 00]), length: 4, session_handle: 0
```
Expected: `pdu_count += 1`; `command_counts[0x0065] += 1`; `all_findings` unchanged.

**UnRegisterSession frame:**
```
ENIP header (hex, little-endian): 66 00 00 00 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
command: 0x0066 (UnRegisterSession, LE), length: 0, session_handle: 0x00000001
```
Expected: `pdu_count += 1`; `command_counts[0x0066] += 1`; `all_findings` unchanged.

| Command | pdu_count delta | command_counts update | Finding emitted? |
|---------|----------------|----------------------|-----------------|
| 0x0065 RegisterSession | +1 | [0x0065] += 1 | No |
| 0x0066 UnRegisterSession | +1 | [0x0066] += 1 | No |
| 0x006F SendRRData (normal) | +1 | [0x006F] += 1 | Per CIP detection BCs |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-032 | Sub-B (indirect): `classify_enip_command(0x0065)` returns `RegisterSession`; `classify_enip_command(0x0066)` returns `UnRegisterSession` — precondition verified | Kani Sub-B totality proof |
| (none) | No-finding policy for session handshake, PDU counting: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — RegisterSession and UnRegisterSession are the EtherNet/IP session establishment/teardown handshake; classifying and counting them (without emitting findings) is required for complete EtherNet/IP protocol coverage and accurate PDU accounting in summarize() output |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — ENIP flows are only routed after TLS/HTTP content rules fail) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 4 (pdu_count and command_counts fields) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — session handshake; no finding emission; session-handle anomaly validation deferred to v0.12.0) |

## Related BCs

- BC-2.17.004 — depends on (classify_enip_command maps 0x0065/0x0066 to RegisterSession/UnRegisterSession)
- BC-2.17.024 — composes with (pdu_count incremented per BC-2.17.024 contract)
- BC-2.17.021 — composes with (command_counts[0x0065/0x0066] reflected in summarize() output)

## Architecture Anchors

- `src/analyzer/enip.rs` — `process_pdu`: `if matches!(cmd_class, EnipCommandClass::RegisterSession | EnipCommandClass::UnRegisterSession) { flow.command_counts.entry(header.command).or_insert(0) += 1; /* no finding */ }`
- `src/analyzer/enip.rs` — `EnipFlowState.command_counts: HashMap<u16, u64>`
- `src/analyzer/enip.rs` — `EnipFlowState.pdu_count: u64` (incremented in `process_pdu` at start of call, per BC-2.17.024)
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 4` (pdu_count and command_counts)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-032 Sub-B (indirect — verifies classify_enip_command precondition for 0x0065/0x0066)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 4 (pdu_count, command_counts); ODVA EtherNet/IP Specification §2-5.1 (RegisterSession command 0x0065); §2-5.2 (UnRegisterSession command 0x0066); F2 adversary Pass-1 finding F-ENIP-004 (session handshake gap) |
| **Confidence** | high — RegisterSession/UnRegisterSession are normative ODVA; no-finding policy is unambiguous for v0.11.0 scope |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow.command_counts; increments flow.pdu_count (via process_pdu) |
| **Deterministic** | yes — same command sequence produces same counter state |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (session-handshake dispatch within process_pdu) |
