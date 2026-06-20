# pcapng Capture-Format Reading: Dependency Decision (A / B / C)

**Project:** wirerust 0.9.2 (Rust 2024, single-crate, strict minimal-dependency / SHA-pinned / supply-chain-conscious posture)
**Research date:** 2026-06-19
**Author:** vsdd-factory research-agent
**Status:** complete (one finding partially inconclusive — flagged inline)

---

## TL;DR / Recommendation

**Recommended: Option A — keep `pcap-file`, use its existing `pcapng` module; bump to 3.x only if/when 3.0.0 ships stable.**

The decisive facts:

1. **wirerust already depends on `pcap-file` 2.0.0**, and `pcap-file` 2.0.0 **already ships a full `pcapng` reader** (`PcapNgReader`, `PcapNgParser`, typed `Block` enum, `InterfaceDescriptionBlock { linktype, snaplen, options }`, `IfTsResol`/`IfTsOffset` options, `EnhancedPacketBlock`, `SimplePacketBlock`). **Option A adds zero new crates** because the capability is already on disk in the version already locked. Verified against docs.rs/pcap-file/2.0.0.
2. **`pcap-file`'s stable line is still 2.0.0.** 3.0.0 is only at `3.0.0-rc.2` (May 2026). A minimal-dependency, supply-chain-conscious project should not adopt a release candidate as a hard dependency. So "bump to latest 3.x" (literal Option A) is premature; the better reading of Option A is **"use the pcapng support already present in the pinned 2.0.0."**
3. **Option B (`pcap-parser`) adds ~4 new crates and duplicates two major versions** already in wirerust's tree (`nom` 7+8, `rusticata-macros` 4+5). That directly contradicts the minimal-dependency posture for **no capability gain** over what `pcap-file` 2.0.0 already provides.
4. **Option C (hand-roll)** is spec-grounded and viable (the pcapng spec is a stable IETF draft) and is wirerust's house style — but it is strictly *more* work and *more* attack surface than Option A, which already has the parser. Hand-rolling is only justified if `pcap-file`'s pcapng reader proves inadequate in practice (it does not, per the API audit below).

**Net: Option A = +0 crates and already-vendored. Adopt it.**

---

## Current state of wirerust (verified from the repo)

| Fact | Source |
|------|--------|
| Direct dep declared `pcap-file = "2"` | `Cargo.toml:29` |
| Locked at `pcap-file 2.0.0`, deps = `byteorder_slice`, `derive-into-owned`, `thiserror` | `Cargo.lock:817-826` |
| Reader currently uses **legacy pcap only**: `PcapReader::new` + `next_raw_packet()` | `src/reader.rs:28,46,69` |
| Reader already deliberately uses `next_raw_packet()` (not `next_packet()`) to honor snaplen-truncated records — documented bug-avoidance of `pcap-file` 2.0.0's validated path | `src/reader.rs:13-18` |
| **wirerust already pulls in `nom` 7.1.3** (via `tls-parser` 0.12.2 → `nom`, `nom-derive` 0.10.1, `rusticata-macros` 4.1.0) | `Cargo.lock:715, 725, 1116, 1309-1314` |
| `memchr` 2.8.0 already in tree | `Cargo.lock:703` |

This last row is the hinge of the whole decision: wirerust is **already in the `nom`/`rusticata` ecosystem at major version 7/4**. `pcap-parser` 0.17 lives at major version **8/5**, so it cannot share — it duplicates.

---

## Verified version facts (crates.io / docs.rs, 2026-06-19)

| Crate | Latest STABLE | Latest published (incl. pre-release) | Notes | Source |
|-------|---------------|--------------------------------------|-------|--------|
| `pcap-file` | **2.0.0** (2023-02-01) | `3.0.0-rc.2` (2026-05-06) | 3.x is RC only; `3.0.0-rc1` was 2024-01, `rc.2` 2026-05. 10M+ downloads on 2.0.0. | crates.io API `/crates/pcap-file` |
| `pcap-parser` | **0.17.0** (2025-07-25) | 0.17.0 | 0.16.0 = 2024-08, 0.15.0 = 2024-02. Requires Rust ≥1.65. | crates.io API `/crates/pcap-parser` |

---

## Capability matrix — does each read the required pcapng blocks?

Required: SHB, IDB, EPB, SPB; per-interface link-type + snaplen + 64-bit timestamp resolution (`if_tsresol`).

| Capability | `pcap-file` 2.0.0 (Option A) | `pcap-parser` 0.17.0 (Option B) | Hand-roll (Option C) |
|---|---|---|---|
| Reads pcapng (not just legacy pcap) | **Yes** — `pcapng` module: `PcapNgReader<R>`, `PcapNgParser`, `PcapNgWriter<W>` | **Yes** — `PcapNGReader`, `create_reader`, whole-file + streaming parsers | Yes, if implemented to spec |
| Section Header Block (SHB) | Yes (typed `Block` variant) | Yes | Must implement (block type `0x0A0D0D0A`) |
| Interface Description Block (IDB) | Yes — `InterfaceDescriptionBlock { linktype: DataLink, snaplen: u32, options: Vec<InterfaceDescriptionOption> }` | Yes (exposed; caller stores per-section IDBs) | Must implement (block type `0x00000001`) |
| Enhanced Packet Block (EPB) | Yes — `EnhancedPacketBlock`; `PcapNgParser::packet_interface()` maps EPB→IDB | Yes | Must implement (block type `0x00000006`) |
| Simple Packet Block (SPB) | Yes — `SimplePacketBlock` variant | Yes | Must implement (block type `0x00000003`) |
| Per-interface link-type | **Yes** — `idb.linktype: DataLink` (same `DataLink` enum wirerust's decoder already uses) | Yes (IDB field) | Must implement |
| Per-interface snaplen | **Yes** — `idb.snaplen: u32` | Yes (IDB field) | Must implement |
| 64-bit timestamp + `if_tsresol` | **Yes** — `InterfaceDescriptionOption::IfTsResol(u8)` and `IfTsOffset(u64)` are explicit variants (18 total) | Surfaced via IDB options layer (no dedicated typed accessor confirmed) | Must implement TLV option walk |
| Snaplen-truncated streaming next-packet contract | **Yes** — `PcapNgParser::next_block()` returns `(remainder, Block)` and signals `PcapError::IncompleteBuffer`; block-length framing tolerates captured_len < orig_len | **Yes** — block-wise streaming, `nom` `Incomplete` signaling; well-suited to incremental reads | Yes by construction (you control framing) |

**Sources for capability rows:** docs.rs/pcap-file/2.0.0 (`pcapng` module; `InterfaceDescriptionBlock`; `InterfaceDescriptionOption` enum — `IfTsResol`/`IfTsOffset` confirmed); docs.rs/pcap-parser/0.17.0 + crates.io listing; pcapng IETF draft for block types.

> **Note on the `DataLink` continuity win for Option A:** wirerust's decoder is built around `pcap_file::DataLink` (`src/decoder.rs:68`, and ~12 test files import `pcap_file::DataLink`). `pcap-file`'s `InterfaceDescriptionBlock.linktype` is that *same* `DataLink` type. Option A feeds the existing decoder with **zero type-translation glue**. Option B/C would require mapping a foreign link-type representation into wirerust's `DataLink`, adding code and a translation table to maintain.

---

## Dependency footprint deltas (the decisive axis for wirerust)

Baseline reminder: wirerust already has `nom` 7.1.3, `rusticata-macros` 4.1.0, `memchr` 2.8.0, `byteorder_slice`, `derive-into-owned`, `thiserror` in its lockfile.

### Option A — `pcap-file` (stay on 2.0.0)
- **New crates: 0.** `pcap-file` 2.0.0 and all three of its deps (`byteorder_slice`, `derive-into-owned`, `thiserror`) are **already locked**. The pcapng reader is already compiled in; only wirerust's own `src/reader.rs` needs to add a pcapng code path.
- (If/when 3.0.0 ships stable and you bump: dep set is reportedly still small/structural per the 3.x RC docs, but **do not adopt the RC** under this project's posture. Inconclusive: exact 3.0.0 dep list not pinned here because 3.0.0 stable does not yet exist.)

### Option B — adopt `pcap-parser` 0.17.0
`pcap-parser` 0.17.0 runtime deps (crates.io API): `nom ^8`, `rusticata-macros ^5`, `circular ^0.3`, `cookie-factory ^0.3` *(optional — writer only, not pulled for read-only use)*.

| New crate pulled in | Already in wirerust tree? | Net |
|---|---|---|
| `pcap-parser` 0.17.0 | no | **+1** |
| `nom` 8.x | **no — wirerust has nom 7.1.3**; majors don't unify | **+1 (duplicate `nom` major: 7 *and* 8 compiled)** |
| `rusticata-macros` 5.x | **no — wirerust has 4.1.0**; majors don't unify | **+1 (duplicate major: 4 *and* 5 compiled)** |
| `circular` 0.3 (zero deps) | no | **+1** |
| `memchr` (nom 8 dep) | yes (2.8.0) | +0 |
| `cookie-factory` | n/a (optional, read-only avoids) | +0 |

- **Net new crates: ~4**, and — worse for a supply-chain-conscious project — it introduces **two duplicated major versions** (`nom` 7+8, `rusticata-macros` 4+5) into the build graph, increasing compile time, binary size, and audit surface for **no capability advantage** over `pcap-file` 2.0.0.
- This would only break even if wirerust *also* upgraded `tls-parser` to a nom-8-based release so the whole tree converges on nom 8 — a larger, unrelated migration that touches the TLS decoder. Out of scope for "add pcapng reading."

### Option C — hand-roll a pcapng block walker
- **New crates: 0.** Consistent with wirerust's "hand-roll small binary parsers" house style (it already hand-rolls e.g. DNP3 per ADR-0007).
- Cost: ~200–400 LOC of new *first-party* code to write, fuzz, and maintain (SHB/IDB/EPB/SPB framing, TLV option walk for `if_tsresol`/`if_tsoffset`, per-section IDB table, endianness from SHB byte-order magic, 32-bit block-length padding). This is net-*new* attack surface that **Option A already gives you for free, pre-tested, with 10M+ downloads of field exposure.**

**Crate-count summary:** A = **+0**, C = **+0** (but +400 LOC first-party), B = **+~4 (with 2 duplicate majors)**.

---

## Maintenance & security

| Crate | Last release | RUSTSEC | Source |
|---|---|---|---|
| `pcap-file` | 2.0.0 (2023-02), `3.0.0-rc.2` (2026-05) — actively curated, RC in progress | **No advisory** | rustsec.org / advisory-db |
| `pcap-parser` (rusticata) | 0.17.0 (2025-07) — actively maintained | **No advisory** | rustsec.org / advisory-db |
| `nom` (7 and 8), `circular`, `rusticata-macros`, `byteorder_slice`, `derive-into-owned`, `thiserror` | — | **No advisories** | rustsec.org / advisory-db |

All candidate crates are clean in RustSec as of 2026-06-19 (verified via `perplexity_ask` against rustsec.org). Caveat: RustSec only reflects *reported* issues; continue auditing with `cargo-audit`.

> **Maintenance nuance:** `pcap-file` stable (2.0.0) is ~3 years old. That is "stable/quiet," not "abandoned" — a 3.0.0 RC was published May 2026, showing the maintainer is active. For a parser of a stable, slow-moving binary format, low churn is acceptable and arguably desirable for a supply-chain-conscious consumer.

---

## pcapng spec authority (grounds Option C if ever chosen)

The format is defined by the IETF draft **"PCAP Next Generation (pcapng) Capture File Format"** (`draft-ietf-opsawg-pcapng`, the opsawg working-group document; canonical living copy at `https://github.com/IETF-OPSAWG-WG/pcapng` / `pcapng.com`). Block types needed: SHB `0x0A0D0D0A`, IDB `0x00000001`, SPB `0x00000003`, EPB `0x00000006`; `if_tsresol` is IDB option code 9, `if_tsoffset` code 14; all blocks are 32-bit-length-framed with trailing length redundancy and 4-byte option padding; section endianness is taken from the SHB byte-order magic `0x1A2B3C4D`. (Spec authority confirmed via Perplexity deep research + Wireshark developer docs.)

---

## Decision rationale, weighted for wirerust's minimal-dependency posture

1. **Minimal-dependency posture → strongly favors A (+0 crates, already vendored) over B (+4 crates, 2 duplicate majors).** This is dispositive: B's only theoretical edge would be capability, and it has none over `pcap-file` 2.0.0.
2. **Supply-chain-conscious / SHA-pinned posture → favors A and C over B.** Fewer new crates = smaller audit/pinning surface. A keeps an already-audited dependency; B widens the graph.
3. **"Already hand-rolls small parsers" → makes C *tolerable*, not *preferred*.** C duplicates work that A delivers for free and adds first-party fuzzing/maintenance burden. Hand-rolling is the right call when no good crate exists; here a good crate is already a dependency.
4. **`DataLink` type continuity** → A plugs into the existing decoder with zero glue; B/C require a link-type translation layer.
5. **Snaplen-truncation contract** → A satisfies it via `next_block()` + `IncompleteBuffer`, mirroring the legacy-path discipline already documented in `src/reader.rs`.

**Therefore: Option A.** Concretely — add a `PcapNgReader` branch in `src/reader.rs` alongside the existing `PcapReader` legacy path (format-detect on the file's magic), reusing the `DataLink` plumbing the decoder already speaks. Stay on `pcap-file` 2.0.0; revisit a 3.0.0 bump only after 3.0.0 ships **stable** (currently RC only — do not pin an RC).

**Fallback:** If, during implementation, `pcap-file` 2.0.0's pcapng reader exhibits a snaplen/truncation bug analogous to the legacy `next_packet()` issue already documented in `src/reader.rs` (plausible given that exact prior), the right escalation is **Option C (hand-roll, +0 crates)**, *not* Option B — because C preserves the minimal-dependency posture while B violates it.

---

## Inconclusive / caveats

- **`pcap-file` 3.0.0 stable dep list**: not pinnable — 3.0.0 stable does not exist yet (only `3.0.0-rc.2`). The recommendation deliberately does not depend on 3.x.
- **`pcap-file` 2.0.0 `IfTsResol` semantics in the reader**: the *option variant* `IfTsResol(u8)` is confirmed present in the IDB options enum, but I did not exercise a runtime test proving the reader correctly applies it to EPB 64-bit timestamps. wirerust's current legacy path converts timestamps itself (`src/reader.rs:18`); the pcapng path should similarly read `if_tsresol` from the IDB options and scale accordingly. Recommend a unit test on a known nanosecond-resolution pcapng fixture.
- **`pcap-parser` `if_tsresol` accessor**: only confirmed available through the generic IDB options layer; no dedicated typed accessor confirmed. Not decisive since B is not recommended.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source state-of-the-crates synthesis: pcapng block support, streaming, dep footprint, maintenance, spec authority (reasoning_effort=high) |
| Perplexity perplexity_ask | 1 | RUSTSEC advisory sweep across all 7 candidate/transitive crates |
| WebFetch | 8 | crates.io API (pcap-file, pcap-parser versions + dep lists; nom 8, circular, rusticata-macros deps); docs.rs/pcap-file/2.0.0 (pcapng module, InterfaceDescriptionBlock, InterfaceDescriptionOption) |
| Read | 3 | wirerust Cargo.toml, Cargo.lock (pcap-file entry), Perplexity result file |
| Grep | 3 | wirerust pcap-file usage, nom/memchr/rusticata presence in Cargo.lock |
| Training data | 1 area | pcapng block-type constants / option codes (cross-checked against Perplexity + spec authority) — flagged |

**Total MCP tool calls:** 2 Perplexity (1 research + 1 ask). WebFetch (8) used for authoritative crates.io/docs.rs version + dependency verification — registry numbers were NOT taken from training data.
**Training data reliance:** low — every version number and dependency count is registry/docs-verified; spec block constants cross-checked against Perplexity research + Wireshark/IETF authority.
