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
  - "v1.5: Pass-4 remediation FINDING-P4-001 — GAP-1 closed: removed stale sentence from PC5 tail that claimed 'E-INP-008 covers SHB and IDB structural errors ONLY; EPB/SPB body truncation routes to E-INP-010'. Per Decision 20 the uniform rule is: crate-framed-but-body-too-short for ALL block types (SHB body<16, IDB body<8, EPB body<20, SPB body<4) → E-INP-008 (wirerust body-decode); btl<12/misaligned/EOF → E-INP-010 (crate framing rejection). No singling out of EPB/SPB as E-INP-010. Authority: ADR-009 rev 7 Decision 20. — 2026-06-20"
  - "v1.4: Pass-4 remediation R2 — Decision 20 (align wording to uniform rule): confirmed btl<12→E-INP-010 (crate rejects), 12<=btl<20→body<8→E-INP-008 (wirerust body-decode) as constructible window; wording in PC5 tightened. Decision 21 (M-2): REMOVED 'if_tsoffset (code 10)' from PC6 options-walk — wirerust does NOT extract if_tsoffset this cycle; added limitation note to PC6 and AC-003. M-1: removed 'crate enforces body>=8' over-claim from Architecture Anchors — wirerust checks body.len()>=8 itself on the raw path before decoding IDB fixed fields; the crate source reference clarified. L-2: EC-003 table cell fixed — unescaped pipe inside markdown table replaced with literal 0x8A (base-2 nanosecond-range if_tsresol value). Authority: ADR-009 rev 7 Decision 20, Decision 21. — 2026-06-20"
  - "v1.3: Pass-3 Q2 remediation — H-2: EC-008 constructible-window stated explicitly: the constructible E-INP-008 IDB body-truncation case is 12<=block_total_length<20 (crate frames the block; body is 0-7 bytes; wirerust body-decode finds <8 minimum bytes). btl<12 => crate Err => E-INP-010 (not constructible for IDB body-decode). H-5 (Decision 16): Invariant 2 'interface table resets at each SHB' deleted and deferred — unreachable under single-section constraint (Decision 7 rejects second SHB via E-INP-012 before any section-2 IDB is parsed); annotated DEFERRED. M-6: added IDB OPTIONS TLV-walking postcondition (PC6), AC-005 (bounds-check every option length against remaining body before reading value; opt_endofopt/end-of-body terminates walk; malformed option-length => E-INP-008; no panic/OOB), and EC-011 (option length > remaining body => E-INP-008). M-7 (Decision 17): added AC-006 (three-level IDB-parse precedence: (1) E-INP-013 position check FIRST; (2) E-INP-001 whitelist SECOND; (3) E-INP-011 conflict THIRD) and EC-012 (late IDB that also conflicts on linktype => E-INP-013 wins; E-INP-001/011 never evaluated). O-3: replaced 'E-INP-013 to be added in a separate burst' with 'E-INP-013: error-taxonomy.md v2.8'. Authority: ADR-009 rev 6. — 2026-06-19"
  - "v1.2: Pass-2 P2a remediation — C-1 (CRITICAL): PC4 and Architecture Anchor corrected: snaplen is at IDB body bytes 4-7, NOT 2-5. Spike Q-A3 confirms IDB wire layout: linktype u16 @0-1, reserved u16 @2-3, snaplen u32 @4-7 (interface_description.rs:45-52). Added: crate enforces reserved==0 and body>=8; wirerust mirrors these checks (non-zero reserved or body<8 is a structural IDB error => E-INP-008). I-7 / PC5 corrected: E-INP-008 covers SHB/IDB structural errors ONLY; EPB/SPB body truncation => E-INP-010 per error-taxonomy. Decision 15 / I-5/I-6: added AC-004 — IDB after first packet block has been emitted (packets_emitted>0) returns Err mapping to NEW code E-INP-013; interface table not updated; processing stops. I-11: added Test: citations per AC. — 2026-06-19"
  - "v1.1: ADR-009 rev 4 Burst B — Add no-panic AC (SEC-005). Add implementation note: interface table MUST be Vec<InterfaceInfo> (O(1) by interface_id index), NOT HashMap. Note that if_tsresol/if_tsoffset captured here feed BC-2.01.014 timestamp conversion. Coverage note: VP-027 (assigned to BC-2.01.012) also proves interface-table accumulation. — 2026-06-19"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.011: Parse pcapng Interface Description Block (IDB): Link Type and Timestamp Resolution

## Description

Each pcapng Interface Description Block (IDB, block type `0x00000001`) defines one capture
interface. wirerust extracts two fields from every IDB: `linktype` (same `DataLink` enum
already used by `decoder.rs`) and the optional `if_tsresol` TLV option (option code 9),
which encodes the per-interface timestamp resolution exponent. The extracted values are stored
per-interface and used when parsing EPBs (BC-2.01.012). The IDB `linktype` feeds the
multi-IDB agreement check (BC-2.01.018). `if_tsresol` absent defaults to 10^-6 (microseconds).

## Preconditions

1. The SHB has been successfully parsed (BC-2.01.010); byte order is established.
2. The block type field reads `0x00000001` (after byte-order correction).
3. The IDB block-total-length is consistent with the block body size.

## Postconditions

1. `linktype` is extracted from the IDB body (bytes 0–1 of the IDB fixed fields after the
   block header) and stored as a `DataLink` value for this interface index.
2. The `if_tsresol` option (option code 9) is extracted if present:
   - The value is a single `u8` exponent `e` where the resolution is `10^-e` seconds
     (base-10 exponent when bit 7 of `e` is 0) or `2^-(e & 0x7F)` (base-2 exponent when
     bit 7 of `e` is 1). `pcap-file` 2.0.0 exposes this as `IfTsResol(u8)`.
   - If `if_tsresol` is absent, the default resolution is 10^-6 (microseconds), which maps
     to an effective exponent of 6 in base-10 (same as classic-pcap microsecond resolution).
3. Both `linktype` and `if_tsresol` (or its default) are stored in an interface table keyed
   by the interface's 0-based index within the current section.
4. The `snaplen` field is at IDB body bytes **4–7** (`u32`, after the 2-byte `linktype` @0-1
   and the 2-byte `reserved` field @2-3). **Confirmed per spike Q-A3** (`interface_description.rs:45-52`):
   wire layout is `linktype u16 @0-1`, `reserved u16 @2-3`, `snaplen u32 @4-7`. `snaplen` is
   extracted and stored for SPB use (BC-2.01.013). wirerust mirrors the crate's `reserved == 0`
   enforcement: a non-zero `reserved` field is a structural IDB error returning `Err` mapped to
   E-INP-008. (The pcapng spec says reserved "should" be zero; the crate treats non-zero as an
   error; wirerust matches this behavior.)
5. **IDB error routing — uniform split (ADR-009 rev 7 Decision 20).**
   - **btl < 12 / btl % 4 != 0 / EOF** — crate rejects before returning any block →
     **E-INP-010** (not the wirerust body-decode path; wirerust never sees the body).
   - **12 ≤ btl < 20 (body 0–7 bytes)** — crate frames and returns the block; wirerust
     body-decode finds body < 8 IDB fixed-field bytes (linktype:2 + reserved:2 + snaplen:4)
     → **E-INP-008** (IDB structural parse failure; wirerust body-decode path). Constructible
     window confirmed by ADR-009 rev 7 per-block fixed-field minimum: IDB = 8 bytes.
     Canonical fixture: btl=16 → body=4 bytes < 8 → E-INP-008.
   **Uniform rule (Decision 20):** E-INP-008 covers wirerust body-decode failures for ALL block
   types — SHB body < 16 bytes, IDB body < 8 bytes, EPB body < 20 bytes, SPB body < 4 bytes —
   wherever the crate successfully framed the block but wirerust's own decode rejects the body.
   btl < 12 / misaligned / EOF is always E-INP-010 (crate framing rejection) regardless of
   block type. There is no per-block-type exception to this split.
6. **IDB OPTIONS TLV-walking (M-6, raw path).** On the raw-block path wirerust parses IDB
   options itself. The options region begins at IDB body offset 8 (immediately after the 8-byte
   fixed fields). Each option is a TLV: option-code u16 + option-length u16 + value bytes,
   where the value is padded to the next 4-byte boundary. wirerust walks options to extract
   `if_tsresol` (code 9). The following invariants apply:
   - Before reading any option value, wirerust MUST bounds-check `option-length` against the
     number of bytes remaining in the options region. If `option-length` exceeds remaining bytes,
     wirerust returns `Err` mapped to **E-INP-008** (no panic, no OOB access).
   - `opt_endofopt` (option-code 0) or end-of-body terminates the options walk immediately.
   - Unknown option codes are silently skipped (length bytes consumed; padding consumed).
   - A malformed option-length that would cause an OOB read is treated as a structural IDB
     error: `Err` mapped to E-INP-008.
   - **Limitation (ADR-009 Decision 21):** `if_tsoffset` (option code 10) is NOT extracted or
     applied this cycle. Only `if_tsresol` (code 9) is extracted. Timestamp offsets embedded
     in IDB options are silently skipped as unknown option codes. This is a known limitation
     scoped out for this cycle.

## Acceptance Criteria

- **AC-001 (no-panic — SEC-005):** This block parser MUST return `Err` for any malformed or
  truncated IDB byte sequence. `unwrap()`, `expect()`, `panic!()`, and `unreachable!()` are
  prohibited in the IDB parse path.
  **Test:** `test_BC_2_01_011_no_panic_fuzz` (property test / fuzz over arbitrary IDB bytes)
- **AC-002 (interface table data structure):** The interface table MUST be a `Vec<InterfaceInfo>`
  (indexed by 0-based interface_id, O(1) access). A `HashMap` is NOT permitted — EPB
  `interface_id` is a 0-based sequential index into a per-section Vec, making Vec the correct
  data structure. This resolves F-PERF MEDIUM finding (O(1) index vs. HashMap overhead).
  **Test:** `test_BC_2_01_011_interface_table_is_vec_indexed`
- **AC-003 (if_tsresol feeds BC-2.01.014):** The `if_tsresol` value (or default of 6 when
  absent) extracted from each IDB MUST be stored in the `InterfaceInfo` struct alongside
  `linktype` and `snaplen`. This value is consumed by the BC-2.01.014 timestamp conversion
  helper on every EPB that references this interface. **Note (Decision 21):** `if_tsoffset`
  (option code 10) is NOT extracted or applied this cycle — it is silently skipped as an
  unknown option code. Only `if_tsresol` (code 9) is stored and propagated.
  **Test:** `test_BC_2_01_011_if_tsresol_stored_in_interface_info`
- **AC-004 (interleaved IDB rejection — Decision 15):** If an IDB block is encountered AFTER
  the first packet block has been emitted (i.e., `packets_emitted > 0` at parse time), wirerust
  MUST return `Err` mapping to error code **E-INP-013**
  ("pcapng interface description block after first packet block — unsupported ordering"). The
  interface table MUST NOT be updated with the late IDB. Processing stops immediately; no
  further blocks are consumed. (E-INP-013: error-taxonomy.md v2.8.)
  **Test:** `test_BC_2_01_011_late_idb_after_packet_rejected_e_inp_013`
- **AC-005 (IDB options TLV bounds-check — M-6, no-panic):** When walking the IDB options
  region, wirerust MUST check each option's `option-length` against the number of bytes
  remaining in the body BEFORE reading the option value or its padding. If `option-length`
  exceeds the remaining bytes, wirerust MUST return `Err` mapped to E-INP-008. Consuming
  bytes past the end of the options region (OOB read / panic) is prohibited. `unwrap()`,
  `expect()`, `panic!()`, and slice-index-without-bounds-check are prohibited in the options
  walk path.
  **Test:** `test_BC_2_01_011_options_malformed_length_e_inp_008`
- **AC-006 (IDB-parse three-level precedence — M-7, Decision 17):** When an IDB is encountered,
  wirerust applies checks in this exact order:
  1. **E-INP-013 position check FIRST** — if `packets_emitted > 0`, return E-INP-013
     immediately. IDB body is NOT decoded. E-INP-001 and E-INP-011 are NOT evaluated.
  2. **E-INP-001 whitelist check SECOND** — if `linktype` is not in the allowed set,
     return E-INP-001. E-INP-011 is NOT evaluated.
  3. **E-INP-011 conflict check THIRD** — if `linktype` differs from the first IDB's
     linktype, return E-INP-011.
  This precedence is normative: a late IDB that also carries a conflicting or non-whitelisted
  linktype returns E-INP-013 only (the higher-priority check wins).
  **Test:** `test_BC_2_01_011_idb_precedence_e_inp_013_wins_over_conflict`

## Invariants

1. Interface indexes are 0-based and assigned in IDB encounter order within the section.
2. ~~The interface table is reset at each SHB (new section = new interface index namespace).~~
   **DEFERRED — unreachable under single-section constraint (ADR Decision 7/16).** Decision 7
   rejects any second SHB with E-INP-012 before wirerust ever parses a section-2 IDB; therefore
   the "reset at SHB boundary" invariant cannot be exercised or tested in the current cycle.
   Required only if multi-section pcapng support is added in a future cycle.
3. `if_tsresol` governs timestamp interpretation for all EPBs referencing this interface;
   it is immutable once the IDB is parsed.
4. `linktype` from the IDB is the same `pcap_file::DataLink` type used everywhere in
   wirerust; no translation or conversion is needed.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `if_tsresol` absent | Default resolution = microseconds (10^-6); `e = 6` used in conversion. **Test:** `test_BC_2_01_011_if_tsresol_absent_default` |
| EC-002 | `if_tsresol` = `0x09` (base-10, 10^-9 = nanoseconds) | `e = 9`; EPB timestamps divided by 1000 to produce microseconds. **Test:** `test_BC_2_01_011_if_tsresol_nanosecond` |
| EC-003 | `if_tsresol` = `0x8A` (i.e., bit 7 set + low 7 bits = 10; base-2, 2^-10 ≈ ~1ms) | Bit 7 set; base-2 exponent 10; conversion uses 2^10 = 1024 ticks/second. **Test:** `test_BC_2_01_011_if_tsresol_base2` |
| EC-004 | Two IDBs with identical `linktype` | Both stored; interface table has 2 entries; multi-IDB policy check passes. **Test:** `test_BC_2_01_011_two_idbs_same_linktype` |
| EC-005 | Two IDBs with different `linktype` | Stored individually; multi-IDB policy (BC-2.01.018) returns `Err`. **Test:** `test_BC_2_01_011_two_idbs_different_linktype` |
| EC-006 | IDB with no options (options section empty) | `if_tsresol` defaults to 10^-6; `snaplen` still extracted from fixed fields @4-7. **Test:** `test_BC_2_01_011_idb_no_options` |
| EC-007 | `linktype` = `DataLink::ETHERNET` | Stored as-is; flows directly to `PcapSource.datalink`. **Test:** `test_BC_2_01_011_linktype_ethernet` |
| EC-008 | IDB body fewer than 8 bytes (wirerust body-decode truncation). **Constructible window: 12 ≤ block_total_length < 20** — the crate frames and returns the block (btl≥12, alignment OK), but the body slice is 0–7 bytes (btl−12 < 8), so wirerust body-decode finds fewer than the 8 minimum bytes (linktype:2 + reserved:2 + snaplen:4). btl<12 is NOT this case — that is crate-rejection → E-INP-010. btl≥20 is NOT this case — body is ≥8 bytes, no truncation. Canonical fixture: btl=16 (body=4 bytes). | `Err` mapping to **E-INP-008** (IDB structural parse failure; wirerust body-decode finds body<8). No panic. **Test:** `test_BC_2_01_011_body_truncated_e_inp_008` |
| EC-009 | IDB encountered after first packet block emitted (`packets_emitted > 0`) | `Err` mapping to **E-INP-013** ("pcapng interface description block after first packet block — unsupported ordering"); interface table not updated; processing stops. **Test:** `test_BC_2_01_011_late_idb_after_packet_rejected_e_inp_013` |
| EC-010 | IDB `reserved` field non-zero | `Err` mapping to **E-INP-008** (structural IDB error; mirrors crate enforcement at `interface_description.rs:48-49`). **Test:** `test_BC_2_01_011_nonzero_reserved_e_inp_008` |
| EC-011 | IDB options region contains an option with `option-length` exceeding the remaining body bytes (malformed TLV) | `Err` mapping to **E-INP-008** (IDB structural parse failure); no panic; no OOB read. **Test:** `test_BC_2_01_011_options_malformed_length_e_inp_008` |
| EC-012 | Late IDB (`packets_emitted > 0`) that ALSO carries a `linktype` differing from the first IDB's (conflict) | `Err` mapping to **E-INP-013** (position check wins; E-INP-001 and E-INP-011 are never evaluated); no panic. **Test:** `test_BC_2_01_011_idb_precedence_e_inp_013_wins_over_conflict` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| IDB with `linktype=ETHERNET`, no options | `interface[0] = { linktype: ETHERNET, if_tsresol_exponent: 6 (default) }` | happy-path |
| IDB with `linktype=ETHERNET`, `if_tsresol=0x09` (nanoseconds) | `interface[0] = { linktype: ETHERNET, if_tsresol_exponent: 9, base: 10 }` | happy-path |
| Two IDBs, both `linktype=ETHERNET` | `interface[0]` and `interface[1]` both ETHERNET; agreement check passes | edge-case |
| Two IDBs, `linktype=ETHERNET` then `linktype=LINUX_SLL` | Parse both IDBs; agreement check (BC-2.01.018) returns `Err` with E-INP-011 | error |
| IDB body 7 bytes (truncated — body < 8 minimum: linktype:2 + reserved:2 + snaplen:4) | `Err` (E-INP-008) | error |
| IDB with `reserved != 0` | `Err` (E-INP-008) | error |
| IDB encountered after first EPB emitted | `Err` (E-INP-013) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — (covered by VP-027) | Interface-table accumulation: interface_id bounds-check before table index; interface count matches IDB count in file | Kani VP-027 on BC-2.01.012 proves interface-table accumulation across the EPB path; IDB accumulation unit test verifies 3-IDB file has 3 entries |
| — | `if_tsresol` absent → default exponent 6 | unit: craft IDB with no options; verify stored exponent = 6 |
| — | Interface index increments in IDB order | unit: 3-IDB file; verify interface table has 3 entries at indexes 0, 1, 2 |
| — | Truncated IDB returns Err; never panics (SEC-005) | unit + fuzz: fuzz IDB bytes, assert no panic |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- IDB parsing extracts the per-interface link type and timestamp parameters that are prerequisites for delivering `PcapSource { datalink, packets }`, which is CAP-01's primary output |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-124 |
| ADR Reference | ADR-009 rev 7 Decision 2 (IDB coverage), Decision 3 (multi-IDB policy), Decision 4 (if_tsresol extraction), Decision 7/16 (single-section only; second SHB rejected; Invariant 2 deferred), Decision 8 (forward-progress contract; no-panic guarantee at framing layer), Decision 10 (panic surface), Decision 15 (interleaved IDB after first packet block => E-INP-013 rejection), Decision 17 (IDB-parse three-level precedence: E-INP-013 position > E-INP-001 whitelist > E-INP-011 conflict), Decision 20 (uniform error-code rule: btl<12→E-INP-010; 12≤btl<20→body<8→E-INP-008; per-block fixed-field minimum IDB=8), Decision 21 (if_tsoffset NOT extracted this cycle; only if_tsresol code 9 is extracted and applied) |
| Error Taxonomy | E-INP-008 (IDB structural parse failure: (a) body < 8 bytes on wirerust body-decode path [constructible window: 12≤btl<20; wirerust checks body.len()>=8 itself]; (b) reserved != 0; (c) malformed options TLV length exceeding remaining body. btl<12 is NOT E-INP-008 — that is E-INP-010 via crate-rejection path), E-INP-010 (btl<12/misaligned/EOF — crate rejects before returning block), E-INP-013 (IDB after first packet block — error-taxonomy.md v2.8) |

## Related BCs

- BC-2.01.010 -- depends on (SHB must be parsed first; byte order established)
- BC-2.01.012 -- composes with (EPB uses the per-interface if_tsresol stored here)
- BC-2.01.014 -- composes with (timestamp conversion uses if_tsresol extracted here)
- BC-2.01.018 -- triggers (multi-IDB agreement check consumes linktypes from all IDBs)

## Architecture Anchors

- `pcap_file::pcapng::blocks::InterfaceDescriptionBlock` (docs.rs/pcap-file/2.0.0) -- IDB struct
- `pcap_file::pcapng::blocks::interface_description::InterfaceDescriptionOption::IfTsResol(u8)` -- option variant
- pcapng spec IETF draft §Interface-Description-Block: fixed fields layout — **`linktype u16 @0-1`, `reserved u16 @2-3`, `snaplen u32 @4-7`** (CORRECTED from prior erroneous "snaplen at bytes 2-5"; spike Q-A3 / `interface_description.rs:45-52` confirms this layout)
- `pcap-file-2.0.0/src/pcapng/blocks/interface_description.rs:40-57` — crate parse source; enforces `reserved==0` before decoding. **Note (M-1):** the `body.len() >= 8` minimum check is performed by **wirerust** on the raw path before decoding IDB fixed fields — the crate source reference is for the crate's own internal parse; wirerust mirrors the reserved==0 enforcement but adds its own body-length guard independently.
- ADR-009 Decision 4: "pure-core timestamp-conversion helper taking (ts_high, ts_low, if_tsresol)"

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads block bytes from stream |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O during block reading; interface table is local per-parse state) |
