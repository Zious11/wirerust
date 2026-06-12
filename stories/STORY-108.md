---
document_type: story
story_id: STORY-108
epic_id: E-15
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-06-10T00:00:00Z
phase: 3
points: 13
priority: P0
depends_on: [STORY-107]
blocks: [STORY-109]
behavioral_contracts:
  - BC-2.15.010
  - BC-2.15.011
  - BC-2.15.012
  - BC-2.15.013
  - BC-2.15.020
  - BC-2.15.022
verification_properties:
  - VP-023
tdd_mode: strict
target_module: analyzer/dnp3
subsystems: [SS-15]
wave: 37
estimated_days: 5
feature_id: issue-008-dnp3-analyzer
github_issue: 8
# BC status: 6 BCs authored 2026-06-10; latest versions BC-2.15.010 v1.2, BC-2.15.011 v1.2, BC-2.15.013 v1.1
inputs:
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.010.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.011.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.012.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.013.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.020.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.022.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
  - .factory/specs/verification-properties/vp-023-dnp3-parse-safety.md
input-hash: "44e3256"
---

# STORY-108: DNP3 Direct Detection Emissions — T1692.001, T0814 (Restart), T0836, Co-Emission, Summarize

## Narrative

- **As a** ICS/OT security analyst using wirerust against DNP3 TCP captures
- **I want** the DNP3 analyzer to detect unauthorized control command bursts (T1692.001), device restarts (T0814), parameter writes (T0836), emit findings in most-specific order with a DoS cap, and produce a function-code distribution summary
- **So that** the analyst receives actionable ICS threat findings for the three primary DNP3 MITRE technique categories with correct emission ordering, timing windows, and bounded memory use

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.15.010 | Unauthorized Control Command — Unexpected Source (count=1) or Control-Class FC Exceeding Threshold Emits T1692.001 |
| BC-2.15.011 | COLD_RESTART/WARM_RESTART Observed — Emits T0814 Per-Occurrence Finding |
| BC-2.15.012 | WRITE FC Observed — Emits T0836 Modify-Parameter Finding Per-Occurrence |
| BC-2.15.013 | Co-Emission Ordering — Direct Finding (T0814/T1692.001) Precedes Derived T0827 |
| BC-2.15.020 | summarize() Emits Function-Code Distribution and Control-Operation Counts |
| BC-2.15.022 | MAX_FINDINGS DoS Bound — Finding Cap Prevents Unbounded all_findings Growth |

## Acceptance Criteria

### AC-001 (traces to BC-2.15.010 postcondition 1/2 — counter increment and window seed)
Every Control-class FC (0x03, 0x04, 0x05, 0x06) on a FIR=1 frame increments `flow.direct_operate_count`. On the first Control-class FC in a new window, `flow.window_start_ts = now_ts`.
- **Test:** `test_direct_operate_count_increments_on_control_fc()`

### AC-002 (traces to BC-2.15.010 postcondition 3 — T1692.001 finding emission on threshold exceed)
When `flow.direct_operate_count > self.direct_operate_threshold` AND `flow.direct_operate_emitted == false` AND window NOT expired (`now_ts.wrapping_sub(flow.window_start_ts) <= DETECTION_WINDOW_SECS`): exactly ONE `Finding` is pushed with `mitre_techniques: vec!["T1692.001"]`, `category: Execution`, `verdict: Likely`, `confidence: Medium`. Summary: `"DNP3 unauthorized control command burst: {count} control FCs in {elapsed}s window (threshold {threshold})"`. `flow.direct_operate_emitted = true`.
- **Default threshold = 10; DETECTION_WINDOW_SECS = 60.**
- **Test:** `test_t1692_001_emitted_at_threshold_plus_one()` — 11 DIRECT_OPERATE FCs; assert finding emitted at count=11, not at count=10.

### AC-003 (traces to BC-2.15.010 postcondition 3 — one-shot guard)
After `direct_operate_emitted=true`, additional Control-class FCs in the same window increment the counter but do NOT push additional T1692.001 findings.
- **Test:** `test_t1692_001_one_shot_guard()` — 16 total Control FCs; assert exactly 1 finding in all_findings.

### AC-004 (traces to BC-2.15.010 postcondition 4 — window expiry reset)
When `now_ts.wrapping_sub(flow.window_start_ts) > DETECTION_WINDOW_SECS (60s)`: `direct_operate_count = 1` (incoming FC seeds new window), `window_start_ts = now_ts`, `direct_operate_emitted = false`.
- **Test:** `test_t1692_001_window_expiry_resets_counter()` — emit finding in window 1; advance time past 60s; send new Control FCs; assert second finding possible.

### AC-005 (traces to BC-2.15.011 postconditions 1/2 — T0814 per-occurrence for restart)
Every COLD_RESTART (0x0D) or WARM_RESTART (0x0E) on a FIR=1 frame pushes ONE `Finding` with `mitre_techniques: vec!["T0814"]`, `category: Execution`, `verdict: Likely`, `confidence: High`. Summary: `"DNP3 restart command observed: FC 0x{fc:02X} ({name}) from src={src:#06X} to dest={dest:#06X}"`. ADDITIONALLY `flow.restart_event_count += 1` unconditionally.
- **FC 0x0F (INITIALIZE_DATA) does NOT trigger T0814** — `classify_dnp3_fc(0x0F)` = Management.
- **Test:** `test_t0814_emitted_per_occurrence_cold_restart()`, `test_t0814_emitted_per_occurrence_warm_restart()`, `test_initialize_data_not_restart()`

### AC-006 (traces to BC-2.15.012 postcondition 1 — T0836 per-occurrence for WRITE)
Every WRITE (FC=0x02) on a FIR=1 frame pushes ONE `Finding` with `mitre_techniques: vec!["T0836"]`, `category: Execution`, `verdict: Likely`, `confidence: Medium`. Summary: `"DNP3 WRITE command observed: parameter modification from src={src:#06X} to dest={dest:#06X}"`. T0836 only (NOT also T1692.001 — WRITE is Write-class, not Control-class).
- **Test:** `test_t0836_emitted_for_write_fc()`, `test_write_fc_not_t1692()`

### AC-007 (traces to BC-2.15.013 postconditions 2/3 — co-emission ordering: direct before derived)
When multiple T0814 findings are pushed across successive `on_data` calls, they appear in the `all_findings` vec in the inter-call order they were observed (first restart finding appended before subsequent restart findings). Intra-call ordering (T0814 pushed before derived T0827 within the same `on_data` call — BC-2.15.013 PC2) and the mid-sequence cap re-check (PC4/PC5) are DEFERRED to STORY-109 where the T0827 derived push is implemented.
- **Test:** `test_restart_findings_append_in_observation_order()` — verifies inter-call append ordering of T0814 findings across successive `on_data` calls.

### AC-008 (traces to BC-2.15.013 postcondition 4/5 — MAX_FINDINGS cap applied per-push)
`MAX_FINDINGS` cap check is `self.all_findings.len() < MAX_FINDINGS` evaluated immediately before each `push`. When cap is hit mid-multi-finding sequence, the first (most specific) finding is preserved and subsequent findings are silently dropped.
- **Test:** `test_max_findings_cap_preserves_first_finding()` — fill all_findings to MAX_FINDINGS-1; deliver COLD_RESTART that would produce [T0814, T0827]; assert T0814 pushed (cap consumed), T0827 dropped.

### AC-009 (traces to BC-2.15.022 postconditions 1/3 — cap; counters still updated)
When `all_findings.len() >= MAX_FINDINGS`, no new Finding is pushed. Per-flow counters (`direct_operate_count`, `restart_event_count`, `fc_counts`, `fn_code_counts`, `frame_count`) ARE still updated regardless of cap.
- **Test:** `test_max_findings_counters_updated_when_capped()`

### AC-010 (traces to BC-2.15.020 postcondition 1 — summarize output fields)
`Dnp3Analyzer::finalize()` or `summarize()` produces output including: `function_code_distribution` (map of FC byte to count, only FCs with count>0), `control_operation_counts` (per-flow `direct_operate_count`), `total_frames`, `total_parse_errors`, `flows_analyzed`. Present even when zero flows analyzed (zero counts, not absent).
- **Test:** `test_summarize_function_code_distribution()` — process 5 DIRECT_OPERATE + 3 READ; assert `fn_code_counts = {0x05: 5, 0x01: 3}`.

### AC-011 (traces to BC-2.15.020 invariant 4 — zero-flow case)
When no DNP3 flows analyzed, summary is present with zero counts, not absent from output.
- **Test:** `test_summarize_zero_flows()`

### AC-012 (traces to BC-2.15.010 invariant 5 — 10/60s is flood guard not primary authz check; unexpected source fires at count=1)
CONFIRMED threshold values: default `direct_operate_threshold = 10` (count > 10), `DETECTION_WINDOW_SECS = 60`. Control FC from an UNEXPECTED source address fires at count=1 (source-address allowlist check is the primary gate; 10/60s is the secondary volumetric flood guard per BC-2.15.010 Invariant 5).
- **Note:** Source-address allowlist is a runtime/config concern; the threshold > 10 check is the detectable behavior in unit tests.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `Dnp3FlowState.direct_operate_count: u32` | `src/analyzer/dnp3.rs` | Effectful shell state |
| `Dnp3FlowState.window_start_ts: u32` | `src/analyzer/dnp3.rs` | Effectful shell state |
| `Dnp3FlowState.direct_operate_emitted: bool` | `src/analyzer/dnp3.rs` | One-shot guard |
| `Dnp3FlowState.restart_event_count: u64` | `src/analyzer/dnp3.rs` | Effectful shell state (feeds T0827 in STORY-109) |
| `Dnp3FlowState.fc_counts: HashMap<u8, u64>` | `src/analyzer/dnp3.rs` | Per-flow FC distribution |
| `Dnp3Analyzer.fn_code_counts: HashMap<u8, u64>` | `src/analyzer/dnp3.rs` | Aggregate FC distribution |
| `Dnp3Analyzer.all_findings: Vec<Finding>` | `src/analyzer/dnp3.rs` | Effectful shell output |
| `Dnp3Analyzer.direct_operate_threshold: u32` | `src/analyzer/dnp3.rs` | Config (set by CLI in STORY-110) |
| `Dnp3Analyzer::finalize()` / `summarize()` | `src/analyzer/dnp3.rs` | Effectful shell |
| `src/mitre.rs` technique_info entries | `src/mitre.rs` | Catalog lookup; T1692.001, T0814, T0836 already seeded in STORY-100 |
| `const MAX_FINDINGS: usize` | `src/analyzer/dnp3.rs` | Shared cap constant |
| `const DETECTION_WINDOW_SECS: u32 = 60` | `src/analyzer/dnp3.rs` | Window constant |

Architecture section references: `architecture/module-decomposition.md` (SS-15 detection branches), ADR-007 Decision 5 (technique-to-FC mapping).

## VP-007 Atomic Update Note (SEEDED/EMITTED counts)

T1692.001, T0814, T0836 were ALREADY added to the `SEEDED_TECHNIQUE_IDS` slice (and the kani_proofs `EMITTED_IDS` set, `#[cfg(kani)]`) in `src/mitre.rs` by STORY-100 (multi-tag schema migration). Verify that `technique_info("T1692.001")`, `technique_info("T0814")`, and `technique_info("T0836")` entries exist in `src/mitre.rs` before implementing detection branches. No new seeding needed in this story for these three techniques.

**New techniques required for STORY-109**: T1691.001 (`Block Operational Technology Message: Command Message`; tactic `IcsInhibitResponseFunction`) and T0827 (`Loss of Control`; tactic `IcsImpact` — NEW `MitreTactic::IcsImpact` variant) must be added to `src/mitre.rs` SEEDED_TECHNIQUE_IDS. These are NOT added in this story — they are added in STORY-109 where those findings are first emitted.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DIRECT_OPERATE_NR (0x06) — counts toward threshold? | Yes — FC 0x06 is `Control`-class per BC-2.15.006; increments `direct_operate_count` |
| EC-002 | Control FC at exactly threshold (count=10, threshold=10) | No finding — threshold check is `>`, not `>=`; 10 > 10 is false |
| EC-003 | FC 0x0F (INITIALIZE_DATA) | `Management`-class; no T0814 (restart detection only for 0x0D, 0x0E) |
| EC-004 | WRITE (0x02) to broadcast dest 0xFFFF | T0836 emitted; no broadcast anomaly (that's BC-2.15.018 in STORY-109; WRITE is Write-class not Control-class) |
| EC-005 | Two COLD_RESTARTs on same flow | Two T0814 findings; `restart_event_count=2` |
| EC-006 | `all_findings.len()==MAX_FINDINGS` when COLD_RESTART | No T0814 pushed; `restart_event_count` still incremented |
| EC-007 | FC=0x05 then FC=0x02 on same FIR=1 flow | Two separate findings: T1692.001 (if threshold reached) and T0836; never co-tagged (FC cannot be both Control and Write simultaneously) |
| EC-008 | `now_ts.wrapping_sub(window_start_ts)` overflow | Uses wrapping subtraction; safe for u32 timestamps going backward (out-of-order pcap replay) |

## Tasks

1. **Add detection-state fields to `Dnp3FlowState`**: `direct_operate_count: u32`, `window_start_ts: u32`, `direct_operate_emitted: bool`, `restart_event_count: u64`, `fc_counts: HashMap<u8, u64>`.
2. **Add `Dnp3Analyzer` fields**: `fn_code_counts: HashMap<u8, u64>`, `all_findings: Vec<Finding>`, `direct_operate_threshold: u32` (default 10), `flows: HashMap<FlowKey, Dnp3FlowState>`.
3. **Implement Control-class detection branch** (BC-2.15.010) in `on_data` — increment counter, check `now_ts.wrapping_sub(window_start_ts)`, emit T1692.001 when count > threshold and guard not set.
4. **Implement Restart detection branch** (BC-2.15.011) — check `classify_dnp3_fc == Restart`; push T0814 with `confidence: High`; increment `restart_event_count`.
5. **Implement Write detection branch** (BC-2.15.012) — check `classify_dnp3_fc == Write`; push T0836 with `confidence: Medium`.
6. **Add `MAX_FINDINGS` cap check** before every push (BC-2.15.022). Evaluate `self.all_findings.len() < MAX_FINDINGS` immediately before each `push`.
7. **Implement co-emission ordering** (BC-2.15.013) — direct detection pushes happen before any derived-impact push within the same `on_data` call. Leave placeholder stub for T0827 (STORY-109 fills it in).
8. **Implement `finalize()` / `summarize()`** (BC-2.15.020) — aggregate `fn_code_counts`, per-flow `direct_operate_count`, total_frames, total_parse_errors, flows_analyzed into output struct.
9. **Add `DETECTION_WINDOW_SECS: u32 = 60`** and `const MAX_FINDINGS` constants.
10. **Unit tests** for AC-001 through AC-012.

## Test Plan

| AC | Test Type | Notes |
|----|-----------|-------|
| AC-001 | Unit | Counter increment |
| AC-002 | Unit | T1692.001 at threshold+1 (default=10, fires at 11th FC) |
| AC-003 | Unit | One-shot guard; exactly 1 finding after 16 Control FCs |
| AC-004 | Unit | Window expiry reset; wrapping_sub used |
| AC-005 | Unit | T0814 per restart; restart_event_count; FC 0x0F excluded |
| AC-006 | Unit | T0836 for WRITE; NOT also T1692.001 |
| AC-007 | Unit | Ordering: direct before derived (stub) |
| AC-008 | Unit | MAX_FINDINGS cap at N-1 in multi-finding sequence |
| AC-009 | Unit | Counters updated when findings capped |
| AC-010 | Unit + Integration | summarize() FC distribution map; 5 DIRECT_OPERATE + 3 READ |
| AC-011 | Unit | summarize() zero-flow case |
| AC-012 | Unit | Threshold semantics: `> 10` not `>= 10` |

## Previous Story Intelligence

STORY-104 (Modbus Detection Emissions, E-14) is the direct structural precedent:
- Same pattern: stateful detection branches inside `on_data`; findings pushed to `all_findings`; `MAX_FINDINGS` cap; per-flow window/counter state.
- Key DNP3 differences: (1) T1692.001 replaces revoked T0855 — do NOT use T0855; (2) DNP3 restart is per-occurrence (no burst threshold) unlike Modbus write-burst; (3) T0836 on DNP3 carries only T0836 (NOT also T1692.001 — DNP3 WRITE and Control are distinct FC classes); (4) `restart_event_count` must be incremented even when `all_findings` is capped (feeds T0827 in STORY-109).
- Lesson from STORY-104: implement the `MAX_FINDINGS` check before EACH `push`, not once at frame entry. A single COLD_RESTART frame may want to push both T0814 and T0827 (STORY-109); the cap check must be re-evaluated before the second push.
- `wrapping_sub` is required for all timestamp arithmetic — overflow-checks=true in the release profile causes panics on plain subtraction with out-of-order pcap replay.

## Architecture Compliance Rules

Derived from ADR-007 Decision 5 and architecture BC cross-references:
1. **T1692.001 replaces T0855** — T0855 is revoked in ics-attack-19.1. Emit `"T1692.001"` only.
2. **T0836 on DNP3 WRITE carries T0836 only** — NOT also T1692.001. (Modbus write-class gets both; DNP3 WRITE is explicitly separated per ADR-007 Decision 5.)
3. **`restart_event_count` incremented unconditionally** — even when `all_findings.len() >= MAX_FINDINGS` (no finding pushed). This counter feeds the T0827 accumulator in STORY-109 and must not be gated behind the findings cap.
4. **`wrapping_sub` for all u32 timestamp arithmetic** — never plain subtraction.
5. **co-emission ordering in one `on_data` call**: direct finding first, derived (T0827) second. `MAX_FINDINGS` cap evaluated per-push.
6. **Forbidden dependencies**: `src/analyzer/dnp3.rs` MUST NOT depend on `src/analyzer/modbus.rs`. If it gains that dependency, the build MUST fail.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `std::collections::HashMap` | stdlib | `fc_counts`, `fn_code_counts`, `flows` |
| `src/mitre.rs` | same crate | `technique_info("T1692.001")`, `technique_info("T0814")`, `technique_info("T0836")` — must be pre-seeded by STORY-100 |
| No external crate | — | ADR-007 Decision 2 |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/analyzer/dnp3.rs` | Modify | Add detection-state fields; implement detection branches; finalize(); MAX_FINDINGS cap |
| `src/mitre.rs` | Verify (no change) | Confirm T1692.001, T0814, T0836 entries present from STORY-100 |
| `tests/dnp3_detection_tests.rs` OR inline | Create/expand | AC-001..AC-012 unit tests |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~4,000 |
| 6 BC files (BC-2.15.010..013, 020, 022) | ~22,000 |
| ADR-007 (Decision 5 detection table) | ~3,000 |
| VP-023 (precondition verification context) | ~1,500 |
| STORY-107 (prior story) | ~2,500 |
| STORY-104 (Modbus precedent) | ~3,000 |
| Existing `src/analyzer/dnp3.rs` | ~3,500 |
| Tool outputs | ~2,000 |
| **Total estimated** | **~41,500** |

Within 20-30% of agent context window (~120k). This is the largest story in the E-15 epic and sits at the upper end of the sizing budget.

## Dependency Rationale

- `depends_on: [STORY-107]` — STORY-107 completes `Dnp3FlowState` struct and the carry-buffer/frame-consume loop. Detection branches fire INSIDE the per-frame processing loop; they cannot be added before that loop exists.
- `blocks: [STORY-109]` — STORY-109 (correlated/derived detections) adds T1691.001 and T0827 emission which reads `block_event_count` and `restart_event_count` accumulated here. The T0827 accumulator (reads `restart_event_count` set in AC-005) and the correlated window (BC-2.15.015) both depend on state fields introduced in this story.
