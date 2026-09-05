---
document_type: story
story_id: STORY-182
epic_id: E-11
version: "2.12"
status: delivered
producer: story-writer
timestamp: 2026-07-25T00:00:00Z
phase: f7
level: feature
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
  - .github/workflows/ci.yml
  - CLAUDE.md
  - .factory/maintenance/fixture-count-gate-entry.md  # created by this story; on factory-artifacts branch (state-manager commits)
  - .gitignore
inputs:
  - .factory/cycles/wave-085/lessons.md
  - .factory/planning/df-validation-2026-07-25.md
input-hash: "9a0f34c"
# NOTE: Stored value is the canonical Python hash (9a0f34c); the bash hook reports a divergent value (0a1812a) — advisory only per PG-HASH-HOOK-DIVERGENCE.
---

# STORY-182: E2E Fixture Manifest + Committed Representative Captures: Eliminate False-Green cargo test in Clean Worktrees

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** delivered
**Wave:** 86
**Points:** 4
**Priority:** P2

## Narrative

- **As a** CI system, gate reviewer, and contributor running `cargo test` in a clean worktree
- **I want** the IEC-104 ITI e2e fixture-gated tests to hard-fail when their committed capture
  is absent, to report visible skip notices for gitignored corpus files, AND for the committed ITI
  capture to ensure CI genuinely exercises the timed-command detection path from STORY-180
- **So that** the clean-worktree silent-skip class behind PG-W85-005 is structurally
  eliminated **for the committed partition of the IEC-104 harness** (the one committed ITI
  capture always runs in CI); the three gitignored fixtures now emit visible FIXTURE-SKIPPED
  notices via the additive CI step instead of skipping silently, but their tests still report
  `ok` on absence; and the D-510 stale-expectation class
  is now detected on every CI run rather than only on fixture-bearing hosts (detection, not
  prevention — accurate expectations remain the implementer's obligation); absence of the
  committed capture is always a visible CI failure, and the timed-command code path always
  runs in CI; sibling e2e harnesses retain the structural gap until a follow-up story (see
  Notes §Sibling e2e harnesses)

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
- `tests/fixtures/README.md` (lines 5–34): licensing notice + provenance table. All fixtures
  committed BY THIS STORY must have a row added to this table with source URL, modification
  indication, and license. This is the authoritative attribution record — not a separate `ATTRIBUTION.md`.
- Lines 7–26 of `tests/fixtures/README.md` contain the licensing notice (lines 7–22 are the notice body; lines 24–26 are the malware clause, also part of the licensing notice). This sweep must be
  updated to describe the CC-BY-4.0 class for the committed ITI capture when the provenance row is added.
- Malware-content check: committed captures must not contain live malware C2 traffic or
  exploit payloads (README.md §Licensing notice).
- CC-BY-4.0 attribution for ITI captures: "ICS Security Tools, Illinois Institute of
  Technology (ITI). Licensed under CC-BY-4.0." Per-file: source URL, modification note,
  upstream license notice retention. Attribution is already recorded in E2E-PCAPS.md §IEC-104;
  the README.md provenance table row is the additional required surface.

### Committed captures decision — files go directly in `tests/fixtures/`

Only `iec104-iti-diverse.pcap` is the MANDATORY committed fixture. It goes **directly in
`tests/fixtures/`** — the same directory as the 25 existing committed captures — NOT in a
new subdirectory. `COMMITTED_SAMPLES` const is set to `"tests/fixtures"`. The existing
convention (25 tracked captures in `tests/fixtures/`) is the authoritative precedent.

- `iec104-iti-diverse.pcap`: 14 KB (< 100 KB), ITI CC-BY-4.0. Exercises timed control-command
  TypeIDs 58/59/61/63 (of the 58–64 detection range, BC-2.19.029/030). Produces 66 findings
  with STORY-180 detection.
- `iec104-iti-dissect.pcap`: 11 KB (< 100 KB), ITI CC-BY-4.0. Exercises 6-flow dissector
  path. Produces 11 findings (T0814×2 + T1692.001×9). **NOT committed** (F-009 orchestrator
  ruling, 2026-07-26 — POSITIVE EVIDENCE OF UPSTREAM-OF-ITI ORIGIN: upstream filename
  `TestDissectIec104.pcap` + E2E-PCAPS.md description "Wireshark-dissector test capture"
  indicate Wireshark dissector test suite origin; D-524 ruling; stays gitignored).

Only `iec104-iti-diverse.pcap` is committed. The FIXTURE_MANIFEST contains 4 entries. In a
clean checkout (only `tests/fixtures/` present, **without** `tests/fixtures/local-samples/`),
the manifest-report shows **1/4 fixtures present** (the two Wireshark samples, iec104.pcap
and iec104-sq.pcapng, are NOT committed — "not redistributed"; and iec104-iti-dissect.pcap
is NOT committed per F-009; all three live in gitignored `tests/fixtures/local-samples/`).
**On a fixture-bearing host** (local-samples present), all 4 fixtures resolve and the
manifest-report shows 4/4 — the "1/4 fixtures present" output is only producible WITHOUT
local-samples (clean-worktree equivalent).

### Must NOT commit

- `iec104.pcap`: Wireshark Foundation "not redistributed" — see `tests/iec104_e2e_real_pcaps_tests.rs`
  licensing prose at line 138 and `tests/fixtures/E2E-PCAPS.md`.
- `iec104-sq.pcapng`: Wireshark Foundation "not redistributed" — see test-file line 273 and E2E-PCAPS.md.

### Gitignore placement

`tests/fixtures/` (root) is NOT covered by any `.gitignore` entry. `.gitignore` covers only
`/tests/fixtures/local-samples/`. Since committed captures go directly in `tests/fixtures/`,
those pcap files are naturally tracked — no `.gitignore` change is required for the committed
captures themselves. However, **`.gitignore` entries ARE required for `coverage-out.txt`
and `red-out.txt`** — `coverage-out.txt` is the transient CI artifact written by the additive
ci.yml step (`tee coverage-out.txt`); `red-out.txt` is the transient artifact from the
AC-182-005 manual RED demonstration. Both must not be accidentally committed. See Task 10b
and the Architecture Mapping + File Structure Requirements for the precise `.gitignore` entries.

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

**`#[ignore]` rejection (F-009):** `#[ignore]` was considered and rejected for committed
fixture presence checks. Correction of a common misconception: libtest DOES report ignored
tests — each produces a per-test `... ignored` line and the summary shows `N ignored`. The
real reasons for rejecting `#[ignore]` here are:
1. Committed fixture absence is a **broken checkout** state that MUST FAIL visibly, not a
   reportable skip. An ignored test never fails; a hard-assert does.
2. `#[ignore]` is **static** — it cannot be conditional on runtime fixture presence without
   nightly custom test harnesses. A regular `#[test]` with a conditional assert (or panic)
   IS conditionally skippable at runtime.
3. df-validation recommended `#[ignore]` as a general pattern for optional fixtures; this
   story deliberately diverges from that recommendation for COMMITTED fixtures (broken-checkout
   semantics). That divergence is recorded here and in the Task 8 gate-entry artifact.
The hard-assert inside a regular `#[test]` function is the correct mechanism for committed
fixtures: it produces a visible test-failure when committed captures are absent from their
expected committed location (`tests/fixtures/`).

### Fixture count anchors

All count literals in this story use the MANDATORY decision (F-009 orchestrator ruling,
2026-07-26): **1 committed capture** (`iec104-iti-diverse.pcap` only), 4 total manifest
entries. `iec104-iti-dissect.pcap` stays gitignored — POSITIVE EVIDENCE OF UPSTREAM-OF-ITI
ORIGIN: upstream filename `TestDissectIec104.pcap` + E2E-PCAPS.md description
"Wireshark-dissector test capture" indicate Wireshark dissector test suite origin
(F-009, D-524 ruling). The canonical manifest-report string in a clean checkout (CI) is:
`Fixture coverage: 1/4 fixtures present (3 fixture-gated tests will be skipped)`.

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
  `[iec104-e2e] SKIP:` prefix; the diagnostic path is branched on
  `COMMITTED_FIXTURES.contains(&filename)` so committed-eligible fixtures show the
  committed path while non-committable fixtures show the local-samples path with a
  licensing clause (F-W86S-P12-008):
  ```rust
  fn fixture_present(filename: &str) -> bool {
      match fixture_path(filename) {
          Some(_) => true,
          None => {
              let base = Path::new(env!("CARGO_MANIFEST_DIR"));
              if COMMITTED_FIXTURES.contains(&filename) {
                  // Committed-eligible fixture: show the committed path in the diagnostic.
                  // Absence here means a broken checkout — the hard-assert in
                  // test_fixture_manifest_report() catches this at test time.
                  let committed_path = base.join(COMMITTED_SAMPLES).join(filename);
                  eprintln!(
                      "[iec104-e2e] SKIP: fixture '{}' not found at {} (or in local-samples). \
                       Run `bin/fetch-e2e-pcaps` to populate local-samples.",
                      filename, committed_path.display()
                  );
              } else {
                  // Gitignored corpus fixture (Wireshark "not redistributed" or
                  // origin-unclear — see COMMITTED_FIXTURES and Background §Must NOT commit).
                  // Show local-samples path; do NOT suggest tests/fixtures/ as a target.
                  let local_path = base.join(LOCAL_SAMPLES).join(filename);
                  eprintln!(
                      "[iec104-e2e] SKIP: fixture '{}' not found at {} \
                       (do not commit to tests/fixtures/ — licensing/redistribution \
                        constraint; run `bin/fetch-e2e-pcaps` to populate local-samples).",
                      filename, local_path.display()
                  );
              }
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
- And a new `FIXTURE_MANIFEST: &[&str]` constant is added inside `mod iec104_e2e_real_pcaps`
  (alongside the existing `use std::path::Path` at :39) listing all 4 expected fixture filenames:
  `["iec104.pcap", "iec104-sq.pcapng", "iec104-iti-diverse.pcap", "iec104-iti-dissect.pcap"]`
- And a new `COMMITTED_FIXTURES: &[&str]` constant is added inside `mod iec104_e2e_real_pcaps`
  (alongside the existing `use std::path::Path` at :39 and alongside `FIXTURE_MANIFEST`;
  NOT inside the test function):
  `["iec104-iti-diverse.pcap"]`
  (single entry only — iec104-iti-dissect.pcap is NOT committed per F-009 D-524 ruling)
- And a new `#[test]` function `test_fixture_manifest_report()` is added inside
  `mod iec104_e2e_real_pcaps` (DF-TEST-NAMESPACE-001; hard-assert partition detailed in
  AC-182-005; skip-reporting half covered here):
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
- And `test_fixture_manifest_report()` includes non-self-referential manifest sanity checks —
  these are defined canonically in **AC-182-005** (the hard-assert partition) to avoid
  prescribing the same loop twice (F-013 deduplication): AC-182-005 covers (a) direct
  `Path::exists()` presence in `tests/fixtures/` for every `COMMITTED_FIXTURES` entry,
  (b) `FIXTURE_MANIFEST.contains()` superset check, and (c) `fixture_path()` resolver
  coupling (every entry must resolve AND the resolved path must be under `tests/fixtures/`,
  not `local-samples/`). See AC-182-005 for the exact Rust implementation of all three loops.

- Then `cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture`
  **WITHOUT local-samples present** (committed captures only) prints:
  `Fixture coverage: 1/4 fixtures present (3 fixture-gated tests will be skipped)`
  **Precondition:** `tests/fixtures/local-samples/` must be absent or empty; on a
  fixture-bearing host the output will show 4/4, not 1/4 (see two-environment protocol in
  Task 9)
- And `cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact 2>&1 | grep -E "1 passed"`
  (WITHOUT --nocapture, as in CI) PASSES silently when committed fixtures are present

Verification (two-environment protocol — F-006):
```bash
set -euo pipefail
# Environment A: fixture-bearing host verification (Task 1/2 prerequisite)
# Run with local-samples present — confirms committed-capture tests pass
cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact 2>&1 | grep -E "1 passed"
# Must show: test result: ok. 1 passed (committed+local fixtures all present)

# Environment B: clean-worktree equivalent (produces the 1/4 output)
# local-samples may be absent on develop (no worktree corpus populated); the if-guard
# prevents a false-error from mv when the source directory does not exist, and prevents
# the trap from firing a bogus restore of a directory that was never moved:
if [ -d tests/fixtures/local-samples ]; then
  [ ! -e /tmp/ls-bak ] || { echo "backup path occupied — clean up first"; exit 1; }
  mv tests/fixtures/local-samples /tmp/ls-bak
  trap 'mv /tmp/ls-bak tests/fixtures/local-samples' EXIT  # unconditional restore under set -e
  set -euo pipefail
  cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture | tee coverage-out.txt
  grep -qE "Fixture coverage: 1/4" coverage-out.txt  # (currently 1/4 — tracks committed-partition/manifest sizes)
  grep -qE "test result: ok" coverage-out.txt
  mv /tmp/ls-bak tests/fixtures/local-samples; trap - EXIT  # restore and disarm trap
else
  echo "local-samples already absent — skip move-aside"
  set -euo pipefail
  cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture | tee coverage-out.txt
  grep -qE "Fixture coverage: 1/4" coverage-out.txt  # (currently 1/4 — tracks committed-partition/manifest sizes)
  grep -qE "test result: ok" coverage-out.txt
fi

# CI equivalent: no --nocapture; test must pass regardless of environment
cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact 2>&1 | grep -E "1 passed"
# Must show: test result: ok. 1 passed
```

### AC-182-002 (traces to PG-W85-005 — committed mandatory captures)

**Only `iec104-iti-diverse.pcap` is committed** directly under `tests/fixtures/`; its
provenance is recorded in `tests/fixtures/README.md`. `iec104-iti-dissect.pcap` is NOT
committed (F-009 orchestrator ruling, 2026-07-26) — see provenance rationale below.

**License precondition (orchestrator-verified, 2026-07-25):** Upstream `ITI/ICS-Security-Tools`
repo-level `LICENSE.md` is CC-BY-4.0 per GitHub API (`repos/ITI/ICS-Security-Tools/license`
response: `{"license":"CC-BY-4.0","name":"Creative Commons Attribution 4.0 International",
"path":"LICENSE.md"}`). The ITI captures reside at `pcaps/IEC60870-5-104/` within the repo;
the repo-level license applies to all in-repo content. The README provenance row must cite
"upstream `LICENSE.md` (CC-BY-4.0, repo-level)".

**Provenance rationale for `iec104-iti-dissect.pcap` exclusion (F-009 orchestrator ruling,
2026-07-26 — discriminator corrected F-001, PASS-8):** The exclusion basis is NOT "no
per-file provenance" (that is true of BOTH ITI captures and therefore cannot discriminate
between them). The correct basis is POSITIVE EVIDENCE OF UPSTREAM-OF-ITI ORIGIN for
`iec104-iti-dissect.pcap`: the upstream filename `TestDissectIec104.pcap` and this repo's
own `tests/fixtures/E2E-PCAPS.md` description "Wireshark-dissector test capture" indicate
the file originated in the Wireshark dissector test suite — i.e., ITI is likely
redistributing a third-party artifact whose license is not ITI's repo-level CC-BY-4.0 to
grant. By contrast, `090813_diverse.pcap` (a date-stamped capture name with no
third-party attribution signal) shows NO indication of non-ITI origin, so the repo-level
CC-BY-4.0 credibly covers it as first-party ITI content. **Rule: repo-level license
suffices ABSENT contrary indication of third-party upstream origin;
`iec104-iti-dissect.pcap` has such indication; `iec104-iti-diverse.pcap` does not.**
`iec104-iti-dissect.pcap` stays gitignored and is fetched only into `local-samples/` for
local development (Task 1 Steps 1c/1d).

- Given `iec104-iti-diverse.pcap` (14 KB, 173 packets, ITI CC-BY-4.0) exercises timed
  control-command TypeIDs 58/59/61/63 (of the 58–64 detection range) and is the ground-truth
  capture for STORY-180's timed-command detection (66 findings) —
  licensed CC-BY-4.0 permitting redistribution with attribution; provenance independently
  verifiable via repo-level `LICENSE.md`
- When the implementer verifies on a fixture-bearing host that the file is ≤ 100 KB
  (E2E-PCAPS.md §IEC-104 records: diverse=14 KB — well within the limit)
- And copies `iec104-iti-diverse.pcap` **directly to `tests/fixtures/`** (alongside existing
  committed captures)
- And adds a provenance row to `tests/fixtures/README.md` in the existing "Fixtures with
  recorded provenance" table (lines 30–34), or adds a new IEC-104 subsection following the
  established table format, including the direct download URL:
  ```
  | `iec104-iti-diverse.pcap` | [090813_diverse.pcap](https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/IEC60870-5-104/090813_diverse.pcap) from [ITI/ICS-Security-Tools](https://github.com/ITI/ICS-Security-Tools/tree/master/pcaps/IEC60870-5-104) | IEC-104; 173 packets; 14 KB; upstream `LICENSE.md` (CC-BY-4.0, repo-level); exercises timed control-command TypeIDs 58/59/61/63 (of the 58–64 detection range, BC-2.19.029/030); produces 66 findings (STORY-180 ground-truth). Attribution: see §Licensing notice. |
  ```
  (Notes cell above carries `Attribution: see §Licensing notice`; the full attribution text
  WITH the Source: sentence lives ONLY in README §Licensing notice per Task 7 — single
  authoritative destination.)

- Then `git ls-files --error-unmatch tests/fixtures/iec104-iti-diverse.pcap` exits 0 (file is tracked)

**MUST NOT commit:**
- `iec104.pcap` (Wireshark Foundation "not redistributed" — test-file line 138)
- `iec104-sq.pcapng` (Wireshark Foundation "not redistributed" — test-file line 273)
- `iec104-iti-dissect.pcap` (POSITIVE EVIDENCE OF UPSTREAM-OF-ITI ORIGIN: upstream filename TestDissectIec104.pcap + E2E-PCAPS.md "Wireshark-dissector test capture" indicate Wireshark dissector test suite origin — F-009, D-524 ruling; stays gitignored)

**Size gate (hard):** The committed ITI capture is recorded as 14 KB in E2E-PCAPS.md§IEC-104,
making the >100 KB branch UNREACHABLE IN PRACTICE for this specific file. IF the branch is
ever exercised (e.g., the file is replaced with a larger capture), the implementer MUST NOT
commit the oversized file and is obligated to co-update: re-record the sha256 in E2E-PCAPS.md,
re-derive the expected finding count (currently 66), and update the README attribution note.
The size constraint is not waivable.

Verification:
```bash
set -euo pipefail
git ls-files --error-unmatch tests/fixtures/iec104-iti-diverse.pcap
# Must list the file as tracked (exits non-zero on untracked files; plain git ls-files
# exits 0 even for untracked paths, which would silently pass — --error-unmatch is required)

test "$(wc -c <"tests/fixtures/iec104-iti-diverse.pcap")" -le 102400
# (portable: wc -c works on macOS and Linux; stat -f%z is macOS-only)

test "$(shasum -a 256 tests/fixtures/iec104-iti-diverse.pcap | cut -d' ' -f1)" = "07b9a0879dc83e420c4cf83b37fb5830d1d8fb5f6ac6edc435896f70b0fc6bc7"
# (value from tests/fixtures/E2E-PCAPS.md:358 — F-003 integrity gate)
```

**Download-and-verify gate for the gitignored dissect capture** (F-P11-010 — relocated from
AC-182-003; applies to the downloaded file only, not to committed state): when downloading
`iec104-iti-dissect.pcap` via Task 1 Step 1c, the sha256 hash
`292c18a8765db3b1bcaa9bd0b8455e4e61b8366cc5910a7363b7381eb11441b8`
(recorded at `tests/fixtures/E2E-PCAPS.md:359`) MUST be verified after download and before
placing the file in `local-samples/`; a hash mismatch MUST abort the download step.
This gate applies to the downloaded gitignored file only — committed state has its own
integrity gate above (sha256 check against E2E-PCAPS.md:358).

### AC-182-003 (traces to PG-W85-005 — committed fixtures run on every cargo test invocation)

With `fixture_path()` as the shared resolver (AC-182-001), the committed captures are found
automatically and their corresponding tests run on every `cargo test` invocation — including CI.
Committed fixtures never trigger the skip path.

- Given `fixture_path()` checks `COMMITTED_SAMPLES` (= `"tests/fixtures"`) before `LOCAL_SAMPLES`
- And `const COMMITTED_SAMPLES: &str = "tests/fixtures"` is added alongside the existing
  `const LOCAL_SAMPLES: &str = "tests/fixtures/local-samples"`
- When `iec104-iti-diverse.pcap` is present in `tests/fixtures/` (as a committed file,
  always available in any checkout; `iec104-iti-dissect.pcap` stays gitignored per F-009)
- Then `fixture_present("iec104-iti-diverse.pcap")` returns `true` in a clean worktree,
  causing the corresponding test body to run all assertions (not return early)
- And `run_iec104_pipeline("iec104-iti-diverse.pcap")` successfully opens the file from the
  committed `tests/fixtures/` path
- And `fixture_present("iec104-iti-diverse.pcap")` NEVER prints the `[iec104-e2e] SKIP:` line
  (committed fixture is always found in `tests/fixtures/` — no skip path is taken)
- _See AC-182-002 for the download-and-verify sha256 gate that applies to the gitignored
  dissect capture (Task 1 Step 1c); that gate covers the downloaded file, not committed state_

Verification:
```bash
set -euo pipefail
# On a clean worktree with only tests/fixtures/ committed captures populated:
cargo test --test iec104_e2e_real_pcaps_tests \
  iec104_e2e_real_pcaps::test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu -- --exact 2>&1 | \
  grep -E "1 passed"
# Must match: test result: ok. 1 passed (not silently skip); 66 findings expected

# Non-vacuous (F-016): confirm no SKIP messages for the committed ITI fixture
# (gating form: tee-to-file then test count; grep -c || true is non-gating — F-P10-003):
cargo test --test iec104_e2e_real_pcaps_tests \
  iec104_e2e_real_pcaps::test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu -- --exact --nocapture 2>&1 | tee coverage-out.txt
# Existence guard: coverage-out.txt must be non-empty before count check (prevents false-GREEN when file absent):
test -s coverage-out.txt
# SKIP-count check (non-vacuous — file existence asserted above):
test "$(grep -c '\[iec104-e2e\] SKIP:' coverage-out.txt)" -eq 0
# Expected: 0 SKIP lines (committed fixture always found; pattern catches ANY iec104-e2e SKIP)
```

### AC-182-004 (traces to PG-W85-005 — regression: clean-worktree observable outcome)

A clean-worktree run (only `tests/fixtures/` committed captures present, local-samples absent)
has a fully observable, non-silent outcome:
(a) The one ITI committed-capture test (`iec104-iti-diverse.pcap`) RUNS and PASSES (not
    silently skip); `iec104-iti-dissect.pcap` skips (gitignored, not committed per F-009)
(b) The two Wireshark fixture tests skip via the existing `fixture_present()` stderr path
    (visible with `--nocapture 2>&1`; not visible in standard CI output without it)
(c) `test_fixture_manifest_report()` itself PASSES (never panics when committed fixtures present)
(d) `test_fixture_manifest_report()` FAILS with a visible assertion message if a committed
    fixture is absent (CI-visible because panics always appear in test output regardless of
    capture mode — see AC-182-005)
(e) The `Fixture coverage: N/4 fixtures present` summary (denominator 4 currently — tracks
    `FIXTURE_MANIFEST.len()`; update ci.yml step and this note together when adding fixtures)
    is visible on every run incl. after TEST failures (but NOT after compile failures —
    the step's own `cargo test` cannot produce coverage output if the crate doesn't build)
    via a dedicated step with `if: ${{ !cancelled() }}` named "IEC-104 fixture
    coverage report (visible)" added to the test job in `.github/workflows/ci.yml`,
    placed AFTER the main `cargo test --all-targets` step (additive step per F-014
    orchestrator ruling — closes the df-validation "loud, machine-readable" requirement;
    `if: ${{ !cancelled() }}` is visible after TEST failures; after checkout/compile
    failures the step runs but cannot emit the coverage line — !cancelled() guarantees
    execution, not output; only workflow cancellation prevents this step, unlike
    `if: always()` which fires even on cancellation)

**Stdout advisory note (F-003):** The `Fixture coverage: 1/4 ...` and `FIXTURE-SKIPPED:` lines
are printed via `println!()` and are ONLY visible when `--nocapture` is passed. The standard
CI `cargo test --all-targets` step does NOT produce them. The additive ci.yml step (outcome
(e) above) makes them permanently CI-visible. The CI-visible signals are:
- Test PASS/FAIL counts (always reported by libtest)
- Hard-assert panics in `test_fixture_manifest_report()` (AC-182-005)
- `Fixture coverage: N/4` println!() summary (denominator 4 currently — tracks
  `FIXTURE_MANIFEST.len()`) — visible after TEST failures (not after compile failures)
  via additive ci.yml step with `if: ${{ !cancelled() }}` (F-014)
- `[iec104-e2e] SKIP:` eprintln() in `fixture_present()` — visible with `--nocapture`

Verification:
```bash
set -euo pipefail
# WITHOUT-LOCAL-SAMPLES PRECONDITION (item 1/4): tests/fixtures/local-samples/ must be
# absent or moved out of scope to produce the 1/4 output. On a fixture-bearing host with
# local-samples present, this check shows 4/4 — the precondition makes it a genuine gate
# rather than a no-op (see two-environment protocol in Task 9 for the move protocol).
if [ -d tests/fixtures/local-samples ]; then
  [ ! -e /tmp/ls-bak ] || { echo "backup path occupied — clean up first"; exit 1; }
  mv tests/fixtures/local-samples /tmp/ls-bak
  trap 'mv /tmp/ls-bak tests/fixtures/local-samples' EXIT  # unconditional restore under set -e
  set -euo pipefail
  cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture | tee coverage-out.txt
  grep -qE "Fixture coverage: 1/4" coverage-out.txt  # (currently 1/4 — tracks committed-partition/manifest sizes)
  grep -qE "test result: ok" coverage-out.txt
  mv /tmp/ls-bak tests/fixtures/local-samples; trap - EXIT  # restore and disarm trap
else
  echo "local-samples already absent — skip move-aside"
  set -euo pipefail
  cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture | tee coverage-out.txt
  grep -qE "Fixture coverage: 1/4" coverage-out.txt  # (currently 1/4 — tracks committed-partition/manifest sizes)
  grep -qE "test result: ok" coverage-out.txt
fi
# Must print (WITHOUT local-samples):
#   Fixture coverage: 1/4 fixtures present (3 fixture-gated tests will be skipped)

# CI-mode verification (no --nocapture):
cargo test --test iec104_e2e_real_pcaps_tests
# Must exit 0; iec104-iti-diverse.pcap test PASSES; manifest test PASSES silently

# Additive CI step verification (F-014 — makes coverage summary visible on every run):
set -euo pipefail
cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture | tee coverage-out.txt
grep -qE "Fixture coverage: [1-9][0-9]*/[0-9]+" coverage-out.txt
grep -qE "test result: ok" coverage-out.txt
# Must print "Fixture coverage: N/M fixtures present ..." (if: ${{ !cancelled() }} — visible after TEST failures but NOT after workflow cancellation)
# In CI: GitHub Actions runs bash with -e by default → non-zero cargo test exit genuinely gates the step even without explicit set -e.
# In this local block: set -euo pipefail is explicit and required — bash does NOT default to -e in a manual terminal session.
# Both greps on file: step FAILS if cargo test exits non-zero, no coverage line, OR test did not pass ("test result: ok" absent).
# Denominator is [0-9]+ (not hardcoded /4) — loosened so adding fixtures only requires updating
# the FIXTURE_MANIFEST.len() assertion, ci.yml step, and Task 9 Env A 4/4 expected value together.
```

### AC-182-005 (traces to PG-W85-005 — hard-assert: committed fixtures MUST be present)

A test that FAILS (not silently passes) when a committed/tracked fixture file is absent from
the repo. Both `FIXTURE_MANIFEST` and `COMMITTED_FIXTURES` are declared inside
`mod iec104_e2e_real_pcaps` (alongside the existing `use std::path::Path` at :39).

- Given `iec104-iti-diverse.pcap` is committed into `tests/fixtures/` (tracked by git,
  always present in any checkout); `iec104-iti-dissect.pcap` is NOT committed (F-009)
- And `COMMITTED_FIXTURES` is declared inside `mod iec104_e2e_real_pcaps` (alongside the
  existing `use std::path::Path` at :39) alongside `FIXTURE_MANIFEST`:
  ```rust
  const FIXTURE_MANIFEST: &[&str] = &[
      "iec104.pcap",
      "iec104-sq.pcapng",
      "iec104-iti-diverse.pcap",
      "iec104-iti-dissect.pcap",
  ];

  const COMMITTED_FIXTURES: &[&str] = &[
      "iec104-iti-diverse.pcap",
      // iec104-iti-dissect.pcap: NOT committed — POSITIVE EVIDENCE OF UPSTREAM-OF-ITI ORIGIN
      // (upstream filename TestDissectIec104.pcap + E2E-PCAPS.md "Wireshark-dissector test
      // capture"); stays gitignored per F-009 D-524 ruling
  ];

  // FIXTURE_GATED_TESTS: maps every fixture-gated test function to its fixture filename.
  // Declared at module level inside mod iec104_e2e_real_pcaps (alongside FIXTURE_MANIFEST
  // and COMMITTED_FIXTURES). New tests calling fixture_present() MUST register here.
  const FIXTURE_GATED_TESTS: &[(&str, &str)] = &[
      ("test_e2e_BC_2_19_iec104_pcap_T0836_T1692_001_interrogation", "iec104.pcap"),
      ("test_e2e_BC_2_19_iec104_sq_pcapng_zero_findings_benign_uframes", "iec104-sq.pcapng"),
      ("test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu", "iec104-iti-diverse.pcap"),
      ("test_e2e_BC_2_19_iec104_iti_dissect_T0814_T1692_001_control_coverage", "iec104-iti-dissect.pcap"),
  ];
  ```
- When `test_fixture_manifest_report()` is extended with a hard-assert partition that checks
  the **committed location directly** (not via `fixture_path()`, which also checks
  `local-samples/` and would be INERT on a fixture-bearing host — F-005):
  ```rust
  // Hard assert: committed (tracked) fixtures MUST be present in tests/fixtures/ directly.
  // Using Path::exists() on the committed path, NOT fixture_path() which also checks
  // local-samples/ — the direct check works correctly regardless of local-samples presence.
  // This panic IS always visible in CI output regardless of --nocapture (assertion failure).
  for name in COMMITTED_FIXTURES {
      assert!(
          Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name).exists(),
          "[iec104-e2e] REGRESSION: committed fixture '{}' is absent from \
           tests/fixtures/ — this is a broken checkout. \
           Run `git checkout tests/fixtures/` to restore.",
          name
      );
  }
  // FIXTURE_MANIFEST superset check (F-013 canonical location — was in AC-182-001):
  // Guards against a committed fixture being removed from the manifest (drift).
  for name in COMMITTED_FIXTURES {
      assert!(
          FIXTURE_MANIFEST.contains(name),
          "FIXTURE_MANIFEST does not contain committed fixture '{}' — \
           update FIXTURE_MANIFEST to include all entries from COMMITTED_FIXTURES",
          name
      );
  }
  // Manifest-size drift pin (fires on manifest growth/shrink only):
  // This len() check fires when FIXTURE_MANIFEST gains or loses entries.
  // It does NOT detect an unregistered test silently using a fixture not in the manifest.
  // FIXTURE_GATED_TESTS registry (module-level) catches renames of registered tests;
  // the fixture_present() call-site count assertion below catches unregistered additions.
  // Co-update loci when fixtures are added: FIXTURE_MANIFEST, this assertion,
  // ci.yml "Fixture coverage: [1-9][0-9]*/[0-9]+" step, Task 9 Env A 4/4 expected value,
  // and the "1/4" expected-value literals (Env B blocks, Task 8 obligation, EC rows).
  assert_eq!(
      FIXTURE_MANIFEST.len(), 4,
      "FIXTURE_MANIFEST.len() must equal the count of distinct fixture names used by \
       fixture-gated tests (currently 4: 1 committed ITI + 3 gitignored (2 Wireshark + 1 ITI)); \
       update FIXTURE_MANIFEST, this assertion, ci.yml coverage step, and Task 9 Env A \
       4/4 expected value together when new fixtures are added"
  );
  // Fixture-gated test registry assertion — BIDIRECTIONAL set equality:
  // Direction 1 (gated ⊆ manifest): every FIXTURE_GATED_TESTS entry's fixture_name
  // must be in FIXTURE_MANIFEST. Registered entries with wrong names fail here.
  for (_, fixture_name) in FIXTURE_GATED_TESTS {
      assert!(
          FIXTURE_MANIFEST.contains(fixture_name),
          "FIXTURE_GATED_TESTS entry '{}' is not in FIXTURE_MANIFEST — \
           update FIXTURE_MANIFEST to include it, or correct this registry entry",
          fixture_name
      );
  }
  // Direction 2 (manifest ⊆ gated): every FIXTURE_MANIFEST entry must be exercised
  // by at least one FIXTURE_GATED_TESTS entry. Catches fixtures added to the manifest
  // that no test actually gates on — a manifest entry with no gated test is dead weight.
  for manifest_name in FIXTURE_MANIFEST {
      assert!(
          FIXTURE_GATED_TESTS.iter().any(|(_, f)| f == manifest_name),
          "FIXTURE_MANIFEST entry '{}' is not exercised by any FIXTURE_GATED_TESTS entry — \
           add a registry entry for the test that uses it, or remove it from FIXTURE_MANIFEST",
          manifest_name
      );
  }
  // FIXTURE_GATED_TESTS count pin (update when a new fixture-gated test is added):
  assert_eq!(
      FIXTURE_GATED_TESTS.len(), 4,
      "FIXTURE_GATED_TESTS.len() must equal the count of fixture-gated tests (currently 4); \
       update FIXTURE_GATED_TESTS and this assertion together when tests are added or removed"
  );
  // Per-test function-name coupling: reads the harness source file at test time and asserts
  // each registered name exists as `fn <name>` in the source.
  // NON-SELF-REFERENTIAL: the predicate checks for `fn test_name` (present only at the
  // function-definition site), not merely `test_name` (which also appears inside the
  // FIXTURE_GATED_TESTS string literal). If a test is renamed but FIXTURE_GATED_TESTS is
  // not updated, `fn <old_name>` will not be found in source → assertion fails.
  // This predicate CAN fail: the fn-definition span and the FIXTURE_GATED_TESTS string
  // literal are different text, so renaming a test without updating the registry produces
  // a genuine test failure (not a vacuous pass).
  let harness_src = std::fs::read_to_string(
      Path::new(env!("CARGO_MANIFEST_DIR"))
          .join("tests/iec104_e2e_real_pcaps_tests.rs")
  ).expect("[iec104-e2e] failed to read harness source for FIXTURE_GATED_TESTS coupling check");
  for (test_name, _) in FIXTURE_GATED_TESTS {
      assert!(
          harness_src.contains(&format!("fn {}", test_name)),
          "FIXTURE_GATED_TESTS entry '{}' has no matching `fn {}` definition in \
           tests/iec104_e2e_real_pcaps_tests.rs — the test was renamed or removed; \
           update FIXTURE_GATED_TESTS accordingly",
          test_name, test_name
      );
  }
  // fixture_path() resolver coupling (F-001):
  // A mistyped COMMITTED_SAMPLES const causes this check to fail in CI; an inverted
  // ordering in fixture_path() is caught only on a fixture-bearing host (Task 9 Env A),
  // because local-samples/ is absent in CI.
  // Uses parent() equality (NOT starts_with) to catch local-samples/ as a subdir:
  // starts_with(tests/fixtures/) would pass for tests/fixtures/local-samples/foo.pcap
  // because local-samples/ is a subdirectory of tests/fixtures/. parent() equality
  // asserts the file is DIRECTLY in tests/fixtures/, not in any subdirectory.
  for name in COMMITTED_FIXTURES {
      let resolved = fixture_path(name).unwrap_or_else(|| panic!(
          "[iec104-e2e] fixture_path('{}') returned None for a COMMITTED_FIXTURES entry — \
           COMMITTED_SAMPLES resolver is broken or the ordering in fixture_path() is inverted",
          name
      ));
      let committed_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
      assert_eq!(
          resolved.parent(),
          Some(committed_dir.as_path()),
          "[iec104-e2e] fixture_path('{}') resolved to {:?} — parent dir must be \
           tests/fixtures/ exactly (not tests/fixtures/local-samples/ or any other \
           subdirectory); COMMITTED_SAMPLES ordering may be inverted or const is wrong",
          name, resolved
      );
  }
  // Forbidden-committed negative guard (F-P10-005):
  // For every FIXTURE_MANIFEST entry NOT in COMMITTED_FIXTURES, assert it is absent
  // from tests/fixtures/ (the committed path). Catches accidental commits of
  // non-redistributable or origin-unclear captures (Wireshark "not redistributed",
  // iec104-iti-dissect.pcap Wireshark dissector test suite origin — F-009 D-524 ruling).
  // Fails exactly when a forbidden capture is dropped in tests/fixtures/.
  for name in FIXTURE_MANIFEST {
      if !COMMITTED_FIXTURES.contains(name) {
          let forbidden_path = Path::new(env!("CARGO_MANIFEST_DIR"))
              .join("tests/fixtures").join(name);
          assert!(
              !forbidden_path.exists(),
              "[iec104-e2e] LICENSING/REDISTRIBUTION VIOLATION: '{}' is present in \
               tests/fixtures/ but is NOT in COMMITTED_FIXTURES — this file MUST NOT \
               be committed; Wireshark captures and origin-unclear files are prohibited \
               from redistribution (see Background §Must NOT commit). Remove the file \
               from tests/fixtures/ and place it only in tests/fixtures/local-samples/.",
              name
          );
      }
  }
  // Call-site count assertion (F-P10-009 / F-P11-001):
  // Counts call sites for the fixture-gating function. The needle is built via concat!
  // so this file's own prose CANNOT match it (source-self-scanning guard: a literal
  // needle appearing in comments or assertions inside the scanned file inflates the count,
  // causing FALSE FAILURES — the concat! split prevents this by ensuring the contiguous
  // needle never appears in this file's source text).
  // HAZARD: a source-self-scanning predicate risks FALSE FAILURES from prose/comment
  // occurrences of the needle literal. The concat! split plus a no-prose-occurrence
  // discipline prevents this. Verified: the prescribed code block above contains ZERO
  // contiguous occurrences of the needle — grep confirms.
  // REAL failure conditions:
  //   (a) A new fixture-gated test is added WITHOUT registering it in FIXTURE_GATED_TESTS
  //       → count becomes 5 vs len 4 → assertion fires (new call site not in registry).
  //   (b) A registered test's call is removed without updating FIXTURE_GATED_TESTS
  //       → count becomes 3 vs len 4 → assertion fires (stale registry entry).
  //   (c) The fixture-gating function is renamed without updating FIXTURE_GATED_TESTS
  //       → count becomes 0 vs len 4 → assertion fires (needle no longer matches).
  //   (d) BYPASS: a gated test calling fixture_present(name_var) with a NON-LITERAL
  //       variable never matches the needle (concat!("fixture_present", "(\"") requires
  //       the literal open-quote immediately after the paren) → count stays unchanged →
  //       this assertion does NOT fire for that unregistered test.
  //       Mitigation: the bidirectional manifest/registry set-equality checks above
  //       (manifest⊆gated and gated⊆manifest directions) + the fn-name coupling loop
  //       still catch the fixture name (if it appears in FIXTURE_MANIFEST) and the
  //       test function name (if absent from FIXTURE_GATED_TESTS fn-coupling assertion).
  //       Residual exposure: a test using a non-literal call whose fixture name is NOT
  //       in FIXTURE_MANIFEST is invisible to all three mechanisms — genuinely undetected.
  let needle = concat!("fixture_present", "(\"");
  assert_eq!(
      harness_src.matches(needle).count(),
      FIXTURE_GATED_TESTS.len(),
      "Call-site count for the fixture-gating function != FIXTURE_GATED_TESTS.len() — \
       a new fixture-gated test was added without a FIXTURE_GATED_TESTS registry entry, \
       or a registered test's call was removed; update FIXTURE_GATED_TESTS accordingly"
  );
  ```
- Then `test_fixture_manifest_report()` FAILS with a clear assertion message when the
  committed capture is absent from `tests/fixtures/` — the panic text appears in CI test
  failure output (always visible, regardless of capture mode and regardless of whether
  local-samples is present)

**Partition semantics:**
- Committed fixtures (`COMMITTED_FIXTURES` array, tracked in `tests/fixtures/`):
  hard assert — absent → test FAILS (CI red — visible because panics bypass stdout capture)
- Gitignored corpus (iec104.pcap, iec104-sq.pcapng, iec104-iti-dissect.pcap in `tests/fixtures/local-samples/`):
  advisory only — absent → stdout FIXTURE-SKIPPED notice (visible only with --nocapture),
  test still passes

This is the structural prevention for PG-W85-005: committed fixture absence is always a
visible CI failure (panic); gitignored corpus absence is an advisory notice only.

Verification:
```bash
set -euo pipefail
# On a fresh clone with committed captures present (with OR without local-samples) — must pass:
cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact 2>&1 | grep -E "1 passed"
# Must show: test result: ok. 1 passed (direct Path::exists() check works regardless of local-samples)

# Verify the hard-assert fires by temporarily renaming a committed capture:
# (manual test only — do not automate file removal in CI)
[ ! -e /tmp/iec104-iti-diverse.pcap.bak ] || { echo "backup path occupied — clean up first"; exit 1; }
mv tests/fixtures/iec104-iti-diverse.pcap /tmp/iec104-iti-diverse.pcap.bak
# Expected: cargo test MUST FAIL (hard-assert fires); inverted-gate form captures failure:
if cargo test --test iec104_e2e_real_pcaps_tests \
     iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact > red-out.txt 2>&1; then
  # cargo test passed — hard-assert did NOT fire — this is the failure case:
  mv /tmp/iec104-iti-diverse.pcap.bak tests/fixtures/iec104-iti-diverse.pcap
  echo "FAIL: expected hard-assert to fire with the committed capture moved aside"; exit 1
fi
# cargo test failed as expected; assert the right message appeared:
grep -qF "REGRESSION: committed fixture 'iec104-iti-diverse.pcap' is absent" red-out.txt \
  || { mv /tmp/iec104-iti-diverse.pcap.bak tests/fixtures/iec104-iti-diverse.pcap; \
       echo "FAIL: test failed for the wrong reason (expected REGRESSION message absent)"; exit 1; }
mv /tmp/iec104-iti-diverse.pcap.bak tests/fixtures/iec104-iti-diverse.pcap  # restore
```

### AC-182-006 (traces to PG-W85-005 — governance-surface completeness)

The governance surfaces touched by this story are all present and consistent.

- Then `tests/fixtures/E2E-PCAPS.md` records the committed-capture annotation and does not
  still claim all captures are auto-fetchable only:
  ```bash
  set -euo pipefail
  grep -qF 'committed at `tests/fixtures/`' tests/fixtures/E2E-PCAPS.md
  test "$(awk '/^## IEC 60870-5-104/{f=1;next} /^## /{f=0} f' tests/fixtures/E2E-PCAPS.md \
    | grep -c 'All are auto-fetchable via `bin/fetch-e2e-pcaps`')" -eq 0
  # (IEC-104 section only — :279 is the ENIP section intro, a true statement, do NOT edit)
  ```
- And `tests/fixtures/README.md` records the committed-capture provenance:
  ```bash
  grep -qF 'iec104-iti-diverse.pcap' tests/fixtures/README.md
  ```
- And both `coverage-out.txt` and `red-out.txt` are listed in `.gitignore`:
  ```bash
  set -euo pipefail
  grep -qF 'coverage-out.txt' .gitignore
  grep -qF 'red-out.txt' .gitignore
  ```
- And `.factory/maintenance/fixture-count-gate-entry.md` is referenced in `CLAUDE.md`:
  ```bash
  grep -qF '.factory/maintenance/fixture-count-gate-entry.md' CLAUDE.md
  ```
- And `.factory/maintenance/fixture-count-gate-entry.md` exists on the factory-artifacts branch
  (environment: any checkout where the `factory-artifacts` ref has been fetched
  (`git fetch origin factory-artifacts`); reads the object store, not the working tree, so
  the `.gitignore:4` exclusion of `.factory/` does not affect it):
  ```bash
  git cat-file -e factory-artifacts:maintenance/fixture-count-gate-entry.md
  ```
- And the additive CI step for fixture coverage reporting is present in `.github/workflows/ci.yml`:
  ```bash
  set -euo pipefail
  grep -qF 'IEC-104 fixture coverage report (visible)' .github/workflows/ci.yml
  grep -qF 'Fixture coverage: [1-9][0-9]*/[0-9]+' .github/workflows/ci.yml
  grep -qF '!cancelled()' .github/workflows/ci.yml
  ```
- And no doc-comment in `tests/iec104_e2e_real_pcaps_tests.rs` still asserts the silent-skip semantics:
  ```bash
  set -euo pipefail
  test "$(grep -c 'keeps CI green' tests/iec104_e2e_real_pcaps_tests.rs)" -eq 0
  # Must be 0: currently returns 2 (lines :12 and :62); both rewritten per Task 7
  ```

## Architecture Mapping

| Component | File | Branch |
|-----------|------|--------|
| `fixture_path()` shared resolver + `COMMITTED_SAMPLES` const | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| `fixture_present()` updated to use `fixture_path()` | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| `run_iec104_pipeline()` updated to use `fixture_path()` | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| `FIXTURE_MANIFEST` const + `COMMITTED_FIXTURES` const + `FIXTURE_GATED_TESTS` const (all inside `mod iec104_e2e_real_pcaps`; `FIXTURE_GATED_TESTS` maps test function names to fixture filenames enabling bidirectional manifest-coupling assertions) | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| `test_fixture_manifest_report()` with hard-assert partition | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| Committed ITI capture (timed-command BC-2.19.029/030) | `tests/fixtures/iec104-iti-diverse.pcap` (new binary) | develop |
| Sweep E2E-PCAPS.md stale loci: document `iec104-iti-diverse.pcap` as committed and `iec104-iti-dissect.pcap` as gitignored; amend IEC-104 section intro (:337-340), Captures table :358 row (:352-359), Attribution (:374-380), document-scope (:3-6), committed-captures section (:48-50), and Adding-a-capture procedure (:391-396) | `tests/fixtures/E2E-PCAPS.md` (amend) | develop |
| Provenance row + attribution + CC-BY-4.0 licensing sweep | `tests/fixtures/README.md` (amend) | develop |
| Additive CI step "IEC-104 fixture coverage report (visible)" | `.github/workflows/ci.yml` (amend — additive step only; no functional job changes) | develop |
| Project References row for `.factory/maintenance/fixture-count-gate-entry.md` | `CLAUDE.md` (amend) | develop |
| Gate-entry evidence doc (manifest counts, `#[ignore]` rejection, D-510 G1 retrospective, enforcement obligation) | `.factory/maintenance/fixture-count-gate-entry.md` (new) | factory-artifacts |
| `coverage-out.txt` and `red-out.txt` gitignore entries (transient artifacts: CI step + manual RED demo) | `.gitignore` (amend — add two entries) | develop |

**No `src/` changes, no `bin/` changes, no `Cargo.toml` changes.**
CHANGELOG obligation: `tests/`, `.github/workflows/ci.yml` (additive run step), `CLAUDE.md`, and `.factory/` are all **excluded** from the AC-158-001 changelog-gate trigger set.
**No CHANGELOG entry required.**

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `tests/fixtures/iec104-iti-diverse.pcap` absent (broken checkout, git LFS not pulled, etc.) | `fixture_path()` MAY return `Some(local-samples/iec104-iti-diverse.pcap)` if the corpus was fetched locally — but the hard-assert fires via a DIRECT `Path::exists()` check on `tests/fixtures/iec104-iti-diverse.pcap`, not via `fixture_path()`. The direct check correctly fails regardless of local-samples presence, producing a visible CI failure. |
| EC-002 | `iec104-iti-diverse.pcap` exists in both `tests/fixtures/` and `tests/fixtures/local-samples/` | `fixture_path()` returns `tests/fixtures/` path (checked first); no double-run; safe |
| EC-003 | Wireshark captures (iec104.pcap, iec104-sq.pcapng) and iec104-iti-dissect.pcap not present in clean checkout | Only 1/4 fixtures present (iec104-iti-diverse.pcap committed); gitignored captures absent → stdout FIXTURE-SKIPPED notice (visible with --nocapture) AND `[iec104-e2e] SKIP:` stderr notice when each fixture's own test runs via `fixture_present()` eprintln; test still passes (gitignored partition). iec104.pcap, iec104-sq.pcapng, and iec104-iti-dissect.pcap each emit both stdout FIXTURE-SKIPPED (from manifest test's absent-list) and stderr SKIP (from their individual test's `fixture_present()` call at :529 for iec104-iti-dissect.pcap) |
| EC-004 | The committed ITI capture (`iec104-iti-diverse.pcap`) > 100 KB at commit time | UNREACHABLE IN PRACTICE given the recorded 14 KB size (E2E-PCAPS.md:358). IF ever exercised: commit is BLOCKED; implementer must re-derive or truncate; additionally obligated to co-update: re-record sha256 in E2E-PCAPS.md, re-derive expected finding count (currently 66), and update README attribution note. |
| EC-005 | `test_fixture_manifest_report()` run in CI without --nocapture | Passes silently when committed fixtures are present. Hard-assert panic (committed fixture absent) IS visible in CI regardless of capture mode. Advisory println!() lines are NOT visible — this is expected behavior. |
| EC-006 | `fixture_path()` called for a file not in COMMITTED_SAMPLES or LOCAL_SAMPLES | Returns None; `fixture_present()` prints `[iec104-e2e] SKIP:` to stderr; calling test returns early (passes silently) |
| EC-007 | Stale-assertion D-510 class: test on fixture-bearing host but with wrong expected count | Hard to prevent structurally; STORY-180 gate-fix PR #439 updated 31→66. COMMITTED_FIXTURES hard-assert prevents ABSENCE; it does not prevent wrong fixture content. Expectation updates are the implementer's obligation after fixture content changes. |
| EC-008 | `test_fixture_manifest_report()` self-read via `CARGO_MANIFEST_DIR` in vendored/packaged builds | The harness self-read (`read_to_string` of own source via `CARGO_MANIFEST_DIR` / `tests/iec104_e2e_real_pcaps_tests.rs`) requires the source file to be present in the source tree at test time. In environments shipping only the compiled `target/` directory without source (vendored builds, pre-built CI artifacts), the `read_to_string` call fails and the manifest test panics at the source-read step. **Accepted for this repo's CI** (source tree always present in CI; no `#[ignore]` is added — consistent with this story's `#[ignore]` rejection rationale). If the test is ever run in a no-source environment, the fail message will cite the `read_to_string` step, not an assertion failure. |

## Tasks

1. **Fetch the ITI captures into local-samples and verify sizes + integrity (AC-182-002
   pre-commit gate — `tests/fixtures/local-samples/` is absent in a fresh clone; may
   already be present on a fixture-bearing host; must be fetched before Task 2; Task 9
   Env A also requires local-samples to be present):** For each file below, only fetch if
   the file is missing OR its sha256 mismatches the expected value; always fetch to a
   temp path and move into place only after the hash verifies (never clobber a verified
   local copy with an unverified download):
   ```bash
   set -euo pipefail
   # Step 1a: Create local-samples directory if it doesn't exist:
   mkdir -p tests/fixtures/local-samples/

   # Step 1b: Fetch iec104-iti-diverse.pcap (WILL BE COMMITTED — see Task 2):
   # Gate: only fetch if missing OR sha256 mismatches; fetch to temp then move after verification.
   DIVERSE_SHA="07b9a0879dc83e420c4cf83b37fb5830d1d8fb5f6ac6edc435896f70b0fc6bc7"
   DIVERSE_DEST="tests/fixtures/local-samples/iec104-iti-diverse.pcap"
   if [ ! -f "$DIVERSE_DEST" ] || \
      ! shasum -a 256 "$DIVERSE_DEST" | grep -q "$DIVERSE_SHA"; then
     curl -L -o /tmp/iec104-iti-diverse.pcap.tmp \
       https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/IEC60870-5-104/090813_diverse.pcap
     shasum -a 256 /tmp/iec104-iti-diverse.pcap.tmp | grep -q "$DIVERSE_SHA" || \
       { echo "sha256 mismatch on diverse — aborting"; rm /tmp/iec104-iti-diverse.pcap.tmp; exit 1; }
     mv /tmp/iec104-iti-diverse.pcap.tmp "$DIVERSE_DEST"
   else
     echo "iec104-iti-diverse.pcap present and verified — skipping fetch"
   fi

   # Step 1c: Fetch iec104-iti-dissect.pcap (LOCAL-SAMPLES ONLY — NOT committed per F-009;
   # upstream filename indicates Wireshark dissector test suite origin — stays gitignored):
   # Gate: fetch if missing OR sha256 mismatches; verify download against E2E-PCAPS.md:359.
   # sha256 IS recorded at E2E-PCAPS.md:359 — gate verifies the DOWNLOAD, not committed state.
   DISSECT_SHA="292c18a8765db3b1bcaa9bd0b8455e4e61b8366cc5910a7363b7381eb11441b8"
   DISSECT_DEST="tests/fixtures/local-samples/iec104-iti-dissect.pcap"
   if [ ! -f "$DISSECT_DEST" ] || \
      ! shasum -a 256 "$DISSECT_DEST" | grep -q "$DISSECT_SHA"; then
     curl -L -o /tmp/iec104-iti-dissect.pcap.tmp \
       https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/IEC60870-5-104/TestDissectIec104.pcap
     shasum -a 256 /tmp/iec104-iti-dissect.pcap.tmp | grep -q "$DISSECT_SHA" || \
       { echo "sha256 mismatch on dissect — aborting"; rm /tmp/iec104-iti-dissect.pcap.tmp; exit 1; }
     mv /tmp/iec104-iti-dissect.pcap.tmp "$DISSECT_DEST"
   else
     echo "iec104-iti-dissect.pcap present and verified — skipping fetch"
   fi

   # Step 1d: Verify sizes — diverse must be ≤ 100 KB before committing:
   test "$(wc -c <"tests/fixtures/local-samples/iec104-iti-diverse.pcap")" -le 102400
   # (portable: wc -c works on macOS and Linux; stat -f%z is macOS-only)

   # Step 1e: Verify sha256 integrity against E2E-PCAPS.md recorded values:
   test "$(shasum -a 256 tests/fixtures/local-samples/iec104-iti-diverse.pcap | cut -d' ' -f1)" = "07b9a0879dc83e420c4cf83b37fb5830d1d8fb5f6ac6edc435896f70b0fc6bc7"

   # Step 1f: Fetch the Wireshark pair into local-samples (required for Task 9 Env A
   # 4/4 fixture coverage — avoids ~400 MB full corpus; 4 small files total):
   # Gate: only fetch if missing OR sha256 mismatches; fetch to temp then move after verification.
   IEC104_SHA="a78aa971adc51e54413a865937f1799ef57118d397cef57ccd93a358ed5b85d6"
   IEC104_DEST="tests/fixtures/local-samples/iec104.pcap"
   if [ ! -f "$IEC104_DEST" ] || \
      ! shasum -a 256 "$IEC104_DEST" | grep -q "$IEC104_SHA"; then
     curl -L -o /tmp/iec104.pcap.tmp \
       https://gitlab.com/wireshark/wireshark/-/wikis/uploads/__moin_import__/attachments/SampleCaptures/iec104.pcap
     shasum -a 256 /tmp/iec104.pcap.tmp | grep -q "$IEC104_SHA" || \
       { echo "sha256 mismatch on iec104.pcap — aborting"; rm /tmp/iec104.pcap.tmp; exit 1; }
     mv /tmp/iec104.pcap.tmp "$IEC104_DEST"
   else
     echo "iec104.pcap present and verified — skipping fetch"
   fi

   SQ_SHA="f855a11326f7aa4f719b1fbb65e5f8dfe3d9d194185a8f5faf5b5dc3cb831227"
   SQ_DEST="tests/fixtures/local-samples/iec104-sq.pcapng"
   if [ ! -f "$SQ_DEST" ] || \
      ! shasum -a 256 "$SQ_DEST" | grep -q "$SQ_SHA"; then
     curl -L -o /tmp/iec104-sq.pcapng.tmp \
       https://gitlab.com/wireshark/wireshark/-/wikis/uploads/__moin_import__/attachments/SampleCaptures/IEC104_SQ.pcapng
     shasum -a 256 /tmp/iec104-sq.pcapng.tmp | grep -q "$SQ_SHA" || \
       { echo "sha256 mismatch on iec104-sq.pcapng — aborting"; rm /tmp/iec104-sq.pcapng.tmp; exit 1; }
     mv /tmp/iec104-sq.pcapng.tmp "$SQ_DEST"
   else
     echo "iec104-sq.pcapng present and verified — skipping fetch"
   fi

   # Step 1g: Final sha256 verification pass (values from bin/fetch-e2e-pcaps:154,157):
   test "$(shasum -a 256 tests/fixtures/local-samples/iec104.pcap | cut -d' ' -f1)" = "a78aa971adc51e54413a865937f1799ef57118d397cef57ccd93a358ed5b85d6"

   test "$(shasum -a 256 tests/fixtures/local-samples/iec104-sq.pcapng | cut -d' ' -f1)" = "f855a11326f7aa4f719b1fbb65e5f8dfe3d9d194185a8f5faf5b5dc3cb831227"
   ```
   E2E-PCAPS.md records all 4 files as small (ITI: 14 KB + 11 KB; Wireshark pair: well
   under 100 KB each). All 4 fetched directly — avoids the ~400 MB full corpus.
   If iec104-iti-diverse.pcap exceeds 100 KB, do NOT commit — re-derive. A sha256 mismatch
   means the file was corrupted in transit — do NOT commit.

2. **Copy ONLY iec104-iti-diverse.pcap to `tests/fixtures/` and add README provenance row
   (AC-182-002; iec104-iti-dissect.pcap is NOT copied or committed per F-009):**
   ```bash
   cp tests/fixtures/local-samples/iec104-iti-diverse.pcap tests/fixtures/
   ```
   Add a provenance row to `tests/fixtures/README.md` using the existing provenance table
   format at lines 30–34 as the model. Include the direct download URL (from E2E-PCAPS.md
   §IEC-104 Direct download URLs table):
   - `iec104-iti-diverse.pcap`: upstream `090813_diverse.pcap` at
     `https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/IEC60870-5-104/090813_diverse.pcap`
   Full attribution text WITH the Source: sentence belongs ONLY in `tests/fixtures/README.md`
   §Licensing notice (lines 7–26 sweep, Task 7) — single authoritative destination per
   P18-004 ruling. The Notes cell in the provenance row carries only
   `Attribution: see §Licensing notice`. The provenance row MUST cite "upstream `LICENSE.md`
   (CC-BY-4.0, repo-level)" as the license source (per AC-182-002 license precondition).

   After copying, verify sha256 integrity against E2E-PCAPS.md:358 (F-003):
   ```bash
   set -euo pipefail
   test "$(shasum -a 256 tests/fixtures/iec104-iti-diverse.pcap | cut -d' ' -f1)" = "07b9a0879dc83e420c4cf83b37fb5830d1d8fb5f6ac6edc435896f70b0fc6bc7"
   ```
   A hash mismatch means the file was corrupted in transit — do NOT commit.

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

6. **Add `FIXTURE_MANIFEST`, `COMMITTED_FIXTURES`, and `FIXTURE_GATED_TESTS` constants inside
   `mod iec104_e2e_real_pcaps`, and add `test_fixture_manifest_report()` inside that same
   module (AC-182-001/005; DF-TEST-NAMESPACE-001):** All three constants go inside
   `mod iec104_e2e_real_pcaps` (alongside the existing `use std::path::Path` at :39;
   NOT inside the test function). `test_fixture_manifest_report()` MUST also be inside
   `mod iec104_e2e_real_pcaps`. Add the combined manifest-report + hard-assert test function.
   The test MUST:
   (a) Print the coverage summary to stdout (visible with --nocapture; advisory)
   (b) Print per-absent-fixture FIXTURE-SKIPPED lines to stdout (visible with --nocapture; advisory)
   (c) Hard-assert on each `COMMITTED_FIXTURES` entry via direct `Path::exists()` on
       `tests/fixtures/{name}` (test FAILS in CI if any committed fixture absent from
       committed path — direct check, not `fixture_path()` which also checks local-samples)
   (d) Non-self-referential manifest sanity checks (canonical loops in AC-182-005 — F-013):
       (i) assert every COMMITTED_FIXTURES entry exists on disk via direct Path::exists() in
       tests/fixtures/; (ii) assert FIXTURE_MANIFEST is a superset of COMMITTED_FIXTURES;
       (iii) assert fixture_path() resolves every COMMITTED_FIXTURES entry to a path under
       tests/fixtures/ (couples resolver to committed set — F-001). See AC-182-005 for the
       exact Rust implementation of all three loops.

7. **Sibling-prose sweep (including ALL stale E2E-PCAPS.md loci — F-012):**
   - `tests/fixtures/README.md` lines 7–26 (licensing notice — notice body :7-22, malware clause :24-26): sweep for the CC-BY-4.0 class.
     Update to describe the ITI CC-BY-4.0 class of committed captures in addition to the
     existing Wireshark Foundation de-facto license. Keep licensing notice accurate and complete.
     **This section is the single authoritative destination for the full CC-BY-4.0 attribution
     text: "ICS Security Tools, Illinois Institute of Technology (ITI). Licensed under
     CC-BY-4.0 (https://creativecommons.org/licenses/by/4.0/). Source:
     https://github.com/ITI/ICS-Security-Tools. Renamed from 090813_diverse.pcap; content
     unmodified." Place the full attribution text here (not only in the table Notes cell).**
   - `tests/iec104_e2e_real_pcaps_tests.rs` module docstring (lines 10–13): update "Captures
     live in `tests/fixtures/local-samples/`" to acknowledge `tests/fixtures/` (committed) and
     `tests/fixtures/local-samples/` (gitignored corpus). **Also rewrite the false-green claim
     at :12-13 — "This keeps CI green without fixtures" is FALSE post-story for the committed
     partition: iec104-iti-diverse.pcap is always present in `tests/fixtures/` and CI never
     silently skips it. Rewrite mirroring the :59-62 treatment (committed capture always present
     in CI; gitignored corpus absent in clean checkouts).**
   - `tests/iec104_e2e_real_pcaps_tests.rs` lines :47-49 ("Fixture root — relative to crate
     root" comment block): update the path description to reflect that committed captures
     reside directly in `tests/fixtures/` (the committed path) with `tests/fixtures/local-samples/`
     as the secondary search location. Use committed-first wording.
   - `tests/iec104_e2e_real_pcaps_tests.rs` lines :53-57 (section banner): update the
     fixture-section banner to reflect the dual-location scheme: `tests/fixtures/` for
     committed captures, `tests/fixtures/local-samples/` for the gitignored corpus.
   - `tests/iec104_e2e_real_pcaps_tests.rs` lines :59-62 (doc-comment on the
     fixture-checking helper): the current text says "present in local-samples … keeps CI
     green when gitignored local-samples not populated" — this is false post-story. The
     committed capture is always present in `tests/fixtures/` and CI never silently skips
     it. Rewrite to committed-first wording: the helper first checks `tests/fixtures/`
     (committed captures, always present in CI), then `tests/fixtures/local-samples/`
     (gitignored corpus, absent in clean checkouts). Needle guard: this doc-comment must
     not assemble the contiguous call-site needle used by the count assertion.
   - `run_iec104_pipeline` doc-comment contract (line ~90): update "The file must exist under
     `tests/fixtures/local-samples/`" to reflect the shared `fixture_path()` resolver.
   - `tests/fixtures/E2E-PCAPS.md` — sweep ALL stale loci (F-012):
     - Lines :3-6 (document-scope claim): update to reflect that some captures are now
       committed to `tests/fixtures/` directly, not only in `local-samples/`
     - Lines :48-50 (committed-captures section): update to enumerate `iec104-iti-diverse.pcap`
       as a newly committed capture (add the diverse filename only; `iec104-iti-dissect.pcap`
       remains gitignored — NOT committed per F-009 D-524 ruling; do NOT list it as committed)
     - Lines :337-340 (IEC-104 section intro): amend the "All are auto-fetchable" claim to
       state that one capture (`iec104-iti-diverse.pcap`) is now committed directly in
       `tests/fixtures/` and always available without fetching; the remaining captures
       require `bin/fetch-e2e-pcaps` or manual download into `local-samples/`
     - Lines :352-359 (§Captures table): annotate the :358 `iec104-iti-diverse.pcap` row
       with the **exact literal** inline note `` committed at `tests/fixtures/` `` —
       AC-182-006 asserts this string with `grep -qF`; equivalent paraphrases will fail the
       gate. Do NOT edit `:279` (ENIP section intro — true claim, must remain unchanged);
       add the annotation for the :359 `iec104-iti-dissect.pcap` row as gitignored+CI-downloaded
     - Lines :374-380 (§Attribution): note that `iec104-iti-diverse.pcap` is committed
       (always available) while `iec104-iti-dissect.pcap` remains gitignored and is
       CI-downloaded by `bin/fetch-e2e-pcaps`
     - Lines :391-396 (Adding-a-capture procedure): update to document when a capture goes
       directly in `tests/fixtures/` (committed, ≤100 KB, redistributable license) vs
       `local-samples/` (gitignored, large, non-redistributable)
   - **Dual-presence note (F-012):** `bin/fetch-e2e-pcaps` continues downloading ITI captures
     to `local-samples/` alongside the committed copies. Dual-presence (in both `tests/fixtures/`
     and `tests/fixtures/local-samples/`) is expected and correct: `fixture_path()` returns the
     committed path first (checked before local-samples). The F-005 direct-path predicate in
     `test_fixture_manifest_report()` guards against the committed path being missing. No
     `bin/` changes are required; this is a note-only observation.
   - `tests/fixtures/README.md` lines 41–44 (F-022): the "remaining fixtures … provenance is
     not recorded here" clause partially changes once the ITI provenance row lands. Update to
     accurately reflect the post-story state: "the ITI IEC-104 capture now has recorded
     provenance; the remaining pre-README fixtures do not." Do NOT claim all 25+ committed
     captures have recorded provenance — only the 1 new ITI entry added in this story does.
   - `tests/iec104_e2e_real_pcaps_tests.rs` lines 23–28 (F-022): the fixture-to-test mapping
     table in the module header. Mark the ITI rows as committed (e.g., append "(committed)" or
     update the "Location" column to `tests/fixtures/` for the ITI entries).
     `test_fixture_manifest_report` is deliberately NOT added to this table: the table maps
     tests to pcap files and the manifest test is not pcap-bound (DF-TEST-CITATION-SWEEP-001
     item 4 considered — a test that verifies fixture metadata rather than pcap content is
     outside this table's scope).
   - Per-test license comment at lines ~353–354 (`test_e2e_BC_2_19_iec104_iti_diverse_…`)
     ONLY (F-022): note the committed status (e.g., "Capture committed to
     tests/fixtures/ — always available in CI"). **Leave lines ~503–504
     (`test_e2e_BC_2_19_iec104_iti_dissect_…`) and the dissect mapping-table row (:28)
     unchanged — `iec104-iti-dissect.pcap` remains gitignored and MUST NOT be annotated
     as committed.**
   - **Needle guard (F-P11-001):** No comment or doc-comment edited in
     `tests/iec104_e2e_real_pcaps_tests.rs` may contain the contiguous call-site needle
     text used by the `fixture_present()` count assertion in `test_fixture_manifest_report()`
     — that assertion reads ONLY this file, so only edits to this file's comments matter.
     The needle is constructed via `concat!` to avoid the source file itself matching — any
     prose edit that assembles the needle literally will cause the count assertion to
     over-report and fail. Keep all documentary references to `fixture_present`
     non-contiguous with the call-site argument form.

8. **Create gate-entry artifact (AC-182-005 gate-entry evidence):**
   Create `.factory/maintenance/fixture-count-gate-entry.md` documenting:
   - The FIXTURE_MANIFEST count (4 entries, 1 committed)
   - The COMMITTED_FIXTURES members
   - The `#[ignore]` rejection rationale (F-009 — corrected): libtest DOES report ignored
     tests (per-test `... ignored` line + `N ignored` summary). The correct reason for
     rejecting `#[ignore]` here: (a) committed fixture absence is a BROKEN CHECKOUT that
     MUST FAIL, not a reportable skip; (b) `#[ignore]` is static and cannot be conditional
     on runtime fixture presence without nightly custom harnesses. This story deliberately
     diverges from df-validation's `#[ignore]` recommendation for COMMITTED (not optional)
     fixtures.
   - The D-510 gate G1 FAIL retrospective: initial FAIL was on a fixture-bearing host with a
     stale 31-finding assertion (the correct count was 66 after STORY-180); gate-fix PR #439
     updated the expectation. STORY-182 closes the structural gap so committed fixture absence
     is always a CI-visible failure.
   - **Enforceable wave-gate obligation (F-026):** Before G1 of any wave-gate evaluation that
     includes e2e pcap tests, run:
     ```bash
     set -euo pipefail
     cargo test --test iec104_e2e_real_pcaps_tests \
       iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture | tee coverage-out.txt
     grep -qE "test result: ok" coverage-out.txt          # blocks on hard-assert failure
     grep -qE "Fixture coverage: [0-9]+/4" coverage-out.txt   # M must be 4
     ```
     and record the printed N/M in the gate entry. **`M ≠ 4` blocks gate entry** (manifest-size
     drift not co-updated here). **`N` is recorded as evidence only**, together with the
     environment declaration (local-samples absent / partially populated / fully populated);
     committed-capture absence is blocked by the AC-182-005 hard-assert, which fails
     `cargo test`; the `grep -qE "test result: ok"` in the command above surfaces that failure
     — a non-zero `cargo test` exit blocks gate entry regardless of the printed N/M.
     No `policies.yaml` entry — sibling convention (see `.factory/maintenance/` for analogous
     enforcement docs).

9. **Regression verification (AC-182-004/005) — two-environment protocol (F-006):**

   **Environment A: fixture-bearing host (4/4 case)** — discriminator: with local-samples
   present ALL 4 fixtures are found and the manifest-report shows 4/4 with zero SKIP lines
   including the Wireshark pair. Precondition: run Task 1 (direct curl fetches) so
   local-samples is present:
   ```bash
   set -euo pipefail
   cargo test --test iec104_e2e_real_pcaps_tests \
     iec104_e2e_real_pcaps::test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu -- --exact 2>&1 | \
     grep -E "1 passed"
   # Must match: test result: ok. 1 passed (not silently skip); 66 findings expected

   # Non-vacuous (F-016): must produce 0 SKIP messages for committed fixtures
   # (gating form: tee-to-file then test count; grep -c || true is non-gating — F-P10-003):
   cargo test --test iec104_e2e_real_pcaps_tests \
     iec104_e2e_real_pcaps::test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu -- --exact --nocapture 2>&1 | tee coverage-out.txt
   # Existence guard: coverage-out.txt must be non-empty before count check (prevents false-GREEN when file absent):
   test -s coverage-out.txt
   # SKIP-count check (non-vacuous — file existence asserted above):
   test "$(grep -c '\[iec104-e2e\] SKIP:' coverage-out.txt)" -eq 0
   # Expected: 0 (committed fixture always found; pattern catches ANY iec104-e2e SKIP)

   # iec104-iti-dissect.pcap: NOT committed (F-009); skips in CI (gitignored)
   # In Env A (local-samples present), runs via local-samples — NOT verified here since
   # it is not a committed fixture and not part of the CI gate.

   # Env A discriminator — manifest-report must show 4/4 (all fixtures found):
   # NOTE: this block overwrites coverage-out.txt from the preceding block; the grep-qE
   # below is the sole discriminator. No 2>&1 here because test_fixture_manifest_report
   # never calls fixture_present() — SKIP eprintln! output does not occur and would not
   # reach coverage-out.txt even if it did (no stderr redirect). Correct zero-SKIP forms
   # with 2>&1 are at the non-manifest-report blocks above — see AC-182-003 verification
   # and Task 9 Env A non-manifest-report block (locate by content: the blocks begin with
   # "set -euo pipefail" and contain "tee coverage-out.txt" with "2>&1").
   set -euo pipefail
   cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture | tee coverage-out.txt
   grep -qE "Fixture coverage: 4/4" coverage-out.txt
   grep -qE "test result: ok" coverage-out.txt
   # Must print "Fixture coverage: 4/4 fixtures present (0 fixture-gated tests will be skipped)"
   # set -euo pipefail: explicit for local block (bash does NOT default to -e in terminal).
   # Both greps on file: GATES wave-gate entry — fails if cargo exits non-zero, 4/4 line
   # absent, OR test did not pass ("test result: ok" absent; manifest prints coverage BEFORE
   # asserts, so a failing run can still write "4/4" — second grep prevents false-GREEN).
   ```

   **Environment B: WITHOUT local-samples** — verify 1/4 coverage output (precondition:
   `tests/fixtures/local-samples/` must be absent or moved out of scope):
   ```bash
   set -euo pipefail
   # local-samples may be absent on develop (no worktree corpus populated); the if-guard
   # prevents mv from erroring on a missing source directory, and prevents the trap from
   # attempting to restore a directory that was never moved:
   if [ -d tests/fixtures/local-samples ]; then
     [ ! -e /tmp/ls-bak ] || { echo "backup path occupied — clean up first"; exit 1; }
     mv tests/fixtures/local-samples /tmp/ls-bak  # remove local-samples
     trap 'mv /tmp/ls-bak tests/fixtures/local-samples' EXIT  # unconditional restore under set -e
     set -euo pipefail
     cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture | tee coverage-out.txt
     grep -qE "Fixture coverage: 1/4" coverage-out.txt  # (currently 1/4 — tracks committed-partition/manifest sizes)
     grep -qE "test result: ok" coverage-out.txt
     mv /tmp/ls-bak tests/fixtures/local-samples; trap - EXIT  # restore and disarm trap
   else
     echo "local-samples already absent — skip move-aside"
     set -euo pipefail
     cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture | tee coverage-out.txt
     grep -qE "Fixture coverage: 1/4" coverage-out.txt  # (currently 1/4 — tracks committed-partition/manifest sizes)
     grep -qE "test result: ok" coverage-out.txt
   fi
   ```

10. **Add `CLAUDE.md` Project References row and gitignore transient artifact (F-015 + F-P9-011):**
    (a) Append a row to the Project References table in `CLAUDE.md` for
    `.factory/maintenance/fixture-count-gate-entry.md`, following the pattern of the 6
    existing protocol-doc rows at the bottom of that table:
    ```
    | `.factory/maintenance/fixture-count-gate-entry.md` | Fixture manifest counts, COMMITTED_FIXTURES members, #[ignore] rejection rationale, D-510 G1 retrospective; wave-gate-entry obligation: run manifest test before G1 per Task 8 wave-gate procedure (includes `tee coverage-out.txt` + `grep -qE "test result: ok"`) and record N/M — `M ≠ 4` blocks gate entry (manifest-size drift); `N` is recorded as evidence only with the environment declaration (local-samples absent / partially populated / fully populated); committed-capture absence is blocked by the AC-182-005 hard-assert, which fails `cargo test`; the `grep -qE "test result: ok"` in the Task 8 command surfaces that failure; `N` itself is not a blocking datum |
    ```
    (b) Add `coverage-out.txt` and `red-out.txt` to `.gitignore` — both are transient artifacts
    not meant to be tracked. Append as new top-level entries:
    ```
    # Transient CI artifact from IEC-104 fixture coverage report step (STORY-182)
    coverage-out.txt
    # Transient artifact from the AC-182-005 manual RED demonstration (STORY-182)
    red-out.txt
    ```
    (c) Add the additive CI step "IEC-104 fixture coverage report (visible)" to the test job
    in `.github/workflows/ci.yml`, placed AFTER the `- run: cargo test --all-targets` step
    (approximately at `.github/workflows/ci.yml:47`, inside the `test:` job at `:40-47` —
    re-locate by content match if the file was already edited), with `if: ${{ !cancelled() }}`:
    ```yaml
    - name: IEC-104 fixture coverage report (visible)
      if: ${{ !cancelled() }}
      run: |
        set -euo pipefail
        cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture | tee coverage-out.txt
        grep -qE "Fixture coverage: [1-9][0-9]*/[0-9]+" coverage-out.txt
        grep -qE "test result: ok" coverage-out.txt
    ```
    This is the same step body specified in Architecture Compliance Rules §Additive `ci.yml` step.

11. **Develop PR:** All changes in `tests/`, committed binary fixtures, additive `ci.yml`
    step, `CLAUDE.md` row, and `.gitignore` entries for `coverage-out.txt` and `red-out.txt` — no CHANGELOG
    required (`tests/`, `ci.yml` executable run step, `CLAUDE.md`, and `.gitignore` are all
    excluded from the AC-158-001 trigger set). The PR description MUST include the actual test output proving:
    (a) The 1/4 fixture coverage summary (with --nocapture, WITHOUT local-samples present
        per the Environment B protocol in Task 9 — required to produce the 1/4 output)
    (b) The iec104-iti-diverse.pcap test pass with zero SKIP lines (Environment A,
        fixture-bearing host, using the grep-c output from Task 9 Env A verification).

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
  construction from `LOCAL_SAMPLES` or `COMMITTED_SAMPLES` is permitted for RESOLVING OR
  OPENING a fixture in either function after this story lands; constructing a display-only
  path for the SKIP diagnostic message in `fixture_present()` is explicitly permitted and
  expected (see AC-182-001 prescribed implementation — `committed_path` is used only in the
  `eprintln!()` diagnostic, not for file resolution or opening).
- **`COMMITTED_SAMPLES = "tests/fixtures"`:** Committed captures go directly in `tests/fixtures/`
  alongside the existing committed captures (25 capture files on disk; to verify:
  `git ls-files tests/fixtures/ | grep -cE '\.(pcap|pcapng|cap|trace)$'` — the `*.pcap*`
  glob misses files with `.cap` or `.trace` extensions; previously cited as 21, corrected per
  F-016). No subdirectory created. The `tests/fixtures/` root is NOT covered by any `.gitignore`
  entry — committed files here are naturally tracked.
- **`test_fixture_manifest_report()` pass/fail contract:**
  - Committed fixtures absent → test FAILS via hard-assert panic (CI-visible)
  - Gitignored corpus absent → advisory stdout only (visible with --nocapture) — test still passes
  - Standard CI without --nocapture: stdout NOT visible; hard-assert panic IS visible
- **Constants placement:** All three constants — `FIXTURE_MANIFEST`, `COMMITTED_FIXTURES`, and
  `FIXTURE_GATED_TESTS` — MUST be declared inside `mod iec104_e2e_real_pcaps` (alongside the
  existing `use std::path::Path` at :39), NOT inside the test function. This allows other test
  utilities and future assertions to reference them. `FIXTURE_GATED_TESTS` in particular must be
  at module level because both the count-pin assertion and the per-test name-coupling loop in
  `test_fixture_manifest_report()` reference it from test-function scope.
- **`#[ignore]` prohibition (F-009 — rationale corrected):** Do NOT mark committed fixture
  presence checks with `#[ignore]`. The real reason (not "completely invisible in CI" — that
  is false; libtest reports ignored tests): committed fixture absence is a BROKEN CHECKOUT
  that MUST FAIL, not a reportable skip. `#[ignore]` is static and cannot be made conditional
  on runtime fixture presence without nightly harnesses. Deliberate divergence from
  df-validation's `#[ignore]` recommendation — recorded in Task 8 gate-entry artifact.
- **Needle guard (F-P11-001):** No comment or doc-comment edited in
  `tests/iec104_e2e_real_pcaps_tests.rs` may contain the contiguous `fixture_present()`
  call-site needle used in the count assertion inside `test_fixture_manifest_report()` —
  the count assertion reads ONLY that file, so only edits to that file's comments can
  inflate the count. The needle is built via `concat!` so the story file and test source
  cannot self-match. Any prose edit that assembles the needle contiguously will cause the
  count assertion to over-report and fail CI. Keep all documentary references to
  `fixture_present` non-contiguous with the literal call-site form.
- **Additive `ci.yml` step (F-014 orchestrator ruling) — GATES the test job:** One new step
  is permitted in the existing test job, placed AFTER the main `cargo test --all-targets` step
  with `if: ${{ !cancelled() }}` so it is visible after TEST failures; after
  checkout/compile failures the step runs but cannot emit the coverage line —
  !cancelled() guarantees execution, not output; only workflow cancellation prevents
  this step (unlike `if: always()` which fires even on cancellation). The
  step run block MUST begin with `set -euo pipefail` so a non-zero exit from cargo test or a
  grep miss both fail the step:
  ```
  set -euo pipefail
  cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --exact --nocapture | tee coverage-out.txt
  grep -qE "Fixture coverage: [1-9][0-9]*/[0-9]+" coverage-out.txt
  grep -qE "test result: ok" coverage-out.txt
  ```
  with step name "IEC-104 fixture coverage report (visible)". This step **GATES the CI test
  job**: if no `Fixture coverage: N/M` line is printed (N ≥ 1), the step exits non-zero and
  the job fails. This makes the `Fixture coverage: N/4` println!() output (denominator 4
  currently — tracks `FIXTURE_MANIFEST.len()`) visible after TEST failures (but NOT after
  compile failures — the step's own `cargo test` cannot produce coverage output if the crate
  doesn't build) via `if: ${{ !cancelled() }}` without modifying any functional job steps. The
  step uses only the existing `cargo test` invocation — no new SHA-pinned actions are added.
  `.github/workflows/ci.yml` is NOT in the forbidden list for this additive step only
  (F-014 ruling).

## Library & Framework Requirements

| Dependency | Version | Source |
|------------|---------|--------|
| Rust stable | 1.91+ | CLAUDE.md MSRV |
| No new Cargo.toml deps | — | tests/ only, no new crate dependencies |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `tests/iec104_e2e_real_pcaps_tests.rs` | Modify | Add `COMMITTED_SAMPLES` const, `fixture_path()` fn, update `fixture_present()` and `run_iec104_pipeline()`, add `FIXTURE_MANIFEST`/`COMMITTED_FIXTURES`/`FIXTURE_GATED_TESTS` consts inside `mod iec104_e2e_real_pcaps` (alongside `:39`), add `test_fixture_manifest_report()`, update module docstring :10-13 (false-green claim at :12-13 must be rewritten — "This keeps CI green without fixtures" is FALSE post-story for the committed partition) + pipeline doc-comment; also sweep :47-49 (fixture-root comment — committed-first wording), :53-57 (section banner — dual-location scheme), :59-62 (doc-comment on fixture-checking helper — "local-samples only" claim false post-story; needle guard applies) |
| `tests/fixtures/iec104-iti-diverse.pcap` | New (binary) | ITI CC-BY-4.0; ≤100 KB; exercises TypeIDs 58–64; goes DIRECTLY in tests/fixtures/ |
| `tests/fixtures/README.md` | Modify | Add IEC-104 committed-capture provenance row (licensing notice lines 7–26 sweep + source URL + direct download URL + attribution) |
| `tests/fixtures/E2E-PCAPS.md` | Modify | Sweep all stale loci: IEC-104 section intro (:337-340), Captures table :358 row (:352-359), Attribution (:374-380), document-scope (:3-6), committed-captures section (:48-50), Adding-a-capture procedure (:391-396) — `iec104-iti-diverse.pcap` noted as committed in `tests/fixtures/`; `iec104-iti-dissect.pcap` remains gitignored (CI download-and-verify path only, not committed) |
| `.factory/maintenance/fixture-count-gate-entry.md` | New (factory-artifacts branch — committed by state-manager, NOT in develop PR) | Gate-entry evidence: manifest counts, COMMITTED_FIXTURES members, #[ignore] rejection rationale, D-510 G1 retrospective |
| `.github/workflows/ci.yml` | Modify (additive step only) | Add "IEC-104 fixture coverage report (visible)" step in test job — makes `Fixture coverage:` println!() CI-visible (F-014 orchestrator ruling) |
| `CLAUDE.md` | Modify | Add Project References row for `.factory/maintenance/fixture-count-gate-entry.md` (F-015 process-gap) |
| `.gitignore` | Modify | Add `coverage-out.txt` and `red-out.txt` entries — transient artifacts (CI step + manual RED demo) (F-P9-011, F-W86S-P21-006) |

**Forbidden modifications:** `src/**/*`, `Cargo.toml`, `bin/*`, `CHANGELOG.md`
**Note:** `.github/workflows/ci.yml` is permitted for the ONE additive step described above
(F-014 orchestrator ruling); all functional CI job steps remain unchanged.

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
- **Sibling e2e harnesses (deferred):**
  - `tests/enip_e2e_real_pcaps_tests.rs`: identical `LOCAL_SAMPLES`/`fixture_present()`/silent-skip idiom; its `enip_test.pcap` + `EthernetIP-CIP.pcap` are the SAME ITI CC-BY-4.0 redistributable class — direct analog for a follow-up story.
  - `tests/e2e_corpus_smoke_tests.rs`: directory-level skip VARIANT (lines :206-224) — structurally different from the per-file `fixture_present()` idiom above.
  - `tests/bc_2_12_011_story127_tests.rs`: synthetic fallback — writes a synthetic 16-packet pcapng when the fixture is None and runs full assertions; NOT in the silent-skip class (earlier characterization as "same idiom" was incorrect; STATE.md DRIFT-e2e-sibling-harnesses row corrected at D-532; residual wording alignment (e2e_corpus_smoke as directory-level VARIANT, not same idiom) handled by state-manager at D-533).
  All three deferred from wave-86 (scope containment after 6 adversarial passes); tracked via STATE.md drift row DRIFT-e2e-sibling-harnesses (already present in STATE.md, D-522; no action in this story's PR); follow-up story candidate at next planning.
- **No behavioral contract required:** E-11 convention.
- **tdd_mode: strict — E-11 template note:** `tdd_mode: strict` is satisfied for this
  governance story by the documented manual RED demonstration (move the committed capture
  aside via `mv tests/fixtures/iec104-iti-diverse.pcap /tmp/...`, run
  `cargo test ... test_fixture_manifest_report -- --exact`, observe the hard-assert fire
  citing the missing capture, then restore — see AC-182-005 hard-assert verification block).
  The chosen task ordering (fixture committed at Task 2 before the manifest test at Task 6)
  does not produce an automated RED in the normal TDD loop; the documented manual RED
  (move-capture-aside procedure, re-anchored per F-P11-003) is the accepted substitute for
  THIS story specifically — not an epic-wide assertion about all E-11 governance stories.
- **Develop PR:** All ACs can be batched in a single develop PR. The PR touches: `tests/`
  (test code + binary fixtures), `.github/workflows/ci.yml` (additive step), `CLAUDE.md`
  (Project References row), and `.gitignore` (`coverage-out.txt` and `red-out.txt` entries). No CHANGELOG entry
  required — `.github/`, `CLAUDE.md`, `.gitignore`, and `tests/` are all outside the
  AC-158-001 trigger set; binary fixtures are also excluded.
  **Note:** `.factory/maintenance/fixture-count-gate-entry.md` is NOT in the develop PR.
  It is committed separately to the `factory-artifacts` branch by state-manager (consistent
  with the `.factory/` branch convention per CLAUDE.md).
  **Sibling story (STORY-183) also touches `.github/workflows/ci.yml`** in disjoint
  regions: STORY-182 edits the test job ~:40-47 (baseline develop@e8841d76, pre-STORY-183
  — re-locate by content match if STORY-183 merged first) adding an additive `cargo test`
  run step; STORY-183 edits comment lines :434/:442 + step-name line :462 only (non-functional;
  baseline develop@e8841d76, pre-STORY-182 — re-locate by content match if STORY-182 merged first).
  Merge order is irrelevant for conflict purposes — the two edit regions do not overlap —
  but line anchors are order-dependent. A rebase is required if both PRs are in flight
  simultaneously to avoid merge conflicts from adjacent line changes.
- **Wave-85 gate G1 retrospective:** D-510 was triggered on a **fixture-bearing host** where
  `test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu` ran with the stale assertion
  of 31 findings when the correct count (post-STORY-180 timed-command detection) is 66. This
  was a **stale-assertion failure**, not a clean-worktree silent-skip. Gate-fix PR #439
  (0ab6f52e) updated expectations 31→66. STORY-182 addresses the complementary structural
  gap: committed fixture absence is now a visible CI failure; the stale-assertion class now
  fails CI on every run because the committed ITI fixture always runs in CI (no longer only on
  fixture-bearing hosts); preventing wrong-content stale assertions remains the implementer's
  obligation via accurate expected counts.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 2.12 | 2026-07-28 | story-writer | WAVE-86 PASS-22 REMEDIATION — F-W86S-P22-001 MED (AC-182-006 E2E-PCAPS.md predicate: `sed -n '337,345p' \| grep -c` replaced with content-anchored `awk '/^## IEC 60870-5-104/{f=1;next} /^## /{f=0} f' \| grep -c` form — drift-proof, section-scoped, ENIP intro :279 excluded; `set -euo pipefail` already present in containing fence; AUDIT4 now 0); F-W86S-P22-002 MED (`red-out.txt` predicate added to three formerly-singular loci: (1) AC-182-006 gitignore bullet prose updated to name both entries; `grep -qF 'red-out.txt' .gitignore` predicate added alongside existing sibling; `set -euo pipefail` added to now-two-command fence; (2) Task 11 "entry for `coverage-out.txt`" → "entries for `coverage-out.txt` and `red-out.txt`"; (3) Notes §Develop PR "`coverage-out.txt` entry" → "`coverage-out.txt` and `red-out.txt` entries"; D-539 residual: `grep -c 'entry for.*coverage-out' STORY-182.md` = 0 outside changelog provenance rows — confirmed pre-changelog-row-insertion); F-W86S-P22-003 MED (Task 10c added between Task 10b and Task 11: prescribes adding "IEC-104 fixture coverage report (visible)" step to `.github/workflows/ci.yml` test job with `if: ${{ !cancelled() }}` and the four-line `run:` block from ACR §Additive `ci.yml` step verbatim — closes the implementer gap where the step was declared in 5 ACs/ACR/FSR/Arch-Mapping loci but had no Task delivering it); N-1 NIT (README.md predicate extracted from the E2E-PCAPS.md bullet into its own bullet; `-q` → `-qF` — `.` in filename is a regex wildcard, `-qF` makes it a fixed-string match); AUDIT-5 hardening (Task 2 sha256 fence: `set -euo pipefail` added — already fail-closed: `test "" = "<sha>"` exits 1 on pipeline failure; hardened for consistency, NOT fixing a live false-GREEN per D-530; AUDIT5 now 0). ADDENDUM (2026-09-04, wave-86 human story-approval gate D-544, no version bump): `level: maintenance` → `level: feature` (E-11 convention alignment matching STORY-147/166/176). METADATA-ONLY classification change — `level` is not adversary-reviewed spec content and is not part of the `input-hash` (inputs: unchanged, hash unchanged at 9a0f34c). Adversarial convergence 3/3 (passes 25/26/27, BC-5.39.001 SATISFIED at v2.12) is PRESERVED — no re-convergence required. ADDENDUM (2026-09-05, D-548, no version bump): `status: ready` → `status: delivered` — PR #460 squash-merged to `develop` as `35ffa135`; per-story Step-4.5 adversarial CONVERGED 3/3, pr-reviewer APPROVE (0 blocking), security-reviewer CLEAN (NONE); CI 13/13 green on the rebased branch (base `bd244ddf`, D-547 gate-fix). Gate-entry evidence (`fixture-count-gate-entry.md`), Red Gate log, and per-story convergence report written to factory-artifacts (state-manager). `status` is not a hashed input — canonical hash unchanged at `9a0f34c`. |
| 2.11 | 2026-07-28 | story-writer | WAVE-86 PASS-21 REMEDIATION — F-W86S-P21-007 LOW (Narrative "So that" clause: "eliminated for the IEC-104 harness delivered here" → "eliminated for the committed partition of the IEC-104 harness" with explicit note that the 3 gitignored fixtures emit visible FIXTURE-SKIPPED notices but still report `ok` on absence); F-W86S-P21-002 MED (two `SKIP_COUNT="$(grep -c ...)"` assignment-position forms at AC-182-003 Verification and Task 9 Env A replaced with argument-position `test "$(grep -c ...)" -eq 0` — assignment form propagates grep-c exit status 1 under `set -e` when count=0, causing false abort in the expected-pass case; FIX B restored); F-W86S-P21-003 MED (AC-182-006 E2E-PCAPS.md predicate section-scoped from whole-file to IEC-104 section lines 337-345 via `sed -n '337,345p' \| grep -c`; inline note added: ":279 is ENIP section intro — true, do NOT edit"; Task 7 :352-359 bullet: "e.g." removed, exact literal `` committed at `tests/fixtures/` `` made normative with `grep -qF` gate warning; "Do NOT edit :279" directive added); F-W86S-P21-004 MED (Task 8 wave-gate command made self-gating: added `tee coverage-out.txt` + `grep -qE "test result: ok"` + `grep -qE "Fixture coverage: [0-9]+/4"` inside fenced block; prose reworded to reference "the command above"; Task 10a CLAUDE.md row updated to reference Task 8 wave-gate procedure); F-W86S-P21-005 LOW (AC-182-006 factory-artifacts predicate: "runs from main repo root ONLY" and `.gitignore:4` restriction removed — `git cat-file -e` reads the object store, not the working tree, so it is runnable anywhere the ref is fetched; environment note rewritten accordingly); F-W86S-P21-006 LOW (`red-out.txt` gitignore entry added alongside `coverage-out.txt` at all four loci: Background §Gitignore placement, Architecture Mapping row, Task 10b, and FSR `.gitignore` row). |
| 2.10 | 2026-07-27 | story-writer | WAVE-86 PASS-20 REMEDIATION — F-015 MED (Narrative "So that" clause: "structurally prevented" → "clean-worktree silent-skip class behind PG-W85-005 is structurally eliminated"; "detection, not prevention" caveat added); F-002 MED (AC-182-006: tautological E2E-PCAPS.md predicate `grep -c 'tests/fixtures/' -ge 1` replaced with discriminating `grep -qF 'committed at \`tests/fixtures/\`'` + `grep -c 'All are auto-fetchable via \`bin/fetch-e2e-pcaps\`' -eq 0`); F-003 MED (AC-182-006: additive ci.yml step predicate block added with three `grep -qF` gates for step name, coverage pattern, and `!cancelled()` condition); F-004 MED (AC-182-005 Verification: `cargo test ... \|\| true` no-assert RED demo replaced with inverted-gate form + `grep -qF "REGRESSION: committed fixture ..." red-out.txt` positive assertion; `set -euo pipefail` added); F-006 MED (six missing `set -euo pipefail` first lines added: AC-182-001 Verification, AC-182-003 Verification, AC-182-004 Verification, AC-182-005 Verification, Task 9 Env A, Task 9 Env B); F-007 MED (Task 8 Enforceable wave-gate obligation and Task 10a CLAUDE.md row rewritten: self-contradictory "`N < 1` blocks" / "`N` is evidence only" contradiction resolved — single coherent rule: `M ≠ 4` blocks, `N` is recorded as evidence only with environment declaration); F-008 MED (AC-182-006: environment-blind `test -f .factory/maintenance/fixture-count-gate-entry.md` replaced with environment note + `git cat-file -e factory-artifacts:maintenance/fixture-count-gate-entry.md`); F-010 MED (AC-182-006: "The four governance deliverables" → "The governance surfaces touched by this story"); F-013 LOW (two `:11-12` line anchors corrected to `:12-13`: Task 7 bullet and FSR Notes cell); AUDIT-2 (two tautological SKIP-count predicates at AC-182-003 Verification and Task 9 Env A fixed: `test "$(grep -c ...)" -eq 0` replaced with `test -s coverage-out.txt` guard + variable-assignment form to prevent false-GREEN when coverage-out.txt absent). |
| 2.9 | 2026-07-27 | story-writer | WAVE-86 PASS-19 REMEDIATION — F-001 MED (Task 7 bullet 2: extended to prescribe rewriting false-green claim at :11-12 — "This keeps CI green without fixtures" is FALSE post-story for the committed partition; rewrite mirrors :59-62 treatment; FSR Notes cell for tests/iec104_e2e_real_pcaps_tests.rs updated to enumerate :10-13 with false-green note); F-002 MED (two missing `set -euo pipefail` added: (a) first line of AC-182-002 Verification bash block, after ```bash fence before git ls-files; (b) first line of Task 1 bash block, before # Step 1a:); F-003 MED (Task 8 Enforceable wave-gate obligation paragraph rewritten — environment-explicit N/M: N MUST equal 1 when local-samples absent or 4 when fully populated; intermediate N is legitimate on partially-populated host; blocking condition is M ≠ 4 or N < 1 only — corrects old "any other N/M pair" language that would have blocked N=4 on a fixture-bearing host; Task 10a CLAUDE.md row cell text rewritten to match same enforcement posture); F-004 MED (AC-182-006 added after AC-182-005 and before Architecture Mapping — governance-surface completeness AC covering E2E-PCAPS.md paths reference, .gitignore coverage-out.txt entry, CLAUDE.md fixture-count-gate-entry.md reference, factory-artifacts file existence, and false-green doc-comment elimination). |
| 2.8 | 2026-07-27 | story-writer | WAVE-86 PASS-18 REMEDIATION — P18-003 MED (Task 7 sweep: added three new test-file bullets for :47-49 fixture-root comment, :53-57 section banner, :59-62 doc-comment on fixture-checking helper — all prescribe committed-first rewording, needle guard applies to :59-62; FSR row Notes updated to enumerate all three ranges); P18-004 MED (attribution single-destination: Notes cell at :416 reduced from full attribution text to `Attribution: see §Licensing notice` + kept `upstream LICENSE.md (CC-BY-4.0, repo-level)` clause; prose at :418-420 updated to match; Task 2 :931-936 rewritten to state full attribution WITH Source: sentence lives ONLY in README §Licensing notice — single authoritative destination per P18-004 ruling; "7-22" → "7-26" propagated in Task 2 rewrite); P18-005 MED (AC-182-004 outcome (e) :515-517 and ACR ci.yml step :1219: corrected `!cancelled()` visibility claim — "visible after TEST failures; after checkout/compile failures the step runs but cannot emit the coverage line — !cancelled() guarantees execution, not output"); P18-006 MED (Task 8 :1064-1065: replaced tautology "M MUST equal FIXTURE_MANIFEST.len()" with discriminating predicate "M MUST equal 4 (the literal) and N MUST equal COMMITTED_FIXTURES.len() (currently 1) in a clean checkout; any other N/M pair blocks gate entry"; CLAUDE.md row :1142 updated to match); P18-008 LOW (three live "7-22" loci corrected to "7-26": background :115 with added explanation of malware clause :24-26; Task 7 :983; FSR :1253); NIT-1/P18-010 LOW (AC-182-004 verification :536-539: replaced bare `set -euo pipefail` block with full if [ -d tests/fixtures/local-samples ] guard form matching Env-B siblings); NIT-2/P18-011 LOW (Task 7 needle guard :1039-1044 rescoped from "Task 7 sweep targets" to "tests/iec104_e2e_real_pcaps_tests.rs" — count assertion reads only that file; ACR needle guard :1211-1216 rescoped from "any file touched by this story" to "tests/iec104_e2e_real_pcaps_tests.rs"). |
| 2.7 | 2026-07-26 | story-writer | WAVE-86 PASS-17 REMEDIATION — P17-001 MED (Task 9 Env A :1069 `set -o pipefail` → `set -euo pipefail`; already applied in prior burst); P17-002 MED (5 Env-B blocks hardened to tee+grep-qE "Fixture coverage: 1/4"+grep-qE "test result: ok" gating form; applied in prior burst); P17-003a MED (README table Notes cell: folded full CC-BY attribution text into cell; added §Licensing notice pointer); P17-003b MED (freestanding attribution block after code fence replaced with destination pointer note); P17-003c MED (Task 2 attribution block replaced with destination pointer to README §Licensing notice); P17-003d MED (Task 7 README lines 7–22 bullet: added explicit directive to place full attribution text there as single authoritative destination); P17-004a MED (wave-gate obligation rewritten: N/M is evidence artifact not independent gate; M MUST equal FIXTURE_MANIFEST.len(); absence detected by AC-182-005 hard-assert / cargo test failure); P17-004b MED (CLAUDE.md row updated to match new wave-gate semantics); P17-005a MED (Task 7 needle guard bullet added after last sweep bullet — no edited comment may contain contiguous call-site needle text); P17-005b MED (ACR needle guard rule added before Additive ci.yml step rule); P17-008a MED (outcome (e) `!cancelled()` rationale corrected: visible after TEST failures AND checkout failures; only cancellation prevents the step); P17-008b MED (ACR `!cancelled()` rationale corrected to match); P17-009 LOW (co-update comment extended with "1/4 expected-value literals (Env B blocks, Task 8 obligation, EC rows)"); P17-010 LOW (git ls-files hardened to `git ls-files --error-unmatch tests/fixtures/iec104-iti-diverse.pcap`); NIT-1 (v2.6 changelog row: ".gitignore has no transient-file entries" corrected — .gitignore already has `mutants.out*/`). |
| 2.6 | 2026-07-26 | story-writer | WAVE-86 PASS-16 REMEDIATION — F-W86S-P16-001 MED (Task 9 Env A :1069 bare `set -o pipefail` → `set -euo pipefail`; this was the byte-identical sibling of the AC-182-003 block upgraded in v2.5 — v2.5 upgrade missed this locus); F-W86S-P16-002 MED (ACR ci.yml step block :1200-1206: (i) `set -o pipefail` → `set -euo pipefail` in prose and code block; (ii) `grep -qE "test result: ok" coverage-out.txt` added to code block so ACR step spec matches AC-182-004 two-grep form); P16-007 LOW (:419-420 "git diff --stat from a fresh clone shows it as a new tracked file" deleted — inert on a fresh clone (clean tree = empty diff); `git ls-files --error-unmatch` gate stands); P16-008 LOW (:1270 stale pointer rewritten: STATE.md:216 locus corrected to D-532-already-applied form; STATE.md line anchor and "state-manager will fix in same burst" language replaced with "STATE.md DRIFT-e2e-sibling-harnesses row corrected at D-532; residual wording alignment (e2e_corpus_smoke as directory-level VARIANT, not same idiom) handled by state-manager at D-533"); NIT-1 (AC-182-002 README row :412: outer `CC-BY-4.0 (upstream \`LICENSE.md\` (CC-BY-4.0, repo-level))` flattened to `upstream \`LICENSE.md\` (CC-BY-4.0, repo-level)` — one occurrence of CC-BY-4.0; AC-182-002 MUST-cite clause :377 matches verbatim); NIT-2 (Task 10(b) :1131: "Add it adjacent to other transient file entries in `.gitignore`" → "Append as a new top-level entry (the file is a transient CI artifact)" — `.gitignore` already has a transient-file entry (`mutants.out*/`); "new top-level entry" is still the correct phrasing but "no transient-file entries" was false). |
| 2.5 | 2026-07-26 | story-writer | WAVE-86 PASS-15 REMEDIATION — F-W86S-P15-001 MED (AC-182-002 prescribed row :404 + Task 2 instruction :909-914: added "upstream `LICENSE.md` (CC-BY-4.0, repo-level)" citation to both loci); F-W86S-P15-002 MED (two local-samples move-asides wrapped in `if [ -d tests/fixtures/local-samples ]; then … else echo "already absent"; <run>; fi` guards at AC-182-001 Env B and Task 9 Env B — prevents mv error and bogus trap-restore when local-samples is absent on develop; committed-pcap move-aside :767-773 left without source guard — file always exists); F-W86S-P15-003 MED (three blocks at AC-182-003, AC-182-004, Task 9 Env A: (i) `set -o pipefail` → `set -euo pipefail`; (ii) `grep -qE "test result: ok" coverage-out.txt` added to AC-182-004 and Env-A discriminator; (iii) AC-182-004 annotation corrected to distinguish CI bash-e-default vs local explicit set -euo); F-W86S-P15-004 MED (Notes sibling-list corrected: enip_e2e_real_pcaps_tests.rs = direct analog; e2e_corpus_smoke_tests.rs = directory-level skip VARIANT :206-224; bc_2_12_011_story127_tests.rs = synthetic fallback — NOT silent-skip class; STATE.md:216 carries same mislabel — state-manager fixes in same burst); F-W86S-P15-008 LOW (needle-count failure conditions: condition (d) added — non-literal fixture_present(name_var) bypasses needle; mitigation via bidirectional manifest/registry set-equality + fn-name coupling; residual exposure stated honestly); F-W86S-P15-009 LOW (`stat -f%z` replaced with portable `wc -c <"file"` at AC-182-002 :432-433 and Task 1 Step 1d :856-857); F-W86S-P15-010 LOW (EC-003: iec104-iti-dissect.pcap added to both-streams enumeration — its test calls fixture_present at :529); NIT-1 (v2.4 changelog row: always() loci corrected :505/:1148 → :506/:1161; move-aside loci corrected :350-354/:766-771/:1062-1068 → :350-355/:767-773/:1074-1082); NIT-2 (Task 7: "remains annotated as gitignored+CI-downloaded" → "add the annotation as gitignored+CI-downloaded"). |
| 2.4 | 2026-07-26 | story-writer | WAVE-86 PASS-14 REMEDIATION — F-W86S-P14-002 MED (4th unswept if: always() locus fixed at :540 → if: ${{ !cancelled() }} with correct visibility wording; full-story always()-sweep: 0 live loci remain — :506/:1161 are explanatory contrast references; changelog rows exempt); F-W86S-P14-003 MED (Task 7 E2E-PCAPS.md sweep list extended with three missing loci: :337-340 IEC-104 section intro, :352-359 Captures table :358 row, :374-380 Attribution section; Arch Mapping and FSR E2E-PCAPS.md rows updated for consistency — enumerate all 6 sweep loci); F-W86S-P14-006 LOW (three move-aside procedures at :350-355, :767-773, :1074-1082 each have pre-existence guard added — `[ ! -e <backup-path> ] \|\| { echo "backup path occupied — clean up first"; exit 1; }` before the mv; approach consistent across all three). |
| 2.3 | 2026-07-26 | story-writer | WAVE-86 PASS-13 REMEDIATION — F-W86S-P13-002 HIGH (three stale self-anchors fixed: (a) :1216 tdd_mode note removed stale ":740-745" ref, replaced with description-based reference to AC-182-005 hard-assert block; (b)+(c) :1045 stale ":461-463 / :1008-1010" replaced with content-based locators — "locate by content: set -o pipefail ... tee coverage-out.txt ... 2>&1"); F-W86S-P13-005 MED (merge-order note: "irrelevant" scoped to conflict purposes; ci.yml line anchors labeled as baseline develop@e8841d76 order-dependent); F-W86S-P13-007 LOW (TypeIDs overclaim: "exercises TypeIDs 58–64" → "exercises timed control-command TypeIDs 58/59/61/63 (of the 58–64 detection range)" at 3 loci including permanent README row); F-W86S-P13-008 LOW (three move-aside procedures: added trap/\|\| true for unconditional restore — procedures 1 and 3 use trap EXIT, procedure 2 uses \|\| true (expected-failure block); rationale stated per fix); F-W86S-P13-015 LOW (if: always() → if: ${{ !cancelled() }} at all 3 loci — AC-182-004 outcome (e), CI-visible signals list, ACR — with scope note: visible after TEST failures but NOT after compile failures); NIT-4 (bare prose comments converted to gating forms at 4 loci: wc -c → stat -f%z gating with portability note; shasum → test "$(shasum ... \| cut ...)" = "<sha>" gating forms); NIT-5 (EC-003: noted that iec104.pcap / iec104-sq.pcapng additionally emit [iec104-e2e] SKIP: stderr notice from fixture_present() when their own tests run). |
| 2.2 | 2026-07-26 | story-writer | WAVE-86 PASS-12 REMEDIATION — F-W86S-P12-002 MED (Task 9 Env A discriminator block: deleted zero-SKIP assertion `test "$(grep -c "\[iec104-e2e\] SKIP:" coverage-out.txt)" -eq 0` and "ZERO SKIP lines" comment — (a) no 2>&1 so SKIP eprintln! output never reaches coverage-out.txt; (b) test_fixture_manifest_report never calls fixture_present(); discriminator is `grep -qE "Fixture coverage: 4/4"` alone; added sequential-overwrite note clarifying coverage-out.txt is overwritten by this second block); F-W86S-P12-004 MED (resolver-coupling comment at AC-182-005 :649-651 corrected: "mistyped COMMITTED_SAMPLES const causes this check to fail in CI; inverted ordering in fixture_path() is caught only on a fixture-bearing host (Task 9 Env A), because local-samples/ is absent in CI"; swept sibling locus in ACR §Shared resolver contract); F-W86S-P12-008 LOW (fixture_present() SKIP diagnostic: branched on COMMITTED_FIXTURES.contains(&filename) — committed-eligible fixtures show committed path; non-committable fixtures show local-samples path with "do not commit to tests/fixtures/ (licensing)" clause; inert-predicate disciplines preserved with concat! for needle); F-W86S-P12-009 LOW (Notes §Develop PR: sibling story note added — STORY-183 also touches ci.yml in disjoint regions: STORY-182 edits test job ~:40-47 additive step; STORY-183 edits comment lines :434/:442 + step name :462; merge order irrelevant; rebase required if both in flight); NIT-4 (AC-182-004 :479/:491, ACR :1123: N/4 prose annotated with "(currently 4 — tracks FIXTURE_MANIFEST.len())" to signal co-update obligation); NIT-5 (Task 7: explicit note added that test_fixture_manifest_report is deliberately NOT added to the pcap-mapping table — table maps tests to pcaps; manifest test is not pcap-bound; DF-TEST-CITATION-SWEEP-001 item 4 considered). |
| 2.1 | 2026-07-26 | story-writer | WAVE-86 PASS-11 REMEDIATION — F-P11-001 HIGH (fixture_present call-site count assertion: replaced `harness_src.matches("fixture_present(\"")` with concat!-built needle so file's own prose cannot match it; rewritten comments eliminate all contiguous needle occurrences; justification updated to cite source-self-scanning hazard; concrete failure conditions (a)/(b)/(c) stated); F-P11-002 MED (factory-artifacts branch: gate-entry Arch Mapping row branch develop→factory-artifacts; FSR row annotated "factory-artifacts branch — committed by state-manager, NOT in develop PR"; Notes §Develop PR removes gate-entry doc from develop list + adds factory-artifacts note; traces_to entry annotated with inline comment); F-P11-003 MED (tdd_mode note re-anchored: :669-674→:740-745 post-edit lines; reworded as per-story claim, not epic-wide assertion; "do not reorder tasks" absolutism removed); F-P11-004 MED (ACR §Constants placement: FIXTURE_GATED_TESTS added as third mandatory module-level const alongside FIXTURE_MANIFEST and COMMITTED_FIXTURES); F-P11-008 LOW (duplicate Arch Mapping rows merged: FIXTURE_GATED_TESTS registry-detail description folded into the three-constants row; redundant row deleted; row count reduced by 1); F-P11-009 LOW (EC-008 added: harness self-read via CARGO_MANIFEST_DIR requires source tree; accepted for this repo's CI; no #[ignore] added); F-P11-010 LOW (sha256 dissect-download gate clause relocated from AC-182-003 to AC-182-002 with one-line cross-reference in AC-182-003); F-P11-011 LOW (Task 10a CLAUDE.md row extended: wave-gate-entry obligation stated — count below committed-partition size BLOCKS gate entry). |
| 2.0 | 2026-07-26 | story-writer | WAVE-86 PASS-10 REMEDIATION — F-P10-001 MED (.gitignore entry for coverage-out.txt added to ALL structural loci: Background §Gitignore rewritten (coverage-out.txt IS required; committed pcaps naturally tracked separately); .gitignore row added to Architecture Mapping + FSR; .gitignore added to traces_to + Task 11 PR file list + Notes §Develop PR); F-P10-003 MED (two `grep -c ... \|\| true` non-gating forms replaced with SIGPIPE-safe tee-to-file gating pattern at both loci: AC-182-003 :441-445 + Task 9 Env A :932-937; pattern widened to `\[iec104-e2e\] SKIP:` to catch any skip, not only diverse fixture); F-P10-004 MED (ACR §Shared resolver contract scoped: "for RESOLVING OR OPENING" added; display-only path construction in fixture_present() explicitly permitted and expected); F-P10-005 MED (AC-182-005 forbidden-committed negative guard added: for every FIXTURE_MANIFEST entry NOT in COMMITTED_FIXTURES assert !committed_dir.join(name).exists() with licensing/redistribution panic message; SKIP-diagnostic prose note added in fixture_present() code — committed_path displayed is for committed-eligible fixture only); F-P10-006 LOW (Size-gate :391-394 + EC-004 :705 marked UNREACHABLE-IN-PRACTICE given 14 KB recorded size; IF branch ever exercised, co-updates stated: re-record sha256 in E2E-PCAPS.md, re-derive finding counts, update README attribution); F-P10-007 LOW (ci.yml grep denominator loosened at 3 loci: `[1-9]/4` → `[1-9][0-9]*/[0-9]+` at :497/:499/:1039; Task 9 Env A 4/4 retained as env-specific expected value but added to co-update list; FIXTURE_MANIFEST.len() assertion message extended to name ci.yml + Task 9 Env A as co-update loci); F-P10-008 LOW (Background :110-113 overclaim fixed: "ALL committed fixtures" → "all fixtures committed BY THIS STORY"); F-P10-009 LOW (Manifest-size drift pin comment tempered: "genuine per-test coupling" → registry catches renames, not unregistered additions; NEW assertion added: `assert_eq!(harness_src.matches("fixture_present(\"").count(), FIXTURE_GATED_TESTS.len(), ...)` — fails when a new fixture-gated test is added without registering); F-P10-010 LOW (tdd_mode: strict E-11 template note added in Notes). |
| 1.9 | 2026-07-26 | story-writer | WAVE-86 PASS-9 REMEDIATION — F-P9-001 HIGH (COMMITTED_FIXTURES line 264 two-entry residue fixed: `["iec104-iti-diverse.pcap"]` — single entry only; iec104-iti-dissect.pcap is not committed per F-009 D-524 ruling); F-P9-002 HIGH (include_str!(file!()) removed — does not compile); F-P9-003 HIGH (per-test name coupling replaced with non-self-referential mechanism: std::fs::read_to_string reads harness source at test time and asserts fn <name> exists — checked content and predicate source are different text spans, so renaming a test without updating registry causes failure); F-P9-004 HIGH (three loci with retired pre-D-524 discriminator "provenance unverifiable" updated to D-524 discriminator: POSITIVE EVIDENCE OF UPSTREAM-OF-ITI ORIGIN — filename TestDissectIec104.pcap + E2E-PCAPS.md "Wireshark-dissector test capture"; lines :134, :193-196, :385, :510); F-P9-005 HIGH (Task 7 E2E-PCAPS.md locus + FSR row: "ITI IEC-104 captures" → "iec104-iti-diverse.pcap" singular — dissect remains gitignored, not committed); F-P9-006 MED (dissect sha256 gate reinstated for CI download-and-verify path: DISSECT_SHA added to Task 1 Step 1c using E2E-PCAPS.md:359 value 292c18a8…; AC-182-003 note added); F-P9-007 MED (AC-182-004 verification item 1: WITHOUT-LOCAL-SAMPLES precondition added explicitly); F-P9-008 MED (FIXTURE_GATED_TESTS added to Architecture Mapping row + FSR description; E2E-PCAPS.md added to Architecture Mapping table); F-P9-011 LOW (Task 10/11: add coverage-out.txt to .gitignore — gitignored transient artifact from CI step); NIT-04 (grep confirmed: no underscore-form iec104_iti_diverse.pcap anywhere — canonical hyphen form already used throughout; no text change needed). |
| 1.8 | 2026-07-26 | story-writer | WAVE-86 PASS-8 REMEDIATION — F-001 HIGH (AC-182-002 provenance rationale rewritten: dissect excluded for POSITIVE EVIDENCE OF UPSTREAM-OF-ITI ORIGIN — upstream filename TestDissectIec104.pcap + E2E-PCAPS.md "Wireshark-dissector test capture" indicate Wireshark dissector test suite origin; diverse excluded for NO indication of non-ITI origin; rule: "repo-level license suffices ABSENT contrary indication of third-party upstream origin"); F-002 HIGH (:125 "Both ITI CC-BY-4.0 captures are MANDATORY committed"→"Only iec104-iti-diverse.pcap is the MANDATORY committed fixture"); F-003 HIGH (Task 7 :793-798 restricted to diverse-only annotation; explicit instruction added: leave :503-504 and dissect mapping row (:28) unchanged — dissect remains gitignored); F-004 MED (cardinality sweep: :52/:54 captures→capture; :53 are→is; :59 committed captures→committed capture; :602-603 either committed capture→the committed capture; :610 add iec104-iti-dissect.pcap to gitignored list; :657 Either ITI capture→The committed ITI capture; :791-792 captures now have→capture now has + 2 new ITI entries→1 new ITI entry; :953 provenance rows→provenance row); F-005 MED (SIGPIPE fix: 3 `\| tee /dev/stderr \| grep -q` pipelines at :474-478/:845-849/:928-931 replaced with safe two-step: cargo\|tee coverage-out.txt then grep on file); F-006 MED (Env A regex :845-849 changed to grep -qE "Fixture coverage: 4/4" + zero-SKIP assertion; CI step + Env B keep [1-9]/4); F-007 MED (include_str!(file!()) coupling assertion added to test_fixture_manifest_report() — registered test_names must exist in source file); F-008 MED (Task 1 precondition reworded: "absent in a fresh clone; may already be present on fixture-bearing host"; each curl gated: only fetch if file missing OR sha256 mismatches; fetch to temp path + move after hash verification); F-013 LOW (:999 "state-manager adds it"→"already present in STATE.md (DRIFT-e2e-sibling-harnesses, D-522); no action in this story's PR"). |
| 1.7 | 2026-07-26 | story-writer | WAVE-86 PASS-7 REMEDIATION — F-001 HIGH (add `-- --exact` to missed sites :612 and :813; all 16+4 sites now module-qualified with --exact); F-007 MED (FIXTURE_GATED_TESTS assertion made bidirectional: direction 2 manifest⊆gated added + assert_eq!(FIXTURE_GATED_TESTS.len(), 4) count pin); F-008 MED (ci.yml step: `set -o pipefail` added to code blocks at :477 and :865; grep strengthened to `grep -qE "Fixture coverage: [1-9]/4"` at all 3 active sites; ACR updated to state step GATES the test job); F-009 MED ORCHESTRATOR RULING (COMMITTED_FIXTURES=1 entry only — iec104-iti-dissect.pcap stays gitignored; sweep: fixture count anchors 2/4→1/4 everywhere, AC-182-002 rewritten to cover only iec104-iti-diverse.pcap + provenance rationale for dissect exclusion, AC-182-003 dissect test block removed, AC-182-004 "two ITI"→"one ITI" + 2/4→1/4 output, AC-182-005 COMMITTED_FIXTURES const 1 entry + FIXTURE_MANIFEST comment 1 committed ITI, Architecture Mapping dissect row removed, FSR dissect row removed, Task 1 dissect step moved to local-samples-only with exclusion note, Task 2 single copy only, Task 9 Env B 1/4, Task 11 PR evidence 1/4, EC-003 updated, Background §fixture counts updated); F-011 LOW (EC-001 reworded: fixture_path() may return Some(local-samples/...) when corpus fetched; hard-assert fires via DIRECT Path::exists() on tests/fixtures/ regardless). |
| 1.6 | 2026-07-25 | story-writer | WAVE-86 PASS-6 REMEDIATION — F-001 HIGH (Task 1 extended: Steps 1e+1f curl Wireshark pair iec104.pcap+iec104-sq.pcapng into local-samples with sha256 verification from bin/fetch-e2e-pcaps; Env A stays 4/4; "avoids ~400 MB corpus" accurate — 4 small files only); F-002 HIGH (Narrative "structurally prevented" scoped to IEC-104 harness; Notes §Sibling e2e harnesses (deferred) added — enip_e2e_real_pcaps_tests.rs+bc_2_12_011_story127_tests.rs+e2e_corpus_smoke_tests.rs deferred wave-86 scope containment; DRIFT-e2e-sibling-harnesses STATE.md row noted); F-003 MED (assert_eq comment relabeled "manifest-size drift pin (fires on manifest growth/shrink only)"; FIXTURE_GATED_TESTS module-level const registry added with all 4 fixture-gated tests mapping (test_name,fixture_name); manifest test iterates and asserts each fixture_name in FIXTURE_MANIFEST); F-004 MED (--exact added to all single-test cargo test filters; \| grep -E "1 passed" piped to pass-assertions; ci.yml step additionally \| tee /dev/stderr \| grep -q "Fixture coverage:"; sweep AC-182-001/003/004/005+Task 9 all 4+ACR); F-012 MED (input-hash NOTE reworded: "Stored value is the canonical Python hash (9a0f34c); the bash hook reports a divergent value (0a1812a) — advisory only per PG-HASH-HOOK-DIVERGENCE"); F-014 LOW (ci.yml step spec: if: always() added; placement AFTER main cargo test step noted; "permanently CI-visible" → "visible on every run incl. after test failures (if: always())"); F-015 LOW ("comment-only additive step" → "additive run step"; "comment-style step" → "executable run step"); F-016 LOW (test_fixture_manifest_report() placement mandated inside mod iec104_e2e_real_pcaps in AC-182-001 and Task 6 — DF-TEST-NAMESPACE-001). |
| 1.5 | 2026-07-25 | story-writer | WAVE-86 PASS-5 REMEDIATION — F-001 HIGH (AC-182-005 resolver coupling: `starts_with(committed_dir)` replaced with `assert_eq!(resolved.parent(), Some(committed_dir.as_path()))` — catches local-samples/ as a subdir of tests/fixtures/ where starts_with was vacuous); F-011 MED (Task 7 README:41-44 reword: "the ITI IEC-104 captures now have recorded provenance; the remaining pre-README fixtures do not"); F-012 MED (AC-182-005: FIXTURE_MANIFEST exhaustiveness assertion `assert_eq!(FIXTURE_MANIFEST.len(), 4, ...)` restored — satisfies df-validation manifest-derived expected-count requirement); F-013 MED (Task 1 redesigned: direct curl fetches of ITI URLs + shasum -a 256 verification replace bare bin/fetch-e2e-pcaps ~400 MB download; Env A repointed to 4/4 case — manifest-report shows 4/4 and zero SKIP lines including Wireshark pair); F-015 MED (Architecture Mapping: 3 rows added for ci.yml additive step, CLAUDE.md reference row, .factory/maintenance/fixture-count-gate-entry.md; Notes §Develop PR updated with full file list and CHANGELOG non-obligation rationale; traces_to frontmatter propagated); F-016 MED (AC-182-002 license precondition added: upstream ITI/ICS-Security-Tools LICENSE.md = CC-BY-4.0 per GitHub API 2026-07-25; README provenance row must cite "upstream LICENSE.md (CC-BY-4.0, repo-level)"); F-023 LOW ("module level" replaced with "inside `mod iec104_e2e_real_pcaps` (alongside the existing `use std::path::Path` at :39)" everywhere it appeared); F-024 LOW (grep -c pipelines: `\|\| true` appended, "Must output 0" → "Expected output: 0", 4 sites); F-026 LOW (Task 8 gate-entry artifact: enforceable wave-gate G1 obligation stated — run manifest test + record N/M; count below committed-partition size BLOCKS gate entry; no policies.yaml entry per sibling convention); F-028 NIT (Task 7 E2E-PCAPS.md:48-50 sweep item reworded: removed "modbus-write as THE committed exception" overclaim). |
| 1.4 | 2026-07-25 | story-writer | WAVE-86 PASS-4 REMEDIATION — F-001 HIGH (AC-182-005 augmented: fixture_path() resolver coupling added — for every COMMITTED_FIXTURES entry assert fixture_path(name).is_some() AND resolved path is under tests/fixtures/; couples resolver to committed set so mistyped COMMITTED_SAMPLES or inverted ordering fails CI); F-003 HIGH (Task 1 redesigned: bin/fetch-e2e-pcaps prescribed as explicit first step because local-samples is ABSENT on develop; Task 2: sha256 integrity verification added using E2E-PCAPS.md:358-359 hashes; AC-182-002 Verification: sha256 checks added); F-011 MED (Narrative + Notes:776-777 softened: "cannot recur" overclaim removed; softer claim: stale-expectation failure now fails CI on every run; wrong-content class remains implementer obligation); F-012 MED (Background: 21→25 existing committed captures, line 120 and 122 propagated; ACR verification command corrected: `git ls-files tests/fixtures/ \| grep -cE` replaces `*.pcap*` glob that misses .cap/.trace files); F-013 MED (loop deduplication: canonical COMMITTED_FIXTURES hard-assert loops live in AC-182-005 only; AC-182-001 F-010 block replaced with reference to AC-182-005; FIXTURE_MANIFEST.contains check merged into AC-182-005 as canonical location); F-014 MED (ORCHESTRATOR RULING — ci.yml additive step: new step in test job `cargo test --test iec104_e2e_real_pcaps_tests iec104_e2e_real_pcaps::test_fixture_manifest_report -- --nocapture` named "IEC-104 fixture coverage report (visible)"; .github/workflows/ci.yml removed from Forbidden; added to FSR as Modify (additive step only); AC-182-004 updated to assert report IS CI-visible via this step); F-015 MED [process-gap] (new Task added: append CLAUDE.md Project References row for .factory/maintenance/fixture-count-gate-entry.md; CLAUDE.md added to FSR as Modify); F-016 MED (AC-182-003/Task 9 Env A non-vacuous: `--nocapture 2>&1 \| grep -c "\[iec104-e2e\] SKIP: fixture 'iec104-iti-"` must output 0 added after each ITI test invocation); F-022 LOW (Task 7 sweep extended: tests/fixtures/README.md:41-44 provenance-not-recorded clause; tests/iec104_e2e_real_pcaps_tests.rs:23-28 mapping table; per-test license comments :353-354 and :503-504); F-023 LOW (.gitignore removed from traces_to — no .gitignore change required per Background). |
| 1.3 | 2026-07-25 | story-writer | WAVE-86 PASS-3 REMEDIATION — F-005 HIGH (AC-182-005 hard-assert predicate corrected: `fixture_path()` is INERT on fixture-bearing hosts—replaced with direct `Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name).exists()` check; manual verification note updated); F-006 HIGH (two-environment verification protocol added to AC-182-001 Background/Task 9/Task 10; explicit precondition "WITHOUT local-samples" on every 2/4 fixture-count assertion; `mv tests/fixtures/local-samples /tmp/ls-bak` protocol prescribed); F-009 MED (#[ignore] rejection rationale corrected: libtest DOES report ignored tests—real reason is committed-fixture absence = broken checkout that MUST FAIL, not a reportable skip; #[ignore] is static and cannot be conditional on fixture presence without nightly harnesses; deliberate divergence from df-validation #[ignore] recommendation recorded; Task 8 gate-entry artifact text propagated); F-010 MED (manifest-count sanity assertions made non-self-referential: assert every COMMITTED_FIXTURES entry exists on disk in tests/fixtures/ via Path::exists(); assert FIXTURE_MANIFEST covers every name passed to fixture_present() via a module-level const used by both); F-011 MED (AC-182-002 + Task 2 attribution: license URI added https://creativecommons.org/licenses/by/4.0/; "No modifications made."→"Renamed from 090813_diverse.pcap / TestDissectIec104.pcap; content unmodified."); F-012 MED (Task 7 E2E-PCAPS.md sweep extended to ALL stale loci: :3-6 document-scope claim, :48-50 modbus-write exception, :391-396 Adding-a-capture procedure; note added: bin/fetch-e2e-pcaps dual-presence expected, cite F-005 direct-path predicate as guard); F-015 LOW (AC-182-002 verification: `git ls-files`→`git ls-files --error-unmatch`); F-016 LOW ("21 existing committed captures"→"25 capture files on disk (verify-at-implementation)"). |
| 1.2 | 2026-07-25 | story-writer | WAVE-86 PASS-2 REMEDIATION — F-003 HIGH (struck all "loud"/"visible in CI" claims for println!() stdout; documented --nocapture requirement and libtest capture semantics; AC-182-004 updated with advisory note; #[ignore] rejection rationale added to Background and Architecture Compliance); F-004 HIGH (AC-182-001 adds manifest-count sanity assertions: FIXTURE_MANIFEST.len()==4, COMMITTED_FIXTURES.len()==2; Task 8 gate-entry artifact added with #[ignore] rejection rationale; D-510 G1 retrospective documented); F-007 MED (Notes corrected: D-510 G1 FAIL was stale-assertion on fixture-bearing host at 31 vs 66, NOT clean-worktree silent skip; clean-worktree gap was separate concern addressed structurally by this story); F-008 MED (Narrative fixed: "31 tests ran out of 66 expected" → "test asserting 31 findings when 66 is now the correct expectation" — 31/66 are FINDING counts, not test counts); F-009 MED (AC-182-001 fixture_present() snippet updated to include path.display() via committed_path.display() while mentioning both search locations; Task 4 aligned); F-010 MED (tests/fixtures/local-samples/README.md removed from FSR and Task 7 — this file is gitignored/untracked; Task 7 updated to remove the local-samples/README.md sweep); F-012 MED (AC-182-002 MUST NOT commit line cites fixed: :25/:26 → :138/:273 per actual test file line numbers); F-013 MED (committed-samples/ directory concept DROPPED — F-013 rules files go directly in tests/fixtures/ following 21-capture convention; COMMITTED_SAMPLES="tests/fixtures"; all directory references updated throughout; FSR updated; Background §Gitignore updated; EC-001 updated; traces_to updated); F-014 MED (AC-182-003 now explicitly notes committed fixtures NEVER trigger skip path since they live in tests/fixtures/ directly); F-016 LOW (AC-182-002 and Task 2 updated with per-file direct download URLs: 090813_diverse.pcap and TestDissectIec104.pcap upstream filenames and full raw URLs); F-017 LOW (Task 7 updated to include tests/fixtures/README.md lines 7–22 licensing notice sweep for CC-BY-4.0 class); F-021 LOW (AC-182-005 now explicitly requires COMMITTED_FIXTURES at module level alongside FIXTURE_MANIFEST; both constants shown in module-level placement). |
| 1.1 | 2026-07-25 | story-writer | WAVE-86 PASS-1 REMEDIATION — F-011 CRIT (AC-182-003 complete redesign: `fixture_path(name) -> Option<PathBuf>` shared resolver used by BOTH `fixture_present()` and `run_iec104_pipeline()`); F-012 HIGH (union approach: manifest + loud skip-reporting + hard-assert partition + committed samples; AC-182-005 new); F-013 HIGH (committed fixtures absent → test FAILS visible; AC-182-005 formalises the partition); F-014 HIGH (resolved by F-012/F-013 hard-assert partition); F-015 MED (unified skip-notice format); F-016 MED (attribution via README.md provenance table); F-017 MED (sibling-prose sweep tasks); F-018 MED (test_fixture_manifest_report() returns unit); F-021 MED (iec104-iti-dissect.pcap made MANDATORY); F-022 LOW (≤100 KB gate hard); F-020 MED (inputs: set). |
| 1.0 | 2026-07-25 | story-writer | Initial authorship — wave-86 STORY-CREATION BURST (D-516). |
