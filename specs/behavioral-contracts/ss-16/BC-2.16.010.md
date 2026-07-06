---
document_type: behavioral-contract
level: L3
version: "1.9"
status: draft
producer: product-owner
timestamp: 2026-06-12T02:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-16
capability: CAP-16
lifecycle_status: active
introduced: v0.7.0-feature-arp
modified:
  - "v1.3: Pass-4 remediation F-B4-M04: canonical vector row 2 self-contradiction resolved — removed '3 additional malformed frames (total 3 malformed frames)' clause; replaced with 'no other-opcode frames'; malformed_findings:3/malformed_frames:3 already encodes intent. — 2026-06-12"
  - "v1.4: Pass-6 remediation F-B6-H02: Invariant 5 added — malformed_findings <= malformed_frames; equality holds only when --arp is active (per ADR-008 Decision 7 key 11 and BC-2.16.009 PC4). No unconditional equality between the two counts. — 2026-06-12"
  - "v1.5: Pass-9 remediation F-B9-M04: EC-003 clarifying clause added — the 5 GARP and 3 spoof findings are detection classifications of frames already counted among the 100 request/reply frames; they are NOT additional frames; the reconciliation invariant counts frames, not findings. — 2026-06-12"
  - "v1.6: corpus-consistency-audit-2026-06-13 PR-1a: H1 updated to include '(11 Keys)' enrichment per Criterion-75 (title enrichment must live in H1, not only downstream indexes). — 2026-06-13"
  - "v1.7: F3 story-anchor back-fill (primary owner STORY-113; cross-story extension STORY-115 for storm_findings VALUE). — 2026-06-14"
  - "v1.8: fix-pc-013-014-015 PC-015 cross-reference — Related BCs updated to include BC-2.16.016 (ARP findings output is unbounded). Invariant 6 added: findings output from process_arp is NOT bounded by any MAX_FINDINGS constant; BC-2.16.016 is the authoritative contract for this invariant. Clarifies that the eleven summarize() keys do NOT include a dropped_findings key and MUST NOT gain one without a BC-2.16.010 version bump. — 2026-06-23"
  - "v1.9: Silent-limit audit — add two LRU-eviction observability counters. Key 12: `bindings_evicted` (u64) — count of ARP binding-table LRU evictions (MAX_ARP_BINDINGS=65,536); sourced from `ArpAnalyzer.bindings_evicted`. Key 13: `storm_counters_evicted` (u64) — count of ARP storm-counter LRU evictions (MAX_STORM_COUNTERS=4,096); sourced from `ArpAnalyzer.storm_counters_evicted` (see BC-2.16.008 v2.0 PC-6). Both counters are monotonically non-decreasing. IMPORTANT: BC-2.16.006 Invariant 3 ('eviction emits no Finding') is UNCHANGED — these are COUNTERS ONLY, not Findings. H1 updated 11 Keys → 13 Keys. Invariant 1 updated 11→13. — 2026-07-06"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md
  - .factory/specs/architecture/arp-architecture-delta.md
input-hash: TBD
---

# BC-2.16.010: ArpAnalyzer::summarize() Returns AnalysisSummary with Required Keys (13 Keys)

## Description

`ArpAnalyzer::summarize(&self) -> AnalysisSummary` produces an `AnalysisSummary` value
containing all thirteen required keys derived from `ArpAnalyzer` internal state. This method
is called once per `run_analyze()` invocation after all frames have been processed, and its
output is appended to `analyzer_summaries` in `main.rs` following the Modbus/DNP3 pattern.
The presence and content of all thirteen keys is the contract verified by this BC.
(Updated v1.1→v1.2: `other_opcode_count` added as key 4 per ADR-008 Decision 7 canonical
11-key set, resolving F-B-006. Reconciliation invariant stated explicitly. Updated v1.8→v1.9:
keys 12 and 13 added — `bindings_evicted` and `storm_counters_evicted` — two LRU-eviction
observability counters from the silent-limit audit. Both are COUNTERS ONLY; no Finding is
emitted for LRU eviction (BC-2.16.006 Invariant 3 unchanged).)

## Preconditions

1. `ArpAnalyzer::summarize()` is called after processing zero or more ARP frames.
2. The analyzer was created with `ArpAnalyzer::new(spoof_threshold, storm_rate)`.
3. No precondition on frame count — `summarize()` is valid even with zero frames processed.

## Postconditions

1. `summarize()` returns an `AnalysisSummary` containing the following `detail` keys
   (exact string names as specified; values are `u64` or compatible numeric type; listed
   in canonical order per ADR-008 Decision 7):
   - `"frames_analyzed"` — total number of well-formed Ethernet/IPv4 ARP frames processed
     (requests + replies + other opcodes). Malformed frames (extract_arp_frame → None,
     E-DEC-004) are NOT counted in `frames_analyzed`; they are counted separately in
     `malformed_frames`. (ARP-AMB-004 RESOLVED in F2.)
   - `"request_count"` — number of ARP frames with `operation == 1`
   - `"reply_count"` — number of ARP frames with `operation == 2`
   - `"other_opcode_count"` — number of ARP frames with `operation != 1` AND `operation != 2`
     (opcodes other than Request and Reply). These frames are counted in `frames_analyzed`
     but not in `request_count` or `reply_count`.
   - `"bindings_tracked"` — current `bindings.len()` at time of summarize call
   - `"spoof_findings"` — count of D1 spoof Findings emitted (MEDIUM + HIGH combined)
   - `"garp_findings"` — count of D2 GARP Findings emitted
   - `"storm_findings"` — count of D3 storm Findings emitted
   - `"mismatch_findings"` — count of D12 L2/L3 mismatch Findings emitted
   - `"malformed_findings"` — count of D11 malformed ARP Findings emitted
   - `"malformed_frames"` — count of ARP frames with non-Ethernet/IPv4 hw/proto sizes
     (extract_arp_frame → None); distinct from `malformed_findings` (a finding is only
     emitted when `--arp` is active, but the frame is still counted in `malformed_frames`
     regardless)
   - `"bindings_evicted"` — (key 12) total count of ARP binding-table LRU evictions over
     the lifetime of this analyzer run; sourced from `ArpAnalyzer.bindings_evicted: u64`;
     incremented by 1 each time `insert_binding_lru` evicts an entry because
     `bindings.len() == MAX_ARP_BINDINGS = 65,536`. Monotonically non-decreasing.
     **NO Finding is emitted for eviction** — this is a counter only (BC-2.16.006
     Invariant 3 is unchanged).
   - `"storm_counters_evicted"` — (key 13) total count of ARP storm-counter LRU evictions
     over the lifetime of this analyzer run; sourced from `ArpAnalyzer.storm_counters_evicted: u64`;
     incremented by 1 each time the storm_counters LRU eviction fires because
     `storm_counters.len() == MAX_STORM_COUNTERS = 4,096` (see BC-2.16.008 PC-6).
     Monotonically non-decreasing. **NO Finding is emitted for eviction** — this is a
     counter only (BC-2.16.008 Invariant 5 is unchanged).
2. `summarize()` NEVER panics.
3. `summarize()` is a `&self` method — it does not mutate any `ArpAnalyzer` state.
4. When zero frames have been processed, all thirteen keys are present with value 0.

## Invariants

1. **Thirteen required keys**: the exact key names above are the authoritative contract (updated
   from nine to ten in v1.1 by adding `malformed_frames`; updated from ten to eleven in v1.2
   by adding `other_opcode_count` per ADR-008 Decision 7 canonical 11-key set; updated from
   eleven to thirteen in v1.9 by adding `bindings_evicted` and `storm_counters_evicted` — two
   LRU-eviction observability counters from the silent-limit audit). The reporting pipeline
   (JSON reporter) renders these keys in the summary output. Any key rename or addition must
   update this BC.
2. **Additive/monotonic counts**: `frames_analyzed`, `request_count`, `reply_count`,
   `other_opcode_count`, `spoof_findings`, `garp_findings`, `storm_findings`,
   `mismatch_findings`, `malformed_findings`, `bindings_evicted`, and
   `storm_counters_evicted` are monotonically non-decreasing across the lifetime of the
   analyzer. `bindings_tracked` reflects current table size (can decrease due to LRU
   eviction). `malformed_frames` is also monotonically non-decreasing.
3. **Reconciliation invariant** (ARP-AMB-004 RESOLVED): `request_count + reply_count +
   other_opcode_count == frames_analyzed` holds exactly. Malformed frames (non-Ethernet/IPv4,
   extract_arp_frame → None) do NOT increment `frames_analyzed`; they increment `malformed_frames`
   only and are therefore excluded from the reconciliation identity. There is no ambiguity.
4. **ADR-008 Decision 7 compliance**: ARP findings and summary follow the existing pipeline.
   `AnalysisSummary` is the same type used by Modbus and DNP3 analyzers. No new reporter
   changes are required (ADR-008 §Decision 7).
5. **malformed_findings <= malformed_frames (conditional equality)**: `malformed_findings` is
   NEVER greater than `malformed_frames`. Equality (`malformed_findings == malformed_frames`)
   holds only when `--arp` is active — in that mode, every malformed frame produces a D11
   finding. When `--arp` is absent, `malformed_frames` still increments (the frame counter is
   unconditional) but no D11 finding is emitted, so `malformed_findings` remains lower. No
   invariant or test vector in this BC may assert unconditional equality between the two counts
   (per ADR-008 Decision 7 key 11 and BC-2.16.009 PC4).
6. **No dropped_findings key — findings output is unbounded (BC-2.16.016)**: The thirteen
   required keys specified in Postcondition 1 are EXHAUSTIVE. `summarize()` NEVER emits a
   `dropped_findings` key. The ARP findings output (from `process_arp`) is not bounded by
   any `MAX_FINDINGS` constant — unlike stream-reassembly analyzers (HTTP, TLS, Modbus, DNP3)
   which apply `MAX_FINDINGS = 10,000` via the reassembly layer. ARP bypasses reassembly
   entirely (BC-2.16.015 Invariant 2). BC-2.16.016 is the authoritative contract for the
   unbounded-findings invariant. Adding a `dropped_findings` key to summarize() would require
   a version bump of BOTH this BC (BC-2.16.010) AND BC-2.16.016.
7. **Eviction counters are COUNTERS ONLY — no Finding invariant preserved**: `bindings_evicted`
   and `storm_counters_evicted` count silent LRU evictions for observability. They are NOT
   Findings and do NOT trigger Finding emission. BC-2.16.006 Invariant 3 ("eviction does not
   emit a Finding") and BC-2.16.008 Invariant 5 ("storm counter cap — LRU eviction analogous
   to binding table") remain fully intact. Any implementation that emits a Finding on eviction
   is non-conformant with those BCs regardless of this counter's value.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Zero frames processed | All thirteen keys present; all values = 0 |
| EC-002 | Only ARP Requests processed (all op=1) | request_count = frames_analyzed; reply_count = 0; other_opcode_count = 0 |
| EC-003 | Mixed traffic: 50 requests, 50 replies, 5 GARPs, 3 spoofs | frames_analyzed=100; request_count=50; reply_count=50; other_opcode_count=0; garp_findings=5; spoof_findings=3 (the 5 GARP and 3 spoof findings are detection classifications of frames already counted among the 100 request/reply frames — they are NOT additional frames; the reconciliation invariant counts frames, not findings) |
| EC-004 | Binding table at MAX_ARP_BINDINGS (65,536) | bindings_tracked = 65,536 |
| EC-005 | ARP frame with operation=0 (undefined) | frames_analyzed incremented; other_opcode_count incremented; request_count and reply_count unchanged; reconciliation invariant holds: 0+0+1=1 |
| EC-006 | 3 malformed frames (extract_arp_frame → None) processed, 7 well-formed (5 requests, 2 replies) | frames_analyzed=7; request_count=5; reply_count=2; other_opcode_count=0; malformed_frames=3; reconciliation: 5+2+0=7 |
| EC-007 | Mixed: 4 requests, 3 replies, 2 other-opcode frames (op=5, op=6) | frames_analyzed=9; request_count=4; reply_count=3; other_opcode_count=2; reconciliation: 4+3+2=9 |
| EC-008 | 65,537 distinct source IPs processed (binding table eviction fires once) | bindings_evicted=1; bindings_tracked=65,536; no Finding emitted for eviction; all other keys reflect frame counts normally |
| EC-009 | 4,097 distinct source MACs each sending ARP frames (storm counter table eviction fires once) | storm_counters_evicted=1; no Finding emitted for eviction; all other keys reflect frame and finding counts normally |

## Canonical Test Vectors

| Analyzer state | Expected summary keys |
|---|---|
| new analyzer, no frames | {frames_analyzed:0, request_count:0, reply_count:0, other_opcode_count:0, bindings_tracked:0, spoof_findings:0, garp_findings:0, storm_findings:0, mismatch_findings:0, malformed_findings:0, malformed_frames:0, bindings_evicted:0, storm_counters_evicted:0} — all thirteen keys present |
| 10 Requests, 5 Replies, 2 GARP findings, 1 Spoof finding, 0 Storm, 0 Mismatch, 3 D11 malformed findings (3 malformed frames producing findings), no other-opcode frames | {frames_analyzed:15, request_count:10, reply_count:5, other_opcode_count:0, bindings_tracked:≥1, garp_findings:2, spoof_findings:1, storm_findings:0, mismatch_findings:0, malformed_findings:3, malformed_frames:3, bindings_evicted:0, storm_counters_evicted:0}; reconciliation: 10+5+0=15 ✓; malformed_frames excluded from frames_analyzed ✓ |
| 6 Requests, 4 Replies, 2 other-opcode frames (op=3), 0 findings, 0 malformed frames | {frames_analyzed:12, request_count:6, reply_count:4, other_opcode_count:2, bindings_tracked:≥1, spoof_findings:0, garp_findings:0, storm_findings:0, mismatch_findings:0, malformed_findings:0, malformed_frames:0, bindings_evicted:0, storm_counters_evicted:0}; reconciliation: 6+4+2=12 ✓ — exercises other_opcode_count > 0 |
| 5 well-formed frames (3 requests, 2 replies), 4 malformed frames (extract_arp_frame → None) | {frames_analyzed:5, request_count:3, reply_count:2, other_opcode_count:0, bindings_tracked:≥1, spoof_findings:0, garp_findings:0, storm_findings:0, mismatch_findings:0, malformed_findings:0, malformed_frames:4, bindings_evicted:0, storm_counters_evicted:0}; reconciliation: 3+2+0=5 ✓; 4 malformed frames excluded from frames_analyzed |
| 65,537 distinct IPs (binding table eviction fires once) then summarize | {bindings_tracked:65536, bindings_evicted:1, storm_counters_evicted:0, …} — bindings_evicted incremented to 1 after first LRU eviction; no Finding for eviction |
| 4,097 distinct source MACs each sending 1 frame (storm counter table eviction fires once) then summarize | {storm_counters_evicted:1, bindings_evicted:0, …} — storm_counters_evicted incremented to 1 after first storm-counter LRU eviction; no Finding for eviction |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none — summarize is a deterministic aggregation, not a formal-verification target) | Thirteen keys present; values match internal counters; reconciliation invariant holds; no panic; bindings_evicted and storm_counters_evicted reflect LRU eviction counts | unit tests: process known sequence of frames (including eviction scenarios), call summarize(), assert each key and the reconciliation identity |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — the summarize() output is the analysis result surface for ARP Security Analysis; it feeds the CLI summary and JSON report that expose the capability's output to users |
| L2 Domain Invariants | (none directly) |
| Architecture Module | SS-16 (src/analyzer/arp.rs ArpAnalyzer::summarize, C-23); ADR-008 Decision 7 |
| Stories | STORY-113 (primary owner — keys defined here); STORY-115 (storm_findings VALUE, cross-story extension) |
| Feature | arp-security-analyzer |
| MITRE Techniques | (none — summary method, no finding emission) |

## Related BCs

- BC-2.16.004 — depends on (spoof_findings count sourced from ArpAnalyzer internal counter)
- BC-2.16.003 — depends on (garp_findings count)
- BC-2.16.006 — depends on (bindings_evicted count sourced from ArpAnalyzer.bindings_evicted; BC-2.16.006 Invariant 3 "eviction emits no Finding" is preserved — this BC adds an observability counter only)
- BC-2.16.008 — depends on (storm_findings count; storm_counters_evicted count sourced from ArpAnalyzer.storm_counters_evicted per BC-2.16.008 PC-6)
- BC-2.16.007 — depends on (mismatch_findings count)
- BC-2.16.009 — depends on (malformed_findings count)
- BC-2.16.016 — composes with (ARP findings output is unbounded — no MAX_FINDINGS cap; summarize() MUST NOT add a dropped_findings key without bumping both this BC and BC-2.16.016)

## Architecture Anchors

- `src/analyzer/arp.rs` — `impl ArpAnalyzer { pub fn summarize(&self) -> AnalysisSummary }`
- `src/analyzer/arp.rs` — `ArpAnalyzer.frames_analyzed: u64`, `request_count: u64`, `reply_count: u64` fields
- `src/analyzer/arp.rs` — `ArpAnalyzer.bindings_evicted: u64` (new field; incremented on each `insert_binding_lru` LRU eviction; surfaced as key `"bindings_evicted"` in `summarize()`)
- `src/analyzer/arp.rs` — `ArpAnalyzer.storm_counters_evicted: u64` (new field; incremented on each storm-counter LRU eviction; surfaced as key `"storm_counters_evicted"` in `summarize()`; see BC-2.16.008 PC-6)
- `src/main.rs` — `analyzer_summaries.push(arp_analyzer.summarize())` (analogous to Modbus/DNP3 pattern)
- `.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md §Decision 7`

## Story Anchor

STORY-113 (primary); STORY-115 (cross-story extension — storm_findings VALUE)

## VP Anchors

- (none)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 7 (original eleven summary keys, canonical set per v1.2 update: frames_analyzed, request_count, reply_count, other_opcode_count, bindings_tracked, spoof_findings, garp_findings, storm_findings, mismatch_findings, malformed_findings, malformed_frames); arp-architecture-delta.md §1 (ArpAnalyzer struct fields); v1.9 silent-limit audit adds keys 12–13: bindings_evicted, storm_counters_evicted |
| **Confidence** | high — key names are authoritative per ADR-008 Decision 7; counter fields are explicit in ArpAnalyzer struct definition |
| **Extraction Date** | 2026-06-12 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same analyzer state always produces same summary |
| **Thread safety** | single-threaded |
| **Overall classification** | pure read-only aggregation |
