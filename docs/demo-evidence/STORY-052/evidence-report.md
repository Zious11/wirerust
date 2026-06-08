# Demo Evidence Report — STORY-052

**Story:** STORY-052 — ClientHello Parsing: Handshake Counting, Version/JA3 Tracking, and Done Short-Circuit
**Branch:** `feature/STORY-052`
**Implementation strategy:** brownfield-formalization (no runtime change; one `#[cfg(test)]` accessor added)
**Date:** 2026-05-28
**Recorded by:** Demo Recorder agent

---

## Coverage Summary

| Recording | ACs Covered | Test | Result |
|-----------|-------------|------|--------|
| AC-001-006-parse-client-hello | AC-001, AC-002, AC-003, AC-004, AC-005, AC-006 | `test_parse_client_hello` | PASS |
| AC-007-map-bounds | AC-007 | `test_BC_2_07_001_inv2_version_counts_bounded_at_max_map_entries`, `test_BC_2_07_001_inv2_ja3_counts_bounded_at_max_map_entries` | PASS |
| AC-008-009-012-stop-after-handshake | AC-008, AC-009, AC-012 | `test_stop_after_handshake` | PASS |
| AC-011-legacy-version-only | AC-011 | `test_BC_2_07_032_inv1_supported_versions_not_inspected` | PASS |
| AC-010-011-tls13-integration | AC-010, AC-011 | `test_tls13_pcap_version_and_ja3` (integration) | PASS |

**Total ACs:** 12
**ACs with recorded evidence:** 12 (AC-001 through AC-012)
**Unrecorded ACs:** 0

---

## AC → Evidence Mapping

### AC-001 (BC-2.07.001 postcondition 1): `handshakes_seen` incremented by exactly 1

- **Test:** `test_parse_client_hello` — asserts `analyzer.handshake_count() == 1` after one ClientHello
- **Recording:** [AC-001-006-parse-client-hello.gif](AC-001-006-parse-client-hello.gif) / [.webm](AC-001-006-parse-client-hello.webm)
- **Tape:** [AC-001-006-parse-client-hello.tape](AC-001-006-parse-client-hello.tape)
- **Behavioral Contract:** BC-2.07.001 postcondition 1

---

### AC-002 (BC-2.07.001 postcondition 2): `version_counts[0x0303]` incremented

- **Test:** `test_parse_client_hello` — asserts `version_counts.get(&0x0303) == 1`
- **Recording:** [AC-001-006-parse-client-hello.gif](AC-001-006-parse-client-hello.gif) / [.webm](AC-001-006-parse-client-hello.webm)
- **Tape:** [AC-001-006-parse-client-hello.tape](AC-001-006-parse-client-hello.tape)
- **Behavioral Contract:** BC-2.07.001 postcondition 2

---

### AC-003 (BC-2.07.001 postcondition 3): JA3 MD5 hex (32 lowercase hex chars) computed

- **Test:** `test_parse_client_hello` — asserts `ja3_counts.len() == 1` and key is 32-char lowercase hex
- **Recording:** [AC-001-006-parse-client-hello.gif](AC-001-006-parse-client-hello.gif) / [.webm](AC-001-006-parse-client-hello.webm)
- **Tape:** [AC-001-006-parse-client-hello.tape](AC-001-006-parse-client-hello.tape)
- **Behavioral Contract:** BC-2.07.001 postcondition 3

---

### AC-004 (BC-2.07.001 postcondition 4): SNI hostname inserted into `sni_counts`

- **Test:** `test_parse_client_hello` — asserts `sni_counts.get("example.com") == 1`
- **Recording:** [AC-001-006-parse-client-hello.gif](AC-001-006-parse-client-hello.gif) / [.webm](AC-001-006-parse-client-hello.webm)
- **Tape:** [AC-001-006-parse-client-hello.tape](AC-001-006-parse-client-hello.tape)
- **Behavioral Contract:** BC-2.07.001 postcondition 4

---

### AC-005 (BC-2.07.001 postcondition 8): `client_buf` drained after processing

- **Test:** `test_parse_client_hello` — asserts `client_buf_len_for_testing(&fk) == 0`
- **Recording:** [AC-001-006-parse-client-hello.gif](AC-001-006-parse-client-hello.gif) / [.webm](AC-001-006-parse-client-hello.webm)
- **Tape:** [AC-001-006-parse-client-hello.tape](AC-001-006-parse-client-hello.tape)
- **Behavioral Contract:** BC-2.07.001 postcondition 8

---

### AC-006 (BC-2.07.001 invariant 1): `handshakes_seen` increments exactly once per ClientHello

- **Test:** `test_parse_client_hello` — confirms count == 1 regardless of cipher count
- **Recording:** [AC-001-006-parse-client-hello.gif](AC-001-006-parse-client-hello.gif) / [.webm](AC-001-006-parse-client-hello.webm)
- **Tape:** [AC-001-006-parse-client-hello.tape](AC-001-006-parse-client-hello.tape)
- **Behavioral Contract:** BC-2.07.001 invariant 1

---

### AC-007 (BC-2.07.001 invariant 2): All counter maps bounded at `MAX_MAP_ENTRIES = 50,000`

- **Tests:**
  - `test_BC_2_07_001_inv2_version_counts_bounded_at_max_map_entries` — fills `version_counts` to 50k; asserts new key dropped silently
  - `test_BC_2_07_001_inv2_ja3_counts_bounded_at_max_map_entries` — fills `ja3_counts` to 50k; asserts new key dropped silently
- **Recording:** [AC-007-map-bounds.gif](AC-007-map-bounds.gif) / [.webm](AC-007-map-bounds.webm)
- **Tape:** [AC-007-map-bounds.tape](AC-007-map-bounds.tape)
- **Behavioral Contract:** BC-2.07.001 invariant 2

---

### AC-008 (BC-2.07.003 postconditions 1-5): After both hellos, `on_data` returns immediately

- **Test:** `test_stop_after_handshake` — asserts no bytes buffered, no counters changed, no findings after `done() == true`
- **Recording:** [AC-008-009-012-stop-after-handshake.gif](AC-008-009-012-stop-after-handshake.gif) / [.webm](AC-008-009-012-stop-after-handshake.webm)
- **Tape:** [AC-008-009-012-stop-after-handshake.tape](AC-008-009-012-stop-after-handshake.tape)
- **Behavioral Contract:** BC-2.07.003 postconditions 1-5

---

### AC-009 (BC-2.07.003 invariants 1-2): `done()` check is first; once true, it is permanent

- **Test:** `test_stop_after_handshake` — sends retransmitted ClientHello after done; asserts `handshakes_seen` unchanged
- **Recording:** [AC-008-009-012-stop-after-handshake.gif](AC-008-009-012-stop-after-handshake.gif) / [.webm](AC-008-009-012-stop-after-handshake.webm)
- **Tape:** [AC-008-009-012-stop-after-handshake.tape](AC-008-009-012-stop-after-handshake.tape)
- **Behavioral Contract:** BC-2.07.003 invariants 1-2

---

### AC-010 (BC-2.07.032 postconditions 1-3): TLS 1.3 legacy_version=0x0303 counted; no deprecated-protocol finding

- **Test:** `test_tls13_pcap_version_and_ja3` — asserts `version_counts.contains_key(&0x0303)` and `!version_counts.contains_key(&0x0304)` and `findings().is_empty()`
- **Recording:** [AC-010-011-tls13-integration.gif](AC-010-011-tls13-integration.gif) / [.webm](AC-010-011-tls13-integration.webm)
- **Tape:** [AC-010-011-tls13-integration.tape](AC-010-011-tls13-integration.tape)
- **Behavioral Contract:** BC-2.07.032 postconditions 1-3

---

### AC-011 (BC-2.07.032 invariants 1-2): Only `ch.version.0` used; `supported_versions` extension NOT inspected

- **Tests:**
  - `test_BC_2_07_032_inv1_supported_versions_not_inspected` — unit test asserts `version_counts` has 0x0303 only, not 0x0304
  - `test_tls13_pcap_version_and_ja3` — integration test asserts `!version_counts.contains_key(&0x0304)`
- **Recordings:**
  - [AC-011-legacy-version-only.gif](AC-011-legacy-version-only.gif) / [.webm](AC-011-legacy-version-only.webm) — unit test
  - [AC-010-011-tls13-integration.gif](AC-010-011-tls13-integration.gif) / [.webm](AC-010-011-tls13-integration.webm) — integration test
- **Tapes:**
  - [AC-011-legacy-version-only.tape](AC-011-legacy-version-only.tape)
  - [AC-010-011-tls13-integration.tape](AC-010-011-tls13-integration.tape)
- **Behavioral Contract:** BC-2.07.032 invariants 1-2

---

### AC-012 (BC-2.07.034 postconditions 1-3): 1 MB app-data burst after both hellos leaves all counters unchanged

- **Test:** `test_stop_after_handshake` — sends 1,048,576-byte burst; asserts all counters at post-handshake values
- **Recording:** [AC-008-009-012-stop-after-handshake.gif](AC-008-009-012-stop-after-handshake.gif) / [.webm](AC-008-009-012-stop-after-handshake.webm)
- **Tape:** [AC-008-009-012-stop-after-handshake.tape](AC-008-009-012-stop-after-handshake.tape)
- **Behavioral Contract:** BC-2.07.034 postconditions 1-3

---

## Recordings

| File | Format | Size | ACs |
|------|--------|------|-----|
| AC-001-006-parse-client-hello.gif | GIF | 98 KB | 001, 002, 003, 004, 005, 006 |
| AC-001-006-parse-client-hello.webm | WebM | 126 KB | 001, 002, 003, 004, 005, 006 |
| AC-007-map-bounds.gif | GIF | 105 KB | 007 |
| AC-007-map-bounds.webm | WebM | 177 KB | 007 |
| AC-008-009-012-stop-after-handshake.gif | GIF | 89 KB | 008, 009, 012 |
| AC-008-009-012-stop-after-handshake.webm | WebM | 112 KB | 008, 009, 012 |
| AC-010-011-tls13-integration.gif | GIF | 92 KB | 010, 011 |
| AC-010-011-tls13-integration.webm | WebM | 118 KB | 010, 011 |
| AC-011-legacy-version-only.gif | GIF | 104 KB | 011 |
| AC-011-legacy-version-only.webm | WebM | 140 KB | 011 |

---

## Test Commands

```bash
# AC-001..006
cargo test --test tls_analyzer_tests test_parse_client_hello -- --nocapture

# AC-007
cargo test --test tls_analyzer_tests test_BC_2_07_001_inv2 -- --nocapture

# AC-008, AC-009, AC-012
cargo test --test tls_analyzer_tests test_stop_after_handshake -- --nocapture

# AC-011 (unit)
cargo test --test tls_analyzer_tests test_BC_2_07_032_inv1_supported_versions_not_inspected -- --nocapture

# AC-010, AC-011 (integration)
cargo test --test tls_integration_tests test_tls13_pcap_version_and_ja3 -- --nocapture
```

All tests pass on `feature/STORY-052`. No source code was modified during recording.
