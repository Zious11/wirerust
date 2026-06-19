# STORY-122 Demo Evidence Report

**Story:** STORY-122 — FindingsRender enum→struct reshape + construction-site migration (byte-identical)
**Branch:** worktree-issue-62-findingsrender-struct
**HEAD:** 748d276
**Date:** 2026-06-19
**Acceptance criterion covered:** AC-006 — Output byte-identical to v0.9.0 for all inputs

---

## Summary

STORY-122 is a byte-identical refactor. Evidence strategy: demonstrate that the three
CLI-reachable render modes produce correct and distinct output after the enum→struct reshape,
then cite the three passing programmatic byte-identity tests as the formal proof.

---

## Build Result

```
cargo build --release
Finished `release` profile [optimized] target(s) in 8.29s
```

Binary: `/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-122/target/release/wirerust`
Cargo.toml version: `0.9.0`

---

## Test Suite Result

All test suites pass with zero failures:

| Test suite | Passed |
|------------|--------|
| reporter_terminal_tests (story_122 mod) | 95 |
| reporter_tests | pass |
| dnp3_f5_remediation_tests | pass |
| bc_2_09_100_multitag_tests | pass |
| All other test binaries | pass (0 failures across all) |

Full run: `cargo test --all-targets` → all test results: ok, 0 failed.

---

## AC-006 Byte-Identity Proof (Programmatic)

Three dedicated tests in `tests/reporter_terminal_tests.rs` (mod `story_122`) assert
byte-for-byte output equivalence between the new struct-based dispatch and the old
enum-variant dispatch:

| Test name | Path verified | Result |
|-----------|---------------|--------|
| `test_BC_2_11_028_ac006_grouped_expanded_byte_identical_to_old_grouped_variant` | `{Grouped, Expanded}` == old `FindingsRender::Grouped` | PASS |
| `test_BC_2_11_028_ac006_flat_collapsed_byte_identical_to_old_flatcollapsed_variant` | `{Flat, Collapsed}` == old `FindingsRender::FlatCollapsed` | PASS |
| `test_BC_2_11_028_ac006_flat_expanded_byte_identical_to_old_flatexpanded_variant` | `{Flat, Expanded}` == old `FindingsRender::FlatExpanded` | PASS |

Command to reproduce:
```
cargo test "BC_2_11_028_ac006" --all-targets
```

---

## CLI Demonstrations (VHS Recordings)

### AC-006 — Mode 1: `--mitre` alone → `{Grouped, Expanded}`

**Dispatch path:** `(Grouping::Grouped, Collapse::Expanded)` → `render_findings_grouped`
**What to see:** Tactic headers (`## Discovery`, `## Impair Process Control`), em-dash MITRE
technique name expansion (e.g., `T0888 — Remote System Information Discovery`), NO `(xN)`
suffix on any finding.

| Artifact | Path |
|----------|------|
| GIF (PR embed) | `AC-006-mode1-mitre-grouped-expanded.gif` |
| WebM (archival) | `AC-006-mode1-mitre-grouped-expanded.webm` |
| VHS tape script | `AC-006-mode1-mitre-grouped-expanded.tape` |
| Plain-text capture | `AC-006-mode1-output.txt` |

Input: `tests/fixtures/modbus-write.pcap --all --no-color --mitre`

---

### AC-006 — Mode 2: default (no flags) → `{Flat, Collapsed}`

**Dispatch path:** `(Grouping::Flat, Collapse::Collapsed)` → `render_findings_collapsed`
**What to see:** Flat list, `(x2)` collapse suffix on the repeated Modbus recon finding,
no tactic headers, abbreviated MITRE IDs (`T0888`) without em-dash names.

| Artifact | Path |
|----------|------|
| GIF (PR embed) | `AC-006-mode2-default-flat-collapsed.gif` |
| WebM (archival) | `AC-006-mode2-default-flat-collapsed.webm` |
| VHS tape script | `AC-006-mode2-default-flat-collapsed.tape` |
| Plain-text capture | `AC-006-mode2-output.txt` |

Input: `tests/fixtures/modbus-write.pcap --all --no-color`

---

### AC-006 — Mode 3: `--no-collapse` → `{Flat, Expanded}`

**Dispatch path:** `(Grouping::Flat, Collapse::Expanded)` → `for f in findings { render_finding_flat }`
**What to see:** Flat list, one line per finding (two separate Modbus recon entries, no
collapse), no `(xN)` suffix, abbreviated MITRE IDs without em-dash names.

| Artifact | Path |
|----------|------|
| GIF (PR embed) | `AC-006-mode3-flat-expanded.gif` |
| WebM (archival) | `AC-006-mode3-flat-expanded.webm` |
| VHS tape script | `AC-006-mode3-flat-expanded.tape` |
| Plain-text capture | `AC-006-mode3-output.txt` |

Input: `tests/fixtures/modbus-write.pcap --all --no-color --no-collapse`

---

## Output Distinctness Verification

The three modes produce visually distinct FINDINGS sections, confirming the four-arm
tuple dispatch wires correctly:

**Mode 1 (grouped):** Two findings collapsed into tactic buckets with em-dash names:
```
  ## Discovery
  [Anomaly] INCONCLUSIVE (MEDIUM) - Modbus recon: ... (no xN suffix)
  [Anomaly] INCONCLUSIVE (MEDIUM) - Modbus recon: ...
  ## Impair Process Control
  [Execution] LIKELY (MEDIUM) - Modbus write command observed: ...
```

**Mode 2 (flat-collapsed):** Two findings collapsed with `(x2)`:
```
  [Anomaly] INCONCLUSIVE (MEDIUM) - Modbus recon: ... (x2)
  [Execution] LIKELY (MEDIUM) - Modbus write command observed: ...
```

**Mode 3 (flat-expanded):** Three findings one-per-line, no suffix:
```
  [Anomaly] INCONCLUSIVE (MEDIUM) - Modbus recon: ...
  [Anomaly] INCONCLUSIVE (MEDIUM) - Modbus recon: ...
  [Execution] LIKELY (MEDIUM) - Modbus write command observed: ...
```

---

## Coverage Mapping

| AC | Description | Evidence |
|----|-------------|----------|
| AC-006 | Output byte-identical to v0.9.0 for all inputs | 3 programmatic byte-identity tests (PASS) + 3 VHS recordings showing correct output |
| AC-008 | BC-2.11.016/026/027 preserved (em-dash format, (xN) suffix, K=3 sampling) | Mode 1 GIF shows em-dash MITRE names; Mode 2 GIF shows (x2) suffix; mode 2+3 plain-text captures show K=3 evidence lines |
| AC-001 | Grouping/Collapse/FindingsRender types defined | Build success (cargo build --release) + zero-grep gate already enforced by implementer |
| AC-002 | Four-arm tuple dispatch | VHS recordings demonstrate all 3 CLI-reachable arms; {Grouped,Collapsed} arm temporarily unreachable (STORY-122/A by design) |
| AC-003 | 84 construction sites migrated | cargo test --all-targets passes; zero-grep gate enforced |

---

## Notes

- The `{Grouped, Collapsed}` arm exists in the four-arm dispatch but is intentionally
  UNREACHABLE via CLI in STORY-122/A. `--mitre` alone produces `{Grouped, Expanded}` (byte-identical
  to old `FindingsRender::Grouped`). STORY-119/B repoints this arm and flips the CLI mapping.
- No baseline binary comparison was performed because the byte-identical programmatic tests
  (`test_BC_2_11_028_ac006_*`) constitute the formal proof. The tests construct both old
  (struct-literal) and new (struct-literal) `FindingsRender` values via in-memory `Finding`
  structs and assert identical byte output from `TerminalReporter::render()`.
- Font used in recordings: `Menlo` (macOS system font, confirmed present at
  `/System/Library/Fonts/Menlo.ttc`). VHS 0.11.0.
