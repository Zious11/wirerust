# Red Gate Log — STORY-005

## Summary

- **Story:** STORY-005 — Packet Decoding: packet_len Semantics and TCP Flag/Sequence Extraction
- **BCs:** BC-2.02.014, BC-2.02.015
- **Test file:** `tests/bc_2_02_story005_tests.rs`
- **Date:** 2026-05-22
- **Agent:** test-writer

## Red Gate Result

**PASS** — All 15 tests failed with the expected `RED GATE: <AC/EC id> not yet verified` panic messages. No test passed vacuously.

```
running 15 tests
test test_BC_2_02_015_psh_urg_not_in_transport_info ... FAILED
test test_BC_2_02_014_ec002_54_byte_pure_ack ... FAILED
test test_BC_2_02_015_ec004_seq_number_max_u32 ... FAILED
test test_BC_2_02_014_packet_len_set_on_both_strict_and_lax_paths ... FAILED
test test_BC_2_02_014_packet_len_equals_data_len ... FAILED
test test_BC_2_02_014_ec003_snaplen_truncated_at_100 ... FAILED
test test_BC_2_02_014_snaplen_truncated_packet_len ... FAILED
test test_BC_2_02_015_tcp_payload_bytes ... FAILED
test test_BC_2_02_015_ec006_no_flags_set ... FAILED
test test_BC_2_02_015_tcp_rst_and_fin_ack_flags ... FAILED
test test_BC_2_02_014_ec001_1500_byte_frame_packet_len ... FAILED
test test_BC_2_02_015_tcp_seq_number_extracted ... FAILED
test test_BC_2_02_015_tcp_syn_flags ... FAILED
test test_BC_2_02_015_tcp_syn_ack_flags ... FAILED
test test_BC_2_02_015_ec005_all_four_flags_set ... FAILED

failures:
    test_BC_2_02_014_ec001_1500_byte_frame_packet_len
    test_BC_2_02_014_ec002_54_byte_pure_ack
    test_BC_2_02_014_ec003_snaplen_truncated_at_100
    test_BC_2_02_014_packet_len_equals_data_len
    test_BC_2_02_014_packet_len_set_on_both_strict_and_lax_paths
    test_BC_2_02_014_snaplen_truncated_packet_len
    test_BC_2_02_015_ec004_seq_number_max_u32
    test_BC_2_02_015_ec005_all_four_flags_set
    test_BC_2_02_015_ec006_no_flags_set
    test_BC_2_02_015_psh_urg_not_in_transport_info
    test_BC_2_02_015_tcp_payload_bytes
    test_BC_2_02_015_tcp_rst_and_fin_ack_flags
    test_BC_2_02_015_tcp_seq_number_extracted
    test_BC_2_02_015_tcp_syn_ack_flags
    test_BC_2_02_015_tcp_syn_flags

test result: FAILED. 0 passed; 15 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Test Coverage

| Test Name | AC/EC | BC Clause |
|-----------|-------|-----------|
| `test_BC_2_02_014_packet_len_equals_data_len` | AC-001 | BC-2.02.014 postcondition 1 |
| `test_BC_2_02_014_packet_len_set_on_both_strict_and_lax_paths` | AC-002 | BC-2.02.014 invariant 1 |
| `test_BC_2_02_014_snaplen_truncated_packet_len` | AC-003 | BC-2.02.014 invariant 2 |
| `test_BC_2_02_015_tcp_syn_flags` | AC-004 | BC-2.02.015 postcondition 4 |
| `test_BC_2_02_015_tcp_syn_ack_flags` | AC-005 | BC-2.02.015 postcondition 5 |
| `test_BC_2_02_015_tcp_rst_and_fin_ack_flags` | AC-006 | BC-2.02.015 postconditions 6+7 |
| `test_BC_2_02_015_tcp_seq_number_extracted` | AC-007 | BC-2.02.015 postcondition 3 |
| `test_BC_2_02_015_tcp_payload_bytes` | AC-008 | BC-2.02.015 postcondition 8 |
| `test_BC_2_02_015_psh_urg_not_in_transport_info` | AC-009 | BC-2.02.015 invariant 3 |
| `test_BC_2_02_014_ec001_1500_byte_frame_packet_len` | EC-001 | BC-2.02.014 EC-001 |
| `test_BC_2_02_014_ec002_54_byte_pure_ack` | EC-002 | BC-2.02.014 EC-004 + BC-2.02.015 EC-006 |
| `test_BC_2_02_014_ec003_snaplen_truncated_at_100` | EC-003 | BC-2.02.014 EC-003 |
| `test_BC_2_02_015_ec004_seq_number_max_u32` | EC-004 | BC-2.02.015 EC-005 |
| `test_BC_2_02_015_ec005_all_four_flags_set` | EC-005 | BC-2.02.015 (all flags) |
| `test_BC_2_02_015_ec006_no_flags_set` | EC-006 | BC-2.02.015 EC-007 |

## BC Ambiguities / Divergences

None found. The existing `src/decoder.rs` implementation visibly satisfies all BCs:
- `build_parsed` receives `data.len()` at both call sites (decoder.rs:145 and decoder.rs:161)
- `TransportInfo::Tcp` struct contains exactly `src_port`, `dst_port`, `seq_number`, `syn`, `ack`, `fin`, `rst` — no `psh` or `urg` fields
- `seq_number` is extracted via `tcp.to_header().sequence_number` as specified by BC-2.02.015 invariant 1

## Next Step

Hand off to implementer: replace each `panic!("RED GATE: ...")` stub body with the real
assertion exercising the BC postcondition/invariant against `src/decoder.rs`.
