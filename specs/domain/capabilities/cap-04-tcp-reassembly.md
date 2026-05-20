---
artifact: L2-cap-04
traces_to: ../domain-spec.md
cap_id: CAP-04
title: TCP Stream Reassembly
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
---

# CAP-04: TCP Stream Reassembly

## What the system does today

`TcpReassembler` (E-20) maintains a stateful BTreeMap-based reassembly buffer per TCP flow
direction. It reorders out-of-order segments, enforces the first-wins overlap policy for
conflicting retransmissions, and delivers contiguous in-order data to a `StreamHandler` via
`on_data` callbacks.

The engine is split across `reassembly/mod.rs` (~691 LOC), `config.rs`, `stats.rs`, and
`lifecycle.rs` (P2.01 / #85). The original 565-LOC `mod.rs` god-module smell is partially
closed: config and stats are fully extracted; the hot path remains in `mod.rs`.

**Sources:** C-6..C-9 (reassembly/mod.rs, config.rs, stats.rs, lifecycle.rs, flow.rs,
segment.rs, handler.rs). BC-RAS-001..054.

## Configuration (E-18 ReassemblyConfig)

All defaults from `ReassemblyConfig::default()` (config.rs). All five anomaly fields are
also CLI-overridable (LESSON-P2.05 / #88 / #92 / #93 / #96):

| Field | Default | CLI flag | Purpose |
|---|---|---|---|
| max_depth | 10 MB | --reassembly-depth | Max reassembled bytes per flow direction |
| memcap | 1 GB | --reassembly-memcap | Total buffered bytes before eviction |
| flow_timeout_secs | 300 s | (none) | Idle flow expiry |
| max_flows | 100,000 | (none) | Flow table size cap before LRU eviction |
| max_segments_per_direction | 10,000 | (none) | BTreeMap size guard per direction |
| max_receive_window | 1 MB | (none) | Out-of-window detection threshold |
| overlap_alert_threshold | 50 | --overlap-threshold (0-255) | Per-direction overlap anomaly threshold |
| small_segment_alert_threshold | 100 | --small-segment-threshold (0-2048) | Consecutive-run length before alert |
| small_segment_max_bytes | 16 | --small-segment-max-bytes (0-2048) | Payload bytes below which a segment is "small" |
| small_segment_ignore_ports | [23, 513] | --small-segment-ignore-ports | Ports exempt from small-segment detection |
| out_of_window_alert_threshold | 100 | --out-of-window-threshold | Per-direction out-of-window threshold |

Range validation on the five threshold CLI flags is enforced at parse time (#96; sane-range
process gap closed).

## Segment insertion and overlap policy

`FlowDirection::insert_segment` classifies each new segment via `InsertResult` (E-13):

| Result | Meaning | Action |
|---|---|---|
| Inserted | No overlap | Buffered in BTreeMap |
| Duplicate | Exact byte-range already present | Discarded; stats.segments_duplicates++ |
| PartialOverlap | New segment covers only gap bytes | Gap bytes added; overlap bytes discarded |
| ConflictingOverlap | Full overlap, different bytes | Finding emitted (Anomaly/Likely/High, T1036); first-wins |
| Truncated | Would exceed max_depth | DepthExceeded finding emitted |
| DepthExceeded | Already at depth limit | Finding emitted; segment dropped |
| SegmentLimitReached | BTreeMap size == max_segments_per_direction | Segment dropped; count tracked for finalize summary |
| OutOfWindow | Seq outside (base_offset, base_offset + max_receive_window) | Finding emitted (Anomaly/Inconclusive/Low, no MITRE) |
| IsnMissing | ISN not yet set | Silent drop with one-shot eprintln; programming error (VO-12) |

**First-wins overlap policy (INV-3):** only ConflictingOverlap produces a finding.
PartialOverlap quietly fills gaps.

## Anomaly detection design

Alerts are per-direction with one-shot latches (alert fires at most once per direction per
flow). Worst-case per bidirectional flow = 6 findings (3 types x 2 directions).

**Overlap detector:** cumulative count of overlapping segments per direction; fires when
`overlap_count > overlap_alert_threshold`. Emits T1036.

**Small-segment detector (redesigned #92/#93):** counts CONSECUTIVE runs of undersized
segments (payload < `small_segment_max_bytes` bytes). A normal-sized segment resets the
run counter to zero. This is a consecutive-run model (analogous to Snort's
`stream_tcp.small_segments`) rather than the prior cumulative-count design (which never
fired in practice). Flows where either endpoint port is in `small_segment_ignore_ports`
are entirely exempt (default: telnet/23, rlogin/513). Emits T1036.

**Out-of-window detector:** cumulative count of segments with seq outside the forward
receive window; fires when count > `out_of_window_alert_threshold`. No MITRE tag.

**Calibration status (domain-debt O-03):** These thresholds are documented engineering
defaults, not values calibrated against labelled traffic. A research pass (P2.05 / #88)
confirmed no production NIDS ships a directly-comparable count-based threshold with an
endorsed default value.

## Finalize contract and impl Drop tripwire (INV-7)

`TcpReassembler::finalize(&mut self, handler)` must be called exactly once.
A `finalized: bool` latch prevents double-execution. Finalize:
1. Closes all remaining open flows with `CloseReason::Timeout`.
2. Emits one aggregate Anomaly finding (bypassing MAX_FINDINGS cap, BC-RAS-054) if
   `segments_segment_limit > 0`.
3. Sets `finalized = true`.

`impl Drop for TcpReassembler` (#72 / LESSON-P0.03) is a lifecycle tripwire: if the
reassembler is dropped without `finalize()` having been called, it emits a one-shot
eprintln naming how many flows and bytes were discarded. The `run_analyze` IIFE pattern
in main.rs guarantees `finalize` is reached before any `Err` escapes the function.

## MAX_FINDINGS cap and dropped_findings counter

`const MAX_FINDINGS: usize = 10_000;` When this cap is reached, new reassembly-engine
findings are suppressed. `ReassemblyStats.dropped_findings: u64` (#73 / LESSON-P1.01) is
incremented each time a finding is suppressed, making the cap hit observable via
`summarize().detail["dropped_findings"]`. The finalize bypass (BC-RAS-054) is the only
path that pushes unconditionally past the cap.

The `HttpAnalyzer.all_findings` and `TlsAnalyzer.all_findings` vecs are NOT subject to
this cap. Only the reassembly engine enforces MAX_FINDINGS.

## Memory eviction

When total buffered bytes approach the memcap, `evict_flows` closes the least-recently-seen
non-Established flows (LRU policy) with `CloseReason::MemoryPressure`. The `evictions: u64`
stat tracks eviction count.

## BC references

BC-RAS-001..054 (54 contracts). Key: BC-RAS-022 (per-direction alert latch), BC-RAS-036/037
(first-wins overlap), BC-RAS-018 (ConflictingOverlap finding), BC-RAS-054 (finalize cap-bypass).

## NFR references

NFR-RES-001 (MAX_FINDINGS=10,000), NFR-RES-022 (dropped_findings counter -- now closed by
#73), NFR-REL-001..003 (overflow-checks, 12 saturating arithmetic sites).
