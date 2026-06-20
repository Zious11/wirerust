---
document_type: behavioral-contract
level: L3
version: "1.0"
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
modified: []
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
4. The `snaplen` field (IDB bytes 2–5) is extracted and stored for SPB use (BC-2.01.013).
5. If the IDB is truncated below its minimum fixed-field size, returns `Err` mapping to
   E-INP-008 (pcapng SHB/structural parse failure — reused for any block-level truncation).

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
| EC-001 | `if_tsresol` absent | Default resolution = microseconds (10^-6); `e = 6` used in conversion |
| EC-002 | `if_tsresol` = `0x09` (base-10, 10^-9 = nanoseconds) | `e = 9`; EPB timestamps divided by 1000 to produce microseconds |
| EC-003 | `if_tsresol` = `0x80 | 0x0A` (base-2, 2^-10 ≈ ~1ms) | Bit 7 set; base-2 exponent 10; conversion uses 2^10 = 1024 ticks/second |
| EC-004 | Two IDBs with identical `linktype` | Both stored; interface table has 2 entries; multi-IDB policy check passes |
| EC-005 | Two IDBs with different `linktype` | Stored individually; multi-IDB policy (BC-2.01.018) returns `Err` |
| EC-006 | IDB with no options (options section empty) | `if_tsresol` defaults to 10^-6; `snaplen` still extracted from fixed fields |
| EC-007 | `linktype` = `DataLink::ETHERNET` | Stored as-is; flows directly to `PcapSource.datalink` |
| EC-008 | IDB body truncated (block-total-length < minimum) | `Err` mapping to E-INP-008 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| IDB with `linktype=ETHERNET`, no options | `interface[0] = { linktype: ETHERNET, if_tsresol_exponent: 6 (default) }` | happy-path |
| IDB with `linktype=ETHERNET`, `if_tsresol=0x09` (nanoseconds) | `interface[0] = { linktype: ETHERNET, if_tsresol_exponent: 9, base: 10 }` | happy-path |
| Two IDBs, both `linktype=ETHERNET` | `interface[0]` and `interface[1]` both ETHERNET; agreement check passes | edge-case |
| Two IDBs, `linktype=ETHERNET` then `linktype=LINUX_SLL` | Parse both IDBs; agreement check (BC-2.01.018) returns `Err` with E-INP-011 | error |
| IDB body 3 bytes (truncated) | `Err` (E-INP-008) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | `if_tsresol` absent → default exponent 6 | unit: craft IDB with no options; verify stored exponent |
| — | Interface index increments in IDB order | unit: 3-IDB file; verify interface table has 3 entries at indexes 0, 1, 2 |
| — | Truncated IDB never panics | fuzz: fuzz IDB bytes, assert no panic |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- IDB parsing extracts the per-interface link type and timestamp parameters that are prerequisites for delivering `PcapSource { datalink, packets }`, which is CAP-01's primary output |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-124 |
| ADR Reference | ADR-009 Decision 2 (IDB coverage), Decision 3 (multi-IDB policy), Decision 4 (if_tsresol extraction) |

## Related BCs

- BC-2.01.010 -- depends on (SHB must be parsed first; byte order established)
- BC-2.01.012 -- composes with (EPB uses the per-interface if_tsresol stored here)
- BC-2.01.014 -- composes with (timestamp conversion uses if_tsresol extracted here)
- BC-2.01.018 -- triggers (multi-IDB agreement check consumes linktypes from all IDBs)

## Architecture Anchors

- `pcap_file::pcapng::blocks::InterfaceDescriptionBlock` (docs.rs/pcap-file/2.0.0) -- IDB struct
- `pcap_file::pcapng::blocks::interface_description::InterfaceDescriptionOption::IfTsResol(u8)` -- option variant
- pcapng spec IETF draft §Interface-Description-Block: fixed fields layout (linktype at bytes 0-1, snaplen at 2-5)
- ADR-009 Decision 4: "pure-core timestamp-conversion helper taking (ts_high, ts_low, if_tsresol)"

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads block bytes from stream |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O during block reading; interface table is local per-parse state) |
