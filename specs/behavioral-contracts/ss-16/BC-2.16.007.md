---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-06-12T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-16
capability: CAP-16
lifecycle_status: active
introduced: v0.7.0-feature-arp
modified:
  - "v1.1: Pass-7 remediation F-B7-H01/F-B7-H02: added tactic-anchor cross-reference to Invariant 4 — T0830 maps to MitreTactic::LateralMovement and T1557.002 to MitreTactic::CredentialAccess per ADR-008 Decision 6 (merge-by-name policy); the F3/STORY-114 implementer wires these in technique_info. — 2026-06-12"
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

# BC-2.16.007: D12 L2/L3 Sender Mismatch — Ethernet Src MAC != ARP Sender HW Addr

## Description

When `ArpFrame.outer_src_mac` is `Some(eth_mac)` and `eth_mac != frame.sender_mac`
(byte-wise inequality), `ArpAnalyzer::process_arp` emits a Finding at MEDIUM severity
(Anomaly). This is Detection D12: the Ethernet frame's source MAC (extracted from the
outer link-layer header) disagrees with the ARP-payload sender hardware address field.
This mismatch is near-impossible in legitimate traffic and is the signal used by Snort
GID 112 SID 2/3 (confirmed from `spp_arpspoof.c` primary source review;
mitre-arp-additional-detections.md §4a). D12 is stateless (single-packet) and requires
no binding table. MITRE techniques T0830 and T1557.002 are attached.

## Preconditions

1. `frame` is a fully-populated `ArpFrame` produced by `extract_arp_frame`.
2. `frame.outer_src_mac` is `Some(eth_mac)` — the Ethernet frame source MAC was captured.
   (If `outer_src_mac` is `None`, the check is skipped — no SLL false positives.)
3. `eth_mac != frame.sender_mac` (byte-wise: the 6-byte arrays differ).
4. `--arp` flag is active (analysis gate per BC-2.16.011).

## Postconditions

1. A Finding is emitted with:
   - `confidence: MEDIUM`
   - `finding_type: Anomaly`
   - `description` indicating Ethernet/ARP sender MAC mismatch (L2/L3 sender discrepancy)
   - `mitre_techniques: ["T0830", "T1557.002"]`
   - Evidence includes: `eth_mac` (Ethernet source MAC), `arp_sender_mac` (ARP sender HW addr),
     `sender_ip` (ARP sender protocol address)
2. No state is updated in `ArpAnalyzer` as a result of this detection alone (D12 is stateless).
3. D12 and other detections (D1, D2) are independent: a single frame can trigger D12
   simultaneously with D1 (spoof) or D2 (GARP) if the conditions for both are met.

**When outer_src_mac is None (Postcondition 4):**
4. When `frame.outer_src_mac == None`, no D12 finding is emitted. The check is silently
   skipped. This handles non-Ethernet captures (SLL link type) where the outer MAC is not
   meaningful.

**When outer_src_mac matches sender_mac (Postcondition 5):**
5. When `frame.outer_src_mac == Some(mac)` and `mac == frame.sender_mac`, no D12 finding is
   emitted. This is the normal case (Ethernet source MAC == ARP sender hardware address).

## Invariants

1. **Stateless detection**: D12 inspects only the current frame. It does not consult the
   binding table, does not update any counter, and does not track per-IP or per-MAC state.
2. **Near-zero FP rate**: legitimate Ethernet devices set both the Ethernet source MAC and
   the ARP sender HW addr to the same value. The only benign exceptions involve certain
   bridges/virtual switches that rewrite Ethernet headers (mitre-arp-additional-detections.md
   §4a arpwatch "ethernet mismatch" analogue). MEDIUM confidence is appropriate.
3. **Snort precedent**: this is precisely the signal detected by Snort GID 112 SID 2
   ("Ethernet/ARP mismatch request") and SID 3 ("Ethernet/ARP mismatch reply") from
   `spp_arpspoof.c`. wirerust's D12 is an equivalent stateless check (source cited in
   mitre-arp-additional-detections.md §4a).
4. **MITRE tagging**: T0830 (ICS AiTM) and T1557.002 (Enterprise ARP Cache Poisoning) are
   appropriate because a sender-MAC mismatch is a technique used in ARP-based AiTM attacks
   (the attacker forges the ARP payload sender-MAC to differ from the Ethernet header MAC,
   or vice versa, to confuse monitoring). Confirmed via mitre-arp-research.md §4 and
   mitre-arp-additional-detections.md §3 D12 entry.
   **Tactic anchors (ADR-008 Decision 6 — merge-by-name policy):** T0830 maps to
   `MitreTactic::LateralMovement` and T1557.002 maps to `MitreTactic::CredentialAccess`; the
   F3/STORY-114 implementer wires these in `technique_info`. Normative source: ADR-008 Decision 6.
5. **Not applicable to non-Ethernet captures**: `outer_src_mac = None` when the capture
   uses SLL (Linux cooked captures), where no Ethernet header is present. Skipping D12 in
   this case avoids undefined-behavior false positives.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | outer_src_mac=Some(AA:AA:AA:AA:AA:AA), sender_mac=AA:AA:AA:AA:AA:AA (match) | No D12 finding — MACs agree |
| EC-002 | outer_src_mac=Some(AA:AA:AA:AA:AA:AA), sender_mac=BB:BB:BB:BB:BB:BB (mismatch) | MEDIUM Finding emitted; T0830+T1557.002 |
| EC-003 | outer_src_mac=None (SLL capture) | No D12 finding — check skipped |
| EC-004 | Frame also triggers D2 (GARP): sender_ip==target_ip AND outer!=sender | Both D12 MEDIUM and D2 LOW findings emitted on same frame |
| EC-005 | Frame also triggers D1 (spoof via binding conflict) AND outer!=sender | D12 MEDIUM + D1 MEDIUM (or HIGH) findings emitted on same frame |
| EC-006 | outer_src_mac=Some([0x00;6]) (all-zero Ethernet src), sender_mac=[0x11;6] | MEDIUM D12 Finding — zero Ethernet src is a real mismatch indicator |
| EC-007 | Broadcast outer_src_mac=Some([0xFF;6]), sender_mac=[0xAA;6] | MEDIUM D12 Finding — broadcast Ethernet src with unicast ARP sender is a mismatch |

## Canonical Test Vectors

| `outer_src_mac` | `sender_mac` | `sender_ip` | Expected outcome |
|---|---|---|---|
| Some([0x11,0x22,0x33,0x44,0x55,0x66]) | [0x11,0x22,0x33,0x44,0x55,0x66] | 192.168.1.1 | No finding (MACs match) |
| Some([0x11,0x22,0x33,0x44,0x55,0x66]) | [0xAA,0xBB,0xCC,0xDD,0xEE,0xFF] | 192.168.1.1 | MEDIUM/Anomaly Finding; mitre_techniques=["T0830","T1557.002"]; evidence: eth_mac=11:22:33:44:55:66, arp_mac=AA:BB:CC:DD:EE:FF, ip=192.168.1.1 |
| None | [0xAA,0xBB,0xCC,0xDD,0xEE,0xFF] | 10.0.0.1 | No finding (outer_src_mac absent; SLL capture) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (VP-024 not applicable — D12 is stateless, not a pure-core function formal target) | D12 is verified by unit tests: construct ArpFrame with mismatched outer_src_mac/sender_mac; assert Finding emitted; construct matching MACs; assert no Finding; construct None outer_src_mac; assert no Finding | unit tests in src/analyzer/arp.rs tests module |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — D12 L2/L3 sender mismatch is a named detection in the ARP Security Analysis capability; it is the single highest-fidelity stateless ARP security signal (Snort GID 112 SID 2/3 equivalent), making it essential to the capability's detection coverage |
| L2 Domain Invariants | (none directly) |
| Architecture Module | SS-16 (src/analyzer/arp.rs ArpAnalyzer::process_arp, C-23); ADR-008 Decision 5 D12 |
| Stories | TBD (F3 story decomposition) |
| Feature | arp-security-analyzer |
| MITRE Techniques | T0830 (Adversary-in-the-Middle, ICS, ATT&CK v19.1 — current); T1557.002 (ARP Cache Poisoning, Enterprise, ATT&CK v19.1 — current) |

## Related BCs

- BC-2.16.001 — depends on (outer_src_mac is populated by extract_arp_frame from ArpFrame.outer_src_mac)
- BC-2.16.002 — depends on (same — Replies are the primary D12 vector)
- BC-2.16.011 — depends on (--arp gate must be active)

## Architecture Anchors

- `src/analyzer/arp.rs` — `impl ArpAnalyzer { fn process_arp(...) }` — D12 check: `if let Some(eth_mac) = frame.outer_src_mac { if eth_mac != frame.sender_mac { emit Finding } }`
- `src/decoder.rs` — `ArpFrame.outer_src_mac: Option<[u8; 6]>` — outer Ethernet src MAC, populated from `slice.link` in extract_arp_frame
- `.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md §Decision 5` — D12 requires outer Ethernet MAC; extract_arp_frame signature extension
- `.factory/specs/architecture/arp-architecture-delta.md §3.3 D12`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- (unit tests only — D12 is not part of VP-024's formal scope per VP-024 §Source Contract note)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 5; arp-architecture-delta.md §3.3; mitre-arp-additional-detections.md §4a (Snort GID 112 SID 2/3 as primary source for D12 signal) |
| **Confidence** | high — D12 check is trivially implementable (two MAC byte-array comparison) with confirmed Snort precedent; technique IDs verified live 2026-06-12 |
| **Extraction Date** | 2026-06-12 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (D12 is stateless) |
| **Deterministic** | yes — same outer_src_mac and sender_mac always produces same result |
| **Thread safety** | single-threaded |
| **Overall classification** | stateless inline check within process_arp |
