---
document_type: behavioral-contract
level: L3
version: "1.2"
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
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md
  - .factory/specs/architecture/arp-architecture-delta.md
input-hash: TBD
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

**Malformed ARP frames (non-Ethernet/IPv4):**
7. When `extract_arp_frame` returns `None` (malformed per BC-2.16.009 precondition):
   `decode_packet` returns `Err("Non-Ethernet/IPv4 ARP frame")`. This is not a `DecodedFrame::Arp`
   — the malformed frame is an error, not a successfully decoded frame.

## Invariants

1. **Decode is always performed**: `DecodedFrame::Arp` production is unconditional for
   well-formed Ethernet/IPv4 ARP. Analysis is conditional on `--arp`. This prevents the
   decode path from being dead code that breaks under etherparse 0.20's non-exhaustive
   `NetSlice` enum, regardless of user flags.
2. **ARP bypasses IP pipeline entirely**: well-formed Ethernet/IPv4 ARP frames exit
   `decode_packet` as `DecodedFrame::Arp` via the strict `Ok(slice)` arm before
   `strict_ip_triple` is ever called; snaplen-truncated ARP frames reach `decode_packet`'s
   lax `Err(SliceError::Len(_))` arm and are routed explicitly in `lax_ip_triple`. In neither
   case do ARP frames reach `StreamDispatcher`, TCP reassembly, or any `ProtocolAnalyzer`.
   The `unreachable!` arm in `strict_ip_triple` (ADR-008 Decision 3) is a compile-safety net,
   never reached at runtime. The `lax_ip_triple` ARP arm is EXPLICIT ROUTING to
   `extract_arp_frame`, NOT `unreachable!` — a snaplen-truncated ARP frame yields
   `Some(LaxNetSlice::Arp(_))` from etherparse 0.20's lax parser and DOES reach
   `lax_ip_triple`; an `unreachable!` there would be a VP-008/VP-024 Sub-A violating panic
   (see BC-2.02.009 Invariant 2, arp-architecture-delta.md §2.2).
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
| Stories | STORY-112 |
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
- `src/decoder.rs` — `LaxNetSlice::Arp(arp) => /* explicit routing to extract_arp_frame */` in `lax_ip_triple` (NOT unreachable! — snaplen-truncated ARP frames yield Some(LaxNetSlice::Arp(_)) from etherparse 0.20's lax parser and DO reach lax_ip_triple at runtime; unreachable! there would be a VP-008/VP-024 Sub-A violating panic; see arp-architecture-delta.md §2.2)
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
