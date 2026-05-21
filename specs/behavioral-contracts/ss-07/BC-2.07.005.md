---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified: ["v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"]
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

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- per-direction buffer cap is part of TLS analysis bounded-resource design (ARCH-INDEX Cross-Cutting Concerns) |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:726-748, C-13) |
| Stories | STORY-058 |
| Origin BC | BC-TLS-005 (pass-3 ingestion corpus, MEDIUM confidence -- not directly tested) |

## Related BCs

- BC-2.07.004 -- related to (MAX_RECORD_PAYLOAD is a separate, record-level cap)
- BC-2.07.003 -- related to (after done, buffering is bypassed entirely before the cap check)

## Architecture Anchors

- `src/analyzer/tls.rs:726-748` -- on_data buffer-append logic with remaining/to_copy cap
- `src/analyzer/tls.rs:29` -- `const MAX_BUF: usize = 65_536`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:726-748` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `remaining = MAX_BUF.saturating_sub(state.buf.len()); to_copy = data.len().min(remaining)`
- **inferred**: no direct test drives the cap; behavior inferred from code structure

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates client_buf or server_buf |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
