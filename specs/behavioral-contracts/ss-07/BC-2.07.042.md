---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-06-29T00:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
lifecycle_status: active
introduced: fix-tls-clienthello-frag
modified:
  - "v1.1: Pass-1 adversarial reconciliation (F-P1-004/SR-005 HIGH) — EC-005: resolve post-done() mid-drain dispatch concern; state explicitly that ClientHello (0x01) and ServerHello (0x02) travel in OPPOSITE directions and cannot coalesce within one direction's record, so done() cannot flip mid-loop from ClientHello+ServerHello coalescing; drain loop breaks when done() is true; classify EC-005 as structurally impossible for the ClientHello/ServerHello coalescing scenario — 2026-06-29"
  - "v1.2: Pass-2 adversarial reconciliation (F-F2-008 MEDIUM / MISMATCH-POST-001) — Precondition 4: removed 'and the direction's carry is not abandoned' clause; the abandoned-direction concept does not exist in the clear-and-recover policy (contradicted BC-2.07.041 Inv-3 / BC-2.07.039 Inv-4); restate PC-4 as 'The flow is not yet done().' only — 2026-06-29"
  - "v1.3: Pass-3 adversarial reconciliation (F-P3-007 MEDIUM) — EC-001: removed false claim that 'second ClientHello overrides first per BC-2.07.001 (duplicate ClientHello semantics)'; BC-2.07.001 specifies NO override postcondition — it only increments handshakes_seen and updates sni_counts/ja3_counts/version_counts; EC-001 rewritten to describe only what is specified: both ClientHellos are dispatched via handle_client_hello; handshakes_seen incremented twice; sni_counts and ja3_counts updated for each — 2026-06-29"
  - "v1.4: Fix burst 9 adversarial reconciliation (F-IMPL-001 MEDIUM) — Inv-1: added third permitted drain-loop exit (c) body_len-spoof (declared body_len > MAX_BUF) clears entire carry and breaks; this is a total-clear followed by break, equivalent to continue since carry is empty after clear; enumeration of partial-drain exits is now exhaustive: (a) carry_buf.len() < 4, (b) next body incomplete, (c) body_len-spoof clear; EC-006 added: complete valid message coalesced with trailing body_len-spoof header — valid message dispatches first (exact-consume), then spoof header clears remaining carry (spoof-only bytes); no valid data lost; EC-007 added: body_len-spoof header PRECEDING valid bytes — total-clears entire carry including valid bytes; acceptable adversarial input; recovery on next well-formed record — 2026-06-29"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.042: Coalesced Handshake Messages in One Record Are Each Dispatched Independently

## Description

When a single TLS 0x16 record payload contains the complete byte sequences of two or
more consecutive handshake messages (e.g., a `ClientHello` immediately followed by a
`Certificate` or another message type), each message is parsed and dispatched in wire
order. The exact-consume invariant from BC-2.07.038 (Invariant 2) causes the carry
drain loop to advance to the next message header after each complete message, so no
message is silently skipped. This handles the RFC-permitted coalescing case (RFC 5246
§6.2.1: "multiple client messages of the same ContentType MAY be coalesced into a
single TLSPlaintext record") without regressing the fragmentation case. This BC
specifies the inverse of fragmentation: the same carry drain loop that handles
fragmentation also handles coalescing, as an emergent property of the exact-consume
design.

## Preconditions

1. `TlsAnalyzer::on_data` has processed a 0x16 record whose payload, when appended to
   the carry buffer, contains at least two complete handshake messages back-to-back.
2. The first message is complete: `carry_buf.len() >= 4 + body_len_1`.
3. After consuming the first message, the remaining bytes form at least one more
   complete handshake message: `remaining.len() >= 4 + body_len_2`.
4. The flow is not yet `done()`. (There is no abandoned-direction state in the
   clear-and-recover policy; a cleared carry is simply empty, not abandoned.)

## Postconditions

1. The carry drain loop dispatches the first complete handshake message (types 0x01 or
   0x02 are dispatched; other types are consumed without dispatching).
2. Immediately after consuming the first message (exact-consume: `4 + body_len_1`
   bytes removed), the loop continues and dispatches the next complete message.
3. All complete messages in the carry buffer at a given drain call are dispatched (or
   advanced past) in wire order before the drain call returns.
4. After all complete messages are dispatched, the carry buffer contains at most 3 bytes
   (a partial handshake header — type byte + 0, 1, or 2 of the 3 length bytes; less
   than a complete 4-byte header cannot be interpreted as a message).
5. No message is dispatched more than once (the drain loop advances past exactly
   `4 + body_len` bytes per iteration; there is no re-scan of already-consumed bytes).
6. `parse_errors` is NOT incremented for any message successfully consumed in the
   coalesced drain (each message is structurally well-formed if the carry drain
   finds a complete header + body).

## Invariants

1. All handshake messages present in the carry buffer at any given drain call are
   dispatched in wire order before the drain call returns. The only permitted
   partial-drain exits are:
   **(a)** `carry_buf.len() < 4` — the handshake header is incomplete; no `body_len` can
   be decoded; loop breaks.
   **(b)** `carry_buf.len() < 4 + body_len` — header is present but the body is not yet
   complete; loop breaks and waits for the next record.
   **(c)** Declared `body_len > MAX_BUF` (body_len-spoof guard, per BC-2.07.038 Inv-5) —
   the entire carry buffer for that direction is cleared and the drain loop breaks. This is
   a total-clear-then-break, which is semantically equivalent to a continue (since the carry
   is empty after the clear). This is the third permitted exit alongside (a) and (b); its
   enumeration here makes the exit list exhaustive. See EC-006 and EC-007 below for the
   two sub-cases of how valid bytes and a spoof header may be coalesced.
2. After all complete messages are dispatched, the carry buffer contains at most 3 bytes
   (partial header — anything shorter than 4 bytes cannot be a complete header, so the
   loop breaks cleanly without a spurious advance).
3. The drain loop is bounded: in the worst case it processes at most
   `carry_buf.len() / 4` iterations per call (each message consumes at least its 4-byte
   header). The loop cannot spin indefinitely even in the presence of zero-length
   messages because the exact-consume removes at least 4 bytes per iteration.
4. If the first message is a `ClientHello` (0x01) and the second is a non-hello type
   (e.g., Certificate 0x0B), the ClientHello is dispatched via `handle_client_hello`
   and the Certificate is consumed without dispatching. This is the primary real-world
   coalescing scenario (RFC 5246 §7.4.2 full handshake: ClientHello alone is in the
   initial flight; coalesced ClientHello+Certificate is atypical but permitted).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Two complete ClientHellos coalesced in one record (protocol anomaly) | Both dispatched via `handle_client_hello`; `handshakes_seen` incremented twice; `sni_counts` and `ja3_counts` updated for each ClientHello per BC-2.07.001 postconditions (PC-1 through PC-4); `version_counts` updated for each. No "override" semantic is specified — BC-2.07.001 defines no postcondition that suppresses or replaces the first ClientHello's contributions to the count maps |
| EC-002 | ClientHello + Certificate coalesced | ClientHello dispatched; Certificate consumed without dispatching; `parse_errors=0` |
| EC-003 | ClientHello + partial next message (coalesced but fragmented) | ClientHello dispatched; partial next message bytes remain in carry buffer; loop exits at `carry_buf.len() < 4 + body_len_N` guard |
| EC-004 | Three complete non-hello handshake messages coalesced (e.g., three Certificates) | All three consumed without dispatching; carry buffer empty after drain; `parse_errors=0` |
| EC-005 | Concern: could `done()` flip mid-drain if ClientHello (0x01) and ServerHello (0x02) are coalesced in the same direction's record? | STRUCTURALLY IMPOSSIBLE. ClientHello is a client-to-server message (direction `ClientToServer`); ServerHello is a server-to-client message (direction `ServerToClient`). A single TLS record carries one direction's payload. It is therefore structurally impossible for a ClientHello and a ServerHello to be coalesced within the same direction's carry buffer or dispatched from the same drain call. Within a single direction's drain: dispatching a ClientHello (0x01) sets `client_hello_seen=true` but cannot set `server_hello_seen=true`; `done()` requires BOTH flags, so it cannot become true mid-drain from this mechanism. Non-hello message types (Certificate, Finished, etc.) are consumed without dispatch and do not touch hello-seen flags. The drain loop does break after the complete-message loop exits (no remaining complete message), not by checking `done()` mid-iteration. |
| EC-006 | A complete valid handshake message is coalesced with a trailing body_len-spoof header in one carry: e.g., carry holds `[valid ClientHello bytes (complete) ++ spoof_header_bytes (declared body_len > MAX_BUF)]` | The drain loop processes the valid ClientHello first (exact-consume `4 + body_len_valid` bytes via the normal dispatch path); `handle_client_hello` is called; `client_hello_seen=true`; SNI and JA3 populated. On the next loop iteration, the spoof header is the next 4-byte header; the body_len-spoof guard (BC-2.07.038 Inv-5) fires: the remaining carry (spoof-only bytes) is cleared; `handshake_reassembly_overflows` incremented; drain breaks. **No valid data is lost.** The valid ClientHello was fully dispatched before the spoof guard fired. |
| EC-007 | A body_len-spoof header PRECEDES valid bytes in one carry: e.g., carry holds `[spoof_header_bytes (declared body_len > MAX_BUF) ++ valid_bytes]` | The body_len-spoof guard fires on the first iteration (the spoof header is the first 4 bytes of the carry); the ENTIRE carry is cleared (including the valid bytes that follow); `handshake_reassembly_overflows` incremented; drain breaks. The valid bytes are lost. This is the accepted adversarial input outcome: the attacker controls byte order and has deliberately placed the spoof header first to discard subsequent valid bytes. Recovery occurs on the next well-formed record. This is bounded and non-permanent (clear-and-recover per BC-2.07.039); the real client will need to retransmit or the flow will be tracked with an incomplete carry — which is standard behaviour. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ClientHello bytes followed immediately by Certificate bytes, both delivered as one 0x16 record | `client_hello_seen=true`; SNI and JA3 populated; Certificate consumed silently; `parse_errors=0`; carry buffer empty after drain | happy-path |
| Two distinct non-hello handshake messages coalesced in one record | Both consumed without dispatch; `parse_errors=0`; carry buffer empty after drain | edge-case |
| ClientHello + incomplete Certificate (partial body) in one record | ClientHello dispatched; Certificate header present but body incomplete; carry holds partial Certificate; `parse_errors=0` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-039 (Sub-B) | Two complete handshake messages coalesced into one record are each dispatched independently; after both records are processed the carry buffer length is 0 | proptest: `proptest_vp039_exact_consume_coalesced` |
| — | No double-dispatch: the drain loop advances past exactly `4 + body_len` bytes per message | unit: `test_BC_2_07_042_exact_consume_no_double_dispatch` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md — correct handling of coalesced handshake messages is required for accurate TLS analysis; missed messages or double-dispatch would corrupt handshakes_seen, ja3_counts, and sni_counts |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs — carry drain loop in `try_parse_records`; exact-consume logic) |
| Finding Source | TLS-CLIENTHELLO-FRAG-001; RFC 5246 §6.2.1 (coalescing explicitly permitted) |
| RFC Authority | RFC 5246 §6.2.1: "multiple client messages of the same ContentType MAY be coalesced into a single TLSPlaintext record" |
| Stories | TBD (F3 STORY-A) |
| Origin | greenfield (fix-tls-clienthello-frag cycle) |

## Related BCs

- BC-2.07.038 — composes with (the exact-consume invariant (Inv-2) is the mechanism that enables correct coalesced dispatch)
- BC-2.07.001 — composes with (ClientHello dispatch is one of the messages that may be coalesced)
- BC-2.07.002 — composes with (ServerHello dispatch; symmetric coalescing applies to server direction)
- BC-2.07.039 — related to (carry buffer overflow; coalesced messages that individually fit in the buffer do not trigger overflow)
- BC-2.07.041 — depends on (per-direction isolation; coalesced messages on one direction do not affect the other)

## Architecture Anchors

- `src/analyzer/tls.rs` — carry drain loop in `try_parse_records`: `loop { if carry_buf.len() < 4 { break; } let body_len = ...; if carry_buf.len() < 4 + body_len { break; } dispatch_or_consume(); carry_buf.drain(0..4+body_len); }`
- `tests/tls_analyzer_tests.rs` — `proptest_vp039_exact_consume_coalesced`
- `tests/tls_analyzer_tests.rs` — `test_BC_2_07_042_exact_consume_no_double_dispatch`

## Story Anchor

TBD (F3 STORY-A)

## VP Anchors

VP-039 (Sub-B)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates carry buffer; calls `handle_client_hello` / `handle_server_hello` as needed |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
