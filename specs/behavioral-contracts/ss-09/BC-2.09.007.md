---
document_type: behavioral-contract
level: L3
version: "1.1.1"
status: draft
producer: product-owner
timestamp: 2026-06-08T00:00:00Z
phase: 1a
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-09
capability: CAP-09
lifecycle_status: active
introduced: v0.2.0-feature-100
modified:
  - "v1.1: F5 ADV-F5-HIGH-001 — corrected canonical ts_sec=1_000_000 vector from 2001-09-08 to arithmetically-correct 1970-01-12T13:46:40Z (1_000_000_000 ≠ 1_000_000). — 2026-06-08"
  - "v1.1.1: PATCH — VP Anchor annotation updated: VP-021 status corrected from draft/unverified to verified @256a490 (F6 lock propagation, FINDING-005). No functional postcondition change. — 2026-06-09"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/feature-delta/issue-100-pcap-timestamps/delta-analysis.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.001.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.055.md
input-hash: adf5906
---

# BC-2.09.007: Finding.timestamp Carries Capture-Relative Pcap Timestamp from on_data Call Site

## Description

When a `Finding` is emitted from a flow-data path (either `flush_contiguous_data` or
`close_flow`), `Finding.timestamp` must be `Some(DateTime<Utc>)` derived from the `timestamp:
u32` argument passed by the reassembler to `StreamHandler::on_data`. The value is
CAPTURE-RELATIVE PROVENANCE: the pcap capture-host wall clock, possibly unsynchronized and
non-monotonic across reboots. It is NOT authoritative time (per NIST SP 800-86). The single
exception is the segment-limit summary finding emitted from `finalize`, which is a
post-capture aggregate not tied to any specific packet; that finding correctly retains
`timestamp: None`.

## Preconditions

1. A `Finding` is emitted from a flow-data path (hot-path flush via `flush_contiguous_data`
   or close-flush via `close_flow` — FIN, RST, timeout, or eviction).
2. The flow that produced the finding has seen at least one packet, so the reassembler
   holds a valid `u32` timestamp in scope at the `on_data` call site.
3. `RawPacket.timestamp_secs` is a valid `u32` pcap `ts_sec` value (guaranteed by the
   pcap_file crate reader; max valid value is `u32::MAX = 4_294_967_295`, corresponding to
   approximately 2106 CE).

## Postconditions

1. `Finding.timestamp = Some(DateTime::from_timestamp(ts_sec as i64, ts_usec * 1_000))`
   where `ts_sec` is the `u32` timestamp argument received by `StreamHandler::on_data` at
   the flush call site.
2. For hot-path flushes (`flush_contiguous_data`): `ts_sec` equals the current packet's
   `timestamp_secs` — the packet that triggered the flush is the provenance anchor.
3. For close-flush paths (`close_flow` — FIN, RST, timeout, eviction): `ts_sec` equals
   `TcpFlow.last_seen` — the timestamp of the most-recently-seen packet on the flow,
   captured before `self.flows.remove(key)` is called. This is the closest available
   temporal bound for the detected event.
4. The segment-limit summary finding emitted from `finalize` retains `timestamp: None`.
   This is correct behavior: the summary is a post-capture aggregate, not tied to any
   specific packet. See BC-2.04.054.
5. `Finding.timestamp`, when `Some`, serializes to ISO-8601 in JSON output via the existing
   `#[serde(skip_serializing_if = "Option::is_none")]` serde attribute. No change to
   `findings.rs` is required — the serde wiring already handles `Some(DateTime<Utc>)`
   correctly.
6. `Finding.timestamp` is absent from JSON when `None` (existing skip-serializing-if
   behavior; the segment-limit summary finding produces no `"timestamp"` key in JSON).

## Invariants

1. Exactly 21 of 22 production emission sites set `timestamp: Some(...)` after this feature
   is implemented. The 22nd site (segment-limit summary in `finalize`) retains `None`.
2. The `u32 → DateTime<Utc>` conversion via `DateTime::from_timestamp(ts_sec as i64, 0)` is
   lossless for all valid pcap `ts_sec` values. The max `u32` value maps to approximately
   2106 CE, which is within the `chrono` datetime range. No silent precision loss occurs.
3. CAPTURE-RELATIVE PROVENANCE framing: `Finding.timestamp` reflects the capture host's
   wall clock at packet-capture time. It is not synchronized to a reference clock, may be
   non-monotonic across reboots, and is subject to pcap capture-tool limitations. Per NIST
   SP 800-86 (Guide to Integrating Forensic Techniques into Incident Response), timestamps
   in captured evidence must be qualified with their provenance; wirerust satisfies this
   by documenting this invariant rather than silently presenting the value as authoritative.
4. Cross-flow isolation: the timestamp stored per-flow in `TlsAnalyzer` and `HttpAnalyzer`
   is keyed by `FlowKey` (consistent with existing per-flow state patterns). A timestamp
   from flow A must never appear in a Finding attributed to flow B. VP-014 (HttpAnalyzer
   Cross-Flow Isolation) already covers the isolation invariant for per-flow state; the
   per-flow timestamp map follows the same keying pattern.
5. The `timestamp_usecs` field from `RawPacket` is used for sub-second precision in the
   `DateTime` constructor (`nsecs = timestamp_usecs * 1_000`). If only seconds precision
   is wired, `nsecs = 0` is an acceptable fallback (sub-second precision is a follow-on
   concern per F1 open question 1).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Finding emitted when no timestamp has yet been received for a flow | Cannot occur: `TcpFlow::new` sets `last_seen` to the constructor timestamp argument; any flow that reaches `on_data` has a non-zero `last_seen` |
| EC-002 | Segment-limit summary finding (finalize aggregate) | `timestamp: None` — correct, not a gap. JSON omits the `"timestamp"` key per existing serde attribute |
| EC-003 | ts_sec = 0 (capture timestamp is Unix epoch) | `Finding.timestamp = Some(DateTime::from_timestamp(0, 0)) = Some(1970-01-01T00:00:00Z)` — valid and correctly serialized |
| EC-004 | ts_sec = u32::MAX = 4_294_967_295 | `Finding.timestamp = Some(DateTime::from_timestamp(4_294_967_295, 0))` — within chrono range (~2106 CE); conversion is lossless |
| EC-005 | JSON output for a finding with Some(ts) | `"timestamp": "1970-01-12T13:46:40Z"` for ts_sec=1_000_000 — ISO-8601 UTC format via chrono's serde integration |
| EC-006 | JSON output for the segment-limit summary finding | No `"timestamp"` key in JSON (skip_serializing_if = "Option::is_none") |
| EC-007 | Two concurrent flows with different timestamps | Each flow's Finding carries only that flow's most-recently-seen timestamp; no cross-contamination (VP-014 cross-flow isolation invariant applies) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| pcap packet at ts_sec=1_000_000 → HTTP path traversal detected → flush_contiguous_data called | `Finding.timestamp = Some(1970-01-12T13:46:40Z)` | happy-path |
| pcap packet at ts_sec=1_000_000 → flow FIN → close_flow called with flow.last_seen=1_000_000 | `Finding.timestamp = Some(1970-01-12T13:46:40Z)` | happy-path |
| finalize called after segment-limit exceeded | Segment-limit summary finding has `timestamp = None`; no `"timestamp"` key in JSON | edge-case |
| ts_sec=0 (epoch) packet emits finding | `Finding.timestamp = Some(1970-01-01T00:00:00Z)` | edge-case |
| Two flows A (ts=1000) and B (ts=2000) emit findings in same run | Finding from flow A has ts=1000; finding from flow B has ts=2000; no cross-contamination | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-021 | on_data timestamp is correctly threaded to Finding.timestamp with two-case semantics | integration test + proptest |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-09 ("Forensic finding emission") per domain/capabilities/cap-09-finding-emission.md |
| Capability Anchor Justification | CAP-09 ("Forensic finding emission") per domain/capabilities/cap-09-finding-emission.md — this BC defines the provenance and value invariants for `Finding.timestamp`, which is the forensic timestamp field of the core Finding output type; populating it is a direct extension of the finding-emission capability |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation — timestamp value is stored raw; no formatting at emission) |
| Architecture Module | SS-09 (findings.rs, C-14); SS-04 (reassembly/mod.rs:flush_contiguous_data, lifecycle.rs:close_flow); SS-06 (analyzer/http.rs); SS-07 (analyzer/tls.rs) |
| Stories | STORY-098, STORY-099 |
| Feature | issue-100-pcap-timestamps |
| Resolves | O-01 (domain-debt: timestamp always None) for 21 of 22 emission sites |

## Related BCs

- BC-2.09.001 — composes with (Finding struct schema; this BC constrains the `timestamp` field value)
- BC-2.09.006 — composes with (JSON serialization; `Some(ts)` now appears in JSON output)
- BC-2.04.055 — depends on (on_data timestamp parameter is the source of the value threading)
- BC-2.04.054 — related to (segment-limit summary bypass; the one site that retains timestamp: None)

## Architecture Anchors

- `src/findings.rs:119-146` — Finding struct; `timestamp: Option<DateTime<Utc>>` field with existing serde attribute
- `src/reassembly/mod.rs` — `flush_contiguous_data`; call site passes current-packet timestamp to `handler.on_data`
- `src/reassembly/lifecycle.rs` — `close_flow`; call site passes `flow.last_seen` to `handler.on_data`
- `src/analyzer/http.rs` — per-flow last-seen timestamp storage; 9 emission sites set `Some(stored_last_ts)`
- `src/analyzer/tls.rs` — per-flow last-seen timestamp storage; 7 emission sites set `Some(stored_last_ts)`

## Story Anchor

STORY-098 (analyzer state + emission-site wiring — Story B from F3 decomposition); STORY-099 (E2E integration test — Story C from F3 decomposition).

## VP Anchors

- VP-021 — timestamp-provenance-threading (integration test + proptest; verified @256a490)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | F1 delta-analysis §4.5 (emission sites) + §3 (timestamp semantics decision) |
| **Confidence** | high |
| **Extraction Date** | 2026-06-08 |

## Evidence Types Used

- **delta-analysis**: F1 analysis documents the 22 emission sites, two-path semantics, and NIST SP 800-86 framing
- **type constraint**: `DateTime::from_timestamp(u32 as i64, 0)` is infallible for all valid `u32` values

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (Finding construction is pure; I/O happens upstream in reader) |
| **Global state access** | none |
| **Deterministic** | yes — same ts_sec always produces same DateTime<Utc> |
| **Thread safety** | Send + Sync (Finding is an owned value) |
| **Overall classification** | pure |
