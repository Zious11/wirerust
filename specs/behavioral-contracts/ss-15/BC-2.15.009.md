---
document_type: behavioral-contract
level: L3
version: "2.0"
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
  - "v1.5: F3 story-anchor back-fill. — 2026-06-14"
  - "v1.6: Pass-28 F3-convergence Slice-B FIX 1: corrected wrong cross-ref in Related BCs — BC-2.15.020 (stats BC) → BC-2.15.016 (carry buffer management BC); descriptor text was already correct; BC-2.15.016 reciprocally lists this BC. — 2026-06-14"
  - "v2.0: RULING-DNP3-DESYNC-001 (2026-06-28, §3): Breaking — Precondition 3 amended: desync bail latch fires ONLY when BOTH directional carries are empty (`flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()`), NOT just the active direction's carry. After STORY-140 carry split, checking only the active direction's carry allows a junk s2c delivery to latch `is_non_dnp3=true` on a flow where `carry_c2s` has an established partial c2s frame (DRIFT-DNP3-DESYNC-001 / DESIGN-CROSS-DIRECTION-STATE §2.1). Description paragraph 2 updated to explain both-carries-empty semantics. Architecture Anchors updated: `flow.carry` → `flow.carry_c2s` / `flow.carry_s2c`. EC-010 added: partial c2s frame in carry_c2s + junk s2c delivery → bail does NOT fire → c2s stream continues. EC-011 added: complete c2s frames consumed (carry_c2s drained), then junk s2c delivery → both empty → bail fires (same as pre-STORY-140 behavior). — 2026-06-28"
  - "v2.0 modified (ADDENDUM-2026-06-28-frame-count-guard): RULING-DNP3-DESYNC-001 ADDENDUM-2026-06-28 — sub-case ii correction: the both-carries-empty-only predicate was INCOMPLETE. Carries are transiently drained after a complete frame parse; an established flow (frame_count>=1) with both carries empty would still latch under the prior condition (sub-case ii: c2s request fully parsed → carry_c2s drained → junk s2c arrives → both carries empty → latch fires incorrectly). Complete canonical predicate now requires `flow.frame_count == 0` as an additional gate: latch fires ONLY when frame_count==0 AND both carries empty AND data.len()>=2 AND no sync. Description updated with sub-case ii explanation. Precondition 3 updated to full predicate. EC-011 corrected: bail does NOT fire when frame_count>=1 (sub-case ii). EC-012 added: complete c2s frame then junk s2c with both carries drained — latch does NOT fire because frame_count==1. Architecture Anchors updated to full predicate. — 2026-06-28"
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
no-ops. This is a one-shot, initial-delivery-only mechanism: after the STORY-140 carry split
and the sub-case ii correction (ADDENDUM-2026-06-28-frame-count-guard), the bail fires only
when ALL of the following are true: `flow.frame_count == 0` (no frame has ever been
successfully parsed in any direction), `flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()`
(no in-flight carry bytes in either direction), `data.len() >= 2`, and the data does not begin
with the DNP3 sync word. Checking BOTH directional carries AND `frame_count == 0` ensures that
a junk delivery in one direction cannot latch `is_non_dnp3` while the other direction either has
an established partial frame buffered OR has already successfully parsed at least one complete
frame. The `frame_count == 0` guard is load-bearing: carries are transiently drained to empty
after each complete frame is consumed, so the both-carries-empty condition alone is insufficient
for established flows — once `frame_count >= 1` the flow is unconditionally established and the
latch never fires regardless of carry state. (Sub-case ii: a normal c2s request→s2c response
lifecycle always reaches frame_count=1 before any s2c delivery, so an anomalous s2c packet on
an established flow cannot silence the c2s direction.) Once either `frame_count >= 1` OR any
bytes have been accepted into `carry_c2s` or `carry_s2c`, the flow is established as DNP3 and
`is_non_dnp3` cannot be set from this path. (RULING-DNP3-DESYNC-001) Mid-carry sync-loss on an
established flow is handled by byte-walk-forward resync (BC-2.15.016 EC-007), not by a second
bail. This desync-safe bail prevents cascading parse errors and false findings from non-DNP3
binary protocols that happen to use port 20000. The per-flow `is_non_dnp3` flag, once set, is
never cleared (flows are immutable in their desync state).

## Preconditions

1. A new flow has been created with `is_non_dnp3 = false`.
2. `on_data` is called with `data` for this flow.
3. `flow.frame_count == 0` (no frame has ever been successfully parsed in any direction) AND
   `flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()` (no bytes have yet been accepted
   into either directional carry — the flow is unestablished in BOTH directions) AND
   `data.len() >= 2` AND `(data[0] != 0x05 || data[1] != 0x64)` (no valid DNP3 sync word at
   offset 0 of the first delivery). The bail fires ONLY when ALL four conditions are met:
   `frame_count == 0`, both carries empty, `data.len() >= 2`, and no sync. The `frame_count == 0`
   guard is the primary establishment check: once any frame has been successfully parsed in
   either direction the flow is unconditionally established and the latch never fires regardless
   of carry state. If `carry_c2s` is non-empty (partial c2s frame in flight), a junk s2c
   delivery does NOT latch `is_non_dnp3` — the c2s stream has established the flow.
   (RULING-DNP3-DESYNC-001 §2.1; ADDENDUM-2026-06-28-frame-count-guard)
   If `data.len() < 2` on the first call (both carries still empty), the bail does NOT fire —
   the byte is accumulated into the appropriate directional carry and the check defers to the
   next delivery that arrives while both carries are still empty, `frame_count == 0`, and
   `data.len() >= 2`. Once `frame_count >= 1` OR either carry is non-empty (any bytes
   accepted), this precondition can never be satisfied again; the bail path is closed
   permanently.

## Postconditions

**On bail trigger** (`frame_count == 0`, both carries empty, `data.len() >= 2`, no valid DNP3 sync at offset 0 — complete predicate per ADDENDUM-2026-06-28-frame-count-guard):
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

1. **Initial-delivery-only check**: the desync bail is a one-shot mechanism guarded by the
   complete predicate: `flow.frame_count == 0 && flow.carry_c2s.is_empty() &&
   flow.carry_s2c.is_empty() && data.len() >= 2`. It fires only when ALL conditions are met.
   After the STORY-140 carry split and the ADDENDUM-2026-06-28-frame-count-guard sub-case ii
   correction, "unestablished flow" requires BOTH that no frame has been parsed (`frame_count == 0`)
   AND that both carries are empty. The `frame_count == 0` guard is load-bearing: carries are
   transiently drained to empty after each complete frame parse, so the both-carries-empty
   condition alone cannot distinguish a genuinely unestablished flow from an established flow
   between frames. Once `frame_count >= 1` the latch never fires regardless of carry state.
   A junk delivery in one direction cannot latch `is_non_dnp3` if the other direction has a
   partial frame buffered — `carry_c2s.is_non_empty()` makes the both-empty guard false.
   A single-byte first delivery (< 2 bytes) defers the check: the byte is accumulated into
   the active directional carry and the next delivery while `frame_count == 0`, both carries
   still empty, re-evaluates. Once `frame_count >= 1` OR either carry is non-empty, this
   bail path is permanently closed. There is NO cross-segment 16-byte accumulation bail;
   such a mechanism was architecturally rejected (latching an established flow as non-DNP3
   based on transient carry misalignment is a self-DoS risk — ADJ-001 Addendum Q1). Mid-carry
   sync-loss is handled by byte-walk-forward resync (BC-2.15.016 EC-007), which clears and
   repositions the carry without setting `is_non_dnp3`. [ADR-007 Decision 2; ADJ-001 Decision 1
   BC-2.15.009 Interaction; ADJ-001 Addendum Q1; RULING-DNP3-DESYNC-001 §2.1;
   ADDENDUM-2026-06-28-frame-count-guard]
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
| EC-010 | First delivery: `direction=ClientToServer`, valid DNP3 sync `[0x05, 0x64, ...]` but incomplete frame — `carry_c2s` accumulates 6 bytes. Second delivery: `direction=ServerToClient`, non-DNP3 junk `[0xFF, 0xFE, 0x00]`. | `flow.carry_c2s.is_empty() = false` → both-carries-empty guard is false → bail condition does NOT fire → `is_non_dnp3` remains false → c2s stream continues processing on subsequent deliveries. (RULING-DNP3-DESYNC-001 §2.3 Case 2; §3 EC-010) |
| EC-011 | First delivery: `direction=ClientToServer`, non-DNP3 junk. `frame_count=0`, `carry_c2s.is_empty()=true`, `carry_s2c.is_empty()=true`. | All four conditions true: `frame_count==0`, both carries empty, `data.len()>=2`, no sync → bail fires → `is_non_dnp3=true`. Correct: genuinely unestablished flow with junk first delivery. (RULING-DNP3-DESYNC-001 §2.3 Case 1; §3 EC-011; ADDENDUM-2026-06-28-frame-count-guard) |
| EC-012 | First delivery: `direction=ClientToServer`, valid DNP3 sync `[0x05, 0x64, ...]`, complete frame fully parsed, `carry_c2s` drained to empty, `frame_count=1`. Second delivery: `direction=ServerToClient`, non-DNP3 junk `[0xFF, 0xFE, 0x00]`. State at latch check: `frame_count=1`, `carry_c2s.is_empty()=true`, `carry_s2c.is_empty()=true`. | `frame_count==0` is FALSE (frame_count=1 ≥ 1) → complete predicate fails → bail does NOT fire → `is_non_dnp3` remains false → c2s stream continues on subsequent deliveries. **Sub-case ii — the common request→response lifecycle: a completed c2s frame drains carry_c2s to empty; the both-carries-empty-only condition would have incorrectly latched is_non_dnp3 here. The `frame_count==0` guard is the load-bearing fix.** (RULING-DNP3-DESYNC-001 ADDENDUM-2026-06-28-frame-count-guard; §2.3 Case 2 sub-case ii; §3 EC-012) |

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
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24 `Dnp3FlowState.is_non_dnp3`); ADR-007 Decision 2 |
| Stories | STORY-106 |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — safety/bail logic; no finding emission) |

## Related BCs

- BC-2.15.004 — composes with (validity gate checks individual frames; is_non_dnp3 is a flow-level bail that short-circuits before any individual frame is parsed)
- BC-2.15.016 — composes with (carry buffer management BC; bail prevents unbounded carry growth)
- BC-2.15.024 — composes with (malformed-anomaly counter; is_non_dnp3 bail is NOT a parse_errors source per this BC's PC3 — the bail fires BEFORE any frame parse stage and explicitly does not increment parse_errors to avoid misleading metrics on misclassified non-DNP3 flows; F-F5-004 reconciliation; F-C-006 cross-reference)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `Dnp3FlowState.is_non_dnp3: bool` — false on creation; set true on bail
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>` (STORY-140 / RULING-DNP3-SIBLING-001 carry split)
- `src/analyzer/dnp3.rs` — `Dnp3Analyzer::on_data` — early return if `flow.is_non_dnp3`
- `src/analyzer/dnp3.rs` — desync-latch block (pre-fix line 363): complete predicate (post-ADDENDUM-2026-06-28-frame-count-guard): `flow.frame_count == 0 && flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty() && data.len() >= 2 && (data[0] != 0x05 || data[1] != 0x64)` (RULING-DNP3-DESYNC-001 §2.1; ADDENDUM-2026-06-28-frame-count-guard)
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §2.3` — `is_non_dnp3: bool` field description
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 2` — "is_non_dnp3 desync-safe bail"
- Note: line numbers are PRE-fix (STORY-142 implementer re-anchors to post-fix lines)

## Story Anchor

STORY-106

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
