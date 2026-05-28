---
document_type: story
story_id: "STORY-041"
epic_id: "E-4"
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.001.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.002.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.003.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.004.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.026.md
input-hash: "518f2d5"
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
- **Test:** `test_parse_get_request`

### AC-002 (traces to BC-2.06.001 invariant 1)
Header field values for Host and User-Agent are extracted via `String::from_utf8_lossy(h.value).trim().to_string()` — non-UTF-8 bytes are replaced with U+FFFD and surrounding whitespace is stripped.
- **Test:** `test_header_utf8_lossy_trim`

### AC-003 (traces to BC-2.06.002 postcondition 1-5)
`try_parse_requests` operates as an inner loop: each successful parse drains consumed bytes, increments method/host/UA/URI counters independently, triggers anomaly detection per request, and the loop exits when the buffer is exhausted, partially filled, or an error occurs.
- **Test:** `test_parse_pipelined_requests`

### AC-004 (traces to BC-2.06.002 invariant 1)
`request_error_count` is reset to 0 after each successful parse within the pipelined loop, and the `had_success` flag prevents error counting for body bytes that follow a successfully parsed header.
- **Test:** `test_pipelined_error_count_reset`

### AC-005 (traces to BC-2.06.003 postcondition 1-5)
When httparse returns `Status::Partial`, no method/host/UA/URI counters are updated, no anomaly detection fires, `request_buf` retains the partial bytes unchanged, and `request_error_count` is not incremented.
- **Test:** `test_parse_partial_request`

### AC-006 (traces to BC-2.06.003 invariant 1)
`Status::Partial` is distinct from `Err` — it does not increment `parse_errors` and does not advance `request_error_count` toward the poison threshold.
- **Test:** `test_partial_not_counted_as_error`

### AC-007 (traces to BC-2.06.004 postcondition 1-4)
When a complete HTTP response header is parsed, `transactions` is incremented by 1, `status_codes[status_code]` is incremented (using `unwrap_or(0)` for None codes), consumed bytes are drained from `response_buf`, and `response_error_count` is reset to 0.
- **Test:** `test_parse_response`

### AC-008 (traces to BC-2.06.004 invariant 1)
`transactions` counts parsed HTTP RESPONSES only — parsing requests does NOT increment `transactions`. The `summarize()` method maps `packets_analyzed = self.transactions`.
- **Test:** `test_parse_pipelined_responses`

### AC-009 (traces to BC-2.06.026 postcondition 1-4)
`find_header` performs case-insensitive name matching (`eq_ignore_ascii_case`), returns `Some(trimmed_lossy_string)` for present headers with values, and returns `None` for absent headers. The raw URI from `req.path` flows into detection code without escaping.
- **Test:** `test_parse_get_request`

### AC-010 (traces to BC-2.06.026 invariant 4)
No escape function is called at parse time — raw URI bytes from `req.path` flow directly into detection code and into `Finding.evidence` per ADR 0003 / INV-4.
- **Test:** `test_http_finding_c1_csi_escaped_by_terminal_reporter` (integration)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| parse_one_request | src/analyzer/http.rs:35-50 | effectful-shell (mutates HttpAnalyzer state) |
| try_parse_requests | src/analyzer/http.rs:359-438 | effectful-shell (pipelined loop) |
| try_parse_responses | src/analyzer/http.rs:440-497 | effectful-shell |
| find_header | src/analyzer/http.rs:70-75 | pure-core |
| HttpFlowState | src/analyzer/http.rs (struct) | effectful-shell (mutable per-flow state) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | HTTP/1.0 request (version byte == 0) | Parsed normally; Host checks exempt for missing Host |
| EC-002 | Request with both Host and User-Agent | Both values stored in respective maps |
| EC-003 | Request with no User-Agent | user_agent field is None; no UA map entry added |
| EC-004 | Request with no Host (HTTP/1.0) | host field is None; no hosts map entry; no finding |
| EC-005 | Two complete requests in one buffer | Loop parses both; each counted independently |
| EC-006 | Partial request (no \r\n\r\n) | Buffer retained; no stats updated |
| EC-007 | Response with httparse code==None | status_codes[0] incremented; transactions incremented |
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

1. [ ] Write failing tests for AC-001 through AC-010 (test-writer)
2. [ ] Verify Red Gate: all tests fail before implementation
3. [ ] Implement `parse_one_request` per BC-2.06.001 (extract method, URI, host, UA; drain buffer; reset error count; call detections)
4. [ ] Implement `try_parse_requests` pipelined loop per BC-2.06.002 (drain-and-retry; had_success flag)
5. [ ] Implement partial buffering behavior per BC-2.06.003 (Partial arm exits loop; buffer retained)
6. [ ] Implement `try_parse_responses` per BC-2.06.004 (transactions++; status_codes update; drain buffer)
7. [ ] Implement `find_header` per BC-2.06.026 (eq_ignore_ascii_case; from_utf8_lossy; trim)
8. [ ] Run all tests; verify all pass
9. [ ] Verify purity boundaries (find_header is pure; parse functions are effectful-shell)
10. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A — first story in E-4 | N/A | N/A | N/A |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Raw URI bytes must NOT be escaped at parse time | ADR 0003 / INV-4 | Code review: confirm no escape call in parse_one_request or find_header |
| `transactions` increments only on response parse, not request parse | BC-2.06.004 invariant 1 | Unit test: AC-008 |
| `MAX_HEADERS = 96` is the httparse capacity | BC-2.06.001 precondition 4 | Code review: confirm httparse config |
| `had_success` flag prevents body bytes from counting as parse errors | BC-2.06.002 invariant 2 / BC-2.06.020 | Unit test: AC-004 |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| httparse | (as pinned in Cargo.toml) | HTTP/1.x header parsing (Status::Complete / Partial / Err) |
| Rust std | 2024 edition (stable) | String::from_utf8_lossy, drain, BTreeMap |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/http.rs | modify | Core parsing functions: parse_one_request, try_parse_requests, try_parse_responses, find_header |
| tests/http_analyzer_tests.rs | modify | Add/extend tests for AC-001 through AC-010 |
