---
document_type: demo-evidence-report
story_id: STORY-112
worktree_branch: worktree-issue-9-story-112-arp-extract-frame
head_at_recording: c68964d
demo_commit: 76bdf16
recorded: 2026-06-15
producer: demo-recorder
---

# Demo Evidence Report — STORY-112: extract_arp_frame + decode_packet ARP Routing + ArpAnalyzer Stub

## What Changed

Before STORY-112, Ethernet ARP frames hit a `"No IP layer found"` decode error and were counted
as `skipped_packets`. After STORY-112, `extract_arp_frame` + `decode_packet` route
Ethernet/IPv4 ARP frames to `DecodedFrame::Arp`. `ArpAnalyzer::process_arp` is a no-op stub
(full detection lands in STORY-113), so no new findings/alerts appear yet — the observable
effect is fewer skipped packets and zero decode-error warnings on ARP-containing pcaps.

## Exact CLI Command

```
./target/release/wirerust --output-format json --no-color analyze <pcap-file>
```

Note: `--output-format json` is the correct flag for JSON output to stdout (not `--json`).

## Recordings

| File | Evidences | Type |
|------|-----------|------|
| `AC-006-008-arp-decode-routing.tape` | AC-006, AC-008 (success path) | VHS script |
| `AC-006-008-arp-decode-routing.gif` | AC-006, AC-008 (success path) | GIF (PR embed) |
| `AC-006-008-arp-decode-routing.webm` | AC-006, AC-008 (success path) | WebM (archival) |
| `AC-006-008-one-decode-error.tape` | AC-006, AC-008 (error path) | VHS script |
| `AC-006-008-one-decode-error.gif` | AC-006, AC-008 (error path) | GIF (PR embed) |
| `AC-006-008-one-decode-error.webm` | AC-006, AC-008 (error path) | WebM (archival) |
| `AC-001-012-tests.tape` | AC-001..AC-012 (unit tests) | VHS script |
| `AC-001-012-tests.gif` | AC-001..AC-012 (unit tests) | GIF (PR embed) |
| `AC-001-012-tests.webm` | AC-001..AC-012 (unit tests) | WebM (archival) |
| `FULL-SUITE-1512-tests.tape` | All ACs (full suite) | VHS script |
| `FULL-SUITE-1512-tests.gif` | All ACs (full suite) | GIF (PR embed) |
| `FULL-SUITE-1512-tests.webm` | All ACs (full suite) | WebM (archival) |

## Observed skipped_packets Values

| Fixture | Before STORY-112 | After STORY-112 | Delta |
|---------|-----------------|-----------------|-------|
| `dns-remoteshell.pcap` | 73 | **69** | -4 (4 ARP frames now decoded as DecodedFrame::Arp) |
| `one-decode-error.pcap` | 1 (decode warning) | **0** | -1 (ARP packet now decoded cleanly, no stderr warning) |

## Test Results

- Focused `cargo test test_BC_2_16`: **15 passed, 0 failed** (all AC-001..AC-012 unit tests)
- Full `cargo test --all-targets`: **1512 passed, 0 failed**

Focused tests confirmed passing at `76bdf16` (demo commit):
- `test_BC_2_16_001_extract_arp_frame_request_returns_some` (AC-001)
- `test_BC_2_16_001_extract_arp_frame_request_field_copy_fidelity` (AC-002)
- `test_BC_2_16_002_extract_arp_frame_reply_returns_some_with_correct_fields` (AC-003)
- `test_BC_2_16_001_extract_arp_frame_none_on_hw_addr_size_8` (AC-004a)
- `test_BC_2_16_001_extract_arp_frame_none_on_proto_addr_size_16` (AC-004b)
- `test_BC_2_16_001_extract_arp_frame_outer_src_mac_none_passthrough` (AC-005)
- `test_BC_2_16_015_decode_packet_routes_arp_to_decoded_frame_arp` (AC-006)
- `test_BC_2_16_015_decode_packet_lax_arm_truncated_arp_non_panic` (AC-007)
- `test_BC_2_16_015_main_arp_arm_calls_process_arp_stub` (AC-008)
- `test_BC_2_16_015_arp_frame_never_reaches_stream_dispatcher` (AC-009)
- `test_BC_2_16_015_decode_packet_arp_non_eth_ipv4_returns_error` (AC-012)
- + 4 invariant tests (opcode-agnostic, zero target MAC, GARP reply, outer_src_mac mismatch)

## AC Coverage Map

| AC | Description | Evidenced By |
|----|-------------|--------------|
| AC-001 | `extract_arp_frame` returns Some for ARP Request | `AC-001-012-tests.*` |
| AC-002 | Field copy fidelity for ARP Request | `AC-001-012-tests.*` |
| AC-003 | ARP Reply extraction returns Some with correct fields | `AC-001-012-tests.*` |
| AC-004 | Returns None for non-standard hw/proto sizes (hw=8, proto=16) | `AC-001-012-tests.*` |
| AC-005 | `outer_src_mac=None` passed through unchanged | `AC-001-012-tests.*` |
| AC-006 | `decode_packet` routes ARP to `Ok(DecodedFrame::Arp)` | `AC-001-012-tests.*`, `AC-006-008-arp-decode-routing.*` |
| AC-007 | Truncated ARP lax arm never panics | `AC-001-012-tests.*` |
| AC-008 | `ArpAnalyzer::process_arp` stub called, returns `vec![]` | `AC-001-012-tests.*`, `AC-006-008-arp-decode-routing.*` |
| AC-009 | `DecodedFrame::Arp` never reaches `StreamDispatcher` | `AC-001-012-tests.*` |
| AC-010 | ArpAnalyzer stub compiles; clippy clean | Full suite + STORY-112 Step-4.5 CI evidence |
| AC-011 | VP-024 Sub-A Kani harnesses (todo!() skeletons) | Deferred to F6 formal-hardening (D-062 precedent) |
| AC-012 | Non-Eth/IPv4 ARP returns `Err("Non-Ethernet/IPv4 ARP frame")` | `AC-001-012-tests.*`, `AC-006-008-one-decode-error.*` |
