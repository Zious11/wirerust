---
document_type: behavioral-contract
level: L3
version: "1.6"
status: draft
producer: product-owner
timestamp: 2026-06-12T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/decoder.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-02
capability: CAP-02
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21
  - v1.3: Correct Architecture Anchors and Invariants — strict-path None arm is line 150, lax-path None arm is line 163; narrow ranges from comment-inclusive spans to the statement lines (STORY-003 m-2) — 2026-05-22
  - v1.4: Reclassify lax-path None arm (decoder.rs:163) as structurally unreachable; reword Invariant 2, EC-003, and Architecture Anchor for :163 to reflect etherparse 0.16 analysis — 2026-05-22
  - "v1.5: F2 ARP delta revision (ADR-008 Decision 1): decode_packet return type changed from Result<ParsedPacket> to Result<DecodedFrame>. ARP frames (EtherType 0x0806, Ethernet/IPv4 format) now return Ok(DecodedFrame::Arp(...)) instead of Err('No IP layer found'). Non-Ethernet/IPv4 ARP frames return Err('Non-Ethernet/IPv4 ARP frame'). Non-IP non-ARP frames continue to return Err('No IP layer found'). Postconditions, Preconditions, Invariants, Edge Cases, Test Vectors, and VP section updated to reflect three-way postcondition. VP-008 obligation noted (fuzz harness return type update required). — 2026-06-12"
  - "v1.6: Pass-10 remediation F-D10-M03: Description (line ~41-42) incorrectly stated both strict and lax ARP arms are unreachable!. ADR-008 Decision 3 v1.6+ corrects this: strict_ip_triple NetSlice::Arp arm = compile-safety unreachable! (ARP routed out before strict_ip_triple is called); lax_ip_triple LaxNetSlice::Arp arm = explicit routing to extract_arp_frame (NOT unreachable!) — truncated ARP reaches lax_ip_triple, explicit routing to Err on bad size, no panic; VP-008/VP-024 Sub-A no-panic preserved. Description, Invariants 2-4, and Architecture Anchors updated to canonical wording. — 2026-06-12"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.009: Non-IP Non-ARP Frames Return No-IP-Layer Error; ARP Frames Return DecodedFrame::Arp

## Description

**v1.5 revision (ADR-008, F2 ARP delta):** `decode_packet` now returns `Result<DecodedFrame>`
rather than `Result<ParsedPacket>`. The ARP handling path has a three-way postcondition:
(1) Ethernet/IPv4 ARP frames (EtherType 0x0806, hw_addr_size=6, proto_addr_size=4) return
`Ok(DecodedFrame::Arp(ArpFrame { ... }))`. (2) ARP frames with non-Ethernet/IPv4 field
values return `Err("Non-Ethernet/IPv4 ARP frame")`. (3) Non-IP, non-ARP frames (e.g. LLDP,
AppleTalk, EtherType 0x9000) continue to return `Err("No IP layer found")`.

The previous behavior — ARP frames returning `Err("No IP layer found")` — is retired.
`strict_ip_triple` ARP arm: compile-safety `unreachable!` (ARP frames are routed out of the
strict `Ok(slice)` arm before `strict_ip_triple` is ever called — this arm is a compile-safety
net only, never reached at runtime). `lax_ip_triple` ARP arm: explicit routing (NOT
`unreachable!`) — a snaplen-truncated ARP frame can yield `Some(LaxNetSlice::Arp(_))` from
the lax parser and does reach `lax_ip_triple`; an `unreachable!` there would panic, violating
VP-008/VP-024 Sub-A no-panic. The lax arm is therefore handled explicitly: truncated ARP →
`Err` (no panic). (ADR-008 Decision 3, v1.6+.)

## Preconditions

1. `data` is a valid link-layer frame processed by `decode_packet(data, datalink)`.
2. `datalink` is one of the five accepted variants.
3. etherparse 0.20 `SlicedPacket::from_*` returns `Ok` with `slice.net` set to either
   `Some(NetSlice::Arp(_))`, `Some(other_net_slice)`, or `None`.

## Postconditions

**Path 1 — Ethernet/IPv4 ARP frame (new in v1.5):**
1. When `slice.net == Some(NetSlice::Arp(arp_slice))` AND `extract_arp_frame` returns
   `Some(frame)` (hw_addr_size=6, proto_addr_size=4, hw_type=Ethernet, proto_type=IPv4):
   `decode_packet` returns `Ok(DecodedFrame::Arp(frame))`. No panic. Frame is routed to
   the ARP arm in `main.rs`.

**Path 2 — Non-Ethernet/IPv4 ARP frame (new in v1.5):**
2. When `slice.net == Some(NetSlice::Arp(arp_slice))` AND `extract_arp_frame` returns
   `None` (non-standard hw/proto sizes or types):
   `decode_packet` returns `Err(anyhow!("Non-Ethernet/IPv4 ARP frame"))`. No panic.
   The caller increments `summary.skipped_packets` (or equivalent error counter).

**Path 3 — Non-IP non-ARP frame (unchanged behavior):**
3. When `slice.net == None` after a successful strict parse:
   `decode_packet` returns `Err(anyhow!("No IP layer found"))`. No panic.
   The caller increments `summary.skipped_packets`.

**General:**
4. No panic occurs in any of the three paths.
5. All three paths are exercised by separate test vectors.

## Invariants

1. **Three-way dispatch on `slice.net`**: the `Ok(slice)` match arm now handles three
   cases: `Some(NetSlice::Arp(arp))` → extract_arp_frame → Ok(DecodedFrame::Arp) or Err;
   `Some(other_net)` → strict_ip_triple → Ok(DecodedFrame::Ip); `None` → Err("No IP layer
   found"). This is a minimal, additive change to the existing two-arm dispatch.
2. **strict_ip_triple ARP arm: compile-safety unreachable!; lax_ip_triple ARP arm: explicit
   routing (NOT unreachable!)**: `strict_ip_triple` receives a compile-safety `unreachable!`
   for `NetSlice::Arp(_)` because ARP frames are routed out of `decode_packet`'s strict
   `Ok(slice)` arm before `strict_ip_triple` is ever called — this arm is never reached at
   runtime. `lax_ip_triple` MUST NOT use `unreachable!`: a snaplen-truncated ARP frame yields
   `Some(LaxNetSlice::Arp(_))` from etherparse 0.20's lax parser and DOES reach `lax_ip_triple`;
   a panic there would violate VP-008/VP-024 Sub-A no-panic. The lax arm is therefore handled
   explicitly via explicit routing to `extract_arp_frame`; truncated ARP body → `Err` without
   panic. (ADR-008 Decision 3, v1.6+.)
3. **Lax retry handles ARP via explicit routing**: the `Err(SliceError::Len(_))` lax-retry
   path is affected by ARP in etherparse 0.20. A snaplen-truncated ARP frame can enter the lax
   path and yield `Some(LaxNetSlice::Arp(_))`; `lax_ip_triple` routes this explicitly to
   `extract_arp_frame`. Complete (non-truncated) ARP frames succeed on the strict path and
   never enter the lax retry arm.
4. **lax-path None arm (decoder.rs, lax path) remains structurally unreachable** for non-ARP
   content: the analysis from v1.4 regarding the `None` arm of `lax_ip_triple` is unchanged.
   The `LaxNetSlice::Arp` arm is explicit routing (not `unreachable!`); the `None` arm for
   non-ARP/non-IP content remains structurally unreachable (etherparse 0.20 lax parser does
   not produce `None` net-slice for complete or truncated frames).
5. **VP-008 fuzz harness update required**: the cargo-fuzz harness for VP-008 must accept
   `Result<DecodedFrame>` (was `Result<ParsedPacket>`). Both `Ip` and `Arp` variants are
   non-panic outcomes. The no-panic invariant is unchanged; only the return type broadens.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Ethernet/IPv4 ARP frame (EtherType 0x0806, hw_size=6, proto_size=4) | Path 1: Ok(DecodedFrame::Arp) — ARP frame decoded and routed to ARP analyzer |
| EC-002 | Non-Ethernet ARP frame (hw_type != 0x0001, e.g. IEEE 802) | Path 2: Err("Non-Ethernet/IPv4 ARP frame") |
| EC-003 | ARP frame with hw_addr_size=8 (non-standard MAC length) | Path 2: Err("Non-Ethernet/IPv4 ARP frame") |
| EC-004 | LLDP frame (EtherType 0x88CC, non-ARP non-IP) | Path 3: Err("No IP layer found") — net==None |
| EC-005 | Ethernet frame EtherType 0x9000 (custom, non-ARP non-IP) | Path 3: Err("No IP layer found") |
| EC-006 | IPv6 content via ETHERNET with IPv6 EtherType | IP layer present; Ok(DecodedFrame::Ip) returned |
| EC-007 | Snaplen-truncated frame with no IP bytes (lax path) | Lax retry; lax-path None arm structurally unreachable (Invariant 4); Err("No IP layer found") or recovered Ip variant depending on truncation type |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 42-byte Ethernet frame: EtherType=0x0806, hw_type=0x0001, proto_type=0x0800, hlen=6, plen=4, ARP Request fields | Ok(DecodedFrame::Arp(ArpFrame { operation: 1, ... })) | happy-path: Ethernet/IPv4 ARP Request → DecodedFrame::Arp |
| 42-byte Ethernet frame: EtherType=0x0806, hw_type=0x0006, proto_type=0x0800, hlen=6, plen=4 (IEEE 802 hw type) | Err containing "Non-Ethernet/IPv4 ARP frame" | error: non-Ethernet ARP |
| Ethernet frame with EtherType 0x9000 (custom, non-IP, non-ARP) | Err containing "No IP layer found" | error: non-IP non-ARP frame (Path 3, unchanged) |
| Ethernet frame with EtherType 0x86DD (IPv6) | Ok(DecodedFrame::Ip(ParsedPacket)) | happy-path: IPv6 IP frame |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-008 | decode_packet no-panic: all three paths return Ok or Err without panicking; fuzz harness updated to accept Result<DecodedFrame> (both Ip and Arp variants are non-panic outcomes) | cargo-fuzz: arbitrary input bytes; assert no panic for any input |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-02 ("Link-type gating") per domain/capabilities/cap-02-link-type-gating.md |
| Capability Anchor Justification | CAP-02 ("Link-type gating") per domain/capabilities/cap-02-link-type-gating.md — the decoder's three-way ARP/non-Ethernet-ARP/non-IP dispatch is the link-type gating mechanism; Ethernet/IPv4 ARP frames are now gated to the ARP analysis path rather than discarded, while non-IP non-ARP frames continue to be rejected |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5); ADR-008 Decisions 1–3 |
| Stories | STORY-003 (original brownfield); STORY-111 (F2 ARP migration — etherparse 0.20 + DecodedFrame enum + BC-2.02.009 revision) |
| Origin BC | BC-DEC-009 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.007 — related to (both produce Err without panic; different conditions)
- BC-2.16.015 — composes with (BC-2.16.015 specifies the decode-vs-analysis separation architectural invariant that this BC now participates in)
- BC-2.16.001 — depends on (the Path 1 Ok(DecodedFrame::Arp) result is the input to BC-2.16.001 extraction)
- BC-2.16.009 — depends on (the Path 2 Err("Non-Ethernet/IPv4 ARP frame") result triggers D11 malformed detection)

## Architecture Anchors

- `src/decoder.rs` — `pub enum DecodedFrame { Ip(ParsedPacket), Arp(ArpFrame) }` (new in v0.7.0)
- `src/decoder.rs` — `decode_packet` ARP routing: `Some(NetSlice::Arp(arp)) => { match extract_arp_frame(arp, outer_mac, data.len()) { Some(f) => Ok(DecodedFrame::Arp(f)), None => Err(anyhow!("Non-Ethernet/IPv4 ARP frame")) } }`
- `src/decoder.rs` — `None => Err(anyhow!("No IP layer found"))` (Path 3, unchanged, strict path)
- `src/decoder.rs` — `NetSlice::Arp(_) => unreachable!(...)` in `strict_ip_triple` (compile-safety; never reached at runtime — ARP routed out before this function is called)
- `src/decoder.rs` — `LaxNetSlice::Arp(arp) => return Err(LaxNetArpSignal(arp))` in `lax_ip_triple` (explicit routing, NOT unreachable! — truncated ARP reaches lax_ip_triple; explicit routing to extract_arp_frame; Err on bad size, no panic; VP-008/VP-024 Sub-A no-panic preserved)
- ADR-008 Decision 1 (return type change), Decision 2 (extract_arp_frame None → Err), Decision 3 (unreachable! arms)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decisions 1–3 (normative — F2 ARP delta revision); arp-architecture-delta.md §2.1–§2.2 (three-way dispatch routing code) |
| **Confidence** | high — three-way postcondition is explicitly specified in ADR-008 §BC-2.02.009 revised postcondition (normative) section |
| **Extraction Date** | 2026-06-12 (v1.5 revision) |

## Evidence Types Used

- **guard clause**: match on `slice.net` returning DecodedFrame::Arp, Err("Non-Ethernet/IPv4 ARP frame"), or Err("No IP layer found") depending on ARP field values

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

v1.5: BC revised to reflect ADR-008 three-way postcondition. Previous v1.4 text describing
ARP frames as returning Err("No IP layer found") is superseded. The test
`test_decode_non_ip_frame_returns_error` (or equivalent) must be updated: the ARP subtest
should assert `Ok(DecodedFrame::Arp(...))` rather than `Err("No IP layer found")`.
A new test for Path 2 (`Err("Non-Ethernet/IPv4 ARP frame")`) should be added.
The Path 3 (non-IP non-ARP) test remains valid unchanged.
