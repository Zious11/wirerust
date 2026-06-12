---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - "v1.2: F5-R2 change (F-C-006) — Related BCs: added reciprocal cross-reference to BC-2.15.024 with explicit statement that is_non_dnp3 bail is NOT a parse_errors source per this BC's PC3, consistent with BC-2.15.024 v1.3 F-F5-004 reconciliation. — 2026-06-12"
  - "v1.3: F7 F-S1-001 reconciliation — Invariant 1/Precondition 3/EC-004 corrected to match ADJ-001 initial-delivery-only is_non_dnp3 semantics (cross-segment 16-byte accumulation bail was never implemented and is architecturally rejected per ADJ-001 Addendum Q1; established-flow misalignment handled by byte-walk-forward resync per BC-2.15.016 EC-007). Vestige cleanup (same F7 F-S1-001 reconciliation): EC-002 reframed from '16-byte window' phrasing to initial-delivery-only semantics (carry empty, data.len()>=2, no offset-0 sync); Canonical Test Vectors column header changed from 'First 16 bytes (hex)' to 'First delivery (hex)'; vector rows annotated with carry-state context to match initial-delivery model. — 2026-06-12"
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

# BC-2.15.009: is_non_dnp3 Desync-Safe Bail — Flow Silenced on Initial-Delivery No-Sync (One-Shot, First Delivery Only)

## Description

When a TCP flow is classified as DNP3 (port 20000, ADR-007 Rule 6) and the very first
`on_data` delivery finds no valid DNP3 start word `[0x05, 0x64]` at offset 0, the analyzer
sets `flow.is_non_dnp3 = true` and treats all subsequent `on_data` calls for that flow as
no-ops. This is a one-shot, initial-delivery-only mechanism: the bail fires only when
`flow.carry.is_empty() && data.len() >= 2 && data[0..2] != [0x05, 0x64]`. Once any bytes
have been accepted into carry the flow is established as DNP3 and `is_non_dnp3` cannot be
set from this path. Mid-carry sync-loss on an established flow is handled by byte-walk-forward
resync (BC-2.15.016 EC-007), not by a second bail. This desync-safe bail prevents cascading
parse errors and false findings from non-DNP3 binary protocols that happen to use port 20000.
The per-flow `is_non_dnp3` flag, once set, is never cleared (flows are immutable in their
desync state).

## Preconditions

1. A new flow has been created with `is_non_dnp3 = false`.
2. `on_data` is called with `data` for this flow.
3. `flow.carry.is_empty()` (no bytes have yet been accepted into carry — flow is not yet
   established) AND `data.len() >= 2` AND `data[0] != 0x05 || data[1] != 0x64` (no valid
   DNP3 sync word at offset 0 of the first delivery). If `data.len() < 2` on the first
   call (carry still empty), the bail does NOT fire — the byte is accumulated into carry and
   the check defers to the next delivery that arrives while carry is still empty and
   `data.len() >= 2`. Once carry is non-empty (any bytes accepted), this precondition can
   never be satisfied again; the bail path is closed permanently.

## Postconditions

**On bail trigger** (first delivery while carry empty: `data.len() >= 2` and no valid DNP3 sync at offset 0):
1. `flow.is_non_dnp3` is set to `true`.
2. The current `on_data` call returns immediately without emitting findings.
3. `flow.frame_count` and `flow.parse_errors` are NOT incremented for the triggering segment.
4. No further parse operations are performed on this flow.

**On all subsequent `on_data` calls when `flow.is_non_dnp3 == true`:**
5. `on_data` returns immediately (no-op) — no parsing, no detection, no finding emission.
6. `flow.is_non_dnp3` remains `true`; it is never reset to `false`.

**Normal flows (valid DNP3 sync observed in first delivery, or first delivery has < 2 bytes):**
7. `flow.is_non_dnp3` remains `false`; normal processing continues.

## Invariants

1. **Initial-delivery-only check**: the desync bail is a one-shot mechanism guarded by
   `flow.carry.is_empty() && data.len() >= 2`. It fires on the first delivery that arrives
   while carry is empty and delivers at least 2 bytes without `[0x05, 0x64]` at offset 0.
   A single-byte first delivery (< 2 bytes) defers the check: the byte is accumulated and
   the next delivery while carry is still empty re-evaluates. Once carry is non-empty —
   meaning the flow is established as DNP3 — this bail path is permanently closed. There is
   NO cross-segment 16-byte accumulation bail; such a mechanism was architecturally rejected
   (latching an established flow as non-DNP3 based on transient carry misalignment is a
   self-DoS risk — ADJ-001 Addendum Q1). Mid-carry sync-loss is handled by byte-walk-forward
   resync (BC-2.15.016 EC-007), which clears and repositions the carry without setting
   `is_non_dnp3`. [ADR-007 Decision 2; ADJ-001 Decision 1 BC-2.15.009 Interaction; ADJ-001 Addendum Q1]
2. **One-way latch**: `is_non_dnp3` is a one-way latch — once set to `true`, it never reverts.
   There is no re-sync mechanism in v1 (deferred to a future cycle).
3. **No side effects on bailed flow**: a bailed flow is a permanent no-op. It does not increment
   parse_errors per segment (would produce misleading metrics), does not emit findings, and does
   not grow the carry buffer.
4. **Max carry**: the carry buffer is bounded to `MAX_DNP3_FRAME_LEN = 292 bytes` per flow.
   The initial-delivery bail ensures the carry buffer never grows at all on misclassified
   non-DNP3 flows: the bail fires before the accumulation step, so carry is never touched
   and stays empty permanently after bail.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First byte = 0x05, second byte = 0x65 (almost sync) | Bail — START2 is 0x65, not 0x64; no valid sync |
| EC-002 | First delivery is ≥2 bytes, all 0x00 (no `[0x05, 0x64]` at offset 0) | Bail — carry is empty, `data.len() >= 2`, no valid sync at offset 0 |
| EC-003 | Valid sync `0x05 0x64` appears at offset 2 (not offset 0) | Bail — the check is for sync at offset 0 (v1 does not scan for offset-N sync) |
| EC-004 | First delivery is only 1 byte (`0x05`); carry is empty; `data.len() < 2` | Defer — the bail guard (`data.len() >= 2`) is not satisfied. The single byte is accumulated into carry. On the next delivery while carry is still empty, if the two-byte check then passes (no sync), bail fires; if it passes as valid sync or carry is now non-empty (flow established), normal processing continues. Once carry is non-empty after any acceptance, the bail path is closed. |
| EC-005 | Flow correctly starts with `0x05 0x64` | No bail — `is_non_dnp3` stays `false`, normal processing |
| EC-006 | Subsequent `on_data` call on a bailed flow | Immediate no-op; no parse; no metrics change |

## Canonical Test Vectors

| Scenario | First delivery (hex) | Expected outcome |
|----------|---------------------|-----------------|
| Non-DNP3 binary on port 20000 | `FF FE 00 01 02 03 ...` (no `[0x05, 0x64]` at offset 0; carry empty) | `is_non_dnp3 = true`; no findings emitted ever |
| Valid DNP3 frame start | `05 64 0E C4 03 00 01 00 ...` (carry empty, sync at offset 0) | `is_non_dnp3` stays false; parsing proceeds |
| TLS ClientHello on port 20000 | `16 03 01 00 ...` | Routed to TLS by content rule (ADR-007 Rule 1) before DNP3 port rule; does not reach this BC |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | `is_non_dnp3` flag state transition is a simple boolean latch; no formal proof target. Covered by unit/integration tests. | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — the desync bail is the false-positive prevention mechanism for port-only classification, ensuring the DNP3/ICS analyzer does not emit erroneous ICS threat findings for non-DNP3 protocols on port 20000 |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — the bail maintains the invariant that ICS findings are only emitted for flows carrying actual DNP3 protocol content) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-23 `Dnp3FlowState.is_non_dnp3`); ADR-007 Decision 2 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — safety/bail logic; no finding emission) |

## Related BCs

- BC-2.15.004 — composes with (validity gate checks individual frames; is_non_dnp3 is a flow-level bail that short-circuits before any individual frame is parsed)
- BC-2.15.020 — composes with (carry buffer management BC; bail prevents unbounded carry growth)
- BC-2.15.024 — composes with (malformed-anomaly counter; is_non_dnp3 bail is NOT a parse_errors source per this BC's PC3 — the bail fires BEFORE any frame parse stage and explicitly does not increment parse_errors to avoid misleading metrics on misclassified non-DNP3 flows; F-F5-004 reconciliation; F-C-006 cross-reference)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `Dnp3FlowState.is_non_dnp3: bool` — false on creation; set true on bail
- `src/analyzer/dnp3.rs` — `Dnp3Analyzer::on_data` — early return if `flow.is_non_dnp3`
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §2.3` — `is_non_dnp3: bool` field description
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 2` — "is_non_dnp3 desync-safe bail"

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

(none — unit test coverage; trivial state machine)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-007 Decision 2 (desync-safe bail); dnp3-architecture-delta.md §2.3 (is_non_dnp3 field); modbus precedent (ModbusFlowState) |
| **Confidence** | high — architectural decision; mirrors established Modbus pattern |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates `flow.is_non_dnp3` (per-flow state) |
| **Deterministic** | yes — same carry bytes produce same bail decision |
| **Thread safety** | single-threaded; no concurrent access |
| **Overall classification** | effectful shell (mutates flow state) |
