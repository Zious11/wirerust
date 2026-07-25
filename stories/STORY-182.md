---
document_type: story
story_id: STORY-182
epic_id: E-11
version: "1.2"
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
  - tests/fixtures/
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
- **So that** the wave-85 gate G1 initial FAIL (D-510, PG-W85-005) is structurally prevented:
  a test asserting 31 findings when 66 is now the correct expectation (stale assertion on a
  fixture-bearing host) cannot recur, absence of committed captures is always a visible CI
  failure, and the timed-command code path always runs in CI

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

The following EXISTING governance surfaces apply to committed fixtures:
- `tests/fixtures/README.md` (lines 5–34): licensing notice + provenance table. ALL committed
  fixtures must have a row added to this table with source URL, modification indication, and
  license. This is the authoritative attribution record — not a separate `ATTRIBUTION.md`.
- Lines 7–22 of `tests/fixtures/README.md` contain the licensing notice. This sweep must be
  updated to describe the CC-BY-4.0 class for ITI captures when provenance rows are added.
- Malware-content check: committed captures must not contain live malware C2 traffic or
  exploit payloads (README.md §Licensing notice).
- CC-BY-4.0 attribution for ITI captures: "ICS Security Tools, Illinois Institute of
  Technology (ITI). Licensed under CC-BY-4.0." Per-file: source URL, modification note,
  upstream license notice retention. Attribution is already recorded in E2E-PCAPS.md §IEC-104;
  the README.md provenance table row is the additional required surface.

### Committed captures decision — files go directly in `tests/fixtures/`

Both ITI CC-BY-4.0 captures are MANDATORY committed fixtures. They go **directly in
`tests/fixtures/`** — the same directory as the 21 existing committed captures — NOT in a
new subdirectory. `COMMITTED_SAMPLES` const is set to `"tests/fixtures"`. The existing
convention (21 tracked captures in `tests/fixtures/`) is the authoritative precedent.

- `iec104-iti-diverse.pcap`: 14 KB (< 100 KB), ITI CC-BY-4.0. Exercises TypeIDs 58–64
  (timed control commands, BC-2.19.029/030). Produces 66 findings with STORY-180 detection.
- `iec104-iti-dissect.pcap`: 11 KB (< 100 KB), ITI CC-BY-4.0. Exercises 6-flow dissector
  path. Produces 11 findings (T0814×2 + T1692.001×9).

Both are committed. The FIXTURE_MANIFEST contains 4 entries. In a clean checkout (only
`tests/fixtures/` present), the manifest-report shows **2/4 fixtures present** (the two
Wireshark samples, iec104.pcap and iec104-sq.pcapng, are NOT committed — "not redistributed";
they live in gitignored `tests/fixtures/local-samples/`).

### Must NOT commit

- `iec104.pcap`: Wireshark Foundation "not redistributed" — see `tests/iec104_e2e_real_pcaps_tests.rs`
  licensing prose at line 138 and `tests/fixtures/E2E-PCAPS.md`.
- `iec104-sq.pcapng`: Wireshark Foundation "not redistributed" — see test-file line 273 and E2E-PCAPS.md.

### Gitignore placement

`tests/fixtures/` (root) is NOT covered by any `.gitignore` entry. `.gitignore` covers only
`/tests/fixtures/local-samples/`. Since committed captures go directly in `tests/fixtures/`,
no `.gitignore` change is required — these files are naturally tracked.

### Stdout vs. CI visibility — `println!()` requires `--nocapture`

`println!()` output in a passing test is **captured by libtest** and NOT displayed unless
`--nocapture` is passed. CI runs `cargo test --all-targets` (`.github/workflows/ci.yml` line 47)
with no `--nocapture` flag. Therefore:
- `test_fixture_manifest_report()` `println!()` lines are visible LOCALLY when using
  `cargo test -- --nocapture`, but are **NOT visible in standard CI output**.
- The **hard-assert panic** (AC-182-005) in `test_fixture_manifest_report()` IS always
  visible in CI because panics produce test-failure output regardless of capture mode.
- `fixture_present()` uses `eprintln!()` (stderr), which libtest also captures. Skip notices
  are visible with `--nocapture` or via `2>&1` redirect.

This partition is intentional: the manifest-report function uses `println!()` for informational
coverage summaries (available for local debugging with `--nocapture`) and relies on the
hard-assert panic for CI-visible failure detection.

**`#[ignore]` rejection:** `#[ignore]` was considered and rejected. `#[ignore]` tests are
excluded from `cargo test` output entirely (only visible with `--ignored`), making committed
fixture absence completely invisible in CI. The hard-assert inside a regular `#[test]`
function is the correct mechanism: it produces a visible test-failure when committed fixtures
are absent.

### Fixture count anchors

All count literals in this story use the MANDATORY decision: 2 committed captures (both ITI),
4 total manifest entries. The canonical manifest-report string in a clean checkout is:
`Fixture coverage: 2/4 fixtures present (2 fixture-gated tests will be skipped)`.

## Acceptance Criteria

### AC-182-001 (traces to PG-W85-005 — shared resolver + fixture manifest with skip reporting)

A shared path resolver `fixture_path(name) -> Option<PathBuf>` is introduced and used by
BOTH `fixture_present()` and `run_iec104_pipeline()`. A manifest mechanism reports fixture
coverage on every run with `--nocapture`.

- Given `fixture_present()` (line 63) currently hardcodes `LOCAL_SAMPLES` and returns false
  (with eprintln to stderr) when absent; and `run_iec104_pipeline()` (line 97) independently
  hardcodes `LOCAL_SAMPLES` and panics on open failure
- When a new helper function is added:
  ```rust
  /// Resolve the path of a fixture file, checking tests/fixtures/ (committed) before
  /// tests/fixtures/local-samples/ (gitignored corpus).
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
- And `fixture_present()` is updated to use `fixture_path()`, preserving the existing
  `[iec104-e2e] SKIP:` prefix and including the resolved path in the diagnostic:
  ```rust
  fn fixture_present(filename: &str) -> bool {
      match fixture_path(filename) {
          Some(_) => true,
          None => {
              let committed_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                  .join(COMMITTED_SAMPLES).join(filename);
              eprintln!(
                  "[iec104-e2e] SKIP: fixture '{}' not found at {} (or in local-samples). \
                   Run `bin/fetch-e2e-pcaps` to populate local-samples.",
                  filename, committed_path.display()
              );
              false
          }
      }
  }
  ```
- And `run_iec104_pipeline()` is updated to use `fixture_path()` and panics with a clear
  message if the file is absent:
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
- And a new `FIXTURE_MANIFEST: &[&str]` constant is added at **module level** listing all 4
  expected fixture filenames:
  `["iec104.pcap", "iec104-sq.pcapng", "iec104-iti-diverse.pcap", "iec104-iti-dissect.pcap"]`
- And a new `COMMITTED_FIXTURES: &[&str]` constant is added at **module level** (alongside
  `FIXTURE_MANIFEST`, not inside the test function):
  `["iec104-iti-diverse.pcap", "iec104-iti-dissect.pcap"]`
- And a new `#[test]` function `test_fixture_manifest_report()` is added (hard-assert
  partition detailed in AC-182-005; skip-reporting half covered here):
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
      // Advisory stdout: visible with --nocapture only; not visible in standard CI output
      println!(
          "Fixture coverage: {}/{} fixtures present ({} fixture-gated tests will be skipped)",
          present.len(), FIXTURE_MANIFEST.len(), absent.len()
      );
      for name in &absent {
          println!(
              "FIXTURE-SKIPPED: '{}' absent — corpus test will not run \
               (check tests/fixtures/ for committed or tests/fixtures/local-samples/ for corpus)",
              name
          );
      }
      // Hard-assert partition: committed/tracked fixtures MUST be present — see AC-182-005
      // Gitignored corpus absence is advisory only — no panic (see AC-182-005 partition)
  }
  ```
- And `test_fixture_manifest_report()` includes a manifest-count sanity assertion:
  ```rust
  assert_eq!(
      FIXTURE_MANIFEST.len(), 4,
      "FIXTURE_MANIFEST must have exactly 4 entries — update if a new fixture is committed"
  );
  assert_eq!(
      COMMITTED_FIXTURES.len(), 2,
      "COMMITTED_FIXTURES must have exactly 2 entries — update if a new committed fixture is added"
  );
  ```

- Then `cargo test --test iec104_e2e_real_pcaps_tests test_fixture_manifest_report -- --nocapture`
  in a clean worktree (both committed captures present) prints:
  `Fixture coverage: 2/4 fixtures present (2 fixture-gated tests will be skipped)`
- And `cargo test --test iec104_e2e_real_pcaps_tests test_fixture_manifest_report`
  (WITHOUT --nocapture, as in CI) PASSES silently when committed fixtures are present

Verification:
```bash
# Local: stdout visible with --nocapture
cargo test --test iec104_e2e_real_pcaps_tests test_fixture_manifest_report -- --nocapture
# Must print "Fixture coverage: 2/4 fixtures present" to stdout

# CI equivalent: no --nocapture; test must still pass (stdout not shown)
cargo test --test iec104_e2e_real_pcaps_tests test_fixture_manifest_report
# Must exit 0 (passes silently when committed fixtures present)
```

### AC-182-002 (traces to PG-W85-005 — committed mandatory captures)

Both ITI CC-BY-4.0 representative captures are committed directly under `tests/fixtures/`
and their provenance is recorded in `tests/fixtures/README.md`.

- Given `iec104-iti-diverse.pcap` (14 KB, 173 packets, ITI CC-BY-4.0) exercises TypeIDs 58–64
  and is the ground-truth capture for STORY-180's timed-command detection (66 findings); and
  `iec104-iti-dissect.pcap` (11 KB, 147 packets, ITI CC-BY-4.0) exercises 6 flows with
  TypeIDs C_SC/C_DC/C_SE (11 findings: T0814×2 + T1692.001×9) — both licensed CC-BY-4.0
  permitting redistribution with attribution
- When the implementer verifies on a fixture-bearing host that BOTH files are ≤ 100 KB
  (E2E-PCAPS.md §IEC-104 records: diverse=14 KB, dissect=11 KB — both well within the limit)
- And copies BOTH files **directly to `tests/fixtures/`** (alongside existing committed captures)
- And adds a provenance row to `tests/fixtures/README.md` in the existing "Fixtures with
  recorded provenance" table (lines 30–34), or adds a new IEC-104 subsection following the
  established table format, including per-file direct download URLs:
  ```
  | `iec104-iti-diverse.pcap` | [090813_diverse.pcap](https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/IEC60870-5-104/090813_diverse.pcap) from [ITI/ICS-Security-Tools](https://github.com/ITI/ICS-Security-Tools/tree/master/pcaps/IEC60870-5-104) | IEC-104; 173 packets; 14 KB; CC-BY-4.0; exercises TypeIDs 58–64 timed control commands (BC-2.19.029/030); produces 66 findings (STORY-180 ground-truth). |
  | `iec104-iti-dissect.pcap` | [TestDissectIec104.pcap](https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/IEC60870-5-104/TestDissectIec104.pcap) from [ITI/ICS-Security-Tools](https://github.com/ITI/ICS-Security-Tools/tree/master/pcaps/IEC60870-5-104) | IEC-104; 147 packets; 11 KB; CC-BY-4.0; 6-flow dissector exercise; produces 11 findings (T0814×2 + T1692.001×9). |
  ```
  Attribution text: "ICS Security Tools, Illinois Institute of Technology (ITI). Licensed
  under CC-BY-4.0 (https://creativecommons.org/licenses/by/4.0/). Source:
  https://github.com/ITI/ICS-Security-Tools. No modifications made."

- Then `git ls-files tests/fixtures/` lists both pcap files as tracked; `git diff --stat`
  from a fresh clone shows them as new tracked files

**MUST NOT commit:**
- `iec104.pcap` (Wireshark Foundation "not redistributed" — test-file line 138)
- `iec104-sq.pcapng` (Wireshark Foundation "not redistributed" — test-file line 273)

**Size gate (hard):** If either file exceeds 100 KB at commit time (E2E-PCAPS.md records 14 KB
and 11 KB respectively — both well within limit), the implementer MUST NOT commit the oversized
file. Instead: truncate or re-derive the capture to meet the limit. The size constraint is not
waivable via a documented exception — the file must be brought under the limit.

Verification:
```bash
git ls-files tests/fixtures/iec104-iti-diverse.pcap tests/fixtures/iec104-iti-dissect.pcap
# Must list both files as tracked

wc -c tests/fixtures/iec104-iti-diverse.pcap
# Must be <= 102400 bytes (100 KB)

wc -c tests/fixtures/iec104-iti-dissect.pcap
# Must be <= 102400 bytes (100 KB)
```

### AC-182-003 (traces to PG-W85-005 — committed fixtures run on every cargo test invocation)

With `fixture_path()` as the shared resolver (AC-182-001), the committed captures are found
automatically and their corresponding tests run on every `cargo test` invocation — including CI.
Committed fixtures never trigger the skip path.

- Given `fixture_path()` checks `COMMITTED_SAMPLES` (= `"tests/fixtures"`) before `LOCAL_SAMPLES`
- And `const COMMITTED_SAMPLES: &str = "tests/fixtures"` is added alongside the existing
  `const LOCAL_SAMPLES: &str = "tests/fixtures/local-samples"`
- When `iec104-iti-diverse.pcap` and `iec104-iti-dissect.pcap` are present in `tests/fixtures/`
  (as committed files, always available in any checkout)
- Then `fixture_present("iec104-iti-diverse.pcap")` returns `true` in a clean worktree,
  causing the corresponding test body to run all assertions (not return early)
- And `run_iec104_pipeline("iec104-iti-diverse.pcap")` successfully opens the file from the
  committed `tests/fixtures/` path
- And `fixture_present("iec104-iti-diverse.pcap")` NEVER prints the `[iec104-e2e] SKIP:` line
  (committed fixture is always found in `tests/fixtures/` — no skip path is taken)

Verification:
```bash
# On a clean worktree with only tests/fixtures/ committed captures populated:
cargo test --test iec104_e2e_real_pcaps_tests \
  test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu
# Must PASS (not silently skip): 66 findings, T0836×20 + T1692.001×46

cargo test --test iec104_e2e_real_pcaps_tests \
  test_e2e_BC_2_19_iec104_iti_dissect_T0814_T1692_001_control_coverage
# Must PASS: 11 findings, T0814×2 + T1692.001×9
```

### AC-182-004 (traces to PG-W85-005 — regression: clean-worktree observable outcome)

A clean-worktree run (only `tests/fixtures/` committed captures present, local-samples absent)
has a fully observable, non-silent outcome:
(a) The two ITI committed-capture tests RUN and PASS (not silently skip)
(b) The two Wireshark fixture tests skip via the existing `fixture_present()` stderr path
    (visible with `--nocapture 2>&1`; not visible in standard CI output without it)
(c) `test_fixture_manifest_report()` itself PASSES (never panics when committed fixtures present)
(d) `test_fixture_manifest_report()` FAILS with a visible assertion message if a committed
    fixture is absent (CI-visible because panics always appear in test output regardless of
    capture mode — see AC-182-005)

**Stdout advisory note (F-003):** The `Fixture coverage: 2/4 ...` and `FIXTURE-SKIPPED:` lines
are printed via `println!()` and are ONLY visible when `--nocapture` is passed. Standard CI
(`cargo test --all-targets` in ci.yml line 47) does NOT use `--nocapture`. The coverage summary
is useful for local debugging but is NOT a CI-visible signal. The CI-visible signals are:
- Test PASS/FAIL counts (always reported by libtest)
- Hard-assert panics in `test_fixture_manifest_report()` (AC-182-005)
- `[iec104-e2e] SKIP:` eprintln() in `fixture_present()` — also captured; visible with `--nocapture`

Verification:
```bash
# Local debugging (--nocapture required to see stdout/stderr):
cargo test --test iec104_e2e_real_pcaps_tests -- --nocapture 2>&1 | \
  grep -E "FIXTURE-SKIPPED|Fixture coverage"
# Must print:
#   Fixture coverage: 2/4 fixtures present (2 fixture-gated tests will be skipped)
#   FIXTURE-SKIPPED: 'iec104.pcap' absent ...
#   FIXTURE-SKIPPED: 'iec104-sq.pcapng' absent ...

# CI-mode verification (no --nocapture):
cargo test --test iec104_e2e_real_pcaps_tests
# Must exit 0; both ITI tests PASS; manifest test PASSES silently
```

### AC-182-005 (traces to PG-W85-005 — hard-assert: committed fixtures MUST be present)

A test that FAILS (not silently passes) when a committed/tracked fixture file is absent from
the repo. Both `FIXTURE_MANIFEST` and `COMMITTED_FIXTURES` are declared at module level.

- Given `iec104-iti-diverse.pcap` and `iec104-iti-dissect.pcap` are committed into
  `tests/fixtures/` (tracked by git, always present in any checkout)
- And `COMMITTED_FIXTURES` is declared at **module level** alongside `FIXTURE_MANIFEST`:
  ```rust
  const FIXTURE_MANIFEST: &[&str] = &[
      "iec104.pcap",
      "iec104-sq.pcapng",
      "iec104-iti-diverse.pcap",
      "iec104-iti-dissect.pcap",
  ];

  const COMMITTED_FIXTURES: &[&str] = &[
      "iec104-iti-diverse.pcap",
      "iec104-iti-dissect.pcap",
  ];
  ```
- When `test_fixture_manifest_report()` is extended with a hard-assert partition:
  ```rust
  // Hard assert: committed (tracked) fixtures MUST be present.
  // If either is missing, that is a broken repo state — fail visibly in CI.
  // This panic IS always visible in CI output regardless of --nocapture (assertion failure).
  for name in COMMITTED_FIXTURES {
      assert!(
          fixture_path(name).is_some(),
          "[iec104-e2e] REGRESSION: committed fixture '{}' is absent from \
           tests/fixtures/ — this is a broken checkout. \
           Run `git checkout tests/fixtures/` to restore.",
          name
      );
  }
  ```
- Then `test_fixture_manifest_report()` FAILS with a clear assertion message when either
  committed capture is absent — the panic text appears in CI test failure output (always
  visible, regardless of capture mode)

**Partition semantics:**
- Committed fixtures (`COMMITTED_FIXTURES` array, tracked in `tests/fixtures/`):
  hard assert — absent → test FAILS (CI red — visible because panics bypass stdout capture)
- Gitignored corpus (iec104.pcap, iec104-sq.pcapng in `tests/fixtures/local-samples/`):
  advisory only — absent → stdout FIXTURE-SKIPPED notice (visible only with --nocapture),
  test still passes

This is the structural prevention for PG-W85-005: committed fixture absence is always a
visible CI failure (panic); gitignored corpus absence is an advisory notice only.

Verification:
```bash
# On a fresh clone with committed captures present — must pass:
cargo test --test iec104_e2e_real_pcaps_tests test_fixture_manifest_report
# Must PASS

# Verify the hard-assert fires by temporarily renaming a committed capture:
# (manual test only — do not automate file removal in CI)
# Expected: test_fixture_manifest_report fails with assertion message
```

## Architecture Mapping

| Component | File | Branch |
|-----------|------|--------|
| `fixture_path()` shared resolver + `COMMITTED_SAMPLES` const | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| `fixture_present()` updated to use `fixture_path()` | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| `run_iec104_pipeline()` updated to use `fixture_path()` | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| `FIXTURE_MANIFEST` const (module-level) + `COMMITTED_FIXTURES` const (module-level) | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| `test_fixture_manifest_report()` with hard-assert partition | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| Committed ITI capture (timed-command BC-2.19.029/030) | `tests/fixtures/iec104-iti-diverse.pcap` (new binary) | develop |
| Committed ITI capture (multi-flow dissector coverage) | `tests/fixtures/iec104-iti-dissect.pcap` (new binary) | develop |
| Provenance row + attribution + CC-BY-4.0 licensing sweep | `tests/fixtures/README.md` (amend) | develop |

**No `src/` changes, no `bin/` changes, no `Cargo.toml` changes.**
CHANGELOG obligation: `tests/` is **excluded** from the AC-158-001 changelog-gate trigger set.
**No CHANGELOG entry required.**

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `tests/fixtures/iec104-iti-diverse.pcap` absent (broken checkout, git LFS not pulled, etc.) | `fixture_path()` returns None; `test_fixture_manifest_report()` FAILS on the hard-assert with visible CI error — correct behavior for broken checkout |
| EC-002 | `iec104-iti-diverse.pcap` exists in both `tests/fixtures/` and `tests/fixtures/local-samples/` | `fixture_path()` returns `tests/fixtures/` path (checked first); no double-run; safe |
| EC-003 | Wireshark captures (iec104.pcap, iec104-sq.pcapng) not present in clean checkout | Only 2/4 fixtures present; gitignored captures absent → stdout FIXTURE-SKIPPED notice (visible with --nocapture); test still passes (gitignored partition) |
| EC-004 | Either ITI capture > 100 KB at commit time | Commit is BLOCKED — implementer must re-derive or truncate to meet the 100 KB limit. The size exception path is NOT available. |
| EC-005 | `test_fixture_manifest_report()` run in CI without --nocapture | Passes silently when committed fixtures are present. Hard-assert panic (committed fixture absent) IS visible in CI regardless of capture mode. Advisory println!() lines are NOT visible — this is expected behavior. |
| EC-006 | `fixture_path()` called for a file not in COMMITTED_SAMPLES or LOCAL_SAMPLES | Returns None; `fixture_present()` prints `[iec104-e2e] SKIP:` to stderr; calling test returns early (passes silently) |
| EC-007 | Stale-assertion D-510 class: test on fixture-bearing host but with wrong expected count | Hard to prevent structurally; STORY-180 gate-fix PR #439 updated 31→66. COMMITTED_FIXTURES hard-assert prevents ABSENCE; it does not prevent wrong fixture content. Expectation updates are the implementer's obligation after fixture content changes. |

## Tasks

1. **Verify file sizes on a fixture-bearing host (AC-182-002 pre-commit gate):**
   ```bash
   wc -c tests/fixtures/local-samples/iec104-iti-diverse.pcap   # must be <= 102400
   wc -c tests/fixtures/local-samples/iec104-iti-dissect.pcap   # must be <= 102400
   ```
   E2E-PCAPS.md records both as 14 KB and 11 KB respectively — both comfortably below limit.
   If actual file exceeds 100 KB, do NOT commit — re-derive.

2. **Copy committed captures to `tests/fixtures/` and add README provenance rows (AC-182-002):**
   ```bash
   cp tests/fixtures/local-samples/iec104-iti-diverse.pcap tests/fixtures/
   cp tests/fixtures/local-samples/iec104-iti-dissect.pcap tests/fixtures/
   ```
   Add provenance rows to `tests/fixtures/README.md` using the existing provenance table
   format at lines 30–34 as the model. Include per-file direct download URLs (from E2E-PCAPS.md
   §IEC-104 Direct download URLs table):
   - `iec104-iti-diverse.pcap`: upstream `090813_diverse.pcap` at
     `https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/IEC60870-5-104/090813_diverse.pcap`
   - `iec104-iti-dissect.pcap`: upstream `TestDissectIec104.pcap` at
     `https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/IEC60870-5-104/TestDissectIec104.pcap`
   Attribution: "ICS Security Tools, Illinois Institute of Technology (ITI). Licensed under
   CC-BY-4.0. Source: https://github.com/ITI/ICS-Security-Tools. No modifications made."

3. **Add `COMMITTED_SAMPLES` const and `fixture_path()` function (AC-182-001/003):**
   ```rust
   const COMMITTED_SAMPLES: &str = "tests/fixtures";
   ```
   Add `fixture_path()` as described in AC-182-001. Return type: `Option<std::path::PathBuf>`.

4. **Update `fixture_present()` to use `fixture_path()` (AC-182-001):**
   Replace the current LOCAL_SAMPLES-only logic with `fixture_path()` call. Preserve the
   existing `[iec104-e2e] SKIP:` eprintln format to stderr (maintain the prefix exactly —
   existing observers may monitor this pattern). Include path display in the message.

5. **Update `run_iec104_pipeline()` to use `fixture_path()` (AC-182-001/AC-182-003):**
   Replace the `LOCAL_SAMPLES` hardcode with a `fixture_path()` call that panics only on
   None (broken pre-condition: caller must have verified presence). The error message must
   make clear this is a pre-condition violation, not a file-absent skip condition.

6. **Add module-level `FIXTURE_MANIFEST` and `COMMITTED_FIXTURES` constants, add
   `test_fixture_manifest_report()` (AC-182-001/005):** Both constants go at module level
   (NOT inside the test function). Add the combined manifest-report + hard-assert test function.
   The test MUST:
   (a) Print the coverage summary to stdout (visible with --nocapture; advisory)
   (b) Print per-absent-fixture FIXTURE-SKIPPED lines to stdout (visible with --nocapture; advisory)
   (c) Hard-assert on each `COMMITTED_FIXTURES` entry (test FAILS in CI if any committed fixture absent)
   (d) Assert `FIXTURE_MANIFEST.len() == 4` and `COMMITTED_FIXTURES.len() == 2` (manifest-count sanity)

7. **Sibling-prose sweep:**
   - `tests/fixtures/README.md` lines 7–22 (licensing notice): sweep for the CC-BY-4.0 class.
     Update to describe the ITI CC-BY-4.0 class of committed captures in addition to the
     existing Wireshark Foundation de-facto license. Keep licensing notice accurate and complete.
   - `tests/iec104_e2e_real_pcaps_tests.rs` module docstring (lines 10–13): update "Captures
     live in `tests/fixtures/local-samples/`" to acknowledge `tests/fixtures/` (committed) and
     `tests/fixtures/local-samples/` (gitignored corpus).
   - `run_iec104_pipeline` doc-comment contract (line ~90): update "The file must exist under
     `tests/fixtures/local-samples/`" to reflect the shared `fixture_path()` resolver.
   - `tests/fixtures/E2E-PCAPS.md` §IEC-104: note that both ITI captures are now committed
     under `tests/fixtures/` (add a sentence explaining the committed-vs-local-samples
     distinction for these files).

8. **Create gate-entry artifact (AC-182-005 gate-entry evidence):**
   Create `.factory/maintenance/fixture-count-gate-entry.md` documenting:
   - The FIXTURE_MANIFEST count (4 entries, 2 committed)
   - The COMMITTED_FIXTURES members
   - The `#[ignore]` rejection rationale (invisible in CI; hard-assert is the correct mechanism)
   - The D-510 gate G1 FAIL retrospective: initial FAIL was on a fixture-bearing host with a
     stale 31-finding assertion (the correct count was 66 after STORY-180); gate-fix PR #439
     updated the expectation. STORY-182 closes the structural gap so committed fixture absence
     is always a CI-visible failure.

9. **Regression verification (AC-182-004/005):**
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

10. **Develop PR:** All changes in `tests/` and committed binary fixtures — no CHANGELOG
    required. The PR description MUST include the actual test output proving the 2/4 fixture
    coverage summary (with --nocapture) and the two ITI test passes.

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
- **`COMMITTED_SAMPLES = "tests/fixtures"`:** Committed captures go directly in `tests/fixtures/`
  alongside the 21 existing committed captures. No subdirectory created. The `tests/fixtures/`
  root is NOT covered by any `.gitignore` entry — committed files here are naturally tracked.
- **`test_fixture_manifest_report()` pass/fail contract:**
  - Committed fixtures absent → test FAILS via hard-assert panic (CI-visible)
  - Gitignored corpus absent → advisory stdout only (visible with --nocapture) — test still passes
  - Standard CI without --nocapture: stdout NOT visible; hard-assert panic IS visible
- **Module-level constants:** `FIXTURE_MANIFEST` and `COMMITTED_FIXTURES` MUST be at module
  level, not inside the test function. This allows other test utilities or future assertions
  to reference them.
- **`#[ignore]` prohibition:** Do NOT mark any fixture-gated tests `#[ignore]`. Ignored tests
  are completely invisible in standard CI output. The hard-assert in `test_fixture_manifest_report()`
  is the correct CI-visibility mechanism.
- **Action SHA-pin policy (CI):** STORY-182 makes no `ci.yml` changes; no new SHA pins required.

## Library & Framework Requirements

| Dependency | Version | Source |
|------------|---------|--------|
| Rust stable | 1.91+ | CLAUDE.md MSRV |
| No new Cargo.toml deps | — | tests/ only, no new crate dependencies |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `tests/iec104_e2e_real_pcaps_tests.rs` | Modify | Add `COMMITTED_SAMPLES` const, `fixture_path()` fn, update `fixture_present()` and `run_iec104_pipeline()`, add module-level `FIXTURE_MANIFEST`/`COMMITTED_FIXTURES` consts, add `test_fixture_manifest_report()`, update module docstring + pipeline doc-comment |
| `tests/fixtures/iec104-iti-diverse.pcap` | New (binary) | ITI CC-BY-4.0; ≤100 KB; exercises TypeIDs 58–64; goes DIRECTLY in tests/fixtures/ |
| `tests/fixtures/iec104-iti-dissect.pcap` | New (binary) | ITI CC-BY-4.0; ≤100 KB; exercises 6-flow dissector path; goes DIRECTLY in tests/fixtures/ |
| `tests/fixtures/README.md` | Modify | Add IEC-104 committed-captures provenance rows (licensing notice lines 7–22 sweep + source URLs + per-file direct download URLs + attribution) |
| `tests/fixtures/E2E-PCAPS.md` | Modify | Note that ITI IEC-104 captures are now also committed under tests/fixtures/ |
| `.factory/maintenance/fixture-count-gate-entry.md` | New | Gate-entry evidence: manifest counts, COMMITTED_FIXTURES members, #[ignore] rejection rationale, D-510 G1 retrospective |

**Forbidden modifications:** `src/**/*`, `Cargo.toml`, `bin/*`, `CHANGELOG.md`,
`.github/workflows/ci.yml`

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `tests/iec104_e2e_real_pcaps_tests.rs` | effectful-shell | Filesystem I/O (pcap reads, path checks), analyzer pipeline execution, assertion outputs. |
| `tests/fixtures/` (binary captures) | data artifact | Static binary captures; not a code module. |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~5.5 k |
| `tests/iec104_e2e_real_pcaps_tests.rs` (full file, ~654 lines) | ~5.0 k |
| `tests/fixtures/README.md` (provenance table section) | ~1.0 k |
| `tests/fixtures/E2E-PCAPS.md` (IEC-104 section) | ~1.0 k |
| Binary pcap files (not token-counted) | ~0 k |
| **Total** | **~12.5 k** |
| Agent context window | 200 k (Sonnet) |
| **Budget usage** | **~6%** |

Well within context window. No story split required.

## Notes

- **DF-VALIDATION-001 status:** PG-W85-005 is LOCAL-CARRY-FORWARD per
  `df-validation-2026-07-25.md §PG-W85-005` (HIGH confidence). No upstream filing required.
- **No behavioral contract required:** E-11 convention.
- **Develop PR:** All five ACs can be batched in a single develop PR. No CHANGELOG entry
  required (`tests/` and `tests/fixtures/` are not in the AC-158-001 trigger set).
- **Wave-85 gate G1 retrospective:** D-510 was triggered on a **fixture-bearing host** where
  `test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu` ran with the stale assertion
  of 31 findings when the correct count (post-STORY-180 timed-command detection) is 66. This
  was a **stale-assertion failure**, not a clean-worktree silent-skip. Gate-fix PR #439
  (0ab6f52e) updated expectations 31→66. STORY-182 addresses the complementary structural
  gap: committed fixture absence is now a visible CI failure; the stale-assertion class is
  prevented by keeping STORY-180 ground-truth expectations in sync with the committed captures.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.2 | 2026-07-25 | story-writer | WAVE-86 PASS-2 REMEDIATION — F-003 HIGH (struck all "loud"/"visible in CI" claims for println!() stdout; documented --nocapture requirement and libtest capture semantics; AC-182-004 updated with advisory note; #[ignore] rejection rationale added to Background and Architecture Compliance); F-004 HIGH (AC-182-001 adds manifest-count sanity assertions: FIXTURE_MANIFEST.len()==4, COMMITTED_FIXTURES.len()==2; Task 8 gate-entry artifact added with #[ignore] rejection rationale; D-510 G1 retrospective documented); F-007 MED (Notes corrected: D-510 G1 FAIL was stale-assertion on fixture-bearing host at 31 vs 66, NOT clean-worktree silent skip; clean-worktree gap was separate concern addressed structurally by this story); F-008 MED (Narrative fixed: "31 tests ran out of 66 expected" → "test asserting 31 findings when 66 is now the correct expectation" — 31/66 are FINDING counts, not test counts); F-009 MED (AC-182-001 fixture_present() snippet updated to include path.display() via committed_path.display() while mentioning both search locations; Task 4 aligned); F-010 MED (tests/fixtures/local-samples/README.md removed from FSR and Task 7 — this file is gitignored/untracked; Task 7 updated to remove the local-samples/README.md sweep); F-012 MED (AC-182-002 MUST NOT commit line cites fixed: :25/:26 → :138/:273 per actual test file line numbers); F-013 MED (committed-samples/ directory concept DROPPED — F-013 rules files go directly in tests/fixtures/ following 21-capture convention; COMMITTED_SAMPLES="tests/fixtures"; all directory references updated throughout; FSR updated; Background §Gitignore updated; EC-001 updated; traces_to updated); F-014 MED (AC-182-003 now explicitly notes committed fixtures NEVER trigger skip path since they live in tests/fixtures/ directly); F-016 LOW (AC-182-002 and Task 2 updated with per-file direct download URLs: 090813_diverse.pcap and TestDissectIec104.pcap upstream filenames and full raw URLs); F-017 LOW (Task 7 updated to include tests/fixtures/README.md lines 7–22 licensing notice sweep for CC-BY-4.0 class); F-021 LOW (AC-182-005 now explicitly requires COMMITTED_FIXTURES at module level alongside FIXTURE_MANIFEST; both constants shown in module-level placement). |
| 1.1 | 2026-07-25 | story-writer | WAVE-86 PASS-1 REMEDIATION — F-011 CRIT (AC-182-003 complete redesign: `fixture_path(name) -> Option<PathBuf>` shared resolver used by BOTH `fixture_present()` and `run_iec104_pipeline()`); F-012 HIGH (union approach: manifest + loud skip-reporting + hard-assert partition + committed samples; AC-182-005 new); F-013 HIGH (committed fixtures absent → test FAILS visible; AC-182-005 formalises the partition); F-014 HIGH (resolved by F-012/F-013 hard-assert partition); F-015 MED (unified skip-notice format); F-016 MED (attribution via README.md provenance table); F-017 MED (sibling-prose sweep tasks); F-018 MED (test_fixture_manifest_report() returns unit); F-021 MED (iec104-iti-dissect.pcap made MANDATORY); F-022 LOW (≤100 KB gate hard); F-020 MED (inputs: set). |
| 1.0 | 2026-07-25 | story-writer | Initial authorship — wave-86 STORY-CREATION BURST (D-516). |
