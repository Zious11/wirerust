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
  - "v1.1: Pass-3 adversarial fix HIGH-2: added pending_requests DoS bound — MAX_PENDING_REQUESTS=256 entries. On insert when at cap, evict the OLDEST entry (minimum request_ts). Postconditions 8–10 added; Invariant 5 added; EC-008 added; canonical test vector added; Architecture Anchors and Description updated. BC-2.15.014 Invariant 8 now correctly cross-references this BC for the pending_requests cap. — 2026-06-10"
  - "v1.2: EC-007 resync policy updated — drain-1 (STORY-107 v1 behavior) replaced by byte-walk-forward resync (STORY-109 realization of the STORY-107 explicitly deferred resync). STORY-107 in-code comment stated: 'Byte-walk resync on mid-carry sync-loss is deferred to a later detection story'; STORY-109 is that story. EC-007 now specifies: after the LENGTH gate increments parse_errors and malformed_in_window, the carry head is repositioned by scanning from index 1 for the next [0x05,0x64] sync word; bytes before it are drained; if none found, carry is cleared. No postcondition or invariant logic changed — this is an EC-007 navigation-detail clarification only. Authorized by STORY-109-resync-adjudication.md Decision 2. — 2026-06-11. Additionally (per ADJ-001-A): Canonical Test Vectors 'Carry overflow (adversarial)' row clarified to note that the frame-walk subsequently runs post-overflow and, if no [0x05,0x64] sync word is found, byte-walk-forward resync clears the carry (final carry may be empty); the 292-cap proof rests on the parse_errors increment, not residual carry length."
  - "v1.3: F5-R2 changes (F-F5-001 REVISION 2 + F-F5-003 REVISION 2) — (A1) Postcondition 5 corrected: DIR bit is bit 7 (mask 0x80) per IEEE 1815 DNP3 link-layer framing — the previous text implied mask 0x10 (bit 4, FCV/DFC), which is wrong; CTRL=0xC4 canonical master frame now correctly returns is_master_frame=true. Architecture Anchors updated to note the 0x80 mask. (B7) EC-007 inline-resync-location clarification added: the LENGTH-gate arm performs byte-walk-forward resync INLINE before continue, so the loop's next iteration begins with a valid sync head or empty carry; the sync-check arm is NOT entered as a consequence of a LENGTH-gate drain. (B8) EC-004 Edge Cases row and Canonical Test Vectors 'Carry overflow (adversarial)' row updated to reflect that the overflow arm now performs INLINE resync (identical to Change 2) — a recoverable valid head frame is preserved; carry is cleared only if no [0x05,0x64] sync word is found; the frame-walk then runs on the repositioned carry; the sync-check arm is NOT entered as a consequence of the overflow. (B9) EC-009 added (new): junk-at-clean-boundary counted as one structural malformed event via the sync-check arm. — 2026-06-12"
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

# BC-2.15.016: Per-Flow State Bounds — Carry Buffer ≤292 B, master_addrs ≤64, pending_requests ≤256

## Description

`Dnp3Analyzer` maintains per-flow state in `self.flows: HashMap<FlowKey, Dnp3FlowState>`.
Each flow's carry buffer (`flow.carry: Vec<u8>`) accumulates partial TCP segments until a
complete DNP3 link frame boundary is available, then processes the complete frame. The carry
buffer is bounded to `MAX_DNP3_FRAME_LEN = 292 bytes` per flow — excess bytes are discarded.
The `master_addrs_seen` vec is bounded to `MAX_MASTER_ADDRS = 64` entries per flow. The
`pending_requests` map is bounded to `MAX_PENDING_REQUESTS = 256` entries per flow — when
full, the oldest entry (smallest `request_ts`) is evicted before inserting the new request.
These three bounds collectively prevent unbounded memory growth under adversarial traffic.

## Preconditions

1. `flow` is a `Dnp3FlowState` associated with an active TCP flow.
2. `on_data` delivers new bytes from the reassembled TCP stream.
3. `flow.is_non_dnp3 == false` (bailed flows do not grow the carry buffer).
4. (For pending_requests postconditions 8–10) A Control-class request tracking insert or lookup is being performed by the request/response correlation logic (BC-2.15.014).

## Postconditions

**Carry buffer management:**
1. Incoming bytes are appended to `flow.carry` on each `on_data` call.
2. `flow.carry.len()` NEVER exceeds `MAX_DNP3_FRAME_LEN = 292` bytes. When `carry.len() + new_bytes.len() > 292`, excess bytes beyond 292 are discarded and `flow.parse_errors` is incremented.
3. When `flow.carry.len() >= compute_dnp3_frame_len(flow.carry[2])` (enough bytes for a complete frame), the frame is parsed and consumed from the carry buffer: `flow.carry.drain(..frame_len)`.
4. After frame consumption, remaining bytes (start of the next frame) stay in the carry buffer.

**Bounded master-address tracking:**
5. When a frame with DIR=1 (master-direction, `is_master_frame(control)`) is observed, `src` is appended to `flow.master_addrs_seen` if not already present.
   IMPLEMENTATION NOTE: DIR is bit 7 of the link-control byte (mask 0x80), per IEEE 1815 DNP3 link-layer framing. `is_master_frame(control)` tests `control & 0x80 != 0`. Mask 0x10 is FCV/DFC (bit 4), NOT DIR. Canonical master frame CTRL=0xC4: `0xC4 & 0x80 = 0x80 != 0` → `is_master_frame(0xC4) = true`. This is a correction from a pre-existing bug where the implementation used mask 0x10 (F-F5-001 REVISION 2 R2-1 — F-A-001 BLOCKER fix).
6. `flow.master_addrs_seen.len()` NEVER exceeds `MAX_MASTER_ADDRS = 64`. Once full, new source addresses are silently ignored (not appended).

**Flow counter updates:**
7. `flow.frame_count += 1` for each complete frame processed.

**Bounded pending-request tracking:**
8. `flow.pending_requests.len()` NEVER exceeds `MAX_PENDING_REQUESTS = 256` entries.
9. When a new Control-class request is about to be inserted into `flow.pending_requests` AND
   `flow.pending_requests.len() == MAX_PENDING_REQUESTS`: the entry with the smallest
   `request_ts` value (oldest insertion, by the u32 second timestamp stored as the map value)
   is evicted before the new entry is inserted. If multiple entries share the minimum
   `request_ts`, any one of them may be evicted (tie-breaking is implementation-defined).
   After eviction, the map has 255 entries; the new entry is then inserted, restoring it to
   256 entries.
10. The evicted entry is silently dropped — no T1691.001 timeout-event is generated for the
    evicted entry. This is the DoS-safe overflow behavior: attacker-injected request floods
    are absorbed by eviction rather than unbounded memory growth. (Normal traffic never reaches
    256 simultaneous pending requests per flow; eviction signals an adversarial traffic pattern.)

## Invariants

1. **Carry buffer bounded at 292 bytes** [ADR-007 Decision 2]: `MAX_DNP3_FRAME_LEN = 292` is the maximum on-wire DNP3 link frame size. Bounding carry to 292 means no more than one frame can be over-accumulated. Excess signals protocol violation or misclassified flow.
2. **master_addrs_seen bounded at 64** [dnp3-architecture-delta.md §2.3]: prevents unbounded Vec growth on adversarial traffic spoofing many source addresses. 64 entries is sufficient for any realistic DNP3 segment topology.
3. **Frame consumption uses compute_dnp3_frame_len**: the carry-buffer consume boundary is computed by `compute_dnp3_frame_len(length_byte)` (BC-2.15.007), which is proven safe [VP-023 Sub-D]. The carry indexing never goes out of bounds because `compute_dnp3_frame_len` returns `None` for `length < 5` (handled by the validity gate BC-2.15.004) and always returns a value ≤ 292 (guaranteed by VP-023 Sub-D).
4. **Single-threaded**: `self.flows` HashMap is accessed from a single thread; no concurrent modification.
5. **pending_requests bounded at 256** [architecture-delta.md §2.3, const MAX_PENDING_REQUESTS=256]: prevents unbounded HashMap growth when an attacker sends a flood of unanswered Control-class requests. Eviction of the oldest entry mirrors the DoS-bound pattern used in the Modbus pending-table design (see BC-2.14.x). At most one eviction occurs per insert; the map oscillates at exactly 256 entries under adversarial saturation. Normal legitimate DNP3 control traffic (SELECT/OPERATE pairs with ~3–10s SBO dwell time) accumulates at most a handful of pending entries per flow — reaching 256 is a strong indicator of either a replay attack or a mis-tuned capture environment.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Single on_data call delivers a partial frame (< 10 bytes) | Bytes accumulated in carry; no frame parse attempted |
| EC-002 | Single on_data call delivers exactly one complete frame | Frame parsed and consumed; carry empty after |
| EC-003 | Single on_data call delivers one complete frame + start of a second | First frame parsed; remaining bytes stay in carry |
| EC-004 | Carry reaches 291 bytes (1 byte short of 292); on_data delivers 2 more bytes | 1 byte accepted (total=292); 1 byte discarded; `parse_errors++`; `malformed_in_window++`; overflow arm performs inline resync (byte-walk-forward): if carry head is `[0x05, 0x64, ...]`, carry preserved at head (valid head frame recoverable); if carry is all-junk with no `[0x05, 0x64]`, carry cleared. Frame-walk then runs on the repositioned carry. The sync-check arm is NOT entered as a consequence of the overflow (no double-count). |
| EC-005 | `master_addrs_seen` already has 64 entries; new master source addr arrives | Silently ignored; vec stays at 64 entries |
| EC-006 | Desync-bailed flow (`is_non_dnp3=true`); on_data delivers bytes | Immediate no-op; carry NOT updated (per BC-2.15.009) |
| EC-007 | `flow.carry[2]` (LENGTH byte) is invalid (< 5) after partial accumulation | Validity gate (BC-2.15.004) handles this; `parse_errors++` (lifetime) and `malformed_in_window++` (windowed, per BC-2.15.024); then carry advanced via byte-walk-forward resync: scan carry from index 1 for the next `[0x05, 0x64]` sync word; drain all bytes before it if found; if no sync word found, clear carry entirely. No further `parse_errors` or `malformed_in_window` increment occurs during resync navigation — the error was already counted at the LENGTH gate. The LENGTH-gate arm performs this resync navigation INLINE before `continue`, so the loop's next iteration begins with a valid sync head or an empty carry; the sync-check arm is NOT entered as a consequence of a LENGTH-gate drain (no double-count across iterations). The carry-clear on no-sync-found does NOT set `is_non_dnp3 = true`. Each non-break iteration drains ≥1 byte; carry bounded ≤292 bytes; loop terminates. This replaces the STORY-107 v1 drain-1 behavior for this path (STORY-109 realization; authorized by STORY-109-resync-adjudication.md Decision 2). |
| EC-008 | `pending_requests` already has 256 entries; new Control-class request arrives | Oldest entry (minimum request_ts) evicted; new entry inserted; map stays at 256 entries. No timeout-event generated for evicted entry. |
| EC-009 | After a clean frame consume (`carry.drain(..frame_len)`), carry head is immediately non-sync (junk injected at frame boundary, or corruption) | `parse_errors++` (lifetime); `malformed_in_window++` (windowed, per BC-2.15.024); byte-walk-forward resync locates next `[0x05, 0x64]` or clears carry; if `malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD`, T0814 emitted (BC-2.15.024). This counts as one structural malformed event. The sync-check arm is entered ONLY from Path B (clean consume → junk head) — it is NOT entered after a LENGTH-gate or overflow-arm reject (those arms perform inline resync that leaves a valid head or empty carry before continue). Attacker-crafted fake-sync `[0x05, 0x64, invalid-LENGTH]` floods crossing the malformed threshold are INTENDED T0814 (Possible/Low) Crain-Sistrunk-probe behavior (no de-dup); each embedded fake-sync triplet is a distinct counter-arm entry per Principle 1 ("one per arm entry"). (F-F5-003 REVISION 2 R2-SECTION 3 + R2-SECTION 2 Principle 1) |

## Canonical Test Vectors

| Scenario | Carry state before | on_data bytes | Expected carry state after |
|----------|---------------------|--------------|---------------------------|
| Partial frame | [] (empty) | 5 bytes of a 10-byte header | carry = [5 bytes]; no frame processed |
| Complete minimum frame | [partial 5 bytes] | 5 more bytes | carry = [] (frame consumed); frame_count=1 |
| Frame + next frame start | [] | 21 bytes (10 + 11) | carry = [11 bytes]; frame_count=1 |
| Carry overflow (adversarial) | [290 bytes] | 5 bytes | 2 bytes appended (292); 3 discarded; `parse_errors++`; `malformed_in_window++`; then INLINE resync within the overflow arm (before falling through to the frame-walk): if a `[0x05,0x64]` sync word is found in the carry, bytes before it are drained (preserving a valid head frame if present); if no sync word found, carry is cleared. The frame-walk then runs on the repositioned carry. The sync-check arm is NOT entered as a consequence of the overflow (no double-count). If carry was all-junk with no sync word: final carry is empty; `parse_errors==1`, `malformed_in_window==1`. (F-F5-003 REVISION 2 — recoverable valid head frame preserved; REVISION 1 Change 3 carry.clear()+return rejected as data-loss defect) |
| pending_requests at cap (adversarial flood) | 256 pending entries; new SELECT (0x03) arrives | Entry with oldest request_ts evicted; new SELECT inserted; map.len() == 256 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-023 | Sub-property D: `compute_dnp3_frame_len` result ≤ 292 proves carry indexing is in-bounds | Kani (Sub-D: result in [10, 292]) |
| (none) | Carry buffer management logic: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — bounded carry buffer, master-address tracking, and pending-request table are the three memory-safety foundations of the DNP3/ICS analyzer; unbounded growth in any of the three would allow an attacker to exhaust analyzer memory by sending adversarial DNP3 traffic (partial frames, spoofed source addresses, or unanswered control floods respectively) |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — memory bounds ensure analyzer stability under adversarial DNP3 traffic; pending_requests cap enforces DoS safety for the request/response correlation table) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24 `Dnp3FlowState`); ADR-007 Decision 2 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — state management BC; no finding emission) |

## Related BCs

- BC-2.15.007 — depends on (`compute_dnp3_frame_len` determines carry consume boundary)
- BC-2.15.009 — composes with (bailed flow does not grow carry)
- BC-2.15.014 — composes with (pending_requests is populated by BC-2.15.014 request tracking; BC-2.15.016 enforces the MAX_PENDING_REQUESTS=256 cap with oldest-eviction)
- BC-2.15.022 — composes with (MAX_FINDINGS cap; this BC defines the carry/address/pending-request caps)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `const MAX_DNP3_FRAME_LEN: usize = 292`
- `src/analyzer/dnp3.rs` — `const MAX_MASTER_ADDRS: usize = 64`
- `src/analyzer/dnp3.rs` — `const MAX_PENDING_REQUESTS: usize = 256`
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.carry: Vec<u8>`, `.master_addrs_seen: Vec<u16>`
- `src/analyzer/dnp3.rs` — `is_master_frame(control: u8) -> bool` tests `control & 0x80 != 0` (DIR bit = bit 7 per IEEE 1815 DNP3 link-layer framing; mask 0x80 is CORRECT; mask 0x10 was a pre-existing bug — F-F5-001 REVISION 2 R2-1 fix)
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.pending_requests: HashMap<(u16, u8), u32>` (key=(dest_addr, app_seq), value=request_ts as u32 seconds; bounded to 256 by eviction)
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §2.2` — constants and struct layout
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §2.3` — MAX_PENDING_REQUESTS=256 constant
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 2` — carry-buffer pattern; max 292 bytes

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-023 — Sub-property D (proves frame_len ≤ 292, guaranteeing carry indexing is in-bounds)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-007 Decision 2; dnp3-architecture-delta.md §2.2 (MAX_DNP3_FRAME_LEN=292, MAX_MASTER_ADDRS=64); dnp3-architecture-delta.md §2.3 (MAX_PENDING_REQUESTS=256); dnp3-research.md §1.4 (292-byte maximum [SPEC]) |
| **Confidence** | high — 292-byte maximum is SPEC-confirmed; bounded-resource design is ADR-007 architectural policy; pending_requests cap mirrors Modbus pending-table DoS-bound pattern |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow.carry, flow.master_addrs_seen, flow.frame_count, flow.parse_errors, flow.pending_requests (eviction enforcement) |
| **Deterministic** | yes — same byte sequence produces same carry state |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell |
