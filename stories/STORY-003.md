---
document_type: story
story_id: "STORY-003"
epic_id: "E-1"
version: "1.5"
status: completed
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.006.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.007.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.008.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.009.md
  - .factory/specs/prd.md
input-hash: "f2f8611"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-001]
blocks: [STORY-005]
behavioral_contracts:
  - BC-2.02.006
  - BC-2.02.007
  - BC-2.02.008
  - BC-2.02.009
verification_properties: [VP-008]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 2
target_module: decoder
subsystems: [SS-02]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

# STORY-003: Packet Decoding — Linux SLL, No-Panic Safety, and Non-IP Frame Rejection

## Narrative
- **As a** forensic analyst
- **I want** wirerust to handle Linux cooked-capture (SLL) frames correctly, never panic on any malformed or attacker-crafted byte input, and clearly reject non-IP frames
- **So that** I can safely process captures from `tcpdump -i any`, corrupted files, and adversarial inputs without the tool crashing or silently dropping frames

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.02.006 | Decode Linux SLL (Cooked) TCP Packets |
| BC-2.02.007 | Reject Malformed Input Bytes with anyhow Error (No Panic) |
| BC-2.02.008 | Reject Unsupported Link Types in decode_packet |
| BC-2.02.009 | Surface No IP Layer Found Error for Non-IP Frames |

## Acceptance Criteria

### AC-001 (traces to BC-2.02.006 postcondition 1)
Given a valid Linux SLL (16-byte cooked header + IPv4 TCP payload) frame bytes with `datalink = LINUX_SLL`, `decode_packet` returns `Ok(ParsedPacket)` with correct IP addresses, `protocol = Protocol::Tcp`, and `transport = TransportInfo::Tcp`.
- **Test:** `test_BC_2_02_006_linux_sll_ipv4_tcp()`

### AC-002 (traces to BC-2.02.006 postcondition 1)
Given a Linux SLL frame containing an IPv6 TCP payload, `decode_packet` returns `Ok(ParsedPacket)` with `src_ip` and `dst_ip` as `IpAddr::V6`.
- **Test:** `test_BC_2_02_006_linux_sll_ipv6_tcp()`

### AC-003 (traces to BC-2.02.006 invariant 2)
When a snaplen-truncated SLL frame causes the strict parse to fail with a length error (SliceError::Len), the lax fallback path strips the 16-byte SLL header manually and invokes `LaxSlicedPacket::from_ether_type` to recover the IP layer.
- **Test:** `test_BC_2_02_006_linux_sll_snaplen_truncated_lax_recovery()`

### AC-004 (traces to BC-2.02.006 invariant 3)
A LINUX_SLL frame shorter than 16 bytes fails the strict parse with a non-Len error and is immediately rejected with `Err`; the lax fallback is NOT invoked.
- **Test:** `test_BC_2_02_006_linux_sll_sub_16_bytes_rejected()`

### AC-005 (traces to BC-2.02.007 postcondition 1)
Calling `decode_packet` with random byte inputs (e.g., 20 random bytes with any supported link type) returns `Err` with message containing "Parse error:" and does NOT panic.
- **Test:** `test_BC_2_02_007_random_bytes_no_panic()`

### AC-006 (traces to BC-2.02.007 postcondition 1)
Calling `decode_packet` with an empty slice (`data = &[]`) returns `Err` (no panic).
- **Test:** `test_BC_2_02_007_empty_slice_no_panic()`

### AC-007 (traces to BC-2.02.007 invariant 1)
Each of the three known error prefixes ("Unsupported link type:", "No IP layer found", "Parse error:") is produced by a representative input, and no representative input's error message contains a foreign prefix. This is a representative spot-check, not an exhaustive proof — universal exhaustiveness of the three-prefix set is a BC-2.02.007 invariant verified by code review, not by this test.
- **Test:** `test_BC_2_02_007_error_prefix_representative_check()`

### AC-008 (traces to BC-2.02.008 postcondition 1)
Calling `decode_packet` with a `DataLink` variant outside the whitelist (e.g., IEEE802_11) returns `Err` containing "Unsupported link type:" immediately, without reading any bytes from `data`.
- **Test:** `test_BC_2_02_008_unsupported_link_type_error()`

### AC-009 (traces to BC-2.02.009 postcondition 1)
Passing a valid Ethernet ARP frame (EtherType 0x0806, no IP layer) with `datalink = ETHERNET` to `decode_packet` returns `Err` containing "No IP layer found".
- **Test:** `test_BC_2_02_009_non_ip_frame_rejected()`

### AC-010 (traces to BC-2.02.009 invariant 1)
The "No IP layer found" error fires on the strict-parse path (decoder.rs:150) when `slice.net == None` — verified by a complete SLL ARP frame whose strict parse succeeds but yields no IP layer. The corresponding lax-path arm (decoder.rs:163) exists only for Rust match-exhaustiveness and is structurally unreachable against etherparse 0.16: when the strict parse fails with `SliceError::Len` the lax re-parse always recovers the IP header (`net = Some`); when the EtherType is non-IP the strict path returns `Ok(net=None)` and the lax fallback is never invoked. Therefore the lax arm is not separately test-exercised.
- **Test:** `test_BC_2_02_009_strict_path_sll_arp_no_ip()`

### AC-011 (traces to BC-2.02.007 postcondition 1; implements VP-008)
A cargo-fuzz harness targeting `decode_packet` MUST exist at `fuzz/fuzz_targets/fuzz_decode_packet.rs` and MUST compile without errors (`cargo +nightly fuzz build fuzz_decode_packet` succeeds). The harness passes arbitrary byte slices to `decode_packet` with both the supported (whitelisted) `DataLink` variants AND representative unsupported `DataLink` variants, asserting that no call panics on either path. VP-008 ("decode_packet Never Panics on Arbitrary Input") is a P0 verification property; this harness is its mandatory implementation vehicle.
- **Test:** `test_VP_008_fuzz_harness_exists()` (compile-check: verify `fuzz/fuzz_targets/fuzz_decode_packet.rs` is present and non-empty)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| decode_packet (strict path) | src/decoder.rs:128-172 | pure |
| lax_parse (SLL lax fallback) | src/decoder.rs:176-206 | pure |
| SLL_HEADER_LEN constant | src/decoder.rs:119-121 | pure |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | SLL frame with IPv4 TCP | Decoded correctly via SlicedPacket::from_linux_sll |
| EC-002 | SLL frame snaplen-truncated (Len error) | Lax path invoked; 16-byte header stripped; from_ether_type called |
| EC-003 | SLL frame < 16 bytes | Non-Len error; immediate Err; NO lax retry |
| EC-004 | Empty data slice | Err("Parse error: ...") |
| EC-005 | DataLink::IEEE802_11 | Err("Unsupported link type: IEEE802_11") |
| EC-006 | ARP Ethernet frame | Err("No IP layer found") |
| EC-007 | Custom EtherType 0x9000 | Err("No IP layer found") |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/decoder.rs | pure | In-memory byte slice operations only; no I/O |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,000 |
| src/decoder.rs (focus on lax_parse and error paths) | ~2,500 |
| BC files (4 BCs) | ~3,500 |
| Test files | ~800 |
| Tool outputs overhead | ~500 |
| **Total** | **~9,300** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~4.7%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-011 (test-writer)
2. [ ] Verify all tests fail at Red Gate
3. [ ] Verify `src/decoder.rs` already satisfies all ACs (brownfield confirm)
4. [ ] Confirm `SLL_HEADER_LEN = 16` constant at decoder.rs:119-121
5. [ ] Confirm lax retry logic is triggered only for `SliceError::Len` (not structural errors)
6. [ ] Confirm `decode_packet` has no `unwrap()` or `panic!` calls
7. [ ] Run `cargo test --all-targets` to confirm green
8. [ ] MANDATORY: Create cargo-fuzz harness at `fuzz/fuzz_targets/fuzz_decode_packet.rs` implementing VP-008; harness must pass arbitrary bytes to `decode_packet` with each supported `DataLink` and assert no panic. Run `cargo +nightly fuzz build fuzz_decode_packet` to confirm compilation. This is a P0 obligation — do not skip.

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-001 | PcapSource reads RawPacket with raw bytes | Reader is effectful; decoder is pure | N/A |
| STORY-002 | Ethernet/RAW/IPV4/IPv6 decode paths confirmed pure | `app_protocol_hint` uses 7-entry port table | DataLink::RAW | IPV4 | IPV6 share a single match arm |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Lax retry fires ONLY for `SliceError::Len`, not structural errors | BC-2.02.006 invariant 3 | Code review: match arm on `EthernetSliceError::Len` variant |
| SLL header is exactly 16 bytes; `SLL_HEADER_LEN` is the only constant used | BC-2.02.006 invariant 1 | No magic number 16 outside the constant |
| Three error prefixes only: "Unsupported link type:", "No IP layer found", "Parse error:" | BC-2.02.007 invariant 1 | Grep decoder.rs for `anyhow!` calls to audit prefixes |
| No `unwrap()` or `panic!` in `decode_packet` or `lax_parse` | BC-2.02.007 invariant 2 | Code review |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| etherparse | (per Cargo.lock) | SlicedPacket::from_linux_sll (strict), LaxSlicedPacket::from_ether_type (lax fallback) |
| anyhow | (per Cargo.lock) | Error construction with anyhow! macro |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/decoder.rs | verify/modify | lax_parse (SLL fallback), error paths — all 4 BCs live here |
| tests/ | create or modify | Synthetic SLL frame bytes, malformed inputs, ARP frames |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.5 | 2026-05-22 | story-writer | AC-010 corrected: test renamed to test_BC_2_02_009_strict_path_sll_arp_no_ip; prose updated to cover strict-path only (decoder.rs:150); lax-path arm (decoder.rs:163) documented as structurally unreachable per etherparse 0.16 analysis; no claim of lax-path coverage |
| 1.4 | 2026-05-22 | story-writer | Wave 2 Ph3 pass-3 adversarial fix: AC-011 description updated to state the harness exercises both supported (whitelisted) AND representative unsupported DataLink variants, asserting no panic on either path |
| 1.3 | 2026-05-22 | story-writer | Wave 2 Ph3 pass-2 adversarial fix: corrected lax_parse Architecture Mapping line range from 184-205 to 176-206 (full function body per decoder.rs); aligns with BC-2.02.006 v1.3 anchor |
| 1.2 | 2026-05-22 | story-writer | Wave 2 Ph3 adversarial fix: AC-007 reworded from exhaustive universal claim to representative spot-check; test renamed to test_BC_2_02_007_error_prefix_representative_check; code-review note added for invariant exhaustiveness |
| 1.1 | 2026-05-21 | story-writer | Initial story decomposition |
