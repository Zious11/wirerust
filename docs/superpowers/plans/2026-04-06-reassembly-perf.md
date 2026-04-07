# TCP Reassembly Performance Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace three O(n) hotspots in the TCP reassembly engine with O(1)/O(log n) operations using incremental counters and BTreeMap range queries.

**Architecture:** Add `buffered_bytes: usize` field to `FlowDirection` for incremental byte tracking. Replace `segments.iter()` with `segments.range(..new_end)` for overlap detection. Remove `update_memory()` and track `total_memory` via deltas at each mutation site.

**Tech Stack:** Rust std `BTreeMap::range()`, no new dependencies.

---

## File Structure

| File | Responsibility | Changes |
|------|---------------|---------|
| `src/reassembly/flow.rs` | Flow/direction data structures | Add `buffered_bytes` field, update `memory_used()` |
| `src/reassembly/segment.rs` | Segment insert/flush operations | Track `buffered_bytes` on insert/flush, use `range()`, replace O(n) sum |
| `src/reassembly/mod.rs` | Reassembly engine orchestration | Remove `update_memory()`, add inline delta tracking |
| `tests/reassembly_segment_tests.rs` | Segment operation tests | Add `buffered_bytes` assertions to existing + new tests |
| `tests/reassembly_flow_tests.rs` | Flow structure tests | Add `buffered_bytes` initialization assertion |
| `tests/reassembly_engine_tests.rs` | Engine integration tests | Add `total_memory` tracking test |

---

### Task 1: Add `buffered_bytes` field and track on segment insert

**Files:**
- Modify: `src/reassembly/flow.rs:56-91`
- Modify: `src/reassembly/segment.rs:145-178`
- Modify: `tests/reassembly_segment_tests.rs`
- Modify: `tests/reassembly_flow_tests.rs`

- [ ] **Step 1: Add `buffered_bytes` field to `FlowDirection`**

In `src/reassembly/flow.rs`, add the field to the struct (after `segments` on line 59):

```rust
pub struct FlowDirection {
    pub isn: Option<u32>,
    pub base_offset: u64,
    pub segments: BTreeMap<u64, Vec<u8>>,
    pub buffered_bytes: usize,
    pub reassembled_bytes: usize,
    pub overlap_count: u32,
    pub overlap_alert_fired: bool,
    pub small_segment_count: u32,
    pub small_segment_alert_fired: bool,
    pub fin_seen: bool,
    pub rst_seen: bool,
    pub depth_exceeded: bool,
}
```

Initialize to 0 in `FlowDirection::new()`:

```rust
pub fn new() -> Self {
    FlowDirection {
        isn: None,
        base_offset: 0,
        segments: BTreeMap::new(),
        buffered_bytes: 0,
        reassembled_bytes: 0,
        overlap_count: 0,
        overlap_alert_fired: false,
        small_segment_count: 0,
        small_segment_alert_fired: false,
        fin_seen: false,
        rst_seen: false,
        depth_exceeded: false,
    }
}
```

- [ ] **Step 2: Add `buffered_bytes` assertion to existing flow init test**

In `tests/reassembly_flow_tests.rs`, add this assertion to `test_flow_direction_new`:

```rust
#[test]
fn test_flow_direction_new() {
    let dir = FlowDirection::new();
    assert_eq!(dir.isn, None);
    assert_eq!(dir.base_offset, 0);
    assert!(dir.segments.is_empty());
    assert_eq!(dir.buffered_bytes, 0);
    assert_eq!(dir.reassembled_bytes, 0);
    assert!(!dir.fin_seen);
    assert!(!dir.rst_seen);
    assert!(!dir.depth_exceeded);
}
```

- [ ] **Step 3: Write failing test for `buffered_bytes` after insert**

In `tests/reassembly_segment_tests.rs`, add:

```rust
#[test]
fn test_buffered_bytes_after_insert() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    insert_segment(&mut dir, 1001, b"hello", 10_485_760, 10_000);
    assert_eq!(dir.buffered_bytes, 5);

    insert_segment(&mut dir, 1006, b"world", 10_485_760, 10_000);
    assert_eq!(dir.buffered_bytes, 10);
}
```

- [ ] **Step 4: Run test to verify it fails**

Run: `cargo test test_buffered_bytes_after_insert -- --nocapture`
Expected: FAIL — `buffered_bytes` is 0 (not updated yet).

- [ ] **Step 5: Write failing test for `buffered_bytes` after overlapping insert**

In `tests/reassembly_segment_tests.rs`, add:

```rust
#[test]
fn test_buffered_bytes_after_overlap() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert "AAABBB" at offset 1 (6 bytes)
    insert_segment(&mut dir, 1001, b"AAABBB", 10_485_760, 10_000);
    assert_eq!(dir.buffered_bytes, 6);

    // Overlapping insert: "XXXCC" at offset 4 — only "CC" (2 bytes) is new
    insert_segment(&mut dir, 1004, b"XXXCC", 10_485_760, 10_000);
    assert_eq!(dir.buffered_bytes, 8); // 6 original + 2 gap bytes
}
```

- [ ] **Step 6: Update `insert_segment` to track `buffered_bytes`**

In `src/reassembly/segment.rs`, modify the normal insert at line 170-171:

```rust
    // No overlap — insert normally
    let data_len = segment_data.len();
    if let Some(old) = dir.segments.insert(offset, segment_data) {
        dir.buffered_bytes -= old.len();
    }
    dir.buffered_bytes += data_len;
```

And the gap insert at line 153-157, replace:

```rust
                let gap_data = segment_data[start_idx..end_idx].to_vec();
                if !gap_data.is_empty() {
                    dir.segments.insert(gap_start, gap_data);
                }
```

with:

```rust
                let gap_data = segment_data[start_idx..end_idx].to_vec();
                if !gap_data.is_empty() {
                    let gap_len = gap_data.len();
                    if let Some(old) = dir.segments.insert(gap_start, gap_data) {
                        dir.buffered_bytes -= old.len();
                    }
                    dir.buffered_bytes += gap_len;
                }
```

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test test_buffered_bytes -- --nocapture`
Expected: PASS for both `test_buffered_bytes_after_insert` and `test_buffered_bytes_after_overlap`.

- [ ] **Step 8: Run all existing segment tests**

Run: `cargo test reassembly_segment -- --nocapture`
Expected: All existing tests PASS (behavior unchanged).

- [ ] **Step 9: Commit**

```bash
git add src/reassembly/flow.rs src/reassembly/segment.rs tests/reassembly_segment_tests.rs tests/reassembly_flow_tests.rs
git commit -m "perf: add buffered_bytes field and track on segment insert"
```

---

### Task 2: Track `buffered_bytes` on flush and update `memory_used()`

**Files:**
- Modify: `src/reassembly/segment.rs:180-193`
- Modify: `src/reassembly/flow.rs:107-109`
- Modify: `tests/reassembly_segment_tests.rs`

- [ ] **Step 1: Write failing test for `buffered_bytes` after insert + flush**

In `tests/reassembly_segment_tests.rs`, add:

```rust
#[test]
fn test_buffered_bytes_after_flush() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    insert_segment(&mut dir, 1001, b"hello", 10_485_760, 10_000);
    assert_eq!(dir.buffered_bytes, 5);

    let flushed = flush_contiguous(&mut dir);
    assert_eq!(flushed.len(), 1);
    assert_eq!(dir.buffered_bytes, 0);
}

#[test]
fn test_buffered_bytes_partial_flush() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert segment at offset 1 (contiguous) and offset 10 (gap)
    insert_segment(&mut dir, 1001, b"aaa", 10_485_760, 10_000);
    insert_segment(&mut dir, 1010, b"bbb", 10_485_760, 10_000);
    assert_eq!(dir.buffered_bytes, 6);

    // Flush only flushes contiguous segment at offset 1
    let flushed = flush_contiguous(&mut dir);
    assert_eq!(flushed.len(), 1);
    assert_eq!(flushed[0].1, b"aaa");
    assert_eq!(dir.buffered_bytes, 3); // "bbb" remains buffered
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_buffered_bytes_after_flush test_buffered_bytes_partial_flush -- --nocapture`
Expected: FAIL — `flush_contiguous` doesn't update `buffered_bytes` yet.

- [ ] **Step 3: Update `flush_contiguous` to decrement `buffered_bytes`**

In `src/reassembly/segment.rs`, modify `flush_contiguous`:

```rust
pub fn flush_contiguous(dir: &mut FlowDirection) -> Vec<(u64, Vec<u8>)> {
    let mut flushed = Vec::new();

    while let Some(data) = dir.segments.remove(&dir.base_offset) {
        let offset = dir.base_offset;
        dir.buffered_bytes -= data.len();
        dir.base_offset += data.len() as u64;
        dir.reassembled_bytes += data.len();
        flushed.push((offset, data));
    }

    flushed
}
```

- [ ] **Step 4: Run flush tests to verify they pass**

Run: `cargo test test_buffered_bytes_after_flush test_buffered_bytes_partial_flush -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Update `memory_used()` with debug assertion**

In `src/reassembly/flow.rs`, replace `memory_used()`:

```rust
    pub fn memory_used(&self) -> usize {
        debug_assert_eq!(
            self.buffered_bytes,
            self.segments.values().map(|v| v.len()).sum::<usize>(),
            "buffered_bytes counter drifted from actual segment sizes"
        );
        self.buffered_bytes
    }
```

- [ ] **Step 6: Run all tests to verify debug_assert doesn't fire**

Run: `cargo test -- --nocapture`
Expected: All tests PASS. The `debug_assert_eq!` verifies `buffered_bytes` matches the recomputed sum on every `memory_used()` call during tests.

- [ ] **Step 7: Commit**

```bash
git add src/reassembly/segment.rs src/reassembly/flow.rs tests/reassembly_segment_tests.rs
git commit -m "perf: track buffered_bytes on flush and add debug assertion in memory_used()"
```

---

### Task 3: Replace O(n) buffered sum with field access and use `range()` for overlap detection

**Files:**
- Modify: `src/reassembly/segment.rs:62,85`
- Modify: `tests/reassembly_segment_tests.rs`

- [ ] **Step 1: Write test for range-based overlap boundary**

In `tests/reassembly_segment_tests.rs`, add:

```rust
#[test]
fn test_overlap_detection_boundary() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert segment at offset 1, length 5 (covers 1-5)
    insert_segment(&mut dir, 1001, b"AAAAA", 10_485_760, 10_000);
    // Insert segment at offset 10, length 5 (covers 10-14) — no overlap with above
    insert_segment(&mut dir, 1010, b"BBBBB", 10_485_760, 10_000);
    assert_eq!(dir.segments.len(), 2);
    assert_eq!(dir.overlap_count, 0);

    // Insert segment at offset 3, length 4 (covers 3-6) — overlaps first, not second
    let result = insert_segment(&mut dir, 1003, b"XXXX", 10_485_760, 10_000);
    assert_eq!(result, InsertResult::PartialOverlap);
    assert_eq!(dir.overlap_count, 1);

    // Insert segment at offset 6, length 4 (covers 6-9) — no overlap with either
    let result = insert_segment(&mut dir, 1006, b"CCCC", 10_485_760, 10_000);
    assert_eq!(result, InsertResult::Inserted);
    assert_eq!(dir.overlap_count, 1); // unchanged
}
```

- [ ] **Step 2: Run test to verify it passes (establishes baseline)**

Run: `cargo test test_overlap_detection_boundary -- --nocapture`
Expected: PASS (current O(n) logic produces correct results).

- [ ] **Step 3: Replace O(n) buffered sum with field access**

In `src/reassembly/segment.rs`, line 62, replace:

```rust
    let buffered: usize = dir.segments.values().map(|v| v.len()).sum();
```

with:

```rust
    let buffered = dir.buffered_bytes;
```

- [ ] **Step 4: Replace `iter()` with `range(..new_end)` for overlap detection**

In `src/reassembly/segment.rs`, line 85, replace:

```rust
    for (&existing_offset, existing_data) in dir.segments.iter() {
```

with:

```rust
    for (&existing_offset, existing_data) in dir.segments.range(..new_end) {
```

- [ ] **Step 5: Run all segment tests**

Run: `cargo test reassembly_segment -- --nocapture`
Expected: All PASS — including `test_overlap_detection_boundary`, `test_overlap_first_wins`, `test_overlap_conflicting_data_detected`, `test_retransmission_dedup`.

- [ ] **Step 6: Run full test suite**

Run: `cargo test -- --nocapture`
Expected: All PASS.

- [ ] **Step 7: Commit**

```bash
git add src/reassembly/segment.rs tests/reassembly_segment_tests.rs
git commit -m "perf: use BTreeMap::range() for overlap detection and replace O(n) buffered sum"
```

---

### Task 4: Remove `update_memory()` and add incremental `total_memory` tracking

**Files:**
- Modify: `src/reassembly/mod.rs`
- Modify: `tests/reassembly_engine_tests.rs`

This is the largest task. It replaces 5 `update_memory()` call sites with inline delta tracking. The key patterns are:

**For insert+flush (no removal):** Track deltas before/after each operation.
**For flush+remove paths (RST, FIN, expire, finalize, evict):** Capture flow's `memory_used()` before flushing, subtract that amount on removal. This is simpler than tracking individual flush deltas because the entire flow is about to be destroyed.

- [ ] **Step 1: Write test verifying `total_memory` tracking**

In `tests/reassembly_engine_tests.rs`, add:

```rust
#[test]
fn test_total_memory_tracking() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN — no payload, no memory change
    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false);
    reassembler.process_packet(&syn, 1, &mut handler);

    // Out-of-order segment — buffered (not flushed)
    let p2 = make_tcp_packet(client, 12345, server, 80, 1004, b"bbb", false, false, false);
    reassembler.process_packet(&p2, 2, &mut handler);
    // "bbb" is buffered (3 bytes) because offset 1 is missing
    assert!(handler.data_events.is_empty()); // nothing flushed yet

    // In-order segment — triggers flush of both
    let p1 = make_tcp_packet(client, 12345, server, 80, 1001, b"aaa", false, false, false);
    reassembler.process_packet(&p1, 3, &mut handler);
    // Both segments flushed — total_memory should be 0
    assert_eq!(handler.all_data(), b"aaabbb");

    // Finalize — closes flow, should leave total_memory at 0
    reassembler.finalize(&mut handler);
}
```

Note: We cannot directly assert `total_memory` because it is a private field. The test verifies correct behavior indirectly — the `debug_assert_eq!` inside `memory_used()` (called during finalize's flow removal) will panic if counters have drifted. If this test passes, incremental tracking is correct.

- [ ] **Step 2: Run test to verify it passes with current `update_memory()`**

Run: `cargo test test_total_memory_tracking -- --nocapture`
Expected: PASS (baseline with current O(n) recomputation).

- [ ] **Step 3: Update payload processing path (process_packet lines 211-291)**

In `src/reassembly/mod.rs`, replace the insert + flush section. Find the block starting around line 211:

```rust
            let flow_dir = flow.get_direction_mut(dir);
            let result = insert_segment(
                flow_dir,
                seq,
                payload,
                self.config.max_depth,
                self.config.max_segments_per_direction,
            );
```

Replace with:

```rust
            let flow_dir = flow.get_direction_mut(dir);
            let before_insert = flow_dir.buffered_bytes;
            let result = insert_segment(
                flow_dir,
                seq,
                payload,
                self.config.max_depth,
                self.config.max_segments_per_direction,
            );
            self.total_memory += flow_dir.buffered_bytes - before_insert;
```

Then find the flush section around line 283:

```rust
            let flow = self.flows.get_mut(&key).unwrap();
            let flow_dir = flow.get_direction_mut(dir);
            let flushed = flush_contiguous(flow_dir);

            for (offset, data) in &flushed {
                self.stats.bytes_reassembled += data.len() as u64;
                handler.on_data(&key, dir, data, *offset);
            }
```

Replace with:

```rust
            let flow = self.flows.get_mut(&key).unwrap();
            let flow_dir = flow.get_direction_mut(dir);
            let before_flush = flow_dir.buffered_bytes;
            let flushed = flush_contiguous(flow_dir);
            self.total_memory -= before_flush - flow_dir.buffered_bytes;

            for (offset, data) in &flushed {
                self.stats.bytes_reassembled += data.len() as u64;
                handler.on_data(&key, dir, data, *offset);
            }
```

Remove the `self.update_memory();` call at line 317 (after the payload section).

- [ ] **Step 4: Update RST handler (process_packet lines 159-179)**

Find the RST handler block:

```rust
        if rst {
            flow.on_rst();
            self.stats.flows_rst += 1;
            let key_clone = key.clone();
            // Flush buffered contiguous data before removing
            if let Some(flow) = self.flows.get_mut(&key_clone) {
                use crate::reassembly::handler::Direction;
                for dir in [Direction::ClientToServer, Direction::ServerToClient] {
                    let flow_dir = flow.get_direction_mut(dir);
                    let flushed = flush_contiguous(flow_dir);
                    for (offset, data) in &flushed {
                        self.stats.bytes_reassembled += data.len() as u64;
                        handler.on_data(&key_clone, dir, data, *offset);
                    }
                }
            }
            handler.on_flow_close(&key_clone, CloseReason::Rst);
            self.flows.remove(&key_clone);
            self.update_memory();
            return;
        }
```

Replace with:

```rust
        if rst {
            flow.on_rst();
            self.stats.flows_rst += 1;
            let key_clone = key.clone();
            let flow_mem = self.flows.get(&key_clone).map(|f| f.memory_used()).unwrap_or(0);
            // Flush buffered contiguous data before removing
            if let Some(flow) = self.flows.get_mut(&key_clone) {
                use crate::reassembly::handler::Direction;
                for dir in [Direction::ClientToServer, Direction::ServerToClient] {
                    let flow_dir = flow.get_direction_mut(dir);
                    let flushed = flush_contiguous(flow_dir);
                    for (offset, data) in &flushed {
                        self.stats.bytes_reassembled += data.len() as u64;
                        handler.on_data(&key_clone, dir, data, *offset);
                    }
                }
            }
            handler.on_flow_close(&key_clone, CloseReason::Rst);
            self.flows.remove(&key_clone);
            self.total_memory -= flow_mem;
            return;
        }
```

Pattern: capture `memory_used()` before flushing, subtract on removal. This correctly accounts for both flushed bytes and any remaining non-contiguous bytes.

- [ ] **Step 5: Update FIN-closed flow removal (process_packet lines 294-314)**

Find the FIN closure block:

```rust
        if self
            .flows
            .get(&key)
            .is_some_and(|f| f.state == FlowState::Closed)
        {
            // Flush remaining data in both directions before removal
            if let Some(flow) = self.flows.get_mut(&key) {
                use crate::reassembly::handler::Direction;
                for dir in [Direction::ClientToServer, Direction::ServerToClient] {
                    let flow_dir = flow.get_direction_mut(dir);
                    let flushed = flush_contiguous(flow_dir);
                    for (offset, data) in &flushed {
                        self.stats.bytes_reassembled += data.len() as u64;
                        handler.on_data(&key, dir, data, *offset);
                    }
                }
            }
            self.stats.flows_fin += 1;
            handler.on_flow_close(&key, CloseReason::Fin);
            self.flows.remove(&key);
        }
```

Replace with:

```rust
        if self
            .flows
            .get(&key)
            .is_some_and(|f| f.state == FlowState::Closed)
        {
            let flow_mem = self.flows.get(&key).map(|f| f.memory_used()).unwrap_or(0);
            // Flush remaining data in both directions before removal
            if let Some(flow) = self.flows.get_mut(&key) {
                use crate::reassembly::handler::Direction;
                for dir in [Direction::ClientToServer, Direction::ServerToClient] {
                    let flow_dir = flow.get_direction_mut(dir);
                    let flushed = flush_contiguous(flow_dir);
                    for (offset, data) in &flushed {
                        self.stats.bytes_reassembled += data.len() as u64;
                        handler.on_data(&key, dir, data, *offset);
                    }
                }
            }
            self.stats.flows_fin += 1;
            handler.on_flow_close(&key, CloseReason::Fin);
            self.flows.remove(&key);
            self.total_memory -= flow_mem;
        }
```

- [ ] **Step 6: Update `expire_flows` (lines 326-356)**

Find `expire_flows`:

```rust
        for key in expired_keys {
            // Flush salvageable data before removing
            if let Some(flow) = self.flows.get_mut(&key) {
                use crate::reassembly::handler::Direction;
                for dir in [Direction::ClientToServer, Direction::ServerToClient] {
                    let flow_dir = flow.get_direction_mut(dir);
                    let flushed = flush_contiguous(flow_dir);
                    for (offset, data) in &flushed {
                        handler.on_data(&key, dir, data, *offset);
                    }
                }
            }
            self.flows.remove(&key);
            self.stats.flows_expired += 1;
            handler.on_flow_close(&key, CloseReason::Timeout);
        }

        self.update_memory();
```

Replace with:

```rust
        for key in expired_keys {
            let flow_mem = self.flows.get(&key).map(|f| f.memory_used()).unwrap_or(0);
            // Flush salvageable data before removing
            if let Some(flow) = self.flows.get_mut(&key) {
                use crate::reassembly::handler::Direction;
                for dir in [Direction::ClientToServer, Direction::ServerToClient] {
                    let flow_dir = flow.get_direction_mut(dir);
                    let flushed = flush_contiguous(flow_dir);
                    for (offset, data) in &flushed {
                        handler.on_data(&key, dir, data, *offset);
                    }
                }
            }
            self.flows.remove(&key);
            self.total_memory -= flow_mem;
            self.stats.flows_expired += 1;
            handler.on_flow_close(&key, CloseReason::Timeout);
        }
```

- [ ] **Step 7: Update `finalize` (lines 358-377)**

Find `finalize`:

```rust
        let all_keys: Vec<FlowKey> = self.flows.keys().cloned().collect();
        for key in all_keys {
            // Flush any remaining contiguous data before closing
            if let Some(flow) = self.flows.get_mut(&key) {
                for dir in [Direction::ClientToServer, Direction::ServerToClient] {
                    let flow_dir = flow.get_direction_mut(dir);
                    let flushed = flush_contiguous(flow_dir);
                    for (offset, data) in &flushed {
                        handler.on_data(&key, dir, data, *offset);
                    }
                }
            }
            self.flows.remove(&key);
            handler.on_flow_close(&key, CloseReason::Timeout);
        }
        self.update_memory();
```

Replace with:

```rust
        let all_keys: Vec<FlowKey> = self.flows.keys().cloned().collect();
        for key in all_keys {
            let flow_mem = self.flows.get(&key).map(|f| f.memory_used()).unwrap_or(0);
            // Flush any remaining contiguous data before closing
            if let Some(flow) = self.flows.get_mut(&key) {
                for dir in [Direction::ClientToServer, Direction::ServerToClient] {
                    let flow_dir = flow.get_direction_mut(dir);
                    let flushed = flush_contiguous(flow_dir);
                    for (offset, data) in &flushed {
                        handler.on_data(&key, dir, data, *offset);
                    }
                }
            }
            self.flows.remove(&key);
            self.total_memory -= flow_mem;
            handler.on_flow_close(&key, CloseReason::Timeout);
        }
```

- [ ] **Step 8: Update `evict_flows` (lines 398-436)**

Find the eviction loop:

```rust
        for (key, _, _) in &candidates {
            if self.total_memory <= self.config.memcap && self.flows.len() <= self.config.max_flows
            {
                break;
            }
            // Flush salvageable contiguous data before evicting
            if let Some(flow) = self.flows.get_mut(key) {
                use crate::reassembly::handler::Direction;
                for dir in [Direction::ClientToServer, Direction::ServerToClient] {
                    let flow_dir = flow.get_direction_mut(dir);
                    let flushed = flush_contiguous(flow_dir);
                    for (offset, data) in &flushed {
                        handler.on_data(key, dir, data, *offset);
                    }
                }
            }
            self.flows.remove(key);
            self.stats.evictions += 1;
            handler.on_flow_close(key, CloseReason::MemoryPressure);
            self.update_memory();
        }
```

Replace with:

```rust
        for (key, _, _) in &candidates {
            if self.total_memory <= self.config.memcap && self.flows.len() <= self.config.max_flows
            {
                break;
            }
            let flow_mem = self.flows.get(key).map(|f| f.memory_used()).unwrap_or(0);
            // Flush salvageable contiguous data before evicting
            if let Some(flow) = self.flows.get_mut(key) {
                use crate::reassembly::handler::Direction;
                for dir in [Direction::ClientToServer, Direction::ServerToClient] {
                    let flow_dir = flow.get_direction_mut(dir);
                    let flushed = flush_contiguous(flow_dir);
                    for (offset, data) in &flushed {
                        handler.on_data(key, dir, data, *offset);
                    }
                }
            }
            self.flows.remove(key);
            self.total_memory -= flow_mem;
            self.stats.evictions += 1;
            handler.on_flow_close(key, CloseReason::MemoryPressure);
        }
```

- [ ] **Step 9: Remove `update_memory()` function**

In `src/reassembly/mod.rs`, delete the `update_memory` function entirely:

```rust
    // DELETE THIS FUNCTION:
    fn update_memory(&mut self) {
        self.total_memory = self.flows.values().map(|f| f.memory_used()).sum();
    }
```

- [ ] **Step 10: Run full test suite**

Run: `cargo test -- --nocapture`
Expected: All tests PASS — including existing engine tests (`test_three_packet_stream_ordered`, `test_out_of_order_delivery`, `test_rst_closes_flow`, `test_finalize_flushes_remaining`, `test_flow_timeout_expiration`) and the new `test_total_memory_tracking`.

The `debug_assert_eq!` in `memory_used()` fires on every flow removal during tests, verifying that incremental `buffered_bytes` matches the recomputed sum.

- [ ] **Step 11: Run clippy and fmt**

Run: `cargo clippy -- -D warnings && cargo fmt --check`
Expected: No warnings, no formatting issues.

- [ ] **Step 12: Commit**

```bash
git add src/reassembly/mod.rs tests/reassembly_engine_tests.rs
git commit -m "perf: remove update_memory() and track total_memory incrementally"
```
