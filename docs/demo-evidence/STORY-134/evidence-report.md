# Demo Evidence Report — STORY-134

**Story:** ENIP Recon Detections: T0846 ListIdentity, T0888 Identity Read / Error Burst, and CIP Error Accumulation
**Story ID:** STORY-134
**Wave:** 60
**Product type:** Pure-core library (no CLI/UI surface — EtherNet/IP analyzer recon detection only)
**Recording tool:** VHS 0.11.0 (terminal recordings of `cargo test --test enip_analyzer_tests`)
**Recorded:** 2026-06-25
**Test result at recording time:** 20 passed / 0 failed / 0 ignored (mod recon)

---

## AC Coverage Map

| AC | Title | Test filter used | Artifact (GIF) | Artifact (WebM) | Tape |
|----|-------|-----------------|---------------|----------------|------|
| AC-134-001 | ListIdentity ENIP command emits T0846 finding (per-flow one-shot) | `recon::test_list_identity` | `AC-001-t0846-list-identity.gif` | `AC-001-t0846-list-identity.webm` | `AC-001-t0846-list-identity.tape` |
| AC-134-002 | CIP error responses accumulate per-status in error_counts_in_window | `recon::test_error` | `AC-002-003-cip-error-window.gif` | `AC-002-003-cip-error-window.webm` | `AC-002-003-cip-error-window.tape` |
| AC-134-003 | T0888 Pattern A — GetAttribute to Identity Object (Class 0x01) emits finding | `recon::test_t0888_pattern_a` | `AC-004-t0888-pattern-a.gif` | `AC-004-t0888-pattern-a.webm` | `AC-004-t0888-pattern-a.tape` |
| AC-134-004 | T0888 Pattern B — error burst crossing threshold emits one-shot finding | `recon::test_t0888_pattern_b\|recon::test_non_enip\|recon::test_aggregate` | `AC-005-006-t0888-pattern-b-suppression.gif` | `AC-005-006-t0888-pattern-b-suppression.webm` | `AC-005-006-t0888-pattern-b-suppression.tape` |
| AC-134-005 | is_non_enip flow flag suppresses all ENIP detections | `recon::test_non_enip` (in AC-005-006 recording) | `AC-005-006-t0888-pattern-b-suppression.gif` | `AC-005-006-t0888-pattern-b-suppression.webm` | `AC-005-006-t0888-pattern-b-suppression.tape` |
| AC-134-006 | EnipAnalyzer aggregate error_count increments on every CIP error response | `recon::test_aggregate` (in AC-005-006 recording) | `AC-005-006-t0888-pattern-b-suppression.gif` | `AC-005-006-t0888-pattern-b-suppression.webm` | `AC-005-006-t0888-pattern-b-suppression.tape` |

---

## Recordings Detail

### AC-001-t0846-list-identity

Demonstrates `EnipAnalyzer::process_pdu` ListIdentity (command=0x0063) → T0846 one-shot detection
(BC-2.17.010).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests 'recon::test_list_identity'`
- `test_list_identity_emits_t0846`: first ListIdentity frame on a new flow → exactly one Finding
  with category=Reconnaissance, verdict=Likely, confidence=High, mitre_techniques=["T0846"],
  summary="EtherNet/IP ListIdentity broadcast observed: network-wide device enumeration (T0846)"
- `test_list_identity_one_shot_guard_multi_frame`: 5 ListIdentity frames on the same flow →
  exactly 1 finding total; `command_counts[0x0063] == 5`; `list_identity_emitted == true`
- `test_list_identity_respects_max_findings`: ListIdentity with `all_findings.len() == MAX_FINDINGS`
  → no finding pushed; guard remains false
- All 3 tests pass green

**Tests in recording:**
- `test_list_identity_emits_t0846`
- `test_list_identity_one_shot_guard_multi_frame`
- `test_list_identity_respects_max_findings`

---

### AC-002-003-cip-error-window

Demonstrates CIP error-response accumulation and 10-second window management including the
ts=0 regression fix (BC-2.17.008).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests 'recon::test_error'`
- `test_error_accumulation_increments_per_status`: two distinct error codes (0x08, 0xFF) →
  each independently tracked in `error_counts_in_window`; `error_window_active` set on first error
- `test_error_accumulation_ignores_success`: general_status=0x00 → no increment, no window seed
- `test_error_window_resets_after_10s`: errors at ts=0..4 then ts=15 → window reset;
  `error_counts_in_window` cleared, `error_rate_emitted` reset to false
- `test_error_window_resets_after_10s_from_ts_zero`: ts=0 regression — error at ts=0 seeds
  window with `error_window_active=true`; subsequent errors at ts=5 accumulate (no reset);
  error at ts=11 triggers reset (wrapping_sub=11 > 10)
- `test_error_accumulation_skips_connected_item`: type_id=0x00B1 (Connected Data Item) → no
  error counter update (F-P9-001 / BC-2.17.008 precondition 2 hard scope gate)
- `test_error_accumulation_requires_4_bytes`: cip_item_data.len() < 4 → no extraction
- All 6 tests pass green

**Tests in recording:**
- `test_error_accumulation_increments_per_status`
- `test_error_accumulation_ignores_success`
- `test_error_window_resets_after_10s`
- `test_error_window_resets_after_10s_from_ts_zero`
- `test_error_accumulation_skips_connected_item`
- `test_error_accumulation_requires_4_bytes`

---

### AC-004-t0888-pattern-a

Demonstrates T0888 Pattern A: CIP GetAttribute request targeting Identity Object (Class 0x01)
via 0x00B2 Unconnected Data Item emits a per-occurrence Reconnaissance finding (BC-2.17.014).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests 'recon::test_t0888_pattern_a'`
- `test_t0888_pattern_a_identity_read`: GetAttributeSingle to Class 0x01 via 0x00B2 →
  One Finding (category=Reconnaissance, verdict=Likely, confidence=High, mitre=["T0888"],
  summary="CIP Identity Object attribute read: single-device reconnaissance (T0888)")
- `test_t0888_pattern_a_non_identity_no_finding`: GetAttributeSingle to Class 0x04 (Assembly)
  via 0x00B2 → no finding
- `test_t0888_pattern_a_connected_item_no_finding`: GetAttributeSingle to Class 0x01 via
  0x00B1 (Connected Data Item) → no finding (F-P9-001 gate)
- `test_t0888_pattern_a_fires_per_occurrence`: three identical GetAttribute frames on the same
  flow → three separate findings (Pattern A is not one-shot)
- All 4 tests pass green

**Tests in recording:**
- `test_t0888_pattern_a_identity_read`
- `test_t0888_pattern_a_non_identity_no_finding`
- `test_t0888_pattern_a_connected_item_no_finding`
- `test_t0888_pattern_a_fires_per_occurrence`

---

### AC-005-006-t0888-pattern-b-suppression

Demonstrates T0888 Pattern B (CIP error-burst one-shot), `is_non_enip` flow suppression, and
aggregate `EnipAnalyzer.error_count` accumulation (BC-2.17.014 Pattern B; BC-2.17.008 Postcondition 2b).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests` with filter covering pattern_b, non_enip, aggregate
- `test_t0888_pattern_b_fires_at_threshold_plus_one`: threshold=5 (default); 5 errors → no finding;
  6th error → ONE Finding (verdict=Possible, confidence=Medium, mitre=["T0888"],
  summary="CIP error-response burst: 6 error responses in 10s window — possible service enumeration (T0888)")
- `test_t0888_pattern_b_one_shot_guard`: after Pattern B fires, 7th error → NO additional finding;
  `error_rate_emitted == true` guard holds
- `test_t0888_pattern_b_no_fire_at_threshold`: exactly 5 errors with threshold=5 → no finding
  (strict `>` semantics per BC-2.17.014 Invariant 3)
- `test_t0888_pattern_b_threshold_zero`: threshold=0; first error (count=1 > 0) → fires immediately
- `test_non_enip_flow_suppresses_recon`: flow constructed with `is_non_enip=true`; both ListIdentity
  and GetAttribute-to-Identity frames → zero findings
- `test_aggregate_error_count_increments`: N error responses across multiple flows →
  `analyzer.error_count == N` (aggregate lifetime counter, not reset between flows)
- All 6 tests pass green

**Tests in recording:**
- `test_t0888_pattern_b_fires_at_threshold_plus_one`
- `test_t0888_pattern_b_one_shot_guard`
- `test_t0888_pattern_b_no_fire_at_threshold`
- `test_t0888_pattern_b_threshold_zero`
- `test_non_enip_flow_suppresses_recon`
- `test_aggregate_error_count_increments`

---

## Full Recon Test Suite Summary

All 20 tests in `mod recon` pass at recording time:

```
test recon::test_aggregate_error_count_increments ... ok
test recon::test_error_accumulation_ignores_success ... ok
test recon::test_error_accumulation_increments_per_status ... ok
test recon::test_error_accumulation_requires_4_bytes ... ok
test recon::test_error_accumulation_skips_connected_item ... ok
test recon::test_error_window_resets_after_10s ... ok
test recon::test_error_window_resets_after_10s_from_ts_zero ... ok
test recon::test_list_identity_emits_t0846 ... ok
test recon::test_list_identity_one_shot_guard_multi_frame ... ok
test recon::test_list_identity_respects_max_findings ... ok
test recon::test_non_enip_flow_suppresses_recon ... ok
test recon::test_process_pdu_canonical_sendrr_cpf_offset ... ok
test recon::test_t0888_pattern_a_connected_item_no_finding ... ok
test recon::test_t0888_pattern_a_fires_per_occurrence ... ok
test recon::test_t0888_pattern_a_identity_read ... ok
test recon::test_t0888_pattern_a_non_identity_no_finding ... ok
test recon::test_t0888_pattern_b_fires_at_threshold_plus_one ... ok
test recon::test_t0888_pattern_b_no_fire_at_threshold ... ok
test recon::test_t0888_pattern_b_one_shot_guard ... ok
test recon::test_t0888_pattern_b_threshold_zero ... ok

test result: ok. 20 passed; 0 failed; 0 ignored
```

---

## Deferred / Not Applicable

None. All 6 ACs have recorded demo coverage. AC-134-003 story numbering (formerly labelled
AC-134-003 in the story file but covering T0888 Pattern A, now mapped as AC-004 in artifact
naming for clarity) and AC-134-005/006 are bundled in the final recording group as both
tests are covered by the same `cargo test` invocation filter.
