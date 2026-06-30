---
document_type: behavioral-contract
level: L3
version: "2.4"
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
  - "v2.0: Pass-1 adversarial reconciliation (F-P1-001/SR-001 CRITICAL, F-P1-005/SR-006 MED) — adopt clear-and-recover overflow policy (Policy A) per TLS-REASSEMBLY-OVERFLOW-POLICY.md; remove hs_carry_abandoned flag and sticky-abandon semantics; fix Inv-2 memory-ceiling wording; add exact-fit EC explicitly — 2026-06-29"
  - "v2.1: Pass-2 adversarial reconciliation (F-F2-002 HIGH, F-F2-003 HIGH) — PC-2 counter-home clarified: handshake_reassembly_overflows lives on TlsAnalyzer (aggregate across all flows), NOT on TlsFlowState; it is NOT reset at flow close; EC-002 rewritten: a single record cannot reach carry overflow because BC-2.07.004/BC-2.07.038 PC-3 reject any record with payload_len > MAX_RECORD_PAYLOAD (18,432) at the record layer before touching the carry; EC-002 now describes ACCUMULATION across multiple <=MAX_RECORD_PAYLOAD records (e.g., carry at 49,200 from 3 prior records; 4th 16,400-byte record pushes total to 65,600 > MAX_BUF); separate EC added for the body_len-spoof path (declared length > MAX_BUF, not an actual oversized payload); Inv-5 anomaly-signal clarified to rely on the analyzer-aggregate counter — 2026-06-29"
  - "v2.2: Pass-3 adversarial reconciliation (F-P3-001 HIGH, F-P3-004 MEDIUM) — Architecture Anchors: handshake_reassembly_overflows type corrected u32→u64 to mirror truncated_records (u64 at tls.rs:319); PC-7 added: handshake_reassembly_overflows MUST be surfaced as a key in the summarize() JSON detail map, mirroring truncated_records (tls.rs:888-889); Red-Gate test name added: test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key — 2026-06-29"
  - "v2.3: Fix burst 6 adversarial reconciliation (F-FRESH-003 MEDIUM) — Inv-2 4×MAX_BUF ceiling qualified as a POST-on_data-return RESIDUE bound (mirrors BC-2.07.005 Observability Note convention); in-call transient peak higher: includes the record_bytes clone simultaneously live with client_buf drain + client_hs_carry bytes; '256 KiB hard peak' claim removed because it overstates the in-call peak (clone is transient, freed before on_data returns) — 2026-06-29"
  - "v2.4: Fix burst 9 adversarial reconciliation (F-EV-002 MEDIUM) — EC-009 added: mid-legitimate-assembly overflow-clear residual risk — an overflow-clear (buffer-fill or body_len-spoof) that fires while a legitimate ClientHello is being assembled discards the in-progress bytes; recovery requires a subsequent well-formed record; a real TLS client will not re-send a mid-handshake fragment (handshake layer is not a request-response protocol for individual records), so that handshake's SNI/JA3 may be permanently missed for that flow; this is the ACCEPTED BOUNDED outcome per TLS-REASSEMBLY-OVERFLOW-POLICY.md: clear-and-recover was chosen over sticky-abandon precisely to bound and deny permanence; the alternative (sticky-abandon) permanently blinds the entire direction with one packet; mid-assembly loss is the bounded residual risk of the chosen policy — 2026-06-29"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.039: Handshake Carry Buffer Bounded at MAX_BUF with Clear-and-Recover Overflow Policy

## Description

The per-direction handshake carry buffer (`client_hs_carry` / `server_hs_carry`) is
bounded at `MAX_BUF = 65,536` bytes per direction. When new 0x16 record payload would
cause the carry buffer to exceed this cap, the carry buffer for that direction is
silently cleared (all accumulated bytes discarded), a `handshake_reassembly_overflows`
counter is incremented (one per direction-overflow event), and processing continues —
there is NO sticky abandoned flag. A subsequent well-formed 0x16 record re-populates
the carry and can still be parsed and dispatched normally. This is the clear-and-recover
policy (Policy A), selected over sticky abandon-direction (Policy B) on evasion-resistance
grounds: a sticky-abandon flag is a one-packet, permanent, attacker-triggered blinding
primitive (Ptacek/Newsham desync; Suricata CVE-2019-18792 precedent); clear-and-recover
denies permanence. See `.factory/research/TLS-REASSEMBLY-OVERFLOW-POLICY.md` for the
full evidence basis. Consistency: the existing per-record overflow handler in tls.rs
L689-698 already implements clear-and-recover for MAX_RECORD_PAYLOAD oversize records.

## Preconditions

1. `TlsAnalyzer::on_data` is processing a 0x16 record for a given flow direction.
2. Appending the record payload would cause `carry_buf.len() + record_payload.len()`
   to exceed `MAX_BUF`.

## Postconditions

1. The carry buffer for the affected direction is cleared (all accumulated bytes
   discarded; `carry_buf.clear()`).
2. The `handshake_reassembly_overflows` counter on `TlsAnalyzer` (NOT on
   `TlsFlowState`) is incremented by 1. This is an aggregate counter across all flows,
   mirroring the `truncated_records` counter pattern. It is NOT reset at flow close —
   `on_flow_close` drops the per-flow `TlsFlowState` but never touches
   `handshake_reassembly_overflows`. Consistent with BC-2.07.041 PC-3 and Invariant 5
   below.
3. `parse_errors` is NOT incremented.
4. No finding is emitted (no `Finding` pushed to `all_findings`).
5. The flow remains active; future non-0x16 records and the opposite direction continue
   to be processed normally.
6. Future 0x16 records for the affected direction ARE accepted normally — they are
   appended to the now-empty carry buffer. There is NO sticky abandoned state. A
   subsequent well-formed handshake record re-populates the carry and dispatches
   normally.
7. The `handshake_reassembly_overflows` counter MUST be surfaced as a key
   `"handshake_reassembly_overflows"` in the `detail` map returned by
   `TlsAnalyzer::summarize()`, in the same manner as `"truncated_records"` is inserted
   at `tls.rs:888-889`. This backs ADR-011 Decision 5's observability rationale:
   the counter is not a private diagnostic field — it is part of the durable telemetry
   surface accessible to consumers of `AnalysisSummary`. Red-Gate test:
   `test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key` — an overflow
   event is triggered; `summarize()` is called; the test asserts
   `detail["handshake_reassembly_overflows"]` exists and equals the expected count.

## Invariants

1. `client_hs_carry.len() <= MAX_BUF` and `server_hs_carry.len() <= MAX_BUF` at all
   times after any `on_data` call returns.
2. The carry buffer cap (`MAX_BUF = 65,536`) is INDEPENDENT of and ADDITIVE to the
   TCP-stream record buffer cap (`client_buf` / `server_buf` in BC-2.07.005, also
   `MAX_BUF`). **Post-`on_data`-return residue ceiling** (mirroring BC-2.07.005
   Observability Note convention): 4 × MAX_BUF ≈ 256 KiB per flow
   (client_buf + server_buf + client_hs_carry + server_hs_carry), not 2 × MAX_BUF.
   This is the maximum memory held in durable per-flow state AFTER `on_data` returns.
   See ADR-011 Consequences for the per-flow budget decision.

   **Transient in-call peak note:** During a single `on_data` invocation the in-call
   memory peak is transiently higher than 256 KiB. The `record_bytes` `clone()` of the
   incoming 0x16 record payload is simultaneously live in the stack frame alongside the
   existing `client_buf` (or `server_buf`) bytes and the existing `client_hs_carry` (or
   `server_hs_carry`) bytes. The `record_bytes` clone is freed before `on_data` returns,
   so it does not contribute to the post-return residue ceiling. The statement "256 KiB
   hard peak" would overstate the in-call peak; this invariant intentionally scopes the
   ceiling to the post-return residue only.
3. The overflow check fires BEFORE appending bytes: if
   `carry_buf.len() + record_payload.len() > MAX_BUF`, the clear path is taken without
   a partial append. There is no intermediate state where the carry buffer exceeds
   `MAX_BUF`.
4. There is NO `hs_carry_abandoned` flag anywhere in `TlsFlowState`. The concept of
   "abandoned direction" does not exist in this design.
5. Carry buffer overflow is observable: `TlsAnalyzer.handshake_reassembly_overflows`
   (an aggregate counter on the analyzer, NOT a per-flow field) is incremented and
   survives beyond any individual flow close. Repeated overflow increments across flows
   are an anomaly signal available to `summarize()`. Unlike the TCP buffer overflow in
   BC-2.07.005 Inv-3, carry overflow is NOT entirely silent — the analyzer-level counter
   provides durable telemetry that is NOT lost when the overflowing flow is closed.
6. The per-message body_len guard (BC-2.07.038 Invariant 5: `body_len > MAX_BUF`
   triggers clear) is a separate, earlier guard that fires during the carry consume loop
   before this buffer-size check. Both guards produce the same clear outcome (no
   abandoned flag) but are triggered at different points in the data flow.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Carry buffer at 65,535 bytes; incoming 0x16 record payload is 2 bytes | 2-byte payload would bring carry to 65,537 > MAX_BUF → carry cleared; `handshake_reassembly_overflows` incremented; `parse_errors=0`; no finding; subsequent records accepted |
| EC-002 | Carry buffer at 49,200 bytes (accumulated across 3 prior <=18,432-byte 0x16 records); 4th 0x16 record has payload_len 16,400 bytes (individually valid per BC-2.07.004 / BC-2.07.038 PC-3 guard, <=18,432) | 49,200 + 16,400 = 65,600 > MAX_BUF → carry cleared; `TlsAnalyzer.handshake_reassembly_overflows` incremented; `parse_errors=0`; subsequent records accepted. Note: a single record payload can NEVER exceed MAX_RECORD_PAYLOAD (18,432) — BC-2.07.004 / BC-2.07.038 PC-3 reject it before it reaches the carry; overflow is therefore always an ACCUMULATION across multiple individually-valid records |
| EC-003 | Carry buffer at 65,536 bytes exactly (full); incoming 0x16 record payload is 1 byte | 1-byte payload would bring carry to 65,537 > MAX_BUF → carry cleared; `handshake_reassembly_overflows` incremented; future 0x16 records accepted |
| EC-004 | Overflow on client direction only | Client carry cleared; `handshake_reassembly_overflows` incremented; `server_hs_carry` unaffected; server direction continues normally; next valid ClientHello on client direction accepted |
| EC-005 | Exact-fit: carry buffer accumulates to exactly MAX_BUF bytes (65,536) | No overflow triggered (condition is `> MAX_BUF`, not `>= MAX_BUF`); carry is full but valid; next drain call may dispatch one or more complete messages; no `handshake_reassembly_overflows` increment |
| EC-006 | Post-overflow recovery: carry cleared by overflow; then a well-formed single-record ClientHello arrives | ClientHello payload appended to now-empty carry; complete message detected; `handle_client_hello` dispatched; `client_hello_seen=true`; SNI and JA3 populated; `parse_errors=0` |
| EC-007 | Repeated overflow: attacker continuously sends oversized fragments to prevent reassembly | Each overflow increments `handshake_reassembly_overflows`; carry is cleared each time; no sticky blindness; each legitimate well-formed record in a clean window is still processed; overflow count itself is an anomaly signal |
| EC-008 | Adversarial handshake declares body_len > MAX_BUF via length-spoofed header (Inv-5 of BC-2.07.038) | Body_len > MAX_BUF guard fires in carry consume loop; carry cleared; `handshake_reassembly_overflows` incremented (same outcome as this BC's overflow path; same clear-not-abandon semantics) |
| EC-009 | **Mid-legitimate-assembly overflow-clear (residual risk — F-EV-002; accepted bounded outcome):** An overflow-clear event (either buffer-fill overflow per this BC, or body_len-spoof guard per BC-2.07.038 Inv-5) fires while a legitimate ClientHello is being assembled across multiple records — i.e., the carry holds partial bytes of a real ClientHello at the moment the clear fires | The in-progress ClientHello bytes are discarded along with all other carry bytes (total clear). Recovery requires a subsequent well-formed record to re-populate the carry. However, a real TLS client operating under the TLS handshake protocol will not re-send individual handshake record fragments that were already transmitted — the handshake layer is not a per-record request-response protocol. Consequently, **the SNI and JA3 of that specific flow may be permanently missed** for that flow's lifetime. This is the **accepted bounded residual risk of the clear-and-recover policy (Policy A)**, explicitly acknowledged in `.factory/research/TLS-REASSEMBLY-OVERFLOW-POLICY.md` §Q2 and §Q5. The policy trade-off is: clear-and-recover bounds the loss (at most one handshake's SNI/JA3 missed per overflow event, and only if the clear fires mid-assembly); sticky-abandon (Policy B) would permanently blind the entire direction for the whole flow duration with a single adversarial packet. Clear-and-recover was chosen because its residual risk is bounded, per-event, and not attacker-controlled-permanent. The `handshake_reassembly_overflows` counter signals that mid-assembly loss may have occurred; repeated overflows on one flow are themselves an anomaly signal. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Carry buffer accumulates to 65,534 bytes via multiple 0x16 records; next record payload is 3 bytes | Carry cleared; `handshake_reassembly_overflows=1`; `parse_errors=0`; no finding; carry is empty (len=0) | edge-case |
| Post-overflow: send a valid single-record ClientHello for same direction | ClientHello dispatched; `client_hello_seen=true`; SNI/JA3 populated; `parse_errors=0` — recovery confirmed | edge-case |
| Overflow on client direction; valid ServerHello arrives on server direction immediately after | ServerHello dispatched normally; `server_hello_seen=true`; `client_hs_carry` is empty; `handshake_reassembly_overflows=1` | edge-case |
| Exact-fit carry (65,536 bytes); next 0x16 payload is 0 bytes (empty record) | No overflow (0-byte append cannot exceed MAX_BUF when carry == MAX_BUF if condition is strict >); no change to `handshake_reassembly_overflows` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-039 (Sub-C) | When a 0x16 record would push `client_hs_carry` above MAX_BUF, the carry buffer is cleared, `handshake_reassembly_overflows` is incremented by 1, `parse_errors` is unchanged, and no finding is emitted | unit: `test_vp039_carry_overflow_clear_and_recover` |
| VP-039 (Sub-C) | After overflow-clear, a subsequent well-formed 0x16 ClientHello record is dispatched normally (`client_hello_seen=true`, SNI populated) | unit: `test_vp039_carry_overflow_recovery` |
| — | Invariant 1: carry buffer length never exceeds MAX_BUF after any on_data call | proptest: fuzz multi-record on_data calls with arbitrary 0x16 payloads |
| — | No hs_carry_abandoned field exists on TlsFlowState | structural: compile-time (absent field) |
| VP-039 (Sub-C) | PC-7: `summarize()` detail map contains key `"handshake_reassembly_overflows"` with the correct count after an overflow event | unit (Red-Gate): `test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md — bounded carry buffer with evasion-resistant overflow handling is a resource-safety and security invariant of the TLS analysis subsystem |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs — `TlsFlowState` carry fields + `handshake_reassembly_overflows` counter + overflow check in `try_parse_records`) |
| Finding Source | TLS-CLIENTHELLO-FRAG-001; Pass-1 adversarial findings F-P1-001/SR-001 (CRITICAL), F-P1-005/SR-006 (MED) |
| Design Decision | Pass-1 F2 reconciliation: clear-and-recover (Policy A) adopted over sticky abandon-direction (Policy B). Rationale: sticky-abandon is a one-packet permanent blinding primitive (Ptacek/Newsham desync; Suricata CVE-2019-18792); clear-and-recover matches Wireshark/Suricata norms and wirerust's existing per-record oversize handling (tls.rs L689-698). See `.factory/research/TLS-REASSEMBLY-OVERFLOW-POLICY.md`. |
| Stories | STORY-144 |
| Origin | greenfield (fix-tls-clienthello-frag cycle) |

## Related BCs

- BC-2.07.038 — composes with (carry buffer used by reassembly; overflow guard fires before append)
- BC-2.07.005 — related to (same MAX_BUF constant; carry cap is ADDITIVE to TCP stream buffer cap — per-flow ceiling is 4×MAX_BUF, not 2×MAX_BUF; see Invariant 2)
- BC-2.07.040 — related to (truncated carry at flow close is a separate case — no overflow counter; just discarded)
- BC-2.07.041 — depends on (per-direction isolation; each direction clears independently)

## Architecture Anchors

- `src/analyzer/tls.rs:30` — `const MAX_BUF: usize = 65_536` (shared constant)
- `src/analyzer/tls.rs` — `TlsFlowState` struct: `client_hs_carry: Vec<u8>`, `server_hs_carry: Vec<u8>` (NO abandoned-flag fields; NO per-flow overflow counter)
- `src/analyzer/tls.rs` — `TlsAnalyzer` struct: `handshake_reassembly_overflows: u64` (aggregate counter across all flows; mirrors `truncated_records` which is `u64` at tls.rs:319; NOT a per-flow field; NOT reset at flow close)
- `src/analyzer/tls.rs` — `try_parse_records` 0x16 drain path: overflow check before append; `carry.clear()` + `self.handshake_reassembly_overflows += 1` on overflow; continue (no break/skip)
- `tests/tls_analyzer_tests.rs` — `test_vp039_carry_overflow_clear_and_recover`
- `tests/tls_analyzer_tests.rs` — `test_vp039_carry_overflow_recovery`
- `tests/tls_analyzer_tests.rs` — `test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key` (Red-Gate: VP-039 Sub-C PC-7)

## Story Anchor

STORY-144 (TLS Carry Buffer + ClientHello Fragmentation Reassembly — BC primary; wave 65)

## VP Anchors

VP-039 (Sub-C)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates `client_hs_carry` or `server_hs_carry` (clear); mutates `handshake_reassembly_overflows` counter |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
