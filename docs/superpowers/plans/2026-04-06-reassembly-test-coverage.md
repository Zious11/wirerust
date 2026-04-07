# Reassembly Test Coverage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 7 missing integration tests to the TCP reassembly engine, covering SYN+ACK bidirectional data, flow eviction, FIN teardown, anomaly findings, and segment limits.

**Architecture:** All tests go in `tests/reassembly_engine_tests.rs`. The shared `make_tcp_packet` helper gains an `ack` parameter. No production code changes.

**Tech Stack:** Rust 2024 edition, `cargo test`

---

### Task 1: Add `ack` Parameter to `make_tcp_packet` Helper

**Files:**
- Modify: `tests/reassembly_engine_tests.rs`

- [ ] **Step 1: Update `make_tcp_packet` signature and body**

Add `ack: bool` parameter after `syn`. Pass it to `TransportInfo::Tcp`:

```rust
#[allow(clippy::too_many_arguments)]
fn make_tcp_packet(
    src_ip: [u8; 4],
    src_port: u16,
    dst_ip: [u8; 4],
    dst_port: u16,
    seq: u32,
    payload: &[u8],
    syn: bool,
    ack: bool,
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
            ack,
            fin,
            rst,
        },
        payload: payload.to_vec(),
        packet_len: 54 + payload.len(),
    }
}
```

- [ ] **Step 2: Update all existing call sites**

Every existing call passes `false` for the new `ack` parameter (inserted after `syn`). The calls follow the pattern:

```rust
// Before:
make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false)
//                                                      syn   fin   rst

// After:
make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false, false)
//                                                      syn   ack   fin   rst
```

Update ALL call sites in these tests:
- `test_three_packet_stream_ordered` (4 calls)
- `test_out_of_order_delivery` (4 calls)
- `test_mid_stream_no_syn` (1 call)
- `test_rst_closes_flow` (3 calls)
- `test_finalize_flushes_remaining` (2 calls)
- `test_flow_timeout_expiration` (1 call)
- `test_total_memory_tracking` (3 calls)
- `test_fin_close_total_memory` (4 calls)

- [ ] **Step 3: Run tests to verify no regressions**

Run: `cargo test --test reassembly_engine_tests`
Expected: All 8 existing tests pass.

- [ ] **Step 4: Commit**

```bash
git add tests/reassembly_engine_tests.rs
git commit -m "refactor: add ack parameter to make_tcp_packet test helper"
```

---

### Task 2: Add SYN+ACK Bidirectional and FIN Teardown Tests

**Files:**
- Modify: `tests/reassembly_engine_tests.rs`

**Context:** These tests need the `Direction` import for asserting data direction. The `RecordingHandler` already records `Direction` in `data_events`.

- [ ] **Step 1: Write `test_syn_ack_bidirectional_data`**

```rust
#[test]
fn test_syn_ack_bidirectional_data() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // 3-way handshake: SYN, SYN+ACK
    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false, false);
    reassembler.process_packet(&syn, 1, &mut handler);

    let syn_ack = make_tcp_packet(server, 80, client, 12345, 2000, &[], true, true, false, false);
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Client sends data
    let req = make_tcp_packet(client, 12345, server, 80, 1001, b"request", false, false, false, false);
    reassembler.process_packet(&req, 3, &mut handler);

    // Server sends data
    let resp = make_tcp_packet(server, 80, client, 12345, 2001, b"response", false, false, false, false);
    reassembler.process_packet(&resp, 4, &mut handler);

    // Verify proper handshake (not partial/mid-stream)
    let stats = reassembler.stats();
    assert_eq!(stats.flows_partial, 0);
    assert_eq!(stats.flows_total, 1);

    // Verify bidirectional data with correct directions
    assert_eq!(handler.data_events.len(), 2);
    assert_eq!(handler.data_events[0].1, Direction::ClientToServer);
    assert_eq!(handler.data_events[0].2, b"request");
    assert_eq!(handler.data_events[1].1, Direction::ServerToClient);
    assert_eq!(handler.data_events[1].2, b"response");
}
```

- [ ] **Step 2: Write `test_full_handshake_fin_teardown`**

```rust
#[test]
fn test_full_handshake_fin_teardown() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Full 3-way handshake
    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false, false);
    reassembler.process_packet(&syn, 1, &mut handler);

    let syn_ack = make_tcp_packet(server, 80, client, 12345, 2000, &[], true, true, false, false);
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Bidirectional data
    let req = make_tcp_packet(client, 12345, server, 80, 1001, b"hello", false, false, false, false);
    reassembler.process_packet(&req, 3, &mut handler);

    let resp = make_tcp_packet(server, 80, client, 12345, 2001, b"world", false, false, false, false);
    reassembler.process_packet(&resp, 4, &mut handler);

    // FIN from client
    let fin1 = make_tcp_packet(client, 12345, server, 80, 1006, &[], false, false, true, false);
    reassembler.process_packet(&fin1, 5, &mut handler);

    // FIN from server
    let fin2 = make_tcp_packet(server, 80, client, 12345, 2006, &[], false, false, true, false);
    reassembler.process_packet(&fin2, 6, &mut handler);

    // Flow closed via FIN
    let stats = reassembler.stats();
    assert_eq!(stats.flows_fin, 1);
    assert_eq!(reassembler.total_memory(), 0);

    // Close reason is Fin
    assert_eq!(handler.close_events.len(), 1);
    assert_eq!(handler.close_events[0].1, CloseReason::Fin);

    // Both directions' data delivered
    let client_data: Vec<&[u8]> = handler
        .data_events
        .iter()
        .filter(|(_, d, _, _)| *d == Direction::ClientToServer)
        .map(|(_, _, data, _)| data.as_slice())
        .collect();
    let server_data: Vec<&[u8]> = handler
        .data_events
        .iter()
        .filter(|(_, d, _, _)| *d == Direction::ServerToClient)
        .map(|(_, _, data, _)| data.as_slice())
        .collect();
    assert_eq!(client_data, vec![b"hello".as_slice()]);
    assert_eq!(server_data, vec![b"world".as_slice()]);
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --test reassembly_engine_tests`
Expected: All tests pass including 2 new ones.

- [ ] **Step 4: Commit**

```bash
git add tests/reassembly_engine_tests.rs
git commit -m "test: add SYN+ACK bidirectional and FIN teardown tests"
```

---

### Task 3: Add Eviction Tests (max_flows and memcap)

**Files:**
- Modify: `tests/reassembly_engine_tests.rs`

- [ ] **Step 1: Write `test_max_flows_eviction`**

```rust
#[test]
fn test_max_flows_eviction() {
    let config = ReassemblyConfig {
        max_flows: 2,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Flow A (oldest): SYN + data
    let syn_a = make_tcp_packet([10, 0, 0, 1], 1000, server, 80, 1000, &[], true, false, false, false);
    reassembler.process_packet(&syn_a, 1, &mut handler);
    let data_a = make_tcp_packet([10, 0, 0, 1], 1000, server, 80, 1001, b"aaa", false, false, false, false);
    reassembler.process_packet(&data_a, 2, &mut handler);

    // Flow B: SYN + data
    let syn_b = make_tcp_packet([10, 0, 0, 1], 2000, server, 80, 1000, &[], true, false, false, false);
    reassembler.process_packet(&syn_b, 3, &mut handler);
    let data_b = make_tcp_packet([10, 0, 0, 1], 2000, server, 80, 1001, b"bbb", false, false, false, false);
    reassembler.process_packet(&data_b, 4, &mut handler);

    // Flow C: SYN triggers eviction (max_flows=2, already have 2)
    let syn_c = make_tcp_packet([10, 0, 0, 1], 3000, server, 80, 1000, &[], true, false, false, false);
    reassembler.process_packet(&syn_c, 5, &mut handler);

    // Eviction occurred
    let stats = reassembler.stats();
    assert!(stats.evictions >= 1);

    // MemoryPressure close reason present
    assert!(
        handler
            .close_events
            .iter()
            .any(|(_, r)| *r == CloseReason::MemoryPressure)
    );

    // Flow C was successfully created
    assert_eq!(stats.flows_total, 3);
}
```

- [ ] **Step 2: Write `test_memcap_eviction`**

```rust
#[test]
fn test_memcap_eviction() {
    let config = ReassemblyConfig {
        memcap: 10,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Flow A: SYN + out-of-order data (buffered, not flushed)
    let syn_a = make_tcp_packet(client, 1000, server, 80, 1000, &[], true, false, false, false);
    reassembler.process_packet(&syn_a, 1, &mut handler);

    // Skip offset 1 to prevent flush — send at offset 2 (seq 1002)
    let data_a1 = make_tcp_packet(client, 1000, server, 80, 1002, b"aaaaa", false, false, false, false);
    reassembler.process_packet(&data_a1, 2, &mut handler);
    assert_eq!(reassembler.total_memory(), 5);

    // Flow B: SYN + out-of-order data that pushes past memcap
    let syn_b = make_tcp_packet(client, 2000, server, 80, 2000, &[], true, false, false, false);
    reassembler.process_packet(&syn_b, 3, &mut handler);

    let data_b1 = make_tcp_packet(client, 2000, server, 80, 2002, b"bbbbbb", false, false, false, false);
    reassembler.process_packet(&data_b1, 4, &mut handler);
    // total_memory would be 11 (5+6) which exceeds memcap=10, triggering eviction

    // Eviction should have fired
    let stats = reassembler.stats();
    assert!(stats.evictions >= 1);
    assert!(reassembler.total_memory() <= config.memcap);
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --test reassembly_engine_tests`
Expected: All tests pass including 2 new ones.

- [ ] **Step 4: Commit**

```bash
git add tests/reassembly_engine_tests.rs
git commit -m "test: add max_flows and memcap eviction tests"
```

---

### Task 4: Add Anomaly Finding and Max Segments Tests

**Files:**
- Modify: `tests/reassembly_engine_tests.rs`

**Context:** `findings()` returns `&[Finding]`. The `Finding` struct has `summary: String`, `category: ThreatCategory`, `confidence: Confidence`. These are in `wirerust::findings`.

- [ ] **Step 1: Add findings imports**

At the top of the file, add:
```rust
use wirerust::findings::{Confidence, ThreatCategory};
```

- [ ] **Step 2: Write `test_overlap_anomaly_finding`**

Need 52 packets total: 1 SYN + 1 original + 50 duplicates = overlap_count 50 (not > 50). Need 51 duplicates = overlap_count 51 > 50. So 53 packets: 1 SYN + 1 original + 51 duplicates.

```rust
#[test]
fn test_overlap_anomaly_finding() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN
    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false, false);
    reassembler.process_packet(&syn, 1, &mut handler);

    // Original segment
    let original = make_tcp_packet(client, 12345, server, 80, 1001, b"AAAA", false, false, false, false);
    reassembler.process_packet(&original, 2, &mut handler);

    // No findings yet
    assert!(reassembler.findings().is_empty());

    // Send 51 duplicates to reach overlap_count=51 (> threshold of 50)
    for i in 0..51u32 {
        let dup = make_tcp_packet(client, 12345, server, 80, 1001, b"AAAA", false, false, false, false);
        reassembler.process_packet(&dup, 3 + i, &mut handler);
    }

    // Overlap anomaly finding should be generated
    let findings = reassembler.findings();
    assert!(!findings.is_empty(), "expected overlap anomaly finding");
    let overlap_finding = findings
        .iter()
        .find(|f| f.summary.contains("Excessive segment overlaps"))
        .expect("overlap anomaly finding not found");
    assert_eq!(overlap_finding.category, ThreatCategory::Anomaly);
}
```

- [ ] **Step 3: Write `test_conflicting_overlap_finding`**

```rust
#[test]
fn test_conflicting_overlap_finding() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN
    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false, false);
    reassembler.process_packet(&syn, 1, &mut handler);

    // Original segment
    let original = make_tcp_packet(client, 12345, server, 80, 1001, b"AAAA", false, false, false, false);
    reassembler.process_packet(&original, 2, &mut handler);

    // Conflicting retransmission: same offset, different data
    let conflict = make_tcp_packet(client, 12345, server, 80, 1001, b"BBBB", false, false, false, false);
    reassembler.process_packet(&conflict, 3, &mut handler);

    // Conflicting overlap finding should be generated
    let findings = reassembler.findings();
    let conflict_finding = findings
        .iter()
        .find(|f| f.summary.contains("Conflicting TCP segment overlap"))
        .expect("conflicting overlap finding not found");
    assert_eq!(conflict_finding.category, ThreatCategory::Anomaly);
    assert_eq!(conflict_finding.confidence, Confidence::High);
}
```

- [ ] **Step 4: Write `test_max_segments_per_direction`**

```rust
#[test]
fn test_max_segments_per_direction() {
    let config = ReassemblyConfig {
        max_segments_per_direction: 5,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN
    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false, false);
    reassembler.process_packet(&syn, 1, &mut handler);

    // 5 non-contiguous segments (skip offset 1 to prevent flush)
    // Offsets: 2, 4, 6, 8, 10 — each 1 byte with gaps between
    for i in 0..5u32 {
        let seq = 1002 + (i * 2);
        let pkt = make_tcp_packet(client, 12345, server, 80, seq, b"x", false, false, false, false);
        reassembler.process_packet(&pkt, 2 + i, &mut handler);
    }

    let stats_before = reassembler.stats().segments_inserted;

    // 6th non-contiguous segment — should be rejected (DepthExceeded)
    let rejected = make_tcp_packet(client, 12345, server, 80, 1012, b"y", false, false, false, false);
    reassembler.process_packet(&rejected, 7, &mut handler);

    // segments_inserted should not have increased
    assert_eq!(reassembler.stats().segments_inserted, stats_before);

    // Fill the gap at offset 1 — triggers flush of contiguous segments
    let fill = make_tcp_packet(client, 12345, server, 80, 1001, b"Z", false, false, false, false);
    reassembler.process_packet(&fill, 8, &mut handler);

    // Existing segments should flush intact (offset 1 "Z" + offset 2 "x")
    assert!(
        handler
            .data_events
            .iter()
            .any(|(_, _, data, _)| data == b"Z"),
        "gap-fill segment should flush"
    );
    assert!(
        handler
            .data_events
            .iter()
            .any(|(_, _, data, _)| data == b"x"),
        "existing buffered segment should flush intact"
    );
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test reassembly_engine_tests`
Expected: All tests pass including 3 new ones.

- [ ] **Step 6: Commit**

```bash
git add tests/reassembly_engine_tests.rs
git commit -m "test: add anomaly finding and max_segments tests"
```
