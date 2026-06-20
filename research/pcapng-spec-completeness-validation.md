# pcapng F2 Spec-Completeness Validation

**Date:** 2026-06-19
**Agent:** vsdd-factory:research-agent
**Scope:** Validate the COMPLETENESS of wirerust's F2 pcapng-reader spec (ADR-009; BC-2.01.009..018; error taxonomy E-INP-008..011) against the authoritative pcapng specification (IETF draft `draft-ietf-opsawg-pcapng` / pcapng.com / WinPcap original) and against the real-world captures this cycle intends to unlock, BEFORE F3 story decomposition locks scope.
**Gate purpose:** Find coverage GAPS that would cause real pcapng files to fail or be silently mis-read.

---

## VERDICT

**COMPLETE for the intended real-world pcapng corpus** — with two NON-BLOCKING gaps flagged for explicit F3 acknowledgement and one spec-precision correction.

The 10 BCs (BC-2.01.009 through BC-2.01.018) are sufficient to correctly READ packets, timestamps, and link type from the vast majority of real-world pcapng files, **and specifically from every file in the intended unlock corpus** (`arp-baseline-16pkt.cap`, `smb3.pcapng`, Wireshark `dump.pcapng`, `tls12-dsb.pcapng`, and dumpcap/Wireshark-default captures generally). These are all single-section, single-IDB, little-endian, base-10-timestamp files — the exact happy path the BCs cover.

Two genuine completeness gaps exist (multi-section handling and base-2 `if_tsresol`), but **neither affects the intended corpus**, and one of them (base-2 `if_tsresol`) is already covered at the BC level and only at risk at the implementation/parser level. They are flagged below so F3 can either (a) accept them as documented known-limitations like the multi-IDB policy, or (b) add explicit ACs.

The single most important confirmation for this cycle: **the Decryption Secrets Block (DSB) is safe to skip for wirerust** — it carries no packet data and does not change EPB interpretation. `tls12-dsb.pcapng` will be read correctly (handshake metadata intact; only TLS decryption, which wirerust does not do, is unavailable). CONFIRMED-OK, not a gap.

---

## Coverage Matrix — pcapng Block Type → wirerust Disposition

Block type codes verified against the IETF pcapng draft, the WinPcap/pcapng.com original spec, and the `pcap-file` 2.0.0 `Block` enum (docs.rs).

| Block | Code (hex) | Carries packet data? | Affects EPB interpretation? | wirerust disposition | Safe? |
|-------|-----------|----------------------|-----------------------------|----------------------|-------|
| Section Header Block (SHB) | `0x0A0D0D0A` | No | Yes (endianness, section, version) | **HANDLED** — BC-2.01.010 | OK |
| Interface Description Block (IDB) | `0x00000001` | No | Yes (linktype, snaplen, if_tsresol) | **HANDLED** — BC-2.01.011 | OK |
| Packet Block / "Obsolete Packet Block" (OPB) | `0x00000002` | **YES (legacy)** | No | **SKIPPED as Unknown-class** (BC-2.01.015 EC-002) | **GAP-LOW** (see F-08) |
| Simple Packet Block (SPB) | `0x00000003` | YES | No | **HANDLED** — BC-2.01.013 | OK |
| Name Resolution Block (NRB) | `0x00000004` | No | No | **SKIPPED** (metadata only) | OK |
| Interface Statistics Block (ISB) | `0x00000005` | No | No | **SKIPPED** — BC-2.01.015 EC-001 | OK |
| Enhanced Packet Block (EPB) | `0x00000006` | YES (primary) | n/a (is the container) | **HANDLED** — BC-2.01.012 | OK |
| Systemd Journal Export Block (SJE) | `0x00000009` | Journal events (not packets) | No | **SKIPPED** (non-packet records) | OK |
| Decryption Secrets Block (DSB) | `0x0000000A` | **No** | **No** | **SKIPPED** (secrets only) | **OK — CONFIRMED safe (F-01)** |
| Custom Block | `0x00000BAD` | No (vendor metadata) | No | **SKIPPED** | OK |
| Custom Block ("copy" variant) | `0x40000BAD` | No (vendor metadata) | No | **SKIPPED** | OK |
| Any unknown/future block (MSB-set local range, etc.) | any | Unknown | No | **SKIPPED via block-total-length** — BC-2.01.015 | OK |

**Spec guarantee underpinning all SKIPPED rows:** Every pcapng block — present and future — begins with a 4-byte Block Type followed by a 4-byte Block Total Length, with a redundant trailing Block Total Length; this is mandated by the General Block Structure clause specifically so unrecognized blocks can always be skipped by length. So "skip as unknown block" is provably safe for any block wirerust does not parse, **with the one caveat** that the obsolete Packet Block (`0x00000002`) is the only skipped block that actually carries packet data (see F-08).

---

## Findings

Each finding is tagged **GAP** (real coverage hole), **CONFIRMED-OK** (spec confirms current spec is correct/complete), or **RECOMMENDATION**. Severity reflects real-world impact on reading the *intended* corpus and on general pcapng robustness.

### F-01 — DSB is safe to skip for a TLS-metadata analyzer — CONFIRMED-OK (HIGH importance)

**Question #1 (DSB-for-TLS).** The Decryption Secrets Block (`0x0000000A`) is the single highest-stakes question for this cycle because the unlock corpus is TLS-heavy and includes `tls12-dsb.pcapng` (a capture whose entire purpose is to carry a DSB).

The IETF draft characterizes the DSB as a block storing "(session) secrets that enable decryption of packets within the capture file" — TLS key-log entries (NSS `SSLKEYLOGFILE` format), WireGuard keys, ZigBee/802.15.4 keys, etc. Definitively:

- **The DSB does NOT contain packet bytes.** Packet data lives only in EPB/SPB/(legacy)PB.
- **The DSB does NOT change EPB interpretation.** EPB timestamps, link type, captured/original length, and packet bytes are determined solely by SHB + IDB + the EPB's own fields. The presence or absence of a DSB changes nothing about how an EPB is decoded; it only adds the *capability* to decrypt already-captured encrypted payload.
- **All TLS handshake metadata wirerust analyzes is in the cleartext packet bytes, not the DSB.** SNI, JA3/JA3S, cipher suites, supported groups, and certificate messages are all carried unencrypted in the ClientHello/ServerHello/Certificate handshake records, which are ordinary packet bytes inside EPBs.

**Conclusion:** wirerust does not decrypt TLS, so skipping the DSB is completely safe and lossless. `tls12-dsb.pcapng` will be read correctly: all packets, all handshake metadata, intact. Only the encrypted application-data decryption — out of scope for wirerust — is unavailable, and that would be unavailable anyway.

**Spec-precision note:** In `pcap-file` 2.0.0 the DSB has **no typed `Block` variant** (the enum has no `DecryptionSecrets`); a DSB therefore arrives as `Block::Unknown` and is skipped by the unknown-block path. This is the correct outcome, but BC-2.01.015 names only ISB and OPB as its worked examples. See F-07.

*Sources: IETF draft pcapng §Decryption Secrets Block; Wireshark wiki Custom/Decryption documentation; pcapng.com. Deep-research synthesis (Perplexity sonar-deep-research), 2026-06-19.*

### F-02 — Block-type coverage (SHB/IDB/EPB/SPB) is sufficient to read real captures — CONFIRMED-OK (HIGH)

**Question #1 (block coverage).** SHB + IDB + EPB + SPB is sufficient to read packets from the overwhelming majority of real pcapng files. In practice "only Enhanced Packet Blocks are used to store packets in pcapng files" (pcapng.com); SPB is rare; everything else (NRB, ISB, DSB, SJE, Custom) is metadata that never carries packet bytes and never alters EPB decoding. Skipping all of them is safe and lossless with respect to packet bytes + timestamps + link type. **The only block that is both (a) packet-bearing and (b) not parsed by wirerust is the obsolete Packet Block** — see F-08.

*Sources: IETF draft pcapng §3 (block taxonomy), §EPB; pcapng.com; WinPcap original spec Appendix B (block codes). Perplexity deep-research, 2026-06-19.*

### F-03 — Endianness / Byte-Order Magic — CONFIRMED-OK; per-section is correct (HIGH)

**Question #2 (endianness).** BC-2.01.010 correctly requires per-section byte-order detection from the SHB Byte-Order Magic (`0x1A2B3C4D` read in native order = big-endian on-disk bytes `1A 2B 3C 4D`; little-endian on-disk bytes `4D 3C 2B 1A`). The spec is explicit that endianness is a **per-section** property, declared by each SHB, applying to all multi-octet fields in that section. BC-2.01.010 Invariant 1 and EC-006 correctly state that each SHB resets byte order for its section. The *spec content* is complete and correct. The *implementation* risk lives in the parser (F-06).

*Sources: IETF draft pcapng §SHB Byte-Order Magic; pcapng.com; Rust `pcap_parser` docs ("endianness of a block is indicated by the SHB that started the section"). Perplexity deep-research, 2026-06-19.*

### F-04 — Multi-section files: spec is correct; interface-index reset is correctly specified — CONFIRMED-OK at BC level (MEDIUM)

**Question #3 (multi-section).** A single pcapng file may legally contain multiple SHB sections concatenated, and different sections may have different endianness. Critically, **interface indexes are scoped per-section and reset to 0 at each new SHB** — an EPB's `interface_id` references an IDB in the *same* section only. This is confirmed by the spec, by libpcap's reader (which validates `interface_id < current-section interface count`), and negatively by a real Wireshark bug (gitlab #) where packets in later sections were wrongly attributed to first-section interfaces.

wirerust's BCs get this **right at the spec level**: BC-2.01.010 Invariant 1 + EC-006 (byte order resets per SHB), BC-2.01.011 Invariant 2 (interface table resets at each SHB; indexes are 0-based per section), and BC-2.01.018 Invariant 4 + EC-005 (multi-IDB agreement check applies per section; per-section isolation).

**However — see F-06:** the chosen parser (`pcap-file` 2.0.0) appears NOT to implement per-section reset, so the BC-level correctness is not guaranteed to be delivered by the implementation. The BC text is complete; the parser may not honor it. This is the most important gap to resolve in F3.

*Sources: IETF draft pcapng §Sections, §SHB; libpcap pcapng reader source; Wireshark multi-section bug report; Rust `pcap_parser` docs. Perplexity deep-research, 2026-06-19.*

### F-05 — Power-of-2 `if_tsresol` IS handled at the BC level — CONFIRMED-OK (MEDIUM)

**Question #5 (timestamp edge cases).** The concern was that BC-2.01.014's normalization might assume base-10 (decimal) `if_tsresol` only. **It does not.** The spec defines `if_tsresol` (IDB option code 9, 1 byte): MSB (bit 7) clear → remaining 7 bits are a base-10 exponent (resolution `10^-e`); MSB set → remaining 7 bits are a base-2 exponent (resolution `2^-e`); absent → default `10^-6` (microseconds, matching classic libpcap).

The wirerust spec correctly covers BOTH bases:
- BC-2.01.011 Postcondition 2 and EC-003 explicitly handle bit-7-set (base-2).
- BC-2.01.014 Postcondition 3 specifies the base-2 branch (`ticks_per_sec = 1u64 << e`), Postcondition 2 the base-10 branch, and EC-005/EC-006 are base-2 test vectors. Default `e=6` is correct.

This is **not a gap** — the spec already avoids the decimal-only trap. Two minor robustness notes:
1. **Real-world frequency:** base-2 `if_tsresol` is fully specified but very rare in practice; mainstream tools (Wireshark, dumpcap, tcpdump) emit base-10 (exponent 6 or 9). The intended corpus uses base-10. So base-2 correctness is a robustness/conformance property, not a corpus blocker.
2. **Implementation delivery risk:** `pcap-file` 2.0.0 exposes `IfTsResol(u8)` as the **raw byte with no interpretation** (docs.rs confirms the crate does not apply the exponent). This is exactly why BC-2.01.014 is a wirerust-owned pure-core function — good design. F3 must ensure the base-2 branch is actually unit-tested, since no real corpus file will exercise it (Kani over the full `u8` space, already specified in BC-2.01.014 VPs, covers this).

*Sources: IETF draft pcapng §if_tsresol; pcapng.com (`2^-10 = 1/1024 s` example); docs.rs/pcap-file/2.0.0 `InterfaceDescriptionOption::IfTsResol(u8)`. Perplexity deep-research + WebFetch docs.rs, 2026-06-19.*

### F-06 — `pcap-file` 2.0.0 likely does NOT reset interface state per section — GAP (MEDIUM severity, LOW corpus impact)

**Question #3/#4 (parser limitations).** Inspection of `pcap-file` 2.0.0's `PcapNgReader` (docs.rs API + source) indicates it tracks a **single "current" section** and accumulates IDBs in one growing interface list (`interfaces()` returns "all the current InterfaceDescriptionBlock") with **no visible per-section reset** when a second SHB is encountered. The `section()` accessor is singular ("the current SectionHeaderBlock").

**Consequence:** For a genuine multi-section file, the parser would likely carry first-section interfaces into later sections, producing exactly the mis-attribution class of bug seen historically in Wireshark. This **contradicts the delivery of** BC-2.01.010 EC-006, BC-2.01.011 Invariant 2, and BC-2.01.018 EC-005, which assert correct per-section behavior. The BC text promises behavior the chosen parser may not provide.

**Severity rationale:** MEDIUM as a spec/implementation consistency defect; **LOW** in real-world corpus impact because (a) Wireshark/dumpcap-generated pcapng is essentially always single-section (Wireshark's own pcapng implementation historically supports only one section per file), and (b) the entire intended unlock corpus is single-section. Multi-section files arise mainly from `cat`-style concatenation or merge tools.

**This is partially INCONCLUSIVE:** the determination is from API shape and source reading, not a runtime test against a crafted multi-section fixture. F3 should verify empirically.

**Recommendation (F3):** Pick ONE:
- (a) **Document as known-limitation** (mirroring the multi-IDB fail-closed policy): detect a second SHB and either reject with a clear error ("multi-section pcapng not supported") or process only the first section. Add an AC + an E-INP-008-class error. This is the tight-scope choice and matches the cycle's "single DataLink" posture.
- (b) **Verify pcap-file actually resets per-section** with a crafted 2-section fixture; if it does, keep the BCs as-is and add the fixture as a regression test. If it does not, fall back to (a).

Either way, the current BC text (which silently *assumes* correct multi-section handling) should be made explicit so it is not a latent silent-mis-read.

*Sources: docs.rs/pcap-file/2.0.0 `pcapng` module; GitHub `courvoif/pcap-file` source (WebFetch). Flagged INCONCLUSIVE pending runtime test. 2026-06-19.*

### F-07 — Unknown-block skip via `pcap-file` typed variants vs. raw skip — RECOMMENDATION (LOW)

BC-2.01.015 frames unknown-block skipping purely as "read and discard `block_total_length - 8` bytes," and names only ISB (`0x5`) and OPB (`0x2`) as examples. But `pcap-file` 2.0.0 does NOT hand wirerust raw bytes for most of these — it returns **typed `Block` variants**: `NameResolution`, `InterfaceStatistics`, `SystemdJournalExport`, `Packet` (OPB), and `Unknown` (the catch-all, which is where DSB/Custom/future blocks land, since 2.0.0 has no `DecryptionSecrets` or `Custom` variant).

So wirerust's actual skip logic is a **match-arm coverage** problem, not a byte-counting problem: the reader must produce a `RawPacket` only for `EnhancedPacket` and `SimplePacket`, drive interface state from `SectionHeader`/`InterfaceDescription`, and **silently ignore** `NameResolution | InterfaceStatistics | SystemdJournalExport | Packet | Unknown` (and the unreachable `DecryptionSecrets`/`Custom` once on pcap-file 3.x). BC-2.01.015 is not *wrong* (the spec-level invariant holds), but its examples are incomplete relative to the chosen parser's surface.

**Recommendation (F3):** Add to BC-2.01.015 (or its AC) an explicit enumeration that ALL of `Block::NameResolution`, `Block::InterfaceStatistics`, `Block::SystemdJournalExport`, `Block::Packet`, and `Block::Unknown` are skip arms, so the implementer cannot accidentally omit one (e.g., forgetting `NameResolution` and panicking on a `todo!()` arm). This is a defensive-completeness clarification, not a correctness gap.

*Sources: docs.rs/pcap-file/2.0.0 `Block` enum variants (SectionHeader, InterfaceDescription, Packet, SimplePacket, NameResolution, InterfaceStatistics, EnhancedPacket, SystemdJournalExport, Unknown). WebFetch, 2026-06-19.*

### F-08 — Obsolete Packet Block (OPB, `0x00000002`) is the one skipped block carrying packet data — GAP (LOW)

OPB is the legacy predecessor of EPB. It is marked obsolete (SHOULD NOT appear in new files) but **does carry captured packet bytes**. wirerust skips it (BC-2.01.015 EC-002 / `Block::Packet` arm). For OPB-bearing files, those packets are silently lost — the only case in the entire matrix where skipping loses packets rather than metadata.

**Severity LOW:** OPB is obsolete; Wireshark and modern tools emit EPB exclusively. No file in the intended corpus uses OPB. The F1 delta appendix even suggested "handle as EPB or skip." Skipping is acceptable for this cycle.

**Recommendation (F3):** Accept as documented known-limitation. Optionally, since OPB's layout is EPB-like (interface_id, ts_high/low, captured_len, original_len, data), a future cycle could promote `Block::Packet` to a packet-producing arm cheaply. Not required now. If accepted, add one line to ADR-009 Consequences ("obsolete Packet Block packets are skipped, not read") so the loss is not silent at the spec level.

*Sources: IETF draft pcapng §Packet Block (obsolete); WinPcap original spec. Perplexity deep-research, 2026-06-19.*

### F-09 — Block padding / 32-bit alignment / trailing length — CONFIRMED-OK (LOW)

**Question #4 (padding/alignment).** All blocks are 32-bit aligned; `block_total_length` is a multiple of 4 and includes the type field, both length fields, body, and padding; Captured Packet Length excludes trailing alignment padding. Relying on `pcap-file` 2.0.0 to walk blocks by `block_total_length` and strip alignment padding is appropriate — this is core, long-exercised functionality of the crate (10M+ downloads). No known padding/alignment defects surfaced for `pcap-file` 2.x in the available sources. The redundant trailing Block Total Length (used for backward navigation) is handled internally by the crate; wirerust does not need to consume it directly.

**Note:** This is the area where ADR-009's own escalation path (Option C hand-roll) exists precisely if a `pcap-file` 2.0.0 snaplen/truncation defect appears, analogous to the documented classic-path `next_packet()` bug. No such pcapng-path defect was found in research, but absence-of-evidence is not proof; the ADR's fallback is the correct hedge.

*Sources: IETF draft pcapng §General Block Structure; pcapng.com. Perplexity deep-research, 2026-06-19.*

### F-10 — EPB captured-len vs original-len matches the classic snaplen contract — CONFIRMED-OK (MEDIUM)

**Question #6 (snaplen/truncation).** The spec defines Captured Packet Length = "number of octets captured from the packet … the minimum of the Original Packet Length and the SnapLen for the interface" and Original Packet Length = "actual length of the packet … on the wire / frame length." This is an exact 1:1 mapping to classic libpcap `incl_len` (= Captured) and `orig_len` (= Original), with IDB SnapLen playing libpcap's snaplen role. BC-2.01.012 correctly mandates using `captured_length` (never `original_length`) to bound the data slice (PC3, Invariant 2, EC-002), which matches the classic-pcap snaplen-truncation contract documented in `src/reader.rs:13-18` and feeds the downstream lax-decode fallback identically. SPB (BC-2.01.013) correctly bounds by `min(block body, idb[0].snaplen)`.

One spec edge the BCs implicitly handle well: the rare malformed case `original_length < captured_length` (spec-violating). wirerust slices by `captured_length` regardless, so it cannot over-read; the only effect is the truncation flag being miscomputed, which is benign for wirerust's purposes.

*Sources: IETF draft pcapng §EPB length fields; Wireshark dev mailing-list (Captured vs Original = Capture Length vs Frame Length); libpcap incl_len/orig_len semantics. Perplexity deep-research, 2026-06-19.*

### F-11 — Multi-IDB fail-closed policy: right call for THIS corpus; relax later — CONFIRMED-OK with caveat (MEDIUM)

**Question #7 (multi-linktype realism).** wirerust's policy (BC-2.01.018, ADR-009 Decision 3) is fail-closed: all IDBs in a section must agree on `linktype`, else E-INP-011.

**Real-world frequency judgment:**
- **Multi-IDB files are common.** Any `tcpdump -i any` capture, any multi-NIC capture, and many merged captures contain multiple IDBs.
- **Multi-IDB with *differing* link-types is much less common, but real.** The classic case is `-i any` on Linux mixing `LINUX_SLL`/`LINUX_SLL2` with other types, and merges of heterogeneous captures. Single-physical-interface and same-type multi-interface captures (the common majority) all PASS the agreement check.
- **The intended unlock corpus is entirely single-IDB** (`arp-baseline-16pkt.cap`, `smb3.pcapng`, `dump.pcapng`, `tls12-dsb.pcapng` are all single-interface Wireshark/PacketLife captures). So fail-closed rejects ZERO files in this cycle's corpus.

**Conclusion:** Fail-closed is the **right call for this cycle** — it preserves the single-`DataLink` model with zero analyzer changes, and the only files it rejects (mixed-link-type captures) are out-of-scope and would need per-packet `DataLink` threading (explicitly deferred). The known-limitation is already documented in BC-2.01.018 and ADR-009 Consequences.

**Caveat / RECOMMENDATION:** The error message must be actionable. E-INP-011 already names the conflicting types and interface indexes — good. F3 should ensure the message hints at the cause for `-i any` users (e.g., "; multi-interface captures mixing link types are not supported — re-capture per-interface or split the file"). This converts a hard failure into a self-service fix and is the main UX risk of the policy. Also confirm directory-mode (`src/main.rs` glob) does not abort the entire run on one mixed-link-type file — per E-INP-005 it should be a per-file error, letting other files process.

*Sources: IETF draft pcapng §IDB / interface indexing; tcpdump `-i any` / LINUX_SLL behavior; BC-2.01.018; ADR-009. Perplexity deep-research + model knowledge of tcpdump `-i any`, 2026-06-19.*

---

## High-Risk Trap Scorecard (the four called-out items)

| # | Trap | Status | F3 action |
|---|------|--------|-----------|
| #1 | DSB-for-TLS (`tls12-dsb.pcapng`) | **CONFIRMED-OK — safe to skip; corpus reads correctly** | None required; add DSB to BC-2.01.015 example list (F-07) |
| #5 | Power-of-2 `if_tsresol` | **CONFIRMED-OK — already in BC-2.01.011 / BC-2.01.014** | Ensure base-2 unit/Kani test exists (no corpus file exercises it) |
| #3 | Multi-section files | **GAP (parser may not reset per-section) — BC text correct, delivery uncertain** | **Add AC: reject-or-first-section + verify pcap-file behavior (F-06)** |
| #7 | Multi-linktype realism | **CONFIRMED-OK — fail-closed correct for corpus; 0 files rejected** | Improve E-INP-011 message; confirm per-file (not per-run) failure (F-11) |

---

## Recommended F3 additions (BC/AC deltas)

These do NOT block the cycle. They close the flagged gaps explicitly so nothing is a silent mis-read.

1. **[from F-06, highest priority] Multi-section handling AC.** Add an explicit acceptance criterion to BC-2.01.010 (or a new BC-2.01.019): on encountering a second SHB, wirerust either (a) returns a clear error (preferred, tight scope) or (b) correctly resets per-section interface state — and this is *verified with a crafted 2-section fixture*, not assumed from `pcap-file`. Today the BCs assume correct multi-section behavior that the parser likely does not deliver.

2. **[from F-07] Enumerate skip arms in BC-2.01.015.** State that `Block::{NameResolution, InterfaceStatistics, SystemdJournalExport, Packet, Unknown}` are ALL silently skipped (and DSB/Custom on a future pcap-file 3.x). Prevents an omitted/`todo!()` match arm.

3. **[from F-08] Document OPB packet loss.** One line in ADR-009 Consequences: obsolete Packet Block (`0x00000002`) packets are skipped, not read (acceptable; obsolete; not in corpus).

4. **[from F-11] E-INP-011 actionable message + per-file failure.** Add a remediation hint for `-i any` mixed-link-type captures; confirm directory-mode treats it as a per-file error.

5. **[from F-05] Base-2 `if_tsresol` test obligation.** Because no corpus file uses base-2, BC-2.01.014's Kani proof over the full `u8` space (already specified) is the *only* guard — make sure it ships, since runtime data will never hit that branch.

---

## What was confirmed vs. inconclusive

- **CONFIRMED from spec (high confidence):** complete block enumeration + codes; DSB carries no packet data and does not affect EPB decoding; per-section endianness and per-section interface-index scoping; `if_tsresol` base-10/base-2 encoding + microsecond default; Captured/Original length = libpcap incl_len/orig_len; universal skip-by-block-total-length guarantee.
- **CONFIRMED from crate sources:** `pcap-file` 2.0.0 `Block` variants (no DSB/Custom variant → both route to `Unknown`); `IfTsResol(u8)` exposed as a raw, *uninterpreted* byte (so wirerust's pure-core conversion is correctly owned, not delegated).
- **INCONCLUSIVE / verify in F3:** whether `pcap-file` 2.0.0 actually resets interface state across multiple SHBs (API/source reading strongly suggests NOT, but no runtime test was run — F-06). This is the one item that should be empirically settled before F3 locks the multi-section AC.
- **NOT a gap (debunked concerns):** base-2 `if_tsresol` is already handled (was the suspected decimal-only trap); DSB is already safe (was the suspected TLS-corpus blocker).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 3 | (1) Full standard block-type enumeration + codes + DSB safety + skip-by-length guarantee; (2) endianness/multi-section/interface-index-scoping + if_tsresol base-2 + captured-vs-original-length semantics; (3) `pcap-file` 2.0.0 pcapng reader known limitations (multi-section, if_tsresol application, unknown-block handling) |
| Perplexity perplexity_ask | 1 | Real-world structure of `dump.pcapng` / `tls12-dsb.pcapng` and whether Wireshark/dumpcap pcapng is single-section (corpus frequency judgment) |
| WebFetch | 4 | docs.rs/pcap-file/2.0.0 — `pcapng` module surface, `Block` enum variants, `InterfaceDescriptionOption` (`IfTsResol(u8)`); GitHub `courvoif/pcap-file` reader source (per-section reset check) |
| Training data | 1 area | `tcpdump -i any` / LINUX_SLL mixed-link-type behavior for the F-11 frequency judgment (cross-checked against spec for IDB indexing) |

**Total MCP tool calls:** 4 (3 `perplexity_research` + 1 `perplexity_ask`) + 4 WebFetch
**Training data reliance:** low — every block code, DSB property, endianness/section rule, if_tsresol encoding, and length-field mapping was verified against the IETF draft / pcapng.com / crate sources via live tools; training data used only for the `-i any` real-world-frequency judgment in F-11, itself cross-checked against the spec's IDB semantics.

*Note on tool usage: the three `perplexity_research` (sonar-deep-research) outputs each exceeded the inline token cap and were returned as on-disk JSON; findings were extracted by reading the saved files. Full primary text retained at the harness tool-results path for audit.*
