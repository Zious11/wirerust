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
  - "v1.3: Pass-5 remediation F-B5-M02: PC2 citation corrected from 'Postcondition 1.b' (first_rebind_ts setter) to 'Postcondition 1.c (Step 3 — escalation evaluation)'. F-B5-M03: PC4 citation corrected from 'BC-2.16.004 Postcondition 3' (first_rebind_ts semantics) to 'BC-2.16.004 Postcondition 1 (Step 1 / 1.a — rebind_count increment)'. — 2026-06-12"
  - "v1.4: Pass-7 remediation F-B7-H01/F-B7-H02: added tactic-anchor cross-reference to Invariant 4 — T0830 maps to MitreTactic::LateralMovement and T1557.002 to MitreTactic::CredentialAccess per ADR-008 Decision 6 (merge-by-name policy); the F3/STORY-114 implementer wires these in technique_info. — 2026-06-12"
  - "v1.5: Pass-11 remediation F-B11-L01: Source Evidence 'LOW/Inconclusive' retired verdict token corrected to 'LOW/Anomaly (confidence: LOW, finding_type: Anomaly)' per BC-2.16.003 v1.5 normalization. — 2026-06-12"
  - "v1.6: F3 story-anchor back-fill. — 2026-06-14"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md
  - .factory/specs/architecture/arp-architecture-delta.md
  - .factory/phase-f1-delta-analysis/mitre-arp-research.md
  - .factory/phase-f1-delta-analysis/mitre-arp-additional-detections.md
input-hash: TBD
---

# BC-2.16.014: GARP-That-Conflicts Upgrades to MEDIUM and Triggers D1 Spoof Finding

## Description

When a frame is simultaneously a Gratuitous ARP (sender_ip == target_ip, per BC-2.16.003)
AND the sender_ip is already in the binding table with a different sender_mac (a binding
conflict, per BC-2.16.004), two coordinated actions occur: (1) the GARP finding severity is
upgraded from LOW to MEDIUM, and (2) a separate D1 ARP spoof Finding is ALSO emitted for the
same frame. The two findings may be emitted on the same frame for the same IP. This escalation
rule is specified in arp-architecture-delta.md §3.3 ("GARP escalation rule"). MITRE techniques
T0830 and T1557.002 are attached to both findings.

## Preconditions

1. `frame.sender_ip == frame.target_ip` (GARP condition, per BC-2.16.003 Postcondition 1).
2. `bindings[frame.sender_ip].mac != frame.sender_mac` (binding conflict — sender_ip is
   already tracked with a different MAC).
3. `--arp` flag is active.

## Postconditions

1. A GARP Finding is emitted at `confidence: MEDIUM` (upgraded from LOW).
   - `finding_type: Anomaly`
   - `description` indicating Gratuitous ARP AND binding conflict
   - `mitre_techniques: ["T0830", "T1557.002"]`
2. A separate D1 ARP Spoof Finding is ALSO emitted on the same frame. Severity is
   determined by BC-2.16.004 Postcondition 1.c (Step 3 — escalation evaluation, evaluated
   after Steps 1 and 2: rebind_count incremented and first_rebind_ts set if unset). All
   three conditions per BC-2.16.004 Postcondition 1.c:
   - `confidence: HIGH` iff `rebind_count >= spoof_threshold AND
     (timestamp_secs - first_rebind_ts <= ARP_FLAP_WINDOW_SECS) AND !spoof_high_emitted`
   - `confidence: MEDIUM` otherwise
   (i.e., with `--arp-spoof-threshold 1`, this first GARP-that-conflicts emits D1 HIGH
   because all three conditions are satisfied: rebind_count=1 >= threshold=1, elapsed=0 <=
   window, and spoof_high_emitted=false)
   - `finding_type: Anomaly`
   - `description` indicating IP→MAC rebind / potential ARP cache poisoning
   - `mitre_techniques: ["T0830", "T1557.002"]`
3. The binding table is updated: `bindings[sender_ip].mac = frame.sender_mac` (last-write-wins
   per BC-2.16.005).
4. `rebind_count` is incremented per BC-2.16.004 Postcondition 1 (Step 1 / 1.a —
   rebind_count increment).
5. Two Findings are produced by `process_arp` for a single frame satisfying this BC's
   preconditions: one GARP Finding (MEDIUM) and one D1 Spoof Finding.

**When GARP is detected but NO binding conflict exists (Postcondition 6):**
6. A GARP Finding is emitted at `confidence: LOW` (standard GARP, per BC-2.16.003).
   No D1 spoof finding. Binding is created or updated normally.

## Invariants

1. **GARP finding escalation condition**: the GARP severity upgrade from LOW to MEDIUM
   requires the binding conflict condition (precondition 2). A GARP without a binding conflict
   always produces LOW per BC-2.16.003.
2. **Two findings, not one**: the GARP-that-conflicts produces two distinct Findings in the
   output, not a single merged Finding. The reporting pipeline handles multiple Findings per
   frame normally (already the case for `Vec<Finding>` return from `process_arp`).
3. **D1 escalation applies independently**: the D1 finding emitted via this path participates
   in the rebind_count escalation logic of BC-2.16.004. If this is the 3rd rebind within 60s,
   the D1 Finding will be HIGH (not MEDIUM), while the GARP Finding remains MEDIUM.
4. **MITRE tagging**: both findings carry T0830 and T1557.002 — the GARP-that-conflicts is
   the most unambiguous ARP-based AiTM signal, warranting both tags on both findings.
   **Tactic anchors (ADR-008 Decision 6 — merge-by-name policy):** T0830 maps to
   `MitreTactic::LateralMovement` and T1557.002 maps to `MitreTactic::CredentialAccess`; the
   F3/STORY-114 implementer wires these in `technique_info`. Normative source: ADR-008 Decision 6.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | GARP Reply (op=2), sender_ip=10.0.0.1, sender_mac=ATTACKER, binding exists with VICTIM_MAC | GARP Finding MEDIUM + D1 MEDIUM Finding; binding updated to ATTACKER |
| EC-002 | GARP Request (op=1), sender_ip==target_ip, MAC conflict in binding table | Same escalation — GARP form does not matter; GARP Finding MEDIUM + D1 Finding |
| EC-003 | GARP, sender_ip NOT in binding table | GARP Finding LOW only; binding initialized; no D1 finding |
| EC-004 | GARP-that-conflicts AND this is the 3rd rebind within 60s | GARP Finding MEDIUM + D1 HIGH finding (rebind_count threshold reached) |
| EC-005 | GARP-that-conflicts AND spoof_high_emitted=true (one-shot guard active) | GARP Finding MEDIUM + D1 MEDIUM finding (HIGH guard prevents second HIGH) |
| EC-006 | Non-GARP frame with binding conflict (normal spoof) | D1 MEDIUM finding only; no GARP finding |

## Canonical Test Vectors

| Binding table | Frame | Expected findings |
|---|---|---|
| Empty | op=2, sender_ip=10.0.0.1==target_ip, sender_mac=AA:AA | GARP LOW (no conflict) |
| {10.0.0.1 → BB:BB, rebind=0} | op=2, sender_ip=10.0.0.1==target_ip, sender_mac=AA:AA | GARP MEDIUM + D1 MEDIUM; T0830+T1557.002 on both |
| {10.0.0.1 → BB:BB, rebind=2, first_rebind_ts=5} | op=2, sender_ip=10.0.0.1==target_ip, sender_mac=AA:AA, ts=30 (within 60s) | GARP MEDIUM + D1 HIGH (rebind_count→3 >= threshold=3); T0830+T1557.002 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none — escalation logic tested by unit tests) | GARP LOW when no conflict; GARP MEDIUM + D1 when conflict; both findings carry T0830+T1557.002 | unit tests covering EC-001 through EC-006 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — the GARP-that-conflicts escalation rule produces the highest-fidelity ARP AiTM signal: a gratuitous ARP that simultaneously conflicts with an existing binding is the canonical ARP cache poisoning attack procedure (T1557.002 page describes "gratuitous ARP reply" as the ARP Cache Poisoning procedure) |
| L2 Domain Invariants | (none directly) |
| Architecture Module | SS-16 (src/analyzer/arp.rs ArpAnalyzer::process_arp escalation logic, C-23); ADR-008 Decision 5 D2/D1 interaction; arp-architecture-delta.md §3.3 GARP escalation rule |
| Stories | STORY-114 |
| Feature | arp-security-analyzer |
| MITRE Techniques | T0830 (Adversary-in-the-Middle, ICS, ATT&CK v19.1 — current); T1557.002 (ARP Cache Poisoning, Enterprise, ATT&CK v19.1 — current) |

## Related BCs

- BC-2.16.003 — composes with (GARP detection is a precondition; this BC defines the escalation case)
- BC-2.16.004 — composes with (D1 spoof detection; escalation uses same rebind_count logic)
- BC-2.16.005 — depends on (binding table update occurs on this path)

## Architecture Anchors

- `src/analyzer/arp.rs` — `impl ArpAnalyzer { fn process_arp(...) }` — escalation rule: `if is_gratuitous_arp(frame) && binding_conflict { garp_severity = MEDIUM; emit_d1_finding(); }`
- `.factory/specs/architecture/arp-architecture-delta.md §3.3` — "GARP escalation rule" paragraph
- `.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md §Decision 5` — D2 and D1 interaction note

## Story Anchor

STORY-114

## VP Anchors

- (none — escalation logic not a formal verification target; covered by unit tests)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 5 (D2 GARP confidence LOW/Anomaly (confidence: LOW, finding_type: Anomaly); interaction with D1 spoof); arp-architecture-delta.md §3.3 (GARP escalation rule verbatim: "if D2 (GARP) also triggers D1 (binding conflict — the GARP claims an IP already bound to a different MAC), the GARP finding is upgraded to MEDIUM and the D1 spoof finding is also emitted") |
| **Confidence** | high — escalation rule is explicitly specified in architecture delta; MITRE tags confirmed live 2026-06-12 |
| **Extraction Date** | 2026-06-12 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | stateful pure core (reads binding table + GARP condition; emits two Findings) |
