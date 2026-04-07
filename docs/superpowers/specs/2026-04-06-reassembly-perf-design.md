# TCP Reassembly Performance Optimization Design

**Issue:** #11 — perf: use BTreeMap::range() and incremental memory tracking in reassembly
**Scope:** Three targeted O(n) → O(1)/O(log n) optimizations in the reassembly engine internals. No behavioral changes. Adds `total_memory()` public accessor for testability.

## Problem

Before these optimizations, the TCP reassembly engine had three O(n) hotspots identified during PR #10 review:

1. **Overlap detection** (`segment.rs:85`): iterated all segments in the BTreeMap per insert, even though only segments starting before `new_end` could overlap.
2. **`buffered_bytes` computation** (`segment.rs:62`): recomputed `segments.values().map(|v| v.len()).sum()` on every insert — O(n) per call.
3. **`update_memory()` recomputation** (`mod.rs:391`): iterated all flows × all segments per direction to sum total memory — O(m × n). Called at 5 sites per packet (lines 178, 317, 355, 376, 434).

With `max_segments_per_direction = 10,000` and `max_flows = 100,000`, these costs were bounded but wasteful.

## Approach

Incremental counters + BTreeMap range query. All three optimizations are independent and can be implemented/tested separately.

## Changes

### 1. `FlowDirection.buffered_bytes` — Incremental Counter (`flow.rs`)

Add `pub buffered_bytes: usize` field to `FlowDirection`, initialized to 0.

Update at every `BTreeMap::insert` and `BTreeMap::remove` site:

**On insert** (both `segment.rs:155` gap insert and `segment.rs:171` normal insert):
```rust
let data_len = segment_data.len();
if let Some(old) = dir.segments.insert(offset, segment_data) {
    dir.buffered_bytes -= old.len();
}
dir.buffered_bytes += data_len;
```

Capturing `data_len` before `insert()` is required because `segment_data` is moved into the BTreeMap.

The `if let Some(old)` handles the theoretical edge case where `insert()` replaces an existing key. In practice, the overlap detection prevents this for the normal path (line 171), and gap computation prevents it for the gap path (line 155). The guard is defensive.

**On flush** (`segment.rs:185`, inside `flush_contiguous`):
```rust
while let Some(data) = dir.segments.remove(&dir.base_offset) {
    dir.buffered_bytes -= data.len();
    // ... existing logic ...
}
```

**Replace `memory_used()`** (`flow.rs:107`):
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

The `debug_assert_eq!` verifies the counter in debug/test builds (O(n) check). Compiled out in release.

**Depth check** (`segment.rs:62`): Replace O(n) summation with field access:
```rust
// Before:
let buffered: usize = dir.segments.values().map(|v| v.len()).sum();

// After:
let buffered = dir.buffered_bytes;
```

### 2. `BTreeMap::range()` for Overlap Detection (`segment.rs`)

Replace full iteration at line 85:
```rust
// Before (O(n) — iterates all segments):
for (&existing_offset, existing_data) in dir.segments.iter() {

// After (O(log n + k) — only segments that could overlap):
for (&existing_offset, existing_data) in dir.segments.range(..new_end) {
```

**Why this is correct:** A segment at `existing_offset` overlaps `[new_start, new_end)` only if `existing_offset < new_end AND new_start < existing_end`. The first condition is enforced by `range(..new_end)`. The second condition is checked inside the loop (unchanged). Segments with `existing_offset >= new_end` cannot overlap regardless of their length, because the overlap condition requires `existing_offset < new_end`.

**No lower bound needed:** We could theoretically skip segments whose end is before `new_start`, but since segment length varies and the BTreeMap is keyed by start offset (not end), a lower bound would require knowing max segment size. The `range(..new_end)` upper bound is sufficient — the inner `new_start < existing_end` check handles the rest.

### 3. Incremental `total_memory` Tracking (`mod.rs`)

Remove `fn update_memory()` entirely. Replace all 5 call sites with inline delta computation.

**Pattern for insert sites:**
```rust
let before = flow_dir.buffered_bytes;
let result = insert_segment(flow_dir, seq, payload, ...);
let delta = flow_dir.buffered_bytes - before;
self.total_memory += delta;
```

**Pattern for flush sites:**
```rust
let before = flow_dir.buffered_bytes;
let flushed = flush_contiguous(flow_dir);
self.total_memory -= before - flow_dir.buffered_bytes;
```

**Pattern for flow removal:**
```rust
if let Some(flow) = self.flows.remove(&key) {
    self.total_memory -= flow.memory_used();
}
```

`flow.memory_used()` is now O(1) — just `client_to_server.buffered_bytes + server_to_client.buffered_bytes`.

**Call site mapping:**

| Line | Context | Replacement |
|------|---------|-------------|
| 178 | After RST removal | `self.total_memory -= flow.memory_used()` before remove |
| 317 | After payload processing | Delta after insert + delta after flush |
| 355 | After `expire_flows` | Subtract removed flow memory |
| 376 | After `finalize` | Subtract removed flow memory |
| 434 | Inside `evict_flows` loop | Subtract removed flow memory |

## Files Modified

| File | Change |
|------|--------|
| `src/reassembly/flow.rs` | Add `buffered_bytes: usize` field, update `memory_used()` |
| `src/reassembly/segment.rs` | Use `range(..new_end)`, update `buffered_bytes` on insert/flush, replace O(n) sum |
| `src/reassembly/mod.rs` | Remove `update_memory()`, add inline delta tracking at all mutation sites |

## Testing

**Unit tests (new):**
1. `buffered_bytes` correct after single insert
2. `buffered_bytes` correct after insert + flush sequence
3. `buffered_bytes` correct after overlapping inserts (gap portions only)
4. `buffered_bytes` correct after replacement insert (defensive case)
5. `total_memory` delta correct after insert
6. `total_memory` correct after flow removal
7. Range-based overlap detection finds all overlapping segments
8. Range-based overlap detection skips non-overlapping segments

**Existing tests:** All must pass unchanged. The `debug_assert_eq!` in `memory_used()` acts as a cross-check in all test runs.

**No integration test changes** — behavior is identical, only performance characteristics change.

## Not In Scope

- **Per-flow cached memory** — `FlowDirection.buffered_bytes` makes `TcpFlow.memory_used()` O(1) already
- **Lower bound on range query** — diminishing returns, inner check handles it
- **Benchmarks** — can be added in a follow-up issue
- **Structural refactoring** — keeping changes minimal and targeted
