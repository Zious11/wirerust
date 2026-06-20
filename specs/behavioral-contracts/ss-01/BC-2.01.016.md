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
capability: CAP-02
lifecycle_status: active
introduced: v0.10.0-pcapng
modified:
  - "v1.2: Pass-2 remediation Burst P2b (ADR-009 rev 5) — (I-5/Decision 15 amendment) rewrite Description and Preconditions to make clear whitelist fires at IDB-PARSE TIME, not deferred to post-multi-IDB-check. Remove all text implying check is 'after all IDBs' or 'at first packet'. (I-11) add Test: citations to ACs. — 2026-06-19"
  - "v1.1: ADR-009 rev 4 Burst B — No VP assigned (test-sufficient; ADR-009 dispatch confirmed). Confirm mirrors BC-2.01.001 (CAP-02). Add no-panic AC (SEC-005). Add explicit mirror-confirmation note. Minimal normative change. — 2026-06-19"
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
When an IDB's `linktype` field carries a `DataLink` variant not in the accepted whitelist
`{ETHERNET, RAW, IPV4, IPV6, LINUX_SLL}`, the reader MUST return an error with the same
message format as BC-2.01.001 Postcondition 2. The whitelist check fires at **IDB-PARSE
TIME** — immediately when the IDB block body is decoded — before any packet block from that
interface is consumed. A whitelist violation returns `Err` immediately at the IDB stage; no
packets from the violating interface are ever read. This preserves the invariant that
`PcapSource.datalink` is always a whitelisted value, regardless of capture format. The check
does NOT wait until "after all IDBs are parsed" or "at first packet time".

## Preconditions

1. The pcapng SHB has been parsed.
2. The block-walk loop has reached an IDB block and decoded its body (the `linktype` field is
   now available).
3. **The whitelist check fires here — at IDB-parse time — before any packet block is
   consumed from this interface.** There is no dependency on the multi-IDB agreement check
   (BC-2.01.018); the whitelist fires independently per IDB as each IDB is parsed.

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

## Acceptance Criteria

- **AC-001 (mirror of BC-2.01.001):** The link-type acceptance whitelist is IDENTICAL to
  BC-2.01.001: exactly `{ETHERNET, RAW, IPV4, IPV6, LINUX_SLL}` (5 variants). Any change
  to the whitelist is a coordinated breaking change to both BCs simultaneously.
  **Test:** `test_BC_2_01_016_whitelist_mirrors_bc_2_01_001` — assert that the set of
  accepted `DataLink` variants in the pcapng path equals `{ETHERNET, RAW, IPV4, IPV6,
  LINUX_SLL}` and matches the classic-pcap whitelist.
- **AC-002 (no-panic — SEC-005):** This whitelist check MUST return `Err` for any non-whitelisted
  `DataLink` variant. `unwrap()`, `expect()`, `panic!()`, and `unreachable!()` are prohibited.
  Since this is a pure match on an enum value, no panic path exists; this AC asserts that
  the implementation retains that property through any future enum variant additions.
  **Test:** `test_BC_2_01_016_non_whitelisted_linktype_returns_err_no_panic` — inject
  `DataLink::IEEE802_11` in a pcapng IDB; assert `Err`, no panic, no unwrap.
- **AC-003 (no VP — test-sufficient):** No new formal VP is assigned to this BC per
  ADR-009 dispatch. The integration test covering BC-2.01.001's whitelist (STORY-126)
  is sufficient for this parallel check.
  **Test:** Covered by STORY-126 integration suite; no additional VP file required.

## Invariants

1. The acceptance whitelist is identical to BC-2.01.001: exactly 5 variants. Any change to
   the whitelist is a breaking change to BOTH BC-2.01.001 and BC-2.01.016.
2. The error message format is identical to BC-2.01.001 Postcondition 2 (E-INP-001); the
   same error taxonomy entry applies.
3. This check fires at IDB-parse time. The multi-IDB agreement check (BC-2.01.018) is a
   separate, independent check that runs after the interface table is fully built. A
   multi-IDB conflict produces E-INP-011 (BC-2.01.018), not E-INP-001. However, if any
   individual IDB fails the whitelist check first (at IDB-parse time), E-INP-001 is
   returned before the multi-IDB check can run.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | pcapng IDB `linktype = ETHERNET` | Accepted; `PcapSource.datalink = ETHERNET` |
| EC-002 | pcapng IDB `linktype = IEEE802_11` (numeric 105) | `Err` with message identical to BC-2.01.001 EC-001 |
| EC-003 | pcapng with two IDBs, both `linktype = IEEE802_11` | Whitelist fires at IDB-parse time on the FIRST IDB → `Err` E-INP-001 immediately; the second IDB and the multi-IDB check are never reached |
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
- BC-2.01.018 -- related (multi-IDB agreement check is independent; both check linktype but
  at different stages — this BC fires at IDB-parse time, BC-2.01.018 fires after the full
  interface table is built)

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
