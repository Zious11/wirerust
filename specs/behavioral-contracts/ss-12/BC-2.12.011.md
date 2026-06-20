---
document_type: behavioral-contract
level: L3
version: "1.4"
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
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: DF-SIBLING-SWEEP-001 — fix stale main.rs line anchor: resolve_targets range 340-360 → 344-364 (fn at 344, bail at 363, closing at 364); also fix inline ref in capability anchor justification and description; verified against HEAD cfe0112a — 2026-06-01"
  - "v1.4: F2 audit FINDING-004 — annotate Related BCs BC-2.01.004 ref as STALE (pcapng now accepted via BC-2.01.009); add F3/STORY-127 forward-action note — 2026-06-19"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.011: Directory Target Expands to *.pcap Sorted; *.pcapng Excluded

## Description

`resolve_targets` in main.rs expands a directory target to all files with extension `.pcap`
(case-sensitive) within that directory. The files are sorted lexicographically before being
returned. Files with extension `.pcapng` are NOT included. Non-file entries (subdirectories,
symlinks to directories) are skipped. Only files where `path.is_file() && ext == "pcap"` are
included.

## Preconditions

1. The `target` PathBuf points to an existing directory.
2. The directory may contain `.pcap` files, `.pcapng` files, other files, or be empty.

## Postconditions

1. Returns `Ok(Vec<PathBuf>)` with all `.pcap` files in the directory, sorted.
2. `.pcapng` files are excluded.
3. Files with no extension, `.txt`, or other extensions are excluded.
4. The returned Vec is sorted by path (lexicographic order on PathBuf, which is
   platform-dependent byte comparison).
5. An empty directory returns `Ok(vec![])`.

## Invariants

1. Extension check is `ext == "pcap"` -- case-sensitive on all platforms.
2. `files.sort()` is called before returning (main.rs:360).
3. Subdirectory expansion is NOT recursive; only the immediate directory content is scanned.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Directory with a.pcap and b.pcapng | Returns [a.pcap] only |
| EC-002 | Directory with b.pcap and a.pcap | Returns [a.pcap, b.pcap] (sorted) |
| EC-003 | Empty directory | Returns Ok(vec![]) |
| EC-004 | Directory with only .txt files | Returns Ok(vec![]) |
| EC-005 | Directory with .PCAP (uppercase extension) | Excluded (case-sensitive comparison) |
| EC-006 | Subdirectories inside target directory | Skipped (is_file() = false) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Dir with [b.pcap, a.pcap, c.pcapng] | [a.pcap, b.pcap] (sorted, pcapng excluded) | happy-path |
| Empty directory | [] | edge-case |
| Directory with only .pcapng files | [] | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | *.pcap included, sorted; *.pcapng excluded | unit: resolve_targets unit test (MEDIUM -- not directly tested) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md -- resolve_targets (main.rs:344-364) is CAP-12's target-resolution step; choosing which files to read (filtering by .pcap extension, sorting, and expanding directories) is the entry-point orchestration that precedes any ingestion or analysis |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (main.rs, C-1) |
| Stories | STORY-088 |
| Origin BC | BC-CLI-011 (pass-3 ingestion corpus, MEDIUM confidence -- behavior is in code but not directly tested) |

## Related BCs

- BC-2.12.012 -- composes with (non-existent targets handled by the else branch)
- ~~BC-2.01.004~~ -- [STALE — 2026-06-19] related to (pcapng is rejected at reader level; here it is excluded before reader). **This rationale is now inverted**: BC-2.01.004 was RETIRED by the F2 pcapng-reader-support feature (ADR-009); pcapng is now ACCEPTED via BC-2.01.009 magic-byte probe. The `*.pcapng` directory-glob exclusion in this BC will be revised or retired when STORY-127 is decomposed in F3.

> **F3 FORWARD ACTION (STORY-127):** This BC describes `resolve_targets` excluding `*.pcapng`
> from directory glob expansion. That behavior was correct when reader.rs rejected pcapng.
> Now that BC-2.01.009 accepts pcapng, STORY-127 will update `resolve_targets` to include
> `*.pcapng`. At that point this BC requires revision (update Postcondition 2, Invariants,
> Edge Cases EC-001, and Canonical Test Vectors) or retirement + replacement. Do NOT implement
> this change before STORY-127 is formally decomposed.

## Architecture Anchors

- `src/main.rs:344-364` -- resolve_targets function implementation

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/main.rs:344-364` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **documentation**: code at lines 340-360 is clear
- **inferred**: no unit test for resolve_targets directory expansion

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads directory contents (filesystem I/O) |
| **Global state access** | none |
| **Deterministic** | yes (sorted output) |
| **Thread safety** | N/A (single-threaded) |
| **Overall classification** | effectful shell (filesystem reads) |

#### Refactoring Notes

The `resolve_targets` function is currently in main.rs. Extracting it to a testable library
function in lib.rs would enable unit testing without requiring actual filesystem I/O (could
use a tempdir or mock). This would upgrade the BC to HIGH confidence.
