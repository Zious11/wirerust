---
document_type: story
story_id: STORY-124
epic_id: E-19
version: "1.0"
status: draft
# BC status: BCs authored and anchored below; all traces complete.
producer: story-writer
timestamp: 2026-06-20T00:00:00Z
phase: f3
points: 8
priority: P0
depends_on: [STORY-123]
blocks: [STORY-125, STORY-126, STORY-127]
behavioral_contracts:
  - BC-2.01.011
  - BC-2.01.016
  - BC-2.01.018
verification_properties: [VP-030]
tdd_mode: strict
target_module: reader
subsystems: [SS-01]
estimated_days: 3
feature_id: f3-pcapng-reader-support
wave: 52
inputs:
  - .factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.011.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.016.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.018.md
input-hash: ""
# Dependency anchor: STORY-124 depends on STORY-123 because the IDB parser
#   requires the SHB parse and magic-byte probe infrastructure established by
#   STORY-123; the byte-order state and pcapng routing established by STORY-123
#   must be in place before IDB fixed fields can be decoded.
# Subsystem anchor: SS-01 owns this story's scope because BC-2.01.011,
#   BC-2.01.016, and BC-2.01.018 are all SS-01 behavioral contracts per their
#   traceability tables, anchored to src/reader.rs (C-4) per ARCH-INDEX
#   Subsystem Registry.
input-hash: "7ec43f8"
---

# STORY-124: IDB Parse (Link Type + if_tsresol), Interface Whitelist, and Multi-IDB Agreement

## Narrative

- **As a** security analyst running wirerust against pcapng captures from multi-interface captures
  and captures with uncommon link types
- **I want** the pcapng reader to parse Interface Description Blocks (IDBs) to extract link type
  and timestamp resolution, enforce the link-type whitelist at IDB-parse time, and reject
  multi-IDB captures where the link types disagree
- **So that** `PcapSource` always carries a whitelisted `DataLink`, the per-interface `if_tsresol`
  is correctly stored for EPB timestamp conversion, and captures mixing link types produce a clear
  error rather than silently corrupt packet parsing

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.01.011 | Parse pcapng Interface Description Block (IDB): Link Type and Timestamp Resolution |
| BC-2.01.016 | Reject pcapng with Unsupported Link Type in IDB (Mirrors BC-2.01.001) |
| BC-2.01.018 | Multi-IDB Link-Type Agreement Policy: Conflict Returns Error (Fail-Closed) |

## Acceptance Criteria

### AC-001 (traces to BC-2.01.011 postcondition 1 — linktype extracted and stored)
`linktype` is extracted from IDB body bytes 0–1 (after byte-order correction) and stored as
`DataLink` in a `Vec<InterfaceInfo>` at the 0-based interface index. The interface table MUST be
a `Vec<InterfaceInfo>`, NOT a `HashMap`. Interface indexes are assigned in IDB encounter order
within the section (BC-2.01.011 Invariant 1).

**Test:** `test_BC_2_01_011_linktype_ethernet`, `test_BC_2_01_011_interface_table_is_vec_indexed`

### AC-002 (traces to BC-2.01.011 postcondition 2 — if_tsresol extraction and default)
`if_tsresol` (option code 9) is extracted from the IDB options TLV walk if present; if absent, the
stored exponent is `6u8` (10^-6 microseconds, pcapng spec default). The `if_tsresol` value is
stored in `InterfaceInfo.if_tsresol`. `snaplen` (IDB body bytes 4–7) is READ to advance past fixed
fields and DISCARDED — it is NOT stored (ADR-009 Decision 9 / rev 9 F-M3). `if_tsoffset` (option
code 10) is NOT extracted this cycle (Decision 21) — it is silently skipped as an unknown option.

**Test:** `test_BC_2_01_011_if_tsresol_stored_in_interface_info`, `test_BC_2_01_011_if_tsresol_absent_default`

### AC-003 (traces to BC-2.01.011 postcondition 5 — IDB error routing uniform split)
IDB errors are routed according to the uniform 4-way split (ADR-009 Decision 20):
- (a) btl < 12 / btl % 4 != 0 / EOF → crate rejects → **E-INP-010**
- (b) 12 ≤ btl < 20 (body 0–7 bytes) → crate frames block; wirerust body-decode finds body < 8
  IDB fixed-field bytes → **E-INP-008**. Canonical fixture: btl=16 → body=4 bytes
- (c) Non-zero `reserved` field → **E-INP-008** (structural IDB error; mirrors crate enforcement
  at `interface_description.rs:48-49`)
- (d) Well-formed → parse proceeds

**Test:** `test_BC_2_01_011_body_truncated_e_inp_008`, `test_BC_2_01_011_nonzero_reserved_e_inp_008`

### AC-004 (traces to BC-2.01.011 postcondition 6 — IDB options TLV bounds-check)
The options TLV walk MUST bounds-check each `option-length` against remaining body bytes BEFORE
reading the value. If `option-length` exceeds remaining bytes → `Err` mapped to **E-INP-008** (no
panic, no OOB). `opt_endofopt` (code 0) or end-of-body terminates the walk. Unknown option codes
are silently skipped (length + padding consumed). `if_tsresol` option (code 9) MUST have
`option_length == 1`; if `option_length != 1` → **E-INP-008** (MUST NOT silently ignore or default
to 6; F-M5 / ADR-009 rev 9 — this applies ONLY to code 9, which wirerust specially handles).

**Test:** `test_BC_2_01_011_options_malformed_length_e_inp_008`,
`test_BC_2_01_011_if_tsresol_wrong_length_e_inp_008`

### AC-005 (traces to BC-2.01.011 AC-004 — late IDB rejection E-INP-013)
If an IDB is encountered AFTER the first packet block has been emitted (`packets_emitted > 0`),
wirerust MUST return `Err` mapping to **E-INP-013** ("pcapng interface description block after
first packet block — unsupported ordering"). The interface table MUST NOT be updated. Processing
stops immediately. This is the FIRST check in the three-level IDB-parse precedence (Decision 17);
the IDB body is NOT decoded.

**Test:** `test_BC_2_01_011_late_idb_after_packet_rejected_e_inp_013`

### AC-006 (traces to BC-2.01.011 AC-006 — IDB-parse three-level precedence, Decision 17)
The three-level precedence at IDB-parse time is applied in EXACT order with no reordering:
1. **E-INP-013 position check FIRST** — if `packets_emitted > 0` → return E-INP-013; IDB body
   NOT decoded; E-INP-001 and E-INP-011 NOT evaluated
2. **E-INP-001 whitelist check SECOND** — if `linktype` not in whitelist → return E-INP-001;
   E-INP-011 NOT evaluated
3. **E-INP-011 conflict check THIRD** — if `linktype` differs from first IDB's → return E-INP-011

A late IDB that also carries a non-whitelisted or conflicting linktype receives ONLY E-INP-013.

**Test:** `test_BC_2_01_011_idb_precedence_e_inp_013_wins_over_conflict`

### AC-007 (traces to BC-2.01.016 postcondition 2 — whitelist rejection at IDB-parse time)
When an IDB carries a `linktype` NOT in `{ETHERNET, RAW, IPV4, IPV6, LINUX_SLL}`, wirerust MUST
return `Err` with message
`"Unsupported pcap link type: {linktype:?}. Supported: Ethernet (1), Raw IP (101), Linux Cooked (113), IPv4 (228), IPv6 (229)"`.
The check fires immediately at IDB-parse time (check #2 per Decision 17). No packets from the
violating interface are ever returned. The whitelist is IDENTICAL to BC-2.01.001; any change to the
whitelist is a coordinated breaking change to both BCs simultaneously.

**Test:** `test_BC_2_01_016_whitelist_mirrors_bc_2_01_001`,
`test_BC_2_01_016_non_whitelisted_linktype_returns_err_no_panic`

### AC-008 (traces to BC-2.01.018 postcondition 2 — multi-IDB conflict returns E-INP-011)
When a second (or subsequent) IDB carries a `linktype` differing from the first IDB's `linktype`
(and both are whitelisted), wirerust MUST return `Err` with context:
`"pcapng multi-interface link-type conflict: interface 0 has {first:?}, interface {n} has {other:?}"`.
No packets are returned. The check is the THIRD check per Decision 17 (whitelist passes, then
conflict fires). The check runs lazily: each new IDB's linktype is compared to the first; first
mismatch triggers the error immediately; subsequent IDBs are not parsed.

**Test:** `test_BC_2_01_018_two_idbs_different_linktype_e_inp_011`,
`test_BC_2_01_018_three_idbs_third_conflicts`

### AC-009 (traces to BC-2.01.018 postcondition 1 — same-linktype multi-IDB succeeds)
When all IDBs in a section carry the same `linktype`, agreement is satisfied and parsing continues.
`PcapSource.datalink` is set to that agreed value. Single-IDB files (the common case) trivially
satisfy the agreement policy.

**Test:** `test_BC_2_01_018_two_idbs_same_linktype_ok`, `test_BC_2_01_011_two_idbs_same_linktype`

### AC-010 (traces to BC-2.01.011 AC-001 — no-panic, SEC-005)
The IDB parse path MUST return `Err` for any malformed or truncated IDB byte sequence.
`unwrap()`, `expect()`, `panic!()`, and `unreachable!()` are prohibited in the IDB parse path.

**Test:** `test_BC_2_01_011_no_panic_fuzz` (property test over arbitrary IDB bytes)

### AC-011 (traces to BC-2.01.018 — VP-030 domain is whitelisted DataLink values only)
VP-030 proptest covers multi-IDB agreement over WHITELISTED `DataLink` values ONLY
(`{ETHERNET, RAW, IPV4, IPV6, LINUX_SLL}`). Non-whitelisted values short-circuit to E-INP-001 at
whitelist check #2 (BC-2.01.016) and are OUT of VP-030 scope. Comparison unit is `DataLink` enum
(NOT raw u16). Property: any sequence of whitelisted DataLink values either all-equal → `Ok` with
`PcapSource.datalink` = that variant, or first-differing whitelisted value → `Err` with E-INP-011.

**Test:** VP-030 proptest (`generate arbitrary Vec<DataLink>` restricted to whitelist set)

## Behavioral Contracts Table

| BC | Version | Clauses Covered |
|----|---------|-----------------|
| BC-2.01.011 | v1.7 | PC1 (linktype extracted), PC2 (if_tsresol extracted + default), PC3 (interface table Vec), PC4 (snaplen read-and-discarded — F-M3), PC5 (uniform error-code split Decision 20), PC6 (options TLV walk; bounds-check; if_tsresol enforcement F-M5), AC-001 (no-panic SEC-005), AC-002 (Vec not HashMap), AC-003 (if_tsresol feeds BC-2.01.014), AC-004 (late IDB → E-INP-013), AC-005 (TLV bounds-check), AC-006 (three-level precedence Decision 17), Inv1 (0-based index), Inv3 (if_tsresol immutable), Inv4 (linktype DataLink type) |
| BC-2.01.016 | v1.4 | PC1 (whitelist), PC2 (rejection message format), PC3 (PcapSource.datalink always whitelisted), Inv1 (whitelist identical to BC-2.01.001), Inv2 (error message format identical), Inv3 (check ordering: position first, whitelist second, conflict third), AC-001 (mirror of BC-2.01.001), AC-002 (no-panic SEC-005) |
| BC-2.01.018 | v1.6 | AC-001 (conflict message format + E-INP-011 is THIRD check), PC1 (all-same → Ok), PC2 (conflict → Err E-INP-011 message format), PC3 (single-IDB trivially passes), PC4 (lazy check; first mismatch), Inv1 (fail-closed), Inv2 (known limitation documented), Inv3 (E-INP-011 is THIRD check per Decision 17) |

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| IDB body decode (linktype, reserved, snaplen-discard) | `src/reader.rs` (pcapng block-walk) | Effectful shell (I/O: block reading) |
| IDB options TLV walk (if_tsresol extraction) | `src/reader.rs` (pure-core helper) | Pure core (byte slice decode) |
| Interface table (`Vec<InterfaceInfo>`) | `src/reader.rs` (local parse state) | Pure state (no I/O) |
| Link-type whitelist check (BC-2.01.016) | `src/reader.rs` | Pure core (enum match) |
| Multi-IDB agreement check (BC-2.01.018) | `src/reader.rs` | Pure core (DataLink comparison) |
| Three-level precedence dispatch (Decision 17) | `src/reader.rs` | Pure core |

Architecture section references: `architecture/module-decomposition.md` (SS-01 C-4,
`src/reader.rs`); ADR-009 Decision 2 (IDB coverage), Decision 3 (multi-IDB policy), Decision 4
(if_tsresol extraction), Decision 17 (three-level precedence), Decision 20 (uniform error-code
rule), Decision 21 (if_tsoffset NOT extracted this cycle).

## Forbidden Dependencies

- `src/reader.rs` MUST NOT gain any new crate dependency. +0 new crates per ADR-009 Decision 1
  (Option A).
- The IDB parse path MUST NOT use `pcap_file::pcapng::blocks::InterfaceDescriptionBlock` from the
  high-level API (that API does not apply `if_tsresol` correctly on the raw path and does not
  expose the raw body bytes needed for TLV walking). Use the `RawBlock` body bytes directly.
- `InterfaceInfo` MUST NOT include a `snaplen` field (F-M3 — snaplen is read-and-discarded per
  ADR-009 rev 9; if any `snaplen: u32` field appears in the diff, the review MUST fail).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `if_tsresol` absent | Default `e = 6`; stored in `InterfaceInfo.if_tsresol = 6` |
| EC-002 | `if_tsresol = 0x09` (base-10, nanoseconds) | `e = 9`; stored correctly |
| EC-003 | `if_tsresol = 0x8A` (base-2, e=10) | Bit 7 set; stored as raw byte; BC-2.01.014 interprets base bit |
| EC-004 | Two IDBs with identical `ETHERNET` linktype | Both stored; agreement check passes; `PcapSource.datalink = ETHERNET` |
| EC-005 | Two IDBs with `ETHERNET` then `LINUX_SLL` | E-INP-011 on second IDB: "interface 0 has ETHERNET, interface 1 has LINUX_SLL" |
| EC-006 | IDB with `IEEE802_11` linktype (non-whitelisted) | E-INP-001 at whitelist check #2; error message matches BC-2.01.001 format |
| EC-007 | IDB body 4 bytes (body < 8 minimum) — btl=16 | E-INP-008; no panic |
| EC-008 | IDB `reserved` field non-zero | E-INP-008; mirrors crate enforcement |
| EC-009 | IDB encountered after first EPB emitted | E-INP-013 (position check #1 wins; body not decoded) |
| EC-010 | Late IDB with conflicting linktype | E-INP-013 (position check #1 wins; E-INP-011 never evaluated) |
| EC-011 | IDB options region: `option-length` exceeds remaining bytes | E-INP-008; no OOB read; no panic |
| EC-012 | `if_tsresol` option with `option_length = 4` (not 1) | E-INP-008; NOT silently ignored (F-M5) |
| EC-013 | Two `IEEE802_11` IDBs (both non-whitelisted) | E-INP-001 on FIRST IDB; second never parsed |
| EC-014 | `ETHERNET` then `IEEE802_11` IDB | E-INP-001 on second IDB; E-INP-011 never evaluated (whitelist preempts conflict) |

## Tasks

1. Define `InterfaceInfo` struct with fields `linktype: DataLink` and `if_tsresol: u8` ONLY (no
   `snaplen`). Store instances in `Vec<InterfaceInfo>` local to the pcapng parse state.
2. Implement IDB body decode (pure-core helper on the raw block body slice): check
   `body.len() >= 8`; read `linktype u16 @0-1` (apply section byte-order from SHB), `reserved u16
   @2-3` (assert == 0 else E-INP-008), `snaplen u32 @4-7` (read to advance cursor, discard
   immediately — do NOT store).
3. Implement IDB options TLV walk: starting at body offset 8; for each TLV, read option-code u16
   and option-length u16; bounds-check option-length against remaining body bytes BEFORE reading the
   value or padding; terminate on code 0 (`opt_endofopt`) or end-of-body; for code 9 (`if_tsresol`)
   enforce `option_length == 1` (E-INP-008 if not); for all other codes skip (length + padding).
4. Implement three-level precedence (Decision 17): (1) E-INP-013 position check (`packets_emitted >
   0`), (2) E-INP-001 whitelist check (DataLink not in whitelist set), (3) E-INP-011 conflict check
   (`linktype != interfaces[0].linktype` when table non-empty).
5. Push `InterfaceInfo { linktype, if_tsresol }` onto `Vec<InterfaceInfo>` AFTER all three checks
   pass.
6. Write unit tests covering all ACs above. Include proptest for VP-030 (whitelisted DataLink
   values only, agreement totality).
7. Run `cargo test --all-targets` to verify all existing classic-pcap tests remain green.
8. Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.

## Test Plan

| AC | Test | Type |
|----|------|------|
| AC-001 | `test_BC_2_01_011_linktype_ethernet`, `test_BC_2_01_011_interface_table_is_vec_indexed` | Unit |
| AC-002 | `test_BC_2_01_011_if_tsresol_stored_in_interface_info`, `test_BC_2_01_011_if_tsresol_absent_default` | Unit |
| AC-003 | `test_BC_2_01_011_body_truncated_e_inp_008`, `test_BC_2_01_011_nonzero_reserved_e_inp_008` | Unit |
| AC-004 | `test_BC_2_01_011_options_malformed_length_e_inp_008`, `test_BC_2_01_011_if_tsresol_wrong_length_e_inp_008` | Unit |
| AC-005 | `test_BC_2_01_011_late_idb_after_packet_rejected_e_inp_013` | Unit |
| AC-006 | `test_BC_2_01_011_idb_precedence_e_inp_013_wins_over_conflict` | Unit |
| AC-007 | `test_BC_2_01_016_whitelist_mirrors_bc_2_01_001`, `test_BC_2_01_016_non_whitelisted_linktype_returns_err_no_panic` | Unit |
| AC-008 | `test_BC_2_01_018_two_idbs_different_linktype_e_inp_011`, `test_BC_2_01_018_three_idbs_third_conflicts` | Unit |
| AC-009 | `test_BC_2_01_018_two_idbs_same_linktype_ok`, `test_BC_2_01_011_two_idbs_same_linktype` | Unit |
| AC-010 | `test_BC_2_01_011_no_panic_fuzz` | Property |
| AC-011 | VP-030 proptest (whitelisted DataLink values, agreement totality) | Property |

## Previous Story Intelligence

STORY-123 established the magic-byte probe, SHB parse, section-wide endianness storage, and
`PcapSource::from_pcap_reader` pcapng routing. STORY-124 builds directly on that infrastructure:
- The block-walk loop that STORY-123 established is the integration point where IDB blocks are
  dispatched.
- BC-2.01.010 Invariant 4: section-wide endianness from SHB must already be stored and propagated
  to IDB byte-order decoding in STORY-124 — linktype u16 at bytes 0-1 must be byte-order-corrected
  using the section endianness established by STORY-123's SHB parse.
- The `packets_emitted` counter (needed for E-INP-013 position check) is maintained by the
  block-walk loop established in STORY-123.

## Architecture Compliance Rules

Derived from ADR-009 rev 9 and BC-2.01.011/016/018:

1. **+0 new crates** — `pcap-file` 2.0.0 already in `Cargo.toml`. No additional dependency.
2. **Interface table MUST be `Vec<InterfaceInfo>`** — `interface_id` in EPBs is a 0-based
   sequential index; O(1) by Vec index. HashMap is prohibited (BC-2.01.011 AC-002).
3. **`InterfaceInfo` has exactly two fields:** `linktype: DataLink` and `if_tsresol: u8`. No
   `snaplen` field (F-M3 — snaplen is read-and-discarded; the field has no consumer this cycle).
4. **Options TLV: bounds-check before read** — before reading any option value or its padding,
   check `option-length <= remaining_body_bytes`. OOB panic is prohibited (SEC-005).
5. **`if_tsresol` length enforcement** — code 9 with `option_length != 1` → E-INP-008 immediately.
   Do NOT silently ignore and fall back to the default exponent 6 (F-M5 / ADR-009 rev 9).
6. **Three-level precedence is non-negotiable** — E-INP-013 fires BEFORE body decode; E-INP-001
   fires BEFORE conflict check. Any reordering is a correctness defect against Decision 17.
7. **Body minimum check is wirerust's responsibility** — the crate does NOT enforce `body.len() >= 8`
   on the raw path; wirerust must check it before reading any IDB fixed field (M-1 note).

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `pcap_file` | 2.0.0 | Use `RawBlock` / `next_raw_block` API only; IDB body bytes read directly |
| `anyhow` | existing | `.context(...)` chaining for all IDB errors |
| `std::io::BufReader` | stdlib | Already wrapping `R` from STORY-123; no change needed |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/reader.rs` | Modify | Add `InterfaceInfo` struct; IDB body decode helper; options TLV walk; three-level precedence; multi-IDB agreement check |
| `tests/reader_tests.rs` (or equivalent) | Modify | Add IDB parse tests, whitelist tests, multi-IDB agreement tests, VP-030 proptest |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~6,000 |
| BC files (3 BCs: BC-2.01.011 v1.7, BC-2.01.016 v1.4, BC-2.01.018 v1.6) | ~15,000 |
| ADR-009 rev 9 (canonical constants + relevant decisions) | ~4,000 |
| `src/reader.rs` (post-STORY-123) | ~5,000 |
| Test files | ~4,000 |
| Tool outputs | ~1,000 |
| **Total estimated** | **~35,000** |

Within 20-30% of agent context window.

## Dependency Rationale

- `depends_on: [STORY-123]` — STORY-124 requires the magic-byte probe and SHB parse infrastructure
  from STORY-123; without the pcapng routing and section-wide endianness, the IDB body cannot be
  byte-order-corrected. The block-walk loop that processes IDB blocks is first established by
  STORY-123.
- `blocks: [STORY-125, STORY-126, STORY-127]` — EPB parsing (STORY-125) requires the interface
  table (`Vec<InterfaceInfo>`) and `if_tsresol` populated by STORY-124; without these, EPB
  `interface_id` lookup would crash or produce wrong timestamps. SPB parsing (STORY-126) requires
  the non-empty interface table existence check from STORY-124. STORY-127 requires the full reader
  stack including IDB to run E2E corpus tests.
