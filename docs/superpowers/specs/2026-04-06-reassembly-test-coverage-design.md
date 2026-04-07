# Reassembly Engine Test Coverage Design

**Issue:** #13 — test: add missing reassembly engine test coverage
**Scope:** 7 new integration tests + `ack` parameter on `make_tcp_packet` helper. No production code changes.

## Problem

Test gaps identified during PR #10 review. The reassembly engine has untested paths:

1. **SYN+ACK / bidirectional data** — `on_syn_ack()` (mod.rs:151) never exercised. No test sends data from both directions or verifies `Direction` assignment.
2. **max_flows eviction** — `evict_flows()` (mod.rs:429) never tested. `CloseReason::MemoryPressure` never asserted.
3. **memcap eviction** — memcap threshold check (mod.rs:341) never tested.
4. **FIN teardown (full lifecycle)** — Existing `test_fin_close_total_memory` focuses on memory tracking. No test covers full 3-way handshake + bidirectional data + FIN teardown with `stats.flows_fin` verification.
5. **Overlap anomaly finding** — `OVERLAP_ALERT_THRESHOLD=50` (mod.rs:14) and `findings()` never tested.
6. **Conflicting overlap finding** — `generate_conflicting_overlap_finding()` (mod.rs:474) never tested.
7. **max_segments_per_direction** — Segment count limit (segment.rs:40) never tested at engine level.

## Approach

Add 7 tests to `tests/reassembly_engine_tests.rs`. Add `ack: bool` parameter to the shared `make_tcp_packet` helper (update all existing call sites).

## Changes

### Helper: `make_tcp_packet` — Add `ack` Parameter

Add `ack: bool` after `syn` in the parameter list. Pass it through to `TransportInfo::Tcp { ack }`. Update all ~22 existing call sites to pass `false`.

### Test 1: `test_syn_ack_bidirectional_data`

Packet sequence:
1. Client SYN (seq 1000, `syn=true`)
2. Server SYN+ACK (seq 2000, `syn=true, ack=true`)
3. Client data "request" (seq 1001)
4. Server data "response" (seq 2001)

Assertions:
- `stats.flows_partial == 0` (proper handshake, not mid-stream)
- `stats.flows_total == 1`
- 2 data events: first with `Direction::ClientToServer`, second with `Direction::ServerToClient`
- Data content matches "request" and "response"

### Test 2: `test_max_flows_eviction`

Config: `max_flows=2`, small `flow_timeout_secs`.

Packet sequence:
1. Flow A: SYN + data "aaa" (client 10.0.0.1:1000 → server 10.0.0.2:80, timestamp=1)
2. Flow B: SYN + data "bbb" (client 10.0.0.1:2000 → server 10.0.0.2:80, timestamp=2)
3. Flow C: SYN (client 10.0.0.1:3000 → server 10.0.0.2:80, timestamp=3) — triggers eviction

Assertions:
- `stats.evictions >= 1`
- Close event with `CloseReason::MemoryPressure` present
- Data from evicted flow was flushed (delivered via `on_data`) before close
- Flow C successfully created (flows table not at capacity after eviction)

### Test 3: `test_memcap_eviction`

Config: `memcap` set to a small value (e.g., 10 bytes).

Packet sequence:
1. Flow A: SYN + out-of-order data (stays buffered, 5 bytes)
2. Flow A: more out-of-order data (stays buffered, 5 bytes) — at memcap
3. Flow A: another out-of-order segment (3 bytes) — exceeds memcap, triggers eviction after processing

Assertions:
- `CloseReason::MemoryPressure` or memcap enforcement observed
- `total_memory()` returns to within memcap bounds

Note: memcap eviction fires at mod.rs:341 after payload processing. Since there's only one flow, it will evict itself. Alternative: use two flows to make the eviction target clearer.

### Test 4: `test_full_handshake_fin_teardown`

Packet sequence:
1. Client SYN (seq 1000)
2. Server SYN+ACK (seq 2000)
3. Client data "hello" (seq 1001)
4. Server data "world" (seq 2001)
5. Client FIN (seq 1006)
6. Server FIN (seq 2006)

Assertions:
- `stats.flows_fin == 1`
- `CloseReason::Fin` in close events
- Client data "hello" delivered as `ClientToServer`
- Server data "world" delivered as `ServerToClient`
- `total_memory() == 0`

### Test 5: `test_overlap_anomaly_finding`

Config: default (threshold is 50).

Packet sequence:
1. SYN (seq 1000)
2. Data "AAAA" at seq 1001 — original segment
3. 51 duplicate sends of "AAAA" at seq 1001 — each increments `overlap_count`

After 52 total packets (1 SYN + 1 original + 50 duplicates gives overlap_count=50, then 1 more = 51 > threshold).

Assertions:
- `findings().len() >= 1`
- Finding contains "Excessive segment overlaps"
- Finding has `ThreatCategory::Anomaly`

### Test 6: `test_conflicting_overlap_finding`

Packet sequence:
1. SYN (seq 1000)
2. Data "AAAA" at seq 1001
3. Data "BBBB" at seq 1001 — same offset, different data

Assertions:
- `findings().len() >= 1`
- Finding contains "Conflicting TCP segment overlap"
- Finding has `Confidence::High`

### Test 7: `test_max_segments_per_direction`

Config: `max_segments_per_direction=5`.

Packet sequence:
1. SYN (seq 1000)
2. 5 non-contiguous segments: seq 1002 ("a"), 1004 ("b"), 1006 ("c"), 1008 ("d"), 1010 ("e") — each 1 byte with gaps, all buffered
3. 6th segment: seq 1012 ("f") — should be rejected (DepthExceeded)

Assertions:
- `stats.segments_inserted == 5` (6th rejected)
- Fill the gap: send seq 1001 ("X") — triggers flush of offset 1 + offset 2 ("a")
- Verify existing buffered segments are intact and delivered on flush

## Files Modified

| File | Change |
|------|--------|
| `tests/reassembly_engine_tests.rs` | Add `ack` to helper, update call sites, add 7 tests |

## Not In Scope

- Production code changes
- Segment-level tests (already comprehensive)
- Benchmark tests
- Small-segment anomaly threshold testing (similar pattern to overlap, lower priority)
