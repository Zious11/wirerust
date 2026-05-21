---
document_type: story
story_id: "STORY-011"
epic_id: "E-2"
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.001.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.003.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.049.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-005]
blocks: [STORY-012]
behavioral_contracts: [BC-2.04.001, BC-2.04.003, BC-2.04.049]
verification_properties: [VP-001]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 4
target_module: reassembly
subsystems: [SS-04]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — all ACs must be backed by tests; `todo!()` stubs required before implementation.

> **Execute:** `/vsdd-factory:deliver-story STORY-011`

# STORY-011: TcpReassembler Constructor Validation and FlowKey Canonicalization

## Narrative
- **As a** forensic analyst
- **I want to** have the TCP reassembly engine fail loudly at construction time when given zero-valued configuration fields, and have bidirectional flow keys that are commutative
- **So that** programming errors in configuration are detected immediately (not silently downstream) and every packet from both directions of a TCP connection maps to the same flow table entry

## Behavioral Contracts

| BC | Title | Role in Story |
|----|-------|---------------|
| BC-2.04.001 | TcpReassembler::new Panics on Invalid Config | Constructor validation — five `assert!` calls |
| BC-2.04.003 | Canonical FlowKey Ordering Ensures A->B and B->A Produce Identical Key | FlowKey::new commutativity |
| BC-2.04.049 | FlowKey::Display Uses U+2192 Arrow (Not ASCII ->) | Display output encoding contract |

## Acceptance Criteria

### AC-001 (traces to BC-2.04.001 postcondition 1)
- When `TcpReassembler::new` is called with `config.max_depth == 0` and all other fields valid, the constructor panics with a message containing the exact string `"max_depth must be > 0"`.
- **Test:** `test_BC_2_04_001_max_depth_zero_panics()`

### AC-002 (traces to BC-2.04.001 postcondition 2)
- When `config.memcap == 0`, the constructor panics with a message containing `"memcap must be > 0"`.
- **Test:** `test_BC_2_04_001_memcap_zero_panics()`

### AC-003 (traces to BC-2.04.001 postcondition 3)
- When `config.max_flows == 0`, the constructor panics with a message containing `"max_flows must be > 0"`.
- **Test:** `test_BC_2_04_001_max_flows_zero_panics()`

### AC-004 (traces to BC-2.04.001 postcondition 4)
- When `config.max_segments_per_direction == 0`, the constructor panics with a message containing `"max_segments_per_direction must be > 0"`.
- **Test:** `test_BC_2_04_001_max_segments_per_direction_zero_panics()`

### AC-005 (traces to BC-2.04.001 postcondition 5)
- When `config.max_receive_window == 0`, the constructor panics with a message containing `"max_receive_window must be > 0"`.
- **Test:** `test_BC_2_04_001_max_receive_window_zero_panics()`

### AC-006 (traces to BC-2.04.001 postcondition 6)
- When all five validated fields are `> 0` (including `ReassemblyConfig::default()`), the constructor succeeds and returns a `TcpReassembler` with empty flows, empty findings, `total_memory == 0`, and `finalized == false`.
- **Test:** `test_BC_2_04_001_valid_config_constructs_successfully()`

### AC-007 (traces to BC-2.04.001 invariant 2)
- `flow_timeout_secs == 0` in the config does NOT cause a panic; the constructor accepts it as a legal value.
- **Test:** `test_BC_2_04_001_flow_timeout_zero_is_legal()`

### AC-008 (traces to BC-2.04.003 postcondition 1)
- `FlowKey::new(ip_a, port_a, ip_b, port_b)` stores the endpoint where `(ip, port) <= (other_ip, other_port)` as the lower endpoint, using tuple-pair comparison.
- **Test:** `test_BC_2_04_003_lower_endpoint_stored_correctly()`

### AC-009 (traces to BC-2.04.003 postcondition 2)
- `FlowKey::new(ip_a, port_a, ip_b, port_b) == FlowKey::new(ip_b, port_b, ip_a, port_a)` for all valid inputs. The key is commutative.
- **Test:** `test_BC_2_04_003_flow_key_is_commutative()`

### AC-010 (traces to BC-2.04.003 invariant 1)
- The ordering uses tuple-pair comparison `(ip_a, port_a) <= (ip_b, port_b)`, NOT independent per-field sorting. A case that distinguishes the two orderings (same IP, different ports) must produce the correct result.
- **Test:** `test_BC_2_04_003_tuple_pair_ordering_not_independent_field()`

### AC-011 (traces to BC-2.04.049 postcondition 1)
- `format!("{}", flow_key)` produces a string where the separator between endpoints is the Unicode RIGHT ARROW character U+2192 (UTF-8: 0xE2 0x86 0x92), NOT ASCII `->` (0x2D 0x3E).
- **Test:** `test_BC_2_04_049_display_uses_unicode_arrow()`

### AC-012 (traces to BC-2.04.049 invariant 1)
- The lower (canonically-ordered) endpoint appears on the left side of the U+2192 arrow, and the upper endpoint on the right.
- **Test:** `test_BC_2_04_049_display_canonical_order()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| TcpReassembler::new | src/reassembly/mod.rs | effectful-shell (panics on invalid config) |
| FlowKey::new | src/reassembly/flow.rs | pure-core |
| FlowKey::Display | src/reassembly/flow.rs | pure-core |
| ReassemblyConfig | src/reassembly/config.rs | pure-core |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | max_depth=0, all others valid | panic: "max_depth must be > 0" |
| EC-002 | memcap=0, all others valid | panic: "memcap must be > 0" |
| EC-003 | max_flows=0 | panic: "max_flows must be > 0" |
| EC-004 | max_segments_per_direction=0 | panic: "max_segments_per_direction must be > 0" |
| EC-005 | max_receive_window=0 | panic: "max_receive_window must be > 0" |
| EC-006 | ReassemblyConfig::default() (all > 0) | Valid TcpReassembler; no panic |
| EC-007 | flow_timeout_secs=0 | No panic; legal value |
| EC-008 | FlowKey with same IP, different ports | tuple-pair ordering applies; lower port wins |
| EC-009 | IPv4 vs IPv6 addresses in FlowKey | IPv4 < IPv6 in IpAddr PartialOrd; IPv4 endpoint becomes lower |
| EC-010 | FlowKey display with IPv6 endpoint | No RFC-3986 brackets; plain IPv6 colon notation |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reassembly/mod.rs (TcpReassembler::new) | effectful-shell | Panics on invalid config; side effect on programming error |
| src/reassembly/flow.rs (FlowKey::new) | pure-core | Deterministic tuple-pair comparison, no I/O |
| src/reassembly/flow.rs (FlowKey::Display) | pure-core | Pure fmt::Display implementation |
| src/reassembly/config.rs | pure-core | Data definition only |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| BC files (3 BCs) | ~3,000 |
| src/reassembly/mod.rs (constructor section) | ~1,500 |
| src/reassembly/flow.rs (FlowKey section) | ~1,500 |
| src/reassembly/config.rs | ~500 |
| Test files (existing + new) | ~3,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~13,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~6.5%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for all 12 ACs in `tests/reassembly_engine_tests.rs` and `tests/reassembly_flow_tests.rs`
2. [ ] Verify Red Gate: all tests fail before implementation changes
3. [ ] Verify existing implementation satisfies all ACs (brownfield: tests should pass against existing code)
4. [ ] Add any missing `#[should_panic(expected = "...")]` tests for the five panic cases
5. [ ] Add test for U+2192 Unicode arrow by asserting exact UTF-8 bytes in Display output
6. [ ] Add property-based test for FlowKey commutativity using proptest
7. [ ] Verify purity boundaries: FlowKey::new is pure-core, TcpReassembler::new is effectful-shell
8. [ ] Update STATE.md with story completion status

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A — first story in epic | — | — | — |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| No `unsafe` blocks in reassembly module | prd.md §1.2 ("no unsafe blocks") | `cargo clippy --all-targets -- -D warnings` |
| Rust 2024 edition | CLAUDE.md | `rustfmt.toml` edition field; cargo check |
| Overflow-checked arithmetic | CLAUDE.md (`overflow-checks = true` in release) | `cargo build --release` |
| Pure functions must have no I/O or global state access | Architecture purity policy | Code review; grep for eprintln!/println! in pure modules |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stable toolchain | MSRV 1.85+ (2024 edition) | Core language; `wrapping_sub`, `saturating_add` |
| proptest | from Cargo.toml | Property-based test for FlowKey commutativity |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/reassembly/mod.rs` | verify (lines 107-127) | TcpReassembler::new with five assert! calls |
| `src/reassembly/flow.rs` | verify (lines 45-74) | FlowKey::new and Display implementation |
| `src/reassembly/config.rs` | verify | ReassemblyConfig field definitions |
| `tests/reassembly_engine_tests.rs` | modify | Add AC-001 through AC-007 tests |
| `tests/reassembly_flow_tests.rs` | modify | Add AC-008 through AC-012 tests |
