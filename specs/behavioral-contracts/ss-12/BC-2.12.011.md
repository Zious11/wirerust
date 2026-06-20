---
document_type: behavioral-contract
level: L3
version: "1.5"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/main.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-12
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v1.5: ADR-009 rev 4 Burst B (Decision 11) — FULL REWRITE: extension-based filtering replaced by MAGIC-BYTE CONTENT DETECTION. Title, description, preconditions, postconditions, invariants, edge cases, test vectors, and verification properties all updated to reflect magic-byte-probe semantics. Resolves C-2: arp-baseline-16pkt.cap (pcapng with .cap extension) now detected by magic. Anchored to STORY-127. — 2026-06-19"
  - "v1.4: F2 audit FINDING-004 — annotate Related BCs BC-2.01.004 ref as STALE (pcapng now accepted via BC-2.01.009); add F3/STORY-127 forward-action note — 2026-06-19"
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: DF-SIBLING-SWEEP-001 — fix stale main.rs line anchor: resolve_targets range 340-360 → 344-364 (fn at 344, bail at 363, closing at 364); also fix inline ref in capability anchor justification and description; verified against HEAD cfe0112a — 2026-06-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.011: Directory Target Expands to Capture Files Detected by Magic Bytes (Content Detection)

## Description

`resolve_targets` in main.rs expands a directory target to all files whose first 4 bytes
match a known pcap or pcapng magic value. Detection is CONTENT-BASED (magic-byte probe),
NOT extension-based. A file named `arp-baseline-16pkt.cap` that begins with pcapng magic
`0x0A0D0D0A` is accepted as a capture; a file named `analysis.pcap` whose first 4 bytes
are `0xDEADBEEF` is silently skipped. The 5 accepted magic values are:

- Classic pcap LE: `0xA1B2C3D4` (bytes: A1 B2 C3 D4)
- Classic pcap BE: `0xD4C3B2A1` (bytes: D4 C3 B2 A1)
- Classic pcap ns-resolution LE: `0xA1B23C4D` (bytes: A1 B2 3C 4D)
- Classic pcap ns-resolution BE: `0x4D3CB2A1` (bytes: 4D 3C B2 A1)
- pcapng SHB: `0x0A0D0D0A` (bytes: 0A 0D 0D 0A)

Files with fewer than 4 bytes are silently skipped. Files that cannot be read (permission
error, I/O error) are silently skipped at the magic-probe stage. The returned list is
sorted lexicographically. This BC is owned by STORY-127. It resolves ADR-009 Decision 11
and C-2 (arp-baseline-16pkt.cap is pcapng-with-.cap-extension, now detected by magic).

## Preconditions

1. The `target` PathBuf points to an existing directory.
2. The directory may contain any mix of files: `.pcap`, `.pcapng`, `.cap`, `.txt`, other
   extensions, or files with no extension.
3. `resolve_targets` has filesystem read access to the directory (directory listing).

## Postconditions

1. Returns `Ok(Vec<PathBuf>)` containing all files in the directory whose first 4 bytes
   match one of the 5 known magic values listed in the Description.
2. Files whose first 4 bytes do NOT match any known magic are silently skipped (not
   included in the result, no error, no warning).
3. Files with fewer than 4 readable bytes are silently skipped.
4. Files that fail the magic-byte read (I/O error, permission denied) are silently skipped
   at the magic-probe stage. (Note: if the same file then passes to the reader, the reader
   will produce a proper error.)
5. The returned Vec is sorted by path (lexicographic order on PathBuf, platform byte comparison).
6. An empty directory returns `Ok(vec![])`.
7. Non-file entries (subdirectories, symlinks to directories) are skipped (is_file() check
   precedes magic probe).

## Invariants

1. Detection is CONTENT-BASED. File extension is IGNORED. A `.cap` file with pcapng magic
   is accepted; a `.pcapng` file with non-magic first bytes is skipped.
2. Exactly 5 magic values are accepted (see Description). A 6th magic value MUST NOT be
   added without a corresponding ADR or BC revision.
3. `files.sort()` is called before returning (lexicographic sort on PathBuf).
4. Subdirectory expansion is NOT recursive; only the immediate directory content is scanned.
5. The magic-probe reads the FIRST 4 BYTES ONLY. It does not validate the full file header
   or block structure — that is the reader's responsibility (BC-2.01.009 et al.).
6. Magic-probe failures are silent (skip, not error) to preserve scan progress across a
   directory of mixed-format files.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Directory with `a.pcap` (LE magic) and `b.pcapng` (pcapng magic) and `c.cap` (pcapng magic) | Returns [a.pcap, b.pcapng, c.cap] sorted — ALL THREE accepted by content |
| EC-002 | `arp-baseline-16pkt.cap` with pcapng magic `0x0A0D0D0A` | Accepted (resolves C-2; ADR-009 Decision 11) |
| EC-003 | `b.pcap` and `a.pcap` both with LE magic | Returns [a.pcap, b.pcap] (sorted) |
| EC-004 | `analysis.pcap` with first 4 bytes `0xDEADBEEF` (not a magic value) | Silently skipped |
| EC-005 | Empty directory | Returns Ok(vec![]) |
| EC-006 | Directory with only `.txt` files (none with pcap magic) | Returns Ok(vec![]) |
| EC-007 | File with 3 bytes (too short for 4-byte probe) | Silently skipped |
| EC-008 | `dump.pcapng` with pcapng magic `0x0A0D0D0A` (Wireshark default capture) | Accepted |
| EC-009 | Classic pcap BE (`0xD4C3B2A1` bytes) with `.pcap` extension | Accepted |
| EC-010 | ns-resolution pcap LE (`0xA1B23C4D` bytes) with `.cap` extension | Accepted |
| EC-011 | Subdirectory named `captures/` inside target directory | Skipped (`is_file()` = false) |
| EC-012 | File with `.PCAP` (uppercase extension) but valid LE magic | Accepted (extension ignored; magic matches) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Dir with [b.pcap (LE magic), a.pcap (LE magic), c.cap (pcapng magic)] | [a.pcap, b.pcap, c.cap] (sorted; all accepted by content) | happy-path |
| Dir with [arp-baseline-16pkt.cap (pcapng magic), x.pcap (LE magic)] | [arp-baseline-16pkt.cap, x.pcap] sorted (resolves C-2) | happy-path (holdout) |
| Empty directory | [] | edge-case |
| Dir with only .txt files (no magic match) | [] | edge-case |
| Dir with [a.pcap (LE magic), b.pcap (wrong bytes)] | [a.pcap] only | edge-case |
| Dir with [a.pcapng (pcapng magic), b.pcap (LE magic), c.txt (no magic)] | [a.pcapng, b.pcap] sorted | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | All 5 magic values accepted; non-magic bytes silently skipped | unit: resolve_targets unit test against tempdir with crafted 4-byte magic files |
| — | Extension-independence: .cap with pcapng magic accepted; .pcap with wrong bytes skipped | unit: tempdir with cross-extension magic files (resolves C-2) |
| — | Sorted output | unit: multi-file tempdir; assert lexicographic sort |
| — | 3-byte file silently skipped (no panic, no error) | unit: create 3-byte file; assert not in result |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md -- resolve_targets (main.rs:344-364 pre-STORY-127; STORY-127 refactors this function) is CAP-12's target-resolution step; determining which files to read via magic-byte content detection is the entry-point orchestration that precedes any ingestion or analysis. Magic-byte detection is required because extension ≠ format (ADR-009 Decision 11) |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (main.rs, C-1) |
| Stories | STORY-127 (implements magic-byte content detection in resolve_targets, main.rs); STORY-128 (per-file isolation in main.rs loop — separate story, separate scope) |
| ADR Reference | ADR-009 Decision 11 ("directory-mode target detection: magic-byte content detection"); Decision 12 (per-file isolation owned by STORY-128, not STORY-127) |
| Origin BC | BC-CLI-011 (pass-3 ingestion corpus, MEDIUM confidence — behavior now revised to content-detection in v1.5) |

## Related BCs

- BC-2.12.012 -- composes with (non-existent targets handled by the else branch)
- BC-2.01.009 -- composes with (magic-byte probe semantics in the reader mirror the magic values detected here)
- ~~BC-2.01.004~~ -- [RETIRED — 2026-06-19] was related (pcapng rejected at reader level; extension-excluded at glob). Both behaviors inverted by F2 feature cycle (ADR-009).
- BC-2.01.018 -- related (per-file isolation for E-INP-011 is re-attributed to STORY-128; directory behavior documented in BC-2.01.018 AC-002)

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/main.rs:344-364` (pre-STORY-127 baseline; STORY-127 will refactor this function) |
| **Confidence** | medium (pre-STORY-127 behavior described; STORY-127 implements content-detection) |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **documentation**: code at lines 344-364 is clear (pre-STORY-127 extension-based behavior)
- **inferred**: no unit test for resolve_targets directory expansion (STORY-127 adds unit test)
- **ADR mandate**: ADR-009 Decision 11 mandates magic-byte content detection (this revision)

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads directory contents (filesystem I/O) + reads first 4 bytes of each candidate file |
| **Global state access** | none |
| **Deterministic** | yes (sorted output; magic-byte probe is deterministic) |
| **Thread safety** | N/A (single-threaded) |
| **Overall classification** | effectful shell (filesystem reads) |

#### Refactoring Notes

The `resolve_targets` function is currently in main.rs. STORY-127 refactors it to implement
magic-byte content detection. Extracting it to a testable library function in lib.rs would
enable unit testing without requiring actual filesystem I/O (could use a tempdir or mock).
This would upgrade the BC to HIGH confidence. The magic-probe I/O (4-byte read per file)
is a new I/O operation relative to the v1.4 extension-based behavior.
