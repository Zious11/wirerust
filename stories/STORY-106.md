---
document_type: story
story_id: STORY-106
epic_id: E-15
version: "1.6"
# v1.6 2026-06-12: F7 F-S1-001 reconciliation — AC-009 and Previous Story Intelligence corrected from stale "first 16 bytes of carry data" cross-segment framing to BC-2.15.009 v1.3 initial-delivery-only semantics (carry.is_empty && data.len>=2 && no-offset-0-sync; no cross-segment 16-byte accumulation bail per ADJ-001 Addendum Q1)
# v1.5 2026-06-11: adversarial Pass-7 F1 — add AC-004 BC-2.15.004 PC4 caller-side parse_errors STORY-107 deferral note (symmetry with AC-008/AC-009)
# v1.4 2026-06-11: adversarial Pass-3 F-P3-001 — document BC-2.15.008 link-FC guard in AC-008 + cite has_user_data/RESET_LINK tests (sibling-sweep)
# v1.3 2026-06-11: adversarial Pass-2 B1/B2 scope notes + B4 test citation
status: completed
producer: story-writer
timestamp: 2026-06-10T00:00:00Z
phase: 3
points: 8
priority: P0
depends_on: [STORY-100]
blocks: [STORY-107]
behavioral_contracts:
  - BC-2.15.001
  - BC-2.15.002
  - BC-2.15.003
  - BC-2.15.004
  - BC-2.15.005
  - BC-2.15.006
  - BC-2.15.007
  - BC-2.15.008
  - BC-2.15.009
verification_properties:
  - VP-023
tdd_mode: strict
target_module: analyzer/dnp3
subsystems: [SS-15]
wave: 35
estimated_days: 3
feature_id: issue-008-dnp3-analyzer
github_issue: 8
# BC status: 9 BCs authored 2026-06-10 as draft; pending PO sign-off
inputs:
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.001.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.002.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.003.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.004.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.005.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.006.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.007.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.008.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.009.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
  - .factory/specs/verification-properties/vp-023-dnp3-parse-safety.md
input-hash: "4fcf8aa"
---

# STORY-106: DNP3 DL/Transport Parse + FC Classify — Pure Core (VP-023 Kani)

## Narrative

- **As a** ICS/OT security analyst using wirerust against DNP3 TCP captures
- **I want** the DNP3 analyzer to correctly parse data-link headers, classify all 256 function codes, validate frames via the three-point gate, compute frame-length arithmetic, gate on FIR=1 transport fragments, and bail cleanly on desync — all with Kani formal proofs verifying no panic paths
- **So that** the pure-core parsing and classification layer is formally verified before any stateful flow-correlation or detection logic is built on top of it

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.15.001 | DNP3 DL Header Accepted for Well-Formed 10-Byte-Minimum Frame |
| BC-2.15.002 | DNP3 DL Header Rejected for Frame Shorter Than 10 Bytes (Truncation Safety) |
| BC-2.15.003 | DEST/SOURCE Addresses Decoded Little-Endian from Fixed Offsets 4–7 |
| BC-2.15.004 | Three-Point Validity Gate Returns True Iff Sync==0x0564 and LENGTH>=5 |
| BC-2.15.005 | classify_dnp3_fc Is Total Over All 256 FC Values (No Gap, No Panic) |
| BC-2.15.006 | FC Classification Correctness — Control {0x03,0x04,0x05,0x06}, Restart {0x0D,0x0E}, Write {0x02}, Read {0x01} |
| BC-2.15.007 | compute_dnp3_frame_len Arithmetic Correct; Result in [10,292]; No Overflow |
| BC-2.15.008 | Transport FIR=1 First-Fragment Gates Application-Layer FC Extraction |
| BC-2.15.009 | is_non_dnp3 Desync-Safe Bail — Flow Silenced on Initial-Delivery No-Sync (One-Shot, First Delivery Only) |

## Acceptance Criteria

### AC-001 (traces to BC-2.15.001 postcondition 1 — Some for len>=10)
`parse_dnp3_dl_header(data: &[u8]) -> Option<Dnp3DlHeader>` returns `Some(Dnp3DlHeader)` when `data.len() >= 10`. All six fields decoded from fixed offsets: `start1=data[0]`, `start2=data[1]`, `length=data[2]`, `control=data[3]`, `destination=u16::from_le_bytes([data[4],data[5]])`, `source=u16::from_le_bytes([data[6],data[7]])`. Bytes 8–9 (header CRC) are NOT decoded as struct fields.
- **Canonical vector:** `05 64 0E C4 03 00 01 00 88 C5` → `Some { start1:0x05, start2:0x64, length:14, control:0xC4, destination:0x0003, source:0x0001 }`.
- **Test:** `test_parse_dnp3_dl_header_returns_some_for_minimum_10_bytes()`

### AC-002 (traces to BC-2.15.002 postcondition 1 — None for len<10)
`parse_dnp3_dl_header(data)` returns `None` when `data.len() < 10`. No panic on zero-length input or any input length 0..=9.
- **Canonical vectors:** empty slice → `None`; 9-byte slice → `None`; 10-byte slice → `Some`.
- **Test:** `test_parse_dnp3_dl_header_rejects_truncated_input()`

### AC-003 (traces to BC-2.15.003 postcondition 1/2 — LE address decode)
`Dnp3DlHeader.destination == u16::from_le_bytes([data[4], data[5]])` and `Dnp3DlHeader.source == u16::from_le_bytes([data[6], data[7]])`. Wire bytes `[0x03, 0x00]` decode to 0x0003 (NOT 0x0300). Wire bytes `[0xFF, 0xFF]` decode to 0xFFFF.
- **Test:** `test_parse_dnp3_dl_header_le_address_decode()` — assert `[0x03, 0x00]`→0x0003, `[0xFD, 0xFF]`→0xFFFD, `[0x00, 0x01]`→0x0100 (LE vs BE disambiguation)
- **Kani:** VP-023 Sub-property A verifies LE decode for all 10-byte symbolic inputs.

### AC-004 (traces to BC-2.15.004 postcondition 1 — validity gate biconditional)
`is_valid_dnp3_frame_header(h: &Dnp3DlHeader) -> bool` returns `true` IFF `h.start1==0x05 && h.start2==0x64 && h.length>=5`. Returns `false` if any condition fails: wrong START1 (0x04), wrong START2 (0x63), LENGTH=4 (below minimum).
- **Test:** `test_is_valid_dnp3_frame_header_biconditional()` — 6 vectors covering all partial-match cases.
- **Kani:** VP-023 Sub-property C proves biconditional for all symbolic `Dnp3DlHeader` inputs.
- **STORY-106 scope boundary (BC-2.15.004 PC4 caller obligation):** AC-004 covers the PURE validity-gate biconditional (`is_valid_dnp3_frame_header`) only. BC-2.15.004 Postcondition 4's caller-side obligation — `on_data` must increment `flow.parse_errors` and skip processing when the gate returns false — requires the per-frame gating in the STORY-107 frame-walk and is deferred to STORY-107 (consistent with the `frame_count`-semantics note in AC-008).

### AC-005 (traces to BC-2.15.005 postcondition 2 — totality no panic)
`classify_dnp3_fc(fc: u8) -> Dnp3FcClass` returns exactly one of `{Read, Write, Control, Restart, Management, Response, Unknown}` for every `fc` in 0x00..=0xFF. Never panics. No `unreachable!` macro. Wildcard arm `_ => Dnp3FcClass::Unknown` ensures totality.
- **Test:** `test_classify_dnp3_fc_total()` — spot-check FC=0xFF and FC=0x80 return `Unknown`.
- **Kani:** VP-023 Sub-property B proves totality and returns-defined-variant for all 256 values.

### AC-006 (traces to BC-2.15.006 postconditions 1-11 — set membership)
Control set {0x03, 0x04, 0x05, 0x06} → `Control`. Restart set {0x0D, 0x0E} → `Restart`. Write {0x02} → `Write`. Read {0x01} → `Read`. Response {0x81, 0x82, 0x83} → `Response`. FC 0x07 (IMMED_FREEZE) → `Management`. FC 0x0F (INITIALIZE_DATA) → `Management` (NOT `Restart`). FC 0x06 (DIRECT_OPERATE_NR) → `Control`.
- **Test:** `test_classify_dnp3_fc_set_membership()` — one assertion per listed FC.
- **Kani:** VP-023 Sub-property B set-membership assertions over all 256 symbolic FC values.

### AC-007 (traces to BC-2.15.007 postcondition 2 — frame_len formula)
`compute_dnp3_frame_len(length: u8) -> Option<usize>` returns `None` for `length<5`. For `length` in 5..=255 returns `Some(5 + length + 2*ceil((length-5)/16))`. Boundaries: LENGTH=5→Some(10), LENGTH=6→Some(13), LENGTH=21→Some(28), LENGTH=22→Some(31), LENGTH=255→Some(292). Result always in [10, 292]. No overflow.
- **Test:** `test_compute_dnp3_frame_len_formula()` — 7 canonical vectors including both block boundaries.
- **Kani:** VP-023 Sub-property D proves formula correctness, [10,292] bound, and no-panic over all 256 `u8` values.

### AC-008 (traces to BC-2.15.008 postconditions 1-4 — FIR=1 gating + link-FC guard)
When `transport_octet & 0x40 != 0` (FIR=1): application FC is extracted from `payload_buf[2]`, `classify_dnp3_fc` is called, and `fc_counts`/`fn_code_counts` are updated. When FIR=0 (continuation): no App FC extraction, no finding emission, `frame_count` still incremented. App-FC extraction additionally requires the link CONTROL nibble (`data[3] & 0x0F`) to be CONFIRMED_USER_DATA (0x03) or UNCONFIRMED_USER_DATA (0x04) per BC-2.15.008 precondition 2 / Invariant 4; a FIR=1 fragment in a non-user-data link frame (e.g. RESET_LINK 0x00) is counted but NOT app-extracted (EC-005).
- **Test:** `test_fir_gating_extract_on_fir1_skip_on_fir0()` (predicate) and `test_on_data_fir_gating_updates_counters()` (on_data end-to-end counter update) — transport_octet=0xC0 (FIR=1)→FC extracted; transport_octet=0x80 (FIR=0)→no extraction. Also: `test_has_user_data_link_fc_guard()` (link-FC predicate unit test) and `test_on_data_fir_but_non_user_data_link_fc_no_extraction()` (RESET_LINK 0x00 with FIR=1 — frame counted, app-FC NOT extracted).
- **STORY-106 scope boundary (Finding 5):** This AC covers FIR=1 gating and app-FC extraction for minimum-single-block frames only — the `on_data` FIR-extraction path reads the app FC at the minimum-frame offset. BC-2.15.008 EC-006 (`parse_errors` accounting for payloads &lt;3 bytes) requires the frame-walk and error-accounting infrastructure that is STORY-107 scope; that EC is deferred to STORY-107. Full multi-block CRC-strip indexing for `payload_buf[2]` access is likewise STORY-107.
- **STORY-106 scope boundary (frame_count semantics):** `on_data` increments `frame_count` for sync-valid deliveries (`[0x05, 0x64]` at offset 0) WITHOUT a per-frame validity-gate check. Gating each frame through `is_valid_dnp3_frame_header` before counting (so `frame_count` counts validated frames, not just sync-valid deliveries) is part of the STORY-107 frame-walk. In STORY-106, `frame_count` means "sync-valid delivery count".
- **STORY-106 scope boundary (app-FC extraction minimum frame size):** App-FC extraction occurs only for frames >=13 bytes (the minimum that carries an application FC at byte 12: 10 header + 1 transport + 2 app). FIR=1 frames of 10–12 bytes have no app-layer FC and are counted but not extracted — this is correct; multi-block and short-frame handling is STORY-107 frame-walk scope.

### AC-009 (traces to BC-2.15.009 postconditions 1-6 — desync bail)
When the first `on_data` delivery arrives while `flow.carry.is_empty()`, `data.len() >= 2`, and there is no valid DNP3 sync word `[0x05, 0x64]` at offset 0, `flow.is_non_dnp3` is set to `true`. This is a one-shot, initial-delivery-only mechanism — once carry is non-empty the bail path is permanently closed. All subsequent `on_data` calls for that flow return immediately (no-op). The latch is one-way: `is_non_dnp3` once true is never reset.
- **Test:** `test_desync_bail_non_dnp3_traffic()` — deliver `[0xFF, 0xFE, ...]` first; assert `is_non_dnp3=true`; deliver second segment; assert no findings, no carry growth.
- **STORY-106 scope boundary (Finding 4):** This AC covers the single-delivery desync latch — when the first delivered segment is ≥2 bytes and has no `[0x05, 0x64]` at offset 0, `is_non_dnp3` is set. BC-2.15.009 EC-004 (a sub-2-byte first delivery, e.g. a lone `0x05`, defers because `data.len() < 2` — the bail guard is not satisfied; the byte is accumulated into carry and the bail re-evaluates on the next delivery while carry is still empty; once carry is non-empty the flow is established as DNP3 and the bail path is permanently closed — there is NO cross-segment 16-byte accumulation bail; ADJ-001 Addendum Q1) requires the carry buffer that is STORY-107 scope. The lone-`0x05` re-evaluation on the next delivery while carry is empty, and the permanent closure of the bail path once carry is non-empty, are verified in STORY-107 when the carry buffer lands.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `fn parse_dnp3_dl_header(data: &[u8]) -> Option<Dnp3DlHeader>` | `src/analyzer/dnp3.rs` | Pure (Kani target Sub-A) |
| `fn is_valid_dnp3_frame_header(h: &Dnp3DlHeader) -> bool` | `src/analyzer/dnp3.rs` | Pure (Kani target Sub-C) |
| `fn classify_dnp3_fc(fc: u8) -> Dnp3FcClass` | `src/analyzer/dnp3.rs` | Pure (Kani target Sub-B) |
| `fn compute_dnp3_frame_len(length: u8) -> Option<usize>` | `src/analyzer/dnp3.rs` | Pure (Kani target Sub-D) |
| `fn transport_is_fir(transport_octet: u8) -> bool` | `src/analyzer/dnp3.rs` | Pure (trivial 1-liner; unit test only) |
| `fn has_user_data(control: u8) -> bool` | `src/analyzer/dnp3.rs` | Pure (trivial; unit test only) |
| `struct Dnp3DlHeader` | `src/analyzer/dnp3.rs` | Data type |
| `enum Dnp3FcClass` | `src/analyzer/dnp3.rs` | Data type |
| `Dnp3FlowState.is_non_dnp3: bool` | `src/analyzer/dnp3.rs` | Effectful shell state (desync latch) |
| Kani harness `#[cfg(kani)] mod kani_proofs` | `src/analyzer/dnp3.rs` | Verification (VP-023 Sub-A, B, C, D) |

Architecture section references: `architecture/module-decomposition.md` (SS-15 boundary), `architecture/dependency-graph.md` (SS-15 depends on SS-05 dispatcher, resolved in STORY-110).

## VP-023 Kani Obligation (ALL FOUR SUB-PROPERTIES LAND HERE)

This story is the **anchor story for VP-023**. The implementer MUST author all four Kani harnesses in `src/analyzer/dnp3.rs` under `#[cfg(kani)] mod kani_proofs`:

| Harness | Sub-property | BC Coverage |
|---------|-------------|-------------|
| `verify_parse_dnp3_dl_header_safety` | A — parse safety, None/Some, LE decode | BC-2.15.001, BC-2.15.002, BC-2.15.003 |
| `verify_classify_dnp3_fc_total` | B — totality + set membership | BC-2.15.005, BC-2.15.006 |
| `verify_is_valid_dnp3_frame_gate` | C — validity gate biconditional | BC-2.15.004 |
| `verify_compute_dnp3_frame_len` | D — frame_len arithmetic, [10,292], no overflow | BC-2.15.007 |

All four harnesses use straight-line symbolics (no user loops); expected Kani runtime < 1s each. Mirror the harness skeleton in `vp-023-dnp3-parse-safety.md §Proof Harness Skeleton` exactly.

**Scope note:** VP-023 covers BC-2.15.001..007 only (the four Kani-provable pure functions). BC-2.15.008 (FIR=1 transport gating) and BC-2.15.009 (desync bail) are effectful shell behaviors; they are covered by UNIT TESTS in this story (AC-008, AC-009) but are NOT VP-023 Kani obligations. This story's `bcs:` frontmatter correctly includes all 9 BCs (STORY-106 implements all of them), but only 001-007 contribute to VP-023 Kani coverage.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Zero-length input to `parse_dnp3_dl_header` | Returns `None`; no panic |
| EC-002 | `data` = 9 bytes (one short of minimum) | Returns `None` |
| EC-003 | `data` = 10 bytes, START1=0x00 (invalid sync) | Returns `Some { start1:0x00, ... }`; validity gate rejects separately |
| EC-004 | DEST bytes `[0xFF, 0xFF]` | `destination = 0xFFFF` (broadcast); LE decode `[0xFF,0xFF]` = 65535 |
| EC-005 | `compute_dnp3_frame_len(4)` | `None` (LENGTH below minimum 5) |
| EC-006 | `compute_dnp3_frame_len(255)` | `Some(292)` — U=250, blocks=16, 5+255+32=292 |
| EC-007 | FC 0x06 (DIRECT_OPERATE_NR) | `Dnp3FcClass::Control` — still Control-class even though no response expected |
| EC-008 | FC 0x82 (UNSOLICITED_RESPONSE) | `Dnp3FcClass::Response` |
| EC-009 | FIR=0 continuation (transport_octet=0x80) | No App FC extraction; no findings |
| EC-010 | Valid sync at offset 2 (not offset 0) | Desync bail fires — BC-2.15.009 checks only offset 0 |

## Tasks

1. **Create `src/analyzer/dnp3.rs`** — new module with `pub struct Dnp3DlHeader`, `pub enum Dnp3FcClass`, `pub struct Dnp3FlowState` (is_non_dnp3 field + carry placeholder for STORY-107).
2. **Implement `parse_dnp3_dl_header`** — `if data.len() < 10 { return None; }` guard; decode 6 fields at fixed offsets using `u16::from_le_bytes` for DEST/SRC.
3. **Implement `is_valid_dnp3_frame_header`** — pure 3-clause boolean: `h.start1==0x05 && h.start2==0x64 && h.length>=5`.
4. **Implement `classify_dnp3_fc`** — `match fc` with explicit arms for Control set (0x03..=0x06), Restart set (0x0D, 0x0E), Write (0x02), Read (0x01), Response (0x81..=0x83), Management (0x00, 0x07..=0x1A range), and `_ => Unknown` wildcard.
5. **Implement `compute_dnp3_frame_len`** — `if length < 5 { return None; }` guard; formula: `let u = (length as usize) - 5; let blocks = (u + 15) / 16; Some(5 + length as usize + 2 * blocks)`.
6. **Implement `transport_is_fir` and `has_user_data`** — trivial 1-liners.
7. **Implement `on_data` skeleton** — desync bail check (BC-2.15.009) + FIR=1 gate (BC-2.15.008) + carry placeholder; no detection logic yet (STORY-108).
8. **Wire `src/analyzer/mod.rs`** — add `pub mod dnp3;`.
9. **Author all four Kani harnesses** under `#[cfg(kani)] mod kani_proofs` per VP-023 harness skeleton.
10. **Unit tests** — AC-001 through AC-009 test functions + EC edge cases.

## Test Plan

| AC | Test Type | Kani? | Notes |
|----|-----------|-------|-------|
| AC-001, AC-002 | Unit | Sub-A | `parse_dnp3_dl_header` Some/None boundary |
| AC-003 | Unit + Kani | Sub-A | LE address decode correctness; Sub-A field-decode assertions |
| AC-004 | Unit + Kani | Sub-C | Validity gate; biconditional asserted symbolically |
| AC-005 | Unit + Kani | Sub-B | `classify_dnp3_fc` totality; Sub-B no-panic assertion |
| AC-006 | Unit + Kani | Sub-B | Set membership; Sub-B per-set `if matches! ... assert!` |
| AC-007 | Unit + Kani | Sub-D | `compute_dnp3_frame_len` formula + bounds; Sub-D biconditional |
| AC-008 | Unit | No | FIR gating + link-FC guard; effectful shell logic; Kani not targeting effectful path. Covers: `test_fir_gating_extract_on_fir1_skip_on_fir0`, `test_on_data_fir_gating_updates_counters`, `test_has_user_data_link_fc_guard` (has_user_data predicate), `test_on_data_fir_but_non_user_data_link_fc_no_extraction` (RESET_LINK 0x00 no-extraction). Minimum-single-block frame offset only — `parse_errors` for &lt;3-byte payloads (BC-2.15.008 EC-006) deferred to STORY-107. |
| AC-009 | Unit | No | Desync bail; single-delivery latch (first segment ≥2 bytes, no sync at offset 0). Lone-0x05 carry-accumulation case (BC-2.15.009 EC-004) deferred to STORY-107. |

Run Kani with `cargo kani --harness verify_parse_dnp3_dl_header_safety`, `--harness verify_classify_dnp3_fc_total`, `--harness verify_is_valid_dnp3_frame_gate`, `--harness verify_compute_dnp3_frame_len`. All four must report `VERIFICATION:- SUCCESSFUL`.

## Previous Story Intelligence

STORY-102 (Modbus MBAP Parse, E-14) is the direct structural precedent:
- Same pure-core + Kani pattern (VP-022 → VP-023 parallel)
- Same `#[cfg(kani)] mod kani_proofs` placement in the analyzer module
- Key DNP3 differences: (1) DEST/SRC are **little-endian** (Modbus is big-endian); (2) an additional Sub-D property for `compute_dnp3_frame_len` arithmetic (no Modbus equivalent); (3) FIR=1 gating is a DNP3-specific transport layer concept; (4) desync bail is initial-delivery-only (fires on the first delivery when `carry.is_empty() && data.len() >= 2 && no sync at offset 0`; there is no cross-segment 16-byte accumulation bail — ADJ-001 Addendum Q1, ADR-007 Decision 2).
- Lesson from STORY-102: write all Kani harnesses in the SAME commit as the pure-core functions, not as a follow-up. The Red Gate check counts harness lines as non-trivial.

## Architecture Compliance Rules

Derived from `architecture/module-decomposition.md` (SS-15) and ADR-007:
1. **Pure-core functions MUST be free `fn`s**, not `impl Dnp3Analyzer` methods — Kani harnesses call them directly without constructing the struct.
2. **Little-endian ONLY** for DEST/SOURCE decode — `u16::from_le_bytes`, never `from_be_bytes` or native-endian. This is a SPEC invariant; big-endian would produce wrong addresses silently.
3. **No `unreachable!()` in `classify_dnp3_fc`** — the wildcard arm is required; an exhaustiveness proof via Kani Sub-B relies on `_ => Unknown` being present.
4. **`compute_dnp3_frame_len` uses integer ceil formula** `(u + 15) / 16` — no floating-point math.
5. **`parse_dnp3_dl_header` does NOT check sync or LENGTH validity** — the three-point gate is `is_valid_dnp3_frame_header`; separation of parse and validate is required for Kani Sub-A to prove over all 2^80 inputs.
6. **Forbidden dependencies**: `src/analyzer/dnp3.rs` MUST NOT depend on `src/analyzer/modbus.rs` or any external DNP3 parsing crate. If either dependency appears, the build MUST fail.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `kani` | via `cargo-kani` (latest stable) | Kani model checker for VP-023; same version used by VP-022 (STORY-102) |
| `std::collections::HashMap` | stdlib | Used in `Dnp3FlowState` (effectful shell only; NOT in pure-core Kani targets) |
| No external DNP3 crate | — | ADR-007 Decision 2: PDU-oriented manual binary parsing; no external crate |

All Rust stdlib `u16::from_le_bytes` and integer arithmetic — no version concerns.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/analyzer/dnp3.rs` | Create | New module: struct, enum, pure-core fns, `Dnp3FlowState` skeleton, Kani harnesses |
| `src/analyzer/mod.rs` | Modify | Add `pub mod dnp3;` |
| `tests/dnp3_parse_core_tests.rs` OR inline `#[cfg(test)] mod tests` in `dnp3.rs` | Create | Unit tests for AC-001..AC-009 |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~3,500 |
| 9 BC files (BC-2.15.001..009) | ~18,000 |
| ADR-007 (relevant sections) | ~4,000 |
| VP-023 (harness skeleton) | ~2,500 |
| STORY-102 (Modbus precedent) | ~3,000 |
| Existing `src/analyzer/modbus.rs` (structural reference) | ~4,000 |
| Tool outputs (cargo check, kani) | ~2,000 |
| **Total estimated** | **~37,000** |

Within 20-30% of agent context window (assumes ~120k window). Story is appropriately sized.

## Dependency Rationale

- `depends_on: [STORY-100]` — STORY-100 delivered the multi-tag `Finding` schema and `MitreTactic` enum that DNP3 findings will use. DNP3 findings reference `T1692.001`, `T0814`, `T0836`, `T1691.001`, `T0827` — all require the catalog established in STORY-100.
- `blocks: [STORY-107]` — STORY-107 (per-flow state + carry buffer) builds directly on the pure-core functions defined here. No flow-state story can be written before the parsers and classifiers exist.
