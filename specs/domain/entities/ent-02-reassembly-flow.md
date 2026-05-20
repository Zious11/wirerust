---
artifact: L2-ent-02
traces_to: ../domain-spec.md
title: Entities -- Reassembly and Flow (L2)
status: descriptive (brownfield) -- reconciled against develop HEAD aa2ece9
reconciled: 2026-05-20
---

# Entities: Reassembly and Flow (L2)

Covers E-9 through E-15, E-18 through E-20. These are the L2 Stream/Routing entities
(C-6..C-9). Source: pass-2-domain-model.md + pass-2-R3.md.

## E-9: FlowKey (src/reassembly/flow.rs:6-12)

Canonicalized 4-tuple identifying a TCP connection.

```
struct FlowKey {
    lower_ip:   IpAddr,   // private
    lower_port: u16,      // private
    upper_ip:   IpAddr,   // private
    upper_port: u16,      // private
}
```

Derives: `Debug, Clone, PartialEq, Eq, Hash`.

Construction invariant (VO-1 / INV-1): `FlowKey::new(ip_a, port_a, ip_b, port_b)` always
stores the pair with the SMALLER (ip, port) tuple as (lower_ip, lower_port). The comparison
is `(ip_a, port_a) <= (ip_b, port_b)` -- tuple comparison, NOT independent field comparison.
This means A->B and B->A produce the identical key. Sorting ip and port independently would
silently merge unrelated connections sharing an IP.

## E-10: FlowState (src/reassembly/flow.rs:62-69)

```
enum FlowState { New, SynSent, Established, Closing, Closed }
```

TCP handshake/teardown state machine. Transitions (VO-11):
- `New -> SynSent` on SYN.
- `SynSent -> Established` on SYN-ACK.
- `Established -> Closing` on first FIN.
- `Closing -> Closed` on second FIN.
- Any state -> `Closed` on RST (direct jump).
- `New -> Established` via `on_data_without_syn` (mid-stream capture; sets `partial=true`).

## E-11: FlowDirection (src/reassembly/flow.rs:71-87)

Per-direction reassembly buffer and anomaly counters.

```
struct FlowDirection {
    isn:                      Option<u32>,             // Initial sequence number
    base_offset:              u64,                     // first expected data byte offset
    segments:                 BTreeMap<u64, Vec<u8>>,  // private to super
    buffered_bytes:           usize,                   // private to super
    reassembled_bytes:        usize,
    overlap_count:            u32,
    overlap_alert_fired:      bool,   // one-shot latch
    small_segment_run_count:  u32,    // consecutive-run counter (not cumulative; resets on normal segment)
    small_segment_alert_fired:bool,   // one-shot latch
    out_of_window_count:      u32,
    out_of_window_alert_fired:bool,   // one-shot latch
    fin_seen:                 bool,
    rst_seen:                 bool,
    depth_exceeded:           bool,
}
```

The three `_alert_fired` latches are monotonic false->true. Alerts fire exactly once per
direction (worst-case 6 findings per bidirectional flow across all three types).

`small_segment_run_count` tracks consecutive small-segment runs, not cumulative count. It
resets to 0 whenever a segment with payload >= `small_segment_max_bytes` is inserted. This
is the redesigned consecutive-run model from #92/#93.

## E-12: TcpFlow (src/reassembly/flow.rs:159-170)

Aggregate of two FlowDirections plus handshake state.

```
struct TcpFlow {
    key:              FlowKey,
    client_to_server: FlowDirection,
    server_to_client: FlowDirection,
    state:            FlowState,
    partial:          bool,       // true if mid-stream capture (no SYN observed)
    first_seen:       u32,        // pcap timestamp (u32)
    last_seen:        u32,        // updated on each process_packet call
    initiator:        Option<(IpAddr, u16)>,  // private; first-set-wins
    fin_count:        u8,         // private; saturating_add
}
```

`initiator` is latched: first `set_initiator` call wins; subsequent calls are no-ops.
If never set, `TcpFlow::direction()` defaults to `ServerToClient` (VO-10).

## E-13: InsertResult (src/reassembly/segment.rs:7-18)

```
enum InsertResult {
    Inserted, Duplicate, PartialOverlap, ConflictingOverlap,
    Truncated, DepthExceeded, SegmentLimitReached, OutOfWindow, IsnMissing
}
```

The engine matches all 9 variants at mod.rs:232-265. `IsnMissing` is a programming-error
sentinel (VO-12): triggers one-shot eprintln via `ISN_MISSING_WARNED` atomic; segment is
silently dropped.

## E-14: Direction (src/reassembly/handler.rs:5-9)

```
enum Direction { ClientToServer, ServerToClient }
```

Binary enum (VO-10). No Unknown variant; the engine commits to a direction at first SYN or
via `on_data_without_syn`.

## E-15: CloseReason (src/reassembly/handler.rs:11-17)

```
enum CloseReason { Fin, Rst, Timeout, MemoryPressure }
```

Passed to `StreamHandler::on_flow_close`. Both HTTP and TLS analyzers currently ignore the
reason (`_reason` binding) and simply remove flow state.

## E-18: ReassemblyConfig (src/reassembly/config.rs)

All defaults from `ReassemblyConfig::default()`. Derives `Debug, Clone`. Validated on
construction: all caps must be > 0. All five anomaly threshold fields are CLI-overridable
(P2.05 / #88 + #93 + #96):

| Field | Default | CLI flag | Purpose |
|---|---|---|---|
| max_depth | 10 MB | --reassembly-depth | Max reassembled bytes per flow direction |
| memcap | 1 GB | --reassembly-memcap | Total buffered bytes before eviction |
| flow_timeout_secs | 300 s | (none) | Idle flow expiry |
| max_flows | 100,000 | (none) | Flow table size cap before LRU eviction |
| max_segments_per_direction | 10,000 | (none) | BTreeMap size guard per direction |
| max_receive_window | 1 MB | (none) | Out-of-window detection threshold |
| overlap_alert_threshold | 50 | --overlap-threshold (0-255) | Per-direction overlap anomaly threshold |
| small_segment_alert_threshold | 100 | --small-segment-threshold (0-2048) | Consecutive-run length before alert |
| small_segment_max_bytes | 16 | --small-segment-max-bytes (0-2048) | Payload bytes below which a segment is "small" |
| small_segment_ignore_ports | [23, 513] | --small-segment-ignore-ports | Ports exempt from small-segment detection |
| out_of_window_alert_threshold | 100 | --out-of-window-threshold (0-2048) | Per-direction out-of-window threshold |

Range validation on the five threshold CLI flags is enforced at parse time (#96).

## E-19: ReassemblyStats (src/reassembly/stats.rs)

17-field counter struct (all u64, all zero-initialized via `Default`):
`packets_processed`, `packets_tcp`, `packets_skipped_non_tcp`, `flows_total`,
`flows_partial`, `flows_expired`, `flows_rst`, `flows_fin`, `segments_inserted`,
`segments_duplicates`, `segments_overlaps`, `segments_out_of_window`,
`segments_segment_limit`, `segments_depth_exceeded`, `bytes_reassembled`, `evictions`,
`dropped_findings`.

`dropped_findings: u64` was added by P1.01 (#73). It is incremented each time a finding is
suppressed due to `MAX_FINDINGS` cap, and surfaced via `summarize().detail["dropped_findings"]`.
This closes the prior observability gap (NFR-RES-022).

## E-20: TcpReassembler (src/reassembly/mod.rs)

Top-level engine. Not Serialize, not Clone. Source now split across four files:
`reassembly/mod.rs` (~691 LOC), `config.rs`, `stats.rs`, `lifecycle.rs` (P2.01 / #85).

```
struct TcpReassembler {
    config:       ReassemblyConfig,    // private
    flows:        HashMap<FlowKey, TcpFlow>, // private
    stats:        ReassemblyStats,     // private
    findings:     Vec<Finding>,        // private; capped at MAX_FINDINGS=10,000
    total_memory: usize,               // private
    finalized:    bool,                // private; latch for finalize()
    classification_attempts: HashMap<FlowKey, u32>,  // (carried by StreamDispatcher, not here)
}
```

`impl Drop for TcpReassembler` was added by P0.03 (#72). If the reassembler is dropped
without `finalize()` having been called, Drop emits a one-shot `eprintln!` naming how many
flows and bytes were discarded. The `run_analyze` IIFE pattern in main.rs guarantees
`finalize()` is reached before any `Err` escapes the function body.
