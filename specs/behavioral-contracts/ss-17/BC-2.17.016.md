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

# BC-2.17.016: Carry-Buffer Frame-Walk Loop — Partial Frame Stash and MAX_ENIP_CARRY_BYTES Cap

## Description

`EnipAnalyzer::on_data()` implements a frame-walk loop that processes complete ENIP frames
from a per-flow byte buffer. When the buffer contains fewer bytes than a complete header (< 24)
or fewer bytes than a complete frame (< 24 + header.length), the remaining bytes are stashed
into `EnipFlowState.carry`. The carry buffer is bounded to `MAX_ENIP_CARRY_BYTES = 600`.
If the stash would exceed 600 bytes, `flow.is_non_enip` is set to `true` and
`flow.parse_errors` is incremented. All subsequent `on_data` calls for a flow with
`is_non_enip = true` are immediate no-ops.

## Preconditions

1. `EnipAnalyzer::on_data(flow_key, data, now_ts)` is called with new TCP bytes.
2. `buf = flow.carry ++ data` — carry buffer prepended to new data.
3. `flow.is_non_enip == false` at call time (flows with `is_non_enip=true` exit immediately
   at the top of on_data).

## Postconditions

**Frame walk loop:**
1. While `buf.len() - cursor >= 24`:
   - Parse header at `buf[cursor..cursor+24]` (BC-2.17.001/002).
   - If `!is_valid_enip_frame(&header)`: `flow.parse_errors += 1`; `flow.malformed_in_window += 1`; `cursor += 1`; continue (byte-walk resync).
   - **Frame-skip path (oversized declared frame)**: if `24 + header.length as usize > MAX_ENIP_CARRY_BYTES` (600):
     - `flow.parse_errors += 1`; `flow.malformed_in_window += 1`.
     - Advance cursor: `cursor += min(24 + header.length as usize, buf.len() - cursor)` (bounded by remaining buffer).
     - Continue the walk. Do NOT set `is_non_enip`. Do NOT stash into carry.
   - If `buf.len() - cursor < 24 + header.length as usize` (partial frame in buffer): stash `buf[cursor..]` into carry; apply cap check; break.
   - Otherwise: call `process_pdu(&buf[cursor..cursor+24+header.length], ...)`.
   - `cursor += 24 + header.length`.
2. If `buf.len() - cursor < 24` (partial header): stash `buf[cursor..]` into carry.
3. After loop: `flow.carry = buf[cursor..]` (remaining partial frame bytes).

**Carry-buffer cap:**
4. After any carry stash: if `flow.carry.len() > MAX_ENIP_CARRY_BYTES`:
   - `flow.is_non_enip = true`.
   - `flow.parse_errors += 1`.
   - `flow.carry` is cleared (or left up to `MAX_ENIP_CARRY_BYTES` — implementation choice
     consistent with preventing unbounded growth).
5. All subsequent `on_data` calls with `flow.is_non_enip == true` are immediate no-ops:
   no parsing, no counter updates, no findings emitted.

## Invariants

1. **Carry is bounded**: `flow.carry.len() <= MAX_ENIP_CARRY_BYTES = 600` after every
   `on_data` call on a non-bailed flow. Exceeding the cap triggers the `is_non_enip` latch.
2. **parse_errors is lifetime**: `flow.parse_errors` is incremented on every structural
   reject (invalid frame, carry overflow) and is NEVER reset at any window boundary. It is
   the lifetime counter reported by `summarize()`.
3. **malformed_in_window is windowed**: `flow.malformed_in_window` is incremented in parallel
   with `parse_errors` on every structural reject. It is reset at window expiry (BC-2.17.018).
4. **is_non_enip latches ONLY on carry-buffer overflow**: `is_non_enip` is set to `true`
   exclusively when `flow.carry.len() > MAX_ENIP_CARRY_BYTES` after a stash. It is NOT set
   when a single frame's declared `header.length` implies `total_frame_len > MAX_ENIP_CARRY_BYTES`
   (600). An oversized declared frame is handled by the frame-skip path (Postcondition 1 body:
   `parse_errors += 1; malformed_in_window += 1; cursor advances past the declared frame,
   bounded by buf.len(); continue`). Once set to `true`, `is_non_enip` is never cleared.
   This mirrors the DNP3 `is_non_dnp3` pattern.
5. **Cursor arithmetic is valid**: all slice indices in the frame-walk loop are bounds-checked
   before access. The cursor is never advanced past `buf.len()`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | New data is exactly one complete ENIP frame (no carry) | Frame processed; carry = empty |
| EC-002 | New data is a complete frame + partial second frame | First frame processed; partial stashed in carry |
| EC-003 | Carry grows to exactly 600 bytes | Cap not yet exceeded; next on_data may complete the frame |
| EC-004 | Carry would grow to 601 bytes | `is_non_enip=true`; `parse_errors++`; all future calls no-ops |
| EC-005 | is_non_enip already true when on_data called | Immediate no-op; no parse, no counter update |
| EC-006 | Large ENIP payload (`header.length = 600 - 24 = 576 bytes`) fills carry exactly | Carry = 600 bytes; cap not exceeded; complete frame arrives on next on_data and is processed |
| EC-007 | Invalid ENIP header (command not in known set): byte-advance by 1 | `parse_errors++`; `malformed_in_window++`; cursor advances by 1 (byte-walk resync) |
| EC-008 | Valid ENIP header with `header.length = 600` (total_frame = 624 > MAX_ENIP_CARRY_BYTES=600) | Frame-skip: `parse_errors++`; `malformed_in_window++`; cursor advances by `min(624, buf.len()-cursor)`; continue walk; `is_non_enip` NOT set |
| EC-009 | Two complete ENIP frames in one `on_data` call (buf = frame1 + frame2, each 28 bytes) | Both frames processed in the same frame-walk iteration: `process_pdu` called twice; `pdu_count += 2`; carry = empty after loop |

## Canonical Test Vectors

| Scenario | carry before | new data | Expected carry after | is_non_enip? |
|----------|-------------|---------|---------------------|-------------|
| First 12 bytes of ENIP header | `[]` | 12 bytes | 12 bytes | false |
| Complete frame (600 bytes total, header.length=576) | 0 bytes | 600 bytes | 0 bytes | false |
| Partial frame stash grows from 580 to 601 bytes | 580 bytes | 21 bytes | cap triggered | true |
| `is_non_enip=true` on entry | any | any | unchanged | true (no-op) |
| Oversized declared frame (header.length=600; total=624) in buffer of 624 bytes | 0 bytes | 624 bytes | 0 bytes (cursor advanced 624) | false (frame-skip, not overflow) |
| Two complete frames (28 bytes each) in one on_data call | 0 bytes | 56 bytes | 0 bytes (both processed) | false |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-032 | Sub-A (indirect): `parse_enip_header` never panics in the frame-walk loop | Kani Sub-A |
| (none) | Frame-walk loop, carry-cap, is_non_enip latch: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — the carry-buffer frame-walk loop is the central stream reassembly mechanism for the EtherNet/IP analyzer; the MAX_ENIP_CARRY_BYTES=600 cap is the primary defense against per-flow memory exhaustion (ADR-010 Decision 3 rationale) |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 3 (carry-buffer cap), Decision 4 (frame-walk algorithm) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — carry-buffer / frame-walk infrastructure; no finding emission directly) |

## Related BCs

- BC-2.17.001 — composes with (parse_enip_header None on short input triggers carry stash)
- BC-2.17.003 — composes with (is_valid_enip_frame false triggers parse_errors++)
- BC-2.17.018 — composes with (malformed_in_window counter accumulation and T0814 detection)
- BC-2.17.022 — composes with (MAX_FINDINGS cap on downstream findings)

## Architecture Anchors

- `src/analyzer/enip.rs` — `EnipAnalyzer::on_data()` — frame-walk loop
- `src/analyzer/enip.rs` — `const MAX_ENIP_CARRY_BYTES: usize = 600`
- `src/analyzer/enip.rs` — `EnipFlowState.carry: Vec<u8>`
- `src/analyzer/enip.rs` — `EnipFlowState.is_non_enip: bool`
- `src/analyzer/enip.rs` — `EnipFlowState.parse_errors: u64` (lifetime)
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 3` (carry-buffer cap rationale)
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 4` (frame-walk algorithm)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-032 Sub-A (indirect — parse_enip_header safety within the loop)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 3 (600-byte cap rationale); ADR-010 Decision 4 (frame-walk pseudocode); architecture-delta.md §4.1 |
| **Confidence** | high — carry-buffer cap is explicit architectural decision; frame-walk algorithm is specified in ADR-010 |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow.carry, flow.is_non_enip, flow.parse_errors, flow.malformed_in_window |
| **Deterministic** | yes — same byte sequence produces same state |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (on_data core loop) |
