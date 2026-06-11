---
document_type: story
story_id: STORY-110
epic_id: E-15
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-06-10T00:00:00Z
phase: 3
points: 8
priority: P0
depends_on: [STORY-109]
blocks: []
behavioral_contracts:
  - BC-2.15.017
  - BC-2.15.021
verification_properties:
  - VP-004
tdd_mode: strict
target_module: dispatcher
subsystems: [SS-05, SS-15]
wave: 39
estimated_days: 3
feature_id: issue-008-dnp3-analyzer
github_issue: 8
# BC status: BC-2.15.017 v1.0, BC-2.15.021 v1.0; authored 2026-06-10
# VP-004 oracle obligation: classify_oracle gains port-20000 arm (ADR-007 Decision 1)
# VP-007 atomic-update obligation: SEEDED 21→23 / EMITTED 13→15 verified here
inputs:
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.017.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.021.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
  - .factory/specs/verification-properties/vp-023-dnp3-parse-safety.md
input-hash: "a9cdfb5"
---

# STORY-110: DNP3 Dispatcher Integration + CLI Flag — VP-004 Oracle + VP-007 Atomic-Update

## Narrative

- **As a** ICS/OT security analyst using wirerust
- **I want** wirerust to automatically route TCP flows on port 20000 to the DNP3 analyzer (as Rule 6 in the StreamDispatcher, after all content rules and after Modbus port 502), and to expose a `--dnp3-direct-operate-threshold` CLI flag to tune the unauthorized-control burst detection threshold
- **So that** DNP3 traffic in pcap files is automatically analyzed without additional flags, the content-first dispatch precedence invariant (INV-2) is formally verified via VP-004, and the threshold can be tuned for quiet OT environments

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.15.017 | --dnp3-direct-operate-threshold CLI Flag Controls Control-Command Detection Window |
| BC-2.15.021 | Port-20000 Flow Dispatched to Dnp3Analyzer (DispatchTarget::Dnp3, Rule 6) |

## VP-004 Oracle Obligation (LANDS HERE)

BC-2.15.021 carries a Kani obligation: the `classify_oracle` function in `src/dispatcher.rs`'s `#[cfg(kani)] mod kani_proofs` MUST gain the `port-20000 → DispatchTarget::Dnp3` arm immediately after the `port-502 → DispatchTarget::Modbus` arm. The existing Kani proof `verify_content_first_precedence_exhaustive` asserts `got == want` for all 65536² port combinations. Oracle and production divergence causes this proof to fail at F6.

The implementer MUST update `classify_oracle` in the same commit as the `classify()` production change. No deferred oracle update is allowed.

## VP-007 Atomic-Update Obligation (SEEDED 21→23 / EMITTED 13→15)

After STORY-109 (which adds T1691.001 and T0827 to SEEDED and EMITTED), the final catalog counts are:
- **SEEDED**: 23 technique IDs (was 21 before F3; T1691.001 and T0827 added in STORY-109)
- **EMITTED**: 15 technique IDs (was 13 before F3; T1691.001 and T0827 first emitted in STORY-109)
- **CATALOGUE-ONLY**: 8 technique IDs (unchanged)

The implementer MUST verify in this story that `SEEDED_TECHNIQUE_ID_COUNT == 23` (module-level const) and `SEEDED_TECHNIQUE_IDS.len() == 23` after all E-15 stories are integrated. The Kani-local `kani_proofs::EMITTED_IDS` slice must have length 15. The catalogue-only count (8) is derived: 23 − 15 == 8. There is no named `EMITTED_TECHNIQUE_IDS` or `CATALOGUE_ONLY_TECHNIQUE_IDS` const in `src/mitre.rs`; the real symbols are `SEEDED_TECHNIQUE_IDS: &[&str]`, `SEEDED_TECHNIQUE_ID_COUNT: usize`, and `kani_proofs::EMITTED_IDS: &[&str]`. If any count is wrong, the VP-007 drift-guard test (`test_technique_catalog_integrity`) will fail at F6. This is the final integration story for E-15 — it is the correct place to confirm the atomic-update obligation was fully satisfied.

## Acceptance Criteria

### AC-001 (traces to BC-2.15.021 postcondition 1 — Rule 6 dispatch)
`StreamDispatcher::classify(ports, data)` returns `DispatchTarget::Dnp3` when `ports` contains 20000 AND Rules 1–5 all returned false (no TLS content, no HTTP content, no TLS port, no HTTP port, no Modbus port). The rule is inserted as **Rule 6** (after Rule 5, Modbus port 502). `DispatchTarget::None` becomes Rule 7.
- **Test:** `test_port_20000_dispatches_to_dnp3()`

### AC-002 (traces to BC-2.15.021 postconditions 5/6 — content-first precedence preserved)
A TLS ClientHello (`data[0]==0x16 && data[1]==0x03`) on port 20000 returns `DispatchTarget::Tls` (Rule 1), NOT `DispatchTarget::Dnp3`. An HTTP GET prefix on port 20000 returns `DispatchTarget::Http` (Rule 2), NOT `DispatchTarget::Dnp3`. Rule 6 is never reached for these content-matched flows.
- **Test:** `test_tls_on_port_20000_routes_to_tls()`, `test_http_on_port_20000_routes_to_http()`

### AC-003 (traces to BC-2.15.021 invariant 4 — early-exit guard includes dnp3)
The early-exit guard `if self.http.is_none() && self.tls.is_none() && self.modbus.is_none()` MUST be extended to `&& self.dnp3.is_none()`. Without this, `on_data` silently drops data when only a DNP3 analyzer is active.
- **Test:** `test_early_exit_guard_includes_dnp3()`

### AC-004 (traces to BC-2.15.021 invariant 5 — take_dnp3_analyzer)
Post-finalization, `dispatcher.take_dnp3_analyzer()` moves the `Dnp3Analyzer` out for result collection, mirroring `take_modbus_analyzer()`.
- **Test:** `test_take_dnp3_analyzer_moves_out()`

### AC-005 (traces to BC-2.15.021 VP-004 oracle obligation)
`classify_oracle` in `#[cfg(kani)] mod kani_proofs` in `src/dispatcher.rs` gains the `port-20000 → Dnp3` arm immediately after the `port-502 → Modbus` arm. `verify_content_first_precedence_exhaustive` Kani proof passes (VERIFICATION:- SUCCESSFUL) with the new arm.
- **Kani:** `cargo kani --harness verify_content_first_precedence_exhaustive`
- **Test:** Run Kani proof after oracle update; assert passes.

### AC-006 (traces to BC-2.15.017 postconditions 1/2 — CLI flag wires to threshold)
`wirerust analyze --dnp3-direct-operate-threshold <N>` parses `N` as `u32` and sets `Dnp3Analyzer.direct_operate_threshold = N`. When flag omitted, default `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT = 10` applies.
- **Test:** `test_cli_flag_dnp3_direct_operate_threshold_parsed()`

### AC-007 (traces to BC-2.15.017 postcondition 3/4 — edge values)
`--dnp3-direct-operate-threshold 0` causes T1692.001 to fire on the very first Control-class FC (count=1 > 0). `--dnp3-direct-operate-threshold 4294967295` causes T1692.001 never to fire (counter cannot exceed u32::MAX).
- **Test:** `test_threshold_0_fires_immediately()`, `test_threshold_max_never_fires()`

### AC-008 (traces to BC-2.15.017 postcondition 5 — threshold echoed in finding summary)
The flag value is echoed in the T1692.001 finding summary string: `"(threshold {threshold})"`. When the flag is omitted, the default (10) appears in the summary.
- **Test:** `test_threshold_echoed_in_t1692_summary()`

### AC-009 (traces to BC-2.15.021 invariant 1/2 — Rule 6 ordering; no stolen flows)
Rule 5 (port 502, Modbus) fires BEFORE Rule 6 (port 20000, DNP3). A flow with both port 502 and 20000 in its port set routes to `DispatchTarget::Modbus` (Rule 5 wins). `DispatchTarget::None` (formerly Rule 6) is now Rule 7.
- **Test:** `test_port_502_and_20000_routes_to_modbus()`

### AC-010 (traces to BC-2.15.021 — VP-007 atomic-update: SEEDED=23, EMITTED=15)
After full E-15 integration (STORY-106 through STORY-110), the VP-007 invariants hold:
- `SEEDED_TECHNIQUE_ID_COUNT == 23` (module-level const in `src/mitre.rs`; also verified via `SEEDED_TECHNIQUE_IDS.len() == 23` in the drift-guard test)
- `kani_proofs::EMITTED_IDS.len() == 15` (Kani-local const in `src/mitre.rs` under `#[cfg(kani)] mod kani_proofs`)
- Catalogue-only count is a DERIVED assertion: `SEEDED_TECHNIQUE_ID_COUNT - kani_proofs::EMITTED_IDS.len() == 8` (23 − 15 == 8). There is no named `CATALOGUE_ONLY_TECHNIQUE_IDS` const in the codebase; the check is purely arithmetic.
- T1691.001 and T0827 are present in `SEEDED_TECHNIQUE_IDS` and in `kani_proofs::EMITTED_IDS` (added in STORY-109).
- **Test:** `test_vp007_seeded_23_emitted_15()` — assert `SEEDED_TECHNIQUE_IDS.len() == 23` (via the existing mitre drift-guard test `test_technique_catalog_integrity`), `SEEDED_TECHNIQUE_ID_COUNT == 23`, and the Kani `EMITTED_IDS` slice length == 15. Verify arithmetic: 23 − 15 == 8.

### AC-011 (traces to BC-2.15.021 — VP-023 status propagation at F6 gate)
When the four Kani proofs in STORY-106 run green at the Wave 39 F6 gate, VP-023 status MUST be propagated from `draft` to `verified` and VP-INDEX bumped: `verified` count 22 → 23, `draft` count 1 → 0. This mirrors the VP-021 and VP-022 lock pattern applied at prior F6 gates. The STORY-110 implementer records this propagation obligation so the F6 state-manager step can execute it.
- **Task:** After `cargo kani --harness verify_parse_dnp3_dl_header_safety` + `verify_classify_dnp3_fc_total` + `verify_is_valid_dnp3_frame_gate` + `verify_compute_dnp3_frame_len` all report `VERIFICATION:- SUCCESSFUL`, open a factory-artifacts commit that sets `vp-023-dnp3-parse-safety.md` status field to `verified` and updates VP-INDEX verified/draft counts.
- **Note:** This task does NOT block story delivery — the Kani proofs run at F6, not at unit-test time.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `DispatchTarget::Dnp3` variant | `src/dispatcher.rs` | Data type (NEW variant) |
| `fn classify()` Rule 6 arm | `src/dispatcher.rs` | Effectful shell |
| `StreamDispatcher.dnp3: Option<Dnp3Analyzer>` | `src/dispatcher.rs` | NEW field |
| `fn take_dnp3_analyzer()` | `src/dispatcher.rs` | Effectful (moves out) |
| Early-exit guard extended | `src/dispatcher.rs` | Guard fix |
| `classify_oracle` Rule 6 arm | `src/dispatcher.rs` (kani mod) | VP-004 oracle (Kani) |
| `Commands::Analyze.dnp3_direct_operate_threshold: u32` | `src/cli.rs` | CLI arg (NEW field) |
| `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT: u32 = 10` | `src/analyzer/dnp3.rs` or `src/cli.rs` | Constant |
| `Dnp3Analyzer.direct_operate_threshold: u32` | `src/analyzer/dnp3.rs` | Config field |

Architecture section references: `architecture/module-decomposition.md` (SS-05 dispatcher, SS-12 CLI), `architecture/dependency-graph.md` (SS-15 depends on SS-05 for dispatch entry).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Port 20000, no content data | Rule 6 fires: `DispatchTarget::Dnp3` |
| EC-002 | Port 20000, TLS ClientHello | Rule 1 fires: `DispatchTarget::Tls` |
| EC-003 | Port 20000, HTTP GET | Rule 2 fires: `DispatchTarget::Http` |
| EC-004 | Port 502 (Modbus) | Rule 5 fires: `DispatchTarget::Modbus` |
| EC-005 | Port 12345 (unknown) | Rule 7 fires: `DispatchTarget::None` |
| EC-006 | Port 502 AND port 20000 | Rule 5 fires: `DispatchTarget::Modbus` (Rule 5 before Rule 6) |
| EC-007 | DNP3 analyzer present, HTTP/TLS absent | Early-exit guard passes (`dnp3.is_some()`); `on_data` proceeds |
| EC-008 | `--dnp3-direct-operate-threshold` omitted | Default 10 applied |

## Tasks

1. **Add `DispatchTarget::Dnp3` variant** to the `DispatchTarget` enum in `src/dispatcher.rs`.
2. **Add `StreamDispatcher.dnp3: Option<Dnp3Analyzer>` field** and `take_dnp3_analyzer()` method.
3. **Extend `classify()` with Rule 6** — `if ports.contains(&20000) { return DispatchTarget::Dnp3; }` placed after the Rule 5 port-502 arm.
4. **Extend early-exit guard** — add `&& self.dnp3.is_none()` (or refactor to per-arm `if let` checks).
5. **Update `classify_oracle` in `#[cfg(kani)] mod kani_proofs`** — add port-20000 arm after port-502 arm. Run `cargo kani --harness verify_content_first_precedence_exhaustive` and confirm SUCCESSFUL.
6. **Add `--dnp3-direct-operate-threshold` flag** to `Commands::Analyze` in `src/cli.rs` — `#[arg(long, default_value_t = DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT)]`.
7. **Wire threshold to `Dnp3Analyzer`** in `run_analyze` orchestration — pass parsed threshold value to `Dnp3Analyzer::new(threshold)`.
8. **Verify SEEDED=23, EMITTED=15** catalog counts (VP-007 check).
9. **Unit tests** for AC-001 through AC-010.
10. **Integration test** — small DNP3 PCAP (or synthetic byte sequence) exercising end-to-end dispatch → parse → detection → output.
11. **VP-023 status propagation (F6 gate obligation)** — after all four Kani proofs in STORY-106 run green at Wave 39 F6, update `vp-023-dnp3-parse-safety.md` status to `verified` and bump VP-INDEX verified 22→23, draft 1→0. Mirror the VP-021/VP-022 lock pattern.

## Test Plan

| AC | Test Type | Notes |
|----|-----------|-------|
| AC-001 | Unit | Port 20000 dispatches to Dnp3 |
| AC-002 | Unit | TLS/HTTP content-first on port 20000 |
| AC-003 | Unit | Early-exit guard extended |
| AC-004 | Unit | take_dnp3_analyzer() moves out |
| AC-005 | Kani | VP-004 oracle; verify_content_first_precedence_exhaustive |
| AC-006 | Unit | CLI flag parsed; default 10 |
| AC-007 | Unit | Threshold 0 fires immediately; threshold MAX never fires |
| AC-008 | Unit | Threshold echoed in T1692.001 summary |
| AC-009 | Unit | Port 502+20000 → Modbus (Rule 5 wins) |
| AC-010 | Unit | VP-007 catalog counts SEEDED=23, EMITTED=15 |
| AC-011 | F6 gate (manual) | VP-023 status draft→verified; VP-INDEX bump |

## Previous Story Intelligence

STORY-105 (Modbus Dispatcher Integration, E-14) is the direct structural precedent:
- STORY-105 added `DispatchTarget::Modbus`, `StreamDispatcher.modbus: Option<ModbusAnalyzer>`, Rule 5 in `classify()`, and the `--modbus-write-burst-threshold` CLI flag.
- The DNP3 story mirrors this exactly: `DispatchTarget::Dnp3` is the fifth variant (after None→Rule7), Rule 6 is the new port rule, and the early-exit guard extension is the same pattern.
- **Critical lesson from STORY-105**: the VP-004 oracle update (classifier_oracle in `#[cfg(kani)] mod kani_proofs`) was missed in the initial Modbus integration and caused the Kani proof to diverge at F6. For DNP3, the oracle update MUST happen in the same commit as the `classify()` production change — no exceptions.
- **New in DNP3 vs Modbus**: the `--dnp3-direct-operate-threshold` flag has a `u32` type (not a simple bool); ensure clap parses it correctly with `default_value_t`.

## Architecture Compliance Rules

Derived from ADR-007 Decision 1 and architecture/module-decomposition.md (SS-05):
1. **Rule 6 is placed AFTER Rule 5** (Modbus/502) — port order is fixed by ADR-007 Rule Table. No existing flow classification can be stolen by the new rule.
2. **`classify_oracle` updated in same commit as `classify()`** — VP-004 oracle obligation is non-negotiable. Divergence causes Kani failure at F6.
3. **Early-exit guard must include `self.dnp3.is_none()`** — silent data drop when only DNP3 analyzer is active is a functional bug.
4. **`DispatchTarget::Dnp3` is the fifth variant** — the enum must handle the Dnp3 arm in all match arms across the codebase (dispatcher, reporters, etc.). Compiler will catch missing arms.
5. **Forbidden dependencies**: `src/dispatcher.rs` gains a dependency on `src/analyzer/dnp3.rs` (correct); it MUST NOT gain a dependency on `src/analyzer/modbus.rs` through the DNP3 path.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `clap` | same version as existing CLI (derived from Cargo.toml) | `#[arg(long, default_value_t)]` for threshold flag |
| `src/analyzer/dnp3.rs` | same crate | `Dnp3Analyzer` struct |
| `kani` | via cargo-kani | VP-004 oracle harness verification |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/dispatcher.rs` | Modify | Add `DispatchTarget::Dnp3`; Rule 6 in `classify()`; new field; early-exit guard; `take_dnp3_analyzer()`; oracle arm in `#[cfg(kani)]` mod |
| `src/cli.rs` | Modify | Add `--dnp3-direct-operate-threshold` to `Commands::Analyze` |
| `src/main.rs` or `src/run.rs` | Modify | Wire threshold from parsed CLI args to `Dnp3Analyzer::new()` |
| `tests/dispatcher_integration_test.rs` OR inline | Create/expand | AC-001..AC-010 unit + Kani |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~3,500 |
| BC-2.15.017 + BC-2.15.021 | ~6,000 |
| ADR-007 (Decision 1 rule table + oracle obligation) | ~3,000 |
| STORY-105 (Modbus dispatcher precedent) | ~3,000 |
| Existing `src/dispatcher.rs` | ~3,500 |
| Existing `src/cli.rs` | ~2,000 |
| STORY-109 (prior story, full detection surface) | ~3,000 |
| Tool outputs (cargo kani, cargo test) | ~2,000 |
| **Total estimated** | **~26,000** |

Well within 20-30% of agent context window.

## Dependency Rationale

- `depends_on: [STORY-109]` — STORY-109 completes the full `Dnp3Analyzer` detection surface including T1691.001, T0827, all anomaly findings, and the mitre.rs seeding. The dispatcher story wires this complete analyzer into the `StreamDispatcher`; it cannot do so before the analyzer is complete. Also, VP-007 catalog verification (SEEDED=23, EMITTED=15) requires T1691.001 and T0827 to be seeded (STORY-109).
- `blocks: []` — STORY-110 is the last story in E-15. No downstream stories in this epic depend on it. Downstream cycles (holdout evaluation, wave scheduling) may reference it but are not modeled as story dependencies.
