---
document_type: behavioral-contract
level: L3
version: "1.6"
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
  - "v1.4: Pass-4 remediation F-B4-M01 (canonical vector row 4: pinned intra-second distribution to 25@ts=100 then 25@ts=101); F-B4-M02 (EC-011: contrast note vs EC-002 added); F-B4-M06 (Invariant 6: storm_counter re-initialization after LRU eviction specified; analogous to BC-2.16.005 Invariant 4). — 2026-06-12"
  - "v1.5: Pass-5 remediation F-B5-L02 (explicit ordered step sequence for intra-frame processing: Step 1=window-expiry check, Step 2=increment, Step 3=rate evaluation — mirrors BC-2.16.004 Step pattern); F-B5-L01 (PC2 extended: first-observation of a never-before-seen MAC initializes count_in_window=1, window_start_ts=timestamp_secs, storm_emitted=false, symmetric to Invariant 6); (LOW) Description 'per-MAC sliding window counter' corrected to 'per-MAC 60-second flap-window counter' to avoid contradiction with Invariant 2. — 2026-06-12"
  - "v1.6: Pass-12 corpus-cleanup F-B12-003: ARP_FLAP_WINDOW_SECS anchor added to Architecture Anchors (BC-2.16.008 uses ARP_FLAP_WINDOW_SECS but the const was previously undeclared in this BC's anchors; defined in BC-2.16.004 and shared). — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md
  - .factory/specs/architecture/arp-architecture-delta.md
  - .factory/phase-f1-delta-analysis/mitre-arp-additional-detections.md
input-hash: TBD
---

# BC-2.16.008: D3 ARP Storm Rate Detection — Source MAC Exceeds ARP_STORM_RATE_DEFAULT Frames/Sec

## Description

`ArpAnalyzer` tracks ARP frame rate per source MAC using a per-MAC 60-second flap-window counter
(`storm_counters: HashMap<[u8; 6], StormCounter>`). When a source MAC's **average ARP frame
rate since `window_start_ts` within the 60-second flap window** reaches or exceeds
`ARP_STORM_RATE_DEFAULT = 50` frames per second, a single MEDIUM/Anomaly Finding is emitted
for that MAC and that window. The rate metric is an average-since-window-start computed as
`count_in_window / max(1, timestamp_secs - window_start_ts)` — not a sustained-rate detector.
The rate threshold is a wirerust engineering default — no authoritative industry numeric
standard exists for ARP storm detection rates (mitre-arp-additional-detections.md §4
CRITICAL CORRECTION: Snort/arpwatch/Zeek all use state-event logic without numeric rate
thresholds). No MITRE technique is attached to D3; T0814 (Denial of Service, ICS) is a
possible mapping but has not been validated live per DF-VALIDATION-001 as of 2026-06-12
and is therefore withheld until validated.

## Preconditions

1. `frame.sender_mac` is a 6-byte MAC address (non-broadcast in typical operation).
2. `timestamp_secs` is the packet timestamp in Unix seconds (u32).
3. `--arp` flag is active (analysis gate per BC-2.16.011).
4. `--arp-storm-rate` flag (BC-2.16.013) may override `ARP_STORM_RATE_DEFAULT`.

## Postconditions

**Intra-frame ordered step sequence (fixed order; must be implemented in this exact sequence):**

1. **Step 1 — window-expiry check and initialization:**
   - If `frame.sender_mac` is NOT in `storm_counters` (first-ever observation of this MAC):
     Initialize a new `StormCounter`: `count_in_window = 1`, `window_start_ts = timestamp_secs`,
     `storm_emitted = false`. Proceed to Step 3 (no further increment — count already set to 1).
   - If `frame.sender_mac` IS in `storm_counters` AND
     `timestamp_secs - window_start_ts > ARP_FLAP_WINDOW_SECS`: window expired — reset:
     `count_in_window = 1`, `window_start_ts = timestamp_secs`, `storm_emitted = false`.
     Proceed to Step 3 (count reset to 1; no additional increment).
   - Otherwise (existing entry, window still active): proceed to Step 2.

2. **Step 2 — increment:** `count_in_window` is incremented by 1 for this frame (window is
   active and entry already exists).

3. **Step 3 — rate evaluation:** The rate is evaluated using `timestamp_secs` of the frame
   just processed as the "current time" in the denominator. When
   `count_in_window / max(1, timestamp_secs - window_start_ts) >= storm_rate` AND
   `storm_emitted == false` for a given source MAC, a Finding is emitted:
   - `confidence: MEDIUM`
   - `finding_type: Anomaly`
   - `description` indicating ARP storm / high ARP frame rate from source MAC
   - `mitre_techniques: []` (empty — no MITRE tag per DF-VALIDATION-001; T0814 withheld)
   - Evidence includes: `source_mac`, `frame_count`, `window_secs`, `rate_pps`
4. `storm_emitted` is set to `true` after a storm Finding is emitted for this MAC and window
   (one-shot guard: at most one storm Finding per MAC per window).
5. `storm_counters.len() <= MAX_STORM_COUNTERS = 4,096` at all times (LRU eviction analogous
   to binding table cap per BC-2.16.006).

**Note on rate calculation — same-second case (ARP-AMB-003 RESOLVED in F2):**
6. Timestamps are integer seconds (`u32`). The rate formula is
   `rate = count_in_window / max(1, timestamp_secs - window_start_ts)`.
   - When `timestamp_secs == window_start_ts` (all frames in the same integer second),
     `max(1, 0) = 1`, so `rate = count_in_window`. A count >= storm_rate within the same
     second triggers the storm finding (e.g., 50 frames at ts=100 → rate=50 → triggers at
     storm_rate=50).
   - When frames span 2 or more integer seconds, the denominator is the elapsed seconds
     (e.g., 100 frames from ts=100 to ts=101 → elapsed=1 → rate=100; 100 frames from
     ts=100 to ts=102 → elapsed=2 → rate=50).
   - There is no sub-second ambiguity: timestamps are coarse integer seconds. The `+1`
     denominator from the previous (defective) formula was incorrect for the 2-second burst
     case (e.g., 50 frames from ts=100 to ts=101 would have produced rate=50/2=25, missing
     the threshold; the correct `max(1, 101-100)=1` gives rate=50/1=50, correctly triggering).
   ARP-AMB-003 is RESOLVED as of F2. The formula is fully determined by u32 integer-seconds
   semantics; there is no remaining sub-second ambiguity.

## Invariants

1. **One-shot per window**: `storm_emitted = true` prevents repeated storm Findings for the
   same MAC in the same window. This avoids alert fatigue. The guard resets when the window
   expires.
2. **Rate metric is average-since-window-start, not a sustained-rate detector**: The formula
   `count_in_window / max(1, timestamp_secs - window_start_ts)` computes the average ARP
   frame rate since `window_start_ts` within the 60-second flap window. A late burst (e.g.,
   49 frames at ts=100, then 50 more frames at ts=159 within the same window) or a burst
   that spans two integer-second boundaries may be suppressed because the denominator
   dilutes the short-term rate. This is ACCEPTED behavior as of v0.7.0 (a known limitation,
   not a bug) — the metric is a simple average-since-window-start, not a peak-rate or
   sliding-window rate detector. Operators requiring finer granularity should lower
   `--arp-storm-rate` accordingly. `ARP_STORM_RATE_DEFAULT = 50` frames/sec is a wirerust
   engineering default; not derived from any external standard
   (mitre-arp-additional-detections.md §4b).
3. **No MITRE technique tag (DF-VALIDATION-001 compliance)**: D3 does NOT carry T0814 (Denial
   of Service) because T0814 was NOT validated live in the research pass. Per project policy
   DF-VALIDATION-001, no technique tag is attached until the technique ID is validated live
   against attack.mitre.org. The D3 Finding emits an empty `mitre_techniques: []`. This is a
   documented human decision point (ADR-008 §Decision 5 D3 entry; §Open Items).
4. **Benign overlap**: high ARP rate from misconfigured devices, system startup storms, and
   ICS scan-on-boot events is common. MEDIUM/Anomaly is the correct severity; false positives
   are expected and documented.
5. **Storm counter cap**: `MAX_STORM_COUNTERS = 4,096` is a wirerust engineering default
   bounding memory for per-MAC storm tracking. LRU eviction analogous to binding table.
6. **Storm counter re-initialization after eviction**: When a previously-evicted MAC reappears,
   a new `StormCounter` is initialized as a first-time observation: `count_in_window = 1`,
   `window_start_ts = timestamp_secs`, `storm_emitted = false`. The prior state for that MAC
   was discarded at eviction. Analogous to BC-2.16.005 Invariant 4 for the binding table.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Source MAC sends 49 frames all at ts=100 (window_start_ts=100, same integer second) | No storm finding: rate=49/1=49 < storm_rate=50 |
| EC-002 | Source MAC sends exactly 50 frames all at ts=100 (same integer second, window_start_ts=100) | Storm finding emitted: `max(1, 100-100)=1`; rate=50/1=50 >= storm_rate=50 |
| EC-003 | Source MAC sends 51st frame within same window (storm_emitted=true) | No additional storm finding (one-shot guard active) |
| EC-004 | Storm window expires (>60s since first frame) | Window resets; storm_emitted=false; counter resets |
| EC-005 | `--arp-storm-rate 10` set: storm threshold lowered | Storm triggers at 10 frames/sec instead of 50 |
| EC-006 | `--arp-storm-rate 0` set (edge case) | Treat as "always storm" or "no storm detection" — implementation should clamp to minimum 1; document as invalid value |
| EC-007 | 4,097 distinct MACs each sending frames | MAX_STORM_COUNTERS cap: LRU eviction on 4,097th MAC; oldest MAC counter evicted; no storm finding for evicted MAC even if it later storms |
| EC-008 | Same MAC, all frames in same second (ts==window_start_ts) | Rate calculated as count/1; avoids divide-by-zero; finding emits if count >= storm_rate |
| EC-009 | ts - window_start_ts == 60 exactly (boundary): frame at ts=160, window_start_ts=100, count_in_window=50 | `160-100=60 <= ARP_FLAP_WINDOW_SECS=60`; still in-window per the <= boundary; no reset; rate=50/60≈0.83 < 50; no storm finding |
| EC-010 | ts - window_start_ts == 61 (one second past window): frame at ts=161, window_start_ts=100, count_in_window=3000 | `161-100=61 > 60`; window resets: count_in_window=1, window_start_ts=161, storm_emitted=false; no storm finding emitted for the huge prior count (window expired before detection) |
| EC-011 | Late-burst suppression (ACCEPTED v0.7.0 limitation): 49 frames at ts=100 then 50 more at ts=159 (same window, window_start_ts=100) | At ts=159: count_in_window=99, elapsed=59; rate=99/59≈1.68 < 50; NO storm finding despite 50-frame burst within 1s of ts=159. Suppressed by window-averaging. Documented accepted limitation per Invariant 2. Contrast EC-002: the same 50-frame same-second burst fires when window_start_ts equals the burst second (denominator=1); here window_start_ts=100 dilutes the denominator to 59. This is the accepted average-since-window-start limitation (Invariant 2). |

## Canonical Test Vectors

Rate formula: `rate = count_in_window / max(1, ts - window_start_ts)`.

| Source MAC | Frames | window_start_ts | last ts | storm_rate | rate calculation | Expected outcome |
|---|---|---|---|---|---|---|
| AA:BB:CC:DD:EE:FF | 50, all at ts=100 | 100 | 100 | 50 | 50/max(1,0)=50/1=50 >= 50 | Storm Finding emitted |
| AA:BB:CC:DD:EE:FF | 49, all at ts=100 | 100 | 100 | 50 | 49/max(1,0)=49/1=49 < 50 | No storm finding |
| AA:BB:CC:DD:EE:FF | 100, ts=100..ts=159 (60s span) | 100 | 159 | 50 | 100/max(1,59)=100/59≈1.69 < 50 | No storm finding (spread over window) |
| AA:BB:CC:DD:EE:FF | 50 frames: 25 at ts=100 (rate peaks at 25/max(1,0)=25 — no fire), then 25 at ts=101 (count=50, elapsed=ts=101-window_start_ts=100=1, rate=50/max(1,1)=50/1=50 >= 50 — fires at the 50th frame) | 100 | 101 | 50 | elapsed=1; rate=50/1=50 >= 50 — rate evaluated after the 50th frame at ts=101 is processed | Storm Finding emitted; denominator=max(1,1)=1 (elapsed=1 second, not 2) |
| AA:BB:CC:DD:EE:FF | 3000 frames at ts=100 | 100 | 100 | 50 | storm fires at frame 50 (count=50>=50); storm_emitted=true | One storm finding; no additional findings after (one-shot guard) |
| 11:22:33:44:55:66 | 10, all at ts=200 | 200 | 200 | 10 (via --arp-storm-rate) | 10/max(1,0)=10/1=10 >= 10 | Storm Finding with custom threshold |
| AA:BB:CC:DD:EE:FF | 50 frames, last at ts=160, window_start_ts=100 | 100 | 160 | 50 | elapsed=160-100=60 <= 60 (in-window per <= boundary); rate=50/max(1,60)=50/60≈0.83 < 50 | No storm finding; ts-window_start_ts==60 is still in-window |
| AA:BB:CC:DD:EE:FF | count_in_window=3000 up to ts=160, then frame at ts=161, window_start_ts=100 | 100 | 161 | 50 | elapsed=161-100=61 > 60; window RESETS: count_in_window=1, window_start_ts=161, storm_emitted=false | No storm finding; window expired; prior count discarded; reset |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (no Kani/proptest for D3 — rate-counter logic verified by unit tests) | One-shot storm finding per window; correct frame counting; window reset; rate calculation | unit tests in src/analyzer/arp.rs tests module |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — ARP storm rate detection (D3) is a named detection in the ARP Security Analysis capability; high ARP volume from a single source is an indicator of flooding/DoS behavior or misconfigured device |
| L2 Domain Invariants | (none directly) |
| Architecture Module | SS-16 (src/analyzer/arp.rs ArpAnalyzer::process_arp, C-23); ADR-008 Decision 5 D3 |
| Stories | TBD (F3 story decomposition) |
| Feature | arp-security-analyzer |
| MITRE Techniques | NONE — T0814 withheld per DF-VALIDATION-001 (not validated live as of 2026-06-12) |

## Related BCs

- BC-2.16.013 — depends on (--arp-storm-rate overrides ARP_STORM_RATE_DEFAULT)
- BC-2.16.011 — depends on (--arp gate must be active)

## Architecture Anchors

- `src/analyzer/arp.rs` — `ArpAnalyzer.storm_counters: HashMap<[u8; 6], StormCounter>`
- `src/analyzer/arp.rs` — `struct StormCounter { count_in_window: u64, window_start_ts: u32, storm_emitted: bool }`
- `src/analyzer/arp.rs` — `const ARP_FLAP_WINDOW_SECS: u32 = 60` (wirerust engineering default; shared with D1/D2 flap detection — defined as authoritative in BC-2.16.004)
- `src/analyzer/arp.rs` — `const ARP_STORM_RATE_DEFAULT: u32 = 50` (wirerust engineering default — NOT an industry standard)
- `src/analyzer/arp.rs` — `const MAX_STORM_COUNTERS: usize = 4_096` (wirerust engineering default)
- `.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md §Decision 5` — D3 and D3 MITRE deferred note
- `.factory/specs/architecture/arp-architecture-delta.md §3.2, §3.3 D3`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- (none — D3 not a VP-024 formal target)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 5 (D3 scope, MITRE deferred, DF-VALIDATION-001); arp-architecture-delta.md §3.3 D3; mitre-arp-additional-detections.md §4 (ARP_STORM_RATE_DEFAULT=50 is wirerust engineering default; fabricated industry thresholds REJECTED); §3 D3 (T0814 not re-fetched, deferred) |
| **Confidence** | high for functional behavior; T0814 MITRE mapping withheld per DF-VALIDATION-001 |
| **Extraction Date** | 2026-06-12 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same sequence of frames and timestamps always produces same findings |
| **Thread safety** | single-threaded |
| **Overall classification** | stateful pure core |
