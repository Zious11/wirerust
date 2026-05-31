---
document_type: story
story_id: "STORY-033"
epic_id: "E-3"
version: "1.3"
status: completed
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.007.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.008.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.009.md
input-hash: "53e33d2"
traces_to: .factory/specs/prd.md
points: 3
depends_on: [STORY-031, STORY-032]
blocks: [STORY-041, STORY-051]
behavioral_contracts:
  - BC-2.05.007
  - BC-2.05.008
  - BC-2.05.009
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 14
target_module: src/dispatcher.rs
subsystems: [SS-05]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **Execute:** `/vsdd-factory:deliver-story STORY-033`

# STORY-033: Flow Lifecycle — Close, Unclassified Counter, No-Op Dispatcher

## Narrative
- **As a** forensic analyst
- **I want to** have `on_flow_close` correctly clean up dispatcher state (routes, attempt counters), forward the close event to the correct analyzer, increment the `unclassified_flows` counter for flows that could not be classified, and have the dispatcher do nothing when no analyzers are configured
- **So that** memory is not leaked across flows and the unclassified-flows metric provides accurate audit coverage information

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.05.007 | unclassified_flows Increments Only at on_flow_close |
| BC-2.05.008 | No Analyzer Configured: Dispatcher Early-Returns |
| BC-2.05.009 | on_flow_close Removes Route Entry and Forwards Close |

## Acceptance Criteria

### AC-001 (traces to BC-2.05.007 postcondition 1-2)
`unclassified_flows` is incremented by 1 in `on_flow_close` when `routes.remove(flow_key)` returns either `None` (no route entry) or `Some(DispatchTarget::None)` (cached None from retry-cap logic). No analyzer's `on_flow_close` is called for unclassified flows.
- **Test:** `test_BC_2_05_007_unclassified_flows_counter`

### AC-002 (traces to BC-2.05.007 invariant 1-3)
`unclassified_flows` is monotonically increasing and never decrements. Classified flows (Http or Tls route) do NOT increment `unclassified_flows` on close. The counter increments only when at least one analyzer is configured (guard: `if self.http.is_some() || self.tls.is_some()`).
- **Test:** `test_BC_2_05_007_classified_flow_not_counted_as_unclassified` (classified non-increment + monotonicity); `test_BC_2_05_008_no_analyzer_dispatcher_early_returns` (guard-False branch — no-analyzer dispatcher)

### AC-003 (traces to BC-2.05.007 invariant 4)
Flows with no data (no `on_data` called before `on_flow_close`) may contribute to `unclassified_flows` because they have no cached route — this is noted as a known metric limitation (SYN-only / handshake-only flows).
- **Test:** `test_BC_2_05_007_unclassified_flows_counter`

### AC-004 (traces to BC-2.05.008 postcondition 1-5)
When `StreamDispatcher` is created with `http = None` and `tls = None`, `on_data` returns immediately at the first check (`if self.http.is_none() && self.tls.is_none() { return; }`) without running `classify`, updating any counters, or touching `routes` or `classification_attempts`.
- **Test:** `test_BC_2_05_008_no_analyzer_dispatcher_early_returns`

### AC-005 (traces to BC-2.05.008 invariant 1-3)
The early-return guard is the FIRST statement in `on_data` (before route lookup and classify). This guard does NOT affect `on_flow_close`. A dispatcher with only one analyzer (http=Some, tls=None) is NOT subject to this early return.
- **Test:** `test_BC_2_05_008_single_analyzer_not_early_returned`

### AC-006 (traces to BC-2.05.009 postcondition 1-2)
`on_flow_close` always calls `self.classification_attempts.remove(flow_key)` and `let target = self.routes.remove(flow_key)` unconditionally — both side effects execute regardless of flow classification state.
- **Test:** `test_BC_2_05_007_unclassified_flows_counter`

### AC-007 (traces to BC-2.05.009 postcondition 3-4)
If `target == Some(DispatchTarget::Http)`, `http.on_flow_close(flow_key, reason)` is called (via `if let Some(ref mut http) = self.http`). If `target == Some(DispatchTarget::Tls)`, `tls.on_flow_close(flow_key, reason)` is called. No panic occurs if the respective analyzer is None.
- **Test:** `test_BC_2_05_009_flow_close_forwards_to_http_analyzer`

### AC-008 (traces to BC-2.05.009 invariant 3)
Each flow contributes its close event to exactly one destination: one analyzer (Http or Tls) or the `unclassified_flows` counter. No flow contributes to both an analyzer close and the unclassified counter.
- **Test:** `test_BC_2_05_009_flow_close_forwards_to_http_analyzer`

### AC-009 (traces to BC-2.05.009 edge case EC-004)
`on_flow_close` called for a FlowKey not present in `routes` (no prior `on_data`) results in `routes.remove()` returning None — which executes the unclassified branch, incrementing `unclassified_flows` if analyzers are configured. No panic.
- **Test:** `test_BC_2_05_009_flow_close_for_unknown_flow_key`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| on_flow_close | src/dispatcher.rs:171-194 | effectful-shell (removes from routes and classification_attempts; mutates unclassified_flows; calls downstream analyzer) |
| early-return guard | src/dispatcher.rs:121-123 | effectful-shell (early return in on_data) |
| unclassified_flows counter | src/dispatcher.rs:188-191 | effectful-shell |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Classified as Http then closed | Http.on_flow_close called; unclassified NOT incremented |
| EC-002 | Classified as Tls then closed | Tls.on_flow_close called; unclassified NOT incremented |
| EC-003 | Flow never classified (no data sent) | unclassified_flows=1 on close |
| EC-004 | Flow with None-cached route closed | unclassified_flows=1 on close |
| EC-005 | Two unclassified flows closed | unclassified_flows=2 |
| EC-006 | Dispatcher has no analyzers; unclassified flow closed | unclassified NOT incremented (guard: no analyzers) |
| EC-007 | http=None, tls=None; on_data called | Returns immediately; no classify; no state change |
| EC-008 | http=Some, tls=None; TLS data | Not early-returned; classify runs; may classify as Tls but no Tls analyzer to forward to |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/dispatcher.rs (on_flow_close) | effectful-shell | Mutates routes, classification_attempts, unclassified_flows; calls downstream analyzer |
| src/dispatcher.rs (early-return in on_data) | effectful-shell | Guard reads self state; no mutation on early return path |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| Referenced code (dispatcher.rs:121-123, 171-194) | ~2,000 |
| Test files (dispatcher_tests.rs) | ~30,000 |
| BC files (3 BCs) | ~3,000 |
| Tool outputs overhead | ~1,500 |
| **Total** | **~39,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~20%** |

## Tasks (MANDATORY)

0. [ ] Confirm brownfield-formalization invariant: src/dispatcher.rs is byte-identical to develop (no production-behavior changes)
1. [ ] Write failing tests for AC-001 through AC-009 (test-writer)
2. [ ] Verify Red Gate: all tests fail before implementation
3. [ ] Verify `on_flow_close` behavior per BC-2.05.009 at dispatcher.rs:171-194 (unconditional attempts.remove + routes.remove, match on returned target, forward to correct analyzer or increment unclassified_flows)
4. [ ] Verify `unclassified_flows` increment guard per BC-2.05.007 at dispatcher.rs:188-191 (only when analyzer configured; only for None/unclassified flows)
5. [ ] Verify no-analyzer early-return guard per BC-2.05.008 at dispatcher.rs:121-123 (first statement in on_data; not in on_flow_close)
6. [ ] Verify close forwarding uses safe `if let Some` pattern (no panic when analyzer is None) — already present in develop
7. [ ] Run all tests; verify all pass
8. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-031 | `StreamDispatcher` holds optional Box<dyn Analyzer> for http and tls | on_flow_close must use `if let Some(ref mut http) = self.http` pattern to avoid panic when one analyzer is absent | Flow close for a never-seen FlowKey results in routes.remove() returning None — this is a valid no-op case, not an error |
| STORY-032 | `classification_attempts` stores attempt counts per FlowKey; `routes` stores final classification | Both maps must be cleaned up in on_flow_close; missing either causes memory leak per flow | The unclassified_flows counter guard checks for at least one configured analyzer — dispatchers with no analyzers never increment it |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `classification_attempts.remove()` is always called in on_flow_close, unconditionally | BC-2.05.009 invariant 1 | Code review: confirm remove() before the match |
| `routes.remove()` is always called in on_flow_close, unconditionally | BC-2.05.009 invariant 1 | Code review: confirm remove() before the match |
| unclassified_flows guard: only increments when `self.http.is_some() || self.tls.is_some()` | BC-2.05.007 invariant 3 | Unit test: AC-006 (no-analyzer dispatcher) |
| Early-return guard in on_data is the FIRST statement (before route lookup) | BC-2.05.008 invariant 1 | Code review: confirm position at dispatcher.rs:121-123 |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust std | 2024 edition (stable) | HashMap::remove, Option matching, if let patterns |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/dispatcher.rs | modify | on_flow_close (171-194): unconditional cleanup, match on target, analyzer forwarding, unclassified_flows increment; on_data early-return guard (121-123) |
| tests/dispatcher_tests.rs | modify | Add: test_BC_2_05_007_unclassified_flows_counter, test_BC_2_05_007_classified_flow_not_counted_as_unclassified, test_BC_2_05_008_no_analyzer_dispatcher_early_returns, test_BC_2_05_008_single_analyzer_not_early_returned, test_BC_2_05_009_flow_close_forwards_to_http_analyzer, test_BC_2_05_009_flow_close_for_unknown_flow_key |
| src/analyzer/http.rs | modify | Add `#[doc(hidden)] pub fn active_flows_len_for_testing(&self) -> usize` at line 640-651 (AC-007 indirect-observability seam for HttpAnalyzer.on_flow_close → flows.remove proof) |
| src/analyzer/tls.rs | modify | Add `#[doc(hidden)] pub fn active_flows_len_for_testing(&self) -> usize` at line 847-858 (AC-007 indirect-observability seam for TlsAnalyzer.on_flow_close → flows.remove proof) |

**Note**: HttpAnalyzer and TlsAnalyzer expose no public accessor for self.flows.len(); AC-007 requires observation of post-close flow-state cleanup. Two minimal `#[doc(hidden)] pub fn _for_testing` seams were added per the W11 STORY-021 / DF-SIBLING-SWEEP-001 v3 doctrine — these are pre-existing test-only patterns, additive only, with zero production-behavior impact.

## Changelog

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| v1.0 | 2026-05-21 | Initial story creation | story-writer |
| v1.1 | 2026-05-28 | W14 Pass 1 remediation: AC test-name refresh to BC-prefixed (F-001); FSR row 5→6 tests (F-002); FSR rows added for analyzer seams (F-003); AC-002 dual-test cite (F-007); Tasks reworded for brownfield-formalization (F-009); token budget recalibrated (F-010) | story-writer |
| v1.2 | 2026-05-28 | DF-SIBLING-SWEEP-001 v4 propagation — BC-2.05.008 v1.4 (EC-002 wording disambiguated); input-hash recomputed: `9610207` → `4aa119d` (sha256 over sorted cited-BC files BC-2.05.007/008/009, first 7 chars). No AC citation changes required. | story-writer |
| v1.3 | 2026-05-29 | Drift remediation F-DRIFT-S-001: status reconciled from `in-progress` to `completed` (STORY-033 was merged in Wave 14 PR #137; frontmatter not updated at wave-close). | state-manager |
| v1.4 | 2026-05-29 | state-manager | input-hash corrected via canonical bin/compute-input-hash --update (prior value `4aa119d` was hand-computed sha256 over sorted inputs-file list; tool uses MD5 over inputs-order file list). New value: `9610207`. |
