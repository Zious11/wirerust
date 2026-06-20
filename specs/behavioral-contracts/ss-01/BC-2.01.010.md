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

# BC-2.01.010: Parse pcapng Section Header Block (SHB): Byte-Order Detection and Version

## Description

After the magic-byte probe (BC-2.01.009) identifies a pcapng stream, the reader MUST parse
the Section Header Block (SHB, block type `0x0A0D0D0A`). The SHB carries a Byte-Order Magic
field (`0x1A2B3C4D` or `0x4D3C2B1A`) that governs the endianness of all subsequent numeric
fields in the section. The reader extracts the byte order, validates the pcapng major/minor
version, and establishes the parsing context passed to IDB and EPB/SPB parsers. This uses
`pcap-file` 2.0.0's `PcapNgReader`/`PcapNgParser`, which handles SHB parsing internally.

## Preconditions

1. The magic-byte probe (BC-2.01.009) has confirmed the stream begins with pcapng SHB magic.
2. The stream is positioned at byte 0 (probe did not consume bytes).
3. `pcap-file` 2.0.0's pcapng reader is available (`pcapng::PcapNgReader` or `PcapNgParser`).

## Postconditions

1. The SHB Byte-Order Magic is read and the byte order for the section is determined:
   - `0x1A2B3C4D` (big-endian BOM) → all subsequent block fields read as big-endian.
   - `0x4D3C2B1A` (little-endian BOM) → all subsequent block fields read as little-endian.
2. The pcapng major version MUST be 1; the minor version MAY be any value ≥ 0. A major
   version other than 1 returns `Err` with context identifying the unsupported version.
3. The section length field in the SHB is accepted regardless of value (it may be `-1` / all
   bits set, meaning the length is unspecified). The reader does not use section length for
   bounds checking.
4. After successful SHB parse, the reader proceeds to walk subsequent blocks (IDB, EPB, SPB).
5. A truncated SHB (fewer bytes than the minimum SHB fixed fields) returns `Err` mapped to
   E-INP-008.

## Invariants

1. Byte-order detection is done once per section. If the stream contains multiple SHBs (a
   rare but legal multi-section pcapng), each SHB resets the byte order for its section.
2. The pcapng specification requires `major_version == 1`; wirerust enforces this hard
   constraint and returns an error for non-1 major versions.
3. The SHB magic bytes (`0x0A0D0D0A`) are not themselves byte-order-dependent; they serve only
   to identify the block type. The BOM field inside the SHB body carries the endianness signal.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Little-endian pcapng (most common on x86) | BOM reads `0x4D3C2B1A`; little-endian mode selected |
| EC-002 | Big-endian pcapng (less common) | BOM reads `0x1A2B3C4D`; big-endian mode selected |
| EC-003 | Section length = `0xFFFFFFFFFFFFFFFF` (unspecified) | Accepted; reader does not use section length for bounds |
| EC-004 | Major version = 2 (future) | `Err` with "Unsupported pcapng major version: 2" context |
| EC-005 | SHB truncated at 8 bytes (missing BOM) | `Err` mapping to E-INP-008 |
| EC-006 | Multi-section pcapng (second SHB mid-file) | Second SHB resets byte order; blocks after it are parsed with new context |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Well-formed little-endian pcapng SHB | Byte order = little-endian; version (1, 0); parse continues | happy-path |
| Well-formed big-endian pcapng SHB | Byte order = big-endian; version (1, 0); parse continues | happy-path |
| SHB with section length = `0xFFFFFFFFFFFFFFFF` | Parse succeeds; section length ignored | edge-case |
| SHB with major version = 2 | `Err` containing "unsupported" | error |
| Truncated SHB (first 8 bytes only) | `Err` (E-INP-008) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Both byte orders produce identical `PcapSource` from identical logical content | unit: craft same-content pcapng in big-endian and little-endian; assert equal packet data |
| — | Truncated SHB never panics | fuzz: fuzz SHB bytes, assert no panic |
| — | Major version ≠ 1 always returns Err | unit: inject major_version=2 SHB |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- SHB parsing is the opening gate of pcapng ingestion; byte-order detection is required before any field in the file can be correctly interpreted |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-123 |
| ADR Reference | ADR-009 Decision 1 (use pcap-file 2.0.0 PcapNgReader), Decision 2 (SHB block coverage) |

## Related BCs

- BC-2.01.009 -- depends on (probe routes to this BC's parse path)
- BC-2.01.011 -- composes with (SHB establishes byte order; IDB uses it)
- BC-2.01.012 -- composes with (EPB parsing uses byte order established by SHB)

## Architecture Anchors

- `pcap_file::pcapng::PcapNgReader` (docs.rs/pcap-file/2.0.0) -- SHB parsing entry point
- pcapng spec IETF draft §Section-Header-Block: BOM field at offset 8; major/minor version at offsets 12/14
- ADR-009 Decision 2: "The reader MUST handle SHB (byte-order detection and version)"

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads stream bytes for SHB block |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O only) |
