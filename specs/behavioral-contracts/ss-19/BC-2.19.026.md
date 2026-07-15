---
document_type: behavioral-contract
level: L3
version: "1.6"
status: draft
producer: product-owner
timestamp: 2026-07-15T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-19
capability: CAP-19
lifecycle_status: active
introduced: feature-iec104
modified:
  - "v1.1: F-P2-H1 — VP-044 over-scope corrected: loop no-panic and termination route to VP-047; VP-044 retained only for the parse_apci_header pure-core sub-call per ADR-013 Decision 8. F-P2-L2 — bad-start-byte recovery reconciled to ADR: 'skip 2 bytes' replaced with 'clear carry buffer + advance 1 byte' per BC-2.19.002 postcondition-3 and ADR-013 Decision 3. 2026-07-13"
  - "v1.2: F-P3-M3 — added VP-045 forward-anchor for directional carry isolation, reciprocal with VP-INDEX source_bc. 2026-07-14"
  - "v1.3: F-P4-M2 — VP-045 harness names synced to registry: proptest_vp045_direction_isolation + proptest_vp045_independent_run_equivalence. 2026-07-14"
  - "v1.4: F-P5-H1 / F-P5-L2 — PC-4 and Inv-1 reconciled to ADR-013 Decision 3: bad-start-byte carry is NOT cleared; full advance-mode enumeration added (bad-start-byte→+1/no-clear; malformed-LEN→+2-stub; valid→LEN+2; insufficient→stash). Removes stale BC-2.19.002 postcondition-3 cite (which itself was corrected in v1.2). 2026-07-14"
  - "v1.5: F7-L2 — Invariant 4 math corrected: ceil(255/6)=43 → floor(255/6)=42 (max whole 6-byte frames in 255 bytes); unsound 'input ≤ 255' assumption dropped — termination holds for any finite input (≥1 byte/iteration guarantees it regardless of input size). 2026-07-14"
  - "v1.6: SR-172-03 EMIT-WITH-DEDUP (pre-STORY-172 fidelity check, 2026-07-15) — malformed-LEN branch (valid 0x68 start byte, LEN outside [4,253]) revised: FIRST occurrence per flow direction emits ONE T0814 (Denial of Service, ThreatCategory::Anomaly / Verdict::Possible / Confidence::Medium) then advances 2-byte APCI stub; per-direction dedup flags (malformed_len_reported_c2s / malformed_len_reported_s2c) in Iec104FlowState prevent re-emission; subsequent malformed-LEN in same direction: advance-only (silent resync). Bad-start-byte branch unchanged. Invariant 5 added (dedup guarantee). EC-006..EC-008 added. MITRE Techniques updated. Architecture Anchors updated. Validation: .factory/cycles/feature-iec104/research/sr-172-03-malformed-len-validation.md (CVE-2023-5768, Snort3 IEC104_BAD_LENGTH, Wireshark iec104.apdu_invalid_len, Zeek weird+sampling)."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "a153144"
---

# BC-2.19.026: Frame-Walk Loop Processes Multiple APDUs per on_data Call

## Description

`Iec104Analyzer::on_data(data: &[u8], direction: Direction, state: &mut Iec104FlowState)`
implements a frame-walk loop that processes all complete APCI frames in `data` in a
single call. It prepends any carry bytes from the previous call, then iterates: parse
APCI header → if complete frame available, process it → advance cursor → repeat until
insufficient bytes remain. Remaining bytes are stashed into the directional carry buffer.
This loop is the main processing engine and the primary target of VP-047 (no panic under
fuzz; top-level harness calls `on_data` directly). The `parse_apci_header` pure-core call
made within the loop is additionally verified by VP-044 per ADR-013 Decision 8 scope
(`on_data` is the effectful shell; `parse_apci_header` is the Kani target). Multiple
complete frames in one on_data call are processed sequentially without interleaving.

## Preconditions

1. `data` is a `&[u8]` slice of TCP segment data (may contain 0 or more complete APCI frames, partial frames, or junk bytes).
2. `direction` is `Direction::ClientToServer` or `Direction::ServerToClient`.
3. `state` contains carry bytes from prior on_data calls for this direction.

## Postconditions

1. All complete APCI frames in `carry + data` are parsed and dispatched.
2. For each parsed frame: findings emitted per BC-2.19.010..024.
3. Remaining incomplete bytes stashed into `state.carry_{c2s,s2c}` bounded at 255 (per BC-2.19.025).
4. Cursor always advances per loop iteration (loop termination guaranteed): bad start byte → advance 1 byte (resync scan to next 0x68 candidate, carry NOT cleared per ADR-013 Decision 3 step 3); malformed-LEN (valid 0x68 start byte, LEN outside [4,253]) → advance 2-byte APCI stub (per ADR-013 Decision 3 step 4); on the FIRST malformed-LEN occurrence in a given flow direction, also emit ONE T0814 (Denial of Service) finding with ThreatCategory::Anomaly / Verdict::Possible / Confidence::Medium and set the per-direction dedup flag (malformed_len_reported_c2s or malformed_len_reported_s2c) in Iec104FlowState; subsequent malformed-LEN frames in the same direction advance the cursor only — no additional finding (silent resync); valid frame → advance LEN+2; insufficient data → stash in carry and return.
5. Function does not panic for any input (VP-047 top-level target; the `parse_apci_header` call within the loop is verified by VP-044 per ADR-013 Decision 8).

## Invariants

1. **Loop termination**: the cursor advances by at least 1 byte per iteration (1-byte advance on bad start byte, carry NOT cleared; 2-byte APCI stub on malformed-LEN; `LEN + 2` on valid frame), guaranteeing termination for any finite input.
2. **No cross-direction interleaving**: C2S and S2C carry buffers are processed independently via the direction parameter.
3. **Pure-core for Kani**: `parse_apci_header` is called as a pure function; `on_data` is the effectful shell that calls it (ADR-013 Decision 8). VP-044 Kani scope is `parse_apci_header` only — not the `on_data` loop.
4. **Maximum frames per call**: bounded by floor(255/6) = 42 complete frames per carry segment (minimum APCI frame = 6 bytes: 1 start + 1 LEN + 4 CF); termination holds for any finite input — ≥ 1 byte advance per iteration guarantees it regardless of input size.
5. **Per-direction malformed-LEN dedup**: `Iec104FlowState` carries one boolean flag per direction (`malformed_len_reported_c2s`, `malformed_len_reported_s2c`); at most one T0814 finding is emitted per direction per flow lifetime for out-of-range LEN. Once the flag is set for a direction all subsequent malformed-LEN frames in that direction advance the cursor only (silent resync); the flag is never reset within a flow.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `data` contains exactly one complete APCI frame | Processed, cursor at len(data) |
| EC-002 | `data` contains 3 complete APCI frames | All three processed sequentially |
| EC-003 | `data` is all partial (no complete frame) | All bytes stashed to carry |
| EC-004 | `data` is empty | No processing; carry unchanged |
| EC-005 | `data` = valid frame + junk bytes | Frame processed; junk stashed to carry (will fail is_valid check on next call) |
| EC-006 | First malformed-LEN frame (valid 0x68, LEN=3) in C2S direction | 2-byte APCI stub advance; ONE T0814 (Anomaly/Possible/Medium) emitted; `malformed_len_reported_c2s` flag set in Iec104FlowState |
| EC-007 | Second malformed-LEN frame in same C2S direction (flag already set) | 2-byte APCI stub advance; NO finding (silent resync) |
| EC-008 | First malformed-LEN frame in S2C direction after C2S flag is already set | 2-byte APCI stub advance; ONE T0814 emitted for S2C independently; `malformed_len_reported_s2c` flag set; C2S flag unchanged |

## Canonical Test Vectors

| Input data | Expected behavior |
|------------|------------------|
| `[0x68, 0x04, 0x07, 0x00, 0x00, 0x00]` (1 STARTDT frame) | 1 frame processed; STARTDT handled (BC-2.19.010) |
| `[0x68, 0x04, ...]` × 2 back-to-back | 2 frames processed |
| `[0x68, 0x04, 0x07, 0x00]` (partial) | stashed to carry |
| `[]` | no-op |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-044 | `parse_apci_header` pure-core sub-call within the loop: no panic, correctness on all symbolic inputs (ADR-013 Decision 8 scope — does NOT cover `on_data` loop itself) | Kani: `verify_parse_apci_header_safety` |
| VP-045 | Directional carry isolation: `carry_c2s` and `carry_s2c` remain independent across multi-frame `on_data` calls; the frame-walk loop drives the carry lifecycle that VP-045 verifies | proptest: `proptest_vp045_direction_isolation`, `proptest_vp045_independent_run_equivalence` |
| VP-047 | `on_data` does not panic for any byte sequence; loop termination | cargo-fuzz: `fuzz_iec104_parser` (top-level harness calls `on_data`) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — the frame-walk loop is the central processing engine of the IEC-104 passive analyzer |
| L2 Domain Invariants | INV-1 (Protocol State Accuracy), INV-2 (Content-First Dispatch Precedence), INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decisions 3, 8 |
| Feature | feature-iec104 |
| MITRE Techniques | T0814 (Denial of Service — malformed-LEN first occurrence per direction; ThreatCategory::Anomaly / Verdict::Possible / Confidence::Medium per SR-172-03 EMIT-WITH-DEDUP); additional technique findings in BC-2.19.010..024 |

## Related BCs

- BC-2.19.001..005 — depends on (APCI header parse guards called inside loop)
- BC-2.19.025 — depends on (carry buffer management inside loop)
- BC-2.19.027 — depends on (flow close: loop stops; carry discarded)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `fn on_data(&mut self, data: &[u8], dir: Direction, state: &mut Iec104FlowState)` — frame-walk loop
- `src/analyzer/iec104.rs` — `Iec104FlowState.malformed_len_reported_c2s: bool`, `Iec104FlowState.malformed_len_reported_s2c: bool` — per-direction dedup flags; T0814 emitted on first malformed-LEN per direction only; flag is set on emission and never reset within a flow (SR-172-03 EMIT-WITH-DEDUP)
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3` — pseudocode (bad-start-byte path: carry NOT cleared, cursor +1 resync scan; malformed-LEN: 2-byte APCI stub advance)
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 8` — VP-044 Kani skeleton (pure-core `parse_apci_header` target; `on_data` is the effectful shell)
- `.factory/cycles/feature-iec104/research/sr-172-03-malformed-len-validation.md` — validation evidence for EMIT-WITH-DEDUP decision: CVE-2023-5768 wrong-APDU-length DoS; Snort3 IEC104_BAD_LENGTH; Wireshark iec104.apdu_invalid_len; Zeek weird+sampling — all established monitors emit on malformed LEN and all dedup

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-044 — `verify_parse_apci_header_safety` (`parse_apci_header` pure-core sub-call only per ADR-013 Decision 8 scope; `on_data` loop no-panic belongs to VP-047)
- VP-045 — `proptest_vp045_direction_isolation`, `proptest_vp045_independent_run_equivalence` (directional carry isolation; frame-walk loop drives carry lifecycle — BC-2.19.025 is the primary target; this BC is in VP-045 source_bc per VP-INDEX)
- VP-047 — `fuzz_iec104_parser` (top-level `on_data` no-panic harness; loop termination)
