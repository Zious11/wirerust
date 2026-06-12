---
document_type: story
story_id: STORY-109
epic_id: E-15
version: "1.3"
status: completed
producer: story-writer
timestamp: 2026-06-10T00:00:00Z
phase: 3
points: 13
priority: P0
depends_on: [STORY-108]
blocks: [STORY-110]
behavioral_contracts:
  - BC-2.15.014
  - BC-2.15.015
  - BC-2.15.018
  - BC-2.15.019
  - BC-2.15.023
  - BC-2.15.024
verification_properties: [VP-007]
tdd_mode: strict
target_module: analyzer/dnp3
subsystems: [SS-15]
wave: 38
estimated_days: 5
feature_id: issue-008-dnp3-analyzer
github_issue: 8
# BC status: 6 BCs authored 2026-06-10; BC-2.15.014 v1.6, BC-2.15.015 v1.5, BC-2.15.018 v1.1, BC-2.15.019 v1.1, BC-2.15.023 v1.1, BC-2.15.024 v1.1 (BC-2.15.016 removed from inputs: F7 F-001/F-002 correction — transposed invariant annotation; no AC legitimately references it)
# VP-007 atomic update obligation: T1691.001 and T0827 (+ MitreTactic::IcsImpact) seeded HERE
inputs:
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.014.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.015.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.018.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.019.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.023.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.024.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
input-hash: "cf0bb94"
---

# STORY-109: DNP3 Correlated/Derived + Anomaly Detections — T1691.001, T0827, Broadcast, Unsolicited, ENABLE/DISABLE, Malformed

## Narrative

- **As a** ICS/OT security analyst using wirerust against DNP3 TCP captures
- **I want** the DNP3 analyzer to detect inferred blocked commands (T1691.001), derive loss-of-control impact findings (T0827) from correlated restart/block events, flag broadcast control anomalies, flag unexpected unsolicited responses, flag ENABLE/DISABLE_UNSOLICITED abuse, and raise structural malformation anomalies consistent with Crain-Sistrunk crash probes — all within a shared 300s correlation window
- **So that** the analyst receives correlated ICS impact assessments and protocol anomaly signals beyond the per-packet direct detections, covering the full DNP3 threat model defined in the DNP3/ICS analyzer spec

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.15.014 | Inferred Block-Command — Control Request Without Response Within Window Emits T1691.001 |
| BC-2.15.015 | Derived Loss-of-Control — N Restart/Block Events in Window Emits T0827 as Correlated Finding |
| BC-2.15.018 | Broadcast Destination Anomaly — DEST in 0xFFFD/0xFFFE/0xFFFF Emits Anomaly Finding |
| BC-2.15.019 | Unsolicited Response Anomaly — UNS Bit Set or FC 0x82 From Unexpected Pattern |
| BC-2.15.023 | Unsolicited-Response Enable/Disable Abuse — FC 0x15/0x14 Observed Emits T0814 |
| BC-2.15.024 | Malformed/Structural DNP3 Anomaly — malformed_in_window Threshold Emits T0814 |

## VP-007 Atomic Update Obligation (SEEDED_TECHNIQUE_IDS — LANDS HERE)

This story first EMITS T1691.001 and T0827. Per the VP-007 atomic-update obligation (ADR-007 Decision 5), the implementer MUST add to `src/mitre.rs` in the SAME commit as the T1691.001 and T0827 detection branches:

1. **T1691.001** — `technique_info("T1691.001")` arm: `("Block Operational Technology Message: Command Message", MitreTactic::IcsInhibitResponseFunction)`. Add to `SEEDED_TECHNIQUE_IDS` and the kani_proofs `EMITTED_IDS` set (`#[cfg(kani)]`). T1691.001 is the active replacement for revoked T0803.
2. **T0827** — `technique_info("T0827")` arm: `("Loss of Control", MitreTactic::IcsImpact)`. Add to `SEEDED_TECHNIQUE_IDS` and the kani_proofs `EMITTED_IDS` set (`#[cfg(kani)]`).
3. **`MitreTactic::IcsImpact`** — NEW enum variant required in `MitreTactic` (ADR-007 Decision 5). This is the ONLY story that introduces this variant. It must be added atomically with the T0827 emission branch.

**MITRE catalog counts after this story: 23 seeded / 15 emitted / 8 catalogue-only** (unchanged per BC-2.15.023 Invariant 3 and BC-2.15.024 Invariant 4 — T0814 is already seeded and emitted from STORY-108; no new technique introduced there).

## Acceptance Criteria

### AC-001 (traces to BC-2.15.014 postcondition 1 — block_event_count unconditional increment)
On every Control-class request timeout (FC in {0x03, 0x04, 0x05}, NOT 0x06) where no RESPONSE (FC=0x81) matching (dest_addr, app_seq) arrived within `BLOCK_CMD_TIMEOUT_SECS = 10s`: `flow.block_event_count += 1` UNCONDITIONALLY (regardless of whether a T1691.001 finding is emitted). The timed-out entry is removed from `flow.pending_requests`.
- **FC 0x06 (DIRECT_OPERATE_NR) is EXCLUDED** — expects no response by design; does NOT increment `block_event_count`.
- **Test:** `test_block_event_count_increments_unconditionally()`

### AC-002 (traces to BC-2.15.014 postcondition 3 — T1691.001 finding at threshold 3-of-300s)
When `flow.block_event_count >= BLOCK_CMD_THRESHOLD = 3` (after increment) AND `flow.block_finding_emitted_this_window == false` AND within `CORRELATION_WINDOW_SECS = 300s`: exactly ONE `Finding` pushed with `mitre_techniques: vec!["T1691.001"]`, `verdict: Possible`, `confidence: Low`. `flow.block_finding_emitted_this_window = true`.
- **Test:** `test_t1691_001_emitted_at_threshold_3_of_300s()` — 3 DIRECT_OPERATE requests each without response within 10s; assert finding at 3rd event.

### AC-003 (traces to BC-2.15.014 postcondition 2/invariant 6/7 — single shared 300s window)
`block_event_count` and `block_finding_emitted_this_window` are part of the shared `CORRELATION_WINDOW_SECS = 300s` window tracked by `correlation_window_start_ts`. They reset ONLY at 300s expiry (owned by BC-2.15.015 window-expiry handler), NOT on any separate shorter timer. Two block events spaced 150s apart BOTH count toward the threshold (NOT reset at 120s as in the old design).
- **Test:** `test_block_events_not_reset_at_120s()` — block at t=0, block at t=150s (both within 300s); assert `block_event_count=2`; no premature reset.

### AC-004 (traces to BC-2.15.015 postconditions 1/2 — T0827 emitted when combined ≥3)
When `flow.restart_event_count + flow.block_event_count >= T0827_THRESHOLD = 3` AND `flow.loss_of_control_emitted == false` AND within `CORRELATION_WINDOW_SECS = 300s`: ONE `Finding` pushed with `mitre_techniques: vec!["T0827"]`, `category: Impact`, `verdict: Likely`, `confidence: Medium`, tactic `IcsImpact` (NEW variant). `flow.loss_of_control_emitted = true`. T0827 is pushed AFTER the triggering direct finding (BC-2.15.013 ordering).
- **T0827 MUST NOT fire from a single restart or block event alone.**
- **Test:** `test_t0827_emitted_at_combined_threshold()` — 2 block events + 1 restart = 3; assert T0827 after restart (following Trace B from BC-2.15.015).

### AC-005 (traces to BC-2.15.015 postcondition 3 — window expiry resets SIX fields)
When `now_ts.wrapping_sub(flow.correlation_window_start_ts) >= CORRELATION_WINDOW_SECS = 300s`, BC-2.15.015 window-expiry handler resets ALL SIX windowed fields: `restart_event_count=0`, `block_event_count=0`, `block_finding_emitted_this_window=false`, `loss_of_control_emitted=false`, `malformed_in_window=0`, `malformed_anomaly_emitted=false`. PLUS `correlation_window_start_ts=now_ts`. `flow.parse_errors` is NOT reset (lifetime counter).
- **Test:** `test_correlation_window_expiry_resets_six_fields()` — advance past 300s; assert all six fields reset; parse_errors unchanged.

### AC-006 (traces to BC-2.15.015 invariant 7 — distinct-event guard, no double-counting)
The T0827 threshold accumulates DISTINCT impact events. A single block-timeout can increment at most `block_event_count` by 1 per occurrence. A single restart increments `restart_event_count` by 1. The two code paths are independent; no single incident can double-count.
- **Test:** `test_t0827_requires_distinct_events()` — 2 restarts + 0 blocks = 2 < threshold; no T0827.

### AC-007 (traces to BC-2.15.018 postcondition 1 — broadcast Control anomaly)
When `h.destination >= 0xFFFD` (broadcast range: 0xFFFD, 0xFFFE, 0xFFFF) AND `classify_dnp3_fc(app_fc) == Control` on FIR=1: ONE `Finding` pushed with `mitre_techniques: vec!["T1692.001"]`, `category: Suspicious`, `verdict: Possible`, `confidence: Medium`. Also `direct_operate_count` is incremented (broadcast Control still feeds the BC-2.15.010 threshold).
- **READ (0x01) to broadcast does NOT trigger anomaly** — only Control-class FCs.
- **Test:** `test_broadcast_control_anomaly_fires_for_dest_ffff()`, `test_broadcast_read_no_anomaly()`

### AC-008 (traces to BC-2.15.018 invariant 4/BC-2.15.013 invariant 4 — broadcast + burst both retained)
A broadcast Control FC emits the BC-2.15.018 anomaly finding FIRST (Suspicious/Possible/Medium). It ALSO increments `direct_operate_count`. If/when the burst threshold is later crossed, BC-2.15.010 emits a SECOND T1692.001 finding (Execution/Likely/Medium). Both findings are RETAINED — implementation must not deduplicate on technique ID alone.
- **Test:** `test_broadcast_and_burst_both_retained()` — 11 broadcast Control FCs; assert >1 T1692.001 finding in all_findings.

### AC-009 (traces to BC-2.15.019 postconditions 1/2 — unsolicited anomaly one-shot)
When FC=0x82 (UNSOLICITED_RESPONSE) observed AND `flow.enable_unsolicited_seen == false` AND `flow.response_seen == false` AND `flow.unsolicited_anomaly_emitted == false`: ONE `Finding` pushed with `mitre_techniques: vec!["T0814"]`, `verdict: Possible`, `confidence: Low`. `flow.unsolicited_anomaly_emitted = true` (one-shot).
- **ENABLE_UNSOLICITED (0x14) sets `flow.enable_unsolicited_seen=true`** — subsequent FC=0x82 NOT anomalous.
- **Test:** `test_unsolicited_response_anomaly_no_prior_enable()`, `test_unsolicited_response_no_anomaly_after_enable()`

### AC-010 (traces to BC-2.15.023 postcondition 1 — DISABLE_UNSOLICITED T0814 Likely/Medium)
FC=0x15 (DISABLE_UNSOLICITED) on FIR=1 pushes ONE `Finding` with `mitre_techniques: vec!["T0814"]`, `verdict: Likely`, `confidence: Medium`. Summary: `"DNP3 DISABLE_UNSOLICITED observed: FC 0x15 from src=... to dest=... — alarm suppression / event-blinding primitive"`. Per-occurrence, no one-shot guard.
- **Detection is on RAW FC byte (app_fc == 0x15), NOT via classify_dnp3_fc** — classify returns Management which is NOT used here.
- **Test:** `test_disable_unsolicited_emits_t0814_likely_medium()`

### AC-011 (traces to BC-2.15.023 postcondition 1 — ENABLE_UNSOLICITED T0814 Possible/Low)
FC=0x14 (ENABLE_UNSOLICITED) on FIR=1 pushes ONE `Finding` with `mitre_techniques: vec!["T0814"]`, `verdict: Possible`, `confidence: Low`. Per-occurrence, no one-shot guard.
- **Test:** `test_enable_unsolicited_emits_t0814_possible_low()`

### AC-012 (traces to BC-2.15.024 postconditions 2/3 — malformed_in_window threshold T0814)
On each structural-reject path fire (LENGTH<5, frame-length mismatch, carry overflow), BOTH `flow.parse_errors += 1` (lifetime) AND `flow.malformed_in_window += 1` (windowed) are incremented. When `flow.malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD = 3` AND `flow.malformed_anomaly_emitted == false` AND within 300s window: ONE `Finding` pushed with `mitre_techniques: vec!["T0814"]`, `verdict: Possible`, `confidence: Low`. Summary: `"DNP3 structural anomaly: {count} malformed frames in {elapsed}s window ... — possible Crain-Sistrunk crash-probe"`. `flow.malformed_anomaly_emitted = true`.
- **`parse_errors` (lifetime) is NEVER reset at window expiry. `malformed_in_window` (windowed) IS reset at 300s expiry.**
- **Test:** `test_malformed_anomaly_at_threshold_3_of_300s()` — 3 malformed frames within 300s; assert T0814 at 3rd; parse_errors=3; malformed_in_window=3.

### AC-013 (traces to BC-2.15.024 invariant 1 — parse_errors never reset)
`flow.parse_errors` is a lifetime monotonic counter. After the 300s window expiry reset, `parse_errors` remains at its accumulated value (NOT reset to 0). `malformed_in_window` IS reset to 0 at window expiry.
- **Test:** `test_parse_errors_not_reset_at_window_expiry()` — 3 malformed frames (parse_errors=3); advance past 300s; assert parse_errors still 3, malformed_in_window=0.

### AC-014 (traces to BC-2.15.014 invariant 8 — pending_requests timeout-check uses wrapping_sub)
`now_ts.wrapping_sub(request_ts) > BLOCK_CMD_TIMEOUT_SECS = 10` is the timeout check (not `now_ts - request_ts`). This prevents panic under overflow-checks=true when timestamps go backward (out-of-order pcap replay).
- **Test:** `test_pending_request_timeout_wrapping_sub()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `Dnp3FlowState.block_event_count: u64` | `src/analyzer/dnp3.rs` | Effectful shell state (feeds T0827) |
| `Dnp3FlowState.block_finding_emitted_this_window: bool` | `src/analyzer/dnp3.rs` | One-shot guard |
| `Dnp3FlowState.loss_of_control_emitted: bool` | `src/analyzer/dnp3.rs` | One-shot guard |
| `Dnp3FlowState.correlation_window_start_ts: u32` | `src/analyzer/dnp3.rs` | Shared 300s window start (this BC owns reset) |
| `Dnp3FlowState.malformed_in_window: u64` | `src/analyzer/dnp3.rs` | NEW windowed counter (NOT parse_errors) |
| `Dnp3FlowState.malformed_anomaly_emitted: bool` | `src/analyzer/dnp3.rs` | NEW one-shot guard |
| `Dnp3FlowState.enable_unsolicited_seen: bool` | `src/analyzer/dnp3.rs` | Context flag |
| `Dnp3FlowState.response_seen: bool` | `src/analyzer/dnp3.rs` | Context flag |
| `Dnp3FlowState.unsolicited_anomaly_emitted: bool` | `src/analyzer/dnp3.rs` | One-shot guard |
| `fn is_broadcast_destination(dest: u16) -> bool` | `src/analyzer/dnp3.rs` | Pure helper: `dest >= 0xFFFD` |
| `src/mitre.rs` T1691.001 arm | `src/mitre.rs` | NEW technique entry (atomic with emission) |
| `src/mitre.rs` T0827 arm | `src/mitre.rs` | NEW technique entry (atomic with emission) |
| `MitreTactic::IcsImpact` | `src/mitre.rs` | NEW enum variant (atomic with T0827 emission) |
| `const CORRELATION_WINDOW_SECS: u32 = 300` | `src/analyzer/dnp3.rs` | Window constant |
| `const BLOCK_CMD_TIMEOUT_SECS: u32 = 10` | `src/analyzer/dnp3.rs` | Timeout constant |
| `const BLOCK_CMD_THRESHOLD: u64 = 3` | `src/analyzer/dnp3.rs` | Sustained-pattern threshold |
| `const T0827_THRESHOLD: u64 = 3` | `src/analyzer/dnp3.rs` | Combined-event threshold |
| `const MALFORMED_ANOMALY_THRESHOLD: u64 = 3` | `src/analyzer/dnp3.rs` | Windowed malformed threshold |

Architecture section references: `architecture/module-decomposition.md` (SS-15 correlation window), ADR-007 Decision 5 (all anomaly detections), BC-2.15.015 (single reset owner).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DIRECT_OPERATE_NR (0x06) — no response | Not tracked; `block_event_count` NOT incremented |
| EC-002 | SELECT (0x03) → matching RESPONSE within 10s | Entry removed from pending_requests; no block timeout; `block_event_count` NOT incremented |
| EC-003 | 2 block + 1 restart at t=0, t=150s, t=200s (Trace B) | Both blocks within 300s (not reset at 120s); combined=3; T0827 fires after restart |
| EC-004 | T0827 one-shot: 4th restart same window | `loss_of_control_emitted` prevents second T0827 |
| EC-005 | Broadcast READ (0x01) to 0xFFFF | No broadcast anomaly (only Control-class triggers BC-2.15.018) |
| EC-006 | DISABLE_UNSOLICITED on already-bailed flow (`is_non_dnp3=true`) | Immediate no-op (BC-2.15.009 PC5): no parsing, `parse_errors` NOT incremented, `malformed_in_window` NOT incremented, no Finding emitted, carry untouched. Note: counter increments described in AC-010/AC-012 apply only to frames that arrive on a live (non-bailed) flow. |
| EC-007 | ENABLE_UNSOLICITED before UNSOLICITED_RESPONSE | `enable_unsolicited_seen=true`; no unsolicited anomaly |
| EC-008 | Multiple DISABLE_UNSOLICITED — adversarial flood | Each emits T0814 (per-occurrence) up to MAX_FINDINGS cap |
| EC-009 | 4th malformed frame (same window, guard set) | `parse_errors=4`, `malformed_in_window=4`; NO second T0814 (one-shot guard) |
| EC-010 | Single COLD_RESTART (T0827 requires ≥3) | T0814 emitted; T0827 NOT emitted (invariant: never from single event) |

## Tasks

1. **Add new `Dnp3FlowState` fields**: `block_event_count: u64`, `block_finding_emitted_this_window: bool`, `loss_of_control_emitted: bool`, `correlation_window_start_ts: u32`, `malformed_in_window: u64`, `malformed_anomaly_emitted: bool`, `enable_unsolicited_seen: bool`, `response_seen: bool`, `unsolicited_anomaly_emitted: bool`.
2. **Add new constants**: `CORRELATION_WINDOW_SECS=300`, `BLOCK_CMD_TIMEOUT_SECS=10`, `BLOCK_CMD_THRESHOLD=3`, `T0827_THRESHOLD=3`, `MALFORMED_ANOMALY_THRESHOLD=3`.
3. **Implement pending-request tracking** in `on_data` — on Control-class FC (0x03/0x04/0x05, NOT 0x06), insert `(dest_addr, app_seq)` → `now_ts` into `flow.pending_requests` (with cap/eviction from STORY-107). On RESPONSE (FC=0x81), remove matching entry.
4. **Implement block-timeout scan** in `on_data` — iterate pending_requests; any entry where `now_ts.wrapping_sub(request_ts) > BLOCK_CMD_TIMEOUT_SECS`: increment `block_event_count`, remove entry, check for T1691.001 emission.
5. **Implement 300s correlation window expiry handler** in `on_data` — `wrapping_sub` check; if expired, reset all six windowed fields + update `correlation_window_start_ts`. This handler fires BEFORE emission checks.
6. **Implement T0827 emission** — check `restart_event_count + block_event_count >= T0827_THRESHOLD` within 300s window; push after the triggering direct finding (BC-2.15.013 ordering).
7. **Implement broadcast anomaly** (BC-2.15.018) — `is_broadcast_destination(h.destination)` helper; only fires on Control-class FC.
8. **Implement unsolicited anomaly** (BC-2.15.019) — track `response_seen`, `enable_unsolicited_seen`; one-shot guard.
9. **Implement DISABLE/ENABLE_UNSOLICITED detection** (BC-2.15.023) — raw `app_fc == 0x15 || app_fc == 0x14` check (NOT via classify_dnp3_fc); per-occurrence; severity split.
10. **Implement malformed anomaly** (BC-2.15.024) — add `malformed_in_window += 1` alongside existing `parse_errors += 1` at each structural-reject path; threshold check + one-shot guard.
11. **Add T1691.001, T0827, MitreTactic::IcsImpact to `src/mitre.rs`** atomically with the emission branches (VP-007 obligation).
12. **Unit tests** for AC-001 through AC-014.

## Test Plan

| AC | Test Type | Notes |
|----|-----------|-------|
| AC-001 | Unit | block_event_count unconditional; DIRECT_OPERATE_NR excluded |
| AC-002 | Unit | T1691.001 at 3rd block event; Possible/Low |
| AC-003 | Unit | Block events not reset at 120s (key Trace B test) |
| AC-004 | Unit | T0827 Trace B (2 block + 1 restart); IcsImpact tactic |
| AC-005 | Unit | Six-field reset at 300s; parse_errors not reset |
| AC-006 | Unit | T0827 requires distinct events; 2 restarts alone insufficient |
| AC-007 | Unit | Broadcast anomaly; READ to broadcast negative |
| AC-008 | Unit | Broadcast + burst both retained; dedup by summary not technique ID |
| AC-009 | Unit | Unsolicited anomaly; ENABLE suppresses it |
| AC-010 | Unit | DISABLE_UNSOLICITED Likely/Medium; raw FC check |
| AC-011 | Unit | ENABLE_UNSOLICITED Possible/Low |
| AC-012 | Unit | malformed_in_window threshold 3; T0814 Possible/Low |
| AC-013 | Unit | parse_errors lifetime; not reset at window expiry |
| AC-014 | Unit | wrapping_sub for timeout check |

## Previous Story Intelligence

STORY-108 (Direct Detections) establishes `restart_event_count`, `all_findings`, `MAX_FINDINGS` cap pattern, and `wrapping_sub` timestamp convention. All must be carried forward unchanged.

Key complexity in this story:
- **The single 300s shared correlation window** (BC-2.15.015 owning reset) is the most complex invariant. The reset handler runs at the TOP of `on_data` before any detection logic. This prevents stale state from previous windows affecting new window detections.
- **Two-counter model for malformed frames** (BC-2.15.024 v1.1 fix): `parse_errors` is lifetime/never-reset; `malformed_in_window` is windowed/resets. An adversarial finding (C-2) in the adversarial pass identified the bug where a single counter was used for both purposes — the implementation MUST use two separate fields.
- **Trace B verification** (BC-2.15.015): the canonical test is 2 block timeouts at t=0 and t=150s (both within 300s), then 1 restart at t=200s. Under the old 120s sub-window design, block_event_count would reset at t=120s and T0827 would NOT fire. The test verifies it DOES fire (combined=3).

## Architecture Compliance Rules

1. **`parse_errors` is NEVER reset** — not at window expiry, not anywhere. BC-2.15.024 Invariant 1 is absolute.
2. **`malformed_in_window` IS reset at 300s expiry** — in the same handler as the other five windowed fields (BC-2.15.015 Postcondition 3 six-field reset).
3. **T1691.001 replaces revoked T0803** — never emit T0803. T1691.001 is active in ics-attack-19.1.
4. **T0827 uses `MitreTactic::IcsImpact` (NEW)** — this is a NEW variant; it is NOT `MitreTactic::Impact` (enterprise). These are distinct tactics. The enum variant MUST be added to `MitreTactic` in `src/mitre.rs`.
5. **BC-2.15.023 raw FC check** — `app_fc == 0x14 || app_fc == 0x15` directly; do NOT use `classify_dnp3_fc` for this detection. The classifier correctly returns `Management` for these FCs, but that return value is NOT the detection gate.
6. **wrapping_sub throughout** — all u32 timestamp arithmetic uses `now_ts.wrapping_sub(...)`.
7. **Forbidden dependencies**: `src/analyzer/dnp3.rs` MUST NOT depend on `src/analyzer/modbus.rs`.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `std::collections::HashMap` | stdlib | `pending_requests` (from STORY-107) |
| `src/mitre.rs` | same crate | NEW arms: T1691.001, T0827; NEW MitreTactic::IcsImpact |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/analyzer/dnp3.rs` | Modify | Add 9 new flow-state fields; constants; block timeout; broadcast/unsolicited/DISABLE/malformed branches; T0827 correlation; window-expiry reset handler |
| `src/mitre.rs` | Modify | Add T1691.001 arm, T0827 arm; add MitreTactic::IcsImpact variant — ATOMIC with emission |
| `tests/dnp3_correlation_tests.rs` OR inline | Create/expand | AC-001..AC-014 |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~4,500 |
| 6 BC files (BC-2.15.014, 015, 018, 019, 023, 024) | ~27,000 |
| ADR-007 (Decision 5 anomaly table) | ~3,000 |
| STORY-108 (prior story context) | ~3,500 |
| Existing `src/analyzer/dnp3.rs` | ~5,000 |
| Existing `src/mitre.rs` (for MitreTactic enum context) | ~2,000 |
| Tool outputs | ~2,000 |
| **Total estimated** | **~47,000** |

At the upper end of the 20-30% budget for a ~120k context window. Consider reading only the relevant BC sections if context is tight.

## Dependency Rationale

- `depends_on: [STORY-108]` — `restart_event_count` (set in STORY-108 by the T0814 restart detection branch) is consumed here by the T0827 accumulator. `all_findings`, `MAX_FINDINGS` cap pattern, `direct_operate_count` (broadcast anomaly increments it), and the carry-buffer/frame-processing loop all come from STORY-107/108. The correlation window and pending-request tracking in this story require the complete per-flow state from prior stories.
- `blocks: [STORY-110]` — STORY-110 wires the dispatcher (BC-2.15.021) and CLI flag (BC-2.15.017). It needs a complete `Dnp3Analyzer` that already emits all findings (this story completes the detection surface) before wiring into the StreamDispatcher and CLI argument parser.
