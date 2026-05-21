---
document_type: story
story_id: "STORY-021"
epic_id: "E-2"
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.012.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.024.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.025.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.026.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.054.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
points: "5"
depends_on: [STORY-017, STORY-018, STORY-019, STORY-020]
blocks: [STORY-031]
behavioral_contracts: [BC-2.04.012, BC-2.04.024, BC-2.04.025, BC-2.04.026, BC-2.04.054]
verification_properties: [VP-003]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 11
target_module: reassembly
subsystems: [SS-04]
estimated_days: "1"
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-verify
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
- **Test:** `test_BC_2_04_012_drop_without_finalize_emits_warning()`

### AC-005 (traces to BC-2.04.024 postcondition 1-2)
- When `findings.len() >= MAX_FINDINGS` (= 10,000) and a new finding would normally be emitted by normal packet processing, the finding is NOT pushed and `stats.dropped_findings` increments by 1.
- **Test:** `test_BC_2_04_024_findings_capped_at_max_findings()`

### AC-006 (traces to BC-2.04.024 edge case EC-001)
- When `findings.len() == MAX_FINDINGS - 1` (9,999), the next finding IS added (bringing the count to 10,000).
- **Test:** `test_BC_2_04_024_finding_added_at_one_below_cap()`

### AC-007 (traces to BC-2.04.024 invariant 4)
- HttpAnalyzer and TlsAnalyzer findings are NOT subject to the `MAX_FINDINGS` cap (this cap applies to the reassembly engine only).
- **Test:** `test_BC_2_04_024_max_findings_applies_only_to_reassembly_engine()` (documentation test — asserts the cap constant is local to the reassembly module)

### AC-008 (traces to BC-2.04.025 postcondition 1)
- When `finalize` is called and `stats.segments_segment_limit > 0`, exactly one finding is pushed with: `category: Anomaly`, `verdict: Inconclusive`, `confidence: Medium`, `mitre_technique: None`, and `source_ip: None`, `direction: None`.
- **Test:** `test_BC_2_04_025_finalize_emits_segment_limit_finding()`

### AC-009 (traces to BC-2.04.025 postcondition 1 and invariant 3)
- The summary string uses correct singular/plural grammar: `"1 segment dropped due to per-flow segment count limit"` when count==1, and `"N segments dropped due to per-flow segment count limit"` when count==N (N>1).
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

### AC-013 (traces to BC-2.04.054 invariant 3)
- The maximum possible `findings.len()` after any complete run is `MAX_FINDINGS + 1`. No run can produce more than 10,001 findings from the reassembly engine.
- **Test:** `test_BC_2_04_054_max_findings_plus_one_is_absolute_upper_bound()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| finalize (latch, flow loop, segment-limit block) | src/reassembly/mod.rs:557-591 | effectful-shell |
| impl Drop tripwire | src/reassembly/mod.rs:677-690 | effectful-shell (stderr write) |
| MAX_FINDINGS constant | src/reassembly/mod.rs:54 | pure-core (constant) |
| dropped_findings counter sites | src/reassembly/mod.rs:432,466,495; lifecycle.rs:101,121 | effectful-shell |
| plural_s helper | src/reassembly/mod.rs:66-68 | pure-core |
| segment-limit finding push (unconditional) | src/reassembly/mod.rs:573 | effectful-shell |

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

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reassembly/mod.rs (finalize, impl Drop) | effectful-shell | Mutates self.flows, self.finalized, self.findings; invokes callbacks; writes stderr |
| src/reassembly/mod.rs (plural_s, MAX_FINDINGS const) | pure-core | No I/O; deterministic |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| BC files (5 BCs) | ~5,500 |
| src/reassembly/mod.rs (finalize ~557-591, impl Drop ~677-690, MAX_FINDINGS ~54, dropped_findings sites ~432,466,495) | ~2,500 |
| src/reassembly/lifecycle.rs (dropped_findings sites ~101,121) | ~500 |
| Test files | ~3,500 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~15,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~7.75%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for all 13 ACs in `tests/reassembly_engine_tests.rs`
2. [ ] Verify Red Gate: all tests fail before implementation changes
3. [ ] Verify existing implementation satisfies all ACs (brownfield)
4. [ ] Test finalize idempotency: call twice; assert on_flow_close callbacks fire exactly N times
5. [ ] Test MAX_FINDINGS boundary: fill to 9999, verify 10000th is accepted; fill to 10000, verify 10001st is dropped
6. [ ] Test finalize bypass: fill findings to 10000 (MAX); trigger segment limit; call finalize; assert findings.len()==10001
7. [ ] Test singular/plural: segments_segment_limit=1 produces "1 segment dropped..."; limit=2 produces "2 segments dropped..."
8. [ ] Test Drop tripwire: create reassembler, drop without finalizing, verify eprintln (may require output capture)
9. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-020 | close_flow triggers total_memory decrement | evict_flows uses CloseReason::MemoryPressure | finalize uses CloseReason::Timeout (not MemoryPressure) |
| STORY-019 | close_flow is idempotent for missing keys (one-shot warning) | Each close path has a distinct CloseReason | RST, FIN, Timeout, MemoryPressure are the four close reasons |
| STORY-018 | segments_segment_limit is incremented by the engine on SegmentLimitReached results | Segment limit is per-direction, not per-flow | finalize emits the per-flow aggregate count, not a per-direction count |
| STORY-017 | MAX_FINDINGS cap is enforced at 5 sites (3 in check_anomaly_thresholds, 2 in lifecycle.rs) | latch-before-cap ensures findings are only at cap, not over | finalize is the ONE intentional bypass of the cap |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `self.finalized = true` is set BEFORE the flow-closing loop | BC-2.04.012 invariant 1 | Code review: finalized set at mod.rs:560, loop at mod.rs:562+ |
| finalize calls `close_flow(key, CloseReason::Timeout, handler)` for remaining flows | BC-2.04.012 postcondition 1 | Test: capture on_flow_close reason; assert == Timeout |
| finalize segment-limit push has NO MAX_FINDINGS guard (unconditional) | BC-2.04.054 invariant 1 | Code review: absence of guard at mod.rs:573 vs guard presence at all other 5 sites |
| `plural_s` helper: returns `""` for count==1, `"s"` otherwise | BC-2.04.025 invariant 3 | Test: AC-009 with count=1 and count=2 |
| `impl Drop` is diagnostic ONLY — it cannot flush flows (no handler argument) | BC-2.04.012 invariant 3 | Code review: Drop::drop signature has no handler |
| `MAX_FINDINGS = 10_000` is the constant; findings.len() <= 10001 after any run | BC-2.04.024 invariant 1; BC-2.04.054 invariant 3 | Test: AC-013 upper-bound assertion |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stable toolchain | MSRV 1.85+ | Drop trait, Vec<Finding>, usize constants |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/reassembly/mod.rs` | verify (lines 54, 66-68, 432, 466, 495, 557-591, 677-690) | MAX_FINDINGS const, plural_s, cap guard sites, finalize, impl Drop |
| `src/reassembly/lifecycle.rs` | verify (lines 101, 121) | dropped_findings guard sites in generate_ functions |
| `tests/reassembly_engine_tests.rs` | modify | Add AC-001 through AC-013 |
