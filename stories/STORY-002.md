---
document_type: story
story_id: "STORY-002"
epic_id: "E-1"
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.001.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.002.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.003.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.004.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.005.md
  - .factory/specs/prd.md
input-hash: "85eb316"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-001]
blocks: [STORY-005]
behavioral_contracts:
  - BC-2.02.001
  - BC-2.02.002
  - BC-2.02.003
  - BC-2.02.004
  - BC-2.02.005
verification_properties: []
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

# STORY-002: Packet Decoding — Ethernet, RAW/IPV4, and IPv6 Link-Layer Paths

## Narrative
- **As a** forensic analyst
- **I want** wirerust to correctly decode packets from Ethernet, RAW, IPV4, and IPV6 link-type captures into structured ParsedPacket values
- **So that** every downstream analyzer receives accurate IP addresses, transport-layer ports and flags, and payload bytes regardless of the capture tool used

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.02.001 | Decode Ethernet-framed IPv4 TCP Packet to ParsedPacket |
| BC-2.02.002 | Decode Ethernet-framed IPv4 UDP Packet with DNS Port Hint |
| BC-2.02.003 | Decode RAW Link-Layer IPv4 TCP Packet via from_ip |
| BC-2.02.004 | DataLink::IPV4 Decodes Identically to DataLink::RAW |
| BC-2.02.005 | Decode RAW IPv6 TCP Packet Surfacing IPv6 Addresses |

## Acceptance Criteria

### AC-001 (traces to BC-2.02.001 postcondition 2)
Given a valid Ethernet II / IPv4 / TCP frame bytes and `datalink = ETHERNET`, `decode_packet` returns `Ok(ParsedPacket)` where `src_ip` and `dst_ip` are `IpAddr::V4` values matching the IPv4 header, `protocol = Protocol::Tcp`, `transport = TransportInfo::Tcp { src_port, dst_port, seq_number, syn, ack, fin, rst }` with correct values, and `payload` contains the TCP segment payload bytes.
- **Test:** `test_BC_2_02_001_ethernet_ipv4_tcp_decode()`

### AC-002 (traces to BC-2.02.001 postcondition 6)
For any successfully decoded frame, `ParsedPacket.packet_len` equals `data.len()` (the total frame byte length including all headers).
- **Test:** `test_BC_2_02_001_packet_len_is_total_frame_length()`

### AC-003 (traces to BC-2.02.002 postcondition 3)
Given an Ethernet / IPv4 / UDP frame with `dst_port = 53`, `decode_packet` returns `Ok(ParsedPacket)` with `protocol = Protocol::Udp`, `transport = TransportInfo::Udp { src_port, dst_port }`, and `app_protocol_hint()` returns `Some("DNS")`.
- **Test:** `test_BC_2_02_002_udp_dns_port_hint()`

### AC-004 (traces to BC-2.02.002 postcondition 6)
Given a UDP frame with `src_port = 53` (DNS response direction), `app_protocol_hint()` also returns `Some("DNS")`.
- **Test:** `test_BC_2_02_002_udp_dns_src_port_hint()`

### AC-005 (traces to BC-2.02.003 postcondition 1)
Given raw IPv4 TCP bytes (no link-layer header) with `datalink = RAW`, `decode_packet` returns `Ok(ParsedPacket)` with the same structure as the Ethernet path for identical IP+TCP content.
- **Test:** `test_BC_2_02_003_raw_ipv4_tcp_decode()`

### AC-006 (traces to BC-2.02.004 postcondition 3)
Calling `decode_packet` with the same IPv4 TCP byte slice using `DataLink::RAW` and then `DataLink::IPV4` produces field-for-field identical `ParsedPacket` values.
- **Test:** `test_BC_2_02_004_raw_and_ipv4_identical()`

### AC-007 (traces to BC-2.02.005 postcondition 2)
Given raw IPv6 TCP bytes with `datalink = RAW`, `decode_packet` returns `Ok(ParsedPacket)` where `src_ip` and `dst_ip` are `IpAddr::V6` values containing the IPv6 addresses from the IP header.
- **Test:** `test_BC_2_02_005_raw_ipv6_tcp_decode()`

### AC-008 (traces to BC-2.02.005 postcondition 4)
For an IPv6 TCP frame, `protocol = Protocol::Tcp` and `transport = TransportInfo::Tcp { ... }` with correct port and flag values.
- **Test:** `test_BC_2_02_005_ipv6_tcp_transport()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| decode_packet | src/decoder.rs:128-172 | pure |
| strict_ip_triple | src/decoder.rs:209-228 | pure |
| lax_parse | src/decoder.rs:184-205 | pure |
| build_parsed | src/decoder.rs:255-302 | pure |
| app_protocol_hint | src/decoder.rs:94-116 | pure |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | TCP SYN packet (syn=true, ack=false) | TransportInfo::Tcp { syn: true, ack: false, fin: false, rst: false } |
| EC-002 | TCP pure ACK (no payload) | payload is empty Vec; Ok returned |
| EC-003 | UDP port = 80 | app_protocol_hint() returns Some("HTTP") |
| EC-004 | UDP port = 9999 (unknown) | app_protocol_hint() returns None |
| EC-005 | DataLink::IPV4 with IPv4 TCP | Identical result to DataLink::RAW |
| EC-006 | IPv6 loopback (::1) | Decoded normally; IpAddr::V6(::1) |
| EC-007 | IPv6 with extension headers | etherparse traverses them; TCP/UDP surfaced |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/decoder.rs | pure | Operates on in-memory byte slices; no I/O; no shared mutable state |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,000 |
| src/decoder.rs (full file) | ~3,000 |
| BC files (5 BCs) | ~4,000 |
| Test files | ~1,000 |
| Tool outputs overhead | ~500 |
| **Total** | **~10,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~5.3%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-008 (test-writer)
2. [ ] Verify all tests fail at Red Gate
3. [ ] Verify `src/decoder.rs` already satisfies all ACs (brownfield confirm pass)
4. [ ] Run `cargo test --all-targets` to confirm green
5. [ ] Confirm `decode_packet` match arm for `DataLink::RAW | DataLink::IPV4 | DataLink::IPV6` is a single arm (BC-2.02.004)
6. [ ] Confirm `build_parsed` extracts IPv4 and IPv6 addresses via separate match arms (BC-2.02.005)
7. [ ] Verify `app_protocol_hint` 7-entry port table is complete (ports 53, 80, 443, 22, 445, 502, 20000)
8. [ ] Write property-based test: `packet_len == data.len()` for all decoded frames

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-001 | PcapSource produces RawPacket with raw bytes and timestamps | Reader is effectful-shell; decoder is pure | Timestamp split logic (us vs ns resolution) must be preserved exactly |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `decode_packet` is a pure function — no I/O, no global state | BC-2.02.001 purity classification | No `use std::fs` or similar in decoder.rs |
| `DataLink::RAW` and `DataLink::IPV4` and `DataLink::IPV6` must be in the same match arm calling `from_ip` | BC-2.02.004 invariant 1 | Code review: single `|`-delimited match arm at decoder.rs:134 |
| `build_parsed` receives `data.len()` directly; `packet_len` is never derived from IP header fields | BC-2.02.001 invariant 1 | Code review of call sites at decoder.rs:142-146 and decoder.rs:161 |
| No application-layer parsing in decoder | BC-2.02.001 invariant 3 | Code review: no HTTP/TLS/DNS parsing in decoder.rs |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| etherparse | (per Cargo.lock) | SlicedPacket, LaxSlicedPacket — link-layer and IP/TCP/UDP header parsing |
| anyhow | (per Cargo.lock) | Error wrapping |
| std::net::IpAddr | stdlib | IPv4/IPv6 address representation |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/decoder.rs | verify/modify | All 5 BCs live here (decode_packet, build_parsed, app_protocol_hint) |
| tests/ | create or modify | Add tests for AC-001 through AC-008 using synthetic frame bytes |
