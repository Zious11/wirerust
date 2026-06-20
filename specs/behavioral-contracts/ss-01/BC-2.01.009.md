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
  - "v1.2: Pass-2 P2a remediation — I-12: PC3/Invariant-1 restated in observable terms: 'the probe consumes no bytes; the next read returns the byte that was at offset 0' (BufReader::fill_buf peek semantics); removed unobservable 'stream positioned at byte 0' phrasing. Clarified pcapng branch also receives the un-consumed reader (RawBlock reads from the same stream). I-3 zero-packet trap: added PC6 — when a non-empty pcapng file parses cleanly but yields ZERO packets because all packet-bearing blocks were skipped (OPB / unknown block types), the reader emits a ONE-SHOT stderr notice including the skipped-block count (from BC-2.01.015 counter; no block bodies logged — SEC-007); exit code remains 0. I-11: added Test: citations per AC. — 2026-06-19"
  - "v1.1: H5-1 remediation — Postcondition 1 reworded: removed over-promise 'with at least one readable packet'; now reads 'returns Ok(PcapSource) for a valid pcapng file; packets contains one RawPacket per readable EPB/SPB in encounter order (possibly empty)'. packets.len() > 0 assertion demoted to fixture-specific test vector annotation for smb3.pcapng only, not a general postcondition. Parity with BC-2.01.002 EC-001 (empty pcapng) and OPB-only zero-packet case established. — 2026-06-19"
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
2. The stream supports non-destructive peek via `BufReader::fill_buf()` + `consume(4)`;
   wrapping any `Read` implementation in `std::io::BufReader` is sufficient, and this pattern
   works on non-seekable streams (pipes, sockets) with no seek required.
3. At least 4 bytes are available at the start of the stream.

## Postconditions

1. When the first 4 bytes are `[0x0A, 0x0D, 0x0D, 0x0A]` (pcapng SHB magic):
   the reader selects the pcapng parse path; returns `Ok(PcapSource)` for a valid pcapng
   file; `packets` contains one `RawPacket` per readable EPB/SPB in encounter order
   (possibly empty — an OPB-only or zero-data-block pcapng is valid and yields
   `packets.len() == 0`).
2. When the first 4 bytes are a valid classic-pcap magic, the classic-pcap path (`PcapReader`)
   is taken exactly as before this feature; all classic-pcap behavioral contracts remain valid.
3. The probe consumes no bytes from the underlying stream: implemented via
   `BufReader::fill_buf()` (which peeks without advancing) followed by reading 4 bytes from
   the filled buffer, then `consume(4)` only after the routing decision is committed and
   the downstream parser is ready to take over. Observable invariant: the next read on the
   stream returns the byte that was at offset 0 before the probe. The pcapng branch receives
   the same un-consumed `BufReader` reader; `RawBlock`/`next_raw_block` reads from that same
   stream starting at byte 0 (spike confirms `RawBlock.body` begins at the BOM, which is the
   first byte of the stream body).
4. When the first 4 bytes match neither format, returns `Err` with context indicating the
   unrecognized magic.
5. The `smb3.pcapng` fixture (formerly the negative-assertion fixture for BC-2.01.004)
   MUST now return `Ok(PcapSource)` with the correct packet count and link type.
6. (Zero-packet silent-trap prevention — I-3) When a non-empty pcapng file parses cleanly
   but yields ZERO packets because all packet-bearing blocks were skipped (Obsolete Packet
   Block / `Block::Unknown` block types not supported as packet sources), the reader emits a
   ONE-SHOT stderr notice including the count of skipped blocks (sourced from BC-2.01.015's
   per-block-type skip counter; no block body content is logged — SEC-007 compliance). The
   notice is emitted once per file, not once per skipped block. Exit code remains 0 (the
   file is structurally valid). This prevents a silent false-negative where a pcapng file
   appears to succeed but produces no analysis output.

## Invariants

1. The probe peeks exactly 4 bytes without consuming them; the next read on the `BufReader`
   returns the byte that was at offset 0 before the probe. This is observable in tests via
   `BufReader::fill_buf()` before and after the probe — the filled buffer must be identical.
   This formulation is testable on non-seekable streams (pipes) where `seek(SeekFrom::Start(0))`
   is not available (EC-005).
2. The classic-pcap path is structurally unchanged: after probing, the classic branch passes
   the `BufReader` — with byte 0 still unconsumed — to `PcapReader::new` exactly as before.
3. Both `from_file` and `from_pcap_reader` route through the same probe; the probe is not
   duplicated.
4. The pcapng SHB magic (`0x0A0D0D0A`) is endian-independent as a 4-byte literal; it reads
   identically in both byte orders.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `smb3.pcapng` (previously a rejection fixture) | Returns `Ok(PcapSource)` with correct packet count; link type from IDB. **Test:** `test_BC_2_01_009_smb3_pcapng_accepted` |
| EC-002 | Classic `.pcap` file passed alongside pcapng (directory mode) | Both succeed; each routes via the correct probe branch. **Test:** `test_BC_2_01_009_mixed_directory_routing` |
| EC-003 | Stream under 4 bytes (truncated header) | Returns `Err` wrapping the short-read error. **Test:** `test_BC_2_01_009_stream_under_4_bytes` |
| EC-004 | File with 4-byte content that is neither pcap nor pcapng magic | Returns `Err` with unrecognized magic context. **Test:** `test_BC_2_01_009_unrecognized_magic` |
| EC-005 | Non-seekable `Read` stream (pipe) | Probe uses `BufReader::fill_buf()` (peek, no seek) then routes; byte 0 remains the next readable byte after probe; works on non-seekable streams. **Test:** `test_BC_2_01_009_pipe_stream_probe_observable` (assert next-byte == original byte-0 after probe) |
| EC-006 | Classic nanosecond-resolution pcap (`0xA1B23C4D`) | Routed to classic-pcap path; timestamp resolution handled by existing `TsResolution` branch. **Test:** `test_BC_2_01_009_nanosecond_pcap_routing` |
| EC-007 | Non-empty pcapng with zero EPB/SPB (all OPB or Unknown blocks) | `Ok(PcapSource)` with `packets.len() == 0`; one-shot stderr notice emitted with skipped-block count; exit code 0. **Test:** `test_BC_2_01_009_zero_packet_opb_only_notice` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `tests/fixtures/smb3.pcapng` | `Ok(PcapSource)`; `packets.len() > 0` for this specific fixture (smb3.pcapng contains EPBs) — this is a fixture assertion, NOT a general postcondition | happy-path (inversion of former negative test) |
| `tests/fixtures/arp-baseline-16pkt.cap` (pcapng with `.cap` extension) | `Ok(PcapSource)` with 16 packets | happy-path |
| Classic `tests/fixtures/*.pcap` files | `Ok(PcapSource)` via classic-pcap path unchanged | regression |
| Stream of 2 bytes only | `Err` | error |
| 4 bytes `[0xDE, 0xAD, 0xBE, 0xEF]` | `Err` containing "unrecognized pcap magic" or equivalent | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Probe consumes no bytes — observable: next read returns byte-0 | unit: `fill_buf()` before probe, capture buf[0]; after probe `fill_buf()` again; assert identical; works on pipe (non-seekable). Covers I-12 observable reformulation. |
| — | Classic-pcap test suite fully green after probe insertion | regression: all prior reader tests pass |
| — | pcapng file returns `Ok` not `Err` | unit: `from_file(smb3.pcapng)` returns `Ok` |
| — | Zero-packet OPB-only pcapng emits one-shot stderr notice (PC6 / I-3) | unit: craft pcapng with only OPB blocks; assert `packets.len()==0`, assert stderr contains "skipped" with block count, assert exit code 0, assert notice emitted exactly once |

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
