---
document_type: behavioral-contract
level: L3
version: "2.10"
status: draft
producer: product-owner
timestamp: 2026-06-30T00:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
lifecycle_status: active
introduced: fix-tls-clienthello-frag
modified:
  - "v2.0: Pass-1 adversarial reconciliation (F-P1-001/SR-001 CRITICAL, F-P1-006 MED, F-P1-010 LOW) — remove hs_carry_abandoned precondition; raise per-message body_len cap from MAX_RECORD_PAYLOAD (18432) to MAX_BUF (65536); downgrade priority P0→P1 for consistency with sibling BCs and VP-039; update Architecture Anchors to remove abandoned-flag fields; update EC-005 to clear-and-recover semantics — 2026-06-29"
  - "v2.1: Pass-2 adversarial reconciliation (F-F2-010 CRITICAL / DF-CANONICAL-FRAME-HOLDOUT-001) — add Acceptance Criterion AC-CANONICAL-FRAME: at least one test must decode a canonical handshake header byte sequence cited verbatim from RFC 8446 §4 (explicit msg_type || uint24 length bytes and expected body_len), authored independently of the project's build_client_hello/build_server_hello helpers, to pin the 3-byte big-endian length decode against an authoritative source (anti shared-assumption); Red-Gate test name added: test_BC_2_07_038_canonical_frame_rfc8446_s4 — 2026-06-29"
  - "v2.2: Pass-3 adversarial reconciliation (F-P3-001 HIGH) — Architecture Anchors: handshake_reassembly_overflows type corrected u32→u64 to mirror truncated_records (u64 in src/analyzer/tls.rs:319); ADR-011 and counter accessor already specify u64 — 2026-06-29"
  - "v2.3: Fix burst 6 adversarial reconciliation (F-FRESH-001 CRITICAL, F-FRESH-004 MEDIUM, F-FRESH-005 MEDIUM) — (1) PC-9 added: assembled-body parse boundary specified — parse_tls_message_handshake (not parse_tls_plaintext, which requires a record header); malformed-but-complete assembled body MUST increment parse_errors by 1, exact-consume 4+body_len bytes, emit no finding, and not panic (parity with single-record path per ADR-011 Decision 4); Red-Gate test name test_BC_2_07_038_malformed_assembled_body; distinction from BC-2.07.040 Inv-1 (carry overflow/truncation does NOT touch parse_errors; malformed-complete body DOES) stated explicitly in PC-9; (2) EC-008 added: carry fate when BC-2.07.004 record-layer oversize guard fires mid-reassembly — existing guard clears client_buf, increments parse_errors and truncated_records, returns; client_hs_carry is NOT touched; orphaned partial carry persists bounded by MAX_BUF, is dropped at flow close per BC-2.07.040, emits no finding (accepted-risk: bounded + harmless per BC-2.07.040); (3) EC-009 added: per-record work-amplification bound — MAX_BUF/MAX_RECORD_PAYLOAD bounds record SIZE not record COUNT; per-record CPU work (header peek, clone, drain) is bounded by upstream TCP-reassembly record cap; upstream stream reassembler limits record count; references research doc Q5 fragmentation-control note — 2026-06-29"
  - "v2.4: Fix burst 8 adversarial reconciliation (F-ADV-002 MEDIUM, F-ADV-003 MEDIUM) — Verification Properties table: two dangling test citations resolved — (1) 'test_BC_2_07_038_single_record_regression' (non-existent) re-pointed to VP-039 Sub-A proptest_vp039_carry_reassembly_two_record (the two-record proptest implicitly asserts single-vs-fragmented equivalence baseline); (2) 'test_BC_2_07_038_non_hello_type_consumed' (non-existent) re-pointed to test_BC_2_07_042_exact_consume_no_double_dispatch (BC-2.07.042 asserts non-hello/coalesced message consumed with parse_errors==0). No new tests; 14-harness count unchanged. — 2026-06-29"
  - "v2.5: Fix burst 9 adversarial reconciliation (F-IMPL-002 MEDIUM) — AC-CANONICAL-FRAME expanded to enumerate three canonical test frames (Frame A: degenerate body_len=5 / Frame B: BE-vs-LE discriminator 66,816 vs 1,281 / Frame C: body_len=256 all-zero body exercises PC-9 malformed-body path); AC-CANONICAL-FRAME explicitly documents that the canonical-frame test test_BC_2_07_038_canonical_frame_rfc8446_s4 legitimately bundles BE-decode verification (Frame A), clear-and-recover after body_len-spoof (Frame B), and PC-9 malformed-body parse_errors+1 (Frame C) in one test; parse_errors+1 assertion for Frame C is now backed by an explicitly documented BC behavior; EC-010 added: complete valid message followed by trailing body_len-spoof header coalesced in one carry — valid message dispatches first; spoof-only remainder cleared; no valid data lost (see BC-2.07.042 EC-006 for drain-loop perspective) — 2026-06-29"
  - "v2.6: Fix burst 10 adversarial reconciliation (F-ADVF2-002 MED, F-ADVF2-003 LOW) — (1) Frame B prose corrected: the 4-byte header [0x01,0x01,0x05,0x00] is the ENTIRE input (no body bytes follow); the body_len-spoof guard fires on header decode before any body is appended — stating 66,816 zero bytes follow was physically wrong; the 66,816 / 1,281 decoded values are unchanged and correct; (2) Frame B AC backing tightened: Inv-5 / Decision-4 (body_len-spoof guard on header decode) is the PRIMARY backing; BC-2.07.039 shares only the clear+counter outcome (not the trigger — BC-2.07.039 is the Decision-5 buffer-fill guard, a different trigger); Frame A semantics (PC-9 malformed, parse_errors+1) unchanged — 2026-06-29"
  - "v2.7: Fix burst 11 (F-COMP-001 / F-COMP-003) — Verification Properties table: two new VP rows added citing architect-authored tests in VP-039: (1) test_vp039_n_record_reassembly for N-record drip-feed incl. 4-byte header split (PC-1/PC-2/PC-6 + EC-003); (2) test_vp039_large_valid_hello_reassembly for large ClientHello body 18,433..65,536 bytes (positively verifies Inv-5 cap raise to MAX_BUF) — 2026-06-29"
  - "v2.8: Artifact-fidelity correction (HS-F4-001-FRAMEC-validation.md) — corrected Frame C input vector from all-zero 256-byte body to 256-byte 0xcc body (session-id length = 0xcc = 204 > 32 triggers tls_parser 0.12.2 verify(be_u8, |&n| n <= 32) in tls_handshake.rs, yielding genuine Err; all-zero body yields Ok per parse_cipher_suites len==0 acceptance, NOT a malformed-reject case); corrected PC-9 example list: replaced 'zero-length cipher suite list' with 'session-id length byte > 32' as the canonical failing example; generalized stale parse_tls_plaintext tls.rs L787-789 citation in PC-9(a) to function-level reference; added PC-9 NOTE clarifying that a degenerate all-zero ClientHello body is accepted (parse_errors=0, JA3 emitted) — conformant lenient behavior per tls-parser 0.12.2 and standard JA3 tooling — 2026-06-30"
  - "v2.9: F5 scoped-adversarial spec-precision reconciliation (F-01 MEDIUM, F-03 LOW) — Inv-4: scoped done() guarantee to the per-on_data-call boundary; removed false 'structurally guaranteed … never buffered … fires before try_parse_records' over-claim; documented bounded within-call behavior (mid-drain done() flip possible in out-of-order hello scenario; worst-case effect is a spurious parse_errors+1 or additional handshakes_seen increment — bounded, benign, not a DoS or correctness defect); PC-3: added direction-gating precision — 0x01 dispatches only on client-direction carry (client_hs_carry), 0x02 dispatches only on server-direction carry (server_hs_carry); off-direction hello msg_type falls into the _ arm without dispatch and without parse_errors increment; consistent with BC-2.07.041 Inv-2; no code change — 2026-06-30"
  - "v2.10: F5 architecture-anchor re-anchor (F-F5-001) — truncated_records field :319→:339 in Architecture Anchors; EC-010 prose nit: 'remaining carry (spoof header bytes only)' → 'entire carry buffer (post-loop drain skipped; already-dispatched valid prefix harmlessly discarded — end state: carry empty)'; Frame-A body bytes: 'exactly 5 zero bytes' → '5 arbitrary body bytes (test uses 0xff; content irrelevant — 5-byte body below ClientHello minimum, parse_tls_message_handshake returns Err)'; develop 8b52046; no semantic change — 2026-06-30"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.038: TLS Handshake-Message Reassembly Across Record Boundaries

## Description

When a TLS handshake message (content type 0x16) is fragmented across two or more
consecutive TLS records for the same flow direction, `TlsAnalyzer` accumulates the
handshake-fragment bytes from each record into a per-direction carry buffer
(`client_hs_carry` or `server_hs_carry`) and dispatches `ClientHello` or `ServerHello`
only when the carry buffer contains at least `4 + body_len` bytes (1-byte type + 3-byte
big-endian length header + full body). The dispatch call receives the same assembled
bytes as if the message had arrived in a single record. This closes the SNI/JA3 evasion
gap identified in finding TLS-CLIENTHELLO-FRAG-001 (RFC 5246 §6.2.1; RFC 8446 §5.1).

## Preconditions

1. `TlsAnalyzer::on_data` has been called with bytes for a given flow direction.
2. The bytes include at least one complete TLS record with `record_type == 0x16`
   (Handshake) whose payload, when combined with any existing carry buffer bytes for
   that direction, begins a or continues an in-progress handshake message.
3. `payload_len <= MAX_RECORD_PAYLOAD` (18,432 bytes); oversized individual records are
   rejected before touching the carry buffer (BC-2.07.004 guard fires first).
4. The flow has not yet been marked `done()` (both hellos already seen); the `done()`
   short-circuit fires before carry buffer processing.

## Postconditions

1. The `record_payload` bytes from any 0x16 record are appended to the direction's
   carry buffer (`client_hs_carry` or `server_hs_carry`) before attempting to parse
   a complete handshake message.
2. A handshake message is dispatched only when `carry_buf.len() >= 4 + body_len`,
   where `body_len` is decoded from bytes `[1..4]` of the carry buffer as a 3-byte
   big-endian unsigned integer.
3. When a complete handshake message is present, dispatch is **direction-gated**
   (consistent with BC-2.07.041 Inv-2, which requires carry selection to match the
   `Direction` parameter):
   a. If `msg_type == 0x01` (ClientHello) **and** the carry is the **client-direction
      carry** (`client_hs_carry`, i.e., `Direction::ClientToServer`), `handle_client_hello`
      is called with the assembled message body. A `0x01` msg_type arriving on the
      server-direction carry (`server_hs_carry`) falls into the `_` arm: consumed
      silently without dispatch and without incrementing `parse_errors`.
   b. If `msg_type == 0x02` (ServerHello) **and** the carry is the **server-direction
      carry** (`server_hs_carry`, i.e., `Direction::ServerToClient`), `handle_server_hello`
      is called with the assembled message body. A `0x02` msg_type arriving on the
      client-direction carry (`client_hs_carry`) falls into the `_` arm: consumed
      silently without dispatch and without incrementing `parse_errors`.
   c. If `msg_type` is any other value, the message is advanced past (consumed) without
      dispatching (regardless of direction). This prevents a future implementer from
      adding cross-direction dispatch, which would corrupt `client_hello_seen`,
      `server_hello_seen`, and the associated fingerprint maps (`sni_counts`,
      `ja3_counts`, `ja3s_counts`).
4. After dispatching or advancing past a complete message, exactly `4 + body_len` bytes
   are removed from the front of the carry buffer (exact-consume). Any remaining bytes
   belong to the next handshake message.
5. The consume loop repeats until no complete handshake message remains in the carry
   buffer (handles coalesced messages — see BC-2.07.042).
6. If `carry_buf.len() < 4`, the loop breaks immediately (the handshake header is
   incomplete; no `body_len` can be decoded).
7. Content types other than 0x16 never feed the carry buffer; the existing
   guard-before-allocate path for non-handshake records (see BC-2.07.033) is
   unaffected.
8. Two distinct drain operations occur when a complete handshake message is dispatched:
   (a) the TLS record bytes are drained from `client_buf` (or `server_buf`) at the
   record layer as the full TLS record is consumed; and (b) exactly `4 + body_len` bytes
   of the assembled handshake message are exact-consumed from `client_hs_carry` (or
   `server_hs_carry`) at the handshake carry layer. Both drains are required; neither
   alone is sufficient. This resolves SR-008 ambiguity about which buffer is "drained."
9. **Parse boundary and malformed-body semantics (ADR-011 Decision 4; F-FRESH-001).**
   When `carry_buf.len() >= 4 + body_len` and `msg_type` is `0x01` (ClientHello) or
   `0x02` (ServerHello), the assembled message bytes are passed to
   `parse_tls_message_handshake` (from `tls_parser` 0.12.2) — NOT `parse_tls_plaintext`.
   Rationale: the carry buffer holds raw handshake-message bytes (1-byte type + 3-byte
   `uint24` length + body); there is NO TLS record header. `parse_tls_plaintext` requires
   a 5-byte record header prefix (`content_type` + `version` + `len`) that the carry
   does not contain — calling it would require synthesizing a synthetic header, adding
   artificial complexity and version-field coupling. `parse_tls_message_handshake`
   directly consumes the `4 + body_len` byte slice the carry holds.

   If `parse_tls_message_handshake` returns `Err(_)` (assembled body is length-complete
   but internally malformed — e.g., truncated extensions, out-of-range version, or a
   session-id length byte > 32, which `tls_parser` 0.12.2 rejects via
   `verify(be_u8, |&n| n <= 32)` in `tls_handshake.rs`), the following MUST hold:
   - (a) `parse_errors` is incremented by exactly 1 (parity with the single-record
     path where `parse_tls_message_handshake` failure increments `parse_errors`).
   - (b) The message bytes are still exact-consumed: `drain(..4 + body_len)` executes
     regardless — the 4-byte header + body are a complete, length-consistent frame.
   - (c) No finding is emitted. A malformed reassembled body is not a detection event.
   - (d) No panic. `parse_tls_message_handshake` returns `Result`; the `Err(_)` arm is
     handled explicitly.

   **NOTE — degenerate-but-structurally-valid ClientHello is ACCEPTED (conformant):**
   A ClientHello body that is structurally valid but semantically degenerate — e.g.,
   version=0, empty cipher-suite list (0-byte `ciphers_len`), empty compression list,
   empty or absent extensions, trailing zero-byte padding within the declared body
   length — parses as `Ok` under tls-parser 0.12.2. Empty cipher lists are explicitly
   accepted (`parse_cipher_suites` returns `Ok` when `len == 0`, `tls_handshake.rs`);
   inner trailing bytes within the declared `body_len` are discarded by
   `parse_tls_message_handshake`. Such a body does NOT fire PC-9. `parse_errors` stays
   0; `client_hello_seen` is set; a JA3 is emitted from the degenerate fields. This is
   conformant lenient behavior, consistent with tls-parser 0.12.2 and standard JA3
   tooling (Zeek, Suricata). Canonical example: an all-zero 256-byte ClientHello body
   has `sidlen=0` (ok), `ciphers_len=0` (ok, empty accepted), `comp_len=0` (ok),
   trailing zeros within declared length discarded → `Ok` → `parse_errors=0`, JA3
   emitted. This is NOT a malformed-reject case and does not fire PC-9.

   **Distinction from BC-2.07.040 Inv-1 and BC-2.07.039 PC-3 (no contradiction):**
   These are three separate, non-overlapping failure paths:
   - *Malformed-but-complete assembled body* (this postcondition): the carry holds all
     `4 + body_len` bytes; the inner structure fails to parse → `parse_errors += 1`
     (parity with single-record path). DOES increment `parse_errors`.
   - *Carry overflow* (BC-2.07.039 PC-3): accumulation would exceed `MAX_BUF` → carry
     cleared, `handshake_reassembly_overflows += 1`. Does NOT touch `parse_errors`.
   - *Truncated carry at flow close* (BC-2.07.040 Inv-1): incomplete carry discarded
     via `HashMap::remove` at flow close. Does NOT touch `parse_errors`.
   The three paths are triggered at different code points and have different counter
   semantics. No path subsumes another.

   Red-Gate test name (for VP-039 / architect to author):
   **`test_BC_2_07_038_malformed_assembled_body`**
   Test: construct a 0x16 carry that is `4 + body_len` bytes (length-complete) but
   whose body is internally malformed (e.g., truncated cipher-suite list); assert
   `parse_errors == 1`, `client_hello_seen == false`, no finding emitted.

## Invariants

1. A message whose type byte is neither `0x01` (ClientHello) nor `0x02` (ServerHello)
   is parsed for its length header and consumed (advanced past) without dispatching —
   it does NOT trigger a `parse_errors` increment.
2. After dispatching a complete message, exactly `4 + body_len` bytes are removed from
   the carry buffer (exact-consume); no bytes are double-counted, no bytes are skipped.
3. Content types other than 0x16 (`Handshake`) never feed the carry buffer; the CR-010
   guard-before-allocate path for non-handshake records is unaffected.
4. Reassembly is scoped to the **per-`on_data`-call boundary**: when `done()` returns
   true at `on_data` entry (both `client_hello_seen` and `server_hello_seen` are set),
   that call short-circuits before carry buffer processing — records arriving in any
   later `on_data` call after `done()` are never buffered. This cross-call guarantee
   is complete and holds unconditionally.

   **Bounded within-call behavior:** A record buffered while `done()` was false at
   `on_data` entry is processed to completion by the current call. The `done()` check
   fires only once, at `on_data` entry (`src/analyzer/tls.rs:1120-1123`); there is no
   mid-loop re-check inside `try_parse_records`. Consequently, if a dispatch mid-drain
   flips `done()` true — for example, a C2S record carries a late ClientHello coalesced
   with trailing 0x16 bytes, and dispatching the ClientHello sets `client_hello_seen=true`
   while `server_hello_seen` was already true from a prior S2C call — the drain loop
   continues consuming any remaining complete messages in that same record. The
   worst-case observable effect is bounded and benign: a possible single spurious
   `parse_errors += 1` (if trailing bytes fail to parse as a complete message), or an
   additional `handshakes_seen` increment (already permitted by BC-2.07.042 EC-001).
   No false Finding is emitted; the cursor advances ≥ 4 bytes per iteration
   (exact-consume invariant, Invariant 2), so no unbounded work occurs. This is NOT
   a DoS or correctness defect.

   This covers the RFC 8446 §5.1 "MUST NOT span key changes" constraint: the
   unencrypted ClientHello/ServerHello are always in the initial epoch, and `done()`
   fires after both are seen.
5. If the handshake length header declares `body_len > MAX_BUF` (65,536), the message
   is treated as adversarial: the carry buffer for that direction is cleared and
   `TlsAnalyzer.handshake_reassembly_overflows` (aggregate counter, NOT per-flow) is
   incremented (same clear-and-recover outcome as BC-2.07.039 buffer-fill overflow;
   NO abandoned flag; recovery permitted). This guards
   against a length-field-spoofing attack that would otherwise attempt to buffer more
   than MAX_BUF bytes for a single "message." Rationale for raising cap from
   MAX_RECORD_PAYLOAD (18,432) to MAX_BUF (65,536): Go crypto/tls maxHandshake=65536
   is the strictest real-world interoperable ceiling; legitimate large ClientHellos
   (ECH/post-quantum, ~1.5-2.5 KiB) and assembled multi-record messages are reassembled
   correctly up to 65,536 bytes without triggering this guard. See
   TLS-REASSEMBLY-OVERFLOW-POLICY.md §Q4 for the evidence basis.
6. The `handle_client_hello` and `handle_server_hello` call sites within the carry
   drain loop set `client_hello_seen` and `server_hello_seen` respectively — identical
   to the single-record path. The `done()` short-circuit fires correctly after both
   reassembled hellos are seen.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ClientHello split exactly at the SNI extension boundary (first record ends before SNI hostname bytes) | Both records accumulated; full ClientHello dispatched after second record arrives; SNI and JA3 populated correctly |
| EC-002 | First record payload is exactly 1 byte (partial handshake header — type byte only; no length bytes yet) | Byte appended to carry buffer; carry loop breaks immediately (`carry_buf.len() < 4`); no dispatch until remaining bytes arrive |
| EC-003 | Handshake header spans two records (type + 0–2 of 3 length bytes in first record, rest in second) | Carry accumulates; dispatch deferred until `carry_buf.len() >= 4` |
| EC-004 | 0x16 record with a non-ClientHello/non-ServerHello handshake type (e.g., Certificate 0x0B) | Type read, `body_len` decoded, message consumed (exact-consume) without dispatching; `parse_errors` NOT incremented |
| EC-005 | Handshake length header declares `body_len > MAX_BUF` (length-spoofing — a declared length in the header bytes, not an actual oversized payload) | Carry buffer cleared; `TlsAnalyzer.handshake_reassembly_overflows` (aggregate) incremented; no `parse_errors` increment; no finding; subsequent well-formed record still accepted (clear-and-recover, no sticky flag); see Invariant 5. This is a DISTINCT path from BC-2.07.039 EC-002: the trigger here is the DECLARED body_len > MAX_BUF field value, not actual carry-buffer accumulation |
| EC-006 | `done()` is true when a 0x16 record arrives for the same flow | `on_data` short-circuits before carry buffer access; record is silently ignored (BC-2.07.034) |
| EC-007 | Single-record ClientHello (common case, no fragmentation) | Record payload appended to carry buffer (currently empty); consume loop immediately finds a complete message; dispatches in one pass; carry buffer empty after dispatch. Behavior identical to pre-fix path; no regression |
| EC-008 | BC-2.07.004 record-layer oversize guard fires mid-reassembly: a 0x16 record arrives with `payload_len > MAX_RECORD_PAYLOAD` (18,432) while `client_hs_carry` holds partial bytes from earlier records | BC-2.07.004 guard fires FIRST (before the carry append): `client_buf` is cleared, `parse_errors` incremented, `truncated_records` incremented, `on_data` returns. `client_hs_carry` is NOT touched — it retains its prior partial bytes (orphaned partial carry). The orphaned partial carry persists bounded by `MAX_BUF`; it will be silently discarded at `on_flow_close` per BC-2.07.040 (no finding, no additional `parse_errors` increment at close). This is an accepted-risk path: the orphaned carry is bounded, harmless, and cleaned up at flow close. No separate carry-clearing is required by this BC. |
| EC-009 | Attacker sends the maximum number of individually-valid 0x16 records (each `<= MAX_RECORD_PAYLOAD` bytes) without completing a handshake message, maximizing carry-processing work per `on_data` call | Per-record CPU work is bounded: for each record, `try_parse_records` does a record-header peek, a `record_bytes.clone()` into the carry, and a `drain` from the TCP stream buffer. The consume loop breaks immediately (`carry_buf.len() < 4 + body_len`) after each such partial append — only O(1) work per record at the carry layer. The total number of records that can accumulate before triggering the BC-2.07.039 overflow clear is bounded by `MAX_BUF / 1` (at most MAX_BUF single-byte records before carry overflows and is cleared). In practice, record COUNT is bounded by the upstream TCP-reassembly stream reassembler, which limits how many records can arrive per `on_data` invocation. `MAX_BUF / MAX_RECORD_PAYLOAD` (≈4) bounds the number of full-size records that can be accumulated before overflow clears the carry. Per-record work is O(MAX_RECORD_PAYLOAD) clone cost; total per-`on_data` cost is O(MAX_BUF). This is an accepted risk: the upstream stream reassembler is the primary rate-limiting bound; the carry overflow clear is the secondary bound. See `.factory/research/TLS-REASSEMBLY-OVERFLOW-POLICY.md` §Q5 fragmentation-control note. |
| EC-010 | A complete valid handshake message is immediately followed (in the same carry) by a body_len-spoof header — e.g., carry holds `[complete ClientHello bytes ++ spoof_header (declared body_len > MAX_BUF)]` | The drain loop consumes the complete ClientHello first (exact-consume: `4 + body_len_valid` bytes removed); `handle_client_hello` is called; `client_hello_seen=true`; SNI and JA3 populated; `parse_errors=0` for this message. On the next loop iteration, the spoof header becomes the front of the carry; the body_len-spoof guard (Invariant 5) fires: the entire carry buffer is cleared (post-loop drain skipped; any already-dispatched valid prefix is harmlessly discarded — end state: carry empty); `handshake_reassembly_overflows` incremented; drain breaks. **No valid data is lost** — the ClientHello was fully dispatched before the guard fired. This is the F-IMPL-001 coalesced-spoof scenario (see also BC-2.07.042 EC-006 for the drain-loop perspective on this exact scenario). |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ClientHello bytes split 50/50 across two 0x16 records (first 50% in record 1, remainder in record 2) | After record 2: `client_hello_seen=true`, `sni_counts` has expected hostname, `ja3_counts` has entry, `parse_errors=0` | happy-path |
| ClientHello bytes split with first record = 1 byte | After record 2: same as above; `parse_errors=0` | edge-case |
| Complete single-record ClientHello (regression check) | `client_hello_seen=true`, `sni_counts` populated, `parse_errors=0` — identical to pre-fix behavior | regression |
| 0x16 record with Certificate (0x0B) type before ClientHello | Certificate consumed silently; subsequent ClientHello still dispatched; `parse_errors=0` | edge-case |

## Acceptance Criteria

### AC-CANONICAL-FRAME (policy: DF-CANONICAL-FRAME-HOLDOUT-001)

At least one test MUST decode canonical handshake header byte sequences cited verbatim
from RFC 8446 §4, independently of the project's `build_client_hello` /
`build_server_hello` helpers, to pin the 3-byte big-endian length decode against an
authoritative, independently-authored source (anti shared-assumption guard).

The test constructs three canonical frames by hand (no project helpers) and exercises a
distinct behaviour with each:

**Frame A — degenerate body_len (BE-decode baseline):**
- Bytes: `[0x01, 0x00, 0x00, 0x05]` followed by exactly 5 arbitrary body bytes (the test uses 0xff); content is irrelevant because a 5-byte body is below the ClientHello minimum, so parse_tls_message_handshake returns Err regardless.
- Cited from RFC 8446 §4: "struct { HandshakeType msg_type; uint24 length; ... }"
- `body_len` MUST be decoded as `5` (big-endian: `0x00_00_05`)
- This frame has `msg_type=0x01` (ClientHello) and a degenerate 5-byte body
  (too short to be a valid ClientHello; exercises the PC-9 malformed-body path:
  `parse_errors` incremented by 1, exact-consume, no dispatch, no panic)
- Assertion: `body_len == 5`; `parse_errors == 1` after the drain call

**Frame B — big-endian vs little-endian discriminator:**
- Bytes: `[0x01, 0x01, 0x05, 0x00]` ONLY — this is the 4-byte header and nothing more
  (no body bytes follow; the body_len-spoof guard fires on the header decode itself,
  before any body could be appended — and the declared 66,816-byte body would exceed
  MAX_RECORD_PAYLOAD (18,432) anyway, so no single record could carry it)
- If decoded big-endian (correct per RFC 8446 §4): `body_len = 0x01_05_00 = 66,816`
  → exceeds `MAX_BUF (65,536)` → body_len-spoof guard fires on header decode; carry
  cleared; `handshake_reassembly_overflows` incremented; `parse_errors=0`; no dispatch
- If decoded little-endian (incorrect): `body_len = 0x00_05_01 = 1,281` → ≤ MAX_BUF,
  no guard fires → test would wrongly pass without clearing; this discriminates the
  two encodings
- Assertion: `handshake_reassembly_overflows == 1`; carry buffer empty after drain
  (exercises BC-2.07.038 Inv-5 / Decision-4: body_len-spoof guard on header decode)

**Frame C — body_len=256, session-id length overflow (PC-9 malformed-body path):**
- Bytes: `[0x01, 0x00, 0x01, 0x00]` followed by exactly 256 bytes of `0xcc` (body)
- `body_len` MUST be decoded as `256` (big-endian: `0x00_01_00`)
- `msg_type=0x01` (ClientHello); body is length-complete (256 bytes present) but
  internally malformed: after the 2-byte version field and 32-byte random field, the
  session-id length byte arrives at body offset 34 as `0xcc` = 204. `tls_parser` 0.12.2
  enforces `verify(be_u8, |&n| n <= 32)` on this field (`tls_handshake.rs` session-id
  length guard); the guard fails → `parse_tls_message_handshake` returns `Err(_)` →
  exercises PC-9: `parse_errors` incremented by 1, exact-consume `4 + 256 = 260` bytes,
  no finding, no panic; `client_hello_seen=false`
- **Why not all-zero?** An all-zero body has `sidlen=0` (ok), `ciphers_len=0` (ok —
  empty cipher lists are explicitly accepted by `parse_cipher_suites`), trailing zeros
  within declared length discarded → `parse_tls_message_handshake` returns `Ok`
  (degenerate but structurally valid; see PC-9 NOTE above). An all-zero body does NOT
  fire PC-9. The `0xcc` body is the genuinely-malformed vector that triggers the
  tls_parser session-id length guard.
- Assertion: `body_len == 256`; `parse_errors == 1`; `client_hello_seen == false`;
  carry buffer empty after drain (all 260 bytes consumed)

The test `test_BC_2_07_038_canonical_frame_rfc8446_s4` MUST exercise all three frames in
a single test (or as three sub-cases of the same Red-Gate). Bundling is intentional: the
test asserts BE-decode correctness (Frame A), clear-and-recover triggered by the BE→LE
discriminator (Frame B), and PC-9 malformed-body semantics (Frame C) — all as
consequences of the same 3-byte big-endian decode being the only conformant interpretation
of RFC 8446 §4. The `parse_errors+1` assertion for Frame C (and Frame A's degenerate
body) is backed by BC postcondition PC-9. The `handshake_reassembly_overflows+1`
assertion for Frame B is backed **primarily by BC-2.07.038 Inv-5 (the Decision-4
body_len-spoof guard that fires during header decode when `body_len > MAX_BUF`)**;
BC-2.07.039 shares only the clear-carry + increment-counter outcome — it does NOT share
the trigger (BC-2.07.039 is the Decision-5 buffer-fill guard, a distinct trigger: it
fires when accumulating bytes would exceed MAX_BUF, not on header decode).

Red-Gate test name (for VP-039 Sub-A / architect to author in VP-039):
**`test_BC_2_07_038_canonical_frame_rfc8446_s4`**

This test is a Red-Gate: it must FAIL before the carry-buffer implementation is written
(the header bytes are raw input; the decoder does not exist yet). It must pass after the
3-byte big-endian `body_len` decode is implemented exactly as specified in the BC.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-039 (Sub-A) | For any split offset `1 <= k < ClientHello_len`, delivering bytes `[0..k]` then `[k..n]` as two 0x16 records produces `client_hello_seen=true`, `ja3_counts.len()==1`, `sni_counts.len()==1` (when SNI present), `parse_errors=0` | proptest: `proptest_vp039_carry_reassembly_two_record` |
| VP-039 (Sub-A) | Canonical handshake header byte sequence from RFC 8446 §4 is decoded correctly (3-byte big-endian `body_len`); test authored independently of project helpers | unit (Red-Gate): `test_BC_2_07_038_canonical_frame_rfc8446_s4` |
| VP-039 (Sub-A) | Single-record ClientHello produces identical output to pre-fix path (baseline equivalence asserted by two-record proptest split at k=0) | proptest: `proptest_vp039_carry_reassembly_two_record` |
| BC-2.07.042 | Non-ClientHello handshake type consumed without parse_errors increment (coalesced message consumed exactly; parse_errors==0) | unit: `test_BC_2_07_042_exact_consume_no_double_dispatch` |
| VP-039 (Sub-A) | Malformed-but-complete assembled body (PC-9): `parse_errors == 1`, `client_hello_seen == false`, no finding, no panic — Red-Gate: fails before carry implementation, passes after; authored independently of project helpers | unit (Red-Gate): `test_BC_2_07_038_malformed_assembled_body` |
| VP-039 (Sub-A) | Single valid ClientHello reassembled across >=3 records (N-record drip-feed), including a 4-byte handshake header split across two consecutive records: `client_hello_seen=true`, `sni_counts` populated, `parse_errors=0` — exercises PC-1/PC-2/PC-6 and EC-003 re-entrancy across >2 `on_data` calls | unit: `test_vp039_n_record_reassembly` |
| VP-039 (Sub-A) | Large valid ClientHello (body 18,433..65,536 bytes) reassembles and dispatches: `client_hello_seen=true`, `parse_errors=0`, no finding dropped — positively verifies the Inv-5 cap raise from MAX_RECORD_PAYLOAD (18,432) to MAX_BUF (65,536); not dropped, not cleared as a spoof | unit: `test_vp039_large_valid_hello_reassembly` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md — this BC specifies the handshake-message reassembly layer that enables SNI and JA3 extraction from fragmented ClientHellos, a core sub-capability of TLS traffic analysis |
| L2 Domain Invariants | INV-5 (SNI 4-way classification), INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs — `try_parse_records` 0x16 drain path + `TlsFlowState` carry fields) |
| Finding Source | TLS-CLIENTHELLO-FRAG-001 (validated 2026-06-29; see `.factory/research/TLS-CLIENTHELLO-FRAG-001-validation.md`) |
| Pass-1 Reconciliation | F-P1-001/SR-001 (CRITICAL — remove sticky abandon); F-P1-006 (MED — raise body_len cap 18432→65536); F-P1-010 (LOW — downgrade P0→P1); SR-008 (MED — name both drains in Postcondition 8) |
| RFC Authority | RFC 5246 §6.2.1 (TLS 1.2 fragmentation); RFC 8446 §5.1 (TLS 1.3 fragmentation + MUST NOT span key changes) |
| Stories | STORY-144 |
| Origin | greenfield (fix-tls-clienthello-frag cycle) |

## Related BCs

- BC-2.07.001 — composes with (ClientHello dispatch; this BC specifies how the assembled bytes reach handle_client_hello)
- BC-2.07.002 — composes with (ServerHello dispatch; symmetric path for server direction)
- BC-2.07.039 — composes with (carry buffer bound; overflow discard policy)
- BC-2.07.040 — composes with (truncation safety at flow close)
- BC-2.07.041 — composes with (per-flow and per-direction isolation of carry buffers)
- BC-2.07.042 — composes with (coalesced message dispatch — the exact-consume loop enabling it)
- BC-2.07.005 — related to (TCP stream buffer cap; carry buffer cap is separate but uses same MAX_BUF constant)
- BC-2.07.034 — depends on (done() short-circuit fires before carry buffer processing)
- BC-2.07.033 — related to (non-handshake records never feed carry buffer; guard fires before carry append)

## Architecture Anchors

- `src/analyzer/tls.rs` — `TlsFlowState` struct (add `client_hs_carry: Vec<u8>`, `server_hs_carry: Vec<u8>`; NO `handshake_reassembly_overflows` on TlsFlowState; NO abandoned-flag fields)
- `src/analyzer/tls.rs` — `TlsAnalyzer` struct (add `handshake_reassembly_overflows: u64` as aggregate counter; mirrors `truncated_records` which is `u64` at tls.rs:339; NOT per-flow; NOT reset at flow close; surfaced in `summarize()`)
- `src/analyzer/tls.rs` — `TlsFlowState::new()` (initialize carry fields to `Vec::new()` only)
- `src/analyzer/tls.rs` — `try_parse_records` 0x16 record drain path (append payload to direction carry; consume loop; exact-consume dispatch)
- `src/analyzer/tls.rs` — `on_flow_close` (drop carry fields alongside existing state via HashMap remove)
- `src/analyzer/tls.rs` — `client_hs_carry_len_for_testing()` / `server_hs_carry_len_for_testing()` (new test seams; mirrors `client_buf_len_for_testing`)
- `tests/tls_analyzer_tests.rs` — `test_BC_2_07_038_malformed_assembled_body` (Red-Gate: VP-039 Sub-A PC-9; malformed-but-complete assembled body → `parse_errors==1`, `client_hello_seen==false`, no finding)

## Story Anchor

STORY-144 (TLS Carry Buffer + ClientHello Fragmentation Reassembly — BC primary; wave 65)

## VP Anchors

VP-039 (Sub-A, Sub-B)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates `client_hs_carry` or `server_hs_carry`; calls `handle_client_hello` / `handle_server_hello` |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
