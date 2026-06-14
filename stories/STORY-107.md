---
document_type: story
story_id: STORY-107
epic_id: E-15
version: "1.3"
status: completed
producer: story-writer
timestamp: 2026-06-10T00:00:00Z
phase: 3
points: 5
priority: P0
depends_on: [STORY-106]
blocks: [STORY-108]
behavioral_contracts:
  - BC-2.15.016
verification_properties:
  - VP-023
tdd_mode: strict
target_module: analyzer/dnp3
subsystems: [SS-15]
wave: 36
estimated_days: 2
feature_id: issue-008-dnp3-analyzer
github_issue: 8
# BC status: BC-2.15.016 authored at v1.3 as of 2026-06-12
inputs:
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.016.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
  - .factory/specs/verification-properties/vp-023-dnp3-parse-safety.md
input-hash: "df8fb38"
---

# STORY-107: DNP3 Per-Flow State + Carry Buffer + Pending-Request Bounds

## Narrative

- **As a** ICS/OT security analyst using wirerust against DNP3 TCP captures
- **I want** the DNP3 analyzer to safely manage per-flow reassembly state: a carry buffer bounded at 292 bytes, a master-address tracker bounded at 64 entries, and a pending-request table bounded at 256 entries with oldest-eviction on overflow
- **So that** adversarial DNP3 traffic (partial frames, spoofed source floods, unanswered control floods) cannot exhaust analyzer memory, and downstream detection logic has clean per-frame boundaries to work with

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.15.016 | Per-Flow State Bounds — Carry Buffer ≤292 B, master_addrs ≤64, pending_requests ≤256 |

## Acceptance Criteria

### AC-001 (traces to BC-2.15.016 postcondition 1/2 — carry buffer accumulate and cap)
Incoming bytes are appended to `flow.carry` on each `on_data` call. `flow.carry.len()` NEVER exceeds `MAX_DNP3_FRAME_LEN = 292` bytes. When `carry.len() + new_bytes.len() > 292`, excess bytes beyond 292 are discarded and `flow.parse_errors` is incremented.
- **Test:** `test_carry_buffer_cap_at_292()` — deliver 290-byte carry + 5-byte segment; assert carry=292, parse_errors=1, 3 bytes discarded.

### AC-002 (traces to BC-2.15.016 postcondition 3/4 — frame consumption from carry)
When `flow.carry.len() >= compute_dnp3_frame_len(flow.carry[2])`, the frame is parsed and consumed: `flow.carry.drain(..frame_len)`. Remaining bytes (start of next frame) stay in carry.
- **Test:** `test_carry_buffer_frame_consumption()` — deliver 21-byte carry containing one 10-byte frame + 11-byte partial second frame; assert frame consumed, 11 bytes remain in carry, `frame_count=1`.

### AC-003 (traces to BC-2.15.016 postcondition 5/6 — master_addrs_seen bounded)
When a frame with DIR=1 (master-direction, `control & 0x80 != 0` — DIR is bit 7 per IEEE 1815 DNP3 link-layer framing; mask 0x80 is correct, 0x10 is FCV/DFC and incorrect) is observed, `src` is appended to `flow.master_addrs_seen` if not already present. `flow.master_addrs_seen.len()` NEVER exceeds `MAX_MASTER_ADDRS = 64`. Once full, new source addresses are silently ignored. Canonical master frame CTRL=0xC4: `0xC4 & 0x80 = 0x80 != 0` → `is_master_frame(0xC4) = true`.
- **Test:** `test_master_addrs_cap_at_64()` — insert 64 unique source addresses then a 65th; assert vec len stays at 64.
- **Test:** `test_is_master_frame_uses_0x80_mask()` — assert `is_master_frame(0xC4) == true` (CTRL=0xC4, canonical master frame); assert `is_master_frame(0x44) == false` (CTRL=0x44, DIR=0); assert `is_master_frame(0x10) == false` (bit-4 set, bit-7 clear — old wrong mask would have returned true; 0x10 & 0x80 == 0 → false).

### AC-004 (traces to BC-2.15.016 postcondition 7 — frame_count incremented)
`flow.frame_count` increments by 1 for each complete frame processed through the carry-consume cycle.
- **Test:** `test_frame_count_increments()` — deliver 3 complete frames; assert `frame_count=3`.

### AC-005 (traces to BC-2.15.016 postconditions 8/9/10 — pending_requests bounded at 256 with eviction)
`flow.pending_requests: HashMap<(u16, u8), u32>` (key=(dest_addr, app_seq), value=request_ts) NEVER exceeds `MAX_PENDING_REQUESTS = 256` entries. When a new Control-class request would be inserted AND the map is full, the entry with the smallest `request_ts` (oldest) is evicted before the new entry is inserted. After eviction the map remains at 256 entries. The evicted entry generates NO T1691.001 timeout event.
- **Test:** `test_pending_requests_eviction_at_256()` — insert 256 entries with timestamps 0..=255; insert entry 257 with ts=300; assert map.len()==256; assert oldest entry (ts=0) is evicted.
- **STORY-107 scope boundary (pending_requests seeding):** AC-005 verifies the bounded-insert + oldest-eviction MECHANISM (≤256, min request_ts evicted, no timeout event emitted). The minimal seed that drives this through on_data is intentionally decoupled from the sync-gated carry-walk and may seed spurious/duplicate entries — harmless here because STORY-107 contracts only the bound and no detection reads pending_requests yet. Detection-driven seeding (which entries are inserted, gated on a validated Control-class request frame) is STORY-108 scope; STORY-108 relocates seeding onto the gate-validated frame inside the carry walk.

### AC-006 (traces to BC-2.15.016 invariant 1 — VP-023 Sub-D guarantees in-bounds indexing)
Carry-buffer frame-consumption indexing uses `compute_dnp3_frame_len(length_byte)` (proved in VP-023 Sub-D to return values in [10,292]). Carrying this invariant forward: `flow.carry.drain(..frame_len)` can never index out of bounds when `flow.carry.len() >= frame_len` and `frame_len <= 292`.
- **Note:** This is enforced by the VP-023 Sub-D proof (STORY-106). Story-107 must rely on it, not re-prove it. The test verifies the carry-drain path does not panic on the boundary case (LENGTH=5 frame → frame_len=10).

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `Dnp3FlowState.carry: Vec<u8>` | `src/analyzer/dnp3.rs` | Effectful shell state |
| `Dnp3FlowState.master_addrs_seen: Vec<u16>` | `src/analyzer/dnp3.rs` | Effectful shell state |
| `Dnp3FlowState.pending_requests: HashMap<(u16,u8),u32>` | `src/analyzer/dnp3.rs` | Effectful shell state |
| `Dnp3FlowState.frame_count: u64` | `src/analyzer/dnp3.rs` | Effectful shell state |
| `Dnp3FlowState.parse_errors: u64` | `src/analyzer/dnp3.rs` | Effectful shell state (LIFETIME counter; never reset at window expiry) |
| `const MAX_DNP3_FRAME_LEN: usize = 292` | `src/analyzer/dnp3.rs` | Constant |
| `const MAX_MASTER_ADDRS: usize = 64` | `src/analyzer/dnp3.rs` | Constant |
| `const MAX_PENDING_REQUESTS: usize = 256` | `src/analyzer/dnp3.rs` | Constant |
| `fn is_master_frame(control: u8) -> bool` — tests `control & 0x80 != 0` (DIR bit 7, IEEE 1815; mask 0x80; F-F5-001 R2 fix) | `src/analyzer/dnp3.rs` | Pure (helper) |
| `Dnp3Analyzer::on_data` carry-loop | `src/analyzer/dnp3.rs` | Effectful shell |

Architecture section references: `architecture/module-decomposition.md` (SS-15 `Dnp3FlowState` struct), ADR-007 Decision 2 (carry-buffer pattern, 292 max).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Partial frame delivery (7 bytes of a 10-byte header) | Bytes in carry; no frame parse attempted |
| EC-002 | Two complete frames in one on_data call | Both frames parsed and consumed; carry empty |
| EC-003 | Carry reaches 291 bytes; on_data delivers 2 bytes | 1 accepted (total=292); 1 discarded; parse_errors++ |
| EC-004 | Bailed flow (`is_non_dnp3=true`) receives bytes | Immediate no-op; carry NOT updated (BC-2.15.009) |
| EC-005 | `pending_requests` at 256, two entries share minimum ts | Either may be evicted (tie-breaking implementation-defined per BC-2.15.016 postcondition 9) |
| EC-006 | `flow.carry[2]` (LENGTH byte) = 4 (invalid, <5) | Validity gate (BC-2.15.004) fires; parse_errors++; carry advanced past this frame |

## Tasks

1. **Complete `Dnp3FlowState` struct** — add all fields: `carry: Vec<u8>`, `master_addrs_seen: Vec<u16>`, `pending_requests: HashMap<(u16,u8),u32>`, `frame_count: u64`, `parse_errors: u64` (plus placeholders for detection fields in STORY-108/109).
2. **Add constants** — `MAX_DNP3_FRAME_LEN = 292`, `MAX_MASTER_ADDRS = 64`, `MAX_PENDING_REQUESTS = 256`.
3. **Implement carry-buffer accumulate-and-cap** in `on_data` — append bytes; if `carry.len() > 292`, discard excess and increment `parse_errors`.
4. **Implement frame-consumption loop** — while `carry.len() >= compute_dnp3_frame_len(carry[2])`, consume the frame.
5. **Implement master-addr tracking** — `is_master_frame(control: u8) -> bool` helper: tests `control & 0x80 != 0` (DIR = bit 7, mask 0x80 per IEEE 1815; mask 0x10 is FCV/DFC and MUST NOT be used — F-F5-001 REVISION 2 fix); cap-guarded push.
6. **Implement pending-request insert with eviction** — before insert when at cap, find and remove entry with minimum `request_ts`.
7. **Unit tests** for AC-001 through AC-006 + edge cases.

## Test Plan

| AC | Test Type | Notes |
|----|-----------|-------|
| AC-001 | Unit | Carry cap; parse_errors increment |
| AC-002 | Unit | Frame consumption; drain; frame_count |
| AC-003 | Unit | master_addrs_seen cap |
| AC-004 | Unit | frame_count increments |
| AC-005 | Unit | pending_requests eviction at 256 |
| AC-006 | Unit | Carry-drain boundary (LENGTH=5 minimum frame) |

## Previous Story Intelligence

STORY-103 (Modbus Flow State, E-14) is the direct structural precedent:
- Modbus `ModbusFlowState` has `carry: Vec<u8>` bounded at 260 bytes (Modbus max ADU), `pending_requests` map, and frame-consumption loop — same pattern, different constant (292 vs 260).
- Key DNP3 difference: CRC bytes interleaved in the carry buffer are NOT stripped here (ADR-007 Decision 3 CRC-skip is done during frame parsing, not accumulation); the carry buffer holds the raw wire bytes including CRC octets.
- Lesson from STORY-103: implement carry-consume as a `while` loop (not `if`), because a single `on_data` call may deliver multiple complete frames. Modbus found a bug where `if` was used initially, causing frame alignment drift.
- `parse_errors` is a **lifetime** counter (never reset). This differs from `malformed_in_window` (windowed, added in STORY-109). Do not confuse them.

## Architecture Compliance Rules

Derived from `architecture/module-decomposition.md` (SS-15) and ADR-007 Decision 2:
1. **`carry` cap is 292 bytes** — `MAX_DNP3_FRAME_LEN = 292` is the maximum on-wire DNP3 link frame (proven by VP-023 Sub-D). Excess bytes signal protocol violation; discard and increment `parse_errors`.
2. **Frame consumption uses `compute_dnp3_frame_len`** — never a manual offset. The VP-023 Sub-D proof guarantees this returns values in [10,292], so `carry.drain(..frame_len)` cannot panic when `carry.len() >= frame_len`.
3. **`parse_errors` is LIFETIME** — it is incremented by carry overflow, validity gate reject, and sync-loss bail. It is NEVER reset at the 300s correlation window expiry (BC-2.15.015). `malformed_in_window` is the windowed counter (added in STORY-109).
4. **`is_master_frame` uses mask 0x80 (DIR = bit 7 per IEEE 1815)** — `is_master_frame(control: u8) -> bool` MUST test `control & 0x80 != 0`. Mask 0x10 is FCV/DFC (bit 4) and is WRONG; using 0x10 would cause DIR=1 master frames (e.g., CTRL=0xC4) to be misclassified as non-master. This is the F-F5-001 REVISION 2 R2-1 fix (BC-2.15.016 v1.3 postcondition 5 IMPLEMENTATION NOTE).
5. **Pending-request eviction is by minimum `request_ts`** — oldest entry evicted. The evicted entry generates NO T1691.001 timeout event. This is the DoS-safe overflow behavior.
6. **Forbidden dependencies**: `src/analyzer/dnp3.rs` MUST NOT depend on `src/analyzer/modbus.rs`.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `std::collections::HashMap` | stdlib | `pending_requests` and (later) `fc_counts` |
| `std::collections::HashMap` min-find | stdlib `.iter().min_by_key()` | Oldest-eviction implementation |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/analyzer/dnp3.rs` | Modify | Expand `Dnp3FlowState`; add constants; implement carry loop + master-addr + pending-request logic |
| `tests/dnp3_flow_state_tests.rs` OR inline `#[cfg(test)]` | Create/expand | Unit tests for AC-001..AC-006 |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~2,500 |
| BC-2.15.016 | ~3,500 |
| ADR-007 (Decision 2 carry section) | ~1,500 |
| VP-023 (Sub-D carry-bounds context) | ~1,000 |
| STORY-106 (parse-core, prior story) | ~3,000 |
| STORY-103 (Modbus precedent) | ~2,500 |
| Existing `src/analyzer/dnp3.rs` (from STORY-106) | ~2,000 |
| Tool outputs | ~1,500 |
| **Total estimated** | **~17,500** |

Well within 20-30% of agent context window.

## Dependency Rationale

- `depends_on: [STORY-106]` — `compute_dnp3_frame_len` (VP-023 Sub-D), `is_valid_dnp3_frame_header`, `Dnp3FlowState.is_non_dnp3`, and the `src/analyzer/dnp3.rs` module all come from STORY-106. The carry-consume loop calls `compute_dnp3_frame_len`; it cannot exist before the function does.
- `blocks: [STORY-108]` — STORY-108 (direct detection emissions) adds detection fields to `Dnp3FlowState` and writes detection branches inside the `on_data` loop that the carry buffer manages. The carry loop must be functional first.

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| v1.0 | 2026-06-10 | story-writer | Initial decomposition |
| v1.1 | 2026-06-11 | story-writer | input-hash hygiene — remove duplicate TBD key + regenerate after VP-023 v1.4 (STORY-106 delivery) |
| v1.2 | 2026-06-11 | story-writer | adversarial Pass-1 F-1 — document pending-seed scope boundary (STORY-108 owns detection-driven seeding) |
| v1.3 | 2026-06-12 | story-writer | F7 input-hash reconciliation — BC-2.15.016 v1.3 F5-R2 fix: correct is_master_frame mask 0x10→0x80 (DIR=bit 7, IEEE 1815; F-F5-001 REVISION 2 R2-1); updated AC-003, Task 5, Architecture Compliance Rule 4, Architecture Mapping table, and added test_is_master_frame_uses_0x80_mask() |
