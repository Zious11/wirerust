# Evidence Report — STORY-106: DNP3 DL/Transport Parse + FC Classify

**Story:** STORY-106  
**Story title:** DNP3 DL/Transport Parse + FC Classify — Pure Core (VP-023 Kani)  
**Branch:** feature/story-106-dnp3-parse-core  
**Date captured:** 2026-06-11  
**Product type:** Pure-core Rust library (no CLI/web surface — evidence is test harness + Kani output)  
**VHS recordings produced:** Yes (2 recordings)

---

## Coverage Summary

| AC | Status | Evidence artifact(s) |
|----|--------|---------------------|
| AC-001 | PASS | `AC-ALL-cargo-test-output.txt`, `AC-ALL-unit-tests.gif/.webm` |
| AC-002 | PASS | `AC-ALL-cargo-test-output.txt`, `AC-ALL-unit-tests.gif/.webm` |
| AC-003 | PASS | `AC-ALL-cargo-test-output.txt`, `AC-ALL-unit-tests.gif/.webm` |
| AC-004 | PASS | `AC-ALL-cargo-test-output.txt`, `AC-ALL-unit-tests.gif/.webm` |
| AC-005 | PASS | `AC-ALL-cargo-test-output.txt`, `AC-ALL-unit-tests.gif/.webm` |
| AC-006 | PASS | `AC-ALL-cargo-test-output.txt`, `AC-ALL-unit-tests.gif/.webm` |
| AC-007 | PASS | `AC-ALL-cargo-test-output.txt`, `AC-ALL-unit-tests.gif/.webm` |
| AC-008 | PASS | `AC-ALL-cargo-test-output.txt`, `AC-ALL-unit-tests.gif/.webm` |
| AC-009 | PASS | `AC-ALL-cargo-test-output.txt`, `AC-ALL-unit-tests.gif/.webm` |

**AC coverage: 9/9**

---

## Per-AC Test Mapping

### AC-001 — `parse_dnp3_dl_header` returns `Some` for `len >= 10` (BC-2.15.001)

**Success path tests:**
- `test_parse_dnp3_dl_header_returns_some_for_minimum_10_bytes` — canonical vector `05 64 0E C4 03 00 01 00 88 C5` → `Some { start1:0x05, start2:0x64, length:14, control:0xC4, destination:0x0003, source:0x0001 }` ✓
- `test_BC_2_15_001_minimum_length_control_frame` — 10-byte minimum frame returns Some ✓
- `test_BC_2_15_001_trailing_bytes_not_decoded` — bytes 8-9 (header CRC) not exposed as fields ✓
- `test_BC_2_15_001_invariant_parse_does_not_gate_on_sync` — `parse_dnp3_dl_header` does not validate sync (separation of parse and validate) ✓

**Result:** `test result: ok. 1 passed` (canonical test) + 3 additional BC-aligned tests passing

---

### AC-002 — `parse_dnp3_dl_header` returns `None` for `len < 10` (BC-2.15.002)

**Error path tests (negative/truncation):**
- `test_parse_dnp3_dl_header_rejects_truncated_input` — empty slice → `None`; 9-byte slice → `None`; 10-byte slice → `Some` ✓
- `test_BC_2_15_002_ec001_zero_length_no_panic` — zero-length input returns `None` without panic ✓
- `test_BC_2_15_002_ec002_nine_bytes_returns_none` — 9-byte slice returns `None` ✓
- `test_BC_2_15_002_boundary_sweep_all_short_lengths` — all lengths 0..=9 return `None` ✓

**Result:** `test result: ok. 1 passed` (canonical test) + 3 additional boundary/EC tests passing

---

### AC-003 — Little-endian DEST/SRC address decode (BC-2.15.003)

**Success path tests:**
- `test_parse_dnp3_dl_header_le_address_decode` — `[0x03, 0x00]` → 0x0003; `[0xFD, 0xFF]` → 0xFFFD; `[0x00, 0x01]` → 0x0100 (LE vs BE disambiguation) ✓
- `test_BC_2_15_003_ec003_invalid_sync_returns_some_with_raw_fields` — invalid sync still returns `Some` with raw decoded fields ✓
- `test_BC_2_15_003_ec004_broadcast_0xffff` — `[0xFF, 0xFF]` → `destination = 0xFFFF` (broadcast) ✓

**Kani coverage:** VP-023 Sub-A (verify_parse_dnp3_dl_header_safety) proves LE decode over all symbolic 10-byte inputs. See `VP-023-A-verify_parse_dnp3_dl_header_safety.txt`.

**Result:** `test result: ok. 1 passed` (canonical test) + EC tests passing

---

### AC-004 — `is_valid_dnp3_frame_header` biconditional gate (BC-2.15.004)

**Success + error path tests:**
- `test_is_valid_dnp3_frame_header_biconditional` — 6 vectors: correct sync+length → `true`; wrong START1 (0x04) → `false`; wrong START2 (0x63) → `false`; LENGTH=4 → `false`; all partial-match cases ✓
- `test_BC_2_15_004_length_zero_false` — LENGTH=0 → `false` ✓
- `test_BC_2_15_004_length_255_valid` — LENGTH=255 → `true` ✓
- `test_BC_2_15_004_ec005_length_4_rejected_by_gate` — EC-005: LENGTH=4 (below minimum 5) → `false` ✓

**Kani coverage:** VP-023 Sub-C (verify_is_valid_dnp3_frame_gate) proves biconditional for all symbolic `Dnp3DlHeader` inputs. See `VP-023-C-verify_is_valid_dnp3_frame_gate.txt`.

**Result:** `test result: ok. 1 passed` (canonical test) + additional EC tests passing

---

### AC-005 — `classify_dnp3_fc` totality over all 256 FC values (BC-2.15.005)

**Success path tests:**
- `test_classify_dnp3_fc_total` — spot-check FC=0xFF and FC=0x80 return `Unknown` ✓
- `test_BC_2_15_005_totality_sweep_all_256_values` — exhaustive sweep 0x00..=0xFF, no panic ✓
- `test_BC_2_15_005_canonical_vectors` — canonical FC vectors with expected classifications ✓

**Kani coverage:** VP-023 Sub-B (verify_classify_dnp3_fc_total) proves totality and returns-defined-variant for all 256 values symbolically. See `VP-023-B-verify_classify_dnp3_fc_total.txt`.

**Result:** `test result: ok. 1 passed` (canonical test) + exhaustive sweep passing

---

### AC-006 — FC set membership correctness (BC-2.15.006)

**Success path tests:**
- `test_classify_dnp3_fc_set_membership` — Control {0x03,0x04,0x05,0x06}, Restart {0x0D,0x0E}, Write {0x02}, Read {0x01}, Response {0x81,0x82,0x83}, FC 0x0F → Management ✓
- `test_BC_2_15_006_ec007_direct_operate_nr_is_control` — FC 0x06 (DIRECT_OPERATE_NR) → `Control` ✓
- `test_BC_2_15_006_ec008_unsolicited_response_is_response` — FC 0x82 (UNSOLICITED_RESPONSE) → `Response` ✓

**Kani coverage:** VP-023 Sub-B set-membership assertions over all 256 symbolic FC values. See `VP-023-B-verify_classify_dnp3_fc_total.txt`.

**Result:** `test result: ok. 1 passed` (canonical test) + EC tests passing

---

### AC-007 — `compute_dnp3_frame_len` arithmetic, [10,292], no overflow (BC-2.15.007)

**Success + error path tests:**
- `test_compute_dnp3_frame_len_formula` — 7 canonical vectors: LENGTH=5→Some(10), LENGTH=6→Some(13), LENGTH=21→Some(28), LENGTH=22→Some(31), LENGTH=255→Some(292) ✓
- `test_BC_2_15_007_ec005_length_4_returns_none` — `compute_dnp3_frame_len(4)` → `None` (LENGTH < 5) ✓
- `test_BC_2_15_007_ec006_length_255_returns_292` — EC-006: LENGTH=255 → `Some(292)` ✓
- `test_BC_2_15_007_length_14_direct_operate` — LENGTH=14 → correct formula result ✓
- `test_BC_2_15_007_result_bounds_all_valid_lengths` — all valid lengths 5..=255 produce results in [10,292] ✓

**Kani coverage:** VP-023 Sub-D (verify_compute_dnp3_frame_len) proves formula correctness, [10,292] bound, and no-panic over all 256 `u8` values. See `VP-023-D-verify_compute_dnp3_frame_len.txt`.

**Result:** `test result: ok. 1 passed` (canonical test) + bounds sweep passing

---

### AC-008 — FIR=1 gating + link-FC guard (BC-2.15.008)

**Success + error path tests (4 tests covering 2 paths):**

Success path (FIR=1, user-data link FC → app-FC extracted):
- `test_fir_gating_extract_on_fir1_skip_on_fir0` — transport_octet=0xC0 (FIR=1) → FC extracted; transport_octet=0x80 (FIR=0) → no extraction ✓
- `test_on_data_fir_gating_updates_counters` — `on_data` end-to-end: FIR=1 frame updates `fc_counts`/`fn_code_counts` ✓
- `test_BC_2_15_008_ec009_fir0_continuation_returns_false` — FIR=0 continuation transport_octet=0x80 → `transport_is_fir` returns `false` ✓
- `test_BC_2_15_008_fir_biconditional_all_256_transport_octets` — bit-40 biconditional sweep ✓

Error path (link-FC guard — RESET_LINK FIR=1 → no app-FC extraction):
- `test_has_user_data_link_fc_guard` — `has_user_data` predicate: 0x03 (CONFIRMED_USER_DATA) → `true`; 0x04 (UNCONFIRMED_USER_DATA) → `true`; 0x00 (RESET_LINK) → `false` ✓
- `test_on_data_fir_but_non_user_data_link_fc_no_extraction` — RESET_LINK 0x00 with FIR=1: frame counted, app-FC NOT extracted (BC-2.15.008 EC-005) ✓

**Result:** All 4+ AC-008 tests passing

---

### AC-009 — `is_non_dnp3` desync-safe bail latch (BC-2.15.009)

**Success + error path tests:**

Error path (desync detection — negative case):
- `test_desync_bail_non_dnp3_traffic` — deliver `[0xFF, 0xFE, ...]` first; assert `is_non_dnp3=true`; deliver second segment; assert no findings, no carry growth ✓
- `test_BC_2_15_009_ec010_sync_at_offset_2_triggers_bail` — EC-010: valid sync at offset 2 (not offset 0) → desync bail fires ✓
- `test_BC_2_15_009_flow_state_defaults_to_not_bailed` — initial `is_non_dnp3` = `false` ✓

Success path (valid sync does NOT bail):
- `test_BC_2_15_009_valid_sync_no_bail` — deliver `[0x05, 0x64, ...]`; assert `is_non_dnp3` remains `false` ✓

**Result:** `test result: ok. 1 passed` (canonical desync test) + EC tests passing

---

## VP-023 Kani Formal Verification (All 4 Harnesses)

All four harnesses ran with `cargo kani` under Kani Rust Verifier 0.67.0 (CaDiCaL 2.0.0 solver).

| Harness | Sub-property | BC Coverage | Result |
|---------|-------------|-------------|--------|
| `verify_parse_dnp3_dl_header_safety` | A — parse safety, None/Some, LE decode | BC-2.15.001, BC-2.15.002, BC-2.15.003 | **VERIFICATION:- SUCCESSFUL** |
| `verify_classify_dnp3_fc_total` | B — totality + set membership | BC-2.15.005, BC-2.15.006 | **VERIFICATION:- SUCCESSFUL** |
| `verify_is_valid_dnp3_frame_gate` | C — validity gate biconditional | BC-2.15.004 | **VERIFICATION:- SUCCESSFUL** |
| `verify_compute_dnp3_frame_len` | D — frame_len arithmetic, [10,292], no overflow | BC-2.15.007 | **VERIFICATION:- SUCCESSFUL** |

Detailed Kani result logs (filtered to key lines):
- `VP-023-A-verify_parse_dnp3_dl_header_safety.txt`
- `VP-023-B-verify_classify_dnp3_fc_total.txt`
- `VP-023-C-verify_is_valid_dnp3_frame_gate.txt`
- `VP-023-D-verify_compute_dnp3_frame_len.txt`

---

## Full Test Suite Run

**File:** `AC-ALL-cargo-test-output.txt`

```
running 36 tests
test story_106::test_BC_2_15_001_invariant_parse_does_not_gate_on_sync ... ok
test story_106::test_BC_2_15_001_minimum_length_control_frame ... ok
test story_106::test_BC_2_15_001_trailing_bytes_not_decoded ... ok
test story_106::test_BC_2_15_002_boundary_sweep_all_short_lengths ... ok
test story_106::test_BC_2_15_002_ec001_zero_length_no_panic ... ok
test story_106::test_BC_2_15_002_ec002_nine_bytes_returns_none ... ok
test story_106::test_BC_2_15_003_ec003_invalid_sync_returns_some_with_raw_fields ... ok
test story_106::test_BC_2_15_003_ec004_broadcast_0xffff ... ok
test story_106::test_BC_2_15_004_ec005_length_4_rejected_by_gate ... ok
test story_106::test_BC_2_15_004_length_255_valid ... ok
test story_106::test_BC_2_15_004_length_zero_false ... ok
test story_106::test_BC_2_15_005_canonical_vectors ... ok
test story_106::test_BC_2_15_005_totality_sweep_all_256_values ... ok
test story_106::test_BC_2_15_006_ec007_direct_operate_nr_is_control ... ok
test story_106::test_BC_2_15_006_ec008_unsolicited_response_is_response ... ok
test story_106::test_BC_2_15_007_ec005_length_4_returns_none ... ok
test story_106::test_BC_2_15_007_ec006_length_255_returns_292 ... ok
test story_106::test_BC_2_15_007_length_14_direct_operate ... ok
test story_106::test_BC_2_15_007_result_bounds_all_valid_lengths ... ok
test story_106::test_BC_2_15_008_ec009_fir0_continuation_returns_false ... ok
test story_106::test_BC_2_15_008_fir_biconditional_all_256_transport_octets ... ok
test story_106::test_BC_2_15_009_ec010_sync_at_offset_2_triggers_bail ... ok
test story_106::test_BC_2_15_009_flow_state_defaults_to_not_bailed ... ok
test story_106::test_BC_2_15_009_valid_sync_no_bail ... ok
test story_106::test_classify_dnp3_fc_set_membership ... ok
test story_106::test_classify_dnp3_fc_total ... ok
test story_106::test_compute_dnp3_frame_len_formula ... ok
test story_106::test_desync_bail_non_dnp3_traffic ... ok
test story_106::test_fir_gating_extract_on_fir1_skip_on_fir0 ... ok
test story_106::test_has_user_data_link_fc_guard ... ok
test story_106::test_is_valid_dnp3_frame_header_biconditional ... ok
test story_106::test_on_data_fir_but_non_user_data_link_fc_no_extraction ... ok
test story_106::test_on_data_fir_gating_updates_counters ... ok
test story_106::test_parse_dnp3_dl_header_le_address_decode ... ok
test story_106::test_parse_dnp3_dl_header_rejects_truncated_input ... ok
test story_106::test_parse_dnp3_dl_header_returns_some_for_minimum_10_bytes ... ok

test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

## VHS Recordings

This is a pure-core library story with no CLI/web surface. VHS recordings capture the test harness and Kani invocations as visual terminal evidence.

| Recording | Format | Contents |
|-----------|--------|---------|
| `AC-ALL-unit-tests.gif` | GIF (PR embed) | `cargo test --test dnp3_parse_core_tests` — all 36 tests passing |
| `AC-ALL-unit-tests.webm` | WebM (archival) | Same as above |
| `VP-023-kani-all.gif` | GIF (PR embed) | All 4 Kani harnesses — each `VERIFICATION:- SUCCESSFUL` |
| `VP-023-kani-all.webm` | WebM (archival) | Same as above |

**VHS tape scripts:** `AC-ALL-unit-tests.tape`, `VP-023-kani-all.tape`

---

## Artifact Index

```
docs/demo-evidence/STORY-106/
├── evidence-report.md                            (this file)
├── AC-ALL-cargo-test-output.txt                  (full cargo test output)
├── AC-ALL-unit-tests.tape                        (VHS script)
├── AC-ALL-unit-tests.gif                         (VHS recording — unit tests)
├── AC-ALL-unit-tests.webm                        (VHS recording — unit tests)
├── VP-023-kani-all.tape                          (VHS script)
├── VP-023-kani-all.gif                           (VHS recording — Kani harnesses)
├── VP-023-kani-all.webm                          (VHS recording — Kani harnesses)
├── VP-023-A-verify_parse_dnp3_dl_header_safety.txt
├── VP-023-B-verify_classify_dnp3_fc_total.txt
├── VP-023-C-verify_is_valid_dnp3_frame_gate.txt
└── VP-023-D-verify_compute_dnp3_frame_len.txt
```

---

## Scope Notes

- **AC-004 scope boundary:** The pure biconditional gate (`is_valid_dnp3_frame_header`) is fully covered. BC-2.15.004 PC4 (caller-side `parse_errors` increment) is deferred to STORY-107.
- **AC-008 scope boundary:** FIR=1 gating and link-FC guard for minimum-single-block frames are covered. `parse_errors` for <3-byte payloads (BC-2.15.008 EC-006) and multi-block frame-walk are STORY-107 scope.
- **AC-009 scope boundary:** Single-delivery desync latch (first segment ≥2 bytes, no sync at offset 0) is covered. Carry-accumulated sync deferral for lone-0x05 first segment (BC-2.15.009 EC-004) is STORY-107 scope.
- **VP-023 scope:** Covers BC-2.15.001..007 (four Kani-provable pure functions). BC-2.15.008 and BC-2.15.009 are effectful shell behaviors covered by unit tests only (not Kani obligations for this story).
