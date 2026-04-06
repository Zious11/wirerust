# TCP Stream Reassembly Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add forensic-grade TCP stream reassembly to wirerust with overlapping segment handling, memory limits, anomaly detection, and incremental stream delivery via callbacks.

**Architecture:** Standalone `src/reassembly/` module between decoder and analyzers. FlowKey identifies connections, BTreeMap<u64, Vec<u8>> stores out-of-order segments keyed by ISN-relative offset, contiguous data is flushed to StreamHandler callbacks incrementally. First-wins overlap policy. Configurable depth (10MB/direction) and memcap (1GB global).

**Tech Stack:** Rust 2024 edition. No new crate dependencies — uses std collections (HashMap, BTreeMap) and existing wirerust types (ParsedPacket, TransportInfo, Finding).

---

## File Structure

```
src/
├── decoder.rs             — MODIFY: add seq_number to TransportInfo::Tcp
├── cli.rs                 — MODIFY: add --reassemble, --no-reassemble, --reassembly-depth, --reassembly-memcap
├── lib.rs                 — MODIFY: add pub mod reassembly;
├── main.rs                — MODIFY: wire reassembler into analyze pipeline
├── reassembly/
│   ├── mod.rs             — TcpReassembler, ReassemblyConfig, ReassemblyStats, public API
│   ├── flow.rs            — FlowKey, TcpFlow, FlowDirection, FlowState, Direction, CloseReason
│   ├── segment.rs         — insert_segment(), flush_contiguous(), overlap trimming
│   └── handler.rs         — StreamHandler trait, StreamAnalyzer trait
tests/
├── decoder_tests.rs       — MODIFY: update tests for new seq_number field
├── reassembly_flow_tests.rs    — FlowKey canonicalization, state transitions
├── reassembly_segment_tests.rs — Segment insertion, overlap, flush, wraparound
├── reassembly_engine_tests.rs  — Full engine: depth limit, memcap, eviction, anomaly detection
```

---

### Task 1: Add seq_number to TransportInfo::Tcp

**Files:**
- Modify: `src/decoder.rs`
- Modify: `tests/decoder_tests.rs`
- Modify: `tests/analyzer_tests.rs`
- Modify: `tests/summary_tests.rs`
- Modify: `tests/integration_test.rs`

The reassembler needs the TCP sequence number from each packet. Currently `TransportInfo::Tcp` only has ports and flags.

- [ ] **Step 1: Update TransportInfo::Tcp to include seq_number**

In `src/decoder.rs`, change the `Tcp` variant:

```rust
#[derive(Debug, Clone)]
pub enum TransportInfo {
    Tcp {
        src_port: u16,
        dst_port: u16,
        seq_number: u32,
        syn: bool,
        ack: bool,
        fin: bool,
        rst: bool,
    },
    Udp {
        src_port: u16,
        dst_port: u16,
    },
    None,
}
```

In the `decode_packet` function, update the TCP match arm:

```rust
Some(etherparse::TransportSlice::Tcp(tcp)) => (
    Protocol::Tcp,
    TransportInfo::Tcp {
        src_port: tcp.source_port(),
        dst_port: tcp.destination_port(),
        seq_number: tcp.sequence_number(),
        syn: tcp.syn(),
        ack: tcp.ack(),
        fin: tcp.fin(),
        rst: tcp.rst(),
    },
),
```

- [ ] **Step 2: Fix all test files that construct TransportInfo::Tcp**

In `tests/decoder_tests.rs`, the existing tests use pattern matching with `..` so they should still compile. Verify by running:

Run: `cargo test --test decoder_tests`

In `tests/analyzer_tests.rs`, update `make_non_dns_packet()`:

```rust
fn make_non_dns_packet() -> ParsedPacket {
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        protocol: Protocol::Tcp,
        transport: TransportInfo::Tcp {
            src_port: 12345,
            dst_port: 80,
            seq_number: 1000,
            syn: true,
            ack: false,
            fin: false,
            rst: false,
        },
        payload: vec![],
        packet_len: 54,
    }
}
```

In `tests/summary_tests.rs`, update `make_parsed()`:

```rust
fn make_parsed(src: [u8; 4], dst: [u8; 4], src_port: u16, dst_port: u16) -> ParsedPacket {
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::from(src)),
        dst_ip: IpAddr::V4(Ipv4Addr::from(dst)),
        protocol: Protocol::Tcp,
        transport: TransportInfo::Tcp {
            src_port,
            dst_port,
            seq_number: 1000,
            syn: false,
            ack: false,
            fin: false,
            rst: false,
        },
        payload: vec![],
        packet_len: 54,
    }
}
```

- [ ] **Step 3: Run all tests**

Run: `cargo test`
Expected: All 19 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/decoder.rs tests/decoder_tests.rs tests/analyzer_tests.rs tests/summary_tests.rs tests/integration_test.rs
git commit -m "feat: add seq_number to TransportInfo::Tcp for reassembly"
```

---

### Task 2: StreamHandler and Flow Types (handler.rs + flow.rs)

**Files:**
- Create: `src/reassembly/handler.rs`
- Create: `src/reassembly/flow.rs`
- Create: `src/reassembly/mod.rs` (stub)
- Modify: `src/lib.rs`
- Create: `tests/reassembly_flow_tests.rs`

- [ ] **Step 1: Write the failing test for FlowKey canonicalization**

Create `tests/reassembly_flow_tests.rs`:

```rust
use std::net::{IpAddr, Ipv4Addr};

use wirerust::reassembly::flow::{FlowDirection, FlowKey, FlowState, TcpFlow};
use wirerust::reassembly::handler::Direction;

#[test]
fn test_flow_key_canonicalization() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

    let key_ab = FlowKey::new(ip_a, 12345, ip_b, 80);
    let key_ba = FlowKey::new(ip_b, 80, ip_a, 12345);

    assert_eq!(key_ab, key_ba);
    assert_eq!(key_ab.lower_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    assert_eq!(key_ab.lower_port, 80);
    assert_eq!(key_ab.upper_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)));
    assert_eq!(key_ab.upper_port, 12345);
}

#[test]
fn test_flow_key_same_ip_different_ports() {
    let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

    let key1 = FlowKey::new(ip, 80, ip, 12345);
    let key2 = FlowKey::new(ip, 12345, ip, 80);

    assert_eq!(key1, key2);
    assert_eq!(key1.lower_port, 80);
    assert_eq!(key1.upper_port, 12345);
}

#[test]
fn test_flow_direction_determines_client_server() {
    let ip_client = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_server = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

    let mut flow = TcpFlow::new(FlowKey::new(ip_client, 12345, ip_server, 80), 1000);
    flow.set_initiator(ip_client, 12345);

    assert_eq!(
        flow.direction(ip_client, 12345),
        Direction::ClientToServer
    );
    assert_eq!(
        flow.direction(ip_server, 80),
        Direction::ServerToClient
    );
}

#[test]
fn test_flow_state_transitions() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 12345, ip_b, 80), 1000);
    assert_eq!(flow.state, FlowState::New);

    flow.on_syn();
    assert_eq!(flow.state, FlowState::SynSent);

    flow.on_syn_ack();
    assert_eq!(flow.state, FlowState::Established);

    flow.on_fin();
    assert_eq!(flow.state, FlowState::Closing);

    flow.on_fin();
    assert_eq!(flow.state, FlowState::Closed);
}

#[test]
fn test_flow_rst_from_any_state() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 12345, ip_b, 80), 1000);
    flow.on_syn();
    assert_eq!(flow.state, FlowState::SynSent);

    flow.on_rst();
    assert_eq!(flow.state, FlowState::Closed);
}

#[test]
fn test_mid_stream_pickup() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 12345, ip_b, 80), 1000);
    flow.on_data_without_syn();
    assert_eq!(flow.state, FlowState::Established);
    assert!(flow.partial);
}

#[test]
fn test_flow_direction_new() {
    let dir = FlowDirection::new();
    assert_eq!(dir.isn, None);
    assert_eq!(dir.base_offset, 0);
    assert!(dir.segments.is_empty());
    assert_eq!(dir.reassembled_bytes, 0);
    assert!(!dir.fin_seen);
    assert!(!dir.rst_seen);
    assert!(!dir.depth_exceeded);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test reassembly_flow_tests`
Expected: FAIL — module `reassembly` not found.

- [ ] **Step 3: Create handler.rs with traits and enums**

Create `src/reassembly/handler.rs`:

```rust
use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;
use crate::reassembly::flow::FlowKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    ClientToServer,
    ServerToClient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    fn on_flow_close(&mut self, flow_key: &FlowKey, reason: CloseReason);
}

pub trait StreamAnalyzer: StreamHandler {
    fn name(&self) -> &'static str;
    fn summarize(&self) -> AnalysisSummary;
    fn findings(&self) -> Vec<Finding>;
}
```

- [ ] **Step 4: Create flow.rs with FlowKey, FlowDirection, TcpFlow, FlowState**

Create `src/reassembly/flow.rs`:

```rust
use std::collections::BTreeMap;
use std::net::IpAddr;

use crate::reassembly::handler::Direction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowKey {
    pub lower_ip: IpAddr,
    pub lower_port: u16,
    pub upper_ip: IpAddr,
    pub upper_port: u16,
}

impl FlowKey {
    pub fn new(ip_a: IpAddr, port_a: u16, ip_b: IpAddr, port_b: u16) -> Self {
        if (ip_a, port_a) <= (ip_b, port_b) {
            FlowKey {
                lower_ip: ip_a,
                lower_port: port_a,
                upper_ip: ip_b,
                upper_port: port_b,
            }
        } else {
            FlowKey {
                lower_ip: ip_b,
                lower_port: port_b,
                upper_ip: ip_a,
                upper_port: port_a,
            }
        }
    }
}

impl std::fmt::Display for FlowKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{} → {}:{}",
            self.lower_ip, self.lower_port, self.upper_ip, self.upper_port
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowState {
    New,
    SynSent,
    Established,
    Closing,
    Closed,
    TimedOut,
}

#[derive(Debug)]
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

impl FlowDirection {
    pub fn new() -> Self {
        FlowDirection {
            isn: None,
            base_offset: 0,
            segments: BTreeMap::new(),
            reassembled_bytes: 0,
            overlap_count: 0,
            small_segment_count: 0,
            fin_seen: false,
            rst_seen: false,
            depth_exceeded: false,
        }
    }

    pub fn set_isn(&mut self, isn: u32) {
        if self.isn.is_none() {
            self.isn = Some(isn);
            self.base_offset = 1; // ISN+1 is first data byte
        }
    }

    pub fn infer_isn(&mut self, first_seq: u32) {
        if self.isn.is_none() {
            self.isn = Some(first_seq.wrapping_sub(1));
            self.base_offset = 1;
        }
    }

    pub fn memory_used(&self) -> usize {
        self.segments.values().map(|v| v.len()).sum()
    }
}

#[derive(Debug)]
pub struct TcpFlow {
    pub key: FlowKey,
    pub client_to_server: FlowDirection,
    pub server_to_client: FlowDirection,
    pub state: FlowState,
    pub partial: bool,
    pub first_seen: u32,
    pub last_seen: u32,
    initiator_ip: Option<IpAddr>,
    initiator_port: Option<u16>,
    fin_count: u8,
}

impl TcpFlow {
    pub fn new(key: FlowKey, timestamp: u32) -> Self {
        TcpFlow {
            key,
            client_to_server: FlowDirection::new(),
            server_to_client: FlowDirection::new(),
            state: FlowState::New,
            partial: false,
            first_seen: timestamp,
            last_seen: timestamp,
            initiator_ip: None,
            initiator_port: None,
            fin_count: 0,
        }
    }

    pub fn set_initiator(&mut self, ip: IpAddr, port: u16) {
        if self.initiator_ip.is_none() {
            self.initiator_ip = Some(ip);
            self.initiator_port = Some(port);
        }
    }

    pub fn direction(&self, src_ip: IpAddr, src_port: u16) -> Direction {
        if self.initiator_ip == Some(src_ip) && self.initiator_port == Some(src_port) {
            Direction::ClientToServer
        } else {
            Direction::ServerToClient
        }
    }

    pub fn get_direction_mut(&mut self, dir: Direction) -> &mut FlowDirection {
        match dir {
            Direction::ClientToServer => &mut self.client_to_server,
            Direction::ServerToClient => &mut self.server_to_client,
        }
    }

    pub fn on_syn(&mut self) {
        if self.state == FlowState::New {
            self.state = FlowState::SynSent;
        }
    }

    pub fn on_syn_ack(&mut self) {
        if self.state == FlowState::SynSent || self.state == FlowState::New {
            self.state = FlowState::Established;
        }
    }

    pub fn on_data_without_syn(&mut self) {
        if self.state == FlowState::New {
            self.state = FlowState::Established;
            self.partial = true;
        }
    }

    pub fn on_fin(&mut self) {
        self.fin_count += 1;
        if self.fin_count >= 2 {
            self.state = FlowState::Closed;
        } else if self.state == FlowState::Established || self.state == FlowState::SynSent {
            self.state = FlowState::Closing;
        }
    }

    pub fn on_rst(&mut self) {
        self.state = FlowState::Closed;
    }

    pub fn memory_used(&self) -> usize {
        self.client_to_server.memory_used() + self.server_to_client.memory_used()
    }
}
```

- [ ] **Step 5: Create reassembly/mod.rs stub**

Create `src/reassembly/mod.rs`:

```rust
pub mod flow;
pub mod handler;
```

- [ ] **Step 6: Add reassembly module to lib.rs**

Update `src/lib.rs`:

```rust
pub mod analyzer;
pub mod cli;
pub mod decoder;
pub mod findings;
pub mod reader;
pub mod reassembly;
pub mod reporter;
pub mod summary;
```

- [ ] **Step 7: Run tests**

Run: `cargo test --test reassembly_flow_tests`
Expected: 7 tests PASS.

- [ ] **Step 8: Commit**

```bash
git add src/reassembly/ src/lib.rs tests/reassembly_flow_tests.rs
git commit -m "feat: add FlowKey, TcpFlow, FlowDirection, StreamHandler types"
```

---

### Task 3: Segment Insertion and Contiguous Flush (segment.rs)

**Files:**
- Create: `src/reassembly/segment.rs`
- Modify: `src/reassembly/mod.rs` (add `pub mod segment;`)
- Create: `tests/reassembly_segment_tests.rs`

- [ ] **Step 1: Write failing tests for segment operations**

Create `tests/reassembly_segment_tests.rs`:

```rust
use wirerust::reassembly::flow::FlowDirection;
use wirerust::reassembly::segment::{flush_contiguous, insert_segment, InsertResult};

#[test]
fn test_insert_single_segment() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    let result = insert_segment(&mut dir, 1001, b"hello", 10_485_760);
    assert_eq!(result, InsertResult::Inserted);
    assert_eq!(dir.segments.len(), 1);
    assert_eq!(dir.segments.get(&1), Some(&b"hello".to_vec()));
}

#[test]
fn test_flush_contiguous_single() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    insert_segment(&mut dir, 1001, b"hello", 10_485_760);

    let flushed = flush_contiguous(&mut dir);
    assert_eq!(flushed.len(), 1);
    assert_eq!(flushed[0].0, 1); // offset
    assert_eq!(flushed[0].1, b"hello");
    assert_eq!(dir.base_offset, 6); // 1 + 5
    assert_eq!(dir.reassembled_bytes, 5);
    assert!(dir.segments.is_empty());
}

#[test]
fn test_flush_contiguous_ordered() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    insert_segment(&mut dir, 1001, b"aaa", 10_485_760);
    insert_segment(&mut dir, 1004, b"bbb", 10_485_760);

    let flushed = flush_contiguous(&mut dir);
    assert_eq!(flushed.len(), 2);
    assert_eq!(flushed[0].1, b"aaa");
    assert_eq!(flushed[1].1, b"bbb");
    assert_eq!(dir.base_offset, 7); // 1 + 3 + 3
    assert!(dir.segments.is_empty());
}

#[test]
fn test_out_of_order_buffering() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert segment 2 first (out of order)
    insert_segment(&mut dir, 1004, b"bbb", 10_485_760);
    let flushed = flush_contiguous(&mut dir);
    assert!(flushed.is_empty()); // Can't flush — gap at offset 1

    // Now insert segment 1
    insert_segment(&mut dir, 1001, b"aaa", 10_485_760);
    let flushed = flush_contiguous(&mut dir);
    assert_eq!(flushed.len(), 2); // Both flush now
    assert_eq!(flushed[0].1, b"aaa");
    assert_eq!(flushed[1].1, b"bbb");
    assert_eq!(dir.base_offset, 7);
}

#[test]
fn test_retransmission_dedup() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    insert_segment(&mut dir, 1001, b"hello", 10_485_760);
    let result = insert_segment(&mut dir, 1001, b"hello", 10_485_760);
    assert_eq!(result, InsertResult::Duplicate);
    assert_eq!(dir.segments.len(), 1); // No duplicate stored
}

#[test]
fn test_overlap_first_wins() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert "AAABBB" at offset 1
    insert_segment(&mut dir, 1001, b"AAABBB", 10_485_760);

    // Overlapping insert: "XXXCC" at offset 4 (overlaps with "BBB" at 4-6)
    let result = insert_segment(&mut dir, 1004, b"XXXCC", 10_485_760);
    assert_eq!(result, InsertResult::PartialOverlap);
    assert_eq!(dir.overlap_count, 1);

    // Flush and verify: first 6 bytes from original, then "CC" from new
    let flushed = flush_contiguous(&mut dir);
    let all_bytes: Vec<u8> = flushed.iter().flat_map(|(_, data)| data.iter().copied()).collect();
    assert_eq!(&all_bytes, b"AAABBBCC");
}

#[test]
fn test_overlap_conflicting_data_detected() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    insert_segment(&mut dir, 1001, b"AAAA", 10_485_760);

    // Same range, different data
    let result = insert_segment(&mut dir, 1001, b"BBBB", 10_485_760);
    assert_eq!(result, InsertResult::ConflictingOverlap);
    assert_eq!(dir.overlap_count, 1);

    // Original data preserved (first-wins)
    let flushed = flush_contiguous(&mut dir);
    assert_eq!(flushed[0].1, b"AAAA");
}

#[test]
fn test_sequence_wraparound() {
    let mut dir = FlowDirection::new();
    // ISN near wraparound
    dir.set_isn(0xFFFF_FFF0);

    // First data byte at ISN+1 = 0xFFFF_FFF1, offset = 1
    insert_segment(&mut dir, 0xFFFF_FFF1, b"before", 10_485_760);
    // Next segment wraps: seq = 0xFFFF_FFF1 + 6 = 0xFFFF_FFF7, offset = 7
    insert_segment(&mut dir, 0xFFFF_FFF7, b"wrap", 10_485_760);
    // Another after wrap: seq = 0xFFFF_FFFB, offset = 11
    insert_segment(&mut dir, 0xFFFF_FFFB, b"around", 10_485_760);

    let flushed = flush_contiguous(&mut dir);
    let all_bytes: Vec<u8> = flushed.iter().flat_map(|(_, data)| data.iter().copied()).collect();
    assert_eq!(&all_bytes, b"beforewraparound");
}

#[test]
fn test_small_segment_tracking() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert small segments
    for i in 0..5u32 {
        let seq = 1001 + i;
        insert_segment(&mut dir, seq, &[b'a'], 10_485_760);
    }

    assert_eq!(dir.small_segment_count, 5);
}

#[test]
fn test_depth_limit_truncation() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 100; // small for testing
    let data = vec![b'A'; 80];
    insert_segment(&mut dir, 1001, &data, max_depth);
    flush_contiguous(&mut dir);
    assert_eq!(dir.reassembled_bytes, 80);
    assert!(!dir.depth_exceeded);

    // This should be truncated to 20 bytes
    let data2 = vec![b'B'; 50];
    let result = insert_segment(&mut dir, 1081, &data2, max_depth);
    assert_eq!(result, InsertResult::Truncated);
    assert!(dir.depth_exceeded);

    let flushed = flush_contiguous(&mut dir);
    assert_eq!(flushed[0].1.len(), 20); // truncated from 50 to 20
    assert_eq!(dir.reassembled_bytes, 100);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test reassembly_segment_tests`
Expected: FAIL — module `segment` not found.

- [ ] **Step 3: Implement segment.rs**

Create `src/reassembly/segment.rs`:

```rust
use crate::reassembly::flow::FlowDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertResult {
    Inserted,
    Duplicate,
    PartialOverlap,
    ConflictingOverlap,
    Truncated,
    DepthExceeded,
}

/// Compute the ISN-relative offset for a sequence number.
fn seq_offset(seq: u32, isn: u32) -> u64 {
    seq.wrapping_sub(isn) as u64
}

/// Insert a segment into the flow direction's out-of-order buffer.
/// Applies first-wins overlap policy and tracks anomaly counters.
pub fn insert_segment(
    dir: &mut FlowDirection,
    seq: u32,
    data: &[u8],
    max_depth: usize,
) -> InsertResult {
    if data.is_empty() {
        return InsertResult::Inserted;
    }

    let isn = match dir.isn {
        Some(isn) => isn,
        None => return InsertResult::Inserted, // no ISN yet, skip
    };

    // Track small segments
    if data.len() < 8 {
        dir.small_segment_count += 1;
    } else {
        dir.small_segment_count = 0; // reset on normal-sized segment
    }

    // Check depth limit
    let remaining_depth = max_depth.saturating_sub(dir.reassembled_bytes);
    if remaining_depth == 0 {
        if !dir.depth_exceeded {
            dir.depth_exceeded = true;
        }
        return InsertResult::DepthExceeded;
    }

    let offset = seq_offset(seq, isn);
    let mut segment_data = data.to_vec();

    // Truncate if exceeding depth
    let buffered: usize = dir.segments.values().map(|v| v.len()).sum();
    let total_after = dir.reassembled_bytes + buffered + segment_data.len();
    let truncated = if total_after > max_depth {
        let allowed = max_depth.saturating_sub(dir.reassembled_bytes + buffered);
        if allowed == 0 {
            dir.depth_exceeded = true;
            return InsertResult::DepthExceeded;
        }
        segment_data.truncate(allowed);
        dir.depth_exceeded = true;
        true
    } else {
        false
    };

    let new_start = offset;
    let new_end = offset + segment_data.len() as u64;

    // Check for overlaps with existing segments
    let mut has_overlap = false;
    let mut has_conflict = false;
    let mut trimmed_ranges: Vec<(u64, u64)> = Vec::new();

    // Collect existing segment ranges that overlap
    for (&existing_offset, existing_data) in dir.segments.iter() {
        let existing_end = existing_offset + existing_data.len() as u64;

        if new_start < existing_end && new_end > existing_offset {
            // Overlap detected
            has_overlap = true;

            // Check if overlapping region has different data (conflict)
            let overlap_start = new_start.max(existing_offset);
            let overlap_end = new_end.min(existing_end);

            for pos in overlap_start..overlap_end {
                let new_idx = (pos - new_start) as usize;
                let existing_idx = (pos - existing_offset) as usize;
                if new_idx < segment_data.len()
                    && existing_idx < existing_data.len()
                    && segment_data[new_idx] != existing_data[existing_idx]
                {
                    has_conflict = true;
                    break;
                }
            }

            trimmed_ranges.push((existing_offset, existing_end));
        }
    }

    if has_overlap {
        dir.overlap_count += 1;

        // Check if fully covered (duplicate/retransmission)
        let fully_covered = trimmed_ranges.iter().any(|&(es, ee)| es <= new_start && ee >= new_end);
        if fully_covered {
            return if has_conflict {
                InsertResult::ConflictingOverlap
            } else {
                InsertResult::Duplicate
            };
        }

        // First-wins: trim new segment to only cover gaps
        // Build list of gap regions within [new_start, new_end)
        let mut gaps: Vec<(u64, u64)> = Vec::new();
        let mut cursor = new_start;

        // Sort existing overlapping ranges
        let mut sorted_ranges = trimmed_ranges.clone();
        sorted_ranges.sort_by_key(|&(start, _)| start);

        for &(es, ee) in &sorted_ranges {
            if cursor < es {
                gaps.push((cursor, es.min(new_end)));
            }
            cursor = cursor.max(ee);
        }
        if cursor < new_end {
            gaps.push((cursor, new_end));
        }

        // Insert only gap portions
        for (gap_start, gap_end) in gaps {
            let start_idx = (gap_start - new_start) as usize;
            let end_idx = (gap_end - new_start) as usize;
            if start_idx < segment_data.len() && end_idx <= segment_data.len() {
                let gap_data = segment_data[start_idx..end_idx].to_vec();
                if !gap_data.is_empty() {
                    dir.segments.insert(gap_start, gap_data);
                }
            }
        }

        return if has_conflict {
            InsertResult::ConflictingOverlap
        } else if truncated {
            InsertResult::Truncated
        } else {
            InsertResult::PartialOverlap
        };
    }

    // No overlap — insert normally
    dir.segments.insert(offset, segment_data);

    if truncated {
        InsertResult::Truncated
    } else {
        InsertResult::Inserted
    }
}

/// Flush contiguous segments starting from base_offset.
/// Returns Vec of (offset, data) pairs that were flushed.
pub fn flush_contiguous(dir: &mut FlowDirection) -> Vec<(u64, Vec<u8>)> {
    let mut flushed = Vec::new();

    loop {
        if let Some(data) = dir.segments.remove(&dir.base_offset) {
            let offset = dir.base_offset;
            dir.base_offset += data.len() as u64;
            dir.reassembled_bytes += data.len();
            flushed.push((offset, data));
        } else {
            break;
        }
    }

    flushed
}
```

- [ ] **Step 4: Add segment module to reassembly/mod.rs**

Update `src/reassembly/mod.rs`:

```rust
pub mod flow;
pub mod handler;
pub mod segment;
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test reassembly_segment_tests`
Expected: 10 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/reassembly/segment.rs src/reassembly/mod.rs tests/reassembly_segment_tests.rs
git commit -m "feat: add segment insertion with first-wins overlap and contiguous flush"
```

---

### Task 4: TcpReassembler Engine (mod.rs)

**Files:**
- Modify: `src/reassembly/mod.rs`
- Create: `tests/reassembly_engine_tests.rs`

- [ ] **Step 1: Write failing tests for the engine**

Create `tests/reassembly_engine_tests.rs`:

```rust
use std::net::{IpAddr, Ipv4Addr};

use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};
use wirerust::findings::Finding;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{CloseReason, Direction, StreamHandler};
use wirerust::reassembly::mod_public::{ReassemblyConfig, ReassemblyStats, TcpReassembler};

/// Test handler that records all callbacks.
struct RecordingHandler {
    data_events: Vec<(FlowKey, Direction, Vec<u8>, u64)>,
    close_events: Vec<(FlowKey, CloseReason)>,
}

impl RecordingHandler {
    fn new() -> Self {
        RecordingHandler {
            data_events: Vec::new(),
            close_events: Vec::new(),
        }
    }

    fn all_data(&self) -> Vec<u8> {
        self.data_events
            .iter()
            .flat_map(|(_, _, data, _)| data.iter().copied())
            .collect()
    }
}

impl StreamHandler for RecordingHandler {
    fn on_data(
        &mut self,
        flow_key: &FlowKey,
        direction: Direction,
        data: &[u8],
        offset: u64,
    ) {
        self.data_events
            .push((flow_key.clone(), direction, data.to_vec(), offset));
    }

    fn on_flow_close(&mut self, flow_key: &FlowKey, reason: CloseReason) {
        self.close_events.push((flow_key.clone(), reason));
    }
}

fn make_tcp_packet(
    src_ip: [u8; 4],
    src_port: u16,
    dst_ip: [u8; 4],
    dst_port: u16,
    seq: u32,
    payload: &[u8],
    syn: bool,
    fin: bool,
    rst: bool,
) -> ParsedPacket {
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::from(src_ip)),
        dst_ip: IpAddr::V4(Ipv4Addr::from(dst_ip)),
        protocol: Protocol::Tcp,
        transport: TransportInfo::Tcp {
            src_port,
            dst_port,
            seq_number: seq,
            syn,
            ack: false,
            fin,
            rst,
        },
        payload: payload.to_vec(),
        packet_len: 54 + payload.len(),
    }
}

#[test]
fn test_three_packet_stream_ordered() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN
    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false);
    reassembler.process_packet(&syn, 1, &mut handler);

    // Data packets
    let p1 = make_tcp_packet(client, 12345, server, 80, 1001, b"aaa", false, false, false);
    reassembler.process_packet(&p1, 2, &mut handler);

    let p2 = make_tcp_packet(client, 12345, server, 80, 1004, b"bbb", false, false, false);
    reassembler.process_packet(&p2, 3, &mut handler);

    let p3 = make_tcp_packet(client, 12345, server, 80, 1007, b"ccc", false, false, false);
    reassembler.process_packet(&p3, 4, &mut handler);

    assert_eq!(handler.all_data(), b"aaabbbccc");
    assert_eq!(handler.data_events.len(), 3);
}

#[test]
fn test_out_of_order_delivery() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN
    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false);
    reassembler.process_packet(&syn, 1, &mut handler);

    // Send packets [1, 3, 2]
    let p1 = make_tcp_packet(client, 12345, server, 80, 1001, b"aaa", false, false, false);
    reassembler.process_packet(&p1, 2, &mut handler);

    let p3 = make_tcp_packet(client, 12345, server, 80, 1007, b"ccc", false, false, false);
    reassembler.process_packet(&p3, 3, &mut handler);
    assert_eq!(handler.data_events.len(), 1); // only p1 flushed

    let p2 = make_tcp_packet(client, 12345, server, 80, 1004, b"bbb", false, false, false);
    reassembler.process_packet(&p2, 4, &mut handler);

    // Now all three should be flushed
    assert_eq!(handler.all_data(), b"aaabbbccc");
}

#[test]
fn test_mid_stream_no_syn() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Data without SYN
    let p1 = make_tcp_packet(client, 12345, server, 80, 5000, b"hello", false, false, false);
    reassembler.process_packet(&p1, 1, &mut handler);

    assert_eq!(handler.all_data(), b"hello");

    let stats = reassembler.stats();
    assert_eq!(stats.flows_total, 1);
    assert_eq!(stats.flows_partial, 1);
}

#[test]
fn test_rst_closes_flow() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false);
    reassembler.process_packet(&syn, 1, &mut handler);

    let data = make_tcp_packet(client, 12345, server, 80, 1001, b"data", false, false, false);
    reassembler.process_packet(&data, 2, &mut handler);

    let rst = make_tcp_packet(server, 80, client, 12345, 2000, &[], false, false, true);
    reassembler.process_packet(&rst, 3, &mut handler);

    assert_eq!(handler.close_events.len(), 1);
    assert_eq!(handler.close_events[0].1, CloseReason::Rst);
}

#[test]
fn test_finalize_flushes_remaining() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false);
    reassembler.process_packet(&syn, 1, &mut handler);

    let data = make_tcp_packet(client, 12345, server, 80, 1001, b"leftover", false, false, false);
    reassembler.process_packet(&data, 2, &mut handler);

    // Finalize — should close all flows
    reassembler.finalize(&mut handler);

    assert_eq!(handler.close_events.len(), 1);
    assert_eq!(handler.close_events[0].1, CloseReason::Timeout);
}

#[test]
fn test_flow_timeout_expiration() {
    let config = ReassemblyConfig {
        flow_timeout_secs: 10,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false);
    reassembler.process_packet(&syn, 100, &mut handler);

    // Expire at time 200 (100 seconds later, > 10s timeout)
    reassembler.expire_flows(200, &mut handler);

    assert_eq!(handler.close_events.len(), 1);
    assert_eq!(handler.close_events[0].1, CloseReason::Timeout);

    let stats = reassembler.stats();
    assert_eq!(stats.flows_expired, 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test reassembly_engine_tests`
Expected: FAIL — `mod_public` not found (we need to expose TcpReassembler from mod.rs).

- [ ] **Step 3: Implement TcpReassembler in mod.rs**

Replace `src/reassembly/mod.rs` with:

```rust
pub mod flow;
pub mod handler;
pub mod segment;

use std::collections::HashMap;

use crate::decoder::{ParsedPacket, Protocol, TransportInfo};
use crate::findings::{Confidence, Finding, ThreatCategory, Verdict};
use crate::reassembly::flow::{FlowKey, FlowState, TcpFlow};
use crate::reassembly::handler::{CloseReason, Direction, StreamHandler};
use crate::reassembly::segment::{flush_contiguous, insert_segment, InsertResult};

#[derive(Debug, Clone)]
pub struct ReassemblyConfig {
    pub max_depth_per_direction: usize,
    pub global_memcap: usize,
    pub flow_timeout_secs: u32,
}

impl Default for ReassemblyConfig {
    fn default() -> Self {
        ReassemblyConfig {
            max_depth_per_direction: 10_485_760, // 10MB
            global_memcap: 1_073_741_824,        // 1GB
            flow_timeout_secs: 300,              // 5 min
        }
    }
}

#[derive(Debug, Default)]
pub struct ReassemblyStats {
    pub flows_total: u64,
    pub flows_partial: u64,
    pub flows_expired: u64,
    pub flows_evicted: u64,
    pub packets_processed: u64,
    pub packets_skipped: u64,
    pub depth_exceeded_count: u64,
    pub memcap_exceeded: bool,
}

pub struct TcpReassembler {
    flows: HashMap<FlowKey, TcpFlow>,
    config: ReassemblyConfig,
    total_memory_used: usize,
    stats: ReassemblyStats,
    findings: Vec<Finding>,
}

impl TcpReassembler {
    pub fn new(config: ReassemblyConfig) -> Self {
        TcpReassembler {
            flows: HashMap::new(),
            config,
            total_memory_used: 0,
            stats: ReassemblyStats::default(),
            findings: Vec::new(),
        }
    }

    pub fn process_packet(
        &mut self,
        packet: &ParsedPacket,
        timestamp: u32,
        handler: &mut dyn StreamHandler,
    ) {
        // Only process TCP packets
        let (src_port, dst_port, seq_number, syn, fin, rst) = match &packet.transport {
            TransportInfo::Tcp {
                src_port,
                dst_port,
                seq_number,
                syn,
                fin,
                rst,
                ..
            } => (*src_port, *dst_port, *seq_number, *syn, *fin, *rst),
            _ => {
                self.stats.packets_skipped += 1;
                return;
            }
        };

        self.stats.packets_processed += 1;

        let key = FlowKey::new(packet.src_ip, src_port, packet.dst_ip, dst_port);

        // Get or create flow
        let is_new_flow = !self.flows.contains_key(&key);
        let flow = self.flows.entry(key.clone()).or_insert_with(|| {
            let mut f = TcpFlow::new(key.clone(), timestamp);
            self.stats.flows_total += 1;
            f
        });
        flow.last_seen = timestamp;

        // Determine direction and handle flags
        if syn && !flow.client_to_server.fin_seen {
            flow.set_initiator(packet.src_ip, src_port);
            if rst {
                // SYN+RST is weird, ignore
            } else if flow.state == FlowState::SynSent {
                // SYN+ACK (server response)
                flow.on_syn_ack();
                flow.server_to_client.set_isn(seq_number);
            } else {
                // SYN (client initiating)
                flow.on_syn();
                flow.client_to_server.set_isn(seq_number);
            }
        }

        if rst {
            flow.on_rst();
            handler.on_flow_close(&flow.key, CloseReason::Rst);
            return;
        }

        if fin {
            let dir = flow.direction(packet.src_ip, src_port);
            flow.get_direction_mut(dir).fin_seen = true;
            flow.on_fin();
        }

        // Process payload
        if !packet.payload.is_empty() {
            if flow.state == FlowState::New {
                // Mid-stream pickup
                flow.set_initiator(packet.src_ip, src_port);
                flow.on_data_without_syn();
                self.stats.flows_partial += 1;
            }

            let dir = flow.direction(packet.src_ip, src_port);
            let flow_dir = flow.get_direction_mut(dir);

            // Infer ISN if not set (mid-stream)
            if flow_dir.isn.is_none() {
                flow_dir.infer_isn(seq_number);
            }

            // Check memcap before inserting
            if self.total_memory_used + packet.payload.len() > self.config.global_memcap {
                self.evict_flows(timestamp, handler);
                self.stats.memcap_exceeded = true;
                eprintln!(
                    "Warning: reassembly memory cap reached, evicting flows. \
                     Re-run with --reassembly-memcap to increase."
                );
            }

            let result = insert_segment(
                flow_dir,
                seq_number,
                &packet.payload,
                self.config.max_depth_per_direction,
            );

            // Generate findings for anomalies
            match result {
                InsertResult::ConflictingOverlap => {
                    self.findings.push(Finding {
                        category: ThreatCategory::Anomaly,
                        verdict: Verdict::Likely,
                        confidence: Confidence::High,
                        summary: format!(
                            "Conflicting data in overlapping TCP segments on flow {}",
                            flow.key
                        ),
                        evidence: vec!["Possible insertion/evasion attack".into()],
                        mitre_technique: Some("T1036".into()),
                        source_ip: Some(packet.src_ip),
                        timestamp: None,
                    });
                }
                InsertResult::Truncated => {
                    self.stats.depth_exceeded_count += 1;
                    self.findings.push(Finding {
                        category: ThreatCategory::Anomaly,
                        verdict: Verdict::Inconclusive,
                        confidence: Confidence::Low,
                        summary: format!(
                            "Flow {} exceeded reassembly depth ({}MB), stream truncated",
                            flow.key,
                            self.config.max_depth_per_direction / 1_048_576,
                        ),
                        evidence: vec![],
                        mitre_technique: None,
                        source_ip: None,
                        timestamp: None,
                    });
                }
                InsertResult::DepthExceeded => {
                    // Already counted
                }
                _ => {}
            }

            // Check overlap count threshold
            if flow_dir.overlap_count == 51 {
                self.findings.push(Finding {
                    category: ThreatCategory::Anomaly,
                    verdict: Verdict::Likely,
                    confidence: Confidence::Medium,
                    summary: format!(
                        "Excessive TCP segment overlaps on flow {} ({} overlaps)",
                        flow.key, flow_dir.overlap_count
                    ),
                    evidence: vec!["Possible evasion attempt".into()],
                    mitre_technique: Some("T1036".into()),
                    source_ip: Some(packet.src_ip),
                    timestamp: None,
                });
            }

            // Check small segment flood
            if flow_dir.small_segment_count == 2049 {
                self.findings.push(Finding {
                    category: ThreatCategory::Anomaly,
                    verdict: Verdict::Inconclusive,
                    confidence: Confidence::Medium,
                    summary: format!(
                        "Excessive small TCP segments on flow {} ({} segments <8 bytes)",
                        flow.key, flow_dir.small_segment_count
                    ),
                    evidence: vec!["Possible IDS evasion".into()],
                    mitre_technique: None,
                    source_ip: Some(packet.src_ip),
                    timestamp: None,
                });
            }

            // Flush contiguous data
            let flushed = flush_contiguous(flow_dir);
            let dir_enum = dir;
            for (offset, data) in &flushed {
                self.total_memory_used = self.total_memory_used.saturating_sub(data.len());
                handler.on_data(&flow.key, dir_enum, data, *offset);
            }

            // Track memory added by non-flushed segments
            self.total_memory_used = self
                .flows
                .values()
                .map(|f| f.memory_used())
                .sum();
        }
    }

    pub fn expire_flows(&mut self, current_time: u32, handler: &mut dyn StreamHandler) {
        let timeout = self.config.flow_timeout_secs;
        let expired_keys: Vec<FlowKey> = self
            .flows
            .iter()
            .filter(|(_, flow)| {
                flow.state != FlowState::Closed
                    && current_time.saturating_sub(flow.last_seen) > timeout
            })
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            if let Some(mut flow) = self.flows.remove(&key) {
                flow.state = FlowState::TimedOut;
                self.total_memory_used = self.total_memory_used.saturating_sub(flow.memory_used());
                handler.on_flow_close(&flow.key, CloseReason::Timeout);
                self.stats.flows_expired += 1;
            }
        }
    }

    pub fn finalize(&mut self, handler: &mut dyn StreamHandler) {
        let all_keys: Vec<FlowKey> = self.flows.keys().cloned().collect();
        for key in all_keys {
            if let Some(flow) = self.flows.remove(&key) {
                self.total_memory_used = self.total_memory_used.saturating_sub(flow.memory_used());
                handler.on_flow_close(&flow.key, CloseReason::Timeout);
            }
        }
    }

    pub fn stats(&self) -> &ReassemblyStats {
        &self.stats
    }

    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    fn evict_flows(&mut self, current_time: u32, handler: &mut dyn StreamHandler) {
        // First evict non-established flows
        let non_established: Vec<FlowKey> = self
            .flows
            .iter()
            .filter(|(_, f)| {
                f.state != FlowState::Established && f.state != FlowState::Closing
            })
            .map(|(k, _)| k.clone())
            .collect();

        for key in non_established {
            if let Some(flow) = self.flows.remove(&key) {
                self.total_memory_used = self.total_memory_used.saturating_sub(flow.memory_used());
                handler.on_flow_close(&flow.key, CloseReason::MemoryPressure);
                self.stats.flows_evicted += 1;
            }
            if self.total_memory_used < self.config.global_memcap {
                return;
            }
        }

        // Then evict LRU established flows
        let mut by_last_seen: Vec<(FlowKey, u32)> = self
            .flows
            .iter()
            .map(|(k, f)| (k.clone(), f.last_seen))
            .collect();
        by_last_seen.sort_by_key(|(_, ts)| *ts);

        for (key, _) in by_last_seen {
            if self.total_memory_used < self.config.global_memcap {
                return;
            }
            if let Some(flow) = self.flows.remove(&key) {
                self.total_memory_used = self.total_memory_used.saturating_sub(flow.memory_used());
                handler.on_flow_close(&flow.key, CloseReason::MemoryPressure);
                self.stats.flows_evicted += 1;
            }
        }
    }
}
```

- [ ] **Step 4: Fix test imports**

The test uses `mod_public` which was a placeholder. The actual types are at `wirerust::reassembly::{ReassemblyConfig, ReassemblyStats, TcpReassembler}`. Update the import in `tests/reassembly_engine_tests.rs`:

Replace:
```rust
use wirerust::reassembly::mod_public::{ReassemblyConfig, ReassemblyStats, TcpReassembler};
```
With:
```rust
use wirerust::reassembly::{ReassemblyConfig, ReassemblyStats, TcpReassembler};
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test reassembly_engine_tests`
Expected: 6 tests PASS.

Run: `cargo test`
Expected: All tests pass (existing + new).

- [ ] **Step 6: Commit**

```bash
git add src/reassembly/mod.rs tests/reassembly_engine_tests.rs
git commit -m "feat: add TcpReassembler engine with memcap, depth limits, and anomaly detection"
```

---

### Task 5: CLI Flags and main.rs Integration

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/main.rs`
- Modify: `tests/cli_tests.rs`

- [ ] **Step 1: Add reassembly CLI flags**

In `src/cli.rs`, add to the `Cli` struct:

```rust
/// Force TCP stream reassembly on
#[arg(long, global = true)]
pub reassemble: bool,

/// Force TCP stream reassembly off (quick scan)
#[arg(long, global = true)]
pub no_reassemble: bool,

/// Per-direction stream reassembly limit in MB (default: 10)
#[arg(long, global = true, default_value_t = 10)]
pub reassembly_depth: usize,

/// Global reassembly memory cap in MB (default: 1024)
#[arg(long, global = true, default_value_t = 1024)]
pub reassembly_memcap: usize,
```

- [ ] **Step 2: Add CLI test for reassembly flags**

Add to `tests/cli_tests.rs`:

```rust
#[test]
fn test_reassembly_flags() {
    let cli = Cli::parse_from([
        "wirerust",
        "analyze",
        "test.pcap",
        "--reassemble",
        "--reassembly-depth",
        "20",
        "--reassembly-memcap",
        "2048",
    ]);
    assert!(cli.reassemble);
    assert_eq!(cli.reassembly_depth, 20);
    assert_eq!(cli.reassembly_memcap, 2048);
}

#[test]
fn test_no_reassemble_flag() {
    let cli = Cli::parse_from(["wirerust", "analyze", "test.pcap", "--no-reassemble"]);
    assert!(cli.no_reassemble);
}
```

- [ ] **Step 3: Wire reassembler into main.rs**

Update `src/main.rs` — add imports and modify `run_analyze`:

Add to imports:
```rust
use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};
use wirerust::reassembly::handler::StreamHandler;
```

Update `run_analyze` to create and use the reassembler:

```rust
fn run_analyze(
    targets: &[std::path::PathBuf],
    enable_dns: bool,
    use_color: bool,
    cli: &Cli,
) -> Result<()> {
    let mut summary = Summary::new();
    let mut dns_analyzer = DnsAnalyzer::new();
    let mut all_findings = Vec::new();

    // Determine if reassembly is needed
    let needs_reassembly = cli.reassemble; // Will expand when HTTP/TLS analyzers added
    let skip_reassembly = cli.no_reassemble;

    let mut reassembler = if needs_reassembly && !skip_reassembly {
        let config = ReassemblyConfig {
            max_depth_per_direction: cli.reassembly_depth * 1_048_576,
            global_memcap: cli.reassembly_memcap * 1_048_576,
            ..ReassemblyConfig::default()
        };
        Some(TcpReassembler::new(config))
    } else {
        None
    };

    // Placeholder handler for now — will be replaced by actual stream analyzers
    struct NullHandler;
    impl StreamHandler for NullHandler {
        fn on_data(&mut self, _: &wirerust::reassembly::flow::FlowKey, _: wirerust::reassembly::handler::Direction, _: &[u8], _: u64) {}
        fn on_flow_close(&mut self, _: &wirerust::reassembly::flow::FlowKey, _: wirerust::reassembly::handler::CloseReason) {}
    }
    let mut stream_handler = NullHandler;

    for target in targets {
        let pcap_files = resolve_targets(target)?;
        for path in &pcap_files {
            let source = PcapSource::from_file(path)
                .with_context(|| format!("Failed to read {}", path.display()))?;

            let pb = ProgressBar::new(source.packets.len() as u64);
            pb.set_style(ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40} {pos}/{len} packets",
            )?);

            for raw in &source.packets {
                if let Ok(parsed) = decode_packet(&raw.data) {
                    summary.ingest(&parsed);

                    if enable_dns && dns_analyzer.can_decode(&parsed) {
                        let findings = dns_analyzer.analyze(&parsed);
                        all_findings.extend(findings);
                    }

                    if let Some(ref mut reasm) = reassembler {
                        reasm.process_packet(&parsed, raw.timestamp_secs, &mut stream_handler);
                    }
                }
                pb.inc(1);
            }
            pb.finish_and_clear();
        }
    }

    // Finalize reassembler
    if let Some(ref mut reasm) = reassembler {
        reasm.finalize(&mut stream_handler);
        all_findings.extend(reasm.findings().to_vec());
    }

    let analyzer_summaries = if enable_dns {
        vec![dns_analyzer.summarize()]
    } else {
        vec![]
    };

    let output = match cli.output_format {
        Some(OutputFormat::Json) => {
            let reporter = JsonReporter;
            reporter.render(&summary, &all_findings, &analyzer_summaries)
        }
        _ => {
            let reporter = TerminalReporter { use_color };
            reporter.render(&summary, &all_findings, &analyzer_summaries)
        }
    };

    println!("{output}");
    Ok(())
}
```

- [ ] **Step 4: Run all tests**

Run: `cargo test`
Expected: All tests pass.

Run: `cargo clippy --all-targets -- -D warnings`
Expected: No errors.

Run: `cargo run -- --help`
Expected: Shows `--reassemble`, `--no-reassemble`, `--reassembly-depth`, `--reassembly-memcap` flags.

- [ ] **Step 5: Commit**

```bash
git add src/cli.rs src/main.rs tests/cli_tests.rs
git commit -m "feat: wire TCP reassembler into CLI and analyze pipeline"
```

---

### Task 6: Final Validation and Push

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests pass.

Run: `cargo clippy --all-targets -- -D warnings`
Expected: No errors.

Run: `cargo fmt --all --check`
Expected: No formatting issues.

- [ ] **Step 2: Push branch**

```bash
git push -u origin feature/tcp-reassembly
```

- [ ] **Step 3: Create PR**

```bash
gh pr create --repo Zious11/wirerust --base develop --title "feat: add TCP stream reassembly engine" --body "$(cat <<'EOF'
## Summary
- Forensic-grade TCP stream reassembly module (`src/reassembly/`)
- FlowKey canonicalization, ISN-relative u64 offsets, BTreeMap segment storage
- First-wins overlap policy with anomaly detection (conflicting data, excessive overlaps, small segment floods)
- Configurable depth limit (10MB/direction) and global memcap (1GB) with LRU eviction
- Mid-stream pickup (missing SYN) with partial flow flagging
- Incremental stream delivery via StreamHandler callbacks
- CLI flags: `--reassemble`, `--no-reassemble`, `--reassembly-depth`, `--reassembly-memcap`

Closes: n/a (infrastructure for #1, #2)

## Test plan
- [ ] `cargo test` — all tests pass
- [ ] `cargo clippy -- -D warnings` — clean
- [ ] `cargo fmt --check` — clean
- [ ] Segment tests: ordered, out-of-order, overlap, retransmit, wraparound, depth truncation
- [ ] Engine tests: three-packet stream, OOO delivery, mid-stream, RST, finalize, timeout
- [ ] Flow tests: canonicalization, state transitions, direction detection
EOF
)"
```
