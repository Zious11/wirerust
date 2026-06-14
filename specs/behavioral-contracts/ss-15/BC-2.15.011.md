---
document_type: behavioral-contract
level: L3
version: "1.4"
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
  - "v1.1: Pass-2 adversarial fix CRITICAL-2: collapsed dual-window model (120s BLOCK_CMD_WINDOW + 300s T0827_WINDOW) into a single shared CORRELATION_WINDOW_SECS=300s [F2-GATE]. restart_event_count incremented unconditionally; both restart_event_count and block_event_count reset ONLY at shared 300s window expiry (reset owner: BC-2.15.015 / window-expiry handler). Added correlation_window_start_ts reference to Architecture Anchors. Verified '2 block + 1 restart → T0827' trace fires correctly under single-window model. — 2026-06-10"
  - "v1.2: Pass-3 adversarial fix LOW: EC-007 wording corrected from '2 block events (120–300s apart)' (imprecise) to 'both within the same 300s correlation window (e.g. 150s apart)'. The old wording implied events must be 120–300s apart; the correct invariant is simply that both are within the same 300s window. Also updates matching table entry in Canonical Test Vectors trace note. — 2026-06-10"
  - "v1.4: F3 story-anchor back-fill. — 2026-06-14"
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

# BC-2.15.011: COLD_RESTART/WARM_RESTART Observed — Emits T0814 Per-Occurrence Finding

## Description

When a DNP3 application function code COLD_RESTART (0x0D) or WARM_RESTART (0x0E) is observed
on a FIR=1 fragment, a `Finding` is emitted immediately carrying `T0814` ("Denial of Service").
These restart commands force the outstation to reboot (full or partial), rendering it temporarily
unresponsive and removing operator visibility. Detection is per-occurrence: one finding per
observed restart FC, not gated by a burst threshold. The `restart_event_count` counter is
also incremented unconditionally to feed the T0827 derived-impact accumulator (BC-2.15.015).
Both `restart_event_count` and `block_event_count` share the same 300s correlation window
(CORRELATION_WINDOW_SECS, tracked by `correlation_window_start_ts`); both counters reset
together at window expiry. ADR-007 Decision 5.

## Preconditions

1. The validity gate (BC-2.15.004) returned `true`.
2. `has_user_data(control)` is `true`.
3. `transport_is_fir(transport_octet)` is `true` (FIR=1, BC-2.15.008).
4. `classify_dnp3_fc(app_fc)` returns `Dnp3FcClass::Restart` (BC-2.15.006: FCs 0x0D or 0x0E).
5. `flow.is_non_dnp3 == false`.
6. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

1. Exactly ONE `Finding` is pushed to `self.all_findings`:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::High` (restart commands are rarely issued in normal operations)
   - `summary`: `"DNP3 restart command observed: FC 0x{fc:02X} ({name}) from src={src:#06X} to dest={dest:#06X}"`
     where `{name}` is "COLD_RESTART" or "WARM_RESTART".
   - `evidence`: one entry — `"FC=0x{fc:02X} dest={dest:#06X} src={src:#06X}"`
   - `mitre_techniques: vec!["T0814"]`
   - `source_ip: Some(<source endpoint>)` — resolved from flow_key
   - `timestamp: Some(...)` — pcap-relative capture timestamp
2. `flow.restart_event_count += 1` — unconditional; feeds T0827 correlation (BC-2.15.015).
   (The correlation window `CORRELATION_WINDOW_SECS` is shared with `block_event_count`;
   both counters reset together at window expiry — see BC-2.15.015 for the single reset owner.)
3. `flow.fc_counts.entry(app_fc).or_insert(0) += 1`.
4. `self.fn_code_counts.entry(app_fc).or_insert(0) += 1`.

## Invariants

1. **Per-occurrence detection**: one T0814 finding per observed COLD_RESTART or WARM_RESTART FC,
   unlike the T1692.001 threshold-based window (BC-2.15.010). Restart commands are inherently
   disruptive and individually suspicious. [ADR-007 Decision 5]
2. **T0814 is the correct v19.1 technique** [MITRE: dnp3-research.md §6]: T0814 "Denial of
   Service" is active and unchanged in ics-attack-19.1. Tactic: IcsInhibitResponseFunction.
3. **T0827 accumulation**: every T0814 finding increments `restart_event_count` unconditionally.
   When `restart_event_count + block_event_count >= T0827_THRESHOLD` within the shared
   CORRELATION_WINDOW_SECS (300s), a derived T0827 finding is emitted (BC-2.15.015). The
   T0814 finding is always emitted first.
4. **COLD vs WARM**: both FCs emit T0814 with `Confidence::High`. COLD_RESTART (0x0D) causes
   a full device restart (loses all runtime state); WARM_RESTART (0x0E) causes a partial restart
   (retains some state). Both are denial-of-service triggers from the attacker perspective.
5. **No threshold guard**: unlike BC-2.15.010, restart detection is not gated by a counter.
   Every individual restart FC is individually significant. There is no `restart_emitted` guard.
6. **Single shared correlation window**: `restart_event_count` and `block_event_count` share the
   same `CORRELATION_WINDOW_SECS = 300s` [F2-GATE: human to confirm] window tracked by
   `correlation_window_start_ts: u32`. There is NO separate 120s window for block events.
   Both counters reset to 0 together ONLY at the 300s expiry. Reset owner: BC-2.15.015
   window-expiry handler.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First COLD_RESTART on a flow | Finding immediately: T0814, confidence=High |
| EC-002 | WARM_RESTART immediately following COLD_RESTART (2 findings) | Two separate T0814 findings emitted; `restart_event_count=2` |
| EC-003 | `all_findings.len() == MAX_FINDINGS` when restart arrives | No finding pushed; `restart_event_count` still incremented (feeds T0827 accumulator) |
| EC-004 | FC 0x0F (INITIALIZE_DATA) — looks similar to restart | NOT a Restart-class FC; `classify_dnp3_fc(0x0F)` returns `Management`; no T0814 |
| EC-005 | Restart from broadcast destination | T0814 finding emitted normally; no additional broadcast anomaly (restart to broadcast is inherently anomalous and the T0814 finding already captures this) |
| EC-006 | Multiple COLD_RESTARTs accumulating toward T0827 threshold | N T0814 findings emitted; when `restart_event_count + block_event_count >= T0827_THRESHOLD` within 300s, an additional T0827 finding is pushed (BC-2.15.015) |
| EC-007 | 2 block events both within the same 300s correlation window (e.g. 150s apart) + 1 restart | Under single 300s window: both block events still in window (not reset at 120s); `block_event_count=2` + `restart_event_count=1` = 3 ≥ T0827_THRESHOLD; T0827 fires. This is the key fix: the old 120s BLOCK_CMD_WINDOW_SECS would have reset block_event_count before the restart arrived, suppressing T0827. |

## Canonical Test Vectors

**COLD_RESTART frame (outstation 3, master 1):**
```
DNP3 frame:  05 64 09 C4 03 00 01 00 [hdr-crc]  C0 81 0D  [data-crc]
Link:        START=0x0564, LEN=9, CTRL=0xC4, DEST=0x0003, SRC=0x0001
Transport:   0xC0 (FIR=1, FIN=1)
App FC:      0x0D → COLD_RESTART → Dnp3FcClass::Restart
```
Expected: `Finding { mitre_techniques: ["T0814"], confidence: High, summary: "DNP3 restart command observed: FC 0x0D (COLD_RESTART) from src=0x0001 to dest=0x0003" }`

| FC (hex) | Name | Expected `Finding.mitre_techniques` | Expected `confidence` |
|----------|------|------------------------------------|--------------------|
| `0x0D` | COLD_RESTART | `["T0814"]` | High |
| `0x0E` | WARM_RESTART | `["T0814"]` | High |
| `0x0F` | INITIALIZE_DATA | (no T0814 finding) | N/A |

**T0827 trace — 2 block events (spaced 150s apart) + 1 restart:**

| t (s) | Event | restart_event_count | block_event_count | T0827? |
|-------|-------|--------------------|--------------------|--------|
| 0 | Block timeout #1 (correlation_window_start_ts set to 0) | 0 | 1 | No |
| 150 | Block timeout #2 (150s < 300s; still in same window) | 0 | 2 | No |
| 200 | COLD_RESTART observed | 1 | 2 | **Yes** — 1+2=3 ≥ threshold |

This trace verifies that block events both within the same 300s correlation window (e.g. 150s apart) are NOT reset by any 120s sub-window and correctly accumulate toward T0827.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-023 | Sub-property B (correctness): `classify_dnp3_fc(0x0D)` and `classify_dnp3_fc(0x0E)` return `Restart` | Kani (Sub-B set membership) |
| (none) | Per-occurrence finding emission: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — detecting COLD_RESTART and WARM_RESTART abuse is a core DNP3/ICS threat-detection capability; these commands are the primary mechanism for denial-of-service against DNP3 outstations (Ukraine 2015: Sandworm forced outstation restarts to remove relay visibility) |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — findings emitted only on port-20000 flows with valid DNP3 frames) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24); ADR-007 Decision 5 |
| Stories | STORY-108 |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | T0814 — Denial of Service (ICS; Inhibit Response Function tactic TA0107; active in v19.1) |

## Related BCs

- BC-2.15.006 — depends on (Restart-class FC classification)
- BC-2.15.008 — depends on (FIR=1 gate)
- BC-2.15.015 — composes with (restart_event_count feeds T0827 derived-impact accumulator; shares single 300s correlation window)
- BC-2.15.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `Dnp3Analyzer::on_data` — Restart-class branch
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.restart_event_count: u64` (feeds T0827; shares 300s window)
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.correlation_window_start_ts: u32` (single shared window start; owns the 300s CORRELATION_WINDOW_SECS tracking for BOTH restart_event_count and block_event_count)
- `src/mitre.rs` — `technique_info("T0814")` arm (existing; shared with Modbus force-listen-only)
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §8` (detection table: "DoS via restart COLD_RESTART/WARM_RESTART → T0814")
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 5`

## Story Anchor

STORY-108

## VP Anchors

- VP-023 — Sub-property B (verifies Restart-class classification precondition)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-007 Decision 5; dnp3-research.md §5 (T0814 mapping: "COLD_RESTART 0x0D / WARM_RESTART 0x0E → T0814"; confirmed active in v19.1); §3.2 (FC hex values confirmed [SPEC]) |
| **Confidence** | high — T0814 technique confirmed [MITRE] active; FC values confirmed [SPEC] |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow.restart_event_count, all_findings |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell |
