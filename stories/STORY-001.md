---
document_type: story
story_id: "STORY-001"
epic_id: "E-1"
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.001.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.002.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.003.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.004.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.005.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.006.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.007.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.008.md
  - .factory/specs/prd.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
points: 5
depends_on: []
blocks: [STORY-002, STORY-003, STORY-004]
behavioral_contracts:
  - BC-2.01.001
  - BC-2.01.002
  - BC-2.01.003
  - BC-2.01.004
  - BC-2.01.005
  - BC-2.01.006
  - BC-2.01.007
  - BC-2.01.008
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 1
target_module: reader
subsystems: [SS-01]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

# STORY-001: PCAP File Ingestion — Link-Type Gating, Eager Packet Load, and Error Surfaces

## Narrative
- **As a** forensic analyst
- **I want to** open any supported pcap file and have wirerust either load all packets or return a clear error
- **So that** I immediately know whether my capture is usable and can trust that every packet is in memory before analysis begins

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.01.001 | Accept Supported Link Types and Reject Unsupported at File Open |
| BC-2.01.002 | Read All Packets from PCAP as Vec<RawPacket> Preserving Timestamps |
| BC-2.01.003 | Accept PCAP with Zero Packets Without Error |
| BC-2.01.004 | Reject pcapng-Format Input at Reader Level |
| BC-2.01.005 | Convert PCAP Record Timestamp to (timestamp_secs: u32, timestamp_usecs: u32) |
| BC-2.01.006 | Surface PCAP Header Parse Errors with Anyhow Context |
| BC-2.01.007 | Surface Per-Packet Read Errors with Anyhow Context |
| BC-2.01.008 | from_file Opens via BufReader and Delegates to from_pcap_reader |

## Acceptance Criteria

### AC-001 (traces to BC-2.01.001 postcondition 1)
Calling `PcapSource::from_file` on a pcap with any of the five accepted link types (ETHERNET=1, RAW=101, LINUX_SLL=113, IPV4=228, IPV6=229) returns `Ok(PcapSource)` with `datalink` set to the accepted variant.
- **Test:** `test_BC_2_01_001_accepts_all_five_link_types()`

### AC-002 (traces to BC-2.01.001 postcondition 2)
Calling `PcapSource::from_file` on a pcap whose link type is not in the accepted set (e.g., IEEE 802.11 = 105) returns `Err` containing the string "Unsupported pcap link type" without panicking.
- **Test:** `test_BC_2_01_001_rejects_unsupported_link_type()`

### AC-003 (traces to BC-2.01.002 postcondition 1)
For a pcap with N packet records, the returned `PcapSource.packets` contains exactly N `RawPacket` entries in file order.
- **Test:** `test_BC_2_01_002_packet_count_and_order()`

### AC-004 (traces to BC-2.01.002 postcondition 2)
Each `RawPacket` in the returned `Vec` has `timestamp_secs` equal to the pcap record's `ts_sec` field and `timestamp_usecs` equal to `ts_frac` (for microsecond-resolution files) or `ts_frac / 1_000` (for nanosecond-resolution files).
- **Test:** `test_BC_2_01_002_timestamp_preserved_microsecond()`
- **Test:** `test_BC_2_01_002_timestamp_preserved_nanosecond()`

### AC-005 (traces to BC-2.01.003 postcondition 1)
A pcap containing only the global header (zero packet records) returns `Ok(PcapSource { packets: vec![], datalink })` without error or panic.
- **Test:** `test_BC_2_01_003_zero_packet_pcap()`

### AC-006 (traces to BC-2.01.004 postcondition 1)
Passing a pcapng-format file to `from_file` returns `Err` with message containing "Failed to parse pcap header"; no packets are returned.
- **Test:** `test_BC_2_01_004_rejects_pcapng()`

### AC-007 (traces to BC-2.01.005 postcondition 2)
For a nanosecond-resolution pcap record with `ts_frac = 500_000`, the resulting `RawPacket.timestamp_usecs` equals 500 (sub-microsecond precision is discarded by integer division).
- **Test:** `test_BC_2_01_005_nanosecond_resolution_conversion()`

### AC-008 (traces to BC-2.01.006 postcondition 1)
Passing a zero-byte file or a file with invalid pcap magic bytes to `from_file` returns `Err` whose error chain contains "Failed to parse pcap header".
- **Test:** `test_BC_2_01_006_corrupt_header_error_message()`

### AC-009 (traces to BC-2.01.007 invariant 1)
When a pcap has a valid header but a truncated packet record mid-stream, `from_pcap_reader` returns `Err` with context "Failed to read packet" and does NOT return a partial `Vec`.
- **Test:** `test_BC_2_01_007_truncated_packet_error()`

### AC-010 (traces to BC-2.01.008 postcondition 2)
Calling `from_file` on a path that does not exist returns `Err` with context "Failed to open" and the path in the message.
- **Test:** `test_BC_2_01_008_file_not_found_error()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| PcapSource | src/reader.rs | effectful-shell (file I/O) |
| PcapSource::from_file | src/reader.rs:85-90 | effectful-shell |
| PcapSource::from_pcap_reader | src/reader.rs:69-79 | effectful-shell |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | DataLink = IEEE 802.11 (numeric 105) | Err("Unsupported pcap link type: ...") |
| EC-002 | Zero-packet pcap (header only, 24 bytes) | Ok(PcapSource { packets: [] }) |
| EC-003 | pcapng file (different magic number) | Err("Failed to parse pcap header") |
| EC-004 | Truncated packet record mid-stream | Err("Failed to read packet") |
| EC-005 | Packet ts_sec = u32::MAX | Stored as-is; no wrapping or error |
| EC-006 | Nanosecond-resolution ts_frac = 500_000 | timestamp_usecs = 500 (integer division) |
| EC-007 | Non-existent file path | Err("Failed to open {path}") |
| EC-008 | RAW and IPV4 link types | Both accepted; identical behavior |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reader.rs | effectful-shell | File I/O via BufReader; no mutable shared state |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,000 |
| src/reader.rs (full file) | ~1,500 |
| BC files (8 BCs) | ~6,000 |
| Test files (existing tests) | ~1,000 |
| Tool outputs overhead | ~500 |
| **Total** | **~11,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~5.5%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-010 (test-writer)
2. [ ] Verify all tests fail at Red Gate (no implementation bypasses the gate)
3. [ ] Verify `src/reader.rs` already satisfies all ACs (brownfield confirm pass)
4. [ ] Run `cargo test --all-targets` to confirm green
5. [ ] Confirm `src/reader.rs:50-60` link-type whitelist contains exactly 5 variants
6. [ ] Confirm `src/reader.rs:69-79` timestamp extraction matches BC-2.01.005
7. [ ] Confirm error message strings match BC-2.01.006/007/008 exactly
8. [ ] Write property-based test: any DataLink not in whitelist produces Err (no panic)
9. [ ] Update STORY-INDEX.md status to completed

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A -- first story in E-1 | N/A | N/A | N/A |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `from_file` is a thin wrapper delegating to `from_pcap_reader`; no logic duplication | BC-2.01.008 / src/reader.rs:85-90 | Code review: `from_file` body must contain only File::open + BufReader + from_pcap_reader call |
| No panic in any error path; use `anyhow::Error` returns | BC-2.01.001 invariant 2, BC-2.01.007 | `cargo test` + proptest fuzzing |
| Link-type whitelist contains exactly 5 variants (ETHERNET, RAW, IPV4, IPV6, LINUX_SLL) | BC-2.01.001 invariant 1 | Code review of match arms at reader.rs:51-55 |
| Eager load only — no streaming API | BC-2.01.002 invariant 1 | API surface review: no Iterator or Stream return type |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| pcap_file | (per Cargo.lock) | Pcap global header and packet record parsing |
| anyhow | (per Cargo.lock) | Error wrapping with context strings |
| std::io::BufReader | stdlib | Buffered file reading |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/reader.rs | verify/modify | Core ingestion module; all 8 BCs live here |
| tests/ (integration tests) | create or modify | Add/verify tests for AC-001 through AC-010 |
| tests/fixtures/ | verify | Confirm existing pcap fixtures cover link-type variants |
