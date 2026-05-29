---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/tls.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: add buffer-cap observability note + residue-test back-refs; cap now literally verified (F-S058-P1-001) — 2026-05-29"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.005: Per-Direction Buffer Capped at MAX_BUF = 65536 Bytes

## Description

When `on_data` is called with new bytes for a flow direction, the bytes are appended to
the per-direction buffer (`client_buf` or `server_buf`) only up to the remaining capacity.
`MAX_BUF = 65,536` bytes. If the buffer is already full, no bytes are copied and the call
returns without error. This prevents unbounded memory growth from a flow that sends a large
volume of data before any parseable TLS record appears.

## Observability Note (F-S058-P1-001)

The 65,536-byte peak resident in the buffer is **instantaneous and not externally
observable** after `on_data` returns in the "normal" path. The reason: to keep
65,536 bytes resident, the buffered content must include an incomplete TLS record
(one whose declared `payload_len` exceeds the bytes available). However, a record
with `payload_len > 18,432` trips the oversized-record guard (BC-2.07.004), which
**clears** the buffer unconditionally before returning. Therefore:

- A buffer exactly at 65,536 bytes after `on_data` can only arise if the incomplete
  record's declared `payload_len` is within the valid range (≤ 18,432) but its
  payload bytes have not yet arrived — a transient state that resolves when the next
  `on_data` call delivers the missing payload or the flow closes.
- No external API exposes the mid-call buffer length; the cap constraint is
  observable only through residue: after `on_data` completes, `client_buf.len()` is
  bounded by the bytes that survived record parsing, not by the raw bytes fed in.

**Proof via residue technique (STORY-058 executable tests):**

| Test Name | What It Proves |
|-----------|----------------|
| `test_buffer_cap_appends_at_most_max_buf_literal_residue` | Feeds MAX_BUF+1 bytes (Alert drain records + a 6-byte incomplete handshake header + 1 trailing byte). After `on_data`, asserts resident buffer == 6 bytes. Without the `.min(remaining)` cap clip the residue would be 7, proving the clip fires and is not a no-op. |
| `test_buffer_full_append_noop_literal` | Proves the no-op append path: when the buffer is pre-filled to MAX_BUF, a subsequent `on_data` call does not increase `client_buf.len()`. |
| `test_buffer_cap_appends_at_most_max_buf` | Broader property-style coverage of the cap (silence variant). |
| `test_buffer_full_append_noop` | Silence-path coverage of the no-op append. |
| `test_buffer_overflow_silent_no_counters` | Confirms `parse_errors` and `truncated_records` remain 0 when bytes are dropped by the cap (silent overflow). |

## Preconditions

1. `on_data` is called for a flow that is NOT yet done (both hellos not yet seen).
2. `data` contains bytes to be buffered for the given direction.

## Postconditions

1. At most `MAX_BUF - current_buf_len` bytes from `data` are appended to the buffer.
2. If `current_buf_len >= MAX_BUF`, no bytes are appended.
3. After appending, `try_parse_records` is called with whatever is now in the buffer.
4. No error is returned; the truncation is silent (no counter increment).
5. `parse_errors` and `truncated_records` are NOT incremented for buffer overflow.

## Invariants

1. `client_buf.len()` and `server_buf.len()` are always `<= MAX_BUF`.
2. The cap is computed as `remaining = MAX_BUF.saturating_sub(state.buf.len())`.
   `to_copy = data.len().min(remaining)`. This is a safe, non-panicking calculation.
3. Buffer overflow is silent. There is no finding, no log line, and no counter
   tracking how many bytes were dropped beyond the cap.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Buffer at 65,535; data is 2 bytes | 1 byte appended; 1 byte dropped |
| EC-002 | Buffer at 65,536 (full); data is 1000 bytes | 0 bytes appended; data silently dropped |
| EC-003 | Buffer at 0; data is 65,537 bytes | 65,536 bytes appended; 1 byte dropped |
| EC-004 | Buffer at 0; data is exactly 65,536 bytes | All 65,536 bytes appended; no drop |
| EC-005 | Buffer is full and contains an incomplete TLS record | Record assembly stalls; no parse progress until flow closes |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Append 65,537 bytes to empty client_buf | client_buf.len() == 65,536; try_parse_records called with full buffer | edge-case |
| Append 1 byte when buffer is full | client_buf.len() unchanged at 65,536 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | client_buf.len() never exceeds MAX_BUF | proptest: fuzz on_data with arbitrary lengths |
| — | Cap clip is not a no-op: residue after MAX_BUF+1-byte feed is 6, not 7 | unit: test_buffer_cap_appends_at_most_max_buf_literal_residue (STORY-058) |
| — | No-op append when buffer already full | unit: test_buffer_full_append_noop_literal (STORY-058) |
| — | parse_errors and truncated_records remain 0 on silent cap drop | unit: test_buffer_overflow_silent_no_counters (STORY-058) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- per-direction buffer cap is part of TLS analysis bounded-resource design (ARCH-INDEX Cross-Cutting Concerns) |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:726-748, C-13) |
| Stories | STORY-058 |
| Origin BC | BC-TLS-005 (pass-3 ingestion corpus; confidence upgraded to HIGH — cap literally proven via residue tests in STORY-058, F-S058-P1-001) |

## Related BCs

- BC-2.07.004 -- related to (MAX_RECORD_PAYLOAD is a separate, record-level cap)
- BC-2.07.003 -- related to (after done, buffering is bypassed entirely before the cap check)

## Architecture Anchors

- `src/analyzer/tls.rs:726-748` -- on_data buffer-append logic with remaining/to_copy cap
- `src/analyzer/tls.rs:29` -- `const MAX_BUF: usize = 65_536`
- `tests/tls_analyzer_tests.rs` -- test_buffer_cap_appends_at_most_max_buf_literal_residue (residue proof)
- `tests/tls_analyzer_tests.rs` -- test_buffer_full_append_noop_literal (no-op append proof)
- `tests/tls_analyzer_tests.rs` -- test_buffer_cap_appends_at_most_max_buf (silence-variant coverage)
- `tests/tls_analyzer_tests.rs` -- test_buffer_full_append_noop (silence-variant coverage)
- `tests/tls_analyzer_tests.rs` -- test_buffer_overflow_silent_no_counters (silent-drop counter proof)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:726-748` |
| **Confidence** | high (cap now literally proven via residue technique — see Observability Note) |
| **Extraction Date** | 2026-05-20 |
| **Confidence Upgraded** | 2026-05-29 (F-S058-P1-001): residue tests in STORY-058 worktree confirm the `.min(remaining)` clip fires and is not dead code |

## Evidence Types Used

- **guard clause**: `remaining = MAX_BUF.saturating_sub(state.buf.len()); to_copy = data.len().min(remaining)`
- **literal residue test**: `test_buffer_cap_appends_at_most_max_buf_literal_residue` — feeds MAX_BUF+1 bytes and asserts resident buffer == 6 bytes; clip removal would yield 7
- **noop assertion**: `test_buffer_full_append_noop_literal` — proves zero-copy path when buffer is full
- **silence assertion**: `test_buffer_overflow_silent_no_counters` — proves no counter increments on silent drop

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates client_buf or server_buf |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
