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
  - "v1.2: ADR-009 rev 4 Burst B — Add VP-030 to Verification Properties. KEEP multi-IDB linktype-conflict rule (all IDBs must agree; first conflict → E-INP-011 before any packet). MOVE directory-mode per-file-isolation claim: AC-002 re-attributed to STORY-128 (main.rs loop refactor) per ADR-009 Decision 12; AC-002 now documents that STORY-128 owns this behavior. BC-2.01.018 owns the CONFLICT RULE, not the main.rs loop behavior. Add holdout: two-IDB-different-linktypes (→ E-INP-011); two-IDB-same-linktype (→ accepted). — 2026-06-19"
  - "v1.1: F-11 completeness delta — (1) Add AC for directory-mode per-file error isolation: E-INP-011 on one file does not abort the full run; (2) Add AC clarifying common user trigger (tcpdump -i any) in E-INP-011 message; cross-reference BC-2.12.011 directory-mode isolation. — 2026-06-19"
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

## Acceptance Criteria

- **AC-001:** When two or more IDBs in a single pcapng section carry differing `linktype` values,
  the reader returns `Err` with a message that (a) identifies the conflicting link types by
  `DataLink` Debug repr and (b) includes a hint that this commonly arises from `tcpdump -i any`
  captures mixing link types, and that wirerust requires a single link type per file. The exact
  message format is defined by E-INP-011.
- **AC-002 (Directory-Mode Per-File Isolation — OWNED BY STORY-128):** [Re-attributed per
  ADR-009 Decision 12, rev 4.] The directory-mode per-file error isolation behavior (catch
  reader errors per-file, do not propagate via `?`, accumulate errors, report to stderr, set
  exit code 1 if any file failed) is OWNED BY STORY-128 (src/main.rs:241-244 loop refactor),
  NOT by this BC (reader.rs scope). BC-2.01.018 owns the multi-IDB CONFLICT RULE only (all
  IDBs must agree; first conflict → E-INP-011; reader returns Err immediately). The main.rs
  loop's catch-and-continue behavior is the responsibility of STORY-128 and applies to ALL
  reader error classes, not only E-INP-011. BC-2.01.018 makes no postcondition about what
  happens after the reader returns Err — that is main.rs scope.
  - Implementation note: E-INP-011 is produced BEFORE any packet is returned (Postcondition
    4: "lazy check; first mismatch triggers error immediately"). STORY-128 catches this Err
    at the directory-loop boundary and continues to the next file.

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
| EC-009 | Directory with file_a.pcapng (ETHERNET+LINUX_SLL conflict) and file_b.pcapng (ETHERNET only) | **STORY-128 scope (re-attributed per ADR-009 Decision 12):** E-INP-011 on file_a; STORY-128 main.rs loop catches Err and continues; file_b processed successfully; overall exit code 1. BC-2.01.018 owns only the E-INP-011 production; the catch-and-continue is STORY-128. |

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
| VP-030 | Multi-IDB linktype agreement totality: any sequence of IDB linktype u16 values either all-equal (accepted, Ok) or first-conflict returns E-INP-011 immediately; no sequence produces a panic or silent incorrect result | proptest (P1): generate arbitrary Vec<u16> as IDB linktype sequence; assert all-same → Ok, any-different → Err with E-INP-011 context |
| — | Single-IDB pcapng always passes agreement check (holdout: single-IDB file) | unit: craft single-IDB pcapng; assert Ok |
| — | Two-IDB different linktypes → E-INP-011 before any packet (holdout: two IDBs linktype 1 & 113) | unit: craft two-IDB pcapng with ETHERNET then LINUX_SLL; assert Err E-INP-011; confirm no packets returned |
| — | Two-IDB same linktype (holdout: both linktype 1) → Ok | unit: craft two-IDB ETHERNET pcapng; assert Ok with datalink=ETHERNET |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- the multi-IDB agreement policy is a constraint on the ingestion pipeline's ability to produce a single `PcapSource.datalink` value; enforcing it here preserves CAP-01's output contract without changes to downstream consumers |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-124 (multi-IDB conflict rule, reader.rs); STORY-128 (directory-mode per-file isolation, main.rs — owns AC-002 re-attribution per Decision 12) |
| ADR Reference | ADR-009 Decision 3 ("wirerust requires all IDB blocks in a section to agree on linktype; if two or more IDBs carry differing linktype values, the reader returns an error with context identifying the conflicting link types"), Decision 12 (per-file isolation is STORY-128 main.rs scope, not reader.rs scope), Consequences (known limitation: multi-NIC captures with different interfaces) |
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
