---
document_type: story
story_id: STORY-090
epic_id: E-9
version: "1.2"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.018.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.019.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.020.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.021.md
input-hash: "0ee093d"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-086, STORY-088, STORY-089]
blocks: []
behavioral_contracts:
  - BC-2.12.018
  - BC-2.12.019
  - BC-2.12.020
  - BC-2.12.021
verification_properties: []
priority: P0
cycle: v0.1.0-greenfield-spec
wave: 27
target_module: summary
subsystems: [SS-12]
estimated_days: 2
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — full TDD Iron Law enforced.

> **Execute:** `/vsdd-factory:deliver-story STORY-090`

# STORY-090: Summary Data Model — ingest, Service Hints, unique_hosts, Serialization

## Narrative
- **As a** forensic analyst
- **I want** the `Summary` struct to accurately accumulate per-packet statistics (packet count, bytes, unique hosts, protocol distribution, port-based service hints) and expose them via sorted accessors and serde serialization
- **So that** any reporter can produce a complete and accurate summary block without reimplementing the accumulation logic

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.12.018 | Summary::ingest Increments total_packets, total_bytes, hosts, protocols |
| BC-2.12.019 | Summary::ingest Derives Service Name from app_protocol_hint |
| BC-2.12.020 | Summary::unique_hosts Returns Sorted Deduplicated Vec<IpAddr> |
| BC-2.12.021 | Summary Serializes with total_packets/total_bytes/skipped_packets Fields |

## Acceptance Criteria

### AC-001 (traces to BC-2.12.018 postcondition 1)
`Summary::ingest` increments `total_packets` by 1 on each call.
- **Test:** `test_summary_ingest_increments_total_packets()`

### AC-002 (traces to BC-2.12.018 postcondition 2)
`Summary::ingest` increments `total_bytes` by `packet.packet_len as u64` on each call.
- **Test:** `test_summary_ingest_increments_total_bytes()`

### AC-003 (traces to BC-2.12.018 postcondition 3)
Both `packet.src_ip` and `packet.dst_ip` are inserted into the `hosts: HashSet<IpAddr>` on each call; repeated addresses are deduplicated.
- **Test:** `test_BC_2_12_018_host_counting_src_and_dst()`

### AC-004 (traces to BC-2.12.018 postcondition 4)
`protocols[packet.protocol]` is incremented by 1 on each call; the entry is created if absent.
- **Test:** `test_BC_2_12_018_protocol_breakdown()`

### AC-005 (traces to BC-2.12.018 invariant 2)
`skipped_packets` is NOT set by `ingest`; it is set by the caller after the packet loop.
- **Test:** `test_skipped_packets_not_modified_by_ingest()`

### AC-006 (traces to BC-2.12.019 postcondition 1)
When `packet.app_protocol_hint()` returns `Some("HTTP")`, `services["HTTP"]` is incremented.
- **Test:** `test_summary_service_detection_http()`

### AC-007 (traces to BC-2.12.019 postcondition 2)
When `packet.app_protocol_hint()` returns `None` (non-standard port), the `services` map is unchanged.
- **Test:** `test_summary_service_detection_none_on_unknown_port()`

### AC-008 (traces to BC-2.12.019 invariant 2)
Service attribution is port-based (via `app_protocol_hint`); it does NOT consult the stream dispatcher.
- **Test:** `test_summary_service_is_port_based_not_content_based()`

### AC-009 (traces to BC-2.12.020 postcondition 1)
`Summary::unique_hosts()` returns a sorted `Vec<IpAddr>` containing exactly the deduplicated set of all src and dst IPs seen across all `ingest` calls.
- **Test:** `test_unique_hosts_sorted_and_deduplicated()`

### AC-010 (traces to BC-2.12.020 postcondition 3)
An empty `Summary` (zero calls to `ingest`) returns an empty `Vec` from `unique_hosts()`.
- **Test:** `test_unique_hosts_empty_when_no_packets()`

### AC-011 (traces to BC-2.12.020 invariant 4)
`unique_hosts()` takes `&self` and does NOT mutate the Summary.
- **Test:** `test_unique_hosts_is_non_mutating()`

### AC-012 (traces to BC-2.12.021 postcondition 1)
A `Summary` serialized via `JsonReporter`'s `serde_json::json!` block produces JSON with `"total_packets"`, `"total_bytes"`, `"skipped_packets"` as u64 integers.
- **Test:** `test_json_reporter_includes_skipped_packets()`

### AC-013 (traces to BC-2.12.021 postcondition 7)
Protocol keys in the JSON `"protocols"` object use Debug format (e.g., `"Tcp"`, `"Udp"`, `"Icmp"`).
- **Test:** `test_summary_protocol_keys_use_debug_format()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `Summary::ingest` | `src/summary.rs:58-68` | pure-core (in-memory mutation) |
| `Summary::unique_hosts` | `src/summary.rs:70-74` | pure-core |
| `Summary` serde integration | `src/summary.rs:18-19`; `src/reporter/json.rs:47-57` | pure-core |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Packet where `src_ip == dst_ip` | Host appears once in `unique_hosts()` (HashSet dedup) |
| EC-002 | Mix of IPv4 and IPv6 addresses | Both in `unique_hosts()`; IPv4 sorts before IPv6 |
| EC-003 | Two packets with the same protocol | `protocols[protocol] = 2` |
| EC-004 | `packet_len = 0` | `total_bytes` unchanged (incremented by 0) |
| EC-005 | `services["HTTP"] = 2` | JSON: `"services": {"HTTP": 2}` |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `src/summary.rs` | pure-core | All operations are in-memory; no I/O, no global state |
| `src/reporter/json.rs` (summary block) | pure-core | Pure serde serialization |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,800 |
| `src/summary.rs` | ~3,000 |
| `src/reporter/json.rs` (summary section) | ~2,000 |
| `tests/summary_story_090_tests.rs` | ~3,000 |
| BC files (4 BCs) | ~5,500 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~17,300** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~9%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-013 (test-writer)
2. [ ] Verify Red Gate: all tests fail
3. [ ] Implement `Summary` struct with `total_packets`, `total_bytes`, `skipped_packets`, `hosts`, `protocols`, `services` fields
4. [ ] Implement `Summary::ingest` incrementing all counters
5. [ ] Implement `Summary::unique_hosts()` sorted dedup accessor
6. [ ] Implement `protocol_counts()` and `service_counts()` accessors for reporters
7. [ ] Add `#[derive(Serialize)]` to `Summary`
8. [ ] Verify JSON reporter builds summary JSON via `serde_json::json!` using accessors (not direct struct serialization of private fields)
9. [ ] Write edge-case tests for EC-001 through EC-005
10. [ ] Run `cargo test --all-targets` and `cargo clippy -- -D warnings`

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-086 | `Cli` parsing established | — | — |
| STORY-088 | `run_analyze` drives `summary.ingest(&parsed)` per packet | `skipped_packets` is set by caller after loop, NOT by `ingest` | Confusion between `ingest` (increments counts) and `skipped_packets` assignment (done at main.rs:183) |
| STORY-089 | `skipped_packets = total_decode_errors` assignment site known | Reporter receives Summary by reference | Protocol keys use `{k:?}` Debug format in JSON (not `Display`) |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `skipped_packets` is NOT set by `ingest` | BC-2.12.018 invariant 2 | Test: call `ingest` N times; assert `skipped_packets == 0` |
| `unique_hosts()` takes `&self` (non-mutating) | BC-2.12.020 invariant 4 | Compiler enforces via signature |
| Protocol keys use `{k:?}` Debug format | BC-2.12.021 invariant 2 | Test: assert JSON key is `"Tcp"` not `"TCP"` |
| Private fields (`hosts`, `protocols`, `services`) are NOT serialized directly via derive | BC-2.12.021 invariant 1 | Code review: serde_json::json! block in json.rs uses accessor methods |
| `app_protocol_hint` is port-based; does NOT call dispatcher | BC-2.12.019 invariant 2 | No dispatcher import in `summary.rs` |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `serde` | workspace version | `#[derive(Serialize)]` on `Summary` |
| `serde_json` | workspace version | `serde_json::json!` for summary JSON construction in `JsonReporter` |
| `std::collections::HashSet` | stdlib | Deduplication of host IPs |
| `std::collections::HashMap` | stdlib | Protocol and service counters |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/summary.rs` | modify | `Summary` struct, `ingest`, `unique_hosts`, accessors, `#[derive(Serialize)]` |
| `src/reporter/json.rs` | modify | Summary JSON block using `serde_json::json!` with accessor methods |
| `tests/summary_story_090_tests.rs` | modify | All AC test functions and edge-case tests |
