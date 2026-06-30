---
document_type: story
story_id: STORY-145
title: "ServerHello Carry Symmetry + Per-Flow / Per-Direction Isolation (BC-2.07.041 + amended BC-2.07.002)"
epic_id: E-5
wave: 66
points: 5
phase: f3
tdd_mode: strict
status: merged
feature_id: fix-tls-clienthello-frag
github_issue: fix-tls-clienthello-frag
subsystems: [SS-07]
target_module: analyzer/tls
depends_on: [STORY-144]
blocks: []
behavioral_contracts:
  - BC-2.07.041
  - BC-2.07.002
verification_properties:
  - VP-039
assumption_validations: []
risk_mitigations: []
# BC status: all BCs authored and anchored; status remains draft pending PO wave assignment
inputs:
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.041.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.002.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.038.md
  - .factory/specs/architecture/decisions/ADR-011-tls-handshake-reassembly.md
  - .factory/cycles/fix-tls-clienthello-frag/delta-analysis.md
input-hash: "88e29c9"
---

# STORY-145: ServerHello Carry Symmetry + Per-Flow / Per-Direction Isolation

## Narrative

**As a** security analyst relying on wirerust JA3S fingerprinting of fragmented TLS ServerHellos,
**I want** the TLS analyzer to apply the same carry-buffer reassembly logic to the `server_hs_carry`
direction as STORY-144 applies to `client_hs_carry`, and to formally verify that no flow's carry
buffer state ever bleeds into another flow's carry buffers or the opposite direction,
**so that** fragmented ServerHellos are correctly reassembled (RFC 5246 §6.2.1 symmetric case)
and JA3S is computed even when the ServerHello spans multiple TLS 0x16 records, with no risk
of cross-flow or cross-direction contamination.

## Behavioral Contracts

| BC ID | Version | Title | Story Role |
|-------|---------|-------|-----------|
| BC-2.07.041 | v1.2 | Handshake Carry Buffers Are Per-Flow and Per-Direction Isolated | Primary: direction parameter drives carry selection; cross-flow keying via HashMap; Sub-E direction isolation proptest; `test_BC_2_07_041_cross_flow_isolation` unit test |
| BC-2.07.002 | v1.6 | Parse Complete TLS ServerHello: JA3S Fingerprint Computed | Scope expansion: "complete" now includes multi-record reassembled path via `server_hs_carry`; single-record fast path unchanged |

## Acceptance Criteria

### AC-145-001: `try_parse_records` applies symmetric carry logic to `server_hs_carry` for `ServerHello` (0x02) dispatch
**Traces to:** BC-2.07.038 v2.7 Postcondition 3b; BC-2.07.002 v1.6 Precondition 2; ADR-011 Decision 2

The `Direction::ServerToClient` arm of `try_parse_records` mirrors the `Direction::ClientToServer` arm established in STORY-144. Specifically, for `ServerToClient` 0x16 records:
1. Payload bytes append to `state.server_hs_carry` (not `client_hs_carry`)
2. The carry drain loop decodes `msg_type` from `server_hs_carry[0]`: if `0x02` (ServerHello), call `handle_server_hello` with the assembled body (same API as single-record path)
3. All overflow, exact-consume, and truncation semantics from BC-2.07.038/039/040 apply identically (STORY-144 implementation reused — same drain loop, different carry reference)
4. `server_hello_seen` is set by `handle_server_hello` regardless of whether the hello was single-record or reassembled

The `done()` short-circuit (`client_hello_seen && server_hello_seen`) fires correctly after both reassembled hellos (BC-2.07.038 Invariant 6).

If STORY-144 implemented the direction-parameterized drain loop (carry reference selected by `match direction { ClientToServer => &mut state.client_hs_carry, ServerToClient => &mut state.server_hs_carry }`), then STORY-145 implementation is limited to: (a) adding the `ServerToClient` match arm to the carry-reference selection, and (b) dispatching `handle_server_hello` for `msg_type == 0x02` inside the shared drain loop body. No loop body duplication.

(traces to BC-2.07.002 v1.6 Precondition 2; BC-2.07.038 v2.7 Postcondition 3b)

**Red-Gate test (VP-039 Sub-E):** `proptest_vp039_direction_isolation` — interleaved C2S + S2C fragmented hello deliveries; `client_hello_seen==true`, `server_hello_seen==true`, `parse_errors==0`, carry_len_client==0, carry_len_server==0 after all records

### AC-145-002: Direction parameter drives carry selection; no cross-direction bleed
**Traces to:** BC-2.07.041 v1.2 Invariant 2; Postconditions 1–2; ADR-011 Decision 2

The carry buffer selected for any `on_data` invocation is determined solely by the `direction` parameter:
```
match direction {
    Direction::ClientToServer => &mut state.client_hs_carry,
    Direction::ServerToClient => &mut state.server_hs_carry,
}
```
No code path exists that appends `ClientToServer` bytes to `server_hs_carry` or vice versa. The direction parameter is the SOLE selector; no secondary conditionals override it.

(traces to BC-2.07.041 v1.2 Invariant 2; Postcondition 2)

**Red-Gate test (VP-039 Sub-E):** The `proptest_vp039_direction_isolation` harness verifies that running interleaved C2S/S2C fragments through `on_data` produces the same `client_hello_seen`, `server_hello_seen`, `parse_errors` as two independent same-direction runs — i.e., no cross-direction contamination.

### AC-145-003: Cross-flow isolation — Flow A's carry never affects Flow B's carry
**Traces to:** BC-2.07.041 v1.2 Invariant 1; Postconditions 1, 4–5; ADR-011 §Isolation

The `HashMap<FlowKey, TlsFlowState>` structure guarantees that each `on_data` invocation accesses ONLY the `TlsFlowState` keyed by its `flow_key` argument. Carry buffer append and drain operations are scoped to that single `TlsFlowState` instance. No carry buffer access path dereferences a `FlowKey` other than the one passed to the current `on_data` call.

After `on_flow_close(flow_key_A)`, `flow_B`'s carry buffers are unaffected (BC-2.07.041 Invariant 4).

(traces to BC-2.07.041 v1.2 Invariants 1, 4; Postconditions 1, 4–5)

**Red-Gate test (VP-039 Sub-E-ext):** `test_BC_2_07_041_cross_flow_isolation` — Two distinct FlowKeys: Flow A complete single-record ClientHello (SNI=a.example); Flow B same-shaped ClientHello fragmented across records (SNI=b.example); assert `sni_counts["a.example"]==1`, `sni_counts["b.example"]==1`, no cross-flow bleed, exactly 2 sni_counts entries

### AC-145-004: Fragmented ServerHello assembled and dispatched; `server_hello_seen` set; `done()` fires after both
**Traces to:** BC-2.07.002 v1.6 Postcondition 7 (drain operations), Invariant 4; BC-2.07.038 v2.7 Invariant 6

A ServerHello fragmented across two 0x16 server-direction records produces `server_hello_seen==true`, `ja3s_counts.len()==1`, `parse_errors==0` — identical to a single-record ServerHello.

After both `client_hello_seen==true` (from STORY-144) and `server_hello_seen==true` (from this story), `done()` returns `true` and subsequent records on the same flow are silently ignored (BC-2.07.038 Invariant 4).

(traces to BC-2.07.002 v1.6 Precondition 2; Invariant 4 "single-record fast path preserved")

**Red-Gate test included in `proptest_vp039_direction_isolation`:** the proptest delivers both fragmented hellos interleaved; both `client_hello_seen==true` and `server_hello_seen==true` are asserted.

### AC-145-005: Existing single-record ServerHello tests unaffected (regression)
**Traces to:** BC-2.07.002 v1.6 Invariant 4; BC-2.07.038 v2.7 EC-007

All existing `tls_analyzer_tests.rs` ServerHello tests (STORY-053 scope: JA3S fingerprinting, cipher tracking, weak-cipher findings) must pass without modification. The single-record ServerHello fast path is preserved: append to `server_hs_carry` (empty) → drain loop finds complete message → dispatch → carry empty.

(traces to BC-2.07.002 v1.6 Invariant 4)

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `try_parse_records` `ServerToClient` 0x16 carry path | `src/analyzer/tls.rs` | Effectful |
| Direction match arm carry selection | `src/analyzer/tls.rs` | Effectful |
| `proptest_vp039_direction_isolation` | `tests/tls_analyzer_tests.rs` | Pure |
| `test_BC_2_07_041_cross_flow_isolation` | `tests/tls_analyzer_tests.rs` | Pure |

Architecture compliance: SS-07 only. `server_hs_carry` was introduced as a struct field in STORY-144; this story wires the `ServerToClient` drain loop path to use it. No new struct fields. No new files.

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-B1 | BC-2.07.041 EC-001 | Two concurrent flows with fragmented ClientHellos | Each flow accumulates independently; both `client_hello_seen==true`; sni_counts has entries from both; no cross-contamination |
| EC-B2 | BC-2.07.041 EC-002 | Flow A client carry overflows; Flow B client carry still active | Flow B continues normally; `flow_B.client_hs_carry` unaffected; overflow_count incremented once |
| EC-B3 | BC-2.07.041 EC-003 | Interleaved: Flow A C2S, Flow B C2S, Flow A C2S (completing A's hello) | Flow A's `client_hello_seen==true`; Flow B's unchanged; no direction or flow confusion |
| EC-B4 | BC-2.07.041 EC-005 | Only server direction data arrives for a flow (no client data) | `client_hs_carry` empty; `server_hs_carry` accumulates ServerHello bytes; no cross-direction bleed |
| EC-B5 | BC-2.07.002 EC-005 | Fragmented ServerHello (first record partial) | `server_hs_carry` accumulates; `handle_server_hello` called when complete; JA3S computed; `server_hello_seen==true` |

## Estimated Complexity

**Story points: 5** (the server-direction carry path is structurally symmetric with the client-direction path delivered in STORY-144; the primary new work is VP-039 Sub-E: 4 test functions — 2 AC-cited Red-Gate harnesses covering interleaved direction isolation and cross-flow isolation, plus 2 DoS-guard sibling-coverage tests per DF-SIBLING-SWEEP-001)

## Token Budget Estimate

| Context source | Estimated tokens |
|---------------|-----------------|
| This story spec | ~2,000 |
| BC files (2 BCs: 041/002) + BC-2.07.038 (referenced) | ~4,500 |
| ADR-011 | ~3,000 |
| STORY-144 implementation in `src/analyzer/tls.rs` | ~24,000 |
| `tests/tls_analyzer_tests.rs` (pre-existing + STORY-144 additions) | ~25,000 |
| VP-039 Sub-E harness skeletons | ~2,000 |
| Tool outputs | ~1,500 |
| **Total estimate** | **~62,000** |

Fits within a 200k context window (~31%). Implementer should read STORY-144's completed code before starting Sub-E harnesses.

## Tasks

1. **Write Red-Gate tests first (TDD Step 1)**
   - `proptest_vp039_direction_isolation` — interleaved C2S + S2C fragments; assert both hellos seen, no cross-contamination
   - `test_BC_2_07_041_cross_flow_isolation` — two FlowKeys; Flow A complete, Flow B fragmented; sni_counts has both
   - Confirm these FAIL before any `ServerToClient` carry wiring is added

2. **Wire `ServerToClient` carry drain path (AC-145-001, AC-145-002)**
   - In `try_parse_records`, identify the `Direction::ServerToClient` 0x16 arm
   - Replace single-record dispatch with the same overflow-check + append + drain loop as `ClientToServer`, using `state.server_hs_carry` and dispatching `handle_server_hello` for `msg_type == 0x02`
   - Confirm: `proptest_vp039_direction_isolation` turns GREEN

3. **Verify cross-flow isolation (AC-145-003)**
   - The isolation is structural (HashMap keying); confirm `test_BC_2_07_041_cross_flow_isolation` turns GREEN without additional code changes

4. **Verify `done()` fires after both reassembled hellos (AC-145-004)**
   - Deliver fragmented ClientHello (C2S) + fragmented ServerHello (S2C) for same flow; assert `done()==true` after both; assert subsequent records silently skipped

5. **Full regression sweep (AC-145-005)**
   - `cargo test --all-targets` — all tests GREEN (STORY-144 + STORY-145 combined)
   - `cargo clippy --all-targets -- -D warnings` — zero warnings

6. **Micro-commit and PR** targeting `develop` (wave 66)

## Previous Story Intelligence

**From STORY-144:** The carry struct fields (`client_hs_carry`, `server_hs_carry`) and the aggregate counter (`handshake_reassembly_overflows`) are already in place. The `ClientToServer` drain loop is implemented. This story only needs to wire the `ServerToClient` arm using the same loop logic and the pre-existing `server_hs_carry` field. No struct changes needed.

**From STORY-139/140 (ENIP/DNP3 carry-split sibling stories):** The isolation tests (`proptest_vp039_direction_isolation` in VP-039 corresponds to `proptest_vp035_direction_isolation` pattern in VP-035 for DNP3) follow the same structure: generate two fragmented byte sequences, deliver interleaved, assert equivalence to independent runs.

## Architecture Compliance Rules

Source: `architecture/module-decomposition.md` + ADR-011

1. **Symmetric loop reuse:** The server-direction drain loop MUST be byte-for-byte equivalent to the client-direction loop delivered in STORY-144, differing only in the carry reference (`server_hs_carry` vs `client_hs_carry`) and dispatch target (`handle_server_hello` vs `handle_client_hello`). If STORY-144 implemented the direction-parameterized form (a single loop body with carry reference selected by a `match direction` arm), STORY-145 only needs to wire the `ServerToClient` arm.
2. **No global carry state:** `TlsAnalyzer` holds only the aggregate `handshake_reassembly_overflows` counter; per-direction carry state lives exclusively in `TlsFlowState`.
3. **HashMap keying enforces isolation:** No additional synchronization primitives are needed. The existing `HashMap<FlowKey, TlsFlowState>` keying is sufficient (BC-2.07.041 Postcondition 5).
4. **Test namespace isolation (DF-TEST-NAMESPACE-001):** ALL 4 new test functions added by STORY-145 MUST be placed inside a dedicated `mod story_145 { ... }` wrapper in `tests/tls_analyzer_tests.rs`. No new test function from this story may be added at the flat module root.

## Library & Framework Requirements

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `tls-parser` | 0.12.2 (pinned) | `parse_tls_message_handshake` for assembled ServerHello bytes (symmetric with ClientHello) |
| `proptest` | 1.x (existing) | `proptest_vp039_direction_isolation` |

No new dependencies. Same versions as STORY-144.

**Forbidden dependencies:** same as STORY-144 — no `src/reassembly/` dependency for carry logic.

## File Structure Requirements

| File | Change Type | Purpose |
|------|------------|---------|
| `src/analyzer/tls.rs` | Modify | Wire `ServerToClient` 0x16 carry drain path |
| `tests/tls_analyzer_tests.rs` | Modify | Add VP-039 Sub-E harnesses (4 new: 1 proptest + 3 unit), all inside `mod story_145 { ... }` per DF-TEST-NAMESPACE-001 |

No new files. STORY-145 is a narrow additive change on top of STORY-144.

### Test Helper Note (STORY-145)

STORY-145's Sub-E harnesses (`proptest_vp039_direction_isolation` and `test_BC_2_07_041_cross_flow_isolation`) require the same helpers that STORY-144 introduced. The design decision for helper sharing is:

**Decision: re-declare minimal helpers per mod (do NOT import across mods).**

Each `mod story_NNN { ... }` block is self-contained. `tests/tls_analyzer_tests.rs` uses a flat module namespace and Rust does not allow cross-`mod` imports within the same file without explicit `use super::story_144::*` or equivalent. Re-declaring the minimal helpers in `mod story_145` is simpler, avoids pub visibility escalation, and keeps each mod independently auditable.

| Helper needed by Sub-E | Availability in `mod story_145` | Action |
|------------------------|--------------------------------|--------|
| `build_server_hello() -> Vec<u8>` | Not yet in scope for `mod story_145` | The real `build_server_hello(0x002f)` returns a COMPLETE TLS record (5-byte record header + handshake body). Re-declare a LOCAL wrapper in `mod story_145` that STRIPS the 5-byte record header: `build_server_hello(0x002f)[5..].to_vec()`. Returns RAW handshake-message bytes with NO record header — so fragmentation tests can re-frame them via `wrap_as_tls_record` per fragment. Before creating, `grep` for the real name in the suite — use it directly if it is at crate scope. |
| `make_test_flow_key(seed: u8) -> FlowKey` | Defined in `mod story_144` (private to that mod) | Re-declare an identical LOCAL copy in `mod story_145`. The function body is trivial (2–3 lines: construct a `FlowKey` varying `src_port` or address octets by `seed`); duplication cost is negligible. |
| `build_client_hello_with_sni(sni: &str) -> Vec<u8>` | Defined in `mod story_144` (private) | The real `build_client_hello(sni, &[0x002f])` returns a COMPLETE TLS record (5-byte record header + handshake body). Re-declare a LOCAL wrapper in `mod story_145` that STRIPS the 5-byte record header: `build_client_hello(sni, &[0x002f])[5..].to_vec()`. Returns RAW handshake-message bytes with NO record header. |
| `wrap_as_tls_record(content_type: u8, payload: &[u8]) -> Vec<u8>` | Possibly at crate scope (check `grep`) | Use the real existing name if at crate scope; otherwise re-declare LOCAL copy. |
| `all_findings_len_for_testing` | EXISTING tls.rs seam | Use as-is — do NOT rename. |
| `client_hello_seen_for_testing`, `server_hello_seen_for_testing` | tls.rs seams from STORY-144 / EXISTING | Use directly — `client_hello_seen_for_testing` is NEW (created in STORY-144 AC-144-001); `server_hello_seen_for_testing` is the EXISTING seam at tls.rs:991. |
| `client_hs_carry_len_for_testing`, `server_hs_carry_len_for_testing` | NEW tls.rs seams from STORY-144 | Use directly. |

**Implementer action:** before writing any helper in `mod story_145`, run `grep -n 'fn build_server_hello\|fn make_test_flow_key\|fn build_client_hello\|fn wrap_as_tls_record\|fn test_flow_key' tests/tls_analyzer_tests.rs` to discover crate-scope definitions. Use the real name for anything found at crate scope; re-declare locally for anything that is only in `mod story_144` (private to that mod).

## Red-Gate Test Set (VP-039 Sub-E scope)

### AC-cited Red-Gate tests (TDD Step 1 — must fail before implementation)

| Test name | Sub | BC | Type | Fails before? |
|-----------|-----|----|------|--------------|
| `proptest_vp039_direction_isolation` | Sub-E | BC-2.07.041 Invariant 2 | proptest | Yes |
| `test_BC_2_07_041_cross_flow_isolation` | Sub-E-ext | BC-2.07.041 Invariants 1/4 | unit | Yes (until STORY-145) |

These 2 AC-cited harnesses complete VP-039's full 17-harness set when combined with STORY-144's 15 (Sub-A/B/C/D/F) + STORY-146's 0 (VP-040 harnesses are in STORY-146). `proptest_vp039_carry_bounded_invariant` (Sub-F) is assigned to STORY-144 ONLY — it is NOT in STORY-145 scope.

### Additional sibling-sweep coverage (DF-SIBLING-SWEEP-001)

The implementation also delivered 2 symmetric DoS-guard coverage tests required by DF-SIBLING-SWEEP-001 (ServerToClient direction must have the same overflow/body-len-spoof guards as the ClientToServer direction established in STORY-144). These are coverage tests — they are NOT cited by any AC and do NOT affect AC-to-BC traceability.

| Test name | Guard exercised | Mirrors STORY-144 test |
|-----------|----------------|----------------------|
| `test_vp039_server_carry_overflow_clear_and_recover` | `ServerToClient` Step-1 pre-append overflow guard (clear-and-recover, `MAX_BUF=65536`) | `test_vp039_carry_overflow_clear_and_recover` |
| `test_vp039_server_body_len_spoof` | `ServerToClient` Decision-4 `body_len`-spoof guard | `test_vp039_body_len_spoof` |

Total `mod story_145` test count: **4** (1 proptest + 3 unit: 2 AC-cited Red-Gate + 2 DoS-guard sibling-coverage).

## Holdout Scenarios (F4)

STORY-145 specifically gates on:
- **HS-F4-007**: interleaved fragmented ClientHello + ServerHello for same flow → both seen, done() fires, JA3 + JA3S populated, parse_errors==0
- **HS-F4-008**: two concurrent flows (Flow A complete single-record, Flow B fragmented two-record) → sni_counts has both hostnames, no cross-flow bleed
- **HS-F4-009**: fragmented ServerHello regression (single-record ServerHello unchanged) → server_hello_seen==true, JA3S computed, parse_errors==0

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| v1.1 | 2026-06-30 | story-writer | OBS-1 / DF-SIBLING-SWEEP-001 documentation update: added 2 DoS-guard sibling-coverage tests (`test_vp039_server_carry_overflow_clear_and_recover`, `test_vp039_server_body_len_spoof`) to the Red-Gate Test Set section under a dedicated "Additional sibling-sweep coverage" sub-table; updated test-count prose from "2 new test functions" to "4 test functions (2 AC-cited Red-Gate + 2 DoS-guard sibling-coverage)" in Architecture Compliance Rules, File Structure Requirements, and Estimated Complexity. No ACs were re-mapped; no inputs or BC references changed. |
