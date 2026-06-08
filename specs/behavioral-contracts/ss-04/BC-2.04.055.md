---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-08T00:00:00Z
phase: 1a
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-04
capability: CAP-04
lifecycle_status: active
introduced: v0.2.0-feature-100
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/feature-delta/issue-100-pcap-timestamps/delta-analysis.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.054.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.007.md
input-hash: fb17ddb
---

# BC-2.04.055: StreamHandler::on_data Carries Capture-Relative Timestamp Parameter

## Description

`StreamHandler::on_data` gains a `timestamp: u32` parameter that carries the
CAPTURE-RELATIVE pcap `ts_sec` value at the flush call site. The reassembler
passes the timestamp in two semantically distinct cases: hot-path flushes receive the
current packet's `timestamp_secs`; close-flush paths receive `TcpFlow.last_seen`, the
most-recently-seen packet timestamp for the flow. This two-case design is zero-new-state:
`TcpFlow.last_seen` already exists. All five production implementors of the trait
(`TlsAnalyzer`, `HttpAnalyzer`, `StreamDispatcher`, and the two test handlers) must add
this parameter to compile; the trait is a breaking-change boundary.

## Preconditions

1. `TcpReassembler` calls `handler.on_data(...)` during either `flush_contiguous_data`
   (hot-path, triggered by a new in-order or gap-filling packet) or `close_flow` (lifecycle
   close: FIN, RST, timeout, or eviction).
2. For hot-path calls: the current packet's `timestamp_secs: u32` is in scope within
   `process_packet` and flows through to `flush_contiguous_data` via parameter.
3. For close-flush calls: `TcpFlow.last_seen: u32` is available from the just-removed flow
   value (`let Some(mut flow) = self.flows.remove(key)` at lifecycle.rs:42). No additional
   state storage is needed.
4. Any flow that reaches `on_data` has seen at least one packet; `TcpFlow::new` sets
   `last_seen` to the constructor's `timestamp` argument, so `last_seen` is always non-zero
   for a real flow.

## Postconditions

1. The `timestamp: u32` argument to `on_data` equals:
   - **Hot-path (`flush_contiguous_data`)**: the current packet's `timestamp_secs` — the
     `u32` Unix epoch seconds of the packet that triggered the flush. This is the direct
     pcap `ts_sec` value, not a derived or averaged value.
   - **Close-flush (`close_flow`)**: `flow.last_seen` — the `u32` Unix epoch seconds of
     the most-recently-seen packet on this flow, read from the `TcpFlow` value before it
     is removed from `self.flows`. This is the closest available temporal bound for the
     closing event (FIN/RST/timeout/eviction).
2. `StreamDispatcher::on_data` forwards the `timestamp` argument unchanged to both
   `http.on_data(...)` and `tls.on_data(...)` downstream calls.
3. Implementors of `StreamHandler::on_data` that store per-flow last-seen timestamp
   (specifically `TlsAnalyzer` and `HttpAnalyzer`) update their per-flow timestamp on
   every `on_data` call: `self.flows.get_mut(flow_key).map(|s| s.last_ts = timestamp)`.
   The stored value is then used to populate `Finding.timestamp` at all emission sites.
4. Test handlers (`RecordingHandler` in reassembly_engine_tests.rs and the anonymous
   handler in hs043_flow_expiry_tests.rs) compile with the new parameter; `RecordingHandler`
   may or may not record the timestamp value depending on what test assertions require.

## Invariants

1. The timestamp value is CAPTURE-RELATIVE PROVENANCE: pcap capture-host wall clock,
   possibly unsynchronized and non-monotonic. Implementors must not treat it as
   authoritative time. (See BC-2.09.007 for the NIST SP 800-86 framing.)
2. The `timestamp: u32` parameter is never `Option<u32>` — it is always a concrete value.
   There is no "no timestamp available" state for hot-path flushes (current packet is in
   scope) or for close-flush paths (`flow.last_seen` is always set). Using `u32` (not
   `Option<u32>`) ensures implementors cannot accidentally ignore the value.
3. Cross-flow isolation (VP-014): per-flow timestamp storage in `HttpAnalyzer` and
   `TlsAnalyzer` MUST use `FlowKey` as the map key, consistent with all other per-flow
   state. A timestamp from flow A must never appear in a Finding from flow B. This is
   the same isolation invariant VP-014 already covers for HTTP per-flow state; the
   per-flow timestamp `HashMap<FlowKey, u32>` follows the same keying pattern.
4. The existing `on_data` call sites in test files that currently compile with 4 arguments
   must be updated to 5 arguments. The build fails to compile if any implementor is
   missed — the trait-signature break is a compile-time guarantee of completeness.
5. The `timestamp` parameter introduced here is `u32` (seconds precision). Sub-second
   precision via `timestamp_usecs` is explicitly deferred (F1 open question 1). Analyzers
   that need sub-second precision for future features must use a separate mechanism; this
   BC does not guarantee microsecond threading.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | flush_contiguous_data called for a packet with ts_sec=0 (Unix epoch) | timestamp=0 passed to on_data; implementors store 0; Finding.timestamp = Some(1970-01-01T00:00:00Z) |
| EC-002 | close_flow called after flow has seen exactly one packet | flow.last_seen = timestamp of that single packet; passed to on_data; non-zero and correct |
| EC-003 | StreamDispatcher forwards timestamp to both analyzers | http.on_data(..., timestamp) and tls.on_data(..., timestamp) receive identical u32 value |
| EC-004 | RecordingHandler in tests — timestamp not captured in data_events tuple | Compilation succeeds; on_data has `_timestamp: u32` or `timestamp: u32`; existing test assertions unaffected |
| EC-005 | close_flow called for FIN path | `flow.last_seen` is the timestamp of the FIN-carrying packet (or the last data packet if FIN has no payload); closest available anchor |
| EC-006 | close_flow called for timeout/eviction path | `flow.last_seen` is the timestamp of the last packet seen before the flow went idle; correct forensic anchor for "when was this flow last active?" |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| process_packet(packet { ts_sec=5000 }) → flush_contiguous_data triggers on_data | on_data receives timestamp=5000 | happy-path |
| flow with last_seen=3000 closed by FIN → close_flow triggers on_data | on_data receives timestamp=3000 | happy-path |
| StreamDispatcher.on_data(..., timestamp=7777) called | http.on_data receives 7777; tls.on_data receives 7777 (if both analyzers active) | happy-path |
| Two concurrent flows: A last_seen=1000, B last_seen=2000; both closed | on_data for A receives 1000; on_data for B receives 2000; no cross-contamination | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-021 | on_data timestamp correctly threaded with two-case semantics | integration test + proptest |
| VP-014 | Cross-flow isolation holds for per-flow timestamp state added to HttpAnalyzer | proptest (existing VP; extended scope) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md — adding `timestamp: u32` to the `on_data` trait method is a direct extension of the reassembly engine's data-delivery contract; the flush call site is the only point where the pcap timestamp is available and must be threaded downstream |
| L2 Domain Invariants | None directly; supports INV-4 (raw-data preservation) by threading raw pcap u32 without conversion at the reassembly layer |
| Architecture Module | SS-04 (reassembly/handler.rs trait definition; reassembly/mod.rs flush_contiguous_data; reassembly/lifecycle.rs close_flow); SS-05 (dispatcher.rs forwarding) |
| Stories | STORY-097, STORY-098, STORY-099 |
| Feature | issue-100-pcap-timestamps |
| Resolves | O-01 (timestamp threading gap) at the trait-boundary level |

## Related BCs

- BC-2.09.007 — depends on (the timestamp value threaded here becomes Finding.timestamp there)
- BC-2.04.054 — related to (segment-limit summary finding retains timestamp: None — that path goes through finalize, not on_data)
- BC-2.04.012 — related to (finalize calls close_flow; the close-flush timestamp semantics interact with finalize's lifecycle)
- BC-2.04.007 — related to (flush_contiguous is the hot-path; on_data is called from here)

## Architecture Anchors

- `src/reassembly/handler.rs:49` — `StreamHandler::on_data` trait method (add `timestamp: u32` parameter)
- `src/reassembly/mod.rs` — `flush_contiguous_data`: pass current-packet `timestamp: u32` to `handler.on_data`
- `src/reassembly/lifecycle.rs:42-57` — `close_flow`: pass `flow.last_seen` to `handler.on_data`
- `src/dispatcher.rs:144` — `StreamDispatcher::on_data` (add and forward `timestamp: u32`)
- `src/analyzer/http.rs:501` — `HttpAnalyzer::on_data` (add `timestamp: u32`; store per-flow)
- `src/analyzer/tls.rs:771` — `TlsAnalyzer::on_data` (add `timestamp: u32`; store per-flow)

## Story Anchor

STORY-097 (trait + flush-path wiring — Story A from F3 decomposition); STORY-098 (analyzer emission); STORY-099 (E2E verification).

## VP Anchors

- VP-021 — timestamp-provenance-threading (draft/unverified; integration test + proptest)
- VP-014 — HttpAnalyzer Cross-Flow Isolation (verified; extended scope for per-flow timestamp map)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | F1 delta-analysis §4.1 (trait definition), §4.3 (call sites), §3 (two-case semantics decision) |
| **Confidence** | high |
| **Extraction Date** | 2026-06-08 |

## Evidence Types Used

- **delta-analysis**: F1 analysis documents the exact files/lines, two-case semantics justification, and zero-new-state property
- **type constraint**: trait break is a compile-time guarantee all implementors are updated
- **guard clause**: `TcpFlow::new` sets `last_seen` at construction; `last_seen` is never zero for a real flow

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (trait method call; I/O upstream) |
| **Global state access** | none |
| **Deterministic** | yes — same timestamp argument always produces same value at call site |
| **Thread safety** | not thread-safe (&mut self on implementors) |
| **Overall classification** | mixed (stateful mutation in implementors that store per-flow timestamp) |
