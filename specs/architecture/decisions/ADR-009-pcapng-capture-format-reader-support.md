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

# ADR-009: pcapng Capture-Format Reader Support (rev 8)

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

**Decision 9 — Snaplen enforcement.** [NEW — rev 4, resolves O-4. AMENDED — rev 8,
resolves H-3 + M-2.] The crate does NOT validate `captured_len <= snaplen` for either
EPB or SPB (confirmed: `enhanced_packet.rs` EPB parser has no IDB access;
`simple_packet.rs` has no snaplen reference). `snaplen` is stored on the IDB but never
used for packet validation. wirerust may implement snaplen-enforcement as a caller-side
check, but this is NOT required for this cycle.

**Rev 8 amendment — SPB captured_len computation (resolves H-3 + M-2):** EPB already
ignores snaplen (the crate validates only `captured_len + pad <= remaining body`). SPB
must match this posture. wirerust MUST compute SPB `captured_len` as:

  ```
  captured_len = min(original_len, block_body_available)
  ```

where `block_body_available = body.len() as u32` (the raw bytes in the RawBlock body
slice minus the 4-byte `original_len` header field). The `snaplen` term is DROPPED from
the SPB captured-len formula. Rationale:

  - The on-disk SPB body is authoritative: `block_body_available` is the real extent of
    available packet data, including padding. Using it as the upper bound is sufficient
    and avoids any risk of silent truncation from a stale or absent IDB snaplen.
  - Symmetry with EPB: neither block type consults snaplen for captured-len derivation in
    wirerust. Both use the on-disk block content as the authoritative bound.
  - Decision 9 (rev 4) already stated "snaplen not enforced for either EPB or SPB."
    This amendment makes that posture precise for the SPB's implicit captured-len formula.

VP-031 property update (see VP-INDEX amendment): the VP-031 property now reads
`captured_len == min(original_len, body.len() as u32)` (two-argument min, snaplen
dropped). The PO must update BC-2.01.013 PC1/AC-002/EC-007 and HS-107 Case E
accordingly (see PO BC-Change Dispatch, rev 8 additions).

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

**Decision 15 — Interleaved IDB policy (IDB after first packet block).** [NEW — rev 5,
resolves I-5/I-6.] The pcapng specification permits an Interface Description Block (IDB)
to appear anywhere within a section, including after packet blocks (EPB/SPB), to describe
a new capture interface opened mid-capture. wirerust DOES NOT support this ordering.
An IDB encountered AFTER the first packet block has been emitted is REJECTED immediately
with `Err` mapping to error code **E-INP-013** ("pcapng interface description block after
first packet block — unsupported ordering"). Processing stops at that point; any
subsequent blocks are not parsed. The interface table state at the point of rejection is
discarded with the error.

Rationale: the intended corpus (Wireshark default captures, PacketLife traces, public
protocol fixtures) opens all interfaces before the first packet — the IDB-first ordering
is universal in practice. Interleaved IDBs arise only in live-capture scenarios where a
second NIC comes online after packets from the first NIC have already been written.
wirerust's fail-closed posture requires an explicit, diagnosed error over silent misbehavior.
Full interleaved-IDB support is the correct scope for a future cycle when live-capture
streaming is introduced (estimated cost: add per-IDB lazy registration to the block-walk
loop, promote interface_table from Vec to a RefCell-backed structure, thread packet
DataLink per-packet — a medium-scope refactor outside this cycle's NFR-VIO-001 deferral).

**Linktype whitelist timing (Decision 15 amendment, resolves I-5):** BC-2.01.016's
linktype whitelist check is applied **at first-IDB-parse time**, immediately when the IDB
block body is decoded, before any packet from that interface is consumed. It is NOT
deferred to "after all IDBs" (undefined under streaming) nor "at first packet" (too late
for early error reporting). This means a linktype-whitelist violation (E-INP-010 / TBD
error code) is reported at the IDB stage and parsing aborts before any packets are
emitted from that interface. The PO must update BC-2.01.016 to specify: "linktype check
fires at IDB-parse time, not at first-packet time."

**Decision 16 — Single-section only: per-section interface-table reset is DEAD SPEC (resolves H-5).** [NEW — rev 6.] Because Decision 7 rejects any second SHB with E-INP-012 before parsing begins, the "interface table resets at each SHB" behavior mandated by BC-2.01.011 Inv 2 and BC-2.01.018 Inv 4 + EC-005 can NEVER execute in wirerust. There is no code path by which a second section's IDB list could be processed or the interface table cleared between sections. The per-section reset behaviour is therefore DEAD SPEC — it describes a code path that is statically unreachable given Decision 7. This is explicitly DEFERRED to the future multi-section escape hatch (if that hatch is ever opened, the per-section reset must be implemented at that time). The architect records this as a conscious scope narrowing, not an oversight. The PO must remove or mark-deferred the per-section-reset invariant from BC-2.01.011 and BC-2.01.018, and correct BC-2.01.018 EC-005 so that a multi-section file is rejected with E-INP-012 at the second SHB (not "succeeds individually per section").

**Decision 17 — IDB error-code precedence at IDB-parse time (resolves M-7).** [NEW — rev 6.] When an IDB is encountered, three error conditions may apply, and their evaluation order is architecturally fixed:

  1. **E-INP-013 FIRST — position check (IDB-after-first-packet-block):** If `packets_emitted > 0` at IDB-parse time, the IDB is immediately rejected with E-INP-013 and processing stops. The IDB body is NOT decoded; its linktype is NOT examined. A late IDB that also conflicts on linktype still receives only E-INP-013 — it is rejected by POSITION before its CONTENT is examined. It is never admitted to the interface table.

  2. **E-INP-001 SECOND — linktype whitelist (content gate):** If the IDB is positionally valid (`packets_emitted == 0`), the IDB body is decoded and the `linktype` field is checked against the whitelist. If the linktype is not on the whitelist, the reader returns E-INP-001 immediately (at IDB-parse time per Decision 15 amendment) and parsing aborts before any packets from that interface are emitted.

  3. **E-INP-011 THIRD — multi-IDB linktype agreement:** If the IDB is positionally valid and the linktype is on the whitelist, the linktype is compared against the linktype of all previously registered IDBs. If the new IDB's linktype conflicts with the existing interface table's agreed linktype, the reader returns E-INP-011 immediately on that IDB, before any packets from that interface are consumed.

This precedence is architecturally motivated: POSITION errors are cheaper to check (no decode required) and represent structural violations; CONTENT errors (whitelist) require a decode but are caught before table mutation; AGREEMENT errors require prior table state. The PO must state this precedence ordering in BC-2.01.011, BC-2.01.016, and BC-2.01.018, and add an edge-case scenario: "late IDB that also conflicts on linktype → E-INP-013 wins (position wins over content)."

**Decision 18 — VP-031: SPB captured-len computation correctness (proptest, resolves M-2).** [NEW — rev 6. PROPERTY UPDATED — rev 8, Decision 9 amendment.] HS-107 asserts byte-exact SPB captured-len arithmetic but `cargo-fuzz` (VP-028) cannot express this as a typed property — fuzzing exercises no-panic but cannot assert arithmetic relationships between fields. VP-031 is assigned as a dedicated proptest VP for BC-2.01.013 covering SPB captured-len computation correctness:

  - **Property (rev 8):** For all `(original_len: u32, body: &[u8])` with `body.len() <= u32::MAX as usize`: `captured_len == min(original_len, body.len() as u32)`, the returned slice has exactly `captured_len` bytes, and no out-of-bounds access occurs. The `snaplen` parameter has been DROPPED from the domain per Decision 9 rev 8 amendment (SPB captured_len uses only on-disk block body as the upper bound; see Decision 9 rev 8).
  - **Tool:** proptest (pure arithmetic predicate over a pure-core helper — no I/O, no block framing).
  - **Phase:** P1.
  - **Module:** `reader.rs (pcapng_pure_core fns)` — the SPB body-clamp helper function is a pure-core sub-function colocated in `src/reader.rs`, extracted from the block-walk loop exactly as the EPB field-decode function (VP-027) is.
  - **Status:** draft (pending F3 story decomposition).

VP-031 gives SPB a real framing VP per DF-CANONICAL-FRAME-HOLDOUT-001. It is NOT a fuzz-only-covered BC. See VP-INDEX and verification-coverage-matrix.md for updated counts (total 30→31, proptest 9→10, P1 16→17; counts unchanged at rev 8 — property domain change only).

**Decision 19 — Zero-packet notice gating (SOUL-#4 silent-failure-prevention anchor).** [NEW — rev 7, resolves M-4. AMENDED — rev 8, resolves H-2 + M-5.] A capture file that is STRUCTURALLY VALID — parses to EOF without error — but yields `packets.len() == 0` MUST emit exactly one stderr notice before exit 0. This is the one-shot zero-packet notice.

**Emission point (rev 8 amendment, resolves M-5):** The notice is EMITTED from
`main.rs`, NOT from the reader (`from_pcap_reader` / `PcapSource`). Rationale: only
`main.rs` has the filename available; `from_pcap_reader` has no filename and already
returns `Result`; diagnostics belong in `main.rs` per the project's E-INP-007 pattern.
`from_pcap_reader`/`PcapSource` MUST SURFACE the data needed so that `main.rs` can
emit the notice:

  - `PcapSource::skipped_blocks: u32` — count of all blocks skipped (unknown types,
    OPB, etc.) during the walk.
  - `PcapSource::opb_skipped: u32` — count of blocks skipped that were specifically
    Obsolete Packet Blocks (block type `0x00000002`). A non-zero value means OPB packet
    data was silently discarded.
  - The `packets.is_empty()` check is the primary condition `main.rs` tests.

**OPB distinction (rev 8 amendment, resolves H-2):** When any skipped blocks include
OPB (Obsolete Packet Block, type `0x00000002`), the notice MUST state explicitly that
packet data from obsolete Packet Blocks was not ingested. The notice format with OPB
present is:

  ```
  notice: <filename>: 0 packets read from pcapng file
  (N blocks skipped, including M obsolete Packet Blocks whose packet data was not analyzed — re-save with mergecap to convert to EPB)
  ```

  When skipped blocks include no OPB:
  ```
  notice: <filename>: 0 packets read from pcapng file (N blocks skipped)
  ```

  When no blocks were skipped:
  ```
  notice: <filename>: 0 packets read from pcapng file
  ```

  The exact wording is owned by the PO; the MANDATORY distinction is: if `opb_skipped >
  0`, the notice MUST differentiate OPB-bearing skipped blocks from non-packet-bearing
  skipped blocks (NRB/ISB/DSB/SJE) and MUST state that packet data was not ingested.
  A remediation hint (mergecap) MUST accompany any OPB notice.

**Classic-pcap symmetry (rev 8 amendment, resolves M-5):** A structurally valid EMPTY
classic pcap file (valid header, zero packets) MUST ALSO emit the same zero-packet
notice. Consistency requires that the "zero packets from a valid file" diagnostic fires
for both file formats, not only for pcapng. `main.rs` owns this for classic pcap as
well. The classic-pcap notice format: `"notice: <filename>: 0 packets read from pcap file"`.

**Remaining invariants (unchanged from rev 7):**
  - Block body bytes MUST NOT appear in the notice or in any diagnostic output (SEC-007 / DSB key material).
  - One notice per file, not per block.
  - Exit code: 0. A structurally valid zero-packet file is not an error.
  - A file that yields a parse error does NOT use this notice path; it emits a normal error and exits 1.

Rationale: silently returning an empty `PcapSource` from a valid file is SOUL-#4
silent-failure. The notice is the minimum diagnostic required to distinguish "no packets
in this valid file" from "file unreadable." BC-2.01.009 PC6 and BC-2.01.015 PC9 MUST
cite THIS decision (Decision 19) — they currently mis-cite Decision 17 (IDB error-code
precedence), which is unrelated. The PO must correct those citations.

**Decision 20 — Uniform block error-code rule (CORRECTS pass-3 over-narrowing; resolves H-1). CLARIFIED — rev 8, resolves C-1.** [NEW — rev 7.] One rule applies to ALL block types on the raw-block path. Three mutually-exclusive tiers, evaluated in order:

**Tier 1 — Framing rejection (E-INP-010):** Fired by the CRATE before wirerust sees the block body.
  - `block_total_length < 12` → crate returns `Err(InvalidField("Block: initial_len < 12"))`.
  - `block_total_length % 4 != 0` → crate returns `Err` (misalignment).
  - EOF before trailing-length field → crate returns `Err`.
  - wirerust maps all `Err` from `next_raw_block` to **E-INP-010** (framing error).
  - The block body is NOT decoded; the block type is NOT examined.

**Tier 2 — Body-decode failure (E-INP-008):** Fired by wirerust after the crate hands a valid RawBlock.
  - The crate has framed the block (`block_total_length >= 12`, aligned, trailer present).
  - The crate passes the block to wirerust with `body = block_data[0 .. block_total_length - 12]`.
  - If `body.len() < BLOCK_FIXED_FIELD_BYTES` for the block type, wirerust returns **E-INP-008**.
  - Per-block fixed-field minimums (body-relative, i.e., not counting the 12-byte outer header):

  | Block type | Block code | Body fixed-field bytes | Window for E-INP-008 (total btl range) |
  |------------|-----------|----------------------|----------------------------------------|
  | SHB | 0x0A0D0D0A | 16 (BOM:4 + major:2 + minor:2 + section_length:8) | 12 <= btl < 28 |
  | IDB | 0x00000001 | 8 (linktype:2 + reserved:2 + snaplen:4) | 12 <= btl < 20 |
  | EPB | 0x00000006 | 20 (interface_id:4 + ts_high:4 + ts_low:4 + captured_len:4 + original_len:4) | 12 <= btl < 32 |
  | SPB | 0x00000003 | 4 (original_len:4) | btl == 12 exactly (12-4=8 body; 4 fixed; SPB with btl=12 has 0 bytes of packet data) |

  - SEMANTICALLY INVALID body (body bytes long enough but content wrong): also E-INP-008.
    Example: SHB with correct body length but BOM != 0x1A2B3C4D / 0x4D3C2B1A, OR SHB major version != 1 → E-INP-008.
  - Unknown block types: no fixed-field check; skip via block_total_length (Decision 2). E-INP-010 from crate if framing fails.
  - **EPB padding-overrun check (rev 8 clarification, resolves C-1):** After wirerust decodes the EPB fixed fields (including `captured_len`), it must verify that `captured_len + pad_len(captured_len) <= body.len()` (i.e., the padded extent fits within the block body). This is a WIRERUST body-decode check performed AFTER the crate has framed the block successfully. A violation → **E-INP-008**, NOT E-INP-010. The crate already validated `captured_len + pad <= remaining body` internally at `enhanced_packet.rs:55-57` on the high-level path, but wirerust on the RAW path recomputes this from the raw body slice and must signal the same condition as E-INP-008. Similarly, the bound-by-body check `captured_len <= body.len()` is a wirerust body-decode constraint → **E-INP-008**. Summary: ALL wirerust-computed body/length-consistency failures for EPB (body-too-short for fixed fields, captured_len > body.len(), padded extent > body.len()) → E-INP-008. Only crate framing rejection (btl < 12, misaligned, EOF before trailer) → E-INP-010.

**Tier 3 — Well-formed:** Body long enough and semantically valid → proceed with full decode.

**Correction of pass-3 over-narrowing:** The pass-3 review (ADR-009 rev 5/6) asserted that "SHB body-too-short is unconstructible on the raw-block path." This is INCORRECT. A block with `btl=16` is aligned (16 % 4 == 0), >= 12, and the crate accepts it; wirerust receives a body of 4 bytes (btl-12=4), which is shorter than the SHB fixed-field minimum of 16 bytes. Therefore the SHB body-too-short → E-INP-008 case IS constructible and MUST be handled. Similarly for EPB (btl=12 → body=0 bytes, < 20) and SPB (btl=12 → body=0 bytes, < 4).

**PO actions required:**
  - BC-2.01.010 (SHB): RE-ADD the SHB body-too-short → E-INP-008 case (window 12 <= btl < 28). State the SHB fixed-field minimum as 16 body bytes. State the E-INP-008/E-INP-010 split: framing failure (crate rejects btl < 12) → E-INP-010; body-too-short (crate accepts btl >= 12, wirerust body-decode fails) → E-INP-008; semantic failure (bad BOM, wrong major version) → E-INP-008.
  - BC-2.01.012 (EPB): State the EPB fixed-field minimum as 20 body bytes; window 12 <= btl < 32 → E-INP-008. Align with the table above. ALSO state explicitly: EPB padding-overrun (`20 + captured_len + pad_len(captured_len) > body.len()`) → E-INP-008; bound-by-body check (`captured_len > body.len() - 20`) → E-INP-008. All wirerust-computed body/length-consistency failures → E-INP-008 (NOT E-INP-010). Reclassify padding-overrun + bound-by-body from E-INP-010 → E-INP-008 in BC-2.01.012, error-taxonomy E-INP-010 item (c), HS-104 Case E, and VP-027 property description.
  - BC-2.01.013 (SPB): State the SPB fixed-field minimum as 4 body bytes; btl = 12 → body = 0 bytes < 4 → E-INP-008. Align with the table above.
  - BC-2.01.011 (IDB): State the IDB fixed-field minimum as 8 body bytes; window 12 <= btl < 20 → E-INP-008. Already partially correct; align with the table.
  - error-taxonomy.md: Update E-INP-008 scope text to state: "wirerust body-decode failure for any block type after successful crate framing — body shorter than the block's required fixed-field bytes, body semantically invalid (SHB: bad BOM or major!=1), or EPB body/length-consistency failure (padding-overrun, captured_len > body extent)." Update E-INP-010 scope text to state: "block framing rejection by pcap-file 2.0.0 crate — btl < 12, btl % 4 != 0, or EOF before trailer. Does NOT include wirerust body-decode checks computed after successful crate framing."

**Decision 21 — if_tsoffset OUT OF SCOPE this cycle (resolves M-2 option-walk gap).** [NEW — rev 7.] wirerust MUST NOT extract `if_tsoffset` (IDB option code 10) in this cycle. Extracting an option without applying it is architecturally deceptive — it implies the value is used when it is not. The limitation is:

  - `if_tsoffset` is a signed 64-bit value (pcapng spec option code 10) added to the raw timestamp ticks before applying `if_tsresol`. Its effect is small and its presence is rare in the target corpus (Wireshark/PacketLife captures do not set it).
  - wirerust's BC-2.01.014 timestamp helper takes `(ts_high, ts_low, if_tsresol)` only. Adding `if_tsoffset` requires a 64-bit signed intermediate, extending the overflow surface and the VP-025 Kani proof domain.
  - Scope deferral: future cycle escape hatch. When `if_tsoffset` support is introduced: (a) extend the pure-core helper signature to `(ts_high: u32, ts_low: u32, if_tsresol: u8, if_tsoffset: i64) -> (u32, u32)`; (b) add a new VP covering signed overflow for the offset addition; (c) update IDB option parsing to extract code 10.

**PO actions required:**
  - BC-2.01.011 PC6 (IDB options walk): REMOVE "and if_tsoffset (code 10)" from the options-walk description. The IDB options walk MUST enumerate only the options that wirerust actually processes: `if_tsresol` (code 9). Add a limitation note: "if_tsoffset (option code 10) is NOT extracted or applied in this cycle; its effect on timestamps is silently ignored."
  - BC-2.01.014: Add a limitation note in the specification: "This helper does not accept or apply if_tsoffset (IDB option code 10). Files with if_tsoffset set will have a timestamp bias equal to the offset value × 1/ticks_per_sec seconds. This is an accepted limitation for this cycle."

**VP-030 RESTATEMENT (resolves H-3).** [NEW — rev 7.] VP-030 as written in ADR-009 rev 6 and VP-INDEX v2.5 is unsatisfiable in its current domain: "any sequence of IDB linktype u16 values" includes non-whitelisted values, but non-whitelisted linktypes trigger E-INP-001 at IDB-parse time (Decision 17 step 2) before the multi-IDB agreement check (Decision 17 step 3 / E-INP-011) is ever reached. A proptest generating arbitrary u16 linktype values will saturate on E-INP-001 rejections for non-whitelisted values and never exercise the agreement check.

Restated property (VP-030 v2): generate sequences of **WHITELISTED DataLink values only** (the domain where the E-INP-011 conflict check is reachable). The whitelist precedes the conflict check in Decision 17 IDB evaluation order; non-whitelisted values short-circuit to E-INP-001 at the FIRST IDB and the conflict check is never reached. Pin the comparison unit to `DataLink` (not raw u16). Property:
  - All-equal sequence of whitelisted DataLink values → Ok; `PcapSource.datalink` equals that value.
  - First-differing whitelisted DataLink value in the sequence → Err(E-INP-011) immediately on that IDB; no subsequent IDBs are processed.
  - Non-whitelisted DataLink value → E-INP-001 (out of VP-030 scope; covered by BC-2.01.016 integration tests).

**M-5 — Block sequence numbering convention.** [NEW — rev 7.] The `block #<seq>` field referenced in E-INP-010, E-INP-012, and E-INP-013 error context strings MUST use a UNIFORM 1-based block index counting the SHB as block #1. Every block seen by the crate's `next_raw_block` call increments the counter, regardless of block type. The counter starts at 1 when the first block is returned, not at 0.

Rationale: 1-based indexing is the human-readable convention (consistent with Wireshark's "frame #N" display); 0-based indexing is the internal array-index convention. Error messages are for humans; use 1-based. Uniformity across all three error codes prevents off-by-one inconsistencies in error message context between error sites.

PO actions: Update E-INP-010, E-INP-012, and E-INP-013 entries in error-taxonomy.md to state "block #<seq> where seq is the 1-based block index (SHB = block #1)" in the Context field. Remove any existing "0-based" qualifiers.

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

### Verification Properties Assigned (rev 4, updated through rev 8)

The following VPs are assigned by this ADR to the BC-2.01.NNN framing contracts. All
are new (previously unassigned, VP-NNN = `—`). Resolves C-3 / DF-CANONICAL-FRAME-
HOLDOUT-001.

| VP-ID | BC(s) | Tool | Phase | Property |
|-------|-------|------|-------|---------|
| VP-025 | BC-2.01.014 | Kani | P1 | pcapng_timestamp_to_secs_usecs totality: no panic, ts_usecs in [0, 999_999], ts_sec saturated (`.min(u32::MAX)`) for all (u32, u32, u8) inputs; saturating arithmetic; MUST include large-ts_high vector where `(ts_high << 32 | ts_low) / ticks_per_sec > u32::MAX` to lock the saturation (rev 8 / M-3) |
| VP-026 | BC-2.01.010 | Kani | P1 | SHB parse safety: no panic for any truncated/malformed SHB byte sequence; byte-order BOM detection correct for LE and BE magic values |
| VP-027 | BC-2.01.012 | Kani | P1 | EPB parse safety: no panic; interface_id bounds-check before table index; captured_len guard precedes allocation; padding-overrun check (`20 + captured_len + pad_len > body.len()`) and bound-by-body check (`captured_len > body.len() - 20`) both → Err(E-INP-008) (NOT E-INP-010); returns Err for all invalid inputs (rev 8 / C-1) |
| VP-028 | BC-2.01.017 | cargo-fuzz | P1 | pcapng reader no-panic: PcapSource::from_pcap_reader returns Ok or Err for any byte sequence; no panic, no infinite loop (F6 hardening deliverable) |
| VP-029 | BC-2.01.015 | proptest | P1 | Block-walk skip correctness: unknown-block skip always advances past block_total_length bytes; no infinite loop; loop terminates for any valid/malformed block sequence |
| VP-030 | BC-2.01.018 | proptest | P1 | Multi-IDB linktype agreement totality (RESTATED rev 7 / H-3): any sequence of WHITELISTED DataLink values either all-equal → Ok(PcapSource.datalink = that value), or first-differing whitelisted DataLink → Err(E-INP-011) immediately on that IDB. Non-whitelisted values short-circuit to E-INP-001 at first IDB (before conflict check) — NOT in VP-030 domain. Comparison unit: DataLink (not raw u16). |
| VP-031 | BC-2.01.013 | proptest | P1 | SPB captured-len computation correctness: for all (original_len: u32, body: &[u8]), captured_len == min(original_len, body.len() as u32); returned slice has exactly captured_len bytes; no out-of-bounds access. Snaplen term DROPPED (rev 8 / Decision 9 amendment, resolves H-3 + M-2). (resolves M-2 / DF-CANONICAL-FRAME-HOLDOUT-001) |

Note: BC-2.01.013 (SPB) is now covered by VP-031 (proptest, P1) for arithmetic
correctness of the on-disk-body-clamped captured-len computation (snaplen dropped per
Decision 9 rev 8), AND by VP-028 (fuzz) for the full no-panic contract on the raw-block
path. VP-031 fills the framing VP gap identified in M-2: cargo-fuzz cannot express the
arithmetic relationship between original_len and the returned slice length, which is the
core HS-107 assertion. BC-2.01.011 (IDB parse) is covered under VP-027's
interface-table accumulation proof. BC-2.01.016 (linktype whitelist) is test-sufficient
(integration test, no new formal VP).

#### VP-025 Kani Provability Note — unwind bound (I-2 resolution, rev 5)

VP-025 targets `pcapng_timestamp_to_secs_usecs(ts_high: u32, ts_low: u32, if_tsresol: u8)`,
which calls `10u64.checked_pow(e as u32)` where `e = if_tsresol & 0x7F` (base-10 branch).
`checked_pow` is iterative (loop over `e` multiplications); with symbolic `e` up to 127
Kani's default loop unwind of 1 will not explore the full iteration count, causing the
proof to be vacuous (Kani reports SUCCESSFUL with insufficient unwind, which is a false
pass).

**Required implementation choice (PO/implementer must pick one):**

Option A (PREFERRED — bounded, Kani-decidable): Implement `ticks_per_sec` for base-10
using a precomputed lookup table for `e ∈ [0, 19]` and saturating to `u64::MAX` for
`e ≥ 20`. `10^19 = 10_000_000_000_000_000_000 < u64::MAX`; `10^20` overflows u64.
The table has 20 entries, constant-time, no loop. Kani sees no iteration; the proof is
trivially bounded. The VP harness then requires NO explicit `#[kani::unwind]` annotation
and is deterministic.

Option B (acceptable, explicit unwind): Keep the `checked_pow` loop but add
`#[kani::unwind(128)]` to the VP-025 proof harness (e ∈ [0, 127] requires at most
127 iterations). Kani will explore the full range. Proof time increases modestly
(benchmark: ~30–60 s for this input domain). The harness MUST carry a comment
documenting the unwind bound and its justification.

Option A is the preferred implementation: it eliminates the loop from the pure-core
function entirely, making the function trivially provable and faster at runtime for the
common case (e=6, microseconds). The PO must add an implementation note to BC-2.01.014
specifying that the base-10 path uses a precomputed power-of-ten table or, if Option B
is chosen, that the Kani harness carries `#[kani::unwind(128)]`.

This decision MUST be recorded in BC-2.01.014 before STORY-125 F3 story decomposition.

### HS-Completeness Map — Framing BCs → Required Holdout Scenarios (rev 5, resolves I-14)

The following table maps each framing BC to its required holdout scenario (HS-NNN). A
missing holdout is a process gap that makes a BC untestable at the Phase 4 holdout gate.
This map makes gaps visible; each row must be satisfied before the Phase 4 gate opens.

| BC | Title (short) | Required Holdout | Status |
|----|---------------|-----------------|--------|
| BC-2.01.010 | SHB parse / byte-order detection | HS-103 | AUTHORED |
| BC-2.01.012 | EPB parse / interface_id bounds | HS-104 | AUTHORED |
| BC-2.01.013 | SPB parse / snaplen clamping | HS-107 | AUTHORED (VP-031 proptest + VP-028 fuzz) |
| BC-2.01.014 | Pure-core timestamp normalization | HS-101, HS-102 | AUTHORED |
| BC-2.01.015 | Block-walk skip / forward progress | HS-105 | AUTHORED |
| BC-2.01.018 | Multi-IDB linktype agreement | HS-106 | AUTHORED |
| BC-2.01.009 / BC-2.01.015 | Zero-packet notice (valid file, 0 packets) | HS-108 | MISSING — PO must author |

**C-2 resolved: HS-107 is now AUTHORED** (`.factory/holdout-scenarios/HS-107-pcapng-spb-framing-truncation-padding-and-no-idb.md`; reflected in HS-INDEX v2.1). BC-2.01.013 SPB parsing and snaplen clamping are fully covered at the Phase 4 holdout gate.

Holdout scenarios required by the PO per BC:
- BC-2.01.010 (VP-026): byte-exact crafted SHB with BE byte-order magic u32 `0x1A2B3C4D`, on-disk bytes `1A 2B 3C 4D` (LE section stores same u32 as on-disk bytes `4D 3C 2B 1A`);
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

### Status as of 2026-06-19 (rev 5)

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

**Rev 4 minor correction (2026-06-19):** BC-2.01.013 dispatch note corrected: removed double-subtraction of original_len field; SPB fixed overhead is 16 bytes total (block-type:4 + block-total-length:4 + original-packet-length:4 + trailing block-total-length:4); available data = btl - 16; body-relative overhead = 4 bytes (original_len only). Now consistent with BC-2.01.013 v1.1.

**Rev 4 minor correction 2 (2026-06-19):** Corrected two mislabeled BE byte-order magic values. The canonical BOM is u32 `0x1A2B3C4D`; a BE section stores it on-disk as bytes `1A 2B 3C 4D`; an LE section stores it on-disk as bytes `4D 3C 2B 1A`. Both holdout-scenario references (line ~411 and PO BC-change dispatch line ~464) previously and incorrectly named the BE case as `0x4D3C2B1A` (the LE on-disk byte sequence). Fixed per BC-2.01.010 v1.5 / HS-103 v1.2.

**Rev 5 (2026-06-19):** Pass-2 adversarial remediation — architect-owned items. Added Decision 15 (interleaved-IDB policy: IDB-after-first-packet-block → E-INP-013, reject with clear error; linktype-whitelist timing: check at IDB-parse time, not deferred). Added HS-completeness map (framing BC → holdout HS-NNN; flags BC-2.01.013 SPB as MISSING HS-107 — PO to author). Added VP-025 Kani provability note with unwind-bound analysis: Option A (precomputed power-of-ten lookup for e∈[0,19], preferred) and Option B (explicit `#[kani::unwind(128)]` on harness). Error code E-INP-013 introduced for interleaved-IDB rejection; PO must add to error-taxonomy.md. VP-025/026/027 module re-anchor from `reader.rs` to `reader.rs (pcapng_pure_core fns)` recorded in VP-INDEX and arch docs (I-1 resolution).

**Rev 5 minor correction (2026-06-19):** HS-Completeness Map BC-2.01.013 row updated from MISSING to AUTHORED; C-2 flag resolved. HS-107 (`HS-107-pcapng-spb-framing-truncation-padding-and-no-idb.md`) authored and reflected in HS-INDEX v2.1.

**Rev 6 (2026-06-19):** Pass-3 adversarial remediation — dead-spec reconciliation, error-code precedence, and SPB framing VP. Added Decision 16 (H-5: per-section interface-table reset is DEAD SPEC — unreachable given Decision 7 single-section-only constraint; explicitly deferred to future multi-section escape hatch; PO must delete/defer the reset invariant in BC-2.01.011 and BC-2.01.018 and correct EC-005 so multi-section file is rejected with E-INP-012 at second SHB). Added Decision 17 (M-7: IDB error-code precedence fixed at three-level ordering: E-INP-013 position check FIRST, E-INP-001 whitelist SECOND, E-INP-011 multi-IDB agreement THIRD; PO must state this in BC-2.01.011/016/018 and add late-IDB-with-conflicting-linktype edge case). Added Decision 18 and VP-031 (M-2: SPB captured-len computation correctness as dedicated proptest VP for BC-2.01.013; fills framing VP gap; VP-028 fuzz continues to cover no-panic; total 30→31, proptest 9→10, P1 16→17). Updated VP table to include VP-031 row; updated BC-2.01.013 coverage note. O-3 process-gap noted in PO BC-change dispatch.

**Rev 7 (2026-06-19):** Pass-4 adversarial remediation — zero-packet notice, uniform block error-code rule, if_tsoffset scope deferral, VP-030 restatement, seq convention, HS-completeness map extension. Added Decision 19 (M-4: zero-packet notice — valid pcapng with 0 packets emits exactly one stderr notice, exit 0, skipped-block count when >0, no block bodies logged; SOUL-#4 anchor; BC-2.01.009 PC6 / BC-2.01.015 PC9 must re-cite this decision not Decision 17). Added Decision 20 (H-1: uniform block error-code rule — corrects pass-3 "SHB body-too-short unconstructible" claim; three-tier rule: crate framing failure → E-INP-010; wirerust body-decode failure or semantic failure → E-INP-008; well-formed → proceed; per-block fixed-field minimums table: SHB=16, IDB=8, EPB=20, SPB=4 body bytes; PO must RE-ADD SHB body-too-short to BC-2.01.010, align EPB/SPB/IDB BCs and error-taxonomy E-INP-008/010 scope text). Added Decision 21 (M-2 option-walk: if_tsoffset code 10 MUST NOT be extracted or applied this cycle; out-of-scope deferral with future escape hatch; PO must remove "and if_tsoffset (code 10)" from BC-2.01.011 PC6 and add limitation note to BC-2.01.014). Added VP-030 restatement (H-3: VP-030 domain narrowed to WHITELISTED DataLink values only; non-whitelisted values short-circuit to E-INP-001 before the conflict check; comparison unit pinned to DataLink not raw u16). Added M-5 seq convention (1-based block index, SHB = block #1; E-INP-010/012/013 context strings must be consistent). Added HS-108 to HS-completeness map (zero-packet notice end-to-end; MISSING — PO must author). VP-030 restatement propagated to VP-INDEX v2.6, verification-architecture.md v2.2, and verification-coverage-matrix.md v1.16.

**Rev 8 (2026-06-20):** Pass-5 adversarial remediation — Decision 20 clarification (C-1), Decision 9 amendment (H-3 + M-2), Decision 19 amendments (H-2 + M-5), VP property updates.
  - Decision 20 (C-1 EPB error-code clarification): EPB padding-overrun check (`20 + captured_len + pad_len(captured_len) > body.len()`) and bound-by-body check (`captured_len > body.len() - 20`) are WIRERUST body-decode failures → E-INP-008, NOT E-INP-010. The uniform rule now explicitly covers all wirerust-computed body/length-consistency failures for EPB as E-INP-008. PO must reclassify padding-overrun + bound-by-body in BC-2.01.012, error-taxonomy E-INP-010 item (c), HS-104 Case E, and VP-027.
  - Decision 9 amendment (H-3 + M-2 SPB captured_len): SPB captured_len formula changed from `min(original_len, snaplen, block_body_available)` to `min(original_len, block_body_available)` — snaplen DROPPED. Matches EPB (which ignores snaplen) and makes Decision 9's "snaplen not enforced" precise. VP-031 property domain narrowed to two-argument `min(original_len, body.len() as u32)`. PO must update BC-2.01.013 PC1/AC-002/EC-007, HS-107, and VP-031 property.
  - Decision 19 amendment 1 (H-2 OPB distinction): Zero-packet notice must distinguish OPB-bearing skipped blocks from non-packet skipped blocks (NRB/ISB/DSB/SJE). When OPB skipped, notice MUST state packet data from obsolete Packet Blocks was not ingested + remediation hint (mergecap). PcapSource must surface `opb_skipped: u32` so main.rs can emit the distinction. PO must update BC-2.01.009 and BC-2.01.015 with this distinction and the new `opb_skipped` field.
  - Decision 19 amendment 2 (M-5 emission point + symmetry): Notice emitted from main.rs (not reader); PcapSource surfaces `skipped_blocks`, `opb_skipped`, and empty-packets condition; notice format unified per Decision 19 authority; classic-pcap empty files also get zero-packet notice for consistency. PO must update BC-2.01.009 (emission point) and BC-2.01.015 (counter surfaced not emitted).
  - VP-025 (M-3): Property updated to note ts_sec saturates (`.min(u32::MAX)`) and MUST include large-ts_high vector where ticks/ticks_per_sec > u32::MAX to lock the saturation. PO must ensure BC-2.01.014 µs fast path also saturates ts_sec.
  - VP-027 (C-1): Property updated to explicitly state padding-overrun and bound-by-body → Err(E-INP-008) not E-INP-010.
  - VP-031 (Decision 9 amendment): Property domain changed from `(original_len, snaplen, body)` to `(original_len, body)` with `min(original_len, body.len() as u32)`. No VP count changes (total 31, proptest 10, P1 17 unchanged).

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
- Add holdout scenario: BE byte-order magic u32 `0x1A2B3C4D`, on-disk bytes `1A 2B 3C 4D` (LE section stores same u32 as on-disk bytes `4D 3C 2B 1A`); SHB truncated at byte 15.

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
  SPB total fixed overhead = 16 bytes (block-type:4 + block-total-length:4 +
  original-packet-length:4 + trailing block-total-length:4). Available bytes for
  padded packet data = `block_total_length - 16`; on the raw-block path the only
  body-relative fixed overhead beyond the outer 12-byte frame is the 4-byte
  `original_len` field (SPB_FIXED_OVERHEAD_BYTES = 4). Callers strip padding to
  `captured_len = min(original_len, snaplen)`.
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

**Rev 5 additions — PO must also action the following (Decision 15 / I-5/I-6/I-14 dispatch):**

**BC-2.01.011 (IDB) — interleaved-IDB guard (rev 5 / Decision 15):**
- Add AC: "If an IDB is encountered AFTER the first EPB or SPB has been emitted (i.e.,
  `packets_emitted > 0` at the time of IDB parsing), the reader MUST return `Err` mapping
  to E-INP-013 ('pcapng interface description block after first packet block — unsupported
  ordering'). The interface table is NOT updated for the late IDB; processing stops."
- This is a TIGHT-SCOPE reject consistent with the fail-closed posture and the IDB-first
  ordering of the intended corpus (Wireshark, PacketLife, public fixtures all open
  interfaces before the first packet).

**BC-2.01.016 (linktype whitelist) — timing clarification (rev 5 / Decision 15):**
- Add or update the existing postcondition/AC to state: "The linktype whitelist check
  fires at IDB-parse time, not at first-packet time or after all IDBs. If the linktype
  value in an IDB is not on the whitelist, the reader MUST return `Err` immediately
  when parsing that IDB block, before any packet from that interface is consumed."
- Remove or correct any existing text that implies linktype validation is deferred to
  the packet-processing stage.

**BC-2.01.013 (SPB) — add HS-107 holdout (rev 5 / HS-completeness map):**
- Author HS-107: holdout scenarios for SPB parsing and snaplen clamping. Required
  scenarios: (a) well-formed SPB with IDB-snaplen=96; packet data truncated to 96 bytes;
  (b) SPB with original_len > snaplen (padding present; `captured_len = snaplen` after
  clamping); (c) SPB arriving before any IDB (→ E-INP-009: interface table empty).
- HS-107 is MISSING and must be authored before Phase 4. BC-2.01.013 is untestable
  at the Phase 4 gate without it.

**error-taxonomy.md — add E-INP-013:**
- Add entry: `E-INP-013` — "pcapng interface description block after first packet block
  — unsupported ordering". Severity: error. Category: input. Context: include the block
  sequence number of the late IDB and the block sequence number of the first packet block
  previously emitted. Remediation hint: "capture files with mid-capture interface changes
  are not supported in this cycle; use mergecap to re-capture or filter to a single
  interface."

**VP-025/BC-2.01.014 — implementation note for Kani provability (I-2 resolution):**
- Add implementation note to BC-2.01.014 Verification Properties section: "The base-10
  branch MUST use a precomputed lookup table for e∈[0,19] (saturating to u64::MAX for
  e≥20) OR the VP-025 Kani harness MUST carry `#[kani::unwind(128)]`. Without one of
  these, the Kani proof over symbolic `e` is vacuous (insufficient unwind). Option A
  (lookup table) is preferred: no loop in the pure-core function, proof trivially bounded,
  faster runtime on the common e=6 path." See ADR-009 rev 5 VP-025 Kani Provability Note
  for full analysis.

**Rev 6 additions — PO must also action the following (Decisions 16/17/18 + O-3 dispatch):**

**BC-2.01.011 (IDB) — per-section-reset removal (Decision 16 / H-5):**
- Delete or mark DEFERRED any invariant text that states the interface table resets at each SHB. Because wirerust is single-section-only (Decision 7 rejects the second SHB with E-INP-012 before any second section is parsed), there is no code path by which a per-section reset can ever execute. The invariant describes unreachable behavior; retaining it creates a spec/implementation contradiction. Mark it: "DEFERRED — not implemented; will be required if the single-section constraint is ever lifted." Do NOT implement a reset; do NOT attempt to test it.
- Add the IDB error-code precedence per Decision 17: "If an IDB arrives after the first packet block has been emitted (E-INP-013, position check), the position check fires FIRST, before the linktype whitelist check (E-INP-001) or multi-IDB agreement check (E-INP-011). An IDB that is both late AND has a conflicting linktype receives E-INP-013 only."
- Add edge case: "late IDB (after first packet block) that also conflicts on linktype → E-INP-013 wins; E-INP-001 is never evaluated."

**BC-2.01.016 (linktype whitelist) — precedence clarification (Decision 17 / M-7):**
- State that E-INP-001 is the SECOND check in the IDB evaluation order, after E-INP-013 (position) and before E-INP-011 (agreement). The whitelist check is reached only when the IDB is positionally valid (`packets_emitted == 0`).

**BC-2.01.018 (multi-IDB / per-file isolation) — two changes (Decisions 16 and 17 / H-5 and M-7):**
- Delete or mark DEFERRED any invariant or edge-case text (Inv 4, EC-005) asserting that "each section succeeds individually" or that a multi-section file is parsed per-section. The correct behavior is: the second SHB is rejected with E-INP-012 (not parsed). Correct EC-005: "a pcapng file with two SHB blocks is rejected with E-INP-012 at the second SHB; processing stops; the second section is not parsed; there is no per-section success outcome."
- State that E-INP-011 is the THIRD check in the IDB evaluation order (after E-INP-013 and E-INP-001 per Decision 17). Add edge case: "late IDB that also conflicts on linktype → E-INP-013 wins."

**BC-2.01.013 (SPB) — add VP-031 (Decision 18 / M-2):**
- Add VP-031 to Verification Properties cell.
- VP-031 covers: for all `(original_len: u32, snaplen: u32, body: &[u8])`, `captured_len == min(original_len, snaplen, body.len() as u32)`; the returned slice has exactly `captured_len` bytes; no out-of-bounds access. Tool: proptest. Phase: P1. Module: `reader.rs (pcapng_pure_core fns)`.
- Note: VP-028 (cargo-fuzz) remains assigned to BC-2.01.017 and also exercises SPB via the full pcapng reader no-panic property. VP-031 is the arithmetic/correctness VP; VP-028 is the structural no-panic VP. Both are required.

**O-3 (process-gap) — stale "taxonomy updated in a separate burst" notes:**
- Sweep all BCs in the BC-2.01.NNN series for any note of the form "E-INP-NNN to be added to error-taxonomy.md in a separate burst" or "taxonomy update pending." In particular, E-INP-013 is now in error-taxonomy.md (added per rev 5 dispatch). Any BC referencing E-INP-013 as "pending taxonomy addition" should have that note removed and replaced with "E-INP-013: error-taxonomy.md v2.8 or later." Perform a sweep of all pcapng BCs (BC-2.01.009 through BC-2.01.018) for stale process-gap notes and remove them.

**Rev 7 additions — PO must also action the following (Decisions 19/20/21 + H-3/M-5/H-4 dispatch):**

**BC-2.01.009 (pcapng acceptance) and BC-2.01.015 (block-walk skip) — zero-packet notice citation fix (Decision 19 / M-4):**
- BC-2.01.009 PC6: correct the decision citation from "Decision 17" to "Decision 19." Add postcondition text: "A structurally valid pcapng file that yields `packets.len() == 0` MUST emit exactly one stderr notice before exit 0; see Decision 19 for format."
- BC-2.01.015 PC9: same correction — replace mis-citation of "Decision 17" with "Decision 19." Add: "The zero-packet notice fires at end of block-walk when packets.len()==0; skipped-block count appended when >0; block body bytes MUST NOT appear in the notice (SEC-007)."

**BC-2.01.010 (SHB) — re-add body-too-short E-INP-008 case (Decision 20 / H-1):**
- RE-ADD the case: `12 <= block_total_length < 28` → wirerust body-decode failure → **E-INP-008**. The crate accepts btl >= 12 (framing ok); wirerust receives a body of btl-12 bytes; if body < 16 bytes (SHB fixed-field minimum), wirerust MUST return E-INP-008. The pass-3 claim "SHB body-too-short is unconstructible" is incorrect; this case IS constructible and MUST be handled.
- State clearly: framing failure (btl < 12, crate rejects) → E-INP-010; body-decode failure (btl >= 12 but body < 16) → E-INP-008; semantic failure (body >= 16, bad BOM or major != 1) → E-INP-008; well-formed → proceed.
- Add holdout scenario: SHB with `block_total_length = 16` (body = 4 bytes, < 16 fixed-field minimum) → E-INP-008.

**BC-2.01.011 (IDB) — if_tsoffset removal + body-too-short note (Decisions 20 / 21):**
- PC6 options walk: REMOVE "and if_tsoffset (code 10)." The IDB options walk extracts only `if_tsresol` (code 9) in this cycle. Add limitation: "if_tsoffset (option code 10) is NOT extracted or applied in this cycle."
- Add body-too-short note per Decision 20 table: IDB `12 <= btl < 20` → body < 8 fixed-field bytes → E-INP-008.

**BC-2.01.012 (EPB) — body-too-short note (Decision 20 / H-1):**
- Add or verify: EPB `12 <= btl < 32` → body < 20 fixed-field bytes → E-INP-008. Already partially implied by the fixed-overhead constant; make it explicit per the Decision 20 table.

**BC-2.01.013 (SPB) — body-too-short note (Decision 20 / H-1):**
- Add: SPB `btl == 12` → body = 0 bytes < 4 fixed-field bytes → E-INP-008. (btl < 12 is framing failure → E-INP-010 from crate; btl == 12 is the minimum accepted btl with 0-byte body.) Add holdout scenario: SPB with `block_total_length = 12` → E-INP-008.

**BC-2.01.014 (timestamp helper) — if_tsoffset limitation note (Decision 21):**
- Add limitation: "This helper does not accept or apply if_tsoffset (IDB option code 10). Files with if_tsoffset set will have a timestamp bias equal to offset × 1/ticks_per_sec seconds. This is an accepted limitation for this cycle; see Decision 21."

**error-taxonomy.md — E-INP-008 and E-INP-010 scope text updates (Decision 20 / H-1):**
- E-INP-008 scope: "wirerust body-decode failure for any block type after successful crate framing — body shorter than the block's required fixed-field bytes (SHB: 16 body bytes, IDB: 8, EPB: 20, SPB: 4), or body semantically invalid (SHB: BOM != known magic or major version != 1)."
- E-INP-010 scope: "block framing rejection by pcap-file 2.0.0 — block_total_length < 12, block_total_length % 4 != 0, or EOF before trailing length field. The crate owns this check; wirerust maps all next_raw_block Err variants to E-INP-010."

**error-taxonomy.md — E-INP-010 / E-INP-012 / E-INP-013 context string seq convention (M-5):**
- Update Context field of E-INP-010, E-INP-012, and E-INP-013 to state: "block #<seq> where seq is the 1-based block index; the SHB is block #1; every block returned by next_raw_block increments the counter." Remove any 0-based qualifier.

**Holdout scenarios — author HS-108 (H-4):**
- HS-108: "Zero-packet notice end-to-end." Required scenarios: (a) valid pcapng file with SHB + IDB + no EPBs/SPBs → stdout empty, stderr contains exactly one notice, exit 0; (b) same file but with 2 unknown-type blocks skipped → notice includes "(2 blocks skipped)"; (c) malformed pcapng (parse error) → normal error on stderr, exit 1, NO zero-packet notice.
- HS-108 is MISSING. The Phase 4 gate cannot test the zero-packet notice (Decision 19 / SOUL-#4 anchor) without it.

**VP-030 — domain restatement (H-3):**
- Update BC-2.01.018 Verification Properties entry for VP-030: "VP-030 (proptest, P1): generates sequences of WHITELISTED DataLink values only (the domain where the E-INP-011 conflict check is reachable). Non-whitelisted values short-circuit to E-INP-001 at IDB-parse time and never reach the agreement check. Property: all-equal → Ok; first-differing whitelisted DataLink → Err(E-INP-011) on that IDB. Comparison unit: DataLink, not raw u16."

**Rev 8 additions — PO must action the following (Decision 20 C-1, Decision 9 amendment, Decision 19 amendments, VP property updates):**

**BC-2.01.012 (EPB) — reclassify padding-overrun + bound-by-body → E-INP-008 (Decision 20 / C-1):**
- In BC-2.01.012, error-taxonomy, HS-104 Case E, and VP-027: RECLASSIFY the EPB padding-overrun check (`20 + captured_len + pad_len(captured_len) > body.len()`) and the bound-by-body check (`captured_len > body.len() - 20`) from E-INP-010 → **E-INP-008**. These are WIRERUST body-decode failures computed AFTER the crate successfully frames the block. Framing (btl < 12, misaligned) is crate-owned → E-INP-010. Body-decode (body too short for fixed fields, padded extent overruns available bytes, captured_len exceeds body) is wirerust-owned → E-INP-008. The uniform rule (Decision 20 Tier 2) governs; C-1 makes this explicit for the EPB overrun cases.
- HS-104 Case E must be updated: "EPB with captured_len = N and padding such that captured_len + pad_len > body.len() → E-INP-008 (NOT E-INP-010)."
- VP-027 property must include: "padding-overrun (`20 + captured_len + pad_len(captured_len) > body.len()`) → Err(E-INP-008); bound-by-body (`captured_len > body.len() - 20`) → Err(E-INP-008)."
- error-taxonomy.md E-INP-010 description: remove item (c) if it references EPB body-decode checks; add note "E-INP-010 is strictly crate-side framing rejection; wirerust body-decode failures use E-INP-008."

**BC-2.01.013 (SPB) — drop snaplen from captured_len formula (Decision 9 rev 8 / H-3 + M-2):**
- PC1/AC-002/EC-007: Update SPB captured_len formula from `min(original_len, snaplen, block_body_available)` to `min(original_len, block_body_available)`. Snaplen is NOT consulted for SPB captured-len derivation. `block_body_available` is `body.len() as u32` (raw bytes in the RawBlock body after the 4-byte original_len field).
- HS-107 Case E: If a scenario relies on snaplen clamping SPB data, remove or restate it. SPB captured_len clamps only to `original_len` and `body.len()`.
- VP-031 property: change from `min(original_len, snaplen, body.len() as u32)` to `min(original_len, body.len() as u32)`. The `snaplen` argument is removed from the pure-core helper domain. The property is now: "for all (original_len: u32, body: &[u8]), captured_len == min(original_len, body.len() as u32); returned slice has exactly captured_len bytes; no OOB access."

**BC-2.01.014 (timestamp helper) — µs fast path saturation + VP-025 vector (Decision 19 / M-3):**
- Any µs fast path in BC-2.01.014 (e.g., a shortcut for the default tsresol=6 case) MUST apply the same ts_sec saturation (`.min(u32::MAX)`) as the general formula. The shortcut MUST NOT skip saturation under any input. State this explicitly in BC-2.01.014.
- VP-025 Kani harness: MUST include a large-ts_high vector where `ticks / ticks_per_sec > u32::MAX` (e.g., ts_high = u32::MAX, ts_low = u32::MAX, if_tsresol = 6) so the saturation `.min(u32::MAX)` is exercised and locked by the proof.

**BC-2.01.009 (pcapng acceptance) and BC-2.01.015 (block-walk skip) — zero-packet notice OPB distinction + emission point (Decision 19 rev 8 / H-2 + M-5):**
- BC-2.01.009: Update the zero-packet notice postcondition to state that PcapSource MUST surface `skipped_blocks: u32` and `opb_skipped: u32`. The notice is EMITTED from main.rs, NOT from from_pcap_reader. The notice format when OPB blocks were skipped MUST distinguish them (per Decision 19 rev 8 OPB distinction text). Add: "A structurally valid classic pcap file with zero packets MUST also emit the zero-packet notice with format `notice: <filename>: 0 packets read from pcap file`."
- BC-2.01.015: Update the skipped-block counter postcondition to state that `PcapSource::skipped_blocks` includes OPB and that `PcapSource::opb_skipped` is the OPB-specific sub-count. The COUNTER is surfaced (not emitted) — main.rs reads it and emits the distinction.
- HS-108 (MISSING — PO must author): extend the required scenarios to include: (d) valid pcapng with SHB + IDB + 1 OPB (no EPBs) → notice includes OPB count and re-save hint; (e) valid pcapng with 2 NRB blocks skipped + 1 OPB skipped → notice distinguishes OPB from NRB count.

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
