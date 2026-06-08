---
document_type: story
story_id: "STORY-005"
epic_id: "E-1"
version: "1.8"
status: completed
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.014.md
  - .factory/specs/behavioral-contracts/ss-02/BC-2.02.015.md
  - .factory/specs/prd.md
input-hash: "4632242"
traces_to: .factory/specs/prd.md
points: 3
depends_on: [STORY-002, STORY-003, STORY-004]
blocks: [STORY-011, STORY-066]
behavioral_contracts:
  - BC-2.02.014
  - BC-2.02.015
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 3
target_module: decoder
subsystems: [SS-02]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
nfr:
  - NFR-PERF-001
  - NFR-PORT-005
implementation_strategy: brownfield-formalization
---

# STORY-005: Packet Decoding — packet_len Semantics and TCP Flag/Sequence Extraction

## Narrative
- **As a** forensic analyst and downstream TCP reassembly engine consumer
- **I want** every decoded packet to carry the total captured frame byte count as `packet_len` and every TCP packet to have accurate flags (syn, ack, fin, rst) and sequence number
- **So that** the summary's total byte counter is accurate and the TCP reassembly state machine has the correct inputs to sequence and reorder segments

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.02.014 | packet_len is Set to Total Frame Length, Not Just Payload Length |
| BC-2.02.015 | Extract TCP Control Flags and Sequence Number into TransportInfo::Tcp |

## Acceptance Criteria

### AC-001 (traces to BC-2.02.014 postcondition 1, invariant 1)
For any successfully decoded frame, `ParsedPacket.packet_len` equals `data.len()` — the total byte length of the raw frame slice passed to `decode_packet`, regardless of header sizes or payload content. This AC covers both a non-trivial payload (data.len() != payload.len()) and a variable header size (a frame with IPv4 options, header > 54 bytes), so a buggy header-dependent `packet_len` formula that violates invariant 1 ("packet_len is always the full frame length, never a partial or payload-only length") is caught.
- **Test:** `test_BC_2_02_014_packet_len_equals_data_len()`

### AC-002 (traces to BC-2.02.014 postcondition 2)
`packet_len` is set to the full frame length (`data.len()`) on BOTH the strict parse path (decoder.rs:142-146 (the build_parsed call; the data.len() argument is on line 145)) and the lax parse path (decoder.rs:161). Neither path uses IP header `total_length` for this field. Invariant 1 (never payload-only/partial length) is verified by AC-001's variable-header and non-trivial-payload cases; AC-002 verifies only that both code paths set the field via the same data.len() mechanism.
- **Test:** `test_BC_2_02_014_packet_len_set_on_both_strict_and_lax_paths()`

### AC-003 (traces to BC-2.02.014 invariant 2)
For a snaplen-truncated packet where `data.len() < on-wire frame length`, `packet_len` equals the captured (truncated) length. No `on_wire_len` field exists.
- **Test:** `test_BC_2_02_014_snaplen_truncated_packet_len()`

### AC-004 (traces to BC-2.02.015 postcondition 4)
For a TCP SYN packet, `TransportInfo::Tcp.syn = true` and `TransportInfo::Tcp.ack = false`.
- **Test:** `test_BC_2_02_015_tcp_syn_flags()`

### AC-005 (traces to BC-2.02.015 postcondition 5)
For a TCP SYN-ACK packet, `syn = true` and `ack = true`.
- **Test:** `test_BC_2_02_015_tcp_syn_ack_flags()`

### AC-006 (traces to BC-2.02.015 postconditions 6, 7)
For a TCP FIN-ACK packet, `fin = true` and `ack = true`; for a TCP RST packet, `rst = true`.
- **Test:** `test_BC_2_02_015_tcp_rst_and_fin_ack_flags()`

### AC-007 (traces to BC-2.02.015 postcondition 3)
`TransportInfo::Tcp.seq_number` equals the 32-bit sequence number from the TCP header.
- **Test:** `test_BC_2_02_015_tcp_seq_number_extracted()`

### AC-008 (traces to BC-2.02.015 postcondition 8)
`ParsedPacket.payload` contains the TCP segment payload bytes (bytes after the TCP header); for a pure ACK with no data, payload is an empty `Vec`.
- **Test:** `test_BC_2_02_015_tcp_payload_bytes()`

### AC-009 (traces to BC-2.02.015 invariant 3)
PSH and URG are NOT present as fields of `TransportInfo::Tcp`. Adding them would require a struct change — this is a deliberate scope constraint.
- **Test:** `test_BC_2_02_015_psh_urg_not_in_transport_info()` (verify TransportInfo::Tcp struct fields)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| build_parsed (packet_len) | src/decoder.rs:255-302 | pure |
| build_parsed (Tcp arm, flags/seq) | src/decoder.rs:263-274 | pure |
| build_parsed (Tcp payload extraction) | src/decoder.rs:288-292 | pure |
| TransportInfo::Tcp struct | src/decoder.rs | pure |

## Edge Cases

| ID | Scenario | Expected Behavior | BC Edge Case |
|----|----------|-------------------|-------------|
| EC-001 | 1500-byte Ethernet frame | packet_len == 1500 | BC-2.02.014 EC-001 |
| EC-002 | 54-byte TCP ACK (no payload) | packet_len == 54; payload is empty Vec | BC-2.02.014 EC-004 |
| EC-003 | Snaplen-truncated at 100 bytes | packet_len == 100 (not 1500) | BC-2.02.014 EC-003 |
| EC-004 | seq_number = 0xFFFFFFFF (max u32) | seq_number == 4294967295; no overflow | BC-2.02.015 EC-005 |
| EC-005 | All four flags simultaneously set | syn=true, ack=true, fin=true, rst=true | — |
| EC-006 | No flags set (data segment) | syn=false, ack=false, fin=false, rst=false | BC-2.02.015 EC-007 |
| EC-007 | 60-byte minimum Ethernet frame (small IP packet + Ethernet padding to reach the 60-byte minimum) | packet_len == 60 (== data.len()); Ethernet padding is NOT counted into the TCP payload — test: `test_BC_2_02_014_ec007_60_byte_padded_frame` | BC-2.02.014 EC-002 |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/decoder.rs | pure | In-memory byte slice operations only |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~1,800 |
| src/decoder.rs (build_parsed section) | ~1,500 |
| BC files (2 BCs) | ~2,000 |
| Test files | ~600 |
| Tool outputs overhead | ~300 |
| **Total** | **~6,200** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~3.1%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-009 (test-writer)
2. [ ] Verify all tests fail at Red Gate
3. [ ] Verify `src/decoder.rs` already satisfies all ACs (brownfield confirm)
4. [ ] Confirm `build_parsed` call sites at decoder.rs:142-146 and decoder.rs:161 both pass `data.len()` as `packet_len`
5. [ ] Confirm `TransportSlice::Tcp` arm at decoder.rs:263-274 extracts syn/ack/fin/rst and seq_number via etherparse API
6. [ ] Confirm `TransportInfo::Tcp` struct lacks psh and urg fields
7. [ ] Run `cargo test --all-targets` to confirm green
8. [ ] Confirm the 16 BC tests in `tests/bc_2_02_story005_tests.rs` cover all AC/EC clauses; property-based verification of `packet_len` is out of scope for this story — no VP assigned

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-002 | Ethernet/RAW/IPV4/IPv6 decode paths; `packet_len` described as `data.len()` | Packet length is always total frame length | None |
| STORY-003 | Lax parse path also uses `data.len()` for packet_len | Both strict and lax set packet_len from data.len() | Lax path decoder.rs:161 call site confirmed |
| STORY-004 | ICMP and Protocol::Other also route through build_parsed | build_parsed always sets packet_len | N/A |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `packet_len` is ALWAYS `data.len()`; never derived from IP or TCP header length fields | BC-2.02.014 invariant 1 | Code review: both call sites to `build_parsed` pass `data.len()` as third argument |
| `TransportInfo::Tcp` struct has exactly these fields: src_port, dst_port, seq_number, syn, ack, fin, rst | BC-2.02.015 invariant 3 | Struct definition review; no psh/urg fields present |
| seq_number extraction uses `tcp.to_header().sequence_number` (etherparse API) | BC-2.02.015 invariant 1 | Code review of decoder.rs:263-274 |
| TCP payload extraction uses `tcp.payload().to_vec()` in the separate payload match block | BC-2.02.015 postcondition 8 | Code review of decoder.rs:288-292 |
| packet_len is set on BOTH strict and lax paths | BC-2.02.014 postcondition 2 | Code review: decoder.rs:142-146 (the build_parsed call; the data.len() argument is on line 145) (strict) and decoder.rs:161 (lax) |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| etherparse | (per Cargo.lock) | `TcpHeaderSlice::to_header().sequence_number`, `flags()` accessors |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/decoder.rs | verify/modify | `build_parsed`: packet_len (both paths), Tcp flag/seq extraction |
| src/decoder.rs | verify | `TransportInfo::Tcp` struct definition (approx lines 65-80) |
| tests/ | create or modify | TCP flag combinations, sequence number, payload length tests |

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.7 | 2026-05-22 | story-writer | Wave 3 wave-level adversarial fix F-2: status advanced draft → completed — STORY-005 delivered via PR #114, merge f0b5007 |
| 1.6 | 2026-05-22 | story-writer | Wave 3 Ph3 pass-4 adversarial fixes: F-2 payload-extraction anchor added (decoder.rs:288-292), F-3 AC-002 over-claim removed ("or TCP segment length"); full AC trace-annotation self-audit — all 9 ACs confirmed clean |
| 1.5 | 2026-05-22 | story-writer | Wave 3 Ph3 pass-2 adversarial fixes: F-1, F-3, F-4, F-5, F-6, F-7 |
| 1.4 | 2026-05-22 | story-writer | Wave 3 Ph3 pass-1 adversarial fixes: F-1, F-3, F-6, F-4, F-7, N-1 |
