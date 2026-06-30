# Demo Evidence Report — STORY-145

**Story:** TLS ServerHello direction-symmetry handshake reassembly  
**Date:** 2026-06-30  
**Product:** wirerust (CLI, Rust)  
**Recording tool:** VHS 0.11.0 + ffmpeg 8.1

---

## Summary

STORY-145 extends the TLS handshake-message reassembly carry-buffer mechanism
(introduced in STORY-144 for ClientHello/ClientToServer) to the ServerToClient
direction. A ServerHello fragmented across multiple TLS records is now
reassembled so JA3S extraction and detection work correctly. All four STORY-145
acceptance tests pass; three VHS terminal recordings provide per-AC visual evidence.

---

## Per-AC Demo Recordings

| AC | Test / command | Recording | Duration | Size | Result |
|----|----------------|-----------|----------|------|--------|
| AC-145-001, AC-145-002 | `proptest_vp039_direction_isolation` | [WebM](AC-001-002-direction-carry-drain.webm) [GIF](AC-001-002-direction-carry-drain.gif) | 14s | 136K / 537K | ok |
| AC-145-003 | `test_BC_2_07_041_cross_flow_isolation` | [WebM](AC-003-cross-flow-isolation.webm) [GIF](AC-003-cross-flow-isolation.gif) | 15s | 136K / 472K | ok |
| AC-145-005 | `test_parse_server_hello` + CLI pcap smoke | [WebM](AC-005-single-record-regression.webm) [GIF](AC-005-single-record-regression.gif) | 23s | 266K / 1.3M | ok |

---

## AC Coverage Detail

### AC-145-001 + AC-145-002: Direction-parameterized carry drain & C2S/S2C isolation

**Recording:** `AC-001-002-direction-carry-drain.webm`

**What is shown:** `proptest_vp039_direction_isolation` — a property-based test
that runs at multiple random split points. Three parallel analyzer instances
receive the same fragmented ClientHello (C2S) and fragmented ServerHello (S2C):
one interleaved, one C2S-only, one S2C-only. After full delivery the test asserts:

- `client_hello_seen == true` (C2S carry drain — STORY-144 path)
- `server_hello_seen == true` (S2C carry drain — **STORY-145 path**)
- `sni_counts` non-empty (SNI extracted from fragmented ClientHello)
- `ja3s_counts` non-empty (JA3S extracted from fragmented ServerHello)
- `parse_errors == 0`
- both `client_hs_carry_len == 0` and `server_hs_carry_len == 0` after full delivery

Terminal output visible: `test story_145::proptest_vp039_direction_isolation ... ok`
`test result: ok. 1 passed; 0 failed`

---

### AC-145-003: Cross-flow isolation (two concurrent fragmented-ServerHello flows)

**Recording:** `AC-003-cross-flow-isolation.webm`

**What is shown:** `test_BC_2_07_041_cross_flow_isolation` — one `TlsAnalyzer`
instance processes two flows simultaneously:

- **Flow A** (seed=10): complete single-record ClientHello (`a.example`) + complete S2C ServerHello
- **Flow B** (seed=20): fragmented 2-record ClientHello (`b.example`) + fragmented 2-record S2C ServerHello

Assertions verified: both `server_hello_seen == true`; `sni_counts` has exactly
2 entries (a.example + b.example, no cross-flow bleed); `ja3s_counts >= 1`; all
carry buffers drain to zero; `parse_errors == 0`.

Terminal output visible: `test story_145::test_BC_2_07_041_cross_flow_isolation ... ok`

---

### AC-145-005: Single-record ServerHello regression check

**Recording:** `AC-005-single-record-regression.webm`

**What is shown (two parts):**

1. **Unit test:** `test_parse_server_hello` — delivers a complete single-record
   ServerHello in `Direction::ServerToClient` and asserts `server_hello_seen == true`.
   Confirms the new carry-drain code path does not break non-fragmented delivery.
   
2. **CLI smoke test:** `wirerust analyze tests/fixtures/tls12-aes256gcm.pcap` —
   real TLS 1.2 capture produces a valid WIRERUST TRIAGE REPORT (Packets: 9,
   TLS: 9) with no parse errors in the output.

Terminal output visible: `test test_parse_server_hello ... ok` followed by the
CLI triage report.

---

## AC-145-004: Not separately recorded

AC-145-004 (overflow guard — `server_hs_carry` clears on Step-1 overflow and
recovers on subsequent complete ServerHello) is covered by
`test_vp039_server_carry_overflow_clear_and_recover` and
`test_vp039_server_body_len_spoof`, both of which pass in the full test suite.
These are overflow/error-path tests exercising internal guard invariants; they
produce no user-observable CLI output distinguishable from a normal pass.
Full suite evidence: `cargo test --test tls_analyzer_tests story_145` →
`4 passed; 0 failed`.

---

## VHS Tape Scripts

| Script | AC |
|--------|----|
| [AC-001-002-direction-carry-drain.tape](AC-001-002-direction-carry-drain.tape) | AC-145-001, AC-145-002 |
| [AC-003-cross-flow-isolation.tape](AC-003-cross-flow-isolation.tape) | AC-145-003 |
| [AC-005-single-record-regression.tape](AC-005-single-record-regression.tape) | AC-145-005 |

---

## Demo Toolchain

| Tool | Version |
|------|---------|
| VHS | 0.11.0 |
| ffmpeg | 8.1 |
| Rust / cargo | stable (incremental build, pre-compiled) |
