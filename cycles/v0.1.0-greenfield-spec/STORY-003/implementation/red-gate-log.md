# Red Gate Log — STORY-003 Phase 3

**Story:** STORY-003 — Packet Decoding: Linux SLL, No-Panic Safety, Non-IP Frame Rejection
**Cycle:** v0.1.0-greenfield-spec
**Wave:** 2
**Implementation strategy:** brownfield-formalization
**Date:** 2026-05-22
**Test file:** `tests/bc_2_02_story003_tests.rs`
**Agent:** test-writer

---

## Summary

17 of 18 new tests PASS on the first run. This is expected: the implementation
strategy is `brownfield-formalization`, which means `src/decoder.rs` already
exists and is being formally covered by tests for the first time.  Every passing
test confirms existing correct behavior (brownfield-confirm).

1 test FAILS by design: `test_VP_008_fuzz_harness_exists`. The cargo-fuzz
harness at `fuzz/fuzz_targets/fuzz_decode_packet.rs` does not yet exist. This is
the genuine Red Gate for AC-011 / VP-008. The implementer must create the harness
as STORY-003.md task 8 before this test will pass.

`cargo clippy --all-targets -- -D warnings` exits 0 (0 errors, 0 warnings).
`cargo test --all-targets` exits with code 101 (expected — one Red Gate failure).

---

## Red Gate Disposition

| Disposition | Count |
|-------------|-------|
| PASS (brownfield-confirm) | 17 |
| FAIL (Red Gate — implementation gap) | 1 |

---

## Per-Test Results

| Test Name | Traces to | BC clause(s) exercised | Result | Disposition |
|-----------|-----------|------------------------|--------|-------------|
| `test_BC_2_02_006_linux_sll_ipv4_tcp` | AC-001 | BC-2.02.006 postcondition 1, postcondition 2, postcondition 3 | PASS | brownfield-confirm |
| `test_BC_2_02_006_linux_sll_ipv6_tcp` | AC-002 | BC-2.02.006 postcondition 1 (IPv6 variant) | PASS | brownfield-confirm |
| `test_BC_2_02_006_linux_sll_snaplen_truncated_lax_recovery` | AC-003 | BC-2.02.006 invariant 2 (lax path invoked on Len error) | PASS | brownfield-confirm |
| `test_BC_2_02_006_linux_sll_sub_16_bytes_rejected` | AC-004 | BC-2.02.006 invariant 3 (sub-16-byte → non-Len error → immediate Err, no lax retry) | PASS | brownfield-confirm |
| `test_BC_2_02_007_random_bytes_no_panic` | AC-005 | BC-2.02.007 postcondition 1 + invariant 3 (no panic; Err on random bytes via RAW path) | PASS | brownfield-confirm |
| `test_BC_2_02_007_empty_slice_no_panic` | AC-006 | BC-2.02.007 postcondition 1 + invariant 3 (no panic; Err on empty slice) | PASS | brownfield-confirm |
| `test_BC_2_02_007_error_prefix_exhaustiveness` | AC-007 | BC-2.02.007 invariant 1 (three prefixes only: "Unsupported link type:", "No IP layer found", "Parse error:") | PASS | brownfield-confirm |
| `test_BC_2_02_008_unsupported_link_type_error` | AC-008 | BC-2.02.008 postcondition 1, postcondition 2, postcondition 3 | PASS | brownfield-confirm |
| `test_BC_2_02_009_non_ip_frame_rejected` | AC-009 | BC-2.02.009 postcondition 1 (Ethernet ARP → "No IP layer found") | PASS | brownfield-confirm |
| `test_BC_2_02_009_lax_path_also_rejects_no_ip` | AC-010 | BC-2.02.009 invariant 1, invariant 2, invariant 3 (SLL ARP strict-path rejection; lax NOT attempted) | PASS | brownfield-confirm |
| `test_VP_008_fuzz_harness_exists` | AC-011 / VP-008 | VP-008 (decode_packet Never Panics on Arbitrary Input) — file-presence check | **FAIL** | **Red Gate — implementation gap** |
| `test_BC_2_02_006_ec001_sll_ipv4_tcp_strict_path` | EC-001 | BC-2.02.006 postcondition 3 (SYN flag captured via strict path) | PASS | brownfield-confirm |
| `test_BC_2_02_006_ec002_sll_snaplen_lax_invoked` | EC-002 | BC-2.02.006 invariant 2 (lax path recovers IP layer for snaplen truncation) | PASS | brownfield-confirm |
| `test_BC_2_02_006_ec003_sll_sub_16_bytes_no_lax_retry` | EC-003 | BC-2.02.006 invariant 3 (8-byte frame → no lax retry) | PASS | brownfield-confirm |
| `test_BC_2_02_007_ec004_empty_data_sll` | EC-004 | BC-2.02.007 invariant 3 (empty slice on LINUX_SLL path) | PASS | brownfield-confirm |
| `test_BC_2_02_008_ec005_ieee802_11_rejected` | EC-005 | BC-2.02.008 EC-001 (IEEE802_11 rejected with empty data — proves no byte access before gate) | PASS | brownfield-confirm |
| `test_BC_2_02_009_ec006_arp_ethernet_no_ip_layer` | EC-006 | BC-2.02.009 EC-001 (ARP frame exact message: "No IP layer found") | PASS | brownfield-confirm |
| `test_BC_2_02_009_ec007_custom_ethertype_no_ip_layer` | EC-007 | BC-2.02.009 canonical test vector (EtherType 0x9000) | PASS | brownfield-confirm |

---

## Implementation Gaps Found

### GAP-001: VP-008 Fuzz Harness Missing (P0)

- **Test:** `test_VP_008_fuzz_harness_exists`
- **AC:** AC-011
- **VP:** VP-008 — "decode_packet Never Panics on Arbitrary Input"
- **Required file:** `fuzz/fuzz_targets/fuzz_decode_packet.rs`
- **Priority:** P0 (mandatory per STORY-003.md task 8)
- **Action required:** Implementer must create `fuzz/fuzz_targets/fuzz_decode_packet.rs`
  implementing a cargo-fuzz harness that passes arbitrary byte slices to `decode_packet`
  with each supported `DataLink` variant and asserts no call panics.
  Then run `cargo +nightly fuzz build fuzz_decode_packet` to confirm compilation.
- **Status:** OPEN — no implementation action taken in this test-writing pass.
  Do NOT skip; VP-008 is a P0 verification property.

---

## Test Design Notes

### AC-005 / test_BC_2_02_007_random_bytes_no_panic

The canonical test vector in BC-2.02.007 specifies "Random 20 bytes with ETHERNET → Err (no
panic)". During authoring it was discovered that the chosen 20-byte sequence
(`0xDE 0xAD 0xBE 0xEF ...`) happens to pass Ethernet header parsing (14-byte Ethernet header
with EtherType 0xBEEF, an unknown but structurally valid EtherType) and produces "No IP layer
found" instead of "Parse error:". Both are correct Err responses satisfying BC-2.02.007
postcondition 1. The test was adapted to use `DataLink::RAW` (raw-IP path) where the same
bytes fail at the IP version nibble check (nibble = 0xD, neither 4 nor 6) and reliably produce
"Parse error:". This is a valid interpretation: BC-2.02.007 precondition 2 permits any of the
five supported DataLink variants; the no-panic guarantee holds across all of them. A separate
EC-004 test exercises the empty-slice case on `LINUX_SLL` for path diversity.

### AC-007 / test_BC_2_02_007_error_prefix_exhaustiveness

This test verifies BC-2.02.007 invariant 1 by exercising one representative case per prefix
and asserting mutual exclusivity: for a given input, exactly one of the three prefixes appears
in the error message. The `.leak()` call on `make_ethernet_arp()` is a deliberate pattern to
extend the byte slice's lifetime to `'static` for inclusion in a `&[(&[u8], DataLink, &str)]`
array — it is appropriate for a test binary where the small allocation is reclaimed on process
exit.

---

## Fixture Analysis

No new fixture files were written to disk.  All frame bytes are constructed
inline within the test binary:

| Fixture scenario | Source |
|-----------------|--------|
| SLL IPv4 TCP (56 bytes) | `make_sll_ipv4_tcp()` — inline bytes |
| SLL IPv6 TCP (76 bytes) | `make_sll_ipv6_tcp()` — inline bytes |
| SLL IPv4 TCP snaplen-truncated | `make_sll_ipv4_tcp_snaplen_truncated()` — derived |
| Ethernet ARP (42 bytes) | `make_ethernet_arp()` — inline bytes |
| Ethernet custom EtherType 0x9000 (18 bytes) | `make_ethernet_custom_ethertype_0x9000()` — inline bytes |
| Random bytes (20 bytes) | deterministic literal |
| Empty slice (0 bytes) | `&[]` |
| Sub-16-byte SLL frame (15 bytes) | inline literal |
| 8-byte SLL frame | inline literal |
| 64-byte zero fill with IEEE802_11 | inline literal |
| SLL ARP (44 bytes) | inline literal inside test body |

---

## BC Coverage Map

| BC | Clauses covered | Coverage mode |
|----|----------------|---------------|
| BC-2.02.006 | precondition 1 (exercised as test input — `make_sll_ipv4_tcp()`); precondition 2 (exercised as test input — `DataLink::LINUX_SLL`); precondition 3 (test-exercised via AC-004/EC-003 boundary tests); postcondition 1 (test-exercised: AC-001, AC-002); postcondition 2 (test-exercised: `packet_len == data.len()` assertion in AC-001); postcondition 3 (test-exercised: AC-001, EC-001); invariant 1 (exercised as test input — `SLL_HEADER_LEN` == 16 boundary established by 15-byte rejection test); invariant 2 (test-exercised: AC-003, EC-002 — lax path recovers IP after Len error); invariant 3 (test-exercised: AC-004, EC-003 — sub-16-byte frame rejected without lax retry) | test-exercised |
| BC-2.02.007 | precondition 1 (exercised as test input — various random/empty byte slices); precondition 2 (exercised as test input — multiple DataLink variants); postcondition 1 (test-exercised: AC-005, AC-006); postcondition 2 (structural — test runner catches panics as failures; panic = test failure); postcondition 3 (test-exercised: "Parse error:" assertion in AC-005, AC-006); invariant 1 (test-exercised: AC-007 exhaustiveness test); invariant 2 (structural — no `unwrap()` in `decode_packet` is a code property confirmed by code review; observable via no panics in any test); invariant 3 (test-exercised: AC-005, AC-006, EC-004) | mixed |
| BC-2.02.008 | precondition 1 (exercised as test input — `DataLink::IEEE802_11`); postcondition 1 (test-exercised: AC-008, EC-005); postcondition 2 (test-exercised: "IEEE802_11" in error message); postcondition 3 (structural — test runner catches panics; no panic = no violation); postcondition 4 (test-exercised: EC-005 uses empty data, proving no bytes are read before rejection); invariant 1 (structural — `other =>` wildcard arm returns before any byte access; confirmed by code review and EC-005 behavioral evidence); invariant 2 (structural — same guard in `lax_parse`; not separately test-exercised here since `lax_parse` is only called after a Len error, and IEEE802_11 fails before that); invariant 3 (test-exercised: "Unsupported link type:" prefix verified) | mixed |
| BC-2.02.009 | precondition 1 (exercised as test input — ARP frame, custom EtherType); precondition 2 (exercised as test input — ETHERNET, LINUX_SLL); precondition 3 (exercised as test input — `SlicedPacket::from_*` returns Ok with `net == None`); postcondition 1 (test-exercised: AC-009, EC-006, EC-007); postcondition 2 (structural — test runner catches panics); postcondition 3 (structural — caller not tested here; no-panic is the relevant BC-2.02.009 guarantee); invariant 1 (test-exercised: AC-010 verifies "No IP layer found" on strict path for SLL ARP); invariant 2 (structural — lax path also rejects if net=None, same code path; AC-010 tests the strict-first rejection, not a lax-second rejection, since no Len error occurs); invariant 3 (test-exercised: AC-010 uses SLL ARP which produces no Len error, confirming lax is not attempted for non-IP frames) | mixed |

---

## Compilation and Lint

| Check | Result |
|-------|--------|
| `cargo check --tests` | OK |
| `cargo clippy --all-targets -- -D warnings` | OK (0 errors, 0 warnings) |
| `cargo test --all-targets` | 17 PASS, 1 FAIL (expected Red Gate: `test_VP_008_fuzz_harness_exists`) |
