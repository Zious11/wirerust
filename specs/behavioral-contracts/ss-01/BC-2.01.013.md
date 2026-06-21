---
document_type: behavioral-contract
level: L3
version: "1.10"
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
  - "v1.6: Pass-6 remediation T1 — ADR-009 rev 9 Decision 22: resolved contradictory body-bound definitions. On the raw path RawBlock.body = [original_len:4][padded data], so the available data bytes after original_len = spb_data_available = body.len() - 4 (NOT body.len(), which is 4 bytes too large). Canonical formula everywhere: captured_len = min(original_len, spb_data_available) = min(original_len, body.len() - 4). Deleted all 'equivalently body.len()' / 'body.len() as the bound' text. Updated: Description (define spb_data_available symbol), PC1 (use spb_data_available), Invariant-2 (canonical symbol definition), AC-002 (min(original_len, body.len()-4)), EC-007 (captured_len = body.len()-4 when original_len > spb_data_available), VP-031 row (property min(original_len, body.len()-4); domain starts where body.len()>=4, else E-INP-008 body-too-short), Architecture Anchors (remove body.len() bare equivalences). — 2026-06-20"
  - "v1.7: Pass-7 remediation per ADR-009 rev 9 (F-7 symbol rename; F-8 misaligned fixture) — (F-7) Renamed retired symbol `block_body_available` → `spb_data_available` in EC-001, EC-002, EC-003, and Canonical Test Vectors; each value confirmed as min(original_len, body.len()-4). Also updated stale `block_body_available` reference in Precondition 4. (F-8) Expanded EC-005 to enumerate both crate-rejection sub-cases: (a) btl < 12 (e.g., btl=8) → crate rejects → E-INP-010; (b) btl=14 (>=12 but 14%4!=0) → crate rejects for 4-byte alignment → E-INP-010. Added canonical test vector for the btl=14 misaligned case so the BC's enumerated cases match HS-107 Case E and make clear that E-INP-010 fires for misalignment (not only for btl<12). No residual `block_body_available` occurrences remain in EC-001–EC-003 or Canonical Test Vectors. — 2026-06-20"
  - "v1.8: Pass-8 M-3 remediation (DF-AC-TEST-NAME-SYNC-001) — AC-001 test name renamed: `test_BC_2_01_013_snaplen_lookup_guarded` → `test_BC_2_01_013_empty_interface_table_guarded`. Snaplen was removed from the SPB path (ADR-009 rev 8 Decision 9 amendment / rev 9 F-M3); the old name was stale. AC-001 scope note clarified: the guard solely prevents an unchecked index on an empty interface table (E-INP-009 path); the body-too-short error path (btl=12 → body=0 < 4 → E-INP-008) is handled distinctly by AC-004a/EC-008, so these ACs are non-redundant. No normative behavior change. — 2026-06-20"
  - "v1.10: F5 adversarial O-2 adjudication (documentation-only) — added accepted-behavior note to Postcondition 5 documenting that the SPB decode path checks the empty-interface-table guard (E-INP-009) BEFORE the body-too-short guard (E-INP-008), which is the reverse of the EPB five-step precedence mandated by BC-2.01.012 PC9. The asymmetry is accepted: SPB has no interface_id body field, so the empty-table check does not depend on reading any body bytes and evaluating it first is semantically coherent. For the single constructible overlap case (btl=12, body=0, empty table) the implementation yields E-INP-009; EPB-aligned ordering would yield E-INP-008. Both correctly reject the block with no silent pass-through. BC-2.01.013 imposes no ordering constraint between these two guards; this is documentation-only — no normative change. Cross-reference: BC-2.01.012 PC9, finding O-2, adjudication F-F5P1-003-O2-adjudication.md. — 2026-06-21"
  - "v1.9: Pass-9 LOW-1 remediation — resolved sibling asymmetry with BC-2.01.012. Precondition 3, Postcondition 5, and AC-001 now mandate the exact SPB E-INP-009 message string: 'SPB encountered but interface table is empty — no IDB has been parsed' (mirrors EPB PC5a form from BC-2.01.012). The prior text said only 'Err mapping to E-INP-009' with no string, leaving the SPB message unconstrained (HS-107 Case D accepted 'or similar'). The mandated string is also registered in error-taxonomy.md E-INP-009 SPB row (v3.6). No other normative behavior change. — 2026-06-20"
  - "v1.5: Pass-5 remediation S2 — ADR-009 rev 8 Decision 9 amendment: snaplen DROPPED from SPB captured_len. Decision 9 states snaplen is NOT enforced for SPB (same as EPB). captured_len now = min(original_len, block_body_available) everywhere — block_body_available = body.len() is the authoritative on-disk bound. Removed snaplen from: Description, PC1, AC-002, EC-007, EC-001, Invariant 2, Canonical Test Vectors, Architecture Anchors. VP-031 updated: captured_len == min(original_len, body.len() as u32). EC-007 'snaplen wins' case restated: original_len > block_body_available → data clamped to block_body_available. HS-107 VP row description corrected to match HS-107 actual scope (SPB framing truncation/padding/no-IDB, incl. Case F btl=12→E-INP-008). Removed 4x stale '(HS-107 btl=12→E-INP-008 holdout deferred to a separate burst.)' notes — HS-107 Case F now covers it. — 2026-06-20"
  - "v1.4: Pass-4 remediation R2 — Decision 20: added body-too-short E-INP-008 case for SPB: btl=12 (aligned, >=12, crate frames and returns block) → body=0 bytes < 4 SPB fixed-field bytes (original_len:u32) → wirerust body-decode → E-INP-008. Distinguishes from btl<12/misaligned/EOF → crate Err → E-INP-010. Updated Postcondition 6, added EC-008, added AC-004a body-truncation test, updated Canonical Test Vectors and Traceability. M-1: removed 'crate enforces body minimum' over-claim from Architecture Anchors — wirerust checks body.len()>=4 itself on the raw path before decoding SPB fixed fields. Authority: ADR-009 rev 7 Decision 20, per-block fixed-field minimum SPB=4. — 2026-06-20"
  - "v1.1: F2 Burst-A remediation per ADR-009 rev 4 PO dispatch — (1) Corrected SPB body-relative fixed overhead to 4 bytes (original_len: u32 only; H-2 fix — was incorrectly stated as 20 bytes in the Description and Postcondition 1). (2) Corrected minimum block_total_length to 16 bytes (12 outer + 4 body-fixed); available padded-data bytes = block_total_length - 16. (3) Added explicit note: RawBlock `data` includes padding — caller MUST compute captured_len = min(original_len, snaplen) and strip accordingly. (4) Added SPB-without-IDB case as E-INP-009 (empty interface table; do NOT index idb[0] unguarded — H-4). (5) Added no-panic AC (SEC-005). (6) Removed incorrect 'block_total_length - 20' formula from Postcondition 1 (20 was the EPB overhead, not SPB). — 2026-06-19"
  - "v1.2: Pass-2 remediation per ADR-009 rev 5 (I-4, I-11) — (I-4) EC-001 corrected: data bound changed from min(original_len, block_body_available) to min(original_len, snaplen, block_body_available) — consistent with PC1/Invariant-2 which both specify the three-way minimum. (I-11) Added Test: citations to all ACs. Added HS-107 holdout reference in Verification Properties. — 2026-06-19"
  - "v1.3: Pass-3 remediation per ADR-009 rev 6 (C-1+H-4, M-2, M-1) — (C-1+H-4 CRITICAL) PC1 and AC-002 were still using the two-way min(original_len, snaplen) despite v1.2 changelog claiming otherwise; both rewritten to the three-way min(original_len, snaplen, block_body_available) where block_body_available = block_total_length - 16. Description and Architecture Anchors two-way citations also corrected to three-way. The data slice is now stated as bounded by actual body length UNCONDITIONALLY first (preventing any OOB slice). Invariant-2 split form (min + 'further by') unified to a single three-way expression. EC-007 wire-impossible phrasing removed: when original_len>snaplen the body holds at most snaplen captured bytes, not full original_len data; EC-007 rewritten accordingly. (M-2) VP-031 (proptest, framing-arithmetic) added to Verification Properties: property forall (original_len, snaplen, body) captured_len == min(original_len, snaplen, body.len()) and slice is exactly captured_len bytes with no OOB access, Phase P1. (M-1) HS-107 path corrected from .factory/specs/holdout-scenarios/ (nonexistent) to .factory/holdout-scenarios/HS-107-pcapng-spb-framing-truncation-padding-and-no-idb.md. — 2026-06-19"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.013: Parse pcapng Simple Packet Block (SPB): Packet Data Without Timestamp

## Description

The Simple Packet Block (SPB, block type `0x00000003`) is a compact packet container that
carries raw packet data and an `original_len: u32` field but no per-packet timestamp, no
interface ID, and no options. `SPB_FIXED_OVERHEAD_BYTES = 4` (body-relative: `original_len`
field only; the outer 12-byte block header is separate). On the raw-block path,
`RawBlock.body = [original_len:4][padded data]`, so the crate exposes `data: Cow<[u8]>`
which is the ENTIRE remaining block body after the 4-byte `original_len` field — this slice
INCLUDES padding bytes; the crate performs NO `captured_len` computation. The available
padded-data bytes after `original_len` are defined as the canonical symbol
`spb_data_available = body.len() - 4` (NOT `body.len()` alone, which is 4 bytes too large
because it counts the `original_len` field itself). Per ADR-009 rev 8 Decision 9 amendment,
snaplen is NOT enforced for SPB (same policy as EPB). The caller MUST compute
`captured_len = min(original_len, spb_data_available)` and strip the padding accordingly.
The data slice MUST be bounded by `spb_data_available` unconditionally so that a malformed
SPB with `original_len` exceeding `spb_data_available` cannot produce an out-of-bounds slice.
SPBs are rare in practice (Wireshark does not emit them) but are legal per the pcapng
specification. Timestamp fields on `RawPacket` are always set to zero for SPBs.

## Preconditions

1. The SHB has been parsed; byte order is established.
2. The block type reads `0x00000003`.
3. The interface table is checked before accessing `idb[0]`; if the table is empty, the
   call returns `Err` mapping to E-INP-009 (SPB-without-IDB). The error message MUST be:
   `"SPB encountered but interface table is empty — no IDB has been parsed"`.
4. The crate requires `block_total_length >= 12` to return a block; blocks with btl < 12 / misaligned / EOF are rejected by the crate with E-INP-010 before wirerust receives them. A btl=12 block (body=0 bytes) is returned by the crate but wirerust body-decode will find insufficient bytes for the `original_len: u32` field and return E-INP-008. The minimum legal SPB carrying any data has `block_total_length = 16` (12 outer + 4 body-fixed for `original_len` + 0 padded data; btl=16 → body=4 → exactly 4 bytes available for `original_len` → parse succeeds with `spb_data_available = 0`).

## Postconditions

1. On the raw-block path `RawBlock.body = [original_len:4][padded data]`. The raw `data`
   slice from `RawBlock` is the block body after `original_len` (4 bytes), padded to a
   4-byte boundary. The canonical symbol `spb_data_available = body.len() - 4` is the
   available padded-data bytes after the `original_len` field (equivalently,
   `block_total_length - 16`). Note: `body.len()` alone (= `block_total_length - 12`) is 4
   bytes too large and MUST NOT be used as the data bound.
   `captured_len = min(original_len, spb_data_available)`.
   Snaplen is NOT applied for SPB (ADR-009 rev 8 Decision 9 amendment; same policy as EPB).
   The data slice MUST be bounded by `spb_data_available` unconditionally (so no
   slice can ever exceed the actual padded data region).
   The data slice MUST be truncated to exactly `captured_len` bytes (stripping padding).
2. `original_len` is noted but NOT used to extend the data slice beyond the
   padded block body (a malformed file could claim `original_len` larger than available
   block data; the padded block body is the authoritative bound).
3. A `RawPacket` is produced with `timestamp_secs = 0` and `timestamp_usecs = 0`.
4. The `RawPacket` is appended to `PcapSource.packets` in block-encounter order.
5. An SPB encountered when the interface table is EMPTY (no IDB has been seen) returns `Err`
   mapping to E-INP-009. The caller MUST guard the `idb[0]` access; an unchecked index on an
   empty table is NOT permitted (H-4 fix). The error message MUST be:
   `"SPB encountered but interface table is empty — no IDB has been parsed"`.

   **PC5 — Accepted-behavior note (O-2 / F-F5P1-003-O2-adjudication.md):** The SPB decode
   path checks the empty-interface-table guard (PC5 / E-INP-009) BEFORE the body-too-short
   guard (PC6 / E-INP-008). This is the REVERSE of the five-step EPB evaluation order
   mandated by BC-2.01.012 PC9. The asymmetry is **accepted and correct**: SPB has no
   `interface_id` body field — it always uses interface 0 — so the empty-table check does not
   depend on reading any body bytes, and evaluating it first is semantically coherent. For the
   single constructible overlap case (btl=12, body=0, empty table), the implementation yields
   E-INP-009; EPB-aligned ordering (body-len check first) would yield E-INP-008. Both
   correctly reject the block with no silent pass-through. BC-2.01.013 imposes no ordering
   constraint between these two guards. This is documentation-only — no normative constraint
   change. Cross-reference: BC-2.01.012 PC9 (EPB five-step precedence); finding O-2.

6. **SPB error routing — uniform split (ADR-009 rev 7 Decision 20).**
   - **btl < 12 / btl % 4 != 0 / EOF** — crate rejects before returning any block →
     **E-INP-010**. wirerust never sees the body.
   - **btl = 12 (aligned, ≥12; crate frames and returns block)** — body = 0 bytes (12−12=0),
     which is < 4 SPB fixed-field bytes (`original_len: u32`) → wirerust body-decode finds
     insufficient bytes → **E-INP-008**. This is the constructible body-too-short window for
     SPB: btl must be exactly 12 for body=0; btl=16 → body=4 (exactly sufficient; no error from
     body-decode alone). Per-block fixed-field minimum for SPB = 4 bytes (ADR-009 rev 7).
     Canonical fixture: btl=12 → body=0 bytes < 4 → E-INP-008. Covered by HS-107 Case F.

## Acceptance Criteria

- **AC-001 (idb[0] existence guard):** wirerust MUST check that the interface table is
  non-empty before processing an SPB. If the interface table is empty, return `Err` mapping
  to E-INP-009 rather than indexing an empty Vec. The error message MUST be exactly:
  `"SPB encountered but interface table is empty — no IDB has been parsed"`.
  Snaplen from `idb[0]` is NOT used in the SPB `captured_len` computation (ADR-009 rev 8
  Decision 9 amendment); this guard solely prevents an unchecked index on an empty table
  (the structural precondition for E-INP-009). **Scope note:** this AC covers only the
  empty-table index guard (EC-006). The body-too-short error path (btl=12 → body=0 < 4 bytes
  → E-INP-008) is a distinct concern handled by AC-004a and EC-008; these two ACs are
  non-redundant.
  **Test:** `test_BC_2_01_013_empty_interface_table_guarded`
- **AC-002 (padding strip):** The raw `data` slice from the crate INCLUDES padding bytes to
  the 4-byte boundary. On the raw path `RawBlock.body = [original_len:4][padded data]`, so
  the available data bytes after `original_len` = `spb_data_available = body.len() - 4`.
  wirerust MUST compute `captured_len = min(original_len, body.len() - 4)` and slice to
  exactly `captured_len` bytes before populating `RawPacket.data`. The bound `body.len()`
  alone (4 bytes too large) MUST NOT be used. Snaplen is NOT applied for SPB (ADR-009 rev 8
  Decision 9 amendment). The slice MUST be bounded by `spb_data_available` unconditionally
  so that a malformed SPB where `original_len` exceeds the actual padded-data region cannot
  produce an out-of-bounds slice. Handing the padded or unbounded slice to downstream
  decoders verbatim is prohibited.
  **Test:** `test_BC_2_01_013_padding_strip`
- **AC-003 (no-panic, SEC-005):** This block parser MUST return `Err` for any malformed or
  truncated input; `unwrap()`, `expect()`, and `panic!()` are prohibited in the SPB parse path.
  **Test:** `test_BC_2_01_013_no_panic_malformed`
- **AC-004a (Body-too-short — E-INP-008):** An SPB where the crate returns a valid-framed
  `RawBlock` (btl ≥ 12, btl % 4 == 0) but the body is fewer than 4 bytes (insufficient for
  `original_len: u32`) causes wirerust body-decode to return `Err` mapped to **E-INP-008**.
  wirerust checks `body.len() >= 4` itself on the raw path before decoding SPB fixed fields —
  the crate does NOT enforce this minimum for the caller. Constructible fixture: btl=12 →
  body=0 bytes < 4 → E-INP-008. Covered by HS-107 Case F.
  **Test:** `test_BC_2_01_013_spb_body_truncated_e_inp_008`
- **AC-004b (SPB_FIXED_OVERHEAD_BYTES = 4):** The named constant `SPB_FIXED_OVERHEAD_BYTES`
  MUST equal 4 (body-relative; `original_len: u32` only). This constant MUST NOT be confused
  with `EPB_FIXED_OVERHEAD_BYTES = 20`.
  **Test:** `test_BC_2_01_013_fixed_overhead_constant`

## Invariants

1. SPB timestamps are always zero — there is no per-packet timestamp in the SPB format.
   Downstream consumers (reassembly, findings timestamp) receive zero-timestamps for SPBs.
2. Packet data is bounded by `min(original_len, spb_data_available)` where
   `spb_data_available = body.len() - 4` (the canonical symbol for available padded-data
   bytes after the `original_len` field; equivalently `block_total_length - 16`).
   The bare `body.len()` (= `block_total_length - 12`) is 4 bytes too large and MUST NOT be
   used as the data bound. No out-of-bounds read is possible. Snaplen is NOT applied for SPB
   (ADR-009 rev 8 Decision 9 amendment). The `spb_data_available` bound is applied
   unconditionally.
3. SPB parsing shares the same `RawPacket` output type as EPB and classic-pcap parsing.
4. `SPB_FIXED_OVERHEAD_BYTES = 4` (body-relative: `original_len: u32` only). The minimum
   SPB `block_total_length` is 16 bytes (12 outer + 4 body-fixed).
5. The pcapng specification requires that a file using SPBs must have exactly one IDB;
   wirerust enforces this by checking that the interface table is non-empty before accessing
   `idb[0]`, with an explicit guard for the empty-table case (E-INP-009). Snaplen from
   `idb[0]` is NOT used in the SPB `captured_len` computation (ADR-009 rev 8 Decision 9
   amendment); the guard only prevents an unchecked index on an empty table.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SPB with `original_len > block body data` (truncated on disk) | Data slice bounded to `min(original_len, spb_data_available)` bytes where `spb_data_available = body.len() - 4`; `RawPacket.data.len() == spb_data_available < original_len` |
| EC-002 | SPB where `original_len` exactly matches `spb_data_available` (= `body.len() - 4`) | Data sliced to `captured_len = original_len = spb_data_available`; no truncation |
| EC-003 | SPB in file with multiple IDBs (spec violation) | Guard only checks `idb.is_empty()`; if non-empty, table access succeeds (snaplen is not used for SPB captured_len; `spb_data_available` bound applied unconditionally); no panic; proceeds |
| EC-004 | SPB with zero-byte data section (`original_len = 0`) | `RawPacket { data: vec![] }` produced |
| EC-005 | SPB where the crate rejects the block before returning it to wirerust (two distinct sub-cases, both → E-INP-010): **(a) btl < 12** (e.g., btl=8 — below the outer-header minimum; crate rejects before returning block; wirerust never sees the body → E-INP-010); **(b) btl misaligned** (e.g., btl=14 — while 14 >= 12, the pcapng specification requires all block_total_length values to be a multiple of 4; 14 % 4 != 0 violates 4-byte alignment; crate rejects before returning block; wirerust never sees the body → E-INP-010). Both sub-cases are crate-level framing failures; E-INP-010 fires for alignment as well as for btl<12. Distinct from EC-008 (btl=12 → body=0 < 4 → wirerust body-decode → E-INP-008). HS-107 Case E exercises the btl=14 misaligned sub-case. |
| EC-006 | SPB encountered before any IDB (empty interface table) | `Err` mapping to E-INP-009 (guard fires before any idb[0] access) |
| EC-007 | `original_len` > `spb_data_available` (on-disk file shorter than original_len indicates truncation or intentional capture limit by the writing tool; `spb_data_available = body.len() - 4`) | `captured_len = min(original_len, body.len() - 4) = body.len() - 4`; data sliced to `spb_data_available` bytes; snaplen is NOT applied (ADR-009 rev 8 Decision 9 amendment) |
| EC-008 | SPB with btl=12 (aligned, crate frames and returns block; body=0 bytes < 4 SPB fixed-field bytes for original_len:u32) | `Err` mapping to **E-INP-008** (body-too-short; wirerust body-decode checks body.len()>=4 itself). No panic. Constructible window for SPB body-too-short: btl=12 only (body=0). btl<12 would be E-INP-010. Covered by HS-107 Case F. **Test:** `test_BC_2_01_013_spb_body_truncated_e_inp_008` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SPB with 64 bytes of Ethernet frame data, `original_len=64`, `spb_data_available=64` (body.len()-4=64) | `RawPacket { timestamp_secs: 0, timestamp_usecs: 0, data.len(): 64 }` | happy-path |
| SPB with `original_len=1500`, block body 64 padded bytes (`spb_data_available = body.len()-4 = 64`) | `data.len() == 64` (`min(1500, 64) = 64`; bounded by spb_data_available; snaplen not applied) | edge-case |
| SPB with `original_len=100`, block body 100 bytes (`spb_data_available = body.len()-4 = 100`) | `data.len() == 100` (`min(100, 100) = 100`; no truncation needed) | edge-case |
| SPB before any IDB (empty interface table) | `Err` (E-INP-009) | error |
| SPB with btl=12 (crate returns block; body=0 bytes < 4 SPB fixed fields for original_len) | `Err` (E-INP-008); no panic | error (body-too-short; wirerust body-decode path) |
| SPB with btl=8 (btl < 12 — crate rejects before returning block) | `Err` (E-INP-010) | error (crate-rejection path; EC-005a) |
| SPB with btl=14 (btl >= 12 but 14 % 4 != 0 — crate rejects for 4-byte alignment) | `Err` (E-INP-010) | error (crate-rejection path; EC-005b; HS-107 Case E) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | SPB always produces timestamp (0, 0) | unit: parse SPB; assert timestamp_secs=0, timestamp_usecs=0 |
| — | SPB data length bounded by min(original_len, spb_data_available) where spb_data_available = body.len()-4; snaplen not applied | unit: SPB with original_len > block body; assert data.len() == spb_data_available bytes |
| — | SPB-without-IDB returns E-INP-009, not panic | unit: SPB with empty interface table; assert Err(E-INP-009); no panic |
| — | SPB padding stripped before RawPacket | unit: SPB with original_len not 4-byte aligned; assert data.len() == original_len (not padded length) |
| — | Covered under VP-028 (cargo-fuzz) for full no-panic coverage | fuzz: fuzz SPB bytes, assert no panic (F6 hardening deliverable) |
| VP-031 | For all (original_len: u32, body: &[u8]) where `body.len() >= 4` (bodies shorter than 4 bytes are the E-INP-008 body-too-short path, not the framing-arithmetic path): `captured_len == min(original_len, (body.len() - 4) as u32)` and the returned data slice has EXACTLY `captured_len` bytes with no out-of-bounds access. The bound `body.len()` alone (4 too large, counts the `original_len` field) is excluded. Snaplen is excluded from the pure-core helper domain (ADR-009 rev 8 Decision 9 amendment). | proptest: generate arbitrary (original_len, body) pairs with body.len()>=4; assert framing arithmetic `min(original_len, body.len()-4)` and exact slice length; Phase P1 |
| HS-107 | SPB holdout scenario: exercises SPB framing truncation, padding strip, no-IDB guard (E-INP-009), minimum-length crate rejection (btl=14 → E-INP-010), and body-too-short (btl=12 → E-INP-008, Case F). Six crafted pcapng fixtures (Cases A–F). | holdout evaluation (Phase 4); see `.factory/holdout-scenarios/HS-107-pcapng-spb-framing-truncation-padding-and-no-idb.md` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- SPB parsing is an alternative packet-extraction path within pcapng ingestion; its `RawPacket` output is the same artifact as EPB and classic-pcap parsing under CAP-01 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-126 |
| ADR Reference | ADR-009 rev 9 Decision 22 (canonical spb_data_available = body.len()-4; captured_len = min(original_len, body.len()-4); bare body.len() bound is WRONG — 4 bytes too large), ADR-009 rev 8 Decision 2 (SPB block coverage), Decision 9 amendment (snaplen NOT enforced for SPB — same as EPB; captured_len = min(original_len, spb_data_available) only), Decision 10 (panic surface), Decision 20 (uniform error-code rule: btl<12/misaligned→E-INP-010; btl=12→body=0<4→E-INP-008 via wirerust body-decode; wirerust checks body.len()>=4 itself; per-block fixed-field minimum SPB=4; covered by HS-107 Case F) |

## Related BCs

- BC-2.01.011 -- depends on (SPB checks interface table is non-empty before accessing idb[0]; empty-table guard prevents H-4 panic; snaplen from idb[0] is NOT used in SPB captured_len per ADR-009 rev 8 Decision 9 amendment)
- BC-2.01.012 -- sibling (EPB is the timestamp-bearing alternative to SPB; same RawPacket output)
- BC-2.01.015 -- related to (unknown blocks are skipped; SPB is a known block that must be parsed)

## Architecture Anchors

- ADR-009 rev 9 Decision 22 + rev 8 Decision 2: `SPB_FIXED_OVERHEAD_BYTES = 4` (body-relative: `original_len: u32` only); minimum valid SPB `block_total_length = 16` (btl=12 is crate-framable but body=0 < 4 → wirerust E-INP-008). On the raw path `RawBlock.body = [original_len:4][padded data]`; the canonical symbol `spb_data_available = body.len() - 4` (NOT `body.len()` alone, which is 4 bytes too large). Caller derives `captured_len = min(original_len, spb_data_available) = min(original_len, body.len() - 4)`. Snaplen NOT applied for SPB (Decision 9 amendment); `spb_data_available` bound applied unconditionally; strips padding to exactly `captured_len` bytes. Decision 20: uniform error-code rule for SPB: btl<12/misaligned→E-INP-010 (crate rejects); btl=12→body=0<4→E-INP-008 (wirerust body-decode; wirerust checks body.len()>=4 itself; VP-031 domain starts at body.len()>=4; covered by HS-107 Case F).
- `simple_packet.rs:19-37` (pcap-file 2.0.0 source): `data: Cow<[u8]>` includes padding; no `captured_len` field; no snaplen clamp in crate. **Note (M-1):** the crate does NOT enforce a body minimum for the caller on the raw path — wirerust checks `body.len() >= 4` itself before decoding `original_len`.
- pcapng spec IETF draft §Simple-Packet-Block: `original_len` field only; on-disk payload is padded to 4-byte boundary

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads block bytes from stream |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O during block reading) |
