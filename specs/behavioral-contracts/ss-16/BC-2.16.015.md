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
  - "v1.1: F3 story-anchor back-fill. — 2026-06-14"
  - "v1.2: F4 scoped-adversarial remediation — sibling-propagation gap from BC-2.02.009 v1.6 correction. Invariant 2 and Architecture Anchors wrongly attributed unreachable! to lax_ip_triple / LaxNetSlice::Arp arm. ADR-008 Decision 3 v1.6 and arp-architecture-delta.md §2.2 state unambiguously: strict_ip_triple NetSlice::Arp arm = compile-safety unreachable! (ARP routed out before strict_ip_triple is called); lax_ip_triple LaxNetSlice::Arp arm = explicit routing to extract_arp_frame (NOT unreachable!) — truncated ARP reaches lax_ip_triple at runtime; unreachable! there would be a VP-008/VP-024 Sub-A violating panic. EC-007 (strict_ip_triple only) was already correct and is now consistent with the fixed Invariant 2. — 2026-06-14"
  - "v1.3: F4 architect ruling supersedes v1.2 'explicit routing NOT unreachable' framing (ADR-008 Decision 3 v2.1; arp-architecture-delta.md §2.2 v1.16): the design is SYMMETRIC. decode_packet intercepts Some(LaxNetSlice::Arp(_)) in the Err(SliceError::Len(_)) arm BEFORE calling lax_ip_triple, routing it to extract_arp_frame. lax_ip_triple returns IpTriple and cannot route ARP; its LaxNetSlice::Arp(_) arm IS unreachable! (compile-safety guard, symmetric to strict_ip_triple). Invariant 2 and Architecture Anchor for lax_ip_triple updated to symmetric unreachable! framing. — 2026-06-14"
  - "v1.4: D-078 (F5 finding O-A, human-adjudicated FIX) — PC-7 split into two sub-cases: (7a) lax-built ArpPacketSlice + extract_arp_frame returns None (bad type/size) → Err(\"Non-Ethernet/IPv4 ARP frame\") → D11 malformed finding; (7b) lax parser cannot build ArpPacketSlice at all (stop_err == Layer::Arp, lax.net == None) → Err(\"truncated ARP frame\") → generic decode-error (not D11). Old PC-7 incorrectly described both lax-None sub-cases as Err(\"truncated ARP frame\"). EC-008 added to document the lax-built-slice D11 sub-case explicitly. — 2026-06-15"
  - "v1.5: D-078 mechanism correction — peek-in-None-arm, not lax-built-slice. PC-7a mechanism was based on an incorrect hypothesis: etherparse's ArpPacketSlice::from_slice validates len >= 8 + 2*hlen + 2*plen BEFORE building any slice, so strict and lax both fail together on length. A malformed-AND-short ARP never yields a LaxNetSlice::Arp to inspect. ACTUAL mechanism (commit 9228e34): such frames land in the lax None arm (lax.net == None, stop_err == Layer::Arp); decode_packet derives the ARP payload offset from lax.link (Ethernet2 → offset 14; other/None → conservative truncation path), then bounds-checked-peeks the 8-byte ARP fixed header from raw bytes; non-standard htype/ptype/hlen/plen → Err(\"Non-Ethernet/IPv4 ARP frame\") → D11; valid Ethernet/IPv4 fixed header but truncated variable section, OR too short for 8-byte peek, OR non-Ethernet link → Err(\"truncated ARP frame\") → generic decode-error. PC-7a and PC-7b rewritten; EC-008 updated. Observable D11 outcome unchanged. — 2026-06-15"
  - "v1.6: D-078 F-1 fix — VLAN/link-extension offset correction. The v1.5 text stated the ARP payload offset was derived from lax.link (Ethernet2 → offset 14) with non-Ethernet links falling to the conservative truncation path. This was an oversimplification: VLAN-tagged (802.1Q/802.1ad) and MACsec frames carry link-extension headers in lax.link_exts, which were not accounted for. The actual offset is now: base Ethernet2 header length (from lax.link) PLUS the summed byte-lengths of all link-extension headers in lax.link_exts. A VLAN-tagged ARP is therefore read at the correct offset and classified correctly (non-standard htype/ptype/hlen/plen in a VLAN ARP → D11; genuinely truncated VLAN ARP → conservative truncation path, no false-positive D11). Only genuinely non-Ethernet link layers (e.g. raw/other, where lax.link is not Ethernet2 or is None) still fall to the conservative 'truncated ARP frame' path. PC-7a, PC-7b, and EC-008 updated. Observable D11 outcome and classification logic unchanged for non-VLAN frames. — 2026-06-15"
  - "v1.7: E-17 F2 spec evolution — QinQ confirmed offset and MACsec documented-limitation clause. PC-7a updated with confirmed offset values for QinQ (+8) and MACsec Unmodified (no-SCI +8 / SCI +16). EC-008 updated with the confirmed offset table. EC-009 added mirroring BC-2.16.009 v1.8 EC-009: MACsec offset correctness proven for all reachable variants; Encrypted/Modified MACsec payloads are safe by construction (unreachable via Layer::Arp); DOCUMENTED-UNVERIFIED boundary for absence of real-traffic fixture. DF-SIBLING-SWEEP with BC-2.16.009 v1.8. — 2026-06-16. EC-009 citation reconciliation to canonical test file bc_2_16_e17_macsec_offset_tests.rs (E-17 F2, DF-SIBLING-SWEEP): no-SCI offset==22 now cites test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22; SCI-present offset==30 now cites test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30; both in tests/bc_2_16_e17_macsec_offset_tests.rs. Replaces previously cited non-existent names from bc_2_16_qinq_macsec_offset_tests.rs. DF-SIBLING-SWEEP: identical change applied to BC-2.16.009 v1.8. EC-008 completeness note (combined/triple-stack untested; formula generalizes) — E-17 F2 Pass-1 remediation. DF-SIBLING-SWEEP: identical change applied to BC-2.16.009. F-2 symbol-pair clarification + O-1 notation fix + O-3 version-pin note — E-17 F2 Pass remediation. DF-SIBLING-SWEEP: identical change applied to BC-2.16.009."
  - "v1.8: DF-CONSISTENCY-AUDIT (E-17 F2 adversarial finding M-1) — PC-7a QinQ offset-22 citation corrected: added `test_BC_2_16_015_qinq_truncated_benign_arp_no_false_positive_d11` as the test that confirms the actual offset-22 ARP byte-read; `test_BC_2_16_015_qinq_link_exts_offset_formula_pin` retained as the citation for the +8 link-exts-sum invariant only. input-hash field set to null (BCs are not covered by the story input-hash mechanism per CLAUDE.md). DF-SIBLING-SWEEP: BC-2.16.009 receives no corresponding change (it cited only the offset value without citing the pin test by name). — 2026-06-16"
  - "v1.9: E-17 F3 backlink: added STORY-116/STORY-117 to Stories traceability (BC Backlink Update Obligation; DF-SIBLING-SWEEP with BC-2.16.009 v1.10). — 2026-06-17"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md
  - .factory/specs/architecture/arp-architecture-delta.md
input-hash: null  # BC files are not covered by the story input-hash drift-check mechanism (see CLAUDE.md)
---

# BC-2.16.015: Decode-vs-Analysis Separation — DecodedFrame::Arp Always Produced; Analysis Gated on --arp

## Description

The architectural separation between decoding and analysis for ARP frames is a two-stage
invariant: (1) `decode_packet` ALWAYS returns `Ok(DecodedFrame::Arp(ArpFrame { ... }))` for
well-formed Ethernet/IPv4 ARP frames regardless of whether `--arp` is present, and (2)
`ArpAnalyzer::process_arp` is called ONLY when `--arp` is active. This mirrors the
decode-vs-analysis pattern established for IP frames (decoded regardless of which analyzer
flags are set) and ensures that the decode path is always exercised, making it observable in
tests even without `--arp`. This BC is an architectural invariant BC, not a detection BC —
it specifies the structural pipeline guarantee rather than a security finding.

## Preconditions

1. An Ethernet frame with EtherType 0x0806 is received by `decode_packet`.
2. etherparse 0.20 successfully parses the frame as having `NetSlice::Arp(ArpPacketSlice)`.
3. `extract_arp_frame` returns `Some(ArpFrame { ... })` (Ethernet/IPv4 format).

## Postconditions

**Decode stage (always, regardless of --arp flag):**
1. `decode_packet` returns `Ok(DecodedFrame::Arp(frame))`.
2. The `DecodedFrame::Arp` variant is always produced for well-formed Ethernet/IPv4 ARP frames.
3. No Err is returned from `decode_packet` for well-formed Ethernet/IPv4 ARP (this represents
   the BC-2.02.009 revision: the old behavior of returning `Err("No IP layer found")` for ARP
   is retired).

**Analysis stage (gated on --arp):**
4. When `--arp` is active: `main.rs` calls `arp_analyzer.process_arp(&frame, ts)` for every
   `DecodedFrame::Arp(frame)` in the main packet loop.
5. When `--arp` is absent: `main.rs` pattern-matches on `DecodedFrame::Arp(frame)` but
   performs no action on it (the frame is acknowledged and discarded).
6. In neither case does the `DecodedFrame::Arp` arm fall through to the IP pipeline; ARP frames
   never reach `StreamDispatcher`, the reassembler, or any `ProtocolAnalyzer`.

**Malformed ARP frames and genuine truncation (lax path disambiguation — D-078, mechanism corrected v1.5):**
7. The lax `Err(SliceError::Len(_))` arm handles two distinct sub-cases that MUST NOT be
   conflated. The routing key is the error string; `main.rs` dispatches accordingly.

   **Background (D-078 mechanism correction):** `etherparse`'s `ArpPacketSlice::from_slice`
   validates `len >= 8 + 2*hlen + 2*plen` BEFORE constructing any slice. Strict and lax paths
   both fail together on length. A malformed-AND-short ARP frame therefore NEVER yields a
   `LaxNetSlice::Arp` to inspect. Both sub-cases below start from `lax.net == None`,
   `stop_err == Layer::Arp`.

   **(7a) Non-standard fixed-header fields detected via raw peek — D11 malformed finding:**
   When `lax.net == None` and `stop_err == Layer::Arp`, `decode_packet` derives the ARP
   payload offset from `lax.link` (Ethernet2 base header length) PLUS the summed byte-lengths
   of all link-extension headers present in `lax.link_exts` via `LaxLinkExtSlice::header_len()`.
   Confirmed offset values: no-extension Ethernet2 = 14; single 802.1Q VLAN = 18 (14+4); QinQ
   (outer 0x88a8 + inner 0x8100, two Vlan link_exts) = 22 (14+4+4; the +8
   link-exts sum is confirmed by `test_BC_2_16_015_qinq_link_exts_offset_formula_pin`;
   the full offset-22 ARP byte-read is confirmed by
   `test_BC_2_16_015_qinq_truncated_benign_arp_no_false_positive_d11`); MACsec
   Unmodified/no-SCI = 22
   (14+8, `header_len()` == 8: 6-byte SecTag + 2-byte next-EtherType); MACsec
   Unmodified/SCI-present = 30 (14+16, `header_len()` == 16: 6-byte SecTag + 8-byte SCI +
   2-byte next-EtherType). It then bounds-checked-peeks the 8-byte ARP fixed header from the
   raw packet bytes at that derived offset: htype (BE u16 at [0..2]), ptype (BE u16 at [2..4]),
   hlen (u8 at [4]), plen (u8 at [5]). If any peeked value is non-standard (`htype != 0x0001`
   OR `ptype != 0x0800` OR `hlen != 6` OR `plen != 4`), `decode_packet` returns
   `Err("Non-Ethernet/IPv4 ARP frame")`. `main.rs` routes this error string to
   `arp_analyzer.record_malformed(packet_len)`, emitting a D11 LOW/Anomaly finding (per
   BC-2.16.009). This is the **same D11 malformed path** as the strict decode arm, and applies
   correctly to VLAN-tagged, QinQ, and MACsec Unmodified ARP captures. A `LaxNetSlice::Arp`
   slice was NEVER built — the detection happens via raw-byte peek in the `None` arm. For
   MACsec: only the Unmodified payload variant can produce `stop_err == Layer::Arp`; see EC-009
   for the full MACsec documented-limitation clause.

   **(7b) Valid fixed-header but truncated variable section, or non-Ethernet link — generic decode-error (NOT D11):**
   When `lax.net == None` and `stop_err == Layer::Arp` AND one of:
   - The peeked 8-byte fixed header (read at the offset derived from `lax.link` +
     `lax.link_exts`) shows standard Ethernet/IPv4 values (the frame is genuinely truncated
     at the variable section — valid header, missing payload); OR
   - The frame is too short to contain even the 8-byte fixed header at the derived offset
     (peek is not possible); OR
   - `lax.link` is not `LinkSlice::Ethernet2` (or is `None`) — ARP payload base offset is
     unknown, so the conservative path applies;
   then `decode_packet` returns `Err("truncated ARP frame")`. This is a genuine truncation
   or non-Ethernet link condition. It is NOT routed to `record_malformed` and does NOT
   produce a D11 finding. It is absorbed into the existing generic decode-error handling
   path.

   **The distinction (D-078, updated for VLAN/QinQ/MACsec link-extension frames — F-1 fix,
   confirmed E-17):** Sub-case 7a: peek reveals non-standard type/size → D11. Sub-case 7b:
   peek reveals valid Ethernet/IPv4 header (truncated body), or peek cannot be performed (too
   short / non-Ethernet link) → generic decode-error. The error string is the routing key:
   `main.rs` dispatches `"Non-Ethernet/IPv4 ARP frame"` → D11; `"truncated ARP frame"` → no
   D11. The D11/decode-error distinction is attempted for Ethernet2 link-layer captures
   (including VLAN-tagged, QinQ double-tagged, and MACsec Unmodified-wrapped frames where
   `lax.link_exts` carries the extension headers), with enough bytes for the 8-byte
   fixed-header peek at the correctly-computed offset. Only genuinely non-Ethernet link layers
   (e.g. raw/other, where `lax.link` is not `Ethernet2` or is `None`) always take the
   conservative "truncated ARP frame" path. For MACsec Encrypted/Modified frames, see EC-009:
   these never produce `stop_err == Layer::Arp` and therefore never enter sub-case 7a or 7b.

## Invariants

1. **Decode is always performed**: `DecodedFrame::Arp` production is unconditional for
   well-formed Ethernet/IPv4 ARP. Analysis is conditional on `--arp`. This prevents the
   decode path from being dead code that breaks under etherparse 0.20's non-exhaustive
   `NetSlice` enum, regardless of user flags.
2. **ARP bypasses IP pipeline entirely — both IP-triple helpers carry symmetric unreachable!
   ARP arms**: well-formed Ethernet/IPv4 ARP frames exit `decode_packet` as `DecodedFrame::Arp`
   via the strict `Ok(slice)` arm before `strict_ip_triple` is ever called; snaplen-truncated
   ARP frames are intercepted by `decode_packet`'s `Err(SliceError::Len(_))` arm before
   `lax_ip_triple` is ever called, routing them to `extract_arp_frame`. In neither case do ARP
   frames reach `StreamDispatcher`, TCP reassembly, or any `ProtocolAnalyzer`. The
   `lax_ip_triple` ARP arm IS `unreachable!` — a compile-safety guard, symmetric to
   `strict_ip_triple`'s `NetSlice::Arp(_) => unreachable!` arm, and equally provably dead.
   `decode_packet`'s `Err(SliceError::Len(_))` arm intercepts `Some(LaxNetSlice::Arp(_))`
   BEFORE calling `lax_ip_triple`, routing it to `extract_arp_frame`. `lax_ip_triple` returns
   `IpTriple` and is never called with an ARP slice at runtime. The VP-008/VP-024 Sub-A
   no-panic guarantee is provided by `decode_packet`'s interception and panic-free
   `extract_arp_frame`. (ADR-008 Decision 3 v2.1; arp-architecture-delta.md §2.2 v1.16.)
3. **BC-2.02.009 revision embodied here**: the old BC-2.02.009 postcondition (ARP frames →
   `Err("No IP layer found")`) is superseded. This BC and the revised BC-2.02.009 together
   describe the complete decoder ARP postcondition set.
4. **VP-008 obligation**: the cargo-fuzz harness for VP-008 (`decode_packet` no-panic) must
   accept `Result<DecodedFrame>` — both `Ip` and `Arp` variants are non-panic outcomes. The
   no-panic invariant is unchanged; only the return type broadens (ADR-008 §Decision 1).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Well-formed Ethernet/IPv4 ARP frame; `--arp` absent | `Ok(DecodedFrame::Arp)` from decode_packet; Arp arm in main.rs: no action |
| EC-002 | Well-formed Ethernet/IPv4 ARP frame; `--arp` present | `Ok(DecodedFrame::Arp)` from decode_packet; Arp arm: process_arp called |
| EC-003 | Non-Ethernet/IPv4 ARP frame (malformed) | `Err("Non-Ethernet/IPv4 ARP frame")` from decode_packet; not DecodedFrame::Arp |
| EC-004 | Non-ARP non-IP frame (e.g. LLDP, EtherType 0x88CC) | `Err("No IP layer found")` (unchanged path — net==None) |
| EC-005 | IPv4 frame (EtherType 0x0800) | `Ok(DecodedFrame::Ip(ParsedPacket))` (IP path, unchanged) |
| EC-006 | IPv6 frame (EtherType 0x86DD) | `Ok(DecodedFrame::Ip(ParsedPacket))` (IP path, unchanged) |
| EC-007 | NetSlice::Arp in strict_ip_triple | `unreachable!("ARP frames are routed before strict_ip_triple")` — compile-safety arm, never reached at runtime |
| EC-008 | Malformed-AND-short ARP capture: non-standard htype/ptype/hlen/plen AND frame too short for `ArpPacketSlice::from_slice` to build any slice (lax.net == None, stop_err == Layer::Arp); Ethernet2 link layer (with or without VLAN/802.1Q/802.1ad/MACsec Unmodified link-extension headers in lax.link_exts) | `Err("Non-Ethernet/IPv4 ARP frame")` — D11 malformed path (PC-7a); raw fixed-header peek at offset derived from Ethernet2 base header + summed lax.link_exts `header_len()` values detects non-standard type/size values. Confirmed offsets: 14 (untagged), 18 (single 802.1Q), 22 (QinQ, confirmed by pin test), 22 (MACsec Unmodified/no-SCI), 30 (MACsec Unmodified/SCI-present). Routes to `record_malformed` → LOW finding. A `LaxNetSlice::Arp` slice is NEVER built. NOT a "truncated ARP frame" error. VLAN-tagged and QinQ malformed ARP is classified correctly (no false-positive). For MACsec Encrypted/Modified: see EC-009 — these never produce `stop_err == Layer::Arp`. Observable D11 outcome unchanged from v1.4/D-078; mechanism corrected in v1.5; offset extended to VLAN/link-extensions in v1.6 (F-1 fix); QinQ and MACsec offset values confirmed in v1.7 (E-17). Combined/triple-stacked extensions (QinQ + MACsec, or ≥3 stacked tags) and MACsec-with-non-ARP-inner are not individually tested; the offset formula 14 + Σ link_exts.header_len() generalizes to them by construction, and the real-on-wire-traffic boundary is per EC-009 part (c). |
| EC-009 | **MACsec (802.1AE) stacked over ARP — DOCUMENTED-LIMITATION (E-17, DF-SIBLING-SWEEP with BC-2.16.009 v1.8 EC-009).** (a) OFFSET CORRECTNESS — Offset arithmetic is proven (source + proptest + synthetic probe); only real-world traffic existence is unverified (part c). `LaxLinkExtSlice::header_len()` returns the correct SecTag byte count for all reachable MACsec variants: 8 for Unmodified/no-SCI (6-byte SecTag + 2-byte next-EtherType), 16 for Unmodified/SCI-present (6-byte SecTag + 8-byte SCI + 2-byte next-EtherType). The SCI bytes ARE included (etherparse `macsec_header_slice.rs:246-248`) (etherparse 0.20.2 internal line numbers; re-verify on any etherparse version bump — runtime header_len()==8/16 assertions in the e17 tests guard the behavior regardless of line drift). Confirmed arp_offset values for a single MACsec Unmodified tag over Ethernet2: 22 (no-SCI) and 30 (SCI-present). These land exactly on ARP byte 0. Proof sources: etherparse upstream proptest (`macsec_header.rs:340-347`); conformance test asserting `layer_start_offset == Σ header lengths` for stacked extensions (`lax_packet_headers.rs:1371-1419`); wirerust regression test `test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22` in `tests/bc_2_16_e17_macsec_offset_tests.rs` (confirms no-SCI offset == 22); wirerust regression test `test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30` in `tests/bc_2_16_e17_macsec_offset_tests.rs` (confirms SCI-present offset == 30). Both documented offsets (22 and 30) are empirically confirmed by wirerust synthetic tests, not solely by source analysis. (b) ENCRYPTED/MODIFIED PAYLOADS — SAFE BY CONSTRUCTION: Only the Unmodified MACsec payload variant (inner EtherType=ARP exposed) can produce `stop_err == Layer::Arp` and enter the D11/truncation arm. Modified, Encrypted, and EncryptedUnmodified MACsec payloads are opaque. The phenomenon manifests at two layers — these are the same safety property described at different abstraction levels, not a contradiction. At the link-extension layer: etherparse's lax driver matches `LaxMacsecPayloadSlice::Modified { incomplete, payload }` (etherparse-0.20.2 `src/link/lax_macsec_payload_slice.rs:9` — the link-extension payload variant for modified/encrypted MACsec content) and executes `return result` in that arm (`lax_packet_headers.rs:364-373`) before the inner-ARP parse block. At the top-level packet result layer: the overall packet decode result becomes `LaxPayloadSlice::MacsecModified { payload, incomplete }` (etherparse-0.20.2 `src/lax_payload_slice.rs:15`, set by the lax driver at `src/lax_packet_headers.rs:368` — the top-level result for a packet whose MACsec payload is modified/encrypted). The security-guard tests `test_BC_2_16_015_macsec_no_sci_modified_opaque_payload_unreachable` and `test_BC_2_16_015_macsec_sci_present_modified_opaque_payload_unreachable` (in `tests/bc_2_16_e17_macsec_offset_tests.rs`) match on `LaxMacsecPayloadSlice::Modified` to prove that the opaque payload never reaches `Layer::Arp`. `stop_err == Layer::Arp` is unreachable for these variants. This is a security property: treating ciphertext as ARP fields would be semantically wrong. (c) DOCUMENTED-UNVERIFIED BOUNDARY: No public on-wire MACsec-over-ARP PCAP exists (deep web sweep: Wireshark SampleCaptures wiki, packetlife, cloudshark, GitHub fixtures — none carry Unmodified MACsec with inner ARP). The offset arithmetic is proven by etherparse source, upstream proptests, and wirerust synthetic probe tests (parts a above); what remains unverified is solely the existence and behavior of MACsec-over-ARP in real captured traffic. MACsec decapsulation commonly occurs at the NIC before pcap capture, so MACsec-tagged frames may not appear in practice. This boundary is DOCUMENTED-UNVERIFIED; no code change is planned until a failing real-world test demonstrates a defect. |

## Canonical Test Vectors

| Frame type | `--arp` flag | `decode_packet` result | `process_arp` called? |
|---|---|---|---|
| Ethernet/IPv4 ARP Request | absent | Ok(DecodedFrame::Arp) | NO |
| Ethernet/IPv4 ARP Reply | present | Ok(DecodedFrame::Arp) | YES |
| Non-Eth/IPv4 ARP (hw_size=8) | present | Err("Non-Ethernet/IPv4 ARP frame") | NO (not a DecodedFrame::Arp) |
| LLDP frame (non-ARP non-IP) | present | Err("No IP layer found") | NO |
| TCP/IPv4 frame | present | Ok(DecodedFrame::Ip) | NO (IP pipeline) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-008 | decode_packet no-panic: both DecodedFrame::Ip and DecodedFrame::Arp variants are non-panic outcomes; fuzz harness must be updated to accept Result<DecodedFrame> | cargo-fuzz: arbitrary input bytes; assert no panic for any input (both Ok variants and Err are valid non-panic outcomes) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — the decode-vs-analysis separation is the foundational architectural invariant enabling the ARP Security Analysis capability: decode is always exercised (correctness and fuzz coverage), analysis is opt-in (performance and noise control) |
| L2 Domain Invariants | (none directly) |
| Architecture Module | SS-16 + SS-02 (src/decoder.rs DecodedFrame enum + decode_packet; src/main.rs packet loop); ADR-008 Decisions 1–3 |
| Stories | STORY-112, STORY-116, STORY-117 |
| Feature | arp-security-analyzer |
| MITRE Techniques | (none — architectural invariant; no finding emission) |

## Related BCs

- BC-2.02.009 (REVISED) — composes with (this BC and the revised BC-2.02.009 together describe the full decoder ARP postcondition set)
- BC-2.16.011 — composes with (--arp gate is the analysis activation mechanism this BC references)
- BC-2.16.001 — depends on (ARP Request decode is the Arp variant production; this BC is the gate)
- BC-2.16.002 — depends on (ARP Reply decode; same relationship)

## Architecture Anchors

- `src/decoder.rs` — `pub enum DecodedFrame { Ip(ParsedPacket), Arp(ArpFrame) }` — the enum that embodies decode-vs-analysis separation
- `src/decoder.rs` — `decode_packet` routing: `NetSlice::Arp(arp) => { match extract_arp_frame(...) { Some(f) => Ok(DecodedFrame::Arp(f)), None => Err(...) } }`
- `src/decoder.rs` — `NetSlice::Arp(_) => unreachable!(...)` in `strict_ip_triple` ONLY (compile-safety arm; never reached at runtime — ARP frames are routed out of decode_packet's strict Ok arm before strict_ip_triple is ever called)
- `src/decoder.rs` — `LaxNetSlice::Arp(_) => unreachable!(...)` in `lax_ip_triple` (compile-safety guard, provably dead — symmetric to `strict_ip_triple`'s `NetSlice::Arp(_) => unreachable!` arm; `decode_packet`'s `Err(SliceError::Len(_))` arm intercepts `Some(LaxNetSlice::Arp(_))` BEFORE calling `lax_ip_triple`, routing it to `extract_arp_frame`; `lax_ip_triple` returns `IpTriple` and cannot route ARP; ADR-008 Decision 3 v2.1; arp-architecture-delta.md §2.2 v1.16)
- `src/main.rs` — `Ok(DecodedFrame::Ip(p)) => { /* existing IP pipeline */ }` / `Ok(DecodedFrame::Arp(a)) => { if args.arp { arp_analyzer.process_arp(&a, ts); } }`
- `.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md §Decision 1` (decode_packet return type change) and §Decision 3 (unreachable! arms)
- `.factory/specs/architecture/arp-architecture-delta.md §2.1, §2.2` — DecodedFrame enum and match-site additions

## Story Anchor

STORY-112

## VP Anchors

- VP-008 — decode_packet no-panic cargo-fuzz harness (must be updated to accept Result<DecodedFrame>; both Ip and Arp variants are non-panic outcomes per ADR-008 §Decision 1 VP-008 obligation)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 1 (return type change, main.rs pattern-match), Decision 3 (unreachable! arms), Decision 4 (ArpAnalyzer structure) |
| **Confidence** | high — this is a structural/architectural invariant, not a detection heuristic; the invariant is mechanically enforced by the Rust enum match exhaustiveness |
| **Extraction Date** | 2026-06-12 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | decode_packet: reads raw bytes (effectful shell input); decode logic is pure |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | architectural invariant spanning effectful shell (main.rs) + pure core (decoder.rs) |
