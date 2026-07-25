---
document_type: story
story_id: STORY-182
epic_id: E-11
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-07-25T00:00:00Z
phase: f7
level: maintenance
cycle: wave-086
points: 4
priority: P2
depends_on: []
blocks: []
# BC status: E-11 convention — governance-only story; no BCs authored
behavioral_contracts: []
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: tests/
subsystems: []
estimated_days: 2
wave: "86"
traces_to:
  - .factory/cycles/wave-085/lessons.md
  - .factory/planning/df-validation-2026-07-25.md
  - tests/iec104_e2e_real_pcaps_tests.rs
  - tests/fixtures/committed-samples/
  - .gitignore
inputs:
  - .factory/cycles/wave-085/lessons.md
  - .factory/planning/df-validation-2026-07-25.md
input-hash: "f025b3b"
---

# STORY-182: E2E Fixture Manifest + Committed Representative Captures: Eliminate False-Green cargo test in Clean Worktrees

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** 86
**Points:** 4
**Priority:** P2

## Narrative

- **As a** CI system, gate reviewer, and contributor running `cargo test` in a clean worktree
- **I want** the IEC-104 ITI e2e fixture-gated tests to hard-fail when their committed captures
  are absent, to report visible skip notices for gitignored corpus files, AND for committed ITI
  captures to ensure CI genuinely exercises the timed-command detection path from STORY-180
- **So that** "31 tests ran out of 66 expected" can never appear silently again — the wave-85
  gate G1 initial FAIL (D-510, PG-W85-005) is structurally prevented: absence of committed
  captures is always a visible CI failure, absence of gitignored corpus is always loud, and the
  timed-command code path always runs in CI

## Behavioral Contracts

_(none — E-11 convention: governance-only story; no BCs authored; tooling-and-test change only)_

## Background

### PG-W85-005 — Gitignored machine-local e2e fixtures produce false-green `cargo test`

The IEC-104 ITI e2e tests in `tests/iec104_e2e_real_pcaps_tests.rs` discover their fixture
files at runtime using the `fixture_present()` helper (line 63). Current implementation:

```rust
fn fixture_present(filename: &str) -> bool {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(LOCAL_SAMPLES)   // hardcoded const LOCAL_SAMPLES = "tests/fixtures/local-samples"
        .join(filename);
    if !path.exists() {
        eprintln!("[iec104-e2e] SKIP: fixture '{}' not found at {}. …", filename, path.display());
        false
    } else {
        true
    }
}
```

When a fixture is absent the function prints to stderr and returns false; the calling `#[test]`
returns early. From cargo test's perspective the test **passed** — no panic, no skip marker.

`run_iec104_pipeline()` (line 97) independently hardcodes `LOCAL_SAMPLES`:
```rust
fn run_iec104_pipeline(filename: &str) -> Iec104Analyzer {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(LOCAL_SAMPLES)          // <— hardcoded; panics if file absent
        .join(filename);
    let source = PcapSource::from_file(&path)
        .unwrap_or_else(|e| panic!("[iec104-e2e] failed to open {filename}: {e:#}"));
    …
}
```

If `fixture_present()` is extended to check `COMMITTED_SAMPLES` but `run_iec104_pipeline()`
still uses `LOCAL_SAMPLES`, a committed fixture would pass the presence check but cause a
panic at open time (file not found in `local-samples/`). These two functions MUST use a
shared path resolver.

### Fixture governance surfaces

The following EXISTING governance surfaces apply to committed fixtures (F-016):
- `tests/fixtures/README.md` (lines 5–34): licensing notice + provenance table. ALL committed
  fixtures must have a row added to this table with source URL, modification indication, and
  license. This is the authoritative attribution record — not a separate `ATTRIBUTION.md`.
- Malware-content check: committed captures must not contain live malware C2 traffic or
  exploit payloads (README.md §Licensing notice).
- CC-BY-4.0 attribution for ITI captures: "ICS Security Tools, Illinois Institute of
  Technology (ITI). Licensed under CC-BY-4.0." Per-file: source URL, modification note,
  upstream license notice retention. Attribution is already recorded in E2E-PCAPS.md §IEC-104;
  the README.md provenance table row is the additional required surface.

### Committed captures decision (F-021 resolution)

Both ITI CC-BY-4.0 captures are MANDATORY committed fixtures:
- `iec104-iti-diverse.pcap`: 14 KB (< 100 KB), ITI CC-BY-4.0. Exercises TypeIDs 58–64
  (timed control commands, BC-2.19.029/030). Produces 66 findings with STORY-180 detection.
- `iec104-iti-dissect.pcap`: 11 KB (< 100 KB), ITI CC-BY-4.0. Exercises 6-flow dissector
  path. Produces 11 findings (T0814×2 + T1692.001×9).

Both are committed. The FIXTURE_MANIFEST contains 4 entries. In a clean checkout (only
committed samples present), the manifest-report shows **2/4 fixtures present** (the two
Wireshark samples, iec104.pcap and iec104-sq.pcapng, are NOT committed — "not redistributed").

### Must NOT commit

- `iec104.pcap`: Wireshark Foundation "not redistributed" — see test-file header and E2E-PCAPS.md.
- `iec104-sq.pcapng`: Wireshark Foundation "not redistributed" — same.

### Gitignore placement

`tests/fixtures/committed-samples/` is NOT covered by any `.gitignore` entry (`.gitignore`
covers only `/tests/fixtures/local-samples/`). No `.gitignore` change required.

### Fixture count anchors

All count literals in this story use the MANDATORY decision: 2 committed captures (both ITI),
4 total manifest entries. The canonical manifest-report string in a clean checkout is:
`Fixture coverage: 2/4 fixtures present (2 fixture-gated tests will be skipped)`.

## Acceptance Criteria

### AC-182-001 (traces to PG-W85-005 — shared resolver + fixture manifest with loud skip reporting)

A shared path resolver `fixture_path(name) -> Option<PathBuf>` is introduced and used by
BOTH `fixture_present()` and `run_iec104_pipeline()`. A manifest mechanism reports fixture
coverage on every run.

- Given `fixture_present()` (line 63) currently hardcodes `LOCAL_SAMPLES` and returns false
  (with eprintln to stderr) when absent; and `run_iec104_pipeline()` (line 97) independently
  hardcodes `LOCAL_SAMPLES` and panics on open failure
- When a new helper function is added:
  ```rust
  /// Resolve the path of a fixture file, checking committed-samples/ before local-samples/.
  ///
  /// Returns Some(path) if the file exists in either location, None if absent in both.
  fn fixture_path(filename: &str) -> Option<std::path::PathBuf> {
      let base = Path::new(env!("CARGO_MANIFEST_DIR"));
      let committed = base.join(COMMITTED_SAMPLES).join(filename);
      if committed.exists() { return Some(committed); }
      let local = base.join(LOCAL_SAMPLES).join(filename);
      if local.exists() { return Some(local); }
      None
  }
  ```
- And `fixture_present()` is updated to use `fixture_path()`:
  ```rust
  fn fixture_present(filename: &str) -> bool {
      match fixture_path(filename) {
          Some(_) => true,
          None => {
              // Stderr: preserve existing [iec104-e2e] SKIP: diagnostic with path display
              eprintln!(
                  "[iec104-e2e] SKIP: fixture '{}' not found in committed-samples or \
                   local-samples. Run `bin/fetch-e2e-pcaps` to populate local-samples.",
                  filename
              );
              false
          }
      }
  }
  ```
- And `run_iec104_pipeline()` is updated to use `fixture_path()` and panics with a clear
  message if the file is absent (not a skip — at this call site the fixture was already
  confirmed present by `fixture_present()`):
  ```rust
  fn run_iec104_pipeline(filename: &str) -> Iec104Analyzer {
      let path = fixture_path(filename)
          .unwrap_or_else(|| panic!(
              "[iec104-e2e] fixture_path returned None for '{}' — \
               fixture_present() must be called before run_iec104_pipeline()",
              filename
          ));
      let source = PcapSource::from_file(&path)
          .unwrap_or_else(|e| panic!("[iec104-e2e] failed to open {filename}: {e:#}"));
      …
  }
  ```
- And a new `FIXTURE_MANIFEST: &[&str]` constant lists all 4 expected fixture filenames:
  `["iec104.pcap", "iec104-sq.pcapng", "iec104-iti-diverse.pcap", "iec104-iti-dissect.pcap"]`
- And a new `#[test]` function `test_fixture_manifest_report()` is added (see AC-182-005 for
  hard-assert partition; this AC covers the skip-reporting half for gitignored fixtures):
  ```rust
  #[test]
  fn test_fixture_manifest_report() {
      let present: Vec<&str> = FIXTURE_MANIFEST.iter()
          .copied()
          .filter(|n| fixture_path(n).is_some())
          .collect();
      let absent: Vec<&str> = FIXTURE_MANIFEST.iter()
          .copied()
          .filter(|n| fixture_path(n).is_none())
          .collect();
      // Stdout: visible with --nocapture; for absent fixtures this is advisory
      println!(
          "Fixture coverage: {}/{} fixtures present ({} fixture-gated tests will be skipped)",
          present.len(), FIXTURE_MANIFEST.len(), absent.len()
      );
      for name in &absent {
          println!(
              "FIXTURE-SKIPPED: '{}' absent — corpus test will not run \
               (check tests/fixtures/local-samples/ or tests/fixtures/committed-samples/)",
              name
          );
      }
      // Hard-assert: committed/tracked fixtures MUST be present (see AC-182-005)
      // Gitignored fixtures absence is advisory only — no panic
  }
  ```

- Then `cargo test --test iec104_e2e_real_pcaps_tests test_fixture_manifest_report -- --nocapture`
  in a clean worktree (both committed captures present) prints:
  `Fixture coverage: 2/4 fixtures present (2 fixture-gated tests will be skipped)`

Verification:
```bash
cargo test --test iec104_e2e_real_pcaps_tests test_fixture_manifest_report -- --nocapture
# Must print "Fixture coverage: 2/4 fixtures present" to stdout
```

### AC-182-002 (traces to PG-W85-005 — committed mandatory captures)

Both ITI CC-BY-4.0 representative captures are committed under `tests/fixtures/committed-samples/`
and their provenance is recorded in `tests/fixtures/README.md`.

- Given `iec104-iti-diverse.pcap` (14 KB, 173 packets, ITI CC-BY-4.0) exercises TypeIDs 58–64
  and is the ground-truth capture for STORY-180's timed-command detection (66 findings); and
  `iec104-iti-dissect.pcap` (11 KB, 147 packets, ITI CC-BY-4.0) exercises 6 flows with
  TypeIDs C_SC/C_DC/C_SE (11 findings: T0814×2 + T1692.001×9) — both licensed CC-BY-4.0
  permitting redistribution with attribution
- When the implementer verifies on a fixture-bearing host that BOTH files are ≤ 100 KB
  (E2E-PCAPS.md §IEC-104 records: diverse=14 KB, dissect=11 KB — both well within the limit)
- And copies BOTH files to `tests/fixtures/committed-samples/`
- And adds a provenance row to `tests/fixtures/README.md` in the existing "Fixtures with
  recorded provenance" table (lines 30–34), or adds a new IEC-104 subsection following the
  established table format:
  ```
  | `iec104-iti-diverse.pcap` | [ITI/ICS-Security-Tools](https://github.com/ITI/ICS-Security-Tools/tree/master/pcaps/IEC60870-5-104) | IEC-104; 173 packets; 14 KB; CC-BY-4.0; exercises TypeIDs 58–64 timed control commands (BC-2.19.029/030); produces 66 findings (STORY-180 ground-truth). |
  | `iec104-iti-dissect.pcap` | [ITI/ICS-Security-Tools](https://github.com/ITI/ICS-Security-Tools/tree/master/pcaps/IEC60870-5-104) | IEC-104; 147 packets; 11 KB; CC-BY-4.0; 6-flow dissector exercise; produces 11 findings (T0814×2 + T1692.001×9). |
  ```
  Attribution text in README.md (or a companion `tests/fixtures/committed-samples/ATTRIBUTION.md`
  as complementary, not replacement): "ICS Security Tools, Illinois Institute of Technology
  (ITI). Licensed under CC-BY-4.0 (https://creativecommons.org/licenses/by/4.0/). Source:
  https://github.com/ITI/ICS-Security-Tools. No modifications made."

- Then `git ls-files tests/fixtures/committed-samples/` lists both pcap files and any
  attribution file; `git diff --stat` from a fresh clone shows them as tracked

**MUST NOT commit:**
- `iec104.pcap` (Wireshark Foundation "not redistributed" — test-file header line 25)
- `iec104-sq.pcapng` (Wireshark Foundation "not redistributed" — test-file header line 26)

**Size gate (hard — F-022):** If either file exceeds 100 KB at commit time (E2E-PCAPS.md
records 14 KB and 11 KB respectively — both well within limit), the implementer MUST NOT
commit the oversized file. Instead: truncate or re-derive the capture to meet the limit.
EC-004 does NOT authorize waiving the size constraint via a documented exception — the file
must be brought under the limit.

Verification:
```bash
git ls-files tests/fixtures/committed-samples/
# Must list both pcap files

wc -c tests/fixtures/committed-samples/iec104-iti-diverse.pcap
# Must be <= 102400 bytes (100 KB)

wc -c tests/fixtures/committed-samples/iec104-iti-dissect.pcap
# Must be <= 102400 bytes (100 KB)
```

### AC-182-003 (traces to PG-W85-005 — committed fixtures run on every cargo test invocation)

With `fixture_path()` as the shared resolver (AC-182-001), the committed captures are found
automatically and their corresponding tests run on every `cargo test` invocation — including CI.

- Given `fixture_path()` checks `COMMITTED_SAMPLES` before `LOCAL_SAMPLES`
- And `const COMMITTED_SAMPLES: &str = "tests/fixtures/committed-samples"` is added alongside
  the existing `const LOCAL_SAMPLES: &str = "tests/fixtures/local-samples"`
- When `iec104-iti-diverse.pcap` and `iec104-iti-dissect.pcap` are present in `committed-samples/`
  (as committed files, always available in any checkout)
- Then `fixture_present("iec104-iti-diverse.pcap")` returns `true` in a clean worktree,
  causing the corresponding test body to run all assertions (not return early)
- And `run_iec104_pipeline("iec104-iti-diverse.pcap")` successfully opens the file from the
  committed-samples path, panicking ONLY on actual read failure (not on absence)

Verification:
```bash
# On a clean worktree with only committed-samples populated:
cargo test --test iec104_e2e_real_pcaps_tests \
  test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu
# Must PASS (not silently skip): 66 findings, T0836×20 + T1692.001×46 (STORY-180 ground-truth)

cargo test --test iec104_e2e_real_pcaps_tests \
  test_e2e_BC_2_19_iec104_iti_dissect_T0814_T1692_001_control_coverage
# Must PASS: 11 findings, T0814×2 + T1692.001×9
```

### AC-182-004 (traces to PG-W85-005 — regression: clean-worktree observable outcome)

A clean-worktree run (only committed-samples present, local-samples absent) has a fully
observable, non-silent outcome:
(a) The two ITI committed-capture tests RUN and PASS (not silently skip)
(b) The two Wireshark fixture tests skip LOUDLY via stderr
(c) `test_fixture_manifest_report()` prints: `Fixture coverage: 2/4 fixtures present (2 fixture-gated tests will be skipped)`
(d) `test_fixture_manifest_report()` itself passes (never panics regardless of corpus state)

This AC specifies the combined observable outcome that guards against future regressions
of the PG-W85-005 class.

Verification:
```bash
cargo test --test iec104_e2e_real_pcaps_tests -- --nocapture 2>&1 | \
  grep -E "FIXTURE-SKIPPED|Fixture coverage"
# Must print:
#   Fixture coverage: 2/4 fixtures present (2 fixture-gated tests will be skipped)
#   FIXTURE-SKIPPED: 'iec104.pcap' absent — corpus test will not run ...
#   FIXTURE-SKIPPED: 'iec104-sq.pcapng' absent — corpus test will not run ...
```

### AC-182-005 (traces to PG-W85-005 — hard-assert: committed fixtures MUST be present)

A test that FAILS (not silently passes) when a committed/tracked fixture file is absent from
the repo. Committed fixture absence is a test infrastructure failure, not a corpus-availability
advisory.

- Given `iec104-iti-diverse.pcap` and `iec104-iti-dissect.pcap` are committed into
  `tests/fixtures/committed-samples/` (tracked by git, always present in any checkout)
- When `test_fixture_manifest_report()` is extended with a hard-assert partition:
  ```rust
  // Hard assert: committed (tracked) fixtures MUST be present.
  // If either is missing, that is a broken repo state — fail visibly.
  const COMMITTED_FIXTURES: &[&str] = &[
      "iec104-iti-diverse.pcap",
      "iec104-iti-dissect.pcap",
  ];
  for name in COMMITTED_FIXTURES {
      assert!(
          fixture_path(name).is_some(),
          "[iec104-e2e] REGRESSION: committed fixture '{}' is absent from \
           tests/fixtures/committed-samples/ — this is a broken checkout. \
           Run `git checkout tests/fixtures/committed-samples/` to restore.",
          name
      );
  }
  ```
- Then `test_fixture_manifest_report()` FAILS with a clear assertion message when either
  committed capture is absent, making the breakage visible in CI output at the test-failure
  level (not silently swallowed by stdout capture)

**Partition semantics:**
- Committed fixtures (`COMMITTED_FIXTURES` array): hard assert — absent → test FAILS (CI red)
- Gitignored corpus (iec104.pcap, iec104-sq.pcapng): advisory only — absent → stdout
  FIXTURE-SKIPPED notice, test still passes

This is the structural prevention for PG-W85-005: committed fixture absence is always a
visible failure; gitignored corpus absence is always a visible notice but never a failure.

Verification:
```bash
# On a fresh clone with committed captures present — must pass:
cargo test --test iec104_e2e_real_pcaps_tests test_fixture_manifest_report
# Must PASS

# Verify the hard-assert fires by temporarily renaming a committed capture:
# (manual test only — do not automate file removal)
# Expected: test_fixture_manifest_report fails with assertion message
```

## Architecture Mapping

| Component | File | Branch |
|-----------|------|--------|
| `fixture_path()` shared resolver + `COMMITTED_SAMPLES` const | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| `fixture_present()` updated to use `fixture_path()` | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| `run_iec104_pipeline()` updated to use `fixture_path()` | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| `FIXTURE_MANIFEST` const + `COMMITTED_FIXTURES` const | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| `test_fixture_manifest_report()` with hard-assert partition | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| Committed ITI capture (timed-command BC-2.19.029/030) | `tests/fixtures/committed-samples/iec104-iti-diverse.pcap` (new binary) | develop |
| Committed ITI capture (multi-flow dissector coverage) | `tests/fixtures/committed-samples/iec104-iti-dissect.pcap` (new binary) | develop |
| Provenance row + attribution | `tests/fixtures/README.md` (amend provenance table) | develop |

**No `src/` changes, no `bin/` changes, no `Cargo.toml` changes.**
CHANGELOG obligation: `tests/` is **excluded** from the AC-158-001 changelog-gate trigger set.
**No CHANGELOG entry required.**

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `tests/fixtures/committed-samples/` directory absent (freshly cloned repo before committed samples land) | `fixture_path()` returns None for all fixtures; `test_fixture_manifest_report()` FAILS on the hard-assert for committed captures (visible CI failure — correct behavior for broken checkout) |
| EC-002 | `iec104-iti-diverse.pcap` exists in both committed-samples and local-samples | `fixture_path()` returns committed-samples path (checked first); no double-run; safe |
| EC-003 | Wireshark captures (iec104.pcap, iec104-sq.pcapng) not present on clean checkout | Only 2/4 fixtures present; gitignored captures absent → stdout FIXTURE-SKIPPED notice; test still passes (gitignored partition) |
| EC-004 | Either ITI capture > 100 KB at commit time | Commit is BLOCKED — implementer must re-derive or truncate to meet the 100 KB limit. The size exception path is NOT available. |
| EC-005 | `test_fixture_manifest_report()` is expected to always pass in CI | True ONLY when committed fixtures are present (normal case after AC-182-002 delivery). If committed fixtures are absent due to broken checkout, the test FAILS — this is the intended regression guard |
| EC-006 | `fixture_path()` called before COMMITTED_SAMPLES directory exists | `committed.exists()` returns false; falls through to local-samples check; safe |

## Tasks

1. **Verify file sizes on a fixture-bearing host (AC-182-002 pre-commit gate):**
   ```bash
   wc -c tests/fixtures/local-samples/iec104-iti-diverse.pcap   # must be <= 102400
   wc -c tests/fixtures/local-samples/iec104-iti-dissect.pcap   # must be <= 102400
   ```
   E2E-PCAPS.md records both as 14 KB and 11 KB respectively — both comfortably below limit.
   If actual file exceeds 100 KB, do NOT commit — re-derive.

2. **Create committed-samples directory and add README provenance row (AC-182-002):**
   ```bash
   mkdir -p tests/fixtures/committed-samples/
   cp tests/fixtures/local-samples/iec104-iti-diverse.pcap tests/fixtures/committed-samples/
   cp tests/fixtures/local-samples/iec104-iti-dissect.pcap tests/fixtures/committed-samples/
   ```
   Add provenance rows to `tests/fixtures/README.md` (use the existing provenance table
   format at lines 30–34 as the model). Attribution: "ICS Security Tools, Illinois Institute
   of Technology (ITI). Licensed under CC-BY-4.0. Source:
   https://github.com/ITI/ICS-Security-Tools. No modifications made." Include per-file source
   URLs from E2E-PCAPS.md §IEC-104 Direct download URLs table.
   Optionally create `tests/fixtures/committed-samples/ATTRIBUTION.md` as a complementary
   file (not a replacement for the README.md provenance row).

3. **Add `COMMITTED_SAMPLES` const and `fixture_path()` function (AC-182-001/003):**
   ```rust
   const COMMITTED_SAMPLES: &str = "tests/fixtures/committed-samples";
   ```
   Add `fixture_path()` as described in AC-182-001. Use `std::path::PathBuf` as return type.

4. **Update `fixture_present()` to use `fixture_path()` (AC-182-001):**
   Replace the current LOCAL_SAMPLES-only logic with `fixture_path()` call. Preserve the
   existing `[iec104-e2e] SKIP:` eprintln format to stderr (do not change stderr diagnostic
   text unnecessarily — this is the SKIP notice format that existing observers may monitor).
   Specifically keep `[iec104-e2e] SKIP:` prefix.

5. **Update `run_iec104_pipeline()` to use `fixture_path()` (AC-182-001/AC-182-003):**
   Replace the `LOCAL_SAMPLES` hardcode with a `fixture_path()` call that panics only on
   None (broken pre-condition: caller must have verified presence). The error message must
   make clear this is a pre-condition violation, not a file-absent skip condition.

6. **Add `FIXTURE_MANIFEST`, `COMMITTED_FIXTURES` constants and `test_fixture_manifest_report()`
   (AC-182-001/005):** Add the 4-entry manifest, the 2-entry committed fixtures array, and
   the combined manifest-report + hard-assert test function. The test MUST:
   (a) Print the coverage summary to stdout (visible with --nocapture)
   (b) Print per-absent-fixture FIXTURE-SKIPPED lines to stdout
   (c) Hard-assert on each `COMMITTED_FIXTURES` entry (test fails if any committed fixture absent)

7. **Sibling-prose sweep (F-017):**
   - `tests/iec104_e2e_real_pcaps_tests.rs` module docstring (lines 10–13): update "Captures
     live in `tests/fixtures/local-samples/`" to acknowledge `committed-samples/` alongside
     `local-samples/`.
   - `run_iec104_pipeline` doc-comment contract (line ~90): update "The file must exist under
     `tests/fixtures/local-samples/`" to reflect the shared `fixture_path()` resolver.
   - `tests/fixtures/E2E-PCAPS.md` §IEC-104: note that both ITI captures are now committed
     under `tests/fixtures/committed-samples/` (add a sentence to the IEC-104 section
     explaining the committed-vs-local-samples distinction for these files).
   - `tests/fixtures/local-samples/README.md`: note that ITI IEC-104 captures are now also
     available as committed samples in `tests/fixtures/committed-samples/`.

8. **Regression verification (AC-182-004/005):**
   ```bash
   cargo test --test iec104_e2e_real_pcaps_tests -- --nocapture 2>&1 | \
     grep -E "FIXTURE-SKIPPED|Fixture coverage"
   # Must show: "Fixture coverage: 2/4 fixtures present (2 fixture-gated tests will be skipped)"
   # and FIXTURE-SKIPPED lines for iec104.pcap and iec104-sq.pcapng

   cargo test --test iec104_e2e_real_pcaps_tests \
     test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu
   # Must PASS with 66 findings (not silently skip)

   cargo test --test iec104_e2e_real_pcaps_tests \
     test_e2e_BC_2_19_iec104_iti_dissect_T0814_T1692_001_control_coverage
   # Must PASS with 11 findings (not silently skip)
   ```

9. **Develop PR:** All changes in `tests/` and committed binary fixtures — no CHANGELOG
   required. The PR description MUST include the actual test output proving the 2/4 fixture
   coverage summary and the two ITI test passes.

> **Note for implementer:** Task 1 MUST run on the fixture-bearing host before committing.
> Size check (≤100 KB) is a hard gate per AC-182-002 and EC-004 — no size exception path
> exists. The two Wireshark Foundation samples (iec104.pcap, iec104-sq.pcapng) MUST NOT
> be committed (see Background §Must NOT commit).

## Previous Story Intelligence

- **STORY-176 (wave-84):** Established E-11 governance pattern: no BCs, no Rust source
  changes, `tdd_mode: strict`, CHANGELOG only when `bin/` is touched. STORY-182 follows
  the same convention (tests/ changes only, no CHANGELOG).
- **STORY-180 (wave-85, BC-2.19.029/030):** Delivered timed-command detection for TypeIDs
  58–64. Updated `test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu` to expect
  66 findings (was 31). The committed `iec104-iti-diverse.pcap` in AC-182-002 must produce
  this wave-85-updated expectation: T0836×20 + T1692.001×46 = 66.
- **Gate-fix PR #439 (0ab6f52e, wave-85):** Updated ITI diverse expectations from 31→66
  after gate G1 FAIL. STORY-182 closes the structural gap that made gate G1 fail silently.

## Architecture Compliance Rules

- **No `src/` modification:** Test-infrastructure only. No production code changes. If any
  `src/` file needs changing, stop and escalate.
- **Shared resolver contract:** `fixture_path()` is the SINGLE path-resolution authority.
  `fixture_present()` and `run_iec104_pipeline()` both delegate to it. No independent path
  construction from `LOCAL_SAMPLES` or `COMMITTED_SAMPLES` is permitted in either function
  after this story lands.
- **`COMMITTED_SAMPLES` path (`tests/fixtures/committed-samples/`):** NOT covered by any
  `.gitignore` entry. No `.gitignore` amendment required.
- **`test_fixture_manifest_report()` pass/fail contract:**
  - Committed fixtures absent → test FAILS (visible CI failure — regression guard)
  - Gitignored corpus absent → advisory stdout only — test still passes
- **Stdout for skip notices:** `test_fixture_manifest_report()` uses `println!()` for
  coverage summary and FIXTURE-SKIPPED lines (visible with `--nocapture`). `fixture_present()`
  preserves the existing `eprintln!("[iec104-e2e] SKIP: …")` stderr diagnostic for per-test
  early-exit notices.
- **Test signature — `test_fixture_manifest_report()` returns `()`** (unit function). The
  function never returns `Ok(())`. It panics on the hard-assert if committed fixtures are
  absent; it completes normally (returns `()`) otherwise. The word "returns" in the docstring
  should be "never panics" (when fixtures present), not "always returns Ok(())".
- **Action SHA-pin policy (CI):** STORY-182 makes no `ci.yml` changes; no new SHA pins
  required.

## Library & Framework Requirements

| Dependency | Version | Source |
|------------|---------|--------|
| Rust stable | 1.91+ | CLAUDE.md MSRV |
| No new Cargo.toml deps | — | tests/ only, no new crate dependencies |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `tests/iec104_e2e_real_pcaps_tests.rs` | Modify | Add `COMMITTED_SAMPLES` const, `fixture_path()` fn, update `fixture_present()` and `run_iec104_pipeline()`, add `FIXTURE_MANIFEST`/`COMMITTED_FIXTURES` consts, add `test_fixture_manifest_report()`, update module docstring + pipeline doc-comment |
| `tests/fixtures/committed-samples/iec104-iti-diverse.pcap` | New (binary) | ITI CC-BY-4.0; ≤100 KB; exercises TypeIDs 58–64 |
| `tests/fixtures/committed-samples/iec104-iti-dissect.pcap` | New (binary) | ITI CC-BY-4.0; ≤100 KB; exercises 6-flow dissector path |
| `tests/fixtures/README.md` | Modify | Add IEC-104 committed-captures provenance rows (licensing notice + source URLs + attribution) |
| `tests/fixtures/E2E-PCAPS.md` | Modify | Note that ITI IEC-104 captures are now also committed under committed-samples/ |
| `tests/fixtures/local-samples/README.md` | Modify | Note that ITI IEC-104 captures are also available as committed samples |

**Forbidden modifications:** `src/**/*`, `Cargo.toml`, `bin/*`, `CHANGELOG.md`,
`.github/workflows/ci.yml`

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `tests/iec104_e2e_real_pcaps_tests.rs` | effectful-shell | Filesystem I/O (pcap reads, path checks), analyzer pipeline execution, assertion outputs. |
| `tests/fixtures/committed-samples/` | data artifact | Static binary captures; not a code module. |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~5.0 k |
| `tests/iec104_e2e_real_pcaps_tests.rs` (full file, ~654 lines) | ~5.0 k |
| `tests/fixtures/README.md` (provenance table section) | ~1.0 k |
| `tests/fixtures/E2E-PCAPS.md` (IEC-104 section) | ~1.0 k |
| Binary pcap files (not token-counted) | ~0 k |
| **Total** | **~12.0 k** |
| Agent context window | 200 k (Sonnet) |
| **Budget usage** | **~6%** |

Well within context window. No story split required.

## Notes

- **DF-VALIDATION-001 status:** PG-W85-005 is LOCAL-CARRY-FORWARD per
  `df-validation-2026-07-25.md §PG-W85-005` (HIGH confidence). No upstream filing required.
- **No behavioral contract required:** E-11 convention.
- **Develop PR:** All four+one ACs can be batched in a single develop PR. No CHANGELOG entry
  required (`tests/` and `tests/fixtures/` are not in the AC-158-001 trigger set).
- **Wave-85 gate G1 retrospective:** The initial FAIL (D-510) that motivated PG-W85-005 was
  caused by the ITI diverse test silently skipping in the evaluator's clean worktree while
  passing on the fixture host. Gate-fix PR #439 updated expectations from 31→66. STORY-182
  closes the structural gap so this class of false-green can no longer occur silently.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-07-25 | story-writer | WAVE-86 PASS-1 REMEDIATION — F-011 CRIT (AC-182-003 complete redesign: `fixture_path(name) -> Option<PathBuf>` shared resolver used by BOTH `fixture_present()` and `run_iec104_pipeline()`; the pipeline fn no longer hardcodes LOCAL_SAMPLES or panics on absent committed fixture); F-012 HIGH (union approach (b)+(a)+(c): manifest + loud skip-reporting + hard-assert partition + committed samples; AC-182-005 new hard-assert test added); F-013 HIGH (committed fixtures absent → test FAILS visible; gitignored absent → advisory stdout only; AC-182-005 formalises the partition); F-014 HIGH (resolved by F-012/F-013 hard-assert partition); F-015 MED (unified skip-notice: single `[iec104-e2e] SKIP:` stderr format in `fixture_present()`; stdout FIXTURE-SKIPPED in manifest-report test); F-016 MED (attribution via tests/fixtures/README.md provenance table; ATTRIBUTION.md demoted to complementary); F-017 MED (sibling-prose sweep tasks: module docstring 10-13, pipeline contract line ~90, E2E-PCAPS.md, local-samples/README.md — Task 7); F-018 MED (test_fixture_manifest_report() returns unit `()`; "Always returns Ok(())" removed; "never panics" is the correct description when committed fixtures present); F-021 MED (iec104-iti-dissect.pcap made MANDATORY; both ITI captures committed; all 2/4 count literals reconciled); F-022 LOW (≤100 KB gate hard: no exception path; EC-004 rewritten to require re-derive/truncation if oversized); F-020 MED (inputs: set to [wave-085/lessons.md, df-validation-2026-07-25.md]); F-023 LOW (level already maintenance — confirmed). |
| 1.0 | 2026-07-25 | story-writer | Initial authorship — wave-86 STORY-CREATION BURST (D-516). |
