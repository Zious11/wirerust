# TCP Stream Reassembly — Design Spec

## Goal

Add forensic-grade TCP stream reassembly to wirerust so that TCP-based protocol analyzers (HTTP, TLS, SMB, etc.) can operate on complete, ordered byte streams instead of individual packet payloads.

## Why

Without reassembly, TCP-based analyzers only see whatever fits in a single packet. HTTP requests span multiple segments, TLS ClientHellos can be fragmented, and any protocol over TCP is unreliable to parse packet-by-packet. pcapper (Python/Scapy) has basic reassembly via `TCPSession` but handles retransmissions and overlapping segments poorly. wirerust's reassembly is a correctness advantage over pcapper, not just a speed advantage.

## Architecture

The reassembly module sits between the decoder and stream-based analyzers:

```
Reader → Decoder → TcpReassembler ──→ StreamAnalyzers (HTTP, TLS, SMB...)
                        │
                        └──→ per-packet Analyzers (DNS, port scan...)
```

Every decoded packet goes through both paths. The reassembler tracks TCP flows and delivers contiguous byte streams to stream analyzers via callbacks. Per-packet analyzers (DNS, etc.) continue to receive individual packets unchanged.

## Core Data Model

### FlowKey

Identifies a TCP connection. Canonicalized so both directions map to the same key.

```rust
pub struct FlowKey {
    pub lower_ip: IpAddr,
    pub lower_port: u16,
    pub upper_ip: IpAddr,
    pub upper_port: u16,
}
```

Canonicalization: compare `(ip, port)` tuples lexicographically; the smaller one is `lower`. This means `A→B` and `B→A` produce the same `FlowKey`.

### FlowDirection

One side of a TCP connection. Each flow has two: client→server and server→client.

```rust
pub struct FlowDirection {
    pub isn: Option<u32>,
    pub base_offset: u64,
    pub segments: BTreeMap<u64, Vec<u8>>,
    pub reassembled_bytes: usize,
    pub overlap_count: u32,
    pub small_segment_count: u32,
    pub fin_seen: bool,
    pub rst_seen: bool,
    pub depth_exceeded: bool,
}
```

- `isn`: Initial Sequence Number. Set from SYN or inferred from first data packet.
- `base_offset`: The next contiguous byte expected, ISN-relative. Starts at 1 (ISN+1 is the first data byte after SYN). Uses `u64` to handle streams >4GB without key-space wraparound.
- `segments`: Out-of-order buffer. Keyed by ISN-relative offset as `u64` (`(seq.wrapping_sub(isn)) as u64`). BTreeMap provides ordered iteration for flush.
- `reassembled_bytes`: Total bytes flushed so far. Used to enforce depth limit.
- `overlap_count`: Number of overlapping segments seen. If >50, generate an evasion-attempt Finding.
- `small_segment_count`: Consecutive segments <8 bytes. If >2048, generate an evasion-attempt Finding.
- `fin_seen`, `rst_seen`: Terminal flag tracking.
- `depth_exceeded`: Set when `reassembled_bytes` exceeds the per-direction limit.

### TcpFlow

A complete TCP connection.

```rust
pub struct TcpFlow {
    pub key: FlowKey,
    pub client_to_server: FlowDirection,
    pub server_to_client: FlowDirection,
    pub state: FlowState,
    pub partial: bool,
    pub first_seen: u32,
    pub last_seen: u32,
}
```

- `state`: `New`, `SynSent`, `Established`, `Closing`, `Closed`, `TimedOut`.
- `partial`: `true` if the flow was picked up mid-stream (no SYN observed). Forensic reports include this flag.

### FlowState Transitions

```
New → SynSent         (SYN seen)
SynSent → Established (SYN+ACK seen, or data seen)
New → Established     (data without SYN — mid-stream pickup, sets partial=true)
Established → Closing (FIN seen on either direction)
Closing → Closed      (FIN seen on both directions, or timeout)
Any → Closed          (RST seen)
Any → TimedOut        (flow_timeout_secs exceeded)
```

## Sequence Number Handling

All segment keys are stored as ISN-relative offsets cast to `u64`: `(seq.wrapping_sub(isn)) as u64`. The wrapping subtraction handles the 32-bit sequence space correctly, and promoting to `u64` ensures BTreeMap key ordering works for streams of any size (including >4GB transfers that wrap the sequence space).

Comparison helpers for raw seq numbers use wrapping arithmetic:

```rust
fn seq_before(a: u32, b: u32) -> bool {
    // a is "before" b in TCP sequence space (signed comparison of difference)
    (a.wrapping_sub(b) as i32) < 0
}

fn seq_offset(seq: u32, isn: u32) -> u64 {
    seq.wrapping_sub(isn) as u64
}
```

## Overlap Handling

**Policy: first-wins (hardcoded).** When a new segment overlaps existing data, the existing bytes are kept. This matches the behavior of Windows, macOS, and BSD — the majority of real-world targets. It also matches what Zeek and NetworkMiner do.

Implementation: on segment insertion, check BTreeMap neighbors. If the new segment's range overlaps any existing segment, trim the new one to only cover gaps. If fully covered, discard it (retransmission dedup).

**Anomaly detection on overlaps:**
- Increment `overlap_count` on every overlap.
- If `overlap_count > 50` on a flow direction, generate a Finding: `[Anomaly] LIKELY (MEDIUM) — Excessive TCP segment overlaps on flow {key} ({count} overlaps), possible evasion attempt. MITRE: T1036.`
- If overlapping data differs from existing data (not a simple retransmit), generate: `[Anomaly] LIKELY (HIGH) — Conflicting data in overlapping TCP segments on flow {key}, possible insertion/evasion attack.`

**Depth truncation mid-segment:** When inserting a segment that would exceed `max_depth_per_direction`, truncate it to `depth - reassembled_bytes` rather than dropping entirely. This captures as much as possible before cutting off.

**Small segment flood detection:** Track consecutive segments <8 bytes in `small_segment_count`. If >2048, generate: `[Anomaly] INCONCLUSIVE (MEDIUM) — Excessive small TCP segments on flow {key} ({count} segments <8 bytes), possible IDS evasion.`

## Mid-Stream Pickup

If data arrives for a flow with no SYN observed:
1. Set `isn` to the first segment's sequence number minus 1.
2. Set state to `Established`.
3. Set `partial = true`.
4. Reassembly proceeds normally from there.

This handles common scenarios: pcap capture started after connection was established, asymmetric SPAN port configs, or SYN packets dropped.

## Reassembly Engine

### TcpReassembler

```rust
pub struct TcpReassembler {
    flows: HashMap<FlowKey, TcpFlow>,
    config: ReassemblyConfig,
    total_memory_used: usize,
}

pub struct ReassemblyConfig {
    pub max_depth_per_direction: usize,  // default: 10MB (10_485_760)
    pub global_memcap: usize,            // default: 1GB (1_073_741_824)
    pub flow_timeout_secs: u32,          // default: 300
}
```

### Public API

```rust
impl TcpReassembler {
    pub fn new(config: ReassemblyConfig) -> Self;

    /// Process a decoded packet. Calls handler callbacks when new
    /// contiguous data becomes available. Uses pcap timestamp for
    /// timeout tracking (not wall clock).
    pub fn process_packet(
        &mut self,
        packet: &ParsedPacket,
        timestamp: u32,
        handler: &mut dyn StreamHandler,
    );

    /// Expire flows older than flow_timeout_secs.
    /// Evicts non-established first, then LRU.
    pub fn expire_flows(
        &mut self,
        current_time: u32,
        handler: &mut dyn StreamHandler,
    );

    /// Expire all remaining flows. Call at end of pcap.
    pub fn finalize(&mut self, handler: &mut dyn StreamHandler);

    pub fn stats(&self) -> ReassemblyStats;
}
```

### Processing Flow (per packet)

1. Skip non-TCP packets.
2. Extract FlowKey from ParsedPacket (canonicalize).
3. Look up or create TcpFlow in HashMap.
4. Determine direction (client→server or server→client) by comparing src against flow initiator.
5. Handle TCP flags:
   - SYN: record ISN, update state.
   - SYN+ACK: record server ISN, transition to Established.
   - FIN: mark `fin_seen` on that direction, transition state.
   - RST: mark `rst_seen`, transition to Closed, call `handler.on_flow_close()`.
6. If payload present and depth not exceeded and memcap not exceeded:
   - Compute ISN-relative offset.
   - Check for overlaps with existing segments (first-wins: trim new).
   - Insert into BTreeMap.
   - Flush: iterate from `base_offset`, move contiguous segments out, call `handler.on_data()` with the new bytes.
   - Advance `base_offset`, increment `reassembled_bytes`.
   - Update `total_memory_used`.
7. Update `last_seen` with pcap timestamp.

### Contiguous Flush

After inserting a segment, scan the BTreeMap starting from `base_offset`:

```
While BTreeMap contains a segment at base_offset:
    Remove it from BTreeMap
    Call handler.on_data(flow_key, direction, &data, base_offset)
    base_offset += data.len()
    reassembled_bytes += data.len()
```

If `reassembled_bytes` exceeds `max_depth_per_direction`, set `depth_exceeded = true` and generate a Finding.

## StreamHandler Trait

Analyzers that need reassembled streams implement this:

```rust
pub enum Direction {
    ClientToServer,
    ServerToClient,
}

pub enum CloseReason {
    Fin,
    Rst,
    Timeout,
    MemoryPressure,
}

pub trait StreamHandler {
    fn on_data(
        &mut self,
        flow_key: &FlowKey,
        direction: Direction,
        data: &[u8],
        offset: u64,
    );

    fn on_flow_close(
        &mut self,
        flow_key: &FlowKey,
        reason: CloseReason,
    );
}
```

### StreamAnalyzer Trait

Extends StreamHandler with reporting methods compatible with the existing Reporter system:

```rust
pub trait StreamAnalyzer: StreamHandler {
    fn name(&self) -> &'static str;
    fn summarize(&self) -> AnalysisSummary;
    fn findings(&self) -> Vec<Finding>;
}
```

Reporters consume `Vec<Finding>` and `Vec<AnalysisSummary>` from both `ProtocolAnalyzer` (per-packet) and `StreamAnalyzer` (stream-based) — no changes to reporters needed.

## Memory Management

### Per-Direction Depth Limit (default 10MB)

Once `reassembled_bytes` exceeds `max_depth_per_direction` on a flow direction:
- Stop storing new payload for that direction.
- Continue tracking the flow for metadata (packet counts, flags, timing).
- Generate a Finding: `[Anomaly] INCONCLUSIVE (LOW) — Flow {key} exceeded reassembly depth (10MB), stream truncated.`

### Global Memory Cap (default 1GB)

Before inserting a new segment, check `total_memory_used` against `global_memcap`. If exceeded:

1. Evict non-established flows first (SynSent, Closing, half-open) — likely port scans.
2. If still over, evict established flows by LRU (`last_seen` oldest first).
3. Each eviction: flush accumulated data to handler, call `on_flow_close(MemoryPressure)`.
4. Increment `stats.flows_evicted`.
5. Log warning to stderr: "Reassembly memory cap reached, evicting flows."

The final report includes: "Warning: reassembly memory cap reached, N flows evicted. Re-run with --reassembly-memcap to increase."

## Auto-Detect Activation

Reassembly is not always-on. It activates automatically when any TCP-based analyzer is enabled:

```rust
let needs_reassembly = enable_http || enable_tls || enable_smb || cli.reassemble;
let skip_reassembly = cli.no_reassemble;

let mut reassembler = if needs_reassembly && !skip_reassembly {
    Some(TcpReassembler::new(config))
} else {
    None
};
```

When no TCP analyzers are active and `--reassemble` is not set, the reassembler is not created. Zero overhead for DNS-only or summary-only runs.

## CLI Flags

```
--reassemble              Force TCP reassembly on
--no-reassemble           Force TCP reassembly off (quick header scan)
--reassembly-depth <MB>   Per-direction stream limit in MB (default: 10)
--reassembly-memcap <MB>  Global reassembly memory cap in MB (default: 1024)
```

## Edge Cases

| Scenario | Handling |
|----------|----------|
| Malformed TCP header | Skip packet, increment `stats.malformed_packets` |
| SYN retransmit with different ISN | Keep first ISN (first-wins) |
| Zero-window probes / keepalives | Detect single byte at `expected_seq - 1`, ignore |
| One-sided capture (only one direction) | Visible direction reassembles, other stays empty |
| Duplicate FIN/RST | Ignore after the first |
| Half-close (FIN one direction, data continues other) | Each direction tracked independently |
| Pcap timestamp not monotonic | Use packet timestamps as-is; timeout still works since we compare against `last_seen` |

## File Structure

```
src/reassembly/
├── mod.rs          — TcpReassembler, ReassemblyConfig, ReassemblyStats, public API
├── flow.rs         — FlowKey, TcpFlow, FlowDirection, FlowState
├── segment.rs      — Segment insertion, overlap handling, contiguous flush logic
└── handler.rs      — StreamHandler trait, StreamAnalyzer trait, Direction, CloseReason
```

## Testing Strategy

### Unit Tests

- **segment.rs**: Insert ordered, out-of-order, overlapping, retransmitted, and wrapping segments. Verify first-wins overlap. Verify contiguous flush produces correct bytes at correct offsets.
- **flow.rs**: State transitions through full lifecycle. FlowKey canonicalization (A→B == B→A). Mid-stream pickup sets partial flag. ISN inference from first data packet.
- **mod.rs**: Depth limit enforcement (>10MB stops buffering, generates Finding). Memcap eviction (non-established first, then LRU). Stats tracking.

### Integration Tests

All tests use synthetic packet bytes (same pattern as existing wirerust tests — no external fixtures).

- Three-packet stream in order → reassembled payload matches concatenation.
- Three-packet stream out of order [1, 3, 2] → same result after reordering.
- Retransmission of packet 1 → deduplicated, single copy in stream.
- Overlapping segments with different data → first-wins, original data preserved, Finding generated for conflicting data.
- Excessive overlaps (>50) → evasion-attempt Finding generated.
- Small segment flood (>2048 segments <8 bytes) → evasion-attempt Finding generated.
- Depth truncation mid-segment → partial segment stored up to limit, truncation Finding generated.
- 11MB stream → first 10MB reassembled, truncation Finding generated.
- 100+ flows exceeding memcap → eviction occurs, stats.flows_evicted > 0.
- Mid-stream flow (no SYN) → reassembles correctly, partial flag set.
- RST mid-stream → on_flow_close(Rst) called, accumulated data flushed.

## Future Considerations (Not In Scope)

These are deliberately deferred. They can be added later without changing the core architecture:

- **Configurable overlap policy** (last-wins, per-OS policy) — first-wins covers the majority of targets; add configurability only if users request it.
- **TCP Fast Open (TFO)** — Data in SYN packets. Rare in practice. Would require special handling of payload attached to SYN.
- **PAWS (Protection Against Wrapped Sequence Numbers)** — Uses TCP timestamps option to reject old duplicates. Not needed for offline pcap analysis since we see all packets.
- **Urgent pointer / out-of-band data** — Rarely used. Would need special offset handling.
- **ACK-based progress tracking** — Not needed for offline analysis. Sequence-number-only tracking is sufficient.
- **Live capture mode** — Current design uses pcap timestamps. Live mode would need wall-clock timeouts and periodic expiration on a timer.
- **Parallel reassembly with rayon** — Flow-level parallelism is possible since flows are independent, but adds complexity to the callback model.
