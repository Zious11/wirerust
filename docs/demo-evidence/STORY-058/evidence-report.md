# STORY-058 Demo Evidence Report

**Story:** STORY-058 — Buffer Management, Record Parsing Infrastructure, Flow Lifecycle, and summarize Output
**Wave:** 18
**Strategy:** brownfield-formalization (zero production behavior change; tests formalize
existing behavior in `src/analyzer/tls.rs` — oversized-record guard, per-direction buffer
cap, nom error handling, non-handshake record skip, `summarize`, and `on_flow_close`)
**Test module:** `tests/tls_analyzer_tests.rs` (AC-001..AC-015); `tests/tls_integration_tests.rs` (AC-009)
**Date:** 2026-05-29
**Suite result:** 903/903 PASS — `cargo test --all-targets` fully green (no failures across all modules)

---

## Per-AC Evidence Table

| AC | BC | Test Function(s) | Test Module | Result | What It Proves |
|----|----|-----------------|-------------|--------|----------------|
| AC-001 | BC-2.07.004 | `test_oversized_sni_exceeds_record_payload_limit` | `tls_analyzer_tests` | PASS | When `payload_len > MAX_RECORD_PAYLOAD` (18,432), both `parse_errors` and `truncated_records` increment by 1, the direction buffer is cleared entirely, `try_parse_records` returns, no finding is emitted, and `handshakes_seen` is not incremented |
| AC-002 | BC-2.07.004 | `test_oversized_after_valid_hello_increments_both` | `tls_analyzer_tests` | PASS | `parse_errors` and `truncated_records` are always incremented together for oversized records — never independently; buffer clearing is unconditional |
| AC-003 | BC-2.07.004 | `test_record_payload_boundary_18432_vs_18433` | `tls_analyzer_tests` | PASS | `payload_len = 18,432` (boundary, equal to MAX_RECORD_PAYLOAD) is accepted with no counter increment; `payload_len = 18,433` (one over) triggers both `parse_errors` and `truncated_records` increments |
| AC-004 | BC-2.07.005 | `test_buffer_cap_appends_at_most_max_buf`, `test_buffer_cap_appends_at_most_max_buf_literal_residue` | `tls_analyzer_tests` | PASS | At most `MAX_BUF - current_buf_len` bytes from `data` are appended; if the buffer is already full no bytes are appended; no error is returned and no counter is incremented |
| AC-005 | BC-2.07.005 | `test_buffer_full_append_noop`, `test_buffer_full_append_noop_literal` | `tls_analyzer_tests` | PASS | `client_buf.len()` and `server_buf.len()` are always `<= MAX_BUF = 65,536`; cap computed via `MAX_BUF.saturating_sub(state.buf.len())` — non-panicking for any input size |
| AC-006 | BC-2.07.005 | `test_buffer_overflow_silent_no_counters` | `tls_analyzer_tests` | PASS | Buffer overflow is silent — no finding, no log line, and no counter (`parse_errors` and `truncated_records` are NOT incremented for buffer overflow) |
| AC-007 | BC-2.07.029 | `test_parse_error_counter` | `tls_analyzer_tests` | PASS | When `parse_tls_plaintext` returns `Err(_)` on a well-sized handshake record, `parse_errors` increments by 1; no finding emitted, no panic, flow state remains in `flows` HashMap |
| AC-008 | BC-2.07.029 | `test_malformed_handshake_increments_parse_errors_only` | `tls_analyzer_tests` | PASS | `parse_errors` increments ONLY for genuine parse failures (nom `Err(_)` on a handshake record); `truncated_records` is NOT incremented for a malformed-but-sized record |
| AC-009 | BC-2.07.031 | `test_summarize_output`, `test_summarize_has_all_required_fields` | `tls_analyzer_tests`, `tls_integration_tests` | PASS | `TlsAnalyzer::summarize` returns `AnalysisSummary { analyzer_name: "TLS", packets_analyzed: handshakes_seen }` with a `detail` BTreeMap containing all 7 required keys: `"cipher_suites"`, `"ja3_hashes"`, `"ja3s_hashes"`, `"parse_errors"`, `"tls_versions"`, `"top_snis"`, `"truncated_records"` |
| AC-010 | BC-2.07.031 | `test_summarize_output`, `test_summarize_top_snis_capped_at_20` | `tls_analyzer_tests` | PASS | `detail` is a `BTreeMap` (alphabetically ordered keys); `top_snis` contains at most 20 entries sorted by count descending; `version_counts` u16 keys are converted to decimal String via `k.to_string()` |
| AC-011 | BC-2.07.031 | `test_fresh_summarize_truncated_records_zero` | `tls_analyzer_tests` | PASS | `detail["parse_errors"]` and `detail["truncated_records"]` are always present as JSON numbers — both keys present even when both values are 0 (fresh analyzer) |
| AC-012 | BC-2.07.033 | `test_appdata_record_skipped_then_hello` | `tls_analyzer_tests` | PASS | After extracting a complete TLS record with `record_type != 0x16` (e.g., AppData 0x17), bytes are consumed from the buffer and the loop `continue`s — no `parse_errors` increment, no finding emitted |
| AC-013 | BC-2.07.033 | `test_within_loop_nonhandshake_skip_before_done`, `test_nonhandshake_types_0x14_0x15_0x17_0x18_all_skip_silently` | `tls_analyzer_tests` | PASS | Non-handshake records are consumed even though they are not parsed (preventing buffer stalls); types 0x14 (ChangeCipherSpec), 0x15 (Alert), 0x17 (AppData), and 0x18 (unknown) all skip silently via the within-loop path at `tls.rs:678-682` |
| AC-014 | BC-2.07.035 | `test_on_flow_close_drops_state_preserves_aggregates` | `tls_analyzer_tests` | PASS | `on_flow_close` with a present `flow_key` calls `self.flows.remove(flow_key)`, dropping `TlsFlowState` (freeing `client_buf`/`server_buf`); `sni_counts`, `ja3_counts`, `ja3s_counts`, `version_counts`, `cipher_counts`, `handshakes_seen`, `parse_errors`, and `all_findings` are all unchanged; `flows.len()` decreases by 1 |
| AC-015 | BC-2.07.035 | `test_on_flow_close_absent_key_no_panic` | `tls_analyzer_tests` | PASS | When `on_flow_close` is called with a key NOT in `flows`, `HashMap::remove` returns `None` — no panic; `_reason` parameter (CloseReason) is ignored |

---

## Test Run Output

### AC-001 — `test_oversized_sni_exceeds_record_payload_limit`

```
running 1 test
test test_oversized_sni_exceeds_record_payload_limit ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-002 — `test_oversized_after_valid_hello_increments_both`

```
running 1 test
test test_oversized_after_valid_hello_increments_both ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-003 — `test_record_payload_boundary_18432_vs_18433`

```
running 1 test
test test_record_payload_boundary_18432_vs_18433 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-004 (test 1) — `test_buffer_cap_appends_at_most_max_buf`

```
running 1 test
test test_buffer_cap_appends_at_most_max_buf ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.02s
```

### AC-004 (test 2) — `test_buffer_cap_appends_at_most_max_buf_literal_residue`

```
running 1 test
test test_buffer_cap_appends_at_most_max_buf_literal_residue ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.02s
```

### AC-005 (test 1) — `test_buffer_full_append_noop`

```
running 1 test
test test_buffer_full_append_noop ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.05s
```

### AC-005 (test 2) — `test_buffer_full_append_noop_literal`

```
running 1 test
test test_buffer_full_append_noop_literal ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.05s
```

### AC-006 — `test_buffer_overflow_silent_no_counters`

```
running 1 test
test test_buffer_overflow_silent_no_counters ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.02s
```

### AC-007 — `test_parse_error_counter`

```
running 1 test
test test_parse_error_counter ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-008 — `test_malformed_handshake_increments_parse_errors_only`

```
running 1 test
test test_malformed_handshake_increments_parse_errors_only ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-009 (unit) — `test_summarize_output`

```
running 1 test
test test_summarize_output ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-009 (integration) — `test_summarize_has_all_required_fields`

```
running 1 test
test test_summarize_has_all_required_fields ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

### AC-010 — `test_summarize_top_snis_capped_at_20`

```
running 1 test
test test_summarize_top_snis_capped_at_20 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-011 — `test_fresh_summarize_truncated_records_zero`

```
running 1 test
test test_fresh_summarize_truncated_records_zero ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-012 — `test_appdata_record_skipped_then_hello`

```
running 1 test
test test_appdata_record_skipped_then_hello ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-013 (test 1) — `test_within_loop_nonhandshake_skip_before_done`

```
running 1 test
test test_within_loop_nonhandshake_skip_before_done ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-013 (test 2) — `test_nonhandshake_types_0x14_0x15_0x17_0x18_all_skip_silently`

```
running 1 test
test test_nonhandshake_types_0x14_0x15_0x17_0x18_all_skip_silently ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-014 — `test_on_flow_close_drops_state_preserves_aggregates`

```
running 1 test
test test_on_flow_close_drops_state_preserves_aggregates ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-015 — `test_on_flow_close_absent_key_no_panic`

```
running 1 test
test test_on_flow_close_absent_key_no_panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

---

## Recording Method

**Type:** text transcript (brownfield test-formalization; no CLI/UI behavior change)
VHS recordings are not applicable — this story formalizes existing internal analyzer logic,
not an observable CLI command or UI flow. Evidence is captured via per-test `cargo test --exact`
invocations against the Rust test harness.

---

## Coverage Summary

- **ACs covered:** 15 / 15 (100%)
- **Unique test functions exercised:** 20
  - `test_oversized_sni_exceeds_record_payload_limit` (covers AC-001)
  - `test_oversized_after_valid_hello_increments_both` (covers AC-002)
  - `test_record_payload_boundary_18432_vs_18433` (covers AC-003)
  - `test_buffer_cap_appends_at_most_max_buf` (covers AC-004)
  - `test_buffer_cap_appends_at_most_max_buf_literal_residue` (covers AC-004)
  - `test_buffer_full_append_noop` (covers AC-005)
  - `test_buffer_full_append_noop_literal` (covers AC-005)
  - `test_buffer_overflow_silent_no_counters` (covers AC-006)
  - `test_parse_error_counter` (covers AC-007)
  - `test_malformed_handshake_increments_parse_errors_only` (covers AC-008)
  - `test_summarize_output` (covers AC-009, AC-010)
  - `test_summarize_has_all_required_fields` (covers AC-009, integration)
  - `test_summarize_top_snis_capped_at_20` (covers AC-010)
  - `test_fresh_summarize_truncated_records_zero` (covers AC-011)
  - `test_appdata_record_skipped_then_hello` (covers AC-012)
  - `test_within_loop_nonhandshake_skip_before_done` (covers AC-013)
  - `test_nonhandshake_types_0x14_0x15_0x17_0x18_all_skip_silently` (covers AC-013)
  - `test_on_flow_close_drops_state_preserves_aggregates` (covers AC-014)
  - `test_on_flow_close_absent_key_no_panic` (covers AC-015)
- **BCs traced:** BC-2.07.004, BC-2.07.005, BC-2.07.029, BC-2.07.031, BC-2.07.033, BC-2.07.035
- **Full suite:** `cargo test --all-targets` — 903 passed, 0 failed across all modules
