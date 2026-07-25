# AC-181-001: Unsafe *mut EnipFlowState Split-Borrow Eliminated

**AC:** AC-181-001  
**Story:** STORY-181 (SEC-001 ENIP unsafe split-borrow refactor)  
**Date:** 2026-07-24  
**Branch:** feature/STORY-181-enip-sec001-split-borrow

---

## Verdict: PASS

---

## grep Confirms Zero Unsafe Symbols at Former Site

Command:
```
grep -n "flow_ptr\|ptr_as_ptr\|\*mut EnipFlowState\|unsafe" src/analyzer/enip.rs
```

Output:
```
(no output — zero matches)
```

All four patterns that defined the SEC-001 unsafe block (`flow_ptr`, `ptr_as_ptr`,
`*mut EnipFlowState`, `unsafe`) are absent from `src/analyzer/enip.rs` after the refactor.

---

## Before: Old Unsafe Block (git show 421bf572:src/analyzer/enip.rs, lines 985–1000)

```rust
        for pdu in pdu_queue {
            // SAFETY (split-borrow): flow_ptr aliases self.flows[flow_key]. process_pdu
            // only touches self.all_findings, self.error_count, self.write_count,
            // self.dropped_findings, self.enip_write_burst_threshold,
            // self.enip_error_burst_threshold — none of which overlap with self.flows.
            // The aliased field is therefore not accessed by process_pdu, making the
            // exclusive-reference invariant sound.
            let flow_ptr: *mut EnipFlowState = self
                .flows
                .get_mut(&flow_key)
                .expect("flow exists: inserted above and not removed");
            // SAFETY: flow_ptr is a valid &mut obtained from self.flows. process_pdu does
            // not call self.flows or alias flow_ptr through any other path.
            #[allow(clippy::ptr_as_ptr)]
            self.process_pdu(unsafe { &mut *flow_ptr }, &pdu, timestamp, src_ip);
        }
```

The old pattern:
- Created `*mut EnipFlowState` via `self.flows.get_mut(&flow_key)` (live aliasing `self.flows`)
- Called `self.process_pdu(unsafe { &mut *flow_ptr }, ...)` (simultaneously `&mut self` + `&mut *flow_ptr`)
- Required a multi-line SAFETY comment to document a convention the compiler could not enforce
- Carried `#[allow(clippy::ptr_as_ptr)]` to silence the lint on the raw pointer cast

---

## After: Take-Remove-Reinsert Pattern (current HEAD, src/analyzer/enip.rs, lines 978–1001)

```rust
        // Dispatch each collected valid PDU using take-remove-reinsert (SEC-001 fix).
        // The flow is removed from self.flows before the loop; the resulting owned local
        // EnipFlowState is structurally disjoint from self.flows. process_pdu(&mut self,
        // &mut flow, ...) is therefore safe: &mut self (for process_pdu's access to
        // self.all_findings, self.error_count, self.write_count, self.dropped_findings,
        // and threshold fields) and &mut flow (the local variable) do not alias — the
        // compiler enforces this disjointness, not a convention. After all PDUs are
        // dispatched, the flow is re-inserted.
        // is_non_enip safety: if the flag was already true on on_data entry, the early
        // return at ~801 fired before any PDUs were collected, so pdu_queue is empty here.
        // If the flag was latched during this call's carry-cap check (~955-974), pdu_queue
        // may still contain items (per RULING-137-002 this latch branch is currently
        // structurally unreachable; retained as defensive documentation for the anticipated
        // carry-cap redesign); process_pdu gates on flow.is_non_enip (process_pdu's
        // is_non_enip early-return gate) and skips all detection for those PDUs.
        let mut flow = self
            .flows
            .remove(&flow_key)
            .expect("flow exists: inserted above and not removed");
        for pdu in pdu_queue {
            self.process_pdu(&mut flow, &pdu, timestamp, src_ip);
        }
        self.flows.insert(flow_key, flow);
```

The new pattern:
- `self.flows.remove(&flow_key)` produces an owned local `EnipFlowState`
- `&mut flow` (the local) and `&mut self` (for `process_pdu`) are structurally disjoint
- The compiler enforces disjointness — no convention required
- No `unsafe` block, no `*mut` cast, no `#[allow(clippy::ptr_as_ptr)]`

---

## process_pdu Signature: Unchanged

The method signature at `src/analyzer/enip.rs` line 1032 is:
```rust
    pub fn process_pdu(
        &mut self,
        flow: &mut EnipFlowState,
        pdu: &[u8],
        timestamp: u32,
        src_ip: IpAddr,
    ) {
```

Identical to the pre-refactor signature. The fix is local to the `on_data` call site.
