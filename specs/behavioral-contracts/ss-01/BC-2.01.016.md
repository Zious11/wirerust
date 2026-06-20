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
capability: CAP-02
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

# BC-2.01.016: Reject pcapng with Unsupported Link Type in IDB (Mirrors BC-2.01.001)

## Description

The link-type whitelist enforced by BC-2.01.001 for classic-pcap applies equally to pcapng.
When the first (or only) IDB's `linktype` field carries a `DataLink` variant not in the
accepted whitelist `{ETHERNET, RAW, IPV4, IPV6, LINUX_SLL}`, the reader MUST return an error
with the same message format as BC-2.01.001 Postcondition 2. The check fires after all IDBs
are parsed and the multi-IDB agreement check (BC-2.01.018) has passed; the accepted single
`linktype` value is then checked against the whitelist. This preserves the invariant that
`PcapSource.datalink` is always a whitelisted value, regardless of capture format.

## Preconditions

1. The pcapng SHB and at least one IDB have been parsed.
2. The multi-IDB agreement check (BC-2.01.018) has passed (all IDBs agree on `linktype`).
3. The agreed `linktype` value is now being evaluated against the whitelist.

## Postconditions

1. If `linktype` is in `{ETHERNET, RAW, IPV4, IPV6, LINUX_SLL}`:
   - `PcapSource.datalink` is set to the accepted `DataLink` variant.
   - Packet reading proceeds.
2. If `linktype` is any other value:
   - Returns `Err` with message: `"Unsupported pcap link type: {linktype:?}. Supported: Ethernet (1), Raw IP (101), Linux Cooked (113), IPv4 (228), IPv6 (229)"`.
   - `{linktype:?}` is the `DataLink` enum Debug variant name (same as BC-2.01.001).
   - No packets are returned.
   - No panic occurs.
3. `PcapSource.datalink` is always a whitelisted value; this invariant holds for both
   classic-pcap and pcapng.

## Invariants

1. The acceptance whitelist is identical to BC-2.01.001: exactly 5 variants. Any change to
   the whitelist is a breaking change to BOTH BC-2.01.001 and BC-2.01.016.
2. The error message format is identical to BC-2.01.001 Postcondition 2 (E-INP-001); the
   same error taxonomy entry applies.
3. This check is downstream of the multi-IDB agreement check; a multi-IDB conflict produces
   E-INP-011 (BC-2.01.018), not E-INP-001.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | pcapng IDB `linktype = ETHERNET` | Accepted; `PcapSource.datalink = ETHERNET` |
| EC-002 | pcapng IDB `linktype = IEEE802_11` (numeric 105) | `Err` with message identical to BC-2.01.001 EC-001 |
| EC-003 | pcapng with two IDBs, both `linktype = IEEE802_11` | Multi-IDB check passes (they agree); this whitelist check fires with IEEE802_11 → `Err` E-INP-001 |
| EC-004 | pcapng with one IDB `linktype = LINUX_SLL` | Accepted; `PcapSource.datalink = LINUX_SLL` |
| EC-005 | pcapng with zero IDBs (malformed) | Handled by BC-2.01.018 / BC-2.01.017; this check never reached |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| pcapng with IDB `linktype=ETHERNET` | `Ok(PcapSource { datalink: ETHERNET })` | happy-path |
| pcapng with IDB `linktype=IEEE802_11` | `Err` containing "Unsupported pcap link type: IEEE802_11" | error |
| pcapng with IDB `linktype=LINUX_SLL` | `Ok(PcapSource { datalink: LINUX_SLL })` | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Whitelist is identical between classic-pcap and pcapng paths | unit: same DataLink variants accepted in both branches |
| — | Rejection path never panics for any DataLink variant | unit: inject all known non-whitelisted variants |
| — | `PcapSource.datalink` is always whitelisted on Ok path | proptest: pcapng with arbitrary IDB linktype; assert Ok iff in whitelist |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-02 ("Link-Type Gating") per domain/capabilities/cap-02-link-type-gating.md |
| Capability Anchor Justification | CAP-02 ("Link-Type Gating") per domain/capabilities/cap-02-link-type-gating.md -- this BC extends the link-type whitelist gate (CAP-02's primary concern) to the pcapng IDB path; the whitelist and error message are identical to BC-2.01.001 because CAP-02 applies regardless of capture format |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-124 |
| ADR Reference | ADR-009 (implicit: zero analyzer changes means DataLink whitelist unchanged) |

## Related BCs

- BC-2.01.001 -- mirrors (identical whitelist; pcapng analog of classic-pcap link-type gate)
- BC-2.01.011 -- depends on (linktype value extracted from IDB in BC-2.01.011)
- BC-2.01.018 -- depends on (multi-IDB agreement check runs before this whitelist check)

## Architecture Anchors

- `src/reader.rs:50-61` -- existing DataLink match whitelist; pcapng path must share or replicate this check
- `pcap_file::DataLink` -- shared type; IDB `linktype` is the same enum as classic-pcap global header `network`
- ADR-009 Consequences: "The DataLink type flows directly from idb.linktype to PcapSource.datalink with zero translation code"

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (check performed on already-parsed linktype value) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | trivially safe |
| **Overall classification** | pure core (whitelist membership check only) |
