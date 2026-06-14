---
document_type: story
story_id: STORY-115
epic_id: E-16
version: "1.1"
# Pass-32: align analyzer field name storm_findings_count→storm_findings (matches STORY-113 declaration + sibling convention + BC-2.16.010 summarize key)
status: draft
producer: story-writer
timestamp: 2026-06-13T00:00:00Z
phase: f3
points: 8
priority: P0
depends_on: [STORY-114]
blocks: []
behavioral_contracts:
  - BC-2.16.008
  - BC-2.16.013
verification_properties: []
tdd_mode: strict
target_module: analyzer/arp
subsystems: [SS-16]
estimated_days: 3
feature_id: issue-009-arp-security-analyzer
github_issue: 9
# BC status: BC-2.16.008 v1.6, BC-2.16.013 v1.1 — authored 2026-06-12
# BC-2.16.010 cross-story extension: wires storm_findings VALUE (key already defined by STORY-113); primary owner is STORY-113
# No Kani/proptest for D3: unit-tested only. T0814 MITRE tag deferred per DF-VALIDATION-001.
# NOTE: verification_properties: [] because D3 is not a VP-024 formal target; no new VP in this story.
inputs:
  - .factory/specs/architecture/arp-architecture-delta.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.008.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.013.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.010.md
input-hash: "5ca9835"
---

# STORY-115: D3 ARP Storm Detection + --arp-storm-rate CLI Flag + storm_findings Summary Key

## Narrative

- **As a** ICS/OT security analyst using wirerust in an OT/ICS environment
- **I want** `ArpAnalyzer` to detect ARP packet storms by tracking per-MAC frame rate using a per-MAC 60-second flap-window counter, emitting a MEDIUM/Anomaly finding when a source MAC's average-frames-per-second-since-window-start reaches or exceeds the threshold, and to be able to lower the threshold via `--arp-storm-rate` (since ICS devices operate at much lower ARP rates than the 50/s default)
- **So that** high-volume ARP flooding behavior is surfaced as a security anomaly for OT analysts, and the ARP security feature is fully complete for the v0.7.0 release

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.16.008 | D3 ARP Storm Rate Detection — Source MAC Exceeds ARP_STORM_RATE_DEFAULT Frames/Sec |
| BC-2.16.013 | --arp-storm-rate Overrides ARP_STORM_RATE_DEFAULT |

## BC-2.16.010 Cross-Story Extension

This story wires the VALUE of the existing `storm_findings` key in `ArpAnalyzer::summarize()`. The primary owner
of BC-2.16.010 and the 11-key summarize contract is STORY-113. STORY-115 does NOT add a new key — the `storm_findings` key is canonical key 8 of BC-2.16.010's 11-key set, already defined by BC-2.16.010 and declared by STORY-113's `summarize()` with value 0. This story populates its VALUE from `ArpAnalyzer.storm_findings` so it becomes non-zero when D3 detections fire.

## D3 MITRE Attribution — DF-VALIDATION-001 Compliance

D3 storm findings MUST have `mitre_techniques: []` (empty). T0814 (Denial of Service, ICS) is
a possible mapping but has NOT been validated live against attack.mitre.org per project policy
DF-VALIDATION-001 as of 2026-06-12. The D3 finding emits an empty `mitre_techniques: []`.
This is a documented human decision point. Do NOT add T0814 to D3 findings or to
`src/mitre.rs` SEEDED/EMITTED arrays without a validated live research pass.

## Acceptance Criteria

### AC-001 (traces to BC-2.16.008 postcondition 1 — first-observation initializes StormCounter)
When a source MAC is seen for the first time, a new `StormCounter` is initialized:
`count_in_window = 1`, `window_start_ts = timestamp_secs`, `storm_emitted = false`.
Step 3 rate evaluation proceeds with count=1 (no step 2 increment for first observation).
No storm finding is emitted if count=1 < storm_rate.
- **Test:** `test_storm_first_observation_no_finding()` (default storm_rate=50; count=1 < 50)

### AC-002 (traces to BC-2.16.008 postcondition 2 — window-active increment)
When the source MAC is already in `storm_counters` AND `timestamp_secs - window_start_ts <= ARP_FLAP_WINDOW_SECS`, `count_in_window` is incremented by 1 (Step 2) before rate evaluation (Step 3).
- **Test:** `test_storm_in_window_increments_count()`

### AC-003 (traces to BC-2.16.008 postcondition 3 — storm finding emitted at threshold)
When `count_in_window / max(1, timestamp_secs - window_start_ts) >= storm_rate` AND
`storm_emitted == false`, a `Finding` is emitted with `confidence: MEDIUM`, `finding_type: Anomaly`,
description indicating ARP storm / high ARP frame rate from source MAC, `mitre_techniques: []`
(empty — T0814 withheld per DF-VALIDATION-001). Evidence includes: `source_mac`, `frame_count`,
`window_secs`, `rate_pps`.
- **Test:** `test_storm_finding_emitted_at_threshold()` (50 frames all at ts=100 → rate=50/1=50 >= 50 → finding)

### AC-004 (traces to BC-2.16.008 postcondition 4 — one-shot guard)
After `storm_emitted == true`, subsequent frames from the same MAC in the same window do NOT
trigger additional storm findings.
- **Test:** `test_storm_one_shot_guard_prevents_second_finding()`

### AC-005 (traces to BC-2.16.008 postcondition 1/Step 1 — window expiry resets counter)
When `timestamp_secs - window_start_ts > ARP_FLAP_WINDOW_SECS = 60`, the window resets:
`count_in_window = 1`, `window_start_ts = timestamp_secs`, `storm_emitted = false`. The reset
frame is treated as a first-time observation for that MAC in the new window.
- **Test:** `test_storm_window_expiry_resets_counter()`

### AC-006 (traces to BC-2.16.008 postcondition 6, Note — rate formula: max(1, ts - window_start_ts))
The rate formula is `count_in_window / max(1, timestamp_secs - window_start_ts)`. When
`timestamp_secs == window_start_ts` (same integer second), denominator = `max(1, 0) = 1`.
50 frames at ts=100 → rate = 50/1 = 50 >= 50 → storm finding.
- **Test:** `test_storm_same_second_denominator_is_1()` (verifies max(1,0)=1 avoids divide-by-zero and triggers correctly)

### AC-007 (traces to BC-2.16.008 EC-001/EC-002 — below and at threshold)
49 frames all at ts=100 → rate=49/1=49 < 50 → no storm finding.
50 frames all at ts=100 → rate=50/1=50 >= 50 → storm finding.
- **Test:** `test_storm_49_below_threshold_50_at_threshold()`

### AC-008 (traces to BC-2.16.008 EC-009/EC-010 — window boundary: <= 60 in-window, 61 expired)
Frame at ts=160, window_start_ts=100: elapsed=60 <= 60 → still in-window; no reset.
Frame at ts=161, window_start_ts=100: elapsed=61 > 60 → window expired; reset.
- **Test:** `test_storm_window_boundary_60_in_window_61_expired()`

### AC-009 (traces to BC-2.16.008 EC-011 — late-burst suppression accepted limitation)
49 frames at ts=100, then 50 more frames at ts=159 (same window, window_start_ts=100):
at ts=159, count=99, elapsed=59, rate=99/59≈1.68 < 50. No storm finding despite the burst.
This is the documented average-since-window-start limitation (BC-2.16.008 Invariant 2).
- **Test:** `test_storm_late_burst_suppression_accepted_limitation()` (documents the behavior; asserts no finding)

### AC-010 (traces to BC-2.16.008 postcondition 5 — storm counter cap: MAX_STORM_COUNTERS=4096)
`storm_counters.len()` NEVER exceeds `MAX_STORM_COUNTERS = 4_096`. LRU eviction analogous to
binding table. A 4097th distinct MAC's counter is inserted after evicting the oldest entry.
- **Test:** `test_storm_counter_cap_enforced()` (4097 distinct MACs; assert len ≤ 4096)

### AC-011 (traces to BC-2.16.013 postcondition 1/2 — --arp-storm-rate wiring)
`ArpAnalyzer::new(spoof_threshold, storm_rate)` uses the `storm_rate` parameter in D3
detection. `src/cli.rs` declares `#[arg(long, default_value_t = 50)] arp_storm_rate: u32`
on `Commands::Analyze`. `src/main.rs` passes `args.arp_storm_rate` to `ArpAnalyzer::new`.
When flag is absent, default 50 applies. When `--arp-storm-rate 10` is set, storm triggers
at 10 frames/sec.
- **Test:** `test_cli_arp_storm_rate_parsed()`, `test_cli_arp_storm_rate_default_50()`, `test_storm_custom_rate_10()`

### AC-012 (traces to BC-2.16.013 EC-006 — flag accepted without --arp)
`--arp-storm-rate N` is accepted by the CLI without `--arp` (no parse error). When `--arp`
is absent, the flag has no effect (process_arp is not called).
- **Test:** `test_storm_rate_flag_accepted_without_arp_flag()`

### AC-013 (traces to BC-2.16.010 cross-story extension — storm_findings key non-zero after D3 detection)
After a D3 storm finding is emitted, `ArpAnalyzer::summarize()["storm_findings"] > 0`.
The existing eleven-key contract from BC-2.16.010 (primary owner: STORY-113) is unchanged.
`storm_findings` was 0 in STORY-113; this story makes it reflect actual D3 detection count.
- **Note (cross-story extension):** BC-2.16.010 is NOT in this story's `behavioral_contracts:` frontmatter — STORY-113 is the primary owner. This AC extends STORY-113's `summarize()` implementation without re-contracting ownership. The cross-story extension is intentional per the BC-2.16.010 cross-story extension section above.
- **Test:** `test_summarize_storm_findings_key_non_zero_after_detection()`

### AC-014 (traces to BC-2.16.008 — D3 finding has empty mitre_techniques)
D3 storm findings have `mitre_techniques: []`. Verify that T0814 is NOT present in any D3
finding's MITRE list. This is the DF-VALIDATION-001 compliance requirement.
- **Test:** `test_d3_finding_has_empty_mitre_techniques()`

### AC-015 (traces to BC-2.16.008 — integration test for storm rate end-to-end)
An integration test exercises the full CLI pipeline: `wirerust analyze --arp --arp-storm-rate 10 <pcap>` where the pcap contains a synthetic ARP storm (≥10 frames/sec from one source MAC). The output includes a D3 storm finding with MEDIUM confidence and empty MITRE list.
- **Test:** `test_integration_arp_storm_end_to_end()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `storm_counters: HashMap<[u8; 6], StormCounter>` (full impl) | `src/analyzer/arp.rs` | Pure core (stateful) |
| D3 storm rate detection logic in `process_arp` | `src/analyzer/arp.rs` | Pure core (stateful) |
| `const ARP_STORM_RATE_DEFAULT: u32 = 50` | `src/analyzer/arp.rs` | Constant (wirerust engineering default) |
| `const MAX_STORM_COUNTERS: usize = 4_096` | `src/analyzer/arp.rs` | Constant |
| `storm_findings` counter field | `src/analyzer/arp.rs` | State field |
| `summarize()` `storm_findings` key wiring | `src/analyzer/arp.rs` | Pure read-only aggregation |
| `--arp-storm-rate: u32` CLI flag | `src/cli.rs` | Effectful shell (CLI) |
| `src/main.rs` — `ArpAnalyzer::new(spoof_threshold, storm_rate)` | `src/main.rs` | Effectful shell |

Architecture section references: `architecture/module-decomposition.md` (SS-16 C-23 ArpAnalyzer), arp-architecture-delta.md §3.1–§3.2.

## Forbidden Dependencies

- D3 storm detection MUST NOT add T0814 to `src/mitre.rs` or to any finding's `mitre_techniques`. T0814 is deferred per DF-VALIDATION-001. Violating this introduces an unvalidated MITRE technique reference that will fail future holdout evaluation.
- `src/analyzer/arp.rs` MUST NOT gain a dependency on `src/dispatcher.rs` (ArpAnalyzer is not a StreamAnalyzer; unchanged from prior stories).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First MAC observation | count=1; rate=1/1=1 < 50; no storm (unless storm_rate=1) |
| EC-002 | 50 frames, all at same ts (ts==window_start_ts) | rate=50/max(1,0)=50/1=50 >= 50 → storm finding |
| EC-003 | 49 frames at ts=100 | rate=49/1=49 < 50 → no storm finding |
| EC-004 | storm_emitted=true; 51st frame | One-shot guard: no additional storm finding |
| EC-005 | Window expired (elapsed > 60); reset | count=1; storm_emitted=false; no finding if count < rate |
| EC-006 | ts - window_start_ts == 60 (boundary) | Still in-window (≤ 60); no reset; rate diluted |
| EC-007 | ts - window_start_ts == 61 | Window expired; reset |
| EC-008 | Late-burst suppression: 49 at ts=100, 50 at ts=159 | rate=99/59≈1.68 < 50; no storm (accepted limitation) |
| EC-009 | 4097 distinct MACs | LRU eviction at 4097th; len ≤ 4096 |
| EC-010 | `--arp-storm-rate 10`; 10 frames in 1s | Storm finding at 10/1=10 >= threshold=10 |
| EC-011 | `--arp-storm-rate 0` | Clamp to 1 or CLI error (F3 implementation decision) |
| EC-012 | D3 finding examined | `mitre_techniques: []` — T0814 absent |

## Tasks

1. **Implement D3 storm detection** in `process_arp`: follow the exact 3-step intra-frame sequence from BC-2.16.008 postconditions 1–4 (Step 1: window expiry check and initialization; Step 2: increment if window active; Step 3: rate evaluation using `count_in_window / max(1, timestamp_secs - window_start_ts)`). Emit MEDIUM/Anomaly finding with `mitre_techniques: []` when threshold met and `storm_emitted == false`; set `storm_emitted = true` after emission.
2. **Implement storm counter LRU eviction**: when `storm_counters.len() >= MAX_STORM_COUNTERS` and a new MAC arrives, evict the entry with the minimum `window_start_ts` (heuristic LRU approximation). One-in-one-out.
3. **Wire `storm_findings` counter**: increment in `process_arp` each time a D3 finding is emitted.
4. **Update `summarize()`**: ensure `storm_findings` key returns `self.storm_findings` (was 0 in STORY-113 stub; now non-zero after D3 detections).
5. **Add `--arp-storm-rate` CLI flag** to `src/cli.rs`: `#[arg(long, default_value_t = 50)] arp_storm_rate: u32` on `Commands::Analyze`. This flag is STORY-115's primary deliverable per BC-2.16.013 — it is NOT added in earlier stories. Wire it to `ArpAnalyzer::new(spoof_threshold, storm_rate)` in `src/main.rs`.
6. **Confirm `ArpAnalyzer::new(spoof_threshold, storm_rate)`** properly uses `storm_rate` in D3 evaluation. Confirm `src/main.rs` passes `args.arp_storm_rate`.
7. **Write unit tests** for AC-001 through AC-015.
8. **Write integration test** `test_integration_arp_storm_end_to_end` using synthetic pcap or byte sequence (AC-015).
9. **Run `cargo test --all-targets`**: all tests green.
10. **Run `cargo clippy --all-targets -- -D warnings`**: clean.

## Test Plan

| AC | Test | Type |
|----|------|------|
| AC-001 | `test_storm_first_observation_no_finding` | Unit |
| AC-002 | `test_storm_in_window_increments_count` | Unit |
| AC-003 | `test_storm_finding_emitted_at_threshold` | Unit |
| AC-004 | `test_storm_one_shot_guard_prevents_second_finding` | Unit |
| AC-005 | `test_storm_window_expiry_resets_counter` | Unit |
| AC-006 | `test_storm_same_second_denominator_is_1` | Unit |
| AC-007 | `test_storm_49_below_threshold_50_at_threshold` | Unit |
| AC-008 | `test_storm_window_boundary_60_in_window_61_expired` | Unit |
| AC-009 | `test_storm_late_burst_suppression_accepted_limitation` | Unit (documents limitation) |
| AC-010 | `test_storm_counter_cap_enforced` | Unit |
| AC-011 | `test_cli_arp_storm_rate_parsed`, `test_cli_arp_storm_rate_default_50`, `test_storm_custom_rate_10` | Unit |
| AC-012 | `test_storm_rate_flag_accepted_without_arp_flag` | Unit |
| AC-013 | `test_summarize_storm_findings_key_non_zero_after_detection` | Unit |
| AC-014 | `test_d3_finding_has_empty_mitre_techniques` | Unit |
| AC-015 | `test_integration_arp_storm_end_to_end` | Integration |

## Previous Story Intelligence

STORY-114 (this epic's predecessor) established:
- D1 spoof escalation + GARP-that-conflicts (BC-2.16.014) fully implemented.
- MITRE catalog at SEEDED=25, EMITTED=17. `src/mitre.rs` is now complete for the ARP feature.
- `ARP_FLAP_WINDOW_SECS = 60` is defined in `src/analyzer/arp.rs` (shared between D1 flap window and D3 storm window).
- `storm_counters: HashMap<[u8; 6], StormCounter>` was declared as a stub field in STORY-113 (empty map, no D3 logic). This story fully implements D3 detection.

**Key implementation note**: `ARP_FLAP_WINDOW_SECS` is already defined in `src/analyzer/arp.rs` (primary anchor: BC-2.16.004); D3 reuses this constant for its 60-second window. Do NOT redefine it. `ARP_STORM_RATE_DEFAULT` is a separate constant (BC-2.16.008).

**Modbus/DNP3 precedent**: no equivalent "storm rate" detection exists in prior analyzers. D3 is novel to ARP. The `count_in_window / max(1, ts - window_start_ts)` formula and the `max(1, ...)` denominator guard are both critical for the same-second burst case (ARP-AMB-003 RESOLVED — no sub-second ambiguity; timestamps are coarse integer seconds).

**Rate formula canonical test vector** from BC-2.16.008:
- 50 frames: 25 at ts=100, then 25 at ts=101 → count=50, elapsed=max(1,1)=1, rate=50/1=50 >= 50 → storm fires. Denominator is 1 (not 2) because elapsed = ts - window_start_ts = 101-100 = 1. The `max(1, ...)` does NOT add 1 to elapsed — it takes the maximum of 1 and elapsed.

## Architecture Compliance Rules

Derived from arp-architecture-delta.md §3.2, BC-2.16.008, BC-2.16.013:

1. **Rate formula is authoritative in BC-2.16.008 only** — BC-2.16.013 cross-references BC-2.16.008 PC3/Note 6 explicitly and does not restate the formula. The formula is `count_in_window / max(1, timestamp_secs - window_start_ts)`. Do not add `+1` to the denominator (that was the F2 v1.1 bugfix that resolved ARP-AMB-003).
2. **`ARP_FLAP_WINDOW_SECS = 60` is shared with D1** — defined as `const ARP_FLAP_WINDOW_SECS: u32 = 60` in `src/analyzer/arp.rs`. BC-2.16.004 is the primary anchor. Do not duplicate the constant.
3. **D3 mitre_techniques is empty — no T0814** — DF-VALIDATION-001 compliance. Any test asserting `mitre_techniques.contains("T0814")` is incorrect.
4. **`storm_emitted` resets on window expiry** — the reset in Step 1 (window-expiry path) sets `storm_emitted = false`, allowing the next window to fire a storm finding if the rate is exceeded again.
5. **`storm_findings` key in summarize() is non-zero only after D3 fires** — the key is present in all 11-key summaries with value 0 until D3 fires.
6. **This is the final story in E-16** — `blocks: []`. STORY-115 completes the ARP Security Analyzer feature for v0.7.0.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `std::collections::HashMap` | std | `storm_counters: HashMap<[u8; 6], StormCounter>` production substrate |
| `clap` | same as existing | `--arp-storm-rate: u32` flag with `default_value_t = 50` |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/analyzer/arp.rs` | Modify | D3 storm detection logic; LRU eviction for storm_counters; storm_findings counter; summarize() storm_findings key |
| `src/cli.rs` | Modify | Add `#[arg(long, default_value_t = 50)] arp_storm_rate: u32` — STORY-115 is the owner of this flag per BC-2.16.013; it is not carried forward from any earlier story |
| `tests/arp_integration_test.rs` (or equivalent) | Create/expand | `test_integration_arp_storm_end_to_end` integration test |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~4,000 |
| BC files (3 BCs: BC-2.16.008, BC-2.16.013, BC-2.16.010 storm key) | ~7,000 |
| arp-architecture-delta.md §3.2 (thresholds and D3 storm) | ~2,000 |
| STORY-113 / STORY-114 context (ArpAnalyzer current state) | ~2,000 |
| Existing `src/analyzer/arp.rs` (after STORY-113/114) | ~3,500 |
| Tool outputs (cargo test) | ~1,500 |
| **Total estimated** | **~20,000** |

Within 20–30% of agent context window.

## Dependency Rationale

- `depends_on: [STORY-114]` — D3 storm detection shares `ARP_FLAP_WINDOW_SECS = 60` (defined as a constant in BC-2.16.004, primary anchor). STORY-115 reuses this constant from `src/analyzer/arp.rs`. Also, STORY-114's VP-007 atomic update must be complete (SEEDED=25, EMITTED=17) before STORY-115 integration tests exercise the full detection + MITRE reporting pipeline — otherwise `technique_name("T0830")` returns "Unknown" in the integration test output.
- `blocks: []` — STORY-115 is the final story in E-16. No downstream E-16 stories depend on it. Phase-4 holdout evaluation (wave-gate) follows STORY-115's merge.
