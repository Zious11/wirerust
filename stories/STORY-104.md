---
document_type: story
story_id: STORY-104
epic_id: E-14
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-06-09T00:00:00Z
phase: 4
inputs:
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.013.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.014.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.015.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.016.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.017.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.018.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.019.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.020.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.021.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.022.md
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/phase-f2-spec-evolution/f2-fix-directives.md
input-hash: TBD
traces_to: .factory/specs/prd.md
points: 13
depends_on: [STORY-103]
blocks: [STORY-105]
behavioral_contracts:
  - BC-2.14.013
  - BC-2.14.014
  - BC-2.14.015
  - BC-2.14.016
  - BC-2.14.017
  - BC-2.14.018
  - BC-2.14.019
  - BC-2.14.020
  - BC-2.14.021
  - BC-2.14.022
verification_properties:
  - VP-022
priority: P0
cycle: v0.4.0-modbus
wave: 33
target_module: analyzer
subsystems: [SS-14]
estimated_days: 5
tdd_mode: strict
feature_id: issue-007-modbus-analyzer
github_issue: 7
# BC status: all 10 BCs authored at v2.0 (multi-tag) as of 2026-06-09
input-hash: "6eeea2c"
---

# STORY-104: Modbus Detection Emissions + Summary

## Narrative

- **As a** ICS/OT security analyst using wirerust to detect Modbus attacks
- **I want** all seven Modbus MITRE detection rules to fire correctly (write-class co-emission, coordinated-write T0831, dual-window burst/sustained detection, diagnostics DoS, exception-burst anomaly, recon), with findings capped at MAX_FINDINGS, and a complete `summarize()` result
- **So that** the Modbus analyzer produces actionable, correctly-attributed findings for every Modbus attack pattern defined in BC-2.14.013 through BC-2.14.022

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.14.013 | Write-Class FC in Request Direction Emits Multi-Tag Finding Carrying T1692.001 and Applicable Technique Tags |
| BC-2.14.014 | Write FC 0x06/0x10/0x16 in Request Direction Emits Finding Tagged ["T1692.001","T0836"] |
| BC-2.14.015 | Write FC to Coil (0x05/0x0F) Emits Finding Tagged ["T1692.001","T0835"] |
| BC-2.14.016 | Coordinated Write Sequence to Holding Registers Within 5-Second Window Tags Per-PDU Finding with T0831 Inline |
| BC-2.14.017 | Write-Rate Burst Exceeding Either Burst or Sustained Threshold Emits T0806 and T1692.001 Findings |
| BC-2.14.018 | Diagnostics FC 0x08 Sub-Function 0x0004 or 0x0001 Emits T0814 Denial of Service Finding |
| BC-2.14.019 | Exception Response Anomaly — Burst of Exception Codes Emits Anomaly Finding for Recon/Scanning |
| BC-2.14.020 | Reconnaissance Function Codes (0x11, 0x2B/0x0E) Emit T0888 Remote System Information Discovery Finding |
| BC-2.14.021 | summarize() Returns AnalysisSummary with Six Specified Keys |
| BC-2.14.022 | MAX_FINDINGS Cap (10,000) and Poison-Skip Behavior for ModbusAnalyzer |

## Acceptance Criteria

### AC-001 (traces to BC-2.14.013 postcondition 1 + BC-2.14.014 — register write tags)
A write-class ADU with FC in {0x06, 0x10, 0x16} in `Direction::ClientToServer` pushes exactly ONE `Finding` with `mitre_techniques: vec!["T1692.001", "T0836"]` (canonical order per ADR-006 sub-decision 3). `category: ThreatCategory::Execution`, `verdict: Verdict::Likely`, `confidence: Confidence::Medium`.
- **Test:** `test_holding_register_write_emits_t1692_001_t0836()` — FC=0x06; assert one finding; assert `mitre_techniques == ["T1692.001","T0836"]`.

### AC-002 (traces to BC-2.14.013 postcondition 1 + BC-2.14.015 — coil write tags)
A write-class ADU with FC in {0x05, 0x0F} pushes exactly ONE `Finding` with `mitre_techniques: vec!["T1692.001", "T0835"]`. FC in {0x15, 0x17} pushes ONE `Finding` with `mitre_techniques: vec!["T1692.001"]` only.
- **Test:** `test_coil_write_emits_t1692_001_t0835()` and `test_file_write_emits_t1692_001_only()`.

### AC-003 (traces to BC-2.14.016 — T0831 inline co-tag on 2nd holding-register write within 5s)
The FIRST holding-register write (FC 0x06/0x10/0x16) within a 5-second window produces `mitre_techniques: vec!["T1692.001", "T0836"]` (`t0831_window_write_count = 1`). The SECOND holding-register write within the same 5-second window produces `mitre_techniques: vec!["T1692.001", "T0836", "T0831"]` (T0831 co-tagged inline; `t0831_burst_emitted = true`). Subsequent writes in the same window produce `vec!["T1692.001", "T0836"]` again (T0831 emit-once exhausted). T0831 window uses pcap-relative `u32` microsecond timestamps; `T0831_WINDOW_SECS = 5`.
- **Test:** `test_t0831_inline_cotag_on_second_holding_register_write()` — deliver three writes within 5s; assert findings[0].mitre_techniques = ["T1692.001","T0836"], findings[1].mitre_techniques = ["T1692.001","T0836","T0831"], findings[2].mitre_techniques = ["T1692.001","T0836"].

### AC-004 (traces to BC-2.14.017 — burst detector: >N writes in 1-second window)
When `window_write_count > write_burst_threshold` within a 1-second window (pcap-relative `u32` microseconds; `wrapping_sub` for elapsed), a SEPARATE Finding is emitted with `mitre_techniques: vec!["T0806", "T1692.001"]`. `window_burst_emitted = true` (fires at most once per 1-second window). The per-PDU write finding is ALSO emitted alongside the burst finding (burst supplements, does not replace). `WRITE_BURST_WINDOW_SECS = 1`.
- **Test:** `test_burst_detector_fires_at_threshold_plus_1()` — deliver 21 writes within 1 second (default `write_burst_threshold=20`); assert exactly one burst finding with `["T0806","T1692.001"]`; assert 21 per-PDU write findings also emitted.

### AC-005 (traces to BC-2.14.017 — sustained detector: truncation-free microsecond math)
When `(sustained_window_write_count as u64) * 1_000_000 > (write_sustained_threshold as u64) * (elapsed_us as u64)` AND `elapsed_us >= WRITE_SUSTAINED_WINDOW_SECS * 1_000_000` (i.e., elapsed >= 2 seconds) AND `!sustained_burst_emitted`, a SEPARATE Finding with `mitre_techniques: vec!["T0806", "T1692.001"]` is emitted. `elapsed_us = now_ts.wrapping_sub(sustained_window_start_ts)`. The evidence string includes `"Sustained write rate exceeded: N writes over E seconds (>T/s average)"`.
- **Test:** `test_sustained_detector_uses_truncation_free_math()` — 25 writes over 2.9s (elapsed_us=2_900_000; rate=8.6/s). With naive integer division: 25 > 10*2=20 → FALSE POSITIVE. With correct formula: 25*1_000_000=25_000_000 > 10*2_900_000=29_000_000 → FALSE → no burst. Assert no sustained finding emitted.
- **Test:** `test_sustained_detector_fires_when_rate_exceeded()` — 25 writes over 2.0s (elapsed_us=2_000_000; rate=12.5/s). 25*1_000_000=25_000_000 > 10*2_000_000=20_000_000 → TRUE → burst fires. Assert one sustained finding with `["T0806","T1692.001"]`.

### AC-006 (traces to BC-2.14.017 — wrapping_sub for all window elapsed computations)
All four window-duration computations use `now_ts.wrapping_sub(window_start_ts)` (not plain subtraction). Plain subtraction panics in Rust debug mode (overflow-checks = true) when timestamps are near `u32` boundaries. `cargo test` must pass with `overflow-checks = true` (Cargo.toml release profile already has this; debug profile also has it by default).
- **Test:** `test_window_elapsed_uses_wrapping_sub()` — deliver a write at `ts=0xFFFFFF00` (near u32::MAX) followed by a write at `ts=0x00000100` (wrapped); assert `wrapping_sub(0x00000100, 0xFFFFFF00) = 0x00000200` (512 µs); assert no panic.

### AC-007 (traces to BC-2.14.018 — diagnostics FC 0x08 sub-function 0x0004 or 0x0001)
When a validated ADU has FC=0x08 and the PDU payload's sub-function is `0x0004` (Force Listen Only Mode) OR `0x0001` (Restart Communications Option), a `Finding` is emitted with `mitre_techniques: vec!["T0814"]`, `category: ThreatCategory::IcsInhibitResponseFunction`.
- **Test:** `test_diagnostics_force_listen_only_emits_t0814()` — FC=0x08 with sub-function bytes `0x00 0x04` in PDU; assert finding with `["T0814"]`.

### AC-008 (traces to BC-2.14.019 — exception-burst anomaly Anomaly finding)
When exception responses for the same exception code exceed the exception-burst threshold within a 10-second window, an `Anomaly` finding is emitted (no MITRE technique — `mitre_techniques: vec![]`). `exception_burst_emitted[code] = true` guards re-emission.
- **Test:** `test_exception_burst_emits_anomaly_finding()` — deliver 11+ exception responses for FC=0x83 within 10s; assert an `Anomaly` finding with `mitre_techniques: vec![]`.

### AC-009 (traces to BC-2.14.020 — recon FCs 0x11 and 0x2B/0x0E emit T0888)
FC=0x11 (Report Server ID) in `ClientToServer` direction emits a `Finding` with `mitre_techniques: vec!["T0888"]`. FC=0x2B (MEI Type 0x0E — Read Device Identification) in `ClientToServer` direction emits `vec!["T0888"]`. FC=0x07 (Read Exception Status) does NOT emit a finding (Decision 12).
- **Test:** `test_recon_fc_0x11_emits_t0888()`, `test_recon_fc_0x2b_0x0e_emits_t0888()`, `test_fc_0x07_does_not_emit()`.

### AC-010 (traces to BC-2.14.021 — summarize() returns six keys)
`ModbusAnalyzer::summarize()` returns `AnalysisSummary` with exactly six keys: `pdu_count`, `write_count`, `exception_count`, `parse_errors`, `findings_emitted`, `dropped_findings`. All values are `u64`. `dropped_findings` is the count of findings silently dropped due to the MAX_FINDINGS cap.
- **Test:** `test_modbus_summarize_returns_six_keys()` — process a mix of ADUs; call `summarize()`; assert all six keys are present with correct values.

### AC-011 (traces to BC-2.14.022 — MAX_FINDINGS = 10,000 poison-skip)
When `all_findings.len() >= MAX_FINDINGS (10_000)`, subsequent finding-push calls are skipped (no panic, no push). `dropped_findings` is incremented for each skipped finding. `write_count`, `fn_code_counts`, and other counters are still incremented normally (only the finding push is skipped).
- **Test:** `test_max_findings_cap_poison_skip()` — pre-fill `all_findings` to 10,000; deliver one more write-class PDU; assert `all_findings.len() == 10_000`; assert `dropped_findings == 1`; assert `write_count` incremented.

### AC-012 (traces to BC-2.14.013 invariant 5 — burst finding is independent of per-PDU finding)
The T0806+T1692.001 burst finding (from the burst detector) is a SEPARATE `Finding` object pushed alongside (not instead of) the per-PDU write finding. When the burst threshold tips on a PDU, that PDU generates up to 2 findings: the per-PDU write finding (`["T1692.001","T0836"]` or `["T1692.001","T0835"]`) AND the burst finding (`["T0806","T1692.001"]`). If the T0831 condition also fires on the same PDU, that per-PDU finding is `["T1692.001","T0836","T0831"]` AND the burst finding is separate — up to 2 findings for the threshold-tipping PDU.
- **Test:** `test_burst_and_per_pdu_finding_are_separate()` — deliver 21st write (tips threshold); assert `all_findings.len() == 22` (21 per-PDU + 1 burst).

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| Write-class detection (T1692.001/T0836/T0835) | `src/analyzer/modbus.rs` | Effectful (mutates `all_findings`, counters) |
| T0831 inline co-tag window | `src/analyzer/modbus.rs` | Effectful (window state) |
| Burst detector (T0806+T1692.001, 1s window) | `src/analyzer/modbus.rs` | Effectful (window state) |
| Sustained detector (T0806+T1692.001, >=2s) | `src/analyzer/modbus.rs` | Effectful (window state; truncation-free math) |
| Diagnostics detector (T0814) | `src/analyzer/modbus.rs` | Effectful |
| Exception-burst anomaly detector | `src/analyzer/modbus.rs` | Effectful (per-code windows) |
| Recon detector (T0888) | `src/analyzer/modbus.rs` | Effectful |
| `summarize()` | `src/analyzer/modbus.rs` | Pure-functional (reads counters, produces summary) |
| MAX_FINDINGS cap guard | `src/analyzer/modbus.rs` | Effectful (counter increment) |

**Subsystem anchor justification:** SS-14 owns this story's complete scope — all detection logic is in `src/analyzer/modbus.rs`, which is the Modbus/ICS Analysis subsystem (SS-14, C-22) per ARCH-INDEX.

**Dependency anchor justification:** STORY-104 depends on STORY-103 because all detection logic reads `ModbusFlowState` fields (window counters, exception counts) that must exist before detection can run. Writing to `t0831_window_write_count` or `window_write_count` requires the fields from STORY-103's `ModbusFlowState`.

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | FC 0x17 (Read/Write Multiple Registers) — write class | `mitre_techniques: vec!["T1692.001"]` only (not in register/coil subsets) |
| EC-002 | Burst threshold tipping on the SAME PDU as the T0831 window fire | Per-PDU: `["T1692.001","T0836","T0831"]`; Burst: `["T0806","T1692.001"]` (separate finding); up to 2 findings for this PDU |
| EC-003 | `all_findings.len() == 9999`; burst threshold also tips on the same PDU (would emit 2 findings) | First finding pushed (count=10_000); second finding skipped (`dropped_findings = 1`) |
| EC-004 | `wrapping_sub` on timestamp wrap: `now_ts=100`, `window_start_ts=u32::MAX-99` | `wrapping_sub = 200` µs (< 1s); window NOT reset; normal accumulation continues |
| EC-005 | Recon FC 0x2B with MEI type != 0x0E (not Read Device ID) | No T0888 finding; FC=0x2B is diagnostic; only sub-function 0x0E maps to T0888 |
| EC-006 | Diagnostics FC 0x08 sub-function != 0x0004 and != 0x0001 | No T0814 finding; other sub-functions are not DoS indicators |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~5,500 |
| `src/analyzer/modbus.rs` (from STORY-103 + detection logic) | ~9,000 |
| BC files (10 BCs: BC-2.14.013 through BC-2.14.022) | ~18,000 |
| `f2-fix-directives.md` §11.5 (sustained window math) + §13.5 (co-emission model) | ~4,000 |
| `tests/modbus_tests.rs` (detection tests) | ~8,000 |
| Tool outputs overhead | ~1,500 |
| **Total** | **~46,000** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~23%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-012, focusing first on AC-001 (simplest detection), AC-003 (T0831 co-tag), AC-005 (sustained detector truncation-free math), and AC-011 (MAX_FINDINGS cap). Red Gate: detection functions not yet implemented.
2. [ ] **Red Gate:** Confirm `cargo test` fails on the new detection assertions.
3. [ ] Implement write-class detection in `on_data` (per-PDU write path): determine the `mitre_techniques` vec based on FC subset (register/coil/other), check T0831 co-tag condition, push ONE finding per write PDU.
4. [ ] Implement T0831 window management: on each holding-register write, update `t0831_window_write_count`; check `now_ts.wrapping_sub(t0831_window_start_ts) > T0831_WINDOW_SECS * 1_000_000` for window expiry; set `t0831_burst_emitted` guard.
5. [ ] Implement burst detector (1-second window): update `window_write_count`; check `window_write_count > write_burst_threshold`; emit T0806+T1692.001 burst finding if `!window_burst_emitted`; reset window on expiry with `wrapping_sub`.
6. [ ] Implement sustained detector (>=2-second window): accumulate `sustained_window_write_count`; check truncation-free math: `(count as u64)*1_000_000 > (threshold as u64)*(elapsed_us as u64)`; emit T0806+T1692.001 sustained finding if `!sustained_burst_emitted`; always reset window when `elapsed_us >= 2_000_000`.
7. [ ] Implement diagnostics detector: when FC=0x08 and sub-function bytes in PDU match 0x0004 or 0x0001, emit T0814 finding.
8. [ ] Implement exception-burst anomaly detector: per-exception-code window; emit `Anomaly` (no technique) when burst threshold exceeded.
9. [ ] Implement recon detector: FC=0x11 → emit T0888; FC=0x2B with MEI type=0x0E → emit T0888; FC=0x07 → no finding.
10. [ ] Implement MAX_FINDINGS cap guard: wrap every `all_findings.push(...)` call: `if self.all_findings.len() >= MAX_FINDINGS { self.dropped_findings += 1; } else { self.all_findings.push(finding); }`.
11. [ ] Implement `summarize()` returning six keys: `pdu_count`, `write_count`, `exception_count`, `parse_errors`, `findings_emitted` (= `all_findings.len()`), `dropped_findings`.
12. [ ] **Green Gate:** `cargo build --all-targets` exits 0. `cargo test --all-targets` green. AC-001 through AC-012 pass.
13. [ ] `cargo clippy --all-targets -- -D warnings` clean. Pay special attention to `as u64` cast warnings — use explicit casts.
14. [ ] `cargo fmt --check` clean.

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-103 | Full `ModbusFlowState` field list established. `on_data` parsing loop structure established (parse → validate → branch on direction). | `wrapping_sub` for all elapsed computations. `HashMap::insert` return value used for `duplicate_inflight_txn` counter. | Window reset order: the sustained detector ALWAYS resets the window after the duration check — whether or not a finding fired. This prevents unbounded `sustained_window_write_count` accumulation. |
| STORY-100 | `mitre_techniques: Vec<String>` is the field name. Canonical emission order: T0806 > T1692.001 > T0836 > T0835 > T0831 > T0814 > T0888. | `vec!["T1692.001", "T0836"]` — T1692.001 first in all write-class per-PDU findings; `vec!["T0806", "T1692.001"]` for burst findings. | T0836 and T0835 are NEVER both in the same finding's tag list — they are FC-subset exclusive, not priority-selected. |
| STORY-102 | `classify_fc` is total over all 256 values; high bit = Exception first. | Exception FCs handled before write/read/diagnostic in the `on_data` branch. | |

**Critical design constraints (from f2-fix-directives.md):**

1. **Sustained detector math is cross-multiplication, not division** (§11.5a): `(count as u64)*1_000_000 > (threshold as u64)*(elapsed_us as u64)`. NEVER use `elapsed_secs = elapsed_us / 1_000_000` — integer truncation creates false positives. The adversarial finding that discovered this defect is documented in §11.5a with a concrete counterexample.

2. **T0831 is an inline co-tag, not a separate Finding** (§13.5): The 2nd holding-register write within the T0831 window gets `vec!["T1692.001","T0836","T0831"]` — T0831 is added to the existing per-PDU vec. No separate T0831 `Finding` object is ever created.

3. **Window reset in sustained detector** (§11.5 step 5): When `elapsed_us >= 2_000_000`, ALWAYS reset (`start_ts = now_ts`, `count = 1`, `emitted = false`) — whether or not a finding fired. This is the "slide window forward" behavior.

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Canonical `mitre_techniques` vec order: T0806 > T1692.001 > T0836 > T0835 > T0831 > T0814 > T0888 | ADR-006 Decision 13 §13.7 sub-decision 3 | Code review: all `vec![...]` literals at emission sites |
| Sustained detector uses cross-multiplication NOT integer division | f2-fix-directives.md §11.5a (Defect Eliminated) | AC-005 test; code review |
| All window elapsed computations use `now_ts.wrapping_sub(window_start_ts)` | f2-fix-directives.md §11.5b (Timestamp Wrap Policy) | AC-006 test; code review |
| T0831 is co-tagged inline on the per-PDU write finding, NOT a separate Finding | BC-2.14.016 v2.0; f2-fix-directives.md §13.5 | AC-003 test; code review |
| Burst finding is SEPARATE from per-PDU finding; burst supplements, not replaces | BC-2.14.013 invariant 5 | AC-012 test |
| `all_findings.len() >= MAX_FINDINGS` → skip push, increment `dropped_findings` | BC-2.14.022 | AC-011 test |
| `summarize()` returns exactly SIX keys (not 7+; `duplicate_inflight_txn` is internal) | BC-2.14.021; BC-2.14.009 invariant 6 | AC-010 test |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `chrono` | workspace version (0.4) | `DateTime<Utc>` conversion from pcap `u32` timestamp for `Finding.timestamp` field (same pattern as BC-2.09.007) |
| No new external crates | — | Window math uses `u64` arithmetic in stdlib |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/analyzer/modbus.rs` | **modify** | Implement all 7 detection rules; `summarize()`; MAX_FINDINGS cap; window management constants |
| `tests/modbus_tests.rs` | **modify** | Detection tests AC-001 through AC-012 |

## Forbidden Dependencies

`src/analyzer/modbus.rs` MUST NOT import:
- `src/reporter/` (L3 analyzer must not depend on L4 output)
- Any parse-combinator library
- Any window management external crate (all implemented via stdlib arithmetic)
