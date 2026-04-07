# HTTP Parse Error Counter and Finding Design

**Issue:** #17 — fix: add HTTP parse error counter and surface in summary
**Scope:** Production changes to `src/analyzer/http.rs` + new tests in `tests/http_analyzer_tests.rs`.

## Problem

`httparse` errors in the HTTP analyzer are silently discarded. Both `parse_one_request` (line 35) and `parse_one_response` (line 53) map `Err(_) => Err(())`, losing all error variant information. The callers (`try_parse_requests` line 301, `try_parse_responses` line 330) clear the buffer and return with no counter or diagnostic. The `summarize()` output gives false confidence that no malformed HTTP exists.

`httparse` 1.10.1 defines 7 error variants: `HeaderName`, `HeaderValue`, `NewLine`, `Status`, `Token`, `TooManyHeaders`, `Version`. The enum is **not** `#[non_exhaustive]`, implements `Display` and `Debug`, and is `Copy + Clone + PartialEq + Eq`.

## Approach

### 1. Preserve Error Variant in Return Type

Change `parse_one_request` and `parse_one_response` return types from `Result<_, ()>` to `Result<_, httparse::Error>`. The `Err(_) => Err(())` arm becomes `Err(e) => Err(e)`, which can be simplified to removing the `Err` match arm entirely and using `?` or just `Err(e) => Err(e)`.

### 2. Add Aggregate Parse Error Counter

Add `parse_errors: u64` to `HttpAnalyzer`. Increment on any `httparse::Error` variant, for both requests and responses. Single aggregate counter (not split by direction) — matches Suricata's `http.error` pattern.

### 3. Surface in `summarize()`

Add `"parse_errors"` key to the `detail` HashMap in `summarize()`, with the counter value.

### 4. Generate Finding for `TooManyHeaders`

When `httparse::Error::TooManyHeaders` is encountered (request or response), generate a `Finding`:

- **Category:** `ThreatCategory::Anomaly`
- **Verdict:** `Verdict::Inconclusive` — high false positive rate from legitimate cookie jars, proxy chains adding headers
- **Confidence:** `Confidence::Medium`
- **MITRE:** `T1499.002` (Service Exhaustion Flood) — TooManyHeaders maps to resource exhaustion via header flooding, not T1190 (which requires exploiting a software vulnerability)
- **Summary:** `"Excessive HTTP headers exceeded parser limit (possible DoS or header-based attack)"`
- **Evidence:** direction (request/response) indicated in evidence

Other error variants (`HeaderName`, `HeaderValue`, `NewLine`, `Status`, `Token`, `Version`) increment the counter only — they indicate malformed traffic, not specific attacks at individual occurrence level.

### 5. Add `parse_error_count()` Accessor

Public method on `HttpAnalyzer` returning `u64`, for test assertions.

## Changes

### `src/analyzer/http.rs`

| Location | Change |
|----------|--------|
| `parse_one_request` (line 22-37) | Return type `Result<_, httparse::Error>`, change `Err(_) => Err(())` to `Err(e) => Err(e)` |
| `parse_one_response` (line 44-55) | Same return type change |
| `HttpAnalyzer` struct (line 86-95) | Add `parse_errors: u64` field |
| `HttpAnalyzer::new()` (line 104-115) | Initialize `parse_errors: 0` |
| `try_parse_requests` (line 264-310) | Match on `Some(Err(e))` instead of `Some(Err(()))`, increment `self.parse_errors`, generate finding if `e == httparse::Error::TooManyHeaders` |
| `try_parse_responses` (line 312-339) | Same error handling change |
| `summarize()` (line 384-420) | Add `parse_errors` to detail map |
| New method | `pub fn parse_error_count(&self) -> u64` |

### `tests/http_analyzer_tests.rs`

| Test | Description |
|------|-------------|
| `test_parse_error_increments_counter` | Send malformed HTTP (e.g. `"NOT_HTTP\r\n\r\n"`), assert `parse_error_count() == 1` |
| `test_parse_error_in_response` | Send malformed response data, assert counter increments |
| `test_parse_error_clears_buffer_and_continues` | Send malformed request then valid request, assert counter == 1 and valid request still parsed |
| `test_too_many_headers_generates_finding` | Programmatically build a request with 97 headers (exceeds `MAX_HEADERS=96`), assert finding with `ThreatCategory::Anomaly`, `Confidence::Medium`, MITRE `T1499.002` |
| `test_parse_error_in_summarize` | Send malformed request, assert `summarize().detail["parse_errors"]` reflects count |
| `test_normal_request_no_parse_errors` | Send valid HTTP, assert `parse_error_count() == 0` |

## Not In Scope

- Splitting counter by request vs response (Suricata uses aggregate)
- Findings for non-TooManyHeaders error variants (benign malformed traffic at individual level)
- Changes to `Finding` struct
- Changes to reassembly engine or other analyzers
