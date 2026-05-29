---
document_type: behavioral-contract
level: L3
version: "1.6"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reader.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-01
capability: CAP-02
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21
  - v1.3: Phase 3 per-story adversarial review — corrected extracted_from accuracy: EC-001 and Postcondition 2 now state the rejection message contains the DataLink Debug variant name (e.g. "IEEE802_11") plus the supported-types suffix, not "the numeric value" — 2026-05-21
  - v1.4: Phase 3 per-story adversarial review — corrected Architecture Anchor line ranges: DataLink match block closes at :61, not :60; Traceability Architecture Module updated to match — 2026-05-21
  - v1.5: Phase 3 per-story adversarial review — F-8.1: corrected Traceability "Architecture Module" range from reader.rs:46-61 to reader.rs:50-61 (46 is the header-parse line, not the link-type gate); F-9.06: rewrote EC-004 to remove out-of-scope decoder `from_ip` reference, keeping behavior description within reader link-type-gate scope — 2026-05-21
  - v1.6: DF-16.A citation fix — re-anchored capability from CAP-01 to CAP-02 (link-type gating is explicitly CAP-02's domain per cap-02-link-type-gating.md; this BC describes the 5-element whitelist acceptance/rejection gate at reader.rs:50-61, not the packet-vector loading of CAP-01); corrected broken capabilities.md §CAP-NN citation to per-cap file path — 2026-05-28
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.001: Accept Supported Link Types and Reject Unsupported at File Open

## Description

The PCAP reader performs link-type gating at file-open time: it accepts exactly five
`pcap_file::DataLink` variants (ETHERNET, RAW, IPV4, IPV6, LINUX_SLL) and immediately returns
an anyhow error for any other value. This is the primary ingestion gate -- no packets from an
unsupported file are processed. This contract is enforced in `src/reader.rs:50-61`.

## Preconditions

1. A pcap file path is provided and the file exists and is readable.
2. The pcap file has a valid classic-pcap global header (magic number matches `pcap_file`
   crate expectations).
3. The `DataLink` value is read from the pcap global header `network` field.

## Postconditions

1. If `DataLink` is one of {ETHERNET, RAW, IPV4, IPV6, LINUX_SLL}: returns `Ok(PcapSource)`
   with `datalink` set to the accepted variant. Packet reading proceeds.
2. If `DataLink` is any other value: returns `Err(anyhow!("Unsupported pcap link type: {other:?}. Supported: Ethernet (1), Raw IP (101), Linux Cooked (113), IPv4 (228), IPv6 (229)"))`.
   `{other:?}` expands to the `DataLink` enum's Debug variant name (e.g., `"IEEE802_11"`), not a raw integer.
   No packets are loaded. No panic occurs.
3. The returned `PcapSource.datalink` is always a member of the accepted whitelist.

## Invariants

1. The acceptance whitelist is exactly 5 variants. Adding or removing link types is a
   breaking change to this contract.
2. Rejection is always via `anyhow::Error` return, never via `panic!`.
3. The numeric DataLink values for the accepted types are: ETHERNET=1, RAW=101,
   LINUX_SLL=113, IPV4=228, IPV6=229.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DataLink = IEEE 802.11 (numeric 105) | Returns Err whose message is `"Unsupported pcap link type: IEEE802_11. Supported: Ethernet (1), Raw IP (101), Linux Cooked (113), IPv4 (228), IPv6 (229)"` — the token after the colon is the DataLink Debug variant name, not a raw integer |
| EC-002 | DataLink = ETHERNET (most common case) | Returns Ok(PcapSource { datalink: ETHERNET, ... }) |
| EC-003 | DataLink = LINUX_SLL (tcpdump -i any captures) | Returns Ok(PcapSource { datalink: LINUX_SLL, ... }) |
| EC-004 | DataLink = RAW and DataLink = IPV4 | Both are members of the accepted whitelist; the reader's link-type gate returns Ok(PcapSource) for each without any decode step |
| EC-005 | pcapng file (different magic number) | pcap_file crate header parse fails before link-type check; error wraps as "Failed to parse pcap header" |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| pcap with ETHERNET link type | Ok(PcapSource { datalink: ETHERNET }) | happy-path |
| pcap with RAW link type | Ok(PcapSource { datalink: RAW }) | happy-path |
| pcap with IPV4 link type | Ok(PcapSource { datalink: IPV4 }) | happy-path |
| pcap with IPV6 link type | Ok(PcapSource { datalink: IPV6 }) | happy-path |
| pcap with LINUX_SLL link type | Ok(PcapSource { datalink: LINUX_SLL }) | happy-path |
| pcap with IEEE 802.11 (link type 105) | Err with message containing "Unsupported pcap link type" | error |
| pcap with arbitrary unknown link type | Err (no panic) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Accepted DataLink values are exactly {ETHERNET, RAW, IPV4, IPV6, LINUX_SLL} | proptest: generate arbitrary DataLink variants; assert Ok iff in whitelist |
| — | Rejection path never panics | fuzz: fuzz pcap header bytes, assert no panic |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-02 ("Link-Type Gating") per domain/capabilities/cap-02-link-type-gating.md |
| Capability Anchor Justification | CAP-02 ("Link-Type Gating") per domain/capabilities/cap-02-link-type-gating.md -- this BC describes the 5-element whitelist acceptance/rejection gate at reader.rs:50-61, which is exactly the behavior CAP-02 defines; CAP-01 covers packet-vector loading after the gate passes |
| L2 Domain Invariants | None directly (link-type gating is a precondition to all invariants) |
| Architecture Module | SS-01 (reader.rs:50-61, C-4) |
| Stories | STORY-001 |
| Origin BC | BC-RDR-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.01.002 -- depends on (packet reading only proceeds if link type is accepted)
- BC-2.02.001 -- composes with (decoder uses the DataLink from PcapSource)

## Architecture Anchors

- `src/reader.rs:46` -- `PcapReader::new(reader).context("Failed to parse pcap header")` -- header parse
- `src/reader.rs:50-61` -- DataLink match: whitelist arms at :51-55, rejection branch at :56-60, closing brace at :61
- `src/reader.rs:51-55` -- acceptance whitelist (ETHERNET, RAW, IPV4, IPV6, LINUX_SLL)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reader.rs:50-61` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **guard clause**: match arm returns Err for non-whitelisted variants
- **type constraint**: `pcap_file::DataLink` is an external crate enum; all variants are known

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads file (BufReader); no writes |
| **Global state access** | none |
| **Deterministic** | yes -- same file always produces same result |
| **Thread safety** | Send + Sync (takes &Path, returns owned value) |
| **Overall classification** | effectful shell (I/O only; no mutable shared state) |
