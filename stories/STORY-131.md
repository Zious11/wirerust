---
document_type: story
story_id: STORY-131
title: "EtherNet/IP StreamDispatcher Integration, CLI Flags, and TCP Reassembly Wiring"
epic_id: E-20
wave: 58
points: 8
phase: f3
tdd_mode: strict
status: ready
feature_id: issue-316-enip-analyzer
github_issue: 316
subsystems: [SS-05, SS-17]
target_module: dispatcher
depends_on: []
behavioral_contracts:
  - BC-2.17.019
  - BC-2.17.020
  - BC-2.17.023
  - BC-2.17.026
verification_properties: []
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.019.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.020.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.023.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.026.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
input-hash: "56743d2"
---

# STORY-131: EtherNet/IP StreamDispatcher Integration, CLI Flags, and TCP Reassembly Wiring

## Narrative

**As a** security analyst running wirerust against industrial network captures,
**I want** the `--enip` CLI flag to enable EtherNet/IP analysis (including `--enip-write-burst-threshold`
and `--enip-error-burst-threshold` thresholds), the StreamDispatcher to route port-44818 TCP flows
to the `EnipAnalyzer`, and `take_enip_analyzer()` to transfer findings to the reporter,
**so that** EtherNet/IP traffic captured in pcaps is automatically detected and analyzed
when the user opts in via `--enip` or `--all`.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.17.019 | StreamDispatcher routes port-44818 TCP to EnipAnalyzer | Core dispatcher implementation |
| BC-2.17.020 | CLI `--enip` flag enables analyzer; threshold flags configure detection | CLI wiring implementation |
| BC-2.17.023 | `--enip-write-burst-threshold` configures T0836 write-burst threshold | CLI flag implementation |
| BC-2.17.026 | `--enip-error-burst-threshold` configures T0888 error-burst threshold | CLI flag implementation |

## Acceptance Criteria

### AC-131-001: StreamDispatcher routes port-44818 TCP flows to EnipAnalyzer
**Traces to:** BC-2.17.019 postconditions 1–3
- Given a TCP stream with destination port 44818 (or source port 44818 for bidirectional matching)
- When `StreamDispatcher::dispatch(flow_key, payload)` is called
- Then `DispatchTarget::Enip` is selected (Rule 7, after Rule 6 DNP3/20000)
- The dispatcher has an `enip_analyzer: Option<EnipAnalyzer>` field set when `--enip` is active
- Port 44818 routing appears AFTER port 20000 (DNP3) in priority order — flows matching both ports use DNP3 (port 20000 takes precedence as Rule 6)
- Non-44818 flows are NOT routed to the ENIP analyzer
- **PC-2 wiring guarantee:** routing correctness is observable via `EnipAnalyzer.bytes_received` — after feeding a port-44818 flow through `StreamDispatcher::on_data`, `bytes_received > 0`; after feeding a non-44818 flow, `bytes_received == 0`. The dispatcher routes data into a minimal `on_data` that increments `bytes_received`; full CIP frame-walk is deferred to STORY-132.
- **BC-2.17.019 EC-007 guard:** if `dispatcher.enip_analyzer` is `None` and a port-44818 flow arrives, the early-exit guard fires and the call is a no-op (no panic, no routing attempt). See `test_dispatcher_no_enip_analyzer_port_44818_is_noop` below.
- **Test:** `tests/enip_analyzer_tests.rs::dispatch::test_dispatcher_routes_port_44818`
- **Test:** `tests/enip_analyzer_tests.rs::dispatch::test_dispatcher_does_not_route_other_ports`
- **Test:** `tests/enip_analyzer_tests.rs::dispatch::test_dispatcher_rule_order_dnp3_before_enip`
- **Test (EC-007):** `tests/enip_analyzer_tests.rs::dispatch::test_dispatcher_no_enip_analyzer_port_44818_is_noop`

### AC-131-002: `take_enip_analyzer()` transfers EnipAnalyzer findings to caller
**Traces to:** BC-2.17.019 postcondition 4
- `StreamDispatcher::take_enip_analyzer() -> Option<EnipAnalyzer>` removes and returns the ENIP analyzer
- After `take_enip_analyzer()` is called, `dispatcher.enip_analyzer` is `None`
- The returned analyzer carries all accumulated findings and summary data
- Mirrors `take_dnp3_analyzer()` / `take_modbus_analyzer()` pattern
- **Test:** `tests/enip_analyzer_tests.rs::dispatch::test_take_enip_analyzer_transfers_ownership`
- **Test:** `tests/enip_analyzer_tests.rs::dispatch::test_take_enip_analyzer_returns_none_when_not_set`

### AC-131-003: CLI `--enip` flag enables EnipAnalyzer construction and wiring
**Traces to:** BC-2.17.020 postconditions 1, 3, 4
- When `wirerust analyze pcap --enip` is invoked (with TCP reassembly active)
- `EnipAnalyzer` is constructed with `enip_write_burst_threshold = 50` (default) and `enip_error_burst_threshold = 5` (default)
- `EnipAnalyzer` is pushed to `needs_reassembly` list
- `EnipAnalyzer` is wired to `StreamDispatcher`
- When `--enip` is NOT set and `--all` is NOT set: no `EnipAnalyzer` is constructed; no port-44818 routing
- **Test:** `tests/enip_analyzer_tests.rs::dispatch::test_cli_enip_flag_constructs_analyzer`
- **Test:** `tests/enip_analyzer_tests.rs::dispatch::test_cli_no_enip_flag_no_analyzer`
- **Test:** `tests/enip_analyzer_tests.rs::dispatch::test_cli_all_flag_includes_enip`

### AC-131-004: Missing TCP reassembly with `--enip` emits WARNING and disables ENIP
**Traces to:** BC-2.17.020 postcondition 2
- When `--enip` is set but `--tcp-reassembly` is not active (and `--all` is not set)
- A WARNING is emitted to stderr: `"--enip requires TCP reassembly; ENIP analysis disabled"`
- `EnipAnalyzer` is NOT constructed
- No ENIP findings are emitted for the run
- Mirrors `--modbus`/`--dnp3` reassembly-guard pattern
- **Test:** `tests/enip_analyzer_tests.rs::dispatch::test_enip_without_reassembly_warns_and_disables`

### AC-131-005: `--enip-write-burst-threshold` sets T0836 burst threshold on EnipAnalyzer
**Traces to:** BC-2.17.023 postconditions 1–4
- When `--enip-write-burst-threshold N` is provided: `EnipAnalyzer.enip_write_burst_threshold = N`
- When not provided: default is 50 (`OA-001 RESOLVED=50`)
- Threshold is u32; values near `u32::MAX` are accepted (operator responsibility)
- Does NOT affect error-burst, T0858, T0816, T0846, or T0814 thresholds
- **Test:** `tests/enip_analyzer_tests.rs::dispatch::test_write_burst_threshold_custom`
- **Test:** `tests/enip_analyzer_tests.rs::dispatch::test_write_burst_threshold_default`

### AC-131-006: `--enip-error-burst-threshold` sets T0888 error-burst threshold on EnipAnalyzer
**Traces to:** BC-2.17.026 postconditions 1–4
- When `--enip-error-burst-threshold M` is provided: `EnipAnalyzer.enip_error_burst_threshold = M`
- When not provided: default is 5
- Strict `>` semantics: with default 5, the 6th CIP error in 10s fires T0888 Pattern B
- Does NOT affect write-burst, T0858, T0816, T0846, or T0814 thresholds
- **Test:** `tests/enip_analyzer_tests.rs::dispatch::test_error_burst_threshold_custom`
- **Test:** `tests/enip_analyzer_tests.rs::dispatch::test_error_burst_threshold_default`
- **Test:** `tests/enip_analyzer_tests.rs::dispatch::test_error_burst_threshold_zero_semantics`

## Architecture Mapping

| Component | Location | Role |
|-----------|----------|------|
| `DispatchTarget::Enip` | `src/dispatcher.rs` | New enum variant; Rule 7 at port 44818 |
| `StreamDispatcher.enip_analyzer` | `src/dispatcher.rs` | `Option<EnipAnalyzer>` field |
| `take_enip_analyzer()` | `src/dispatcher.rs` | Ownership transfer; mirrors `take_dnp3_analyzer()` |
| `Commands::Analyze.enip` | `src/cli.rs` | `bool`, default false |
| `Commands::Analyze.enip_write_burst_threshold` | `src/cli.rs` | `u32`, default 50 |
| `Commands::Analyze.enip_error_burst_threshold` | `src/cli.rs` | `u32`, default 5 |
| ENIP construction logic | `src/main.rs` | Constructs and wires `EnipAnalyzer`; reassembly guard |
| `EnipAnalyzer.enip_write_burst_threshold` | `src/analyzer/enip.rs` | `u32` field, populated from CLI |
| `EnipAnalyzer.enip_error_burst_threshold` | `src/analyzer/enip.rs` | `u32` field, populated from CLI |
| `EnipAnalyzer.bytes_received` | `src/analyzer/enip.rs` | `u64` field, incremented by `on_data`; evidences PC-2 routing (full frame-walk in STORY-132) |

**Dispatch Rule Order (ADR-010 Decision 3):**
```
Rule 1: ARP/IPv4/etc (existing)
...
Rule 6: TCP port 20000 → DispatchTarget::Dnp3
Rule 7: TCP port 44818 → DispatchTarget::Enip    ← NEW this story
Rule 8: default → DispatchTarget::Unknown
```

Rule 7 is appended AFTER Rule 6 (DNP3). The classifier must check port 20000 before port 44818.

## VP Oracle Obligation (VP-004)

VP-004 requires that the `classify_oracle` in `#[cfg(kani)] mod kani_proofs` in `src/dispatcher.rs` gains a port-44818 arm:

```rust
// In kani_proofs::classify_oracle (existing function):
44818 => DispatchTarget::Enip,
```

This ensures the Kani oracle for dispatcher correctness covers the new ENIP routing rule. Without this arm, the VP-004 oracle-completeness proof would be incomplete for port 44818. The implementer must add this arm alongside the Rule 7 addition in the main dispatch logic.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TCP flow dst_port=44818 | → `DispatchTarget::Enip` |
| EC-002 | TCP flow dst_port=20000 | → `DispatchTarget::Dnp3` (Rule 6 fires before Rule 7) |
| EC-003 | TCP flow dst_port=502 | → `DispatchTarget::Modbus` (existing rule) |
| EC-004 | `--enip` without `--reassembly` | WARNING; no analyzer; no findings |
| EC-005 | `--all` with reassembly | ENIP enabled; thresholds at defaults (write=50, error=5) |
| EC-006 | `--enip-write-burst-threshold 0` | `enip_write_burst_threshold = 0`; first write triggers T0836 (1 > 0) |
| EC-007 | `--enip-error-burst-threshold 0` | `enip_error_burst_threshold = 0`; first error triggers T0888 Pattern B (1 > 0) |
| EC-008 | `take_enip_analyzer()` called twice | First call returns `Some(...)`; second call returns `None` |
| EC-009 | No `--enip` flag | `dispatcher.enip_analyzer = None`; port 44818 flows fall through to default |

## Tasks

- [ ] Add `DispatchTarget::Enip` variant to `src/dispatcher.rs` enum
- [ ] Add `enip_analyzer: Option<EnipAnalyzer>` field to `StreamDispatcher` struct
- [ ] Implement Rule 7 in `StreamDispatcher::dispatch()`: TCP port 44818 → `DispatchTarget::Enip` (after Rule 6 for port 20000)
- [ ] Implement `take_enip_analyzer(&mut self) -> Option<EnipAnalyzer>` on `StreamDispatcher`
- [ ] Update `kani_proofs::classify_oracle` in `src/dispatcher.rs` to add `44818 => DispatchTarget::Enip` arm (VP-004 oracle obligation)
- [ ] Add to `src/cli.rs` `Commands::Analyze`: `enip: bool` (default false), `enip_write_burst_threshold: u32` (default 50), `enip_error_burst_threshold: u32` (default 5); include `--enip` in `--all` expansion
- [ ] Add `EnipAnalyzer::new(write_burst_threshold: u32, error_burst_threshold: u32) -> EnipAnalyzer` constructor to `src/analyzer/enip.rs` (stub: empty struct fields; later stories populate detection logic)
- [ ] Add fields `enip_write_burst_threshold: u32` and `enip_error_burst_threshold: u32` to `EnipAnalyzer` struct in `src/analyzer/enip.rs`
- [ ] Update `src/main.rs` analyze flow: if `args.enip && !has_reassembly { eprintln!("--enip requires TCP reassembly; ENIP analysis disabled"); } else if args.enip { let analyzer = EnipAnalyzer::new(args.enip_write_burst_threshold, args.enip_error_burst_threshold); dispatcher.set_enip_analyzer(analyzer); needs_reassembly.push(...); }`
- [ ] Add `mod dispatch { ... }` test wrapper to `tests/enip_analyzer_tests.rs` with all AC-131 tests (15 tests including `test_dispatcher_no_enip_analyzer_port_44818_is_noop`)
- [ ] Run `cargo check` — zero errors
- [ ] Run `cargo test enip` — all 15 new tests pass
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings

## Test Plan

**Test file:** `tests/enip_analyzer_tests.rs`
**Test module:** `mod dispatch { ... }`

```
dispatch::test_dispatcher_routes_port_44818
dispatch::test_dispatcher_does_not_route_other_ports
dispatch::test_dispatcher_rule_order_dnp3_before_enip
dispatch::test_take_enip_analyzer_transfers_ownership
dispatch::test_take_enip_analyzer_returns_none_when_not_set
dispatch::test_cli_enip_flag_constructs_analyzer
dispatch::test_cli_no_enip_flag_no_analyzer
dispatch::test_cli_all_flag_includes_enip
dispatch::test_enip_without_reassembly_warns_and_disables
dispatch::test_write_burst_threshold_custom
dispatch::test_write_burst_threshold_default
dispatch::test_error_burst_threshold_custom
dispatch::test_error_burst_threshold_default
dispatch::test_error_burst_threshold_zero_semantics
dispatch::test_dispatcher_no_enip_analyzer_port_44818_is_noop
```

**Total: 15 tests**

## Previous Story Intelligence

- **STORY-131 does NOT depend on STORY-130** in the strict implementation sense — it adds skeleton infrastructure (`EnipAnalyzer` struct with threshold fields, `DispatchTarget::Enip`, CLI flags) but the pure-core parse functions from STORY-130 are not called here. However, STORY-130 must define the `EnipAnalyzer` type (or at minimum a stub) before `src/dispatcher.rs` can import it.
- **In practice:** STORY-130 and STORY-131 share Wave 58 but STORY-130's `EnipAnalyzer` stub must exist before STORY-131 can compile. The implementer should ensure STORY-130 (which creates `src/analyzer/enip.rs` with the struct) is merged before STORY-131's dispatcher changes.
- **Reference pattern:** STORY-110 (DNP3 dispatcher integration) — use the same `take_dnp3_analyzer()` signature shape for `take_enip_analyzer()`. The `set_enip_analyzer()` / `enip_analyzer()` accessor pair mirrors the DNP3 pattern in `dispatcher.rs`.
- **VP-004 oracle:** The Kani oracle must be updated. See STORY-110's VP-004 section for the exact oracle structure to replicate for the ENIP arm.

## Architecture Compliance Rules

From ADR-010 Decision 3 (stream dispatch) and Decision 9 (CLI pattern):

1. **Rule 7 must come after Rule 6 (ADR-010 Decision 3):** DNP3 (port 20000) is Rule 6; ENIP (port 44818) is Rule 7. The order matters: a hypothetical flow on both ports (impossible in practice but possible in tests) must be routed by Rule 6. Never swap the order.
2. **Reassembly guard is mandatory (ADR-010 Decision 9, BC-2.17.020):** The WARNING-and-disable pattern (same as Modbus/DNP3) must be implemented. Never silently construct an `EnipAnalyzer` without TCP reassembly — silent empty results confuse operators.
3. **`--all` expands to include `--enip` (BC-2.17.020 Invariant 4):** The `--all` flag must set `enip = true` in the same code path that sets `modbus = true` and `dnp3 = true`. Verify by tracing the `--all` expansion in `src/cli.rs` or `src/main.rs`.
4. **`take_enip_analyzer()` pattern (ADR-010 Decision 9):** The method takes `&mut self`, returns `Option<EnipAnalyzer>`, and sets `self.enip_analyzer = None` after moving out. It must not clone the analyzer (findings must transfer ownership, not be duplicated).
5. **VP-004 oracle completeness:** Every dispatched port that has a production rule MUST have a corresponding arm in `kani_proofs::classify_oracle`. Omitting port 44818 from the oracle while adding it to production code creates a gap in the VP-004 proof.
6. **Default threshold values are load-bearing (BC-2.17.023 Invariant 1, BC-2.17.026 Invariant 1):** Default write-burst = 50 and error-burst = 5. These are `clap` default values — use `#[arg(default_value_t = 50)]` and `#[arg(default_value_t = 5)]` respectively. Do not use magic numbers inline in `main.rs`.

## Library & Framework Requirements

- **clap ≥ 4.x** (already present in project): Use `#[arg(long, default_value_t = 50)]` for threshold flags. Match the exact pattern used by existing `--dnp3-*` or `--modbus-*` threshold flags in `src/cli.rs`.
- **No `log` crate** (project has no `log` dependency): emit the reassembly guard warning via `eprintln!("--enip requires TCP reassembly; ENIP analysis disabled")` to stderr — matching the identical pattern used by the `--modbus` and `--dnp3` guards in `src/main.rs`. Do NOT introduce a `log` crate dependency or use `warn!`.
- No new crate dependencies.

## File Structure Requirements

**Files to create:**
- (none — `src/analyzer/enip.rs` and `tests/enip_analyzer_tests.rs` created by STORY-130)

**Files to modify:**
- `src/dispatcher.rs` — add `DispatchTarget::Enip`, `enip_analyzer` field, Rule 7, `take_enip_analyzer()`, VP-004 oracle arm
- `src/cli.rs` — add `enip: bool`, `enip_write_burst_threshold: u32`, `enip_error_burst_threshold: u32` to `Commands::Analyze`; add `enip` to `--all` expansion
- `src/main.rs` — add ENIP construction, reassembly guard, `take_enip_analyzer()` call
- `src/analyzer/enip.rs` — add `EnipAnalyzer` struct with `enip_write_burst_threshold` and `enip_error_burst_threshold` fields; add `EnipAnalyzer::new(...)` constructor
- `tests/enip_analyzer_tests.rs` — add `mod dispatch { ... }` block

**Files NOT touched by this story:**
- `src/mitre.rs` — STORY-133 adds new MITRE entries
- `src/analyzer/enip.rs` detection logic — STORY-132/134/135/136/137 add CPF/CIP parse and detection

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/dispatcher.rs` changes | ~200 |
| `src/cli.rs` changes | ~150 |
| `src/main.rs` changes | ~150 |
| `src/analyzer/enip.rs` additions (struct + constructor) | ~100 |
| `tests/enip_analyzer_tests.rs` dispatch mod (15 tests) | ~580 |
| **Total** | **~1,180** |

## Dependency Rationale

STORY-131 is Wave 58 (parallel with STORY-130). It requires STORY-130's `EnipAnalyzer` stub to compile but not any detection logic. The CLI/dispatcher infrastructure must be ready before Wave 59 detection stories (STORY-132, STORY-133) can be wired end-to-end.
