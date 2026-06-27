---
document_type: behavioral-contract
level: L3
version: "2.0"
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
modified:
  - "v1.1: F8-001 — command_counts increment relocated to frame-walk (fires on every successful parse_enip_header, before is_valid_enip_frame); Postcondition 0 added as canonical single command_counts increment site; Invariant 6 added (command_counts vs pdu_count separation of concerns)"
  - "v2.0: RULING-EDGECASE-001 §1 (EC-X1) — per-direction carry split: replace carry: Vec<u8> with carry_c2s: Vec<u8> + carry_s2c: Vec<u8>; on_data gains direction: Direction parameter; Precondition 2 updated (direction-selected carry prepend); Postcondition 3 updated (direction-selected stash); Postcondition 4 updated (cap check on active directional carry); Invariant 1 updated (per-direction ≤600 bound); Invariant 7 added (direction isolation — c2s and s2c carry buffers never mixed); EC-010 added (direction non-contamination — partial c2s + s2c frame → independent parsing, no splice) [F2-correction F-003: propagated per-direction carry field names to Description, PC-1 inner body, and Invariant 4 — doc-fidelity only, no behavior change]"
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
into the direction-selected carry buffer (`carry_c2s` for client→server, `carry_s2c` for
server→client). The carry buffer is bounded to `MAX_ENIP_CARRY_BYTES = 600`.
If the stash would exceed 600 bytes, `flow.is_non_enip` is set to `true` and
`flow.parse_errors` is incremented. All subsequent `on_data` calls for a flow with
`is_non_enip = true` are immediate no-ops.

## Preconditions

1. `EnipAnalyzer::on_data(flow_key, data, now_ts, direction)` is called with new TCP bytes.
   The `direction: Direction` parameter (`crate::reassembly::handler::Direction`) identifies
   whether these bytes come from the TCP initiator (ClientToServer) or responder (ServerToClient).
2. `buf = (match direction { ClientToServer => flow.carry_c2s, ServerToClient => flow.carry_s2c }) ++ data`
   — the directional carry buffer is prepended to new data. The OTHER direction's carry buffer
   is NOT touched. (RULING-EDGECASE-001 §1.2)
3. `flow.is_non_enip == false` at call time (flows with `is_non_enip=true` exit immediately
   at the top of on_data).

## Postconditions

**Frame walk loop:**
1. While `buf.len() - cursor >= 24`:
   - Parse header at `buf[cursor..cursor+24]` (BC-2.17.001/002).
   - **PC-0 — command_counts canonical increment site (F8-001):** If `parse_enip_header` returns `Some(header)`, immediately invoke `classify_enip_command(header.command)` and execute `flow.command_counts.entry(header.command).or_insert(0) += 1`. This fires for **every successfully parsed 24-byte header** — including headers that subsequently fail `is_valid_enip_frame`. This is the **single canonical command_counts increment site** (BC-2.17.004 Inv3). `process_pdu` does NOT increment `command_counts`.
   - If `!is_valid_enip_frame(&header)`: `flow.parse_errors += 1`; `flow.malformed_in_window += 1`; `cursor += 1`; continue (byte-walk resync).
   - **Frame-skip path (oversized declared frame)**: if `24 + header.length as usize > MAX_ENIP_CARRY_BYTES` (600):
     - `flow.parse_errors += 1`; `flow.malformed_in_window += 1`.
     - Advance cursor: `cursor += min(24 + header.length as usize, buf.len() - cursor)` (bounded by remaining buffer).
     - Continue the walk. Do NOT set `is_non_enip`. Do NOT stash into carry.
   - If `buf.len() - cursor < 24 + header.length as usize` (partial frame in buffer): stash `buf[cursor..]` into the directional carry (`carry_c2s`/`carry_s2c` per `direction`); apply cap check; break.
   - Otherwise: call `process_pdu(&buf[cursor..cursor+24+header.length], ...)`.
   - `cursor += 24 + header.length`.
2. If `buf.len() - cursor < 24` (partial header): stash `buf[cursor..]` into the directional carry (`carry_c2s`/`carry_s2c` per `direction`).
3. After loop: `match direction { ClientToServer => flow.carry_c2s = buf[cursor..], ServerToClient => flow.carry_s2c = buf[cursor..] }` — remaining partial frame bytes stashed back into the SAME directional carry buffer that was prepended at call entry. The other direction's carry is never modified. (RULING-EDGECASE-001 §1.2)

**Carry-buffer cap:**
4. After any carry stash: check the carry buffer that was just written (whichever of `carry_c2s`
   or `carry_s2c` is selected by `direction`). If `active_carry.len() > MAX_ENIP_CARRY_BYTES`:
   - `flow.is_non_enip = true`.
   - `flow.parse_errors += 1`.
   - The overflowed directional carry is cleared (or bounded to `MAX_ENIP_CARRY_BYTES` —
     implementation choice consistent with preventing unbounded growth). The other direction's
     carry is unaffected. (RULING-EDGECASE-001 §1.2)

   > **NOTE (RULING-137-002 / RULING-EDGECASE-001 ADDENDUM):** The trigger condition
   > `active_carry_len > MAX_ENIP_CARRY_BYTES` is structurally unreachable under this spec's
   > own algorithm. The maximum possible carry after any on_data call is 599 bytes
   > (RULING-137-002 §1.2). The canonical test vector "580+21=601 → cap triggered" in the
   > Canonical Test Vectors table is mathematically impossible (RULING-137-002 §1.4).
   > Postcondition 4 and EC-004 are retained as belt-and-suspenders defensive code
   > specifications pending a v0.12.0 redesign of the quarantine mechanism. No BC version bump
   > is triggered by this additive note.

5. All subsequent `on_data` calls with `flow.is_non_enip == true` are immediate no-ops:
   no parsing, no counter updates, no findings emitted.

## Invariants

1. **Carry is bounded (per-direction)**: `flow.carry_c2s.len() <= MAX_ENIP_CARRY_BYTES = 600`
   AND `flow.carry_s2c.len() <= MAX_ENIP_CARRY_BYTES = 600` after every `on_data` call on a
   non-bailed flow. Each directional carry buffer is independently bounded. Exceeding the cap
   on EITHER directional buffer triggers the `is_non_enip` latch. (RULING-EDGECASE-001 §1.2)
2. **parse_errors is lifetime**: `flow.parse_errors` is incremented on every structural
   reject (invalid frame, carry overflow) and is NEVER reset at any window boundary. It is
   the lifetime counter reported by `summarize()`.
3. **malformed_in_window is windowed**: `flow.malformed_in_window` is incremented in parallel
   with `parse_errors` on every structural reject. It is reset at window expiry (BC-2.17.018).
4. **is_non_enip latches ONLY on carry-buffer overflow**: `is_non_enip` is set to `true`
   exclusively when `flow.carry_c2s.len() > MAX_ENIP_CARRY_BYTES` OR
   `flow.carry_s2c.len() > MAX_ENIP_CARRY_BYTES` after a stash. It is NOT set
   when a single frame's declared `header.length` implies `total_frame_len > MAX_ENIP_CARRY_BYTES`
   (600). An oversized declared frame is handled by the frame-skip path (Postcondition 1 body:
   `parse_errors += 1; malformed_in_window += 1; cursor advances past the declared frame,
   bounded by buf.len(); continue`). Once set to `true`, `is_non_enip` is never cleared.
   This mirrors the DNP3 `is_non_dnp3` pattern.
5. **Cursor arithmetic is valid**: all slice indices in the frame-walk loop are bounds-checked
   before access. The cursor is never advanced past `buf.len()`.
6. **command_counts vs pdu_count separation of concerns (F8-001)**: `flow.command_counts` is
   incremented in the frame-walk (PC-0 above) for every successfully parsed header, before the
   `is_valid_enip_frame` validity gate. `flow.pdu_count` is incremented inside `process_pdu`
   only for frames that pass the validity gate (BC-2.17.024). These two increment sites are
   independent. Unknown/invalid-command frames contribute to `command_counts[cmd]` but NOT to
   `pdu_count`. This is the SINGLE canonical `command_counts` increment site — `process_pdu`
   must NOT contain a `command_counts` increment.
7. **Direction isolation** (RULING-EDGECASE-001 §1.2 EC-X1): `carry_c2s` and `carry_s2c` are
   NEVER mixed. `on_data` selects exactly one of the two buffers based on the `direction`
   argument on every call. No frame-walk loop ever prepends bytes from one direction into the
   other direction's data stream. A partial c2s frame stashed in `carry_c2s` is NEVER spliced
   with an s2c delivery, and vice versa. This invariant prevents the cross-direction carry
   splice bug documented in EC-X1 (RULING-EDGECASE-001 §1.1 root cause analysis).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | New data is exactly one complete ENIP frame (no carry) | Frame processed; carry = empty |
| EC-002 | New data is a complete frame + partial second frame | First frame processed; partial stashed in carry |
| EC-003 | Carry grows to exactly 600 bytes | Cap not yet exceeded; next on_data may complete the frame |
| EC-004 | Carry would grow to 601 bytes _(NOTE: this trigger is structurally unreachable per RULING-137-002 §1.2; max reachable carry is 599 bytes — retained as belt-and-suspenders; see PC-4 addendum note)_ | `is_non_enip=true`; `parse_errors++`; all future calls no-ops |
| EC-005 | is_non_enip already true when on_data called | Immediate no-op; no parse, no counter update |
| EC-006 | Large ENIP payload (`header.length = 600 - 24 = 576 bytes`) fills carry exactly | Carry = 600 bytes; cap not exceeded; complete frame arrives on next on_data and is processed |
| EC-007 | Invalid ENIP header (command not in known set): byte-advance by 1 | `parse_errors++`; `malformed_in_window++`; cursor advances by 1 (byte-walk resync) |
| EC-008 | Valid ENIP header with `header.length = 600` (total_frame = 624 > MAX_ENIP_CARRY_BYTES=600) | Frame-skip: `parse_errors++`; `malformed_in_window++`; cursor advances by `min(624, buf.len()-cursor)`; continue walk; `is_non_enip` NOT set |
| EC-009 | Two complete ENIP frames in one `on_data` call (buf = frame1 + frame2, each 28 bytes) | Both frames processed in the same frame-walk iteration: `process_pdu` called twice; `pdu_count += 2`; carry = empty after loop |
| EC-010 | Partial c2s frame (12 bytes) stashed in `carry_c2s`; next `on_data` call is `direction=ServerToClient` with a full s2c response frame | `carry_s2c` (empty) is prepended to the s2c data; s2c frame processes cleanly via the frame-walk; `carry_c2s` retains its 12-byte partial c2s bytes unchanged; NO splice, NO spurious finding, NO missed detection. (RULING-EDGECASE-001 §1.2 — direction non-contamination) |

## Canonical Test Vectors

| Scenario | carry before | new data | Expected carry after | is_non_enip? |
|----------|-------------|---------|---------------------|-------------|
| First 12 bytes of ENIP header | `[]` | 12 bytes | 12 bytes | false |
| Complete frame (600 bytes total, header.length=576) | 0 bytes | 600 bytes | 0 bytes | false |
| Partial frame stash grows from 580 to 601 bytes _(NOTE: mathematically impossible per RULING-137-002 §1.4 — max carry is 599 bytes; row retained as historical record, superseded by PC-4 addendum note)_ | 580 bytes | 21 bytes | cap triggered | true |
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
- BC-2.17.004 — depends on (classify_enip_command called at PC-0; this BC is the single canonical command_counts increment site; F8-001)
- BC-2.17.018 — composes with (malformed_in_window counter accumulation and T0814 detection)
- BC-2.17.022 — composes with (MAX_FINDINGS cap on downstream findings)

## Architecture Anchors

- `src/analyzer/enip.rs` — `EnipAnalyzer::on_data()` — frame-walk loop
- `src/analyzer/enip.rs` — `const MAX_ENIP_CARRY_BYTES: usize = 600`
- `src/analyzer/enip.rs` — `EnipFlowState.carry_c2s: Vec<u8>` (RULING-EDGECASE-001 §1.2 — replaces carry: Vec<u8>)
- `src/analyzer/enip.rs` — `EnipFlowState.carry_s2c: Vec<u8>` (RULING-EDGECASE-001 §1.2 — replaces carry: Vec<u8>)
- `src/analyzer/enip.rs` — `EnipFlowState.is_non_enip: bool`
- `src/analyzer/enip.rs` — `EnipFlowState.parse_errors: u64` (lifetime)
- `src/analyzer/enip.rs` — `EnipFlowState.command_counts: HashMap<u16, u64>` — incremented in `on_data()` frame-walk loop (PC-0) immediately after `parse_enip_header` returns `Some`, before `is_valid_enip_frame` check (F8-001 canonical site)
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
| **Global state access** | mutates flow.carry_c2s or flow.carry_s2c (direction-selected), flow.is_non_enip, flow.parse_errors, flow.malformed_in_window, flow.command_counts (PC-0 — canonical single increment site) |
| **Deterministic** | yes — same byte sequence produces same state |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (on_data core loop) |
