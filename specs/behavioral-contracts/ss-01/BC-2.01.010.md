---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - "v1.3: RC-2 flag-spelling consistency — AC-002 and EC-006: standardized remediation hint from 'mergecap -F pcapng' to 'mergecap -w out.pcapng <file>' to match ADR-009 Decision 7 form. Traceability Error Taxonomy note updated to reflect same. No normative behavior change. — 2026-06-19"
  - "v1.2: pcapng-multisection-decision correctness edits — AC-002 rationale reframed: rejection is a SCOPE decision (single-section corpus this cycle; multi-section is rare and absent from corpus), not a correctness workaround. pcap-file 2.0.0 correctly resets interface state per section (source-level verification 2026-06-19; F-06's INCONCLUSIVE premise superseded — see research/pcapng-multisection-decision.md). AC-002 and EC-006 updated to reference the E-INP-012 remediation hint (mergecap/editcap). Normative behavior (reject second SHB → E-INP-012) unchanged. — 2026-06-19"
  - "v1.1: F-06 completeness delta — EC-006 changed from 'reset byte order' (attempt) to REJECT with E-INP-012; AC added: second SHB in a single-section file is rejected; canonical test vector added for 2-section pcapng; error taxonomy cross-reference E-INP-012 added. — 2026-06-19"
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

## Acceptance Criteria

- **AC-001:** A well-formed SHB with BOM `0x4D3C2B1A` selects little-endian mode; a BOM of
  `0x1A2B3C4D` selects big-endian mode. No other BOM value is valid.
- **AC-002:** A second Section Header Block encountered anywhere after the first is REJECTED
  with `Err` containing context that maps to E-INP-012. wirerust supports single-section
  pcapng files only (scope decision for this cycle — multi-section is rare and absent from
  the intended corpus; pcap-file 2.0.0 itself handles multi-section correctly at the library
  level, but wirerust does not exercise that path). The second SHB's byte-order reset MUST
  NOT be applied before rejection. The E-INP-012 error message includes an actionable
  remediation hint directing users to `mergecap -w out.pcapng <file>` or `editcap` to
  re-save multi-section captures as single-section files (see E-INP-012 in error-taxonomy.md).
  - Canonical fixture: a crafted 2-section pcapng file (SHB₁ + IDB + EPB + SHB₂); expected
    result: reader returns `Err` after consuming SHB₁ and before yielding any packet from
    section 2.
- **AC-003:** A pcapng major version other than 1 returns `Err` immediately; no packets are
  emitted from that section.
- **AC-004:** A truncated SHB (fewer bytes than the minimum SHB fixed fields, i.e. < 28 bytes)
  returns `Err` mapping to E-INP-008; no panic.

## Invariants

1. Byte-order detection is done once per file. A second SHB constitutes a multi-section file;
   wirerust does NOT support multi-section pcapng and MUST reject it with E-INP-012 (see
   AC-002 above). This is a scope decision — multi-section pcapng is rare and absent from
   the intended corpus; pcap-file 2.0.0 handles multi-section correctly at the library level,
   but wirerust does not exercise that path. Attempting to reset byte order on a second SHB
   is NOT permitted — the rejection path fires before any reset can occur.
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
| EC-006 | Multi-section pcapng (second SHB mid-file) | `Err` mapping to E-INP-012: "pcapng multi-section files are not supported (second Section Header Block at block #<seq>) (hint: split the capture into single-section files, or re-save with 'mergecap -w out.pcapng <file>' or 'editcap' which emit single-section pcapng)"; wirerust supports single-section pcapng only (scope decision; pcap-file 2.0.0 handles multi-section correctly but wirerust does not exercise that path). No byte-order reset is attempted before rejection. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Well-formed little-endian pcapng SHB | Byte order = little-endian; version (1, 0); parse continues | happy-path |
| Well-formed big-endian pcapng SHB | Byte order = big-endian; version (1, 0); parse continues | happy-path |
| SHB with section length = `0xFFFFFFFFFFFFFFFF` | Parse succeeds; section length ignored | edge-case |
| SHB with major version = 2 | `Err` containing "unsupported" | error |
| Truncated SHB (first 8 bytes only) | `Err` (E-INP-008) | error |
| Crafted 2-section pcapng (SHB₁ + IDB + EPB + SHB₂) | `Err` (E-INP-012) after SHB₁ section; no packets from section 2 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Both byte orders produce identical `PcapSource` from identical logical content | unit: craft same-content pcapng in big-endian and little-endian; assert equal packet data |
| — | Truncated SHB never panics | fuzz: fuzz SHB bytes, assert no panic |
| — | Major version ≠ 1 always returns Err | unit: inject major_version=2 SHB |
| — | Second SHB in any stream always returns E-INP-012 Err | unit: craft 2-section pcapng; assert Err contains "multi-section" / E-INP-012 context |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- SHB parsing is the opening gate of pcapng ingestion; byte-order detection is required before any field in the file can be correctly interpreted |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-123 |
| ADR Reference | ADR-009 Decision 1 (use pcap-file 2.0.0 PcapNgReader), Decision 2 (SHB block coverage) |
| Error Taxonomy | E-INP-008 (truncated SHB), E-INP-012 (multi-section SHB reject — scope decision; pcap-file 2.0.0 handles multi-section correctly; wirerust rejects as out-of-scope; message includes `mergecap -w out.pcapng <file>` / editcap remediation hint per ADR-009 Decision 7) |

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
