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

# BC-2.01.018: Multi-IDB Link-Type Agreement Policy: Conflict Returns Error (Fail-Closed)

## Description

A legal pcapng file may contain multiple Interface Description Blocks (IDBs), each representing
a distinct capture interface. wirerust's `PcapSource.datalink` is a single `DataLink` field
(not per-packet); accommodating multiple different `linktype` values without structural changes
to `RawPacket` or `decoder.rs` is out of scope for this cycle (ADR-009 Decision 3). The chosen
policy is **fail-closed**: all IDBs encountered within a section MUST carry the same `linktype`
value. If any two IDBs differ, the reader returns `Err` immediately with context identifying
the conflicting link types, mapping to E-INP-011. Files where all IDBs agree (including the
common case of a single IDB) succeed normally.

## Preconditions

1. At least one IDB has been parsed within the current section.
2. A second (or subsequent) IDB has been parsed.
3. The `linktype` fields from all parsed IDBs are available for comparison.

## Postconditions

1. If all IDBs carry the same `linktype`: the agreed value is accepted as `PcapSource.datalink`.
   Parse continues to EPB/SPB blocks.
2. If any IDB carries a `linktype` differing from the first IDB's `linktype`:
   - Returns `Err` whose anyhow chain contains context: `"pcapng multi-interface link-type conflict: interface 0 has {first:?}, interface {n} has {other:?}"`.
   - `first` and `other` are `DataLink` Debug repr values.
   - No packets are returned.
   - No panic occurs.
3. Single-IDB files (the common case) trivially satisfy the agreement policy and always proceed
   to packet parsing.
4. The check runs lazily: on each new IDB parsed, its `linktype` is compared to the first
   IDB's. The first mismatch triggers the error immediately; subsequent IDBs are not parsed.

## Invariants

1. This policy is fail-closed for the current cycle. Relaxation (per-packet link type) is
   explicitly deferred to a future cycle per ADR-009 Decision 3.
2. The known limitation — rejecting legitimate multi-NIC captures that mix link types
   (e.g., Ethernet + Linux Cooked from `tcpdump -i any`) — is documented in this BC and in
   ADR-009 Consequences.
3. The error produced is E-INP-011 (separate from E-INP-008/010 which cover structural
   truncation; E-INP-011 is a semantic policy violation).
4. The check applies per section: a multi-section pcapng where each section has a single IDB
   is legal and succeeds (each section's IDB table resets at the SHB boundary).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | One IDB (most common case) | Agreement trivially satisfied; `PcapSource.datalink = idb[0].linktype` |
| EC-002 | Two IDBs, both `ETHERNET` | Agreement satisfied |
| EC-003 | Two IDBs: `ETHERNET` then `LINUX_SLL` | `Err` E-INP-011: "interface 0 has ETHERNET, interface 1 has LINUX_SLL" |
| EC-004 | Three IDBs: `ETHERNET`, `ETHERNET`, `RAW` | `Err` E-INP-011 on third IDB: "interface 0 has ETHERNET, interface 2 has RAW" |
| EC-005 | Multi-section pcapng: section 1 has `ETHERNET` IDB, section 2 has `LINUX_SLL` IDB | Per-section isolation; each section succeeds individually (sections never interleave) |
| EC-006 | Two IDBs: `ETHERNET` (whitelisted) then `IEEE802_11` (non-whitelisted) | E-INP-011 fires first (linktype mismatch); E-INP-001 whitelist check is never reached |
| EC-007 | pcapng file with 0 IDBs before first EPB | No IDB error: separate error path (E-INP-009 / BC-2.01.017); this BC's check is never reached |
| EC-008 | Two IDBs both `IEEE802_11` (non-whitelisted but agreeing) | E-INP-011 does NOT fire (they agree); E-INP-001 (BC-2.01.016 whitelist check) fires instead |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| pcapng with single IDB `ETHERNET` | `Ok(PcapSource { datalink: ETHERNET })` | happy-path |
| pcapng with two IDBs, both `ETHERNET` | `Ok(PcapSource { datalink: ETHERNET })` | happy-path |
| pcapng with IDB `ETHERNET` then IDB `LINUX_SLL` | `Err` containing "link-type conflict" and both variant names | error |
| pcapng with three IDBs: `ETH`, `ETH`, `RAW` | `Err` on third IDB; mentions interface 2 and `RAW` | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Single-IDB pcapng always passes agreement check | unit: craft single-IDB pcapng; assert Ok |
| — | Mixed-linktype pcapng always returns Err with E-INP-011 context | unit: craft two-IDB pcapng with differing linktypes; assert Err contains "link-type conflict" |
| — | Agreeing two-IDB pcapng passes and sets correct datalink | unit: two-IDB ETHERNET pcapng; assert Ok with datalink=ETHERNET |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- the multi-IDB agreement policy is a constraint on the ingestion pipeline's ability to produce a single `PcapSource.datalink` value; enforcing it here preserves CAP-01's output contract without changes to downstream consumers |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-124 |
| ADR Reference | ADR-009 Decision 3 ("wirerust requires all IDB blocks in a section to agree on linktype; if two or more IDBs carry differing linktype values, the reader returns an error with context identifying the conflicting link types"), Consequences (known limitation: multi-NIC captures with different interfaces) |
| Error Taxonomy | E-INP-011 (new entry; proposed taxonomy addendum) |
| Known Limitation | Rejects legitimate multi-NIC captures mixing Ethernet and Linux Cooked interfaces. Documented per ADR-009. Revisit if mixed-interface captures become a user requirement. |

## Related BCs

- BC-2.01.011 -- depends on (IDB linktype values come from BC-2.01.011 parsing)
- BC-2.01.016 -- composes with (agreement check runs first; whitelist check runs second)
- BC-2.01.017 -- composes with (E-INP-011 is surfaced via the error-context chain defined in BC-2.01.017)

## Architecture Anchors

- ADR-009 Decision 3: "require all IDB blocks in a section to agree on linktype"
- ADR-009 Consequences (Negative/Trade-offs): "multi-IDB link-type-agreement policy will reject legitimate multi-NIC capture files that mix Ethernet and Linux Cooked interfaces"
- `pcap_file::pcapng::blocks::InterfaceDescriptionBlock.linktype` -- the field compared across IDBs

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (check performed on already-parsed linktype values) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | trivially safe |
| **Overall classification** | pure core (comparison of two DataLink values; no I/O) |
