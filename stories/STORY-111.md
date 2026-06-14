---
document_type: story
story_id: STORY-111
epic_id: E-16
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-06-13T00:00:00Z
phase: f3
points: 5
priority: P0
depends_on: [STORY-110]
blocks: [STORY-112]
behavioral_contracts:
  - BC-2.02.009
verification_properties: [VP-008]
tdd_mode: strict
target_module: decoder
subsystems: [SS-02]
estimated_days: 2
feature_id: issue-009-arp-security-analyzer
github_issue: 9
# BC status: BC-2.02.009 v1.6 — revised in F2 for three-way decode postcondition; ARP authors 2026-06-12
# VP-008 carry-forward: fuzz harness return-type MUST change decode_packet -> DecodedFrame before STORY-112 dispatches
inputs:
  - .factory/specs/architecture/arp-architecture-delta.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.009.md
  - .factory/specs/verification-properties/vp-008-decode-packet-no-panic.md
input-hash: "d5bda72"
---

# STORY-111: etherparse 0.20 Migration + DecodedFrame/ArpFrame Types + BC-2.02.009 Revision

## Narrative

- **As a** wirerust maintainer
- **I want** to upgrade etherparse from 0.16 to 0.20, introduce the `DecodedFrame` enum and `ArpFrame` struct in `src/decoder.rs`, add the `strict_ip_triple` compile-safety arm and the `lax_ip_triple` explicit-routing arm for ARP, and revise BC-2.02.009's postcondition so that Ethernet/IPv4 ARP frames return `Ok(DecodedFrame::Arp(...))` rather than an error
- **So that** the decode pipeline is prepared to route ARP frames to the new ArpAnalyzer without breaking any existing IP-pipeline tests, and the no-panic VP-008 guarantee is maintained across the new return type

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.02.009 | Non-IP Non-ARP Frames Return No-IP-Layer Error; ARP Frames Return DecodedFrame::Arp |

## VP-008 Carry-Forward Obligation (F3-OBL-STORY111-VP008)

The cargo-fuzz harness for VP-008 (`decode_packet` no-panic) currently expects `Result<ParsedPacket>`. After this story's changes `decode_packet` returns `Result<DecodedFrame>`. The fuzz harness MUST be updated to:

1. Accept `Result<DecodedFrame>` as the return type.
2. Handle both `Ok(DecodedFrame::Ip(_))` and `Ok(DecodedFrame::Arp(_))` as non-panic outcomes.
3. Remove any match-exhaustiveness arm that assumed `SliceError::Len` is the only lax path (two non-exhaustive `NetSlice`/`LaxNetSlice` match breaks will appear and MUST be resolved with the new arms added in this story).
4. Keep the no-panic invariant assertion unchanged (any `Ok` or `Err` variant is acceptable; only a panic fails the harness).

This obligation is carried from BC-2.02.009 Invariant 5 and arp-architecture-delta.md §4.3.

## Acceptance Criteria

### AC-001 (traces to BC-2.02.009 postcondition 1 — Path 1: Ethernet/IPv4 ARP frame)
`decode_packet(data, datalink)` returns `Ok(DecodedFrame::Arp(frame))` when `data` is a
well-formed 42-byte Ethernet/IPv4 ARP frame (EtherType 0x0806, hw_type=0x0001,
proto_type=0x0800, hlen=6, plen=4). The `ArpFrame` fields match the frame content exactly.
No panic occurs.
- **Test:** `test_decode_eth_ipv4_arp_request_returns_decoded_frame_arp()`

### AC-002 (traces to BC-2.02.009 postcondition 2 — Path 2: non-Ethernet/IPv4 ARP)
`decode_packet(data, datalink)` returns `Err(...)` containing "Non-Ethernet/IPv4 ARP frame"
when `data` is an ARP frame with hw_addr_size=8 (non-standard) or hw_type != 0x0001 or
proto_type != 0x0800. No panic.
- **Test:** `test_decode_non_eth_ipv4_arp_returns_error()`

### AC-003 (traces to BC-2.02.009 postcondition 3 — Path 3: non-IP non-ARP unchanged)
`decode_packet(data, datalink)` returns `Err(...)` containing "No IP layer found" for
non-IP, non-ARP frames (e.g., LLDP EtherType 0x88CC, EtherType 0x9000). This path is
unchanged from pre-ARP behavior.
- **Test:** `test_decode_non_ip_non_arp_frame_returns_no_ip_error()` (existing test updated; ARP subtest now asserts `Ok(DecodedFrame::Arp)` not `Err`)

### AC-004 (traces to BC-2.02.009 postcondition 4 — no panic on all three paths)
No panic occurs for any of the three decode paths: Path 1 returns `Ok(DecodedFrame::Arp)`,
Path 2 returns `Err("Non-Ethernet/IPv4 ARP frame")`, Path 3 returns `Err("No IP layer found")`.
This is verified by VP-008 fuzz harness (updated per VP-008 carry-forward obligation above).
- **Fuzz:** VP-008 harness updated to accept `Result<DecodedFrame>`

### AC-005 (traces to BC-2.02.009 invariant 1 — three-way dispatch on slice.net)
The `Ok(slice)` strict match arm in `decode_packet` implements the three-way dispatch:
`Some(NetSlice::Arp(arp))` → `extract_arp_frame` → `Ok(DecodedFrame::Arp)` or
`Err("Non-Ethernet/IPv4 ARP frame")`; `Some(other_net)` → `strict_ip_triple` →
`Ok(DecodedFrame::Ip)`; `None` → `Err("No IP layer found")`. All three arms compile and
no prior arm is changed.
- **Test:** `cargo check` green; all existing `test_decode_*` tests green.

### AC-006 (traces to BC-2.02.009 invariant 2 — strict_ip_triple unreachable arm)
`strict_ip_triple` gains exactly one new arm `NetSlice::Arp(_) => unreachable!("ARP frames are routed before strict_ip_triple")`. This is a compile-safety arm only — it is never reached at runtime because ARP frames are extracted in `decode_packet`'s `Ok(slice)` arm before `strict_ip_triple` is called. The unreachable! is permitted here (ADR-008 Decision 3, strict path).
- **Test:** `cargo test --all-targets` green (no runtime panic from this arm in any test).

### AC-007 (traces to BC-2.02.009 invariant 2 — lax_ip_triple explicit routing arm, NOT unreachable!)
`lax_ip_triple` gains exactly one new arm `LaxNetSlice::Arp(arp) => ...` that routes to `extract_arp_frame` via the lax arm's `outer_src_mac` extraction from `lax.link`. This arm MUST NOT use `unreachable!`: a snaplen-truncated ARP frame yields `Some(LaxNetSlice::Arp(_))` from etherparse 0.20's lax parser and does reach `lax_ip_triple` at runtime. The arm returns an appropriate sentinel to the caller which then calls `extract_arp_frame` (per ADR-008 Decision 3 v1.6 spec in arp-architecture-delta.md §2.2). No panic occurs on truncated ARP input.
- **Test:** `test_decode_snaplen_truncated_arp_does_not_panic()` — verifies no panic on a truncated ARP byte sequence.

### AC-008 (traces to BC-2.02.009 invariant 3 — lax retry path handles truncated ARP)
The `Err(SliceError::Len(_))` lax-retry arm in `decode_packet` handles `Some(LaxNetSlice::Arp(_))` explicitly, mirroring the strict arm's `outer_src_mac` extraction from `lax.link`. When `extract_arp_frame` returns `Some(frame)`, the lax arm returns `Ok(DecodedFrame::Arp(frame))`; when it returns `None`, the lax arm returns `Err(anyhow!("truncated ARP frame"))`. Both outcomes are non-panic.
- **Test:** `test_decode_snaplen_truncated_arp_lax_path_non_panic()`

### AC-009 (traces to BC-2.02.009 invariant 5 — VP-008 harness return type updated)
The cargo-fuzz harness for VP-008 is updated to accept `Result<DecodedFrame>` (was `Result<ParsedPacket>`). Both `Ok(DecodedFrame::Ip(_))` and `Ok(DecodedFrame::Arp(_))` are valid non-panic outcomes. The existing `SliceError::Len` contract tests `test_decode_snaplen_truncated_ipv6_recovers_via_lax_parsing` and `test_decode_structurally_corrupt_packet_is_rejected_not_lax_recovered` both remain green after the etherparse 0.20 migration (per arp-architecture-delta.md §4.5).
- **Test:** Both contract tests above remain green. VP-008 harness compiles and runs without modification to the assertion logic (only the return type annotation changes).

### AC-010 (traces to BC-2.02.009 — Cargo.toml and prose-sweep)
`Cargo.toml` is updated: `etherparse = "0.16"` → `etherparse = "0.20"`. The version-pin comment block at `Cargo.toml` ~lines 21–26 is updated to reference 0.20 API contract. The `src/decoder.rs` top-of-file `//!` module doc comment AND the `SliceError` import comment block (src ~lines 42–48, which reference "etherparse 0.16 API contract" / "0.17 bump") are both updated to reference etherparse 0.20 in the same prose-sweep commit.
- **Test:** `cargo check` after Cargo.toml bump is green; `cargo test --all-targets` green.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `pub enum DecodedFrame { Ip(ParsedPacket), Arp(ArpFrame) }` | `src/decoder.rs` | Data type (NEW) |
| `pub struct ArpFrame { operation, sender_mac, sender_ip, target_mac, target_ip, outer_src_mac, packet_len }` | `src/decoder.rs` | Data type (NEW) |
| `decode_packet` return type change | `src/decoder.rs` | Effectful shell (reads bytes) |
| `strict_ip_triple` `NetSlice::Arp(_)` arm | `src/decoder.rs` | Compile-safety; unreachable at runtime |
| `lax_ip_triple` `LaxNetSlice::Arp` arm | `src/decoder.rs` | Explicit routing; NOT unreachable |
| `Err(SliceError::Len(_))` lax arm, ARP branch | `src/decoder.rs` | Effectful shell |
| VP-008 cargo-fuzz harness | `fuzz/fuzz_targets/` | Effectful (reads arbitrary bytes) |

Architecture section references: `architecture/module-decomposition.md` (SS-02 decoder.rs, C-5).

## Forbidden Dependencies

`src/decoder.rs` MUST NOT gain a dependency on `src/analyzer/arp.rs`. The `ArpFrame` struct and `DecodedFrame` enum are defined in `src/decoder.rs` and exported; no analyzer code is imported by the decoder. If `src/decoder.rs` gains an `use crate::analyzer::arp` import, the build MUST fail (enforce via `cargo check` or a module-boundary lint).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | 42-byte Ethernet/IPv4 ARP Request (hlen=6, plen=4, hw=Ethernet, proto=IPv4) | Ok(DecodedFrame::Arp) — AC-001 |
| EC-002 | ARP frame with hw_addr_size=8 (non-standard) | Err("Non-Ethernet/IPv4 ARP frame") — AC-002 |
| EC-003 | LLDP frame (EtherType 0x88CC) | Err("No IP layer found") — AC-003 |
| EC-004 | Snaplen-truncated ARP frame (lax path) | lax arm routes to extract_arp_frame; no panic — AC-007/AC-008 |
| EC-005 | Snaplen-truncated IPv6 frame (existing test) | Lax recovery unchanged; DecodedFrame::Ip returned — AC-009 |
| EC-006 | Structurally corrupt packet (existing test) | Rejected, not lax-recovered; Err returned — AC-009 |
| EC-007 | etherparse 0.20 SlicedPacket.link_exts field (formerly .vlan) | No wirerust code accesses .vlan; no compile error. If new ARP test code needs VLAN fields, use .link_exts. |

## Tasks

1. **Bump etherparse in Cargo.toml**: change `etherparse = "0.16"` to `etherparse = "0.20"`. Confirm `cargo check` is green after the bump.
2. **Prose-sweep in src/decoder.rs**: update the top-of-file `//!` module doc AND the `SliceError` import comment block (~lines 42–48) to reference etherparse 0.20 (remove "0.16 API contract" / "0.17 bump" references). Do this in the same commit as the Cargo.toml bump.
3. **Add `DecodedFrame` enum** to `src/decoder.rs`: `pub enum DecodedFrame { Ip(ParsedPacket), Arp(ArpFrame) }`.
4. **Add `ArpFrame` struct** to `src/decoder.rs` with all seven fields exactly as specified in arp-architecture-delta.md §2.1 (operation: u16, sender_mac: [u8;6], sender_ip: [u8;4], target_mac: [u8;6], target_ip: [u8;4], outer_src_mac: Option<[u8;6]>, packet_len: usize).
5. **Update `decode_packet` return type** from `Result<ParsedPacket>` to `Result<DecodedFrame>`.
6. **Add `NetSlice::Arp(_) => unreachable!()` arm** to `strict_ip_triple` (compile-safety; never reached).
7. **Add `LaxNetSlice::Arp(arp) => ...` arm** to `lax_ip_triple` — explicit routing sentinel (NOT unreachable!); see arp-architecture-delta.md §2.2 for the exact code pattern.
8. **Update `Err(SliceError::Len(_))` lax arm** in `decode_packet` to handle `Some(LaxNetSlice::Arp(_))` via `extract_arp_frame` (stub: implement as a placeholder that returns `Err(anyhow!("ARP extraction not yet implemented"))` in STORY-111; full implementation is STORY-112). The lax arm must not panic.
9. **Update VP-008 fuzz harness** return type from `Result<ParsedPacket>` to `Result<DecodedFrame>`; handle both `Ip` and `Arp` variants as non-panic outcomes.
10. **Run `cargo test --all-targets`**: confirm all existing tests pass with 0 failures; confirm the two `SliceError::Len` contract tests pass.
11. **Write new unit tests** for AC-001 through AC-010.

## Test Plan

| AC | Test | Type |
|----|------|------|
| AC-001 | `test_decode_eth_ipv4_arp_request_returns_decoded_frame_arp` | Unit |
| AC-002 | `test_decode_non_eth_ipv4_arp_returns_error` | Unit |
| AC-003 | `test_decode_non_ip_non_arp_frame_returns_no_ip_error` | Unit (update existing) |
| AC-004 | VP-008 harness | cargo-fuzz |
| AC-005 | `cargo check` + existing tests | Compile + Unit |
| AC-006 | All tests (no runtime unreachable!) | Unit |
| AC-007 | `test_decode_snaplen_truncated_arp_does_not_panic` | Unit |
| AC-008 | `test_decode_snaplen_truncated_arp_lax_path_non_panic` | Unit |
| AC-009 | `test_decode_snaplen_truncated_ipv6_recovers_via_lax_parsing`, `test_decode_structurally_corrupt_packet_is_rejected_not_lax_recovered` | Unit (existing — must stay green) |
| AC-010 | `cargo check` after Cargo.toml bump | Build |

## Previous Story Intelligence

STORY-110 (DNP3 Dispatcher Integration, E-15) is the immediate predecessor. Key precedents:
- The VP-007 atomic update completed in STORY-110 raised SEEDED to 23 and EMITTED to 15. STORY-111 does NOT touch `src/mitre.rs` — that work lands in STORY-114.
- The `src/dispatcher.rs` multi-analyzer wiring pattern (STORY-110 added `DispatchTarget::Dnp3`) will be mirrored in STORY-113 for `ArpAnalyzer`. STORY-111 only prepares the decode-path types; no dispatcher or main.rs changes are made in this story.
- E-15 established the strict/lax `decode_packet` shape. This story modifies that shape. The two `SliceError::Len` contract tests from `src/decoder.rs` (added in STORY-003/earlier) are the regression oracle for this migration.

N/A for direct previous-story intelligence within E-16 (this is the first E-16 story).

## Architecture Compliance Rules

Derived from arp-architecture-delta.md §2.1, §2.2, ADR-008 Decisions 1–3, BC-2.02.009 v1.6:

1. **strict_ip_triple ARP arm: `unreachable!` is correct** — ARP frames are routed out of the strict `Ok(slice)` arm before `strict_ip_triple` is ever called. This is a compile-safety net, never reached at runtime (ADR-008 Decision 3).
2. **lax_ip_triple ARP arm: `unreachable!` is FORBIDDEN** — snaplen-truncated ARP frames yield `Some(LaxNetSlice::Arp(_))` from etherparse 0.20's lax parser and DO reach `lax_ip_triple` at runtime. An `unreachable!` here would be a reachable panic, violating VP-008/VP-024 Sub-A (ADR-008 Decision 3 v1.6).
3. **`DecodedFrame` and `ArpFrame` live in `src/decoder.rs`** — not in `src/analyzer/arp.rs`. The decoder is the extraction boundary.
4. **`outer_src_mac` field is `Option<[u8; 6]>`** — `None` for non-Ethernet (SLL) captures, `Some([u8; 6])` for Ethernet. This field is required for D12 mismatch detection in STORY-113.
5. **Cargo.toml version comment** must be updated alongside the version pin — stale comments were a finding class in prior stories.
6. **Both prose blocks in src/decoder.rs must be updated to 0.20** in the same commit (arp-architecture-delta.md §4.1, F-A03 Pass 3 / F-1.3 changelog note).

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `etherparse` | 0.20 (specifically 0.20.1 confirmed) | `SlicedPacket.link_exts` replaces `.vlan`; `SliceError::Len` unchanged; `Ethernet2Slice::source()` returns `[u8; 6]` by value (no dereference needed) |
| `anyhow` | same as existing | `anyhow!("Non-Ethernet/IPv4 ARP frame")`, `anyhow!("truncated ARP frame")` |
| `cargo-fuzz` | same as existing | VP-008 harness; return type annotation update only |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `Cargo.toml` | Modify | `etherparse = "0.16"` → `etherparse = "0.20"`; update version-pin comment ~lines 21–26 |
| `src/decoder.rs` | Modify | Add `DecodedFrame` enum, `ArpFrame` struct; update `decode_packet` return type; add `NetSlice::Arp` and `LaxNetSlice::Arp` arms; prose-sweep both comment blocks to 0.20 |
| `fuzz/fuzz_targets/decode_packet_fuzz.rs` (or equivalent VP-008 harness) | Modify | Update return type from `Result<ParsedPacket>` to `Result<DecodedFrame>`; handle both `Ip` and `Arp` variants |
| `src/decoder.rs` tests module | Modify | Add AC-001..AC-010 tests; update existing ARP subtest in `test_decode_non_ip_frame_returns_error` |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~3,500 |
| BC-2.02.009 | ~4,000 |
| arp-architecture-delta.md §2.1, §2.2, §4.1, §4.3, §4.5 | ~3,000 |
| Existing `src/decoder.rs` | ~3,000 |
| VP-008 fuzz harness file | ~500 |
| Tool outputs (cargo check, cargo test) | ~1,500 |
| **Total estimated** | **~15,500** |

Well within 20–30% of agent context window.

## Dependency Rationale

- `depends_on: [STORY-110]` — STORY-110 is the last completed E-15 story. E-16 begins here. No specific code from STORY-110 is a prerequisite; the dependency ensures the E-15 wave is fully merged before E-16 begins (VP-007 SEEDED=23/EMITTED=15 counts must be stable before STORY-114 adds the ARP entries).
- `blocks: [STORY-112]` — STORY-112 implements `extract_arp_frame` and the full ARP routing in `decode_packet`. It cannot proceed until the `DecodedFrame` enum, `ArpFrame` struct, and return-type change from this story are merged.
