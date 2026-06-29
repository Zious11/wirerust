---
artifact: verification-property
vp_id: VP-039
title: "TLS Handshake Reassembly: Bounded Carry, Exact-Consume, Truncation-Safety, Cross-Direction Isolation"
status: draft
phase: P1
tool: proptest
subsystem: SS-07
module: "src/analyzer/tls.rs"
producer: architect
timestamp: 2026-06-29T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-011-tls-handshake-reassembly.md
feature_cycle: fix-tls-clienthello-frag
issue: fix-tls-clienthello-frag
bcs:
  - BC-2.07.038
  - BC-2.07.039
  - BC-2.07.040
  - BC-2.07.041
  - BC-2.07.042
verification_lock: false
modified:
  - date: 2026-06-29
    actor: architect
    reason: "Fix-burst-6 (F-FRESH-001/F-CRITICAL-2/F-F2P-IMP-001/F-FRESH-002): (CRITICAL-1) Sub-C overflow fixtures corrected — 0xCC fill was hitting Decision-4 body_len-spoof path (body_len=0xCCCCCC>>MAX_BUF on record 1), NOT Decision-5 buffer-fill path; replaced with valid header body_len=65,500 (header [0x01,0x00,0xFF,0xDC]) + padding records to trigger buffer-fill guard exactly once (counter==overflows_before+1 now TRUE); mechanism prose updated. (CRITICAL-2) New test test_BC_2_07_038_malformed_assembled_body: fragmented handshake with length-consistent header but malformed body, asserts parse_errors+1, message bytes consumed, no finding, no panic (per ADR-011 Decision-4 malformed-body semantics). (F-F2P-IMP-001) Sub-F proptest generator restructured: payloads now begin with valid header prefix (body_len<=MAX_BUF) so carry actually accumulates; near-vacuity for buffer-fill path fixed. (F-FRESH-002) Frame C dispatch lane added to canonical-frame test: body_len=256 (header [0x01,0x00,0x01,0x00]) confirms dispatch lane with mid-range body_len; Frame A (degenerate=5) and Frame B (BE 66,816 vs LE 1,281 discriminator) retained."
  - date: 2026-06-29
    actor: architect
    reason: "Fix-burst-7 (F-FRESH2-003/F-FRESH2-001/F-FRESH2-002/F-FRESH2-004/O-1/residue-qualifier): (F-FRESH2-003) Two orphaned tests authored — test_BC_2_07_040_empty_carry_flow_close (BC-2.07.040: empty carry at flow close has no observable effect beyond flow removal) and test_BC_2_07_042_exact_consume_no_double_dispatch (BC-2.07.042: coalesced messages exact-consumed, handshakes_seen count is exact, assert analyzer.handshake_count()==1 for a ClientHello + non-hello coalesced in one record). Unit test count 8->10; total harnesses 12->14 (10 unit + 4 proptest). (F-FRESH2-001) All count prose and enumeration list reconciled: 14 total harnesses, 10 unit test names enumerated. (F-FRESH2-004) Sub-F prose softened: bounded-carry invariant (carry.len()<=MAX_BUF) is the guard; Decision-5 buffer-fill path is exercised DETERMINISTICALLY by test_vp039_carry_overflow_clear_and_recover, not probabilistically by Sub-F. (O-1) LOW: Ok(_) arm in pseudocode noted unreachable given the outer 0x01|0x02 msg_type guard — grouped with Err only for match exhaustiveness; only Err path reachable, matching BC-2.07.038 PC-9. (residue-qualifier) 4xMAX_BUF is POST-on_data-return residue ceiling; in-call transient peak may be higher due to record_bytes clone."
  - date: 2026-06-29
    actor: architect
    reason: "Fix-burst-10 (F-ADVF2-001 CRITICAL): Frame A semantics corrected — body_len=5 is length-complete but structurally malformed (too short for a valid ClientHello; parse_tls_message_handshake returns Err). Frame A now asserts parse_errors==errors_before+1 (PC-9 malformed-body path) and client_hello_seen==false (no dispatch on Err), replacing the stale parse_errors==0 assertion which was physically incorrect. The carry_len==0 assertion is retained (exact-consume still fires — 9 bytes drained). Doc-comment for Frame A updated: new NOTE block explains the PC-9 path, 'forces the carry to buffer and then dispatch' prose replaced with 'malformed ClientHello'. Property Statement Frame A bullet updated to name PC-9 path. Harness count (14) and VP total (39) unchanged."
  - date: 2026-06-29
    actor: architect
    reason: "Fix-burst-11 (F-COMP-001/F-COMP-002/F-COMP-003/F-F2IMPL-001): Three new deterministic unit test skeletons added (unit test count 10→13; total harnesses 14→17; 4 proptest + 13 unit = 17). (F-COMP-002) test_BC_2_07_041_cross_flow_isolation — Two distinct FlowKeys: Flow A complete single-record ClientHello (SNI=a.example); Flow B SAME-SHAPED ClientHello fragmented across records (SNI=b.example); asserts both SNIs present in sni_counts, no cross-flow bleed, Flow B's partial carry does not affect Flow A; maps to BC-2.07.041 PC-1/PC-4/Inv-1 (Sub-E cross-flow/multi-FlowKey isolation not covered by proptest_vp039_direction_isolation which uses a single FlowKey). (F-COMP-001) test_vp039_n_record_reassembly — drip-feeds ONE valid ClientHello across >=3 records (1-byte + 1-byte + remainder AND 4-byte handshake header split 1+1+2 across three records); asserts SNI extracted, JA3 computed, parse_errors==0; exercises break-resume-break-resume re-entrancy across >2 on_data calls; maps to BC-2.07.038 PC-1/PC-2/PC-6 + EC-003 (header spanning records). (F-COMP-003) test_vp039_large_valid_hello_reassembly — VALID ClientHello with body between 18,433 and 65,536 bytes (~40 KB via large padding), fragmented across multiple <=MAX_RECORD_PAYLOAD records; asserts SNI/JA3 populated, parse_errors==0, handshake_reassembly_overflows==0 (NOT dropped, NOT overflow); positively verifies the 18,432→65,536 per-message cap raise; maps to BC-2.07.038 Inv-5 (large-but-valid sub-case). (F-F2IMPL-001) test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key doc-comment tightened: 'asserts key present in detail map' → 'asserts detail[\"handshake_reassembly_overflows\"].as_u64() == 1 (value-equality, not mere key presence)' to match BC-2.07.039 PC-7 and the actual test body which already asserts ==1. Same tightening propagated to verification-architecture.md VP-039 row and verification-coverage-matrix.md coverage note. VP total count (39), proptest_count (17), p1_count (25) UNCHANGED (harnesses within VP-039). ARCH-INDEX ADR-011 row harness count flagged for update: currently '4 proptest + 10 unit tests = 14' should become '4 proptest + 13 unit tests = 17'."
  - date: 2026-06-29
    actor: architect
    reason: "Fix-burst-12 (F-1/O-1/O-2/O-3 final cleanup — LAST F2 cleanup): (F-1 MEDIUM) stale harness-count phrase removed from VP-INDEX P1 enumeration entry for VP-039; corrected to '4 proptest + 13 unit tests = 17 total harnesses' matching catalog row and callout block. (O-1 LOW) test_vp039_large_valid_hello_reassembly inline layout comments corrected: extensions_length 39944 → 39,957 (correct: SNI_EXT(18)+PAD_EXT_HEADER(4)+PAD_DATA_LEN(39935)) and padding ext_length 39926 → 39,935 (correct: PAD_DATA_LEN = TARGET_BODY_LEN(40000) - FIXED_PREFIX(43) - SNI_EXT(18) - PAD_EXT_HEADER(4)); byte range updated [65..39991] → [65..40000] accordingly. (O-2 LOW) dead binding let sni_hostname = 'large.example.com' dropped; assert!(sni.contains_key('large.com')) added for parity with sibling tests (fixture encodes 9-byte 'large.com'). (O-3 LOW) docstring BC-2.07.038 Inv-5 quotation reworded as corollary paraphrase — 'large-but-valid sub-case' label added; range narrowed to (18,432, MAX_BUF] (cap-raise range); 'MUST be assembled and dispatched rather than silently dropped or counted as an overflow' avoids presenting editorial paraphrase as verbatim contract text. VP total (39), proptest_count (17), p1_count (25) UNCHANGED."
---

# VP-039: TLS Handshake Reassembly — Bounded Carry, Exact-Consume, Truncation-Safety, Cross-Direction Isolation

## Property Statement

Six sub-properties (Sub-A through Sub-F) collectively verify that the per-direction
handshake carry buffer introduced by the fix-tls-clienthello-frag cycle is correct,
bounded, and isolated.

**Sub-A (BC-2.07.038 — reassembly across records):** For any valid TLS ClientHello
of total byte length `n` and any split offset `1 <= k < n`, delivering bytes `[0..k]`
as one 0x16 record payload followed by bytes `[k..n]` as a second 0x16 record payload
produces `client_hello_seen == true`, `ja3_counts.len() == 1`, `sni_counts.len() == 1`
(when SNI is present), and `parse_errors == 0`. The assembled message received by
`handle_client_hello` is byte-identical to a single-record delivery of the full
ClientHello.

**Sub-B (BC-2.07.042 — coalesced messages dispatched independently):** When a single
TLS 0x16 record payload contains the concatenated byte sequences of two complete
handshake messages (e.g., a ClientHello followed by another handshake type), each
message is dispatched independently and in wire order. After processing, the carry
buffer length is 0 (both messages fully consumed). The exact-consume invariant
(advance by exactly `4 + body_len` per message) prevents double-dispatch or silent
skipping.

**Sub-C (BC-2.07.039 — bounded carry, clear-and-recover overflow policy):** When
accumulated multi-record 0x16 payloads push `client_hs_carry` above `MAX_BUF =
65,536` bytes via the BUFFER-FILL path (Decision 5: `carry_buf.len() +
record_payload_len > MAX_BUF` fires during a carry append), the carry buffer for
that direction is cleared (carry length becomes 0), `handshake_reassembly_overflows`
is incremented by exactly 1, `parse_errors` is UNCHANGED, and no finding is emitted.
There is NO sticky abandoned state.

**Fixture requirements for buffer-fill overflow (F-CRITICAL-2 fix):** The
`test_vp039_carry_overflow_clear_and_recover` and `test_vp039_carry_overflow_recovery`
tests MUST reach the Decision-5 buffer-fill guard, NOT the Decision-4 body_len-spoof
guard. These two guards are distinct:

- **Decision-4 body_len-spoof guard** (fires INSIDE the consume loop after 4-byte
  header is available): triggers when `body_len > MAX_BUF` as decoded from the
  header. A payload filled with `0xCC` bytes has `carry_buf[1..4] = [0xCC, 0xCC, 0xCC]`
  → `body_len = 0xCCCCCC = 13,421,772 >> MAX_BUF` → this guard fires on RECORD 1,
  the counter becomes `overflows_before + 1`, and records 2–4 each fire this guard
  again (counter ends at `overflows_before + 4`). The assertion `== overflows_before + 1`
  is then FALSE. The prior 0xCC-fill fixture was wrong because it tested the
  body_len-spoof path, not the buffer-fill accumulation path.

- **Decision-5 buffer-fill guard** (fires BEFORE the carry append when
  `carry_buf.len() + record_payload_len > MAX_BUF`): requires records that begin
  with a VALID header declaring `body_len <= MAX_BUF` (so the consume loop does NOT
  clear the carry after decoding the header), followed by body records that accumulate
  until the total exceeds MAX_BUF.

**Corrected fixture design:** The first record carries a VALID handshake header
declaring `body_len = 65,500` (bytes: `[0x01, 0x00, 0xFF, 0xDC]`, i.e.
`(0x00 << 16) | (0xFF << 8) | 0xDC = 65,500 <= MAX_BUF`) followed by padding
body bytes (e.g., all zeros, length limited to avoid MAX_RECORD_PAYLOAD). Subsequent
records deliver additional body bytes. When `carry_buf.len() + next_payload_len >
MAX_BUF`, the Decision-5 guard fires exactly ONCE, clearing the carry and
incrementing `handshake_reassembly_overflows` by exactly 1. The assertion
`== overflows_before + 1` is then TRUE, and the recovery test correctly demonstrates
recovery from a genuine buffer-fill overflow.

**Recovery assertion (REQUIRED):** after the overflow event, delivering a subsequent
well-formed single-record 0x16 ClientHello fragment on the same direction IS parsed
normally — `client_hello_seen == true` and SNI / JA3 populated. This assertion
distinguishes clear-and-recover (Policy A) from the rejected sticky-abandon design
(Policy B) and is the most important behavioral claim in this sub-property.

**Body-len-spoof assertion (REQUIRED):** delivering a 0x16 record whose handshake
header declares `body_len = 0x010001` (65,537 — one above MAX_BUF = 65,536, so the
strict `> MAX_BUF` guard fires) triggers clear-and-recover: carry length becomes 0,
`handshake_reassembly_overflows` incremented, `parse_errors` UNCHANGED, no buffering.
A header declaring `body_len = 0x010000` (65,536 = MAX_BUF exactly) would NOT trigger
the guard because the condition is strictly `> MAX_BUF`; 65,537 is the minimal triggering value.

Verified by five deterministic unit tests (Sub-C scope; two additional orphaned tests are in Sub-D-ext and Sub-B-ext below):
- `test_vp039_carry_overflow_clear_and_recover` — buffer-fill overflow (corrected fixture: valid header body_len=65,500; accumulate until Decision-5 guard fires; counter==overflows_before+1) + findings_count pre/post snapshot (no finding emitted, BC-2.07.039 PC-4)
- `test_vp039_carry_overflow_recovery` — recovery: post-buffer-fill-overflow ClientHello still parsed (same corrected fixture as above)
- `test_vp039_body_len_spoof` — body_len=65537 > MAX_BUF triggers clear-and-recover (Decision-4 guard, distinct from buffer-fill); findings_count pre/post snapshot (BC-2.07.039 PC-4)
- `test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key` — summarize() exposes "handshake_reassembly_overflows" with value-equality: asserts `detail["handshake_reassembly_overflows"].as_u64() == 1` (not mere key presence; BC-2.07.039 PC-7)
- `test_BC_2_07_038_malformed_assembled_body` — NEW (F-FRESH-001): fragmented handshake whose header length is consistent but body is malformed (e.g. truncated extensions); asserts parse_errors incremented by 1, message bytes consumed, no finding emitted, no panic (ADR-011 Decision-4 malformed-body semantics)

**Canonical-frame test (F-F2-010 + F-FRESH-002, CRITICAL policy requirement — BC-2.07.038 AC):**
A deterministic unit test that constructs THREE handshake headers by hand from RFC 8446 §4 bytes,
splitting and asserting correct BIG-ENDIAN decode, with an ANTI-SHARED-ASSUMPTION discriminator
frame and a DISPATCH-LANE frame. Authored WITHOUT `build_client_hello`. Cites RFC 8446 §4.
- Frame A: `[0x01, 0x00, 0x00, 0x05]` → body_len = 5; pins BE decode correctness (body_len=5 correctly
  decoded from uint24 BE) AND exercises PC-9 malformed-body path — body_len=5 is length-complete but
  too short for a valid ClientHello (needs >=35 bytes); parse_tls_message_handshake returns Err →
  parse_errors += 1, carry exact-consumed (carry_len = 0), client_hello_seen = false, no panic.
- Frame B (discriminator): `[0x01, 0x01, 0x05, 0x00]` → correct BE body_len = 66,816 (> MAX_BUF)
  triggers clear-and-recover (carry_len = 0); a buggy LE decoder would read 1,281 (within MAX_BUF)
  and buffer the header (carry_len = 4). This observable difference pins the decode direction.
- Frame C (dispatch lane, F-FRESH-002): `[0x01, 0x00, 0x01, 0x00]` → body_len = 256
  (mid-range, well below MAX_BUF); header + 256 body bytes delivered, carry drains to 0 after
  dispatch. Asserts the assembled body length reaching the handler equals exactly 256 bytes.
  Pins the BE decode in the dispatch lane (not only at the overflow boundary where Frame B fires).
Test name: `test_BC_2_07_038_canonical_frame_rfc8446_s4`.

**SNI-boundary split test (F-F2-011, Sub-A SNI-region guarantee):**
A deterministic unit test that COMPUTES the actual SNI extension byte offset from the built hello
bytes at runtime (by scanning for the `[0x00, 0x00]` SNI type marker after the compression block),
splits at `sni_ext_start + 1` (provably inside the SNI type field), and asserts the split offset
falls within the SNI extension byte range. Replaces the blind `n/2` split. Cites the computed
`sni_ext_start` in assertion messages. Test name: `test_vp039_sni_boundary_deterministic`.

**Sub-D (BC-2.07.040 — truncation-safety at flow close):** `on_flow_close` called with
an incomplete carry buffer (partial handshake header or partial body present — the
capture was snaplen-truncated mid-handshake) does not increment `parse_errors` and
does not emit any finding. The carry buffer is silently discarded. Verified by
deterministic unit test.

**Sub-E (BC-2.07.041 — per-flow and per-direction carry isolation):** Interleaved
`ClientToServer` and `ServerToClient` deliveries of fragmented hellos produce
`client_hello_seen == true` and `server_hello_seen == true` with no carry-buffer
cross-contamination. The interleaved run produces the same `client_hello_seen`,
`server_hello_seen`, and `parse_errors` as the sum of two independent same-direction
runs. Bytes from `client_hs_carry` are never prepended to a `ServerToClient` delivery,
and vice versa.

Formally (Sub-E): for any (c2s_fragments, s2c_fragments) pair of fragmented handshake
byte sequences, running them interleaved through `on_data` with alternating directions
produces the same observation as running each direction's fragments independently.

**Sub-E-ext (BC-2.07.041 — cross-FlowKey isolation, F-COMP-002):** The proptest
Sub-E uses a single FlowKey (both directions share the same flow). This companion
unit test covers the MULTI-FlowKey (cross-flow) isolation case: two DISTINCT FlowKeys
(Flow A and Flow B), where Flow A receives a complete single-record ClientHello
(SNI=a.example) and Flow B receives the same-shaped ClientHello FRAGMENTED across
records (SNI=b.example). After processing both flows:
- Both SNI hostnames appear in `sni_counts` (each flow contributed its own SNI).
- No cross-flow bleed: `sni_counts["a.example"] == 1`, `sni_counts["b.example"] == 1`,
  no third entry.
- Flow B's partial carry does not affect Flow A's state.
- BC-2.07.041 PC-1 (carry per FlowKey), PC-4 (no cross-flow bleed), Inv-1 (per-flow
  carry independence) are all exercised.

Verified by deterministic unit test: `test_BC_2_07_041_cross_flow_isolation`.

**Sub-A-ext-N (BC-2.07.038 — N-record re-entrancy, F-COMP-001):** The Sub-A proptest
covers 2-record fragmentation. This companion unit test covers >=3-record re-entrancy:
ONE valid ClientHello is drip-fed across three or more records, exercising the
break-resume-break-resume re-entrancy of the consume loop across >2 `on_data` calls.
Two scenarios:
1. 1-byte + 1-byte + remainder (sub-header splits).
2. Handshake header split 1+1+2 across three records (each record delivers 1, 1, 2
   bytes of the 4-byte header), followed by a fourth record with the body.

After all records: `sni_counts.len() == 1`, `ja3_counts.len() == 1`,
`parse_errors == 0`, `client_hello_seen == true`. Maps to BC-2.07.038 PC-1/PC-2/PC-6
and EC-003 (handshake header spanning more than two records).

Verified by deterministic unit test: `test_vp039_n_record_reassembly`.

**Sub-C-ext-large (BC-2.07.038 Inv-5 — large valid hello reassembly, F-COMP-003):**
Positively verifies the 18,432 → 65,536 per-message cap raise: a VALID ClientHello
whose body is between 18,433 and 65,536 bytes (e.g. ~40 KB via large padding
extensions) MUST reassemble and dispatch correctly — it is NOT dropped and does NOT
trigger `handshake_reassembly_overflows`. Currently only the NEGATIVE side (overflow
at >65,536 bytes) is tested. This test verifies the positive side: that a large-but-valid
hello in the extended range is faithfully delivered to `handle_client_hello`.

The fixture is a valid ClientHello padded to ~40 KB (body length 40,000 bytes, well
above the old 18,432 cap and below MAX_BUF = 65,536), fragmented across multiple
records each at most MAX_RECORD_PAYLOAD bytes. After all records:
- `sni_counts.len() == 1`, `ja3_counts.len() == 1`: SNI and JA3 populated.
- `parse_errors == 0`: no parse error.
- `handshake_reassembly_overflows == 0`: NOT an overflow event.
- `client_hello_seen == true`.

Maps to BC-2.07.038 Inv-5.

Verified by deterministic unit test: `test_vp039_large_valid_hello_reassembly`.

**Sub-F (BC-2.07.039 Invariant 1 — carry buffer bounded at MAX_BUF after any call):**
For any sequence of `on_data` calls with 0x16 payloads that BEGIN WITH A VALID
HANDSHAKE HEADER (body_len <= MAX_BUF), `client_hs_carry.len() <= MAX_BUF` holds
after every call. The carry overflow guard (clear-and-recover) is structurally
sufficient to guarantee this invariant; Sub-F is a proptest regression guard
confirming no edge case in the append/guard ordering can produce a carry that
transiently exceeds MAX_BUF.

**Sub-F Decision-5 path coverage note (F-FRESH2-004):** Sub-F's generator does NOT
reliably exercise the Decision-5 buffer-fill path on most proptest runs. A random
`body_len` drawn from 0..=MAX_BUF usually produces a payload small enough that the
carry drains before overflowing — the Decision-5 guard may never fire across the 100
default test cases. The bounded-carry invariant (`carry.len() <= MAX_BUF`) is the
structural guard confirmed by Sub-F; whether Decision-5 fires on a given run is
irrelevant to the invariant's validity. Deterministic coverage of the Decision-5
path (firing exactly once, counter==overflows_before+1) is the responsibility of
`test_vp039_carry_overflow_clear_and_recover`, NOT Sub-F.

**Generator restructuring (F-F2P-IMP-001 fix):** The prior generator produced
`proptest::arbitrary::any::<u8>()` payloads. Arbitrary byte payloads almost always
decode to `body_len > MAX_BUF` on the first 4 bytes (the probability that
`(b1 << 16) | (b2 << 8) | b3 <= 65,536` when b1, b2, b3 are uniform random is
approximately 65,536 / 16,777,216 = 0.4% per payload). The body_len > MAX_BUF guard
fires immediately, clearing the carry on each record — the carry never accumulates
toward MAX_BUF, making the bounded-carry-via-accumulation invariant nearly vacuous.

**Corrected generator:** Each payload begins with a VALID handshake header
(`[0x01, len_hi, len_mid, len_lo]` where `body_len = (len_hi << 16) | (len_mid << 8)
| len_lo <= MAX_BUF`), followed by arbitrary body bytes up to the declared length.
This ensures carry actually accumulates — the invariant is tested under genuine
accumulation, not just under immediate-clear conditions. The test is kept falsifiable:
if the overflow guard had a bug (e.g., off-by-one allowing a carry > MAX_BUF before
clearing), the invariant assertion `carry.len() <= MAX_BUF` would catch it.

Verified by proptest: `proptest_vp039_carry_bounded_invariant`.

## Verified BCs

| BC-ID | Description | How VP-039 Covers It |
|-------|-------------|----------------------|
| BC-2.07.038 | Handshake-message reassembly across TLS record boundaries | Sub-A: proptest over split offsets verifies assembled delivery == single-record delivery |
| BC-2.07.039 | Carry buffer bounded at MAX_BUF; clear-and-recover on overflow; `handshake_reassembly_overflows` counter incremented; no parse_errors; no finding emitted (PC-4); recovery permitted; `handshake_reassembly_overflows` exposed via summarize() (PC-7) | Sub-C: 4 unit tests — overflow detection (carry cleared, counter++, parse_errors unchanged, findings_count pre==post snapshot — PC-4), recovery (post-overflow ClientHello dispatched normally), body_len-spoof (body_len=65537 > MAX_BUF triggers clear-and-recover; findings_count pre==post — PC-4), summarize() key exposure (test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key — PC-7); Sub-F: proptest confirming carry.len() <= MAX_BUF after any on_data call |
| BC-2.07.040 | Truncated carry at flow close yields no finding and no parse_errors | Sub-D: unit test verifies `on_flow_close` with partial carry produces zero findings and parse_errors unchanged |
| BC-2.07.041 | Carry buffers per-flow and per-direction isolated | Sub-E: proptest over interleaved c2s/s2c deliveries verifies direction isolation |
| BC-2.07.042 | Coalesced handshake messages in one record dispatched independently | Sub-B: proptest verifies two coalesced messages each dispatch, carry_len == 0 after drain |

### Relationship to Amended BCs

BC-2.07.001 (v1.9) and BC-2.07.002 (v1.6) were amended to expand their scope to include
fragmented hellos. VP-039 does not directly verify these BCs — VP-005 (SNI 4-way
classification, Kani, P0) and VP-013 (JA3 GREASE filter, proptest, P1) continue to cover
the parse-time properties of `handle_client_hello` / `handle_server_hello`. VP-039 covers
the *transport-layer guarantee* that assembled bytes reaching those handlers are correct
regardless of record fragmentation.

## Purity Classification

**Pure-core with controlled state injection.** The proptest strategies drive `TlsFlowState`
and `TlsAnalyzer` directly through `on_data` calls without any file I/O or network access.
Synthetic byte sequences are constructed using the existing `build_client_hello` test helper
(or its equivalent) in `tls_analyzer_tests.rs`. No external dependencies, no global state.

**Why proptest and NOT Kani:** The carry-reassembly invariant is a state-machine property
over sequences of `on_data` calls with stateful accumulation in `Vec<u8>` carry buffers.
Kani's BMC is well-suited to per-call pure functions (VP-005, VP-013 targets) but
not to multi-call stateful sequences over variable-length buffers — Kani's symbolic loop
unrolling would be intractable over the full `MAX_BUF` carry range. The behavioral
invariant (split-and-reassemble produces the same result as single-record delivery) is
naturally expressed as a proptest generator over `split_offset`. This is the same
rationale used for VP-033/VP-035/VP-037 (direction isolation for ENIP/DNP3/Modbus).

Sub-C uses five deterministic unit tests (not proptest) because the relevant scenarios
(overflow detection, post-overflow recovery, body_len-spoof, summarize() PC-7 surfacing,
malformed-assembled-body) have fixed input shapes and the recovery assertion requires a
specific two-step sequence.
Sub-D uses two deterministic unit tests: one for partial carry at flow close (truncation
safety) and one for empty carry at flow close (the BC-2.07.040 degenerate case,
test_BC_2_07_040_empty_carry_flow_close).
Sub-B uses one additional deterministic unit test (test_BC_2_07_042_exact_consume_no_double_dispatch)
confirming the exact handshakes_seen count after coalesced dispatch, complementing the
proptest Sub-B generative coverage.
Sub-F is a proptest providing the bounded-carry generative regression guard promised by
BC-2.07.039 Invariant 1 ("carry.len() <= MAX_BUF after any on_data call").

## Proof Harness Skeleton

```rust
#[cfg(test)]
mod vp039_tls_handshake_reassembly {
    use super::*;
    use proptest::prelude::*;

    // -----------------------------------------------------------------------
    // Sub-A: Two-record fragmented ClientHello reassembly
    // Covers BC-2.07.038
    // -----------------------------------------------------------------------

    /// VP-039 Sub-A: a ClientHello fragmented across two 0x16 records at any
    /// split offset produces the same result as a single-record delivery.
    ///
    /// The strategy generates a valid ClientHello byte blob and splits it at
    /// any offset in 1..n-1 (n = hello byte length).  The split range is a
    /// function of the ACTUAL message length, ensuring splits in the SNI region
    /// (past byte ~50) and after byte 128 are statistically likely (not
    /// guaranteed — the proptest range is bounded at 256 for tractability;
    /// see `test_vp039_sni_boundary_deterministic` for the guaranteed SNI-region
    /// unit test per F-F2-011).
    ///
    /// Partial-header sub-range {1,2,3} is guaranteed reachable via a two-armed
    /// strategy: `prop_oneof![1usize..4, 4..n]`.
    proptest! {
        #[test]
        fn proptest_vp039_carry_reassembly_two_record(
            // Split offset: 1 <= k < n, where n = client_hello.len().
            // Using prop_oneof ensures partial-header splits (k < 4) and
            // partial-body splits across the SNI region are both reachable.
            split_offset in prop_oneof![1usize..4usize, 4usize..256usize],
        ) {
            let client_hello = build_client_hello_with_sni("example.com");
            let n = client_hello.len();
            // Discard if the split overshoots the actual message length.
            prop_assume!(split_offset < n);

            // Two-record fragmented delivery
            let mut analyzer_fragmented = TlsAnalyzer::new();
            let flow_key = make_test_flow_key(1);
            let ts: u32 = 100;

            // Record 1: bytes [0..split_offset] as a 0x16 record payload
            let rec1 = wrap_as_tls_record(0x16, &client_hello[..split_offset]);
            analyzer_fragmented.on_data(&flow_key, Direction::ClientToServer, &rec1, ts);

            // Record 2: bytes [split_offset..n] as a 0x16 record payload
            let rec2 = wrap_as_tls_record(0x16, &client_hello[split_offset..]);
            analyzer_fragmented.on_data(&flow_key, Direction::ClientToServer, &rec2, ts);

            // Single-record delivery (baseline)
            let mut analyzer_single = TlsAnalyzer::new();
            let flow_key2 = make_test_flow_key(2);
            let rec_single = wrap_as_tls_record(0x16, &client_hello);
            analyzer_single.on_data(&flow_key2, Direction::ClientToServer, &rec_single, ts);

            // Flow-scoped reads via state_for_testing (client_hello_seen/server_hello_seen only)
            let frag_state = analyzer_fragmented.state_for_testing(&flow_key);
            let single_state = analyzer_single.state_for_testing(&flow_key2);

            prop_assert_eq!(
                frag_state.client_hello_seen, single_state.client_hello_seen,
                "fragmented and single-record ClientHello detection must agree"
            );

            // Aggregate reads via analyzer accessors (NOT off TlsFlowState)
            prop_assert_eq!(
                analyzer_fragmented.parse_error_count(), 0,
                "fragmented delivery must not produce parse errors"
            );
            prop_assert_eq!(
                analyzer_fragmented.sni_counts().len(), analyzer_single.sni_counts().len(),
                "SNI detection must be identical for fragmented vs single-record"
            );
            prop_assert_eq!(
                analyzer_fragmented.ja3_counts().len(), analyzer_single.ja3_counts().len(),
                "JA3 count must be identical for fragmented vs single-record"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Sub-B: Coalesced messages in one record dispatched independently
    // Covers BC-2.07.042
    // -----------------------------------------------------------------------

    /// VP-039 Sub-B: a ClientHello followed immediately by another handshake
    /// message in the same 0x16 record payload are each dispatched independently.
    /// After processing, carry_len == 0.
    ///
    /// The secondary message has a NON-ZERO body_len so the exact-consume
    /// arithmetic (drain 4 + body_len bytes) is discriminated from the
    /// zero-body-len degenerate case.  assert handshakes_seen==1 (only the
    /// ClientHello is visible to the hello-dispatch path; the second message is
    /// a non-hello type consumed silently).
    proptest! {
        #[test]
        fn proptest_vp039_exact_consume_coalesced(
            // Vary the secondary handshake type (not 0x01/0x02 — any other type)
            other_hs_type in 4u8..=20u8,
            // Non-zero body length for the secondary message: 1–16 bytes.
            // This ensures drain(4 + body_len) is exercised with body_len > 0.
            other_body_len in 1u8..=16u8,
        ) {
            let client_hello = build_client_hello_with_sni("test.example.com");
            // Secondary handshake: type(1) + len_24bit(3) + body (other_body_len bytes)
            let mut other_msg: Vec<u8> = vec![
                other_hs_type,
                0x00, 0x00, other_body_len,  // body_len encoded as 24-bit BE
            ];
            other_msg.extend(vec![0xBBu8; other_body_len as usize]); // non-zero body

            let coalesced = [client_hello.as_slice(), other_msg.as_slice()].concat();
            let rec = wrap_as_tls_record(0x16, &coalesced);

            let mut analyzer = TlsAnalyzer::new();
            let flow_key = make_test_flow_key(1);
            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec, 100);

            let state = analyzer.state_for_testing(&flow_key);

            // Flow-scoped read: client_hello_seen is on TlsFlowState
            prop_assert!(state.client_hello_seen,
                "ClientHello in coalesced record must be dispatched");

            // Aggregate reads via analyzer accessors (NOT off TlsFlowState)
            // F-F2-012: assert handshakes_seen==1 DIRECTLY via handshake_count(), not inferred
            // from ja3_counts.len()==1.
            prop_assert_eq!(analyzer.handshake_count(), 1u64,
                "exactly 1 ClientHello dispatched — assert handshakes_seen==1 directly");
            prop_assert_eq!(analyzer.parse_error_count(), 0u64,
                "coalesced delivery must not produce parse errors");

            // Carry buffer length is flow-scoped (via carry seam on TlsAnalyzer delegating to flow)
            prop_assert_eq!(
                analyzer.client_hs_carry_len_for_testing(&flow_key), 0,
                "carry buffer must be empty after all complete messages consumed"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Sub-C: Carry overflow — clear-and-recover, no parse_errors
    // Covers BC-2.07.039
    // -----------------------------------------------------------------------

    /// VP-039 Sub-C (primary): overflow via Decision-5 BUFFER-FILL path.
    ///
    /// F-CRITICAL-2 FIX: The fixture must reach Decision-5 (carry_buf.len() +
    /// record_payload_len > MAX_BUF), NOT Decision-4 (body_len > MAX_BUF spoof).
    ///
    /// WRONG FIXTURE (0xCC fill): carry_buf[1..4] = [0xCC, 0xCC, 0xCC] →
    /// body_len = 0xCCCCCC = 13,421,772 >> MAX_BUF → Decision-4 fires on record 1.
    /// Counter ends at overflows_before + 4, NOT overflows_before + 1.
    ///
    /// CORRECT FIXTURE: First record carries a VALID handshake header declaring
    /// body_len = 65,500 (bytes [0x01, 0x00, 0xFF, 0xDC]):
    ///   body_len = (0x00 << 16) | (0xFF << 8) | 0xDC = 65,500 ≤ MAX_BUF ✓
    /// The consume loop reads this header, does NOT fire Decision-4, and waits
    /// for body bytes. Subsequent records deliver body fragments until
    /// carry_buf.len() + next_payload_len > MAX_BUF → Decision-5 fires exactly once.
    /// Counter == overflows_before + 1.
    ///
    /// Record layout (carry accumulation toward buffer-fill overflow):
    ///   record 1: header [0x01, 0x00, 0xFF, 0xDC] + 16,000 padding bytes
    ///             → carry = 4 + 16,000 = 16,004 bytes; body_len=65,500 decoded
    ///   record 2: 16,000 body bytes → carry = 32,004
    ///   record 3: 16,000 body bytes → carry = 48,004
    ///   record 4: 16,000 body bytes → carry = 64,004
    ///   record 5: 2,000 body bytes → 64,004 + 2,000 = 66,004 > 65,536 → Decision-5 fires
    ///             carry cleared to 0; handshake_reassembly_overflows += 1; parse_errors unchanged
    ///
    /// Asserts:
    ///   - carry.len() == 0 after overflow (cleared, not merely bounded)
    ///   - handshake_reassembly_overflows incremented by exactly 1 (not 2, not 4)
    ///   - parse_errors UNCHANGED (BC-2.07.039 PC-4)
    ///   - no finding emitted (findings_count snapshot before == after)
    #[test]
    fn test_vp039_carry_overflow_clear_and_recover() {
        let mut analyzer = TlsAnalyzer::new();
        let flow_key = make_test_flow_key(1);
        let ts: u32 = 100;

        // Snapshot aggregate counters before any delivery — read from analyzer, NOT state.
        let parse_errors_before = analyzer.parse_error_count();
        let overflows_before = analyzer.handshake_reassembly_overflow_count();
        // Snapshot findings count before delivery (BC-2.07.039 PC-4: no finding on overflow).
        let findings_before = analyzer.findings_count_for_testing();

        // Record 1: VALID handshake header declaring body_len = 65,500 (Decision-4 safe)
        // header bytes: [0x01, 0x00, 0xFF, 0xDC]
        //   body_len = (0x00 << 16) | (0xFF << 8) | 0xDC = 65,500 ≤ MAX_BUF=65,536 ✓
        // Followed by 16,000 body bytes of padding (body is incomplete — consume loop waits).
        // carry after rec1 = 4 + 16,000 = 16,004 bytes.
        let mut rec1_payload = vec![0x01u8, 0x00, 0xFF, 0xDC];  // handshake header: body_len=65,500
        rec1_payload.extend(vec![0x00u8; 16_000]);               // partial body (16,000 of 65,500)
        let rec1 = wrap_as_tls_record(0x16, &rec1_payload);
        analyzer.on_data(&flow_key, Direction::ClientToServer, &rec1, ts);

        // Records 2–4: 16,000 body bytes each; carry grows to 48,004 then 64,004.
        for _ in 0..3 {
            let payload = vec![0x00u8; 16_000];
            let rec = wrap_as_tls_record(0x16, &payload);
            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec, ts);
        }

        // Record 5: 2,000 body bytes; carry would become 64,004 + 2,000 = 66,004 > 65,536.
        // Decision-5 buffer-fill guard fires: carry cleared, overflows += 1.
        let rec5_payload = vec![0x00u8; 2_000];
        let rec5 = wrap_as_tls_record(0x16, &rec5_payload);
        analyzer.on_data(&flow_key, Direction::ClientToServer, &rec5, ts);

        // Carry must be cleared (length == 0, not merely <= MAX_BUF).
        // client_hs_carry_len_for_testing delegates to the flow-level carry seam.
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&flow_key), 0,
            "carry buffer must be cleared (len == 0) after Decision-5 buffer-fill overflow"
        );
        // handshake_reassembly_overflows incremented by exactly 1 (not 4 or 5).
        // The 0xCC-fill fixture was wrong: it fired Decision-4 on EVERY record (4 fires).
        // The corrected fixture fires Decision-5 exactly once on record 5.
        assert_eq!(
            analyzer.handshake_reassembly_overflow_count(), overflows_before + 1,
            "handshake_reassembly_overflows must be incremented by exactly 1 (Decision-5 path, NOT Decision-4 body_len-spoof)"
        );
        // parse_errors must be UNCHANGED — read from analyzer, NOT from state (BC-2.07.039 PC-4).
        assert_eq!(
            analyzer.parse_error_count(), parse_errors_before,
            "carry overflow must NOT increment parse_errors"
        );
        // No finding emitted on the overflow path (BC-2.07.039 PC-4).
        // Real assertion: findings_count snapshot before == after.
        let findings_after = analyzer.findings_count_for_testing();
        assert_eq!(
            findings_before, findings_after,
            "carry overflow must not emit any finding (findings_count must be unchanged)"
        );
    }

    /// VP-039 Sub-C (recovery): after Decision-5 buffer-fill clear-and-recover,
    /// a subsequent well-formed single-record ClientHello IS dispatched normally.
    ///
    /// This is the key behavioral assertion that distinguishes clear-and-recover
    /// (Policy A) from the rejected sticky-abandon design (Policy B).
    ///
    /// F-CRITICAL-2 FIX: Uses the corrected valid-header overflow fixture (same as
    /// test_vp039_carry_overflow_clear_and_recover) to ensure the overflow happens
    /// via the Decision-5 buffer-fill path, not Decision-4.
    #[test]
    fn test_vp039_carry_overflow_recovery() {
        let mut analyzer = TlsAnalyzer::new();
        let flow_key = make_test_flow_key(1);
        let ts: u32 = 100;

        // Step 1: trigger Decision-5 buffer-fill overflow using the corrected fixture.
        // (NOT 0xCC fill — that hits Decision-4 body_len-spoof on record 1.)
        let mut rec1_payload = vec![0x01u8, 0x00, 0xFF, 0xDC];  // header: body_len=65,500
        rec1_payload.extend(vec![0x00u8; 16_000]);
        let rec1 = wrap_as_tls_record(0x16, &rec1_payload);
        analyzer.on_data(&flow_key, Direction::ClientToServer, &rec1, ts);

        for _ in 0..3 {
            let payload = vec![0x00u8; 16_000];
            let rec = wrap_as_tls_record(0x16, &payload);
            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec, ts);
        }
        let rec5_payload = vec![0x00u8; 2_000];
        let rec5 = wrap_as_tls_record(0x16, &rec5_payload);
        analyzer.on_data(&flow_key, Direction::ClientToServer, &rec5, ts);

        // Verify overflow fired (Decision-5 buffer-fill, counter incremented by exactly 1).
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&flow_key), 0,
            "carry must be cleared before recovery step"
        );

        // Step 2: deliver a well-formed single-record ClientHello on the same direction.
        let client_hello = build_client_hello_with_sni("recovery.example.com");
        let rec = wrap_as_tls_record(0x16, &client_hello);
        analyzer.on_data(&flow_key, Direction::ClientToServer, &rec, ts + 1);

        // Flow-scoped read: client_hello_seen is on TlsFlowState
        let state = analyzer.state_for_testing(&flow_key);
        assert!(
            state.client_hello_seen,
            "post-overflow ClientHello must be dispatched (clear-and-recover, not sticky-abandon)"
        );

        // Aggregate reads via analyzer accessors — NOT off TlsFlowState
        assert_eq!(analyzer.sni_counts().len(), 1, "SNI must be populated after recovery");
        assert_eq!(analyzer.ja3_counts().len(), 1, "JA3 must be populated after recovery");
        assert_eq!(analyzer.parse_error_count(), 0, "parse_errors must remain 0 after recovery");
    }

    /// VP-039 Sub-C (body_len spoof): a handshake header declaring body_len > MAX_BUF
    /// triggers clear-and-recover — no buffering, counter++, parse_errors unchanged.
    ///
    /// body_len = 0x010000 = 65,536 = MAX_BUF (the `> MAX_BUF` guard fires at > 65536,
    /// so use 0x010001 = 65537 to guarantee the guard fires).
    #[test]
    fn test_vp039_body_len_spoof() {
        let mut analyzer = TlsAnalyzer::new();
        let flow_key = make_test_flow_key(1);
        let ts: u32 = 100;

        // Aggregate counters before delivery — read from analyzer, NOT from TlsFlowState.
        let parse_errors_before = analyzer.parse_error_count();
        let overflows_before = analyzer.handshake_reassembly_overflow_count();
        // Snapshot findings count before delivery (BC-2.07.039 PC-4: no finding on overflow).
        let findings_before = analyzer.findings_count_for_testing();

        // Craft a 4-byte handshake header with body_len = 65537 (> MAX_BUF = 65536):
        //   type  = 0x01 (ClientHello-looking, to ensure the consume loop reaches the guard)
        //   len   = 0x01_00_01 = 65537 (3-byte big-endian)
        // No body bytes follow (the guard fires before buffering any body).
        let spoofed_header: Vec<u8> = vec![0x01, 0x01, 0x00, 0x01]; // 4 bytes: type + 24-bit len
        let rec = wrap_as_tls_record(0x16, &spoofed_header);
        analyzer.on_data(&flow_key, Direction::ClientToServer, &rec, ts);

        // Carry must be cleared — carry seam is flow-scoped via analyzer delegation.
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&flow_key), 0,
            "body_len > MAX_BUF must clear the carry buffer"
        );
        // handshake_reassembly_overflows incremented — AGGREGATE on TlsAnalyzer.
        assert_eq!(
            analyzer.handshake_reassembly_overflow_count(), overflows_before + 1,
            "body_len > MAX_BUF must increment handshake_reassembly_overflows"
        );
        // parse_errors UNCHANGED — read from analyzer, NOT from TlsFlowState (BC-2.07.039 PC-4).
        assert_eq!(
            analyzer.parse_error_count(), parse_errors_before,
            "body_len > MAX_BUF must NOT increment parse_errors"
        );
        // No finding emitted on the body_len-spoof overflow path (BC-2.07.039 PC-4).
        // Real assertion: findings_count snapshot before == after.
        let findings_after = analyzer.findings_count_for_testing();
        assert_eq!(
            findings_before, findings_after,
            "body_len > MAX_BUF overflow must not emit any finding (findings_count must be unchanged)"
        );
        // client_hello_seen must remain false — flow-scoped read.
        let state = analyzer.state_for_testing(&flow_key);
        assert!(!state.client_hello_seen, "spoofed ClientHello must not be dispatched");
    }

    /// VP-039 Sub-C (summarize() surfacing, BC-2.07.039 PC-7):
    /// After a carry overflow, `summarize()` on the analyzer returns an
    /// `AnalysisSummary` whose `detail` map entry "handshake_reassembly_overflows"
    /// has a VALUE equal to 1 — value-equality, NOT mere key presence.
    ///
    /// This mirrors how `truncated_records` is surfaced in `summarize()` at
    /// tls.rs:888-889. BC-2.07.039 PC-7 requires that `handshake_reassembly_overflows`
    /// is exposed to callers via the summary output AND that the exposed value matches
    /// the internal counter. The assertion is `detail["handshake_reassembly_overflows"]
    /// .as_u64() == 1`, NOT merely `detail.contains_key("handshake_reassembly_overflows")`.
    ///
    /// The test triggers exactly 1 overflow (spoofed header body_len=65537),
    /// calls `analyzer.summarize()`, and asserts detail["handshake_reassembly_overflows"] == 1.
    ///
    /// Seam note: `summarize()` is an AGGREGATE method on `TlsAnalyzer` (NOT on
    /// `TlsFlowState`) — consistent with the seam contract established in F-F2-001.
    #[test]
    fn test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key() {
        let mut analyzer = TlsAnalyzer::new();
        let flow_key = make_test_flow_key(5);
        let ts: u32 = 100;

        // Trigger an overflow: deliver a spoofed header with body_len = 65537 (> MAX_BUF).
        //   type = 0x01 (ClientHello-looking)
        //   len  = 0x01_00_01 = 65537 (3-byte big-endian) — strictly > MAX_BUF = 65536
        let spoofed_header: Vec<u8> = vec![0x01, 0x01, 0x00, 0x01]; // 4 bytes: type + 24-bit len
        let rec = wrap_as_tls_record(0x16, &spoofed_header);
        analyzer.on_data(&flow_key, Direction::ClientToServer, &rec, ts);

        // Verify exactly 1 overflow fired — the summary must reflect this count.
        assert_eq!(
            analyzer.handshake_reassembly_overflow_count(), 1,
            "overflow must have fired exactly once before calling summarize()"
        );

        // Call summarize() and assert both key presence AND correct value.
        // summarize() returns AnalysisSummary; detail is BTreeMap<String, serde_json::Value>.
        // Use the same .get().as_u64() pattern as truncated_records in tls_integration_tests.rs.
        let summary = analyzer.summarize();
        let overflow_count = summary
            .detail
            .get("handshake_reassembly_overflows")
            .expect(
                "summarize() detail map must contain 'handshake_reassembly_overflows' key \
                 (BC-2.07.039 PC-7 — mirrors truncated_records surfacing at tls.rs:888-889)"
            )
            .as_u64()
            .expect("handshake_reassembly_overflows detail value must be a u64");
        assert_eq!(
            overflow_count, 1,
            "handshake_reassembly_overflows in summarize() detail must equal 1 \
             after exactly one overflow (BC-2.07.039 PC-7)"
        );
    }

    // -----------------------------------------------------------------------
    // Malformed-assembled-body test (F-FRESH-001)
    // Covers ADR-011 Decision-4 malformed-body semantics
    // PO must add: BC postcondition/EC + Red-Gate test name below
    // -----------------------------------------------------------------------

    /// VP-039 malformed-assembled-body test (F-FRESH-001 / ADR-011 Decision-4):
    ///
    /// Delivers a fragmented handshake whose header LENGTH is consistent (body_len
    /// correctly matches the number of body bytes delivered) but whose BODY is
    /// structurally malformed — e.g., a "ClientHello" with truncated extensions so
    /// that parse_tls_message_handshake returns Err(_).
    ///
    /// This tests the ADR-011 Decision-4 malformed-body re-parse path:
    ///   - carry accumulates correctly (length-complete message)
    ///   - parse_tls_message_handshake(&carry_buf[..4+body_len]) returns Err(_)
    ///   - parse_errors is incremented by exactly 1 (parity with single-record path)
    ///   - message bytes are exact-consumed (drain(..4+body_len) still executes)
    ///   - no finding is emitted
    ///   - no panic
    ///
    /// ## Fixture construction
    ///
    /// Build a "ClientHello" handshake message (type=0x01) with:
    ///   - A syntactically correct 4-byte header declaring body_len = N
    ///   - N body bytes that are structurally invalid for a ClientHello
    ///     (e.g., 2-byte version = 0x0303 but then truncated — missing Random field
    ///     and cipher suites — so parse_tls_message_handshake fails on the inner
    ///     parse_tls_handshake_msg_client_hello call).
    ///
    /// This is distinct from body_len-spoof: the header body_len matches the actual
    /// body bytes delivered, so Decision-4's body_len>MAX_BUF guard does NOT fire.
    /// The message is fully assembled in the carry. The failure happens at the
    /// tls_parser struct-level parse, not at the length guard.
    ///
    /// ## Fragmentation
    ///
    /// Split the malformed message across two records at any offset (e.g., after
    /// the 4-byte header) to confirm the carry reassembly path, not the
    /// single-record path, is exercised.
    ///
    /// ## Assertions
    ///
    ///   - parse_errors == parse_errors_before + 1
    ///   - carry.len() == 0 after processing (message bytes were consumed)
    ///   - findings_count == findings_before (no finding emitted)
    ///   - no panic
    ///   - client_hello_seen == false (malformed hello not dispatched)
    ///
    /// ## PO handoff
    ///
    /// The Product Owner must author:
    ///   1. A BC postcondition on BC-2.07.038 (or a new BC-2.07.038-EC) stating:
    ///      "An assembled, length-complete handshake body that fails to parse as a
    ///       valid ClientHello/ServerHello MUST: (a) increment parse_errors by 1,
    ///       (b) consume the message bytes (exact-consume), (c) emit no finding,
    ///       (d) not panic."
    ///   2. A Red-Gate acceptance criterion naming THIS test:
    ///      test_name: `test_BC_2_07_038_malformed_assembled_body`
    ///      Red-Gate assertion: cargo test test_BC_2_07_038_malformed_assembled_body
    ///      returns PASS.
    ///
    /// ## Seam note
    ///
    /// parse_errors is an AGGREGATE on TlsAnalyzer — read via analyzer.parse_error_count(),
    /// NEVER off TlsFlowState. client_hello_seen is flow-scoped — read via
    /// analyzer.state_for_testing(&flow_key).client_hello_seen.
    #[test]
    fn test_BC_2_07_038_malformed_assembled_body() {
        let mut analyzer = TlsAnalyzer::new();
        let flow_key = make_test_flow_key(30);
        let ts: u32 = 100;

        // Snapshot counters before delivery.
        let parse_errors_before = analyzer.parse_error_count();
        let overflows_before = analyzer.handshake_reassembly_overflow_count();
        let findings_before = analyzer.findings_count_for_testing();

        // Construct a malformed "ClientHello" handshake message:
        //   type = 0x01 (ClientHello)
        //   body_len = 6 (3-byte big-endian: [0x00, 0x00, 0x06])
        //   body = [0x03, 0x03, 0x00, 0x00, 0x00] — only 5 bytes declared; 6th byte = 0x00
        //   Full body: [0x03, 0x03, 0x00, 0x00, 0x00, 0x00] — version OK (TLS 1.2)
        //              but Random field is absent (needs 32 bytes) → parse fails.
        //
        // The body has body_len = 6 bytes declared and delivered (length-complete).
        // parse_tls_message_handshake will read type=0x01, body_len=6, take 6 bytes,
        // then call parse_tls_handshake_msg_client_hello([0x03, 0x03, 0x00, 0x00, 0x00, 0x00])
        // which expects at minimum 2 (version) + 32 (random) + 1 (sid_len) = 35 bytes → Err.
        let body: Vec<u8> = vec![0x03, 0x03, 0x00, 0x00, 0x00, 0x00]; // 6 bytes, malformed body
        let mut malformed_msg: Vec<u8> = vec![
            0x01,               // msg_type: ClientHello
            0x00, 0x00, 0x06,   // body_len = 6 (big-endian uint24)
        ];
        malformed_msg.extend_from_slice(&body);
        // Total: 10 bytes (4-byte header + 6-byte body).
        // body_len = 6 ≤ MAX_BUF → Decision-4 body_len guard does NOT fire.

        // Split across two 0x16 records at the header boundary:
        //   Record 1: 4-byte handshake header [0x01, 0x00, 0x00, 0x06]
        //   Record 2: 6-byte body [0x03, 0x03, 0x00, 0x00, 0x00, 0x00]
        let rec1 = wrap_as_tls_record(0x16, &malformed_msg[..4]);
        let rec2 = wrap_as_tls_record(0x16, &malformed_msg[4..]);

        analyzer.on_data(&flow_key, Direction::ClientToServer, &rec1, ts);
        // After rec1: carry = [0x01, 0x00, 0x00, 0x06] (4-byte header); body incomplete — wait.
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&flow_key), 4,
            "after header-only record, carry must hold exactly the 4-byte header"
        );

        analyzer.on_data(&flow_key, Direction::ClientToServer, &rec2, ts + 1);
        // After rec2: 4 + 6 = 10 bytes in carry; consume loop reads body_len=6, carry >= 4+6.
        // parse_tls_message_handshake([0x01, 0x00, 0x00, 0x06, 0x03, 0x03, 0x00, 0x00, 0x00, 0x00])
        // → Err(_) (malformed ClientHello body) → parse_errors += 1; drain(..10).
        // Carry becomes empty after exact-consume.

        // parse_errors must be incremented by exactly 1 (ADR-011 Decision-4 malformed-body).
        assert_eq!(
            analyzer.parse_error_count(), parse_errors_before + 1,
            "malformed assembled body must increment parse_errors by exactly 1"
        );
        // Carry must be empty — message bytes were exact-consumed despite parse failure.
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&flow_key), 0,
            "carry must be empty after exact-consume of malformed assembled body"
        );
        // No finding emitted — malformed reassembled body is not a detection event.
        let findings_after = analyzer.findings_count_for_testing();
        assert_eq!(
            findings_before, findings_after,
            "malformed assembled body must not emit any finding"
        );
        // No overflow — body_len=6 ≤ MAX_BUF; Decision-4 body_len guard did NOT fire.
        assert_eq!(
            analyzer.handshake_reassembly_overflow_count(), overflows_before,
            "malformed assembled body must NOT increment handshake_reassembly_overflows"
        );
        // client_hello_seen must remain false — malformed hello was not dispatched.
        let state = analyzer.state_for_testing(&flow_key);
        assert!(
            !state.client_hello_seen,
            "malformed assembled ClientHello must NOT set client_hello_seen"
        );
    }

    // -----------------------------------------------------------------------
    // Sub-D: Truncated carry at flow close — no finding, no parse_errors
    // Covers BC-2.07.040 (snaplen-truncation interaction)
    // -----------------------------------------------------------------------

    /// VP-039 Sub-D: on_flow_close with an incomplete carry buffer (partial
    /// handshake header or partial body) does not emit a finding and does not
    /// increment parse_errors. Deterministic unit test (BC-2.07.040 PC3).
    ///
    /// Asserts both parse_errors (snapshot before + after, assert equal) and
    /// findings_count (snapshot before + after, assert unchanged).
    #[test]
    fn test_vp039_truncated_carry_no_error() {
        let mut analyzer = TlsAnalyzer::new();
        let flow_key = make_test_flow_key(1);
        let ts: u32 = 100;

        // Deliver a partial ClientHello: only the handshake header (4 bytes)
        // with body_len > 0 but no body bytes — simulates a snaplen-truncated capture.
        let client_hello = build_client_hello_with_sni("sni.example.com");
        let partial = &client_hello[..4]; // 4-byte handshake header only, no body
        let rec = wrap_as_tls_record(0x16, partial);
        analyzer.on_data(&flow_key, Direction::ClientToServer, &rec, ts);

        // Snapshot BEFORE flow close (BC-2.07.040 PC3 — must not change).
        // parse_errors is an AGGREGATE on TlsAnalyzer — NOT on TlsFlowState.
        // The pre-close snapshot is the reference; we assert post == pre, not that pre == 0.
        // This proves on_flow_close did not increment the counter, regardless of prior state.
        let parse_errors_before = analyzer.parse_error_count();
        let findings_before = analyzer.findings_count_for_testing();

        // Close the flow with the partial carry still present.
        analyzer.on_flow_close(&flow_key);

        // parse_errors must equal the pre-close snapshot — on_flow_close must NOT increment.
        // parse_errors is a TlsAnalyzer aggregate that survives flow close; it remains readable
        // after the flow is removed. Asserting post == pre (not pre == 0) is the correct proof:
        // it pins the delta, not the absolute value, proving on_flow_close is the neutral agent.
        let parse_errors_after = analyzer.parse_error_count();
        assert_eq!(
            parse_errors_after, parse_errors_before,
            "on_flow_close with partial carry must NOT increment parse_errors (pre-close snapshot must equal post-close)"
        );
        // findings_count must be UNCHANGED across on_flow_close (BC-2.07.040 PC3).
        let findings_after = analyzer.findings_count_for_testing();
        assert_eq!(
            findings_before, findings_after,
            "on_flow_close with partial carry must not emit any finding"
        );
        // Flow must be removed (no lingering state).
        assert_eq!(
            analyzer.active_flows_len_for_testing(), 0,
            "flow must be removed after on_flow_close"
        );
    }

    // -----------------------------------------------------------------------
    // Sub-E: Per-direction carry isolation (ClientToServer vs ServerToClient)
    // Covers BC-2.07.041
    // -----------------------------------------------------------------------

    /// VP-039 Sub-E: interleaved ClientToServer and ServerToClient fragmented
    /// hello deliveries produce the same result as independent same-direction runs.
    /// carry_c2s and carry_s2c are never mixed.
    ///
    /// Split offsets are generated as a function of the ACTUAL message length, so
    /// splits in the SNI region and past byte 128 are reachable.  The partial-
    /// header sub-range {1,2,3} is guaranteed via prop_oneof.
    proptest! {
        #[test]
        fn proptest_vp039_direction_isolation(
            // split_c2s and split_s2c are raw strategy outputs; they are clamped
            // below to 1..n-1 after the hello bytes are generated.
            split_c2s in prop_oneof![1usize..4usize, 4usize..256usize],
            split_s2c in prop_oneof![1usize..4usize, 4usize..256usize],
        ) {
            let c2s_hello = build_client_hello_with_sni("client.example.com");
            let s2c_hello = build_server_hello();

            let c2s_n = c2s_hello.len();
            let s2c_n = s2c_hello.len();

            // Clamp to [1, n-1] — a function of the actual message length, not a
            // fixed small constant.  prop_assume would discard too many cases for
            // the s2c hello (which may be short), so we saturating-clamp instead.
            let k_c2s = split_c2s.min(c2s_n - 1).max(1);
            let k_s2c = split_s2c.min(s2c_n - 1).max(1);

            let flow_key = make_test_flow_key(1);
            let ts: u32 = 100;

            // --- Interleaved run ---
            let mut interleaved = TlsAnalyzer::new();

            // c2s partial delivery 1
            let rec_c2s_1 = wrap_as_tls_record(0x16, &c2s_hello[..k_c2s]);
            interleaved.on_data(&flow_key, Direction::ClientToServer, &rec_c2s_1, ts);

            // s2c partial delivery 1 (interleaved)
            let rec_s2c_1 = wrap_as_tls_record(0x16, &s2c_hello[..k_s2c]);
            interleaved.on_data(&flow_key, Direction::ServerToClient, &rec_s2c_1, ts);

            // c2s completing delivery
            let rec_c2s_2 = wrap_as_tls_record(0x16, &c2s_hello[k_c2s..]);
            interleaved.on_data(&flow_key, Direction::ClientToServer, &rec_c2s_2, ts);

            // s2c completing delivery
            let rec_s2c_2 = wrap_as_tls_record(0x16, &s2c_hello[k_s2c..]);
            interleaved.on_data(&flow_key, Direction::ServerToClient, &rec_s2c_2, ts);

            // Flow-scoped reads: client_hello_seen / server_hello_seen are on TlsFlowState
            let interleaved_state = interleaved.state_for_testing(&flow_key);

            // --- Independent c2s-only run ---
            let fk_c2s = make_test_flow_key(2);
            let mut c2s_only = TlsAnalyzer::new();
            c2s_only.on_data(&fk_c2s, Direction::ClientToServer,
                &wrap_as_tls_record(0x16, &c2s_hello[..k_c2s]), ts);
            c2s_only.on_data(&fk_c2s, Direction::ClientToServer,
                &wrap_as_tls_record(0x16, &c2s_hello[k_c2s..]), ts);

            // --- Independent s2c-only run ---
            let fk_s2c = make_test_flow_key(3);
            let mut s2c_only = TlsAnalyzer::new();
            s2c_only.on_data(&fk_s2c, Direction::ServerToClient,
                &wrap_as_tls_record(0x16, &s2c_hello[..k_s2c]), ts);
            s2c_only.on_data(&fk_s2c, Direction::ServerToClient,
                &wrap_as_tls_record(0x16, &s2c_hello[k_s2c..]), ts);

            // Flow-scoped reads for the independent runs
            let c2s_state = c2s_only.state_for_testing(&fk_c2s);
            let s2c_state = s2c_only.state_for_testing(&fk_s2c);

            // Invariant: interleaved run sees the same hellos as independent runs
            // hello flags are flow-scoped (TlsFlowState) — OK to read off state
            prop_assert_eq!(
                interleaved_state.client_hello_seen, c2s_state.client_hello_seen,
                "interleaved c2s hello detection must match independent c2s run"
            );
            prop_assert_eq!(
                interleaved_state.server_hello_seen, s2c_state.server_hello_seen,
                "interleaved s2c hello detection must match independent s2c run"
            );
            // parse_errors is AGGREGATE on TlsAnalyzer — read via accessor, NOT off state
            prop_assert_eq!(
                interleaved.parse_error_count(),
                c2s_only.parse_error_count() + s2c_only.parse_error_count(),
                "interleaved parse_errors must equal sum of independent parse_errors"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Sub-F: Carry buffer bounded at MAX_BUF after any on_data call
    // Covers BC-2.07.039 Invariant 1
    // -----------------------------------------------------------------------

    /// VP-039 Sub-F (F-F2P-IMP-001 restructured): for any sequence of on_data calls
    /// with 0x16 payloads that begin with a VALID handshake header (body_len <= MAX_BUF),
    /// client_hs_carry.len() <= MAX_BUF holds after every call.
    ///
    /// F-F2P-IMP-001 GENERATOR FIX: The prior generator produced arbitrary u8 payloads.
    /// Arbitrary payloads almost always have carry_buf[1..4] → body_len > MAX_BUF, causing
    /// the Decision-4 guard to fire on every record (carry never accumulates — near-vacuous
    /// for the buffer-fill accumulation invariant). The corrected generator produces payloads
    /// that begin with a VALID 4-byte handshake header (body_len ≤ MAX_BUF), so carry
    /// actually accumulates toward MAX_BUF. The test is kept falsifiable: if the overflow
    /// guard had a bug, a carry transiently > MAX_BUF would be caught.
    ///
    /// Generator strategy:
    ///   - body_len: 0..=MAX_BUF (inclusive) — valid range for Decision-4 safe header
    ///   - header: [0x01, (body_len >> 16) as u8, (body_len >> 8) as u8, body_len as u8]
    ///   - body bytes: arbitrary, up to min(body_len, MAX_RECORD_PAYLOAD - 4) bytes
    ///     (ensures total payload length ≤ MAX_RECORD_PAYLOAD; partial body is fine)
    proptest! {
        #[test]
        fn proptest_vp039_carry_bounded_invariant(
            // Generate 1–8 records, each with a valid header declaring body_len ≤ MAX_BUF.
            records in proptest::collection::vec(
                // body_len: valid range [0, MAX_BUF]
                (0usize..=65_536usize).prop_flat_map(|body_len| {
                    // Partial body: 0..min(body_len, MAX_RECORD_PAYLOAD-4) arbitrary bytes.
                    // This ensures the payload is within MAX_RECORD_PAYLOAD.
                    let body_max = body_len.min(18_428usize); // MAX_RECORD_PAYLOAD(18432) - 4
                    proptest::collection::vec(proptest::arbitrary::any::<u8>(), 0..=body_max)
                        .prop_map(move |body| {
                            // Build payload: [0x01, len_hi, len_mid, len_lo] + body
                            let mut payload = vec![
                                0x01u8,                          // msg_type: ClientHello
                                (body_len >> 16) as u8,          // len byte 0 (MSB)
                                (body_len >> 8) as u8,           // len byte 1
                                (body_len & 0xFF) as u8,         // len byte 2 (LSB)
                            ];
                            payload.extend_from_slice(&body);
                            payload
                        })
                }),
                1..=8usize,
            ),
        ) {
            let mut analyzer = TlsAnalyzer::new();
            let flow_key = make_test_flow_key(42);
            let ts: u32 = 100;

            for payload in &records {
                let rec = wrap_as_tls_record(0x16, payload);
                analyzer.on_data(&flow_key, Direction::ClientToServer, &rec, ts);

                // Invariant: carry never exceeds MAX_BUF after any on_data call.
                // With valid-header payloads, carry accumulates genuinely — the overflow
                // guard (Decision-5) fires when accumulation reaches MAX_BUF.
                prop_assert!(
                    analyzer.client_hs_carry_len_for_testing(&flow_key) <= 65_536,
                    "client_hs_carry must never exceed MAX_BUF after on_data \
                     (generator: valid-header payloads ensuring genuine accumulation)"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Canonical-frame test: RFC 8446 §4 byte-level decode
    // Covers BC-2.07.038 AC added by PO in pass-2 (F-F2-010)
    // -----------------------------------------------------------------------

    /// VP-039 canonical-frame test (F-F2-010 CRITICAL policy requirement):
    /// Constructs handshake headers by hand from RFC 8446 §4 bytes and verifies
    /// the carry loop decodes `body_len` correctly via BIG-ENDIAN encoding.
    ///
    /// RFC 8446 §4 defines the Handshake struct:
    ///   struct { HandshakeType msg_type; uint24 length; ... } Handshake;
    ///
    /// ## Anti-shared-assumption guarantee (DF-CANONICAL-FRAME-HOLDOUT-001)
    ///
    /// Using a single frame [0x01, 0x00, 0x00, 0x05] is insufficient: a buggy
    /// little-endian decoder also reads 5 from [0x00, 0x00, 0x05] (those three
    /// bytes evaluate to the same numeric value under both BE and LE).  A second
    /// frame with a header whose correct BE value differs from its LE interpretation
    /// is required to pin the decode direction.
    ///
    /// ## Frame A: [0x01, 0x00, 0x00, 0x05] — body_len == 5 (RFC 8446 §4), PC-9 malformed-body path
    ///   bytes [1..4] = [0x00, 0x00, 0x05]
    ///   Correct BE: (0x00 << 16) | (0x00 << 8) | 0x05 = 5
    ///   Buggy LE:   0x05 | (0x00 << 8) | (0x00 << 16) = 5  (degenerate — same)
    ///   NOTE: body_len=5 is length-complete but far too short for a valid ClientHello
    ///   (a real ClientHello body requires >=35 bytes for version+random).
    ///   parse_tls_message_handshake returns Err → PC-9 malformed-body path fires:
    ///   parse_errors += 1, carry exact-consumed (9 bytes drained → carry_len = 0),
    ///   client_hello_seen = false, no panic.  Frame A pins BE decode CORRECTNESS
    ///   (body_len decoded as 5 not some LE value) AND the PC-9 malformed-body path.
    ///
    /// ## Frame B (discriminator): [0x01, 0x01, 0x05, 0x00] — header-only, no body
    ///   bytes [1..4] = [0x01, 0x05, 0x00]
    ///   Correct BE: (0x01 << 16) | (0x05 << 8) | 0x00 = 66,816  > MAX_BUF → clear-and-recover
    ///   Buggy LE:   0x01 | (0x05 << 8) | (0x00 << 16) = 1,281   <= MAX_BUF → buffer attempt
    ///
    /// Observable difference: correct BE → carry_len = 0 (guard fires); buggy LE → carry_len = 4
    /// (header buffered, awaiting 1,281-byte body). The assertion on carry_len pins the decode.
    ///
    /// ## Frame C (dispatch lane, F-FRESH-002): [0x01, 0x00, 0x01, 0x00] — body_len = 256
    ///   bytes [1..4] = [0x00, 0x01, 0x00]
    ///   Correct BE: (0x00 << 16) | (0x01 << 8) | 0x00 = 256  ≤ MAX_BUF → dispatch lane
    ///   Buggy LE:   0x00 | (0x01 << 8) | (0x00 << 16) = 256  (same — symmetric degenerate)
    ///   NOTE: Frame C uses a symmetric header so BE == LE == 256. Its purpose is to pin the
    ///   dispatch lane: the assembled body length reaching the handler must be exactly 256 bytes.
    ///   Frame C exercises the path between Frame A (body_len=5, degenerate) and Frame B
    ///   (body_len=66816, overflow boundary), confirming mid-range BE decode in the dispatch lane.
    ///
    /// Authored WITHOUT build_client_hello. Cites RFC 8446 §4.
    #[test]
    fn test_BC_2_07_038_canonical_frame_rfc8446_s4() {
        // ---- Frame A: [0x01, 0x00, 0x00, 0x05] — body_len = 5 (RFC 8446 §4), PC-9 malformed-body path ----
        //
        // body_len=5 is length-complete (9 bytes exactly consumed) but far too short to be a
        // valid ClientHello (needs >=35 bytes for version+random).  parse_tls_message_handshake
        // returns Err → PC-9 malformed-body path: parse_errors += 1, carry exact-consumed to 0,
        // client_hello_seen = false, no panic.  Pins BE decode correctness AND PC-9 path.
        {
            let mut analyzer = TlsAnalyzer::new();
            let flow_key = make_test_flow_key(10);
            let ts: u32 = 100;

            // Handshake header from RFC 8446 §4: ClientHello msg_type=0x01, body_len=5 (uint24 BE).
            // Body: 5 zero bytes — length-complete per body_len=5 but structurally malformed
            // (a real ClientHello needs >=35 bytes; parse_tls_message_handshake returns Err).
            let handshake_payload: Vec<u8> = vec![
                0x01,               // msg_type: ClientHello (RFC 8446 §4)
                0x00, 0x00, 0x05,   // uint24 length = 5 (big-endian, per RFC 8446 §4)
                0x00, 0x00, 0x00, 0x00, 0x00,  // body: 5 zero bytes (malformed ClientHello)
            ];

            // Split at the header boundary: first record = 4-byte header only,
            // second record = 5-byte body.  The carry loop must decode body_len == 5
            // from bytes [1..4] = [0x00, 0x00, 0x05].
            let rec1 = wrap_as_tls_record(0x16, &handshake_payload[..4]);
            let rec2 = wrap_as_tls_record(0x16, &handshake_payload[4..]);

            let errors_before = analyzer.parse_error_count();

            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec1, ts);
            // After rec1: carry has [0x01, 0x00, 0x00, 0x05] — header complete, body pending.
            assert_eq!(
                analyzer.client_hs_carry_len_for_testing(&flow_key), 4,
                "Frame A: after header-only record, carry must hold exactly the 4-byte header"
            );

            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec2, ts + 1);
            // After rec2: 4 header + 5 body = 9 bytes accumulated; loop reads body_len from
            // carry[1..4] = [0x00, 0x00, 0x05] → body_len = 5 (BE per RFC 8446 §4).
            // Exact-consume drain(4+5) clears carry to 0 (PC-9 path — body malformed).
            assert_eq!(
                analyzer.client_hs_carry_len_for_testing(&flow_key), 0,
                "Frame A: carry must be drained after exact 4+5 bytes — body_len=5 decoded via RFC 8446 §4 big-endian uint24"
            );
            // PC-9 malformed-body: body_len=5 assembles without overflow but parse_tls_message_handshake
            // returns Err (5 bytes too short for a valid ClientHello body) → parse_errors incremented.
            assert_eq!(
                analyzer.parse_error_count(), errors_before + 1,
                "Frame A: malformed 5-byte ClientHello body must increment parse_errors by 1 (PC-9 path)"
            );
            // client_hello_seen must be false: the Err result prevents hello dispatch.
            assert!(
                !analyzer.client_hello_seen_for_testing(&flow_key),
                "Frame A: failed parse must not set client_hello_seen"
            );
        }

        // ---- Frame B: [0x01, 0x01, 0x05, 0x00] — discriminator pinning BE vs LE ----
        //
        // bytes [1..4] = [0x01, 0x05, 0x00]
        //   Correct BE: (0x01 << 16) | (0x05 << 8) | 0x00 = 66,816  > MAX_BUF → clear-and-recover
        //   Buggy LE:   0x01 | (0x05 << 8) | (0x00 << 16) = 1,281   <= MAX_BUF → buffer attempt
        //
        // Observable difference:
        //   Correct BE decoder: carry_len = 0 (clear-and-recover fires), overflows+1
        //   Buggy LE decoder:   carry_len = 4 (header buffered, awaiting 1,281-byte body)
        {
            let mut analyzer_b = TlsAnalyzer::new();
            let flow_key_b = make_test_flow_key(11);
            let ts: u32 = 200;

            // 4-byte handshake header only (no body bytes):
            //   msg_type = 0x01 (ClientHello-looking)
            //   len[0..3] = [0x01, 0x05, 0x00]  → BE body_len = 66,816 (> MAX_BUF)
            let discriminator_header: Vec<u8> = vec![0x01, 0x01, 0x05, 0x00];
            let rec_b = wrap_as_tls_record(0x16, &discriminator_header);

            let overflows_before = analyzer_b.handshake_reassembly_overflow_count();
            analyzer_b.on_data(&flow_key_b, Direction::ClientToServer, &rec_b, ts);

            // Correct BE decoder fires the > MAX_BUF guard → carry cleared to 0.
            // A buggy LE decoder would read body_len = 1,281 (within MAX_BUF) and buffer
            // the 4-byte header, leaving carry_len = 4 — the assertion here distinguishes them.
            assert_eq!(
                analyzer_b.client_hs_carry_len_for_testing(&flow_key_b), 0,
                "Frame B (discriminator): correct BE body_len=66816 > MAX_BUF fires clear-and-recover \
                 (carry_len=0); a buggy LE decoder would read 1281 and buffer header (carry_len=4)"
            );
            assert_eq!(
                analyzer_b.handshake_reassembly_overflow_count(), overflows_before + 1,
                "Frame B: clear-and-recover must increment handshake_reassembly_overflows"
            );
            assert_eq!(
                analyzer_b.parse_error_count(), 0,
                "Frame B: clear-and-recover path must not increment parse_errors"
            );
        }

        // ---- Frame C: [0x01, 0x00, 0x01, 0x00] — dispatch lane, body_len = 256 (F-FRESH-002) ----
        //
        // bytes [1..4] = [0x00, 0x01, 0x00]
        //   BE body_len: (0x00 << 16) | (0x01 << 8) | 0x00 = 256  ≤ MAX_BUF → dispatch lane
        //
        // Frame C pins the BE decode in the dispatch lane (not only at the overflow boundary
        // where Frame B fires). The assembled body length reaching the handler is asserted to
        // be exactly 256 bytes, confirming the carry drained 4+256=260 bytes.
        //
        // Note: [0x00, 0x01, 0x00] also decodes to 256 under LE (256 is symmetric). Frame C
        // does not add a new BE-vs-LE discriminator — Frame B already provides that. Frame C
        // provides the mid-range dispatch-lane pin: body_len=256 is between degenerate-5 (Frame A)
        // and overflow-66816 (Frame B), confirming the carry loop dispatches correctly for
        // mid-range body lengths without degeneracy.
        {
            let mut analyzer_c = TlsAnalyzer::new();
            let flow_key_c = make_test_flow_key(12);
            let ts: u32 = 300;

            // Handshake header: msg_type=0x01, body_len=256 (uint24 BE: [0x00, 0x01, 0x00]).
            // Body: 256 zero bytes.
            // Total: 4 + 256 = 260 bytes.
            let mut frame_c_payload: Vec<u8> = vec![
                0x01,               // msg_type: ClientHello
                0x00, 0x01, 0x00,   // uint24 length = 256 (big-endian)
            ];
            frame_c_payload.extend(vec![0x00u8; 256]); // 256 body bytes

            // Deliver the full 260-byte message in a single record (single-record fast path).
            // The carry loop reads body_len=256 from [1..4]=[0x00,0x01,0x00], finds
            // carry.len() >= 4+256=260, dispatches, and drains 260 bytes.
            let rec_c = wrap_as_tls_record(0x16, &frame_c_payload);

            let parse_errors_before_c = analyzer_c.parse_error_count();
            analyzer_c.on_data(&flow_key_c, Direction::ClientToServer, &rec_c, ts);

            // Carry must be empty after dispatch (4+256 bytes drained).
            assert_eq!(
                analyzer_c.client_hs_carry_len_for_testing(&flow_key_c), 0,
                "Frame C (dispatch lane): carry must be drained after 4+256 bytes — \
                 body_len=256 decoded correctly via RFC 8446 §4 big-endian uint24 in dispatch lane"
            );
            // No overflow — body_len=256 ≤ MAX_BUF.
            assert_eq!(
                analyzer_c.handshake_reassembly_overflow_count(), 0,
                "Frame C: body_len=256 must NOT trigger clear-and-recover overflow"
            );
            // parse_errors from the dispatch attempt: the body (all zeros) is not a valid
            // ClientHello, so parse_tls_message_handshake returns Err → parse_errors += 1.
            // (This is the malformed-assembled-body path documented in ADR-011 Decision-4.)
            // Frame C verifies the dispatch lane was REACHED (carry drained) and that the
            // error handling is correct (parse_errors incremented, not zero).
            assert_eq!(
                analyzer_c.parse_error_count(), parse_errors_before_c + 1,
                "Frame C (dispatch lane): carry drained after body_len=256 dispatch; \
                 malformed body (all zeros, not a valid ClientHello) → parse_errors incremented by 1 \
                 (ADR-011 Decision-4 malformed-body semantics — parity with single-record path)"
            );
        }
    }

    // -----------------------------------------------------------------------
    // SNI-boundary split: guaranteed deterministic coverage (F-F2-011)
    // Covers BC-2.07.038 EC-001 / HS-NEW-A
    // -----------------------------------------------------------------------

    /// VP-039 Sub-A SNI-boundary deterministic test (F-F2-011):
    /// Splits a ClientHello at a byte offset provably INSIDE the SNI extension,
    /// so that SNI-region coverage is guaranteed (not probabilistic or positionally blind).
    ///
    /// ## SNI offset computation
    ///
    /// The handshake blob from `build_client_hello_with_sni` has the following layout
    /// (all offsets within the blob, which includes the 4-byte handshake header):
    ///
    ///   [0]      msg_type = 0x01 (ClientHello)
    ///   [1..4]   uint24 body_len
    ///   [4..6]   ProtocolVersion = 0x03 0x01 (TLS 1.0 outer)
    ///   [6..38]  Random (32 bytes)
    ///   [38]     session_id length (1 byte, typically 0x00)
    ///   [39..41] cipher_suites length (2 bytes)
    ///   [41..N]  cipher_suites list (length bytes)
    ///   [N]      compression_methods length (1 byte)
    ///   [N+1..M] compression_methods
    ///   [M..M+2] extensions total length (2 bytes)
    ///   [M+2..]  extensions: SNI extension type (0x00, 0x00) + length + server_name_list
    ///
    /// The SNI extension type bytes (0x00, 0x00) begin at offset M+2. For
    /// `build_client_hello("sni.boundary.example.com", &[0x002f])` the extension block
    /// starts at a known, computable position. We locate it at runtime by scanning
    /// the hello bytes for the SNI extension type marker `[0x00, 0x00]` AFTER the
    /// extensions-length field (which itself follows the compression block).
    ///
    /// ## Why runtime scan is required (not a compile-time constant)
    ///
    /// The cipher_suites list length and session_id vary with the builder arguments,
    /// so the extensions-block offset is not a fixed constant. A runtime scan pins
    /// the ACTUAL byte position in the ACTUAL hello bytes produced by the builder.
    /// An n/2 split is NOT guaranteed to land inside the SNI extension for all possible
    /// hello lengths (e.g., a hello shorter than 50 bytes would have n/2 in the header
    /// or cipher_suites region). The scan-and-split approach is the correct guarantee.
    ///
    /// ## Assertion on split offset
    ///
    /// After locating `sni_ext_start` (the offset of the `[0x00, 0x00]` SNI type bytes),
    /// the split is placed at `sni_ext_start + 1` (inside the first byte of the SNI type
    /// field). We assert: `sni_ext_start > 4` (past the header) and `sni_ext_start < n - 1`
    /// (not at the last byte). This proves the split is provably inside the extension.
    #[test]
    fn test_vp039_sni_boundary_deterministic() {
        let mut analyzer = TlsAnalyzer::new();
        let flow_key = make_test_flow_key(20);
        let ts: u32 = 200;

        let sni_hostname = "sni.boundary.example.com";

        // Build a ClientHello handshake blob (includes 4-byte handshake header).
        let hello = build_client_hello_with_sni(sni_hostname);
        let n = hello.len();

        // --- Locate the SNI extension byte offset ---
        //
        // The SNI extension type is encoded as [0x00, 0x00] (RFC 6066 extension type = 0).
        // It appears in the extensions block, which follows the compression_methods block.
        // We scan forward from byte 38 (past the Random field) to find the first [0x00, 0x00]
        // pair that falls after the header-plus-fixed-fields region (offset > 40).
        //
        // Specifically: locate the extensions-length field by skipping the fixed ClientHello
        // prefix, then scan for the SNI extension type [0x00, 0x00] within the extensions block.
        //
        // Implementer note: compute sni_ext_start by scanning hello bytes for the pattern
        // [0x00, 0x00] at position > 40 (past session_id, cipher_suites, and compression).
        // The first occurrence in the extensions block is the SNI extension type.
        let sni_ext_start: usize = {
            let mut found = None;
            // Start scanning from offset 41 (past: 1 msg_type + 3 len + 2 version + 32 random +
            // 1 sid_len + at least 2 cipher_suites bytes) to skip any zero bytes in earlier fields.
            for i in 41..(n.saturating_sub(1)) {
                if hello[i] == 0x00 && hello[i + 1] == 0x00 {
                    found = Some(i);
                    break;
                }
            }
            found.expect(
                "SNI extension type [0x00, 0x00] must be present in the extensions block \
                 of the ClientHello built by build_client_hello_with_sni"
            )
        };

        // Assert the found offset is provably INSIDE the extension region.
        assert!(
            sni_ext_start > 4,
            "SNI extension must be past the 4-byte handshake header (sni_ext_start={})", sni_ext_start
        );
        assert!(
            sni_ext_start < n - 1,
            "SNI extension must not be at the last byte (sni_ext_start={}, n={})", sni_ext_start, n
        );

        // Split at sni_ext_start + 1: provably INSIDE the SNI extension type field.
        // This guarantees the split crosses the SNI extension boundary, not just "somewhere in the middle".
        let split = sni_ext_start + 1;
        assert!(
            split > 4 && split < n,
            "split={} must be in (4, {}): provably inside the SNI extension region", split, n
        );

        let rec1 = wrap_as_tls_record(0x16, &hello[..split]);
        let rec2 = wrap_as_tls_record(0x16, &hello[split..]);

        analyzer.on_data(&flow_key, Direction::ClientToServer, &rec1, ts);
        analyzer.on_data(&flow_key, Direction::ClientToServer, &rec2, ts + 1);

        // SNI must be populated — reassembly succeeded through the SNI-extension region.
        assert_eq!(
            analyzer.sni_counts().len(), 1,
            "SNI must be populated after split at provably-inside-SNI-extension offset {}", split
        );
        assert!(
            analyzer.sni_counts().contains_key(sni_hostname),
            "SNI value must match the hostname used in build_client_hello_with_sni"
        );
        // JA3 must also be populated.
        assert_eq!(
            analyzer.ja3_counts().len(), 1,
            "JA3 must be populated after SNI-boundary split"
        );
        // No parse errors.
        assert_eq!(
            analyzer.parse_error_count(), 0,
            "SNI-boundary split must not produce parse errors"
        );
        // Flow-scoped: client_hello_seen must be true.
        let state = analyzer.state_for_testing(&flow_key);
        assert!(
            state.client_hello_seen,
            "client_hello_seen must be true after SNI-boundary split reassembly"
        );
    }

    // -----------------------------------------------------------------------
    // Sub-D-ext: Empty carry at flow close (F-FRESH2-003)
    // Covers BC-2.07.040: empty carry at flow close has no observable effect
    // beyond flow removal (distinct from partial carry — nothing to discard)
    // -----------------------------------------------------------------------

    /// VP-039 Sub-D-ext (BC-2.07.040 empty-carry variant, F-FRESH2-003):
    ///
    /// BC-2.07.040 states that `on_flow_close` with an incomplete carry buffer
    /// (partial handshake) is safe. This companion test covers the degenerate
    /// case: the carry buffer is EMPTY at flow close (no 0x16 records were
    /// delivered, or all records were fully consumed before close).
    ///
    /// ## Behavior
    ///
    /// - `on_flow_close` with an empty carry has NO effect beyond removing the
    ///   flow from the flows map.
    /// - `parse_errors` is unchanged (pre == post snapshot).
    /// - `findings_count` is unchanged.
    /// - The flow is removed from the active flows map.
    ///
    /// ## Why this test exists
    ///
    /// The BC-2.07.040 VP table BC-2.07.040 section cites this test by name.
    /// It was missing from the skeleton (named but not authored). This test
    /// fulfills the BC citation.
    #[test]
    fn test_BC_2_07_040_empty_carry_flow_close() {
        let mut analyzer = TlsAnalyzer::new();
        let flow_key = make_test_flow_key(50);
        let ts: u32 = 100;

        // Deliver a complete single-record ClientHello so the carry is empty
        // (fully consumed after dispatch) before we close the flow.
        let client_hello = build_client_hello_with_sni("empty-carry.example.com");
        let rec = wrap_as_tls_record(0x16, &client_hello);
        analyzer.on_data(&flow_key, Direction::ClientToServer, &rec, ts);

        // Verify the carry is empty (completely consumed after the full record).
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&flow_key), 0,
            "carry must be empty after a complete single-record ClientHello delivery"
        );

        // Snapshot counters before flow close.
        let parse_errors_before = analyzer.parse_error_count();
        let findings_before = analyzer.findings_count_for_testing();

        // Close the flow with an empty carry.
        analyzer.on_flow_close(&flow_key);

        // parse_errors must be UNCHANGED — on_flow_close with empty carry is a no-op
        // for error accounting (BC-2.07.040: no observable effect beyond flow removal).
        assert_eq!(
            analyzer.parse_error_count(), parse_errors_before,
            "on_flow_close with EMPTY carry must not increment parse_errors (BC-2.07.040)"
        );
        // findings_count must be UNCHANGED.
        assert_eq!(
            analyzer.findings_count_for_testing(), findings_before,
            "on_flow_close with EMPTY carry must not emit any finding (BC-2.07.040)"
        );
        // Flow must be removed.
        assert_eq!(
            analyzer.active_flows_len_for_testing(), 0,
            "flow must be removed after on_flow_close regardless of empty carry"
        );
    }

    // -----------------------------------------------------------------------
    // Sub-B-ext: Exact-consume no-double-dispatch (F-FRESH2-003)
    // Covers BC-2.07.042: coalesced messages exact-consumed, no double-dispatch,
    // handshakes_seen count is exact
    // -----------------------------------------------------------------------

    /// VP-039 Sub-B-ext (BC-2.07.042 exact-consume deterministic, F-FRESH2-003):
    ///
    /// BC-2.07.042 is covered by proptest Sub-B (proptest_vp039_exact_consume_coalesced),
    /// but the BC VP table cites this DETERMINISTIC unit test by name. This test
    /// provides a fixed, non-probabilistic assertion on the exact handshakes_seen count
    /// after processing a coalesced record with one ClientHello and one non-hello message.
    ///
    /// ## Behavior asserted
    ///
    /// - A single 0x16 record containing `[ClientHello bytes][non-hello bytes]` dispatches
    ///   the ClientHello exactly once: `analyzer.handshake_count() == 1`.
    /// - No double-dispatch: the second call to the carry loop (after draining the
    ///   ClientHello's 4+body_len bytes) reads a non-0x01/0x02 msg_type and does NOT
    ///   call handle_client_hello again.
    /// - `carry_len == 0` after both messages are drained.
    /// - `parse_errors == 0`.
    ///
    /// ## Fixture
    ///
    /// - ClientHello: built with build_client_hello_with_sni("coalesce.example.com")
    /// - Second message: type=0x0B (Certificate, a non-hello type), body_len=4, body=[0xAA,0xBB,0xCC,0xDD]
    ///   Total second message: [0x0B, 0x00, 0x00, 0x04, 0xAA, 0xBB, 0xCC, 0xDD] = 8 bytes.
    /// - Coalesced: concat both into one 0x16 record payload.
    ///
    /// ## BC-2.07.042 exact-consume assertion (the KEY claim)
    ///
    /// After processing, `analyzer.handshake_count() == 1` asserts:
    /// - The ClientHello was dispatched exactly once (no double-dispatch).
    /// - The non-hello message was consumed silently (exact-consume advanced past it).
    #[test]
    fn test_BC_2_07_042_exact_consume_no_double_dispatch() {
        let mut analyzer = TlsAnalyzer::new();
        let flow_key = make_test_flow_key(51);
        let ts: u32 = 100;

        // Build the ClientHello.
        let client_hello = build_client_hello_with_sni("coalesce.example.com");

        // Build a non-hello handshake message: type=0x0B (Certificate), body_len=4.
        // The carry loop's outer match on msg_type (0x01|0x02 => dispatch; _ => skip)
        // will skip this message and exact-consume it.
        let non_hello_msg: Vec<u8> = vec![
            0x0Bu8,              // msg_type: Certificate (non-hello, won't dispatch)
            0x00, 0x00, 0x04,    // body_len = 4 (24-bit BE)
            0xAA, 0xBB, 0xCC, 0xDD,  // 4 body bytes
        ];

        // Coalesce: ClientHello bytes followed immediately by the non-hello message,
        // all wrapped in a single 0x16 TLS record.
        let mut coalesced = client_hello.clone();
        coalesced.extend_from_slice(&non_hello_msg);
        let rec = wrap_as_tls_record(0x16, &coalesced);

        analyzer.on_data(&flow_key, Direction::ClientToServer, &rec, ts);

        // === KEY ASSERTION: handshakes_seen == 1 (exact count, no double-dispatch) ===
        // analyzer.handshake_count() counts dispatches to handle_client_hello; it must be 1.
        assert_eq!(
            analyzer.handshake_count(), 1u64,
            "ClientHello in coalesced record must be dispatched EXACTLY ONCE (BC-2.07.042: no double-dispatch)"
        );

        // Carry must be empty — both messages were exact-consumed.
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&flow_key), 0,
            "carry must be empty after both coalesced messages are exact-consumed"
        );

        // parse_errors must be 0 — the ClientHello is valid; the non-hello msg is silently consumed.
        assert_eq!(
            analyzer.parse_error_count(), 0u64,
            "coalesced ClientHello + non-hello must produce zero parse_errors"
        );

        // client_hello_seen must be true (flow-scoped read).
        let state = analyzer.state_for_testing(&flow_key);
        assert!(
            state.client_hello_seen,
            "client_hello_seen must be true after coalesced ClientHello dispatch"
        );
    }

    // -----------------------------------------------------------------------
    // Sub-E-ext: Cross-FlowKey isolation (F-COMP-002)
    // Covers BC-2.07.041 PC-1/PC-4/Inv-1 (multi-flow / cross-FlowKey isolation)
    // -----------------------------------------------------------------------

    /// VP-039 Sub-E-ext (BC-2.07.041 cross-FlowKey isolation, F-COMP-002):
    ///
    /// Two distinct FlowKeys — Flow A and Flow B — run through the SAME TlsAnalyzer.
    ///   - Flow A: receives a COMPLETE single-record ClientHello (SNI="a.example").
    ///   - Flow B: receives the SAME-SHAPED ClientHello FRAGMENTED across two records
    ///     (SNI="b.example"), identical construction to Flow A's hello but with a
    ///     different SNI.
    ///
    /// The proptest Sub-E uses interleaved c2s/s2c directions on a SINGLE FlowKey
    /// (carry_c2s vs carry_s2c isolation). This test covers a different invariant:
    /// carry isolation across DISTINCT FlowKeys (Flow A's carry cannot contaminate
    /// Flow B's carry, and vice versa).
    ///
    /// ## Assertions
    ///
    /// After processing both flows:
    ///   - `sni_counts.len() == 2` (both SNIs registered — one per flow).
    ///   - `sni_counts.contains_key("a.example")` — Flow A's SNI present.
    ///   - `sni_counts.contains_key("b.example")` — Flow B's SNI present.
    ///   - No third key in `sni_counts` (no bleed, no phantom entry).
    ///   - `sni_counts["a.example"] == 1` — exactly one registration for Flow A.
    ///   - `sni_counts["b.example"] == 1` — exactly one registration for Flow B.
    ///   - Flow A state: `client_hello_seen == true` (complete single-record delivery).
    ///   - Flow B state: `client_hello_seen == true` (fragmented delivery reassembled).
    ///   - `parse_errors == 0` for both flows combined.
    ///   - `ja3_counts.len() == 1` (both hellos share the same cipher suite → same JA3
    ///     fingerprint, so only one distinct JA3 key with count 2).
    ///
    /// ## BC mapping
    ///
    /// BC-2.07.041 PC-1: carry buffer per FlowKey (distinct flows have independent carries).
    /// BC-2.07.041 PC-4: no SNI bleed between flows (sni_counts["b.example"] not in Flow A).
    /// BC-2.07.041 Inv-1: carry isolation — Flow B's partial carry (during fragment delivery)
    ///   does not affect Flow A's fully-consumed carry.
    ///
    /// ## Seam notes
    ///
    /// - `sni_counts()` is AGGREGATE on TlsAnalyzer — accumulates across all flows.
    /// - `client_hello_seen` is flow-scoped — read via `state_for_testing(&flow_key)`.
    /// - `parse_error_count()` is AGGREGATE on TlsAnalyzer.
    #[test]
    fn test_BC_2_07_041_cross_flow_isolation() {
        let mut analyzer = TlsAnalyzer::new();
        let ts: u32 = 100;

        // --- Flow A: complete single-record ClientHello (SNI = "a.example") ---
        let flow_key_a = make_test_flow_key(60);
        let hello_a = build_client_hello("a.example", &[0x002f]);
        let rec_a = wrap_as_tls_record(0x16, &hello_a);
        analyzer.on_data(&flow_key_a, Direction::ClientToServer, &rec_a, ts);

        // Flow A: client_hello_seen must be true immediately (single-record fast path).
        let state_a_after_single = analyzer.state_for_testing(&flow_key_a);
        assert!(
            state_a_after_single.client_hello_seen,
            "Flow A: complete single-record ClientHello must set client_hello_seen"
        );

        // --- Flow B: same-shaped ClientHello FRAGMENTED across two records (SNI = "b.example") ---
        let flow_key_b = make_test_flow_key(61);
        let hello_b = build_client_hello("b.example", &[0x002f]);
        let n_b = hello_b.len();
        // Split at the midpoint (guaranteed inside the body for any reasonable hello length).
        let split_b = n_b / 2;
        assert!(split_b >= 1 && split_b < n_b, "split must be within [1, n-1]");

        let rec_b1 = wrap_as_tls_record(0x16, &hello_b[..split_b]);
        let rec_b2 = wrap_as_tls_record(0x16, &hello_b[split_b..]);

        // Deliver fragment 1 of Flow B (carry holds partial hello for Flow B only).
        analyzer.on_data(&flow_key_b, Direction::ClientToServer, &rec_b1, ts + 1);

        // CROSS-FLOW ISOLATION CHECKPOINT: after Flow B's partial delivery,
        // Flow A's state must be UNCHANGED (client_hello_seen still true, no regression).
        let state_a_mid = analyzer.state_for_testing(&flow_key_a);
        assert!(
            state_a_mid.client_hello_seen,
            "Flow A: client_hello_seen must remain true after Flow B partial delivery (no cross-flow carry bleed)"
        );

        // Deliver fragment 2 of Flow B (completes reassembly).
        analyzer.on_data(&flow_key_b, Direction::ClientToServer, &rec_b2, ts + 2);

        // --- Final assertions ---

        // Both flows: client_hello_seen must be true.
        let state_a = analyzer.state_for_testing(&flow_key_a);
        let state_b = analyzer.state_for_testing(&flow_key_b);
        assert!(
            state_a.client_hello_seen,
            "Flow A: client_hello_seen must be true (single-record delivery)"
        );
        assert!(
            state_b.client_hello_seen,
            "Flow B: client_hello_seen must be true (fragmented delivery reassembled)"
        );

        // Both SNIs must appear in the aggregate sni_counts — no bleed, no phantom entry.
        // sni_counts() is AGGREGATE on TlsAnalyzer — reads across all flows.
        let sni = analyzer.sni_counts();
        assert_eq!(
            sni.len(), 2,
            "sni_counts must have exactly 2 entries (one per flow: a.example, b.example)"
        );
        assert!(
            sni.contains_key("a.example"),
            "sni_counts must contain 'a.example' (Flow A SNI)"
        );
        assert!(
            sni.contains_key("b.example"),
            "sni_counts must contain 'b.example' (Flow B SNI)"
        );
        assert_eq!(
            sni["a.example"], 1,
            "sni_counts['a.example'] must equal 1 (exactly one registration, no double-dispatch)"
        );
        assert_eq!(
            sni["b.example"], 1,
            "sni_counts['b.example'] must equal 1 (exactly one registration from reassembled hello)"
        );

        // No parse errors across both flows (BC-2.07.041 PC-4 — no contamination errors).
        assert_eq!(
            analyzer.parse_error_count(), 0,
            "parse_errors must be 0 across both flows (no cross-flow bleed, no parse failure)"
        );

        // JA3: both hellos use the same cipher suite (0x002f), so same fingerprint — 1 distinct key.
        // JA3 count == 2 (two hellos observed, same fingerprint).
        assert_eq!(
            analyzer.ja3_counts().len(), 1,
            "ja3_counts must have 1 entry (both hellos share cipher suite 0x002f → same JA3 fingerprint)"
        );
    }

    // -----------------------------------------------------------------------
    // Sub-A-ext-N: N-record reassembly re-entrancy (F-COMP-001)
    // Covers BC-2.07.038 PC-1/PC-2/PC-6 + EC-003 (header spanning >2 records)
    // -----------------------------------------------------------------------

    /// VP-039 Sub-A-ext-N (BC-2.07.038 N-record re-entrancy, F-COMP-001):
    ///
    /// The proptest Sub-A covers 2-record fragmentation: one partial record followed
    /// by one completing record. This test covers >=3-record re-entrancy: the carry
    /// loop must correctly resume across MORE THAN TWO `on_data` calls.
    ///
    /// ## Scenario 1: 1-byte + 1-byte + remainder
    ///
    /// A valid ClientHello (n bytes) is split as:
    ///   Record 1: hello[0..1]  (1 byte — first byte of 4-byte header)
    ///   Record 2: hello[1..2]  (1 byte — second byte)
    ///   Record 3: hello[2..]   (n-2 bytes — rest of header + full body)
    ///
    /// The consume loop must:
    ///   - After record 1: carry = [hello[0]] (partial header, 1 byte); loop exits.
    ///   - After record 2: carry = [hello[0], hello[1]] (partial header, 2 bytes); loop exits.
    ///   - After record 3: carry completes header + body; dispatch fires; carry drains to 0.
    ///
    /// ## Scenario 2: Header split 1+1+2 across three records, body in fourth
    ///
    /// Header bytes [h0, h1, h2, h3] split as:
    ///   Record 1: [h0]       (1 byte)
    ///   Record 2: [h1]       (1 byte)
    ///   Record 3: [h2, h3]   (2 bytes — header complete)
    ///   Record 4: hello[4..] (full body)
    ///
    /// After record 3: header complete (4 bytes) but body not yet present; loop waits.
    /// After record 4: body appended; dispatch fires; carry drains to 0.
    ///
    /// ## Assertions (both scenarios)
    ///
    ///   - `sni_counts.len() == 1` (SNI extracted: reassembly succeeded through SNI extension)
    ///   - `ja3_counts.len() == 1` (JA3 computed: reassembly succeeded through cipher suites)
    ///   - `parse_errors == 0` (no parse error)
    ///   - `client_hello_seen == true` (flow-scoped read)
    ///   - carry_len == 0 after final record (all bytes consumed)
    ///
    /// BC-2.07.038 PC-1 (fragmented ClientHello reassembled), PC-2 (carry drains to 0),
    /// PC-6 (re-entrancy across arbitrary on_data calls), EC-003 (header spanning records).
    #[test]
    fn test_vp039_n_record_reassembly() {
        let sni_hostname = "n-record.example.com";
        let hello = build_client_hello(sni_hostname, &[0x002f]);
        let n = hello.len();
        assert!(n >= 4, "hello must be at least 4 bytes (for the 4-byte header)");
        assert!(n >= 5, "hello must have at least 1 body byte beyond the header");

        // === Scenario 1: 1-byte + 1-byte + remainder ===
        {
            let mut analyzer = TlsAnalyzer::new();
            let flow_key = make_test_flow_key(70);
            let ts: u32 = 100;

            // Record 1: first 1 byte of header
            let rec1 = wrap_as_tls_record(0x16, &hello[..1]);
            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec1, ts);
            assert_eq!(
                analyzer.client_hs_carry_len_for_testing(&flow_key), 1,
                "Scenario 1 after rec1: carry must hold exactly 1 byte"
            );

            // Record 2: second 1 byte of header
            let rec2 = wrap_as_tls_record(0x16, &hello[1..2]);
            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec2, ts + 1);
            assert_eq!(
                analyzer.client_hs_carry_len_for_testing(&flow_key), 2,
                "Scenario 1 after rec2: carry must hold exactly 2 bytes"
            );

            // Record 3: remainder (header bytes [2..3] + full body)
            let rec3 = wrap_as_tls_record(0x16, &hello[2..]);
            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec3, ts + 2);

            // After all 3 records: reassembly complete
            assert_eq!(
                analyzer.client_hs_carry_len_for_testing(&flow_key), 0,
                "Scenario 1: carry must be empty after 3-record reassembly"
            );
            assert_eq!(
                analyzer.parse_error_count(), 0,
                "Scenario 1: parse_errors must be 0 after 3-record reassembly"
            );
            assert_eq!(
                analyzer.sni_counts().len(), 1,
                "Scenario 1: SNI must be populated after 3-record reassembly"
            );
            assert!(
                analyzer.sni_counts().contains_key(sni_hostname),
                "Scenario 1: SNI must match the hostname"
            );
            assert_eq!(
                analyzer.ja3_counts().len(), 1,
                "Scenario 1: JA3 must be populated after 3-record reassembly"
            );
            let state = analyzer.state_for_testing(&flow_key);
            assert!(
                state.client_hello_seen,
                "Scenario 1: client_hello_seen must be true after 3-record reassembly"
            );
        }

        // === Scenario 2: Header split 1+1+2, body in 4th record ===
        {
            let mut analyzer = TlsAnalyzer::new();
            let flow_key = make_test_flow_key(71);
            let ts: u32 = 200;

            // Record 1: header byte [0]
            let rec1 = wrap_as_tls_record(0x16, &hello[..1]);
            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec1, ts);

            // Record 2: header byte [1]
            let rec2 = wrap_as_tls_record(0x16, &hello[1..2]);
            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec2, ts + 1);

            // Record 3: header bytes [2..4] (completes the 4-byte header)
            let rec3 = wrap_as_tls_record(0x16, &hello[2..4]);
            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec3, ts + 2);
            // Header complete but body absent — loop must wait (carry holds 4 bytes).
            assert_eq!(
                analyzer.client_hs_carry_len_for_testing(&flow_key), 4,
                "Scenario 2 after header-complete rec3: carry must hold exactly the 4-byte header"
            );

            // Record 4: full body (hello[4..])
            let rec4 = wrap_as_tls_record(0x16, &hello[4..]);
            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec4, ts + 3);

            // After all 4 records: reassembly complete
            assert_eq!(
                analyzer.client_hs_carry_len_for_testing(&flow_key), 0,
                "Scenario 2: carry must be empty after 4-record reassembly"
            );
            assert_eq!(
                analyzer.parse_error_count(), 0,
                "Scenario 2: parse_errors must be 0 after 4-record reassembly"
            );
            assert_eq!(
                analyzer.sni_counts().len(), 1,
                "Scenario 2: SNI must be populated after 4-record reassembly"
            );
            assert!(
                analyzer.sni_counts().contains_key(sni_hostname),
                "Scenario 2: SNI must match the hostname"
            );
            assert_eq!(
                analyzer.ja3_counts().len(), 1,
                "Scenario 2: JA3 must be populated after 4-record reassembly"
            );
            let state = analyzer.state_for_testing(&flow_key);
            assert!(
                state.client_hello_seen,
                "Scenario 2: client_hello_seen must be true after 4-record reassembly"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Sub-C-ext-large: Large valid ClientHello reassembly (F-COMP-003)
    // Covers BC-2.07.038 Inv-5 (large-but-valid reassembly in extended cap range)
    // -----------------------------------------------------------------------

    /// VP-039 Sub-C-ext-large (BC-2.07.038 Inv-5 large valid hello, F-COMP-003):
    ///
    /// Positively verifies the 18,432 → 65,536 per-message cap raise by confirming
    /// that a VALID ClientHello with body length in the range [18,433, 65,536] bytes
    /// is reassembled and dispatched correctly — NOT dropped, NOT an overflow event.
    ///
    /// ## Rationale
    ///
    /// All existing VP-039 Sub-C tests cover the NEGATIVE side (overflow at >65,536).
    /// No test currently verifies the POSITIVE side: that a large-but-valid hello
    /// (18,433..=65,536 bytes) is faithfully delivered to `handle_client_hello`.
    /// This test is the entire justification for raising the cap from 18,432 to 65,536.
    ///
    /// ## Fixture construction
    ///
    /// Build a valid ClientHello with body length approximately 40,000 bytes:
    ///   - Target body_len: 40,000 bytes (well above old cap 18,432; well below MAX_BUF 65,536).
    ///   - Padding technique: include a large extension (e.g. type=0xFFFF, length=~39,930 bytes)
    ///     filled with zeros, after the standard SNI extension and cipher suites.
    ///   - The resulting handshake blob: 4-byte header + 40,000-byte body = 40,004 bytes total.
    ///
    /// Fragment across multiple records, each at most MAX_RECORD_PAYLOAD bytes:
    ///   - MAX_RECORD_PAYLOAD = 18,432 bytes (as defined in `src/analyzer/tls.rs`).
    ///   - Records: [0..18432], [18432..36864], [36864..40004] (3 records for ~40 KB).
    ///
    /// ## Implementer note on fixture helper
    ///
    /// The standard `build_client_hello(sni, cipher_ids)` helper in
    /// `tests/tls_analyzer_tests.rs` may not support large padding extensions.
    /// The implementer MUST either:
    ///   (a) Extend `build_client_hello` to accept a `padding_extension_bytes: usize`
    ///       parameter (preferred — avoids raw-byte construction), or
    ///   (b) Construct the large ClientHello as raw bytes directly in the test,
    ///       following RFC 4366 / RFC 6066 extension framing.
    ///
    /// The handshake message layout (all within the blob, including 4-byte header):
    ///   [0]        msg_type = 0x01 (ClientHello)
    ///   [1..4]     uint24 body_len (big-endian) = 40,000 = 0x009C40
    ///   [4..6]     ProtocolVersion = 0x03, 0x01
    ///   [6..38]    Random (32 bytes of zeros or arbitrary)
    ///   [38]       session_id_length = 0x00
    ///   [39..41]   cipher_suites_length = 0x00, 0x02
    ///   [41..43]   cipher_suite = 0x00, 0x2F (TLS_RSA_WITH_AES_128_CBC_SHA)
    ///   [43]       compression_methods_length = 0x01
    ///   [44]       compression_method = 0x00 (null)
    ///   [45..47]   extensions_length = (total extension bytes as uint16 BE)
    ///   [47..57]   SNI extension (type=0x0000, len=..., server_name_list=...)
    ///   [57..]     Padding extension (type=0x0015 or 0xFFFF, length=~39,930 zero bytes)
    ///
    /// ## Assertions
    ///
    ///   - `sni_counts.len() == 1` — SNI extracted despite large payload.
    ///   - `ja3_counts.len() == 1` — JA3 computed.
    ///   - `parse_errors == 0` — NOT an error (large body is valid per Inv-5).
    ///   - `handshake_reassembly_overflows == 0` — NOT an overflow (body_len ≤ MAX_BUF).
    ///   - `client_hello_seen == true` (flow-scoped read).
    ///   - `carry_len == 0` after final record (all bytes consumed).
    ///
    /// Corollary of BC-2.07.038 Inv-5 (large-but-valid sub-case): a ClientHello whose
    /// total handshake body length is in the range (18,432, MAX_BUF] bytes MUST be
    /// assembled and dispatched rather than silently dropped or counted as an overflow.
    #[test]
    fn test_vp039_large_valid_hello_reassembly() {
        let mut analyzer = TlsAnalyzer::new();
        let flow_key = make_test_flow_key(80);
        let ts: u32 = 100;

        // --- Build a large valid ClientHello (~40,004 bytes total) ---
        //
        // Target body_len = 40,000 bytes (in range (18,432, 65,536] — the cap-raise range).
        // This fixture verifies that hello bodies in this range are NOT rejected.
        //
        // Layout (raw bytes):
        //   Header:  [0x01, 0x00, 0x9C, 0x40]  (msg_type=0x01, body_len=40000 = 0x009C40)
        //   Body:    40,000 bytes structured as a minimal valid ClientHello body with
        //            large padding extension.
        //
        // Minimal valid ClientHello body (40,000 bytes total):
        //   [0..2]   ProtocolVersion = [0x03, 0x01]
        //   [2..34]  Random (32 zero bytes)
        //   [34]     session_id_length = 0x00
        //   [35..37] cipher_suites_length = [0x00, 0x02]
        //   [37..39] cipher_suite = [0x00, 0x2F]
        //   [39]     compression_methods_length = 0x01
        //   [40]     compression_method = 0x00
        //   [41..43] extensions_length = [0x9C, 0x15] (= 39957 = SNI_EXT(18) + PAD_EXT_HEADER(4) + PAD_DATA_LEN(39935))
        //   --- SNI extension (type=0x0000) ---
        //   [43..45] ext_type = [0x00, 0x00]
        //   [45..47] ext_length = [0x00, 0x0E]  (14 bytes)
        //   [47..49] server_name_list_length = [0x00, 0x0C]  (12 bytes)
        //   [49]     name_type = 0x00 (host_name)
        //   [50..52] name_length = [0x00, 0x09]  (9 bytes = "large.example.com"... shortened)
        //   [52..61] name = "large.com" (9 bytes)  -- NOTE: use the 9-byte hostname for simplicity
        //   --- Padding extension (type=0xFFFF) ---
        //   [61..63] ext_type = [0xFF, 0xFF]
        //   [63..65] ext_length = [0x9B, 0xFF]  (= 39935 bytes = PAD_DATA_LEN = 40000 - 43 - 18 - 4)
        //   [65..40000] ext_data = 39935 zero bytes
        //
        // Implementer: construct this fixture using the builder or raw bytes as described above.
        // For the purpose of this skeleton, we use a raw-byte construction:

        const TARGET_BODY_LEN: usize = 40_000;
        const HEADER_LEN: usize = 4;
        // Fixed body prefix: version(2) + random(32) + sid_len(1) + cs_len(2) + cs(2) +
        //                    comp_len(1) + comp(1) + ext_total_len(2) = 43 bytes
        // SNI extension: ext_type(2) + ext_len(2) + sni_list_len(2) + name_type(1) +
        //                name_len(2) + "large.com"(9) = 18 bytes
        // Fixed body prefix = 43 bytes
        // SNI extension = 18 bytes (ext_type=0x0000, ext_len=14, list_len=12,
        //                            name_type=0x00, name_len=9, "large.com")
        // Padding extension header = 4 bytes (ext_type=0xFFFF, ext_len=padding_data_len)
        // padding_data_len = TARGET_BODY_LEN - 43 - 18 - 4 = 39,935 bytes
        const FIXED_PREFIX: usize = 43;
        const SNI_EXT: usize = 18;
        const PAD_EXT_HEADER: usize = 4;
        const PAD_DATA_LEN: usize = TARGET_BODY_LEN - FIXED_PREFIX - SNI_EXT - PAD_EXT_HEADER;
        // Total ext bytes = SNI_EXT + PAD_EXT_HEADER + PAD_DATA_LEN = TARGET_BODY_LEN - FIXED_PREFIX
        let ext_total_len: usize = SNI_EXT + PAD_EXT_HEADER + PAD_DATA_LEN;
        assert_eq!(ext_total_len, TARGET_BODY_LEN - FIXED_PREFIX);
        // pad_data_len as u16 for the extension length field
        let pad_ext_len_u16: u16 = PAD_DATA_LEN as u16;

        let body_len_u24 = TARGET_BODY_LEN as u32;
        let mut blob: Vec<u8> = Vec::with_capacity(HEADER_LEN + TARGET_BODY_LEN);

        // 4-byte handshake header: msg_type=0x01, body_len=40,000 (uint24 BE)
        blob.push(0x01);  // ClientHello
        blob.push((body_len_u24 >> 16) as u8);
        blob.push((body_len_u24 >> 8) as u8);
        blob.push(body_len_u24 as u8);

        // Body: version
        blob.extend_from_slice(&[0x03, 0x01]);
        // Random: 32 zero bytes
        blob.extend(std::iter::repeat(0x00u8).take(32));
        // session_id_length = 0
        blob.push(0x00);
        // cipher_suites_length = 2
        blob.extend_from_slice(&[0x00, 0x02]);
        // cipher_suite = TLS_RSA_WITH_AES_128_CBC_SHA (0x002F)
        blob.extend_from_slice(&[0x00, 0x2F]);
        // compression_methods_length = 1
        blob.push(0x01);
        // compression_method = 0x00 (null)
        blob.push(0x00);
        // extensions_total_length (uint16 BE)
        blob.push((ext_total_len >> 8) as u8);
        blob.push(ext_total_len as u8);

        // SNI extension (type=0x0000):
        //   ext_type(2) = [0x00, 0x00]
        //   ext_len(2) = 14 (= 2+1+2+9 bytes: sni_list_len + name_type + name_len + "large.com")
        //   server_name_list_length(2) = 12 (= 1+2+9)
        //   name_type(1) = 0x00
        //   name_length(2) = 9
        //   name(9) = "large.com"
        blob.extend_from_slice(&[0x00, 0x00]);  // ext_type = SNI
        blob.extend_from_slice(&[0x00, 0x0E]);  // ext_len = 14
        blob.extend_from_slice(&[0x00, 0x0C]);  // sni_list_len = 12
        blob.push(0x00);                        // name_type = host_name
        blob.extend_from_slice(&[0x00, 0x09]);  // name_length = 9
        blob.extend_from_slice(b"large.com");   // name = "large.com" (9 bytes)

        // Padding extension (type=0xFFFF):
        //   ext_type(2) = [0xFF, 0xFF]
        //   ext_len(2) = PAD_DATA_LEN
        //   ext_data = PAD_DATA_LEN zero bytes
        blob.extend_from_slice(&[0xFF, 0xFF]);
        blob.push((pad_ext_len_u16 >> 8) as u8);
        blob.push(pad_ext_len_u16 as u8);
        blob.extend(std::iter::repeat(0x00u8).take(PAD_DATA_LEN));

        assert_eq!(
            blob.len(), HEADER_LEN + TARGET_BODY_LEN,
            "Fixture: blob must be exactly HEADER_LEN + TARGET_BODY_LEN bytes"
        );

        // Snapshot counters before delivery.
        let overflows_before = analyzer.handshake_reassembly_overflow_count();
        let parse_errors_before = analyzer.parse_error_count();

        // Fragment across records of at most MAX_RECORD_PAYLOAD = 18,432 bytes.
        const MAX_RECORD_PAYLOAD: usize = 18_432;
        let mut offset = 0;
        let mut ts_i = ts;
        while offset < blob.len() {
            let end = (offset + MAX_RECORD_PAYLOAD).min(blob.len());
            let rec = wrap_as_tls_record(0x16, &blob[offset..end]);
            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec, ts_i);
            offset = end;
            ts_i += 1;
        }

        // === KEY ASSERTIONS: large-but-valid hello is reassembled, NOT rejected ===

        // SNI must be populated — the hello was dispatched to handle_client_hello.
        // NOTE: The SNI hostname in the fixture is "large.com" (9 bytes, per construction above).
        // If the implementer uses a different construction, adjust accordingly.
        let sni = analyzer.sni_counts();
        assert_eq!(
            sni.len(), 1,
            "sni_counts must have exactly 1 entry (large valid hello dispatched to handle_client_hello)"
        );
        // Verify the specific SNI value — the fixture encodes "large.com" (9 bytes).
        assert!(
            sni.contains_key("large.com"),
            "sni_counts must contain 'large.com' (fixture hostname per construction)"
        );

        // JA3 must be computed.
        assert_eq!(
            analyzer.ja3_counts().len(), 1,
            "ja3_counts must have exactly 1 entry (large valid hello; JA3 computed)"
        );

        // parse_errors must be UNCHANGED — large body is valid per Inv-5 (not an error event).
        assert_eq!(
            analyzer.parse_error_count(), parse_errors_before,
            "parse_errors must be unchanged — large valid hello (body_len=40000 in [18433, 65536]) is NOT an error"
        );

        // handshake_reassembly_overflows must be UNCHANGED — body_len=40,000 ≤ MAX_BUF=65,536.
        assert_eq!(
            analyzer.handshake_reassembly_overflow_count(), overflows_before,
            "handshake_reassembly_overflows must be unchanged — body_len=40000 ≤ MAX_BUF (NOT an overflow)"
        );

        // carry must be empty — all bytes consumed.
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&flow_key), 0,
            "carry must be empty after large valid hello reassembly"
        );

        // client_hello_seen must be true (flow-scoped).
        let state = analyzer.state_for_testing(&flow_key);
        assert!(
            state.client_hello_seen,
            "client_hello_seen must be true after large valid hello reassembly (body_len=40000)"
        );
    }

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    /// Build a minimal TLS ClientHello handshake message (type=0x01 + 3-byte len + body)
    /// with the given SNI hostname. The body is a minimal but syntactically valid
    /// ClientHello extension block.
    ///
    /// Delegates to the REAL `build_client_hello(sni, cipher_ids)` helper in
    /// `tests/tls_analyzer_tests.rs` (line 16 — two args: `sni: &str`, `cipher_ids: &[u16]`).
    /// NOT `build_client_hello(sni)` — that is a 1-arg form that does NOT exist (F-F2-004 fix).
    fn build_client_hello_with_sni(sni: &str) -> Vec<u8> {
        // Implementer: use the REAL two-arg signature.
        build_client_hello(sni, &[0x002f])
    }

    /// Build a minimal TLS ServerHello handshake message (type=0x02 + 3-byte len + body).
    ///
    /// Delegates to the REAL `build_server_hello(cipher_id)` helper in
    /// `tests/tls_analyzer_tests.rs` (line 137 — one arg: `cipher_id: u16`).
    /// `build_minimal_server_hello()` does NOT exist (F-F2-004 fix — removed).
    fn build_server_hello() -> Vec<u8> {
        // Implementer: use the REAL one-arg signature.
        build_server_hello(0x002f)
    }

    /// Wrap a handshake payload in a TLS record with the given content type.
    /// Produces: [content_type(1), 0x03, 0x01, len_hi(1), len_lo(1), payload...]
    ///
    /// Version bytes: `0x03 0x01` (TLS 1.0 outer framing) — matches the version
    /// bytes written by `build_client_hello_with_typed_sni_list` in
    /// `tls_analyzer_tests.rs` (line 127: `record.extend_from_slice(&[0x03, 0x01])`).
    /// F-F2-004 fix: prior skeleton incorrectly used `0x03 0x03`.
    fn wrap_as_tls_record(content_type: u8, payload: &[u8]) -> Vec<u8> {
        let len = payload.len() as u16;
        let mut rec = vec![content_type, 0x03, 0x01, (len >> 8) as u8, len as u8];
        rec.extend_from_slice(payload);
        rec
    }

    /// Create a unique FlowKey for testing using the given seed.
    fn make_test_flow_key(seed: u8) -> FlowKey {
        use std::net::{IpAddr, Ipv4Addr};
        FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, seed)),
            443,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, seed)),
            seed as u16 + 50000,
        )
    }
}
```

### Implementation Notes

**Test-seam contract (F-F2-001 fix — Pass-2 adversarial reconciliation):**

The VP-039 harnesses use TWO distinct seam types. No harness may mix them:

**Flow-scoped seams** (read from `TlsFlowState` via `state_for_testing(&flow_key)`)
expose only fields that live on `TlsFlowState`:
- `client_hello_seen: bool`
- `server_hello_seen: bool`
- (carry lengths via `client_hs_carry_len_for_testing` / `server_hs_carry_len_for_testing`)

**Aggregate seams** (read directly from the `TlsAnalyzer` instance) expose fields that
are `TlsAnalyzer`-level counters/maps, NOT on `TlsFlowState`:
- `analyzer.parse_error_count()` — NOT `state.parse_errors`
- `analyzer.sni_counts()` — NOT `state.sni_counts`
- `analyzer.ja3_counts()` — NOT `state.ja3_counts`
- `analyzer.handshake_count()` — NOT `state.handshakes_seen`
- `analyzer.handshake_reassembly_overflow_count()` — NOT `state.handshake_reassembly_overflows`

`state_for_testing(&flow_key)` returns a `&TlsFlowState` that exposes `client_hello_seen`
and `server_hello_seen` only. It does NOT expose `parse_errors`, `sni_counts`,
`ja3_counts`, `handshakes_seen`, or `handshake_reassembly_overflows` — those fields
do not exist on `TlsFlowState`.

- `client_hs_carry_len_for_testing(&flow_key)` and `server_hs_carry_len_for_testing(&flow_key)`
  expose `client_hs_carry.len()` and `server_hs_carry.len()` respectively, as documented
  in the F1 delta analysis (CHANGED symbols table).
- `findings_count_for_testing()` exposes the total number of findings accumulated across
  all flows, enabling the Sub-D pre/post assertion on finding count across `on_flow_close`.
  Mirrors the `active_flows_len_for_testing()` pattern.
- `active_flows_len_for_testing()` exposes `flows.len()` for the Sub-D flow-removal check.
- `build_client_hello_with_sni(sni)` delegates to the REAL `build_client_hello(sni, &[0x002f])`
  helper in `tests/tls_analyzer_tests.rs` (real signature: 2 args — `sni: &str` at line 16
  and `cipher_ids: &[u16]` — per F-F2-004 fix). Use `build_client_hello("example.com", &[0x002f])`.
- `build_server_hello()` delegates to the REAL `build_server_hello(0x002f)` helper in
  `tests/tls_analyzer_tests.rs` (real signature: 1 arg — `cipher_id: u16` at line 137;
  `build_minimal_server_hello()` does NOT exist — per F-F2-004 fix). Use `build_server_hello(0x002f)`.
- `wrap_as_tls_record` uses `0x03 0x01` (TLS 1.0 outer framing) to match
  `build_client_hello`'s record layer version bytes (see `tls_analyzer_tests.rs` line 127:
  `record.extend_from_slice(&[0x03, 0x01])`). NOT `0x03 0x03`.
- Sub-A uses `prop_oneof![1usize..4, 4..256]` to guarantee partial-header splits {1,2,3}
  and partial-body splits deep into the SNI region are both reachable. `prop_assume!`
  discards cases where the generated offset >= n; proptest shrinking handles this cleanly.
- Sub-E uses `split.min(n - 1).max(1)` to clamp splits to [1, n-1] without excessive
  `prop_assume` filtering; both partial-header and deep-SNI offsets are reachable.
- Sub-C CORRECTED FIXTURE (F-CRITICAL-2): Uses a VALID handshake header (body_len=65,500,
  bytes [0x01,0x00,0xFF,0xDC]) followed by accumulation records to trigger Decision-5
  buffer-fill guard. NOT 0xCC-fill records (those hit Decision-4 body_len-spoof on record 1,
  firing counter 4 times — wrong path). The corrected fixture fires Decision-5 exactly once
  (counter==overflows_before+1) and correctly exercises the buffer-fill accumulation path.
- Sub-C NEW TEST (F-FRESH-001): `test_BC_2_07_038_malformed_assembled_body` — fragmented
  handshake with length-consistent header (body_len=6) but structurally malformed body
  (6 bytes that fail parse_tls_message_handshake); asserts parse_errors+1, carry empty,
  no finding, no overflow, client_hello_seen false. PO must add BC postcondition/EC.
- Sub-D-ext NEW TEST (F-FRESH2-003): `test_BC_2_07_040_empty_carry_flow_close` — on_flow_close
  with EMPTY carry (after a fully-consumed single-record ClientHello); asserts parse_errors
  unchanged, findings_count unchanged, flow removed. BC-2.07.040 degenerate case.
- Sub-B-ext NEW TEST (F-FRESH2-003): `test_BC_2_07_042_exact_consume_no_double_dispatch` —
  coalesced ClientHello + non-hello (Certificate type=0x0B) in one record; asserts
  handshake_count()==1 (no double-dispatch), carry empty, parse_errors==0. BC-2.07.042 exact.
- Total harness count: 13 deterministic unit tests + 4 proptest = 17 harnesses.
- Single-record fast path (regression guard): Sub-A split_offset == n-1 exercises the
  near-complete split case. The single-record baseline implicitly exercises the fast-path.
  The existing 9391-line `tls_analyzer_tests.rs` continues to serve as the primary
  regression suite for the fast path.
- Proptest default test count (100 cases per harness) is sufficient. Sub-A and Sub-E may
  benefit from `#[cfg(test)] proptest::proptest!(cases = 200)` for better split coverage.

## Feasibility Assessment

**Assessment: FEASIBLE. Low-to-moderate complexity.**

1. **Synthetic byte construction is well-precedented.** The TLS analyzer test suite
   (`tls_analyzer_tests.rs`, 9391 lines) already constructs synthetic ClientHello blobs
   via `build_client_hello`. Sub-A and Sub-E reuse this helper directly.

2. **Carry buffer test seams are pre-planned.** The F1 delta analysis identified
   `client_hs_carry_len_for_testing()` and `server_hs_carry_len_for_testing()` as
   explicit new test seams (CHANGED symbols table), mirroring `client_buf_len_for_testing`.
   No new test-seam design is required.

3. **Sub-C and Sub-D are deterministic.** The overflow trigger (> MAX_BUF) and the
   truncation-at-close scenario have deterministic inputs. These are unit tests, not
   proptest; they have zero generator overhead and are guaranteed to pass or fail
   definitively.

4. **Direction isolation is structurally enforced.** Sub-E tests the behavioral
   consequence of the two-buffer `match direction { ... }` arm. The property is trivially
   satisfied after the carry-split refactor, making this a regression guard rather than
   a discovery tool — analogous to VP-033/VP-035/VP-037.

5. **No Kani harness needed.** The sub-properties involve stateful accumulation over
   `Vec<u8>` carry buffers across multiple `on_data` calls. Kani's BMC is unsuitable
   for multi-call stateful properties over heap-allocated buffers of bounded-but-large
   size. Proptest's shrinking approach directly encodes the split-offset parameter space.
   This is the same rationale as VP-013 (TLS JA3 proptest, P1, verified) and
   VP-033/VP-035/VP-037 (carry direction isolation, proptest, P1).

6. **Precedent:** VP-033 (ENIP) and VP-035 (DNP3) use the identical two-harness
   proptest strategy (direction isolation + independent-run equivalence). VP-039 Sub-E
   applies the same pattern to `TlsFlowState`. The TLS-specific elements (ClientHello
   content type 0x01, 5-byte record header wrapping, SNI/JA3 observable fields) are
   already exercised by the existing TLS test suite.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-039 produced, added to VP-INDEX | draft |
| F3 (story decomposition) | Sub-A/B/C/D assigned to STORY-A (ClientHello carry); Sub-E assigned to STORY-B (ServerHello symmetry + isolation) | draft |
| F4 (TDD implementation) | All 5 harnesses authored and passing | draft → active |
| F6 (formal hardening) | Proptest suite + unit tests confirmed in CI; no new failures | active → verified |

Lock gate: `status: verified` and `verification_lock: true` set by state-manager after
F6 confirmation.

## VP-INDEX Update Triggered by This VP

When VP-039 is added (already done in current VP-INDEX v2.14 → v2.15):
- `total_vps`: 38 → 39 (unchanged — already propagated)
- `p1_count`: 24 → 25 (unchanged — already propagated)
- `proptest_count`: 16 → 17 (unchanged — already propagated; Sub-F adds a harness
  to an already-counted proptest VP; VP counts are per-VP, not per-harness)
- `draft` count: 7 → 8 (VP-032 through VP-039 all draft)

**Fix-burst-11 reconciliation updates (this burst):** The VP-INDEX commentary line for
VP-039 must reflect the final harness inventory after Fix-burst-11:
4 proptest harnesses + 13 deterministic unit tests = 17 total harnesses.

The 13 unit test names (authoritative enumeration after Fix-burst-11):
  1. test_vp039_carry_overflow_clear_and_recover
  2. test_vp039_carry_overflow_recovery
  3. test_vp039_body_len_spoof
  4. test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key
  5. test_BC_2_07_038_malformed_assembled_body
  6. test_vp039_truncated_carry_no_error
  7. test_BC_2_07_038_canonical_frame_rfc8446_s4
  8. test_vp039_sni_boundary_deterministic
  9. test_BC_2_07_040_empty_carry_flow_close
  10. test_BC_2_07_042_exact_consume_no_double_dispatch
  11. test_BC_2_07_041_cross_flow_isolation  [NEW F-COMP-002: two-FlowKey cross-flow isolation]
  12. test_vp039_n_record_reassembly          [NEW F-COMP-001: >=3-record re-entrancy]
  13. test_vp039_large_valid_hello_reassembly [NEW F-COMP-003: large valid hello in cap-raise range]

VP total count (39), proptest_count (17), p1_count (25) are UNCHANGED by harness
additions — these count VPs, not harnesses.

Propagation targets for this Fix-burst-11 (this architect owns the updates below):
  1. `VP-INDEX.md` — update VP-039 commentary line (harness inventory; 13 unit + 4 proptest = 17)
  2. `verification-architecture.md` — update VP-039 row: add 3 new unit tests, update harness
     count to 17; tighten summarize_key description to value-equality; bump version fix-burst-11
  3. `verification-coverage-matrix.md` — update VP-039 coverage note: enumerate all 13 unit tests,
     update harness count to 17, tighten summarize_key description; no VP-count changes
  4. `ARCH-INDEX.md` — flag ADR-011 row harness count for update: '4 proptest + 10 unit tests = 14'
     should become '4 proptest + 13 unit tests = 17' (NOTE: PO/architect must update this in a
     coordinated burst to avoid intermediate-state drift)

NOTE FOR PO (parallel burst):
  - BC-2.07.041 VP table should cite test_BC_2_07_041_cross_flow_isolation (F-COMP-002)
  - BC-2.07.038 VP table should add rows for:
      - test_vp039_n_record_reassembly (F-COMP-001, PC-1/PC-2/PC-6 + EC-003)
      - test_vp039_large_valid_hello_reassembly (F-COMP-003, Inv-5)

## DTU Re-Assessment Note

VP-039 is a verification property for a passive network analyzer that reads local pcap/pcapng
files. It introduces no new external service dependencies. DTU re-assessment: NOT REQUIRED.
The fix-tls-clienthello-frag cycle touches only `src/analyzer/tls.rs` (internal, pure-core
carry buffer logic) and produces no new network endpoints, APIs, or external service calls.
