---
document_type: behavioral-contract
level: L3
version: "2.2"
status: draft
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-14
capability: CAP-14
lifecycle_status: active
introduced: v0.3.0-feature-007
modified:
  - version: "2.0"
    date: 2026-06-09
    change: "UPDATED (v2.0 — Decision 13, f2-fix-directives.md §13.5): T0831 detection is now co-tagged in the per-PDU write finding (mitre_techniques vec includes \"T0831\" alongside T0855+T0836) rather than emitted as a separate Finding object. Discriminator table updated: T0836/T0835/T0831 are union-tagging rules, not priority-suppression rules. Removed all 'T0836 priority suppresses T0835' language. Burst finding for T0806+T0855 is unchanged (separate Finding). Targets v0.3.0."
  - version: "2.1"
    date: 2026-06-09
    change: "F5 spec defect fix: timestamp units corrected microseconds→seconds to match the pipeline's timestamp_secs delivery (BC-2.09.007; StreamHandler::on_data passes seconds, not microseconds). The f2 microsecond-scale assumption (T0831_WINDOW_SECS*1_000_000) was wrong. Window math now uses elapsed_secs = now_ts.wrapping_sub(window_start_ts); expiry check is elapsed_secs > T0831_WINDOW_SECS (5). Canonical test vectors updated to second-scale timestamp values. Note: sub-second rate precision is a future enhancement requiring timestamp_usecs threading through on_data."
  - version: "2.2"
    date: 2026-06-10
    change: "v19 remap: T0855 → T1692.001 per MITRE ATT&CK for ICS v19.0 revocation. All T0855 technique ID references in Technique Union-Tagging Rule Table, Postconditions, Invariants, Edge Cases, and Canonical Test Vectors updated to T1692.001. Tactic unchanged: IcsImpairProcessControl. Issue #222; audit: mitre-ics-v19-catalog-audit.md."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/phase-f2-spec-evolution/f2-fix-directives.md
  - .factory/research/modbus-tcp-research.md
  - .factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md
  - .factory/specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md
input-hash: TBD
---

# BC-2.14.016: Coordinated Write Sequence to Holding Registers Within 5-Second Window Tags the Per-PDU Finding with T0831

<!-- Previous version (v1.0): "Coordinated Write Sequence to Holding Registers Within 5-Second Window Emits T0831 Manipulation of Control Finding"
     v1.0 model: T0831 was emitted as a SEPARATE Finding object with mitre_technique=Some("T0831").
       Per-PDU findings (T0855 + T0836) were ALSO emitted, yielding up to 3 separate findings for
       a T0831-triggering write.
     v2.0 model (Decision 13, §13.5): T0831 is co-tagged inline on the per-PDU write Finding.
       The 2nd+ holding-register write within the 5s window produces ONE finding with
       mitre_techniques: ["T0855","T0836","T0831"]. No separate T0831 Finding object.
       The discriminator table below replaces the v1.0 "priority selection" table with a
       union-tagging rule table. Targets v0.3.0.
-->

## Description

When two or more write FCs targeting holding registers (FC 0x06, 0x10, 0x16, or 0x17) are
observed within the same flow within a 5-second pcap-timestamp window, T0831 ("Manipulation of
Control") is co-tagged on the per-PDU write finding. FC 0x17 (Read/Write Multiple Registers)
is included because it atomically writes holding registers and must not be excluded from the
coordinated-write detector — omitting it allows evasion via the atomic R/W function code.
Per Decision 13 (ADR-006), there is no separate T0831 Finding object: the 2nd (and subsequent)
holding-register write within the active window emits ONE finding with
`mitre_techniques: vec!["T1692.001", "T0836", "T0831"]`. The first write in the window starts the
window accumulation and emits a finding with `["T1692.001", "T0836"]` only (T0831 has not yet
fired). On the 2nd write that tips the sequence detector, the T0831 tag is appended to the
per-PDU finding's `mitre_techniques` vec. Subsequent writes in the same window (after
`t0831_burst_emitted = true`) emit `["T1692.001", "T0836"]` again (no T0831 tag — emit-once per
window overflow). Volume control is via the `t0831_burst_emitted` flag, not via finding
suppression. The five-second window is fixed in v1 (not CLI-configurable).

## Technique Union-Tagging Rule Table (replaces v1.0 discriminator/priority table)

This table is the authoritative co-tagging model for all write-technique BCs (Decision 13):

| FC subset | Per-PDU finding mitre_techniques | Notes |
|-----------|----------------------------------|-------|
| FC {0x06, 0x10, 0x16, 0x17} — 1st write in flow, or after T0831 window reset | `["T1692.001", "T0836"]` | T0831 not yet fired in this window |
| FC {0x06, 0x10, 0x16, 0x17} — 2nd write within 5s window (T0831 fires) | `["T1692.001", "T0836", "T0831"]` | T0831 co-tagged once per window overflow |
| FC {0x06, 0x10, 0x16, 0x17} — 3rd+ write in same window (after T0831 fired) | `["T1692.001", "T0836"]` | T0831 emit-once exhausted; burst_emitted=true |
| FC {0x05, 0x0F} — coil write (never contributes to T0831 window) | `["T1692.001", "T0835"]` | T0831 inapplicable to coil-write FCs |
| Burst threshold tripped (any write FC) | Separate Finding: `["T0806", "T1692.001"]` | Burst finding is independent; emitted alongside per-PDU finding |

**No priority, no suppression.** T0836 and T0835 are FC-subset exclusive (not competing).
T0831 is co-tagged when the window condition fires (not a separate priority tier).
"T0836 priority suppresses T0835" language from v1.0 is SUPERSEDED and must not appear
in implementation comments, tests, or downstream documents.

## Preconditions

1. The MBAP ADU has passed the three-point validity gate.
2. The TCP direction is `Direction::ClientToServer`.
3. `function_code` is one of: `0x06`, `0x10`, `0x16`, `0x17`.
   FC 0x17 (Read/Write Multiple Registers) is included because it atomically writes holding
   registers and must not be excluded from the T0831 coordinated-write window.
4. The window-update logic (see Invariant 2) runs FIRST on every holding-register write,
   unconditionally. The T0831 co-tag is applied to the per-PDU finding only when the
   post-update `t0831_window_write_count >= 2` AND `t0831_burst_emitted == false`.
5. `self.all_findings.len() < MAX_FINDINGS`.
6. `flow.t0831_burst_emitted == false` (T0831 tag is added at most once per window overflow).

## Postconditions

1. The per-PDU `Finding` (as specified in BC-2.14.013 / BC-2.14.014 for holding-register
   writes) has T0831 appended to its `mitre_techniques` vec when the T0831 window condition
   is met:
   - `mitre_techniques: vec!["T1692.001", "T0836", "T0831"]`
   - All other fields unchanged from BC-2.14.013 postcondition 1.
2. `flow.t0831_burst_emitted = true` (prevents T0831 co-tagging on subsequent writes in the
   same window).
3. `flow.t0831_window_write_count` is incremented (the current write is counted).
4. `flow.write_count` and `self.total_write_count` incremented once.
5. **No separate T0831 Finding object is created.** The T0831 attribution is co-tagged
   inline on the per-PDU finding. `all_findings` receives exactly one new entry for the
   T0831-triggering write (not two or three as in v1.0).

## Invariants

1. **Window state fields** (on `ModbusFlowState` — per architecture-delta.md §2.3):
   - `t0831_window_start_ts: u32` — timestamp of first holding-register write in current window.
   - `t0831_window_write_count: u32` — count of holding-register writes in current window.
   - `t0831_burst_emitted: bool` — true once T0831 has co-tagged in the current window.
2. **Canonical evaluation ORDER** (window-update FIRST, then emission check):
   On every holding-register write FC (0x06, 0x10, 0x16, 0x17) in ClientToServer direction:
   ```
   // STEP 1: Window-update runs FIRST on every qualifying write, unconditionally.
   // Qualifying FCs: {0x06, 0x10, 0x16, 0x17}
   // NOTE: now_ts is timestamp_secs (seconds, per BC-2.09.007 / pipeline delivery).
   //   wrapping_sub is used for u32 second timestamps; at ~136 years of capture time
   //   these will never overflow in practice, but the policy is kept for correctness.
   elapsed_secs = now_ts.wrapping_sub(t0831_window_start_ts)
   if elapsed_secs > T0831_WINDOW_SECS:
       // Window expired: reset (this write starts a new window)
       t0831_window_start_ts = now_ts
       t0831_window_write_count = 1
       t0831_burst_emitted = false
   else:
       // Still in window: increment
       t0831_window_write_count += 1

   // STEP 2: Determine mitre_techniques for the per-PDU finding.
   if t0831_window_write_count >= 2 AND NOT t0831_burst_emitted:
       mitre_techniques = vec!["T1692.001", "T0836", "T0831"]
       t0831_burst_emitted = true
   else:
       mitre_techniques = vec!["T1692.001", "T0836"]

   // STEP 3: Push ONE finding with the determined mitre_techniques.
   push Finding { mitre_techniques, ... }
   ```
   **Critical ordering rule**: the window-update (Step 1) ALWAYS runs before the tag
   determination (Step 2). This ensures the count-establishing write (first write, count 0→1)
   is tracked even though it does NOT trigger T0831 co-tagging.
3. **T0831 co-tags at most once per 5-second window per flow.** Subsequent holding-register
   writes within the same window do not re-include the T0831 tag.
4. **`T0831_WINDOW_SECS = 5`** constant is fixed in v1 (not CLI-configurable).
   Window expiry check: `elapsed_secs > T0831_WINDOW_SECS` where
   `elapsed_secs = now_ts.wrapping_sub(t0831_window_start_ts)` and `now_ts` is in SECONDS
   (the pipeline delivers `timestamp_secs` per BC-2.09.007 — NOT microseconds).
   Sub-second rate precision is a future enhancement; it would require threading
   `timestamp_usecs` through `on_data`, which is not in v1 scope.
5. A single write FC from a fresh flow (no prior holding-register writes) never triggers T0831.
   The minimum condition is two writes within the window.
6. T0831 applies to the same FC subset as T0836 ({0x06, 0x10, 0x16, 0x17}). FC 0x17 (Read/Write
   Multiple Registers) is classified as holding-register Write and contributes to the T0831
   window. Coil writes (T0835 subset {0x05, 0x0F}) do NOT contribute to the T0831 window
   counter or trigger T0831. Excluding FC 0x17 would allow evasion of coordinated-write
   detection via the atomic read/write function code.
7. **No double-counting concern with per-PDU T1692.001/T0836:** since T0831 is co-tagged on
   the same per-PDU finding (not a separate finding), there is no "T0831 on top of T1692.001+T0836
   for same PDU as separate objects" scenario. One PDU → one finding, always.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First holding-register write in a flow | `t0831_window_write_count = 1`, `t0831_window_start_ts = now_ts`. ONE finding: `mitre_techniques=["T1692.001","T0836"]` (T0831 not yet fired). |
| EC-002 | Second holding-register write within 5 seconds of the first | ONE finding: `mitre_techniques=["T1692.001","T0836","T0831"]`. `t0831_burst_emitted = true`. |
| EC-003 | Third write within the same window | ONE finding: `mitre_techniques=["T1692.001","T0836"]` (T0831 emit-once exhausted; `t0831_burst_emitted == true`). `t0831_window_write_count = 3`. |
| EC-004 | Write at exactly `t0831_window_start_ts + 5` seconds (boundary, elapsed_secs = 5) | Window NOT expired (condition is `>`, not `>=`). Counted toward T0831. |
| EC-005 | Write at `t0831_window_start_ts + 6` seconds (elapsed_secs = 6 > 5) | Window expired. Reset. This write starts a new window (`count=1`). `mitre_techniques=["T1692.001","T0836"]` (no T0831 yet). |
| EC-006 | Coil write (FC 0x05) between two holding-register writes | Coil writes do NOT increment `t0831_window_write_count`. Only FCs {0x06, 0x10, 0x16, 0x17} count toward T0831. |
| EC-011 | FC 0x17 (Read/Write Multiple Registers) as the 2nd write within 5 seconds | `t0831_window_write_count=2` ≥ 2 → T0831 co-tags: ONE finding `["T1692.001","T0836","T0831"]`; `t0831_burst_emitted=true`. Evasion attempt via atomic R/W FC is detected. |
| EC-012 | FC 0x06 first write, then FC 0x17 second write within 5 seconds | Same as EC-011: 0x17 counted in the T0831 window; T0831 co-tags on the 0x17 write. |
| EC-007 | `all_findings.len() == MAX_FINDINGS - 1` when T0831 would co-tag | The one finding with `["T1692.001","T0836","T0831"]` fills the last slot. `t0831_burst_emitted` still set to true. |
| EC-008 | Two flows with overlapping timestamps; second flow gets two writes within 5 seconds | T0831 co-tags for the second flow's 2nd write; first flow is unaffected (per-flow state isolation). |
| EC-009 | First holding-register write on a fresh flow (t0831_window_write_count is 0) | Window-update runs: count becomes 1, window_start_ts = now_ts, burst_emitted = false. Tag determination: count = 1 < 2 → NO T0831 co-tag. ONE finding with `["T1692.001","T0836"]`. |
| EC-010 | now_ts < t0831_window_start_ts (timestamp wrap-around or out-of-order packet; timestamps in seconds) | `now_ts.wrapping_sub(t0831_window_start_ts)` yields a very large u32 value (≫ 5 seconds). Window-expiry check fires (resets window). This write starts a new window (count=1). No T0831 co-tag on this write. Evasion-resistant: attacker forcing a window reset at most delays T0831 by one write. |

## Canonical Test Vectors

All timestamps are in SECONDS (timestamp_secs per BC-2.09.007; the pipeline delivers seconds, not microseconds).

| Input | Expected Output | Category |
|-------|----------------|----------|
| Flow A: write 1 at ts=1000s (FC=0x06, UnitID=1) → write 2 at ts=1002s (FC=0x10, UnitID=1); elapsed=2s ≤ 5 | Write1: ONE Finding `mitre_techniques=["T1692.001","T0836"]`; Write2: ONE Finding `mitre_techniques=["T1692.001","T0836","T0831"]` (T0831 co-tagged, count=2, elapsed_secs=2) | happy-path (T0831 co-tags on 2nd write) |
| Flow A: write 1 at ts=1000s → write 2 at ts=1007s (7 seconds later; elapsed=7s > 5) | Write1: `["T1692.001","T0836"]`; Write2: window expired (reset), ONE Finding `["T1692.001","T0836"]` (count=1 in new window; no T0831) | edge-case (expired window) |
| Flow A: three writes at ts=1000s, 1001s, 1002s (all FC=0x06) | Write1: `["T1692.001","T0836"]`; Write2: `["T1692.001","T0836","T0831"]` (burst_emitted=true); Write3: `["T1692.001","T0836"]` (T0831 exhausted for this window) | edge-case (once-per-window, emit-once) |
| Flow A: FC=0x05 (coil) at ts=1000s; FC=0x06 at ts=1002s | Coil write: ONE Finding `["T1692.001","T0835"]` (T0831 window NOT started by coil); Holding-register write at ts=1002s: ONE Finding `["T1692.001","T0836"]` (count=1, no T0831 yet) | edge-case (coil excluded from T0831 window) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | classify_fc total and Write-class completeness | Kani |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC implements the coordinated-sequence detector for process control manipulation, the highest-complexity ICS attack pattern in the approved v1 scope |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-14 (analyzer/modbus.rs, C-22; ModbusFlowState t0831_* fields) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Technique | T0831 — Manipulation of Control (ATT&CK for ICS; IcsImpairProcessControl tactic) |

## Related BCs

- BC-2.14.013 — composes with (T0831 is co-tagged in the per-PDU finding defined there; no separate Finding object)
- BC-2.14.014 — composes with (T0836 is always co-tagged alongside T0831 for holding-register writes)
- BC-2.14.017 — related to (T0806 burst uses a separate 1-second window; T0831 uses 5-second window; both run independently in the same flow)
- BC-2.14.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `ModbusFlowState` with `t0831_window_start_ts`, `t0831_window_write_count`, `t0831_burst_emitted`
- `src/analyzer/modbus.rs` — T0831 coordination tag logic in `on_data` holding-register branch: appends "T0831" to mitre_techniques vec
- `src/mitre.rs` — `technique_info("T0831")` arm (new per ADR-005 §4.2)
- `.factory/specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Kani: Write-class sub-property B

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | f2-fix-directives.md §13.5 (T0831 burst finding `["T0831"]` — revised: T0831 is co-tagged on per-PDU finding, one finding per PDU not per technique); ADR-006 §union-tagging; architecture-delta.md §12 (T0831 scoping note: v1 heuristic two writes within 5s window) |
| **Confidence** | medium (v1 heuristic is intentionally loose per §12 simplification note) |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Deterministic** | yes (same PDU sequence + timestamps always produces same output) |
| **Overall classification** | effectful shell |
