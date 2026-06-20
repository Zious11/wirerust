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

# ADR-009: pcapng Capture-Format Reader Support

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

**Decision 1 — Parser: Option A (use pcap-file 2.0.0's existing pcapng reader, +0 new crates).** wirerust will use `PcapNgReader` / `PcapNgParser` from the `pcapng`
module of the already-vendored `pcap-file` 2.0.0 crate. This crate is already
compiled into every wirerust build; the pcapng code path is dead code today. Pinning
`pcap-file` 3.0.0 is explicitly rejected: as of 2026-06-19 only `3.0.0-rc.2` exists,
and wirerust's supply-chain posture prohibits adopting release candidates as hard
dependencies.

**Decision 2 — Block coverage.** The reader MUST handle SHB (byte-order detection
and version), IDB (per-interface `linktype: DataLink` and `if_tsresol` option
extraction), EPB (packet data and 64-bit timestamp), and SPB (packet data, no
timestamp). Unknown block types MUST be silently skipped using the block-total-length
field; neither a warning nor an error is emitted for unknown types.

**Decision 3 — Multi-IDB policy.** A pcapng file may contain multiple IDB blocks.
wirerust requires all IDB blocks in a section to agree on `linktype`. If two or
more IDBs carry differing `linktype` values, the reader returns an error with context
identifying the conflicting link types. This preserves the single-`DataLink` model
in `PcapSource.datalink` with zero changes to `decoder.rs`, `RawPacket`, or any
downstream consumer.

**Decision 4 — 64-bit timestamp normalization.** The EPB 64-bit timestamp MUST be
converted to `(ts_sec: u32, ts_usecs: u32)` using the `if_tsresol` exponent from
the IDB. When `if_tsresol` is absent, the default resolution is 10^-6 (microseconds)
per the pcapng specification. A pure-core timestamp-conversion helper MUST be
extracted (free function or standalone module, no I/O), taking `(ts_high: u32,
ts_low: u32, if_tsresol: u8)` and returning `(ts_sec: u32, ts_usecs: u32)`.
This function is formally verifiable.

**Decision 5 — Magic-byte probe discipline.** The entry point `PcapSource::from_pcap_reader<R: Read>` receives a stream and MUST peek the first four bytes without
advancing the stream position before selecting the parser. The implementation
MUST wrap the reader in `std::io::BufReader` (if not already) and use
`fill_buf()` + `consume()` — or equivalent non-destructive peek — to read the magic
without consuming bytes. The existing `PcapReader::new` classic-pcap path, including
its `next_raw_packet()` discipline (not `next_packet()`), MUST remain unchanged for
the classic-pcap branch; the snaplen-truncation contract documented in
`src/reader.rs:13-18` continues to apply to that branch.

**Decision 6 — BC-2.01.004 retirement.** BC-2.01.004 ("Reject pcapng-Format Input
at Reader Level") is retired and its normative postconditions are inverted. The test
`test_BC_2_01_004_rejects_pcapng` MUST be rewritten as a positive acceptance test
for `smb3.pcapng` in the same story that retires the BC. The replacement BC
(BC-2.01.009, "Accept pcapng Format: Transparent Detection via Magic-Byte Probe")
supersedes BC-2.01.004.

**Fallback — Option C (hand-roll, +0 crates).** If during implementation
`pcap-file` 2.0.0's pcapng reader exhibits a snaplen/truncation defect analogous to
the `next_packet()` validated-path bug already documented in `src/reader.rs:13-18`,
the escalation path is a hand-rolled minimal pcapng block walker (~300 LOC), NOT
Option B (`pcap-parser` 0.17). Option B is permanently rejected (see Rationale).

## Rationale

The dependency decision (Option A vs B vs C) is dispositive. The research evaluation
(`pcapng-parser-dependency-eval.md`, 2026-06-19) establishes that `pcap-file` 2.0.0
already ships a full pcapng reader with SHB, IDB, EPB, SPB support and typed
`IfTsResol`/`IfTsOffset` option variants — confirmed against docs.rs/pcap-file/2.0.0.
The crate is already locked in `Cargo.lock:817-826` with no new transitive
dependencies required. Option A therefore delivers the required capability at +0 new
crates with +0 supply-chain audit burden — the strongest possible outcome given
wirerust's minimal-dependency NFR.

Option B (`pcap-parser` 0.17.0) is rejected on supply-chain grounds. It introduces
approximately four new crates: `pcap-parser` itself, `nom` 8.x, `rusticata-macros`
5.x, and `circular` 0.3. Critically, wirerust's existing TLS-parser dependency tree
already carries `nom` 7.1.3 and `rusticata-macros` 4.1.0; `pcap-parser` 0.17's
requirement for major-version 8/5 of those crates creates two duplicate major-version
pairs in the build graph, increasing compile time, binary size, and supply-chain audit
surface for precisely zero capability gain over `pcap-file` 2.0.0. The `DataLink`
type-continuity argument further favors Option A: `InterfaceDescriptionBlock.linktype`
is the same `pcap_file::DataLink` type that `decoder.rs` already imports at ~12 call
sites, requiring zero translation glue.

Option C (hand-roll, ~300 LOC) is viable and consistent with wirerust's house style
(ADR-007's DNP3 parser is hand-rolled; `decoder.rs` hand-rolls the SLL header parse).
However, Option C adds first-party attack surface that `pcap-file` 2.0.0 already
eliminates for free, and hand-rolling a block walker that already exists in a
well-exercised library (10M+ downloads) is development cost without commensurate
benefit. Option C is preserved as an escalation path, not a primary choice.

The multi-IDB policy (require `linktype` agreement, reject on conflict) is selected
over per-packet link-type dispatch because it preserves the `PcapSource.datalink`
single-field model with zero impact on `decoder.rs`, `reassembly/`, or any analyzer.
Per-packet `DataLink` threading would require structural changes to `RawPacket` and
all call sites — a scope expansion inconsistent with this feature cycle's boundary of
"zero analyzer changes."

The timestamp-conversion helper is extracted as a pure-core function specifically to
enable Kani-assisted property verification. The conversion from 64-bit pcapng
timestamp to `(ts_sec, ts_usecs)` involves division and modular arithmetic with a
resolution exponent; this is precisely the class of integer arithmetic where formal
methods add high assurance at low cost. The function is pure (no I/O, no global
state, deterministic) and feasible to Kani-prove. The `if_tsresol`-absent default
of 10^-6 (microseconds) is the pcapng specification's normative default and matches
the resolution already used in the classic-pcap path for microsecond-precision files.

The magic-byte probe discipline (peek without consuming) is required because
`from_pcap_reader<R: Read>` receives an opaque `Read` implementation that may not
support `Seek`. The BufReader `fill_buf()` + `consume()` pattern is the idiomatic
zero-copy peek for non-seekable streams. This constraint is called out explicitly
because an incorrect implementation (advancing past the magic bytes before handing
the reader to `PcapNgParser`) would produce a hard-to-diagnose parse error on every
pcapng file.

## Consequences

### Positive

- SS-01 supports both classic libpcap and pcapng at +0 new dependencies; supply-chain
  audit surface is unchanged.
- `arp-baseline-16pkt.cap` (pcapng-with-.cap-extension), public TLS corpus pcapng
  files, and all modern Wireshark-default captures become processable without format
  conversion.
- The pure-core timestamp-conversion helper is provable by Kani, extending the
  project's formal-verification coverage into SS-01 for the first time.
- The `DataLink` type flows directly from `idb.linktype` to `PcapSource.datalink`
  with zero translation code; no type-mapping table is introduced.
- `src/decoder.rs`, all analyzers, reassembly, dispatcher, and all reporters require
  zero changes.
- BC-2.01.004's test inversion (reject → accept for `smb3.pcapng`) corrects a
  fixture-level lie that has persisted since brownfield ingestion.

### Negative / Trade-offs

- MEDIUM regression risk on the classic-pcap path: adding a peek branch at the top
  of `from_pcap_reader` is a new code path adjacent to the hot path. The full
  existing reader test suite MUST be green before any pcapng story merges.
- The `if_tsresol` option handling in `pcap-file` 2.0.0's pcapng reader has not been
  exercised under wirerust's conditions. The research evaluation flags this as a
  partially inconclusive area: the `IfTsResol(u8)` variant is confirmed present, but
  correct end-to-end application of the exponent to EPB 64-bit timestamps has not been
  validated at runtime. A unit test on a nanosecond-resolution pcapng fixture is
  mandatory before STORY-125 merges.
- If `pcap-file` 2.0.0's pcapng path exhibits a snaplen/truncation defect (analogous
  to the known `next_packet()` validation bug in the classic path), the fallback to
  Option C (~300 LOC hand-roll) adds implementation scope to a future cycle.
- Adding `*.pcapng` to the `src/main.rs` directory glob means malformed pcapng files
  that were silently excluded now produce errors at the reader level. This is correct
  behavior but changes the user-visible error surface.
- The multi-IDB link-type-agreement policy will reject legitimate multi-NIC capture
  files that mix Ethernet and, e.g., Linux Cooked interfaces. This is a known
  limitation, documented in BC-2.01.018.

### Status as of 2026-06-19

Proposed. `pcap-file` 2.0.0's pcapng module is dead code in the compiled binary;
`src/reader.rs` does not import it. BC-2.01.004 ("Reject pcapng-Format Input at Reader
Level") was RETIRED during F2 spec evolution and is superseded by BC-2.01.009 ("Accept
pcapng Format: Transparent Detection via Magic-Byte Probe"); the spec changes are complete.
Only the implementation remains pending, scheduled across STORY-123 through STORY-127
(F3 story decomposition forthcoming).

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
- **Crate API authority:** docs.rs/pcap-file/2.0.0 (`pcapng` module, `InterfaceDescriptionBlock`, `InterfaceDescriptionOption` enum — `IfTsResol`/`IfTsOffset` confirmed present); crates.io API verified 2026-06-19
- **Format specification:** IETF draft `draft-ietf-opsawg-pcapng` (canonical: `github.com/IETF-OPSAWG-WG/pcapng`); block types SHB `0x0A0D0D0A`, IDB `0x00000001`, SPB `0x00000003`, EPB `0x00000006`; `if_tsresol` IDB option code 9
- **ADR precedent:** ADR-007 (hand-rolled DNP3 parser, Issue #8) — establishes "hand-roll binary parsers when no crate exists"; this ADR establishes the converse: "use the crate when it is already vendored and capable"
