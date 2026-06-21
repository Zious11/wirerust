---
document_type: story
story_id: STORY-127
epic_id: E-19
version: "1.0"
status: completed
# BC status: BCs authored and anchored below; all traces complete.
producer: story-writer
timestamp: 2026-06-20T00:00:00Z
phase: f3
points: 5
priority: P0
depends_on: [STORY-123, STORY-124, STORY-125, STORY-126]
blocks: [STORY-128]
behavioral_contracts:
  - BC-2.12.011
verification_properties: []
tdd_mode: strict
target_module: main
subsystems: [SS-12]
estimated_days: 2
feature_id: f3-pcapng-reader-support
wave: 55
inputs:
  - .factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.011.md
# Dependency anchor: STORY-127 depends on STORY-123..126 because resolve_targets
#   (main.rs) now accepts pcapng files, and the E2E corpus tests exercise the
#   full reader stack (SHB+IDB+EPB+SPB+skip) that those stories implement. An
#   E2E test that fires before the reader stack is complete would generate false
#   failures. STORY-127 does NOT need all 4 predecessors to be in the same commit
#   — it just must not be dispatched before STORY-123..126 are merged.
# Subsystem anchor: SS-12 owns this story's scope because BC-2.12.011 is an
#   SS-12 behavioral contract (main.rs, C-1) per the ARCH-INDEX Subsystem
#   Registry. resolve_targets lives in main.rs, not reader.rs.
input-hash: "3df9e4b"
---

# STORY-127: Magic-Byte Glob (resolve_targets Content Detection) and E2E Corpus Wiring

## Narrative

- **As a** security analyst running `wirerust <directory>` against a directory containing a
  mix of classic pcap, pcapng (with `.pcapng`), and pcapng-with-`.cap`-extension files
- **I want** `resolve_targets` to detect files by magic-byte content (not by extension),
  so that `arp-baseline-16pkt.cap` (pcapng with `.cap`) is included, `analysis.pcap` with
  wrong magic is excluded, and all 5 canonical magic values are recognized
- **So that** directory-mode processing automatically includes the full capture corpus
  without requiring file renames, and the E2E integration test suite validates the complete
  pcapng reader stack against real fixtures

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.12.011 | Directory Target Expands to Capture Files Detected by Magic Bytes (Content Detection) |

## Acceptance Criteria

### AC-001 (traces to BC-2.12.011 postcondition 1 — all 5 magic values accepted)
`resolve_targets` MUST accept exactly these 5 magic values by comparing the first 4 bytes
of each file's content:
- Classic pcap LE: `[0xA1, 0xB2, 0xC3, 0xD4]`
- Classic pcap BE: `[0xD4, 0xC3, 0xB2, 0xA1]`
- Classic pcap ns-resolution LE: `[0xA1, 0xB2, 0x3C, 0x4D]`
- Classic pcap ns-resolution BE: `[0x4D, 0x3C, 0xB2, 0xA1]`
- pcapng SHB: `[0x0A, 0x0D, 0x0D, 0x0A]`

Detection is CONTENT-BASED; file extension is IGNORED. No 6th magic value may be added
without a corresponding BC revision (BC-2.12.011 Invariant 2).

**Test:** `test_BC_2_12_011_all_5_magic_values_accepted`

### AC-002 (traces to BC-2.12.011 postcondition 2 — non-magic files silently skipped)
Files whose first 4 bytes do NOT match any of the 5 magic values are silently skipped
(not included in the result, no error, no warning). Example: `analysis.pcap` with first
bytes `[0xDE, 0xAD, 0xBE, 0xEF]` is silently excluded.

**Test:** `test_BC_2_12_011_non_magic_silently_skipped`

### AC-003 (traces to BC-2.12.011 postcondition 3 — short files silently skipped)
Files with fewer than 4 readable bytes are silently skipped. No panic, no error, no
warning.

**Test:** `test_BC_2_12_011_short_file_skipped`

### AC-004 (traces to BC-2.12.011 invariant 1 — extension independence, resolves C-2)
`arp-baseline-16pkt.cap` (pcapng with `.cap` extension) MUST be accepted when its first
4 bytes are `[0x0A, 0x0D, 0x0D, 0x0A]`. A `.pcap` file with wrong first bytes is
rejected regardless of its extension. This resolves ADR-009 C-2 — the key motivating
failure was that `arp-baseline-16pkt.cap` was excluded by the prior extension-based filter.

**Test:** `test_BC_2_12_011_cap_extension_pcapng_magic_accepted` (the C-2 regression fixture)

### AC-005 (traces to BC-2.12.011 postcondition 5 — sorted output)
The returned `Vec<PathBuf>` MUST be sorted lexicographically (PathBuf comparison,
platform byte order). `files.sort()` MUST be called before returning to ensure
deterministic, reproducible test assertions.

**Test:** `test_BC_2_12_011_sorted_output`

### AC-006 (traces to BC-2.12.011 postcondition 6 — empty directory)
An empty directory returns `Ok(vec![])`. No error, no warning.

**Test:** `test_BC_2_12_011_empty_directory`

### AC-007 (traces to BC-2.12.011 postcondition 4 — magic-probe I/O errors silently skipped)
Files that fail the magic-byte read (permission denied, I/O error, unreadable) are
silently skipped at the magic-probe stage. The probe failure does NOT abort directory
scanning. If the same file is subsequently passed to the reader, the reader will produce
a proper error.

**Test:** `test_BC_2_12_011_io_error_on_probe_silently_skipped`

### AC-008 (traces to BC-2.12.011 postcondition 7 — non-file entries skipped)
Subdirectories and symlinks to directories are skipped. The `is_file()` check MUST
precede the magic probe. Subdirectory expansion is NOT recursive — only the immediate
directory contents are scanned.

**Test:** `test_BC_2_12_011_subdir_skipped`

### AC-009 (traces to BC-2.12.011 edge cases EC-001..002 — E2E corpus wiring)
After refactoring `resolve_targets`, the following integration tests MUST pass end-to-end
against the full reader stack (STORY-123..126 must be merged first):
- `smb3.pcapng`: `Ok(PcapSource)` via pcapng reader (routed by STORY-123 probe)
- `arp-baseline-16pkt.cap`: `Ok(PcapSource)` with `packets.len() == 16` via pcapng path
  (resolves C-2; STORY-123 probe + STORY-125 EPB parse)
- A synthetic two-IDB pcapng fixture (both `ETHERNET`): `Ok(PcapSource)` with
  `datalink == ETHERNET` (STORY-124 multi-IDB agreement pass)
- A synthetic OPB-only pcapng: `Ok(PcapSource)` with `packets.len() == 0` and
  `opb_skipped > 0` (STORY-126 skip dispatch + counter surfacing)

**Test:** `test_BC_2_12_011_e2e_corpus_pcapng_reader_stack` (integration suite running
`PcapSource::from_pcap_reader` against corpus fixtures; asserts packet counts and file
properties)

## Behavioral Contracts Table

| BC | Version | Clauses Covered |
|----|---------|-----------------|
| BC-2.12.011 | v1.5 | PC1 (all 5 magic values accepted by content), PC2 (non-magic silently skipped), PC3 (short files silently skipped), PC4 (magic-probe I/O errors silently skipped), PC5 (sorted output), PC6 (empty directory → Ok(vec![])), PC7 (non-file entries skipped; is_file() precedes probe), Inv1 (content-based detection; extension ignored), Inv2 (exactly 5 magic values; 6th requires BC revision), Inv3 (files.sort() before return), Inv4 (non-recursive; immediate directory only), Inv5 (magic probe reads FIRST 4 BYTES only), Inv6 (probe failures are silent), EC-001..012 |

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `resolve_targets` (refactored to magic-byte content detection) | `src/main.rs` | Effectful shell (filesystem I/O: directory list + 4-byte file probe per entry) |
| `read_magic(path: &Path) -> Option<[u8; 4]>` helper | `src/main.rs` or `src/lib.rs` | Effectful shell (4-byte file read; returns None on I/O error or < 4 bytes) |
| E2E corpus test fixtures | `tests/` or fixture directory | Test infrastructure |

Architecture section references: `architecture/module-decomposition.md` (SS-12 C-1,
`src/main.rs`; `resolve_targets` pre-STORY-127 baseline at lines 344-364); ADR-009
Decision 11 ("directory-mode target detection: magic-byte content detection; resolves C-2"),
Decision 12 (per-file isolation is STORY-128 scope, NOT STORY-127 scope).

## Forbidden Dependencies

- `resolve_targets` refactor MUST NOT introduce any new crate dependency.
- Per-file error isolation (catch-and-continue in the main.rs file-processing loop) MUST
  NOT be implemented in STORY-127. That is STORY-128 scope per ADR-009 Decision 12.
  STORY-127 touches `resolve_targets` only; it does NOT touch the file-processing loop.
- Extension-based filtering (e.g., filtering by `.pcapng`, `.pcap`, `.cap` extension
  strings) MUST be REMOVED entirely from `resolve_targets`. Magic-byte detection is the
  ONLY permitted mechanism per BC-2.12.011 Invariant 1.
- The magic probe MUST read ONLY the first 4 bytes — it does NOT validate the full file
  header or block structure. Full validation is the reader's responsibility.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Dir with `a.pcap` (LE magic), `b.pcapng` (pcapng magic), `c.cap` (pcapng magic), `d.txt` (no magic) | Returns [a.pcap, b.pcapng, c.cap] sorted; d.txt silently excluded |
| EC-002 | `arp-baseline-16pkt.cap` with pcapng magic `0x0A0D0D0A` in target directory | Accepted (resolves C-2; ADR-009 Decision 11) |
| EC-003 | `b.pcap` and `a.pcap` both with LE magic | Returns [a.pcap, b.pcap] (lexicographic sort) |
| EC-004 | `analysis.pcap` with first 4 bytes `0xDEADBEEF` | Silently skipped |
| EC-005 | Empty directory | `Ok(vec![])` |
| EC-006 | Directory with only `.txt` files (none with pcap magic) | `Ok(vec![])` |
| EC-007 | File with 3 bytes (too short for 4-byte probe) | Silently skipped; no panic |
| EC-008 | `dump.pcapng` with pcapng magic `0x0A0D0D0A` | Accepted (Wireshark default capture) |
| EC-009 | Classic pcap BE (`0xD4C3B2A1`) with `.pcap` extension | Accepted |
| EC-010 | ns-resolution LE (`0xA1B23C4D`) with `.cap` extension | Accepted |
| EC-011 | Subdirectory `captures/` inside target directory | Skipped (`is_file() = false`) |
| EC-012 | File with `.PCAP` (uppercase extension) but valid LE magic | Accepted (extension ignored; magic matches) |

## Tasks

1. **Refactor `resolve_targets` in `src/main.rs`** (pre-STORY-127 baseline at lines
   344-364):
   - Iterate over `fs::read_dir(dir)?` entries.
   - For each entry, call `entry.path().is_file()` first; skip if false (EC-011).
   - Call `read_magic(&path)` — returns `None` on I/O error or < 4 bytes.
   - Match on the 4-byte magic array against the 5 canonical values (AC-001).
   - If match → push `path` to `files: Vec<PathBuf>`.
   - Call `files.sort()` before returning.
   - Remove ALL extension-based filtering from the function.
2. **Extract `read_magic(path: &Path) -> Option<[u8; 4]>` helper:**
   - Open the file; read exactly 4 bytes.
   - Return `None` if I/O error or fewer than 4 bytes available.
   - Return `Some([b0, b1, b2, b3])` on success.
   - If extracting to `src/lib.rs` for testability, use `pub(crate)`.
3. **Write `test_BC_2_12_011_*` unit tests** using tempdir for all ACs (AC-001..008).
   Create temporary files with crafted 4-byte magic headers; assert inclusion/exclusion.
4. **Wire E2E corpus integration tests (AC-009):** Run
   `PcapSource::from_pcap_reader` on pcapng fixtures; assert expected `packets.len()` and
   `datalink` values. Reference existing fixture paths (`smb3.pcapng`, `arp-baseline-16pkt.cap`).
5. Run `cargo test --all-targets` (verify all prior reader tests remain green).
6. Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.

## Test Plan

| AC | Test | Type |
|----|------|------|
| AC-001 | `test_BC_2_12_011_all_5_magic_values_accepted` | Unit (tempdir) |
| AC-002 | `test_BC_2_12_011_non_magic_silently_skipped` | Unit (tempdir) |
| AC-003 | `test_BC_2_12_011_short_file_skipped` | Unit (tempdir) |
| AC-004 | `test_BC_2_12_011_cap_extension_pcapng_magic_accepted` | Unit (tempdir; C-2 regression) |
| AC-005 | `test_BC_2_12_011_sorted_output` | Unit (tempdir) |
| AC-006 | `test_BC_2_12_011_empty_directory` | Unit (tempdir) |
| AC-007 | `test_BC_2_12_011_io_error_on_probe_silently_skipped` | Unit (tempdir) |
| AC-008 | `test_BC_2_12_011_subdir_skipped` | Unit (tempdir) |
| AC-009 | `test_BC_2_12_011_e2e_corpus_pcapng_reader_stack` | Integration |

## Previous Story Intelligence

- The pre-STORY-127 `resolve_targets` (baseline `src/main.rs:344-364`) used extension-based
  filtering (only `.pcap` and `.cap` extensions). This excluded `smb3.pcapng` from directory
  scans entirely and relied on file extension rather than content. STORY-127 removes all
  extension logic and replaces it with the magic-byte probe.
- STORY-123 established pcapng routing in `from_pcap_reader` and confirmed `smb3.pcapng`
  now returns `Ok(PcapSource)`. STORY-127 ensures `resolve_targets` no longer filters out
  `.pcapng` extension files before they reach the reader.
- The `read_magic` helper performs the same 4-byte peek that `from_pcap_reader`'s internal
  BufReader probe performs — but it is a SEPARATE operation on disk (open + read 4 bytes)
  rather than the in-memory `BufReader::fill_buf()` peek. These are two distinct reads; the
  `read_magic` probe for directory scanning does NOT bypass the reader's own probe.
- Per-file error isolation is STORY-128 scope. If reviewing `main.rs` and noticing the
  file-processing loop still uses `?` propagation, do NOT fix it here — leave a comment
  or note it, but do NOT implement it in STORY-127.

## Architecture Compliance Rules

Derived from ADR-009 Decision 11 and BC-2.12.011:

1. **Content-based detection only** — all extension filtering is REMOVED. Any remaining
   `.pcap`/`.pcapng`/`.cap` string comparison in `resolve_targets` after this story is a
   regression.
2. **Exactly 5 magic values** — `{LE, BE, ns-LE, ns-BE, pcapng-SHB}`. Adding a 6th
   requires a BC revision (BC-2.12.011 Invariant 2). Hard-code the 5 arrays; no enum.
3. **Magic-probe failures are silent** — I/O errors on the probe skip the file silently
   and do NOT abort directory scanning. The caller is responsible for proper error handling
   if the same file then fails the reader.
4. **`files.sort()` before return** — deterministic lexicographic order is mandatory for
   reproducible test assertions (BC-2.12.011 Invariant 3).
5. **`is_file()` before probe** — subdirectories MUST be excluded before any 4-byte read
   attempt (EC-011; BC-2.12.011 PC7).
6. **Per-file isolation is STORY-128 scope** — STORY-127 MUST NOT add catch-and-continue
   error handling in the main.rs file-processing loop. That refactor is STORY-128's
   responsibility (ADR-009 Decision 12).

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `std::fs` | stdlib | `fs::read_dir` for directory iteration; `File::open` for magic probe |
| `std::io` | stdlib | `Read::read` or `Read::read_exact` for 4-byte magic probe |
| `tempfile` | existing | tempdir for unit tests (if already in dev-dependencies) |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/main.rs` | Modify | Refactor `resolve_targets` to magic-byte content detection; remove extension filtering |
| `src/main.rs` or `src/lib.rs` | Modify | Add `read_magic(path: &Path) -> Option<[u8; 4]>` helper (pub(crate) for testability) |
| `tests/main_tests.rs` or `tests/integration_tests.rs` | Create/Modify | `test_BC_2_12_011_*` unit tests (tempdir) + E2E integration test |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~5,000 |
| BC files (1 BC: BC-2.12.011 v1.5) | ~5,000 |
| ADR-009 rev 9 (Decision 11, Decision 12) | ~3,000 |
| `src/main.rs` (resolve_targets function + surrounding context) | ~3,000 |
| Test files + fixtures | ~4,000 |
| Tool outputs (cargo test, clippy) | ~1,000 |
| **Total estimated** | **~21,000** |

Within 20-30% of agent context window.

## Dependency Rationale

- `depends_on: [STORY-123, STORY-124, STORY-125, STORY-126]` — All four predecessor
  stories must be complete before STORY-127's E2E corpus tests (AC-009) can produce valid
  results. The reader stack (SHB+IDB+EPB+SPB+skip dispatch) must be fully functional for
  the integration tests to not produce spurious failures. The `resolve_targets` refactor
  itself (ACs 001-008) is independent and could be implemented before STORY-123..126, but
  the E2E wiring (AC-009) must wait.
- `blocks: [STORY-128]` — STORY-128 (per-file isolation in the main.rs processing loop)
  operates on the file list produced by the refactored `resolve_targets`. STORY-128 must
  not be dispatched before STORY-127 is merged because the isolation semantics only make
  sense once `resolve_targets` correctly identifies all capture files by magic bytes.
