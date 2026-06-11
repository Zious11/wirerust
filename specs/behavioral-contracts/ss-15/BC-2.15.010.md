---
document_type: behavioral-contract
level: L3
version: "1.2"
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
  - "v1.1: Pass-1 adversarial fix C-2: corrected tactic cardinality — T1692.001 maps to exactly ONE MitreTactic (IcsImpairProcessControl); removed erroneous '+ Evasion' from Traceability MITRE Techniques field. The technique_info table is single-tactic per entry. — 2026-06-10"
  - "v1.2: Research threshold clarification (dnp3-f2-scope-threshold-validation.md §Q1 Threshold-1): clarified the semantic role of the 10/60s threshold — it is a flood/burst guard for the allowlisted-but-abnormally-busy case, NOT the primary unauthorized-source detector. Unauthorized control from an UNEXPECTED SOURCE ADDRESS fires at count=1, independent of this rate threshold. Added Invariant 5 and expanded [F2-GATE] note. Confirmed 10/60s default; noted ~5/60s tighter option for quiet transmission profiles. — 2026-06-10"
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

# BC-2.15.010: Unauthorized Control Command — Control-Class FC Exceeding Threshold Emits T1692.001

## Description

When a DNP3 application function code belonging to the Control class (SELECT=0x03, OPERATE=0x04,
DIRECT_OPERATE=0x05, DIRECT_OPERATE_NR=0x06) is observed on a FIR=1 fragment, the per-flow
DIRECT_OPERATE counter is incremented. When the counter exceeds `direct_operate_threshold`
within the detection window, a single `Finding` is emitted carrying `T1692.001` ("Unauthorized
Message: Command Message") and the one-shot guard is set. The finding is emitted at most once
per detection window per flow. ADR-007 Decision 5.

## Preconditions

1. The validity gate (BC-2.15.004) returned `true`.
2. `has_user_data(control)` is `true` (link FC is 0x03 or 0x04).
3. `transport_is_fir(transport_octet)` is `true` (FIR=1, BC-2.15.008).
4. `classify_dnp3_fc(app_fc)` returns `Dnp3FcClass::Control` (BC-2.15.006).
5. `flow.is_non_dnp3 == false`.
6. `self.all_findings.len() < MAX_FINDINGS` (DoS cap, see BC-2.15.022).

## Postconditions

**Counter increment (every Control-class FC on FIR=1 frame):**
1. `flow.direct_operate_count += 1`.
2. If `flow.direct_operate_count == 1` (first in window): `flow.window_start_ts = now_ts`.

**Finding emission (when threshold is exceeded AND guard is not set):**
3. When `flow.direct_operate_count > self.direct_operate_threshold`
   AND `flow.direct_operate_emitted == false`
   AND the window has NOT expired (`now_ts.wrapping_sub(flow.window_start_ts) <= DETECTION_WINDOW_SECS`):
   - One `Finding` is pushed to `self.all_findings`:
     - `category: ThreatCategory::Execution`
     - `verdict: Verdict::Likely`
     - `confidence: Confidence::Medium`
     - `summary`: `"DNP3 unauthorized control command burst: {count} control FCs in {elapsed}s window (threshold {threshold})"`
     - `evidence`: one entry — `"FC=0x{fc:02X} dest={dest:#06X} src={src:#06X}"`
     - `mitre_techniques: vec!["T1692.001"]`
     - `source_ip: Some(<source endpoint>)` — resolved from flow_key
     - `timestamp: Some(...)` — pcap-relative capture timestamp
   - `flow.direct_operate_emitted = true` (one-shot guard set).

**Window expiry / reset:**
4. When `now_ts.wrapping_sub(flow.window_start_ts) > DETECTION_WINDOW_SECS`:
   `flow.direct_operate_count = 1`, `flow.window_start_ts = now_ts`,
   `flow.direct_operate_emitted = false` — window resets; the incoming FC seeds the new window.

## Invariants

1. **One finding per window**: the `direct_operate_emitted` guard ensures at most one
   T1692.001 finding per detection window per flow. High-volume bursts do not flood `all_findings`.
2. **All Control-class FCs count**: SELECT (0x03), OPERATE (0x04), DIRECT_OPERATE (0x05), and
   DIRECT_OPERATE_NR (0x06) all increment `direct_operate_count`. This BC covers the full
   Control set from BC-2.15.006. [ADR-007 Decision 5: "SELECT/OPERATE/DIRECT_OPERATE/DIRECT_OPERATE_NR"]
3. **T1692.001 is the correct v19.1 technique** [MITRE: dnp3-research.md §6]: T1692.001
   "Unauthorized Message: Command Message" replaces revoked T0855. Do NOT emit T0855.
4. **Broadcast co-emission**: if `h.destination` is in 0xFFFD..=0xFFFF, the finding additionally
   notes the broadcast destination in the evidence field (anomaly note — see BC-2.15.018).
   The broadcast anomaly does NOT change the technique tag set; T1692.001 remains the sole tag.
5. **10/60s threshold is a flood guard, NOT the primary authz check (v1.2 clarification)**:
   [JUDGMENT: dnp3-f2-scope-threshold-validation.md §Q2 Threshold-1] The threshold catches
   a source that is issuing an abnormally high VOLUME of control commands. The high-value
   unauthorized-command signal — control FC from a non-allowlisted SOURCE address — fires at
   **count=1**, independent of this threshold. The source-address check is the primary gate;
   the 10/60s rate check is the secondary volumetric gate for the flood/burst case. This design
   matches real-world DNP3 security posture: an illegitimate source sending even ONE control
   command is anomalous; a legitimate source sending 10+/60s is also anomalous for different
   reasons (flood, misconfiguration, replay). The `--dnp3-direct-operate-threshold` flag
   (BC-2.15.017) allows operators to tighten to ~5/60s for quiet transmission profiles.

**[F2-GATE: human to confirm default]**
The default value of `direct_operate_threshold` is proposed as **10** (ten Control-class FCs
within `DETECTION_WINDOW_SECS = 60 seconds`). This threshold is CONFIRMED as sound by
dnp3-f2-scope-threshold-validation.md §Q2 Threshold-1 [JUDGMENT, grounded in vendor device
profiles and ICS literature].

**Semantic role clarification (v1.2):** the 10/60s threshold is a deliberately-lax BURST /
FLOOD GUARD for the case where a source is allowlisted but issuing an abnormally high command
rate. It is NOT the primary unauthorized-source detector.

- **Unauthorized control from an UNEXPECTED SOURCE ADDRESS** must fire at **count=1**,
  independent of this rate threshold. The source-address check (comparing `src` against an
  allowlist or expected master-address set) is the high-value gate for the classic
  unauthorized-operator scenario. The 10/60s threshold is the secondary volumetric gate.
- **10/60s default is deliberately conservative** against false positives. Mechanical reality:
  DNP3 control commands (SBO round-trips for breakers/relays) occur at most a few per minute
  in normal operations; sustained bursts above 10/min are genuinely anomalous.
- **Quiet transmission profiles:** operators in environments with very low legitimate control
  rates (substation with a single breaker, for example) may tighten to **~5/60s** using the
  `--dnp3-direct-operate-threshold` CLI flag (BC-2.15.017). The default is NOT raised above 10.
  [JUDGMENT: dnp3-f2-scope-threshold-validation.md §Q2 Threshold-1 — "do not raise above 10"]

The human should confirm whether 10/60s is appropriate for their OT environment or whether
5/60s is preferred for a quieter transmission profile.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First Control-class FC on a new flow | `direct_operate_count=1`, window seeded; no finding yet (threshold not exceeded) |
| EC-002 | `direct_operate_count` reaches threshold+1 | Finding emitted; `direct_operate_emitted = true` |
| EC-003 | Additional Control FCs after `direct_operate_emitted = true` (same window) | `direct_operate_count` incremented; NO additional finding (one-shot guard) |
| EC-004 | Window expires (elapsed > DETECTION_WINDOW_SECS) | Window reset; counter = 1 (new window seeded); `direct_operate_emitted = false` |
| EC-005 | `all_findings.len() == MAX_FINDINGS` when threshold exceeded | No finding pushed; counter still incremented |
| EC-006 | Control FC 0x06 (DIRECT_OPERATE_NR, no response) | Still counted and detected as T1692.001; this FC intentionally expects no response — but it IS an unauthorized control command if threshold exceeded |
| EC-007 | Control FC to broadcast dest 0xFFFF | Finding carries broadcast anomaly note in evidence; technique tag unchanged: T1692.001 |
| EC-008 | FC 0x03 (SELECT) without subsequent 0x04 (OPERATE) | Still counted; incomplete SBO sequence is itself anomalous but no separate finding for incompleteness in v1 |

## Canonical Test Vectors

| Scenario | Frames | Expected outcome |
|----------|--------|-----------------|
| 11 consecutive DIRECT_OPERATE (threshold=10, window=60s) | FC=0x05 ×11 on same flow within 60s | Finding emitted at frame 11: `{mitre_techniques:["T1692.001"], summary:"DNP3 unauthorized control command burst: 11 control FCs in Xs window (threshold 10)"}` |
| 10 control FCs (exactly at threshold) | FC=0x05 ×10 | No finding — threshold is `>`, not `>=`; count=10, threshold=10, 10 > 10 is false |
| 11 control FCs then 5 more (same window) | 16 total | Only ONE finding (at count=11); no additional finding for counts 12–16 |
| Control FCs in two consecutive windows | 11 in window 1, 11 in window 2 (after expiry) | Two findings: one per window |

**Byte-level test vector** (DIRECT_OPERATE, outstation 3, master 1):
```
DNP3 frame:  05 64 0E C4 03 00 01 00 [hdr-crc]  C0 81 05 [app-objects]  [data-crc]
Link header: START=0x0564, LEN=14, CTRL=0xC4 (DIR=1, PRM=1, link-FC=4=UNCONFIRMED_USER_DATA)
             DEST=0x0003, SRC=0x0001
Transport:   0xC0 (FIR=1, FIN=1, SEQ=0)
App Control: 0x81
App FC:      0x05 → DIRECT_OPERATE → Dnp3FcClass::Control
```

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-023 | Sub-property B (correctness): `classify_dnp3_fc(0x03..=0x06)` returns `Control` — precondition verified | Kani (Sub-B set membership) |
| (none) | Window/threshold/guard logic: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — detecting unauthorized control commands is the primary threat-detection objective of the DNP3/ICS analyzer; T1692.001 is the MITRE v19.1 technique for unauthorized command messages in ICS environments |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — DNP3 findings are only emitted for flows that passed port-20000 classification AND the validity gate) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-23); ADR-007 Decision 5 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | T1692.001 — Unauthorized Message: Command Message (ICS sub-technique, v19.1; tactic: IcsImpairProcessControl; replaces revoked T0855) |

## Related BCs

- BC-2.15.006 — depends on (Control-class FC classification by classify_dnp3_fc)
- BC-2.15.008 — depends on (FIR=1 gate enables App FC extraction)
- BC-2.15.015 — composes with (T0827 derived-impact may be co-emitted after sustained T1692.001 conditions)
- BC-2.15.017 — composes with (threshold CLI flag --dnp3-direct-operate-threshold controls direct_operate_threshold)
- BC-2.15.018 — composes with (broadcast destination anomaly adds note to this finding's evidence)
- BC-2.15.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `Dnp3Analyzer::on_data` — Control-class branch; `direct_operate_count`, `window_start_ts`, `direct_operate_emitted`
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.direct_operate_count: u32`, `.window_start_ts: u32`, `.direct_operate_emitted: bool`
- `src/analyzer/dnp3.rs` — `Dnp3Analyzer.direct_operate_threshold: u32`
- `src/mitre.rs` — `technique_info("T1692.001")` arm (existing; shared with Modbus)
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §8` (detection table)
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 5`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-023 — Sub-property B (verifies Control-class classification precondition)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-007 Decision 5; dnp3-research.md §5 (T1692.001 mapping: "SELECT 0x03/OPERATE 0x04/DIRECT_OPERATE 0x05/DIRECT_OPERATE_NR 0x06"); dnp3-architecture-delta.md §8 |
| **Confidence** | high — T1692.001 technique confirmed [MITRE] active in ics-attack-19.1; FC mapping confirmed [SPEC] |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow state counters and `all_findings` |
| **Deterministic** | yes — same frame sequence + threshold produces same finding |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (ADR-007 Decision 2 purity boundary) |
