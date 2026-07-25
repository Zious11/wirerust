---
document_type: story
story_id: STORY-182
epic_id: E-11
version: "1.0"
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
inputs: []
input-hash: "d41d8cd"
---

# STORY-182: E2E Fixture Manifest + Committed Representative Captures: Eliminate False-Green cargo test in Clean Worktrees

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** 86
**Points:** 4
**Priority:** P2

## Narrative

- **As a** CI system, gate reviewer, and contributor running `cargo test` in a clean worktree
- **I want** the IEC-104 ITI e2e fixture-gated tests to report visible, named skip notices
  when their corpus files are absent, AND for a set of committed representative captures to
  ensure CI genuinely exercises the timed-command detection path introduced in STORY-180
- **So that** "31 tests ran out of 66 expected" can never appear silently again — the wave-85
  gate G1 initial FAIL (D-510, PG-W85-005) is structurally prevented: absence of the full
  corpus is always loud, and the timed-command code path always runs in CI

## Behavioral Contracts

_(none — E-11 convention: governance-only story; no BCs authored; tooling-and-test change only)_

## Background

### PG-W85-005 — Gitignored machine-local e2e fixtures produce false-green `cargo test`

The IEC-104 ITI e2e tests in `tests/iec104_e2e_real_pcaps_tests.rs` discover their fixture
files at runtime using the `fixture_present()` helper (line 63). When a fixture is absent,
the function prints to `stderr` and returns `false`; the calling `#[test]` function returns
early. From the Rust libtest harness's perspective, the test **passed** — no panic, no skip
marker, no ignored count. The discrepancy is invisible until a reviewer runs `cargo test` on a
fixture-bearing host and sees a different total count.

Wave-85 gate G1 initially failed (D-510, 2026-07-24) because the gate evaluated counts on
the fixture host (66 timed tests running) while CI and clean worktrees silently ran only 31.
Gate-fix PR #439 (0ab6f52e) updated the expectations after STORY-180 added timed command
detection. The underlying false-green infrastructure was left in place.

**Root cause validation (DF-VALIDATION-001, df-validation-2026-07-25.md §PG-W85-005):**
Confirmed LOCAL-CARRY-FORWARD, HIGH confidence. Three candidate fixes were evaluated:
(a) gate-entry fixture-count sweep; (b) fixture manifest with loud skip reporting;
(c) committed small representative fixtures. DF-VALIDATION-001 research confirmed that
`cargo-nextest --no-tests=fail` detects *zero* tests but not *fewer-than-expected* tests,
so a manifest-based approach is the correct resolution. Recommendation: combine (b) and (c).

**License scope for committed captures:**
- `iec104-iti-diverse.pcap` — ITI/ICS-Security-Tools, CC-BY-4.0. Redistribution permitted
  with attribution. Attribution: "ICS Security Tools, Illinois Institute of Technology (ITI)."
- `iec104-iti-dissect.pcap` — ITI/ICS-Security-Tools, CC-BY-4.0. Same terms.
- `iec104.pcap` — Wireshark Foundation public sample. Comment in test file reads "not
  redistributed" — do NOT commit.
- `iec104-sq.pcapng` — Wireshark Foundation public sample. Same "not redistributed" caveat —
  do NOT commit.

**Committed-samples placement:**
The gitignore entry `/tests/fixtures/local-samples/` covers only that specific directory.
`tests/fixtures/committed-samples/` is NOT covered — files placed there are tracked normally
by git. No `.gitignore` change is required for committed samples.

## Acceptance Criteria

### AC-182-001 (traces to PG-W85-005 — fixture manifest with loud skip reporting)

A fixture manifest mechanism is introduced in `tests/iec104_e2e_real_pcaps_tests.rs` such
that fixture-absent runs are **never silent**.

- Given the current `fixture_present()` function (line 63) only prints to `stderr` and
  returns silently when a fixture is absent (making `cargo test` report the test as "passed"
  with no indication that 0 assertions ran)
- When a new `FIXTURE_MANIFEST` constant (type `&[&str]`) is added inside
  `mod iec104_e2e_real_pcaps` listing all expected fixture filenames:
  `["iec104.pcap", "iec104-sq.pcapng", "iec104-iti-diverse.pcap", "iec104-iti-dissect.pcap"]`
- And a new `#[test]` function `test_fixture_manifest_report()` is added that:
  (a) Iterates every filename in `FIXTURE_MANIFEST`
  (b) For each absent entry, prints to **stdout** (not stderr): `FIXTURE-SKIPPED: '{name}' absent — corpus test will not run (check tests/fixtures/local-samples/ or tests/fixtures/committed-samples/)`
  (c) At the end, prints a one-line summary: `Fixture coverage: N/M fixtures present (K fixture-gated tests will be skipped)` where N = present, M = total manifest entries, K = absent count
  (d) Always returns `Ok(())` (never fails — absence of gitignored corpus is expected on CI)
- And `fixture_present()` is updated to also print a stdout line (not just stderr) of the form
  `FIXTURE-SKIPPED: '{name}' not found — returning early (0 assertions run in this test)` so
  that each fixture-absent test body is individually visible in `cargo test -- --nocapture`
  output

- Then `cargo test --test iec104_e2e_real_pcaps_tests -- --nocapture` in a clean worktree
  (corpus absent) shows: `Fixture coverage: 2/4 fixtures present (2 fixture-gated tests will
  be skipped)` (or similar — exact numbers depend on which committed samples land; at minimum
  the two ITI files must be present, raising the cover count to ≥2/4)

Verification:
```bash
# In a repo with only committed-samples populated, confirm test_fixture_manifest_report runs
cargo test --test iec104_e2e_real_pcaps_tests test_fixture_manifest_report -- --nocapture
# Must print "Fixture coverage: N/4 fixtures present" to stdout
```

### AC-182-002 (traces to PG-W85-005 — committed representative captures)

Committed ITI CC-BY-4.0 representative captures are added under
`tests/fixtures/committed-samples/` so CI genuinely exercises the timed-command detection
path from STORY-180 (BC-2.19.029 / BC-2.19.030).

- Given `iec104-iti-diverse.pcap` exercises TypeIDs 58–64 (time-tagged control commands,
  BC-2.19.029/030) and produces 66 findings with STORY-180 detection enabled (31 before
  wave-85); and given it is licensed ITI/ICS-Security-Tools CC-BY-4.0 allowing redistribution
  with attribution
- When the implementer verifies that `iec104-iti-diverse.pcap` from the local corpus is
  **≤ 100 KB** (a reasonable threshold for a 173-packet single-flow capture) and is the
  canonical ITI ICS-Security-Tools capture file
- And the implementer copies it to `tests/fixtures/committed-samples/iec104-iti-diverse.pcap`
  and commits it
- And `tests/fixtures/committed-samples/ATTRIBUTION.md` is added with verbatim CC-BY-4.0
  attribution text: "ICS Security Tools, Illinois Institute of Technology (ITI). Licensed
  under CC-BY-4.0 (https://creativecommons.org/licenses/by/4.0/)." plus a link to the
  original corpus (https://github.com/ITI/ICS-Security-Tools)
- And optionally `iec104-iti-dissect.pcap` (6-flow dissector capture, also ITI CC-BY-4.0,
  also likely ≤ 100KB) is similarly committed

- Then `git ls-files tests/fixtures/committed-samples/` shows the committed files and
  `git diff --stat` for a fresh clone shows them as tracked

**MUST NOT commit:**
- `iec104.pcap` (Wireshark Foundation "not redistributed" — see test-file header)
- `iec104-sq.pcapng` (Wireshark Foundation "not redistributed" — see test-file header)

Verification:
```bash
# Confirm committed-samples directory and attribution are tracked
git ls-files tests/fixtures/committed-samples/
# Must list: iec104-iti-diverse.pcap, ATTRIBUTION.md (and optionally iec104-iti-dissect.pcap)

# Confirm size constraint
wc -c tests/fixtures/committed-samples/iec104-iti-diverse.pcap
# Must be <= 102400 bytes (100 KB)
```

### AC-182-003 (traces to PG-W85-005 — fixture_present() checks committed-samples)

`fixture_present()` is updated to check `tests/fixtures/committed-samples/` in addition to
`tests/fixtures/local-samples/`, so committed captures are found automatically and their
corresponding tests run on every `cargo test` invocation (including CI).

- Given committed captures reside in `tests/fixtures/committed-samples/` and the current
  `fixture_present()` only looks in `const LOCAL_SAMPLES: &str = "tests/fixtures/local-samples"`
- When a `const COMMITTED_SAMPLES: &str = "tests/fixtures/committed-samples"` is added
  alongside the existing `LOCAL_SAMPLES` constant
- And `fixture_present()` is updated to check `COMMITTED_SAMPLES` before `LOCAL_SAMPLES`:
  ```rust
  fn fixture_present(filename: &str) -> bool {
      // Check committed-samples/ first (always present in git)
      let committed = Path::new(env!("CARGO_MANIFEST_DIR"))
          .join(COMMITTED_SAMPLES).join(filename);
      if committed.exists() { return true; }
      // Fall back to gitignored local-samples/ corpus
      let local = Path::new(env!("CARGO_MANIFEST_DIR"))
          .join(LOCAL_SAMPLES).join(filename);
      if local.exists() { return true; }
      // Fixture absent: print loud skip notice (stdout for cargo test visibility)
      println!("FIXTURE-SKIPPED: '{}' not found in committed-samples or local-samples — \
               0 assertions will run in this test. Run `bin/fetch-e2e-pcaps` for full corpus.",
               filename);
      false
  }
  ```
- And no test assertions change — `run_iec104_pipeline()` is unchanged; the test bodies
  continue to use the same filename argument (e.g., `"iec104-iti-diverse.pcap"`)

- Then `cargo test test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu` runs
  (does not silently skip) on any host where `tests/fixtures/committed-samples/iec104-iti-diverse.pcap`
  exists — which includes every clean checkout of the repo

Verification:
```bash
# Confirm the test runs and passes (not a silent skip) on a clean worktree
cargo test --test iec104_e2e_real_pcaps_tests \
  test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu
# Must PASS (not skip): exercises TypeID 58-64 timed-command detection (BC-2.19.029/030)
# Expected outcome: 66 findings (20 T0836 + 46 T1692.001 + 35 time-tagged; STORY-180 ground-truth)
```

### AC-182-004 (traces to PG-W85-005 — regression: clean-worktree skip count is visible)

A clean-worktree run (only committed-samples present, local-samples absent) must:
(a) Run the two ITI fixture tests that have committed captures
(b) Skip the two Wireshark fixture tests (iec104.pcap, iec104-sq.pcapng) LOUDLY
(c) Report via `test_fixture_manifest_report()`: `Fixture coverage: 2/4 fixtures present
    (2 fixture-gated tests will be skipped)`

This AC is regression-only (no new code beyond AC-182-001..003) — it specifies the combined
observable outcome that guards against future regressions.

Verification:
```bash
# Simulate clean worktree by temporarily removing local-samples
# (or just run on a fresh git clone with committed-samples present)
cargo test --test iec104_e2e_real_pcaps_tests -- --nocapture 2>&1 | grep -E "FIXTURE-SKIPPED|Fixture coverage"
# Must print:
#   FIXTURE-SKIPPED: 'iec104.pcap' not found ...
#   FIXTURE-SKIPPED: 'iec104-sq.pcapng' not found ...
#   Fixture coverage: 2/4 fixtures present (2 fixture-gated tests will be skipped)
# (exact filenames depend on whether iec104-iti-dissect.pcap is also committed)
```

## Architecture Mapping

| Component | File | Branch |
|-----------|------|--------|
| FIXTURE_MANIFEST constant + test_fixture_manifest_report() | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| COMMITTED_SAMPLES constant + fixture_present() update | `tests/iec104_e2e_real_pcaps_tests.rs` (amend) | develop |
| Committed ITI capture (timed-command, BC-2.19.029/030) | `tests/fixtures/committed-samples/iec104-iti-diverse.pcap` (new) | develop |
| Committed ITI capture (multi-flow dissector coverage) | `tests/fixtures/committed-samples/iec104-iti-dissect.pcap` (new, optional) | develop |
| CC-BY-4.0 attribution for committed ITI captures | `tests/fixtures/committed-samples/ATTRIBUTION.md` (new) | develop |

**No `src/` changes, no `bin/` changes, no `Cargo.toml` changes.**
CHANGELOG obligation: `tests/` is **excluded** from the AC-158-001 changelog-gate trigger set
(`src/`, `Cargo.toml`, `bin/` only). **No CHANGELOG entry required.**

No new `bin/test_*.py` files — L-W84-003 / AC-165-001 CI-wiring obligation does not apply.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `tests/fixtures/committed-samples/` directory absent (freshly cloned repo before committed samples land) | `fixture_present()` falls through to local-samples check, then prints FIXTURE-SKIPPED; `test_fixture_manifest_report()` reports 0/4 fixtures present — correct behavior |
| EC-002 | `iec104-iti-diverse.pcap` exists in both committed-samples and local-samples | fixture_present() returns true on the first (committed-samples) check; no double-run; safe |
| EC-003 | Wireshark Foundation captures NOT committed (intentional) | Only the two ITI CC-BY-4.0 files are in committed-samples; the two Wireshark files are SKIPPED on clean worktrees — this is the expected state, not a defect |
| EC-004 | iec104-iti-diverse.pcap size > 100 KB at time of commit | AC-182-002 size check BLOCKS commit — implementer must find a smaller derivative or document the size exception with an updated threshold |
| EC-005 | test_fixture_manifest_report() failure count check | This test NEVER fails (regardless of how many fixtures are absent) because the local-samples corpus is gitignored by design; a failing test_fixture_manifest_report would block CI on every clean checkout |
| EC-006 | pipeline_run() called with committed-samples path but local-samples path used by run_iec104_pipeline | run_iec104_pipeline() constructs the path from LOCAL_SAMPLES; AC-182-003 adds COMMITTED_SAMPLES check only in fixture_present(); the pipeline itself is path-agnostic (CARGO_MANIFEST_DIR base); no path conflict |

## Tasks

1. **Verify file sizes and licenses (AC-182-002 pre-commit check):** Run
   `wc -c tests/fixtures/local-samples/iec104-iti-diverse.pcap` and
   `wc -c tests/fixtures/local-samples/iec104-iti-dissect.pcap` on a fixture-bearing host.
   Confirm both are ≤ 100 KB. Confirm CC-BY-4.0 license from
   https://github.com/ITI/ICS-Security-Tools. If either exceeds 100 KB, re-evaluate and
   document the size decision.

2. **Create committed-samples directory and attribution file (AC-182-002):**
   `mkdir -p tests/fixtures/committed-samples/` and write `ATTRIBUTION.md` with CC-BY-4.0
   attribution. Copy `iec104-iti-diverse.pcap` (and optionally `iec104-iti-dissect.pcap`)
   from local-samples into committed-samples.

3. **Update fixture_present() and add COMMITTED_SAMPLES constant (AC-182-003):** Add
   `const COMMITTED_SAMPLES` and update the two-stage path check. Add stdout println for
   skip notice (visible in `cargo test -- --nocapture`).

4. **Add FIXTURE_MANIFEST and test_fixture_manifest_report() (AC-182-001):** Add the
   manifest constant and the always-running manifest status test. Verify the test prints the
   fixture coverage summary.

5. **Regression verification (AC-182-004):** Run `cargo test --test iec104_e2e_real_pcaps_tests`
   and confirm: (a) the ITI tests PASS (not skip) with committed captures present; (b) the
   manifest report prints the expected coverage summary; (c) the Wireshark-fixture tests
   print FIXTURE-SKIPPED.

6. **Develop PR:** All changes are in `tests/` and committed fixtures — no CHANGELOG required.
   Batch all four ACs in a single develop PR.

> **Note for implementer:** Task 1 MUST run on the fixture-bearing host before committing.
> The size check (≤100 KB) is a hard gate for AC-182-002. If `iec104-iti-diverse.pcap`
> exceeds 100 KB on your local corpus, document the actual size in the PR description and
> propose an updated threshold. Do NOT commit a file that was not verified for size.
> The two Wireshark Foundation samples (iec104.pcap, iec104-sq.pcapng) MUST NOT be committed
> (see Background §License scope).

## Previous Story Intelligence

- **STORY-176 (wave-84):** Established the pattern for E-11 governance stories: no BCs,
  no Rust source changes, `tdd_mode: strict`, CHANGELOG only when `bin/` is touched. STORY-182
  follows the same convention (tests/ changes only, no CHANGELOG).
- **STORY-180 (wave-85, BC-2.19.029/BC-2.19.030):** Delivered timed-command detection for
  TypeIDs 58–64. Updated `test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu`
  to expect 66 findings (was 31 before timed detection). The committed `iec104-iti-diverse.pcap`
  in AC-182-002 must produce the wave-85-updated expectation: T0836 ×20 + T1692.001 ×46 = 66.
- **Gate-fix PR #439 (0ab6f52e, wave-85):** Updated ITI diverse expectations from 31→66 after
  initial gate G1 FAIL. STORY-182 closes the structural gap that made gate G1 fail silently
  in the first place.

## Architecture Compliance Rules

- **No src/ modification:** This story is test-infrastructure only. No production code,
  no analyzer changes, no dispatcher changes. If any src/ file needs to change, stop and
  escalate.
- **COMMITTED_SAMPLES path (tests/fixtures/committed-samples/):** This directory is NOT
  covered by any `.gitignore` entry (`.gitignore` only ignores `/tests/fixtures/local-samples/`).
  No `.gitignore` amendment required for STORY-182.
- **test_fixture_manifest_report() must always pass:** This test reports corpus availability
  but MUST NOT fail when corpus is absent. A failing test would cause CI red on clean checkouts.
  The test body must always `return Ok(())` (or equivalent Rust test pass).
- **Stdout for skip notices:** Skip notices must print to **stdout** (using `println!()`,
  not `eprintln!()`) so that `cargo test -- --nocapture` shows them. Stderr output is hidden
  by default in `cargo test` output; stdout capture requires `--nocapture` but is at least
  visible when requested.
- **Action SHA-pin policy (CI):** STORY-182 makes no `ci.yml` changes; no new SHA pins
  required.

## Library & Framework Requirements

| Dependency | Version | Source |
|------------|---------|--------|
| Rust stable | 1.91+ | CLAUDE.md MSRV |
| No new Cargo.toml deps | — | tests/ only, no new crate deps |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `tests/iec104_e2e_real_pcaps_tests.rs` | Modify | Add COMMITTED_SAMPLES const, update fixture_present(), add FIXTURE_MANIFEST const, add test_fixture_manifest_report() |
| `tests/fixtures/committed-samples/iec104-iti-diverse.pcap` | New (binary) | ITI CC-BY-4.0; ≤100 KB; exercises TypeIDs 58-64 |
| `tests/fixtures/committed-samples/iec104-iti-dissect.pcap` | New (binary, optional) | ITI CC-BY-4.0; ≤100 KB; exercises 6-flow dissector path |
| `tests/fixtures/committed-samples/ATTRIBUTION.md` | New | CC-BY-4.0 attribution for ITI corpus files |

**Forbidden modifications:** `src/**/*`, `Cargo.toml`, `bin/*`, `CHANGELOG.md`, `.github/workflows/ci.yml`

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `tests/iec104_e2e_real_pcaps_tests.rs` | effectful-shell | Performs filesystem I/O (pcap file reads, path existence checks), spawns the analyzer pipeline, and asserts on output findings. No pure-core logic is introduced by this story — only test infrastructure and fixture discovery. |
| `tests/fixtures/committed-samples/` | data artifact | Static binary captures; not a code module; no purity classification applies. |

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~3.5 k |
| `tests/iec104_e2e_real_pcaps_tests.rs` (full file, 654 lines) | ~5.0 k |
| `tests/fixtures/committed-samples/ATTRIBUTION.md` (new file) | ~0.2 k |
| Binary pcap files (not token-counted) | ~0 k |
| **Total** | **~8.7 k** |
| Agent context window | 200 k (Sonnet) |
| **Budget usage** | **~4.4%** |

Well within context window. No story split required.

## Notes

- **DF-VALIDATION-001 status:** PG-W85-005 is LOCAL-CARRY-FORWARD per
  `df-validation-2026-07-25.md` §PG-W85-005 (HIGH confidence). No upstream filing required.
  Adjacent upstream issue #694 tracks the class but does not cover this specific mechanism.
- **No behavioral contract required:** E-11 convention.
- **Develop PR:** All four ACs can be batched in a single develop PR. No CHANGELOG entry
  required (`tests/` and `tests/fixtures/` are not in the AC-158-001 trigger set).
- **Wave-85 gate G1 retrospective:** The initial FAIL (D-510) that motivated PG-W85-005 was
  caused by the ITI diverse test silently skipping in the evaluator's clean worktree while
  passing on the fixture host. Gate-fix PR #439 updated expectations from 31→66. STORY-182
  closes the structural gap so this class of false-green can no longer occur silently.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-07-25 | story-writer | Initial authorship — wave-86 STORY-CREATION BURST (D-516); PG-W85-005 fixture-manifest + committed-captures story; 4 pts; E-11; wave 86; grounded against tests/iec104_e2e_real_pcaps_tests.rs line 63 fixture_present() and ITI CC-BY-4.0 license confirmation. |
