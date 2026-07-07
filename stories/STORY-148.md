---
document_type: story
story_id: STORY-148
epic_id: E-20
version: "1.1"
status: superseded
producer: story-writer
timestamp: 2026-07-01T00:00:00Z
phase: f7
level: feature
cycle: maint-2026-07-01
points: 5
priority: P2
depends_on: []
blocks: []
behavioral_contracts: []
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: src/dispatcher.rs
subsystems: []
estimated_days: 2
wave: "~"
traces_to:
  - src/dispatcher.rs
  - src/analyzer/enip.rs
  - src/analyzer/dnp3.rs
  - tests/issue_342_flow_leak_regression_tests.rs
input-hash: d41d8cd
inputs: []
# superseded-by: PR #362 (D-383, issue #342 closed 2026-07-06)
# reconciled: 2026-07-07 (pipeline resume); all ACs verified on develop
# v1.1 (2026-07-07): Status changed draft→superseded. PR #362 fully implemented
#   all acceptance criteria: SEC-005 ENIP on_flow_close wiring + SEC-006 DNP3
#   on_flow_close routing both wired in dispatcher.rs; regression tests at
#   tests/issue_342_flow_leak_regression_tests.rs; issue #342 closed 2026-07-06.
#   No separate implementation wave required. Template compliance fields added.
---

# STORY-148 — Fix Analyzer Flow-State Lifecycle: EnipAnalyzer on_flow_close Wiring + DNP3 Flow-Map Cap (SEC-005 / SEC-006)

**Epic:** E-20 (EtherNet/IP ENIP/CIP Analyzer)
**Status:** superseded — resolved by PR #362 (D-383, issue #342 closed 2026-07-06)
**Wave:** TBD (superseded; no implementation wave needed)
**Points:** 5

## Narrative

- **As a** developer and security reviewer on the wirerust project
- **I want** the analyzer flow-state lifecycle to correctly forward close events to
  EnipAnalyzer and Dnp3Analyzer via the stream dispatcher
- **So that** per-flow state maps do not grow monotonically under long-running captures
  or crafted pcaps with large numbers of short-lived flows (file-based DoS, CWE-400)

_(Superseded — this narrative was fulfilled by PR #362.)_

## Behavioral Contracts

_(none — no BCs were authored; story superseded before BC authorship phase)_

## Background

Maintenance run maint-2026-07-01 identified two related memory-safety defects in the
analyzer flow-state lifecycle:

**SEC-005 (MEDIUM, CWE-400, real bug):** `StreamDispatcher::on_flow_close` in
`src/dispatcher.rs` (~lines 409–414) contains a no-op arm for the ENIP analyzer — it
does not forward the close event to `EnipAnalyzer::on_flow_close`. As a result,
`EnipAnalyzer.flows` (enip.rs ~line 782, `.entry().or_default()`) grows monotonically:
every distinct port-44818 flow inserts an entry that is never removed. A crafted pcap
with a large number of short-lived ENIP flows exhausts heap memory, constituting a
file-based DoS.

Additionally, all the flow-close aggregation logic in `EnipAnalyzer::on_flow_close`
(enip.rs ~line 693) — including final-byte accounting and per-flow statistics folding —
has been dead code since STORY-138 delivered the ENIP session lifecycle. Root cause:
the dispatcher dispatch table was wired for ENIP data delivery (`on_data`) but the
close arm was left as a no-op placeholder.

**SEC-006 (MEDIUM, CWE-400, design decision required):** `Dnp3Analyzer.flows`
(dnp3.rs ~line 303) accumulates all historical flows. The `summarize()` path consumes
them — which appears by design — but the same file-based DoS profile exists for
long-running captures with many distinct DNP3 flows. Unlike SEC-005, SEC-006 requires
an explicit design decision before implementation: either add a hard cap on `flows.len()`
with LRU eviction (analogous to `TcpReassembler.max_flows`) or wire through a DNP3
`on_flow_close` callback. This story captures the design decision and its implementation
as a scoped AC.

## Goal

1. Wire `StreamDispatcher::on_flow_close` to call `EnipAnalyzer::on_flow_close(flow_key)`
   in the ENIP arm — making the long-present aggregation logic reachable and preventing
   unbounded `flows` map growth.
2. Add a regression test that creates multiple ENIP flows, closes each, and asserts
   that the flow map entry is removed and per-flow aggregates are folded into totals.
3. Document and implement the chosen mitigation for DNP3 flow-map growth (hard cap with
   LRU eviction, or `on_flow_close` routing), with a corresponding test.

## Acceptance Criteria

AC-148-001: `StreamDispatcher::on_flow_close` in `src/dispatcher.rs` calls
  `enip.on_flow_close(flow_key)` (or equivalent entry removal + aggregate fold) in the
  ENIP arm — the no-op arm is eliminated. The `EnipAnalyzer::on_flow_close` method at
  enip.rs ~line 693 is no longer dead code.

AC-148-002: A regression test (in `tests/` or `src/analyzer/enip.rs` test module)
  creates N distinct ENIP flows (N >= 2), calls the dispatcher close path for each,
  and asserts that `EnipAnalyzer.flows.len()` is 0 after all closures and that per-flow
  aggregates (bytes_total, packet_count) appear in the analyzer summary.

AC-148-003: An explicit design note is committed (in `docs/adr/` as an addendum to an
  existing ADR, or as a doc-comment policy block in the DNP3 analyzer source) documenting
  the chosen approach for DNP3 flow-map growth: Option A — `max_flows` hard cap with LRU
  eviction (default capped at a value matching `TcpReassembler.max_flows` convention); or
  Option B — DNP3 `on_flow_close` routing mirroring the SEC-005 fix. The note records the
  rationale for the choice.

AC-148-004: The chosen DNP3 mitigation is implemented: either `Dnp3Analyzer.flows` has
  a configurable capacity cap with LRU eviction OR `Dnp3Analyzer::on_flow_close` is
  wired through the dispatcher. A corresponding test asserts the memory bound is respected
  under synthetic flow churn (create N flows > the cap, verify `flows.len()` stays bounded).

AC-148-005: `cargo clippy --all-targets -- -D warnings` and `cargo test --all-targets`
  pass without new warnings or regressions introduced by this change.

## Architecture Mapping

_(Superseded — see Reconciliation Note below for code locations.)_

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| ENIP on_flow_close wiring | `src/dispatcher.rs` | Effectful (state mutation) |
| EnipAnalyzer::on_flow_close | `src/analyzer/enip.rs` | Effectful (map removal + aggregate fold) |
| Dnp3Analyzer::on_flow_close | `src/analyzer/dnp3.rs` | Effectful (map removal + aggregate fold) |
| SEC-005/SEC-006 regression tests | `tests/issue_342_flow_leak_regression_tests.rs` | Test |

## Purity Classification

| File | Classification | Reason |
|------|---------------|--------|
| `src/dispatcher.rs` | Effectful | Mutates analyzer state on flow close |
| `src/analyzer/enip.rs` | Effectful | HashMap::remove + aggregate fold |
| `src/analyzer/dnp3.rs` | Effectful | HashMap::remove + aggregate fold |
| `tests/issue_342_flow_leak_regression_tests.rs` | Test | No production side effects |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flow closed before any `on_data` (no map entry) | `on_flow_close` is a no-op; no panic |
| EC-002 | Multiple closes of same flow key | Second remove returns None; aggregate unchanged |
| EC-003 | N flows opened and closed in churn loop | `flows.len()` remains 0 after all closes |

## Tasks

_(Superseded — all tasks were completed via PR #362. No implementation tasks remain.)_

1. ~~Wire ENIP on_flow_close arm in dispatcher.rs~~ — done (PR #362)
2. ~~Add ENIP regression test~~ — done (`tests/issue_342_flow_leak_regression_tests.rs`)
3. ~~Design note + DNP3 on_flow_close wiring (Option B chosen)~~ — done (PR #362)
4. ~~DNP3 regression test~~ — done (`tests/issue_342_flow_leak_regression_tests.rs`)

## Previous Story Intelligence

- STORY-138 (wave 61) delivered the ENIP session lifecycle and introduced
  `EnipAnalyzer::on_flow_close` — but the dispatcher arm was left as a no-op,
  which is the root cause of SEC-005.
- STORY-139 (wave 62) fixed EC-X1/EC-X2 carry-direction issues in the same ENIP
  analyzer; same epic, same pattern of correctness fix after initial delivery.

## Architecture Compliance Rules

_(Superseded — constraints were met by PR #362.)_

- The fix must not alter any analyzer's `on_data` path or existing detection logic.
- The empty-map case (no entry for a given flow key) must be handled without panic.
- No new Rust dependencies are permitted for this fix.

## Library & Framework Requirements

- No new Rust dependencies. Uses `std::collections::HashMap::remove`.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/dispatcher.rs` | Modified | ENIP + DNP3 on_flow_close arms wired (PR #362) |
| `src/analyzer/enip.rs` | Modified | on_flow_close + aggregate folding activated (PR #362) |
| `src/analyzer/dnp3.rs` | Modified | on_flow_close routing + aggregate fields added (PR #362) |
| `tests/issue_342_flow_leak_regression_tests.rs` | Created | SEC-005 + SEC-006 regression tests (PR #362) |

## Token Budget Estimate (MANDATORY)

_(Superseded — story never entered implementation phase.)_

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~4 k |
| `src/dispatcher.rs` (relevant section) | ~1 k |
| `src/analyzer/enip.rs` (on_flow_close + aggregate) | ~2 k |
| `src/analyzer/dnp3.rs` (on_flow_close + aggregate) | ~2 k |
| **Total** | **~9 k** |

## Notes

- Strong v0.12.0 candidate: SEC-005 is a file-based DoS reachable via a crafted pcap
  against any wirerust deployment that analyzes ENIP traffic. SEC-006 is lower urgency
  (the `summarize()` consumption provides partial mitigation for in-process live captures)
  but should be resolved in the same release window to close the CWE-400 class.
- Source findings: SEC-005 (MEDIUM, CWE-400) + SEC-006 (MEDIUM, CWE-400),
  maintenance run maint-2026-07-01.
- Primary modules: `src/dispatcher.rs` (SEC-005 wiring fix), `src/analyzer/enip.rs`
  (aggregation activation), `src/analyzer/dnp3.rs` (SEC-006 mitigation).
- STORY-138 is the original ENIP session-lifecycle story; STORY-148 closes the gap
  left when the dispatcher on_flow_close arm was not wired during STORY-138 delivery.
- Precedent for E-20 maintenance fix pattern: STORY-139 (EC-X1/EC-X2 carry-direction
  fixes, wave 62) — same epic, same pattern of correctness fix added after initial
  delivery wave.
- Wave assignment is TBD — schedule at v0.12.0 planning alongside STORY-091, STORY-121,
  STORY-143, STORY-147, STORY-149, and STORY-150 (all unscheduled).

---

## Reconciliation Note (2026-07-07)

**Status: SUPERSEDED — all scope delivered by PR #362 (D-383, issue #342 closed 2026-07-06)**

All acceptance criteria were implemented and merged to `develop` via PR #362. No
separate implementation wave is required. Evidence verified on `develop` (2026-07-07):

**AC-148-001 — ENIP on_flow_close wiring:**
`src/dispatcher.rs` lines 456–462: `Some(DispatchTarget::Enip)` arm calls
`enip.on_flow_close(flow_key.clone())` with comment `BC-2.17.019 / SEC-005 / issue
#342`. The no-op arm is eliminated. `EnipAnalyzer::on_flow_close` at
`src/analyzer/enip.rs:693` is live reachable code.

**AC-148-002 — ENIP regression test:**
`tests/issue_342_flow_leak_regression_tests.rs`: SEC-005 single-flow close test
asserts `EnipAnalyzer.flows.len() == 0` after dispatcher close; bounded-retention
test opens and closes N ENIP flows and asserts zero residual state.

**AC-148-003 — DNP3 design note (Option B chosen):**
`src/analyzer/dnp3.rs:362` doc-comment explicitly mirrors `EnipAnalyzer::on_flow_close`
pattern (SEC-006 / issue #342 / BC-2.15.021). `src/dispatcher.rs:449–455` comment:
`BC-2.15.021 / SEC-006 / issue #342: forward on_flow_close to Dnp3Analyzer to purge
per-flow state and fold metrics into aggregates.` Option B (on_flow_close routing)
was chosen over Option A (hard cap + LRU eviction).

**AC-148-004 — DNP3 on_flow_close implemented:**
`src/dispatcher.rs` lines 448–455: `Some(DispatchTarget::Dnp3)` arm calls
`dnp3.on_flow_close(flow_key.clone())`. `Dnp3Analyzer::on_flow_close` at
`src/analyzer/dnp3.rs:378` is implemented with aggregate folding.

**AC-148-005 — Tests and lints pass:**
PR #362 merged to `develop`; CI green (cargo test + cargo clippy per D-383).
