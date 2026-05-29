---
document_type: story
story_id: "STORY-021"
epic_id: "E-2"
version: "1.9"
status: draft
producer: story-writer
timestamp: 2026-05-27T06:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.012.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.024.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.025.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.026.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.054.md
input-hash: "68dadd4"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-017, STORY-018, STORY-019, STORY-020]
blocks: [STORY-031]
behavioral_contracts: [BC-2.04.012, BC-2.04.024, BC-2.04.025, BC-2.04.026, BC-2.04.054]
verification_properties: [VP-003]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 11
target_module: reassembly
subsystems: [SS-04]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — all ACs must be backed by tests.

> **Execute:** `/vsdd-factory:deliver-story STORY-021`

# STORY-021: Finalize Lifecycle, MAX_FINDINGS Cap, and Segment-Limit Summary Finding

## Narrative
- **As a** forensic analyst
- **I want** the TCP reassembly engine to correctly conclude end-of-capture by flushing and closing all remaining flows, enforce a hard `MAX_FINDINGS = 10,000` cap on normal finding emission (with one intentional bypass for the segment-limit summary), and emit a grammatically correct segment-limit summary finding only when segments were actually dropped
- **So that** every PCAP analysis run terminates cleanly, findings are bounded to prevent memory exhaustion even under adversarial conditions, and the segment-limit summary provides an accurate forensic signal when the BTreeMap overflow protection triggered

## Behavioral Contracts

| BC | Title | Role in Story |
|----|-------|---------------|
| BC-2.04.012 | finalize Flushes All Remaining Flows; Idempotent | End-of-capture lifecycle and idempotency latch |
| BC-2.04.024 | Total Findings Capped at MAX_FINDINGS=10000; Excess Silently Dropped | Normal finding cap and dropped_findings counter |
| BC-2.04.025 | finalize Emits Segment-Limit Summary Finding When Segments Dropped | Summary finding content and unconditional push |
| BC-2.04.026 | finalize Does NOT Emit Segment-Limit Finding When Counter is Zero | Zero-count guard; no spurious findings |
| BC-2.04.054 | finalize Unconditionally Bypasses MAX_FINDINGS Cap for Segment-Limit Finding | The sole INV-6 exception |

## Acceptance Criteria

### AC-001 (traces to BC-2.04.012 postcondition 1-2)
- After `finalize(handler)` is called on an engine with N open flows, all N flows are closed via `close_flow(key, CloseReason::Timeout, handler)` and `self.flows` is empty.
- **Test:** `test_BC_2_04_012_finalize_closes_all_remaining_flows()`

### AC-002 (traces to BC-2.04.012 postcondition 3)
- After `finalize` completes, `self.finalized == true`.
- **Test:** `test_BC_2_04_012_finalize_sets_finalized_latch()`

### AC-003 (traces to BC-2.04.012 postcondition 5 and invariant 1)
- Calling `finalize` a second time is a complete no-op: no additional `close_flow` calls, no additional findings, no additional callbacks.
- **Test:** `test_BC_2_04_012_finalize_is_idempotent()`

### AC-004 (traces to BC-2.04.012 edge case EC-006)
- When the reassembler is dropped WITHOUT calling `finalize`, a one-shot `eprintln!` warning fires (from `impl Drop` tripwire). Flows are NOT flushed (Drop has no handler argument).
- **Scope limitation:** Under cargo parallel scheduling, this test verifies the Drop hook fires AT LEAST ONCE in this process, not unique per-Drop attribution (Option B per F-W11P2-002 rationale; see Architecture Compliance Rule for AC-004 and the FINALIZE_SKIPPED_WARNED_LOCK docstring in tests/reassembly_engine_tests.rs).
- **Test:** `test_BC_2_04_012_drop_without_finalize_emits_warning()`

### AC-005 (traces to BC-2.04.024 postcondition 1-2)
- When `findings.len() >= MAX_FINDINGS` (= 10,000) and a new finding would normally be emitted by normal packet processing, the finding is NOT pushed and `stats.dropped_findings` increments by 1.
- **Test:** `test_BC_2_04_024_findings_capped_at_max_findings()`

### AC-006 (traces to BC-2.04.024 edge case EC-001)
- When `findings.len() == MAX_FINDINGS - 1` (9,999), the next finding IS added (bringing the count to 10,000).
- **Test:** `test_BC_2_04_024_finding_added_at_one_below_cap()`

### AC-007 (traces to BC-2.04.024 invariant 1 — engine cap boundary)
- When `findings.len()` reaches exactly `MAX_FINDINGS` (10,000) the boundary is held precisely: the 10,000th finding is accepted, and the 10,001st from the normal path is rejected and increments `stats.dropped_findings` by 1.
- **Test:** `test_BC_2_04_024_engine_cap_at_exactly_10000`

### AC-007b (traces to BC-2.04.024 invariant 4 — analyzer cap isolation)
- HttpAnalyzer **and** TlsAnalyzer findings are NOT subject to the reassembly-engine `MAX_FINDINGS` cap; analyzer-side findings collections can grow beyond 10,000. Test: `test_BC_2_04_024_http_tls_analyzer_findings_not_capped` (pushes 10,001 findings via additive `push_finding_for_testing` seams on **both** HttpAnalyzer and TlsAnalyzer; asserts each accumulates all 10,001).

### AC-008 (traces to BC-2.04.025 postcondition 1)
- When `finalize` is called and `stats.segments_segment_limit > 0`, exactly one finding is pushed with: `category: Anomaly`, `verdict: Inconclusive`, `confidence: Medium`, `mitre_technique: None`, and `source_ip: None`, `direction: None`.
- **Test:** `test_BC_2_04_025_finalize_emits_segment_limit_finding()`

### AC-009 (traces to BC-2.04.025 postcondition 1 and invariant 3)
- The summary string uses correct singular/plural grammar: `"1 segment dropped due to per-flow segment count limit"` when count==1, and `"N segments dropped due to per-flow segment count limit"` when count==N (N>1; tested at count=2 — the true plural-boundary — with count=100 as sanity).
- **Test:** `test_BC_2_04_025_segment_limit_finding_singular_plural_grammar()`

### AC-010 (traces to BC-2.04.025 postcondition 1)
- The segment-limit finding's `evidence` vec contains exactly: `["Segment count limit prevents BTreeMap overhead explosion", "May indicate segmentation-based evasion attempt"]`.
- **Test:** `test_BC_2_04_025_segment_limit_finding_evidence_strings()`

### AC-011 (traces to BC-2.04.026 postcondition 1-2)
- When `finalize` is called and `stats.segments_segment_limit == 0`, NO segment-limit finding is pushed. The findings list contains only findings generated during packet processing.
- **Test:** `test_BC_2_04_026_no_segment_limit_finding_when_counter_zero()`

### AC-012 (traces to BC-2.04.054 postcondition 1-2 and invariant 1)
- When `findings.len() == MAX_FINDINGS` at finalize time and `segments_segment_limit > 0`, the segment-limit finding IS pushed unconditionally, causing `findings.len()` to become `MAX_FINDINGS + 1` (= 10,001).
- **Test:** `test_BC_2_04_054_finalize_bypasses_max_findings_cap()`

### AC-013 (traces to BC-2.04.054 invariant 3 — representative bound scenario)
- Under a representative finalize-bypass scenario (findings pre-filled to `MAX_FINDINGS`, `segments_segment_limit > 0`, then `finalize` called), the resulting `findings.len()` is exactly `MAX_FINDINGS + 1` (= 10,001) — a smoke test of the bound under known-stressful inputs. The universal upper-bound proof (∀ runs, `len ≤ 10,001`) belongs to VP-003 (property-based test); this AC verifies the bound at a single representative scenario only. The test also defensively verifies post-bypass idempotency (a second `finalize` call produces no additional segment-limit finding) — this is redundant with AC-003 but exercises the bypass→idempotency state transition specifically.
- **Test:** `test_BC_2_04_054_finalize_bypass_smoke_at_max_findings_representative_scenario`

### AC-014 (traces to BC-2.04.024 edge case EC-004 — dropped_findings monotonicity)
- Consecutive cap-hit events each increment `stats.dropped_findings` by exactly 1; the counter is monotonic and accumulates correctly across N successive cap-hit emissions (verified at N=3).
- **Test:** `test_BC_2_04_024_dropped_findings_monotone_over_multiple_cap_hits`

### AC-015 (traces to BC-2.04.024 postconditions 1-2 — small_segment cap-guard site)
- The small-segment cap-guard at `src/reassembly/mod.rs:466` (small_segment_alert emission path) correctly rejects the finding and increments `stats.dropped_findings` when `findings.len() >= MAX_FINDINGS`. The out-of-window cap-guard at `mod.rs:495` is covered by sibling-story STORY-017 test `test_story_017_ec007_oow_alert_at_max_findings_latch_set_dropped_incremented`.
- **Test:** `test_BC_2_04_024_cap_guard_small_segment_site`

### AC-016 (traces to BC-2.04.054 postconditions 1-2 + BC-2.04.054 EC-002 'bypass semantics not triggered below cap'; satisfies story EC-006 boundary)
- When `findings.len() == MAX_FINDINGS - 1` (9,999) at finalize time and `segments_segment_limit > 0`, the segment-limit finding pushes to exactly `MAX_FINDINGS` (10,000) — still within the normal cap (the bypass path accommodates this exactly).
- **Test:** `test_BC_2_04_054_finalize_at_max_findings_minus_one`

### AC-017 (traces to BC-2.04.054 edge case EC-002 — segment-limit push is unconditional)
- The finalize segment-limit push fires regardless of initial `findings.len()` value (verified at 0, 5,000, 9,999, and 10,000). At `<MAX` initial count the push uses the normal-path semantic; at `==MAX` the push uses the bypass semantic; the outcome (exactly one segment-limit finding pushed) is identical in all cases.
- **Test:** `test_BC_2_04_054_segment_limit_finding_emitted_regardless_of_initial_count`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| finalize (latch, flow loop, segment-limit block) | src/reassembly/mod.rs:557-591 | effectful-shell |
| impl Drop tripwire | src/reassembly/mod.rs:794-808 | effectful-shell (stderr write) |
| MAX_FINDINGS constant | src/reassembly/mod.rs:54 | pure-core (constant) |
| dropped_findings counter sites | src/reassembly/mod.rs:432,466,495; lifecycle.rs:101,121 | effectful-shell |
| plural_s helper | src/reassembly/mod.rs:66-68 | pure-core |
| segment-limit finding push (unconditional) | src/reassembly/mod.rs:573 | effectful-shell |
| `all_findings_len_for_testing` / `push_finding_for_testing` | src/analyzer/http.rs | effectful-shell (test-only) |
| `all_findings_len_for_testing` / `push_finding_for_testing` | src/analyzer/tls.rs | effectful-shell (test-only) |
| `finalize_skipped_warned_for_testing() -> bool` | src/reassembly/mod.rs | effectful-shell (test-only) |
| `reset_finalize_skipped_warned_for_testing()` | src/reassembly/mod.rs | effectful-shell (test-only) |
| `swap_finalize_skipped_warned_for_testing(value: bool) -> bool` | src/reassembly/mod.rs | effectful-shell (test-only) |
| `set_segments_segment_limit_for_testing(&mut self, count: u64)` | src/reassembly/mod.rs | effectful-shell (test-only, trust-boundary per F-W11P1-009) |
| `push_finding_for_testing(&mut self, finding: Finding)` | src/reassembly/mod.rs | effectful-shell (test-only) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | finalize with zero open flows | No close calls; finalized=true; segment-limit finding conditionally emitted |
| EC-002 | finalize called twice | Second call immediate no-op via finalized guard |
| EC-003 | segments_segment_limit == 1 | Summary: "1 segment dropped..." (singular) |
| EC-004 | segments_segment_limit == 100 | Summary: "100 segments dropped..." (plural) |
| EC-005 | findings at MAX_FINDINGS, then finalize with limit>0 | findings.len() becomes 10001; not 10000 |
| EC-006 | findings at MAX_FINDINGS-1, then finalize with limit>0 | Segment-limit finding pushes to 10000; still within MAX_FINDINGS |
| EC-007 | Drop without finalize | One-shot eprintln; flows NOT flushed (no handler in Drop) |
| EC-008 | Clean PCAP (no anomalies, no segment limit) | findings.is_empty() (or only flow-generated findings); no segment-limit finding |
| EC-009 | Multiple consecutive cap-hit events | `dropped_findings` accumulates monotonically (N events → counter += N); verified at N=3 (BC-2.04.024 EC-004). Backed by AC-014 |
| EC-010 | All 5 cap-guard sites correctly reject + increment `dropped_findings` at MAX_FINDINGS | (1) **mod.rs:432** overlap_alert path — covered by `test_BC_2_04_022_latch_fires_before_cap_check` (sibling story); (2) **mod.rs:466** small_segment_alert path — covered by AC-015 `test_BC_2_04_024_cap_guard_small_segment_site`; (3) **mod.rs:495** out_of_window_alert path — covered by `test_story_017_ec007_oow_alert_at_max_findings_latch_set_dropped_incremented` (STORY-017 sibling); (4) **lifecycle.rs:101** conflicting_overlap_finding path — primarily covered by `test_story_017_ec001_conflicting_overlap_at_max_findings_drops_and_counts` (STORY-017 sibling, structurally specific to this site); also incidentally exercised by AC-005 `test_BC_2_04_024_findings_capped_at_max_findings`; (5) **lifecycle.rs:121** truncated_finding path — covered by `test_BC_2_04_023_truncated_finding_dropped_at_cap` (sibling story) |
| EC-011 | HttpAnalyzer / TlsAnalyzer findings collections exceed 10,000 | NOT capped — the MAX_FINDINGS cap is engine-local to TcpReassembler; analyzer collections grow unbounded. Backed by AC-007b |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reassembly/mod.rs (finalize, impl Drop) | effectful-shell | Mutates self.flows, self.finalized, self.findings; invokes callbacks; writes stderr |
| src/reassembly/mod.rs (plural_s, MAX_FINDINGS const) | pure-core | No I/O; deterministic |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| BC files (5 BCs) | ~5,500 |
| src/reassembly/mod.rs (finalize ~557-591, impl Drop ~794-808, MAX_FINDINGS ~54, dropped_findings sites ~432,466,495) | ~2,500 |
| src/reassembly/lifecycle.rs (dropped_findings sites ~101,121) | ~500 |
| src/analyzer/http.rs (test seams: push_finding_for_testing, all_findings_len_for_testing) | ~500 |
| src/analyzer/tls.rs (test seams: push_finding_for_testing, all_findings_len_for_testing) | ~500 |
| Test files (13 original + 5 new post-pass-1 ACs: AC-007b, AC-014..AC-017) | ~5,500 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~19,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~9.5%** |

## Tasks (MANDATORY)

1. [x] Write failing tests for the initial 13 ACs (AC-001..AC-013) in `tests/reassembly_engine_tests.rs` — post-pass-1 ACs (AC-007b, AC-014..AC-017) added by tasks 11-19 below
2. [x] Verify Red Gate: all tests fail before implementation changes
3. [x] Verify existing implementation satisfies all ACs (brownfield)
4. [x] Test finalize idempotency: call twice; assert on_flow_close callbacks fire exactly N times (pin close_count == 2 after first finalize before idempotency check)
5. [x] Test MAX_FINDINGS boundary: fill to 9999, verify 10000th is accepted; fill to 10000, verify 10001st is dropped
6. [x] Test finalize bypass: fill findings to 10000 (MAX); trigger segment limit; call finalize; assert findings.len()==10001
7. [x] Test singular/plural: segments_segment_limit=1 produces "1 segment dropped..."; limit=2 (true plural-boundary) produces "2 segments dropped..."
8. [x] Test Drop tripwire: create reassembler, drop without finalizing, verify eprintln (may require output capture)
9. [ ] Update STATE.md
10. [x] (POST-PASS-1) Add `swap_finalize_skipped_warned_for_testing` seam to break AC-004 false-green vulnerability (F-W11P1-001)
11. [x] (POST-PASS-1) Split AC-007 into engine-cap test (`test_BC_2_04_024_engine_cap_at_exactly_10000`) + HttpAnalyzer/TlsAnalyzer non-cap test (`test_BC_2_04_024_http_tls_analyzer_findings_not_capped`) (F-W11P1-002)
12. [x] (POST-PASS-1) Add EC-006 boundary test (MAX-1 + finalize → 10,000) via `test_BC_2_04_054_finalize_at_max_findings_minus_one` (F-W11P1-003)
13. [x] (POST-PASS-1) Add dropped_findings monotonicity test (BC-2.04.024 EC-004) via `test_BC_2_04_024_dropped_findings_monotone_over_multiple_cap_hits` (F-W11P1-004)
14. [x] (POST-PASS-1) Add small_segment cap-guard test (mod.rs:466) + cite STORY-017 sibling for out-of-window site (mod.rs:495) (F-W11P1-005)
15. [x] (POST-PASS-1) Align FINALIZE_SKIPPED_WARNED_LOCK error-handling to `.expect()` sibling convention (F-W11P1-006)
16. [x] (POST-PASS-1) Drop tautological terminal assert in AC-013; rephrase AC text to remove universal-upper-bound overpromise (F-W11P1-007)
17. [x] (POST-PASS-1) Parameterize segment-limit push test across initial findings.len() ∈ {0, 5000, 9999, 10000} via `test_BC_2_04_054_segment_limit_finding_emitted_regardless_of_initial_count` (F-W11P1-010)
18. [x] (POST-PASS-1) AC-009 plural boundary changed N=42 → N=2 (true plural-boundary) (F-W11P1-011)
19. [x] (POST-PASS-1) AC-003 pin first-finalize close_count == 2 before idempotency check (F-W11P1-014)
20. [x] (POST-PASS-2) Add TlsAnalyzer push_finding_for_testing + all_findings_len_for_testing seams to close AC-007b TlsAnalyzer coverage (F-W11P2-001 + F-W11P2-008)
21. [x] Extend test_BC_2_04_024_http_tls_analyzer_findings_not_capped to exercise both HttpAnalyzer AND TlsAnalyzer in sequence
22. [x] Rewrite AC-004 architecture compliance rule + Previous-Story-Intelligence row to reflect honest scope limit (Option B chosen over Option A ~130+-site lock-spread; 194 was the unverified pre-pass-5 figure) (F-W11P2-002 + F-W11P2-009)
23. [x] Rewrite test_BC_2_04_054_finalize_at_max_findings_minus_one docstring to match actual test body (F-W11P2-003)
24. [x] Fix AC-016 BC EC trace from BC-2.04.054 EC-006 (non-existent) to BC-2.04.054 EC-002 + story EC-006 (F-W11P2-004)
25. [x] Fix test_BC_2_04_024_cap_guard_small_segment_site docstring trace from BC-2.04.024 EC-004 → BC-2.04.024 postconditions 1-2 (F-W11P2-005)
26. [x] Extend EC-010 to enumerate all 5 cap-guard sites with covering tests (F-W11P2-006)
27. [x] Update Task 1 wording from "13 ACs" → "initial 13 ACs (AC-001..AC-013); post-pass-1 ACs added by tasks 11-19" (F-W11P2-007)
28. [x] Re-categorize FINALIZE_SKIPPED_WARNED_LOCK docstring buckets (F-W11P2-011)
29. [x] Sweep STORY-021 line-citations to current source line numbers (F-W11P2-010)
30. [x] (POST-PASS-3 ADDITIONS) Replace stale `mod.rs:677-690` with `mod.rs:793-807` in Token Budget + File Structure Requirements tables (F-W11P3-003)
31. [x] Propagate AC-013 test rename (`...absolute_upper_bound` → `...finalize_bypass_smoke_at_max_findings_representative_scenario`) to AC-013, EC-005, and any other story-side citations (F-W11P3-004b)
32. [x] (POST-PASS-4 ADDITIONS) Replace 3 stale `mod.rs:793-807` citations with `mod.rs:796-810` (worktree post-STORY-021 line numbers) — Architecture Mapping, Token Budget, File Structure Requirements (F-W11P4-001)
33. [x] Add 4 missing test-seam rows to Architecture Mapping table (finalize_skipped_warned_for_testing, reset_..., set_segments_segment_limit_for_testing, push_finding_for_testing) (F-W11P4-003)
34. [x] Append Option-B scope-limitation note to AC-004 body text (F-W11P4-007)
35. [x] Extend AC-013 prose to mention defensive post-bypass idempotency coverage (redundant with AC-003; exercises bypass→idempotency state transition) (F-W11P4-006)
36. [x] Refine FINALIZE_SKIPPED_WARNED_LOCK compliance rule to acknowledge ISN_MISSING_WARNED_LOCK lock-naming outlier as pre-existing codebase inconsistency (F-W11P4-008)
37. [x] (POST-PASS-5 ADDITIONS) Replace "194 sibling un-finalized-drop sites" magic number with orchestrator-verified "~130+ sites" (count derived from 175 TcpReassembler::new − 44 .finalize() upper bound) in Previous-Story-Intelligence row, Architecture Compliance Rule for AC-004, and task 22 description (F-W11P5-004)
38. [x] AC-009 body pinned to "count=2 (true plural-boundary)" matching Architecture Compliance Rule and task 18 — removed ambiguous "N>1" (F-W11P5-006)
39. [x] Reword trust-boundary compliance rule for set_segments_segment_limit_for_testing — drop incorrect BC-2.04.026 EC-002 "violates" citation; replace with accurate "bypasses normal increment path / does NOT violate EC-002" wording (F-W11P5-007)
40. [x] Token budget arithmetic fix: "6 new post-pass-1 tests" → "5 new post-pass-1 ACs: AC-007b, AC-014..AC-017"; test row bumped ~5,000 → ~5,500; total bumped ~18,500 → ~19,000; budget usage ~9.3% → ~9.5% (F-W11P5-009)
41. [x] EC-010 row 4 swap primary/secondary citation for lifecycle.rs:101: STORY-017 sibling test promoted to primary (structurally specific); AC-005 demoted to incidental (F-W11P5-012)
42. [x] (POST-PASS-7 ADDITIONS) Replace `mod.rs:796-810` → `mod.rs:794-808` in story citations (impl Drop shifted up 2 lines after pass-6 docstring shortening) (F-W11P7-001)
43. [x] (POST-PASS-8 ADDITIONS) BC-2.04.012 pre-merge re-anchor doctrine adopted — input-hash bumped to reflect BC-2.04.012 v1.5 (impl Drop citation now worktree-post-STORY-021 794-808); doctrine codification candidate for DF-SIBLING-SWEEP-001 v6 at Wave 11 close (F-W11P8-001, F-W11P8-002)

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-020 | close_flow triggers total_memory decrement | evict_flows uses CloseReason::MemoryPressure | finalize uses CloseReason::Timeout (not MemoryPressure) |
| STORY-019 | close_flow is idempotent for missing keys (one-shot warning) | Each close path has a distinct CloseReason | RST, FIN, Timeout, MemoryPressure are the four close reasons |
| STORY-018 | segments_segment_limit is incremented by the engine on SegmentLimitReached results | Segment limit is per-direction, not per-flow | finalize emits the per-flow aggregate count, not a per-direction count |
| STORY-017 | MAX_FINDINGS cap is enforced at 5 sites (3 in check_anomaly_thresholds, 2 in lifecycle.rs) | latch-before-cap ensures findings are only at cap, not over | finalize is the ONE intentional bypass of the cap |
| STORY-021 | swap_for_testing pattern + honest scope limit: AC-004 asserts Drop hook fires AT LEAST ONCE per process; unique per-Drop attribution is non-deterministic under cargo's parallel scheduler with ~130+ sibling un-finalized-drop sites in tests/reassembly_engine_tests.rs (orchestrator-verified count of 175 TcpReassembler::new sites minus 44 .finalize() calls upper-bounds the un-finalized count; spot-audit not exhaustively performed) | Process-global atomics with global-latch design (one-shot eprintln) cannot be uniquely attributed without process-isolation or stderr capture; document scope honestly | Option A (lock all ~130+ sites) would exceed sibling-sweep cost threshold; Option B (honest docs) chosen |
| (process lesson) | After AC additions to a story, the sibling-sweep step (a) MUST update existing task descriptions referencing AC counts/ranges, not just append new tasks | Codification candidate for DF-SIBLING-SWEEP-001 v5 | Applied in task 27 above; stale "13 ACs" in Task 1 survived pass-1 because the sibling-sweep only checked AC numbering, not task description wording |
| (process lesson) | impl Drop line citation drifted across 4 cycles (677-690 → 793-807 → 796-810 → 794-808) because seam-block edits shift downstream line numbers; story-side citations must be re-verified against source after every test-writer pass that touches mod.rs seams | Codification candidate for DF-SIBLING-SWEEP-001 v6 (story re-anchor on every burst that modifies cited source files) | Applied in task 42 above; citation `796-810` survived passes 4-6 because sibling-sweep did not re-grep impl Drop line after pass-6 docstring shortening |
| (process lesson — closure) | DF-SIBLING-SWEEP-001 v6 candidate REALIZED in pass-8: BC-2.04.012 re-anchored to worktree-post-STORY-021 (794-808) per pre-merge re-anchor doctrine. Orchestrator to codify in policies.yaml at Wave 11 close. | Pre-merge re-anchor doctrine: when a BC's input content changes (e.g. line-citation update), bump story input-hash immediately so drift detection fires on the next pass | F-W11P8-001 resolved |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `self.finalized = true` is set BEFORE the flow-closing loop | BC-2.04.012 invariant 1 | Code review: `self.finalized = true` at mod.rs:561, flow-closing loop at mod.rs:564+ |
| finalize calls `close_flow(key, CloseReason::Timeout, handler)` for remaining flows | BC-2.04.012 postcondition 1 | Test: capture on_flow_close reason; assert == Timeout |
| finalize segment-limit push has NO MAX_FINDINGS guard (unconditional) | BC-2.04.054 invariant 1 | Code review: absence of guard at mod.rs:573 vs guard presence at all other 5 sites |
| `plural_s` helper: returns `""` for count==1, `"s"` otherwise | BC-2.04.025 invariant 3 | Test: AC-009 with count=1 and count=2 |
| `impl Drop` is diagnostic ONLY — it cannot flush flows (no handler argument) | BC-2.04.012 invariant 3 | Code review: Drop::drop signature has no handler |
| `MAX_FINDINGS = 10_000` is the constant; findings.len() <= 10001 after any run | BC-2.04.024 invariant 1; BC-2.04.054 invariant 3 | Test: AC-013 representative-scenario assertion; universal upper-bound proof owned by VP-003 |
| AC-004 assertion scope honestly bounded: tests Drop hook fires at least once per process; unique per-Drop attribution non-deterministic under parallel scheduler (~130+ sibling un-finalized-drop sites in tests/reassembly_engine_tests.rs) — see FINALIZE_SKIPPED_WARNED_LOCK docstring in tests/reassembly_engine_tests.rs | F-W11P2-002 Option B rationale | Code review: AC-004 docstring + FINALIZE_SKIPPED_WARNED_LOCK docstring document the scope limit |
| All `FINALIZE_SKIPPED_WARNED_LOCK` acquisitions use `.expect("FINALIZE_SKIPPED_WARNED_LOCK poisoned")` (fail-fast convention matching CLOSE_FLOW_MISSING_WARNED_LOCK sibling lock; note: ISN_MISSING_WARNED_LOCK uses generic `"test lock poisoned"` message — predates the lock-named convention and is a pre-existing codebase inconsistency) | F-W11P1-006 | Code review: grep for `unwrap_or_else(|e| e.into_inner())` returns no hits in this file |
| `set_segments_segment_limit_for_testing` bypasses the engine's normal increment path (which is `process_packet` → `SegmentLimitReached`); production code MUST NOT call this seam (trust-boundary convention). The seam does NOT violate BC-2.04.026 EC-002 specifically (EC-002 forbids increments *during finalize*, while this seam sets the counter *before* finalize is called). | F-W11P1-009 | Code comment at seam definition; convention enforced by review |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stable toolchain | MSRV 1.85+ | Drop trait, Vec<Finding>, usize constants |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/reassembly/mod.rs` | verify (lines 54, 66-68, 432, 466, 495, 557-591, 794-808) | MAX_FINDINGS const, plural_s, cap guard sites, finalize, impl Drop |
| `src/reassembly/lifecycle.rs` | verify (lines 101, 121) | dropped_findings guard sites in generate_ functions |
| `tests/reassembly_engine_tests.rs` | modify | Add AC-001 through AC-013 (original); AC-007b, AC-014 through AC-017 (post-pass-1) |
| `src/analyzer/http.rs` | verify (test seams) | `push_finding_for_testing`, `all_findings_len_for_testing` — used by AC-007b test |
| `src/analyzer/tls.rs` | modify (additive test seams) | `push_finding_for_testing`, `all_findings_len_for_testing` — added post-pass-2 for AC-007b TlsAnalyzer coverage (F-W11P2-001) |

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-05-14 | Initial story decomposition |
| 1.1 | 2026-05-21 | Brownfield formalization; 13 ACs, full BC traceability, architecture compliance rules, edge case table |
| 1.2 | 2026-05-27 | Post-adversarial-pass-1 remediation (F-W11P1-001 through F-W11P1-015): AC-007 revised to engine-cap boundary assertion; AC-007b added (HttpAnalyzer/TlsAnalyzer cap isolation, `test_BC_2_04_024_http_tls_analyzer_findings_not_capped`); AC-013 rephrased to remove universal-upper-bound overpromise; AC-014 added (dropped_findings monotonicity, N=3); AC-015 added (small_segment cap-guard site mod.rs:466); AC-016 added (EC-006 boundary MAX-1+finalize→10000); AC-017 added (segment-limit push unconditional across initial counts 0/5000/9999/10000); EC table extended to 11 rows (EC-009, EC-010, EC-011); 4 architecture compliance rules added (F-W11P1-001, F-W11P1-006, F-W11P1-009, AC-013 VP-003 delegation); token budget revised to ~18,000; 10 post-pass-1 tasks appended (tasks 10-19); STORY-021 self-learning row added to Previous Story Intelligence; 2 new architecture mapping seam rows added |
| 1.3 | 2026-05-27 | Post-adversarial-pass-2 remediation (F-W11P2-001 through F-W11P2-014): AC-007b clarified to emphasize both HttpAnalyzer AND TlsAnalyzer are exercised; AC-016 trace fixed from non-existent BC-2.04.054 EC-006 → BC-2.04.054 EC-002 + story EC-006 (F-W11P2-004); EC-010 expanded to enumerate all 5 cap-guard sites with covering tests (F-W11P2-006); Task 1 wording updated from "13 ACs" to "initial 13 ACs (AC-001..AC-013)" (F-W11P2-007); STORY-021 Previous-Story-Intelligence self-row replaced: linearizability overclaim → honest Option B scope-limit (F-W11P2-009); process-lesson row added for DF-SIBLING-SWEEP-001 task-description update obligation (F-W11P2-014); Architecture Compliance Rules: stale AC-004 linearizability rule replaced with honest scope-limit rule (F-W11P2-002); `self.finalized = true` line-citation corrected mod.rs:560→561, loop 562+→564+ (F-W11P2-010); impl Drop line-citation corrected mod.rs:677-690→793-807 (F-W11P2-010); TlsAnalyzer test seam row added to Architecture Mapping and File Structure Requirements (F-W11P2-001); token budget updated to ~18,500; 10 post-pass-2 tasks appended (tasks 20-29) |
| 1.4 | 2026-05-27 | Post-adversarial-pass-3 propagation fixes (F-W11P3-003, F-W11P3-004b): replaced remaining stale `mod.rs:677-690` citations with `mod.rs:793-807` in Token Budget table and File Structure Requirements table (F-W11P3-003); propagated AC-013 test rename `test_BC_2_04_054_max_findings_plus_one_is_absolute_upper_bound` → `test_BC_2_04_054_finalize_bypass_smoke_at_max_findings_representative_scenario` to AC-013 Test trace line (F-W11P3-004b); tasks 30-31 appended |
| 1.5 | 2026-05-27 | Post-adversarial-pass-4 story-side remediation (F-W11P4-001, F-W11P4-003, F-W11P4-006, F-W11P4-007, F-W11P4-008): replaced 3 stale `mod.rs:793-807` citations with `mod.rs:796-810` (worktree post-STORY-021 line numbers) across Architecture Mapping, Token Budget, and File Structure Requirements tables (F-W11P4-001); added 4 missing mod.rs test-seam rows to Architecture Mapping (finalize_skipped_warned_for_testing, reset_finalize_skipped_warned_for_testing, set_segments_segment_limit_for_testing, push_finding_for_testing) (F-W11P4-003); appended Option-B scope-limitation note to AC-004 body (F-W11P4-007); extended AC-013 to mention defensive post-bypass idempotency coverage (F-W11P4-006); refined FINALIZE_SKIPPED_WARNED_LOCK compliance rule to acknowledge ISN_MISSING_WARNED_LOCK lock-naming outlier (F-W11P4-008); tasks 32-36 appended |
| 1.6 | 2026-05-27 | Post-adversarial-pass-5 story-side remediation (F-W11P5-004, F-W11P5-006, F-W11P5-007, F-W11P5-009, F-W11P5-012): replaced unverified "194 sibling un-finalized-drop sites" magic number with orchestrator-verified "~130+" in Previous-Story-Intelligence, Architecture Compliance Rule for AC-004, and task 22 description (F-W11P5-004); pinned AC-009 body to "count=2 (true plural-boundary)" matching rule and task 18 (F-W11P5-006); rewrote trust-boundary compliance rule for set_segments_segment_limit_for_testing to drop incorrect BC-2.04.026 EC-002 "violates" claim (F-W11P5-007); fixed token budget test-row arithmetic from "6 new tests" to "5 new ACs (AC-007b, AC-014..AC-017)" and updated total/percentage accordingly (F-W11P5-009); swapped primary/secondary citation order for EC-010 row 4 lifecycle.rs:101 site (F-W11P5-012); tasks 37-41 appended |
| 1.7 | 2026-05-27 | Post-adversarial-pass-7 citation re-anchor (F-W11P7-001): replaced 3 stale `mod.rs:796-810` citations with `mod.rs:794-808` (impl Drop shifted up 2 lines after pass-6 trust-boundary seam docstring shortening) across Architecture Mapping table, Token Budget table, and File Structure Requirements table; process-lesson PSI row added (impl Drop line citation drift pattern across 4 cycles); task 42 appended |
| 1.9 | 2026-05-28 | W11-D1 propagation: BC-2.04.012 v1.6, BC-2.04.025 v1.3, BC-2.04.026 v1.4 replaced bare `—` VP placeholders with explicit N/A markers. These BC-internal VP table changes do not affect any body AC text or architecture anchors in STORY-021 (VP field is frontmatter-only). input-hash bumped 9e32780→220a653 to reflect all three BC content updates. DF-SIBLING-SWEEP-001: grep confirmed no `—` VP placeholders mirrored into STORY-021 body from these BCs; verification_properties: [VP-003] frontmatter unchanged.
| 2.0 | 2026-05-29 | state-manager | input-hash corrected via canonical bin/compute-input-hash --update (prior value `220a653` was hand-computed sha256 over sorted inputs-file list; tool uses MD5 over inputs-order file list). New value: `68dadd4`. |
| 1.8 | 2026-05-27 | Post-pass-8 product-owner remediation (F-W11P8-001, F-W11P8-002): input-hash bumped from edf8559 → 9e32780 to reflect BC-2.04.012 v1.5 content change (impl Drop citation now worktree-post-STORY-021 794-808); pre-merge re-anchor doctrine adopted; PSI process-lesson closure row added; task 43 appended |
