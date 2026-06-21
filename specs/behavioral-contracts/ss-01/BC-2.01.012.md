---
document_type: behavioral-contract
level: L3
version: "2.0"
status: draft
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: F2
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-01
capability: CAP-01
lifecycle_status: active
introduced: v0.10.0-pcapng
modified:
  - "v1.1: F2 Burst-A remediation per ADR-009 rev 4 PO dispatch — (1) VP-027 added to Verification Properties. (2) Postcondition 5 corrected: EPB with interface_id referencing an EMPTY table → E-INP-009 (not E-INP-008); EPB with interface_id OOB on a NON-EMPTY table → E-INP-010 (not E-INP-008). (3) Added explicit AC: interface_id MUST be bounds-checked before any indexing. (4) Added guard-before-allocate AC (SEC-004): captured_len vs. block_total_length - 32 check MUST precede any data allocation. (5) Corrected and named EPB body-relative fixed overhead = 20 bytes (EPB_FIXED_OVERHEAD_BYTES); outer 12-byte raw header is separate; validation: captured_len <= block_total_length - 32. (6) Added no-panic AC (SEC-005). (7) Added boundary edge cases (captured_len = btl-32 valid; btl-31 invalid). (8) Clarified raw-block path: timestamp is raw split ticks fed to BC-2.01.014 — NOT the crate's Duration. (9) Updated EC-005 to reflect empty-table vs OOB distinction. — 2026-06-19"
  - "v1.2: Pass-2 remediation per ADR-009 rev 5 (I-10, I-11) — (I-10) Removed duplicate ticks combine from Postcondition 1: EPB parser reads raw (ts_high, ts_low) from the block body but does NOT form ticks=(ts_high<<32)|ts_low itself; that combine is owned exclusively by BC-2.01.014. Postcondition 2 updated to reflect that the helper receives (ts_high, ts_low, if_tsresol) and performs the combine internally. (I-11) Added Test: citations to all ACs. — 2026-06-19"
  - "v1.3: Pass-3 remediation per ADR-009 rev 6 (M-5 / DF-BC-COMPLETENESS-SWEEP-001) — Added Postcondition 8: happy-path N-packet in-order + payload-fidelity guarantee, anchored to the arp-baseline-16pkt.cap fixture (16 packets). Added canonical test vector for this case. Added VP row for encounter-order and byte-fidelity. — 2026-06-19"
  - "v1.4: Pass-4 remediation per ADR-009 rev 7 (C-1, H-1/Decision-20, M-1, M-3) — (C-1) Replaced captured_len guard in PC3/AC-002/EC-009/EC-010/VP-027 with padding-aware bound: EPB_FIXED_OVERHEAD_BYTES(20) + captured_len + pad_len(captured_len) <= body.len() where pad_len(n)=(4-n%4)%4; added unconditional bound-by-body-first clause (captured_len can never exceed body.len()). (H-1/Decision-20) Added explicit mapping: 12 <= btl < 32 → body < 20 fixed-field bytes → wirerust body-decode failure → E-INP-008 (not E-INP-010); btl < 12 or btl misaligned → crate Err → E-INP-010; EPB fixed-field minimum = 20 body bytes. (M-1) Fixed AC-003: on the raw-block path the crate does NOT run its EnhancedPacketBlock parser; wirerust MUST itself check body.len() >= 20 before reading any EPB fixed field — the 20-byte check is NOT delegated to the crate. (M-3) Scoped PC8/test_BC_2_01_012_happy_path_n_packet_order_and_byte_fidelity to encounter-order + byte-fidelity on the 16-packet ARP fixture ONLY; moved EC-008 (zero-byte) and EC-009 (max-boundary) byte-fidelity claims to standalone ACs (AC-005/AC-006) and HS-104 cross-reference; removed over-claim that the single ARP fixture covers boundary cases. — 2026-06-20"
  - "v1.5: Pass-5 remediation per ADR-009 rev 8 (C-1 reclassification) — EPB body-decode failures reclassified from E-INP-010 → E-INP-008 at all sites. Decision 20 rule: the crate has already successfully framed the block (btl >= 12, aligned, trailing-length match) before any EPB body-decode runs; therefore wirerust body-decode rejections (captured_len > body.len() - 20 bound-by-body; 20 + captured_len + pad_len(captured_len) > body.len() padding-overrun) are wirerust body-decode failures → E-INP-008. Updated: PC6a (bound-by-body → E-INP-008); PC6b (padding-overrun → E-INP-008); AC-002 both sub-checks → E-INP-008; AC-006 one-over case → E-INP-008; EC-010 → E-INP-008; canonical test vectors rows for padding-overrun and bound-by-body → E-INP-008; VP-027 updated. E-INP-010 in this BC is now STRICTLY: (i) crate framing rejection (btl<12/misaligned/EOF) per EC-012; (ii) EPB interface_id OOB on non-empty table per EC-006/EC-007/PC5. — 2026-06-20"
  - "v1.6: Pass-6 remediation per ADR-009 rev 9 (F-H4 discriminant split) — PC5 split into two explicit sub-postconditions: PC5a (empty-table path → E-INP-009 with exact message format) and PC5b (OOB-on-non-empty path → E-INP-010 with exact message format). AC-001 strengthened to require the two discriminants to be DIFFERENT (empty⇒009, OOB-non-empty⇒010) — returning any single code for both cases is an AC violation. VP-027 updated: now asserts the discriminant itself (not just 'returns Err') for the empty-table vs OOB split. This resolves the F-H4 finding: prior text used ambiguous '(→ E-INP-009 / E-INP-010)' notation that did not specify which condition maps to which code. — 2026-06-20"
  - "v1.7: Pass-7 remediation per ADR-009 rev 9 (F-4 EPB decode precedence) — (1) Removed contradictory 'interface table is non-empty' assertion from Precondition 1 (empty-table is a handled case → PC5a / E-INP-009, not a precondition for success). Rewrote Precondition 1 to describe the dependency on BC-2.01.011 for if_tsresol lookup only. (2) Added Postcondition 9: explicit EPB evaluation-order postcondition pinning the 5-step precedence — (i) body.len() >= 20 else E-INP-008; (ii) read interface_id; (iii) if table EMPTY → E-INP-009 (before any captured_len / data-slice decode); (iv) if interface_id >= table.len() on non-empty table → E-INP-010; (v) captured_len bound-by-body / padding-overrun → E-INP-008. Empty-table check (step iii) is evaluated AFTER the body.len()>=20 gate (step i) but BEFORE any data-slice body-decode (step v), and is independent of captured_len arithmetic. This makes HS-104 Case (empty) and HS-108 Case C (both demand E-INP-009 exactly) unambiguously derivable from the BC. — 2026-06-20"
  - "v1.9: Pass-10 remediation (MEDIUM-1) — Removed false snaplen-attribution from PC3 and EC-002 (and the canonical test vector row). Both sites now read: captured-length-truncated by the writing tool (captured_len < original_len on the wire); wirerust copies exactly captured_len bytes and does NOT compute or apply snaplen (Decision 9 amendment: snaplen is read-and-discarded by BC-2.01.011; InterfaceInfo has no snaplen field). Invariant 2 (captured_len used for data slicing, never original_len) is unchanged. — 2026-06-20"
  - "v2.0: STORY-125 adversarial Minor-2 reconciliation — Corrected Invariant 6 to resolve spec-internal contradiction with Invariant 3. Prior text: 'original_len IS retained on the RawPacket.' This was mutually exclusive with Invariant 3 ('RawPacket struct is structurally identical to the classic-pcap RawPacket; no new fields are added'). Ground truth (confirmed from implementation): original_len IS read from the EPB body (the 5th fixed field at bytes 16-19) and IS used contextually alongside captured_len (EC-001/EC-002 truncation semantics), but is DISCARDED after reading (stored into _original_len / ignored) and is NOT retained on RawPacket. This matches the classic-pcap path, which also discards original_len. No observable behavior change — the code is correct; this is a wording correction only. No postcondition or AC asserted retention (only Invariant 6's second sentence was the offending claim). — 2026-06-20"
  - "v1.8: Pass-9 remediation (LOW-2, LOW-3) — (LOW-3) Added explicit PC6a/PC6b anchor labels to Postcondition 6 sub-items (consistent with PC5a/PC5b) so HS-104 citations resolve; PC9 step (v) now REFERENCES PC6a/PC6b instead of restating the rule (one canonical statement). (LOW-2) PC6b (padding-overrun check) marked as DEFENSE-IN-DEPTH: on a crate-framed (4-aligned) block btl-32 is a multiple of 4, so the maximum valid captured_len requires no padding, making the 20+captured_len+pad_len(captured_len) > body.len() overrun condition unreachable via a well-framed block — the crate's alignment rejection (E-INP-010) subsumes it before PC6b can fire. PC6a (unconditional bound-by-body: captured_len > body.len()) remains the live, reachable guard. Updated AC-002 and VP-027 to reflect PC6b defense-in-depth status. — 2026-06-20"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.012: Parse pcapng Enhanced Packet Block (EPB): Packet Data and Timestamp

## Description

The Enhanced Packet Block (EPB, block type `0x00000006`) is the primary packet container in
pcapng. On the raw-block path (ADR-009 Decision 1, rev 4), wirerust reads EPB fixed fields
directly from `RawBlock` body bytes: `interface_id: u32`, `ts_high: u32`, `ts_low: u32`,
`captured_len: u32`, `original_len: u32` (20 bytes of body-relative fixed overhead,
`EPB_FIXED_OVERHEAD_BYTES = 20`; outer 12-byte block header is separate). The raw split ticks
`(ts_high, ts_low)` are passed to the BC-2.01.014 pure-core helper together with the
per-interface `if_tsresol` to produce `(ts_sec: u32, ts_usecs: u32)` for `RawPacket`. The
`captured_len` field bounds the data slice and MUST be validated with a two-step check:
first, `captured_len` can never exceed `body.len()` (unconditional bound-by-body); second,
the padding-aware overhead test `EPB_FIXED_OVERHEAD_BYTES(20) + captured_len + pad_len(captured_len) <= body.len()`
(where `pad_len(n) = (4 - n%4) % 4`) must pass before any allocation. A `block_total_length`
in the range `[12, 32)` produces a body shorter than 20 bytes; wirerust MUST return
E-INP-008 (not E-INP-010) when the body is too short to hold the EPB fixed fields.

## Preconditions

1. The SHB has been parsed and byte order is established; BC-2.01.011 populates `if_tsresol`
   for each parsed IDB. The interface table MAY be empty when an EPB is encountered — the
   empty-table case is a handled error path (→ E-INP-009 via PC5a), NOT a precondition for
   successful parsing. No assumption is made about whether any IDB has been seen before
   this EPB; that state is checked at step (iii) of the EPB evaluation order (PC9).
2. The block type reads `0x00000006` (after byte-order correction from SHB).
3. The RawBlock body contains at least `EPB_FIXED_OVERHEAD_BYTES = 20` bytes. When
   `block_total_length` is in range `[12, 32)` the body is shorter than 20 bytes; wirerust
   MUST itself check `body.len() >= 20` before reading any fixed field and MUST return
   `Err` mapping to E-INP-008 (body too short to hold EPB fixed fields). This check belongs
   to wirerust, NOT the crate — the crate only frames the block (verifies btl >= 12 and
   trailing length match); it does NOT run its EnhancedPacketBlock parser on the raw path.
4. `block_total_length` is the value reported by the crate's block framing layer. The crate
   rejects `block_total_length < 12` or a misaligned/inconsistent trailing length before
   handing any block to the caller; those conditions produce a crate `Err` that wirerust
   maps to E-INP-010.

## Postconditions

1. The raw split-tick fields `ts_high: u32` and `ts_low: u32` are read from the EPB block
   body. These are the RAW values from wire bytes — NOT the crate's `Duration` type (which
   hard-codes nanoseconds and NEVER applies `if_tsresol` — confirmed at
   `enhanced_packet.rs:46-48,65`). The EPB parser DOES NOT form the combined 64-bit ticks
   value itself; combining is the exclusive responsibility of the BC-2.01.014 helper.
2. `(ts_sec, ts_usecs)` is produced by calling the BC-2.01.014 pure-core helper
   with `(ts_high, ts_low, if_tsresol)` where `if_tsresol` comes from the IDB for
   `interface_id` (defaulting to `6u8` when absent from the IDB). The helper owns the
   `ticks = (ts_high as u64) << 32 | ts_low as u64` combine and all subsequent arithmetic.
3. Packet data is copied from the EPB body bounded by `captured_len` bytes (not
   `original_len`). If `captured_len < original_len`, the packet is captured-length-truncated
   by the writing tool (`captured_len < original_len` on the wire); wirerust copies exactly
   `captured_len` bytes and does NOT compute or apply snaplen (Decision 9 amendment: snaplen
   is read-and-discarded by BC-2.01.011 IDB parsing; `InterfaceInfo` has no snaplen field).
   The `data` field carries only the captured bytes.
4. The resulting `RawPacket` is appended to the `PcapSource.packets` vector in EPB encounter
   order.
5. `interface_id` bounds checking produces two distinct discriminants depending on table state.
   These MUST be different error codes — returning the same code for both paths is an
   implementation defect and an AC-001 violation:

   a. **PC5a (empty-table path):** An EPB whose `interface_id` is evaluated when the interface
      table is EMPTY (no IDB has been parsed in the current section) MUST return `Err` mapping to
      **E-INP-009**. The error message MUST be:
      `"EPB references interface_id=<id> but interface table is empty — no IDB has been parsed"`
      (where `<id>` is the decimal value of the `interface_id` field from the EPB).

   b. **PC5b (OOB-on-non-empty-table path):** An EPB whose `interface_id` is >= the current
      table size on a NON-EMPTY table (at least one IDB has been parsed) MUST return `Err`
      mapping to **E-INP-010**. The error message MUST be:
      `"EPB interface_id=<id> out of range (table size=<n>)"`
      (where `<id>` is the decimal `interface_id` and `<n>` is the current table length).

6. Packet data slice validation uses a two-step, padding-aware check (applied in this order,
   both BEFORE any allocation):

   **PC6a (bound-by-body, unconditional — LIVE REACHABLE GUARD):** `captured_len <=
   body.len()`. A `captured_len` that exceeds the body byte count is impossible in a valid
   block and MUST return `Err` mapping to **E-INP-008** (wirerust body-decode failure —
   crate already framed the block; wirerust rejects the body content). This check is
   independently reachable: if the crate delivers a block whose captured_len field in the
   raw bytes exceeds the available body, PC6a fires.

   **PC6b (padding-aware overhead — DEFENSE-IN-DEPTH; unreachable on a crate-framed
   4-aligned block):** `EPB_FIXED_OVERHEAD_BYTES(20) + captured_len + pad_len(captured_len)
   <= body.len()` where `pad_len(n) = (4 - n % 4) % 4`. A `captured_len` that passes
   PC6a but whose padded total exceeds `body.len()` MUST return `Err` mapping to
   **E-INP-008** (wirerust body-decode failure — padding overrun). **Reachability note:**
   The crate rejects any `block_total_length` that is not 4-byte aligned before handing
   the block to wirerust (producing E-INP-010). On a well-framed (4-aligned) block,
   `btl - 32` is a multiple of 4, meaning the maximum valid `captured_len` (= btl - 32)
   requires no padding; the padded extent therefore never exceeds the data zone. Consequently,
   the PC6b overrun condition (`pad_len(captured_len) > 0` AND padded total exceeds body)
   cannot be triggered by any block that passes the crate's alignment gate. PC6b is retained
   as a guarded invariant and defense-in-depth safety net against crate bugs or future
   refactoring, but it is NOT a live operational gate for well-framed pcapng input.
7. No EPB is silently dropped on parse error — the error propagates immediately.
8. For a valid single-section pcapng file containing N EPBs, the resulting
   `PcapSource.packets` vector has exactly `packets.len() == N` entries. Packets appear in
   EPB encounter order (first EPB in the block stream → `packets[0]`, last EPB →
   `packets[N-1]`). Each packet's `data` field is byte-for-byte identical to the captured
   bytes extracted from the EPB body — no bytes are added, dropped, or reordered.

   **Test scope:** `test_BC_2_01_012_happy_path_n_packet_order_and_byte_fidelity` is
   anchored EXCLUSIVELY to the `arp-baseline-16pkt.cap` fixture (realistic 16-packet ARP
   capture): assert `packets.len() == 16`; assert each `packets[i].data` equals the
   known-good extracted bytes for position `i`; assert encounter order matches the fixture's
   EPB sequence. This test does NOT claim to exercise the zero-byte (EC-008) or
   max-boundary (EC-009) cases — those boundary cases are verified by AC-005, AC-006, and
   HS-104 respectively (see below).
9. **EPB evaluation order (decode precedence):** The EPB parse path MUST evaluate checks in
   this exact sequence with no reordering. Each step short-circuits on failure (returns `Err`
   immediately); subsequent steps are not evaluated:

   **(i) Minimum body length gate:** `body.len() >= 20` (EPB_FIXED_OVERHEAD_BYTES). If false
   → return `Err` mapping to **E-INP-008** (body too short for EPB fixed fields; wirerust
   body-decode failure; crate already successfully framed the block at btl >= 12).

   **(ii) Read interface_id:** Read `interface_id: u32` from the first 4 bytes of the body
   (little-endian, per SHB byte-order). This read is safe because step (i) guarantees
   body.len() >= 20.

   **(iii) Interface table empty check:** If the interface table length is zero (no IDB has
   been parsed in the current section) → return `Err` mapping to **E-INP-009** with message
   `"EPB references interface_id=<id> but interface table is empty — no IDB has been parsed"`.
   This check MUST occur BEFORE any captured_len read or data-slice computation. A file
   presenting EPBs before any IDB is a structural violation; the empty-table error is
   independent of the captured_len field value.

   **(iv) interface_id OOB check on non-empty table:** If the interface table is NON-EMPTY
   but `interface_id >= table.len()` → return `Err` mapping to **E-INP-010** with message
   `"EPB interface_id=<id> out of range (table size=<n>)"`.

   **(v) captured_len validation (two-step, padding-aware):** Only reached if steps (i)–(iv)
   all pass. Apply in order: **PC6a** (unconditional bound-by-body — live, reachable guard):
   if `captured_len > body.len()` → `Err` mapping to **E-INP-008**. Then **PC6b**
   (padding-aware overhead — defense-in-depth, unreachable on a crate-framed 4-aligned
   block): if `EPB_FIXED_OVERHEAD_BYTES(20) + captured_len + pad_len(captured_len) >
   body.len()` → `Err` mapping to **E-INP-008**. See PC6 for the canonical statement of
   both sub-checks, including the PC6b reachability note.

   **Implication for HS-104 and HS-108:** An EPB presented when the interface table is EMPTY
   must produce E-INP-009 regardless of whether captured_len would also be malformed — the
   empty-table check at step (iii) fires before any captured_len / data-slice arithmetic at
   step (v). This makes the E-INP-009 outcome for the empty-table case unambiguous and
   independently derivable from this BC.

## Acceptance Criteria

- **AC-001 (interface_id bounds-check before indexing with discriminant split):** The
  `interface_id` field MUST be checked against the current interface table state before any
  indexing operation. The check MUST produce TWO DIFFERENT error discriminants:
  - Empty-table path (zero IDBs parsed) → E-INP-009 EXACTLY (not E-INP-010).
  - OOB-on-non-empty-table path (>= 1 IDB parsed, but interface_id >= table.len()) → E-INP-010
    EXACTLY (not E-INP-009).
  Returning the same error code for both paths is an AC-001 violation. An unchecked array
  index on `interface_id` is NOT permitted regardless of table state.
  **Test:** `test_BC_2_01_012_interface_id_bounds_check` (must assert E-INP-009 for
  empty-table case AND E-INP-010 for OOB-non-empty case and confirm they are different codes)
- **AC-002 (guard-before-allocate, SEC-004):** Two validations MUST be performed BEFORE any
  memory allocation for packet data, in this order:
  1. **PC6a** — Unconditional bound-by-body: `captured_len <= body.len()` — the data slice can
     NEVER exceed the raw body length regardless of `block_total_length`. Failure → `Err`
     mapping to **E-INP-008** (wirerust body-decode failure; crate already framed the block).
     This is the live, operationally reachable guard.
  2. **PC6b** — Padding-aware overhead: `EPB_FIXED_OVERHEAD_BYTES(20) + captured_len +
     pad_len(captured_len) <= body.len()` where `pad_len(n) = (4 - n % 4) % 4`. Failure →
     `Err` mapping to **E-INP-008** (wirerust body-decode failure; padding overrun). This
     check is DEFENSE-IN-DEPTH: on a crate-framed 4-aligned block, the crate's alignment
     rejection (E-INP-010) subsumes the condition before PC6b can fire. The check MUST still
     be coded and MUST still fire if (hypothetically) a non-aligned block bypassed the crate
     gate — it is a safety net, not a dead letter.
  Allocating based on an attacker-controlled `captured_len` without both checks is prohibited.
  **Test:** `test_BC_2_01_012_guard_before_allocate`
- **AC-003 (no-panic, SEC-005):** This block parser MUST return `Err` for any malformed or
  truncated input; `unwrap()`, `expect()`, and `panic!()` are prohibited in the EPB parse path.
  **IMPORTANT:** On the raw-block path, the crate does NOT run its `EnhancedPacketBlock`
  parser — it only frames the block (validates `block_total_length >= 12` and trailing
  length consistency). Therefore wirerust MUST itself check `body.len() >= 20` before
  reading any EPB fixed field. Do NOT attribute this check to the crate; do NOT rely on a
  crate-level `slice.len() < 20` guard that does not run on this path.
  **Test:** `test_BC_2_01_012_no_panic_malformed`
- **AC-004 (raw-block path):** The raw split ticks `(ts_high, ts_low)` MUST be read from the
  `RawBlock` body and passed to the BC-2.01.014 helper. wirerust MUST NOT consume
  `EnhancedPacketBlock::timestamp` (the crate's `Duration` type) — that field hard-codes
  nanosecond resolution and discards the raw ticks, making tsresol-correct conversion
  impossible.
  **Test:** `test_BC_2_01_012_raw_block_path_not_crate_duration`
- **AC-005 (zero-byte captured_len fidelity, EC-008):** When `captured_len = 0`, wirerust
  MUST produce `RawPacket { data: vec![] }` — a valid zero-byte packet. The padding-aware
  check still passes (`20 + 0 + 0 <= body.len()` for any valid 20-byte-minimum body). The
  byte-fidelity guarantee holds: `data` is empty, not absent.
  **Test:** `test_BC_2_01_012_zero_byte_captured_len` (unit; synthetic EPB with
  `captured_len=0`). Cross-referenced by HS-104.
- **AC-006 (maximum-boundary captured_len fidelity, EC-009):** When `captured_len` equals
  the largest value satisfying the padding-aware bound (i.e.,
  `20 + captured_len + pad_len(captured_len) == body.len()`), wirerust MUST produce
  `Ok(RawPacket)` with `data.len() == captured_len`. A `captured_len` one byte larger that
  causes the padded total to exceed `body.len()` MUST return `Err` mapping to **E-INP-008**
  (wirerust body-decode failure — padding overrun; crate framed the block successfully,
  wirerust body-decode rejects the padded extent).
  **Test:** `test_BC_2_01_012_max_boundary_captured_len` (unit; synthetic EPB at exact
  boundary and one-over). Cross-referenced by HS-104.

## Invariants

1. Packet order in `PcapSource.packets` matches EPB encounter order in the block stream.
2. `captured_len` MUST be used for data slicing, never `original_len`. Using
   `original_len` would read past the actual bytes in the block.
3. The `RawPacket` struct produced by EPB parsing is structurally identical to the struct
   produced by classic-pcap parsing; no new fields are added.
4. An EPB's `interface_id` must resolve to an already-seen IDB; forward references (EPB before
   any IDB) produce E-INP-009 — a pcapng structural violation.
5. `EPB_FIXED_OVERHEAD_BYTES = 20` (body-relative: interface_id:4 + ts_high:4 + ts_low:4 +
   captured_len:4 + original_len:4). The outer 12-byte block header
   (block_type:4 + block_total_length:4 + trailing_total_length:4) is NOT included in this
   constant. The combined minimum block size is therefore 32 bytes (12 + 20).
6. The `captured_len` field is NOT retained on the parsed type (`data.len()` recovers it).
   `original_len` is READ from the EPB body (the 5th 4-byte fixed field, bytes 16–19 of the
   body) and is used contextually to characterise the captured/original-length relationship
   (EC-001: `captured_len == original_len` — no truncation; EC-002: `captured_len <
   original_len` — writing-tool truncation). After this contextual use, `original_len` is
   DISCARDED and is NOT stored on `RawPacket`. This is consistent with Invariant 3: the
   `RawPacket` struct is structurally identical to the classic-pcap `RawPacket`; no new
   fields are added. The classic-pcap path likewise reads then discards `orig_len` from the
   packet record header without storing it.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `captured_len == original_len` (no truncation) | Data copied in full; normal case |
| EC-002 | `captured_len < original_len` (captured-length-truncated by the writing tool) | Data bounded to `captured_len`; wirerust copies exactly `captured_len` bytes and does NOT compute or apply snaplen (Decision 9 amendment); truncated `RawPacket` produced; downstream decoder handles via lax fallback |
| EC-003 | `ts_high = 0, ts_low = 0` | `timestamp_secs=0, timestamp_usecs=0`; valid zero-epoch packet |
| EC-004 | `ts_high` and `ts_low` combine to a very large u64 (near u64::MAX) | BC-2.01.014 saturating arithmetic handles; `ts_sec` saturates at u32::MAX; no panic |
| EC-005 | EPB `interface_id = 0` with EMPTY interface table (no IDB seen yet) | `Err` mapping to E-INP-009 (empty-table path) |
| EC-006 | EPB `interface_id = 1` with only one IDB (index 0) in non-empty table | `Err` mapping to E-INP-010 (OOB on non-empty table); context: `"EPB interface_id=1 out of range (table size=1)"` |
| EC-007 | EPB `interface_id = u32::MAX` with any non-empty table | `Err` mapping to E-INP-010 (OOB on non-empty table) |
| EC-008 | `captured_len = 0` (zero-length captured data) | `RawPacket { data: vec![] }`; zero-byte packet is valid |
| EC-009 | `captured_len` at maximum padding-aware boundary: `20 + captured_len + pad_len(captured_len) == body.len()` | Exactly valid; padded data occupies the entire remaining body after fixed fields; `Ok(RawPacket)` with `data.len() == captured_len` |
| EC-010 | `captured_len` one byte over the padding-aware boundary: `20 + captured_len + pad_len(captured_len) > body.len()` | `Err` mapping to E-INP-008 (wirerust body-decode failure — padded total exceeds body; crate framed the block successfully, wirerust rejects the padded extent) |
| EC-011 | EPB body shorter than 20 bytes due to `block_total_length` in `[12, 32)` (wirerust body-decode, btl present but too small) | `Err` mapping to E-INP-008 (body too short for EPB fixed fields — NOT E-INP-010) |
| EC-012 | `block_total_length < 12` or trailing-length mismatch (crate framing rejects before body is accessible) | Crate returns `Err`; wirerust maps to E-INP-010 (block framing error, distinct from body-too-short) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| EPB with `ts_high=0, ts_low=1000000`, `if_tsresol` absent (default µs) | `RawPacket { timestamp_secs: 1, timestamp_usecs: 0 }` | happy-path |
| EPB with `ts_high=0, ts_low=1500000000`, `if_tsresol=0x09` (nanoseconds) | `RawPacket { timestamp_secs: 1, timestamp_usecs: 500000 }` | happy-path |
| EPB with `captured_len=64, original_len=1500` | `RawPacket { data.len() == 64 }` (captured-length-truncated; snaplen not applied — Decision 9 amendment) | edge-case |
| EPB with `interface_id=0`, empty interface table (no IDB) | `Err` mapping to E-INP-009 | error |
| EPB with `interface_id=5`, one IDB (index 0 only) | `Err` mapping to E-INP-010; context includes `"interface_id=5 out of range (table size=1)"` | error |
| EPB where `20 + captured_len + pad_len(captured_len) == body.len()` (exact padding-aware boundary) | `Ok(RawPacket)` with `data.len() == captured_len` | boundary-valid (EC-009) |
| EPB where `20 + captured_len + pad_len(captured_len) > body.len()` (one byte over padding-aware boundary) | `Err` mapping to E-INP-008 (wirerust body-decode: padding overrun; crate framed the block) | boundary-invalid (EC-010) |
| EPB body shorter than 20 bytes (`block_total_length` in `[12, 32)`) | `Err` mapping to E-INP-008 (body too short — NOT E-INP-010) | error (EC-011 / Decision 20) |
| `block_total_length < 12` or trailing-length mismatch | Crate `Err` → wirerust maps to E-INP-010 | error (EC-012) |
| `arp-baseline-16pkt.cap` fixture (16 EPBs, single section, LE, if_tsresol=6) | `packets.len() == 16`; each `packets[i].data` byte-identical to known-good extraction; encounter order preserved | happy-path (M-5 byte-fidelity) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-027 | EPB parse safety: no panic; interface_id discriminant split — empty-table MUST return E-INP-009 (not E-INP-010), OOB-on-non-empty MUST return E-INP-010 (not E-INP-009); the two codes MUST be distinct (returning one code for both paths fails this VP); body.len() >= 20 check (wirerust-owned) before any fixed-field read (body < 20 → E-INP-008); PC6a unconditional bound-by-body check (`captured_len <= body.len()`) → E-INP-008 on failure (live reachable guard); PC6b padding-aware overhead check (`20 + captured_len + pad_len(captured_len) <= body.len()`) → E-INP-008 on failure (defense-in-depth; unreachable on a crate-framed 4-aligned block — crate alignment rejection subsumes this path); both precede any allocation; returns Err for all invalid inputs. Decision 20: crate-framing rejection (btl<12/misaligned/EOF) → E-INP-010; ALL wirerust body-decode failures (body-too-short, PC6a bound-by-body, PC6b padding-overrun) → E-INP-008. ADR-009 rev 9: VP asserts the discriminant identity, not merely that an Err is returned. Note: Kani proof for the PC6b (padded-overrun) branch requires a synthetic non-4-aligned body bypassing the crate gate — the proof harness must inject this directly as it cannot be reached via normal crate block delivery. | Kani: `#[kani::proof]` over EPB byte sequences with symbolic interface_id, captured_len, and body length; includes empty-table (table.len()==0, any interface_id → assert E-INP-009), OOB-non-empty (table.len()==1, interface_id>=1 → assert E-INP-010), body-length=19 (EC-011 → assert E-INP-008), and padded-overrun (EC-010 → assert E-INP-008, injected via synthetic non-aligned body) cases |
| — | `captured_len` is always used for data slice, never `original_len` | unit: EPB with captured < original; assert data.len() == captured |
| — | Packet order preserved across multiple EPBs | unit: 3-EPB file; assert order matches |
| — | Raw split ticks routed to BC-2.01.014 (not crate Duration) | unit: EPB with `if_tsresol=6` known-µs ticks; assert timestamp 1000× correct (regression guard for crate's ns-hardcode bug) |
| — | N-packet encounter order + byte fidelity (PC8 / M-5 completeness) | integration: `arp-baseline-16pkt.cap` fixture; assert `packets.len()==16`, encounter order matches EPB sequence, each `data` byte-identical to known-good extraction **Test:** `test_BC_2_01_012_happy_path_n_packet_order_and_byte_fidelity` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- EPB parsing is the primary packet-extraction path for pcapng; the `Vec<RawPacket>` produced by EPB parsing is the output artifact of CAP-01 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-125 |
| ADR Reference | ADR-009 rev 4 Decision 1 (raw-block path), Decision 2 (EPB coverage), Decision 4 (64-bit timestamp normalization via pure-core helper), Decision 8 (forward-progress), Decision 10 (panic surface) |

## Related BCs

- BC-2.01.011 -- depends on (interface table populated by IDB parsing; EPB uses interface_id to look up if_tsresol)
- BC-2.01.014 -- composes with (raw split ticks passed to timestamp conversion helper)
- BC-2.01.002 -- mirrors (classic-pcap analog; same RawPacket output type)

## Architecture Anchors

- ADR-009 rev 4 Decision 1: raw-block path (`RawBlock` / `next_raw_block`); EPB fixed fields read from raw body: interface_id:4, ts_high:4, ts_low:4, captured_len:4, original_len:4
- ADR-009 rev 4 Decision 4: `EPB_FIXED_OVERHEAD_BYTES = 20` (body-relative); superseded by ADR-009 rev 7/8 Decision 20 padding-aware validation: (1) unconditional `captured_len <= body.len()` → E-INP-008 on failure; (2) `20 + captured_len + pad_len(captured_len) <= body.len()` where `pad_len(n) = (4 - n%4) % 4` → E-INP-008 on failure; body < 20 bytes → E-INP-008; ALL wirerust body-decode failures → E-INP-008 per ADR-009 rev 8 (C-1)
- ADR-009 rev 7 Decision 20 / rev 8 C-1 reclassification: EPB body-decode taxonomy — `12 <= btl < 32` → body < 20 → wirerust body-decode failure → E-INP-008 (NOT E-INP-010); `captured_len > body.len() - 20` (bound-by-body) → wirerust body-decode failure → E-INP-008; `20 + captured_len + pad_len(captured_len) > body.len()` (padding-overrun) → wirerust body-decode failure → E-INP-008; `btl < 12` or trailing-length mismatch → crate Err → E-INP-010; EPB interface_id OOB on non-empty table → E-INP-010. E-INP-010 in this BC is STRICTLY crate-framing rejection + interface_id OOB; all wirerust body-decode failures are E-INP-008
- `enhanced_packet.rs:46-48,65` (pcap-file 2.0.0 source): `Duration::from_nanos` hard-codes ns, never applies `if_tsresol` — confirms wirerust MUST NOT use `EnhancedPacketBlock::timestamp`
- pcapng spec IETF draft §Enhanced-Packet-Block: fixed-fields layout; captured vs. original length semantics

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads block bytes from stream (raw-block path) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O during block reading); timestamp sub-computation is pure-core (BC-2.01.014) |
