---
document_type: behavioral-contract
level: L3
version: "1.6"
status: draft
producer: product-owner
timestamp: 2026-06-10T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-15
capability: CAP-15
lifecycle_status: active
introduced: v0.6.0-feature-008
modified:
  - "v1.1: Pass-1 adversarial fix I-5 (companion update): clarified Invariant 5 — block_event_count is fed unconditionally by BC-2.15.014 (every block-timeout, not only on T1691.001 finding emission). Added explanatory note that 2-block+1-restart correctly yields T0827 even with no T1691.001 finding emitted for the first 2 events. — 2026-06-10"
  - "v1.2: Pass-2 adversarial fix CRITICAL-2: this BC is now the single reset owner for the shared correlation window. Added explicit window-expiry handler spec: when now_ts - correlation_window_start_ts >= CORRELATION_WINDOW_SECS (300s [F2-GATE]), reset ALL four fields together: restart_event_count=0, block_event_count=0, block_finding_emitted_this_window=false, loss_of_control_emitted=false, then set correlation_window_start_ts=now_ts. Invariant 6 rewritten to name the single reset owner. T0827_WINDOW_SECS constant removed (now CORRELATION_WINDOW_SECS=300s shared with BC-2.15.011/014). Verified both T0827 traces end-to-end: (A) 2-block+1-restart within 300s → T0827 fires; (B) 2-block spaced 150s apart + 1-restart at 200s → T0827 fires (key: no 120s sub-window reset). — 2026-06-10"
  - "v1.3: Pass-3 adversarial fix HIGH-3: changed all four plain `now_ts - flow.correlation_window_start_ts` subtractions to `now_ts.wrapping_sub(flow.correlation_window_start_ts)` to prevent panic under overflow-checks=true when timestamps go backward (out-of-order pcap replay, explicitly a valid confound per BC-2.15.014 Inv 2). Rationale: u32 second timestamps wrap at ~136 years — effectively never, policy kept. Matches BC-2.15.010 and Modbus BC-2.14.017 convention. — 2026-06-10"
  - "v1.4: Research threshold clarification (dnp3-f2-scope-threshold-validation.md §Q3): added Invariant 7 and clarification note to Precondition 1 that the ≥3 combined events must be DISTINCT impact events — a distinct restart event and/or a distinct sustained-block finding — not a single incident double-counted (e.g. one block_event_count increment that also satisfies the T1691.001 threshold does not count as two separate events toward T0827). The current implementation is already correct because restart_event_count and block_event_count are incremented by independent code paths (BC-2.15.011 and BC-2.15.014 respectively) and a single underlying incident can increment at most one of the two counters per occurrence. This clarification makes the invariant explicit for reviewers. — 2026-06-10"
  - "v1.5: Adversarial finding C-2 companion fix: extended the window-expiry reset set to include the two new BC-2.15.024 windowed fields: malformed_in_window=0 and malformed_anomaly_emitted=false. parse_errors (BC-2.15.024 lifetime counter) is explicitly NOT in the reset set — it is a monotonic lifetime counter consumed by BC-2.15.020 summarize(). Updated Description reset list, Postcondition 3 reset list, and Invariant 6 to name SIX fields reset at window expiry (restart_event_count, block_event_count, block_finding_emitted_this_window, loss_of_control_emitted, malformed_in_window, malformed_anomaly_emitted; plus correlation_window_start_ts updated to now_ts). Added Architecture Anchors for the two new fields. — 2026-06-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/dnp3-architecture-delta.md
  - .factory/research/dnp3-research.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
input-hash: TBD
---

# BC-2.15.015: Derived Loss-of-Control — N Restart/Block Events in Window Emits T0827 as Correlated Finding

## Description

T0827 ("Loss of Control") is an ICS Impact-tactic outcome, not a per-packet detection. It
is emitted as a CORRELATED finding: the consequence of observing a sustained pattern of
T0814 (restart events, via `flow.restart_event_count`) and/or T1691.001 (block-command
events, via `flow.block_event_count`) accumulating beyond a threshold within a shared
correlation time window on the same flow. T0827 MUST NOT be emitted from a single packet.
The `IcsImpact` tactic (new `MitreTactic::IcsImpact` variant, ADR-007 Decision 5) is
the tactic for this finding.

**This BC is the single reset owner** for the shared per-flow correlation window. When
`now_ts.wrapping_sub(flow.correlation_window_start_ts) >= CORRELATION_WINDOW_SECS = 300s` **[F2-GATE:
human to confirm]**, this handler resets ALL windowed correlated state together (six fields):
- `flow.restart_event_count = 0`
- `flow.block_event_count = 0`
- `flow.block_finding_emitted_this_window = false`
- `flow.loss_of_control_emitted = false`
- `flow.malformed_in_window = 0` (BC-2.15.024 windowed counter — NEW v1.5)
- `flow.malformed_anomaly_emitted = false` (BC-2.15.024 one-shot guard — NEW v1.5)
- `flow.correlation_window_start_ts = now_ts` (start of new window)

**`flow.parse_errors` is NOT in this reset set.** `parse_errors` is a LIFETIME/monotonic
counter (BC-2.15.024 Invariant 1) reported by BC-2.15.020 `summarize()`. It must never be
reset at window expiry.

BC-2.15.011, BC-2.15.014, and BC-2.15.024 reference this single reset; they do NOT own
separate window timers.

**[F2-GATE: human to confirm]**
The proposed default for the T0827 emission guard is:
- **Threshold**: `restart_event_count + block_event_count >= 3` (three or more combined restart
  and block-command events on the same flow).
- **Window**: **300 seconds** (5 minutes). A 5-minute window captures sustained disruption
  campaigns while avoiding false positives from isolated maintenance events.
- **One-shot guard**: T0827 is emitted at most ONCE per (flow, window). The `loss_of_control_emitted`
  flag prevents repeated T0827 findings for the same sustained pattern.

The human should confirm whether 3-of-300s is appropriate, or specify alternative values.
A lower threshold (e.g., 2) increases sensitivity but also false-positive rate. A higher
threshold (e.g., 5) misses shorter but still impactful attack sequences.

## Preconditions

1. `flow.restart_event_count + flow.block_event_count >= T0827_THRESHOLD` (proposed: 3).
   **Distinct-event clarification (v1.4):** the ≥3 combined count must represent DISTINCT
   impact events — each increment of `restart_event_count` (from BC-2.15.011) counts as one
   restart event; each increment of `block_event_count` (from BC-2.15.014) counts as one
   block-command event. A single underlying network incident increments at most one of the
   two counters per occurrence (the code paths are independent), so double-counting of a
   single incident is not possible under the current architecture. This clarification makes
   the semantic intent explicit: three correlated events, not one event counted multiple times.
2. The accumulated events occurred within `CORRELATION_WINDOW_SECS` (proposed: 300s) of
   `flow.correlation_window_start_ts` — i.e., `now_ts.wrapping_sub(flow.correlation_window_start_ts) < CORRELATION_WINDOW_SECS`.
   // wrapping_sub used for u32 second timestamps; wrap at ~136 years — effectively never, policy kept.
3. `flow.loss_of_control_emitted == false` (one-shot guard).
4. `self.all_findings.len() < MAX_FINDINGS`.
5. The triggering event (the Nth restart or block-command event that crossed the threshold) was
   just observed — this BC fires within the same `on_data` call that crosses the threshold.

**Window expiry check** (evaluated at every on_data call for this flow, BEFORE the emission check):
- If `now_ts.wrapping_sub(flow.correlation_window_start_ts) >= CORRELATION_WINDOW_SECS`: execute the window
  reset (see Description) and re-evaluate preconditions 1–5 with the freshly-reset state.
  (After reset, precondition 1 will be false unless the triggering event alone crosses the
  threshold, which is impossible at threshold=3.)

## Postconditions

1. Exactly ONE `Finding` is pushed (after the triggering T0814 or T1691.001 finding, per BC-2.15.013 ordering):
   - `category: ThreatCategory::Impact`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::Medium`
   - `summary`: `"DNP3 sustained loss-of-control pattern: {restart_count} restart events + {block_count} blocked commands within {elapsed}s on flow (dest={dest:#06X})"`
   - `evidence`: summary of accumulated events on the flow
   - `mitre_techniques: vec!["T0827"]`
   - `source_ip: Some(...)`, `timestamp: Some(...)`
2. `flow.loss_of_control_emitted = true` (one-shot guard set).

**Window reset postcondition** (when window expiry fires — separate from T0827 emission):
3. When `now_ts.wrapping_sub(flow.correlation_window_start_ts) >= CORRELATION_WINDOW_SECS`:
   // wrapping_sub used for u32 second timestamps; wrap at ~136 years — effectively never, policy kept.
   - `flow.restart_event_count = 0`
   - `flow.block_event_count = 0`
   - `flow.block_finding_emitted_this_window = false`
   - `flow.loss_of_control_emitted = false`
   - `flow.malformed_in_window = 0` (BC-2.15.024 windowed counter; NEW v1.5)
   - `flow.malformed_anomaly_emitted = false` (BC-2.15.024 one-shot guard; NEW v1.5)
   - `flow.correlation_window_start_ts = now_ts`
   **Note: `flow.parse_errors` is NOT reset here.** It is a lifetime counter; see BC-2.15.024
   Invariant 1 and BC-2.15.020.

## Invariants

1. **Multi-event requirement**: T0827 requires `restart_event_count + block_event_count >= T0827_THRESHOLD`.
   It MUST NOT be emitted from a single restart or block-command event. [ADR-007 Decision 5:
   "T0827 MUST NOT be emitted from a single packet and MUST require a multi-event correlation window"]
2. **T0827 tactic is IcsImpact (NEW variant)**: T0827 "Loss of Control" uses `MitreTactic::IcsImpact`
   (new variant added in this feature cycle, distinct from enterprise `Impact`). [ADR-007 Decision 5]
3. **One-shot guard**: `loss_of_control_emitted` prevents repeated T0827 findings within the same
   correlation window. Only one T0827 impact finding is emitted per (flow, window).
4. **Emitted AFTER the triggering T0814/T1691.001 finding**: ordering per BC-2.15.013. T0827 is
   a consequence finding, not a primary detection finding.
5. **Independent accumulator feeds**: `restart_event_count` is fed unconditionally by BC-2.15.011;
   `block_event_count` is fed unconditionally by BC-2.15.014. Either or both can contribute to
   the T0827 threshold. Critically, BC-2.15.014 increments `block_event_count` UNCONDITIONALLY
   on every block-timeout (not only when a T1691.001 finding is emitted), so the first 2 block
   events (below the T1691.001 threshold of 3) still contribute to this T0827 accumulator.
   A scenario of 2 block events + 1 restart correctly yields T0827 (combined count=3) even
   though no T1691.001 finding was emitted for the first 2 block events.
6. **Single reset owner**: This BC (BC-2.15.015) owns the window-expiry reset logic. After
   `CORRELATION_WINDOW_SECS = 300s` **[F2-GATE]** elapsed since `correlation_window_start_ts`,
   ALL six windowed correlated-state fields reset together: `restart_event_count`,
   `block_event_count`, `block_finding_emitted_this_window`, `loss_of_control_emitted`,
   `malformed_in_window` (BC-2.15.024 windowed counter; v1.5 addition), and
   `malformed_anomaly_emitted` (BC-2.15.024 one-shot guard; v1.5 addition); plus
   `correlation_window_start_ts` is set to `now_ts`. **`parse_errors` is explicitly NOT in
   this reset set** — it is the lifetime structural-error counter consumed by BC-2.15.020
   `summarize()`. BC-2.15.011, BC-2.15.014, and BC-2.15.024 do NOT own separate window
   timers; they reference this single reset. This eliminates the contradiction where a 120s
   BLOCK_CMD_WINDOW_SECS reset (old v1.1 BC-2.15.014 design) would discard block events
   before T0827 could see them.
7. **Distinct impact events** (v1.4 clarification, sourced from
   dnp3-f2-scope-threshold-validation.md §Q3): the ≥3 combined guard is satisfied by DISTINCT
   impact events. A restart event (increment of `restart_event_count`) and a block-command
   event (increment of `block_event_count`) are events from independent code paths (BC-2.15.011
   and BC-2.15.014 respectively). A single underlying incident (e.g. one timed-out control
   request) can increment at most one of the two counters per occurrence — the T0827 accumulator
   sum therefore represents genuinely distinct events and cannot be satisfied by a single
   incident double-counted through separate code paths.
8. **DoS-bounded**: `restart_event_count` and `block_event_count` are `u64` (no practical
   overflow). `loss_of_control_emitted` is a bool one-shot guard. Window reset at 300s bounds
   the worst-case accumulator value at `300s / BLOCK_CMD_TIMEOUT_SECS = 300/10 = 30` block
   events per window per flow (tight upper bound on pending_requests churn).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | 2 restarts + 0 block events (threshold=3) | No T0827 yet (combined count=2 < 3) |
| EC-002 | 1 restart + 2 block events = 3 total | T0827 emitted (threshold met); `loss_of_control_emitted=true` |
| EC-003 | 3 restarts (all T0814, no T1691.001) | T0827 emitted (restart_event_count=3 ≥ threshold) |
| EC-004 | T0827 already emitted, 4th restart arrives (same window) | No additional T0827 (one-shot guard); `loss_of_control_emitted` still true |
| EC-005 | Window expires (300s) then new pattern begins | All counters and guards reset (Postcondition 3); new T0827 possible in next window |
| EC-006 | `all_findings.len() == MAX_FINDINGS` when T0827 threshold crossed | No T0827 pushed; triggering T0814/T1691.001 already in vec (was pushed first per ordering) |
| EC-007 | Single COLD_RESTART (count=1, threshold=3) | No T0827 (Invariant 1: must NOT fire from single event) |
| EC-008 | 2 block events at t=0 and t=150s (no reset at t=120s), restart at t=200s | Both block events still in window (300s > 200s); `block_event_count=2`, `restart_event_count=1`; combined=3 → T0827 fires. This is the primary trace verifying the single-window fix: old 120s sub-window would have reset `block_event_count` at t=120s, making combined count=0+1=1 < threshold. |
| EC-009 | 3 restarts > 300s apart (t=0, t=200, t=400) | Window expiry at 300s resets state; 3rd restart at t=400 is in new window with `restart_event_count=1`; no T0827 |

## Canonical Test Vectors

**Trace A — 3× COLD_RESTART within 300s (pure restart pattern):**

| t (s) | Event | restart_event_count | block_event_count | T0827? |
|-------|-------|--------------------|--------------------|--------|
| 0 | COLD_RESTART → T0814 (correlation_window_start_ts=0) | 1 | 0 | No |
| 60 | COLD_RESTART → T0814 | 2 | 0 | No |
| 120 | COLD_RESTART → T0814 | 3 | 0 | **Yes** — 3+0=3 ≥ threshold |

Expected after t=120: `[T0814 finding, T0827 finding]`; `loss_of_control_emitted=true`.

**Trace B — 2 block events (spaced 150s apart) + 1 restart (key single-window fix trace):**

| t (s) | Event | restart_event_count | block_event_count | Window reset? | T0827? |
|-------|-------|--------------------|--------------------|---------------|--------|
| 0 | Block timeout #1 (correlation_window_start_ts=0) | 0 | 1 | No | No |
| 150 | Block timeout #2 (150s < 300s; NO sub-window reset) | 0 | 2 | No | No |
| 200 | COLD_RESTART → T0814 | 1 | 2 | No | **Yes** — 1+2=3 ≥ threshold |

Expected after t=200: `[T0814 finding, T0827 finding]`; `loss_of_control_emitted=true`.
(Under old 120s BLOCK_CMD_WINDOW_SECS: block_event_count would reset to 0 at t=120s → combined=0+1=1 at t=200s → T0827 would NOT fire. **This is the contradiction CRITICAL-2 fixes.**)

**Trace C — T0827 one-shot guard:**

| t (s) | Event | Combined count | T0827? |
|-------|-------|---------------|--------|
| 0 | COLD_RESTART | 1 | No |
| 60 | COLD_RESTART | 2 | No |
| 120 | COLD_RESTART | 3 | **Yes** (1st T0827) |
| 180 | COLD_RESTART | 4 | No (one-shot guard active) |

**Trace D — window expiry reset, new pattern:**

| t (s) | Event | Combined count | T0827? |
|-------|-------|---------------|--------|
| 0 | Block timeout #1 | 1 | No |
| 100 | Block timeout #2 | 2 | No |
| 300 | Window expiry → reset (all fields=0) | 0 | No |
| 310 | COLD_RESTART (new window, start_ts=300) | 1 | No |

No T0827 in either window; first window expired before threshold reached.

| Scenario summary | Event sequence | Expected outcome |
|-----------------|---------------|-----------------|
| 3× COLD_RESTART within 300s | T0814×3 | After 3rd: [T0814 finding] then [T0827 finding]; `loss_of_control_emitted=true` |
| 2 block (spaced 150s) + 1 restart within 300s | T1691-event×2 + T0814×1 | After restart: [T0814 finding] then [T0827 finding] (Trace B) |
| 3× COLD_RESTART, then 4th COLD_RESTART same window | T0814×4 | [3 T0814 findings] + [1 T0827 finding] after 3rd; no 2nd T0827 at 4th (Trace C) |
| 3 restarts > 300s apart | T0814 at t=0, t=200, t=400 | No T0827 (3rd event is in new window; Trace D variant) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Multi-event accumulation and guard: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — T0827 derived-impact finding is the correlated consequence detection for sustained ICS disruption; it elevates isolated T0814/T1691.001 signals into a higher-confidence impact assessment when they co-occur on the same flow, providing operators with an actionable Impact-tactic alert |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — findings emitted only on valid DNP3 port-20000 flows) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24); ADR-007 Decision 5 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | T0827 — Loss of Control (ICS Impact tactic TA0105; active in v19.1; new `MitreTactic::IcsImpact` variant required — ADR-007 Decision 5) |

## Related BCs

- BC-2.15.011 — composes with (T0814 increments restart_event_count; shares single 300s window owned here)
- BC-2.15.014 — composes with (block-timeout increments block_event_count; shares single 300s window owned here)
- BC-2.15.013 — composes with (emission ordering: T0827 last, after triggering finding)
- BC-2.15.022 — depends on (MAX_FINDINGS cap)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `Dnp3FlowState.restart_event_count: u64`
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.block_event_count: u64`
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.pending_requests: HashMap<(u16, u8), u32>`
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.block_finding_emitted_this_window: bool`
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.loss_of_control_emitted: bool`
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.malformed_in_window: u64` (NEW v1.5; windowed malformed-frame counter owned by BC-2.15.024; reset here at window expiry)
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.malformed_anomaly_emitted: bool` (NEW v1.5; one-shot guard owned by BC-2.15.024; reset here at window expiry)
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.correlation_window_start_ts: u32` (single shared window; this BC owns its reset logic)
- `src/mitre.rs` — `technique_info("T0827")` arm (NEW: `("Loss of Control", MitreTactic::IcsImpact)`); `MitreTactic::IcsImpact` variant (NEW)
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §8` (detection table: "Loss of control derived → T0827 correlated finding")
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §9.2` (MitreTactic::IcsImpact enum addition)
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 5`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

(none — effectful shell; VP-007 in F4 verifies T0827 is in SEEDED_TECHNIQUE_IDS and EMITTED_IDS)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-007 Decision 5; dnp3-research.md §6 (T0827 confirmed active [MITRE]: "Loss of Control", Impact TA0105, last-mod 2026-05-12); dnp3-research.md §5 ("T0827 is an Impact-tactic outcome, not a per-packet detection") |
| **Confidence** | medium — T0827 confirmed [MITRE] active; threshold/window values are [JUDGMENT] pending [F2-GATE] confirmation |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads restart_event_count + block_event_count; writes loss_of_control_emitted, all_findings; owns window-expiry reset of all six windowed correlation fields (restart_event_count, block_event_count, block_finding_emitted_this_window, loss_of_control_emitted, malformed_in_window, malformed_anomaly_emitted) + correlation_window_start_ts; does NOT reset parse_errors (lifetime) |
| **Deterministic** | yes — same event sequence produces same T0827 trigger |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (correlated/derived finding) |
