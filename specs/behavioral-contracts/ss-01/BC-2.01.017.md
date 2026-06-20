---
document_type: behavioral-contract
level: L3
version: "1.4"
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
  - "v1.4: Pass-3 remediation Burst Q3 (ADR-009 rev 6) — (H-3) E-INP-001 added to PC1 context-string list: 'pcapng Interface Description Block link type rejected' → E-INP-001 (whitelist Err raised at IDB-parse time paths through this cross-cutting contract). Error Taxonomy traceability field updated to include E-INP-001. Description updated to note taxonomy range includes E-INP-001. — 2026-06-19"
  - "v1.3: Pass-2 remediation Burst P2b (ADR-009 rev 5) — (C-4 CRITICAL) EC-002 error code corrected: EPB OOB on non-empty table → E-INP-010 (was E-INP-008). EC-005 minimum corrected: 'below minimum 8' → 'below minimum 12' (ADR Decision 8; crate rejects block_total_length < 12). (O-2) PC1 context strings extended: add E-INP-009 'before any Interface Description Block' context wording. Add E-INP-013 (interleaved-IDB) reference to edge-case map and Error Taxonomy field. (I-11) add Test: citations to ACs. — 2026-06-19"
  - "v1.2: ADR-009 rev 4 Burst B — Add VP-028 (cargo-fuzz fuzz_pcapng_reader) to Verification Properties, explicitly tagged as F6 hardening deliverable NOT F3. State that the no-panic-on-malformed-input contract is the cross-cutting parent of per-BC no-panic ACs. Add PC3 (no panic, no infinite loop). — 2026-06-19"
  - "v1.1: 2026-06-19 — added E-INP-012 to Error Taxonomy traceability field (cosmetic consistency; no normative behavior change)"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.017: pcapng Block-Level Parse Errors Surface via anyhow Context Chain

## Description

All pcapng parse failures (truncated SHB, truncated IDB, truncated EPB, truncated SPB,
invalid block-total-length, missing IDB before EPB, malformed block structure, unsupported
IDB link type) MUST surface as `Err(anyhow::Error)` via the existing `?` propagation chain
— the same mechanism used by the classic-pcap path. Each error MUST be wrapped with
`anyhow::Context` text that identifies the block type and, where applicable, the interface
index or block sequence number. The error ultimately maps to one of the taxonomy entries
(E-INP-001, E-INP-008 through E-INP-013). E-INP-001 applies when the IDB linktype whitelist
check fires at IDB-parse time (BC-2.01.016); E-INP-008 through E-INP-013 cover the remaining
pcapng structural error classes. No pcapng parse error produces a `panic!` or an `unwrap`
in production code.

**Cross-cutting no-panic parent:** This BC is the authoritative cross-cutting contract for
the no-panic property across the entire pcapng reader. The per-BC no-panic ACs (SEC-005)
in BC-2.01.010 AC-005, BC-2.01.011 AC-001, BC-2.01.016 AC-002, BC-2.01.018 (postconditions)
are per-block specializations of this contract. BC-2.01.017 PC3 (below) is the top-level
statement. VP-028 (cargo-fuzz, F6) is the primary vehicle for proving this contract
across the full input space.

## Preconditions

1. A pcapng parse error has been detected at any block level (SHB, IDB, EPB, SPB, or
   unknown-block skip).
2. The error is surfaced from within `PcapSource::from_pcap_reader` or a helper it calls.

## Postconditions

1. The function returns `Err(anyhow::Error)` whose error chain contains at minimum:
   - The root cause from `pcap-file` 2.0.0's parser (e.g., an I/O error or a parse error).
   - An anyhow context string identifying the block type, e.g.:
     - `"Failed to parse pcapng Section Header Block"` (→ E-INP-008)
     - `"Failed to parse pcapng Interface Description Block at interface index <N>"` (→ E-INP-008)
     - `"Failed to parse pcapng Enhanced Packet Block (packet <seq>)"` (→ E-INP-010)
     - `"Failed to read pcapng Simple Packet Block"` (→ E-INP-010)
     - `"Failed to skip pcapng block (type=0x{block_type:08X})"` (→ E-INP-010)
     - `"pcapng Enhanced Packet Block encountered before any Interface Description Block"` (→ E-INP-009)
     - `"pcapng Simple Packet Block encountered before any Interface Description Block"` (→ E-INP-009)
     - `"pcapng Interface Description Block link type rejected"` (→ E-INP-001; context string wraps the root `Err` from the BC-2.01.016 whitelist check; the full message rendered to the user is the E-INP-001 format: `"Unsupported pcap link type: <type>. Supported: ..."` propagated via the anyhow chain)
2. No partial `PcapSource` is returned on parse error; the entire operation fails.
3. **No panic, no infinite loop (cross-cutting no-panic contract):** For ANY byte sequence
   fed to `PcapSource::from_pcap_reader`, the function returns `Ok(_)` or `Err(_)` — it
   MUST NOT panic and MUST NOT loop indefinitely. This is the top-level statement of the
   no-panic guarantee across the full pcapng reader path. The block-walk loop MUST break on
   `Err(_)` from the crate (ADR-009 Decision 8). Per-BC AC (SEC-005) in BC-2.01.010/011/016
   are specializations of this postcondition. **VP-028** (cargo-fuzz `fuzz_pcapng_reader`,
   F6 hardening deliverable — NOT an F3 obligation) is the primary verification vehicle.
4. No `unwrap`, no `expect` in the pcapng code path (same invariant as the classic-pcap path).
5. The error is visible to the caller (e.g., `main.rs`) via the existing
   `with_context(|| format!("Failed to read {:?}", path))` wrapper (E-INP-005),
   which wraps pcapng errors identically to classic-pcap errors.

## Invariants

1. Error propagation style matches the existing codebase: `?` operator + `anyhow::Context`
   chaining. No new error type is introduced.
2. Every pcapng error path that can produce `Err` MUST have a context string; bare `?`
   without context is prohibited for pcapng paths.
3. The error taxonomy codes (E-INP-008..013) are categorization labels for this spec; the
   Rust implementation uses anyhow context strings, not numeric codes, at runtime.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Truncated SHB | `Err` chain: root I/O error + "Failed to parse pcapng Section Header Block" context → E-INP-008 |
| EC-002 | EPB references interface index 5 when only 2 IDBs exist | `Err` with context "Enhanced Packet Block references interface 5 but only 2 interfaces defined" → E-INP-010 (OOB on non-empty table; empty-table case is E-INP-009) |
| EC-003 | EPB packet data truncated mid-block | `Err` with EPB context + block sequence hint → E-INP-010 |
| EC-004 | Multi-IDB linktype conflict | `Err` with context identifying conflicting types → E-INP-011 |
| EC-005 | Unknown block with `block_total_length < 12` | `Err` with context "block_total_length=<N> is below minimum 12" → E-INP-010 (ADR-009 Decision 8: crate rejects block_total_length < 12, not < 8) |
| EC-006 | IDB block appears after first EPB (interleaved ordering) | `Err` → E-INP-013: "pcapng interface description block after first packet block — unsupported ordering"; block sequence numbers of the late IDB and first packet block included in context |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| File with SHB only (no IDB, no EPB) but truncated SHB | `Err` containing "Section Header Block" context | error |
| Well-formed pcapng with truncated 3rd EPB | `Err` containing "Enhanced Packet Block" context; packets 1 and 2 NOT returned | error |
| Valid pcapng (all blocks well-formed) | `Ok(PcapSource)` — no error surfaces | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-028 | pcapng reader no-panic: `PcapSource::from_pcap_reader` returns `Ok` or `Err` for any byte sequence; no panic, no infinite loop. **F6 hardening deliverable — NOT an F3 obligation.** The cargo-fuzz harness `fuzz_pcapng_reader` exercises the full block-walk path including edge cases not reached by unit tests. | cargo-fuzz (F6 Phase) |
| — | No panic on malformed pcapng (any truncation point) — covered by VP-028. **Test:** `test_BC_2_01_017_no_panic_truncated_pcapng` — truncate at every offset; assert no panic | unit (F3); VP-028 fuzz (F6) |
| — | Every error path includes a context string. **Test:** `test_BC_2_01_017_all_error_paths_have_context` — inject each error class; assert anyhow chain contains expected context substring | code review + unit |
| — | E-INP-005 wrapping applies to pcapng errors identically to classic-pcap. **Test:** `test_BC_2_01_017_einp005_wraps_pcapng_error` — assert chain has both "Failed to read {path}" and a pcapng block context | unit |
| — | E-INP-009 context string emitted when EPB/SPB encountered before any IDB. **Test:** `test_BC_2_01_017_epb_before_idb_emits_einp009_context` — file with EPB before any IDB; assert "before any Interface Description Block" in chain | unit |
| — | E-INP-013 surfaced when late IDB is interleaved after a packet block. **Test:** `test_BC_2_01_017_interleaved_idb_emits_einp013` — file with IDB appearing after first EPB; assert E-INP-013 context in chain | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- error surfacing is a quality property of the ingestion pipeline; consistent anyhow context chaining enables the CLI's error reporting (E-INP-005) to display useful diagnostics for pcapng failures, exactly as it does for classic-pcap failures |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-126 |
| ADR Reference | ADR-009 Consequences: "Adding *.pcapng to the src/main.rs directory glob means malformed pcapng files that were silently excluded now produce errors at the reader level" |
| Error Taxonomy | E-INP-001 (pcapng IDB linktype whitelist, raised by BC-2.01.016 at IDB-parse time; same code and message format as classic-pcap path), E-INP-008, E-INP-009, E-INP-010, E-INP-011, E-INP-012, E-INP-013 (see taxonomy) |

## Related BCs

- BC-2.01.010 -- related (SHB parse errors surface via this contract; E-INP-008, E-INP-012)
- BC-2.01.011 -- related (IDB parse errors surface via this contract; E-INP-008, E-INP-013)
- BC-2.01.012 -- related (EPB parse errors surface via this contract; E-INP-009, E-INP-010)
- BC-2.01.013 -- related (SPB parse errors surface via this contract; E-INP-009, E-INP-010)
- BC-2.01.015 -- related (unknown-block skip errors surface via this contract; E-INP-010)
- BC-2.01.018 -- related (multi-IDB conflict surfaces as E-INP-011 via this contract)
- BC-2.01.011 -- cross-ref: E-INP-013 (interleaved-IDB: late IDB encountered after first
  packet block; unsupported ordering; see error-taxonomy.md E-INP-013)

## Architecture Anchors

- `src/reader.rs` -- existing `?` + `.context(...)` pattern; pcapng errors follow same style
- `src/main.rs` -- E-INP-005: `with_context(|| format!("Failed to read {:?}", path))` wraps pcapng errors identically

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads stream; no writes |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (error propagation pattern; no new I/O) |
