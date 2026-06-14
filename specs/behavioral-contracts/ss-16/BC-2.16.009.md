---
document_type: behavioral-contract
level: L3
version: "1.4"
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
  - "v1.2: Pass-4 remediation F-B4-M05: PC4 'increment together' contradiction resolved — reworded to clarify: when --arp active, malformed_findings increments with malformed_frames; when --arp absent, malformed_frames still increments but no finding emitted (malformed_findings unchanged), per BC-2.16.010 key 11 and ADR-008 Decision 7. — 2026-06-12"
  - "v1.3: Pass-8 remediation F-B8-L02: PC4 clarified — note added explaining that the --arp-absent clause in PC4 describes counter behavior that operates outside the --arp-active gate stated in Precondition 4; PC4's --arp-absent sub-clause is not a contradiction of PC4's position within a contract whose outer precondition requires --arp active, but rather a specification of how malformed_frames still increments even without the active gate. — 2026-06-12"
  - "v1.4: F3 story-anchor back-fill. — 2026-06-14"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md
  - .factory/specs/architecture/arp-architecture-delta.md
  - .factory/phase-f1-delta-analysis/mitre-arp-additional-detections.md
input-hash: TBD
---

# BC-2.16.009: D11 Malformed ARP — Non-Ethernet/IPv4 HW/Proto Address Sizes Emit LOW Finding

## Description

When `extract_arp_frame` returns `None` for a frame with EtherType 0x0806 (an ARP frame
recognized by etherparse), it indicates that the ARP payload has unexpected hardware or
protocol address size fields (hw_addr_size ≠ 6 or proto_addr_size ≠ 4) or a non-Ethernet
hardware type or non-IPv4 protocol type — i.e., the frame is malformed relative to
Ethernet/IPv4 ARP (Detection D11). In this case `decode_packet` returns
`Err("Non-Ethernet/IPv4 ARP frame")` (per BC-2.02.009 revised postcondition). `ArpAnalyzer`
receives notification of this malformed frame and emits a LOW/Anomaly Finding. No MITRE
technique is attached to D11; tagging with T0814 requires live DF-VALIDATION-001
validation that has not been performed (same rationale as D3).

## Preconditions

1. An Ethernet frame with EtherType 0x0806 was parsed by etherparse 0.20.
2. etherparse successfully constructed an `ArpPacketSlice` from the frame.
3. `extract_arp_frame` returned `None` because at least one of the following holds:
   a. `hw_addr_type != ArpHardwareId::ETHERNET` (hardware type is not 0x0001)
   b. `proto_addr_type != EtherType::IPV4` (protocol type is not 0x0800)
   c. `hw_addr_size != 6` (hardware address length is not 6 bytes)
   d. `proto_addr_size != 4` (protocol address length is not 4 bytes)
4. `--arp` flag is active (analysis gate per BC-2.16.011).

## Postconditions

1. `decode_packet` returns `Err("Non-Ethernet/IPv4 ARP frame")` (not `Ok`, not a panic).
2. The ARP frame is NOT passed to `ArpAnalyzer::process_arp` as a fully decoded `ArpFrame`
   (the malformed check occurs at the extraction layer, before analysis).
3. `ArpAnalyzer` receives notification of the malformed frame event (via a separate
   `process_malformed_arp` method or equivalent mechanism in `main.rs`) and emits a Finding:
   - `confidence: LOW`
   - `finding_type: Anomaly`
   - `description` indicating malformed ARP frame (non-standard HW/proto address sizes or types)
   - `mitre_techniques: []` (empty — T0814 withheld per DF-VALIDATION-001)
   - Evidence includes: raw error string ("Non-Ethernet/IPv4 ARP frame") and packet_len
4. `ArpAnalyzer.frames_analyzed` counter is NOT incremented for malformed frames.
   Malformed frames are counted in the mandatory `malformed_frames` summary key (BC-2.16.010;
   ADR-008 Decision 7). When `--arp` is active, one `malformed_findings` increment accompanies
   each `malformed_frames` increment (one finding emitted per rejected frame). When `--arp`
   is absent, `malformed_frames` still increments (input-side count) but no finding is emitted
   (`malformed_findings` unchanged), per BC-2.16.010 key 11 and ADR-008 Decision 7.
   ARP-AMB-004 RESOLVED in F2.

   **Note — PC4 scope clarification (F-B8-L02):** The `--arp-absent` sub-clause above
   describes the counter behavior that operates *outside* the `--arp active` analysis gate
   established by Precondition 4. Precondition 4 scopes this BC's happy-path (finding is
   emitted); the `--arp-absent` sub-clause is not a contradiction but a normative statement of
   what the frame-counter (`malformed_frames`) does unconditionally, independent of the analysis
   gate. The finding-emission path (PC3) and the counter-increment path (PC4's `malformed_frames`
   clause) are separable: `malformed_frames` increments unconditionally; `malformed_findings`
   increments only under the `--arp active` gate.
5. No panic occurs for any ARP frame payload content.

**Note — integration ambiguity (resolved for F3):**
6. The exact mechanism for routing "malformed ARP" notifications to `ArpAnalyzer` is an F3
   implementation decision. One pattern: `main.rs` catches `Err("Non-Ethernet/IPv4 ARP frame")`
   and calls `arp_analyzer.record_malformed(packet_len)`. Another: a separate `MalformedArp`
   variant in `DecodedFrame`. The BC specifies the observable behavior (finding emitted), not
   the implementation mechanism.

## Invariants

1. **Never panic on malformed input**: `extract_arp_frame` returns `None` gracefully for any
   non-Ethernet/IPv4 ARP field combination — no unwrap, no out-of-bounds. The panic-freedom
   guarantee is enforced by VP-024 Sub-property A (Kani harness `verify_extract_arp_frame_none_on_bad_size`
   covers all hw_addr_size/proto_addr_size values symbolically). The None-on-bad-size paths
   are also documented as EC-007 and EC-008 in BC-2.16.001 (hw_addr_size==8 and
   proto_addr_size==16 cases respectively). See BC-2.16.001 and BC-2.16.002 generally for
   the happy-path counterparts to this None path.
2. **Low confidence rationale in ICS context**: legacy ICS protocol converters and gateway
   devices sometimes repurpose ARP fields with non-standard address sizes (e.g. custom
   hardware-type fields for proprietary network stacks). LOW confidence avoids over-alerting
   on known-quirky ICS equipment (mitre-arp-additional-detections.md §3 D11 entry).
3. **No MITRE technique tag (DF-VALIDATION-001 compliance)**: same as D3. D11 does NOT carry
   a MITRE tag until T0814 (or another applicable technique) is validated live.
4. **Structural check only**: D11 detection occurs at extraction time, not analysis time. The
   check is built into `extract_arp_frame`'s `None` return path — it is not a separate
   analyzer method.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | hw_addr_type=0x0006 (IEEE 802), hw_addr_size=6, proto_addr_size=4 | `None` (hw_addr_type != Ethernet); Err("Non-Ethernet/IPv4 ARP frame"); malformed D11 finding |
| EC-002 | hw_addr_type=Ethernet, proto_addr_type=0x86DD (IPv6), proto_addr_size=16 | `None` (proto_addr_type != IPv4 AND proto_addr_size != 4); malformed D11 finding |
| EC-003 | hw_addr_size=8 (64-bit MAC address, unusual) | `None` (hw_addr_size != 6); malformed D11 finding |
| EC-004 | hw_addr_size=0 (zero-length HW address) | `None`; malformed D11 finding |
| EC-005 | proto_addr_size=0 (zero-length protocol address) | `None`; malformed D11 finding |
| EC-006 | hw_addr_type=Ethernet, proto_addr_type=IPv4, hw_addr_size=6, proto_addr_size=4 (all correct) | `Some(ArpFrame { ... })` — NOT malformed; normal path (BC-2.16.001 / BC-2.16.002) |
| EC-007 | etherparse rejects the frame entirely (malformed EtherType, truncated payload) | etherparse returns Err (not ArpPacketSlice); the frame never reaches extract_arp_frame; handled by existing decode error path (not D11) |

## Canonical Test Vectors

| ARP payload characteristics | Expected outcome |
|---|---|
| hw_type=0x0001 (Ethernet), proto=0x0800 (IPv4), hlen=6, plen=4 | `Some(ArpFrame)` — valid path |
| hw_type=0x0001 (Ethernet), proto=0x0800 (IPv4), hlen=8, plen=4 | `None` + Err("Non-Ethernet/IPv4 ARP frame") + LOW D11 finding |
| hw_type=0x0006 (IEEE 802), proto=0x0800 (IPv4), hlen=6, plen=4 | `None` + Err("Non-Ethernet/IPv4 ARP frame") + LOW D11 finding |
| hw_type=0x0001, proto=0x86DD (IPv6), hlen=6, plen=16 | `None` + Err("Non-Ethernet/IPv4 ARP frame") + LOW D11 finding |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-024 | Sub-property A (EC-007 / EC-008 path): `extract_arp_frame` returns `None` without panic when hw/proto constraints fail | Kani: Sub-A safety harness covers all hw_addr_size/proto_addr_size values (fully symbolic); `None` for non-Ethernet/IPv4 is implicitly verified (no panic on any valid ArpPacketSlice) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — D11 malformed ARP detection is a named detection in the ARP Security Analysis capability; malformed ARP frames can indicate protocol fuzzing, non-standard ICS stacks, or deliberate evasion attempts |
| L2 Domain Invariants | (none directly) |
| Architecture Module | SS-16 (src/decoder.rs `extract_arp_frame` None path, C-23); ADR-008 Decision 5 D11 |
| Stories | STORY-113 |
| Feature | arp-security-analyzer |
| MITRE Techniques | NONE — T0814 withheld per DF-VALIDATION-001 (not validated live as of 2026-06-12) |

## Related BCs

- BC-2.02.009 — depends on (the revised BC-2.02.009 postcondition specifies that `Err("Non-Ethernet/IPv4 ARP frame")` is the return value for this path)
- BC-2.16.001 — composes with (the None path is the inverse of the Some path; same extraction function)

## Architecture Anchors

- `src/decoder.rs` — `fn extract_arp_frame(...) -> Option<ArpFrame>` — returns `None` when hw/proto type or size fields fail Ethernet/IPv4 constraints
- `src/decoder.rs` — `decode_packet` routes `None` from `extract_arp_frame` to `Err(anyhow!("Non-Ethernet/IPv4 ARP frame"))`
- `src/main.rs` — catches `Err("Non-Ethernet/IPv4 ARP frame")` and notifies `ArpAnalyzer` (F3 implementation detail)
- `.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md §Decision 2` — None path maps to Err; §Decision 5 D11
- `.factory/specs/architecture/arp-architecture-delta.md §3.3 D11`

## Story Anchor

STORY-113

## VP Anchors

- VP-024 — Sub-property A (implicit: None return on non-Ethernet/IPv4 fields is part of the no-panic coverage)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 2 (None → Err mapping) and Decision 5 (D11 scope); arp-architecture-delta.md §3.3; mitre-arp-additional-detections.md §3 D11 (Zeek bad_arp analogue; LOW confidence in ICS context) |
| **Confidence** | high — structural check; None return is deterministic for any non-Ethernet/IPv4 field combination |
| **Extraction Date** | 2026-06-12 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | stateless structural check (extraction layer) |
