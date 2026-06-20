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
supersedes: BC-2.01.004
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.009: Accept pcapng Format: Transparent Detection via Magic-Byte Probe

## Description

`PcapSource::from_pcap_reader<R: Read>` MUST peek the first four bytes of the input stream
without consuming them before branching to the appropriate parser. If the magic bytes match
the pcapng Section Header Block constant (`0x0A0D0D0A`), the reader routes to the pcapng
parse path (BC-2.01.010 onward) and returns a `PcapSource` with normalized packets. If the
bytes match a classic-pcap magic (`0xA1B2C3D4`, `0xD4C3B2A1`, `0xA1B23C4D`, `0x4D3CB2A1`),
the existing classic-pcap path is taken unchanged. This BC supersedes BC-2.01.004, inverting
its postconditions from rejection to acceptance.

## Preconditions

1. A readable byte stream is passed to `PcapSource::from_pcap_reader`.
2. The stream supports non-destructive peek (wrapping in `std::io::BufReader` is sufficient).
3. At least 4 bytes are available at the start of the stream.

## Postconditions

1. When the first 4 bytes are `[0x0A, 0x0D, 0x0D, 0x0A]` (pcapng SHB magic):
   the reader selects the pcapng parse path; returns `Ok(PcapSource)` for a valid pcapng
   file with at least one readable packet.
2. When the first 4 bytes are a valid classic-pcap magic, the classic-pcap path (`PcapReader`)
   is taken exactly as before this feature; all classic-pcap behavioral contracts remain valid.
3. The peek operation MUST NOT advance the stream position. After the probe, the stream
   must still be positioned at byte 0.
4. When the first 4 bytes match neither format, returns `Err` with context indicating the
   unrecognized magic.
5. The `smb3.pcapng` fixture (formerly the negative-assertion fixture for BC-2.01.004)
   MUST now return `Ok(PcapSource)` with the correct packet count and link type.

## Invariants

1. The probe reads exactly 4 bytes and consumes none of them from the underlying stream.
2. The classic-pcap path is structurally unchanged: after probing, the classic branch passes
   the stream — still positioned at byte 0 — to `PcapReader::new` exactly as before.
3. Both `from_file` and `from_pcap_reader` route through the same probe; the probe is not
   duplicated.
4. The pcapng SHB magic (`0x0A0D0D0A`) is endian-independent as a 4-byte literal; it reads
   identically in both byte orders.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `smb3.pcapng` (previously a rejection fixture) | Returns `Ok(PcapSource)` with correct packet count; link type from IDB |
| EC-002 | Classic `.pcap` file passed alongside pcapng (directory mode) | Both succeed; each routes via the correct probe branch |
| EC-003 | Stream under 4 bytes (truncated header) | Returns `Err` wrapping the short-read error |
| EC-004 | File with 4-byte content that is neither pcap nor pcapng magic | Returns `Err` with unrecognized magic context |
| EC-005 | Non-seekable `Read` stream (pipe) | Probe uses `BufReader::fill_buf()` + `consume()`; works on non-seekable streams |
| EC-006 | Classic nanosecond-resolution pcap (`0xA1B23C4D`) | Routed to classic-pcap path; timestamp resolution handled by existing `TsResolution` branch |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `tests/fixtures/smb3.pcapng` | `Ok(PcapSource)` with `packets.len() > 0` | happy-path (inversion of former negative test) |
| `tests/fixtures/arp-baseline-16pkt.cap` (pcapng with `.cap` extension) | `Ok(PcapSource)` with 16 packets | happy-path |
| Classic `tests/fixtures/*.pcap` files | `Ok(PcapSource)` via classic-pcap path unchanged | regression |
| Stream of 2 bytes only | `Err` | error |
| 4 bytes `[0xDE, 0xAD, 0xBE, 0xEF]` | `Err` containing "unrecognized pcap magic" or equivalent | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Probe does not advance stream position | unit: `fill_buf()` then verify stream still starts at byte 0 |
| — | Classic-pcap test suite fully green after probe insertion | regression: all prior reader tests pass |
| — | pcapng file returns `Ok` not `Err` | unit: `from_file(smb3.pcapng)` returns `Ok` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- magic-byte format detection is the first gate of the ingestion pipeline; routing to the correct parser is a format-detection concern within CAP-01's scope |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-123 |
| ADR Reference | ADR-009 Decision 5 (magic-byte probe discipline), Decision 6 (BC-2.01.004 retirement) |

## Related BCs

- BC-2.01.004 -- supersedes (this BC inverts BC-2.01.004's rejection postconditions)
- BC-2.01.010 -- depends on (pcapng branch delegates SHB parsing to BC-2.01.010)
- BC-2.01.008 -- composes with (from_file wraps from_pcap_reader; probe is inside from_pcap_reader)

## Architecture Anchors

- `src/reader.rs` -- `PcapSource::from_pcap_reader`: magic-byte probe insertion point
- `src/reader.rs` -- `BufReader::fill_buf()` + `consume(4)` pattern for non-destructive peek
- ADR-009 Decision 5: "peek the first four bytes without consuming them before branching"

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads stream (BufReader peek); no writes |
| **Global state access** | none |
| **Deterministic** | yes -- same bytes always produce same routing decision |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O only) |
