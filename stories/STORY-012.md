---
document_type: story
story_id: "STORY-012"
epic_id: "E-2"
version: "1.5"
status: completed
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.002.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.028.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.030.md
input-hash: "f66c50b"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-011]
blocks: [STORY-013]
behavioral_contracts: [BC-2.04.002, BC-2.04.028, BC-2.04.030]
verification_properties: [VP-010]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 5
target_module: reassembly
subsystems: [SS-04]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
nfr:
  - NFR-OBS-001
  - NFR-OBS-002
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — all ACs must be backed by tests.

> **Execute:** `/vsdd-factory:deliver-story STORY-012`

# STORY-012: Non-TCP Packet Filter, Statistics Summary, and bytes_reassembled Accounting

## Narrative
- **As a** forensic analyst
- **I want** the reassembly engine to silently skip non-TCP packets (counting them in statistics), expose a complete `AnalysisSummary` with all reassembly metrics, and accurately track every reassembled byte
- **So that** the engine operates correctly in mixed-protocol captures and the summary output faithfully reflects engine behavior for analyst review

## Behavioral Contracts

| BC | Title | Role in Story |
|----|-------|---------------|
| BC-2.04.002 | Non-TCP Packets Skipped; packets_skipped_non_tcp Increments | Protocol filter at process_packet entry |
| BC-2.04.028 | summarize Returns AnalysisSummary with Reassembly Stats Detail Map | Statistics observability surface |
| BC-2.04.030 | bytes_reassembled Equals Total Bytes Delivered to Handler | Byte accounting invariant |

## Acceptance Criteria

### AC-001 (traces to BC-2.04.002 postcondition 1)
- When `process_packet` is called with a non-TCP packet, `stats.packets_processed` increments by 1.
- **Test:** `test_BC_2_04_002_non_tcp_increments_packets_processed()`

### AC-002 (traces to BC-2.04.002 postcondition 2)
- When `process_packet` is called with a non-TCP packet, `stats.packets_skipped_non_tcp` increments by 1.
- **Test:** `test_BC_2_04_002_non_tcp_increments_skipped_counter()`

### AC-003 (traces to BC-2.04.002 postcondition 3)
- When `process_packet` is called with a non-TCP packet, `stats.packets_tcp` does NOT change.
- **Test:** `test_BC_2_04_002_non_tcp_does_not_increment_tcp_counter()`

### AC-004 (traces to BC-2.04.002 postcondition 4-6)
- When `process_packet` is called with a non-TCP packet, no flow is created or modified, no findings are emitted, and no handler callbacks (`on_data`, `on_flow_close`) are triggered.
- **Test:** `test_BC_2_04_002_non_tcp_creates_no_flow_no_callbacks()`

### AC-005 (traces to BC-2.04.002 invariant 1)
- After N non-TCP and M TCP packets, `packets_processed == N + M` and `packets_skipped_non_tcp == N` and `packets_tcp == M` (assuming all TCP packets have TransportInfo::Tcp).
- **Test:** `test_BC_2_04_002_mixed_protocol_counter_arithmetic()`

### AC-006 (traces to BC-2.04.028 postcondition 1)
- `summarize()` returns an `AnalysisSummary` with `analyzer_name == "TCP Reassembly"`.
- **Test:** `test_BC_2_04_028_summarize_analyzer_name()`

### AC-007 (traces to BC-2.04.028 postcondition 2)
- `summarize()` returns `packets_analyzed == stats.packets_tcp` (not `packets_processed`).
- **Test:** `test_BC_2_04_028_summarize_packets_analyzed_equals_tcp_count()`

### AC-008 (traces to BC-2.04.028 postcondition 3)
- The `detail` BTreeMap in the returned `AnalysisSummary` contains exactly the following keys and no others: `bytes_reassembled`, `dropped_findings`, `evictions`, `flows_completed`, `flows_expired`, `flows_fin`, `flows_partial`, `flows_rst`, `flows_total`, `packets_processed`, `packets_skipped_non_tcp`, `segments_depth_exceeded`, `segments_duplicates`, `segments_inserted`, `segments_out_of_window`, `segments_overlaps`, `segments_segment_limit`. Any missing or extra key is a test failure.
- **Test:** `test_BC_2_04_028_summarize_exact_key_set()`

### AC-009 (traces to BC-2.04.028 invariant 1)
- `flows_completed` in the detail map always equals `flows_fin + flows_rst` as reported by the stats.
- **Test:** `test_BC_2_04_028_flows_completed_derived_correctly()`

### AC-010 (traces to BC-2.04.028 invariant 3)
- The `detail` BTreeMap uses BTreeMap ordering (not HashMap), ensuring alphabetical key ordering in JSON serialization across runs.
- **Test:** `test_BC_2_04_028_detail_is_btreemap_ordered()`

### AC-011 (traces to BC-2.04.030 postcondition 1)
- After processing packets and calling `finalize()`, `stats.bytes_reassembled` equals the sum of all `data.len()` values passed to `handler.on_data` callbacks across all flows and both directions.
- **Test:** `test_BC_2_04_030_bytes_reassembled_matches_handler_total()`

### AC-012 (traces to BC-2.04.030 invariant 1)
- `bytes_reassembled` is monotonically non-decreasing; it never decreases between any two calls.
- **Test:** `test_BC_2_04_030_bytes_reassembled_is_monotonic()`

### AC-013 (traces to BC-2.04.030 postcondition 4)
- Duplicate retransmissions and out-of-window segments do NOT contribute to `bytes_reassembled` (they are discarded before flush).
- **Test:** `test_BC_2_04_030_duplicates_not_counted_in_bytes_reassembled()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| TcpReassembler::process_packet (entry guard) | src/reassembly/mod.rs | effectful-shell |
| TcpReassembler::summarize | src/reassembly/mod.rs | pure-core (immutable read of stats) |
| ReassemblyStats | src/reassembly/stats.rs | pure-core (data) |
| bytes_reassembled accounting | src/reassembly/mod.rs, lifecycle.rs | effectful-shell |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | UDP packet | Skipped; packets_skipped_non_tcp++ |
| EC-002 | ICMP packet (Protocol::Icmp) | Skipped; packets_skipped_non_tcp++ |
| EC-003 | Protocol::Other(n) packet | Skipped; packets_skipped_non_tcp++ |
| EC-004 | All packets non-TCP | flows.is_empty(), findings.is_empty() after all processed |
| EC-005 | summarize() called before any packets | All counters are 0; all-zero detail map |
| EC-006 | summarize() called after finalize() | Accurate snapshot; finalize does not reset stats |
| EC-007 | Non-TCP packets injected before summarize | packets_analyzed == packets_tcp (non-TCP excluded) |
| EC-008 | bytes_reassembled after out-of-order segment | Only counts after flush, not while buffered |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reassembly/mod.rs (process_packet entry) | effectful-shell | Mutates self.stats |
| src/reassembly/mod.rs (summarize) | pure-core | Immutable borrow of self.stats; no mutation |
| src/reassembly/stats.rs | pure-core | Data struct; no behavior |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| BC files (3 BCs) | ~3,000 |
| src/reassembly/mod.rs (summarize section ~lines 620-658) | ~1,000 |
| src/reassembly/stats.rs | ~600 |
| src/reassembly/mod.rs (process_packet entry) | ~800 |
| Test files | ~3,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~11,900** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~6%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for all 13 ACs in `tests/reassembly_engine_tests.rs`
2. [ ] Verify Red Gate: all tests fail before any implementation changes
3. [ ] Verify existing implementation satisfies all ACs (brownfield: tests should pass against existing code)
4. [ ] Add a test (AC-011) that captures ALL `on_data` callbacks across both flow directions and asserts `bytes_reassembled` equals the sum of their `data.len()` values
5. [ ] Add test verifying exact key set in detail map for AC-008 (no missing, no extra keys)
6. [ ] Verify purity: `summarize()` never mutates `self`
7. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-011 | Brownfield-verify strategy; five assert! in constructor | Tests live in `tests/reassembly_engine_tests.rs` and `tests/reassembly_flow_tests.rs` | BTreeMap key set must be exact — no missing, no extra |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `summarize()` must take `&self` (immutable borrow) | prd.md §1.2 (no global state) | cargo check; no &mut self in summarize signature |
| BTreeMap (not HashMap) for detail map | BC-2.04.028 invariant 3 (LESSON-P2.09) | Type check: `BTreeMap<String, serde_json::Value>` |
| No `unsafe` blocks | prd.md §1.2 | `cargo clippy --all-targets -- -D warnings` |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| serde_json | from Cargo.toml | `serde_json::Value::Number` for detail map values |
| Rust stable toolchain | MSRV 1.85+ | BTreeMap, IpAddr, etc. |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/reassembly/mod.rs` | verify (lines 140-145, 187-190, 620-658) | Non-TCP skip, process_packet entry, summarize |
| `src/reassembly/stats.rs` | verify | ReassemblyStats struct fields |
| `tests/reassembly_engine_tests.rs` | modify | Add AC-001 through AC-013 tests |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.4 | 2026-05-22 | story-writer | Wave 5 delivery: status advanced draft → completed — STORY-012 delivered via PR #118, merge bbddac6 |
| 1.3 | 2026-05-22 | story-writer | Wave 5 Ph3 per-story adversarial fix Min-3: File Structure anchor synced with BC-2.04.002 and actual src/reassembly/mod.rs — non-TCP guard range corrected from 186-190 to 187-190 (line 186 is the fn signature; lines 187-190 are the guard body) |
| 1.2 | 2026-05-22 | story-writer | Wave 5 Ph3 per-story adversarial fix M-2: Task 4 corrected — removed stale '16 on_data calls' planning estimate that did not match the delivered AC-011 test |
| 1.1 | 2026-05-21 | story-writer | Initial story decomposition |
