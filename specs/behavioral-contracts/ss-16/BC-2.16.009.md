---
document_type: behavioral-contract
level: L3
version: "1.9"
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
  - "v1.5: D-078 (F5 finding O-A, human-adjudicated FIX) — Precondition 3 clarified: the 4-part type/size guard failure that triggers the D11 path occurs regardless of which decode arm (strict or lax) built the ArpPacketSlice. A lax-built slice that fails extract_arp_frame is a D11 malformed condition (same error string, same D11 routing) — not a generic decode-error. EC-008 added: lax-built-slice + extract_arp_frame None case explicitly documented as D11. The ONLY case that remains a generic decode-error (not D11) is when the lax parser cannot build an ArpPacketSlice at all (stop_err == Layer::Arp, lax.net == None). — 2026-06-15"
  - "v1.6: D-078 mechanism correction — peek-in-None-arm, not lax-built-slice. The v1.5 description of the lax-path D11 mechanism was based on an incorrect hypothesis. etherparse's ArpPacketSlice::from_slice validates len >= 8 + 2*hlen + 2*plen BEFORE building any slice — strict and lax both fail together on length. A malformed-AND-short ARP therefore never yields a LaxNetSlice::Arp to inspect. The ACTUAL mechanism (commit 9228e34): a malformed ARP that is also too short fails from_slice entirely and lands in decode_packet's lax None arm (lax.net == None, stop_err == Layer::Arp). That None arm then performs a raw fixed-header peek (8 bytes) from the raw bytes using the ARP payload offset derived from lax.link (Ethernet2 only — offset 14; other/None link → conservative truncation path). If the peeked htype (BE u16) != 0x0001 OR ptype (BE u16) != 0x0800 OR hlen (u8) != 6 OR plen (u8) != 4 → Err(\"Non-Ethernet/IPv4 ARP frame\") → record_malformed → D11. If the 8-byte fixed header is valid Ethernet/IPv4 (but the variable section is truncated or the frame is too short to even contain the 8-byte header) OR the link layer is non-Ethernet → Err(\"truncated ARP frame\") → generic decode-error. Preconditions 2–3 and EC-008 updated to describe this peek mechanism. Observable D11 outcome unchanged. — 2026-06-15"
  - "v1.7: D-078 F-1 fix — VLAN/link-extension offset correction. The v1.6 text stated the ARP payload offset was derived from lax.link (Ethernet2 only — offset 14). This was an oversimplification: VLAN-tagged (802.1Q/802.1ad) and MACsec frames carry extension headers in lax.link_exts. The actual offset is now: Ethernet2 base header length (from lax.link) PLUS the summed byte-lengths of all headers in lax.link_exts. A VLAN-tagged ARP with non-standard type/size fields is therefore read at the correct offset and classified as D11 (no false-negative). A genuinely truncated VLAN ARP with a valid 8-byte fixed header is classified as the conservative 'truncated ARP frame' path (no false-positive D11). Only genuinely non-Ethernet link layers (lax.link not Ethernet2 or None) remain on the conservative path. Precondition 2 lax-path description and EC-008 updated accordingly. Observable D11 outcome unchanged for non-VLAN frames. — 2026-06-15"
  - "v1.8: E-17 F2 spec evolution — stacked link-extension documented-limitation clause. EC-008 expanded and EC-009 added to cover QinQ and MACsec offset correctness and the MACsec documented-unverified boundary. Precondition 2 lax-path text updated to name the QinQ (+8) and MACsec Unmodified (no-SCI +8 / SCI +16) offset values explicitly. No change to observable D11 outcome. Evidence: etherparse 0.20.2 source (macsec_header_slice.rs:246-248) + upstream proptest + tests/bc_2_16_qinq_macsec_offset_tests.rs. — 2026-06-16. EC-009 citation reconciliation to canonical test file bc_2_16_e17_macsec_offset_tests.rs (E-17 F2, DF-SIBLING-SWEEP): no-SCI offset==22 now cites test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22; SCI-present offset==30 now cites test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30; both in tests/bc_2_16_e17_macsec_offset_tests.rs. Replaces previously cited non-existent names from bc_2_16_qinq_macsec_offset_tests.rs. DF-SIBLING-SWEEP: identical change applied to BC-2.16.015 v1.7. EC-008 completeness note (combined/triple-stack untested; formula generalizes) — E-17 F2 Pass-1 remediation. DF-SIBLING-SWEEP: identical change applied to BC-2.16.015. F-2 symbol-pair clarification + O-1 notation fix + O-3 version-pin note — E-17 F2 Pass remediation. DF-SIBLING-SWEEP: identical change applied to BC-2.16.015."
  - "v1.9: DF-CONSISTENCY-AUDIT (E-17 F2 adversarial finding M-2) — input-hash field set to null (BCs are not covered by the story input-hash drift-check mechanism per CLAUDE.md). No BC content change. DF-SIBLING-SWEEP: BC-2.16.015 receives identical input-hash treatment at its v1.8. — 2026-06-16"
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
input-hash: null  # BC files are not covered by the story input-hash drift-check mechanism (see CLAUDE.md)
---

# BC-2.16.009: D11 Malformed ARP — Non-Ethernet/IPv4 HW/Proto Address Sizes Emit LOW Finding

## Description

When a frame with EtherType 0x0806 (ARP) has non-Ethernet/IPv4 fixed-header type or size
fields (hw_addr_type ≠ 0x0001, proto_addr_type ≠ 0x0800, hw_addr_size ≠ 6, or
proto_addr_size ≠ 4), `decode_packet` returns `Err("Non-Ethernet/IPv4 ARP frame")` (per
BC-2.02.009 revised postcondition), triggering Detection D11. The guard is applied via
`extract_arp_frame`'s `None` return for the strict decode path, and via a raw fixed-header
peek in `decode_packet`'s lax `None` arm for frames that are also too short for etherparse
to build any `ArpPacketSlice` (D-078 mechanism — see Preconditions). The lax-path peek
derives the ARP payload offset from the Ethernet2 base header PLUS the summed lengths of any
link-extension headers (VLAN 802.1Q/802.1ad, MACsec) in `lax.link_exts`, ensuring correct
classification for VLAN-tagged ARP captures. `ArpAnalyzer` receives notification of this
malformed frame and emits a LOW/Anomaly Finding. No MITRE technique is attached to D11;
tagging with T0814 requires live DF-VALIDATION-001 validation that has not been performed
(same rationale as D3).

## Preconditions

1. An Ethernet frame with EtherType 0x0806 was received by `decode_packet` using etherparse 0.20.
2. **Strict path** (well-formed length): etherparse constructs `Ok(NetSlice::Arp(arp))` via
   `SlicedPacket::from_ethernet`. `extract_arp_frame(arp, ...)` is called on the resulting
   `ArpPacketSlice` and returns `None` because at least one of the four type/size guards fails
   (see Precondition 3). `decode_packet` returns `Err("Non-Ethernet/IPv4 ARP frame")` → D11.

   **Lax path** (malformed-AND-short, D-078 mechanism correction): etherparse's
   `ArpPacketSlice::from_slice` validates `len >= 8 + 2*hlen + 2*plen` BEFORE building any
   slice. A malformed ARP frame that is also too short therefore fails `from_slice` entirely —
   strict and lax paths both fail together. The frame lands in `decode_packet`'s lax `None` arm
   (`lax.net == None`, `stop_err == Layer::Arp`). In that `None` arm, `decode_packet` performs
   a **raw fixed-header peek** (8 bytes) from the raw packet bytes:
   - The ARP payload offset is derived from `lax.link` (Ethernet2 base header length) PLUS
     the summed byte-lengths of all link-extension headers in `lax.link_exts` (VLAN
     802.1Q/802.1ad, MACsec, etc.) via `LaxLinkExtSlice::header_len()`. Confirmed offset
     values: no-extension Ethernet2 = 14; single 802.1Q VLAN = 18 (14+4); QinQ (outer
     0x88a8 + inner 0x8100, two Vlan link_exts) = 22 (14+4+4); MACsec Unmodified/no-SCI
     = 22 (14+8, header_len() == 8: 6-byte SecTag + 2-byte next-EtherType); MACsec
     Unmodified/SCI-present = 30 (14+16, header_len() == 16: 6-byte SecTag + 8-byte SCI +
     2-byte next-EtherType). If `lax.link` is not Ethernet2 (or is `None`) the ARP payload
     base offset is unknown → conservative path (see Precondition 3d).
   - Bytes are peeked at offsets [0..2] = htype (BE u16), [2..4] = ptype (BE u16),
     [4] = hlen (u8), [5] = plen (u8) relative to the ARP payload start.
   - If all four values match Ethernet/IPv4 defaults (htype 0x0001, ptype 0x0800, hlen 6,
     plen 4) but the variable section is truncated → `Err("truncated ARP frame")` (not D11).
   - If any of the four values is non-standard → `Err("Non-Ethernet/IPv4 ARP frame")` → D11.
   - If the frame is too short to contain even the 8-byte fixed header at the derived offset →
     same as valid-header truncation → `Err("truncated ARP frame")` (not D11, conservative path).

3. The D11 condition (which routes to `Err("Non-Ethernet/IPv4 ARP frame")`) is triggered when
   at least one of the following holds:
   a. `hw_addr_type != 0x0001` (hardware type is not Ethernet)
   b. `proto_addr_type != 0x0800` (protocol type is not IPv4)
   c. `hw_addr_size != 6` (hardware address length is not 6 bytes)
   d. `hw_addr_type`, `proto_addr_type`, `hw_addr_size`, or `proto_addr_size` cannot be
      determined because the link layer is non-Ethernet (non-Ethernet2) or `lax.link == None` —
      in this case the conservative path emits `Err("truncated ARP frame")` (not D11), since
      the payload offset is unknown.

   **For the strict path:** these guards are checked via `extract_arp_frame`'s inspection of
   the `ArpPacketSlice` accessors. **For the lax path:** these guards are checked via the raw
   fixed-header peek described in Precondition 2.

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
| EC-007 | etherparse rejects the frame entirely (malformed EtherType, truncated payload where lax parser also cannot build any slice) | `lax.net == None`, `stop_err == Layer::Arp`; `decode_packet` performs raw fixed-header peek; if peeked header reveals non-Ethernet/IPv4 type/size fields → `Err("Non-Ethernet/IPv4 ARP frame")` → D11. If peek reveals valid Ethernet/IPv4 fixed header (truncated variable section) OR frame too short for the 8-byte fixed header OR non-Ethernet link → `Err("truncated ARP frame")` → generic decode-error (not D11). |
| EC-008 | Malformed-AND-short ARP capture: frame has non-standard htype/ptype/hlen/plen AND is too short for `ArpPacketSlice::from_slice` to build a slice (both strict and lax fail together on length validation); Ethernet2 link layer with or without VLAN/802.1Q/802.1ad/MACsec extension headers in lax.link_exts | **D11 malformed finding** (observable outcome unchanged from v1.5). MECHANISM (corrected D-078, offset extended v1.7 F-1 fix): the frame lands in `decode_packet`'s lax `None` arm; `decode_packet` performs a raw fixed-header peek at the offset derived from the Ethernet2 base header length PLUS the summed lengths of all headers in lax.link_exts via `LaxLinkExtSlice::header_len()` (e.g. offset 14 for untagged, 18 for single 802.1Q VLAN, 22 for QinQ, 22 for MACsec Unmodified/no-SCI, 30 for MACsec Unmodified/SCI-present) from the raw bytes; the peeked non-standard type/size values trigger `Err("Non-Ethernet/IPv4 ARP frame")` → `record_malformed` → D11 LOW/Anomaly finding. A `LaxNetSlice::Arp` slice is NEVER built for this case — `ArpPacketSlice::from_slice` fails before constructing any slice. VLAN-tagged and QinQ malformed ARP is classified correctly. For MACsec: only the Unmodified payload variant (ptype=ARP) can produce `stop_err == Layer::Arp`; Encrypted/Modified MACsec payloads never reach this arm (early `return result` in the lax driver — see EC-009). Non-Ethernet link layer (lax.link not Ethernet2 or None) → conservative `Err("truncated ARP frame")` (not D11). Added by D-078; mechanism corrected v1.6; offset extended to VLAN/link-extensions v1.7; MACsec correctness confirmed and EC-009 added v1.8. Combined/triple-stacked extensions (QinQ + MACsec, or ≥3 stacked tags) and MACsec-with-non-ARP-inner are not individually tested; the offset formula 14 + Σ link_exts.header_len() generalizes to them by construction, and the real-on-wire-traffic boundary is per EC-009 part (c). |
| EC-009 | **MACsec (802.1AE) stacked over ARP — DOCUMENTED-LIMITATION (E-17).** (a) OFFSET CORRECTNESS — Offset arithmetic is proven (source + proptest + synthetic probe); only real-world traffic existence is unverified (part c). For all **reachable** MACsec variants (Unmodified payload with inner EtherType=ARP), `LaxLinkExtSlice::header_len()` returns the exact SecTag byte count including the 8-byte SCI when present: 8 for Unmodified/no-SCI (6-byte SecTag + 2-byte next-EtherType), 16 for Unmodified/SCI-present (6-byte SecTag + 8-byte SCI + 2-byte next-EtherType). The computed arp_offset values (22 and 30 respectively for a single MACsec tag over Ethernet2) land exactly on ARP byte 0. Proof sources: etherparse 0.20.2 `macsec_header_slice.rs:246-248` (`header_len() = 6 + sci?8:0 + unmodified?2:0`) (etherparse 0.20.2 internal line numbers; re-verify on any etherparse version bump — runtime header_len()==8/16 assertions in the e17 tests guard the behavior regardless of line drift); etherparse upstream proptest asserting 8/16 for the two Unmodified cases (`macsec_header.rs:340-347`); etherparse conformance test asserting `layer_start_offset == Σ header lengths` for stacked extensions (`lax_packet_headers.rs:1371-1419`); wirerust regression test `test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22` in `tests/bc_2_16_e17_macsec_offset_tests.rs` (confirms no-SCI offset == 22); wirerust regression test `test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30` in `tests/bc_2_16_e17_macsec_offset_tests.rs` (confirms SCI-present offset == 30). Both documented offsets (22 and 30) are empirically confirmed by wirerust synthetic tests, not solely by source analysis. (b) ENCRYPTED/MODIFIED PAYLOADS — SAFE BY CONSTRUCTION: Modified, Encrypted, and EncryptedUnmodified MACsec payloads are opaque. The phenomenon manifests at two layers — these are the same safety property described at different abstraction levels, not a contradiction. At the link-extension layer: etherparse's lax driver matches `LaxMacsecPayloadSlice::Modified { incomplete, payload }` (etherparse-0.20.2 `src/link/lax_macsec_payload_slice.rs:9` — the link-extension payload variant for modified/encrypted MACsec content) and executes `return result` before the inner-ARP parse block (`lax_packet_headers.rs:364-373`). At the top-level packet result layer: the overall packet decode result becomes `LaxPayloadSlice::MacsecModified { payload, incomplete }` (etherparse-0.20.2 `src/lax_payload_slice.rs:15`, set by the lax driver at `src/lax_packet_headers.rs:368` — the top-level result for a packet whose MACsec payload is modified/encrypted). The security-guard tests `test_BC_2_16_015_macsec_no_sci_modified_opaque_payload_unreachable` and `test_BC_2_16_015_macsec_sci_present_modified_opaque_payload_unreachable` (in `tests/bc_2_16_e17_macsec_offset_tests.rs`) match on `LaxMacsecPayloadSlice::Modified` to prove that the opaque payload never reaches `Layer::Arp`. Therefore `stop_err == Layer::Arp` is **unreachable** for encrypted/modified MACsec; the offset code in the lax `None` arm never runs for these variants. This is a security property, not a gap (treating ciphertext as ARP fields would be semantically wrong). (c) DOCUMENTED-UNVERIFIED BOUNDARY: No public on-wire MACsec-over-ARP PCAP capture exists (deep web sweep: Wireshark SampleCaptures wiki, packetlife, cloudshark, GitHub fixtures — none carry Unmodified MACsec with inner ARP). The offset arithmetic is proven by etherparse source, upstream proptests, and wirerust synthetic probe tests (parts a above); what remains unverified is solely the existence and behavior of MACsec-over-ARP in real captured traffic. Additionally, MACsec decapsulation commonly occurs at the NIC before pcap capture, so MACsec-tagged frames may never appear in practice on the wire. This boundary is DOCUMENTED-UNVERIFIED: no code change is planned until a failing real-world test demonstrates a defect. |

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
