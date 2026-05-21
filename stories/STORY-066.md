---
document_type: story
story_id: STORY-066
epic_id: E-6
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-08/BC-2.08.001.md
  - .factory/specs/behavioral-contracts/ss-08/BC-2.08.002.md
  - .factory/specs/behavioral-contracts/ss-08/BC-2.08.003.md
  - .factory/specs/behavioral-contracts/ss-08/BC-2.08.004.md
input-hash: ""
traces_to: .factory/specs/prd.md
points: 5
depends_on: []
blocks: []
behavioral_contracts:
  - BC-2.08.001
  - BC-2.08.002
  - BC-2.08.003
  - BC-2.08.004
verification_properties: []
priority: P1
cycle: v1.0.0-brownfield
wave: 4
target_module: analyzer/dns
subsystems: [SS-08]
estimated_days: 2
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — full TDD Iron Law enforced.

> **Execute:** `/vsdd-factory:deliver-story STORY-066`

# STORY-066: DNS Traffic Statistics — Port-53 Dispatch, QR-Bit Counting, and Never-Emit Contract

## Narrative
- **As a** forensic analyst or SOC operator
- **I want** wirerust to count DNS queries and responses in any pcap with port-53 traffic (TCP or UDP), report those counts in the analysis summary, and guarantee that no DNS-based `Finding` is ever emitted
- **So that** the analysis summary provides a baseline DNS traffic picture for triage without false-positive findings from DNS anomaly detection that does not yet exist

## Acceptance Criteria

### AC-001 (traces to BC-2.08.001 postcondition 1)
`DnsAnalyzer::can_decode(packet)` returns `true` for any packet where `src_port == 53` OR `dst_port == 53`, regardless of whether transport is TCP or UDP.
- **Test:** `test_dns_can_decode_port_53_tcp_and_udp()`

### AC-002 (traces to BC-2.08.001 postcondition 2)
`can_decode` returns `false` for `TransportInfo::None` (ICMP / unknown transport).
- **Test:** `test_dns_can_decode_false_for_icmp()`

### AC-003 (traces to BC-2.08.001 postcondition 3)
`can_decode` returns `false` when neither port is 53 (e.g., UDP src=54, dst=54).
- **Test:** `test_dns_can_decode_false_for_non_dns_port()`

### AC-004 (traces to BC-2.08.002 postcondition 1)
When `payload.len() >= 12` and `(payload[2] & 0x80) == 0` (QR=0), `analyze` increments `query_count` by 1.
- **Test:** `test_dns_analyzer_counts_queries()`

### AC-005 (traces to BC-2.08.002 postcondition 2)
When `payload.len() >= 12` and `(payload[2] & 0x80) != 0` (QR=1), `analyze` increments `response_count` by 1.
- **Test:** `test_dns_analyzer_counts_responses()`

### AC-006 (traces to BC-2.08.002 postcondition 3)
When `payload.len() < 12` (truncated DNS header), `is_query` returns `false` and `response_count` is incremented (not `query_count`).
- **Test:** `test_dns_short_payload_counted_as_response()`

### AC-007 (traces to BC-2.08.002 invariant 1)
Exactly one counter (either `query_count` or `response_count`) is incremented per call to `analyze`.
- **Test:** `test_dns_analyze_increments_exactly_one_counter()`

### AC-008 (traces to BC-2.08.003 postcondition 1)
`DnsAnalyzer::summarize()` returns an `AnalysisSummary` with `analyzer_name == "DNS"`.
- **Test:** `test_dns_summarize_analyzer_name()`

### AC-009 (traces to BC-2.08.003 postcondition 5)
`summarize().packets_analyzed == query_count + response_count`.
- **Test:** `test_dns_summarize_packets_analyzed_is_sum()`

### AC-010 (traces to BC-2.08.003 postcondition 2)
`summarize().detail["dns_queries"]` equals `query_count` as a JSON number.
- **Test:** `test_dns_summarize_detail_keys()`

### AC-011 (traces to BC-2.08.003 postcondition 3)
`summarize().detail["dns_responses"]` equals `response_count` as a JSON number.
- **Test:** `test_dns_summarize_detail_keys()` (same test; asserts both keys)

### AC-012 (traces to BC-2.08.003 postcondition 4)
The detail `BTreeMap` contains exactly two keys: `"dns_queries"` and `"dns_responses"` (no other keys).
- **Test:** `test_dns_summarize_exactly_two_detail_keys()`

### AC-013 (traces to BC-2.08.004 postcondition 1)
`DnsAnalyzer::analyze(packet)` always returns an empty `Vec<Finding>` — no DNS condition causes a Finding to be emitted.
- **Test:** `test_dns_analyze_always_returns_empty_findings()`

### AC-014 (traces to BC-2.08.004 invariant 1)
Even with a very high DNS query volume (e.g., 1000 packets), `analyze` returns `vec![]` for each call.
- **Test:** `test_dns_high_volume_no_findings()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `DnsAnalyzer::can_decode` | `src/analyzer/dns.rs:52-60` | pure-core |
| `DnsAnalyzer::analyze` | `src/analyzer/dns.rs:62-70` | mixed (pure logic; counter mutation trivial) |
| `DnsAnalyzer::summarize` | `src/analyzer/dns.rs:72-88` | pure-core (snapshot) |
| `is_dns_port` helper | `src/analyzer/dns.rs:34-35` | pure-core |
| `is_query` helper | `src/analyzer/dns.rs:38-44` | pure-core |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | UDP src=53, dst=12345 (DNS response to client) | `can_decode = true` |
| EC-002 | TCP dst=53 (DNS-over-TCP) | `can_decode = true` |
| EC-003 | `payload.len() == 0` | `is_query = false`; `response_count++` |
| EC-004 | `payload[2] == 0xFF` (QR=1 + other flags set) | `response_count++` (QR bit is set) |
| EC-005 | No packets analyzed (zero calls to analyze) | `summarize` returns `packets_analyzed=0`, `dns_queries=0`, `dns_responses=0` |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `src/analyzer/dns.rs` | mixed (can_decode/summarize are pure-core; analyze mutates counters) | Counter mutation is the only side effect; no I/O |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,800 |
| `src/analyzer/dns.rs` | ~3,000 |
| `tests/dns_tests.rs` (existing or new) | ~2,500 |
| BC files (4 BCs) | ~4,500 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~13,800** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~7%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-014 (test-writer)
2. [ ] Verify Red Gate: all tests fail
3. [ ] Implement `is_dns_port(src, dst)` helper: `src == 53 || dst == 53`
4. [ ] Implement `can_decode` dispatching on `TransportInfo` variants with port check
5. [ ] Implement `is_query`: length guard (< 12 bytes returns false), then QR-bit test `(payload[2] & 0x80) == 0`
6. [ ] Implement `analyze`: call `is_query`; increment `query_count` if true, `response_count` if false; return `vec![]`
7. [ ] Implement `summarize`: return `AnalysisSummary` with `analyzer_name = "DNS"`, `packets_analyzed = query_count + response_count`, `detail = BTreeMap { "dns_queries": query_count, "dns_responses": response_count }`
8. [ ] Write edge-case tests for EC-001 through EC-005
9. [ ] Verify `analyze` NEVER returns a non-empty `Vec<Finding>`
10. [ ] Run `cargo test --all-targets` and `cargo clippy -- -D warnings`

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A — first story in E-6 | — | — | — |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `analyze` unconditionally returns `vec![]` | BC-2.08.004 invariant 1 | Test: `assert!(findings.is_empty())` for all input types |
| `is_query` returns `false` for payloads < 12 bytes | BC-2.08.002 invariant 2 | Test: short payload goes to `response_count` not `query_count` |
| `detail` BTreeMap uses BTreeMap (not HashMap) for deterministic key order | BC-2.08.003 invariant 1 | Type annotation: `BTreeMap<String, serde_json::Value>` |
| `summarize` does NOT reset counters | BC-2.08.003 invariant 4 | Test: call `summarize` twice; assert same values |
| `can_decode` is port-based only; no payload inspection | BC-2.08.001 invariant 3 | Code review: no `payload` access in `can_decode` |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `serde_json` | workspace version | `serde_json::Value::Number` for detail map values |
| `std::collections::BTreeMap` | stdlib | Deterministic key order in `AnalysisSummary.detail` |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/analyzer/dns.rs` | modify | `DnsAnalyzer` struct, `can_decode`, `analyze`, `summarize`, `is_dns_port`, `is_query` |
| `tests/dns_tests.rs` | create or modify | All AC test functions and edge-case tests |
