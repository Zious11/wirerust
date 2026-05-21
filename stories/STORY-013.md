---
document_type: story
story_id: "STORY-013"
epic_id: "E-2"
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.004.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.005.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.050.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.051.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.052.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.053.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
points: "8"
depends_on: [STORY-011, STORY-012]
blocks: [STORY-014, STORY-015]
behavioral_contracts: [BC-2.04.004, BC-2.04.005, BC-2.04.050, BC-2.04.051, BC-2.04.052, BC-2.04.053]
verification_properties: [VP-001, VP-009]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 6
target_module: reassembly
subsystems: [SS-04]
estimated_days: "2"
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-verify
---

> **tdd_mode:** strict — all ACs must be backed by tests.

> **Execute:** `/vsdd-factory:deliver-story STORY-013`

# STORY-013: TCP Three-Way Handshake State Machine and Direction Tagging

## Narrative
- **As a** forensic analyst
- **I want** the TCP reassembly engine to correctly track the three-way handshake (SYN, SYN+ACK), derive initiator/responder roles, transition the flow state machine correctly, and tag all data with the right direction
- **So that** protocol analyzers receive correctly-labeled ClientToServer and ServerToClient data streams, and RST/FIN events reliably close flows

## Behavioral Contracts

| BC | Title | Role in Story |
|----|-------|---------------|
| BC-2.04.004 | First SYN Sets Client ISN and Initiator | SYN handling; state New->SynSent |
| BC-2.04.005 | SYN+ACK Marks Server as Responder; State Transitions to Established | SYN+ACK handling; state->Established |
| BC-2.04.050 | Flow State Machine: New->SynSent->Established->Closing->Closed | Full state transition table |
| BC-2.04.051 | RST Transitions State to Closed from Any Prior State | on_rst() unconditional close |
| BC-2.04.052 | on_data_without_syn: New->Established; partial=true | Mid-stream join state transition |
| BC-2.04.053 | TcpFlow::direction Returns ClientToServer When src Matches Initiator | Direction tagging logic |

## Acceptance Criteria

### AC-001 (traces to BC-2.04.004 postcondition 1)
- After processing a SYN packet from `src_ip:src_port`, `flow.initiator == Some((src_ip, src_port))`.
- **Test:** `test_BC_2_04_004_syn_sets_initiator()`

### AC-002 (traces to BC-2.04.004 postcondition 2)
- After processing a SYN packet, the client-to-server direction has `isn == Some(tcp.seq)`.
- **Test:** `test_BC_2_04_004_syn_sets_client_isn()`

### AC-003 (traces to BC-2.04.004 postcondition 3)
- After processing a SYN packet, `flow.state == FlowState::SynSent`.
- **Test:** `test_BC_2_04_004_syn_transitions_to_synsent()`

### AC-004 (traces to BC-2.04.004 invariant 1-2)
- A retransmitted SYN on the same flow does not change the stored initiator or ISN (both `set_initiator` and `set_isn` are idempotent).
- **Test:** `test_BC_2_04_004_retransmitted_syn_is_idempotent()`

### AC-005 (traces to BC-2.04.005 postcondition 1)
- After processing a SYN+ACK packet, `flow.initiator == Some((packet.dst_ip, tcp.dst_port))` — the DESTINATION of the SYN+ACK is the initiator.
- **Test:** `test_BC_2_04_005_syn_ack_sets_initiator_to_dst()`

### AC-006 (traces to BC-2.04.005 postcondition 2-3)
- After processing a SYN+ACK packet, the server-to-client direction has `isn == Some(tcp.seq)` and `flow.state == FlowState::Established`.
- **Test:** `test_BC_2_04_005_syn_ack_establishes_flow()`

### AC-007 (traces to BC-2.04.005 invariant 3)
- A SYN+ACK received without a prior SYN (mid-capture) still transitions the flow from `New` directly to `Established`.
- **Test:** `test_BC_2_04_005_syn_ack_without_prior_syn()`

### AC-008 (traces to BC-2.04.050 postcondition, all 9 transitions)
- The flow state machine implements the full 9-row transition table from BC-2.04.050:
  1. `on_syn()` New → SynSent
  2. `on_syn()` SynSent → SynSent (no-op guard; state unchanged)
  3. `on_syn_ack()` SynSent → Established
  4. `on_syn_ack()` New → Established (server-first: SYN+ACK without prior SYN)
  5. `on_data_without_syn()` New → Established (+ sets `partial = true`)
  6. `on_fin()` (first, `fin_count` becomes 1) Established → Closing
  7. `on_fin()` (first, `fin_count` becomes 1) SynSent → Closing
  8. `on_fin()` (second, `fin_count >= 2`) any → Closed
  9. `on_rst()` any → Closed
- **Test:** `test_BC_2_04_050_state_machine_all_transitions()` (must assert each of the 9 rows above individually)

### AC-009 (traces to BC-2.04.050 invariant 1)
- `on_syn()` is a no-op when the flow is already in `SynSent`, `Established`, `Closing`, or `Closed` state.
- **Test:** `test_BC_2_04_050_on_syn_no_op_when_not_new()`

### AC-010 (traces to BC-2.04.050 invariant 4)
- `fin_count` uses `saturating_add(1)` to prevent u8 overflow at 255.
- **Test:** `test_BC_2_04_050_fin_count_saturates_at_255()`

### AC-011 (traces to BC-2.04.051 invariant 1)
- `on_rst()` transitions the flow state to `Closed` from any prior state — `New`, `SynSent`, `Established`, `Closing`, or `Closed` — without any state guard.
- **Test:** `test_BC_2_04_051_rst_closes_from_any_state()`

### AC-012 (traces to BC-2.04.052 postcondition 1-2)
- When the first packet on a `New` flow carries data (no SYN), `on_data_without_syn()` transitions state to `Established` and sets `partial = true`.
- **Test:** `test_BC_2_04_052_data_without_syn_sets_partial()`

### AC-013 (traces to BC-2.04.052 invariant 1)
- `on_data_without_syn()` is a no-op when the flow is already in `Established` state.
- **Test:** `test_BC_2_04_052_on_data_without_syn_no_op_when_established()`

### AC-014 (traces to BC-2.04.053 postcondition 1)
- `TcpFlow::direction(src_ip, src_port)` returns `Direction::ClientToServer` when `src_ip:src_port` matches the stored initiator.
- **Test:** `test_BC_2_04_053_direction_client_to_server_when_src_is_initiator()`

### AC-015 (traces to BC-2.04.053 postcondition 2)
- `TcpFlow::direction(src_ip, src_port)` returns `Direction::ServerToClient` when `src_ip:src_port` does NOT match the stored initiator.
- **Test:** `test_BC_2_04_053_direction_server_to_client_when_src_is_not_initiator()`

### AC-016 (traces to BC-2.04.053 invariant 2)
- When `initiator` is `None`, `direction()` returns `Direction::ServerToClient` as a conservative default.
- **Test:** `test_BC_2_04_053_direction_server_to_client_when_no_initiator()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| TcpFlow::on_syn | src/reassembly/flow.rs | pure-core |
| TcpFlow::on_syn_ack | src/reassembly/flow.rs | pure-core |
| TcpFlow::on_fin | src/reassembly/flow.rs | pure-core |
| TcpFlow::on_rst | src/reassembly/flow.rs | pure-core |
| TcpFlow::on_data_without_syn | src/reassembly/flow.rs | pure-core |
| TcpFlow::direction | src/reassembly/flow.rs | pure-core |
| apply_handshake_flags | src/reassembly/mod.rs | effectful-shell (mutates flow table) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Retransmitted SYN | set_initiator/set_isn no-ops; state stays SynSent |
| EC-002 | SYN+ACK without prior SYN | initiator=dst; state->Established from New |
| EC-003 | SYN+ACK retransmission | All setters idempotent; state unchanged if already Established |
| EC-004 | SYN with payload | ISN set; payload processed normally in same call |
| EC-005 | RST on New flow | state=Closed; flows_rst++ |
| EC-006 | RST on Closing flow | state=Closed; flows_rst++ |
| EC-007 | RST with payload | Payload NOT processed; PostHandshake::FlowClosed returned |
| EC-008 | Both FINs from same direction (retransmit) | fin_count >= 2; flow closed |
| EC-009 | FIN on New flow | state->Closing; fin_count=1 |
| EC-010 | initiator=None when direction() called | Returns ServerToClient (conservative default) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reassembly/flow.rs (TcpFlow methods) | pure-core | No I/O; purely in-memory state mutations |
| src/reassembly/mod.rs (apply_handshake_flags) | effectful-shell | Mutates self.flows, self.stats |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| BC files (6 BCs) | ~6,000 |
| src/reassembly/flow.rs (state machine section ~lines 208-259) | ~1,500 |
| src/reassembly/mod.rs (apply_handshake_flags ~lines 257-287) | ~1,000 |
| Test files | ~4,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~16,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~8%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for all 16 ACs in `tests/reassembly_flow_tests.rs` and `tests/reassembly_engine_tests.rs`
2. [ ] Verify Red Gate: all tests fail before implementation changes
3. [ ] Verify existing implementation satisfies all ACs (brownfield)
4. [ ] Add state machine completeness test covering all 9 transitions in AC-008
5. [ ] Add test for fin_count saturation at u8::MAX (AC-010)
6. [ ] Verify direction() is called correctly in flush path
7. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-011 | brownfield-verify strategy | Tests use `tests/reassembly_flow_tests.rs` for flow-level, `tests/reassembly_engine_tests.rs` for engine-level | FlowKey commutativity is a prerequisite to direction tagging |
| STORY-012 | BTreeMap for detail map; summarize is pure | on_data callbacks are the source of truth for byte accounting | Non-TCP skip happens before flow lookup |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `on_syn()` guards on `FlowState::New` | BC-2.04.050 invariant 1 | Code review: assert guard in on_syn source |
| `fin_count` uses `saturating_add` | BC-2.04.050 invariant 4 | Code review: grep for saturating_add in flow.rs |
| `on_rst()` has NO state guard | BC-2.04.051 invariant 1 | Code review: absence of if-guard in on_rst |
| RST closes before payload processing | BC-2.04.051 postcondition 2 | Code review: PostHandshake::FlowClosed return |
| No `unsafe` blocks | prd.md §1.2 | cargo clippy |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stable toolchain | MSRV 1.85+ | FlowState enum, Direction enum |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/reassembly/flow.rs` | verify (lines 208-259) | on_syn, on_syn_ack, on_fin, on_rst, on_data_without_syn, direction |
| `src/reassembly/mod.rs` | verify (lines 257-287) | apply_handshake_flags with SYN/SYN+ACK/RST/FIN blocks |
| `tests/reassembly_flow_tests.rs` | modify | Add AC-001 through AC-016 |
| `tests/reassembly_engine_tests.rs` | modify | Add integration-level state transition tests |
