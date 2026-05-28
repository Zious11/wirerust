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
  - "v1.3: Wave 16 Pass-1 prose fix (F-W16-S052-P1-004) — line citation corrected: done check at tls.rs:721, early return at tls.rs:723 — 2026-05-28"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.003: After Both Hellos Seen, Subsequent Records Are Silently Skipped

## Description

Once a flow's `TlsFlowState.done()` returns true (both `client_hello_seen` and
`server_hello_seen` are true), any further `on_data` call for that flow immediately
returns without buffering data or incrementing any counter. Subsequent TLS records
(including application-data, change-cipher-spec, alerts, and any late-arriving
handshake fragments) are all discarded silently. No `parse_errors` or any other
counter is incremented.

## Preconditions

1. The flow's `TlsFlowState` exists in the `flows` HashMap.
2. Both `client_hello_seen == true` and `server_hello_seen == true`.
3. `TlsFlowState::done()` returns true.

## Postconditions

1. `on_data` returns immediately (tls.rs:723) once the `done` check (tls.rs:721) is true.
2. No bytes are appended to `client_buf` or `server_buf`.
3. No counters are incremented (`parse_errors`, `handshakes_seen`, etc. all unchanged).
4. No findings are emitted.
5. The `TlsFlowState` record remains in the `flows` HashMap (not removed; that only
   happens in `on_flow_close`).

## Invariants

1. The `done()` check is the first operation in `on_data` after the HashMap lookup.
2. The short-circuit is permanent: once `done()` is true for a flow, no future call
   can re-enter processing for that flow.
3. Application-data records (type 0x17) do not trigger any parse counter because they
   are never reached after the short-circuit.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Large application-data record (e.g. HTTPS response body, 50 KB) arrives after both hellos | Entire chunk discarded; no parse_errors increment |
| EC-002 | A second (retransmitted) ClientHello arrives after both hellos are seen | Discarded; handshakes_seen NOT incremented again |
| EC-003 | on_data called with empty slice (data.len() == 0) after hellos done | Returns immediately; no effect |
| EC-004 | Flow with only ClientHello seen (server_hello_seen = false) | Short-circuit does NOT fire; bytes are buffered normally |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ClientHello bytes then ServerHello bytes then 100 bytes of application data | handshakes_seen=1; parse_errors=0; all_findings from hellos only | happy-path |
| ClientHello then ServerHello then malformed handshake record | parse_errors=0 (malformed bytes discarded before parse attempt) | edge-case |
| Only ClientHello seen, then more data arrives | Data is buffered/parsed; server_hello_seen remains false | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | parse_errors stays 0 after application data post-done | unit: test_stop_after_handshake |
| — | handshakes_seen not incremented by post-done ClientHello | unit: assert count after re-sending ClientHello bytes |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- the done-short-circuit is the resource-bounding mechanism that prevents unbounded post-handshake buffering |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation -- raw bytes are not stored after done) |
| Architecture Module | SS-07 (analyzer/tls.rs:718-724, C-13) |
| Stories | STORY-052 |
| Origin BC | BC-TLS-003 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.034 -- composes with (same short-circuit, viewed from on_data level)
- BC-2.07.001 -- depends on (ClientHello must have been processed first)
- BC-2.07.002 -- depends on (ServerHello must have been processed first)

## Architecture Anchors

- `src/analyzer/tls.rs:718-724` -- `on_data` done-check and early return
- `src/analyzer/tls.rs:290-293` -- `TlsFlowState::done()` predicate
- `tests/tls_analyzer_tests.rs` -- test_stop_after_handshake

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:718-724` (on_data done guard) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `let done = self.flows.get(flow_key).is_some_and(|s| s.done()); if done { return; }`
- **assertion**: test_stop_after_handshake confirms no parse_errors after application data

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads flows HashMap only when done |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (when done path taken) |
