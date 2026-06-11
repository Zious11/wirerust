---
document_type: behavioral-contract
level: L3
version: "1.1"
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
introduced: v0.5.0-feature-008
modified:
  - "v1.1: Adversarial finding C-2 fix: replaced windowed use of parse_errors with a SEPARATE
    windowed counter malformed_in_window (new field). parse_errors is now lifetime/monotonic
    ONLY — incremented on every malformed frame, NEVER reset at window expiry, so BC-2.15.020
    summarize() continues to report the correct lifetime total. malformed_in_window is the new
    windowed counter used for threshold checks; it resets to 0 at 300s window expiry (owned by
    BC-2.15.015). malformed_anomaly_emitted bool guard also resets at window expiry. Added
    MALFORMED_ANOMALY_THRESHOLD const = 3. Updated Description, Preconditions, Postconditions,
    Invariants, Edge Cases, Architecture Anchors, and Purity Classification accordingly.
    EC-005 re-verified: window expiry resets malformed_in_window=0 + guard=false, allowing a
    fresh accumulation to 3 and a new T0814 emission in the next window, while parse_errors
    continues to reflect the lifetime total. BC-2.15.015 owns the reset of malformed_in_window
    and malformed_anomaly_emitted (added in BC-2.15.015 v1.5). — 2026-06-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/dnp3-architecture-delta.md
  - .factory/research/dnp3-research.md
  - .factory/research/dnp3-f2-scope-threshold-validation.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
input-hash: TBD
---

# BC-2.15.024: Malformed/Structural DNP3 Anomaly — malformed_in_window Threshold Emits T0814

## Description

When the DNP3 parser's existing structural-reject paths (LENGTH<5 in BC-2.15.004, frame-length /
`num_data_blocks` mismatch in BC-2.15.007, sync-loss / carry-overflow in BC-2.15.016) reject a
frame, a T0814 anomaly finding is emitted when the **windowed** malformed-frame counter
(`malformed_in_window`) reaches the per-window threshold. This is the ONLY coverage for the
Crain-Sistrunk malformed-frame outstation-crash class: ~28-30 DNP3 vulnerabilities across 16+
ICS-CERT advisories (Project Robus, S4x14) are caused by structurally malformed frames — ASDUs
too short to hold a valid object header, LENGTH/block-count mismatches, unhandled parser states.
Critically, **these frames carry VALID CRCs** (dnp3-f2-scope-threshold-validation.md §Q1(c):
"Crain/Sistrunk frames carry *correct* CRCs so CRC validation would not have caught them
anyway"). CRC deferral and malformed-frame detection are therefore INDEPENDENT concerns —
deferring CRC validation does NOT excuse the malformed-frame blind spot.

**Two-counter model (v1.1 fix, adversarial finding C-2):**
- `parse_errors: u64` — **LIFETIME / monotonic counter**. Incremented on every malformed frame
  by the existing reject paths (BC-2.15.016 Postcondition 2, BC-2.15.004 validity-gate reject,
  BC-2.15.009 sync-loss bail). **NEVER reset at window expiry.** Used exclusively by
  BC-2.15.020 `summarize()` to report the total lifetime structural-error count. This counter
  is already defined and incremented by existing code; this BC does NOT add new increments to it.
- `malformed_in_window: u64` — **NEW windowed counter**. Incremented on each malformed frame
  (in parallel with `parse_errors`). Reset to 0 at each 300s window expiry (owned by
  BC-2.15.015). **This is the counter used for all threshold checks.**
- `malformed_anomaly_emitted: bool` — **NEW one-shot guard**. Set true when a T0814 finding is
  emitted within a window; reset to false at 300s window expiry (owned by BC-2.15.015).
- `const MALFORMED_ANOMALY_THRESHOLD: u64 = 3` — **NEW constant** **[F2-GATE-DEFAULT]**.

Rationale for ≥3/300s: a single malformed frame is consistent with normal packet loss or a
briefly corrupt stream; three or more within 300s on a single flow is anomalous and consistent
with a scanning or crash-injection attempt. Operators may lower this threshold in high-noise
environments if 3/300s produces too many false positives on poorly-maintained equipment.

**Window:** reuses the existing shared `CORRELATION_WINDOW_SECS = 300s` and
`correlation_window_start_ts` field (owned by BC-2.15.015). No new window is added.
`malformed_in_window` and `malformed_anomaly_emitted` reset together with all other correlated
state at the 300s window expiry (BC-2.15.015 owns the single reset). `parse_errors` is NOT in
the reset set — it is lifetime.

## Preconditions

1. One of the existing structural-reject paths has just fired, causing:
   - `flow.parse_errors += 1` (lifetime counter — already incremented by the existing reject
     path; this BC does not change that increment), AND
   - `flow.malformed_in_window += 1` (new windowed counter — incremented HERE, in parallel).
   The three existing reject paths are:
   - BC-2.15.016 Postcondition 2 (carry overflow / LENGTH byte truncation), **or**
   - BC-2.15.004 validity gate reject (LENGTH<5 or sync!=0x0564), **or**
   - BC-2.15.009 sync-loss bail (no valid sync in first 16 bytes, is_non_dnp3 set).
2. `flow.malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD` **[F2-GATE-DEFAULT: proposed ≥3]**
   after the increment in Precondition 1 (threshold check uses `malformed_in_window`, NOT
   `parse_errors`).
3. The accumulated errors occurred within `CORRELATION_WINDOW_SECS` (300s) of
   `flow.correlation_window_start_ts` — i.e., `now_ts.wrapping_sub(flow.correlation_window_start_ts) < CORRELATION_WINDOW_SECS`.
   // wrapping_sub used for u32 second timestamps; wrap at ~136 years — effectively never, policy kept.
4. `flow.malformed_anomaly_emitted == false` (one-shot guard within the correlation window).
5. `self.all_findings.len() < MAX_FINDINGS`.
6. `flow.is_non_dnp3 == false` at the time of finding emission. (The is_non_dnp3 bail in
   BC-2.15.009 sets this flag permanently — a bailed flow still has both `parse_errors` and
   `malformed_in_window` incremented for the triggering event but will not emit further findings
   after bail.)

## Postconditions

**Unconditional (on every malformed frame — on every structural-reject path fire):**
1. `flow.parse_errors += 1`. (This always happens in the existing reject paths. This BC
   does NOT change those increments — they were already there.)
2. `flow.malformed_in_window += 1`. (NEW: windowed counter incremented in parallel with
   `parse_errors` on every malformed frame. This is the increment added by this BC.)

**Conditional — finding emission (when Preconditions 2–5 are met after increment):**
3. Exactly ONE `Finding` is pushed to `self.all_findings`:
   - `category: ThreatCategory::Anomaly`
   - `verdict: Verdict::Possible`
   - `confidence: Confidence::Low`
   - `summary`: `"DNP3 structural anomaly: {count} malformed frames in {elapsed}s window (flow {src_ip}→{dest_ip}) — possible Crain-Sistrunk crash-probe"`
   - `evidence`: one entry — `"malformed_in_window={count} in correlation window; threshold={threshold}"`
   - `mitre_techniques: vec!["T0814"]`
   - `source_ip: Some(...)`, `timestamp: Some(...)`
4. `flow.malformed_anomaly_emitted = true` (one-shot guard: at most one T0814 malformed-anomaly
   finding per `CORRELATION_WINDOW_SECS` per flow).

**Window-expiry reset (owned by BC-2.15.015 — this BC does NOT own the reset):**
5. When `now_ts.wrapping_sub(flow.correlation_window_start_ts) >= CORRELATION_WINDOW_SECS`,
   BC-2.15.015 resets ALL windowed correlated state. This BC's windowed fields are added to
   that reset set (in BC-2.15.015 v1.5):
   - `flow.malformed_in_window = 0` (windowed counter reset to 0 at window expiry)
   - `flow.malformed_anomaly_emitted = false` (guard reset at window expiry)
   **`flow.parse_errors` is NOT reset — it is a lifetime/monotonic counter. BC-2.15.015 does
   NOT include `parse_errors` in the reset set. BC-2.15.020 summarize() reports the lifetime
   `parse_errors` total, which is unaffected by window expiry.**
   These resets are executed by BC-2.15.015's single reset owner, not here.

## Invariants

1. **parse_errors is LIFETIME — incremented by existing paths, NEVER reset at window expiry**:
   `parse_errors` is fed unconditionally by BC-2.15.016 Postcondition 2, BC-2.15.004
   validity-gate reject, and BC-2.15.009 sync-loss bail. BC-2.15.015 does NOT reset
   `parse_errors` at window expiry. BC-2.15.020 `summarize()` reports the lifetime structural-
   error count from `parse_errors` — this total is monotonically non-decreasing over the flow
   lifetime. Resetting `parse_errors` at 300s would corrupt the lifetime summary.
2. **malformed_in_window is WINDOWED — incremented in parallel with parse_errors, RESET at
   window expiry**: `malformed_in_window` is a separate counter, incremented on each malformed
   frame alongside `parse_errors` (both incremented on the same events). BC-2.15.015 resets
   `malformed_in_window = 0` at every 300s window expiry. **All threshold checks use
   `malformed_in_window`, NOT `parse_errors`.**
3. **Distinct from CRC validation**: CRC-16/DNP validation is deferred (dnp3-architecture-delta.md;
   ADR-007). This BC covers STRUCTURAL malformation (LENGTH<5, frame-length/block-count
   mismatch, sync-loss) — not CRC corruption. Critically, Crain-Sistrunk attack frames carry
   **valid CRCs** [VERIFIED: dnp3-f2-scope-threshold-validation.md §Q1(c)], so CRC validation
   would NOT have caught them. The structural-reject paths that this BC surfaces are therefore
   the ONLY coverage for the Crain-Sistrunk crash class. "CRC deferred" must NOT be read as
   "malformed-frame coverage deferred" — these are orthogonal concerns.
4. **T0814 is the correct v19.1 technique**: T0814 "Denial of Service" (IcsInhibitResponseFunction
   TA0107) is the appropriate mapping because Crain-Sistrunk malformed frames are crash/DoS
   vectors — they are designed to crash or wedge the outstation, denying legitimate service.
   T0814 is already seeded and emitted (BC-2.15.011, BC-2.15.023). Catalog counts 23/15/8
   remain UNCHANGED.
5. **Low confidence**: `Verdict::Possible`, `Confidence::Low`. Malformed frames can be caused
   by packet corruption, capture-interface errors, or misclassified non-DNP3 traffic, not only
   by adversarial crafting. The low confidence communicates this uncertainty. At threshold
   ≥3/300s, the finding is a weak anomaly signal, not a high-confidence attack attribution.
6. **One-shot guard**: `malformed_anomaly_emitted` prevents repeated T0814 malformed-frame
   findings within the same 300s window. Only one such finding is emitted per (flow, window).
7. **Shared 300s window — no new window added**: `malformed_in_window` and
   `malformed_anomaly_emitted` are part of the shared per-flow correlation state
   (`correlation_window_start_ts`). They reset to 0/false together with `restart_event_count`,
   `block_event_count`, `block_finding_emitted_this_window`, and `loss_of_control_emitted` at
   the 300s expiry — single reset owner BC-2.15.015. `parse_errors` is explicitly NOT in this
   reset set. This preserves the invariant of the single-window model and the integrity of the
   lifetime summary counter.
8. **malformed_in_window does NOT feed T0827**: `restart_event_count + block_event_count` is
   the T0827 accumulator. Malformed-frame events are NOT added to this sum. Malformed-frame
   anomaly is a separate, independent signal from the loss-of-control correlation model.
9. **DoS-bounded**: `parse_errors` and `malformed_in_window` are both `u64` (no practical
   overflow). `malformed_anomaly_emitted` is a bool one-shot guard. The 300s window reset
   bounds `malformed_in_window` at ≤300s worth of frames per window.
10. **Deep object-level malformation deferred to v2**: this BC catches the STRUCTURAL reject
    paths the parser already computes (length, sync, carry). Object-count-vs-payload
    consistency, qualifier validation, and short-ASDU object header checks are v2 scope.
    [JUDGMENT: dnp3-f2-scope-threshold-validation.md §Q1 GAP-2]
11. **BC-2.15.006, VP-023, classify_dnp3_fc UNCHANGED**: malformed-frame detection does not
    involve FC classification. This BC adds new flow-state fields and a new emission branch.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First malformed frame (malformed_in_window=1, threshold=3) | parse_errors incremented to 1 (lifetime); malformed_in_window incremented to 1 (windowed); no finding (below threshold) |
| EC-002 | Second malformed frame (malformed_in_window=2) | parse_errors incremented to 2; malformed_in_window incremented to 2; no finding |
| EC-003 | Third malformed frame (malformed_in_window=3, threshold=3) | parse_errors incremented to 3; malformed_in_window incremented to 3; T0814 Possible/Low finding emitted; malformed_anomaly_emitted=true |
| EC-004 | Fourth malformed frame (same window, malformed_anomaly_emitted=true) | parse_errors incremented to 4; malformed_in_window incremented to 4; NO additional finding (one-shot guard) |
| EC-005 | 300s window expires; then 3 more malformed frames | BC-2.15.015 resets: malformed_in_window=0, malformed_anomaly_emitted=false (parse_errors is NOT reset — it remains at lifetime total). Then: 3 new malformed frames each increment BOTH parse_errors (lifetime continues from prior value) AND malformed_in_window (fresh from 0). At malformed_in_window=3: new T0814 emitted; malformed_anomaly_emitted=true. BC-2.15.020 summary reports the updated lifetime parse_errors total, unaffected by the window expiry. |
| EC-006 | Malformed frame on is_non_dnp3 bailed flow | parse_errors and malformed_in_window both incremented (existing + new behavior); malformed_anomaly_emitted check skipped (Precondition 6: flow is bailed) |
| EC-007 | `all_findings.len() == MAX_FINDINGS` when threshold crossed | No finding pushed (DoS cap); malformed_anomaly_emitted NOT set (per BC-2.15.022 EC-002 pattern: guard not set when first fire dropped at MAX_FINDINGS) |
| EC-008 | Single malformed frame (corrupt packet on a noisy link) | parse_errors=1; malformed_in_window=1; no finding — threshold requires ≥3 to distinguish noise from adversarial probing |
| EC-009 | Valid frame interleaved with malformed frames | Valid frames do NOT decrement parse_errors or malformed_in_window; both counters are monotonically non-decreasing within a window |

## Canonical Test Vectors

**Length-invalid frame (LENGTH=2, below minimum of 5):**
```
DNP3 bytes:  05 64 02 44 03 00 01 00  [truncated]
Link:        START=0x0564 (valid sync), LEN=2 (LENGTH<5 → validity gate REJECT)
             DEST=0x0003, SRC=0x0001
```
Expected: `flow.parse_errors += 1` (lifetime, BC-2.15.004 / BC-2.15.016 existing behavior) AND
`flow.malformed_in_window += 1` (windowed, new). If this is the 3rd malformed frame in the
300s window (malformed_in_window reaches 3): T0814 finding emitted:
`{ verdict: Possible, confidence: Low, mitre_techniques: ["T0814"], summary: "DNP3 structural anomaly: 3 malformed frames in Xs window ..." }`

**Frame-length mismatch (LENGTH byte says N but fewer bytes present):**
```
DNP3 bytes:  05 64 0F 44 03 00 01 00  [only 8 bytes total, LEN=0x0F=15 claims more]
```
Expected: parser cannot consume a complete frame; carry-overflow or sync-loss path triggers;
`parse_errors += 1` (lifetime) AND `malformed_in_window += 1` (windowed).

| Scenario | parse_errors (lifetime) | malformed_in_window (windowed) | Finding emitted? | Notes |
|----------|------------------------|-------------------------------|-----------------|-------|
| 1 malformed frame (threshold=3) | 1 | 1 | No | Below threshold |
| 2 malformed frames | 2 | 2 | No | Below threshold |
| 3 malformed frames (threshold crossed) | 3 | 3 | **Yes** — T0814 Possible/Low | malformed_anomaly_emitted=true |
| 4th malformed frame (same window) | 4 | 4 | No | One-shot guard |
| 3 valid frames between malformed events | still N (not decremented) | still M (not decremented) | (depends on M vs threshold) | neither counter decremented by valid frames |
| Window expiry (300s) | unchanged (lifetime) | reset to 0 | — | parse_errors stays at N; malformed_in_window=0; malformed_anomaly_emitted=false |
| 3 more malformed frames after expiry | N+1, N+2, N+3 | 1, 2, 3 | **Yes** at malformed_in_window=3 — new window | Second T0814 in new window; parse_errors continues from N+3 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | parse_errors accumulation, threshold check, one-shot guard: effectful shell; unit test | unit test |

Note: VP-023 Sub-B and Sub-D (BC-2.15.006/007) are NOT verification anchors for this BC.
Sub-B covers FC classification (unchanged); Sub-D covers frame_len arithmetic (unchanged).
This BC adds a threshold-based emission on an existing counter — test-sufficient, not a
Kani formal verification target.

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — surfacing malformed/structural DNP3 anomalies is a required DNP3/ICS threat-detection capability: the Crain-Sistrunk "Project Robus" class (~28-30 DNP3 vulns, 16+ ICS-CERT advisories) is caused entirely by structurally malformed frames that crash outstations via DoS; the parser's existing reject paths already detect these conditions but previously discarded them silently; this BC surfaces them as a low-confidence T0814 signal [VERIFIED: dnp3-f2-scope-threshold-validation.md §Q1 GAP-2] |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — findings emitted only on port-20000 flows that reached the DNP3 parser) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-23); ADR-007 Decision 5 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | T0814 — Denial of Service (ICS; Inhibit Response Function tactic TA0107; active in v19.1). No new technique: T0814 is already seeded and emitted. Catalog counts 23/15/8 unchanged. |
| Research Source | dnp3-f2-scope-threshold-validation.md §Q1 GAP-2 [VERIFIED gap, JUDGMENT on v1 scoping]: "passive analyzer blind to the single most-documented DNP3 attack class of the last decade"; §Q1(c): "Crain/Sistrunk frames carry *correct* CRCs — CRC validation would NOT have caught them"; Crain & Sistrunk S4x14 [VERIFIED]: ~28-30 vulns, 16+ ICS-CERT advisories, ASDUs too short, LENGTH/block-count mismatch, invalid parser states |

## Related BCs

- BC-2.15.002 — depends on (DL header rejected for frame shorter than 10 bytes — one source of parse_errors increment)
- BC-2.15.004 — depends on (validity gate reject for LENGTH<5 — primary source of parse_errors increment; Precondition 1 path A)
- BC-2.15.007 — depends on (frame_len arithmetic; mismatch between LENGTH byte and block count — source of parse_errors increment; Precondition 1 path B)
- BC-2.15.009 — depends on (sync-loss bail — source of parse_errors increment; Precondition 1 path C)
- BC-2.15.015 — composes with (single reset owner for shared 300s correlation window; parse_errors and malformed_anomaly_emitted reset at 300s expiry by BC-2.15.015)
- BC-2.15.016 — depends on (parse_errors is defined in Dnp3FlowState; BC-2.15.016 Postcondition 2 is one increment path; this BC adds the emission check on top of the existing counter)
- BC-2.15.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `Dnp3FlowState.parse_errors: u64` (EXISTING field; LIFETIME/monotonic counter; incremented by existing reject paths on every malformed frame; **NEVER reset at window expiry**; reported by BC-2.15.020 summarize())
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.malformed_in_window: u64` (NEW field; WINDOWED counter; incremented on each malformed frame in parallel with parse_errors; reset to 0 at 300s window expiry by BC-2.15.015; **used for all threshold checks**)
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.malformed_anomaly_emitted: bool` (NEW field; one-shot guard; reset to false at 300s window expiry by BC-2.15.015)
- `src/analyzer/dnp3.rs` — `const MALFORMED_ANOMALY_THRESHOLD: u64 = 3` (NEW constant; **[F2-GATE-DEFAULT]**)
- `src/analyzer/dnp3.rs` — on each structural-reject path: `flow.malformed_in_window += 1;` (NEW, added alongside existing `flow.parse_errors += 1`)
- `src/analyzer/dnp3.rs` — emission check after each increment site: `if flow.malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD && !flow.malformed_anomaly_emitted && /* window check */ { /* emit T0814 */ }`
- `src/analyzer/dnp3.rs` — BC-2.15.015 window-expiry reset handler: add `flow.malformed_in_window = 0; flow.malformed_anomaly_emitted = false;` to the existing reset block (parse_errors is NOT added to that block)
- `src/mitre.rs` — `technique_info("T0814")` arm (existing; shared)
- `.factory/research/dnp3-f2-scope-threshold-validation.md §Q1 GAP-2` (research validation: must-add detection)
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 5` (detection extension)

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

(none — effectful shell; threshold/guard logic verified by unit test; VP-023 Sub-B/D are unchanged and cover orthogonal properties)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | dnp3-f2-scope-threshold-validation.md §Q1 GAP-2 [VERIFIED gap, JUDGMENT on 3/300s default]; Crain & Sistrunk S4x14 / Project Robus [VERIFIED]: ~28-30 vulns, 16+ ICS-CERT advisories, malformed-frame crash vectors; dnp3-f2-scope-threshold-validation.md §Q1(c) [VERIFIED]: Crain-Sistrunk frames carry valid CRCs; dnp3-research.md §1.1 [VERIFIED]: LENGTH<5 reject; dnp3-research.md §4 [VERIFIED]: frame_len arithmetic |
| **Confidence** | medium — gap confirmed [VERIFIED]; threshold 3/300s is [JUDGMENT] ([F2-GATE-DEFAULT]); malformed-frame T0814 mapping is defensible engineering (DoS via crash) |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | increments flow.parse_errors (lifetime; existing paths already do this) + flow.malformed_in_window (windowed; new); reads flow.malformed_in_window for threshold check; writes flow.malformed_anomaly_emitted, all_findings |
| **Deterministic** | yes — same frame sequence produces same threshold trigger |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell |
