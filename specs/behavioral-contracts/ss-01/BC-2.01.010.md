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
  - "v1.7: Pass-2 P2a remediation — I-8: split SHB truncation error codes. AC-004/EC-005/EC-008 updated: (a) crate returns block (block_total_length>=12, %4==0) but wirerust body-decode finds body < 16 fixed bytes => wirerust body-decode Err => E-INP-008; (b) crate rejects block_total_length<12 / EOF / misalignment BEFORE returning block => crate framing Err => routed via BC-2.01.017 taxonomy to E-INP-010. EC-005 and EC-008 now specify E-INP-010 for sub-12-byte cases; EC-005 canonical fixture (27 bytes total, body=15 bytes) is case (a) => E-INP-008. New EC-009 captures crate-rejection case (block_total_length=8) => E-INP-010. BOM self-detection verified implementable: RawBlock.body[0..4] = raw BOM; read as BE u32, match 0x1A2B3C4D (big) / 0x4D3C2B1A (little) — spike confirms no regression. I-11: added Test: citations per AC. — 2026-06-19"
  - "v1.6: BOM consistency sweep — eliminated all contradictory u32-value→endianness shorthand from Description, Postcondition 1, AC-001 opening, EC-001, EC-002, EC-007, and Canonical Test Vectors. Every BOM statement now uses unambiguous on-disk byte-sequence form: on-disk bytes 1A 2B 3C 4D ⇒ big-endian section; on-disk bytes 4D 3C 2B 1A ⇒ little-endian section. v1.4 changelog annotation corrected (BE magic was wrongly stated as '0x4D3C2B1A'; annotated as corrected). Consistent with HS-103 v1.2 (BE on-disk 1A 2B 3C 4D) and ADR-009. — 2026-06-19"
  - "v1.5: BOM-1 remediation — AC-001 parenthetical replaced: removed circular 'wire big-endian encoding of 0x1A2B3C4D read big-endian' phrasing; now reads 'the Byte-Order Magic field is the u32 value 0x1A2B3C4D; in a big-endian section the on-disk bytes are 1A 2B 3C 4D, in a little-endian section the on-disk bytes are 4D 3C 2B 1A (the same logical value, opposite byte order); detection compares the read u32 against 0x1A2B3C4D (native) vs 0x4D3C2B1A (byte-swapped) to determine section endianness'. — 2026-06-19"
  - "v1.4: ADR-009 rev 4 Burst B — Add VP-026 to Verification Properties. Add no-panic AC (SEC-005). Correct EC-004: block_total_length<12 is rejected by crate (not 'no error'); remove 'EC-004 is minor version rejection' mislabeling by renumbering — major_version=2 moves to EC-004 (corrected), block_total_length<12 edge case added as EC-007. Align SHB minimum to 28 bytes total (12 outer + 16 body: BOM:4 + major:2 + minor:2 + section_length:8); update E-INP-008 threshold to 28. AC-004 truncation fixture updated to 27 bytes. Add no-panic AC-005. Add holdout scenarios: BE magic 0x4D3C2B1A [CORRECTED in v1.6: BE on-disk bytes are 1A 2B 3C 4D, not 4D 3C 2B 1A; 4D 3C 2B 1A is the LE on-disk pattern], invalid BOM, SHB truncated at byte 15/27. ADR reference updated to include Decision 8. — 2026-06-19"
  - "v1.3: RC-2 flag-spelling consistency — AC-002 and EC-006: standardized remediation hint from 'mergecap -F pcapng' to 'mergecap -w out.pcapng <file>' to match ADR-009 Decision 7 form. Traceability Error Taxonomy note updated to reflect same. No normative behavior change. — 2026-06-19"
  - "v1.2: pcapng-multisection-decision correctness edits — AC-002 rationale reframed: rejection is a SCOPE decision (single-section corpus this cycle; multi-section is rare and absent from corpus), not a correctness workaround. pcap-file 2.0.0 correctly resets interface state per section (source-level verification 2026-06-19; F-06's INCONCLUSIVE premise superseded — see research/pcapng-multisection-decision.md). AC-002 and EC-006 updated to reference the E-INP-012 remediation hint (mergecap/editcap). Normative behavior (reject second SHB → E-INP-012) unchanged. — 2026-06-19"
  - "v1.1: F-06 completeness delta — EC-006 changed from 'reset byte order' (attempt) to REJECT with E-INP-012; AC added: second SHB in a single-section file is rejected; canonical test vector added for 2-section pcapng; error taxonomy cross-reference E-INP-012 added. — 2026-06-19"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.010: Parse pcapng Section Header Block (SHB): Byte-Order Detection and Version

## Description

After the magic-byte probe (BC-2.01.009) identifies a pcapng stream, the reader MUST parse
the Section Header Block (SHB, block type `0x0A0D0D0A`). The SHB carries a Byte-Order Magic
field (on-disk bytes `1A 2B 3C 4D` ⇒ big-endian, `4D 3C 2B 1A` ⇒ little-endian) that governs the endianness of all subsequent
numeric fields in the section. The reader is on the RAW-BLOCK path (ADR-009 Decision 1,
rev 4): `pcap-file` 2.0.0's `RawBlock` / `next_raw_block` API is used for block framing;
wirerust decodes SHB body fields (BOM, major, minor, section_length) directly. The SHB
minimum total block_total_length is 28 bytes (12-byte outer block header + 16 bytes of
fixed body: BOM:4 + major:2 + minor:2 + section_length:8). The crate rejects
`block_total_length < 12` before returning any block (forward-progress contract, Decision 8).

## Preconditions

1. The magic-byte probe (BC-2.01.009) has confirmed the stream begins with pcapng SHB magic.
2. The stream is positioned at byte 0 (probe did not consume bytes).
3. `pcap-file` 2.0.0's raw-block API (`RawBlock` / `next_raw_block`) is in use per ADR-009 Decision 1 (rev 4).

## Postconditions

1. The SHB Byte-Order Magic is read and the byte order for the section is determined by the
   four raw on-disk bytes at BOM position:
   - On-disk bytes `1A 2B 3C 4D` ⇒ big-endian section (the u32 value `0x1A2B3C4D` read big-endian).
   - On-disk bytes `4D 3C 2B 1A` ⇒ little-endian section (the same u32 value `0x1A2B3C4D`
     stored in little-endian byte order; equivalently, a big-endian read yields `0x4D3C2B1A`).
   - Any other four bytes ⇒ invalid Byte-Order Magic ⇒ `Err` mapped to E-INP-008.
2. The pcapng major version MUST be 1; the minor version MAY be any value ≥ 0. A major
   version other than 1 returns `Err` with context identifying the unsupported version.
3. The section length field in the SHB is accepted regardless of value (it may be `-1` / all
   bits set, meaning the length is unspecified). The reader does not use section length for
   bounds checking.
4. After successful SHB parse, the reader proceeds to walk subsequent blocks (IDB, EPB, SPB).
5. SHB truncation generates two distinct error codes depending on where the truncation is
   detected:
   - **(a) Crate-framing layer rejection (E-INP-010):** The crate rejects blocks where
     `block_total_length < 12`, `block_total_length % 4 != 0`, or EOF before the trailer, BEFORE
     returning any block to wirerust. These crate framing errors are routed through BC-2.01.017's
     error taxonomy machinery to E-INP-010. wirerust does not see the SHB body in this case.
   - **(b) wirerust body-decode rejection (E-INP-008):** The crate returns a `RawBlock` with
     `block_total_length >= 12` and valid alignment, but the body slice is fewer than 16 fixed
     bytes (BOM:4 + major:2 + minor:2 + section_length:8). wirerust detects this shortfall during
     SHB body decoding and returns `Err` mapped to E-INP-008. The 28-byte minimum total
     (12-byte outer + 16-byte body) is therefore a wirerust-level constraint on the body, not
     a crate-level constraint.

## Acceptance Criteria

- **AC-001:** Detection is by the four raw on-disk bytes at the BOM field position:
  on-disk bytes `4D 3C 2B 1A` ⇒ little-endian section; on-disk bytes `1A 2B 3C 4D` ⇒
  big-endian section. Any other four bytes ⇒ `Err` (E-INP-008). The Byte-Order Magic encodes
  the u32 value `0x1A2B3C4D` in the section's native byte order; detection reads the field as
  a big-endian u32 and compares: `0x1A2B3C4D` (unchanged) ⇒ big-endian section;
  `0x4D3C2B1A` (byte-reversed) ⇒ little-endian section.
  Implementation: `RawBlock.body[0..4]` is the raw BOM; read as fixed BE u32 and match (spike Q-A1 confirms BOM survives into `RawBlock.body[0..4]` on the raw-block path).
  Holdout: SHB with on-disk bytes `1A 2B 3C 4D` is correctly identified as big-endian mode.
  **Test:** `test_BC_2_01_010_bom_little_endian` / `test_BC_2_01_010_bom_big_endian`
- **AC-002:** A second Section Header Block encountered anywhere after the first is REJECTED
  with `Err` containing context that maps to E-INP-012. wirerust supports single-section
  pcapng files only (scope decision for this cycle — multi-section is rare and absent from
  the intended corpus; pcap-file 2.0.0 itself handles multi-section correctly at the library
  level, but wirerust does not exercise that path). The second SHB's byte-order reset MUST
  NOT be applied before rejection. The E-INP-012 error message includes an actionable
  remediation hint directing users to `mergecap -w out.pcapng <file>` or `editcap` to
  re-save multi-section captures as single-section files (see E-INP-012 in error-taxonomy.md).
  - Canonical fixture: a crafted 2-section pcapng file (SHB₁ + IDB + EPB + SHB₂); expected
    result: reader returns `Err` after consuming SHB₁ and before yielding any packet from
    section 2.
  **Test:** `test_BC_2_01_010_second_shb_rejected_e_inp_012`
- **AC-003:** A pcapng major version other than 1 returns `Err` immediately; no packets are
  emitted from that section.
  **Test:** `test_BC_2_01_010_major_version_not_1_rejected`
- **AC-004:** SHB truncation must map to the correct error code depending on where the
  truncation is detected (Postcondition 5 split):
  - **(a) Body truncation — E-INP-008:** SHB with `block_total_length >= 12` but body < 16 fixed
    bytes (wirerust body-decode failure). Canonical fixture: SHB with `block_total_length = 28`
    but body bytes deliberately short (body = 15 bytes, i.e., 27 bytes total from wire perspective
    but crate-framed as 28 — construct by injecting a valid outer length field but truncated body
    content). Result: `Err` mapping to E-INP-008; no panic.
    **Test:** `test_BC_2_01_010_shb_body_truncation_e_inp_008`
  - **(b) Framing rejection — E-INP-010:** SHB with `block_total_length < 12`
    (e.g., `block_total_length = 8`) or EOF before trailer — crate rejects before returning block;
    error routes to E-INP-010. Note: HS-103 Case C ("15 bytes total") is also case (b) because
    block_total_length < 12 for a 15-byte total block — a separate holdout burst fixes HS-103;
    the error code here is E-INP-010, not E-INP-008.
    **Test:** `test_BC_2_01_010_shb_framing_rejection_e_inp_010`
- **AC-005 (no-panic — SEC-005):** This block parser MUST return `Err` for any malformed or
  truncated SHB byte sequence. `unwrap()`, `expect()`, `panic!()`, and `unreachable!()` are
  prohibited in the SHB parse path. The crate's `RawBlock::from_slice` is Result-clean on
  the framing layer; wirerust's field decode above it must also be Result-clean.
  **Test:** `test_BC_2_01_010_no_panic_fuzz` (property test / fuzz over arbitrary SHB bytes)

## Invariants

1. Byte-order detection is done once per file. A second SHB constitutes a multi-section file;
   wirerust does NOT support multi-section pcapng and MUST reject it with E-INP-012 (see
   AC-002 above). This is a scope decision — multi-section pcapng is rare and absent from
   the intended corpus; pcap-file 2.0.0 handles multi-section correctly at the library level,
   but wirerust does not exercise that path. Attempting to reset byte order on a second SHB
   is NOT permitted — the rejection path fires before any reset can occur.
2. The pcapng specification requires `major_version == 1`; wirerust enforces this hard
   constraint and returns an error for non-1 major versions.
3. The SHB magic bytes (`0x0A0D0D0A`) are not themselves byte-order-dependent; they serve only
   to identify the block type. The BOM field inside the SHB body carries the endianness signal.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Little-endian pcapng (most common on x86) | On-disk BOM bytes `4D 3C 2B 1A`; little-endian mode selected |
| EC-002 | Big-endian pcapng (less common; holdout scenario) | On-disk BOM bytes `1A 2B 3C 4D`; big-endian mode selected; all subsequent block fields read big-endian |
| EC-003 | Section length = `0xFFFFFFFFFFFFFFFF` (unspecified) | Accepted; reader does not use section length for bounds |
| EC-004 | Major version = 2 (future) | `Err` with "Unsupported pcapng major version: 2" context; no packets emitted |
| EC-005 | SHB where crate returns RawBlock (block_total_length=28, body=15 bytes — wirerust sees valid framing but short body) | `Err` mapping to **E-INP-008** (wirerust body-decode detects body < 16 fixed bytes); no panic. Canonical truncation fixture per M-1 resolution. **Test:** `test_BC_2_01_010_shb_body_truncation_e_inp_008` |
| EC-006 | Multi-section pcapng (second SHB mid-file) | `Err` mapping to E-INP-012: "pcapng multi-section files are not supported (second Section Header Block at block #<seq>) (hint: split the capture into single-section files, or re-save with 'mergecap -w out.pcapng <file>' or 'editcap' which emit single-section pcapng)"; wirerust supports single-section pcapng only (scope decision; pcap-file 2.0.0 handles multi-section correctly but wirerust does not exercise that path). No byte-order reset is attempted before rejection. **Test:** `test_BC_2_01_010_second_shb_rejected_e_inp_012` |
| EC-007 | Invalid BOM value (on-disk bytes neither `4D 3C 2B 1A` nor `1A 2B 3C 4D`) | `Err` mapping to E-INP-008; no panic (holdout: craft SHB with BOM on-disk bytes `DE AD BE EF`). **Test:** `test_BC_2_01_010_invalid_bom_e_inp_008` |
| EC-008 | SHB with block_total_length < 12 (e.g., total = 8) — crate rejects before returning block | `Err` routed via BC-2.01.017 taxonomy to **E-INP-010** (crate framing rejection, not wirerust body-decode). No panic. **Test:** `test_BC_2_01_010_shb_framing_rejection_e_inp_010` |
| EC-009 | SHB total length = 15 bytes (block_total_length < 12 or misaligned) — crate rejects (HS-103 Case C category) | `Err` mapping to **E-INP-010** (crate framing Err before block returned to wirerust); HS-103 Case C fix deferred to holdout burst. **Test:** `test_BC_2_01_010_hs103_case_c_e_inp_010` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Well-formed little-endian pcapng SHB | Byte order = little-endian; version (1, 0); parse continues | happy-path |
| Well-formed big-endian pcapng SHB (on-disk BOM bytes `1A 2B 3C 4D`) | Byte order = big-endian; version (1, 0); parse continues | happy-path (holdout) |
| SHB with section length = `0xFFFFFFFFFFFFFFFF` | Parse succeeds; section length ignored | edge-case |
| SHB with major version = 2 | `Err` containing "unsupported" | error |
| SHB with valid block_total_length=28 framing but body truncated to 15 bytes (wirerust body-decode failure) | `Err` (E-INP-008); no panic | error (canonical fixture, M-1, case a) |
| SHB with block_total_length=8 (crate framing rejection before block returned) | `Err` (E-INP-010); no panic | error (crate-rejection path, case b) |
| SHB with invalid BOM (on-disk bytes not `4D 3C 2B 1A` or `1A 2B 3C 4D`) | `Err` (E-INP-008); no panic | error (holdout) |
| Crafted 2-section pcapng (SHB₁ + IDB + EPB + SHB₂) | `Err` (E-INP-012) after SHB₁ section; no packets from section 2 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-026 | SHB parse safety: no panic for any truncated/malformed SHB byte sequence; byte-order BOM detection correct for LE and BE magic values | Kani (P1): pure SHB-body decode function; proves totality over all byte inputs |
| — | Both byte orders produce identical `PcapSource` from identical logical content | unit: craft same-content pcapng in big-endian and little-endian; assert equal packet data |
| — | Truncated SHB never panics (covered by VP-026) | unit + fuzz: truncate well-formed SHB at every offset; assert no panic |
| — | Major version ≠ 1 always returns Err | unit: inject major_version=2 SHB |
| — | Second SHB in any stream always returns E-INP-012 Err | unit: craft 2-section pcapng; assert Err contains "multi-section" / E-INP-012 context |
| — | Invalid BOM always returns Err (E-INP-008) | unit: inject BOM on-disk bytes `DE AD BE EF`; assert Err |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- SHB parsing is the opening gate of pcapng ingestion; byte-order detection is required before any field in the file can be correctly interpreted |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-123 |
| ADR Reference | ADR-009 Decision 1 rev 4 (raw-block path via `RawBlock`/`next_raw_block`), Decision 2 (SHB block coverage), Decision 8 (forward-progress contract: crate rejects block_total_length<12), Decision 10 (panic surface) |
| Error Taxonomy | E-INP-008 (wirerust body-decode: SHB body < 16 fixed bytes when crate returned a valid-framed block; also invalid BOM), E-INP-010 (crate framing rejection: block_total_length<12 / misalignment / EOF-before-trailer — wirerust never sees the block body), E-INP-012 (multi-section SHB reject — scope decision; pcap-file 2.0.0 handles multi-section correctly; wirerust rejects as out-of-scope; message includes `mergecap -w out.pcapng <file>` / editcap remediation hint per ADR-009 Decision 7) |

## Related BCs

- BC-2.01.009 -- depends on (probe routes to this BC's parse path)
- BC-2.01.011 -- composes with (SHB establishes byte order; IDB uses it)
- BC-2.01.012 -- composes with (EPB parsing uses byte order established by SHB)

## Architecture Anchors

- `pcap_file::pcapng::RawBlock` / `next_raw_block` (docs.rs/pcap-file/2.0.0) -- raw-block framing layer (Decision 1 rev 4); wirerust decodes SHB body fields directly
- `block_common.rs:101-103` -- crate rejects block_total_length < 12 before returning any block (Decision 8 forward-progress contract)
- pcapng spec IETF draft §Section-Header-Block: BOM field at body offset 0 (wire offset 8); major/minor version at body offsets 4/6; section_length at body offset 8
- ADR-009 Decision 2: "The reader MUST handle SHB (byte-order detection and version)"
- ADR-009 Decision 8: forward-progress contract; crate rejects < 12 before returning; caller breaks on Err(_)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads stream bytes for SHB block |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O only) |
