# STORY-046 Demo Evidence Report

**Story:** STORY-046 — HTTP Analyzer Summary Output
**Wave:** 18
**Strategy:** brownfield-formalization (zero production behavior change; tests formalize
`HttpAnalyzer::summarize()` behavior already present in `src/analyzer/http.rs`)
**Behavioral contract:** BC-2.06.023 — `summarize` Emits `AnalysisSummary` with HTTP Stats Detail Map
**Test module:** `mod bc_2_06_023_formalization` + top-level tests in `tests/http_analyzer_tests.rs`
**Date:** 2026-05-29
**Suite result:** 7/7 PASS — `cargo test --all-targets` fully green (zero failures across all modules)

**Recording method:** VHS (.gif + .webm) — VHS and ffmpeg available on this machine.
Font: `Menlo` (system macOS font, confirmed present at `/System/Library/Fonts/Menlo.ttc`).

---

## Recording

| File | Contents |
|------|----------|
| `AC-001-008-bc-2-06-023-formalization.gif` | All 7 AC tests running and passing (PR embed) |
| `AC-001-008-bc-2-06-023-formalization.webm` | Same run — archival quality |
| `AC-001-008-bc-2-06-023-formalization.tape` | VHS script source |

The single recording covers all 8 acceptance criteria in three `cargo test` invocations:
1. `test_summarize_produces_complete_output` (AC-001, AC-002)
2. `test_parse_error_in_summarize` (AC-008)
3. `bc_2_06_023_formalization::*` — 5 tests covering AC-003 through AC-007

---

## Per-AC Evidence Table

| AC | BC | Test Function | Module | Result | What It Proves |
|----|----|--------------|--------|--------|----------------|
| AC-001 | BC-2.06.023 postcondition 1 | `test_summarize_produces_complete_output` | top-level | PASS | `summarize()` returns `AnalysisSummary` with `analyzer_name = "HTTP"` and `packets_analyzed = self.transactions` (parsed response count) |
| AC-002 | BC-2.06.023 postcondition 1 detail map keys | `test_summarize_produces_complete_output` | top-level | PASS | `detail` BTreeMap contains exactly 9 keys: `"methods"`, `"non_http_flows"`, `"parse_errors"`, `"poisoned_bytes_skipped"`, `"recent_uris"`, `"status_codes"`, `"top_hosts"`, `"transactions"`, `"user_agents"` — no extras, no omissions |
| AC-003 | BC-2.06.023 postcondition 2 | `test_summarize_top_hosts_sorted_and_truncated` | `bc_2_06_023_formalization` | PASS | `top_hosts` sorted by count descending; truncated to exactly 20 entries when 25 distinct hosts are present; "high.example.com" (10 hits) ranks first |
| AC-004 | BC-2.06.023 postcondition 3 | `test_summarize_recent_uris_first_20` | `bc_2_06_023_formalization` | PASS | `recent_uris` is first 20 entries from `self.uris` in insertion order; when 25 URIs are pushed, entries 21–25 are absent; first 20 URIs are present in order |
| AC-005 | BC-2.06.023 invariant 1 | `test_summarize_btreemap_key_order_is_deterministic` | `bc_2_06_023_formalization` | PASS | BTreeMap alphabetical key ordering is stable; calling `summarize()` twice on the same analyzer produces byte-identical `detail` maps |
| AC-006 | BC-2.06.023 invariant 2 | `test_summarize_packets_analyzed_equals_transactions` | `bc_2_06_023_formalization` | PASS | After 5 requests + 3 responses, `packets_analyzed = 3` (response count only, not request count) |
| AC-007 | BC-2.06.023 invariant 4 | `test_summarize_does_not_mutate_state` | `bc_2_06_023_formalization` | PASS | `summarize()` between two `on_data` calls does not affect subsequent parsing; `transactions` after a mid-session `summarize()` equals the count accumulated without it |
| AC-008 | BC-2.06.023 edge case EC-001 | `test_parse_error_in_summarize` | top-level | PASS | Zero-traffic state: all maps empty, `transactions=0`, `parse_errors=0`, `non_http_flows=0`, `poisoned_bytes_skipped=0`, `recent_uris=[]` |

---

## Individual Test Run Transcripts

### AC-001 / AC-002 — `test_summarize_produces_complete_output`

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/http_analyzer_tests.rs (target/debug/deps/http_analyzer_tests-45d489206aff0be5)

running 1 test
test test_summarize_produces_complete_output ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 136 filtered out; finished in 0.00s
```

### AC-008 — `test_parse_error_in_summarize`

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/http_analyzer_tests.rs (target/debug/deps/http_analyzer_tests-45d489206aff0be5)

running 1 test
test test_parse_error_in_summarize ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 136 filtered out; finished in 0.00s
```

### AC-003 — `test_summarize_top_hosts_sorted_and_truncated`

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/http_analyzer_tests.rs (target/debug/deps/http_analyzer_tests-45d489206aff0be5)

running 1 test
test bc_2_06_023_formalization::test_summarize_top_hosts_sorted_and_truncated ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 136 filtered out; finished in 0.00s
```

### AC-004 — `test_summarize_recent_uris_first_20`

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/http_analyzer_tests.rs (target/debug/deps/http_analyzer_tests-45d489206aff0be5)

running 1 test
test bc_2_06_023_formalization::test_summarize_recent_uris_first_20 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 136 filtered out; finished in 0.00s
```

### AC-005 — `test_summarize_btreemap_key_order_is_deterministic`

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/http_analyzer_tests.rs (target/debug/deps/http_analyzer_tests-45d489206aff0be5)

running 1 test
test bc_2_06_023_formalization::test_summarize_btreemap_key_order_is_deterministic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 136 filtered out; finished in 0.00s
```

### AC-006 — `test_summarize_packets_analyzed_equals_transactions`

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/http_analyzer_tests.rs (target/debug/deps/http_analyzer_tests-45d489206aff0be5)

running 1 test
test bc_2_06_023_formalization::test_summarize_packets_analyzed_equals_transactions ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 136 filtered out; finished in 0.00s
```

### AC-007 — `test_summarize_does_not_mutate_state`

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/http_analyzer_tests.rs (target/debug/deps/http_analyzer_tests-45d489206aff0be5)

running 1 test
test bc_2_06_023_formalization::test_summarize_does_not_mutate_state ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 136 filtered out; finished in 0.00s
```

---

## Full Module Run (bc_2_06_023_formalization)

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/http_analyzer_tests.rs (target/debug/deps/http_analyzer_tests-45d489206aff0be5)

running 5 tests
test bc_2_06_023_formalization::test_summarize_does_not_mutate_state ... ok
test bc_2_06_023_formalization::test_summarize_btreemap_key_order_is_deterministic ... ok
test bc_2_06_023_formalization::test_summarize_packets_analyzed_equals_transactions ... ok
test bc_2_06_023_formalization::test_summarize_recent_uris_first_20 ... ok
test bc_2_06_023_formalization::test_summarize_top_hosts_sorted_and_truncated ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 132 filtered out; finished in 0.00s
```

---

## Full Suite Regression Gate

`cargo test --all-targets` — zero failures across all test modules (20 modules, 137 tests in
`http_analyzer_tests.rs` alone; all pass).

---

## Coverage Summary

- **ACs covered:** 8 / 8 (100%)
- **Tests exercised:** 7 (2 top-level + 5 in `bc_2_06_023_formalization` module)
- **BCs traced:** BC-2.06.023
- **VHS recordings:** 1 combined tape covering all 8 ACs (`.gif` + `.webm`)
- **Full suite:** `cargo test --all-targets` — 0 failures
