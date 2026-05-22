# STORY-001 Demo Evidence Report

**Story:** STORY-001 — PCAP File Ingestion: Link-Type Gating, Eager Packet Load, and Error Surfaces
**Story version:** 1.3
**Branch:** feature/story-001-pcap-ingestion
**Worktree:** /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-001
**Test file:** tests/bc_2_01_story001_tests.rs
**Recording tool:** VHS 0.11.0
**Date:** 2026-05-21
**Result:** 20/20 tests pass. All 10 ACs demonstrated.

---

## Full Test Suite Run

```
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
 Running tests/bc_2_01_story001_tests.rs (target/debug/deps/bc_2_01_story001_tests-74d5181ba3384d64)

running 20 tests
test test_BC_2_01_002_timestamp_preserved_microsecond ... ok
test test_BC_2_01_002_timestamp_preserved_nanosecond ... ok
test test_BC_2_01_003_zero_packet_pcap ... ok
test test_BC_2_01_005_nanosecond_resolution_conversion ... ok
test test_BC_2_01_002_packet_count_and_order ... ok
test test_BC_2_01_004_rejects_pcapng ... ok
test test_BC_2_01_008_file_not_found_error ... ok
test test_BC_2_01_008_permission_denied_error ... ok
test test_BC_2_01_001_rejects_unsupported_link_type ... ok
test test_BC_2_01_001_accepts_all_five_link_types ... ok
test test_BC_2_01_005_zero_timestamp_nanosecond ... ok
test test_BC_2_01_006_corrupt_header_error_message ... ok
test test_BC_2_01_003_zero_packet_linux_sll ... ok
test test_BC_2_01_007_truncated_packet_error ... ok
test test_BC_2_01_001_raw_and_ipv4_both_accepted ... ok
test test_BC_2_01_006_truncated_header_error_message ... ok
test test_BC_2_01_005_ts_sec_u32_max_stored_as_is ... ok
test test_BC_2_01_008_from_file_delegates_to_from_pcap_reader ... ok
test test_BC_2_01_001_proptest_whitelist_link_type_accepted ... ok
test test_BC_2_01_001_proptest_non_whitelist_link_type_rejected ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

---

## AC-to-Test-to-Evidence Mapping

| AC | BC | Test(s) | Path Type | GIF | WEBM | Tape | Status |
|----|----|---------|-----------|-----|------|------|--------|
| AC-001 | BC-2.01.001 PC1 | `test_BC_2_01_001_accepts_all_five_link_types` | success | AC-001-accepts-all-five-link-types.gif | AC-001-accepts-all-five-link-types.webm | AC-001-accepts-all-five-link-types.tape | PASS |
| AC-002 | BC-2.01.001 PC2 | `test_BC_2_01_001_rejects_unsupported_link_type` | error | AC-002-rejects-unsupported-link-type.gif | AC-002-rejects-unsupported-link-type.webm | AC-002-rejects-unsupported-link-type.tape | PASS |
| AC-003 | BC-2.01.002 PC1 | `test_BC_2_01_002_packet_count_and_order` | success | AC-003-packet-count-and-order.gif | AC-003-packet-count-and-order.webm | AC-003-packet-count-and-order.tape | PASS |
| AC-004 | BC-2.01.002 PC2 + BC-2.01.005 PC1/2 | `test_BC_2_01_002_timestamp_preserved_microsecond`, `test_BC_2_01_002_timestamp_preserved_nanosecond` | success | AC-004-timestamp-preserved.gif | AC-004-timestamp-preserved.webm | AC-004-timestamp-preserved.tape | PASS |
| AC-005 | BC-2.01.003 PC1 | `test_BC_2_01_003_zero_packet_pcap` | success (empty Vec) | AC-005-zero-packet-pcap.gif | AC-005-zero-packet-pcap.webm | AC-005-zero-packet-pcap.tape | PASS |
| AC-006 | BC-2.01.004 PC1 | `test_BC_2_01_004_rejects_pcapng` | error | AC-006-rejects-pcapng.gif | AC-006-rejects-pcapng.webm | AC-006-rejects-pcapng.tape | PASS |
| AC-007 | BC-2.01.005 PC2 | `test_BC_2_01_005_nanosecond_resolution_conversion` | success | AC-007-nanosecond-conversion.gif | AC-007-nanosecond-conversion.webm | AC-007-nanosecond-conversion.tape | PASS |
| AC-008 | BC-2.01.006 PC1 | `test_BC_2_01_006_corrupt_header_error_message` | error | AC-008-corrupt-header-error.gif | AC-008-corrupt-header-error.webm | AC-008-corrupt-header-error.tape | PASS |
| AC-009 | BC-2.01.007 PC1 | `test_BC_2_01_007_truncated_packet_error` | error | AC-009-truncated-packet-error.gif | AC-009-truncated-packet-error.webm | AC-009-truncated-packet-error.tape | PASS |
| AC-010 | BC-2.01.008 PC2 | `test_BC_2_01_008_file_not_found_error` | error | AC-010-file-not-found-error.gif | AC-010-file-not-found-error.webm | AC-010-file-not-found-error.tape | PASS |

---

## Coverage Notes

### AC-001 (success path)
Demo invokes `test_BC_2_01_001_accepts_all_five_link_types`, which loops over ETHERNET=1, RAW=101, LINUX_SLL=113, IPV4=228, IPV6=229 and asserts `Ok(PcapSource)` with `datalink` set to the expected variant. All five accepted.

### AC-002 (error path)
Demo invokes `test_BC_2_01_001_rejects_unsupported_link_type`, which sends IEEE 802.11 (numeric 105). Asserts `Err` containing both `"Unsupported pcap link type"` and `"IEEE802_11"` (Debug repr). The proptest `test_BC_2_01_001_proptest_non_whitelist_link_type_rejected` covers the broader universe (1000 non-whitelist u32 values); also recorded in the full suite run.

### AC-003 (success path)
Demo shows 3-packet pcap with distinct payloads. Asserts `packets.len() == 3` and that each `data` field equals the byte sequence written at that position.

### AC-004 (success paths — microsecond + nanosecond)
Single demo runs both `timestamp_preserved_microsecond` (ts_frac=500 → usecs=500) and `timestamp_preserved_nanosecond` (ts_frac=900_000 → usecs=900). Both pass; both branches of the `match ts_resolution` block in `reader.rs:71-74` are exercised.

### AC-005 (success path — empty Vec)
24-byte header-only pcap returns `Ok(PcapSource { packets: [], datalink: ETHERNET })`. Zero-packet LINUX_SLL variant (`test_BC_2_01_003_zero_packet_linux_sll`) also passes in full suite.

### AC-006 (error path)
Uses `tests/fixtures/smb3.pcapng` fixture. The pcapng magic (0x0A0D0D0A) differs from classic pcap; `pcap_file` crate parse fails, returning `Err` with `"Failed to parse pcap header"` in the chain.

### AC-007 (success path — nanosecond edge case)
ts_frac=500_000 ns. Integer division by 1_000 yields timestamp_usecs=500. Confirms sub-microsecond precision is discarded, not rounded.

### AC-008 (error paths — zero-byte + garbage magic)
Two sub-cases: (1) empty `[u8; 0]` cursor; (2) 10-byte garbage starting `0xDEADBEEF`. Both return `Err` with `"Failed to parse pcap header"`. Truncated-valid-magic variant (`test_BC_2_01_006_truncated_header_error_message`) also passes in full suite.

### AC-009 (error path — all-or-nothing)
Valid packet followed by a 20-byte claimed record with only 4 bytes present. Returns `Err` with `"Failed to read packet"`. The valid first packet does NOT leak as a partial `Vec` — structurally impossible via `Result<PcapSource>` return type.

### AC-010 (error path)
`from_file` on `/tmp/wirerust-test-does-not-exist-bc-2-01-008.pcap` returns `Err` containing `"Failed to open"` and the path string. Permission-denied variant (`test_BC_2_01_008_permission_denied_error`) also passes; `from_file`→`from_pcap_reader` delegation equivalence (`test_BC_2_01_008_from_file_delegates_to_from_pcap_reader`) also passes.

---

## Artifact Inventory

All artifacts reside in:
```
.factory/cycles/v0.1.0-greenfield-spec/STORY-001/demos/
```

| File | Size | Purpose |
|------|------|---------|
| AC-001-accepts-all-five-link-types.tape | 583 B | VHS script source |
| AC-001-accepts-all-five-link-types.gif | 96 KB | PR-embeddable recording |
| AC-001-accepts-all-five-link-types.webm | 88 KB | Archival recording |
| AC-002-rejects-unsupported-link-type.tape | 606 B | VHS script source |
| AC-002-rejects-unsupported-link-type.gif | 104 KB | PR-embeddable recording |
| AC-002-rejects-unsupported-link-type.webm | 111 KB | Archival recording |
| AC-003-packet-count-and-order.tape | 554 B | VHS script source |
| AC-003-packet-count-and-order.gif | 91 KB | PR-embeddable recording |
| AC-003-packet-count-and-order.webm | 88 KB | Archival recording |
| AC-004-timestamp-preserved.tape | 540 B | VHS script source |
| AC-004-timestamp-preserved.gif | 95 KB | PR-embeddable recording |
| AC-004-timestamp-preserved.webm | 96 KB | Archival recording |
| AC-005-zero-packet-pcap.tape | 550 B | VHS script source |
| AC-005-zero-packet-pcap.gif | 92 KB | PR-embeddable recording |
| AC-005-zero-packet-pcap.webm | 90 KB | Archival recording |
| AC-006-rejects-pcapng.tape | 539 B | VHS script source |
| AC-006-rejects-pcapng.gif | 91 KB | PR-embeddable recording |
| AC-006-rejects-pcapng.webm | 87 KB | Archival recording |
| AC-007-nanosecond-conversion.tape | 577 B | VHS script source |
| AC-007-nanosecond-conversion.gif | 100 KB | PR-embeddable recording |
| AC-007-nanosecond-conversion.webm | 71 KB | Archival recording |
| AC-008-corrupt-header-error.tape | 578 B | VHS script source |
| AC-008-corrupt-header-error.gif | 100 KB | PR-embeddable recording |
| AC-008-corrupt-header-error.webm | 71 KB | Archival recording |
| AC-009-truncated-packet-error.tape | 582 B | VHS script source |
| AC-009-truncated-packet-error.gif | 99 KB | PR-embeddable recording |
| AC-009-truncated-packet-error.webm | 71 KB | Archival recording |
| AC-010-file-not-found-error.tape | 566 B | VHS script source |
| AC-010-file-not-found-error.gif | 97 KB | PR-embeddable recording |
| AC-010-file-not-found-error.webm | 87 KB | Archival recording |
| evidence-report.md | — | This file |

---

## Conclusion

All 10 acceptance criteria (AC-001 through AC-010) have passing demo evidence. Every recording maps to a named test in `tests/bc_2_01_story001_tests.rs`. Error paths are recorded for AC-002, AC-006, AC-008, AC-009, and AC-010. Success paths are recorded for AC-001, AC-003, AC-004, AC-005, and AC-007. The 20-test suite passes in 0.07s with zero failures.
