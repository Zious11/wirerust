---
document_type: story
story_id: "STORY-004"
epic_id: "E-1"
version: "1.2"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.010.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.011.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.012.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.013.md
  - .factory/specs/prd.md
input-hash: "5fbcd60"
traces_to: .factory/specs/prd.md
points: 3
depends_on: [STORY-001]
blocks: [STORY-005]
behavioral_contracts:
  - BC-2.02.010
  - BC-2.02.011
  - BC-2.02.012
  - BC-2.02.013
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 2
target_module: decoder
subsystems: [SS-02]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

# STORY-004: Packet Decoding — ICMP, Protocol::Other, and app_protocol_hint Port Table

## Narrative
- **As a** forensic analyst
- **I want** wirerust to correctly classify ICMP packets, gracefully degrade for unknown IP protocols, and provide a deterministic port-to-service lookup used by downstream dispatchers
- **So that** every packet in the capture has a known protocol classification and the DNS/HTTP/TLS/SSH/SMB/Modbus/DNP3 analyzers are routed correctly via the app_protocol_hint mechanism

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.02.010 | Classify ICMP as Protocol::Icmp with TransportInfo::None |
| BC-2.02.011 | Classify Other IP Protocols as Protocol::Other(byte) |
| BC-2.02.012 | app_protocol_hint Returns Service Strings from Port Number |
| BC-2.02.013 | app_protocol_hint Returns None When TransportInfo is None |

## Acceptance Criteria

### AC-001 (traces to BC-2.02.010 postcondition 1)
An IPv4 ICMP echo-request frame decoded via `decode_packet` produces `ParsedPacket { protocol: Protocol::Icmp, transport: TransportInfo::None, payload: [] }`.
- **Test:** `test_BC_2_02_010_icmpv4_protocol_icmp()`

### AC-002 (traces to BC-2.02.010 invariant 1)
Both ICMPv4 (IP protocol 1) and ICMPv6 (IP protocol 58) produce `Protocol::Icmp` — there is no `Protocol::Icmpv6` variant.
- **Test:** `test_BC_2_02_010_icmpv4_and_icmpv6_both_produce_protocol_icmp()`

### AC-003 (traces to BC-2.02.010 postcondition 4)
`app_protocol_hint()` on an ICMP `ParsedPacket` (with `TransportInfo::None`) returns `None`.
- **Test:** `test_BC_2_02_010_icmp_app_protocol_hint_none()`

### AC-004 (traces to BC-2.02.011 postcondition 1)
A packet with IP protocol 47 (GRE) that produces no `TransportSlice` match results in `ParsedPacket { protocol: Protocol::Other(47), transport: TransportInfo::None, payload: [] }`.
- **Test:** `test_BC_2_02_011_gre_protocol_other()`

### AC-005 (traces to BC-2.02.011 invariant 1)
`Protocol::Other(u8)` preserves the raw IP protocol byte; a GRE packet has `Other(47)`, an ESP packet has `Other(50)`.
- **Test:** `test_BC_2_02_011_protocol_other_preserves_byte()`

### AC-006 (traces to BC-2.02.012 postcondition 3)
`app_protocol_hint()` returns the correct service string for all 7 recognized ports:
- port 53 (src or dst) -> `Some("DNS")`
- port 80 (src or dst) -> `Some("HTTP")`
- port 443 (src or dst) -> `Some("TLS")`
- port 22 (src or dst) -> `Some("SSH")`
- port 445 (src or dst) -> `Some("SMB")`
- port 502 (src or dst) -> `Some("Modbus")`
- port 20000 (src or dst) -> `Some("DNP3")`
- **Test:** `test_BC_2_02_012_app_protocol_hint_all_seven_ports()`

### AC-007 (traces to BC-2.02.012 postcondition 2)
`app_protocol_hint()` returns `None` for any port not in the 7-entry table (e.g., port 9999).
- **Test:** `test_BC_2_02_012_app_protocol_hint_unknown_port_returns_none()`

### AC-008 (traces to BC-2.02.012 postcondition 4)
When both src and dst ports are known but different (e.g., src=80, dst=443), the first matching match arm wins: `Some("HTTP")` is returned (80 arm fires before 443 arm).
- **Test:** `test_BC_2_02_012_app_protocol_hint_match_order()`

### AC-009 (traces to BC-2.02.013 postcondition 1)
`app_protocol_hint()` returns `None` when `transport = TransportInfo::None`. Note: the structural property that the port table is not consulted in this path is a BC-2.02.013 precondition 2 invariant verified by code review, not by this test.
- **Test:** `test_BC_2_02_013_transport_none_returns_none_hint()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| build_parsed — Icmpv4/Icmpv6 match arm | src/decoder.rs:282-284 | pure |
| build_parsed — None/Other match arm | src/decoder.rs:285 | pure |
| app_protocol_hint | src/decoder.rs:94-116 | pure |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | ICMPv4 echo reply (type 0) | Protocol::Icmp, TransportInfo::None |
| EC-002 | ICMPv6 neighbor solicitation (type 135) | Protocol::Icmp, TransportInfo::None |
| EC-003 | GRE (proto 47) | Protocol::Other(47), TransportInfo::None |
| EC-004 | ESP (proto 50) | Protocol::Other(50), TransportInfo::None |
| EC-005 | src=53, dst=9999 | app_protocol_hint() returns Some("DNS") |
| EC-006 | src=9999, dst=53 | app_protocol_hint() returns Some("DNS") |
| EC-007 | src=80, dst=443 | Some("HTTP") (match order: 80 arm first) |
| EC-008 | TransportInfo::None with any IP protocol | app_protocol_hint() always None |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/decoder.rs | pure | In-memory operations; no I/O |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~1,800 |
| src/decoder.rs (relevant sections) | ~2,000 |
| BC files (4 BCs) | ~3,000 |
| Test files | ~700 |
| Tool outputs overhead | ~400 |
| **Total** | **~7,900** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~4.0%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-009 (test-writer)
2. [ ] Verify all tests fail at Red Gate
3. [ ] Verify `src/decoder.rs` already satisfies all ACs (brownfield confirm)
4. [ ] Confirm `Icmpv4 | Icmpv6 => (Protocol::Icmp, TransportInfo::None)` at decoder.rs:282-284
5. [ ] Confirm `None => (Protocol::Other(ip_protocol.0), TransportInfo::None)` at decoder.rs:285
6. [ ] Confirm `app_protocol_hint` match has exactly 7 recognized port pairs in correct order
7. [ ] Confirm `TransportInfo::None => return None` early-return at decoder.rs:98-99
8. [ ] Run `cargo test --all-targets` to confirm green

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-002 | Ethernet/RAW/IPV4/IPv6 decode paths; app_protocol_hint for DNS/HTTP/TLS confirmed | Pure decoder pattern | Port table must be ordered: 80 arm before 443 arm changes which string is returned when both ports are known |
| STORY-003 | SLL + no-panic + no-IP-layer paths confirmed | Lax retry only on Len errors | Three error prefixes exactly |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `Protocol::Icmp` is produced for BOTH ICMPv4 AND ICMPv6; no separate `Protocol::Icmpv6` | BC-2.02.010 invariant 1 | Check Protocol enum definition; confirm single Icmp variant |
| `Protocol::Other(u8)` preserves raw IP protocol byte from IpTriple | BC-2.02.011 invariant 1 | Code review of None arm at decoder.rs:285 |
| `app_protocol_hint` port table has exactly 7 entries; no 8th entry silently added | BC-2.02.012 postcondition 3 | Code review of match arms at decoder.rs:94-116 |
| `TransportInfo::None` arm in `app_protocol_hint` is the FIRST arm (early return) | BC-2.02.013 invariant 1 | Code review: None arm at decoder.rs:98-99 precedes port table |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| etherparse | (per Cargo.lock) | TransportSlice::Icmpv4, TransportSlice::Icmpv6 variants |
| anyhow | (per Cargo.lock) | Error paths |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/decoder.rs | verify/modify | ICMP/Other classification (build_parsed); port table (app_protocol_hint) |
| tests/ | create or modify | ICMP frame bytes, GRE IP packet, port table exhaustive test |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.2 | 2026-05-22 | story-writer | Wave 2 Ph3 adversarial fixes: AC-009 testable claim narrowed to TransportInfo::None returns None; port-table non-consultation relegated to BC-2.02.013 PC2 structural invariant (code review only); Architecture Mapping row labels updated from "(Icmpv4/Icmpv6 arm)" to "— Icmpv4/Icmpv6 match arm" to accurately describe cited line ranges |
| 1.1 | 2026-05-21 | story-writer | Initial story decomposition |
