# F-EV-001 Validation — TLS `client_buf` Saturation Silent-Blinding

**Cycle:** fix-tls-clienthello-frag
**Finding:** F-EV-001 — attacker saturates the per-direction TLS buffer (`client_buf`/`server_buf`,
cap `MAX_BUF = 65_536`, silent saturating append) so a real ClientHello's bytes are silently
dropped one layer below the handshake-reassembly carry fix, with zero telemetry.
**Validated against:** `develop` @ `ab0b388` (working tree clean).
**Verdict:** **NOT-EXPLOITABLE** as stated (silent, zero-telemetry ClientHello blinding via
`client_buf` saturation). The silent tail-drop code path is real, but it is **structurally
unreachable** for the ClientHello-blinding goal given how `on_data` is fed and the per-record
oversize guard.

---

## 1. The silent tail-drop append — CONFIRMED to exist as code

`src/analyzer/tls.rs:820-835` (`TlsAnalyzer::on_data`, ClientToServer arm):

```rust
Direction::ClientToServer => {
    let remaining = MAX_BUF.saturating_sub(state.client_buf.len());   // 822
    if remaining > 0 {                                                // 823
        let to_copy = data.len().min(remaining);                      // 824
        state.client_buf.extend_from_slice(&data[..to_copy]);         // 825
    }
}
```

- `MAX_BUF = 65_536` (`src/analyzer/tls.rs:30`).
- The append is **saturating and silent**: if `data.len() > remaining`, the tail
  `data[to_copy..]` is dropped. No counter (`parse_errors`, `truncated_records`) is touched,
  no `Finding` is pushed. When `remaining == 0` the entire `data` is dropped silently.
- The claim in F-EV-001 that the append is "SILENT, lossy, saturating" is **factually correct
  at this line** (matches BC-2.07.005). The drop is genuinely telemetry-free.

So the *primitive* the finding relies on exists. The question is whether it is **reachable** in a
way that drops a ClientHello's bytes. It is not, for the two independent structural reasons below.

---

## 2. How `on_data` is fed — each `data` slice is ONE reassembled segment

Trace of the feeding path (`StreamHandler::on_data` caller):

1. `TcpReassembler::process_packet` → `flush_contiguous_data` (`src/reassembly/mod.rs:605-622`):
   ```rust
   let flushed = flow_dir.flush_contiguous();              // 615  -> Vec<(u64, Vec<u8>)>
   for (offset, data) in &flushed {                        // 618
       handler.on_data(key, dir, data, *offset, timestamp); // 620
   }
   ```
2. `FlowDirection::flush_contiguous` (`src/reassembly/segment.rs:395-407`):
   ```rust
   while let Some(data) = self.segments.remove(&self.base_offset) {  // 398
       ...
       flushed.push((offset, data));                                 // 403
   }
   ```
   **Each loop iteration removes exactly ONE BTreeMap entry and pushes it as a separate
   `(offset, data)` pair.** There is NO coalescing of adjacent contiguous segments into a single
   larger `data` buffer. Two contiguous stored segments become two separate `on_data` calls.
3. `StreamDispatcher::on_data` (`src/dispatcher.rs:348-351`) forwards each `data` slice verbatim
   to `tls.on_data` — no buffering, no concatenation.

**Consequence:** the size of any single `data` slice handed to `TlsAnalyzer::on_data` equals the
byte length of one stored segment. A stored segment is created from one TCP packet's
`packet.payload` (`src/reassembly/mod.rs:370`, `insert_segment(tcp.seq, payload, ...)`), where
`payload = tcp.payload().to_vec()` from a single IP datagram (`src/decoder.rs:580-584`). A stored
segment is only ever **truncated DOWN** by the depth limit (`segment.rs:264-280`); it is never
grown. So:

> max single `on_data` `data` length  ==  max single-IP-datagram TCP payload.

For IPv4 the IP total-length field is 16 bits, so a single datagram's TCP payload is `< 65_535`
bytes — i.e. `< MAX_BUF` and, after subtracting the 20+ byte IP+TCP headers, comfortably under
`MAX_BUF`. (IPv6 non-jumbo payload-length is likewise 16-bit.)

This already means a **single `on_data` call cannot, by itself, deliver more than ~64 KB**, and in
the IPv4/standard-IPv6 case cannot deliver enough to overflow an *empty* `client_buf`
(`remaining == MAX_BUF == 65_536` on the first call; a `< 65_535`-byte payload fits whole). The
silent tail-drop on a single call therefore requires `client_buf` to ALREADY hold close to
`MAX_BUF` undrained bytes — see §3.

---

## 3. Maximum undrained residue in `client_buf` — bounded by ~one incomplete record

The attacker's only lever to drive `client_buf.len()` toward `MAX_BUF` (so a later append's tail is
dropped) is to leave **undrained** bytes in the buffer across `on_data` calls. `try_parse_records`
(`src/analyzer/tls.rs:654-792`) governs draining:

- It loops, peeking the 5-byte record header and reading `payload_len = u16::from_be_bytes([buf[3],
  buf[4]])` (`tls.rs:676`). `payload_len` is a **16-bit field → max 65_535**.
- **Per-record oversize guard (BC-2.07.004), `tls.rs:689-699`:**
  ```rust
  if payload_len > MAX_RECORD_PAYLOAD {        // MAX_RECORD_PAYLOAD = 18_432 (tls.rs:34)
      self.parse_errors += 1;                  // 690  TELEMETERED
      self.truncated_records += 1;             // 691  TELEMETERED
      ... client_buf.clear() ...               // 692-697  buffer emptied
      return;
  }
  ```
  Any record declaring `payload_len > 18_432` **clears the entire buffer** and bumps **two
  counters** (`parse_errors` AND `truncated_records`). This is loud, not silent. So an attacker
  cannot park a single giant declared-length record in the buffer to inflate residue — it is
  evicted with telemetry the moment its header is at the front of the buffer.
- A **complete** record (`buf_len >= 5 + payload_len`, `tls.rs:701-705`) is drained
  (`drain(..total_record_len)`, `tls.rs:726/753`), so it contributes **zero** lasting residue.
- An **incomplete** record (`buf_len < total_record_len`, `tls.rs:702-705`) causes `return` — its
  bytes stay buffered. But a valid (non-oversize) record has `total_record_len <= 5 + 18_432 =
  18_437` bytes. So the trailing incomplete record contributes **at most 18_437 undrained bytes.**

Crucially, the parse loop **drains every complete leading record before it can return on an
incomplete trailing one.** You cannot accumulate "many complete records' worth" of undrained bytes:
complete records are consumed each pass. The maximum undrained residue at any quiescent point is
therefore one partial record header/body, **≤ 18_437 bytes** — far below `MAX_BUF = 65_536`.

(There is one transient window where the buffer can momentarily exceed 18_437 bytes *within a single
`on_data` call*: an append could place, say, two complete 18 KB records plus a partial third before
`try_parse_records` runs. But `try_parse_records` runs immediately at the end of the SAME `on_data`
call (`tls.rs:838`) and drains the two complete records, returning residue to ≤ 18_437. The buffer
is never observed at >18_437 undrained bytes by any *subsequent* append, because each append is
followed by a drain pass. The peak transient is bounded by `prior_residue (<=18_437) + one
on_data data slice (<~64KB)`, and even that peak append cannot itself silently drop bytes unless the
running buffer already held near-`MAX_BUF` undrained — which §3 shows it cannot.)

---

## 4. Putting it together — the ClientHello cannot be silently dropped at `client_buf`

For F-EV-001 to fire, the attacker must get `client_buf.len()` to within `< (size of the
ClientHello-bearing append)` of `MAX_BUF` **using only valid (`<= 18_432`) records that leave
undrained residue, with no counter firing**, and then deliver the ClientHello so its tail spills
past `MAX_BUF`.

This is impossible because:

1. **Residue ceiling is ~18 KB, not ~64 KB.** Valid records are drained as they complete
   (§3); the only durable residue is a single trailing incomplete record `<= 18_437` bytes. With
   `client_buf.len() <= 18_437`, `remaining = MAX_BUF - len >= 47_099` bytes — more than enough to
   hold any ClientHello, and a ClientHello-bearing single segment is itself `< 65_535`. The tail-drop
   branch (`data.len() > remaining`) is not reached for any single ClientHello segment.

2. **The oversize guard is telemetered, not silent.** The only way to push declared lengths past
   18 KB is `payload_len > 18_432`, which hits `tls.rs:689` and increments BOTH `parse_errors` and
   `truncated_records` while clearing the buffer. So even an *attempt* to bloat the buffer with a
   big record produces telemetry — the opposite of the "zero telemetry" claim — and additionally
   wipes the buffer rather than retaining residue toward `MAX_BUF`.

3. **No single `on_data` delivers >64 KB** (§2: one IP datagram, 16-bit length field; segments are
   only truncated down, never coalesced up). So saturation cannot be achieved in one shot against an
   empty buffer either.

There is no attacker-controllable record sequence that simultaneously (a) drives `client_buf`
near `MAX_BUF`, (b) keeps that near-full state durable across to the ClientHello append, and
(c) emits zero telemetry. The premise "accumulate toward 64 KB of undrained bytes WITHOUT tripping
the telemetered oversize guard" is **false**: undrained residue is capped at ~18 KB by the
drain-complete-records-first loop, well below `MAX_BUF`.

---

## 5. Does ANY counter fire on the closest reachable scenario?

- Attacker sends a record header with `payload_len > 18_432` (the only way to "aim" at buffer
  pressure): **`parse_errors`++ AND `truncated_records`++** fire (`tls.rs:690-691`); buffer cleared.
  Loud.
- Attacker floods post-handshake application data after both hellos: the `done()` short-circuit
  (`tls.rs:807-810`) drops it before buffering — but that is *after* a ClientHello was already
  parsed, so it does not blind the ClientHello. (Covered by `test_*BC_2_07_034*`,
  `tests/tls_analyzer_tests.rs:819-865`.)
- A genuinely truncated/incomplete ClientHello that never completes: its bytes sit in `client_buf`
  (`<= 18_437`) and it simply never parses — but this is the *honest* "handshake never finished"
  case the new reassembly-carry layer is designed to handle, and it is NOT a silent drop *below*
  `MAX_BUF`; the bytes are retained, not discarded. No saturation occurred.

---

## 6. What would change the verdict (preconditions for CONDITIONAL exploitability)

The verdict is NOT-EXPLOITABLE under the current code. It would become **CONDITIONALLY-EXPLOITABLE**
only if one of these structural facts changed:

- **(P1) The reassembler began coalescing adjacent contiguous segments into a single `on_data`
  `data` buffer > 64 KB.** Today `flush_contiguous` (`segment.rs:398`) emits one chunk per
  BTreeMap entry, so this does not hold. If a future refactor merged the flush into a single
  growing `Vec`, a single `on_data` could exceed `MAX_BUF` and the tail-drop would become reachable
  against an empty buffer.
- **(P2) An IPv6 jumbogram path delivered a single TCP payload > 65_535 bytes** in one segment.
  This depends on the `etherparse` 0.20 slicer surfacing jumbogram (Hop-by-Hop, 32-bit length)
  payloads as a single `tcp.payload()`. Even then, the drop would be of the *segment tail*, and
  the question of whether the ClientHello specifically is the dropped portion is attacker-uncertain;
  and `truncated_records` still would not fire (the tail-drop in `on_data` is the silent one). This
  is a narrow, capture-format-dependent edge worth a defensive counter but is not the
  `client_buf`-saturation-via-valid-records mechanism F-EV-001 describes.
- **(P3) `MAX_RECORD_PAYLOAD` were raised at/above `MAX_BUF`** (removing the 18 KB residue ceiling),
  letting a single incomplete record approach 64 KB of durable residue. Today `18_432 < 65_536`, so
  the ceiling holds with ~47 KB of headroom.

None of P1–P3 hold on `develop @ ab0b388`.

---

## 7. Verdict

**NOT-EXPLOITABLE.**

The silent saturating tail-drop (`tls.rs:822-825`) exists exactly as F-EV-001 describes, but it
cannot be weaponized to silently blind a ClientHello, because:

- Each `on_data` `data` slice is a single reassembled segment (no coalescing; `segment.rs:398`),
  bounded by one IP datagram's TCP payload (`< MAX_BUF`).
- Durable undrained residue in `client_buf` is capped at one incomplete valid record
  (`<= 18_437` bytes), leaving `>= 47_099` bytes of headroom — the drain-complete-records-first
  loop (`tls.rs:654-792`) prevents accumulation toward 64 KB.
- The only mechanism to declare lengths above 18 KB is the per-record oversize guard
  (`tls.rs:689-699`), which is **telemetered** (`parse_errors` + `truncated_records`) and clears the
  buffer — directly contradicting the finding's "zero telemetry" premise.

The finding's central assumption — that `client_buf` can accumulate toward 64 KB of undrained bytes
without tripping a telemetered guard — is false. The 18 KB per-record cap plus eager draining of
complete records structurally prevents the saturation precondition.

**Recommendation:** Do not file F-EV-001 as an exploitable evasion against current `develop`. If
defense-in-depth is desired, the cheapest hardening is a counter on the `on_data` tail-drop branch
itself (`tls.rs:824`, when `to_copy < data.len()` or `remaining == 0`) so that the *primitive* is no
longer silent regardless of reachability — this also pre-empts precondition P1/P2 above. Validate
any such follow-up via `vsdd-factory:research-agent` per `DF-VALIDATION-001` before issue creation.
