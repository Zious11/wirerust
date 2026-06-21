---
document_type: story
story_id: STORY-128
epic_id: E-19
version: "1.0"
status: completed
# BC status: BCs authored and anchored below; all traces complete.
producer: story-writer
timestamp: 2026-06-20T00:00:00Z
phase: f3
points: 3
priority: P0
depends_on: [STORY-127]
blocks: []
behavioral_contracts:
  - BC-2.01.018
verification_properties: []
tdd_mode: strict
target_module: main
subsystems: [SS-12]
estimated_days: 1
feature_id: f3-pcapng-reader-support
wave: 56
inputs:
  - .factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.018.md
# Dependency anchor: STORY-128 depends on STORY-127 because the per-file
#   isolation loop operates on the file list produced by the refactored
#   resolve_targets (STORY-127). STORY-128 must not be dispatched before
#   STORY-127 is merged because the isolation semantics only make sense with
#   the full magic-byte file list in place.
# Subsystem anchor: SS-12 owns this story's scope because the per-file
#   isolation loop lives in src/main.rs (C-1), which is the SS-12 component
#   per ARCH-INDEX Subsystem Registry. BC-2.01.018 AC-002 (re-attributed per
#   ADR-009 Decision 12) designates STORY-128 as the owner of directory-mode
#   per-file error isolation; the BC itself (reader-level conflict rule) lives
#   in SS-01 but the isolation behavior owned by this story is in SS-12.
input-hash: "735a394"
---

# STORY-128: main.rs Per-File Error Isolation Loop (Catch-and-Continue)

## Narrative

- **As a** security analyst running `wirerust <directory>` against a corpus that may contain
  one or more malformed, truncated, or multi-interface-conflict pcapng files alongside valid
  captures
- **I want** wirerust to process each file independently — catching per-file reader errors,
  reporting them to stderr, and continuing to the next file — rather than aborting the
  entire batch on the first error
- **So that** valid files in the directory are always processed even when some files are
  corrupted, and the exit code correctly reflects whether any file failed

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.01.018 | Multi-IDB Link-Type Agreement Policy: Conflict Returns Error (Fail-Closed) |

Note: STORY-128 owns **BC-2.01.018 AC-002 ("Directory-Mode Per-File Isolation")** per
ADR-009 Decision 12. BC-2.01.018 itself owns the E-INP-011 conflict rule (reader.rs scope);
STORY-128 owns the catch-and-continue main.rs loop that catches that Err and continues.
The BC comment states: "OWNED BY STORY-128 (src/main.rs:241-244 loop refactor)."

## Acceptance Criteria

### AC-001 (traces to BC-2.01.018 AC-002 — per-file error isolation; STORY-128 ownership)
In directory mode, the main.rs file-processing loop MUST catch per-file reader errors —
any `Err(anyhow::Error)` returned by `PcapSource::from_pcap_reader` — via an explicit
error arm (match or if-let), NOT via `?` propagation. For each failing file, wirerust MUST:
1. Print the error to stderr using the E-INP-005 format:
   `eprintln!("error: {}: {e:#}", path.display())` (or equivalent pattern that
   includes the path and the full cause chain)
2. Set `any_error = true` (or equivalent tracking variable)
3. `continue` to the next file in the list (do NOT abort the batch)

At the end of the run:
- If ANY file failed: exit code MUST be 1.
- If ALL files succeeded (including zero-packet files): exit code MUST be 0.

**Test:** `test_BC_2_01_018_per_file_isolation_continues_on_error` — directory with two
files: `file_a.pcapng` (crafted to produce E-INP-011 conflict) and `file_b.pcapng` (valid
ETHERNET IDB + EPBs); assert `file_b` produces packets successfully; assert exit code 1;
assert `file_a` error message present on stderr.

### AC-002 (traces to BC-2.01.018 AC-002 — E-INP-011 does not abort batch)
Specifically, an E-INP-011 error (multi-IDB linktype conflict — ETHERNET then LINUX_SLL)
on one file MUST NOT abort the batch. The remaining files in the directory are processed.
This is the canonical motivating case for STORY-128 per BC-2.01.018 EC-009.

**Test:** `test_BC_2_01_018_einp011_does_not_abort_batch`

### AC-003 (traces to BC-2.01.018 AC-002 — all reader error classes isolated)
The isolation applies to ALL reader error classes — not only E-INP-011. Any `Err` from
`from_pcap_reader` is caught and isolated per-file, including:
- E-INP-001 (unsupported link type in IDB)
- E-INP-008 (structural body-decode failure: truncated SHB/IDB/EPB/SPB)
- E-INP-009 (EPB/SPB before any IDB)
- E-INP-010 (crate framing rejection; EPB interface_id OOB)
- E-INP-011 (multi-IDB linktype conflict)
- E-INP-012 (second SHB — multi-section file)
- E-INP-013 (IDB after first packet block — interleaved ordering)

**Test:** `test_BC_2_01_018_any_reader_error_isolated`

### AC-004 (traces to BC-2.01.018 AC-002 — zero-packet notice not suppressed)
For files that succeed with `packets.is_empty()` (zero-packet valid file — e.g.,
SHB-only pcapng, OPB-only pcapng), the zero-packet notice MUST still be emitted to
stderr (per BC-2.01.009 PC6 / BC-2.01.015 PC9 / ADR-009 Decision 19). The per-file
isolation logic MUST NOT suppress this notice. The notice is emitted in the `Ok(source)`
arm of the match; the isolation catch arm handles only `Err(_)`. The two arms are
independent.

**Test:** `test_BC_2_01_018_zero_packet_notice_not_suppressed_by_isolation`

### AC-005 (traces to BC-2.01.018 invariant 1 — isolation does not affect reader behavior)
The per-file isolation is in `main.rs` scope ONLY. `PcapSource::from_pcap_reader`
behavior is UNCHANGED — it still returns `Err` immediately on reader errors. The isolation
is the caller's (main.rs) responsibility, not the reader's. No modification to
`src/reader.rs` is permitted in this story.

**Test:** confirmed structurally by AC-001..003 (reader still returns `Err`; main.rs
catches and continues; no reader-level change).

## Behavioral Contracts Table

| BC | Version | Clauses Covered |
|----|---------|-----------------|
| BC-2.01.018 | v1.6 | AC-002 (directory-mode per-file isolation; re-attributed to STORY-128 per ADR-009 Decision 12; catch-and-continue in main.rs loop for ALL reader Err classes; exit code 1 if any file failed; zero-packet notice not suppressed; reader behavior unchanged), EC-009 (directory with E-INP-011 file and valid file: E-INP-011 isolated, valid file processed, exit code 1), Inv1 (fail-closed policy unchanged — reader still returns Err immediately on conflict; isolation is the caller's concern), Postcondition 2 note (E-INP-011 produced BEFORE any packet is returned; STORY-128 catches this Err at the directory-loop boundary and continues) |

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| Per-file isolation loop (catch-and-continue; replaces `?` propagation) | `src/main.rs` (~lines 241-244 pre-refactor) | Effectful shell (I/O: stderr write; exit code management) |
| Error reporting per file (`eprintln!` to stderr; E-INP-005 format) | `src/main.rs` | Effectful shell |
| Exit code tracking (`any_error: bool` flag) | `src/main.rs` | Pure state (boolean accumulator) |

Architecture section references: `architecture/module-decomposition.md` (SS-12 C-1,
`src/main.rs`); ADR-009 Decision 12 (per-file isolation is main.rs scope; owned by
STORY-128; NOT a reader-level concern), Decision 19 (zero-packet notice emission by
main.rs post-`Ok(source)` — unaffected by isolation catch arm).

## Forbidden Dependencies

- STORY-128 MUST NOT modify `src/reader.rs`. Reader behavior (`from_pcap_reader` returning
  `Err` immediately on error) is unchanged by this story.
- STORY-128 MUST NOT re-implement `resolve_targets` — that is STORY-127 scope.
- STORY-128 MUST NOT add any new crate dependency to `Cargo.toml`.
- STORY-128 MUST NOT suppress the zero-packet notice. The notice is emitted in the `Ok`
  arm of the main.rs match; it is independent of the `Err` catch arm.
- STORY-128 MUST NOT add new CLI flags, new error codes, or new reader-level logic.
  Scope is strictly the file-processing loop error-handling pattern.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Directory with `file_a.pcapng` (E-INP-011 conflict) and `file_b.pcapng` (valid) | `file_a` error printed to stderr; `file_b` processed successfully; exit code 1 |
| EC-002 | Directory with all files valid | All files processed; exit code 0 |
| EC-003 | Directory with all files bad (all produce `Err`) | All errors printed to stderr; exit code 1; no panic, no crash |
| EC-004 | Single valid file (direct path mode, not directory) | Existing behavior unchanged; no regression introduced |
| EC-005 | OPB-only pcapng (returns `Ok` with `packets.len()==0`) followed by good file | Zero-packet notice emitted for OPB file (Ok arm); good file processed; exit code 0 |
| EC-006 | File with E-INP-012 (second SHB — multi-section pcapng) | Error caught and isolated; `any_error = true`; processing continues |
| EC-007 | File with E-INP-008 (truncated EPB — wirerust body-decode failure) | Error caught and isolated; processing continues |
| EC-008 | Empty directory (no files after `resolve_targets` returns `Ok(vec![])`) | No loop iterations; exit code 0; no output |

## Tasks

1. **Identify the file-processing loop** in `src/main.rs` (pre-refactor baseline at
   approximately lines 241-244 where `?` propagates reader errors to `main`).
2. **Add exit-code tracking:** Before the loop, add:
   `let mut any_error = false;`
3. **Refactor the loop:** Replace `?` propagation from `from_pcap_reader` with an
   explicit match or if-let:
   - `Ok(source)` arm: existing processing (zero-packet notice check, analyzer dispatch)
   - `Err(e)` arm: `eprintln!("error: {}: {e:#}", path.display()); any_error = true; continue;`
   The E-INP-005 `with_context(|| format!("Failed to read {:?}", path))` wrapper should
   already be applied before the match; if not, apply it in the Err arm or at the call site.
4. **Set exit code after the loop:** Match the existing codebase style:
   - If `main()` returns `Result<(), anyhow::Error>`: add
     `if any_error { std::process::exit(1); }` before the final `Ok(())` return.
   - If `main()` already uses `std::process::exit`: integrate `any_error` into the
     existing exit-code path. Do NOT change the pattern for non-error exits.
5. **Write integration tests** using tempdir + crafted pcapng byte sequences:
   - Craft a minimal valid pcapng fixture (SHB + ETHERNET IDB + one EPB)
   - Craft a minimal conflict fixture (SHB + ETHERNET IDB + LINUX_SLL IDB => E-INP-011)
   - Assert exit-code behavior (may require subprocess invocation depending on harness)
6. Run `cargo test --all-targets` (verify all prior tests remain green).
7. Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.

## Test Plan

| AC | Test | Type |
|----|------|------|
| AC-001 | `test_BC_2_01_018_per_file_isolation_continues_on_error` | Integration |
| AC-002 | `test_BC_2_01_018_einp011_does_not_abort_batch` | Integration |
| AC-003 | `test_BC_2_01_018_any_reader_error_isolated` | Integration |
| AC-004 | `test_BC_2_01_018_zero_packet_notice_not_suppressed_by_isolation` | Integration |

## Previous Story Intelligence

- STORY-127 refactored `resolve_targets` to return magic-byte-detected files. STORY-128
  refactors the processing loop that iterates over those files. The two refactors are
  ADJACENT in `main.rs` but SEPARATE in scope — STORY-127 touches `resolve_targets`;
  STORY-128 touches the file-processing loop. Do NOT conflate them.
- BC-2.01.018 AC-002 was re-attributed from the reader to main.rs during F2 spec evolution
  (ADR-009 Decision 12 rev 4). The BC comment in `BC-2.01.018.md` explicitly states:
  "Re-attributed per ADR-009 Decision 12, rev 4. OWNED BY STORY-128." This re-attribution
  is normative — do NOT implement isolation behavior in `reader.rs`.
- Exit code behavior: the existing `main() -> Result<(), anyhow::Error>` would have caused
  early return with non-zero exit code on `?`-propagated reader errors. STORY-128 removes
  that early-return path and replaces it with per-file isolation + explicit `any_error`
  tracking. Exit code 1 iff any file failed; processing always completes.
- OPB-only files (zero packets) return `Ok(PcapSource)` with `packets.len()==0` — they
  do NOT produce `Err`. The zero-packet notice is emitted in the `Ok` arm and is NOT
  intercepted by the `Err` catch arm. EC-005 verifies this.

## Architecture Compliance Rules

Derived from ADR-009 Decision 12 and BC-2.01.018 AC-002:

1. **Isolation is `main.rs` scope ONLY** — no modifications to `src/reader.rs`.
   `from_pcap_reader` returns `Err` immediately on reader errors; main.rs catches and
   continues. The reader's fail-closed semantics are preserved.
2. **`?` operator is REMOVED from the file-processing loop** — replaced with explicit
   `match` or `if let` on the `from_pcap_reader` result. Any remaining `?` on the reader
   call in the directory loop is a regression.
3. **Error printed to stderr per file** — use the E-INP-005 format pattern. Include the
   path and the full cause chain via `{e:#}` (anyhow pretty-print format).
4. **Exit code 1 iff any file failed** — even if the majority of files succeeded.
5. **Zero-packet notice is NOT suppressed** — emitted in the `Ok` arm; the isolation
   catch arm handles only `Err(_)`. These arms are orthogonal.
6. **Keep scope tight** — do NOT refactor reader internals, analyzer dispatch, CLI argument
   parsing, or any other `main.rs` aspect beyond the file-processing loop error-handling.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `anyhow` | existing | Error display via `{e:#}` for full cause chain in stderr output |
| `std::process` | stdlib | `std::process::exit(1)` for non-zero exit code; match existing codebase pattern |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/main.rs` | Modify | Refactor file-processing loop: replace `?` propagation with explicit match/if-let; add `any_error` flag; set exit code 1 when `any_error` |
| `tests/integration_tests.rs` or `tests/main_tests.rs` | Modify | Add per-file isolation integration tests (tempdir + crafted pcapng fixtures; exit-code assertions) |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~4,000 |
| BC files (1 BC: BC-2.01.018 v1.6; AC-002 and related clauses) | ~4,000 |
| ADR-009 rev 9 (Decision 12) | ~2,000 |
| `src/main.rs` (file-processing loop + surrounding context) | ~3,000 |
| Test files (integration tests with subprocess invocation) | ~3,000 |
| Tool outputs (cargo test, clippy) | ~1,000 |
| **Total estimated** | **~17,000** |

Well within 20-30% of agent context window.

## Dependency Rationale

- `depends_on: [STORY-127]` — STORY-128 refactors the processing loop that iterates over
  files returned by the refactored `resolve_targets` (STORY-127). The refactored
  `resolve_targets` must be in place first so that the isolation semantics apply to the
  correct (magic-byte-detected) file set.
- `blocks: []` — STORY-128 is the final story in E-19 (pcapng reader support epic). No
  other stories in this epic depend on it. The pcapng feature is complete when STORY-128
  is merged.
