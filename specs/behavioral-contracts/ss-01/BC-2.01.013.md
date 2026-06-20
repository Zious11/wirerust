---
document_type: behavioral-contract
level: L3
version: "1.5"
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
field only; the outer 12-byte block header is separate). On the raw-block path, the crate
exposes `data: Cow<[u8]>` which is the ENTIRE remaining block body after the 4-byte
`original_len` field — this slice INCLUDES padding bytes; the crate performs NO
`captured_len` computation. Per ADR-009 rev 8 Decision 9 amendment, snaplen is NOT
enforced for SPB (same policy as EPB). The caller MUST compute
`captured_len = min(original_len, block_body_available)` where
`block_body_available = block_total_length - 16` (equivalently, `body.len()`), and strip
the padding accordingly. The data slice MUST be bounded by the actual body length
unconditionally so that a malformed SPB with original_len exceeding body.len() cannot
produce an out-of-bounds slice.
SPBs are rare in practice (Wireshark does not emit them) but are legal per the pcapng
specification. Timestamp fields on `RawPacket` are always set to zero for SPBs.

## Preconditions

1. The SHB has been parsed; byte order is established.
2. The block type reads `0x00000003`.
3. The interface table is checked before accessing `idb[0]`; if the table is empty, the
   call returns `Err` mapping to E-INP-009 (SPB-without-IDB).
4. The crate requires `block_total_length >= 12` to return a block; blocks with btl < 12 / misaligned / EOF are rejected by the crate with E-INP-010 before wirerust receives them. A btl=12 block (body=0 bytes) is returned by the crate but wirerust body-decode will find insufficient bytes for the `original_len: u32` field and return E-INP-008. The minimum legal SPB carrying any data has `block_total_length = 16` (12 outer + 4 body-fixed for `original_len` + 0 padded data; btl=16 → body=4 → exactly 4 bytes available for `original_len` → parse succeeds with `block_body_available = 0`).

## Postconditions

1. The raw `data` slice from `RawBlock` is the block body after `original_len` (4 bytes),
   padded to a 4-byte boundary. The available padded-data bytes (`block_body_available`) =
   `block_total_length - 16` (12-byte outer header + 4-byte `original_len` field);
   equivalently, `body.len()` is the authoritative on-disk bound.
   `captured_len = min(original_len, block_body_available)`.
   Snaplen is NOT applied for SPB (ADR-009 rev 8 Decision 9 amendment; same policy as EPB).
   The data slice MUST be bounded by `block_body_available` unconditionally (so no
   slice can ever exceed `body.len()`).
   The data slice MUST be truncated to exactly `captured_len` bytes (stripping padding).
2. `original_len` is noted but NOT used to extend the data slice beyond the
   padded block body (a malformed file could claim `original_len` larger than available
   block data; the padded block body is the authoritative bound).
3. A `RawPacket` is produced with `timestamp_secs = 0` and `timestamp_usecs = 0`.
4. The `RawPacket` is appended to `PcapSource.packets` in block-encounter order.
5. An SPB encountered when the interface table is EMPTY (no IDB has been seen) returns `Err`
   mapping to E-INP-009. The caller MUST guard the `idb[0]` access; an unchecked index on an
   empty table is NOT permitted (H-4 fix).
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
  to E-INP-009 rather than indexing an empty Vec. Snaplen from `idb[0]` is NOT used in
  the SPB `captured_len` computation (ADR-009 rev 8 Decision 9 amendment); this guard
  solely prevents an unchecked index on an empty table.
  **Test:** `test_BC_2_01_013_snaplen_lookup_guarded`
- **AC-002 (padding strip):** The raw `data` slice from the crate INCLUDES padding bytes to
  the 4-byte boundary. wirerust MUST compute
  `captured_len = min(original_len, block_body_available)` where
  `block_body_available = block_total_length - 16` (equivalently, `body.len()`), and slice
  to exactly `captured_len` bytes before populating `RawPacket.data`. Snaplen is NOT applied
  for SPB (ADR-009 rev 8 Decision 9 amendment). The slice MUST be bounded by
  `block_body_available` unconditionally so that a malformed SPB where original_len exceeds
  the actual body cannot produce an out-of-bounds slice. Handing the padded or unbounded
  slice to downstream decoders verbatim is prohibited.
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
2. Packet data is bounded by `min(original_len, block_body_available)` where
   `block_body_available = block_total_length - 16` (equivalently, `body.len()`);
   no out-of-bounds read is possible. Snaplen is NOT applied for SPB
   (ADR-009 rev 8 Decision 9 amendment). The body-available bound is applied unconditionally.
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
| EC-001 | SPB with `original_len > block body data` (truncated on disk) | Data slice bounded to `min(original_len, block_body_available)` bytes; `RawPacket.data.len() == block_body_available < original_len` |
| EC-002 | SPB where `original_len` exactly matches `block_body_available` | Data sliced to `captured_len = original_len = block_body_available`; no truncation |
| EC-003 | SPB in file with multiple IDBs (spec violation) | Guard only checks `idb.is_empty()`; if non-empty, table access succeeds (snaplen is not used for SPB captured_len); no panic; proceeds |
| EC-004 | SPB with zero-byte data section (`original_len = 0`) | `RawPacket { data: vec![] }` produced |
| EC-005 | SPB where btl < 12 (e.g., btl=8 — crate rejects before returning block; wirerust never sees the body) | `Err` mapping to **E-INP-010** (crate-rejection path; distinct from EC-008 which is the wirerust body-decode path). Note: btl=12 (body=0 < 4) is EC-008 → E-INP-008, not E-INP-010. |
| EC-006 | SPB encountered before any IDB (empty interface table) | `Err` mapping to E-INP-009 (guard fires before any idb[0] access) |
| EC-007 | `original_len` > `block_body_available` (on-disk file shorter than original_len indicates truncation or intentional capture limit by the writing tool) | `captured_len = min(original_len, block_body_available) = block_body_available`; data sliced to `block_body_available` bytes; snaplen is NOT applied (ADR-009 rev 8 Decision 9 amendment) |
| EC-008 | SPB with btl=12 (aligned, crate frames and returns block; body=0 bytes < 4 SPB fixed-field bytes for original_len:u32) | `Err` mapping to **E-INP-008** (body-too-short; wirerust body-decode checks body.len()>=4 itself). No panic. Constructible window for SPB body-too-short: btl=12 only (body=0). btl<12 would be E-INP-010. Covered by HS-107 Case F. **Test:** `test_BC_2_01_013_spb_body_truncated_e_inp_008` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SPB with 64 bytes of Ethernet frame data, `original_len=64`, `block_body_available=64` | `RawPacket { timestamp_secs: 0, timestamp_usecs: 0, data.len(): 64 }` | happy-path |
| SPB with `original_len=1500`, block body 64 padded bytes (`block_body_available=64`) | `data.len() == 64` (`min(1500, 64) = 64`; bounded by on-disk body; snaplen not applied) | edge-case |
| SPB with `original_len=100`, block body 100 bytes (`block_body_available=100`) | `data.len() == 100` (`min(100, 100) = 100`; no truncation needed) | edge-case |
| SPB before any IDB (empty interface table) | `Err` (E-INP-009) | error |
| SPB with btl=12 (crate returns block; body=0 bytes < 4 SPB fixed fields for original_len) | `Err` (E-INP-008); no panic | error (body-too-short; wirerust body-decode path) |
| Truncated SPB (`block_total_length = 8`, btl < 12 — crate rejects before returning) | `Err` (E-INP-010) | error (crate-rejection path) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | SPB always produces timestamp (0, 0) | unit: parse SPB; assert timestamp_secs=0, timestamp_usecs=0 |
| — | SPB data length bounded by min(original_len, block_body_available); snaplen not applied | unit: SPB with original_len > block body; assert data.len() == block body available bytes |
| — | SPB-without-IDB returns E-INP-009, not panic | unit: SPB with empty interface table; assert Err(E-INP-009); no panic |
| — | SPB padding stripped before RawPacket | unit: SPB with original_len not 4-byte aligned; assert data.len() == original_len (not padded length) |
| — | Covered under VP-028 (cargo-fuzz) for full no-panic coverage | fuzz: fuzz SPB bytes, assert no panic (F6 hardening deliverable) |
| VP-031 | For all (original_len: u32, body: &[u8]): `captured_len == min(original_len, body.len() as u32)` and the returned data slice has EXACTLY `captured_len` bytes with no out-of-bounds access. Snaplen is excluded from the pure-core helper domain (ADR-009 rev 8 Decision 9 amendment). | proptest: generate arbitrary (original_len, body) pairs; assert framing arithmetic and slice length; Phase P1 |
| HS-107 | SPB holdout scenario: exercises SPB framing truncation, padding strip, no-IDB guard (E-INP-009), minimum-length crate rejection (btl=14 → E-INP-010), and body-too-short (btl=12 → E-INP-008, Case F). Six crafted pcapng fixtures (Cases A–F). | holdout evaluation (Phase 4); see `.factory/holdout-scenarios/HS-107-pcapng-spb-framing-truncation-padding-and-no-idb.md` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- SPB parsing is an alternative packet-extraction path within pcapng ingestion; its `RawPacket` output is the same artifact as EPB and classic-pcap parsing under CAP-01 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-126 |
| ADR Reference | ADR-009 rev 8 Decision 2 (SPB block coverage), Decision 9 amendment (snaplen NOT enforced for SPB — same as EPB; captured_len = min(original_len, block_body_available) only), Decision 10 (panic surface), Decision 20 (uniform error-code rule: btl<12/misaligned→E-INP-010; btl=12→body=0<4→E-INP-008 via wirerust body-decode; wirerust checks body.len()>=4 itself; per-block fixed-field minimum SPB=4; covered by HS-107 Case F) |

## Related BCs

- BC-2.01.011 -- depends on (SPB checks interface table is non-empty before accessing idb[0]; empty-table guard prevents H-4 panic; snaplen from idb[0] is NOT used in SPB captured_len per ADR-009 rev 8 Decision 9 amendment)
- BC-2.01.012 -- sibling (EPB is the timestamp-bearing alternative to SPB; same RawPacket output)
- BC-2.01.015 -- related to (unknown blocks are skipped; SPB is a known block that must be parsed)

## Architecture Anchors

- ADR-009 rev 8 Decision 2: `SPB_FIXED_OVERHEAD_BYTES = 4` (body-relative: `original_len: u32` only); minimum valid SPB `block_total_length = 16` (btl=12 is crate-framable but body=0 < 4 → wirerust E-INP-008); caller derives `captured_len = min(original_len, block_body_available)` where `block_body_available = block_total_length - 16` (equivalently `body.len()`); snaplen NOT applied for SPB (Decision 9 amendment); body-available bound applied unconditionally; strips padding to exactly `captured_len` bytes. Decision 20: uniform error-code rule for SPB: btl<12/misaligned→E-INP-010 (crate rejects); btl=12→body=0<4→E-INP-008 (wirerust body-decode; wirerust checks body.len()>=4 itself; covered by HS-107 Case F).
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
