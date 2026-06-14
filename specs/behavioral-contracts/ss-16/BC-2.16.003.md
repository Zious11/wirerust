---
document_type: behavioral-contract
level: L3
version: "1.9"
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
  - "v1.2: Pass-7 remediation F-B7-H01/F-B7-H02: added tactic-anchor cross-reference to Invariant 3 — T0830 maps to MitreTactic::LateralMovement and T1557.002 to MitreTactic::CredentialAccess per ADR-008 Decision 6 (merge-by-name policy); the F3/STORY-114 implementer wires these in technique_info. — 2026-06-12"
  - "v1.3: Pass-8 remediation F-B8-L01: PC6 reworded to eliminate self-referential 'one-shot' phrasing and clarify GARP-per-frame emission semantics. — 2026-06-12"
  - "v1.4: Pass-9 remediation F-B9-L01: EC table ordering corrected — EC-009 (real RFC 5227 ACD probe) was inserted between EC-003 and EC-004 (non-monotonic); moved to end after EC-008. All EC content and citations unchanged. — 2026-06-12"
  - "v1.5: Pass-10 remediation F-D10-L01: verdict-triple normalization — Description and Invariant 4 used 'LOW/Inconclusive' while PC5, EC-001, and canonical vectors used 'LOW/Anomaly'; PC5 is authoritative (confidence:LOW, finding_type:Anomaly); 'Inconclusive' verdict token removed. Description updated to 'LOW/Anomaly (confidence: LOW, finding_type: Anomaly)'; Invariant 4 updated to 'confidence: LOW, finding_type: Anomaly'; Architecture Anchor updated to 'confidence=LOW, finding_type=Anomaly'. — 2026-06-12"
  - "v1.6: D-068 adjudication — benign non-conflicting GARP must NOT be attributed to T0830/T1557.002. Research (arp-garp-mitre-attribution.md; MITRE T1557.002 DET0387; T0830; arpwatch/Zeek/Suricata convention) confirms MITRE AiTM techniques apply only to the CONFLICTING case (GARP claiming an IP already bound to a different MAC), not to benign self-announcement. D2 GARP finding now emits mitre_techniques: [] (empty). T0830+T1557.002 attribution is exclusively the GARP-that-conflicts escalation path (BC-2.16.014, co-triggers D1). Updated: Description (removed AiTM attribution from benign GARP); PC5 (mitre_techniques: []); Invariant 2 (opcode agnosticism — GARP form identified; MITRE tag removed from benign path); Invariant 3 (retitled — tactic anchors apply only via BC-2.16.014); EC-001; EC-002; EC-007; all canonical test vector rows. ADR-008 Decision 5 updated in parallel by architect under same D-068. — 2026-06-14"
  - "v1.7: Pass-5 remediation F-A-1 — Architecture Anchors §3.3 line was unconditional ('MITRE T0830+T1557.002'); replaced with conditional form: confidence=LOW (base), finding_type=Anomaly; MITRE=[] for benign GARP; T0830+T1557.002 only via BC-2.16.014 escalation. No other residual unconditional benign-GARP MITRE assertions found (full file grep confirmed). — 2026-06-14"
  - "v1.8: Pass-13 clarity note (F4, pre-empt re-flag): PC7 annotated to explain that the GARP-that-conflicts escalation and its T0830/T1557.002 MITRE attribution are born in STORY-114 (co-committed with src/mitre.rs catalog seeding) — unlike D12 (BC-2.16.007), there is NO wave-42 mitre_techniques:[] intermediate window for the conflict path (STORY-113 emits benign GARP LOW only; conflict-escalation deferred to STORY-114). — 2026-06-14"
  - "v1.9: F3 story-anchor back-fill. — 2026-06-14"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md
  - .factory/specs/architecture/arp-architecture-delta.md
  - .factory/specs/verification-properties/vp-024-arp-parse-safety.md
  - .factory/phase-f1-delta-analysis/mitre-arp-research.md
  - .factory/phase-f1-delta-analysis/mitre-arp-additional-detections.md
input-hash: TBD
---

# BC-2.16.003: Gratuitous ARP Detection — sender_ip == target_ip Classified as GARP

## Description

`is_gratuitous_arp(frame: &ArpFrame) -> bool` returns `true` if and only if
`frame.sender_ip == frame.target_ip`, regardless of the ARP opcode. A Gratuitous ARP (GARP)
is defined by the IP equality condition; both the Request form (opcode=1, used in RFC 5227 ACD
announcements) and Reply form (opcode=2, the most common spoofing-assist form) satisfy the
condition. When `ArpAnalyzer::process_arp` detects a benign (non-conflicting) GARP, it emits
a Finding at severity LOW/Anomaly (confidence: LOW, finding_type: Anomaly) with
`mitre_techniques: []` (empty — no MITRE technique attribution). Benign GARP is extremely
common in legitimate traffic (link-up announcements, RFC 5227 ACD, DHCP lease acknowledgement,
HA/VRRP failover); MITRE T0830 and T1557.002 apply ONLY to the CONFLICTING case where the GARP
claims an IP already bound to a different MAC. See BC-2.16.014 for the GARP-that-conflicts
escalation path (which DOES emit T0830+T1557.002).

## Preconditions

1. `frame` is a fully-populated `ArpFrame` produced by `extract_arp_frame` (BC-2.16.001 or
   BC-2.16.002).
2. `--arp` flag is active (analysis gate per BC-2.16.011).
3. `frame.operation` may be any u16 value; the GARP check is opcode-agnostic.

## Postconditions

1. `is_gratuitous_arp(frame)` returns `true` if and only if `frame.sender_ip == frame.target_ip`
   (byte-wise equality of the 4-byte arrays).
2. `is_gratuitous_arp(frame)` returns `false` if and only if `frame.sender_ip != frame.target_ip`.
3. No other condition (opcode, MAC values, outer_src_mac, packet_len) affects the return value.
4. The function NEVER panics for any `ArpFrame` input.
5. When `is_gratuitous_arp` returns `true` AND there is NO binding conflict (benign GARP),
   `ArpAnalyzer::process_arp` emits a Finding with:
   - `confidence: LOW`
   - `finding_type: Anomaly` (or equivalent)
   - `description` indicating Gratuitous ARP
   - `mitre_techniques: []` (empty — MITRE AiTM techniques are NOT attributed to benign,
     non-conflicting GARP; T0830/T1557.002 apply only via BC-2.16.014 GARP-that-conflicts path)
6. Exactly one GARP finding is emitted per GARP frame; there is no cross-frame one-shot guard
   for GARP (unlike detections D1 and D3, which carry per-IP or per-rate deduplication guards).
   Every GARP frame observed produces its own finding, preserving a complete forensic record of
   all occurrences.
7. If the GARP frame also triggers BC-2.16.014 (GARP-that-conflicts), the GARP finding severity
   is upgraded to MEDIUM and a D1 spoof finding is additionally emitted on the same frame.
   *(Cross-story delivery note — Pass-13 v1.8): The GARP-that-conflicts escalation (severity
   upgrade to MEDIUM) and its T0830/T1557.002 MITRE attribution are born in STORY-114,
   co-committed with the `src/mitre.rs` catalog seeding of those two technique IDs. Unlike
   D12 (BC-2.16.007, which required a wave-42 mitre_techniques:[] intermediate window before
   STORY-114 back-filled the catalog), there is NO intermediate window for the conflict path
   here: STORY-113 emits benign GARP LOW only (mitre_techniques=[]); the conflict-escalation
   path (this postcondition) is deferred entirely to STORY-114. The end-state described by PC7
   is therefore correct as written and is not a pre-merge intermediate state.*

## Invariants

1. **Biconditional**: `is_gratuitous_arp(frame) == (frame.sender_ip == frame.target_ip)`. This
   is the exact property verified by VP-024 Sub-property B Kani harness.
2. **Opcode agnosticism**: the GARP condition applies to both opcode=1 (Request form,
   RFC 5227 ACD announce) and opcode=2 (Reply form). Both are classified as GARP. The GARP
   finding for either opcode emits `mitre_techniques: []` in the benign (non-conflicting) case.
   AiTM attribution (T0830/T1557.002) is gated exclusively on binding conflict; the opcode alone
   does not trigger MITRE tagging.
3. **MITRE attribution gate (D-068 — 2026-06-14)**: T0830 (Adversary-in-the-Middle, ICS,
   ATT&CK v19.1) and T1557.002 (ARP Cache Poisoning, Enterprise, ATT&CK v19.1) apply ONLY to
   the CONFLICTING GARP case (BC-2.16.014). Both are confirmed current and non-revoked in
   ATT&CK v19.1. The D2 GARP finding for a non-conflicting GARP emits `mitre_techniques: []`
   because MITRE's own DET0387 analytics (AN1091-AN1093) and IDS convention (arpwatch, Zeek,
   Suricata, Snort) universally gate AiTM attribution on IP→MAC binding inconsistency, not on
   the gratuitous mechanism itself. (arp-garp-mitre-attribution.md §§1-4 research basis.)
   **Tactic anchors (ADR-008 Decision 6 — merge-by-name policy):** When T0830/T1557.002 ARE
   emitted (via BC-2.16.014 escalation), T0830 maps to `MitreTactic::LateralMovement` and
   T1557.002 maps to `MitreTactic::CredentialAccess`; the F3/STORY-114 implementer wires these
   in `technique_info`. Normative source: ADR-008 Decision 6.
4. **LOW confidence rationale**: GARP is the default mechanism for address announcement at
   link-up, NIC boot, DHCP lease grant acknowledgement, HA/VRRP failover, and RFC 5227 ACD.
   confidence: LOW, finding_type: Anomaly is the correct classification; an analyst must
   correlate with other indicators before acting. (arp-architecture-delta.md §3.3 D2 entry.)
5. **Purity**: `is_gratuitous_arp` is a pure core function — no I/O, no global state. It is
   the VP-024 Sub-property B Kani formal-verification target.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | op=2, sender_ip==target_ip (classic gratuitous ARP Reply, no binding conflict) | `is_gratuitous_arp` → true; Finding emitted at LOW/Anomaly with mitre_techniques=[] (no AiTM attribution for benign GARP) |
| EC-002 | op=1, sender_ip==target_ip (RFC 5227 ACD announcement form, no binding conflict) | `is_gratuitous_arp` → true; same LOW/Anomaly Finding with mitre_techniques=[] — both GARP forms handled identically; opcode does not affect MITRE attribution |
| EC-003 | op=1, sender_ip==0.0.0.0, target_ip==0.0.0.0 (both IPs zero — unusual frame, NOT an RFC 5227 probe) | `is_gratuitous_arp` → true (0.0.0.0 == 0.0.0.0); Finding emitted. This is NOT a real RFC 5227 probe: genuine RFC 5227 Address Conflict Detection (ACD) probes have sender_ip=0.0.0.0 and target_ip=the address being probed (target_ip != 0). Both-zero is either a malformed frame or extremely unusual; it satisfies the sender_ip==target_ip biconditional so is_gratuitous_arp returns true. |
| EC-004 | op=2, sender_ip != target_ip (normal ARP Reply) | `is_gratuitous_arp` → false; no GARP finding emitted |
| EC-005 | op=1, sender_ip != target_ip (normal ARP Request who-has) | `is_gratuitous_arp` → false; no GARP finding emitted |
| EC-006 | GARP frame where sender_ip is already in binding table with a DIFFERENT MAC | `is_gratuitous_arp` → true AND binding conflict detected; escalated per BC-2.16.014: GARP finding → MEDIUM + D1 spoof finding also emitted |
| EC-007 | GARP frame where sender_ip is NOT in binding table (first announcement) | `is_gratuitous_arp` → true; Finding at LOW/Anomaly with mitre_techniques=[] (no prior binding to conflict with; this is a benign announcement); binding table updated with sender_ip → sender_mac |
| EC-008 | `frame.sender_ip = [0xFF,0xFF,0xFF,0xFF]`, `frame.target_ip = [0xFF,0xFF,0xFF,0xFF]` | `is_gratuitous_arp` → true (broadcast IPs equal) |
| EC-009 | Real RFC 5227 ACD probe: op=1, sender_ip=0.0.0.0, target_ip=192.0.2.1 (target_ip != 0) | `is_gratuitous_arp` → false (0.0.0.0 != 192.0.2.1); NO GARP finding emitted. A real RFC 5227 ACD probe does not satisfy the sender_ip==target_ip condition and is therefore not classified as GARP. |

## Canonical Test Vectors

| Frame (relevant fields) | Expected `is_gratuitous_arp` | Finding emitted | Category |
|---|---|---|---|
| op=2, sender_ip=192.168.1.1, target_ip=192.168.1.1, sender_mac=AA:BB:CC:DD:EE:FF (no prior binding) | true | LOW/Anomaly, mitre_techniques=[] (benign GARP — no MITRE attribution) | happy-path: benign GARP Reply |
| op=1, sender_ip=10.0.0.5, target_ip=10.0.0.5, sender_mac=11:22:33:44:55:66 (no prior binding) | true | LOW/Anomaly, mitre_techniques=[] (benign GARP — no MITRE attribution) | happy-path: benign GARP Request (ACD announce form) |
| op=2, sender_ip=192.168.1.1, target_ip=192.168.1.2, sender_mac=AA:BB:CC:DD:EE:FF | false | (none for GARP; standard reply processed for spoof/D12 checks) | negative: normal ARP Reply |
| op=1, sender_ip=10.0.0.1, target_ip=10.0.0.2 | false | (none for GARP) | negative: normal ARP Request |
| op=2, sender_ip=10.0.0.1, target_ip=10.0.0.1, sender_mac=EE:EE:EE:EE:EE:EE (binding_table[10.0.0.1].mac = 11:11:11:11:11:11) | true + binding conflict | MEDIUM/Anomaly + D1 MEDIUM Finding (BC-2.16.014); mitre_techniques=["T0830","T1557.002"] on BOTH findings (conflict path only) | escalation: GARP-that-conflicts |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-024 | Sub-property B (GARP detection totality): `is_gratuitous_arp(frame) == (frame.sender_ip == frame.target_ip)` for all symbolic `ArpFrame` inputs; never panics | Kani: symbolic ArpFrame with all fields symbolic; biconditional assertion over all 2^32 * 2^32 sender/target IP combinations |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — Gratuitous ARP detection is a named ARP security detection (D2) in the ARP Security Analysis capability; T1557.002 page explicitly names "gratuitous ARP reply" as a sub-procedure of ARP Cache Poisoning |
| L2 Domain Invariants | (none directly) |
| Architecture Module | SS-16 (src/analyzer/arp.rs `is_gratuitous_arp`, C-23 ArpAnalyzer); ADR-008 Decision 5 |
| Stories | STORY-113 |
| Feature | arp-security-analyzer |
| MITRE Techniques | None attributed to the D2 benign GARP finding (mitre_techniques=[]). T0830 and T1557.002 are attributed exclusively via BC-2.16.014 (GARP-that-conflicts escalation). See D-068 adjudication (arp-garp-mitre-attribution.md). |

## Related BCs

- BC-2.16.002 — depends on (GARP most commonly detected from Reply frames, op=2)
- BC-2.16.001 — depends on (GARP Request form, op=1)
- BC-2.16.014 — composes with (GARP-that-conflicts: GARP + binding conflict → MEDIUM escalation)
- BC-2.16.004 — related to (spoof detection also emitted when GARP conflicts with binding table)

## Architecture Anchors

- `src/analyzer/arp.rs` — `fn is_gratuitous_arp(frame: &ArpFrame) -> bool` (free pure-core function; VP-024 Sub-B target)
- `src/analyzer/arp.rs` — `impl ArpAnalyzer { fn process_arp(...) }` — calls `is_gratuitous_arp` and emits GARP finding
- `.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md §Decision 5` — D2 GARP detection scope
- `.factory/specs/architecture/arp-architecture-delta.md §3.3` — D2 confidence=LOW (base), finding_type=Anomaly; MITRE: none (mitre_techniques=[]) for benign non-conflicting GARP; T0830+T1557.002 emitted ONLY on GARP-that-conflicts escalation path (BC-2.16.014)

## Story Anchor

STORY-113

## VP Anchors

- VP-024 — ARP Frame Parse Safety and Binding-Table Invariant (Sub-property B: GARP detection totality)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 5; arp-architecture-delta.md §3.3; mitre-arp-research.md §3; mitre-arp-additional-detections.md §3; arp-garp-mitre-attribution.md (D-068 research — MITRE T1557.002 DET0387, T0830 detection guidance, arpwatch/Zeek/Suricata/Snort convention all gate AiTM attribution on binding conflict, not GARP type) |
| **Confidence** | high — GARP condition (sender_ip == target_ip) is definitional per RFC 826; MITRE AiTM attribution requires binding conflict per D-068 research 2026-06-14 |
| **Extraction Date** | 2026-06-12 (updated 2026-06-14 D-068) |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (is_gratuitous_arp is pure; ArpAnalyzer.process_arp is stateful for finding emission but is_gratuitous_arp itself is pure) |
| **Deterministic** | yes — same ArpFrame always produces same boolean |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | is_gratuitous_arp: pure core — VP-024 Kani target (Sub-B) |
