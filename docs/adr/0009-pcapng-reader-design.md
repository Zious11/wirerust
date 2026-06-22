# ADR 0009: pcapng Capture-Format Reader Support

**Status:** Accepted
**Date:** 2026-06-19 (rev 13 — 2026-06-21)
**Context:** v0.9.3 (feature cycle STORY-123..128, PRs #279–#303). Adding transparent
pcapng support to the reader layer raised decisions about dependency strategy, API
selection within the existing `pcap-file` 2.0.0 crate, timestamp correctness,
block-type coverage, error-code assignment, and security hardening. This ADR records
those decisions for future contributors.

## Problem

wirerust's reader (`src/reader.rs`) previously supported only the classic libpcap
format (`0xA1B2C3D4` / `0xD4C3B2A1` magic variants). Any pcapng file was rejected
with `Err("Failed to parse pcap header")` at the `PcapReader::new` call site. This
exclusion blocked three concrete use cases:

1. `arp-baseline-16pkt.cap` — the ARP regression baseline — is a pcapng file with a
   `.cap` extension. It was rejected as "wrong magic number."
2. Rich public TLS-handshake fixtures (`dump.pcapng`, `tls12-dsb.pcapng`) are
   pcapng-only.
3. Wireshark's default save format since v1.8 (2012) is pcapng, meaning the majority
   of the shared public capture corpus was inaccessible.

## Decisions

### Decision 1: Option A — raw-block path via `pcap-file` 2.0.0 (+0 new crates)

wirerust uses `pcap-file` 2.0.0's `RawBlock` / `next_raw_block` API to walk pcapng
blocks, **not** the crate's high-level `Block::EnhancedPacket` / `EnhancedPacketBlock`
API. This adds zero new dependencies — the crate was already in the dependency tree
for classic pcap support.

The high-level API was rejected because `EnhancedPacketBlock.timestamp` hard-codes
`Duration::from_nanos(timestamp)` and never applies the `if_tsresol` IDB option.
For the pcapng specification's default resolution (microseconds, `if_tsresol = 6`
— the common case in Wireshark output) the crate produces timestamps 1000× too
large with no error or warning. The raw ticks (`ts_high`, `ts_low`) are not
recoverable from the parsed struct. The raw-block path is the only correct path.

A full hand-roll (Option C, +0 crates) was rejected as the primary implementation:
`RawBlock` provides block framing, byte-order detection, and forward-progress
guarantees for free. Option C is the escalation path if `RawBlock` proves defective.
Adding `pcap-parser` 0.17.0 (Option B) was rejected on supply-chain grounds
(duplicate major versions of `nom` and `rusticata-macros` for zero capability gain).

### Decision 2: Block-type coverage

The reader handles the following pcapng block types:

| Block | Type code | Action |
|-------|-----------|--------|
| SHB (Section Header) | `0x0A0D0D0A` | Byte-order detection, version validation |
| IDB (Interface Description) | `0x00000001` | Builds interface table; extracts `linktype` and `if_tsresol` option |
| EPB (Enhanced Packet) | `0x00000006` | Primary packet-producing block |
| SPB (Simple Packet) | `0x00000003` | Secondary packet-producing block |
| Unknown / OPB / NRB / ISB / SJE / DSB / etc. | all others | Silently skipped via `block_total_length`; block body bytes are never logged (DSB carries TLS key material) |

The Obsolete Packet Block (OPB, type `0x00000002`) is skipped. A file that contains
only OPB packet data will yield zero packets and trigger the zero-packet notice
(Decision 7).

### Decision 3: Magic-byte probe without consuming

Format detection peeks the first four bytes using `BufReader::fill_buf()` +
`consume()` without consuming them before branching to the appropriate parser path.
This is required because `from_pcap_reader<R: Read>` accepts an opaque `Read`
implementation that may not support `Seek`.

Detection is by magic-byte content, not file extension. This is necessary because
the ARP regression baseline (`arp-baseline-16pkt.cap`) is a pcapng file with a
`.cap` extension. Directory-mode traversal likewise performs content detection and
accepts any file whose first four bytes match a known magic (classic pcap variants
or `0x0A0D0D0A`), regardless of extension.

### Decision 4: Multi-IDB policy — require `linktype` agreement

A pcapng file may contain multiple IDB blocks. wirerust requires all IDB blocks in
a section to agree on `linktype`. A second IDB with a different `linktype` returns
`Err` mapping to `E-INP-011`. This preserves the single-`DataLink` field on
`PcapSource` with zero changes to `decoder.rs`, analyzers, or reporters.

### Decision 5: Single-section only; second SHB is rejected

A second SHB block is rejected immediately with `E-INP-012`. Multi-section pcapng
files must be flattened with `mergecap -w out.pcapng <file>` before use. This is a
scope decision, not a library-distrust decision. The per-section interface-table
reset described in the pcapng specification is unreachable in wirerust because the
second SHB is rejected before the interface table can be cleared.

### Decision 6: Timestamp conversion via pure-core helper

The EPB 64-bit timestamp (split as `ts_high: u32`, `ts_low: u32`) MUST be converted
to `(ts_sec: u32, ts_usecs: u32)` by the BC-2.01.014 pure-core helper, using the
`if_tsresol: u8` value extracted from the IDB. When `if_tsresol` is absent, the
caller passes `6` (10^-6, microseconds — the pcapng specification default).

All intermediate arithmetic uses saturating / checked operations:
- Base-10 (`bit7 = 0`): `ticks_per_sec = 10u64.checked_pow(e).unwrap_or(u64::MAX)`
- Base-2 (`bit7 = 1`): clamp `e` to `[0, 63]` before shift to prevent `u64` overflow
- Intermediate product `(ticks % ticks_per_sec) * 1_000_000` uses `u128` intermediate
  (overflows `u64` for base-2 `e >= 43`)
- `ts_sec` saturates to `u32::MAX`

This helper is the Kani formal-verification target for VP-025 and is load-bearing
because the crate's high-level API collapses raw ticks without applying `if_tsresol`.

The `if_tsoffset` option (IDB option code 10) is **not** extracted or applied in this
cycle; its effect on timestamps is silently ignored. This is an accepted limitation.

### Decision 7: Zero-packet notice emitted from `main.rs`

A structurally valid pcapng file that parses to EOF without error but yields zero
packets emits exactly one notice to stderr before exit 0. The notice is emitted
from `main.rs`, not from the reader, because only `main.rs` has the filename.

`PcapSource` surfaces two fields to enable the notice:
- `skipped_blocks: u32` — count of all blocks skipped during the walk
- `opb_skipped: u32` — count of skipped Obsolete Packet Blocks specifically

When `opb_skipped > 0`, the notice explicitly states that OPB packet data was not
ingested and provides a `mergecap` remediation hint. A classic-pcap empty file
(valid header, zero packets) also emits an analogous notice for consistency.

### Decision 8: IDB-after-first-packet rejection

An IDB encountered after the first packet block has been emitted (EPB or SPB) is
rejected with `E-INP-013`. wirerust does not support interleaved IDBs (an IDB
appearing mid-capture for a new NIC that came online after packets from the first NIC
were already written). Full interleaved-IDB support requires per-packet `DataLink`
threading and is deferred.

IDB error-code precedence at IDB-parse time (in evaluation order):
1. `E-INP-013` — position check (`packets_emitted > 0`); IDB body is not decoded
2. `E-INP-001` — `linktype` whitelist check (content gate; fires at IDB-parse time)
3. `E-INP-011` — multi-IDB `linktype` agreement (requires prior table state)

### Decision 9: SPB captured-length formula

For SPB on the raw-block path, the crate strips the 12-byte outer frame
(type:4 + btl:4 + trailing-btl:4), leaving `body = [original_len:4][padded_packet_data]`.
The canonical formula is:

```
spb_data_available = body.len() - 4   # strips the 4-byte original_len header
captured_len       = min(original_len, spb_data_available)
```

`snaplen` is **not** extracted or stored; it is read from IDB fixed fields and
discarded. Neither EPB nor SPB validates `captured_len` against snaplen. SPB
packets always carry `timestamp_secs: 0, timestamp_usecs: 0` (BC-2.01.013 PC3).

### Decision 10: Error-code assignment

Three tiers apply on the raw-block walk path:

| Tier | Condition | Error code |
|------|-----------|-----------|
| Tier 1 — framing rejection (crate) | `btl < 12`, `btl % 4 != 0`, or EOF before trailer | `E-INP-010` |
| Tier 2 — body-decode failure (wirerust) | Body shorter than block's required fixed-field bytes; EPB padding/bound violation; SHB bad BOM or wrong major version | `E-INP-008` |
| Tier 3 — well-formed | Body long enough and semantically valid | proceed with full decode |

Additional error codes:

| Condition | Error code |
|-----------|-----------|
| EPB/SPB with empty interface table (no IDB yet) | `E-INP-009` |
| EPB `interface_id >= table.len()` on non-empty table | `E-INP-010` |
| Multi-IDB `linktype` conflict | `E-INP-011` |
| Second SHB (multi-section file) | `E-INP-012` |
| IDB after first packet block | `E-INP-013` |
| pcapng file exceeds 4 GiB size guard | `E-INP-014` |
| Interface table exceeds 65,535-entry cap | `E-INP-015` |

The first-SHB path (`PcapNgParser::new`) behaves differently from the block-walk
path (`next_raw_block`): a btl-degenerate SHB causes the crate to surface
`InvalidField("SectionHeaderBlock: invalid magic number")` rather than the usual
framing-rejection error. wirerust string-matches this message to `E-INP-008` via
the existing invalid-magic arm. This coupling is load-bearing; the regression test
`test_BC_2_01_010_shb_btl8_maps_to_e_inp_008` pins the `E-INP-008` assertion and
must be retained.

Similarly, IDB structural failures (`reserved != 0` and `block length < 8`) are
validated by the crate inside `next_raw_block` via `InterfaceDescriptionBlock::from_slice`
before the `RawBlock` is returned. wirerust string-matches the two `InvalidField`
messages to `E-INP-008`. The regression test `test_BC_2_01_011_nonzero_reserved_e_inp_008`
pins this mapping and must not be weakened to a bare `is_err()` assertion.

### Decision 11: `PcapSource::is_pcapng` discriminant

`PcapSource` carries `is_pcapng: bool`, set at the format-branch point inside
`from_pcap_reader`. `format_zero_packet_notice` in `main.rs` reads this field to
determine whether to say "pcap file" or "pcapng file" in the notice — it does not
re-open the file. This closes a TOCTOU mislabel window where a deleted file could
produce a wrong-format notice string.

### Decision 12: Per-file isolation in directory mode

`main.rs` catches per-file reader errors and continues the directory scan rather
than aborting on the first error. Per-file errors are accumulated and reported to
stderr; exit code 1 is set at end if any file failed. This applies to all reader
error classes, not only pcapng errors.

### Decision 13: All-in-memory model; 4 GiB size guard (interim)

The pcapng path uses the same all-in-memory `Vec<RawPacket>` model as the
classic-pcap path. A streaming-reader rework is tracked as technical debt.

As an interim mitigation, `from_file` checks `fs::metadata(path)?.len()` against
`MAX_PCAPNG_FILE_BYTES = 4_294_967_296` (4 GiB) on the pcapng branch before
`read_to_end`. Files exceeding this limit are rejected with `E-INP-014`. The classic
pcap path does not have this guard. `from_pcap_reader<R: Read>` (generic path) has
no file metadata and is also ungated.

### Decision 14: Interface-table cap (defense-in-depth)

The interface table (`Vec<InterfaceInfo>`) is capped at
`MAX_INTERFACE_TABLE_ENTRIES = 65_535`. An IDB push that would exceed this limit
returns `E-INP-015`. No real-world capture approaches this count; the cap defends
against adversarially crafted files filled with minimal IDB blocks.

### Decision 15: EPB `decode_epb_body` extraction for Kani tractability

The EPB fixed-field decode is extracted into a pure function `decode_epb_body`
(`src/reader.rs`). Taking `(body: &[u8], interfaces: &[InterfaceInfo], endianness)`,
returning `anyhow::Result<RawPacket>`. This function is the Kani anchor for VP-027.
A `decode_epb_body_discriminant` twin function is used in the Kani harness for
BMC tractability (checked at `MAX_BODY = 28` bytes); twin-faithfulness is verified
by a `#[cfg(test)]` smoke test.

### Decision 17: IDB error-code precedence — four-level evaluation order

IDB blocks are evaluated in a fixed four-level precedence order. The position check
(`E-INP-013`, fires when `packets_emitted > 0`) is always first; the IDB body is
not decoded if this check fires. The interface-table cap check (`E-INP-015`,
Decision 28) is second and also skips body decode. The link-type whitelist check
(`E-INP-001`) is third, operating on the decoded `linktype` field. The multi-IDB
agreement check (`E-INP-011`) is fourth and last. This ordering ensures that checks
requiring no body parse run before more expensive body-decode operations, and that
the cap guard (Decision 28) is inserted between the position and whitelist checks
per the F6 security review.

### Decision 19: `PcapSource::is_pcapng` and `skipped_blocks` / `opb_skipped` discriminants

`PcapSource` carries two discriminant fields populated at format-branch time. The
`is_pcapng: bool` field is set inside `from_pcap_reader` at the magic-byte branch
point; `format_zero_packet_notice` in `main.rs` reads it to choose between "pcap
file" and "pcapng file" wording without re-opening the file, closing a TOCTOU
mislabel window. The `skipped_blocks: u32` and `opb_skipped: u32` fields count
skipped blocks during the pcapng block walk; they are always 0 for classic pcap.
`main.rs` reads both to shape the zero-packet notice wording (BC-2.01.009 PC6).

### Decision 20: SHB error-code remapping — `E-INP-010` to `E-INP-008` for body-decode failures

The `pcap-file` 2.0.0 crate processes IDB blocks internally inside
`next_raw_block` before returning control to wirerust. Two crate-side IDB
validation failures that are semantically Tier-2 body-decode failures are
remapped from the crate's default framing-error surface to `E-INP-008`. The
`"block length < 8"` message maps to `E-INP-008` because the IDB body is too
short to contain the required fixed fields. The `"reserved != 0"` message also
maps to `E-INP-008` as a structural body-decode failure (see Decision 24). All
other crate errors from `next_raw_block` remain Tier-1 framing rejections and
produce `E-INP-010`.

### Decision 21: `snaplen` MUST NOT be stored; `if_tsoffset` (option code 10) is silently skipped

`snaplen` is read from IDB body bytes 4–7 solely to advance the field cursor and
is immediately discarded. It is not stored in `InterfaceInfo` and no consumer for
it exists in this cycle. Adding `snaplen` to `InterfaceInfo` is explicitly
prohibited (F-M3 / BC-2.01.011 PC4). Separately, `if_tsoffset` (IDB option code
10) is silently skipped during the TLV options walk — its effect on timestamps is
not applied, and this is an accepted limitation for the current cycle.

### Decision 22: SPB block type code, captured-length formula, and zero timestamps

The SPB block type code is `0x00000003`. SPB has no `interface_id` field and always
binds to interface 0; if the interface table is empty when an SPB is encountered,
wirerust returns `E-INP-009`. The captured-length formula is the pure-core
`spb_captured_len` helper (VP-031 proptest target): `spb_data_available =
body.len() - 4; captured_len = min(original_len, spb_data_available)`. Using the
bare `body.len()` without subtracting 4 is explicitly prohibited because it counts
the `original_len` field itself and would overcount available packet bytes. SPB
carries no per-packet timestamp; every SPB packet produces `timestamp_secs: 0,
timestamp_usecs: 0` (BC-2.01.013 PC3 zero-timestamp mandate).

### Decision 23: String-coupling precedent for SHB btl-degenerate error mapping

When a btl-degenerate SHB (body that triggers `InvalidField("SectionHeaderBlock:
invalid magic number")`) is encountered on the `PcapNgParser::new` path, the crate
surfaces a string-coupled error rather than the usual `RawBlock` framing error.
wirerust string-matches this specific message to remap it to `E-INP-008` via the
existing invalid-magic arm. This mapping is the first string-coupling precedent in
the pcapng reader; it established the pattern that Decision 24 follows for IDB
structural failures. The regression test `test_BC_2_01_010_shb_btl8_maps_to_e_inp_008`
pins this assertion and must not be weakened.

### Decision 24: IDB `reserved != 0` and `block length < 8` — string-coupling to `E-INP-008`

The `pcap-file` 2.0.0 crate validates IDB structural fields (`reserved != 0` and
`block length < 8`) inside `next_raw_block` via `InterfaceDescriptionBlock::from_slice`
before returning the `RawBlock`. wirerust never receives the raw body for these
failures and therefore cannot perform its own pre-check. Both `InvalidField`
messages are string-matched in the `map_err` closure and remapped to `E-INP-008`
(following the string-coupling precedent of Decision 23). The regression test
`test_BC_2_01_011_nonzero_reserved_e_inp_008` pins the `E-INP-008` assertion
and asserts `E-INP-010` is absent; this test catches any upstream crate message
change.

### Decision 27: 4 GiB file-size gate on the pcapng branch (`MAX_PCAPNG_FILE_BYTES`, `E-INP-014`)

`from_file` checks the file size via `fstat` on the already-open file descriptor
(not a second `stat()` call on the path) after the magic-byte probe confirms
pcapng, before calling `read_to_end`. Files exceeding `MAX_PCAPNG_FILE_BYTES =
4_294_967_296` (4 GiB) are rejected with `E-INP-014`. Using `fstat` on the open
fd closes the CWE-367 TOCTOU path-substitution vector identified in the F6
security review. The `from_pcap_reader<R: Read>` generic path is not gated because
no `fs::metadata` is available for an opaque `Read` stream.

### Decision 28: Interface-table cap at 65,535 entries; guard placed before body decode

The interface table is capped at `MAX_INTERFACE_TABLE_ENTRIES = 65_535`. The cap
check is the second step in the four-level IDB precedence (Decision 17): it fires
after the position check but before body decode, so a full-table rejection does not
pay the cost of options-TLV parsing. An IDB push that would exceed the cap returns
`E-INP-015` immediately. No real-world capture approaches this count; the cap
defends against adversarially crafted files filled with minimal IDB blocks.

## Link-Type Whitelist

`{ Ethernet = 1, Raw = 101, IPv4 = 228, IPv6 = 229, LinuxSLL = 113 }` — identical
to the classic-pcap whitelist. An IDB with an unsupported `linktype` returns `E-INP-001`
at IDB-parse time.

## Supported Link Types in Both Formats

| Type | ID | Status |
|------|----|--------|
| Ethernet | 1 | Supported |
| Raw IP | 101 | Supported |
| Linux Cooked (SLL) | 113 | Supported |
| IPv4 | 228 | Supported |
| IPv6 | 229 | Supported |

## Verification Properties

| VP-ID | Tool | BC(s) | Property |
|-------|------|-------|---------|
| VP-025 | Kani | BC-2.01.014 | Timestamp helper totality — no panic, `ts_usecs` in `[0, 999_999]`, `ts_sec` saturates at `u32::MAX` |
| VP-026 | Kani | BC-2.01.010 | SHB parse safety — no panic, BOM/byte-order correct for LE/BE |
| VP-027 | Kani | BC-2.01.012 | EPB parse safety — no panic; empty-table → `E-INP-009`; OOB-non-empty → `E-INP-010`; padding/bound → `E-INP-008` |
| VP-028 | cargo-fuzz | BC-2.01.017 | pcapng reader no-panic and no-infinite-loop for arbitrary byte sequences |
| VP-029 | proptest | BC-2.01.015 | Block-walk skip always advances past `btl`; no infinite loop; terminates |
| VP-030 | proptest | BC-2.01.018 | Multi-IDB `linktype` agreement over whitelisted `DataLink` values only |
| VP-031 | proptest | BC-2.01.013 | SPB captured-len: `min(original_len, body.len() - 4)` for all `(u32, &[u8])` with `body.len() >= 4` |

## Alternatives Considered

### Option B: `pcap-parser` 0.17.0 crate (+4 new transitive crates)

Introduces duplicate major versions of `nom` (7 and 8) and `rusticata-macros` (4 and 5)
already in wirerust's dependency tree. Zero capability gain over the raw-block path.
**Rejected** on supply-chain grounds.

### Option C: Full hand-roll (+0 crates)

`RawBlock` / `next_raw_block` provides block framing, byte-order detection, and
forward-progress guarantees for free. Hand-rolling the same layer adds ~300 LOC of
first-party attack surface. **Rejected** as primary path; retained as escalation path
if `RawBlock` proves defective.

### High-level API (`Block::EnhancedPacket`)

`EnhancedPacketBlock.timestamp` hard-codes `Duration::from_nanos(timestamp)` without
applying `if_tsresol`. For Wireshark-default captures (microsecond resolution), this
produces timestamps 1000× too large with no error or warning, and the raw ticks are
not recoverable from the parsed struct. **Rejected** as incorrect by construction.

### Extension-based file filtering

The lead motivator file (`arp-baseline-16pkt.cap`) is a pcapng file with a `.cap`
extension. Extension-based filtering would permanently exclude it. **Rejected**;
content detection is the only approach consistent with the feature's motivation.

## Consequences

### Positive

- Classic pcap and pcapng are now both transparently supported at +0 new dependencies.
- `arp-baseline-16pkt.cap`, public TLS corpus pcapng files, and all modern
  Wireshark-default captures are processable without format conversion.
- Timestamps are correct for the common case (microsecond resolution, `tsresol = 6`)
  via the BC-2.01.014 pure-core helper path. The high-level API would have silently
  produced wrong timestamps.
- The `DataLink` type flows from raw IDB bytes to `PcapSource.datalink` unchanged;
  `decoder.rs`, analyzers, reassembly, dispatcher, and reporters require zero changes.
- The BC-2.01.014 helper is Kani-provable (VP-025), extending formal-verification
  coverage into SS-01 for the first time.
- Per-file isolation in directory mode (Decision 12) benefits all reader error classes,
  not only pcapng errors.

### Negative / Trade-offs

- The raw-block path requires wirerust to hand-decode EPB/SPB/IDB fields from raw bytes
  (~80–120 LOC of first-party byte-decode code above the framing layer).
- The pcapng path has ~2.0× file-size peak RSS versus ~1.5× for classic pcap, due to
  `pcap-file` 2.0.0's internal block representation held alongside the accumulating
  `Vec<RawPacket>`. Streaming refactor is deferred.
- Files that contain only Obsolete Packet Block (OPB) data will appear to contain zero
  packets.
- Multi-section pcapng files are rejected (`E-INP-012`); users must flatten with
  `mergecap`.
- Multi-NIC captures mixing link types will be rejected (`E-INP-011`).
- Two string-coupling mappings (`invalid magic number` → `E-INP-008` for SHB;
  `reserved != 0` / `block length < 8` → `E-INP-008` for IDB) are load-bearing and
  brittle to a `pcap-file` version bump. Regression tests pin these mappings.
- The 4 GiB size guard (`E-INP-014`) is an interim mitigation of the all-in-memory
  model. Files larger than 4 GiB on the pcapng path are rejected with an error.
