---
artifact: verification-property
vp_id: VP-040
title: "TLS Per-Direction Buffer Saturation Observability"
version: "1.1"
status: draft
phase: P1
tool: unit
subsystem: SS-07
module: "src/analyzer/tls.rs"
producer: architect
timestamp: 2026-06-29T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-011-tls-handshake-reassembly.md
feature_cycle: fix-tls-clienthello-frag
issue: fix-tls-clienthello-frag
bcs:
  - BC-2.07.043
  - BC-2.07.005
verification_lock: false
---

# VP-040: TLS Per-Direction Buffer Saturation Observability

## Property Statement

BC-2.07.043 introduces `buffer_saturation_drops: u64` — an aggregate counter on
`TlsAnalyzer` that increments by exactly 1 each time `on_data`'s buffer-append
discards bytes due to the `MAX_BUF` (65,536-byte) capacity cap. Specifically, the
drop path fires when:

- `data.len() > remaining` — this single condition covers BOTH sub-cases:
  - **Partial drop** (`remaining > 0` but `remaining < data.len()`): `to_copy` bytes
    are appended, the rest are dropped.
  - **Full drop** (`remaining == 0`): the `if remaining > 0` guard causes the entire
    `data` slice to be skipped; `to_copy` is never computed in this arm.

NOTE: the increment condition is `data.len() > remaining`, NOT `to_copy < data.len()`.
`to_copy` is only computed inside the `if remaining > 0` branch — it is undefined when
`remaining == 0`. Using `data.len() > remaining` as the gate covers the full-drop case
(`remaining == 0` implies `data.len() > 0 > remaining` when `data.len() > 0`) as well
as the partial-drop case.

Both the client-direction buffer (`client_buf`) and the server-direction buffer
(`server_buf`) share the same aggregate counter. The counter is NOT per-flow and NOT
reset when a flow is closed (`on_flow_close`). The counter is surfaced via
`summarize()` in the `detail` map under the key `"buffer_saturation_drops"`.

**Increment site (borrow constraint — C-1):** The `buffer_saturation_drops` field is
on `TlsAnalyzer` (`self`), not on `TlsFlowState`. Inside `on_data`, the flow state is
accessed via `let state: &mut TlsFlowState = ...`. Rust forbids mutating `self` (to
increment the counter) while `state: &mut TlsFlowState` borrows from `self.flows`.
Therefore the increment is performed AFTER the `&mut state` block closes, using a
local bool flag (`let mut did_drop = false`) set inside the block, with `if did_drop
{ self.buffer_saturation_drops += 1; }` placed AFTER the block. This is required by
the borrow checker — it is not an implementation choice but a language constraint.

This counter is the F-EV-001 defense-in-depth realization: the pre-existing `MAX_BUF`
tail-drop at `tls.rs:822–825` was previously silent (observable only by the absence of
further analysis). The counter makes the primitive non-silent, enabling operators to
distinguish "TLS flow had no relevant data" from "TLS flow was silently truncated at
the TCP-segment buffer layer."

**Sub-A (BC-2.07.043 — drop increments counter):** When a call to `on_data` discards
bytes because the target buffer has no remaining capacity (or remaining < data.len()),
`buffer_saturation_drops` increments by exactly 1. `parse_errors` is NOT incremented.
`truncated_records` is NOT incremented. No finding is emitted.

**Sub-B (BC-2.07.043 — no drop, counter unchanged):** When a call to `on_data`
delivers data that fits entirely in the available buffer capacity, `buffer_saturation_drops`
is unchanged.

**Sub-C (BC-2.07.043 — counter persists across flows):** The counter is a
`TlsAnalyzer`-level aggregate. After a flow that triggered a drop is closed via
`on_flow_close`, the counter retains its value (NOT reset). Subsequent queries to
`buffer_saturation_drop_count()` return the accumulated value.

**Sub-D (BC-2.07.043 — summarize() exposes key):** `summarize()` includes the key
`"buffer_saturation_drops"` in the detail map with the correct u64 value. This mirrors
the `"truncated_records"` and `"handshake_reassembly_overflows"` surfacing pattern at
`tls.rs:888–889`.

**Sub-E (BC-2.07.043 — both directions increment the same counter):** A drop in the
client-direction buffer (`client_buf`) and a drop in the server-direction buffer
(`server_buf`) both increment the same `buffer_saturation_drops` aggregate counter.
After one client-direction drop followed by one server-direction drop, the counter
is `initial + 2`.

## Purity Boundary

`buffer_saturation_drops` is a `TlsAnalyzer`-level aggregate (effectful shell
boundary). The counter increment logic is a pure arithmetic step (`+= 1`) inside
`on_data`, which operates on the TLS flow state map. The counter itself is not
independently Kani-provable (it is an integer field on a stateful struct with I/O
interaction), but all five sub-properties are deterministically verifiable as unit
tests over the public `TlsAnalyzer` API (Sub-A is split into 2 tests: partial-drop
and full-drop, giving 6 harnesses total).

**Tool choice rationale:** Proptest is not warranted here. The property is entirely
deterministic: trigger a known-size overflow, assert an exact counter delta. Six
targeted unit tests with fixed fixtures are more precise and faster than proptest
for arithmetic-increment properties with exact expected values. This mirrors the
pattern used for `truncated_records` and `handshake_reassembly_overflows` in VP-039
Sub-C unit tests.

## Proof Harness Skeletons

### Sub-A: test_BC_2_07_043_buffer_saturation_observable

This test covers the **partial-drop** path: `remaining > 0` but `data.len() > remaining`
(equivalently `to_copy < data.len()`). A single `on_data` slice of `MAX_BUF + 1` bytes
is sufficient — `remaining == MAX_BUF` (empty buffer), `to_copy == MAX_BUF`, 1 byte
dropped, `data.len() > remaining` is true. No test seam is needed for the partial-drop
case. This is the straightforward falsifiable test.

```rust
#[test]
fn test_BC_2_07_043_buffer_saturation_observable() {
    // Verify: a single on_data delivery larger than available buffer capacity
    // increments buffer_saturation_drops by exactly 1 (partial-drop path).
    //
    // Trigger mechanism (partial drop, no seam required):
    // Deliver a slice of exactly MAX_BUF + 1 bytes (65,537 bytes) to an empty
    // client_buf (remaining == MAX_BUF == 65,536).
    //   - to_copy = min(data.len(), remaining) = 65,536
    //   - data.len() > remaining → 65,537 > 65,536 → TRUE
    //   - 1 byte is dropped; buffer_saturation_drops increments by 1.
    //
    // Increment site: the drop flag is detected AFTER the &mut state block closes
    // (Rust borrow constraint — C-1: self.buffer_saturation_drops cannot be mutated
    // while state: &mut TlsFlowState borrows from self.flows; a local bool `did_drop`
    // is set inside the block, then `if did_drop { self.buffer_saturation_drops += 1; }`
    // fires after the block).
    //
    // No-seam construction: deliver a raw byte slice of length MAX_BUF + 1.
    // Use 0x00-filled content (not a valid TLS record header); on_data will
    // fill client_buf with the first 65,536 bytes and drop the 65,537th byte.
    // parse_errors is NOT incremented on buffer saturation (BC-2.07.043 PC-6).

    let mut analyzer = TlsAnalyzer::new();
    let flow = FlowKey::new(/* ... */);

    let drops_before = analyzer.buffer_saturation_drop_count();
    let parse_errors_before = analyzer.parse_error_count();

    // Deliver a slice of MAX_BUF + 1 bytes to a fresh (empty) buffer.
    // This is a self-contained, falsifiable test — no pre-fill phase needed.
    let oversized = vec![0u8; 65_537]; // MAX_BUF + 1
    analyzer.on_data(&flow, Direction::ClientToServer, &oversized, 0u64, 1000u32);

    let drops_after = analyzer.buffer_saturation_drop_count();
    let parse_errors_after = analyzer.parse_error_count();

    assert_eq!(drops_after, drops_before + 1,
        "buffer_saturation_drops must increment by 1 on partial tail-drop \
         (data.len() > remaining: 65537 > 65536)");
    assert_eq!(parse_errors_after, parse_errors_before,
        "parse_errors must NOT increment on buffer saturation drop (BC-2.07.043 PC-6)");
    // No finding emitted — findings_count unchanged
    // (assert via analyzer.findings_count() if exposed, or check finding vec length).
}
```

### Sub-A-full-drop: test_BC_2_07_043_buffer_saturation_full_drop

This test covers the **full-drop** path: `remaining == 0` (buffer already at MAX_BUF
capacity). The `if remaining > 0` guard in `on_data` causes the entire incoming `data`
slice to be skipped. The increment condition `data.len() > remaining` evaluates as
`data.len() > 0` (true for any non-empty slice). This is the EC-002 edge case in
BC-2.07.043.

This path cannot be reached by a single-slice `on_data` call on a fresh analyzer
without first filling the buffer. The `fill_buf_for_testing` seam (added by the PO
alongside BC-2.07.043 v1.1) pre-fills `client_buf` or `server_buf` directly to
`MAX_BUF` without going through the `on_data` count-and-append loop, enabling a
deterministic, fast full-drop trigger.

**Seam contract:** `TlsAnalyzer::fill_buf_for_testing(flow_key, Direction, len)` fills
the specified direction's buffer to exactly `len` bytes (precondition: `len <= MAX_BUF`).
This is the ONLY accepted seam form (BC-2.07.043 Architecture Anchor). The test MUST
use `fill_buf_for_testing` on `TlsAnalyzer`. The `TlsFlowState`-level alternative
(`state_for_testing_mut().set_client_buf_len_for_testing(...)`) is NOT used here.

```rust
#[test]
fn test_BC_2_07_043_buffer_saturation_full_drop() {
    // Verify: when client_buf is already at MAX_BUF (remaining == 0), any
    // non-empty on_data delivery fires the full-drop path and increments
    // buffer_saturation_drops by exactly 1 (EC-002: "if remaining > 0" guard
    // skips append entirely; 0 bytes appended; event still counted).
    //
    // Trigger mechanism (full drop, uses fill_buf_for_testing seam):
    //   remaining = MAX_BUF.saturating_sub(MAX_BUF) = 0
    //   data.len() = 1000 > 0 = remaining → drop fires, counter increments.
    //
    // The seam pre-fills client_buf to MAX_BUF = 65,536 bytes directly,
    // bypassing the on_data loop. This makes the test independent of the
    // pre-fill iteration count and deterministic in one call.
    //
    // Seam: fill_buf_for_testing is the ONLY accepted seam (BC-2.07.043 Architecture Anchor).

    let mut analyzer = TlsAnalyzer::new();
    let flow = FlowKey::new(/* ... */);

    // Fill client_buf to capacity using the test seam (FINAL — not a placeholder).
    // Seam contract: buf.len() == MAX_BUF (65,536) after this call.
    analyzer.fill_buf_for_testing(&flow, Direction::ClientToServer, 65_536);

    let drops_before = analyzer.buffer_saturation_drop_count();
    let parse_errors_before = analyzer.parse_error_count();

    // Deliver any non-empty slice — the buffer is full, so the entire slice is dropped.
    analyzer.on_data(&flow, Direction::ClientToServer, &[0xABu8; 1000], 0u64, 1001u32);

    let drops_after = analyzer.buffer_saturation_drop_count();
    let parse_errors_after = analyzer.parse_error_count();

    assert_eq!(drops_after, drops_before + 1,
        "buffer_saturation_drops must increment by 1 on full-drop \
         (remaining==0: entire slice dropped; BC-2.07.043 EC-002)");
    assert_eq!(parse_errors_after, parse_errors_before,
        "parse_errors must NOT increment on buffer saturation drop (BC-2.07.043 PC-6)");
}
```

### Sub-B: test_BC_2_07_043_no_drop_no_counter

```rust
#[test]
fn test_BC_2_07_043_no_drop_no_counter() {
    // Verify: when on_data delivers data that fits entirely within the available
    // buffer capacity, buffer_saturation_drops is unchanged.

    let mut analyzer = TlsAnalyzer::new();
    let flow = FlowKey::new(/* ... */);

    let drops_before = analyzer.buffer_saturation_drop_count();

    // Deliver a small TLS record (well within MAX_BUF) — no drop possible.
    // Use a minimal valid-framed record (e.g. 5-byte TLS record header + 1 payload byte).
    let small_record = &[0x17u8, 0x03, 0x01, 0x00, 0x01, 0xAA];
    analyzer.on_data(&flow, Direction::ClientToServer, small_record, 0u64, 1000u32);

    let drops_after = analyzer.buffer_saturation_drop_count();

    assert_eq!(drops_after, drops_before,
        "buffer_saturation_drops must NOT increment when data fits in buffer");
}
```

### Sub-C: test_BC_2_07_043_counter_persists_across_flows

```rust
#[test]
fn test_BC_2_07_043_counter_persists_across_flows() {
    // Verify: buffer_saturation_drops accumulates across flows and is NOT reset
    // when on_flow_close is called.

    let mut analyzer = TlsAnalyzer::new();
    let flow = FlowKey::new(/* ... */);

    // Phase 1: trigger exactly one drop on the flow (same pre-fill + overflow
    // approach as Sub-A).
    // ...fill_to_capacity(&mut analyzer, &flow);
    // ...deliver_one_more_byte(&mut analyzer, &flow);

    let drops_after_drop = analyzer.buffer_saturation_drop_count();
    assert_eq!(drops_after_drop, 1,
        "counter must be 1 after one drop");

    // Phase 2: close the flow.
    // Import: use wirerust::reassembly::handler::CloseReason; (or the crate's re-export path)
    analyzer.on_flow_close(&flow, CloseReason::Fin);

    // Phase 3: assert counter is unchanged after flow close.
    let drops_after_close = analyzer.buffer_saturation_drop_count();
    assert_eq!(drops_after_close, drops_after_drop,
        "buffer_saturation_drops must NOT be reset by on_flow_close — it is a TlsAnalyzer aggregate");
}
```

### Sub-D: test_BC_2_07_043_summarize_value_equals_drop_count

```rust
#[test]
fn test_BC_2_07_043_summarize_value_equals_drop_count() {
    // Verify: summarize() detail map contains key "buffer_saturation_drops"
    // with value equal to the current counter value (value-equality, not mere
    // key presence — mirrors test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key;
    // BC-2.07.043 PC-4).

    let mut analyzer = TlsAnalyzer::new();
    let flow = FlowKey::new(/* ... */);

    // Trigger exactly one drop (same pre-fill pattern as Sub-A).
    // ...fill_to_capacity(&mut analyzer, &flow);
    // ...deliver_one_more_byte(&mut analyzer, &flow);

    let summary = analyzer.summarize();
    let detail = summary.detail; // serde_json::Value map

    let key_value = detail["buffer_saturation_drops"]
        .as_u64()
        .expect("summarize() detail map must contain key 'buffer_saturation_drops'");
    assert_eq!(key_value, 1u64,
        "detail['buffer_saturation_drops'] must equal the actual counter value (1); \
         mere key presence is not sufficient — BC-2.07.043 PC-4 requires value equality \
         (the detail map must expose the correct u64 value, mirroring the \
         'truncated_records' and 'handshake_reassembly_overflows' surfacing pattern)");
}
```

### Sub-E: test_BC_2_07_043_both_directions_increment_same_counter

```rust
#[test]
fn test_BC_2_07_043_both_directions_increment_same_counter() {
    // Verify: a drop in the client direction AND a drop in the server direction
    // both increment the SAME buffer_saturation_drops aggregate counter.
    // After one c2s drop + one s2c drop, counter == initial + 2.

    let mut analyzer = TlsAnalyzer::new();
    let flow = FlowKey::new(/* ... */);

    let drops_initial = analyzer.buffer_saturation_drop_count();

    // Phase 1: trigger a drop in the client direction (client_buf overflow).
    // ...fill_to_capacity_c2s(&mut analyzer, &flow);
    // ...deliver_one_more_byte_c2s(&mut analyzer, &flow);
    // (buffer_saturation_drops == initial + 1)

    // Phase 2: trigger a drop in the server direction (server_buf overflow).
    // Must use a DIFFERENT flow key OR close/reopen the flow to reset the
    // server_buf on a fresh flow, OR use a flow where server_buf was not
    // pre-filled. Alternatively, open a second flow for the server-direction test.
    let flow2 = FlowKey::new(/* different port or IP ... */);
    // ...fill_to_capacity_s2c(&mut analyzer, &flow2);
    // ...deliver_one_more_byte_s2c(&mut analyzer, &flow2);
    // (buffer_saturation_drops == initial + 2)

    let drops_final = analyzer.buffer_saturation_drop_count();
    assert_eq!(drops_final, drops_initial + 2,
        "one c2s drop + one s2c drop must both increment the SAME aggregate counter; \
         expected initial+2, got initial+{}", drops_final - drops_initial);
}
```

## Feasibility Assessment

**Two drop paths; two test strategies:**

1. **Partial-drop path** (`remaining > 0` but `data.len() > remaining`): Covered by
   Sub-A (`test_BC_2_07_043_buffer_saturation_observable`). A single `on_data` call with
   a `MAX_BUF + 1` byte slice (65,537 bytes) against a fresh empty buffer is sufficient.
   `remaining == MAX_BUF == 65,536`; one byte is dropped. No seam required — this is
   self-contained and falsifiable.

2. **Full-drop path** (`remaining == 0`, the `if remaining > 0` guard skips the append):
   Covered by `test_BC_2_07_043_buffer_saturation_full_drop`. To reach `remaining == 0`,
   `client_buf.len()` must equal `MAX_BUF = 65,536`. Without a seam, this requires ~45
   `on_data` calls (each carrying a maximum-sized chunk of ~1,460 bytes) to fill the
   buffer through the normal `on_data` loop — feasible but slow. The `fill_buf_for_testing`
   seam (to be delivered by the PO alongside BC-2.07.043 v1.1) allows filling the buffer
   directly in one call, making the test instantaneous and the mechanism explicit.

   **Seam trigger mechanism:** `fill_buf_for_testing(&flow, Direction, MAX_BUF)` sets
   `client_buf.len() == 65,536`. The subsequent `on_data` call finds `remaining == 0`,
   the `if remaining > 0` block is skipped entirely, the local `did_drop` flag is set to
   `true`, and after the `&mut state` block closes, `self.buffer_saturation_drops += 1`.

   **Seam name is FINAL:** The seam is `TlsAnalyzer::fill_buf_for_testing(flow_key,
   Direction, n)` per BC-2.07.043 Architecture Anchor. The `TlsFlowState`-level
   alternative is NOT used. No coordination deferred.

**The `buffer_saturation_drop_count()` accessor** must be added to `TlsAnalyzer` following
the existing `parse_error_count()` / `truncated_record_count()` / `handshake_reassembly_overflow_count()`
pattern.

**Seam contract (mirrors VP-039):** All counter reads use `TlsAnalyzer`-level accessors.
`buffer_saturation_drops` is NOT on `TlsFlowState`. The counter is NOT read directly from
the struct field in tests — always use the accessor.

**Increment site borrow constraint (C-1 documented):** `self.buffer_saturation_drops`
cannot be mutated while `state: &mut TlsFlowState` borrows from `self.flows`. The
implementation uses a local `did_drop: bool` flag (set inside the `&mut state` block on
the tail-drop path) with `if did_drop { self.buffer_saturation_drops += 1; }` placed
AFTER the block. This is a Rust borrow constraint, not an architectural choice.

## BC Traceability

| BC | Postcondition | Sub-Property |
|----|--------------|--------------|
| BC-2.07.043 | PC-1: drop increments counter by exactly 1 | Sub-A |
| BC-2.07.043 | PC-1 (negative case): when data fits (no drop), increment condition is NOT met — counter unchanged | Sub-B |
| BC-2.07.043 | PC-5: counter is NOT reset when a flow closes (aggregate on TlsAnalyzer, not TlsFlowState) | Sub-C |
| BC-2.07.043 | PC-4: summarize() exposes key "buffer_saturation_drops" in detail map with value equal to current counter | Sub-D |
| BC-2.07.043 | PC-3: both Direction::ClientToServer and Direction::ServerToClient saturation events increment the same aggregate counter | Sub-E |
| BC-2.07.043 | PC-6: parse_errors NOT incremented; no Finding pushed; byte-drop behavior of BC-2.07.005 unchanged | Sub-A |

## Red-Gate Test Names (from BC-2.07.043)

Per the PO-specified Red-Gate contract, the following test names are FIXED and must
appear verbatim in the test suite. These names are the authoritative DF-AC-TEST-NAME-SYNC
anchor — the VP-040 test function names MUST match the BC-2.07.043 VP-table test names
exactly (per DF-AC-TEST-NAME-SYNC policy).

1. `test_BC_2_07_043_buffer_saturation_observable` (Sub-A: partial-drop path, no seam)
2. `test_BC_2_07_043_buffer_saturation_full_drop` (Sub-A-full-drop: full-drop path, uses fill_buf_for_testing seam; remaining==0)
3. `test_BC_2_07_043_no_drop_no_counter` (Sub-B)
4. `test_BC_2_07_043_counter_persists_across_flows` (Sub-C)
5. `test_BC_2_07_043_summarize_value_equals_drop_count` (Sub-D: value-equality assertion — detail["buffer_saturation_drops"].as_u64()==expected count; BC-2.07.043 PC-4)
6. `test_BC_2_07_043_both_directions_increment_same_counter` (Sub-E)

These 6 names are FINAL and authoritative per DF-AC-TEST-NAME-SYNC. The PO's
BC-2.07.043 VP table MUST cite these exact names.
