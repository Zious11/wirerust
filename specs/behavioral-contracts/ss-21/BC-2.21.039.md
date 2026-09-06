---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-09-06T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-21
capability: CAP-21
lifecycle_status: active
introduced: feature-s7comm
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
  - .factory/specs/architecture/ARCH-INDEX.md
input-hash: "cf116b5"
---

# BC-2.21.039: Multi-Host Setup-Communication Sweep Emits T0846 Remote System Discovery Finding

## Description

When a single source IP's `port102_setup_targets` entry (BC-2.21.033) reaches or exceeds
`S7_SWEEP_THRESHOLD_DEFAULT` (wirerust engineering default) distinct destination IPs within the
sweep window, a `Finding` carrying `T0846` ("Remote System Discovery") is emitted. This is
deliberately a **cross-flow** detection distinct from a single-PDU signal — no individual
`SetupCommunication` frame is itself evidence of a sweep; only the accumulated distinct-target
count is. T0846 is **already seeded and already emitted** in `src/mitre.rs` (ENIP,
`MitreTactic::IcsDiscovery`) — this BC adds ONLY the S7comm emission call-site, scoped per
BC-2.21.033's disclosed narrowing (Setup-Communication-based proxy, not true TCP-SYN-sweep
detection).

## Preconditions

1. A `SetupCommunication` (`0x1F0`/`0xF0`) request from `src_ip` to `dst_ip` has just been
   recorded into `port102_setup_targets[src_ip]` per BC-2.21.033 Postcondition 3.
2. `port102_setup_targets[src_ip].len() >= S7_SWEEP_THRESHOLD_DEFAULT`.
3. `self.all_findings.len() < MAX_S7COMM_FINDINGS`.
4. `sweep_reported[src_ip] == false` (a one-shot-per-source-per-window guard, new field
   alongside `port102_setup_targets`).

## Postconditions

1. Exactly ONE `Finding` is pushed when the threshold is first crossed for `src_ip` within the
   current sweep window:
   - `category: ThreatCategory::Reconnaissance`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::Medium`
   - `summary: "S7comm Setup Communication sweep observed: {n} distinct TCP/102 destinations from a single source within {window}s (T0846)"`
   - `evidence`: one entry — `"S7comm FC 0xF0 (SetupCommunication) from src={src_ip} to {n} distinct destinations: {sample of dst_ips}"`
   - `mitre_techniques: vec!["T0846"]`
   - `source_ip: Some(src_ip)`, `timestamp: Some(...)` (timestamp of the threshold-crossing frame)
2. `sweep_reported[src_ip]` is set to `true` (one-shot guard: subsequent new destinations from
   the same source within the same window do not re-emit).
3. When `port102_setup_targets[src_ip]` resets (window elapsed, BC-2.21.033 Postcondition 3),
   `sweep_reported[src_ip]` also resets to `false` — a genuinely new sweep campaign after the
   window can re-trigger.
4. `Confidence::Medium` (not `High`): unlike ENIP's ListIdentity (a single, unambiguous
   broadcast enumeration command), a series of `SetupCommunication` requests to distinct hosts
   could also reflect legitimate multi-PLC engineering-station polling (e.g. a SCADA/HMI
   periodically connecting to many PLCs) — the source research's own characterization of this
   evidence as "Conditional (from wider pcap, not one PDU)" is honored by the reduced
   confidence relative to `PlcStop`'s `High`.

## Invariants

1. **T0846 already seeded + emitted** [MITRE: s7comm-mitre-ics-tagging.md §Already-seeded
   confirmation]: `MitreTactic::IcsDiscovery` (`TA0102`) — no `src/mitre.rs` catalog or enum
   change required.
2. **`S7_SWEEP_THRESHOLD_DEFAULT = 5`** (wirerust engineering default, no external standard —
   disclosed per the same discipline as `ARP_FLAP_WINDOW_SECS`/`SPOOF_REBIND_ESCALATION_DEFAULT`,
   BC-2.16.004 Invariant 2): chosen as a small, round number distinguishing "a handful of
   legitimate engineering-station connections" from "systematic enumeration," with no claim to
   an authoritative source.
3. **Sweep detection is per-source, not per-(source,destination) pair**: this BC fires once per
   source crossing the threshold, not once per destination — the evidence is about the
   BREADTH of one source's targeting, and the evidence string records a sample of the targeted
   destinations for forensic review rather than emitting one finding per destination.
4. **Deliberately narrower than "TCP SYN sweep"**: per BC-2.21.033's disclosed scope reduction,
   this BC only observes SUCCESSFUL classic-S7comm session establishments (a `SetupCommunication`
   PDU implies TCP handshake completion and COTP/TPKT framing success) — a raw SYN-only sweep
   against closed or filtered ports on 102 is invisible to this BC, since `S7commAnalyzer` never
   sees such traffic (no S7comm PDU is ever produced). This is a disclosed, not silent,
   limitation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Source touches 5 distinct destinations (at threshold) within the window | T0846 finding emitted on the 5th distinct destination |
| EC-002 | Source touches 4 distinct destinations only | NO T0846 — below threshold |
| EC-003 | Source touches a 6th distinct destination after the 5th (already reported) | NO additional finding (one-shot guard, Postcondition 2) |
| EC-004 | Sweep window elapses, then the same source touches 5 NEW distinct destinations in a fresh window | A NEW T0846 finding is emitted (Postcondition 3 reset) |
| EC-005 | Two different sources each independently reach the threshold | Two independent T0846 findings, one per source |
| EC-006 | `self.all_findings.len() == MAX_S7COMM_FINDINGS` when the threshold is crossed | No finding pushed; `sweep_reported[src_ip]` still set (guard applies regardless of cap outcome, mirroring ENIP's BC-2.17.010 EC-003 precedent) |

## Canonical Test Vectors

| Source behavior | Expected outcome | Category |
|---|---|---|
| 5 distinct `SetupCommunication` destinations, one source, within window | T0846 finding, Medium confidence | happy-path: threshold crossed |
| 4 distinct destinations | (no finding) | negative: below threshold |
| 5 destinations, then a 6th | 1 finding total (one-shot guard) | edge-case: guard suppresses repeat |
| 5 destinations, window resets, then 5 more | 2 findings total (one per window) | edge-case: window reset re-arms |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Threshold-crossing → exactly one T0846 finding per source per window (one-shot guard correctness): effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — CAP-21 names T0846 among the 8 reused technique IDs (ADR-014 Decision 5), scoped to the cross-flow correlation substrate BC-2.21.033 provides |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); `src/mitre.rs` (existing T0846 catalog entry, no change) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0846 — Remote System Discovery (ICS Discovery, TA0102; already seeded + emitted via ENIP; S7comm adds an emission call-site, cross-flow, scoped to Setup-Communication-based sweep evidence only) |

## Related BCs

- BC-2.21.010 — depends on (`SetupCommunication` classification)
- BC-2.21.033 — depends on (`port102_setup_targets` global state this BC's threshold check consumes)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — sweep-threshold check invoked after BC-2.21.033's `port102_setup_targets` update: `if port102_setup_targets[src].len() >= S7_SWEEP_THRESHOLD_DEFAULT && !sweep_reported[src] { /* emit T0846 */ }`
- `src/analyzer/s7comm.rs` (planned) — `const S7_SWEEP_THRESHOLD_DEFAULT: usize = 5;` (wirerust engineering default); `S7commAnalyzer.sweep_reported: HashMap<IpAddr, bool>`
- `src/mitre.rs` — `technique_info("T0846")` arm (existing; shared with ENIP)
- `.factory/research/s7comm-mitre-ics-tagging.md` §Per-technique validation table (T0846 row)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(None dedicated.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates `all_findings` and cross-flow `S7commAnalyzer` state (BC-2.21.033) |
| **Deterministic** | yes |
| **Thread safety** | `S7commAnalyzer` is single-threaded |
| **Overall classification** | effectful shell |
