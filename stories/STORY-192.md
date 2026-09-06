---
document_type: story
level: ops
story_id: STORY-192
title: "S7comm Cross-Flow Correlation State + Reused MITRE Emissions (T0835/T0836/T0858/T0888/T0846/T1692.001) + Excluded-Technique Non-Goals"
epic_id: E-23
version: "1.0"
status: ready
producer: story-writer
timestamp: 2026-09-06T00:00:00Z
phase: f3
traces_to: .factory/specs/prd.md
points: 8
priority: P1
cycle: feature-s7comm
wave: 95
target_module: analyzer/s7comm
subsystems: [SS-21]
estimated_days: null
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
feature_id: feature-s7comm
depends_on: [STORY-191]
blocks: [STORY-193]
behavioral_contracts: [BC-2.21.033, BC-2.21.034, BC-2.21.035, BC-2.21.036, BC-2.21.037, BC-2.21.038, BC-2.21.039, BC-2.21.040, BC-2.21.041]
verification_properties: []
inputs:
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.033.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.034.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.035.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.036.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.037.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.038.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.039.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.040.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.041.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
  - .factory/research/s7comm-mitre-ics-tagging.md
input-hash: "e445d91"
---

> **tdd_mode:** `strict` — full TDD Iron Law enforced.

# STORY-192: S7comm Cross-Flow Correlation State + Reused MITRE Emissions

## Narrative

**As a** security engineer using wirerust to detect S7comm write/reconnaissance/DoS/
unauthorized-command activity,
**I want** cross-flow correlation state for multi-host sweep and per-destination
expected-source baselining, plus emission call-sites for the eight already-seeded MITRE
techniques (T0835, T0836, T0858, T0816, T0888, T0846, T1692.001) this feature reuses,
**so that** the full S7comm passive dissection surface produces defensible findings
without requiring any new `mitre.rs` catalog entries — all eight IDs are already
seeded and emitted by Modbus/ENIP/DNP3.

This story adds NO new MITRE catalog entries (unlike STORY-191) — every technique here
reuses an existing `SEEDED_TECHNIQUE_IDS`/`EMITTED_IDS` entry; only new S7comm-specific
emission call-sites are added.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.21.033 | Cross-Flow Global Correlation State on S7commAnalyzer | State-only substrate; sweep + baseline tracking |
| BC-2.21.034 | Write Var to I/O Area Emits T0835 Manipulate I/O Image | Reused technique, emission call-site only |
| BC-2.21.035 | Write Var to DB/Marker Area Emits T0836 Modify Parameter | Reused technique, emission call-site only |
| BC-2.21.036 | PLC Stop or PLC Control Program-Start Emits T0858 Change Operating Mode | Reused technique, two emission paths |
| BC-2.21.037 | Decoded Restart Sub-Operation Co-Tags T0816 Device Restart/Shutdown | Reused technique; RESOLVED to zero call-sites this cycle |
| BC-2.21.038 | Read SZL or Block-List Userdata Emits T0888 Remote System Information Discovery | Reused technique; consumes STORY-189's group correction |
| BC-2.21.039 | Multi-Host Setup-Communication Sweep Emits T0846 Remote System Discovery | Reused technique, cross-flow |
| BC-2.21.040 | Command-Class Frame From Unexpected Source Co-Tags T1692.001 Unauthorized Message | Reused technique, baseline-relative (DNP3-aligned, not Modbus-aligned) |
| BC-2.21.041 | Excluded and Deferred MITRE Techniques Are Explicit Non-Goals | Negative-space contract; version pin retained |

## Acceptance Criteria

### AC-192-001: Cross-flow state tracks per-source Setup-Communication targets with a sweep window
(traces to BC-2.21.033 postcondition 1)
- Given `S7commAnalyzer` (not `S7commFlowState` — this is instance-level, not per-flow)
- When a `SetupCommunication` request is observed from `src_ip` to `dst_ip`
- Then `port102_setup_targets[src_ip]` gains `dst_ip` (set insert; a repeat `dst_ip` is a
  no-op); a `S7_SWEEP_WINDOW_SECS = 300` window is tracked per-source via a
  `first_setup_ts` companion map; when the window elapses, that source's entry resets on
  its next `SetupCommunication` (traces to BC-2.21.033 postcondition 3)
- **Test:** `test_BC_2_21_033_setup_targets_tracked_with_window`

### AC-192-002: Cross-flow state establishes a first-write-wins expected-source baseline per destination
(traces to BC-2.21.033 postcondition 2)
- Given a command-class frame (`WriteVar`, the download triad, `PlcControl` with a
  recognized service, or `PlcStop`) targeting `dst_ip` from `src_ip`
- When `expected_source_by_destination` has no entry for `dst_ip`
- Then it is set to `src_ip` — no finding is emitted for the baseline-establishing frame
  itself
- If an entry already exists and differs from `src_ip`, the entry is left UNCHANGED —
  the original baseline source is never overwritten (traces to BC-2.21.033
  postcondition 4)
- **Test:** `test_BC_2_21_033_expected_source_baseline_first_write_wins`

### AC-192-003: Write Var to I/O area emits T0835
(traces to BC-2.21.034 postcondition 1)
- Given `S7ClassicFunction::WriteVar(area)` with `area ∈ {DirectPeripheral, Inputs,
  Outputs}` and `self.all_findings.len() < MAX_S7COMM_FINDINGS`
- When the frame is processed
- Then exactly ONE `Finding` is pushed: `category: Execution`, `verdict: Likely`,
  `confidence: High`, `mitre_techniques: vec!["T0835"]`; no one-shot guard —
  per-occurrence
- **Test:** `test_BC_2_21_034_write_var_io_area_emits_t0835`

### AC-192-004: Write Var to DB/Marker area emits T0836
(traces to BC-2.21.035 postcondition 1)
- Given `S7ClassicFunction::WriteVar(area)` with `area ∈ {Markers, DataBlock}`
- When the frame is processed
- Then exactly ONE `Finding` is pushed: `category: Execution`, `verdict: Likely`,
  `confidence: Medium` (lower than T0835's `High`, reflecting more routine legitimate
  DB/marker writes — traces to BC-2.21.035 postcondition 3), `mitre_techniques:
  vec!["T0836"]`
- **Test:** `test_BC_2_21_035_write_var_db_marker_area_emits_t0836`

### AC-192-005: PlcStop and PlcControl(ProgramStart) both emit T0858 with different confidence
(traces to BC-2.21.036 postconditions 1-2)
- Given `S7ClassicFunction::PlcStop`
- When the frame is processed
- Then exactly ONE `Finding` is pushed with `confidence: High`, `mitre_techniques:
  vec!["T0858"]`
- Given `S7ClassicFunction::PlcControl(ProgramStart)`
- Then exactly ONE `Finding` is pushed with `confidence: Medium` (lower — sub-operation
  not decoded, per BC-2.21.036 Invariant 3), `mitre_techniques: vec!["T0858"]`
- **Test:** `test_BC_2_21_036_plc_stop_high_confidence_t0858`,
  `test_BC_2_21_036_program_start_medium_confidence_t0858`

### AC-192-006: T0816 is never emitted this cycle — zero call-sites regression guard
(traces to BC-2.21.037 postconditions 2-3)
- Given `S7ClassicFunction::PlcControl(ProgramStart)` under any input this feature
  produces
- When findings are generated
- Then no `Finding` ever contains `"T0816"` in `mitre_techniques` — the restart
  sub-operation decode this BC would require has been RESOLVED to zero call-sites this
  cycle (no independent verification of the restart-byte convention was performed)
- `PlcStop` NEVER co-tags T0816 under any circumstance (traces to BC-2.21.037
  postcondition 3)
- **Test:** `test_BC_2_21_037_t0816_never_emitted_regression_guard` (exhaustive check
  across all `PlcControl`/`PlcStop` test fixtures)

### AC-192-007: Read SZL or Block-List Userdata emits T0888
(traces to BC-2.21.038 postcondition 1)
- Given `S7ClassicFunction::Userdata(CpuReadSzl)` OR
  `S7ClassicFunction::Userdata(BlockFunctions(_))` (any subfunction)
- When the frame is processed
- Then exactly ONE `Finding` is pushed: `category: Reconnaissance`, `verdict: Likely`,
  `confidence: High`, `mitre_techniques: vec!["T0888"]`
- `CpuOther(_)`, `TimeFunctions(_)`, and `OtherGroup(_, _)` never trigger this emission
  (traces to BC-2.21.038 postcondition 3) — this is the direct regression-guard consumer
  of the load-bearing group `0x03`/`0x07` correction established in STORY-189
- **Test:** `test_BC_2_21_038_read_szl_emits_t0888`,
  `test_BC_2_21_038_block_functions_emits_t0888`,
  `test_BC_2_21_038_time_functions_never_emits_t0888` (the group-correction regression
  guard — MUST fail if the STORY-189 group `0x03`/`0x07` mapping were ever swapped)

### AC-192-008: Multi-host Setup-Communication sweep emits T0846 exactly once per source per window
(traces to BC-2.21.039 postcondition 1)
- Given `port102_setup_targets[src_ip].len() >= S7_SWEEP_THRESHOLD_DEFAULT` and
  `sweep_reported[src_ip] == false`
- When the threshold is first crossed
- Then exactly ONE `Finding` is pushed: `category: Reconnaissance`, `verdict: Likely`,
  `confidence: Medium`, `mitre_techniques: vec!["T0846"]`; `sweep_reported[src_ip]` is
  set to `true` (traces to BC-2.21.039 postcondition 2)
- When the sweep window elapses and resets, `sweep_reported[src_ip]` also resets to
  `false` — a genuinely new campaign can re-trigger (traces to BC-2.21.039
  postcondition 3)
- **Test:** `test_BC_2_21_039_sweep_threshold_emits_t0846_once_per_window`

### AC-192-009: Command-class frame from an unexpected source co-tags T1692.001 (baseline-relative)
(traces to BC-2.21.040 postcondition 1)
- Given a command-class frame targets `dst_ip` from `src_ip`,
  `expected_source_by_destination[dst_ip] == Some(baseline_src)`, and
  `src_ip != baseline_src`, and this frame already produces a host `Finding` via one of
  the write/download/PlcStop emission paths
- When the frame is processed
- Then `"T1692.001"` is appended to that host finding's `mitre_techniques` vec; the
  baseline is left UNCHANGED (traces to BC-2.21.040 postcondition 2)
- No one-shot guard: every subsequent mismatched-source frame re-triggers the co-tag
  (traces to BC-2.21.040 postcondition 3)
- **Test:** `test_BC_2_21_040_unexpected_source_cotags_t1692_001`

### AC-192-010: T1692.001's baseline-relative policy is intentional and NOT reconciled toward Modbus's blanket policy
(traces to BC-2.21.040 design note, RESOLVED at F2 INTEGRATE)
- Given Modbus's existing blanket (unconditional) T1692.001 co-tag convention
- When S7comm's T1692.001 emission condition is implemented
- Then it follows DNP3's gated, baseline-relative model — this divergence is
  intentional, per-protocol, evidence-strength-driven, and MUST NOT be "fixed" toward a
  single uniform cross-protocol policy in this or any future maintenance sweep without
  first re-deriving each protocol's own evidence-strength justification
- **Test:** `test_BC_2_21_040_baseline_relative_not_blanket_regression_guard` (asserts a
  command-class frame from the FIRST-observed source to a destination never emits
  T1692.001, unlike Modbus's blanket policy)

### AC-192-011: Excluded and deferred techniques never appear; version pin unchanged
(traces to BC-2.21.041 postconditions 1-4)
- Given any classification or state this feature produces
- When findings are generated across the full test fixture set
- Then no `Finding` ever contains `"T0851"`, `"T0873"`, `"T0873.001"`, or `"T0813"` in
  `mitre_techniques`
- `MITRE_ATTACK_VERSION` remains `"ics-attack-19.1"` — this feature does not bump the pin
- **Test:** `test_BC_2_21_041_excluded_deferred_techniques_never_emitted`,
  `test_BC_2_21_041_mitre_version_pin_unchanged`

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `port102_setup_targets`, `first_setup_ts`, `sweep_reported` fields on `S7commAnalyzer` | SS-21 instance-level state | `src/analyzer/s7comm.rs` | Mutable state (cross-flow, not per-flow) |
| `expected_source_by_destination` field on `S7commAnalyzer` | SS-21 instance-level state | `src/analyzer/s7comm.rs` | Mutable state |
| `const S7_SWEEP_WINDOW_SECS: u32 = 300`, `const S7_SWEEP_THRESHOLD_DEFAULT: usize` | SS-21 constants | `src/analyzer/s7comm.rs` | N/A |
| Reused-technique emission call-sites (T0835/T0836/T0858/T0888/T0846/T1692.001) | SS-21 effectful shell | `src/analyzer/s7comm.rs` | Effectful |

Subsystem anchor: SS-21 owns this story's scope — cross-flow correlation state and
emission call-sites for already-seeded techniques are the final dissection capabilities
of the S7comm analyzer per ARCH-INDEX.md §SS-21.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Cross-flow correlation update logic, all emission call-sites | effectful-shell | Mutates `S7commAnalyzer`-instance-level `HashMap`/`HashSet` state, pushes `Finding` objects |
| `mitre.rs` `technique_info`/`EMITTED_IDS` (unchanged; consulted, not modified) | pure-core | Read-only lookups against the existing catalog (no new entries this story) |

## Tasks

- [ ] Add `port102_setup_targets: HashMap<IpAddr, HashSet<IpAddr>>`,
      `first_setup_ts: HashMap<IpAddr, u32>`, `sweep_reported: HashMap<IpAddr, bool>`,
      `expected_source_by_destination: HashMap<IpAddr, IpAddr>` fields to
      `S7commAnalyzer` (instance-level, NOT `S7commFlowState`)
- [ ] Define `const S7_SWEEP_WINDOW_SECS: u32 = 300;` and
      `const S7_SWEEP_THRESHOLD_DEFAULT: usize = <value per research>;`
- [ ] Implement the `SetupCommunication` observation update rule (BC-2.21.033
      postcondition 3) and the command-class-frame baseline update rule (postcondition 4)
- [ ] Implement T0835 emission for `WriteVar` I/O areas (BC-2.21.034)
- [ ] Implement T0836 emission for `WriteVar` DB/marker areas (BC-2.21.035)
- [ ] Implement T0858 emission for `PlcStop` (High confidence) and
      `PlcControl(ProgramStart)` (Medium confidence) (BC-2.21.036)
- [ ] Add an explicit code comment and regression test documenting T0816's
      zero-call-site resolution (BC-2.21.037) — do NOT implement a restart-byte decode
- [ ] Implement T0888 emission for `CpuReadSzl` and `BlockFunctions(_)` (BC-2.21.038)
- [ ] Implement T0846 emission on sweep-threshold crossing (BC-2.21.039)
- [ ] Implement T1692.001 co-tag on baseline mismatch for command-class frames
      (BC-2.21.040), following DNP3's gated model — NOT Modbus's blanket model
- [ ] Write the excluded/deferred-technique regression-guard tests (BC-2.21.041)
- [ ] Write unit tests: one per AC, named `test_BC_2_21_033_*` .. `test_BC_2_21_041_*`
- [ ] Extend `tests/fixtures/mk_s7comm_pcap.py` with multi-destination Setup
      Communication frames (sweep scenario) and a mismatched-source command-class frame
- [ ] Verify `cargo test` passes
- [ ] Add a CHANGELOG entry under `[Unreleased] > Added` describing the reused MITRE
      technique emission call-sites and cross-flow correlation, before creating the PR

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.21.033 | Same `dst_ip` observed twice for the same `src_ip`'s Setup Communication | Set insert is a no-op the second time; sweep count does not double-count |
| EC-002 | BC-2.21.036 | `PlcControl(ProgramStart)` where the parameter block also happens to satisfy a hypothetical restart decode | No T0816 emitted regardless — BC-2.21.037 is RESOLVED to zero call-sites, not a conditional gate |
| EC-003 | BC-2.21.038 | `Userdata(OtherGroup(0x02, subfn))` (an unenumerated group) | No T0888 emission — only `CpuReadSzl` and `BlockFunctions(_)` qualify |
| EC-004 | BC-2.21.039 | Exactly `S7_SWEEP_THRESHOLD_DEFAULT - 1` distinct destinations observed | No T0846 emitted yet — threshold is `>=`, not `>` |
| EC-005 | BC-2.21.040 | The very first command-class frame to a never-before-seen destination | Establishes the baseline; never itself triggers T1692.001 |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~7,000 |
| BC-2.21.033-041 (9 BCs) | ~9,500 |
| ADR-014 Decision 5, s7comm-mitre-ics-tagging.md | ~6,000 |
| src/analyzer/s7comm.rs (from STORY-187-191) | ~9,000 |
| Test file delta + fixture generator extension | ~4,500 |
| **Total** | **~36,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~18%** |

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-191 | `MAX_S7COMM_FINDINGS` introduced; T0843/T0889/T0821 six-part atomic complete | Findings-cap check pattern (`self.all_findings.len() < MAX_S7COMM_FINDINGS`) established for every emission call-site | This story's technique set is ENTIRELY reused (no `mitre.rs` catalog changes) — resist the urge to touch `SEEDED_TECHNIQUE_IDS`/`EMITTED_IDS` for any of the eight IDs here; they already exist from Modbus/ENIP/DNP3 |

The T0888 emission call-site (AC-192-007) is the direct downstream consumer of
STORY-189's load-bearing group `0x03`/`0x07` correction — its regression-guard test
(`test_BC_2_21_038_time_functions_never_emits_t0888`) is the end-to-end proof that the
correction actually matters at the finding-emission layer, not just at classification.

## Architecture Compliance Rules

Extracted from `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md`:
- **ADR-014 Decision 5**: all eight techniques in this story (T0835, T0836, T0858, T0816,
  T0888, T0846, T1692.001 — 7 distinct IDs, T0816 resolved to zero call-sites) are
  reused, already-seeded, already-emitted (via Modbus/ENIP/DNP3) — this story adds S7comm
  emission call-sites ONLY, no `SEEDED_TECHNIQUE_IDS`/`EMITTED_IDS`/`technique_info`
  changes.
- **T0846's scope is finally and deliberately narrow**: Setup-Communication-based sweep
  proxy only, never a true TCP-SYN sweep (wirerust has no packet-capture-layer SYN
  visibility at this analyzer layer).
- **T1692.001's baseline-relative policy is a deliberate per-protocol divergence** from
  Modbus's blanket policy — do not reconcile toward one uniform cross-protocol
  convention.
- **T0816 is RESOLVED to zero call-sites this cycle** — do not implement a restart-byte
  decode without independent research verification in a future cycle.
- **T0851, T0873, T0873.001 are EXCLUDED (never seeded, never emitted)**; T0813 is
  DEFERRED (never seeded, never emitted this cycle, not precluded from a future cycle).
- Version pin remains `ics-attack-19.1` — no bump.

## Library & Framework Requirements

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | `HashMap<IpAddr, ...>`, `HashSet<IpAddr>` |

No new external crate dependencies. No `mitre.rs` catalog changes in this story.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `src/analyzer/s7comm.rs` | MODIFY | Add cross-flow correlation state on `S7commAnalyzer`; add T0835/T0836/T0858/T0888/T0846/T1692.001 emission call-sites; document T0816's zero-call-site resolution |
| `tests/s7comm_analyzer_tests.rs` | MODIFY | Add BC-2.21.033-041 unit tests including the excluded/deferred-technique regression guards |
| `tests/fixtures/mk_s7comm_pcap.py` | MODIFY | Add multi-destination Setup Communication sweep frames + mismatched-source command-class frame |

## Forbidden Dependencies

- `src/analyzer/s7comm.rs` MUST NOT add `"T0851"`, `"T0873"`, `"T0873.001"`, or `"T0813"`
  to any `mitre_techniques` vec — these are excluded/deferred per BC-2.21.041
- `src/mitre.rs` MUST NOT be modified by this story — all eight reused technique IDs
  already exist in `SEEDED_TECHNIQUE_IDS`/`EMITTED_IDS`

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-06 | story-writer | Initial authorship — cross-flow correlation state, reused MITRE emission call-sites (T0835/T0836/T0858/T0888/T0846/T1692.001), T0816 zero-call-site resolution, excluded/deferred-technique regression guards, AC-192-001..011. |
