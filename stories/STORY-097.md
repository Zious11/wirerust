---
document_type: story
story_id: STORY-097
epic_id: E-12
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 3
inputs:
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.055.md
  - .factory/feature-delta/issue-100-pcap-timestamps/delta-analysis.md
input-hash: 4f869f0
traces_to: .factory/specs/prd.md
points: 5
depends_on: []
blocks: [STORY-098]
behavioral_contracts:
  - BC-2.04.055
verification_properties:
  - VP-021
priority: P1
cycle: v0.2.0-feature-100
wave: 28
target_module: reassembly
subsystems: [SS-04, SS-05]
estimated_days: 2
tdd_mode: strict
feature_id: issue-100-pcap-timestamps
github_issue: 100
---

# STORY-097: Thread Capture-Relative Timestamp Through StreamHandler::on_data

## Narrative

- **As a** forensic analyst using wirerust to analyze pcap captures
- **I want** `StreamHandler::on_data` to carry the capture-relative pcap `ts_sec` as a `timestamp: u32` parameter
- **So that** all downstream analyzers receive the packet-level timestamp at every flush call site, enabling `Finding.timestamp` to be populated with real pcap provenance instead of `None`

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.04.055 | StreamHandler::on_data Carries Capture-Relative Timestamp Parameter |

## Acceptance Criteria

### AC-001 (traces to BC-2.04.055 postcondition 1 — hot-path case)
`StreamHandler::on_data` signature includes `timestamp: u32` as the fifth parameter: `fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], offset: u64, timestamp: u32)`. The trait definition at `src/reassembly/handler.rs:49` compiles with this signature.
- **Test:** `test_on_data_signature_has_timestamp_param()` — compile-time; confirmed by `cargo check` passing.

### AC-002 (traces to BC-2.04.055 postcondition 1 — hot-path case)
`flush_contiguous_data` in `src/reassembly/mod.rs` passes the current packet's `timestamp_secs` as the `timestamp` argument to `handler.on_data`. The `u32` value is taken from the `timestamp: u32` parameter already in scope in `process_packet` and threaded through to `flush_contiguous_data`.
- **Test:** `test_flush_contiguous_data_passes_current_packet_timestamp()` — integration: process a packet with a known `ts_sec`, assert the `RecordingHandler` receives that exact `timestamp` value in its `on_data` callback.

### AC-003 (traces to BC-2.04.055 postcondition 1 — close-flush case)
`close_flow` in `src/reassembly/lifecycle.rs` passes `flow.last_seen` as the `timestamp` argument to `handler.on_data`. The value is read from the `TcpFlow` value obtained via `self.flows.remove(key)` before it is dropped.
- **Test:** `test_close_flow_passes_flow_last_seen_timestamp()` — integration: set up a flow with a known `last_seen`, trigger FIN close, assert the `RecordingHandler` receives `flow.last_seen` as timestamp.

### AC-004 (traces to BC-2.04.055 postcondition 2)
`StreamDispatcher::on_data` at `src/dispatcher.rs:144` accepts the new `timestamp: u32` parameter and forwards it unchanged to both `http.on_data(...)` and `tls.on_data(...)` downstream calls at the dispatch sites.
- **Test:** `test_stream_dispatcher_forwards_timestamp_to_analyzers()` — unit test on `StreamDispatcher`: given `timestamp=7777`, assert both downstream `on_data` calls receive `7777`.

### AC-005 (traces to BC-2.04.055 invariant 4 — compile completeness)
All five production implementors of `StreamHandler` compile with the updated signature: `TlsAnalyzer`, `HttpAnalyzer`, `StreamDispatcher`, `RecordingHandler` (in `tests/reassembly_engine_tests.rs`), and the anonymous handler in `tests/hs043_flow_expiry_tests.rs`. No `cargo build` error remains.
- **Test:** `cargo build --all-targets` exits 0 — compile-time guarantee by the trait-signature break.

### AC-006 (traces to BC-2.04.055 invariant 3 — existing tests pass)
All existing `cargo test --all-targets` tests pass after the signature update. The test handlers (`RecordingHandler` and the anonymous handler in `hs043_flow_expiry_tests.rs`) accept the new `_timestamp: u32` (or `timestamp: u32`) parameter without breaking any existing assertion.
- **Test:** `cargo test --all-targets` green with no regressions on the reassembly test suite.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `StreamHandler` trait | `src/reassembly/handler.rs` | Pure-core (trait definition) |
| `flush_contiguous_data` | `src/reassembly/mod.rs` | Effectful (mutates reassembler state) |
| `close_flow` | `src/reassembly/lifecycle.rs` | Effectful (removes flow from map) |
| `StreamDispatcher::on_data` | `src/dispatcher.rs` | Effectful (dispatches to analyzers) |
| `TlsAnalyzer::on_data` (signature only) | `src/analyzer/tls.rs` | Effectful (per-flow state; emission deferred to STORY-098) |
| `HttpAnalyzer::on_data` (signature only) | `src/analyzer/http.rs` | Effectful (per-flow state; emission deferred to STORY-098) |
| `RecordingHandler::on_data` | `tests/reassembly_engine_tests.rs` | Effectful (test infrastructure) |
| Anonymous handler | `tests/hs043_flow_expiry_tests.rs` | Effectful (test infrastructure) |

**Subsystem anchor justification:**
- SS-04 owns this story's scope because the `StreamHandler` trait and its flush call sites (`flush_contiguous_data`, `close_flow`) live in `src/reassembly/` — SS-04 (TCP Stream Reassembly Engine) per ARCH-INDEX Subsystem Registry.
- SS-05 is touched because `StreamDispatcher` (the dispatcher module) forwards the new parameter; the dispatcher is the SS-04/SS-06/SS-07 boundary.

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | `flush_contiguous_data` called for a packet with `ts_sec=0` (Unix epoch) | `timestamp=0` is passed to `on_data`; implementors must accept 0 as valid; `TcpFlow::new` can set `last_seen=0` if the first packet has epoch timestamp |
| EC-002 | `close_flow` called after exactly one packet | `flow.last_seen` equals the timestamp of that single packet; non-zero for any real packet; passed correctly |
| EC-003 | `StreamDispatcher` routing to only one active analyzer (e.g., HTTP but not TLS) | The active analyzer's `on_data` receives `timestamp`; inactive path is a no-op; no panic |
| EC-004 | `RecordingHandler` does not record timestamp in `data_events` tuple | Compilation succeeds; existing test assertions on `flow_key`, `direction`, `data`, `offset` are unaffected; `_timestamp` is discarded if unused |
| EC-005 | Timeout-eviction path in `close_flow` | `flow.last_seen` is the timestamp of the last packet seen before idle timeout; correct forensic anchor for expiry |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `src/reassembly/handler.rs` (trait) | Pure | Trait method declaration; no state |
| `src/reassembly/mod.rs` (`flush_contiguous_data`) | Effectful | Mutates reassembler state; calls `handler.on_data` |
| `src/reassembly/lifecycle.rs` (`close_flow`) | Effectful | Removes flow from `self.flows`; calls `handler.on_data` |
| `src/dispatcher.rs` (`StreamDispatcher::on_data`) | Effectful | Dispatches to child analyzers |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| `src/reassembly/handler.rs` (trait definition) | ~1,500 |
| `src/reassembly/mod.rs` (flush_contiguous_data) | ~6,000 |
| `src/reassembly/lifecycle.rs` (close_flow) | ~3,000 |
| `src/dispatcher.rs` (StreamDispatcher) | ~4,000 |
| `src/analyzer/tls.rs` (on_data signature update only) | ~3,000 |
| `src/analyzer/http.rs` (on_data signature update only) | ~3,000 |
| `tests/reassembly_engine_tests.rs` (RecordingHandler + new tests) | ~8,000 |
| `tests/hs043_flow_expiry_tests.rs` (anonymous handler) | ~2,000 |
| BC files (1 BC: BC-2.04.055) | ~3,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~37,500** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~19%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-002 and AC-003 in `tests/reassembly_engine_tests.rs` — specifically `test_flush_contiguous_data_passes_current_packet_timestamp()` and `test_close_flow_passes_flow_last_seen_timestamp()`. Confirm `RecordingHandler.data_events` tuple needs a `u32` timestamp field added to support these assertions.
2. [ ] Write failing test for AC-004 in `tests/reassembly_engine_tests.rs` or a new `tests/timestamp_threading_tests.rs` — `test_stream_dispatcher_forwards_timestamp_to_analyzers()`.
3. [ ] **Red Gate:** Confirm `cargo test` FAILS (does not compile) because `on_data` still has 4 params while tests expect 5.
4. [ ] Update `StreamHandler::on_data` in `src/reassembly/handler.rs:49` to add `timestamp: u32` as the fifth parameter.
5. [ ] Update `flush_contiguous_data` in `src/reassembly/mod.rs` to pass the current-packet `timestamp: u32` to `handler.on_data(...)`.
6. [ ] Update `close_flow` in `src/reassembly/lifecycle.rs` to pass `flow.last_seen` to `handler.on_data(...)`. Note: `flow` is available from `let Some(mut flow) = self.flows.remove(key)` at `:42`.
7. [ ] Update `StreamDispatcher::on_data` in `src/dispatcher.rs:144` to accept `timestamp: u32` and forward it to both `http.on_data(...)` and `tls.on_data(...)` downstream calls.
8. [ ] Update `TlsAnalyzer::on_data` signature at `src/analyzer/tls.rs:771` to accept `timestamp: u32`. Store the value in a per-flow field (add `last_ts: u32` to flow state map) for use by STORY-098 emission sites. The `timestamp` parameter must not be silently ignored.
9. [ ] Update `HttpAnalyzer::on_data` signature at `src/analyzer/http.rs:501` to accept `timestamp: u32`. Store the value per-flow (add `last_ts: u32` to flow state map) for STORY-098. The `timestamp` parameter must not be silently ignored.
10. [ ] Update `RecordingHandler::on_data` in `tests/reassembly_engine_tests.rs:51` to accept `timestamp: u32`. Update `data_events` tuple type from `(FlowKey, Direction, Vec<u8>, u64)` to `(FlowKey, Direction, Vec<u8>, u64, u32)` if timestamp assertions in AC-002/AC-003 require it; otherwise use `_timestamp`.
11. [ ] Update anonymous inline handler `on_data` in `tests/hs043_flow_expiry_tests.rs:69` to accept `_timestamp: u32`.
12. [ ] **Green Gate:** `cargo build --all-targets` compiles (AC-005). `cargo test --all-targets` passes with no regressions (AC-006). New tests for AC-002, AC-003, AC-004 pass (Green Gate confirmed).
13. [ ] `cargo clippy --all-targets -- -D warnings` clean.
14. [ ] `cargo fmt --check` clean.

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A — first story in E-12 (feature issue-100) | — | — | This is a BREAKING CHANGE to a public trait. The trait-signature update must happen atomically: all five implementors must be updated in the same PR. The build cannot be in a partially-updated state — `cargo build` fails until every implementor compiles. Do not merge a PR that only updates the trait definition without updating all implementors. |

**Design reference:** Per F1 delta-analysis §4.3, `close_flow` at `lifecycle.rs:42` already has `let Some(mut flow) = self.flows.remove(key)`, so `flow.last_seen` is directly accessible with zero new state. Do not add a separate timestamp parameter to `close_flow` itself — the `TcpFlow` value is already in scope.

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `StreamHandler::on_data` signature is `fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], offset: u64, timestamp: u32)` | BC-2.04.055 postcondition 1 | Compiler: all implementors must match; any missed implementor causes a compile error |
| `flush_contiguous_data` passes `current_packet_timestamp` (not `flow.last_seen`) | BC-2.04.055 postcondition 1 (hot-path case) | Code review; test AC-002 asserts exact value |
| `close_flow` passes `flow.last_seen` (not zero, not current packet) | BC-2.04.055 postcondition 1 (close-flush case) | Code review; test AC-003 asserts exact value |
| `StreamDispatcher::on_data` forwards `timestamp` UNCHANGED to both `http.on_data` and `tls.on_data` | BC-2.04.055 postcondition 2 | Test AC-004 asserts value identity |
| Per-flow timestamp storage in `HttpAnalyzer` and `TlsAnalyzer` MUST key by `FlowKey` | BC-2.04.055 invariant 3 (cross-flow isolation) | Code review; VP-014 cross-flow isolation property covers this pattern |
| The `timestamp` parameter is `u32` (not `Option<u32>`) — always concrete | BC-2.04.055 invariant 2 | Compiler: trait signature enforces non-optional type |
| SS-04 must NOT depend on SS-06/SS-07 internals (only through the trait interface) | Architecture layer rules (ARCH-INDEX) | `src/reassembly/` must not import `src/analyzer/`; only `handler.rs` trait is the boundary |
| Sub-second precision (`timestamp_usecs`) is OUT OF SCOPE for this story | BC-2.04.055 invariant 5 | F1 open question 1 deferred; `timestamp: u32` carries seconds precision only |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `chrono` | workspace version (current: `0.4`) | `DateTime<Utc>` type used in Finding; NOT used in this story (u32 is passed, not converted here) |
| `pcap_file` | workspace version | `RawPacket.timestamp_secs: u32` source; no version change |
| `proptest` | workspace version | Used by VP-014 tests; not required for this story's tests (integration tests suffice) |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/reassembly/handler.rs` | **modify** | Add `timestamp: u32` as fifth param to `StreamHandler::on_data` trait method |
| `src/reassembly/mod.rs` | **modify** | `flush_contiguous_data`: pass current-packet `timestamp` to `handler.on_data` |
| `src/reassembly/lifecycle.rs` | **modify** | `close_flow`: pass `flow.last_seen` to `handler.on_data` |
| `src/dispatcher.rs` | **modify** | `StreamDispatcher::on_data`: add `timestamp: u32` param; forward to `http.on_data` and `tls.on_data` |
| `src/analyzer/tls.rs` | **modify** | `TlsAnalyzer::on_data`: add `timestamp: u32` param; store in per-flow `last_ts: u32` field |
| `src/analyzer/http.rs` | **modify** | `HttpAnalyzer::on_data`: add `timestamp: u32` param; store in per-flow `last_ts: u32` field |
| `tests/reassembly_engine_tests.rs` | **modify** | `RecordingHandler::on_data`: add `timestamp: u32` param; update `data_events` tuple if needed; add AC-002/AC-003 tests |
| `tests/hs043_flow_expiry_tests.rs` | **modify** | Anonymous handler `on_data`: add `_timestamp: u32` param |
