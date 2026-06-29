---
document_type: adr
adr_id: ADR-011
status: accepted
accepted_date: "2026-06-29"
date: 2026-06-29
modified:
  - date: 2026-06-29
    actor: architect
    reason: "Pass-1 adversarial reconciliation (F-P1-008/SR-002/SR-012 fixes): Decision 5 renamed abandon-direction→clear-and-recover; added handshake_reassembly_overflows counter; confirmed no hs_carry_abandoned flag; updated Decision 1 struct description to include counter field; Decision 4 extended with per-message body_len guard (body_len > MAX_BUF triggers clear-and-recover); SR-012 carry-append placement sentence added; Per-message cap rationale updated to 65,536 (was MAX_RECORD_PAYLOAD)."
  - date: 2026-06-29
    actor: architect
    reason: "Pass-2 adversarial reconciliation (F-F2-001/F-F2-002): handshake_reassembly_overflows moved from TlsFlowState to TlsAnalyzer (aggregate counter, mirrors truncated_records; NOT dropped at on_flow_close). TlsFlowState Decision 1 struct now has only 2 new fields (client_hs_carry + server_hs_carry). Decision 4/5 pseudocode updated: increment target is self.handshake_reassembly_overflows on the TlsAnalyzer (not TlsFlowState). on_flow_close note updated: only carry bufs are dropped; the aggregate counter accumulates across all flows."
  - date: 2026-06-29
    actor: architect
    reason: "Fix-burst-6 (F-FRESH-001/F-CRITICAL-2/F-F2P-IMP-001/F-FRESH-002): Decision 4 — parse boundary specified: parse_tls_message_handshake (tls_parser 0.12.2) is the correct function for assembled handshake message bytes (no record header); import added; error semantics for malformed-but-complete assembled body specified (parse_errors++, consume bytes, no finding, no panic); new VP-039 malformed-assembled-body test named. Sub-C overflow-fixture corrected: 0xCC fill hits body_len-spoof path (body_len=0xCCCCCC>>MAX_BUF) on record 1, not buffer-fill path; replaced with valid header body_len=65,500 + accumulation records to trigger Decision-5 buffer-fill guard with counter==overflows_before+1. Sub-F proptest generator restructured: valid header prefix ensures carry actually accumulates (not vacuous on first record). Frame C (dispatch lane) added to canonical-frame test."
  - date: 2026-06-29
    actor: architect
    reason: "Fix-burst-7 (O-1/residue-qualifier): Decision-4 Ok(_) arm annotated as UNREACHABLE — given the outer match msg_type { 0x01 | 0x02 => ... } guard, parse_tls_message_handshake is only called for type bytes 0x01 (ClientHello) and 0x02 (ServerHello); an Ok(_) result for these types would only arise if the parser returned a non-ClientHello/ServerHello variant from a 0x01/0x02 byte, which is impossible per the tls-parser grammar; the Ok(_) arm is grouped with Err(_) only for match exhaustiveness; BC-2.07.038 PC-9. Consequences section: per-flow memory ceiling note updated with residue qualifier — 4×MAX_BUF is the POST-on_data-return residue ceiling; the in-call transient peak is higher due to record_bytes clone; per BC-2.07.039 v2.3 Inv-2."
  - date: 2026-06-29
    actor: architect
    reason: "Fix-burst-9 (F-IMPL-001 clarification): Decision 4 body_len-spoof guard expanded with explicit TOTAL-CLEAR SEMANTICS note (carry_buf.clear() removes ALL bytes in the direction carry, not just the bad header), WHY-break-IS-SAFE note (break ≡ continue because carry is now empty — len()==0 — so next iteration would immediately break on the <4 guard anyway), DISTINCTION FROM DECISION 5 note (Decision 5 uses continue because it fires pre-append in the outer record loop; Decision 4 guard fires inside the drain loop post-clear; same outcome, different loop positions), and ORDERING note (valid message preceding spoof header is dispatched and exact-consumed before the spoof is hit, so no valid data is lost; spoof header preceding valid bytes discards trailing bytes — accepted adversarial-input tradeoff, recovery on next record)."
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

RFC 5246 §6.2.1 explicitly permits TLS handshake messages to be fragmented across
multiple consecutive TLS records with content type 0x16 (Handshake):

> "Handshake messages can be fragmented across multiple TLSPlaintext records or
> can be coalesced into a single TLSPlaintext record."

A TLS ClientHello commonly spans 200–600 bytes. An adversary or any compliant TLS
implementation can split the ClientHello across two or more 0x16 records, placing
the SNI extension (and JA3-relevant cipher/extension fields) in the second or later
record. Prior to this fix, wirerust's `try_parse_records` dispatched `handle_client_hello`
only on the first 0x16 record it encountered. A fragmented ClientHello whose first
record delivered fewer than the full handshake message bytes was silently dropped
without detection.

### Evasion Consequence

When a TLS ClientHello is fragmented, wirerust produced no finding, no SNI entry, and
no JA3 hash — identical to an opaque TLS flow. An adversary using known published
techniques (Kubernetes ingress-nginx CVE-2021-25742, F5 BIG-IP fragmentation bypass,
Palo Alto NGFW bypass) can fragment the ClientHello to evade SNI-based detection.

The research-validated finding for this cycle is documented in
`.factory/research/TLS-CLIENTHELLO-FRAG-001-validation.md`.

### Existing Architecture Constraints

The current TLS analyzer (`src/analyzer/tls.rs`) is structured around:

1. **TLS record layer** (`try_parse_records`): iterates over complete TLS records
   in the current TCP segment buffer (`client_buf` / `server_buf`).
2. **Application layer dispatch** (`handle_client_hello`, `handle_server_hello`):
   receives fully parsed `TlsClientHelloContents` / `TlsServerHelloContents`.
3. **Per-flow state** (`TlsFlowState`): holds `client_buf` / `server_buf` (TCP
   segment carry, capped at MAX_BUF = 65,536 bytes) and `done()` gate.

The existing `client_buf` / `server_buf` carry buffers reassemble TCP segment
fragmentation (cases where TCP segments arrive in partial chunks). They do NOT
reassemble TLS record fragmentation — that is, a case where the TLS record is
complete (all `record_length` bytes present) but the handshake message spans
multiple TLS records.

### Design Space

Three alternatives were considered:

**Option A (Rejected): Full TLS State Machine**
Implement a complete TLS state machine tracking cipher negotiation, record epoch,
and key material. This would handle all TLS framing correctly including encrypted
records, post-handshake messages, and key updates.

Rejected because: (1) The scope is disproportionate to the detection goal — wirerust
extracts SNI and JA3/JA3S from the unencrypted ClientHello and ServerHello only;
(2) Encrypted records cannot be parsed without key material; (3) A full TLS state
machine introduces hundreds of new code lines, new dependencies, and new attack
surface; (4) The `done()` gate already stops buffering after both hellos are seen.

**Option B (Rejected): Reject Fragmented Hellos as Malformed**
Treat a 0x16 record that does not contain a complete handshake message header (4 bytes:
type + 3-byte big-endian body length) as a parse error, incrementing `parse_errors`.

Rejected because: RFC 5246 explicitly permits fragmentation; treating standard-compliant
input as malformed would produce false parse errors on any compliant TLS stack that
fragments, and would permanently exclude those flows from detection. This is the
opposite of the desired behavior.

**Option C (Accepted): Bounded Per-Direction Carry Buffer with Exact-Consume**
Introduce two new fields (`client_hs_carry: Vec<u8>` and `server_hs_carry: Vec<u8>`)
in `TlsFlowState` to accumulate 0x16 record payloads across calls to `try_parse_records`.
When a complete handshake message (4-byte header declares `body_len`; `carry_len >= 4 +
body_len`) is accumulated, dispatch to the existing `handle_client_hello` /
`handle_server_hello`. Exact-consume: advance the carry buffer by exactly `4 + body_len`
bytes after each dispatch. Cap the carry buffer at MAX_BUF; on overflow, clear and
recover — clear that direction's carry, increment `handshake_reassembly_overflows`
counter, continue (no parse error, no finding, no sticky state; recovery permitted).

Accepted because: (1) minimal code addition (one new code block in `try_parse_records`,
two new fields in `TlsFlowState` — two carry buffers; one new aggregate counter on
`TlsAnalyzer` — `handshake_reassembly_overflows`); (2) no
new dependencies; (3) pure-core logic (deterministic, no I/O); (4) satisfies RFC 5246;
(5) maintains all existing BCs (BC-2.07.001 through BC-2.07.037 are unaffected;
`handle_client_hello` receives the same assembled bytes regardless of fragmentation);
(6) clear-and-recover overflow policy is more evasion-resistant than sticky-abandon
(Policy B rejected — see Decision 5 and `.factory/research/TLS-REASSEMBLY-OVERFLOW-POLICY.md`).

## Decision

### Decision 1 — New Fields: TlsFlowState (2 carry bufs) and TlsAnalyzer (1 aggregate counter)

Add two new fields to `TlsFlowState`:

```rust
pub struct TlsFlowState {
    // ... existing fields ...
    pub(crate) client_hs_carry: Vec<u8>,   // handshake fragment carry (ClientToServer)
    pub(crate) server_hs_carry: Vec<u8>,   // handshake fragment carry (ServerToClient)
    // NOTE: handshake_reassembly_overflows is NOT here — it is an aggregate counter on TlsAnalyzer
}
```

Both carry fields are initialized to `Vec::new()` in `TlsFlowState::new()`.
Both carry fields are automatically dropped when `on_flow_close` removes the
`TlsFlowState` from the `flows: HashMap<FlowKey, TlsFlowState>` map (no explicit
clear needed).

Add one new aggregate counter to `TlsAnalyzer` (alongside the existing `truncated_records` counter):

```rust
pub struct TlsAnalyzer {
    // ... existing fields ...
    handshake_reassembly_overflows: u64,  // aggregate across ALL flows; mirrors truncated_records
}
```

`handshake_reassembly_overflows` is initialized to `0` in `TlsAnalyzer::new()`.
It is an AGGREGATE counter: it accumulates across all flows and is NOT dropped when
a flow is closed. This mirrors the existing `truncated_records` field which is also
a `TlsAnalyzer`-level aggregate, NOT a per-flow `TlsFlowState` field.

A public accessor is added following the existing `truncated_record_count()` /
`parse_error_count()` pattern:

```rust
pub fn handshake_reassembly_overflow_count(&self) -> u64 {
    self.handshake_reassembly_overflows
}
```

There is NO `hs_carry_abandoned` flag and no `*_abandoned` bool field of any kind in
either `TlsFlowState` or `TlsAnalyzer`. The concept of permanently abandoning a
direction does not exist in this design; see Decision 5 for the clear-and-recover policy.

### Decision 2 — Reassembly Only for Content Type 0x16

The carry buffer accumulation applies ONLY to TLS records with content type `0x16`
(Handshake). Content types `0x14` (ChangeCipherSpec), `0x15` (Alert), `0x17`
(ApplicationData), and `0x18` (Heartbeat) never feed the carry buffer. The existing
guard-before-allocate path for non-0x16 records (CR-010) is unaffected.

Rationale: Reassembly is only needed for handshake messages. Application data records
are encrypted and not parsed; appending them to a carry buffer would waste memory and
produce no useful result.

### Decision 3 — Single-Record Fast-Path Preservation

The reassembly layer sits between record-drain and handshake parse in
`try_parse_records`. The single-record fast-path (a 0x16 record whose payload already
contains a complete handshake message) is behaviorally identical to the carry-buffer
path: the record payload is appended to the carry buffer (which was empty), the
consume loop immediately finds a complete message, dispatches it, and drains the carry
to empty. The carry buffer is empty at the end of the call in both paths.

This equivalence is explicitly tested by VP-039 Sub-A (the single-record baseline)
and is a regression guarantee: all 9,391 lines of existing `tls_analyzer_tests.rs`
continue to exercise this path and must remain green.

### Decision 4 — Exact-Consume Loop

After appending a 0x16 record payload to the carry buffer, the inner consume loop:

```
loop {
    if carry_buf.len() < 4 { break; }  // incomplete header — wait
    let body_len = u24_be(&carry_buf[1..4]) as usize;

    // Per-message body_len guard (BC-2.07.038 Invariant 5; SR-002 fix):
    // If body_len > MAX_BUF (65,536), the header declares an adversarially large
    // message — clear-and-recover (same policy as the buffer-fill overflow guard
    // in Decision 5).  This fires BEFORE the buffer-fill check on the next append.
    //
    // TOTAL-CLEAR SEMANTICS (F-IMPL-001 clarification):
    // carry_buf.clear() removes EVERY byte from this direction's carry — including
    // the 4-byte spoof header AND any bytes that were already in the carry before
    // this drain iteration (e.g., bytes belonging to a partial next-message or to
    // trailing coalesced messages that followed the spoof header in the stream).
    // This is an intentional total clear, not a selective removal of the bad header.
    //
    // WHY `break` IS SAFE HERE (break ≡ continue for this guard):
    // After carry_buf.clear(), the carry buffer is empty — len() == 0. Whether the
    // code executes `break` (exit the drain loop) or `continue` (start the next
    // iteration), the next iteration would immediately hit `carry_buf.len() < 4`
    // and break anyway. `break` is used to make the intent explicit: "there is
    // nothing left to drain." The two control-flow choices are behaviorally
    // identical because the clear is total.
    //
    // DISTINCTION FROM DECISION 5 (`continue` vs `break`):
    // Decision 5 (buffer-fill guard) uses `continue` because it fires BEFORE the
    // carry append — the incoming record_payload_len would overflow, so the carry
    // is cleared and the loop advances to the NEXT 0x16 record in the outer loop.
    // The carry may legitimately hold valid in-progress bytes from prior iterations
    // of the outer record loop (though after a clear they are gone), and future
    // records can still be appended. `continue` signals "skip this record, keep
    // processing outer-loop records."
    // This guard (Decision 4 body_len guard) fires INSIDE the drain loop after the
    // carry already holds the spoof header. The carry is cleared and there is
    // nothing in the carry to process — `break` signals "exit the drain loop now."
    // Both guards produce the same clear-and-recover outcome (carry cleared,
    // handshake_reassembly_overflows++, no parse_errors, no finding); only the
    // loop control differs because they fire at different points in the data flow.
    //
    // ORDERING: VALID MESSAGE PRECEDING A SPOOF HEADER IN THE SAME CARRY:
    // If a valid complete handshake message (e.g., a well-formed ClientHello)
    // arrives in the carry during an earlier iteration of this drain loop, it is
    // dispatched and exact-consumed (drain(..4 + body_len)) BEFORE the spoof
    // header is encountered. When the loop then reads the spoof header and fires
    // this guard, the carry contains only the spoof-header bytes (and any trailing
    // bytes after it). The total clear discards only what remains — no valid
    // already-dispatched data is affected.
    // Conversely, if a spoof header PRECEDES valid bytes in the carry (adversarial
    // coalescing: spoof header first, then a real handshake message), this guard
    // fires on the spoof header and the total clear discards the trailing valid
    // bytes as well. This is the accepted tradeoff: adversarial input that
    // coalesces a spoofed header with real data is treated as fully adversarial,
    // and recovery occurs on the next 0x16 record arrival (Decision 5 / Decision 3
    // single-record equivalence).
    if body_len > MAX_BUF {
        carry_buf.clear();
        analyzer.handshake_reassembly_overflows += 1;  // TlsAnalyzer aggregate counter (NOT TlsFlowState)
        break;  // nothing more to dispatch from this (now-cleared) carry
    }

    if carry_buf.len() < 4 + body_len { break; }  // incomplete body — wait
    let msg_type = carry_buf[0];
    let msg_bytes = &carry_buf[..4 + body_len];  // full message: header (4) + body (body_len)
    match msg_type {
        0x01 | 0x02 => {
            // PARSE BOUNDARY (F-FRESH-001): The carry contains assembled handshake message
            // bytes only — NO TLS record header (content type + version + length).
            // parse_tls_message_handshake takes these raw message bytes and produces
            // TlsMessage::Handshake(TlsMessageHandshake::ClientHello(...)) or ServerHello.
            // Import: tls_parser::parse_tls_message_handshake (already exported from lib.rs).
            match parse_tls_message_handshake(msg_bytes) {
                Ok((_, TlsMessage::Handshake(TlsMessageHandshake::ClientHello(ch)))) => {
                    // set client_hello_seen flag on TlsFlowState (same as single-record path)
                    state.client_hello_seen = true;
                    self.handle_client_hello(&ch, flow_key, last_ts);
                }
                Ok((_, TlsMessage::Handshake(TlsMessageHandshake::ServerHello(sh)))) => {
                    state.server_hello_seen = true;
                    self.handle_server_hello(&sh, flow_key, last_ts);
                }
                Ok(_) | Err(_) => {
                    // MALFORMED-BUT-COMPLETE BODY SEMANTICS (F-FRESH-001):
                    // An assembled, length-complete handshake body that fails to parse
                    // as a valid ClientHello/ServerHello (e.g., truncated extensions,
                    // out-of-range version) increments parse_errors by exactly 1.
                    // The message bytes are still exact-consumed (drain below).
                    // No finding is emitted. No panic. No sticky state.
                    //
                    // Rationale: this is parity with the single-record parse path
                    // (tls.rs L787-789: Err(_) => self.parse_errors += 1).
                    // The body was framed correctly (header length matched); the failure
                    // is in the inner ClientHello/ServerHello structure, not in TLS
                    // record framing. parse_errors discipline: BC-2.07.040 Inv-1
                    // (carry overflow does NOT touch parse_errors; this path does).
                    //
                    // O-1 UNREACHABILITY NOTE (Fix-burst-7, BC-2.07.038 PC-9):
                    // The `Ok(_)` arm is UNREACHABLE in practice given the outer
                    // `match msg_type { 0x01 | 0x02 => ... }` guard. Only type bytes
                    // 0x01 (ClientHello) and 0x02 (ServerHello) reach this point, and
                    // parse_tls_message_handshake on a 0x01/0x02 byte will either
                    // return the correct variant (matched above) or Err(_). A non-
                    // ClientHello/ServerHello Ok(_) variant from these type bytes is
                    // impossible per the tls-parser grammar. The Ok(_) arm is included
                    // only for match exhaustiveness — only the Err(_) path is reachable.
                    self.parse_errors += 1;
                }
            }
        }
        _ => { /* non-ClientHello/ServerHello message type — consume and skip silently */ }
    }
    carry_buf.drain(..4 + body_len);  // exact-consume
}
```

**Parse function choice — `parse_tls_message_handshake` vs `parse_tls_plaintext`
(F-FRESH-001 resolution):**

The carry buffer holds assembled HANDSHAKE MESSAGE bytes: a 1-byte type field, a
3-byte big-endian `uint24` body length, and `body_len` body bytes. There is NO TLS
record header (no content type byte, no 2-byte version, no 2-byte record length).

Two candidates from tls-parser 0.12.2 were considered:

- **`parse_tls_plaintext(i: &[u8])`** — parses a FULL TLS record: 5-byte record
  header (`content_type` + `version` + `len`) + payload. Requires re-synthesizing a
  5-byte TLS record header around the assembled message bytes before calling. This
  would produce a synthetic `[0x16, 0x03, 0x01, len_hi, len_lo]` prefix, adding
  unnecessary complexity and an artificial dependency on version bytes.

- **`parse_tls_message_handshake(i: &[u8])`** — parses a SINGLE handshake message
  from raw message bytes: reads `u8` type, `u24` length, then `length` body bytes,
  dispatches to the appropriate handshake parser, and returns
  `TlsMessage::Handshake(...)`. This is exactly what the carry buffer holds after
  reassembly.

**Decision: `parse_tls_message_handshake` is the correct function.** It consumes
exactly `4 + body_len` bytes (matching the `drain(..4 + body_len)` exact-consume),
requires no synthetic header construction, and returns the same `TlsMessage`
variants matched by the existing single-record path (L766-778 in `try_parse_records`).

Import needed (add to the existing `use tls_parser::{...}` block in `src/analyzer/tls.rs`):

```rust
use tls_parser::{
    Err as NomErr, TlsCipherSuite, TlsCipherSuiteID, TlsExtension, TlsExtensionType,
    TlsMessage, TlsMessageHandshake,
    parse_tls_extensions, parse_tls_message_handshake, parse_tls_plaintext,  // <-- add parse_tls_message_handshake
};
```

**Malformed-but-complete assembled body error semantics (F-FRESH-001):**

When `parse_tls_message_handshake` returns `Err(_)` on an assembled, length-complete
message (body_len bytes are all present but the inner ClientHello/ServerHello structure
is malformed — e.g., a ClientHello with truncated extensions, an out-of-range version
field, or a zero-length cipher suite list):

1. `parse_errors` is incremented by 1. This is parity with the single-record path
   (tls.rs L787-789). The body was framed correctly at the TLS record layer; the
   failure is in the inner handshake structure.
2. The message bytes ARE exact-consumed (`drain(..4 + body_len)` still executes).
   The 4-byte header + body_len body bytes are a complete message frame — they are
   consumed regardless of whether parse succeeded.
3. No finding is emitted. A malformed reassembled body is not a detection event.
4. No panic. `parse_tls_message_handshake` returns `Result` — the `Err(_)` arm is
   handled explicitly.

This is consistent with BC-2.07.040 Inv-1: carry overflow does NOT increment
`parse_errors`; a malformed-body re-parse failure DOES. These are distinct failure
modes at different layers.

The `Ok(_)` wildcard arm (message parsed but type is not ClientHello/ServerHello —
e.g., `parse_tls_message_handshake` on a non-0x01/0x02 type byte) does NOT increment
`parse_errors`. Non-hello message types in the reassembly path are consumed and
skipped silently (same as the `_` arm for msg_type bytes other than 0x01/0x02).
In practice the `match msg_type { 0x01 | 0x02 => ..., _ => {} }` outer guard
already filters non-hello types before `parse_tls_message_handshake` is called.

**Ordering relative to the buffer-fill check:** The per-message body_len guard
lives INSIDE the consume loop (fires after the 4-byte header is available in the
carry). The buffer-fill overflow guard (Decision 5) lives BEFORE the carry append
(fires when `carry_buf.len() + record_payload_len > MAX_BUF`). Both produce the
same clear-and-recover outcome (carry cleared, `handshake_reassembly_overflows`
incremented), but trigger at different points in the data flow.

**Rationale for MAX_BUF (65,536) as the per-message cap (SR-002 / F-P1-006 fix):**
The previous skeleton used `MAX_RECORD_PAYLOAD` (18,432) as the per-message body
guard. This was incorrect: a handshake message may legally span several records and
legitimately exceed one record's payload. Go's `crypto/tls` hard-caps a single
reassembled handshake message at `maxHandshake = 65536` bytes — the strictest
real-world interoperable ceiling. No ClientHello or ServerHello that a Go server
would accept can exceed 65,536 bytes. Using 18,432 as the per-message cap would
silently drop legitimate large ClientHellos (ECH/post-quantum, ~1.5–2.5 KiB, easily
multi-record after deliberate fragmentation). 65,536 is the correct cap and is already
the `MAX_BUF` constant in scope.

The `break` guard on `carry_buf.len() < 4` handles the case where the first record
contains only 1, 2, or 3 bytes of the handshake header (partial header — cannot yet
determine `body_len`). No parse error is emitted in this case; the carry retains the
partial header bytes until the next record arrives.

**Carry append + overflow check placement (SR-012 fix):** The carry append and the
buffer-fill overflow check (Decision 5) live in `try_parse_records` at the per-0x16-
record drain site — NOT in `on_data` (which calls `try_parse_records` after
appending to `client_buf` / `server_buf`, the TCP-level buffer). The handshake
carry buffers (`client_hs_carry`, `server_hs_carry`) are distinct from the TCP
segment buffers; their append and overflow guard both occur inside the 0x16 record
processing branch of `try_parse_records`.

The `drain(..4 + body_len)` exact-consume ensures:
- Coalesced messages (BC-2.07.042): the loop naturally proceeds to the next message
  header after consuming the first complete message.
- No double-dispatch: each message is dispatched exactly once.
- No silent skip: the loop continues until `carry_buf.len() < 4` (incomplete header)
  or the body is incomplete.

### Decision 5 — Bounded Carry: Clear-and-Recover Overflow Policy

When a new 0x16 record would push `carry_buf.len() + record_payload_len > MAX_BUF`:

```rust
if carry_buf.len() + record_payload_len > MAX_BUF {
    carry_buf.clear();                                       // clear this direction's carry (TlsFlowState field)
    analyzer.handshake_reassembly_overflows += 1;            // TlsAnalyzer aggregate counter — NOT TlsFlowState
    // do NOT increment parse_errors
    // do NOT emit a finding
    // do NOT set any abandoned flag — there is NO hs_carry_abandoned field
    continue;  // to next record in the outer loop (recovery permitted)
}
```

This is the **clear-and-recover policy** (Policy A, F1 Open Design Question §Q1
resolved by Pass-1 adversarial reconciliation). Clearing the carry buffer rather
than setting a sticky `hs_abandoned: bool` flag was chosen for the following reasons:

1. **Evasion-resistance (primary driver, F-P1-008).** A sticky abandon-direction
   flag is a one-packet, permanent, attacker-triggerable blinding primitive: one
   crafted oversized fragment sets the flag and permanently prevents the analyzer
   from seeing any later well-formed ClientHello on that direction (Ptacek/Newsham
   1998 desynchronization-to-evasion pattern; Suricata CVE-2019-18792 real-world
   precedent). Clear-and-recover denies this permanence: the attacker cannot convert
   a transient overflow into a durable blind spot without continuous effort.
2. **Industry alignment.** Passive analyzers (Wireshark) and stateful ones (Suricata
   at stream-depth, Zeek per-analyzer) overwhelmingly favor "curtail inspection /
   signal anomaly / keep the flow" over per-flow blacklisting. wirerust is a passive
   tool; "keep inspecting, signal the anomaly" matches the correct posture.
3. **Consistency with existing code.** `src/analyzer/tls.rs` L689–698 already
   implements clear-and-recover for the `MAX_RECORD_PAYLOAD` over-size case
   (`client_buf.clear()` / `server_buf.clear()` then `return`). Adopting the same
   policy for the handshake carry buffers keeps one coherent overflow discipline.
4. **Observability.** The new `handshake_reassembly_overflows` counter (Decision 1,
   added per Pass-1 F-P1-008 fix; placed on TlsAnalyzer per Pass-2 F-F2-002) increments
   on each overflow. Repeated overflows on flows are an anomaly signal, converting the
   residual risk into detectable telemetry rather than a silent blind spot. This mirrors
   the existing `truncated_records` aggregate counter on `TlsAnalyzer` (also NOT a
   per-flow TlsFlowState field — it is an analyzer-level aggregate, not dropped at
   on_flow_close).
5. **Recovery.** A subsequent well-formed 0x16 record starting a new handshake
   message (type byte 0x01 or 0x02) re-populates the now-empty carry correctly.
   The cleared partial handshake was not a complete message and would never have
   produced a finding; the real ClientHello that follows IS parsed and dispatched
   normally. There is NO recovery-blocking state.

`parse_errors` is NOT incremented on carry overflow. This is deliberate: an oversized
partial handshake is an adversarial or unusual input, not a protocol parse failure.
The existing `parse_errors` counter is reserved for TLS record framing failures
(`parse_tls_plaintext` errors) and extension parse failures, following BC-2.07.039
Invariant 1. Incrementing `parse_errors` on carry overflow would inflate the counter
for inputs that are RFC-permitted (a legitimately large but valid ClientHello).

Full evidence basis for the clear-and-recover choice: `.factory/research/TLS-REASSEMBLY-OVERFLOW-POLICY.md`.

### Decision 6 — Truncation Safety at Flow Close

When `on_flow_close` is called with a partial carry buffer (a handshake header is
present but the body has not arrived — the capture was snaplen-truncated or the TCP
connection was terminated mid-handshake), the carry buffer is silently discarded
with `self.flows.remove(flow_key)`. No finding is emitted and `parse_errors` is
NOT incremented.

**Interaction with READER cand-05 (snaplen truncation, BC-2.01.017):**

A snaplen-truncated capture and a legitimately fragmented-but-incomplete handshake
are byte-identical at the `on_flow_close` call site: both present as a partial carry
buffer with a non-zero byte count and no complete handshake message. The architecture
treats both as "nothing to report":

- Truncated capture: the EPB `captured_len < original_len` field is surfaced by the
  pcapng reader (BC-2.01.012); the analyzer is not aware of it.
- Legitimate incomplete fragment: the TCP connection was closed before the second
  record arrived (e.g., an RST during the handshake).

In both cases, emitting a finding or a parse error would produce a false positive
(finding without evidence) or a false negative (suppressing a genuine detection that
might have been possible with a longer capture). The correct policy is the same for
both: discard the partial carry silently.

This decision closes F1 Open Design Question §Q5 for the pre-`done()` epoch and
explicitly resolves BC-2.07.040's "READER cand-05 interaction" note.

### Decision 7 — Reassembly Only Pre-`done()`

The carry buffer accumulation only occurs for records delivered before `TlsFlowState::done()`
returns `true`. Once `done()` is true (both `client_hello_seen` and `server_hello_seen`),
`try_parse_records` exits early (existing behavior). No carry bytes are accumulated for
post-handshake 0x16 records.

RFC 8446 §5.1: "Handshake messages MUST NOT span key changes." For the ClientHello and
ServerHello (the only messages wirerust extracts), this constraint is moot — they are
sent in the initial unencrypted epoch before any key change. The `done()` gate
automatically prevents buffering post-key-change 0x16 records (which are encrypted
application data masquerading as handshake content in TLS 1.3). This closes F1 Open
Design Question §Q5 for the post-`done()` case.

### Decision 8 — Test Seams

Two new test seams are added to `TlsFlowState` following the existing pattern
(`client_buf_len_for_testing()`, `server_buf_len_for_testing()`):

```rust
#[doc(hidden)]
pub fn client_hs_carry_len_for_testing(&self) -> usize {
    self.client_hs_carry.len()
}

#[doc(hidden)]
pub fn server_hs_carry_len_for_testing(&self) -> usize {
    self.server_hs_carry.len()
}
```

These are `pub` (not `pub(crate)`) to remain accessible from `tests/` integration tests,
matching the convention of the existing test seams. They have no effect on the public API
because `TlsFlowState` is a private type.

**Seam contract — flow-scoped vs. aggregate reads:**

- **Flow-scoped reads** (fields on `TlsFlowState`): `client_hello_seen`, `server_hello_seen`,
  `client_hs_carry.len()` (via `client_hs_carry_len_for_testing`),
  `server_hs_carry.len()` (via `server_hs_carry_len_for_testing`).
  These are accessed through `analyzer.state_for_testing(&flow_key) -> &TlsFlowState`.

- **Aggregate reads** (fields on `TlsAnalyzer`): `parse_errors` (via `parse_error_count()`),
  `sni_counts` (via `sni_counts()`), `ja3_counts` (via `ja3_counts()`),
  `handshakes_seen` (via `handshake_count()`), `handshake_reassembly_overflows`
  (via `handshake_reassembly_overflow_count()`).
  These are accessed directly on the `TlsAnalyzer` instance.

NO test harness may read aggregate counters (`parse_errors`, `sni_counts`, `ja3_counts`,
`handshakes_seen`, `handshake_reassembly_overflows`) off a `TlsFlowState` reference.
These fields do not exist on `TlsFlowState`.

### Decision 9 — DTU Re-Assessment

DTU re-assessment: NOT REQUIRED. This decision record introduces no new external service
dependencies. wirerust is a passive offline analyzer that reads local pcap/pcapng files.
The `client_hs_carry` and `server_hs_carry` buffers are purely in-process state. No new
network endpoints, APIs, or external service calls are introduced.

## Consequences

### Positive

- **SNI and JA3 are now extracted from fragmented ClientHellos.** The evasion class
  documented in `TLS-CLIENTHELLO-FRAG-001-validation.md` is closed.
- **Single-record fast path is unaffected.** The carry buffer drain loop handles the
  already-complete case identically to the old path.
- **Symmetric coverage of ServerHello.** JA3S is also now extracted from fragmented
  ServerHellos (STORY-B scope, symmetric to ClientHello).
- **Memory ceiling maintained.** Two new carry buffers add at most `2 × MAX_BUF = 128 KiB`
  per flow (one for each direction). This is the same order of magnitude as the existing
  `client_buf` + `server_buf` ceiling (also `2 × MAX_BUF`). The total POST-on_data-return
  per-flow residue ceiling increases from `2 × MAX_BUF` to `4 × MAX_BUF` (256 KiB) — within
  the existing TLS flow's `max_flows` × `memcap` budget. The clear-and-recover guard
  (Decision 5) strictly enforces the per-direction carry cap; no direction can grow past
  MAX_BUF at the END of an `on_data` call. Note: the IN-CALL transient peak may be higher
  because `try_parse_records` clones the incoming `record_bytes` slice before appending to
  the carry buffer; the transient allocation is bounded by `record_len` (at most
  `MAX_RECORD_PAYLOAD`) and is released before `on_data` returns (per BC-2.07.039 v2.3 Inv-2).
- **Minimal code surface.** The change is confined to `src/analyzer/tls.rs`:
  - `TlsFlowState` struct: 2 new fields (`client_hs_carry`, `server_hs_carry`)
  - `TlsFlowState::new()`: 2 initialization lines
  - `TlsAnalyzer` struct: 1 new aggregate counter field (`handshake_reassembly_overflows`)
  - `TlsAnalyzer::new()`: 1 initialization line; 1 new accessor `handshake_reassembly_overflow_count()`
  - `try_parse_records`: 1 new code block (~30 lines) replacing the existing
    single-record 0x16 dispatch branch
  - `on_flow_close`: no change (HashMap remove already drops the TlsFlowState carry fields;
    the analyzer-level aggregate counter is NOT touched at on_flow_close)
  - 2 new test seams on TlsFlowState (carry length accessors)

### Negative / Risks

- **Per-flow memory increase.** The carry buffers add up to `2 × MAX_BUF` bytes per
  TLS flow on top of the existing `client_buf` / `server_buf`. In high-flow-rate captures,
  this could increase peak memory consumption. Mitigation: the clear-and-recover policy
  (Decision 5) strictly enforces the per-direction cap at MAX_BUF; the existing
  `max_flows` / `memcap` configuration in `ReassemblyConfig` provides the outer bound.
  Total POST-on_data-return per-flow residue ceiling: 4 × MAX_BUF ≈ 256 KiB
  (client_buf + server_buf + client_hs_carry + server_hs_carry). In-call transient
  peak is higher due to the record_bytes clone (see Positive Consequences note above
  and BC-2.07.039 v2.3 Inv-2).
- **Single-record fast-path regression risk.** Off-by-one in the exact-consume loop
  (Decision 4) could mis-dispatch or double-dispatch a previously-working single-record
  ClientHello. Mitigation: all 9,391 lines of existing `tls_analyzer_tests.rs` exercise
  the single-record path and must remain green (regression gate). VP-039 Sub-A explicitly
  verifies the single-record equivalence.
- **`parse_errors` accounting discipline.** The new code block must not increment
  `parse_errors` for carry overflow (Decision 5), body_len > MAX_BUF (Decision 4 guard),
  or truncation at flow close (Decision 6). This requires discipline in the implementation;
  VP-039 Sub-C (overflow, including the new `test_vp039_carry_overflow_clear_and_recover`
  and `test_vp039_carry_overflow_recovery` tests) and Sub-D are regression guards for
  exactly this invariant.
- **`done()` flag propagation.** The `client_hello_seen` flag must be set whenever
  `handle_client_hello` is called — including the carry-buffer reassembly path. A missing
  flag set would cause `done()` to never fire and the flow to buffer indefinitely.
  Mitigation: the carry-buffer dispatch calls the same `handle_client_hello` + flag-set
  code block as the single-record path (structural guarantee).

## ADR Cross-References

| ADR | Relationship |
|-----|-------------|
| ADR-0001 | Stream dispatch (content-first): TLS 0x16 0x03 fingerprint triggers TLS classification. Reassembly operates after classification; ADR-0001 is unaffected. |
| ADR-0002 | Modular analyzer pattern (two-trait split): `TlsAnalyzer` still implements `StreamAnalyzer`; the `on_data` signature is unchanged. |
| ADR-009 | pcapng reader (snaplen truncation): Decision 6 of this ADR resolves the interaction between READER cand-05 (EPB `captured_len < original_len`) and partial carry at flow close. Both cases are byte-identical at the `on_flow_close` call site and receive the same "discard silently" treatment. |

## Subsystems Affected

- **SS-07 (TLS Analysis):** Primary change. `TlsFlowState` struct, `try_parse_records`,
  new test seams. BC-2.07.001 and BC-2.07.002 amended (scope expansion — fragmented
  input class). New BCs: BC-2.07.038 through BC-2.07.042.
- **No other subsystems affected.** `src/analyzer/tls.rs` is the only file that changes.
  The `FlowKey` keying, `StreamDispatcher`, `TcpReassembler`, `Finding` struct, and all
  reporters are unchanged.
