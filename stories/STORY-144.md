---
document_type: story
story_id: STORY-144
title: "TLS Handshake Carry Buffer + Fragmented ClientHello Reassembly (BC-2.07.038/039/040/042 + amended BC-2.07.001)"
epic_id: E-5
wave: 65
points: 8
phase: f3
tdd_mode: strict
status: draft
feature_id: fix-tls-clienthello-frag
github_issue: fix-tls-clienthello-frag
subsystems: [SS-07]
target_module: analyzer/tls
depends_on: []
blocks: [STORY-145, STORY-146]
behavioral_contracts:
  - BC-2.07.038
  - BC-2.07.039
  - BC-2.07.040
  - BC-2.07.042
  - BC-2.07.001
verification_properties:
  - VP-039
assumption_validations: []
risk_mitigations: []
# BC status: all BCs authored and anchored; status remains draft pending PO wave assignment
inputs:
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.038.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.039.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.040.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.042.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.001.md
  - .factory/specs/architecture/decisions/ADR-011-tls-handshake-reassembly.md
  - .factory/cycles/fix-tls-clienthello-frag/delta-analysis.md
input-hash: "52fb717"
---

# STORY-144: TLS Handshake Carry Buffer + Fragmented ClientHello Reassembly

## Narrative

**As a** security analyst relying on wirerust TLS SNI and JA3 detections,
**I want** the TLS analyzer to accumulate fragmented ClientHello handshake bytes
across multiple TLS 0x16 records into a per-direction carry buffer and dispatch
`handle_client_hello` only when the full handshake message is assembled,
**so that** an adversary fragmenting the ClientHello across record boundaries (RFC
5246 §6.2.1 / RFC 8446 §5.1) cannot evade SNI-based detection or JA3 fingerprinting
(TLS-CLIENTHELLO-FRAG-001).

## Behavioral Contracts

| BC ID | Version | Title | Story Role |
|-------|---------|-------|-----------|
| BC-2.07.038 | v2.7 | TLS Handshake-Message Reassembly Across Record Boundaries | Primary: carry-buffer struct fields, drain loop, 3-byte BE length decode, exact-consume, PC-9 malformed-body semantics, canonical-frame AC |
| BC-2.07.039 | v2.4 | Handshake Carry Buffer Bounded at MAX_BUF (Clear-and-Recover) | Carry cap enforcement, `handshake_reassembly_overflows` counter, no sticky-abandon, `summarize()` surfacing |
| BC-2.07.040 | v1.3 | Truncated Handshake at Flow Close Yields No Finding and No parse_errors Increment | `on_flow_close` carry discard semantics; no false parse_errors inflation |
| BC-2.07.042 | v1.4 | Coalesced Handshake Messages in One Record Dispatched Independently | Exact-consume drain loop handles back-to-back messages; wire-order dispatch |
| BC-2.07.001 | v1.9 | Parse Complete TLS ClientHello: Version, Ciphers, Extensions, SNI, JA3 | Scope expansion: "complete" now includes multi-record reassembled path; single-record fast path unchanged |

## Acceptance Criteria

### AC-144-001: `TlsFlowState` gains `client_hs_carry` and `server_hs_carry` fields; `TlsAnalyzer` gains `handshake_reassembly_overflows: u64` aggregate counter
**Traces to:** BC-2.07.038 v2.7 Architecture Anchors; BC-2.07.039 v2.4 Invariant 5; ADR-011 Decision 1

`TlsFlowState` gets two new fields (NO other new fields):
- `client_hs_carry: Vec<u8>` — initialized to `Vec::new()` in `TlsFlowState::new()`
- `server_hs_carry: Vec<u8>` — initialized to `Vec::new()` in `TlsFlowState::new()`

`TlsAnalyzer` gets one new u64 aggregate counter (NOT on `TlsFlowState`):
- `handshake_reassembly_overflows: u64` — initialized to `0` in `TlsAnalyzer::new()`; mirrors `truncated_records: u64` at tls.rs:339; NOT reset at flow close

NO `hs_carry_abandoned` flag exists anywhere in `TlsFlowState` (BC-2.07.039 Invariant 4; ADR-011 Decision 5).

Two test seams added (matching the `#[doc(hidden)] pub fn` convention of siblings):
- `TlsAnalyzer::client_hs_carry_len_for_testing(&self, flow_key: &FlowKey) -> usize`
- `TlsAnalyzer::server_hs_carry_len_for_testing(&self, flow_key: &FlowKey) -> usize`
- `TlsAnalyzer::handshake_reassembly_overflow_count(&self) -> u64` (accessor for the aggregate counter)

**Red-Gate test:** `test_BC_2_07_038_canonical_frame_rfc8446_s4` — must FAIL before carry implementation; asserts three canonical RFC 8446 §4 frames (A/B/C) with hand-crafted header bytes and their BC-2.07.038 AC-CANONICAL-FRAME behaviors (traces to BC-2.07.038 v2.7 AC-CANONICAL-FRAME + Inv-5)

### AC-144-002: `try_parse_records` 0x16 path appends payload to direction carry; drain loop dispatches when `carry_buf.len() >= 4 + body_len`
**Traces to:** BC-2.07.038 v2.7 Postconditions 1–6; BC-2.07.042 v1.4 Postconditions 1–5; ADR-011 Decision 3–4

The existing single-record 0x16 dispatch path is replaced by:
1. **Overflow check before append (Decision 5):** if `carry_buf.len() + record_payload.len() > MAX_BUF`, clear carry and `self.handshake_reassembly_overflows += 1`; continue to next record (BC-2.07.039 Invariants 3–4).
2. **Append:** `carry_buf.extend_from_slice(record_payload_bytes)`.
3. **Drain loop:** `loop { if carry_buf.len() < 4 { break; } let body_len = u24_big_endian(&carry_buf[1..4]); if body_len > MAX_BUF { carry_buf.clear(); self.handshake_reassembly_overflows += 1; break; } if carry_buf.len() < 4 + body_len { break; } /* dispatch or advance */ carry_buf.drain(..4 + body_len); }` (BC-2.07.038 Postcondition 6; Invariant 5; BC-2.07.042 Invariant 1(a/b/c)).
4. **Dispatch:** `msg_type == 0x01` → `parse_tls_message_handshake` → on `Ok` call `handle_client_hello`; on `Err` → `parse_errors += 1`, no finding, no panic (BC-2.07.038 Postcondition 9 / ADR-011 Decision 4). `msg_type == 0x02` → handled in STORY-145. Any other type → consume silently, `parse_errors` NOT incremented (BC-2.07.038 Invariant 1; BC-2.07.042 EC-002).
5. Exact-consume: `carry_buf.drain(..4 + body_len)` fires regardless of Ok/Err (BC-2.07.038 Postcondition 4; Invariant 2).
6. **Non-0x16 records** never touch the carry buffer (BC-2.07.038 Invariant 3).
7. The `done()` short-circuit in `on_data` fires BEFORE `try_parse_records` — records after `done()` are never buffered (BC-2.07.038 Invariant 4; EC-006).

**Direction-parameterized drain loop (STORY-145 additivity requirement):** The `ClientToServer` carry logic MUST be implemented as a carry-reference selection followed by the shared drain loop body, so that STORY-145 can ADD the `ServerToClient` path by adding a second match arm that selects `server_hs_carry` and calls `handle_server_hello` for `msg_type == 0x02`. Do NOT implement the `ClientToServer` path in a way that bakes in the dispatch target — use a form such as: `let carry_buf = match direction { Direction::ClientToServer => &mut state.client_hs_carry, Direction::ServerToClient => &mut state.server_hs_carry, };` so STORY-145 reuses the identical loop.

The single-record ClientHello regression path is preserved: carry is empty → append → loop finds complete message → dispatch in one pass → carry empty again (BC-2.07.038 EC-007; BC-2.07.001 v1.9 Invariant 5). (traces to BC-2.07.038 v2.7 Postconditions 1–4, Invariants 1–4; BC-2.07.042 v1.4 Postconditions 1–5)

**Red-Gate tests (VP-039 Sub-A):**
- `proptest_vp039_carry_reassembly_two_record` — any split offset `1<=k<n`; asserts `client_hello_seen==true`, `sni_counts.len()==1`, `parse_errors==0`
- `test_BC_2_07_038_canonical_frame_rfc8446_s4` — Frame A (degenerate body_len=5, parse_errors+1), Frame B (BE-vs-LE discriminator body_len=66816>MAX_BUF, handshake_reassembly_overflows+1), Frame C (body_len=256 malformed, parse_errors+1, client_hello_seen=false)
- `test_BC_2_07_038_malformed_assembled_body` — length-complete header + malformed body → parse_errors+1, carry empty, no finding, no panic, client_hello_seen=false
- `test_vp039_n_record_reassembly` — ONE ClientHello drip-fed across >=3 records (header-split scenarios); sni_counts.len()==1, parse_errors==0
- `test_vp039_large_valid_hello_reassembly` — body 18,433..65,536 bytes (large valid hello); SNI/JA3 populated, handshake_reassembly_overflows==0

**Red-Gate tests (VP-039 Sub-A — additional):**
- `test_vp039_sni_boundary_deterministic` — a ClientHello split at the exact SNI field boundary (two records: first record ends mid-SNI bytes, second record completes them); asserts SNI extracted correctly (`sni_counts.len()==1`, SNI value matches expected hostname), `parse_errors==0`, `client_hello_seen==true` (traces to BC-2.07.038 v2.7 EC-001 "SNI boundary split")

**Red-Gate test (VP-039 Sub-B):**
- `proptest_vp039_exact_consume_coalesced` — ClientHello + other_msg in one record; handshake_count()==1, parse_errors==0, carry_len==0 after drain
- `test_BC_2_07_042_exact_consume_no_double_dispatch` — no double-dispatch; handshakes_seen exact

**Red-Gate test (VP-039 Sub-F — STORY-144 sole owner):**
- `proptest_vp039_carry_bounded_invariant` — generative invariant: for any sequence of `on_data` calls with arbitrary byte slices on arbitrary directions, `client_hs_carry_len_for_testing` and `server_hs_carry_len_for_testing` are both `<= MAX_BUF` after every call; confirms the overflow-check-before-append invariant holds universally (traces to BC-2.07.039 v2.4 Invariant 1)

### AC-144-003: Carry overflow fires clear-and-recover; `handshake_reassembly_overflows` surfaced in `summarize()`
**Traces to:** BC-2.07.039 v2.4 Postconditions 1–7; Invariants 1–6; ADR-011 Decision 5

When `carry_buf.len() + record_payload.len() > MAX_BUF` (Decision 5 buffer-fill guard):
- `carry_buf.clear()` (NOT `carry_buf.push(...)`)
- `self.handshake_reassembly_overflows += 1` (aggregate on `TlsAnalyzer`, NOT `TlsFlowState`)
- `parse_errors` NOT incremented
- No finding emitted
- Processing continues: `continue` to next record (no sticky abandoned flag)
- A subsequent well-formed 0x16 ClientHello on the same direction IS dispatched normally (clear-and-recover, Policy A; ADR-011 Decision 5)

`summarize()` inserts `"handshake_reassembly_overflows"` into the `detail` `HashMap<String, Value>` with the u64 value (value-equality, not mere key presence). Mirrors `"truncated_records"` at tls.rs:1223-1226.

**Red-Gate tests (VP-039 Sub-C):**
- `test_vp039_carry_overflow_clear_and_recover` — valid header body_len=65,500 + 4 padding records accumulate to 66,004 > MAX_BUF; Decision-5 fires exactly once; carry_len==0; overflow_count==overflows_before+1; parse_errors unchanged; findings_count snapshot before==after
- `test_vp039_carry_overflow_recovery` — post-overflow single-record ClientHello dispatched; client_hello_seen==true; SNI/JA3 populated; parse_errors==0
- `test_vp039_body_len_spoof` — body_len=65537>MAX_BUF triggers Decision-4 clear-and-recover; overflow_count+1; parse_errors unchanged; findings snapshot unchanged
- `test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key` — detail["handshake_reassembly_overflows"].as_u64()==1 (value-equality)

(traces to BC-2.07.039 v2.4 Postconditions 1–7, Invariants 1–6)

### AC-144-004: `on_flow_close` silently discards incomplete carry with no `parse_errors` increment and no finding
**Traces to:** BC-2.07.040 v1.3 Postconditions 1–5; Invariants 1–4

`on_flow_close` calls `self.flows.remove(flow_key)` which drops `TlsFlowState` including `client_hs_carry` and `server_hs_carry`. No additional carry-clearing step is needed. No `parse_errors` increment. No finding. Behavior is identical for 0-byte, 1-3 byte (partial header), header-complete-but-body-incomplete, and overflow-cleared (empty) carry states. The `on_flow_close` path introduces no new behavior beyond the existing `HashMap::remove`.

**Red-Gate tests (VP-039 Sub-D):**
- `test_vp039_truncated_carry_no_error` — partial 4-byte header only → `on_flow_close`; parse_errors post-close == pre-close snapshot; findings_count unchanged
- `test_BC_2_07_040_empty_carry_flow_close` — empty carry at flow close; no observable effect beyond flow removal; `active_flows_len_for_testing() == 0 (single-flow test)`

(traces to BC-2.07.040 v1.3 Postconditions 1–5, Invariants 1–4)

### AC-144-005: Single-record ClientHello regression — existing test suite passes unmodified
**Traces to:** BC-2.07.001 v1.9 Invariant 5; BC-2.07.038 v2.7 EC-007; ADR-011 §Regression Risk

All tests in `tests/tls_analyzer_tests.rs` (9391 lines; single-record ClientHello, SNI classification, JA3/JA3S, weak ciphers, deprecated protocols, parse_errors accounting, flow lifecycle, timestamp threading) MUST pass without modification. All tests in `tests/tls_integration_tests.rs` (267 lines; tls.pcap, tls12-aes256gcm.pcap, tls13-rfc8446.pcap) MUST pass.

The carry-buffer path for a complete single-record ClientHello appends to the carry (empty), immediately drains a complete message, and leaves the carry empty — behavior identical to the pre-fix path.

(traces to BC-2.07.001 v1.9 Invariant 5 "single-record fast path preserved"; BC-2.07.038 v2.7 EC-007)

**Proptest regression (VP-039 Sub-A):** `proptest_vp039_carry_reassembly_two_record` at split_offset approaching n-1 converges to the single-record case.

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `TlsFlowState` struct | `src/analyzer/tls.rs` | Effectful (per-flow mutable state) |
| `TlsAnalyzer` struct | `src/analyzer/tls.rs` | Effectful (aggregate counters) |
| `try_parse_records` 0x16 path | `src/analyzer/tls.rs` | Effectful (carry mutation) |
| `on_flow_close` carry drop | `src/analyzer/tls.rs` | Effectful (HashMap remove) |
| Test seams (`client_hs_carry_len_for_testing`, etc.) | `src/analyzer/tls.rs` | Pure (read-only observers) |
| `proptest_vp039_carry_reassembly_two_record` | `tests/tls_analyzer_tests.rs` | Pure |
| Deterministic Red-Gate unit tests | `tests/tls_analyzer_tests.rs` | Pure |

Architecture compliance: SS-07 (analyzer/tls.rs only — F1 §2 confirmed no other files change). No changes to `src/reassembly/`, `src/analyzer/http.rs`, `src/analyzer/modbus.rs`, `src/findings.rs`, `src/dispatcher.rs`, or `src/reporter/`.

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-A1 | BC-2.07.038 EC-002 | First record is exactly 1 byte (partial type byte) | Carry accumulates; drain loop breaks `carry_buf.len() < 4`; no dispatch |
| EC-A2 | BC-2.07.038 EC-003 | Handshake header spans 2 records (1+3 or 2+2 byte split) | Carry accumulates across both records; dispatch after header complete + body complete |
| EC-A3 | BC-2.07.038 EC-004 | Non-ClientHello type (0x0B Certificate) before ClientHello | Certificate consumed silently (`parse_errors` unchanged); subsequent ClientHello dispatched |
| EC-A4 | BC-2.07.038 EC-005/EC-010 | body_len > MAX_BUF (spoof) immediately followed by valid bytes | Clear+increment fires on spoof header; remaining carry cleared; no valid data lost if valid msg preceded spoof (EC-010 coalesced-spoof) |
| EC-A5 | BC-2.07.039 EC-002 | 3 prior records accumulate to 49,200 bytes; 4th record pushes to 65,600 > MAX_BUF | Buffer-fill overflow (Decision 5); carry cleared; overflow_count+1; 4th record content lost |
| EC-A6 | BC-2.07.039 EC-005 | Exact-fit carry (65,536 bytes) | No overflow (condition is `>`, not `>=`); carry valid; next drain may dispatch |
| EC-A7 | BC-2.07.040 EC-003 | `on_flow_close` with complete 4-byte header but body not yet arrived | Zero findings, parse_errors=0; flow removed via HashMap |
| EC-A8 | BC-2.07.038 EC-008 | BC-2.07.004 oversize guard fires mid-reassembly | BC-2.07.004 guard clears `client_buf` + increments parse_errors; `client_hs_carry` NOT touched; orphaned carry persists bounded by MAX_BUF; discarded at flow close per BC-2.07.040 |
| EC-A9 | BC-2.07.042 EC-005 | ClientHello + ServerHello in same direction (structurally impossible) | ServerHello is server→client direction; ClientHello is client→server; they cannot coalesce in one direction's carry |

## Estimated Complexity

**Story points: 8** (carry buffer struct + drain loop + tests; algorithm is additive to existing `try_parse_records`; VP-039 Sub-A/B/C/D harnesses required; single-record regression must remain green; ServerHello symmetry deferred to STORY-145)

## Token Budget Estimate

| Context source | Estimated tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| BC files (5 BCs: 038/039/040/042/001) | ~8,000 |
| ADR-011 | ~3,000 |
| `src/analyzer/tls.rs` (9 KLoC) | ~22,000 |
| `tests/tls_analyzer_tests.rs` (9391 lines) | ~24,000 |
| VP-039 (harness skeletons) | ~5,000 |
| Tool outputs (cargo test) | ~2,000 |
| **Total estimate** | **~67,000** |

This story fits within a 200k-token context window (~34%). If the implementer agent's context is 100k, split at Task 4 (VP-039 Sub-C/D tests can be written separately from the struct + drain loop changes).

## Tasks

1. **Write Red-Gate tests first (TDD Step 1 — all must FAIL before implementation)**
   - `test_BC_2_07_038_canonical_frame_rfc8446_s4` (three frames from RFC 8446 §4 header bytes)
   - `test_BC_2_07_038_malformed_assembled_body`
   - `proptest_vp039_carry_reassembly_two_record`
   - `proptest_vp039_exact_consume_coalesced`
   - `proptest_vp039_carry_bounded_invariant` (Sub-F: invariant that `carry.len() <= MAX_BUF` after any `on_data` call sequence; generative regression guard assigned to STORY-144)
   - `test_vp039_sni_boundary_deterministic` (SNI bytes split across record boundary; asserts SNI extracted correctly, parse_errors==0)
   - `test_vp039_carry_overflow_clear_and_recover`
   - `test_vp039_carry_overflow_recovery`
   - `test_vp039_body_len_spoof`
   - `test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key`
   - `test_vp039_truncated_carry_no_error`
   - `test_BC_2_07_040_empty_carry_flow_close`
   - `test_BC_2_07_042_exact_consume_no_double_dispatch`
   - `test_vp039_n_record_reassembly`
   - `test_vp039_large_valid_hello_reassembly`
   - Confirm: `cargo test --all-targets` shows exactly these 15 tests FAILING, all others GREEN

2. **Implement `TlsFlowState` struct changes (AC-144-001)**
   - Add `client_hs_carry: Vec<u8>` and `server_hs_carry: Vec<u8>` to `TlsFlowState`
   - Initialize both to `Vec::new()` in `TlsFlowState::new()`
   - Add `handshake_reassembly_overflows: u64` (init to 0) to `TlsAnalyzer`
   - Add test seams: `client_hs_carry_len_for_testing`, `server_hs_carry_len_for_testing`, `handshake_reassembly_overflow_count`
   - Verify: `cargo check` passes; existing tests still GREEN; Red-Gate tests now compile but fail

3. **Implement carry drain loop in `try_parse_records` (AC-144-002, AC-144-003)**
   - Replace the 0x16 single-record dispatch with: overflow-check-before-append + extend + drain loop
   - Implement 3-byte big-endian body_len decode: `((carry_buf[1] as u32) << 16) | ((carry_buf[2] as u32) << 8) | (carry_buf[3] as u32)`
   - Implement `parse_tls_message_handshake` call for 0x01 msg_type (import from `tls_parser`)
   - Insert `"handshake_reassembly_overflows"` in `summarize()` detail map (mirroring `truncated_records`)
   - Verify: Sub-A/B/C/D/F Red-Gate tests turn GREEN one by one
   - Note: `buffer_saturation_drops` counter wiring is STORY-146 scope — do NOT add it in this story

4. **Verify `on_flow_close` carry discard (AC-144-004)**
   - Confirm `self.flows.remove(flow_key)` already drops carry fields (no explicit clearing needed)
   - Verify: `test_vp039_truncated_carry_no_error` and `test_BC_2_07_040_empty_carry_flow_close` GREEN

5. **Full regression sweep (AC-144-005)**
   - `cargo test --all-targets` — ALL 9391-line test suite GREEN
   - `cargo clippy --all-targets -- -D warnings` — zero warnings
   - `cargo fmt --check` — format clean
   - Verify tls_integration_tests.rs passes (tls.pcap, tls12-aes256gcm.pcap, tls13-rfc8446.pcap)

6. **Micro-commit each step** per TDD Iron Law; open PR targeting `develop`

## Previous Story Intelligence

N/A — this is the first story in E-5 for the fix-tls-clienthello-frag cycle. STORY-058 (E-5 Wave 18) delivered the original TLS buffer management and summarize output; the new carry fields are additive and do not alter the existing `client_buf`/`server_buf` logic.

Lessons from STORY-139/140/141 (sibling carry-buffer stories for ENIP/DNP3/Modbus):
- **Read each BC file before writing**: BC summaries cause drift. All postconditions above were derived directly from the BC files, not paraphrased.
- **Borrow constraint for aggregate counter increment**: the `buffer_saturation_drops` pattern (set local `did_drop: bool` inside `&mut state` block; increment `self.counter` AFTER block) applies equally to `handshake_reassembly_overflows` increment placement — but the carry overflow guard fires BEFORE the append in the outer record loop, so the placement is different: `self.handshake_reassembly_overflows += 1` fires inside the overflow-check branch before the append is attempted (no borrow conflict since carry is cleared before calling `handle_client_hello`).
- **Decision-5 vs Decision-4 fixture distinction**: the overflow unit tests must use a valid header body_len=65,500 to trigger the buffer-fill path (Decision 5), NOT a 0xCC-filled payload which triggers the body_len-spoof path (Decision 4) on the first record. See VP-039 Sub-C fixture prose.

## Architecture Compliance Rules

Source: `architecture/module-decomposition.md` + ADR-011

1. **Single-file change:** ONLY `src/analyzer/tls.rs` changes in Rust source. No other Rust files are modified.
2. **No abandoned-flag:** `TlsFlowState` must not gain any `hs_carry_abandoned: bool` or equivalent sticky-discard field. The absence of this field is an invariant (BC-2.07.039 Invariant 4; ADR-011 Decision 5).
3. **Carry counter lives on `TlsAnalyzer`, not `TlsFlowState`:** `handshake_reassembly_overflows` is an aggregate counter — same pattern as `truncated_records: u64` at tls.rs:339.
4. **Carry cap uses same constant as TCP stream buffer:** `MAX_BUF = 65_536` (tls.rs:33) — same constant, NOT a new constant.
5. **`parse_tls_message_handshake` (not `parse_tls_plaintext`) for assembled bytes:** the carry holds raw handshake message bytes with NO 5-byte TLS record header prefix; `parse_tls_plaintext` requires a record header and must not be used here (ADR-011 Decision 4; BC-2.07.038 Postcondition 9).
6. **Overflow check fires BEFORE append (not after):** `if carry_buf.len() + record_payload.len() > MAX_BUF` fires in the outer record loop BEFORE `carry_buf.extend_from_slice(...)` (BC-2.07.039 Invariant 3).
7. **Test namespace isolation (DF-TEST-NAMESPACE-001):** `tests/tls_analyzer_tests.rs` uses a FLAT module namespace. ALL 15 new test functions added by STORY-144 MUST be placed inside a dedicated `mod story_144 { ... }` wrapper at the end of the file. No new test function from this story may be added at the flat module root. The standalone-refactor option (extracting all existing flat-root tests into their own mods) is acceptable but NOT required for STORY-144; at minimum, wrap STORY-144's own tests. STORY-145 and STORY-146 will follow the same pattern with their own `mod story_145` and `mod story_146` wrappers.

## Library & Framework Requirements

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `tls-parser` | 0.12.2 (pinned in Cargo.toml) | `parse_tls_message_handshake` for assembled carry bytes |
| `proptest` | 1.x (existing dev-dependency) | `proptest_vp039_carry_reassembly_two_record`, `proptest_vp039_exact_consume_coalesced`, `proptest_vp039_carry_bounded_invariant` |

No new dependencies. `tls-parser` 0.12.2 is already used — `parse_tls_message_handshake` is already in scope but a new import path may be needed inside the carry drain loop. Confirm by checking existing import block in `tls.rs`.

**Forbidden dependencies:** `src/analyzer/tls.rs` MUST NOT depend on `src/reassembly/` for the carry buffer implementation. Carry buffers are local `Vec<u8>` fields on `TlsFlowState` — no shared reassembly infrastructure.

## File Structure Requirements

| File | Change Type | Purpose |
|------|------------|---------|
| `src/analyzer/tls.rs` | Modify | Add carry fields to `TlsFlowState`; add aggregate counter to `TlsAnalyzer`; rewrite 0x16 path in `try_parse_records`; add test seams; update `summarize()` |
| `tests/tls_analyzer_tests.rs` | Modify | Add VP-039 Sub-A/B/C/D/F test functions (15 new unit/proptest harnesses), all inside `mod story_144 { ... }` per DF-TEST-NAMESPACE-001 |

No new files. No changes to `src/reassembly/`, `src/dispatcher.rs`, `src/findings.rs`, `src/reporter/`, `tests/tls_integration_tests.rs`.

### Test Helper / Seam Ownership (STORY-144 creates these)

The VP-039 harnesses require the following helpers and seams. The implementer MUST create or reconcile them in this story. This table is the **complete contract** — every harness symbol is accounted for.

**tls.rs seams (new `#[doc(hidden)] pub fn` on `TlsAnalyzer`, all created in AC-144-001):**

| Helper / Seam | Form | Notes |
|---------------|------|-------|
| `TlsAnalyzer::client_hello_seen_for_testing(&self, flow_key: &FlowKey) -> bool` | NEW test seam in `src/analyzer/tls.rs` | NEW — symmetric to the EXISTING `server_hello_seen_for_testing` at tls.rs:991. Created in STORY-144 (AC-144-001). The server direction uses the existing `server_hello_seen_for_testing`; this story adds the matching client-direction accessor. Follows `#[doc(hidden)] pub fn` convention. |
| `TlsAnalyzer::client_hs_carry_len_for_testing(&self, flow_key: &FlowKey) -> usize` | NEW test seam in `src/analyzer/tls.rs` | Created in AC-144-001; follows `#[doc(hidden)] pub fn` convention at tls.rs:957+ |
| `TlsAnalyzer::server_hs_carry_len_for_testing(&self, flow_key: &FlowKey) -> usize` | NEW test seam in `src/analyzer/tls.rs` | Created in AC-144-001; same convention |
| `TlsAnalyzer::handshake_reassembly_overflow_count(&self) -> u64` | NEW test seam / accessor in `src/analyzer/tls.rs` | Created in AC-144-001; read-only observer of the aggregate counter on `TlsAnalyzer` |
| `TlsAnalyzer::server_hello_seen_for_testing` | EXISTING seam at tls.rs:991 | Used by Sub-E in STORY-145 for the server direction; do NOT redefine. |

**Local test helpers (defined inside `mod story_144 { ... }` in `tests/tls_analyzer_tests.rs`):**

| Helper / Seam | Form | Notes |
|---------------|------|-------|
| `make_test_flow_key(seed: u8) -> FlowKey` | LOCAL helper in `mod story_144` | NEW — varies `src_addr`/`dst_addr`/`src_port`/`dst_port` by `seed` so that cross-flow and independent-flow tests can create distinct `FlowKey`s. The zero-arg `test_flow_key()` that may exist elsewhere is insufficient for multi-flow tests. Defined in `mod story_144`. STORY-145 may re-declare or import from here (see STORY-145 helper note). |
| `build_client_hello_with_sni(sni: &str) -> Vec<u8>` | LOCAL wrapper in `mod story_144` | The real `build_client_hello(sni, &[0x002f])` returns a COMPLETE TLS record (5-byte record header + handshake body). This LOCAL wrapper MUST strip the 5-byte record header: `build_client_hello(sni, &[0x002f])[5..].to_vec()`. Returns RAW handshake-message bytes with NO record header — so fragmentation tests can re-frame them via `wrap_as_tls_record` per fragment. Before creating, `grep` for `build_client_hello_with_sni_list` or `build_client_hello` in the existing suite and use the real name if present. |
| `build_server_hello() -> Vec<u8>` | LOCAL wrapper in `mod story_144` | The real `build_server_hello(0x002f)` returns a COMPLETE TLS record (5-byte record header + handshake body). This LOCAL wrapper MUST strip the 5-byte record header: `build_server_hello(0x002f)[5..].to_vec()`. Returns RAW handshake-message bytes with NO record header. Used by STORY-144 drain-loop tests that need a server-direction message for the neutral/non-dispatch path. Before creating, `grep` for `build_server_hello` in the existing suite. NOTE: tests that deliver a COMPLETE single-record hello (e.g. cross-flow control flow) MUST deliver the full-record output of the real `build_client_hello` / `build_server_hello` directly via `on_data` — do NOT double-wrap (the full record already has the 5-byte header). |
| `wrap_as_tls_record(content_type: u8, payload: &[u8]) -> Vec<u8>` | LOCAL helper (or reconcile existing) | Wraps bytes in a 5-byte TLS record header `[content_type, 0x03, 0x03, len_hi, len_lo]`. Before creating, `grep` for `make_tls_record` or `wrap_as_tls_record` in the existing suite — use the real name if found. |
| `all_findings_len_for_testing` | EXISTING seam | The existing seam for findings count is `all_findings_len_for_testing` (NOT `findings_count_for_testing`). Use the real name — do NOT call it `findings_count_for_testing`. |

**Reconciliation rule:** Before creating any new helper, `grep` the existing `tests/tls_analyzer_tests.rs` for the relevant name pattern. Use the real existing name if found.

## Red-Gate Test Set (VP-039 Sub-A/B/C/D/F scope)

All 15 test names below are CANONICAL per VP-039 and BC-2.07.038/039/040/042 VP tables. These names must appear verbatim in the test suite (DF-AC-TEST-NAME-SYNC-001).

| Test name | Sub | BC | Type | Fails before? |
|-----------|-----|----|------|--------------|
| `proptest_vp039_carry_reassembly_two_record` | Sub-A | BC-2.07.038 | proptest | Yes |
| `test_BC_2_07_038_canonical_frame_rfc8446_s4` | Sub-A | BC-2.07.038 AC-CANONICAL-FRAME | unit | Yes |
| `test_BC_2_07_038_malformed_assembled_body` | Sub-A | BC-2.07.038 PC-9 | unit | Yes |
| `test_vp039_sni_boundary_deterministic` | Sub-A | BC-2.07.038 EC-001 | unit | Yes |
| `test_vp039_n_record_reassembly` | Sub-A-ext-N | BC-2.07.038 EC-003 | unit | Yes |
| `test_vp039_large_valid_hello_reassembly` | Sub-C-ext-large | BC-2.07.038 Inv-5 | unit | Yes |
| `proptest_vp039_exact_consume_coalesced` | Sub-B | BC-2.07.042 | proptest | Yes |
| `test_BC_2_07_042_exact_consume_no_double_dispatch` | Sub-B | BC-2.07.042 | unit | Yes |
| `test_vp039_carry_overflow_clear_and_recover` | Sub-C | BC-2.07.039 PC-1–6 | unit | Yes |
| `test_vp039_carry_overflow_recovery` | Sub-C | BC-2.07.039 PC-6 | unit | Yes |
| `test_vp039_body_len_spoof` | Sub-C | BC-2.07.038 Inv-5 | unit | Yes |
| `test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key` | Sub-C | BC-2.07.039 PC-7 | unit | Yes |
| `test_vp039_truncated_carry_no_error` | Sub-D | BC-2.07.040 | unit | Yes |
| `test_BC_2_07_040_empty_carry_flow_close` | Sub-D | BC-2.07.040 | unit | Yes |
| `proptest_vp039_carry_bounded_invariant` | Sub-F | BC-2.07.039 Invariant 1 | proptest | Yes |

Total: **15 new harnesses (3 proptest + 12 unit) in STORY-144 scope** — Sub-F (`proptest_vp039_carry_bounded_invariant`) is assigned to STORY-144 ONLY; it is NOT shared with STORY-145.

## Holdout Scenarios (F4)

See `.factory/cycles/fix-tls-clienthello-frag/holdout-scenarios.md` for the full F4 scenario registry (authoritative HS-F4-NNN definitions). STORY-144 specifically gates on:
- **HS-F4-001** (DF-CANONICAL-FRAME-HOLDOUT-001): canonical RFC-byte holdout — `test_BC_2_07_038_canonical_frame_rfc8446_s4` must pass
- **HS-F4-002**: two-record ClientHello split at SNI boundary → SNI extracted, JA3 computed, parse_errors==0
- **HS-F4-003**: N-record fragmented ClientHello (1-byte first record) → SNI extracted, parse_errors==0
- **HS-F4-004**: fragment + coalesce (ClientHello + Certificate in one record, preceded by a fragmented-first-record) → client_hello_seen==true, parse_errors==0
- **HS-F4-005**: snaplen-truncated flow close (second record absent) → client_hello_seen==false, parse_errors==0
- **HS-F4-006**: single-record regression (common case unchanged) → client_hello_seen==true, SNI/JA3 populated
