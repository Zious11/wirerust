---
document_type: story
story_id: STORY-112
epic_id: E-16
version: "1.4"
status: draft
producer: story-writer
timestamp: 2026-06-13T00:00:00Z
phase: f3
points: 8
priority: P0
depends_on: [STORY-111]
blocks: [STORY-113]
behavioral_contracts:
  - BC-2.16.001
  - BC-2.16.002
  - BC-2.16.015
verification_properties: [VP-024]
tdd_mode: strict
target_module: decoder
subsystems: [SS-02, SS-16]
estimated_days: 3
feature_id: issue-009-arp-security-analyzer
github_issue: 9
# BC status: BC-2.16.001 v1.0, BC-2.16.002 v1.0, BC-2.16.015 v1.2 — BC-2.16.015 updated 2026-06-14
# VP-024 Sub-A: 3 Kani harnesses land in this story (safety, eth_ipv4_correctness, none_on_bad_size)
# ArpAnalyzer stub: new, process_arp no-op, for main.rs pattern-match wiring only
# v1.1 changelog: F4-surfaced decomposition fix: added AC-012 to explicitly cover
#   decode_packet-level Err("Non-Ethernet/IPv4 ARP frame") for non-Eth/IPv4 ARP at the
#   decode layer. This behavior was previously implicit (covered by Task 2 prose + AC-004),
#   but lacked a named AC. Added to close coverage gap surfaced when STORY-111
#   AC-002 was removed and mapped here.
# v1.4 changelog: F-3 citation-exactness fix (DF-AC-TEST-NAME-SYNC-001 v2, pass 2):
#   All AC `**Test:**` citations updated from unprefixed names to exact BC-prefixed
#   fn names as they appear in tests/bc_2_16_story112_arp_tests.rs. Test Plan table
#   updated to match. No semantic or AC changes; body-only edit.
# v1.3 changelog: F4 symmetric-unreachable! alignment (D-072):
#   Frontmatter changelog stale framing removed — prior v1.2 note referencing
#     "lax_ip_triple ARP arm is explicit routing, NOT unreachable" is superseded by
#     architect v1.16 / ADR-008 Decision 3 v2.1 ruling (symmetric-unreachable!).
#   Input-hash recomputed for BC-2.16.015 v1.3 + arp-architecture-delta v1.16.
# v1.2 changelog: F4 scoped-adversarial remediation:
#   File Structure AC count — changed "AC-001..AC-011" to "AC-001..AC-012" to include AC-012
#     (Task 10 and Test Plan already referenced AC-012; only this prose row lagged);
#   Input-hash restamped to reflect new BC-2.16.015 content.
inputs:
  - .factory/specs/architecture/arp-architecture-delta.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.001.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.002.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.015.md
  - .factory/specs/verification-properties/vp-024-arp-parse-safety.md
input-hash: "8a4d566"
---

# STORY-112: extract_arp_frame + decode_packet ARP Routing (Both Paths) + ArpAnalyzer Stub + VP-024 Sub-A

## Narrative

- **As a** wirerust maintainer
- **I want** to implement `extract_arp_frame(arp, outer_src_mac, packet_len)` in `src/decoder.rs`, complete the `decode_packet` ARP early-extraction routing in BOTH the strict `Ok(slice)` arm (NetSlice::Arp) and the lax `Err(SliceError::Len(_))` arm (LaxNetSlice::Arp per ADR-008 Decision 3 v1.6), wire `DecodedFrame` pattern-matching in `main.rs`, add an `ArpAnalyzer` stub, and deliver VP-024 Sub-A Kani harnesses
- **So that** ARP frames are correctly extracted as `ArpFrame` structs and routed to the `ArpAnalyzer` stub in `main.rs`, the decode-vs-analysis separation invariant (BC-2.16.015) is structurally enforced, and the Sub-A Kani harnesses formally verify extraction panic-freedom and field-copy correctness

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.16.001 | ARP Request Frame Correctly Parsed from ArpPacketSlice |
| BC-2.16.002 | ARP Reply Frame Correctly Parsed from ArpPacketSlice |
| BC-2.16.015 | Decode-vs-Analysis Separation — DecodedFrame::Arp Always Produced; Analysis Gated on --arp |

## VP-024 Sub-A Kani Harnesses

> **VP-024 lock-timing note:** VP-024 does not LOCK at STORY-112 — only Sub-A (3 Kani harnesses) lands here; the umbrella VP-024 locks at STORY-113/F6 once Sub-B/C/D are present (per VP-INDEX).

Three Kani harnesses are written in this story, all targeting `extract_arp_frame`:

**verify_extract_arp_frame_safety** — arbitrary `[u8; 28]` buffer with HTYPE/PTYPE/HLEN/PLEN fixed for Ethernet/IPv4 (0x0001, 0x0800, 6, 4); OPER and address fields symbolic; assert `extract_arp_frame` never panics and returns `Some(...)`. This is the primary VP-024 Sub-A no-panic proof.

**verify_extract_arp_frame_eth_ipv4_correctness** — fixed Request frame (op=1, known MAC/IP values); assert returned `ArpFrame` fields match the byte-exact copies from `ArpPacketSlice` accessors (BC-2.16.001 postconditions 2–8). Uses `kani::assume` to constrain input to valid Ethernet/IPv4 ARP.

**verify_extract_arp_frame_none_on_bad_size** — symbolic buffer with hw_addr_size and proto_addr_size drawn from all values ≠ 6 and ≠ 4 respectively; assert `extract_arp_frame` returns `None`. No panic. Covers BC-2.16.001 EC-007 (hw_addr_size=8) and EC-008 (proto_addr_size=16) paths.

## Acceptance Criteria

### AC-001 (traces to BC-2.16.001 postcondition 1 — ARP Request extraction returns Some)
`extract_arp_frame(arp, outer_src_mac, packet_len)` returns `Some(ArpFrame { ... })` when
given an `ArpPacketSlice` from a valid 28-byte (minimum) Ethernet/IPv4 ARP Request (hw_type=0x0001, proto_type=0x0800, hlen=6, plen=4, op=1).
- **Test:** `test_BC_2_16_001_extract_arp_frame_request_returns_some`

### AC-002 (traces to BC-2.16.001 postconditions 2–8 — field copy fidelity, Request)
For an ARP Request, the returned `ArpFrame` has: `operation == 1`; `sender_mac` equals
`arp.sender_hw_addr()[..6]` exactly; `sender_ip` equals `arp.sender_protocol_addr()[..4]`
exactly; `target_mac` equals `arp.target_hw_addr()[..6]` exactly; `target_ip` equals
`arp.target_protocol_addr()[..4]` exactly; `outer_src_mac` equals the parameter passed in;
`packet_len` equals the parameter passed in.
- **Test:** `test_BC_2_16_001_extract_arp_frame_request_field_copy_fidelity`

### AC-003 (traces to BC-2.16.002 postcondition 1 and 2 — ARP Reply extraction returns Some with operation=2)
`extract_arp_frame(arp, outer_src_mac, packet_len)` returns `Some(ArpFrame { operation: 2, ... })`
when given a valid Ethernet/IPv4 ARP Reply (op=2). All seven fields are copied exactly (same
field-copy assertions as AC-002 with operation=2).
- **Test:** `test_BC_2_16_002_extract_arp_frame_reply_returns_some_with_correct_fields`

### AC-004 (traces to BC-2.16.001 EC-007/EC-008 — None on bad hw/proto size)
`extract_arp_frame` returns `None` when `hw_addr_size != 6` (e.g., 8) or `proto_addr_size != 4`
(e.g., 16). No panic. This is the D11 malformed path (finding emission is STORY-113's responsibility — this story only ensures None is returned).
- **Test:** `test_BC_2_16_001_extract_arp_frame_none_on_hw_addr_size_8`, `test_BC_2_16_001_extract_arp_frame_none_on_proto_addr_size_16`

### AC-005 (traces to BC-2.16.001 EC-003 — outer_src_mac=None passthrough)
`extract_arp_frame(arp, None, pkt_len)` returns `Some(ArpFrame { outer_src_mac: None, ... })`.
The function passes `outer_src_mac` through unchanged regardless of its value.
- **Test:** `test_BC_2_16_001_extract_arp_frame_outer_src_mac_none_passthrough`

### AC-006 (traces to BC-2.16.015 postcondition 1/2 — decode always produces DecodedFrame::Arp for valid frames)
`decode_packet` routes `NetSlice::Arp(arp)` in the strict `Ok(slice)` arm to `extract_arp_frame`,
producing `Ok(DecodedFrame::Arp(frame))` for valid Ethernet/IPv4 ARP. The `outer_src_mac` is
extracted from `slice.link` (Ethernet2Slice::source() by value in etherparse 0.20.1). This
routing is unconditional — it does not depend on the `--arp` flag.
- **Test:** `test_BC_2_16_015_decode_packet_routes_arp_to_decoded_frame_arp`

### AC-007 (traces to BC-2.16.015 postcondition — lax arm also routes ARP)
`decode_packet`'s `Err(SliceError::Len(_))` lax arm handles `Some(LaxNetSlice::Arp(arp))` via
`extract_arp_frame` with `outer_src_mac` extracted from `lax.link`. When `extract_arp_frame`
returns `Some(frame)`, the lax arm returns `Ok(DecodedFrame::Arp(frame))`; when it returns
`None`, the lax arm returns `Err(anyhow!("truncated ARP frame"))`. No panic for truncated input.
(Completes the stub left in STORY-111 task 8.)
- **Test:** `test_BC_2_16_015_decode_packet_lax_arm_truncated_arp_non_panic`

### AC-008 (traces to BC-2.16.015 postconditions 5/6 — unconditional ArpAnalyzer stub wiring in main.rs)
`main.rs` pattern-matches on `DecodedFrame::Arp(frame)` and calls
`arp_analyzer.process_arp(&frame, ts)` unconditionally (the stub returns `vec![]` — no-op).
The `--arp` flag does NOT exist in this story; `args.arp` flag-gating is added in STORY-113
(BC-2.16.011). The `DecodedFrame::Ip(p)` arm remains unchanged, routing `p` to the existing
IP pipeline. ARP frames NEVER reach `StreamDispatcher`.
- **Note:** `--arp-spoof-threshold` is added in STORY-114; `--arp-storm-rate` is added in STORY-115.
- **Test:** `test_BC_2_16_015_main_arp_arm_calls_process_arp_stub` (verifies `process_arp` is called and returns `vec![]`)

### AC-009 (traces to BC-2.16.015 invariant 2 — ARP bypasses IP pipeline)
An `Ok(DecodedFrame::Arp(_))` result from `decode_packet` never reaches `StreamDispatcher`,
the TCP reassembler, or any `ProtocolAnalyzer`. The `DecodedFrame::Arp` arm in `main.rs` exits
the match without calling `dispatcher.on_data(...)` or any IP-pipeline method.
- **Test:** `test_BC_2_16_015_arp_frame_never_reaches_stream_dispatcher` — processes a `DecodedFrame::Arp` through the main-loop dispatch path and asserts via spy/counter that `dispatcher.on_data` (or equivalent IP-pipeline entry) is never invoked.

### AC-010 (traces to BC-2.16.015 — ArpAnalyzer stub: process_arp no-op)
`src/analyzer/arp.rs` is created with `pub struct ArpAnalyzer` and `impl ArpAnalyzer`:
`pub fn new() -> ArpAnalyzer` (parameterless default stub — `--arp-spoof-threshold` and
`--arp-storm-rate` CLI flags do not exist yet; they are added in STORY-114 and STORY-115
respectively, at which point the constructor signature gains those parameters);
`pub fn process_arp(&mut self, frame: &ArpFrame, timestamp_secs: u32) -> Vec<Finding>` (returns
`vec![]` — no-op stub). `pub mod arp;` is added to `src/analyzer/mod.rs`. The stub compiles
with `cargo check` and passes `cargo clippy --all-targets -- -D warnings`.
- **Test:** `cargo check`, `cargo clippy` green.

### AC-011 (traces to BC-2.16.001/BC-2.16.002 — VP-024 Sub-A Kani harnesses)
Three Kani harnesses (`verify_extract_arp_frame_safety`, `verify_extract_arp_frame_eth_ipv4_correctness`, `verify_extract_arp_frame_none_on_bad_size`) are written in `src/decoder.rs`'s `#[cfg(kani)] mod kani_proofs` (or equivalent). All three report `VERIFICATION:- SUCCESSFUL` when run individually with `cargo kani --harness <name>`.
- **Kani:** Run at F6 formal-hardening gate for wave 40. Not a `cargo test` prerequisite.

### AC-012 (traces to BC-2.16.015 postcondition 1 — decode_packet returns Err for non-Eth/IPv4 ARP at decode layer)
`decode_packet(data, datalink)` returns `Err(...)` containing "Non-Ethernet/IPv4 ARP frame" when
`data` is an ARP frame where `extract_arp_frame` returns `None` (i.e., hw_addr_size != 6, or
proto_addr_size != 4, or hw_type != 0x0001, or proto_type != 0x0800). No panic occurs.
This is the decode_packet-level error string produced by the strict `Ok(slice)` arm when
`extract_arp_frame` returns `None` (per Task 2: `None => Err(anyhow!("Non-Ethernet/IPv4 ARP frame"))`).
- **Test:** `test_BC_2_16_015_decode_packet_arp_non_eth_ipv4_returns_error` — calls `decode_packet` with a
  raw byte sequence for an ARP frame with hw_addr_size=8 (or hw_type != 0x0001); asserts the
  result is `Err(e)` where `e.to_string()` contains "Non-Ethernet/IPv4 ARP frame".
- **Coverage note:** This AC was added in v1.1 to close the coverage gap created when STORY-111
  AC-002 was removed. AC-004 covers `extract_arp_frame` returning `None`; this AC covers the
  resulting `decode_packet`-level error string (the two together close the full decode path).

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `fn extract_arp_frame(arp: &ArpPacketSlice<'_>, outer_src_mac: Option<[u8; 6]>, packet_len: usize) -> Option<ArpFrame>` | `src/decoder.rs` | Pure core (VP-024 Sub-A Kani target) |
| `decode_packet` ARP routing — strict arm | `src/decoder.rs` | Effectful shell |
| `decode_packet` ARP routing — lax arm | `src/decoder.rs` | Effectful shell |
| `main.rs` `DecodedFrame` pattern-match | `src/main.rs` | Effectful shell |
| `pub struct ArpAnalyzer` stub | `src/analyzer/arp.rs` | Stub (pure core shell; no-op) |
| `impl ArpAnalyzer::new` | `src/analyzer/arp.rs` | Stub |
| `impl ArpAnalyzer::process_arp` (no-op) | `src/analyzer/arp.rs` | Stub |
| `pub mod arp;` | `src/analyzer/mod.rs` | Module declaration |
| VP-024 Sub-A Kani harnesses | `src/decoder.rs` `#[cfg(kani)]` | Kani formal verification |

Architecture section references: `architecture/module-decomposition.md` (SS-02 decoder.rs C-5, SS-16 C-23 ArpAnalyzer), `architecture/dependency-graph.md` (SS-16 depends on SS-02 for ArpFrame type).

## Forbidden Dependencies

- `src/decoder.rs` MUST NOT import from `src/analyzer/arp.rs`. The `ArpFrame` type flows outward from decoder to analyzer, not the reverse.
- `src/analyzer/arp.rs` stub MUST NOT import `src/dispatcher.rs`. `ArpAnalyzer` does not go through the stream dispatcher (per arp-architecture-delta.md §1: "ArpAnalyzer does NOT implement ProtocolAnalyzer or StreamAnalyzer").

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ARP Request, outer_src_mac=Some(matching sender_mac) | Some(ArpFrame { outer_src_mac: Some(...) }); D12 check is analyzer's responsibility |
| EC-002 | ARP Request, outer_src_mac=None (SLL capture) | Some(ArpFrame { outer_src_mac: None }); D12 skipped by analyzer |
| EC-003 | ARP Reply with op=2, sender_ip==target_ip (GARP) | Some(ArpFrame { operation: 2, ... }); extractor is agnostic to GARP condition |
| EC-004 | hw_addr_size=8 (non-standard) | None; no panic |
| EC-005 | proto_addr_size=16 (IPv6 proto) | None; no panic |
| EC-006 | op=0 (undefined opcode) | Some(ArpFrame { operation: 0 }); extractor is opcode-agnostic |
| EC-007 | Snaplen-truncated ARP (lax path) | Ok(DecodedFrame::Arp) if truncated but hw/proto correct; Err("truncated ARP frame") if extract returns None |
| EC-008 | DecodedFrame::Arp arm in main.rs; stub unconditionally calls process_arp (returns vec![]) | Stub wiring verified; --arp flag-gating added in STORY-113 |

## Tasks

1. **Implement `extract_arp_frame`** in `src/decoder.rs`: check hw_addr_type, proto_addr_type, hw_addr_size, proto_addr_size; return `None` if any check fails; copy all six address fields and operation into `ArpFrame`; pass `outer_src_mac` and `packet_len` through unchanged.
2. **Complete the strict `Ok(slice)` arm** in `decode_packet`: replace the STORY-111 placeholder with the full `NetSlice::Arp(arp) => { let outer_src_mac = slice.link.as_ref().and_then(|l| if let etherparse::LinkSlice::Ethernet2(eth) = l { Some(eth.source()) } else { None }); match extract_arp_frame(arp, outer_src_mac, data.len()) { Some(f) => Ok(DecodedFrame::Arp(f)), None => Err(anyhow!("Non-Ethernet/IPv4 ARP frame")) } }`. Note: use the inline closure directly (no `extract_outer_src_mac` helper — that function is not defined anywhere); `eth.source()` returns `[u8; 6]` by value (etherparse 0.20.1, confirmed).
3. **Complete the lax `Err(SliceError::Len(_))` arm** in `decode_packet`: replace the STORY-111 stub with the full lax ARP routing per arp-architecture-delta.md §2.2 (extract `outer_src_mac` from `lax.link` via `if let etherparse::LinkSlice::Ethernet2(eth) = l { Some(eth.source()) } else { None }`; call `extract_arp_frame`; map `Some(f) → Ok(DecodedFrame::Arp(f))`, `None → Err(anyhow!("truncated ARP frame"))`).
4. **Create `src/analyzer/arp.rs`** with `ArpAnalyzer` stub: `new()` (parameterless — `--arp-spoof-threshold`/`--arp-storm-rate` flags are not yet defined; those parameters are added when STORY-114/115 land their CLI flags) and `process_arp(&mut self, frame: &ArpFrame, ts: u32) -> Vec<Finding>` returning `vec![]`.
5. **Add `pub mod arp;`** to `src/analyzer/mod.rs`.
6. **Update `src/main.rs`** to pattern-match on `DecodedFrame`: `Ok(DecodedFrame::Ip(p)) => { existing IP pipeline }` / `Ok(DecodedFrame::Arp(a)) => { arp_analyzer.process_arp(&a, ts); }` (unconditional — `args.arp` flag-gating is added in STORY-113 per BC-2.16.011; `args.arp_spoof_threshold` in STORY-114; `args.arp_storm_rate` in STORY-115). Instantiate `ArpAnalyzer::new()` (parameterless stub).
7. **Write VP-024 Sub-A Kani harnesses** in `#[cfg(kani)] mod kani_proofs` in `src/decoder.rs`: `verify_extract_arp_frame_safety`, `verify_extract_arp_frame_eth_ipv4_correctness`, `verify_extract_arp_frame_none_on_bad_size`.
8. **Run `cargo test --all-targets`**: all tests green; no new failures.
9. **Run `cargo clippy --all-targets -- -D warnings`**: clean.
10. **Write unit tests** for AC-001 through AC-012 (AC-012 added in v1.1: `test_BC_2_16_015_decode_packet_arp_non_eth_ipv4_returns_error`).

## Test Plan

| AC | Test | Type |
|----|------|------|
| AC-001 | `test_BC_2_16_001_extract_arp_frame_request_returns_some` | Unit |
| AC-002 | `test_BC_2_16_001_extract_arp_frame_request_field_copy_fidelity` | Unit |
| AC-003 | `test_BC_2_16_002_extract_arp_frame_reply_returns_some_with_correct_fields` | Unit |
| AC-004 | `test_BC_2_16_001_extract_arp_frame_none_on_hw_addr_size_8`, `test_BC_2_16_001_extract_arp_frame_none_on_proto_addr_size_16` | Unit |
| AC-005 | `test_BC_2_16_001_extract_arp_frame_outer_src_mac_none_passthrough` | Unit |
| AC-006 | `test_BC_2_16_015_decode_packet_routes_arp_to_decoded_frame_arp` | Unit |
| AC-007 | `test_BC_2_16_015_decode_packet_lax_arm_truncated_arp_non_panic` | Unit |
| AC-008 | `test_BC_2_16_015_main_arp_arm_calls_process_arp_stub` | Integration/unit |
| AC-009 | `test_BC_2_16_015_arp_frame_never_reaches_stream_dispatcher` | Unit |
| AC-010 | `cargo check`, `cargo clippy` | Build |
| AC-011 | `verify_extract_arp_frame_safety`, `verify_extract_arp_frame_eth_ipv4_correctness`, `verify_extract_arp_frame_none_on_bad_size` | Kani (F6 gate) |
| AC-012 | `test_BC_2_16_015_decode_packet_arp_non_eth_ipv4_returns_error` | Unit |

## Previous Story Intelligence

STORY-111 (this epic's predecessor) established:
- `DecodedFrame` enum and `ArpFrame` struct are in `src/decoder.rs`.
- `decode_packet` return type is now `Result<DecodedFrame>`.
- The lax `LaxNetSlice::Arp` arm was stubbed as `Err(anyhow!("ARP extraction not yet implemented"))` — this story completes it.
- etherparse 0.20.1 is in use: `Ethernet2Slice::source()` returns `[u8; 6]` by value (no dereference needed).

**Critical from STORY-112 perspective**: the `outer_src_mac` extraction from `slice.link` in the strict arm uses `slice.link.as_ref().and_then(|l| if let etherparse::LinkSlice::Ethernet2(eth) = l { Some(eth.source()) } else { None })`. The lax arm mirrors this pattern on `lax.link`. Confirmed: `eth.source()` returns `[u8; 6]` directly, not `&[u8]` (ADR-008 §Source confirmation 2026-06-12).

## Architecture Compliance Rules

Derived from arp-architecture-delta.md §2.1, §2.2, ADR-008 Decisions 2–3, BC-2.16.015:

1. **`extract_arp_frame` is a pure core function** — no I/O, no global state, no panic for any valid `ArpPacketSlice`. VP-024 Sub-A proves this formally.
2. **Field copy from `ArpPacketSlice` accessors is byte-exact** — confirmed accessor names from etherparse 0.20.1 docs.rs: `sender_hw_addr()`, `target_hw_addr()`, `sender_protocol_addr()`, `target_protocol_addr()`, `operation().0`, `hw_addr_size()`, `proto_addr_size()`, `hw_addr_type()`, `proto_addr_type()`.
3. **Decode is unconditional; analysis stub is wired unconditionally in this story** — `main.rs` MUST call `extract_arp_frame` (via `decode_packet`) regardless of any flag. In STORY-112 the `process_arp` stub is called unconditionally (it is a no-op). The `--arp` flag-gate (`args.arp`) is added in STORY-113 (BC-2.16.011); until then the stub wiring is always active.
4. **No `summarize()` call in this story** — the stub `ArpAnalyzer` has no `summarize()` method. That method is introduced in STORY-113.
5. **VP-024 Sub-A harnesses target `extract_arp_frame` directly** — they do not go through `decode_packet`. This ensures the formal proof is focused on the pure-core extraction function.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `etherparse` | 0.20 (0.20.1) | `ArpPacketSlice` accessor names confirmed; `Ethernet2Slice::source()` returns `[u8; 6]` by value |
| `kani` | via cargo-kani | VP-024 Sub-A harnesses; `kani::any()` for symbolic fields |
| `anyhow` | same as existing | Error strings for ARP decode failures |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/decoder.rs` | Modify | Implement `extract_arp_frame`; complete strict and lax ARP arms in `decode_packet`; add VP-024 Sub-A Kani harnesses in `#[cfg(kani)]` mod |
| `src/analyzer/arp.rs` | Create | `ArpAnalyzer` stub: `new`, `process_arp` (no-op) |
| `src/analyzer/mod.rs` | Modify | Add `pub mod arp;` |
| `src/main.rs` | Modify | `DecodedFrame` pattern-match; `ArpAnalyzer::new()` instantiation (parameterless stub); unconditional `process_arp` call (--arp gate added in STORY-113) |
| `src/decoder.rs` tests module | Modify | Add AC-001..AC-012 unit tests |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~4,000 |
| BC files (3 BCs) | ~7,000 |
| arp-architecture-delta.md §2.1, §2.2, §1 | ~3,000 |
| VP-024 file (Sub-A section) | ~1,500 |
| Existing `src/decoder.rs` + `src/main.rs` | ~4,000 |
| Tool outputs (cargo test, kani) | ~2,000 |
| **Total estimated** | **~21,500** |

Within 20–30% of agent context window.

## Dependency Rationale

- `depends_on: [STORY-111]` — `extract_arp_frame` requires the `ArpFrame` struct and the updated `decode_packet` return type `Result<DecodedFrame>` from STORY-111. Cannot implement the extraction function without the types.
- `blocks: [STORY-113]` — STORY-113 implements the full `ArpAnalyzer` (binding table, GARP detection, D11 malformed emission, D12 mismatch, summarize). It requires the `ArpAnalyzer` stub and `process_arp` signature from this story, plus the working `extract_arp_frame` function that STORY-113's tests will exercise.
