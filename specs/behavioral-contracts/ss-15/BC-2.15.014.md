---
document_type: behavioral-contract
level: L3
version: "1.5"
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
  - "v1.1: Pass-1 adversarial fix I-5: separated block_event_count (unconditional timeout counter, feeds T0827 accumulator) from block_finding_emitted_this_window (one-shot guard, prevents T1691.001 finding flood). Original model had block_event_count gated behind finding emission, causing the T0827 accumulator to never see the first 2 block events. Fixed: Postconditions restructured into unconditional (counter increment) + conditional (finding emission). Precondition 5 rewritten to check counter AFTER increment. EC-003, EC-004, EC-007 and canonical test vectors updated. Architecture anchors updated with new block_finding_emitted_this_window field. — 2026-06-10"
  - "v1.2: Pass-2 adversarial fix CRITICAL-2: eliminated separate BLOCK_CMD_WINDOW_SECS=120s window entirely. T1691.001 emission now uses the shared CORRELATION_WINDOW_SECS=300s [F2-GATE: human to confirm] tracked by correlation_window_start_ts. Sustained pattern is now 3-of-300s (was 3-of-120s). block_event_count and block_finding_emitted_this_window reset ONLY at 300s window expiry together with restart_event_count and loss_of_control_emitted — single reset owner in BC-2.15.015 window-expiry handler. Invariant 7 (old 120s window reset) rewritten. EC-006 (old 120s reset) rewritten to 300s. Canonical test vectors updated. Architecture anchors updated: removed BLOCK_CMD_WINDOW_SECS, added CORRELATION_WINDOW_SECS reference. The key security implication: block events spaced 120–300s apart are no longer silently dropped; they accumulate toward both T1691.001 and T0827. — 2026-06-10"
  - "v1.3: Pass-3 adversarial fix HIGH-2 (cross-ref accuracy): Invariant 8 cross-reference '(see BC-2.15.016 for the pending_requests cap)' is now accurate — BC-2.15.016 v1.1 adds Postconditions 8–10 and Invariant 5 specifying MAX_PENDING_REQUESTS=256 with oldest-eviction. Also fix HIGH-3: Precondition 3 timeout check changed from plain subtraction `now_ts - request_ts` to `now_ts.wrapping_sub(request_ts)` to prevent panic under overflow-checks=true when timestamps go backward (out-of-order pcap replay). Rationale: u32 second timestamps wrap at ~136 years — effectively never, policy kept. — 2026-06-10"
  - "v1.4: Research validation confirmation (dnp3-f2-scope-threshold-validation.md §Q2 Threshold-2): DIRECT_OPERATE_NR (0x06) exclusion from the block-command timeout count is explicitly confirmed as a required guard by the research pass [VERIFIED]. The exclusion is already present in Precondition 1 and Invariant 1 (since v1.0). This entry records the explicit research-backed validation. No behavioral change. — 2026-06-10"
  - "v1.5: Adversarial Pass-3 fix F-P3-001 (MEDIUM): PC3 evidence format reconciled to producible form. The original format 'FC=0x{fc:02X} dest={dest:#06X} app_seq={seq} ts={ts}' required an FC byte that is NOT retained in pending_requests (keyed (dest_addr, app_seq) → request_ts only; FC was deliberately excluded from the value type in STORY-107). Additionally, all entries in pending_requests are Control-class by construction (only FC 0x03/0x04/0x05 are inserted; FC 0x06 DIRECT_OPERATE_NR is excluded at insert), so the FC byte adds no discriminating value. New canonical format: one entry per timed-out request, 'dest={dest:#06X} app_seq={seq} ts={ts}' — producible from the (dest, app_seq) key and request_ts value available at removal time in scan_block_timeouts. — 2026-06-11"
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

# BC-2.15.014: Inferred Block-Command — Control Request Without Response Within Window Emits T1691.001

## Description

wirerust is a passive PCAP analyzer. It cannot directly observe a blocked command. The
defensible inference signal for T1691.001 ("Block Operational Technology Message: Command
Message") is a Control-class request (SELECT/OPERATE/DIRECT_OPERATE) for which no
corresponding outstation RESPONSE (FC 0x81) or CONFIRM arrives within a timeout window.
Crucially, DIRECT_OPERATE_NR (0x06) is EXCLUDED — it intentionally expects no response.
The inference is per-correlation-key: (flow_key + destination_address + app_seq). When the
timeout expires with no matching response, the `block_event_count` counter is
UNCONDITIONALLY incremented (feeding T0827 accumulation in BC-2.15.015). When the counter
reaches `BLOCK_CMD_THRESHOLD = 3` within the shared `CORRELATION_WINDOW_SECS = 300s`
**[F2-GATE: human to confirm]** window, a single T1691.001 finding is emitted and the
`block_finding_emitted_this_window` one-shot guard is set. Subsequent timeouts in the same
window continue to increment `block_event_count` (for T0827 accumulation) but do not emit
additional T1691.001 findings.

**Note on window change (v1.2):** The T1691.001 "sustained pattern" is now **3-of-300s**
(was 3-of-120s in v1.1). This is intentional: collapsing to the single shared 300s
CORRELATION_WINDOW_SECS eliminates the window-reset contradiction where block events
spaced 120–300s apart would be lost before T0827 could see them. The higher false-positive
tolerance at 120s versus 300s is accepted. **[F2-GATE: human to confirm 300s for T1691.001
threshold, or specify a different per-technique window that does not cause the contradiction.]**

**[F2-GATE: human to confirm]**
The proposed defaults are:
- **Correlation key**: per-flow (`FlowKey`) + per-`destination` (link address) + per-application SEQ (App Control `0x0F` bits). This uniquely identifies a request/response exchange on a given flow.
- **Timeout window**: **10 seconds**. Rationale: DNP3 SBO select-to-operate timeout is typically 3–10 seconds per device profile; 10s gives headroom for slow outstations while still catching sustained blocking patterns. Packet loss and brief capture gaps will produce false positives if this value is too low.
- **Sustained pattern threshold**: at least **3 request-without-response events** within a **300-second window** before emitting. A single timeout could be packet loss; 3+ suggests a sustained blocking condition. (This is the guard that separates packet-loss noise from actual T1691.001 evidence.)

The human should confirm whether 10s timeout and 3-of-300s threshold are appropriate, or specify alternative values.

## Preconditions

1. A Control-class request (FC in {0x03, 0x04, 0x05}) has been observed on a FIR=1 frame
   (BC-2.15.008 / BC-2.15.010). FC 0x06 (DIRECT_OPERATE_NR) is EXCLUDED — it expects no response by design. [SPEC: dnp3-research.md §8 "DIRECT_OPERATE_NR: do not map missing response to T1691.001"]
2. The correlation key (flow_key + dest_addr + app_seq) has been recorded in `flow.pending_requests`.
3. The timeout window has elapsed (`now_ts.wrapping_sub(request_ts) > BLOCK_CMD_TIMEOUT_SECS = 10`).
   // wrapping_sub used for u32 second timestamps; wrap at ~136 years — effectively never, policy kept.
   // Plain subtraction would panic under overflow-checks=true on out-of-order pcap replay (see BC-2.15.014 Inv 2).
4. No RESPONSE (FC 0x81) matching (dest_addr, app_seq) has been observed within the timeout.

**Finding-emission preconditions** (applied AFTER the unconditional counter increment in Postcondition 1):
5. `flow.block_event_count >= BLOCK_CMD_THRESHOLD` (i.e., after the increment, the count equals or exceeds `BLOCK_CMD_THRESHOLD = 3`) AND `flow.block_finding_emitted_this_window == false`.
6. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

**Unconditional (every block-timeout event, regardless of threshold):**
1. `flow.block_event_count += 1`. (This always happens when preconditions 1–4 are met, BEFORE
   the emission check. The counter feeds both the T1691.001 threshold AND the T0827 accumulator
   in BC-2.15.015. This counter is part of the shared correlation state and resets ONLY at
   the 300s CORRELATION_WINDOW_SECS expiry — not on any separate timer.)
2. The timed-out entry is removed from `flow.pending_requests`.

**Conditional — finding emission (when preconditions 5–6 are met):**
3. Exactly ONE `Finding` is pushed:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Possible` (inferred, not directly observed)
   - `confidence: Confidence::Low` (passive inference; packet loss is a confound)
   - `summary`: `"DNP3 inferred blocked command: {count} requests without response within {window}s (dest={dest:#06X})"`
   - `evidence`: one entry per timed-out request: `"dest={dest:#06X} app_seq={seq} ts={ts}"` (FC omitted — not retained in `pending_requests` keyed `(dest_addr, app_seq) → request_ts`; all entries are Control-class by construction)
   - `mitre_techniques: vec!["T1691.001"]`
   - `source_ip: Some(...)`, `timestamp: Some(...)`
4. `flow.block_finding_emitted_this_window = true` (one-shot guard: at most one T1691.001 finding
   per CORRELATION_WINDOW_SECS per flow; subsequent timeouts still increment `block_event_count`
   but do not emit additional T1691.001 findings).

## Invariants

1. **DIRECT_OPERATE_NR excluded**: FC 0x06 expects no response by spec design. Its "missing response" is NOT evidence of blocking. [SPEC: dnp3-research.md §8 FP caveat 3; dnp3-research.md §3.2]
2. **Verdict::Possible / Confidence::Low**: this is an inferred finding, not a direct observation. Packet loss and capture gaps are valid confounds. The lower confidence communicates this uncertainty.
3. **T1691.001 is the correct v19.1 technique** [MITRE: dnp3-research.md §6]: T1691.001 "Block Operational Technology Message: Command Message" is the active replacement for revoked T0803. Tactic: IcsInhibitResponseFunction.
4. **Sustained pattern guard**: `BLOCK_CMD_THRESHOLD = 3` events in `CORRELATION_WINDOW_SECS = 300s` **[F2-GATE: human to confirm]** prevents single-packet-loss false positives. (Changed from 120s in v1.1 to 300s in v1.2 to eliminate window-reset contradiction — see Description for rationale.)
5. **block_event_count feeds T0827 unconditionally**: `block_event_count` is incremented on EVERY block-timeout (Postcondition 1), not only when a T1691.001 finding is emitted. This ensures the T0827 accumulator in BC-2.15.015 sees all block events, including the first two that are below the T1691.001 emission threshold. After the threshold is crossed, subsequent timeouts still increment `block_event_count` (feeding the T0827 accumulator) but do not emit additional T1691.001 findings (`block_finding_emitted_this_window` guard).
6. **Separate counter from finding guard**: `block_event_count` (the accumulator) and `block_finding_emitted_this_window` (the one-shot guard) are distinct state fields. The counter is always updated; the guard prevents finding flooding. BC-2.15.015 reads `block_event_count`, not the one-shot guard.
7. **Single shared 300s correlation window**: `block_event_count` and `block_finding_emitted_this_window` are part of the shared per-flow correlation state (`correlation_window_start_ts: u32`). They reset to 0 / false ONLY when `now_ts.wrapping_sub(correlation_window_start_ts) >= CORRELATION_WINDOW_SECS = 300s` — together with `restart_event_count` and `loss_of_control_emitted`. There is NO separate BLOCK_CMD_WINDOW_SECS timer. The single reset owner is the window-expiry handler in BC-2.15.015. This eliminates the contradiction where block events spaced 120–300s apart would be silently dropped by a 120s sub-window expiry before T0827 could see them.
8. **DoS-bounded**: `pending_requests` is a `HashMap<(u16, u8), u32>` bounded to `MAX_PENDING_REQUESTS = 256` entries — when full, the oldest entry (minimum `request_ts`) is evicted before a new insert (see BC-2.15.016 Postconditions 8–10 and Invariant 5 for the full eviction spec). `block_event_count` is `u64` (no practical overflow). `block_finding_emitted_this_window` is a bool one-shot guard.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DIRECT_OPERATE_NR (0x06) — no response expected | Not tracked in pending_requests; no T1691.001 inference; `block_event_count` NOT incremented |
| EC-002 | SELECT (0x03) response arrives before timeout | Entry removed from pending_requests; no T1691.001; `block_event_count` NOT incremented |
| EC-003 | First 2 timeouts (below threshold of 3) | `block_event_count` unconditionally incremented (to 1, then 2); no T1691.001 finding emitted yet (Precondition 5 not met). T0827 accumulator in BC-2.15.015 DOES see these increments. |
| EC-004 | 3rd timeout (threshold met: `block_event_count` reaches 3 after increment) | `block_event_count` incremented to 3 (Postcondition 1). Precondition 5 met; T1691.001 finding emitted (Postcondition 3). `block_finding_emitted_this_window = true`. |
| EC-007 | 4th timeout (same window, `block_finding_emitted_this_window = true`) | `block_event_count` incremented to 4 (feeds T0827 accumulator); NO additional T1691.001 finding (one-shot guard). |
| EC-005 | Capture gap (packet loss) causes response to appear lost | T1691.001 may fire at threshold; this is an accepted false positive per the passive-analyzer caveat |
| EC-006 | Window reset (300s elapsed since `correlation_window_start_ts`) | `block_event_count` resets to 0; `block_finding_emitted_this_window` resets to `false`; `restart_event_count` resets to 0; `loss_of_control_emitted` resets to `false` — all four fields reset together by the single reset owner. Threshold re-accumulates from 0 in new window. |
| EC-008 | 2 block timeouts at t=0 and t=150s, then 1 restart at t=200s | Both block timeouts within 300s window; `block_event_count=2` at t=150s. Restart at t=200s: `restart_event_count=1`. Combined=3 ≥ T0827_THRESHOLD → T0827 fires. (Key test: under old 120s model, block_event_count would reset at t=120s and T0827 would NOT fire.) |

## Canonical Test Vectors

| Scenario | Events | Expected outcome |
|----------|--------|-----------------|
| 3× DIRECT_OPERATE (0x05) with no RESPONSE within 10s each | 3 timeout events in 300s | After timeout 1: `block_event_count=1`, no finding. After timeout 2: `block_event_count=2`, no finding. After timeout 3: `block_event_count=3`, T1691.001 finding emitted: `{verdict:Possible, confidence:Low, mitre_techniques:["T1691.001"]}`. `block_finding_emitted_this_window=true`. `block_event_count=3` feeds BC-2.15.015. |
| 2 block events (spaced at t=0, t=150s) then T0827 threshold crossed by restart at t=200s | 2 block + 1 restart in 300s window | Both block events increment `block_event_count` (to 1, 2) — NOT reset at 120s. BC-2.15.015 sees `block_event_count=2 + restart_event_count=1 = 3` and emits T0827. (No T1691.001 emitted — block_event_count=2 below threshold of 3.) |
| DIRECT_OPERATE_NR (0x06), no response | FC=0x06 observed | No T1691.001 inference (FC excluded); `block_event_count` NOT incremented |
| SELECT (0x03) → RESPONSE (0x81) within 5s | Matched pair | No T1691.001 (response received); `block_event_count` NOT incremented |
| 2 timeouts then response pattern resumes | 2 block events, then responses | `block_event_count=2` after two timeouts; T0827 accumulator sees both. No T1691.001 (threshold not reached). |
| 300s window expires, then new block events begin | Block events after 300s | All counters and guards reset at 300s expiry; fresh accumulation from 0 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Request/response correlation and timeout: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — T1691.001 block-command inference is a distinctive capability of the DNP3/ICS analyzer that detects the Ukraine 2015 (Sandworm) technique of blocking control commands to prevent operators from restoring tripped circuit breakers |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — findings emitted only on valid DNP3 port-20000 flows) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-23); ADR-007 Decision 5 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | T1691.001 — Block Operational Technology Message: Command Message (ICS sub-technique, v19.1; tactic: IcsInhibitResponseFunction TA0107; replaces revoked T0803) |

## Related BCs

- BC-2.15.006 — depends on (Control-class FC classification; FC 0x06 exclusion)
- BC-2.15.008 — depends on (FIR=1 gate enables request tracking)
- BC-2.15.015 — composes with (block_event_count feeds T0827 derived-impact accumulator; shares single 300s CORRELATION_WINDOW_SECS and reset owner)
- BC-2.15.022 — depends on (MAX_FINDINGS cap)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `Dnp3FlowState.pending_requests: HashMap<(u16, u8), u32>` (bounded; key=(dest_addr, app_seq), value=request_ts)
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.block_event_count: u64` (incremented unconditionally on every block-timeout; part of shared 300s correlation state)
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.block_finding_emitted_this_window: bool` (one-shot guard; prevents T1691.001 finding flood within 300s window; resets with shared window)
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.correlation_window_start_ts: u32` (single shared 300s window start; reset owner in BC-2.15.015; this field — NOT a separate BLOCK_CMD_WINDOW_SECS — governs the T1691.001 threshold window)
- `src/mitre.rs` — `technique_info("T1691.001")` arm (NEW; to be added in F4)
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §8` (detection table: "Block command inferred → T1691.001")
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 5`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

(none — effectful shell logic; MITRE catalog compliance verified by VP-007 in F4)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-007 Decision 5; dnp3-research.md §5.2 (passive analyzer T1691.001 rationale [JUDGMENT]); §6 (T1691.001 confirmed active [MITRE]) |
| **Confidence** | medium — T1691.001 confirmed [MITRE]; timeout/threshold values are [JUDGMENT] (F2-GATE); passive-inference nature acknowledged |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow.pending_requests, flow.block_event_count, flow.block_finding_emitted_this_window, all_findings |
| **Deterministic** | yes — same request/response sequence produces same outcome |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell |
