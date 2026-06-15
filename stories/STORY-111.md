---
document_type: story
story_id: STORY-111
epic_id: E-16
version: "1.4"
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
# v1.1 changelog: F4-surfaced decomposition fix: re-scoped ACs to §6 scaffolding boundary
#   (extract_arp_frame end-to-end behavior is STORY-112); added non-panicking placeholder AC
#   for VP-008 (AC-005b); AC-001/002/004/007/008 removed (covered by STORY-112 AC-006/007/004/012).
# v1.4 changelog: F4 symmetric-unreachable! alignment (D-072):
#   lax_ip_triple ARP arm reframed to symmetric-unreachable (NOT "explicit routing")
#     per architect v1.16 / ADR-008 Decision 3 v2.1 ruling; decode_packet intercepts
#     LaxNetSlice::Arp in its Err(SliceError::Len(_)) arm before lax_ip_triple is called;
#     lax_ip_triple returns IpTriple and cannot route ARP; unreachable! is correct compile-
#     safety guard symmetric to strict_ip_triple. Architecture Compliance Rule 2 inverted
#     accordingly. Input-hash recomputed for BC-2.02.009 v1.7 + arp-architecture-delta v1.16.
# v1.3 changelog: F4 confirming-review LOW residuals:
#   coverage-map 'Removed test rows' AC-004→AC-012 sibling-list sync
#     (AC-006/AC-007/AC-004 → AC-006/AC-012/AC-007, matching Coverage Mapping table);
#   File-Structure test-name aligned to AC-003/Test-Plan
#     (rename test_decode_non_ip_frame_returns_error → test_decode_non_ip_non_arp_frame_returns_no_ip_error).
# v1.2 changelog: F4 scoped-adversarial remediation:
#   AC-005 seam pinning — strict Some(NetSlice::Arp) arm maps placeholder None to TEMPORARY
#     Err("ARP extraction not yet implemented"), not "Non-Ethernet/IPv4 ARP frame" (STORY-112 AC-012);
#   AC-005b type fix — placeholder signature fixed to -> Option<ArpFrame> returning None (not Err);
#     Err return in type-mismatched AC-005b prose corrected; non-panic assertion preserved;
#   Task 8 — removed stale "(or None if Option<ArpFrame> signature is chosen)" optionality;
#     signature is fixed to -> Option<ArpFrame>, placeholder body returns None;
#   Coverage-mapping AC-004 row — changed "AC-004 (None/no-panic)" to "AC-012 (strict None→Err path)".
inputs:
  - .factory/specs/architecture/arp-architecture-delta.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.009.md
  - .factory/specs/verification-properties/vp-008-decode-packet-no-panic.md
input-hash: "d05149f"
---

# STORY-111: etherparse 0.20 Migration + DecodedFrame/ArpFrame Types + BC-2.02.009 Revision

## Narrative

- **As a** wirerust maintainer
- **I want** to upgrade etherparse from 0.16 to 0.20, introduce the `DecodedFrame` enum and `ArpFrame` struct in `src/decoder.rs`, change the `decode_packet` return type to `Result<DecodedFrame>`, add the `strict_ip_triple` compile-safety arm for `NetSlice::Arp`, add the `lax_ip_triple` compile-safety arm for `LaxNetSlice::Arp` (also `unreachable!` — symmetric to strict; decode_packet intercepts LaxNetSlice::Arp before lax_ip_triple is called), and ship a non-panicking `extract_arp_frame` placeholder
- **So that** the decode pipeline compiles with the new three-way dispatch structure, all existing IP-pipeline tests remain green, the VP-008 no-panic invariant holds across the new return type and ARP-shaped fuzz inputs, and STORY-112 can implement the real `extract_arp_frame` on top of these scaffolding types

## Behavioral Contracts

| BC | Title | This Story's Role |
|----|-------|-------------------|
| BC-2.02.009 | Non-IP Non-ARP Frames Return No-IP-Layer Error; ARP Frames Return DecodedFrame::Arp | **Structural/type enablement only.** STORY-111 satisfies BC-2.02.009 structurally: it introduces the types (`DecodedFrame`, `ArpFrame`), the dispatch arms, and the return-type change. BC-2.02.009 postcondition 1 (ARP → `Ok(DecodedFrame::Arp(...))`) is behaviorally satisfied in STORY-112 (where `extract_arp_frame` is fully implemented). The no-panic invariant (postcondition 4 / invariant 5) holds in STORY-111 because the `extract_arp_frame` placeholder is non-panicking. **FLAG for orchestrator:** BC-2.02.009 itself may benefit from a per-story split note distinguishing structural enablement (STORY-111) from behavioral satisfaction (STORY-112); this observation is noted here for the PO/orchestrator rather than editing the BC. |

## VP-008 Carry-Forward Obligation (F3-OBL-STORY111-VP008)

The cargo-fuzz harness for VP-008 (`decode_packet` no-panic) currently expects `Result<ParsedPacket>`. After this story's changes `decode_packet` returns `Result<DecodedFrame>`. The fuzz harness MUST be updated to:

1. Accept `Result<DecodedFrame>` as the return type.
2. Handle both `Ok(DecodedFrame::Ip(_))` and `Ok(DecodedFrame::Arp(_))` as non-panic outcomes.
3. Remove any match-exhaustiveness arm that assumed `SliceError::Len` is the only lax path (two non-exhaustive `NetSlice`/`LaxNetSlice` match breaks will appear and MUST be resolved with the new arms added in this story).
4. Keep the no-panic invariant assertion unchanged (any `Ok` or `Err` variant is acceptable; only a panic fails the harness).

This obligation is carried from BC-2.02.009 Invariant 5 and arp-architecture-delta.md §4.3.

## Acceptance Criteria

> **Scope boundary (§6 scaffolding):** Every AC below is satisfiable within STORY-111 without any dependency on STORY-112's `extract_arp_frame` implementation. End-to-end ARP decode behavior (`Ok(DecodedFrame::Arp(...))` with real field values, `Err("Non-Ethernet/IPv4 ARP frame")` from the decode_packet layer) is owned by STORY-112.
>
> **Removed ACs (covered by STORY-112):** AC-001, AC-002, AC-004, AC-007, AC-008 have been removed from STORY-111. See "Coverage Mapping" section below for the exact STORY-112 ACs that cover each removed behavior.

### AC-003 (traces to BC-2.02.009 postcondition 3 — Path 3: non-IP non-ARP unchanged)
`decode_packet(data, datalink)` returns `Err(...)` containing "No IP layer found" for
non-IP, non-ARP frames (e.g., LLDP EtherType 0x88CC, EtherType 0x9000). This path is
unchanged from pre-ARP behavior. The existing test for this behavior is updated to remove
any ARP subtest clause (the ARP subtest asserting `Ok(DecodedFrame::Arp)` belongs to
STORY-112, not this story).
- **Test:** `test_decode_non_ip_non_arp_frame_returns_no_ip_error()` (existing test updated to
  exercise only the non-IP non-ARP path; the ARP path is not asserted here)

### AC-005 (traces to BC-2.02.009 invariant 1 — three-way dispatch arms exist and crate compiles)
The `Ok(slice)` strict match arm in `decode_packet` introduces the three-way dispatch structure:
`Some(NetSlice::Arp(arp))` → calls `extract_arp_frame(...)` (non-panicking placeholder, signature
`-> Option<ArpFrame>`); the arm maps the returned `Option` to `Result<DecodedFrame>` as follows:
`None` maps to a TEMPORARY `Err(anyhow!("ARP extraction not yet implemented"))` — NOT the string
`"Non-Ethernet/IPv4 ARP frame"` (that decode-layer error string is STORY-112 AC-012's territory).
STORY-112 replaces this temporary mapping with the real `Some(f) => Ok(DecodedFrame::Arp(f))` /
`None => Err(anyhow!("Non-Ethernet/IPv4 ARP frame"))` logic. The two remaining arms:
`Some(other_net)` → `strict_ip_triple` → `Ok(DecodedFrame::Ip)`;
`None` → `Err("No IP layer found")`. All three arms compile with `cargo check` green.
No assertion is made about `extract_arp_frame`'s return value in this story — that is
STORY-112's scope. All existing `test_decode_*` IP-path tests remain green.
- **Test:** `cargo check` green; all existing `test_decode_*` IP-path tests green.

### AC-005b (traces to BC-2.02.009 invariant 5 and VP-008 — non-panicking extract_arp_frame placeholder)
STORY-111 ships `extract_arp_frame` as a non-panicking placeholder with signature
`-> Option<ArpFrame>`; the placeholder body returns `None`. The placeholder MUST NOT use
`todo!()`, `unimplemented!()`, or any other panicking macro. The `decode_packet` ARP arm
wraps this `None` into a TEMPORARY `Err(anyhow!("ARP extraction not yet implemented"))`.
STORY-112 replaces this placeholder with the full implementation and changes the temporary
mapping to the real error string. `decode_packet` does not panic on ARP-shaped input in
STORY-111: the ARP arm routes to the non-panicking placeholder and returns a non-panic `Err`.
- **Test:** `test_decode_arp_shaped_input_does_not_panic()` — verifies that `decode_packet`
  called with a valid Ethernet/IPv4 ARP frame byte sequence does not panic (the result value
  is not asserted; only non-panic is required here).

### AC-006 (traces to BC-2.02.009 invariant 2 — strict_ip_triple unreachable arm)
`strict_ip_triple` gains exactly one new arm `NetSlice::Arp(_) => unreachable!("ARP frames are routed before strict_ip_triple")`. This is a compile-safety arm only — it is never reached at runtime because ARP frames are extracted in `decode_packet`'s `Ok(slice)` arm before `strict_ip_triple` is called. The unreachable! is permitted here (ADR-008 Decision 3, strict path).
- **Test:** `cargo test --all-targets` green (no runtime panic from this arm in any test).

### AC-009 (traces to BC-2.02.009 invariant 5 — VP-008 harness return type updated)
The cargo-fuzz harness for VP-008 is updated to accept `Result<DecodedFrame>` (was `Result<ParsedPacket>`). Both `Ok(DecodedFrame::Ip(_))` and `Ok(DecodedFrame::Arp(_))` are valid non-panic outcomes. The existing `SliceError::Len` contract tests `test_decode_snaplen_truncated_ipv6_recovers_via_lax_parsing` and `test_decode_structurally_corrupt_packet_is_rejected_not_lax_recovered` both remain green after the etherparse 0.20 migration (per arp-architecture-delta.md §4.5).
- **Test:** Both contract tests above remain green. VP-008 harness compiles and runs without modification to the assertion logic (only the return type annotation changes).

### AC-010 (traces to BC-2.02.009 — Cargo.toml and prose-sweep)
`Cargo.toml` is updated: `etherparse = "0.16"` → `etherparse = "0.20"`. The version-pin comment block at `Cargo.toml` ~lines 21–26 is updated to reference 0.20 API contract. The `src/decoder.rs` top-of-file `//!` module doc comment AND the `SliceError` import comment block (src ~lines 42–48, which reference "etherparse 0.16 API contract" / "0.17 bump") are both updated to reference etherparse 0.20 in the same prose-sweep commit.
- **Test:** `cargo check` after Cargo.toml bump is green; `cargo test --all-targets` green.

## Coverage Mapping (Removed ACs → STORY-112)

The following behaviors were removed from STORY-111 because they require `extract_arp_frame`
(STORY-112's scope). No coverage gap exists: each removed behavior maps to an existing or newly
added STORY-112 AC.

| Removed STORY-111 AC | Behavior | Covered by STORY-112 AC |
|----------------------|----------|------------------------|
| AC-001 | `decode_packet` returns `Ok(DecodedFrame::Arp(frame))` for well-formed Ethernet/IPv4 ARP | AC-006 (`test_decode_packet_routes_arp_to_decoded_frame_arp`) |
| AC-002 | `decode_packet` returns `Err("Non-Ethernet/IPv4 ARP frame")` for non-Eth/IPv4 ARP at decode layer | AC-012 (NEW — added to STORY-112; see below) |
| AC-004 | No panic on all three ARP decode paths (via extract_arp_frame) | AC-006 (strict Ok path), AC-012 (strict None→Err path), AC-007 (lax path) |
| AC-007 | `lax_ip_triple` routes `LaxNetSlice::Arp` to `extract_arp_frame`; no panic on truncated ARP | AC-007 (`test_decode_packet_lax_arm_truncated_arp_non_panic`) |
| AC-008 | Lax `Err(SliceError::Len)` arm maps `extract_arp_frame` `Some(f)→Ok(DecodedFrame::Arp)`, `None→Err("truncated ARP frame")` | AC-007 (`test_decode_packet_lax_arm_truncated_arp_non_panic`) |

**Note on AC-002 gap:** STORY-112 AC-004 covers `extract_arp_frame` returning `None` for bad
hw/proto size, and STORY-112 Task 2 documents the `None → Err(anyhow!("Non-Ethernet/IPv4 ARP frame"))`
mapping at the `decode_packet` level. However, the decode_packet-level Err string assertion was
not an explicit AC in STORY-112. A new AC-012 has been added to STORY-112 to cover this explicitly
(see STORY-112 v1.1).

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `pub enum DecodedFrame { Ip(ParsedPacket), Arp(ArpFrame) }` | `src/decoder.rs` | Data type (NEW) |
| `pub struct ArpFrame { operation, sender_mac, sender_ip, target_mac, target_ip, outer_src_mac, packet_len }` | `src/decoder.rs` | Data type (NEW) |
| `decode_packet` return type change | `src/decoder.rs` | Effectful shell (reads bytes) |
| `strict_ip_triple` `NetSlice::Arp(_)` arm | `src/decoder.rs` | Compile-safety; unreachable at runtime |
| `lax_ip_triple` `LaxNetSlice::Arp` arm | `src/decoder.rs` | `unreachable!` compile-safety guard (provably dead; decode_packet intercepts LaxNetSlice::Arp in Err(SliceError::Len) arm before lax_ip_triple is called) — symmetric to strict_ip_triple |
| `Err(SliceError::Len(_))` lax arm, ARP branch | `src/decoder.rs` | Effectful shell |
| VP-008 cargo-fuzz harness | `fuzz/fuzz_targets/` | Effectful (reads arbitrary bytes) |

Architecture section references: `architecture/module-decomposition.md` (SS-02 decoder.rs, C-5).

## Forbidden Dependencies

`src/decoder.rs` MUST NOT gain a dependency on `src/analyzer/arp.rs`. The `ArpFrame` struct and `DecodedFrame` enum are defined in `src/decoder.rs` and exported; no analyzer code is imported by the decoder. If `src/decoder.rs` gains an `use crate::analyzer::arp` import, the build MUST fail (enforce via `cargo check` or a module-boundary lint).

## Edge Cases

| ID | Description | Expected Behavior | Scope |
|----|-------------|-------------------|-------|
| EC-001 | 42-byte Ethernet/IPv4 ARP Request input to `decode_packet` | Routed to non-panicking placeholder; no panic — AC-005b | STORY-111 (no-panic only; Ok(DecodedFrame::Arp) asserted in STORY-112 EC-007/AC-006) |
| EC-002 | ARP frame with hw_addr_size=8 (non-standard) at decode_packet level | Non-panicking placeholder returns non-panic result — AC-005b | STORY-111 no-panic; Err("Non-Ethernet/IPv4 ARP frame") asserted in STORY-112 AC-012 |
| EC-003 | LLDP frame (EtherType 0x88CC) | Err("No IP layer found") — AC-003 | STORY-111 |
| EC-004 | Snaplen-truncated ARP frame (lax path) | lax arm structure exists (AC-005); no panic via placeholder (AC-005b) | STORY-111 structure; end-to-end routing asserted in STORY-112 AC-007 |
| EC-005 | Snaplen-truncated IPv6 frame (existing test) | Lax recovery unchanged; DecodedFrame::Ip returned — AC-009 | STORY-111 |
| EC-006 | Structurally corrupt packet (existing test) | Rejected, not lax-recovered; Err returned — AC-009 | STORY-111 |
| EC-007 | etherparse 0.20 SlicedPacket.link_exts field (formerly .vlan) | No wirerust code accesses .vlan; no compile error. If new ARP test code needs VLAN fields, use .link_exts. | STORY-111 |

## Tasks

1. **Bump etherparse in Cargo.toml**: change `etherparse = "0.16"` to `etherparse = "0.20"`. Confirm `cargo check` is green after the bump (AC-010).
2. **Prose-sweep in src/decoder.rs**: update the top-of-file `//!` module doc AND the `SliceError` import comment block (~lines 42–48) to reference etherparse 0.20 (remove "0.16 API contract" / "0.17 bump" references). Do this in the same commit as the Cargo.toml bump (AC-010).
3. **Add `DecodedFrame` enum** to `src/decoder.rs`: `pub enum DecodedFrame { Ip(ParsedPacket), Arp(ArpFrame) }`.
4. **Add `ArpFrame` struct** to `src/decoder.rs` with all seven fields exactly as specified in arp-architecture-delta.md §2.1 (operation: u16, sender_mac: [u8;6], sender_ip: [u8;4], target_mac: [u8;6], target_ip: [u8;4], outer_src_mac: Option<[u8;6]>, packet_len: usize).
5. **Update `decode_packet` return type** from `Result<ParsedPacket>` to `Result<DecodedFrame>`.
6. **Add `NetSlice::Arp(_) => unreachable!()` arm** to `strict_ip_triple` (compile-safety; never reached) (AC-006).
7. **Add `LaxNetSlice::Arp(_) => unreachable!(...)` arm** to `lax_ip_triple` — compile-safety guard symmetric to the strict_ip_triple arm; provably dead because decode_packet intercepts `Some(LaxNetSlice::Arp(_))` in its `Err(SliceError::Len(_))` arm before lax_ip_triple is ever called; lax_ip_triple returns IpTriple and cannot route ARP; see arp-architecture-delta.md §2.2 (ADR-008 Decision 3 v2.1) (AC-005).
8. **Add non-panicking `extract_arp_frame` placeholder** in `src/decoder.rs`: the signature is fixed to `fn extract_arp_frame(arp: &ArpPacketSlice<'_>, outer_src_mac: Option<[u8; 6]>, packet_len: usize) -> Option<ArpFrame>` (matching STORY-112's expectation — the `Option<ArpFrame>` return type is not a choice, it is fixed). The placeholder body returns `None`. MUST NOT use `todo!()`, `unimplemented!()`, or any other panicking macro. The `decode_packet` ARP arm (`Some(NetSlice::Arp(arp))`) maps this `None` to a temporary `Err(anyhow!("ARP extraction not yet implemented"))`. STORY-112 replaces both the placeholder body and the temporary mapping with the real implementation (AC-005b).
9. **Update `Err(SliceError::Len(_))` lax arm** in `decode_packet` to handle `Some(LaxNetSlice::Arp(_))` by calling the non-panicking `extract_arp_frame` placeholder. This arm must not panic (AC-005, AC-005b).
10. **Update VP-008 fuzz harness** return type from `Result<ParsedPacket>` to `Result<DecodedFrame>`; handle both `Ip` and `Arp` variants as non-panic outcomes (AC-009).
11. **Run `cargo test --all-targets`**: confirm all existing tests pass with 0 failures; confirm the two `SliceError::Len` contract tests pass (AC-009).
12. **Write new unit tests** for AC-003, AC-005, AC-005b, AC-006, AC-009, AC-010 only. Do NOT write tests asserting `Ok(DecodedFrame::Arp(...))` with real field values — those are STORY-112's test territory.

## Test Plan

| AC | Test | Type | Notes |
|----|------|------|-------|
| AC-003 | `test_decode_non_ip_non_arp_frame_returns_no_ip_error` | Unit (update existing) | Remove any ARP subtest clause |
| AC-005 | `cargo check` + existing `test_decode_*` IP-path tests | Compile + Unit | Three-way dispatch arms compile; no existing test regressions |
| AC-005b | `test_decode_arp_shaped_input_does_not_panic` | Unit (NEW) | ARP input → no panic; result value not asserted |
| AC-006 | All tests (no runtime unreachable! from this arm) | Unit | `cargo test --all-targets` green |
| AC-009 | `test_decode_snaplen_truncated_ipv6_recovers_via_lax_parsing`, `test_decode_structurally_corrupt_packet_is_rejected_not_lax_recovered` | Unit (existing — must stay green) | |
| AC-010 | `cargo check` after Cargo.toml bump | Build | |

**Removed test rows (covered by STORY-112):**
- AC-001 → STORY-112 `test_decode_packet_routes_arp_to_decoded_frame_arp`
- AC-002 → STORY-112 `test_decode_packet_arp_non_eth_ipv4_returns_error` (AC-012, NEW)
- AC-004 → STORY-112 AC-006/AC-012/AC-007
- AC-007 → STORY-112 `test_decode_packet_lax_arm_truncated_arp_non_panic`
- AC-008 → STORY-112 `test_decode_packet_lax_arm_truncated_arp_non_panic`

## Previous Story Intelligence

STORY-110 (DNP3 Dispatcher Integration, E-15) is the immediate predecessor. Key precedents:
- The VP-007 atomic update completed in STORY-110 raised SEEDED to 23 and EMITTED to 15. STORY-111 does NOT touch `src/mitre.rs` — that work lands in STORY-114.
- The `src/dispatcher.rs` multi-analyzer wiring pattern (STORY-110 added `DispatchTarget::Dnp3`) will be mirrored in STORY-113 for `ArpAnalyzer`. STORY-111 only prepares the decode-path types; no dispatcher or main.rs changes are made in this story.
- E-15 established the strict/lax `decode_packet` shape. This story modifies that shape. The two `SliceError::Len` contract tests from `src/decoder.rs` (added in STORY-003/earlier) are the regression oracle for this migration.

N/A for direct previous-story intelligence within E-16 (this is the first E-16 story).

## Architecture Compliance Rules

Derived from arp-architecture-delta.md §2.1, §2.2, ADR-008 Decisions 1–3, BC-2.02.009 v1.6:

1. **strict_ip_triple ARP arm: `unreachable!` is correct** — ARP frames are routed out of the strict `Ok(slice)` arm before `strict_ip_triple` is ever called. This is a compile-safety net, never reached at runtime (ADR-008 Decision 3).
2. **lax_ip_triple ARP arm: `unreachable!` is CORRECT (compile-safety guard, provably dead)** — decode_packet intercepts `Some(LaxNetSlice::Arp(_))` in its `Err(SliceError::Len(_))` arm before lax_ip_triple is called; lax_ip_triple returns IpTriple and cannot route ARP; routing lives in decode_packet. VP-008/VP-024 Sub-A no-panic is guaranteed by decode_packet interception + panic-free extract_arp_frame (ADR-008 Decision 3 v2.1; arp-architecture-delta §2.2 v1.16). Symmetric to strict_ip_triple.
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
| `src/decoder.rs` | Modify | Add `DecodedFrame` enum, `ArpFrame` struct; update `decode_packet` return type; add `NetSlice::Arp` dispatch arm (strict path) + `LaxNetSlice::Arp` routing arm (lax path); add non-panicking `extract_arp_frame` placeholder (returns `None` — signature matches STORY-112 expectation); prose-sweep both comment blocks to 0.20 |
| `fuzz/fuzz_targets/decode_packet_fuzz.rs` (or equivalent VP-008 harness) | Modify | Update return type from `Result<ParsedPacket>` to `Result<DecodedFrame>`; handle both `Ip` and `Arp` variants as non-panic outcomes |
| `src/decoder.rs` tests module | Modify | Add AC-003, AC-005, AC-005b, AC-006, AC-009, AC-010 tests; update + rename existing `test_decode_non_ip_frame_returns_error` → `test_decode_non_ip_non_arp_frame_returns_no_ip_error` (remove the ARP subtest clause; ARP assertions move to STORY-112); `extract_arp_frame` entry = non-panicking placeholder (full impl in STORY-112) |

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
