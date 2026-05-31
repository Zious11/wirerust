# STORY-057 Demo Evidence Report

**Story:** STORY-057 — SNI Edge Cases — Empty Lists, Empty Hostnames, Multi-Name, NameType, Trailing Bytes, Large SNI, and Count-Cap Decoupling
**Wave:** 19
**Strategy:** brownfield-formalization (zero production behavior change; tests formalize
existing behavior in `src/analyzer/tls.rs` — empty ServerNameList guard, empty-hostname
arm-1 routing, first-only multi-name processing, NameType discard, trailing-bytes tolerance,
16 KB large-SNI parsing, and sni_counts/all_findings decoupling)
**Test module:** `tests/tls_analyzer_tests.rs` (AC-001..AC-013)
**Date:** 2026-05-29
**Suite result:** 903/903 PASS — `cargo test --all-targets` fully green (no failures across all modules)

---

## Per-AC Evidence Table

| AC | BC | Test Function(s) | Test Module | Result | What It Proves |
|----|----|-----------------|-------------|--------|----------------|
| AC-001 | BC-2.07.022 | `test_sni_extension_with_empty_hostname_list` | `tls_analyzer_tests` | PASS | When a ClientHello SNI extension has an empty `ServerNameList`, `extract_sni` returns `None`; `sni_counts` is unchanged; no finding is emitted; `handshakes_seen` is still incremented |
| AC-002 | BC-2.07.022 | `test_sni_extension_with_empty_hostname_list` | `tls_analyzer_tests` | PASS | Empty ServerNameList is indistinguishable from no SNI extension (same state changes); the `None` return short-circuits the entire SNI handling block; EC-002 weak-cipher independence confirmed (empty-list + weak cipher: no SNI finding, weak-cipher finding fires) |
| AC-003 | BC-2.07.023 | `test_sni_with_empty_hostname_bytes` | `tls_analyzer_tests` | PASS | When the SNI list has one entry with zero-length hostname bytes, `extract_sni` classifies via arm 1 (`SniValue::Ascii("")`); `sni_counts[""]` is incremented by 1; no finding is emitted |
| AC-004 | BC-2.07.023 | `test_sni_with_empty_hostname_bytes` | `tls_analyzer_tests` | PASS | The `sni_counts` key for empty-byte SNI is `""` (empty string), NOT `"<non-utf8:...>"`; `sni_counts.contains_key("")` asserts true; the empty string vacuously satisfies all arm-1 conditions (`is_ascii()` → true, `contains_c0_or_del` → false) |
| AC-005 | BC-2.07.024 | `test_multi_name_sni_list_only_first_entry_counted` | `tls_analyzer_tests` | PASS | When a ClientHello SNI extension contains 2+ entries, `extract_sni` uses `list.first()` and processes only the first; second and subsequent entries are silently ignored; exactly one `sni_counts` entry is inserted; at most one finding is emitted |
| AC-006 | BC-2.07.024 | `test_multi_name_sni_list_only_first_entry_counted` | `tls_analyzer_tests` | PASS | With SNI list `["example.com", "evil\x01.com"]`, only `"example.com"` (arm 1, clean ASCII) is processed; the second entry's C0 bytes are never inspected; no finding is emitted (arm 1 never emits); dual-vector scenario confirmed |
| AC-007 | BC-2.07.025 | `test_non_zero_name_type_sni_entry`, `test_non_zero_name_type_with_valid_first_entry` | `tls_analyzer_tests` | PASS | When the first SNI entry has a non-zero `NameType` byte (e.g., NameType=1), the NameType is discarded via the `let Some((_, hostname))` destructure pattern; only hostname bytes are passed to the 4-way classification; behavior is identical to NameType=0 processing |
| AC-008 | BC-2.07.025 | `test_non_zero_name_type_sni_entry` | `tls_analyzer_tests` | PASS | No finding is emitted solely due to non-zero NameType; with NameType=1 and hostname=`"example.com"` (clean ASCII), `sni_counts` has exactly one entry keyed on `"example.com"` and `all_findings` is empty; arm-3 café.example proof confirmed via `test_non_zero_name_type_with_valid_first_entry` |
| AC-009 | BC-2.07.026 | `test_trailing_bytes_in_server_name_list` | `tls_analyzer_tests` | PASS | When a TLS ClientHello SNI extension has trailing bytes after the last valid hostname entry (but `parse_tls_extensions` succeeds with a non-empty list), `extract_sni` processes the first hostname entry normally; trailing bytes are silently ignored; `parse_errors` is not incremented |
| AC-010 | BC-2.07.027 | `test_large_sni_near_record_payload_limit` | `tls_analyzer_tests` | PASS | A ClientHello with a 16,384-byte clean ASCII SNI hostname (payload_len <= 18,432 = MAX_RECORD_PAYLOAD) is accepted and parsed without error; `parse_errors` is NOT incremented; the large hostname is classified via arm 1 and counted in `sni_counts`; `handshakes_seen` is incremented |
| AC-011 | BC-2.07.027 | `test_large_sni_near_record_payload_limit` | `tls_analyzer_tests` | PASS | `MAX_RECORD_PAYLOAD = 18,432` is the binding size constraint, not `MAX_BUF = 65,536`; the 16 KB SNI fits within MAX_BUF without triggering truncation; `truncated_records` is NOT incremented; no SNI-length-specific cap below MAX_RECORD_PAYLOAD exists |
| AC-012 | BC-2.07.028 | `test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity` | `tls_analyzer_tests` | PASS | When `sni_counts` is at `MAX_MAP_ENTRIES = 50,000` capacity and a new anomalous SNI arrives (not already in the map), the new key is NOT inserted into `sni_counts` (count silently dropped), but the anomaly finding IS pushed to `all_findings`; `sni_counts.len()` remains at 50,000; `all_findings.len()` increases by 1 |
| AC-013 | BC-2.07.028 | `test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity` | `tls_analyzer_tests` | PASS | Finding emission is decoupled from count insertion; the `Self::increment` call and the `match sni { ... }` finding-emission block are sequential, not conditional on each other; `all_findings` in `TlsAnalyzer` has no cap; at capacity + new anomalous SNI: count dropped, finding still fires |

---

## Test Run Output

### AC-001 / AC-002 — `test_sni_extension_with_empty_hostname_list`

```
running 1 test
test test_sni_extension_with_empty_hostname_list ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-003 / AC-004 — `test_sni_with_empty_hostname_bytes`

```
running 1 test
test test_sni_with_empty_hostname_bytes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-005 / AC-006 — `test_multi_name_sni_list_only_first_entry_counted`

```
running 1 test
test test_multi_name_sni_list_only_first_entry_counted ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-007 / AC-008 (test 1) — `test_non_zero_name_type_sni_entry`

```
running 1 test
test test_non_zero_name_type_sni_entry ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-007 / AC-008 (test 2) — `test_non_zero_name_type_with_valid_first_entry`

```
running 1 test
test test_non_zero_name_type_with_valid_first_entry ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-009 — `test_trailing_bytes_in_server_name_list`

```
running 1 test
test test_trailing_bytes_in_server_name_list ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-010 / AC-011 — `test_large_sni_near_record_payload_limit`

```
running 1 test
test test_large_sni_near_record_payload_limit ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.00s
```

### AC-012 / AC-013 — `test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity`

```
running 1 test
test test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.65s
```

---

## Recording Method

**Type:** text transcript (brownfield test-formalization; no CLI/UI behavior change)
VHS recordings are not applicable — this story formalizes existing internal analyzer logic,
not an observable CLI command or UI flow. Evidence is captured via per-test `cargo test --exact`
invocations against the Rust test harness.

---

## Coverage Summary

- **ACs covered:** 13 / 13 (100%)
- **Unique test functions exercised:** 8
  - `test_sni_extension_with_empty_hostname_list` (covers AC-001, AC-002)
  - `test_sni_with_empty_hostname_bytes` (covers AC-003, AC-004)
  - `test_multi_name_sni_list_only_first_entry_counted` (covers AC-005, AC-006)
  - `test_non_zero_name_type_sni_entry` (covers AC-007, AC-008)
  - `test_non_zero_name_type_with_valid_first_entry` (covers AC-007, EC-004 arm-3 proof)
  - `test_trailing_bytes_in_server_name_list` (covers AC-009)
  - `test_large_sni_near_record_payload_limit` (covers AC-010, AC-011)
  - `test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity` (covers AC-012, AC-013)
- **BCs traced:** BC-2.07.022, BC-2.07.023, BC-2.07.024, BC-2.07.025, BC-2.07.026, BC-2.07.027, BC-2.07.028
- **Full suite:** `cargo test --all-targets` — 903 passed, 0 failed across all modules
