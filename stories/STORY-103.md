---
document_type: story
story_id: STORY-103
epic_id: E-14
version: "1.0"
status: completed
producer: story-writer
timestamp: 2026-06-09T00:00:00Z
phase: 4
inputs:
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.009.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.010.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.011.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.012.md
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/phase-f2-spec-evolution/f2-fix-directives.md
input-hash: TBD
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-102]
blocks: [STORY-104]
behavioral_contracts:
  - BC-2.14.009
  - BC-2.14.010
  - BC-2.14.011
  - BC-2.14.012
verification_properties:
  - VP-022
priority: P0
cycle: v0.4.0-modbus
wave: 33
target_module: analyzer
subsystems: [SS-14]
estimated_days: 3
tdd_mode: strict
feature_id: issue-007-modbus-analyzer
github_issue: 7
# BC status: all 4 BCs authored at v1.0 as of 2026-06-09
input-hash: "50effc8"
---

# STORY-103: Modbus Flow State + Transaction Correlation

## Narrative

- **As a** ICS/OT security analyst using wirerust to detect Modbus attack sequences
- **I want** the Modbus analyzer to maintain per-flow state tracking all request/response pairs via a bounded pending table, and to attribute exception responses back to the originating request function code
- **So that** later detection stories (STORY-104) can use this state to identify write-class sequences, coordinated operations, and exception-burst anomalies

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.14.009 | Request PDU Inserted into Per-Flow Pending Table Keyed on (Transaction ID, Unit ID) |
| BC-2.14.010 | Response PDU Matched Against Pending Table and Entry Removed on FC Echo Match |
| BC-2.14.011 | Exception Response PDU Attributed to Originating Request FC via Pending Table Lookup |
| BC-2.14.012 | Pending Table Bounded to MAX_PENDING_TRANSACTIONS=256; New Requests Dropped When Full |

## Acceptance Criteria

### AC-001 (traces to BC-2.14.009 postcondition 1 — insert request into pending)
When a validated Modbus ADU arrives in `Direction::ClientToServer` and `classify_fc` returns a non-Exception class, `ModbusFlowState.pending` receives an entry keyed on `(transaction_id, unit_id): (u16, u8)` with value `(function_code, timestamp): (u8, u32)`. `ModbusFlowState.pdu_count` is incremented. `ModbusFlowState.last_ts` is updated to `timestamp`.
- **Test:** `test_request_pdu_inserted_into_pending()` — drive `on_data` with a Read-class ADU in ClientToServer direction; assert `flow.pending.contains_key(&(txn_id, unit_id))`; assert `pdu_count == 1`.

### AC-002 (traces to BC-2.14.009 postcondition 5 — overwrite on key collision; duplicate_inflight_txn)
If `pending.insert((txn_id, unit_id), (fc, ts))` overwrites an existing entry (reused transaction ID), `ModbusAnalyzer.duplicate_inflight_txn` is incremented by 1. The `HashMap::insert` return value is checked: `if insert.is_some() { self.duplicate_inflight_txn += 1; }`.
- **Test:** `test_duplicate_txn_id_increments_counter()` — insert two requests with the same `(txn_id, unit_id)` before a response; assert `duplicate_inflight_txn == 1`; assert pending table still has one entry (overwritten).

### AC-003 (traces to BC-2.14.010 postcondition 1 — response matched and entry removed)
When a validated Modbus ADU arrives in `Direction::ServerToClient` and the FC byte equals the `function_code` stored in `pending[(transaction_id, unit_id)]`, the pending entry is removed. `ModbusFlowState.pdu_count` is incremented. `ModbusFlowState.last_ts` is updated.
- **Test:** `test_response_pdu_matched_and_entry_removed()` — insert a request (ClientToServer, FC=0x03); then deliver a response (ServerToClient, FC=0x03, same txn_id/unit_id); assert `pending.is_empty()` after the response; assert `pdu_count == 2`.

### AC-004 (traces to BC-2.14.010 — unmatched response drops silently)
When a response arrives for a `(txn_id, unit_id)` key not in `pending` (e.g., connection mid-join), it is silently ignored. No finding emitted; `pdu_count` incremented; no error.
- **Test:** `test_unmatched_response_silently_ignored()` — deliver a ServerToClient response with no prior request; assert `all_findings.is_empty()`; assert `pdu_count == 1`.

### AC-005 (traces to BC-2.14.011 — exception response attributed to original FC)
When a response arrives with `classify_fc(fc) == Exception` and `pending` contains the key `(transaction_id, unit_id)`, the original request FC is recovered from the pending entry via `pending_fc = pending.remove((txn_id, unit_id)).unwrap().0`. The original FC is used for detection in STORY-104 (write-class exception attribution). `exception_count` is incremented.
- **Test:** `test_exception_response_attributed_to_original_fc()` — insert request (FC=0x06 Write); deliver exception response (FC=0x86 = 0x06 | 0x80); assert: `exception_count == 1`; the original FC (0x06) is correctly recovered (the test stubs the detection call or asserts a side-effect counter).

### AC-006 (traces to BC-2.14.012 — MAX_PENDING_TRANSACTIONS=256 cap)
`ModbusFlowState.pending` is capped at `MAX_PENDING_TRANSACTIONS = 256`. When the pending table is full and a new request arrives, the new entry is silently dropped (not inserted). The 256 existing entries are unaffected.
- **Test:** `test_pending_table_bounded_at_256()` — insert 256 requests with distinct `(txn_id, unit_id)` keys; attempt a 257th insert; assert `pending.len() == 256`; assert the 257th key is absent.

### AC-007 (traces to BC-2.14.012 — VP-022 pending-table bound integration test)
The `pending.len() < MAX_PENDING_TRANSACTIONS` guard is verified via an integration test that confirms no unbounded growth occurs. This is the VP-022 pending-table bound verification.
- **Test:** `test_pending_table_no_unbounded_growth()` — flood with 300 ClientToServer requests; assert `pending.len() <= 256` at all points; assert `pdu_count == 300` (all PDUs counted even if not inserted into pending).

### AC-008 (traces to BC-2.14.009 invariant 1 — key is (txn_id, unit_id), not txn_id alone)
The pending table key is `(u16, u8)` — a tuple of `transaction_id` and `unit_id`. This is explicitly NOT just `transaction_id: u16`. Two requests with the same `transaction_id` but different `unit_id` values produce TWO distinct pending entries.
- **Test:** `test_pending_key_is_txn_id_plus_unit_id()` — insert request with `(txn_id=1, unit_id=1)`; insert request with `(txn_id=1, unit_id=2)`; assert `pending.len() == 2`.

### AC-009 (traces to BC-2.14.009 — complete ModbusFlowState field list)
`ModbusFlowState` carries all 15+ fields per the authoritative field list in `f2-fix-directives.md Decision 11.4` and `architecture-delta.md §2.3`. This includes the dual-window fields required by STORY-104. The full field list is:
- `pending: HashMap<(u16, u8), (u8, u32)>` — bounded at 256
- `write_count: u64`, `exception_count: u64`, `pdu_count: u64`, `last_ts: u32`
- Burst window: `window_write_count: u32`, `window_start_ts: u32`, `window_burst_emitted: bool`
- Sustained window: `sustained_window_start_ts: u32`, `sustained_window_write_count: u32`, `sustained_burst_emitted: bool`
- T0831 window: `t0831_window_start_ts: u32`, `t0831_window_write_count: u32`, `t0831_burst_emitted: bool`
- Exception windows: `exception_window_counts: HashMap<u8, u32>`, `exception_window_start_ts: HashMap<u8, u32>`, `exception_burst_emitted: HashMap<u8, bool>`
- `is_non_modbus: bool`

**Test:** `test_modbus_flow_state_has_all_required_fields()` — compile-time; constructing `ModbusFlowState::default()` exercises all fields. Field count assertion is structural (code review + compile).

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `ModbusFlowState` (full field set) | `src/analyzer/modbus.rs` | Effectful (per-flow mutable state) |
| `pending: HashMap<(u16, u8), (u8, u32)>` | `src/analyzer/modbus.rs` | Effectful (mutable map) |
| Request-direction branch in `on_data` (pending insert) | `src/analyzer/modbus.rs` | Effectful |
| Response-direction branch in `on_data` (pending remove) | `src/analyzer/modbus.rs` | Effectful |
| Exception-direction branch (attribution lookup) | `src/analyzer/modbus.rs` | Effectful |
| `ModbusAnalyzer.duplicate_inflight_txn: u64` | `src/analyzer/modbus.rs` | Effectful (counter) |
| `MAX_PENDING_TRANSACTIONS: usize = 256` | `src/analyzer/modbus.rs` | Pure (constant) |

**Subsystem anchor justification:** SS-14 owns this story's complete scope — all changes are in `src/analyzer/modbus.rs`, the Modbus/ICS Analysis subsystem per ARCH-INDEX.

**Dependency anchor justification:** STORY-103 depends on STORY-102 because `parse_mbap_header`, `classify_fc`, `is_valid_modbus_adu`, and the `ModbusFlowState` stub are defined there. The `on_data` parsing loop that drives the request/response dispatch is built here on top of those foundations.

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | First ADU on new flow (empty pending) | Entry inserted; `pdu_count = 1`; `last_ts = timestamp` |
| EC-002 | Response arrives before any request (mid-join) | Silently ignored; `pdu_count` incremented; no finding |
| EC-003 | Exception response with no matching pending entry | No attribution; `exception_count` incremented; no finding |
| EC-004 | Pending table at 255/256 (one slot left) | 256th insert succeeds; 257th insert dropped (cap = 256, not 255) |
| EC-005 | Write-class FC in ClientToServer direction | Inserted into pending AND `window_write_count` incremented (both happen in the same `on_data` call — the detection windows share state with the pending-insert path) |
| EC-006 | `timestamp = 0` (epoch) | Valid; stored in pending entry as `(fc, 0)`; no special casing |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~4,000 |
| `src/analyzer/modbus.rs` (from STORY-102 + extensions) | ~5,000 |
| BC files (4 BCs: BC-2.14.009 through BC-2.14.012) | ~8,000 |
| `f2-fix-directives.md` Decision 11.4 (field list) | ~2,000 |
| `architecture-delta.md` §2.3 (ModbusFlowState layout) | ~2,000 |
| `tests/modbus_tests.rs` (transaction correlation tests) | ~5,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~27,000** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~14%** |

## Tasks (MANDATORY)

1. [ ] Extend `tests/modbus_tests.rs` with failing tests for AC-001 through AC-009. Red Gate: `ModbusFlowState` still has stub fields from STORY-102; the full `pending` HashMap and counter fields are not yet initialized.
2. [ ] **Red Gate:** Confirm `cargo test` fails on new AC assertions.
3. [ ] Expand `ModbusFlowState` to the full authoritative field list (AC-009). Replace the stub from STORY-102 with the complete struct. Use `#[derive(Default)]` where possible (HashMap, bool, u32, u64 all have sensible defaults).
4. [ ] Add `ModbusAnalyzer` struct: `all_findings: Vec<Finding>`, `total_pdu_count: u64`, `total_write_count: u64`, `duplicate_inflight_txn: u64`, `fn_code_counts: HashMap<u8, u64>`, `write_burst_threshold: u32`, `write_sustained_threshold: u32`. Add `ModbusAnalyzer::new(write_burst_threshold: u32, write_sustained_threshold: u32) -> Self`. Flow states stored in a `HashMap<FlowKey, ModbusFlowState>` on the analyzer.
5. [ ] Implement `on_data` in `ModbusAnalyzer` (the `StreamHandler` trait implementation stub — without detection logic yet, just parsing + pending table management):
   a. Check `flow.is_non_modbus`; bail if true.
   b. Loop over ADUs in the `data` slice: parse with `parse_mbap_header`; validate with `is_valid_modbus_adu`; advance offset by `6 + header.length`.
   c. For ClientToServer direction: classify FC; if not Exception, insert into `pending` if `< MAX_PENDING_TRANSACTIONS`; increment `pdu_count`, `last_ts`, `fn_code_counts`.
   d. For ServerToClient direction: if FC matches pending entry (echo or exception), remove entry; increment `exception_count` if exception.
   e. Detection stubs (empty functions or TODO comments) for STORY-104 calls.
6. [ ] Add `ModbusAnalyzer.flow_states: HashMap<FlowKey, ModbusFlowState>` and a `get_or_create_flow(&mut self, key: &FlowKey) -> &mut ModbusFlowState` helper.
7. [ ] **Green Gate:** `cargo build --all-targets` exits 0. `cargo test --all-targets` green. AC-001 through AC-009 pass.
8. [ ] `cargo clippy --all-targets -- -D warnings` clean.
9. [ ] `cargo fmt --check` clean.

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-102 | `parse_mbap_header`, `classify_fc`, `is_valid_modbus_adu` are all pure functions in `src/analyzer/modbus.rs`. `ModbusFlowState` has a stub with `is_non_modbus: bool`. | The `6 + header.length` offset advance rule. Exception FCs have high bit set; original FC = `fc & 0x7F`. | Do NOT check validity inside `parse_mbap_header`. The parser and the validator are separate functions — this separation enables the Kani proofs and the `is_non_modbus` desync detection. |

**Design reference:** Per `f2-fix-directives.md Decision 11.4`, the complete `ModbusFlowState` field list is authoritative. Do not add or remove fields without consulting the architect. The dual-window fields (`sustained_window_*`, `t0831_window_*`, `exception_window_*`) must be present in STORY-103 so that STORY-104 detection logic can write to them — even though no detection logic runs yet in this story.

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `ModbusFlowState.pending` is bounded at `MAX_PENDING_TRANSACTIONS = 256` | BC-2.14.012 | AC-006/AC-007 tests |
| Key is `(u16, u8)` — NOT `u16` alone | BC-2.14.009 invariant 1 | AC-008 test; Rust type system |
| `duplicate_inflight_txn` is on `ModbusAnalyzer` (NOT on `ModbusFlowState`) | BC-2.14.009 invariant 6 | Code review: field belongs to the analyzer-level counter, not the per-flow state |
| `on_data` request branch does NOT emit findings — only pending insert | BC-2.14.009 invariant 4 | Code review; no `all_findings.push` in request-insert path in this story |
| `exception_count` is incremented for exception responses (regardless of pending match) | BC-2.14.011 | AC-005 test |
| `src/analyzer/modbus.rs` MUST NOT import `src/reporter/` | Architecture layer rule | Compiler |
| All timestamp arithmetic uses `wrapping_sub` | f2-fix-directives.md §11.5b | Code review — even though no window timers fire in this story, initialize the wrapping-sub pattern now |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `std::collections::HashMap` | stdlib | `pending: HashMap<(u16, u8), (u8, u32)>`; `flow_states: HashMap<FlowKey, ModbusFlowState>`; exception window maps |
| No new external crates | — | All logic uses stdlib |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/analyzer/modbus.rs` | **modify** | Expand `ModbusFlowState` to full 15+ field set; add `ModbusAnalyzer` struct + `new()` + `flow_states` map; implement `on_data` parsing loop with pending table management |
| `tests/modbus_tests.rs` | **modify** | Add transaction correlation tests AC-001 through AC-009 |

## Forbidden Dependencies

`src/analyzer/modbus.rs` MUST NOT import:
- `src/reporter/` (L3 must not depend on L4)
- Any parse-combinator library (`nom`, `byteorder`, etc.) — only stdlib primitives
- `src/reassembly/` internals beyond what the `StreamHandler` trait requires (to be wired in STORY-105)
