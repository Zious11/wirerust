---
document_type: story
story_id: STORY-105
epic_id: E-14
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-06-09T00:00:00Z
phase: 4
inputs:
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.023.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.024.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.025.md
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/phase-f2-spec-evolution/f2-fix-directives.md
input-hash: TBD
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-104]
blocks: []
behavioral_contracts:
  - BC-2.14.023
  - BC-2.14.024
  - BC-2.14.025
verification_properties:
  - VP-004
  - VP-022
priority: P0
cycle: v0.4.0-modbus
wave: 34
target_module: analyzer
subsystems: [SS-14, SS-05, SS-12]
estimated_days: 3
tdd_mode: strict
feature_id: issue-007-modbus-analyzer
github_issue: 7
# BC status: BC-2.14.023/024/025 authored at v1.0/v2.0 as of 2026-06-09
input-hash: "a9ac815"
---

# STORY-105: Modbus Dispatcher Integration + CLI

## Narrative

- **As a** SOC analyst or ICS security engineer invoking wirerust on Modbus TCP captures
- **I want** to be able to pass `--modbus` (or `--all`) to enable the Modbus analyzer, configure burst thresholds via `--modbus-write-burst-threshold` and `--modbus-write-sustained-threshold`, and have port-502 flows routed to `ModbusAnalyzer` by the dispatcher
- **So that** the Modbus TCP analyzer is wired into the full wirerust pipeline and produces findings and summaries in all supported output formats

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.14.023 | --modbus CLI Flag Enables ModbusAnalyzer; --all Includes Modbus; Default-Off; Requires Stream Reassembly |
| BC-2.14.024 | --modbus-write-burst-threshold and --modbus-write-sustained-threshold Configure Dual-Window Burst Detection |
| BC-2.14.025 | StreamDispatcher Classifies Port-502 Flows to DispatchTarget::Modbus as Rule 5 (After Content and TLS/HTTP Port Rules) |

## Acceptance Criteria

### AC-001 (traces to BC-2.14.023 postcondition P2 — `--modbus` enables analyzer)
When `--modbus` is present and `--no-reassemble` is absent: `ModbusAnalyzer::new(write_burst_threshold, write_sustained_threshold)` is constructed with the CLI-supplied (or default) thresholds; it is passed to `StreamDispatcher::new` as `modbus: Some(modbus_analyzer)`. `needs_reassembly = enable_http || enable_tls || enable_modbus` — the `|| enable_modbus` term is present.
- **Test:** Integration test: `wirerust analyze empty.pcap --modbus` exits 0; output includes a Modbus section in the analyzer summaries.

### AC-002 (traces to BC-2.14.023 postcondition P3 — `--all` includes Modbus)
`enable_modbus = *modbus || *all`. When `--all` is passed without `--modbus`, Modbus is still enabled. The pattern is identical to the existing `--http`/`--tls`/`--dns` expansion.
- **Test:** `test_all_flag_enables_modbus()` — run with `--all`; assert `ModbusAnalyzer` is constructed (check for Modbus summary section in output).

### AC-003 (traces to BC-2.14.023 postcondition P1 — default off)
Without `--modbus` or `--all`, `enable_modbus = false`; `modbus_analyzer = None`; no Modbus section in output; port-502 flows receive `DispatchTarget::None`. This is the default behavior — no breaking change to existing invocations.
- **Test:** `test_modbus_disabled_by_default()` — run with only `--http`; assert no Modbus section in output.

### AC-004 (traces to BC-2.14.023 postcondition P2 sub-case — `--modbus` + `--no-reassemble` warning)
When `--modbus` and `--no-reassemble` are both present: a warning is printed to stderr `"WARNING: --modbus requires stream reassembly; ignoring --modbus (pass --reassemble or omit --no-reassemble)"`; `modbus_analyzer = None`; exit 0.
- **Test:** `test_modbus_with_no_reassemble_prints_warning()` — run with both flags; assert stderr contains the warning; assert no Modbus section in output.

### AC-005 (traces to BC-2.14.024 postcondition — dual CLI flags with defaults)
`--modbus-write-burst-threshold` (default 20) and `--modbus-write-sustained-threshold` (default 10) are accepted by the `analyze` subcommand. Both are `u32` fields in the `Cli` struct. `ModbusAnalyzer::new(burst, sustained)` receives the parsed values. Both flags are validated: value must be >= 1; value 0 produces error `"--modbus-write-burst-threshold must be >= 1"` / `"--modbus-write-sustained-threshold must be >= 1"`.
- **Test:** `test_burst_threshold_flag_parsed_and_forwarded()` — run with `--modbus --modbus-write-burst-threshold 5`; process 6 writes within 1s; assert burst finding emitted. `test_threshold_zero_rejected()` — run with `--modbus-write-burst-threshold 0`; assert non-zero exit and error message.

### AC-006 (traces to BC-2.14.025 postcondition — port-502 Rule 5)
`StreamDispatcher::classify` adds `DispatchTarget::Modbus` as Rule 5: AFTER content rules (TLS ClientHello / HTTP GET/POST = Rules 1, 2) and AFTER TLS/HTTP port rules (443/8443 = Rule 3, 80/8080 = Rule 4) — but BEFORE `DispatchTarget::None`. Port 502 flows that do not match Rules 1-4 → Rule 5 → `DispatchTarget::Modbus`. Port 502 flows carrying TLS ClientHello bytes → Rule 1 → `DispatchTarget::Tls` (content first).
- **Test:** `test_port_502_classified_to_modbus_as_rule_5()` — classify a port-502 flow with non-TLS/HTTP bytes; assert `DispatchTarget::Modbus`. `test_port_502_tls_content_classified_to_tls()` — classify a port-502 flow with TLS ClientHello bytes; assert `DispatchTarget::Tls`.

### AC-007 (traces to BC-2.14.025 — `on_data` routing to ModbusAnalyzer)
`StreamDispatcher::on_data` dispatches to `self.modbus.as_mut().unwrap().on_data(...)` when the flow's `DispatchTarget` is `Modbus`. The `timestamp: u32` parameter is forwarded unchanged (same pattern as HTTP/TLS dispatch per BC-2.04.055).
- **Test:** `test_dispatcher_routes_modbus_flow_to_analyzer()` — construct a `StreamDispatcher` with a `ModbusAnalyzer`; deliver bytes on a port-502 flow; assert `ModbusAnalyzer.total_pdu_count > 0`.

### AC-008 (traces to BC-2.14.023 postcondition P5 — post-finalize collection)
After the packet loop and `reassembler.finalize()`, `dispatcher.take_modbus_analyzer()` returns `Some(modbus)`. `modbus.findings()` are appended to `all_findings`. `modbus.summarize()` is pushed to `analyzer_summaries`. The `take_modbus_analyzer()` accessor consumes the `Option<ModbusAnalyzer>` from the dispatcher.
- **Test:** `test_modbus_findings_collected_post_finalize()` — run a pcap with write-class Modbus PDUs; assert findings in the output JSON include Modbus-attributed findings; assert Modbus appears in the `analyzers` summary array.

### AC-009 (traces to BC-2.14.023 + BC-2.14.025 — VP-004 classify-oracle Kani extension)
The VP-004 `classify_oracle` Kani harness is extended to include `DispatchTarget::Modbus` for port-502 flows. The harness verifies `verify_content_first_precedence_exhaustive` covers the port-502 branch: for all symbolic inputs where port==502 and content bytes do NOT match TLS/HTTP rules, `classify_oracle` returns `Modbus`.
- **Test:** Updated VP-004 Kani harness compiles with `#[cfg(kani)]` and passes the formal proof. `cargo test --all-targets` is green including the non-Kani unit-test analogue.

### AC-010 (traces to BC-2.14.023 postcondition P4 — `needs_reassembly` includes Modbus)
`let needs_reassembly = enable_http || enable_tls || enable_modbus;` — the `|| enable_modbus` term is present in `main.rs::run_analyze`. Without this term, `--modbus` alone would not start the reassembly engine, and Modbus PDUs would never be delivered to the analyzer (silent analysis regression per architecture-delta.md §5.2).
- **Test:** `test_modbus_alone_triggers_reassembly()` — verify that `--modbus` (without `--http`/`--tls`) causes the reassembly engine to initialize. Asserted by the end-to-end test AC-001 (a pcap with Modbus PDUs produces findings only if reassembly runs).

### AC-011 (traces to BC-2.14.024 — `on_flow_close` routing for Modbus)
`StreamDispatcher::on_flow_close` routes to `self.modbus.as_mut()?.on_flow_close(...)` when the flow's `DispatchTarget` is `Modbus`. The `ModbusAnalyzer` implements the `on_flow_close` method (from the `StreamAnalyzer` trait) to finalize per-flow summary stats.
- **Test:** `test_dispatcher_routes_flow_close_to_modbus()` — deliver data on a Modbus flow; then close the flow; assert flow-close reached the Modbus analyzer (e.g., `last_ts` finalized).

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `--modbus` flag | `src/cli.rs` | Effectful (clap arg parse) |
| `--modbus-write-burst-threshold` flag | `src/cli.rs` | Effectful (clap arg parse) |
| `--modbus-write-sustained-threshold` flag | `src/cli.rs` | Effectful (clap arg parse) |
| `enable_modbus = *modbus || *all` | `src/main.rs` | Effectful (orchestration) |
| `needs_reassembly = ... || enable_modbus` | `src/main.rs` | Effectful (orchestration) |
| `ModbusAnalyzer::new(burst, sustained)` construction | `src/main.rs` | Effectful (allocation) |
| `dispatcher.take_modbus_analyzer()` post-finalize | `src/main.rs` | Effectful (move out of Option) |
| `DispatchTarget::Modbus` (Rule 5) | `src/dispatcher.rs` | Pure (classifier) |
| `StreamDispatcher::on_data` Modbus arm | `src/dispatcher.rs` | Effectful (dispatch) |
| VP-004 Kani oracle extension | `src/dispatcher.rs` (`#[cfg(kani)]`) | Pure (proof) |

**Subsystem anchor justification:**
- SS-14 owns the `ModbusAnalyzer` wiring contract.
- SS-05 is touched because `StreamDispatcher` (the dispatcher module) gains a new `DispatchTarget::Modbus` arm and a `modbus: Option<ModbusAnalyzer>` field — SS-05 is the Content-First Protocol Dispatch subsystem per ARCH-INDEX.
- SS-12 is touched because `src/cli.rs` and `src/main.rs` (CLI entry point) gain new flags and orchestration logic — SS-12 is the CLI/Entry subsystem per ARCH-INDEX.

**Dependency anchor justification:** STORY-105 depends on STORY-104 because `ModbusAnalyzer` must have `on_data`, `on_flow_close`, `findings()`, and `summarize()` fully implemented before the dispatcher can wire it up and `main.rs` can collect findings from it. Wiring an incomplete analyzer would produce silent failures or compilation errors.

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | `--modbus` + `--no-reassemble` | Warning to stderr; `modbus_analyzer = None`; exit 0; no Modbus section in output |
| EC-002 | `--all` + `--no-reassemble` | Same as EC-001 plus HTTP/TLS also omitted; multiple warnings |
| EC-003 | Port-502 flow carrying TLS ClientHello bytes | Rule 1 fires first (content-first dispatch); `DispatchTarget::Tls` — NOT Modbus |
| EC-004 | Port-502 flow with HTTP GET bytes | Rule 2 fires first; `DispatchTarget::Http` |
| EC-005 | Port-502 non-Modbus binary (random bytes that fail 3-point validity gate) | Rule 5 fires → `DispatchTarget::Modbus`; `parse_errors` incremented; `is_non_modbus = true` after desync; no findings |
| EC-006 | Empty pcap with `--modbus` | Analyzer constructed; zero PDUs; `summarize()` returns all-zero stats; exit 0 |
| EC-007 | `--modbus-write-burst-threshold 0` | clap validation error or main.rs validation: `"must be >= 1"`; non-zero exit |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~4,500 |
| `src/cli.rs` (new flags) | ~2,500 |
| `src/main.rs` (4-step wiring) | ~4,000 |
| `src/dispatcher.rs` (Rule 5 + Modbus arm) | ~5,000 |
| BC files (3 BCs: BC-2.14.023, BC-2.14.024, BC-2.14.025) | ~6,000 |
| `src/analyzer/modbus.rs` (StreamAnalyzer trait impl) | ~3,000 |
| `tests/` (integration tests) | ~6,000 |
| Tool outputs overhead | ~1,500 |
| **Total** | **~32,500** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~16%** |

## Tasks (MANDATORY)

1. [ ] Write failing integration test for AC-001: `wirerust analyze empty.pcap --modbus` — fails because `--modbus` flag does not exist yet.
2. [ ] Write failing test for AC-006: port-502 classified to Modbus — fails because `DispatchTarget::Modbus` variant does not exist yet.
3. [ ] **Red Gate:** Confirm `cargo build` fails because `--modbus` flag and `DispatchTarget::Modbus` are absent.
4. [ ] Add `DispatchTarget::Modbus` variant to the enum in `src/dispatcher.rs`.
5. [ ] Add Rule 5 to `StreamDispatcher::classify`: after Rule 4 (port 80/8080 → HTTP), check `dst_port == 502 && self.modbus.is_some()` → `DispatchTarget::Modbus`.
6. [ ] Add `modbus: Option<ModbusAnalyzer>` field to `StreamDispatcher`. Update `StreamDispatcher::new` to accept `modbus: Option<ModbusAnalyzer>`.
7. [ ] Add Modbus arm to `StreamDispatcher::on_data`: when `DispatchTarget::Modbus`, call `self.modbus.as_mut().unwrap().on_data(flow_key, direction, data, offset, timestamp)`.
8. [ ] Add `take_modbus_analyzer(&mut self) -> Option<ModbusAnalyzer>` accessor to `StreamDispatcher`.
9. [ ] Add `--modbus: bool` to `src/cli.rs` `Commands::Analyze` using `#[arg(long, default_value_t = false)]`. Add `--modbus-write-burst-threshold: u32` with `#[arg(long, default_value_t = 20)]`. Add `--modbus-write-sustained-threshold: u32` with `#[arg(long, default_value_t = 10)]`.
10. [ ] Add 4-step wiring to `src/main.rs::run_analyze`:
    a. `let enable_modbus = *modbus || *all;`
    b. `let needs_reassembly = enable_http || enable_tls || enable_modbus;`
    c. `let modbus_analyzer = if enable_modbus && !skip_reassembly { Some(ModbusAnalyzer::new(*modbus_write_burst_threshold, *modbus_write_sustained_threshold)) } else { None };` (with stderr warning if `enable_modbus && skip_reassembly`).
    d. Post-finalize: `if let Some(modbus) = dispatcher.take_modbus_analyzer() { all_findings.extend(modbus.findings()); analyzer_summaries.push(modbus.summarize()); }`.
11. [ ] Add threshold validation (>=1) in `main.rs` before constructing `ModbusAnalyzer`. Emit clear error if 0.
12. [ ] Implement `StreamAnalyzer` trait for `ModbusAnalyzer` (or ensure `on_flow_close` is wired). `ModbusAnalyzer::on_flow_close` can be a no-op or finalize per-flow stats.
13. [ ] Extend VP-004 Kani harness in `src/dispatcher.rs` to cover `DispatchTarget::Modbus` for port-502 flows with non-TLS/HTTP content.
14. [ ] **Green Gate:** `cargo build --all-targets` exits 0. `cargo test --all-targets` green. AC-001 through AC-011 pass.
15. [ ] `cargo clippy --all-targets -- -D warnings` clean.
16. [ ] `cargo fmt --check` clean.

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-104 | `ModbusAnalyzer` has `findings()` returning `&[Finding]`, `summarize()` returning `AnalysisSummary`, `on_data(...)` with the full `StreamHandler` signature. | `take_modbus_analyzer()` uses `Option::take()` — not `clone()`. | |
| STORY-097 | `StreamHandler::on_data` has `timestamp: u32` as the 5th parameter. `StreamDispatcher` forwards it to all analyzers. The Modbus arm MUST also forward it unchanged. | The pattern: `self.modbus.as_mut().unwrap().on_data(flow_key, direction, data, offset, timestamp)` exactly mirrors the HTTP/TLS dispatch. | |
| STORY-031 (existing — E-3) | `StreamDispatcher::classify` rules and `DispatchTarget` enum live in `src/dispatcher.rs`. Rule ordering is CRITICAL: content first (Rules 1-2), then TLS/HTTP port (Rules 3-4). Rule 5 (Modbus, port 502) comes AFTER these four. | `DispatchTarget::Modbus` is the 5th rule; it must not supersede TLS or HTTP classification for port-502 flows. | VP-004 Kani harness already tests classify-oracle. Extension must cover the Modbus branch; do NOT invalidate existing VP-004 properties. |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Port-502 Rule 5 fires AFTER content rules (TLS ClientHello, HTTP GET/POST) and port rules (443/8443, 80/8080) | BC-2.14.025; BC-2.05.001 content-first invariant | AC-006 test; VP-004 Kani oracle |
| `--modbus` is default-off; no breaking change to existing invocations | BC-2.14.023 invariant 1 | AC-003 test |
| `needs_reassembly` MUST include `|| enable_modbus` | BC-2.14.023 postcondition P4 | AC-010 test; code review |
| `--modbus-write-burst-threshold` must be `>= 1`; reject 0 with error | BC-2.14.024 validation rule | AC-005 test |
| `--modbus-write-sustained-threshold` must be `>= 1`; reject 0 with error | BC-2.14.024 validation rule | AC-005 test |
| `StreamDispatcher::on_data` forwards `timestamp: u32` UNCHANGED to Modbus | BC-2.04.055 postcondition 2 (extended) | Code review; AC-007 |
| `take_modbus_analyzer` uses `Option::take()`, leaving `self.modbus = None` | Architecture pattern (prevent double-collection) | Code review |
| VP-004 Kani harness MUST be extended, not replaced; existing properties must still hold | BC-2.14.023 VP-004 extension obligation | AC-009; Kani green |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `clap` | workspace version (derive feature) | `#[arg(long)]` for new CLI flags with default values |
| No new external crates | — | All wiring logic uses stdlib + existing workspace dependencies |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/cli.rs` | **modify** | Add `modbus: bool`, `modbus_write_burst_threshold: u32`, `modbus_write_sustained_threshold: u32` fields |
| `src/main.rs` | **modify** | 4-step Modbus wiring: `enable_modbus`, `needs_reassembly`, analyzer construction, post-finalize collection |
| `src/dispatcher.rs` | **modify** | `DispatchTarget::Modbus` variant; `modbus: Option<ModbusAnalyzer>` field; Rule 5; `on_data` Modbus arm; `take_modbus_analyzer`; VP-004 Kani oracle extension |
| `src/analyzer/modbus.rs` | **modify** | `StreamAnalyzer` trait impl (if not already done in STORY-104); `on_flow_close` |
| `tests/` (integration tests) | **modify** or **create** | End-to-end tests for CLI wiring (AC-001 through AC-011) |

## Forbidden Dependencies

`src/dispatcher.rs` MUST NOT import `src/reporter/`. The dispatcher is L3 (Protocol Dispatch, SS-05); reporters are L4 (Output, SS-11). The dependency direction is SS-05 → SS-14 (dispatcher calls analyzer), SS-06/07/14 → SS-09 (analyzers emit Finding), SS-11 → SS-09 (reporters consume Finding). No upward dependency is permitted.
