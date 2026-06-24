---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-17
capability: CAP-17
lifecycle_status: active
introduced: v0.11.0-feature-enip
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
  - .factory/research/enip-mitre-ics-tagging.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/specs/verification-properties/vp-032-enip-parse-safety.md
input-hash: TBD
---

# BC-2.17.018: Malformed ENIP Frame Threshold Emits T0814 Structural Anomaly Finding

## Description

When a structural-reject path fires (invalid ENIP command in validity gate; oversized declared
frame skip where `total_frame_len > MAX_ENIP_CARRY_BYTES`; or carry overflow triggering
`is_non_enip`), the windowed counter `malformed_in_window` is incremented in parallel with the
lifetime `parse_errors` counter. When `malformed_in_window` reaches
`MALFORMED_ANOMALY_THRESHOLD` (= 3) within the correlation window, a T0814 finding is emitted
once per window. This mirrors the DNP3 pattern from BC-2.15.024 (STORY-109). A burst of
malformed ENIP frames on port 44818 may indicate a scanning or crash-injection attempt
targeting poorly-implemented EtherNet/IP stacks.

## Preconditions

1. One of the structural-reject paths has fired:
   - `is_valid_enip_frame` returned `false` (unknown command), **or**
   - oversized declared frame: ENIP header parsed OK but `total_frame_len > MAX_ENIP_CARRY_BYTES`
     (frame-skip path: `parse_errors++; malformed_in_window++; cursor advances past declared frame`), **or**
   - carry-buffer overflow (`carry.len() > MAX_ENIP_CARRY_BYTES`) set `is_non_enip=true`.
2. `flow.malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD` (= 3) after the increment.
3. Within the correlation window (proposed: 300s).
4. `flow.malformed_anomaly_emitted == false`.
5. `self.all_findings.len() < MAX_FINDINGS`.
6. `flow.is_non_enip == false` at the time of the triggering reject (is_non_enip bail fires
   before any parse and does not count as a malformed-frame event after the bail is set).

## Postconditions

**Unconditional (on every structural reject):**
1. `flow.parse_errors += 1` (lifetime, monotonic).
2. `flow.malformed_in_window += 1` (windowed counter).

**Conditional finding emission (when Preconditions 2–5 met):**
3. Exactly ONE `Finding` pushed:
   - `category: ThreatCategory::Anomaly`
   - `verdict: Verdict::Possible`
   - `confidence: Confidence::Low`
   - `summary: "EtherNet/IP structural anomaly: {count} malformed frames in {elapsed}s window (flow {src_ip}→{dest_ip}) — possible crash-probe"`
   - `evidence`: `"malformed_in_window={count}; threshold={MALFORMED_ANOMALY_THRESHOLD}"`
   - `mitre_techniques: vec!["T0814"]`
   - `source_ip: Some(...)`, `timestamp: Some(...)`
4. `flow.malformed_anomaly_emitted = true` (one-shot guard per window).

**Window-expiry reset (300s):**
5. At window expiry: `flow.malformed_in_window = 0`, `flow.malformed_anomaly_emitted = false`.
   `flow.parse_errors` is NOT reset (lifetime counter).

## Invariants

1. **Two-counter model** (mirrors BC-2.15.024):
   - `parse_errors`: LIFETIME, never reset. Used by `summarize()`.
   - `malformed_in_window`: WINDOWED, reset at 300s expiry. Used for threshold checks.
2. **T0814 is the correct v19.1 technique**: T0814 "Denial of Service" (ICS Inhibit Response
   Function TA0107) — malformed frames on port 44818 are a DoS vector targeting EtherNet/IP
   device stacks. T0814 is already seeded; no new catalog entry required.
3. **MALFORMED_ANOMALY_THRESHOLD = 3**: threshold constant. Single malformed frames can be
   packet loss; three within 300s on a flow is anomalous.
4. **Low confidence**: malformed frames may be packet corruption, capture-interface artifacts,
   or non-ENIP binary traffic mis-routed. Low confidence communicates this.
5. **is_non_enip carry-overflow**: when carry overflow sets `is_non_enip=true`, that event
   increments `parse_errors` and `malformed_in_window` exactly once (at the moment of
   overflow). Subsequent on_data calls with `is_non_enip=true` are immediate no-ops and
   do not increment either counter.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First malformed frame | parse_errors=1; malformed_in_window=1; no finding |
| EC-002 | Second malformed frame | parse_errors=2; malformed_in_window=2; no finding |
| EC-003 | Third malformed frame (threshold=3) | parse_errors=3; malformed_in_window=3; T0814 Possible/Low emitted; guard set |
| EC-004 | Fourth malformed frame (same window, guard set) | parse_errors=4; malformed_in_window=4; no additional finding |
| EC-005 | 300s window expires; 3 more malformed frames | Window reset; malformed_in_window=0; fresh accumulation to 3; new T0814 |
| EC-006 | `all_findings.len() == MAX_FINDINGS` when threshold crossed | No finding; guard NOT set |
| EC-007 | is_non_enip triggered by carry overflow (counts as one malformed event) | parse_errors++; malformed_in_window++; if threshold reached, T0814 emitted; then all subsequent on_data are no-ops (is_non_enip=true) |

## Canonical Test Vectors

| malformed_in_window | parse_errors (lifetime) | Finding emitted? |
|--------------------|------------------------|-----------------|
| 1 | 1 | No |
| 2 | 2 | No |
| 3 | 3 | Yes — T0814 Possible/Low |
| 4 (same window) | 4 | No (one-shot guard) |
| 0 (after 300s reset) | 3 (unchanged) | — |
| 3 (new window) | 6 | Yes — new T0814 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-032 | Sub-C (indirect): `is_valid_enip_frame` false triggers malformed counter | Kani Sub-C biconditional |
| (none) | Threshold, windowed counter, lifetime counter, one-shot guard: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — malformed ENIP frame detection is required for T0814 Denial of Service coverage: crash-injection attacks against EtherNet/IP stacks use structurally malformed frames to trigger parser failures; the 3/300s threshold balances detection sensitivity against false positives from packet loss |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 4 (malformed_in_window field), architecture-delta.md §4.2 (MALFORMED_ANOMALY_THRESHOLD=3) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | T0814 — Denial of Service (ICS Inhibit Response Function TA0107; active ics-attack-19.1; already seeded in src/mitre.rs) |

## Related BCs

- BC-2.17.003 — depends on (is_valid_enip_frame false is one structural-reject path)
- BC-2.17.016 — depends on (carry-overflow is the other structural-reject path)
- BC-2.17.021 — composes with (parse_errors lifetime counter reported in summarize())
- BC-2.17.022 — depends on (MAX_FINDINGS cap)

## Architecture Anchors

- `src/analyzer/enip.rs` — `EnipFlowState.parse_errors: u64` (LIFETIME, never reset)
- `src/analyzer/enip.rs` — `EnipFlowState.malformed_in_window: u64` (WINDOWED, reset 300s)
- `src/analyzer/enip.rs` — `EnipFlowState.malformed_anomaly_emitted: bool`
- `src/analyzer/enip.rs` — `const MALFORMED_ANOMALY_THRESHOLD: u64 = 3`
- `src/mitre.rs` — `technique_info("T0814")` arm (existing; shared with DNP3)
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 4`

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-032 Sub-C (indirect — is_valid_enip_frame false triggers this BC's counter)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 4 (malformed_in_window); architecture-delta.md §4.2 (MALFORMED_ANOMALY_THRESHOLD=3); BC-2.15.024 (DNP3 precedent pattern) |
| **Confidence** | high — mirrors proven DNP3 malformed-frame detection pattern |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates parse_errors, malformed_in_window, malformed_anomaly_emitted, all_findings |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell |
