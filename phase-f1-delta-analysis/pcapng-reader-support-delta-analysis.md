---
document_type: feature-delta-analysis
feature_id: FE-001-pcapng-reader-support
github_issue: null
title: "Add pcapng capture-format reader support"
intent: feature
feature_type: backend
trivial_scope: false
trivial_justification: >
  Requires format-detection logic in reader.rs (magic-byte probe, byte-order
  normalization, block-type dispatch for SHB/IDB/EPB/SPB), new timestamp-resolution
  path (per-interface resolution in IDB vs. file-level in classic pcap), link-type
  extraction from IDB instead of global header, retirement/inversion of BC-2.01.004
  (pcapng rejection becomes acceptance), new error-taxonomy entries, and a dependency
  decision on pcap-parser vs hand-rolled parser. Minimum five files with non-trivial
  logic changes.
scope_classification: standard
status: draft
producer: architect
created: 2026-06-19
base_commit: b73b242
branch: develop
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/module-criticality.md
---

# F1 Delta Analysis — FE-001: Add pcapng Capture-Format Reader Support

## 1. Feature Summary

wirerust's reader (`src/reader.rs`) currently supports only the classic libpcap
format (magic bytes `0xA1B2C3D4` or `0xD4C3B2A1`). Any pcapng file — regardless
of extension — is rejected with `Err("Failed to parse pcap header")` at the
`PcapReader::new` call site.

The goal of this feature is **transparent pcapng detection**: the reader probes the
file's first four bytes, routes to the appropriate parser (classic or pcapng), and
delivers the same `PcapSource { packets: Vec<RawPacket>, datalink: DataLink }` output
to the rest of the pipeline. The analyzer pipeline (`decoder.rs`, all stream analyzers,
reporters) must require zero changes.

### Concrete unblocks

| Blocker | Description |
|---------|-------------|
| `arp-baseline-16pkt.cap` | PacketLife capture stored as pcapng with `.cap` extension; currently rejected with "wrong magic number"; needed for ARP regression baseline. |
| TLS-heavy corpus | Public TLS captures (Wireshark `dump.pcapng`, `tls12-dsb.pcapng`) are pcapng-only; they are the richest TLS-handshake fixtures available. |
| Modern captures | Wireshark's default save format since 1.8 (2012) is pcapng; the majority of shared public captures are now pcapng. |

---

## 2. Intent and Scope Classification

| Field | Value |
|-------|-------|
| Intent | feature (new capability; format acceptance where previously format rejection) |
| Feature type | backend |
| Trivial | NO |
| Trivial justification | New format-detection + block-dispatch logic, per-interface timestamp resolution, IDB link-type extraction, BC retirement/replacement, new error taxonomy entries, dependency decision |
| Scope classification | standard — full F1–F7 cycle required |
| Recommended subsystem | SS-01 (PCAP File Ingestion, CAP-01) — reader changes are squarely within SS-01 scope |

---

## 3. Impact Boundary

### 3.1 Pure-Core / Effectful-Shell Boundary

`src/reader.rs` is classified as **effectful shell** (it performs file I/O).
The feature does not cross into the pure-core zone (`src/decoder.rs`,
`src/reassembly/`, analyzers) — those modules receive `RawPacket` slices
already normalized to the same type they receive today.

The magic-byte probe and block-parsing logic will be new effectful-shell code
inside `reader.rs`. If a pure-core timestamp-conversion function is extracted
(e.g., to convert pcapng's 64-bit timestamp units using an IDB resolution
exponent), that function is pure and can be unit-tested and Kani-proven in
isolation.

### 3.2 Modified Files

| File | Risk | Change Description |
|------|------|--------------------|
| `src/reader.rs` | HIGH | Primary change site. Add magic-byte probe in `from_file` / `from_pcap_reader` that reads the first 4 bytes and branches to either the existing classic-pcap path (`PcapReader`) or a new pcapng parse path. The pcapng path must: parse the Section Header Block (SHB) to determine byte-order; walk blocks to find all Interface Description Blocks (IDB) and extract `link_type` + `if_tsresol` option; collect packets from Enhanced Packet Blocks (EPB) and Simple Packet Blocks (SPB); normalize timestamps to the `(ts_sec: u32, ts_usecs: u32)` form that `RawPacket` already carries. |
| `src/main.rs` | LOW | The directory-glob pattern currently excludes `*.pcapng` (LESSON-P0.02 / #69, NFR-VIO-002). Once pcapng is supported, the glob must include `*.pcapng` and `*.pcap` (and optionally `*.cap`). This is a one-line regex/glob change. |
| `src/cli.rs` | NONE | No new flags. pcapng support is transparent; no operator-visible knob is needed. |

### 3.3 New Files

| Component | Rationale |
|-----------|-----------|
| `src/pcapng_reader.rs` (or inline in `reader.rs`) | Pcapng block-walk implementation. Whether this is a new module or an inline private block inside `reader.rs` is an F2 architecture decision; either satisfies the purity boundary — the public API surface (`PcapSource::from_file`) does not change. |

### 3.4 Unchanged / Dependent (must stay green)

| Component | Dependency |
|-----------|-----------|
| `src/decoder.rs` | Receives `(&[u8], DataLink)` — contract unchanged. |
| `src/dispatcher.rs` | Unaffected. |
| `src/reassembly/` | Unaffected. |
| `src/analyzer/` (all) | Unaffected. |
| `src/reporter/` | Unaffected. |
| `tests/` (all existing) | Must remain green; see §6. |
| `tests/fixtures/smb3.pcapng` | Currently used as a *rejection* fixture by `test_BC_2_01_004_rejects_pcapng`. After this feature, that test becomes a *successful parse* assertion. The fixture remains but the test semantics invert. |

---

## 4. Affected Behavioral Contracts

### 4.1 BCs That Change (modification required)

| BC ID | Current Postcondition | Change Required |
|-------|-----------------------|-----------------|
| **BC-2.01.004** "Reject pcapng-Format Input at Reader Level" | PC1: returns `Err("Failed to parse pcap header")`; PC2: no packets read | **RETIRE / INVERT.** This BC's normative content becomes false after the feature lands. It must be retired (lifecycle_status: retired, deprecated_by: BC-2.01.009 or equivalent) and replaced by a new "Accept pcapng Format" BC (see §4.2). The existing negative-coverage test (`test_BC_2_01_004_rejects_pcapng`) must be rewritten as a positive acceptance test. |
| **BC-2.01.002** "Read All Packets from PCAP as Vec<RawPacket> Preserving Timestamps" | Currently implicitly covers only classic pcap | **EXTEND.** Add postconditions covering pcapng: packets from EPB/SPB blocks are delivered in block-order; timestamps are normalized to `(ts_sec, ts_usecs)` using the IDB `if_tsresol` option (default 10^-6 = microseconds if option absent). |
| **BC-2.01.001** "Accept Supported Link Types and Reject Unsupported at File Open" | Link type from classic pcap global header | **EXTEND.** Link type now sourced from pcapng IDB `linktype` field. Multi-interface pcapng files (multiple IDBs with different link types) must be addressed: either (a) all IDBs must agree on link type (simplest constraint, reject mixed-linktype files), or (b) link type is resolved per-packet from the IDB index. This is a significant design decision — flag for §7 (ADR impact). |
| **E-INP-002 error taxonomy entry** | Message: "Failed to parse pcap header: <underlying>"; triggered by wrong magic / truncated file OR pcapng format | **REVISE.** After this feature, E-INP-002 is no longer triggered by pcapng magic. The error taxonomy note "or pcapng format" must be removed. A new INP error must cover pcapng-specific parse failures (e.g., truncated SHB, missing IDB before EPB, malformed block length). |
| **NFR-COMPAT-001** "Only classic pcap accepted" | "5 link types accepted; all others rejected" | **REVISE.** Both classic pcap and pcapng are now accepted. The test `test_pcapng_rejected` must be rewritten. |

### 4.2 New BCs Required (estimated count: 7–10)

These are net-new behavioral contracts in SS-01. They cover the pcapng block
grammar that has no analog in classic pcap.

| Proposed BC | Title | Priority |
|-------------|-------|----------|
| BC-2.01.009 | Accept pcapng Format: Transparent Detection via Magic-Byte Probe | P0 |
| BC-2.01.010 | Parse pcapng Section Header Block (SHB): Byte-Order Detection and Version | P0 |
| BC-2.01.011 | Parse pcapng Interface Description Block (IDB): Link Type and Timestamp Resolution | P0 |
| BC-2.01.012 | Parse pcapng Enhanced Packet Block (EPB): Packet Data and Timestamp | P0 |
| BC-2.01.013 | Parse pcapng Simple Packet Block (SPB): Packet Data (No Timestamp) | P1 |
| BC-2.01.014 | Per-Interface Timestamp Resolution: Convert 64-bit pcapng Timestamp to (sec, usecs) | P0 |
| BC-2.01.015 | Unknown/Unsupported pcapng Block Types Are Silently Skipped | P1 |
| BC-2.01.016 | Reject pcapng with Unsupported Link Type (Mirrors BC-2.01.001) | P0 |
| BC-2.01.017 | Surface pcapng Parse Errors with Anyhow Context (Block-Level) | P1 |
| BC-2.01.018 | Mixed-Interface pcapng: Accept or Reject Multiple IDBs (TBD — see §7) | P0 |

**Estimated new BCs: 9–10** (BC-2.01.009 through BC-2.01.018).

The total SS-01 BC count grows from 8 to approximately 17–18.

### 4.3 New Error Taxonomy Entries Required

| Proposed Code | Category | Severity | Description |
|---------------|----------|----------|-------------|
| E-INP-008 | INP | `broken` | pcapng SHB parse failure: truncated file or malformed Section Header Block |
| E-INP-009 | INP | `broken` | pcapng IDB missing before first EPB: file structure violation |
| E-INP-010 | INP | `broken` | pcapng EPB/SPB parse failure: block length inconsistency or truncated packet data |
| E-INP-011 | INP | `broken` | pcapng multi-interface link-type conflict (if mixed-IDB policy is "reject") |

---

## 5. Affected Stories and Tests

### 5.1 Existing Tests at Regression Risk

| Test / File | Risk | Impact |
|-------------|------|--------|
| `test_BC_2_01_004_rejects_pcapng` | HIGH | Test semantics invert completely. Currently asserts `Err("Failed to parse pcap header")` for `smb3.pcapng`; after this feature it must assert successful parse and correct packet count. The test must be rewritten, not just renamed. |
| `test_pcapng_rejected` (NFR-COMPAT-001) | HIGH | Same inversion. |
| All classic-pcap reader tests | MEDIUM | `from_pcap_reader` and `from_file` gain a branch at the top; existing tests must still route to the classic path and pass unchanged. The probe must be non-destructive (peeking, not consuming, the first 4 bytes before handing the reader to `PcapReader`). |
| `tests/` E2E tests using `*.pcap` fixtures | LOW | These call `PcapSource::from_file` with classic pcap files; the magic-byte probe must correctly route to the existing `PcapReader` path without regression. |
| Directory-glob resolution tests (if any) | MEDIUM | The `*.pcapng` exclusion from the glob (LESSON-P0.02 / NFR-VIO-002) must be revised to inclusion; any test asserting glob exclusion of `.pcapng` files must be updated. |

### 5.2 New Stories Required (estimated: 4–5)

| Story | Scope | Est. Points |
|-------|-------|-------------|
| STORY-123 | pcapng magic probe + SHB + byte-order detection (BC-2.01.009, BC-2.01.010); retire BC-2.01.004; rewrite negative test as acceptance test | 5 |
| STORY-124 | IDB parsing: link-type extraction, `if_tsresol` option, multi-IDB policy (BC-2.01.011, BC-2.01.016, BC-2.01.018) | 8 |
| STORY-125 | EPB parsing and timestamp normalization (BC-2.01.012, BC-2.01.014) | 8 |
| STORY-126 | SPB parsing + unknown-block skip (BC-2.01.013, BC-2.01.015) + pcapng-specific error surfaces (BC-2.01.017, E-INP-008..011) | 5 |
| STORY-127 | Glob update (include `*.pcapng`), E2E corpus unlock (`arp-baseline-16pkt.cap`, TLS pcapng fixtures), regression suite | 3 |

**Estimated new stories: 5**, wave-ordered after STORY-122 (currently the last scheduled story at wave 50).

### 5.3 Classic-pcap Regression Suite (must stay green throughout)

These tests form the non-negotiable green baseline. Any story implementing pcapng
support must demonstrate this suite stays green before merge.

- All `tests/reader_tests.rs` classic-pcap tests
- `test_BC_2_01_004_rejects_pcapng` (as rewritten positive test)
- `test_pcapng_rejected` (as rewritten acceptance test)
- All `tests/integration_tests.rs` tests using `tests/fixtures/*.pcap` files
- All `tests/reassembly_*`, `tests/http_analyzer_tests.rs`, `tests/tls_analyzer_tests.rs`,
  `tests/modbus_analyzer_tests.rs`, `tests/dnp3_analyzer_tests.rs`, `tests/arp_analyzer_tests.rs`
  — these pass through classic pcap fixtures; must remain unchanged
- `tests/dispatcher_tests.rs` — classic pcap capture path; unaffected but must be verified

---

## 6. ADR Impact

### 6.1 Existing ADRs

No existing ADR directly governs the pcap reader format selection. ADR-0005
(Binary ICS Protocol Integration) covers Modbus/DNP3 stream dispatch but not
file format detection. None of ADR-0001 through ADR-0007 are invalidated by
this feature.

### 6.2 New ADR Required

**Recommendation: create ADR-0008 "pcapng Format Detection and Block-Walk Strategy".**

The ADR must address three decisions that have architecture-wide traceability
consequences:

**Decision 1 — Dependency strategy (see §7 for full dependency analysis).**
Whether to use an existing crate (`pcap-parser`, `pcap-file` 3.x if it adds
pcapng support) or hand-roll a minimal pcapng block walker. This is the highest-
stakes decision in the feature and is flagged as a **research-agent question**
(see §7).

**Decision 2 — Multi-interface policy.**
A pcapng file may contain multiple IDB blocks with different `link_type` values.
The current pipeline assumes a single `DataLink` for the entire capture
(`PcapSource.datalink`). Options:
- (A) **Reject** any pcapng file with more than one IDB (simplest; blocks real
  captures with multiple interfaces).
- (B) **Require all IDBs agree on link type** (practical constraint; most
  single-NIC captures comply).
- (C) **Per-packet link type** — store `link_type` on each `RawPacket` and thread
  it through `decode_packet`. This changes the `RawPacket` struct, which is a
  public library type; a breaking change to the public API surface.

Option (A) or (B) preserves the current single-`DataLink` model with zero
downstream changes. Option (C) is architecturally cleaner but touches
`decoder.rs`, `RawPacket`, and all call sites — a significantly larger scope.

**Recommendation for F2:** select Option (B) initially (require IDB link-type
agreement; reject with `Err` on conflict). This keeps scope tight and can be
relaxed in a future cycle.

**Decision 3 — Byte-order handling.**
pcapng SHB carries a Byte-Order Magic (BOM) field (`0x1A2B3C4D` = big-endian;
`0x4D3C2B1A` = little-endian). The reader must branch on this; all subsequent
block field reads must use the detected endianness. This is a pure-core
decision (no I/O dependency) suitable for Kani-assisted property verification
(e.g., prove that byte-swapping a big-endian SHB and a little-endian SHB with
the same logical content produces identical `PcapSource` output).

---

## 7. Dependency Decision — Research-Agent Question

**THIS DECISION MUST NOT BE MADE UNILATERALLY IN F1. It requires a research-agent
evaluation before the F2 spec-evolution phase begins.**

### Context

The current reader depends on `pcap-file = "2.0.0"` (classic pcap only). Two
strategies exist for adding pcapng support:

#### Option A: `pcap-file` 3.x (if pcapng support was added)

The `pcap-file` crate's 3.x series may have added pcapng support. wirerust's
existing `pcap-file = "2"` pin uses the `2.0.0` crate (Cargo.lock confirmed).
A major-version bump to 3.x would need supply-chain vetting (new transitive
deps, API compatibility with the existing `DataLink`, `TsResolution`, and
`next_raw_packet()` usage in `reader.rs`). The `next_raw_packet()` unvalidated
path is relied upon for snaplen-truncated captures (reader.rs module doc,
LESSON-P0.02); this path must survive any crate version bump.

#### Option B: `pcap-parser` crate (nom-based)

The `pcap-parser` crate (`0.16.x` as of 2026) supports both classic pcap and
pcapng natively. It is nom-based, which introduces `nom` as a transitive dep.
wirerust has a minimal-dep posture (NFR-SUP: supply chain hygiene). The
research agent must assess:
- Whether `pcap-parser` introduces nom or other heavyweight transitive deps
  that conflict with wirerust's supply-chain NFR.
- Whether `pcap-parser`'s pcapng API surface covers SHB, IDB, EPB, SPB, and
  the `if_tsresol` option.
- License compatibility (MIT/Apache-2.0 required).
- Maintenance activity (last commit, open issues, CVE history).

#### Option C: Hand-rolled minimal pcapng block walker

The pcapng block grammar (RFC 2988 / pcapng spec) is well-specified and the
relevant blocks (SHB, IDB, EPB, SPB) are simple binary structures. A hand-rolled
implementation would be ~200–400 lines of Rust, fully controlled, zero new
transitive deps, and amenable to Kani proofs (pure-core timestamp conversion,
block-length bounds check). The tradeoff is implementation risk and maintenance
surface.

Given wirerust's existing pattern (hand-rolling the SLL header parse in
`decoder.rs:lax_parse` rather than relying on etherparse's missing
`LaxSlicedPacket::from_linux_sll`) and its supply-chain posture, Option C
may be the most consistent choice — but this must be confirmed with actual
crate research.

**Research-agent tasks for F2 gate:**
1. Fetch current `pcap-file` changelog/readme to confirm whether 3.x added
   pcapng and whether the classic-pcap `next_raw_packet()` path is preserved.
2. Fetch `pcap-parser` 0.16.x docs and `Cargo.toml` dependency tree.
3. Compare transitive dep counts: Option A, Option B, Option C.
4. Check for open CVEs or known security issues in either crate.
5. Verify `if_tsresol` option coverage in whichever crate option is pursued.
6. Produce a written recommendation in the research document before F2 spec
   begins.

---

## 8. Regression Risk Assessment

### Risk Level: MEDIUM overall

The analyzer pipeline, reassembler, dispatcher, and all downstream components
are entirely unaffected — they consume `Vec<RawPacket>` and `DataLink`, which
remain unchanged. The blast radius is confined to `src/reader.rs` and
`src/main.rs` (glob pattern).

| Risk | Severity | Mitigation |
|------|----------|-----------|
| Classic-pcap path disruption | HIGH | The magic-byte probe must peek (not consume) the first 4 bytes before handing the reader to `PcapReader`. Using `std::io::BufReader::fill_buf()` + `consume()` or re-wrapping the file as a `Cursor` after peeking avoids consuming the stream. This must be validated by running the full existing test suite against an implementation that only adds the probe (no pcapng path yet). |
| Snapshot-truncated classic pcap still works | MEDIUM | The `next_raw_packet()` unvalidated path in the existing reader (reader.rs:69) handles snaplen captures. If the crate or path changes, this guarantee is lost. Any dependency change must verify this property. |
| Timestamp normalization regression | MEDIUM | `RawPacket.timestamp_usecs` is currently normalized from pcap's `ts_frac` field (either µs or ns depending on magic). pcapng uses 64-bit timestamps with a per-interface resolution option (`if_tsresol`). A conversion bug here would silently corrupt timestamps in `Finding.timestamp` (BC-2.09.007) and in the capture-relative timestamp threading (BC-2.04.055). Property-test the conversion function with known edge cases (resolution = 9 → nanoseconds, resolution = 0 or absent → default µs). |
| Glob includes bad files | LOW | Adding `*.pcapng` to the glob means malformed pcapng files that previously were silently excluded now produce `Err` at the reader level. This is handled by the existing `with_context` / `?` propagation in `from_file`. |
| BC-2.01.004 test inversion | HIGH | The `smb3.pcapng` fixture and its associated test must change in lockstep. Any CI run where one but not the other is updated will fail. A single story that retires BC-2.01.004, rewrites the test, and adds BC-2.01.009 prevents split-commit failures. |

---

## 9. Scope Recommendation

**Confirm: single F1–F7 feature cycle.**

The feature scope is well-bounded:
- One primary change site (`src/reader.rs`)
- One minor change site (`src/main.rs` glob)
- Zero analyzer or dispatcher changes
- 5 new stories (estimated)
- 9–10 new BCs (all in SS-01)

The dependency decision (§7) is the primary gate before F2. If Option C
(hand-rolled) is selected, the implementation scope is slightly larger but
more verifiable. If Option A or B is selected, the scope may shrink (the crate
does the block walking). Either way, the feature does not require splitting into
multiple cycles.

**Single-cycle confidence: HIGH.**

---

## 10. Open Questions for F2 Spec-Evolution

| ID | Question | Owner |
|----|----------|-------|
| Q-001 | Which crate / approach for pcapng parsing? (see §7) | research-agent before F2 |
| Q-002 | Multi-IDB policy: reject on conflict (Option B) or per-packet link type (Option C)? | architect at F2 spec |
| Q-003 | Should `*.cap` extension be added to the glob? (Many pcapng files distributed with `.cap`) | product-owner at F2 |
| Q-004 | Should the SPB (Simple Packet Block) be supported, or deferred? SPBs carry no timestamp and no interface ID; they are legal but uncommon. | architect at F2 spec |
| Q-005 | VP scope: is a Kani proof of the pcapng timestamp-conversion function feasible (pure-core)? | formal-verifier at F6 |

---

## Appendix: pcapng Block-Format Reference

For F2 spec-evolution, the blocks relevant to wirerust's use case are:

| Block Type | Block Code | Mandatory? | Wirerust Relevance |
|------------|-----------|------------|-------------------|
| Section Header Block (SHB) | 0x0A0D0D0A | Yes (first block) | Byte-order detection via BOM; contains file version |
| Interface Description Block (IDB) | 0x00000001 | Yes (before any EPB) | `link_type` field; `if_tsresol` option for timestamp resolution |
| Enhanced Packet Block (EPB) | 0x00000006 | Common | Primary packet container; holds 64-bit timestamp + packet data |
| Simple Packet Block (SPB) | 0x00000003 | Rare | No timestamp; uses SHB-level or IDB snaplen |
| Interface Statistics Block (ISB) | 0x00000005 | Optional | Capture statistics; can be silently skipped |
| Obsolete Packet Block (OPB) | 0x00000002 | Legacy; Wireshark emits EPB | If encountered, handle as EPB or skip |

Unknown block types must be silently skipped per the pcapng specification
(all block types have a block-total-length field enabling safe skip).
