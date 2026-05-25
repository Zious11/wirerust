---
document_type: story
story_id: "STORY-019"
epic_id: "E-2"
version: "1.3"
status: in_progress
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.010.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.011.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.013.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.029.md
input-hash: "fe76bd9"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-013, STORY-014]
blocks: [STORY-020, STORY-021]
behavioral_contracts: [BC-2.04.010, BC-2.04.011, BC-2.04.013, BC-2.04.029]
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 8
target_module: reassembly
subsystems: [SS-04]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — all ACs must be backed by tests.

> **Execute:** `/vsdd-factory:deliver-story STORY-019`

# STORY-019: Flow Lifecycle — RST Close, FIN Close, Timeout Expiry, and Missing-Key Warning

## Narrative
- **As a** forensic analyst
- **I want** the TCP reassembly engine to correctly close flows on RST (immediately, skipping subsequent payload), on two FINs (after payload of the triggering packet), on idle timeout (`expire_flows`), and to safely handle programming errors where `close_flow` is called for a key that has already been removed
- **So that** flows are always retired via a well-defined reason code delivered to the handler, residual buffered data is always flushed before close, and defensive one-shot warnings prevent stderr flooding from bugs

## Behavioral Contracts

| BC | Title | Role in Story |
|----|-------|---------------|
| BC-2.04.010 | RST Closes Flow Immediately with CloseReason::Rst | RST close path |
| BC-2.04.011 | Both FINs Close Flow with CloseReason::Fin | FIN close path |
| BC-2.04.013 | expire_flows Closes Idle Flows Past flow_timeout_secs | Timeout-based cleanup |
| BC-2.04.029 | close_flow for Missing Key Logs One-Shot Process-Wide Warning | Defensive missing-key guard |

## Acceptance Criteria

### AC-001 (traces to BC-2.04.010 postcondition 1)
- When a TCP RST packet arrives for an established flow, `stats.flows_rst` increments by 1.
- **Test:** `test_BC_2_04_010_rst_increments_flows_rst()`

### AC-002 (traces to BC-2.04.010 postcondition 2-4)
- After a RST, any contiguous data buffered in both directions is flushed to the handler via `on_data` calls, then `handler.on_flow_close(key, CloseReason::Rst)` is called exactly once, and the flow is removed from `self.flows`.
- **Test:** `test_BC_2_04_010_rst_flushes_then_closes()`

### AC-003 (traces to BC-2.04.010 postcondition 6 and invariant 3)
- Payload carried in the RST packet itself is NOT processed (RST triggers `PostHandshake::FlowClosed`, preventing payload insertion).
- **Test:** `test_BC_2_04_010_rst_payload_not_processed()`

### AC-004 (traces to BC-2.04.010 invariant 1)
- RST closes the flow regardless of current state: `New`, `SynSent`, `Established`, `Closing`. The flow is removed from `self.flows` in all cases.
- **Test:** `test_BC_2_04_010_rst_closes_from_any_state()`

### AC-005 (traces to BC-2.04.011 invariant 1)
- The first FIN transitions the flow state to `Closing` and `fin_count` becomes 1. The flow is NOT removed (still open).
- **Test:** `test_BC_2_04_011_first_fin_transitions_to_closing()`

### AC-006 (traces to BC-2.04.011 postconditions 1-6)
- When a second FIN arrives (from either direction), `flow.state == Closed`; `stats.flows_fin` increments by 1; remaining contiguous data is flushed; `handler.on_flow_close(key, CloseReason::Fin)` is called exactly once; the flow is removed from `self.flows`.
- **Test:** `test_BC_2_04_011_second_fin_closes_flow()`

### AC-007 (traces to BC-2.04.011 invariant 2)
- FIN close happens AFTER payload processing for the FIN packet (data carried in a FIN segment is reassembled and delivered before the flow closes).
- **Test:** `test_BC_2_04_011_fin_payload_processed_before_close()`

### AC-008 (traces to BC-2.04.011 edge case EC-002)
- Two FINs from the SAME direction (retransmit) are sufficient to close the flow (fin_count reaches 2 regardless of which direction each FIN came from).
- **Test:** `test_BC_2_04_011_same_direction_fin_retransmit_closes_flow()`

### AC-009 (traces to BC-2.04.013 postcondition 1-2)
- `expire_flows(current_time, handler)` closes all flows where `current_time > last_seen AND (current_time - last_seen) > flow_timeout_secs` with `CloseReason::Timeout`. `stats.flows_expired` increments by the number of flows expired.
- **Test:** `test_BC_2_04_013_expire_flows_closes_idle_flows()`

### AC-010 (traces to BC-2.04.013 postcondition 4)
- Flows that are within the timeout window are NOT closed by `expire_flows`.
- **Test:** `test_BC_2_04_013_expire_flows_does_not_close_active_flows()`

### AC-011 (traces to BC-2.04.013 invariant 1)
- `expire_flows` uses underflow-safe subtraction: `current_time > flow.last_seen` is checked BEFORE `current_time - flow.last_seen > timeout`, preventing u32 underflow.
- **Test:** `test_BC_2_04_013_expire_flows_does_not_underflow_when_time_travels_backwards()`

### AC-012 (traces to BC-2.04.013 invariant 2 and edge case EC-004)
- A flow with `state == FlowState::Closed` is also expired by `expire_flows` regardless of its idle time (handles flows that were FIN-closed but not yet removed).
- **Test:** `test_BC_2_04_013_already_closed_state_is_expired()`

### AC-013 (traces to BC-2.04.029 postcondition 4)
- When `close_flow` is called for a key NOT in `self.flows` and `CLOSE_FLOW_MISSING_WARNED == false`, `eprintln!` fires exactly once, `CLOSE_FLOW_MISSING_WARNED` is set to `true`, and `self.flows` is unmodified.
- **Test:** `test_BC_2_04_029_close_flow_missing_key_warns_once()`

### AC-014 (traces to BC-2.04.029 PC5)

On subsequent calls to `close_flow` for a missing key (after the first warning):
- The atomic-state latching property (`CLOSE_FLOW_MISSING_WARNED` remains `true`) is automated-test-verified via `close_flow_missing_warned_for_testing()`.
- The "no additional `eprintln!` is emitted" sub-property is enforced **structurally** by the swap-guarded `if`-block at `src/reassembly/lifecycle.rs:42-50` and verified by code review (mirrors the BC-2.04.048 PC2 / inv-3 enforcement-mode precedent established in STORY-014 / Wave 7 / ADR-0004 amendment). In-process stderr capture is fragile and out of scope for this story.

- **Test:** `test_BC_2_04_029_close_flow_missing_key_warns_once` (combined with AC-013 + EC-009; rationale: process-global atomic ordering)

### AC-015 (traces to BC-2.04.029 postcondition 1-3)
- When `close_flow` returns early for a missing key: no `on_flow_close` callback fires, `total_memory` is unchanged, and `self.flows` is unmodified.
- **Test:** `test_BC_2_04_029_close_flow_missing_key_does_not_modify_state()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| RST handling in apply_handshake_flags | src/reassembly/mod.rs:273-279 | effectful-shell |
| FIN close detection in process_packet | src/reassembly/mod.rs:165-174 | effectful-shell |
| FIN flag block in apply_handshake_flags | src/reassembly/mod.rs:281-287 | effectful-shell |
| expire_flows | src/reassembly/mod.rs:536-552 | effectful-shell |
| close_flow (missing-key guard) | src/reassembly/lifecycle.rs:42-50 | effectful-shell (stderr write) |
| CLOSE_FLOW_MISSING_WARNED AtomicBool | src/reassembly/lifecycle.rs:31 | effectful-shell (global state) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | RST on New flow (no handshake, no data) | flows_rst++; on_flow_close(Rst); flow removed; no data flushed |
| EC-002 | RST on flow with buffered data | Buffered data flushed first; then on_flow_close(Rst) |
| EC-003 | RST packet carries payload | Payload is NOT inserted; RST wins |
| EC-004 | FIN on New flow | state->Closing; fin_count=1; flow open; second FIN closes |
| EC-005 | FIN+data packet | Data inserted and flushed; then FIN-close detected |
| EC-006 | RST and FIN in same packet | RST block runs first (PostHandshake::FlowClosed returned); FIN block not reached |
| EC-007 | Flow idle for exactly `flow_timeout_secs` seconds | NOT expired (condition is `> timeout`, not `>=`) |
| EC-008 | current_time < last_seen (timestamp reorder) | NOT expired (underflow guard: `current_time > last_seen` check fails) |
| EC-009 | CLOSE_FLOW_MISSING_WARNED already true on test run | Silent return for all subsequent missing-key calls |
| EC-010 | close_flow called for key that IS in flows | Normal close behavior; no warning; CLOSE_FLOW_MISSING_WARNED unchanged |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reassembly/lifecycle.rs (close_flow missing-key path) | effectful-shell | Writes to stderr; reads/writes global AtomicBool |
| src/reassembly/mod.rs (expire_flows, RST/FIN blocks) | effectful-shell | Mutates self.flows, self.stats, invokes callbacks |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,800 |
| BC files (4 BCs) | ~5,000 |
| src/reassembly/mod.rs (RST block ~273-279, FIN close ~165-174, expire_flows ~536-552) | ~1,500 |
| src/reassembly/lifecycle.rs (close_flow missing-key ~31-50) | ~600 |
| src/reassembly/flow.rs (on_fin ~255-262, on_rst ~264-266) | ~400 |
| Test files | ~4,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~15,300** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~7.7%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for all 15 ACs in `tests/reassembly_engine_tests.rs` and `tests/reassembly_flow_tests.rs`
2. [ ] Verify Red Gate: all tests fail before implementation changes
3. [ ] Verify existing implementation satisfies all ACs (brownfield)
4. [ ] Test RST from all flow states (New, SynSent, Established, Closing) — 4 separate test cases
5. [ ] Test FIN timing: payload in FIN segment must be delivered before flow closes
6. [ ] Test expire_flows underflow guard: pass current_time < last_seen; assert not expired
7. [ ] Test CLOSE_FLOW_MISSING_WARNED atomic carefully — it is process-wide; use separate test ordering or reset strategy
8. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-013 | State machine tests in reassembly_flow_tests.rs | on_rst() has no state guard — closes unconditionally | SYN+ACK sets initiator to DST (not SRC) |
| STORY-014 | ISN_MISSING_WARNED is process-wide; ordering matters | ISN-related atomics use Ordering::Relaxed | CLOSE_FLOW_MISSING_WARNED follows the same one-shot warning pattern |
| STORY-011 | FlowKey commutativity; engine-level tests in reassembly_engine_tests.rs | Test handler records callbacks for assertion | |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| RST triggers `PostHandshake::FlowClosed`; payload processing skipped | BC-2.04.010 invariant 3 | Code review: RST block returns FlowClosed before payload branch |
| FIN close fires AFTER payload processing (fin-close detection is post-flush) | BC-2.04.011 invariant 2 | Test: insert data in FIN packet; assert data delivered before on_flow_close |
| expire_flows underflow guard: `current_time > last_seen` checked BEFORE subtraction | BC-2.04.013 invariant 1 | Code review: guard ordering in mod.rs:536-552 (fn decl :536, closing :552) |
| `CLOSE_FLOW_MISSING_WARNED` uses `swap(true, Ordering::Relaxed)` one-shot pattern | BC-2.04.029 invariant 1 | Code review: grep for swap in lifecycle.rs |
| debug_assert fires in debug builds when close_flow called for missing key | BC-2.04.029 postcondition 6 | Compile with debug assertions; run tests |
| **No additional eprintln on subsequent missing-key calls** | BC-2.04.029 PC5 | Code review of swap-guarded if-block at `src/reassembly/lifecycle.rs:42-50` (matches BC-2.04.048 PC2 / inv-3 / ADR-0004 amendment precedent) |
| **`#[doc(hidden)]` on test-only accessors** | BC-2.04.029 + brownfield-formalization API hygiene | Code review: `close_flow_missing_warned_for_testing()`, `reset_close_flow_missing_warned_for_testing()`, `trigger_close_flow_missing_key_for_testing()`, and `force_set_flow_state_for_testing()` (added in W8.3 + F-1 pre-empt) all carry `#[doc(hidden)]` |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stable toolchain | MSRV 1.85+ | AtomicBool with Ordering::Relaxed, FlowState enum |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/reassembly/lifecycle.rs` | verify + append | Verified close_flow missing-key guard at :42-50. Appended #[doc(hidden)] pub fn test seams at end-of-file: close_flow_missing_warned_for_testing(), reset_close_flow_missing_warned_for_testing(), trigger_close_flow_missing_key_for_testing(), force_set_flow_state_for_testing() — opt-in-per-guard per ADR-0004 amendment |
| `src/reassembly/mod.rs` | verify + visibility widen | Widened `mod lifecycle` → `pub mod lifecycle` to expose test-seam accessors; verified RST/FIN/expire_flows blocks at lines 165-174, 273-279, 281-287, 536-552 |
| `src/reassembly/flow.rs` | verify (lines 255-262, 264-266) | on_fin and on_rst implementations |
| `tests/reassembly_engine_tests.rs` | modify | Add AC-001 through AC-015 (engine-level lifecycle tests) |
| `tests/reassembly_flow_tests.rs` | modify | Add flow-level state transition tests for on_fin/on_rst |

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.3 | 2026-05-25 | story-writer | Wave 8 STORY-019 adv-pass-1 F-4 closure: explicit enforcement-mode notation on AC-014 ("no additional eprintln" sub-property moved from automated-test to code-review enforcement per BC-2.04.029 PC5; mirrors BC-2.04.048 PC2 / inv-3 / Wave-7 ADR-0004 amendment precedent). Added Architecture Compliance Rules rows for swap-guard code review + #[doc(hidden)] test-accessor hygiene (covers 4 test seams including the F-1 pre-empt force_set_flow_state_for_testing). Updated File Structure Requirements to acknowledge the appended lifecycle.rs test seams and the mod.rs visibility widening. |
| v1.0 | 2026-05-21 | story-writer | Initial decomposition |
| v1.1 | 2026-05-21 | story-writer | Wave 7 partial anchor refresh |
| v1.2 | 2026-05-25 | story-writer | Wave 8 pre-flight — refreshed body line-anchors to match post-Wave-7 source state (BC-2.04.010 v1.4 mod.rs:273-279 RST block; BC-2.04.011 v1.4 mod.rs:165-174 FIN-close; flow.rs:255-262 on_fin; flow.rs:264-266 on_rst; and verified expire_flows mod.rs:536-552, FIN flag block mod.rs:281-287, close_flow guard lifecycle.rs:42-50, CLOSE_FLOW_MISSING_WARNED lifecycle.rs:31 against current source). Frontmatter status → in_progress. |
