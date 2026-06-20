---
document_type: behavioral-contract
level: L3
version: "1.2"
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
5. If the IDB body is fewer than 8 bytes (the minimum to contain linktype:2 + reserved:2 +
   snaplen:4), wirerust returns `Err` mapping to E-INP-008 (SHB/IDB structural parse failure).
   E-INP-008 covers SHB and IDB structural errors ONLY. EPB/SPB body truncation is a distinct
   failure mode routed to E-INP-010 per error-taxonomy.md — E-INP-008 is NOT reused for
   packet-block truncation.

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
- **AC-003 (if_tsresol / if_tsoffset feeds BC-2.01.014):** The `if_tsresol` value (or default
  of 6 when absent) extracted from each IDB MUST be stored in the `InterfaceInfo` struct
  alongside `linktype` and `snaplen`. This value is consumed by the BC-2.01.014 timestamp
  conversion helper on every EPB that references this interface.
  **Test:** `test_BC_2_01_011_if_tsresol_stored_in_interface_info`
- **AC-004 (interleaved IDB rejection — Decision 15):** If an IDB block is encountered AFTER
  the first packet block has been emitted (i.e., `packets_emitted > 0` at parse time), wirerust
  MUST return `Err` mapping to NEW error code **E-INP-013**
  ("pcapng interface description block after first packet block — unsupported ordering"). The
  interface table MUST NOT be updated with the late IDB. Processing stops immediately; no
  further blocks are consumed. (E-INP-013 is added to the error taxonomy in a separate burst;
  this AC references it by code for forward-reference tracking.)
  **Test:** `test_BC_2_01_011_late_idb_after_packet_rejected_e_inp_013`

## Invariants

1. Interface indexes are 0-based and assigned in IDB encounter order within the section.
2. The interface table is reset at each SHB (new section = new interface index namespace).
3. `if_tsresol` governs timestamp interpretation for all EPBs referencing this interface;
   it is immutable once the IDB is parsed.
4. `linktype` from the IDB is the same `pcap_file::DataLink` type used everywhere in
   wirerust; no translation or conversion is needed.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `if_tsresol` absent | Default resolution = microseconds (10^-6); `e = 6` used in conversion. **Test:** `test_BC_2_01_011_if_tsresol_absent_default` |
| EC-002 | `if_tsresol` = `0x09` (base-10, 10^-9 = nanoseconds) | `e = 9`; EPB timestamps divided by 1000 to produce microseconds. **Test:** `test_BC_2_01_011_if_tsresol_nanosecond` |
| EC-003 | `if_tsresol` = `0x80 | 0x0A` (base-2, 2^-10 ≈ ~1ms) | Bit 7 set; base-2 exponent 10; conversion uses 2^10 = 1024 ticks/second. **Test:** `test_BC_2_01_011_if_tsresol_base2` |
| EC-004 | Two IDBs with identical `linktype` | Both stored; interface table has 2 entries; multi-IDB policy check passes. **Test:** `test_BC_2_01_011_two_idbs_same_linktype` |
| EC-005 | Two IDBs with different `linktype` | Stored individually; multi-IDB policy (BC-2.01.018) returns `Err`. **Test:** `test_BC_2_01_011_two_idbs_different_linktype` |
| EC-006 | IDB with no options (options section empty) | `if_tsresol` defaults to 10^-6; `snaplen` still extracted from fixed fields @4-7. **Test:** `test_BC_2_01_011_idb_no_options` |
| EC-007 | `linktype` = `DataLink::ETHERNET` | Stored as-is; flows directly to `PcapSource.datalink`. **Test:** `test_BC_2_01_011_linktype_ethernet` |
| EC-008 | IDB body fewer than 8 bytes (wirerust body-decode truncation) | `Err` mapping to **E-INP-008** (IDB structural parse failure — crate returned block but body too short). **Test:** `test_BC_2_01_011_body_truncated_e_inp_008` |
| EC-009 | IDB encountered after first packet block emitted (`packets_emitted > 0`) | `Err` mapping to **E-INP-013** ("pcapng interface description block after first packet block — unsupported ordering"); interface table not updated; processing stops. **Test:** `test_BC_2_01_011_late_idb_after_packet_rejected_e_inp_013` |
| EC-010 | IDB `reserved` field non-zero | `Err` mapping to **E-INP-008** (structural IDB error; mirrors crate enforcement at `interface_description.rs:48-49`). **Test:** `test_BC_2_01_011_nonzero_reserved_e_inp_008` |

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
| ADR Reference | ADR-009 Decision 2 (IDB coverage), Decision 3 (multi-IDB policy), Decision 4 (if_tsresol extraction), Decision 8 (forward-progress contract; no-panic guarantee at framing layer), Decision 10 (panic surface), Decision 15 (interleaved IDB after first packet block => E-INP-013 rejection) |
| Error Taxonomy | E-INP-008 (IDB structural parse failure: body < 8 bytes or reserved != 0), E-INP-013 (IDB after first packet block — forward-reference; taxonomy updated in separate burst) |

## Related BCs

- BC-2.01.010 -- depends on (SHB must be parsed first; byte order established)
- BC-2.01.012 -- composes with (EPB uses the per-interface if_tsresol stored here)
- BC-2.01.014 -- composes with (timestamp conversion uses if_tsresol extracted here)
- BC-2.01.018 -- triggers (multi-IDB agreement check consumes linktypes from all IDBs)

## Architecture Anchors

- `pcap_file::pcapng::blocks::InterfaceDescriptionBlock` (docs.rs/pcap-file/2.0.0) -- IDB struct
- `pcap_file::pcapng::blocks::interface_description::InterfaceDescriptionOption::IfTsResol(u8)` -- option variant
- pcapng spec IETF draft §Interface-Description-Block: fixed fields layout — **`linktype u16 @0-1`, `reserved u16 @2-3`, `snaplen u32 @4-7`** (CORRECTED from prior erroneous "snaplen at bytes 2-5"; spike Q-A3 / `interface_description.rs:45-52` confirms this layout)
- `pcap-file-2.0.0/src/pcapng/blocks/interface_description.rs:40-57` — crate parse source; enforces `reserved==0` and `body.len() >= 8` before decoding
- ADR-009 Decision 4: "pure-core timestamp-conversion helper taking (ts_high, ts_low, if_tsresol)"

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads block bytes from stream |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O during block reading; interface table is local per-parse state) |
