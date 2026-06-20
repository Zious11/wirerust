---
document_type: adr
adr_id: ADR-009
status: proposed
date: 2026-06-19
subsystems_affected:
  - SS-01
supersedes: null
superseded_by: null
---

# ADR-009: pcapng Capture-Format Reader Support (rev 4)

> **One-per-file:** Each architectural decision lives in its own file.
> Filename convention: `ADR-NNN-<short-name>.md` (e.g., `ADR-001-rust-dispatcher.md`)
> ADR IDs are sequential 3-digit (`ADR-001`, `ADR-002`, ...). Once issued, never renumber.
> Lifecycle: `proposed` -> `accepted` -> (optional) `superseded` or `deprecated`.
> Frontmatter `subsystems_affected` is an array of `SS-NN` identifiers from ARCH-INDEX
> Subsystem Registry. `supersedes` / `superseded_by` link to other ADR IDs (e.g., `ADR-007`).

## Context

wirerust's reader (`src/reader.rs`) currently supports only the classic libpcap
format (magic bytes `0xA1B2C3D4` / `0xD4C3B2A1` and their nanosecond variants).
Any pcapng file — regardless of file extension — is rejected with
`Err("Failed to parse pcap header")` at the `PcapReader::new` call site. The module
doc header (`src/reader.rs:5`) documents this as an intentional exclusion ("pcapng
is intentionally NOT supported here — see LESSON-P0.02"), and the directory-glob
in `src/main.rs` enforced the exclusion by filtering `*.pcapng` files out before
they reach the reader (LESSON-P0.02 / NFR-VIO-002).

Three concrete unblocks drive this feature cycle. `arp-baseline-16pkt.cap` — the
ARP regression baseline captured on PacketLife — is stored as pcapng with a `.cap`
extension; it is currently rejected with "wrong magic number." The richest public
TLS-handshake fixtures (Wireshark `dump.pcapng`, `tls12-dsb.pcapng`) are
pcapng-only and are entirely blocked. And Wireshark's default save format since
v1.8 (2012) is pcapng, meaning the majority of the shared public capture corpus
is now inaccessible. The cumulative impact is that SS-01 is stranded on a minority
format while all modern capture tooling defaults to the other.

The bcf BC-2.01.004 ("Reject pcapng-Format Input at Reader Level", SS-01) is the
normative statement of the current exclusion. It specifies `Err("Failed to parse
pcap header")` as its PC1 postcondition and "no packets read" as PC2. This ADR
records the decision to retire and invert BC-2.01.004: what was rejection becomes
acceptance. The test `test_BC_2_01_004_rejects_pcapng` will be rewritten from a
negative-assertion test into a positive-acceptance test as a direct consequence.

The pcapng specification (IETF draft `draft-ietf-opsawg-pcapng`) defines four
block types relevant to wirerust's use case: Section Header Block (SHB, type
`0x0A0D0D0A`), Interface Description Block (IDB, type `0x00000001`), Enhanced
Packet Block (EPB, type `0x00000006`), and Simple Packet Block (SPB, type
`0x00000003`). The SHB carries a Byte-Order Magic field (`0x1A2B3C4D`) that
governs endianness for all subsequent fields. The IDB carries the per-interface
`linktype` (same DataLink enum wirerust already uses) and an optional `if_tsresol`
TLV option (code 9) that defines the timestamp resolution exponent. EPBs carry a
64-bit timestamp split as (ts_high, ts_low) interpreted against the resolution of
their IDB. Unknown block types carry a 32-bit block-total-length field enabling
safe skip without understanding their content.

A dependency decision is the primary gate before implementation. `pcap-file` 2.0.0
is wirerust's existing pcap dependency (Cargo.toml:29, locked at 2.0.0); it already
ships a full pcapng reader module (`PcapNgReader`, `PcapNgParser`, typed `Block`
enum, `InterfaceDescriptionBlock { linktype: DataLink, snaplen, options }`,
`IfTsResol`/`IfTsOffset` option variants) that was never used. This makes the
dependency decision non-obvious: two of the three candidate strategies add zero
new crates, and the three strategies differ sharply in implementation risk.

## Decision

wirerust will transparently detect and read pcapng capture files by magic-byte
auto-detection at the reader layer (`src/reader.rs`), then deliver normalized
`PcapSource { packets: Vec<RawPacket>, datalink: DataLink }` output to the existing
analyzer pipeline unchanged. Detection MUST peek the first four bytes without
consuming them before branching to the appropriate parser path.

**Decision 1 — Parser: Option A via the RAW-BLOCK path (+0 new crates).** [REV 4
ARCHITECTURAL PIVOT — supersedes the rev 2/rev 3 high-level-API decision.] wirerust
will use `pcap-file` 2.0.0's `RawBlock` / `next_raw_block` API to walk pcapng blocks,
NOT the high-level `Block::EnhancedPacket` / `EnhancedPacketBlock` API. This is still
Option A (+0 new crates): the crate is already compiled into every wirerust build and
provides block framing, byte-order handling, and forward-progress guarantees via
`RawBlock::from_slice`. wirerust owns the per-block dispatch and all field decoding
above the raw framing layer.

Rationale for the pivot (source: pcap-file 2.0.0 API spike, `.factory/research/pcap-file-2.0.0-api-spike.md`):

  - `EnhancedPacketBlock.timestamp` is a `Duration` computed via
    `Duration::from_nanos(timestamp)` — the crate **hard-codes nanosecond resolution**
    and NEVER applies `if_tsresol` (confirmed at `enhanced_packet.rs:46-48,65`). The
    `IfTsResol(u8)` option is parsed onto the IDB but is dead data w.r.t. timestamp
    conversion (`interface_description.rs:179-184` — no application site).
  - For the pcapng specification default resolution of 10^-6 (microseconds, tsresol=6),
    the crate is silently WRONG: it treats microsecond ticks as nanoseconds, yielding
    timestamps 1000× too large. This is the common case — Wireshark's default.
  - By the time `EnhancedPacketBlock` is returned, the raw `(timestamp_high, timestamp_low)`
    ticks have been collapsed into a `Duration` under a false ns assumption. The raw ticks
    are not recoverable from the parsed struct. The BC-2.01.014 pure-core helper requires
    raw ticks as input; it cannot be fed from `EnhancedPacketBlock::timestamp`.
  - Therefore the raw-block path is the ONLY correct path. `RawBlock` exposes the block's
    raw bytes; wirerust reads the EPB header fields directly (interface_id: u32, ts_high: u32,
    ts_low: u32, captured_len: u32, original_len: u32) and passes `(ts_high, ts_low,
    if_tsresol)` to the BC-2.01.014 helper. This path is both correct and formally
    verifiable.
  - Option C (full hand-roll, +0 crates) is REJECTED as the primary path: `RawBlock`
    / `next_raw_block` provides block framing, byte-order detection via `BlockType`
    endian dispatch, and the `initial_len` / trailer-length consistency check, all of
    which wirerust gets for free. Hand-rolling these adds first-party attack surface
    with no capability gain. Option C remains the escalation path if `RawBlock` proves
    defective in practice.

BC-2.01.014 consequence: the timestamp helper is LOAD-BEARING, not redundant. The
H-1/SEC-001/SEC-006 overflow guards (saturating arithmetic throughout) are real fixes
that the Kani proof (VP-025) will verify.

**Decision 2 — Block coverage via RawBlock dispatch.** The reader MUST handle:

  - SHB (`block_type == 0x0A0D0D0A`): byte-order detection and version validation.
    On the raw-block path, byte-order is determined from the Byte-Order Magic within
    the SHB body before subsequent blocks are read.
  - IDB (`block_type == 0x00000001`): per-interface `linktype` (4-byte field) and
    `if_tsresol` option (TLV option code 9, 1-byte value) extracted from the raw
    block body. Interface table is a `Vec<InterfaceInfo>` (O(1) access by 0-based
    interface_id).
  - EPB (`block_type == 0x00000006`): fixed fields read from raw body (interface_id:4,
    ts_high:4, ts_low:4, captured_len:4, original_len:4 = 20 bytes of fixed overhead,
    THEN packet data, THEN padding). `EPB_FIXED_OVERHEAD_BYTES = 20` (body-relative,
    i.e., not counting the outer 12-byte block header). Validation: `captured_len <=
    block_total_length - 32` (12-byte outer header + 20-byte body fixed fields). Guard
    MUST precede any data allocation (SEC-004). `captured_len` is NOT validated against
    `snaplen` by the crate; wirerust may enforce snaplen parity but is not required to
    for this cycle.
  - SPB (`block_type == 0x00000003`): fixed fields are `original_len: u32` (4 bytes,
    body-relative overhead = 4 bytes). `captured_len` is implicitly
    `min(original_len, snaplen)` and must be derived by the caller; padding bytes are
    included in the raw `data` slice and MUST be stripped to `captured_len` bytes.
    `SPB_FIXED_OVERHEAD_BYTES = 4` (body-relative). Crate does NOT validate or clamp
    to snaplen; caller owns snaplen semantics.
  - Unknown block types (all other `block_type` values, including OPB `0x00000002`,
    NRB `0x00000004`, ISB `0x00000005`, SJE `0x00000009`, DSB `0x0000000A`, and
    anything else): silently skipped using the `block_total_length` field to advance
    the cursor. No warning, no error. Block body bytes MUST NOT be logged, printed, or
    included in any diagnostic output at any log level (DSB carries TLS key material).

The `Block` enum on the high-level API has 9 variants (SectionHeader, InterfaceDescription,
Packet, SimplePacket, NameResolution, InterfaceStatistics, EnhancedPacket,
SystemdJournalExport, Unknown) and is NOT `#[non_exhaustive]` (confirmed: no
`non_exhaustive` attribute anywhere in the crate source). There is NO
`DecryptionSecrets` variant — DSB (type `0x0A`) arrives as `Block::Unknown`
(`block_common.rs:217-251`). On the raw-block path, wirerust does not use the `Block`
enum at all; block-type identification is done by reading the first 4 bytes of each
`RawBlock` body.

The obsolete **Packet Block (OPB, type `0x00000002`)** remains SKIPPED. OPB is marked
obsolete in the pcapng specification, is absent from every tool-generated capture in the
intended corpus, and is out of scope for this cycle. The `Block::Packet` variant (if
ever accessed via the high-level API) exposes `timestamp: u64` (raw, not pre-converted)
— so it is the only high-level variant that does NOT silently mangle timestamps. A
future cycle may promote OPB to a packet-producing arm cheaply. (Source: completeness-
validation finding F-08, 2026-06-19.)

**Decision 3 — Multi-IDB policy.** Unchanged from rev 3. A pcapng file may contain
multiple IDB blocks. wirerust requires all IDB blocks in a section to agree on
`linktype`. If two or more IDBs carry differing `linktype` values, the reader returns
an error (`E-INP-011`) with context identifying the conflicting link types. This
preserves the single-`DataLink` model in `PcapSource.datalink` with zero changes to
`decoder.rs`, `RawPacket`, or any downstream consumer.

**Decision 4 — 64-bit timestamp normalization via pure-core helper.** [Confirmed
LOAD-BEARING by spike; previous "unverified" qualifier removed.] The EPB 64-bit
timestamp MUST be converted to `(ts_sec: u32, ts_usecs: u32)` by the BC-2.01.014
pure-core helper, taking `(ts_high: u32, ts_low: u32, if_tsresol: u8)`. When
`if_tsresol` is absent from the IDB, the caller MUST pass `6` (10^-6, microseconds —
the pcapng specification default). The crate does NOT apply `if_tsresol`; wirerust
MUST apply it via this helper. All intermediate arithmetic MUST use saturating /
checked operations throughout (H-1/SEC-001/SEC-006 cluster):

  - Base-10 (`if_tsresol bit7 = 0`): `ticks_per_sec = 10u64.checked_pow(e as u32)
    .unwrap_or(u64::MAX)` — clamp overflow to u64::MAX; if saturated, ts_usecs = 0.
  - Base-2 (`if_tsresol bit7 = 1`): `e = if_tsresol & 0x7F`; MUST clamp `e` to [0,
    63] before shift: `ticks_per_sec = 1u64.checked_shl(e as u32).unwrap_or(u64::MAX)`.
    Shift for `e >= 64` panics in Rust with `overflow-checks = true`; the clamp is
    mandatory.
  - Intermediate product `(ticks % ticks_per_sec) * 1_000_000` MUST use a `u128`
    intermediate or `u64::saturating_mul` — the product overflows u64 for base-2 exponents
    `e >= 43`.
  - The helper is formally verifiable (pure function, no I/O) and is the Kani target
    for VP-025 (see Verification Properties section below).

**Decision 5 — Magic-byte probe discipline.** Unchanged from rev 3.

**Decision 6 — BC-2.01.004 retirement.** Unchanged from rev 3.

**Decision 7 — Single-section pcapng only; second SHB is REJECTED.** Unchanged from
rev 3. Rationale: scope-discipline and corpus-coverage decision, not a library-distrust
decision. `pcap-file` 2.0.0 resets the interface table per section
(`self.interfaces.clear()` on `Block::SectionHeader`) — confirmed by source inspection.

**Decision 8 — Forward-progress contract (raw-block path).** [NEW — rev 4, resolves
SEC-002.] The crate OWNS forward progress on the raw-block path. `RawBlock::from_slice`
rejects `block_total_length < 12` with `Err(InvalidField("Block: initial_len < 12"))`
(`block_common.rs:101-103`) and likewise rejects `initial_len % 4 != 0`
(`block_common.rs:97-99`). A malicious `block_total_length = 8` is rejected before any
block is returned — the crate does not hand a zero-advance block to the caller. On
error, the reader cursor does NOT advance (`read_buffer.rs:65`); the caller's only duty
is to `break` on `Err(_)` rather than retrying the same source. wirerust's block-walk
loop MUST break on `Err(_)` — the documented `Err(_) => {}` empty arm in the rustdoc
example is WRONG and MUST NOT be copied; it spins. The BC-2.01.015 EC-004 note that
`block_total_length = 8` yields "0 bytes skipped; no error" is INCORRECT as of the
spike finding and MUST be corrected by the PO to match the crate behavior (`< 12` is
rejected).

**Decision 9 — Snaplen enforcement.** [NEW — rev 4, resolves O-4.] The crate does NOT
validate `captured_len <= snaplen` for either EPB or SPB (confirmed: `enhanced_packet.rs`
EPB parser has no IDB access; `simple_packet.rs` has no snaplen reference). `snaplen` is
stored on the IDB but never used for packet validation. wirerust may implement
snaplen-enforcement as a caller-side check via `parser.packet_interface(&epb)` /
`reader.packet_interface(&epb)` (`parser.rs:124-126`), but this is NOT required for this
cycle. Parity with the classic path (which uses `next_raw_packet()` to avoid a crate
snaplen-rejection bug) is noted: there is no analogous bug on the pcapng path because
the crate never rejects on snaplen in the first place.

**Decision 10 — Panic surface of pcap-file 2.0.0.** [NEW — rev 4, resolves SEC-008.]
The crate is Result-clean on truncated/malformed input at the block framing layer
(all `RawBlock::from_slice` length/alignment/trailer checks return `Err`). There are
guarded `unwrap()` calls on fixed-width field reads after explicit length checks — these
cannot panic under the crate's own guards. Two panic/unimplemented! sites exist on MISUSE
paths only: `block_common.rs:213-215` (owned `RawBlock` calling `try_from_raw_block`)
and `unknown.rs:36` (`UnknownBlock::from_slice`, never called by the crate's own
dispatch). wirerust MUST NOT call either. The cargo-fuzz harness over the pcapng reader
(VP-028, F6 hardening deliverable) will exercise the full input space.

**Decision 11 — Directory-mode target detection: magic-byte content detection.** [NEW
— rev 4, resolves C-2.] Directory-mode target resolution MUST detect pcapng and classic
pcap captures by MAGIC-BYTE CONTENT, not by file extension. The ADR's lead motivator
file `arp-baseline-16pkt.cap` is a pcapng file with a `.cap` extension — extension-based
filtering would permanently exclude it. The detection logic reads the first 4 bytes of each
candidate file and accepts it as a capture if the bytes match one of the known pcap magic
values (classic: `0xA1B2C3D4` / `0xD4C3B2A1`; ns-pcap: `0xA1B23C4D` / `0x4D3CB2A1`;
pcapng: `0x0A0D0D0A`). Files that do not start with a known magic are silently skipped.
This behavior is normatively owned by BC-2.12.011 (STORY-127 glob/corpus story), which the
PO must revise to require content-detection rather than extension-filtering.

**Decision 12 — Per-file isolation: STORY-128.** [NEW — rev 4, resolves C-1.]
`main.rs:241-244` currently uses the `?` operator in the directory traversal loop,
causing the first reader error to abort the entire run. A new story STORY-128 will
refactor this loop to catch-and-continue per file: each file's reader result is
collected independently; per-file errors are accumulated and reported to stderr; exit
code 1 is set at end if any file failed. BC-2.01.018 AC-002 and E-INP-011/012 per-file-
isolation claims belong in STORY-128 (main.rs scope), not in the reader.rs BCs
(STORY-124 through STORY-127 scope). This fix benefits ALL reader errors, not only
pcapng errors.

**Decision 13 — Memory model.** [NEW — rev 4, resolves F-PERF-001.] The pcapng path
uses the same all-in-memory `Vec<RawPacket>` model as the classic-pcap path. Blocks are
iterated eagerly and each EPB/SPB that yields a packet appends one `RawPacket` to
`PcapSource.packets` before iteration continues. No streaming refactor is in scope for
this cycle; the `pcap-file` 2.0.0 block iterator enables streaming but wirerust defers
it per NFR-VIO-001. Peak RSS for a pcapng capture of file size N is approximately
N × 2.0 (Vec<RawPacket> overhead plus pcap-file 2.0.0 internal block representation
headroom), versus N × 1.5 for classic pcap. NFR-PERF-005, NFR-PERF-006, and
NFR-PERF-007 (added by PO per F2 remediation) define the quantitative memory and
throughput contracts.

**Decision 14 — Fuzzing as F6 hardening deliverable.** [NEW — rev 4.] A `cargo-fuzz`
harness `fuzz_pcapng_reader` feeding arbitrary bytes to `PcapSource::from_pcap_reader`
is mandated as an F6 hardening deliverable (assigned VP-028; see below). It is NOT an
F3 deliverable. The harness is the primary vehicle for exercising the no-panic contract
(BC-2.01.017 PC3) across the full pcapng block-walk path including edge cases not
reached by unit tests.

**Fallback — Option C (hand-roll, +0 crates).** If during implementation `pcap-file`
2.0.0's `RawBlock` / `next_raw_block` path exhibits a defect (incorrect byte-order
handling, forward-progress violation not caught by the spike), the escalation path is a
hand-rolled minimal pcapng block walker (~300 LOC), NOT Option B (`pcap-parser` 0.17).
Option B is permanently rejected (see Rationale).

## Rationale

The dependency decision (Option A via `RawBlock` path vs Option B vs full Option C
hand-roll) is dispositive. `pcap-file` 2.0.0 is already locked in `Cargo.lock:817-826`
with no new transitive dependencies required. The `RawBlock` / `next_raw_block` API
provides block framing, byte-order handling, and forward-progress guarantees (the crate
rejects `block_total_length < 12` before returning any block) — exactly the layer that
is tedious and error-prone to hand-roll. wirerust owns everything above the framing
layer (field decode, timestamp conversion, interface table, block-type dispatch), which
is correct: the crate's high-level API is where the timestamp bug lives, and by staying
on the raw path wirerust sidesteps it entirely.

The raw-block pivot (Decision 1, rev 4) is required — not merely preferred — because
the crate's `EnhancedPacketBlock.timestamp` is silently wrong for any non-nanosecond
capture (hard-codes `Duration::from_nanos` unconditionally; never applies `if_tsresol`;
raw ticks not recoverable from the parsed struct). For the pcapng specification's
default resolution (microseconds, tsresol=6 — the common case in Wireshark output),
the crate would produce timestamps 1000× too large with no error or warning. The only
correct implementation path is raw ticks → BC-2.01.014 helper → `(ts_sec, ts_usecs)`.

Option B (`pcap-parser` 0.17.0) is rejected on supply-chain grounds. It introduces
approximately four new crates including duplicate major versions of `nom` (7 and 8) and
`rusticata-macros` (4 and 5) already in wirerust's tree — a violation of the minimal-
dependency NFR — for zero capability gain over the raw-block path.

Full Option C (hand-roll, ~300 LOC) is rejected as primary: `RawBlock` provides block
framing and byte-order detection for free; hand-rolling these adds first-party attack
surface without benefit. Option C is retained as the escalation path.

The multi-IDB policy (require `linktype` agreement, reject on conflict) preserves the
`PcapSource.datalink` single-field model with zero impact on `decoder.rs` or analyzers.
Per-packet `DataLink` threading is a deferred scope expansion.

The BC-2.01.014 timestamp-conversion helper is load-bearing: it is the ONLY correct
path for tsresol-aware conversion because the crate's high-level API collapses the raw
ticks before the caller can apply `if_tsresol`. The function's pureness (no I/O, no
global state, deterministic over a 65-bit input space) makes it an ideal Kani target
(VP-025). The saturating-arithmetic prescription in Decision 4 ensures the Kani proof
can actually pass; without it, the spec itself is the source of the overflow (H-1/
SEC-001/SEC-006).

Magic-byte content detection (Decision 11) rather than extension filtering is required
because the ADR's lead motivator (`arp-baseline-16pkt.cap`) is a pcapng file with a
`.cap` extension. Extension-based filtering is insufficient by construction for a
feature whose premise is that extension ≠ format. Content detection is the only approach
consistent with the feature's motivation.

The magic-byte probe discipline (peek without consuming) is required because
`from_pcap_reader<R: Read>` receives an opaque `Read` implementation that may not
support `Seek`. The BufReader `fill_buf()` + `consume()` pattern is the idiomatic
zero-copy peek for non-seekable streams.

## Consequences

### Positive

- SS-01 supports both classic libpcap and pcapng at +0 new dependencies; supply-chain
  audit surface is unchanged.
- `arp-baseline-16pkt.cap` (pcapng-with-.cap-extension), public TLS corpus pcapng
  files, and all modern Wireshark-default captures become processable without format
  conversion. Magic-byte content detection (Decision 11) ensures `.cap` files with
  pcapng bytes are correctly identified regardless of extension.
- The BC-2.01.014 pure-core timestamp helper is Kani-provable (VP-025), extending
  formal-verification coverage into SS-01 for the first time. The helper is
  LOAD-BEARING on the raw-block path because the crate's high-level API is wrong for
  non-nanosecond captures (silently drops `if_tsresol` context).
- Timestamps are CORRECT for the common case (microsecond default, tsresol=6) via the
  raw-block path. The rev 2/rev 3 high-level API would have produced 1000× wrong
  timestamps for all Wireshark-default captures without error.
- The `DataLink` type flows from raw IDB field bytes to `PcapSource.datalink` with
  no translation table needed; `decoder.rs`, all analyzers, reassembly, dispatcher,
  and reporters require zero changes.
- BC-2.01.004's test inversion corrects a fixture-level lie that has persisted since
  brownfield ingestion.
- Per-file isolation (Decision 12, STORY-128) benefits all reader error classes, not
  only pcapng: classic pcap errors in directory mode will also no longer abort the run.
- The OPB skip (Decision 2) and multi-section rejection (Decision 7) are explicitly
  recorded decisions: implementers know exactly what wirerust does with legacy OPB
  content and multi-section files.
- The cargo-fuzz harness (VP-028, F6) provides broad coverage of the integration
  boundary between the block-walk loop and the timestamp helper.

### Negative / Trade-offs

- The raw-block path requires wirerust to hand-decode EPB/SPB/IDB fields from raw
  bytes (using `byteorder_slice` or direct byte reads in the correct endian). This is
  ~80-120 LOC of first-party byte-decode code above the framing layer, versus zero LOC
  if the high-level API were correct. The benefit (correct timestamps) justifies the
  cost; the code is straightforward and Kani-provable.
- MEDIUM regression risk on the classic-pcap path: adding a peek branch at the top
  of `from_pcap_reader` is a new code path adjacent to the hot path. The full existing
  reader test suite MUST be green before any pcapng story merges.
- The pcapng path's memory model (all-in-memory, Vec<RawPacket>) has ~2.0× file-size
  peak RSS versus ~1.5× for classic pcap (Decision 13), due to pcap-file 2.0.0's
  internal block representation held alongside the accumulating RawPacket Vec.
  Streaming refactor is deferred per NFR-VIO-001.
- Adding magic-byte detection to directory-mode traversal changes the target-file
  population: files that were silently excluded by extension filter now produce errors
  at the reader level if they start with a pcap/pcapng magic but are malformed. This
  is correct behavior but widens the error surface.
- The multi-IDB link-type-agreement policy will reject legitimate multi-NIC capture
  files that mix Ethernet and, e.g., Linux Cooked interfaces. This is a known
  limitation documented in BC-2.01.018.
- OPB packets are silently skipped (Decision 2). Any file captured by legacy tooling
  emitting OPB instead of EPB will appear to contain zero packets. Accepted limitation;
  a future cycle may promote OPB cheaply.
- Multi-section pcapng files are rejected with `E-INP-012` (Decision 7). Users can
  flatten with `mergecap -w out.pcapng <file>`.
- STORY-128 (per-file isolation in main.rs) is a new story outside the reader.rs
  scope of STORY-123 through STORY-127. It must be added to the cycle manifest and
  wave scheduling before F3 story decomposition.

### Verification Properties Assigned (rev 4)

The following VPs are assigned by this ADR to the BC-2.01.NNN framing contracts. All
are new (previously unassigned, VP-NNN = `—`). Resolves C-3 / DF-CANONICAL-FRAME-
HOLDOUT-001.

| VP-ID | BC(s) | Tool | Phase | Property |
|-------|-------|------|-------|---------|
| VP-025 | BC-2.01.014 | Kani | P1 | pcapng_timestamp_to_secs_usecs totality: no panic, ts_usecs in [0, 999_999], ts_sec plausible, for all (u32, u32, u8) inputs with saturating arithmetic |
| VP-026 | BC-2.01.010 | Kani | P1 | SHB parse safety: no panic for any truncated/malformed SHB byte sequence; byte-order BOM detection correct for LE and BE magic values |
| VP-027 | BC-2.01.012 | Kani | P1 | EPB parse safety: no panic; interface_id bounds-check before table index; captured_len guard precedes allocation; returns Err for all invalid inputs |
| VP-028 | BC-2.01.017 | cargo-fuzz | P1 | pcapng reader no-panic: PcapSource::from_pcap_reader returns Ok or Err for any byte sequence; no panic, no infinite loop (F6 hardening deliverable) |
| VP-029 | BC-2.01.015 | proptest | P1 | Block-walk skip correctness: unknown-block skip always advances past block_total_length bytes; no infinite loop; loop terminates for any valid/malformed block sequence |
| VP-030 | BC-2.01.018 | proptest | P1 | Multi-IDB linktype agreement totality: any sequence of IDB linktype u16 values either all-equal (accepted) or first-conflict returns E-INP-011 |

Note: BC-2.01.013 (SPB) is covered under VP-028 (fuzz) rather than a dedicated Kani VP
because the SPB on the raw-block path requires caller-side snaplen clamping logic that
is not a pure arithmetic invariant — it is behavioral. BC-2.01.011 (IDB parse) is
covered under VP-027's interface-table accumulation proof. BC-2.01.016 (linktype
whitelist) is test-sufficient (integration test, no new formal VP).

Holdout scenarios required by the PO per BC:
- BC-2.01.010 (VP-026): byte-exact crafted SHB with BE byte-order magic `0x4D3C2B1A`;
  SHB with invalid Byte-Order Magic; SHB truncated at byte 15 (< 28 total).
- BC-2.01.012 (VP-027): EPB with `interface_id = u32::MAX` on a 1-entry table (→ E-INP-009
  / E-INP-010); EPB with `captured_len = block_total_length - 31` (boundary valid); EPB
  with `captured_len = block_total_length - 30` (boundary invalid).
- BC-2.01.014 (VP-025): pcapng file with `if_tsresol = 6` (microsecond default) and
  known timestamp values — regression guard proving 1000× timestamp bug is absent;
  EPB with `if_tsresol = 9` (nanosecond); EPB with `if_tsresol = 0xFF` (base-2, e=127,
  must not panic); EPB with `if_tsresol = 0x94` (base-2, e=20, large exponent).
- BC-2.01.015 (VP-029): block stream containing an unknown block at a block_total_length
  boundary (e.g., total_length exactly fills remaining bytes); block stream with DSB
  immediately followed by EPB.
- BC-2.01.018 (VP-030): pcapng with two IDBs having different linktypes (→ E-INP-011
  immediately on second IDB, not deferred to EPB); pcapng with two IDBs same linktype
  (valid, multi-IDB accepted).

### Status as of 2026-06-19 (rev 4)

Proposed. `pcap-file` 2.0.0's pcapng module is dead code in the compiled binary;
`src/reader.rs` does not import it. BC-2.01.004 was RETIRED during F2 spec evolution
and superseded by BC-2.01.009. F3 story decomposition is pending adversarial
reconvergence and BC amendments listed in the PO BC-change dispatch (below).

**Rev 2 (2026-06-19):** Added Decision 7 (multi-section rejection) and OPB-skip record.

**Rev 3 (2026-06-19):** Corrected Decision 7 rationale: `pcap-file` 2.0.0 does reset
interface table per section; reject is scope-discipline, not library-distrust.

**Rev 4 (2026-06-19):** ARCHITECTURAL PIVOT — raw-block path (Decision 1) supersedes
the rev 2/rev 3 high-level-API approach, based on pcap-file 2.0.0 API spike confirming
the `EnhancedPacketBlock.timestamp` crate bug (hard-codes ns, never applies if_tsresol).
Added Decisions 8-14 covering forward-progress contract, snaplen enforcement, panic
surface, directory content-detection, per-file isolation (STORY-128), memory model, and
fuzzing mandate. Block enum corrections (9 variants, no #[non_exhaustive], no DSB
variant). VP-025 through VP-030 assigned to resolve C-3. Holdout scenarios specified.
BC-change dispatch documented for PO.

### PO BC-Change Dispatch (rev 4)

The following BC changes are REQUIRED by the PO based on rev 4 decisions. The
architect does not edit BC files; this section is the handoff specification.

**BC-2.01.010 (SHB):**
- Add VP-026 to Verification Properties cell.
- Add no-panic AC (SEC-005 resolution): "This block parser MUST return Err for any
  malformed or truncated input; unwrap/expect/panic are prohibited."
- Remove EC-004 text referencing `block_total_length = 8` as "no error" — the crate
  rejects `< 12`; EC-004 must be corrected to match (`block_total_length < 12 → Err`).
- Align SHB minimum byte count: 28 bytes minimum total block_total_length (12 outer
  header + 16 body fixed fields: BOM:4 + major:2 + minor:2 + section_length:8).
  Update E-INP-008 threshold to 28 to match (M-1 resolution).
- Add holdout scenario: BE byte-order magic `0x4D3C2B1A`; SHB truncated at byte 15.

**BC-2.01.011 (IDB):**
- Add no-panic AC per SEC-005.
- Add implementation note: interface table SHOULD be `Vec<InterfaceInfo>` (O(1) by
  interface_id); NOT HashMap.
- No new VP assigned (covered by VP-027's interface-table accumulation proof).

**BC-2.01.012 (EPB):**
- Add VP-027 to Verification Properties cell.
- Add no-panic AC per SEC-005.
- Correct Postcondition 5: EPB with interface_id referencing empty table → E-INP-009
  (NOT E-INP-008). EPB with interface_id OOB on non-empty table → E-INP-010 with
  context string `"EPB interface_id={id} out of range (table size={n})"`.
- Add explicit AC: "The interface_id field MUST be checked against the current
  interface table size before any indexing operation. An unchecked array index on
  interface_id is not permitted."
- Add guard-before-allocate AC (SEC-004): "captured_len vs. block_total_length - 32
  check MUST precede any data allocation."
- Correct EPB fixed overhead constant: name it `EPB_FIXED_OVERHEAD_BYTES`. On the
  raw-block path, the outer 12-byte block header (type:4 + total_length:4 + trailing_length:4)
  is not part of the block body; the body-relative fixed overhead is 20 bytes
  (interface_id:4 + ts_high:4 + ts_low:4 + captured_len:4 + original_len:4).
  Validation condition: `captured_len <= block_total_length - 32`
  (12 outer + 20 body-fixed = 32). Correct from any prior value.
- Add EPB captured_len = block_total_length - 32 boundary holdout.
- Note: `captured_len` field is NOT retained on the parsed type (`data.len()` recovers
  it). `original_len` IS retained. No `captured_len` field on EPB struct.

**BC-2.01.013 (SPB):**
- Correct SPB body-relative overhead to 4 bytes (original_len: u32 only).
  Validation: `block_total_length - 16 - 4 = block_total_length - 20` bytes available
  for padded packet data (12 outer + 4 body-fixed = 16 minimum; callers strip padding
  to `captured_len = min(original_len, snaplen)`).
- Add no-panic AC per SEC-005.
- Add explicit note: `data` from the crate includes padding bytes; caller MUST compute
  `captured_len = min(original_len, snaplen_from_idb[0])` and slice accordingly.
- Define SPB-without-IDB as E-INP-009 (the interface table is empty when SPB arrives).
- No dedicated Kani VP (VP-028 fuzz covers SPB field misparse).

**BC-2.01.014 (timestamp helper):**
- Add VP-025 to Verification Properties cell.
- Require saturating arithmetic throughout (Decision 4, H-1/SEC-001/SEC-006):
  base-10 `checked_pow`; base-2 e clamped to [0,63] before shift; intermediate
  product via `u128` or `saturating_mul`.
- Add holdout: `if_tsresol = 6` file with known timestamp values (microsecond
  regression guard — proves 1000× bug absent); `if_tsresol = 0xFF` (must not panic);
  `if_tsresol = 0x94` (base-2 e=20).
- Remove or update the EC-006 note about `if_tsresol = 0x3F` to also state that e >= 64
  panics without clamping, and that the spec mandates clamping.

**BC-2.01.015 (block-walk skip):**
- Add VP-029 to Verification Properties cell.
- Add no-panic AC per SEC-005.
- Correct EC-004: `block_total_length = 8` is REJECTED by the crate (`< 12` threshold
  at `block_common.rs:101`). The crate owns forward progress; the caller MUST break on
  Err(_) (not retry). Remove the "0 bytes consumed; no error" characterization.
- Add forward-progress invariant: block-walk loop MUST break on Err(_); the crate
  guarantees the cursor advances by the full block extent on Ok(_).
- Note: on the raw-block path, `Block::Unknown` / the `_` skip arm from BC-2.01.015
  does not exist — the raw block is just bytes; skip means ignoring the body bytes
  (they are already in the RawBlock body slice). The loop-break-on-error invariant
  covers both named and unknown block types.
- Add DSB log-guard note (SEC-007): "Block body bytes MUST NOT be logged, printed, or
  included in any diagnostic output regardless of log level."

**BC-2.01.016 (linktype whitelist):**
- No VP assignment (test-sufficient; integration test in STORY-126 sufficient).
- No structural change required from rev 4 decisions.

**BC-2.01.017 (cross-cutting / no-panic):**
- Add VP-028 (cargo-fuzz) to Verification Properties cell as the F6 hardening
  deliverable covering the whole pcapng reader path.
- Add confirmation that VP-028 is NOT an F3 deliverable.

**BC-2.01.018 (multi-IDB / per-file isolation):**
- Add VP-030 to Verification Properties cell.
- Move per-file isolation claims (AC-002, E-INP-011/012 notes) to STORY-128 scope.
  BC-2.01.018's AC-002 should reference STORY-128 as the owning story for the
  directory-mode per-file isolation behavior.
- Add holdout: two-IDB-different-linktypes file (→ E-INP-011 on second IDB);
  two-IDB-same-linktype file (→ accepted).

**BC-2.12.011 (directory-mode glob, STORY-127 scope):**
- Revise to require content-detection (magic-byte probe on first 4 bytes) rather than
  extension-based filtering. Known magic values: classic pcap `0xA1B2C3D4` /
  `0xD4C3B2A1`; ns-pcap `0xA1B23C4D` / `0x4D3CB2A1`; pcapng `0x0A0D0D0A`. Files
  not matching a known magic are silently skipped (not errored). This resolves C-2:
  `arp-baseline-16pkt.cap` is detected as pcapng by magic, not excluded by extension.

**STORY-128 (new story — per-file isolation in main.rs):**
- New story scoped to `src/main.rs:241-244` directory-traversal loop refactor.
- Behavior: catch reader errors per-file (do not propagate via `?`); accumulate
  per-file errors; report each to stderr; set exit code 1 if any file failed.
- This story owns E-INP-011/012 per-file-isolation AC-002 claims currently in
  BC-2.01.018.

## Alternatives Considered

- **Option B — `pcap-parser` 0.17.0 (nom-based):** Rejected. Introduces ~4 new crates
  including duplicate major versions of `nom` (7 and 8) and `rusticata-macros` (4 and 5)
  already in wirerust's tree — a direct violation of the minimal-dependency / supply-chain
  NFR — for zero capability gain over `pcap-file` 2.0.0. See research evaluation
  `pcapng-parser-dependency-eval.md` §Dependency footprint deltas.

- **Option A (3.x bump) — `pcap-file` 3.0.0:** Rejected as premature. As of 2026-06-19
  only `3.0.0-rc.2` exists (published 2026-05-06). wirerust's supply-chain posture prohibits
  pinning release candidates as hard dependencies. Reconsider when 3.0.0 stable ships.

- **Per-packet DataLink dispatch (multi-IDB Option C from F1):** Rejected for this cycle.
  Would require `DataLink` field on `RawPacket`, threading through `decode_packet`, and
  touching all analyzer call sites. Scope is disproportionate to the feature; revisit
  if mixed-interface captures become a user requirement.

- **Hand-rolled block walker (Option C) as primary:** Viable but not preferred. `pcap-file`
  2.0.0 already delivers the required capability; hand-rolling ~300 LOC of first-party
  SHB/IDB/EPB/SPB parsing duplicates a well-exercised library at additional maintenance
  cost. Retained as the designated escalation path if `pcap-file` 2.0.0's pcapng reader
  proves defective in practice.

## Source / Origin

- **F1 Delta Analysis:** `.factory/phase-f1-delta-analysis/pcapng-reader-support-delta-analysis.md` — §3, §6, §7 (feature scope, ADR requirement, dependency question)
- **Dependency research:** `.factory/research/pcapng-parser-dependency-eval.md` — full crates.io/docs.rs verification, capability matrix, dep-footprint comparison (Option A/B/C)
- **Implementation baseline:** `src/reader.rs` — classic-pcap path, snaplen-truncation discipline (`next_raw_packet()`), pcapng exclusion module doc (`src/reader.rs:5-7`)
- **Affected contract:** BC-2.01.004 ("Reject pcapng-Format Input at Reader Level", SS-01) — retired by this decision; replaced by BC-2.01.009
- **Crate API authority (rev 4):** pcap-file 2.0.0 source at `/Users/zious/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/pcap-file-2.0.0/`; `enhanced_packet.rs:27,46-48,65` (Duration::from_nanos, no tsresol); `interface_description.rs:110,179-184` (IfTsResol parsed, never applied); `block_common.rs:97-123,146-166,217-251` (enum, forward-progress, reject < 12); `simple_packet.rs:19-37` (SPB shape, no captured_len, no snaplen); `read_buffer.rs:48-103` (advance via rem pointer). See `.factory/research/pcap-file-2.0.0-api-spike.md`.
- **Format specification:** IETF draft `draft-ietf-opsawg-pcapng` (canonical: `github.com/IETF-OPSAWG-WG/pcapng`); block types SHB `0x0A0D0D0A`, IDB `0x00000001`, SPB `0x00000003`, EPB `0x00000006`; `if_tsresol` IDB option code 9
- **ADR precedent:** ADR-007 (hand-rolled DNP3 parser, Issue #8) — establishes "hand-roll binary parsers when no crate exists"; this ADR establishes the converse: "use the crate framing layer when vendored, hand-decode fields above it for correctness"
- **Multi-section research (rev 3 correction):** `.factory/research/pcapng-multisection-decision.md` — source-level confirmation that `pcap-file` 2.0.0 resets interface table per section (`interfaces.clear()` on SHB); supersedes F-06 "likely does NOT reset" inference; establishes true reject rationale (scope/corpus, not library distrust)
- **F2 remediation inputs (rev 4):** `.factory/cycles/feature-pcapng-reader/f2-review-remediation-tracker.md`; `.factory/cycles/feature-pcapng-reader/f2-adversarial-spec-review-pass1.md`; `.factory/cycles/feature-pcapng-reader/f2-security-review.md`; `.factory/cycles/feature-pcapng-reader/f2-performance-review.md`
