---
document_type: behavioral-contract
level: L3
version: "1.7"
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
  - "v1.7: Pass-7 remediation U1 (F-3) — PC6 generic skip segment rule made explicit: the notice parenthetical has two independently gated segments. (1) Generic segment: '(skipped_blocks - opb_skipped) block(s) skipped as unsupported' — emitted ONLY when (skipped_blocks - opb_skipped) > 0. (2) OPB clause: '(includes <opb_skipped> obsolete Packet Block(s) whose data was not analyzed; re-save with mergecap)' — emitted ONLY when opb_skipped > 0. So: Case D (skipped_blocks=1, opb_skipped=1) → no generic segment (1-1=0), OPB clause '1'. Case E (skipped_blocks=3, opb_skipped=1) → generic segment '2', OPB clause '1'. When both segments are present they appear space-separated in one parenthetical or as two adjacent parentheticals. Neither segment is emitted when its gate is zero. — 2026-06-20"
  - "v1.6: Pass-6 remediation T4 (ADR-009 rev 9) — (F-M4) Added EC-010: SHB-only pcapng (no IDB, no packet blocks, no skipped blocks) is structurally valid per pcapng spec. Yields Ok(PcapSource) with packets.len()==0, skipped_blocks==0, opb_skipped==0. The zero-packet notice IS emitted (canonical format, no parenthetical — skipped_blocks==0 and opb_skipped==0). Exit 0. An SHB alone constitutes a valid empty section; it is NOT an error. Added test vector and VP for this case. — 2026-06-20"
  - "v1.5: Pass-5 remediation S3 (ADR-009 rev 8) — (M-5) Rewrote PC6: emission moves from reader to main.rs; PcapSource now exposes skipped_blocks:u32 and opb_skipped:u32 (OPB sub-count); from_pcap_reader surfaces these; main.rs reads them and emits the notice. Canonical Decision 19 format: 'notice: <filename>: 0 packets read from <pcap|pcapng> file'; when opb_skipped>0 append '(includes N obsolete Packet Blocks whose data was not analyzed; re-save with mergecap)'. Classic empty pcap also triggers notice (classic/pcapng symmetry). Removed old 'reader emits / wirerust:' wording from PC6. (M-1) Deleted Precondition 3 ('at least 4 bytes available') — contradicts EC-003 (graceful Err on <4 bytes); <4-byte case is a runtime condition handled by postcondition, NOT an input precondition. (M-4) Added AC-007 pinning the BufReader wrap site: from_pcap_reader MUST internally wrap its R:Read in BufReader and feed the SAME BufReader to both fill_buf and downstream parsers. — 2026-06-20"
  - "v1.4: Pass-4 remediation R3a (ADR-009 rev 7) — (H-2 CRITICAL implementability) Removed all consume(4) references: the probe is PEEK-ONLY via BufReader::fill_buf() with ZERO consumption; BOTH branches (classic PcapReader AND pcapng RawBlock) receive the FULL un-consumed stream starting at byte 0. Implementing consume(4) would break every file — removed from Description and Precondition 2 and Postcondition 3. (Decision 19 / M-4) Fixed PC6 citation: 'Decision 17' corrected to 'Decision 19'. (H-4) Added explicit disambiguation rule in Postcondition 3 tail: a file is 'structurally-valid zero-packet' (notice, exit 0) IFF it parses to EOF with no error AND packets.len()==0; an EPB/SPB before any IDB is an ERROR (E-INP-009, exit 1), NOT a zero-packet success. — 2026-06-20"
  - "v1.3: Pass-3 remediation Burst Q3 (ADR-009 rev 6) — (M-3) PC6 broadened: the one-shot stderr notice now fires whenever a STRUCTURALLY VALID pcapng yields zero packets — regardless of skipped_blocks count (fixes silent-trap for IDB-only / SHB-only valid files where skipped_blocks==0). The skipped-block count is included in the message when >0; otherwise a generic 'wirerust: 0 packets read from a valid pcapng file' notice is emitted. Gating condition is 'valid file + zero packets', not 'zero packets AND skipped_blocks>0'. EC-007 updated to reflect broadened condition. VP updated accordingly. — 2026-06-19"
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
without consuming them before branching to the appropriate parser. The probe is PEEK-ONLY via
`BufReader::fill_buf()` with ZERO consumption — the stream is not advanced after the peek;
the byte at offset 0 remains the next readable byte on both the pcapng branch and the
classic-pcap branch. If the magic bytes match the pcapng Section Header Block constant
(`0x0A0D0D0A`), the reader routes to the pcapng parse path (BC-2.01.010 onward) and returns
a `PcapSource` with normalized packets; the pcapng parser (`RawBlock`/`next_raw_block`)
receives the full un-consumed stream starting at byte 0 (the SHB block_type `0x0A0D0D0A`
occupies bytes 0-3). If the bytes match a classic-pcap magic (`0xA1B2C3D4`, `0xD4C3B2A1`,
`0xA1B23C4D`, `0x4D3CB2A1`), the existing classic-pcap path is taken unchanged; `PcapReader`
also receives the full un-consumed stream starting at byte 0 (the classic magic occupies
bytes 0-3). This BC supersedes BC-2.01.004, inverting its postconditions from rejection to
acceptance.

## Preconditions

1. A readable byte stream is passed to `PcapSource::from_pcap_reader`.
2. The stream supports non-destructive peek via `BufReader::fill_buf()`;
   wrapping any `Read` implementation in `std::io::BufReader` is sufficient. The probe reads
   4 bytes from the filled buffer WITHOUT calling `consume()` — the stream position is
   unchanged after the peek. This pattern works on non-seekable streams (pipes, sockets)
   with no seek required.

## Postconditions

1. When the first 4 bytes are `[0x0A, 0x0D, 0x0D, 0x0A]` (pcapng SHB magic):
   the reader selects the pcapng parse path; returns `Ok(PcapSource)` for a valid pcapng
   file; `packets` contains one `RawPacket` per readable EPB/SPB in encounter order
   (possibly empty — an OPB-only or zero-data-block pcapng is valid and yields
   `packets.len() == 0`).
2. When the first 4 bytes are a valid classic-pcap magic, the classic-pcap path (`PcapReader`)
   is taken exactly as before this feature; all classic-pcap behavioral contracts remain valid.
3. The probe consumes NO bytes from the underlying stream: implemented via
   `BufReader::fill_buf()` (which fills the internal buffer without advancing the read
   position) followed by reading 4 bytes from the filled buffer slice — NO `consume()` call
   is made. Observable invariant: the next read on the `BufReader` returns the byte that was
   at offset 0 before the probe. BOTH downstream parsers receive the full un-consumed
   `BufReader`: the pcapng branch passes the reader directly to `RawBlock`/`next_raw_block`,
   which reads from byte 0 (the SHB block_type `0x0A0D0D0A`); the classic-pcap branch
   passes the reader directly to `PcapReader::new`, which reads from byte 0 (the classic
   magic). Disambiguation rule (H-4): a pcapng file is classified as "structurally-valid
   zero-packet" (triggers the notice in PC6, exit 0) if and only if it parses to EOF with
   NO error AND `packets.len() == 0`. An EPB or SPB encountered before any IDB has been
   parsed is a structural error (E-INP-009, exit 1) — it is NOT classified as zero-packet
   success; "parses to EOF with no error" excludes this case.
4. When the first 4 bytes match neither format, returns `Err` with context indicating the
   unrecognized magic.
5. The `smb3.pcapng` fixture (formerly the negative-assertion fixture for BC-2.01.004)
   MUST now return `Ok(PcapSource)` with the correct packet count and link type.
6. (Zero-packet silent-trap prevention — I-3 / M-3 broadened / Decision 19) When a
   STRUCTURALLY VALID file (pcapng OR classic pcap) parses cleanly but yields ZERO packets,
   **main.rs** emits a ONE-SHOT stderr notice. The trigger condition is **"valid file + zero
   packets"** regardless of the `skipped_blocks` count.

   **Emission architecture (M-5):**
   - `from_pcap_reader` is a LIBRARY function returning `Result<PcapSource, ...>` with no
     access to the filename. It does NOT emit anything to stderr.
   - `PcapSource` MUST expose two public counter fields populated by `from_pcap_reader`:
     - `skipped_blocks: u32` — total blocks entering the skip arm (any unknown block type).
     - `opb_skipped: u32` — sub-count: blocks skipped specifically because they are obsolete
       Packet Blocks (OPB, type `0x00000002`).
   - After `from_pcap_reader` returns `Ok(source)`, **main.rs** checks:
     `if source.packets.is_empty()` → emit the notice to stderr, then continue (exit 0).
   - main.rs has the filename (from CLI argument), so the notice is emitted with the filename.

   **Canonical notice format (Decision 19):**
   ```
   notice: <filename>: 0 packets read from <pcap|pcapng> file
   ```
   where `<pcap|pcapng>` is `pcap` for classic-pcap inputs and `pcapng` for pcapng inputs.

   The notice MAY be followed by one or two parenthetical segments, each independently gated:

   **Generic skip segment** — emitted ONLY when `(source.skipped_blocks - source.opb_skipped) > 0`:
   ```
   (G block(s) skipped as unsupported)
   ```
   where `G = source.skipped_blocks - source.opb_skipped` (the count of non-OPB skipped blocks).
   When `G == 0` this segment is OMITTED entirely — an OPB-only skip file has no generic segment.

   **OPB clause** — emitted ONLY when `source.opb_skipped > 0`:
   ```
   (includes N obsolete Packet Block(s) whose data was not analyzed; re-save with mergecap)
   ```
   where `N = source.opb_skipped`.

   When both segments are emitted they appear space-separated after the base notice line.
   When neither gate is satisfied (both `G == 0` and `opb_skipped == 0`) no parenthetical
   is appended.

   **Derivation examples:**
   - Case D: `skipped_blocks=1, opb_skipped=1` → G=0 → no generic segment; OPB clause "1".
     Full notice: `"notice: <f>: 0 packets read from pcapng file (includes 1 obsolete Packet Block(s) whose data was not analyzed; re-save with mergecap)"`
   - Case E: `skipped_blocks=3, opb_skipped=1` → G=2 → generic segment "2"; OPB clause "1".
     Full notice: `"notice: <f>: 0 packets read from pcapng file (2 block(s) skipped as unsupported) (includes 1 obsolete Packet Block(s) whose data was not analyzed; re-save with mergecap)"`

   **Classic pcap symmetry:** A structurally valid EMPTY classic pcap file (zero packet
   records, valid global header) ALSO triggers this notice with `<pcap>` in the format.
   This ensures pcap and pcapng behave identically on the zero-packet path.

   No block body content is logged in either message variant (SEC-007 compliance). The
   notice is emitted once per file, not once per skipped block. Exit code remains 0 (the
   file is structurally valid). This prevents a silent false-negative where a file appears
   to succeed but produces no analysis output.

## Acceptance Criteria

- **AC-007 (BufReader wrap-site, M-4):** `from_pcap_reader<R: Read>` MUST internally wrap
  its `R` argument in `std::io::BufReader` before performing the magic-byte probe or calling
  any downstream parser. The SAME `BufReader<R>` instance MUST be passed to both:
  - `BufReader::fill_buf()` for the peek (zero consumption), AND
  - The downstream parser on whichever branch is taken (`PcapReader::new` on the classic
    path; `next_raw_block` on the pcapng path).
  Double-wrapping (if the caller already passes a `BufReader`) is acceptable and idempotent
  — the outer buffer will still satisfy `fill_buf()` semantics. The wrap MUST NOT be
  deferred to the caller: `from_pcap_reader` is responsible for ensuring buffering is present
  regardless of what `R: Read` the caller provides (including unbuffered types such as
  `std::io::Cursor<&[u8]>` or `std::fs::File`).
  **Test:** `test_BC_2_01_009_unbuffered_read_routes_correctly` — pass an unbuffered
  `Cursor<&[u8]>` containing a valid pcapng SHB; assert `Ok(PcapSource)` with correct
  routing; this test would panic or misroute if the BufReader wrap is absent.

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
| EC-007 | Non-empty pcapng with zero EPB/SPB (all OPB blocks) | `Ok(PcapSource)` with `packets.len() == 0`; `source.opb_skipped > 0`; `source.skipped_blocks == source.opb_skipped` (OPB increments both); G=(skipped_blocks - opb_skipped)=0 → no generic segment; main.rs emits notice: `"notice: <filename>: 0 packets read from pcapng file (includes N obsolete Packet Block(s) whose data was not analyzed; re-save with mergecap)"`; exit code 0. **Test:** `test_BC_2_01_009_zero_packet_opb_only_notice` |
| EC-008 | Valid pcapng with IDB and SHB but zero EPB/SPB and zero skipped blocks (e.g., capture session opened but no packets recorded) | `Ok(PcapSource)` with `packets.len() == 0`; `source.skipped_blocks == 0`; main.rs emits notice: `"notice: <filename>: 0 packets read from pcapng file"`; exit code 0. **Test:** `test_BC_2_01_009_zero_packet_idb_only_no_skips_notice` |
| EC-009 | Valid EMPTY classic pcap (valid global header, zero packet records) | `Ok(PcapSource)` with `packets.len() == 0`; main.rs emits notice: `"notice: <filename>: 0 packets read from pcap file"`; exit code 0. Classic/pcapng symmetry. **Test:** `test_BC_2_01_009_zero_packet_empty_classic_pcap_notice` |
| EC-010 | SHB-only pcapng (no IDB, no packet blocks, no blocks of any kind after the SHB) — degenerate but structurally valid file (F-M4) | `Ok(PcapSource)` with `packets.len() == 0`; `source.skipped_blocks == 0`; `source.opb_skipped == 0`. main.rs emits notice: `"notice: <filename>: 0 packets read from pcapng file"` (no parenthetical segment — `skipped_blocks == 0` and `opb_skipped == 0`); exit code 0. An SHB alone is a valid empty pcapng section per spec §4.1; there are no blocks to skip and no packets to read. **Test:** `test_BC_2_01_009_shb_only_zero_packet_notice` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `tests/fixtures/smb3.pcapng` | `Ok(PcapSource)`; `packets.len() > 0` for this specific fixture (smb3.pcapng contains EPBs) — this is a fixture assertion, NOT a general postcondition | happy-path (inversion of former negative test) |
| `tests/fixtures/arp-baseline-16pkt.cap` (pcapng with `.cap` extension) | `Ok(PcapSource)` with 16 packets | happy-path |
| Classic `tests/fixtures/*.pcap` files | `Ok(PcapSource)` via classic-pcap path unchanged | regression |
| Stream of 2 bytes only | `Err` (short-read — gracefully handled; NOT an input precondition violation) | error |
| 4 bytes `[0xDE, 0xAD, 0xBE, 0xEF]` | `Err` containing "unrecognized pcap magic" or equivalent | error |
| Unbuffered `Cursor<&[u8]>` with valid pcapng SHB | `Ok(PcapSource)` with correct routing (proves internal BufReader wrap) | regression (AC-007) |
| Valid EMPTY classic pcap (no packet records) | `Ok(PcapSource)` with `packets.len()==0`; `source.skipped_blocks==0`; `source.opb_skipped==0` | edge-case (EC-009) |
| SHB-only pcapng (28-byte file: SHB only, no IDB, no subsequent blocks) | `Ok(PcapSource)` with `packets.len()==0`; `source.skipped_blocks==0`; `source.opb_skipped==0`; main.rs emits `"notice: <filename>: 0 packets read from pcapng file"` (no parenthetical); exit 0 | edge-case (EC-010, F-M4) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Probe consumes no bytes — observable: next read returns byte-0 | unit: `fill_buf()` before probe, capture buf[0]; after probe `fill_buf()` again; assert identical; works on pipe (non-seekable). Covers I-12 observable reformulation. |
| — | Classic-pcap test suite fully green after probe insertion | regression: all prior reader tests pass |
| — | pcapng file returns `Ok` not `Err` | unit: `from_file(smb3.pcapng)` returns `Ok` |
| — | Zero-packet OPB-only pcapng: PcapSource.opb_skipped>0; main.rs emits one-shot notice with OPB appendage (PC6 / M-3 / M-5) | integration (main.rs): craft pcapng with only OPB blocks; assert `packets.len()==0`; assert `source.opb_skipped>0`; assert stderr from main.rs contains canonical `"notice: <filename>: 0 packets read from pcapng file (includes N obsolete Packet Blocks...")`; assert exit code 0; assert from_pcap_reader itself emits NOTHING to stderr |
| — | Valid IDB-only pcapng (zero skipped blocks): PcapSource.skipped_blocks==0; main.rs emits one-shot notice without OPB appendage (PC6 / M-3 / M-5) | integration (main.rs): craft pcapng with SHB + IDB only; assert `packets.len()==0`; assert `source.skipped_blocks==0`; assert stderr from main.rs contains `"notice: <filename>: 0 packets read from pcapng file"` (no OPB segment); assert exit code 0 |
| — | Valid EMPTY classic pcap: main.rs emits one-shot notice with pcap variant (PC6 / M-5 classic symmetry) | integration (main.rs): valid global-header-only .pcap file; assert `packets.len()==0`; assert stderr from main.rs contains `"notice: <filename>: 0 packets read from pcap file"`; assert exit code 0 |
| — | from_pcap_reader accepts unbuffered Read (AC-007 BufReader wrap-site) | unit: pass `Cursor<&[u8]>` (unbuffered) with valid pcapng SHB; assert `Ok(PcapSource)` routes to pcapng path; no panic or misroute **Test:** `test_BC_2_01_009_unbuffered_read_routes_correctly` |
| — | SHB-only pcapng emits notice, skipped_blocks==0, opb_skipped==0 (EC-010 / F-M4) | integration (main.rs): craft 28-byte pcapng with SHB only (no IDB, no subsequent blocks); assert `Ok(PcapSource)`, `packets.len()==0`, `source.skipped_blocks==0`, `source.opb_skipped==0`; assert stderr from main.rs contains `"notice: <filename>: 0 packets read from pcapng file"` WITHOUT any parenthetical segment; assert no "skipped" or "obsolete" substring; assert exit code 0. **Test:** `test_BC_2_01_009_shb_only_zero_packet_notice` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- magic-byte format detection is the first gate of the ingestion pipeline; routing to the correct parser is a format-detection concern within CAP-01's scope |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-123 |
| ADR Reference | ADR-009 Decision 5 (magic-byte probe discipline), Decision 6 (BC-2.01.004 retirement), Decision 19 (canonical zero-packet notice format; emission in main.rs; PcapSource.skipped_blocks/opb_skipped fields) — rev 9 (F-M4: SHB-only file is structurally valid, yields skipped_blocks==0, notice emitted, exit 0) |

## Related BCs

- BC-2.01.004 -- supersedes (this BC inverts BC-2.01.004's rejection postconditions)
- BC-2.01.010 -- depends on (pcapng branch delegates SHB parsing to BC-2.01.010)
- BC-2.01.008 -- composes with (from_file wraps from_pcap_reader; probe is inside from_pcap_reader)

## Architecture Anchors

- `src/reader.rs` -- `PcapSource::from_pcap_reader`: magic-byte probe insertion point
- `src/reader.rs` -- `BufReader::fill_buf()` PEEK-ONLY pattern (no `consume()` call) for non-destructive probe
- ADR-009 Decision 5: "peek the first four bytes without consuming them before branching"

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads stream (BufReader peek); no writes |
| **Global state access** | none |
| **Deterministic** | yes -- same bytes always produce same routing decision |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O only) |
