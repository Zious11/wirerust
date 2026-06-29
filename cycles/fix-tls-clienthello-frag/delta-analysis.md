---
document_type: phase-f1-delta-analysis
cycle_id: fix-tls-clienthello-frag
finding_id: TLS-CLIENTHELLO-FRAG-001
phase: F1
status: complete
produced_by: architect
date: 2026-06-29
traces_to: .factory/cycles/fix-tls-clienthello-frag/cycle-manifest.md
validation_artifact: .factory/research/TLS-CLIENTHELLO-FRAG-001-validation.md
---

# Phase F1 — Delta Analysis: TLS-CLIENTHELLO-FRAG-001

## 1. Classification

**Classification: Enhancement** (with a security-correctness character).

Justification: The fix introduces genuinely new behavior — per-direction
handshake-message reassembly across TLS record boundaries — that does not exist
in any form today. The existing analyzer is not broken in the sense of producing
wrong output from valid single-record input; it is *incomplete* against the
RFC-permitted fragmented input class. This meets the VSDD definition of
"enhancement": a new behavioral capability that expands the contract surface, not
a repair of a logic error in an existing code path. The security angle (SNI/JA3
evasion is the consequence of the gap) justifies the severity designation HIGH and
mandatory-fix treatment, but the code change is additive rather than corrective.

New BC(s) required. New VP(s) required. Story count is bounded and modest
(see §5).

---

## 2. Impact Boundary

### Primary file

**`src/analyzer/tls.rs`** — the only file that changes.

All four impacted layers live in this single file:

| Layer | Location | Change type |
|-------|----------|-------------|
| `TlsFlowState` struct | L274–301 | CHANGED: add `client_hs_carry: Vec<u8>` and `server_hs_carry: Vec<u8>` fields |
| `TlsFlowState::new()` | L287–295 | CHANGED: initialize carry fields to `Vec::new()` |
| `TlsFlowState::done()` | L298–300 | UNCHANGED: only gates on `client_hello_seen && server_hello_seen` |
| `try_parse_records()` | L654–792 | CHANGED: 0x16 records append payload to carry buf; consume completed handshake messages out of carry; loop until no complete message remains |
| `handle_client_hello()` | L389–580 | UNCHANGED: receives a fully assembled `TlsClientHelloContents`, no change needed |
| `handle_server_hello()` | L586–651 | UNCHANGED: symmetric |
| `on_data()` | L797–839 | UNCHANGED: existing MAX_BUF guard on `client_buf`/`server_buf` remains; carry buffers are independent |
| `on_flow_close()` | L841–843 | CHANGED (minor): drop carry fields alongside existing state on flow removal |

### No other files touched

- `src/reassembly/` — not touched. There is no shared reassembly infra for TLS
  handshake bytes; the carry buffers are local to `TlsFlowState`.
- `src/analyzer/http.rs`, `src/analyzer/modbus.rs`, etc. — not touched.
- `src/findings.rs` — not touched. The `Finding` struct is unchanged.
- `src/dispatcher.rs` — not touched. The dispatch trigger (content fingerprint
  `0x16 0x03`) is unaffected.
- `src/reporter/` — not touched.

### CHANGED vs NEW

| Symbol | Type |
|--------|------|
| `TlsFlowState.client_hs_carry` | NEW field |
| `TlsFlowState.server_hs_carry` | NEW field |
| Carry-buffer drain path in `try_parse_records` | NEW code block |
| Handshake header parser (1-byte type + 3-byte big-endian length) | NEW inline logic |
| `client_hs_carry_len_for_testing()` test seam | NEW (mirrors `client_buf_len_for_testing`) |
| `server_hs_carry_len_for_testing()` test seam | NEW (mirrors `server_buf_len_for_testing`) |
| Rest of `try_parse_records`, all helpers | UNCHANGED |

---

## 3. Affected and Needed Behavioral Contracts

### Existing BCs touched or narrowed by this change

| BC | Current statement | Impact |
|----|------------------|--------|
| **BC-2.07.001** | Parse Complete TLS ClientHello: Version, Ciphers, Extensions, SNI, JA3 | EXPANDED scope: "complete" must now mean assembled across records, not single-record only. BC description requires an amendment clarifying that a fragmented ClientHello is assembled before parsing. |
| **BC-2.07.002** | Parse Complete TLS ServerHello: JA3S | EXPANDED scope (same rationale): ServerHello fragmented across records must also be reassembled. |
| **BC-2.07.003** | After both hellos seen, subsequent records are silently skipped | UNCHANGED in semantics; the `done()` short-circuit still fires after SNI+JA3 are extracted from the assembled hellos. No BC text change needed. |
| **BC-2.07.004** | TLS record payload > MAX_RECORD_PAYLOAD increments parse_errors and truncated_records | UNCHANGED. The existing guard fires on the record layer before bytes touch the carry buffer. No BC text change needed. |
| **BC-2.07.005** | Per-direction buffer capped at MAX_BUF = 65536 bytes | UNCHANGED for `client_buf`/`server_buf`. A new but analogous invariant is needed for carry buffers (new BC below). |
| **VP-005** anchor BCs (BC-2.07.013 through BC-2.07.019, BC-2.07.037) | SNI 4-way ordered classification | UNCHANGED: `extract_sni` operates on a fully assembled `TlsClientHelloContents` regardless of how many records it crossed. |

### New BCs required (proposed IDs in BC-2.07.038 through BC-2.07.042 space)

The current BC numbering ends at BC-2.07.037. Five new BCs are proposed for the
reassembly layer. IDs BC-2.07.038–BC-2.07.042 are the next available slots.

---

**BC-2.07.038 — TLS Handshake-Message Reassembly Across Record Boundaries**

Behavioral statement: When a TLS handshake message (content type 0x16) is
fragmented across two or more consecutive TLS records for the same flow direction,
the analyzer accumulates the handshake-fragment bytes from each record into a
per-direction carry buffer and dispatches `ClientHello` or `ServerHello` only when
the carry buffer contains at least `4 + body_len` bytes (1-byte type + 3-byte
big-endian length header + full body). The dispatch call receives the same
assembled bytes as if the message had arrived in a single record.

Invariants:
1. A message whose type byte is not `ClientHello (0x01)` or `ServerHello (0x02)`
   is parsed for its length header and consumed (advanced past) without dispatching.
2. After dispatching a complete message, exactly `4 + body_len` bytes are removed
   from the carry buffer (exact-consume); any remaining bytes belong to the next
   handshake message in the stream (coalesced-message case).
3. Content types other than `0x16` never feed the carry buffer; the existing
   guard-before-allocate (CR-010) path for non-handshake records is unaffected.

---

**BC-2.07.039 — Handshake Carry Buffer Bounded at MAX_BUF**

Behavioral statement: The per-direction handshake carry buffer (`client_hs_carry`
/ `server_hs_carry`) is bounded at `MAX_BUF = 65,536` bytes per direction. When
new record payload would cause the carry buffer to exceed this cap, all bytes in
that direction's carry buffer are silently discarded; no `parse_errors` increment
occurs and no finding is emitted. The flow remains active and future records are
accepted; only the oversized partial handshake is dropped.

Invariants:
1. `client_hs_carry.len() <= MAX_BUF` and `server_hs_carry.len() <= MAX_BUF` at
   all times after any `on_data` call returns.
2. The bound uses the same `MAX_BUF` constant (65,536) as the TCP-stream record
   buffer (`client_buf`/`server_buf`), maintaining a consistent per-flow memory
   ceiling.

---

**BC-2.07.040 — Truncated Handshake at Flow Close Yields No Finding and No parse_errors Increment**

Behavioral statement: When `on_flow_close` is called for a flow that has an
incomplete handshake message accumulated in either carry buffer (i.e., the
handshake-length header is present but not all body bytes have arrived), the carry
buffer is silently discarded without emitting any finding and without incrementing
`parse_errors`. This preserves existing truncation semantics for snaplen-truncated
captures (READER cand-05 interaction): a capture truncated mid-handshake is
indistinguishable from an incomplete fragment, and both are treated as
"nothing to report" rather than as parse failures.

Invariants:
1. `parse_errors` is incremented ONLY for records that fail `parse_tls_plaintext`
   or whose extension bytes fail `parse_tls_extensions`. An incomplete carry
   buffer at flow close is NOT a parse error.
2. An empty or partial carry buffer at flow close produces zero findings from the
   reassembly layer.

---

**BC-2.07.041 — Handshake Carry Buffers Are Per-Flow and Per-Direction Isolated**

Behavioral statement: Each active flow has its own independent
`client_hs_carry` and `server_hs_carry` vectors. Bytes from flow A's
`client_hs_carry` are never read, written, or cleared in response to data
arriving on flow B, regardless of arrival order. Bytes from the
`client_hs_carry` direction are never mixed into `server_hs_carry` and
vice versa.

Invariants:
1. Carry buffer identity is keyed by `FlowKey`; no carry-buffer access path
   dereferences a `FlowKey` other than the one passed to the current `on_data`
   call (mirrors VP-014 cross-flow isolation invariant for carry buffers).
2. `Direction::ClientToServer` data is appended only to `client_hs_carry`;
   `Direction::ServerToClient` data is appended only to `server_hs_carry`.

---

**BC-2.07.042 — Coalesced Handshake Messages in One Record Are Each Dispatched Independently**

Behavioral statement: When a single TLS record payload contains the complete byte
sequences of two or more consecutive handshake messages (e.g., a
`ClientHello` followed immediately by another handshake type), each message is
parsed and dispatched in order. The exact-consume invariant (BC-2.07.038 Inv-2)
causes the parser to advance to the next message header after each complete
message, so no message is silently skipped. This handles the RFC-permitted
coalescing case (RFC 5246 §6.2.1: "multiple client messages of the same
ContentType MAY be coalesced into a single TLSPlaintext record").

Invariants:
1. All handshake messages present in the carry buffer at any given drain call are
   dispatched in wire order before the drain call returns.
2. After all complete messages are dispatched, the carry buffer contains at most
   `3` bytes (a partial handshake header — type + 0, 1, or 2 of the 3 length
   bytes; anything shorter cannot be a complete header).

---

### BC numbering summary for F2 spec-evolution

Next available BC ID after these five: **BC-2.07.043**.
SS-07 BC count changes from 37 to 42.

---

## 4. Affected and Needed Verification Properties

### Existing VPs affected

| VP | Status | Impact |
|----|--------|--------|
| **VP-005** (SNI 4-way classification; Kani; P0; verified) | No impact to harness. `classify_hostname_vp005` and the Kani proofs operate on the SNI bytes *after* full ClientHello parse. Reassembly happens before those bytes arrive at the classifier. **No harness changes needed.** |
| **VP-013** (JA3 GREASE filter; proptest; P1; verified) | No impact. `compute_ja3` receives the fully assembled ciphers/extensions list. |
| **VP-021** (Timestamp provenance threading; proptest; test-sufficient; verified) | No impact. `last_ts` is updated in `on_data` before `try_parse_records` is called; the threading path is unaffected by carry buffers. |
| **VP-014** (HttpAnalyzer cross-flow isolation; proptest; P1; verified) | No impact to HttpAnalyzer. However, the *analogue* of this VP for TLS carry buffers is mandated by BC-2.07.041 and should be tested as part of the new VP (see below). |

All 38 existing VPs remain unaffected in scope, harness, or status.

### New VP proposed

**VP-039 — TLS Handshake Reassembly: Bounded Carry, Exact-Consume, Truncation-Safety, and Cross-Direction Isolation**

- Module: `analyzer/tls.rs`
- Tool: **proptest** (primary; two harnesses covering the stateful carry path); supplemented by targeted unit tests for the truncation / carry-overflow arms
- Phase: **P1**
- Status: draft
- Verified BCs: BC-2.07.038, BC-2.07.039, BC-2.07.040, BC-2.07.041, BC-2.07.042

Sub-properties:

**Sub-A (proptest harness `proptest_vp039_carry_reassembly_two_record`):**
Property: for any split offset `1 <= k < ClientHello_len` and any valid ClientHello
of total byte length `n`, delivering bytes `[0..k]` as one record payload followed
by bytes `[k..n]` as a second record payload produces `client_hello_seen == true`,
`ja3_counts.len() == 1`, `sni_counts.len() == 1` (when SNI is present), and
`parse_errors == 0`. Covers BC-2.07.038.

**Sub-B (proptest harness `proptest_vp039_exact_consume_coalesced`):**
Property: two complete handshake messages coalesced into one record each dispatch
independently. After both records are processed the carry buffer length is 0.
Covers BC-2.07.042.

**Sub-C (unit test `test_vp039_carry_overflow_silent_drop`):**
Property: when a 0x16 record would push `client_hs_carry` above MAX_BUF, the carry
buffer is cleared, `parse_errors` is unchanged, and no finding is emitted. Covers
BC-2.07.039.

**Sub-D (unit test `test_vp039_truncated_carry_no_error`):**
Property: `on_flow_close` called with an incomplete carry buffer (partial length
header present) does not increment `parse_errors` and does not emit a finding.
Covers BC-2.07.040.

**Sub-E (proptest harness `proptest_vp039_direction_isolation`):**
Property: interleaved `ClientToServer` and `ServerToClient` deliveries of
fragmented hellos produce `client_hello_seen == true` and `server_hello_seen == true`
with no carry-buffer cross-contamination (each carry buffer behaves identically to
an independent same-direction run). Covers BC-2.07.041.

Feasibility assessment: All five sub-properties are testable with proptest/unit
tests on `TlsAnalyzer`. The split-offset and fragmentation shape are straightforward
to generate with the existing `build_client_hello` test helper. No Kani harness is
needed or appropriate here — the property involves stateful accumulation across
multiple `on_data` calls and mutable carry buffers, which is better exercised by
proptest's shrinking than by Kani's BMC (same rationale as VP-014, VP-033–VP-038).
Sub-D and Sub-C are unit-test-only (deterministic single-case). Sub-A, Sub-B, Sub-E
are proptest.

### VP count impact

VP-INDEX v2.14 current total: **38**. After this delta: **39** (VP-039 added).
p1_count: 24 → 25. proptest_count: 16 → 17.

---

## 5. Affected and Needed Stories

### Story count estimate

**2 stories** for F3 decomposition.

**STORY-A (primary):** TLS Handshake Carry Buffer + Fragmented ClientHello Reassembly

Scope:
- Add `client_hs_carry: Vec<u8>` and `server_hs_carry: Vec<u8>` to `TlsFlowState`
- Add `client_hs_carry_len_for_testing()` and `server_hs_carry_len_for_testing()` test seams
- Rewrite the 0x16 record-drain path in `try_parse_records` to: (1) append record payload to carry buffer under MAX_BUF cap, (2) parse handshake headers out of the carry buffer in a loop, (3) dispatch `ClientHello` when a complete message is present, (4) exact-consume dispatched message bytes, (5) break when carry buffer is incomplete
- Accept new BCs: BC-2.07.038, BC-2.07.039, BC-2.07.040, BC-2.07.042 (ClientHello path)
- All new VP-039 sub-property tests except Sub-E (symmetric story)
- Regression: all existing `tls_analyzer_tests.rs` pass (9391 lines; single-record path must be unaffected)

Wave estimate: 1 wave, ~3–5 ACs.

**STORY-B (symmetric):** ServerHello Reassembly + Cross-Direction Isolation

Scope:
- Apply symmetric carry-buffer logic to `server_hs_carry` for the ServerHello path
- Accept new BCs: BC-2.07.041 (cross-direction isolation for both directions), and
  BC-2.07.002 amendment (ServerHello fragmentation coverage)
- VP-039 Sub-E (direction isolation proptest harness)
- Holdout scenarios (see §7)
- Confirm `done()` short-circuit still fires correctly after both reassembled hellos

Wave estimate: 1 wave, ~2–3 ACs.

**Why 2 stories:** ClientHello and ServerHello share the same carry-buffer mechanism
but are driven from independent direction arms in `try_parse_records`. Splitting them
preserves the TDD red-green cycle discipline: STORY-A delivers a green single-direction
path verifiable via existing `client_hello_seen` test seams; STORY-B extends to the
symmetric case and adds the cross-direction isolation property. This mirrors the
approach used for Modbus/DNP3 carry-buffer stories (VP-035/VP-036 split pattern).

---

## 6. Regression Risk

### High-risk surfaces

**1. Single-record ClientHello fast path** (highest regression risk)

The new code must not touch records that were already complete in one record.
Risk: if the carry-buffer drain loop has an off-by-one or the exact-consume logic
miscounts, a previously-working single-record ClientHello could be double-dispatched
(`handshakes_seen` inflated) or dropped entirely.

Mitigation: the test helper `build_client_hello` in `tls_analyzer_tests.rs` (L16–18)
already builds single-record ClientHellos. All 9391 lines of existing `tls_analyzer_tests.rs`
exercise this path and must remain green. The carry-buffer path only activates when
`record_bytes` are appended to the carry buffer; if a single record contains a complete
handshake message, the inner consume loop dispatches it in the same call and leaves the
carry buffer empty — behavior is identical to the current path.

**2. `parse_errors` accounting**

Current code: `parse_errors` increments on `NomErr::Incomplete` or `Err(_)` from
`parse_tls_plaintext` (L783–790), and on `parse_tls_extensions` failure (L405–408,
L598–601). The new carry-buffer overflow (BC-2.07.039) must NOT increment
`parse_errors`. Truncation at flow close (BC-2.07.040) must NOT increment
`parse_errors`. Any violation here creates false inflation of the `parse_errors`
counter in `summarize()` output.

**3. MAX_BUF / memcap**

Two distinct buffer caps exist per direction: `client_buf` (TCP record reassembly,
capped at MAX_BUF by `on_data` L823–825) and `client_hs_carry` (handshake carry,
capped at MAX_BUF by new logic). Both caps must be enforced independently. A
failure to cap `client_hs_carry` while `client_buf` is capped would allow
adversarial input to double the per-flow memory ceiling.

**4. VP-014 cross-flow isolation invariant (TLS analog)**

VP-014 currently covers `HttpAnalyzer`. The TLS carry buffers introduce an analogous
per-flow isolation requirement (BC-2.07.041). The `flows: HashMap<FlowKey, TlsFlowState>`
map already enforces per-flow keying; the carry buffers live inside `TlsFlowState` and
are therefore automatically isolated by the same HashMap keying. No new cross-flow
contamination path is introduced, but the new VP-039 Sub-E harness must explicitly
verify this for the reassembled-hello case.

**5. `done()` short-circuit**

`TlsFlowState::done()` (L298–300) gates on `client_hello_seen && server_hello_seen`.
This must still fire correctly after both hellos are assembled from fragments. Risk: if
the `client_hello_seen = true` flag set at L768 is reached only for single-record
hellos (because the new carry path calls `handle_client_hello` through a different
code route without setting the flag), `done()` never fires and the flow accumulates
data indefinitely.

Mitigation: the flag must be set whenever `handle_client_hello` is called,
regardless of whether the hello was single-record or reassembled. This is
structurally guaranteed if the carry-buffer dispatch calls the same
`handle_client_hello` and flag-set code block as the current path.

**6. `on_flow_close` state cleanup**

`on_flow_close` (L841–843) currently calls `self.flows.remove(flow_key)`. This
drops `TlsFlowState` including the new carry fields — no explicit carry-clearing
needed, since the HashMap remove drops the entire struct. The existing test seam
`active_flows_len_for_testing` (L944–946) verifies flow removal; it will continue
to work because the removal path is unchanged.

### Regression tests that must stay green

- All tests in `tests/tls_analyzer_tests.rs` (9391 lines; covers single-record
  ClientHello, SNI classification, JA3/JA3S, weak ciphers, deprecated protocols,
  parse_errors accounting, flow lifecycle, timestamp threading)
- All tests in `tests/tls_integration_tests.rs` (267 lines; end-to-end with
  `tests/fixtures/tls.pcap`, `tls12-aes256gcm.pcap`, `tls13-rfc8446.pcap`)
- `cargo test --all-targets` (full suite, no regressions)
- `cargo clippy --all-targets -- -D warnings` (no new warnings)

---

## 7. Holdout Scenario Needs

Four new holdout scenarios are required for F4 / phase-f4-holdout.

**HS-NEW-A: Two-Record Fragmented ClientHello (Split at SNI Boundary)**

Split a valid ClientHello so that the SNI extension body is split across the
record boundary (first record contains the ClientHello header and extension type
but not the SNI hostname bytes; second record contains the hostname). Expected:
`sni_counts` contains the correct hostname, `ja3_counts` contains a hash,
`parse_errors == 0`.

**HS-NEW-B: N-Record Fragmented ClientHello (1-Byte First Record)**

First record payload: exactly 1 byte of the ClientHello handshake body. Remaining
bytes delivered in a second record. Mirrors the Kubernetes ingress-nginx reported
pattern (validation §source [14]). Expected: same as HS-NEW-A (SNI extracted,
JA3 computed, parse_errors == 0).

**HS-NEW-C: Fragment + Coalesce (Two Handshake Messages in One Record)**

A ClientHello and a Certificate (or CertificateRequest) handshake message
coalesced in one 0x16 record, where the ClientHello portion was itself fragmented
across the preceding record. Expected: `client_hello_seen == true`,
`parse_errors == 0`, no double-dispatch (handshakes_seen == 1 for the ClientHello
type).

**HS-NEW-D: Snaplen-Truncated ClientHello at Flow Close (Must Not Inflate parse_errors)**

A ClientHello fragmented across two records where the second record is absent
(capture ends mid-handshake, simulating an EPB with `original_len > captured_len`).
Expected: `client_hello_seen == false`, `sni_counts.is_empty()`,
`ja3_counts.is_empty()`, `parse_errors == 0` (the incomplete carry is silently
discarded at flow close — no false positive, no false error).

**HS-NEW-E: Single-Record ClientHello Regression**

A standard single-record TLS 1.2 / 1.3 ClientHello (the common case today). After
the fix: behavior identical to today. Expected: `client_hello_seen == true`,
SNI and JA3 populated, `parse_errors == 0`. This is a regression guard, not a new
capability test.

---

## 8. Open Design Questions for F2

> **SUPERSEDED — Pass-2 adversarial reconciliation (F-F2-007):**
> Q1 and Q2 recommendations below (abandon-direction design + 18,432 cap) have been
> REJECTED by the Pass-1 adversarial review. The final design is:
> - **Q1**: clear-and-recover (Policy A) — NOT abandon-direction (Policy B). Sticky-abandon
>   is a permanent attacker-triggerable blinding primitive (Ptacek/Newsham 1998 pattern);
>   clear-and-recover denies permanence and is industry-aligned (Wireshark, Suricata posture).
> - **Q2**: per-message body_len cap of 65,536 (MAX_BUF = Go crypto/tls maxHandshake) —
>   NOT 18,432 (MAX_RECORD_PAYLOAD). A handshake may legally span several records and
>   legitimately exceed one record's payload; 18,432 would silently drop legitimate large
>   ClientHellos (ECH/post-quantum ~1.5–2.5 KiB multi-record).
> Authoritative decision: **ADR-011 Decision 4 + Decision 5** and
> **`.factory/research/TLS-REASSEMBLY-OVERFLOW-POLICY.md`**.
> The historical text below is PRESERVED for traceability only and must NOT be treated
> as active guidance.

**Q1 — Carry buffer cap behavior: clear-on-overflow vs abandon-direction**

BC-2.07.039 proposes clearing the carry buffer on overflow (silent discard of the
partial handshake). An alternative is to set an `hs_abandoned: bool` flag that
causes all future 0x16 record bytes for that direction to be discarded until flow
close. Tradeoff: clear-on-overflow allows recovery if a subsequent record starts
a fresh handshake message; abandon-direction is simpler and avoids the risk of
misaligned re-parse after a partial discard. RECOMMENDATION: adopt the abandon-direction
approach (set flag, stop processing carry for that direction) — it matches the
existing record-level truncated_records discard pattern (L689–699) and is simpler to
verify. Must be decided at F2 and reflected in BC-2.07.039.

**Q2 — Maximum handshake message size cap (separate from carry cap)**

The carry cap (MAX_BUF = 65,536) is a buffer-fill limit. A separate question is
whether to impose a per-message size cap: if the handshake length header (3-byte
big-endian) declares a body size > some threshold (e.g., > 16,384 bytes — the
TLS 1.2 max record payload), treat it as adversarial and abandon. This guards
against a length-field-spoofing attack that could trick the carry into expecting
65,531 body bytes (3-byte header declares body = 0xFFFF − 4 = 65531) before the
carry-cap fires. Decide at F2: either impose a per-message cap (e.g., `body_len >
MAX_RECORD_PAYLOAD` triggers abandon) or rely on the carry cap alone.

**Q3 — ServerHello inclusion scope in STORY-B**

The cycle manifest scopes the fix to ClientHello (SNI + JA3) because that is the
evasion vector. ServerHello carries JA3S. Should STORY-B apply the symmetric carry
path to ServerHello? RECOMMENDATION: yes — ServerHello fragmentation is equally
permitted by RFC 5246 §6.2.1, and omitting it would leave JA3S subject to the same
gap. This is low additional cost given the symmetric code structure.

**Q4 — Handling of a `ClientHello` length header that spans two records**

If the first record contains only 1, 2, or 3 bytes (partial handshake header —
type byte present but not all 3 length bytes), the carry buffer cannot yet determine
`body_len`. The consume loop must wait. This is the correct behavior but requires
an explicit guard: `if carry_buf.len() < 4 { break; }` before attempting to read
`body_len`. Must be reflected in BC-2.07.038 description as an explicit sub-case
and confirmed in the F2 spec.

**Q5 — RFC 8446 TLS 1.3 `key_update` / post-handshake context**

RFC 8446 §5.1: "Handshake messages MUST NOT span key changes." For the ClientHello
case this is moot (the ClientHello is always in the initial unencrypted epoch,
before any key change). For later handshake messages (post-handshake auth, etc.) the
constraint applies. Since wirerust sets `done()` after both hellos are seen (L298–300)
and ignores subsequent records, the post-key-change MUST NOT span boundary is
automatically respected: the analyzer stops buffering 0x16 records once `done()` is
true. No special handling needed, but F2 spec should document this as an explicit
invariant: "reassembly only applies to the pre-done() epoch; records after `done()`
are not buffered."

---

## 9. Cross-Repo Impact

**Confirmed: none.**

wirerust is a single Rust crate (`src/lib.rs` + `src/main.rs`, single `Cargo.toml`).
There are no sub-crates, no workspace members, and no other repositories that depend
on `wirerust` as a library crate. The change is entirely internal to
`src/analyzer/tls.rs`. No API surface change: `TlsAnalyzer` is not part of the
public API (it is constructed and used only through the `StreamHandler` / `StreamAnalyzer`
traits). The `TlsFlowState` struct is private. All new test seams
(`client_hs_carry_len_for_testing`, `server_hs_carry_len_for_testing`) follow the
existing `#[doc(hidden)] pub fn` test-seam convention and do not affect the public API.

The only external facing behavior change is in the analyzer's output (SNI and JA3
now populated for fragmented ClientHellos), which is an additive capability
improvement in the JSON/CSV/terminal report output. No breaking schema change.

---

## Summary

| Field | Value |
|-------|-------|
| Classification | Enhancement (security-correctness; new behavioral capability) |
| Primary file | `src/analyzer/tls.rs` (only file changed) |
| Changed symbols | `TlsFlowState` (2 new fields), `try_parse_records` (0x16 path), `on_flow_close` (minor), 2 new test seams |
| Existing BCs amended | BC-2.07.001, BC-2.07.002 (scope expansion — fragmented input class) |
| New BCs proposed | BC-2.07.038, BC-2.07.039, BC-2.07.040, BC-2.07.041, BC-2.07.042 (5 new; SS-07 grows from 37 to 42) |
| New VP proposed | VP-039 (proptest; P1; 5 sub-properties; draft; `analyzer/tls.rs`) |
| VP count after delta | 39 (38 → 39; proptest_count 16 → 17; p1_count 24 → 25) |
| Story count (F3) | 2 stories (STORY-A: ClientHello carry + SubA/B/C/D; STORY-B: ServerHello symmetry + Sub-E) |
| Holdout scenarios | 5 (HS-NEW-A through HS-NEW-E) |
| Top regression risks | Single-record fast path unchanged; parse_errors not inflated; MAX_BUF carry cap; done() fires after reassembled hellos; VP-014-analog direction isolation |
| Cross-repo impact | None (single crate, private struct, no public API change) |
