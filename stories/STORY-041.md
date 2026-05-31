---
document_type: story
story_id: "STORY-041"
epic_id: "E-4"
version: "1.8"
status: completed
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.001.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.002.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.003.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.004.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.026.md
input-hash: "3df5dc6"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-033, STORY-071]
blocks: [STORY-042, STORY-043, STORY-044, STORY-045, STORY-046]
behavioral_contracts:
  - BC-2.06.001
  - BC-2.06.002
  - BC-2.06.003
  - BC-2.06.004
  - BC-2.06.026
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 15
target_module: src/analyzer/http.rs
subsystems: [SS-06]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **Execute:** `/vsdd-factory:deliver-story STORY-041`

# STORY-041: HTTP/1.1 Request/Response Parsing and Core Statistics

## Narrative
- **As a** forensic analyst
- **I want to** have all HTTP/1.1 and HTTP/1.0 request and response headers parsed from TCP stream data — including pipelined requests, partial buffering until complete, and per-response transaction counting
- **So that** all downstream threat-detection rules and summary statistics have accurate, well-structured input from the `HttpAnalyzer`

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.06.001 | Parse Complete HTTP/1.1 Request with Method/URI/Version/Host/UA |
| BC-2.06.002 | Parse Pipelined Requests with Independent Per-Request Counting |
| BC-2.06.003 | Partial Requests Buffered Until Complete; Not Counted Until Full |
| BC-2.06.004 | Parse HTTP/1.1 Responses with Status Code Counting |
| BC-2.06.026 | Header Values Extracted via from_utf8_lossy.trim(); Raw Bytes Preserved |

## Acceptance Criteria

### AC-001 (traces to BC-2.06.001 postcondition 1-7)
When a complete HTTP/1.1 or HTTP/1.0 request is parsed via httparse, the `methods` map gains/increments an entry for the method, `hosts` map gains/increments for the trimmed Host value if present, `user_agents` gains/increments for the trimmed UA if present, the URI is appended to `uris` if below `MAX_URIS`, consumed header bytes are drained from `request_buf`, `request_error_count` is reset to 0, and `check_request_detections` is invoked.
- **Postcondition delegation notes:**
  - Postconditions 1-4 + 7 are directly asserted by `test_BC_2_06_001_complete_request_updates_all_counters`.
  - Postcondition 5 (buf drained) is asserted by `test_BC_2_06_001_consumed_bytes_drained_from_buf` (companion).
  - Postcondition 6 (`request_error_count` reset to 0) is asserted by `test_BC_2_06_002_request_error_count_reset_after_success` (cross-AC delegation — covers the POISON_THRESHOLD reset mechanic shared with BC-2.06.002 invariant 1).
- **Test:** `test_BC_2_06_001_complete_request_updates_all_counters` (in `mod bc_2_06_formalization`)
- **Companion tests:** `test_BC_2_06_001_consumed_bytes_drained_from_buf`, `test_BC_2_06_001_request_parse_does_not_increment_transactions`, `test_BC_2_06_001_http10_parsed_without_host_finding`, `test_BC_2_06_001_absent_optional_headers_produce_no_map_entries`

### AC-002 (traces to BC-2.06.026 invariant 2-3)
Header field values for Host and User-Agent are extracted via `String::from_utf8_lossy(h.value).trim().to_string()`. Per invariant 2, `from_utf8_lossy` guarantees the stored string is valid UTF-8 (non-UTF-8 bytes become U+FFFD). Per invariant 3, `.trim()` removes ASCII whitespace (space and tab; LF/CR are unreachable per httparse C0 rejection — see BC-2.06.026 invariant 3 note). Bare LF (`\n`, C0 0x0A) is rejected at the httparse layer before `trim()` is reached — httparse rejects C0 control bytes in header values — so LF trimming is not an observable behavior of this contract (space + tab coverage is sufficient).

Round-1 added explicit tab (`\t`) assertions to `test_BC_2_06_026_header_utf8_lossy_whitespace_trimmed`, confirming invariant 3 coverage beyond space-only. Invariant 2 is covered indirectly by `test_BC_2_06_026_non_utf8_header_value_replaced_with_replacement_char` (U+FFFD presence in the output implies the result is valid UTF-8).
- **Test:** `test_BC_2_06_026_header_utf8_lossy_whitespace_trimmed` (in `mod bc_2_06_formalization`)
- **Companion tests:** `test_BC_2_06_026_non_utf8_header_value_replaced_with_replacement_char`

### AC-003 (traces to BC-2.06.002 postcondition 1-5 + invariant 3)
`try_parse_requests` operates as an inner loop: each successful parse drains consumed bytes, increments method/host/UA/URI counters independently, triggers anomaly detection per request, and the loop exits when the buffer is exhausted, partially filled, or an error occurs. Per BC-2.06.002 invariant 3, detection findings are NOT aggregated across pipelined requests in a single call — each request's anomaly detection fires independently, and its findings are emitted per-request rather than being collected into a batch for the whole loop invocation.
- **Test:** `test_BC_2_06_002_pipelined_requests_each_counted_independently` (in `mod bc_2_06_formalization`)
- **Companion tests:** `test_BC_2_06_002_pipelined_detections_per_request_not_aggregated`, `test_BC_2_06_002_pipelined_loop_stops_on_partial_tail`

### AC-004 (traces to BC-2.06.002 invariant 1-2)
`request_error_count` is reset to 0 after each successful parse within the pipelined loop, and the `had_success` flag prevents error counting for body bytes that follow a successfully parsed header.
- **Test:** `test_BC_2_06_002_request_error_count_reset_after_success` (in `mod bc_2_06_formalization`)
- **Companion tests:** `test_BC_2_06_002_had_success_suppresses_request_body_byte_errors`

### AC-005 (traces to BC-2.06.003 postcondition 1-5)
When httparse returns `Status::Partial`, no method/host/UA/URI counters are updated, no anomaly detection fires, `request_buf` retains the partial bytes unchanged, and `request_error_count` is not incremented. Per BC-2.06.003 postcondition 5, a subsequent `on_data` call with completion bytes will append to the buffered partial and complete the parse — counted as a single request.
- **Test:** `test_BC_2_06_003_partial_request_leaves_counters_unchanged` (in `mod bc_2_06_formalization`)
- **Companion tests:** `test_BC_2_06_003_partial_request_no_anomaly_detection`

### AC-006 (traces to BC-2.06.003 invariant 1)
`Status::Partial` is distinct from `Err` — it does not increment `parse_errors` and does not advance `request_error_count` toward the poison threshold.
- **Test:** `test_BC_2_06_003_partial_not_counted_as_error` (in `mod bc_2_06_formalization`)

### AC-007 (traces to BC-2.06.004 postcondition 1-4 + invariant 4)
When a complete HTTP response header is parsed, `transactions` is incremented by 1, `status_codes[status_code]` is incremented (using `unwrap_or(0)` for None codes), consumed bytes are drained from `response_buf`, and `response_error_count` is reset to 0. Per BC-2.06.004 invariant 4 (added v1.5), the response-side `had_success` guard at http.rs:462 prevents body bytes (NUL-injected) after a successful response header parse from inflating `parse_errors`. Mental-deletion of the guard verified by `test_BC_2_06_004_had_success_suppresses_response_body_byte_errors`.
- **Test:** `test_BC_2_06_004_response_parse_increments_transactions_and_status_code` (in `mod bc_2_06_formalization`)
- **Companion tests:** `test_BC_2_06_004_response_buf_drained_enables_pipelined_parsing`, `test_BC_2_06_004_well_formed_404_response_status_code_counted` (cross-codes coverage: 200 in primary test, 304 in pipelined test, 404 here — proves status_codes is keyed dynamically by httparse-returned code value, not hardcoded to 200), `test_BC_2_06_004_had_success_suppresses_response_body_byte_errors` (response-side analog [in try_parse_responses at src/analyzer/http.rs:462] of the request-side had_success guard [in try_parse_requests at src/analyzer/http.rs:404]). Mental-deletion verified: removing the `if !had_success` guard at line 462 would cause body bytes (NUL-injected) after a successful response header parse to inflate `parse_errors` and the test would fail.

### AC-008 (traces to BC-2.06.004 invariant 1)
`transactions` counts parsed HTTP RESPONSES only — parsing requests does NOT increment `transactions`. The `summarize()` method maps `packets_analyzed = self.transactions`.
- **Test:** `test_BC_2_06_004_transactions_counts_responses_not_requests` (in `mod bc_2_06_formalization`)
- **Companion tests:** `test_BC_2_06_004_summarize_packets_analyzed_equals_transactions`

### AC-009 (traces to BC-2.06.026 postcondition 1, 4 + invariant 1)
`find_header` performs case-insensitive name matching (`eq_ignore_ascii_case`), returns `Some(trimmed_lossy_string)` for present headers with values, and returns `None` for absent headers. The raw URI from `req.path` flows into detection code without escaping.
- **Test:** `test_BC_2_06_026_find_header_case_insensitive_match` (in `mod bc_2_06_formalization`)
- **Companion tests:** `test_BC_2_06_026_find_header_returns_none_for_absent_header`

### AC-010 (traces to BC-2.06.026 invariant 4)
No escape function is called at parse time — raw URI bytes from `req.path` flow directly into detection code and into `Finding.evidence` per ADR 0003 / INV-4.
- **Test:** `test_http_finding_c1_csi_escaped_by_terminal_reporter` (integration, in reporter_tests.rs)
- **Companion tests:** `test_BC_2_06_026_raw_uri_bytes_preserved_in_finding_evidence` (unit, in `mod bc_2_06_formalization`)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| parse_one_request | src/analyzer/http.rs:35-50 | effectful-shell (mutates HttpAnalyzer state) |
| try_parse_requests | src/analyzer/http.rs:359-438 | effectful-shell (pipelined loop) |
| try_parse_responses | src/analyzer/http.rs:440-497 | effectful-shell |
| find_header | src/analyzer/http.rs:70-75 | pure-core |
| HttpFlowState | src/analyzer/http.rs:82-90 | effectful-shell (mutable per-flow state) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | HTTP/1.0 request (version byte == 0) | Parsed normally; Host checks exempt for missing Host |
| EC-002 | Request with both Host and User-Agent | Both values stored in respective maps |
| EC-003 | Request with no User-Agent | user_agent field is None; no UA map entry added |
| EC-004 | Request with no Host (HTTP/1.0) | host field is None; no hosts map entry; no finding |
| EC-005 | Two complete requests in one buffer | Loop parses both; each counted independently |
| EC-006 | Partial request (no \r\n\r\n) | Buffer retained; no stats updated |
| EC-007 | Response with httparse code==None | status_codes[0] incremented; transactions incremented — DEFENSIVE PATH: `parse_one_response` uses `resp.code.unwrap_or(0)`; empirically httparse rejects status lines without numeric codes via `Err(InvalidStatus)`, so this branch may be unreachable via public on_data API (see W15.D1). |
| EC-008 | Two pipelined responses | transactions=2; status_codes incremented twice |
| EC-009 | Host value with non-UTF-8 bytes | Stored with U+FFFD replacement |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/analyzer/http.rs (find_header) | pure-core | No I/O, no global state; pure string extraction |
| src/analyzer/http.rs (parse_one_request, try_parse_requests, try_parse_responses) | effectful-shell | Mutates HttpAnalyzer.methods, hosts, user_agents, uris, flows HashMap |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| Referenced code (http.rs relevant sections) | ~8,000 |
| Test files (http_analyzer_tests.rs relevant) | ~4,000 |
| BC files (5 BCs) | ~5,000 |
| Tool outputs overhead | ~2,000 |
| **Total** | **~22,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~11%** |

## Tasks (MANDATORY)

1. [x] Write 24 BC-prefixed tests for AC-001 through AC-010 in `mod bc_2_06_formalization` (test-writer) — DONE; 24 tests added (+694 lines to tests/http_analyzer_tests.rs)
2. [x] Red Gate verdict: formalization-confirms-existing — 8 basis tests already passing; 23 new BC-prefixed tests confirm brownfield behavior without src/ changes
3. [x] Verify `parse_one_request` at src/analyzer/http.rs:35-50 satisfies BC-2.06.001 postconditions 1-7 via 5 BC-2.06.001 tests in mod bc_2_06_formalization (brownfield-formalization — no src/ changes)
4. [x] Verify `try_parse_requests` at src/analyzer/http.rs:359-438 satisfies BC-2.06.002 postconditions 1-5 + invariants 1-3 via 5 BC-2.06.002 tests (including pipelined detection isolation per invariant 3)
5. [x] Verify partial buffering at src/analyzer/http.rs:401-402 (request Partial arm) and src/analyzer/http.rs:459-460 (response Partial arm) satisfies BC-2.06.003 postconditions 1-5 via 3 BC-2.06.003 tests
6. [x] Verify `try_parse_responses` at src/analyzer/http.rs:440-497 satisfies BC-2.06.004 postconditions 1-4 + invariants 1, 4 via 6 BC-2.06.004 tests (invariant 4 = response-side had_success guard at http.rs:462; added in v1.5)
7. [x] Verify `find_header` at src/analyzer/http.rs:70-75 (header lookup) and parse-time non-escape per ADR 0003 / INV-4 satisfies BC-2.06.026 postconditions 1-4 + invariants 1-4 via 5 BC-2.06.026 tests
8. [x] Run all tests; verify all pass (Green Gate stamped at worktree commit 7cfff3f — reinforced by Pass-1 remediation at dbb60c0)
9. [x] Verify purity boundaries (find_header pure-core; parse functions effectful-shell — confirmed by Architecture Mapping table)
10. [ ] Update STATE.md (deferred to wave close per state-manager dispatch)

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A — first story in E-4 | N/A | N/A | N/A |
| STORY-033 | BC-prefixed test naming convention (`test_BC_S_SS_NNN_*`); indirect observability via public summarize/summarize_request APIs; `active_flows_len_for_testing` seam pattern | Tests prove behavior through observable outputs, not internal state | STORY-041 reused existing public API only — no additive seams needed; brownfield-formalization invariant satisfied (ZERO src/ changes) |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Raw URI bytes must NOT be escaped at parse time | ADR 0003 / INV-4 | Code review: confirm no escape call in parse_one_request or find_header |
| `transactions` increments only on response parse, not request parse | BC-2.06.004 invariant 1 | Unit test: AC-008 |
| `MAX_HEADERS = 96` is the httparse capacity | BC-2.06.001 precondition 4 | Code review: confirm httparse config |
| `had_success` flag prevents body bytes from counting as parse errors | BC-2.06.002 invariant 2 / BC-2.06.004 invariant 4 / BC-2.06.020 | Unit tests: AC-004 (request, src/analyzer/http.rs:404), AC-007 (response, src/analyzer/http.rs:462) |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| httparse | (as pinned in Cargo.toml) | HTTP/1.x header parsing (Status::Complete / Partial / Err) |
| Rust std | 2024 edition (stable) | String::from_utf8_lossy, drain, BTreeMap |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/http.rs | NO CHANGES (brownfield invariant) | All parsing functions pre-existed; zero src/ modifications required or permitted |
| tests/http_analyzer_tests.rs | modify | 24 BC-prefixed tests in `mod bc_2_06_formalization`; 8 basis tests preserved (+694 lines) |

## Changelog

| Version | Date | Change |
|---------|------|--------|
| v1.0 | 2026-05-21 | Initial story decomposition |
| v1.1 | 2026-05-28 | DF-AC-TEST-NAME-SYNC-001 v1 sync — AC Test lines bound to actual `fn test_BC_2_06_*` names (23 tests); status draft→in-progress (F-W15P2-001: +1 net new test added in Round-3 bringing total to 24) |
| v1.2 | 2026-05-28 | Pass-1 remediation — Tasks 3-7 reframed as brownfield verification steps [x]; AC-002 traces extended to invariant 1-3 (tab/LF/CR whitespace coverage confirmed); AC-003 traces extended to include invariant 3 (no detection aggregation); AC-009 trace corrected to postcondition 1, 4 + invariant 1; HttpFlowState line range cited (82-90); AC-001 postcondition delegation notes added; AC-007 companion test renamed sync with Round-1 test commit dbb60c0 (status_code_none_mapped_to_zero → well_formed_404_response_status_code_counted). Captures W15.D1 + W15.D2 deferred findings for STATE.md. |
| v1.3 | 2026-05-28 | Pass-2 remediation — EC-007 marked defensive (W15.D1); AC-005 narrative extended to postcondition 5 (buffered partial completes as single request); Task 5 line range corrected to Partial arms (401-402 request, 459-460 response); Task 7 test count + invariant range corrected (5 tests, inv 1-4); AC-007 companion role clarified (cross-codes coverage: 200/304/404); AC-002 narrative finalized — LF rejected at httparse layer (C0 control bytes rejected), space + tab coverage confirmed sufficient (F-W15P2-004/005/006/007/010). |
| v1.4 | 2026-05-28 | Pass-3 remediation — AC-004 companion synced to renamed request-side test (F-W15P3-001); response-side had_success test added as AC-007 companion (F-W15P3-004); Tasks 1+6 counts updated 23→24 and 5→6 to match Round-3 test additions (F-W15P3-003); v1.1 changelog extended to mention F-W15P2-001 +1 net new test in Round-3. |
| v1.5 | 2026-05-28 | Pass-4 remediation — AC-002 trace narrowed to invariant 2-3 + paraphrase corrected (F-W15P4-002); AC-007 companion grammar fixed re. guard direction + locations (F-W15P4-003); v1.X changelog provenance note corrected (F-W15P4-004); AC-007 trace extended with invariant 4 (BC-2.06.004 v1.5 landed). |
| v1.6 | 2026-05-28 | Pass-5 remediation — sibling-sweep cascade from BC-2.06.004 v1.5 invariant 4: AC-004 trace extended to invariant 1-2 (F-W15P5-001); Task 6 + Arch Compliance Rules row updated to reflect both request-side + response-side had_success guards (F-W15P5-004/005); AC-007 narrative extended to describe invariant 4 (O-1). |
| v1.8 | 2026-05-28 | DF-SIBLING-SWEEP-001 v4 propagation — BC-2.06.004 v1.7 (reciprocal Related-BC to BC-2.06.020 added); input-hash recomputed: `518f2d5` → `3d05d98` (sha256 over sorted cited-BC files BC-2.06.001/002/003/004/026, first 7 chars). No AC citation changes required. |
| v1.9 | 2026-05-29 | input-hash corrected via canonical bin/compute-input-hash --update (prior value `3d05d98` was hand-computed sha256 over sorted inputs-file list; tool uses MD5 over inputs-order file list). New value: `518f2d5`. |
