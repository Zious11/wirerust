---
document_type: feature-delta-analysis
feature_id: issue-100-pcap-timestamps
github_issue: 100
title: "Thread pcap per-packet timestamps through to Finding.timestamp"
intent: enhancement
feature_type: backend
trivial_scope: false
scope_classification: standard
status: draft
producer: architect
created: 2026-06-08
traces_to:
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.001.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.006.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.005.md
---

# F1 Delta Analysis — Issue #100: Thread pcap per-packet timestamps through to Finding.timestamp

## 1. Feature Summary

`Finding.timestamp: Option<DateTime<Utc>>` exists and is fully wired into all three reporters
(terminal, JSON, CSV) with `#[serde(skip_serializing_if = "Option::is_none")]`. However all
22 emission sites pass `timestamp: None` (documented as domain-debt O-01 in BC-2.09.001 and
BC-2.09.006). The pcap source timestamp is available as `RawPacket.timestamp_secs: u32` and
`RawPacket.timestamp_usecs: u32` (reader.rs:33-35) and is fed to `TcpReassembler::process_packet`
as a `u32` (main.rs:167), but stops there — `StreamHandler::on_data` (handler.rs:49) has no
timestamp parameter, so no downstream analyzer can attach it to a Finding.

This feature wires the capture-relative timestamp all the way to every Finding emission site.

---

## 2. Intent and Scope Classification

| Field | Value |
|-------|-------|
| Intent | enhancement (existing field, never populated) |
| Feature type | backend |
| Trivial scope? | NO — trait-signature break ripples through 5 implementing types + 2 test handlers + 22 emission sites |
| Pipeline route | Full F1 → F2 → F3 → F4 (standard Feature Mode) |

---

## 3. Timestamp Semantics Decision

### The core design question

Two call paths invoke `handler.on_data` at flush time:

**Path A — `flush_contiguous_data` (mod.rs:546–561):** called during `process_packet` after a
new segment is inserted. The reassembler is actively processing a specific packet at timestamp
`T`. The flushed data is the contiguous prefix newly unblocked by that packet; `T` is the
timestamp of the packet that caused the flush.

**Path B — `close_flow` (lifecycle.rs:36–61):** called from `finalize`, `expire_idle_by_timeout`,
`evict_flows`, and `apply_handshake_flags` (RST). There is no current-packet timestamp available.
The flow's `TcpFlow` struct carries `last_seen: u32` — the timestamp of the last packet that
touched this flow. The `TcpFlow` also carries `first_seen: u32`.

### Candidates considered

| Option | Semantics | Verdict |
|--------|-----------|---------|
| A. Pass current packet ts to `on_data` everywhere | Path A: exact. Path B: unavailable — would require a per-call `Option<u32>` | Overly complex; None on flush is misleading |
| B. Store per-flow `last_seen_ts` in FlowState and read it in `close_flow` | Delivers `TcpFlow::last_seen` for flush-on-close paths | `last_seen` already exists as `TcpFlow.last_seen: u32` — zero new state |
| C. Store per-flow `first_seen_ts` | First-seen is already tracked as `TcpFlow.first_seen: u32` | Misleading for long-lived flows; analysts expect ts near event |

### Recommendation: per-flow `last_seen` timestamp for all flush calls

**Add `timestamp: u32` to `StreamHandler::on_data`** and pass it in both flush paths:

- **Path A** (`flush_contiguous_data`): pass the current packet's `timestamp` (already in scope
  as the `u32` argument to `process_packet`).
- **Path B** (`close_flow`): pass `flow.last_seen` — the timestamp of the most-recent packet
  seen on the flow, which was just removed from `self.flows` via `flows.remove(key)`. This is
  the best proxy available: it bounds the Finding within the activity window of the flow.

### Justification vs alternatives

Using `last_seen` for close-flush paths is semantically correct because:
1. It is always populated; `TcpFlow::new` sets both `first_seen` and `last_seen` to the
   constructor's `timestamp` argument, so no flow ever has a zero `last_seen` from a real
   packet.
2. For timeout and eviction flushes, `last_seen` is the most recent evidence of activity —
   exactly what forensic analysts want for "when was this anomaly active?"
3. For FIN/RST flushes, `last_seen` is the timestamp of the packet that closed the flow, which
   is the closest available timestamp to the detection event.

**NIST SP 800-86 framing (per issue statement):** The timestamp field must be framed as
CAPTURE-RELATIVE PROVENANCE, not authoritative clock time. pcap `ts_sec` is capture-host
wall-clock, possibly unsynchronized, non-monotonic across reboots, and file-format-specified
as seconds since Unix epoch with no timezone guarantee. The Finding docs and BC statement must
explicitly carry this caveat.

### Conversion: u32 seconds → DateTime<Utc>

`RawPacket.timestamp_secs` is a `u32` (Unix epoch seconds). Conversion:
```rust
DateTime::from_timestamp(u32_ts as i64, 0).map(|dt| dt.with_timezone(&Utc))
```
This is infallible for any valid u32 (max ~2106 CE). `timestamp_usecs` should be wired to
the `nsecs` argument (`u32_usecs * 1_000`) for sub-second precision where available.

---

## 4. Impact Boundary — Exact Files and Signatures

### 4.1 MODIFIED: Trait definition (breaking change)

| File | Current signature (line) | New signature |
|------|--------------------------|---------------|
| `src/reassembly/handler.rs:49` | `fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], offset: u64)` | `fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], offset: u64, timestamp: u32)` |

This is a **breaking change to a public trait**. All 5 implementing types below must update
simultaneously.

### 4.2 MODIFIED: Production trait implementors (3 types)

| File | Type | Current line | Change |
|------|------|-------------|--------|
| `src/analyzer/tls.rs:771` | `impl StreamHandler for TlsAnalyzer` | `:771` | Add `timestamp: u32` param; store per-flow for use at emission sites |
| `src/analyzer/http.rs:501` | `impl StreamHandler for HttpAnalyzer` | `:501` | Add `timestamp: u32` param; store per-flow for use at emission sites |
| `src/dispatcher.rs:144` | `impl StreamHandler for StreamDispatcher` | `:144` | Add `timestamp: u32` param; forward to downstream `http.on_data(...)` at `:183` and `tls.on_data(...)` at `:188` |

### 4.3 MODIFIED: Internal reassembler call sites (2 call sites, 0 signature changes)

These are callers of `handler.on_data`, not implementors:

| File | Function | Line | Change |
|------|----------|------|--------|
| `src/reassembly/mod.rs` | `flush_contiguous_data` | `:560` | Pass current-packet `timestamp: u32` (already in `process_packet` scope via parameter) |
| `src/reassembly/lifecycle.rs` | `close_flow` | `:57` | Pass `flow.last_seen` (available from the just-removed `TcpFlow` value) |

Note: `close_flow` already holds `let Some(mut flow) = self.flows.remove(key)` at `:42`, so
`flow.last_seen` is directly accessible. No new state needed.

### 4.4 MODIFIED: FlowState (possible — for per-flow timestamp threading to analyzers)

| File | Change |
|------|--------|
| `src/reassembly/flow.rs` | No struct change needed — `TcpFlow.last_seen` already exists |

For TLS and HTTP analyzers, the timestamp arrives per `on_data` call. Analyzers need to store
the most-recently-seen timestamp and attach it to Findings at emission time. This requires
adding a `last_ts: u32` (or `Option<u32>`) field to `TlsAnalyzer` and `HttpAnalyzer` state,
updated in `on_data`. Exact struct fields TBD in F2 spec.

### 4.5 MODIFIED: All 22 emission sites

| Location | Count | Change |
|----------|-------|--------|
| `src/reassembly/mod.rs` (overlap, small-segment, out-of-window anomalies, segment-limit summary) | 4 sites | `timestamp: None` → `timestamp: Some(...)` |
| `src/reassembly/lifecycle.rs` (conflicting-overlap, stream-depth-exceeded) | 2 sites | `timestamp: None` → `timestamp: Some(...)` |
| `src/analyzer/tls.rs` | 7 sites | `timestamp: None` → `timestamp: Some(stored_last_ts)` |
| `src/analyzer/http.rs` | 9 sites | `timestamp: None` → `timestamp: Some(stored_last_ts)` |

Total: 22 production emission sites.

**Reassembly engine sites (mod.rs):** the current-packet `timestamp: u32` is already in scope
for the anomaly-threshold sites (they are called from `check_anomaly_thresholds` which is
called from `process_packet` which holds `timestamp: u32`). For the segment-limit summary in
`finalize`, there is no per-packet timestamp; `None` is correct and must remain `None` (the
summary finding is a post-capture aggregate, not tied to any specific packet).

**Special case — segment-limit summary finding (mod.rs:~644):** remains `timestamp: None`.
This is a summary aggregate emitted at finalize, not tied to any packet. This is correct
behavior, not a gap.

### 4.6 MODIFIED: Test handlers (trait-signature break forces update)

| File | Handler | Line | Change |
|------|---------|------|--------|
| `tests/reassembly_engine_tests.rs` | `RecordingHandler::on_data` | `:51` | Add `timestamp: u32` param |
| `tests/hs043_flow_expiry_tests.rs` | anonymous inline handler `on_data` | `:69` | Add `_timestamp: u32` param |

**Note on `reassembly_engine_tests.rs`:** The `data_events` tuple is currently
`(FlowKey, Direction, Vec<u8>, u64)`. The tuple type may or may not need to capture the new
`timestamp` argument depending on which test assertions need it. At minimum the signature must
compile; assertions that verify timestamp propagation end-to-end are new tests, not changes to
existing ones.

### 4.7 MODIFIED: Test files with inline `timestamp: None` (assertion or construction sites)

| File | Lines | Change needed |
|------|-------|---------------|
| `tests/reassembly_engine_tests.rs` | `:14365`, `:15229`, `:15256` | These are Finding construction sites; update to `timestamp: Some(...)` where the test now provides a known timestamp, or leave as `None` for the one summary site |

### 4.8 NOT CHANGED (regression baseline)

| File | Reason |
|------|--------|
| `src/findings.rs` | `Finding.timestamp` field already exists and has correct type + serde attribute |
| `src/reader.rs` | `RawPacket.timestamp_secs` / `timestamp_usecs` already captured correctly |
| `src/main.rs` | `raw.timestamp_secs` already passed to `process_packet` at `:167` |
| `src/reporter/json.rs` | No change — serde handles Some(ts) serialization automatically |
| `src/reporter/csv.rs` | Check if timestamp column is already emitted or needs adding (likely minor format change) |
| `src/reporter/terminal.rs` | Check if timestamp is rendered; if `Some(ts)` now appears, verify display is correct |
| `src/reassembly/config.rs` | No change |
| `src/reassembly/stats.rs` | No change |
| `src/reassembly/segment.rs` | No change |
| `src/decoder.rs` | No change |
| `src/dispatcher.rs` (on_flow_close) | No change to this method |
| `src/analyzer/dns.rs` | DNS analyzer does not use StreamHandler trait; no change |
| `src/reassembly/mod.rs` (Kani proofs) | VP-003 proofs operate on `findings.len()` only, not field values; no change |
| `src/dispatcher.rs` (Kani proofs, VP-004) | VP-004 proofs model `classify` and cache state; no change |

---

## 5. Affected Specifications and BCs

### 5.1 BCs that must be MODIFIED

| BC ID | File | Current State | Required Change |
|-------|------|---------------|-----------------|
| BC-2.09.001 | ss-09/BC-2.09.001.md | Invariant 1: "All 22 sites set `timestamp: None` (O-01)" | Update: 21 of 22 sites now set `Some(...)`; the segment-limit summary remains `None`. Remove O-01 classification. |
| BC-2.09.006 | ss-09/BC-2.09.006.md | "timestamp never appears in JSON output" (Invariant 2) | Update: timestamp now appears in JSON when `Some`. EC-005 becomes a positive test. |
| BC-2.01.005 | ss-01/BC-2.01.005.md | "O-01: timestamps are read but never threaded to Finding constructors" | Update: O-01 resolved. Add traceability to new BC. |

### 5.2 New BC required

**A new behavioral contract is needed** because no existing BC covers the end-to-end invariant:
"when a Finding is emitted from a flow-data path, its `timestamp` field carries the
capture-relative pcap `ts_sec` of the flush-triggering packet (or `flow.last_seen` for
close-flush paths), expressed as `DateTime<Utc>` with CAPTURE-RELATIVE-PROVENANCE framing."

**Proposed BC: BC-2.09.007**

```
BC-2.09.007: Finding.timestamp Carries Capture-Relative Pcap Timestamp from on_data Call Site

Subsystem: SS-09 (findings) + SS-04 (reassembly) + SS-06 (HTTP) + SS-07 (TLS)

Preconditions:
  1. A Finding is emitted from a flow-data path (on_data flush or close_flow flush).
  2. The pcap source packet has a valid `timestamp_secs: u32` field.

Postconditions:
  1. Finding.timestamp = Some(DateTime::from_timestamp(ts_sec as i64, ts_usec * 1000 as u32))
     where ts_sec is the on_data-provided timestamp.
  2. For close-flush paths (close_flow): timestamp derives from TcpFlow.last_seen
     (the timestamp of the most-recently-seen packet on the flow).
  3. For the segment-limit summary finding (finalize aggregate): timestamp remains None.
  4. Finding.timestamp, when Some, serializes to ISO-8601 in JSON and is omitted when None
     (existing skip_serializing_if behavior; no change needed to serde attribute).
  5. The timestamp value is labeled capture-relative provenance. It reflects the
     capture host's wall clock and may be unsynchronized, non-monotonic, or subject to
     capture-tool limitations (per NIST SP 800-86). It is NOT authoritative time.

Invariants:
  1. timestamp: Some(...) appears on all 21 flow-data-path emission sites.
  2. timestamp: None appears only on the segment-limit summary Finding.
  3. The u32 ts_sec value is never silently dropped; conversion from u32 to DateTime<Utc>
     is lossless for all valid pcap values (max u32 = 2106 CE).
```

**Proposed BC: BC-2.04.055** (reassembly subsystem complement)

```
BC-2.04.055: StreamHandler::on_data Carries Capture-Relative Timestamp Parameter

Preconditions:
  1. TcpReassembler calls handler.on_data during flush_contiguous_data or close_flow.

Postconditions:
  1. The `timestamp: u32` argument to on_data equals:
     - The current packet's timestamp_secs for flush_contiguous_data calls.
     - TcpFlow.last_seen for close_flow calls (the most-recent packet timestamp
       for this flow, captured before the flow is removed).
  2. Implementors of StreamHandler::on_data receive a non-zero timestamp for any
     flow that saw at least one packet (guaranteed by TcpFlow::new invariant).
```

### 5.3 VP impact

| VP | Impact |
|----|--------|
| VP-003 (max-findings cap) | No structural impact. The cap proof operates on `findings.len()`, not field values. However, the Kani harness skeletons in `mod.rs` construct `Finding` literals with `timestamp: None`; those literals must be updated to compile after the BC change. Proof soundness is unaffected. |
| VP-016 (mitre-tactic-grouping-order) | Harness in VP-016 constructs `Finding` literals with `timestamp: None`. Must update to compile but proof is unaffected. |
| VP-006 (http-poison-monotonicity) | Calls `analyzer.on_data(...)` with 4 args. Signature update required; no proof logic change. |
| All other VPs | No impact — none reference the on_data signature or Finding.timestamp field content. |

**New VP proposed: VP-021 — timestamp-provenance-threading**

```
VP-021: Finding.timestamp Reflects on_data-Provided Timestamp

Property: For any Finding emitted from a flow-data path, Finding.timestamp == Some(ts)
where ts derives from the on_data timestamp argument passed by the reassembler.

Proof method: proptest / integration test (not Kani — involves DateTime conversion
and HashMap state; not a pure arithmetic invariant amenable to model checking).

Feasibility: HIGH. End-to-end integration test: craft a pcap with known ts_sec values,
run full pipeline, assert Finding.timestamp matches expected DateTime<Utc> value.
```

---

## 6. Affected Stories and Tests

### 6.1 Existing tests that MUST change (trait-signature break)

All files that implement `StreamHandler::on_data` must update their signature. Compilation fails
until all implementors are updated.

| Test File | Handler | Line | Required Change |
|-----------|---------|------|-----------------|
| `tests/reassembly_engine_tests.rs` | `RecordingHandler::on_data` | `:51` | Add `timestamp: u32` param. `data_events` tuple may need `u32` added if timestamp assertions are added. |
| `tests/hs043_flow_expiry_tests.rs` | inline `on_data` | `:69` | Add `_timestamp: u32` (unused param) |

**Inline VP proof harnesses in `src/reassembly/mod.rs` and `src/dispatcher.rs`:**
These are `#[cfg(kani)]`-gated and do not affect `cargo test`, but do affect `cargo kani`.
Any `Finding` literal constructions in those harnesses with `timestamp: None` must be verified
to still compile (they will — `timestamp: None` remains valid for the summary site and for
symbolic testing).

### 6.2 Existing stories in the regression risk zone

Stories whose implementation touches the `StreamHandler` trait surface:

| Story | File(s) touched | Risk |
|-------|----------------|------|
| STORY-042/043 (reassembly engine) | reassembly/mod.rs, lifecycle.rs | HIGH — core flush paths |
| STORY-044 (dispatcher) | dispatcher.rs | HIGH — on_data forwarding |
| STORY-052/053 (HTTP analyzer) | analyzer/http.rs | HIGH — on_data impl |
| STORY-056/057/058 (TLS analyzer) | analyzer/tls.rs | HIGH — on_data impl |
| STORY-069/070 (findings/reporters) | findings.rs, reporter/ | MEDIUM — field already exists |
| STORY-076 (VP-003 Kani proofs) | reassembly/mod.rs kani_proofs | LOW — gated behind cfg(kani) |
| STORY-079/080 (BC-2.04.013/flow expiry) | reassembly/mod.rs, lifecycle.rs | MEDIUM — close_flow paths |

### 6.3 New stories/ACs required (for F3 decomposition)

**Story A — Trait and flush-path wiring (SS-04, CRITICAL):**
- AC1: `StreamHandler::on_data` compiles with new `timestamp: u32` parameter.
- AC2: `RecordingHandler` in tests compiles and all existing tests pass unchanged.
- AC3: `flush_contiguous_data` passes current-packet timestamp to `handler.on_data`.
- AC4: `close_flow` passes `flow.last_seen` to `handler.on_data` for both-direction flushes.
- AC5: `StreamDispatcher::on_data` forwards the `timestamp` parameter to the downstream
  analyzer's `on_data`.

**Story B — Analyzer state and emission-site wiring (SS-06 HTTP, SS-07 TLS, CRITICAL):**
- AC1: `HttpAnalyzer` stores a per-flow last-seen timestamp in `on_data`; emits `Some(ts)` at
  all 9 HTTP emission sites.
- AC2: `TlsAnalyzer` stores a per-flow last-seen timestamp in `on_data`; emits `Some(ts)` at
  all 7 TLS emission sites.
- AC3: All 22 emission sites compile with new timestamp assignment.
- AC4: The segment-limit summary finding in `finalize` retains `timestamp: None`.

**Story C — E2E integration test (SS-09, HIGH):**
- AC1: Craft a synthetic pcap with a packet at `ts_sec=1_000_000`; run full analyze pipeline
  with HTTP or TLS analyzer; assert at least one `Finding.timestamp == Some(DateTime::from_timestamp(1_000_000, 0).unwrap())`.
- AC2: JSON output from `--json` contains `"timestamp": "1970-01-12T13:46:40Z"` (ISO-8601 for
  epoch 1_000_000) in at least one finding.
- AC3: `Finding.timestamp` is absent from JSON for a finding that genuinely has `None`
  (the segment-limit summary finding, if produced).
- AC4: BC-2.09.007 and BC-2.04.055 spec documents are written and pass consistency validation.

---

## 7. Regression Risk Assessment

| Module | Risk Level | Rationale |
|--------|-----------|-----------|
| `src/reassembly/handler.rs` (trait) | HIGH | Public trait; any implementor not updated fails to compile. Full regression suite must pass. |
| `src/reassembly/mod.rs` | HIGH | Core hot path; `flush_contiguous_data` is called on every non-empty packet. Incorrect timestamp threading causes wrong values in all Findings. |
| `src/reassembly/lifecycle.rs` | HIGH | `close_flow` is the sole path for FIN/RST/timeout/eviction flushes. `flow.last_seen` access must be correct. |
| `src/dispatcher.rs` | HIGH | Must forward the `timestamp` parameter unchanged. A mistake here silently passes zero or stale timestamps to analyzers. |
| `src/analyzer/http.rs` | HIGH | 9 emission sites. Per-flow state storage introduces new mutable state; cross-flow isolation (VP-014) must still hold. |
| `src/analyzer/tls.rs` | HIGH | 7 emission sites. Same risk as HTTP. |
| `tests/reassembly_engine_tests.rs` | MEDIUM | 16,090-line test file; signature break forces compile fix but test logic unchanged. New timestamp assertions are additive. |
| `src/findings.rs` | LOW | No change needed; field already exists with correct type and serde attribute. |
| Reporters (json, csv, terminal) | LOW | `Some(ts)` will now appear; serde handles serialization automatically. Verify CSV column if timestamp was previously always blank. |

**Safety net:** The full test suite (`cargo test --all-targets`) is the primary regression guard.
The trait-signature change is a compile-time guarantee — the build cannot pass if any implementor
is missed. All ~16,000 lines of reassembly tests will run against the updated engine. The
mutation-hardened reassembly suite (Phase 6) provides additional confidence on the flush paths.

**Key regression risk — cross-flow isolation (VP-014):** If `HttpAnalyzer` stores per-flow
timestamp in a `HashMap<FlowKey, u32>`, the isolation invariant must be verified: a timestamp
from flow A must not appear in a Finding from flow B. This is the same isolation property
VP-014 already covers for HTTP state; the F4 implementation must follow the same keying pattern.

---

## 8. Effort and Wave Sizing

### F2 Spec Evolution (1–2 days)

- Draft BC-2.09.007 (full behavioral contract)
- Draft BC-2.04.055 (full behavioral contract)
- Update BC-2.09.001, BC-2.09.006, BC-2.01.005 (remove O-01, update invariants)
- Draft VP-021 (timestamp-provenance-threading, proptest strategy)
- Update ARCH-INDEX and VP-INDEX with new VP

### F3 Story Decomposition (0.5 days)

- Story A: Trait + flush-path wiring (3–4 ACs, CRITICAL)
- Story B: Analyzer state + emission-site wiring (4 ACs, CRITICAL)
- Story C: E2E integration test + BC docs (4 ACs, HIGH)

### F4 TDD Implementation (3–5 days)

- Implement in order: A → B → C (A is the foundation; B and C depend on A).
- Each story: worktree → write failing tests → implement → `cargo test --all-targets` green →
  `cargo clippy -- -D warnings` green → `cargo fmt --check` green → PR → review → merge.
- Recommended: single PR per story (3 PRs total) to keep diffs reviewable.

### Total estimated effort

5–8 days end-to-end including spec, stories, and implementation. No architecture change needed;
no new external dependencies; no Kani proof work required (VP-021 uses integration tests).

---

## 9. Files Changed Summary Table

| File | Change Type | Reason |
|------|------------|--------|
| `src/reassembly/handler.rs` | MODIFIED | Add `timestamp: u32` to `StreamHandler::on_data` trait |
| `src/reassembly/mod.rs` | MODIFIED | `flush_contiguous_data` passes timestamp; anomaly Finding sites use it; `timestamp: None` remains on segment-limit summary |
| `src/reassembly/lifecycle.rs` | MODIFIED | `close_flow` passes `flow.last_seen`; emission sites use it |
| `src/dispatcher.rs` | MODIFIED | `on_data` forwards timestamp to downstream analyzers |
| `src/analyzer/http.rs` | MODIFIED | `on_data` stores per-flow ts; 9 emission sites set `Some(ts)` |
| `src/analyzer/tls.rs` | MODIFIED | `on_data` stores per-flow ts; 7 emission sites set `Some(ts)` |
| `tests/reassembly_engine_tests.rs` | MODIFIED | `RecordingHandler::on_data` signature update; new E2E test added |
| `tests/hs043_flow_expiry_tests.rs` | MODIFIED | Inline `on_data` signature update |
| `.factory/specs/behavioral-contracts/ss-09/BC-2.09.001.md` | MODIFIED | Remove O-01; update invariants |
| `.factory/specs/behavioral-contracts/ss-09/BC-2.09.006.md` | MODIFIED | timestamp now appears in JSON |
| `.factory/specs/behavioral-contracts/ss-01/BC-2.01.005.md` | MODIFIED | O-01 resolved |
| `.factory/specs/behavioral-contracts/ss-09/BC-2.09.007.md` | NEW | Finding.timestamp provenance BC |
| `.factory/specs/behavioral-contracts/ss-04/BC-2.04.055.md` | NEW | on_data timestamp parameter BC |
| `.factory/specs/verification-properties/vp-021-timestamp-provenance-threading.md` | NEW | VP-021 |
| `.factory/specs/verification-properties/VP-INDEX.md` | MODIFIED | Add VP-021 |

**NOT CHANGED:** `src/findings.rs`, `src/reader.rs`, `src/main.rs`, `src/reassembly/flow.rs`,
`src/reassembly/config.rs`, `src/reassembly/stats.rs`, `src/reassembly/segment.rs`,
`src/decoder.rs`, `src/reporter/json.rs`, `src/reporter/terminal.rs`, all Kani proof semantics.

---

## 10. Recommended F2 → F4 Plan

```
F2 (Spec Evolution):
  - Write BC-2.09.007 and BC-2.04.055 (full contracts with ACs, invariants, test vectors)
  - Update BC-2.09.001 (remove O-01, update invariant 1 from "all 22 = None" to "21 = Some, 1 = None")
  - Update BC-2.09.006 (EC-005 becomes positive; timestamp key now appears in JSON)
  - Update BC-2.01.005 (O-01 cross-ref removed)
  - Write VP-021 with proptest strategy skeleton
  - Update VP-INDEX.md with VP-021 entry
  - Update verification-architecture.md and verification-coverage-matrix.md for VP-021

F3 (Story Decomposition):
  - STORY-097: Trait + flush-path wiring (Story A ACs; implements BC-2.04.055)
  - STORY-098: Analyzer state + emission-site wiring (Story B ACs; implements BC-2.09.007 partial)
  - STORY-099: E2E integration test + BC-2.09.007 full verification (Story C ACs)

F4 (TDD Implementation):
  - Deliver STORY-097 → STORY-098 → STORY-099 in sequence
  - Each story: failing test first → implement → full suite green → PR
  - Regression: `cargo test --all-targets` must stay green throughout
```

---

## 11. Human Approval Gate

This delta analysis requires explicit human approval before proceeding to F2.

Open questions for human resolution:

1. **Timestamp precision:** Should `timestamp_usecs` also be wired (for sub-second precision)?
   The `on_data` parameter is `u32` seconds. Adding microseconds requires a second parameter or
   a custom type. Recommend: wire only `timestamp_secs` (seconds precision) for this feature;
   sub-second precision is a follow-on enhancement with a separate cost.

2. **Per-flow timestamp storage in analyzers:** Should the per-flow timestamp map in
   `HttpAnalyzer` and `TlsAnalyzer` use `FlowKey` as key (matching existing per-flow state
   patterns), or should the timestamp be passed through a different mechanism? The
   `HashMap<FlowKey, u32>` approach is consistent with existing state management.

3. **CSV reporter:** The CSV reporter currently emits `Finding` fields. Does `timestamp` need
   to appear as a CSV column? If so, Story C should include a CSV assertion. Current status
   unknown — needs a quick inspection of `src/reporter/csv.rs`.

4. **Segment-limit summary finding:** Confirm that `timestamp: None` for this one aggregate
   finding is acceptable to the issue author. Alternative: use `last packet's timestamp` from
   the finalize call, but `finalize` currently takes no timestamp parameter.

---

## 12. Revision Notes

| Rev | Date | Change |
|-----|------|--------|
| r1 | 2026-06-08 | Initial draft |
| r2 | 2026-06-08 | F5 ADV-F5-HIGH-001 — corrected canonical ts_sec=1_000_000 vector in §6.3 Story C AC2 from `2001-09-08T21:46:40Z` (the 1-billion-second epoch value) to arithmetically-correct `1970-01-12T13:46:40Z` (the 1-million-second epoch value) |
