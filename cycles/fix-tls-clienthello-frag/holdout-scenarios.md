---
document_type: holdout-scenario-registry
cycle_id: fix-tls-clienthello-frag
phase: F4
version: "1.0"
status: draft
produced_by: story-writer
date: 2026-06-29
canonical_id_scheme: HS-F4-NNN
total_scenarios: 12
---

# F4 Holdout Scenario Registry — fix-tls-clienthello-frag

This file is the **single authoritative source** for all F4 holdout scenarios for the
`fix-tls-clienthello-frag` cycle (TLS-CLIENTHELLO-FRAG-001 carry-buffer reassembly).

The holdout evaluator consumes this registry. Stories cross-reference this file by
ID. Do NOT add new scenarios to story files inline — register them here and reference
the registry path.

## HS-NEW → HS-F4 ID Mapping

The F1 delta-analysis §7 introduced five scenarios under the `HS-NEW-*` scheme.
Those IDs are superseded by the `HS-F4-NNN` scheme used in the F3 story decomposition.
The table below is the canonical mapping; `HS-NEW-*` IDs must not appear in any
new artifact.

| HS-NEW ID | HS-F4 ID | Notes |
|-----------|----------|-------|
| HS-NEW-A  | HS-F4-002 | Two-record split at SNI boundary — scope narrowed from "SNI boundary" to "exact SNI extension boundary" |
| HS-NEW-B  | HS-F4-003 | N-record fragmented ClientHello — first record is 1 byte |
| HS-NEW-C  | HS-F4-004 | Fragment + coalesce — ClientHello preceding a Certificate |
| HS-NEW-D  | HS-F4-005 | Snaplen-truncated flow close — second record absent |
| HS-NEW-E  | HS-F4-006 | Single-record regression holdout |

Additional scenarios HS-F4-001 and HS-F4-007 through HS-F4-012 were introduced
during F3 story decomposition and have no corresponding HS-NEW-* equivalent.

---

## Scenario Definitions

### HS-F4-001 (DF-CANONICAL-FRAME-HOLDOUT-001)

**Description:** Canonical RFC byte sequence holdout — three hand-crafted frames
decoded from verbatim RFC 8446 §4 byte sequences, independently of the project's
`build_client_hello` / `build_server_hello` helpers.

- **Frame A** (`[0x01, 0x00, 0x00, 0x05]` + 5 zero bytes): degenerate body_len=5,
  ClientHello (0x01), internally malformed body. Expected: `parse_errors` incremented
  by 1 (PC-9 malformed-body path), `client_hello_seen=false`.
- **Frame B** (`[0x01, 0x01, 0x05, 0x00]` header only, no body bytes): big-endian
  decode → `body_len=66,816 > MAX_BUF`. Expected: body_len-spoof guard fires;
  `handshake_reassembly_overflows` incremented by 1; carry cleared; `parse_errors=0`.
- **Frame C** (`[0x01, 0x00, 0x01, 0x00]` + 256 bytes of `0xcc`): `body_len=256`,
  length-complete but internally malformed: session-id length byte = `0xcc` = 204 > 32,
  rejected by `tls_parser` 0.12.2 `verify(be_u8, |&n| n <= 32)` (`tls_handshake.rs`
  session-id length guard) → genuine `Err` → `parse_errors` incremented by 1; carry
  emptied (260 bytes consumed); `client_hello_seen=false`. (An all-zero body parses as
  a structurally valid degenerate ClientHello — `Ok`, `parse_errors=0` — and is NOT
  the malformed vector; see BC-2.07.038 v2.8 PC-9 NOTE.)

**BCs exercised:** BC-2.07.038 v2.8 AC-CANONICAL-FRAME; BC-2.07.038 Inv-5 (Frame B);
BC-2.07.038 PC-9 (Frames A and C).

**Expected outcome:** PASS (all three frame assertions hold).

**Covering story:** STORY-144

**Red-Gate test:** `test_BC_2_07_038_canonical_frame_rfc8446_s4`

**Policy:** DF-CANONICAL-FRAME-HOLDOUT-001 — this is the mandatory canonical-frame
holdout; its presence is a policy requirement, not merely a nice-to-have.

---

### HS-F4-002

**Description:** Two-record fragmented ClientHello split at the exact SNI extension
boundary. The first TLS 0x16 record contains the ClientHello header through the
extension type field (SNI type present) but the hostname bytes are absent; the second
record contains the hostname bytes and completes the ClientHello body.

**BCs exercised:** BC-2.07.038 v2.7 EC-001 (SNI boundary split);
BC-2.07.038 Postconditions 1–4.

**Expected outcome:** PASS — `sni_counts` contains the correct hostname,
`ja3_counts` has an entry, `parse_errors == 0`, `client_hello_seen == true`.

**Covering story:** STORY-144

**Red-Gate test:** `test_vp039_sni_boundary_deterministic`

*(Corresponds to HS-NEW-A in delta-analysis §7.)*

---

### HS-F4-003

**Description:** N-record fragmented ClientHello where the first record payload is
exactly 1 byte of the ClientHello handshake body. Remaining bytes delivered in one or
more subsequent records. Mirrors the Kubernetes ingress-nginx reported fragmentation
pattern.

**BCs exercised:** BC-2.07.038 v2.7 EC-002 (1-byte first record); EC-003 (header
spans two records); PC-1, PC-2, PC-6.

**Expected outcome:** PASS — `client_hello_seen == true`, `sni_counts.len() == 1`,
`ja3_counts.len() == 1`, `parse_errors == 0`.

**Covering story:** STORY-144

**Red-Gate test:** `test_vp039_n_record_reassembly`

*(Corresponds to HS-NEW-B in delta-analysis §7.)*

---

### HS-F4-004

**Description:** Fragment + coalesce — the ClientHello was itself fragmented across a
preceding record, and the final record completing the ClientHello also coalesces a
Certificate (or other non-hello handshake type) message immediately following it.

**BCs exercised:** BC-2.07.038 v2.7 PC-5 (coalesce loop repeats); BC-2.07.042 v1.4
(coalesced dispatch, no double-dispatch); BC-2.07.042 EC-002.

**Expected outcome:** PASS — `client_hello_seen == true`, `parse_errors == 0`,
`handshakes_seen` count matches exactly one ClientHello (no double-dispatch).

**Covering story:** STORY-144

**Red-Gate test:** `proptest_vp039_exact_consume_coalesced`

*(Corresponds to HS-NEW-C in delta-analysis §7.)*

---

### HS-F4-005

**Description:** Snaplen-truncated ClientHello flow close — a ClientHello is fragmented
across two records, but the second record is absent (the capture ends mid-handshake,
simulating an EPB with `original_len > captured_len`). `on_flow_close` is called
without the second record ever arriving.

**BCs exercised:** BC-2.07.040 v1.3 Postconditions 1–5; Invariants 1–4; EC-003
(header complete but body not arrived).

**Expected outcome:** PASS — `client_hello_seen == false`, `sni_counts.is_empty()`,
`ja3_counts.is_empty()`, `parse_errors == 0`.

**Covering story:** STORY-144

**Red-Gate test:** `test_vp039_truncated_carry_no_error`

*(Corresponds to HS-NEW-D in delta-analysis §7.)*

---

### HS-F4-006

**Description:** Single-record ClientHello regression — a standard single-record
TLS 1.2 / 1.3 ClientHello (the common case today). After the fix, behavior must be
identical to the pre-fix path: the carry buffer is populated then immediately consumed
in one pass, leaving the carry empty.

**BCs exercised:** BC-2.07.001 v1.9 Invariant 5 (single-record fast path preserved);
BC-2.07.038 v2.7 EC-007.

**Expected outcome:** PASS — `client_hello_seen == true`, SNI and JA3 populated,
`parse_errors == 0`; all 9391-line `tls_analyzer_tests.rs` suite passes unmodified.

**Covering story:** STORY-144

**Red-Gate test:** `proptest_vp039_carry_reassembly_two_record` (split_offset
approaching n-1 converges to the single-record case); full suite via AC-144-005.

*(Corresponds to HS-NEW-E in delta-analysis §7.)*

---

### HS-F4-007

**Description:** Interleaved fragmented ClientHello + ServerHello for the same flow.
Both hellos are fragmented across two records each; records are delivered interleaved
(C2S-frag-1, S2C-frag-1, C2S-frag-2, S2C-frag-2). After all four records are
delivered, `done()` must fire.

**BCs exercised:** BC-2.07.041 v1.2 Invariant 2 (direction isolation under
interleaving); BC-2.07.002 v1.6 Postcondition 7 (ServerHello assembled via
`server_hs_carry`); BC-2.07.038 v2.7 Invariant 6 (`done()` fires after both
reassembled hellos).

**Expected outcome:** PASS — `client_hello_seen == true`, `server_hello_seen == true`,
`done() == true`, `ja3_counts.len() == 1`, `ja3s_counts.len() == 1`, `parse_errors == 0`.

**Covering story:** STORY-145

**Red-Gate test:** `proptest_vp039_direction_isolation`

---

### HS-F4-008

**Description:** Two concurrent flows — Flow A delivers a complete single-record
ClientHello (SNI=a.example); Flow B delivers a fragmented two-record ClientHello
(SNI=b.example). Records are interleaved between flows. No cross-flow contamination
is permitted.

**BCs exercised:** BC-2.07.041 v1.2 Invariant 1 (cross-flow isolation);
Postconditions 1, 4–5; EC-001.

**Expected outcome:** PASS — `sni_counts["a.example"] == 1`,
`sni_counts["b.example"] == 1`, exactly 2 entries in `sni_counts`, no cross-flow
bleed.

**Covering story:** STORY-145

**Red-Gate test:** `test_BC_2_07_041_cross_flow_isolation`

---

### HS-F4-009

**Description:** Fragmented ServerHello regression — a single-record ServerHello
(the common case today) must behave identically after the STORY-145 `server_hs_carry`
drain path is wired. The carry is populated then immediately consumed in one pass,
leaving the carry empty.

**BCs exercised:** BC-2.07.002 v1.6 Invariant 4 (single-record fast path preserved);
BC-2.07.038 v2.7 EC-007 (symmetric server direction).

**Expected outcome:** PASS — `server_hello_seen == true`, `ja3s_counts.len() == 1`,
`parse_errors == 0`.

**Covering story:** STORY-145

**Red-Gate test:** `proptest_vp039_direction_isolation` (server direction arms);
full regression via AC-145-005.

---

### HS-F4-010

**Description:** Buffer-saturation holdout — use `fill_buf_for_testing` to park the
per-direction buffer at exactly `MAX_BUF` bytes, then deliver a 1,000-byte
`on_data` payload. The tail-drop fires; `buffer_saturation_drops` must equal 1;
`summarize()` must expose the counter with value-equality.

**BCs exercised:** BC-2.07.043 v1.3 Postconditions 1, 4; EC-002 (full-drop,
remaining == 0); BC-2.07.005 v1.7 Invariant 3 (tail-drop now counted).

**Expected outcome:** PASS — `buffer_saturation_drops == 1`, `parse_errors == 0`,
`summarize()` detail `"buffer_saturation_drops" == 1`.

**Covering story:** STORY-146

**Red-Gate test:** `test_BC_2_07_043_buffer_saturation_full_drop`

---

### HS-F4-011

**Description:** Zero-drop holdout — a fresh flow receives data well within `MAX_BUF`.
No tail-drop fires. `summarize()` must still expose the key
`"buffer_saturation_drops"` with value `0` (the key is always present).

**BCs exercised:** BC-2.07.043 v1.3 Invariant 5 (no drop when data fits);
EC-008 (key always present at zero).

**Expected outcome:** PASS — `buffer_saturation_drops == 0` throughout;
`summarize()` detail `"buffer_saturation_drops" == 0`.

**Covering story:** STORY-146

**Red-Gate test:** `test_BC_2_07_043_no_drop_no_counter`

---

### HS-F4-012

**Description:** Cross-direction aggregate holdout — one `ClientToServer` buffer
saturation drop event followed by one `ServerToClient` buffer saturation drop event
(on possibly separate flows). Both events must increment the same aggregate counter.

**BCs exercised:** BC-2.07.043 v1.3 Postcondition 3 (both directions, same counter);
EC-007.

**Expected outcome:** PASS — after two separate drop events,
`buffer_saturation_drops == initial + 2`.

**Covering story:** STORY-146

**Red-Gate test:** `test_BC_2_07_043_both_directions_increment_same_counter`

---

## Summary Table

| ID | Description (short) | BCs | Expected Outcome | Story | Red-Gate Test |
|----|----------------------|-----|-----------------|-------|---------------|
| HS-F4-001 | Canonical RFC 8446 §4 frame holdout (3 frames) | BC-2.07.038 AC-CANONICAL-FRAME, Inv-5, PC-9 | PASS | STORY-144 | `test_BC_2_07_038_canonical_frame_rfc8446_s4` |
| HS-F4-002 | Two-record split at SNI extension boundary | BC-2.07.038 EC-001, PC-1–4 | PASS | STORY-144 | `test_vp039_sni_boundary_deterministic` |
| HS-F4-003 | N-record (1-byte first record) fragmented ClientHello | BC-2.07.038 EC-002/003, PC-1/2/6 | PASS | STORY-144 | `test_vp039_n_record_reassembly` |
| HS-F4-004 | Fragment + coalesce (ClientHello + Certificate) | BC-2.07.038 PC-5; BC-2.07.042 | PASS | STORY-144 | `proptest_vp039_exact_consume_coalesced` |
| HS-F4-005 | Snaplen-truncated flow close (no finding, no error) | BC-2.07.040 PC-1–5, Inv-1–4 | PASS | STORY-144 | `test_vp039_truncated_carry_no_error` |
| HS-F4-006 | Single-record regression (common case unchanged) | BC-2.07.001 Inv-5; BC-2.07.038 EC-007 | PASS | STORY-144 | `proptest_vp039_carry_reassembly_two_record` |
| HS-F4-007 | Interleaved fragmented ClientHello + ServerHello; done() fires | BC-2.07.041 Inv-2; BC-2.07.002 PC-7; BC-2.07.038 Inv-6 | PASS | STORY-145 | `proptest_vp039_direction_isolation` |
| HS-F4-008 | Two concurrent flows; cross-flow isolation | BC-2.07.041 Inv-1, PC-1/4–5 | PASS | STORY-145 | `test_BC_2_07_041_cross_flow_isolation` |
| HS-F4-009 | Single-record ServerHello regression | BC-2.07.002 Inv-4; BC-2.07.038 EC-007 | PASS | STORY-145 | `proptest_vp039_direction_isolation` |
| HS-F4-010 | Buffer saturation full-drop + summarize() value-equality | BC-2.07.043 PC-1/4; EC-002 | PASS | STORY-146 | `test_BC_2_07_043_buffer_saturation_full_drop` |
| HS-F4-011 | Zero-drop: key always present in summarize() | BC-2.07.043 Inv-5; EC-008 | PASS | STORY-146 | `test_BC_2_07_043_no_drop_no_counter` |
| HS-F4-012 | Cross-direction aggregate counter (C2S + S2C = +2) | BC-2.07.043 PC-3; EC-007 | PASS | STORY-146 | `test_BC_2_07_043_both_directions_increment_same_counter` |
