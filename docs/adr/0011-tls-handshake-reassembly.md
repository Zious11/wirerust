---
document_type: adr
adr_id: ADR-011
status: accepted
accepted_date: "2026-06-29"
date: 2026-06-29
modified:
  - "Pass-1 adversarial reconciliation: Decision 5 renamed abandon-direction→clear-and-recover; added handshake_reassembly_overflows counter; Decision 4 extended with per-message body_len guard; SR-012 carry-append placement added."
  - "Pass-2 adversarial reconciliation: handshake_reassembly_overflows moved from TlsFlowState to TlsAnalyzer aggregate (mirrors truncated_records; NOT dropped at on_flow_close)."
  - "Fix-burst-6: parse_tls_message_handshake chosen as parse boundary; malformed-but-complete assembled body error semantics specified; VP-039 Sub-C overflow-fixture corrected."
  - "Fix-burst-7: Decision-4 Ok(_) arm annotated UNREACHABLE given outer msg_type guard; BC-2.07.038 PC-9."
  - "F2 scope-addition (BC-2.07.043): buffer_saturation_drops counter added to TlsAnalyzer; surfaced in summarize() detail['buffer_saturation_drops']; VP-040 registered."
  - "Adversary fix burst: buffer_saturation_drops increment condition corrected to 'data.len() > remaining' (covers partial-drop AND full-drop)."
subsystems_affected:
  - SS-07
supersedes: null
superseded_by: null
feature_cycle: fix-tls-clienthello-frag
issue: fix-tls-clienthello-frag
---

# ADR-011: TLS Handshake-Message Reassembly Across Record Boundaries

> **One-per-file:** Each architectural decision lives in its own file.
> Filename convention: `ADR-NNN-<short-name>.md`.
> ADR IDs are sequential 3-digit (ADR-001, ADR-002, ...). Once issued, never renumber.
> Lifecycle: `proposed` -> `accepted` -> (optional) `superseded` or `deprecated`.

## Context

### RFC-Permitted Fragmentation

RFC 5246 §6.2.1 explicitly permits TLS handshake messages to be fragmented across multiple
consecutive TLS records with content type 0x16 (Handshake):

> "Handshake messages can be fragmented across multiple TLSPlaintext records or can be
> coalesced into a single TLSPlaintext record."

A TLS ClientHello commonly spans 200–600 bytes. An adversary or any compliant TLS
implementation can split the ClientHello across two or more 0x16 records, placing the SNI
extension (and JA3-relevant cipher/extension fields) in a later record. Prior to this ADR,
wirerust's `try_parse_records` dispatched `handle_client_hello` only on the first 0x16 record
encountered. A fragmented ClientHello whose first record delivered fewer than the full
handshake message bytes was silently dropped without detection.

### Evasion Consequence

When a TLS ClientHello is fragmented, wirerust produced no finding, no SNI entry, and no JA3
hash — identical to an opaque TLS flow. Adversaries using known published techniques
(F5 BIG-IP TLS fragmentation bypass, Palo Alto NGFW TLS fragmentation bypass) can fragment
the ClientHello to evade SNI-based detection. The research-validated
finding for this cycle is documented in
`.factory/research/TLS-CLIENTHELLO-FRAG-001-validation.md`.

### Existing Architecture Constraints

The current TLS analyzer (`src/analyzer/tls.rs`) is structured around:

1. **TLS record layer** (`try_parse_records`): iterates over complete TLS records in the
   current TCP segment buffer (`client_buf` / `server_buf`).
2. **Application layer dispatch** (`handle_client_hello`, `handle_server_hello`): receives
   fully parsed `TlsClientHelloContents` / `TlsServerHelloContents`.
3. **Per-flow state** (`TlsFlowState`): holds `client_buf` / `server_buf` (TCP segment carry,
   capped at `MAX_BUF` = 65,536 bytes) and `done()` gate.

The existing `client_buf` / `server_buf` carry buffers reassemble TCP segment fragmentation.
They do **not** reassemble TLS record fragmentation — a case where the TLS record is complete
but the handshake message spans multiple TLS records.

### Design Space

Three alternatives were considered:

**Option A (Rejected): Full TLS State Machine** — implement a complete TLS state machine
tracking cipher negotiation, record epoch, and key material. Rejected: scope is
disproportionate; encrypted records cannot be parsed without key material; hundreds of new
code lines and new attack surface.

**Option B (Rejected): Reject Fragmented Hellos as Malformed** — treat a 0x16 record that
does not contain a complete handshake message header as a parse error. Rejected: RFC 5246
explicitly permits fragmentation; this would produce false parse errors and permanently
exclude compliant TLS stacks from detection.

**Option C (Accepted): Bounded Per-Direction Carry Buffer with Exact-Consume** — introduce
two new carry buffer fields in `TlsFlowState` to accumulate 0x16 record payloads across
calls to `try_parse_records`. Cap at `MAX_BUF`; on overflow, clear and recover. This is
Option C and is the adopted design.

## Decision

### Decision 1 — New Fields: TlsFlowState (2 carry bufs) and TlsAnalyzer (2 aggregate counters)

Add two new fields to `TlsFlowState`:

```rust
pub struct TlsFlowState {
    // ... existing fields ...
    pub(crate) client_hs_carry: Vec<u8>,   // handshake fragment carry (ClientToServer)
    pub(crate) server_hs_carry: Vec<u8>,   // handshake fragment carry (ServerToClient)
}
```

Both carry fields initialize to `Vec::new()` in `TlsFlowState::new()` and are automatically
dropped when `on_flow_close` removes the `TlsFlowState` from the `flows` map.

Add two new aggregate counters to `TlsAnalyzer` (alongside the existing `truncated_records`):

```rust
pub struct TlsAnalyzer {
    // ... existing fields ...
    handshake_reassembly_overflows: u64,  // aggregate; NOT per-flow; NOT reset at on_flow_close
    buffer_saturation_drops: u64,         // F2 scope: increments when on_data tail-drops bytes
                                          // because client_buf/server_buf is at MAX_BUF capacity
                                          // increment condition: data.len() > remaining
                                          // surfaced in summarize() detail["buffer_saturation_drops"]
}
```

`handshake_reassembly_overflows` accumulates across all flows and is NOT reset at flow close,
mirroring the existing `truncated_records` aggregate counter. `buffer_saturation_drops`
accumulates at the TCP-segment buffer layer (`client_buf`/`server_buf`) — a distinct layer
from the handshake-carry layer (`client_hs_carry`/`server_hs_carry`) tracked by
`handshake_reassembly_overflows`.

Public accessors follow the existing `truncated_record_count()` / `parse_error_count()` pattern:

```rust
pub fn handshake_reassembly_overflow_count(&self) -> u64 { self.handshake_reassembly_overflows }
pub fn buffer_saturation_drop_count(&self) -> u64 { self.buffer_saturation_drops }
```

### Decision 2 — Reassembly Only for Content Type 0x16

Carry buffer accumulation applies **only** to TLS records with content type `0x16`
(Handshake). Content types `0x14` (ChangeCipherSpec), `0x15` (Alert), `0x17`
(ApplicationData), and `0x18` (Heartbeat) never feed the carry buffer. Application data
records are encrypted and not parsed; appending them to a carry buffer would waste memory
and produce no useful result.

### Decision 3 — Single-Record Fast-Path Preservation

The reassembly layer sits between record-drain and handshake parse in `try_parse_records`.
The single-record fast-path (a 0x16 record whose payload already contains a complete
handshake message) is behaviorally identical to the carry-buffer path: the record payload is
appended to the (empty) carry buffer, the consume loop immediately finds a complete message,
dispatches it, and drains the carry to empty. All existing `tls_analyzer_tests.rs` tests
exercise this path and must remain green (regression gate; VP-039 Sub-A).

### Decision 4 — Exact-Consume Loop

After appending a 0x16 record payload to the carry buffer, the inner consume loop:

```
loop {
    if carry_buf.len() < 4 { break; }          // incomplete header — wait
    let body_len = u24_be(&carry_buf[1..4]) as usize;

    // Per-message body_len guard: body_len > MAX_BUF (65,536) means adversarial header.
    // Clear-and-recover: carry_buf.clear(), handshake_reassembly_overflows++; break.
    if body_len > MAX_BUF { carry_buf.clear(); analyzer.handshake_reassembly_overflows += 1; break; }

    if carry_buf.len() < 4 + body_len { break; }  // incomplete body — wait
    let msg_type = carry_buf[0];
    let msg_bytes = &carry_buf[..4 + body_len];
    match msg_type {
        0x01 | 0x02 => {
            // parse_tls_message_handshake: correct function for assembled handshake bytes
            // (no TLS record header; consume exactly 4 + body_len bytes).
            match parse_tls_message_handshake(msg_bytes) {
                Ok((_, TlsMessage::Handshake(TlsMessageHandshake::ClientHello(ch)))) => { ... }
                Ok((_, TlsMessage::Handshake(TlsMessageHandshake::ServerHello(sh)))) => { ... }
                Ok(_) | Err(_) => { self.parse_errors += 1; }
                // Ok(_) is UNREACHABLE in practice: msg_type 0x01/0x02 either parses as
                // ClientHello/ServerHello or errors; included for match exhaustiveness only.
            }
        }
        _ => { /* non-hello message type — consume and skip silently */ }
    }
    carry_buf.drain(..4 + body_len);  // exact-consume
}
```

**Parse function choice:** `parse_tls_message_handshake` is the correct function. The carry
buffer holds assembled handshake message bytes — a 1-byte type field, a 3-byte big-endian
`uint24` body length, and `body_len` body bytes — with no TLS record header. This function
consumes exactly `4 + body_len` bytes, requires no synthetic header construction, and returns
the same `TlsMessage` variants matched by the existing single-record path.

**Malformed-but-complete body semantics:** When `parse_tls_message_handshake` returns
`Err(_)` on a length-complete message, `parse_errors` increments by 1 (parity with the
single-record path). The message is still exact-consumed. No finding is emitted.

### Decision 5 — Bounded Carry: Clear-and-Recover Overflow Policy

When a new 0x16 record would push `carry_buf.len() + record_payload_len > MAX_BUF`:

```rust
if carry_buf.len() + record_payload_len > MAX_BUF {
    carry_buf.clear();
    analyzer.handshake_reassembly_overflows += 1;
    // do NOT increment parse_errors
    // do NOT emit a finding
    // do NOT set any abandoned flag
    continue;  // to next record in the outer loop
}
```

**Rationale for clear-and-recover over sticky-abandon:**

1. **Evasion-resistance:** A sticky abandon flag is a one-packet, permanent,
   attacker-triggerable blinding primitive. Clear-and-recover denies this permanence
   (Ptacek/Newsham 1998 desynchronization-to-evasion; Suricata CVE-2019-18792).
2. **Industry alignment:** Passive analyzers (Wireshark) and stateful ones (Suricata,
   Zeek) favor "curtail inspection / signal anomaly / keep the flow."
3. **Consistency:** `src/analyzer/tls.rs` already implements clear-and-recover for
   `MAX_RECORD_PAYLOAD` over-size at the TCP-segment buffer layer.
4. **Observability:** `handshake_reassembly_overflows` converts repeated overflows
   into detectable telemetry rather than a silent blind spot.
5. **Recovery:** A subsequent well-formed 0x16 record re-populates the now-empty carry.

`parse_errors` is NOT incremented on carry overflow. An oversized partial handshake is
adversarial or unusual input, not a TLS record framing failure. Evidence basis:
`.factory/research/TLS-REASSEMBLY-OVERFLOW-POLICY.md`.

### Decision 6 — Truncation Safety at Flow Close

When `on_flow_close` is called with a partial carry buffer, the carry is silently discarded
via `self.flows.remove(flow_key)`. No finding is emitted and `parse_errors` is NOT
incremented. A snaplen-truncated capture and a legitimately fragmented-but-incomplete
handshake are byte-identical at the `on_flow_close` call site; both receive the same
"discard silently" treatment.

### Decision 7 — Reassembly Only Pre-`done()`

Carry buffer accumulation only occurs for records delivered before `TlsFlowState::done()`
returns `true`. Once `done()` is true (both `client_hello_seen` and `server_hello_seen`),
`try_parse_records` exits early. This automatically prevents buffering post-key-change 0x16
records (which are encrypted application data in TLS 1.3).

### Decision 8 — Test Seams

Two new test seams on `TlsFlowState` follow the existing `client_buf_len_for_testing()` /
`server_buf_len_for_testing()` pattern:

```rust
#[doc(hidden)]
pub fn client_hs_carry_len_for_testing(&self) -> usize { self.client_hs_carry.len() }

#[doc(hidden)]
pub fn server_hs_carry_len_for_testing(&self) -> usize { self.server_hs_carry.len() }
```

### Decision 9 — DTU Re-Assessment

NOT REQUIRED. This decision introduces no new external service dependencies. wirerust is a
passive offline analyzer; the carry buffers are purely in-process state.

## Consequences

### Positive

- SNI and JA3/JA3S are now extracted from fragmented ClientHellos and ServerHellos.
  The evasion class documented in `TLS-CLIENTHELLO-FRAG-001-validation.md` is closed.
- The single-record fast path is unaffected. All existing `tls_analyzer_tests.rs` tests
  remain green.
- `buffer_saturation_drops` makes the previously silent TCP-segment buffer tail-drop
  observable; operators can distinguish "no TLS data" from "TCP-segment buffer saturated."
- Memory ceiling: two new carry buffers add at most `2 × MAX_BUF = 128 KiB` per flow.
  Total POST-on_data-return per-flow residue ceiling increases from `2 × MAX_BUF` to
  `4 × MAX_BUF` (256 KiB).
- Minimal code surface: change confined to `src/analyzer/tls.rs` — 2 new `TlsFlowState`
  fields, 2 new `TlsAnalyzer` aggregate counters, 2 accessors, 1 new code block in
  `try_parse_records` (~30 lines), 1 `buffer_saturation_drops` increment in `on_data`,
  2 new test seams.

### Negative / Risks

- Per-flow memory increases by up to `2 × MAX_BUF` per TLS flow. Mitigated by the
  clear-and-recover cap (Decision 5) and the existing `max_flows` / `memcap` configuration.
- Single-record fast-path regression risk: off-by-one in the exact-consume loop could
  mis-dispatch a previously-working single-record ClientHello. Mitigation: all existing
  `tls_analyzer_tests.rs` tests are regression guards (VP-039 Sub-A).
- `parse_errors` accounting discipline: the new block must not increment `parse_errors`
  for carry overflow, `body_len > MAX_BUF`, or truncation at flow close.

## ADR Cross-References

| ADR | Relationship |
|-----|-------------|
| ADR-0001 | Stream dispatch (content-first): TLS 0x16 0x03 fingerprint triggers TLS classification. Reassembly operates after classification; ADR-0001 is unaffected. |
| ADR-0002 | Modular analyzer pattern (two-trait split): `TlsAnalyzer` still implements `StreamAnalyzer`; the `on_data` signature is unchanged. |
| ADR-009 | pcapng reader (snaplen truncation): Decision 6 of this ADR resolves the interaction between READER cand-05 (`captured_len < original_len`) and partial carry at flow close. |

## Subsystems Affected

- **SS-07 (TLS Analysis):** Primary change. `TlsFlowState` struct, `TlsAnalyzer` struct,
  `try_parse_records`, `on_data`, `on_flow_close`, `summarize()`, new test seams.
  New BCs: BC-2.07.038 through BC-2.07.043.
- **No other subsystems affected.** `src/analyzer/tls.rs` is the only file that changes.
  `FlowKey` keying, `StreamDispatcher`, `TcpReassembler`, `Finding` struct, and all
  reporters are unchanged.
