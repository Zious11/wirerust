---
document_type: story
story_id: STORY-134
title: "ENIP Recon Detections: T0846 ListIdentity, T0888 Identity Read / Error Burst, and CIP Error Accumulation"
epic_id: E-20
wave: 60
points: 8
phase: f3
tdd_mode: strict
status: ready
feature_id: issue-316-enip-analyzer
github_issue: 316
subsystems: [SS-17]
target_module: analyzer/enip
depends_on: [STORY-132, STORY-133]
behavioral_contracts:
  - BC-2.17.010
  - BC-2.17.008
  - BC-2.17.014
verification_properties: []
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.010.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.008.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.014.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
  - .factory/research/enip-mitre-ics-tagging.md
input-hash: "b147e79"
---

# STORY-134: ENIP Recon Detections: T0846 ListIdentity, T0888 Identity Read / Error Burst, and CIP Error Accumulation

## Narrative

**As a** security analyst monitoring industrial networks with wirerust,
**I want** the EtherNet/IP analyzer to detect ENIP ListIdentity command broadcasts (T0846
Remote System Discovery), CIP Identity Object attribute reads (T0888 Remote System Information
Discovery Pattern A), and CIP error-response bursts (T0888 Pattern B),
**so that** adversary network reconnaissance against EtherNet/IP devices is detected and reported.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.17.010 | ListIdentity ENIP command emits T0846 (Remote System Discovery) | Core detection implementation |
| BC-2.17.008 | CIP error responses accumulate in per-flow error_counts_in_window | Supporting state accumulation |
| BC-2.17.014 | CIP Identity-read or error burst emits T0888 | Core detection implementation |

## Acceptance Criteria

### AC-134-001: ListIdentity ENIP command emits T0846 finding (per-flow one-shot)
**Traces to:** BC-2.17.010 postconditions 1–3; BC-2.17.010 invariant 1
- Given an ENIP frame with `classify_enip_command(header.command)` returning `EnipCommandClass::ListIdentity` (0x0063)
- When `EnipAnalyzer::process_pdu(flow_key, payload, timestamp, src_ip)` processes the frame
- Then **`command_counts[0x0063]` is incremented exclusively by the BC-2.17.016 frame-walk (`on_data`, STORY-137) — `process_pdu` does NOT increment `command_counts` for ListIdentity or any other command (F8-001, BC-2.17.010 postcondition 1)**
- AND if `flow.list_identity_emitted == false` AND `all_findings.len() < MAX_FINDINGS`:
  - Push exactly ONE `Finding` to `all_findings`:
    - `category: ThreatCategory::Reconnaissance`
    - `verdict: Verdict::Likely`
    - `confidence: Confidence::High`
    - `summary: "EtherNet/IP ListIdentity broadcast observed: network-wide device enumeration (T0846)"` (BC-2.17.010 postcondition 2 — EXACT string)
    - `evidence`: one entry — `"ENIP command=0x0063 (ListIdentity) src={src_ip} session={session_handle}"` (BC-2.17.010 postcondition 2)
    - `mitre_techniques: vec!["T0846"]`
    - `source_ip: Some(src_ip)`, `timestamp: Some(timestamp)`
  - Set `flow.list_identity_emitted = true` (per-flow one-shot guard — BC-2.17.010 postcondition 2 last line)
- AND if `flow.list_identity_emitted == true`: `command_counts` updated; NO additional finding (BC-2.17.010 postcondition 3)
- **Per-flow one-shot guard (BC-2.17.010 invariant 1):** A scan campaign sending 5 ListIdentity frames on the SAME flow emits exactly 1 T0846 finding (first frame) and 0 additional findings for frames 2–5; `command_counts[0x0063]` = 5. This must pass holdout HS-114 Case B.
- Multi-frame one-shot test: Send 5 ListIdentity frames on same flow → assert exactly 1 finding total, `command_counts[0x0063] == 5`, `list_identity_emitted == true`
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_list_identity_emits_t0846`
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_list_identity_one_shot_guard_multi_frame`
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_list_identity_respects_max_findings`

### AC-134-002: CIP error responses accumulate per-status in error_counts_in_window
**Traces to:** BC-2.17.008 postconditions 1–5; BC-2.17.008 invariants 1–4
- **Scope gate precondition (BC-2.17.008 precondition 2):** HARD scope gate — only applies when `cpf_item.type_id == 0x00B2` (Unconnected Data Item). If `type_id != 0x00B2`, skip extraction entirely; no counter update
- **Precondition (BC-2.17.008 precondition 3):** `cip_item_data.len() >= 4` required before indexing byte 2
- Given the above preconditions hold and `classify_cip_service(service)` returns `CipServiceClass::Response` (BC-2.17.008 precondition 1)
- When the analyzer processes the CIP response
- Then `general_status = cip_item_data[2]` (BC-2.17.008 postcondition 1 — third byte of CIP response: byte 0 = service|0x80, byte 1 = reserved 0x00, byte 2 = general_status, byte 3 = additional_status_size)
- If `general_status != 0x00` (error response): `flow.error_counts_in_window.entry(general_status).or_insert(0) += 1`; if `flow.error_window_active == false` (window not yet seeded — first qualifying error on this flow) seed `flow.error_window_start_ts = now_ts` and set `flow.error_window_active = true` (BC-2.17.008 postcondition 2 — do NOT use `error_window_start_ts == 0` as the unseeded sentinel; timestamp 0 is a valid pcap-relative value and must not be overloaded); ALSO `self.error_count += 1` on `EnipAnalyzer` (aggregate lifetime counter — BC-2.17.008 Postcondition 2b / Invariant 2; consumed by `summarize()` per BC-2.17.021 postcondition 1 `error_count` field)
- If `general_status == 0x00` (success): no error counter update, no aggregate increment (BC-2.17.008 postcondition 3)
- Window management: if `flow.error_window_active == true` AND `now_ts.wrapping_sub(flow.error_window_start_ts) > 10` (window expired): reset `flow.error_counts_in_window.clear()`, `flow.error_window_start_ts = now_ts`, `flow.error_rate_emitted = false` (`error_window_active` remains `true` — the window rolls forward; it is only `false` before the very first qualifying error on a new flow) (BC-2.17.008 postcondition 4)
- **Field names (BC-2.17.008 invariants):** `error_counts_in_window: HashMap<u8, u64>`, `error_window_start_ts: u32`, `error_window_active: bool`, `error_rate_emitted: bool`; NOT `error_window_start` or any other alias; `error_window_active` is the dedicated unseeded-window flag (BC-2.17.008 v1.2 Architecture Anchors)
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_error_accumulation_increments_per_status`
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_error_accumulation_ignores_success`
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_error_window_resets_after_10s`
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_error_accumulation_skips_connected_item`
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_error_accumulation_requires_4_bytes`

### AC-134-003: T0888 Pattern A — GetAttribute to Identity Object (Class 0x01) emits finding
**Traces to:** BC-2.17.014 postconditions Pattern A
- Given a CIP request (`service & 0x80 == 0`) with `classify_cip_service(service)` returning `GetAttributeSingle`, `GetAttributesAll`, or `GetAttributeList`
- AND `parse_cip_request_path(path)` contains `CipPathSegment::Class(0x01)` (Identity Object)
- AND item `type_id == 0x00B2` (Unconnected Data Item gate per F-P9-001)
- When the analyzer processes the frame
- Then ONE `Finding`:
  - `category: ThreatCategory::Reconnaissance`
  - `verdict: Verdict::Likely`
  - `confidence: Confidence::High`
  - `summary: "CIP Identity Object attribute read: single-device reconnaissance (T0888)"`
  - `mitre_techniques: vec!["T0888"]`
- Pattern A fires per-occurrence (not one-shot)
- Does NOT fire for `type_id == 0x00B1` items (F-P9-001)
- Does NOT fire for GetAttribute to non-Identity classes (e.g., Class 0x04 Assembly)
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_t0888_pattern_a_identity_read`
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_t0888_pattern_a_non_identity_no_finding`
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_t0888_pattern_a_connected_item_no_finding`
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_t0888_pattern_a_fires_per_occurrence`

### AC-134-004: T0888 Pattern B — error burst crossing threshold emits one-shot finding
**Traces to:** BC-2.17.014 postconditions Pattern B
- Given `total_error_count = sum(flow.error_counts_in_window.values())` strictly exceeds `self.enip_error_burst_threshold` (default 5)
- AND `flow.error_rate_emitted == false`
- When the (threshold+1)th error response arrives
- Then ONE `Finding`:
  - `category: ThreatCategory::Reconnaissance`
  - `verdict: Verdict::Possible`
  - `confidence: Confidence::Medium`
  - `summary: "CIP error-response burst: {total_errors} error responses in 10s window — possible service enumeration (T0888)"`
  - `mitre_techniques: vec!["T0888"]`
  - `flow.error_rate_emitted = true` (one-shot guard)
- With default threshold=5: exactly 5 errors → no finding; 6th error → finding
- With threshold=0: first error (count=1 > 0) → finding immediately
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_t0888_pattern_b_fires_at_threshold_plus_one`
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_t0888_pattern_b_one_shot_guard`
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_t0888_pattern_b_no_fire_at_threshold`
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_t0888_pattern_b_threshold_zero`

### AC-134-005: is_non_enip flow flag suppresses all ENIP detections
**Traces to:** BC-2.17.010 Precondition 2, BC-2.17.014 preconditions
- When `flow.is_non_enip == true`, no T0846 or T0888 findings are emitted regardless of frame content
- `is_non_enip` is set by the frame-walk robustness logic in STORY-137 (BC-2.17.016)
- In this story, test it by constructing a flow state with `is_non_enip=true` and verifying no findings are emitted
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_non_enip_flow_suppresses_recon`

### AC-134-006: EnipAnalyzer aggregate error_count increments on every CIP error response
**Traces to:** BC-2.17.008 Postcondition 2b; BC-2.17.008 Invariant 2; BC-2.17.021 postcondition 1 (`error_count` field in summarize() output)
- `EnipAnalyzer.error_count: u64` is a lifetime aggregate counter on the `EnipAnalyzer` struct (separate from per-flow `error_counts_in_window`)
- On every CIP response with `general_status != 0x00` (and type_id==0x00B2, len>=4 preconditions met): `self.error_count += 1` (formal postcondition — BC-2.17.008 Postcondition 2b)
- `self.error_count` is NOT reset between flows or windows — it accumulates across the entire analysis session
- Success responses (general_status==0x00) do NOT increment `error_count`
- `summarize()` (STORY-138, BC-2.17.021) reads `self.error_count` to populate the `"error_count"` field in `enip_summary` JSON
- **Test:** `tests/enip_analyzer_tests.rs::recon::test_aggregate_error_count_increments` (process N error responses across multiple flows; assert `analyzer.error_count == N`)

## Architecture Mapping

| Component | Location | Role |
|-----------|----------|------|
| `EnipAnalyzer.error_count` | `src/analyzer/enip.rs` | `u64` — aggregate lifetime CIP error count (BC-2.17.008 Postcondition 2b / Invariant 2; BC-2.17.021 Architecture Anchors; feeds summarize()) |
| `EnipFlowState.error_counts_in_window` | `src/analyzer/enip.rs` | `HashMap<u8, u64>` — per-status CIP error counts within 10s window (BC-2.17.008) |
| `EnipFlowState.error_rate_emitted` | `src/analyzer/enip.rs` | `bool` — one-shot guard for T0888 Pattern B |
| `EnipFlowState.is_non_enip` | `src/analyzer/enip.rs` | `bool` — suppress all ENIP detections for non-ENIP flows |
| `EnipFlowState.error_window_start_ts` | `src/analyzer/enip.rs` | `u32` — 10s error window start timestamp (BC-2.17.008 postcondition 2; canonical field name per BC) |
| `EnipFlowState.error_window_active` | `src/analyzer/enip.rs` | `bool` — dedicated flag: `false` on new flow, set `true` on first qualifying error; replaces the former `== 0` sentinel (BC-2.17.008 v1.2 M-1, ADR-010 Decision 4) |
| `EnipFlowState.list_identity_emitted` | `src/analyzer/enip.rs` | `bool` — per-flow one-shot guard for T0846 (BC-2.17.010 invariant 1) |
| `EnipAnalyzer::process_pdu` | `src/analyzer/enip.rs` | Main effectful dispatch; calls all detection logic |
| T0846 detection | `src/analyzer/enip.rs` | `if command == ListIdentity { ... }` |
| T0888 Pattern A | `src/analyzer/enip.rs` | `if GetAttribute && Class(0x01) in path { ... }` |
| T0888 Pattern B | `src/analyzer/enip.rs` | `if total_errors > threshold && !error_rate_emitted { ... }` |
| `Finding` construction | `src/analyzer/enip.rs` | Uses `Finding { category, verdict, confidence, summary, evidence, mitre_techniques, source_ip, timestamp }` |
| Test mod | `tests/enip_analyzer_tests.rs` | `mod recon { ... }` |

**Detection order in `process_pdu` (ADR-010 Decision 4):**
1. Parse ENIP header; call `is_valid_enip_frame`
2. `classify_enip_command`; check ListIdentity → emit T0846
3. For SendRRData/SendUnitData: walk CPF items (0x00B2 only for CIP parse)
4. `parse_cip_header`; `classify_cip_service`
5. If request + GetAttribute + Class(0x01) → emit T0888 Pattern A
6. If response + `general_status != 0x00` → accumulate error; check Pattern B threshold

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Single ListIdentity frame on a new flow | T0846 finding emitted; `command_counts[0x0063]=1`; `list_identity_emitted=true` (BC-2.17.010 EC-001) |
| EC-002 | Multiple ListIdentity frames in sequence (same flow) | First frame: T0846 finding + guard set. Subsequent frames: `command_counts[0x0063]` incremented; NO additional findings (one-shot guard, BC-2.17.010 EC-002) |
| EC-002b | ListIdentity with `all_findings.len() == MAX_FINDINGS` (guard false) | No finding pushed; command_counts still updated; guard remains false (BC-2.17.010 EC-003) |
| EC-003 | GetAttributeSingle to Identity Object (0x01) via 0x00B2 item | T0888 Pattern A finding |
| EC-004 | GetAttributeSingle to Assembly Object (0x04) via 0x00B2 item | No T0888 finding |
| EC-005 | GetAttributeSingle to Identity Object via 0x00B1 item | No T0888 Pattern A (F-P9-001 gate) |
| EC-006 | 5 CIP errors in 10s (threshold=5 default, strict >) | No Pattern B finding |
| EC-007 | 6 CIP errors in 10s | Pattern B finding; `error_rate_emitted=true` |
| EC-008 | 7th error in same window | No additional Pattern B (one-shot guard) |
| EC-009 | 10s window expires; 6 new errors | New window; Pattern B can fire again |
| EC-010 | `is_non_enip=true`; ListIdentity frame | No T0846 finding |
| EC-011 | GetAttributesAll to Identity Object | T0888 Pattern A finding |
| EC-012 | CIP error response `general_status=0xFF` | Accumulated in `error_counts_in_window[0xFF]` |

## Tasks

- [ ] Add `error_count: u64` field to `EnipAnalyzer` struct (aggregate lifetime counter; BC-2.17.008 Postcondition 2b / Invariant 2; BC-2.17.021 Architecture Anchors `EnipAnalyzer.error_count: u64`; feeds summarize())
- [ ] Add to `EnipFlowState` in `src/analyzer/enip.rs`: `error_counts_in_window: HashMap<u8, u64>`, `error_window_start_ts: u32`, `error_window_active: bool`, `error_rate_emitted: bool`, `list_identity_emitted: bool`, `is_non_enip: bool`, `command_counts: HashMap<u16, u64>`, `pdu_count: u64`, `parse_errors: u64`, `malformed_in_window: u64`, `carry: Vec<u8>` (use exact field names per BC-2.17.008/010/016; `error_window_active` is the explicit unseeded-window flag — BC-2.17.008 v1.2 M-1)
- [ ] In `EnipAnalyzer::process_pdu`: if `!flow.list_identity_emitted && !flow.is_non_enip && all_findings.len() < MAX_FINDINGS` → push T0846 finding with exact summary "EtherNet/IP ListIdentity broadcast observed: network-wide device enumeration (T0846)" and evidence "ENIP command=0x0063 (ListIdentity) src={src_ip} session={session_handle}"; set `flow.list_identity_emitted = true`. **SINGLE-INCREMENT NOTE (BC-2.17.024/025):** Do NOT add a `command_counts` increment here. The per-command `flow.command_counts.entry(header.command) += 1` increment is owned exclusively by the frame-walk (`on_data`) in STORY-137 per BC-2.17.016 PC-0, fired immediately after `parse_enip_header` returns `Some` and BEFORE `is_valid_enip_frame`; `process_pdu` owns `pdu_count` only (BC-2.17.024 Inv-5). STORY-134's ListIdentity logic relies on the count already incremented upstream in the frame-walk. Adding a second increment in this ListIdentity detection block would double-count command 0x0063; it must not touch `command_counts` itself.
- [ ] In `process_pdu`, for CIP response frames (only `type_id == 0x00B2` and `cip_item_data.len() >= 4`): extract `general_status = cip_item_data[2]`; if `general_status != 0x00`: `self.error_count += 1` (aggregate — BC-2.17.008 Postcondition 2b / Invariant 2); if `!flow.error_window_active` seed `flow.error_window_start_ts = now_ts` and set `flow.error_window_active = true`; else if window expired (`wrapping_sub > 10`) reset window and clear `error_rate_emitted`; `flow.error_counts_in_window.entry(general_status).or_insert(0) += 1`; compute `total: u64 = flow.error_counts_in_window.values().sum()`; if `total > threshold && !flow.error_rate_emitted` → push T0888 Pattern B; set guard `flow.error_rate_emitted = true`
- [ ] In `process_pdu`, for CIP request frames: if GetAttribute service class && `CipPathSegment::Class(0x01)` in path segments && `type_id == 0x00B2` && `!flow.is_non_enip` → push T0888 Pattern A
- [ ] Add `mod recon { ... }` test wrapper to `tests/enip_analyzer_tests.rs` with all AC-134 tests
- [ ] Construct test ENIP+CPF+CIP byte sequences for each test vector
- [ ] Run `cargo test enip` — all recon tests pass
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings

## Test Plan

**Test file:** `tests/enip_analyzer_tests.rs`
**Test module:** `mod recon { ... }`

```
recon::test_list_identity_emits_t0846
recon::test_list_identity_one_shot_guard_multi_frame
recon::test_list_identity_respects_max_findings
recon::test_error_accumulation_increments_per_status
recon::test_error_accumulation_ignores_success
recon::test_error_window_resets_after_10s
recon::test_error_accumulation_skips_connected_item
recon::test_error_accumulation_requires_4_bytes
recon::test_aggregate_error_count_increments
recon::test_t0888_pattern_a_identity_read
recon::test_t0888_pattern_a_non_identity_no_finding
recon::test_t0888_pattern_a_connected_item_no_finding
recon::test_t0888_pattern_a_fires_per_occurrence
recon::test_t0888_pattern_b_fires_at_threshold_plus_one
recon::test_t0888_pattern_b_one_shot_guard
recon::test_t0888_pattern_b_no_fire_at_threshold
recon::test_t0888_pattern_b_threshold_zero
recon::test_non_enip_flow_suppresses_recon
```

## Previous Story Intelligence

- STORY-132 provides `parse_cpf_items`, `parse_cip_header`, `parse_cip_request_path`, `CipPathSegment` — all used directly in this story's detection logic
- STORY-133 provides `technique_info("T0846")` and `technique_info("T0888")` entries in the MITRE catalog; without STORY-133, emitting these technique IDs would fail catalog consistency checks
- T0846 is first emitted here (this is the story that uses it); STORY-133 pre-registers it in EMITTED_IDS
- The `is_non_enip` flag is set by STORY-137 (frame-walk robustness) — in STORY-134 tests, construct flow state directly with `is_non_enip=true` to test suppression without depending on STORY-137

## Architecture Compliance Rules

From ADR-010 Decision 4 (detection ordering / frame-walk) and BC-2.17.010/008/014:

1. **`is_non_enip` gate is first (ADR-010 Decision 4):** All detection code checks `flow.is_non_enip` as its first guard. No detection runs on flagged flows. This prevents false positives on TCP port 44818 traffic that is not actually ENIP.
2. **`MAX_FINDINGS` cap (BC-2.17.022, via ADR-010):** Every `push` to `all_findings` must be preceded by `self.all_findings.len() < MAX_FINDINGS` check. Never push unconditionally.
3. **T0888 Pattern B strict `>` semantics (BC-2.17.014 Invariant 3):** `total_error_count > threshold` — NOT `>=`. With default threshold=5, exactly 5 errors do NOT fire; 6 fires. Use `>` everywhere, not `>=`.
4. **F-P9-001 gate (BC-2.17.014 precondition 4):** T0888 Pattern A is only triggered for `type_id == 0x00B2` items. The check is `if item.type_id == 0x00B2 { parse_cip_header(...); /* detection */ }`. Connected items (0x00B1) are skipped without firing any detection.
5. **Error window is per-flow (BC-2.17.008):** Each `EnipFlowState` has its own `error_counts_in_window` and `error_window_start_ts`. Window expiry is checked per-PDU against the 10s threshold relative to `error_window_start_ts`.
6. **T0846 is per-flow one-shot, not per-occurrence (BC-2.17.010 invariant 1):** The first ListIdentity frame per flow emits a T0846 finding and sets `flow.list_identity_emitted = true`. Subsequent ListIdentity frames on the SAME flow increment `command_counts[0x0063]` but emit NO additional findings. A new flow from the same source resets the guard. This prevents a single scan campaign from emitting up to MAX_FINDINGS near-identical T0846 findings. Implementation MUST check `!flow.list_identity_emitted` before emitting; MUST set `flow.list_identity_emitted = true` after emitting. This is required to pass holdout HS-114 Case B.

## Library & Framework Requirements

- `std::collections::HashMap` for `error_counts_in_window: HashMap<u8, u64>` — already in Rust stdlib
- No new external dependencies

## File Structure Requirements

**Files to modify:**
- `src/analyzer/enip.rs` — add `EnipFlowState` fields; implement T0846/T0888 detection in `process_pdu`
- `tests/enip_analyzer_tests.rs` — add `mod recon { ... }` block

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/analyzer/enip.rs` detection additions | ~400 |
| `tests/enip_analyzer_tests.rs` recon mod (14 tests) | ~600 |
| **Total** | **~1,000** |

## Dependency Rationale

Wave 60; depends on STORY-132 (parse layer) and STORY-133 (MITRE catalog). T0846 is emitted here for the first time — STORY-133 must have pre-registered it in EMITTED_IDS. Wave 61's STORY-138 session lifecycle story reads `all_findings` counts that are populated here.
